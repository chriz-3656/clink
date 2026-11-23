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
    sensitivity: Float = 1.0f,
    dragLockEnabled: Boolean = false,
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
    var touchDownX by remember { mutableStateOf(0f) }
    var touchDownY by remember { mutableStateOf(0f) }
    var touchDownTime by remember { mutableStateOf(0L) }
    var isMovedBeyondTap by remember { mutableStateOf(false) }
    var isTwoFingerTap by remember { mutableStateOf(false) }

    var localActivePoints by remember { mutableStateOf<List<CanvasPoint>>(emptyList()) }
    var localActiveTool by remember { mutableStateOf(currentTool) }
    var localActiveColor by remember { mutableStateOf(currentColor) }
    var localActiveWidth by remember { mutableStateOf(currentWidth) }

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

                    // Palm rejection: only reject if device hardware explicitly flags a palm touch
                    if (palmRejectionEnabled && toolType == 4 /* MotionEvent.TOOL_TYPE_PALM */) {
                        return@pointerInteropFilter true
                    }

                    if (mode == ClinkMode.ABSOLUTE_ANNOTATION) {
                        when (motionEvent.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                localActiveTool = currentTool
                                localActiveColor = currentColor
                                localActiveWidth = currentWidth
                                localActivePoints = listOf(CanvasPoint(normX, normY, pressure))
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
                                localActivePoints = localActivePoints + batched
                                onStrokeMove(batched)
                            }
                            MotionEvent.ACTION_UP -> {
                                localActivePoints = emptyList()
                                onStrokeEnd(normX, normY, pressure)
                            }
                            MotionEvent.ACTION_CANCEL -> {
                                localActivePoints = emptyList()
                                onStrokeEnd(normX, normY, 0f)
                            }
                        }
                    } else {
                        // RELATIVE_TABLET MODE: Smooth relative desktop controller
                        when (motionEvent.actionMasked) {
                            MotionEvent.ACTION_DOWN -> {
                                lastTouchX = motionEvent.x
                                lastTouchY = motionEvent.y
                                touchDownX = motionEvent.x
                                touchDownY = motionEvent.y
                                touchDownTime = motionEvent.eventTime
                                isMovedBeyondTap = false
                                isTwoFingerTap = false
                                if (toolType == MotionEvent.TOOL_TYPE_STYLUS || dragLockEnabled) {
                                    onPointerDown(1, normX, normY, pressure)
                                }
                            }
                            MotionEvent.ACTION_POINTER_DOWN -> {
                                if (motionEvent.pointerCount == 2) {
                                    isTwoFingerTap = true
                                }
                            }
                            MotionEvent.ACTION_MOVE -> {
                                val rawDx = motionEvent.x - lastTouchX
                                val rawDy = motionEvent.y - lastTouchY
                                lastTouchX = motionEvent.x
                                lastTouchY = motionEvent.y

                                val dist = kotlin.math.hypot(
                                    (motionEvent.x - touchDownX).toDouble(),
                                    (motionEvent.y - touchDownY).toDouble()
                                ).toFloat()
                                if (dist > 10f) {
                                    isMovedBeyondTap = true
                                }

                                val dx = rawDx * sensitivity
                                val dy = rawDy * sensitivity
                                onCursorMove(dx, dy, normX, normY, pressure)
                            }
                            MotionEvent.ACTION_POINTER_UP -> {
                                if (isTwoFingerTap && !isMovedBeyondTap) {
                                    // Two-finger tap -> Right Click (button 3)
                                    onPointerDown(3, normX, normY, pressure)
                                    onPointerUp(3, normX, normY)
                                }
                            }
                            MotionEvent.ACTION_UP -> {
                                val duration = motionEvent.eventTime - touchDownTime
                                if (toolType == MotionEvent.TOOL_TYPE_STYLUS || dragLockEnabled) {
                                    onPointerUp(1, normX, normY)
                                } else if (!isMovedBeyondTap && !isTwoFingerTap && duration < 320L) {
                                    // Single tap -> Left Click (button 1)
                                    onPointerDown(1, normX, normY, pressure)
                                    onPointerUp(1, normX, normY)
                                }
                            }
                            MotionEvent.ACTION_CANCEL -> {
                                if (toolType == MotionEvent.TOOL_TYPE_STYLUS || dragLockEnabled) {
                                    onPointerUp(1, normX, normY)
                                }
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

            // Draw active in-progress stroke with 0ms ink response
            if (localActivePoints.isNotEmpty()) {
                val liveStroke = CanvasStroke(
                    id = "live",
                    tool = localActiveTool,
                    color = localActiveColor.value.toLong(),
                    width = localActiveWidth,
                    points = localActivePoints.toMutableList(),
                    isCompleted = false
                )
                drawStroke(liveStroke, size)
            } else {
                activeStroke?.let {
                    drawStroke(it, size)
                }
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
                text = "Graphics Tablet Active\nStylus or touch operates as desktop cursor controller\nTap to click  •  Two-finger tap for right-click",
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.45f),
                fontWeight = FontWeight.Medium,
                fontSize = 15.sp,
                textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                lineHeight = 22.sp,
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
                val dx = end.x - start.x
                val dy = end.y - start.y
                val dist = kotlin.math.hypot(dx, dy)
                if (dist > 3f) {
                    val angle = atan2(dy.toDouble(), dx.toDouble())
                    val headLen = kotlin.math.max(14f, kotlin.math.min(width * 5.5f, dist * 0.45f))
                    val headAngle = Math.PI / 6.0

                    // Draw main arrow shaft
                    drawLine(color = strokeColor, start = start, end = end, strokeWidth = width, cap = StrokeCap.Round)

                    val pLeft = Offset(
                        (end.x - headLen * cos(angle - headAngle)).toFloat(),
                        (end.y - headLen * sin(angle - headAngle)).toFloat()
                    )
                    val pRight = Offset(
                        (end.x - headLen * cos(angle + headAngle)).toFloat(),
                        (end.y - headLen * sin(angle + headAngle)).toFloat()
                    )

                    // Draw solid filled arrowhead polygon for clear, crisp presentation
                    val headPath = Path().apply {
                        moveTo(end.x, end.y)
                        lineTo(pLeft.x, pLeft.y)
                        lineTo(pRight.x, pRight.y)
                        close()
                    }
                    drawPath(path = headPath, color = strokeColor)
                    drawLine(color = strokeColor, start = end, end = pLeft, strokeWidth = width, cap = StrokeCap.Round)
                    drawLine(color = strokeColor, start = end, end = pRight, strokeWidth = width, cap = StrokeCap.Round)
                }
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
