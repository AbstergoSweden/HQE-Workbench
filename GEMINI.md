# HQE Workbench

**HQE Workbench** is a local-first macOS desktop application and CLI designed for running the **HQE (High Quality Engineering) Engineer Protocol**. It automates codebase health auditing, secret redaction, and provides a "Thinktank" prompt library for AI-assisted engineering tasks, all while prioritizing privacy and security.

## Project Overview

*   **Type:** Hybrid Application (Rust CLI + Tauri/React Desktop App)
*   **Core Logic:** Rust (Workspace with multiple crates)
*   **Frontend:** TypeScript, React, Tailwind CSS, Tauri v2
*   **Key Features:**
    *   Automated repository scanning and health auditing.
    *   Local-only mode for privacy.
    *   Encrypted chat system with persistent history.
    *   "Thinktank" prompt library with 30+ expert prompts.
    *   LLM integration (OpenAI-compatible providers).

## Architecture

The project is structured as a Rust workspace with a modular architecture:

*   **`cli/hqe`**: The CLI entry point.
*   **`desktop/workbench`**: The Tauri/React desktop application.
*   **`crates/`**: Shared internal libraries:
    *   `hqe-core`: Core scan engine, pipeline orchestration, and encrypted chat logic.
    *   `hqe-openai`: OpenAI-compatible client and provider management.
    *   `hqe-git`: Git operations wrapper.
    *   `hqe-artifacts`: Report and manifest generation.
    *   `hqe-mcp`: Model Context Protocol implementation (Thinktank prompts).
    *   `hqe-ingest`: File system walking and ingestion.
    *   `hqe-flow`: Workflow execution.
*   **`mcp-server/`**: Configuration and data for the Thinktank prompt library.

## Building and Running

### Prerequisites

*   **Rust:** 1.75+
*   **Node.js:** 20+
*   **macOS:** 12.0+ (Monterey)
*   **Python:** 3.11+ (for protocol validation scripts)

### Preflight Check

**Always run the preflight check before committing changes.** This executes build, tests, formatting, and linting for both Rust and TypeScript.

```bash
npm run preflight
```

### CLI

To build and run the CLI:

```bash
# Build release binary
cargo build --release -p hqe

# Run a scan
./target/release/hqe scan /path/to/repo --local-only
```

### Desktop App

To develop and build the desktop application:

```bash
cd desktop/workbench

# Install dependencies
npm install

# Run in development mode (hot-reloading)
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Development Conventions

### General

*   **Commit Messages:** Follow [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat(core): ...`, `fix(ui): ...`).
*   **Branching:** Use `feat/` for features, `fix/` for bugs, `docs/` for documentation.

### Rust

*   **Error Handling:** Use `thiserror` for library crates and `anyhow` for the CLI/binary.
*   **Async Runtime:** Use `tokio`.
*   **Documentation:** All public functions must have `///` doc comments.
*   **Testing:** Co-locate unit tests. Run `cargo test --workspace`.

### TypeScript / React

*   **Components:** Use Functional Components with Hooks.
*   **State Management:** Use `zustand` for global state.
*   **Styling:** Use Tailwind CSS.
*   **Type Safety:** **Strictly avoid `any`**. Use interfaces/types and proper narrowing.
*   **Testing:** Use `vitest`. Run `npm test` within `desktop/workbench`.

### Security

*   **Secrets:** Never commit secrets. Use the `keyring` crate/macOS Keychain for storage.
*   **Input/Output:** Validate all inputs and sanitize all LLM outputs (DOMPurify).
