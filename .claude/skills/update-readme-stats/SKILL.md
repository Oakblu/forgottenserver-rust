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

## Dependencies

Invokes `format-lint-test` before running coverage. If format, lint, or tests fail, the script aborts before touching `README.md`.

---

## On-Demand Invocation

Run from the project root:

```bash
bash .claude/skills/update-readme-stats/update_badges.sh
```

The script:
1. Runs `format-lint-test` (format → lint → test); aborts on any failure.
2. Detects `cargo-llvm-cov` (preferred) or `cargo-tarpaulin`; aborts if neither found.
3. Runs the coverage tool, which also runs all unit tests internally.
4. Parses test count and line coverage % from its output.
5. Counts Rust LOC (excluding `target/`) and workspace crate count.
6. Calls `patch_badges.py` to update `README.md` in-place.
7. Prints `<badge>: <old> -> <new>` for each changed badge.

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
