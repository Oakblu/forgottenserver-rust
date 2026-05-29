## 1. Boot — RSA key loading

- [x] 1.1 Move the default OTLand RSA private key PEM from `crates/perf-bot/src/client.rs` into `crates/common/src/rsa.rs` as `pub const DEFAULT_KEY_PEM: &str = "..."` 
- [x] 1.2 In `crates/tfs/src/main.rs` (or `crates/server/src/boot.rs`), call `forgottenserver_common::rsa::load_pem(rsa::DEFAULT_KEY_PEM)` before binding any listeners; return `Err` if it fails
- [x] 1.3 Write unit test: `rsa_default_key_loads_without_error` — call `load_pem(DEFAULT_KEY_PEM)` and assert `Ok(())`

## 2. Protocol — Rewrite `parse_login_packet` for TFS 13 wire format

- [x] 2.1 Rename `parse_login_packet` → `parse_first_packet`; update call sites in `boot.rs` and tests
- [x] 2.2 Change the return type to a new `FirstPacket` struct: `{ os: u16, xtea_key: [u32;4], session_token: Vec<u8>, character_name: String, challenge_timestamp: u32, challenge_random: u8 }`
- [x] 2.3 Implement the field-by-field read order matching C++ `onRecvFirstMessage`: OS (u16), version (u16, range-check), skip 4 bytes build, optional version string skip for ≥ 1240, skip 3 bytes dat+preview, RSA decrypt 128 bytes, XTEA key from decrypted [1..17), skip gm_flag, session_token string + base64-decode, character_name string, timestamp (u32), random (u8)
- [x] 2.4 Return `Err(String)` (for disconnect messages) when: version out of range, RSA decrypted[0] ≠ 0x00, session_token empty after base64 decode
- [x] 2.5 Write unit tests covering: valid TFS-13 packet accepted, version 1098 rejected, bad RSA block rejected (first byte non-zero after "decrypt"), empty session token rejected

## 3. GameLoginHandler — Store challenge + validate echo

- [x] 3.1 In `GameLoginHandler::handle_connection` (boot.rs), after generating and sending the challenge packet, store `challenge_timestamp: u32` and `challenge_random: u8` as local variables
- [x] 3.2 After calling `parse_first_packet`, compare the returned `challenge_timestamp` and `challenge_random` against the stored values; send disconnect and return on mismatch
- [x] 3.3 Write unit test: `challenge_echo_mismatch_disconnects` — build a packet with wrong echo values and assert the handler sends a disconnect payload

## 4. Database — Session token lookup

- [x] 4.1 Add `fn lookup_session(db: &dyn Database, token_blob: &[u8], character_name: &str) -> Option<(i64, i64)>` (returns `(account_id, character_id)`) to `crates/database/src/iologindata.rs`
- [x] 4.2 Implement using the JOIN query from design.md D4; use `db.escape_blob(token_blob)` and `db.escape_string(character_name)`
- [x] 4.3 Write unit tests for `lookup_session`: valid token+name returns `Some`, wrong name returns `None`, expired session returns `None`, unknown token returns `None`

## 5. Database — Load player row for login

- [x] 5.1 Add `fn load_player_for_login(db: &dyn Database, character_id: i64) -> Option<PlayerLoginData>` where `PlayerLoginData` holds `{ name, level, health, healthmax, mana, manamax, stamina, posx, posy, posz }`
- [x] 5.2 Implement: `SELECT name, level, health, healthmax, mana, manamax, stamina, posx, posy, posz FROM players WHERE id = {character_id}`
- [x] 5.3 Return temple-position fallback coords when `posx = posy = posz = 0` (read `ConfigManager` values `TempleX/Y/Z`)
- [x] 5.4 Write unit tests: valid character_id returns data, unknown character_id returns `None`, zero position triggers fallback

## 6. Enter-world burst

- [x] 6.1 In `crates/server/src/game_handler.rs`, add `pub fn build_enter_world_burst(player: &PlayerLoginData, world: &World) -> Vec<u8>` that returns the concatenation of: `[0x0A]` byte, `on_enter_game(world, player_pos)` bytes, `encode(&ServerPacket::PlayerStats { … })` bytes
- [x] 6.2 In `GameLoginHandler::handle_connection`, after loading the player row, call `build_enter_world_burst` and write the result to the TCP stream before enabling XTEA
- [x] 6.3 Write unit test: `enter_world_burst_starts_with_0x0A_then_map_then_stats` — assert burst[0] == 0x0A, burst[1] == 0x64 (map opcode), and that a `0xA0` stats byte appears after the map payload

