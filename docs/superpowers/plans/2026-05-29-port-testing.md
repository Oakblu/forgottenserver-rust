# Port Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify all server ports (7171 status, 7172 game, 8080 HTTP) respond correctly, fix behavioral gaps found during audit, and establish a complete test pyramid (in-process unit → in-process integration → Docker e2e).

**Architecture:** Three sequential tracks. Track 3 fixes known gaps first (version enforcement, disconnect packet, status XML version, Lua CLIENT_VERSION, game TCP listener, e2e DB seeding). Track 1 adds in-process protocol tests using `Cursor`-based mock streams. Track 2 adds Docker-based e2e behavioral tests asserting actual wire responses.

**Tech Stack:** Rust (stable), mlua, testcontainers 0.20, tokio, MariaDB 11.

---

## Discovery notes (read before starting)

- `CLIENT_VERSION_MIN = 1310`, `CLIENT_VERSION_MAX = 1311`, `CLIENT_VERSION_STR = "13.10"` are already in `crates/common/src/definitions.rs` — import them don't redefine them.
- Disconnect packet opcode is `0x14` + length-prefixed string (confirmed from `forgottenserver-upstream/src/protocolgame.cpp`).
- The game TCP listener is **not yet started**. `tfs/src/boot.rs:start_listeners` only calls `start_admin_and_status`. Game port (7172) is unbound.
- `status_handler.rs:build_xml` has `version="860"` and `client="860"` hardcoded — should be `CLIENT_VERSION_STR = "13.10"`.
- TFS 13.10 game login uses session tokens + challenge/response (not account/password). Full session token auth is deferred; this plan scopes game handler to: bind port, read packet, reject wrong version, close (charlist requires session token migration — future sprint).
- `LoginDb::players_by_account(account_id)` already exists and returns `Vec<&PlayerRecord>`.
- `OutputMessage` buffer layout: `buffer[0..2)` = 2-byte LE length written by `write_message_length()`; `get_output_buffer()` returns the full buffer including the length prefix.
- `serialize_*` functions in `protocolgame.rs` return payload bytes **without** the 2-byte length prefix (consistent with wire parity test convention).

---

## File map

**New files:**
- `crates/network/tests/protocol_integration.rs` — in-process protocol integration tests
- `crates/e2e/tests/common/seed.rs` — DB seeding helper
- `crates/e2e/tests/http.rs` — HTTP port e2e test

**Modified files:**
- `crates/network/src/protocolgame.rs` — add `serialize_disconnect`, add version check in `parse_login_packet`
- `crates/server/src/status_handler.rs` — fix `version="860"` and `client="860"` in XML
- `crates/server/src/boot.rs` — add `GameLoginHandler` + `start_game_listener`
- `crates/tfs/src/boot.rs` — call `start_game_listener` in `start_listeners`
- `crates/scripting/src/lua_bindings/misc_globals.rs` — add `CLIENT_VERSION` Lua table
- `crates/e2e/tests/common/mod.rs` — add port 8080 mapping, `http_port()` accessor, call `seed_db()`
- `crates/e2e/tests/game.rs` — remove weak test, add behavioral tests
- `crates/e2e/tests/status.rs` — strengthen existing tests

---

## TRACK 3 — Stub scan and fix

### Task 1: Add `serialize_disconnect` to the network crate

**Files:**
- Modify: `crates/network/src/protocolgame.rs` (after line ~162, before `parse_walk_packet`)
- Test: inline `#[cfg(test)]` mod in `crates/network/src/protocolgame.rs`

- [ ] **Step 1.1: Write the failing test** — add to the `#[cfg(test)]` block at the bottom of `crates/network/src/protocolgame.rs`:

```rust
#[test]
fn serialize_disconnect_opcode_is_0x14() {
    let bytes = serialize_disconnect("hello");
    assert_eq!(bytes[0], 0x14, "disconnect opcode must be 0x14");
}

#[test]
fn serialize_disconnect_includes_message_as_length_prefixed_string() {
    let msg = "Account name or password is not correct.";
    let bytes = serialize_disconnect(msg);
    // bytes[0] = 0x14 opcode
    // bytes[1..3] = u16 LE string length
    // bytes[3..] = string bytes
    let len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    assert_eq!(len, msg.len());
    assert_eq!(&bytes[3..3 + len], msg.as_bytes());
}

#[test]
fn serialize_disconnect_empty_message_produces_3_bytes() {
    let bytes = serialize_disconnect("");
    // opcode (1) + length u16 (2) + string (0) = 3 bytes
    assert_eq!(bytes.len(), 3);
    assert_eq!(bytes[0], 0x14);
    assert_eq!(u16::from_le_bytes([bytes[1], bytes[2]]), 0);
}
```

- [ ] **Step 1.2: Run tests to see them fail**

```bash
cargo test --lib -p forgottenserver-network serialize_disconnect
```
Expected: FAIL — `error[E0425]: cannot find function 'serialize_disconnect'`

- [ ] **Step 1.3: Add the `serialize_disconnect` function**

In `crates/network/src/protocolgame.rs`, add after the `serialize_character_list` function (around line 162):

```rust
/// Serialize a disconnect packet sent to the client.
///
/// Wire format (opcode 0x14 — mirrors `ProtocolGame::disconnectClient`):
/// - opcode `0x14` (u8)
/// - message (length-prefixed string)
///
/// Mirrors C++:
///   output->addByte(0x14);
///   output->addString(message);
pub fn serialize_disconnect(message: &str) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x14);
    out.add_string(message);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}
```

- [ ] **Step 1.4: Run tests to confirm they pass**

```bash
cargo test --lib -p forgottenserver-network serialize_disconnect
```
Expected: all 3 tests PASS

- [ ] **Step 1.5: Lint**

```bash
cargo clippy -p forgottenserver-network --lib --tests -- -D warnings
```
Expected: zero warnings

- [ ] **Step 1.6: Commit** (ask user first per CLAUDE.md Git rules)

---

### Task 2: Add version range check to `parse_login_packet`

