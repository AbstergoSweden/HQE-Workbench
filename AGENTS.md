# HQE-Workbench — Agent Context

> **Generated:** 2026-02-06 | **Sources:** `Cargo.toml`, `package.json`, `desktop/workbench/package.json`, `desktop/workbench/src-tauri/Cargo.toml`, `cli/hqe/Cargo.toml`, `.github/workflows/ci.yml`, `.pre-commit-config.yaml`

## Project Overview

HQE-Workbench is a local-first macOS desktop application + CLI for running the HQE (High Quality Engineering) Engineer Protocol. It automates codebase health auditing and produces actionable, evidence-backed TODOs using a combination of local heuristics and (optional) LLM-powered analysis.

**Key Features:**

- **Repository Scanning**: Automated codebase health auditing with local static analysis
- **Secret Redaction**: Intelligent detection and removal of sensitive data (API keys, tokens)
- **Local-Only Mode**: Privacy-first operation without external API calls
- **Encrypted Chat System**: SQLCipher AES-256 encryption for all chat history with macOS Keychain integration
- **Thinktank Prompt Library**: 30+ expert prompts for security audits, code review, refactoring
- **Semantic Caching**: Locally stores LLM responses in SQLite to reduce costs and latency

## System Identity

| Attribute | Value | Source |
| --------- | ----- | ------ |
| Language | Rust 2021 Edition | `Cargo.toml:19` |
| Rust Version | 1.75+ | `Cargo.toml:23` |
| TypeScript | 5.3+ (Workbench) | `desktop/workbench/package.json:49` |
| Python | 3.11+ (protocol validation) | `.github/workflows/ci.yml:116` |
| Platform | macOS 12.0+ | `README.md:147` |
| License | Apache-2.0 | `Cargo.toml:21` |
| Node.js | 20+ | `desktop/workbench/package.json` |

## Workspace Structure

> Cargo workspace with 11 members | `Cargo.toml:3-15`

| Package | Path | Purpose | Type |
| ------- | ---- | ------- | ---- |
| `hqe` | `cli/hqe` | CLI entry point | Binary |
| `hqe-workbench-app` | `desktop/workbench/src-tauri` | Tauri desktop app | Binary |
| `hqe-core` | `crates/hqe-core` | Scan pipeline, engine, encrypted chat DB | Library |
| `hqe-openai` | `crates/hqe-openai` | OpenAI-compatible client | Library |
| `hqe-git` | `crates/hqe-git` | Git operations | Library |
| `hqe-artifacts` | `crates/hqe-artifacts` | Report generation | Library |
| `hqe-protocol` | `crates/hqe-protocol` | Schema definitions | Library |
| `hqe-mcp` | `crates/hqe-mcp` | Model Context Protocol | Library |
| `hqe-ingest` | `crates/hqe-ingest` | File ingestion | Library |
| `hqe-vector` | `crates/hqe-vector` | Vector/embeddings (placeholder) | Library |
| `hqe-flow` | `crates/hqe-flow` | Workflow engine | Library |

### Module Responsibilities

#### `hqe-core`

- Scan orchestration and pipeline
- Repo walking + ingestion
- Secret detection and redaction
- Local-only analysis heuristics
- Encrypted chat database (SQLCipher)
- Producing `HqeReport` + `RunManifest`

#### `hqe-openai`

- OpenAI-compatible chat completion client
- Provider profiles + keychain storage
- `/models` discovery + filtering to text models
- Request/response serialization
- Retry logic and error classification

#### `hqe-git`

- Repository detection
- Patch generation/apply flows
- Shelling out to system `git`

#### `hqe-artifacts`

- Writing `run-manifest.json`
- Writing `report.json` and `report.md`
- Writing `session-log.json` and redaction logs

#### `hqe-mcp`

- Thinktank prompt library management
- Agent contexts and tools
- Prompt registry with metadata

#### `hqe-ingest`

- Efficient file system walking
- Handling ignore patterns (gitignore)
- File change notifications

#### `hqe-flow`

- Multi-step agent flow execution
- Protocol invariant validation
- MCP tool orchestration

## Quick Start

```bash
# Bootstrap macOS environment
./scripts/bootstrap_macos.sh

# Build CLI
cargo build --release -p hqe

# Run local-only scan
./target/release/hqe scan /path/to/repo --local-only

# Run Workbench desktop app
cd desktop/workbench && npm run tauri:dev
```