## 7. XTEA-encrypted game loop

- [x] 7.1 After flushing the burst, set a 30-second read timeout on the stream (`stream.set_read_timeout(Some(Duration::from_secs(30)))`)
- [x] 7.2 Implement the read loop: read 2-byte outer length, read `outer_len` bytes, call `decrypt_message(&mut msg, &xtea_key)`, call `validate_adler32`; drop and continue on checksum failure
- [x] 7.3 Extract inner length and opcode from the decrypted message; dispatch to stubs in `game_handler.rs` for opcodes: `0x65` walk, `0x96` say, `0xBE` use item — log unknown opcodes with `eprintln!` but do not disconnect
- [x] 7.4 On any `read_exact` error (including timeout) exit the loop and close the connection cleanly
- [x] 7.5 Write unit tests for the XTEA round-trip: `game_loop_decrypts_walk_packet_and_dispatches` — encrypt a walk packet with known key, feed to the loop via a `TcpListener` pair, assert the handler fires

## 8. Wire everything in `GameLoginHandler`

- [x] 8.1 Replace the body of `GameLoginHandler::handle_connection` in `boot.rs` with the full sequence: challenge → parse_first_packet → validate echo → lookup_session → load_player_for_login → build_enter_world_burst → flush → game_loop
- [x] 8.2 Each failure path (parse error, echo mismatch, session not found, player not found) MUST send the appropriate `serialize_disconnect` message before returning

## 9. Fix: Adler32 packet framing (challenge + all outbound)

_Discovered after tasks 1–8 were marked done. OTClient connects to 7172 but sends nothing back — the challenge wire format is wrong (missing Adler32 + wrong structure). See design.md §Defect._

- [x] 9.1 In `boot.rs` `GameLoginHandler::handle_connection`, replace the `OutputMessage`-based challenge with a raw `[u8; 12]` build: `[Adler32(4)][0x0006(2)][0x1F][timestamp_le(4)][rand(1)]` using `adler_checksum` from `forgottenserver_common::tools`. Send all 12 bytes with no TCP length prefix.
- [x] 9.2 Write unit test `challenge_packet_is_12_bytes_adler32_prefixed`: build the challenge bytes, parse the first 4 bytes as Adler32, verify it matches `adler_checksum(&buf[4..12])`, verify buf[4..6] == `[0x06, 0x00]`, buf[6] == `0x1F`.
- [x] 9.3 Add `pub fn add_crypto_header(&mut self)` to `OutputMessage` (`crates/common/src/outputmessage.rs`): compute Adler32 over the payload (`buffer[HEADER_LENGTH..write_pos]`), write it into bytes `[HEADER_LENGTH..HEADER_LENGTH+4]` (shifting the payload right first or reserving 4 extra bytes ahead of the write cursor), then write `outer_len = 4 + payload_len` as the 2-byte header. The resulting `get_output_buffer()` slice must be `[outer_len(2)][adler32(4)][payload(N)]`.
- [x] 9.4 Write unit tests for `add_crypto_header`: verify `get_output_buffer()[0..2]` == `(4 + payload_len).to_le_bytes()`, verify Adler32 at `[2..6]` matches `adler_checksum(payload)`, verify payload bytes after offset 6 match what was written.
- [x] 9.5 Replace all `write_message_length()` + socket write calls that send to OTClient with `add_crypto_header()`: the disconnect packet (`boot.rs:181-185`) and the enter-world burst (`build_enter_world_burst` output in `game_handler.rs`).
- [x] 9.6 In the XTEA game loop (task 7.2), outbound encrypted packets must also be framed with `add_crypto_header()` after `encrypt_output` (since XTEA encrypt already calls `write_message_length`; replace that call with `add_crypto_header`).

## 10. Quality gates

- [x] 10.1 Run `cargo test --lib --workspace` — zero failures
- [x] 10.2 Run `cargo clippy --workspace --lib --tests -- -D warnings` — zero warnings
- [x] 10.3 Run `cargo fmt --all` — no diff
- [ ] 10.4 Rebuild Docker: `docker compose down && docker compose up --build` — server logs show HTTP + game listeners starting
- [ ] 10.5 Connect OTClient: complete login, select character — client renders the game world (map visible, player stats shown, no ERROR 60 or disconnect)
