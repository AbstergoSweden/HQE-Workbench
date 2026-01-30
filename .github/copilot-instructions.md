# GitHub Copilot Instructions for HQE Workbench

## Project Overview

HQE Workbench is a local-first macOS desktop application and CLI tool for running the HQE (High Quality Engineering) Engineer Protocol. It automates codebase health auditing, security scanning, and technical leadership tasks using a combination of local heuristics and LLM-powered analysis.

**Key characteristics:**
- **Hybrid Rust/TypeScript monorepo** - Rust for CLI and backend, TypeScript/React for desktop UI
- **Tauri v2 desktop app** - Native macOS application with web frontend
- **Security-first design** - Local-only mode, secret redaction, keychain storage
- **Protocol-driven** - Implements HQE Protocol v4.2.1 (YAML schema in `protocol/`)

## Build, Test, and Lint

### Development Setup
```bash
# One-time bootstrap (macOS only)
./scripts/bootstrap_macos.sh

# Start development server (validates protocol, runs Tauri dev on port 1420)
./scripts/dev.sh
```

### Testing
```bash
# Run all Rust tests
cargo test --workspace

# Run single Rust crate tests
cargo test -p hqe-core
cargo test -p hqe-openai

# Run single test
cargo test --test integration_test test_name

# Run frontend tests (from apps/workbench)
cd apps/workbench && npm test

# Watch mode for frontend
cd apps/workbench && npm test -- --watch
```

### Linting and Formatting
```bash
# Rust linting (fail on warnings)
cargo clippy --workspace -- -D warnings

# Rust formatting check
cargo fmt --all -- --check

# Rust formatting apply
cargo fmt --all

# TypeScript linting (from apps/workbench)
cd apps/workbench && npm run lint

# Run all quality checks before committing
npm run preflight
```

### Building
```bash
# Build CLI in release mode
cargo build --release -p hqe
# Output: target/release/hqe

# Build desktop app DMG
./scripts/build_dmg.sh
# Output: target/release/bundle/dmg/

# Build all Rust workspace crates
cargo build --workspace
```

### Protocol Validation
```bash
# Validate protocol schema (required before commits)
./scripts/validate_protocol.sh
```

## Architecture

### Workspace Structure

```
hqe-workbench/
├── apps/workbench/          # Tauri v2 desktop app (React + TypeScript)
│   ├── src/                 # React frontend
│   ├── src-tauri/          # Tauri Rust backend
│   └── vite.config.ts      # Dev server on port 1420
├── cli/hqe/                # CLI entry point (Rust)
├── crates/                 # Shared Rust libraries
│   ├── hqe-core/           # Core scan pipeline, models, redaction
│   ├── hqe-openai/         # OpenAI-compatible LLM client
│   ├── hqe-git/            # Git operations wrapper
│   ├── hqe-artifacts/      # Report and manifest generation
│   ├── hqe-protocol/       # Protocol schema types
│   ├── hqe-mcp/            # Model Context Protocol
│   ├── hqe-ingest/         # Content ingestion
│   ├── hqe-vector/         # Vector operations
│   └── hqe-flow/           # Flow control
├── protocol/               # HQE Protocol v4.2.1 (YAML schemas)
├── scripts/                # Build and utility scripts
└── docs/                   # Architecture documentation
```

### Key Crates and Responsibilities

#### `hqe-core`
**The brain of the operation.** Core scan pipeline and data models.

- **Responsibilities:** Scan pipeline orchestration, content redaction engine, repository scanning, local heuristics
- **Key types:** `ScanPipeline`, `RedactionEngine`, `RepoScanner`, `HqeReport`, `RunManifest`
- **Entry point:** `crates/hqe-core/src/lib.rs`

#### `hqe-openai`
OpenAI-compatible LLM provider client.

