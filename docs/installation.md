# Installation Guide

## Linux Host Installation

### Option 1: Debian Package (.deb)
```bash
sudo dpkg -i clink_1.0.0_all.deb
sudo apt-get install -f  # resolve any missing runtime dependencies
```

### Option 2: Python Wheel
```bash
pip install clink-1.0.0-py3-none-any.whl
```

### Option 3: Source Installation
```bash
git clone https://github.com/chriz-3656/clink.git
cd clink
./scripts/install-linux.sh
```

## Android Application Installation
Install `clink-android-release.apk` (or `clink-android-debug.apk`) via adb or direct download:
```bash
adb install dist/clink-android-release.apk
```
