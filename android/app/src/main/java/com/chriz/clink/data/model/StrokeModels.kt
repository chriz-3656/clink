package com.chriz.clink.data.model

import androidx.compose.ui.graphics.Color

enum class DrawingTool(val id: String, val displayName: String) {
    PEN("pen", "Pen"),
    PENCIL("pencil", "Pencil"),
    HIGHLIGHTER("highlighter", "Highlighter"),
    ERASER("eraser", "Eraser"),
    LASER("laser", "Laser Pointer"),
    LINE("line", "Line"),
    ARROW("arrow", "Arrow"),
    RECTANGLE("rectangle", "Rectangle"),
    CIRCLE("circle", "Circle")
}

enum class ClinkMode(val id: String, val displayName: String) {
    ABSOLUTE_ANNOTATION("absolute_annotation", "Absolute Annotation"),
    RELATIVE_TABLET("relative_tablet", "Graphics Tablet")
}

data class CanvasPoint(
    val x: Float,
    val y: Float,
    val pressure: Float = 1.0f,
    val timestamp: Long = System.currentTimeMillis()
)

data class CanvasStroke(
    val id: String,
    val tool: DrawingTool,
    val color: Long,
    val width: Float,
    val points: MutableList<CanvasPoint> = mutableListOf(),
    var isCompleted: Boolean = false
) {
    fun toComposeColor(): Color = Color(color)
}
