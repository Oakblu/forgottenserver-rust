## Context

`main()` (`main.cpp`) → `startServer()` (`otserv.cpp:271`) starts `g_dispatcher`/`g_scheduler` and enqueues `mainLoader(services)` (`otserv.cpp:50`), which runs the real boot: config load, `Database::getInstance().connect()` (`:114`), world-type select (`:192`), `g_game.loadMainMap` (`:207`), `payHouses` (`:242`), `g_game.start(services)` (`:256`), state transitions. This small, fully-understood slice is the ideal seed to prove the format and validator before scaling up.

## Goals / Non-Goals

**Goals:**
- A diff-reviewable YAML graph format that survives an exhaustive build.
- A working `make flow` validator over a real (if tiny) graph.
- Cross-link every node to its ledger/manifest symbol via `{file, qualified_name}`.

**Non-Goals:**
- Automated extraction, dynamic edges, gap analysis, Markdown rendering of the full graph (later phases). The Markdown *view contract* is specified now; its generator ships in phase 6.

## Decisions

### D1: Sharded YAML source of truth, edges as out-edges on nodes
Nodes live in `flow_graph/nodes/<cpp_file>.yml`; `flow_graph/index.yml` holds roots + `unreached`. Edges are adjacency lists on the source node (no separate edge file to drift). Matches the ledger's YAML conventions; sharding keeps files reviewable.

### D2: Cross-link key is `{file, qualified_name}`
Identical to `cpp_symbol_manifest.json`/`MIGRATION_LEDGER.yml`, so no parallel symbol namespace is created.

### D3: Edge fields
`target` (key), `kind` (`static`|`dynamic`), `confidence` (`static`|`curated`), optional `order` (int, sequence within caller) and `condition` (branch guard text). The boot spine uses `confidence: curated` since it is hand-authored.

### D4: Validator scope for phase 1
`validate.py` checks: every node key resolves in `cpp_symbol_manifest.json`; no edge targets a missing key; every node is reachable from the root or listed in `unreached`. Exits non-zero with the offending item named.

## Risks / Trade-offs

- **Format churn later** → keeping phase 1 tiny means a format change here is cheap; lock it before phase 2 scales node count.
- **Boot spine hand-authoring errors** → cross-checked against `otserv.cpp` line numbers; the validator catches key/edge mistakes.
