#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
APK_PATH="$DIST_DIR/clink-android-debug.apk"
GUI_BIN="$DIST_DIR/clink-gui-linux-x86_64"

echo "=========================================================="
echo "          CLINK LOCAL TEST & DEBUGGING SETUP             "
echo "=========================================================="

# Check binaries
if [ ! -f "$GUI_BIN" ]; then
    echo "Error: Native Linux binary not found at $GUI_BIN"
    exit 1
fi

if [ ! -f "$APK_PATH" ]; then
    echo "Error: Android debug APK not found at $APK_PATH"
    exit 1
fi

chmod +x "$GUI_BIN"

# Check ADB
if ! command -v adb >/dev/null 2>&1; then
    echo "Error: adb command not found."
    exit 1
fi

echo "[1/4] Checking for connected Android devices via ADB..."
DEVICES=$(adb devices | grep -v "List of devices attached" | grep -v "^$" || true)

if [ -z "$DEVICES" ]; then
    echo "⚠️  No Android device detected over USB."
    echo "   Please:"
    echo "   1. Plug in your Android device via USB."
    echo "   2. Ensure Developer Options -> USB Debugging is ON."
    echo "   3. Accept the 'Allow USB debugging' prompt on your phone screen."
    echo ""
    echo "   Alternatively, you can test via Wi-Fi:"
    echo "   - Host IP: $(ip -4 addr show wlan0 2>/dev/null | grep -oP '(?<=inet\s)\d+(\.\d+){3}' || echo '127.0.0.1')"
    echo "   - Port: 8765"
else
    echo "✅ Found device(s):"
    echo "$DEVICES"
    echo ""
    echo "[2/4] Installing debug APK onto connected device..."
    adb install -r -d "$APK_PATH"
    
    echo "[3/4] Configuring ADB reverse port forwarding (tcp:8765 -> tcp:8765)..."
    adb reverse tcp:8765 tcp:8765
    echo "✅ Port forwarding active: tcp:8765 <-> tcp:8765"
fi

echo ""
echo "[4/4] Starting Clink Desktop Companion GUI..."
echo "To monitor Android app logs simultaneously in another terminal, run:"
echo "   adb logcat -s ClinkWebSocket DrawingCanvas:V AndroidRuntime:E"
echo ""
exec "$GUI_BIN" "$@"
