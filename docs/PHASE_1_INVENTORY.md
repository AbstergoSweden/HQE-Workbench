# Phase 1 — Inventory Document

**Project:** HQE Workbench  
**Document Version:** 1.0.0  
**Date:** 2025-02-01  
**Status:** COMPLETE

---

## Executive Summary

This document provides a comprehensive inventory of the current HQE Workbench implementation, documenting existing subsystems relevant to the Two-Phase Protocol implementation. The inventory covers: Prompt System, Provider Configuration, Chat/Persistence, and Repo Access/Scanning.

---

## 1. Prompt System

### 1.1 Prompt Definitions Location(s)

| Location | Format | Purpose |
|----------|--------|---------|
| `mcp-server/cli-prompt-library/` | TOML | Primary prompt templates for CLI usage |
| `mcp-server/prompts/server/resources/prompts/` | TOML/YAML | Server-side prompt resources |
| `prompts/` (symlink) | → `mcp-server/prompts/server/resources/prompts` | Repo root convenience symlink |

**File Format Details:**
- **TOML files** contain `description`, `prompt` (template string), and optional `args` array
- Template syntax: `{{variable_name}}` for substitution
- Example structure from `crates/hqe-mcp/src/loader.rs`:
  ```toml
  description = "Analyze code for security issues"
  prompt = """
  Analyze the following code for security vulnerabilities:
  {{code}}
  """
  [[args]]
  name = "code"
  description = "Code to analyze"
  required = true
  ```

### 1.2 Prompt Registry/Menu Logic

**Location:** `crates/hqe-mcp/src/loader.rs` (lines 83-124)

**Current Implementation:**
- `PromptLoader::new(root_path)` creates a loader instance
- `PromptLoader::load()` walks directory tree using `walkdir`
- Loads `.toml`, `.yaml`, `.yml` files
- Filters out directories: `.git`, `node_modules`, `dist`, `build`, `target`, `__pycache__`
- Special handling: skips `prompts/prompts/**` at depth 1 (vendored MCP server)
- **Security:** Validates canonical paths to prevent path traversal (lines 127-144)

**Frontend Menu Logic:**
- **Location:** `desktop/workbench/src/screens/ThinktankScreen.tsx` (lines 102-586)
- Loads prompts via Tauri command `get_available_prompts`
- Filters by search query and agent prompt visibility
- Agent prompts identified by prefix: `conductor_` or `cli_security_`

### 1.3 Prompt Execution Logic

**Location:** `desktop/workbench/src-tauri/src/prompts.rs` (lines 1-267)

**Current Flow:**
1. `get_available_prompts` → loads all prompts from `PromptLoader`
2. `execute_prompt` command:
   - Loads prompts (again)
   - Finds tool by name
   - Initializes OpenAI client from profile
   - Substitutes template variables via `substitute_template()` (lines 252-267)
   - Sends single-turn chat request
   - Returns response content

**Current Template Substitution:**
```rust
fn substitute_template(template: &str, args: &serde_json::Value) -> String {
    // Simple {{key}} → value replacement
    // Uses sanitize_for_prompt from hqe_openai
}
```

### 1.4 Template Engine / Placeholder Substitution

**Current State:** Basic string replacement
- Pattern: `{{key}}`
- Sanitization via `hqe_openai::prompts::sanitize_for_prompt`
- No nested objects, conditionals, or loops
- Located: `desktop/workbench/src-tauri/src/prompts.rs` lines 252-267

### 1.5 Error Sources

**"Missing required key input" errors originate from:**
1. **Template substitution:** When `args` doesn't contain expected keys
2. **Validation in loader:** `loader.rs` lines 216-226 checks for implicit `args` if template uses `{{args}}`
3. **Frontend validation:** `ThinktankScreen.tsx` lines 184-186 uses `buildTypedArgs` to validate schema

---

## 2. Provider Config / API Specs

### 2.1 Provider Profiles Location

**Storage:** `~/.local/share/hqe-workbench/profiles.json`

**Structure:** (from `crates/hqe-openai/src/profile.rs`)
```rust
pub struct ProviderProfile {
    pub name: String,
    pub base_url: String,
    pub api_key_id: String,
    pub default_model: String,
    pub headers: Option<HashMap<String, String>>,
    pub organization: Option<String>,
    pub project: Option<String>,
    pub provider_kind: Option<ProviderKind>,
    pub timeout_s: u64,
}
```

### 2.2 API Key Storage

**Location:** `crates/hqe-openai/src/profile.rs` lines 182-284

**Implementation:**
- macOS Keychain via `keyring` crate
- Service name: `"hqe-workbench"`
- Account format: `"api_key:{profile_name}"`
- In-memory store available for testing

