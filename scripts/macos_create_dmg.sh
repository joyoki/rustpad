#!/bin/bash
# Create a compressed DMG from a signed .app (adds /Applications symlink).
# Usage: macos_create_dmg.sh <RustPad.app> <output.dmg>
set -euo pipefail

APP_PATH="${1:?Usage: macos_create_dmg.sh <RustPad.app> <output.dmg>}"
DMG_PATH="${2:?Usage: macos_create_dmg.sh <RustPad.app> <output.dmg>}"
APP_NAME="RustPad"

if [ ! -d "$APP_PATH" ]; then
    echo "error: app bundle not found: $APP_PATH" >&2
    exit 1
fi

STAGING_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGING_DIR"' EXIT

cp -R "$APP_PATH" "${STAGING_DIR}/"
ln -s /Applications "${STAGING_DIR}/Applications"

rm -f "$DMG_PATH"
hdiutil detach "/Volumes/${APP_NAME}" -force 2>/dev/null || true

for attempt in 1 2 3; do
    if hdiutil create \
        -volname "$APP_NAME" \
        -srcfolder "$STAGING_DIR" \
        -ov \
        -format UDZO \
        "$DMG_PATH"; then
        break
    fi
    rm -f "$DMG_PATH"
    hdiutil detach "/Volumes/${APP_NAME}" -force 2>/dev/null || true
    sleep 2
    if [ "$attempt" -eq 3 ]; then
        echo "error: hdiutil create failed after 3 attempts" >&2
        exit 1
    fi
done

hdiutil verify "$DMG_PATH"
echo "DMG created: ${DMG_PATH}"
