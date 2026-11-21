package com.chriz.clink

import org.junit.Assert.*
import org.junit.Test

class CoordinateMappingTest {

    @Test
    fun testNormalizedCoordinatesClamping() {
        val screenW = 1080f
        val screenH = 2400f

        val rawX = 540f
        val rawY = 1200f
        val normX = (rawX / screenW).coerceIn(0f, 1f)
        val normY = (rawY / screenH).coerceIn(0f, 1f)

        assertEquals(0.5f, normX, 0.001f)
        assertEquals(0.5f, normY, 0.001f)

        // Beyond bounds clamping
        val outX = -50f
        val outY = 3000f
        val clampedX = (outX / screenW).coerceIn(0f, 1f)
        val clampedY = (outY / screenH).coerceIn(0f, 1f)
        assertEquals(0f, clampedX, 0.001f)
        assertEquals(1f, clampedY, 0.001f)
    }

    @Test
    fun testDesktopTransformation() {
        val desktopW = 1920
        val desktopH = 1080

        val normX = 0.25f
        val normY = 0.75f

        val deskX = (normX * desktopW).toInt()
        val deskY = (normY * desktopH).toInt()

        assertEquals(480, deskX)
        assertEquals(810, deskY)
    }

    @Test
    fun testRelativeDeltas() {
        val lastX = 500f
        val currX = 525f
        val screenW = 1000f

        val dx = (currX - lastX) / screenW
        assertEquals(0.025f, dx, 0.0001f)
    }
}
