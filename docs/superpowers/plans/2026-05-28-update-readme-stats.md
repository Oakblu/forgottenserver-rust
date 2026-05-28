# update-readme-stats Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a project-level Claude Code skill that runs the test suite, collects coverage and code metrics, and patches four shield.io badges in README.md — on demand and automatically after successful `cargo test` runs via a PostToolUse hook.

**Architecture:** A shell driver (`update_badges.sh`) invokes the coverage tool (llvm-cov preferred, tarpaulin fallback), which runs tests internally and emits both a test count and a line-coverage percentage. A Python script (`patch_badges.py`) does regex-safe README patching. A thin hook shim (`hook_trigger.sh`) reads the PostToolUse JSON payload from stdin and fires the driver only after a successful `cargo test` Bash call.

**Tech Stack:** Bash, Python 3 stdlib (`re`, `sys`), `cargo-llvm-cov` / `cargo-tarpaulin`

> **Git rule (mandatory):** Per CLAUDE.md, no `git commit`, `git push`, `git checkout`, or branch operations without explicit user approval in the current message. All commit steps below must be confirmed with the user before running.

---

## File Map

| Path | Action | Responsibility |
|------|--------|----------------|
| `.claude/skills/update-readme-stats/SKILL.md` | Create | Skill definition, on-demand invocation, failure modes |
| `.claude/skills/update-readme-stats/update_badges.sh` | Create | Shell driver: detect tool, run coverage, count LOC+crates, call patcher |
| `.claude/skills/update-readme-stats/patch_badges.py` | Create | Python: build shield.io URLs, regex-patch README.md in-place |
| `.claude/skills/update-readme-stats/hook_trigger.sh` | Create | PostToolUse shim: parse JSON stdin, fire driver on successful cargo test |
| `.claude/skills/update-readme-stats/tests/__init__.py` | Create | Makes tests/ a package for unittest discover |
| `.claude/skills/update-readme-stats/tests/test_patch_badges.py` | Create | unittest tests for patch_badges.py |
| `.claude/settings.local.json` | Modify | Add `hooks.PostToolUse` block |

---

### Task 1: patch_badges.py (TDD — tests first)

**Files:**
- Create: `.claude/skills/update-readme-stats/tests/__init__.py`
- Create: `.claude/skills/update-readme-stats/tests/test_patch_badges.py`
- Create: `.claude/skills/update-readme-stats/patch_badges.py`

- [ ] **Step 1: Create skill directory tree**

```bash
mkdir -p .claude/skills/update-readme-stats/tests
touch .claude/skills/update-readme-stats/tests/__init__.py
```

- [ ] **Step 2: Write the failing tests**

Write `.claude/skills/update-readme-stats/tests/test_patch_badges.py`:

