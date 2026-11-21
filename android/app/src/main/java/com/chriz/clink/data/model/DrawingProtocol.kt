package com.chriz.clink.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

val ClinkJson = Json {
    ignoreUnknownKeys = true
    isLenient = true
    encodeDefaults = true
}

@Serializable
data class ProtocolPoint(
    val x: Float,
    val y: Float,
    val pressure: Float = 1.0f,
    val timestamp: Long = 0L
)

@Serializable
data class DisplayGeometry(
    val width: Int = 1920,
    val height: Int = 1080,
    val aspect_ratio: Float = 1.777778f
)

@Serializable
data class HelloMsg(
    val type: String = "hello",
    val version: String = "1.0",
    val client_name: String = "clink-android",
    val device_id: String = "",
    val device_name: String = "",
    val token: String? = null
)

@Serializable
data class HelloAckMsg(
    val type: String = "hello_ack",
    val version: String = "1.0",
    val server_name: String = "clink-linux",
    val authenticated: Boolean = false,
    val pairing_required: Boolean = false,
    val display: DisplayGeometry = DisplayGeometry()
)

@Serializable
data class PairRequestMsg(
    val type: String = "pair_request",
    val device_id: String,
    val device_name: String,
    val pin: String
)

@Serializable
data class PairAckMsg(
    val type: String = "pair_ack",
    val success: Boolean,
    val token: String? = null,
    val message: String? = null
)

@Serializable
data class AuthMsg(
    val type: String = "auth",
    val token: String
)

@Serializable
data class AuthAckMsg(
    val type: String = "auth_ack",
    val success: Boolean,
    val message: String? = null
)

@Serializable
data class ModeChangeMsg(
    val type: String = "mode_change",
    val mode: String
)

@Serializable
data class StrokeStartMsg(
    val type: String = "stroke_start",
    val stroke_id: String,
    val tool: String,
    val color: String,
    val width: Float,
    val x: Float,
    val y: Float,
    val pressure: Float = 1.0f,
    val timestamp: Long = System.currentTimeMillis()
)

@Serializable
data class StrokeMoveMsg(
    val type: String = "stroke_move",
    val stroke_id: String,
    val points: List<ProtocolPoint>
)

@Serializable
data class StrokeEndMsg(
    val type: String = "stroke_end",
    val stroke_id: String,
    val x: Float,
    val y: Float,
    val pressure: Float = 0.0f,
    val timestamp: Long = System.currentTimeMillis()
)

@Serializable
data class CursorMoveMsg(
    val type: String = "cursor_move",
    val dx: Float = 0.0f,
    val dy: Float = 0.0f,
    val x: Float = 0.0f,
    val y: Float = 0.0f,
    val pressure: Float = 1.0f,
    val timestamp: Long = System.currentTimeMillis()
)

@Serializable
data class PointerDownMsg(
    val type: String = "pointer_down",
    val button: Int = 1,
    val x: Float = 0.0f,
    val y: Float = 0.0f,
    val pressure: Float = 1.0f,
    val timestamp: Long = System.currentTimeMillis()
)

@Serializable
data class PointerUpMsg(
    val type: String = "pointer_up",
    val button: Int = 1,
    val x: Float = 0.0f,
    val y: Float = 0.0f,
    val timestamp: Long = System.currentTimeMillis()
)

@Serializable
data class SimpleActionMsg(
    val type: String
)

@Serializable
data class LaserPointerMsg(
    val type: String = "laser_pointer",
    val x: Float,
    val y: Float,
    val active: Boolean
)

@Serializable
data class PingPongMsg(
    val type: String,
    val timestamp: Long
)
