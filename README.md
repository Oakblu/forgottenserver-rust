# forgottenserver-rust

![tests](https://img.shields.io/badge/tests-6255%20passing-brightgreen?style=flat-square)
![coverage](https://img.shields.io/badge/coverage-94.85%25-brightgreen?style=flat-square)
![build](https://img.shields.io/badge/build-passing-brightgreen?style=flat-square)
![rust](https://img.shields.io/badge/rust-1.94-orange?style=flat-square&logo=rust&logoColor=white)
![license](https://img.shields.io/badge/license-GPL--2.0-blue?style=flat-square)
![docker](https://img.shields.io/badge/docker-compose-2496ED?style=flat-square&logo=docker&logoColor=white)
![mariadb](https://img.shields.io/badge/mariadb-11-003545?style=flat-square&logo=mariadb&logoColor=white)
![crates](https://img.shields.io/badge/crates-13-orange?style=flat-square&logo=rust&logoColor=white)
![loc](https://img.shields.io/badge/rust%20LOC-136k-lightgrey?style=flat-square)

A complete Rust port of [ForgottenServer](https://github.com/otland/forgottenserver), the open-source C++ Tibia MMORPG server emulator.

---

## Why Rust

The C++ codebase is correct and battle-tested, but carries structural constraints that make long-term maintenance hard:

- **Memory safety.** The C++ server uses raw pointers, manual reference counting (`std::shared_ptr`), and intrusive parent pointers throughout. Rust eliminates entire classes of use-after-free, data-race, and null-dereference bugs at compile time.
- **Fearless concurrency.** The original dispatcher/scheduler model relies on shared mutable globals (`g_game`, `g_chat`). Rust's ownership model enforces explicit data access patterns, making concurrent game logic auditable and safe by construction.
- **Modern tooling.** Cargo, `clippy`, `rustfmt`, and `cargo test` provide a fast, consistent development loop that the CMake + vcpkg C++ build cannot match.
- **Behavior-preserving migration.** The port is strictly behavior-preserving: every observable C++ behavior is reproduced identically before any refactor is allowed. The C++ source is the spec; the Rust crates are the implementation.

---

## Project structure

```
crates/
  common/           shared types, constants, enums  (tools.h, enums.h)
  items/            Item, Container, ItemType registry  (items.cpp/h)
  map/              Map, Tile, Position, SpectatorVec  (map.cpp/h)
  entity/           Player, Creature, Monster, Npc  (creature.cpp, player.cpp …)
  world/            IOLoginData, IOMapSerialize, world load  (iologindata.cpp …)
  database/         Database, DBResult, DatabaseTasks  (database.cpp/h)
  game/             Game, combat, conditions, vocations  (game.cpp/h)
  scripting/        LuaScriptInterface, *Events, *Functions  (luascript.cpp/h)
  network/          Protocol*, NetworkMessage, Connection  (protocol*.cpp/h)
  server/           entry point, boot, scheduler  (main.cpp, server.cpp)
  poketibia-server/ runnable binary that wires all crates together
data/               game data: items.otb, world map, Lua scripts, XML configs
schema.sql          MariaDB schema (auto-applied on first DB start)
docker/             DB init scripts
```

---

## Running with Docker

### Prerequisites

Docker Desktop v24+ with `docker compose` v2.

### Start everything

```bash
docker compose up --build
```

This will:

1. Pull MariaDB 11 and start the `db` service.
2. Mount `schema.sql` and run `docker/poketibia-mariadb-init/00-init-tibia-dbs.sh` on first start, creating the `tibia_rs` database and applying the full schema automatically.
3. Wait for the DB health check to pass before starting the server.
4. Build the `server` image from this repo and start it.

Expected server output once running:

```
The Forgotten Server (Rust port)
>> Loaded 38284 items, 0 spells, 316 weapons, 9 NPCs, 0 Lua scripts.
>> Forgotten Server Online! (Rust port)
>> Send SIGINT (Ctrl-C) or SIGTERM to shut down.
```

Press Ctrl-C for a clean shutdown.

### Verify

```bash
# Status port reachable?
nc -zv 127.0.0.1 7171

# Raw TFS status probe
printf '\x06\x00\xff\xff\x79\x6c\x00\x00' | nc -w 2 127.0.0.1 7171 | xxd | head
```

### Lifecycle

```bash
# Stop (preserves player data in the named volume)
docker compose down

# Wipe all data and start fresh
docker compose down -v && docker compose up --build
```

---

## Importing database data

The DB container is named `forgottenserver-rust-db-1` by default (check with `docker compose ps`).

### From a SQL dump

```bash
docker exec -i forgottenserver-rust-db-1 \
  mariadb -u forgottenserver -pforgottenserver tibia_rs \
  < /path/to/your/backup.sql
```

### From a compressed dump

```bash
gunzip -c backup.sql.gz | docker exec -i forgottenserver-rust-db-1 \
  mariadb -u forgottenserver -pforgottenserver tibia_rs
```

### Create an account and character manually

```bash
# Account
docker exec -i forgottenserver-rust-db-1 \
  mariadb -u forgottenserver -pforgottenserver tibia_rs -e \
  "INSERT INTO accounts (name, password, type, email)
   VALUES ('myaccount', SHA1('password123'), 1, 'me@local.dev');"

# Character
docker exec -i forgottenserver-rust-db-1 \
  mariadb -u forgottenserver -pforgottenserver tibia_rs -e \
  "INSERT INTO players
     (name, group_id, account_id, level, vocation,
      health, healthmax, mana, manamax, soul, cap,
      sex, town_id, looktype, posx, posy, posz)
   SELECT 'MyHero', 1, id, 1, 0,
          150, 150, 0, 0, 100, 400, 1, 1, 136, 160, 55, 7
   FROM accounts WHERE name='myaccount';"
```

### Verify imported data

```bash
docker exec -i forgottenserver-rust-db-1 \
  mariadb -u forgottenserver -pforgottenserver tibia_rs \
  -e "SELECT id, name, level FROM players LIMIT 20;"
```

---

## Development

```bash
# Unit tests
cargo test --lib --workspace

# Lint
cargo clippy --workspace --lib --tests -- -D warnings

# Format
cargo fmt --all

# Run locally without Docker (status port only, no DB required)
cargo run --release -p poketibia-server -- \
  --config crates/poketibia-server/tests/fixtures/config.lua \
  --data data
```

