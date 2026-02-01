# HQE Workbench - Build Instructions

## Quick Start

```bash
# 1. Bootstrap (installs all dependencies)
./scripts/bootstrap_macos.sh

# 2. Run in development mode
./scripts/dev.sh

# 3. Build release DMG
./scripts/build_dmg.sh
```

## Project Structure

```text
hqe-workbench/
├── protocol/                    # HQE Protocol v3 (verbatim from ZIP)
│   ├── hqe-engineer.yaml       # Protocol definition
│   ├── hqe-schema.json         # JSON Schema
│   ├── verify.py               # Python validator
│   ├── verify.hs               # Haskell validator
│   └── HQE Engineer*.md        # Original docs
│
├── crates/                      # Rust workspace crates
│   ├── hqe-core/               # Core scan pipeline
│   │   ├── src/models.rs       # Data types
│   │   ├── src/scan.rs         # Scan pipeline
│   │   ├── src/redaction.rs    # Secret redaction
│   │   └── src/repo.rs         # Repo scanning
│   │
│   ├── hqe-openai/             # OpenAI-compatible client
│   │   ├── src/lib.rs          # HTTP client
│   │   └── src/prompts.rs      # Prompt templates
│   │
│   ├── hqe-git/                # Git operations
│   │   └── src/lib.rs          # Git wrapper
│   │
│   └── hqe-artifacts/          # Report generation
│       └── src/lib.rs          # Markdown/JSON writers
│
├── cli/                         # CLI binary
│   └── hqe/
│       └── src/main.rs         # CLI commands
│
├── desktop/                     # Desktop applications
│   └── workbench/              # Tauri v2 macOS app
│       ├── src/                # React + TypeScript UI
│       │   ├── screens/        # Page components
│       │   ├── components/     # Shared components
│       │   └── store.ts        # State management
│       └── src-tauri/          # Rust backend
│           └── src/commands.rs # Tauri commands
│
├── scripts/                     # Build scripts
│   ├── bootstrap_macos.sh      # Install dependencies
│   ├── dev.sh                  # Development mode
│   ├── build_dmg.sh            # Build release
│   └── validate_protocol.sh    # Protocol validation
│
├── docs/                        # Documentation
│   ├── architecture.md         # System design
│   ├── threat-model.md         # Security analysis
│   ├── provider-config.md      # LLM setup
│   └── artifact-format.md      # Output specs
│
├── .github/workflows/           # CI/CD
│   └── ci.yml                  # GitHub Actions
│
├── Cargo.toml                  # Workspace config
└── README.md                   # Project overview
```

## Prerequisites

- macOS 12.0+ (Monterey)
- Homebrew
- Node.js 20+
- Rust 1.75+
- Python 3.10+

## Building

### 1. Install Dependencies

```bash
./scripts/bootstrap_macos.sh
```

This installs:

- Homebrew (if needed)
- Node.js
- Rust toolchain
- Tauri system deps
- Python dependencies

### 2. Validate Protocol

```bash
./scripts/validate_protocol.sh
```

Verifies the HQE Protocol YAML against its schema.

### 3. Build CLI

```bash
cargo build --release -p hqe

# Test
./target/release/hqe --version
./target/release/hqe validate-protocol
```

### 4. Build Desktop App

Development mode:

```bash
./scripts/dev.sh
```

Release build:

```bash
./scripts/build_dmg.sh
```

Output: `target/release/bundle/*.dmg`

## Usage

### CLI

```bash
# Validate protocol
hqe validate-protocol

# Local-only scan
hqe scan ./my-repo --local-only --out ./reports

# LLM-powered scan
hqe scan ./my-repo --profile openai --out ./reports

# Configure provider
hqe config add openai --url https://api.openai.com/v1 --key sk-xxx --model gpt-4o-mini

# Test provider
hqe config test openai
```

### GUI App

1. Open `HQE Workbench.app` (from DMG)
2. Select a repository folder
3. Configure provider (optional)
4. Run scan
5. View report
6. Export artifacts

## Development

### Watch Mode

```bash
# Terminal 1: Tauri dev
cd desktop/workbench && npm run tauri:dev

# Terminal 2: Run tests
cargo test --workspace --watch
```

### Testing

```bash
# Rust tests
cargo test --workspace

# With coverage
cargo tarpaulin --workspace
```

### Code Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy --workspace -- -D warnings

# Protocol validation
./scripts/validate_protocol.sh
```

## Troubleshooting

### "No module named 'yaml'"

```bash
pip3 install pyyaml jsonschema
```

### "command not found: cargo"

```bash
source $HOME/.cargo/env
```

### "No such file: tauri.conf.json"

```bash
cd desktop/workbench && npm install
```

### macOS Gatekeeper blocks app

```bash
xattr -cr /Applications/HQE\ Workbench.app
```

## Release Checklist

- [ ] All tests pass (`cargo test`)
- [ ] Protocol validates (`./scripts/validate_protocol.sh`)
- [ ] No clippy warnings
- [ ] Code formatted (`cargo fmt`)
- [ ] DMG builds successfully
- [ ] App launches on clean macOS
- [ ] CLI works standalone

## Platform Notes

### macOS Universal Binary

The build produces a universal binary supporting:

- Apple Silicon (arm64)
- Intel (x86_64)

### Code Signing

The build script doesn't include Apple Developer signing.
For distribution, you'll need:

- Apple Developer account
- Signing certificate
- Notarization (optional but recommended)

To sign manually:

```bash
codesign --force --deep --sign "Developer ID Application: Your Name" HQE\ Workbench.app
```

## Support

- Issues: GitHub Issues
- Docs: See `/docs` directory
- Protocol: See `/protocol` directory
