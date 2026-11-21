package com.chriz.clink.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.chriz.clink.viewmodel.MainViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    viewModel: MainViewModel,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    val prefs = viewModel.preferences
    var host by remember { mutableStateOf(prefs.host) }
    var port by remember { mutableStateOf(prefs.port.toString()) }
    var palmRejection by remember { mutableStateOf(prefs.palmRejectionEnabled) }
    var sensitivity by remember { mutableStateOf(prefs.stylusSensitivity) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings", fontWeight = FontWeight.Bold) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                }
            )
        },
        modifier = modifier.fillMaxSize()
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 20.dp)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            Text("Connection Defaults", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)

            OutlinedTextField(
                value = host,
                onValueChange = { host = it; prefs.host = it },
                label = { Text("Default Host IP") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(12.dp)
            )

            OutlinedTextField(
                value = port,
                onValueChange = {
                    port = it
                    it.toIntOrNull()?.let { p -> prefs.port = p }
                },
                label = { Text("Default Port") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(12.dp)
            )

            HorizontalDivider()

            Text("Stylus & Touch", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text("Palm Rejection", fontWeight = FontWeight.Medium)
                    Text(
                        "Ignore large accidental touch contacts when stylus is active",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f)
                    )
                }
                Switch(
                    checked = palmRejection,
                    onCheckedChange = {
                        palmRejection = it
                        prefs.palmRejectionEnabled = it
                    }
                )
            }

            Column {
                Text("Tablet Pointer Sensitivity", fontWeight = FontWeight.Medium)
                Slider(
                    value = sensitivity,
                    onValueChange = {
                        sensitivity = it
                        prefs.stylusSensitivity = it
                    },
                    valueRange = 0.5f..3.0f,
                    steps = 5
                )
            }

            HorizontalDivider()

            Text("About Clink", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
            Text("Version: 1.0.0 (Protocol v1.0)")
            Text("Open-source Android & Linux Canvas by chriz-3656")
            Text("License: MIT")

            Spacer(Modifier.height(24.dp))
        }
    }
}
