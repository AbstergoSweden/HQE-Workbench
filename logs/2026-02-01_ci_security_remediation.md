# CI/Security Remediation Log - 2026-02-01

This document records the root causes and fixes applied for CI and security check failures.

## CI Failures Fixed

### 1. test-rust Job Failure

**Root Cause**: Missing system dependencies for Tauri/WebKit
```
The system library `javascriptcoregtk-4.1` required by crate `javascriptcore-rs-sys` was not found.
```

**Fix Applied**: Updated `.github/workflows/ci.yml` to install required dependencies:
```yaml
- name: Install system dependencies
  run: sudo apt-get update && sudo apt-get install -y libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev pkg-config
```

**File Changed**: `.github/workflows/ci.yml` (line 71)

### 2. test-js Job Failure

**Root Cause**: Test assertions using outdated API signature for `discover_models` command
```
AssertionError: expected "vi.fn()" to be called with arguments: [ 'discover_models', { ... } ]
```

The test expected old API format:
```javascript
invoke('discover_models', { baseUrl: '...', apiKey: '...' })
```

But the actual code uses:
```javascript
invoke('discover_models', { input: { base_url: '...', api_key: '...', headers: {...}, timeout_s: 60, no_cache: false } })
```

**Fix Applied**: Updated test to use correct API signature with `input` wrapper and snake_case fields.

**File Changed**: `desktop/workbench/src/__tests__/settings.test.tsx`

### 3. Security Audit Failure

**Root Cause**: cargo-audit v0.20.0 cannot parse CVSS 4.0 format advisories
```
error: error loading advisory database: parse error: unsupported CVSS version: 4.0
```

The RustSec advisory database now includes CVSS 4.0 scored vulnerabilities which cargo-audit doesn't support.

**Fix Applied**: Replaced manual `cargo-audit` installation with `rustsec/audit-check` GitHub Action which properly handles CVSS 4.0.

**File Changed**: `.github/workflows/security.yml`

## Security Alerts Status

| Alert Type | Count | Status |
|------------|-------|--------|
| Dependabot (npm) | 4+ | Addressed via Dependabot PRs |
| CodeQL | TBD | Will verify after CI passes |
| Secret Scanning | 0 | Clean |

## License Inconsistency Fixed

**Issue**: Mixed MIT/Apache-2.0 references across repository
- LICENSE file: Apache-2.0 ✓
- Cargo.toml: MIT ✗
- README.md: MIT badge ✗
- LEGAL.md: MIT text ✗
- CITATION.cff: Apache-2.0 ✓
- AGENTS.md: MIT ✗
- REPOSITORY_HEALTH_REPORT.md: MIT ✗

**Fix Applied**: Standardized all to Apache-2.0
- Updated `Cargo.toml` license field
- Updated `README.md` badge and license text
- Updated `LEGAL.md` license section
- Updated `AGENTS.md` license reference
- Updated `REPOSITORY_HEALTH_REPORT.md` license references

## Verification

- [x] JavaScript tests pass locally (`npm test` - 7 tests passed)
- [x] Rust clippy passes on core crates (hqe-core, hqe-openai, hqe-git, hqe-artifacts)
- [x] Code review passed (no comments)
- [x] CodeQL security scan passed (0 alerts)
- [ ] Full CI workflow verification pending (requires workflow approval)

---
*Generated as part of repository hygiene audit on 2026-02-01*
