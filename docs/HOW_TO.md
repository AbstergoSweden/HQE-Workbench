# How To: Install, Configure, and Use HQE Workbench

HQE Workbench ships as:

- a CLI (`hqe`) for scanning repositories and exporting run artifacts
- a macOS desktop app (Tauri + React) for configuring providers, running scans, viewing reports, and exporting artifacts

This guide focuses on the current app behavior (text models only; no image/audio/video).

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Install](#install)
3. [Run Your First Scan (Local-Only)](#run-your-first-scan-local-only)
4. [Add a Provider Profile (Venice/OpenAI/Local)](#add-a-provider-profile-veniceopenailocal)
5. [Run an LLM-Enabled Scan](#run-an-llm-enabled-scan)
6. [Use the Desktop App](#use-the-desktop-app)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

- macOS 12+ recommended (desktop app)
- Rust 1.75+
- Node.js 20+ (desktop UI)
- Python 3.11+ (protocol validation scripts)

## Install

```bash
git clone https://github.com/AbstergoSweden/HQE-Workbench.git
cd HQE-Workbench

./scripts/bootstrap_macos.sh
cargo build --release -p hqe
```

## Run Your First Scan (Local-Only)

```bash
./target/release/hqe scan . --local-only
```

Local-only mode:

- performs no external API calls
- runs local heuristics and produces a full report structure
- still writes the same artifact bundle as an LLM scan

Artifacts are written to `./hqe-output` by default:

```text
hqe-output/
  run-manifest.json
  report.json
  report.md
  session-log.json
  session-log.json
  redaction-log.json   (when redaction runs)
```

## Privacy & Caching

HQE Workbench implements a **Privacy-First Architecture** inspired by Venice.ai.

### Local Database

All interactions are logged locally to a SQLite database (`~/.local/share/hqe-workbench/hqe.db` on macOS). This typically includes:

- **Session Logs**: Audit trails of what was sent to the LLM (after redaction) and what was received.
- **Request Cache**: Hashed keys of prompts and responses.

### Semantic Caching

To save on API costs and improve speed, the app caches LLM responses by default. If you run the exact same scan or prompt again, the result is served locally from `hqe.db`.

To **disable caching** (e.g., if you changed a prompt template or want a fresh non-deterministic answer):

```bash
hqe scan ... --no-cache
# or
hqe prompt ... --no-cache
```

## Add a Provider Profile (Venice/OpenAI/Local)

HQE Workbench supports OpenAI-compatible chat completion providers. It filters out non-text
modalities and focuses on text models (including "code" text models where applicable).

You can add profiles via:

- CLI: `hqe config ...`
- Desktop app: Settings screen (recommended; includes model discovery)

### CLI Examples

Venice.ai (OpenAI-compatible, with Venice extensions):

```bash
export VENICE_API_KEY="..."
export VENICE_MODEL_ID="(copy from Settings > Discover Models)"

./target/release/hqe config add venice \
  --url "https://api.venice.ai/api/v1" \
  --key "$VENICE_API_KEY" \
  --model "$VENICE_MODEL_ID"
```

OpenAI:

```bash
./target/release/hqe config add openai \
  --url "https://api.openai.com/v1" \
  --key "sk-..." \
  --model "gpt-4o-mini"
```

Local OpenAI-schema server (LM Studio / LocalAI / gateway):

```bash
export LOCAL_MODEL_ID="(from your local server /v1/models)"

./target/release/hqe config add local \
  --url "http://127.0.0.1:1234/v1" \
  --key "local-not-required" \
  --model "$LOCAL_MODEL_ID"
```

Test a profile:

```bash
./target/release/hqe config test venice
```

### Model Discovery (Desktop App)

In Settings, use "Discover Models" to call the provider's `/models` endpoint and populate the model
dropdown. For Venice, discovery uses `/models?type=all` and filters down to text-capable models.

## Run an LLM-Enabled Scan

```bash
./target/release/hqe scan /path/to/repo --profile venice
```

Scan pipeline (high level):

```mermaid
flowchart TD
  A[Ingest repo] --> B[Local heuristics]
  B --> C[Redact secrets]
  C -->|optional| D[LLM analysis (text model)]
  D --> E[Generate report]
  E --> F[Write artifacts]
```

### Venice Advanced Options

The CLI supports passing Venice-specific advanced knobs:

```bash
./target/release/hqe scan /path/to/repo \
  --profile venice \
  --venice-parameters '{"some_option": true}' \
  --parallel-tool-calls false
```

## Use the Desktop App

Development mode:

```bash
./scripts/dev.sh
```

What to expect:

- Settings: create/edit provider profiles, store API keys in Keychain, run model discovery
- Scan: choose local-only vs LLM-enabled; for Venice profiles, advanced fields appear
- Report: view findings with evidence; export artifacts to a folder for sharing

## Troubleshooting

### Rust or Node not found

Re-run `./scripts/bootstrap_macos.sh` and restart your terminal session.

### Model discovery returns no models

- Verify base URL ends with `/v1` (OpenAI) or `/api/v1` (Venice).
- Confirm your API key works (`hqe config test ...`).
- Some providers require extra headers; use the desktop Settings screen to configure them.

### Protocol validation fails in CI

Protocol validation uses Python packages `pyyaml` and `jsonschema`.
