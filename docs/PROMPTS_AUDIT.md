# Prompt System Audit (hqe-workbench)

Date: 2026-01-31

This document is a deep dive into every prompt currently present in this repository that can influence runtime behavior:

1) **Workbench "Thinktank" prompt templates** under `mcp-server/**/*.toml` (loaded by the Workbench app).
2) **LLM scan prompts** embedded in Rust under `crates/hqe-openai/src/prompts.rs` (used by the HQE scan pipeline).

It focuses on: intended behavior, actual behavior in this codebase, missing inputs, logic lapses, and compatibility with “single-shot, text-only models” execution.

---

## 1) How Prompts Work In This Repo (Actual Runtime Behavior)

### 1.1 Workbench Thinktank (Tauri)

Key files:
- `apps/workbench/src-tauri/src/prompts.rs`
- `crates/hqe-mcp/src/loader.rs`

Flow:
1) `get_available_prompts` locates a prompts root directory, then loads prompt tools from disk.
2) Each prompt file becomes a “tool” with:
   - `name` derived from its relative path (slashes -> underscores, hyphens -> underscores)
   - `description` (optional)
   - `input_schema` (JSON schema built from args)
3) In Thinktank, the user selects a prompt + provides args, then `execute_prompt`:
   - Performs naive `{{key}}` string replacement (no escaping).
   - Sends the resulting prompt as a *single* OpenAI-chat “user” message to the selected provider/model.
   - No tool execution. No file IO. No git access. No multi-turn memory unless the user pastes it into `{{args}}`.

Important constraints for prompt authors:
- If a prompt needs user-provided content, it MUST include `{{args}}` (or explicit args), or the UI cannot collect inputs.
- If a prompt references “previous response”, “run this command”, “write files”, “call tool X”, it will not work in Thinktank unless the user manually pastes those artifacts or the app grows tool support.

### 1.2 Prompt Loader Rules (Template Files)

Loader: `crates/hqe-mcp/src/loader.rs`

Accepted formats:
- `.toml`, `.yaml`, `.yml`

Schema expectations (current):
- `prompt`: required string
- `description`: optional string
- `args`: optional list of `{ name, description?, required? }` (currently all args are exposed as `type: string` in JSON schema)

Implicit args behavior:
- If the template contains `{{args}}` and no explicit `args` include `args`, the loader auto-adds a required string input named `args`.

Naming:
- A file at `mcp-server/cli-prompt-library/commands/docs/write-readme.toml` becomes:
  `cli_prompt_library_commands_docs_write_readme`

### 1.3 LLM Scan Prompts (HQE analysis pipeline)

Key files:
- `crates/hqe-openai/src/prompts.rs`
- `crates/hqe-openai/src/analysis.rs`

Flow:
1) The scan pipeline builds an `EvidenceBundle` (tree, snippets, local findings).
2) The LLM analyzer sends:
   - a *system* message `HQE_SYSTEM_PROMPT`
   - a *user* message `build_analysis_json_prompt(bundle)`
   - `response_format = json_object` (OpenAI-compatible JSON mode, when supported)
3) The result is parsed into structured findings/todos and normalized downstream.

---

## 2) Repository Prompt Inventory

### 2.1 Workbench Thinktank prompt files (`mcp-server/**/*.toml`)

There are 42 TOML prompt templates under `mcp-server/`:

- `mcp-server/cli-prompt-library/commands/architecture/*.toml` (5)
- `mcp-server/cli-prompt-library/commands/code-review/*.toml` (4)
- `mcp-server/cli-prompt-library/commands/debugging/*.toml` (3)
- `mcp-server/cli-prompt-library/commands/docs/*.toml` (4)
- `mcp-server/cli-prompt-library/commands/learning/*.toml` (5)
- `mcp-server/cli-prompt-library/commands/prompts/*.toml` (4)
- `mcp-server/cli-prompt-library/commands/testing/*.toml` (4)
- `mcp-server/cli-prompt-library/commands/writing/*.toml` (4)
- `mcp-server/cli-security/commands/security/*.toml` (2)
- `mcp-server/conductor/*.toml` (5)
- `mcp-server/criticalthink/criticalthink.toml` (1)
- `mcp-server/code-review.toml` (1)

### 2.2 LLM scan prompts (Rust)

