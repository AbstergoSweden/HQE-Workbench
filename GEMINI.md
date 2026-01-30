# HQE Workbench

HQE Workbench is a local-first macOS desktop application and CLI tool for running the HQE (High Quality Engineering) Engineer Protocol. It automates codebase health auditing, security scanning, and technical leadership tasks using a combination of local heuristics and LLM-powered analysis.

## Project Overview

- **Type:** Hybrid Rust (Backend/CLI) and TypeScript/React (Frontend/Tauri) application.
- **Core capabilities:** Repository scanning, secret redaction, local-only mode, and report generation (Markdown/JSON).
- **Architecture:** Monorepo with a Cargo workspace for Rust crates and a Vite project for the frontend.

## Project Structure

```text
hqe-workbench/
├── apps/
│   └── workbench/       # Tauri v2 macOS application (React + TypeScript)
├── cli/
│   └── hqe/             # CLI entry point (Rust)
├── crates/              # Shared Rust libraries
│   ├── hqe-core/        # Core scan pipeline, models, and redaction logic
│   ├── hqe-openai/      # OpenAI-compatible LLM client
│   ├── hqe-git/         # Git operations wrapper
│   └── hqe-artifacts/   # Report and manifest generation
├── protocol/            # HQE Protocol v3 definitions (YAML/JSON schemas)
├── scripts/             # Build and utility scripts
└── docs/                # Architecture and design documentation
```

## Getting Started

### Prerequisites
- macOS 12.0+ (Monterey)
- Rust 1.75+
- Node.js 20+
- Python 3.10+ (for protocol validation)

### Build & Run

**1. Bootstrap Environment:**
```bash
./scripts/bootstrap_macos.sh
```

**2. Development Mode:**
This starts the Tauri development server and the frontend watcher.
```bash
./scripts/dev.sh
```
*Alternatively:* `cd apps/workbench && npm run tauri:dev`

**3. Build CLI:**
```bash
cargo build --release -p hqe
```

**4. Build Desktop App (Release):**
```bash
./scripts/build_dmg.sh
```

## Development Workflow

### Testing
- **Rust Tests:** `cargo test --workspace`
- **Protocol Validation:** `./scripts/validate_protocol.sh` (Mandatory before commits)
- **Frontend Linting:** `cd apps/workbench && npm run lint`

### Code Quality
- **Formatting:** `cargo fmt`
- **Linting:** `cargo clippy --workspace -- -D warnings`

### Key Modules
- **`hqe-core`**: The brain of the operation. Handles the 4-phase scan pipeline (Ingestion, Analysis, Reporting, Export).
- **`hqe-openai`**: Handles communication with LLM providers. Includes robust retry logic and configuration.
- **`RedactionEngine`**: (In `hqe-core`) Ensures secrets (AWS keys, tokens) are stripped before sending data to any LLM.

## Security & Privacy
- **Local-Only Mode:** Prioritized for sensitive repos. Uses heuristics instead of external APIs.
- **Secret Management:** API keys are stored in the macOS Keychain, never in config files.
- **Redaction:** All content is scanned and redacted before leaving the local machine.

## Protocol
The project implements **HQE Protocol v3**. Definitions are located in the `protocol/` directory. Any changes to the protocol YAML must be validated against the schema using `verify.py`.
