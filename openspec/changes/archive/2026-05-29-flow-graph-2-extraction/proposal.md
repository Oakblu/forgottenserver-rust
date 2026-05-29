## Why

With the format and validator proven on the boot spine (phase 1), the bulk of the graph — every symbol and every statically-resolvable call edge — should be produced by **tooling**, not by hand. Automating this keeps later AI sessions small: the curation phases then only handle the dynamic edges static analysis cannot see. This is phase 2 of 6.

## What Changes

- Add `scripts/flow/bootstrap_nodes.py` to seed one node per `cpp_symbol_manifest.json` symbol into `flow_graph/nodes/<cpp_file>.yml` (keyed by `{file, qualified_name}`), never re-parsing to discover symbols.
- Resolve Open Question O1 (libclang vs. tree-sitter/heuristic) and record the choice in `flow_graph/README.md`.
- Add `scripts/flow/build_edges.py` to derive `kind: static`, `confidence: static` call edges from the read-only `forgottenserver-upstream/src/`, attaching them as out-edges, while **preserving** any existing `kind: dynamic` curated edges across rebuilds.
- Add `make flow-build` (refresh static edges) mirroring the ledger build targets.
- **Non-goal:** curating dynamic edges (phases 3–5) and gap analysis (phase 6).

## Capabilities

### New Capabilities
- `flow-graph-extraction`: Tooling that bootstraps nodes from the C++ symbol manifest and derives statically-resolvable call edges from the read-only source, preserving curated dynamic edges and exposing `make flow-build`.

### Modified Capabilities
<!-- None. Builds on flow-graph-format from phase 1. -->

## Impact

- New files: `scripts/flow/bootstrap_nodes.py`, `scripts/flow/build_edges.py`, `make flow-build` target; expanded `flow_graph/nodes/*.yml`.
- Reads, never modifies: `forgottenserver-upstream/src/`, `cpp_symbol_manifest.json`.
- Pure-additive; no Rust or C++ source changes.
- **Depends on:** phase 1 (format + validator). **Unlocks:** phases 3–5 (dynamic edges) and phase 6 (gap report).