- `HQE_SYSTEM_PROMPT`
- `build_analysis_json_prompt`
- `build_scan_prompt` (used by scan/report flows that want the human-readable v3 report)
- `build_patch_prompt`
- `build_test_prompt`

---

## 3) Prompt-by-Prompt Deep Dive (Thinktank Templates)

Notes:
- “Inputs” refers to what Thinktank can provide via `{{args}}` (or explicit schema args).
- “Fits Thinktank?” indicates if the prompt can succeed with *only* the text LLM call.

### 3.1 `mcp-server/code-review.toml` (tool: `code_review`)

Purpose:
- A strict code review rubric with severity classification and location constraints.

Inputs:
- `{{args}}`: unified diff text (required).

Actual behavior in Workbench:
- The entire prompt + diff is sent as a single user message.
- The model must infer filenames/line numbers from the diff content provided.

Fits Thinktank?
- Yes, if the user pastes a real diff and the model follows the “only + / - lines” constraint.

Logic lapses / risks:
- The “only comment on `+`/`-` lines” rule can conflict with how diffs represent context; many good reviews require understanding surrounding code.
- Requires “line numbers”, but unified diffs often do not contain explicit file line numbers unless included by `@@` hunks; the model may fabricate. Recommended: allow “hunk header” references when absolute line numbers are unavailable.

Suggested improvements:
- Allow referencing hunk ranges (e.g. `@@ -12,6 +12,9 @@`) instead of absolute line numbers.
- Add an explicit “If the diff lacks file paths or hunks, ask for a proper unified diff” guardrail.

### 3.2 `mcp-server/criticalthink/criticalthink.toml` (tool: `criticalthink_criticalthink`)

Purpose:
- Structured critique rubric to identify assumptions, fallacies, risks, and revised recommendation.

Inputs:
- `{{args}}`: the text to critique (required).

Actual behavior in Workbench:
- Works as a single-shot critique of the provided text.

Fits Thinktank?
- Yes.

Logic lapses / risks:
- The “language matching” directive can be ambiguous if `{{args}}` language differs from the surrounding conversation.
- Encourages explicit “Pass/Fail” judgments; for nuanced situations, this can create false certainty.

Suggested improvements:
- Add “If the text is mixed-language, use the user’s UI language / most common language.”
- Allow “Partial / Mixed” on Pass/Fail items.

### 3.3 `mcp-server/cli-security/commands/security/analyze.toml` (tool: `cli_security_commands_security_analyze`)

Purpose:
- A taint-analysis workflow intended for an agent with file system access + specialized MCP tools.

Inputs:
- None (no `{{args}}`).

Actual behavior in Workbench:
- Thinktank can run it, but it will be mostly non-functional:
  - it instructs file creation, tool calls (`get_audit_scope`), and cleanup, none of which exist in Thinktank.

Fits Thinktank?
- No (requires an agent runtime with tools).

Logic lapses / risks:
- High hallucination risk: the model may pretend it created files or called tools.
- Mentions `.gemini_security/` and `gemini-cli-security` MCP tooling, which is not wired into Workbench Thinktank.

Suggested improvements:
- Either hide from Thinktank by default, or add a banner: “Requires agent tooling; not supported in Thinktank.”
- If kept, add `{{args}}` so users can paste file lists + diffs and request a purely text-only review.

### 3.4 `mcp-server/cli-security/commands/security/analyze-github-pr.toml` (tool: `cli_security_commands_security_analyze_github_pr`)

Purpose:
- Same taint-analysis flow as above, but explicitly targeting GitHub Actions env vars + PR tooling.

Inputs:
- None (no `{{args}}`).

Fits Thinktank?
- No (requires GitHub Actions environment + PR MCP tools).

Risks:
- Encourages shell expansion syntax like `!{echo $REPOSITORY}`; Thinktank cannot execute that and the model may fabricate values.

Suggested improvements:
- Same as `analyze.toml`: hide/tag, or refactor into “paste PR diff here” variant for Thinktank.

### 3.5 `mcp-server/conductor/*.toml` (tools: `conductor_setup`, `conductor_newTrack`, `conductor_status`, `conductor_implement`, `conductor_revert`)

Purpose:
- “Conductor” methodology prompts designed for an interactive agent that can:
  - read/write state files
  - run git
  - scaffold code
  - resume from state

Inputs:
- None (no `{{args}}`).

Fits Thinktank?
- No (requires an agent runtime with tool execution + file IO).

