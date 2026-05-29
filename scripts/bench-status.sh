#!/usr/bin/env bash
#
# bench-status.sh — Status-port latency comparison: C++ upstream vs Rust port.
#
# Sends N HTTP GET probes to each server's status port and reports
# p50 / p95 / p99 latency plus error rate for both sides.
#
# Prerequisites:
#   docker compose -f docker-compose.perf.yml up --build -d
#   (wait for both servers to log "Server Online")
#
# Usage:
#   bash scripts/bench-status.sh [N]        # N = probe count, default 500
#
# Output example:
#   Scenario: status_probe (500 probes each)
#                       C++ (port 7371)   Rust (port 7471)
#   p50 latency         4ms               3ms
#   p95 latency         9ms               6ms
#   p99 latency         14ms              9ms
#   errors              0 (0.0%)          0 (0.0%)

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"
cd "$REPO_ROOT"

N="${1:-500}"
CPP_HOST="127.0.0.1"
CPP_PORT="7371"
RUST_HOST="127.0.0.1"
RUST_PORT="7471"
PROBE=$'GET / HTTP/1.0\r\n\r\n'
TIMEOUT_S=2

color() { printf "\033[%sm%s\033[0m" "$1" "$2"; }
ok()    { echo "$(color "1;32" "[ OK ]") $*"; }
info()  { echo "$(color "1;34" "[INFO]") $*"; }
fail()  { echo "$(color "1;31" "[FAIL]") $*" >&2; }

# ── Check nc availability ──────────────────────────────────────────────────
if ! command -v nc >/dev/null 2>&1; then
  fail "nc (netcat) not found — install netcat-openbsd or ncat"
  exit 1
fi

# ── Probe a single host:port, return latency in ms or "ERR" ───────────────
probe_once() {
  local host="$1" port="$2"
  local t0 t1 response
  t0=$(date +%s%N)
  response=$(printf '%s' "$PROBE" | nc -w "$TIMEOUT_S" "$host" "$port" 2>/dev/null || true)
  t1=$(date +%s%N)
  if [ -z "$response" ]; then
    echo "ERR"
  else
    echo $(( (t1 - t0) / 1000000 ))
  fi
}

# ── Run N probes against a host:port, collect latencies ───────────────────
run_probes() {
  local host="$1" port="$2" label="$3"
  local latencies=() errors=0 i result

  info "Probing $label ($host:$port) — $N probes..."
  for (( i=1; i<=N; i++ )); do
    result=$(probe_once "$host" "$port")
    if [ "$result" = "ERR" ]; then
      (( errors++ )) || true
    else
      latencies+=("$result")
    fi
    # Print progress every 100 probes
    if (( i % 100 == 0 )); then
      printf "  %d/%d\r" "$i" "$N"
    fi
  done
  printf "                \r"  # clear progress line

  # Sort latencies numerically and compute percentiles
  if [ ${#latencies[@]} -eq 0 ]; then
    echo "ERR ERR ERR $errors"
    return
  fi

  local sorted
  IFS=$'\n' sorted=($(printf '%s\n' "${latencies[@]}" | sort -n))
  local count=${#sorted[@]}

  p50_idx=$(( (count * 50) / 100 ))
  p95_idx=$(( (count * 95) / 100 ))
  p99_idx=$(( (count * 99) / 100 ))

  # Clamp indices to valid range
  [[ $p50_idx -ge $count ]] && p50_idx=$(( count - 1 ))
  [[ $p95_idx -ge $count ]] && p95_idx=$(( count - 1 ))
  [[ $p99_idx -ge $count ]] && p99_idx=$(( count - 1 ))

  echo "${sorted[$p50_idx]} ${sorted[$p95_idx]} ${sorted[$p99_idx]} $errors"
}

# ── Verify both servers are reachable before benchmarking ─────────────────
info "Checking connectivity to both servers..."
cpp_check=$(probe_once "$CPP_HOST" "$CPP_PORT")
rust_check=$(probe_once "$RUST_HOST" "$RUST_PORT")

if [ "$cpp_check" = "ERR" ]; then
  fail "C++ server not reachable at $CPP_HOST:$CPP_PORT"
  fail "Run: docker compose -f docker-compose.perf.yml up --build -d"
  exit 1
fi
if [ "$rust_check" = "ERR" ]; then
  fail "Rust server not reachable at $RUST_HOST:$RUST_PORT"
  fail "Run: docker compose -f docker-compose.perf.yml up --build -d"
  exit 1
fi
ok "Both servers reachable"

# ── Run benchmarks ─────────────────────────────────────────────────────────
echo
cpp_result=$(run_probes "$CPP_HOST" "$CPP_PORT" "C++ upstream")
rust_result=$(run_probes "$RUST_HOST" "$RUST_PORT" "Rust port")

read -r cpp_p50 cpp_p95 cpp_p99 cpp_errors <<< "$cpp_result"
read -r rust_p50 rust_p95 rust_p99 rust_errors <<< "$rust_result"

cpp_err_pct=$(awk "BEGIN { printf \"%.1f\", ($cpp_errors / $N) * 100 }")
rust_err_pct=$(awk "BEGIN { printf \"%.1f\", ($rust_errors / $N) * 100 }")

# ── Report ─────────────────────────────────────────────────────────────────
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo " Scenario: status_probe ($N probes each)"
printf " %-24s %-20s %-20s\n" "" "C++ (port $CPP_PORT)" "Rust (port $RUST_PORT)"
echo "────────────────────────────────────────────────────────────────────"
printf " %-24s %-20s %-20s\n" "p50 latency" "${cpp_p50}ms" "${rust_p50}ms"
printf " %-24s %-20s %-20s\n" "p95 latency" "${cpp_p95}ms" "${rust_p95}ms"
printf " %-24s %-20s %-20s\n" "p99 latency" "${cpp_p99}ms" "${rust_p99}ms"
printf " %-24s %-20s %-20s\n" "errors" "$cpp_errors ($cpp_err_pct%)" "$rust_errors ($rust_err_pct%)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
