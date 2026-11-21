# Clink System Architecture

Clink is designed as a modular, low-latency drawing and desktop annotation monorepo comprising a native Android client and a Linux companion service.

```
┌─────────────────────────────────────────────────────────────┐
│                       Android Client                        │
│  • Jetpack Compose Hardware Accelerated Canvas              │
│  • MotionEvent Stylus, Pressure, & Palm Rejection Pipeline  │
│  • ZXing Camera QR Code Optical Scanner                     │
│  • OkHttp High-Performance WebSocket Transport              │
│  • Local Session Repository (Room/JSON)                     │
└──────────────────────────────┬──────────────────────────────┘
                               │
                USB (adb reverse) / Wi-Fi LAN / Hotspot
                WebSocket JSON Protocol (Port 8765)
                               │
┌──────────────────────────────v──────────────────────────────┐
│                   Linux Desktop Companion                   │
│  • Asyncio WebSocket Server & Session Dispatcher            │
│  • Pairing Manager with Shared 6-Digit Rolling PIN & Tokens │
│  • Cairo Vector Antialiased Offscreen Surface               │
│  • Transparent POPUP (Override-Redirect) Desktop Overlay    │
│  • Native X11 XFixes & XShape Empty Input Pass-Through      │
│  • X11 XTest Relative Pointer & Button Event Injector       │
│  • GTK3 Graphical Control Center & Daemon CLI               │
└─────────────────────────────────────────────────────────────┘
```

---

## Component Deep Dive

### 1. Transparent Overlay Engine (`clink.overlay`)
- **Override-Redirect Architecture**: Constructed as `Gtk.Window(type=Gtk.WindowType.POPUP)` so that the window manager never iconifies, groups, or unmaps the annotation canvas when the user minimizes the Control Center or switches workspaces.
- **X11 Input Shape Pass-Through**: Uses `libXfixes.so` (`XFixesCreateRegion(dpy, None, 0)`) and `libXext.so` (`XShapeCombineRectangles`) to create a completely empty input region for `ShapeInput` (`2`). Mouse and pointer events bleed 100% through to whatever window, panel, or dock is underneath.
- **Double-Buffered Cairo Pipeline**:
  - A persistent Cairo image surface holds finalized strokes.
  - An ephemeral overlay pass renders active in-progress geometric shapes (rectangles, arrows, circles) and the real-time laser pointer dot without redrawing the whole canvas.

### 2. Input Injection Engine (`clink.input`)
- **Direct X11 XTest**: Utilizes `python-xlib` to interface with the X11 XTest extension, allowing cursor positioning and mouse button presses directly into the X server without requiring root privileges.
- **Safety Auto-Release**: Tracks all active virtual buttons (`1=Left`, `2=Middle`, `3=Right`) and automatically releases them on connection drop, error, or mode switch to prevent stuck clicks.

### 3. Network Transport & Protocol (`clink.protocol` & `clink.server`)
- **WebSocket Protocol**: Fully asynchronous event-driven communication on port `8765`.
- **Coordinate Normalization**: All spatial coordinates are transmitted as floating-point ratios `[0.0, 1.0]`, allowing phones and tablets with any aspect ratio or resolution to map accurately to any Linux display geometry (1080p, 1440p, 4K, ultrawide).
- **Auto-Fade Stroke Expiry**: Strokes carry unique IDs and completion timestamps. When a fade timeout (e.g. 5 seconds) is active, expired strokes are pruned from the vector stack and automatically redrawn.

---

## Protocol Specification

| Message Type | Direction | Payload Attributes | Purpose |
| :--- | :--- | :--- | :--- |
| `hello` | Phone → PC | `device_id`, `device_name`, `token?` | Initial handshake |
| `hello_ack` | PC → Phone | `display` (width, height, aspect), `authenticated` | Geometry info & auth status |
| `pair_request` | Phone → PC | `device_id`, `pin` | Submit 6-digit pairing PIN |
| `pair_ack` | PC → Phone | `success`, `token?`, `message?` | Return authenticated session token |
| `stroke_start`| Phone → PC | `stroke_id`, `tool`, `color`, `width`, `x`, `y`, `pressure` | Begin new drawing stroke |
| `stroke_move` | Phone → PC | `stroke_id`, `points: [{x, y, p, t}, ...]` | Batch of interpolated points |
| `stroke_end`  | Phone → PC | `stroke_id`, `x`, `y`, `pressure` | Finalize stroke |
| `set_timer`   | Phone → PC | `seconds` (0 for persistent, 3..60) | Configure auto-fade timeout |
| `cursor_move` | Phone → PC | `dx`, `dy`, `x`, `y`, `pressure` | Relative tablet cursor motion (Mode B) |
| `pointer_down`| Phone → PC | `button` (1, 2, 3), `x`, `y`, `pressure` | Mouse click down (Mode B) |
| `pointer_up`  | Phone → PC | `button` (1, 2, 3), `x`, `y` | Mouse click up (Mode B) |
| `clear`       | Phone → PC | *(none)* | Erase all active annotations |
| `undo` / `redo`| Phone → PC| *(none)* | History navigation |
