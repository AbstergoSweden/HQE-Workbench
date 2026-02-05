# HQE Workbench - Development Context

## Project Overview

HQE Workbench is a local-first macOS desktop application and CLI tool for running the HQE (High Quality Engineering) Engineer Protocol. It automates codebase health auditing and produces actionable, evidence-backed TODOs using a combination of local heuristics and (optional) LLM-powered analysis.

**Key Features:**
- Repository scanning with automated codebase health auditing
- Secret redaction and intelligent detection of sensitive data
- Local-only mode for privacy-first operation without external API calls
- Semantic caching with locally stored LLM responses in SQLite
- Comprehensive report generation in Markdown and JSON formats
- Encrypted chat system with SQLCipher AES-256 encryption
- Thinktank prompt library with 30+ expert prompts across Security, Quality, Refactor, Test, and Architecture categories
- Multi-provider LLM support (OpenAI, Anthropic, Venice, OpenRouter, xAI, Kimi)

## Architecture

The application follows a hybrid Rust/Python/TypeScript architecture:

- **Rust Core**: `hqe-core` handles scan pipeline and encrypted chat storage
- **LLM Client**: `hqe-openai` provides OpenAI-compatible chat client
- **Desktop App**: Tauri/React application for GUI interaction
- **CLI Tool**: Rust-based command-line interface
- **Protocol**: YAML-based specification for HQE Engineer protocol

### File Structure
```
hqe-workbench/
├── cli/hqe/                    # CLI Application Entry Point
├── crates/
│   ├── hqe-core/              # Scan Engine, Logic, Encrypted Chat DB
│   ├── hqe-flow/              # Workflow & Protocol Execution
│   ├── hqe-git/               # Git Operations
│   ├── hqe-ingest/            # Repository Ingestion & File Watching
│   ├── hqe-mcp/               # Model Context Protocol
│   ├── hqe-openai/            # AI Provider Client
│   ├── hqe-protocol/          # Schema & Type Defs
│   └── hqe-vector/            # Vector Database Operations
├── desktop/workbench/         # Desktop App (Tauri/React)
├── docs/                      # Architecture & Guides
├── mcp-server/                # Thinktank Prompt Library & MCP Server
├── prompts/                   # Prompt Examples & Guidance
├── protocol/                  # HQE Protocol Schemas
└── scripts/                   # Build & Test Scripts
```

## Building and Running

### Prerequisites
- **macOS**: 12.0+ (Monterey)
- **Rust**: 1.75+
- **Python**: 3.11+ (used for protocol validation)
- **Node.js**: 20+ (Workbench UI)

### Installation
```bash
# Bootstrap the environment (macOS)
./scripts/bootstrap_macos.sh

# Build the CLI
cargo build --release -p hqe

# The binary will be available at target/release/hqe
```

### Usage

#### CLI
```bash
# Run a local-only scan
./target/release/hqe scan /path/to/repo --local-only

# LLM-enabled scan (any OpenAI-compatible provider; text models only)
./target/release/hqe scan /path/to/repo --profile my-provider

# Disable local semantic caching
./target/release/hqe scan /path/to/repo --profile my-provider --no-cache

# Export an existing run to a folder
./target/release/hqe export RUN_ID --out ./hqe-exports
```

#### Desktop App
```bash
cd desktop/workbench

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

#### Thinktank Prompts
1. Open the Workbench desktop app
2. Navigate to the **Thinktank** tab
3. Browse prompts by category (Security, Quality, Refactor, etc.)
4. Select a prompt and fill in the required inputs
5. Click **Execute Prompt** to run analysis
6. Click **Start Chat** to continue the conversation

## Security Features

HQE Workbench implements defense-in-depth with multiple layers of protection:

| Layer | Protection | Implementation |
|-------|------------|----------------|
| Input Validation | Template key validation, path canonicalization | `prompts.rs`, `encrypted_db.rs` |
| Output Sanitization | DOMPurify for LLM output | `ConversationPanel.tsx` |
| Database Security | SQLCipher AES-256 encryption | `encrypted_db.rs` |
| Key Management | macOS Keychain integration | `keyring` crate |
| Prompt Security | 50+ jailbreak pattern detection | `system_prompt.rs` |
| Injection Prevention | Parameterized SQL queries | `encrypted_db.rs` |

## Development Conventions

### Rust Coding Standards
- Use `thiserror` for libraries, `anyhow` for CLI/binaries
- Prefer `tokio` for async runtime
- All public functions must have doc comments (`///`)
- Follow clippy linting rules (`cargo clippy --workspace -- -D warnings`)
- Format code with `cargo fmt`

### TypeScript/React Standards
- Use functional components with `React.FC` or simple functions
- Use `zustand` for global state management
- Use Tailwind CSS for styling
- Strict TypeScript mode is enforced (no `any` types)

### Commit Messages
Follow Conventional Commits:
- `feat(core): add new scanner`
- `fix(ui): resolve button alignment`
- `docs: update readme`

### Testing
Run the full preflight check before pushing:
```bash
npm run preflight
```

This runs:
- Rust tests & formatting (`cargo test`, `cargo fmt`)
- TypeScript linting & testing (`npm run lint`, `npm test`)

## Key Commands

- **Full local CI-equivalent checks**: `npm run preflight`
- **Rust tests only**: `cargo test --workspace`
- **Rust tests with SQLCipher**: `cargo test --workspace --features sqlcipher-tests`
- **Workbench lint and tests**: `cd desktop/workbench && npm run lint && npm test`
- **Validate protocol**: `./target/release/hqe validate-protocol`

## Security Considerations

- The application supports both local-only scans and LLM-enabled scans via OpenAI-compatible APIs
- When LLM mode is enabled, code snippets may be sent to the configured provider
- API keys are stored securely in macOS Keychain, not in plaintext
- All chat history is encrypted with SQLCipher AES-256 encryption
- The application implements XSS protection, SQL injection prevention, and prompt injection defense

## Project Governance

The project is maintained by Faye Håkansdotter and follows an open-source model with community contributions welcomed. The project is licensed under Apache License 2.0.

For security vulnerabilities, contact: <2-craze-headmen@icloud.com>