**Files:**
- Modify: `crates/network/src/protocolgame.rs` (function `parse_login_packet`, ~line 127)

- [ ] **Step 2.1: Write the failing tests** — add to the `#[cfg(test)]` block:

```rust
#[test]
fn parse_login_version_760_returns_err_with_version_message() {
    use forgottenserver_common::networkmessage::NetworkMessage;
    // version=760 (0xF8, 0x02 LE) + xtea key (16 bytes) + account "" + password ""
    let payload: &[u8] = &[
        0xF8, 0x02,              // version 760
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, // xtea key (all zeros)
        0x00, 0x00,              // account_name length = 0
        0x00, 0x00,              // password length = 0
    ];
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload);
    msg.set_buffer_position(0);
    let result = parse_login_packet(&mut msg);
    assert!(result.is_err(), "version 760 must be rejected");
    let err = result.unwrap_err();
    assert!(
        err.contains("13.10"),
        "error must mention the allowed version: {err}"
    );
}

#[test]
fn parse_login_version_9999_returns_err_with_version_message() {
    use forgottenserver_common::networkmessage::NetworkMessage;
    // version=9999 (0x0F, 0x27 LE)
    let payload: &[u8] = &[
        0x0F, 0x27,              // version 9999
        0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
        0x00, 0x00,
        0x00, 0x00,
    ];
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload);
    msg.set_buffer_position(0);
    let result = parse_login_packet(&mut msg);
    assert!(result.is_err(), "version 9999 must be rejected");
    assert!(result.unwrap_err().contains("13.10"));
}

#[test]
fn parse_login_version_1310_returns_ok() {
    use forgottenserver_common::networkmessage::NetworkMessage;
    // version=1310 (0x1E, 0x05 LE)
    let payload: &[u8] = &[
        0x1E, 0x05,              // version 1310
        1,0,0,0, 2,0,0,0, 3,0,0,0, 4,0,0,0, // xtea key
        0x03, 0x00, b'a', b'c', b'c', // account_name "acc"
        0x04, 0x00, b'p', b'a', b's', b's', // password "pass"
    ];
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload);
    msg.set_buffer_position(0);
    let result = parse_login_packet(&mut msg);
    assert!(result.is_ok(), "version 1310 must be accepted: {:?}", result);
    let packet = result.unwrap();
    assert_eq!(packet.client_version, 1310);
}

#[test]
fn parse_login_version_1311_returns_ok() {
    use forgottenserver_common::networkmessage::NetworkMessage;
    // version=1311 (0x1F, 0x05 LE)
    let payload: &[u8] = &[
        0x1F, 0x05,              // version 1311
        1,0,0,0, 2,0,0,0, 3,0,0,0, 4,0,0,0,
        0x00, 0x00,
        0x00, 0x00,
    ];
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload);
    msg.set_buffer_position(0);
    let result = parse_login_packet(&mut msg);
    assert!(result.is_ok(), "version 1311 must be accepted: {:?}", result);
    assert_eq!(result.unwrap().client_version, 1311);
}
```

- [ ] **Step 2.2: Run tests to see them fail**

```bash
cargo test --lib -p forgottenserver-network parse_login_version
```
Expected: FAIL — version 760 and 9999 tests fail (currently return Ok)

- [ ] **Step 2.3: Add import and version check to `parse_login_packet`**

In `crates/network/src/protocolgame.rs`, add the import at the top:

```rust
use forgottenserver_common::definitions::{
    CLIENT_VERSION_MAX, CLIENT_VERSION_MIN, CLIENT_VERSION_STR,
};
```

Then modify `parse_login_packet` to add a version check immediately after reading `client_version`:

```rust
pub fn parse_login_packet(msg: &mut NetworkMessage) -> Result<LoginPacket, String> {
    let client_version = msg.get_u16();

    if (client_version as i32) < CLIENT_VERSION_MIN
        || (client_version as i32) > CLIENT_VERSION_MAX
    {
        return Err(format!(
            "Only clients with protocol {} allowed!",
            CLIENT_VERSION_STR
        ));
    }

    let k0 = msg.get_u32();
    let k1 = msg.get_u32();
    let k2 = msg.get_u32();
    let k3 = msg.get_u32();
    let account_name = msg.get_string(0);
    let password = msg.get_string(0);

    if msg.is_overrun() {
        return Err("login packet overrun".into());
    }

    Ok(LoginPacket {
        account_name,
        password,
        client_version,
        xtea_key: [k0, k1, k2, k3],
    })
}
```

- [ ] **Step 2.4: Run tests to confirm they pass**

```bash
cargo test --lib -p forgottenserver-network parse_login_version
```
Expected: all 4 tests PASS

- [ ] **Step 2.5: Run full workspace tests**

```bash
cargo test --lib --workspace
```
Expected: PASS (no regressions)

- [ ] **Step 2.6: Lint**

```bash
cargo clippy --workspace --lib --tests -- -D warnings
```
Expected: zero warnings

- [ ] **Step 2.7: Commit** (ask user first)

---

### Task 3: Fix status handler XML version fields

**Files:**
- Modify: `crates/server/src/status_handler.rs` (function `build_xml`, ~line 160)

- [ ] **Step 3.1: Write the failing test** — add to the `#[cfg(test)]` block in `crates/server/src/status_handler.rs`:

```rust
#[test]
fn build_xml_version_field_matches_client_version_str() {
    use std::sync::{Arc, Mutex};
    use forgottenserver_common::configmanager::ConfigManager;
    use crate::game_state::GameState;

    let gs = Arc::new(Mutex::new(GameState::new()));
    let config = Arc::new(ConfigManager::new());
    let handler = StatusHandler::new(gs, config);
    let xml = handler.build_xml();

    // Must NOT contain the wrong hardcoded version
    assert!(
        !xml.contains(r#"version="860""#),
        "XML must not contain hardcoded version 860"
    );
    assert!(
        !xml.contains(r#"client="860""#),
        "XML must not contain hardcoded client 860"
    );

    // Must contain the correct version string from definitions
    assert!(
        xml.contains(r#"version="13.10""#),
        "XML version must be 13.10: {xml}"
    );
    assert!(
        xml.contains(r#"client="13.10""#),
        "XML client must be 13.10: {xml}"
    );
}
```

