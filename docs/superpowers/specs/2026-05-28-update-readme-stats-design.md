# Design: update-readme-stats skill

**Date:** 2026-05-28  
**Status:** Approved

---

## Overview

A project-level Claude Code skill that runs the test suite, collects coverage and code metrics, and patches the four auto-updatable shield.io badges in `README.md` in-place. Fires on demand (via `/update-readme-stats`) and automatically after any successful `cargo test` Bash tool call (via a `PostToolUse` hook).

---

## Badges Updated

| Badge      | Source                                                       | Color logic                                                                      |
| ---------- | ------------------------------------------------------------ | -------------------------------------------------------------------------------- | -------------------- |
| `tests`    | `cargo test --lib --workspace` output — "N passed"           | `brightgreen` if >0, else `red`                                                  |
| `coverage` | `cargo llvm-cov` or `cargo tarpaulin` — line coverage %      | `brightgreen` ≥90%, `green` 80–89%, `yellow` 70–79%, `orange` 60–69%, `red` <60% |
| `loc`      | `find . -name '\*.rs'                                        | xargs wc -l`— total lines, rounded to`Nk`                                        | `lightgrey` (static) |
| `crates`   | `grep -c '^\s*"crates/' Cargo.toml` — workspace member count | `orange` (static)                                                                |

---

## File Layout

```
.claude/skills/update-readme-stats/
├── SKILL.md              # skill definition and on-demand invocation instructions
├── update_badges.sh      # shell driver: runs tests, collects numbers, calls Python patcher
├── patch_badges.py       # Python: constructs shield.io URLs, patches README.md in-place
└── hook_trigger.sh       # PostToolUse hook shim: fires update only on successful cargo test
```

---

## Component Designs

### `update_badges.sh`

Shell driver. Runs from the project root. Accepts `--skip-tests` flag (used by hook to avoid re-running tests).

Steps:

1. **Detect coverage tool** — `cargo llvm-cov --version` preferred; fallback to `cargo tarpaulin --version`; abort with install instructions if neither found.
2. **Run tests** (skipped if `--skip-tests`) — `cargo test --lib --workspace 2>&1`; grep `test result: ok. N passed`; extract N. Abort if tests fail — stale badge data is worse than no update.
3. **Run coverage** — llvm-cov: `cargo llvm-cov --lib --workspace --summary-only`; grep `TOTAL` line; extract `XX.XX%`. Tarpaulin: `cargo tarpaulin --lib --workspace`; grep `Coverage:`; extract percentage.
4. **Count LOC** — `find . -path ./target -prune -o -name '*.rs' -print | xargs wc -l`; sum; round to nearest `k`.
5. **Count crates** — `grep -c '^\s*"crates/' Cargo.toml`.
6. Call `python3 patch_badges.py "$test_count" "$coverage_pct" "$loc_k" "$crates_count"`.

### `patch_badges.py`

Python script. Four positional args: `test_count`, `coverage_pct`, `loc_k`, `crates_count`.

- Reads `README.md`.
- For each badge, builds the new shield.io URL:
  ```
  https://img.shields.io/badge/<label>-<message>-<color>?style=flat-square
  ```
- Replaces the old URL using a regex anchored on the label portion (e.g. `badge/tests-`) so the match is label-anchored, not value-dependent.
- Prints a `<badge>: <old> → <new>` line for each change.
- Writes `README.md` back in-place.
- Aborts with a clear error (non-zero exit, no file write) if any expected badge line is missing from the README.

Special characters in shield.io badge URLs:

- Spaces → `%20`
- `%` → `%25`

### `hook_trigger.sh`

Thin shim that reads the Claude Code `PostToolUse` JSON payload from stdin.

Logic:

1. Parse `tool_input.command` — skip if it does not contain `cargo test`.
2. Parse exit code from tool response — skip if non-zero.
3. `cd` to project root; call `update_badges.sh --skip-tests`.
4. Silence stdout to avoid polluting the Claude tool output; emit errors to stderr.

### Hook registration in `.claude/settings.local.json`

```json
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
```

Absolute path ensures the hook works regardless of working directory.

---

## `SKILL.md` Behaviour

**Rigid skill** — follow steps in order, no skipping.

**On-demand trigger:** User invokes `/update-readme-stats`. Skill calls `update_badges.sh` (full run including tests). Reports each badge's old→new value. Does **not** commit.

**Auto trigger:** `hook_trigger.sh` detects successful `cargo test` Bash call → calls `update_badges.sh --skip-tests`. Silent on no-change; reports changes if any badge value differs.

---

## Failure Modes

| Condition                                            | Behaviour                                                |
| ---------------------------------------------------- | -------------------------------------------------------- |
| Coverage tool not installed                          | Abort with instructions: `cargo install cargo-llvm-cov`  |
| `cargo test` fails                                   | Abort before touching README — stale pass count is a lie |
| Badge label not found in README                      | Report which pattern was missing; leave README unchanged |
| `patch_badges.py` receives malformed coverage output | Abort with raw output so user can debug the tool         |

---

## Constraints

- **No git operations.** Skill patches the file only. User commits when ready, per project rules.
- **No stub implementations.** All scripts must be fully functional.
- **Project root assumed.** Both `update_badges.sh` and `patch_badges.py` expect to be run from the repo root.
- **README format assumed stable.** Badge lines follow the existing `![label](https://img.shields.io/badge/...)` pattern.
