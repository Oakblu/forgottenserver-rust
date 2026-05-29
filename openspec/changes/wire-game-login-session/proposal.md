## Why

The game server TCP listener on port 7172 sends a challenge packet and validates the client version but then drops the connection — the full TFS 13 handshake (RSA decrypt, session token validation, player load, enter-world burst, XTEA game loop) is not implemented. A player who completes HTTP login and selects a character gets ERROR 60 (connection timeout) or an immediate disconnect, and never enters the world.

## What Changes

- Rewrite `GameLoginHandler::handle_connection` in `crates/server/src/boot.rs` to implement the complete TFS 13 `onRecvFirstMessage` wire protocol: read OS + version header bytes, RSA-decrypt the 128-byte login block, extract XTEA key + session token + character name + challenge echo, and validate the echo against the stored challenge.
- Add session token lookup in the `sessions` + `accounts` + `players` DB tables to resolve `character_id` and `account_id`.
- Add player loading from DB (`IOLoginData::loadPlayerById` equivalent) after session validation.
- Implement the enter-world burst: send the sequence of packets the client requires to render the game world (map description, player stats, skills, inventory, VIP list, channel list).
- Implement the per-connection XTEA-encrypted game loop: a read-dispatch loop that XTEA-decrypts each inbound packet and routes it to the handlers in `game_handler.rs`.
- Store `challengeTimestamp` and `challengeRandom` per connection so the client echo can be validated.

## Capabilities

### New Capabilities

- `game-login-handshake`: TFS 13 `onRecvFirstMessage` wire protocol — RSA decrypt, XTEA key negotiation, challenge validation, session token DB lookup, character resolution.
- `enter-world-burst`: The initial sequence of server→client packets sent after a successful login that place the player in the world (map, stats, skills, inventory, channels, VIPs).
- `game-session-loop`: The persistent per-connection XTEA-encrypted packet read-dispatch loop that handles all in-game actions.

### Modified Capabilities

## Impact

- `crates/server/src/boot.rs` — `GameLoginHandler` rewritten; stores per-connection challenge state.
- `crates/network/src/protocolgame.rs` — `parse_login_packet` rewritten to match TFS 13 wire format (OS field, RSA block layout, session token / character name instead of account / password).
- `crates/server/src/game_handler.rs` — `on_enter_game` wired into a real connection dispatch loop.
- `crates/network/src/protocol.rs` — `decrypt_message` / `encrypt_output` XTEA helpers already exist; need to be called per-packet in the game loop.
- `crates/common/src/rsa.rs` — `rsa::decrypt` already exists; must be loaded with the server PEM key at boot.
- `crates/database/` — new query: session token lookup joining `sessions`, `accounts`, `players`.
- No schema changes; the `sessions` table written by the HTTP login handler is the source of truth.
- `docker/config.lua` — `ip` must be set to a routable address (already fixed to `127.0.0.1` for local dev).