- [ ] **Step 3.2: Run test to see it fail**

```bash
cargo test --lib -p forgottenserver-server build_xml_version_field
```
Expected: FAIL — XML contains `version="860"` instead of `version="13.10"`

- [ ] **Step 3.3: Add import and fix the XML template**

In `crates/server/src/status_handler.rs`, add to the existing imports:

```rust
use forgottenserver_common::definitions::CLIENT_VERSION_STR;
```

Then in `build_xml`, replace the hardcoded values:

```rust
// BEFORE:
r#"<serverinfo uptime="{uptime}" ip="{ip}" servername="{server_name}" port="{game_port}" version="860" client="860"/>"#

// AFTER:
r#"<serverinfo uptime="{uptime}" ip="{ip}" servername="{server_name}" port="{game_port}" version="{CLIENT_VERSION_STR}" client="{CLIENT_VERSION_STR}"/>"#
```

The complete fixed `build_xml` function format string:
```rust
format!(
    r#"<?xml version="1.0"?>
<tsqp version="1.0">
<serverinfo uptime="{uptime}" ip="{ip}" servername="{server_name}" port="{game_port}" version="{CLIENT_VERSION_STR}" client="{CLIENT_VERSION_STR}"/>
<owner name="{owner_name}" email="{owner_email}"/>
<players online="{online}" max="{max_players}" peak="{peak}"/>
<motd>{motd}</motd>
</tsqp>"#
)
```

Note: Rust's `format!` macro interprets `{CLIENT_VERSION_STR}` as a named argument. You must add `CLIENT_VERSION_STR` as a named argument in the format call or interpolate via a local variable. The correct approach:

```rust
let version_str = CLIENT_VERSION_STR;
format!(
    r#"<?xml version="1.0"?>
<tsqp version="1.0">
<serverinfo uptime="{uptime}" ip="{ip}" servername="{server_name}" port="{game_port}" version="{version_str}" client="{version_str}"/>
<owner name="{owner_name}" email="{owner_email}"/>
<players online="{online}" max="{max_players}" peak="{peak}"/>
<motd>{motd}</motd>
</tsqp>"#
)
```

- [ ] **Step 3.4: Run test to confirm it passes**

```bash
cargo test --lib -p forgottenserver-server build_xml_version_field
```
Expected: PASS

- [ ] **Step 3.5: Run full workspace tests + lint**

```bash
cargo test --lib --workspace && cargo clippy --workspace --lib --tests -- -D warnings
```
Expected: PASS, zero warnings

- [ ] **Step 3.6: Commit** (ask user first)

---

### Task 4: Add `CLIENT_VERSION` Lua global

**Files:**
- Modify: `crates/scripting/src/lua_bindings/misc_globals.rs` (at the end of `install()`, before `Ok(())`)

The C++ reference (`forgottenserver-upstream/src/luascript.cpp:5134-5136`):
```cpp
setField(L, "min", CLIENT_VERSION_MIN);
setField(L, "max", CLIENT_VERSION_MAX);
setField(L, "string", CLIENT_VERSION_STR);
```
This creates a global Lua table `CLIENT_VERSION` with fields `min`, `max`, and `string`.

- [ ] **Step 4.1: Write the failing test** — add to `#[cfg(test)]` block in `misc_globals.rs`:

```rust
#[test]
fn client_version_table_has_min_field() {
    let lua = fresh_lua();
    let min: i64 = lua.load("return CLIENT_VERSION.min").eval().unwrap();
    assert_eq!(min, 1310, "CLIENT_VERSION.min must be 1310");
}

#[test]
fn client_version_table_has_max_field() {
    let lua = fresh_lua();
    let max: i64 = lua.load("return CLIENT_VERSION.max").eval().unwrap();
    assert_eq!(max, 1311, "CLIENT_VERSION.max must be 1311");
}

#[test]
fn client_version_table_has_string_field() {
    let lua = fresh_lua();
    let s: String = lua.load("return CLIENT_VERSION.string").eval().unwrap();
    assert_eq!(s, "13.10", "CLIENT_VERSION.string must be '13.10'");
}

#[test]
fn client_version_table_is_a_table() {
    let lua = fresh_lua();
    let is_table: bool = lua
        .load(r#"return type(CLIENT_VERSION) == "table""#)
        .eval()
        .unwrap();
    assert!(is_table, "CLIENT_VERSION must be a table");
}
```

- [ ] **Step 4.2: Run tests to see them fail**

```bash
cargo test --lib -p forgottenserver-scripting client_version_table
```
Expected: FAIL — `CLIENT_VERSION` is nil

- [ ] **Step 4.3: Add the `CLIENT_VERSION` table to `install()`**

In `crates/scripting/src/lua_bindings/misc_globals.rs`, add at the top of the file (after `use mlua::Value;`):

```rust
use forgottenserver_common::definitions::{
    CLIENT_VERSION_MAX, CLIENT_VERSION_MIN, CLIENT_VERSION_STR,
};
```

Then add inside the `install()` function, before `Ok(())`:

```rust
// ── CLIENT_VERSION — mirrors C++ luascript.cpp:5134-5136 ───────────────
// setField(L, "min", CLIENT_VERSION_MIN)
// setField(L, "max", CLIENT_VERSION_MAX)
// setField(L, "string", CLIENT_VERSION_STR)
let client_version_tbl = lua.create_table()?;
client_version_tbl.set("min", CLIENT_VERSION_MIN as i64)?;
client_version_tbl.set("max", CLIENT_VERSION_MAX as i64)?;
client_version_tbl.set("string", CLIENT_VERSION_STR)?;
lua.globals().set("CLIENT_VERSION", client_version_tbl)?;
```

- [ ] **Step 4.4: Run tests to confirm they pass**

```bash
cargo test --lib -p forgottenserver-scripting client_version_table
```
Expected: all 4 tests PASS

- [ ] **Step 4.5: Run full workspace tests + lint**

