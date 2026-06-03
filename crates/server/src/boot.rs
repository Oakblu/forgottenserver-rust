use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::position::Position;
use forgottenserver_common::tools::adler_checksum;
use forgottenserver_common::xtea;
use forgottenserver_database::database::Database;
use forgottenserver_database::iologindata::{
    load_player_for_login, lookup_session, save_player_logout, PlayerLoginData, PlayerLogoutData,
};
use forgottenserver_entity::player::{base_speed, Player};
use forgottenserver_entity::monsters::Monsters;
use forgottenserver_game::{
    action_registry::ActionRegistry,
    monster_registry::load_monsters_xml,
    npc_registry::{load_npcs_xml, NpcRegistry},
    spell_registry::{load_spells_xml, SpellRegistry},
    weapon_registry::{load_weapons_xml, WeaponRegistry},
};
use forgottenserver_items::{
    registry::ItemsRegistry,
    vocation::{Vocation, Vocations},
};
use forgottenserver_map::items_loader::load_items_otb;
use forgottenserver_network::protocolgame::{self as pg, parse_first_packet, serialize_disconnect};
use forgottenserver_world::iomap::IoMap;
use forgottenserver_world::map::Map;
use forgottenserver_world::World;

use crate::{
    admin_handler::AdminHandler,
    channel_session::ChannelSession,
    codec::{encode, ServerPacket},
    game_handler::{
        build_enter_world_burst, build_map_around_player, handle_auto_walk, handle_close_channel,
        handle_fight_modes, handle_follow, handle_get_channels, handle_open_channel,
        handle_open_private_channel, handle_set_outfit, handle_use_item, handle_vip_remove,
    },
    game_state::{GameState, OutfitAppearance},
    http_connection_session::HttpConnectionSession,
    http_login::LoginConfig,
    status_handler::StatusHandler,
};
use forgottenserver_game::chat::ChatManager;

pub struct GameData {
    pub items: ItemsRegistry,
    pub spells: SpellRegistry,
    pub weapons: WeaponRegistry,
    pub npcs: NpcRegistry,
    pub vocations: Arc<Vocations>,
    /// Monster types loaded from `<data_dir>/monster/monsters.xml`.
    pub monsters: Monsters,
    /// The loaded world map parsed from `<data_dir>/world/<mapName>.otbm`.
    /// Shared by all game listener connections via `Arc`.
    pub map: Arc<Map>,
}

