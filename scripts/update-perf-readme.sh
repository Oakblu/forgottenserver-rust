#!/usr/bin/env bash
# update-perf-readme.sh
# Regenerate docs/performance/perf-results.json and docs/performance/README.md
# from a fresh perf-bot run against both servers.
#
# Prerequisites:
#   docker compose -f docker-compose.perf.yml up --build -d
#
# Usage:
#   bash scripts/update-perf-readme.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CPP_PORT=7372
RUST_PORT=7472
OUTPUT="$ROOT/docs/performance/perf-results.json"

# ── Preflight: perf stack must be running ────────────────────────────────────

probe_port() {
  local port=$1
  nc -z 127.0.0.1 "$port" 2>/dev/null
}

if ! probe_port "$CPP_PORT"; then
  echo "ERROR: C++ game port $CPP_PORT is not reachable." >&2
  echo "       Start the perf stack first:" >&2
  echo "         docker compose -f docker-compose.perf.yml up --build -d" >&2
  exit 1
fi

if ! probe_port "$RUST_PORT"; then
  echo "ERROR: Rust game port $RUST_PORT is not reachable." >&2
  echo "       Start the perf stack first:" >&2
  echo "         docker compose -f docker-compose.perf.yml up --build -d" >&2
  exit 1
fi

echo "Both servers reachable. Running perf-bot (login_flood, 20 bots, 60s) …"

# ── Run perf-bot ─────────────────────────────────────────────────────────────

cargo run --release -p perf-bot -- \
  --target both \
  --scenario login_flood \
  --bots 20 \
  --duration 60 \
  --output "$OUTPUT"

echo ""
echo "Results written to $OUTPUT"

# ── Regenerate README table ───────────────────────────────────────────────────

python3 - "$OUTPUT" "$ROOT/docs/performance/README.md" <<'PYEOF'
import json, sys, re

results_path, readme_path = sys.argv[1], sys.argv[2]

with open(results_path) as f:
    d = json.load(f)

cpp  = d["cpp"]
rust = d["rust"]
sc   = d.get("scenario", "login_flood")
bots = d.get("bot_count", "?")
dur  = int(d.get("duration_secs", 0))

def delta(cpp_val, rust_val, higher_is_better=True):
    if cpp_val == 0:
        return "n/a"
    pct = (rust_val - cpp_val) / cpp_val * 100
    sign = "+" if pct > 0 else ""
    better = (pct > 0) == higher_is_better
    marker = "**" if abs(pct) >= 5 else ""
    return f"{marker}{sign}{pct:.0f}%{marker}"

table = f"""### {sc.replace("_", "\\_")} — {bots} bots, {dur} s

| Metric | C++ | Rust | Delta |
|---|---|---|---|
| Actions / sec | {cpp["actions_per_sec"]:.1f} | {rust["actions_per_sec"]:.1f} | {delta(cpp["actions_per_sec"], rust["actions_per_sec"])} |
| p50 latency | {cpp["p50_ms"]} ms | {rust["p50_ms"]} ms | {delta(cpp["p50_ms"], rust["p50_ms"], higher_is_better=False)} |
| p95 latency | {cpp["p95_ms"]} ms | {rust["p95_ms"]} ms | {delta(cpp["p95_ms"], rust["p95_ms"], higher_is_better=False)} |
| p99 latency | {cpp["p99_ms"]} ms | {rust["p99_ms"]} ms | {delta(cpp["p99_ms"], rust["p99_ms"], higher_is_better=False)} |
| Error rate | {cpp["error_rate_pct"]:.2f}% | {rust["error_rate_pct"]:.2f}% | {delta(cpp["error_rate_pct"], rust["error_rate_pct"], higher_is_better=False)} |
| RSS start | {cpp["rss_start_mb"]:.0f} MB | {rust["rss_start_mb"]:.0f} MB | {delta(cpp["rss_start_mb"], rust["rss_start_mb"], higher_is_better=False)} |
| RSS end | {cpp["rss_end_mb"]:.0f} MB | {rust["rss_end_mb"]:.0f} MB | {delta(cpp["rss_end_mb"], rust["rss_end_mb"], higher_is_better=False)} |
| Peak RSS | {cpp["peak_rss_mb"]:.0f} MB | {rust["peak_rss_mb"]:.0f} MB | {delta(cpp["peak_rss_mb"], rust["peak_rss_mb"], higher_is_better=False)} |"""

with open(readme_path) as f:
    content = f.read()

# Replace the block between ## Results and --- (the first --- after ## Results)
pattern = r"(## Results\n\n).*?(\n---)"
replacement = r"\g<1>" + table + r"\g<2>"
new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)

if new_content == content:
    print("WARNING: README pattern not found — table was not updated.", file=sys.stderr)
    sys.exit(1)

with open(readme_path, "w") as f:
    f.write(new_content)

print(f"README updated: {readme_path}")
PYEOF

echo ""
echo "Done. Review the changes and commit:"
echo "  git add docs/performance/"
echo "  git commit -m 'perf: update benchmark results'"
