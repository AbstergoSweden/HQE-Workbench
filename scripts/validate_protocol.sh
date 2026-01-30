#!/usr/bin/env bash
# Validate HQE Protocol YAML against schema
# Usage: ./scripts/validate_protocol.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

YAML_FILE="${REPO_ROOT}/protocol/hqe-engineer.yaml"
SCHEMA_FILE="${REPO_ROOT}/protocol/hqe-schema.json"
VERIFY_SCRIPT="${REPO_ROOT}/protocol/verify.py"

echo "🔍 Validating HQE Protocol..."
echo "   YAML:   ${YAML_FILE}"
echo "   Schema: ${SCHEMA_FILE}"
echo ""

# Check files exist
if [[ ! -f "$YAML_FILE" ]]; then
    echo "❌ Error: YAML file not found: $YAML_FILE"
    exit 2
fi

if [[ ! -f "$SCHEMA_FILE" ]]; then
    echo "❌ Error: Schema file not found: $SCHEMA_FILE"
    exit 2
fi

# Install Python dependencies if needed
if ! python3 -c "import yaml, jsonschema" 2>/dev/null; then
    echo "📦 Installing Python dependencies..."
    python3 -m pip install --quiet pyyaml jsonschema 2>/dev/null || {
        echo "⚠️  Warning: Could not install dependencies automatically"
        echo "   Run: pip install pyyaml jsonschema"
        exit 2
    }
fi

# Run validation
echo "🧪 Running validation..."
echo ""

if python3 "$VERIFY_SCRIPT" --yaml "$YAML_FILE" --schema "$SCHEMA_FILE"; then
    echo ""
    echo "✅ Protocol validation passed"
    exit 0
else
    echo ""
    echo "❌ Protocol validation failed"
    exit 1
fi
