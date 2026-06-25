#!/bin/bash
# Create RustPad.app from the release binary.
# Usage: macos_package_app.sh <output_app_path> [project_dir]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_PATH="${1:?Usage: macos_package_app.sh <output/RustPad.app> [project_dir]}"
PROJECT_DIR="${2:-"$(cd "$SCRIPT_DIR/.." && pwd)"}"
APP_NAME="RustPad"

mkdir -p "${APP_PATH}/Contents/MacOS" "${APP_PATH}/Contents/Resources"
cp "${PROJECT_DIR}/target/release/rustpad" "${APP_PATH}/Contents/MacOS/${APP_NAME}"
chmod +x "${APP_PATH}/Contents/MacOS/${APP_NAME}"

if [ -f "${PROJECT_DIR}/assets/icon.icns" ]; then
    cp "${PROJECT_DIR}/assets/icon.icns" "${APP_PATH}/Contents/Resources/icon.icns"
fi

"$SCRIPT_DIR/generate_macos_info_plist.sh" "${APP_PATH}/Contents" "$PROJECT_DIR"
echo "Packaged: ${APP_PATH}"
