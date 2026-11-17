#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

PKG_NAME="omnicalc"
VERSION=$(grep '^version = ' "$SCRIPT_DIR/Cargo.toml" | head -n1 | cut -d'"' -f2)
TARGET_TRIPLE="linux-x86_64"
DIST_DIR="$SCRIPT_DIR/dist"

echo "================================================================================"
echo "Packaging Distribution Binaries for $PKG_NAME v$VERSION ($TARGET_TRIPLE)"
echo "================================================================================"

# 1. Compile Release Binary with maximum LTO optimizations
echo "[1/5] Building release binary with Cargo (lto=fat, codegen-units=1, opt-level=3)..."
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

RELEASE_BIN="$SCRIPT_DIR/target/release/$PKG_NAME"
if [ ! -f "$RELEASE_BIN" ]; then
    echo "Error: Release binary not found at $RELEASE_BIN" >&2
    exit 1
fi

# 2. Reset clean dist output folder
echo "[2/5] Initializing clean distribution directory: $DIST_DIR..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 3. Create Standalone Tarball Archive
echo "[3/5] Creating standalone binary tarball..."
TAR_STAGING="$DIST_DIR/${PKG_NAME}-v${VERSION}-${TARGET_TRIPLE}"
mkdir -p "$TAR_STAGING"

cp "$RELEASE_BIN" "$TAR_STAGING/$PKG_NAME"
strip --strip-all "$TAR_STAGING/$PKG_NAME"
cp "$SCRIPT_DIR/README.md" "$TAR_STAGING/"
cp "$SCRIPT_DIR/LICENSE" "$TAR_STAGING/"

# Create install script for user convenience
cat << 'EOF' > "$TAR_STAGING/install.sh"
#!/bin/sh
set -e
PREFIX="${PREFIX:-/usr/local}"
DEST="$PREFIX/bin"

if [ ! -w "$DEST" ] && [ "$(id -u)" -ne 0 ]; then
    echo "Error: Cannot write to $DEST. Try running with sudo: sudo ./install.sh" >&2
    exit 1
fi

install -d "$DEST"
install -m 755 omnicalc "$DEST/omnicalc"
echo "omnicalc installed successfully to $DEST/omnicalc"
EOF
chmod +x "$TAR_STAGING/install.sh"

TAR_ARCHIVE="${PKG_NAME}-v${VERSION}-${TARGET_TRIPLE}.tar.gz"
tar -czf "$DIST_DIR/$TAR_ARCHIVE" -C "$DIST_DIR" "${PKG_NAME}-v${VERSION}-${TARGET_TRIPLE}"
echo "  -> Created $DIST_DIR/$TAR_ARCHIVE"

# 4. Create Debian Package (.deb)
echo "[4/5] Building Debian package (.deb)..."
DEB_STAGING="$DIST_DIR/deb_staging"
mkdir -p "$DEB_STAGING/DEBIAN"
mkdir -p "$DEB_STAGING/usr/bin"
mkdir -p "$DEB_STAGING/usr/share/doc/$PKG_NAME"

cp "$TAR_STAGING/$PKG_NAME" "$DEB_STAGING/usr/bin/"
cp "$SCRIPT_DIR/LICENSE" "$DEB_STAGING/usr/share/doc/$PKG_NAME/copyright"
cp "$SCRIPT_DIR/README.md" "$DEB_STAGING/usr/share/doc/$PKG_NAME/"

# Calculate installed size in KB
INSTALLED_SIZE=$(du -sk "$DEB_STAGING" | cut -f1)

cat << EOF > "$DEB_STAGING/DEBIAN/control"
Package: $PKG_NAME
Version: $VERSION
Architecture: amd64
Maintainer: Sagnik Rout <https://github.com/sagnikrout>
Installed-Size: $INSTALLED_SIZE
Section: math
Priority: optional
Homepage: https://github.com/sagnikrout/cal
Description: High-performance scientific CLI calculator and CAS engine
 A lightweight, zero-allocation scientific calculator featuring physical
 unit conversions, Gaussian uncertainty propagation, complex numbers,
 transcendental equation solving, and symbolic differentiation.
EOF

DEB_PACKAGE="${PKG_NAME}_${VERSION}_amd64.deb"
dpkg-deb --build --root-owner-group "$DEB_STAGING" "$DIST_DIR/$DEB_PACKAGE"
rm -rf "$DEB_STAGING"
echo "  -> Created $DIST_DIR/$DEB_PACKAGE"

# 5. Generate SHA-256 Checksums
echo "[5/5] Generating SHA256 checksums..."
(
    cd "$DIST_DIR"
    sha256sum "$TAR_ARCHIVE" "$DEB_PACKAGE" > SHA256SUMS
)
cat "$DIST_DIR/SHA256SUMS"

# 6. Deploy fresh stripped binary to /home/sagnik/bin/omnicalc
mkdir -p /home/sagnik/bin
cp "$TAR_STAGING/$PKG_NAME" /home/sagnik/bin/omnicalc

echo "================================================================================"
echo "Packaging Complete! Distribution artifacts ready in: $DIST_DIR"
ls -lh "$DIST_DIR"
echo "================================================================================"
