## Context

`scripts/find_stubs.py` scans all Rust source files for functions with trivial bodies (literal returns, empty bodies, `drop()` calls, or `panic!` stubs) and produces `scripts/stub_report.json`. The current report flags **95 functions** across 9 crates. However, static pattern-matching cannot distinguish:

- A **correct C++ virtual default** (e.g., `is_attackable() { true }` matching C++ `virtual bool isAttackable() const { return true; }`)
- A **cross-crate deferred behavior** intentionally modeled as a no-op pending the game-glue layer
- A **genuine stub** that silently returns the wrong value

All three look identical in the AST. Without evidence-based classification, the "no stubs" rule cannot be enforced rigorously and migration ledger status cannot advance.

The five C++ reference files (`MIGRATION_LEDGER.yml`, `rust_symbol_manifest.json`, `cpp_symbol_manifest.json`, `AI_MIGRATION_CONTEXT.md`, `intentional_differences.yml`) and `forgottenserver-upstream/src/` provide the evidence needed to classify every stub.

## Goals / Non-Goals

**Goals:**
- Classify all 95 stub-report entries with evidence: `correct-default`, `intentional-deferred`, `needs-implementation`, or `panic-correct`.
- Add a unit test for every `correct-default` entry asserting the return value matches C++ behavior, eliminating "undocumented correct trivial body" as a category.
- Implement real behavior (TDD) for any `needs-implementation` entry.
- Record any previously unrecorded `intentional-deferred` entries in `intentional_differences.yml`.
- Add a `confirmed_stubs.json` allowlist to `find_stubs.py` so confirmed-correct stubs are excluded from future reports.
- Advance `MIGRATION_LEDGER.yml` status for files fully resolved by this change.

**Non-Goals:**
- Implementing cross-crate game-glue behaviors (spectator broadcasts, DB writes, Lua dispatch) — those are tracked in `intentional_differences.yml` and remain deferred.
- Migrating any C++ symbol not already in the stub report.
- Changing public function signatures.
- Modifying the `forgottenserver-upstream/` tree.

## Decisions

### Decision 1: Triage order — crate-by-crate, smallest first

Triage one crate at a time, starting with `game` (1 stub) and `server` (1 stub) before tackling `common` (37 stubs). This front-loads easy wins, validates the classification workflow, and keeps each PR atomic.

**Alternatives considered:**
- File-by-file alphabetical: no advantage, harder to reason about crate-level completeness.
- Risk-first (high-risk manifest entries first): risk score from the manifest is a secondary factor, not the primary ordering.

### Decision 2: Evidence file `stub_triage.json`

Produce `scripts/stub_triage.json` alongside the existing `stub_report.json`. Each entry in the triage file carries `fn_name`, `file`, `classification`, `cpp_evidence` (file:line from upstream), and `test_name` (for `correct-default`) or `intentional_diff_id` (for `intentional-deferred`).

**Format:**
```json
{
  "fn_name": "is_attackable",
  "file": "entity/src/creature.rs",
  "line": 816,
  "classification": "correct-default",
  "cpp_evidence": "forgottenserver-upstream/src/creature.h:269",
  "cpp_snippet": "virtual bool isAttackable() const { return true; }",
  "test_name": "test_creature_is_attackable_returns_true_matching_cpp_default"
}
```

**Alternatives considered:**
- Inline comments in Rust source: mixes concern of implementation with triage audit; triage file is machine-readable and can be regenerated independently.

### Decision 3: `confirmed_stubs.json` allowlist in `find_stubs.py`

After triage, `correct-default` and `intentional-deferred` entries are written to `scripts/confirmed_stubs.json`. `find_stubs.py` loads this file at runtime and skips any `(file, fn_name, line)` triple present in it. The report's root object gains `"unresolved"` and `"confirmed"` arrays.

**Alternatives considered:**
- `#[allow(stub)]` attribute in Rust source: would require a custom lint; adds noise to production code.
- Suppressing trivial_body pattern entirely: hides future real stubs.

### Decision 4: TDD for `needs-implementation` entries

For each genuine stub, write the failing test first (asserting C++ behavior), then implement. Implementation may add parameters to internal helpers but must not change the public signature. If the correct implementation requires cross-crate dispatch that does not yet exist, reclassify as `intentional-deferred` and document in `intentional_differences.yml` instead.

### Decision 5: Classification of the 95 stubs (pre-determined by research)

Based on analysis of `forgottenserver-upstream/src/` against the current Rust implementations:

| Pattern | Count | Likely Classification |
|---|---|---|
| Thing/Cylinder trait defaults | ~25 | `correct-default` (match C++ virtual defaults) |
| Container identity methods (is_container, is_item, is_removed, can_remove) | ~18 | `correct-default` (match C++ inline header overrides) |
| Cross-crate post-notification hooks | ~8 | `intentional-deferred` |
| Database transaction no-ops | 3 | `intentional-deferred` (MariaDB adapter) |
| Creature visibility/attackable defaults | 3 | `correct-default` |
| LuaVariant type-checked accessors | 4 | `panic-correct` |
| Scripting engine methods | 3 | `correct-default` (NoopScriptEngine) / `intentional-deferred` |
| Wand get_element_damage | 1 | `correct-default` (C++ returns 0 inline) |
| HTTP accept_loop (dropped_work) | 1 | `intentional-deferred` |
| u8_to_state panic | 1 | `panic-correct` |
| HouseTile query stubs | 4 | `intentional-deferred` |
| TrashHolder/Mailbox/StoreInbox stubs | ~5 | `correct-default` |
| Tile post_add/remove | 2 | `intentional-deferred` |

## Risks / Trade-offs

- **False-negative risk**: A stub classified as `correct-default` may have a subtle behavior difference not visible in the trivial body. Mitigation: the confirming test explicitly documents the C++ source line and asserts the expected value; the test is the contract.
- **Allowlist staleness**: If a confirmed stub is later found incorrect, the allowlist entry must be removed. Mitigation: `confirmed_stubs.json` entries include the `body_hash` from `rust_symbol_manifest.json`; `find_stubs.py` treats a hash mismatch as a warning that the entry needs re-triage.
- **Scope creep**: Triage may surface additional context that suggests a stub needs a more complete implementation, expanding scope. Mitigation: the spec specifies that if cross-crate dispatch is required, the entry is reclassified `intentional-deferred`, not left as a stub and not partially implemented.

## Migration Plan

1. Run `python3 scripts/find_stubs.py` at the start of each crate phase to confirm the baseline.
2. For each crate, classify stubs → write/update tests → fix any `needs-implementation` entries.
3. Add confirmed entries to `confirmed_stubs.json` after each crate phase.
4. Run `python3 scripts/find_stubs.py` again after each phase; verify the unresolved count drops.
5. After all crates, run `cargo test --lib --workspace` and `cargo clippy` to confirm clean.
6. Update `MIGRATION_LEDGER.yml` for resolved files.

## Open Questions

- None blocking. The classification rationale for each of the 95 stubs is deterministic once the C++ source is checked against the Rust body.
