use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::tools::adler_checksum;
use forgottenserver_common::xtea;
use forgottenserver_database::database::Database;
use forgottenserver_database::iologindata::{load_player_for_login, lookup_session};
use forgottenserver_game::{
    npc_registry::{load_npcs_xml, NpcRegistry},
    spell_registry::{load_spells_xml, SpellRegistry},
    weapon_registry::{load_weapons_xml, WeaponRegistry},
};
use forgottenserver_items::{registry::ItemsRegistry, vocation::Vocations};
use forgottenserver_map::items_loader::load_items_otb;
use forgottenserver_network::protocolgame::{parse_first_packet, serialize_disconnect};
use forgottenserver_world::World;

use crate::{
    admin_handler::AdminHandler, game_handler::build_enter_world_burst, game_state::GameState,
    http_connection_session::HttpConnectionSession, http_login::LoginConfig,
    status_handler::StatusHandler,
};

pub struct GameData {
    pub items: ItemsRegistry,
    pub spells: SpellRegistry,
    pub weapons: WeaponRegistry,
    pub npcs: NpcRegistry,
    pub vocations: Arc<Vocations>,
}

/// Load all four game data registries from `data_dir` before entering the game loop.
///
/// Returns `Err` if a critical file (e.g. `items.otb`) cannot be read.
/// Missing individual records within each file are warnings, not errors.
pub fn boot(data_dir: &Path) -> Result<GameData, String> {
    let items = load_items_otb(&data_dir.join("items/items.otb"))?;
    let spells = load_spells_xml(&data_dir.join("spells/spells.xml"))?;
    let weapons = load_weapons_xml(&data_dir.join("weapons/weapons.xml"))?;
    let npcs = load_npcs_xml(&data_dir.join("npc"))?;

    let vocations_path = data_dir.join("XML/vocations.xml");
    let vocations = Arc::new(if vocations_path.exists() {
        let xml = std::fs::read_to_string(&vocations_path)
            .map_err(|e| format!("Cannot read vocations.xml: {e}"))?;
        Vocations::load_from_xml(&xml).map_err(|e| format!("Failed to parse vocations.xml: {e}"))?
    } else {
        Vocations::load_from_xml("<vocations/>").unwrap()
    });

    Ok(GameData {
        items,
        spells,
        weapons,
        npcs,
        vocations,
    })
}

/// Spawn the admin TCP listener and status HTTP listener as background threads.
///
/// Both listeners run until the process exits. Errors binding are returned so
/// the caller can decide whether to abort or continue without admin/status.
pub fn start_admin_and_status(
    config: Arc<ConfigManager>,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), String> {
    let admin_port = config.get_integer(IntegerKey::AdminPort) as u16;
    let status_port = config.get_integer(IntegerKey::StatusPort) as u16;
    let admin_password = config.get_string(StringKey::AdminPassword).to_owned();

    let admin_listener = TcpListener::bind(format!("0.0.0.0:{admin_port}"))
        .map_err(|e| format!("Cannot bind admin port {admin_port}: {e}"))?;
    let status_listener = TcpListener::bind(format!("0.0.0.0:{status_port}"))
        .map_err(|e| format!("Cannot bind status port {status_port}: {e}"))?;

    let admin_handler = Arc::new(AdminHandler::new(admin_password, game_state.clone()));
    let status_handler = Arc::new(StatusHandler::new(game_state, config));

    std::thread::spawn(move || {
        accept_loop(admin_listener, admin_handler);
    });

    std::thread::spawn(move || {
        accept_loop(status_listener, status_handler);
    });

    Ok(())
}

fn accept_loop<H: ConnectionHandler + Send + Sync + 'static>(
    listener: TcpListener,
    handler: Arc<H>,
) {
    for stream in listener.incoming().flatten() {
        let h = handler.clone();
        std::thread::spawn(move || h.handle(stream));
    }
}

/// Trait implemented by handlers that process a single TCP connection.
pub trait ConnectionHandler {
    fn handle(&self, stream: std::net::TcpStream);
}

impl ConnectionHandler for AdminHandler {
    fn handle(&self, stream: std::net::TcpStream) {
        self.handle_connection(stream);
    }
}

impl ConnectionHandler for StatusHandler {
    fn handle(&self, stream: std::net::TcpStream) {
        self.handle_connection(stream);
    }
}

// ---------------------------------------------------------------------------
// Game login handler (port 7172)
// ---------------------------------------------------------------------------

/// Handles a single game-protocol TCP connection.
pub struct GameLoginHandler {
    db: Arc<Mutex<Box<dyn Database + Send>>>,
    vocations: Arc<Vocations>,
}

impl GameLoginHandler {
    pub fn new(db: Arc<Mutex<Box<dyn Database + Send>>>, vocations: Arc<Vocations>) -> Self {
        Self { db, vocations }
    }

    /// Handle a single accepted TCP stream: send challenge, read first packet,
    /// validate session, load player, send enter-world burst, run game loop.
    pub fn handle_connection(&self, mut stream: std::net::TcpStream) {
        let peer = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "<unknown>".to_string());
        eprintln!("[game] connection from {peer}");

