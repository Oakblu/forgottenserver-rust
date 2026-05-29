## Why

`scripts/stub_report.json` flags 95 Rust functions as trivial/empty/panic/dropped-work stubs, but a large fraction of them are actually **correct implementations** of C++ virtual defaults that the static tool cannot distinguish from real stubs. The remaining genuine stubs represent missing cross-crate behavior and unconfirmed correctness. Without a systematic triage pass the codebase violates the "no stub functions" rule even though most bodies are already right, and the migration ledger cannot be marked DONE for affected files.

## What Changes

- **Triage all 95 flagged stubs** against C++ source and `intentional_differences.yml`, producing a classification for each: `correct-default`, `intentional-deferred`, `needs-implementation`, or `panic-correct`.
- **Add confirming tests** for every `correct-default` stub (asserting the literal return value matches C++ virtual default documentation) so they are no longer "undocumented correct returns."
- **Implement real behavior** for any stub classified `needs-implementation` using TDD: failing test first, then the implementation.
- **Record new intentional differences** for any cross-crate deferred stub not yet in `intentional_differences.yml`.
- **Extend `find_stubs.py`** with a `confirmed_stubs.json` allowlist that excludes proven-correct trivial bodies from future reports, reducing noise without hiding real gaps.
- **Update `MIGRATION_LEDGER.yml`** for all affected files once stubs are resolved.

## Capabilities

### New Capabilities

- `stub-triage`: Systematic classification of all 95 stub-report entries against C++ source, producing a `stub_triage.json` evidence file alongside `stub_report.json`.
- `confirmed-stubs-allowlist`: A `scripts/confirmed_stubs.json` allowlist that `find_stubs.py` consults to skip proven-correct trivial bodies, keeping the stub report focused on real gaps.

### Modified Capabilities

- `stub-detection`: `find_stubs.py` gains an `--allowlist` flag; confirmed stubs are excluded from output and the report separates `unresolved` from `confirmed-correct`.

## Impact

- **Crates modified**: `common`, `items`, `map`, `entity`, `database`, `scripting`, `world`, `server`, `game` (tests added or implementations replaced in each).
- **Scripts modified**: `scripts/find_stubs.py` (allowlist support).
- **New files**: `scripts/confirmed_stubs.json`, `scripts/stub_triage.json`.
- **Docs updated**: `intentional_differences.yml` (new entries for any unrecorded cross-crate deferrals), `MIGRATION_LEDGER.yml` (status upgrades for newly-resolved files).
- **No breaking API changes** — all fixes preserve existing public function signatures; only bodies change.
