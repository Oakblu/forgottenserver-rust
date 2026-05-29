## Why

OTClient cannot connect to the Rust server because the HTTP login server on port 8080 is never started during boot, and even the stub that exists immediately drops accepted connections without sending any response. Every client login attempt results in "Failed to connect to server (HTTP)."

## What Changes

- **New**: `HttpConnectionSession` — reads an HTTP/1.1 request, dispatches by JSON `type` field, writes a well-formed HTTP/1.1 response. This is the Rust equivalent of C++ `session.cpp`.
- **New**: `http_request_handler` — routes `"type": "login"` and `"type": "cacheinfo"` requests to their respective handlers, mirroring C++ `router.cpp::handle_request`.
- **Modified**: `crates/server/src/http.rs` — `accept_loop` currently `drop(stream)`; replaced with a real session dispatch call.
- **Modified**: `crates/tfs/src/boot.rs` `start_listeners()` — wires in `forgottenserver_server::http::start()` so port 8080 is bound on startup.
- **Modified**: `crates/server/src/boot.rs` — adds `start_http_listener()` alongside the existing `start_admin_and_status` and `start_game_listener` functions.
- **Modified**: `crates/server/src/http_login.rs` `handle_login` — accepts a `&dyn Database` reference and queries the DB for account/password/secret/characters instead of using the in-memory store.
- **Modified**: `crates/server/src/http_cacheinfo.rs` `handle_cacheinfo` — accepts a `&dyn Database` reference and queries `players_online` to resolve the `Option<u32>` player count.
- No wire-format changes to the game or status protocols.

## Capabilities

### New Capabilities

- `http-connection-session`: Per-connection HTTP/1.1 read-parse-dispatch-respond cycle that converts raw TCP bytes into routed handler calls and sends conformant HTTP responses.
- `http-login-endpoint`: The `"type":"login"` handler: validates credentials against the DB, issues a session token, and returns the character list JSON matching the C++ `handle_login` output shape exactly.
- `http-cacheinfo-endpoint`: The `"type":"cacheinfo"` handler: queries `players_online` from the DB and returns the count JSON matching C++ `handle_cacheinfo`.

### Modified Capabilities

*(none — no existing spec-level requirements are changing)*

## Impact

- **`crates/server/src/`**: `http.rs`, `boot.rs`, `http_login.rs`, `http_cacheinfo.rs` — all modified.
- **`crates/tfs/src/boot.rs`**: `start_listeners()` gains an HTTP start call.
- **`crates/database/`**: `http_login.rs` and `http_cacheinfo.rs` gain read-only DB queries (`accounts`, `players`, `sessions`, `players_online` tables). No new tables or schema changes.
- **`crates/e2e/tests/http.rs`**: `http_port_returns_200_for_get_request` test already exists and must pass as part of done criteria.
- **No new external dependencies** — HTTP parsing uses `std` I/O + hand-rolled minimal parser (matching the rest of the codebase's std-net approach), or a lightweight crate if the reviewer agrees. Decision recorded in `design.md`.
- Port 8080 must be exposed in Docker — already present in `docker-compose.yml`.
