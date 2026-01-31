# Unified Bug TODO (hqe-workbench)

Generated: 2026-01-30

This is a bug-focused TODO list (major -> minor). Each item includes evidence (file:line) and a suggested fix/verify.

Legend:

- P0: Blocks build/CI or core user flows
- P1: Major correctness/reliability issues
- P2: Important quality gaps (DX/UX/perf) with real risk
- P3: Minor issues / cleanup

Status: ✅ All items below have been completed as of 2026-01-31.

Summary of Completed Work

- Fixed scan pipeline + UI export flow; artifacts now written in Tauri app data dir and export/view wired in UI.
- Added full LLM pipeline integration with provider metadata, model discovery, Venice text+code filtering, headers/org/project, optional auth, retries, and safer JSON parsing.
- Implemented DeepScan categorization + category normalization, improved priority ordering, and corrected local/LLM session log semantics.
- Hardened path safety and error handling (`read_file`, `load_report`, run_id validation) plus Windows path filtering.
- UI upgrades: dynamic scan progress, evidence rendering by type, HashRouter for Tauri, better Thinktank typing + accessibility, provider profile list/edit/delete, and toast error surfacing.
- Test suite stabilized: fixed git test signing, ingestion integration test timing, prompts server test shim, and added real UI tests.

Where to Continue (for future AI)

- No open TODOs remain in this file. Start by reviewing git status and recent changes, then re-run:
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
  - `cd apps/workbench && npm test`
  - `cd apps/workbench && npm run lint`
- If new features are requested, create a fresh TODO section below and track new work items with evidence/fix/verify blocks.

## P0 (Ship / CI Blockers)

1) P0 - `cargo test --workspace` currently fails ✅

- Evidence: `crates/hqe-git/src/lib.rs:185` (implementation), `crates/hqe-git/src/lib.rs:435` (failing test)
- Why: `GitRepo::current_branch()` uses `git rev-parse --abbrev-ref HEAD`, which fails when `HEAD` doesn’t exist (common in freshly `git init` repos). Test doesn’t check that `git commit` succeeded.
- Fix:
  - Make `current_branch()` handle empty repos: prefer `git symbolic-ref --short HEAD` and fall back to rev-parse when needed.
  - In test, check command exit status, or create a commit robustly (write file + `git add` + `git commit`) and assert success.
- Verify: `cargo test -p hqe-git --lib` and then `cargo test --workspace`

1) P0 - Root preflight (clippy with `-D warnings`) fails ✅

- Evidence: `crates/hqe-openai/src/lib.rs:27`, `crates/hqe-openai/src/analysis.rs:28`
- Why: `#![warn(missing_docs)]` + `cargo clippy -- -D warnings` turns missing docs into hard errors.
- Fix: add minimal module docs for `analysis` and docs for `OpenAIAnalyzer::new`, or explicitly `#[allow(missing_docs)]` (prefer docs).
- Verify: `cargo clippy --workspace -- -D warnings`

1) P0 - UI scan/export flow is logically broken (artifacts not written) ✅

- Evidence: `apps/workbench/src-tauri/src/commands.rs:36` (scan returns report only), `apps/workbench/src/screens/ReportScreen.tsx:257` (export buttons exist)
- Why: UI calls `scan_repo` but backend doesn’t write `./hqe-output/...` artifacts; yet `load_report`/`export_artifacts` assume those files exist.
- Fix:
  - In `scan_repo`, write artifacts using `hqe_artifacts::ArtifactWriter` to a stable, app-specific output dir (Tauri app data dir), and return `ScanResult` or (report + run_dir path).
  - Wire UI buttons to Tauri commands and show success/errors.
- Verify: run a UI scan, confirm report + manifest files exist and export works.

1) P0 - LLM findings are silently dropped from `DeepScanResults` ✅

- Evidence: `crates/hqe-core/src/scan.rs:399`
- Why: `deep_scan_results.security` includes only findings where `category == "Security"` and all other sections are empty; LLM analysis returns multi-category findings/todos.
- Fix:
  - Implement a categorizer (map string categories to `DeepScanResults` buckets).
  - Alternatively, store findings as a single list in report and let UI group.
