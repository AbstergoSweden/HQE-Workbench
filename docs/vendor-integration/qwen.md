# HQE Workbench - Project Context

## Overview

HQE Workbench is a local-first macOS desktop application and CLI tool for running the HQE (High Quality Engineering) Engineer Protocol. It automates codebase health auditing and produces actionable, evidence-backed TODOs using a combination of local heuristics and (optional) LLM-powered analysis.

**Version**: 0.2.0  
**License**: Apache-2.0  
**Maintainer**: Faye Håkansdotter <2-craze-headmen@icloud.com>

## Architecture

The project is a hybrid Rust/Python/TypeScript application with the following key components:

### Core Crates (Rust)

- `hqe-core`: Core scan pipeline, redaction, and models
- `hqe-openai`: OpenAI-compatible client for LLM integration
- `hqe-git`: Git operations wrapper
- `hqe-artifacts`: Report generation (Markdown/JSON)
- `hqe-protocol`: Protocol schemas and type definitions
- `hqe-mcp`: Model Context Protocol and Thinktank prompts
- `hqe-ingest`: Repository ingestion and file watching
- `hqe-vector`: Vector database operations
- `hqe-flow`: Workflow and protocol execution

### Applications

- **CLI**: `cli/hqe` - Command-line interface for scanning and analysis
- **Desktop App**: `desktop/workbench` - Tauri-based macOS application with React/TypeScript frontend

### Protocol

- `protocol/` - HQE Engineer Protocol v3 definition (YAML) and JSON schema
- Includes Python and Haskell validators

## Key Features

### Core Capabilities

- Repository scanning with automated codebase health auditing
- Secret redaction and intelligent detection of sensitive data
- Local-only mode for privacy-first operation
- Semantic caching with SQLite to reduce LLM costs and latency
- Comprehensive Markdown and JSON report generation

### Chat & Conversation System

- SQLCipher AES-256 encrypted chat history
- Unified panel for seamless transition from reports to conversations
- Message pagination for efficient loading
- Secure key storage in macOS Keychain

### Thinktank Prompt Library

- 30+ expert prompts for security audits, code review, refactoring
- Category filtering (Security, Quality, Refactor, Test, Architecture)
- Provider integration (OpenAI, Anthropic, Venice, OpenRouter, xAI, Kimi)

### Security Features

- XSS protection with DOMPurify
- SQL injection prevention with parameterized queries
- Prompt injection defense with key validation
- Jailbreak detection with 50+ patterns and Unicode normalization
- Path validation to prevent directory traversal

## Building and Running

### Prerequisites

- macOS 12.0+ (Monterey)
- Rust 1.75+
- Python 3.11+
- Node.js 20+

### Installation

```bash
# Clone the repository
git clone https://github.com/AbstergoSweden/HQE-Workbench.git
cd HQE-Workbench

# Bootstrap the environment (macOS)
./scripts/bootstrap_macos.sh

# Build the CLI
cargo build --release -p hqe
```

### CLI Usage

```bash
# Run a local-only scan
./target/release/hqe scan /path/to/repo --local-only

# LLM-enabled scan (any OpenAI-compatible provider)
./target/release/hqe scan /path/to/repo --profile my-provider

# Disable local semantic caching
./target/release/hqe scan /path/to/repo --profile my-provider --no-cache

# Export an existing run
./target/release/hqe export RUN_ID --out ./hqe-exports

# Execute a Thinktank prompt
./target/release/hqe prompt PROMPT_NAME --args '{"param": "value"}' --profile my-profile

# Validate the HQE protocol
./target/release/hqe validate-protocol
```

### Desktop App

```bash
cd desktop/workbench

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Development

### Running Tests

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

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy --workspace -- -D warnings

# Validate protocol
./scripts/validate_protocol.sh
```

## Security

Security is a top priority with multiple layers of protection:

- **Input Validation**: Template key validation, path canonicalization
- **Output Sanitization**: DOMPurify for LLM output
- **Database Security**: SQLCipher AES-256 encryption
- **Key Management**: macOS Keychain integration
- **Prompt Security**: 50+ jailbreak pattern detection
- **Injection Prevention**: Parameterized SQL queries

## Project Structure

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

## Thinktank Prompts

The Thinktank feature provides 30+ expert prompts organized by category:

- Security: Audit for vulnerabilities, CVE checks, secure coding review
- Quality: Code smells, best practices, performance analysis
- Refactoring: Modernization suggestions, tech debt identification
- Documentation: API docs, README generation, changelog creation
- Testing: Unit test generation, coverage analysis, edge case detection

## Contributing

The project welcomes contributions. See CONTRIBUTING.md for details on:

- Reporting bugs and requesting features
- Development setup and workflow
- Code style and testing requirements
- Pull request process

## Legal and Compliance

- Licensed under Apache License 2.0
- Privacy-focused with local-only mode option
- No telemetry collected without explicit consent
- Security vulnerabilities reported privately to <2-craze-headmen@icloud.com>
