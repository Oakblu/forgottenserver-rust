---
name: upstream-sync-check
description: Use when checking whether the upstream otland/forgottenserver C++ repo has recent changes that need to be migrated into this Rust port. Use before any migration sprint, after upstream activity is suspected, or on periodic sync review.
---

# Upstream Sync Check

## Overview

Checks `https://github.com/otland/forgottenserver.git` for changes in the past 3 months, cross-references them against the migration ledger, and either proves no migration is needed (with evidence) or writes a migration plan for user review.

**Phases 1–4 are automated by [`run_sync.sh`](run_sync.sh), which calls [`sync_check.py`](sync_check.py) internally.**  
Run from the **project root**.

This skill is **rigid**: follow every phase in order. Do not skip phases. Do not open full `.cpp` files to discover symbols — manifests and ledger are the authoritative source.

---

## Phases 1–4 — Run the pipeline

```bash
bash .claude/skills/upstream-sync-check/run_sync.sh
# or with a custom time window:
bash .claude/skills/upstream-sync-check/run_sync.sh "6 months ago"
```

The script runs four stages in sequence and pipes results between them:

| Stage | What it does |
|-------|-------------|
| Phase 1 | Clones `otland/forgottenserver` into `forgottenserver-upstream/` if absent; fetches otherwise |
| Phase 2 | Lists commits + changed `.cpp`/`.h` files for the time window |
| Phase 3 | Maps changed files to C++ symbols via `cpp_symbol_manifest.json` |
| Phase 4a | Stream-parses `MIGRATION_LEDGER.yml` for each symbol (never loads the full 148k-line file) |
| Phase 4b | Checks `intentional_differences.yml` for each symbol |

**Stop conditions printed by the script:**
- `RESULT: No upstream changes` → nothing to migrate, stop here
- `RESULT: No .cpp/.h files changed` → stop here
- `NEW_FILE:` lines in Phase 3 output → files absent from manifest, flag for manual review before continuing

If the script itself fails (network error, missing manifest), stop and report the error. Do not proceed with stale or incomplete data.

---

## Phase 5 — Categorize each changed symbol

Build a summary table from Phase 3 + 4 output:

| Symbol | File | Ledger Status | intentional_differences? | Action |
|--------|------|---------------|--------------------------|--------|
| `Combat::doAreaCombat` | combat.cpp | DONE | no | skip |
| `LuaScriptInterface::lua_xyz` | luascript.cpp | PENDING | no | **migrate** |
| `g_config.getString` | configmanager.cpp | MISSING | yes (recorded) | skip |
| `NewClass::foo` | newfile.cpp | MISSING | no | **investigate** |

**Action rules:**
- `DONE` → skip (already migrated)
- `DONE` + in intentional_differences → skip (divergence recorded and accepted)
- `PENDING` or `PARTIAL` → **migrate**
- `MISSING` + in intentional_differences → skip (divergence recorded)
- `MISSING` + not in intentional_differences → **investigate**
- New file absent from manifest → **investigate**

---

## Phase 6 — Report findings with proof

### If nothing needs migration

State clearly for each skipped symbol:
```
SKIP Combat::doAreaCombat
  Reason: MIGRATION_LEDGER.yml status=DONE
  Evidence: <notes field from check-ledger output>

SKIP g_config.getString
  Reason: intentional_differences.yml entry dated YYYY-MM-DD
  Evidence: <reason field from check-intentional output>
```

End with: **"Upstream sync complete — no migration needed for this period."** Stop here.

### If migration or investigation is needed

List every symbol with its status and the upstream commit(s) that introduced the change. Proceed to Phase 7.

---

## Phase 7 — Write migration plan and enter Plan Mode

**Only if Phase 6 found symbols needing migration or investigation.**

1. Determine today's date and the short SHA of the most recent relevant upstream commit.

2. Write the migration plan to:
   ```
   docs/superpowers/plans/YYYY-MM-DD-upstream-sync-<short-sha>.md
   ```

3. The plan must contain:
   - Upstream commits being addressed (hash + message)
   - Symbols table (symbol, C++ file, ledger status, priority)
   - Per-symbol migration steps:
     1. Read only the specific function from `forgottenserver-upstream/src/<file>` (not the whole file)
     2. Write failing test (TDD — test first, always)
     3. Implement Rust code to pass the test
     4. Update `MIGRATION_LEDGER.yml` status to `DONE`
     5. If behavior intentionally differs, add entry to `intentional_differences.yml`
   - Verification commands:
     ```bash
     cargo test --lib --workspace
     python3 scripts/ledger/validate.py
     python3 scripts/ledger/cross_validate.py
     ```
   - Constraints (copy verbatim):
     - TDD always: failing test before any implementation code
     - Wire format is a hard contract — if any network opcode changes, flag immediately
     - Lua binding contract is strict — function name, arg order, return shape must match C++ exactly
     - No git commits without explicit user permission

4. Call `ExitPlanMode` so the user can review the plan before implementation begins.

---

## Common Mistakes

| Mistake | Correct approach |
|---------|-----------------|
| Opening full `.cpp` files to discover symbols | Run `map-symbols` — manifest only, never the source file |
| Assuming DONE means the upstream change is irrelevant | A DONE symbol can still have upstream behavior changes — always run the full pipeline |
| Skipping `check-ledger` because the symbol name looks done | Always query the ledger; name similarity is not evidence |
| Hard-coding changed files instead of reading `CHANGED_FILES:` output | Copy the JSON line exactly — manual entry introduces typos |
| Committing without user approval | Stop before any `git commit` and ask explicitly |