- Verify: run LLM scan and ensure findings appear under Security/Code Quality/Frontend/Backend/Testing.

1) P0 - “LLM enabled” scan path is still incomplete at the product level ✅

- Evidence: `crates/hqe-core/src/scan.rs:66` (pipeline), `crates/hqe-openai/src/analysis.rs:33` (LLM analyzer)
- Why: LLM analysis exists, but report generation/session log still assumes “Local Analysis” and the UI always shows “Local-Only Analysis”.
- Fix:
  - Include provider mode in the returned data (manifest/provider info).
  - Update `SessionLog` + UI banner logic.
- Verify: LLM scan clearly displays provider name/model and no longer shows “Local-only” banner.

## P1 (Major Correctness / Reliability)

1) P1 - `RepoScanner::read_file` returns an error for missing files (should be `Ok(None)`) ✅

- Evidence: `crates/hqe-core/src/repo.rs:775`
- Why: `canonicalize()` runs before checking existence; canonicalize fails on missing paths.
- Fix: check `full_path.exists()` before canonicalize, or canonicalize parent/root only.
- Verify: unit test `read_file("missing")` returns `Ok(None)` (no error)

1) P1 - Tauri `load_report` returns error instead of `Ok(None)` when report is missing ✅

- Evidence: `apps/workbench/src-tauri/src/commands.rs:172`
- Why: `canonicalize()` is executed before existence check, causing “Report not found” error even for normal “not generated yet” cases.
- Fix: check existence before canonicalize (and keep the prefix check after canonicalize).
- Verify: calling `load_report` for a non-existent run_id returns `null` to UI.

1) P1 - `export_artifacts` lacks run_id validation (path traversal risk) ✅

- Evidence: `apps/workbench/src-tauri/src/commands.rs:211`
- Why: `run_id` is interpolated into a path without applying `is_valid_run_id()`.
- Fix: validate `run_id` and also canonicalize/containment-check source before copying.
- Verify: attempt to pass `../`-style input should be rejected (even though slashes are currently allowed/blocked inconsistently).

1) P1 - OpenAI/Venice client does not support provider profile headers / org / project ✅

- Evidence: `crates/hqe-openai/src/lib.rs:241` (build_headers only sets Authorization/Content-Type), profile definition in `crates/hqe-protocol/src/models.rs:39`
- Why: profile supports custom headers, org, project, but client ignores them; breaks OpenRouter-like requirements and local LLMs that need non-Bearer auth.
- Fix: extend `ClientConfig` to include `headers` and optional auth mode; thread through from `ProviderProfile`.
- Verify: configure a provider requiring extra headers; requests succeed.

1) P1 - OpenAI/Venice client always sends `Authorization: Bearer ...` ✅

- Evidence: `crates/hqe-openai/src/lib.rs:240`
- Why: local OpenAI-compatible endpoints may not require auth; sending `Bearer` can break some servers.
- Fix: make Authorization optional (only set if key is non-empty) and allow user-defined auth headers via profile headers.
- Verify: local endpoint works with empty key.

1) P1 - `ClientConfig.max_retries` is unused ✅

- Evidence: `crates/hqe-openai/src/lib.rs:47` (config), `crates/hqe-openai/src/lib.rs:262` (chat sends once)
- Fix: implement bounded retry with backoff on retryable status codes/timeouts.
- Verify: simulate transient failures (mockito) and ensure retries happen.

1) P1 - Chat schema support is too narrow for “OpenAI-compatible” ✅

- Evidence: `crates/hqe-openai/src/lib.rs:114` (Message content is required String)
- Why: OpenAI-compatible APIs can return tool calls or omit `content`; Venice spec explicitly allows assistant messages with `tool_calls` and without content.
- Fix: model message content as `Option<...>` and add tool_calls fields (or use `serde_json::Value` for message content).
- Verify: handle responses containing tool_calls without deserialization errors.

