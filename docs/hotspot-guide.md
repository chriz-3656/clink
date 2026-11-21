# Portable Hotspot Connection Guide (Zero Router, Zero Internet)

Clink is designed to operate completely offline without requiring any home Wi-Fi router, external network infrastructure, USB cables, or internet access.

By creating a direct wireless connection between your Android phone and Linux computer using **Portable Hotspot**, you achieve:
- **Ultra-low latency (< 3ms)** via direct antenna-to-antenna wireless transmission.
- **Complete portability**: Draw outdoors, in classrooms, on flights, on trains, or during internet outages.
- **Bypasses AP Isolation**: Public Wi-Fi networks (schools, cafes, hotels) block devices from communicating with each other; a personal hotspot creates your own private local network.

---

## Method 1: Android Phone Hotspot (Recommended)

```
┌──────────────────┐           Direct Wi-Fi Signal           ┌──────────────────┐
│   Android Phone  │ ◄─────────────────────────────────────► │     Linux PC     │
│ (Hotspot Active) │         Antenna-to-Antenna (< 3ms)      │ (Joined Hotspot) │
└──────────────────┘          (No Internet Needed)           └──────────────────┘
```

### Steps:
1. **Enable Hotspot on Android**:
   - Open **Settings → Portable Hotspot** (or *Personal Hotspot / Tethering*).
   - Turn the hotspot **ON**.
   - Note your Hotspot Wi-Fi Name (SSID) and Password.
   - > [!TIP]
     > **Mobile data is NOT required.** You can turn Mobile Data / Cellular Data completely OFF. The hotspot creates an internal local Wi-Fi network regardless of whether your phone has mobile internet.

2. **Connect Linux PC to Phone Hotspot**:
   - Open your Linux desktop's network menu (system tray or GNOME Settings → Wi-Fi).
   - Select your phone's hotspot Wi-Fi name and enter the password.
   - Your PC will automatically be assigned a local IP address (typically `192.168.43.x`).

3. **Open Clink & Connect**:
   - Launch the Clink Linux companion:
     ```bash
     clink gui
     # or simply launch Clink from your application menu
     ```
   - The Clink Control Center will automatically detect the hotspot IP and display a pairing QR code.
   - Open **Clink** on your Android phone, tap the **QR Scanner icon** at the top right, and scan the QR code displayed on your PC screen.
   - You are now connected directly over the local wireless hotspot!

---

## Method 2: Linux PC Hotspot (PC as Access Point)

Most modern Linux laptops and desktop Wi-Fi cards support operating as a software access point (AP mode):

### Steps:
1. **Enable Hotspot in Linux**:
   - Open **Settings → Wi-Fi**.
   - Click the options menu (three dots `⋮`) and select **"Turn On Wi-Fi Hotspot..."**.
   - Linux will generate a local network name and password on screen.
2. **Connect Phone**:
   - Open your Android phone's Wi-Fi settings and connect to the PC's hotspot.
3. **Scan QR Code**:
   - The Clink Control Center will detect the hotspot gateway (typically `10.42.0.1`).
   - Tap the QR Scanner icon in the Android app, scan the screen, and start drawing!

---

## Dynamic IP Auto-Detection

The Clink Linux companion includes automatic network monitoring:
- If you start Clink *before* connecting to the hotspot, Clink will automatically detect when the hotspot connection becomes active and update its listening IP address within 3 seconds.
- You can also click the **"Refresh Network IP"** button in the Control Center at any time to immediately recalculate the active subnet and refresh the pairing QR code.

---

## Troubleshooting Hotspot Connections

| Issue | Cause | Solution |
| :--- | :--- | :--- |
| QR Code shows `127.0.0.1` | Clink was started before the PC joined the hotspot | Click **Refresh Network IP** in the Control Center. |
| Connection times out | Linux firewall (`ufw` / `firewalld`) is blocking incoming port 8765 | Allow port 8765 on Linux: `sudo ufw allow 8765/tcp`. |
| Phone prompts "No internet access" | Phone recognizes hotspot has no cellular data | Tap the notification and choose **"Stay connected"** / **"Use this network anyway"**. |
