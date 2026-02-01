<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **License:** Changed from MIT to Apache 2.0 for better patent protection and enterprise compatibility
- **Security:** Hardened CI/CD workflows with SHA-pinned Actions and least-privilege permissions

### Added

- Repository scaffold with community docs (Contributing, Code of Conduct, Security)
- Governance model documentation (GOVERNANCE.md)
- Project authors file (AUTHORS)
- Apache 2.0 NOTICE file with third-party attribution
- Security advisory issue template for private vulnerability reports
- Additional CI workflows: release automation, stale issue cleanup, documentation deployment
- EditorConfig for consistent code formatting across editors
- Dependabot configuration for automated dependency updates
- Pre-commit hooks for code quality and secret detection
- CODEOWNERS updated with security-critical path ownership
- CI workflows for build/test/lint and a security audit workflow
- Documentation structure (architecture, threat model, provider setup)

## [0.2.0] - 2026-01-31

### Added

- **CLI**: New `hqe export` command to extract artifacts from previous runs
- **CLI**: New `hqe patch` command to apply diffs from scan reports
- **Provider Discovery**: Added support for local/private LLM endpoints (no API key required)
- **Settings**: Enhanced configuration for custom headers, organization, and project IDs
- **UI**: Added "Export" functionality to Report screen
- **UI**: Visual improvements to scan progress and evidence rendering

### Changed

- **Discovery**: Improved URL normalization and filtering logic for non-chat models
- **Scanning**: Renamed "Security" findings to "Safety" in some contexts to align with protocol
- **Fixes**: Resolved false positives in secret detection and SQL injection logic
- **Infrastructure**: Updated CI to support Rust 1.75+ and fixed glib dependencies

## [0.1.0] - 2026-01-30

### Added

- Initial project structure with Rust/Python/TypeScript support
- Core scanning pipeline (`hqe-core` crate)
- Git operations wrapper (`hqe-git` crate)
- OpenAI-compatible LLM client (`hqe-openai` crate)
- Report generation (`hqe-artifacts` crate)
- Tauri desktop application (`apps/workbench`)
- CLI entry point (`cli/hqe`)
- HQE Protocol v3 definitions
- MIT License
- Code of Conduct (Contributor Covenant 2.1)
- Contributing guidelines
- Security policy
- Issue and PR templates
- GitHub Actions workflows (CI, Security)

[Unreleased]: https://github.com/AbstergoSweden/HQE-Workbench/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AbstergoSweden/HQE-Workbench/releases/tag/v0.1.0
