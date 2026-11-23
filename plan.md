# NetFreedom: Universal Multi-Mode Connectivity & Privacy Platform
**Platform:** Linux (Fedora/Ubuntu/Debian), Windows (10/11), Android  
**Version:** 3.0.0-ProductionArchitecture  
**Status:** Comprehensive Technical Blueprint & Robust Implementation Specification  
**Author:** Architecture & Systems Engineering Team  
**Date:** November 2026  

---

## 1. Executive Summary & Core Engineering Philosophy

**NetFreedom** is a production-grade, cross-platform privacy, security, and network connectivity platform designed to run on Linux, Windows, and Android. It functions as an **all-in-one connectivity powerhouse ("Swiss Army Knife")** that can be used by anyone—from non-technical everyday users wanting a single-click "Connect" button, to advanced network power users requiring granular protocol control.

### Core Architectural Pillars
1. **"Upstream-First, Zero-Reinvention"**:
   - Rather than writing custom userspace TCP/IP stacks or cryptographic handshakes from scratch, NetFreedom directly embeds and orchestrates battle-tested, prebuilt open-source engines (`boringtun`, `byedpi`, `hev-socks5-tunnel`, `wintun.dll`, `hickory-dns`).
2. **True Multi-Mode Modularity**:
   - Provides 5 distinct, first-class operating modes: Cloudflare WARP Shield, ByeDPI Desync, Custom WireGuard/OpenVPN, Modular SOCKS5 Proxy, and Hybrid Super-Stealth Chaining.
3. **Universal Usability ("Smart Auto-Pilot")**:
   - A single 1-click "Connect" button automatically probes local network constraints (<200ms) and launches the fastest, most secure working engine with zero user configuration.
4. **Empirical Optimization for Institutional / Campus Networks**:
   - Tailored specifically to the active Cisco enterprise network detected during profiling (`10.6.80.0/20`, Cisco HSRP MAC `00:00:0c:07:ac:3f`): defeats ICMP blackholing, broken IPv6 dual-stack routing, and middlebox SNI filtering.
5. **Zero-Destruction Routing & Privilege Separation**:
   - Uses the safe `/1` prefix routing technique (never clobbering the default gateway route `0.0.0.0/0`).
   - Uses a secure Daemon + Client architecture: GUI runs completely unprivileged; a lightweight daemon handles kernel interfaces with `CAP_NET_ADMIN`.

---

## 2. Empirical Network Telemetry & Cisco Campus Optimization

Live probes executed against the host environment provided precise network constraints:

```
+---------------------------------------------------------------------------------------------------+
|                                  ACTIVE HOST NETWORK TELEMETRY                                    |
+---------------------------------------------------------------------------------------------------+
|  * Host System: Fedora Linux 44 (Workstation Edition), Kernel 7.2.13, x86_64                      |
|  * Subnet & IP: 10.6.83.67 on 10.6.80.0/20 (Gateway: 10.6.80.1)                                   |
|  * Gateway Hardware: MAC 00:00:0c:07:ac:3f (Cisco HSRP Virtual Gateway, Group 63)                 |
|  * ICMP Ping to 1.1.1.1: 100% Dropped by Cisco firewall (ICMP Blackhole)                          |
|  * IPv6 Connectivity: Broken dual-stack (AAAA resolved, but IPv6 route is dead upstream)          |
|  * TCP Port 443 Handshake: 40.7ms RTT direct to Cloudflare Edge (Madras / Chennai MAA colo)       |
|  * Cloudflare WARP API: https://api.cloudflareclient.com returns HTTP/2 200                       |
|  * WireGuard UDP: Port 2408 to 162.159.192.1 connects cleanly with zero drops                    |
|  * System DNS: systemd-resolved 259 stub resolver on 127.0.0.53                                   |
+---------------------------------------------------------------------------------------------------+
```

### Dedicated Campus Network Profile Built into NetFreedom:
1. **Non-ICMP TCP Handshake Canary**:
   - Because raw ICMP ping is 100% blocked, standard ping tools fail. NetFreedom measures latency and health using **TCP SYN/ACK round-trip timing to port 443** (yielding exact ~40ms real-world measurements) and DoH query timing.
