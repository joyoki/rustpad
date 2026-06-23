#!/bin/bash
# RustPad build script for macOS
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

echo "=== RustPad macOS Build Script ==="
echo ""

# Clean previous builds
echo "[1/6] Cleaning previous builds..."
cargo clean --release 2>/dev/null || true

# Run tests
echo "[2/6] Running tests..."
cargo test --release

# Run clippy
echo "[3/6] Running clippy..."
cargo clippy --release -- -D warnings

# Build release
echo "[4/6] Building release..."
cargo build --release

# Check if cargo-bundle is installed
echo "[5/6] Packaging..."
if command -v cargo-bundle &> /dev/null; then
    cargo bundle --release
    APP_PATH="target/release/bundle/osx/RustPad.app"
    
    if [ -d "$APP_PATH" ]; then
        echo "  App bundle created: $APP_PATH"
        
        # Create DMG if create-dmg is available
        if command -v create-dmg &> /dev/null; then
            echo "[6/6] Creating DMG..."
            DMG_PATH="target/release/RustPad.dmg"
            create-dmg \
                --volname "RustPad" \
                --window-pos 200 120 \
                --window-size 600 400 \
                --icon-size 100 \
                --icon "RustPad.app" 175 190 \
                --hide-extension "RustPad.app" \
                --app-drop-link 425 190 \
                "$DMG_PATH" \
                "$APP_PATH" 2>/dev/null || echo "  DMG creation skipped (create-dmg not fully configured)"
            echo "  DMG: $DMG_PATH"
        else
            echo "[6/6] Skipping DMG creation (install create-dmg: brew install create-dmg)"
        fi
    else
        echo "  Warning: App bundle not found at expected path"
    fi
else
    echo "[6/6] Skipping bundle (install cargo-bundle: cargo install cargo-bundle)"
    echo "  Release binary: target/release/rustpad"
fi

echo ""
echo "=== Build complete ==="
echo "Binary: target/release/rustpad"
