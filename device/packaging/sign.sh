#!/bin/sh
# Signs a TMStreamer package or manifest: sign.sh <file>
# Writes <file>.sig as base64 Ed25519. The private key must stay offline.
set -eu

if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then
    echo "Usage: sign.sh <existing-file>" >&2
    exit 2
fi

KEY=${TMSTREAMER_SIGNING_KEY:-"$HOME/.config/tmstreamer/release-signing-ed25519.pem"}
if [ ! -r "$KEY" ]; then
    echo "Signing key is not readable: $KEY" >&2
    exit 1
fi

OUTPUT="$1.sig"
RAW=$(mktemp)
ENCODED=$(mktemp)
RESULT=$(mktemp "${OUTPUT}.tmp.XXXXXX")
cleanup() {
    rm -f "$RAW" "$ENCODED" "$RESULT"
}
trap cleanup EXIT HUP INT TERM

# Keep every command separate: POSIX sh pipelines can hide openssl/base64 errors.
if ! openssl pkeyutl -sign -inkey "$KEY" -rawin -in "$1" -out "$RAW"; then
    echo "Ed25519 signing failed" >&2
    exit 1
fi
if [ ! -s "$RAW" ]; then
    echo "OpenSSL produced an empty signature" >&2
    exit 1
fi
if ! base64 < "$RAW" > "$ENCODED"; then
    echo "Base64 encoding failed" >&2
    exit 1
fi
if ! tr -d '\n' < "$ENCODED" > "$RESULT"; then
    echo "Could not prepare signature output" >&2
    exit 1
fi
if [ ! -s "$RESULT" ]; then
    echo "Signature output is empty" >&2
    exit 1
fi
mv -f "$RESULT" "$OUTPUT"
echo "$OUTPUT"
