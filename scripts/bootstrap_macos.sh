#!/usr/bin/env bash
# Bootstrap script for HQE Workbench on macOS
# Installs all prerequisites

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "🚀 HQE Workbench - macOS Bootstrap"
echo "=================================="
echo ""

# Check macOS version
if [[ "$(uname)" != "Darwin" ]]; then
    echo "❌ Error: This script is for macOS only"
    exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Install Homebrew if not present
if ! command_exists brew; then
    echo "📦 Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    eval "$(/opt/homebrew/bin/brew shellenv)" 2>/dev/null || eval "$(/usr/local/bin/brew shellenv)"
else
    echo "✅ Homebrew already installed"
fi

# Install Node.js LTS
if ! command_exists node; then
    echo "📦 Installing Node.js LTS..."
    brew install node
else
    echo "✅ Node.js already installed ($(node --version))"
fi

# Install Rust
if ! command_exists rustc; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust already installed ($(rustc --version))"
fi

# Ensure Rust is up to date
echo "🔄 Updating Rust toolchain..."
rustup update stable

# Install Tauri prerequisites
echo "📦 Installing Tauri system dependencies..."
brew install libiconv

# Install Python (for protocol validation)
if ! command_exists python3; then
    echo "📦 Installing Python..."
    brew install python
else
    echo "✅ Python already installed ($(python3 --version))"
fi

# Install Python dependencies
echo "📦 Installing Python dependencies..."
python3 -m pip install --quiet pyyaml jsonschema 2>/dev/null || true

# Install Node dependencies
echo "📦 Installing Node.js dependencies..."
cd "$REPO_ROOT/apps/workbench"
npm install

# Build Rust dependencies
echo "📦 Building Rust workspace..."
cd "$REPO_ROOT"
cargo fetch

# Validate protocol
echo "🧪 Validating HQE Protocol..."
"$REPO_ROOT/scripts/validate_protocol.sh" || true

echo ""
echo "✅ Bootstrap complete!"
echo ""
echo "Next steps:"
echo "  1. Run: ./scripts/dev.sh        # Start development server"
echo "  2. Or:  cargo build --release   # Build CLI"
echo ""
