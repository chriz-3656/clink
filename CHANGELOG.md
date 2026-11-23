# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.8] - 2026-11-23

### Added
- **Studio Desktop Companion Redesign**:
  - Replaced all emojis and non-standard unicode glyphs with crisp, vector-rendered graphics using `ui.painter()`, eliminating missing font tofu boxes (`[]`) across all Linux distributions.
  - Added custom vector brand emblem with interconnected pink and cyan node vectors, bold `CLINK` typography, and `STUDIO` badge.
  - Implemented responsive centered studio container constrained to maximum 540px width (`ui.vertical_centered`), preventing cards and UI elements from stretching out of proportion when resizing or maximizing on widescreen and 4K displays.
- **Integrated Live Activity & WebSocket Logs Console**:
  - Added thread-safe in-memory activity logging engine (`desktop/src/logging.rs`).
  - Added a dedicated **Activity Logs** channel in the desktop companion displaying real-time events (`[SYSTEM]`, `[INIT]`, `[PAIR]`, `[AUTH]`, `[MODE]`, `[INK]`, `[TIMER]`, `[CANVAS]`) with `[Copy Logs]` and `[Clear Logs]` action buttons.
- **Dedicated Tablet Controller Dock (Android)**:
  - Added bottom controller toolbar in Graphics Tablet mode featuring `[ Left Click ]`, `[ Drag Lock ]` (toggle), and `[ Right Click ]` buttons.
  - Added gesture recognition: tap-to-click for left click and two-finger tap for right click.

### Fixed
- **Graphics Tablet Relative Pointer Injection**:
  - Fixed X11 XTest protocol call in `x11.rs`: switched `detail` argument from `0` (absolute root coordinates) to `1` (relative displacement), resolving the issue where cursor snapped to the top-left screen edge.
  - Added fractional subpixel accumulator (`accum_x`, `accum_y`) in `X11InputBackend` to preserve micro-movements and prevent integer truncation stutter during slow, precise pointing.
  - Replaced fractional screen division (`/ w`, `/ h`) in Android `DrawingCanvas.kt` with physical pixel deltas scaled by preference sensitivity setting (`rawDx * sensitivity`).
  - Removed premature `button_down(1)` on finger contact so finger touch freely guides the cursor like a precision trackpad without locking into an accidental drag-select state.
- **Arrow Drawing Tool Across Platforms**:
  - Fixed panic `clamp: min > max` in desktop tiny-skia overlay renderer on short/medium strokes.
  - Fixed dynamic `coerceIn` exception in Android Compose Canvas for arrows.
  - Standardized arrowhead geometry to a solid-filled polygon matching between client and overlay.
  - Fixed 64-bit Compose color alpha shift in `StrokeModels.kt` that caused drawn strokes to render transparent.
  - Refined palm rejection to filter hardware palm events (`TOOL_TYPE_PALM`) without dropping synthetic or firm finger touches.

---

## [1.0.7] - 2026-11-22

### Added
- **Native Rust Companion (`desktop/`)**: Built a native, single-binary Rust companion (`clink`) powered by `tokio`, `tokio-tungstenite`, `x11rb`, `tiny-skia`, and `qrcode`. Delivers deterministic sub-millisecond latency, instant startup (<50ms), and only ~8MB memory footprint with zero Python or C-library dependencies.
- **Canvas Undo & Redo System**:
  - **Android**: Added Undo and Redo quick actions to both the top action bar and floating bottom toolbar, backed by reactive `canUndo` and `canRedo` StateFlows and animated pill toast confirmations.
  - **Protocol & Desktop Overlay**: Added `undo` (`undo_ack`) and `redo` (`redo_ack`) WebSocket messages, maintaining a full stroke history stack on both Rust and Python companions with immediate Cairo/tiny-skia redraws.
- **Rendered GTK Icons & User Theme Priority**: Replaced raw unicode emojis in the Linux Control Center with native freedesktop vector icons (`Gtk.Image.new_from_icon_name`), added action icons to all primary/secondary buttons, and elevated CSS priority to `GTK_STYLE_PROVIDER_PRIORITY_USER` (800) so the cyberpunk dark theme consistently renders across all Linux desktop environments.

### Fixed
- **Play Store Content Clean-Up**: Removed all non-FOSS Play Store assets, focusing the repository on pure open-source distributions (F-Droid and Flathub).

---

## [1.0.6] - 2026-11-21

### Added
- **Matching Dark/Cyberpunk Theme on Linux**: Implemented custom GTK3 CSS provider theming matching the Android app aesthetic (`#121218` dark background, `#191926` rounded card containers, Clink Pink `#FF3366` primary buttons, dark glass secondary buttons, and active status pill badges).
- **Custom Section Icons & Modern Cards**: Replaced standard GTK frames with modern card containers featuring custom icons (`⚡ COMPANION SERVICE`, `📱 INSTANT PAIRING`, `🔌 USB CONNECTION`, `🎨 DESKTOP OVERLAY`, `📲 PAIRED DEVICES`, `ℹ️ DEVELOPER & PROJECT`).

