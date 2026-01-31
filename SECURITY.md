# Security Policy

## Supported Versions

We support security fixes for:

- `main` branch (current development)
- The most recent tagged release (when releases are published)

## Reporting a Vulnerability

Please report security issues **privately**.

- Email: <2-craze-headmen@icloud.com>
- Subject: `HQE Workbench Security Report`

Include:

- A clear description of the issue and impact
- Reproduction steps (proof-of-concept if possible)
- Affected version/commit and environment
- Any relevant logs (please redact secrets/tokens)

## What *Not* to Do

- Do not open public GitHub issues for vulnerabilities.
- Do not include API keys, access tokens, or private repository contents in reports.

## Disclosure Process

1. We will acknowledge receipt and begin triage.
2. We will confirm severity and scope.
3. We will work on a fix and (when applicable) a mitigation/workaround.
4. We will coordinate a disclosure timeline with the reporter when possible.

## Provider / Data Handling Notes

HQE Workbench supports **local-only** scans and **LLM-enabled** scans via OpenAI-compatible APIs
(including Venice.ai and local OpenAI-schema servers). When LLM mode is enabled, code snippets may
be sent to the configured provider. Use local-only mode for high privacy contexts.

