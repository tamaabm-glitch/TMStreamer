# TM-LIC device integration

This branch adds a native Rust verifier and device-side activation flow for TM-LIC v1.

## Device storage

- Machine ID: `/var/lib/ystreamer/tm-lic/machine_id`
- Activated license: `/var/lib/ystreamer/tm-lic/license.json`
- Public key registry: `/usr/share/ystreamer/tm-lic/registry.json`
- Public keys: `/usr/share/ystreamer/tm-lic/public_keys/<key_id>.pem`

The default state path is a proposed location under the existing systemd `StateDirectory=ystreamer`. Before the first Radxa deployment, verify the actual image mount layout and persistence of `/var/lib/ystreamer`; override it with `TM_LIC_STATE_DIR` if the device uses a different persistent data mount. Do not deploy until that storage audit is complete. The TM-LIC subdirectory is set to mode 0700; Machine ID and license files are created with mode 0600. Machine ID is generated once from the operating system CSPRNG and persisted. It is a random persistent identifier, not a hardware fingerprint. A malformed existing Machine ID is reported as an error and is never silently replaced.

For test or staging environments, the paths can be overridden with:

- `TM_LIC_STATE_DIR`
- `TM_LIC_REGISTRY`
- `TM_LIC_PUBLIC_KEY_DIR`

## API

- `GET /api/license` returns activation status and Machine ID.
- `POST /api/license/activate` accepts the raw JSON license envelope (maximum 64 KiB).

Initial activation is public so a locked installation can be activated without an administrative session. Replacing an already active license requires an administrator session. The license is validated before an atomic write. Only a signed license is stored on the device.

## Feature enforcement

- The `recording` entitlement is required for all `/api/recording` endpoints, recording settings, auto-start, and the physical record button.
- The `streaming` entitlement is required for RTSP/SRT/RTMP/UDP outputs, browser video/WebRTC, and their settings.
- Full settings replacement requires both entitlements.
- Network outputs start only after entitlements have been applied. A background check stops them and stops active recording when a license expires or its signing key is revoked. Browser video fanout also stops; the DJI control/telemetry channel remains available.
- DJI input selection, network configuration, HDMI/display configuration, system/update access, logs, and diagnostics remain available while activation is pending.
- The HDMI pipeline, DJI receiver, and device boot path are not gated by TM-LIC.

The frontend displays the activation page until a valid license is present. It keeps the setup pages above available during activation.

## Key handling and production readiness

Never put a private signing key on the device, in this repository, or in a Debian package. The device only needs the public key registry, public PEM key, and signed license.

The sample key and signed sample license distributed in the TM-LIC v1.0 archive are used only by a verifier compatibility test. They are not production signing material. The packaged registry is deliberately a placeholder (`PRODUCTION_KEY_REQUIRED`) with no active key entries, so it cannot validate any production license yet. Create a fresh production Ed25519 keypair on a trusted issuer workstation, retain the private key there, and replace the placeholder registry plus add only the matching public PEM to `device/packaging/tm-lic/` before producing a release package.
