#!/bin/bash
# RustPad macOS release builder.
#
# Local development (default — ad-hoc signature, no notarization):
#   ./scripts/build_macos.sh
#   ./scripts/build_macos.sh --adhoc
#
# Public distribution (Developer ID + notarization + staple):
#   export SIGN_MODE=release
#   export MACOS_SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)"
#   export APPLE_ID="you@example.com"
#   export APPLE_APP_PASSWORD="xxxx-xxxx-xxxx-xxxx"
#   export APPLE_TEAM_ID="XXXXXXXXXX"
#   ./scripts/build_macos.sh --release
#
# Or use a stored notarytool keychain profile:
#   xcrun notarytool store-credentials rustpad-notary \
#     --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD"
#   export NOTARYTOOL_KEYCHAIN_PROFILE=rustpad-notary
#   ./scripts/build_macos.sh --release
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="RustPad"
SIGN_MODE="${SIGN_MODE:-adhoc}"
VERSION="$("$SCRIPT_DIR/read_cargo_version.sh" "$PROJECT_DIR")"
BUNDLE_OSX_DIR="${PROJECT_DIR}/target/release/bundle/osx"
APP_DIR="${PROJECT_DIR}/${APP_NAME}.app"
DMG_PATH="${PROJECT_DIR}/${APP_NAME}.dmg"

usage() {
    sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --adhoc)
            SIGN_MODE=adhoc
            shift
            ;;
        --release)
            SIGN_MODE=release
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

export SIGN_MODE

echo "=== RustPad macOS 构建脚本 ==="
echo "项目目录: $PROJECT_DIR"
echo "版本号:   $VERSION (来自 Cargo.toml)"
echo "签名模式: $SIGN_MODE"

echo ""
echo ">>> 步骤 1/6: 编译 release..."
cd "$PROJECT_DIR"
cargo build --release
echo "编译完成"

echo ""
echo ">>> 步骤 2/6: 清理旧产物..."
rm -rf "$APP_DIR"
rm -rf "${BUNDLE_OSX_DIR}/${APP_NAME}.app"
rm -f "$DMG_PATH"
mkdir -p "$BUNDLE_OSX_DIR"

echo ""
echo ">>> 步骤 3/6: 创建 .app 包..."
"$SCRIPT_DIR/macos_package_app.sh" "$APP_DIR" "$PROJECT_DIR"

echo ""
echo ">>> 步骤 4/6: 同步到 target/release/bundle/osx/..."
cp -R "$APP_DIR" "${BUNDLE_OSX_DIR}/${APP_NAME}.app"

echo ""
echo ">>> 步骤 5/6: 代码签名..."
"$SCRIPT_DIR/macos_codesign.sh" "$APP_DIR" "$PROJECT_DIR"
"$SCRIPT_DIR/macos_codesign.sh" "${BUNDLE_OSX_DIR}/${APP_NAME}.app" "$PROJECT_DIR"

echo ""
echo ">>> 步骤 6/6: 创建 DMG..."
"$SCRIPT_DIR/macos_create_dmg.sh" "${BUNDLE_OSX_DIR}/${APP_NAME}.app" "$DMG_PATH"
cp -f "$DMG_PATH" "${PROJECT_DIR}/${APP_NAME}-${VERSION}.dmg" 2>/dev/null || true

if [ "$SIGN_MODE" = "release" ]; then
    echo ""
    echo ">>> 步骤 7/7: 签名 DMG、公证并 staple..."
    "$SCRIPT_DIR/macos_codesign.sh" "$DMG_PATH" "$PROJECT_DIR"
    "$SCRIPT_DIR/macos_notarize.sh" "$DMG_PATH"
    "$SCRIPT_DIR/macos_notarize.sh" "$APP_DIR"
    cp -f "$DMG_PATH" "${PROJECT_DIR}/${APP_NAME}-${VERSION}-notarized.dmg"
fi

echo ""
echo "=== 构建完成 ==="
echo "  .app (根目录):     ${APP_DIR}"
echo "  .app (bundle路径): ${BUNDLE_OSX_DIR}/${APP_NAME}.app"
echo "  .dmg:              ${DMG_PATH}"
if [ "$SIGN_MODE" = "release" ]; then
    echo "  .dmg (notarized):  ${PROJECT_DIR}/${APP_NAME}-${VERSION}-notarized.dmg"
    echo ""
    echo "分发检查:"
    spctl -a -vv "$DMG_PATH" || true
    codesign -dv --verbose=4 "$APP_DIR" 2>&1 | grep -E 'Authority|TeamIdentifier|Signature' || true
else
    echo ""
    echo "提示: 当前为 ad-hoc 签名，仅适合本机/内部测试。"
    echo "      正式分发请使用: SIGN_MODE=release ./scripts/build_macos.sh --release"
fi
