# HQE-Workbench — Agent Context

> **Generated:** 2026-01-31 | **Sources:** `Cargo.toml`, `package.json`, `desktop/workbench/package.json`, `desktop/workbench/src-tauri/Cargo.toml`, `cli/hqe/Cargo.toml`, `.github/workflows/ci.yml`

## Quick Start

```bash
# Bootstrap macOS environment
./scripts/bootstrap_macos.sh

# Build CLI
cargo build --release -p hqe

# Run local-only scan
./target/release/hqe scan /path/to/repo --local-only
```

## System Identity

| Attribute | Value | Source |
|-----------|-------|--------|
| Language | Rust 2021 Edition | `Cargo.toml:19` |
| Rust Version | 1.75+ | `Cargo.toml:23` |
| TypeScript | 5.3+ (Workbench), 5.9+ (Prompts) | `desktop/workbench/package.json:45`, `prompts/prompts/server/package.json:143` |
| Python | 3.11+ (protocol validation) | `.github/workflows/ci.yml` |
| Platform | macOS 12.0+ | `README.md:108` |
| License | Apache-2.0 | `Cargo.toml:21` |

## Workspace Structure

> Cargo workspace with 14 members | `Cargo.toml:3-15`

| Package | Path | Purpose | Type |
|---------|------|---------|------|
| `hqe` | `cli/hqe` | CLI entry point | Binary |
| `hqe-workbench-app` | `desktop/workbench/src-tauri` | Tauri desktop app | Binary |
| `hqe-core` | `crates/hqe-core` | Scan pipeline & engine | Library |
| `hqe-openai` | `crates/hqe-openai` | OpenAI-compatible client | Library |
| `hqe-git` | `crates/hqe-git` | Git operations | Library |
| `hqe-artifacts` | `crates/hqe-artifacts` | Report generation | Library |
| `hqe-protocol` | `crates/hqe-protocol` | Schema definitions | Library |
| `hqe-mcp` | `crates/hqe-mcp` | Model Context Protocol | Library |
| `hqe-ingest` | `crates/hqe-ingest` | File ingestion | Library |
| `hqe-vector` | `crates/hqe-vector` | Vector/embeddings (placeholder) | Library |
| `hqe-flow` | `crates/hqe-flow` | Workflow engine | Library |

## Operational Commands

### Root Level

| Intent | Command | Notes | Source |
|--------|---------|-------|--------|
| Preflight | `npm run preflight` | Rust tests + JS lint/test | `package.json:5` |
| Preflight Rust | `npm run preflight:rust` | Tests, clippy, fmt check | `package.json:6` |
| Preflight JS | `npm run preflight:js` | Workbench lint + test | `package.json:7` |
| Bootstrap macOS | `./scripts/bootstrap_macos.sh` | Installs Homebrew, Node, Rust | `scripts/bootstrap_macos.sh` |
| Build DMG | `./scripts/build_dmg.sh` | CLI + Tauri bundle | `scripts/build_dmg.sh` |

### Rust Workspace

| Intent | Command | Notes | Source |
|--------|---------|-------|--------|
| Build CLI | `cargo build --release -p hqe` | Output: `target/release/hqe` | `README.md:124` |
| Test | `cargo test --workspace` | All workspace crates | `.github/workflows/ci.yml` |
| Check | `cargo clippy --workspace -- -D warnings` | Lint | `.github/workflows/ci.yml` |
| Format | `cargo fmt --all -- --check` | Format check | `.github/workflows/ci.yml` |

### Workbench UI (desktop/workbench)

| Intent | Command | Notes | Source |
|--------|---------|-------|--------|
| Dev (Vite) | `npm run dev` | Port 1420 | `package.json:7` |
| Dev (Tauri) | `npm run tauri:dev` | Desktop app dev mode | `package.json:11` |
| Build | `npm run build` | TypeScript + Vite | `package.json:8` |
| Build Tauri | `npm run tauri:build` | Desktop bundle | `package.json:12` |
| Lint | `npm run lint` | ESLint | `package.json:13` |
| Test | `npm run test` | Vitest | `package.json:14` |

### Prompts Server (prompts/prompts/server)

| Intent | Command | Notes | Source |
|--------|---------|-------|--------|
| Dev | `npm run dev` | Watch mode | `package.json:43` |
| Build | `npm run build` | TypeScript compile | `package.json:29` |
| Test | `npm run test` | Jest | `package.json:53` |
| Lint | `npm run lint` | ESLint | `package.json:35` |
| Type Check | `npm run typecheck` | TSC + validation | `package.json:33` |

## Architecture

| Aspect | Details | Source |
|--------|---------|--------|
| CLI Entry | `cli/hqe/src/main.rs` | `cli/hqe/Cargo.toml:14` |
| Workbench UI Entry | `desktop/workbench/src/main.tsx` | `vite.config.ts` |
| Tauri Backend | `desktop/workbench/src-tauri/src/lib.rs` | `Cargo.toml` |
| Protocol Schema | `protocol/hqe-engineer.yaml` | Embedded in CLI binary |
| State Management | Zustand 4.4 | `desktop/workbench/package.json:27` |
| Styling | Tailwind CSS 3.3 | `desktop/workbench/package.json:44` |
| Build Tool | Vite 5 | `desktop/workbench/package.json:47` |
| Framework | React 18.2, Tauri 2.0 | `desktop/workbench/package.json:21,31` |
| Key Dependencies | Tokio, Clap, Serde, Reqwest | `Cargo.toml:27-48` |

## Environment Variables

| Variable | Required | Purpose | Source |
|----------|----------|---------|--------|
| `HQE_OPENAI_TIMEOUT_SECONDS` | No | Override LLM timeout | `crates/hqe-openai/src/lib.rs` |
| `HQE_PROMPTS_DIR` | No | Custom prompts directory | `desktop/workbench/src-tauri/src/prompts.rs` |

## Critical Files

| Path | Purpose |
|------|---------|
| `Cargo.toml` | Workspace configuration |
| `cli/hqe/src/main.rs` | CLI implementation |
| `desktop/workbench/src-tauri/src/lib.rs` | Tauri commands |
| `protocol/hqe-engineer.yaml` | HQE Protocol v3 definition |
| `protocol/hqe-schema.json` | Protocol JSON schema |
| `scripts/bootstrap_macos.sh` | Environment setup |
| `desktop/workbench/vite.config.ts` | Vite configuration |
| `desktop/workbench/tailwind.config.js` | Tailwind theme |

## CI/CD

| Event | Workflow | Actions |
|-------|----------|---------|
| Push/PR to main | `.github/workflows/ci.yml` | Build, test, clippy, fmt |
| Weekly schedule | `.github/workflows/security.yml` | cargo audit |

## Verification

```bash
# Verify Rust workspace
cargo test --workspace && cargo clippy --workspace

# Verify Workbench
cd desktop/workbench && npm install && npm run lint && npm test

# Full preflight
npm run preflight

# Run CLI
./target/release/hqe scan ./example-repo --local-only
```

**Last validated:** 2026-01-31

---

## Verification Checklist

- [ ] All commands tested locally
- [ ] All file paths verified to exist
- [ ] No hallucinated dependencies
- [ ] Environment variables match source code