```python
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
import patch_badges


class TestCoverageColor(unittest.TestCase):
    def test_95_is_brightgreen(self):
        self.assertEqual(patch_badges.coverage_color(95.0), 'brightgreen')

    def test_90_is_brightgreen(self):
        self.assertEqual(patch_badges.coverage_color(90.0), 'brightgreen')

    def test_89_is_green(self):
        self.assertEqual(patch_badges.coverage_color(89.9), 'green')

    def test_80_is_green(self):
        self.assertEqual(patch_badges.coverage_color(80.0), 'green')

    def test_79_is_yellow(self):
        self.assertEqual(patch_badges.coverage_color(79.9), 'yellow')

    def test_70_is_yellow(self):
        self.assertEqual(patch_badges.coverage_color(70.0), 'yellow')

    def test_69_is_orange(self):
        self.assertEqual(patch_badges.coverage_color(69.9), 'orange')

    def test_60_is_orange(self):
        self.assertEqual(patch_badges.coverage_color(60.0), 'orange')

    def test_59_is_red(self):
        self.assertEqual(patch_badges.coverage_color(59.9), 'red')


_SAMPLE_README = """\
# forgottenserver-rust

![tests](https://img.shields.io/badge/tests-6255%20passing-brightgreen?style=flat-square)
![coverage](https://img.shields.io/badge/coverage-94.85%25-brightgreen?style=flat-square)
![crates](https://img.shields.io/badge/crates-13-orange?style=flat-square&logo=rust&logoColor=white)
![loc](https://img.shields.io/badge/rust%20LOC-136k-lightgrey?style=flat-square)
"""


class TestPatchReadme(unittest.TestCase):
    def _tmp(self, content=_SAMPLE_README):
        f = tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False)
        f.write(content)
        f.close()
        return f.name

    def test_patches_tests_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 7000, 96.12, '140k', 14)
            content = open(path).read()
            self.assertIn('badge/tests-7000%20passing-brightgreen', content)
        finally:
            os.unlink(path)

    def test_patches_coverage_badge_value(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 94.85, '10k', 5)
            content = open(path).read()
            self.assertIn('badge/coverage-94.85%25-brightgreen', content)
        finally:
            os.unlink(path)

    def test_patches_loc_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 90.0, '150k', 13)
            content = open(path).read()
            self.assertIn('badge/rust%20LOC-150k-lightgrey', content)
        finally:
            os.unlink(path)

    def test_patches_crates_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 90.0, '136k', 15)
            content = open(path).read()
            self.assertIn('badge/crates-15-orange', content)
        finally:
            os.unlink(path)

    def test_coverage_color_yellow_reflected_in_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 75.0, '100k', 10)
            content = open(path).read()
            self.assertIn('badge/coverage-75.00%25-yellow', content)
        finally:
            os.unlink(path)

    def test_zero_tests_uses_red(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 0, 90.0, '136k', 13)
            content = open(path).read()
            self.assertIn('badge/tests-0%20passing-red', content)
        finally:
            os.unlink(path)

    def test_missing_badge_raises_system_exit(self):
        path = self._tmp('# No badges here\n')
        try:
            with self.assertRaises(SystemExit):
                patch_badges.patch_readme(path, 100, 90.0, '10k', 5)
        finally:
            os.unlink(path)

    def test_file_unchanged_when_values_identical(self):
        path = self._tmp()
        try:
            original = open(path).read()
            # Match current README values exactly
            patch_badges.patch_readme(path, 6255, 94.85, '136k', 13)
            updated = open(path).read()
            self.assertEqual(original, updated)
        finally:
            os.unlink(path)
```

- [ ] **Step 3: Run tests — confirm they fail with ImportError**

```bash
python3 -m unittest discover -s .claude/skills/update-readme-stats/tests -v 2>&1 | head -20
```

Expected: `ModuleNotFoundError: No module named 'patch_badges'`

- [ ] **Step 4: Write patch_badges.py**

Write `.claude/skills/update-readme-stats/patch_badges.py`:

```python
#!/usr/bin/env python3
"""Patches shield.io badge URLs in README.md with fresh test/coverage metrics."""

import re
import sys


def coverage_color(pct: float) -> str:
    if pct >= 90:
        return "brightgreen"
    if pct >= 80:
        return "green"
    if pct >= 70:
        return "yellow"
    if pct >= 60:
        return "orange"
    return "red"


def patch_readme(
    path: str,
    test_count: int,
    coverage_pct: float,
    loc_k: str,
    crates_count: int,
) -> None:
    with open(path) as f:
        content = f.read()

    tests_color = "brightgreen" if test_count > 0 else "red"

    patches = [
        (
            r"badge/tests-[^?]+(?=\?style=flat-square)",
            f"badge/tests-{test_count}%20passing-{tests_color}",
            "tests",
        ),
        (
            r"badge/coverage-[^?]+(?=\?style=flat-square)",
            f"badge/coverage-{coverage_pct:.2f}%25-{coverage_color(coverage_pct)}",
            "coverage",
        ),
        (
            r"badge/rust%20LOC-[^?]+(?=\?style=flat-square)",
            f"badge/rust%20LOC-{loc_k}-lightgrey",
            "loc",
        ),
        (
            r"badge/crates-[^?]+(?=\?style=flat-square&logo=rust&logoColor=white)",
            f"badge/crates-{crates_count}-orange",
            "crates",
        ),
    ]

    for pattern, replacement, label in patches:
        if not re.search(pattern, content):
            sys.exit(f"ERROR: badge '{label}' pattern not found in {path}")
        old = re.search(pattern, content).group(0)
        content = re.sub(pattern, replacement, content)
        new = re.search(pattern, content).group(0)
        if old != new:
            print(f"{label}: {old} -> {new}")

    with open(path, "w") as f:
        f.write(content)


if __name__ == "__main__":
    if len(sys.argv) != 5:
        sys.exit(f"Usage: {sys.argv[0]} <test_count> <coverage_pct> <loc_k> <crates_count>")

    patch_readme(
        "README.md",
        int(sys.argv[1]),
        float(sys.argv[2]),
        sys.argv[3],
        int(sys.argv[4]),
    )
```

