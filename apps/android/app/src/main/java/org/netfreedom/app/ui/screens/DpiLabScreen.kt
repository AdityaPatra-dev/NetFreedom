package org.netfreedom.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.netfreedom.app.data.DpiConfig
import org.netfreedom.app.ui.theme.CyanPrimary
import org.netfreedom.app.ui.theme.DarkCard
import org.netfreedom.app.ui.theme.DarkCardBorder
import org.netfreedom.app.ui.theme.EmeraldSuccess
import org.netfreedom.app.ui.theme.TextMuted
import org.netfreedom.app.ui.theme.TextPrimary
import org.netfreedom.app.ui.theme.TextSecondary

@Composable
fun DpiLabScreen(
    currentConfig: DpiConfig,
    onSaveConfig: (DpiConfig) -> Unit
) {
    var splitPos by remember { mutableStateOf(currentConfig.splitPos) }
    var fakeTtl by remember { mutableFloatStateOf(currentConfig.fakeTtl.toFloat()) }
    var fakeSni by remember { mutableStateOf(currentConfig.fakeSni) }
    var splitHttp by remember { mutableStateOf(currentConfig.splitHttpReq) }
    var tfo by remember { mutableStateOf(currentConfig.tcpFastOpen) }
    var testResult by remember { mutableStateOf<String?>(null) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp)
    ) {
        Text(
            text = "DPI EVASION LAB",
            color = CyanPrimary,
            fontSize = 18.sp,
            fontWeight = FontWeight.Bold
        )
        Text(
            text = "Customize low-level TCP/TLS packet desynchronization parameters",
            color = TextSecondary,
            fontSize = 12.sp
        )

        Spacer(modifier = Modifier.height(20.dp))

        // Split Position Card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = DarkCard),
            border = CardDefaults.outlinedCardBorder().copy(
                brush = androidx.compose.ui.graphics.SolidColor(DarkCardBorder)
            ),
            shape = RoundedCornerShape(12.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "TLS SNI Split Position (-s)",
                    color = TextPrimary,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "Controls where the ClientHello record is cleaved into discrete TCP segments (e.g. 2+s splits right inside the SNI extension).",
                    color = TextMuted,
                    fontSize = 11.sp
                )
                Spacer(modifier = Modifier.height(10.dp))
                OutlinedTextField(
                    value = splitPos,
                    onValueChange = { splitPos = it },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = TextPrimary,
                        unfocusedTextColor = TextPrimary,
                        focusedBorderColor = CyanPrimary,
                        unfocusedBorderColor = DarkCardBorder
                    )
                )
            }
        }

        Spacer(modifier = Modifier.height(14.dp))

        // Fake Packet TTL Card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = DarkCard),
            border = CardDefaults.outlinedCardBorder().copy(
                brush = androidx.compose.ui.graphics.SolidColor(DarkCardBorder)
            ),
            shape = RoundedCornerShape(12.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = "Fake Packet TTL (-t)",
                        color = TextPrimary,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = "${fakeTtl.toInt()} hops",
                        color = CyanPrimary,
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold
                    )
                }
                Text(
                    text = "Spoofed TTL drops fake packet at middlebox router before reaching target server.",
                    color = TextMuted,
                    fontSize = 11.sp
                )
                Spacer(modifier = Modifier.height(6.dp))
                Slider(
                    value = fakeTtl,
                    onValueChange = { fakeTtl = it },
                    valueRange = 1f..10f,
                    steps = 8,
                    colors = SliderDefaults.colors(
                        thumbColor = CyanPrimary,
                        activeTrackColor = CyanPrimary,
                        inactiveTrackColor = DarkCardBorder
                    )
                )
            }
        }

        Spacer(modifier = Modifier.height(14.dp))

        // Fake SNI Domain
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = DarkCard),
            border = CardDefaults.outlinedCardBorder().copy(
                brush = androidx.compose.ui.graphics.SolidColor(DarkCardBorder)
            ),
            shape = RoundedCornerShape(12.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Text(
                    text = "Decoy SNI Domain (-d)",
                    color = TextPrimary,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "Injected inside the fake TCP segment to misdirect DPI state machines.",
                    color = TextMuted,
                    fontSize = 11.sp
                )
                Spacer(modifier = Modifier.height(10.dp))
                OutlinedTextField(
                    value = fakeSni,
                    onValueChange = { fakeSni = it },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = TextPrimary,
                        unfocusedTextColor = TextPrimary,
                        focusedBorderColor = CyanPrimary,
                        unfocusedBorderColor = DarkCardBorder
                    )
                )
            }
        }

        Spacer(modifier = Modifier.height(14.dp))

        // Toggles Card
        Card(
            modifier = Modifier.fillMaxWidth(),
            colors = CardDefaults.cardColors(containerColor = DarkCard),
            border = CardDefaults.outlinedCardBorder().copy(
                brush = androidx.compose.ui.graphics.SolidColor(DarkCardBorder)
            ),
            shape = RoundedCornerShape(12.dp)
        ) {
            Column(modifier = Modifier.padding(16.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(text = "HTTP Request Splitting", color = TextPrimary, fontSize = 13.sp, fontWeight = FontWeight.Bold)
                        Text(text = "Cleave HTTP GET/POST headers into 2 packets", color = TextMuted, fontSize = 11.sp)
                    }
                    Switch(
                        checked = splitHttp,
                        onCheckedChange = { splitHttp = it },
                        colors = SwitchDefaults.colors(checkedThumbColor = CyanPrimary, checkedTrackColor = CyanPrimary.copy(alpha = 0.3f))
                    )
                }
                Spacer(modifier = Modifier.height(12.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(text = "TCP Fast Open (TFO)", color = TextPrimary, fontSize = 13.sp, fontWeight = FontWeight.Bold)
                        Text(text = "Send data inside SYN packet (-F)", color = TextMuted, fontSize = 11.sp)
                    }
                    Switch(
                        checked = tfo,
                        onCheckedChange = { tfo = it },
                        colors = SwitchDefaults.colors(checkedThumbColor = CyanPrimary, checkedTrackColor = CyanPrimary.copy(alpha = 0.3f))
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(18.dp))

        // Save & Test buttons
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Button(
                onClick = {
                    val updated = currentConfig.copy(
                        splitPos = splitPos,
                        fakeTtl = fakeTtl.toInt(),
                        fakeSni = fakeSni,
                        splitHttpReq = splitHttp,
                        tcpFastOpen = tfo
                    )
                    onSaveConfig(updated)
                    testResult = "Settings saved & applied to tunnel engine."
                },
                colors = ButtonDefaults.buttonColors(containerColor = CyanPrimary),
                shape = RoundedCornerShape(10.dp),
                modifier = Modifier.weight(1f)
            ) {
                Text(text = "Apply Settings", color = Color.Black, fontWeight = FontWeight.Bold)
            }
        }

        testResult?.let { msg ->
            Spacer(modifier = Modifier.height(12.dp))
            Text(text = msg, color = EmeraldSuccess, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        }
    }
}
