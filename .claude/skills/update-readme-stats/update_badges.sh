#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SKILL_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$REPO_ROOT"

# ── 1. Detect coverage tool ──────────────────────────────────────────────────
if cargo llvm-cov --version &>/dev/null; then
    COVERAGE_TOOL="llvm-cov"
elif cargo tarpaulin --version &>/dev/null; then
    COVERAGE_TOOL="tarpaulin"
else
    echo "ERROR: Neither cargo-llvm-cov nor cargo-tarpaulin found." >&2
    echo "  Install with: cargo install cargo-llvm-cov" >&2
    exit 1
fi

echo ">> Coverage tool: $COVERAGE_TOOL"

# ── 2. Run cargo test for test count ─────────────────────────────────────────
echo ">> Running cargo test --lib --workspace ..."
if ! test_output=$(cargo test --lib --workspace 2>&1); then
    echo "ERROR: cargo test failed (tests failed)." >&2
    echo "$test_output" | tail -30 >&2
    exit 1
fi

# Sum all "N passed" lines across crates
# || true: grep exits 1 on no-match; set -e would abort before our validation check
test_count=$(echo "$test_output" \
    | grep 'test result: ok' \
    | grep -oE '[0-9]+ passed' \
    | awk '{total += $1} END {print (total+0)}') || true

# ── 3. Validate test count ───────────────────────────────────────────────────
if [[ -z "$test_count" || "$test_count" -eq 0 ]]; then
    echo "ERROR: Could not parse test count. Showing raw output tail:" >&2
    echo "$test_output" | tail -20 >&2
    exit 1
fi

echo ">> Tests passed: $test_count"

# ── 4. Run coverage tool for coverage % only ─────────────────────────────────
coverage=""

if [[ "$COVERAGE_TOOL" == "llvm-cov" ]]; then
    echo ">> Running cargo llvm-cov --lib --workspace --summary-only ..."
    if ! cov_output=$(cargo llvm-cov --lib --workspace --summary-only 2>&1); then
        if cargo tarpaulin --version &>/dev/null; then
            echo "WARNING: cargo llvm-cov failed (possibly OOM). Falling back to tarpaulin..." >&2
            COVERAGE_TOOL="tarpaulin"
        else
            echo "ERROR: cargo llvm-cov failed." >&2
            echo "$cov_output" | tail -30 >&2
            exit 1
        fi
    else
        # Line coverage = 3rd percentage on the TOTAL row
        coverage=$(echo "$cov_output" \
            | grep '^TOTAL' \
            | grep -oE '[0-9]+\.[0-9]+%' \
            | sed -n '3p' \
            | tr -d '%') || true
    fi
fi

if [[ "$COVERAGE_TOOL" == "tarpaulin" ]]; then
    echo ">> Running cargo tarpaulin --lib --workspace ..."
    if ! cov_output=$(cargo tarpaulin --lib --workspace 2>&1); then
        echo "ERROR: cargo tarpaulin failed." >&2
        echo "$cov_output" | tail -30 >&2
        exit 1
    fi

    # tarpaulin prints "XX.XX% coverage, ..."
    coverage=$(echo "$cov_output" \
        | grep -E '^[0-9]+\.[0-9]+% coverage' \
        | grep -oE '^[0-9]+\.[0-9]+' \
        | tail -1) || true
fi

# ── 5. Validate coverage ─────────────────────────────────────────────────────
if [[ -z "$coverage" ]]; then
    echo "ERROR: Could not parse coverage percentage. Showing raw output tail:" >&2
    echo "$cov_output" | tail -20 >&2
    exit 1
fi

echo ">> Coverage:     ${coverage}%"

# ── 6. Count Rust LOC (exclude target/, sum per-file wc -l) ─────────────────
raw_loc=$(find . -path ./target -prune -o -name '*.rs' -print \
    | xargs wc -l 2>/dev/null \
    | awk '$2 != "total" {sum += $1} END {print sum+0}') || true
raw_loc="${raw_loc:-0}"
loc_k="$(( (raw_loc + 500) / 1000 ))k"
echo ">> Rust LOC:     $loc_k  (${raw_loc} lines)"

# ── 7. Count workspace crate members ─────────────────────────────────────────
crates=$(grep -c '^\s*"crates/' Cargo.toml) || true
if [[ -z "$crates" || "$crates" -eq 0 ]]; then
    echo "ERROR: Could not count workspace crates from Cargo.toml." >&2
    exit 1
fi
echo ">> Crates:       $crates"

# ── 8. Patch README ──────────────────────────────────────────────────────────
python3 "$SKILL_DIR/patch_badges.py" "$test_count" "$coverage" "$loc_k" "$crates"
echo ">> README.md updated."
