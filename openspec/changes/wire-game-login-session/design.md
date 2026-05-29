## Context

Port 7172 currently sends a challenge packet (`0x1F` + timestamp + rand) on connect, reads one packet, calls `parse_login_packet` (which reads OS as `client_version` — wrong field order — and has no RSA decryption), and drops the connection. The handshake is structurally broken for any real Tibia 13 / OTClient connection.

The cryptographic primitives needed are already present:
- `crates/common/src/rsa.rs` — raw RSA-1024 no-padding decrypt (`rsa::decrypt`)
- `crates/common/src/xtea.rs` — XTEA block cipher
- `crates/network/src/protocol.rs` — `decrypt_message` / `encrypt_output` XTEA helpers

The game-play dispatch layer (`game_handler.rs`, `codec.rs`, `game_state.rs`) is implemented and tested. The enter-world primitives (`on_enter_game`, `PlayerStats`, `FullMapDescription`) exist in `codec.rs`. What is missing is the wiring between the TCP connection and all of these layers.

## Goals / Non-Goals

**Goals:**
- Implement the full TFS 13 `onRecvFirstMessage` wire format so OTClient can complete a login.
- Validate the session token from the `sessions` table (written by the HTTP login handler).
- Load the player record from the DB and place them in the world.
- Send the enter-world burst that allows the client to render the game.
- Run a per-connection XTEA-encrypted game loop that dispatches inbound packets.

**Non-Goals:**
- Full gameplay parity (combat results, creature AI, scripting hooks) — the loop only needs to dispatch to existing handlers without crashing.
- Support for classic (pre-12) Tibia clients — only TFS 13 protocol (versions 1310–1311).
- XTEA checksum sequence mode (used by QT clients) — OTClient uses Adler32 mode.
- Ban/IP checks, ghost mode, ALLOW_CLONES — those are future hardening tasks.
- Saving player on logout — the loop can close the connection cleanly; save is a separate task.

## Decisions

### D1 — Keep `GameLoginHandler` synchronous and thread-per-connection

The current model spawns one OS thread per accepted TCP connection. This matches the C++ `boost::asio`-per-connection model and keeps ownership simple: the XTEA `RoundKeys`, challenge state, and player context live on the stack of that thread, no `Arc<Mutex<…>>` needed for session state.

**Alternative considered:** `tokio` async tasks. Rejected because (a) the rest of the server is sync, (b) converting introduces widespread `async` contamination, and (c) thread-per-connection is sufficient at the expected player counts for this port.

### D2 — Store challenge state in `GameLoginHandler` fields

The challenge timestamp and random number are generated in `handle_connection` when the challenge packet is sent. They must be validated against the echo in the client's first packet. These two `u32`/`u8` values live as fields on `GameLoginHandler` — or better, as locals passed into `parse_first_packet`.

**Alternative considered:** global `HashMap<peer_addr, challenge>`. Rejected — unnecessary indirection when the values can be passed directly through the call stack.

### D3 — Rewrite `parse_login_packet` to match TFS 13 wire exactly

The current function reads `client_version` as the first u16, but the wire starts with `OS` (u16). The function must be rewritten (and renamed `parse_first_packet` or given a new signature) to:
1. Read OS (u16) — store, used to enable extended opcodes for OTClient
2. Read version (u16) — range-check [1310, 1311]
3. Skip 4 bytes (client build u32)
4. If version >= 1240 and buffer has room: skip string (client version string)
5. Skip 3 bytes (dat revision u16 + preview state u8)
6. Call `rsa::decrypt` on the next 128 bytes in-place
7. Read XTEA key (4 × u32) from decrypted block; byte 0 must be 0x00
8. Skip 1 byte (gm_flag)
9. Read session_token as length-prefixed string; base64-decode it
10. Read character_name as length-prefixed string
11. Read challenge echo: timestamp (u32) + random (u8)
12. Validate echo against stored challenge values

**Alternative considered:** Keeping the old function and adding a "TFS 13 wrapper". Rejected — the old function is wrong at the byte level; a clean rewrite with the correct field order is less error-prone.

### D4 — Session token lookup: single JOIN query

