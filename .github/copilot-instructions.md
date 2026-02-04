# GitHub Copilot Instructions for HQE Workbench

## Project Overview

HQE Workbench is a local-first macOS desktop application and CLI tool for running the HQE (High Quality Engineering) Engineer Protocol. It automates codebase health auditing, security scanning, and technical leadership tasks using a combination of local heuristics and LLM-powered analysis.

**Key characteristics:**
- **Hybrid Rust/TypeScript monorepo** - Rust for CLI and backend, TypeScript/React for desktop UI
- **Tauri v2 desktop app** - Native macOS application with web frontend
- **Security-first design** - Local-only mode, secret redaction, keychain storage
- **Protocol-driven** - Implements HQE Protocol v4.2.1 (YAML schema in `protocol/`)
- **macOS only** - Development and releases target macOS 12.0+
- **Agent Prompts MCP server** - Node/TypeScript MCP server + prompt library in `mcp-server/`

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

# Run Rust tests with SQLCipher (requires library installed)
cargo test --workspace --features sqlcipher-tests

# Run single Rust crate tests
cargo test -p hqe-core
cargo test -p hqe-openai

# Run a single Rust test by name (substring match)
cargo test -p hqe-core -- redaction

# Run a single Rust integration test file
cargo test -p hqe-core --test <test_file_stem> -- <test_name_substring>

# Run Workbench frontend tests (Vitest)
cd desktop/workbench && npm test

# Run a single Workbench test by name (Vitest)
cd desktop/workbench && npm test -- -t "<test name substring>"

# Run prompts MCP server tests (Jest)
cd mcp-server/prompts/server && npm test

# Run a single prompts MCP server unit test
cd mcp-server/prompts/server && npm run test:unit -- tests/unit/<name>.test.ts -t "<test name substring>"
```

### Linting and Formatting
```bash
# Rust linting (fail on warnings)
cargo clippy --workspace -- -D warnings

# Rust formatting check
cargo fmt --all -- --check

# Rust formatting apply
cargo fmt --all

# TypeScript linting (from desktop/workbench)
cd desktop/workbench && npm run lint

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

### CI Invariants
```bash
# CI enforces a small set of architectural invariants (run this when touching LLM/provider plumbing)
bash scripts/verify_invariants.sh
```

### Protocol Validation
```bash
# Validate protocol YAML against JSON schema (auto-installs pyyaml/jsonschema if missing)
./scripts/validate_protocol.sh

# Or via the CLI
cargo build --release -p hqe
./target/release/hqe validate-protocol
```

### MCP server (mcp-server/prompts/server)
```bash
cd mcp-server/prompts/server

# Build/typecheck
npm run build
npm run typecheck

# Lint
npm run lint

# Tests
npm test
npm run test:unit -- tests/unit/<name>.test.ts

# Validate MCP server resolves prompts/gates/etc
MCP_WORKSPACE="$(pwd)/resources" node dist/index.js --startup-test --verbose
```

### GitHub MCP Server (Cursor)

Cursor supports GitHub's official MCP server. **Do not commit your PAT** — add GitHub MCP to your *global* Cursor config at `~/.cursor/mcp.json`.

Remote (recommended):
```json
{
  "mcpServers": {
    "github": {
      "url": "https://api.githubcopilot.com/mcp/",
      "headers": {
        "Authorization": "Bearer YOUR_GITHUB_PAT"
      }
    }
  }
}
```

Local (Docker):
```json
{
  "mcpServers": {
    "github-local": {
      "command": "docker",
      "args": [
        "run",
        "-i",
        "--rm",
        "-e",
        "GITHUB_PERSONAL_ACCESS_TOKEN",
        "ghcr.io/github/github-mcp-server"
      ],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "YOUR_GITHUB_PAT"
      }
    }
  }
}
```

## Architecture

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

#### `hqe-mcp` + `hqe-flow`
Thinktank prompt library and protocol workflow execution.

#### Agent Prompts MCP server (`mcp-server/prompts/server`)
Node/TypeScript MCP server used for prompt management; prompt catalogs live in `mcp-server/cli-prompt-library` and `mcp-server/cli-security`.

#### Tauri app
React UI in `desktop/workbench/src` calls Rust commands in `desktop/workbench/src-tauri` to access the scan pipeline.

### Data Flow

1. **User Input** → CLI or Desktop UI
2. **Ingestion** → `hqe-core` scans repository files
3. **Redaction** → `RedactionEngine` strips secrets before any external calls
4. **Analysis** → Local heuristics or LLM provider (via `hqe-openai`)
5. **Reporting** → `hqe-artifacts` generates Markdown/JSON
6. **Export** → User receives report and manifest

### Frontend Architecture

- **Framework:** React + TypeScript (Tauri WebView)
- **State management:** Zustand (global state)
- **Styling:** Tailwind CSS
- **Build tool:** Vite
- **Entry point:** `desktop/workbench/src/main.tsx`

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

### LLM/Provider Call-Site Invariants (Tauri)

- **Only provider HTTP call site** lives in `desktop/workbench/src-tauri/src/llm.rs` (guarded by `scripts/verify_invariants.sh`).
- **Only model request builder** lives in `crates/hqe-core/src/prompt_runner.rs`.
- If you need new provider/network behavior, route it through these modules rather than adding new call sites.

### TypeScript/React Conventions

- **No `any` types** - TypeScript strict mode enabled
- **Functional components** - Use `React.FC` or simple functions, no class components
- **State management** - Use Zustand for global state, not Context API or Redux
- **Styling** - Tailwind CSS only, no inline styles or CSS modules
- **Imports** - Use absolute imports where possible

### Dependency Management

- **Workspace dependencies** - Shared deps defined in root `Cargo.toml` under `[workspace.dependencies]`
- **Version pinning** - Use workspace versions: `tokio = { workspace = true }`

### MCP Server Conventions

- **Node runtime** - `mcp-server/prompts/server` requires Node.js >= 18.18.
- **Contracts are generated** - Update `mcp-server/prompts/server/tooling/contracts/*.json`, then run `npm run generate:contracts`; do not edit `src/tooling/contracts/_generated` directly.
- **Operational rules** - Follow `mcp-server/prompts/MCP_SERVER_HANDBOOK.md` for prompt/gate/methodology workflows.

### Testing Patterns

- **Integration tests** - Place in `crates/*/tests/*.rs` (separate from `src/`)
- **Unit tests** - Place in same file as code under `#[cfg(test)] mod tests { ... }`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat(core): add new scanner`
- `fix(ui): resolve button alignment`
- `docs: update readme`
- `test(git): add patch application tests`

### Branch Naming

- `feat/` for features
- `fix/` for bug fixes
- `docs/` for documentation updates

## Protocol-Driven Development

The HQE Protocol (v4.2.1) is the source of truth for scan phases, report structure, and findings format.

- **Schema location:** `protocol/hqe-engineer.yaml`
- **Validation required:** Run `./scripts/validate_protocol.sh` before commits
- **Changes propagate:** Protocol changes require updates to Rust types in `crates/hqe-protocol/`
