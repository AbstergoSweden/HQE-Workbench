# About HQE Workbench

## Mission

The **High Quality Engineering (HQE) Workbench** aims to democratize access to elite engineering leadership tools. By combining local static analysis with the reasoning capabilities of Large Language Models (LLMs), we empower developers to maintain cleaner, safer, and more architecturally sound codebases—without sacrificing privacy.

## Vision

To build the "Jarvis" of software engineering: a proactive, intelligent, and secure assistant that lives on your machine, understands your entire codebase, and helps you make high-leverage technical decisions.

## Core Values

### 🔒 Privacy First
- **Local-Only Mode**: Run complete scans without any external API calls
- **Encrypted Storage**: Chat history and audit logs stored with SQLCipher AES-256 encryption
- **Secure Key Management**: API keys stored in macOS Keychain, never in plaintext
- **No Telemetry**: We don't collect usage data or crash reports without explicit consent

### 🛡️ Security Hardened
- **Defense in Depth**: Multiple layers of protection against XSS, SQL injection, and prompt injection
- **Input Validation**: Strict validation of all user inputs and LLM outputs
- **Jailbreak Detection**: Advanced pattern matching with Unicode normalization
- **Regular Audits**: Comprehensive security reviews (see `docs/COMPREHENSIVE_TODO_AND_BUGS.md`)

### 🚀 Performance
- **Rust Core**: Fast, memory-safe scan pipeline
- **Connection Pooling**: Efficient database resource management
- **Semantic Caching**: Intelligent LLM response caching reduces costs and latency
- **Pagination**: Efficient handling of large chat histories

### 🧠 Intelligence
- **Expert Prompts**: 30+ curated prompts for security, refactoring, documentation
- **Context Awareness**: Repository-aware analysis with Git integration
- **Multi-Turn Chat**: Seamless conversation flow from analysis to implementation
- **Provider Agnostic**: Works with any OpenAI-compatible API

## Key Features

### Repository Health Auditing
- Automated codebase scanning with local heuristics
- Secret detection and redaction
- Dependency analysis
- Security vulnerability detection
- Code quality assessment

### Thinktank Prompt Library
- **Security**: Audit for vulnerabilities, CVE checks, secure coding review
- **Quality**: Code smells, best practices, performance analysis
- **Refactoring**: Modernization suggestions, tech debt identification
- **Documentation**: API docs, README generation, changelog creation
- **Testing**: Unit test generation, coverage analysis, edge case detection

### Encrypted Chat System
- Persistent, encrypted chat sessions
- Multi-turn conversations with context preservation
- Report → Chat transition for follow-up questions
- Secure key management via macOS Keychain

### Desktop Application
- Modern React + TypeScript frontend
- Tauri-based native macOS app
- Real-time scanning progress
- Interactive result exploration

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Backend | Rust | Core engine, encryption, performance |
| Frontend | React + TypeScript | UI components, state management |
| Desktop | Tauri v2 | Native app shell, system integration |
| Database | SQLite + SQLCipher | Encrypted persistence |
| Styling | Tailwind CSS | Modern, responsive UI |
| State | Zustand | Lightweight state management |
| Security | DOMPurify, Keyring | XSS prevention, secure storage |

## Team

**Maintainer**: Faye Hakansdotter (GitHub: <https://github.com/AbstergoSweden>)

We are an open-source initiative driven by the belief that high-quality code is a right, not a privilege.

## Roadmap

### Completed (v0.2.0)
- ✅ Encrypted chat database with SQLCipher
- ✅ Unified Conversation Panel (Report → Chat)
- ✅ Thinktank prompt library with 30+ prompts
- ✅ Enhanced jailbreak detection (50+ patterns)
- ✅ XSS protection with DOMPurify
- ✅ SQL injection prevention
- ✅ Database connection pooling
- ✅ Message pagination

### In Progress
- 🔄 Streaming LLM responses
- 🔄 Tool calling in chat
- 🔄 File attachments

### Planned
- ⏳ Multi-provider switching in UI
- ⏳ Vector embeddings for semantic search
- ⏳ GitHub Actions integration
- ⏳ Team collaboration features

## Contact

- **Issues**: [GitHub Issues](https://github.com/AbstergoSweden/HQE-Workbench/issues)
- **Security**: <2-craze-headmen@icloud.com>
- **Profile**: <https://github.com/AbstergoSweden>
- **Secondary Profile**: <https://github.com/Fayeblade1488>
- **Gravatar**: <https://gravatar.com/swingstoccata3h>

## Acknowledgments

Special thanks to:
- The Rust community for excellent tooling and libraries
- Venice.ai for OpenAI-compatible API support
- All contributors who have helped improve the project

---

*Built with ❤️ in Rust + TypeScript*
