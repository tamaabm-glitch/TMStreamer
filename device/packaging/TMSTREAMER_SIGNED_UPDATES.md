# TMStreamer signed update configuration

The updater is deliberately closed in ordinary development and CI builds.
It does not trust the former YStreamer signing key or update server.

The official distribution target is **GitHub Releases only** for
`tamaabm-glitch/TMStreamer`. GitHub Pages, Cloudflare R2, and a custom domain
are not required. The repository must remain public for anonymous device
downloads.

## 1. Create the TMStreamer release key

Run `sh device/packaging/generate-update-key.sh` on the release operator's
trusted workstation (preferably offline). The default outputs are:

- Private Ed25519 PEM: `~/.config/tmstreamer/release-signing-ed25519.pem`
- Public key: `./tmstreamer-update-key.pub` (base64 of the raw 32-byte key)

Back up the private key securely. Never commit it, put it in CI artifacts,
copy it to a Radxa device, or send it to the device. Only the public key may
be shared with the build environment.

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

## 3. Sign the package and prepare the GitHub Release assets

Use `device/packaging/sign.sh path/to/tmstreamer_<version>-1_arm64.deb` on the
trusted signing workstation. It writes a base64 Ed25519 signature in the
adjacent `.deb.sig` file. Do not upload the private key or rely on an
unsigned CI artifact as a production release.

Create a **non-draft, non-prerelease** GitHub Release with tag
`v<version>`, for example `v0.1.7`. Generate its manifest using that exact
release's download base URL:

```sh
RELEASE_TAG="v0.1.7"
RELEASE_BASE="https://github.com/tamaabm-glitch/TMStreamer/releases/download/$RELEASE_TAG"
sh device/packaging/manifest.sh path/to/tmstreamer_0.1.7-1_arm64.deb "$RELEASE_BASE"
sh device/packaging/sign.sh path/to/manifest.json
```

Upload these three files as assets to the same GitHub Release:

- `tmstreamer_0.1.7-1_arm64.deb`
- `manifest.json`
- `manifest.json.sig`

The `.deb.sig` file is consumed by `manifest.sh`; its base64 contents are
embedded in `manifest.json`, so it does not need to be a separate release
asset. The manifest itself is then signed as exact raw bytes. Its detached
signature is published as `manifest.json.sig` and must verify before the
device parses or trusts the manifest. Both signatures use the same Ed25519
release key. The generated manifest points to the versioned package URL,
while devices use this stable manifest URL:

`https://github.com/tamaabm-glitch/TMStreamer/releases/latest/download/manifest.json`

The release must be published as the latest stable release for that URL to
resolve to its `manifest.json` asset. Do not mark a production release as a
prerelease or draft.

## 4. Device configuration — do not apply until security approval

When the signed-update rollout has passed all tests, configure the service
environment variable `TMSTREAMER_UPDATE_URL` to:

`https://github.com/tamaabm-glitch/TMStreamer/releases/latest/download/manifest.json`

Do not configure this variable or enable update checking on production
devices before the verification gate below is complete.

The manifest's architecture key must match `dpkg --print-architecture`.
The signature covers the exact Debian package bytes. Both the manifest and
package downloads must use HTTPS.

## 5. Keep the rollout closed until verified

Before enabling this pipeline on devices, verify all of the following:

- The public key in the production binary matches the private signing key.
- The detached manifest signature verifies; changing one byte in the manifest fails.
- A genuine TMStreamer package verifies and a one-byte-modified package fails.
- Wrong product and wrong architecture packages are rejected.
- The manifest and detached signature resolve over HTTPS from the target network.
- Package URLs outside the official TMStreamer GitHub Releases path are rejected.
- The package URL resolves over HTTPS from the target network.
- The stable `releases/latest/download/manifest.json` URL resolves to the
  intended release asset, not an older source archive.
- The manifest version matches the Debian package version.
- Install health-check and rollback behavior pass on a disposable test device.

The legacy `device/packaging/install.sh` is disabled because it used the
upstream YStreamer endpoint and key. Do not use it to install TMStreamer.
MundoStreamer services on the Radxa remain untouched until an explicitly
approved cutover.