### Fixed
- **Synchronized Artifact Version Naming**: Updated all package metadata and artifacts from outdated `1.0.0` references to dynamic semantic versioning (`clink_1.0.6_all.deb`, `clink-linux-1.0.6-x86_64.tar.gz`, `clink-1.0.6-py3-none-any.whl`, Android `versionCode 6` / `versionName 1.0.6`).

---

## [1.0.5] - 2026-11-21

### Fixed
- **Screen Interaction & Click-Through**: Replaced invalid XFixes region `0` with a genuine empty region created via `libXfixes.XFixesCreateRegion(dpy, None, 0)` and set as `ShapeInput` (`2`), completely eliminating the desktop "invisible barrier" where clicks were blocked.
- **Window Manager Minimization**: Configured `OverlayWindow` with `Gtk.WindowType.POPUP` (override-redirect) so the transparent drawing canvas stays active and rendering on screen even when the Clink Control Center window is minimized.
- **Sub-window Event Interception**: Removed the child `Gtk.DrawingArea` widget and connected the Cairo `"draw"` handler directly to the window with `set_app_paintable(True)`.
- **Annotation Duration Auto-Fade**: Synchronized stroke auto-clearing across both platforms:
  - **Android**: Added coroutine delay filter in `MainViewModel.kt` `endStroke` to automatically remove expired strokes from the phone canvas.
  - **Connection Handshake**: Added `onAuthenticated` callback in `WebSocketClient.kt` to transmit the configured timer duration immediately on connection.
  - **Drawing Screen Quick Toggle**: Added an action bar timer button (`⏱️ ∞`, `3s`, `5s`, `10s`, `30s`, `60s`) with animated visual pill toasts.
- **Dynamic Network & Hotspot IP Refresh**: Added automatic IP detection in the GUI Control Center so transitioning to a Portable Hotspot or switching Wi-Fi networks dynamically updates the listening address and pairing QR code.

---

## [1.0.4] - 2026-11-21

### Added
- **Developer Details**: Added developer and project information cards on both Android Settings (`chriz-3656`, `https://github.com/chriz-3656/clink`, `chrizmonsaji@proton.me`) and the Linux Control Center.
- **Active Tool Switch Indicator**: Animated top-center pill pop-up on Android canvas displaying the active tool name on switch.
- **Instant Clear All**: Top bar quick-clear sweep button to erase all drawings with a single tap.
- **Duration / Auto-Clear Setting**: Added annotation retention selector (Never, 3s, 5s, 10s, 30s, 60s) in Settings and Control Center.

### Fixed
- **M3 SegmentedButton Layout**: Fixed icon and text overlap on the Home Screen connection mode selector by utilizing distinct M3 `icon` and `label` slots.
- **Camera QR Scanner Orientation**: Locked `CaptureActivity` to portrait orientation (`android:screenOrientation="portrait"`) with `setOrientationLocked(true)`.

---

## [1.0.3] - 2026-11-21

### Added
- **Camera QR Code Scanner**: Integrated ZXing camera scanner on Android for 1-tap optical pairing against the desktop monitor.
- **Auto ADB Reverse Configuration**: Background timer in Linux Control Center automatically detects connected USB devices and runs `adb reverse tcp:8765 tcp:8765`.
- **USB Tethering Gateway Fallback**: Added automatic fallback to the network gateway IP if localhost is unreachable over USB tethering.

---

## [1.0.2] - 2026-11-21

### Fixed
- **CI Dependency Fix**: Added `gir1.2-gtk-3.0` and PyGObject introspection typelib dependencies to GitHub Actions workflow environments.
- **Headless Fallback**: Added safe headless fallback for `OverlayWindow` and `X11InputInjector` when running in headless test containers.

---

## [1.0.1] - 2026-11-21

### Fixed
- **Device ID Persistence**: Persisted unique hardware UUID across app reboots in `ClinkPreferences.kt`.
- **Shared Pairing State**: Unified PIN storage so CLI and GUI Control Center share the active pairing code.

---

## [1.0.0] - 2026-11-21

### Added
- **Monorepo Architecture**: Initial production release of Clink Android app and Linux desktop companion.
- **Dual-Mode System**:
  - *Mode A (Absolute Annotation)*: Real-time transparent Cairo desktop overlay with antialiased drawing tools.
  - *Mode B (Relative Graphics Tablet)*: X11 input injection for Krita, GIMP, Inkscape, Blender, and desktop apps.
- **Drawing Tools**: Pen, Pencil, Highlighter, Eraser, Laser Pointer, Line, Arrow, Rectangle, and Circle.
- **Dual Transports**: Low-latency USB via ADB reverse port forwarding, and local Wi-Fi LAN with PIN pairing and token authentication.
- **Linux Control Center**: Native GTK3 GUI companion with real-time pairing status, PIN display, and USB forwarding controls.
- **Packaging**: Debian (`.deb`), Python Wheel (`.whl`), source tarballs, and Android APK artifacts.
