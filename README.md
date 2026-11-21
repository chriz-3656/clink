# Clink

> Your Android. Your Desktop. One Canvas.

Clink turns your Android phone or tablet into a wireless or USB drawing tablet and real-time desktop annotation canvas for Linux.

Draw directly over any Linux application—terminals, browsers, PDF viewers, code editors, or presentations—without screen streaming overhead.

---

## Key Features

- **Absolute Desktop Annotation (Mode A)**:
  - Transparent, always-on-top desktop overlay.
  - Full click-through pass-through to underlying windows.
  - Smooth antialiased ink rendering with Cairo.
  - Tools: Pen, Pencil, Highlighter, Eraser, Laser Pointer, Line, Arrow, Rectangle, Circle.
  - Multi-level Undo, Redo, and Clear.
- **Relative Graphics Tablet (Mode B)**:
  - Genuine X11 desktop pointer and button input injection.
  - Works with Krita, GIMP, Inkscape, Blender, and desktop apps.
  - Pressure support and adjustable sensitivity.
  - Automatic button release cleanup on disconnect.
- **Transports**:
  - **USB Mode**: Ultra-low latency via ADB reverse port forwarding.
  - **Wi-Fi Mode**: Wireless drawing over local LAN with PIN pairing and token auth.
- **Local & Private**: No cloud services, accounts, or internet access required.

---

## Quick Start

### 1. Start the Linux Companion
Install and launch the companion:
```bash
sudo dpkg -i dist/clink_1.0.0_all.deb
clink start
```

### 2. Connect via USB
Connect your Android device with USB debugging enabled, then run:
```bash
clink connect
```
Open Clink on Android and tap **Connect via USB**.

### 3. Connect via Wi-Fi
On Android, enter your Linux computer's local IP and tap **Connect via Wi-Fi**. If prompted, enter the 6-digit PIN shown on the Linux terminal.

---

## CLI Reference
- `clink start`: Launch the companion server and transparent overlay.
- `clink stop`: Stop the running companion daemon.
- `clink status`: Check service status, connected devices, and active mode.
- `clink connect`: Configure USB ADB reverse port forwarding.
- `clink devices`: View paired Android devices.
- `clink doctor`: Run system diagnostics (GTK3, Cairo, X11, ADB, ports).
- `clink version`: Display version information.

---

## Architecture

See [docs/architecture.md](docs/architecture.md) for architectural details and protocol specifications.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