```bash
cargo test --lib --workspace && cargo clippy --workspace --lib --tests -- -D warnings
```
Expected: PASS, zero warnings

- [ ] **Step 4.6: Commit** (ask user first)

---

### Task 5: Add game TCP listener

**Files:**
- Modify: `crates/server/src/boot.rs` — add `GameLoginHandler` struct and `start_game_listener` function
- Modify: `crates/tfs/src/boot.rs` — call `start_game_listener` in `start_listeners`

Background: the game port (7172) is currently unbound. The upstream TFS 13.10 sends a challenge packet (opcode `0x1F` + 4-byte timestamp + 1-byte random) to the client on connect. This plan implements: accept TCP, send challenge, read first packet, validate version, disconnect if wrong version. Full session token auth is deferred.

- [ ] **Step 5.1: Write the failing test** — add to `#[cfg(test)]` in `crates/server/src/boot.rs`:

```rust
#[test]
fn start_game_listener_binds_port_and_accepts_connection() {
    let game_port = free_port();
    let config = make_config(free_port(), free_port(), "");
    let mut config_manager = ConfigManager::new();
    config_manager.set_integer(IntegerKey::GamePort, game_port as i64);
    let config = Arc::new(config_manager);
    let game_state = Arc::new(Mutex::new(GameState::new()));

    let res = start_game_listener(config, game_state);
    assert!(res.is_ok(), "start_game_listener must bind successfully: {:?}", res);

    // Verify connection is accepted and server sends some bytes (challenge)
    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{game_port}")).unwrap();
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
    let mut buf = [0u8; 16];
    let n = stream.read(&mut buf).unwrap_or(0);
    assert!(n > 0, "game listener must send a challenge packet on connect");
    assert_eq!(buf[2], 0x1F, "first payload byte must be challenge opcode 0x1F (after 2-byte length prefix)");
}

#[test]
fn start_game_listener_errors_when_port_already_bound() {
    let game_port = free_port();
    let _hog = std::net::TcpListener::bind(format!("0.0.0.0:{game_port}")).unwrap();

    let mut config_manager = ConfigManager::new();
    config_manager.set_integer(IntegerKey::GamePort, game_port as i64);
    let config = Arc::new(config_manager);
    let game_state = Arc::new(Mutex::new(GameState::new()));

    let res = start_game_listener(config, game_state);
    assert!(res.is_err(), "must error when port is already bound");
    let err = res.unwrap_err();
    assert!(err.contains("Cannot bind game port"), "error: {err}");
}
```

- [ ] **Step 5.2: Run tests to see them fail**

```bash
cargo test --lib -p forgottenserver-server start_game_listener
```
Expected: FAIL — `error[E0425]: cannot find function 'start_game_listener'`

- [ ] **Step 5.3: Add `GameLoginHandler` and `start_game_listener` to `crates/server/src/boot.rs`**

Add these imports at the top:
```rust
use std::io::{Read, Write};
use forgottenserver_common::definitions::CLIENT_VERSION_MIN;
use forgottenserver_common::definitions::CLIENT_VERSION_MAX;
use forgottenserver_common::definitions::CLIENT_VERSION_STR;
use forgottenserver_network::protocolgame::{parse_login_packet, serialize_disconnect};
use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::outputmessage::OutputMessage;
```

Add the handler struct and implementation:
```rust
/// Handles one game-port TCP connection.
///
/// On connect: sends a challenge packet (opcode 0x1F + 4-byte timestamp + 1-byte random).
/// On first client packet: validates version; disconnects if out of range.
/// Full TFS 13.10 session-token auth is deferred.
pub struct GameLoginHandler;

impl GameLoginHandler {
    pub fn handle_connection(&self, mut stream: std::net::TcpStream) {
        // Send challenge packet: [len_lo, len_hi, 0x1F, ts0, ts1, ts2, ts3, rand]
        let timestamp: u32 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);
        let rand_byte: u8 = (timestamp & 0xFF) as u8;
        let mut challenge = OutputMessage::new();
        challenge.add_u8(0x1F);
        challenge.add_u32(timestamp);
        challenge.add_u8(rand_byte);
        challenge.write_message_length();
        if stream.write_all(challenge.get_output_buffer()).is_err() {
            return;
        }

        // Read first client packet: [len_lo, len_hi, opcode, payload...]
        let mut header = [0u8; 2];
        if stream.read_exact(&mut header).is_err() {
            return;
        }
        let body_len = u16::from_le_bytes(header) as usize;
        if body_len == 0 {
            return;
        }
        let mut body = vec![0u8; body_len];
        if stream.read_exact(&mut body).is_err() {
            return;
        }

        // body[0] is the opcode; payload starts at body[1]
        if body.len() < 3 {
            return; // too short to contain version bytes
        }
        let payload = &body[1..];
        let mut msg = NetworkMessage::new();
        msg.add_bytes(payload);
        msg.set_buffer_position(0);

        if let Err(disconnect_msg) = parse_login_packet(&mut msg) {
            // Version rejected or malformed — send disconnect
            let disconnect_payload = serialize_disconnect(&disconnect_msg);
            let len = (disconnect_payload.len() as u16).to_le_bytes();
            let _ = stream.write_all(&len);
            let _ = stream.write_all(&disconnect_payload);
        }
        // On success: charlist flow is deferred (requires TFS 13.10 session token protocol)
    }
}

impl ConnectionHandler for GameLoginHandler {
    fn handle(&self, stream: std::net::TcpStream) {
        self.handle_connection(stream);
    }
}

/// Spawn the game TCP listener (port 7172) as a background thread.
///
/// Mirrors C++ otserv.cpp `mainLoader()` step 15:
///   `g_dispatcher.addTask(createTask([] { ... Service::open(game_port) ... }))`
pub fn start_game_listener(
    config: Arc<ConfigManager>,
    _game_state: Arc<Mutex<GameState>>,
) -> Result<(), String> {
    let game_port = config.get_integer(IntegerKey::GamePort) as u16;
    let listener = TcpListener::bind(format!("0.0.0.0:{game_port}"))
        .map_err(|e| format!("Cannot bind game port {game_port}: {e}"))?;
    let handler = Arc::new(GameLoginHandler);
    std::thread::spawn(move || {
        accept_loop(listener, handler);
    });
    Ok(())
}
```

