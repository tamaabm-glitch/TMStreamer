#!/bin/sh
# The upstream YStreamer installer is intentionally disabled in TMStreamer.
# It referenced the vendor update server and vendor signing key, and could
# replace/start services on a device without TMStreamer cutover approval.
set -eu
cat >&2 <<'EOF'
The legacy YStreamer installer is disabled.
TMStreamer release distribution is not configured yet.
No download, package installation, service restart, or boot configuration was performed.
EOF
exit 1
