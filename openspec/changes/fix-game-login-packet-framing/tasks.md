## 1. TDD — Write failing tests first

- [x] 1.1 In `crates/server/src/boot.rs` tests, rename `challenge_packet_is_12_bytes_adler32_prefixed` to `challenge_packet_is_14_bytes_with_outer_len` and update assertions: buf is 14 bytes, bytes [0..2] == `[12, 0]` (outer_len = 12), bytes [2..6] == adler32 of bytes [6..14], bytes [6..8] == `[6, 0]` (inner_len), byte [8] == `0x1F`, bytes [9..13] == timestamp, byte [13] == rand.
- [x] 1.2 Add a test `challenge_outer_len_guard_rejects_implausible_length` that connects a raw TCP stream to a `GameLoginHandler`, sends 2 bytes `[0xFF, 0xFF]` (outer_len = 65535) as the client's first packet, and asserts the connection is closed without a panic.
- [x] 1.3 Add an integration test `game_login_challenge_round_trip` using an in-process TCP socket pair: (a) spawn `GameLoginHandler::handle_connection` on the server side; (b) on the client side read 14 bytes, parse outer_len, adler32, inner_len, opcode, timestamp, rand; (c) build and send a well-formed first packet that correctly echoes timestamp + rand with a valid (test-key-encrypted) RSA block and a session token that matches a seeded in-memory DB row; (d) assert the server sends the XTEA-encrypted enter-world burst (not a disconnect packet).

## 2. Fix challenge packet framing

- [x] 2.1 In `crates/server/src/boot.rs` `handle_connection`, change the challenge buffer from 12 bytes to 14 bytes. Layout: `buf[0..2] = 12u16.to_le_bytes()` (outer_len), `buf[2..6]` = adler32 placeholder (filled last), `buf[6..8] = 6u16.to_le_bytes()` (inner_len), `buf[8] = 0x1F` (opcode), `buf[9..13] = timestamp.to_le_bytes()`, `buf[13] = rand_byte`. Then set `buf[2..6] = adler_checksum(&buf[6..14]).to_le_bytes()`. Confirm tests from Task 1.1 pass.

## 3. Add outer_len guard

- [x] 3.1 In `handle_connection`, after reading `outer_len` from the client's first packet (line ~195), add a guard: if `outer_len > 32_768`, log `[game] first packet outer_len={outer_len} exceeds limit — closing connection` and return. This covers the "Forgotten (Rust " scenario. Confirm test from Task 1.2 passes.

## 4. Quality gates

- [x] 4.1 Run `cargo test --lib -p forgottenserver-server` — all tests pass.
- [x] 4.2 Run `cargo clippy --workspace --lib --tests -- -D warnings` — zero warnings.
- [x] 4.3 Run `cargo fmt --all -- --check` — no formatting differences.
- [ ] 4.4 Run `docker compose up --build` — watch logs; confirm the `failed to read packet body` error no longer appears after an HTTP login + game connection attempt. Confirm `>> Forgotten Server Online!` line is still present.
