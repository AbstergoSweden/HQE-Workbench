# HQE-Workbench

![HQE Workbench banner](https://github.com/user-attachments/assets/1c762e08-59c0-4451-90cf-bdd474d933cd)

[![CI](https://github.com/AbstergoSweden/HQE-Workbench/actions/workflows/ci.yml/badge.svg)](https://github.com/AbstergoSweden/HQE-Workbench/actions/workflows/ci.yml)
[![Security](https://github.com/AbstergoSweden/HQE-Workbench/actions/workflows/security.yml/badge.svg)](https://github.com/AbstergoSweden/HQE-Workbench/actions/workflows/security.yml)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/AbstergoSweden/HQE-Workbench/badge)](https://securityscorecards.dev/viewer/?uri=github.com/AbstergoSweden/HQE-Workbench)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

A local-first macOS desktop application + CLI for running the HQE (High Quality Engineering) Engineer
Protocol. It automates codebase health auditing and produces actionable, evidence-backed TODOs using
a combination of local heuristics and (optional) LLM-powered analysis.

**New in v0.2.0:** Encrypted chat system, enhanced security hardening, and Thinktank prompt library with 30+ expert prompts.

## Table of Contents

- [Features](#features)
- [Overview](#overview)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Usage](#usage)
- [Security](#security)
- [Development](#development)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [Credits](#credits)
- [License](#license)

## Features

### Core Capabilities

- **Repository Scanning**: Automated codebase health auditing with local static analysis
- **Secret Redaction**: Intelligent detection and removal of sensitive data (API keys, tokens)
- **Local-Only Mode**: Privacy-first operation without external API calls
- **Semantic Caching**: Locally stores LLM responses in SQLite to reduce costs and latency
- **Report Generation**: Comprehensive Markdown and JSON reports with run manifests

### Chat & Conversation System (New)

- **🔒 Encrypted Chat**: SQLCipher AES-256 encryption for all chat history
- **💬 Unified Panel**: Seamless transition from reports to multi-turn conversations
- **📄 Message Pagination**: Efficient loading of large chat histories (100-1000 messages/page)
- **🔑 Secure Key Storage**: Encryption keys stored in macOS Keychain (Secure Enclave)
- **📝 Persistent Sessions**: Chat history survives app restarts

### Thinktank Prompt Library (New)

- **30+ Expert Prompts**: Security audits, code review, refactoring, documentation
- **🔍 Prompt Explanations**: Rich metadata with descriptions, inputs, and examples
- **🏷️ Category Filtering**: Browse by Security, Quality, Refactor, Test, Architecture
- **🤖 Provider Integration**: Works with OpenAI, Anthropic, Venice, OpenRouter, xAI, Kimi
- **🔄 Report → Chat**: Convert any analysis result into a chat session for follow-up

### Security Features

- **XSS Protection**: DOMPurify sanitization of all LLM output
- **SQL Injection Prevention**: Parameterized queries throughout
- **Prompt Injection Defense**: Key validation and delimiter protection
- **Jailbreak Detection**: 50+ pattern detection with Unicode normalization
- **Path Validation**: Canonicalization prevents directory traversal

## Overview

HQE Workbench is a hybrid Rust/Python/TypeScript application that provides:

- **Repository Scanning**: Automated codebase health auditing
- **Secret Redaction**: Intelligent detection and removal of sensitive data
- **Local-Only Mode**: Privacy-first operation without external API calls
- **Semantic Caching**: Locally stores LLM responses in SQLite to reduce costs and latency
- **Report Generation**: Comprehensive Markdown and JSON reports (plus run manifests and session logs)
- **Provider-agnostic LLM mode**: Any OpenAI-compatible chat completion API (text models only)
- **Encrypted Chat**: Private, persistent conversations with AI assistants

### File Structure

```text
hqe-workbench/
├── .github/             # CI/CD and Issue Templates
├── cli/
│   └── hqe/             # CLI Application Entry Point
├── crates/
│   ├── hqe-core/        # Scan Engine, Logic, Encrypted Chat DB
│   ├── hqe-flow/        # Workflow & Protocol Execution
│   ├── hqe-git/         # Git Operations
│   ├── hqe-ingest/      # Repository Ingestion & File Watching
│   ├── hqe-mcp/         # Model Context Protocol
│   ├── hqe-openai/      # AI Provider Client
│   ├── hqe-protocol/    # Schema & Type Defs
│   └── hqe-vector/      # Vector Database Operations
├── desktop/
│   └── workbench/       # Desktop App (Tauri/React)
├── docs/                # Architecture & Guides
├── mcp-server/          # Thinktank Prompt Library & MCP Server
├── prompts/             # Prompt Examples & Guidance
├── protocol/            # HQE Protocol Schemas
└── scripts/             # Build & Test Scripts
```

## Architecture

High-level architecture is documented in `docs/ARCHITECTURE.md`. The core idea:

- `hqe-core` runs the scan pipeline and manages encrypted chat storage.
- `hqe-openai` provides an OpenAI-compatible chat client (used for optional LLM analysis and Thinktank prompts).
- `hqe-artifacts` writes `run-manifest.json`, `report.json`, and `report.md`.
- `hqe-mcp` provides the Thinktank prompt library with rich metadata.

```mermaid
graph TB
    subgraph "HQE Workbench"
        CLI[CLI Entry Point<br/>Rust]
        Core[hqe-core<br/>Scan Pipeline + Encrypted Chat]
        Git[hqe-git<br/>Git Operations]
        OpenAI[hqe-openai<br/>LLM Client]
        Artifacts[hqe-artifacts<br/>Report Generation]
        MCP[hqe-mcp<br/>Thinktank Prompts]
        UI[Tauri Desktop App<br/>React + TypeScript]
    end
    
    User[User] -->|Commands| CLI
    User -->|GUI| UI
    CLI --> Core
    UI --> Core
    Core --> Git
    Core --> OpenAI
    Core --> Artifacts
    Core --> MCP
    UI --> MCP
    Git -->|Repository Data| Core
    OpenAI -->|Analysis| Core
    Artifacts -->|Reports| User
    
    style Core fill:#4a9eff
    style CLI fill:#ff6b6b
    style UI fill:#51cf66
    style MCP fill:#ffd93d
```

## Quick Start

### Prerequisites

- **macOS**: 12.0+ (Monterey)
- **Rust**: 1.75+
- **Python**: 3.11+ (used for protocol validation)
- **Node.js**: 20+ (Workbench UI)

### Installation

```bash
# Clone the repository
git clone https://github.com/AbstergoSweden/HQE-Workbench.git
cd HQE-Workbench

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

## Security

Security is a top priority for HQE Workbench. We implement defense-in-depth with multiple layers of protection:

### Implemented Protections

| Layer | Protection | Implementation |
|-------|------------|----------------|
| Input Validation | Template key validation, path canonicalization | `prompts.rs`, `encrypted_db.rs` |
| Output Sanitization | DOMPurify for LLM output | `ConversationPanel.tsx` |
| Database Security | SQLCipher AES-256 encryption | `encrypted_db.rs` |
| Key Management | macOS Keychain integration | `keyring` crate |
| Prompt Security | 50+ jailbreak pattern detection | `system_prompt.rs` |
| Injection Prevention | Parameterized SQL queries | `encrypted_db.rs` |

### Reporting Vulnerabilities

Please see our [Security Policy](SECURITY.md) for:

- Supported versions
- Vulnerability reporting process
- Security best practices

**To report a security vulnerability**, please email: <2-craze-headmen@icloud.com>

### Security Audit

A comprehensive security audit identifying 50+ issues (including 8 critical fixes) is available in:
- [`docs/COMPREHENSIVE_TODO_AND_BUGS.md`](docs/COMPREHENSIVE_TODO_AND_BUGS.md)

All critical security issues have been addressed in the v0.2.0 release.

## Development

```bash
# Run the full local CI-equivalent checks (Rust + Workbench)
npm run preflight

# Run Rust tests only
cargo test --workspace

# Run tests with SQLCipher (requires library installed)
cargo test --workspace --features sqlcipher-tests

# Run Workbench lint and tests
cd desktop/workbench && npm run lint && npm test
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Development](docs/DEVELOPMENT.md)
- [How-To Guide](docs/HOW_TO.md)
- [About the Project](ABOUT.md)
- [Legal & License](LEGAL.md)
- [Privacy](PRIVACY.md)
- [Support](SUPPORT.md)
- [API Reference](docs/API.md)
- [HQE Protocol v3](protocol/README.md)
- [Security Audit](docs/COMPREHENSIVE_TODO_AND_BUGS.md)

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on:

- Reporting bugs and requesting features
- Development setup and workflow
- Code style and testing requirements
- Pull request process

Please note that this project is released with a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to abide by its terms.

## Credits

- Venice.ai integration is supported via its OpenAI-compatible API interface. See `CREDITS.md` for details.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
