package com.chriz.clink.viewmodel

import android.app.Application
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.chriz.clink.data.model.*
import com.chriz.clink.data.network.ConnectionStatus
import com.chriz.clink.data.network.NetworkUtils
import com.chriz.clink.data.network.WebSocketClient
import com.chriz.clink.data.preferences.ClinkPreferences
import com.chriz.clink.data.repository.SavedSession
import com.chriz.clink.data.repository.SessionRepository
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.util.*

class MainViewModel(application: Application) : AndroidViewModel(application) {
    val preferences = ClinkPreferences(application)
    val sessionRepo = SessionRepository(application)
    val client = WebSocketClient()

    private val _strokes = MutableStateFlow<List<CanvasStroke>>(emptyList())
    val strokes: StateFlow<List<CanvasStroke>> = _strokes.asStateFlow()

    private val undoStack = mutableListOf<List<CanvasStroke>>()
    private val redoStack = mutableListOf<List<CanvasStroke>>()

    private val _activeStroke = MutableStateFlow<CanvasStroke?>(null)
    val activeStroke: StateFlow<CanvasStroke?> = _activeStroke.asStateFlow()

    private val _currentTool = MutableStateFlow(DrawingTool.PEN)
    val currentTool: StateFlow<DrawingTool> = _currentTool.asStateFlow()

    private val _currentColor = MutableStateFlow(Color(0xFFFF3366))
    val currentColor: StateFlow<Color> = _currentColor.asStateFlow()

    private val _currentWidth = MutableStateFlow(4.0f)
    val currentWidth: StateFlow<Float> = _currentWidth.asStateFlow()

    private val _mode = MutableStateFlow(ClinkMode.ABSOLUTE_ANNOTATION)
    val mode: StateFlow<ClinkMode> = _mode.asStateFlow()

    private val _laserPos = MutableStateFlow<Offset?>(null)
    val laserPos: StateFlow<Offset?> = _laserPos.asStateFlow()

    private val _sessions = MutableStateFlow<List<SavedSession>>(emptyList())
    val sessions: StateFlow<List<SavedSession>> = _sessions.asStateFlow()

    private val _pairingError = MutableStateFlow<String?>(null)
    val pairingError: StateFlow<String?> = _pairingError.asStateFlow()

    init {
        client.onTokenReceived = { token ->
            preferences.authToken = token
            _pairingError.value = null
        }
        client.onPairingFailed = { errorMsg ->
            _pairingError.value = errorMsg
        }
        client.onAuthenticated = {
            val timerSec = preferences.annotationTimerSeconds
            client.sendTimer(timerSec)
        }
        refreshSessions()
    }

    fun refreshSessions() {
        _sessions.value = sessionRepo.listSessions()
    }

    fun clearPairingError() {
        _pairingError.value = null
    }

    fun setTool(tool: DrawingTool) {
        _currentTool.value = tool
    }

    fun setColor(color: Color) {
        _currentColor.value = color
    }

    fun setWidth(width: Float) {
        _currentWidth.value = width
    }

    fun setMode(newMode: ClinkMode) {
        _mode.value = newMode
        client.sendMode(newMode)
    }

    fun setAnnotationTimer(seconds: Int) {
        preferences.annotationTimerSeconds = seconds
        client.sendTimer(seconds)
    }

    fun connectUsb() {
        _pairingError.value = null
        val gateway = NetworkUtils.getGatewayIp(getApplication())
        client.connectUsb(preferences.port, preferences.authToken, preferences.deviceId, fallbackHost = gateway)
    }

    fun connectWifi(host: String, port: Int) {
        preferences.host = host
        preferences.port = port
        _pairingError.value = null
        client.connect(host, port, preferences.authToken, preferences.deviceId)
    }

    fun submitPin(pin: String) {
        _pairingError.value = null
        client.sendPairRequest(pin)
    }

    fun setPairingError(err: String?) {
        _pairingError.value = err
    }

    fun handleQrPayload(payload: String): Boolean {
        try {
            var h = "127.0.0.1"
            var p = 8765
            var pin = ""
            val trimmed = payload.trim()

            if (trimmed.startsWith("clink://")) {
                val query = trimmed.substringAfter("?", "")
                val map = query.split("&").associate {
                    val kv = it.split("=")
                    kv[0] to kv.getOrElse(1) { "" }
                }
                h = map["host"] ?: "127.0.0.1"
                p = map["port"]?.toIntOrNull() ?: 8765
                pin = map["pin"] ?: ""
            } else if (":" in trimmed) {
                val parts = trimmed.split(":")
                h = parts[0]
                if (parts.size >= 2) p = parts[1].toIntOrNull() ?: 8765
                if (parts.size >= 3) pin = parts[2]
            } else {
                return false
            }

            preferences.host = h
            preferences.port = p
            _pairingError.value = null
            if (pin.isNotBlank()) {
                client.setPendingPin(pin)
            }
            client.connect(h, p, preferences.authToken, preferences.deviceId)
            return true
        } catch (e: Exception) {
            return false
        }
    }

