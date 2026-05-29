## 1. Schema and docs

- [x] 1.1 Write `flow_graph/README.md` documenting the node/edge YAML schema (node key `{file, qualified_name}`; edge `target`, `kind`, `confidence`, optional `order`/`condition`) and the `index.yml`/`nodes/<file>.yml`/`FLOW_GRAPH.md` layout
- [x] 1.2 Create `flow_graph/index.yml` with `main` as sole root, the `main`→`startServer`→`mainLoader` entrypoint chain, and an empty `unreached` section

## 2. Boot-spine seed

- [x] 2.1 Author `flow_graph/nodes/main.yml`: nodes for `argumentsHandler` and `main`, edge `main`→`startServer`
- [x] 2.2 Author `flow_graph/nodes/otserv.yml`: `startServer`, `mainLoader`, and ordered/conditional out-edges for config load, `Database::connect`, world-type switch, `loadMainMap`, `payHouses`, `g_game.start`, state transitions (cross-checked against `otserv.cpp` line numbers)

## 3. Validator

- [x] 3.1 Implement `scripts/flow/validate.py`: load index + shards, verify every node key resolves in `cpp_symbol_manifest.json`, no dangling edges, reachability from root or membership in `unreached`; exit non-zero naming any offender
- [x] 3.2 Add `make flow` target invoking the validator
- [x] 3.3 Add unit tests: dangling edge fails, orphan fails, clean boot-spine graph passes

## 4. Verification

- [x] 4.1 Run `make flow` against the seeded boot spine and confirm it exits zero
- [x] 4.2 Confirm no file under `forgottenserver-upstream/src/` or any Rust crate was modified
