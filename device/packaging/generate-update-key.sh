#!/bin/sh
# Generate a TMStreamer Ed25519 release key pair.
# Private key stays on the release operator's workstation/offline signer.
# Public key is the 32-byte raw Ed25519 key encoded as base64 for update.rs.
set -eu
umask 077

PRIVATE_KEY=${TMSTREAMER_SIGNING_KEY:-"$HOME/.config/tmstreamer/release-signing-ed25519.pem"}
PUBLIC_KEY_OUT=${TMSTREAMER_PUBLIC_KEY_OUT:-"./tmstreamer-update-key.pub"}

for tool in openssl base64 tail tr wc chmod mkdir dirname mktemp mv rm; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "$tool is required" >&2
        exit 1
    }
done

if [ "$PRIVATE_KEY" = "$PUBLIC_KEY_OUT" ]; then
    echo "Private and public key paths must be different." >&2
    exit 1
fi
if [ -e "$PRIVATE_KEY" ] || [ -L "$PRIVATE_KEY" ] || [ -e "$PUBLIC_KEY_OUT" ] || [ -L "$PUBLIC_KEY_OUT" ]; then
    echo "Refusing to overwrite an existing key file." >&2
    echo "Choose new output paths or back up and remove the existing files deliberately." >&2
    exit 1
fi

mkdir -p "$(dirname "$PRIVATE_KEY")" "$(dirname "$PUBLIC_KEY_OUT")"
DER=$(mktemp)
RAW=$(mktemp)
ENCODED=$(mktemp)
PUBLIC_TMP=$(mktemp "$(dirname "$PUBLIC_KEY_OUT")/.tmstreamer-public.XXXXXX")
PRIVATE_CREATED=0
SUCCESS=0
cleanup() {
    rm -f "$DER" "$RAW" "$ENCODED" "$PUBLIC_TMP"
    if [ "$SUCCESS" -ne 1 ] && [ "$PRIVATE_CREATED" -eq 1 ]; then
        rm -f "$PRIVATE_KEY"
    fi
}
trap cleanup EXIT HUP INT TERM

PRIVATE_CREATED=1
openssl genpkey -algorithm ED25519 -out "$PRIVATE_KEY"
chmod 600 "$PRIVATE_KEY"
openssl pkey -in "$PRIVATE_KEY" -pubout -outform DER -out "$DER"
tail -c 32 "$DER" > "$RAW"
[ "$(wc -c < "$RAW" | tr -d ' ')" = "32" ] || {
    echo "OpenSSL did not produce a 32-byte Ed25519 public key." >&2
    exit 1
}
base64 < "$RAW" > "$ENCODED"
tr -d '\n' < "$ENCODED" > "$PUBLIC_TMP"
printf '\n' >> "$PUBLIC_TMP"
[ -s "$PUBLIC_TMP" ] || {
    echo "Public key output is empty." >&2
    exit 1
}
chmod 644 "$PUBLIC_TMP"
mv -f "$PUBLIC_TMP" "$PUBLIC_KEY_OUT"
SUCCESS=1

echo "Private key: $PRIVATE_KEY"
echo "Public key:  $PUBLIC_KEY_OUT"
echo "IMPORTANT: Keep the private key off GitHub and off every TMStreamer device."
