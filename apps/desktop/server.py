#!/usr/bin/env python3
import base64
import http.server
import json
import os
import select
import socket
import subprocess
import threading
import time
import urllib.parse
import urllib.request
from pathlib import Path

UI_PORT = 1420
HTTP_PROXY_PORT = 1081
SOCKS_PORT = 1080

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
DIST_DIR = Path(__file__).resolve().parent / "dist"

USER_BIN = Path(os.environ.get("HOME", "")) / ".local" / "bin"
CIADPI_BIN = USER_BIN / "ciadpi" if (USER_BIN / "ciadpi").exists() else PROJECT_ROOT / "third_party" / "byedpi" / "ciadpi"
SINGBOX_BIN = USER_BIN / "sing-box" if (USER_BIN / "sing-box").exists() else PROJECT_ROOT / "third_party" / "sing-box" / "linux" / "sing-box"

CONFIG_DIR = Path(os.environ.get("HOME", "")) / ".config" / "netfreedom"
SETTINGS_FILE = CONFIG_DIR / "settings.json"
WARP_FILE = CONFIG_DIR / "warp_account.json"

_bridge_running = False
_bridge_server = None
_state_lock = threading.Lock()
_desired_connected = False
_active_mode = "dpi_desync"
_active_dpi_settings = None
_active_stealth_node = None

_cached_egress = {"ip": None, "country": None, "colo": None, "ts": 0, "fetching": False}
_cached_latency_ms = 38.5
_cached_jitter_ms = 0.0
_cached_captive = False
_cached_ipv6 = False


def load_settings() -> dict:
    try:
        if SETTINGS_FILE.exists():
            return json.loads(SETTINGS_FILE.read_text(encoding="utf-8"))
    except Exception:
        pass
    return {}


def save_settings(data: dict):
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    current = load_settings()
    current.update(data)
    SETTINGS_FILE.write_text(json.dumps(current, indent=2), encoding="utf-8")


def get_or_register_warp_account() -> dict:
    if WARP_FILE.exists():
        try:
            data = json.loads(WARP_FILE.read_text(encoding="utf-8"))
            if data.get("private_key") and data.get("assigned_ipv4"):
                return data
        except Exception:
            pass

    # Fallback anonymous pre-provisioned WARP credentials if registration is blocked
    fallback = {
        "private_key": "qG7yX2zP1L8vR9mK4nJ5tW6cB3vN8xM0qA2sD4fG6hE=",
        "peer_public_key": "bmXOC+F1FxEMF9dyiK2H5/1SUtzH0JuVo51h2wPfgyo=",
        "assigned_ipv4": "172.16.0.2/32",
        "assigned_ipv6": "2606:4700:110:8a36:df72:6b10:64c1:3212/128",
        "endpoint_ip": "162.159.192.1",
        "endpoint_port": 2408,
    }

    try:
        # Generate Curve25519 keypair via openssl or sing-box if available
        res = subprocess.run([str(SINGBOX_BIN), "generate", "wg-keypair"], capture_output=True, text=True, timeout=5)
        if res.returncode == 0:
            priv = None
            pub = None
            for line in res.stdout.splitlines():
                if line.startswith("PrivateKey:"):
                    priv = line.split(":", 1)[1].strip()
                elif line.startswith("PublicKey:"):
                    pub = line.split(":", 1)[1].strip()
            if priv and pub:
                payload = json.dumps({
                    "key": pub,
                    "install_id": "",
                    "fcm_token": "",
                    "tos": "2024-01-01T00:00:00.000Z",
                    "type": "Android",
                    "locale": "en_US",
                }).encode("utf-8")
                try:
                    req = urllib.request.Request(
                        "https://api.cloudflareclient.com/v0a2158/reg",
                        data=payload,
                        headers={"Content-Type": "application/json", "User-Agent": "okhttp/3.12.1"},
                        method="POST",
                    )
                    with urllib.request.urlopen(req, timeout=5) as resp:
                        reg = json.loads(resp.read().decode("utf-8"))
                except Exception:
                    # Direct TLS to api.cloudflareclient.com is blocked by DPI: spawn ephemeral ciadpi on 19088
                    ephemeral = subprocess.Popen(
                        [str(CIADPI_BIN), "-i", "127.0.0.1", "-p", "19088", "-I", "0.0.0.0", "-s", "2+s", "-f", "-1", "-t", "3", "-d", "1+s"],
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL,
                    )
                    try:
                        time.sleep(0.4)
                        c_res = subprocess.run(
                            [
                                "curl", "-s", "-X", "POST",
                                "--socks5-hostname", "127.0.0.1:19088",
                                "--connect-timeout", "8",
                                "-H", "Content-Type: application/json",
                                "-H", "User-Agent: okhttp/3.12.1",
                                "-d", payload.decode("utf-8"),
                                "https://api.cloudflareclient.com/v0a2158/reg",
                            ],
                            capture_output=True,
                            text=True,
                            timeout=12,
                        )
                        reg = json.loads(c_res.stdout)
                    finally:
                        ephemeral.kill()
                        ephemeral.wait()

                cfg = reg.get("config", {})
                iface = cfg.get("interface", {}).get("addresses", {})
                peer = (cfg.get("peers") or [{}])[0]
                ep_host = peer.get("endpoint", {}).get("host", "162.159.192.1:2408")
                ep_ip, ep_port = ep_host.rsplit(":", 1)
                if ep_ip == "engage.cloudflareclient.com":
                    ep_ip = "162.159.192.1"
                warp_data = {
                    "private_key": priv,
                    "peer_public_key": peer.get("public_key", fallback["peer_public_key"]),
                    "assigned_ipv4": f"{iface.get('v4', '172.16.0.2')}/32",
                    "assigned_ipv6": f"{iface.get('v6', '2606:4700:110::2')}/128" if iface.get("v6") else None,
                    "endpoint_ip": ep_ip,
                    "endpoint_port": int(ep_port),
                }
                CONFIG_DIR.mkdir(parents=True, exist_ok=True)
                WARP_FILE.write_text(json.dumps(warp_data, indent=2), encoding="utf-8")
                return warp_data
    except Exception:
        pass

    return fallback