1) P1 - LLM analyzer JSON extraction is fragile ✅

- Evidence: `crates/hqe-openai/src/analysis.rs:80`
- Why: picks first `{` and last `}`; will break if model outputs extra braces (common in code).
- Fix: require strict JSON output (already requested) + add a real JSON object extraction strategy (e.g., scan for balanced braces, or parse from ```json fences).
- Verify: unit tests with prefix/suffix text and braces in strings.

1) P1 - Venice models discovery UI call likely doesn’t match Tauri command signature ✅

- Evidence: `apps/workbench/src/screens/SettingsScreen.tsx:50`, command signature `apps/workbench/src-tauri/src/commands.rs:252`
- Why: Tauri command takes `input: ProviderProfileInput`, but UI sends `{ base_url, headers, ... }` without nesting under `input` (contrast with `execute_prompt` which does nest `request`).
- Fix: align all invoke payload shapes consistently and add a small e2e smoke test that exercises the commands.
- Verify: “Discover Models” works and returns a list for Venice `/models?type=text`.

1) P1 - Scan UI likely has invoke argument naming inconsistencies ✅

- Evidence: `apps/workbench/src/screens/WelcomeScreen.tsx:15`, `apps/workbench/src/screens/ScanScreen.tsx:65`
- Why: keys like `repoPath` are used while Rust args are `repo_path`; if Tauri doesn’t auto-rename, UI commands will fail at runtime.
- Fix: standardize payload keys (snake_case or confirmed Tauri rename strategy) and add runtime checks/logging.
- Verify: manual UI run; add a basic integration test in Rust for command deserialization if possible.

1) P1 - Report UX mislabels all reports as local-only ✅

- Evidence: `apps/workbench/src/screens/ReportScreen.tsx:109`
- Fix: only show banner when scan was local-only (requires storing scan mode/provider info alongside report).
- Verify: local scan shows banner; LLM scan does not.

1) P1 - Tauri prompt library location is brittle ✅

- Evidence: `apps/workbench/src-tauri/src/prompts.rs:131`
- Why: hardcoded `../../../prompts` depends on working directory; breaks depending on how the app is launched.
- Fix: use `resource_dir` in production; in dev, resolve repo root robustly (e.g., env var, compile-time include, or walk up looking for `prompts/`).
- Verify: prompts load in `tauri dev` and in built app.

1) P1 - Scan pipeline artifact paths are misleading ✅

- Evidence: `crates/hqe-core/src/scan.rs:464` (placeholder export), CLI uses `hqe_artifacts::ArtifactWriter` instead.
- Fix: single source of truth for artifact writing and paths (either pipeline writes, or pipeline returns report and caller writes).
- Verify: returned `ScanResult.artifacts` matches actual files on disk.

## P2 (Important Quality / UX / Perf)

1) P2 - Deep scan categorization is under-specified (stringly-typed) ✅

- Evidence: `crates/hqe-core/src/models.rs:327` (Finding.category is String)
- Why: report correctness depends on string matching; easy for LLM to output “security” vs “Security”.
- Fix: define an enum for finding categories (or normalize/canonicalize categories on ingestion).
- Verify: LLM output with lowercase still lands in correct bucket.

1) P2 - Scan health score/top priorities ordering is arbitrary ✅

- Evidence: `crates/hqe-core/src/scan.rs:355` (`take(3)` without sorting)
- Fix: sort by severity/risk and/or confidence before selecting top priorities.

1) P2 - UI progress bar is hard-coded to 40% ✅

- Evidence: `apps/workbench/src/screens/ScanScreen.tsx:214`
- Fix: use `progress` from `useScanStore` to drive width and show percent.

1) P2 - Type safety gaps in UI (multiple `any` and weak types) ✅

- Evidence: `apps/workbench/src/screens/ScanScreen.tsx:17`, `apps/workbench/src/screens/SettingsScreen.tsx:50`, `apps/workbench/src/screens/ThinktankScreen.tsx:14`
- Fix: add explicit types for Tauri responses; share types between Rust and TS (e.g., generate TS from Rust JSON schema).
- Verify: `npm run lint` has zero warnings.

1) P2 - Thinktank prompt execution sends only string args ✅

- Evidence: `apps/workbench/src/screens/ThinktankScreen.tsx:75`
- Why: schema can have types beyond string; everything is sent as string, which can break tools expecting numbers/bools/objects.
- Fix: render inputs based on JSON schema types and serialize accordingly.

1) P2 - React hook dependency warning in Thinktank ✅

- Evidence: `apps/workbench/src/screens/ThinktankScreen.tsx:31` (lint warning)
- Fix: memoize `loadPrompts` via `useCallback` or inline it in the effect.

1) P2 - Router choice likely wrong for Tauri production ✅

- Evidence: `apps/workbench/src/App.tsx:2`
- Why: `BrowserRouter` can break on refresh/deep-link in file/protocol contexts.
- Fix: switch to `HashRouter` or configure Tauri/Vite routing fallback explicitly.
- Verify: built app can deep-link to `/report` without blank screen.

1) P2 - `should_exclude_file` path filtering is case-sensitive and partial ✅

- Evidence: `crates/hqe-core/src/redaction.rs:229`
- Fix: compare on lowercased path; handle Windows separators (`\\`).

1) P2 - Provider discovery filters for non-Venice providers are heuristic-only ✅

- Evidence: `crates/hqe-openai/src/provider_discovery.rs:410`
- Fix: prefer explicit model metadata when available; at minimum add denylist entries for common non-chat models.
- Verify: `/models` list doesn’t include embeddings/asr/tts models in UI.

1) P2 - Settings UX: no “edit existing profile / list profiles / delete profile” flows ✅

- Evidence: `apps/workbench/src/screens/SettingsScreen.tsx` (only create/test)
- Fix: show `list_provider_profiles`, allow select/edit/delete, and persist discovered models per profile.

1) P2 - Prompts MCP server in `prompts/prompts/server` is not testable in this repo as-is ✅

- Evidence: `prompts/prompts/server/package.json:67` (uses `tsx`), running tests fails because `tsx` missing in `.bin`
- Fix: ensure `prompts/prompts/server` has correct dependency install (or explicitly mark it as vendored and exclude from repo QA).
- Verify: `cd prompts/prompts/server && npm test` passes.

1) P2 - UI report rendering assumes `Evidence` is always `file_line` ✅

- Evidence: `apps/workbench/src/screens/ReportScreen.tsx:165` (uses `finding.evidence.file/line/snippet` without switching by `evidence.type`)
- Fix: render evidence conditionally for `file_function` and `reproduction`.

## P3 (Minor / Cleanup)

1) P3 - Duplicate attributes / comments in git wrapper ✅

- Evidence: `crates/hqe-git/src/lib.rs:136` (duplicated `#[instrument]` and doc comments)
- Fix: remove duplicates; keep one `#[instrument]`.

1) P3 - Docs mismatch: tech stack claims React 19 but app uses React 18 ✅

- Evidence: `docs/tech-stack.md:19`, `apps/workbench/package.json:15`
- Fix: update docs to match or bump dependency intentionally.

1) P3 - Remove stray backup file from repo ✅

- Evidence: `crates/hqe-core/src/repo.rs.bak`
- Fix: delete or move to docs; ensure it’s gitignored.

1) P3 - Replace placeholder UI tests with real coverage ✅

- Evidence: `apps/workbench/src/__tests__/smoke.test.ts:1`
- Fix: add tests for Settings (model discovery), Scan (LLM vs local), Report (evidence rendering), Thinktank (prompt execution).

1) P3 - Tighten error surfacing to user ✅

- Evidence: various UI screens catch and `console.error` only (e.g., `apps/workbench/src/screens/WelcomeScreen.tsx:20`)
- Fix: route errors through Toast consistently; include actionable hints.
