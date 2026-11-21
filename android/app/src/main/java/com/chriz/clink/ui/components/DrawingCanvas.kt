package com.chriz.clink.ui.components

import android.view.MotionEvent
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke as ComposeStroke
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import com.chriz.clink.data.model.CanvasPoint
import com.chriz.clink.data.model.CanvasStroke
import com.chriz.clink.data.model.ClinkMode
import com.chriz.clink.data.model.DrawingTool
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.sin

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun DrawingCanvas(
    mode: ClinkMode,
    currentTool: DrawingTool,
    currentColor: Color,
    currentWidth: Float,
    strokes: List<CanvasStroke>,
    activeStroke: CanvasStroke?,
    laserPosition: Offset?,
    palmRejectionEnabled: Boolean,
    onStrokeStart: (Float, Float, Float) -> Unit,
    onStrokeMove: (List<CanvasPoint>) -> Unit,
    onStrokeEnd: (Float, Float, Float) -> Unit,
    onCursorMove: (Float, Float, Float, Float, Float) -> Unit,
    onPointerDown: (Int, Float, Float, Float) -> Unit,
    onPointerUp: (Int, Float, Float) -> Unit,
    modifier: Modifier = Modifier
) {
    var canvasSize by remember { mutableStateOf(Size.Zero) }
    var lastTouchX by remember { mutableStateOf(0f) }
    var lastTouchY by remember { mutableStateOf(0f) }

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
    ) {
        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .pointerInteropFilter { motionEvent ->
                    val w = canvasSize.width.coerceAtLeast(1f)
                    val h = canvasSize.height.coerceAtLeast(1f)
                    val normX = (motionEvent.x / w).coerceIn(0f, 1f)
                    val normY = (motionEvent.y / h).coerceIn(0f, 1f)
                    val pressure = motionEvent.pressure.coerceIn(0.1f, 1.0f)
                    val toolType = motionEvent.getToolType(0)

                    // Palm rejection: ignore finger if palm rejection enabled and stylus is preferred
                    if (palmRejectionEnabled && toolType == MotionEvent.TOOL_TYPE_FINGER && motionEvent.size > 0.5f) {
                        return@pointerInteropFilter true
                    }

                    if (mode == ClinkMode.ABSOLUTE_ANNOTATION) {
                        when (motionEvent.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                onStrokeStart(normX, normY, pressure)
                            }
                            MotionEvent.ACTION_MOVE -> {
                                val batched = mutableListOf<CanvasPoint>()
                                for (i in 0 until motionEvent.historySize) {
                                    val hX = (motionEvent.getHistoricalX(i) / w).coerceIn(0f, 1f)
                                    val hY = (motionEvent.getHistoricalY(i) / h).coerceIn(0f, 1f)
                                    val hP = motionEvent.getHistoricalPressure(i).coerceIn(0.1f, 1.0f)
                                    batched.add(CanvasPoint(hX, hY, hP, motionEvent.getHistoricalEventTime(i)))
                                }
                                batched.add(CanvasPoint(normX, normY, pressure, motionEvent.eventTime))
                                onStrokeMove(batched)
                            }
                            MotionEvent.ACTION_UP -> {
                                onStrokeEnd(normX, normY, pressure)
                            }
                            MotionEvent.ACTION_CANCEL -> {
                                onStrokeEnd(normX, normY, 0f)
                            }
                        }
                    } else {
                        // RELATIVE_TABLET MODE
                        when (motionEvent.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                lastTouchX = motionEvent.x
                                lastTouchY = motionEvent.y
                                onPointerDown(1, normX, normY, pressure)
                            }
                            MotionEvent.ACTION_MOVE -> {
                                val dx = (motionEvent.x - lastTouchX) / w
                                val dy = (motionEvent.y - lastTouchY) / h
                                lastTouchX = motionEvent.x
                                lastTouchY = motionEvent.y
                                onCursorMove(dx, dy, normX, normY, pressure)
                            }
                            MotionEvent.ACTION_UP -> {
                                onPointerUp(1, normX, normY)
                            }
                            MotionEvent.ACTION_CANCEL -> {
                                onPointerUp(1, normX, normY)
                            }
                        }
                    }
                    true
                }
        ) {
            canvasSize = size

            if (mode == ClinkMode.RELATIVE_TABLET) {
                // Draw sleek grid pattern for tablet mode
                drawTabletGrid(size)
                return@Canvas
            }

            // Draw completed strokes
            for (stroke in strokes) {
                drawStroke(stroke, size)
            }

            // Draw active in-progress stroke
            activeStroke?.let {
                drawStroke(it, size)
            }

            // Draw laser pointer
            laserPosition?.let { pos ->
                drawCircle(
                    brush = Brush.radialGradient(
                        colors = listOf(Color(0xFFFF0033), Color(0x88FF0033), Color.Transparent),
                        center = pos,
                        radius = 45f
                    ),
                    radius = 45f,
                    center = pos
                )
                drawCircle(
                    color = Color.White,
                    radius = 8f,
                    center = pos
                )
            }
        }

        if (mode == ClinkMode.RELATIVE_TABLET) {
            Text(
                text = "Graphics Tablet Active\nStylus acts as desktop input controller",
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.4f),
                fontWeight = FontWeight.Medium,
                fontSize = 16.sp,
                modifier = Modifier.align(Alignment.Center)
            )
        }
    }
}

