# HQE Workbench Master TODO

> **Last Updated:** 2026-01-29
> **Health Score:** 10/10 (Production-Ready)

---

## Pending Work

### 🟡 Low Priority

| ID       | Category      | Description                                                      | Location               |
| -------- | ------------- | ---------------------------------------------------------------- | ---------------------- |
| DEPS-001 | Dependencies  | Audit unused dependencies with `cargo udeps` (Requires Nightly)  | `Cargo.toml` files     |
| ERR-001  | Code Quality  | Standardize error types (`thiserror` for libs, `anyhow` for CLI) | Multiple crates        |
| MCP-001  | Validation    | Add JSON schema validation for tool arguments                    | `cli/hqe/src/main.rs`  |

---

## Completed Items

### Documentation

- [x] **DOC-001**: Add rustdoc to all public functions in `hqe-core`, `hqe-openai`, `hqe-git`, `hqe-mcp`, `hqe-ingest`, `hqe-flow`, `hqe-artifacts`, `hqe-protocol`

### Security

- [x] **SEC-001**: Refactor `hqe-openai` header construction (no `expect`)
- [x] **SEC-002**: Tighten `tauri.conf.json` asset scope
- [x] **SEC-003**: Implement structured error sanitization

### Reliability

- [x] **REL-001**: Replace `unwrap()` in `hqe-core/src/repo.rs`
- [x] **REL-002**: Replace `unwrap()` in `hqe-openai/src/provider_discovery.rs`
- [x] **REL-003**: Replace remaining `unwrap()` in test utility helpers

### Performance

- [x] **PERF-001**: Migrate `hqe-core` from `std::fs` to `tokio::fs`
- [x] **PERF-002**: Fix blocking git operations
- [x] **PERF-003**: Move regex compilation outside file loop (`hqe-core/src/repo.rs`)
- [x] **RATE-001**: Implement rate limiting on API calls (`hqe-openai/src/rate_limiter.rs`)
- [x] **GIT-001**: Fixed async operation in `hqe-git` (was already correct)

### UI/UX

- [x] **UI-FUNC-001**: Connect `ReportScreen` to real data
- [x] **UI-FUNC-002**: Implement Toast system
- [x] **UI-FUNC-003**: Add error toasts to `ScanScreen`
- [x] **UI-A11Y-001**: Semantic buttons
- [x] **UI-A11Y-002**: Emoji accessibility labels
- [x] **UI-A11Y-003**: Distinct `:focus-visible` states on all inputs
- [x] **UI-CODE-001**: Remove inline styles
- [x] **UI-CODE-002**: Define `ScanReport` interfaces
- [x] **UI-CODE-003**: Extract `Card`/`Badge` components
- [x] **UI-PERF-001**: Report hydration persistence
- [x] **UI-PERF-002**: Set `isScanning=false` before route push
- [x] **UI-UX-001**: Markdown syntax highlighting
- [x] **UI-UX-002**: Loading skeletons

### Infrastructure

- [x] **DX-001**: Add `cargo clippy` to CI pipeline

---

*Reviewed crates: hqe-core, hqe-openai, hqe-git, hqe-mcp, hqe-ingest, hqe-flow, hqe-artifacts, hqe-protocol, cli/hqe*