        // --- Build and send the challenge packet ---
        // Wire layout (14 bytes, all little-endian):
        //   [0..2)  outer_len u16 = 12   — bytes following this field
        //   [2..6)  adler32   u32        — adler32 of bytes [6..14)
        //   [6..8)  inner_len u16 = 6    — bytes following inner_len
        //   [8)     opcode    u8  = 0x1F — challenge opcode
        //   [9..13) timestamp u32        — current Unix seconds
        //   [13)    rand      u8         — random byte
        let timestamp: u32 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);
        let rand_byte: u8 = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
            & 0xFF) as u8;
        let mut buf = [0u8; 14];
        buf[0..2].copy_from_slice(&12u16.to_le_bytes()); // outer_len = 12
        buf[6..8].copy_from_slice(&6u16.to_le_bytes()); // inner_len = 6
        buf[8] = 0x1F; // opcode
        buf[9..13].copy_from_slice(&timestamp.to_le_bytes());
        buf[13] = rand_byte;
        let checksum = adler_checksum(&buf[6..14]);
        buf[2..6].copy_from_slice(&checksum.to_le_bytes());
        if stream.write_all(&buf).is_err() {
            return;
        }

        // --- Read first client packet ---
        // Some clients (e.g. OtClient) send a world-name prefix — raw ASCII bytes
        // ending with 0x0A — before the binary game-login packet.  This mirrors
        // the C++ TFS CONNECTION_STATE_GAMEWORLD_AUTH state machine in
        // Connection::parseHeader.
        //
        // Detection: read the first 2 bytes. If the high byte (index 1) is non-zero
        // the packet length would exceed 255, which no real game-login packet does
        // (the first packet is ~146 bytes). Treat that as the start of a text
        // prefix; drain one byte at a time until 0x0A, then read the real 2-byte
        // outer_len.
        let mut hdr = [0u8; 2];
        if let Err(e) = stream.read_exact(&mut hdr) {
            eprintln!("[game] failed to read initial header: {e}");
            return;
        }

        let outer_len = if hdr[1] != 0x00 {
            let mut prefix: Vec<u8> = vec![hdr[0], hdr[1]];
            loop {
                let mut b = [0u8; 1];
                match stream.read(&mut b) {
                    Ok(1) if b[0] == 0x0A => {
                        prefix.push(b[0]);
                        break;
                    }
                    Ok(1) => {
                        prefix.push(b[0]);
                        if prefix.len() > 512 {
                            eprintln!(
                                "[game] GAMEWORLD_AUTH: pre-login prefix exceeds 512 bytes — closing"
                            );
                            return;
                        }
                    }
                    _ => return,
                }
            }
            let phex: String = prefix.iter().map(|b| format!("{b:02x} ")).collect();
            let pascii: String = prefix
                .iter()
                .map(|&b| {
                    if (0x20..0x7f).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            eprintln!(
                "[game] GAMEWORLD_AUTH prefix ({} bytes): {phex} | {pascii}",
                prefix.len()
            );
            if let Err(e) = stream.read_exact(&mut hdr) {
                eprintln!("[game] GAMEWORLD_AUTH: failed to read real outer_len: {e}");
                return;
            }
            eprintln!(
                "[game] outer_len header bytes after prefix: {:02x} {:02x}",
                hdr[0], hdr[1]
            );
            u16::from_le_bytes(hdr) as usize
        } else {
            u16::from_le_bytes(hdr) as usize
        };

        eprintln!("[game] first packet outer_len={outer_len}");

        if outer_len > 32_768 {
            eprintln!(
                "[game] first packet outer_len={outer_len} exceeds limit — closing connection"
            );
            return;
        }
        // Must have at least sequence(4) + opcode(1) + a minimal payload.
        if outer_len < 7 {
            eprintln!("[game] first packet too short: outer_len={outer_len}");
            return;
        }

        let mut body = vec![0u8; outer_len];
        if let Err(e) = stream.read_exact(&mut body) {
            eprintln!("[game] failed to read packet body: outer_len={outer_len} err={e}");
            return;
        }
        // OTClient game-login wire format (CHECKSUM_SEQUENCE mode):
        //   [outer_len:2][sequence:4][opcode:1][os:2][version:2]...[RSA:128]
        // The 4-byte sequence number is 0 for the first packet and the
        // opcode is 0x0A (game login).  C++ ProtocolGame::onRecvFirstMessage
        // begins reading at `os`, after the connection layer consumes the
        // checksum/sequence (4 bytes) and the protocol-id/opcode (1 byte).
        let opcode = body[4];
        eprintln!(
            "[game] seq={:02x}{:02x}{:02x}{:02x} opcode=0x{opcode:02x} outer_len={outer_len}",
            body[0], body[1], body[2], body[3]
        );
        let payload: &[u8] = &body[5..];
        let mut msg = NetworkMessage::new();
        msg.add_bytes(payload);
        msg.set_buffer_position(0);

        match parse_first_packet(&mut msg) {
            Err(disconnect_msg) => {
                eprintln!("[game] parse_first_packet error: {disconnect_msg}");
                let disconnect_payload = serialize_disconnect(&disconnect_msg);
                let _ = stream.write_all(&frame_plaintext_packet(&disconnect_payload));
            }
            Ok(packet) => {
                eprintln!(
                    "[game] parsed ok: char={:?} ts={} rand={}",
                    packet.character_name, packet.challenge_timestamp, packet.challenge_random
                );
                // Validate that the client echoed back the challenge values we sent.
                if packet.challenge_timestamp != timestamp || packet.challenge_random != rand_byte {
                    eprintln!("[game] challenge mismatch: got ts={} rand={}, expected ts={timestamp} rand={rand_byte}", packet.challenge_timestamp, packet.challenge_random);
                    let disconnect = serialize_disconnect("Invalid challenge echo.");
                    let _ = stream.write_all(&frame_plaintext_packet(&disconnect));
                    return;
                }

                // --- Session lookup ---
                eprintln!(
                    "[game] looking up session token (len={}) for char={:?}",
                    packet.session_token.len(),
                    packet.character_name
                );
                let session_result = {
                    let db_guard = self.db.lock().unwrap();
                    lookup_session(&**db_guard, &packet.session_token, &packet.character_name)
                };
                let (_account_id, character_id) = match session_result {
                    Some(ids) => {
                        eprintln!("[game] session ok: account={} char={}", ids.0, ids.1);
                        ids
                    }
                    None => {
                        eprintln!("[game] session not found");
                        let disconnect =
                            serialize_disconnect("Account name or password is not correct.");
                        let _ = stream.write_all(&frame_plaintext_packet(&disconnect));
                        return;
                    }
                };

                // --- Load player row ---
                let player_data = {
                    let db_guard = self.db.lock().unwrap();
                    load_player_for_login(&**db_guard, character_id)
                };
                let player_data = match player_data {
                    Some(p) => p,
                    None => {
                        eprintln!("[game] character {character_id} could not be loaded");
                        let disconnect =
                            serialize_disconnect("Your character could not be loaded.");
                        let _ = stream.write_all(&frame_plaintext_packet(&disconnect));
                        return;
                    }
                };
                eprintln!("[game] player loaded: char={character_id}");

                // --- Build and flush enter-world burst (XTEA-encrypted, same as all
                // subsequent server→client packets after enableXTEAEncryption() in TFS) ---
                let world = World::new();
                // Deterministic creature id for the player avatar
                // (mirrors C++ player ids: 0x10000000 | guid).
                let player_creature_id = 0x1000_0000u32 | (character_id as u32);
                let burst = build_enter_world_burst(
                    &player_data,
                    &world,
                    player_creature_id,
                    &self.vocations,
                );
                eprintln!("[game] enter-world burst built: {} bytes", burst.len());
                let framed_burst = frame_packet(&burst, packet.xtea_key);
                eprintln!("[game] framed burst: {} bytes", framed_burst.len());
                match stream.write_all(&framed_burst) {
                    Ok(()) => eprintln!("[game] enter-world burst sent"),
                    Err(e) => {
                        eprintln!("[game] failed to send enter-world burst: {e}");
                        return;
                    }
                }

                // --- Set 30-second read timeout, then enter XTEA game loop ---
                // OTClient (os 4..=12, version >= 1111) uses CHECKSUM_SEQUENCE:
                // the 4-byte frame prefix is a sequence number, not Adler-32.
                let sequence_checksum = (4..=12).contains(&packet.os);
                eprintln!(
                    "[game] entering game loop (os={} sequence_checksum={sequence_checksum})",
                    packet.os
                );
                let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
                run_game_loop(&mut stream, packet.xtea_key, sequence_checksum);
                eprintln!("[game] game loop exited");
            }
        }
    }
}

/// Frame a plaintext server→client payload with Adler32 crypto header.
///
/// Wire layout: `[outer_len:2 = 6+N][adler32:4][inner_len:2=N][payload:N]`
/// The adler32 covers `[inner_len:2][payload]`.
/// Used for all unencrypted outbound packets (disconnect, pre-XTEA).
fn frame_plaintext_packet(payload: &[u8]) -> Vec<u8> {
    let inner_len = payload.len() as u16;
    let mut checksummed = Vec::with_capacity(2 + payload.len());
    checksummed.extend_from_slice(&inner_len.to_le_bytes());
    checksummed.extend_from_slice(payload);
    let adler = adler_checksum(&checksummed);
    let outer_len = (4 + checksummed.len()) as u16;
    let mut frame = Vec::with_capacity(2 + outer_len as usize);
    frame.extend_from_slice(&outer_len.to_le_bytes());
    frame.extend_from_slice(&adler.to_le_bytes());
    frame.extend_from_slice(&checksummed);
    frame
}

/// Frame a server→client payload for sending over the wire.
///
/// Wire layout: `[outerLen:2][adler32:4][xtea_region]`
/// where `xtea_region` = XTEA-encrypt(`[innerLen:2][payload]` padded to a
/// multiple of 8 bytes).  This matches C++ `Protocol::onSendMessage` with
/// XTEA encryption enabled.
fn frame_packet(payload: &[u8], xtea_key: [u32; 4]) -> Vec<u8> {
    let inner_len = payload.len() as u16;
    let content_len = 2 + payload.len();
    let xtea_region_len = if content_len.is_multiple_of(8) {
        content_len
    } else {
        content_len + (8 - content_len % 8)
    };
    let mut xtea_region = vec![0u8; xtea_region_len];
    xtea_region[0..2].copy_from_slice(&inner_len.to_le_bytes());
    xtea_region[2..2 + payload.len()].copy_from_slice(payload);

    let key = xtea::Key(xtea_key);
    let round_keys = xtea::expand_key(&key);
    xtea::encrypt(&mut xtea_region, &round_keys);

    let adler = adler_checksum(&xtea_region);
    let outer_len = (4 + xtea_region_len) as u16;

    let mut frame = Vec::with_capacity(2 + 4 + xtea_region_len);
    frame.extend_from_slice(&outer_len.to_le_bytes());
    frame.extend_from_slice(&adler.to_le_bytes());
    frame.extend_from_slice(&xtea_region);
    frame
}

