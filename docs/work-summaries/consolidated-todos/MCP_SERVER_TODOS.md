# MCP Server TODOs - Consolidated

**Collection Date:** 2026-02-06  
**Source:** mcp-server/prompts/server (TypeScript/Node.js)

---

## 🔴 HIGH PRIORITY

### 1. LLM API Integration for Gate Validation
**Location:** `mcp-server/prompts/server/src/gates/core/gate-validator.ts:272`  
**Line:** 272  
**Category:** Core Feature / LLM Integration  
**Original Comment:**
```typescript
/**
 * TODO: IMPLEMENT LLM API INTEGRATION
 * This is currently a stub. Implementation needed:
 * 1. Call LLM with gate_prompt and evidence bundle
 * 2. Parse structured response
 * 3. Return GateValidationResult
 */
```
**Context:** The gate validator has a stub method for LLM-based validation that needs actual implementation.  
**Impact:** Without this, semantic gate validation doesn't work - only rule-based validation is active.  
**Suggested Action:** 
- Define LLM prompt template for gate validation
- Implement API call to configured provider
- Add response parsing logic
- Add retry/fallback logic

---

### 2. LLM Client Connection for Semantic Validation
**Location:** `mcp-server/prompts/server/src/gates/services/semantic-gate-service.ts:275`  
**Line:** 275  
**Category:** Core Feature / LLM Integration  
**Original Comment:**
```typescript
// TODO: Connect actual LLM client for true semantic validation
// Currently returning mock/simulated results
```
**Context:** The semantic gate service simulates LLM validation instead of calling actual LLM.  
**Impact:** Semantic gates return placeholder results instead of actual AI analysis.  
**Dependencies:** Requires LLM client implementation (related to #1 above).

---

## 🟡 MEDIUM PRIORITY

### 3. Analytics Persistence and Exposure
**Location:** `mcp-server/prompts/server/src/tooling/action-metadata/usage-tracker.ts:5`  
**Line:** 5  
**Category:** Analytics / Telemetry  
**Original Comment:**
```typescript
/**
 * #TODO telemetry: Persist snapshots and expose via system_control analytics 
 * once that surface lands.
 */
```
**Context:** Usage tracking data is collected but not persisted or exposed through system_control tool.  
**Status Update:** ⚠️ Partially addressed by our Rust analytics implementation in `crates/hqe-core/src/analytics/mod.rs`.  
**Suggested Action:** 
- Integrate with new Rust analytics module
- Add persistence layer for telemetry data
- Expose via system_control analytics action

---

### 4. Framework-Specific Judge Template Override
**Location:** `mcp-server/prompts/server/src/execution/pipeline/stages/06a-judge-selection-stage.ts:123`  
**Line:** 123  
**Category:** Framework Integration  
**Original Comment:**
```typescript
// #todo: If a framework is active, override the base judge template 
// with the methodology-specific judgePrompt from the guide/definition.
```
**Context:** When a methodology framework is active, the judge selection should use framework-specific prompts.  
**Impact:** Limited framework customization for evaluation stages.

---

### 5. Processing Steps Integration
**Location:** `mcp-server/prompts/server/src/frameworks/utils/step-generator.ts:127`  
**Line:** 127  
**Category:** Framework / Prompt Generation  
**Original Comment:**
```typescript
// #todo: Wire processingSteps into prompt_guidance as authoring 
// checklists/template hints; also feed into validation/analytics for missing coverage.
```
**Context:** Processing steps are generated but not fully integrated into guidance or validation.  
**Impact:** Framework steps aren't properly validated against execution.

---

### 6. Methodology Steps Toolcall Exposure
**Location:** `mcp-server/prompts/server/src/frameworks/utils/step-generator.ts:160`  
**Line:** 160  
**Category:** MCP Tools / API  
**Original Comment:**
```typescript
// #todo: Expose executionSteps via a "methodology_steps" toolcall 
// (akin to %judge) so the client LLM can request structured steps 
// for the user query; currently guidance-only.
```
**Context:** Execution steps are only available as guidance text, not as a callable tool.  
**Impact:** Client LLMs can't programmatically request methodology steps.

---

## 🟢 LOW PRIORITY

### 7. Jest ESM Mode Issues
**Location:** `mcp-server/prompts/server/tests/e2e/mcp-server-smoke.test.ts:171,206`  
**Lines:** 171, 206  
**Category:** Testing / Infrastructure  
**Original Comment:**
```typescript
// TODO: Jest ESM mode has issues with spawned process stdio capture
```
**Context:** E2E tests have commented-out assertions due to Jest ESM compatibility issues.  
**Impact:** Reduced test coverage for spawned process scenarios.  
**Suggested Action:** Upgrade Jest or migrate to Vitest (already used in main workbench).

---

## 📋 SUMMARY

| Priority | Count | Status |
|----------|-------|--------|
| 🔴 High | 2 | Blocked on LLM client implementation |
| 🟡 Medium | 4 | Can be worked on independently |
| 🟢 Low | 1 | Testing infrastructure |

**Total MCP Server TODOs:** 7

### Dependencies Map
```
TODO #1 (LLM API) ──┬──► TODO #2 (Semantic Validation)
                    └──► TODO #4 (Judge Template)
                    
TODO #3 (Analytics) ──► Integrates with Rust analytics (✅ DONE)

TODO #5, #6 (Steps) ──► Related to framework system
```

### Recommended Work Order
1. **Phase 1:** Implement LLM client foundation (TODO #1)
2. **Phase 2:** Connect semantic validation (TODO #2)
3. **Phase 3:** Analytics integration with Rust (TODO #3)
4. **Phase 4:** Framework enhancements (TODO #4, #5, #6)
5. **Phase 5:** Testing improvements (TODO #7)