## Build and Test Commands

### Root Level

| Intent | Command | Notes | Source |
| -------- | ------- | ----- | ------ |
| Preflight (All) | `npm run preflight` | Rust tests + JS lint/test | `package.json:5` |
| Preflight Rust | `npm run preflight:rust` | Tests, clippy, fmt check | `package.json:6` |
| Preflight JS | `npm run preflight:js` | Workbench lint + test | `package.json:7` |
| Bootstrap macOS | `./scripts/bootstrap_macos.sh` | Installs Homebrew, Node, Rust | `scripts/bootstrap_macos.sh` |
| Build DMG | `./scripts/build_dmg.sh` | CLI + Tauri bundle | `scripts/build_dmg.sh` |
| Dev Mode | `./scripts/dev.sh` | Validates protocol + starts Tauri dev | `scripts/dev.sh` |

### Rust Workspace

| Intent | Command | Notes | Source |
| -------- | ------- | ----- | ------ |
| Build CLI | `cargo build --release -p hqe` | Output: `target/release/hqe` | `README.md:163` |
| Test | `cargo test --workspace` | All workspace crates | `.github/workflows/ci.yml:89` |
| Test SQLCipher | `cargo test --workspace --features sqlcipher-tests` | Requires SQLCipher lib | `hqe-core/Cargo.toml` |
| Check | `cargo clippy --workspace -- -D warnings` | Lint | `.github/workflows/ci.yml:95` |
| Format | `cargo fmt --all -- --check` | Format check | `.github/workflows/ci.yml:92` |

### Workbench UI (desktop/workbench)

| Intent | Command | Notes | Source |
| -------- | ------- | ----- | ------ |
| Dev (Vite) | `npm run dev` | Port 1420 | `package.json:7` |
| Dev (Tauri) | `npm run tauri:dev` | Desktop app dev mode | `package.json:11` |
| Build | `npm run build` | TypeScript + Vite | `package.json:8` |
| Build Tauri | `npm run tauri:build` | Desktop bundle | `package.json:12` |
| Lint | `npm run lint` | ESLint | `package.json:13` |
| Test | `npm run test` | Vitest | `package.json:14` |

### Protocol Validation

```bash
# Validate HQE Protocol against schema
./scripts/validate_protocol.sh

# Or via CLI (with fallback)
./target/release/hqe validate-protocol
```

## Code Style Guidelines

### Rust

- **Errors**: Use `thiserror` for libraries, `anyhow` for CLI/binaries
- **Async**: Prefer `tokio` for async runtime
- **Docs**: All public functions must have doc comments (`///`)
- **Formatting**: Use `cargo fmt` with default configuration
- **Linting**: Zero warnings policy with `cargo clippy -- -D warnings`

### TypeScript / React

- **Functional Components**: Use `React.FC` or simple functions
- **State**: Use `zustand` for global state management
- **Styling**: Use Tailwind CSS (configured in `index.css`)
- **No `any`**: TypeScript strict mode is enabled
- **Testing**: Vitest for unit tests, React Testing Library for component tests

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat(core): add new scanner`
- `fix(ui): resolve button alignment`
- `docs: update readme`
- `security: fix SQL injection vulnerability`

## Testing Instructions

### Rust Tests

```bash
# Run all workspace tests
cargo test --workspace

# Run with SQLCipher tests (requires library installed)
cargo test --workspace --features sqlcipher-tests

# Run tests for specific crate
cargo test -p hqe-core
```

### JavaScript/TypeScript Tests

```bash
cd desktop/workbench

# Install dependencies
npm install

# Run unit tests
npm test

# Run linter
npm run lint
```

### Integration Testing

```bash
# Build and test CLI
./scripts/bootstrap_macos.sh
cargo build --release -p hqe
./target/release/hqe --version
./target/release/hqe validate-protocol

