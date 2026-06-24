#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="RustPad"
VERSION="0.1.0"
BUNDLE_OSX_DIR="${PROJECT_DIR}/target/release/bundle/osx"

echo "=== RustPad macOS 构建脚本 ==="
echo "项目目录: $PROJECT_DIR"

# 1. 编译 release
echo ""
echo ">>> 步骤 1/6: 编译 release..."
cd "$PROJECT_DIR"
cargo build --release 2>&1
echo "编译完成"

# 2. 清理旧产物
echo ""
echo ">>> 步骤 2/6: 清理旧产物..."
rm -rf "${PROJECT_DIR}/${APP_NAME}.app"
rm -rf "${BUNDLE_OSX_DIR}/${APP_NAME}.app"
rm -f "${PROJECT_DIR}/${APP_NAME}.dmg"
mkdir -p "${BUNDLE_OSX_DIR}"

# 3. 创建 .app 结构
echo ""
echo ">>> 步骤 3/6: 创建 .app 包..."
APP_DIR="${PROJECT_DIR}/${APP_NAME}.app"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp "${PROJECT_DIR}/target/release/rustpad" "${APP_DIR}/Contents/MacOS/${APP_NAME}"
chmod +x "${APP_DIR}/Contents/MacOS/${APP_NAME}"

if [ -f "${PROJECT_DIR}/assets/icon.icns" ]; then
    cp "${PROJECT_DIR}/assets/icon.icns" "${APP_DIR}/Contents/Resources/icon.icns"
fi

cat > "${APP_DIR}/Contents/Info.plist" << PLIST
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
    <string>1</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleIconFile</key>
    <string>icon</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
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

echo "Info.plist 已生成"

# 4. 同步到 cargo-bundle 标准路径
echo ""
echo ">>> 步骤 4/6: 同步到 target/release/bundle/osx/..."
cp -R "${APP_DIR}" "${BUNDLE_OSX_DIR}/${APP_NAME}.app"

# 5. 代码签名 (adhoc + entitlements)
echo ""
echo ">>> 步骤 5/6: 代码签名..."
ENTITLEMENTS="${PROJECT_DIR}/RustPad.entitlements"
for APP in "${APP_DIR}" "${BUNDLE_OSX_DIR}/${APP_NAME}.app"; do
    codesign --force --deep --sign - \
        --entitlements "${ENTITLEMENTS}" \
        "${APP}" 2>&1
    codesign --verify --deep --strict "${APP}" 2>&1 && echo "签名验证通过: ${APP}"
done

# 6. 创建 DMG
echo ""
echo ">>> 步骤 6/6: 创建 DMG..."
DMG_NAME="${PROJECT_DIR}/${APP_NAME}.dmg"
STAGING_DIR=$(mktemp -d)
cp -R "${BUNDLE_OSX_DIR}/${APP_NAME}.app" "${STAGING_DIR}/"
ln -s /Applications "${STAGING_DIR}/Applications"

hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "${STAGING_DIR}" \
    -ov \
    -format UDZO \
    "${DMG_NAME}" 2>&1 || {
    echo "警告: hdiutil 创建 DMG 失败，请在本机终端手动运行此脚本"
    exit 1
}

rm -rf "${STAGING_DIR}"
hdiutil verify "${DMG_NAME}" 2>&1

echo ""
echo "=== 构建完成 ==="
echo "  .app (根目录):     ${APP_DIR}"
echo "  .app (bundle路径): ${BUNDLE_OSX_DIR}/${APP_NAME}.app"
echo "  .dmg:              ${DMG_NAME}"
