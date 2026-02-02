<!-- markdownlint-disable MD024 -->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security

- **CRITICAL**: Fixed XSS vulnerability in chat display (C7) - Added DOMPurify sanitization
- **CRITICAL**: Fixed SQL injection vulnerabilities in encrypted database (C1, C2)
- **CRITICAL**: Fixed prompt injection in template substitution (C4)
- **CRITICAL**: Fixed race conditions in chat state management (C3, C8)
- Enhanced jailbreak detection with 50+ patterns and Unicode normalization (M10)

### Added

- **💬 Encrypted Chat System**: SQLCipher AES-256 encrypted chat persistence
- **🔄 Conversation Panel**: Unified UI for reports and chat with seamless transition
- **📄 Message Pagination**: Configurable pagination (100-1000 messages per page)
- **🤖 Thinktank Integration**: 30+ expert prompts with explanations and categories
- **🔐 Key Lock UI**: Toggle between secure storage (keychain) and session-only
- **🏷️ Category Filtering**: Browse prompts by Security, Quality, Refactor, Test, etc.
- **🔍 Model Discovery**: Auto-populate available models from providers
- **📊 Provider Specs**: 6 prefilled provider configurations (OpenAI, Anthropic, Venice, OpenRouter, xAI, Kimi)
- **⚡ Database Connection Pooling**: Shared connection for improved performance
- **📝 Transaction Support**: Atomic message and metadata updates

### Changed

- **License:** Changed from MIT to Apache 2.0 for better patent protection and enterprise compatibility
- **Security:** Hardened CI/CD workflows with SHA-pinned Actions and least-privilege permissions
- **Dependencies:** Added DOMPurify, unicode-normalization for security hardening

### Documentation

- Added comprehensive security audit (`docs/COMPREHENSIVE_TODO_AND_BUGS.md`)
- Updated AGENTS.md with new features and commands
- Updated README with Chat System and Thinktank documentation
- Updated ABOUT.md with Core Values and Roadmap

## [0.2.0] - 2026-02-02

### Added

- **CLI**: New `hqe export` command to extract artifacts from previous runs
- **CLI**: New `hqe patch` command to apply diffs from scan reports
- **Provider Discovery**: Added support for local/private LLM endpoints (no API key required)
- **Settings**: Enhanced configuration for custom headers, organization, and project IDs
- **UI**: Added "Export" functionality to Report screen
- **UI**: Visual improvements to scan progress and evidence rendering
- **💬 Encrypted Chat System**: Full-featured chat with SQLCipher encryption
- **🤖 Thinktank Prompt Library**: 30+ expert prompts with metadata

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

[Unreleased]: https://github.com/AbstergoSweden/HQE-Workbench/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/AbstergoSweden/HQE-Workbench/releases/tag/v0.2.0
[0.1.0]: https://github.com/AbstergoSweden/HQE-Workbench/releases/tag/v0.1.0
