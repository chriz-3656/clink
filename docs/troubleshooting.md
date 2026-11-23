# Clink Troubleshooting Guide

This guide covers common issues and their solutions when using Clink across Linux and Android.

---

## 1. Connection & Networking Issues

### ❓ "Connection Failed" on Android over Wi-Fi
- **Same Network**: Verify that both your Android phone and Linux PC are connected to the exact same Wi-Fi router or Hotspot.
- **Firewall Blocking Port 8765**: Linux distributions often enable a firewall by default (`ufw` on Ubuntu/Debian, `firewalld` on Fedora). Allow incoming TCP traffic on port 8765:
  ```bash
  # For UFW (Ubuntu, Debian, Linux Mint)
  sudo ufw allow 8765/tcp

  # For firewalld (Fedora, RHEL)
  sudo firewall-cmd --add-port=8765/tcp --permanent
  sudo firewall-cmd --reload
  ```
- **Public Wi-Fi / AP Isolation**: Many university, office, cafe, and hotel networks enable "Client Isolation", which blocks devices on the same Wi-Fi from communicating directly with each other.  
  *Solution*: Use the **Portable Hotspot** method (turn on hotspot on phone, connect PC to it) to create your own private direct network.

### ❓ QR Code shows `127.0.0.1` instead of LAN IP
- If the Clink Linux companion was launched before connecting to Wi-Fi or Hotspot, click the **"Refresh Network IP"** button in the Control Center (or click "Generate New PIN") to automatically detect your new subnet IP address.

### ❓ "Connection Failed" over USB
- **USB Debugging**: Ensure **Developer Options → USB Debugging** is toggled ON in your Android settings.
- **RSA Authorization**: When plugging in the phone, look at the phone screen and tap **"Always allow from this computer"** when prompted.
- **Verify ADB on Linux**:
  ```bash
  adb devices
  ```
  If it shows `unauthorized` or `offline`, unplug and reconnect the cable.
- **Port Forwarding**: Run `clink connect` on Linux to manually set up `adb reverse tcp:8765 tcp:8765`.

---

## 2. Desktop Overlay & Interaction Issues

### ❓ Desktop Clicks Feel Blocked
- Clink v1.0.5+ uses native `libXfixes` with an empty input shape (`XFixesCreateRegion(dpy, None, 0)`) and `gdk_win.set_pass_through(True)`.
- Make sure you are running release **v1.0.5** or later:
  ```bash
  clink version
  ```
- Ensure "Click-Through Mode" is switched **ON** in the Control Center.

### ❓ Overlay Disappears When Minimized
- Upgrade to **v1.0.5+**. In earlier versions, window managers treated the overlay as a grouped utility palette. In v1.0.5+, the overlay uses `Gtk.WindowType.POPUP` (override-redirect) which keeps it active and rendered on screen independently.

---

## 3. Stylus & Canvas Issues

### ❓ Stroke Doesn't Fade After 5 Seconds
- Ensure the timer duration is set to your desired time (`3s`, `5s`, `10s`, `30s`, `60s`) rather than `Never (Persistent)`.
- You can cycle the timer directly from the top bar of the Android drawing canvas by tapping the `⏱️` button.

### ❓ Accidental Palm Touches
- In Android Settings within the Clink app, toggle **"Palm Rejection"** ON. When enabled, touch events with large contact tool types or tool size thresholds will be rejected, prioritizing your stylus pen.

### ❓ Graphics Tablet Cursor Moving Too Fast or Slow
- Open Settings in the Clink Android app and adjust the **"Tablet Pointer Sensitivity"** slider (`0.5x` to `3.0x`).
- In Tablet mode, single-tap acts as Left Click, two-finger tap acts as Right Click, or toggle **"Drag Lock"** in the bottom dock to hold left-click while moving.

---

## 4. Diagnostic Commands

If you encounter unexpected behavior, run the built-in diagnostic tool on Linux:

```bash
clink doctor
```

This verifies:
- Python 3 and core dependencies
- GTK 3 introspection (`gir1.2-gtk-3.0`) and Cairo bindings
- X11 display server connectivity and XTest extension
- ADB availability and connected USB devices
- WebSocket port (8765) availability