Validate session token + character name in one query (mirrors C++ line 437):
```sql
SELECT a.id AS account_id, p.id AS character_id, p.name
FROM accounts a
JOIN sessions s ON a.id = s.account_id
JOIN players p ON a.id = p.account_id
WHERE s.token = <escaped_blob>
  AND s.expired_at IS NULL
  AND p.name = <escaped_string>
  AND p.deletion = 0
```
This avoids a round-trip between session lookup and player lookup.

### D5 — Enter-world burst: fixed minimum set

Send only the packets required for the client to render:
1. `0x0A` pending-state byte
2. `0x64` FullMapDescription (18×14 viewport around login position)
3. `0xA0` PlayerStats (HP, mana, level, stamina)

The client will show the world with this minimum set. Skills (`0x8D`), inventory (`0x78`), VIP list, channel list are follow-on tasks; their absence causes missing UI panels but not a crash.

**Alternative considered:** Full burst matching C++ exactly. Deferred — the C++ burst is 20+ packet types requiring full inventory and condition state. The minimum set gets the player visible.

### D6 — XTEA mode: Adler32 checksum (not sequence numbers)

OTClient uses Adler32 checksum mode (not QT sequence number mode). `protocol.rs` already has `validate_adler32` / `stamp_adler32`. The game loop uses these for every inbound packet after XTEA decrypt.

### D7 — RSA key loaded at boot, not per-connection

`rsa::load_pem` uses a `OnceLock` — call it once at server startup (in `tfs/src/main.rs` or `boot.rs`) with the key from `key.pem` (or the embedded default). Each `handle_connection` call just calls `rsa::decrypt` which reads the global key.

The default OTLand RSA private key (used by OTClient out of the box) is already embedded in `crates/perf-bot/src/client.rs` (`SERVER_KEY_PEM`). Move it to `crates/common/src/rsa.rs` as a `DEFAULT_KEY_PEM` constant, or read from `docker/key.pem`.

## Risks / Trade-offs

- **[Risk] Map is empty at login** → The `World` struct loaded at boot may have no tiles populated if the `.otbm` map loader is not run. The map description will be all empty tiles; the client renders a black void but doesn't crash. Mitigation: load the `forgotten.otbm` map at boot before starting listeners.
- **[Risk] Player has no position** → If the DB player row has `posx/posy/posz = 0,0,0`, the login position is invalid and `placeCreature` would fail. Mitigation: fall back to the temple position from `config.lua` (read from `ConfigManager`).
- **[Risk] RSA decrypt with wrong key** → If the embedded key doesn't match what OTClient uses, the decrypted block is garbage and the session token parse fails, causing a disconnect. Mitigation: use the well-known OTLand default key (same one in `perf-bot/src/client.rs`).
- **[Risk] Thread starvation** → A slow or malicious client can hold a thread indefinitely. Mitigation: add a per-connection read timeout (same 30 s pattern as HTTP handler) and a write timeout.

## Open Questions

- Should the enter-world burst include skills and inventory in this change, or is the minimum (map + stats) sufficient to close task 7.7?
- Should `rsa::load_pem` be called from `tfs/src/main.rs` or from `start_game_listener`? (The key is global so either works; `main.rs` is cleaner.)
- After the player enters, should we write `lastlogintime` to the DB immediately (like C++ does) or defer to a future task?

---

## Defect: Challenge packet Adler32 framing (discovered post-implementation)

### Root cause

Port 7172 sends a challenge but OTClient connects and sends nothing back — `[game]` log lines never appear. The bug is in the wire format of the challenge packet and all subsequent server→client packets.

**C++ `NetworkMessage` buffer layout (from `networkmessage.h`):**
```
[0..2)  outer length (HEADER_LENGTH = 2)
[2..6)  Adler32 checksum (CHECKSUM_LENGTH = 4)
[6..8)  inner / encrypted length
[8..)   payload (INITIAL_BUFFER_POSITION = 8)
```

The C++ `OutputMessage::add_header(T)` prepends headers backwards from position 8, so `addCryptoHeader(CHECKSUM_ADLER)` produces `[outer_len][Adler32]` before the payload. The Rust `OutputMessage` has no equivalent — only `write_message_length()`.