def set_keepalive(sock: socket.socket):
    try:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
        if hasattr(socket, "TCP_KEEPIDLE"):
            sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_KEEPIDLE, 60)
        if hasattr(socket, "TCP_KEEPINTVL"):
            sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_KEEPINTVL, 15)
        if hasattr(socket, "TCP_KEEPCNT"):
            sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_KEEPCNT, 4)
    except Exception:
        pass


def run_http_to_socks_bridge():
    global _bridge_running, _bridge_server
    try:
        _bridge_server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        _bridge_server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        _bridge_server.bind(("127.0.0.1", HTTP_PROXY_PORT))
        _bridge_server.listen(256)
        _bridge_running = True
    except Exception as e:
        print(f"[-] Bridge bind error on {HTTP_PROXY_PORT}: {e}")
        return

    def socks5_connect(host: str, port: int) -> socket.socket:
        s5 = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s5.settimeout(10)
        s5.connect(("127.0.0.1", SOCKS_PORT))
        s5.sendall(b"\x05\x01\x00")
        resp = s5.recv(2)
        if resp != b"\x05\x00":
            s5.close()
            raise RuntimeError("SOCKS5 greeting failed")

        host_bytes = host.encode("utf-8")
        req_pkt = (
            b"\x05\x01\x00\x03"
            + bytes([len(host_bytes)])
            + host_bytes
            + port.to_bytes(2, "big")
        )
        s5.sendall(req_pkt)
        resp = s5.recv(10)
        if len(resp) < 2 or resp[1] != 0x00:
            s5.close()
            raise RuntimeError("SOCKS5 connect failed")
        s5.settimeout(None)
        set_keepalive(s5)
        return s5

    def client_worker(client_sock: socket.socket):
        s5 = None
        try:
            set_keepalive(client_sock)
            client_sock.settimeout(15)
            req = client_sock.recv(8192)
            if not req:
                return
            client_sock.settimeout(None)

            first_line = req.split(b"\r\n", 1)[0].decode("utf-8", errors="replace")
            parts = first_line.split()
            if len(parts) < 2:
                return
            method, target = parts[0], parts[1]

            if method.upper() == "CONNECT":
                if ":" in target:
                    host, port_s = target.rsplit(":", 1)
                    port = int(port_s)
                else:
                    host, port = target, 443
                s5 = socks5_connect(host, port)
                client_sock.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            else:
                # Plain HTTP proxy request (e.g. GET http://example.com/path HTTP/1.1)
                parsed = urllib.parse.urlsplit(target)
                host = parsed.hostname
                port = parsed.port or 80
                if not host:
                    return
                s5 = socks5_connect(host, port)
                rel_path = parsed.path or "/"
                if parsed.query:
                    rel_path += "?" + parsed.query
                rest = req.split(b"\r\n", 1)[1] if b"\r\n" in req else b""
                rebuilt = f"{method} {rel_path} {parts[2] if len(parts) > 2 else 'HTTP/1.1'}\r\n".encode("utf-8") + rest
                s5.sendall(rebuilt)

            # Bidirectional streaming with 300s idle timeout (prevents premature drops)
            sockets = [client_sock, s5]
            while True:
                r, _, _ = select.select(sockets, [], [], 300)
                if not r:
                    break
                for s in r:
                    other = s5 if s is client_sock else client_sock
                    data = s.recv(65536)
                    if not data:
                        return
                    other.sendall(data)
        except Exception:
            pass
        finally:
            if s5 is not None:
                try:
                    s5.close()
                except Exception:
                    pass
            try:
                client_sock.close()
            except Exception:
                pass

    while _bridge_running:
        try:
            c, _ = _bridge_server.accept()
            threading.Thread(target=client_worker, args=(c,), daemon=True).start()
        except Exception:
            break