- [ ] **Step 5.4: Run tests to confirm they pass**

```bash
cargo test --lib -p forgottenserver-server start_game_listener
```
Expected: both tests PASS

- [ ] **Step 5.5: Wire `start_game_listener` into `crates/tfs/src/boot.rs`**

In `start_listeners`:
```rust
pub fn start_listeners(modules: &Modules) -> Result<()> {
    srv_boot::start_admin_and_status(modules.config.clone(), modules.game_state.clone())
        .map_err(|e| anyhow!("Failed to start admin/status listeners: {e}"))?;
    srv_boot::start_game_listener(modules.config.clone(), modules.game_state.clone())
        .map_err(|e| anyhow!("Failed to start game listener: {e}"))?;
    Ok(())
}
```

- [ ] **Step 5.6: Run full workspace tests + lint**

```bash
cargo test --lib --workspace && cargo clippy --workspace --lib --tests -- -D warnings
```
Expected: PASS, zero warnings

- [ ] **Step 5.7: Commit** (ask user first)

---

### Task 6: Update e2e fixture — port 8080 and DB seeding

**Files:**
- Modify: `crates/e2e/tests/common/mod.rs`
- Create: `crates/e2e/tests/common/seed.rs`

- [ ] **Step 6.1: Create `crates/e2e/tests/common/seed.rs`**

```rust
#![cfg(feature = "e2e")]

use testcontainers::{core::ExecCommand, ContainerAsync, GenericImage};

/// Insert a test account and one character into the MariaDB container.
///
/// Account: name="test", password=SHA1("test"), type=1
/// Character: name="Testchar", account_id=1, town_id=1, pos=(160,54,7)
///
/// Schema reference: forgottenserver/schema.sql
/// - accounts(id, name, password char(40), type)
/// - players(id, name, account_id, vocation, level, health, healthmax, town_id, posx, posy, posz, cap, sex)
pub async fn seed_db(mariadb: &ContainerAsync<GenericImage>) {
    let sql = concat!(
        "INSERT IGNORE INTO accounts (id, name, password, type) ",
        "VALUES (1, 'test', SHA1('test'), 1); ",
        "INSERT IGNORE INTO players ",
        "(id, name, account_id, vocation, level, health, healthmax, town_id, posx, posy, posz, cap, sex) ",
        "VALUES (1, 'Testchar', 1, 0, 1, 150, 150, 1, 160, 54, 7, 400, 0);"
    );

    let result = mariadb
        .exec(ExecCommand::new([
            "mariadb",
            "-uforgottenserver",
            "-pforgottenserver",
            "tibia_rs",
            "-e",
            sql,
        ]))
        .await
        .expect("seed_db exec failed");

    assert_eq!(
        result.exit_code().await.unwrap_or(Some(1)).unwrap_or(1),
        0,
        "seed_db SQL failed"
    );
}
```

- [ ] **Step 6.2: Update `crates/e2e/tests/common/mod.rs`**

Add `pub mod seed;` at the top (after `#![cfg(feature = "e2e")]`).

Add `http_port: u16` field to `ServerFixture`:
```rust
pub struct ServerFixture {
    _mariadb: ContainerAsync<GenericImage>,
    _server: ContainerAsync<GenericImage>,
    _config_dir: tempfile::TempDir,
    status_port: u16,
    game_port: u16,
    http_port: u16,   // NEW
    _rt: tokio::runtime::Runtime,
}
```

In the async block inside `start()`, add after the game_port mapping:
```rust
let http_port = server
    .get_host_port_ipv4(8080)
    .await
    .expect("http port mapping missing");
```

Call `seed_db` after the server is ready (after `server.start().await`):
```rust
seed::seed_db(&mariadb).await;
```

Add to the return value:
```rust
(mariadb, server, config_dir, status_port, game_port, http_port)
```

Update the `ServerFixture` struct initialization:
```rust
ServerFixture {
    _mariadb: mariadb,
    _server: server,
    _config_dir: config_dir,
    status_port,
    game_port,
    http_port,     // NEW
    _rt: rt,
}
```

Add `http_port()` accessor and `http_addr()`:
```rust
pub fn http_port(&self) -> u16 {
    self.http_port
}

pub fn http_addr(&self) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], self.http_port))
}
```

- [ ] **Step 6.3: Verify the module compiles**

```bash
cargo check -p forgottenserver-e2e --tests
```
Expected: compile success (no e2e tests run without Docker)

- [ ] **Step 6.4: Commit** (ask user first)

---

## TRACK 1 — In-process protocol integration tests

### Task 7: Write in-process protocol integration tests

**Files:**
- Create: `crates/network/tests/protocol_integration.rs`
- Modify: `crates/network/Cargo.toml` — add the test file if not auto-discovered (check `[[test]]` entries)

- [ ] **Step 7.1: Check if `crates/network/Cargo.toml` needs a `[[test]]` entry**

```bash
grep -n "\[\[test\]\]\|integration" /Users/pablohpsilva/Documents/forgottenserver-rust/crates/network/Cargo.toml
```

If the file doesn't auto-discover integration tests (i.e., there's no `tests/` dir mapped), add:
```toml
[[test]]
name = "protocol_integration"
path = "tests/protocol_integration.rs"
```

- [ ] **Step 7.2: Create `crates/network/tests/protocol_integration.rs`** with:

