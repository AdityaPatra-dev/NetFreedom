#!/usr/bin/env python3
import http.server
import json
import os
import select
import socket
import subprocess
import threading
import time
from pathlib import Path

UI_PORT = 1420
HTTP_PROXY_PORT = 1081
SOCKS_PORT = 1080

PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
DIST_DIR = Path(__file__).resolve().parent / "dist"
CIADPI_BIN = PROJECT_ROOT / "third_party" / "byedpi" / "ciadpi"

_bridge_running = False
_bridge_server = None


def run_http_to_socks_bridge():
    global _bridge_running, _bridge_server
    try:
        _bridge_server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        _bridge_server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        _bridge_server.bind(("127.0.0.1", HTTP_PROXY_PORT))
        _bridge_server.listen(128)
        _bridge_running = True
    except Exception as e:
        print(f"[-] Bridge bind error on {HTTP_PROXY_PORT}: {e}")
        return

    def client_worker(client_sock):
        try:
            req = client_sock.recv(4096)
            if not req.startswith(b"CONNECT "):
                client_sock.close()
                return
            first_line = req.split(b"\r\n")[0].decode()
            target = first_line.split()[1]
            host, port = target.split(":")
            port = int(port)

            # SOCKS5 Handshake with ByeDPI (127.0.0.1:1080)
            s5 = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s5.connect(("127.0.0.1", SOCKS_PORT))
            s5.sendall(b"\x05\x01\x00")
            resp = s5.recv(2)
            if resp != b"\x05\x00":
                client_sock.close()
                s5.close()
                return

            # SOCKS5 Connect (domain name type: 0x03)
            host_bytes = host.encode()
            req_pkt = (
                b"\x05\x01\x00\x03"
                + bytes([len(host_bytes)])
                + host_bytes
                + port.to_bytes(2, "big")
            )
            s5.sendall(req_pkt)
            resp = s5.recv(10)
            if len(resp) < 2 or resp[1] != 0x00:
                client_sock.close()
                s5.close()
                return

            client_sock.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")

            # Bidirectional streaming
            sockets = [client_sock, s5]
            while True:
                r, _, _ = select.select(sockets, [], sockets, 30)
                if not r:
                    break
                for s in r:
                    other = s5 if s is client_sock else client_sock
                    data = s.recv(32768)
                    if not data:
                        return
                    other.sendall(data)
        except Exception:
            pass
        finally:
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
    res = subprocess.run(["pgrep", "-f", "ciadpi"], capture_output=True)
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


def start_bypass():
    global _bridge_running
    # 1. Kill any existing instance
    subprocess.run(["pkill", "-9", "-f", "ciadpi"], capture_output=True)

    # 2. Start ciadpi with tested evasion parameters
    cmd = [
        str(CIADPI_BIN),
        "-i",
        "127.0.0.1",
        "-p",
        str(SOCKS_PORT),
        "-I",
        "0.0.0.0",
        "-s",
        "2+s",
        "-f",
        "-1",
        "-t",
        "3",
        "-d",
        "1+s",
        "-D",
    ]
    subprocess.run(cmd, check=True)

    # 3. Start HTTP-to-SOCKS bridge if not running
    if not _bridge_running:
        t = threading.Thread(target=run_http_to_socks_bridge, daemon=True)
        t.start()
        time.sleep(0.2)

    # 4. Set GNOME System Proxy for both HTTP/HTTPS and SOCKS
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy.http", "host", "127.0.0.1"],
        check=True,
    )
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy.http", "port", str(HTTP_PROXY_PORT)],
        check=True,
    )
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy.http", "enabled", "true"],
        check=True,
    )
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy.https", "host", "127.0.0.1"],
        check=True,
    )
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy.https", "port", str(HTTP_PROXY_PORT)],
        check=True,
    )
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"],
        check=True,
    )
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy.socks", "port", str(SOCKS_PORT)],
        check=True,
    )
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy", "mode", "manual"],
        check=True,
    )


def stop_bypass():
    global _bridge_running, _bridge_server
    # 1. Revert GNOME proxy
    subprocess.run(
        ["gsettings", "set", "org.gnome.system.proxy", "mode", "none"],
        check=True,
    )
    # 2. Stop ciadpi
    subprocess.run(["pkill", "-9", "-f", "ciadpi"], capture_output=True)


class NetFreedomHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(DIST_DIR), **kwargs)

    def do_GET(self):
        if self.path == "/api/status":
            running = is_ciadpi_running()
            proxy_mode = get_gnome_proxy_mode()
            is_connected = running and (proxy_mode == "manual")
            data = {
                "status": "connected" if is_connected else "disconnected",
                "ciadpi_running": running,
                "proxy_mode": proxy_mode,
                "socks_port": SOCKS_PORT,
                "http_port": HTTP_PROXY_PORT,
                "sni_mode": "split_2+s_ttl3_disorder",
            }
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(json.dumps(data).encode("utf-8"))
            return
        elif self.path == "/api/verify":
            res = subprocess.run(
                [
                    "curl",
                    "-s",
                    "-I",
                    "--connect-timeout",
                    "3",
                    "--proxy",
                    f"http://127.0.0.1:{HTTP_PROXY_PORT}",
                    "https://en-thunderscans.com",
                ],
                capture_output=True,
                text=True,
            )
            data = {
                "accessible": "200 OK" in res.stdout,
                "output": res.stdout[:200],
            }
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(json.dumps(data).encode("utf-8"))
            return

        super().do_GET()

    def do_POST(self):
        if self.path == "/api/connect":
            try:
                start_bypass()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(
                    json.dumps(
                        {"status": "connected", "message": "Bypass active system-wide"}
                    ).encode("utf-8")
                )
            except Exception as e:
                self.send_response(500)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode("utf-8"))
            return

        elif self.path == "/api/disconnect":
            try:
                stop_bypass()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(
                    json.dumps(
                        {
                            "status": "disconnected",
                            "message": "Bypass deactivated",
                        }
                    ).encode("utf-8")
                )
            except Exception as e:
                self.send_response(500)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode("utf-8"))
            return

        self.send_response(404)
        self.end_headers()


if __name__ == "__main__":
    os.chdir(str(DIST_DIR))
    # Start bridge automatically if ciadpi is running
    if is_ciadpi_running() and not _bridge_running:
        threading.Thread(target=run_http_to_socks_bridge, daemon=True).start()

    server = http.server.ThreadingHTTPServer(
        ("127.0.0.1", UI_PORT), NetFreedomHandler
    )
    print(f"NetFreedom Desktop Controller running at http://127.0.0.1:{UI_PORT}")
    server.serve_forever()
