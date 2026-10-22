#!/usr/bin/env bash
# Bundle the release binary into Qwerkeys.app and a distributable DMG.
#
# Usage: scripts/bundle.sh [version]
#   Expects target/release/qwerkeys to exist. Creates:
#     - Qwerkeys.app          (staged ad-hoc signed bundle)
#     - dmg-staging/          (drag-to-install staging)
#     - Qwerkeys-<version>.dmg

set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>" >&2
  echo "  e.g. $0 v1.2.3 (version used for the output filename only)" >&2
  exit 1
fi
VERSION="${VERSION#v}"

BIN="target/release/qwerkeys"
APP="Qwerkeys.app"

[[ -x "$BIN" ]] || { echo "error: $BIN not found — run 'cargo build --release' first" >&2; exit 1; }
[[ -f "packaging/Info.plist" ]] || { echo "error: packaging/Info.plist not found" >&2; exit 1; }

echo "Bundling Qwerkeys version: $VERSION"

# --- Stage the .app bundle ---------------------------------------------------
rm -rf "$APP" dmg-staging
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp packaging/Info.plist "$APP/Contents/Info.plist"
# Generate the icns from the SVG source, always, so the freshest artwork is
# bundled (requires rsvg-convert; see generate-icon.sh).
scripts/generate-icon.sh
[[ -f "assets/AppIcon.icns" ]] || { echo "error: assets/AppIcon.icns not found — install librsvg ('brew install librsvg') and rerun" >&2; exit 1; }
cp assets/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
/usr/libexec/PlistBuddy \
  -c "Set :CFBundleVersion $VERSION" \
  -c "Set :CFBundleShortVersionString $VERSION" \
  "$APP/Contents/Info.plist"

cp "$BIN" "$APP/Contents/MacOS/qwerkeys"
codesign --force --deep --sign - "$APP"
echo "✓ Staged and signed $APP"

# --- Build the DMG -----------------------------------------------------------
# App + an Applications symlink so users get the classic "drag to install" window.
mkdir dmg-staging
cp -R "$APP" dmg-staging/
ln -s /Applications dmg-staging/Applications

DMG="Qwerkeys-$VERSION.dmg"
hdiutil create -volname "Qwerkeys" \
  -srcfolder dmg-staging \
  -ov -format UDZO \
  "$DMG"
echo "✓ Created $DMG"
