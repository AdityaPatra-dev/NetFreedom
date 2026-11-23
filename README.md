# NetFreedom (网络自由)

<div align="center">

**Next-Generation Multi-Mode Privacy, Security & Anti-Censorship Network Platform**

[![CI](https://github.com/AdityaPatra-dev/NetFreedom/actions/workflows/ci.yml/badge.svg)](https://github.com/AdityaPatra-dev/NetFreedom/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB.svg)](https://v2.tauri.app)
[![Android](https://img.shields.io/badge/Android-SDK%2026--35-3DDC84.svg)](https://developer.android.com)

*Engineered for hostile networks: Cisco Enterprise/Campus Wi-Fi, middlebox Deep Packet Inspection (DPI), dead dual-stack IPv6 routes, and ICMP blackholes.*

</div>

---

## 1. Overview & Core Philosophy

**NetFreedom** is not a single-task utility or a simple VPN wrapper. It is a unified, cross-platform network resilience engine that combines modern userspace WireGuard encryption, serverless TCP/TLS desynchronization, kernel-level TUN virtual routing, and strict hardware killswitches across **Linux**, **Windows**, and **Android**.

### "Upstream-First, Zero-Reinvention" Architecture
Rather than reimplementing low-level networking primitives, NetFreedom leverages battle-tested, audit-grade upstream engines:
- **DPI Desynchronization Engine**: [hufrea/byedpi](https://github.com/hufrea/byedpi) (`ciadpi` C99 coroutine daemon).
- **TUN-to-SOCKS5 Bridge**: [heiher/hev-socks5-tunnel](https://github.com/heiher/hev-socks5-tunnel) (lightweight C coroutine stack with embedded lwIP).
- **WireGuard Core**: [cloudflare/boringtun](https://github.com/cloudflare/boringtun) (pure Rust, memory-safe WireGuard implementation).
- **Windows Kernel TUN**: [wintun](https://www.wintun.net/) (official signed high-performance Windows TUN driver).
- **Encrypted DNS Interceptor**: [hickory-dns](https://github.com/hickory-dns/hickory-dns) (DNS-over-HTTPS/TLS client).
- **Desktop Shell**: [tauri](https://github.com/tauri-apps/tauri) v2 + React 19 + TypeScript + Tailwind CSS.
- **Android Shell**: Android `VpnService` + JNI bindings + Jetpack Compose (Material 3) + Quick Settings Tile.

---

## 2. Multi-Mode Connectivity Matrix

| Mode | Technology Stack | Best Used For | Traffic Characteristics |
|---|---|---|---|
| **Smart Auto-Pilot** | Adaptive Telemetry Probe | Daily automatic protection | Probes network restrictions; automatically selects WARP or ByeDPI. |
| **Cloudflare WARP Shield** | WireGuard + Cloudflare Edge (`boringtun`) | General ISP throttling & surveillance | Full AES-GCM/ChaCha20 encryption to Cloudflare edge; UDP 2408. |
| **ByeDPI Desync** | Serverless TCP/TLS Desync (`ciadpi`) | Campus Wi-Fi & Middlebox DPI | 0 server cost; splits `ClientHello` SNI, spoofed TTL, fake packets. |
| **SOCKS5 Proxy** | `hev-socks5-tunnel` + Custom Proxy | Private proxies / Shadowsocks / Tor | Full OS-wide TUN routing directed to arbitrary SOCKS5 endpoint. |
| **Super-Stealth Hybrid** | ByeDPI + WARP Chaining | Maximum censorship resistance | Cleaves TCP handshake *before* tunneling through encrypted WireGuard. |

---

## 3. Hostile Network Adaptation (Cisco Enterprise / Campus Wi-Fi)

NetFreedom includes purpose-built mitigations specifically tuned for restricted enterprise/campus networks (`10.6.80.0/20` Cisco HSRP gateways):

1. **ICMP Blackhole Bypass (Non-ICMP Telemetry)**:
   - Campus firewalls block 100% of ICMP `ping` packets.
   - NetFreedom measures real-time latency via TCP SYN/ACK handshake round-trip times (RTT) directly to port 443 (~40-46ms to Cloudflare edge).
2. **Dead IPv6 Upstream Route Sanitizer**:
   - Campus DNS advertises IPv6 (`AAAA`) records, but the upstream IPv6 gateway is dead (`Immediate connect fail: Network is unreachable`), causing 3-5 second browser freezes.
   - `Ipv6Sanitizer` immediately fast-fails unreachable IPv6 routes, forcing instantaneous IPv4 connectivity.
3. **Middlebox TLS SNI Inspection Bypass**:
   - The campus firewall drops direct TLS handshakes to `api.cloudflareclient.com`.
   - NetFreedom fragments `ClientHello` packets at the SNI boundary (`-s 2+s -f -1 -t 3 -d 1+s`), successfully evading middlebox drops.

---

## 4. Security & Routing Guarantees

### 4.1. Double `/1` Prefix Trick
NetFreedom never overwrites or deletes the host's default gateway route (`0.0.0.0/0 via 10.6.80.1`):
```bash
# Covers 0.0.0.0 - 127.255.255.255
ip route add 0.0.0.0/1 dev netfreedom0
# Covers 128.0.0.0 - 255.255.255.255
ip route add 128.0.0.0/1 dev netfreedom0
```
- Because `/1` is strictly more specific than `/0`, 100% of internet traffic routes through `netfreedom0`.
- On disconnect or crash, NetFreedom simply removes the two `/1` routes, immediately restoring normal network state with zero disruption.

### 4.2. Fail-Safe Hardware Kill Switch
- **Linux (`nftables`)**: Drops all outgoing traffic unless destined for `lo`, `netfreedom0`, or the pinned VPN endpoint UDP port.
- **Windows (`WFP / netsh`)**: Blocks all outbound traffic except `Wintun` and the configured WireGuard port.

---

## 5. Repository Structure

```
NetFreedom/
├── Cargo.toml                    # Workspace manifest
├── plan.md                       # Comprehensive master architecture specification
├── crates/
│   ├── netfreedom-common/        # Shared models, profiles, telemetry structures
│   ├── netfreedom-daemon/        # Background service & route orchestrator
│   ├── netfreedom-vpn/           # boringtun WireGuard & Cloudflare WARP client
│   ├── netfreedom-dpi/           # Controller & process manager for byedpi
│   ├── netfreedom-tun/           # FFI bindings to hev-socks5-tunnel & Wintun loader
│   ├── netfreedom-dns/           # systemd-resolved D-Bus & Windows NRPT DNS manager
│   ├── netfreedom-health/        # Non-ICMP TCP RTT probe & IPv6 sanitizer
│   ├── netfreedom-security/      # nftables, Windows WFP, and fail-safe killswitch
│   └── netfreedom-cli/           # Unified CLI binary (netfreedom)
├── apps/
│   ├── desktop/                  # Tauri v2 Desktop App (React 19 + TypeScript + Tailwind)
│   └── android/                  # Android APK (Kotlin + Jetpack Compose + NDK)
├── third_party/
│   ├── byedpi/                   # git submodule: github.com/hufrea/byedpi
│   └── hev-socks5-tunnel/        # git submodule: github.com/heiher/hev-socks5-tunnel
├── scripts/                      # Setup, polkit, daemon service, and test scripts
└── .github/workflows/            # CI/CD and multi-platform release pipelines
```

---

## 6. Quick Start

### 6.1. Linux CLI

```bash
# 1. Clone repository with submodules
git clone --recursive https://github.com/AdityaPatra-dev/NetFreedom.git
cd NetFreedom

# 2. Build CLI and Daemon
cargo build --release

# 3. Run Smart Auto-Pilot
sudo ./target/release/netfreedom auto

# 4. Connect explicitly via Cloudflare WARP Shield
sudo ./target/release/netfreedom warp connect

# 5. Start Serverless ByeDPI Desync
sudo ./target/release/netfreedom dpi start --split-pos 2+s --ttl 3

# 6. Run Network Diagnostics
./target/release/netfreedom diag

# 7. Enable Hardware Killswitch
sudo ./target/release/netfreedom killswitch on
```

### 6.2. Desktop App (Tauri v2 + React)

```bash
cd apps/desktop
npm install
npm run build
# Run desktop development shell
npm run tauri dev
```

### 6.3. Android App (Universal & Architecture Splits)

```bash
# 1. Build native NDK libraries across all 4 ABIs (arm64-v8a, armeabi-v7a, x86_64, x86)
./scripts/build-android-libs.sh

# 2. Compile Universal Fat APK and standalone ABI splits
cd apps/android
./gradlew assembleDebug

# Output APKs in apps/android/app/build/outputs/apk/debug/:
# - app-universal-debug.apk   (Large universal fat APK for any device)
# - app-arm64-v8a-debug.apk   (Optimized 64-bit ARM APK)
# - app-armeabi-v7a-debug.apk (Optimized 32-bit ARM APK)
# - app-x86_64-debug.apk      (Optimized 64-bit x86 APK)
# - app-x86-debug.apk         (Optimized 32-bit x86 APK)
```

---

## 7. Testing & Verification

Run the automated leak verification suite:
```bash
./scripts/test-leak-switch.sh
```
Run all workspace Rust tests:
```bash
cargo test --workspace --all-targets
```

---

## 8. License

This project is licensed under the [MIT License](LICENSE).
Upstream submodules retain their respective open-source licenses:
- `byedpi`: MIT License
- `hev-socks5-tunnel`: MIT License
- `boringtun`: 3-Clause BSD License
