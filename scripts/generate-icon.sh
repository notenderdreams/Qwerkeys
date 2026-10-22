#!/usr/bin/env bash
# Regenerate assets/AppIcon.icns from assets/app-icon.svg.
#
# Usage: scripts/generate-icon.sh
#
# Requires: rsvg-convert (brew install librsvg) and iconutil (built into macOS).

set -euo pipefail

export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"

cd "$(dirname "$0")/.."

SRC="assets/app-icon.svg"
OUT="assets/AppIcon.icns"
IMGDIR="packaging/AppIcon.iconset"

command -v rsvg-convert >/dev/null 2>&1 || {
  echo "error: rsvg-convert not found — install with 'brew install librsvg'" >&2
  exit 1
}
command -v iconutil >/dev/null 2>&1 || {
  echo "error: iconutil not found (should be present on macOS)" >&2
  exit 1
}
[[ -f "$SRC" ]] || { echo "error: $SRC not found" >&2; exit 1; }

mkdir -p "$IMGDIR"
trap 'rm -rf "$IMGDIR"' EXIT

# Render every size the iconset needs, then rename the dual-purpose ones.
for size in 16 32 64 128 256 512 1024; do
  rsvg-convert -w "$size" -h "$size" "$SRC" -o "$IMGDIR/icon_${size}x${size}.png"
done
# Keep 512x512 PNG for runtime viewport icon
cp "$IMGDIR/icon_512x512.png" "assets/app-icon.png"

mv "$IMGDIR/icon_64x64.png"    "$IMGDIR/icon_32x32@2x.png"
mv "$IMGDIR/icon_256x256.png"  "$IMGDIR/icon_128x128@2x.png"
mv "$IMGDIR/icon_512x512.png"  "$IMGDIR/icon_256x256@2x.png"
mv "$IMGDIR/icon_1024x1024.png" "$IMGDIR/icon_512x512@2x.png"

iconutil -c icns "$IMGDIR" -o "$OUT"

echo "✓ Generated $OUT from $SRC"
