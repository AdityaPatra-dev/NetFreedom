package org.netfreedom.app.service

import org.netfreedom.app.data.ConnectivityMode

enum class TunnelStatus {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    DISCONNECTING,
    ERROR
}

data class VpnState(
    val status: TunnelStatus = TunnelStatus.DISCONNECTED,
    val activeMode: ConnectivityMode = ConnectivityMode.AUTO,
    val activeProfileName: String = "Campus Auto-Pilot",
    val assignedIp: String = "172.16.0.2",
    val egressIp: String = "Scanning...",
    val latencyMs: Long = 0,
    val txBytes: Long = 0,
    val rxBytes: Long = 0,
    val txSpeedKbps: Double = 0.0,
    val rxSpeedKbps: Double = 0.0,
    val errorMessage: String? = null
)