### Challenge wire format

Two different framing styles are used in TFS:

**Challenge (NO outer TCP length — OTClient reads a fixed 12-byte frame):**
```
[4-byte Adler32 over bytes 4..12] [2-byte inner_len = 0x0006] [0x1F] [4-byte ts] [1-byte rand]
```
C++ source (`protocolgame.cpp` `sendChallenge`):
1. `skipBytes(4)` — reserve 4 bytes for Adler32
2. `add<uint16_t>(0x0006)` — inner_length = 6 (opcode + ts + rand)
3. `addByte(0x1F)` + `add<uint32_t>(ts)` + `addByte(rand)`
4. `skipBytes(-12)` — seek back to start
5. `add<uint32_t>(adlerChecksum(buf + 4, 8))` — Adler32 over bytes 4..12
6. `send(output)` — sends the 12 raw bytes directly (no TCP length prefix)

**Normal game packets:**
```
[2-byte outer_len (= 4 + payload_len)] [4-byte Adler32 over payload] [payload]
```
C++ uses `addCryptoHeader(CHECKSUM_ADLER)` which backwards-prepends `[adler32][outer_len]`.

### What the Rust code sends (wrong)

```
// boot.rs:149-153
let mut challenge = OutputMessage::new();
challenge.add_u8(0x1F);
challenge.add_u32(timestamp);
challenge.add_u8(rand_byte);
challenge.write_message_length();
stream.write_all(challenge.get_output_buffer())
// → sends [0x06, 0x00, 0x1F, ts0, ts1, ts2, ts3, rand] = 8 bytes
```

OTClient reads 4 bytes as Adler32 (`0x001F0006`), then 2 bytes as inner_length (timestamp high bytes ≈ 50 000+), then tries to read ~50 000 more bytes — it times out or silently drops the connection.

### Affected locations (in priority order)

| Location | Bug | Fix |
|---|---|---|
| `boot.rs:149-154` | Challenge: uses `OutputMessage` + `write_message_length()` | Build raw 12-byte buffer: `[Adler32(4)][0x0006(2)][0x1F][ts(4)][rand(1)]` |
| `OutputMessage` (`common/src/outputmessage.rs`) | No `add_crypto_header` method | Add `add_crypto_header(mode)` mirroring C++ `addCryptoHeader` |
| All outbound packets (burst + game loop) | `encode()` / `serialize_*()` return raw payload; callers add no Adler32 | Wrap with `[outer_len][Adler32][payload]` before writing to socket |
| `boot.rs:181-185` | Disconnect packet missing Adler32 | Apply same `add_crypto_header` framing |

### D8 — Challenge packet: build manually, not via `OutputMessage`

`OutputMessage` is not the right abstraction for the challenge because:
- The challenge has no outer TCP length prefix (OTClient reads a fixed 12-byte frame)
- The C++ code writes forward with `skipBytes` then patches the checksum in-place

Fix: build a `[u8; 12]` array directly in `boot.rs`:
```
buf[0..4]  = Adler32 of buf[4..12]
buf[4..6]  = 0x0006 (inner_length)
buf[6]     = 0x1F
buf[7..11] = timestamp.to_le_bytes()
buf[11]    = rand_byte
```

### D9 — `OutputMessage::add_crypto_header(mode)` for game packets

Normal game packets need `[outer_len][Adler32][payload]`. Add to `OutputMessage`:
```rust
pub fn add_crypto_header(&mut self) {
    // Compute Adler32 over [2..write_pos) (the payload)
    // Prepend 4-byte checksum then update the 2-byte length field to include it
}
```
This should be called instead of `write_message_length()` whenever writing a packet
that OTClient will validate (all game loop outbound packets + burst + disconnect).

### D10 — Outer_len value

The C++ `outer_len` field covers `[Adler32(4) + payload(N)]`, not just the payload:
```
outer_len = 4 + N   // N = payload bytes after the Adler32
```
OTClient reads `outer_len` bytes after the 2-byte header, which includes the Adler32 + payload.
The existing Rust `write_message_length()` writes `N` (payload only) — this must change to `4 + N` when crypto framing is applied.
