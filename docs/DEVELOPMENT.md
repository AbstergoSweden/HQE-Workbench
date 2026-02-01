# Development Guide

This guide is for developing HQE Workbench locally (CLI + desktop app).

## Quick Start (macOS)

```bash
./scripts/bootstrap_macos.sh
./scripts/dev.sh
```

`./scripts/dev.sh` validates the protocol and then starts the Tauri dev app.

## Key Commands

Root:

```bash
# Run the local CI-equivalent checks
npm run preflight

# Rust-only checks
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

Workbench UI:

```bash
cd desktop/workbench
npm run lint
npm test
npm run tauri:dev
```

Release build (macOS DMG):

```bash
./scripts/build_dmg.sh
```

## Repository Layout (What Lives Where)

- `cli/hqe/`: CLI entry point (`hqe`).
- `desktop/workbench/`: Tauri + React desktop app.
- `crates/`: core libraries (pipeline, provider client, artifacts, git).
- `protocol/`: protocol YAML + JSON schema.
- `prompts/`: prompt library and the MCP prompts server.

## Provider Integration (Text Models Only)

Provider support is OpenAI-schema based:

- Venice.ai (OpenAI-compatible with Venice extensions)
- OpenAI
- Local OpenAI-schema servers (LM Studio / LocalAI / Ollama gateways, etc.)

See `docs/API.md` and `docs/swagger.yaml` (Venice spec).

## CI Expectations

GitHub Actions runs roughly the same checks as `npm run preflight`:

- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt -- --check`
- `cd desktop/workbench && npm run lint`
- `cd desktop/workbench && npm test`

Protocol validation uses `./scripts/validate_protocol.sh` and requires Python deps (`pyyaml`,
`jsonschema`).
