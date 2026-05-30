## ADDED Requirements

### Requirement: Gap analysis joins C++ reachability with ledger and Rust manifest

`gap_analysis.py` SHALL traverse the graph from the root and, for each reachable node, compare its `MIGRATION_LEDGER.yml` status and presence in `rust_symbol_manifest.json` to determine whether the flow exists in the Rust port.

#### Scenario: Reachable but unported node is flagged

- **WHEN** a node is reachable from `main` and its ledger status is `PENDING`/`PARTIAL` (or it is absent from `rust_symbol_manifest.json`)
- **THEN** a `MISSING_FLOW` finding is recorded for it

#### Scenario: Reachable and DONE node is not flagged

- **WHEN** a node is reachable from `main` and its ledger status is `DONE` with a matching Rust symbol
- **THEN** no finding is recorded for it

### Requirement: Findings are classified

Each finding SHALL carry a category: `MISSING_FLOW`, `BRANCH_GAP`, `DYNAMIC_GAP`, or `ORDER_MISMATCH`.

#### Scenario: Unhandled dynamic dispatch is a DYNAMIC_GAP

- **WHEN** a `kind: dynamic` edge (opcode, event, or virtual override) has no corresponding Rust registration/handler/override symbol
- **THEN** a `DYNAMIC_GAP` finding is recorded

#### Scenario: Missing branch chain is a BRANCH_GAP

- **WHEN** a branch edge (`condition` set) leads to a chain of nodes all unported
- **THEN** a `BRANCH_GAP` finding identifying the guard and the missing chain head is recorded

#### Scenario: Unsatisfiable init order is an ORDER_MISMATCH

- **WHEN** boot/init nodes carry `order` values not reflected Rust-side
- **THEN** an `ORDER_MISMATCH` finding marked low-confidence is recorded

### Requirement: Recorded intentional differences are suppressed

A finding whose symbol appears in `intentional_differences.yml` SHALL be excluded from the actionable report and listed in a separate suppressed appendix.

#### Scenario: Intentional divergence is not actionable

- **WHEN** a node would be flagged `MISSING_FLOW` but its symbol is in `intentional_differences.yml`
- **THEN** it does not appear in the actionable findings and is listed under suppressed

### Requirement: Findings are prioritized

The report SHALL rank findings by a priority derived from reachability depth, ledger status, and node criticality, so boot-critical and shallow-path gaps surface first.

#### Scenario: Boot-critical gap outranks a deep rare-branch gap

- **WHEN** one finding is on the boot spine (shallow) and another on a deep conditional branch
- **THEN** the boot-spine finding ranks higher

### Requirement: Gap report and graph view are generated

`make flow-gap` SHALL regenerate `flow_graph/GAP_REPORT.md` (prioritized non-suppressed findings + suppressed appendix), and a generator SHALL produce `flow_graph/FLOW_GRAPH.md` fully from the YAML. Neither is hand-edited.

#### Scenario: Report regenerates from current state

- **WHEN** `make flow-gap` runs
- **THEN** `GAP_REPORT.md` reflects current edges, ledger statuses, and suppressions, each finding showing category, priority, `{file, qualified_name}`, ledger status, and source location

#### Scenario: No gaps yields an empty actionable report

- **WHEN** every reachable node is `DONE` or suppressed
- **THEN** `GAP_REPORT.md` reports zero actionable findings

### Requirement: CI guards graph consistency

A CI step SHALL run `make flow` (well-formedness, manifest consistency, and the phase 3–5 coverage checks) so the graph cannot drift from the source manifests.

#### Scenario: Drift fails CI

- **WHEN** the graph references a node key absent from `cpp_symbol_manifest.json`, or a dynamic coverage check fails
- **THEN** the CI `make flow` step exits non-zero
