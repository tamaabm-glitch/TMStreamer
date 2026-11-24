# TMStreamer signed update configuration

The updater is deliberately closed in ordinary development and CI builds.
It does not trust the former YStreamer signing key or update server.

## 1. Create the TMStreamer release key

Run `device/packaging/generate-update-key.sh` on the release operator's
trusted workstation (preferably offline). The default outputs are:

- Private Ed25519 PEM: `~/.config/tmstreamer/release-signing-ed25519.pem`
- Public key: `./tmstreamer-update-key.pub` (base64 of the raw 32-byte key)

Back up the private key securely. Never commit it, put it in CI artifacts,
copy it to a Radxa device, or send it to the device.

## 2. Build with the public key

The backend embeds only the public key at compile time. In the release build
environment, set:

```sh
export TMSTREAMER_UPDATE_PUBLIC_KEY="$(cat tmstreamer-update-key.pub)"
```

Then build the Debian package using the normal TMStreamer packaging task.
Builds without this variable have update installation disabled. A malformed
key is rejected at verification time; it must never be replaced by the
upstream YStreamer key.

## 3. Sign and publish a package

Use `device/packaging/sign.sh path/to/tmstreamer_arm64.deb` on the trusted
signing workstation. It writes a base64 Ed25519 signature next to the package.
Generate `manifest.json` with `device/packaging/manifest.sh`, then publish
the manifest and signed package over HTTPS. Configure the device's
`TMSTREAMER_UPDATE_URL` to the HTTPS manifest URL.

The manifest's architecture key must match `dpkg --print-architecture`.
The signature covers the exact Debian package bytes.

## 4. Keep the rollout closed until verified

Before enabling this pipeline on devices, verify all of the following:
- The public key in the production binary matches the private signing key.
- A genuine TMStreamer package verifies and a one-byte-modified package fails.
- Wrong product and wrong architecture packages are rejected.
- HTTPS manifest and package retrieval work from the target network.
- Install health-check and rollback behavior pass on a disposable test device.

The legacy `device/packaging/install.sh` is disabled because it used the
upstream YStreamer endpoint and key. Do not use it to install TMStreamer.
MundoStreamer services on the Radxa remain untouched until an explicitly
approved cutover.
