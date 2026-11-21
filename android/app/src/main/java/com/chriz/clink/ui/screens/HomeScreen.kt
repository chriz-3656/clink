package com.chriz.clink.ui.screens

import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.chriz.clink.R
import com.chriz.clink.data.network.ConnectionStatus
import com.chriz.clink.viewmodel.MainViewModel
import com.journeyapps.barcodescanner.ScanContract
import com.journeyapps.barcodescanner.ScanOptions

@Composable
fun HomeScreen(
    viewModel: MainViewModel,
    onNavigateDrawing: () -> Unit,
    onNavigateSettings: () -> Unit,
    onNavigateSessions: () -> Unit,
    modifier: Modifier = Modifier
) {
    val status by viewModel.client.status.collectAsState()
    val latency by viewModel.client.latencyMs.collectAsState()
    val displayInfo by viewModel.client.displayInfo.collectAsState()
    val pairingError by viewModel.pairingError.collectAsState()

    var isWifiMode by remember { mutableStateOf(false) }
    var wifiHost by remember { mutableStateOf(viewModel.preferences.host) }
    var wifiPort by remember { mutableStateOf(viewModel.preferences.port.toString()) }
    var showPinDialog by remember { mutableStateOf(false) }
    var pinInput by remember { mutableStateOf("") }
    var showQrDialog by remember { mutableStateOf(false) }
    var qrPayloadInput by remember { mutableStateOf("") }
    val context = LocalContext.current

    val qrScannerLauncher = rememberLauncherForActivityResult(ScanContract()) { result ->
        if (result.contents != null) {
            val qrText = result.contents.trim()
            val success = viewModel.handleQrPayload(qrText)
            if (success) {
                Toast.makeText(context, "QR code recognized! Connecting...", Toast.LENGTH_SHORT).show()
            } else {
                viewModel.setPairingError("Unrecognized QR format.")
                Toast.makeText(context, "Invalid QR code format.", Toast.LENGTH_LONG).show()
            }
        }
    }

    LaunchedEffect(status) {
        if (status == ConnectionStatus.PAIRING_REQUIRED) {
            showPinDialog = true
        } else if (status == ConnectionStatus.AUTHENTICATED) {
            showPinDialog = false
        }
    }

    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                horizontalArrangement = Arrangement.End
            ) {
                IconButton(onClick = onNavigateSessions) {
                    Icon(Icons.Default.Folder, contentDescription = "Saved Sessions")
                }
                IconButton(onClick = onNavigateSettings) {
                    Icon(Icons.Default.Settings, contentDescription = "Settings")
                }
            }
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 24.dp)
                .verticalScroll(rememberScrollState()),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Spacer(Modifier.height(16.dp))

            // App Logo & Header
            Image(
                painter = painterResource(id = R.drawable.ic_clink_logo),
                contentDescription = "Clink Logo",
                modifier = Modifier.size(80.dp)
            )
            Spacer(Modifier.height(12.dp))
            Text(
                text = "Clink",
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "Your Android. Your Desktop. One Canvas.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f),
                textAlign = TextAlign.Center
            )

            Spacer(Modifier.height(32.dp))

            // Connection Status Card
            Card(
                shape = RoundedCornerShape(20.dp),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(20.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text("Connection Status", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
                        val (statusText, statusColor) = when (status) {
                            ConnectionStatus.AUTHENTICATED -> "Ready & Active" to Color(0xFF00E676)
                            ConnectionStatus.CONNECTED -> "Connecting..." to Color(0xFFFFEA00)
                            ConnectionStatus.CONNECTING -> "Connecting..." to Color(0xFF00E5FF)
                            ConnectionStatus.PAIRING_REQUIRED -> "Pairing Needed" to Color(0xFFFF9100)
                            ConnectionStatus.ERROR -> "Connection Failed" to Color(0xFFFF3366)
                            ConnectionStatus.DISCONNECTED -> "Offline" to Color(0xFF9E9E9E)
                        }
                        Surface(
                            shape = RoundedCornerShape(12.dp),
                            color = statusColor.copy(alpha = 0.15f)
                        ) {
                            Text(
                                text = statusText,
                                color = statusColor,
                                fontWeight = FontWeight.Bold,
                                fontSize = 12.sp,
                                modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp)
                            )
                        }
                    }

                    if (status == ConnectionStatus.AUTHENTICATED) {
                        Spacer(Modifier.height(12.dp))
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween
                        ) {
                            Text(
                                text = "Display: ${displayInfo.width}x${displayInfo.height}",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.7f)
                            )
                            Text(
                                text = "Latency: ${latency}ms",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.primary,
                                fontWeight = FontWeight.Bold
                            )
                        }
                    }
                }
            }

            Spacer(Modifier.height(20.dp))

            // Transport Selector (USB vs Wi-Fi)
            SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                SegmentedButton(
                    shape = SegmentedButtonDefaults.itemShape(index = 0, count = 2),
                    selected = !isWifiMode,
                    onClick = { isWifiMode = false }
                ) {
                    Icon(Icons.Default.Usb, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(Modifier.width(6.dp))
                    Text("USB (ADB)")
                }
                SegmentedButton(
                    shape = SegmentedButtonDefaults.itemShape(index = 1, count = 2),
                    selected = isWifiMode,
                    onClick = { isWifiMode = true }
                ) {
                    Icon(Icons.Default.Wifi, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(Modifier.width(6.dp))
                    Text("Wi-Fi LAN")
                }
            }

            Spacer(Modifier.height(16.dp))

            if (!isWifiMode) {
                // USB instructions card
                Card(
                    shape = RoundedCornerShape(16.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.6f)),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("USB Quick Connect", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                        Spacer(Modifier.height(6.dp))
                        Text(
                            "1. Connect phone with USB Debugging enabled\n2. Open Clink Control Center on Linux (or run clink connect)\n3. Tap 'Connect via USB' below.",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f)
                        )
                    }
                }
                Spacer(Modifier.height(16.dp))
                Button(
                    onClick = { viewModel.connectUsb() },
                    modifier = Modifier.fillMaxWidth().height(50.dp),
                    shape = RoundedCornerShape(14.dp)
                ) {
                    Icon(Icons.Default.Usb, contentDescription = null)
                    Spacer(Modifier.width(8.dp))
                    Text("Connect via USB", fontWeight = FontWeight.SemiBold)
                }
            } else {
                // Wi-Fi inputs
                OutlinedTextField(
                    value = wifiHost,
                    onValueChange = { wifiHost = it },
                    label = { Text("Linux Host IP Address") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(14.dp)
                )
                Spacer(Modifier.height(10.dp))
                OutlinedTextField(
                    value = wifiPort,
                    onValueChange = { wifiPort = it },
                    label = { Text("Port (Default: 8765)") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(14.dp)
                )
                Spacer(Modifier.height(16.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    Button(
                        onClick = {
                            val p = wifiPort.toIntOrNull() ?: 8765
                            viewModel.connectWifi(wifiHost.trim(), p)
                        },
                        modifier = Modifier.weight(1f).height(50.dp),
                        shape = RoundedCornerShape(14.dp)
                    ) {
                        Icon(Icons.Default.Wifi, contentDescription = null)
                        Spacer(Modifier.width(6.dp))
                        Text("Connect Wi-Fi")
                    }
                    FilledTonalButton(
                        onClick = {
                            val options = ScanOptions().apply {
                                setPrompt("Point camera at Desktop Companion QR Code")
                                setBeepEnabled(true)
                                setOrientationLocked(false)
                                setBarcodeImageEnabled(false)
                            }
                            qrScannerLauncher.launch(options)
                        },
                        modifier = Modifier.weight(1f).height(50.dp),
                        shape = RoundedCornerShape(14.dp)
                    ) {
                        Icon(Icons.Default.QrCodeScanner, contentDescription = null)
                        Spacer(Modifier.width(6.dp))
                        Text("Scan QR")
                    }
                }
                Spacer(Modifier.height(8.dp))
                TextButton(
                    onClick = { showQrDialog = true },
                    modifier = Modifier.align(Alignment.CenterHorizontally)
                ) {
                    Text("Or enter pairing URI / PIN manually", style = MaterialTheme.typography.bodySmall)
                }
            }

            if (status != ConnectionStatus.DISCONNECTED) {
                Spacer(Modifier.height(10.dp))
                OutlinedButton(
                    onClick = { viewModel.disconnect() },
                    modifier = Modifier.fillMaxWidth().height(46.dp),
                    shape = RoundedCornerShape(14.dp)
                ) {
                    Text("Disconnect")
                }
            }

            Spacer(Modifier.height(28.dp))

            // Primary Launch Action
            Button(
                onClick = onNavigateDrawing,
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary),
                modifier = Modifier.fillMaxWidth().height(56.dp),
                shape = RoundedCornerShape(16.dp)
            ) {
                Icon(Icons.Default.Gesture, contentDescription = null)
                Spacer(Modifier.width(8.dp))
                Text("Open Drawing Canvas", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
            }

            Spacer(Modifier.height(32.dp))
        }
    }

    // Pairing PIN Dialog
    if (showPinDialog) {
        AlertDialog(
            onDismissRequest = {
                showPinDialog = false
                viewModel.clearPairingError()
            },
            title = { Text("Enter Pairing PIN") },
            text = {
                Column {
                    Text("Enter the 6-digit PIN shown on your Linux screen (Clink Control Center):")
                    Spacer(Modifier.height(12.dp))
                    OutlinedTextField(
                        value = pinInput,
                        onValueChange = {
                            if (it.length <= 6) {
                                pinInput = it
                                viewModel.clearPairingError()
                            }
                        },
                        placeholder = { Text("e.g. 123456") },
                        singleLine = true,
                        isError = pairingError != null,
                        modifier = Modifier.fillMaxWidth()
                    )
                    if (pairingError != null) {
                        Spacer(Modifier.height(6.dp))
                        Text(
                            text = pairingError ?: "Invalid PIN. Please try again.",
                            color = MaterialTheme.colorScheme.error,
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                }
            },
            confirmButton = {
                Button(
                    onClick = {
                        if (pinInput.isNotBlank()) {
                            viewModel.submitPin(pinInput.trim())
                        }
                    },
                    enabled = pinInput.length == 6
                ) {
                    Text("Pair Device")
                }
            },
            dismissButton = {
                TextButton(onClick = {
                    showPinDialog = false
                    viewModel.clearPairingError()
                    viewModel.disconnect()
                }) {
                    Text("Cancel")
                }
            }
        )
    }

    // QR Instant Pairing Dialog
    if (showQrDialog) {
        AlertDialog(
            onDismissRequest = { showQrDialog = false },
            title = { Text("QR Code Instant Pairing") },
            text = {
                Column {
                    Text("Scan the QR code on your Linux Control Center or paste the pairing URI:")
                    Spacer(Modifier.height(12.dp))
                    OutlinedTextField(
                        value = qrPayloadInput,
                        onValueChange = { qrPayloadInput = it },
                        placeholder = { Text("clink://pair?host=... or 192.168.x.x:8765:pin") },
                        singleLine = false,
                        maxLines = 3,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            },
            confirmButton = {
                Button(
                    onClick = {
                        if (qrPayloadInput.isNotBlank()) {
                            viewModel.handleQrPayload(qrPayloadInput)
                            qrPayloadInput = ""
                            showQrDialog = false
                        }
                    }
                ) {
                    Text("Pair Instantly")
                }
            },
            dismissButton = {
                TextButton(onClick = { showQrDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }
}
