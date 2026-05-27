#!/usr/bin/env bash
#
# lua_bindings.sh — Phase 2 Lua-binding audit lane.
#
# Static sub-lane (always runs): parses C++ `luascript.cpp` for every
# binding registration and the Rust `crates/scripting/` tree for every
# mlua registration; diffs the two surfaces. Failures (missing bindings
# in Rust, unexpected bindings in Rust) produce ledger transitions.
#
# Runtime sub-lane (opt-in via HARNESS_LUA_RUNTIME=1): not yet
# implemented — requires the C++ `--exec-lua-snippet=<file>` hook
# (gated by `#ifdef TIBIA_HARNESS_HOOK`) and the Rust equivalent
# (gated by `cfg(feature = "harness")`). These land in a follow-up
# pass once the binding surface itself exists in Rust; running the
# runtime sub-lane against the current Rust port (~0 real mlua
# registrations) would produce noise, not signal.
#
# Exit: 0 if PASS, 1 if FAIL.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
# shellcheck source=../lib.sh
source "$SCRIPT_DIR/../lib.sh"

# ─── Paths ───────────────────────────────────────────────────────────────────
CPP_LUASCRIPT="$REPO_ROOT/forgottenserver/src/luascript.cpp"
RUST_SCRIPTING="$REPO_ROOT/crates/scripting/src"
AUDIT_BIN="$REPO_ROOT/target/release/lua-static-audit"
REPORT_OUT="$HARNESS_REPORTS_DIR/lua_bindings-static.json"

# ─── Sanity checks ───────────────────────────────────────────────────────────
if [ ! -f "$CPP_LUASCRIPT" ]; then
  harness::fail "C++ luascript.cpp not found at $CPP_LUASCRIPT"
  harness::report "{\"lane\":\"lua_bindings\",\"sub_lane\":\"static\",\"status\":\"ERROR\",\"reason\":\"cpp source missing\"}"
  exit 1
fi

if [ ! -d "$RUST_SCRIPTING" ]; then
  harness::fail "Rust scripting dir not found at $RUST_SCRIPTING"
  harness::report "{\"lane\":\"lua_bindings\",\"sub_lane\":\"static\",\"status\":\"ERROR\",\"reason\":\"rust scripting dir missing\"}"
  exit 1
fi

if [ ! -x "$AUDIT_BIN" ]; then
  harness::warn "lua-static-audit binary missing; building release..."
  (cd "$REPO_ROOT" && \
   cargo build --release -p harness-tools --bin lua-static-audit >/dev/null 2>&1) || {
    harness::fail "Failed to build lua-static-audit"
    harness::report "{\"lane\":\"lua_bindings\",\"sub_lane\":\"static\",\"status\":\"ERROR\",\"reason\":\"build failed\"}"
    exit 1
  }
fi

# ─── Run static audit ────────────────────────────────────────────────────────
harness::info "Running Lua binding static audit (cpp=$(basename "$CPP_LUASCRIPT") rust=crates/scripting/src/)..."

set +e
"$AUDIT_BIN" \
  --cpp  "$CPP_LUASCRIPT" \
  --rust "$RUST_SCRIPTING" \
  --out  "$REPORT_OUT"
audit_exit=$?
set -e

if [ ! -f "$REPORT_OUT" ]; then
  harness::fail "Audit binary produced no report"
  exit 1
fi

# Inline the audit report into the run report so the ledger writer can
# consume the ledger_entries it emits.
cat "$REPORT_OUT" >> "${HARNESS_RUN_REPORT:-/dev/null}" 2>/dev/null || true
# Newline-terminate so JSON-lines parsing stays sane.
echo >> "${HARNESS_RUN_REPORT:-/dev/null}" 2>/dev/null || true

# ─── Summary ─────────────────────────────────────────────────────────────────
cpp_total=$(grep -o '"cpp_total":[0-9]*'   "$REPORT_OUT" | grep -o '[0-9]*' || echo "?")
rust_total=$(grep -o '"rust_total":[0-9]*' "$REPORT_OUT" | grep -o '[0-9]*' || echo "?")
missing=$(grep -o '"missing_in_rust":\[[^]]*\]' "$REPORT_OUT" | grep -o ',' | wc -l | tr -d ' ' || echo "0")
# +1 if the array is non-empty (we counted commas; entries = commas + 1).
if grep -q '"missing_in_rust":\[\]' "$REPORT_OUT"; then missing=0; else missing=$((missing + 1)); fi

if [ "$audit_exit" = "0" ]; then
  harness::ok "Lua static audit PASS — cpp=$cpp_total rust=$rust_total (surfaces match)"
  exit 0
fi
harness::warn "Lua static audit FAIL — cpp=$cpp_total rust=$rust_total missing_in_rust=$missing"
harness::info "Report: $REPORT_OUT"
exit 1
