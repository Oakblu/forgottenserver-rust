## ADDED Requirements

### Requirement: Node set is bootstrapped from the C++ symbol manifest

`bootstrap_nodes.py` SHALL seed graph nodes from `cpp_symbol_manifest.json` and SHALL NOT enumerate symbols by re-parsing the source. It adds nodes; it does not create a second symbol universe, and it preserves existing hand-authored nodes.

#### Scenario: Bootstrap creates a node per manifest symbol

- **WHEN** bootstrap runs against `cpp_symbol_manifest.json`
- **THEN** every manifest symbol has a node keyed by `{file, qualified_name}`
- **AND** no node exists whose key is absent from the manifest

#### Scenario: Boot-spine nodes are preserved

- **WHEN** bootstrap runs over a graph already containing phase-1 boot-spine nodes
- **THEN** those nodes and their curated edges remain intact

### Requirement: Static edges are extracted from the read-only C++ source

`build_edges.py` SHALL derive statically-resolvable call edges from `forgottenserver-upstream/src/`, tagging each `kind: static`, `confidence: static`, without modifying the source.

#### Scenario: Direct call produces a static edge

- **WHEN** a function body contains a directly-resolvable call to another manifest symbol
- **THEN** a `kind: static` out-edge to that symbol is recorded on the caller node

#### Scenario: Source tree is never modified

- **WHEN** any extraction or build command runs
- **THEN** no file under `forgottenserver-upstream/src/` is created, modified, or deleted

### Requirement: Rebuilds preserve curated dynamic edges and are idempotent

`make flow-build` SHALL recompute only static edges and merge them, preserving every `kind: dynamic` curated edge; a rebuild with no source change SHALL produce no diff.

#### Scenario: Curated edge survives a static rebuild

- **WHEN** `make flow-build` runs after a `kind: dynamic` edge was curated
- **THEN** that dynamic edge is still present and unchanged

#### Scenario: Rebuild is idempotent

- **WHEN** `make flow-build` runs twice with no source change
- **THEN** the second run produces no diff in the graph YAML

### Requirement: Build reports static-edge coverage

The build SHALL report static-edge coverage (functions with ≥1 resolved out-edge vs. total) so blind spots are visible.

#### Scenario: Coverage is reported

- **WHEN** `make flow-build` completes
- **THEN** it prints the count and percentage of functions with at least one resolved static out-edge
