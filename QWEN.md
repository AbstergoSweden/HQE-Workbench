# HQE Workbench - AI Agent Context

## Project Overview

HQE Workbench is a local-first macOS desktop application for running the HQE (High Quality Engineering) Engineer Protocol - a comprehensive codebase health auditing and technical leadership automation system. The project provides both a CLI tool and a GUI application built with Tauri v2, designed to analyze repositories for security, quality, and architectural issues.

### Key Capabilities
- **Repository Scanning**: Analyzes codebases for security, quality, and architectural issues
- **LLM Integration**: Supports OpenAI-compatible providers for AI-powered analysis
- **Local-Only Mode**: Full functionality without external API calls for sensitive repositories
- **Secret Redaction**: Automatically detects and masks secrets before transmission
- **Report Generation**: Produces structured reports in Markdown and JSON formats
- **PR Harvesting**: Analyzes existing pull requests for improvements and conflicts

## Technology Stack

### Backend (Rust)
- **Runtime**: Tokio async runtime
- **Error Handling**: `anyhow` for applications, `thiserror` for libraries
- **Serialization**: `serde` with JSON and YAML support
- **HTTP Client**: `reqwest` for LLM provider communication
- **Security**: `secrecy` for sensitive data, `keyring` for macOS Keychain integration
- **Tracing**: `tracing` with `tracing-subscriber` for structured logging

### Frontend (TypeScript/React)
- **Framework**: React 18 with TypeScript 5.3
- **Build Tool**: Vite 5.0
- **Desktop Framework**: Tauri v2
- **State Management**: Zustand
- **Styling**: Tailwind CSS 3.3
- **Markdown Rendering**: `react-markdown` with GitHub Flavored Markdown

### Development Tools
- **Language**: Rust 1.75+, Node.js 20+
- **Package Manager**: Cargo (Rust), npm (Node.js)
- **Platform**: macOS 12.0+ (Monterey)
- **Protocol Validation**: Python 3.10+ with PyYAML and jsonschema

## Project Structure

```
hqe-workbench/
├── Cargo.toml                  # Workspace configuration
├── BUILD.md                    # Build instructions
│
├── protocol/                   # HQE Protocol v3 definitions
│   ├── hqe-engineer.yaml       # Main protocol specification
│   ├── hqe-schema.json         # JSON Schema for validation
│   ├── verify.py               # Python validator
│   └── verify.hs               # Haskell validator
│
├── crates/                     # Rust workspace crates
│   ├── hqe-core/              # Core scan pipeline and data models
│   │   ├── src/models.rs      # Data types (RunManifest, HqeReport, etc.)
│   │   ├── src/scan.rs        # Scan pipeline orchestration
│   │   ├── src/redaction.rs   # Secret detection and masking
│   │   └── src/repo.rs        # Repository scanning
│   │
│   ├── hqe-openai/            # OpenAI-compatible LLM client
│   │   ├── src/lib.rs         # HTTP client implementation
│   │   └── src/prompts.rs     # Prompt templates
│   │
│   ├── hqe-git/               # Git operations wrapper
│   │   └── src/lib.rs         # Repository operations
│   │
│   └── hqe-artifacts/         # Report generation
│       └── src/lib.rs         # Markdown/JSON writers
│
├── cli/                        # CLI binary
│   └── hqe/
│       ├── Cargo.toml
│       └── src/main.rs        # CLI commands and argument parsing
│
├── apps/                       # Desktop applications
│   └── workbench/             # Tauri v2 macOS app
│       ├── package.json       # Node.js dependencies
│       ├── src/               # React + TypeScript UI
│       │   ├── screens/       # Page components (Welcome, Scan, Report, Settings)
│       │   ├── components/    # Shared components (Layout)
│       │   ├── store.ts       # Zustand state management
│       │   ├── App.tsx        # Main app component
│       │   └── main.tsx       # Entry point
│       └── src-tauri/         # Rust backend for Tauri
│           ├── Cargo.toml
│           ├── tauri.conf.json # Tauri configuration
│           └── src/           # Tauri commands
│
├── scripts/                    # Build and utility scripts
│   ├── bootstrap_macos.sh     # Install dependencies
│   ├── dev.sh                 # Start development server
│   ├── build_dmg.sh           # Build release DMG
│   └── validate_protocol.sh   # Validate protocol files
│
├── docs/                       # Documentation
│   ├── architecture.md        # System architecture
│   ├── threat-model.md        # Security analysis
│   ├── provider-config.md     # LLM setup guide
│   └── artifact-format.md     # Output specifications
│
├── tests/                      # Integration tests
│   └── fixtures/              # Test data
│
└── .github/workflows/          # CI/CD
    └── ci.yml                 # GitHub Actions workflow
```

