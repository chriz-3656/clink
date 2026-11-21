package com.chriz.clink

import com.chriz.clink.data.model.*
import kotlinx.serialization.encodeToString
import org.junit.Assert.*
import org.junit.Test

class ProtocolSerializationTest {

    @Test
    fun testHelloMessageSerialization() {
        val msg = HelloMsg(
            version = "1.0",
            client_name = "clink-android",
            device_id = "test_dev_123",
            device_name = "Redmi A5"
        )
        val json = ClinkJson.encodeToString(msg)
        assertTrue(json.contains("\"type\":\"hello\""))
        assertTrue(json.contains("\"device_id\":\"test_dev_123\""))

        val decoded = ClinkJson.decodeFromString<HelloMsg>(json)
        assertEquals("hello", decoded.type)
        assertEquals("test_dev_123", decoded.device_id)
        assertEquals("Redmi A5", decoded.device_name)
    }

    @Test
    fun testHelloAckDeserialization() {
        val json = """{"type":"hello_ack","version":"1.0","server_name":"clink-linux","authenticated":true,"pairing_required":false,"display":{"width":2560,"height":1440,"aspect_ratio":1.7778}}"""
        val ack = ClinkJson.decodeFromString<HelloAckMsg>(json)
        assertEquals("hello_ack", ack.type)
        assertTrue(ack.authenticated)
        assertFalse(ack.pairing_required)
        assertEquals(2560, ack.display.width)
        assertEquals(1440, ack.display.height)
    }

    @Test
    fun testStrokeMessagesSerialization() {
        val start = StrokeStartMsg(
            stroke_id = "s_1001",
            tool = "pen",
            color = "#FF3366",
            width = 4.5f,
            x = 0.25f,
            y = 0.5f,
            pressure = 0.75f
        )
        val startJson = ClinkJson.encodeToString(start)
        assertTrue(startJson.contains("\"tool\":\"pen\""))

        val move = StrokeMoveMsg(
            stroke_id = "s_1001",
            points = listOf(
                ProtocolPoint(0.26f, 0.51f, 0.76f, 100),
                ProtocolPoint(0.27f, 0.52f, 0.78f, 120)
            )
        )
        val moveJson = ClinkJson.encodeToString(move)
        val decodedMove = ClinkJson.decodeFromString<StrokeMoveMsg>(moveJson)
        assertEquals("s_1001", decodedMove.stroke_id)
        assertEquals(2, decodedMove.points.size)

        val end = StrokeEndMsg(stroke_id = "s_1001", x = 0.28f, y = 0.53f, pressure = 0f)
        val endJson = ClinkJson.encodeToString(end)
        assertTrue(endJson.contains("\"type\":\"stroke_end\""))
    }

    @Test
    fun testTabletPointerMessages() {
        val cursor = CursorMoveMsg(dx = 0.02f, dy = -0.01f, x = 0.5f, y = 0.5f, pressure = 0.8f)
        val cursorJson = ClinkJson.encodeToString(cursor)
        val decoded = ClinkJson.decodeFromString<CursorMoveMsg>(cursorJson)
        assertEquals(0.02f, decoded.dx, 0.0001f)
        assertEquals(-0.01f, decoded.dy, 0.0001f)

        val pDown = PointerDownMsg(button = 1, x = 0.5f, y = 0.5f, pressure = 0.9f)
        val pDownJson = ClinkJson.encodeToString(pDown)
        assertTrue(pDownJson.contains("\"button\":1"))

        val pUp = PointerUpMsg(button = 1, x = 0.5f, y = 0.5f)
        val pUpJson = ClinkJson.encodeToString(pUp)
        assertTrue(pUpJson.contains("\"type\":\"pointer_up\""))
    }

    @Test
    fun testLaserPointerSerialization() {
        val laser = LaserPointerMsg(x = 0.45f, y = 0.65f, active = true)
        val json = ClinkJson.encodeToString(laser)
        val decoded = ClinkJson.decodeFromString<LaserPointerMsg>(json)
        assertTrue(decoded.active)
        assertEquals(0.45f, decoded.x, 0.0001f)
    }
}
