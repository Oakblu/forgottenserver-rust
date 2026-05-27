# `poketibia-server` — bootable binary

The runnable entrypoint for the Rust port of `forgottenserver`. This crate
wires the ten library crates of the workspace (`forgottenserver-{common,
items, map, entity, world, database, game, scripting, network, server}`)
into a single `poketibia-server` binary that mirrors the C++
`forgottenserver/src/otserv.cpp` `main()` flow.

## Usage

```bash
poketibia-server [--config <path>] [--data <dir>]

  --config <path>   Path to config.lua (default: ./config.lua)
  --data   <dir>    Path to data/ directory (default: ./data)
  --help            Show help and exit
```

Exit codes:

| Code | Meaning                                                                       |
| ---- | ----------------------------------------------------------------------------- |
| 0    | Clean shutdown after SIGINT/SIGTERM                                           |
| 1    | Fatal error during boot or listener start (message printed to stderr)         |
| 2    | Unknown CLI argument                                                          |

## Boot sequence

Each step maps to a C++ `mainLoader()` step. Steps marked **PARTIAL** depend
on architectural work tracked in
`openspec/changes/forgottenserver-rust-architectural-parity/`.

| #   | C++ step                                            | Rust impl                                            | Status     |
| --- | --------------------------------------------------- | ---------------------------------------------------- | ---------- |
| 1   | `ConfigManager::load(config.lua)`                   | `boot::initialise_modules` → `ConfigManager::load`   | **DONE**   |
| 2   | Database connect (mariadb-connector)                | *deferred*                                           | PARTIAL    |
| 3   | `DatabaseManager::isDatabaseSetup → optimizeTables` | *deferred*                                           | PARTIAL    |
| 4   | `Items::loadFromOtb` + `Items::loadFromXml`         | `srv_boot::boot()` → `load_items_otb`                | **DONE**   |
| 5   | `Vocations::loadFromXml`                            | *deferred*                                           | PARTIAL    |
| 6   | `Monsters::loadFromXml`                             | *deferred*                                           | PARTIAL    |
| 7   | `Outfits::loadFromXml`                              | *deferred*                                           | PARTIAL    |
| 8   | `Mounts::loadFromXml`                               | *deferred*                                           | PARTIAL    |
| 9   | `Houses::loadHousesXML` / `Map::loadMap`            | *deferred*                                           | PARTIAL    |
| 10  | `Scripts::loadScripts(data/scripts)`                | `srv_boot::boot()` → `load_lua_scripts` (best-effort) | **DONE**  |
| 11  | `Actions`/`MoveEvents`/`TalkActions`/`CreatureEvents`/`GlobalEvents` | *deferred*                          | PARTIAL    |
| 12  | `Game::initialise`                                  | `GameState::new()`                                   | **DONE**   |
| 13  | `Scheduler::start` + `Dispatcher::start`            | *deferred*                                           | PARTIAL    |
| 14  | Status listener on 7171                             | `srv_boot::start_admin_and_status`                   | **DONE**   |
| 15  | Game listener on 7172                               | *deferred* (wire bytes are byte-correct per wire-parity change; accept loop wiring is the remaining gap) | PARTIAL    |
| 16  | HTTP listener on 8080                               | *deferred*                                           | PARTIAL    |
| 17  | POSIX signals (SIGINT/SIGTERM)                      | `boot::install_signal_handlers` (libc::signal)       | **DONE**   |
| 18  | Game-loop tick until shutdown                       | `boot::wait_for_shutdown` (100 ms poll on flag)      | **DONE**   |

## Behaviour against architectural PARTIALs

Per the design's "PARTIAL outcomes surface as warnings, not panics" rule:

- **Missing config / data file**: clean error message to stderr, exit code 1.
  Never panics.
- **OTB attribute 0x2f / 0x30 not yet supported**: logged as `[Warning]` per
  occurrence; loader continues.
- **Lua script load errors**: silently swallowed (count returned via
  `GameData::scripts_loaded`). Matches C++ best-effort behaviour.
- **Game-protocol port (7172)**: not bound by this binary yet. The wire-
  bytes serializers are byte-correct (per
  `forgottenserver-rust-protocolgame-wire-parity`), but the accept loop +
  ProtocolGame::onAcceptedConnection wiring lives in
  `forgottenserver-rust-architectural-parity`'s scope.
- **Status port (7171)**: bound and responding. This is the smoke-test
  target for the parity-test stack.

## Testing

```bash
# Unit + integration tests (uses the bundled data/ symlink)
cargo test -p poketibia-server

# Live boot (no MariaDB required — only the status port needs binding)
cargo run --release -p poketibia-server -- \
  --config crates/poketibia-server/tests/fixtures/config.lua \
  --data data
```

You should see, in order:

```
The Forgotten Server (Rust port)
>> Loaded 38284 items, 0 spells, 316 weapons, 9 NPCs, 0 Lua scripts.
>> Forgotten Server Online! (Rust port)
>> Send SIGINT (Ctrl-C) or SIGTERM to shut down.
```

Ctrl-C produces:

```
>> Shutdown requested; exiting cleanly.
```
