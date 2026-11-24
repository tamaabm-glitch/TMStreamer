#!/bin/sh
# Signs a TMStreamer package for the updater: sign.sh <file.deb>
# writes <file.deb>.sig. The key is the TMStreamer Ed25519 private key; never commit or copy it to a device
set -e
KEY=${TMSTREAMER_SIGNING_KEY:-$HOME/tmstreamer-signing-key.pem}
# Signed apart from the encoding, as a pipe would hide a failure and
# leave an empty .sig
signature=$(openssl pkeyutl -sign -inkey "$KEY" -rawin -in "$1" | base64 -w0)
[ -n "$signature" ] || {
    echo "Couldn't sign $1 with $KEY" >&2
    rm -f "$1.sig"
    exit 1
}
printf '%s' "$signature" >"$1.sig"
echo "$1.sig"