private fun DrawScope.drawTabletGrid(size: Size) {
    val step = 60f
    var x = step
    while (x < size.width) {
        drawLine(
            color = Color(0x15FFFFFF),
            start = Offset(x, 0f),
            end = Offset(x, size.height),
            strokeWidth = 1f
        )
        x += step
    }
    var y = step
    while (y < size.height) {
        drawLine(
            color = Color(0x15FFFFFF),
            start = Offset(0f, y),
            end = Offset(size.width, y),
            strokeWidth = 1f
        )
        y += step
    }
}

private fun DrawScope.drawStroke(stroke: CanvasStroke, size: Size) {
    if (stroke.points.isEmpty()) return
    val pts = stroke.points
    val strokeColor = stroke.toComposeColor()
    val width = stroke.width

    when (stroke.tool) {
        DrawingTool.PEN, DrawingTool.PENCIL, DrawingTool.HIGHLIGHTER, DrawingTool.ERASER -> {
            val alpha = when (stroke.tool) {
                DrawingTool.HIGHLIGHTER -> 0.35f
                DrawingTool.PENCIL -> 0.70f
                DrawingTool.ERASER -> 1.0f
                else -> 1.0f
            }
            val color = if (stroke.tool == DrawingTool.ERASER) Color(0xFF1E1E2A) else strokeColor

            if (pts.size == 1) {
                val p = pts[0]
                drawCircle(
                    color = color.copy(alpha = alpha),
                    radius = (width * p.pressure) / 2f,
                    center = Offset(p.x * size.width, p.y * size.height)
                )
            } else {
                val path = Path()
                path.moveTo(pts[0].x * size.width, pts[0].y * size.height)
                for (i in 1 until pts.size) {
                    val prev = pts[i - 1]
                    val curr = pts[i]
                    val midX = (prev.x + curr.x) / 2f * size.width
                    val midY = (prev.y + curr.y) / 2f * size.height
                    path.quadraticTo(prev.x * size.width, prev.y * size.height, midX, midY)
                }
                val last = pts.last()
                path.lineTo(last.x * size.width, last.y * size.height)

                drawPath(
                    path = path,
                    color = color.copy(alpha = alpha),
                    style = ComposeStroke(
                        width = width * (pts.lastOrNull()?.pressure ?: 1f).coerceAtLeast(0.5f),
                        cap = StrokeCap.Round,
                        join = StrokeJoin.Round
                    )
                )
            }
        }
        DrawingTool.LINE -> {
            if (pts.size >= 2) {
                val p1 = pts.first()
                val p2 = pts.last()
                drawLine(
                    color = strokeColor,
                    start = Offset(p1.x * size.width, p1.y * size.height),
                    end = Offset(p2.x * size.width, p2.y * size.height),
                    strokeWidth = width,
                    cap = StrokeCap.Round
                )
            }
        }
        DrawingTool.ARROW -> {
            if (pts.size >= 2) {
                val p1 = pts.first()
                val p2 = pts.last()
                val start = Offset(p1.x * size.width, p1.y * size.height)
                val end = Offset(p2.x * size.width, p2.y * size.height)
                drawLine(color = strokeColor, start = start, end = end, strokeWidth = width, cap = StrokeCap.Round)

                val angle = atan2((end.y - start.y).toDouble(), (end.x - start.x).toDouble())
                val arrowLen = (width * 3.5f).coerceAtLeast(24f)
                val arrowAngle = Math.PI / 6.0

                val left = Offset(
                    (end.x - arrowLen * cos(angle - arrowAngle)).toFloat(),
                    (end.y - arrowLen * sin(angle - arrowAngle)).toFloat()
                )
                val right = Offset(
                    (end.x - arrowLen * cos(angle + arrowAngle)).toFloat(),
                    (end.y - arrowLen * sin(angle + arrowAngle)).toFloat()
                )
                drawLine(color = strokeColor, start = end, end = left, strokeWidth = width, cap = StrokeCap.Round)
                drawLine(color = strokeColor, start = end, end = right, strokeWidth = width, cap = StrokeCap.Round)
            }
        }
        DrawingTool.RECTANGLE -> {
            if (pts.size >= 2) {
                val p1 = pts.first()
                val p2 = pts.last()
                val x1 = p1.x * size.width
                val y1 = p1.y * size.height
                val x2 = p2.x * size.width
                val y2 = p2.y * size.height
                drawRect(
                    color = strokeColor,
                    topLeft = Offset(minOf(x1, x2), minOf(y1, y2)),
                    size = Size(kotlin.math.abs(x2 - x1), kotlin.math.abs(y2 - y1)),
                    style = ComposeStroke(width = width)
                )
            }
        }
        DrawingTool.CIRCLE -> {
            if (pts.size >= 2) {
                val p1 = pts.first()
                val p2 = pts.last()
                val x1 = p1.x * size.width
                val y1 = p1.y * size.height
                val x2 = p2.x * size.width
                val y2 = p2.y * size.height
                drawOval(
                    color = strokeColor,
                    topLeft = Offset(minOf(x1, x2), minOf(y1, y2)),
                    size = Size(kotlin.math.abs(x2 - x1), kotlin.math.abs(y2 - y1)),
                    style = ComposeStroke(width = width)
                )
            }
        }
        DrawingTool.LASER -> {
            // Laser is handled dynamically
        }
    }
}