```rust
//! In-process protocol integration tests.
//!
//! These tests exercise full parse→serialize round-trips using Cursor-based mock
//! streams. No TCP sockets or Docker required.

use forgottenserver_common::configmanager::ConfigManager;
use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_network::protocolgame::{
    parse_login_packet, serialize_character_list, serialize_disconnect, CharacterEntry,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn login_payload(version: u16) -> Vec<u8> {
    let mut msg = NetworkMessage::new();
    msg.add_u16(version);
    // xtea key (4 × u32, all zeros)
    msg.add_u32(0);
    msg.add_u32(0);
    msg.add_u32(0);
    msg.add_u32(0);
    // account_name "" (length 0)
    msg.add_u16(0);
    // password "" (length 0)
    msg.add_u16(0);
    let buf = msg.get_buffer()[..(msg.get_message_length() as usize)].to_vec();
    buf
}

fn parse_from_bytes(payload: &[u8]) -> Result<forgottenserver_network::protocolgame::LoginPacket, String> {
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload);
    msg.set_buffer_position(0);
    parse_login_packet(&mut msg)
}

// ---------------------------------------------------------------------------
// Game protocol — version rejection
// ---------------------------------------------------------------------------

#[test]
fn game_login_version_too_low_returns_disconnect_message() {
    let payload = login_payload(760);
    let result = parse_from_bytes(&payload);
    assert!(result.is_err(), "version 760 must be rejected");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("13.10"),
        "rejection message must mention allowed version: {msg}"
    );
    // Verify the message formats correctly into a disconnect packet
    let disconnect = serialize_disconnect(&msg);
    assert_eq!(disconnect[0], 0x14, "disconnect opcode must be 0x14");
}

#[test]
fn game_login_version_too_high_returns_disconnect_message() {
    let payload = login_payload(9999);
    let result = parse_from_bytes(&payload);
    assert!(result.is_err(), "version 9999 must be rejected");
    let msg = result.unwrap_err();
    assert!(msg.contains("13.10"), "rejection must mention 13.10: {msg}");
    let disconnect = serialize_disconnect(&msg);
    assert_eq!(disconnect[0], 0x14);
}

#[test]
fn game_login_version_1310_accepted() {
    let payload = login_payload(1310);
    let result = parse_from_bytes(&payload);
    assert!(result.is_ok(), "version 1310 must be accepted: {:?}", result);
    assert_eq!(result.unwrap().client_version, 1310);
}

#[test]
fn game_login_version_1311_accepted() {
    let payload = login_payload(1311);
    let result = parse_from_bytes(&payload);
    assert!(result.is_ok(), "version 1311 must be accepted: {:?}", result);
    assert_eq!(result.unwrap().client_version, 1311);
}

// ---------------------------------------------------------------------------
// Game protocol — disconnect packet wire format
// ---------------------------------------------------------------------------

#[test]
fn disconnect_bad_credentials_message_encodes_correctly() {
    let message = "Account name or password is not correct.";
    let bytes = serialize_disconnect(message);
    assert_eq!(bytes[0], 0x14, "disconnect opcode must be 0x14");
    let str_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    assert_eq!(str_len, message.len());
    assert_eq!(&bytes[3..3 + str_len], message.as_bytes());
}

#[test]
fn disconnect_version_message_encodes_correctly() {
    let message = "Only clients with protocol 13.10 allowed!";
    let bytes = serialize_disconnect(message);
    assert_eq!(bytes[0], 0x14);
    let str_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    assert_eq!(&bytes[3..3 + str_len], message.as_bytes());
}

// ---------------------------------------------------------------------------
// Game protocol — character list packet shape
// ---------------------------------------------------------------------------

#[test]
fn serialize_character_list_opcode_0x64_not_applicable() {
    // 0x64 is the MAP DESCRIPTION opcode, NOT the character list opcode.
    // Character list is the struct's raw bytes returned by serialize_character_list.
    // This test verifies the function returns the character count byte as the first byte.
    let chars = vec![CharacterEntry {
        name: "Testchar".to_string(),
        world_name: "Test World".to_string(),
        world_ip: 0x7F000001,
        world_port: 7172,
    }];
    let bytes = serialize_character_list(&chars);
    // First byte is character count
    assert_eq!(bytes[0], 1, "first byte must be character count = 1");
    // Next 2 bytes are the name length prefix (u16 LE)
    let name_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    assert_eq!(name_len, "Testchar".len());
    // Name bytes follow
    assert_eq!(&bytes[3..3 + name_len], b"Testchar");
}

#[test]
fn serialize_character_list_multiple_entries() {
    let chars = vec![
        CharacterEntry {
            name: "Alice".to_string(),
            world_name: "World".to_string(),
            world_ip: 0,
            world_port: 7172,
        },
        CharacterEntry {
            name: "Bob".to_string(),
            world_name: "World".to_string(),
            world_ip: 0,
            world_port: 7172,
        },
    ];
    let bytes = serialize_character_list(&chars);
    assert_eq!(bytes[0], 2, "character count must be 2");
}

// ---------------------------------------------------------------------------
// Status protocol — parse_request dispatch
// ---------------------------------------------------------------------------

use forgottenserver_network::protocolstatus::parse_request;

#[test]
fn status_xml_request_0xff_plus_info_parses_as_xml() {
    use forgottenserver_network::protocolstatus::StatusRequest;
    // Binary XML request: 0xFF + "info"
    let payload: &[u8] = &[0xFF, b'i', b'n', b'f', b'o'];
    let req = parse_request(payload);
    assert!(req.is_some(), "0xFF+info must parse as a StatusRequest");
    assert!(
        matches!(req.unwrap(), StatusRequest::Xml),
        "must parse as StatusRequest::Xml"
    );
}

#[test]
fn status_empty_payload_returns_none() {
    let req = parse_request(&[]);
    assert!(req.is_none(), "empty payload must return None");
}

// ---------------------------------------------------------------------------
// Status handler — dispatch_request (pure function, no sockets)
// ---------------------------------------------------------------------------

use forgottenserver_server::status_handler::StatusHandler;

fn make_status_handler_with_name(server_name: &str) -> StatusHandler {
    use std::sync::{Arc, Mutex};
    use forgottenserver_common::configmanager::{ConfigManager, StringKey};
    use forgottenserver_server::game_state::GameState;

    let gs = Arc::new(Mutex::new(GameState::new()));
    let mut config = ConfigManager::new();
    config.set_string(StringKey::ServerName, server_name);
    StatusHandler::new(gs, Arc::new(config))
}

#[test]
fn status_http_get_returns_200_and_xml() {
    let handler = make_status_handler_with_name("PortTest");
    let response = handler.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
    let text = String::from_utf8_lossy(&response);
    assert!(text.starts_with("HTTP/1.0 200"), "must return HTTP 200");
    assert!(text.contains("Content-Type: text/xml"), "must include XML content type");
    assert!(text.contains("<tsqp"), "body must contain <tsqp element");
}

#[test]
fn status_http_get_xml_contains_configured_server_name() {
    let handler = make_status_handler_with_name("PortTest");
    let response = handler.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.contains(r#"servername="PortTest""#),
        "XML must contain configured server name: {text}"
    );
}

#[test]
fn status_binary_xml_request_returns_raw_xml() {
    let handler = make_status_handler_with_name("PortTest");
    // Strip 2-byte LE length prefix from the full request
    // Full binary request: [0x05, 0x00, 0xFF, 'i', 'n', 'f', 'o']
    // After stripping 2-byte prefix, payload = [0xFF, 'i', 'n', 'f', 'o']
    let request: &[u8] = &[0x05, 0x00, 0xFF, b'i', b'n', b'f', b'o'];
    let response = handler.dispatch_request(request);
    let text = String::from_utf8_lossy(&response);
    assert!(text.contains("<tsqp"), "binary XML request must return <tsqp XML");
    // No HTTP headers in binary response
    assert!(
        !text.starts_with("HTTP"),
        "binary response must not have HTTP headers"
    );
}

#[test]
fn status_xml_contains_correct_client_version() {
    let handler = make_status_handler_with_name("PortTest");
    let response = handler.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.contains(r#"version="13.10""#),
        "XML version must be 13.10: {text}"
    );
    assert!(
        text.contains(r#"client="13.10""#),
        "XML client must be 13.10: {text}"
    );
}
```

