package org.netfreedom.app

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Home
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import org.netfreedom.app.data.ConnectivityMode
import org.netfreedom.app.data.DpiConfig
import org.netfreedom.app.service.NetFreedomVpnService
import org.netfreedom.app.service.TunnelStatus
import org.netfreedom.app.ui.screens.DashboardScreen
import org.netfreedom.app.ui.screens.DiagnosticsScreen
import org.netfreedom.app.ui.screens.DpiLabScreen
import org.netfreedom.app.ui.theme.CyanPrimary
import org.netfreedom.app.ui.theme.DarkBackground
import org.netfreedom.app.ui.theme.DarkCard
import org.netfreedom.app.ui.theme.DarkSurface
import org.netfreedom.app.ui.theme.NetFreedomTheme
import org.netfreedom.app.ui.theme.TextMuted
import org.netfreedom.app.ui.theme.TextPrimary

enum class AppTab(val label: String) {
    DASHBOARD("Dashboard"),
    DPI_LAB("DPI Lab"),
    DIAGNOSTICS("Diagnostics")
}

class MainActivity : ComponentActivity() {

    private var pendingMode: ConnectivityMode = ConnectivityMode.AUTO

    private val vpnPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == Activity.RESULT_OK) {
            NetFreedomVpnService.start(this, pendingMode)
        } else {
            Toast.makeText(this, "VPN permission is required to bypass restrictions", Toast.LENGTH_SHORT).show()
        }
    }

    private fun handleToggleVpn(mode: ConnectivityMode) {
        val currentState = NetFreedomVpnService.vpnState.value
        if (currentState.status == TunnelStatus.CONNECTED || currentState.status == TunnelStatus.CONNECTING) {
            NetFreedomVpnService.stop(this)
        } else {
            pendingMode = mode
            val prepareIntent = VpnService.prepare(this)
            if (prepareIntent != null) {
                vpnPermissionLauncher.launch(prepareIntent)
            } else {
                NetFreedomVpnService.start(this, mode)
            }
        }
    }

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            NetFreedomTheme {
                var currentTab by remember { mutableStateOf(AppTab.DASHBOARD) }
                var selectedMode by remember { mutableStateOf(ConnectivityMode.AUTO) }
                var dpiConfig by remember { mutableStateOf(DpiConfig()) }
                val vpnState by NetFreedomVpnService.vpnState.collectAsState()

                Scaffold(
                    topBar = {
                        TopAppBar(
                            title = {
                                Text(
                                    text = "NetFreedom",
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 20.sp,
                                    color = TextPrimary
                                )
                            },
                            colors = TopAppBarDefaults.topAppBarColors(
                                containerColor = DarkBackground
                            )
                        )
                    },
                    bottomBar = {
                        NavigationBar(
                            containerColor = DarkSurface
                        ) {
                            NavigationBarItem(
                                selected = currentTab == AppTab.DASHBOARD,
                                onClick = { currentTab = AppTab.DASHBOARD },
                                icon = { Icon(Icons.Default.Home, contentDescription = "Dashboard") },
                                label = { Text(AppTab.DASHBOARD.label) },
                                colors = NavigationBarItemDefaults.colors(
                                    selectedIconColor = CyanPrimary,
                                    selectedTextColor = CyanPrimary,
                                    unselectedIconColor = TextMuted,
                                    unselectedTextColor = TextMuted,
                                    indicatorColor = DarkCard
                                )
                            )
                            NavigationBarItem(
                                selected = currentTab == AppTab.DPI_LAB,
                                onClick = { currentTab = AppTab.DPI_LAB },
                                icon = { Icon(Icons.Default.Build, contentDescription = "DPI Lab") },
                                label = { Text(AppTab.DPI_LAB.label) },
                                colors = NavigationBarItemDefaults.colors(
                                    selectedIconColor = CyanPrimary,
                                    selectedTextColor = CyanPrimary,
                                    unselectedIconColor = TextMuted,
                                    unselectedTextColor = TextMuted,
                                    indicatorColor = DarkCard
                                )
                            )
                            NavigationBarItem(
                                selected = currentTab == AppTab.DIAGNOSTICS,
                                onClick = { currentTab = AppTab.DIAGNOSTICS },
                                icon = { Icon(Icons.Default.CheckCircle, contentDescription = "Diagnostics") },
                                label = { Text(AppTab.DIAGNOSTICS.label) },
                                colors = NavigationBarItemDefaults.colors(
                                    selectedIconColor = CyanPrimary,
                                    selectedTextColor = CyanPrimary,
                                    unselectedIconColor = TextMuted,
                                    unselectedTextColor = TextMuted,
                                    indicatorColor = DarkCard
                                )
                            )
                        }
                    }
                ) { innerPadding ->
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(DarkBackground)
                            .padding(innerPadding)
                    ) {
                        when (currentTab) {
                            AppTab.DASHBOARD -> DashboardScreen(
                                vpnState = vpnState,
                                selectedMode = selectedMode,
                                onModeSelect = { selectedMode = it },
                                onToggleVpn = { handleToggleVpn(selectedMode) }
                            )
                            AppTab.DPI_LAB -> DpiLabScreen(
                                currentConfig = dpiConfig,
                                onSaveConfig = { dpiConfig = it }
                            )
                            AppTab.DIAGNOSTICS -> DiagnosticsScreen()
                        }
                    }
                }
            }
        }
    }
}
