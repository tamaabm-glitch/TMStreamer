#!/bin/sh
# Generate a TMStreamer Ed25519 release key pair.
# Private key stays on the release operator's workstation/offline signer.
# Public key is the 32-byte raw Ed25519 key encoded as base64 for update.rs.
set -eu
umask 077

PRIVATE_KEY=${TMSTREAMER_SIGNING_KEY:-"$HOME/.config/tmstreamer/release-signing-ed25519.pem"}
PUBLIC_KEY_OUT=${TMSTREAMER_PUBLIC_KEY_OUT:-"./tmstreamer-update-key.pub"}

if ! command -v openssl >/dev/null 2>&1; then
    echo "openssl is required" >&2
    exit 1
fi
if ! command -v base64 >/dev/null 2>&1 || ! command -v tail >/dev/null 2>&1; then
    echo "base64 and tail are required" >&2
    exit 1
fi
if [ -e "$PRIVATE_KEY" ] || [ -e "$PUBLIC_KEY_OUT" ]; then
    echo "Refusing to overwrite an existing key file." >&2
    echo "Choose new output paths or back up and remove the existing files deliberately." >&2
    exit 1
fi

mkdir -p "$(dirname "$PRIVATE_KEY")" "$(dirname "$PUBLIC_KEY_OUT")"
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT HUP INT TERM

openssl genpkey -algorithm ED25519 -out "$PRIVATE_KEY"
chmod 600 "$PRIVATE_KEY"
openssl pkey -in "$PRIVATE_KEY" -pubout -outform DER -out "$tmp"
tail -c 32 "$tmp" | base64 | tr -d '\n' > "$PUBLIC_KEY_OUT"
printf '\n' >> "$PUBLIC_KEY_OUT"
chmod 644 "$PUBLIC_KEY_OUT"

echo "Private key: $PRIVATE_KEY"
echo "Public key:  $PUBLIC_KEY_OUT"
echo "IMPORTANT: Keep the private key off GitHub and off every TMStreamer device."
