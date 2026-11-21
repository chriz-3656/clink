#!/usr/bin/env bash
set -euo pipefail

echo "Uninstalling Clink Linux Companion..."

# 1. Stop any running instance
clink stop || true

# 2. Uninstall Python package
pip uninstall --break-system-packages -y clink || true

# 3. Remove desktop launcher and icons
rm -f "$HOME/.local/share/applications/clink.desktop"
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/clink.svg"

# 4. Remove deb package if installed
if dpkg -s clink >/dev/null 2>&1; then
    sudo dpkg -r clink
fi

echo "Clink has been successfully uninstalled."