- [ ] **Step 5: Run tests — confirm all pass**

```bash
python3 -m unittest discover -s .claude/skills/update-readme-stats/tests -v
```

Expected output:
```
test_60_is_orange (test_patch_badges.TestCoverageColor) ... ok
test_69_is_orange (test_patch_badges.TestCoverageColor) ... ok
test_70_is_yellow (test_patch_badges.TestCoverageColor) ... ok
test_79_is_yellow (test_patch_badges.TestCoverageColor) ... ok
test_80_is_green (test_patch_badges.TestCoverageColor) ... ok
test_89_is_green (test_patch_badges.TestCoverageColor) ... ok
test_90_is_brightgreen (test_patch_badges.TestCoverageColor) ... ok
test_95_is_brightgreen (test_patch_badges.TestCoverageColor) ... ok
test_59_is_red (test_patch_badges.TestCoverageColor) ... ok
test_coverage_color_yellow_reflected_in_badge (test_patch_badges.TestPatchReadme) ... ok
test_file_unchanged_when_values_identical (test_patch_badges.TestPatchReadme) ... ok
test_missing_badge_raises_system_exit (test_patch_badges.TestPatchReadme) ... ok
test_patches_crates_badge (test_patch_badges.TestPatchReadme) ... ok
test_patches_coverage_badge_value (test_patch_badges.TestPatchReadme) ... ok
test_patches_loc_badge (test_patch_badges.TestPatchReadme) ... ok
test_patches_tests_badge (test_patch_badges.TestPatchReadme) ... ok
test_zero_tests_uses_red (test_patch_badges.TestPatchReadme) ... ok
----------------------------------------------------------------------
Ran 17 tests in X.XXXs

OK
```

---

### Task 2: update_badges.sh

**Files:**
- Create: `.claude/skills/update-readme-stats/update_badges.sh`

- [ ] **Step 1: Write update_badges.sh**

Write `.claude/skills/update-readme-stats/update_badges.sh`:

