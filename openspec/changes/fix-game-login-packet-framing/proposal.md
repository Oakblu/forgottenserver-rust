## Why

The game server's challenge packet is missing the mandatory 2-byte outer TCP length prefix, so game clients (otclient/Tibia 13.x) cannot frame the challenge correctly and respond with a garbled or absent login packet. The server then fails to read the client's first packet and logs `failed to read packet body: outer_len=28486`.

## What Changes

- `crates/server/src/boot.rs` — the challenge packet sent on game connection is enlarged from 12 bytes to 14 bytes by prepending `[outer_len:2 = 12]`, matching the C++ `ProtocolGame::onConnect()` wire format produced by `send(output)`.
- The unit test `challenge_packet_is_12_bytes_adler32_prefixed` is corrected to assert 14 bytes with a valid outer_len header.
- A new integration test (in-process TCP socket pair) verifies the full challenge → login-packet round-trip using the otclient protocol flow.

## Capabilities

### New Capabilities

- `game-login-handshake`: The complete game-server handshake: send correctly-framed challenge, read client first packet, validate RSA block + session token + challenge echo, respond with enter-world burst or disconnect.

### Modified Capabilities

<!-- none — no existing spec-level requirements change -->

## Impact

- **`crates/server/src/boot.rs`**: `handle_connection` challenge builder (lines ~161-170) and the unit test at lines ~1272-1302.
- No other files change.
- No public API or Lua binding changes.
- Docker/integration stack: clean game login should now work; the log error disappears.