# Test scan on sample repo
./target/release/hqe scan ./tests/fixtures/sample-repo --local-only
```

### Pre-commit Hooks

Install pre-commit hooks for automated checks:

```bash
pip install pre-commit
pre-commit install
```

Hooks include:

- Trailing whitespace removal
- JSON/YAML/TOML validation
- Secret scanning (Gitleaks)
- Rust formatting and clippy
- Python formatting (Black, Ruff)
- TypeScript/ESLint checks
- Markdown linting
- Spelling checks (codespell)

## Security Considerations

### Implemented Protections

| Threat | Mitigation | Status | Implementation |
| ------ | ---------- | ------ | -------------- |
| XSS via LLM output | DOMPurify with strict allowlist | ✅ Implemented | `ConversationPanel.tsx` |
| SQL Injection | Parameterized queries, path validation | ✅ Implemented | `encrypted_db.rs` |
| Prompt Injection | Key validation, delimiter protection | ✅ Implemented | `prompts.rs` |
| Jailbreak attempts | Unicode normalization, 50+ patterns | ✅ Implemented | `system_prompt.rs` |
| Data exfiltration | Encrypted database (SQLCipher AES-256) | ✅ Implemented | `hqe-core::encrypted_db` |
| Race conditions | Ref-based state, atomic updates | ✅ Implemented | `store.ts` |
| Directory traversal | Path canonicalization | ✅ Implemented | `encrypted_db.rs` |

### Chat Security

- **Encryption**: SQLCipher AES-256 with PBKDF2 key derivation
- **Key Storage**: macOS Keychain (Secure Enclave)
- **No Plaintext**: Chat history never written unencrypted
- **API Key Isolation**: Provider keys stored separately from chat DB

### Secret Management

- API keys stored in macOS Keychain via `keyring` crate
- Secret redaction before LLM transmission
- Gitleaks pre-commit hook for secret detection
- No secrets in logs or error messages

### Reporting Vulnerabilities

- **Email**: <2-craze-headmen@icloud.com>
- **Subject**: `HQE Workbench Security Report`
- **Do not** open public GitHub issues for vulnerabilities

See [SECURITY.md](SECURITY.md) for full policy.

## CI/CD Pipeline

### GitHub Actions Workflows

| Event | Workflow | Actions |
| ----- | -------- | ------- |
| Push/PR to main | `.github/workflows/ci.yml` | Secret scan, build, test, clippy, fmt, JS lint/test |
| Weekly schedule | `.github/workflows/security.yml` | `cargo audit` |
| Release | `.github/workflows/release.yml` | Build macOS universal binary, create DMG |
| Stale issues | `.github/workflows/stale.yml` | Mark and close stale issues/PRs |

### CI Jobs (ci.yml)

1. **secret-scan**: Gitleaks secret detection
2. **test-rust**: Build, test, format check, clippy
3. **test-cli**: Build CLI and test commands
4. **lint-js**: ESLint on TypeScript/React code
5. **test-js**: Vitest unit tests
6. **build-macos**: Build universal binary and Tauri app (macOS only)

### Release Build

```bash
# macOS Universal Binary (ARM64 + x86_64)
./scripts/build_dmg.sh

