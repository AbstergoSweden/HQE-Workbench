# Consolidated TODOs Index

**Collection Date:** 2026-02-06  
**Session Work Summary:** [2026-02-06 WORK SUMMARY.md](../2026-02-06%20WORK%20SUMMARY.md)

---

## 📁 Organization

This directory contains all TODOs, FIXMEs, and technical debt items collected from the HQE Workbench codebase, organized by category and priority.

### Files in this Directory

| File | Description | TODO Count |
|------|-------------|------------|
| [RUST_CODEBASE_TODOS.md](./RUST_CODEBASE_TODOS.md) | Rust crates (hqe-core, hqe-openai, etc.) | 2 (1 actionable) |
| [MCP_SERVER_TODOS.md](./MCP_SERVER_TODOS.md) | TypeScript MCP server implementation | 7 |
| [TECHNICAL_DEBT_TODOS.md](./TECHNICAL_DEBT_TODOS.md) | Architecture and styleguide items | 2 |
| [README.md](./README.md) | This index file | - |

---

## 📊 Quick Stats

### By Priority
| Priority | Count | Files Affected |
|----------|-------|----------------|
| 🔴 High | 3 | 3 files |
| 🟡 Medium | 6 | 4 files |
| 🟢 Low | 3 | 3 files |

### By Language/Platform
| Platform | Count | Primary Files |
|----------|-------|---------------|
| Rust | 2 | `crates/hqe-core/src/repo.rs`, `crates/hqe-core/src/analytics/mod.rs` |
| TypeScript (MCP) | 7 | `mcp-server/prompts/server/src/**/*.ts` |
| Documentation | 2 | `mcp-server/prompts/plans/**/*.md` |

### By Category
| Category | Count | Status |
|----------|-------|--------|
| Security | 1 | 🔴 Needs attention |
| LLM Integration | 2 | 🔴 Blocked |
| Analytics/Telemetry | 1 | 🟡 Partially addressed |
| Framework | 3 | 🟡 Can proceed |
| Testing | 1 | 🟢 Low priority |
| Type Safety | 1 | 🟢 Planned |

---

## 🎯 Top 3 Action Items

### 1. Repository Path Traversal (RUST - HIGH)
**File:** `crates/hqe-core/src/repo.rs:794`  
**Issue:** Path traversal handling needs refinement  
**Impact:** Security/Reliability

### 2. LLM API Integration (MCP - HIGH)
**File:** `mcp-server/prompts/server/src/gates/core/gate-validator.ts:272`  
**Issue:** Gate validation uses stub instead of actual LLM  
**Impact:** Core feature broken

### 3. LLM Client Connection (MCP - HIGH)
**File:** `mcp-server/prompts/server/src/gates/services/semantic-gate-service.ts:275`  
**Issue:** Semantic validation returns mock results  
**Impact:** Semantic gates non-functional

---

## ✅ Recently Completed (This Session)

### Security Fixes (11 items)
1. ✅ SQL Injection in `encrypted_db.rs`
2. ✅ XSS Vulnerability in `UnifiedOutputPanel.tsx`
3. ✅ Race Condition in Chat Sessions
4. ✅ Error Information Leakage
5. ✅ Template Key Validation
6. ✅ System Prompt Hash Verification
7. ✅ Jailbreak Detection Integration
8. ✅ Database Transaction Support
9. ✅ Connection Management Improvement
10. ✅ Pagination Enhancements
11. ✅ Input Length Limits

### CI/CD Improvements (4 items)
1. ✅ Action version pinning in `ci.yml`
2. ✅ Action version pinning in `security.yml`
3. ✅ Action version pinning in `docs.yml`
4. ✅ `verify_invariants.sh` grep fallback

### Analytics Implementation (2 new modules)
1. ✅ Frontend: `desktop/workbench/src/lib/analytics.ts`
2. ✅ Backend: `crates/hqe-core/src/analytics/mod.rs`

### UI Anti-Spam (20+ elements)
1. ✅ ThinktankScreen button/input protections
2. ✅ ScanScreen configuration protections

---

## 🔄 Maintenance Notes

### How to Update This Collection

1. **Search for new TODOs:**
   ```bash
   find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
     ! -path "./target/*" ! -path "./node_modules/*" \
     -exec grep -H -n -i "todo\|fixme\|hack" {} \;
   ```

2. **Categorize by:**
   - Language (Rust/TypeScript)
   - Priority (High/Medium/Low)
   - Status (Pending/In Progress/Done)

3. **Update relevant file:**
   - `RUST_CODEBASE_TODOS.md` for Rust code
   - `MCP_SERVER_TODOS.md` for MCP server
   - `TECHNICAL_DEBT_TODOS.md` for architectural items

4. **Update this README** with new counts

---

## 📅 Collection History

| Date | Collector | Items Found | Notes |
|------|-----------|-------------|-------|
| 2026-02-06 | Claude (AI Assistant) | 11 | Comprehensive security audit session |

---

## 🔗 Related Documents

- [Work Summary](../2026-02-06%20WORK%20SUMMARY.md) - Full session summary
- [Security Audit](../../../docs/COMPREHENSIVE_TODO_AND_BUGS.md) - Deep security scan results
- [CHANGELOG](../../../CHANGELOG.md) - Project changelog
- [Architecture](../../../docs/architecture.md) - System architecture

---

*Last Updated: 2026-02-06*  
*Next Review: As needed or after major feature work*