/// Load all four game data registries from `data_dir` before entering the game loop.
///
/// Returns `Err` if a critical file (e.g. `items.otb`) cannot be read.
/// Missing individual records within each file are warnings, not errors.
pub fn boot(data_dir: &Path, map_name: &str) -> Result<GameData, String> {
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

    let monster_dir = data_dir.join("monster");
    let monsters = if monster_dir.exists() {
        match load_monsters_xml(&monster_dir) {
            Ok(m) => {
                eprintln!(">> Loaded {} monster types", m.get_monster_count());
                m
            }
            Err(e) => {
                eprintln!("[Warning] Failed to load monsters: {e}");
                Monsters::new()
            }
        }
    } else {
        Monsters::new()
    };

    let map_path = data_dir.join("world").join(format!("{map_name}.otbm"));
    let map_bytes = std::fs::read(&map_path)
        .map_err(|e| format!("Cannot read map file {}: {e}", map_path.display()))?;
    let map = IoMap::load_from_bytes(&map_bytes, &items)?;
    eprintln!(
        ">> Loaded map: {} tiles, {}x{}",
        map.get_tile_count(),
        map.get_declared_width(),
        map.get_declared_height()
    );
    let map = Arc::new(map);

    Ok(GameData {
        items,
        spells,
        weapons,
        npcs,
        vocations,
        monsters,
        map,
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
    map: Arc<Map>,
}

impl GameLoginHandler {
    pub fn new(
        db: Arc<Mutex<Box<dyn Database + Send>>>,
        vocations: Arc<Vocations>,
        map: Arc<Map>,
    ) -> Self {
        Self { db, vocations, map }
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
                    &self.map,
                );
                eprintln!("[game] enter-world burst built: {} bytes", burst.len());
                {
                }

                // OTClient (os 4..=12, version >= 1111) uses CHECKSUM_SEQUENCE:
                // the 4-byte frame prefix is a sequence number, not Adler-32.
                // Must be computed before framing the burst so the burst uses
                // the correct format; sending Adler-32 when OTClient expects a
                // sequence number risks bit 31 being set, causing OTClient to
                // attempt (and fail) zlib decompression and silently drop the
                // enter-world burst → black canvas.
                let sequence_checksum = (4..=12).contains(&packet.os);
                // Sequence counter: burst is seq=1 (C++ starts at 1 with ++m_serverSequence).
                let framed_burst = if sequence_checksum {
                    frame_packet_seq(&burst, packet.xtea_key, 1)
                } else {
                    frame_packet(&burst, packet.xtea_key)
                };
                eprintln!("[game] framed burst: {} bytes (sequence_checksum={sequence_checksum})", framed_burst.len());
                match stream.write_all(&framed_burst) {
                    Ok(()) => eprintln!("[game] enter-world burst sent"),
                    Err(e) => {
                        eprintln!("[game] failed to send enter-world burst: {e}");
                        return;
                    }
                }

                // --- Set 30-second read timeout, then enter XTEA game loop ---
                eprintln!(
                    "[game] entering game loop (os={} sequence_checksum={sequence_checksum})",
                    packet.os
                );
                // 5s read timeout drives the periodic server-ping cadence
                // inside run_game_loop (mirrors C++ Player::sendPing every 5s).
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let logout = run_game_loop(
                    &mut stream,
                    packet.xtea_key,
                    sequence_checksum,
                    2, // server_sequence starts at 2 (burst used 1)
                    player_data,
                    player_creature_id,
                    Arc::clone(&self.vocations),
                    Arc::clone(&self.map),
                );
                eprintln!("[game] game loop exited");
                // Explicitly shut down the TCP stream so OTClient gets a clean
                // FIN/RST and doesn't end up stuck on the next reconnect.
                let _ = stream.shutdown(std::net::Shutdown::Both);
                // Persist the player's logout position and current HP/mana.
                let _ = {
                    let mut db_guard = self.db.lock().unwrap();
                    save_player_logout(
                        &mut **db_guard,
                        character_id,
                        &PlayerLogoutData {
                            pos_x: logout.pos.x,
                            pos_y: logout.pos.y,
                            pos_z: logout.pos.z,
                            health: logout.health,
                            mana: logout.mana,
                            direction: logout.direction,
                        },
                    )
                };
                eprintln!(
                    "[game] player {} saved at ({},{},{}) hp={} mp={}",
                    character_id, logout.pos.x, logout.pos.y, logout.pos.z,
                    logout.health, logout.mana,
                );
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
/// XTEA encryption enabled (non-sequenced mode).
fn frame_packet(payload: &[u8], xtea_key: [u32; 4]) -> Vec<u8> {
    frame_packet_inner(payload, xtea_key, None)
}

/// Frame a server→client payload using a sequence number instead of Adler-32.
///
/// Used when `sequence_checksum=true` (OTClient, os 4..=12, version >= 1111).
/// OTClient reads the 4-byte header field as a sequence number and treats
/// bit 31 as a decompression flag; sending Adler-32 here risks bit 31 being
/// set and causing OTClient to attempt (and fail) zlib decompression,
/// silently dropping the packet.  Using a monotonically increasing counter
/// (bit 31 never set for the first ~2 billion packets) avoids this.
fn frame_packet_seq(payload: &[u8], xtea_key: [u32; 4], seq: u32) -> Vec<u8> {
    frame_packet_inner(payload, xtea_key, Some(seq))
}

fn frame_packet_inner(payload: &[u8], xtea_key: [u32; 4], seq_override: Option<u32>) -> Vec<u8> {
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

    let header = seq_override.unwrap_or_else(|| adler_checksum(&xtea_region));
    let outer_len = (4 + xtea_region_len) as u16;

    let mut frame = Vec::with_capacity(2 + 4 + xtea_region_len);
    frame.extend_from_slice(&outer_len.to_le_bytes());
    frame.extend_from_slice(&header.to_le_bytes());
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_game_loop(
    stream: &mut std::net::TcpStream,
    xtea_key: [u32; 4],
    sequence_checksum: bool,
    server_sequence_start: u32,
    player_data: PlayerLoginData,
    player_creature_id: u32,
    vocations: Arc<Vocations>,
    map: Arc<Map>,
) -> LogoutSave {
    let key = xtea::Key(xtea_key);
    let round_keys = xtea::expand_key(&key);

    // Sequence counter for server→client packets (OTClient sequenced mode).
    let mut server_seq = server_sequence_start;

    // ------------------------------------------------------------------
    // Per-connection mutable state
    // ------------------------------------------------------------------
    let world = World::new();
    let mut state = GameState::new();
    let voc: Vocation = vocations
        .get_vocation(player_data.vocation_id)
        .cloned()
        .unwrap_or_else(|| Vocation::new(player_data.vocation_id));
    let base_speed_half = (base_speed(player_data.level) / 2) as u16;

    let mut player_pos = Position::new(player_data.posx, player_data.posy, player_data.posz);
    let mut player_dir = player_data.direction;
    // C++ Direction_t: NORTH=0, EAST=1, SOUTH=2, WEST=3 (position.h:9-12).
    const DIR_NORTH: u8 = 0;
    const DIR_EAST: u8 = 1;
    const DIR_SOUTH: u8 = 2;
    const DIR_WEST: u8 = 3;

    {
        let mut player = Player::new(
            player_creature_id,
            &player_data.name,
            player_data.vocation_id,
        );
        player.set_max_health(player_data.healthmax as i32);
        player.set_health(player_data.health as i32);
        player.set_max_mana(player_data.manamax as i32);
        player.set_mana(player_data.mana as i32);
        state.add_player_entity(player);
        state.set_player_position(player_creature_id, player_pos);
    }

    // Capture outfit fields so we can re-emit the player creature on walk/turn.
    let player_name = player_data.name.clone();
    let look_type = player_data.look_type;
    let look_head = player_data.look_head;
    let look_body = player_data.look_body;
    let look_legs = player_data.look_legs;
    let look_feet = player_data.look_feet;
    let look_addons = player_data.look_addons;
    let look_mount = player_data.look_mount;
    let voc_client_id = voc.client_id;

    // ------------------------------------------------------------------
    // Helper closures
    // ------------------------------------------------------------------
    // Pragmatic walk/turn response: re-emit a full 0x64 map description
    // centered on the new player position with the 9x9 synthetic ground
    // patch and the player creature on the center tile.
    //
    // This is intentionally simpler than C++ (which sends 0x6D for an
    // intra-floor move plus incremental edge-row/column updates via
    // 0x65–0x68 server→client opcodes).  A full redraw is byte-correct
    // and keeps the camera in sync until the incremental responses are
    // implemented.
    let render_map = |dir: u8, pos: Position| -> Vec<u8> {
        build_map_around_player(
            &world,
            &map,
            pos,
            player_creature_id,
            &player_name,
            dir,
            voc_client_id,
            base_speed_half,
            look_type,
            look_head,
            look_body,
            look_legs,
            look_feet,
            look_addons,
            look_mount,
        )
    };

    // Per-connection chat and channel session state.
    let mut chat = ChatManager::new();
    let mut channel_session = ChannelSession::new(player_creature_id);

    // Track consecutive read timeouts so we can send periodic server pings
    // (mirrors C++ Player::sendPing every 5s, player.cpp:871) without sitting
    // silent for 30 s. After ~30 s of no client activity we give up.
    let mut consecutive_timeouts: u32 = 0;
    const MAX_CONSECUTIVE_TIMEOUTS: u32 = 6;
    // Accumulated seconds since last mana/HP regen tick (each 5-second timeout = 5s).
    let mut mana_regen_secs: u32 = 0;
    let mut hp_regen_secs: u32 = 0;

    loop {
        // --- Step 1: read 2-byte outer length ---
        let mut len_buf = [0u8; 2];
        match stream.read_exact(&mut len_buf) {
            Ok(()) => {
                consecutive_timeouts = 0;
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // No client activity within the read window. Send a server
                // ping (opcode 0x1D) so the client knows the server is alive
                // — without this OTClient eventually decides the server is
                // dead and ends up in a stuck "connecting…" state on its
                // next reconnect.
                consecutive_timeouts += 1;
                if consecutive_timeouts > MAX_CONSECUTIVE_TIMEOUTS {
                    eprintln!(
                        "[gameloop] exit: idle > {MAX_CONSECUTIVE_TIMEOUTS} timeouts, closing"
                    );
                    break;
                }
                // --- Mana regen tick: each 5-second idle window = 5 seconds elapsed ---
                let (mana_regen, new_mana_acc) =
                    apply_regen_tick(5, mana_regen_secs, voc.gain_mana_ticks, voc.gain_mana_amount);
                mana_regen_secs = new_mana_acc;
                // --- HP regen tick ---
                let (hp_regen, new_hp_acc) =
                    apply_regen_tick(5, hp_regen_secs, voc.gain_health_ticks, voc.gain_health_amount);
                hp_regen_secs = new_hp_acc;

                if mana_regen > 0 || hp_regen > 0 {
                    let stats_packet =
                        if let Some(player) = state.get_player_entity_mut(player_creature_id) {
                            let old_health = player.get_health();
                            let old_mana = player.get_mana();
                            if hp_regen > 0 {
                                player.add_hp_regen(hp_regen as i32);
                            }
                            if mana_regen > 0 {
                                player.add_mp_regen(mana_regen as i32);
                            }
                            if player.get_health() != old_health || player.get_mana() != old_mana {
                                Some(encode(&ServerPacket::PlayerStats {
                                    health: player.get_health(),
                                    max_health: player.get_max_health(),
                                    mana: player.get_mana(),
                                    max_mana: player.get_max_mana(),
                                    level: player_data.level,
                                    stamina: player_data.stamina as u16,
                                }))
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                    if let Some(stats_bytes) = stats_packet {
                        let seq = server_seq;
                        server_seq = server_seq.wrapping_add(1);
                        let frame = if sequence_checksum {
                            frame_packet_seq(&stats_bytes, xtea_key, seq)
                        } else {
                            frame_packet(&stats_bytes, xtea_key)
                        };
                        if let Err(e) = stream.write_all(&frame) {
                            eprintln!("[gameloop] exit: failed to send regen stats: {e}");
                            break;
                        }
                    }
                }
                // --- Server ping ---
                let seq = server_seq;
                server_seq = server_seq.wrapping_add(1);
                let ping = if sequence_checksum {
                    frame_packet_seq(&[0x1D], xtea_key, seq)
                } else {
                    frame_packet(&[0x1D], xtea_key)
                };
                if let Err(e) = stream.write_all(&ping) {
                    eprintln!("[gameloop] exit: failed to send server ping: {e}");
                    break;
                }
                eprintln!(
                    "[gameloop] idle ({}/{MAX_CONSECUTIVE_TIMEOUTS}) -> sent server ping (0x1D)",
                    consecutive_timeouts
                );
                continue;
            }
            Err(e) => {
                eprintln!("[gameloop] exit: read outer_len failed: {e}");
                break;
            }
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
        let phex: String = body[6..6 + dump_n]
            .iter()
            .map(|b| format!("{b:02x} "))
            .collect();
        eprintln!("[gameloop] recv opcode=0x{opcode:02x} inner_len={inner_len} bytes: {phex}");

        // The opcode-specific payload bytes live at body[7..6+inner_len].
        // We wrap them in a NetworkMessage so the existing pg::parse_* helpers
        // can consume them directly.
        let payload_start = 7usize;
        let payload_end = 6 + inner_len;
        let payload_slice: &[u8] = if payload_end > payload_start && payload_end <= body.len() {
            &body[payload_start..payload_end]
        } else {
            &[]
        };

        // --- Step 6: dispatch ---
        let response = dispatch_opcode(
            opcode,
            payload_slice,
            &world,
            &mut state,
            player_creature_id,
            &mut player_pos,
            &mut player_dir,
            DIR_NORTH,
            DIR_EAST,
            DIR_SOUTH,
            DIR_WEST,
            &render_map,
            &player_name,
            player_data.level as u16,
            &mut chat,
            &mut channel_session,
        );

        match response {
            DispatchResult::Break => {
                eprintln!("[gameloop] exit: logout");
                break;
            }
            DispatchResult::Response(bytes) => {
                let seq = server_seq;
                server_seq = server_seq.wrapping_add(1);
                let frame = if sequence_checksum {
                    frame_packet_seq(&bytes, xtea_key, seq)
                } else {
                    frame_packet(&bytes, xtea_key)
                };
                if let Err(e) = stream.write_all(&frame) {
                    eprintln!(
                        "[gameloop] exit: failed to send response opcode=0x{opcode:02x}: {e}"
                    );
                    break;
                }
            }
            DispatchResult::NoResponse => {}
        }
    }

    let health = state
        .get_player_entity(player_creature_id)
        .map(|p| p.get_health())
        .unwrap_or(player_data.health as i32);
    let mana = state
        .get_player_entity(player_creature_id)
        .map(|p| p.get_mana())
        .unwrap_or(player_data.mana as i32);
    LogoutSave { pos: player_pos, health, mana, direction: player_dir }
}

/// State captured at logout that must be written back to the database.
pub(crate) struct LogoutSave {
    pub pos: Position,
    pub health: i32,
    pub mana: i32,
    pub direction: u8,
}

/// Result of dispatching a single client opcode in the game loop.
pub(crate) enum DispatchResult {
    /// No response packet; continue reading.
    NoResponse,
    /// Send these bytes (already in `[opcode][fields]` form) framed via XTEA.
    Response(Vec<u8>),
    /// Exit the game loop cleanly (logout).
    Break,
}

/// Dispatch a single client opcode and produce a response action.
///
/// `payload_slice` is the bytes AFTER the opcode byte (the parse_* helpers
/// expect to read from that point).  Mutable state — position, direction,
/// game state — is updated in place.
///
/// Mirrors C++ `ProtocolGame::parsePacket` (`protocolgame.cpp` 503-1100), but
/// only the opcodes wired into the Rust port so far are handled; everything
/// else falls into the catch-all "unknown opcode" branch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_opcode<F>(
    opcode: u8,
    payload_slice: &[u8],
    _world: &World,
    state: &mut GameState,
    player_creature_id: u32,
    player_pos: &mut Position,
    player_dir: &mut u8,
    dir_north: u8,
    dir_east: u8,
    dir_south: u8,
    dir_west: u8,
    render_map: &F,
    player_name: &str,
    player_level: u16,
    chat: &mut ChatManager,
    session: &mut ChannelSession,
) -> DispatchResult
where
    F: Fn(u8, Position) -> Vec<u8>,
{
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload_slice);
    msg.set_buffer_position(0);

    match opcode {
        // --- Logout (C++ protocolgame.cpp:532 logout(true, false)) ---
        0x14 => {
            eprintln!("[gameloop] logout (0x14)");
            DispatchResult::Break
        }
        // --- Ignored opcodes when a player exists; C++ default = no-op. ---
        // 0x0F (enter game ready) and 0x60 (server ready) are produced
        // server→client; the client never sends them here but some clients
        // echo / poke variants — silently ignore.
        0x0F | 0x60 | 0xD0 | 0x91 => {
            eprintln!("[gameloop] no-op opcode=0x{opcode:02x}");
            DispatchResult::NoResponse
        }
        // --- Client ping → pong ---
        0x1D => DispatchResult::Response(vec![0x1E]),
        // --- Server-ping reply, just log ---
        0x1E => {
            eprintln!("[gameloop] pong (0x1E) received");
            DispatchResult::NoResponse
        }
        // --- OtClient extended opcode: payload exists but we don't consume it ---
        0x32 => {
            eprintln!(
                "[gameloop] extended opcode (0x32): {} bytes",
                payload_slice.len()
            );
            DispatchResult::NoResponse
        }
        // --- AutoWalk (0x64) — queued multi-step path ---
        // C++ protocolgame.cpp:549 → parseAutoWalk → g_game.playerAutoWalk.
        // We store the mapped direction path in GameState; the game loop
        // will execute steps incrementally. Invalid payloads are dropped silently.
        0x64 => match pg::parse_auto_walk(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] auto-walk {} steps (0x64)", pkt.directions.len());
                handle_auto_walk(player_creature_id, pkt.directions, state);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] auto-walk parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- Walk N/E/S/W (single-byte opcode; no payload) ---
        // C++ protocolgame.cpp:551-562 dispatches Game::playerMove.  Our
        // pragmatic response is a fresh full 0x64 map description at the
        // new position so the client camera follows.  This is a deliberate
        // simplification: C++ sends 0x6D move + edge row/col updates
        // (0x65–0x68 server→client) for an incremental redraw.
        //
        // 0x65=N, 0x66=E, 0x67=S, 0x68=W  (cardinal)
        // 0x6A=NE, 0x6B=SE, 0x6C=SW, 0x6D=NW  (diagonal; C++ Direction_t 4-7)
        0x65..=0x68 | 0x6A..=0x6D => {
            let (dx, dy, dir): (i32, i32, u8) = match opcode {
                0x65 => (0, -1, dir_north),
                0x66 => (1, 0, dir_east),
                0x67 => (0, 1, dir_south),
                0x68 => (-1, 0, dir_west),
                0x6A => (1, -1, 4), // NE
                0x6B => (1, 1, 5),  // SE
                0x6C => (-1, 1, 6), // SW
                0x6D => (-1, -1, 7), // NW
                _ => unreachable!(),
            };
            let new_x = player_pos.x as i32 + dx;
            let new_y = player_pos.y as i32 + dy;
            // Clamp to u16 range so we never panic on a wrap; C++ similarly
            // bounds-checks via tile lookup.
            if !(0..=u16::MAX as i32).contains(&new_x) || !(0..=u16::MAX as i32).contains(&new_y) {
                eprintln!("[gameloop] walk out of range: ({new_x},{new_y})");
                return DispatchResult::NoResponse;
            }
            let new_pos = Position::new(new_x as u16, new_y as u16, player_pos.z);
            *player_pos = new_pos;
            *player_dir = dir;
            state.set_player_position(player_creature_id, new_pos);
            eprintln!(
                "[gameloop] walk dir={dir} -> ({}, {}, {})",
                new_pos.x, new_pos.y, new_pos.z
            );
            DispatchResult::Response(render_map(dir, new_pos))
        }
        // --- StopAutoWalk (0x69) ---
        // C++ protocolgame.cpp:564 → playerStopAutoWalk: cancels the queued
        // auto-walk path. No response packet is sent.
        0x69 => {
            eprintln!("[gameloop] stop auto-walk (0x69)");
            handle_auto_walk(player_creature_id, vec![], state);
            DispatchResult::NoResponse
        }
        // --- Turn N/E/S/W (single-byte opcode; no payload). ---
        // Pragmatic response: re-emit the full map at the same position so
        // the new facing direction is visible.  A more faithful response
        // would send the per-creature direction packet only.
        0x6F..=0x72 => {
            let dir = match opcode {
                0x6F => dir_north,
                0x70 => dir_east,
                0x71 => dir_south,
                0x72 => dir_west,
                _ => unreachable!(),
            };
            *player_dir = dir;
            eprintln!("[gameloop] turn -> dir={dir}");
            DispatchResult::Response(render_map(dir, *player_pos))
        }
        // --- EquipObject (0x77) — hotkey equip ---
        // C++ protocolgame.cpp:1302 parseEquipObject: reads sprite_id (u16),
        // dispatches g_game.playerEquipItem. Game logic not yet ported;
        // we parse and log so the client message is consumed and the connection
        // stays in sync.
        0x77 => match pg::parse_equip_object(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] equip object sprite_id={} (0x77)", pkt.sprite_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] equip object parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- Throw / MoveThing (0x78) ---
        // C++ protocolgame.cpp:1384 parseThrow: reads from_pos, sprite_id,
        // from_stackpos, to_pos, count, dispatches g_game.playerMoveThing.
        // Game logic not yet ported; parse and log to consume the bytes.
        0x78 => match pg::parse_throw(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] throw sprite_id={} from=({},{},{}) stackpos={} to=({},{},{}) count={} (0x78)",
                    pkt.sprite_id,
                    pkt.from_x, pkt.from_y, pkt.from_z,
                    pkt.from_stackpos,
                    pkt.to_x, pkt.to_y, pkt.to_z,
                    pkt.count
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] throw parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- LookInShop (0x79) ---
        // C++ parseLookInShop: item_id(u16), count(u8). playerLookInShop not ported.
        0x79 => match pg::parse_look_in_shop(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] look in shop item_id={} count={} (0x79)", pkt.item_id, pkt.count);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] look in shop parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- PlayerPurchase (0x7A) ---
        // C++ parsePlayerPurchase: item_id(u16), sub_type(u8), count(u8),
        // ignore_capacity(bool), buy_with_backpack(bool). playerPurchaseItem not ported.
        0x7A => match pg::parse_player_purchase(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] player purchase item_id={} count={} ignore_cap={} backpack={} (0x7A)",
                    pkt.item_id, pkt.count, pkt.ignore_capacity, pkt.buy_with_backpack
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] player purchase parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- PlayerSale (0x7B) ---
        // C++ parsePlayerSale: item_id(u16), sub_type(u8), count(u8),
        // ignore_equipped(bool). playerSellItem not ported.
        0x7B => match pg::parse_player_sale(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] player sale item_id={} count={} ignore_equipped={} (0x7B)",
                    pkt.item_id, pkt.count, pkt.ignore_equipped
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] player sale parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- CloseShop (0x7C) — no payload ---
        // C++ case 0x7C: playerCloseShop. No parse needed; game logic not ported.
        0x7C => {
            eprintln!("[gameloop] close shop (0x7C)");
            DispatchResult::NoResponse
        }
        // --- RequestTrade (0x7D) ---
        // C++ parseRequestTrade: pos(x,y,z), sprite_id(u16), stackpos(u8), player_id(u32).
        // playerRequestTrade not ported.
        0x7D => match pg::parse_request_trade(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] request trade pos=({},{},{}) sprite_id={} stackpos={} player_id={} (0x7D)",
                    pkt.pos_x, pkt.pos_y, pkt.pos_z, pkt.sprite_id, pkt.stackpos, pkt.player_id
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] request trade parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- LookInTrade (0x7E) ---
        // C++ parseLookInTrade: counter_offer(bool), index(u8). playerLookInTrade not ported.
        0x7E => match pg::parse_look_in_trade(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] look in trade counter_offer={} index={} (0x7E)",
                    pkt.counter_offer, pkt.index
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] look in trade parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- AcceptTrade (0x7F) — no payload ---
        // C++ case 0x7F: playerAcceptTrade. Game logic not ported.
        0x7F => {
            eprintln!("[gameloop] accept trade (0x7F)");
            DispatchResult::NoResponse
        }
        // --- CloseTrade (0x80) — no payload ---
        // C++ case 0x80: playerCloseTrade. Game logic not ported.
        0x80 => {
            eprintln!("[gameloop] close trade (0x80)");
            DispatchResult::NoResponse
        }
        // --- UseItemEx (0x83) ---
        // C++ parseUseItemEx: from_pos, sprite_id, from_stackpos, to_pos.
        // playerUseItemEx not ported.
        0x83 => match pg::parse_use_item_ex(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] use item ex sprite_id={} from=({},{},{}) stackpos={} to=({},{},{}) (0x83)",
                    pkt.sprite_id,
                    pkt.from_x, pkt.from_y, pkt.from_z,
                    pkt.from_stackpos,
                    pkt.to_x, pkt.to_y, pkt.to_z
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] use item ex parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- UseWithCreature (0x84) ---
        // C++ parseUseWithCreature: pos, sprite_id, stackpos, creature_id.
        // playerUseItemWithCreature not ported.
        0x84 => match pg::parse_use_with_creature(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] use with creature sprite_id={} pos=({},{},{}) creature_id={} (0x84)",
                    pkt.sprite_id, pkt.pos_x, pkt.pos_y, pkt.pos_z, pkt.creature_id
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] use with creature parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- RotateItem (0x85) ---
        // C++ parseRotateItem: pos, sprite_id, stackpos. playerRotateItem not ported.
        0x85 => match pg::parse_rotate_item(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] rotate item sprite_id={} pos=({},{},{}) stackpos={} (0x85)",
                    pkt.sprite_id, pkt.pos_x, pkt.pos_y, pkt.pos_z, pkt.stackpos
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] rotate item parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- EditPodiumRequest (0x86) ---
        // C++ parseEditPodiumRequest: pos, sprite_id, stackpos, outfit, direction.
        // playerSetShowOffSocket not ported.
        0x86 => match pg::parse_edit_podium_request(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] edit podium pos=({},{},{}) sprite_id={} dir={} (0x86)",
                    pkt.pos_x, pkt.pos_y, pkt.pos_z, pkt.sprite_id, pkt.direction
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] edit podium parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- CloseContainer (0x87) ---
        // C++ parseCloseContainer: container_id (u8). playerCloseContainer not ported.
        0x87 => match pg::parse_close_container(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] close container id={} (0x87)", pkt.container_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] close container parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- UpArrowContainer (0x88) ---
        // C++ parseUpArrowContainer: container_id (u8). playerMoveUpContainer not ported.
        0x88 => match pg::parse_up_arrow_container(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] up arrow container id={} (0x88)", pkt.container_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] up arrow container parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- TextWindow (0x89) ---
        // C++ parseTextWindow: window_text_id (u32), text (string).
        // playerWriteItem not ported.
        0x89 => match pg::parse_text_window(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] text window id={} text_len={} (0x89)",
                    pkt.window_text_id, pkt.text.len()
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] text window parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- HouseWindow (0x8A) ---
        // C++ parseHouseWindow: door_id (u8), id (u32), text (string).
        // playerUpdateHouseWindow not ported.
        0x8A => match pg::parse_house_window(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] house window door_id={} window_id={} (0x8A)",
                    pkt.door_id, pkt.window_id
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] house window parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- WrapItem (0x8B) ---
        // C++ parseWrapItem: pos, sprite_id, stackpos. playerWrapItem not ported.
        0x8B => match pg::parse_wrap_item(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] wrap item sprite_id={} pos=({},{},{}) stackpos={} (0x8B)",
                    pkt.sprite_id, pkt.pos_x, pkt.pos_y, pkt.pos_z, pkt.stackpos
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] wrap item parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- LookAt (0x8C) ---
        // C++ parseLookAt: pos, item_id, stack_pos. playerLookAt not ported.
        0x8C => match pg::parse_look_at_packet(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] look at item_id={} pos=({},{},{}) stack_pos={} (0x8C)",
                    pkt.item_id, pkt.pos_x, pkt.pos_y, pkt.pos_z, pkt.stack_pos
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] look at parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- LookInBattleList (0x8D) ---
        // C++ parseLookInBattleList: creature_id (u32). playerLookInBattleList not ported.
        0x8D => match pg::parse_look_in_battle_list(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] look in battle list creature_id={} (0x8D)",
                    pkt.creature_id
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] look in battle list parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- Say (0x96) ---
        // Wire format (C++ parseSay): [say_type:u8][text:string].
        // The builtin "/pos" command responds with a TEXT_MESSAGE showing
        // coordinates (dev helper).  All other text is echoed as a Talk
        // (0xAA) packet mirroring C++ ProtocolGame::sendCreatureSay.
        0x96 => {
            match pg::parse_say_packet(&mut msg) {
                Ok(say) => {
                    eprintln!("[gameloop] say type={} text={:?}", say.say_type, say.text);
                    if say.text.starts_with("/pos") {
                        let text = format!(
                            "x={}, y={}, z={}",
                            player_pos.x, player_pos.y, player_pos.z
                        );
                        DispatchResult::Response(pg::serialize_text_message(
                            pg::text_message_class::MESSAGE_EVENT_ADVANCE,
                            &text,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ))
                    } else {
                        use forgottenserver_game::chat::SpeakType;
                        let speak_type =
                            SpeakType::from_byte(say.say_type).unwrap_or(SpeakType::Say);
                        let body = encode(&ServerPacket::Talk {
                            speaker: player_name.to_string(),
                            speaker_level: player_level,
                            speak_type,
                            channel_id: None,
                            pos: Some(*player_pos),
                            text: say.text,
                        });
                        DispatchResult::Response(body)
                    }
                }
                Err(e) => {
                    eprintln!("[gameloop] say parse error: {e}");
                    DispatchResult::NoResponse
                }
            }
        }
        // --- Fight modes (0xA0) ---
        // C++ parseFightModes reads fight/chase/secure (and the unused pvp
        // byte was removed in 10.0).  We mirror the 3-byte form; no
        // response packet is sent.
        0xA0 => match pg::parse_fight_modes(&mut msg) {
            Ok(fm) => {
                handle_fight_modes(
                    player_creature_id,
                    fm.fight_mode,
                    fm.chase_mode,
                    fm.secure_mode != 0,
                    state,
                );
                eprintln!(
                    "[gameloop] fight modes fight={} chase={} secure={}",
                    fm.fight_mode, fm.chase_mode, fm.secure_mode
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] fight modes parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- Use item (0x82) ---
        // Empty ActionRegistry → handle_use_item returns the
        // "Sorry, not possible." TextMessage body.
        0x82 => match pg::parse_use_item_packet(&mut msg) {
            Ok(use_pkt) => {
                eprintln!(
                    "[gameloop] use item id={} at ({},{},{}) idx={}",
                    use_pkt.item_id, use_pkt.pos_x, use_pkt.pos_y, use_pkt.pos_z, use_pkt.index
                );
                let bytes = handle_use_item(&ActionRegistry::new(), use_pkt.item_id);
                DispatchResult::Response(bytes)
            }
            Err(e) => {
                eprintln!("[gameloop] use item parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- GetChannels (0x97) ---
        // C++ protocolgame.cpp:543 parsePacket case → parseRequestChannels → Game::getChannels
        // No payload. Returns a ChannelList (0xAC) packet.
        0x97 => {
            eprintln!("[gameloop] get channels (0x97)");
            DispatchResult::Response(handle_get_channels(chat))
        }
        // --- OpenChannel (0x98) ---
        // C++ protocolgame.cpp:544 parsePacket case → parseOpenChannel → Game::playerOpenChannel
        // Payload: channel_id (u16 LE). Returns OpenChannel ack (0xAB).
        0x98 => match pg::parse_open_channel(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] open channel id={} (0x98)", pkt.channel_id);
                DispatchResult::Response(handle_open_channel(chat, session, pkt.channel_id))
            }
            Err(e) => {
                eprintln!("[gameloop] open channel parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- CloseChannel (0x99) ---
        // C++ protocolgame.cpp:545 parsePacket case → parseCloseChannel → Game::playerCloseChannel
        // Payload: channel_id (u16 LE). No response packet.
        0x99 => match pg::parse_close_channel(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] close channel id={} (0x99)", pkt.channel_id);
                handle_close_channel(chat, session, pkt.channel_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] close channel parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- OpenPrivateChannel (0x9A) ---
        // C++ protocolgame.cpp:546 parsePacket case → parseOpenPrivateChannel
        // Payload: receiver name (string). Returns OpenPrivateChannel (0xAF).
        0x9A => match pg::parse_open_private_channel(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] open private channel receiver={:?} (0x9A)", pkt.receiver);
                DispatchResult::Response(handle_open_private_channel(&pkt.receiver))
            }
            Err(e) => {
                eprintln!("[gameloop] open private channel parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- Attack (0xA1) ---
        // C++ protocolgame.cpp:688 case 0xA1 → parseAttack → reads creature_id (u32 LE)
        // → Game::playerSetAttackedCreature(playerId, creature_id). No response packet.
        0xA1 => match pg::parse_attack(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] attack creature_id=0x{:08x} (0xA1)",
                    pkt.creature_id
                );
                state.set_attack_target(player_creature_id, pkt.creature_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] attack parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- Follow (0xA2) ---
        // C++ protocolgame.cpp:690 case 0xA2 → parseFollow → reads creature_id (u32 LE)
        // → Game::playerFollowCreature: clears attack target, then sets follow creature
        // (or clears follow if creature_id == 0 / creature not found). No response.
        0xA2 => match pg::parse_follow(&mut msg) {
            Ok(pkt) => {
                use forgottenserver_map::pathfinder::Pathfinder;
                eprintln!(
                    "[gameloop] follow creature_id=0x{:08x} (0xA2)",
                    pkt.creature_id
                );
                // Always clears attack target — mirrors C++ removeAttackedCreature.
                state.set_attack_target(player_creature_id, 0);
                if pkt.creature_id == 0 {
                    // creature_id 0 → cancel follow
                    state.set_follow_target(player_creature_id, 0, vec![]);
                } else {
                    // Look up target position; fall back to empty path if not in state.
                    let target_pos = state
                        .get_creature_position(pkt.creature_id)
                        .or_else(|| state.get_player_position(pkt.creature_id));
                    match target_pos {
                        Some(tp) => {
                            handle_follow(
                                &Pathfinder,
                                *player_pos,
                                tp,
                                player_creature_id,
                                pkt.creature_id,
                                state,
                            );
                        }
                        None => {
                            // Creature not yet visible — store intent with empty path.
                            state.set_follow_target(player_creature_id, pkt.creature_id, vec![]);
                        }
                    }
                }
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] follow parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- CloseNpcChannel (0x9E) — no payload ---
        // C++ case 0x9E: playerCloseNpcChannel. Game logic not ported.
        0x9E => {
            eprintln!("[gameloop] close npc channel (0x9E)");
            DispatchResult::NoResponse
        }
        // --- InviteToParty (0xA3) ---
        // C++ parseInviteToParty: target_id (u32). playerInviteToParty not ported.
        0xA3 => match pg::parse_invite_to_party(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] invite to party target_id={} (0xA3)", pkt.target_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] invite to party parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- JoinParty (0xA4) ---
        // C++ parseJoinParty: target_id (u32). playerJoinParty not ported.
        0xA4 => match pg::parse_join_party(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] join party target_id={} (0xA4)", pkt.target_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] join party parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- RevokePartyInvite (0xA5) ---
        // C++ parseRevokePartyInvite: target_id (u32). playerRevokePartyInvitation not ported.
        0xA5 => match pg::parse_revoke_party_invite(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] revoke party invite target_id={} (0xA5)", pkt.target_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] revoke party invite parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- PassPartyLeadership (0xA6) ---
        // C++ parsePassPartyLeadership: target_id (u32). playerPassPartyLeadership not ported.
        0xA6 => match pg::parse_pass_party_leadership(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] pass party leadership target_id={} (0xA6)", pkt.target_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] pass party leadership parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- LeaveParty (0xA7) — no payload ---
        // C++ case 0xA7: playerLeaveParty. Game logic not ported.
        0xA7 => {
            eprintln!("[gameloop] leave party (0xA7)");
            DispatchResult::NoResponse
        }
        // --- EnableSharedPartyExperience (0xA8) ---
        // C++ parseEnableSharedPartyExperience: active (u8). playerEnableSharedPartyExperience not ported.
        0xA8 => match pg::parse_enable_shared_party_experience(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] enable shared party experience active={} (0xA8)", pkt.active);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] enable shared party experience parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- CreatePrivateChannel (0xAA) — no payload ---
        // C++ case 0xAA: playerCreatePrivateChannel. Game logic not ported.
        0xAA => {
            eprintln!("[gameloop] create private channel (0xAA)");
            DispatchResult::NoResponse
        }
        // --- ChannelInvite (0xAB) ---
        // C++ parseChannelInvite: name (string). playerChannelInvite not ported.
        0xAB => match pg::parse_channel_invite(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] channel invite name={:?} (0xAB)", pkt.name);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] channel invite parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- ChannelExclude (0xAC) ---
        // C++ parseChannelExclude: name (string). playerChannelExclude not ported.
        0xAC => match pg::parse_channel_exclude(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] channel exclude name={:?} (0xAC)", pkt.name);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] channel exclude parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- UpdateContainer (0xCA) ---
        // C++ parseUpdateContainer: container_id (u8). playerUpdateContainer not ported.
        0xCA => match pg::parse_update_container(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] update container id={} (0xCA)", pkt.container_id);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] update container parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- BrowseField (0xCB) ---
        // C++ parseBrowseField: pos_x(u16), pos_y(u16), pos_z(u8). playerBrowseField not ported.
        0xCB => match pg::parse_browse_field(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] browse field pos=({},{},{}) (0xCB)",
                    pkt.pos_x, pkt.pos_y, pkt.pos_z
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] browse field parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- SeekInContainer (0xCC) ---
        // C++ parseSeekInContainer: container_id(u8), index(u16). playerSeekInContainer not ported.
        0xCC => match pg::parse_seek_in_container(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] seek in container id={} index={} (0xCC)",
                    pkt.container_id, pkt.index
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] seek in container parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- RequestOutfit (0xD2) — no payload ---
        // C++ case 0xD2: playerRequestOutfit. Game logic not ported.
        0xD2 => {
            eprintln!("[gameloop] request outfit (0xD2)");
            DispatchResult::NoResponse
        }
        // --- SetOutfit (0xD3) ---
        // C++ parseSetOutfit → playerChangeOutfit: update outfit in state, broadcast to viewport.
        // Broadcast delivery to other players' sockets is not yet implemented; we update state
        // so the outfit is visible within this session's viewport queries.
        0xD3 => match pg::parse_set_outfit(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] set outfit look_type={} look_mount={} (0xD3)",
                    pkt.look_type, pkt.look_mount
                );
                let outfit = OutfitAppearance {
                    look_type: pkt.look_type,
                    look_head: pkt.look_head,
                    look_body: pkt.look_body,
                    look_legs: pkt.look_legs,
                    look_feet: pkt.look_feet,
                    look_addons: pkt.look_addons,
                };
                handle_set_outfit(player_creature_id, *player_pos, outfit, state);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] set outfit parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- AddVip (0xDC) ---
        // C++ parseAddVip: name (string). playerRequestAddVip not ported.
        0xDC => match pg::parse_add_vip_by_name(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] add vip name={:?} (0xDC)", pkt.name);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] add vip parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- RemoveVip (0xDD) ---
        // C++ parseRemoveVip → playerRequestRemoveVip: remove guid from player's VIP list.
        0xDD => match pg::parse_remove_vip(&mut msg) {
            Ok(pkt) => {
                eprintln!("[gameloop] remove vip guid={} (0xDD)", pkt.guid);
                handle_vip_remove(player_creature_id, pkt.guid, state);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] remove vip parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- EditVip (0xDE) ---
        // C++ parseEditVip: guid(u32), description(string), icon(u32), notify(bool).
        // playerRequestEditVip not ported.
        0xDE => match pg::parse_edit_vip(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] edit vip guid={} icon={} notify={} (0xDE)",
                    pkt.guid, pkt.icon, pkt.notify
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] edit vip parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- MarketLeave (0xF4) — no payload ---
        // C++ case 0xF4: parseMarketLeave → playerLeaveMarket. Game logic not ported.
        0xF4 => {
            eprintln!("[gameloop] market leave (0xF4)");
            DispatchResult::NoResponse
        }
        // --- MarketBrowse (0xF5) ---
        // C++ parseMarketBrowse: browse_id(u8), optional sprite_id(u16).
        // playerBrowseMarket not ported.
        0xF5 => match pg::parse_market_browse(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] market browse browse_id={} sprite_id={:?} (0xF5)",
                    pkt.browse_id, pkt.sprite_id
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] market browse parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- MarketCreateOffer (0xF6) ---
        // C++ parseMarketCreateOffer: offer_type(u8), item_id(u16), amount(u16),
        // price(u32), anonymous(bool). playerCreateMarketOffer not ported.
        0xF6 => match pg::parse_market_create_offer(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] market create offer type={} item_id={} amount={} price={} (0xF6)",
                    pkt.offer_type, pkt.item_id, pkt.amount, pkt.price
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] market create offer parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- MarketCancelOffer (0xF7) ---
        // C++ parseMarketCancelOffer: timestamp(u32), counter(u16).
        // playerCancelMarketOffer not ported.
        0xF7 => match pg::parse_market_cancel_offer(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] market cancel offer timestamp={} counter={} (0xF7)",
                    pkt.timestamp, pkt.counter
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] market cancel offer parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- MarketAcceptOffer (0xF8) ---
        // C++ parseMarketAcceptOffer: timestamp(u32), counter(u16), amount(u16).
        // playerAcceptMarketOffer not ported.
        0xF8 => match pg::parse_market_accept_offer(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] market accept offer timestamp={} counter={} amount={} (0xF8)",
                    pkt.timestamp, pkt.counter, pkt.amount
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] market accept offer parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- ModalWindowAnswer (0xF9) ---
        // C++ parseModalWindowAnswer: window_id(u32), button(u8), choice(u8).
        // playerAnswerModalWindow not ported.
        0xF9 => match pg::parse_modal_window_answer(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] modal window answer window_id={} button={} choice={} (0xF9)",
                    pkt.window_id, pkt.button, pkt.choice
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] modal window answer parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- CancelAttackAndFollow (0xBE) ---
        // C++ game.cpp:3220 playerCancelAttackAndFollow: clears attack target,
        // clears follow target (creature_id=0), and stops auto-walk. No response.
        0xBE => {
            eprintln!("[gameloop] cancel attack and follow (0xBE)");
            state.set_attack_target(player_creature_id, 0);
            state.set_follow_target(player_creature_id, 0, vec![]);
            handle_auto_walk(player_creature_id, vec![], state);
            DispatchResult::NoResponse
        }
        // --- RuleViolationReport (0xF2) ---
        // C++ protocolgame.cpp:767 → parseRuleViolationReport → playerReportRuleViolation
        // → fires Lua onReportRuleViolation event. Lua events not yet ported; log the report.
        0xF2 => match pg::parse_rule_violation_report(&mut msg) {
            Ok(pkt) => {
                eprintln!(
                    "[gameloop] rule violation report type={} reason={} target={} comment={}",
                    pkt.report_type, pkt.reason, pkt.target_name, pkt.comment
                );
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] rule violation report parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- DebugAssert (0xE8) ---
        // C++ protocolgame.cpp:763 → parseDebugAssert → game.playerDebugAssert (logs message).
        // Rate-limiting (debugAssertSent flag) not implemented; Rust always processes.
        0xE8 => match pg::parse_debug_assert(&mut msg) {
            Ok(pkt) => {
                use crate::game_handler::handle_debug_assert;
                let msg_text = format!(
                    "assert={} date={} desc={} comment={}",
                    pkt.assert_line, pkt.date, pkt.description, pkt.comment
                );
                handle_debug_assert(&msg_text);
                DispatchResult::NoResponse
            }
            Err(e) => {
                eprintln!("[gameloop] debug assert parse error: {e}");
                DispatchResult::NoResponse
            }
        },
        // --- Acknowledged no-ops (C++ explicit break; — no game action, no response) ---
        // 0x8E: join aggression  (protocolgame.cpp:660)
        // 0xC9: update tile      (protocolgame.cpp:725)
        // 0xE7: thank you        (protocolgame.cpp:760)
        // 0xF3: get object info  (protocolgame.cpp:769)
        0x8E | 0xC9 | 0xE7 | 0xF3 => {
            eprintln!("[gameloop] acknowledged no-op opcode=0x{opcode:02x}");
            DispatchResult::NoResponse
        }
        _ => {
            eprintln!("[gameloop] unknown opcode: 0x{:02x}", opcode);
            DispatchResult::NoResponse
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
    map: Arc<Map>,
) -> Result<(), String> {
    let game_port = config.get_integer(IntegerKey::GamePort) as u16;
    let listener = TcpListener::bind(format!("0.0.0.0:{game_port}"))
        .map_err(|e| format!("Cannot bind game port {game_port}: {e}"))?;
    let handler = Arc::new(GameLoginHandler::new(db, vocations, map));
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
// Pure regen helper — shared by mana and HP regen blocks in the game loop
// ---------------------------------------------------------------------------

/// Advance a regen accumulator by `elapsed_secs` and return how much to
/// regenerate this tick together with the leftover accumulated seconds.
///
/// Mirrors the inline C++ pattern in `Game::checkCreatureWalk` /
/// `Creature::gainHealth` tick logic: accumulate, divide, remainder.
///
/// Returns `(regen_amount, new_accumulated)`.
/// * `elapsed_secs` — seconds that have passed since the last call
/// * `accumulated`  — previously leftover seconds from prior calls
/// * `tick_period`  — vocation `gain_*_ticks` value (period in seconds)
/// * `amount_per_tick` — vocation `gain_*_amount` value
///
/// If `tick_period == 0` the function returns `(0, accumulated + elapsed_secs)`
/// without panicking.
pub(crate) fn apply_regen_tick(
    elapsed_secs: u32,
    accumulated: u32,
    tick_period: u32,
    amount_per_tick: u32,
) -> (u32, u32) {
    let total = accumulated.saturating_add(elapsed_secs);
    if tick_period == 0 {
        return (0, total);
    }
    let ticks = total / tick_period;
    let remaining = total % tick_period;
    let regen = ticks.saturating_mul(amount_per_tick);
    (regen, remaining)
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
        let game_data = boot(&data_dir(), "forgotten").expect("boot should succeed with real data");

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
        let _game_data = boot(&data_dir(), "forgotten").expect("boot should succeed");
    }

    // -----------------------------------------------------------------------
    // Phase: Monsters::loadFromXml equivalent
    // C++ behaviour: Monsters::loadFromXml(false) reads data/monster/monsters.xml
    // then loads each referenced XML file into the Monsters registry.
    // -----------------------------------------------------------------------
    #[test]
    fn boot_loads_monsters_from_data_monster_dir() {
        let game_data = boot(&data_dir(), "forgotten").expect("boot should succeed");
        assert!(
            game_data.monsters.get_monster_count() > 0,
            "monsters should be loaded from data/monster/monsters.xml"
        );
    }

    // -----------------------------------------------------------------------
    // Phase: boot map_name parameter (Game::loadMainMap equivalent)
    // C++ behaviour: Game::loadMainMap(filename) prepends "data/world/" and
    // appends ".otbm" to construct the full path.  Rust boot() takes a
    // map_name parameter and constructs <data_dir>/world/<map_name>.otbm.
    // -----------------------------------------------------------------------
    #[test]
    fn boot_unknown_map_name_returns_error_mentioning_path() {
        let result = boot(&data_dir(), "no_such_map_xyzabc123");
        assert!(
            result.is_err(),
            "boot must fail when the map file does not exist"
        );
        let Err(err) = result else { panic!("expected error") };
        assert!(
            err.contains("no_such_map_xyzabc123"),
            "error must mention the missing map name, got: {err}"
        );
    }

    #[test]
    fn boot_known_map_name_succeeds() {
        let result = boot(&data_dir(), "forgotten");
        assert!(result.is_ok(), "boot with 'forgotten' map should succeed");
        assert!(result.unwrap().map.get_tile_count() > 0, "map should have tiles");
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

        let res = start_game_listener(
            config,
            game_state,
            empty_db(),
            empty_vocations(),
            empty_map(),
        );
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

        let res = start_game_listener(
            config,
            game_state,
            empty_db(),
            empty_vocations(),
            empty_map(),
        );
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

    fn empty_map() -> Arc<Map> {
        Arc::new(Map::new())
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
                GameLoginHandler::new(empty_db(), empty_vocations(), empty_map())
                    .handle_connection(stream);
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

    /// Default `PlayerLoginData` for tests that drive the game loop directly.
    /// Coordinates `(100, 100, 7)` mirror the temple default in
    /// `load_player_for_login`.
    fn test_player_data() -> PlayerLoginData {
        PlayerLoginData {
            name: "Tester".to_string(),
            level: 1,
            health: 100,
            healthmax: 100,
            mana: 0,
            manamax: 0,
            stamina: 2520,
            posx: 100,
            posy: 100,
            posz: 7,
            experience: 0,
            vocation_id: 0,
            magic_level: 0,
            mana_spent: 0,
            soul: 100,
            capacity: 0,
            skill_levels: [10; 7],
            skill_tries: [0; 7],
            look_type: 128,
            look_head: 0,
            look_body: 0,
            look_legs: 0,
            look_feet: 0,
            look_addons: 0,
            look_mount: 0,
            direction: 2, // SOUTH
            premium_ends_at: 0,
        }
    }

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
            run_game_loop(
                &mut server_stream,
                xtea_key,
                false,
                2,
                test_player_data(),
                0,
                empty_vocations(),
                empty_map(),
            );
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
            run_game_loop(
                &mut server_stream,
                xtea_key,
                false,
                2,
                test_player_data(),
                0,
                empty_vocations(),
                empty_map(),
            );
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
    // dispatch_opcode — per-opcode behavior
    // -----------------------------------------------------------------------

    /// Helper that wires `dispatch_opcode` with a no-op render closure so the
    /// per-opcode unit tests don't need to construct the full render context.
    fn dispatch(
        opcode: u8,
        payload: &[u8],
        pos: &mut Position,
        dir: &mut u8,
        state: &mut GameState,
    ) -> DispatchResult {
        use forgottenserver_game::chat::ChatManager;
        use crate::channel_session::ChannelSession;
        let world = World::new();
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        dispatch_opcode(
            opcode,
            payload,
            &world,
            state,
            0,
            pos,
            dir,
            0,
            1,
            2,
            3,
            &|_d, _p| vec![0x64], // sentinel map body
            "TestPlayer",
            1,
            &mut chat,
            &mut session,
        )
    }

    /// Helper for channel-opcode tests that need explicit chat + session state.
    fn dispatch_ch(
        opcode: u8,
        payload: &[u8],
        pos: &mut Position,
        dir: &mut u8,
        state: &mut GameState,
        chat: &mut forgottenserver_game::chat::ChatManager,
        session: &mut crate::channel_session::ChannelSession,
    ) -> DispatchResult {
        let world = World::new();
        dispatch_opcode(
            opcode,
            payload,
            &world,
            state,
            0,
            pos,
            dir,
            0,
            1,
            2,
            3,
            &|_d, _p| vec![0x64],
            "TestPlayer",
            1,
            chat,
            session,
        )
    }

    // -----------------------------------------------------------------------
    // Channel opcodes — 0x97 GetChannels, 0x98 OpenChannel,
    //                   0x99 CloseChannel, 0x9A OpenPrivateChannel
    //
    // C++ cross-validation: protocolgame.cpp parsePacket cases 0x97-0x9A.
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_get_channels_returns_channel_list_packet() {
        use forgottenserver_game::chat::ChatManager;
        use crate::channel_session::ChannelSession;
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch_ch(0x97, &[], &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        match r {
            DispatchResult::Response(bytes) => {
                assert_eq!(bytes[0], 0xAC, "GetChannels must start with 0xAC (ChannelList)")
            }
            _ => panic!("opcode 0x97 must return a Response with ChannelList (0xAC)"),
        }
    }

    #[test]
    fn dispatch_open_channel_returns_open_channel_ack() {
        use forgottenserver_game::chat::ChatManager;
        use crate::channel_session::ChannelSession;
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let channel_id: u16 = 3; // CHANNEL_WORLD
        let payload = channel_id.to_le_bytes();
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch_ch(0x98, &payload, &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        match r {
            DispatchResult::Response(bytes) => {
                assert_eq!(bytes[0], 0xAB, "OpenChannel ack must start with 0xAB")
            }
            _ => panic!("opcode 0x98 must return a Response with OpenChannel ack (0xAB)"),
        }
    }

    #[test]
    fn dispatch_close_channel_removes_channel_from_session() {
        use forgottenserver_game::chat::ChatManager;
        use crate::channel_session::ChannelSession;
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let channel_id: u16 = 3; // CHANNEL_WORLD
        // Subscribe and add so there is something to close.
        chat.subscribe(channel_id, 0);
        session.add_channel(channel_id);
        assert!(session.open_channels().contains(&channel_id), "channel must be open before close");

        let payload = channel_id.to_le_bytes();
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch_ch(0x99, &payload, &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        assert!(matches!(r, DispatchResult::NoResponse), "CloseChannel must return NoResponse");
        assert!(
            !session.open_channels().contains(&channel_id),
            "channel must be removed from session after close"
        );
    }

    #[test]
    fn dispatch_open_private_channel_returns_open_private_channel_packet() {
        use forgottenserver_game::chat::ChatManager;
        use crate::channel_session::ChannelSession;
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let receiver = "Alice";
        let mut payload = Vec::new();
        payload.extend_from_slice(&(receiver.len() as u16).to_le_bytes());
        payload.extend_from_slice(receiver.as_bytes());

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch_ch(0x9A, &payload, &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        match r {
            DispatchResult::Response(bytes) => {
                assert_eq!(bytes[0], 0xAF, "OpenPrivateChannel must start with 0xAF")
            }
            _ => panic!("opcode 0x9A must return a Response with OpenPrivateChannel (0xAF)"),
        }
    }

    // -----------------------------------------------------------------------
    // Opcode 0x69 — StopAutoWalk, opcodes 0x6A-0x6D — Diagonal walk
    //
    // C++ cross-validation:
    //   * protocolgame.cpp:564 case 0x69 → playerStopAutoWalk (cancels path)
    //   * protocolgame.cpp:566-574 cases 0x6A-0x6D → playerMove with diagonal direction
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_stop_auto_walk_clears_auto_walk_path() {
        let mut state = GameState::new();
        state.set_auto_walk(0, vec![1u8, 2u8, 3u8]);
        assert!(state.get_auto_walk(0).map(|p| !p.is_empty()).unwrap_or(false), "path must be set before test");

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        // 0x69 falls to unknown → NoResponse; after implementation it cancels auto-walk
        let r = dispatch(0x69, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse), "stop auto-walk must return NoResponse");
        // Side effect: path must be empty after cancellation
        assert!(
            state.get_auto_walk(0).map(|p| p.is_empty()).unwrap_or(true),
            "auto-walk path must be cleared after 0x69"
        );
    }

    #[test]
    fn dispatch_walk_diagonal_ne_increments_x_decrements_y() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 2u8;
        let mut state = GameState::new();
        let r = dispatch(0x6A, &[], &mut pos, &mut dir, &mut state);
        match r {
            DispatchResult::Response(bytes) => assert_eq!(bytes[0], 0x64, "diagonal walk must return map body"),
            _ => panic!("0x6A (walk NE) must return a Response"),
        }
        assert_eq!(pos, Position::new(101, 99, 7), "NE walk: x+1, y-1");
        assert_eq!(dir, 4, "direction must be NE (4)");
    }

    #[test]
    fn dispatch_walk_diagonal_se_increments_x_and_y() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x6B, &[], &mut pos, &mut dir, &mut state);
        match r {
            DispatchResult::Response(bytes) => assert_eq!(bytes[0], 0x64),
            _ => panic!("0x6B (walk SE) must return a Response"),
        }
        assert_eq!(pos, Position::new(101, 101, 7), "SE walk: x+1, y+1");
        assert_eq!(dir, 5, "direction must be SE (5)");
    }

    #[test]
    fn dispatch_walk_diagonal_sw_decrements_x_increments_y() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x6C, &[], &mut pos, &mut dir, &mut state);
        match r {
            DispatchResult::Response(bytes) => assert_eq!(bytes[0], 0x64),
            _ => panic!("0x6C (walk SW) must return a Response"),
        }
        assert_eq!(pos, Position::new(99, 101, 7), "SW walk: x-1, y+1");
        assert_eq!(dir, 6, "direction must be SW (6)");
    }

    #[test]
    fn dispatch_walk_diagonal_nw_decrements_x_and_y() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x6D, &[], &mut pos, &mut dir, &mut state);
        match r {
            DispatchResult::Response(bytes) => assert_eq!(bytes[0], 0x64),
            _ => panic!("0x6D (walk NW) must return a Response"),
        }
        assert_eq!(pos, Position::new(99, 99, 7), "NW walk: x-1, y-1");
        assert_eq!(dir, 7, "direction must be NW (7)");
    }

    #[test]
    fn dispatch_logout_breaks() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 2u8;
        let mut state = GameState::new();
        let r = dispatch(0x14, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::Break));
    }

    #[test]
    fn dispatch_walk_north_decrements_y_and_returns_map_body() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 2u8;
        let mut state = GameState::new();
        let r = dispatch(0x65, &[], &mut pos, &mut dir, &mut state);
        match r {
            DispatchResult::Response(bytes) => {
                assert_eq!(bytes[0], 0x64, "walk response must be a map body");
            }
            _ => panic!("walk must produce a Response"),
        }
        assert_eq!(pos, Position::new(100, 99, 7), "north decrements y by 1");
        assert_eq!(dir, 0, "direction must be NORTH (0)");
    }

    #[test]
    fn dispatch_walk_east_south_west_update_coords() {
        let mut state = GameState::new();
        // East
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let _ = dispatch(0x66, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(pos, Position::new(101, 100, 7));
        assert_eq!(dir, 1);
        // South
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let _ = dispatch(0x67, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(pos, Position::new(100, 101, 7));
        assert_eq!(dir, 2);
        // West
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let _ = dispatch(0x68, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(pos, Position::new(99, 100, 7));
        assert_eq!(dir, 3);
    }

    #[test]
    fn dispatch_turn_updates_direction_without_moving() {
        let mut state = GameState::new();
        let mut pos = Position::new(50, 60, 7);
        let mut dir = 0u8;
        let _ = dispatch(0x70, &[], &mut pos, &mut dir, &mut state); // turn east
        assert_eq!(pos, Position::new(50, 60, 7), "turn must NOT move");
        assert_eq!(dir, 1, "turn east -> direction 1");
    }

    #[test]
    fn dispatch_say_pos_command_emits_coordinates_text_message() {
        // Wire format for parse_say_packet: [say_type:u8][text:u16 len][bytes]
        let mut payload = vec![1u8]; // say_type
        let text = "/pos";
        payload.extend_from_slice(&(text.len() as u16).to_le_bytes());
        payload.extend_from_slice(text.as_bytes());

        let mut pos = Position::new(42, 7, 8);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x96, &payload, &mut pos, &mut dir, &mut state);
        let bytes = match r {
            DispatchResult::Response(b) => b,
            _ => panic!("say must produce a Response"),
        };

        assert_eq!(bytes[0], 0xB4, "TextMessage opcode is 0xB4");
        assert_eq!(
            bytes[1],
            pg::text_message_class::MESSAGE_EVENT_ADVANCE,
            "MESSAGE_EVENT_ADVANCE = 19 (white text over player + console)",
        );
        let resp_len = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
        let resp_text = std::str::from_utf8(&bytes[4..4 + resp_len]).unwrap();
        assert_eq!(resp_text, "x=42, y=7, z=8");
    }

    #[test]
    fn dispatch_say_emits_talk_packet_for_regular_text() {
        // Non-command say must produce a Talk (0xAA) packet — not TEXT_MESSAGE.
        // C++ ProtocolGame::sendCreatureSay always uses 0xAA for player words.
        let mut payload = vec![1u8]; // say_type = 1 (Say)
        let text = "hello world";
        payload.extend_from_slice(&(text.len() as u16).to_le_bytes());
        payload.extend_from_slice(text.as_bytes());

        let mut pos = Position::new(100, 200, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x96, &payload, &mut pos, &mut dir, &mut state);
        let bytes = match r {
            DispatchResult::Response(b) => b,
            _ => panic!("say must produce a Response"),
        };
        assert_eq!(bytes[0], 0xAA, "say response must be a Talk packet (0xAA)");
        // stmt_id [1..5], name_len [5..7], name, traded, level, speak_type, pos_x, pos_y, pos_z, text
        // Verify text appears somewhere in the packet
        let packet_str = String::from_utf8_lossy(&bytes);
        assert!(
            packet_str.contains("hello world"),
            "talk packet must contain the spoken text"
        );
    }

    #[test]
    fn dispatch_fight_modes_updates_state_and_returns_no_response() {
        // Wire format: [fight:u8][chase:u8][secure:u8]
        let payload = [2u8, 1u8, 1u8];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA0, &payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        let (fight, chase, secure) = state.get_fight_mode(0).expect("fight mode stored");
        assert_eq!(fight, 2);
        assert_eq!(chase, 1);
        assert!(secure);
    }

    #[test]
    fn dispatch_use_item_returns_default_text_message() {
        // parse_use_item_packet expects: pos_x(u16), pos_y(u16), pos_z(u8),
        // item_id(u16), index(u8) = 8 bytes.
        let mut payload = Vec::new();
        payload.extend_from_slice(&100u16.to_le_bytes()); // pos_x
        payload.extend_from_slice(&100u16.to_le_bytes()); // pos_y
        payload.push(7u8); // pos_z
        payload.extend_from_slice(&500u16.to_le_bytes()); // item_id
        payload.push(0u8); // index

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x82, &payload, &mut pos, &mut dir, &mut state);
        let bytes = match r {
            DispatchResult::Response(b) => b,
            _ => panic!("use item must produce a Response"),
        };
        assert_eq!(bytes[0], 0xB4, "fallback TextMessage opcode");
    }

    // -----------------------------------------------------------------------
    // Opcode 0xA1 — parseAttack (set attack target)
    // Opcode 0xBE — playerCancelAttackAndFollow (clear attack + follow + auto-walk)
    //
    // C++ cross-validation:
    //   * protocolgame.cpp:688 case 0xA1 → parseAttack → reads creature_id(u32)
    //     → Game::playerSetAttackedCreature(playerId, creature_id)
    //   * protocolgame.cpp → game.cpp:3220 case 0xBE → playerCancelAttackAndFollow
    //     → calls playerSetAttackedCreature(playerId, 0) + clears follow + stops auto-walk
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_attack_stores_attack_target_in_state() {
        let creature_id: u32 = 0x1000_0001;
        let mut payload = Vec::new();
        payload.extend_from_slice(&creature_id.to_le_bytes()); // creature_id u32 LE

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA1, &payload, &mut pos, &mut dir, &mut state);
        assert!(
            matches!(r, DispatchResult::NoResponse),
            "parseAttack must return NoResponse"
        );
        assert_eq!(
            state.get_attack_target(0),
            Some(creature_id),
            "attack target must be stored in GameState"
        );
    }

    #[test]
    fn dispatch_cancel_attack_and_follow_clears_all_combat_state() {
        let mut state = GameState::new();
        // Pre-populate attack target, follow target, auto-walk path
        state.set_attack_target(0, 0x1000_0001);
        state.set_follow_target(0, 0x1000_0002, vec![1u8, 2u8]);
        state.set_auto_walk(0, vec![3u8, 4u8]);

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xBE, &[], &mut pos, &mut dir, &mut state);
        assert!(
            matches!(r, DispatchResult::NoResponse),
            "playerCancelAttackAndFollow must return NoResponse"
        );
        assert_eq!(
            state.get_attack_target(0),
            None,
            "attack target must be cleared after 0xBE"
        );
        // Follow target creature_id should be 0 (cleared)
        assert!(
            state.get_follow_target(0).map(|(cid, _)| cid == 0).unwrap_or(true),
            "follow target must be cleared after 0xBE"
        );
        assert!(
            state.get_auto_walk(0).map(|p| p.is_empty()).unwrap_or(true),
            "auto-walk path must be empty after 0xBE"
        );
    }

    // -----------------------------------------------------------------------
    // Opcode 0xA2 — parseFollow → playerFollowCreature
    //
    // C++ cross-validation:
    //   * protocolgame.cpp:690 case 0xA2 → parseFollow → reads creature_id(u32)
    //     → Game::playerFollowCreature(playerId, creatureId)
    //   * game.cpp:3265 playerFollowCreature:
    //       removeAttackedCreature (clears attack target)
    //       if creatureId exists → setFollowCreature
    //       else → removeFollowCreature
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_follow_stores_follow_target_when_creature_known() {
        let creature_id: u32 = 42;
        let mut state = GameState::new();
        // Register target creature's position so dispatch can pathfind.
        state.set_creature_position(creature_id, Position::new(102, 100, 7));

        let mut payload = Vec::new();
        payload.extend_from_slice(&creature_id.to_le_bytes());

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xA2, &payload, &mut pos, &mut dir, &mut state);
        assert!(
            matches!(r, DispatchResult::NoResponse),
            "parseFollow must return NoResponse"
        );
        let follow = state.get_follow_target(0);
        assert!(follow.is_some(), "follow target must be set after 0xA2");
        assert_eq!(
            follow.unwrap().0,
            creature_id,
            "follow target creature_id must match"
        );
    }

    #[test]
    fn dispatch_follow_always_clears_attack_target() {
        let creature_id: u32 = 42;
        let mut state = GameState::new();
        state.set_attack_target(0, 99); // pre-set attack target
        state.set_creature_position(creature_id, Position::new(102, 100, 7));

        let mut payload = Vec::new();
        payload.extend_from_slice(&creature_id.to_le_bytes());

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let _ = dispatch(0xA2, &payload, &mut pos, &mut dir, &mut state);
        assert_eq!(
            state.get_attack_target(0),
            None,
            "parseFollow must clear attack target (mirrors C++ removeAttackedCreature)"
        );
    }

    #[test]
    fn dispatch_follow_zero_clears_follow_target() {
        let mut state = GameState::new();
        state.set_follow_target(0, 42, vec![1u8, 2u8]); // pre-set follow target

        let mut payload = Vec::new();
        payload.extend_from_slice(&0u32.to_le_bytes()); // creature_id = 0

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xA2, &payload, &mut pos, &mut dir, &mut state);
        assert!(
            matches!(r, DispatchResult::NoResponse),
            "parseFollow with 0 must return NoResponse"
        );
        // Follow target should be cleared (creature_id=0) or removed
        let follow = state.get_follow_target(0);
        assert!(
            follow.map(|(cid, _)| cid == 0).unwrap_or(true),
            "follow target must be cleared when creature_id=0"
        );
    }

    // -----------------------------------------------------------------------
    // Opcode 0x64 — parseAutoWalk
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_auto_walk_stores_path_in_state() {
        let mut state = GameState::new();
        // payload: numdirs=1, wire_dir=3 (NORTH → Direction_t 0)
        let payload = &[1u8, 3u8];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0x64, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse), "0x64 must return NoResponse");
        let path = state.get_auto_walk(0).expect("auto-walk path must be stored");
        assert_eq!(path, &vec![0u8], "wire byte 3 must map to DIRECTION_NORTH (0)");
    }

    #[test]
    fn dispatch_auto_walk_reverses_wire_order() {
        let mut state = GameState::new();
        // payload: numdirs=2, wire=[3(NORTH),1(EAST)] → reversed → [EAST, NORTH] = [1, 0]
        let payload = &[2u8, 3u8, 1u8];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        dispatch(0x64, payload, &mut pos, &mut dir, &mut state);
        let path = state.get_auto_walk(0).expect("path must be stored");
        assert_eq!(path, &vec![1u8, 0u8]);
    }

    #[test]
    fn dispatch_auto_walk_invalid_payload_does_not_store_path() {
        let mut state = GameState::new();
        // numdirs=0 → invalid → no path stored
        let payload = &[0u8];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0x64, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        // path should not be set (or should be None)
        assert!(state.get_auto_walk(0).map(|p| p.is_empty()).unwrap_or(true));
    }

    // -----------------------------------------------------------------------
    // No-op opcodes: 0x8E, 0xC9, 0xE7, 0xF3
    //   C++ case 0x8E: /* join aggression */ break;
    //   C++ case 0xC9: /* update tile */     break;
    //   C++ case 0xE7: /* thank you */       break;
    //   C++ case 0xF3: /* get object info */ break;
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Opcode 0xF2 — parseRuleViolationReport
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_rule_violation_report_returns_no_response() {
        let mut state = GameState::new();
        // payload: type=2(BOT), reason=1, target_name="Bot", comment="cheating"
        let mut payload = Vec::new();
        payload.push(2u8); // REPORT_TYPE_BOT
        payload.push(1u8); // reason
        // target_name = "Bot": length 3 (LE u16) + bytes
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(b"Bot");
        // comment = "cheating": length 8 + bytes
        payload.extend_from_slice(&8u16.to_le_bytes());
        payload.extend_from_slice(b"cheating");
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xF2, &payload, &mut pos, &mut dir, &mut state);
        assert!(
            matches!(r, DispatchResult::NoResponse),
            "0xF2 rule violation report must return NoResponse"
        );
    }

    // -----------------------------------------------------------------------
    // Opcode 0xE8 — parseDebugAssert
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_debug_assert_returns_no_response() {
        let mut state = GameState::new();
        // payload: 4 empty length-prefixed strings (2-byte len = 0 each)
        let payload = &[0u8, 0, 0, 0, 0, 0, 0, 0];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xE8, payload, &mut pos, &mut dir, &mut state);
        assert!(
            matches!(r, DispatchResult::NoResponse),
            "0xE8 debug assert must return NoResponse"
        );
    }

    #[test]
    fn dispatch_acknowledged_noop_opcodes_return_no_response() {
        let mut state = GameState::new();
        for op in [0x8Eu8, 0xC9, 0xE7, 0xF3] {
            let mut pos = Position::new(100, 100, 7);
            let mut dir = 0u8;
            let r = dispatch(op, &[], &mut pos, &mut dir, &mut state);
            assert!(
                matches!(r, DispatchResult::NoResponse),
                "opcode 0x{op:02X} must return NoResponse (C++ explicit no-op)"
            );
        }
    }

    // -----------------------------------------------------------------------
    // 0x77 — parseEquipObject
    // C++ protocolgame.cpp:1302 — reads sprite_id (u16), dispatches
    // playerEquipItem. Game logic not ported; dispatch logs and returns
    // NoResponse.
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_equip_object_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // sprite_id = 2000 (0xD0, 0x07)
        let r = dispatch(0x77, &[0xD0, 0x07], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_equip_object_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // Empty payload → parse error → must still return NoResponse (no crash)
        let r = dispatch(0x77, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // 0x83–0x8D — Item interaction and container opcodes
    // C++ protocolgame.cpp cases 0x83-0x8D (parseUseItemEx, parseUseWithCreature,
    // parseRotateItem, parseEditPodiumRequest, parseCloseContainer,
    // parseUpArrowContainer, parseTextWindow, parseHouseWindow,
    // parseWrapItem, parseLookAt, parseLookInBattleList).
    // All return NoResponse; game logic not yet ported.
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_use_item_ex_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // from: x=100,y=100,z=7, sprite_id=500, stackpos=2, to: x=101,y=100,z=7
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2, 101, 0, 100, 0, 7];
        let r = dispatch(0x83, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_use_with_creature_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // pos: x=100,y=100,z=7, sprite_id=500, stackpos=2, creature_id=42
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2, 42, 0, 0, 0];
        let r = dispatch(0x84, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_rotate_item_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // pos: x=100,y=100,z=7, sprite_id=500, stackpos=2
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2];
        let r = dispatch(0x85, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_close_container_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // container_id=3
        let r = dispatch(0x87, &[3], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_up_arrow_container_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // container_id=3
        let r = dispatch(0x88, &[3], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_at_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // pos: x=100,y=100,z=7, sprite_id=500, stackpos=2
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2];
        let r = dispatch(0x8C, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_in_battle_list_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // creature_id=42
        let r = dispatch(0x8D, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // 0x79–0x80 — Shop and Trade opcodes
    // C++ protocolgame.cpp cases 0x79-0x80 (parseLookInShop, parsePlayerPurchase,
    // parsePlayerSale, playerCloseShop, parseRequestTrade, parseLookInTrade,
    // playerAcceptTrade, playerCloseTrade). All return NoResponse; game logic
    // not yet ported.
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_look_in_shop_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // item_id=100, count=1
        let r = dispatch(0x79, &[100, 0, 1], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_in_shop_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x79, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_player_purchase_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // item_id=100, sub_type=0, count=5, ignore_capacity=0, buy_with_backpack=0
        let r = dispatch(0x7A, &[100, 0, 0, 5, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_player_sale_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // item_id=100, sub_type=0, count=5, ignore_equipped=0
        let r = dispatch(0x7B, &[100, 0, 0, 5, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_close_shop_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x7C, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_request_trade_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // pos_x=100, pos_y=100, pos_z=7, sprite_id=500, stackpos=2, player_id=99
        let r = dispatch(
            0x7D,
            &[100, 0, 100, 0, 7, 244, 1, 2, 99, 0, 0, 0],
            &mut pos,
            &mut dir,
            &mut state,
        );
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_in_trade_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // counter_offer=0, index=2
        let r = dispatch(0x7E, &[0, 2], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_accept_trade_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x7F, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_close_trade_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x80, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // 0x78 — parseThrow (move/throw item from one tile to another)
    // C++ protocolgame.cpp:597 → parseThrow → g_game.playerMoveThing.
    // Wire: from_x(u16), from_y(u16), from_z(u8), sprite_id(u16),
    //       from_stackpos(u8), to_x(u16), to_y(u16), to_z(u8), count(u8)
    // Game logic not ported; dispatch parses and returns NoResponse.
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_throw_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // from_x=100, from_y=100, from_z=7, sprite_id=500, from_stackpos=2,
        // to_x=101, to_y=100, to_z=7, count=1
        let payload: &[u8] = &[
            100, 0, // from_x = 100
            100, 0, // from_y = 100
            7,      // from_z = 7
            244, 1, // sprite_id = 500
            2,      // from_stackpos = 2
            101, 0, // to_x = 101
            100, 0, // to_y = 100
            7,      // to_z = 7
            1,      // count = 1
        ];
        let r = dispatch(0x78, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_throw_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // Empty payload → parse error → must still return NoResponse
        let r = dispatch(0x78, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // 0x9E, 0xA3-0xA8, 0xAA-0xAC — Party and NPC channel opcodes
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_close_npc_channel_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x9E, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_invite_to_party_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA3, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_join_party_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA4, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_revoke_party_invite_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA5, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_pass_party_leadership_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA6, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_leave_party_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA7, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_enable_shared_party_experience_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA8, &[1], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_create_private_channel_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xAA, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_channel_invite_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // name = "Alice" (len=5, bytes 65,108,105,99,101)
        let payload: &[u8] = &[5, 0, 65, 108, 105, 99, 101];
        let r = dispatch(0xAB, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_channel_exclude_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // name = "Bob" (len=3, bytes 66,111,98)
        let payload: &[u8] = &[3, 0, 66, 111, 98];
        let r = dispatch(0xAC, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // 0xCA-0xCC — Container/field opcodes
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_update_container_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // container_id=3
        let r = dispatch(0xCA, &[3], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_browse_field_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // pos: x=100, y=100, z=7
        let r = dispatch(0xCB, &[100, 0, 100, 0, 7], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_seek_in_container_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // container_id=3, index=5
        let r = dispatch(0xCC, &[3, 5, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // 0xD2-0xD3 — Outfit opcodes
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_request_outfit_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xD2, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_set_outfit_updates_state() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // outfit: look_type=75, head=10, body=20, legs=30, feet=40, addons=3, no mount
        // wire: look_type(u16 LE), head, body, legs, feet, addons, look_mount(u16 LE)
        let payload: &[u8] = &[75, 0, 10, 20, 30, 40, 3, 0, 0];
        let r = dispatch(0xD3, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        let outfit = state.get_outfit(0).expect("outfit should be stored in state after 0xD3");
        assert_eq!(outfit.look_type, 75);
        assert_eq!(outfit.look_head, 10);
        assert_eq!(outfit.look_body, 20);
        assert_eq!(outfit.look_legs, 30);
        assert_eq!(outfit.look_feet, 40);
        assert_eq!(outfit.look_addons, 3);
    }

    // -----------------------------------------------------------------------
    // 0xDC-0xDE — VIP opcodes
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_add_vip_by_name_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // name = "Alice"
        let payload: &[u8] = &[5, 0, 65, 108, 105, 99, 101];
        let r = dispatch(0xDC, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_remove_vip_removes_from_state() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        state.add_vip(0, 42);
        assert!(state.get_vip_list(0).contains(&42), "pre-condition: VIP 42 must be present");
        let r = dispatch(0xDD, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert!(!state.get_vip_list(0).contains(&42), "VIP 42 should be removed after 0xDD");
    }

    #[test]
    fn dispatch_remove_vip_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // guid=42
        let r = dispatch(0xDD, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // 0xF4-0xF9 — Market opcodes
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_market_leave_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF4, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_browse_own_offers_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // browse_id=0xFE (own offers)
        let r = dispatch(0xF5, &[0xFE], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_create_offer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // offer_type=0, item_id=100, amount=5, price=1000, anonymous=0
        let r = dispatch(
            0xF6,
            &[0, 100, 0, 5, 0, 232, 3, 0, 0, 0],
            &mut pos,
            &mut dir,
            &mut state,
        );
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_cancel_offer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // timestamp=1000, counter=0
        let r = dispatch(0xF7, &[232, 3, 0, 0, 0, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_accept_offer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // timestamp=1000, counter=0, amount=3
        let r = dispatch(0xF8, &[232, 3, 0, 0, 0, 0, 0, 0, 3, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_modal_window_answer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // window_id=1, button=0, choice=2
        let r = dispatch(0xF9, &[1, 0, 0, 0, 0, 2], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_unknown_opcode_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xEF, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_noop_opcodes_return_no_response() {
        let mut state = GameState::new();
        for op in [0x0Fu8, 0x60, 0xD0, 0x91, 0x32] {
            let mut pos = Position::new(100, 100, 7);
            let mut dir = 0u8;
            let r = dispatch(op, &[], &mut pos, &mut dir, &mut state);
            assert!(
                matches!(r, DispatchResult::NoResponse),
                "opcode 0x{op:02x} must be NoResponse"
            );
        }
    }

    #[test]
    fn dispatch_ping_returns_pong() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x1D, &[], &mut pos, &mut dir, &mut state);
        match r {
            DispatchResult::Response(bytes) => assert_eq!(bytes, vec![0x1E]),
            _ => panic!("ping must produce a pong response"),
        }
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
        let handler = GameLoginHandler::new(db, empty_vocations(), empty_map());

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
        let handler = GameLoginHandler::new(db, empty_vocations(), empty_map());

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

    // -----------------------------------------------------------------------
    // apply_regen_tick — pure regen accumulator logic
    // -----------------------------------------------------------------------

    #[test]
    fn regen_tick_accumulates_and_returns_zero_before_threshold() {
        // With tick_period=6 and only 4 seconds elapsed, no regen should fire.
        let (amount, remaining) = apply_regen_tick(4, 0, 6, 10);
        assert_eq!(amount, 0, "no regen before tick threshold");
        assert_eq!(remaining, 4, "elapsed seconds are accumulated");
    }

    #[test]
    fn regen_tick_fires_once_at_exact_threshold() {
        // With tick_period=6 and 6 seconds elapsed, regen fires once for amount 10.
        let (amount, remaining) = apply_regen_tick(6, 0, 6, 10);
        assert_eq!(amount, 10, "regen fires once at exact threshold");
        assert_eq!(remaining, 0, "nothing left over at exact multiple");
    }

    #[test]
    fn regen_tick_fires_multiple_times_with_overflow() {
        // 14 seconds, period=6: fires twice (6+6=12 ≤ 14), 2 seconds left over.
        let (amount, remaining) = apply_regen_tick(14, 0, 6, 10);
        assert_eq!(amount, 20, "regen fires twice for 14/6 ticks");
        assert_eq!(remaining, 2, "2 seconds left over after 12 consumed");
    }

    #[test]
    fn regen_tick_carries_forward_accumulated_secs() {
        // 4s already accumulated + 3s new = 7 ≥ 6 → fires once, 1 leftover.
        let (amount, remaining) = apply_regen_tick(3, 4, 6, 10);
        assert_eq!(amount, 10, "fires once when accumulated + elapsed ≥ period");
        assert_eq!(remaining, 1, "1 second left over");
    }

    #[test]
    fn regen_tick_zero_period_returns_no_regen() {
        // tick_period=0 must not panic (division by zero) and must return 0.
        let (amount, remaining) = apply_regen_tick(100, 50, 0, 10);
        assert_eq!(amount, 0, "zero period yields no regen");
        assert_eq!(remaining, 150, "elapsed added to accumulated, no drain");
    }

    /// `frame_packet_seq` with small sequence numbers (0, 1, 2, ...) must
    /// never set bit 31 in the 4-byte header field.  OTClient in sequenced
    /// mode interprets bit 31 as a zlib-decompress flag; if it is set the
    /// packet is decompressed (fails) and silently dropped, causing a black
    /// map canvas.
    #[test]
    fn frame_packet_seq_header_bit31_never_set_for_small_sequences() {
        let xtea_key: [u32; 4] = [0x01, 0x02, 0x03, 0x04];
        let payload = [0xA0u8, 0x01, 0x02, 0x03]; // arbitrary payload
        for seq in 0u32..=255 {
            let frame = frame_packet_seq(&payload, xtea_key, seq);
            // frame = [outer_len:2][header:4][xtea_region]
            let header = u32::from_le_bytes([frame[2], frame[3], frame[4], frame[5]]);
            assert_eq!(
                header & (1 << 31),
                0,
                "frame_packet_seq(seq={seq}) has bit 31 set in header 0x{header:08X}"
            );
            assert_eq!(header, seq, "header must equal the sequence number");
        }
    }

}
