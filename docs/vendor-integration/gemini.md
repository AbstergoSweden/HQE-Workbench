# HQE Workbench

## Project Overview

**HQE Workbench** is a local-first macOS desktop application and CLI tool designed for high-quality engineering workflows. It automates codebase health auditing, secret redaction, and provides an encrypted, LLM-powered chat interface for code analysis.

The project is a hybrid application combining:
- **Core Logic:** Rust (High performance, safety)
- **Desktop UI:** Tauri + React + TypeScript (Modern frontend)
- **Scripts:** Python & Bash (Automation, protocol validation)

## Architecture

The codebase is organized as a Cargo workspace with a separate Tauri frontend.

### Key Directories

- **`cli/hqe/`**: The entry point for the CLI application.
- **`crates/`**: Internal Rust crates containing the core business logic.
    - `hqe-core`: Scan engine, encrypted chat database, and central logic.
    - `hqe-git`: Git repository operations.
    - `hqe-openai`: OpenAI-compatible API client.
    - `hqe-mcp`: Model Context Protocol implementation (Thinktank prompts).
    - `hqe-artifacts`: Report generation (Markdown/JSON).
- **`desktop/workbench/`**: The Tauri desktop application source code (React, TypeScript, Tailwind CSS).
- **`mcp-server/`**: Contains the "Thinktank" prompt library and agent configurations.
- **`protocol/`**: HQE Protocol schemas and validation scripts (Python).
- **`scripts/`**: Maintenance, build, and setup scripts (e.g., `bootstrap_macos.sh`).

## Building and Running

### Prerequisites

- **OS:** macOS 12.0+ (Monterey)
- **Rust:** v1.75+
- **Node.js:** v20+
- **Python:** v3.11+

### Quick Start

1.  **Bootstrap Environment:**
    ```bash
    ./scripts/bootstrap_macos.sh
    ```

2.  **Full Preflight Check (Recommended):**
    Runs builds, tests, and linting for the entire project.
    ```bash
    npm run preflight
    ```

### CLI

- **Build Release Binary:**
    ```bash
    cargo build --release -p hqe
    # Binary location: target/release/hqe
    ```

- **Run Locally:**
    ```bash
    cargo run -p hqe -- scan /path/to/repo --local-only
    ```

### Desktop App

Navigate to the workbench directory: `cd desktop/workbench`

- **Development Mode:**
    ```bash
    npm run tauri:dev
    ```

- **Build Production App:**
    ```bash
    npm run tauri:build
    ```

## Development Conventions

### Testing

- **Rust (Core/CLI):**
    ```bash
    cargo test --workspace
    # With SQLCipher features
    cargo test --workspace --features sqlcipher-tests
    ```

- **Frontend (React/Tauri):**
    ```bash
    cd desktop/workbench
    npm test  # Runs Vitest
    ```

### Linting & Formatting

- **Frontend:**
    ```bash
    cd desktop/workbench
    npm run lint
    ```
- **Rust:** Standard `cargo fmt` and `cargo clippy`.

### Security

- **Secrets:** The application is designed to redact secrets. **Never** commit API keys or sensitive data to the repository.
- **Encryption:** Chat history is stored in an encrypted SQLite database (SQLCipher). Key management is handled via the macOS Keychain.
