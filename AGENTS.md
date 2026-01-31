# hqe-workbench Context
> **Auto-Generated:** 2026-01-30 | **Sources:** `Cargo.toml`, `package.json`, `apps/workbench/package.json`, `apps/workbench/src-tauri/Cargo.toml`, `apps/workbench/src-tauri/tauri.conf.json`, `apps/workbench/vite.config.ts`, `scripts/*.sh`, `docs/architecture.md`, `prompts/prompts/server/package.json`, `protocol/*`

## 1. System Identity
| Attribute | Value | Source |
| :--- | :--- | :--- |
| **Primary Languages** | Rust (edition 2021, rust-version 1.75), TypeScript (5.3 workbench, 5.9 prompts server), JavaScript, Python 3 (protocol validation) | `Cargo.toml`, `apps/workbench/package.json`, `prompts/prompts/server/package.json`, `scripts/validate_protocol.sh` |
| **Runtimes** | Rust (Tokio), Node.js (Workbench UI + Prompts server; prompts server requires >=18.18.0), Python 3 (protocol validation) | `Cargo.toml`, `prompts/prompts/server/package.json`, `scripts/validate_protocol.sh` |
| **Desktop UI** | Tauri 2.0.0 + React 18.2.0 | `apps/workbench/src-tauri/Cargo.toml`, `apps/workbench/package.json` |
| **Frontend Tooling** | Vite 5, TypeScript, Tailwind CSS 3.3, Zustand 4.4 | `apps/workbench/package.json` |
| **Backend/Infra** | Cargo workspace, Clap CLI, Tokio async runtime | `Cargo.toml`, `cli/hqe/Cargo.toml` |
| **Prompt Server** | Express + @modelcontextprotocol/sdk (Claude prompts MCP server) | `prompts/prompts/server/package.json` |

## 2. Repository Layout
- `apps/workbench/`: React + Vite frontend for the Tauri desktop app.
- `apps/workbench/src-tauri/`: Rust Tauri backend, command bridge, and app config.
- `cli/hqe/`: Rust CLI entry point (`hqe`).
- `crates/`: Rust workspace libraries (core engine, providers, artifacts, MCP, ingest, vector, flow).
- `protocol/`: HQE Engineer protocol YAML, schemas, and validation scripts.
- `prompts/prompts/`: Claude prompts MCP server (Node/TypeScript) plus resources/docs.
- `hqe_workbench_provider_discovery/`: Standalone provider discovery patch workspace.
- `example-repo/`: Sample repository used for scan demos/testing.

## 3. Operational Commands
*Execute these exactly as shown.*
| Scope | Intent | Command | Context/Notes | Source |
| :--- | :--- | :--- | :--- | :--- |
| **Root** | **Bootstrap (macOS)** | `./scripts/bootstrap_macos.sh` | Installs toolchains + deps | `scripts/bootstrap_macos.sh` |
| **Root** | **Dev (Workbench)** | `./scripts/dev.sh` | Runs protocol validation then `tauri:dev` | `scripts/dev.sh` |
| **Root** | **Build (DMG)** | `./scripts/build_dmg.sh` | Builds CLI + Tauri bundle | `scripts/build_dmg.sh` |
| **Root** | **Validate Protocol** | `./scripts/validate_protocol.sh` | Python YAML schema validation | `scripts/validate_protocol.sh` |
| **Root** | **Source Zip** | `./scripts/make_source_zip.sh` | Creates source-only archive | `scripts/make_source_zip.sh` |
| **Root** | **Preflight** | `npm run preflight` | Rust tests/clippy/fmt + Workbench lint/test | `package.json` |
| **Rust** | **Test (Workspace)** | `cargo test --workspace` | Rust unit/integration | `scripts/build_dmg.sh`, `package.json` |
| **Rust** | **Build CLI** | `cargo build --release -p hqe` | CLI binary | `scripts/build_dmg.sh`, `scripts/bootstrap_macos.sh` |
| **Workbench** | **Dev (Vite)** | `cd apps/workbench && npm run dev` | Vite dev server | `apps/workbench/package.json` |
| **Workbench** | **Dev (Tauri)** | `cd apps/workbench && npm run tauri:dev` | Desktop app dev | `apps/workbench/package.json` |
| **Workbench** | **Build (UI)** | `cd apps/workbench && npm run build` | TS + Vite build | `apps/workbench/package.json` |
| **Workbench** | **Build (Tauri)** | `cd apps/workbench && npm run tauri:build` | Desktop bundle | `apps/workbench/package.json` |
| **Workbench** | **Lint** | `cd apps/workbench && npm run lint` | ESLint | `apps/workbench/package.json` |
| **Workbench** | **Test** | `cd apps/workbench && npm run test` | Vitest | `apps/workbench/package.json` |
| **Prompts** | **Dev** | `cd prompts/prompts/server && npm run dev` | Hot reload server | `prompts/prompts/server/package.json` |
| **Prompts** | **Build** | `cd prompts/prompts/server && npm run build` | TypeScript build | `prompts/prompts/server/package.json` |
| **Prompts** | **Lint** | `cd prompts/prompts/server && npm run lint` | ESLint | `prompts/prompts/server/package.json` |
| **Prompts** | **Test** | `cd prompts/prompts/server && npm run test` | Jest (unit) | `prompts/prompts/server/package.json` |

## 4. Entry Points & Config
- **Workbench UI Entry:** `apps/workbench/src/main.tsx`
- **Tauri App Entry:** `apps/workbench/src-tauri/src/main.rs` (launches `hqe_workbench_app::run()` in `apps/workbench/src-tauri/src/lib.rs`)
- **CLI Entry:** `cli/hqe/src/main.rs`
- **Prompts Server Entry:** `prompts/prompts/server/src/index.ts` (compiled to `dist/index.js`)
- **Protocol Schema:** `protocol/hqe-engineer.yaml` + `protocol/hqe-schema.json`
- **Workbench Config:** `apps/workbench/vite.config.ts`, `apps/workbench/src-tauri/tauri.conf.json`

## 5. Testing Strategy
- **Rust:** `cargo test --workspace` (workspace crates + CLI)
- **Workbench UI:** Vitest via `npm run test` in `apps/workbench`
- **Prompts Server:** Jest via `npm run test` in `prompts/prompts/server`

## 6. Environment Variables (.env)
*No `.env` or `.env.example` files found in this repo.*

## 7. Critical Files Map
- `Cargo.toml`: Rust workspace configuration and dependency versions.
- `apps/workbench/package.json`: Frontend dependencies + scripts.
- `apps/workbench/src-tauri/Cargo.toml`: Tauri backend dependencies.
- `apps/workbench/src-tauri/tauri.conf.json`: Tauri build/runtime config.
- `cli/hqe/Cargo.toml`: CLI crate config.
- `protocol/hqe-engineer.yaml`: HQE Engineer protocol source.
- `protocol/hqe-schema.json`: Protocol JSON schema used by validator.
- `scripts/dev.sh`: Dev entrypoint (protocol validation + Tauri dev).
- `scripts/build_dmg.sh`: Release build pipeline.
- `prompts/prompts/server/package.json`: MCP prompt server scripts + deps.
