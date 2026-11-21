package com.chriz.clink.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Save
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.chriz.clink.ui.components.DrawingCanvas
import com.chriz.clink.ui.components.FloatingDrawingToolbar
import com.chriz.clink.ui.components.ModeSegmentedControl
import com.chriz.clink.viewmodel.MainViewModel

@Composable
fun DrawingScreen(
    viewModel: MainViewModel,
    onNavigateBack: () -> Unit,
    modifier: Modifier = Modifier
) {
    val mode by viewModel.mode.collectAsState()
    val tool by viewModel.currentTool.collectAsState()
    val color by viewModel.currentColor.collectAsState()
    val width by viewModel.currentWidth.collectAsState()
    val strokes by viewModel.strokes.collectAsState()
    val activeStroke by viewModel.activeStroke.collectAsState()
    val laserPos by viewModel.laserPos.collectAsState()

    var showSaveDialog by remember { mutableStateOf(false) }
    var sessionNameInput by remember { mutableStateOf("") }

    Box(modifier = modifier.fillMaxSize()) {
        // Main Drawing Canvas
        DrawingCanvas(
            mode = mode,
            currentTool = tool,
            currentColor = color,
            currentWidth = width,
            strokes = strokes,
            activeStroke = activeStroke,
            laserPosition = laserPos,
            palmRejectionEnabled = viewModel.preferences.palmRejectionEnabled,
            onStrokeStart = { x, y, p -> viewModel.startStroke(x, y, p) },
            onStrokeMove = { pts -> viewModel.appendStrokePoints(pts) },
            onStrokeEnd = { x, y, p -> viewModel.endStroke(x, y, p) },
            onCursorMove = { dx, dy, x, y, p -> viewModel.sendCursorMove(dx, dy, x, y, p) },
            onPointerDown = { btn, x, y, p -> viewModel.sendPointerDown(btn, x, y, p) },
            onPointerUp = { btn, x, y -> viewModel.sendPointerUp(btn, x, y) },
            modifier = Modifier.fillMaxSize()
        )

        // Top Navigation and Mode Toggle Bar
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .statusBarsPadding()
                .padding(horizontal = 16.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            FilledTonalIconButton(onClick = onNavigateBack) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
            }

            ModeSegmentedControl(
                selectedMode = mode,
                onModeSelected = { viewModel.setMode(it) }
            )

            FilledTonalIconButton(onClick = { showSaveDialog = true }) {
                Icon(Icons.Default.Save, contentDescription = "Save Session")
            }
        }

        // Floating Bottom Toolbar
        FloatingDrawingToolbar(
            currentTool = tool,
            currentColor = color,
            currentWidth = width,
            canUndo = strokes.isNotEmpty(),
            canRedo = true,
            onToolSelected = { viewModel.setTool(it) },
            onColorSelected = { viewModel.setColor(it) },
            onWidthChanged = { viewModel.setWidth(it) },
            onUndo = { viewModel.undo() },
            onRedo = { viewModel.redo() },
            onClear = { viewModel.clearCanvas() },
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .navigationBarsPadding()
                .padding(bottom = 16.dp)
        )
    }

    if (showSaveDialog) {
        AlertDialog(
            onDismissRequest = { showSaveDialog = false },
            title = { Text("Save Session") },
            text = {
                OutlinedTextField(
                    value = sessionNameInput,
                    onValueChange = { sessionNameInput = it },
                    label = { Text("Session Name") },
                    placeholder = { Text("e.g. Architecture Diagram") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
            },
            confirmButton = {
                Button(onClick = {
                    viewModel.saveSession(sessionNameInput.trim())
                    sessionNameInput = ""
                    showSaveDialog = false
                }) {
                    Text("Save")
                }
            },
            dismissButton = {
                TextButton(onClick = { showSaveDialog = false }) {
                    Text("Cancel")
                }
            }
        )
    }
}