- **Responsibilities:** HTTP client for chat completions, authentication, retry logic with exponential backoff, prompt templates
- **Key types:** `OpenAIClient`, `ProviderProfile`, `ChatRequest`, `ChatResponse`
- **Note:** Works with any OpenAI-compatible API (OpenAI, Anthropic, local models)

#### `hqe-git`
Git operations wrapper.

- **Responsibilities:** Repository detection, clone/status/branch operations, patch application, commit creation
- **Key types:** `GitRepo`
- **Note:** Shells out to system `git` binary

#### `hqe-artifacts`
Report and manifest generation.

- **Responsibilities:** Markdown report rendering, JSON serialization, file output management
- **Key types:** `ArtifactWriter`

### Data Flow

1. **User Input** → CLI or Desktop UI
2. **Ingestion** → `hqe-core` scans repository files
3. **Redaction** → `RedactionEngine` strips secrets before any external calls
4. **Analysis** → Local heuristics or LLM provider (via `hqe-openai`)
5. **Reporting** → `hqe-artifacts` generates Markdown/JSON
6. **Export** → User receives report and manifest

### Frontend Architecture

- **Framework:** React 18.2.0 with TypeScript 5.3.0
- **State management:** Zustand (global state)
- **Routing:** react-router-dom
- **Styling:** Tailwind CSS
- **Testing:** Vitest
- **Build tool:** Vite 5.0.0
- **Entry point:** `apps/workbench/src/main.tsx`

## Key Conventions

### Error Handling

