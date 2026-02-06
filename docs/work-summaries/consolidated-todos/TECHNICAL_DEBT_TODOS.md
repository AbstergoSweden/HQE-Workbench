# Technical Debt TODOs - Consolidated

**Collection Date:** 2026-02-06  
**Source:** Plans, Styleguides, and Architecture Documents

---

## 📚 DOCUMENTED TECHNICAL DEBT

### 1. Any Type Elimination (TypeScript)
**Location:** `mcp-server/prompts/plans/techincal_debt/any-type-elimination.md`  
**Status:** Planned  
**Scope:** Full TypeScript codebase  
**Description:** Systematic elimination of `any` types across the TypeScript codebase to improve type safety.  
**Approach:**
- Phase 1: Identify all `any` usages via ESLint
- Phase 2: Replace with proper interfaces/types
- Phase 3: Add strict null checks
- Phase 4: Enable `no-explicit-any` rule

---

## 🎨 CODE STYLE TODOs

### 2. TODO Comment Format Standardization
**Location:** `mcp-server/conductor/code_styleguides/python.md:23`  
**Line:** 23  
**Category:** Code Style  
**Standard:**
```python
# TODO(username): Description of what needs to be done.
```
**Status:** Guideline documented, needs enforcement  
**Action:** Add linting rule to enforce format

---

## ✅ COMPLETED (This Session)

### 3. CI/CD Action Version Pinning
**Location:** `.github/workflows/*.yml`  
**Status:** ✅ DONE  
**Completed:** 2026-02-06  
**Details:** All GitHub Actions pinned to SHA commits for supply chain security.

### 4. SQLCipher Feature Documentation
**Location:** `crates/hqe-core/Cargo.toml`  
**Status:** ✅ DONE  
**Details:** SQLCipher feature gate now properly documented.

---

## 📊 SUMMARY

| Category | Count | Priority |
|----------|-------|----------|
| Type Safety | 1 | High |
| Code Style | 1 | Low |
| Completed | 2 | ✅ |

**Action Items:**
1. Create tracking issue for Any Type Elimination
2. Add ESLint rule for TODO format
3. Schedule type elimination sprint