- [ ] **Step 7.3: Run the integration tests**

```bash
cargo test --test protocol_integration -p forgottenserver-network
```
Expected: all tests PASS

If any tests reference `forgottenserver_server` (the status handler tests), ensure the network test can import it — if not, move the status handler tests to a separate integration test file in the server crate instead:

```bash
# If network tests can't import server crate:
# Move status dispatch tests to crates/server/tests/status_integration.rs
```

- [ ] **Step 7.4: Run full workspace tests + lint**

```bash
cargo test --lib --workspace && cargo clippy --workspace --lib --tests -- -D warnings
```
Expected: PASS, zero warnings

- [ ] **Step 7.5: Commit** (ask user first)

---

## TRACK 2 — E2E behavioral tests

These require Docker + MariaDB. Run with:
```bash
cargo test -p forgottenserver-e2e --features e2e -- --test-threads=1
```

### Task 8: Upgrade e2e game tests

**Files:**
- Modify: `crates/e2e/tests/game.rs`

- [ ] **Step 8.1: Replace `game_login_records_server_response` with behavioral tests**

Replace the contents of `crates/e2e/tests/game.rs` with:

```rust
#![cfg(feature = "e2e")]

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::OnceLock;
use std::time::Duration;

use common::ServerFixture;

static FIXTURE: OnceLock<ServerFixture> = OnceLock::new();

fn fixture() -> &'static ServerFixture {
    FIXTURE.get_or_init(ServerFixture::start)
}

#[test]
fn game_port_accepts_tcp_connection() {
    let addr = fixture().game_addr();
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5));
    assert!(
        stream.is_ok(),
        "game port refused connection — port may not be bound: {:?}",
        stream.err()
    );
}

#[test]
fn game_port_sends_challenge_on_connect() {
    // TFS 13.10 sends a challenge packet (opcode 0x1F) when client connects.
    // Wire: [len_lo, len_hi, 0x1F, ts0, ts1, ts2, ts3, rand]
    let addr = fixture().game_addr();
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("game port must accept");
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();

    let mut buf = [0u8; 16];
    let n = stream.read(&mut buf).unwrap_or(0);
    assert!(n >= 3, "challenge packet must be at least 3 bytes, got {n}");
    assert_eq!(
        buf[2], 0x1F,
        "first payload byte (after 2-byte length prefix) must be challenge opcode 0x1F"
    );
}

#[test]
fn game_login_bad_version_gets_disconnect() {
    // Send Tibia 7.60 login packet (version 760 = 0xF8 0x02).
    // Server must respond with a disconnect packet (opcode 0x14).
    #[rustfmt::skip]
    let login_packet: &[u8] = &[
        0x1E, 0x00,          // 2-byte LE length = 30
        0x0a,                // opcode: game login
        0xF8, 0x02,          // client version 760
        0x01, 0x00, 0x00, 0x00,  // XTEA key[0]
        0x02, 0x00, 0x00, 0x00,  // XTEA key[1]
        0x03, 0x00, 0x00, 0x00,  // XTEA key[2]
        0x04, 0x00, 0x00, 0x00,  // XTEA key[3]
        0x03, 0x00, b'a', b'c', b'c',       // account "acc"
        0x04, 0x00, b'p', b'a', b's', b's', // password "pass"
    ];

    let addr = fixture().game_addr();
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("game port must accept");
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();

    // Read and discard challenge first
    let mut challenge_buf = [0u8; 16];
    let _ = stream.read(&mut challenge_buf);

    // Send the bad-version login packet
    stream.write_all(login_packet).expect("write login packet");
    let _ = stream.shutdown(std::net::Shutdown::Write);

    // Read the disconnect response
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);

    assert!(
        !response.is_empty(),
        "server must send a disconnect packet for version 760"
    );
    // After 2-byte length prefix, first byte is opcode 0x14 (disconnect)
    assert!(
        response.len() >= 3,
        "disconnect packet must be at least 3 bytes"
    );
    assert_eq!(
        response[2], 0x14,
        "disconnect opcode must be 0x14 (after 2-byte length prefix)"
    );
}
```

- [ ] **Step 8.2: Add e2e test for valid login + charlist (deferred, documented)**

Add a commented-out placeholder explaining what's needed:

