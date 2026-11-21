package com.chriz.clink.data.repository

import android.content.Context
import com.chriz.clink.data.model.CanvasPoint
import com.chriz.clink.data.model.CanvasStroke
import com.chriz.clink.data.model.DrawingTool
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.File

@Serializable
data class SavedPoint(val x: Float, val y: Float, val pressure: Float, val timestamp: Long)

@Serializable
data class SavedStroke(
    val id: String,
    val tool: String,
    val color: Long,
    val width: Float,
    val points: List<SavedPoint>,
    val isCompleted: Boolean
)

@Serializable
data class SavedSession(
    val id: String,
    val name: String,
    val createdAt: Long,
    val strokes: List<SavedStroke>
)

class SessionRepository(private val context: Context) {
    private val sessionsDir: File = File(context.filesDir, "sessions").apply { mkdirs() }
    private val json = Json { ignoreUnknownKeys = true; prettyPrint = true }

    fun saveSession(name: String, strokes: List<CanvasStroke>): String {
        val id = "session_${System.currentTimeMillis()}"
        val savedStrokes = strokes.map { s ->
            SavedStroke(
                id = s.id,
                tool = s.tool.id,
                color = s.color,
                width = s.width,
                points = s.points.map { SavedPoint(it.x, it.y, it.pressure, it.timestamp) },
                isCompleted = s.isCompleted
            )
        }
        val session = SavedSession(
            id = id,
            name = name.ifBlank { "Drawing ${System.currentTimeMillis()}" },
            createdAt = System.currentTimeMillis(),
            strokes = savedStrokes
        )
        val file = File(sessionsDir, "$id.json")
        file.writeText(json.encodeToString(session))
        return id
    }

    fun listSessions(): List<SavedSession> {
        val files = sessionsDir.listFiles { _, name -> name.endsWith(".json") } ?: return emptyList()
        return files.mapNotNull { file ->
            try {
                json.decodeFromString<SavedSession>(file.readText())
            } catch (e: Exception) {
                null
            }
        }.sortedByDescending { it.createdAt }
    }

    fun loadSession(id: String): List<CanvasStroke>? {
        val file = File(sessionsDir, "$id.json")
        if (!file.exists()) return null
        return try {
            val session = json.decodeFromString<SavedSession>(file.readText())
            session.strokes.map { s ->
                val tool = DrawingTool.entries.find { it.id == s.tool } ?: DrawingTool.PEN
                CanvasStroke(
                    id = s.id,
                    tool = tool,
                    color = s.color,
                    width = s.width,
                    points = s.points.map { CanvasPoint(it.x, it.y, it.pressure, it.timestamp) }.toMutableList(),
                    isCompleted = s.isCompleted
                )
            }
        } catch (e: Exception) {
            null
        }
    }

    fun deleteSession(id: String): Boolean {
        val file = File(sessionsDir, "$id.json")
        return file.exists() && file.delete()
    }
}
