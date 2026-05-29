## ADDED Requirements

### Requirement: docs/performance/README.md exists with a comparison table
`docs/performance/README.md` SHALL exist and contain a Markdown table with one row per perf-bot scenario showing C++ and Rust side-by-side columns for: actions/sec (sustained), login p50/p99 ms, error rate %, and RSS at T=0/T=end MB. The table SHALL render correctly on GitHub without a live server.

#### Scenario: README renders on GitHub
- **WHEN** `docs/performance/README.md` is viewed on GitHub
- **THEN** a Markdown table is visible with C++ and Rust columns and numeric data

#### Scenario: README covers all perf-bot scenarios
- **WHEN** the README table is inspected
- **THEN** rows exist for: login_flood, sustained_load, crowd_walk, crowd_chat, crowd_combat, magic_storm, npc_flood, monster_swarm, full_chaos

### Requirement: docs/performance/perf-results.json is the data source for the README table
A committed `docs/performance/perf-results.json` file SHALL store the reference benchmark results as structured JSON. The README table SHALL be derived from this file. The JSON SHALL follow the schema produced by `perf-bot --output <path>` (a `ComparisonReport` object with `rust` and `cpp` fields, each containing scenario metrics).

#### Scenario: perf-results.json is valid JSON
- **WHEN** `jq . docs/performance/perf-results.json` is run
- **THEN** it exits 0 and outputs valid JSON

#### Scenario: perf-results.json contains both cpp and rust fields
- **WHEN** the file is parsed
- **THEN** top-level keys `rust` and `cpp` are present, each with scenario-level metric objects

### Requirement: scripts/update-perf-readme.sh regenerates README from a fresh run
`scripts/update-perf-readme.sh` SHALL: (1) run perf-bot with `--target both --scenario full_chaos` (or all scenarios) and `--output docs/performance/perf-results.json`, (2) regenerate `docs/performance/README.md` from the updated JSON. The script SHALL print instructions for committing the updated files.

#### Scenario: Script produces updated JSON and README
- **WHEN** `bash scripts/update-perf-readme.sh` is run with the perf stack running
- **THEN** `docs/performance/perf-results.json` and `docs/performance/README.md` are both updated with fresh numbers

#### Scenario: Script is self-documenting on missing stack
- **WHEN** the script is run without the perf Docker stack running
- **THEN** it prints an error explaining the perf stack must be started first and exits non-zero

### Requirement: Project README links to the performance comparison
The root `README.md` SHALL contain a link to `docs/performance/README.md` under a "Performance" section, so users can find the comparison from the project landing page.

#### Scenario: Root README has performance link
- **WHEN** `README.md` is viewed
- **THEN** a "Performance" section or link pointing to `docs/performance/README.md` is present
