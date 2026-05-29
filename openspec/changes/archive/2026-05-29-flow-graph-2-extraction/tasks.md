## 1. Node bootstrap

- [x] 1.1 Implement `scripts/flow/bootstrap_nodes.py` seeding one node per `cpp_symbol_manifest.json` symbol into `flow_graph/nodes/<cpp_file>.yml`, merging (not overwriting) phase-1 boot-spine nodes
- [x] 1.2 Add a test asserting every manifest symbol has a node and no node key is absent from the manifest

## 2. Static edge extractor

- [x] 2.1 Resolve O1: detect whether libclang is available; choose libclang vs. tree-sitter/heuristic and record the decision in `flow_graph/README.md`
- [x] 2.2 Implement `scripts/flow/build_edges.py` deriving `kind: static`/`confidence: static` edges from `forgottenserver-upstream/src/` (read-only), attaching them as out-edges
- [x] 2.3 Implement merge semantics: recompute only static edges, preserve all `kind: dynamic` curated edges
- [x] 2.4 Emit a static-edge coverage report on completion

## 3. Build target and tests

- [x] 3.1 Add `make flow-build` invoking bootstrap + edge extraction
- [x] 3.2 Add tests: a known direct call (e.g. `mainLoader`→`Game::loadMainMap`) yields a static edge; running build twice is a no-op diff; a curated dynamic edge survives a rebuild

## 4. Verification

- [x] 4.1 Run `make flow-build` then `make flow`; confirm the expanded graph validates clean
- [x] 4.2 Confirm no file under `forgottenserver-upstream/src/` or any Rust crate was modified
