## Why

The Rust port is tracked per-symbol (`MIGRATION_LEDGER.yml` + manifests), which proves a symbol *exists* but not that it is *reached from `main` the way C++ reaches it*. Runtime divergence hides in that gap. Before building an exhaustive flow graph we need a proven, validated format and a working validator demonstrated on the smallest real slice — the boot spine. This is phase 1 of 6; everything else builds on this contract.

## What Changes

- Define the flow-graph YAML schema (nodes = C++ symbols by `{file, qualified_name}`; edges = out-edges with `kind`, `confidence`, optional `order`/`condition`) and document it in `flow_graph/README.md`.
- Create `flow_graph/index.yml` declaring `main` as the sole root, the entrypoint chain `main` → `startServer` → `mainLoader`, and an `unreached` convention.
- Hand-author the boot-spine nodes/edges for `main.cpp` and `otserv.cpp` only (config load, `Database::connect`, world-type switch, `loadMainMap`, `payHouses`, `g_game.start`, state transitions) as the seed.
- Add `scripts/flow/validate.py` + `make flow` that fails on malformed graphs, dangling edges, unknown node keys, and unreachable orphans.
- **Non-goal:** any automated extraction, dynamic edges, or gap analysis (later phases).

## Capabilities

### New Capabilities
- `flow-graph-format`: The flow-graph data contract — schema for nodes/edges, root/entrypoint declaration, the `{file, qualified_name}` cross-link to `MIGRATION_LEDGER.yml`, sharded-YAML storage with a generated Markdown view, and the well-formedness validator.

### Modified Capabilities
<!-- None. -->

## Impact

- New files: `flow_graph/README.md`, `flow_graph/index.yml`, `flow_graph/nodes/main.yml`, `flow_graph/nodes/otserv.yml`, `scripts/flow/validate.py`, `make flow` target.
- Reads, never modifies: `forgottenserver-upstream/src/main.cpp` + `otserv.cpp`, `cpp_symbol_manifest.json`.
- Pure-additive; no Rust or C++ source changes.
- **Depends on:** nothing. **Unlocks:** phase 2 (extraction).
