#!/usr/bin/env bash
# Create a clean source-only zip archive
# Usage: ./scripts/make_source_zip.sh [output_path]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Default output path
OUTPUT_PATH="${1:-${REPO_ROOT}/hqe-workbench-source.zip}"

# Version info
VERSION=$(grep '^version' "${REPO_ROOT}/Cargo.toml" | head -1 | cut -d'"' -f2)
DATE=$(date +%Y%m%d)

if [[ "$OUTPUT_PATH" == *".zip" ]]; then
    BASENAME=$(basename "$OUTPUT_PATH" .zip)
else
    BASENAME=$(basename "$OUTPUT_PATH")
    OUTPUT_PATH="${OUTPUT_PATH}.zip"
fi

# Use temp directory for staging
TEMP_DIR=$(mktemp -d)
trap "rm -rf '$TEMP_DIR'" EXIT

STAGING_DIR="${TEMP_DIR}/${BASENAME}"
mkdir -p "$STAGING_DIR"

echo "📦 Creating source archive..."
echo "   Source: ${REPO_ROOT}"
echo "   Output: ${OUTPUT_PATH}"
echo "   Version: ${VERSION}"
echo ""

# Use git archive if available, otherwise rsync
if [[ -d "${REPO_ROOT}/.git" ]]; then
    echo "📝 Using git archive..."
    git -C "$REPO_ROOT" archive --format=tar HEAD | tar -C "$STAGING_DIR" -xf -
else
    echo "📝 Using rsync (no git repo found)..."
    rsync -a --exclude='.git' --exclude='target' --exclude='node_modules' \
        --exclude='.DS_Store' --exclude='__MACOSX' --exclude='._*' \
        --exclude='*.pdb' --exclude='*.log' \
        --exclude='dist/' --exclude='dist-ssr/' \
        --exclude='hqe-output/' --exclude='hqe-exports/' \
        "${REPO_ROOT}/" "$STAGING_DIR/"
fi

# Create zip from staging directory
echo "🗜️  Compressing..."
(cd "$TEMP_DIR" && ditto -c -k --keepParent "${BASENAME}" "$OUTPUT_PATH")

# Report
echo ""
echo "✅ Archive created: ${OUTPUT_PATH}"
echo ""
echo "📊 Contents (top-level):"
unzip -l "$OUTPUT_PATH" | tail -10
echo ""
echo "📊 Size:"
du -h "$OUTPUT_PATH"
echo ""

# Verify exclusions
echo "🔍 Verifying exclusions..."
if unzip -l "$OUTPUT_PATH" | grep -E "(node_modules|target/)" > /dev/null 2>&1; then
    echo "⚠️  WARNING: Archive contains node_modules or target/ directories!"
    exit 1
else
    echo "✅ No node_modules or target/ directories found"
fi
