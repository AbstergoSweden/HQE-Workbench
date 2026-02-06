#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Only enforce the "single provider HTTP call site" invariant within src-tauri.
SRC_TAURI="desktop/workbench/src-tauri/src"

# Use ripgrep if available, otherwise fallback to grep
if command -v rg >/dev/null 2>&1; then
  GREP_CMD="rg"
  HAS_RG=1
else
  GREP_CMD="grep"
  HAS_RG=0
  echo "[verify_invariants] Note: ripgrep not found, using grep fallback"
fi

# Patterns that strongly indicate provider HTTP/client execution happening somewhere it shouldn't.
PATTERNS=(
 "OpenAIClient::new"
 "\.chat\("
)

fail=0

echo "[verify_invariants] Checking forbidden provider call patterns outside llm.rs..."

for pat in "${PATTERNS[@]}"; do
  hits_file="/tmp/invariant_hits_$$.txt"
  if [[ "$HAS_RG" -eq 1 ]]; then
    if rg -n --hidden --no-ignore-vcs "$pat" "$SRC_TAURI" --glob "!llm.rs" --glob "!llm/**" --glob "!**/*_test.rs" --glob "!**/tests/**" --glob "!**/mocks/**" >"$hits_file" 2>/dev/null; then
      echo
      echo "❌ Invariant violation: found pattern '$pat' outside allowed llm module:"
      cat "$hits_file"
      fail=1
    fi
  else
    # Fallback to grep with manual filtering
    if grep -r -n "$pat" "$SRC_TAURI" --include="*.rs" 2>/dev/null | \
       grep -v "llm\.rs:" | \
       grep -v "/llm/" | \
       grep -v "_test\.rs:" | \
       grep -v "/tests/" | \
       grep -v "/mocks/" >"$hits_file" || true; then
      if [[ -s "$hits_file" ]]; then
        echo
        echo "❌ Invariant violation: found pattern '$pat' outside allowed llm module:"
        cat "$hits_file"
        fail=1
      fi
    fi
  fi
  rm -f "$hits_file"
done

echo
echo "[verify_invariants] Checking invariant markers exist..."

MARKERS=(
 "INVARIANT: Only model request builder in codebase"
 "INVARIANT: Only provider HTTP call site"
)

for m in "${MARKERS[@]}"; do
  if [[ "$HAS_RG" -eq 1 ]]; then
    count="$(rg -n --hidden --no-ignore-vcs "$m" "$ROOT" --glob '*.rs' | wc -l | tr -d ' ')"
  else
    count="$(grep -r -n "$m" "$ROOT" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')"
  fi
  if [[ "$count" -ne 1 ]]; then
    echo "❌ Invariant marker '$m' expected exactly once, found: $count"
    if [[ "$HAS_RG" -eq 1 ]]; then
      rg -n --hidden --no-ignore-vcs "$m" "$ROOT" --glob '*.rs' || true
    else
      grep -r -n "$m" "$ROOT" --include="*.rs" || true
    fi
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo
  echo "[verify_invariants] FAILED"
  exit 1
fi

echo "[verify_invariants] OK"
