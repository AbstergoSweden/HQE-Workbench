# HQE Workbench

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Build Status](https://img.shields.io/github/actions/workflow/status/AbstergoSweden/hqe-workbench/ci.yml?branch=main)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![Tauri Version](https://img.shields.io/badge/tauri-2.0.0-blue.svg)

**HQE Workbench** is a macOS desktop application and CLI tool for running the **High Quality Engineering (HQE)** protocol. It automates codebase health auditing, security scanning, and technical leadership tasks using a combination of local heuristics and LLM-powered analysis.

## Features

- **Hybrid Architecture**: High-performance Rust backend with a modern React/Tauri v2 frontend.
- **Local-First Security**: API keys stored in macOS Keychain; analysis performed locally where possible.
- **Protocol Automation**: Automated execution of HQE auditing and reporting protocols.
- **Monorepo Design**: Integrated workspace for Core logic, CLI tools, and UI applications.

## Installation

### Prerequisites

- macOS 12.0+ (Monterey)
- Rust 1.75+
- Node.js 20+

### Bootstrap

Run the setup script to initialize your environment:

```bash
./scripts/bootstrap_macos.sh
```

## Usage

### Desktop Application (Dev Mode)

Start the Tauri development server:

```bash
./scripts/dev.sh
```

OR

```bash
cd apps/workbench
npm run tauri:dev
```

### CLI Tool

Build and run the CLI:

```bash
cargo run -p hqe -- --help
```

### Building for Release

Create a universal macOS binary and DMG:

```bash
./scripts/build_dmg.sh
```

Artifacts will be output to `target/release/bundle/dmg/`.

## Development

This project is a **Rust Workspace** containing:

- `crates/`: Shared libraries (`hqe-core`, `hqe-openai`, etc.)
- `cli/`: CLI entry point (`hqe`)
- `apps/workbench`: Tauri frontend application

### Testing

Run the full test suite (Rust + JS):

```bash
# Rust tests
cargo test --workspace

# Frontend tests
cd apps/workbench && npm test
```

### Linting & Formatting

```bash
# Rust
cargo fmt -- --check
cargo clippy --workspace -- -D warnings

# TypeScript
cd apps/workbench && npm run lint
```

## contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the [MIT License](LICENSE).
