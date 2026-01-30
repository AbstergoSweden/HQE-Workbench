# HQE Workbench Architecture

## Overview

HQE Workbench is a local-first macOS desktop application built with Tauri v2, providing a native interface for the HQE Engineer Protocol.

```
┌─────────────────────────────────────────────────────────────┐
│                     HQE Workbench                           │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Tauri App (Rust + WebView)              │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │   React UI  │  │   Commands  │  │   State     │  │  │
│  │  │  (Frontend) │  │   (Bridge)  │  │  (Mutex)    │  │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  │  │
│  └─────────┼────────────────┼────────────────┼─────────┘  │
│            │                │                │            │
│  ┌─────────┴────────────────┴────────────────┴─────────┐  │
│  │                    Rust Core                        │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐ │  │
│  │  │hqe-core  │ │hqe-openai│ │ hqe-git  │ │hqe-art │ │  │
│  │  │(Pipeline)│ │(Provider)│ │ (GitOps) │ │(Export)│ │  │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └───┬────┘ │  │
│  └───────┼────────────┼────────────┼───────────┼──────┘  │
└──────────┼────────────┼────────────┼───────────┼─────────┘
           │            │            │           │
    ┌──────┴──────┐    │     ┌──────┴──────┐    │
    │  Local FS   │    │     │   Git CLI   │    │
    └─────────────┘    │     └─────────────┘    │
                       │                        │
                  ┌────┴────┐            ┌─────┴─────┐
                  │OpenAI   │            │Artifacts  │
                  │Provider │            │(MD/JSON)  │
                  └─────────┘            └───────────┘
```

## Module Boundaries

### hqe-core
**Purpose:** Core scan pipeline and data models

**Responsibilities:**
- Data models (RunManifest, HqeReport, etc.)
- Scan pipeline orchestration
- Content redaction engine
- Repository scanning and analysis
- Local heuristics

**Key Types:**
- `ScanPipeline` - Main orchestrator
- `RedactionEngine` - Secret detection and masking
- `RepoScanner` - File system analysis

### hqe-openai
**Purpose:** OpenAI-compatible LLM provider client

**Responsibilities:**
- HTTP client for chat completions
- Request/response serialization
- Authentication handling
- Retry logic with exponential backoff
- Prompt templates

**Key Types:**
- `OpenAIClient` - Main client
- `ProviderProfile` - Configuration
- `ChatRequest/ChatResponse` - API types

### hqe-git
**Purpose:** Git operations wrapper

**Responsibilities:**
- Repository detection
- Clone, status, branch operations
- Patch application (dry-run + actual)
- Commit creation

**Key Types:**
- `GitRepo` - Repository handle
- Shells out to system `git` binary

### hqe-artifacts
**Purpose:** Report and manifest generation

**Responsibilities:**
- Markdown report rendering
- JSON serialization
- File output management

**Key Types:**
- `ArtifactWriter` - Output coordinator

## Data Flow

### Scan Flow

```
1. User selects repository
   ↓
2. Phase A: Ingestion
   - Walk directory tree
   - Detect tech stack
   - Read key files
   - Run local risk checks
   ↓
3. Redaction
   - Scan content for secrets
   - Replace with REDACTED_*
   - Generate redaction log
   ↓
4. Phase B: Analysis
   IF local-only:
     - Use heuristics only
     - Generate partial report
   ELSE:
     - Build evidence bundle
     - Send to LLM provider
     - Parse response
   ↓
5. Phase C: Report Generation
   - Build HqeReport struct
   - Populate all 8 sections
   ↓
6. Phase D: Artifact Export
   - Write run-manifest.json
   - Write report.json
   - Write report.md
   - Write session-log.json
```

## Security Considerations

1. **API Keys:** Stored in macOS Keychain, never in config files
2. **Redaction:** All content scanned for secrets before LLM transmission
3. **Local Mode:** Full functionality without external API calls
4. **Preview:** User sees exactly what will be sent to provider

## Threading Model

- **Tauri Commands:** Async Tokio runtime
- **UI:** Single-threaded JavaScript (main thread)
- **Git Operations:** Spawned processes
- **File I/O:** Async Tokio

## Error Handling

- Rust: `anyhow` for application errors, `thiserror` for library errors
- JavaScript: Try/catch with user-friendly messages
- Tauri: Serialized errors sent to frontend