### 2.3 Prefilled API Specs — CURRENT STATUS

**User report confirmed: NO prefilled provider profiles exist currently.**

**Default profiles in Settings UI** (`SettingsScreen.tsx` lines 254-261):
```typescript
const defaultProfiles = [
  { name: 'openai', base_url: 'https://api.openai.com/v1', ... },
  { name: 'anthropic', base_url: 'https://api.anthropic.com/v1', ... },
  { name: 'venice', base_url: 'https://api.venice.ai/api/v1', ... },
  { name: 'openrouter', base_url: 'https://openrouter.ai/api/v1', ... },
  { name: 'xai-grok', base_url: 'https://api.x.ai/v1', ... },
  { name: 'google-gemini', base_url: 'https://generativelanguage.googleapis.com/v1beta/openai', ... },
];
```

**Gap:** These are hardcoded in the UI but not implemented as a proper "prefilled profiles" system.

### 2.4 Discovery Implementation

**Location:** `crates/hqe-openai/src/provider_discovery.rs` (lines 1-1000+)

**Supported Providers:**
| Provider | Auto-detect | Discovery Endpoint | Notes |
|----------|-------------|-------------------|-------|
| Venice.ai | ✓ | `/api/v1/models?type=all` | Full capabilities, pricing |
| OpenRouter | ✓ | `/api/v1/models` | Context length, pricing |
| xAI/Grok | ✓ | `/v1/models` | Basic OpenAI-compatible |
| OpenAI | ✓ | `/v1/models` | Basic |
| Azure | ✓ | N/A | Discovery NOT supported |
| Generic | Fallback | `/v1/models` | Basic |

**Inputs to Discovery:**
```rust
pub struct ProviderProfileInput {
    pub base_url: String,
    pub headers: BTreeMap<String, String>,
    pub api_key: String,
    pub timeout_s: u64,
    pub no_cache: bool,
}
```

---

## 3. Chat / Persistence

### 3.1 Chat System — CURRENT STATUS

**Current State:** NO chat system exists.

The Thinktank screen (`ThinktankScreen.tsx`) provides:
- Single-turn prompt execution
- No conversation history
- No multi-turn capability
- No persistence of conversations

### 3.2 Current Local DB Usage

**Location:** `crates/hqe-core/src/persistence.rs` (lines 1-233)

**Implementation:**
- SQLite database at `~/.local/share/hqe-workbench/hqe.db`
- Tables:
  - `request_cache` — LLM request/response caching
  - `session_log` — Basic audit logging
- **NO encryption currently implemented**
- WAL mode enabled for concurrency

### 3.3 Encryption / Secure Storage

**Current State:**
- API keys: ✓ Encrypted (macOS Keychain)
- DB contents: ✗ NOT encrypted
- Chat history: N/A (no chat system)

**Existing primitives:**
- `keyring` crate for keychain access
- `secrecy` crate for SecretString
- No SQLCipher or similar DB encryption

### 3.4 Report Output Page

**Location:** `desktop/workbench/src/screens/ReportScreen.tsx` (lines 1-275)

**Features:**
- Displays scan results (findings, TODO backlog)
- Health score visualization
- Export artifacts functionality
- **NO chat continuity**
- One-way display only

---

## 4. Repo Access / Scanning

### 4.1 Local Repo Utilities

**Location:** `crates/hqe-core/src/repo.rs` (lines 1-41357)

**Capabilities:**
- File tree traversal
- Content reading
- `.gitignore` respect
- Binary file detection
- Size limits enforcement

**Security features:**
- Path validation (lines 124-153 in `commands.rs`)
- Parent directory reference blocking
- Home directory boundary enforcement

### 4.2 GitHub Import

**Current State:** NO GitHub import functionality exists.

The `get_repo_info` command (`commands.rs` lines 156-187) only reads local git metadata.

### 4.3 Scan Pipeline

**Location:** `crates/hqe-core/src/scan.rs` (lines 1-25080)

**Capabilities:**
- Local static analysis
- LLM-enhanced analysis (optional)
- Artifact generation
- Report creation

---

## 5. UI Architecture

### 5.1 Screen Layout

**Location:** `desktop/workbench/src/screens/`

| Screen | Purpose |
|--------|---------|
| `WelcomeScreen.tsx` | Landing, repo selection |
| `ScanScreen.tsx` | Configure and run scans |
| `ReportScreen.tsx` | Display scan results |
| `ThinktankScreen.tsx` | Prompt execution |
| `SettingsScreen.tsx` | Provider configuration |

### 5.2 State Management

**Location:** `desktop/workbench/src/store.ts` (lines 1-69)

**Current Stores:**
- `useRepoStore` — Selected repo (persisted)
- `useScanStore` — Scan progress (ephemeral)
- `useReportStore` — Current report (persisted)

