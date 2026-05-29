# Port Testing Design

**Date:** 2026-05-29
**Scope:** Network port correctness tests for the forgottenserver-rust server

## Goal

Verify that all ports the Rust server exposes (7171 status, 7172 game, 8080 HTTP) accept connections and respond correctly to clients. Simultaneously audit and fix behavioral gaps (stubs, missing version enforcement, missing DB seeding) found in the network, server, scripting, and e2e crates.

## Ports in scope

| Port | Protocol | Current test coverage |
|------|----------|-----------------------|
| 7172 | Game (TCP, Tibia binary) | TCP connect only; login test records but does not assert response |
| 7171 | Status (TCP, binary or HTTP GET) | Binary + HTTP tests exist but assert weakly |
| 8080 | HTTP admin | Not mapped in testcontainer; no tests |

## Client version range

The upstream C++ server (forgottenserver-upstream) accepts **versions 1310–1311** (Tibia 13.10) only, enforced in `protocolgame.cpp:391`. The Rust port currently parses the version field but never validates the range — this is a behavioral gap that must be fixed.

## Three-track plan (executed sequentially: 3 → 1 → 2)

### Track 3 — Stub scan and fix

Audit crates: `network`, `server`, `scripting`, `e2e`.

**Known gaps:**

| Crate | Gap | Fix |
|-------|-----|-----|
| `network` | `parse_login_packet` never validates version against `[1310, 1311]` | Add version range check; send disconnect message matching C++ text |
| `network` | Disconnect packet shape not tested end-to-end | Add unit test for disconnect packet byte layout |
| `server` | Boot registration order for all 3 ports not asserted | Verify + add test if missing |
| `scripting` | `CLIENT_VERSION` Lua global (`min`, `max`, `string`) may not match upstream constants | Audit against `definitions.h`; fix if wrong |
| `e2e` | No test account seeded in MariaDB | Add `seed_db()` helper that inserts account + player rows |
| `e2e` | Port 8080 not mapped in testcontainer | Add to `get_host_port_ipv4` calls in `common/mod.rs` |

**Fix process (TDD — mandatory per CLAUDE.md):**
1. Write failing test capturing C++ behavior
2. Implement fix
3. Confirm test passes
4. Run `cargo test --lib --workspace` + `cargo clippy --workspace --lib --tests -- -D warnings`

---

### Track 1 — In-process protocol tests

Location: `crates/network/tests/protocol_integration.rs`

Uses `Cursor`-based mock streams — no Docker required.

"Valid account" in these tests means a mocked account context — no real DB. The mock returns a successful lookup so each test isolates protocol behavior only. Real DB validation is covered by Track 2 e2e tests.

**Game protocol:**

| Test | Input | Expected |
|------|-------|----------|
| `game_login_version_too_low` | version=760 | Disconnect packet: `"Only clients with protocol 13.10 allowed!"` |
| `game_login_version_too_high` | version=9999 | Same disconnect |
| `game_login_version_accepted` | version=1310, mocked valid account | No disconnect; state advances to `RequestCharlist` |
| `game_login_version_accepted_max` | version=1311, mocked valid account | Same as above |
| `game_login_bad_credentials` | version=1310, mocked bad credentials | Error packet: `"Account name or password is not correct."` |
| `game_login_charlist_packet_shape` | version=1310, mocked valid account + one character | First response byte == `0x64` (charlist opcode, confirmed at `protocolgame.rs:2117`) |

**Status protocol:**

| Test | Input | Expected |
|------|-------|----------|
| `status_binary_request_returns_xml` | byte `0xFF` then `"info"` | Response contains `<tsqp>` |
| `status_http_get_returns_200` | `GET /\r\n\r\n` | HTTP 200 + `Content-Type: text/xml` |
| `status_fields_match_server_config` | binary request with known config | `servername`, `serverport`, `client` fields match config values |

**Connection state machine:**

| Test | Transition |
|------|-----------|
| `connection_starts_pending` | Initial state == `Pending` |
| `connection_advances_to_request_charlist` | Valid login packet → `RequestCharlist` |
| `connection_advances_to_game` | Character select → `Game` |
| `connection_disconnected_on_bad_version` | Bad version → `Disconnected` |

---

### Track 2 — E2E behavioral tests

Location: `crates/e2e/tests/`

Requires Docker + MariaDB. Runs with `cargo test -p forgottenserver-e2e --features e2e -- --test-threads=1`.

**DB seeding helper (`crates/e2e/tests/common/seed.rs`):**

Columns match `schema.sql` exactly (no `coins` column; player columns use schema defaults where possible):
```sql
INSERT INTO accounts (id, name, password, type) VALUES (1, 'test', SHA1('test'), 1);
INSERT INTO players (id, name, account_id, vocation, level, health, healthmax, town_id, posx, posy, posz, cap, sex)
  VALUES (1, 'Testchar', 1, 0, 1, 150, 150, 1, 160, 54, 7, 400, 0);
```
Called in `ServerHandle` setup after `"Forgotten Server Online!"` log line.

**New and upgraded tests:**

| Test | Port | Assertion |
|------|------|-----------|
| `game_login_valid_gets_charlist` | 7172 | Send version=1310 + account `test`/`test` → first response byte == `0x64` |
| `game_login_bad_version_gets_disconnect` | 7172 | Send version=760 → disconnect received (no hang/timeout) |
| `game_login_bad_credentials_gets_error` | 7172 | Send version=1310 + wrong password → error message packet received |
| `status_fields_reflect_config` | 7171 | XML `servername` matches `serverName` from `config.lua` |
| `status_binary_returns_full_xml` | 7171 | Full XML structure validated (upgrade existing weak test) |
| `http_port_responds` | 8080 | Map port in testcontainer; assert HTTP 200 from `GET /` |

**Testcontainer changes (`crates/e2e/tests/common/mod.rs`):**
- Add `8080` to port mapping alongside 7171/7172
- Add `http_port()` accessor to `ServerHandle`
- Call `seed_db()` before returning `ServerHandle`

---

## Out of scope

- Admin port (not exposed in config by default; no upstream behavior to port)
- Client versions below 1310 other than testing the rejection message
- Byte-exact response matching against C++ golden files (reserved for wire parity tests that already exist)
- Character select → enter world (requires full game state; deferred)

## Success criteria

- `cargo test --lib --workspace` passes with no failures
- `cargo test -p forgottenserver-e2e --features e2e -- --test-threads=1` passes with no failures
- `cargo clippy --workspace --lib --tests -- -D warnings` reports zero warnings
- `docker compose up --build` produces clean logs with no runtime errors
