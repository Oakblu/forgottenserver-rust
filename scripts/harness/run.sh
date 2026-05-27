#!/usr/bin/env bash
#
# run.sh — Top-level harness entry point.
#
# Runs each enabled lane driver in turn, aggregates per-lane JSON
# reports into a single run-<timestamp>.json record, and runs the
# ledger writer (when it exists) as the final step.
#
# Lane selection is via the HARNESS_LANES env var (comma-separated).
# When unset, all available lanes are enabled. Each lane is matched
# against a script at scripts/harness/lanes/<name>.sh; missing
# scripts emit a SKIPPED record so the run report stays honest.
#
# Exits 0 if no lane reported FAIL; 1 otherwise.
#
# Usage:
#   scripts/harness/run.sh
#   HARNESS_LANES=wire_replay,otbm_diff scripts/harness/run.sh

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

# ─── Lane discovery ──────────────────────────────────────────────────────────
DEFAULT_LANES=(wire_replay lua_bindings otbm_diff persisted_state)
if [ -n "${HARNESS_LANES:-}" ]; then
  IFS=',' read -r -a LANES <<< "$HARNESS_LANES"
else
  LANES=("${DEFAULT_LANES[@]}")
fi

# ─── Run report bootstrap ────────────────────────────────────────────────────
harness::ensure_reports_dir
TIMESTAMP="$(date -u +'%Y%m%dT%H%M%SZ')"
export HARNESS_RUN_REPORT="$HARNESS_REPORTS_DIR/run-$TIMESTAMP.json"

# Start the JSON report as a single-line metadata header. Each lane
# appends one or more JSON records (one per scenario / sub-lane).
# A jq-style consumer can read this as a JSON-lines stream.
echo "{\"timestamp\":\"$TIMESTAMP\",\"lanes_requested\":$(printf '%s\n' "${LANES[@]}" | jq -R . | jq -s -c .)}" \
  > "$HARNESS_RUN_REPORT" 2>/dev/null || \
  echo "{\"timestamp\":\"$TIMESTAMP\",\"lanes_requested\":[\"${LANES[*]}\"]}" > "$HARNESS_RUN_REPORT"

# ─── Run lanes ───────────────────────────────────────────────────────────────
FAIL_COUNT=0
for lane in "${LANES[@]}"; do
  driver="$SCRIPT_DIR/lanes/${lane}.sh"
  if [ ! -x "$driver" ] && [ ! -f "$driver" ]; then
    harness::warn "Lane '$lane' has no driver at $driver — recording SKIPPED"
    harness::report "{\"lane\":\"$lane\",\"status\":\"SKIPPED\",\"reason\":\"driver not implemented\"}"
    continue
  fi

  harness::info "Running lane: $lane"
  if bash "$driver"; then
    harness::ok "Lane '$lane' completed"
  else
    harness::fail "Lane '$lane' failed (driver exit non-zero)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
done

# ─── Ledger writer (Phase 6) ─────────────────────────────────────────────────
LEDGER_WRITER="$REPO_ROOT/apps/poketibia/forgottenserver-rust/target/release/harness-ledger-writer"
if [ -x "$LEDGER_WRITER" ]; then
  harness::info "Running ledger writer..."
  "$LEDGER_WRITER" \
    --ledger "$REPO_ROOT/apps/poketibia/forgottenserver-rust/MIGRATION_LEDGER.yml" \
    --reports "$HARNESS_RUN_REPORT" \
    --out "$HARNESS_REPORTS_DIR/ledger_proposal-$TIMESTAMP.diff" || true
fi

# ─── Summary ─────────────────────────────────────────────────────────────────
harness::info "Run report: $HARNESS_RUN_REPORT"
if [ "$FAIL_COUNT" -eq 0 ]; then
  harness::ok "Harness run complete — no lane failures"
  exit 0
fi
harness::fail "Harness run complete — $FAIL_COUNT lane(s) failed"
exit 1