**Gap:** No chat/conversation store exists.

---

## 6. Tauri Commands

**Location:** `desktop/workbench/src-tauri/src/commands.rs` (lines 1-505)

**Available Commands:**
| Command | Purpose |
|---------|---------|
| `select_folder` | Native folder picker |
| `scan_repo` | Run HQE scan pipeline |
| `get_repo_info` | Get git metadata |
| `load_report` | Load report by run_id |
| `export_artifacts` | Export scan artifacts |
| `discover_models` | Provider model discovery |
| `list_provider_profiles` | List saved profiles |
| `import_default_profiles` | Import default profiles |
| `get_provider_profile` | Get single profile |
| `save_provider_profile` | Save profile + key |
| `delete_provider_profile` | Delete profile |
| `test_provider_connection` | Test profile connectivity |
| `detect_provider_kind` | Auto-detect provider |
| `get_available_prompts` | List prompt tools |
| `execute_prompt` | Execute prompt tool |

---

## 7. Security Analysis

### 7.1 Current Security Measures

| Control | Status | Location |
|---------|--------|----------|
| Path traversal prevention | ✓ | `commands.rs:124-153` |
| API key keychain storage | ✓ | `profile.rs:208-284` |
| Prompt injection detection | ✓ | `loader.rs:283-336` |
| Template delimiter validation | ✓ | `loader.rs:327-334` |
| Secret redaction | Partial | `redaction.rs` |
| DB encryption | ✗ | N/A |
| System prompt immutability | ✗ | N/A |

### 7.2 Identified Gaps

1. **No universal system prompt** — Each request sends raw user content
2. **No chat encryption** — No encrypted chat persistence
3. **No MCP tool sandbox** — No tool-call interface for repo introspection
4. **No GitHub import** — Cannot import public repos
5. **Prompt execution not deterministic** — No centralized PromptRunner

---

## 8. File Inventory Summary

### Core Rust Crates
```
crates/
├── hqe-artifacts/       # Artifact generation
├── hqe-core/            # Core models, persistence, repo, scan
├── hqe-flow/            # Workflow engine
├── hqe-git/             # Git operations
├── hqe-ingest/          # File ingestion
├── hqe-mcp/             # MCP protocol, prompt loader
├── hqe-openai/          # OpenAI client, provider discovery, profiles
├── hqe-protocol/        # Shared protocol models
└── hqe-vector/          # Vector embeddings
```

### Desktop Application
```
desktop/workbench/
├── src/
│   ├── App.tsx          # Main app component
│   ├── store.ts         # Zustand state stores
│   ├── types.ts         # TypeScript types
│   ├── components/      # UI components
│   ├── context/         # React contexts
│   └── screens/         # Main screens
└── src-tauri/
    └── src/
        ├── commands.rs  # Tauri commands
        ├── prompts.rs   # Prompt execution
        └── lib.rs       # App state
```

### Protocol Definitions
```
protocol/
├── hqe-engineer.yaml    # Main protocol definition (v4.2.1)
├── hqe-engineer-schema.json  # JSON Schema
└── hqe-schema.json      # Legacy schema (v3.1.0)
```

### Prompt Resources
```
mcp-server/
├── cli-prompt-library/  # TOML prompt templates
└── prompts/
    └── server/
        └── resources/
            ├── gates/       # Gate prompts
            ├── methodologies/ # Methodology docs
            ├── prompts/     # Example prompts
            └── styles/      # Style guides
```

---

## 9. Evidence References

All paths cited in this document are relative to repository root. Key files:

| Citation | Path | Lines |
|----------|------|-------|
| Prompt loader | `crates/hqe-mcp/src/loader.rs` | 1-393 |
| Prompt execution | `desktop/workbench/src-tauri/src/prompts.rs` | 1-267 |
| Provider profiles | `crates/hqe-openai/src/profile.rs` | 1-480 |
| Provider discovery | `crates/hqe-openai/src/provider_discovery.rs` | 1-1000+ |
| Persistence | `crates/hqe-core/src/persistence.rs` | 1-233 |
| Tauri commands | `desktop/workbench/src-tauri/src/commands.rs` | 1-505 |
| Thinktank UI | `desktop/workbench/src/screens/ThinktankScreen.tsx` | 1-586 |
| Report UI | `desktop/workbench/src/screens/ReportScreen.tsx` | 1-275 |
| Settings UI | `desktop/workbench/src/screens/SettingsScreen.tsx` | 1-600+ |
| State stores | `desktop/workbench/src/store.ts` | 1-69 |
| Type definitions | `desktop/workbench/src/types.ts` | 1-101 |

---

**END OF INVENTORY DOCUMENT**
