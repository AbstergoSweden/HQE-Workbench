# Privacy

HQE Workbench is designed to be **local-first**.

This document describes data handling at a high level. It is not legal advice.

## What HQE Workbench Stores Locally

- Scan artifacts (reports, manifests, logs) written to local disk.
- **Local Database**: SQLite database (`hqe.db`) storing:
  - **Session Logs**: Timestamped record of prompts (redacted) and responses.
  - **Semantic Cache**: Hashed request keys and cached responses to minimize external data usage.
- Provider profiles (base URL, model id, optional headers) stored on disk.
- API keys stored in the macOS Keychain (not written to plaintext config files).

## What HQE Workbench Sends to a Provider (LLM Mode)

When you run an LLM-enabled scan (or execute a Thinktank prompt) the app may send:

- Redacted snippets of repository files and metadata needed for analysis.
- Your prompt/tool inputs.
- Provider configuration information necessary to make the request (model id, options).

The project includes a secret redaction layer intended to remove obvious credentials and tokens
before any content is transmitted, but no redaction system is perfect.

## Local-Only Mode

If you run in local-only mode, HQE Workbench performs no external API calls. No data is logged to the semantic cache or session log that involves external providers (internal heuristics are not logged to the LLM DB).

## Telemetry

HQE Workbench does not intentionally ship analytics/telemetry. If you believe any telemetry is
present, please file an issue so it can be audited and removed.

## Your Responsibilities

You are responsible for complying with your organization’s policies and applicable laws when
scanning repositories, especially private or regulated codebases.

## Contact

For privacy questions or concerns, contact: <2-craze-headmen@icloud.com>