## Core Components

### Scan Pipeline (hqe-core/src/scan.rs)
The scan pipeline consists of four phases:
1. **Ingestion**: Repository structure analysis, file content retrieval, secret redaction
2. **Analysis**: Local risk checks and optional LLM analysis
3. **Report Generation**: Creation of comprehensive HQE reports
4. **Artifact Export**: Generation of Markdown and JSON reports

### Data Models (hqe-core/src/models.rs)
The project defines comprehensive data structures for the HQE protocol:
- **RunManifest**: Metadata for each scan run
- **HqeReport**: Complete 8-section report structure
- **ExecutiveSummary**: Health score and top priorities
- **ProjectMap**: Architecture and tech stack mapping
- **DeepScanResults**: Categorized security and quality findings
- **TodoItem**: Actionable items with severity and risk assessment
- **PatchAction**: Immediate actions with file diffs

### CLI Interface (cli/hqe/src/main.rs)
The CLI provides commands for:
- `hqe validate-protocol`: Validate protocol files
- `hqe scan <repo>`: Scan a repository
- `hqe config`: Manage provider profiles
- `hqe export`: Export specific runs
- `hqe patch`: Generate or apply patches

## Build Commands

### Prerequisites Setup
```bash
# Install all dependencies (macOS only)
./scripts/bootstrap_macos.sh
```

### Development
```bash
# Start Tauri development server
./scripts/dev.sh
# Or manually:
cd apps/workbench && npm run tauri:dev

# Run Rust tests with watch mode
cargo test --workspace --watch
```

### Building
```bash
# Build CLI release binary
cargo build --release -p hqe

# Build Tauri app for distribution
cd apps/workbench && npm run tauri:build
# Or use the script:
./scripts/build_dmg.sh
```

### Testing
```bash
# Run all Rust tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --workspace

# Run protocol validation
./scripts/validate_protocol.sh

# Run ESLint on frontend
cd apps/workbench && npm run lint
```

### Code Quality
```bash
# Format Rust code
cargo fmt

# Run clippy lints
cargo clippy --workspace -- -D warnings

# Check formatting without changing
cargo fmt -- --check
```

## Key Module Responsibilities

### `hqe-core`
- **Purpose**: Core scan pipeline, data models, and redaction engine
- **Key Types**:
  - `ScanPipeline` - Main orchestrator with 4 phases (Ingestion, Analysis, Report Generation, Artifact Export)
  - `RedactionEngine` - Secret detection and masking (patterns for AWS keys, GitHub tokens, etc.)
  - `RepoScanner` - File system analysis with tech stack detection
  - `HqeReport` - Complete 8-section report structure

### `hqe-openai`
- **Purpose**: OpenAI-compatible LLM provider client
- **Key Types**:
  - `OpenAIClient` - HTTP client with retry logic
  - `ProviderProfile` - Configuration (URL, model, headers)
  - `ClientConfig` - Runtime configuration including API key

### `hqe-git`
- **Purpose**: Git operations wrapper
- **Operations**: Repository detection, clone, status, branch operations, patch application
- **Implementation**: Shells out to system `git` binary

### `hqe-artifacts`
- **Purpose**: Report and manifest generation
- **Output Formats**: Markdown reports, JSON structured data
- **Key Type**: `ArtifactWriter` - Coordinates output to filesystem

## Security Considerations

### API Keys
- Stored in macOS Keychain (encrypted), never in config files
- Key references use format: `api_key:{profile_name}`
- Use `secrecy::SecretString` for in-memory handling

### Secret Redaction
- All content scanned before LLM transmission
- Patterns detect: AWS keys, GitHub tokens, Slack tokens, API keys, passwords
- Redacted values replaced with `REDACTED_*` format

### Trust Boundaries
1. **External Provider** (Untrusted) - HTTPS + API Key
2. **HQE Workbench** (Trusted) - Rust backend + Tauri frontend
3. **User Repository** (Sensitive) - Local filesystem only

### Local-Only Mode
- Full functionality without external API calls
- Use for sensitive repositories
- Heuristics-based analysis instead of LLM

## Configuration Storage

