#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

# Only enforce the "single provider HTTP call site" invariant within src-tauri.
SRC_TAURI="desktop/workbench/src-tauri/src"

# Patterns that strongly indicate provider HTTP/client execution happening somewhere it shouldn't.
PATTERNS=(
 "OpenAIClient::new"
 "\\.chat\\("
)

fail=0

echo "[verify_invariants] Checking forbidden provider call patterns outside llm.rs..."

# Pathspecs for git grep to exclude specific files/dirs
# Note: we are searching inside SRC_TAURI, so these exclusions filter that scope.
# We explicitly exclude llm.rs and anything in llm/ directory, plus tests.
EXCLUDE_SPECS=(
  ":(exclude)**/llm.rs"
  ":(exclude)**/llm/**"
  ":(exclude)**/*_test.rs"
  ":(exclude)**/tests/**"
  ":(exclude)**/mocks/**"
)

for pat in "${PATTERNS[@]}"; do
  # git grep returns 0 if matches found, 1 if none.
  # We run it against SRC_TAURI with exclusion specs.
  if output=$(git grep -n "$pat" -- "$SRC_TAURI" "${EXCLUDE_SPECS[@]}" 2>/dev/null); then
    echo
    echo "❌ Invariant violation: found pattern '$pat' outside allowed llm module:"
    echo "$output"
    fail=1
  fi
done

echo
echo "[verify_invariants] Checking invariant markers exist..."

MARKERS=(
 "INVARIANT: Only model request builder in codebase"
 "INVARIANT: Only provider HTTP call site"
)

for m in "${MARKERS[@]}"; do
  # Search all .rs files in the repo for the marker.
  # || true ensures we don't crash if 0 hits (git grep returns 1).
  hits=$(git grep -n "$m" -- "*.rs" || true)

  # Count lines (occurrences). Handle empty output (wc -l gives 0).
  count=$(echo -n "$hits" | grep -c '^' || true)

  if [[ "$count" -ne 1 ]]; then
    echo "❌ Invariant marker '$m' expected exactly once, found: $count"
    if [[ -n "$hits" ]]; then
        echo "$hits"
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
