package com.chriz.clink.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Redo
import androidx.compose.material.icons.automirrored.filled.Undo
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import com.chriz.clink.data.model.DrawingTool

@Composable
fun FloatingDrawingToolbar(
    currentTool: DrawingTool,
    currentColor: Color,
    currentWidth: Float,
    canUndo: Boolean,
    canRedo: Boolean,
    onToolSelected: (DrawingTool) -> Unit,
    onColorSelected: (Color) -> Unit,
    onWidthChanged: (Float) -> Unit,
    onUndo: () -> Unit,
    onRedo: () -> Unit,
    onClear: () -> Unit,
    modifier: Modifier = Modifier
) {
    var showColorPalette by remember { mutableStateOf(false) }
    var showWidthSlider by remember { mutableStateOf(false) }

    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Dropdown popups
        AnimatedVisibility(visible = showColorPalette) {
            ColorPickerRow(
                selectedColor = currentColor,
                onColorSelected = {
                    onColorSelected(it)
                    showColorPalette = false
                },
                modifier = Modifier.padding(bottom = 8.dp)
            )
        }

        AnimatedVisibility(visible = showWidthSlider) {
            Surface(
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surface.copy(alpha = 0.95f),
                tonalElevation = 6.dp,
                modifier = Modifier.padding(bottom = 8.dp)
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Width", style = MaterialTheme.typography.bodyMedium)
                    Spacer(Modifier.width(8.dp))
                    Slider(
                        value = currentWidth,
                        onValueChange = onWidthChanged,
                        valueRange = 1f..30f,
                        modifier = Modifier.width(160.dp)
                    )
                }
            }
        }

        // Main Toolbar Surface
        Surface(
            shape = RoundedCornerShape(32.dp),
            color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
            tonalElevation = 8.dp,
            shadowElevation = 8.dp
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                ToolIconButton(
                    icon = Icons.Default.Edit,
                    isSelected = currentTool == DrawingTool.PEN,
                    onClick = { onToolSelected(DrawingTool.PEN) }
                )
                ToolIconButton(
                    icon = Icons.Default.Create,
                    isSelected = currentTool == DrawingTool.PENCIL,
                    onClick = { onToolSelected(DrawingTool.PENCIL) }
                )
                ToolIconButton(
                    icon = Icons.Default.Brush,
                    isSelected = currentTool == DrawingTool.HIGHLIGHTER,
                    onClick = { onToolSelected(DrawingTool.HIGHLIGHTER) }
                )
                ToolIconButton(
                    icon = Icons.Default.AutoFixNormal,
                    isSelected = currentTool == DrawingTool.ERASER,
                    onClick = { onToolSelected(DrawingTool.ERASER) }
                )
                ToolIconButton(
                    icon = Icons.Default.Highlight,
                    isSelected = currentTool == DrawingTool.LASER,
                    onClick = { onToolSelected(DrawingTool.LASER) }
                )

                // Shapes Dropdown or Direct tools
                ToolIconButton(
                    icon = Icons.Default.HorizontalRule,
                    isSelected = currentTool == DrawingTool.LINE,
                    onClick = { onToolSelected(DrawingTool.LINE) }
                )
                ToolIconButton(
                    icon = Icons.Default.ArrowOutward,
                    isSelected = currentTool == DrawingTool.ARROW,
                    onClick = { onToolSelected(DrawingTool.ARROW) }
                )
                ToolIconButton(
                    icon = Icons.Default.CropSquare,
                    isSelected = currentTool == DrawingTool.RECTANGLE,
                    onClick = { onToolSelected(DrawingTool.RECTANGLE) }
                )
                ToolIconButton(
                    icon = Icons.Default.Circle,
                    isSelected = currentTool == DrawingTool.CIRCLE,
                    onClick = { onToolSelected(DrawingTool.CIRCLE) }
                )

                VerticalDivider(modifier = Modifier.height(24.dp).padding(horizontal = 4.dp))

                // Color Button
                IconButton(
                    onClick = { showColorPalette = !showColorPalette; showWidthSlider = false }
                ) {
                    Box(
                        modifier = Modifier
                            .size(20.dp)
                            .background(currentColor, RoundedCornerShape(10.dp))
                    )
                }

                // Width Button
                IconButton(
                    onClick = { showWidthSlider = !showWidthSlider; showColorPalette = false }
                ) {
                    Icon(Icons.Default.LineWeight, contentDescription = "Stroke Width")
                }

                VerticalDivider(modifier = Modifier.height(24.dp).padding(horizontal = 4.dp))

                // Undo / Redo / Clear
                IconButton(onClick = onUndo, enabled = canUndo) {
                    Icon(Icons.AutoMirrored.Filled.Undo, contentDescription = "Undo")
                }
                IconButton(onClick = onRedo, enabled = canRedo) {
                    Icon(Icons.AutoMirrored.Filled.Redo, contentDescription = "Redo")
                }
                IconButton(onClick = onClear) {
                    Icon(Icons.Default.DeleteOutline, contentDescription = "Clear Canvas")
                }
            }
        }
    }
}

@Composable
private fun ToolIconButton(
    icon: ImageVector,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    FilledIconToggleButton(
        checked = isSelected,
        onCheckedChange = { onClick() },
        colors = IconButtonDefaults.filledIconToggleButtonColors(
            containerColor = Color.Transparent,
            checkedContainerColor = MaterialTheme.colorScheme.primaryContainer,
            contentColor = MaterialTheme.colorScheme.onSurface,
            checkedContentColor = MaterialTheme.colorScheme.onPrimaryContainer
        ),
        modifier = Modifier.size(38.dp)
    ) {
        Icon(icon, contentDescription = null, modifier = Modifier.size(20.dp))
    }
}
