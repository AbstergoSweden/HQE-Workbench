#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Only enforce the "single provider HTTP call site" invariant within src-tauri.
SRC_TAURI="desktop/workbench/src-tauri/src"

# Allowed locations for provider/network execution code.
# If you later split llm.rs into a module directory, keep llm/** allowed.
ALLOW_GLOBS=(
 "!llm.rs"
 "!llm/**"
 "!**/*_test.rs"
 "!**/tests/**"
 "!**/mocks/**"
)

# Patterns that strongly indicate provider HTTP/client execution happening somewhere it shouldn't.
PATTERNS=(
 "OpenAIClient::new"
 "\\.chat\\("
)

fail=0

echo "[verify_invariants] Checking forbidden provider call patterns outside llm.rs..."

for pat in "${PATTERNS[@]}"; do
  if rg -n --hidden --no-ignore-vcs "$pat" "$SRC_TAURI" --glob "${ALLOW_GLOBS[@]}" >/tmp/invariant_hits.txt 2>/dev/null; then
    echo
    echo "❌ Invariant violation: found pattern '$pat' outside allowed llm module:"
    cat /tmp/invariant_hits.txt
    fail=1
  fi
done

rm -f /tmp/invariant_hits.txt || true

echo
echo "[verify_invariants] Checking invariant markers exist..."

MARKERS=(
 "INVARIANT: Only model request builder in codebase"
 "INVARIANT: Only provider HTTP call site"
)

for m in "${MARKERS[@]}"; do
  count="$(rg -n --hidden --no-ignore-vcs "$m" "$ROOT" --glob '*.rs' | wc -l | tr -d ' ')"
  if [[ "$count" -ne 1 ]]; then
    echo "❌ Invariant marker '$m' expected exactly once, found: $count"
    rg -n --hidden --no-ignore-vcs "$m" "$ROOT" --glob '*.rs' || true
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo
  echo "[verify_invariants] FAILED"
  exit 1
fi

echo "[verify_invariants] OK"
