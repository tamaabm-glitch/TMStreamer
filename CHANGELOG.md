# Changelog

## 0.1.5

- Use TMStreamer names for fresh NetworkManager and WireGuard profiles while preserving existing legacy profiles until the user reapplies settings.
- Save new recordings under /var/lib/tmstreamer and keep older recording folders selectable without moving or deleting data.
- Migrate browser preferences to TMStreamer storage keys and replace upstream support/download links.
- Rename the GPIO recording-button consumer and authentication session cookie.

## 0.1.4

- Add experimental DJI RC-N2 / RC-N3 direct USB video input through FunctionFS Android Accessory mode.
- Add queued USB reads, DUML session reset, H.265 detection and automatic keyframe requests.
- Prefer Rockchip MPP H.265 decode / H.264 encode on Radxa RK3568, with FFmpeg fallback.
- Use Rockchip MPP for USB capture encoding when available and expose DJI codec status in the dashboard.

## 0.1.3

- Rename the installed executable and systemd service to TMStreamer.
- Move fresh recording defaults and runtime paths to TMStreamer locations.
- Disable the legacy Raspberry Pi image pipeline pending a Radxa-specific replacement.
- Harden signed update restarts, rollback, and GitHub Release package validation.
- Add automated offline signing and manifest integrity tests.

## 0.1.2

- Add HDMI output presets for 720p, 1080i, and 1080p frame rates.
- Detect and expose interlaced display modes.
- Prefer progressive HDMI modes when selecting an automatic display mode.
- Update the startup splash branding to TMStreamer.
- Update the Wi-Fi QR prompt to SCAN TO CONNECT.


## 0.1.0

- First release