2. **Smart Dual-Stack IPv6 Sanitizer**:
   - Prevents the 3–5 second browser freeze caused by the campus router resolving AAAA DNS records without working IPv6 routes. NetFreedom automatically blackholes local IPv6 egress or suppresses AAAA queries in IPv4 tunnel mode, forcing instantaneous IPv4 loading.
3. **Cisco HSRP-Aware Route Pinning**:
   - Preserves explicit `/32` host route to `10.6.80.1` on `wlp195s0`, allowing redundant Cisco enterprise switches to failover without breaking tunnel state.
4. **Port 2408 Prioritization**:
   - Prioritizes Cloudflare Anycast port `2408` over the commonly blocked standard WireGuard port `51820`.

---

## 3. High-Level System Architecture & Privilege Model

To adhere to modern Linux/Windows/Android security standards, **the GUI never runs as root**.

```
+---------------------------------------------------------------------------------------------------+
|                                      USER SPACE (UNPRIVILEGED)                                    |
|                                                                                                   |
|   +-------------------------------------------------------------------------------------------+   |
|   |                        Tauri v2 Desktop GUI / Android Compose UI                          |   |
|   |  - Dashboard (Smart 1-Click Connect & Mode Carousel)                                      |   |
|   |  - Profiles Manager & DPI Lab (interactive sliders for split-pos, TTL, disorder)          |   |
|   |  - Diagnostics Suite (non-ICMP latency sparkline, DNS/IPv6 leak tester, speed meter)      |   |
|   |  - Settings (Kill switch toggle, split tunneling rules, custom DoH DNS selector)          |   |
|   +-------------------------------------------------------------------------------------------+   |
|                                                |                                                  |
|                   Linux: UNIX Domain Socket (/run/netfreedom.sock)                                |
|                   Windows: Named Pipe (\\.\pipe\NetFreedomIPC)                                    |
|                   Android: JNI Binder Interface                                                   |
|                                                v                                                  |
+---------------------------------------------------------------------------------------------------+
|                                   PRIVILEGED BACKGROUND DAEMON                                    |
|                               (CAP_NET_ADMIN on Linux | LocalSystem on Windows)                   |
|                                                                                                   |
|   +-------------------------------------------------------------------------------------------+   |
|   |                             netfreedom-daemon (Rust Orchestrator)                         |   |
|   |  - Profile State Machine & Supervisor                                                     |   |
|   |  - Non-ICMP Health Monitor & Auto-Failover Watchdog                                       |   |
|   |  - Routing Table Engine (Safe /1 Prefix Injector)                                         |   |
|   |  - DNS Interceptor (systemd-resolved D-Bus on Linux / NRPT on Windows)                    |   |
|   |  - Kill Switch Controller (nftables on Linux / WFP on Windows)                            |   |
|   +-------------------------------------------------------------------------------------------+   |
|          |                                     |                                     |            |
|          v                                     v                                     v            |
|   +-------------------+                 +-------------------+                 +---------------+   |
|   |  boringtun Crate  |                 | hev-socks5-tunnel |                 |    byedpi     |   |
|   | Cloudflare Userspace|               | C Coroutine TUN-  |                 | C99 Low-Level |   |
|   | WireGuard Engine  |                 | to-SOCKS5 Router  |                 | DPI Desync    |   |
|   | (WARP & Custom)   |                 | (<5MB RAM)        |                 | (Split, TTL)  |   |
|   +-------------------+                 +-------------------+                 +---------------+   |
+---------------------------------------------------------------------------------------------------+
                                                |
                                                v
+---------------------------------------------------------------------------------------------------+
|                                      OPERATING SYSTEM KERNEL                                      |
|   Linux: /dev/net/tun + nftables | Windows: wintun.dll + WFP | Android: android.net.VpnService    |
+---------------------------------------------------------------------------------------------------+
```

---

## 4. Deep Dive into the 5 Operating Modes

