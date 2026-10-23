#!/usr/bin/env bash
# Bundle the release binary into Qwerkeys.app and a distributable DMG.
#
# Usage: scripts/bundle.sh [version]
#   Expects the matching binary to exist. Creates:
#     - Qwerkeys.app          (staged ad-hoc signed bundle)
#     - dmg-staging/          (drag-to-install staging)
#     - Qwerkeys-<version>.dmg
#
# A version ending in `-debug` (e.g. `v0.0.0-debug`) bundles the DEBUG build
# (target/debug/qwerkeys) instead — handy for `just install v0.0.0-debug`
# testing: the bundled .app keeps `debug_assertions` so the onboarding page
# is always shown.

set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>" >&2
  echo "  e.g. $0 v1.2.3 (version used for the output filename only)" >&2
  echo "  append -debug to bundle the debug binary instead of release" >&2
  exit 1
fi
VERSION="${VERSION#v}"

APP="Qwerkeys.app"

# A `-debug` suffix selects the debug binary; everything else uses release.
case "$VERSION" in
  *-debug) BIN="target/debug/qwerkeys" ;;
  *) BIN="target/release/qwerkeys" ;;
esac

[[ -x "$BIN" ]] || { echo "error: $BIN not found — build it first" >&2; exit 1; }
[[ -f "packaging/Info.plist" ]] || { echo "error: packaging/Info.plist not found" >&2; exit 1; }

echo "Bundling Qwerkeys version: $VERSION (bin: $BIN)"

# --- Stage the .app bundle ---------------------------------------------------
rm -rf "$APP" dmg-staging
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp packaging/Info.plist "$APP/Contents/Info.plist"
# Generate the icns from the SVG source, always, so the freshest artwork is
# bundled (see generate-icon.swift).
scripts/generate-icon.swift
[[ -f "assets/AppIcon.icns" ]] || { echo "error: assets/AppIcon.icns not found" >&2; exit 1; }
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
