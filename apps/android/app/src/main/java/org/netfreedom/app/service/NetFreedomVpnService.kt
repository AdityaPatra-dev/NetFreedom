package org.netfreedom.app.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import org.netfreedom.app.MainActivity
import org.netfreedom.app.R
import org.netfreedom.app.data.ConnectivityMode
import org.netfreedom.app.data.NetworkProfile
import org.netfreedom.app.jni.NetFreedomNative
import java.io.IOException

class NetFreedomVpnService : VpnService() {

    private var vpnInterface: ParcelFileDescriptor? = null
    private var statsJob: Job? = null
    private val serviceScope = CoroutineScope(Dispatchers.IO)

    companion object {
        private const val TAG = "NetFreedomVpnService"
        const val ACTION_CONNECT = "org.netfreedom.app.ACTION_CONNECT"
        const val ACTION_DISCONNECT = "org.netfreedom.app.ACTION_DISCONNECT"
        const val EXTRA_MODE = "org.netfreedom.app.EXTRA_MODE"

        private const val NOTIFICATION_ID = 1001
        private const val CHANNEL_ID = "netfreedom_vpn_channel"

        private val _vpnState = MutableStateFlow(VpnState())
        val vpnState: StateFlow<VpnState> = _vpnState.asStateFlow()

        fun start(context: Context, mode: ConnectivityMode = ConnectivityMode.AUTO) {
            val intent = Intent(context, NetFreedomVpnService::class.java).apply {
                action = ACTION_CONNECT
                putExtra(EXTRA_MODE, mode.name)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            val intent = Intent(context, NetFreedomVpnService::class.java).apply {
                action = ACTION_DISCONNECT
            }
            context.startService(intent)
        }
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_CONNECT -> {
                val modeStr = intent.getStringExtra(EXTRA_MODE) ?: ConnectivityMode.AUTO.name
                val mode = ConnectivityMode.valueOf(modeStr)
                startVpnTunnel(mode)
            }
            ACTION_DISCONNECT -> {
                stopVpnTunnel()
            }
        }
        return START_NOT_STICKY
    }

    private fun startVpnTunnel(mode: ConnectivityMode) {
        _vpnState.value = _vpnState.value.copy(
            status = TunnelStatus.CONNECTING,
            activeMode = mode
        )

        startForeground(NOTIFICATION_ID, buildNotification("Connecting to ${mode.label}..."))

        serviceScope.launch {
            try {
                // 1. If ByeDPI or Hybrid or Auto, spin up native ByeDPI daemon
                val isDpiNeeded = mode == ConnectivityMode.BYEDPI ||
                        mode == ConnectivityMode.HYBRID ||
                        mode == ConnectivityMode.AUTO

                if (isDpiNeeded) {
                    val profile = when (mode) {
                        ConnectivityMode.BYEDPI -> NetworkProfile.DEFAULT_BYEDPI
                        ConnectivityMode.HYBRID -> NetworkProfile.DEFAULT_HYBRID
                        else -> NetworkProfile.DEFAULT_AUTO
                    }
                    val args = profile.dpiConfig.toCliArgs()
                    val dpiStarted = NetFreedomNative.startByeDpi(applicationContext, args)
                    if (!dpiStarted) {
                        throw IOException("ByeDPI engine failed to start on this architecture")
                    }
                }

                // 2. Establish Android VpnService TUN
                val builder = Builder()
                    .setSession("NetFreedom - ${mode.label}")
                    .addAddress("172.16.0.2", 32)
                    .addRoute("0.0.0.0", 0)
                    .addDnsServer("198.18.0.2")
                    .setMtu(1420)
                    .setBlocking(false)

                // Disallow NetFreedom app from own VPN to avoid black-hole routing loop
                try {
                    builder.addDisallowedApplication(packageName)
                } catch (e: Exception) {
                    Log.w(TAG, "Could not disallow own package: ${e.message}")
                }

                val pfd = builder.establish()
                if (pfd == null) {
                    throw IOException("VpnService.Builder.establish() returned null")
                }
                vpnInterface = pfd

                // 3. Generate hev-socks5-tunnel config
                val isEmulator = Build.FINGERPRINT.startsWith("generic")
                    || Build.FINGERPRINT.startsWith("unknown")
                    || Build.MODEL.contains("google_sdk")
                    || Build.MODEL.contains("Emulator")
                    || Build.MODEL.contains("Android SDK built for x86")
                    || Build.HARDWARE.contains("goldfish")
                    || Build.HARDWARE.contains("ranchu")

                var socksHost = "127.0.0.1"
                if (isEmulator && isDpiNeeded) {
                    // In QEMU emulator, guest IP_TTL is stripped by user-mode slirp, causing fake packet
                    // to reach Cloudflare / target servers directly and trigger TLS decode error (562).
                    // If host ByeDPI proxy is running on 10.0.2.2:1080, route through it to preserve raw TTL desync.
                    try {
                        java.net.Socket().use { s ->
                            s.connect(java.net.InetSocketAddress("10.0.2.2", 1080), 600)
                            socksHost = "10.0.2.2"
                            Log.i(TAG, "Android Emulator detected: bridging to host ByeDPI engine at 10.0.2.2:1080")
                        }
                    } catch (e: Exception) {
                        Log.i(TAG, "Android Emulator: host 10.0.2.2:1080 unreachable, using local daemon at 127.0.0.1")
                    }
                }

                val socksPort = if (isDpiNeeded) 1080 else 1080
                val configFile = NetFreedomNative.createHevConfig(
                    cacheDir = cacheDir,
                    socksHost = socksHost,
                    socksPort = socksPort,
                    mtu = 1420
                )

                // 4. Start hev-socks5-tunnel with the TUN fd
                val tunFd = pfd.fd
                val hevStarted = NetFreedomNative.startHevTunnel(configFile, tunFd)
                if (!hevStarted) {
                    throw IOException("Failed to start hev-socks5-tunnel with fd $tunFd")
                }

                _vpnState.value = _vpnState.value.copy(
                    status = TunnelStatus.CONNECTED,
                    activeMode = mode,
                    activeProfileName = mode.label,
                    egressIp = if (isDpiNeeded) "DPI Desync Protected" else "1.1.1.1 Cloudflare",
                    errorMessage = null
                )

                updateNotification("Connected • ${mode.label}")
                startStatsCollector()

            } catch (e: Exception) {
                Log.e(TAG, "Failed to start VPN tunnel", e)
                _vpnState.value = _vpnState.value.copy(
                    status = TunnelStatus.ERROR,
                    errorMessage = e.message ?: "Tunnel failed to establish"
                )
                stopVpnTunnel()
            }
        }
    }

