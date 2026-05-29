## ADDED Requirements

### Requirement: Flow graph is rooted at the C++ entry point

The flow graph SHALL declare `main` (`forgottenserver-upstream/src/main.cpp`) as its sole root in `flow_graph/index.yml` and SHALL record the entrypoint chain `main` → `startServer` → `mainLoader`.

#### Scenario: Root is declared in the index

- **WHEN** `flow_graph/index.yml` is loaded
- **THEN** it declares exactly one root node identifying `main` by `{file, qualified_name}`
- **AND** it records the ordered entrypoint chain `main` → `startServer` → `mainLoader`

#### Scenario: Every node is reachable or explicitly unreached

- **WHEN** the graph is traversed from the root following out-edges
- **THEN** every node is reachable, OR is listed in the `unreached` section with a reason

### Requirement: Nodes reuse the C++ symbol identity

Each node SHALL identify its symbol by the `{file, qualified_name}` key used by `cpp_symbol_manifest.json` and `MIGRATION_LEDGER.yml`; the graph SHALL NOT create a parallel symbol namespace.

#### Scenario: Node key resolves to a manifest symbol

- **WHEN** any node's `{file, qualified_name}` key is looked up in `cpp_symbol_manifest.json`
- **THEN** exactly one matching symbol exists

### Requirement: Edges encode kind, confidence, order, and branch condition

Edges SHALL be stored as out-edges on the source node, each carrying `target`, `kind` (`static`|`dynamic`), `confidence` (`static`|`curated`), and OPTIONAL `order` and `condition`.

#### Scenario: Sequential boot steps preserve order

- **WHEN** `mainLoader` calls config load, `Database::connect`, and `loadMainMap` in sequence
- **THEN** each out-edge carries a distinct `order` reflecting the C++ call sequence

#### Scenario: Conditional call records its guard

- **WHEN** a call occurs only inside a branch (e.g. the world-type switch)
- **THEN** the edge carries a `condition` field describing the guard

### Requirement: Graph source is sharded YAML with a generated Markdown view contract

The authoritative graph SHALL be YAML sharded per C++ source file under `flow_graph/nodes/<cpp_file>.yml` plus `flow_graph/index.yml`. The generated `flow_graph/FLOW_GRAPH.md` (produced in a later phase) MUST be fully derived from the YAML and MUST NOT be hand-edited.

#### Scenario: One shard per C++ source file

- **WHEN** nodes exist for symbols defined in `otserv.cpp`
- **THEN** those nodes live in `flow_graph/nodes/otserv.yml`

### Requirement: Validator enforces well-formedness

`make flow` SHALL run a validator that fails (non-zero, naming the offender) on unknown node keys, dangling edges, or orphan nodes not listed in `unreached`, and passes on a well-formed graph.

#### Scenario: Dangling edge fails

- **WHEN** an edge targets a key no node/manifest symbol has
- **THEN** `make flow` exits non-zero and names the offending edge

#### Scenario: Orphan node fails

- **WHEN** a node is unreachable from the root and not listed in `unreached`
- **THEN** `make flow` exits non-zero and names the orphan

#### Scenario: Clean boot-spine graph passes

- **WHEN** the seeded boot-spine graph is well-formed and every key resolves
- **THEN** `make flow` exits zero
