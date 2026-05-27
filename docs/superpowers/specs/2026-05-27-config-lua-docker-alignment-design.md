# Design: config.lua / Docker Alignment & Missing-Config Error

**Date:** 2026-05-27
**Status:** Approved

## Problem

Three related issues prevent `docker compose up` from working out of the box:

1. `config.lua` (and `config.lua.dist`) specify `mysqlDatabase = "forgottenserver"` and `mysqlPass = ""`, but `docker-compose.yml` creates database `tibia_rs` with password `forgottenserver`. The Rust server fails to connect at startup.
2. If `config.lua` is missing entirely, the server emits an opaque error with no recovery hint.
3. `config.lua` is the single user-facing configuration entry point (OT server convention); Docker must conform to it, not the reverse.

## Decision

Align Docker credentials and database name to `config.lua`. Do not parse `config.lua` from Docker scripts (fragile). Do not introduce a `.env` indirection (changes user workflow).

## Changes

### config.lua and config.lua.dist

`mysqlPass = ""` → `mysqlPass = "forgottenserver"`

`mysqlDatabase` stays `"forgottenserver"` (already correct).

### crates/tfs/tests/fixtures/config.lua

Same password update for consistency. This fixture is used by integration tests that never connect to MariaDB, so behavior is unchanged.

### docker-compose.yml

`MARIADB_DATABASE: tibia_rs` → `MARIADB_DATABASE: forgottenserver`

Password and user stay `forgottenserver` (already matching the new config.lua value).

### docker/mariadb-init/00-init-tibia-dbs.sh

Add `forgottenserver` to the database loop alongside the existing harness databases (`tibia_cpp`, `tibia_rs`, `tibia_test`). Schema is applied to all four. Harness databases are untouched.

### crates/tfs/src/main.rs

Before calling `boot::initialise_modules`, check `config_path.exists()`. If missing, print:

```
[FATAL] Config file not found: <path>
To fix: copy config.lua.dist to config.lua and edit the settings.
  cp config.lua.dist config.lua
```

Then exit with code 1. A unit test verifies a nonexistent path triggers this message.

## Testing

- Existing integration tests pass unchanged (fixture config updated, no MariaDB connection made).
- New unit test in `crates/tfs/src/main.rs` covers the missing-config path.
- Smoke test: `docker compose up --build` connects successfully and server prints "Forgotten Server Online!".

## Out of Scope

- Parsing `config.lua` dynamically from Docker.
- Changing the database name away from `forgottenserver`.
- Any changes to `schema.sql` or `bootstrap_schema_if_needed`.
