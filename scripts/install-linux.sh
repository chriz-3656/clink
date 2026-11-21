#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "   Installing Clink Linux Companion     "
echo "========================================="

# 1. Install system dependencies if apt is available
if command -v apt-get >/dev/null 2>&1; then
    echo "Installing system packages via apt..."
    sudo apt-get update -qq
    sudo apt-get install -y --no-install-recommends \
        python3 python3-pip python3-websockets python3-cairo \
        python3-xlib adb libgtk-3-0
fi

# 2. Install Python package
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Installing Clink Python module..."
pip install --break-system-packages -e "$REPO_ROOT/linux" --no-deps

# 3. Install desktop launcher and icon
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
mkdir -p "$DESKTOP_DIR" "$ICON_DIR"

cp "$REPO_ROOT/assets/logo/clink-logo.svg" "$ICON_DIR/clink.svg"
cat << 'DESK' > "$DESKTOP_DIR/clink.desktop"
[Desktop Entry]
Name=Clink
Comment=Android-powered desktop annotation and drawing tablet
Exec=clink gui
Icon=clink
Terminal=false
Type=Application
Categories=Graphics;Utility;
Keywords=tablet;stylus;annotation;whiteboard;
DESK

echo "Clink installation complete!"
echo "Run 'clink doctor' to verify setup."
echo "Run 'clink start' to launch the companion and overlay."
