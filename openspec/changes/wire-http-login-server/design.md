## Context

The Rust server exposes ports 7171 (status/admin), 7172 (game protocol), and 8080 (HTTP login). Ports 7171 and 7172 are live. Port 8080 is never bound during boot; `start_listeners()` in `crates/tfs/src/boot.rs` simply omits the HTTP start call. Even the existing `crates/server/src/http.rs` accept loop, if called, immediately `drop(stream)` without sending any response.

The C++ reference uses Boost.Beast for HTTP parsing and async I/O. The Rust codebase uses `std::net::TcpListener` / `TcpStream` + blocking threads for all existing TCP services (status, admin, game). We must stay consistent with that threading model.

The C++ login protocol routes on a single endpoint: every POST has a `"type"` field in its JSON body that dispatches to `"login"` or `"cacheinfo"` handlers. There is no URL-based routing.

The `Database` trait (`crates/database/src/database.rs`) exposes `query(&self, sql)` for reads and `execute(&mut self, sql)` for writes. The login flow requires both (read: account lookup + player listing; write: INSERT into `sessions`). Concurrent access from multiple HTTP worker threads requires `Arc<Mutex<Box<dyn Database + Send>>>`.

## Goals / Non-Goals

**Goals:**
- Bind port 8080 during boot using `httpPort` + `httpWorkers` config keys.
- For each accepted connection: read the HTTP request bytes, parse method/headers/body, route by JSON `"type"` field, write a well-formed `HTTP/1.1` response with `Content-Type: application/json`.
- Implement `"type": "login"` — query the DB for account credentials, validate SHA-1 password + optional TOTP, insert a session token, return character list JSON matching C++ output shape exactly.
- Implement `"type": "cacheinfo"` — count rows in `players_online`, return `{"playersonline": N}`.
- 30-second read timeout per connection, matching C++ `stream.expires_after(30s)`.
- `GET /` returns `HTTP/1.1 200 OK` (required by existing E2E test).
- All existing tests continue to pass.

**Non-Goals:**
- Async I/O (`tokio` / `async-std`) — out of scope; blocking threads match the existing codebase.
- HTTPS / TLS — the C++ server does not terminate TLS at this layer.
- Keep-alive connection reuse — each connection is closed after one request, matching the existing C++ behaviour for the login flow.
- WebSocket or streaming — not used by the login protocol.
- URL-based routing or REST conventions — the C++ protocol is type-field dispatch only.

## Decisions

### D1 — HTTP parsing: `httparse` crate

**Decision**: Use the `httparse` crate for HTTP/1.1 request parsing.

**Rationale**: `httparse` is a zero-copy, no-unsafe, `no_std`-compatible parser used by Hyper. It handles method, path, version, and headers correctly, covers edge cases (multi-line headers, partial reads) and is widely audited. Hand-rolling is error-prone and an unjustified distraction from the real work.

**Alternative considered**: Full-featured crates like `hyper` or `axum`. Rejected because they pull in `tokio` and async runtimes, conflicting with the project's blocking-thread model. `httparse` is a parser only — no runtime dependency.

**Alternative considered**: Hand-rolled line reader. Rejected because HTTP header parsing has subtle edge cases (folding, LWS, chunked bodies) that `httparse` already handles correctly and testably.

### D2 — Response serialization: `serde_json`

**Decision**: Use `serde_json` (already in the dependency tree) for building JSON response bodies.

**Rationale**: The login response JSON is large and nested (session object, worlds array, characters array). Hand-formatting it with `format!()` is fragile and untestable. `serde_json` + `#[derive(Serialize)]` structs produce correct output and are easy to unit-test field by field.

**Alternative considered**: Hand-formatted strings via `format!()`. Rejected for the response body (too complex); retained only for `cacheinfo` which is a trivial one-field object.

### D3 — Database access: `Arc<Mutex<Box<dyn Database + Send>>>`

**Decision**: Pass the database handle to the HTTP listener as `Arc<Mutex<Box<dyn Database + Send>>>`, cloned into each accepted connection's handler closure.

**Rationale**: Each HTTP worker thread needs write access to the DB (INSERT into `sessions`). The `Database` trait's `execute(&mut self, ...)` requires `&mut` access, which mandates a `Mutex`. `Arc` shares the handle across threads. This mirrors how `game_state` is shared via `Arc<Mutex<GameState>>` in the existing boot sequence.

