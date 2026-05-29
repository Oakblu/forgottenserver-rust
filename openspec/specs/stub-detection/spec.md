## MODIFIED Requirements

### Requirement: find_stubs.py produces a structured JSON report with allowlist support
`find_stubs.py` SHALL load `scripts/confirmed_stubs.json` (if present) and use it to partition detected stubs into `"unresolved"` and `"confirmed"` arrays in its output. The script SHALL accept an optional `--allowlist <path>` flag to override the default allowlist path. When the allowlist is absent, all detected stubs go to `"unresolved"`.

#### Scenario: Script runs without an allowlist and outputs all stubs as unresolved
- **WHEN** `confirmed_stubs.json` does not exist and `find_stubs.py` is run
- **THEN** the JSON output SHALL have `"unresolved"` containing all detected stubs and `"confirmed": []`

#### Scenario: Script loads the allowlist and partitions stubs correctly
- **WHEN** `confirmed_stubs.json` exists with valid entries
- **THEN** stubs matching a confirmed entry (same file, fn_name, line, and body_hash) SHALL appear in `"confirmed"` and NOT in `"unresolved"`

#### Scenario: Hash mismatch moves confirmed entry back to unresolved with a warning
- **WHEN** a stub's body has changed since it was added to the allowlist (hash mismatch)
- **THEN** the stub SHALL appear in `"unresolved"` and a warning line SHALL be printed to stderr: `WARN: confirmed stub body changed: <file>:<fn_name> (line <n>)`

#### Scenario: --allowlist flag overrides the default path
- **WHEN** `find_stubs.py --allowlist /path/to/custom.json` is run
- **THEN** the script SHALL load stubs from the specified path instead of the default `scripts/confirmed_stubs.json`

#### Scenario: Backward compatibility — existing consumers still see a flat list
- **WHEN** a consumer reads the old `stub_report.json` format (a top-level array)
- **THEN** `find_stubs.py` SHALL ALSO write the legacy flat-array format to `stub_report.json` alongside the new structured format, so existing tooling does not break