### Mode 1: Cloudflare WARP Shield (1-Click Universal Privacy)
- **Engine**: Cloudflare `boringtun` (Rust WireGuard) + native Cloudflare Client API provisioner.
- **Workflow**:
  1. NetFreedom generates an ephemeral Curve25519 keypair and registers an anonymous identity against `https://api.cloudflareclient.com/v0a2158/reg`.
  2. Receives client internal IPv4 (`172.16.0.2/32`), IPv6, and server public key.
  3. Launches `boringtun` userspace tunnel connecting to `162.159.192.1:2408`.
  4. Injects safe `/1` routes into `netfreedom0`.
- **Privacy Guarantee**: All traffic encrypted with ChaCha20-Poly1305. The campus Wi-Fi router sees only opaque UDP packets to Cloudflare. Egress IP is Cloudflare Anycast, unblocking all websites.

### Mode 2: DPI Desynchronization (ByeDPI / Zapret Serverless Mode)
- **Engine**: `byedpi` (`ciadpi` C99 core) + `hev-socks5-tunnel` router.
- **Why It's Special**: Bypasses censorship and middlebox blocking **without any external VPN server**—preserving 100% of raw ISP bandwidth with **0ms added latency**.
- **Underlying Mechanisms**:
  - `ciadpi -i 127.0.0.1 -p 1080 -s 1+s -f -1 -t 3 -d 1+s -F`
  - **SNI Splitting (`-s 1+s`)**: Slices the TLS `ClientHello` at offset 1 of the SNI domain name.
  - **Fake Low-TTL Packet (`-f -1 -t 3`)**: Emits a fake TLS handshake with `TTL=3`. The local campus Cisco firewall inspects and accepts the fake packet, but the packet expires before reaching the internet. The destination server receives only the valid payload.
  - **TCP Disordering (`-d 1+s`)**: Sends segment 2 before segment 1. RFC-compliant servers reassemble it; campus DPI firewalls fail to inspect out-of-order packets.
- **DPI Lab Interface**: Interactive UI with visual sliders for split offset, fake TTL, and disorder toggles.

### Mode 3: Custom WireGuard & OpenVPN Client
- **Engine**: `boringtun` with `.conf` parser and QR code scanner.
- **Features**: Allows power users to connect to personal VPS servers (Algo, Outline, Streisand) or commercial VPNs (Mullvad, ProtonVPN, IVPN).

### Mode 4: Modular Proxy Client & Inbound Proxy Manager
- **Engine**: Tokio async proxy core + `hev-socks5-tunnel`.
- **Features**: SOCKS5 (with auth), Shadowsocks (AEAD), and HTTP CONNECT.
- **Local Inbound Proxy**: Exposes `127.0.0.1:10808` (SOCKS5) and `127.0.0.1:10809` (HTTP) for selective app routing (e.g. Firefox or Telegram only) without full system TUN redirection.

### Mode 5: Hybrid Super-Stealth Chaining
- **Engine**: Chains `hev-socks5-tunnel` $\rightarrow$ `byedpi` $\rightarrow$ `boringtun` / Shadowsocks.
- **Workflow**: WireGuard UDP/TCP packets are first desynchronized by ByeDPI before leaving the physical interface, evading deep protocol fingerprinting.

---

## 5. Robust Networking & Anti-Leak Mechanisms

### 5.1. Safe Routing Engine (The Double `/1` Prefix Trick)
Replacing the system default route `0.0.0.0/0 via 10.6.80.1` is dangerous: if the application crashes or the system sleeps, the original gateway is lost, completely breaking connectivity. NetFreedom implements the battle-tested **Double `/1` Prefix Trick**:
```bash
# 1. Pin explicit route to VPN endpoint via physical gateway
ip route add 162.159.192.1/32 via 10.6.80.1 dev wlp195s0

# 2. Add two /1 routes pointing to NetFreedom TUN (more specific than /0)
ip route add 0.0.0.0/1 dev netfreedom0
ip route add 128.0.0.0/1 dev netfreedom0
```
- **Why it's bulletproof**: `0.0.0.0/1` covers `0.0.0.0`–`127.255.255.255`; `128.0.0.0/1` covers `128.0.0.0`–`255.255.255.255`. Because `/1` is more specific than `/0`, the Linux kernel routes 100% of internet traffic through `netfreedom0`.
- **Zero-Disruption Rollback**: The original `0.0.0.0/0 via 10.6.80.1` is never deleted. When disconnecting or on crash recovery, NetFreedom simply removes the two `/1` routes, immediately restoring normal network state.

