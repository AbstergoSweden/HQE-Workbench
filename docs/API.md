# API / Provider Integration

HQE Workbench supports **OpenAI-compatible chat completion APIs** and is designed to work with:

- Venice.ai (OpenAI-compatible, with Venice extensions)
- OpenAI
- OpenRouter
- xAI (Grok)
- Local OpenAI-schema servers (e.g. on `http://localhost:.../v1` or LAN IPs)

## Supported Capabilities (Text Models)

The workbench focuses on **text models** for:

- Repository scan analysis (structured findings/todos)
- Thinktank prompt execution

The client supports common OpenAI/Venice request parameters (sampling controls, stop sequences,
logprobs, schema response formats, etc.).

## Model Discovery

The Settings screen and Tauri backend call a provider's `/models` endpoint and filter to text models.
For Venice, `/models?type=all` is used and filtered down to `text`/`code`.

## Venice API Specification

This repo vendors a copy of the Venice API OpenAPI spec at:

- `docs/swagger.yaml`

This is used as the review baseline for compatibility and feature coverage.

