# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
