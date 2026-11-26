package org.netfreedom.app.data

data class NetworkProfile(
    val id: String,
    val name: String,
    val mode: ConnectivityMode,
    val serverHost: String = "127.0.0.1",
    val serverPort: Int = 1080,
    val dnsServer: String = "1.1.1.1",
    val mtu: Int = 1420,
    val ipv6Block: Boolean = true,
    val dpiConfig: DpiConfig = DpiConfig()
) {
    companion object {
        val DEFAULT_AUTO = NetworkProfile(
            id = "auto-pilot",
            name = "Campus Auto-Pilot",
            mode = ConnectivityMode.AUTO,
            dnsServer = "1.1.1.1",
            ipv6Block = true
        )
        val DEFAULT_STEALTH_TUNNEL = NetworkProfile(
            id = "stealth-tunnel",
            name = "Campus Wi-Fi Stealth (TCP 443)",
            mode = ConnectivityMode.STEALTH_TUNNEL,
            serverHost = "104.16.132.229",
            serverPort = 443,
            dnsServer = "1.1.1.1",
            ipv6Block = true,
            dpiConfig = DpiConfig(splitPos = "2+s", fakeTtl = 3, ttlModifier = -1)
        )
        val DEFAULT_WARP = NetworkProfile(
            id = "warp-shield",
            name = "WARP Shield (Edge)",
            mode = ConnectivityMode.WARP,
            serverHost = "162.159.192.1",
            serverPort = 2408,
            dnsServer = "1.1.1.1",
            ipv6Block = true
        )
        val DEFAULT_BYEDPI = NetworkProfile(
            id = "byedpi-desync",
            name = "ByeDPI Stealth",
            mode = ConnectivityMode.BYEDPI,
            dnsServer = "1.1.1.1",
            ipv6Block = true,
            dpiConfig = DpiConfig(splitPos = "2+s", fakeTtl = 3, ttlModifier = -1)
        )
        val DEFAULT_HYBRID = NetworkProfile(
            id = "hybrid-stealth",
            name = "Super-Stealth Hybrid",
            mode = ConnectivityMode.HYBRID,
            serverHost = "162.159.192.1",
            serverPort = 2408,
            dnsServer = "1.1.1.1",
            ipv6Block = true,
            dpiConfig = DpiConfig(splitPos = "2+s", fakeTtl = 3, ttlModifier = -1)
        )
    }
}
