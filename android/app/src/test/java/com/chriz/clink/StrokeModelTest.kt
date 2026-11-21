package com.chriz.clink

import com.chriz.clink.data.model.*
import org.junit.Assert.*
import org.junit.Test

class StrokeModelTest {

    @Test
    fun testDrawingToolProperties() {
        assertEquals("pen", DrawingTool.PEN.id)
        assertEquals("pencil", DrawingTool.PENCIL.id)
        assertEquals("highlighter", DrawingTool.HIGHLIGHTER.id)
        assertEquals("eraser", DrawingTool.ERASER.id)
        assertEquals("laser", DrawingTool.LASER.id)
        assertEquals("line", DrawingTool.LINE.id)
        assertEquals("arrow", DrawingTool.ARROW.id)
        assertEquals("rectangle", DrawingTool.RECTANGLE.id)
        assertEquals("circle", DrawingTool.CIRCLE.id)
    }

    @Test
    fun testCanvasStrokeCreation() {
        val stroke = CanvasStroke(
            id = "stroke_1",
            tool = DrawingTool.PEN,
            color = 0xFFFF3366,
            width = 4.0f,
            points = mutableListOf(CanvasPoint(0.1f, 0.2f, 0.8f))
        )
        assertEquals("stroke_1", stroke.id)
        assertEquals(1, stroke.points.size)
        assertFalse(stroke.isCompleted)

        stroke.points.add(CanvasPoint(0.2f, 0.3f, 0.9f))
        stroke.isCompleted = true
        assertEquals(2, stroke.points.size)
        assertTrue(stroke.isCompleted)
    }
}
