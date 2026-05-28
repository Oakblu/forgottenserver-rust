---
name: format-lint-test
description: Run format, lint, and test in sequence. Use after any code change, before marking a task done, or whenever quality gates need to be confirmed. Usable by agents (via Skill tool), other skills (via run.sh), and humans (direct bash).
---

# format-lint-test

Runs the three mandatory quality gates in order. Each step must pass before the next runs.

| Step | Command | Passes when |
|------|---------|-------------|
| Format | `cargo fmt --all` | No diffs produced |
| Lint | `cargo clippy --workspace --lib --tests -- -D warnings` | Zero errors and zero warnings |
| Test | `cargo test --lib --workspace` | All tests pass |

This skill is **rigid** — steps run in order, never skipped.

---

## Invocation

### Via Skill tool (agents / other skills)
```
Skill("format-lint-test")
```

### Via bash (humans / shell scripts)
```bash
bash .claude/skills/format-lint-test/run.sh
```

The script exits non-zero on the first failure so callers can detect which gate failed.

---

## Failure Modes

| Condition | Behaviour |
|-----------|-----------|
| `cargo fmt` produces a diff | Exits 1 after formatting. Re-run to confirm clean, then continue. |
| `cargo clippy` emits any warning or error | Exits 1. Fix all clippy output before proceeding. |
| Any test fails | Exits 1. Fix failing tests before proceeding. |
