# Phase 1: Repository Restructure Plan

**Generated:** 2026-02-03  
**Protocol:** High-Rigor Restructure with Evidence-First Deletion

---

## 1. CURRENT STATE SUMMARY

### 1.1 Repository Architecture

| Component | Type | Location | Purpose |
|-----------|------|----------|---------|
| **Workspace** | Cargo | Root `Cargo.toml` | 11 members, resolver = "2" |
| **Core Crates** | Rust Lib | `crates/*` | 9 library crates (hqe-core, hqe-openai, etc.) |
| **CLI** | Rust Bin | `cli/hqe/` | Main command-line interface |
| **Desktop App** | Tauri | `apps/workbench/` | React/Vite frontend + Rust backend |
| **Protocol** | YAML/Python | `protocol/` | HQE Protocol v3 definitions + validators |
| **Documentation** | Markdown | `docs/` | Architecture, API, threat model docs |
| **Scripts** | Shell/Python | `scripts/` | Build, validation, bootstrap utilities |
| **Prompts/MCP** | TypeScript | `prompts/` | Full MCP server implementation |

### 1.2 Crate Dependency Graph

```
hqe-protocol (base)
    ↓
hqe-core → hqe-openai, hqe-artifacts
    ↓
hqe-mcp → hqe-flow, hqe-ingest
    ↓
cli/hqe (consumer of all)
hqe-workbench-app (Tauri, selective deps)
```

### 1.3 Path-Sensitive Hotspots (CRITICAL)

| Hotspot | Location | Risk Level | Description |
|---------|----------|------------|-------------|
| `include_str!` macros | `cli/hqe/src/main.rs:1-3` | **CRITICAL** | Embedded protocol files: `../../../protocol/*` |
| Tauri path deps | `apps/workbench/src-tauri/Cargo.toml` | **HIGH** | Relative paths: `../../../crates/*` |
| Tauri frontendDist | `tauri.conf.json` | **HIGH** | `"../dist"` relative path |
| CI path refs | `.github/workflows/*.yml` | **MEDIUM** | References to `crates/`, `cli/`, `apps/` |
| Script paths | `scripts/*.sh` | **MEDIUM** | May reference relative paths |

### 1.4 Build/Test Entrypoints

| Command | Location | Evidence |
|---------|----------|----------|
| `cargo build --workspace` | Root | `Cargo.toml` workspace definition |
| `cargo test --workspace` | Root | Standard Rust testing |
| `npm run build` | `apps/workbench/` | `package.json` scripts |
| `npm run preflight` | Root | `package.json` |
| Protocol validation | `protocol/` | `validate.py`, `verify.py` |

---

## 2. PROPOSED TARGET STRUCTURE

```
hqe-workbench/
├── Cargo.toml                    # Workspace root (unchanged)
├── Cargo.lock                    # (unchanged)
├── README.md                     # (unchanged)
├── LICENSE                       # Apache 2.0 (updated)
├── NOTICE                        # Attribution file
├── CHANGELOG.md                  # Release notes
├── CITATION.cff                  # Academic citation
├── CODE_OF_CONDUCT.md            # Community standards
├── CONTRIBUTING.md               # Contribution guidelines
├── GOVERNANCE.md                 # Project governance
├── SECURITY.md                   # Security policy
├── AUTHORS                       # Maintainers list
│
├── .github/                      # GitHub configuration
│   ├── workflows/                # CI/CD pipelines
│   ├── ISSUE_TEMPLATE/           # Issue templates
│   └── CODEOWNERS                # Path ownership
│
├── .cargo/                       # Cargo configuration (if needed)
│
├── docs/                         # Documentation
│   ├── architecture.md
│   ├── architecture_v2.md
│   ├── API.md
│   ├── DEVELOPMENT.md
│   ├── HOW_TO.md
│   ├── PROMPTS_AUDIT.md
│   ├── threat-model.md
│   └── ...
│
├── scripts/                      # Build/utility scripts
│   ├── bootstrap_macos.sh
│   ├── build_dmg.sh
│   ├── dev.sh
│   ├── validate_protocol.sh
│   └── ...
│
├── protocol/                     # HQE Protocol definitions
│   ├── hqe-engineer.yaml         # Main protocol spec
│   ├── hqe-engineer-schema.json  # JSON schema
│   ├── hqe-schema.json           # Validation schema
│   ├── validate.py               # Python validator
│   ├── verify.py                 # Verification script
│   └── archive/                  # Historical versions
│
├── crates/                       # Rust workspace crates
│   ├── hqe-core/                 # Core engine
│   ├── hqe-protocol/             # Protocol types
│   ├── hqe-openai/               # OpenAI client
│   ├── hqe-git/                  # Git operations
│   ├── hqe-artifacts/            # Report generation
│   ├── hqe-mcp/                  # MCP protocol
│   ├── hqe-ingest/               # File ingestion
│   ├── hqe-flow/                 # Workflow engine
│   └── hqe-vector/               # Vector/embedding (stub)
│
├── cli/                          # Command-line interface
│   └── hqe/                      # Main CLI crate
│       ├── Cargo.toml
│       └── src/
│           └── main.rs           # Contains include_str! macros
│
├── desktop/                      # Desktop application
│   └── workbench/                # Tauri app (moved from apps/)
│       ├── src-tauri/            # Rust backend
│       ├── src/                  # React frontend
│       ├── dist/                 # Build output
│       └── package.json
│
├── mcp-server/                   # MCP Server (renamed from prompts/)
│   ├── README.md
│   ├── AGENTS.md
│   ├── .agents/                  # Agent configuration
│   ├── server/                   # TypeScript MCP server
│   ├── docs/                     # Server documentation
│   └── ...
│
├── tests/                        # Integration tests
│   └── fixtures/                 # Test data
│
└── examples/                     # Example repositories
    └── demo-repo/                # Renamed from example-repo/
```

