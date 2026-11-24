#!/bin/sh
# The old image pipeline still installs YStreamer-named data-partition
# services and expects the former executable. It is not compatible with the
# TMStreamer/Radxa package; fail closed until replaced with a Radxa image flow.
set -eu
echo "Legacy Raspberry Pi image builder is disabled; use the Debian ARM64 package only after hardware acceptance." >&2
exit 1
