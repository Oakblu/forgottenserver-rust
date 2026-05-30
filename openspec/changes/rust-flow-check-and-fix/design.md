## Context

The flow graph (142 reachable nodes from `main()`) and `MIGRATION_LEDGER.yml` (all 6194 C++ symbols covered) provide a complete picture of what must be implemented. The gap analysis (`make flow-gap`) reports 0 actionable findings — meaning every reachable node is claimed migrated with a Rust body hash. However, a non-empty body hash only proves Rust code was written; it does not prove the Rust behavior matches the C++ spec.

This change works through every reachable node in BFS order (depth 0 → depth 6), reads the C++ source for each, checks the corresponding Rust implementation, writes a failing test if one is missing, and fixes any incorrect or stub-level implementation.

## Goals / Non-Goals

**Goals:**
- For every reachable flow-graph node: at least one unit test that exercises the C++ behavior.
- Fix any stub (`todo!()`, `unimplemented!()`, empty/default return) found during verification.
- Record any confirmed intentional behavioral difference in `intentional_differences.yml`.
- Keep `cargo test --lib --workspace` green throughout; each fix is a self-contained passing task.

**Non-Goals:**
- Full 100% line coverage for every symbol in the manifest (only reachable flow nodes).
- Performance optimizations, refactors, or API changes.
- Editing `forgottenserver-upstream/src/` (read-only spec).
- E2E tests (Docker / DB required — out of scope for unit verification).

## Decisions

**D1 — TDD per node.** Each task: read C++ → write failing test → implement fix → green. Never implement without a test first. Rationale: the only way to distinguish "claimed migrated" from "behaviorally correct" is a test that would fail if the behavior were wrong.

**D2 — BFS depth order.** Work depth 0 → 6. Boot / init nodes at depth 0-2 are dependencies of everything else; verifying them first establishes a stable foundation. Rationale: if a deeper node shares a helper with a shallower one already verified, the helper is already tested.

**D3 — One task per reachable node.** Tasks are maximally fine-grained: one node = one task = one `- [ ]` in `tasks.md`. Rationale: enables parallel subagent dispatch and clear progress tracking.

**D4 — Use manifests, never load full `.cpp` files raw.** Use `cpp_symbol_manifest.json` to locate the file/line range of each symbol, then read only that range. Rationale: avoids context-window saturation on large files (CLAUDE.md §Migration Rules §3).

**D5 — Skip `intentionally_removed` ledger entries.** If a node's ledger status is `intentionally_removed`, record a one-line note and mark the task done with no implementation work. Rationale: the flow graph may include nodes for C++ paths that are explicitly not ported.

**D6 — Group tasks into subsystem blocks.** tasks.md organizes nodes into five groups: Boot/Init, Database/Storage, Game/State, Network/Protocol, Opcode Handlers, Events/Scheduler, Supporting Services. Rationale: related nodes share test infrastructure; grouping reduces context switching.

## Risks / Trade-offs

- **Large C++ files**: `protocolgame.cpp` (~8000 lines) holds 50+ opcode handlers. Reading even a focused slice is expensive. Mitigation: use manifest to get exact line ranges; read only the relevant 50-200 lines per handler.
- **Stateful boot chain**: depth 0-3 nodes involve global singletons (`ConfigManager`, `Database`, `Game`). Unit tests must isolate state. Mitigation: use mock/in-memory variants already present in the codebase; check for existing test fixtures.
- **50 opcode handlers**: if most handlers already have tests, tasks are lightweight. If many are stubs, this is significant work. Mitigation: scan for `todo!()` across network crate before writing tasks; adjust scope.
- **intentional_differences.yml churn**: verification may uncover many legitimate divergences (Rust idioms, removed legacy features). Mitigation: record each clearly; do not treat documented differences as bugs.

## Migration Plan

1. Run through tasks in BFS depth order within each group.
2. For each task: read C++ (manifest-located), read Rust implementation, determine if a test covers the behavior, write failing test if not, fix implementation if needed.
3. After each task: `cargo test --lib -p <crate> <test_name>` passes. Periodically run `cargo test --lib --workspace`.
4. At completion: `make flow-gap` still reports 0 findings (nothing regressed), `cargo clippy` clean.

## Open Questions

- How many of the 50 opcode handlers already have per-handler unit tests vs. only integration coverage? (Scan first, then write tasks.)
- Are any depth-5/6 nodes for opcodes that are intentionally removed from the Rust port? (Check ledger before writing those tasks.)
