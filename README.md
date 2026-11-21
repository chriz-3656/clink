<div align="center">

# 🪄 Clink

### *Your Android. Your Desktop. One Canvas.*

[![GitHub Release](https://img.shields.io/github/v/release/chriz-3656/clink?style=for-the-badge&color=FF3366)](https://github.com/chriz-3656/clink/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/chriz-3656/clink/release.yml?branch=main&style=for-the-badge)](https://github.com/chriz-3656/clink/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Android Target](https://img.shields.io/badge/Android-API%2026--35%20(Android%2015)-3DDC84?style=for-the-badge&logo=android&logoColor=white)](android/)
[![Linux](https://img.shields.io/badge/Linux-X11%20%7C%20GTK3%20%7C%20Cairo-E95420?style=for-the-badge&logo=linux&logoColor=white)](linux/)

<p align="center">
  <b>Clink</b> transforms your Android phone or tablet into an ultra-low-latency wireless/USB drawing tablet and transparent real-time screen annotation overlay for your Linux desktop.
</p>

[Quick Start](#-quick-start) •
[Features](#-features) •
[Connection Modes](#-connection-transports) •
[Portable Hotspot Guide](docs/hotspot-guide.md) •
[Play Store Guide](docs/playstore-publishing.md) •
[Architecture](docs/architecture.md)

</div>

---

## 💡 Why Clink?

Most screen drawing tools require heavy screen capture, video encoding, or proprietary cloud streaming that lag and consume massive CPU/GPU resources.

**Clink takes a radically different approach:**
- Only **lightweight vectorized motion and stroke coordinates** are transmitted over local WebSockets.
- The Linux companion draws strokes locally in hardware using **Cairo** over a native **GTK3 transparent click-through overlay**.
- **Zero screen freezing**: Your mouse clicks, dock, panels, and keyboard shortcuts pass 100% through to your applications underneath.
- **Offline & Local-First**: No internet access, accounts, or telemetry required.

---

## ✨ Features

### 🖌️ Mode A: Absolute Desktop Annotation
Draw notes, sketch diagrams, highlight code, and present slides directly on top of your live desktop:
- **Click-Through Pass-Through**: Full native X11 input shape integration (`libXfixes` / `libXext`). The overlay never blocks your mouse or desktop apps.
- **Minimization-Safe**: Built with an override-redirect window so drawings stay active even when the companion window is minimized.
- **Comprehensive Tool Palette**:
  - **Pen**, **Pencil**, **Highlighter**, **Eraser**, **Laser Pointer**
  - Geometric shapes: **Line**, **Arrow**, **Rectangle**, **Circle**
- **Configurable Auto-Fade Duration**:
  - Keep annotations forever (`∞`) or set auto-fade timers (**3s**, **5s**, **10s**, **30s**, **60s**).
  - Automatically vanishes from both the Android phone canvas and Linux desktop.
- **Instant Sweep Clear**: One-tap full canvas clear button on the action bar.
- **Active Tool Pill Indicator**: Animated pop-up confirming your active tool whenever you switch.

### 🖊️ Mode B: Relative Graphics Tablet
Turn your phone or tablet into an external drawing tablet with genuine X11 pointer and stylus injection:
- Works with **Krita**, **GIMP**, **Inkscape**, **Blender**, and general desktop window managers.
- Pressure sensitivity support, adjustable speed, and acceleration curves.
- Automatic mouse button safety cleanup on disconnects.

---

## 📡 Connection Transports

Clink supports three distinct connection methods to fit any workflow:

| Transport | Setup | Latency | Internet Needed? | Router Needed? |
| :--- | :--- | :---: | :---: | :---: |
| **🔌 USB (ADB Reverse)** | Plug in cable (USB debugging on) | **< 1 ms** | ❌ No | ❌ No |
| **📶 Wi-Fi Local Network** | Scan desktop QR code with camera | **~3–5 ms** | ❌ No | ✅ Yes |
| **⚡ Portable Hotspot** | Connect PC to phone hotspot | **~2–3 ms** | ❌ No | ❌ No |

### ⚡ Portable Hotspot (Zero Router, Zero Internet)
Don't have a Wi-Fi router or USB cable? Turn on **Personal Hotspot** on your Android device (mobile data can be OFF), connect your PC's Wi-Fi to your phone, and scan the QR code. You get direct antenna-to-antenna wireless communication anywhere—in a park, on a train, or in a lecture hall. See our [Portable Hotspot Guide](docs/hotspot-guide.md).

---

## 🚀 Quick Start

### 1. Install Linux Companion

Download the latest release package from the [Releases Page](https://github.com/chriz-3656/clink/releases):

```bash
# Debian / Ubuntu / Linux Mint
sudo dpkg -i clink_1.0.0_all.deb
sudo apt-get install -f   # resolve any missing runtime libraries

# Launch the GUI Control Center
clink gui
```

### 2. Install Android App

Download `clink-android-release.apk` from [Releases](https://github.com/chriz-3656/clink/releases) and install it on your device:

```bash
adb install clink-android-release.apk
```

### 3. Pair & Connect

- **Instant Optical QR Pairing (Wi-Fi / Hotspot)**:  
  Open Clink on Android, tap the **QR Scanner icon** at the top right, and scan the QR code displayed on your Linux monitor.
- **Instant USB (Cabled)**:  
  Connect your phone with a USB cable (with USB debugging enabled), tap **Connect via USB**, and start drawing immediately.

---

## 🖥️ Linux Control Center & CLI

Clink includes both a modern GTK3 GUI Control Center and an automated CLI:

<div align="center">
  <img src="assets/gui_preview.png" alt="Clink Linux Control Center" width="550" />
</div>

### CLI Commands:
```bash
clink start        # Launch headless daemon and transparent overlay
clink stop         # Terminate companion daemon
clink status       # View service status, port, and connected devices
clink connect      # Configure USB port forwarding via adb reverse
clink devices      # View and manage paired Android devices
clink doctor       # Run system diagnostics (X11, GTK3, Cairo, ADB, ports)
clink config       # View or update configuration settings
clink gui          # Open the graphical Control Center
```

---

## 🏗️ Architecture

```
┌──────────────────────────────────────────────┐
│                Android Client                │
│ • Jetpack Compose Canvas (0ms ink response)  │
│ • MotionEvent Stylus & Pressure Engine       │
│ • ZXing Optical Camera QR Pairing            │
│ • OkHttp WebSocket Protocol Encoder          │
└──────────────────────┬───────────────────────┘
                       │
       USB / Wi-Fi LAN / Portable Hotspot
       WebSocket JSON Protocol (Port 8765)
                       │
┌──────────────────────v───────────────────────┐
│            Linux Desktop Companion           │
│ • Python 3 Asyncio WebSocket Server          │
│ • Pairing Manager & Shared PIN Auth          │
│ • Cairo Antialiased Drawing Engine           │
│ • GTK3 Transparent Override-Redirect Overlay │
│ • Native X11 XFixes Empty Shape Pass-Through │
│ • X11 XTest Relative Pointer Injector        │
└──────────────────────────────────────────────┘
```

See [docs/architecture.md](docs/architecture.md) for full architectural documentation and protocol specifications.

---

## 🔒 Security & Privacy

- **100% Offline**: No network traffic ever leaves your local network or USB cable.
- **PIN & Token Authentication**: Devices must authenticate using a 6-digit rolling PIN or cryptographically signed session token before input injection or canvas drawing is accepted.
- **Input Cleanup**: Automatically releases pressed mouse buttons and modifiers if connection drops.
- **Strict Privacy**: View our full [Privacy Policy](PRIVACY_POLICY.md).

---

## 📦 Publishing Roadmap

- [x] GitHub Releases (`.deb`, `.tar.gz`, `.whl`, `.apk`)
- [ ] [Google Play Store Publishing](docs/playstore-publishing.md) (In Review)
- [ ] F-Droid Official Inclusion
- [ ] Flathub Flatpak for Linux

---

## 👨‍💻 Developer & Project Details

- **Maintainer**: chriz-3656
- **GitHub**: [https://github.com/chriz-3656/clink](https://github.com/chriz-3656/clink)
- **Contact / Inquiries**: [chrizmonsaji@proton.me](mailto:chrizmonsaji@proton.me)
- **Bug Reports & Feature Requests**: [GitHub Issues](https://github.com/chriz-3656/clink/issues)

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.
