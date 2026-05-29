## ADDED Requirements

### Requirement: Confirmed stubs are recorded in a persistent allowlist
`scripts/confirmed_stubs.json` SHALL exist after this change and SHALL contain one entry per `correct-default`, `intentional-deferred`, or `panic-correct` stub from the triage file. Each entry SHALL have: `fn_name`, `file`, `line`, and `body_hash` (SHA-256 of the function body, taken from `rust_symbol_manifest.json` or computed inline).

#### Scenario: Allowlist entry is written for each confirmed stub
- **WHEN** a stub is classified `correct-default`, `intentional-deferred`, or `panic-correct` in `stub_triage.json`
- **THEN** a matching entry SHALL appear in `confirmed_stubs.json` with the correct `body_hash`

#### Scenario: Allowlist is a valid JSON array
- **WHEN** `python3 -m json.tool scripts/confirmed_stubs.json` is run
- **THEN** it SHALL exit 0 without errors

### Requirement: Body-hash mismatch triggers a warning
If a confirmed stub's function body is later changed (its hash no longer matches the allowlist entry), `find_stubs.py` SHALL emit a warning line on stderr of the form `WARN: confirmed stub body changed: <file>:<fn_name> (line <n>)` and SHALL include the entry in the `"unresolved"` array of the report rather than the `"confirmed"` array.

#### Scenario: Hash mismatch is surfaced as unresolved
- **WHEN** a function in `confirmed_stubs.json` has its body modified after confirmation
- **THEN** `find_stubs.py` SHALL treat it as unresolved and warn on stderr

#### Scenario: Hash match keeps entry in confirmed array
- **WHEN** a function body is unchanged from when it was confirmed
- **THEN** `find_stubs.py` SHALL keep it in the `"confirmed"` array and NOT include it in `"unresolved"`

### Requirement: Confirmed stubs are excluded from the unresolved count
`find_stubs.py` output SHALL distinguish `"unresolved"` stubs (need attention) from `"confirmed"` stubs (verified correct or deferred). The `"unresolved"` array SHALL be the primary metric for migration progress.

#### Scenario: Report separates unresolved and confirmed
- **WHEN** `find_stubs.py` runs after `confirmed_stubs.json` is populated
- **THEN** the JSON output SHALL have both `"unresolved": [...]` and `"confirmed": [...]` arrays at the root

#### Scenario: Unresolved count is 0 after all stubs are classified
- **WHEN** every stub in the report has a matching confirmed-stubs entry with a valid hash
- **THEN** `find_stubs.py` SHALL output `"unresolved": []`
