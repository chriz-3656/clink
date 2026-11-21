package com.chriz.clink.data.network

import android.os.Handler
import android.os.Looper
import android.util.Log
import com.chriz.clink.data.model.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import okhttp3.*
import java.util.concurrent.TimeUnit

enum class ConnectionStatus {
    DISCONNECTED,
    CONNECTING,
    CONNECTED,
    PAIRING_REQUIRED,
    AUTHENTICATED,
    ERROR
}

class WebSocketClient(
    private val onMessageReceived: (String) -> Unit = {}
) {
    companion object {
        private const val TAG = "ClinkWebSocket"
    }

    var onTokenReceived: (String) -> Unit = {}
    var onPairingFailed: (String) -> Unit = {}

    private val okHttpClient = OkHttpClient.Builder()
        .readTimeout(0, TimeUnit.MILLISECONDS)
        .connectTimeout(5, TimeUnit.SECONDS)
        .pingInterval(10, TimeUnit.SECONDS)
        .build()

    private var webSocket: WebSocket? = null
    private val mainHandler = Handler(Looper.getMainLooper())

    private val _status = MutableStateFlow(ConnectionStatus.DISCONNECTED)
    val status: StateFlow<ConnectionStatus> = _status.asStateFlow()

    private val _latencyMs = MutableStateFlow<Long>(0L)
    val latencyMs: StateFlow<Long> = _latencyMs.asStateFlow()

    private val _displayInfo = MutableStateFlow(DisplayGeometry())
    val displayInfo: StateFlow<DisplayGeometry> = _displayInfo.asStateFlow()

    private var currentHost = "127.0.0.1"
    private var currentPort = 8765
    private var authToken = ""
    private var currentDeviceId = "android_device"
    private var pingStartTime = 0L
    private var pendingPin: String? = null
    private var fallbackUsbHost: String? = null
    private var isTryingFallback = false

    fun setPendingPin(pin: String?) {
        pendingPin = pin
    }

    fun connectUsb(port: Int, token: String = "", deviceId: String = "android_device", fallbackHost: String? = null) {
        fallbackUsbHost = fallbackHost
        isTryingFallback = false
        connect("127.0.0.1", port, token, deviceId)
    }

    fun connect(host: String, port: Int, token: String = "", deviceId: String = "android_device") {
        currentHost = host
        currentPort = port
        authToken = token
        currentDeviceId = deviceId

        disconnect()
        _status.value = ConnectionStatus.CONNECTING

        val url = "ws://$host:$port"
        val request = Request.Builder().url(url).build()

        webSocket = okHttpClient.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(ws: WebSocket, response: Response) {
                Log.d(TAG, "WebSocket connected to $url")
                _status.value = ConnectionStatus.CONNECTED
                sendHello()
                startPingLoop()
                if (pendingPin != null) {
                    sendPairRequest(pendingPin!!)
                }
            }

            override fun onMessage(ws: WebSocket, text: String) {
                handleIncomingMessage(text)
                onMessageReceived(text)
            }

            override fun onClosing(ws: WebSocket, code: Int, reason: String) {
                Log.d(TAG, "WebSocket closing: $code / $reason")
                _status.value = ConnectionStatus.DISCONNECTED
            }

            override fun onClosed(ws: WebSocket, code: Int, reason: String) {
                Log.d(TAG, "WebSocket closed: $code / $reason")
                _status.value = ConnectionStatus.DISCONNECTED
            }

            override fun onFailure(ws: WebSocket, t: Throwable, response: Response?) {
                Log.e(TAG, "WebSocket failure: ${t.message}")
                if (!isTryingFallback && fallbackUsbHost != null && fallbackUsbHost != "127.0.0.1") {
                    Log.i(TAG, "Attempting USB Tethering fallback to gateway: $fallbackUsbHost")
                    isTryingFallback = true
                    connect(fallbackUsbHost!!, currentPort, authToken, currentDeviceId)
                    return
                }
                _status.value = ConnectionStatus.ERROR
            }
        })
    }

    fun disconnect() {
        try {
            webSocket?.close(1000, "User disconnected")
        } catch (e: Exception) {
            Log.e(TAG, "Error closing websocket", e)
        }
        webSocket = null
        _status.value = ConnectionStatus.DISCONNECTED
    }

    private fun handleIncomingMessage(text: String) {
        try {
            val jsonTree = ClinkJson.parseToJsonElement(text) as? JsonObject ?: return
            val type = jsonTree["type"]?.jsonPrimitive?.content ?: return

            when (type) {
                "hello_ack" -> {
                    val ack = ClinkJson.decodeFromString<HelloAckMsg>(text)
                    _displayInfo.value = ack.display
                    if (ack.authenticated) {
                        _status.value = ConnectionStatus.AUTHENTICATED
                        pendingPin = null
                    } else if (ack.pairing_required) {
                        if (pendingPin != null) {
                            val pinToSend = pendingPin!!
                            pendingPin = null
                            sendPairRequest(pinToSend)
                        } else {
                            _status.value = ConnectionStatus.PAIRING_REQUIRED
                        }
                    } else {
                        if (authToken.isNotBlank()) {
                            sendAuth(authToken)
                        } else if (pendingPin != null) {
                            val pinToSend = pendingPin!!
                            pendingPin = null
                            sendPairRequest(pinToSend)
                        } else {
                            _status.value = ConnectionStatus.PAIRING_REQUIRED
                        }
                    }
                }
                "pair_ack" -> {
                    val ack = ClinkJson.decodeFromString<PairAckMsg>(text)
                    pendingPin = null
                    if (ack.success && ack.token != null) {
                        authToken = ack.token
                        onTokenReceived(ack.token)
                        _status.value = ConnectionStatus.AUTHENTICATED
                        sendAuth(ack.token)
                    } else {
                        onPairingFailed(ack.message ?: "Invalid PIN")
                        _status.value = ConnectionStatus.PAIRING_REQUIRED
                    }
                }
                "auth_ack" -> {
                    val ack = ClinkJson.decodeFromString<AuthAckMsg>(text)
                    if (ack.success) {
                        _status.value = ConnectionStatus.AUTHENTICATED
                    } else {
                        _status.value = ConnectionStatus.PAIRING_REQUIRED
                    }
                }
                "pong" -> {
                    val pong = ClinkJson.decodeFromString<PingPongMsg>(text)
                    val rtt = System.currentTimeMillis() - pong.timestamp
                    if (rtt >= 0) {
                        _latencyMs.value = rtt
                    }
                }
            }
        } catch (e: Exception) {
            Log.e(TAG, "Error parsing incoming message", e)
        }
    }

    private fun sendHello() {
        val model = android.os.Build.MODEL ?: "Device"
        val manufacturer = android.os.Build.MANUFACTURER ?: "Android"
        val msg = HelloMsg(
            device_id = currentDeviceId,
            device_name = "$manufacturer $model".trim(),
            token = authToken.ifBlank { null }
        )
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendPairRequest(pin: String) {
        val model = android.os.Build.MODEL ?: "Device"
        val manufacturer = android.os.Build.MANUFACTURER ?: "Android"
        val msg = PairRequestMsg(
            device_id = currentDeviceId,
            device_name = "$manufacturer $model".trim(),
            pin = pin
        )
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendAuth(token: String) {
        val msg = AuthMsg(token = token)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendMode(mode: ClinkMode) {
        val msg = ModeChangeMsg(mode = mode.id)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendStrokeStart(id: String, tool: DrawingTool, colorHex: String, width: Float, x: Float, y: Float, pressure: Float) {
        val msg = StrokeStartMsg(
            stroke_id = id,
            tool = tool.id,
            color = colorHex,
            width = width,
            x = x,
            y = y,
            pressure = pressure
        )
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendStrokeMove(id: String, points: List<ProtocolPoint>) {
        val msg = StrokeMoveMsg(stroke_id = id, points = points)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendStrokeEnd(id: String, x: Float, y: Float, pressure: Float) {
        val msg = StrokeEndMsg(stroke_id = id, x = x, y = y, pressure = pressure)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendCursorMove(dx: Float, dy: Float, x: Float, y: Float, pressure: Float) {
        val msg = CursorMoveMsg(dx = dx, dy = dy, x = x, y = y, pressure = pressure)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendPointerDown(button: Int, x: Float, y: Float, pressure: Float) {
        val msg = PointerDownMsg(button = button, x = x, y = y, pressure = pressure)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendPointerUp(button: Int, x: Float, y: Float) {
        val msg = PointerUpMsg(button = button, x = x, y = y)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    fun sendUndo() = sendMessage(ClinkJson.encodeToString(SimpleActionMsg("undo")))
    fun sendRedo() = sendMessage(ClinkJson.encodeToString(SimpleActionMsg("redo")))
    fun sendClear() = sendMessage(ClinkJson.encodeToString(SimpleActionMsg("clear")))

    fun sendLaser(x: Float, y: Float, active: Boolean) {
        val msg = LaserPointerMsg(x = x, y = y, active = active)
        sendMessage(ClinkJson.encodeToString(msg))
    }

    private fun startPingLoop() {
        mainHandler.postDelayed(object : Runnable {
            override fun run() {
                if (_status.value != ConnectionStatus.DISCONNECTED && _status.value != ConnectionStatus.ERROR) {
                    pingStartTime = System.currentTimeMillis()
                    val ping = PingPongMsg(type = "ping", timestamp = pingStartTime)
                    sendMessage(ClinkJson.encodeToString(ping))
                    mainHandler.postDelayed(this, 3000)
                }
            }
        }, 1000)
    }

    fun sendMessage(text: String): Boolean {
        return webSocket?.send(text) ?: false
    }
}
