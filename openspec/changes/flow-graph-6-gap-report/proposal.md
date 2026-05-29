## Why

After phases 1–5 the flow graph is complete: rooted at `main`, every node, static edges, and the dynamic edges (network, events/scheduler, virtual dispatch). This phase turns it into the actual deliverable — a prioritized, actionable report of which C++ flows are missing or mis-wired in the Rust port — plus a navigable Markdown view and a CI guard so the graph cannot rot. This is the payoff phase that makes the whole effort production-useful. Phase 6 of 6.

## What Changes

- Add `scripts/flow/gap_analysis.py`: traverse from the root and join each reachable node against `MIGRATION_LEDGER.yml` status and `rust_symbol_manifest.json` presence.
- Classify findings: `MISSING_FLOW`, `BRANCH_GAP`, `DYNAMIC_GAP`, `ORDER_MISMATCH`.
- Suppress findings recorded in `intentional_differences.yml`.
- Rank by `priority = f(reachability_depth, ledger_status, node_criticality)` and emit `flow_graph/GAP_REPORT.md` via `make flow-gap`.
- Add `scripts/flow/render_markdown.py` → `flow_graph/FLOW_GRAPH.md` (generated, navigable view).
- Add a CI step running `make flow` so the graph stays consistent with the manifests, analogous to `make ledger`.
- **Non-goal:** implementing the Rust fixes the report surfaces — those are separate follow-up changes seeded from `GAP_REPORT.md`.

## Capabilities

### New Capabilities
- `flow-graph-gap-report`: The gap-analysis engine and outputs — classification, intentional-difference suppression, prioritization, the generated `GAP_REPORT.md` and `FLOW_GRAPH.md`, and the CI consistency guard.

### Modified Capabilities
<!-- None. Consumes the graph built in phases 1–5. -->

## Impact

- New files: `scripts/flow/gap_analysis.py`, `scripts/flow/render_markdown.py`, `flow_graph/GAP_REPORT.md`, `flow_graph/FLOW_GRAPH.md`, `make flow-gap` target, a CI step.
- Reads: the full graph, `MIGRATION_LEDGER.yml`, `rust_symbol_manifest.json`, `intentional_differences.yml`.
- Pure-additive; no Rust or C++ source changes. Fixes are downstream changes.
- **Depends on:** phases 1–5.
