#!/bin/bash
# Notarize and staple a macOS distributable (.dmg or .app).
#
# Required (pick one auth method):
#   NOTARYTOOL_KEYCHAIN_PROFILE  — preferred (store via: xcrun notarytool store-credentials)
#   or APPLE_ID + APPLE_APP_PASSWORD + APPLE_TEAM_ID
#
# Usage: macos_notarize.sh <path/to/RustPad.dmg|.app>
set -euo pipefail

ARTIFACT="${1:?Usage: macos_notarize.sh <dmg-or-app>}"

if [ ! -e "$ARTIFACT" ]; then
    echo "error: artifact not found: $ARTIFACT" >&2
    exit 1
fi

NOTARY_ARGS=()
if [ -n "${NOTARYTOOL_KEYCHAIN_PROFILE:-}" ]; then
    NOTARY_ARGS+=(--keychain-profile "$NOTARYTOOL_KEYCHAIN_PROFILE")
else
    : "${APPLE_ID:?Set APPLE_ID or NOTARYTOOL_KEYCHAIN_PROFILE}"
    : "${APPLE_APP_PASSWORD:?Set APPLE_APP_PASSWORD or NOTARYTOOL_KEYCHAIN_PROFILE}"
    : "${APPLE_TEAM_ID:?Set APPLE_TEAM_ID or NOTARYTOOL_KEYCHAIN_PROFILE}"
    NOTARY_ARGS+=(--apple-id "$APPLE_ID" --password "$APPLE_APP_PASSWORD" --team-id "$APPLE_TEAM_ID")
fi

echo ">>> notarytool submit: ${ARTIFACT}"
xcrun notarytool submit "$ARTIFACT" "${NOTARY_ARGS[@]}" --wait

echo ">>> stapler staple: ${ARTIFACT}"
xcrun stapler staple "$ARTIFACT"
xcrun stapler validate "$ARTIFACT"

echo "Notarization complete: ${ARTIFACT}"
