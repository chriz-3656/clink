# Google Play Store Publishing: Requirements & Research Guide

This document outlines the complete technical, legal, and operational requirements for publishing the **Clink** Android application to the **Google Play Store**, based on Google Play's current developer policies and requirements (2024–2026).

---

## 1. Executive Summary & Viability

> **Can Clink be published on the Google Play Store?**  
> **Yes, 100%!** Clink satisfies all foundational technical requirements:
> - Targets **Android 15 (API level 35)** (surpassing Google's minimum target API requirement).
> - Collects **zero personal data**, has no ads, and contains zero tracking SDKs.
> - Uses only documented Android Jetpack Compose and CameraX/ZXing APIs.
> - Requires only standard runtime permissions (`CAMERA` for QR scanning and `INTERNET` / `ACCESS_NETWORK_STATE` for local WebSockets).

---

## 2. Developer Account Setup & Prerequisites

| Requirement | Details | Cost |
| :--- | :--- | :--- |
| **Google Play Console Account** | One-time registration fee on [play.google.com/console](https://play.google.com/console). | **$25 USD** (one-time) |
| **Identity Verification** | Government-issued photo ID (Passport, Driver's License) and address verification (utility bill/bank statement). | Free |
| **Account Type** | **Personal/Individual** vs. **Organization**. *(Note: Organization accounts require a D-U-N-S number from Dun & Bradstreet).* | Free |

---

## 3. The 12-Tester / 14-Day Closed Testing Rule (Critical)

For any **Personal developer account** registered after November 13, 2023, Google enforces a mandatory testing gate:

### The Rule:
- You must run a **Closed Testing track** with a minimum of **12 testers**.
- These testers must remain opted-in and continuously installed for **at least 14 consecutive days**.
- Once 14 days of active closed testing pass, the Play Console unlocks the **"Apply for Production Access"** button.

### How to Fulfill It:
1. Upload your signed Android App Bundle (`.aab`) to the **Closed testing (Alpha)** track in the Play Console.
2. Create an email list of at least 15–20 friends, family, or open-source community members (to maintain a safety buffer above the 12-tester minimum).
3. Send testers the web opt-in link (`https://play.google.com/apps/testing/com.chriz.clink`).
4. Testers install the app from Google Play on their Android devices and keep it installed for 14 days.
5. Submit the production readiness questionnaire on the 15th day.

> [!NOTE]
> **Organization accounts** with a verified D-U-N-S number are **exempt** from the 12-tester requirement and can publish to Production immediately.

---

## 4. Technical Build Requirements

### 1. Android App Bundle (`.aab`) Format
Google Play requires `.aab` bundles rather than `.apk` files for all new apps.  
To generate the release `.aab` for Clink:

```bash
cd android
./gradlew bundleRelease
```
The resulting bundle is generated at:
`android/app/build/outputs/bundle/release/app-release.aab`

### 2. Release Keystore Signing
For production, create a secure release keystore (do not commit this keystore or passwords to Git!):

```bash
keytool -genkey -v -keystore clink-release-key.jks \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -alias clink -storetype JKS
```

In `android/app/build.gradle.kts`, configure release signing:
```kotlin
signingConfigs {
    create("release") {
        storeFile = file(System.getenv("KEYSTORE_PATH") ?: "clink-release-key.jks")
        storePassword = System.getenv("KEYSTORE_PASSWORD")
        keyAlias = System.getenv("KEY_ALIAS") ?: "clink"
        keyPassword = System.getenv("KEY_PASSWORD")
    }
}
```

### 3. API Level Compliance
- **Target SDK**: Clink targets **API 35 (Android 15)**, which is 100% compliant with Google Play's annual target API requirement (minimum API 34 required through 2024, API 35 for 2025+).
- **Minimum SDK**: Clink's `minSdk` is **26 (Android 8.0 Oreo)**, ensuring broad compatibility with over 95% of active Android devices worldwide.

---

## 5. Google Play Store Listing & Policy Assets

### Store Metadata:
- **App Title**: `Clink - Drawing Tablet & Overlay` (Max 30 characters)
- **Short Description**: `Turn your Android into a desktop drawing tablet and real-time Linux annotation canvas.` (Max 80 characters)
- **Full Description**: Detailed explanation of dual modes (Annotation & Tablet), zero-lag USB ADB, Wi-Fi pairing, QR scanning, and privacy-first local operation.

### Required Graphics:
1. **App Icon**: `512 x 512 px` (32-bit PNG with alpha channel).
2. **Feature Graphic**: `1024 x 500 px` (JPG or PNG, 24-bit, no alpha).
3. **Phone Screenshots**: At least 4 screenshots (Minimum resolution: 1080 x 1920 or 1080 x 2400) showing:
   - Drawing canvas with active tools.
   - Home connection screen (USB / Wi-Fi / Hotspot).
   - Camera QR scanner pairing.
   - Settings & custom duration timers.
4. **7-inch & 10-inch Tablet Screenshots**: Recommended for drawing tablet apps.

---

## 6. Mandatory Policy Declarations

### 1. Hosted Privacy Policy URL
Google Play requires a publicly accessible HTTPS URL. Because Clink is open-source, this can be hosted directly on GitHub Pages or within the repository:
- **URL**: `https://github.com/chriz-3656/clink/blob/main/PRIVACY_POLICY.md`
- **Content**: Explicit statement that Clink is a 100% offline, local-first application that collects zero personal data, transmits no analytics, and uses the camera exclusively for scanning local pairing QR codes.

### 2. Data Safety Form
When completing the Google Play Data Safety questionnaire:
- **Does your app collect or share user data?** `No`.
- **Are all user data transfers encrypted in transit?** `Yes` (Local LAN pairing with tokens and SHA-256 PIN authentication).
- **Can users request data deletion?** `Yes` (All drawing sessions are saved locally on-device and can be deleted by the user anytime).

### 3. Permissions Justification
- `android.permission.CAMERA`: Used exclusively for the integrated optical QR code scanner to pair with the desktop monitor. No images or video are stored, uploaded, or transmitted.
- `android.permission.INTERNET`: Used exclusively for local socket communication (`ws://127.0.0.1` and `ws://192.168.x.x`).

---

## 7. Open-Source Alternatives & Ecosystem Distribution

In addition to or in parallel with Google Play, open-source Android apps commonly distribute through:

1. **F-Droid (Official Open Source Repository)**:
   - Completely free, no registration fee.
   - High reputation among Linux and privacy-focused users.
   - Builds directly from source code on GitHub.
   - Requires no proprietary Google Play Services.
2. **IzzyOnDroid (F-Droid Repository)**:
   - Indexes release APKs directly from GitHub Releases.
   - Instant publication without waiting for F-Droid's build cycle.
3. **GitHub Releases & Direct APK**:
   - Available immediately (already published under [v1.0.5](https://github.com/chriz-3656/clink/releases)).
