#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
mkdir -p "$DIST_DIR"

echo "=========================================================="
echo "          CLINK MONOREPO UNIFIED BUILD SCRIPT            "
echo "=========================================================="

# 1. Validate dependencies
echo "[1/7] Validating build toolchain..."
command -v python3 >/dev/null 2>&1 || { echo "Error: python3 missing"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo missing"; exit 1; }
command -v java >/dev/null 2>&1 || { echo "Error: java missing"; exit 1; }
if [ -z "${ANDROID_HOME:-}" ] && [ -d "/data/android-sdk" ]; then
    export ANDROID_HOME="/data/android-sdk"
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
fi

VERSION=$(python3 -c "import re; print(re.search(r'__version__\s*=\s*[\"\']([^\"\']+)[\"\']', open('$REPO_ROOT/linux/src/clink/__init__.py').read()).group(1))" 2>/dev/null || echo "1.0.8")
echo "Building Clink version: $VERSION"

# 2. Run Rust and Linux Tests
echo "[2/7] Running Native Rust & Linux Companion test suites..."
cargo test --all
PYTHONPATH="$REPO_ROOT/linux/src:$REPO_ROOT/linux/tests" pytest -v "$REPO_ROOT/linux/tests"

# 3. Build Native Linux GUI Binary
echo "[3/7] Compiling Native Linux Companion GUI binary..."
cargo build --release --bin clink
cp "$REPO_ROOT/target/release/clink" "$DIST_DIR/clink-gui-linux-x86_64"
strip "$DIST_DIR/clink-gui-linux-x86_64" 2>/dev/null || true
cp "$DIST_DIR/clink-gui-linux-x86_64" "$DIST_DIR/clink-native-linux-x86_64"

# 4. Build Native Windows GUI Binary (.exe) and Installer
echo "[4/7] Compiling Native Windows Companion GUI binary (.exe)..."
rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
if command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    cargo build --release --target x86_64-pc-windows-gnu --bin clink
    cp "$REPO_ROOT/target/x86_64-pc-windows-gnu/release/clink.exe" "$DIST_DIR/clink-windows-x86_64.exe"
    x86_64-w64-mingw32-strip "$DIST_DIR/clink-windows-x86_64.exe" 2>/dev/null || true
    
    if command -v makensis >/dev/null 2>&1; then
        echo "Building Windows NSIS Installer..."
        cd "$REPO_ROOT/packaging/windows"
        makensis -DPRODUCT_VERSION="$VERSION" installer.nsi
        cp "$REPO_ROOT/packaging/windows/clink-setup-v${VERSION}-windows-x86_64.exe" "$DIST_DIR/"
        cd "$REPO_ROOT"
    fi
else
    echo "Warning: mingw-w64 not installed; skipping Windows binary cross-compilation."
fi

# 5. Build Debian Package with Native GUI Binary
echo "[5/7] Packaging Debian .deb distribution..."
DEB_ROOT="$DIST_DIR/debian/clink_${VERSION}_amd64"
rm -rf "$DEB_ROOT"
mkdir -p "$DEB_ROOT/DEBIAN" "$DEB_ROOT/usr/bin" \
         "$DEB_ROOT/usr/share/applications" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps"

cp "$REPO_ROOT/packaging/debian/DEBIAN/control" "$DEB_ROOT/DEBIAN/"
sed -i "s/^Version:.*/Version: $VERSION/" "$DEB_ROOT/DEBIAN/control"
cp "$DIST_DIR/clink-gui-linux-x86_64" "$DEB_ROOT/usr/bin/clink"
chmod 755 "$DEB_ROOT/usr/bin/clink"
cp "$REPO_ROOT/packaging/debian/usr/share/applications/clink.desktop" "$DEB_ROOT/usr/share/applications/"
cp "$REPO_ROOT/assets/logo/clink-logo.svg" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/clink.svg"
dpkg-deb --build "$DEB_ROOT" "$DIST_DIR/clink_${VERSION}_amd64.deb"

# 6. Build Android APKs & Run Android Tests
echo "[6/7] Building Android application and running tests..."
cd "$REPO_ROOT/android"
./gradlew testDebugUnitTest assembleDebug assembleRelease

cp "$REPO_ROOT/android/app/build/outputs/apk/debug/app-debug.apk" "$DIST_DIR/clink-android-debug.apk"
cp "$REPO_ROOT/android/app/build/outputs/apk/release/app-release.apk" "$DIST_DIR/clink-android-release.apk"

# 7. Generate Checksums
echo "[7/7] Generating SHA256 checksums..."
cd "$DIST_DIR"
rm -f SHA256SUMS.txt
sha256sum clink* > SHA256SUMS.txt

echo "=========================================================="
echo "                 BUILD COMPLETE: SUMMARY                  "
echo "=========================================================="
ls -lh "$DIST_DIR"
echo "----------------------------------------------------------"
cat "$DIST_DIR/SHA256SUMS.txt"
echo "=========================================================="
