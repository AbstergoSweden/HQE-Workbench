# HQE Workbench — Provider Auto-Discovery Patch (Chat Models Only)

This bundle adds **safe, chat-model-only provider discovery** for:

- Venice.ai (OpenAI-compatible)
- xAI / Grok (OpenAI-compatible)
- OpenRouter (OpenAI-like but with its own `/models` schema)
- Generic OpenAI-compatible endpoints

It is designed to plug into the **`hqe-openai`** crate described in your architecture docs.

## What you get

- Rust implementation of `/models` discovery with:
  - URL + header sanitization
  - Chat-model filtering heuristics
  - Provider auto-detection (by hostname + path)
  - Optional disk cache (TTL) to avoid hammering providers
  - Keychain-backed API key storage interface (macOS)

- Tauri command shim you can wire into the React UI for "Refresh models".

## Integration pointers

1) Add `crates/hqe-openai` (or copy the `provider_discovery.rs` module into your existing crate).
2) Expose the Tauri commands in `src-tauri/src/main.rs` (example included).
3) In the UI: call `discover_models(profile)` and populate your model dropdown.

## Security notes

- No hardcoded keys.
- URLs are normalized to `.../v1` if possible.
- Header keys/values are validated; newline injection is rejected.

## Verify

From the crate root:

```bash
cargo test -p hqe-openai
cargo run -p hqe-cli -- list-models --profile venice
```

(Exact workspace wiring depends on your repo layout.)
