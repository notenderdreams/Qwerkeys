#!/usr/bin/env bash
# Regenerate assets/AppIcon.icns from assets/app-icon.svg.
#
# Usage: scripts/generate-icon.sh
#
# Requires: rsvg-convert (brew install librsvg) and iconutil (built into macOS).
# The SVG contains the icon artwork in its left 824x824 square, so the source
# is cropped to that region before scaling.

set -euo pipefail

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

# Crop the SVG to the icon region (0,0 - 824,824) by rewriting the viewBox.
TMP_SVG="packaging/.app-icon-crop.svg"
trap 'rm -rf "$IMGDIR" "$TMP_SVG"' EXIT

sed -e 's/width="2579" height="957" viewBox="0 0 2579 957"/width="824" height="824" viewBox="0 0 824 824"/' \
  "$SRC" > "$TMP_SVG"

# Render every size the iconset needs, then rename the dual-purpose ones.
for size in 16 32 64 128 256 512 1024; do
  rsvg-convert -w "$size" -h "$size" "$TMP_SVG" -o "$IMGDIR/icon_${size}x${size}.png"
done
mv "$IMGDIR/icon_64x64.png"    "$IMGDIR/icon_32x32@2x.png"
mv "$IMGDIR/icon_256x256.png"  "$IMGDIR/icon_128x128@2x.png"
mv "$IMGDIR/icon_512x512.png"  "$IMGDIR/icon_256x256@2x.png"
mv "$IMGDIR/icon_1024x1024.png" "$IMGDIR/icon_512x512@2x.png"

iconutil -c icns "$IMGDIR" -o "$OUT"

echo "✓ Generated $OUT from $SRC"
