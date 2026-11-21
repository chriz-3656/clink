package com.chriz.clink.data.preferences

import android.content.Context
import android.content.SharedPreferences
import java.util.UUID

class ClinkPreferences(context: Context) {
    private val prefs: SharedPreferences = context.getSharedPreferences("clink_prefs", Context.MODE_PRIVATE)

    var host: String
        get() = prefs.getString("host", "127.0.0.1") ?: "127.0.0.1"
        set(value) = prefs.edit().putString("host", value).apply()

    var port: Int
        get() = prefs.getInt("port", 8765)
        set(value) = prefs.edit().putInt("port", value).apply()

    var authToken: String
        get() = prefs.getString("auth_token", "") ?: ""
        set(value) = prefs.edit().putString("auth_token", value).apply()

    var deviceId: String
        get() {
            var id = prefs.getString("device_id", "") ?: ""
            if (id.isBlank()) {
                id = "android_" + UUID.randomUUID().toString().replace("-", "").take(16)
                prefs.edit().putString("device_id", id).apply()
            }
            return id
        }
        set(value) = prefs.edit().putString("device_id", value).apply()

    var palmRejectionEnabled: Boolean
        get() = prefs.getBoolean("palm_rejection", true)
        set(value) = prefs.edit().putBoolean("palm_rejection", value).apply()

    var stylusSensitivity: Float
        get() = prefs.getFloat("stylus_sensitivity", 1.0f)
        set(value) = prefs.edit().putFloat("stylus_sensitivity", value).apply()

    var darkTheme: Boolean
        get() = prefs.getBoolean("dark_theme", true)
        set(value) = prefs.edit().putBoolean("dark_theme", value).apply()
}
