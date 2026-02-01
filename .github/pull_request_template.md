# Pull Request

## What / Why

<!-- What does this PR change? Why is it needed? Link issues. -->

## How to Review

<!-- Suggested review order and any tricky areas. -->

## Screenshots / Recordings (UI)

<!-- If this changes UI, add screenshots or a short recording. -->

## Testing

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cd desktop/workbench && npm test`
- [ ] `cd desktop/workbench && npm run lint`

## Security / Privacy

- [ ] No secrets committed (keys/tokens/PII)
- [ ] New external network calls are documented (provider, endpoint, data sent)
- [ ] Path handling / file IO is safe (no traversal)

## Docs

- [ ] Updated `README.md` if user-facing behavior changed
- [ ] Updated `docs/HOW_TO.md` if setup/usage changed

## Checklist

- [ ] Scope is focused (no drive-by refactors)
- [ ] Error cases handled (good messages, no panics)
- [ ] Tests added/updated where appropriate

