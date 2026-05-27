# forgottenserver-rust

A complete Rust port of [ForgottenServer](https://github.com/opentibiabr/forgottenserver), the open-source C++ Tibia MMORPG server emulator. The C++ source lives at `./forgottenserver/` (vendored, read-only) and is the authoritative reference for all behavior.

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
forgottenserver/    vendored C++ source (read-only reference)
data/               game data: items.otb, world map, Lua scripts, XML configs
schema.sql          MariaDB schema
docker/             DB init scripts (applied automatically on first start)
```

---

## Running with Docker

### Prerequisites

- Docker Desktop v24+ with `docker compose` v2

### Step 1 — Build the server image

The Dockerfile expects to be invoked from the **monorepo root** so that both the Rust workspace and the vendored C++ data directory are in scope:

```bash
docker build \
  -f apps/poketibia/forgottenserver-rust/Dockerfile \
  -t forgottenserver-rust:latest \
  .
```

First build: ~3–5 minutes (compiles Rust crates + vendored Lua 5.4).
Subsequent builds: <30 s with BuildKit layer cache.

### Step 2 — Start MariaDB and apply the schema

The init script at `docker/poketibia-mariadb-init/00-init-tibia-dbs.sh` runs automatically on the first container start. It creates the `tibia_rs` database and applies `schema.sql` before any connections are accepted. Mount both the schema and the init directory as shown below:

```bash
docker run -d \
  --name forgottenserver-db \
  --health-cmd="healthcheck.sh --connect --innodb_initialized" \
  --health-interval=5s \
  --health-retries=12 \
  -e MARIADB_ROOT_PASSWORD=root_secret \
  -e MARIADB_USER=forgottenserver \
  -e MARIADB_PASSWORD=forgottenserver \
  -e MARIADB_DATABASE=tibia_rs \
  -v "$(pwd)/schema.sql:/opt/poketibia-schema.sql:ro" \
  -v "$(pwd)/docker/poketibia-mariadb-init:/docker-entrypoint-initdb.d:ro" \
  mariadb:11
```

Wait for the DB to pass its health check before proceeding:

```bash
until [ "$(docker inspect --format='{{.State.Health.Status}}' forgottenserver-db 2>/dev/null)" = "healthy" ]; do
  echo "Waiting for DB…"; sleep 3
done
echo "DB is healthy — schema applied."
```

### Step 3 — Run the server

```bash
docker run --rm -it \
  --name forgottenserver-rust \
  --link forgottenserver-db:db \
  -p 7171:7171 -p 7172:7172 -p 8080:8080 \
  forgottenserver-rust:latest
```

Expected output:

```
The Forgotten Server (Rust port)
>> Loaded 38284 items, 0 spells, 316 weapons, 9 NPCs, 0 Lua scripts.
>> Forgotten Server Online! (Rust port)
>> Send SIGINT (Ctrl-C) or SIGTERM to shut down.
```

Press Ctrl-C for a clean shutdown.

### Step 4 — Verify

```bash
# Status port should be reachable
nc -zv 127.0.0.1 7171

# Raw TFS status probe
printf '\x06\x00\xff\xff\x79\x6c\x00\x00' | nc -w 2 127.0.0.1 7171 | xxd | head
```

---

## docker-compose example

For a repeatable local stack, create a `docker-compose.yml` at the **monorepo root**:

```yaml
services:
  db:
    image: mariadb:11
    environment:
      MARIADB_ROOT_PASSWORD: root_secret
      MARIADB_USER: forgottenserver
      MARIADB_PASSWORD: forgottenserver
      MARIADB_DATABASE: tibia_rs
    volumes:
      - ./apps/poketibia/forgottenserver-rust/schema.sql:/opt/poketibia-schema.sql:ro
      - ./apps/poketibia/forgottenserver-rust/docker/poketibia-mariadb-init:/docker-entrypoint-initdb.d:ro
    healthcheck:
      test: ["CMD", "healthcheck.sh", "--connect", "--innodb_initialized"]
      interval: 5s
      retries: 12

  server:
    build:
      context: .
      dockerfile: apps/poketibia/forgottenserver-rust/Dockerfile
    ports:
      - "7171:7171"
      - "7172:7172"
      - "8080:8080"
    depends_on:
      db:
        condition: service_healthy
```

```bash
docker compose up --build
```

The `depends_on: condition: service_healthy` ensures the server only starts after MariaDB has finished initialising and applying the schema.

---

## Importing database data

These commands assume the DB container is running as `forgottenserver-db` with the credentials above.

### From a SQL dump

```bash
docker exec -i forgottenserver-db \
  mariadb -u forgottenserver -pforgottenserver tibia_rs \
  < /path/to/your/backup.sql
```

### From a compressed dump

```bash
gunzip -c backup.sql.gz | docker exec -i forgottenserver-db \
  mariadb -u forgottenserver -pforgottenserver tibia_rs
```

### Create an account and character manually

```bash
# Account
docker exec -i forgottenserver-db \
  mariadb -u forgottenserver -pforgottenserver tibia_rs -e \
  "INSERT INTO accounts (name, password, type, email)
   VALUES ('myaccount', SHA1('password123'), 1, 'me@local.dev');"

# Character
docker exec -i forgottenserver-db \
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
docker exec -i forgottenserver-db \
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

# Run locally without Docker (no DB required for the status port)
cargo run --release -p poketibia-server -- \
  --config crates/poketibia-server/tests/fixtures/config.lua \
  --data data
```

See `MIGRATION_LEDGER.yml` for per-symbol migration status and `AI_MIGRATION_CONTEXT.md` for architecture decisions and C++ → Rust mapping.
