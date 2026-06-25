#!/bin/bash
# Generate RustPad.app/Contents/Info.plist using version from Cargo.toml.
# Usage: generate_macos_info_plist.sh <path-to-.app/Contents> [project_dir]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CONTENTS_DIR="${1:?Usage: generate_macos_info_plist.sh <path-to-.app/Contents> [project_dir]}"
PROJECT_DIR="${2:-"$(cd "$SCRIPT_DIR/.." && pwd)"}"

VERSION="$("$SCRIPT_DIR/read_cargo_version.sh" "$PROJECT_DIR")"

MIN_OS="10.15"
if [ -f "${PROJECT_DIR}/Cargo.toml" ]; then
    parsed_min_os="$(grep 'osx_minimum_system_version' "${PROJECT_DIR}/Cargo.toml" | head -1 | sed -E 's/.*"([^"]+)".*/\1/' || true)"
    if [ -n "$parsed_min_os" ]; then
        MIN_OS="$parsed_min_os"
    fi
fi

mkdir -p "$CONTENTS_DIR"

cat > "${CONTENTS_DIR}/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>RustPad</string>
    <key>CFBundleIdentifier</key>
    <string>com.rustpad.editor</string>
    <key>CFBundleName</key>
    <string>RustPad</string>
    <key>CFBundleDisplayName</key>
    <string>RustPad</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleIconFile</key>
    <string>icon</string>
    <key>LSMinimumSystemVersion</key>
    <string>${MIN_OS}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Text File</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>txt</string>
                <string>rs</string>
                <string>py</string>
                <string>js</string>
                <string>ts</string>
                <string>html</string>
                <string>css</string>
                <string>json</string>
                <string>toml</string>
                <string>md</string>
                <string>xml</string>
                <string>yml</string>
                <string>yaml</string>
                <string>sh</string>
                <string>c</string>
                <string>cpp</string>
                <string>h</string>
                <string>java</string>
                <string>go</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
        </dict>
    </array>
</dict>
</plist>
PLIST

echo "Info.plist generated (CFBundleShortVersionString=${VERSION})"
