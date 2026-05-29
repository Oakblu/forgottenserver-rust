## Context

The completed graph encodes, for every C++ flow reachable from `main`, the symbols involved and how control reaches them (static + dynamic edges). Joining that reachability with per-symbol ledger status and Rust manifest presence yields exactly the question the whole effort set out to answer: *which C++ paths are missing or mis-wired in the Rust port?* This phase implements that join and presents it.

## Goals / Non-Goals

**Goals:**
- A prioritized, classified, suppressible gap report consumable by humans and AI.
- A generated navigable Markdown view of the graph.
- A CI guard preventing graph/manifest drift.

**Non-Goals:**
- Implementing the Rust fixes (downstream changes seeded from the report).
- A symmetric Rust-side reachability graph (future work); v1 compares C++ reachability against ledger status + Rust symbol presence.

## Decisions

### D1: Analysis direction = C++ reachability × ledger status × Rust presence
Traverse from the root; for each reachable node resolve its `MIGRATION_LEDGER.yml` status and look it up in `rust_symbol_manifest.json`. This answers "C++ path present, Rust counterpart absent/incomplete." The reverse ("Rust symbol exists but unwired") needs a Rust graph and is out of scope.

### D2: Finding classification
- `MISSING_FLOW` — reachable node, ledger `PENDING`/`PARTIAL` or absent from the Rust manifest.
- `BRANCH_GAP` — a branch edge (`condition` set) whose target chain is entirely missing.
- `DYNAMIC_GAP` — a `kind: dynamic` edge (opcode/event/virtual) with no corresponding Rust registration/handler/override symbol.
- `ORDER_MISMATCH` — boot/init nodes whose `order` cannot be satisfied Rust-side; emitted low-confidence for review.

### D3: Suppression via `intentional_differences.yml`
A finding whose symbol is recorded there is excluded from the actionable report (listed separately as "suppressed" for transparency).

### D4: Priority formula
`priority = w_depth · (1 / (1 + depth_from_main)) + w_status · status_weight + w_crit · criticality`, where shallower depth, more-incomplete status, and higher node criticality (boot-spine and dynamic entrypoints rank highest) raise priority. Weights are tunable (O4); the first run uses sensible defaults and is reviewed.

### D5: Outputs
`GAP_REPORT.md` lists non-suppressed findings sorted by priority, each with category, priority, `{file, qualified_name}`, ledger status, and source location, plus a suppressed-findings appendix. `FLOW_GRAPH.md` is generated from the YAML: boot sequence as an ordered list, per-subsystem node tables linking to ledger status. Both are regenerated, never hand-edited.

### D6: CI guard
A CI step runs `make flow` (well-formedness + manifest consistency, incl. the phase 3–5 coverage checks) so the graph cannot silently drift, mirroring `make ledger`.

## Risks / Trade-offs

- **False positives from unrecorded intentional divergence** → suppression list; spot-check top findings and record any legitimate divergence in `intentional_differences.yml`.
- **Priority weights need tuning (O4)** → defaults first, adjust after reviewing the first real report.
- **`ORDER_MISMATCH` is heuristic** → low-confidence, review-only; never a hard CI failure.
- **Report size** → sorted + categorized so the actionable top is usable even if the long tail is large.
