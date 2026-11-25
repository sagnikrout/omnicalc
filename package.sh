#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKG_NAME="omnicalc"
VERSION="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$SCRIPT_DIR/Cargo.toml" | head -n1)"
TARGET_TRIPLE="linux-x86_64"
DIST_DIR="$SCRIPT_DIR/dist"

if [[ -z "$VERSION" ]]; then
    echo "Error: could not read package version from Cargo.toml" >&2
    exit 1
fi
if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
    echo "Error: this packaging script currently supports Linux x86_64 only" >&2
    exit 1
fi
for tool in cargo strip tar dpkg-deb sha256sum gzip; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "Error: required tool '$tool' is not installed" >&2
        exit 1
    }
done

# Build artifacts in a private temporary directory. Never delete or overwrite
# unrelated files in dist, and do not install binaries into a developer's home.
STAGING_DIR="$(mktemp -d "$SCRIPT_DIR/.omnicalc-package.XXXXXX")"
trap 'rm -rf -- "$STAGING_DIR"' EXIT

cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"
RELEASE_BIN="$SCRIPT_DIR/target/release/$PKG_NAME"
if [[ ! -f "$RELEASE_BIN" ]]; then
    echo "Error: release binary not found at $RELEASE_BIN" >&2
    exit 1
fi

TAR_ROOT="$STAGING_DIR/${PKG_NAME}-v${VERSION}-${TARGET_TRIPLE}"
mkdir -p "$TAR_ROOT"
install -m 755 "$RELEASE_BIN" "$TAR_ROOT/$PKG_NAME"
strip --strip-all "$TAR_ROOT/$PKG_NAME"
cp "$SCRIPT_DIR/README.md" "$SCRIPT_DIR/LICENSE" "$TAR_ROOT/"
if [[ -f "$SCRIPT_DIR/dist/man/omnicalc.1" ]]; then
    cp "$SCRIPT_DIR/dist/man/omnicalc.1" "$TAR_ROOT/"
fi

cat > "$TAR_ROOT/install.sh" <<'INSTALL_EOF'
#!/bin/sh
set -eu
PREFIX="${PREFIX:-/usr/local}"
DEST="$PREFIX/bin"

if [ ! -w "$PREFIX" ] && [ "$(id -u)" -ne 0 ]; then
    echo "Error: cannot install to $DEST; choose a writable PREFIX or use sudo" >&2
    exit 1
fi
install -d "$DEST"
install -m 755 "$(dirname "$0")/omnicalc" "$DEST/omnicalc"
echo "omnicalc installed successfully to $DEST/omnicalc"
INSTALL_EOF
chmod 755 "$TAR_ROOT/install.sh"

TAR_ARCHIVE="${PKG_NAME}-v${VERSION}-${TARGET_TRIPLE}.tar.gz"
tar -czf "$STAGING_DIR/$TAR_ARCHIVE" -C "$STAGING_DIR" "${PKG_NAME}-v${VERSION}-${TARGET_TRIPLE}"

DEB_ROOT="$STAGING_DIR/deb"
mkdir -p "$DEB_ROOT/DEBIAN" "$DEB_ROOT/usr/bin" "$DEB_ROOT/usr/share/doc/$PKG_NAME" "$DEB_ROOT/usr/share/man/man1"
install -m 755 "$TAR_ROOT/$PKG_NAME" "$DEB_ROOT/usr/bin/$PKG_NAME"
cp "$SCRIPT_DIR/LICENSE" "$DEB_ROOT/usr/share/doc/$PKG_NAME/copyright"
cp "$SCRIPT_DIR/README.md" "$DEB_ROOT/usr/share/doc/$PKG_NAME/"
if [[ -f "$SCRIPT_DIR/dist/man/omnicalc.1" ]]; then
    gzip -c -n "$SCRIPT_DIR/dist/man/omnicalc.1" > "$DEB_ROOT/usr/share/man/man1/omnicalc.1.gz"
    chmod 644 "$DEB_ROOT/usr/share/man/man1/omnicalc.1.gz"
fi
INSTALLED_SIZE="$(du -sk "$DEB_ROOT" | cut -f1)"
cat > "$DEB_ROOT/DEBIAN/control" <<CONTROL_EOF
Package: $PKG_NAME
Version: $VERSION
Architecture: amd64
Maintainer: Sagnik Rout <https://github.com/sagnikrout>
Installed-Size: $INSTALLED_SIZE
Section: math
Priority: optional
Homepage: https://github.com/sagnikrout/omnicalc
Description: High-performance scientific CLI calculator and CAS engine
 A lightweight scientific calculator featuring physical unit conversions,
 uncertainty propagation, complex numbers, equation solving, and calculus.
CONTROL_EOF

DEB_PACKAGE="${PKG_NAME}_${VERSION}_amd64.deb"
dpkg-deb --build --root-owner-group "$DEB_ROOT" "$STAGING_DIR/$DEB_PACKAGE"
(
    cd "$STAGING_DIR"
    sha256sum "$TAR_ARCHIVE" "$DEB_PACKAGE" > SHA256SUMS
)

# Publish only after every artifact has been built successfully. Existing
# dist contents are preserved; only artifacts with these exact names change.
mkdir -p "$DIST_DIR"
install -m 644 "$STAGING_DIR/$TAR_ARCHIVE" "$DIST_DIR/$TAR_ARCHIVE"
install -m 644 "$STAGING_DIR/$DEB_PACKAGE" "$DIST_DIR/$DEB_PACKAGE"
install -m 644 "$STAGING_DIR/SHA256SUMS" "$DIST_DIR/SHA256SUMS"
echo "Packaging complete: $DIST_DIR"
ls -lh "$DIST_DIR/$TAR_ARCHIVE" "$DIST_DIR/$DEB_PACKAGE" "$DIST_DIR/SHA256SUMS"