**Alternative considered**: Thread-local DB connections (like C++ `thread_local auto& db = Database::getInstance()`). Rejected because the Rust `Database` trait does not have a global singleton and creating per-thread connections would require the `mariadb` crate's connection pool, which is a larger scope change.

**Alternative considered**: `RwLock` (readers for SELECT, write lock only for INSERT). Rejected because the `execute(&mut self, ...)` signature requires exclusive access for all mutating calls; the design at the trait level does not distinguish read-only vs. write operations.

### D4 — Session token encoding: `base64` + 16 random bytes

**Decision**: Generate session tokens as 16 cryptographically random bytes, Base64-encoded, matching C++ `randomBytes(16)` + `tfs::base64::encode(sessionKey)`.

**Rationale**: The C++ inserts the raw bytes into the DB `sessions.token` (BINARY(16)) and returns the Base64-encoded value to the client. We must preserve that contract exactly.

**Implementation**: `rand::random::<[u8; 16]>()` (or `getrandom`) for the bytes; the `base64` crate (already available, used by `tools.rs`) for encoding.

### D5 — Threading model: one thread per accepted connection

**Decision**: Spawn a new `std::thread` per accepted connection inside the HTTP listener's accept loop, matching the existing pattern in `start_admin_and_status` and `start_game_listener`.

**Rationale**: Consistent with the project's blocking model. A connection lives for the duration of one request/response pair, so thread overhead is bounded by concurrent login attempts (typically very low).

### D6 — Boot wiring: new `start_http_listener()` in `crates/server/src/boot.rs`

**Decision**: Add `pub fn start_http_listener(config, db)` to `crates/server/src/boot.rs` alongside the existing `start_game_listener` and `start_admin_and_status`. Call it from `crates/tfs/src/boot.rs` `start_listeners()`.

**Rationale**: Keeps the pattern consistent. `start_listeners()` is the single place that binds all ports.

## Risks / Trade-offs

- **Mutex contention on DB writes** → At login time, only one thread can hold the DB lock. For a game server, concurrent login bursts are rare; this is acceptable. If contention becomes an issue, a per-request DB connection from a pool is the next step.
- **`httparse` partial reads** → HTTP requests from OTClient arrive in a single TCP segment in practice, but the parser must handle `Incomplete` results and read more bytes in a loop. Incorrectly handling partial reads silently truncates headers.  Mitigation: implement a read-into-buffer loop with a maximum header size of 8 KB (matching common HTTP server defaults).
- **30-second timeout with `std::net`** → `TcpStream::set_read_timeout()` sets a timeout on the underlying socket, but a slow client sending headers byte-by-byte could hold a thread for up to 30 s × N bytes. Mitigation: cap total header bytes at 8 KB before the timeout fires.
- **JSON deserialization of request body** → `serde_json::from_str` panics on stack overflow for adversarially nested JSON. Mitigation: set a max nesting depth via `serde_json::Deserializer` or validate body length (< 4 KB) before parsing.

## Migration Plan

1. Add `httparse` and ensure `serde_json` + `rand`/`getrandom` are in `crates/server/Cargo.toml`.
2. Implement `HttpConnectionSession` in new file `crates/server/src/http_connection_session.rs`.
3. Wire `HttpConnectionSession` as the session factory in `crates/server/src/http.rs` `accept_loop`.
4. Implement `start_http_listener(config, db)` in `crates/server/src/boot.rs`.
5. Rewrite `http_login.rs` `handle_login` to accept `&mut dyn Database`.
6. Rewrite `http_cacheinfo.rs` `handle_cacheinfo` to accept `&dyn Database`.
7. Call `start_http_listener` from `crates/tfs/src/boot.rs` `start_listeners()`, passing the DB handle.
8. Run `cargo test --lib --workspace` — all tests pass.
9. Run `docker compose up --build` — confirm `curl http://localhost:8080/` returns `HTTP/1.1 200 OK`.
10. Manually test OTClient login.

**Rollback**: Removing the `start_http_listener` call restores the pre-change behaviour (port 8080 not bound). No schema changes are introduced; the `sessions` table already exists.

## Open Questions

- *(none — all blocking decisions resolved above)*