```bash
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

# ── 2. Run coverage (which also runs all unit tests) ─────────────────────────
if [[ "$COVERAGE_TOOL" == "llvm-cov" ]]; then
    echo ">> Running cargo llvm-cov --lib --workspace --summary-only ..."
    if ! output=$(cargo llvm-cov --lib --workspace --summary-only 2>&1); then
        echo "ERROR: cargo llvm-cov failed (tests may have failed)." >&2
        echo "$output" | tail -30 >&2
        exit 1
    fi

    # Sum all "N passed" lines across crates
    # || true: grep exits 1 on no-match; set -e would abort before our validation check
    test_count=$(echo "$output" \
        | grep 'test result: ok' \
        | grep -oE '[0-9]+ passed' \
        | awk '{total += $1} END {print (total+0)}') || true

    # Line coverage = 3rd percentage on the TOTAL row
    coverage=$(echo "$output" \
        | grep '^TOTAL' \
        | grep -oE '[0-9]+\.[0-9]+%' \
        | sed -n '3p' \
        | tr -d '%') || true
else
    echo ">> Running cargo tarpaulin --lib --workspace ..."
    if ! output=$(cargo tarpaulin --lib --workspace 2>&1); then
        echo "ERROR: cargo tarpaulin failed (tests may have failed)." >&2
        echo "$output" | tail -30 >&2
        exit 1
    fi

    # Sum all "N passed" lines across crates
    test_count=$(echo "$output" \
        | grep 'test result: ok' \
        | grep -oE '[0-9]+ passed' \
        | awk '{total += $1} END {print (total+0)}') || true

    # tarpaulin prints "XX.XX% coverage, ..."
    coverage=$(echo "$output" \
        | grep -E '^[0-9]+\.[0-9]+% coverage' \
        | grep -oE '^[0-9]+\.[0-9]+' \
        | tail -1) || true
fi

# ── 3. Validate parsed values ────────────────────────────────────────────────
if [[ -z "$test_count" || "$test_count" -eq 0 ]]; then
    echo "ERROR: Could not parse test count. Showing raw output tail:" >&2
    echo "$output" | tail -20 >&2
    exit 1
fi

if [[ -z "$coverage" ]]; then
    echo "ERROR: Could not parse coverage percentage. Showing raw output tail:" >&2
    echo "$output" | tail -20 >&2
    exit 1
fi

echo ">> Tests passed: $test_count"
echo ">> Coverage:     ${coverage}%"

# ── 4. Count Rust LOC (exclude target/, sum per-file wc -l) ─────────────────
raw_loc=$(find . -path ./target -prune -o -name '*.rs' -print \
    | xargs wc -l 2>/dev/null \
    | awk '$2 != "total" {sum += $1} END {print sum+0}') || true
loc_k="$(( (raw_loc + 500) / 1000 ))k"
echo ">> Rust LOC:     $loc_k  (${raw_loc} lines)"

# ── 5. Count workspace crate members ─────────────────────────────────────────
crates=$(grep -c '^\s*"crates/' Cargo.toml)
echo ">> Crates:       $crates"

# ── 6. Patch README ──────────────────────────────────────────────────────────
python3 "$SKILL_DIR/patch_badges.py" "$test_count" "$coverage" "$loc_k" "$crates"
echo ">> README.md updated."
```

- [ ] **Step 2: Make it executable**

```bash
chmod +x .claude/skills/update-readme-stats/update_badges.sh
```

- [ ] **Step 3: Verify the script is syntactically valid**

```bash
bash -n .claude/skills/update-readme-stats/update_badges.sh && echo "syntax OK"
```

Expected: `syntax OK`

---

### Task 3: hook_trigger.sh

**Files:**
- Create: `.claude/skills/update-readme-stats/hook_trigger.sh`

- [ ] **Step 1: Write hook_trigger.sh**

Write `.claude/skills/update-readme-stats/hook_trigger.sh`:

```bash
#!/usr/bin/env bash
# PostToolUse hook — fires update_badges.sh after a successful cargo test call.
# Claude Code pipes the tool-use JSON payload to this script's stdin.

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SKILL_DIR="$(cd "$(dirname "$0")" && pwd)"

payload=$(cat)

# Extract the command that was run
command=$(echo "$payload" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    print(d.get('tool_input', {}).get('command', ''))
except Exception:
    print('')
" 2>/dev/null)

# Only act on cargo test invocations
if ! echo "$command" | grep -q 'cargo test'; then
    exit 0
fi

# Extract exit code from the tool response
exit_code=$(echo "$payload" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    resp = d.get('tool_response', {})
    code = resp.get('exit_code', resp.get('exitCode'))
    if code is None:
        out = str(resp.get('output', '') or resp.get('content', ''))
        code = 1 if ('FAILED' in out or 'error[' in out) else 0
    print(int(code))
except Exception:
    print(1)
" 2>/dev/null)

if [[ "$exit_code" != "0" ]]; then
    exit 0
fi

# Run badge update in background; silence all output so Claude's UI is not polluted
cd "$REPO_ROOT"
bash "$SKILL_DIR/update_badges.sh" >/dev/null 2>&1 &
```

