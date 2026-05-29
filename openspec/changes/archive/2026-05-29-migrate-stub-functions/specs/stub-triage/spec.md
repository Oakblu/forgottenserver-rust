## ADDED Requirements

### Requirement: Every stub-report entry is classified with C++ evidence
Each of the 95 entries in `scripts/stub_report.json` SHALL have a corresponding classification entry in `scripts/stub_triage.json`. Classification SHALL be one of: `correct-default`, `intentional-deferred`, `needs-implementation`, or `panic-correct`. Each entry SHALL include `cpp_evidence` (upstream file:line) and `cpp_snippet` (the C++ source line that proves the classification).

#### Scenario: Correct-default stub is classified with evidence
- **WHEN** a Rust function returns a literal value (e.g. `true`, `false`, `0`, `Ok(())`) and the corresponding C++ virtual method has an identical inline or virtual default
- **THEN** the triage entry SHALL have `"classification": "correct-default"` and `cpp_evidence` pointing to the C++ header line, and a `test_name` referencing the confirming test

#### Scenario: Intentional-deferred stub is classified with diff ID
- **WHEN** a Rust function is empty or returns a no-op because the real behavior requires cross-crate dispatch (spectator updates, DB writes, Lua calls) that is tracked in `intentional_differences.yml`
- **THEN** the triage entry SHALL have `"classification": "intentional-deferred"` and `intentional_diff_id` matching an entry in `intentional_differences.yml`

#### Scenario: Needs-implementation stub is classified for remediation
- **WHEN** a Rust function returns a literal or drops work but the C++ source has non-trivial logic that is not covered by any intentional-difference entry
- **THEN** the triage entry SHALL have `"classification": "needs-implementation"` and the corresponding implementation task SHALL be created

#### Scenario: Panic-correct stub is classified as type-safe assertion
- **WHEN** a Rust function uses `panic!` to enforce a type invariant (e.g. `LuaVariant::get_number` panics when called on a non-Number variant) and the C++ equivalent would cause undefined behavior or return garbage
- **THEN** the triage entry SHALL have `"classification": "panic-correct"` and `cpp_evidence` pointing to the C++ accessor that has the same pre-condition

### Requirement: Triage file is machine-readable JSON
`scripts/stub_triage.json` SHALL be a valid JSON array. Each element SHALL have the fields: `fn_name` (string), `file` (string), `line` (integer), `classification` (enum string), `cpp_evidence` (string), `cpp_snippet` (string). `correct-default` entries SHALL also have `test_name`. `intentional-deferred` entries SHALL also have `intentional_diff_id`.

#### Scenario: Triage file is valid JSON
- **WHEN** `python3 -m json.tool scripts/stub_triage.json` is run
- **THEN** it SHALL exit 0 and print formatted JSON without errors

#### Scenario: All 95 stub-report entries have triage entries
- **WHEN** `python3 scripts/find_stubs.py` produces a report and the triage file exists
- **THEN** every `(file, fn_name, line)` triple in `stub_report.json` SHALL appear in `stub_triage.json`

### Requirement: Each correct-default stub has a confirming unit test
For every entry classified `correct-default`, a Rust unit test SHALL exist in the same module asserting the exact return value and documenting the C++ source line in a comment. The test name SHALL match the `test_name` field in the triage entry.

#### Scenario: Confirming test asserts C++ default value
- **WHEN** a stub is classified `correct-default` (e.g. `is_attackable` returns `true`)
- **THEN** the test SHALL call the function on a fresh instance and assert the expected literal value

#### Scenario: Confirming test documents C++ evidence
- **WHEN** the confirming test is read
- **THEN** it SHALL contain a comment with the C++ file:line evidence (e.g. `// C++: creature.h:269 virtual bool isAttackable() const { return true; }`)

### Requirement: Each needs-implementation stub is replaced with real behavior
For every entry classified `needs-implementation`, the trivial body SHALL be replaced with a full Rust implementation that matches the C++ behavior, preceded by a failing test that asserts the correct behavior.

#### Scenario: Implementation matches C++ logic
- **WHEN** the C++ function has non-trivial logic (branching, state access, error conditions)
- **THEN** the Rust replacement SHALL handle all C++ branches and return the same result for equivalent inputs

#### Scenario: All tests pass after implementation
- **WHEN** `cargo test --lib -p <crate>` is run after replacing a stub
- **THEN** all tests SHALL pass with zero failures
