<div align="center">

<img src="device/frontend/public/logo.svg" alt="TMStreamer" width="96" height="96" />

# TMStreamer

**Your FPV feed, on every screen.**

TMStreamer is an independently maintained distribution based on the upstream YStreamer software. It receives DJI goggles video or USB camera input and can send it to HDMI, browsers, network streams and recordings.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-2563eb.svg?style=flat-square)](LICENSE)
[![ARM64](https://img.shields.io/badge/Target-ARM64-blue?style=flat-square)](https://github.com/tamaabm-glitch/TMStreamer/actions)

[Releases](https://github.com/tamaabm-glitch/TMStreamer/releases) · [Source](https://github.com/tamaabm-glitch/TMStreamer)

</div>

---

## Project status

This repository is the active TMStreamer standalone integration branch. It retains required upstream copyright and provenance notices while replacing product identity, packaging, licensing and update infrastructure. Radxa ROCK 3A support remains under hardware evaluation; a successful ARM64 build alone does not certify DJI input, decoding, HDMI output or receiver compatibility.

## Capabilities inherited from upstream

- DJI goggles USB video ingest using the DUML protocol, plus legacy and UVC input paths.
- HDMI display output through GStreamer.
- Browser viewing and RTSP, SRT, RTMP and UDP outputs.
- Video recording and device status/camera telemetry features.

Availability depends on the connected DJI device, kernel, USB gadget configuration and installed GStreamer plugins.

## Safety and compatibility principles

- Preserve DJI DUML parsing, USB handling, camera control, GStreamer pipelines, streaming, HDMI and recording unless a change is separately tested.
- Debian package identity is `tmstreamer`; the systemd unit is `tmstreamer.service`.
- The executable is `/usr/bin/tmstreamer`. Existing settings in `/etc/ystreamer` and state in `/var/lib/ystreamer` are read only as migration fallbacks when the corresponding TMStreamer paths do not yet exist.
- Never fabricate or reuse upstream signing keys, user credentials, activation secrets or TM-LIC private keys.
- Keep TM-LIC activation and validation independent from upstream licensing.
- Test on Radxa ROCK 3A hardware before declaring an installer ready.
- On-device deployment should preserve existing settings and recordings.

## Installing the Debian package

The ARM64 build produces a package named `tmstreamer`, installs `/usr/bin/tmstreamer`, and provides `tmstreamer.service`.

On a compatible Debian 12 ARM64 test device, install the package only after reviewing its control scripts and dependencies:

```sh
sudo apt install ./tmstreamer_0.1.5-1_arm64.deb
```

The package deliberately does not enable or start `tmstreamer.service`. Do not start it on the Radxa while MundoStreamer or another DJI receiver is active. Service activation is a separate, explicitly approved cutover after the DJI input, HDMI output, TM-LIC gates, and rollback procedure pass on the actual hardware. No production GitHub Release has been published, and the signed updater remains disabled. The current package is not yet certified on Radxa ROCK 3A.

## TM-LIC integration

The integration uses the TM-LIC verifier as the only entitlement authority; it does not use legacy MUNDOLIC tokens or the dashboard's cached license-status JSON.

- Runtime default: `/data/opt/tm-lic`; verifier command: `PYTHONPATH=/data/opt/tm-lic /usr/bin/python3 -m app.cli status`.
- The service reads the verifier's JSON result directly. `TM_LIC_ROOT` and `TM_LIC_PYTHON` can override the runtime and interpreter paths.
- The dashboard accepts a signed TM-LIC JSON license file. The backend verifies the staged file against the device identity and registered public key before atomically replacing `/data/state/tm-lic/licenses/TMStreamer-LICENSE.json`. An invalid file does not replace an existing license.
- Streaming and DVR routes/settings require the corresponding `streaming` and `recording` entitlements. Startup also keeps paid outputs and the physical recording trigger disabled when verification fails.
- Receiver, camera control (`/control`), network, display and HDMI setup remain available without a license.
- Restart `tmstreamer.service` after a new license is activated so output transports disabled during startup can be initialized with the newly verified entitlement.

The device package declares Python 3 because the TM-LIC verifier is a separate runtime installed alongside TMStreamer. The issuer's private signing key must remain off the device.

## Building

The project uses mise to install Rust, Node and pnpm toolchains.

```sh
mise run frontend:install
mise run frontend:build
mise run backend:deps
mise run backend:test
mise run backend:lint
mise run device:package
```

See mise.toml for the full task list. The Debian package build requires the GStreamer development headers and other dependencies listed by the project.

## Repository layout

| Path | Contents |
| --- | --- |
| device/backend/ | Rust service, DJI input, GStreamer and web API |
| device/frontend/ | Svelte dashboard and visual assets |
| device/packaging/ | Debian packaging, service unit and install scripts |
| image/ | Legacy Raspberry Pi image builder (disabled pending a TMStreamer-specific migration) |
| site/ | Documentation website |

## Upstream origin and attribution

TMStreamer is based on YStreamer. The upstream project and its authors retain their applicable copyrights and notices. This repository's modifications do not imply endorsement by upstream authors or DJI.

The upstream README acknowledges CosmoStreamer as inspiration and documents DJI protocol research sources. These acknowledgements are retained as project provenance; TMStreamer does not claim ownership of those projects or protocols.

## License

The upstream code is distributed under GNU General Public License version 3.0 (GPL-3.0-only), as indicated by package metadata and the accompanying LICENSE. Modified distributions must continue to meet applicable GPL source, notice and license requirements. Preserve all applicable copyright notices.
