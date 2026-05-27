#!/usr/bin/env bash
#
# lib.sh — Shared helpers for harness lane drivers.
#
# Source from each lane driver:
#   source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"
#
# Exposes:
#   harness::up           — `docker compose up -d` the side-by-side stack
#   harness::down         — stop + remove the harness stack
#   harness::ready        — block until both servers report Online (max 120s)
#   harness::report       — append a {lane, scenario, status, ...} JSON record
#                           to the current run report
#   harness::diff         — byte-diff with the protocol-aware normalizer
#                           (placeholder until lane 1 lands)
#   harness::db_snapshot  — mysqldump filtered by tables for a logical DB
#                           (placeholder until lane 5 lands)
#
# All helpers use `set -euo pipefail`-friendly conventions; do not assume
# the caller already has `set -e`.

# ─── Path constants ──────────────────────────────────────────────────────────
# lib.sh lives at scripts/harness/lib.sh, so the harness root is the same dir.
HARNESS_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
HARNESS_REPORTS_DIR="$HARNESS_DIR/reports"
# scripts/harness → scripts → forgottenserver-rust root
REPO_ROOT="$(cd -- "$HARNESS_DIR/../../.." >/dev/null 2>&1 && pwd)"

# ─── Color helpers ───────────────────────────────────────────────────────────
harness::color() { printf "\033[%sm%s\033[0m" "$1" "$2"; }
harness::ok()    { echo "$(harness::color "1;32" "[ OK ]") $*"; }
harness::info()  { echo "$(harness::color "1;34" "[INFO]") $*"; }
harness::warn()  { echo "$(harness::color "1;33" "[WARN]") $*"; }
harness::fail()  { echo "$(harness::color "1;31" "[FAIL]") $*" >&2; }

# ─── Service definitions ─────────────────────────────────────────────────────
HARNESS_SERVICES=(
  db
  forgottenserver-cpp
  forgottenserver-rust
)

HARNESS_CPP_DB="tibia_cpp"
HARNESS_RUST_DB="tibia_rs"
HARNESS_DB_USER="forgottenserver"
HARNESS_DB_PASSWORD="forgottenserver"

# ─── harness::up ─────────────────────────────────────────────────────────────
# Bring up the side-by-side stack via docker compose.
# Usage: harness::up
harness::up() {
  harness::info "Bringing up harness stack (${HARNESS_SERVICES[*]})..."
  (cd "$REPO_ROOT" && docker compose up -d "${HARNESS_SERVICES[@]}" >/dev/null)
}

# ─── harness::down ───────────────────────────────────────────────────────────
# Stop and remove the harness stack. Idempotent.
# Usage: harness::down
harness::down() {
  harness::info "Tearing down harness stack..."
  (cd "$REPO_ROOT" && docker compose stop "${HARNESS_SERVICES[@]}" >/dev/null 2>&1 || true)
  (cd "$REPO_ROOT" && docker compose rm -f "${HARNESS_SERVICES[@]}" >/dev/null 2>&1 || true)
}

# ─── harness::ready ──────────────────────────────────────────────────────────
# Block until db is healthy AND both servers report Online.
# Returns 0 on success, 1 on timeout. Default timeout: 120 seconds.
# Usage: harness::ready [timeout_seconds]
harness::ready() {
  local timeout="${1:-120}"
  local i

  harness::info "Waiting for db to report healthy (max ${timeout}s)..."
  for i in $(seq 1 "$timeout"); do
    local health
    health=$(docker inspect --format='{{.State.Health.Status}}' forgottenserver-rust-db-1 2>/dev/null || echo "starting")
    if [ "$health" = "healthy" ]; then
      harness::ok "MariaDB healthy after ${i}s"
      break
    fi
    if [ "$i" = "$timeout" ]; then
      harness::fail "MariaDB did not become healthy within ${timeout}s"
      return 1
    fi
    sleep 1
  done

  harness::info "Waiting for both servers to reach Online (max ${timeout}s)..."
  local cpp_online=0 rust_online=0
  for i in $(seq 1 "$timeout"); do
    local cpp_log rust_log
    cpp_log=$(cd "$REPO_ROOT" && docker compose logs forgottenserver-cpp 2>&1 || true)
    rust_log=$(cd "$REPO_ROOT" && docker compose logs forgottenserver-rust 2>&1 || true)
    if echo "$cpp_log"  | grep -q "Server Online";          then cpp_online=1; fi
    if echo "$rust_log" | grep -q "Forgotten Server Online"; then rust_online=1; fi
    if [ "$cpp_online" = "1" ] && [ "$rust_online" = "1" ]; then
      harness::ok "Both servers online after ${i}s"
      return 0
    fi
    if [ "$i" = "$timeout" ]; then
      harness::fail "Servers did not reach Online within ${timeout}s"
      [ "$cpp_online"  = "0" ] && harness::fail "  C++ did not come online"
      [ "$rust_online" = "0" ] && harness::fail "  Rust did not come online"
      return 1
    fi
    sleep 1
  done
}

# ─── harness::report ─────────────────────────────────────────────────────────
# Append a JSON record to the current run report.
# Usage: harness::report '{"lane":"wire_replay","scenario":"login_walk_logout","status":"PASS"}'
# The current run report path is taken from $HARNESS_RUN_REPORT, which is
# set by run.sh. If unset, prints to stdout.
harness::report() {
  local record="$1"
  if [ -z "${HARNESS_RUN_REPORT:-}" ]; then
    echo "$record"
    return
  fi
  echo "$record" >> "$HARNESS_RUN_REPORT"
}

# ─── harness::diff ───────────────────────────────────────────────────────────
# Placeholder: byte-diff with protocol-aware normalizer. Implemented when
# lane 1 (wire-replay) lands.
# Usage: harness::diff <left_file> <right_file> <packet_type>
harness::diff() {
  harness::warn "harness::diff is a placeholder until lane 1 ships"
  return 1
}

# ─── harness::db_snapshot ────────────────────────────────────────────────────
# Placeholder: mysqldump a named DB filtered by tables. Implemented when
# lane 5 (persisted-state) lands.
# Usage: harness::db_snapshot <db_name> <output_file> <table> [<table> ...]
harness::db_snapshot() {
  harness::warn "harness::db_snapshot is a placeholder until lane 5 ships"
  return 1
}

# ─── harness::scenario_count ─────────────────────────────────────────────────
# Count recorded capture scenarios. Used by lane drivers and run.sh for
# progress reporting.
harness::scenario_count() {
  find "$HARNESS_DIR/captures" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' '
}

# ─── harness::ensure_reports_dir ─────────────────────────────────────────────
# Make sure the reports/ directory exists. Called by run.sh on start.
harness::ensure_reports_dir() {
  mkdir -p "$HARNESS_REPORTS_DIR"
}
