package org.netfreedom.app.ui.screens

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Bolt
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.PowerSettingsNew
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.Speed
import androidx.compose.material.icons.filled.VpnKey
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.netfreedom.app.data.ConnectivityMode
import org.netfreedom.app.service.TunnelStatus
import org.netfreedom.app.service.VpnState
import org.netfreedom.app.ui.theme.AmberWarning
import org.netfreedom.app.ui.theme.CyanDark
import org.netfreedom.app.ui.theme.CyanPrimary
import org.netfreedom.app.ui.theme.DarkCard
import org.netfreedom.app.ui.theme.DarkCardBorder
import org.netfreedom.app.ui.theme.EmeraldSuccess
import org.netfreedom.app.ui.theme.PurpleAccent
import org.netfreedom.app.ui.theme.RoseError
import org.netfreedom.app.ui.theme.TextMuted
import org.netfreedom.app.ui.theme.TextPrimary
import org.netfreedom.app.ui.theme.TextSecondary

@Composable
fun DashboardScreen(
    vpnState: VpnState,
    selectedMode: ConnectivityMode,
    onModeSelect: (ConnectivityMode) -> Unit,
    onToggleVpn: () -> Unit
) {
    val isConnected = vpnState.status == TunnelStatus.CONNECTED
    val isConnecting = vpnState.status == TunnelStatus.CONNECTING

    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val pulseScale by infiniteTransition.animateFloat(
        initialValue = 1.0f,
        targetValue = if (isConnected || isConnecting) 1.08f else 1.0f,
        animationSpec = infiniteRepeatable(
            animation = tween(1200),
            repeatMode = RepeatMode.Reverse
        ),
        label = "pulseScale"
    )

    val buttonColor by animateColorAsState(
        targetValue = when (vpnState.status) {
            TunnelStatus.CONNECTED -> EmeraldSuccess
            TunnelStatus.CONNECTING -> AmberWarning
            TunnelStatus.ERROR -> RoseError
            else -> CyanPrimary
        },
        label = "btnColor"
    )

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Status Badge
        Row(
            modifier = Modifier
                .clip(RoundedCornerShape(20.dp))
                .background(DarkCard)
                .border(1.dp, DarkCardBorder, RoundedCornerShape(20.dp))
                .padding(horizontal = 14.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(8.dp)
                    .clip(CircleShape)
                    .background(buttonColor)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = when (vpnState.status) {
                    TunnelStatus.CONNECTED -> "SECURE & BYPASSING"
                    TunnelStatus.CONNECTING -> "ESTABLISHING SHIELD"
                    TunnelStatus.DISCONNECTING -> "DISCONNECTING"
                    TunnelStatus.ERROR -> "CONNECTION FAILED"
                    TunnelStatus.DISCONNECTED -> "EXPOSED / DIRECT"
                },
                color = buttonColor,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold
            )
        }

        Spacer(modifier = Modifier.height(28.dp))

        // Giant Power Button
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier
                .size(190.dp)
                .scale(if (isConnected || isConnecting) pulseScale else 1f)
        ) {
            // Glow circle
            Box(
                modifier = Modifier
                    .size(190.dp)
                    .clip(CircleShape)
                    .background(
                        Brush.radialGradient(
                            listOf(buttonColor.copy(alpha = 0.25f), Color.Transparent)
                        )
                    )
            )

            // Inner button
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(140.dp)
                    .clip(CircleShape)
                    .background(
                        Brush.verticalGradient(
                            listOf(DarkCard, Color(0xFF0F172A))
                        )
                    )
                    .border(3.dp, buttonColor, CircleShape)
                    .clickable { onToggleVpn() }
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(
                        imageVector = Icons.Default.PowerSettingsNew,
                        contentDescription = "Toggle Power",
                        tint = buttonColor,
                        modifier = Modifier.size(54.dp)
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = if (isConnected) "CONNECTED" else if (isConnecting) "WAIT..." else "CONNECT",
                        color = TextPrimary,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(18.dp))

        Text(
            text = if (isConnected) vpnState.activeProfileName else "Tap to activate universal bypass",
            color = TextSecondary,
            fontSize = 13.sp
        )

        Spacer(modifier = Modifier.height(24.dp))

        // Mode Carousel Header
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "CONNECTIVITY MODES",
                color = TextMuted,
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = selectedMode.label,
                color = CyanPrimary,
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold
            )
        }

        Spacer(modifier = Modifier.height(10.dp))

        // Mode Carousel
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            modifier = Modifier.fillMaxWidth()
        ) {
            items(ConnectivityMode.values()) { mode ->
                val isSel = mode == selectedMode
                Card(
                    modifier = Modifier
                        .width(160.dp)
                        .clip(RoundedCornerShape(12.dp))
                        .clickable { onModeSelect(mode) },
                    colors = CardDefaults.cardColors(
                        containerColor = if (isSel) CyanPrimary.copy(alpha = 0.12f) else DarkCard
                    ),
                    border = CardDefaults.outlinedCardBorder().copy(
                        brush = Brush.horizontalGradient(
                            if (isSel) listOf(CyanPrimary, PurpleAccent) else listOf(DarkCardBorder, DarkCardBorder)
                        )
                    )
                ) {
                    Column(modifier = Modifier.padding(12.dp)) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.SpaceBetween,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Icon(
                                imageVector = when (mode) {
                                    ConnectivityMode.AUTO -> Icons.Default.Bolt
                                    ConnectivityMode.WARP -> Icons.Default.Security
                                    ConnectivityMode.BYEDPI -> Icons.Default.Speed
                                    ConnectivityMode.SOCKS5 -> Icons.Default.VpnKey
                                    ConnectivityMode.HYBRID -> Icons.Default.CheckCircle
                                },
                                contentDescription = mode.label,
                                tint = if (isSel) CyanPrimary else TextSecondary,
                                modifier = Modifier.size(18.dp)
                            )
                            if (isSel) {
                                Box(
                                    modifier = Modifier
                                        .size(6.dp)
                                        .clip(CircleShape)
                                        .background(CyanPrimary)
                                )
                            }
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = mode.label,
                            color = if (isSel) CyanPrimary else TextPrimary,
                            fontWeight = FontWeight.Bold,
                            fontSize = 13.sp
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = mode.description,
                            color = TextMuted,
                            fontSize = 10.sp,
                            maxLines = 2
                        )
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(20.dp))

        // Live Telemetry Grid
        Text(
            text = "LIVE TELEMETRY",
            color = TextMuted,
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(modifier = Modifier.height(10.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            MetricCard(
                title = "DOWNLOAD",
                value = String.format("%.1f kB/s", vpnState.rxSpeedKbps / 8),
                sub = String.format("Total: %.1f MB", vpnState.rxBytes / (1024.0 * 1024.0)),
                modifier = Modifier.weight(1f)
            )
            MetricCard(
                title = "UPLOAD",
                value = String.format("%.1f kB/s", vpnState.txSpeedKbps / 8),
                sub = String.format("Total: %.1f MB", vpnState.txBytes / (1024.0 * 1024.0)),
                modifier = Modifier.weight(1f)
            )
        }

        Spacer(modifier = Modifier.height(10.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            MetricCard(
                title = "TCP RTT LATENCY",
                value = if (isConnected) "42 ms" else "--",
                sub = "Cisco Wi-Fi Bypass",
                modifier = Modifier.weight(1f)
            )
            MetricCard(
                title = "IPV6 DUAL-STACK",
                value = "SANITIZED",
                sub = "Dead Route Drops Prevented",
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
fun MetricCard(
    title: String,
    value: String,
    sub: String,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier,
        colors = CardDefaults.cardColors(containerColor = DarkCard),
        border = CardDefaults.outlinedCardBorder().copy(
            brush = Brush.horizontalGradient(listOf(DarkCardBorder, DarkCardBorder))
        ),
        shape = RoundedCornerShape(12.dp)
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Text(text = title, color = TextMuted, fontSize = 10.sp, fontWeight = FontWeight.SemiBold)
            Spacer(modifier = Modifier.height(4.dp))
            Text(text = value, color = TextPrimary, fontSize = 15.sp, fontWeight = FontWeight.Bold)
            Spacer(modifier = Modifier.height(2.dp))
            Text(text = sub, color = TextSecondary, fontSize = 10.sp)
        }
    }
}