```rust
// DEFERRED: game_login_valid_gets_charlist
//
// Requires TFS 13.10 session token protocol:
//   1. Client connects → server sends challenge (0x1F + timestamp + random)
//   2. Client sends: session_token (base64) + challenge_timestamp + challenge_random + character_name
//   3. Server validates session against `sessions` table → sends character in world
//
// The current `parse_login_packet` implements the old 7.60 format (account + password).
// Migrate `parse_login_packet` to the TFS 13.10 session token format before enabling this test.
```

- [ ] **Step 8.3: Verify e2e game tests compile**

```bash
cargo check -p forgottenserver-e2e --tests --features e2e
```
Expected: compile success

---

### Task 9: Strengthen e2e status tests

**Files:**
- Modify: `crates/e2e/tests/status.rs`

- [ ] **Step 9.1: Add `status_fields_reflect_config` test**

Add to `crates/e2e/tests/status.rs`:

```rust
#[test]
fn status_xml_client_version_is_13_10() {
    let addr = fixture().status_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");
    let body = String::from_utf8_lossy(&response);

    assert!(
        body.contains(r#"version="13.10""#),
        "serverinfo version must be 13.10: {body}"
    );
    assert!(
        body.contains(r#"client="13.10""#),
        "serverinfo client must be 13.10: {body}"
    );
}

#[test]
fn status_xml_servername_matches_config() {
    let addr = fixture().status_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");
    let body = String::from_utf8_lossy(&response);

    // The e2e config.lua sets: serverName = "E2E Test Server"
    assert!(
        body.contains(r#"servername="E2E Test Server""#),
        "servername must match config.lua serverName: {body}"
    );
}

#[test]
fn status_binary_xml_contains_tsqp_and_serverinfo() {
    let request: &[u8] = &[0x05, 0x00, 0xFF, b'i', b'n', b'f', b'o'];
    let response = tcp_roundtrip(fixture().status_addr(), request);
    let body = String::from_utf8_lossy(&response);

    assert!(body.contains("<tsqp"), "binary response must contain <tsqp");
    assert!(
        body.contains("<serverinfo"),
        "binary response must contain <serverinfo"
    );
    assert!(
        body.contains("<players"),
        "binary response must contain <players"
    );
}
```

- [ ] **Step 9.2: Verify e2e status tests compile**

```bash
cargo check -p forgottenserver-e2e --tests --features e2e
```
Expected: compile success

---

### Task 10: Add HTTP port e2e test

**Files:**
- Create: `crates/e2e/tests/http.rs`

- [ ] **Step 10.1: Create `crates/e2e/tests/http.rs`**

```rust
#![cfg(feature = "e2e")]

mod common;

use std::sync::OnceLock;

use common::{tcp_roundtrip, ServerFixture};

static FIXTURE: OnceLock<ServerFixture> = OnceLock::new();

fn fixture() -> &'static ServerFixture {
    FIXTURE.get_or_init(ServerFixture::start)
}

#[test]
fn http_port_accepts_tcp_connection() {
    use std::net::TcpStream;
    use std::time::Duration;

    let addr = fixture().http_addr();
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5));
    assert!(
        stream.is_ok(),
        "HTTP port (8080) refused connection — port may not be bound: {:?}",
        stream.err()
    );
}

#[test]
fn http_port_returns_200_for_get_request() {
    let addr = fixture().http_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");
    assert!(!response.is_empty(), "HTTP port must return a response");
    assert!(
        response.starts_with(b"HTTP/1.0 200") || response.starts_with(b"HTTP/1.1 200"),
        "HTTP port must return 200 OK, got: {:?}",
        String::from_utf8_lossy(&response[..response.len().min(64)])
    );
}
```

- [ ] **Step 10.2: Verify compilation**

```bash
cargo check -p forgottenserver-e2e --tests --features e2e
```
Expected: compile success

---

## TRACK 2 — Final verification

### Task 11: Run full test suite and Docker smoke test

- [ ] **Step 11.1: Run unit tests**

```bash
cargo test --lib --workspace
```
Expected: all tests PASS, no failures

- [ ] **Step 11.2: Run lint**

```bash
cargo clippy --workspace --lib --tests -- -D warnings
```
Expected: zero warnings

- [ ] **Step 11.3: Run format check**

```bash
cargo fmt --all -- --check
```
If any files are unformatted: `cargo fmt --all`

- [ ] **Step 11.4: Build the Docker image**

```bash
docker compose build
```
Expected: build succeeds

- [ ] **Step 11.5: Run Docker stack and verify clean logs**

```bash
docker compose up --build 2>&1 | head -60
```
Expected:
- `>> Forgotten Server Online! (Rust port)` appears
- No `FATAL` or `ERROR` lines in the output

- [ ] **Step 11.6: Run e2e tests** (requires Docker running)

```bash
cargo test -p forgottenserver-e2e --features e2e -- --test-threads=1
```
Expected: all e2e tests PASS

- [ ] **Step 11.7: Final commit** (ask user first)

---

## Spec coverage check

| Spec requirement | Task |
|---|---|
| Version range check: reject < 1310 or > 1311 | Task 2 |
| `serialize_disconnect` with opcode 0x14 | Task 1 |
| Status XML: `version` and `client` fields = "13.10" | Task 3 |
| `CLIENT_VERSION` Lua global with min/max/string | Task 4 |
| Game TCP listener on port 7172 | Task 5 |
| E2E DB seeding: test account + character | Task 6 |
| E2E port 8080 mapped in testcontainer | Task 6 |
| In-process version rejection tests | Task 7 |
| In-process status protocol tests | Task 7 |
| E2E: game bad version → disconnect | Task 8 |
| E2E: game port sends challenge on connect | Task 8 |
| E2E: status client version = 13.10 | Task 9 |
| E2E: HTTP port responds with 200 | Task 10 |

**Explicitly deferred (out of scope per spec):**
- Full TFS 13.10 session token login → charlist (requires `parse_login_packet` migration to session token format)
- Admin port (not exposed in config by default)
- `game_login_bad_credentials_gets_error` e2e test (requires session token protocol)
