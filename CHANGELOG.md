# Changelog

## 0.1.4

- Add experimental DJI RC-N2 / RC-N3 direct USB video input through FunctionFS Android Accessory mode.
- Add queued USB reads, DUML generation reset, H.265 detection and automatic keyframe requests.
- Prefer Rockchip MPP H.265 decode / H.264 encode on Radxa RK3568; retain FFmpeg fallback.
- Use Rockchip MPP for USB capture encoding when available.
- Expose DJI codec and converter status in the dashboard.

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
