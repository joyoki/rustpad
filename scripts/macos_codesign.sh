#!/bin/bash
# Sign a macOS .app bundle or .dmg.
#
# SIGN_MODE=adhoc (default)  — local / CI smoke builds (Signature=adhoc)
# SIGN_MODE=release          — Developer ID Application + hardened runtime
#
# Release mode requires:
#   MACOS_SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)"
#
# Usage: macos_codesign.sh <path/to/RustPad.app|.dmg> [project_dir]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TARGET="${1:?Usage: macos_codesign.sh <app-or-dmg> [project_dir]}"
PROJECT_DIR="${2:-"$(cd "$SCRIPT_DIR/.." && pwd)"}"
SIGN_MODE="${SIGN_MODE:-adhoc}"
ENTITLEMENTS="${ENTITLEMENTS:-${PROJECT_DIR}/RustPad.entitlements}"
APP_NAME="RustPad"

if [ ! -f "$ENTITLEMENTS" ]; then
    echo "error: entitlements not found: $ENTITLEMENTS" >&2
    exit 1
fi

sign_one() {
    local path="$1"
    local identity="$2"
    shift 2
    local -a extra_args=()
    if [ "$#" -gt 0 ]; then
        extra_args=("$@")
    fi

    run_codesign() {
        local target="$1"
        if ((${#extra_args[@]} > 0)); then
            codesign --force "${extra_args[@]}" \
                --entitlements "$ENTITLEMENTS" \
                --sign "$identity" \
                "$target"
        else
            codesign --force \
                --entitlements "$ENTITLEMENTS" \
                --sign "$identity" \
                "$target"
        fi
    }

    sign_file() {
        local target="$1"
        if ((${#extra_args[@]} > 0)); then
            codesign --force "${extra_args[@]}" --sign "$identity" "$target"
        else
            codesign --force --sign "$identity" "$target"
        fi
    }

    if [ -d "$path" ] && [[ "$path" == *.app ]]; then
        local binary="${path}/Contents/MacOS/${APP_NAME}"
        if [ -f "$binary" ]; then
            run_codesign "$binary"
        fi
        run_codesign "$path"
    else
        sign_file "$path"
    fi
}

case "$SIGN_MODE" in
    adhoc)
        echo ">>> codesign (adhoc): ${TARGET}"
        # Ad-hoc signing uses identity "-" only; --options=adhoc is not valid on macOS.
        sign_one "$TARGET" "-"
        ;;
    release)
        if [ -z "${MACOS_SIGN_IDENTITY:-}" ]; then
            echo "error: SIGN_MODE=release requires MACOS_SIGN_IDENTITY" >&2
            echo '  e.g. MACOS_SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)"' >&2
            exit 1
        fi
        echo ">>> codesign (Developer ID): ${TARGET}"
        sign_one "$TARGET" "$MACOS_SIGN_IDENTITY" --options=runtime --timestamp
        ;;
    *)
        echo "error: unknown SIGN_MODE=${SIGN_MODE} (use adhoc or release)" >&2
        exit 1
        ;;
esac

codesign --verify --deep --strict --verbose=2 "$TARGET"
echo "Signature OK: ${TARGET}"

if [ "$SIGN_MODE" = "release" ]; then
    spctl -a -vv "$TARGET" || {
        echo "note: spctl may report 'Unnotarized' until notarization + staple complete" >&2
    }
fi
