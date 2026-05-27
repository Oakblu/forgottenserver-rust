# ForgottenServer Rust ↔ C++ Equivalence Harness

Differential test harness that validates the Rust port's behavior
against the C++ source of truth.

See `openspec/changes/forgottenserver-rust-equivalence-harness/` for
the change proposal, design, and tasks.

## Quick start

From the repo root:

```bash
# All lanes (requires the side-by-side stack to be up):
make -C apps/poketibia/forgottenserver-rust harness

# Subset of lanes:
HARNESS_LANES=wire_replay,otbm_diff make -C apps/poketibia/forgottenserver-rust harness

# Stack management only:
make -C apps/poketibia/forgottenserver-rust harness-up
make -C apps/poketibia/forgottenserver-rust harness-down
```

## Lanes

| Lane              | Driver                              | Purpose                                            |
| ----------------- | ----------------------------------- | -------------------------------------------------- |
| `wire_replay`     | `lanes/wire_replay.sh`              | Replay recorded Tibia client sessions; byte-diff   |
| `lua_bindings`    | `lanes/lua_bindings.sh`             | Static + runtime audit of `LuaScriptInterface`     |
| `otbm_diff`       | `lanes/otbm_diff.sh`                | Identical `.otbm` files dumped + tile-diffed       |
| `persisted_state` | `lanes/persisted_state.sh`          | Post-scenario MariaDB row-diff per-table           |

Each lane produces a JSON-line report appended to
`reports/run-<timestamp>.json`. The ledger writer (Phase 6) consumes
these reports and proposes `MIGRATION_LEDGER.yml` transitions.

Currently all lane drivers are placeholders — they will land in
Phases 1, 2, 3, and 5 respectively. Phase 0 ships only the scaffold.

## Directory layout

```
scripts/harness/
├── README.md           — this file
├── run.sh              — top-level entry point
├── lib.sh              — shared shell helpers
├── lanes/              — per-lane driver scripts (one per lane)
├── captures/           — recorded Tibia client sessions (one dir per scenario)
├── lua_corpus/         — one Lua file per binding-under-test
├── otbm_fixtures/      — handcrafted .otbm files for map-load diff
├── reports/            — per-run JSON reports + ledger proposals
└── lib/                — Rust binaries used by lanes (replayer, diff, …)
```

## How to record a new wire-replay scenario

Documented in detail when Phase 1 lands. Sketch:

1. Bring up `forgottenserver-cpp` only.
2. Use `mitmproxy` or equivalent to capture a Tibia client session.
3. Save as `captures/<scenario>/packets.jsonl`.
4. Add a `README.md` describing the scenario.
5. (Optional) Add `persistence.toml` if the scenario mutates DB state
   that should be diffed by lane 5.
6. (Optional) Add `db_seed.sql` if the scenario assumes a specific
   starting DB state.

## How to add a Lua binding to the corpus

Documented in detail when Phase 2 lands. Sketch:

1. Add a `.lua` file under
   `lua_corpus/<group>/<binding_name>.lua`.
2. Exercise the binding minimally and `print()` an observable result.
3. The lane driver will execute it against both servers and diff
   the output.

## How to add an OTBM fixture

Documented in detail when Phase 3 lands. Sketch:

1. Author the `.otbm` (smallest possible to exercise the behavior).
2. Drop under `otbm_fixtures/<name>.otbm`.
3. The lane driver will dump tiles from both servers and diff.

## Ledger transitions

The ledger writer (Phase 6) proposes status transitions for
`MIGRATION_LEDGER.yml` based on lane reports. Transitions are
proposed, never auto-applied — a human reviews
`reports/ledger_proposal-<timestamp>.diff` before committing.

## Backend switching

The Rust binary defaults to the `MariaDbDatabase` backend (Phase 4).
For unit-test contexts (no Docker), pass `--db-backend=in-memory`.

```bash
# Production / harness:
poketibia-server --config /srv/config.lua --db-backend=mariadb

# Local dev without MariaDB:
poketibia-server --config /srv/config.lua --db-backend=in-memory
```