- [ ] **Step 2: Make it executable**

```bash
chmod +x .claude/skills/update-readme-stats/hook_trigger.sh
```

- [ ] **Step 3: Verify syntax**

```bash
bash -n .claude/skills/update-readme-stats/hook_trigger.sh && echo "syntax OK"
```

Expected: `syntax OK`

- [ ] **Step 4: Smoke test — non-cargo-test command is silently ignored**

```bash
echo '{"tool_input":{"command":"ls -la"},"tool_response":{"exit_code":0}}' \
  | bash .claude/skills/update-readme-stats/hook_trigger.sh
echo "exit: $?"
```

Expected: exit 0, no output, README unchanged.

- [ ] **Step 5: Smoke test — failed cargo test is silently ignored**

```bash
echo '{"tool_input":{"command":"cargo test --lib --workspace"},"tool_response":{"exit_code":1,"output":"FAILED"}}' \
  | bash .claude/skills/update-readme-stats/hook_trigger.sh
echo "exit: $?"
```

Expected: exit 0, no output, README unchanged.

---

### Task 4: SKILL.md

**Files:**
- Create: `.claude/skills/update-readme-stats/SKILL.md`

- [ ] **Step 1: Write SKILL.md**

Write `.claude/skills/update-readme-stats/SKILL.md`:

```markdown
---
name: update-readme-stats
description: Use when asked to update README test/coverage badges, after running tests manually, or to refresh LOC and crate counts. Also fires automatically via hook after any successful `cargo test` run.
---

# update-readme-stats

Updates four shield.io badges in `README.md` with live data from the test suite and codebase:

| Badge | Source |
|-------|--------|
| `tests` | passing count from coverage tool output |
| `coverage` | line coverage % (`cargo llvm-cov` preferred, `cargo tarpaulin` fallback) |
| `loc` | total Rust LOC (`find . -name '*.rs' | xargs wc -l`) |
| `crates` | workspace member count from `Cargo.toml` |

This skill is **rigid** — follow steps in order.

---

## On-Demand Invocation

Run from the project root:

```bash
bash .claude/skills/update-readme-stats/update_badges.sh
```

The script:
1. Detects `cargo-llvm-cov` (preferred) or `cargo-tarpaulin`; aborts if neither found.
2. Runs the coverage tool, which also runs all unit tests internally.
3. Parses test count and line coverage % from its output.
4. Counts Rust LOC (excluding `target/`) and workspace crate count.
5. Calls `patch_badges.py` to update `README.md` in-place.
6. Prints `<badge>: <old> -> <new>` for each changed badge.

**No git operations.** Report what changed and stop. User commits when ready.

---

## Failure Modes

| Condition | Action |
|-----------|--------|
| Neither coverage tool installed | Abort. Print `cargo install cargo-llvm-cov`. |
| Coverage tool exits non-zero | Abort. Print last 30 lines of output for debugging. |
| Test count parsed as 0 | Abort. Print last 20 lines of output for debugging. |
| Coverage % not found in output | Abort. Print last 20 lines of output for debugging. |
| Badge label not found in README | Abort. Print which label was missing. Leave README unchanged. |

---

## Auto-Hook

A `PostToolUse` hook fires `hook_trigger.sh` after every Bash tool call.
The shim silently no-ops unless the command contained `cargo test` and exit code was 0.
When triggered, it calls `update_badges.sh` in the background (stdout suppressed).

Note: the hook re-runs the coverage tool, so tests execute twice per Claude session
`cargo test` call. This is intentional — the hook provides fresh coverage data automatically.

---

## Installing Coverage Tools

```bash
# Preferred
cargo install cargo-llvm-cov

# Fallback
cargo install cargo-tarpaulin
```
```

---

### Task 5: Register the PostToolUse hook

**Files:**
- Modify: `.claude/settings.local.json`

- [ ] **Step 1: Read current settings to verify structure**

```bash
cat .claude/settings.local.json
```

