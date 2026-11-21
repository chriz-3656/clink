# Clink System Architecture

Clink connects an Android device (stylus/touch canvas) to a Linux X11 desktop companion.

```
+------------------------------------+
|          Android Client            |
| - Jetpack Compose Canvas (0ms ink) |
| - MotionEvent Stylus Engine        |
| - Protocol Encoder / WebSocket     |
+-----------------+------------------+
                  |
    USB (adb reverse) or Wi-Fi LAN
    WebSocket (Default port 8765)
                  |
+-----------------v------------------+
|           Linux Companion          |
| - Asyncio WebSocket Server         |
| - Device Pairing & Token Auth      |
| - Cairo Antialiased Drawing Engine |
| - GTK3 Transparent Overlay Window  |
| - X11 XTest Input Injector         |
+------------------------------------+
```

## Core Modules
1. **Overlay Engine (`clink.overlay`)**:
   - GTK 3 window configured with an RGBA visual and `input_shape_combine_region` for click-through.
   - Dual-buffer Cairo rendering for persistent annotations and ephemeral tools (laser pointer).
2. **Input Injection (`clink.input`)**:
   - Uses X11 XTest extension for relative cursor positioning and tablet button actions without requiring root.
   - Cleans up button state on disconnections.
3. **Communication (`clink.protocol` & `clink.server`)**:
   - JSON-based protocol with normalized `[0.0, 1.0]` coordinates.
