## 1. Dependencies

- [x] 1.1 Add `httparse` to `crates/server/Cargo.toml` (HTTP/1.1 request parser)
- [x] 1.2 Add `serde` + `serde_json` (with `derive` feature) to `crates/server/Cargo.toml` if not already present
- [x] 1.3 Add `rand` (or `getrandom`) to `crates/server/Cargo.toml` for cryptographic session token generation
- [x] 1.4 Verify `base64` is accessible from `crates/server` (already used in `common`); add to `Cargo.toml` if needed

## 2. HttpConnectionSession — HTTP read/dispatch/write cycle

- [x] 2.1 Create `crates/server/src/http_connection_session.rs`: struct `HttpConnectionSession` holding `Arc<Mutex<Box<dyn Database + Send>>>`  and config values (`server_name`, `ip`, `game_port`, `location`, `pvp_type`)
- [x] 2.2 Implement `HttpConnectionSession::handle(stream: TcpStream)`: set 30 s read timeout via `stream.set_read_timeout()`
- [x] 2.3 Implement HTTP request reader: read into a 8 192 byte buffer in a loop, call `httparse::Request::parse()`, handle `Incomplete` (read more) and `Complete` results; return `None` for oversized headers
- [x] 2.4 Implement body reader: after header parse completes, read exactly `Content-Length` bytes (capped at 4 096); return empty body if header is absent
- [x] 2.5 Implement `GET /` fast-path: if method is `GET`, write `HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}` and return
- [x] 2.6 Implement JSON dispatch: parse body with `serde_json::from_str`, extract `"type"` field, call `dispatch_type()` returning `(u16 status, String body)`
- [x] 2.7 Implement `dispatch_type()`: match `"login"` → `handle_login_request()`, `"cacheinfo"` → `handle_cacheinfo_request()`, unknown → return error code 2 body
- [x] 2.8 Implement response writer: format `HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {N}\r\nConnection: close\r\n\r\n{body}` and write to stream
- [x] 2.9 Write unit tests for: oversized header rejection, `GET /` response, unknown type error, non-JSON body error, correct Content-Length header

## 3. Login handler — DB-backed credential validation

- [x] 3.1 Add `LoginRequest` struct (fields: `email: String`, `password: String`, `token: Option<String>`) with `#[derive(Deserialize)]`
- [x] 3.2 Add `LoginResponse` struct mirroring C++ JSON shape (Session sub-struct, PlayData sub-struct, World sub-struct, Character sub-struct) with `#[derive(Serialize)]`
- [x] 3.3 Rewrite `http_login.rs` `handle_login_db(db: &mut dyn Database, req: &LoginRequest, ip: &str, config: &LoginConfig) -> (u16, String)` to query `accounts` by `email`
- [x] 3.4 Implement SHA-1 password check: call `transform_to_sha1(password.as_bytes())`, compare to `UNHEX(password)` result from DB (stored as hex string); return error 3 on mismatch
- [x] 3.5 Implement TOTP check: if `accounts.secret` is non-empty, require `token` field; validate against ±1 window using existing `generate_token`; return error 6 on mismatch
- [x] 3.6 Implement session INSERT: generate 16 random bytes via `rand::random::<[u8; 16]>()`, Base64-encode for response, insert raw bytes via `db.execute(INSERT INTO sessions ...)` using `db.escape_blob()`
- [x] 3.7 Implement character list query: `SELECT id, name, level, vocation, lastlogin, sex, looktype, lookhead, lookbody, looklegs, lookfeet, lookaddons FROM players WHERE account_id = ?`, map each row to `Character` struct
- [x] 3.8 Map vocation id to name via `game_data.vocations` (or a passed-in vocation lookup fn); if vocation is not found return `"Unknown"`
- [x] 3.9 Build and serialize full `LoginResponse` JSON: `session` + `playdata` (worlds from config, characters from DB)
- [x] 3.10 Write unit tests for: missing email/password fields, unknown account, wrong password, TOTP required, TOTP wrong window, TOTP correct window, session key is base64, ispremium true/false, character list populated, world array has one entry with correct config values

## 4. Cacheinfo handler — DB-backed player count

- [x] 4.1 Rewrite `http_cacheinfo.rs` `handle_cacheinfo_db(db: &dyn Database) -> (u16, String)` to execute `SELECT COUNT(*) AS count FROM players_online`
- [x] 4.2 Parse the `count` column from the result row; on `DbError` return the error envelope string
- [x] 4.3 Write unit tests for: count = 0, count = N, DB error returns error envelope, HTTP status always 200

## 5. Boot wiring

- [x] 5.1 Add `pub fn start_http_listener(config: Arc<ConfigManager>, db: Arc<Mutex<Box<dyn Database + Send>>>, vocations: Arc<Vocations>) -> Result<(), String>` to `crates/server/src/boot.rs`
- [x] 5.2 Inside `start_http_listener`: read `httpPort` (skip if 0) and `httpWorkers`; bind `TcpListener` on `0.0.0.0:{httpPort}`; spawn `httpWorkers` threads running the accept loop
- [x] 5.3 Each accepted connection calls `HttpConnectionSession::handle(stream)` directly in the worker thread
- [x] 5.4 Add `db: Arc<Mutex<Box<dyn Database + Send>>>` as a field to `Modules` in `crates/tfs/src/boot.rs`; also updated `main.rs` to replace with real DB after `connect_database`
- [x] 5.5 Call `srv_boot::start_http_listener(config.clone(), db.clone(), vocations.clone())` inside `start_listeners()` in `crates/tfs/src/boot.rs`
- [x] 5.6 Write integration test: bind on port 0, send `POST /` with `{"type":"cacheinfo"}`, assert 200 response with `Content-Type: application/json`

## 6. Wire `http.rs` accept_loop (replace drop(stream) stub)

- [x] 6.1 In `crates/server/src/http.rs` `accept_loop`: added `factory: SessionFactory` parameter; replaced `drop(stream)` with spawning a thread calling `factory(stream)`. Also updated `start()` to accept a factory.
- [x] 6.2 Ensure the existing `http.rs` tests still pass after the stub is replaced

## 7. Quality gates

- [x] 7.1 Run `cargo test --lib --workspace` — zero failures
- [x] 7.2 Run `cargo clippy --workspace --lib --tests -- -D warnings` — zero warnings
- [x] 7.3 Run `cargo fmt --all` — no diff
- [ ] 7.4 Run `docker compose up --build`; confirm server logs show HTTP server starting on port 8080
- [ ] 7.5 Run `curl -s http://localhost:8080/` — response starts with `HTTP` or body is `{}`
- [ ] 7.6 Run `curl -s -X POST http://localhost:8080/ -H 'Content-Type: application/json' -d '{"type":"cacheinfo"}'` — response contains `"playersonline"`
- [ ] 7.7 Connect OTClient to the server — login dialog proceeds past the "Failed to connect to server (HTTP)" error (character list loads or shows account-not-found, not a network error)
- [ ] 7.8 Run E2E tests: `cargo test -p forgottenserver-e2e --features e2e -- --test-threads=1` — `http_port_accepts_tcp_connection` and `http_port_returns_200_for_get_request` both pass