---

## 3. MOVE MAP (CRITICAL)

### 3.1 High-Priority Moves

| Old Path | New Path | Type | Rationale | Risk | Follow-up Required |
|----------|----------|------|-----------|------|-------------------|
| `apps/workbench/` | `desktop/workbench/` | Tauri App | Clearer separation from potential future apps | **HIGH** | Update CI paths, Tauri config |
| `prompts/` | `mcp-server/` | MCP Server | Current name is misleading - it's a full server | **HIGH** | Update AGENTS.md references |
| `example-repo/` | `examples/demo-repo/` | Test Fixture | Better organization, clearer purpose | **LOW** | Update any test references |

### 3.2 Path Reference Updates Required

| File | Current Reference | New Reference | Action |
|------|-------------------|---------------|--------|
| `cli/hqe/src/main.rs:1` | `../../../protocol/hqe-engineer.yaml` | `../../../protocol/hqe-engineer.yaml` | **UNCHANGED** (relative to new cli location) |
| `cli/hqe/src/main.rs:2` | `../../../protocol/hqe-schema.json` | `../../../protocol/hqe-schema.json` | **UNCHANGED** |
| `cli/hqe/src/main.rs:3` | `../../../protocol/verify.py` | `../../../protocol/verify.py` | **UNCHANGED** |
| `.github/workflows/ci.yml` | `apps/workbench/` | `desktop/workbench/` | **UPDATE** |
| `.github/workflows/release.yml` | `apps/workbench/` | `desktop/workbench/` | **UPDATE** |
| `desktop/workbench/src-tauri/Cargo.toml` | `../../../crates/*` | `../../../crates/*` | **UNCHANGED** (relative paths preserved) |
| `desktop/workbench/src-tauri/tauri.conf.json` | `../dist` | `../dist` | **UNCHANGED** |

---

## 4. DELETION CANDIDATES (Evidence-Based)

### 4.1 Safe to Delete (High Evidence)

| File/Dir | Evidence | Reasoning | Restore Command |
|----------|----------|-----------|-----------------|
| `GEMINI.md` | AGENTS.md exists, no code refs | Superseded by LLM-agnostic AGENTS.md | `git checkout HEAD -- GEMINI.md` |
| `CLAUDE.md` | AGENTS.md exists, no code refs | Superseded by LLM-agnostic AGENTS.md | `git checkout HEAD -- CLAUDE.md` |
| `QWEN.md` | AGENTS.md exists, no code refs | Superseded by LLM-agnostic AGENTS.md | `git checkout HEAD -- QWEN.md` |
| `TODO.md` | `TODO_UNIFIED.md` is canonical | Redundant, smaller file superseded | `git checkout HEAD -- TODO.md` |
| `hqe_workbench_provider_discovery/` | Self-contained, appears generated | Temporary scaffold directory | `git checkout HEAD -- hqe_workbench_provider_discovery/` |
| `verification_results.md` | `verification_results_v2.md` exists | Superseded version | `git checkout HEAD -- verification_results.md` |

