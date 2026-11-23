package org.netfreedom.app.data

enum class ConnectivityMode(val label: String, val description: String) {
    AUTO("Smart Auto-Pilot", "Auto-selects optimal evasion based on live telemetry"),
    WARP("Cloudflare WARP", "Multi-hop encrypted WireGuard shield to Cloudflare edge"),
    BYEDPI("ByeDPI Desync", "Serverless TCP/TLS fragmentation against campus DPI"),
    SOCKS5("SOCKS5 Proxy", "Custom SOCKS5 proxy routing with TUN interface"),
    HYBRID("Super-Stealth Hybrid", "ByeDPI fragmentation chained through Cloudflare WARP")
}
