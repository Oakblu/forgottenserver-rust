## 1. Gap analysis engine

- [ ] 1.1 Implement `scripts/flow/gap_analysis.py`: traverse from root, join each reachable node against `MIGRATION_LEDGER.yml` status and `rust_symbol_manifest.json` presence
- [ ] 1.2 Classify findings as `MISSING_FLOW`, `BRANCH_GAP`, `DYNAMIC_GAP`, `ORDER_MISMATCH`
- [ ] 1.3 Suppress findings whose symbol is in `intentional_differences.yml`; collect them into a separate suppressed list
- [ ] 1.4 Compute `priority` from reachability depth, ledger status, and node criticality; sort findings

## 2. Report and view generation

- [ ] 2.1 Generate `flow_graph/GAP_REPORT.md` (prioritized actionable findings + suppressed appendix; each finding: category, priority, `{file, qualified_name}`, ledger status, source location) via `make flow-gap`
- [ ] 2.2 Implement `scripts/flow/render_markdown.py` generating `flow_graph/FLOW_GRAPH.md` fully from the YAML (boot sequence as ordered list, per-subsystem node tables linking to ledger status)

## 3. Tests

- [ ] 3.1 Add tests: reachable+PENDING → MISSING_FLOW; reachable+DONE → no finding; intentional difference → suppressed; dynamic edge without Rust handler → DYNAMIC_GAP; all-DONE/suppressed → empty actionable report
- [ ] 3.2 Add a test asserting boot-spine findings outrank deep-branch findings

## 4. CI and wiring

- [ ] 4.1 Add a CI step running `make flow` (well-formedness + manifest consistency + phase 3–5 coverage checks), mirroring `make ledger`
- [ ] 4.2 Document the `flow`/`flow-build`/`flow-gap` workflow in `flow_graph/README.md` and reference it from the migration docs

## 5. Verification

- [ ] 5.1 Run `make flow-gap` end-to-end against current ledger state; confirm a prioritized `GAP_REPORT.md` and a generated `FLOW_GRAPH.md` are produced
- [ ] 5.2 Spot-check the top findings against the C++ source and ledger; record any legitimate divergence in `intentional_differences.yml` and re-run
- [ ] 5.3 Confirm no file under `forgottenserver-upstream/src/` or any Rust crate was modified