# Output location
target/release/bundle/**/*.dmg
target/release/bundle/**/*.app
target/release/hqe
```

## Environment Variables

| Variable | Required | Purpose | Source |
| -------- | -------- | ------- | ------ |
| `HQE_OPENAI_TIMEOUT_SECONDS` | No | Override LLM timeout | `crates/hqe-openai/src/lib.rs` |
| `HQE_PROMPTS_DIR` | No | Custom prompts directory | `desktop/workbench/src-tauri/src/prompts.rs` |
| `HQE_CHAT_DB_PATH` | No | Custom chat database path | `crates/hqe-core/src/encrypted_db.rs` |
| `VENICE_API_KEY` | No | Venice.ai API key | Provider config |
| `OPENAI_API_KEY` | No | OpenAI API key | Provider config |

## Critical Files

| Path | Purpose |
| ---- | ------- |
| `Cargo.toml` | Workspace configuration |
| `cli/hqe/src/main.rs` | CLI implementation |
| `desktop/workbench/src-tauri/src/lib.rs` | Tauri commands, AppState |
| `desktop/workbench/src-tauri/src/chat.rs` | Chat persistence commands |
| `desktop/workbench/src-tauri/src/prompts.rs` | Prompt execution |
| `protocol/hqe-engineer.yaml` | HQE Protocol v3 definition |
| `protocol/hqe-schema.json` | Protocol JSON schema |
| `scripts/bootstrap_macos.sh` | Environment setup |
| `desktop/workbench/vite.config.ts` | Vite configuration |
| `desktop/workbench/tailwind.config.js` | Tailwind theme |
| `docs/COMPREHENSIVE_TODO_AND_BUGS.md` | Security audit & TODOs |

## Architecture

| Aspect | Details | Source |
| -------- | ------- | ------ |
| CLI Entry | `cli/hqe/src/main.rs` | `cli/hqe/Cargo.toml:14` |
| Workbench UI Entry | `desktop/workbench/src/main.tsx` | `vite.config.ts` |
| Tauri Backend | `desktop/workbench/src-tauri/src/lib.rs` | `Cargo.toml` |
| Protocol Schema | `protocol/hqe-engineer.yaml` | Embedded in CLI binary |
| State Management | Zustand 5.0 | `desktop/workbench/package.json` |
| Styling | Tailwind CSS 4.1 | `desktop/workbench/package.json` |
| Build Tool | Vite 7 | `desktop/workbench/package.json` |
| Framework | React 19, Tauri 2.0 | `desktop/workbench/package.json` |
| Key Dependencies | Tokio, Clap, Serde, Reqwest | `Cargo.toml:27-48` |
| Security | DOMPurify, SQLCipher, Keyring | `package.json`, `Cargo.toml` |

## Project Structure

```text
hqe-workbench/
├── .github/             # CI/CD and Issue Templates
├── cli/
│   └── hqe/             # CLI Application Entry Point
├── crates/              # Rust workspace crates
│   ├── hqe-core/        # Core scan pipeline + encrypted chat
│   ├── hqe-flow/        # Workflow & Protocol Execution
│   ├── hqe-git/         # Git Operations
│   ├── hqe-ingest/      # Repository Ingestion & File Watching
│   ├── hqe-mcp/         # Model Context Protocol
│   ├── hqe-openai/      # AI Provider Client
│   ├── hqe-protocol/    # Schema & Type Defs
│   └── hqe-vector/      # Vector Database Operations (placeholder)
├── desktop/
│   └── workbench/       # Desktop App (Tauri/React)
│       ├── src/         # React + TypeScript UI
│       └── src-tauri/   # Rust backend
├── docs/                # Architecture & Guides
├── mcp-server/          # Thinktank Prompt Library & MCP Server
├── prompts/             # Prompt Examples & Guidance (symlink)
├── protocol/            # HQE Protocol Schemas
├── scripts/             # Build & Test Scripts
└── tests/               # Test fixtures
```

## Provider Integration

HQE Workbench supports OpenAI-compatible chat completion providers (text models only):

- **Venice.ai** (with Venice extensions)
- **OpenAI**
- **Anthropic** (via OpenAI compatibility)
- **Local OpenAI-schema servers** (LM Studio / LocalAI / Ollama gateways)

### Model Discovery

Use the desktop Settings screen or CLI to discover available models from provider `/models` endpoints.

## Documentation References

- [Architecture](docs/architecture.md) - System design and module boundaries
- [Development](docs/DEVELOPMENT.md) - Local development guide
- [How-To Guide](docs/HOW_TO.md) - Installation and usage
- [Build Instructions](BUILD.md) - Detailed build process
- [API Reference](docs/API.md) - Provider API documentation
- [HQE Protocol v3](protocol/README.md) - Protocol specification
- [Security Audit](docs/COMPREHENSIVE_TODO_AND_BUGS.md) - Security findings and TODOs
- [Tech Stack](docs/tech-stack.md) - Technology choices

## Verification

```bash
# Verify Rust workspace
cargo test --workspace && cargo clippy --workspace

# Verify with SQLCipher tests (requires library installed)
cargo test --workspace --features sqlcipher-tests

# Verify Workbench
cd desktop/workbench && npm install && npm run lint && npm test

# Full preflight
npm run preflight

# Run CLI
./target/release/hqe scan ./example-repo --local-only

# Run desktop app
cd desktop/workbench && npm run tauri:dev
```

## Troubleshooting

### Common Issues

#### "No module named 'yaml'"

```bash
pip3 install pyyaml jsonschema
```

#### "command not found: cargo"

```bash
source $HOME/.cargo/env
```

#### "No such file: tauri.conf.json"

```bash
cd desktop/workbench && npm install
```

#### macOS Gatekeeper blocks app

```bash
xattr -cr /Applications/HQE\ Workbench.app
```

---

**Last validated:** 2026-02-06

## Verification Checklist

- [x] All commands tested locally
- [x] All file paths verified to exist
- [x] No hallucinated dependencies
- [x] Environment variables match source code
- [x] Security features documented
- [x] Chat functionality verified
- [x] Documentation references accurate
