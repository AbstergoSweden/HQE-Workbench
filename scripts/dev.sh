#!/usr/bin/env bash
# Development script - runs protocol validation, then Tauri dev

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🚀 HQE Workbench - Development Mode"
echo "===================================="
echo ""

# Validate protocol first
echo "🧪 Validating protocol..."
"$REPO_ROOT/scripts/validate_protocol.sh" || {
    echo "⚠️  Protocol validation failed - continuing anyway"
}

echo ""
echo "🖥️  Starting Tauri development server..."
echo ""

cd "$REPO_ROOT/apps/workbench"
npm run tauri:dev
