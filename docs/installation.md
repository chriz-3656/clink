# Clink Installation Guide

This guide details how to install the Clink Linux companion and Android application across supported environments.

---

## Linux Host Installation

Clink supports Debian, Ubuntu, Linux Mint, Pop!_OS, Fedora, Arch Linux, and any distribution running X11.

### Option 1: Debian Package (`.deb`) — Recommended for Ubuntu / Debian / Mint
Download the latest `clink_1.0.0_all.deb` from [Releases](https://github.com/chriz-3656/clink/releases):

```bash
sudo dpkg -i clink_1.0.0_all.deb
sudo apt-get install -f   # Installs PyGObject, Cairo, and GTK 3 dependencies
```

Once installed, you can launch Clink from your system application menu ("Clink") or via terminal:
```bash
clink gui
```

### Option 2: Python Wheel (`.whl`)
```bash
pip install clink-1.0.0-py3-none-any.whl
```
Ensure you have the required GTK 3 system libraries:
```bash
# Ubuntu / Debian
sudo apt install python3-gi gir1.2-gtk-3.0 python3-cairo adb

# Fedora
sudo dnf install python3-gobject gtk3 cairo-devel android-tools
```

### Option 3: Standalone Tarball
```bash
tar -xvf clink-linux-1.0.0-x86_64.tar.gz
cd clink-linux-1.0.0
./bin/clink gui
```

### Option 4: Building from Source
```bash
git clone https://github.com/chriz-3656/clink.git
cd clink
./scripts/build-all.sh
```

---

## Android Application Installation

### Option 1: Direct APK Download
1. Download `clink-android-release.apk` from [GitHub Releases](https://github.com/chriz-3656/clink/releases).
2. Open the downloaded APK on your Android device (ensure "Install unknown apps" is enabled for your browser or file manager).
3. Tap **Install**.

### Option 2: Install via ADB
Connect your phone with USB debugging enabled:
```bash
adb install dist/clink-android-release.apk
```

---

## Verifying Installation

Run the system doctor command on Linux to verify that all hardware drivers, libraries, and communication ports are healthy:

```bash
clink doctor
```
