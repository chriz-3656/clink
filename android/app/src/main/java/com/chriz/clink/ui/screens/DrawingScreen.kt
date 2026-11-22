package com.chriz.clink.ui.screens

import androidx.compose.animation.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowRightAlt
import androidx.compose.material.icons.automirrored.filled.Undo
import androidx.compose.material.icons.automirrored.filled.Redo
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.chriz.clink.data.model.DrawingTool
import com.chriz.clink.ui.components.DrawingCanvas
import com.chriz.clink.ui.components.FloatingDrawingToolbar
import com.chriz.clink.ui.components.ModeSegmentedControl
import com.chriz.clink.viewmodel.MainViewModel
import kotlinx.coroutines.delay

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
    val canUndo by viewModel.canUndo.collectAsState()
    val canRedo by viewModel.canRedo.collectAsState()

    var showSaveDialog by remember { mutableStateOf(false) }
    var sessionNameInput by remember { mutableStateOf("") }
    var toolToastText by remember { mutableStateOf<String?>(null) }
    var toolToastVisible by remember { mutableStateOf(false) }
    var timerSec by remember { mutableStateOf(viewModel.preferences.annotationTimerSeconds) }

    LaunchedEffect(tool) {
        val toolName = when (tool) {
            DrawingTool.PEN -> "Pen Tool"
            DrawingTool.PENCIL -> "Pencil Tool"
            DrawingTool.HIGHLIGHTER -> "Highlighter"
            DrawingTool.ERASER -> "Eraser"
            DrawingTool.LASER -> "Laser Pointer"
            DrawingTool.LINE -> "Line Shape"
            DrawingTool.ARROW -> "Arrow Shape"
            DrawingTool.RECTANGLE -> "Rectangle Shape"
            DrawingTool.CIRCLE -> "Circle Shape"
        }
        toolToastText = toolName
        toolToastVisible = true
    }

    LaunchedEffect(toolToastText, toolToastVisible) {
        if (toolToastVisible) {
            delay(1600)
            toolToastVisible = false
        }
    }

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

        // Top Navigation and Action Bar
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

            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Quick Duration Timer Cycle Button
                FilledTonalButton(
                    onClick = {
                        val nextSec = when (timerSec) {
                            0 -> 3
                            3 -> 5
                            5 -> 10
                            10 -> 30
                            30 -> 60
                            else -> 0
                        }
                        timerSec = nextSec
                        viewModel.setAnnotationTimer(nextSec)
                        toolToastText = if (nextSec == 0) "Duration: Never (Persistent)" else "Duration: ${nextSec}s Auto-Fade"
                        toolToastVisible = true
                    },
                    contentPadding = PaddingValues(horizontal = 10.dp, vertical = 0.dp),
                    modifier = Modifier.height(40.dp)
                ) {
                    Icon(
                        Icons.Default.Timer,
                        contentDescription = "Annotation Duration",
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(Modifier.width(4.dp))
                    Text(
                        text = if (timerSec == 0) "∞" else "${timerSec}s",
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold
                    )
                }

                // Quick Undo & Redo in Top Bar
                FilledTonalIconButton(
                    onClick = {
                        viewModel.undo()
                        toolToastText = "Undo"
                        toolToastVisible = true
                    },
                    enabled = canUndo
                ) {
                    Icon(Icons.AutoMirrored.Filled.Undo, contentDescription = "Undo")
                }

                FilledTonalIconButton(
                    onClick = {
                        viewModel.redo()
                        toolToastText = "Redo"
                        toolToastVisible = true
                    },
                    enabled = canRedo
                ) {
                    Icon(Icons.AutoMirrored.Filled.Redo, contentDescription = "Redo")
                }

                // Instant Clear All Button
                FilledTonalIconButton(
                    onClick = { viewModel.clearCanvas() },
                    colors = IconButtonDefaults.filledTonalIconButtonColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer,
                        contentColor = MaterialTheme.colorScheme.onErrorContainer
                    )
                ) {
                    Icon(Icons.Default.DeleteSweep, contentDescription = "Clear All Annotations")
                }

                FilledTonalIconButton(onClick = { showSaveDialog = true }) {
                    Icon(Icons.Default.Save, contentDescription = "Save Session")
                }
            }
        }

        // Active Tool Switch Popup (Top Center)
        AnimatedVisibility(
            visible = toolToastVisible && toolToastText != null,
            enter = fadeIn() + slideInVertically(initialOffsetY = { -it }),
            exit = fadeOut() + slideOutVertically(targetOffsetY = { -it }),
            modifier = Modifier
                .align(Alignment.TopCenter)
                .statusBarsPadding()
                .padding(top = 64.dp)
        ) {
            Surface(
                shape = RoundedCornerShape(24.dp),
                color = MaterialTheme.colorScheme.primaryContainer,
                tonalElevation = 6.dp,
                shadowElevation = 4.dp
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 18.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    val icon = if (toolToastText?.startsWith("Duration") == true) {
                        Icons.Default.Timer
                    } else if (toolToastText == "Undo") {
                        Icons.AutoMirrored.Filled.Undo
                    } else if (toolToastText == "Redo") {
                        Icons.AutoMirrored.Filled.Redo
                    } else {
                        when (tool) {
                            DrawingTool.PEN -> Icons.Default.Edit
                            DrawingTool.PENCIL -> Icons.Default.Create
                            DrawingTool.HIGHLIGHTER -> Icons.Default.Brush
                            DrawingTool.ERASER -> Icons.Default.CleaningServices
                            DrawingTool.LASER -> Icons.Default.Highlight
                            DrawingTool.LINE -> Icons.Default.HorizontalRule
                            DrawingTool.ARROW -> Icons.AutoMirrored.Filled.ArrowRightAlt
                            DrawingTool.RECTANGLE -> Icons.Default.CropSquare
                            DrawingTool.CIRCLE -> Icons.Default.RadioButtonUnchecked
                        }
                    }
                    val displayText = if (toolToastText?.startsWith("Duration") == true || toolToastText == "Undo" || toolToastText == "Redo") {
                        toolToastText ?: ""
                    } else {
                        "$toolToastText Active"
                    }
                    Icon(
                        icon,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onPrimaryContainer,
                        modifier = Modifier.size(18.dp)
                    )
                    Spacer(Modifier.width(8.dp))
                    Text(
                        text = displayText,
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onPrimaryContainer
                    )
                }
            }
        }

        // Floating Bottom Toolbar
        FloatingDrawingToolbar(
            currentTool = tool,
            currentColor = color,
            currentWidth = width,
            canUndo = canUndo,
            canRedo = canRedo,
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