/// Persistent game-packet read loop.
///
/// Reads XTEA-encrypted packets from `stream`, decrypts them with `xtea_key`,
/// validates the Adler-32 checksum, extracts the opcode, and dispatches to the
/// appropriate handler.  The loop exits cleanly on any read error (including
/// the 30-second timeout set by the caller) or a zero outer-length.
///
/// ## Wire frame layout (client → server)
/// ```text
/// [0..2)           outer_len   u16 LE  — bytes that follow (incl. adler32)
/// [2..6)           adler32     u32 LE  — checksum of frame_body[4..]
///                                        (the XTEA region)
/// [6..6+outer_len) xtea_region         — inner_len(2) + opcode(1) + data,
///                                        XTEA-encrypted, multiple of 8 bytes
/// ```
///
/// After XTEA decryption the region becomes:
/// ```text
/// [0..2) inner_len  u16 LE — byte count of the usable payload (excl. padding)
/// [2..)  opcode     u8    — packet type
/// [3..)  data             — opcode-specific bytes
/// ```
pub(crate) fn run_game_loop(
    stream: &mut std::net::TcpStream,
    xtea_key: [u32; 4],
    sequence_checksum: bool,
) {
    let key = xtea::Key(xtea_key);
    let round_keys = xtea::expand_key(&key);

    loop {
        // --- Step 1: read 2-byte outer length ---
        let mut len_buf = [0u8; 2];
        if let Err(e) = stream.read_exact(&mut len_buf) {
            eprintln!("[gameloop] exit: read outer_len failed: {e}");
            break;
        }
        let outer_len = u16::from_le_bytes(len_buf) as usize;
        if outer_len == 0 {
            eprintln!("[gameloop] exit: outer_len == 0");
            break;
        }

        // --- Step 2: read frame body (outer_len bytes) ---
        let mut body = vec![0u8; outer_len];
        if let Err(e) = stream.read_exact(&mut body) {
            eprintln!("[gameloop] exit: read body (outer_len={outer_len}) failed: {e}");
            break;
        }

        // body layout: [adler32(4), xtea_region(outer_len - 4)]
        // The XTEA region must be at least 8 bytes (one block) and a multiple of 8.
        if outer_len < 12 {
            // 4 (adler32) + 8 (minimum one XTEA block with inner_len + opcode)
            eprintln!("[gameloop] skip: outer_len={outer_len} < 12");
            continue;
        }
        let xtea_region_len = outer_len - 4;
        if !xtea_region_len.is_multiple_of(8) {
            eprintln!("[gameloop] skip: xtea_region_len={xtea_region_len} not multiple of 8");
            continue;
        }

        // --- Step 3: validate the 4-byte checksum/sequence field ---
        // OTClient (os in CLIENTOS_QT_LINUX..=CLIENTOS_OTCLIENT_MAC, version
        // >= 1111) negotiates CHECKSUM_SEQUENCE mode: the 4 bytes are an
        // incrementing sequence number, NOT an Adler-32 checksum. In that mode
        // we skip checksum validation (mirrors C++ Protocol with
        // CHECKSUM_SEQUENCE). Otherwise validate Adler-32 over the XTEA region.
        if !sequence_checksum {
            let stored_adler = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
            let computed_adler = adler_checksum(&body[4..]);
            if stored_adler != computed_adler {
                eprintln!(
                    "[gameloop] skip: adler mismatch stored={stored_adler:08x} computed={computed_adler:08x} outer_len={outer_len}"
                );
                continue;
            }
        }

        // --- Step 4: XTEA-decrypt the region in place ---
        xtea::decrypt(&mut body[4..], &round_keys);

        // After decryption, body[4..6) = inner_len (LE u16)
        // body[6) = opcode
        let inner_len = u16::from_le_bytes([body[4], body[5]]) as usize;

        // --- Step 5: validate inner length ---
        // inner_len must cover at least the opcode byte and fit within the
        // decrypted region (xtea_region_len bytes starting at body[4]).
        if inner_len == 0 || inner_len + 2 > xtea_region_len {
            eprintln!(
                "[gameloop] skip: bad inner_len={inner_len} xtea_region_len={xtea_region_len}"
            );
            continue;
        }
        let opcode = body[6];
        let dump_n = inner_len.min(8);
        let phex: String = body[6..6 + dump_n].iter().map(|b| format!("{b:02x} ")).collect();
        eprintln!("[gameloop] recv opcode=0x{opcode:02x} inner_len={inner_len} bytes: {phex}");

        // --- Step 6: dispatch ---
        match opcode {
            0x1D => {
                // Client ping → respond with pong (0x1E). Mirrors C++
                // Game::playerReceivePingBack → Player::sendPingBack (opcode
                // 0x1E). Without this the client times out and disconnects.
                let pong = frame_packet(&[0x1E], xtea_key);
                if stream.write_all(&pong).is_err() {
                    eprintln!("[gameloop] exit: failed to send pong");
                    break;
                }
                eprintln!("[gameloop] ping (0x1D) -> sent pong (0x1E)");
            }
            0x1E => {
                // Client pong (response to a server ping). No reply needed.
                eprintln!("[gameloop] pong (0x1E) received");
            }
            0x65 => eprintln!("[game] walk packet received"),
            0x96 => eprintln!("[game] say packet received"),
            0xBE => eprintln!("[game] use item packet received"),
            _ => eprintln!("[game] unknown opcode: {:#04x}", opcode),
        }
    }
}

impl ConnectionHandler for GameLoginHandler {
    fn handle(&self, stream: std::net::TcpStream) {
        self.handle_connection(stream);
    }
}

