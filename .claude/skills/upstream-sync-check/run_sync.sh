#!/usr/bin/env bash
# run_sync.sh — orchestrates phases 1–4 of the upstream-sync-check skill.
# Run from the project root.
#
# Usage:
#   bash .claude/skills/upstream-sync-check/run_sync.sh
#   bash .claude/skills/upstream-sync-check/run_sync.sh "6 months ago"

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "$0")" && pwd)"
PY="python3 $SKILL_DIR/sync_check.py"
SINCE="${1:-3 months ago}"

# ── Phase 1: ensure upstream clone is fresh ────────────────────────────────
echo "=== Phase 1: Ensuring upstream clone is fresh ==="
$PY ensure-upstream

# ── Phase 2: get commits ───────────────────────────────────────────────────
echo ""
echo "=== Phase 2: Getting commits since '$SINCE' ==="
commits_out=$($PY get-commits --since "$SINCE")
echo "$commits_out"

if grep -q "^NO_COMMITS" <<< "$commits_out"; then
    echo ""
    echo "RESULT: No upstream changes in the past '$SINCE' — nothing to migrate."
    exit 0
fi

# Parse CHANGED_FILES JSON into a bash array (bash 3.2-safe, no mapfile)
changed_json=$(grep "^CHANGED_FILES:" <<< "$commits_out" | cut -d' ' -f2-)
changed_files=()
while IFS= read -r line; do
    changed_files+=("$line")
done < <(python3 -c "import json,sys; [print(f) for f in json.loads(sys.argv[1])]" "$changed_json")

if [ ${#changed_files[@]} -eq 0 ]; then
    echo ""
    echo "RESULT: No .cpp/.h files changed — nothing to map."
    exit 0
fi

# ── Phase 3: map files to symbols ─────────────────────────────────────────
echo ""
echo "=== Phase 3: Mapping changed files to C++ symbols ==="
symbols_out=$($PY map-symbols --files "${changed_files[@]}")
echo "$symbols_out"

symbols_json=$(grep "^SYMBOLS:" <<< "$symbols_out" | cut -d' ' -f2-)
symbols=()
while IFS= read -r line; do
    symbols+=("$line")
done < <(python3 -c "import json,sys; [print(s) for s in json.loads(sys.argv[1])]" "$symbols_json")

if [ ${#symbols[@]} -eq 0 ]; then
    echo ""
    echo "RESULT: No symbols found in manifest. Check NEW_FILE entries above — manual review required."
    exit 0
fi

# ── Phase 4a: check ledger (file-level — correct; symbol-level has path mismatch) ──
echo ""
echo "=== Phase 4a: Checking MIGRATION_LEDGER.yml (file-level status) ==="
$PY check-file-ledger --files "${changed_files[@]}"

# ── Phase 4b: check intentional differences ────────────────────────────────
echo ""
echo "=== Phase 4b: Checking intentional_differences.yml ==="
$PY check-intentional --symbols "${symbols[@]}"

echo ""
echo "=== Pipeline complete (phases 1–4). Review output above, then follow Phase 5 in SKILL.md. ==="