### 5.2. Non-Destructive DNS Interception (`systemd-resolved` D-Bus)
Instead of overwriting `/etc/resolv.conf`:
- Calls `systemd-resolved` D-Bus API:
  - `SetLinkDNS(netfreedom0, [1.1.1.1])`
  - `SetLinkDomains(netfreedom0, ["~."])` (forces all system DNS through `netfreedom0`).
- When `netfreedom0` is closed, `systemd-resolved` automatically unbinds the route and reverts to Wi-Fi DNS.

### 5.3. Fail-Safe Hardware Kill Switch (`nftables`)
```nftables
table inet netfreedom_killswitch {
    chain output {
        type filter hook output priority -10; policy drop;
        oifname "lo" accept
        oifname "netfreedom0" accept
        ip daddr 10.6.80.0/20 accept # Allow local LAN if enabled
        ip daddr 162.159.192.1 udp dport 2408 accept # Allow VPN endpoint
    }
}
```
If the tunnel unexpectedly drops, all unencrypted outgoing traffic on `wlp195s0` is instantly dropped by the kernel until the tunnel recovers.

---

## 6. Prebuilt Upstream Codebase Integration

All upstream components are mapped to well-defined C/Rust APIs:

| Component | Upstream Project | Concrete API / Entry Point Used | Memory / Footprint |
|---|---|---|---|
| **TUN-to-SOCKS5 Bridge** | [hev-socks5-tunnel](https://github.com/heiher/hev-socks5-tunnel) | `hev_socks5_tunnel_main_from_str(yaml_cfg, len, tun_fd)`<br>`hev_socks5_tunnel_stats(&tx_p, &tx_b, &rx_p, &rx_b)`<br>`hev_socks5_tunnel_quit()` | C Coroutines, `<4.2MB` RAM, 0% idle CPU. |
| **DPI Circumvention** | [byedpi](https://github.com/hufrea/byedpi) | C99 `ciadpi` CLI & parameters (`-s 1+s -f -1 -t 3 -d 1+s -F`) via embedded static binary or companion process | C99, `<3.5MB` RAM. |
| **WireGuard Core** | [cloudflare/boringtun](https://github.com/cloudflare/boringtun) | `boringtun::noise::Tunn` & userspace async socket loop | Pure Rust, memory-safe, wire speed. |
| **Windows TUN Driver** | [wintun](https://www.wintun.net/) | `WintunCreateAdapter`, `WintunStartSession`, packet ring buffer via `wintun` crate | Official signed Windows kernel driver. |
| **Encrypted DNS** | [hickory-dns](https://github.com/hickory-dns/hickory-dns) | Async `DohClientStream` over HTTP/2 | Pure Rust, native TLS. |
| **Desktop Shell** | [tauri](https://github.com/tauri-apps/tauri) v2 | Tauri v2 IPC, WebKitGTK (Linux) & WebView2 (Windows) | `<15MB` binary, `<40MB` RAM. |
| **Android Shell** | [ByeDPIAndroid](https://github.com/dovecoteescapee/ByeDPIAndroid) | `android.net.VpnService` + JNI bindings to `libhev-socks5-tunnel.so` | Kotlin + Jetpack Compose. |

---

## 7. Concrete Project Structure

```
NetFreedom/
├── Cargo.toml                    # Workspace definition
├── plan.md                       # Comprehensive master architecture specification
├── chatgpt-export-2026-11-23.md  # Original vision & requirements
├── third_party/                  # Upstream prebuilt submodules
│   ├── byedpi/                   # git submodule: github.com/hufrea/byedpi
│   ├── hev-socks5-tunnel/        # git submodule: github.com/heiher/hev-socks5-tunnel
│   └── wintun/                   # wintun.dll headers & binaries for Windows
├── crates/
│   ├── netfreedom-common/        # Shared data structures, profiles, IPC messages
│   ├── netfreedom-daemon/        # Privileged background service & route manager
│   ├── netfreedom-vpn/           # boringtun WireGuard & Cloudflare WARP client
│   ├── netfreedom-dpi/           # Controller & process manager for byedpi
│   ├── netfreedom-tun/           # FFI wrapper for hev-socks5-tunnel & wintun
│   ├── netfreedom-dns/           # hickory-dns DoH/DoT resolver & leak blocker
│   ├── netfreedom-health/        # Non-ICMP TCP RTT probe & IPv6 sanitizer
│   ├── netfreedom-security/      # nftables, WFP, and kill-switch manager
│   └── netfreedom-cli/           # Unified developer CLI for testing on Fedora
├── apps/
│   ├── desktop/                  # Tauri v2 Desktop App (Linux & Windows)
│   │   ├── src-tauri/            # Tauri backend communicating with netfreedom-daemon
│   │   ├── src/                  # React 19 + TypeScript + Tailwind UI
│   │   └── package.json
│   └── android/                  # Android APK Project
│       ├── app/
│       │   ├── src/main/java/    # Kotlin + Jetpack Compose UI
│       │   ├── src/main/cpp/     # JNI bindings to libhev-socks5-tunnel & libbyedpi
│       │   └── build.gradle.kts
│       └── build.gradle.kts
└── scripts/                      # Setup, polkit policies, and build scripts
```

---

## 8. Phased Implementation Roadmap

### Phase 1: Core Scaffolding & Upstream Vendor Submodules
- Initialize Cargo workspace (`Cargo.toml`).
- Add `third_party/byedpi` and `third_party/hev-socks5-tunnel` submodules.
- Implement `netfreedom-vpn`: integrate `boringtun` + Cloudflare WARP client API provisioner.
- Implement `netfreedom-dpi`: integrate `byedpi` with configurable desync parameters.
- Implement `netfreedom-tun`: FFI bindings to `hev_socks5_tunnel_main_from_str`.
- Implement `netfreedom-health`: non-ICMP TCP handshake RTT probe + IPv6 dual-stack sanitizer.

### Phase 2: Linux CLI & Multi-Mode Verification (`netfreedom-cli`)
- Build unified CLI on Fedora:
  - `netfreedom auto` (Smart Auto-Pilot: probes network, picks WARP or ByeDPI).
  - `netfreedom warp connect` (launches WARP WireGuard tunnel).
  - `netfreedom dpi start --split-pos 1+s --fake -1 --ttl 3` (launches serverless ByeDPI).
  - `netfreedom diag` (runs DNS leak, IPv6 check, and TCP latency tests).
- Validate all modes live against the Cisco campus network.

### Phase 3: Desktop UI & Tauri v2 Shell
- Initialize `apps/desktop` with Tauri v2, React 19, TypeScript, and Tailwind CSS.
- Build Dashboard (Smart Connect button + Mode Carousel), Profiles, DPI Lab, Diagnostics, and Settings.
- Connect Tauri IPC to `netfreedom-daemon` via UNIX domain socket.
- Add system tray integration with mode switching and status badge.

### Phase 4: Windows Platform Adapter
- Integrate `wintun.dll` on Windows 10/11 for kernel TUN networking.
- Implement Windows NRPT DNS enforcement and WFP kill switch.
- Bundle WiX / NSIS installer with Windows Service.

### Phase 5: Android Platform Adapter
- Scaffold Android Studio Gradle project in `apps/android` with Jetpack Compose.
- Compile native C libraries (`libhev-socks5-tunnel.so`, `libbyedpi.so`) via NDK for `arm64-v8a` and `x86_64`.
- Implement `VpnService` bridging Android's TUN file descriptor to `hev-socks5-tunnel`.
- Build Material 3 UI with Quick Settings Tile.

### Phase 6: Polish, Hardening & Automated Testing
- Stress test rapid switching between modes and auto-recovery from sleep/wake and Wi-Fi disconnects.
- Automated leak test suite verifying zero unencrypted packets escape during unexpected termination.
- Set up GitHub Actions CI/CD to build Linux (`.rpm`, `.deb`, AppImage), Windows (`.msi`), and Android (`.apk`).
