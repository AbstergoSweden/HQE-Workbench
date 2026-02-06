# Rust Codebase TODOs - Consolidated

**Collection Date:** 2026-02-06  
**Source:** HQE Workbench Rust Crates

---

## 🔴 HIGH PRIORITY

### 1. Repository Path Traversal Refinement
**Location:** `crates/hqe-core/src/repo.rs:794`  
**Line:** 794  
**Category:** Security/Reliability  
**Original Comment:**
```rust
// TODO: traversal path to repos needs to be further refined for later. 
// Past errors with same non-fixed log
```
**Context:** The repository scanning path traversal handling needs improvement. There have been recurring errors related to path traversal that haven't been fully addressed.  
**Suggested Action:** Review and refactor `repo.rs` path handling logic, add more comprehensive path validation tests.

---

## 🟡 MEDIUM PRIORITY

### 2. Analytics Module Test Cleanup
**Location:** `crates/hqe-core/src/analytics/mod.rs:749`  
**Line:** 749 (in test context)  
**Category:** Testing  
**Note:** This is actually a security test assertion, not a TODO:
```rust
assert!(EventValidator::validate_event_name("hack_attempt").is_err());
```
**Status:** ✅ Working as intended - test verifies that suspicious event names are rejected.

---

## ✅ COMPLETED (This Session)

### 3. SQL Injection Prevention
**Location:** `crates/hqe-core/src/encrypted_db.rs`  
**Status:** ✅ FIXED  
**Details:** VACUUM INTO SQL injection vulnerability patched using two-step temp file approach.

### 4. Jailbreak Detection Integration
**Location:** `desktop/workbench/src-tauri/src/chat.rs`  
**Status:** ✅ FIXED  
**Details:** Integrated existing `detect_override_attempt()` into chat message flow.

### 5. Error Information Leakage
**Location:** `desktop/workbench/src-tauri/src/chat.rs`, `llm.rs`  
**Status:** ✅ FIXED  
**Details:** Replaced detailed error messages with generic user-friendly ones, internal logging via tracing.

### 6. Template Key Validation
**Location:** `crates/hqe-core/src/prompt_runner.rs`  
**Status:** ✅ FIXED  
**Details:** Strengthened placeholder name validation with max length, alphabetic start requirement.

### 7. System Prompt Hash Verification
**Location:** `crates/hqe-core/src/system_prompt.rs`  
**Status:** ✅ FIXED  
**Details:** Enabled actual SHA-256 verification with environment bypass for development.

### 8. Database Transaction Support
**Location:** `crates/hqe-core/src/encrypted_db.rs`  
**Status:** ✅ FIXED  
**Details:** Added `with_transaction()` method for atomic multi-operation transactions.

### 9. Connection Pooling Improvement
**Location:** `crates/hqe-core/src/encrypted_db.rs`  
**Status:** ✅ FIXED  
**Details:** Migrated from `std::sync::Mutex` to `parking_lot::Mutex`.

### 10. Pagination Bounds Validation
**Location:** `crates/hqe-core/src/encrypted_db.rs`  
**Status:** ✅ FIXED  
**Details:** Added `MAX_LIMIT`, `with_validated_limit()`, and bounds checking.

### 11. Input Length Limits
**Location:** `desktop/workbench/src-tauri/src/chat.rs`  
**Status:** ✅ FIXED  
**Details:** Added 50KB message length limit to prevent DoS.

---

## 📋 SUMMARY

| Category | Count | Status |
|----------|-------|--------|
| High Priority | 1 | Pending |
| Medium Priority | 1 | False positive (test code) |
| Completed This Session | 11 | ✅ Done |

**Total TODOs in Rust Codebase:** 2 (1 actionable)
