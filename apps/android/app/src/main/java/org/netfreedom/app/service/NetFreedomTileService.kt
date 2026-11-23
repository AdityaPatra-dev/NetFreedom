package org.netfreedom.app.service

import android.content.Intent
import android.graphics.drawable.Icon
import android.net.VpnService
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import androidx.annotation.RequiresApi
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import org.netfreedom.app.R
import org.netfreedom.app.data.ConnectivityMode

@RequiresApi(Build.VERSION_CODES.N)
class NetFreedomTileService : TileService() {

    private val serviceScope = CoroutineScope(Dispatchers.Main)
    private var stateJob: Job? = null

    override fun onStartListening() {
        super.onStartListening()
        stateJob?.cancel()
        stateJob = serviceScope.launch {
            NetFreedomVpnService.vpnState.collect { state ->
                updateTileState(state)
            }
        }
    }

    override fun onStopListening() {
        stateJob?.cancel()
        super.onStopListening()
    }

    override fun onClick() {
        super.onClick()
        val currentState = NetFreedomVpnService.vpnState.value
        if (currentState.status == TunnelStatus.CONNECTED || currentState.status == TunnelStatus.CONNECTING) {
            NetFreedomVpnService.stop(this)
        } else {
            val vpnIntent = VpnService.prepare(this)
            if (vpnIntent != null) {
                // Must grant VPN permission via Activity
                val intent = Intent(this, org.netfreedom.app.MainActivity::class.java).apply {
                    flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP
                }
                startActivityAndCollapse(intent)
            } else {
                NetFreedomVpnService.start(this, ConnectivityMode.AUTO)
            }
        }
    }

    private fun updateTileState(vpnState: VpnState) {
        val tile = qsTile ?: return
        when (vpnState.status) {
            TunnelStatus.CONNECTED -> {
                tile.state = Tile.STATE_ACTIVE
                tile.label = "NetFreedom (On)"
                tile.subtitle = vpnState.activeMode.label
                tile.icon = Icon.createWithResource(this, R.drawable.ic_vpn_key)
            }
            TunnelStatus.CONNECTING -> {
                tile.state = Tile.STATE_ACTIVE
                tile.label = "NetFreedom"
                tile.subtitle = "Connecting..."
            }
            TunnelStatus.DISCONNECTING -> {
                tile.state = Tile.STATE_INACTIVE
                tile.label = "NetFreedom"
                tile.subtitle = "Disconnecting..."
            }
            else -> {
                tile.state = Tile.STATE_INACTIVE
                tile.label = "NetFreedom"
                tile.subtitle = "Disconnected"
                tile.icon = Icon.createWithResource(this, R.drawable.ic_vpn_key)
            }
        }
        tile.updateTile()
    }
}
