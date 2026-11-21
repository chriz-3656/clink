# Changelog

## [1.0.0] - 2026-11-21
### Added
- Native Android application with Jetpack Compose drawing workspace.
- Dual-mode support: Absolute Desktop Annotation and Relative Graphics Tablet.
- GTK3 transparent desktop overlay with click-through support and Cairo antialiased rendering.
- X11 XTest pointer input injector with automatic disconnect cleanup.
- Real-time WebSocket communication protocol with normalized coordinates.
- USB connection via ADB reverse forwarding (`adb reverse tcp:8765 tcp:8765`).
- Wi-Fi connection with local PIN pairing and token authentication.
- Drawing tools: Pen, Pencil, Highlighter, Eraser, Laser Pointer, Line, Arrow, Rectangle, Circle.
- Session saving, loading, and PNG export.
- Command-line interface with `start`, `stop`, `status`, `connect`, `devices`, `doctor`, `config`, `version`.
- Comprehensive automated test suites for Android and Linux.
- Debian (.deb), Python Wheel (.whl), source distribution, and Android APK artifacts.