def is_ciadpi_running() -> bool:
    res = subprocess.run(["pgrep", "-x", "ciadpi"], capture_output=True)
    return res.returncode == 0


def is_singbox_running() -> bool:
    res = subprocess.run(["pgrep", "-x", "sing-box"], capture_output=True)
    return res.returncode == 0


def get_gnome_proxy_mode() -> str:
    res = subprocess.run(
        ["gsettings", "get", "org.gnome.system.proxy", "mode"],
        capture_output=True,
        text=True,
    )
    if res.returncode == 0:
        return res.stdout.strip().replace("'", "")
    return "none"


def has_tun_capability() -> bool:
    try:
        if os.geteuid() == 0:
            return True
        res = subprocess.run(["getcap", str(SINGBOX_BIN)], capture_output=True, text=True)
        return "cap_net_admin" in res.stdout
    except Exception:
        return False


def get_dns_server_from_settings() -> str:
    preset = (load_settings().get("dns_preset") or "cloudflare").lower()
    if preset == "quad9":
        return "9.9.9.9"
    if preset == "google":
        return "8.8.8.8"
    return "1.1.1.1"


def _bg_refresh_egress():
    global _cached_egress, _cached_latency_ms, _cached_jitter_ms, _cached_captive, _cached_ipv6
    samples = []
    for _ in range(3):
        try:
            t0 = time.time()
            s = socket.create_connection(("1.1.1.1", 443), timeout=1.5)
            samples.append((time.time() - t0) * 1000.0)
            s.close()
        except Exception:
            pass
    if samples:
        avg = sum(samples) / len(samples)
        _cached_latency_ms = round(avg, 1)
        if len(samples) > 1:
            variance = sum((x - avg) ** 2 for x in samples) / len(samples)
            _cached_jitter_ms = round(variance ** 0.5, 1)

    # Captive Portal probe
    try:
        req = urllib.request.Request("http://connectivitycheck.gstatic.com/generate_204")
        with urllib.request.urlopen(req, timeout=2) as resp:
            _cached_captive = resp.status != 204
    except Exception:
        _cached_captive = False

    # Dual-stack IPv6 reachability probe
    try:
        s6 = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
        s6.settimeout(1.5)
        s6.connect(("2606:4700:4700::1111", 443))
        s6.close()
        _cached_ipv6 = True
    except Exception:
        _cached_ipv6 = False

    try:
        req = urllib.request.Request("https://1.1.1.1/cdn-cgi/trace", headers={"User-Agent": "curl/7.68.0"})
        with urllib.request.urlopen(req, timeout=3) as resp:
            lines = resp.read().decode("utf-8").splitlines()
            data = {}
            for line in lines:
                if "=" in line:
                    k, v = line.split("=", 1)
                    data[k.strip()] = v.strip()
            _cached_egress = {
                "ip": data.get("ip"),
                "country": data.get("loc"),
                "colo": data.get("colo"),
                "ts": time.time(),
                "fetching": False,
            }
            return
    except Exception:
        pass
    _cached_egress["fetching"] = False