    fun disconnect() {
        client.disconnect()
        _pairingError.value = null
    }

    fun startStroke(normX: Float, normY: Float, pressure: Float) {
        val tool = _currentTool.value
        val color = _currentColor.value
        val width = _currentWidth.value
        val strokeId = "s_${System.currentTimeMillis()}_${UUID.randomUUID().toString().take(6)}"

        if (tool == DrawingTool.LASER) {
            client.sendLaser(normX, normY, true)
            return
        }

        val point = CanvasPoint(normX, normY, pressure)
        val stroke = CanvasStroke(
            id = strokeId,
            tool = tool,
            color = color.value.toLong(),
            width = width,
            points = mutableListOf(point),
            isCompleted = false
        )
        _activeStroke.value = stroke

        val hexColor = String.format("#%06X", (0xFFFFFF and color.toArgb()))
        client.sendStrokeStart(strokeId, tool, hexColor, width, normX, normY, pressure)
    }

    fun appendStrokePoints(newPoints: List<CanvasPoint>) {
        val stroke = _activeStroke.value
        if (_currentTool.value == DrawingTool.LASER) {
            newPoints.lastOrNull()?.let {
                client.sendLaser(it.x, it.y, true)
            }
            return
        }
        if (stroke != null) {
            stroke.points.addAll(newPoints)
            val dtos = newPoints.map { ProtocolPoint(it.x, it.y, it.pressure, it.timestamp) }
            client.sendStrokeMove(stroke.id, dtos)
        }
    }

    fun endStroke(normX: Float, normY: Float, pressure: Float) {
        if (_currentTool.value == DrawingTool.LASER) {
            client.sendLaser(normX, normY, false)
            return
        }
        val stroke = _activeStroke.value
        if (stroke != null) {
            stroke.points.add(CanvasPoint(normX, normY, pressure))
            stroke.isCompleted = true

            undoStack.add(_strokes.value.toList())
            redoStack.clear()

            _strokes.value = _strokes.value + stroke
            _activeStroke.value = null

            client.sendStrokeEnd(stroke.id, normX, normY, pressure)

            val timerSec = preferences.annotationTimerSeconds
            if (timerSec > 0) {
                viewModelScope.launch {
                    delay(timerSec * 1000L)
                    _strokes.value = _strokes.value.filter { it.id != stroke.id }
                }
            }
        }
    }

    fun sendCursorMove(dx: Float, dy: Float, normX: Float, normY: Float, pressure: Float) {
        client.sendCursorMove(dx, dy, normX, normY, pressure)
    }

    fun sendPointerDown(button: Int, normX: Float, normY: Float, pressure: Float) {
        client.sendPointerDown(button, normX, normY, pressure)
    }

    fun sendPointerUp(button: Int, normX: Float, normY: Float) {
        client.sendPointerUp(button, normX, normY)
    }

    fun undo() {
        if (_strokes.value.isNotEmpty() || undoStack.isNotEmpty()) {
            redoStack.add(_strokes.value.toList())
            if (undoStack.isNotEmpty()) {
                _strokes.value = undoStack.removeAt(undoStack.lastIndex)
            } else {
                _strokes.value = _strokes.value.dropLast(1)
            }
            client.sendUndo()
        }
    }

    fun redo() {
        if (redoStack.isNotEmpty()) {
            undoStack.add(_strokes.value.toList())
            _strokes.value = redoStack.removeAt(redoStack.lastIndex)
            client.sendRedo()
        }
    }

    fun clearCanvas() {
        if (_strokes.value.isNotEmpty()) {
            undoStack.add(_strokes.value.toList())
            redoStack.clear()
            _strokes.value = emptyList()
            _activeStroke.value = null
            client.sendClear()
        }
    }

    fun saveSession(name: String) {
        sessionRepo.saveSession(name, _strokes.value)
        refreshSessions()
    }

    fun loadSession(id: String) {
        val loaded = sessionRepo.loadSession(id)
        if (loaded != null) {
            undoStack.add(_strokes.value.toList())
            _strokes.value = loaded
            for (s in loaded) {
                val hexColor = String.format("#%06X", (0xFFFFFF and s.toComposeColor().toArgb()))
                if (s.points.isNotEmpty()) {
                    val p0 = s.points[0]
                    client.sendStrokeStart(s.id, s.tool, hexColor, s.width, p0.x, p0.y, p0.pressure)
                    val rest = s.points.drop(1).map { ProtocolPoint(it.x, it.y, it.pressure, it.timestamp) }
                    if (rest.isNotEmpty()) {
                        client.sendStrokeMove(s.id, rest)
                    }
                    val last = s.points.last()
                    client.sendStrokeEnd(s.id, last.x, last.y, last.pressure)
                }
            }
        }
    }

    fun deleteSession(id: String) {
        sessionRepo.deleteSession(id)
        refreshSessions()
    }
}