Logic lapses / risks:
- Contains directives like “validate the success of every tool call” and “write conductor/setup_state.json”.
- Contains model-selection directives (“always select flash model”) that are meaningless in Thinktank and conflict with user expectations.

Suggested improvements:
- Same: tag/hide from Thinktank; consider moving these prompts to an “agent prompts” folder.

### 3.6 `mcp-server/cli-prompt-library/**` (33 prompts)

These are “LLM-only” text prompts that largely follow the same pattern:
- A heading describing the task.
- `{{args}}` inserted near the top.
- A structured rubric / output format guidance.

They generally *do* fit Thinktank’s capabilities (single-shot chat completion).

Common risks across these prompts:
- Many do not specify strict output format; results may vary across models.
- Some include “Show your reasoning / chain-of-thought” style instructions (notably in prompt-engineering best practices). Models may refuse or may reveal long reasoning; prefer “show a brief rationale” instead.
- None of these prompts have a `description` field (UI will show a generic fallback). This is UX-only but makes the library harder to browse.

Below is the full per-file intent summary (each expects `{{args}}` as the main input):

Architecture:
- `mcp-server/cli-prompt-library/commands/architecture/system-design.toml`: full system design rubric (requirements, capacity, architecture, data flow).
- `mcp-server/cli-prompt-library/commands/architecture/design-api.toml`: API design guidance (endpoints, payloads, auth, versioning).
- `mcp-server/cli-prompt-library/commands/architecture/design-database.toml`: database schema/queries/indexing guidance.
- `mcp-server/cli-prompt-library/commands/architecture/design-patterns.toml`: selects/compares design patterns and their tradeoffs.
- `mcp-server/cli-prompt-library/commands/architecture/ddd-modeling.toml`: DDD modeling prompts (bounded contexts, aggregates).

Code review:
- `mcp-server/cli-prompt-library/commands/code-review/security.toml`: security-focused review rubric.
- `mcp-server/cli-prompt-library/commands/code-review/performance.toml`: performance-focused review rubric.
- `mcp-server/cli-prompt-library/commands/code-review/best-practices.toml`: best practices review rubric.
- `mcp-server/cli-prompt-library/commands/code-review/refactor.toml`: refactoring plan suggestions with constraints.

Debugging:
- `mcp-server/cli-prompt-library/commands/debugging/debug-error.toml`: root cause analysis for an error; expects logs/code in `{{args}}`.
- `mcp-server/cli-prompt-library/commands/debugging/trace-issue.toml`: systematic tracing strategy.
- `mcp-server/cli-prompt-library/commands/debugging/performance-profile.toml`: profiling plan and interpretation.

Docs:
- `mcp-server/cli-prompt-library/commands/docs/write-readme.toml`: README drafting.
- `mcp-server/cli-prompt-library/commands/docs/write-contributing.toml`: CONTRIBUTING guide drafting.
- `mcp-server/cli-prompt-library/commands/docs/write-changelog.toml`: changelog entry drafting.
- `mcp-server/cli-prompt-library/commands/docs/write-api-docs.toml`: API documentation drafting.

Learning:
- `mcp-server/cli-prompt-library/commands/learning/eli5.toml`: simple explanation.
- `mcp-server/cli-prompt-library/commands/learning/explain-concept.toml`: structured explanation with examples.
- `mcp-server/cli-prompt-library/commands/learning/compare-tech.toml`: compares technologies.
- `mcp-server/cli-prompt-library/commands/learning/roadmap.toml`: learning roadmap.
- `mcp-server/cli-prompt-library/commands/learning/explain-code.toml`: code explanation with walkthrough.

Prompt engineering:
- `mcp-server/cli-prompt-library/commands/prompts/best-practices.toml`: prompt engineering guidance (contains a “show reasoning” example; consider rewording).
- `mcp-server/cli-prompt-library/commands/prompts/create-template.toml`: creates a reusable prompt template; includes `{{var}}` examples (these are literal examples, not Workbench variables).
- `mcp-server/cli-prompt-library/commands/prompts/improve.toml`: improves a prompt (rewrites + changelog + questions).
- `mcp-server/cli-prompt-library/commands/prompts/optimize-prompt.toml`: optimizes an existing prompt with revisions and rationale.

