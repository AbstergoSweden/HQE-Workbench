#!/usr/bin/env bash
# Build script for HQE Workbench DMG

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="$REPO_ROOT/target/release/bundle"

echo "🔨 HQE Workbench - Build DMG"
echo "============================="
echo ""

# Validate protocol
echo "🧪 Step 1: Validating protocol..."
"$REPO_ROOT/scripts/validate_protocol.sh"

# Run tests
echo ""
echo "🧪 Step 2: Running tests..."
cd "$REPO_ROOT"
cargo test --workspace

# Build CLI
echo ""
echo "🔨 Step 3: Building CLI..."
cargo build --release -p hqe

# Build Tauri app
echo ""
echo "🔨 Step 4: Building Tauri app..."
cd "$REPO_ROOT/desktop/workbench"
npm run tauri:build

# Check outputs
echo ""
echo "📦 Build outputs:"
ls -la "$BUILD_DIR/" 2>/dev/null || true

# Find DMG
DMG_PATH=$(find "$BUILD_DIR" -name "*.dmg" -type f 2>/dev/null | head -n1)
APP_PATH=$(find "$BUILD_DIR" -name "*.app" -type d 2>/dev/null | head -n1)

echo ""
echo "✅ Build complete!"
echo ""

if [[ -f "$DMG_PATH" ]]; then
    echo "📀 DMG: $DMG_PATH"
    echo "   Size: $(du -h "$DMG_PATH" | cut -f1)"
    echo "   SHA256: $(shasum -a 256 "$DMG_PATH" | cut -d' ' -f1)"
fi

if [[ -d "$APP_PATH" ]]; then
    echo "   📁 App: $APP_PATH"
fi

echo ""
echo "🎉 Done! Install with:"
echo "   open \"$DMG_PATH\""
echo ""