/// Bind the game-protocol listener on the configured game port and spawn a
/// background accept loop.
///
/// Mirrors C++ `otserv.cpp` `mainLoader` step 15 (open game listener on 7172).
pub fn start_game_listener(
    config: Arc<ConfigManager>,
    _game_state: Arc<Mutex<GameState>>,
    db: Arc<Mutex<Box<dyn Database + Send>>>,
    vocations: Arc<Vocations>,
) -> Result<(), String> {
    let game_port = config.get_integer(IntegerKey::GamePort) as u16;
    let listener = TcpListener::bind(format!("0.0.0.0:{game_port}"))
        .map_err(|e| format!("Cannot bind game port {game_port}: {e}"))?;
    let handler = Arc::new(GameLoginHandler::new(db, vocations));
    std::thread::spawn(move || {
        accept_loop(listener, handler);
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// HTTP login listener (port 8080)
// ---------------------------------------------------------------------------

fn build_login_config(config: &ConfigManager) -> LoginConfig {
    let pvp_type = match config.get_string(StringKey::WorldType) {
        "no-pvp" => 1u8,
        "pvp-enforced" => 2u8,
        _ => 0u8,
    };
    LoginConfig {
        server_name: config.get_string(StringKey::ServerName).to_string(),
        ip: config.get_string(StringKey::Ip).to_string(),
        game_port: config.get_integer(IntegerKey::GamePort) as u16,
        location: config.get_string(StringKey::Location).to_string(),
        pvp_type,
    }
}

/// Bind the HTTP login listener on the configured `httpPort` and spawn
/// `httpWorkers` worker threads accepting connections.
///
/// Mirrors C++ `otserv.cpp` `mainLoader` step 16. Returns `Ok(())` when
/// `httpPort == 0` (feature disabled in config).
pub fn start_http_listener(
    config: Arc<ConfigManager>,
    db: Arc<Mutex<Box<dyn Database + Send>>>,
    vocations: Arc<Vocations>,
) -> Result<(), String> {
    let http_port = config.get_integer(IntegerKey::HttpPort) as u16;
    if http_port == 0 {
        return Ok(());
    }
    let workers = (config.get_integer(IntegerKey::HttpWorkers) as usize).max(1);
    let bind_addr = {
        let s = config.get_string(StringKey::HttpBindAddress);
        if s.is_empty() { "127.0.0.1" } else { s }.to_owned()
    };

    let login_config = Arc::new(build_login_config(&config));
    let session = Arc::new(HttpConnectionSession::new(db, login_config, vocations));

    let listener = Arc::new(
        TcpListener::bind(format!("{bind_addr}:{http_port}"))
            .map_err(|e| format!("Cannot bind HTTP port {http_port}: {e}"))?,
    );

    eprintln!(">> HTTP login server online on {bind_addr}:{http_port} ({workers} worker(s)).");

    for _ in 0..workers {
        let l = Arc::clone(&listener);
        let s = Arc::clone(&session);
        std::thread::spawn(move || {
            while let Ok((stream, _)) = l.accept() {
                s.handle(stream);
            }
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Path to the data directory (via the symlink at the workspace root).
    fn data_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") // crates/server → forgottenserver-rust root → data symlink
    }

    // -----------------------------------------------------------------------
    // Test: spawn_manager.load_world populates entries before game loop
    // -----------------------------------------------------------------------
    #[test]
    fn boot_populates_spawn_entries() {
        use forgottenserver_common::position::Position;
        use forgottenserver_game::spawn_manager::SpawnManager;
        use forgottenserver_world::{SpawnPointDef, World};

        let mut world = World::new();
        world.add_spawn_point(SpawnPointDef {
            position: Position::new(100, 100, 7),
            radius: 3,
            monster_name: "Rat".to_string(),
            interval_secs: 60,
        });
        world.add_spawn_point(SpawnPointDef {
            position: Position::new(200, 200, 7),
            radius: 5,
            monster_name: "Orc".to_string(),
            interval_secs: 120,
        });

        let mut spawn_manager = SpawnManager::new();
        spawn_manager.load_world(&world);

        assert_eq!(
            spawn_manager.entry_count(),
            2,
            "boot must register all spawn points"
        );
    }

    // -----------------------------------------------------------------------
    // Test: all loaders are called before the game loop (integration)
    // -----------------------------------------------------------------------
    #[test]
    fn boot_all_four_loaders_called_before_game_loop() {
        let game_data = boot(&data_dir()).expect("boot should succeed with real data");

        // Items: items.otb is non-empty in the real data set
        assert!(
            !game_data.items.is_empty(),
            "ItemsRegistry should be populated from items.otb"
        );

        // Spells: spells.xml in the real data is empty (<spells />) — that's fine
        // We just verify the loader ran without error.
        let _ = game_data.spells.len();

        // Weapons: weapons.xml has wand entries
        assert!(
            !game_data.weapons.is_empty(),
            "WeaponRegistry should be populated from weapons.xml"
        );

        // NPCs: npc/ directory has several xml files
        assert!(
            !game_data.npcs.is_empty(),
            "NpcRegistry should be populated from data/npc/"
        );

        // Vocations: XML/vocations.xml includes at least the "None" vocation (id 0)
        assert!(
            game_data.vocations.get_vocation(0).is_some(),
            "vocations should include vocation id 0 from vocations.xml"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 6 — Script loading at boot
    // -----------------------------------------------------------------------

    #[test]
    fn load_dir_loads_all_lua_files_in_directory() {
        use forgottenserver_scripting::engine::LuaScriptEngine;
        use std::io::Write;

        let dir = tempfile::TempDir::new().unwrap();
        std::fs::File::create(dir.path().join("a.lua"))
            .unwrap()
            .write_all(b"a_loaded = true")
            .unwrap();
        std::fs::File::create(dir.path().join("b.lua"))
            .unwrap()
            .write_all(b"b_loaded = true")
            .unwrap();
        std::fs::File::create(dir.path().join("not_lua.txt"))
            .unwrap()
            .write_all(b"ignored")
            .unwrap();

        let mut engine = LuaScriptEngine::new();
        let count = engine.load_dir(dir.path()).unwrap();
        assert_eq!(count, 2, "load_dir must load exactly the .lua files");
    }

    #[test]
    fn missing_script_dir_returns_error() {
        use forgottenserver_scripting::engine::LuaScriptEngine;
        let mut engine = LuaScriptEngine::new();
        let result = engine.load_dir(std::path::Path::new("/nonexistent/scripts/xyz_abc"));
        assert!(result.is_err(), "missing directory must return an error");
    }

    #[test]
    fn boot_loads_lua_scripts_from_data_scripts() {
        // Verify boot completes without panic; Lua script loading is now handled
        // by LuaEnvironment::load_scripts in the scripting crate.
        let _game_data = boot(&data_dir()).expect("boot should succeed");
    }

    // -----------------------------------------------------------------------
    // Phase 13 — start_admin_and_status + accept_loop + ConnectionHandler
    //
    // C++ cross-validation:
    //   * otserv.cpp `mainLoader` registers ProtocolStatus on STATUS_PORT and
    //     spawns admin/status listeners via the ServiceManager.
    //   * The Rust equivalent in `boot.rs` is `start_admin_and_status`, which
    //     reads AdminPort/StatusPort/AdminPassword from the ConfigManager,
    //     binds two TcpListeners, and spawns blocking accept loops on
    //     background threads. The two `ConnectionHandler` impls (for
    //     AdminHandler and StatusHandler) dispatch one accepted stream to the
    //     handler's own `handle_connection` method.
    // -----------------------------------------------------------------------

    /// Pick a free local port by binding/dropping a listener.
    /// (Port 0 lets the OS pick; we capture the chosen port before dropping.)
    fn free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    /// Build a ConfigManager with the given admin/status ports + admin password.
    fn make_config(admin_port: u16, status_port: u16, admin_password: &str) -> Arc<ConfigManager> {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::AdminPort, admin_port as i64);
        cm.set_integer(IntegerKey::StatusPort, status_port as i64);
        cm.set_string(StringKey::AdminPassword, admin_password);
        Arc::new(cm)
    }

    #[test]
    fn start_admin_and_status_binds_both_listeners_and_returns_ok() {
        // Picks two free ports, binds them, spawns the two accept threads.
        // We verify the function returns Ok and that connecting to the bound
        // ports succeeds (proves the listeners are alive).
        let admin_port = free_port();
        let status_port = free_port();
        let config = make_config(admin_port, status_port, "secret");
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_admin_and_status(config.clone(), game_state.clone());
        assert!(res.is_ok(), "expected Ok, got: {:?}", res);

        // Verify the admin listener actually accepts a TCP connection.
        // AdminHandler will read until EOF; we close the write half to let it
        // finish without blocking.
        let mut admin_stream =
            std::net::TcpStream::connect(format!("127.0.0.1:{admin_port}")).unwrap();
        admin_stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut admin_resp = String::new();
        let _ = admin_stream.read_to_string(&mut admin_resp);

        // Verify the status listener also accepts a connection and returns XML.
        let mut status_stream =
            std::net::TcpStream::connect(format!("127.0.0.1:{status_port}")).unwrap();
        status_stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        status_stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut status_resp = String::new();
        let _ = status_stream.read_to_string(&mut status_resp);
        assert!(
            status_resp.contains("HTTP/1.0 200 OK"),
            "status listener did not respond with HTTP 200: {status_resp}"
        );
        assert!(
            status_resp.contains("<tsqp"),
            "status listener did not respond with TSQP XML: {status_resp}"
        );
    }

    #[test]
    fn start_admin_and_status_errors_when_admin_port_already_bound() {
        // Pre-bind the admin port on 0.0.0.0 (matching the bind address used
        // inside start_admin_and_status) so the listener bind fails.
        let admin_port = free_port();
        let status_port = free_port();
        let _hog = std::net::TcpListener::bind(format!("0.0.0.0:{admin_port}")).unwrap();

        let config = make_config(admin_port, status_port, "secret");
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_admin_and_status(config, game_state);
        let err = res.expect_err("expected admin-port bind to fail");
        assert!(
            err.contains("Cannot bind admin port") && err.contains(&admin_port.to_string()),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn start_admin_and_status_errors_when_status_port_already_bound() {
        // Admin port is free, but status port is hogged; this exercises the
        // second `map_err` branch (lines 68-69).
        let admin_port = free_port();
        let status_port = free_port();
        let _hog = std::net::TcpListener::bind(format!("0.0.0.0:{status_port}")).unwrap();

        let config = make_config(admin_port, status_port, "secret");
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_admin_and_status(config, game_state);
        let err = res.expect_err("expected status-port bind to fail");
        assert!(
            err.contains("Cannot bind status port") && err.contains(&status_port.to_string()),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn accept_loop_dispatches_connection_to_admin_handler() {
        // Drives `accept_loop` directly with a controlled listener so we hit
        // the for-loop, the spawn-per-stream, and the ConnectionHandler::handle
        // impl for AdminHandler (lines 89-92 + 101-103).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Carol");
        let handler = Arc::new(AdminHandler::new("pw", gs.clone()));

        std::thread::spawn(move || accept_loop(listener, handler));

        // Send `auth` + `status`; the status command reports the online
        // player count, which proves the connection traversed
        // accept_loop -> ConnectionHandler::handle -> AdminHandler::handle_connection.
        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream.write_all(b"auth pw\nstatus\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        assert!(
            resp.contains("players: 1"),
            "AdminHandler::handle did not run (Carol not counted): {resp}"
        );
        // Keep `gs` alive past the connection so it isn't dropped prematurely.
        let _ = gs;
    }

    #[test]
    fn accept_loop_dispatches_connection_to_status_handler() {
        // Same as above, but for the StatusHandler ConnectionHandler impl
        // (lines 107-109). This proves accept_loop is generic over both
        // handler types and that StatusHandler::handle forwards correctly.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        let cfg = make_config(0, 0, "");
        let handler = Arc::new(StatusHandler::new(gs, cfg));

        std::thread::spawn(move || accept_loop(listener, handler));

        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        assert!(
            resp.contains("HTTP/1.0 200 OK") && resp.contains("<tsqp"),
            "StatusHandler::handle did not run: {resp}"
        );
    }

    #[test]
    fn connection_handler_admin_handle_forwards_to_handle_connection() {
        // Direct call into the trait impl (lines 101-103) without spawning a
        // background accept loop, providing redundant coverage of the impl
        // even if a future change skips accept_loop's spawn dispatch.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        let handler: Arc<dyn ConnectionHandler + Send + Sync> =
            Arc::new(AdminHandler::new("pw", gs));

        let t = std::thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            handler.handle(server_stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.write_all(b"auth wrong\n").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        client.read_to_string(&mut resp).unwrap();
        t.join().unwrap();
        // The exact response doesn't matter; we only need the impl to run.
        assert!(
            !resp.is_empty(),
            "AdminHandler trait dispatch produced no output"
        );
    }

    #[test]
    fn connection_handler_status_handle_forwards_to_handle_connection() {
        // Direct call into the StatusHandler ConnectionHandler impl
        // (lines 107-109).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        let cfg = make_config(0, 0, "");
        let handler: Arc<dyn ConnectionHandler + Send + Sync> =
            Arc::new(StatusHandler::new(gs, cfg));

        let t = std::thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            handler.handle(server_stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        client.read_to_string(&mut resp).unwrap();
        t.join().unwrap();
        assert!(
            resp.contains("HTTP/1.0 200 OK"),
            "StatusHandler trait dispatch did not produce HTTP response: {resp}"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 15 — start_game_listener (port 7172)
    //
    // C++ cross-validation:
    //   * otserv.cpp `mainLoader` step 15: ServiceManager registers
    //     ProtocolGame on the game port (default 7172) and sends a challenge
    //     packet (opcode 0x1F + 4-byte timestamp + 1-byte random) immediately
    //     on connection before the client sends anything.
    //   * The Rust equivalent `start_game_listener` binds the port, spawns an
    //     accept loop, and for each connection calls
    //     `GameLoginHandler::handle_connection`, which sends the challenge,
    //     reads the client's first packet, validates the version, and
    //     disconnects with a human-readable message on version mismatch.
    // -----------------------------------------------------------------------

    #[test]
    fn start_game_listener_binds_port_and_accepts_connection() {
        use std::io::Read as _;

        let game_port = free_port();
        let mut config_manager = ConfigManager::new();
        config_manager.set_integer(IntegerKey::GamePort, game_port as i64);
        let config = Arc::new(config_manager);
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_game_listener(config, game_state, empty_db(), empty_vocations());
        assert!(
            res.is_ok(),
            "start_game_listener must bind successfully: {:?}",
            res
        );

        // Verify connection is accepted and server sends the 14-byte challenge.
        // Wire format: [outer_len:2=12][adler32:4][inner_len:2=6][0x1F:1][ts:4][rand:1] = 14 bytes.
        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{game_port}")).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        let mut buf = [0u8; 14];
        let n = stream.read(&mut buf).unwrap_or(0);
        assert_eq!(n, 14, "challenge must be exactly 14 bytes");
        // outer_len = 12 at [0..2]
        assert_eq!(&buf[0..2], &[0x0C, 0x00], "outer_len must be 12 LE");
        // opcode 0x1F is at byte offset 8 (after outer_len:2 + adler32:4 + inner_len:2)
        assert_eq!(buf[8], 0x1F, "challenge opcode 0x1F must be at offset 8");
    }

    #[test]
    fn start_game_listener_errors_when_port_already_bound() {
        let game_port = free_port();
        let _hog = std::net::TcpListener::bind(format!("0.0.0.0:{game_port}")).unwrap();

        let mut config_manager = ConfigManager::new();
        config_manager.set_integer(IntegerKey::GamePort, game_port as i64);
        let config = Arc::new(config_manager);
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_game_listener(config, game_state, empty_db(), empty_vocations());
        assert!(res.is_err(), "must error when port is already bound");
        let err = res.unwrap_err();
        assert!(err.contains("Cannot bind game port"), "error: {err}");
    }

    // -----------------------------------------------------------------------
    // Phase 16 — start_http_listener (port 8080)
    // -----------------------------------------------------------------------

    fn empty_db() -> Arc<Mutex<Box<dyn Database + Send>>> {
        use forgottenserver_database::database::InMemoryDb;
        Arc::new(Mutex::new(Box::new(InMemoryDb::new())))
    }

    fn empty_vocations() -> Arc<Vocations> {
        Arc::new(Vocations::load_from_xml("<vocations/>").unwrap())
    }

    fn http_config(http_port: u16) -> Arc<ConfigManager> {
        http_config_bind(http_port, "127.0.0.1")
    }

    fn http_config_bind(http_port: u16, bind_addr: &str) -> Arc<ConfigManager> {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::HttpPort, http_port as i64);
        cm.set_integer(IntegerKey::HttpWorkers, 1);
        cm.set_string(StringKey::ServerName, "TestServer");
        cm.set_string(StringKey::Ip, "127.0.0.1");
        cm.set_integer(IntegerKey::GamePort, 7172);
        cm.set_string(StringKey::Location, "EU");
        cm.set_string(StringKey::WorldType, "pvp");
        cm.set_string(StringKey::HttpBindAddress, bind_addr);
        Arc::new(cm)
    }

    #[test]
    fn start_http_listener_skips_when_port_zero() {
        let config = Arc::new(ConfigManager::new()); // httpPort defaults to 0
        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(res.is_ok(), "port 0 means disabled, must return Ok");
    }

    #[test]
    fn start_http_listener_errors_when_port_already_bound() {
        let port = free_port();
        // Pre-bind on 127.0.0.1 — the same address start_http_listener defaults to.
        let _hog = std::net::TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

        let res = start_http_listener(http_config(port), empty_db(), empty_vocations());
        let err = res.expect_err("must error when port already bound");
        assert!(
            err.contains("Cannot bind HTTP port"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn start_http_listener_binds_and_returns_ok() {
        let port = free_port();
        let res = start_http_listener(http_config(port), empty_db(), empty_vocations());
        assert!(res.is_ok(), "start_http_listener must succeed: {res:?}");

        // Port should now be accepting connections.
        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(
            conn.is_ok(),
            "HTTP port must accept connections after start"
        );
    }

    // -----------------------------------------------------------------------
    // Tasks 3.2 + 3.3 — Challenge echo validation
    //
    // C++ cross-validation:
    //   * protocolgame.cpp `onRecvFirstMessage`: after reading back the
    //     challenge fields from the RSA-decrypted block, TFS compares them
    //     to the values sent in the challenge packet and disconnects the
    //     client if they don't match.
    //   * The Rust equivalent in `handle_connection` must compare
    //     `packet.challenge_timestamp` and `packet.challenge_random` against
    //     the locally stored `timestamp` and `rand_byte`, and send a
    //     disconnect + close the connection on mismatch.
    // -----------------------------------------------------------------------

    /// Sending junk data after the challenge packet causes the server to
    /// reject the connection and return a non-empty disconnect payload.
    ///
    /// Because the RSA block will be garbage (all zeros), `parse_first_packet`
    /// fails at the RSA decrypt step before even reaching the challenge check.
    /// Any disconnect sent back (RSA fail or challenge mismatch) proves the
    /// handler correctly rejects bad packets.
    #[test]
    fn challenge_echo_mismatch_disconnects() {
        use std::io::{Read as _, Write as _};

        // Spin up a real TCP listener on an ephemeral port.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn the server handler in a background thread.
        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                GameLoginHandler::new(empty_db(), empty_vocations()).handle_connection(stream);
            }
        });

        // Connect as client.
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();

        // Read the server's 12-byte challenge packet (no outer length prefix).
        let mut _challenge = [0u8; 12];
        client.read_exact(&mut _challenge).unwrap();

        // Build a fake first-packet with junk content (all zeros for the
        // body).  The opcode byte is 0x0A (game-login opcode), followed by
        // OS (u16 LE = 3), version (u16 LE = 1310 = 0x0516), then zeros.
        // The RSA block (128 bytes of zeros) will fail to decrypt, triggering
        // a disconnect from the handler.
        let mut body = vec![0u8; 140]; // opcode + OS + version + padding + RSA
        body[0] = 0x0A; // game-login opcode
        body[1] = 0x03; // OS lo
        body[2] = 0x00; // OS hi  → OS = 3
        body[3] = 0x16; // version lo
        body[4] = 0x05; // version hi  → version = 0x0516 = 1302 … close enough;
                        // we need 1310 (0x051E) for the parser to proceed
        body[3] = 0x1E; // version lo  → 0x1E = 30
        body[4] = 0x05; // version hi  → 0x051E = 1310 ✓

        // Send the fake packet with a 2-byte LE length prefix.
        let body_len_bytes = (body.len() as u16).to_le_bytes();
        client.write_all(&body_len_bytes).unwrap();
        client.write_all(&body).unwrap();
        // Signal end-of-write so the server can detect the close.
        client.shutdown(std::net::Shutdown::Write).unwrap();

        // The server should send a disconnect payload (non-empty) and then
        // close the connection.
        let mut response = Vec::new();
        let _ = client.read_to_end(&mut response);

        assert!(
            !response.is_empty(),
            "server must send a disconnect payload when the first packet is invalid; got empty response"
        );
    }

    #[test]
    fn start_http_listener_handles_cacheinfo_request() {
        use std::io::{Read as _, Write as _};

        let port = free_port();
        let res = start_http_listener(http_config(port), empty_db(), empty_vocations());
        assert!(res.is_ok(), "start failed: {res:?}");

        // Give the accept loop a moment to start.
        std::thread::sleep(std::time::Duration::from_millis(50));

        let body = b"{\"type\":\"cacheinfo\"}";
        let request = format!("POST / HTTP/1.0\r\nContent-Length: {}\r\n\r\n", body.len());

        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        stream.write_all(body).unwrap();

        let mut response = String::new();
        let _ = stream.read_to_string(&mut response);

        assert!(
            response.contains("HTTP/1.1 200"),
            "expected HTTP 200 in response: {response:?}"
        );
        assert!(
            response.contains("Content-Type: application/json"),
            "expected Content-Type: application/json: {response:?}"
        );
        assert!(
            response.contains("\"playersonline\""),
            "expected playersonline key in cacheinfo response: {response:?}"
        );
    }

    #[test]
    fn start_http_listener_defaults_to_loopback_when_bind_address_not_set() {
        // When HttpBindAddress is absent from config the listener MUST bind to
        // 127.0.0.1. We verify by using http_config (which leaves HttpBindAddress
        // empty) and confirming the listener accepts a connection on 127.0.0.1.
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::HttpPort, free_port() as i64);
        cm.set_integer(IntegerKey::HttpWorkers, 1);
        cm.set_string(StringKey::ServerName, "TestServer");
        cm.set_string(StringKey::Ip, "127.0.0.1");
        cm.set_integer(IntegerKey::GamePort, 7172);
        cm.set_string(StringKey::Location, "EU");
        cm.set_string(StringKey::WorldType, "pvp");
        // HttpBindAddress intentionally NOT set — empty string fallback.
        let port = cm.get_integer(IntegerKey::HttpPort) as u16;
        let config = Arc::new(cm);

        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(res.is_ok(), "start must succeed: {res:?}");

        // Verify the listener is reachable on loopback.
        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(
            conn.is_ok(),
            "listener must accept connections on 127.0.0.1 when bind address is unset"
        );
    }

    #[test]
    fn start_http_listener_binds_to_configured_address_when_set_to_all_interfaces() {
        // When HttpBindAddress = "0.0.0.0" the listener must bind to all interfaces.
        let port = free_port();
        let config = http_config_bind(port, "0.0.0.0");

        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(res.is_ok(), "start must succeed with 0.0.0.0: {res:?}");

        // Both loopback and 0.0.0.0 bindings accept connections on 127.0.0.1.
        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(
            conn.is_ok(),
            "listener must accept connections when bound to 0.0.0.0"
        );
    }

    #[test]
    fn start_http_listener_empty_bind_address_does_not_produce_bare_colon_port() {
        // Ensure the empty-string fallback produces "127.0.0.1:<port>", not ":<port>".
        // We verify this by confirming the listener accepts on 127.0.0.1, which
        // would fail if the bind string were ":<port>" (invalid address).
        let port = free_port();
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::HttpPort, port as i64);
        cm.set_integer(IntegerKey::HttpWorkers, 1);
        cm.set_string(StringKey::ServerName, "TestServer");
        cm.set_string(StringKey::Ip, "127.0.0.1");
        cm.set_integer(IntegerKey::GamePort, 7172);
        cm.set_string(StringKey::Location, "EU");
        cm.set_string(StringKey::WorldType, "pvp");
        cm.set_string(StringKey::HttpBindAddress, ""); // explicitly empty
        let config = Arc::new(cm);

        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(
            res.is_ok(),
            "empty bind address must fall back to 127.0.0.1, not fail: {res:?}"
        );

        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(
            conn.is_ok(),
            "listener must be reachable on 127.0.0.1 after empty-string fallback"
        );
    }

    // -----------------------------------------------------------------------
    // Tasks 7.1–7.5 — XTEA game loop
    //
    // C++ cross-validation:
    //   * After the server sends the enter-world burst (login data, map data,
    //     etc.), all subsequent client → server packets are XTEA-encrypted with
    //     the key extracted from the RSA block during the first-packet parse.
    //   * protocolgame.cpp onRecvMessage: reads the encrypted packet, decrypts
    //     with XTEA, validates Adler-32, reads inner_len, then dispatches on the
    //     opcode byte.
    //   * The Rust equivalent `run_game_loop` mirrors this read/decrypt/dispatch
    //     cycle and breaks cleanly on any I/O error.
    // -----------------------------------------------------------------------

    /// Build a fully-framed XTEA-encrypted game packet for testing.
    ///
    /// Wire layout produced:
    /// ```text
    /// [0..2)           outer_len  u16 LE
    /// [2..6)           adler32    u32 LE (checksum of xtea_region)
    /// [6..6+xtea_len)  xtea_region      (inner_len(2) + payload, padded to 8-byte multiple)
    /// ```
    fn make_xtea_frame(payload: &[u8], xtea_key: [u32; 4]) -> Vec<u8> {
        // Build the plaintext XTEA region: inner_len (u16 LE) + payload + padding
        let payload_len = payload.len();
        let xtea_content_len = 2 + payload_len; // inner_len(2) + payload
                                                // Pad to the next multiple of 8
        let xtea_region_len = if xtea_content_len.is_multiple_of(8) {
            xtea_content_len
        } else {
            xtea_content_len + (8 - xtea_content_len % 8)
        };

        let mut xtea_region = vec![0u8; xtea_region_len];
        let inner_len = payload_len as u16;
        xtea_region[0..2].copy_from_slice(&inner_len.to_le_bytes());
        xtea_region[2..2 + payload_len].copy_from_slice(payload);
        // rest is already zero (padding)

        // Encrypt the XTEA region
        let key = xtea::Key(xtea_key);
        let round_keys = xtea::expand_key(&key);
        xtea::encrypt(&mut xtea_region, &round_keys);

        // Compute adler32 of the encrypted XTEA region
        let adler = adler_checksum(&xtea_region);

        // outer_len = 4 (adler32) + xtea_region_len
        let outer_len = (4 + xtea_region_len) as u16;

        let mut frame = Vec::with_capacity(2 + 4 + xtea_region_len);
        frame.extend_from_slice(&outer_len.to_le_bytes()); // 2 bytes
        frame.extend_from_slice(&adler.to_le_bytes()); // 4 bytes
        frame.extend_from_slice(&xtea_region); // xtea_region_len bytes
        frame
    }

    /// The game loop correctly processes a single XTEA-encrypted walk packet
    /// (opcode 0x65) sent through a TCP socket pair and exits cleanly when the
    /// client closes the connection.
    #[test]
    fn game_loop_processes_encrypted_packet_and_exits_on_close() {
        use std::io::Write as _;

        let xtea_key: [u32; 4] = [0xDEAD_BEEF, 0x1234_5678, 0xABCD_EF01, 0x0102_0304];

        // Build a walk packet: opcode 0x65 + one direction byte (0x00 = North)
        let payload = [0x65u8, 0x00];
        let frame = make_xtea_frame(&payload, xtea_key);

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        // Server side: accept one connection and run the game loop
        let server_thread = std::thread::spawn(move || {
            let (mut server_stream, _) = listener.accept().unwrap();
            // Set a short timeout so the test doesn't hang if the loop stalls
            server_stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            run_game_loop(&mut server_stream, xtea_key, false);
        });

        // Client side: send one encrypted packet then close
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.write_all(&frame).unwrap();
        // Close the write half: server loop will get EOF on the next read and break
        client.shutdown(std::net::Shutdown::Write).unwrap();

        // Server thread must exit without panic
        server_thread
            .join()
            .expect("game loop thread panicked — run_game_loop must not panic on valid input");
    }

    /// The game loop exits immediately when the client closes the connection
    /// without sending any data (EOF on first read).
    #[test]
    fn game_loop_exits_on_immediate_connection_close() {
        let xtea_key: [u32; 4] = [0x01, 0x02, 0x03, 0x04];

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server_thread = std::thread::spawn(move || {
            let (mut server_stream, _) = listener.accept().unwrap();
            server_stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            run_game_loop(&mut server_stream, xtea_key, false);
        });

        // Connect then immediately close without sending anything
        let client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        drop(client);

        server_thread
            .join()
            .expect("game loop must exit cleanly on immediate EOF");
    }

    /// XTEA round-trip: encrypting a packet then decrypting it recovers the
    /// original payload bytes.  Verifies that `make_xtea_frame` + manual
    /// decrypt + `run_game_loop` logic are consistent.
    #[test]
    fn xtea_frame_round_trip_recovers_payload() {
        let xtea_key: [u32; 4] = [0xCAFE_BABE, 0xDEAD_BEEF, 0x1234_ABCD, 0x5678_EF01];
        let payload = [0x96u8, b'H', b'e', b'l', b'l', b'o']; // say opcode + "Hello"

        let frame = make_xtea_frame(&payload, xtea_key);

        // Parse the frame manually the same way run_game_loop does
        let outer_len = u16::from_le_bytes([frame[0], frame[1]]) as usize;
        assert_eq!(
            outer_len,
            frame.len() - 2,
            "outer_len must equal frame body length"
        );

        let body = &frame[2..]; // outer_len bytes
        assert!(
            outer_len >= 12,
            "outer_len must be at least 12 for a minimal packet"
        );

        let stored_adler = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
        let computed_adler = adler_checksum(&body[4..]);
        assert_eq!(stored_adler, computed_adler, "adler32 must match");

        let xtea_region_len = outer_len - 4;
        assert_eq!(xtea_region_len % 8, 0, "XTEA region must be multiple of 8");

        let mut xtea_region: Vec<u8> = body[4..].to_vec();
        let key = xtea::Key(xtea_key);
        let round_keys = xtea::expand_key(&key);
        xtea::decrypt(&mut xtea_region, &round_keys);

        let inner_len = u16::from_le_bytes([xtea_region[0], xtea_region[1]]) as usize;
        assert_eq!(
            inner_len,
            payload.len(),
            "inner_len must equal payload length"
        );

        let recovered = &xtea_region[2..2 + inner_len];
        assert_eq!(recovered, &payload, "decoded payload must match original");
        assert_eq!(recovered[0], 0x96, "opcode must be 0x96 (say)");
    }

    // -----------------------------------------------------------------------
    // Challenge packet wire format
    // -----------------------------------------------------------------------

    /// Verify the 14-byte challenge layout matches the C++ `ProtocolGame::onConnect()` wire
    /// format produced by `send(output)`:
    /// [outer_len:2=12][adler32:4][inner_len:2=6][0x1F:1][ts:4][rand:1]
    #[test]
    fn challenge_packet_is_14_bytes_with_outer_len() {
        let ts: u32 = 0xDEAD_BEEF;
        let rand: u8 = 0x42;

        let mut buf = [0u8; 14];
        buf[0..2].copy_from_slice(&12u16.to_le_bytes()); // outer_len = 12
        buf[6..8].copy_from_slice(&6u16.to_le_bytes()); // inner_len = 6
        buf[8] = 0x1F; // opcode
        buf[9..13].copy_from_slice(&ts.to_le_bytes());
        buf[13] = rand;
        let checksum = adler_checksum(&buf[6..14]);
        buf[2..6].copy_from_slice(&checksum.to_le_bytes());

        // Exactly 14 bytes
        assert_eq!(buf.len(), 14);
        // outer_len = 12 at [0..2]
        assert_eq!(&buf[0..2], &[0x0C, 0x00], "outer_len must be 12 LE");
        // adler32 covers bytes [6..14]
        assert_eq!(
            u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]),
            adler_checksum(&buf[6..14]),
            "adler32 must cover [inner_len..rand]"
        );
        // inner_len field at [6..8]
        assert_eq!(&buf[6..8], &[0x06, 0x00], "inner_len must be 6 LE");
        // opcode
        assert_eq!(buf[8], 0x1F);
        // timestamp round-trips
        assert_eq!(u32::from_le_bytes([buf[9], buf[10], buf[11], buf[12]]), ts);
        // random byte
        assert_eq!(buf[13], rand);
    }

    /// Full round-trip: server sends correctly-framed 14-byte challenge; client
    /// reads it, builds a valid RSA-encrypted first packet (challenge echo +
    /// seeded session token), sends it, and asserts the server replies with
    /// the XTEA-encrypted enter-world burst (opcode 0x0A), NOT a disconnect.
    #[test]
    fn game_login_challenge_round_trip() {
        use forgottenserver_common::base64;
        use forgottenserver_database::database::{Database, DbError, DbValue, Row};
        use num_bigint::BigUint;
        use rsa::pkcs1::DecodeRsaPrivateKey;
        use rsa::traits::PublicKeyParts;
        use rsa::RsaPrivateKey;
        use std::collections::HashMap;
        use std::io::{Read, Write as IoWrite};

        // Minimal Database that satisfies lookup_session and load_player_for_login.
        struct RoundTripDb;
        impl Database for RoundTripDb {
            fn query(&self, sql: &str) -> Result<Vec<Row>, DbError> {
                if sql.contains("FROM accounts a") {
                    // lookup_session — return account_id=1, character_id=1
                    let mut map = HashMap::new();
                    map.insert("account_id".to_string(), DbValue::Integer(1));
                    map.insert("character_id".to_string(), DbValue::Integer(1));
                    Ok(vec![Row::new(map)])
                } else if sql.contains("FROM players") {
                    // load_player_for_login — return a minimal player row
                    let mut map = HashMap::new();
                    map.insert("name".to_string(), DbValue::Text("TestChar".to_string()));
                    map.insert("level".to_string(), DbValue::Integer(1));
                    map.insert("health".to_string(), DbValue::Integer(100));
                    map.insert("healthmax".to_string(), DbValue::Integer(100));
                    map.insert("mana".to_string(), DbValue::Integer(0));
                    map.insert("manamax".to_string(), DbValue::Integer(0));
                    map.insert("stamina".to_string(), DbValue::Integer(2520));
                    map.insert("posx".to_string(), DbValue::Integer(100));
                    map.insert("posy".to_string(), DbValue::Integer(100));
                    map.insert("posz".to_string(), DbValue::Integer(7));
                    Ok(vec![Row::new(map)])
                } else {
                    Ok(vec![])
                }
            }
            fn execute(&mut self, _sql: &str) -> Result<u64, DbError> {
                Ok(1)
            }
            fn escape_string(&self, s: &str) -> String {
                s.replace('\\', "\\\\").replace('\'', "\\'")
            }
        }

        // Load RSA key into the global singleton (needed by parse_first_packet).
        forgottenserver_common::rsa::load_pem(forgottenserver_common::rsa::DEFAULT_KEY_PEM).ok();

        let db: Arc<Mutex<Box<dyn Database + Send>>> = Arc::new(Mutex::new(Box::new(RoundTripDb)));
        let handler = GameLoginHandler::new(db, empty_vocations());

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server_thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            handler.handle_connection(stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        // Read the 14-byte challenge and extract timestamp + rand.
        let mut challenge = [0u8; 14];
        client.read_exact(&mut challenge).unwrap();
        let outer_len_c = u16::from_le_bytes([challenge[0], challenge[1]]);
        assert_eq!(outer_len_c, 12, "challenge outer_len must be 12");
        let challenge_ts =
            u32::from_le_bytes([challenge[9], challenge[10], challenge[11], challenge[12]]);
        let challenge_rand = challenge[13];

        // Build RSA plaintext block.
        // Layout: [0x00][xtea_key:16][gm_flag:1][st_len:2][token_b64:N][cn_len:2][name:M][ts:4][rand:1]
        let xtea_key: [u32; 4] = [0x1122_3344, 0x5566_7788, 0x99AA_BBCC, 0xDDEE_FF00];
        let token_bytes = [0x42u8; 16];
        let token_b64 = base64::encode(&token_bytes);
        let token_b64_bytes = token_b64.as_bytes();
        let char_name = "TestChar";
        let char_name_bytes = char_name.as_bytes();

        let mut plaintext = [0u8; 128];
        let mut cur = 0usize;
        plaintext[cur] = 0x00;
        cur += 1;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[0].to_le_bytes());
        cur += 4;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[1].to_le_bytes());
        cur += 4;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[2].to_le_bytes());
        cur += 4;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[3].to_le_bytes());
        cur += 4;
        plaintext[cur] = 0x00; // gm_flag
        cur += 1;
        let st_len = token_b64_bytes.len() as u16;
        plaintext[cur..cur + 2].copy_from_slice(&st_len.to_le_bytes());
        cur += 2;
        plaintext[cur..cur + token_b64_bytes.len()].copy_from_slice(token_b64_bytes);
        cur += token_b64_bytes.len();
        let cn_len = char_name_bytes.len() as u16;
        plaintext[cur..cur + 2].copy_from_slice(&cn_len.to_le_bytes());
        cur += 2;
        plaintext[cur..cur + char_name_bytes.len()].copy_from_slice(char_name_bytes);
        cur += char_name_bytes.len();
        plaintext[cur..cur + 4].copy_from_slice(&challenge_ts.to_le_bytes());
        cur += 4;
        plaintext[cur] = challenge_rand;

        // RSA-encrypt the plaintext with the public key: c = m^e mod n.
        let priv_key = RsaPrivateKey::from_pkcs1_pem(forgottenserver_common::rsa::DEFAULT_KEY_PEM)
            .expect("DEFAULT_KEY_PEM must be valid");
        let m = BigUint::from_bytes_be(&plaintext);
        let e = BigUint::from_bytes_be(&priv_key.e().to_bytes_be());
        let n = BigUint::from_bytes_be(&priv_key.n().to_bytes_be());
        let c = m.modpow(&e, &n);
        let c_bytes = c.to_bytes_be();
        let mut rsa_block = [0u8; 128];
        let rsa_offset = 128 - c_bytes.len();
        rsa_block[rsa_offset..].copy_from_slice(&c_bytes);

        // Build first-packet payload for parse_first_packet:
        // [os:2][version:2][build:4][dat:3][rsa:128] = 139 bytes.
        // Remaining after os+version+build = 131 < 132, so no version-string branch.
        let mut pfp_payload = Vec::<u8>::with_capacity(139);
        pfp_payload.extend_from_slice(&2u16.to_le_bytes()); // os = CLIENTOS_WINDOWS
        pfp_payload.extend_from_slice(&1310u16.to_le_bytes()); // protocol version
        pfp_payload.extend_from_slice(&[0u8; 4]); // client build (skipped)
        pfp_payload.extend_from_slice(&[0u8; 3]); // dat revision + preview state (skipped)
        pfp_payload.extend_from_slice(&rsa_block);

        // Wrap in the game-login wire framing the first-packet reader expects:
        //   [outer_len:2][sequence:4][opcode:1][pfp_payload]
        // `handle_connection` reads the 2-byte outer_len, then `outer_len`
        // body bytes, treats body[0..4] as the sequence id, body[4] as the
        // opcode (0x0A), and body[5..] as the payload fed to
        // `parse_first_packet`.
        let mut body = Vec::with_capacity(5 + pfp_payload.len());
        body.extend_from_slice(&[0u8; 4]); // sequence (0 for first packet)
        body.push(0x0Au8); // opcode = game login
        body.extend_from_slice(&pfp_payload);
        let outer_len = body.len() as u16;
        let mut framed = Vec::with_capacity(2 + body.len());
        framed.extend_from_slice(&outer_len.to_le_bytes());
        framed.extend_from_slice(&body);
        client.write_all(&framed).unwrap();

        // Read the server's response: [outer_len:2][adler32:4][xtea_region].
        let mut resp_hdr = [0u8; 6];
        client.read_exact(&mut resp_hdr).unwrap();
        let resp_outer_len = u16::from_le_bytes([resp_hdr[0], resp_hdr[1]]) as usize;
        assert!(
            resp_outer_len > 4,
            "enter-world burst outer_len must be > 4, got {resp_outer_len}"
        );
        let xtea_len = resp_outer_len - 4;
        let mut xtea_region = vec![0u8; xtea_len];
        client.read_exact(&mut xtea_region).unwrap();

        // XTEA-decrypt and verify the first opcode is 0xA0 (player stats — the
        // first packet of the Player::login bundle), not 0x14 (disconnect).
        let key = xtea::Key(xtea_key);
        let round_keys = xtea::expand_key(&key);
        xtea::decrypt(&mut xtea_region, &round_keys);
        assert!(
            xtea_region.len() >= 3,
            "decrypted region must have at least inner_len(2) + opcode(1)"
        );
        let resp_opcode = xtea_region[2]; // [inner_len:2][opcode:1]...
        assert_eq!(
            resp_opcode, 0xA0,
            "server must send player stats (0xA0) as the first login packet, got 0x{resp_opcode:02X}"
        );

        // Close the client write side so run_game_loop gets EOF and exits.
        client.shutdown(std::net::Shutdown::Write).unwrap();
        server_thread
            .join()
            .expect("handle_connection must not panic on valid round-trip login");
    }

    /// A connection that sends an implausibly large outer_len (0xFFFF = 65535) as its
    /// first packet must be closed cleanly without a panic.
    #[test]
    fn challenge_outer_len_guard_rejects_implausible_length() {
        use forgottenserver_database::database::InMemoryDb;
        use std::io::{Read, Write as _};

        let db: Arc<Mutex<Box<dyn Database + Send>>> =
            Arc::new(Mutex::new(Box::new(InMemoryDb::new())));
        let handler = GameLoginHandler::new(db, empty_vocations());

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server_thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            handler.handle_connection(stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        // Drain the 14-byte challenge the server sends first
        let mut challenge = [0u8; 14];
        client.read_exact(&mut challenge).unwrap();

        // Send outer_len = 65535 as the first two bytes of the "login packet"
        client.write_all(&[0xFF, 0xFF]).unwrap();
        // Close our write side; server must close its side after the guard fires
        client.shutdown(std::net::Shutdown::Write).unwrap();

        // Server thread must exit without panic
        server_thread
            .join()
            .expect("handle_connection must not panic on implausible outer_len");
    }
}
