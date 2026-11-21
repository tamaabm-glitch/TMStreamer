<div align="center">

<img src="device/frontend/public/logo.svg" alt="TMStreamer" width="96" height="96" />

# TMStreamer

**NEW GALIH MULTIMEDIA — Your FPV feed, on every screen.**

TMStreamer is a rebranded and modified distribution of the upstream YStreamer software. It is intended to receive DJI goggles video or USB camera input and send it to HDMI, browsers, network streams and recordings.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-2563eb.svg?style=flat-square)](LICENSE)
[![ARM64](https://img.shields.io/badge/Target-ARM64-blue?style=flat-square)](https://github.com/tamaabm-glitch/TMStreamer/actions)

[Releases](https://github.com/tamaabm-glitch/TMStreamer/releases) · [Source](https://github.com/tamaabm-glitch/TMStreamer)

</div>

---

## Project status

This repository contains an in-progress TMStreamer rebrand and integration branch. The upstream software's documented, tested device is Raspberry Pi 4. Radxa ROCK 3A support is a target under evaluation, not yet certified. An ARM64 build alone does not prove hardware compatibility.

## Capabilities inherited from upstream

- DJI goggles USB video ingest using the DUML protocol, plus legacy and UVC input paths.
- HDMI display output through GStreamer.
- Browser viewing and RTSP, SRT, RTMP and UDP outputs.
- Video recording and device status/camera telemetry features.

Availability depends on the connected DJI device, kernel, USB gadget configuration and installed GStreamer plugins.

## Safety and compatibility principles

- Preserve DJI DUML parsing, USB handling, camera control, GStreamer pipelines, streaming, HDMI and recording unless a change is separately tested.
- Keep Linux service/package identifiers such as ystreamer unchanged during visual rebranding; renaming them can break upgrades, settings and service management.
- Never fabricate or reuse upstream signing keys, user credentials, activation secrets or TM-LIC private keys.
- Keep TM-LIC activation and validation independent from upstream licensing.
- Test on Radxa ROCK 3A hardware before declaring an installer ready.
- Back up settings and recordings before any on-device upgrade.

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
| device/frontend/ | Svelte web dashboard and visual assets |
| device/packaging/ | Debian packaging, service unit and install scripts |
| image/ | Upstream Raspberry Pi image build |
| site/ | Documentation website |

## Upstream origin and attribution

TMStreamer is based on YStreamer. The upstream project and its authors retain their applicable copyrights and notices. This repository's modifications do not imply endorsement by upstream authors or DJI.

The upstream README acknowledges CosmoStreamer as inspiration and documents DJI protocol research sources. These acknowledgements are retained as project provenance; TMStreamer does not claim ownership of those projects or protocols.

## License

The upstream code is distributed under GNU GPL version 3.0 (GPL-3.0-only), as indicated by package metadata and the accompanying LICENSE. Modified distributions must continue to meet applicable GPL source, notice and license requirements. Preserve all applicable copyright notices.