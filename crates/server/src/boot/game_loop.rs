use std::path::Path;
use std::sync::Arc;

use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::position::Position;
use forgottenserver_common::tools::adler_checksum;
use forgottenserver_common::xtea;
use forgottenserver_database::iologindata::PlayerLoginData;
use forgottenserver_entity::player::{base_speed, Player};
use forgottenserver_game::chat::ChatManager;
use forgottenserver_items::vocation::{Vocation, Vocations};
use forgottenserver_scripting::actions::Actions;
use forgottenserver_scripting::talkaction::TalkActions;
use forgottenserver_world::map::Map;
use forgottenserver_network::protocolgame as pg;
use forgottenserver_world::World;

use crate::boot::framing::{frame_packet, frame_packet_seq};
use crate::channel_session::ChannelSession;
use crate::codec::{encode, ServerPacket};
use crate::game_handler::{
    build_map_around_player, handle_auto_walk, handle_close_channel, handle_fight_modes,
    handle_follow, handle_get_channels, handle_open_channel, handle_open_private_channel,
    handle_set_outfit, handle_vip_remove,
};
use crate::game_state::{GameState, OutfitAppearance};

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
    /// Send multiple independent packets, each framed separately via XTEA.
    MultiResponse(Vec<Vec<u8>>),
    /// Exit the game loop cleanly (logout).
    Break,
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
    talk_actions: Arc<TalkActions>,
    script_dir: std::path::PathBuf,
    actions: Arc<Actions>,
    action_data_dir: std::path::PathBuf,
) -> LogoutSave {
    use std::io::{Read, Write};

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
        if outer_len < 12 {
            eprintln!("[gameloop] skip: outer_len={outer_len} < 12");
            continue;
        }
        let xtea_region_len = outer_len - 4;
        if !xtea_region_len.is_multiple_of(8) {
            eprintln!("[gameloop] skip: xtea_region_len={xtea_region_len} not multiple of 8");
            continue;
        }

        // --- Step 3: validate the 4-byte checksum/sequence field ---
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
            &map,
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
            &talk_actions,
            &script_dir,
            &actions,
            &action_data_dir,
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
            DispatchResult::MultiResponse(packets) => {
                let mut send_failed = false;
                for bytes in packets {
                    let seq = server_seq;
                    server_seq = server_seq.wrapping_add(1);
                    let frame = if sequence_checksum {
                        frame_packet_seq(&bytes, xtea_key, seq)
                    } else {
                        frame_packet(&bytes, xtea_key)
                    };
                    if let Err(e) = stream.write_all(&frame) {
                        eprintln!(
                            "[gameloop] exit: failed to send multi-response opcode=0x{opcode:02x}: {e}"
                        );
                        send_failed = true;
                        break;
                    }
                }
                if send_failed {
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

/// Mirrors C++ `Tile::queryDestination`: given the player's computed
/// destination `(x, y, z)`, redirects through any FLOORCHANGE tile to the
/// correct final position.
///
/// Returns the (possibly adjusted) `(x, y, z)` the player should land on.
fn query_destination(map: &Map, x: u16, y: u16, z: u8) -> (u16, u16, u8) {
    use forgottenserver_map::tile::flags;

    let Some(tile) = map.get_tile(x, y, z) else {
        return (x, y, z);
    };

    if tile.has_flag(flags::FLOORCHANGE_DOWN) {
        // Going to a lower floor (z+1). Look at the tile on the floor below
        // to determine the exact landing coordinates.
        let dz = z.saturating_add(1);
        let mut dx = x;
        let mut dy = y;

        // South-alt staircase on the tile south of (dx, dy-1, dz)
        let south_down = dy.checked_sub(1).and_then(|sy| map.get_tile(dx, sy, dz));
        if south_down.is_some_and(|t| t.has_flag(flags::FLOORCHANGE_SOUTH_ALT)) {
            dy = dy.wrapping_sub(2);
            return (dx, dy, dz);
        }

        // East-alt staircase on the tile west of (dx-1, dy, dz)
        let east_down = dx.checked_sub(1).and_then(|sx| map.get_tile(sx, dy, dz));
        if east_down.is_some_and(|t| t.has_flag(flags::FLOORCHANGE_EAST_ALT)) {
            dx = dx.wrapping_sub(2);
            return (dx, dy, dz);
        }

        // Regular floor-down: directional offset from the below tile's flags
        if let Some(down_tile) = map.get_tile(dx, dy, dz) {
            if down_tile.has_flag(flags::FLOORCHANGE_NORTH) {
                dy = dy.wrapping_add(1);
            }
            if down_tile.has_flag(flags::FLOORCHANGE_SOUTH) {
                dy = dy.wrapping_sub(1);
            }
            if down_tile.has_flag(flags::FLOORCHANGE_SOUTH_ALT) {
                dy = dy.wrapping_sub(2);
            }
            if down_tile.has_flag(flags::FLOORCHANGE_EAST) {
                dx = dx.wrapping_sub(1);
            }
            if down_tile.has_flag(flags::FLOORCHANGE_EAST_ALT) {
                dx = dx.wrapping_sub(2);
            }
            if down_tile.has_flag(flags::FLOORCHANGE_WEST) {
                dx = dx.wrapping_add(1);
            }
        }
        (dx, dy, dz)
    } else if tile.has_floor_change() {
        // Going to an upper floor (z-1).
        let dz = z.wrapping_sub(1);
        let mut dx = x;
        let mut dy = y;

        if tile.has_flag(flags::FLOORCHANGE_NORTH) {
            dy = dy.wrapping_sub(1);
        }
        if tile.has_flag(flags::FLOORCHANGE_SOUTH) {
            dy = dy.wrapping_add(1);
        }
        if tile.has_flag(flags::FLOORCHANGE_EAST) {
            dx = dx.wrapping_add(1);
        }
        if tile.has_flag(flags::FLOORCHANGE_WEST) {
            dx = dx.wrapping_sub(1);
        }
        if tile.has_flag(flags::FLOORCHANGE_SOUTH_ALT) {
            dy = dy.wrapping_add(2);
        }
        if tile.has_flag(flags::FLOORCHANGE_EAST_ALT) {
            dx = dx.wrapping_add(2);
        }
        (dx, dy, dz)
    } else {
        (x, y, z)
    }
}

/// Dispatch a single client opcode and produce a response action.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_opcode<F>(
    opcode: u8,
    payload_slice: &[u8],
    _world: &World,
    map: &Map,
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
    talk_actions: &TalkActions,
    script_dir: &Path,
    actions: &Actions,
    action_data_dir: &Path,
) -> DispatchResult
where
    F: Fn(u8, Position) -> Vec<u8>,
{
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload_slice);
    msg.set_buffer_position(0);

    match opcode {
        0x14 => {
            eprintln!("[gameloop] logout (0x14)");
            DispatchResult::Break
        }
        0x0F | 0x60 | 0xD0 | 0x91 => {
            eprintln!("[gameloop] no-op opcode=0x{opcode:02x}");
            DispatchResult::NoResponse
        }
        0x1D => DispatchResult::Response(vec![0x1E]),
        0x1E => {
            eprintln!("[gameloop] pong (0x1E) received");
            DispatchResult::NoResponse
        }
        0x32 => {
            eprintln!(
                "[gameloop] extended opcode (0x32): {} bytes",
                payload_slice.len()
            );
            DispatchResult::NoResponse
        }
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
        0x65..=0x68 | 0x6A..=0x6D => {
            let (dx, dy, dir): (i32, i32, u8) = match opcode {
                0x65 => (0, -1, dir_north),
                0x66 => (1, 0, dir_east),
                0x67 => (0, 1, dir_south),
                0x68 => (-1, 0, dir_west),
                0x6A => (1, -1, 4),
                0x6B => (1, 1, 5),
                0x6C => (-1, 1, 6),
                0x6D => (-1, -1, 7),
                _ => unreachable!(),
            };
            let new_x = player_pos.x as i32 + dx;
            let new_y = player_pos.y as i32 + dy;
            if !(0..=u16::MAX as i32).contains(&new_x) || !(0..=u16::MAX as i32).contains(&new_y) {
                eprintln!("[gameloop] walk out of range: ({new_x},{new_y})");
                return DispatchResult::NoResponse;
            }

            // Floor-change detection — mirrors C++ Game::internalMoveCreature.
            // Diagonal moves never trigger floor changes (C++ diagonalMovement guard).
            let current_z = player_pos.z;
            let mut dest_z = current_z;
            let is_diagonal = dir >= 4;
            if !is_diagonal {
                use forgottenserver_map::tile::flags;
                let dest_x = new_x as u16;
                let dest_y = new_y as u16;

                // Try to go up: if current tile has height ≥ 3, check if the
                // floor above the destination is accessible. Disabled at z=8
                // (first underground floor) — mirrors C++ `currentPos.z != 8`.
                if current_z != 8 {
                    if let Some(cur_tile) = map.get_tile(player_pos.x, player_pos.y, current_z) {
                        if cur_tile.has_height(3) {
                            if let Some(up_z) = current_z.checked_sub(1) {
                                let above_cur_clear = map
                                    .get_tile(player_pos.x, player_pos.y, up_z)
                                    .map(|t| t.get_ground().is_none() && !t.has_flag(flags::BLOCKSOLID))
                                    .unwrap_or(true);
                                if above_cur_clear {
                                    if let Some(above_dest) = map.get_tile(dest_x, dest_y, up_z) {
                                        if above_dest.get_ground().is_some()
                                            && !above_dest.has_flag(flags::IMMOVABLEBLOCKSOLID)
                                            && !above_dest.has_floor_change()
                                        {
                                            dest_z = up_z;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Try to go down: if destination tile at same z has no ground,
                // check if the floor below has height ≥ 3. Disabled at z=7
                // (surface) — mirrors C++ `currentPos.z != 7`.
                if current_z != 7 && dest_z == current_z {
                    let dest_same_z_empty = map
                        .get_tile(dest_x, dest_y, current_z)
                        .map(|t| t.get_ground().is_none() && !t.has_flag(flags::BLOCKSOLID))
                        .unwrap_or(true);
                    if dest_same_z_empty {
                        let below_z = current_z.saturating_add(1);
                        if let Some(below_dest) = map.get_tile(dest_x, dest_y, below_z) {
                            if below_dest.has_height(3)
                                && !below_dest.has_flag(flags::IMMOVABLEBLOCKSOLID)
                            {
                                dest_z = below_z;
                            }
                        }
                    }
                }

                // Apply FLOORCHANGE redirect — mirrors C++ queryDestination.
                // When the destination tile (at dest_z) carries a FLOORCHANGE
                // flag the player is redirected to the appropriate floor+coords.
                let (final_x, final_y, final_z) =
                    query_destination(map, new_x as u16, new_y as u16, dest_z);
                let new_pos = Position::new(final_x, final_y, final_z);
                *player_pos = new_pos;
                *player_dir = dir;
                state.set_player_position(player_creature_id, new_pos);
                eprintln!(
                    "[gameloop] walk dir={dir} -> ({}, {}, {})",
                    new_pos.x, new_pos.y, new_pos.z
                );
                return DispatchResult::Response(render_map(dir, new_pos));
            }

            let new_pos = Position::new(new_x as u16, new_y as u16, dest_z);
            *player_pos = new_pos;
            *player_dir = dir;
            state.set_player_position(player_creature_id, new_pos);
            eprintln!(
                "[gameloop] walk dir={dir} -> ({}, {}, {})",
                new_pos.x, new_pos.y, new_pos.z
            );
            DispatchResult::Response(render_map(dir, new_pos))
        }
        0x69 => {
            eprintln!("[gameloop] stop auto-walk (0x69)");
            handle_auto_walk(player_creature_id, vec![], state);
            DispatchResult::NoResponse
        }
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
        0x7C => {
            eprintln!("[gameloop] close shop (0x7C)");
            DispatchResult::NoResponse
        }
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
        0x7F => {
            eprintln!("[gameloop] accept trade (0x7F)");
            DispatchResult::NoResponse
        }
        0x80 => {
            eprintln!("[gameloop] close trade (0x80)");
            DispatchResult::NoResponse
        }
        0x82 => match pg::parse_use_item_packet(&mut msg) {
            Ok(use_pkt) => {
                eprintln!(
                    "[gameloop] use item id={} at ({},{},{}) idx={}",
                    use_pkt.item_id, use_pkt.pos_x, use_pkt.pos_y, use_pkt.pos_z, use_pkt.index
                );
                let item_pos = Position::new(use_pkt.pos_x, use_pkt.pos_y, use_pkt.pos_z);
                if let Some(action) = actions.get_by_item_id(use_pkt.item_id) {
                    let script_path = action_data_dir.join("scripts").join(&action.script_name);
                    let lib_dir = action_data_dir.join("lib");
                    use forgottenserver_scripting::lua_bindings::action_player::{execute_action, ActionContext};
                    match execute_action(
                        &script_path,
                        Some(&lib_dir),
                        ActionContext { item_id: use_pkt.item_id, item_pos, player_pos: *player_pos, player_name, player_level, has_access: true },
                    ) {
                        Ok(out) => {
                            let mut responses: Vec<Vec<u8>> = Vec::new();
                            for (msg_type, msg_text) in out.messages {
                                responses.push(pg::serialize_text_message(
                                    msg_type, &msg_text,
                                    None, None, None, None, None, None,
                                ));
                            }
                            for (say_type, say_text) in out.creature_says {
                                use forgottenserver_game::chat::SpeakType;
                                let speak = SpeakType::from_byte(say_type)
                                    .unwrap_or(SpeakType::Say);
                                responses.push(encode(&ServerPacket::Talk {
                                    speaker: player_name.to_string(),
                                    speaker_level: player_level,
                                    speak_type: speak,
                                    channel_id: None,
                                    pos: Some(*player_pos),
                                    text: say_text,
                                }));
                            }
                            for (effect_pos, effect_type) in out.magic_effects {
                                responses.push(pg::serialize_magic_effect(effect_pos.x, effect_pos.y, effect_pos.z, effect_type));
                            }
                            if responses.is_empty() {
                                DispatchResult::NoResponse
                            } else {
                                DispatchResult::MultiResponse(responses)
                            }
                        }
                        Err(e) => {
                            eprintln!("[gameloop] action script error: {e}");
                            DispatchResult::NoResponse
                        }
                    }
                } else {
                    DispatchResult::NoResponse
                }
            }
            Err(e) => {
                eprintln!("[gameloop] use item parse error: {e}");
                DispatchResult::NoResponse
            }
        },
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
        0x8E | 0xC9 | 0xE7 | 0xF3 => {
            eprintln!("[gameloop] acknowledged no-op opcode=0x{opcode:02x}");
            DispatchResult::NoResponse
        }
        0x96 => {
            match pg::parse_say_packet(&mut msg) {
                Ok(say) => {
                    eprintln!("[gameloop] say type={} text={:?}", say.say_type, say.text);

                    // Try registered talkactions first.
                    use forgottenserver_scripting::talkaction::TalkActionResult;
                    let mut talkaction_responses: Vec<Vec<u8>> = Vec::new();
                    let mut talkaction_matched = false;

                    let ta_result = talk_actions.on_say(&say.text, |action, param| {
                        talkaction_matched = true;
                        let script_path = script_dir.join(&action.script_name);
                        use forgottenserver_scripting::lua_bindings::talkaction_player::execute_talkaction;
                        eprintln!("[gameloop] talkaction: script={} param={:?}", script_path.display(), param);
                        match execute_talkaction(
                            &script_path,
                            &action.words,  // matched command word only (C++ passes words, not full text)
                            param,
                            *player_pos,
                            player_name,
                            player_level,
                            true,
                        ) {
                            Ok(out) => {
                                eprintln!("[gameloop] talkaction ok: {} messages, new_pos={:?}", out.messages.len(), out.new_pos);
                                for (msg_type, msg_text) in out.messages {
                                    eprintln!("[gameloop] talkaction message: type={msg_type} text={msg_text:?}");
                                    talkaction_responses.push(pg::serialize_text_message(
                                        msg_type,
                                        &msg_text,
                                        None,
                                        None,
                                        None,
                                        None,
                                        None,
                                        None,
                                    ));
                                }
                                if let Some(new_pos) = out.new_pos {
                                    *player_pos = new_pos;
                                    state.set_player_position(player_creature_id, new_pos);
                                    talkaction_responses.push(render_map(*player_dir, *player_pos));
                                }
                                for (effect_pos, effect_type) in out.magic_effects {
                                    talkaction_responses.push(pg::serialize_magic_effect(effect_pos.x, effect_pos.y, effect_pos.z, effect_type));
                                }
                                TalkActionResult::Break
                            }
                            Err(e) => {
                                eprintln!("[gameloop] talkaction script error: {e}");
                                TalkActionResult::Failed
                            }
                        }
                    });

                    if ta_result != TalkActionResult::Continue {
                        return if talkaction_responses.is_empty() {
                            DispatchResult::NoResponse
                        } else {
                            DispatchResult::MultiResponse(talkaction_responses)
                        };
                    }

                    // Hardcoded /pos fallback (not in talkactions.xml).
                    if say.text.starts_with("/pos") {
                        let text = format!(
                            "x={}, y={}, z={}",
                            player_pos.x, player_pos.y, player_pos.z
                        );
                        return DispatchResult::Response(pg::serialize_text_message(
                            pg::text_message_class::MESSAGE_EVENT_ADVANCE,
                            &text,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ));
                    }

                    // Regular chat broadcast.
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
                Err(e) => {
                    eprintln!("[gameloop] say parse error: {e}");
                    DispatchResult::NoResponse
                }
            }
        }
        0x97 => {
            eprintln!("[gameloop] get channels (0x97)");
            DispatchResult::Response(handle_get_channels(chat))
        }
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
        0x9E => {
            eprintln!("[gameloop] close npc channel (0x9E)");
            DispatchResult::NoResponse
        }
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
        0xA2 => match pg::parse_follow(&mut msg) {
            Ok(pkt) => {
                use forgottenserver_map::pathfinder::Pathfinder;
                eprintln!(
                    "[gameloop] follow creature_id=0x{:08x} (0xA2)",
                    pkt.creature_id
                );
                state.set_attack_target(player_creature_id, 0);
                if pkt.creature_id == 0 {
                    state.set_follow_target(player_creature_id, 0, vec![]);
                } else {
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
        0xA7 => {
            eprintln!("[gameloop] leave party (0xA7)");
            DispatchResult::NoResponse
        }
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
        0xAA => {
            eprintln!("[gameloop] create private channel (0xAA)");
            DispatchResult::NoResponse
        }
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
        0xBE => {
            eprintln!("[gameloop] cancel attack and follow (0xBE)");
            state.set_attack_target(player_creature_id, 0);
            state.set_follow_target(player_creature_id, 0, vec![]);
            handle_auto_walk(player_creature_id, vec![], state);
            DispatchResult::NoResponse
        }
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
        0xD2 => {
            eprintln!("[gameloop] request outfit (0xD2)");
            DispatchResult::NoResponse
        }
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
        0xF4 => {
            eprintln!("[gameloop] market leave (0xF4)");
            DispatchResult::NoResponse
        }
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
        _ => {
            eprintln!("[gameloop] unknown opcode: 0x{:02x}", opcode);
            DispatchResult::NoResponse
        }
    }
}

/// Advance a regen accumulator by `elapsed_secs` and return how much to
/// regenerate this tick together with the leftover accumulated seconds.
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

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_database::iologindata::PlayerLoginData;
    use forgottenserver_items::vocation::Vocations;
    use forgottenserver_scripting::actions::Actions;
    use forgottenserver_scripting::talkaction::TalkActions;
    use forgottenserver_world::map::Map;

    fn empty_vocations() -> Arc<Vocations> {
        Arc::new(Vocations::load_from_xml("<vocations/>").unwrap())
    }

    fn empty_map() -> Arc<Map> {
        Arc::new(Map::new())
    }

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
            direction: 2,
            premium_ends_at: 0,
        }
    }

    fn dispatch(
        opcode: u8,
        payload: &[u8],
        pos: &mut Position,
        dir: &mut u8,
        state: &mut GameState,
    ) -> DispatchResult {
        use forgottenserver_scripting::actions::Actions;
        use forgottenserver_scripting::talkaction::TalkActions;
        let world = World::new();
        let map = Map::new();
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let ta = TalkActions::new();
        let acts = Actions::new();
        dispatch_opcode(
            opcode,
            payload,
            &world,
            &map,
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
            &mut chat,
            &mut session,
            &ta,
            std::path::Path::new(""),
            &acts,
            std::path::Path::new(""),
        )
    }

    fn dispatch_with_map(
        opcode: u8,
        payload: &[u8],
        pos: &mut Position,
        dir: &mut u8,
        state: &mut GameState,
        map: &Map,
    ) -> DispatchResult {
        use forgottenserver_scripting::actions::Actions;
        use forgottenserver_scripting::talkaction::TalkActions;
        let world = World::new();
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let ta = TalkActions::new();
        let acts = Actions::new();
        dispatch_opcode(
            opcode,
            payload,
            &world,
            map,
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
            &mut chat,
            &mut session,
            &ta,
            std::path::Path::new(""),
            &acts,
            std::path::Path::new(""),
        )
    }

    fn dispatch_ch(
        opcode: u8,
        payload: &[u8],
        pos: &mut Position,
        dir: &mut u8,
        state: &mut GameState,
        chat: &mut ChatManager,
        session: &mut ChannelSession,
    ) -> DispatchResult {
        use forgottenserver_scripting::actions::Actions;
        use forgottenserver_scripting::talkaction::TalkActions;
        let world = World::new();
        let map = Map::new();
        let ta = TalkActions::new();
        let acts = Actions::new();
        dispatch_opcode(
            opcode,
            payload,
            &world,
            &map,
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
            &ta,
            std::path::Path::new(""),
            &acts,
            std::path::Path::new(""),
        )
    }

    #[test]
    fn game_loop_processes_encrypted_packet_and_exits_on_close() {
        use std::io::Write as _;

        let xtea_key: [u32; 4] = [0xDEAD_BEEF, 0x1234_5678, 0xABCD_EF01, 0x0102_0304];
        let payload = [0x65u8, 0x00];
        let frame = make_xtea_frame(&payload, xtea_key);

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
                Arc::new(TalkActions::new()),
                std::path::PathBuf::new(),
                Arc::new(Actions::new()),
                std::path::PathBuf::new(),
            );
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.write_all(&frame).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        server_thread
            .join()
            .expect("game loop thread panicked — run_game_loop must not panic on valid input");
    }

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
                Arc::new(TalkActions::new()),
                std::path::PathBuf::new(),
                Arc::new(Actions::new()),
                std::path::PathBuf::new(),
            );
        });

        let client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        drop(client);

        server_thread
            .join()
            .expect("game loop must exit cleanly on immediate EOF");
    }

    #[test]
    fn xtea_frame_round_trip_recovers_payload() {
        let xtea_key: [u32; 4] = [0xCAFE_BABE, 0xDEAD_BEEF, 0x1234_ABCD, 0x5678_EF01];
        let payload = [0x96u8, b'H', b'e', b'l', b'l', b'o'];

        let frame = make_xtea_frame(&payload, xtea_key);

        let outer_len = u16::from_le_bytes([frame[0], frame[1]]) as usize;
        assert_eq!(outer_len, frame.len() - 2);

        let body = &frame[2..];
        assert!(outer_len >= 12);

        let stored_adler = u32::from_le_bytes([body[0], body[1], body[2], body[3]]);
        let computed_adler = adler_checksum(&body[4..]);
        assert_eq!(stored_adler, computed_adler, "adler32 must match");

        let xtea_region_len = outer_len - 4;
        assert_eq!(xtea_region_len % 8, 0);

        let mut xtea_region: Vec<u8> = body[4..].to_vec();
        let key = xtea::Key(xtea_key);
        let round_keys = xtea::expand_key(&key);
        xtea::decrypt(&mut xtea_region, &round_keys);

        let inner_len = u16::from_le_bytes([xtea_region[0], xtea_region[1]]) as usize;
        assert_eq!(inner_len, payload.len());

        let recovered = &xtea_region[2..2 + inner_len];
        assert_eq!(recovered, &payload);
        assert_eq!(recovered[0], 0x96);
    }

    fn make_xtea_frame(payload: &[u8], xtea_key: [u32; 4]) -> Vec<u8> {
        let payload_len = payload.len();
        let xtea_content_len = 2 + payload_len;
        let xtea_region_len = if xtea_content_len.is_multiple_of(8) {
            xtea_content_len
        } else {
            xtea_content_len + (8 - xtea_content_len % 8)
        };

        let mut xtea_region = vec![0u8; xtea_region_len];
        let inner_len = payload_len as u16;
        xtea_region[0..2].copy_from_slice(&inner_len.to_le_bytes());
        xtea_region[2..2 + payload_len].copy_from_slice(payload);

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

    #[test]
    fn dispatch_get_channels_returns_channel_list_packet() {
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
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let channel_id: u16 = 3;
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
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let channel_id: u16 = 3;
        chat.subscribe(channel_id, 0);
        session.add_channel(channel_id);
        assert!(session.open_channels().contains(&channel_id));

        let payload = channel_id.to_le_bytes();
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch_ch(0x99, &payload, &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert!(!session.open_channels().contains(&channel_id));
    }

    #[test]
    fn dispatch_open_private_channel_returns_open_private_channel_packet() {
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

    #[test]
    fn dispatch_stop_auto_walk_clears_auto_walk_path() {
        let mut state = GameState::new();
        state.set_auto_walk(0, vec![1u8, 2u8, 3u8]);
        assert!(state.get_auto_walk(0).map(|p| !p.is_empty()).unwrap_or(false));

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0x69, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert!(state.get_auto_walk(0).map(|p| p.is_empty()).unwrap_or(true));
    }

    #[test]
    fn dispatch_walk_diagonal_ne_increments_x_decrements_y() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 2u8;
        let mut state = GameState::new();
        let r = dispatch(0x6A, &[], &mut pos, &mut dir, &mut state);
        match r {
            DispatchResult::Response(bytes) => assert_eq!(bytes[0], 0x64),
            _ => panic!("0x6A (walk NE) must return a Response"),
        }
        assert_eq!(pos, Position::new(101, 99, 7));
        assert_eq!(dir, 4);
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
        assert_eq!(pos, Position::new(101, 101, 7));
        assert_eq!(dir, 5);
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
        assert_eq!(pos, Position::new(99, 101, 7));
        assert_eq!(dir, 6);
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
        assert_eq!(pos, Position::new(99, 99, 7));
        assert_eq!(dir, 7);
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
                assert_eq!(bytes[0], 0x64);
            }
            _ => panic!("walk must produce a Response"),
        }
        assert_eq!(pos, Position::new(100, 99, 7));
        assert_eq!(dir, 0);
    }

    #[test]
    fn dispatch_walk_east_south_west_update_coords() {
        let mut state = GameState::new();
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let _ = dispatch(0x66, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(pos, Position::new(101, 100, 7));
        assert_eq!(dir, 1);
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let _ = dispatch(0x67, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(pos, Position::new(100, 101, 7));
        assert_eq!(dir, 2);
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
        let _ = dispatch(0x70, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(pos, Position::new(50, 60, 7));
        assert_eq!(dir, 1);
    }

    #[test]
    fn dispatch_say_pos_command_emits_coordinates_text_message() {
        let mut payload = vec![1u8];
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

        assert_eq!(bytes[0], 0xB4);
        assert_eq!(bytes[1], pg::text_message_class::MESSAGE_EVENT_ADVANCE);
        let resp_len = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
        let resp_text = std::str::from_utf8(&bytes[4..4 + resp_len]).unwrap();
        assert_eq!(resp_text, "x=42, y=7, z=8");
    }

    #[test]
    fn dispatch_say_emits_talk_packet_for_regular_text() {
        let mut payload = vec![1u8];
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
        assert_eq!(bytes[0], 0xAA);
        let packet_str = String::from_utf8_lossy(&bytes);
        assert!(packet_str.contains("hello world"));
    }

    #[test]
    fn dispatch_fight_modes_updates_state_and_returns_no_response() {
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
    fn dispatch_use_item_unregistered_returns_no_response() {
        // Item id 500 is not in the (empty) actions registry → NoResponse.
        let mut payload = Vec::new();
        payload.extend_from_slice(&100u16.to_le_bytes()); // pos_x
        payload.extend_from_slice(&100u16.to_le_bytes()); // pos_y
        payload.push(7u8);                                // pos_z
        payload.extend_from_slice(&500u16.to_le_bytes()); // item_id
        payload.push(0u8);                                // index

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x82, &payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_attack_stores_attack_target_in_state() {
        let creature_id: u32 = 0x1000_0001;
        let mut payload = Vec::new();
        payload.extend_from_slice(&creature_id.to_le_bytes());

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA1, &payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert_eq!(state.get_attack_target(0), Some(creature_id));
    }

    #[test]
    fn dispatch_cancel_attack_and_follow_clears_all_combat_state() {
        let mut state = GameState::new();
        state.set_attack_target(0, 0x1000_0001);
        state.set_follow_target(0, 0x1000_0002, vec![1u8, 2u8]);
        state.set_auto_walk(0, vec![3u8, 4u8]);

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xBE, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert_eq!(state.get_attack_target(0), None);
        assert!(state.get_follow_target(0).map(|(cid, _)| cid == 0).unwrap_or(true));
        assert!(state.get_auto_walk(0).map(|p| p.is_empty()).unwrap_or(true));
    }

    #[test]
    fn dispatch_follow_stores_follow_target_when_creature_known() {
        let creature_id: u32 = 42;
        let mut state = GameState::new();
        state.set_creature_position(creature_id, Position::new(102, 100, 7));

        let mut payload = Vec::new();
        payload.extend_from_slice(&creature_id.to_le_bytes());

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xA2, &payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        let follow = state.get_follow_target(0);
        assert!(follow.is_some());
        assert_eq!(follow.unwrap().0, creature_id);
    }

    #[test]
    fn dispatch_follow_always_clears_attack_target() {
        let creature_id: u32 = 42;
        let mut state = GameState::new();
        state.set_attack_target(0, 99);
        state.set_creature_position(creature_id, Position::new(102, 100, 7));

        let mut payload = Vec::new();
        payload.extend_from_slice(&creature_id.to_le_bytes());

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let _ = dispatch(0xA2, &payload, &mut pos, &mut dir, &mut state);
        assert_eq!(state.get_attack_target(0), None);
    }

    #[test]
    fn dispatch_follow_zero_clears_follow_target() {
        let mut state = GameState::new();
        state.set_follow_target(0, 42, vec![1u8, 2u8]);

        let mut payload = Vec::new();
        payload.extend_from_slice(&0u32.to_le_bytes());

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xA2, &payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        let follow = state.get_follow_target(0);
        assert!(follow.map(|(cid, _)| cid == 0).unwrap_or(true));
    }

    #[test]
    fn dispatch_auto_walk_stores_path_in_state() {
        let mut state = GameState::new();
        let payload = &[1u8, 3u8];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0x64, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        let path = state.get_auto_walk(0).expect("auto-walk path must be stored");
        assert_eq!(path, &vec![0u8]);
    }

    #[test]
    fn dispatch_auto_walk_reverses_wire_order() {
        let mut state = GameState::new();
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
        let payload = &[0u8];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0x64, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert!(state.get_auto_walk(0).map(|p| p.is_empty()).unwrap_or(true));
    }

    #[test]
    fn dispatch_rule_violation_report_returns_no_response() {
        let mut state = GameState::new();
        let mut payload = Vec::new();
        payload.push(2u8);
        payload.push(1u8);
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(b"Bot");
        payload.extend_from_slice(&8u16.to_le_bytes());
        payload.extend_from_slice(b"cheating");
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xF2, &payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_debug_assert_returns_no_response() {
        let mut state = GameState::new();
        let payload = &[0u8, 0, 0, 0, 0, 0, 0, 0];
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let r = dispatch(0xE8, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_acknowledged_noop_opcodes_return_no_response() {
        let mut state = GameState::new();
        for op in [0x8Eu8, 0xC9, 0xE7, 0xF3] {
            let mut pos = Position::new(100, 100, 7);
            let mut dir = 0u8;
            let r = dispatch(op, &[], &mut pos, &mut dir, &mut state);
            assert!(matches!(r, DispatchResult::NoResponse));
        }
    }

    #[test]
    fn dispatch_equip_object_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x77, &[0xD0, 0x07], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_equip_object_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x77, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_use_item_ex_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2, 101, 0, 100, 0, 7];
        let r = dispatch(0x83, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_use_with_creature_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2, 42, 0, 0, 0];
        let r = dispatch(0x84, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_rotate_item_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2];
        let r = dispatch(0x85, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_close_container_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x87, &[3], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_up_arrow_container_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x88, &[3], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_at_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2];
        let r = dispatch(0x8C, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_in_battle_list_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x8D, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_in_shop_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
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
        let r = dispatch(0x7A, &[100, 0, 0, 5, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_player_sale_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
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

    #[test]
    fn dispatch_throw_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2, 101, 0, 100, 0, 7, 1];
        let r = dispatch(0x78, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_throw_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x78, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

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
        let payload: &[u8] = &[5, 0, 65, 108, 105, 99, 101];
        let r = dispatch(0xAB, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_channel_exclude_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let payload: &[u8] = &[3, 0, 66, 111, 98];
        let r = dispatch(0xAC, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_update_container_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xCA, &[3], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_browse_field_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xCB, &[100, 0, 100, 0, 7], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_seek_in_container_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xCC, &[3, 5, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

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

    #[test]
    fn dispatch_add_vip_by_name_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
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
        assert!(state.get_vip_list(0).contains(&42));
        let r = dispatch(0xDD, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert!(!state.get_vip_list(0).contains(&42));
    }

    #[test]
    fn dispatch_remove_vip_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xDD, &[42, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

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
        let r = dispatch(0xF5, &[0xFE], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_create_offer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF6, &[0, 100, 0, 5, 0, 232, 3, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_cancel_offer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF7, &[232, 3, 0, 0, 0, 0, 0, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_accept_offer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF8, &[232, 3, 0, 0, 0, 0, 0, 0, 3, 0], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_modal_window_answer_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
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
            assert!(matches!(r, DispatchResult::NoResponse));
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
    // opcodes not yet exercised by existing tests
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_pong_0x1e_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x1E, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_edit_podium_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // pos_x(2), pos_y(2), pos_z(1), sprite_id(2), direction(1) = 8 bytes
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2];
        let r = dispatch(0x86, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_edit_podium_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x86, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_text_window_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // window_text_id(4) + text length(2) + text
        let mut payload = Vec::new();
        payload.extend_from_slice(&42u32.to_le_bytes()); // window_text_id
        let text = b"Hello";
        payload.extend_from_slice(&(text.len() as u16).to_le_bytes());
        payload.extend_from_slice(text);
        let r = dispatch(0x89, &payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_text_window_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x89, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_house_window_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // door_id(1) + window_id(4) = 5 bytes
        let payload: &[u8] = &[1u8, 42, 0, 0, 0];
        let r = dispatch(0x8A, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_house_window_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x8A, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_wrap_item_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // pos(5) + sprite_id(2) + stackpos(1) = 8 bytes
        let payload: &[u8] = &[100, 0, 100, 0, 7, 244, 1, 2];
        let r = dispatch(0x8B, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_wrap_item_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x8B, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_edit_vip_valid_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        // guid(4) + icon(1) + notify(1) = 6 bytes
        let payload: &[u8] = &[42, 0, 0, 0, 3, 1];
        let r = dispatch(0xDE, payload, &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_edit_vip_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xDE, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_turn_north_south_west_update_direction() {
        let mut state = GameState::new();

        // 0x6F = turn north (dir 0)
        let mut pos = Position::new(50, 60, 7);
        let mut dir = 2u8;
        let _ = dispatch(0x6F, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(dir, 0, "0x6F must turn north");
        assert_eq!(pos, Position::new(50, 60, 7), "turn must not change pos");

        // 0x71 = turn south (dir 2)
        let mut dir = 0u8;
        let _ = dispatch(0x71, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(dir, 2, "0x71 must turn south");

        // 0x72 = turn west (dir 3)
        let mut dir = 0u8;
        let _ = dispatch(0x72, &[], &mut pos, &mut dir, &mut state);
        assert_eq!(dir, 3, "0x72 must turn west");
    }

    // -----------------------------------------------------------------------
    // run_game_loop — frame validation paths (skip-continue branches)
    // These exercise the game loop's packet-validation logic by sending
    // deliberately malformed frames.
    // -----------------------------------------------------------------------

    #[test]
    fn game_loop_skips_frame_with_outer_len_less_than_12() {
        use std::io::Write as _;

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
                Arc::new(TalkActions::new()),
                std::path::PathBuf::new(),
                Arc::new(Actions::new()),
                std::path::PathBuf::new(),
            );
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        // outer_len = 8 (< 12) → skip-continue branch. Then close.
        let outer_len: u16 = 8;
        client.write_all(&outer_len.to_le_bytes()).unwrap();
        client.write_all(&[0u8; 8]).unwrap(); // 8 body bytes
        // Now close so the loop exits.
        client.shutdown(std::net::Shutdown::Write).unwrap();

        server_thread
            .join()
            .expect("game loop must not panic on outer_len < 12");
    }

    #[test]
    fn game_loop_skips_frame_with_xtea_region_not_multiple_of_8() {
        use std::io::Write as _;

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
                Arc::new(TalkActions::new()),
                std::path::PathBuf::new(),
                Arc::new(Actions::new()),
                std::path::PathBuf::new(),
            );
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        // outer_len = 13 (≥ 12) → xtea_region_len = 13 - 4 = 9, not multiple of 8 → skip.
        let outer_len: u16 = 13;
        client.write_all(&outer_len.to_le_bytes()).unwrap();
        client.write_all(&[0u8; 13]).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        server_thread
            .join()
            .expect("game loop must not panic on non-multiple-of-8 xtea region");
    }

    #[test]
    fn game_loop_skips_frame_with_adler_mismatch() {
        use std::io::Write as _;

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
                false, // sequence_checksum=false → adler validation runs
                2,
                test_player_data(),
                0,
                empty_vocations(),
                empty_map(),
                Arc::new(TalkActions::new()),
                std::path::PathBuf::new(),
                Arc::new(Actions::new()),
                std::path::PathBuf::new(),
            );
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        // outer_len = 12 → xtea_region_len = 8 (multiple of 8, valid).
        // But we set adler32 = 0xDEADBEEF (wrong) → adler mismatch → skip.
        let outer_len: u16 = 12;
        client.write_all(&outer_len.to_le_bytes()).unwrap();
        // body[0..4] = bad checksum, body[4..12] = 8 bytes of xtea region
        client.write_all(&0xDEAD_BEEFu32.to_le_bytes()).unwrap(); // bad adler
        client.write_all(&[0x01u8; 8]).unwrap(); // xtea region (8 bytes)
        client.shutdown(std::net::Shutdown::Write).unwrap();

        server_thread
            .join()
            .expect("game loop must not panic on adler mismatch");
    }

    #[test]
    fn game_loop_skips_frame_with_bad_inner_len() {
        use std::io::Write as _;

        let xtea_key: [u32; 4] = [0xCAFE_BABE, 0xDEAD_BEEF, 0x1234_ABCD, 0x5678_EF01];

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
                Arc::new(TalkActions::new()),
                std::path::PathBuf::new(),
                Arc::new(Actions::new()),
                std::path::PathBuf::new(),
            );
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();

        // Build an encrypted frame whose decrypted inner_len is 0 → bad inner_len.
        let mut xtea_region = [0u8; 8];
        // inner_len = 0 → bad
        xtea_region[0] = 0;
        xtea_region[1] = 0;

        let key = xtea::Key(xtea_key);
        let round_keys = xtea::expand_key(&key);
        xtea::encrypt(&mut xtea_region, &round_keys);

        let adler = adler_checksum(&xtea_region);
        let outer_len: u16 = (4 + 8) as u16; // 12
        client.write_all(&outer_len.to_le_bytes()).unwrap();
        client.write_all(&adler.to_le_bytes()).unwrap();
        client.write_all(&xtea_region).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        server_thread
            .join()
            .expect("game loop must not panic on bad inner_len");
    }

    #[test]
    fn game_loop_sequence_checksum_mode_uses_seq_frame() {
        use std::io::{Read as IoRead, Write as _};

        // sequence_checksum=true uses frame_packet_seq on the response.
        let xtea_key: [u32; 4] = [0xDEAD_BEEF, 0x1234_5678, 0xABCD_EF01, 0x0102_0304];

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
                true, // sequence_checksum = true
                2,
                test_player_data(),
                0,
                empty_vocations(),
                empty_map(),
                Arc::new(TalkActions::new()),
                std::path::PathBuf::new(),
                Arc::new(Actions::new()),
                std::path::PathBuf::new(),
            );
        });

        // Build a valid walk-north frame using the sequence_checksum=true format.
        // In sequence_checksum mode the server expects body[0..4] to be a sequence
        // number (not adler32), so we can skip checksum validation entirely.
        // The frame layout is the same structure; we just need a valid XTEA region.
        let payload = [0x65u8]; // walk north
        let xtea_content = [0x01u8, 0x00u8, 0x65u8]; // inner_len=1, opcode=0x65
        let mut xtea_region = [0u8; 8]; // round up to multiple of 8
        xtea_region[0..3].copy_from_slice(&xtea_content);
        let key = xtea::Key(xtea_key);
        let round_keys = xtea::expand_key(&key);
        xtea::encrypt(&mut xtea_region, &round_keys);

        // In sequence mode: body[0..4] = seq number (not validated); body[4..] = xtea.
        let outer_len: u16 = (4 + 8) as u16;
        let mut frame = Vec::new();
        frame.extend_from_slice(&outer_len.to_le_bytes());
        frame.extend_from_slice(&2u32.to_le_bytes()); // seq=2 (anything)
        frame.extend_from_slice(&xtea_region);

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        client.write_all(&frame).unwrap();
        // Read the response (map data) then close.
        let mut resp_hdr = [0u8; 2];
        let _ = client.read_exact(&mut resp_hdr);
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let _ = payload;
        server_thread
            .join()
            .expect("game loop must not panic in sequence_checksum mode");
    }

    // apply_regen_tick tests
    #[test]
    fn regen_tick_accumulates_and_returns_zero_before_threshold() {
        let (amount, remaining) = apply_regen_tick(4, 0, 6, 10);
        assert_eq!(amount, 0);
        assert_eq!(remaining, 4);
    }

    #[test]
    fn regen_tick_fires_once_at_exact_threshold() {
        let (amount, remaining) = apply_regen_tick(6, 0, 6, 10);
        assert_eq!(amount, 10);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn regen_tick_fires_multiple_times_with_overflow() {
        let (amount, remaining) = apply_regen_tick(14, 0, 6, 10);
        assert_eq!(amount, 20);
        assert_eq!(remaining, 2);
    }

    #[test]
    fn regen_tick_carries_forward_accumulated_secs() {
        let (amount, remaining) = apply_regen_tick(3, 4, 6, 10);
        assert_eq!(amount, 10);
        assert_eq!(remaining, 1);
    }

    #[test]
    fn regen_tick_zero_period_returns_no_regen() {
        let (amount, remaining) = apply_regen_tick(100, 50, 0, 10);
        assert_eq!(amount, 0);
        assert_eq!(remaining, 150);
    }

    // -----------------------------------------------------------------------
    // Err-branch (empty payload) tests for opcodes that parse a packet.
    // Each ensures the parse-failure path returns NoResponse without panicking.
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_player_purchase_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x7A, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_player_sale_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x7B, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_request_trade_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x7D, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_in_trade_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x7E, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_use_item_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x82, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_use_item_ex_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x83, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_use_with_creature_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x84, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_rotate_item_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x85, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_close_container_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x87, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_up_arrow_container_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x88, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_at_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x8C, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_look_in_battle_list_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x8D, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_say_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x96, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_fight_modes_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA0, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_attack_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA1, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_follow_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xA2, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_update_container_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xCA, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_browse_field_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xCB, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_seek_in_container_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xCC, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_set_outfit_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xD3, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_add_vip_by_name_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xDC, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_remove_vip_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xDD, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_rule_violation_report_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF2, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_browse_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF5, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_create_offer_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF6, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_cancel_offer_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF7, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_market_accept_offer_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF8, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_modal_window_answer_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xF9, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_debug_assert_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0xE8, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_open_private_channel_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let r = dispatch_ch(0x9A, &[], &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_open_channel_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let r = dispatch_ch(0x98, &[], &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    #[test]
    fn dispatch_close_channel_empty_payload_returns_no_response() {
        let mut pos = Position::new(100, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(0);
        let r = dispatch_ch(0x99, &[], &mut pos, &mut dir, &mut state, &mut chat, &mut session);
        assert!(matches!(r, DispatchResult::NoResponse));
    }

    // -----------------------------------------------------------------------
    // Walk out-of-bounds: coordinate would underflow/overflow u16.
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_walk_north_at_y_zero_returns_no_response() {
        // y=0, walk north: new_y = -1 → out of range → NoResponse (lines 449-450).
        let mut pos = Position::new(100, 0, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x65, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        // Position must not have changed.
        assert_eq!(pos, Position::new(100, 0, 7));
    }

    #[test]
    fn dispatch_walk_west_at_x_zero_returns_no_response() {
        // x=0, walk west: new_x = -1 → out of range → NoResponse.
        let mut pos = Position::new(0, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x68, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert_eq!(pos, Position::new(0, 100, 7));
    }

    #[test]
    fn dispatch_walk_east_at_x_max_returns_no_response() {
        // x=u16::MAX, walk east: new_x = u16::MAX+1 → overflow → NoResponse.
        let mut pos = Position::new(u16::MAX, 100, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x66, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert_eq!(pos, Position::new(u16::MAX, 100, 7));
    }

    #[test]
    fn dispatch_walk_south_at_y_max_returns_no_response() {
        // y=u16::MAX, walk south: new_y = u16::MAX+1 → overflow → NoResponse.
        let mut pos = Position::new(100, u16::MAX, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x67, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert_eq!(pos, Position::new(100, u16::MAX, 7));
    }

    #[test]
    fn dispatch_walk_nw_at_origin_returns_no_response() {
        // x=0,y=0 walk NW (0x6D): dx=-1,dy=-1 → both out of range → NoResponse.
        let mut pos = Position::new(0, 0, 7);
        let mut dir = 0u8;
        let mut state = GameState::new();
        let r = dispatch(0x6D, &[], &mut pos, &mut dir, &mut state);
        assert!(matches!(r, DispatchResult::NoResponse));
        assert_eq!(pos, Position::new(0, 0, 7));
    }

    // -----------------------------------------------------------------------
    // run_game_loop: outer_len == 0 path (lines 256-258)
    // -----------------------------------------------------------------------
    #[test]
    fn game_loop_exits_when_client_sends_zero_outer_len() {
        use std::io::Write as _;
        let xtea_key = [0x01u32, 0x02, 0x03, 0x04];
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server_thread = std::thread::spawn(move || {
            let (mut server_stream, _) = listener.accept().unwrap();
            server_stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            run_game_loop(&mut server_stream, xtea_key, false, 2, test_player_data(), 0, empty_vocations(), empty_map(), Arc::new(TalkActions::new()), std::path::PathBuf::new(), Arc::new(Actions::new()), std::path::PathBuf::new());
        });
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        // outer_len = 0 triggers break
        client.write_all(&[0x00, 0x00]).unwrap();
        drop(client);
        server_thread.join().expect("server must exit cleanly on outer_len=0");
    }

    // -----------------------------------------------------------------------
    // run_game_loop: read timeout path (lines 171-247)
    // The server issues pings every timeout; client holds connection open.
    // After MAX_CONSECUTIVE_TIMEOUTS+1 timeouts the loop exits.
    // -----------------------------------------------------------------------
    #[test]
    fn game_loop_exits_after_max_consecutive_timeouts() {
        use std::io::Read as _;
        let xtea_key = [0x01u32, 0x02, 0x03, 0x04];
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server_thread = std::thread::spawn(move || {
            let (mut server_stream, _) = listener.accept().unwrap();
            server_stream.set_read_timeout(Some(std::time::Duration::from_millis(5))).unwrap();
            run_game_loop(&mut server_stream, xtea_key, false, 2, test_player_data(), 0, empty_vocations(), empty_map(), Arc::new(TalkActions::new()), std::path::PathBuf::new(), Arc::new(Actions::new()), std::path::PathBuf::new());
        });
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.set_read_timeout(Some(std::time::Duration::from_millis(200))).unwrap();
        let drain_thread = std::thread::spawn(move || {
            let mut buf = [0u8; 256];
            loop {
                match client.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        });
        server_thread.join().expect("server must exit after max timeouts");
        let _ = drain_thread.join();
    }

    #[test]
    fn game_loop_sequence_checksum_timeout_sends_seq_ping() {
        use std::io::Read as _;
        let xtea_key = [0x05u32, 0x06, 0x07, 0x08];
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server_thread = std::thread::spawn(move || {
            let (mut server_stream, _) = listener.accept().unwrap();
            server_stream.set_read_timeout(Some(std::time::Duration::from_millis(5))).unwrap();
            // sequence_checksum=true — pings use frame_packet_seq (lines 220-221)
            run_game_loop(&mut server_stream, xtea_key, true, 4, test_player_data(), 0, empty_vocations(), empty_map(), Arc::new(TalkActions::new()), std::path::PathBuf::new(), Arc::new(Actions::new()), std::path::PathBuf::new());
        });
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.set_read_timeout(Some(std::time::Duration::from_millis(200))).unwrap();
        let drain_thread = std::thread::spawn(move || {
            let mut buf = [0u8; 256];
            loop {
                match client.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        });
        server_thread.join().expect("server must exit after max timeouts in seq mode");
        let _ = drain_thread.join();
    }

    // Cover run_game_loop DispatchResult::Break (lines 342-343): send 0x14 logout.
    #[test]
    fn game_loop_logout_packet_triggers_break() {
        use std::io::Write as _;
        let xtea_key: [u32; 4] = [0xAB, 0xCD, 0xEF, 0x12];
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server_thread = std::thread::spawn(move || {
            let (mut server_stream, _) = listener.accept().unwrap();
            server_stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            run_game_loop(&mut server_stream, xtea_key, false, 2, test_player_data(), 0, empty_vocations(), empty_map(), Arc::new(TalkActions::new()), std::path::PathBuf::new(), Arc::new(Actions::new()), std::path::PathBuf::new());
        });
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        let frame = make_xtea_frame(&[0x14], xtea_key); // logout opcode → DispatchResult::Break
        client.write_all(&frame).unwrap();
        drop(client);
        server_thread.join().expect("server must exit after logout packet");
    }

    // Cover run_game_loop DispatchResult::NoResponse (line 360): send 0x1E pong.
    // -----------------------------------------------------------------------
    // Stair / floor-change tests — mirrors C++ internalMoveCreature logic
    // -----------------------------------------------------------------------

    /// Build an Item with `has_height=true` (represents a stair/raised ground item).
    fn make_height_item() -> forgottenserver_items::item::Item {
        use forgottenserver_items::items_registry::ItemTypeData;
        let data = ItemTypeData {
            id: 1,
            has_height: true,
            ..Default::default()
        };
        forgottenserver_items::item::Item::new(std::sync::Arc::new(data), 1)
    }

    /// Build a plain ground item (no special flags).
    fn make_ground_item() -> forgottenserver_items::item::Item {
        use forgottenserver_items::items_registry::ItemTypeData;
        let data = ItemTypeData {
            id: 3,
            ..Default::default()
        };
        forgottenserver_items::item::Item::new(std::sync::Arc::new(data), 1)
    }

    /// Set a tile's ground + 2 stacked items all with `has_height=true` so that
    /// `tile.has_height(3)` returns `true` (C++ counts height items from ground +
    /// stacked items; requires count ≥ 3).
    fn set_high_tile(map: &mut Map, x: u16, y: u16, z: u8) {
        use forgottenserver_map::tile::Tile;
        let mut t = Tile::new(x, y, z);
        t.set_ground(make_height_item());
        t.add_item(make_height_item());
        t.add_item(make_height_item());
        map.set_tile(x, y, z, t);
    }

    /// Walk north on a tile with height ≥ 3 when the floor above the destination
    /// is solid ground → the player should move up one z level (z decreases).
    #[test]
    fn walk_north_on_high_tile_goes_up_one_floor() {
        use forgottenserver_map::tile::Tile;
        let mut map = Map::new();

        // Current tile (100,100,9) has height=3 (ground + 2 stacked height items).
        set_high_tile(&mut map, 100, 100, 9);

        // Above-current tile (100,100,8) has no ground — path is clear.
        // (not added to map → get_tile returns None → treated as clear by our logic)

        // Above-destination tile (100,99,8) has solid ground, no FLOORCHANGE, no IMMOVABLEBLOCKSOLID.
        let mut above_dest = Tile::new(100, 99, 8);
        above_dest.set_ground(make_ground_item());
        map.set_tile(100, 99, 8, above_dest);

        let mut pos = Position::new(100, 100, 9);
        let mut dir = 2u8;
        let mut state = GameState::new();
        dispatch_with_map(0x65, &[], &mut pos, &mut dir, &mut state, &map);

        assert_eq!(pos, Position::new(100, 99, 8), "player should move up one floor");
    }

    /// When the player is at z=8 (first underground floor), the up-check is
    /// disabled — mirrors C++ `currentPos.z != 8` guard.
    #[test]
    fn walk_up_blocked_when_current_z_is_8() {
        use forgottenserver_map::tile::Tile;
        let mut map = Map::new();

        set_high_tile(&mut map, 100, 100, 8);

        // Tile above dest (100,99,7) is accessible ground — but the check is skipped.
        let mut above_dest = Tile::new(100, 99, 7);
        above_dest.set_ground(make_ground_item());
        map.set_tile(100, 99, 7, above_dest);

        let mut pos = Position::new(100, 100, 8);
        let mut dir = 2u8;
        let mut state = GameState::new();
        dispatch_with_map(0x65, &[], &mut pos, &mut dir, &mut state, &map);

        assert_eq!(pos, Position::new(100, 99, 8), "z must NOT change when current_z==8");
    }

    /// Walking into an empty tile (no ground) when the floor below has height ≥ 3
    /// causes the player to drop one z level (z increases).
    #[test]
    fn walk_north_into_empty_tile_goes_down_one_floor() {
        let mut map = Map::new();

        // Current tile has ground (player can stand) but no special height.
        use forgottenserver_map::tile::Tile;
        let mut cur = Tile::new(100, 100, 6);
        cur.set_ground(make_ground_item());
        map.set_tile(100, 100, 6, cur);

        // Destination at same z (100,99,6) has no ground → empty tile.
        // Not added to map → None → treated as empty.

        // Below the destination (100,99,7) has height=3 (ground + 2 stacked).
        set_high_tile(&mut map, 100, 99, 7);

        let mut pos = Position::new(100, 100, 6);
        let mut dir = 2u8;
        let mut state = GameState::new();
        dispatch_with_map(0x65, &[], &mut pos, &mut dir, &mut state, &map);

        assert_eq!(pos, Position::new(100, 99, 7), "player should drop one floor");
    }

    /// Diagonal movement never triggers floor-change detection.
    #[test]
    fn diagonal_walk_never_changes_floor() {
        use forgottenserver_map::tile::Tile;
        let mut map = Map::new();

        // Same tile setup that would normally allow going up.
        set_high_tile(&mut map, 100, 100, 9);

        let mut above_dest = Tile::new(101, 99, 8);
        above_dest.set_ground(make_ground_item());
        map.set_tile(101, 99, 8, above_dest);

        let mut pos = Position::new(100, 100, 9);
        let mut dir = 2u8;
        let mut state = GameState::new();
        // 0x6A = NE (diagonal)
        dispatch_with_map(0x6A, &[], &mut pos, &mut dir, &mut state, &map);

        assert_eq!(pos.z, 9, "diagonal walk must not change floor");
    }

    /// When the tile above the destination has the FLOORCHANGE flag, going up is
    /// suppressed — mirrors C++ `!tmpTile->hasFlag(TILESTATE_FLOORCHANGE)`.
    #[test]
    fn walk_up_blocked_when_above_dest_has_floorchange_flag() {
        use forgottenserver_map::tile::{flags, Tile};
        let mut map = Map::new();

        set_high_tile(&mut map, 100, 100, 9);

        // Above-dest has ground but also the FLOORCHANGE flag.
        let mut above_dest = Tile::new(100, 99, 8);
        above_dest.set_ground(make_ground_item());
        above_dest.set_flag(flags::FLOORCHANGE_NORTH); // any FLOORCHANGE bit
        map.set_tile(100, 99, 8, above_dest);

        let mut pos = Position::new(100, 100, 9);
        let mut dir = 2u8;
        let mut state = GameState::new();
        dispatch_with_map(0x65, &[], &mut pos, &mut dir, &mut state, &map);

        assert_eq!(pos.z, 9, "FLOORCHANGE on above-dest must suppress the up-move");
    }

    /// When the current tile has no height (< 3), up-detection is skipped.
    #[test]
    fn walk_up_skipped_when_current_tile_has_no_height() {
        use forgottenserver_map::tile::Tile;
        let mut map = Map::new();

        // Current tile has ground but no has_height items.
        let mut cur = Tile::new(100, 100, 9);
        cur.set_ground(make_ground_item());
        map.set_tile(100, 100, 9, cur);

        let mut above_dest = Tile::new(100, 99, 8);
        above_dest.set_ground(make_ground_item());
        map.set_tile(100, 99, 8, above_dest);

        let mut pos = Position::new(100, 100, 9);
        let mut dir = 2u8;
        let mut state = GameState::new();
        dispatch_with_map(0x65, &[], &mut pos, &mut dir, &mut state, &map);

        assert_eq!(pos.z, 9, "no height on current tile means no floor change");
    }

    /// At z=7 (surface), the down-check is disabled — mirrors C++ `currentPos.z != 7`.
    #[test]
    fn walk_down_blocked_when_current_z_is_7() {
        let mut map = Map::new();

        // Destination at same z (100,99,7) has no ground → would normally trigger down.
        // Below dest (100,99,8) has height=3 — but z=7 disables the down-check.
        set_high_tile(&mut map, 100, 99, 8);

        let mut pos = Position::new(100, 100, 7);
        let mut dir = 2u8;
        let mut state = GameState::new();
        dispatch_with_map(0x65, &[], &mut pos, &mut dir, &mut state, &map);

        assert_eq!(pos.z, 7, "z must NOT change when current_z==7 (surface)");
    }

    #[test]
    fn game_loop_pong_packet_gets_no_response_and_exits_on_close() {
        use std::io::Write as _;
        let xtea_key: [u32; 4] = [0x11, 0x22, 0x33, 0x44];
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server_thread = std::thread::spawn(move || {
            let (mut server_stream, _) = listener.accept().unwrap();
            server_stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            run_game_loop(&mut server_stream, xtea_key, false, 2, test_player_data(), 0, empty_vocations(), empty_map(), Arc::new(TalkActions::new()), std::path::PathBuf::new(), Arc::new(Actions::new()), std::path::PathBuf::new());
        });
        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        let frame = make_xtea_frame(&[0x1E], xtea_key); // pong → DispatchResult::NoResponse
        client.write_all(&frame).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        server_thread.join().expect("server must exit after pong + close");
    }
}