- **Libraries (crates/*)**: Use `thiserror` for custom error types
- **Binaries (cli/hqe)**: Use `anyhow` for error propagation
- **Pattern:** Each crate exports a `pub enum XxxError` with `#[derive(Error, Debug)]`

Example from `hqe-core`:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HqeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Scan error: {0}")]
    Scan(String),
}
```

### Async Runtime

- **Always use `tokio`** for async runtime
- **Feature flags:** `tokio = { version = "1.35", features = ["full"] }`
- **Pattern:** Mark async functions with `#[tokio::main]` or `#[tokio::test]`

### Documentation

- **All public functions must have doc comments** (`///`)
- **Crate-level docs:** Use `//!` at the top of `lib.rs`
- **Warning lints enabled:**
  - `#![warn(missing_docs)]`
  - `#![warn(clippy::unwrap_used)]`
  - `#![warn(clippy::expect_used)]`

### Security and Privacy

- **Secret redaction is mandatory** - All content goes through `RedactionEngine` before external API calls
- **Keychain storage** - API keys stored in macOS Keychain, never in config files
- **Local-only mode** - Prioritized for sensitive repos; uses heuristics instead of LLMs
- **Patterns to detect:** AWS keys, API tokens, private keys (see `crates/hqe-core/src/redaction.rs`)

### TypeScript/React Conventions

- **No `any` types** - TypeScript strict mode enabled
- **Functional components** - Use `React.FC` or simple functions, no class components
- **State management** - Use Zustand for global state, not Context API or Redux
- **Styling** - Tailwind CSS only, no inline styles or CSS modules
- **Imports** - Use absolute imports where possible

### Dependency Management

- **Workspace dependencies** - Shared deps defined in root `Cargo.toml` under `[workspace.dependencies]`
- **Version pinning** - Use workspace versions: `tokio = { workspace = true }`
- **Security auditing** - `cargo audit` runs in CI

### Testing Patterns

- **Integration tests** - Place in `crates/*/tests/*.rs` (separate from `src/`)
- **Unit tests** - Place in same file as code under `#[cfg(test)] mod tests { ... }`
- **Test utilities** - Use `pretty_assertions` for readable diffs, `mockito` for HTTP mocks
- **Frontend tests** - Use Vitest, place alongside components as `*.test.tsx`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat(core): add new scanner`
- `fix(ui): resolve button alignment`
- `docs: update readme`
- `test(git): add patch application tests`

## Protocol-Driven Development

The HQE Protocol (v4.2.1) is the source of truth for scan phases, report structure, and findings format.

- **Schema location:** `protocol/hqe-engineer.yaml`
- **Validation required:** Run `./scripts/validate_protocol.sh` before commits
- **Changes propagate:** Protocol changes require updates to Rust types in `crates/hqe-protocol/`

## Common Tasks

### Adding a new scan heuristic
1. Add logic to `crates/hqe-core/src/scan.rs` or `crates/hqe-core/src/repo.rs`
2. Update `ScanPipeline` to call your heuristic
3. Add tests in `crates/hqe-core/tests/` or inline `#[cfg(test)]`
4. Validate output matches protocol schema

### Adding a new LLM provider
1. Implement client in `crates/hqe-openai/src/` (even if not OpenAI)
2. Add configuration to `ProviderProfile`
3. Update CLI args in `cli/hqe/src/main.rs` to accept new provider
4. Add integration test with `mockito` HTTP mocking

### Modifying the desktop UI
1. Edit React components in `apps/workbench/src/`
2. Use Zustand stores for state (see existing stores)
3. Run `cd apps/workbench && npm run lint` before committing
4. Test with `./scripts/dev.sh` (auto-reloads on save)

### Adding a new crate
1. Add to `Cargo.toml` workspace members: `members = ["crates/new-crate"]`
2. Create `crates/new-crate/Cargo.toml` with workspace dependencies
3. Export main types from `crates/new-crate/src/lib.rs`
4. Add to dependent crates via workspace path: `new-crate = { path = "crates/new-crate" }`

## Files to Check Before Modifying

- **Protocol changes:** `protocol/hqe-engineer.yaml` + run validation script
- **Core models:** `crates/hqe-core/src/models.rs` (used across all crates)
- **CLI arguments:** `cli/hqe/src/main.rs` (uses `clap` derive macros)
- **Frontend state:** `apps/workbench/src/stores/` (Zustand stores)
- **Build config:** Root `Cargo.toml` for Rust, `apps/workbench/package.json` for frontend

## Tauri-Specific Notes

- **Commands** - Rust functions exposed to frontend via `#[tauri::command]` in `apps/workbench/src-tauri/src/main.rs`
- **Permissions** - Configure in `apps/workbench/src-tauri/tauri.conf.json`
- **Plugins** - Tauri v2 uses plugins for fs, dialog, shell (see `apps/workbench/package.json` dependencies)
- **IPC** - Frontend calls backend via `@tauri-apps/api`: `import { invoke } from '@tauri-apps/api/core'`

## GitHub MCP Server Integration

The GitHub MCP server is configured and available in Copilot Chat for this repository.

**Available capabilities:**
- **Workflows:** List and inspect CI/Security workflows, check run status, download logs
- **Issues:** Search, read, and track issues (currently 0 open issues)
- **Pull Requests:** List, review, get diffs, check status and review comments
- **Commits:** View commit history, diffs, and file changes
- **Branches:** List branches and compare changes
- **Code Search:** Search across the repository codebase

**Example queries you can ask Copilot:**
- "Show me the latest CI workflow runs"
- "What failed in the last Security workflow?"
- "List recent commits on main branch"
- "Search for uses of RedactionEngine in the codebase"
- "Show me open pull requests"

**Repository details:**
- Owner: `AbstergoSweden`
- Repo: `HQE-Workbench`
- Active workflows: CI, Security

## Development Tips

- **Fast iteration:** Use `./scripts/dev.sh` for hot-reloading frontend + Rust backend
- **Debug logging:** Set `RUST_LOG=debug` to see tracing output
- **Protocol validation:** Auto-runs in `./scripts/dev.sh`, but explicitly run `./scripts/validate_protocol.sh` if modifying protocol
- **Preflight checks:** Always run `npm run preflight` before pushing (runs tests, linting, formatting checks)
- **macOS only:** This project targets macOS 12.0+; Linux/Windows support not planned
