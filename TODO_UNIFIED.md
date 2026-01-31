# Unified Bug TODO (hqe-workbench)

Generated: 2026-01-30

This is a bug-focused TODO list (major -> minor). Each item includes evidence (file:line) and a suggested fix/verify.

Legend:

- P0: Blocks build/CI or core user flows
- P1: Major correctness/reliability issues
- P2: Important quality gaps (DX/UX/perf) with real risk
- P3: Minor issues / cleanup

Status: ✅ Docs/CI sweep completed as of 2026-01-31; remaining open items tracked below.

Summary of Completed Work

- Fixed scan pipeline + UI export flow; artifacts now written in Tauri app data dir and export/view wired in UI.
- Added full LLM pipeline integration with provider metadata, model discovery, Venice text+code filtering, headers/org/project, optional auth, retries, and safer JSON parsing.
- Implemented DeepScan categorization + category normalization, improved priority ordering, and corrected local/LLM session log semantics.
- Hardened path safety and error handling (`read_file`, `load_report`, run_id validation) plus Windows path filtering.
- UI upgrades: dynamic scan progress, evidence rendering by type, HashRouter for Tauri, better Thinktank typing + accessibility, provider profile list/edit/delete, and toast error surfacing.
- Test suite stabilized: fixed git test signing, ingestion integration test timing, prompts server test shim, and added real UI tests.
- Repo/community hygiene: fixed CI workflows, added GitHub templates and standard docs, and ensured `npm run preflight` passes (Rust tests/clippy/fmt + Workbench lint/tests).

Where to Continue (for future AI)

- No remaining open TODOs in this file.
- If you change provider/client behavior, re-run `npm run preflight`.

---

## Open TODO (Starting 2026-01-31 Sweep 2)

These items were discovered during a repo-wide bug sweep focused on provider discovery,
Venice/OpenAI compatibility, and UI model population.

## Sweep 2: P1 (Major Correctness / Reliability)

1) P1 - Model discovery cannot use stored API keys/headers; auto-population fails for saved profiles ✅

- Evidence:
  - Selecting a profile wipes the API key and ignores stored headers: `apps/workbench/src/screens/SettingsScreen.tsx:54-63`
  - Discovery always calls `discover_models` with `headers: {}` and `api_key` from input only: `apps/workbench/src/screens/SettingsScreen.tsx:103-113`
  - Profile manager only writes keys when `api_key` is provided: `crates/hqe-openai/src/profile.rs:359-377`
- Why it matters: users with saved keys (or providers requiring extra headers) cannot discover models unless they re-enter secrets; renaming a profile can orphan the key.
- Fix:
  - Load stored API key via `get_provider_profile` when selecting a profile.
  - Preserve profile headers in state and forward them to `discover_models`.
  - Preserve key on rename by re-saving with the stored key when no new key is provided.
- Verify:
  - `apps/workbench/src/__tests__/settings.test.tsx` covers stored key/headers discovery path.

1) P1 - Local LLMs on LAN (http) are rejected by URL sanitization ✅

- Evidence: `crates/hqe-openai/src/provider_discovery.rs:342-351` only allows `http` for `localhost/127.0.0.1/::1`.
- Why it matters: the product requirement includes “localized LLMs”; many are hosted on LAN IPs (`http://192.168.x.x:8000/v1`) and are currently blocked.
- Fix:
  - Allow `http` for RFC1918/private IPv4 ranges and IPv6 unique-local addresses.
- Verify:
  - New unit tests in `crates/hqe-openai/src/provider_discovery.rs`.

1) P1 - LLM scan forces JSON response format with no capability gating ✅

- Evidence:
  - Analyzer always uses `response_format: json_object`: `crates/hqe-openai/src/analysis.rs:41-60`
  - Model capabilities are discovered but not surfaced to the UI: `crates/hqe-openai/src/provider_discovery.rs:90-107` and `apps/workbench/src/types.ts:81-90`
- Why it matters: selecting a model that doesn’t support JSON/schema responses can cause hard failures on scan.
- Fix:
  - Added fallback in `OpenAIAnalyzer` to retry without `response_format` when the provider rejects JSON mode.
- Verify:
  - Models without JSON mode return output via fallback path (no hard failure).

1) P1 - Venice/OpenAI JSON schema response format is unsupported in the client ✅