Testing:
- `mcp-server/cli-prompt-library/commands/testing/generate-unit-tests.toml`: unit test generation plan.
- `mcp-server/cli-prompt-library/commands/testing/generate-e2e-tests.toml`: end-to-end test plan.
- `mcp-server/cli-prompt-library/commands/testing/coverage-analysis.toml`: test coverage analysis rubric.
- `mcp-server/cli-prompt-library/commands/testing/edge-cases.toml`: edge cases enumeration rubric.

Writing:
- `mcp-server/cli-prompt-library/commands/writing/email.toml`: email drafting.
- `mcp-server/cli-prompt-library/commands/writing/presentation.toml`: presentation outline.
- `mcp-server/cli-prompt-library/commands/writing/technical-blog.toml`: technical blog post outline.
- `mcp-server/cli-prompt-library/commands/writing/write-readme.toml`: README drafting (writing template).

---

## 4) Prompt-by-Prompt Deep Dive (LLM Scan Prompts in Rust)

### 4.1 `HQE_SYSTEM_PROMPT` (`crates/hqe-openai/src/prompts.rs`)

Intended goal:
- Define the HQE Engineer persona, forbid fabrication, require evidence, and define output order for the “v3 report”.

Actual behavior:
- Used as the system message for the LLM analyzer (even when requesting JSON-only output).

Logic lapses / risks:
- Potential instruction conflict: the system prompt demands a multi-section report, while JSON analysis demands “JSON only”.
- In practice, `response_format=json_object` usually forces JSON; but when providers don’t implement strict JSON mode, the conflict can cause mixed output.

Suggested improvements:
- Split into two system prompts:
  - one for human-readable v3 report generation
  - one for JSON-only structured output (with explicit “JSON-only overrides section order”)

### 4.2 `build_analysis_json_prompt(bundle)`

Intended goal:
- Force a strict JSON schema (`findings`, `todos`, `blockers`, `is_partial`) and supply evidence bundle context.

Logic lapses / risks:
- Category mismatch between `findings[].category` (e.g. `Security|Bug|Perf...`) and `todos[].category` (e.g. `SEC|BUG|DX...`).
- Evidence guidance (“FileLine preferred”) is good, but the model can still fabricate line numbers if the snippet lacks explicit lines.

Suggested improvements:
- Use one canonical category enum across findings + todos.
- Include an explicit `confidence` field and/or `evidence_quality` to reduce false certainty.

### 4.3 `build_scan_prompt(bundle)`

Intended goal:
- Generate a full HQE v3 report (human-readable) using the EvidenceBundle.

Logic lapses / risks:
- Very large prompt output: if the EvidenceBundle is large, models can truncate or skip sections; may need a “phased” multi-call strategy.

### 4.4 `build_patch_prompt(...)`

Intended goal:
- Generate a minimal patch as a unified diff plus verify/rollback steps.

Risks:
- If the input file context is incomplete, the model can produce uncompilable diffs; ideally provide more structured context (imports, surrounding code, constraints).

### 4.5 `build_test_prompt(...)`

Intended goal:
- Generate test code only for a function and chosen framework.

Risks:
- Without surrounding module context, generated tests may not compile; consider adding “imports/fixtures conventions” guidance and the existing test harness layout.

---

## 5) Cross-Cutting Findings (Prompt System)

1) **Single-shot limitation**
- Thinktank currently cannot provide: tool execution, file reads, git diffs, or multi-turn “previous response”.
- Any prompt that assumes those will fail or encourage hallucination.

2) **Naive template substitution**
- `{{key}}` replacement is direct string replacement with no escaping or validation.
- If a prompt accidentally contains `{{something}}` as a literal example, it can look like an unfilled variable to the model.

3) **Schema typing is string-only**
- Even if a prompt wants numeric/bool/object inputs, loader currently exposes them as strings.

4) **Library relevance**
- The repo currently contains two different “prompt ecosystems”:
  - Workbench templates (`mcp-server/**/*.toml`)
  - Vendored MCP server resources (`mcp-server/prompts/server/**`)
- These should be separated or filtered so Workbench doesn’t load irrelevant files.

---

## 6) Recommended Next Steps

Priority order:
1) Tag/split “agent-with-tools” prompts vs “llm-only” prompts, and filter the Thinktank list by default.
2) Add descriptions to `cli-prompt-library` templates to improve browsing UX.
3) Add optional support for “system” role templates in prompt files (so templates can supply system + user messages instead of only user content).
4) Consider stricter placeholder validation (warn/error if template contains `{{...}}` not provided and not explicitly declared as literal).
