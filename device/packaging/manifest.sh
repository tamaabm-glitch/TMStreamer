#!/bin/sh
# Writes the signed-update manifest: manifest.sh <file.deb> <release-base-url>
# The package .sig must already exist and is embedded in the manifest.
set -eu

if [ "$#" -ne 2 ] || [ ! -f "$1" ] || [ ! -f "$1.sig" ]; then
    echo "Usage: manifest.sh <existing-file.deb> <official-release-base-url>" >&2
    exit 2
fi

DEB=$1
BASE=${2%/}
CHANGELOG=$(dirname "$0")/../../CHANGELOG.md
PACKAGE_NAME=$(dpkg-deb -f "$DEB" Package)
VERSION=$(dpkg-deb -f "$DEB" Version)
ARCH=$(dpkg-deb -f "$DEB" Architecture)
PACKAGE_VERSION=${VERSION%-*}
FILENAME=$(basename "$DEB")
EXPECTED_BASE="https://github.com/tamaabm-glitch/TMStreamer/releases/download/v$PACKAGE_VERSION"
EXPECTED_FILENAME="tmstreamer_${VERSION}_${ARCH}.deb"
OUT=$(dirname "$DEB")/manifest.json
TMP="$OUT.tmp"

if [ "$PACKAGE_NAME" != "tmstreamer" ]; then
    echo "Debian package must be named tmstreamer, got: $PACKAGE_NAME" >&2
    exit 1
fi
if [ "$ARCH" != "arm64" ]; then
    echo "Only ARM64 packages can be published by this manifest generator." >&2
    exit 1
fi
if [ "$FILENAME" != "$EXPECTED_FILENAME" ]; then
    echo "Package filename must exactly match: $EXPECTED_FILENAME" >&2
    exit 1
fi

case "$PACKAGE_VERSION" in
    ''|*[!A-Za-z0-9._+-]*|*..*)
        echo "Invalid Debian package version: $PACKAGE_VERSION" >&2
        exit 1
        ;;
esac
if [ "$BASE" != "$EXPECTED_BASE" ]; then
    echo "Release base URL must exactly match: $EXPECTED_BASE" >&2
    exit 1
fi
trap 'rm -f "$TMP"' EXIT HUP INT TERM
python3 - "$PACKAGE_VERSION" "$ARCH" "$BASE/$FILENAME" "$(cat "$DEB.sig")" "$CHANGELOG" > "$TMP" <<'PY'
import base64
import json
import re
import sys

version, arch, url, signature, changelog = sys.argv[1:]
try:
    decoded_signature = base64.b64decode(signature, validate=True)
except Exception:
    sys.exit("Package signature is not valid base64")
if len(decoded_signature) != 64:
    sys.exit("Package signature must be a 64-byte Ed25519 signature")

notes = ""
with open(changelog, encoding="utf-8") as source:
    sections = re.split(r"^## ", source.read(), flags=re.M)[1:]
for section in sections:
    heading, _, body = section.partition("\n")
    if heading.split() and heading.split()[0].strip("[]") == version:
        notes = body.strip()
        break
if not notes:
    sys.exit(f"CHANGELOG.md has no section for {version}")

json.dump(
    {
        "version": version,
        "notes": notes,
        "packages": {
            arch: {
                "url": url,
                "signature": signature,
            }
        },
    },
    sys.stdout,
    indent=2,
)
sys.stdout.write("\n")
PY
mv "$TMP" "$OUT"
echo "$OUT"