- Evidence:
  - Venice spec includes `response_format: { type: "json_schema", json_schema: ... }`: `docs/swagger.yaml:821-849`
  - Client only supports `json_object` and `text`: `crates/hqe-openai/src/lib.rs:124-134`
- Fix:
  - Added `ResponseFormat::JsonSchema { json_schema }` to the client serializer.
- Verify:
  - JSON schema requests serialize per spec when used.

## Sweep 2: P2 (Important Quality / UX / Perf)

1) P2 - ChatRequest lacks several Venice/OpenAI text parameters ✅

- Evidence:
  - Spec supports `max_completion_tokens`, `logprobs`, `top_logprobs`, `frequency_penalty`, etc.: `docs/swagger.yaml:70-100`
  - Client only exposes `temperature`, `max_tokens`, `response_format`: `crates/hqe-openai/src/lib.rs:108-121`
- Fix:
  - Added optional fields to `ChatRequest` (penalties, logprobs, top_p/top_k, max_completion_tokens, stop, seed, user, cache controls, reasoning, stream, tools, venice_parameters).
  - Updated all ChatRequest call sites and rate-limit estimation to prefer `max_completion_tokens`.
- Verify:
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`

1) P2 - Venice-specific request parameters are not supported ✅

- Evidence: spec defines `venice_parameters` and `parallel_tool_calls`: `docs/swagger.yaml:810-819`
- Fix:
  - Added Venice options to scan config and analyzer; UI/CLI can now pass `venice_parameters` and `parallel_tool_calls`.
- Verify:
  - Venice options visible in Scan UI when a Venice profile is selected.

1) P2 - Message content only supports string; spec allows array content parts ✅

- Evidence:
  - Spec allows `content` to be a string or array of content objects: `docs/swagger.yaml:100-108`
  - Previously, client `Message.content` was `Option<String>` which fails for array content.
- Fix:
  - Implemented `MessageContent` (serde untagged) supporting either `Text(String)` or `Parts(Vec<serde_json::Value>)`.
  - Added `MessageContent::to_text_lossy()` to extract text from structured content arrays (text-only).
  - Updated analyzers and prompt execution paths to build messages via `MessageContent` and extract response text safely:
    - `crates/hqe-openai/src/analysis.rs`
    - `crates/hqe-openai/src/lib.rs`
    - `apps/workbench/src-tauri/src/prompts.rs`
    - `cli/hqe/src/main.rs`
- Verify:
  - `npm run preflight`

1) P2 - Model discovery output drops traits in UI, preventing “text-only / schema-capable” filtering ✅

- Evidence:
  - `DiscoveredModel` includes `traits` in Rust: `crates/hqe-openai/src/provider_discovery.rs:90-94`
  - UI `ProviderModel` has only `id`/`name`: `apps/workbench/src/types.ts:81-84`
- Fix:
  - Extended UI model types to include `traits`.
  - Settings UI now marks schema-capable models and warns when selected model lacks schema support.
- Verify:
  - Discover models shows “(JSON)” badges where applicable and warning text for non-schema models.

1) P2 - Non-Venice providers rely on ID heuristics for text model filtering ✅

- Evidence: `is_chat_model_id` heuristics only: `crates/hqe-openai/src/provider_discovery.rs:418-448`
- Fix:
  - Preserve `model_type` on discovered models and prefer explicit type/modality when filtering.
- Verify:
  - Unit tests updated in `crates/hqe-openai/src/provider_discovery.rs`.

---

## Open TODO (Starting 2026-01-31)

These items were discovered after the initial stabilization pass.

Status: ✅ All items in this section have been completed as of 2026-01-31.

Summary of Work Completed (2026-01-31)

- Fixed broken Thinktank prompts and TOML parsing; repaired/added `{{args}}` where needed:
  - `prompts/code-review.toml`
  - `prompts/criticalthink/criticalthink.toml`
  - `prompts/cli-prompt-library/commands/prompts/improve.toml`
  - `prompts/cli-prompt-library/commands/testing/edge-cases.toml`
- Reduced prompt loader noise and runtime by skipping vendored directories:
  - `crates/hqe-mcp/src/loader.rs`
- Wrote an exhaustive prompt audit with intended vs actual behavior:
  - `docs/PROMPTS_AUDIT.md`

Where to Continue (next AI / next pass)

- No open TODOs remain in this file again. Start by reviewing git status, then re-run:
  - `cargo test --workspace`
  - `cargo clippy --workspace -- -D warnings`
  - `cd apps/workbench && npm test`
  - `cd apps/workbench && npm run lint`

## Completed: P1 (Major Correctness / Reliability)

1) P1 - Prompt library contains invalid or non-functional prompts (breaks Thinktank UX) ✅

- Evidence:
  - `prompts/cli-prompt-library/commands/prompts/improve.toml` (invalid TOML; missing closing string)
  - `prompts/cli-prompt-library/commands/testing/edge-cases.toml` (invalid TOML escape sequence)
  - `prompts/code-review.toml` (no `{{args}}`; cannot accept diff input)
  - `prompts/criticalthink/criticalthink.toml` (references “previous response” but Thinktank executes as a single-shot prompt)
- Fix:
  - Repair TOML syntax.
  - Update “code-review” to accept a pasted diff via `{{args}}`.
  - Update “criticalthink” to accept the text-to-critique via `{{args}}`.
- Verify:
  - `apps/workbench` Thinktank shows these prompts and renders an args input.
  - Executing each prompt produces sensible output without hallucinating missing context.

1) P1 - Prompt loader scans vendored directories (log spam + perf) ✅

- Evidence: `crates/hqe-mcp/src/loader.rs:90` scans all `.toml/.yaml/.yml` including `node_modules/` under `prompts/prompts/server/` once deps are installed.
- Fix: add ignore filters (at least `node_modules`, `dist`, `.git`) via `WalkDir::filter_entry`.
- Verify: `get_available_prompts` stays fast and doesn’t emit a wall of warnings after `npm i` in `prompts/prompts/server`.

## Completed: P2 (Important Quality / UX / Perf)

1) P2 - Prompt library has mixed “LLM-only” vs “agent-with-tools” prompts ✅

- Evidence: `prompts/conductor/*.toml`, `prompts/cli-security/**` include instructions requiring tool execution, file writes, or GitHub Actions env.
- Fix:
  - Hide agent/tool prompts by default in Thinktank UI (heuristic by prompt name prefix), with an opt-in toggle to show them.
  - Add an “AGENT” badge and an in-context warning banner when selecting one.
- Verify:
  - Thinktank shows only LLM-only prompts by default.
  - Toggling “Show agent/tool prompts” reveals `conductor_*` and `cli_security_*` prompts.

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

## Completed (Batch 2): P1 (Major Correctness / Reliability)

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

## Completed (Batch 2): P2 (Important Quality / UX / Perf)

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

---

## Docs/CI Sweep (Completed 2026-01-31)

- ✅ Added missing GitHub templates:
  - `.github/pull_request_template.md`
  - `.github/ISSUE_TEMPLATE/*`
- ✅ Added/updated standard docs:
  - `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CREDITS.md`, `LEGAL.md`, `protocol/README.md`, `docs/API.md`, `docs/PROMPTS_AUDIT.md`
  - New: `docs/ARCHITECTURE.md`, `docs/DEVELOPMENT.md`, `PRIVACY.md`, `SUPPORT.md`
- ✅ Updated README to be accurate:
  - Correct repo URL (`AbstergoSweden/HQE-Workbench`), centered banner image, fixed badges/links, removed non-existent commands
- ✅ CI reliability improvements:
  - `.github/workflows/ci.yml` installs `rustfmt`/`clippy`, adds JS tests job, installs Python deps for protocol validation
  - `.github/workflows/security.yml` uses `taiki-e/install-action` for `cargo-audit`
- ✅ Lint/test gate is clean locally:
  - `npm run preflight` passes (Rust tests + clippy + fmt; Workbench eslint + vitest)

Where to Continue

- No remaining open TODOs in this file. If new bugs are found, add a new dated sweep section with evidence/fix/verify blocks.

---

## Open TODO (Starting 2026-01-31 Sweep 3)

These items were discovered during a new repo-wide sweep after the Docs/CI pass.

## Sweep 3: P1 (Major Correctness / Reliability)

1) P1 - CLI `export` command is a stub (prints “Not yet implemented”) ✅

- Evidence: `cli/hqe/src/main.rs:737`
- Why it matters: users can invoke `hqe export` but cannot actually export artifacts; CLI promises a feature that doesn't work.
- Fix:
  - Implemented export to locate `hqe_run_{run_id}` in `./hqe-output` or app data dir and copy all artifacts to `--out`.
  - Added run_id validation and clear error when artifacts are missing.
- Verify:
  - `./target/release/hqe export RUN_ID --out ./hqe-exports` writes files.

## Sweep 3: P2 (Important Quality / UX / Perf)

1) P2 - Prompt server LLM self-check is stubbed (auto-pass), so validation gates are ineffective when enabled ✅

- Evidence: `prompts/prompts/server/src/gates/core/gate-validator.ts:245-321`
- Why it matters: enabling semantic validation claims to use LLM self-check, but it never calls an LLM and always passes.
- Fix:
  - Implemented LLM self-check in `GateValidator` using OpenAI-compatible (and Anthropic) endpoints.
  - Restored content/pattern checks as baseline validation when LLM is not enabled.
  - Added prompt template support (`{{content}}`, `{{metadata}}`, `{{execution_context}}`) and JSON parsing for pass/score/feedback.
- Verify:
  - Enable `analysis.semanticAnalysis.llmIntegration.enabled=true` and confirm LLM calls occur and gate results change.

1) P2 - Tauri Thinktank prompt execution ignores profile timeout setting ✅

- Evidence: `apps/workbench/src-tauri/src/prompts.rs:95` hardcodes `timeout_seconds: 120`
- Why it matters: user-configured provider timeouts are ignored for prompt execution.
- Fix:
  - Use `profile.timeout_s` when constructing `ClientConfig` in the Tauri prompt executor.
- Verify:
  - A profile with low timeout causes prompt execution to time out as configured.

Status: ✅ All items in Sweep 3 completed as of 2026-01-31.

---

## Open TODO (Starting 2026-01-31 Sweep 4)

These items were added per latest CI and README findings.

## Sweep 4: P1 (Major Correctness / Reliability)

1) P1 - GitHub Actions `test-rust` fails on glib-sys (missing GLib dev packages) ✅

- Evidence:
  - CI run: `https://github.com/AbstergoSweden/HQE-Workbench/actions/runs/21536040437/job/62061863327`
  - Log: `PKG_CONFIG_PATH` not set; `glib-2.0.pc` missing.
- Fix:
  - Install `libglib2.0-dev` and `pkg-config` before Rust build in `.github/workflows/ci.yml`.
- Verify:
  - CI `test-rust` job passes on Ubuntu runners.

## P3 (Minor / Docs)

1) P3 - README OpenSSF badge shows “invalid repo path” ✅

- Evidence: README badge rendered as invalid path.
- Fix:
  - Updated OpenSSF Scorecard badge URL to use correct repo case (`AbstergoSweden/HQE-Workbench`).
- Verify:
  - Badge renders correctly for public repo.

1) P3 - Placeholder scan across docs (contacts/links/examples) ✅

- Evidence:
  - Placeholder strings in `LEGAL.md`, `CONTRIBUTING.md`, `docs/provider-config.md`, `docs/HOW_TO.md`, `docs/tech-stack.md`.
- Fix:
  - Replaced maintainer identity/contact info with real values.
  - Updated example URLs and commands to actionable, non-placeholder guidance.
  - Clarified future/planned components without “placeholder” language.
- Verify:
  - `rg "YOUR_|replace with|placeholder" README.md ABOUT.md LEGAL.md SECURITY.md SUPPORT.md CODE_OF_CONDUCT.md CONTRIBUTING.md CITATION.cff docs/*.md` returns only non-actionable references (e.g., audit notes).

Status: ✅ All items in Sweep 4 completed as of 2026-01-31.

---

## Open TODO (Starting 2026-01-31 Sweep 5)

Scope: New repo-wide bug sweep (major → minor). Track evidence, fix, verify.

Status: ✅ All items in Sweep 5 completed as of 2026-01-31.

---

## Open TODO (Starting 2026-01-31 Sweep 6)

Scope: Provider edge cases + next-area deep dive.

Status: ⏳ In progress.

## Sweep 6: P2 (Important Quality / UX / Perf)

1) P2 - Base URL normalization breaks when user pastes full chat/models endpoint ✅

- Evidence: `sanitize_base_url` appends `/v1` even if path already includes `/v1/chat/completions` or `/v1/models`: `crates/hqe-openai/src/provider_discovery.rs:325-370`
- Why it matters: providers fail if user pastes a full endpoint URL.
- Fix:
  - Trim `/chat/completions` or `/models` suffix before normalizing path.
- Verify:
  - `sanitize_base_url("https://api.openai.com/v1/chat/completions")` returns `https://api.openai.com/v1`.

1) P2 - Azure OpenAI discovery likely fails (no `/models` support) ✅

- Evidence:
  - Discovery always calls `/models` (OpenAI-style): `crates/hqe-openai/src/provider_discovery.rs:213-265`
  - Azure uses deployments API and requires `api-version` query; `/models` is not standard.
- Why it matters: model discovery in Settings will fail for Azure users.
- Fix:
  - Added `ProviderKind::Azure` and skip discovery for Azure hosts with a friendly error message.
- Verify:
  - `npm run preflight` verifies `provider_discovery.rs` changes.

1) P2 - CLI `export` cannot find runs created with custom `--out` directory ✅

- Evidence: `export_run` searches only `./hqe-output` and app data dir: `cli/hqe/src/main.rs:780-815`
- Why it matters: if a scan uses `--out /custom/path`, export will fail to find the run.
- Fix:
  - Added `--from <DIR>` arg to `hqe export` to specify the source directory for artifacts.
- Verify:
  - `npm run preflight` verifies CLI compilation and tests.

1) P2 - CLI `patch` command is a stub (no patch generation) ✅

- Evidence: `handle_patch` prints "not yet implemented": `cli/hqe/src/main.rs:815-850`
- Why it matters: CLI advertises patching but provides no implementation.
- Fix:
  - Implemented `handle_patch` using system `patch` command to apply diffs from `hqe_report.json`.
- Verify:
  - `npm run preflight` verifies CLI compilation and tests.

1) P2 - CLI `config add` requires `--key` even for local providers ✅

- Evidence: `cli/hqe/src/main.rs` CLI args enforce `--key` for add.
- Why it matters: local LLM servers often require no API key, but CLI cannot create such profiles.
- Fix:
  - Made `--key` optional when base URL is local/private; enforced for non-local providers.
- Verify:
  - `hqe config add local --url http://127.0.0.1:1234/v1 --model llama3` succeeds without a key.

## Sweep 6: P1 (Major Correctness / Reliability)

1) P1 - Local/OpenAI-compatible providers without API keys are blocked (LLM scans + prompts) ✅

- Evidence:
  - Tauri scan requires stored API key: `apps/workbench/src-tauri/src/commands.rs:70-88`
  - Tauri prompts require stored API key: `apps/workbench/src-tauri/src/prompts.rs:80-90`
  - CLI scan requires API key: `cli/hqe/src/main.rs:620-639`
  - CLI prompt/test requires API key: `cli/hqe/src/main.rs:245-275`, `cli/hqe/src/main.rs:885-915`
- Why it matters: local LLM servers typically do not require API keys; current flow blocks them.
- Fix:
  - Added `is_local_or_private_base_url` helper in `crates/hqe-openai/src/provider_discovery.rs`.
  - Allowed empty API keys for local/private base URLs in CLI + Tauri scan/prompt/test flows.
- Verify:
  - Local LLM profile (localhost/LAN) works with no stored key; remote providers still require keys.

1) P1 - Provider profiles cannot configure required headers/org/project/timeout (UI + CLI) ✅

- Evidence:
  - Settings UI has no fields for headers/org/project/timeout; values are preserved but not editable: `apps/workbench/src/screens/SettingsScreen.tsx`
  - CLI `config add` only accepts name/url/key/model (no headers/org/project/timeout): `cli/hqe/src/main.rs:856-894`
- Why it matters: Azure OpenAI requires `api-key` header; OpenRouter and enterprise OpenAI often require extra headers and org/project IDs.
- Fix:
  - UI: added advanced fields for custom headers (JSON), organization, project, and timeout in `SettingsScreen`.
  - CLI: added `--header`, `--organization`, `--project`, `--timeout` flags in `hqe config add`.
- Verify:
  - Azure/OpenRouter profiles can be configured without manual JSON edits; discovery and scans succeed.
