# Privacy Policy for Clink

**Effective Date:** November 21, 2026  
**Project:** Clink (com.chriz.clink)  
**Developer:** chriz-3656  
**Contact:** [chrizmonsaji@proton.me](mailto:chrizmonsaji@proton.me)  
**Repository:** [https://github.com/chriz-3656/clink](https://github.com/chriz-3656/clink)

---

## 1. Overview & Local-First Philosophy

Clink is an open-source Android drawing tablet companion and desktop annotation system for Linux. Clink is designed with a **privacy-first, local-only architecture**. 

Clink **does not collect, harvest, store, transmit, or share any personal user information, telemetry, or analytics**. All communication occurs exclusively over your local USB cable or private local area network (LAN/Wi-Fi/Hotspot) between your Android device and your Linux computer.

---

## 2. Information We Do NOT Collect

- We do NOT collect personal identification information (names, emails, phone numbers, or addresses).
- We do NOT collect location data or GPS coordinates.
- We do NOT use advertising identifiers (AAID) or tracking SDKs.
- We do NOT embed third-party analytics (no Google Analytics, no Firebase Analytics, no telemetry).
- We do NOT transmit your drawings, canvas strokes, or notes to any external server or cloud service.

---

## 3. Permissions Used & Purpose

Clink requests the minimum possible set of permissions required to function:

### Camera (`android.permission.CAMERA`)
- **Purpose**: Used solely to scan the optical QR pairing code displayed on your Linux computer's monitor.
- **Data Handling**: Camera frames are processed in real-time in volatile memory to detect the pairing URI. **No images, video recordings, or photos are ever saved, stored, or transmitted.**

### Local Network & Sockets (`android.permission.INTERNET`, `android.permission.ACCESS_NETWORK_STATE`)
- **Purpose**: Used solely to establish a local WebSocket connection to your Linux companion computer on your local subnet (e.g. `127.0.0.1` for USB ADB reverse or `192.168.x.x` for local Wi-Fi/Hotspot).
- **Data Handling**: No external internet traffic is generated.

---

## 4. Local Data Storage

- **Drawing Sessions**: If you choose to use the "Save Session" feature, your drawing strokes are saved strictly locally in your device's private app storage database. You can delete saved sessions at any time within the application.
- **Preferences**: App preferences (selected tool, stroke width, color, pairing tokens) are stored locally in Android Encrypted/Private SharedPreferences.

---

## 5. Third-Party Services

Clink does not integrate any third-party analytics, crash-reporting SDKs, advertising networks, or tracking services.

---

## 6. Open Source Verification

Because Clink is 100% open-source under the MIT License, anyone can inspect, audit, and verify the source code and build instructions directly on GitHub:  
[https://github.com/chriz-3656/clink](https://github.com/chriz-3656/clink)

---

## 7. Contact Us

If you have questions about this Privacy Policy or Clink's privacy practices, please contact:
- **Maintainer**: chriz-3656
- **Email**: [chrizmonsaji@proton.me](mailto:chrizmonsaji@proton.me)
- **GitHub Issues**: [https://github.com/chriz-3656/clink/issues](https://github.com/chriz-3656/clink/issues)