### Provider Profiles
- **Location**: `~/.local/share/hqe-workbench/profiles.json`
- **Contents**: Name, base_url, model, headers (without keys)
- **API Keys**: Stored separately in macOS Keychain

### Limits (Default)
- `max_files_sent`: 40
- `max_total_chars_sent`: 250,000
- `snippet_chars`: 4,000

## Development Guidelines

### Rust
- Use `anyhow::Result` for application errors, `thiserror` for library errors
- Prefer async/await with Tokio for concurrent operations
- Use `tracing` for structured logging with appropriate levels
- Keep functions focused; extract helpers for complex logic
- Document public APIs with `///` doc comments

### TypeScript/React
- Use functional components with hooks
- Prefer explicit types over `any`
- Use Zustand for state management
- Follow Tailwind CSS utility-first approach

### General
- Protocol version: 3.1.0 (defined in `hqe-core/src/models.rs`)
- All file paths use `PathBuf`/`Path` for cross-platform compatibility
- Secrets must NEVER be logged or stored in plain text

## Common Issues

### "No module named 'yaml'"
```bash
pip3 install pyyaml jsonschema
```

### "command not found: cargo"
```bash
source $HOME/.cargo/env
```

### macOS Gatekeeper blocks app
```bash
xattr -cr /Applications/HQE\ Workbench.app
```

## Protocol Information

The HQE Engineer Protocol (v4.2.1) is a comprehensive codebase audit and remediation framework that includes:

- **Phases**: Orientation, Triage, Deep Analysis, PR Harvesting
- **Evidence Requirements**: File paths, line numbers, code snippets for all findings
- **Severity Classification**: CRITICAL, HIGH, MEDIUM, LOW, INFO
- **Confidence Tags**: FACT, INFERENCE, HYPOTHESIS, NEEDS_VERIFICATION
- **Output Controls**: Size limits, deduplication, stable IDs

The protocol emphasizes evidence-first analysis, adversarial thinking, minimal-change bias, and pragmatic prioritization.

### Deep Clean Protocol (v4.2)

The protocol follows a structured approach with the following key elements:

#### Constraints
- Zero hallucination: No invented paths, functions, or line numbers
- Mandatory evidence: Each finding must include file path, location, and code snippet
- Trust nothing: Treat all external inputs as attacker-controlled
- Minimal change bias: Prefer smallest safe changes
- Fact vs inference vs hypothesis: Clear distinction required

#### Phases
1. **Phase 0 - Orientation**: Establish foundational understanding (inventory, architecture map, trust boundaries, auth/authz map)
2. **Phase 0.5 - Triage**: Applied to repos with >50 files (prioritized scanning tiers)
3. **Phase 1 - Build Sanity**: Syntax, type errors, broken imports, CI workflow issues
4. **Phase 2 - Logic Reliability**: Control flow, error handling, concurrency, resources
5. **Phase 3 - Security**: Taint chain analysis across multiple domains
6. **Phase 4 - Perf/Maint/DX**: Performance, resource leaks, maintainability, developer experience

#### Output Artifacts
The protocol generates 9 required artifacts in specific order:
1. Risk Register
2. Master TODO
3. Patterns
4. Quick Wins vs Structural
5. Security Summary
6. Reliability Summary
7. Testing Gaps
8. Unknowns
9. Confidence Report

#### Key Concepts
- **Taint Chain Analysis**: Tracking untrusted data from source to sink
- **Trust Boundaries**: Points where validation and normalization must occur
- **Sink**: Operations where untrusted data can cause side effects or security impact
- **Attack Scenarios**: Require evidence of entry points
- **Verification Tiers**: Different approaches for validating findings (existing commands, new tests, static analysis)

The protocol is designed to be evidence-first, security-minded, and focused on actionable remediation plans with minimal-change fixes.

## Security Rules for AI Agents

When working on this codebase, AI agents must follow these strict security rules:

### 1. Input Validation and Sanitization
- **ALWAYS** validate user-provided paths to prevent directory traversal
- **NEVER** trust external inputs without validation
- **ALWAYS** use canonicalization when working with file paths
- **Example**: In `repo.rs`, the `read_file` function now validates paths using `canonicalize()` and checks if the resolved path is within the allowed directory

### 2. Secure Deserialization
- **ALWAYS** validate data after deserialization
- **NEVER** deserialize untrusted data without validation
- **ALWAYS** implement schema validation for configuration files
- **Example**: In `cli/hqe/src/main.rs`, profiles are validated after deserialization using `validate_base_url()` and `validate_headers()`

