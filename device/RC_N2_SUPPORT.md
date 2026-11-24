# DJI RC-N2 / RC-N3 USB video input

Status: **implemented in source; experimental until tested on the target Radxa board**.

## Hardware path

The RC-N2 video path is direct USB, not an Android MSDK relay:

1. The DJI RC-N2 is the USB host; TMStreamer presents an Android Accessory (AOA) device on the Radxa USB-C OTG/device-mode port.
2. The FunctionFS interface uses the vendor-specific class/subclass required by the RC-N2. The older generic serial gadget is retained only as an unbound legacy configfs entry; it is not used for the new link.
3. FunctionFS bulk OUT data is read with Linux AIO, keeping multiple reads queued to avoid dropping the remote's continuous video stream. If AIO is unavailable, the reader falls back to one-at-a-time reads.
4. DUML/LogicLink frames are reassembled. A connection-generation change clears any partial frame from the previous USB session.
5. The stream codec is detected. Goggles H.264 is passed through. RC-N2 / RC-N3 H.265 is parsed, VPS/SPS/PPS are retained, and a fresh keyframe is requested when a decoder joins mid-stream.
6. On Radxa ROCK 3A / RK3568, the preferred converter is GStreamer Rockchip MPP (mppvideodec + mpph264enc) to produce H.264 for the existing HDMI, browser, RTSP, SRT, RTMP, UDP and DVR fan-out. The MPP pipeline is selected only when both plugins are discoverable. The existing FFmpeg hardware path remains a fallback for other images.

## Radxa requirements

- Radxa ROCK 3A / RK3568 with the USB 3.0 OTG/HOST port operating in device mode for the RC-N2 connection.
- Power the board independently while the USB-C data link is in use; do not assume the controller cable can power the board.
- configfs, libcomposite, and FunctionFS support in the running kernel.
- gstreamer1.0-tools and the standard GStreamer H.265/H.264 parser plugins.
- For the preferred RK3568 hardware path, both mppvideodec and mpph264enc must be visible to gst-inspect-1.0. The board-specific Rockchip plugin is deliberately not installed or upgraded automatically by the TMStreamer package.
- ffmpeg is installed for the non-MPP hardware fallback.

The RK3568 SoC is specified for H.265/H.264 decoding and H.264/H.265 encoding. That silicon capability does not prove that the particular kernel, MPP library, GStreamer plugin and device permissions on a given board are configured correctly.

## How to connect

- Connect the RC-N2 controller's USB-C phone/data port to the Radxa USB-C OTG/HOST port using a known data-capable cable.
- Leave the TMStreamer input selected as **DJI goggles / RC-N**.
- Power and link the aircraft normally. The stream detector should report H.265, then the converter should report hardware conversion.
- Do not connect the RC-N2 to a USB-A host port and expect this AOA path to work.

## Scope and limitations

- RC-N2 / RC-N3 video reception is experimental until tested with the actual controller, aircraft, cable and Radxa board.
- This code path is video-only at present. It does not claim RC-N2 camera settings, battery telemetry or link-quality support.
- DJI RC 2 is a different controller and is not covered by this RC-N2 implementation.
- No hardware test, package installation or service cutover is implied by a successful CI build.
- The existing MundoStreamer services and receiver must remain untouched until a separate, explicitly approved device test.

## Upstream open-source technical references

This implementation adapts the relevant GPL-licensed upstream YStreamer changes, while retaining TMStreamer product identity and TM-LIC integration:

- [FunctionFS accessory interface](https://github.com/YLabs-FPV/YStreamer/commit/b95cd85415fcd7fcdd59d37fd6e311f3bdda81f5)
- [Queued USB reads using Linux AIO](https://github.com/YLabs-FPV/YStreamer/commit/5c43cf4f4743613d2d12a2e2ecd374245215b0ce)
- [H.265 detection and conversion](https://github.com/YLabs-FPV/YStreamer/commit/7315b1dd1759942c0cd7f4031b6c6958ef64abfc)
- [RC-N2 / RC-N3 keyframe request and live view](https://github.com/YLabs-FPV/YStreamer/commit/08b14e4a6bc3725ec1b3a0bd557625d49709cdfd)
- [Radxa ROCK 3A product brief](https://dl.radxa.com/rock3/docs/hw/3a/radxa_rock3a_product_brief.pdf)
- [Rockchip GStreamer MPP encoder examples](https://community.t-firefly.com/en/docs/software/multimedia/GStreamer/gstreamer-rockchip-encoding)

Required upstream copyright and license notices must remain in the repository.
