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
command -v java >/dev/null 2>&1 || { echo "Error: java missing"; exit 1; }
if [ -z "${ANDROID_HOME:-}" ] && [ -d "/data/android-sdk" ]; then
    export ANDROID_HOME="/data/android-sdk"
    export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
fi

VERSION=$(python3 -c "import re; print(re.search(r'__version__\s*=\s*[\"\']([^\"\']+)[\"\']', open('$REPO_ROOT/linux/src/clink/__init__.py').read()).group(1))" 2>/dev/null || echo "1.0.6")
echo "Building Clink version: $VERSION"

# 2. Run Linux Tests
echo "[2/7] Running Linux Companion automated tests..."
PYTHONPATH="$REPO_ROOT/linux/src:$REPO_ROOT/linux/tests" pytest -v "$REPO_ROOT/linux/tests"

# 3. Build Linux Packages
echo "[3/7] Building Linux distribution packages..."
cd "$REPO_ROOT/linux"
python3 -m build --outdir "$DIST_DIR"

# 4. Build Debian Package
echo "[4/7] Packaging Debian .deb distribution..."
DEB_ROOT="$DIST_DIR/debian/clink_${VERSION}_all"
rm -rf "$DEB_ROOT"
mkdir -p "$DEB_ROOT/DEBIAN" "$DEB_ROOT/usr/bin" "$DEB_ROOT/usr/lib/python3/dist-packages/clink" \
         "$DEB_ROOT/usr/share/applications" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps"

cp "$REPO_ROOT/packaging/debian/DEBIAN/control" "$DEB_ROOT/DEBIAN/"
sed -i "s/^Version:.*/Version: $VERSION/" "$DEB_ROOT/DEBIAN/control"
cp "$REPO_ROOT/packaging/debian/usr/bin/clink" "$DEB_ROOT/usr/bin/"
cp "$REPO_ROOT/packaging/debian/usr/share/applications/clink.desktop" "$DEB_ROOT/usr/share/applications/"
cp "$REPO_ROOT/assets/logo/clink-logo.svg" "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/clink.svg"
cp -r "$REPO_ROOT/linux/src/clink/"* "$DEB_ROOT/usr/lib/python3/dist-packages/clink/"
dpkg-deb --build "$DEB_ROOT" "$DIST_DIR/clink_${VERSION}_all.deb"

# 5. Build Portable Tarball
echo "[5/7] Creating portable Linux tarball..."
tar -czf "$DIST_DIR/clink-linux-${VERSION}-x86_64.tar.gz" -C "$REPO_ROOT/linux" src pyproject.toml requirements.txt

# 6. Build Android APKs & Run Android Tests
echo "[6/7] Building Android application and running tests..."
cd "$REPO_ROOT/android"
./gradlew testDebugUnitTest assembleDebug assembleRelease

cp "$REPO_ROOT/android/app/build/outputs/apk/debug/app-debug.apk" "$DIST_DIR/clink-android-debug.apk"
cp "$REPO_ROOT/android/app/build/outputs/apk/release/app-release.apk" "$DIST_DIR/clink-android-release.apk"

# 7. Generate Checksums
echo "[7/7] Generating SHA256 checksums..."
cd "$DIST_DIR"
sha256sum clink-${VERSION}-py3-none-any.whl clink-${VERSION}.tar.gz clink_${VERSION}_all.deb \
          clink-linux-${VERSION}-x86_64.tar.gz clink-android-debug.apk clink-android-release.apk > SHA256SUMS.txt

echo "=========================================================="
echo "                 BUILD COMPLETE: SUMMARY                  "
echo "=========================================================="
ls -lh "$DIST_DIR"
echo "----------------------------------------------------------"
cat "$DIST_DIR/SHA256SUMS.txt"
echo "=========================================================="
