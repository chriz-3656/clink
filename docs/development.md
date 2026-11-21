# Developer Guide & Monorepo Setup

This guide is for developers and contributors working on the Clink codebase.

---

## Repository Structure

```
clink/
├── android/                   # Native Android application
│   ├── app/src/main/          # Jetpack Compose UI, Canvas, and WebSocket client
│   └── build.gradle.kts       # Gradle build configuration (API 26-35)
├── linux/                     # Linux desktop companion (Python 3)
│   ├── src/clink/             # Overlay, GUI Control Center, X11 input, WebSocket server
│   └── tests/                 # Pytest test suite (26 tests)
├── docs/                      # Technical documentation and guides
├── scripts/                   # Automated build and packaging scripts
│   ├── build-all.sh           # Unified monorepo build script
│   └── package-deb.sh         # Debian package generator
├── packaging/                 # Distribution templates (Debian control, systemd)
└── protocol/                  # Shared WebSocket protocol documentation
```

---

## Environment Prerequisites

- **Linux**: Python 3.10+, PyGObject (`python3-gi`), `gir1.2-gtk-3.0`, `python3-cairo`, `adb`, `python3-pytest`, `python-build`
- **Android**: JDK 21, Android SDK (API 34/35), Gradle 8+

---

## Running Automated Tests

### Linux Automated Tests:
```bash
PYTHONPATH=linux/src:linux/tests python3 -m pytest -v linux/tests
```

To run with full virtual display testing:
```bash
xvfb-run -a python3 -m pytest -v linux/tests
```

### Android Automated Tests:
```bash
cd android
./gradlew testDebugUnitTest
```

---

## Building All Production Artifacts

Run the unified monorepo build script:
```bash
./scripts/build-all.sh
```

This validates the toolchain, runs all Linux and Android test suites, builds the Python wheel, packages the `.deb` distribution, builds the Linux tarball, and compiles both Debug and Release Android APKs into the `dist/` directory.

---

## Generating Android App Bundle (`.aab`) for Play Store

To generate an `.aab` for Google Play Store upload:
```bash
cd android
./gradlew bundleRelease
```
Output: `android/app/build/outputs/bundle/release/app-release.aab`