### 4.2 Deprecate (Uncertain Reachability)

| File/Dir | Concern | Action | Evidence Needed |
|----------|---------|--------|-----------------|
| `SUMMARY_FINDINGS.md` | Unclear if referenced | Mark deprecated | Search for references in docs |
| `SECURITY_AND_BUG_FINDINGS.md` | May be historical | Mark deprecated | Check if linked from SECURITY.md |
| `REPOSITORY_HEALTH_REPORT.md` | Generated report | Keep (but gitignore future?) | Verify not required by CI |

### 4.3 Keep (Required)

| File/Dir | Reason |
|----------|--------|
| All `crates/` | Core workspace members |
| `cli/hqe/` | Primary binary |
| `apps/workbench/` (moving) | Desktop app |
| `protocol/` | Required by include_str! |
| `docs/` | Documentation |
| `scripts/` | Build utilities |
| `AGENTS.md` | Canonical LLM context |

---

## 5. VERIFICATION PLAN

### 5.1 Pre-Change Baseline

```bash
# Record current state
cargo test --workspace 2>&1 | tee /tmp/before_tests.log
cargo build --workspace 2>&1 | tee /tmp/before_build.log
cargo clippy --workspace 2>&1 | tee /tmp/before_clippy.log

# Check frontend builds
cd apps/workbench && npm ci && npm run build && cd ../..
```

### 5.2 Post-Change Verification

| Check | Command | Pass Criteria |
|-------|---------|---------------|
| Workspace builds | `cargo build --workspace` | Exit 0, no errors |
| Tests pass | `cargo test --workspace` | All tests pass |
| Clippy clean | `cargo clippy --workspace -- -D warnings` | Zero warnings |
| Format clean | `cargo fmt --all -- --check` | No formatting changes needed |
| CLI builds | `cargo build -p hqe --release` | Binary created |
| Desktop builds | `cd desktop/workbench && npm run tauri:build` | DMG/app created |
| Protocol valid | `python protocol/validate.py` | Exit 0 |
| Include macros work | `cargo build -p hqe` | Protocol files embedded |

### 5.3 Path Integrity Checks

| Mechanism | Check Method | Expected Result |
|-----------|--------------|-----------------|
| `include_str!` | Build CLI crate | Protocol files compile-time embedded |
| Tauri path deps | Build Tauri app | Crates resolved correctly |
| CI workflows | Dry-run workflows | Paths resolve in GitHub Actions |

---

## 6. ROLLBACK STRATEGY

### 6.1 Staged Commit Strategy

| Commit | Contents | Rollback |
|--------|----------|----------|
| `1/4` | Directory moves only (`git mv`) | `git reset --hard HEAD~4 && git checkout HEAD -- .` |
| `2/4` | Path reference updates | `git revert HEAD~2..HEAD` |
| `3/4` | Safe deletions | `git checkout HEAD~1 -- <deleted-files>` |
| `4/4` | Repairs from verification | `git revert HEAD` |

### 6.2 Full Rollback

```bash
# Complete restore to pre-restructure state
git reset --hard HEAD~4

# Or selective file restore
git checkout HEAD -- GEMINI.md CLAUDE.md QWEN.md TODO.md
```

---

## 7. RISK ASSESSMENT

| Change | Risk Level | Mitigation |
|--------|------------|------------|
| Move `apps/workbench/` → `desktop/workbench/` | **HIGH** | Update all CI refs, verify Tauri config |
| Rename `prompts/` → `mcp-server/` | **HIGH** | Update AGENTS.md, check for external refs |
| Delete LLM-specific md files | **LOW** | Confirmed superseded by AGENTS.md |
| Delete `hqe_workbench_provider_discovery/` | **LOW** | Self-contained, no external refs |
| Move `example-repo/` → `examples/` | **LOW** | Fewer references, test fixture only |

---

## 8. ASSUMPTIONS

1. **AGENTS.md is canonical**: Assumed GEMINI.md, CLAUDE.md, QWEN.md are fully superseded.
2. **hqe_workbench_provider_discovery is temporary**: Appears to be generated scaffolding.
3. **Protocol files don't move**: Keeping `protocol/` at root to avoid breaking `include_str!` macros.
4. **No external consumers of prompts/**: MCP server rename safe for external deps.

---

**STOP: Phase 1 Complete. Await approval before executing Phase 2.**
