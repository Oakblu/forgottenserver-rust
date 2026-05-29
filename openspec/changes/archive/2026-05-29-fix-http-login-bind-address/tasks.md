## 1. ConfigManager Extension

- [x] 1.1 Add `HttpBindAddress` variant to `StringKey` enum in `crates/common/src/configmanager.rs`
- [x] 1.2 Add `"httpLoginBindAddress"` → `StringKey::HttpBindAddress` mapping in `ConfigManager::string_key()`
- [x] 1.3 Add a unit test in `configmanager.rs` that verifies `"httpLoginBindAddress"` maps to `StringKey::HttpBindAddress` and round-trips via `set_string` / `get_string`

## 2. Boot Listener Fix

- [x] 2.1 In `start_http_listener` (`crates/server/src/boot.rs`), read `config.get_string(StringKey::HttpBindAddress)` and apply the empty-string fallback to `"127.0.0.1"`
- [x] 2.2 Replace `TcpListener::bind(format!("0.0.0.0:{http_port}"))` with `TcpListener::bind(format!("{bind_addr}:{http_port}"))`
- [x] 2.3 Update the startup `eprintln!` to include the bind address: `>> HTTP login server online on {bind_addr}:{http_port} ({workers} worker(s)).`

## 3. Fixture Config Updates

- [x] 3.1 Add `httpLoginBindAddress = "127.0.0.1"` to `docker/config.lua`
- [x] 3.2 Add `httpLoginBindAddress = "127.0.0.1"` to `crates/tfs/tests/fixtures/config.lua`

## 4. Tests

- [x] 4.1 Add a unit test in `boot.rs` that verifies `start_http_listener` binds to `127.0.0.1` when `HttpBindAddress` config is empty (use a real `TcpListener::bind` attempt on a free port to confirm the address)
- [x] 4.2 Add a unit test that verifies `start_http_listener` binds to `0.0.0.0` when `HttpBindAddress` is set to `"0.0.0.0"` in config
- [x] 4.3 Add a unit test that verifies an empty `httpLoginBindAddress` value falls back to `127.0.0.1` (not `""`-colon-port)

## 5. Spec Archive Sync

- [x] 5.1 Run `make ledger` and confirm no new ledger gaps are introduced by the `StringKey::HttpBindAddress` addition
- [x] 5.2 Run `cargo test --lib --workspace` and confirm all tests pass
- [x] 5.3 Run `cargo clippy --workspace --lib --tests -- -D warnings` and confirm zero warnings