Expected: JSON with a `permissions.allow` array and no existing `hooks` key.

- [ ] **Step 2: Add the hooks block**

The file currently has only a `"permissions"` key. Add a top-level `"hooks"` key. The complete updated `.claude/settings.local.json` must be:

```json
{
  "permissions": {
    "allow": [
      "Bash(cargo test *)",
      "Bash(awk '-F[. ;]' '{for\\(i=1;i<=NF;i++\\) if\\($i==\"passed\"\\) print $\\(i-1\\)}')",
      "Bash(bc)",
      "Bash(rustc --version)",
      "Bash(cargo --version)",
      "Bash(cargo llvm-cov *)",
      "Bash(cargo tarpaulin *)",
      "Bash(python3 /Users/pablohpsilva/Documents/forgottenserver-rust/scripts/ledger/validate.py --help)",
      "Bash(python3 /Users/pablohpsilva/Documents/forgottenserver-rust/scripts/ledger/cross_validate.py --help)",
      "Bash(python3 /Users/pablohpsilva/Documents/forgottenserver-rust/scripts/ledger/extract_layer_scopes.py --help)",
      "Bash(cargo check *)",
      "Bash(git add *)",
      "Bash(git commit -m ' *)",
      "Bash(git commit *)",
      "Bash(docker compose *)",
      "Bash(cargo clippy *)",
      "Bash(docker stop poketibia-tfs-1 *)",
      "Bash(python3 -c ' *)",
      "Bash(git -C /Users/pablohpsilva/Documents/forgottenserver-rust log --oneline --all)",
      "Bash(git *)",
      "Bash(bash *)",
      "Bash(python3 *)",
      "Bash(awk -F: '{print $2}')",
      "Read(//Users/pablohpsilva/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/**)",
      "Read(//Users/pablohpsilva/.claude/plugins/**)",
      "Bash(cargo build *)"
    ]
  },
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash /Users/pablohpsilva/Documents/forgottenserver-rust/.claude/skills/update-readme-stats/hook_trigger.sh"
          }
        ]
      }
    ]
  }
}
```

- [ ] **Step 3: Validate JSON**

```bash
python3 -m json.tool .claude/settings.local.json > /dev/null && echo "JSON valid"
```

Expected: `JSON valid`

---

### Task 6: End-to-end verification

- [ ] **Step 1: Run all Python tests**

```bash
python3 -m unittest discover -s .claude/skills/update-readme-stats/tests -v
```

Expected: `Ran 17 tests ... OK`

- [ ] **Step 2: Run the skill on-demand**

```bash
bash .claude/skills/update-readme-stats/update_badges.sh
```

Expected output pattern:
```
>> Coverage tool: llvm-cov
>> Running cargo llvm-cov --lib --workspace --summary-only ...
>> Tests passed: NNNN
>> Coverage:     NN.NN%
>> Rust LOC:     NNNk  (NNNNNN lines)
>> Crates:       NN
>> README.md updated.
```

Each badge line with a changed value prints `<badge>: <old> -> <new>`.

- [ ] **Step 3: Verify README diff is correct**

```bash
git diff README.md
```

Confirm the four badge URLs were updated to match the current values.

- [ ] **Step 4: Verify hook smoke tests still pass**

```bash
echo '{"tool_input":{"command":"ls"},"tool_response":{"exit_code":0}}' \
  | bash .claude/skills/update-readme-stats/hook_trigger.sh && echo "no-op: OK"

echo '{"tool_input":{"command":"cargo test --lib"},"tool_response":{"exit_code":1}}' \
  | bash .claude/skills/update-readme-stats/hook_trigger.sh && echo "failed-test no-op: OK"
```

Expected: both print their confirmation and exit 0.

- [ ] **Step 5: Confirm skill is listed in Claude Code**

```bash
ls .claude/skills/update-readme-stats/
```

Expected:
```
SKILL.md
hook_trigger.sh
patch_badges.py
tests/
update_badges.sh
```

The skill will appear as `update-readme-stats` in Claude Code's skill list on next session load.
