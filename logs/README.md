# Logs Directory

This directory contains audit reports, scan outputs, CI failure analyses, remediation notes, and other diagnostic artifacts.

## What Belongs Here

- **Audit Reports**: Security scan results, dependency audits, compliance checks
- **CI Failure Analyses**: Root cause analyses for build/test failures
- **Remediation Notes**: Documentation of fixes applied for issues
- **Scan Outputs**: Results from code quality tools, linters, static analysis
- **Incident Reports**: Post-mortem documentation for production issues

## Naming Conventions

Files should follow this pattern: `YYYY-MM-DD_<scope>_<description>.<ext>`

Examples:
- `2026-02-01_security_cargo-audit-results.md`
- `2026-01-15_ci_test-failure-analysis.md`
- `2026-01-10_deps_vulnerability-remediation.md`

## What Should NOT Be Committed

- **Secrets**: API keys, tokens, passwords, private keys
- **Large Binaries**: Build artifacts, compiled binaries, disk images
- **Sensitive PII**: Personal data, user information
- **Credentials**: Environment files with secrets (`.env` with real values)

## Retention

- Keep logs for at least 90 days for audit purposes
- Archive older logs to external storage if needed
- Delete logs containing any accidentally committed sensitive data immediately