### 3. Error Message Sanitization
- **NEVER** expose sensitive information in error messages
- **ALWAYS** sanitize error messages before returning to users
- **ALWAYS** log full details internally but return generic messages externally
- **Example**: In `crates/hqe-openai/src/lib.rs`, error messages are sanitized using `sanitize_error_message()` function

### 4. Path Traversal Prevention
- **ALWAYS** validate paths before file operations
- **NEVER** allow `../` sequences in user-provided paths
- **ALWAYS** use path canonicalization to resolve symbolic links
- **Example**: In `apps/workbench/src-tauri/src/commands.rs`, the `load_report` function validates run IDs and canonicalizes paths

### 5. SQL Injection Prevention
- **ALWAYS** use parameterized queries
- **ALWAYS** validate both SQL keywords and formatting patterns together
- **NEVER** allow string concatenation in SQL queries
- **Example**: In `crates/hqe-core/src/repo.rs`, the SQL injection detection now properly groups boolean expressions with parentheses

### 6. Configuration Security
- **ALWAYS** validate configuration values after loading
- **NEVER** store sensitive data in plain text
- **ALWAYS** use secure storage mechanisms (keychain) for secrets
- **Example**: In `crates/hqe-openai/src/profile.rs`, profiles are validated using `normalized_base_url()` and `sanitized_headers()`

## Critical Security Fixes Applied

### 1. Boolean Logic Error in Security Pattern Detection (CRITICAL)
- **Issue**: Incorrect operator precedence in SQL injection detection in `repo.rs` line 397
- **Fix**: Added proper parentheses to group boolean expressions: `(condition1 || condition2 || condition3) && (condition4 || condition5)`
- **Example**:
  ```rust
  // BEFORE (incorrect)
  if (line_lower.contains("select") || line_lower.contains("insert") || line_lower.contains("update"))
      && line_lower.contains("format!") || line_lower.contains("format(") || line.contains("$") {

  // AFTER (correct)
  if (line_lower.contains("select") || line_lower.contains("insert") || line_lower.contains("update"))
      && (line_lower.contains("format!") || line_lower.contains("format(") || line.contains("$")) {
  ```

### 2. Path Traversal Vulnerability (HIGH)
- **Issue**: The `read_file` function in `repo.rs` didn't validate paths, allowing directory traversal
- **Fix**: Added path canonicalization and boundary checks
- **Example**:
  ```rust
  // Added validation to prevent path traversal
  let canonical_full_path = full_path.canonicalize()?;
  let canonical_root = self.root_path.canonicalize()?;
  if !canonical_full_path.starts_with(&canonical_root) {
      return Err(anyhow::anyhow!("Path traversal detected"));
  }
  ```

### 3. Insecure Deserialization (HIGH)
- **Issue**: Profiles were deserialized without validation in CLI config handling
- **Fix**: Added validation after deserialization using validation methods
- **Example**:
  ```rust
  // Added validation after deserialization
  for profile in &profiles {
      profile.validate_base_url()
          .map_err(|e| anyhow::anyhow!("Invalid profile base URL: {}", e))?;
      profile.validate_headers()
          .map_err(|e| anyhow::anyhow!("Invalid profile headers: {}", e))?;
  }
  ```

### 4. Information Disclosure in Error Messages (MEDIUM)
- **Issue**: Full API error responses were exposed to users
- **Fix**: Added error message sanitization
- **Example**:
  ```rust
  // Added sanitization function
  fn sanitize_error_message(message: &str) -> String {
      let sanitized = message
          .replace("api_key", "***REDACTED***")
          .replace("secret", "***REDACTED***")
          .replace("token", "***REDACTED***");
      if sanitized.len() > 200 {
          format!("{}...", &sanitized[..200])
      } else {
          sanitized
      }
  }
  ```

## Testing and Quality Assurance

### Security Testing
- **ALWAYS** run all tests after making changes: `cargo test --workspace`
- **ALWAYS** test path validation with edge cases (e.g., `../`, `./`, symbolic links)
- **ALWAYS** verify error handling with invalid inputs
- **ALWAYS** check that sensitive information is not leaked in logs or error messages

### Code Quality Checks
- **ALWAYS** run `cargo fmt` to format code
- **ALWAYS** run `cargo clippy` to catch potential issues
- **ALWAYS** add unit tests for new functionality
- **ALWAYS** ensure all existing tests pass