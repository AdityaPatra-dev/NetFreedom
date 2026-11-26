# NetFreedom for Apple (iOS & macOS)

Unified, native Swift & SwiftUI application for Apple platforms with system-level VPN packet routing powered by `NetworkExtension` and `HevSocks5Tunnel.xcframework`.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│              NetFreedom SwiftUI App (iOS / macOS)           │
│  - Dashboard & Power Button                                 │
│  - Profiles (Auto-Pilot, Stealth, ByeDPI, WARP, Hybrid)     │
│  - Empirical Diagnostics (TCP Handshake RTT, Captive Check) │
│  - Settings (Kill Switch, MTU, Cloudflare Worker Node)       │
└──────────────────────────────┬──────────────────────────────┘
                               │ NETunnelProviderManager API
┌──────────────────────────────▼──────────────────────────────┐
│              PacketTunnel (NEPacketTunnelProvider)          │
│  - utun virtual TUN interface                               │
│  - System DNS hijack to 1.1.1.1                             │
│  - HevSocks5Tunnel.xcframework FFI Bridge                   │
└──────────────────────────────┬──────────────────────────────┘
                               │ SOCKS5 TCP 1080
┌──────────────────────────────▼──────────────────────────────┐
│        ByeDPI (ciadpi) / Sing-Box Anti-Censorship Engine     │
│  - TCP/TLS SNI packet fragmentation                         │
│  - VLESS over WebSocket (TCP 443)                           │
│  - Cloudflare Anycast egress                                │
└─────────────────────────────────────────────────────────────┘
```

---

## Building & Deployment

### Prerequisites
- macOS 14+ with Xcode 15+
- Apple Developer Account (required for NetworkExtension entitlements)

### Steps
1. **Compile Native Frameworks:**
   ```bash
   ./scripts/build-apple-frameworks.sh
   ```
2. **Open in Xcode:**
   Open `apps/apple/` in Xcode, or link `PacketTunnel` target with the `NetworkExtension` framework and embed `HevSocks5Tunnel.xcframework`.
3. **Capabilities & Entitlements:**
   - App: Enable **App Sandbox**, **Network Extensions** (`Packet Tunnel`).
   - PacketTunnel: Enable **Network Extensions** (`Packet Tunnel Provider`).
4. **Build & Run:**
   Target physical iPhone, iPad, or Mac.
