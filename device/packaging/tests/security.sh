#!/bin/sh
# End-to-end tests for the offline Ed25519 key, package and manifest scripts.
set -eu

PACKAGING=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM

export HOME="$TMP/home"
mkdir -p "$HOME"
export TMSTREAMER_SIGNING_KEY="$HOME/.config/tmstreamer/release-signing-ed25519.pem"
export TMSTREAMER_PUBLIC_KEY_OUT="$TMP/tmstreamer-update-key.pub"

sh "$PACKAGING/generate-update-key.sh"
[ "$(stat -c '%a' "$TMSTREAMER_SIGNING_KEY")" = "600" ]
[ "$(stat -c '%a' "$TMSTREAMER_PUBLIC_KEY_OUT")" = "644" ]
base64 -d < "$TMSTREAMER_PUBLIC_KEY_OUT" > "$TMP/public.raw"
[ "$(wc -c < "$TMP/public.raw" | tr -d ' ')" = "32" ]

if sh "$PACKAGING/generate-update-key.sh"; then
    echo "Key generator overwrote an existing key pair." >&2
    exit 1
fi

make_deb() {
    package=$1
    arch=$2
    output=$3
    dir="$TMP/pkg-$package-$arch"
    mkdir -p "$dir/DEBIAN"
    cat > "$dir/DEBIAN/control" <<EOF
Package: $package
Version: 0.1.2-1
Architecture: $arch
Maintainer: TMStreamer Project <noreply@tmstreamer.invalid>
Description: Temporary package for TMStreamer signing tests
EOF
    dpkg-deb --build "$dir" "$output" >/dev/null
}

DEB="$TMP/tmstreamer_0.1.2-1_arm64.deb"
make_deb tmstreamer arm64 "$DEB"
sh "$PACKAGING/sign.sh" "$DEB" >/dev/null

openssl pkey -in "$TMSTREAMER_SIGNING_KEY" -pubout -out "$TMP/public.pem"
base64 -d < "$DEB.sig" > "$TMP/package.sig"
openssl pkeyutl -verify -pubin -inkey "$TMP/public.pem" -rawin \
    -in "$DEB" -sigfile "$TMP/package.sig" >/dev/null

BASE="https://github.com/tamaabm-glitch/TMStreamer/releases/download/v0.1.2"
sh "$PACKAGING/manifest.sh" "$DEB" "$BASE" >/dev/null
MANIFEST="$TMP/manifest.json"
cp "$TMP/manifest.json" "$MANIFEST"
sh "$PACKAGING/sign.sh" "$MANIFEST" >/dev/null
base64 -d < "$MANIFEST.sig" > "$TMP/manifest.sig"
openssl pkeyutl -verify -pubin -inkey "$TMP/public.pem" -rawin \
    -in "$MANIFEST" -sigfile "$TMP/manifest.sig" >/dev/null

cp "$MANIFEST" "$TMP/manifest-original.json"
printf ' ' >> "$MANIFEST"
if openssl pkeyutl -verify -pubin -inkey "$TMP/public.pem" -rawin \
    -in "$MANIFEST" -sigfile "$TMP/manifest.sig" >/dev/null 2>&1; then
    echo "A modified manifest passed signature verification." >&2
    exit 1
fi

if sh "$PACKAGING/manifest.sh" "$DEB" "https://example.com/releases/v0.1.2" >/dev/null 2>&1; then
    echo "Manifest generator accepted a non-official release origin." >&2
    exit 1
fi

WRONG_NAME="$TMP/not-tmstreamer_0.1.2-1_arm64.deb"
make_deb not-tmstreamer arm64 "$WRONG_NAME"
cp "$DEB.sig" "$WRONG_NAME.sig"
if sh "$PACKAGING/manifest.sh" "$WRONG_NAME" "$BASE" >/dev/null 2>&1; then
    echo "Manifest generator accepted the wrong Debian package name." >&2
    exit 1
fi

WRONG_ARCH="$TMP/tmstreamer_0.1.2-1_amd64.deb"
make_deb tmstreamer amd64 "$WRONG_ARCH"
cp "$DEB.sig" "$WRONG_ARCH.sig"
if sh "$PACKAGING/manifest.sh" "$WRONG_ARCH" "$BASE" >/dev/null 2>&1; then
    echo "Manifest generator accepted a non-ARM64 package." >&2
    exit 1
fi

echo "PASS: offline key generation, permissions, no-overwrite, package signature, manifest signature, tamper rejection, origin, package identity and ARM64 validation"