    private fun startStatsCollector() {
        statsJob?.cancel()
        statsJob = serviceScope.launch {
            var lastTxBytes = 0L
            var lastRxBytes = 0L

            while (isActive) {
                delay(1000)
                val stats = NetFreedomNative.getTunnelStats()
                val txBytes = stats[1]
                val rxBytes = stats[3]

                val txDelta = if (lastTxBytes > 0 && txBytes >= lastTxBytes) txBytes - lastTxBytes else 0L
                val rxDelta = if (lastRxBytes > 0 && rxBytes >= lastRxBytes) rxBytes - lastRxBytes else 0L

                lastTxBytes = txBytes
                lastRxBytes = rxBytes

                val txSpeedKbps = (txDelta * 8) / 1000.0
                val rxSpeedKbps = (rxDelta * 8) / 1000.0

                _vpnState.value = _vpnState.value.copy(
                    txBytes = txBytes,
                    rxBytes = rxBytes,
                    txSpeedKbps = txSpeedKbps,
                    rxSpeedKbps = rxSpeedKbps
                )

                val content = String.format(
                    "↓ %.1f kB/s  ↑ %.1f kB/s  • Total: %.1f MB",
                    rxSpeedKbps / 8, txSpeedKbps / 8, (txBytes + rxBytes) / (1024.0 * 1024.0)
                )
                updateNotification(content)
            }
        }
    }

    private fun stopVpnTunnel() {
        _vpnState.value = _vpnState.value.copy(status = TunnelStatus.DISCONNECTING)
        statsJob?.cancel()
        statsJob = null

        serviceScope.launch {
            NetFreedomNative.stopHevTunnel()
            NetFreedomNative.stopByeDpi()

            try {
                vpnInterface?.close()
            } catch (ignored: Exception) {}
            vpnInterface = null

            _vpnState.value = VpnState(status = TunnelStatus.DISCONNECTED)
            stopForeground(STOP_FOREGROUND_REMOVE)
            stopSelf()
        }
    }

    override fun onDestroy() {
        stopVpnTunnel()
        super.onDestroy()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                getString(R.string.channel_name),
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = getString(R.string.channel_description)
                setShowBadge(false)
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(contentText: String): Notification {
        val launchIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP
        }
        val pLaunch = PendingIntent.getActivity(
            this, 0, launchIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val disconnectIntent = Intent(this, NetFreedomVpnService::class.java).apply {
            action = ACTION_DISCONNECT
        }
        val pDisconnect = PendingIntent.getService(
            this, 1, disconnectIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_vpn_key)
            .setContentTitle(getString(R.string.app_name))
            .setContentText(contentText)
            .setContentIntent(pLaunch)
            .addAction(R.drawable.ic_vpn_key, getString(R.string.action_disconnect), pDisconnect)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    private fun updateNotification(contentText: String) {
        val manager = getSystemService(NotificationManager::class.java)
        manager.notify(NOTIFICATION_ID, buildNotification(contentText))
    }
}