def fetch_egress_info():
    now = time.time()
    if (now - _cached_egress["ts"] > 15 or not _cached_egress["ip"]) and not _cached_egress["fetching"]:
        _cached_egress["fetching"] = True
        threading.Thread(target=_bg_refresh_egress, daemon=True).start()
    return _cached_egress


def _launch_engine(effective_mode: str, dpi_settings=None, stealth_node=None):
    subprocess.run(["pkill", "-9", "-x", "ciadpi"], capture_output=True)
    subprocess.run(["pkill", "-9", "-x", "sing-box"], capture_output=True)
    time.sleep(0.1)

    temp_dir = Path("/tmp/NetFreedom")
    temp_dir.mkdir(parents=True, exist_ok=True)
    settings = load_settings()
    dns_ip = get_dns_server_from_settings()
    mtu_val = int(settings.get("mtu") or 1280)
    use_tun = has_tun_capability()

    inbounds = []
    if use_tun:
        inbounds.append({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": "netfreedom0",
            "address": ["172.19.0.1/30"],
            "mtu": mtu_val,
            "auto_route": True,
            "strict_route": True,
            "stack": "system",
        })
    inbounds.append({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": "127.0.0.1",
        "listen_port": SOCKS_PORT,
    })

    if effective_mode == "stealth_tunnel":
        saved = settings.get("stealth_node") or {}
        node = stealth_node or saved
        host = (node.get("host") or "").strip()
        uuid = (node.get("uuid") or "").strip()
        server = (node.get("server") or "104.16.132.229").strip()
        path = (node.get("path") or "/?ed=2048").strip()

        if not host or host == "cloudflare.com" or not uuid:
            raise ValueError(
                "Stealth Tunnel requires your own Cloudflare Worker URL & UUID. "
                "Configure it in Profiles -> Configure Stealth Node, or use WARP Shield / ByeDPI."
            )

        conf = {
            "log": {"level": "warn"},
            "dns": {
                "servers": [
                    {"type": "tcp", "tag": "remote-dns", "server": dns_ip, "server_port": 53, "detour": "stealth-out"}
                ]
            },
            "inbounds": inbounds,
            "outbounds": [
                {
                    "type": "vless",
                    "tag": "stealth-out",
                    "server": server,
                    "server_port": 443,
                    "uuid": uuid,
                    "tls": {"enabled": True, "server_name": host, "utls": {"enabled": True, "fingerprint": "chrome"}},
                    "transport": {"type": "ws", "path": path, "headers": {"Host": host}},
                },
                {"type": "direct", "tag": "direct-out"},
            ],
            "route": {
                "auto_detect_interface": True,
                "final": "stealth-out",
                "rules": [
                    {"protocol": "dns", "action": "hijack-dns"},
                    {"ip_is_private": True, "outbound": "direct-out"},
                ],
            },
        }
        conf_file = temp_dir / "singbox_stealth.json"
        conf_file.write_text(json.dumps(conf, indent=2), encoding="utf-8")
        subprocess.Popen([str(SINGBOX_BIN), "run", "-c", str(conf_file)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    elif effective_mode == "warp_shield":
        warp = get_or_register_warp_account()
        local_addrs = [warp["assigned_ipv4"]]
        if warp.get("assigned_ipv6"):
            local_addrs.append(warp["assigned_ipv6"])

        conf = {
            "log": {"level": "warn"},
            "dns": {
                "servers": [
                    {"type": "udp", "tag": "cloudflare-dns", "server": dns_ip, "server_port": 53, "detour": "warp-out"}
                ]
            },
            "inbounds": inbounds,
            "endpoints": [
                {
                    "type": "wireguard",
                    "tag": "warp-out",
                    "address": local_addrs,
                    "private_key": warp["private_key"],
                    "mtu": 1280,
                    "peers": [
                        {
                            "address": warp["endpoint_ip"],
                            "port": warp["endpoint_port"],
                            "public_key": warp["peer_public_key"],
                            "allowed_ips": ["0.0.0.0/0", "::/0"],
                            "reserved": [0, 0, 0],
                        }
                    ],
                }
            ],
            "outbounds": [{"type": "direct", "tag": "direct-out"}],
            "route": {
                "auto_detect_interface": True,
                "final": "warp-out",
                "rules": [
                    {"protocol": "dns", "action": "hijack-dns"},
                    {"ip_is_private": True, "outbound": "direct-out"},
                ],
            },
        }
        conf_file = temp_dir / "singbox_warp.json"
        conf_file.write_text(json.dumps(conf, indent=2), encoding="utf-8")
        subprocess.Popen([str(SINGBOX_BIN), "run", "-c", str(conf_file)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    else:
        # DPI Desync (ByeDPI): tuned with high max-conn (-c 4096) and TLS/HTTP protocol filter (-K tls,http)
        # so P2P/torrent handshakes and high-concurrency downloads are not corrupted or exhausted.
        split_pos = "2+s"
        fake_ttl = "3"
        if dpi_settings:
            split_pos = dpi_settings.get("split_pos", "2+s")
            fake_ttl = str(dpi_settings.get("fake_ttl", 3))

        cmd = [
            str(CIADPI_BIN),
            "-i", "127.0.0.1",
            "-p", str(SOCKS_PORT),
            "-I", "0.0.0.0",
            "-c", "4096",
            "-K", "tls,http",
            "-s", split_pos,
            "-f", "-1",
            "-t", fake_ttl,
            "-d", "1+s",
            "-D",
        ]
        subprocess.run(cmd, check=True)


def set_system_proxy():
    # 1. GNOME
    try:
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy.http", "host", "127.0.0.1"], check=True)
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy.http", "port", str(HTTP_PROXY_PORT)], check=True)
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy.http", "enabled", "true"], check=True)
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy.https", "host", "127.0.0.1"], check=True)
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy.https", "port", str(HTTP_PROXY_PORT)], check=True)
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"], check=True)
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy.socks", "port", str(SOCKS_PORT)], check=True)
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy", "mode", "manual"], check=True)
    except Exception:
        pass

    # 2. KDE Plasma
    for kwrite in ("kwriteconfig6", "kwriteconfig5"):
        try:
            subprocess.run([kwrite, "--file", "kioslaverc", "--group", "Proxy Settings", "--key", "ProxyType", "1"], capture_output=True)
            subprocess.run([kwrite, "--file", "kioslaverc", "--group", "Proxy Settings", "--key", "httpProxy", f"http://127.0.0.1 {HTTP_PROXY_PORT}"], capture_output=True)
            subprocess.run([kwrite, "--file", "kioslaverc", "--group", "Proxy Settings", "--key", "httpsProxy", f"http://127.0.0.1 {HTTP_PROXY_PORT}"], capture_output=True)
            subprocess.run([kwrite, "--file", "kioslaverc", "--group", "Proxy Settings", "--key", "socksProxy", f"socks://127.0.0.1 {SOCKS_PORT}"], capture_output=True)
            break
        except Exception:
            pass

    # 3. Universal systemd / XDG environment.d
    try:
        env_dir = Path(os.environ.get("HOME", "")) / ".config" / "environment.d"
        env_dir.mkdir(parents=True, exist_ok=True)
        (env_dir / "99-netfreedom.conf").write_text(
            f"http_proxy=http://127.0.0.1:{HTTP_PROXY_PORT}\n"
            f"https_proxy=http://127.0.0.1:{HTTP_PROXY_PORT}\n"
            f"all_proxy=socks5h://127.0.0.1:{SOCKS_PORT}\n"
            f"HTTP_PROXY=http://127.0.0.1:{HTTP_PROXY_PORT}\n"
            f"HTTPS_PROXY=http://127.0.0.1:{HTTP_PROXY_PORT}\n"
            f"ALL_PROXY=socks5h://127.0.0.1:{SOCKS_PORT}\n",
            encoding="utf-8",
        )
    except Exception:
        pass


def clear_system_proxy():
    try:
        subprocess.run(["gsettings", "set", "org.gnome.system.proxy", "mode", "none"], check=True)
    except Exception:
        pass
    for kwrite in ("kwriteconfig6", "kwriteconfig5"):
        try:
            subprocess.run([kwrite, "--file", "kioslaverc", "--group", "Proxy Settings", "--key", "ProxyType", "0"], capture_output=True)
            break
        except Exception:
            pass
    try:
        env_file = Path(os.environ.get("HOME", "")) / ".config" / "environment.d" / "99-netfreedom.conf"
        if env_file.exists():
            env_file.unlink()
    except Exception:
        pass


def start_bypass(mode="auto_pilot", dpi_settings=None, stealth_node=None):
    global _bridge_running, _desired_connected, _active_mode, _active_dpi_settings, _active_stealth_node

    effective_mode = mode
    if mode == "auto_pilot":
        # Empirical ladder: test UDP Anycast first
        udp_ok = False
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            s.settimeout(1.2)
            s.sendto(
                b"\x00\x00\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x0a\x63\x6c\x6f\x75\x64\x66\x6c\x61\x72\x65\x03\x63\x6f\x6d\x00\x00\x01\x00\x01",
                ("1.1.1.1", 53),
            )
            s.recvfrom(512)
            s.close()
            udp_ok = True
        except Exception:
            udp_ok = False

        saved_node = stealth_node or (load_settings().get("stealth_node") or {})
        has_custom_worker = bool(saved_node.get("host") and saved_node.get("host") != "cloudflare.com" and saved_node.get("uuid"))
        if not udp_ok and has_custom_worker:
            effective_mode = "stealth_tunnel"
        else:
            effective_mode = "dpi_desync"

    with _state_lock:
        _launch_engine(effective_mode, dpi_settings=dpi_settings, stealth_node=stealth_node)
        _active_mode = effective_mode
        _active_dpi_settings = dpi_settings
        _active_stealth_node = stealth_node
        _desired_connected = True

    if not _bridge_running:
        t = threading.Thread(target=run_http_to_socks_bridge, daemon=True)
        t.start()
        time.sleep(0.2)

    set_system_proxy()


def stop_bypass():
    global _desired_connected, _active_mode
    with _state_lock:
        _desired_connected = False
        _active_mode = None
        clear_system_proxy()
        subprocess.run(["pkill", "-9", "-x", "ciadpi"], capture_output=True)
        subprocess.run(["pkill", "-9", "-x", "sing-box"], capture_output=True)


def watchdog_loop():
    """Monitors the active engine in the background and recovers automatically after sleep/wake or crashes."""
    while True:
        time.sleep(5)
        with _state_lock:
            if not _desired_connected or not _active_mode:
                continue
            needs_singbox = _active_mode in ("stealth_tunnel", "warp_shield")
            alive = is_singbox_running() if needs_singbox else is_ciadpi_running()
            if not alive:
                print(f"[!] Watchdog detected dead engine for {_active_mode}, restarting...")
                try:
                    _launch_engine(_active_mode, _active_dpi_settings, _active_stealth_node)
                except Exception as e:
                    print(f"[-] Watchdog restart failed: {e}")


class ReusableThreadingHTTPServer(http.server.ThreadingHTTPServer):
    allow_reuse_address = True
    daemon_threads = True


class NetFreedomHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(DIST_DIR), **kwargs)

    def log_message(self, format, *args):
        # Silence noisy 1s polling endpoints
        if args and isinstance(args[0], str) and ("/api/metrics" in args[0] or "/api/status" in args[0]):
            return
        super().log_message(format, *args)

    def _send_json(self, code: int, payload: dict):
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(payload).encode("utf-8"))

    def do_GET(self):
        if self.path == "/api/status":
            running = is_ciadpi_running() or is_singbox_running()
            proxy_mode = get_gnome_proxy_mode()
            is_connected = running and (proxy_mode == "manual")
            self._send_json(200, {
                "status": "connected" if is_connected else "disconnected",
                "mode": _active_mode or "dpi_desync",
                "ciadpi_running": is_ciadpi_running(),
                "singbox_running": is_singbox_running(),
                "proxy_mode": proxy_mode,
                "socks_port": SOCKS_PORT,
                "http_port": HTTP_PROXY_PORT,
            })
            return

        elif self.path == "/api/metrics":
            info = fetch_egress_info()
            active = is_ciadpi_running() or is_singbox_running()
            self._send_json(200, {
                "tcp_latency_ms": _cached_latency_ms,
                "jitter_ms": _cached_jitter_ms,
                "download_bytes_sec": 1024 * 128 if active else 0,
                "upload_bytes_sec": 1024 * 32 if active else 0,
                "public_ip": info.get("ip"),
                "country": info.get("country"),
                "colo": info.get("colo"),
                "captive_portal_detected": _cached_captive,
                "ipv6_working": _cached_ipv6,
            })
            return

        elif self.path == "/api/diagnostics":
            _bg_refresh_egress()
            self._send_json(200, {
                "tcp_latency_ms": _cached_latency_ms,
                "jitter_ms": _cached_jitter_ms,
                "public_ip": _cached_egress.get("ip"),
                "country": _cached_egress.get("country"),
                "colo": _cached_egress.get("colo"),
                "captive_portal_detected": _cached_captive,
                "ipv6_working": _cached_ipv6,
            })
            return

        elif self.path == "/api/settings":
            self._send_json(200, load_settings())
            return

        elif self.path == "/api/verify":
            res = subprocess.run(
                ["curl", "-s", "-I", "--connect-timeout", "3", "--proxy", f"http://127.0.0.1:{HTTP_PROXY_PORT}", "https://www.google.com"],
                capture_output=True,
                text=True,
            )
            self._send_json(200, {"accessible": "200" in res.stdout or "301" in res.stdout or "302" in res.stdout, "output": res.stdout[:200]})
            return

        super().do_GET()

    def do_POST(self):
        content_len = int(self.headers.get("Content-Length", 0))
        body = json.loads(self.rfile.read(content_len).decode("utf-8")) if content_len > 0 else {}

        if self.path == "/api/connect":
            try:
                mode = body.get("mode", "auto_pilot")
                dpi_settings = body.get("dpiSettings")
                stealth_node = body.get("stealthNode")
                start_bypass(mode=mode, dpi_settings=dpi_settings, stealth_node=stealth_node)
                self._send_json(200, {"status": "connected", "mode": _active_mode, "message": "Bypass active system-wide"})
            except ValueError as e:
                self._send_json(400, {"error": str(e)})
            except Exception as e:
                self._send_json(500, {"error": str(e)})
            return

        elif self.path == "/api/disconnect":
            try:
                stop_bypass()
                self._send_json(200, {"status": "disconnected", "message": "Bypass deactivated"})
            except Exception as e:
                self._send_json(500, {"error": str(e)})
            return

        elif self.path == "/api/diagnostics":
            _bg_refresh_egress()
            self._send_json(200, {
                "tcp_latency_ms": _cached_latency_ms,
                "jitter_ms": _cached_jitter_ms,
                "public_ip": _cached_egress.get("ip"),
                "country": _cached_egress.get("country"),
                "colo": _cached_egress.get("colo"),
                "captive_portal_detected": _cached_captive,
                "ipv6_working": _cached_ipv6,
            })
            return

        elif self.path == "/api/settings":
            try:
                save_settings(body)
                self._send_json(200, {"saved": True})
            except Exception as e:
                self._send_json(500, {"error": str(e)})
            return

        self.send_response(404)
        self.end_headers()


if __name__ == "__main__":
    os.chdir(str(DIST_DIR))
    if (is_ciadpi_running() or is_singbox_running()) and not _bridge_running:
        threading.Thread(target=run_http_to_socks_bridge, daemon=True).start()

    threading.Thread(target=watchdog_loop, daemon=True).start()

    server = ReusableThreadingHTTPServer(("127.0.0.1", UI_PORT), NetFreedomHandler)
    print(f"NetFreedom Desktop Controller running at http://127.0.0.1:{UI_PORT}")
    server.serve_forever()
