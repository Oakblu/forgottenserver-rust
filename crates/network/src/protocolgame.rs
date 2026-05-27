//! Game protocol packet parsing and serialization.
//!
//! Migrated from forgottenserver protocolgame.h / protocolgame.cpp.
//!
//! All functions are pure data transformations — no sockets, no I/O.

use forgottenserver_common::networkmessage::{
    ItemTypeMeta, NetworkMessage, PodiumMeta, INITIAL_BUFFER_POSITION,
};
use forgottenserver_common::outputmessage::OutputMessage;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Append the bytes produced by `NetworkMessage::add_item_payload` to an
/// `OutputMessage`.  Used by every send/serialize function that needs the
/// real C++ `addItem(id, count)` wire layout instead of a raw `u16` client id.
fn append_item_payload(out: &mut OutputMessage, count: u8, meta: ItemTypeMeta) {
    let mut nm = NetworkMessage::new();
    nm.add_item_payload(count, meta);
    let start = INITIAL_BUFFER_POSITION as usize;
    let end = start + nm.get_message_length() as usize;
    out.add_bytes(&nm.get_buffer()[start..end]);
}

/// Append the bytes produced by `NetworkMessage::add_item_instance` to an
/// `OutputMessage`.  Used by send/serialize functions that have a real item
/// instance to render with charges / duration / quiver / podium overrides.
#[allow(clippy::too_many_arguments)]
fn append_item_instance(
    out: &mut OutputMessage,
    count: u8,
    charges: u32,
    duration_seconds: u32,
    ammo_count: Option<u32>,
    podium: Option<PodiumMeta>,
    meta: ItemTypeMeta,
) {
    let mut nm = NetworkMessage::new();
    nm.add_item_instance(count, charges, duration_seconds, ammo_count, podium, meta);
    let start = INITIAL_BUFFER_POSITION as usize;
    let end = start + nm.get_message_length() as usize;
    out.add_bytes(&nm.get_buffer()[start..end]);
}

// ---------------------------------------------------------------------------
// Packet structs
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct LoginPacket {
    pub account_name: String,
    pub password: String,
    pub client_version: u16,
    pub xtea_key: [u32; 4],
}

#[derive(Debug, PartialEq, Clone)]
pub struct CharacterEntry {
    pub name: String,
    pub world_name: String,
    pub world_ip: u32,
    pub world_port: u16,
}

#[derive(Debug, PartialEq)]
pub struct WalkPacket {
    pub direction: u8,
}

#[derive(Debug, PartialEq)]
pub struct SayPacket {
    pub say_type: u8,
    pub text: String,
}

#[derive(Debug, PartialEq)]
pub struct UseItemPacket {
    pub pos_x: u16,
    pub pos_y: u16,
    pub pos_z: u8,
    pub item_id: u16,
    pub index: u8,
}

#[derive(Debug, PartialEq)]
pub struct LookAtPacket {
    pub pos_x: u16,
    pub pos_y: u16,
    pub pos_z: u8,
    pub item_id: u16,
    pub stack_pos: u8,
}

#[derive(Debug, PartialEq)]
pub struct TradePacket {
    pub pos_x: u16,
    pub pos_y: u16,
    pub pos_z: u8,
    pub item_id: u16,
    pub count: u8,
}

#[derive(Debug, PartialEq)]
pub struct VipPacket {
    pub player_id: u32,
}

// `CreatureMovePacket` / `AddCreaturePacket` (Tier 3 stubs) were removed
// in Session 4 when `serialize_creature_move` / `serialize_add_creature`
// were promoted to take `&mut OutputMessage` + plain-data args so they
// could match the real C++ wire layout.

// ---------------------------------------------------------------------------
// Parse functions
// ---------------------------------------------------------------------------

/// Parse a login packet from an inbound `NetworkMessage`.
///
/// Wire format:
/// - `client_version` (u16)
/// - `xtea_key`       (4 × u32)
/// - `account_name`   (length-prefixed string)
/// - `password`       (length-prefixed string)
pub fn parse_login_packet(msg: &mut NetworkMessage) -> Result<LoginPacket, String> {
    let client_version = msg.get_u16();
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

/// Serialize a list of character entries into raw bytes.
///
/// Wire format per entry:
/// - `name`       (length-prefixed string)
/// - `world_name` (length-prefixed string)
/// - `world_ip`   (u32)
/// - `world_port` (u16)
///
/// Preceded by a u8 character count.
pub fn serialize_character_list(chars: &[CharacterEntry]) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(chars.len() as u8);
    for c in chars {
        out.add_string(&c.name);
        out.add_string(&c.world_name);
        out.add_u32(c.world_ip);
        out.add_u16(c.world_port);
    }
    out.write_message_length();
    // Return only the payload (skip the 2-byte OutputMessage header)
    out.get_output_buffer()[2..].to_vec()
}

/// Parse a walk packet (single direction byte).
pub fn parse_walk_packet(msg: &mut NetworkMessage) -> Result<WalkPacket, String> {
    let direction = msg.get_u8();
    if msg.is_overrun() {
        return Err("walk packet overrun".into());
    }
    Ok(WalkPacket { direction })
}

/// Parse a say packet.
///
/// Wire format:
/// - `say_type` (u8)
/// - `text`     (length-prefixed string)
pub fn parse_say_packet(msg: &mut NetworkMessage) -> Result<SayPacket, String> {
    let say_type = msg.get_u8();
    let text = msg.get_string(0);
    if msg.is_overrun() {
        return Err("say packet overrun".into());
    }
    Ok(SayPacket { say_type, text })
}

/// Parse a use-item packet.
///
/// Wire format:
/// - `pos_x`   (u16)
/// - `pos_y`   (u16)
/// - `pos_z`   (u8)
/// - `item_id` (u16)
/// - `index`   (u8)
pub fn parse_use_item_packet(msg: &mut NetworkMessage) -> Result<UseItemPacket, String> {
    let pos_x = msg.get_u16();
    let pos_y = msg.get_u16();
    let pos_z = msg.get_u8();
    let item_id = msg.get_u16();
    let index = msg.get_u8();
    if msg.is_overrun() {
        return Err("use item packet overrun".into());
    }
    Ok(UseItemPacket {
        pos_x,
        pos_y,
        pos_z,
        item_id,
        index,
    })
}

/// Parse a look-at packet.
///
/// Wire format:
/// - `pos_x`     (u16)
/// - `pos_y`     (u16)
/// - `pos_z`     (u8)
/// - `item_id`   (u16)
/// - `stack_pos` (u8)
pub fn parse_look_at_packet(msg: &mut NetworkMessage) -> Result<LookAtPacket, String> {
    let pos_x = msg.get_u16();
    let pos_y = msg.get_u16();
    let pos_z = msg.get_u8();
    let item_id = msg.get_u16();
    let stack_pos = msg.get_u8();
    if msg.is_overrun() {
        return Err("look at packet overrun".into());
    }
    Ok(LookAtPacket {
        pos_x,
        pos_y,
        pos_z,
        item_id,
        stack_pos,
    })
}

/// Parse a trade packet.
///
/// Wire format:
/// - `pos_x`   (u16)
/// - `pos_y`   (u16)
/// - `pos_z`   (u8)
/// - `item_id` (u16)
/// - `count`   (u8)
pub fn parse_trade_packet(msg: &mut NetworkMessage) -> Result<TradePacket, String> {
    let pos_x = msg.get_u16();
    let pos_y = msg.get_u16();
    let pos_z = msg.get_u8();
    let item_id = msg.get_u16();
    let count = msg.get_u8();
    if msg.is_overrun() {
        return Err("trade packet overrun".into());
    }
    Ok(TradePacket {
        pos_x,
        pos_y,
        pos_z,
        item_id,
        count,
    })
}

/// Parse a VIP (friends list) packet.
///
/// Wire format:
/// - `player_id` (u32)
pub fn parse_vip_packet(msg: &mut NetworkMessage) -> Result<VipPacket, String> {
    let player_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("vip packet overrun".into());
    }
    Ok(VipPacket { player_id })
}

// ---------------------------------------------------------------------------
// Serialize functions
// ---------------------------------------------------------------------------

/// Serialize the inter-tile move portion of `sendMoveCreature`
/// (opcode `0x6D`).
///
/// Mirrors the per-creature wire layout produced by the same-floor branch
/// of `ProtocolGame::sendMoveCreature` in
/// `forgottenserver/src/protocolgame.cpp` lines 2788–2858 (specifically
/// the `msg.addByte(0x6D); ... msg.addPosition(newPos);` block at lines
/// 2800–2809 and again at 2843–2853).
///
/// The full C++ method has 4 branches (own-player teleport, own-player
/// scroll, other-creature visible, other-creature crossing border).  This
/// port serialises only the common `0x6D` "move" payload that every
/// non-teleport branch writes; map-description scroll bytes (opcodes
/// `0x65`/`0x66`/`0x67`/`0x68`) and the floor-change branches are
/// caller-side concerns (Cylinder/Tile crate dependencies).
///
/// Wire layout (matches C++ exactly for the non-teleport on-floor branch):
/// * opcode `0x6D` (u8)
/// * if `old_stack_pos < MAX_STACKPOS` (10):
///   `old_pos` (5 bytes: u16 x, u16 y, u8 z) + `old_stack_pos` (u8)
/// * else (stackpos overflow path):
///   `0xFFFF` (u16) + `creature_id` (u32)
/// * `new_pos` (5 bytes: u16 x, u16 y, u8 z)
#[allow(clippy::too_many_arguments)]
pub fn serialize_creature_move(
    out: &mut OutputMessage,
    creature_id: u32,
    old_pos_x: u16,
    old_pos_y: u16,
    old_pos_z: u8,
    old_stack_pos: u32,
    new_pos_x: u16,
    new_pos_y: u16,
    new_pos_z: u8,
) {
    out.add_u8(0x6D);
    if old_stack_pos < MAX_STACKPOS {
        out.add_u16(old_pos_x);
        out.add_u16(old_pos_y);
        out.add_u8(old_pos_z);
        out.add_u8(old_stack_pos as u8);
    } else {
        out.add_u16(0xFFFF);
        out.add_u32(creature_id);
    }
    out.add_u16(new_pos_x);
    out.add_u16(new_pos_y);
    out.add_u8(new_pos_z);
}

/// Flat C++ `Outfit_t` descriptor used by `serialize_creature_outfit` and
/// `serialize_add_creature`.
///
/// Mirrors the on-wire ordering of `Outfit_t` from
/// `forgottenserver/src/creature.h`: `lookType`, `lookHead`, `lookBody`,
/// `lookLegs`, `lookFeet`, `lookAddons`, `lookMount`, `lookMountHead`,
/// `lookMountBody`, `lookMountLegs`, `lookMountFeet`.
///
/// The `look_type_ex` field is the C++ `lookTypeEx` fallback used when
/// `look_type == 0` (renders a regular item id as the creature outfit).
#[derive(Debug, Clone, Copy, Default)]
pub struct OutfitDescriptor {
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
    pub look_type_ex: u16,
    pub look_mount: u16,
    pub look_mount_head: u8,
    pub look_mount_body: u8,
    pub look_mount_legs: u8,
    pub look_mount_feet: u8,
}

/// Append an `Outfit_t` block to `out`.
///
/// Mirrors C++ `ProtocolGame::AddOutfit` in
/// `forgottenserver/src/protocolgame.cpp` lines 3579–3601.
///
/// Wire layout:
/// * `look_type` (u16)
/// * if `look_type != 0`:
///   `look_head`, `look_body`, `look_legs`, `look_feet`, `look_addons`
///   (5 × u8)
/// * else:
///   `addItemId(look_type_ex)` — a u16 item id
/// * `look_mount` (u16)
/// * if `look_mount != 0`:
///   `look_mount_head`, `look_mount_body`, `look_mount_legs`,
///   `look_mount_feet` (4 × u8)
fn append_outfit(out: &mut OutputMessage, outfit: OutfitDescriptor) {
    out.add_u16(outfit.look_type);
    if outfit.look_type != 0 {
        out.add_u8(outfit.look_head);
        out.add_u8(outfit.look_body);
        out.add_u8(outfit.look_legs);
        out.add_u8(outfit.look_feet);
        out.add_u8(outfit.look_addons);
    } else {
        // C++ msg.addItemId(lookTypeEx) — writes a u16 LE.
        out.add_u16(outfit.look_type_ex);
    }

    out.add_u16(outfit.look_mount);
    if outfit.look_mount != 0 {
        out.add_u8(outfit.look_mount_head);
        out.add_u8(outfit.look_mount_body);
        out.add_u8(outfit.look_mount_legs);
        out.add_u8(outfit.look_mount_feet);
    }
}

/// Compact `Creature` data needed by `serialize_add_creature` /
/// `AddCreature`.  Keeps the network crate dependency-free of `Creature`,
/// `Player`, etc.
///
/// Fields match the per-creature reads performed by C++ `AddCreature` at
/// `forgottenserver/src/protocolgame.cpp` lines 3388–3475.  Boolean
/// callouts mirror C++ predicates that the caller resolves against the
/// real `Creature` / `Player` instance.
#[derive(Debug, Clone, Copy)]
pub struct AddCreatureMeta {
    /// `Creature::getID()`.
    pub creature_id: u32,
    /// `Creature::getType()` after monster→summon_own promotion.
    pub creature_type: u8,
    /// Set if `creature_type == CREATURETYPE_SUMMON_OWN`; otherwise `0`.
    pub master_id: u32,
    /// `Creature::isHealthHidden()`.
    pub health_hidden: bool,
    /// `health / max(maxHealth, 1) * 100` (ceil), only used when not hidden.
    pub health_percent: u8,
    /// `Creature::getDirection()`.
    pub direction: u8,
    /// True if `Creature::isInGhostMode() || isInvisible()`.
    pub ghost_or_invisible: bool,
    /// `Creature::getCurrentOutfit()` (or default Outfit_t() when hidden).
    pub outfit: OutfitDescriptor,
    /// `LightInfo::level` (or `0xFF` when viewing player is access).
    pub light_level: u8,
    /// `LightInfo::color`.
    pub light_color: u8,
    /// `Creature::getStepSpeed() / 2`.
    pub step_speed_half: u16,
    /// `player->getSkullClient(creature)`.
    pub skull: u8,
    /// `player->getPartyShield(creature->getPlayer())`.
    pub party_shield: u8,
    /// `player->getGuildEmblem(creature->getPlayer())` (only written when
    /// `!known`).
    pub guild_emblem: u8,
    /// `otherPlayer->getVocation()->getClientId()` when `creature_type ==
    /// CREATURETYPE_PLAYER`; ignored otherwise.
    pub player_vocation_client_id: u8,
    /// `npc->getSpeechBubble()` or `SPEECHBUBBLE_NONE = 0`.
    pub speech_bubble: u8,
    /// True if `player->canWalkthroughEx(creature)` (writes `0x00`); else
    /// `0x01`.
    pub can_walkthrough: bool,
}

/// Serialize the `AddCreature` block written by `sendAddCreature` (opcode
/// `0x6A`) and `AddCreature` helper.
///
/// Mirrors C++ `ProtocolGame::sendAddCreature` +
/// `ProtocolGame::AddCreature` in
/// `forgottenserver/src/protocolgame.cpp` lines 2749–2786 and 3388–3475.
///
/// The C++ `sendAddCreature` early-returns on `stackpos >= MAX_STACKPOS`
/// (it instead refreshes the whole tile).  This port serialises only the
/// in-band-stackpos branch; callers must perform the stackpos overflow
/// check themselves.
///
/// Wire layout:
/// * opcode `0x6A` (u8)
/// * `pos` (5 bytes) + `stackpos` (u8)
/// * if `known`: `0x62` (u16) + `creature_id` (u32)
/// * else: `0x61` (u16) + `removed_known` (u32) + `creature_id` (u32)
///   + `creature_type` (u8 — `CREATURETYPE_HIDDEN=12` if health-hidden,
///     else real type)
///   + if SUMMON_OWN: `master_id` (u32)
///   + creature name (length-prefixed string; empty when health-hidden)
/// * `health_percent` (u8 — `0` if health-hidden)
/// * `direction` (u8)
/// * outfit block (see `append_outfit`)
/// * `light_level` (u8) + `light_color` (u8)
/// * `step_speed_half` (u16)
/// * creature-icons block: `0x00` count (no monster icons in this port)
/// * `skull` (u8) + `party_shield` (u8)
/// * if `!known`: `guild_emblem` (u8)
/// * `creature_type` again (u8 — `CREATURETYPE_HIDDEN=12` if hidden)
/// * if SUMMON_OWN: `master_id` (u32) again
/// * if `creature_type == CREATURETYPE_PLAYER`: `player_vocation_client_id`
///   (u8)
/// * `speech_bubble` (u8)
/// * `0xFF` (MARK_UNMARKED) + `0x00` (inspection type)
/// * `walk-through` (u8 — `0x00` if can walk through, else `0x01`)
#[allow(clippy::too_many_arguments)]
pub fn serialize_add_creature(
    out: &mut OutputMessage,
    pos_x: u16,
    pos_y: u16,
    pos_z: u8,
    stackpos: u8,
    known: bool,
    removed_known: u32,
    name: &str,
    meta: AddCreatureMeta,
) {
    out.add_u8(0x6A);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_u8(stackpos);

    let creature_type_byte = if meta.health_hidden {
        CREATURETYPE_HIDDEN
    } else {
        meta.creature_type
    };

    if known {
        out.add_u16(0x62);
        out.add_u32(meta.creature_id);
    } else {
        out.add_u16(0x61);
        out.add_u32(removed_known);
        out.add_u32(meta.creature_id);
        out.add_u8(creature_type_byte);

        if meta.creature_type == CREATURETYPE_SUMMON_OWN {
            out.add_u32(meta.master_id);
        }

        if meta.health_hidden {
            out.add_string("");
        } else {
            out.add_string(name);
        }
    }

    if meta.health_hidden {
        out.add_u8(0x00);
    } else {
        out.add_u8(meta.health_percent);
    }

    out.add_u8(meta.direction);

    if meta.ghost_or_invisible {
        // C++ uses a static default-constructed Outfit_t (all zero).
        append_outfit(out, OutfitDescriptor::default());
    } else {
        append_outfit(out, meta.outfit);
    }

    out.add_u8(meta.light_level);
    out.add_u8(meta.light_color);

    out.add_u16(meta.step_speed_half);

    // AddCreatureIcons: non-monster path — empty icons.
    out.add_u8(0x00);

    out.add_u8(meta.skull);
    out.add_u8(meta.party_shield);

    if !known {
        out.add_u8(meta.guild_emblem);
    }

    out.add_u8(creature_type_byte);
    if meta.creature_type == CREATURETYPE_SUMMON_OWN {
        out.add_u32(meta.master_id);
    }

    if meta.creature_type == CREATURETYPE_PLAYER {
        out.add_u8(meta.player_vocation_client_id);
    }

    out.add_u8(meta.speech_bubble);

    out.add_u8(0xFF); // MARK_UNMARKED
    out.add_u8(0x00); // inspection type

    out.add_u8(if meta.can_walkthrough { 0x00 } else { 0x01 });
}

// MAX_STACKPOS / CreatureType_t literals (mirrors C++ game.h:61 and const.h).
const MAX_STACKPOS: u32 = 10;
const CREATURETYPE_PLAYER: u8 = 0;
const CREATURETYPE_SUMMON_OWN: u8 = 3;
const CREATURETYPE_HIDDEN: u8 = 12;

/// Serialize `sendCreatureOutfit` (opcode `0x8E`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureOutfit` in
/// `forgottenserver/src/protocolgame.cpp` lines 1593–1604.
///
/// Wire layout:
/// * opcode `0x8E` (u8)
/// * `creature_id` (u32)
/// * outfit block (see `append_outfit`)
pub fn serialize_creature_outfit(
    out: &mut OutputMessage,
    creature_id: u32,
    outfit: OutfitDescriptor,
) {
    out.add_u8(0x8E);
    out.add_u32(creature_id);
    append_outfit(out, outfit);
}

/// Serialize `sendCreatureShield` (opcode `0x91`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureShield` in
/// `forgottenserver/src/protocolgame.cpp` lines 1635–1646.
///
/// Wire layout:
/// * opcode `0x91` (u8)
/// * `creature_id` (u32)
/// * `party_shield` (u8 — `player->getPartyShield(creature->getPlayer())`)
pub fn serialize_creature_shield(out: &mut OutputMessage, creature_id: u32, party_shield: u8) {
    out.add_u8(0x91);
    out.add_u32(creature_id);
    out.add_u8(party_shield);
}

/// Serialize `sendCreatureTurn` (opcode `0x6B`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureTurn` in
/// `forgottenserver/src/protocolgame.cpp` lines 2399–2419.
///
/// Wire layout:
/// * opcode `0x6B` (u8)
/// * if `stackpos < MAX_STACKPOS` (10):
///   `pos` (5 bytes) + `stackpos` (u8)
/// * else:
///   `0xFFFF` (u16) + `creature_id` (u32)
/// * `0x63` (u16) — sub-opcode "creature turn"
/// * `creature_id` (u32)
/// * `direction` (u8)
/// * `walk-through` (u8 — `0x00` if `player->canWalkthroughEx(creature)`,
///   else `0x01`)
#[allow(clippy::too_many_arguments)]
pub fn serialize_creature_turn(
    out: &mut OutputMessage,
    creature_id: u32,
    pos_x: u16,
    pos_y: u16,
    pos_z: u8,
    stackpos: u32,
    direction: u8,
    can_walkthrough: bool,
) {
    out.add_u8(0x6B);
    if stackpos < MAX_STACKPOS {
        out.add_u16(pos_x);
        out.add_u16(pos_y);
        out.add_u8(pos_z);
        out.add_u8(stackpos as u8);
    } else {
        out.add_u16(0xFFFF);
        out.add_u32(creature_id);
    }

    out.add_u16(0x63);
    out.add_u32(creature_id);
    out.add_u8(direction);
    out.add_u8(if can_walkthrough { 0x00 } else { 0x01 });
}

/// Serialize `sendCreatureSay` (opcode `0xAA`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureSay` in
/// `forgottenserver/src/protocolgame.cpp` lines 2422–2450.
///
/// The C++ `statementId` is a static counter incremented per call; the
/// caller passes the already-incremented value here for determinism.
///
/// Wire layout:
/// * opcode `0xAA` (u8)
/// * `statement_id` (u32)
/// * `creature_name` (length-prefixed string)
/// * `0x00` (u8) — "(Traded)" suffix flag, always 0
/// * `level` (u16) — player level, or `0` for non-player creatures
/// * `speak_class` (u8) — `SpeakClasses` enum value (e.g. `TALKTYPE_SAY=1`)
/// * `pos` (5 bytes) — explicit `pos` if provided, else creature's
///   position; **caller passes the chosen position** as `(pos_x, pos_y,
///   pos_z)`.
/// * `text` (length-prefixed string)
#[allow(clippy::too_many_arguments)]
pub fn serialize_creature_say(
    out: &mut OutputMessage,
    statement_id: u32,
    creature_name: &str,
    level: u16,
    speak_class: u8,
    pos_x: u16,
    pos_y: u16,
    pos_z: u8,
    text: &str,
) {
    out.add_u8(0xAA);
    out.add_u32(statement_id);
    out.add_string(creature_name);
    out.add_u8(0x00); // "(Traded)" suffix flag
    out.add_u16(level);
    out.add_u8(speak_class);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_string(text);
}

/// One creature-icon row written by `serialize_update_creature_icons` /
/// `AddCreatureIcons`: `(icon_id, icon_type, level)`.
///
/// `icon_type` mirrors C++: `1` for monster special icons, `0` for
/// regular creature icons.
pub type CreatureIconRow = (u8, u8, u16);

/// Serialize `sendUpdateCreatureIcons` (opcode `0x8B` + sub-op `14`).
///
/// Mirrors C++ `ProtocolGame::sendUpdateCreatureIcons` +
/// `AddCreatureIcons` in `forgottenserver/src/protocolgame.cpp` lines
/// 2709–2722 and 3477–3497.
///
/// The C++ `AddCreatureIcons` interleaves a monster-specific count and a
/// regular-icon count.  This port collapses the two sets into a single
/// `icons` slice whose total length is written as the count byte: callers
/// pass monster icons (type `1`) followed by regular icons (type `0`) to
/// match the C++ ordering.
///
/// Wire layout:
/// * opcode `0x8B` (u8)
/// * `creature_id` (u32)
/// * `14` (u8) — sub-op "event player icons"
/// * `count` (u8) = `icons.len()`
/// * per icon: `icon_id` (u8) + `icon_type` (u8) + `level` (u16)
pub fn serialize_update_creature_icons(
    out: &mut OutputMessage,
    creature_id: u32,
    icons: &[CreatureIconRow],
) {
    out.add_u8(0x8B);
    out.add_u32(creature_id);
    out.add_u8(14);

    out.add_u8(icons.len() as u8);
    for &(icon_id, icon_type, level) in icons {
        out.add_u8(icon_id);
        out.add_u8(icon_type);
        out.add_u16(level);
    }
}

// ---------------------------------------------------------------------------
// Additional packet structs
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct FightModesPacket {
    pub fight_mode: u8,
    pub chase_mode: u8,
    pub secure_mode: u8,
}

#[derive(Debug, PartialEq)]
pub struct AttackPacket {
    pub creature_id: u32,
}

#[derive(Debug, PartialEq)]
pub struct FollowPacket {
    pub creature_id: u32,
}

#[derive(Debug, PartialEq)]
pub struct CloseContainerPacket {
    pub container_id: u8,
}

#[derive(Debug, PartialEq)]
pub struct UpArrowContainerPacket {
    pub container_id: u8,
}

#[derive(Debug, PartialEq)]
pub struct ThrowPacket {
    pub from_x: u16,
    pub from_y: u16,
    pub from_z: u8,
    pub sprite_id: u16,
    pub from_stackpos: u8,
    pub to_x: u16,
    pub to_y: u16,
    pub to_z: u8,
    pub count: u8,
}

#[derive(Debug, PartialEq)]
pub struct LookInBattleListPacket {
    pub creature_id: u32,
}

#[derive(Debug, PartialEq)]
pub struct InviteToPartyPacket {
    pub target_id: u32,
}

#[derive(Debug, PartialEq)]
pub struct JoinPartyPacket {
    pub target_id: u32,
}

#[derive(Debug, PartialEq)]
pub struct RevokePartyInvitePacket {
    pub target_id: u32,
}

#[derive(Debug, PartialEq)]
pub struct PassPartyLeadershipPacket {
    pub target_id: u32,
}

#[derive(Debug, PartialEq)]
pub struct EnableSharedPartyExperiencePacket {
    pub active: bool,
}

#[derive(Debug, PartialEq)]
pub struct ModalWindowAnswerPacket {
    pub window_id: u32,
    pub button: u8,
    pub choice: u8,
}

#[derive(Debug, PartialEq)]
pub struct BrowseFieldPacket {
    pub pos_x: u16,
    pub pos_y: u16,
    pub pos_z: u8,
}

#[derive(Debug, PartialEq)]
pub struct SeekInContainerPacket {
    pub container_id: u8,
    pub index: u16,
}

#[derive(Debug, PartialEq)]
pub struct MarketBrowsePacket {
    pub browse_id: u8,
    pub sprite_id: u16, // only valid when browse_id is neither own-offers nor own-history
}

#[derive(Debug, PartialEq)]
pub struct MarketCancelOfferPacket {
    pub timestamp: u32,
    pub counter: u16,
}

#[derive(Debug, PartialEq)]
pub struct MarketAcceptOfferPacket {
    pub timestamp: u32,
    pub counter: u16,
    pub amount: u16,
}

#[derive(Debug, PartialEq)]
pub struct AddVipByNamePacket {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct RemoveVipPacket {
    pub guid: u32,
}

#[derive(Debug, PartialEq)]
pub struct RotateItemPacket {
    pub pos_x: u16,
    pub pos_y: u16,
    pub pos_z: u8,
    pub sprite_id: u16,
    pub stackpos: u8,
}

#[derive(Debug, PartialEq)]
pub struct EquipObjectPacket {
    pub sprite_id: u16,
}

#[derive(Debug, PartialEq)]
pub struct TextWindowPacket {
    pub window_text_id: u32,
    pub text: String,
}

#[derive(Debug, PartialEq)]
pub struct HouseWindowPacket {
    pub door_id: u8,
    pub window_id: u32,
    pub text: String,
}

#[derive(Debug, PartialEq)]
pub struct OpenChannelPacket {
    pub channel_id: u16,
}

#[derive(Debug, PartialEq)]
pub struct CloseChannelPacket {
    pub channel_id: u16,
}

#[derive(Debug, PartialEq)]
pub struct ChannelInvitePacket {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ChannelExcludePacket {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct OpenPrivateChannelPacket {
    pub receiver: String,
}

// ---------------------------------------------------------------------------
// Additional parse functions
// ---------------------------------------------------------------------------

/// Parse fight modes packet.
///
/// Wire format: fight_mode (u8), chase_mode (u8), secure_mode (u8)
pub fn parse_fight_modes(msg: &mut NetworkMessage) -> Result<FightModesPacket, String> {
    let fight_mode = msg.get_u8();
    let chase_mode = msg.get_u8();
    let secure_mode = msg.get_u8();
    if msg.is_overrun() {
        return Err("fight modes packet overrun".into());
    }
    Ok(FightModesPacket {
        fight_mode,
        chase_mode,
        secure_mode,
    })
}

/// Parse an attack packet.
///
/// Wire format: creature_id (u32)
pub fn parse_attack(msg: &mut NetworkMessage) -> Result<AttackPacket, String> {
    let creature_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("attack packet overrun".into());
    }
    Ok(AttackPacket { creature_id })
}

/// Parse a follow packet.
///
/// Wire format: creature_id (u32)
pub fn parse_follow(msg: &mut NetworkMessage) -> Result<FollowPacket, String> {
    let creature_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("follow packet overrun".into());
    }
    Ok(FollowPacket { creature_id })
}

/// Parse a close-container packet.
///
/// Wire format: container_id (u8)
pub fn parse_close_container(msg: &mut NetworkMessage) -> Result<CloseContainerPacket, String> {
    let container_id = msg.get_u8();
    if msg.is_overrun() {
        return Err("close container packet overrun".into());
    }
    Ok(CloseContainerPacket { container_id })
}

/// Parse an up-arrow-container packet.
///
/// Wire format: container_id (u8)
pub fn parse_up_arrow_container(
    msg: &mut NetworkMessage,
) -> Result<UpArrowContainerPacket, String> {
    let container_id = msg.get_u8();
    if msg.is_overrun() {
        return Err("up arrow container packet overrun".into());
    }
    Ok(UpArrowContainerPacket { container_id })
}

/// Parse a throw (move item) packet.
///
/// Wire format:
/// - from_x (u16), from_y (u16), from_z (u8)
/// - sprite_id (u16)
/// - from_stackpos (u8)
/// - to_x (u16), to_y (u16), to_z (u8)
/// - count (u8)
pub fn parse_throw(msg: &mut NetworkMessage) -> Result<ThrowPacket, String> {
    let from_x = msg.get_u16();
    let from_y = msg.get_u16();
    let from_z = msg.get_u8();
    let sprite_id = msg.get_u16();
    let from_stackpos = msg.get_u8();
    let to_x = msg.get_u16();
    let to_y = msg.get_u16();
    let to_z = msg.get_u8();
    let count = msg.get_u8();
    if msg.is_overrun() {
        return Err("throw packet overrun".into());
    }
    Ok(ThrowPacket {
        from_x,
        from_y,
        from_z,
        sprite_id,
        from_stackpos,
        to_x,
        to_y,
        to_z,
        count,
    })
}

/// Parse a look-in-battle-list packet.
///
/// Wire format: creature_id (u32)
pub fn parse_look_in_battle_list(
    msg: &mut NetworkMessage,
) -> Result<LookInBattleListPacket, String> {
    let creature_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("look in battle list packet overrun".into());
    }
    Ok(LookInBattleListPacket { creature_id })
}

/// Parse an invite-to-party packet.
///
/// Wire format: target_id (u32)
pub fn parse_invite_to_party(msg: &mut NetworkMessage) -> Result<InviteToPartyPacket, String> {
    let target_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("invite to party packet overrun".into());
    }
    Ok(InviteToPartyPacket { target_id })
}

/// Parse a join-party packet.
///
/// Wire format: target_id (u32)
pub fn parse_join_party(msg: &mut NetworkMessage) -> Result<JoinPartyPacket, String> {
    let target_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("join party packet overrun".into());
    }
    Ok(JoinPartyPacket { target_id })
}

/// Parse a revoke-party-invite packet.
///
/// Wire format: target_id (u32)
pub fn parse_revoke_party_invite(
    msg: &mut NetworkMessage,
) -> Result<RevokePartyInvitePacket, String> {
    let target_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("revoke party invite packet overrun".into());
    }
    Ok(RevokePartyInvitePacket { target_id })
}

/// Parse a pass-party-leadership packet.
///
/// Wire format: target_id (u32)
pub fn parse_pass_party_leadership(
    msg: &mut NetworkMessage,
) -> Result<PassPartyLeadershipPacket, String> {
    let target_id = msg.get_u32();
    if msg.is_overrun() {
        return Err("pass party leadership packet overrun".into());
    }
    Ok(PassPartyLeadershipPacket { target_id })
}

/// Parse enable-shared-party-experience packet.
///
/// Wire format: active (u8, 1=active)
pub fn parse_enable_shared_party_experience(
    msg: &mut NetworkMessage,
) -> Result<EnableSharedPartyExperiencePacket, String> {
    let raw = msg.get_u8();
    if msg.is_overrun() {
        return Err("enable shared party experience packet overrun".into());
    }
    Ok(EnableSharedPartyExperiencePacket { active: raw == 1 })
}

/// Parse a modal-window-answer packet.
///
/// Wire format: window_id (u32), button (u8), choice (u8)
pub fn parse_modal_window_answer(
    msg: &mut NetworkMessage,
) -> Result<ModalWindowAnswerPacket, String> {
    let window_id = msg.get_u32();
    let button = msg.get_u8();
    let choice = msg.get_u8();
    if msg.is_overrun() {
        return Err("modal window answer packet overrun".into());
    }
    Ok(ModalWindowAnswerPacket {
        window_id,
        button,
        choice,
    })
}

/// Parse a browse-field packet.
///
/// Wire format: pos_x (u16), pos_y (u16), pos_z (u8)
pub fn parse_browse_field(msg: &mut NetworkMessage) -> Result<BrowseFieldPacket, String> {
    let pos_x = msg.get_u16();
    let pos_y = msg.get_u16();
    let pos_z = msg.get_u8();
    if msg.is_overrun() {
        return Err("browse field packet overrun".into());
    }
    Ok(BrowseFieldPacket {
        pos_x,
        pos_y,
        pos_z,
    })
}

/// Parse a seek-in-container packet.
///
/// Wire format: container_id (u8), index (u16)
pub fn parse_seek_in_container(msg: &mut NetworkMessage) -> Result<SeekInContainerPacket, String> {
    let container_id = msg.get_u8();
    let index = msg.get_u16();
    if msg.is_overrun() {
        return Err("seek in container packet overrun".into());
    }
    Ok(SeekInContainerPacket {
        container_id,
        index,
    })
}

/// Parse a market-browse packet.
///
/// Wire format: browse_id (u8); if browse_id is item browse: sprite_id (u16)
pub fn parse_market_browse(msg: &mut NetworkMessage) -> Result<MarketBrowsePacket, String> {
    const MARKETREQUEST_OWN_OFFERS: u8 = 0xFE;
    const MARKETREQUEST_OWN_HISTORY: u8 = 0xFF;
    let browse_id = msg.get_u8();
    let sprite_id =
        if browse_id != MARKETREQUEST_OWN_OFFERS && browse_id != MARKETREQUEST_OWN_HISTORY {
            msg.get_u16()
        } else {
            0
        };
    if msg.is_overrun() {
        return Err("market browse packet overrun".into());
    }
    Ok(MarketBrowsePacket {
        browse_id,
        sprite_id,
    })
}

/// Parse a market-cancel-offer packet.
///
/// Wire format: timestamp (u32), counter (u16)
pub fn parse_market_cancel_offer(
    msg: &mut NetworkMessage,
) -> Result<MarketCancelOfferPacket, String> {
    let timestamp = msg.get_u32();
    let counter = msg.get_u16();
    if msg.is_overrun() {
        return Err("market cancel offer packet overrun".into());
    }
    Ok(MarketCancelOfferPacket { timestamp, counter })
}

/// Parse a market-accept-offer packet.
///
/// Wire format: timestamp (u32), counter (u16), amount (u16)
pub fn parse_market_accept_offer(
    msg: &mut NetworkMessage,
) -> Result<MarketAcceptOfferPacket, String> {
    let timestamp = msg.get_u32();
    let counter = msg.get_u16();
    let amount = msg.get_u16();
    if msg.is_overrun() {
        return Err("market accept offer packet overrun".into());
    }
    Ok(MarketAcceptOfferPacket {
        timestamp,
        counter,
        amount,
    })
}

/// Parse an add-vip-by-name packet.
///
/// Wire format: name (length-prefixed string)
pub fn parse_add_vip_by_name(msg: &mut NetworkMessage) -> Result<AddVipByNamePacket, String> {
    let name = msg.get_string(0);
    if msg.is_overrun() {
        return Err("add vip by name packet overrun".into());
    }
    Ok(AddVipByNamePacket { name })
}

/// Parse a remove-vip packet.
///
/// Wire format: guid (u32)
pub fn parse_remove_vip(msg: &mut NetworkMessage) -> Result<RemoveVipPacket, String> {
    let guid = msg.get_u32();
    if msg.is_overrun() {
        return Err("remove vip packet overrun".into());
    }
    Ok(RemoveVipPacket { guid })
}

/// Parse a rotate-item packet.
///
/// Wire format: pos_x (u16), pos_y (u16), pos_z (u8), sprite_id (u16), stackpos (u8)
pub fn parse_rotate_item(msg: &mut NetworkMessage) -> Result<RotateItemPacket, String> {
    let pos_x = msg.get_u16();
    let pos_y = msg.get_u16();
    let pos_z = msg.get_u8();
    let sprite_id = msg.get_u16();
    let stackpos = msg.get_u8();
    if msg.is_overrun() {
        return Err("rotate item packet overrun".into());
    }
    Ok(RotateItemPacket {
        pos_x,
        pos_y,
        pos_z,
        sprite_id,
        stackpos,
    })
}

/// Parse an equip-object (hotkey equip) packet.
///
/// Wire format: sprite_id (u16)
pub fn parse_equip_object(msg: &mut NetworkMessage) -> Result<EquipObjectPacket, String> {
    let sprite_id = msg.get_u16();
    if msg.is_overrun() {
        return Err("equip object packet overrun".into());
    }
    Ok(EquipObjectPacket { sprite_id })
}

/// Parse a text-window packet.
///
/// Wire format: window_text_id (u32), text (length-prefixed string)
pub fn parse_text_window(msg: &mut NetworkMessage) -> Result<TextWindowPacket, String> {
    let window_text_id = msg.get_u32();
    let text = msg.get_string(0);
    if msg.is_overrun() {
        return Err("text window packet overrun".into());
    }
    Ok(TextWindowPacket {
        window_text_id,
        text,
    })
}

/// Parse a house-window packet.
///
/// Wire format: door_id (u8), window_id (u32), text (length-prefixed string)
pub fn parse_house_window(msg: &mut NetworkMessage) -> Result<HouseWindowPacket, String> {
    let door_id = msg.get_u8();
    let window_id = msg.get_u32();
    let text = msg.get_string(0);
    if msg.is_overrun() {
        return Err("house window packet overrun".into());
    }
    Ok(HouseWindowPacket {
        door_id,
        window_id,
        text,
    })
}

/// Parse an open-channel packet.
///
/// Wire format: channel_id (u16)
pub fn parse_open_channel(msg: &mut NetworkMessage) -> Result<OpenChannelPacket, String> {
    let channel_id = msg.get_u16();
    if msg.is_overrun() {
        return Err("open channel packet overrun".into());
    }
    Ok(OpenChannelPacket { channel_id })
}

/// Parse a close-channel packet.
///
/// Wire format: channel_id (u16)
pub fn parse_close_channel(msg: &mut NetworkMessage) -> Result<CloseChannelPacket, String> {
    let channel_id = msg.get_u16();
    if msg.is_overrun() {
        return Err("close channel packet overrun".into());
    }
    Ok(CloseChannelPacket { channel_id })
}

/// Parse a channel-invite packet.
///
/// Wire format: name (length-prefixed string)
pub fn parse_channel_invite(msg: &mut NetworkMessage) -> Result<ChannelInvitePacket, String> {
    let name = msg.get_string(0);
    if msg.is_overrun() {
        return Err("channel invite packet overrun".into());
    }
    Ok(ChannelInvitePacket { name })
}

/// Parse a channel-exclude packet.
///
/// Wire format: name (length-prefixed string)
pub fn parse_channel_exclude(msg: &mut NetworkMessage) -> Result<ChannelExcludePacket, String> {
    let name = msg.get_string(0);
    if msg.is_overrun() {
        return Err("channel exclude packet overrun".into());
    }
    Ok(ChannelExcludePacket { name })
}

/// Parse an open-private-channel packet.
///
/// Wire format: receiver (length-prefixed string)
pub fn parse_open_private_channel(
    msg: &mut NetworkMessage,
) -> Result<OpenPrivateChannelPacket, String> {
    let receiver = msg.get_string(0);
    if msg.is_overrun() {
        return Err("open private channel packet overrun".into());
    }
    Ok(OpenPrivateChannelPacket { receiver })
}

// ---------------------------------------------------------------------------
// Additional serialize functions
// ---------------------------------------------------------------------------

/// Serialize a ping packet (opcode `0x1D`).
///
/// Mirrors C++ `ProtocolGame::sendPing` in
/// `forgottenserver/src/protocolgame.cpp` lines 2533–2538.  This packet is
/// legitimately a single opcode byte in the C++ source — no payload —
/// hence the matching one-byte Rust body.  Full parity since Tier 3.
pub fn serialize_ping() -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x1D);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a ping-back packet (opcode `0x1E`).
///
/// Mirrors C++ `ProtocolGame::sendPingBack` in
/// `forgottenserver/src/protocolgame.cpp` lines 2540–2545.  Legitimately a
/// single opcode byte in the C++ source — no payload — hence the matching
/// one-byte Rust body.  Full parity since Tier 3.
pub fn serialize_ping_back() -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x1E);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize `sendCreatureHealth` (opcode `0x8C`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureHealth` in
/// `forgottenserver/src/protocolgame.cpp` lines 2575–2588.
///
/// The C++ source replaces `health_percent` with `0` when the creature
/// has `isHealthHidden()` set; callers must perform that check before
/// passing the value here.
///
/// Wire layout:
/// * opcode `0x8C` (u8)
/// * `creature_id` (u32)
/// * `health_percent` (u8 — `ceil(health / max(maxHealth,1) * 100)`, or
///   `0` when health-hidden)
pub fn serialize_creature_health(creature_id: u32, health_percent: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x8C);
    out.add_u32(creature_id);
    out.add_u8(health_percent);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a magic-effect packet.
///
/// Mirrors C++ `ProtocolGame::sendMagicEffect` in
/// `forgottenserver/src/protocolgame.cpp` lines 2560–2573.
///
/// Wire format (opcode 0x83):
/// - pos_x (u16), pos_y (u16), pos_z (u8)
/// - MAGIC_EFFECTS_CREATE_EFFECT byte (0x03)
/// - type (u8)
/// - MAGIC_EFFECTS_END_LOOP byte (0x00)
///
/// Note: caller is responsible for the C++ `canSee(pos)` guard (visibility
/// check) — this function unconditionally writes the packet.
pub fn serialize_magic_effect(pos_x: u16, pos_y: u16, pos_z: u8, effect_type: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x83);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_u8(0x03); // MAGIC_EFFECTS_CREATE_EFFECT
    out.add_u8(effect_type);
    out.add_u8(0x00); // MAGIC_EFFECTS_END_LOOP
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a close-container packet.
///
/// Mirrors C++ `ProtocolGame::sendCloseContainer` in
/// `forgottenserver/src/protocolgame.cpp` lines 2391–2397.  The C++ source
/// writes only the opcode and the client-side container id (`cid`); this
/// Rust port matches that layout byte-for-byte.  WAS-COMPLETE since Tier 3.
///
/// Wire layout:
/// * opcode `0x6F` (u8)
/// * `container_id` (u8 — client-side container slot, `cid`)
pub fn serialize_close_container(container_id: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x6F);
    out.add_u8(container_id);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize `sendContainer` (opcode `0x6E`).
///
/// Mirrors C++ `ProtocolGame::sendContainer` in
/// `forgottenserver/src/protocolgame.cpp` lines 1900–1936.  The C++ source
/// branches on `container->getID() == ITEM_BROWSEFIELD` and uses the
/// per-instance `addItem(container)` plus the per-instance loop over
/// `container->getItemList()`.  This Rust port flattens the data into
/// plain arguments so the network crate stays free of cross-crate
/// `Container`/`Item` imports — callers pre-compute the meta block per
/// item and pass it through.  The browse-field branch is intentionally
/// not modelled here; callers that need it should pass the bag-item meta
/// and `"Browse Field"` as `container_name` explicitly.
///
/// Wire layout (all little-endian):
/// * opcode `0x6E` (u8)
/// * `cid` (u8 — client-side container slot)
/// * container item payload via `add_item_payload(container_count,
///   container_meta)` (mirrors C++ `addItem(container)`)
/// * `container_name` (length-prefixed string)
/// * `capacity` (u8)
/// * `has_parent` (u8 — 0x00 / 0x01)
/// * literal `0x00` (u8 — show search icon, currently always disabled)
/// * `is_unlocked` (u8 — 0x00 / 0x01, drag-and-drop)
/// * `has_pagination` (u8 — 0x00 / 0x01)
/// * `contained_count` (u16 — total items in the container)
/// * `first_index` (u16 — first item in the visible window)
/// * `items_to_send` (u8 — count of (count, meta) tuples that follow)
/// * for each tuple: item payload via `add_item_payload(count, meta)`
///
/// `items_to_send` is `items.len()` clamped to `0xFF`; the C++ source
/// clamps to `min(capacity, contained_count - first_index, u8::MAX)`.
/// Callers pre-compute that window and pass the resulting slice.
#[allow(clippy::too_many_arguments)]
pub fn serialize_container(
    msg: &mut OutputMessage,
    cid: u8,
    container_meta: ItemTypeMeta,
    container_name: &str,
    container_count: u8,
    capacity: u8,
    has_parent: bool,
    is_unlocked: bool,
    has_pagination: bool,
    contained_count: u16,
    first_index: u16,
    items: &[(u8, ItemTypeMeta)],
) {
    msg.add_u8(0x6E);
    msg.add_u8(cid);

    append_item_payload(msg, container_count, container_meta);
    msg.add_string(container_name);

    msg.add_u8(capacity);
    msg.add_u8(if has_parent { 0x01 } else { 0x00 });
    msg.add_u8(0x00); // show search icon
    msg.add_u8(if is_unlocked { 0x01 } else { 0x00 });
    msg.add_u8(if has_pagination { 0x01 } else { 0x00 });

    msg.add_u16(contained_count);
    msg.add_u16(first_index);

    let items_to_send: u8 = items.len().min(u8::MAX as usize) as u8;
    msg.add_u8(items_to_send);
    for (count, meta) in items.iter().take(items_to_send as usize) {
        append_item_payload(msg, *count, *meta);
    }
}

/// Serialize `sendEmptyContainer` (opcode `0x6E`).
///
/// Mirrors C++ `ProtocolGame::sendEmptyContainer` in
/// `forgottenserver/src/protocolgame.cpp` lines 1938–1957.  The C++ source
/// hard-codes a placeholder bag (`ITEM_BAG` = 1987, count = 1) with the
/// string `"Placeholder"`, capacity 8, locked, no pagination, and zero
/// items — used by the client when a real container can't be resolved.
/// This Rust port hard-codes the exact same byte sequence.
///
/// Wire layout (all little-endian):
/// * opcode `0x6E` (u8)
/// * `cid` (u8)
/// * item payload for ITEM_BAG (u16 LE `0x07C3` = 1987 — bag is non-stackable
///   with no special flags, so no payload byte follows)
/// * `"Placeholder"` (length-prefixed string: u16 LE `0x000B` + 11 bytes)
/// * `0x08` (capacity)
/// * `0x00` (no parent)
/// * `0x00` (show search icon)
/// * `0x01` (is unlocked)
/// * `0x00` (no pagination)
/// * `0x0000` (u16 contained count)
/// * `0x0000` (u16 first index)
/// * `0x00` (u8 items-to-send)
pub fn serialize_empty_container(msg: &mut OutputMessage, cid: u8) {
    msg.add_u8(0x6E);
    msg.add_u8(cid);

    // ITEM_BAG client id = 1987, plain non-stackable: just the u16 client id.
    msg.add_u16(1987);
    msg.add_string("Placeholder");

    msg.add_u8(8);
    msg.add_u8(0x00);
    msg.add_u8(0x00);
    msg.add_u8(0x01);
    msg.add_u8(0x00);
    msg.add_u16(0);
    msg.add_u16(0);
    msg.add_u8(0x00);
}

/// Serialize `sendAddContainerItem` (opcode `0x70`).
///
/// Mirrors C++ `ProtocolGame::sendAddContainerItem` in
/// `forgottenserver/src/protocolgame.cpp` lines 2902–2910.  Notifies the
/// client that an item was inserted at `slot` in container `cid`.  The
/// C++ source calls per-instance `addItem(item)`; this Rust port flattens
/// to a `(count, meta)` tuple and uses `add_item_payload`.
///
/// Wire layout (all little-endian):
/// * opcode `0x70` (u8)
/// * `cid` (u8)
/// * `slot` (u16)
/// * item payload via `add_item_payload(item_count, item_meta)`
pub fn serialize_add_container_item(
    msg: &mut OutputMessage,
    cid: u8,
    slot: u16,
    item_count: u8,
    item_meta: ItemTypeMeta,
) {
    msg.add_u8(0x70);
    msg.add_u8(cid);
    msg.add_u16(slot);
    append_item_payload(msg, item_count, item_meta);
}

/// Serialize `sendUpdateContainerItem` (opcode `0x71`).
///
/// Mirrors C++ `ProtocolGame::sendUpdateContainerItem` in
/// `forgottenserver/src/protocolgame.cpp` lines 2912–2920.  Notifies the
/// client that an item at `slot` in container `cid` was replaced /
/// updated.  Identical wire layout to `sendAddContainerItem` except for
/// the opcode byte.
///
/// Wire layout (all little-endian):
/// * opcode `0x71` (u8)
/// * `cid` (u8)
/// * `slot` (u16)
/// * item payload via `add_item_payload(item_count, item_meta)`
pub fn serialize_update_container_item(
    msg: &mut OutputMessage,
    cid: u8,
    slot: u16,
    item_count: u8,
    item_meta: ItemTypeMeta,
) {
    msg.add_u8(0x71);
    msg.add_u8(cid);
    msg.add_u16(slot);
    append_item_payload(msg, item_count, item_meta);
}

/// Serialize `sendRemoveContainerItem` (opcode `0x72`).
///
/// Mirrors C++ `ProtocolGame::sendRemoveContainerItem` in
/// `forgottenserver/src/protocolgame.cpp` lines 2922–2934.  Notifies the
/// client that an item at `slot` in container `cid` was removed.  When
/// the container has pagination, removing one item can shift another
/// into the visible window — the C++ source passes that replacement as
/// `lastItem` (or `nullptr` if no replacement); this Rust port models it
/// as `Option<(u8, ItemTypeMeta)>`.  `None` writes the literal
/// `0x0000` (u16) sentinel the C++ source uses for the "no replacement"
/// branch.
///
/// Wire layout (all little-endian):
/// * opcode `0x72` (u8)
/// * `cid` (u8)
/// * `slot` (u16)
/// * if `last_item.is_some()`: item payload via `add_item_payload`
/// * else: literal `0x0000` (u16 — sentinel)
pub fn serialize_remove_container_item(
    msg: &mut OutputMessage,
    cid: u8,
    slot: u16,
    last_item: Option<(u8, ItemTypeMeta)>,
) {
    msg.add_u8(0x72);
    msg.add_u8(cid);
    msg.add_u16(slot);
    match last_item {
        Some((count, meta)) => append_item_payload(msg, count, meta),
        None => msg.add_u16(0x0000),
    }
}

/// Serialize a cancel-walk packet.
///
/// Mirrors C++ `ProtocolGame::sendCancelWalk` in
/// `forgottenserver/src/protocolgame.cpp` lines 2518–2524.  The C++ pulls
/// the direction from `player->getDirection()`; this Rust port takes it as
/// a plain `u8` to avoid a `Player` cross-crate import.  Full parity
/// since Tier 3.
///
/// Wire layout:
/// * opcode `0xB5` (u8)
/// * `direction` (u8 — `Direction_t`)
pub fn serialize_cancel_walk(direction: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xB5);
    out.add_u8(direction);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a cancel-target packet.
///
/// Mirrors C++ `ProtocolGame::sendCancelTarget` in
/// `forgottenserver/src/protocolgame.cpp` lines 2500–2506.  Full parity
/// since Tier 3.
///
/// Wire layout (little-endian):
/// * opcode `0xA3` (u8)
/// * literal `u32` `0x00000000` — target creature id placeholder
pub fn serialize_cancel_target() -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xA3);
    out.add_u32(0x00);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a change-speed packet.
///
/// Mirrors C++ `ProtocolGame::sendChangeSpeed` in
/// `forgottenserver/src/protocolgame.cpp` lines 2508–2516.  The C++ source
/// halves both speed values inline (`creature->getBaseSpeed() / 2` and
/// `speed / 2`); this Rust port takes the already-halved values to keep
/// the network crate free of `Creature` cross-crate imports — same
/// convention as `serialize_stats`'s `base_speed_half`.
///
/// Wire layout (all little-endian):
/// * opcode `0x8F` (u8)
/// * `creature_id` (u32)
/// * `base_speed_half` (u16 — C++ `creature->getBaseSpeed() / 2`)
/// * `new_speed_half` (u16 — C++ `speed / 2`)
pub fn serialize_change_speed(
    creature_id: u32,
    base_speed_half: u16,
    new_speed_half: u16,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x8F);
    out.add_u32(creature_id);
    out.add_u16(base_speed_half);
    out.add_u16(new_speed_half);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a close-private channel packet.
///
/// Mirrors C++ `ProtocolGame::sendClosePrivate` in
/// `forgottenserver/src/protocolgame.cpp` lines 1814–1820.  Full parity
/// since Tier 3.
///
/// Wire layout (opcode `0xB3`):
/// * `channel_id` (u16)
pub fn serialize_close_private(channel_id: u16) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xB3);
    out.add_u16(channel_id);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a create-private-channel packet.
///
/// Mirrors C++ `ProtocolGame::sendCreatePrivateChannel` in
/// `forgottenserver/src/protocolgame.cpp` lines 1822–1832.
///
/// Wire layout (opcode `0xB2`):
/// * `channel_id` (u16)
/// * `channel_name` (length-prefixed string)
/// * u16 `0x0001` — channel users count (always 1, the owner)
/// * `owner_name` (length-prefixed string)
/// * u16 `0x0000` — invited users count (always 0)
pub fn serialize_create_private_channel(
    channel_id: u16,
    channel_name: &str,
    owner_name: &str,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xB2);
    out.add_u16(channel_id);
    out.add_string(channel_name);
    out.add_u16(0x0001);
    out.add_string(owner_name);
    out.add_u16(0x0000);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a channel-event packet.
///
/// Mirrors C++ `ProtocolGame::sendChannelEvent` in
/// `forgottenserver/src/protocolgame.cpp` lines 1583–1591.  Full parity
/// since Tier 3.
///
/// Wire layout (opcode `0xF3`):
/// * `channel_id` (u16)
/// * `player_name` (length-prefixed string)
/// * `event_type` (u8 — `ChannelEvent_t` enum)
pub fn serialize_channel_event(channel_id: u16, player_name: &str, event_type: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xF3);
    out.add_u16(channel_id);
    out.add_string(player_name);
    out.add_u8(event_type);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an open-private-channel-send packet.
///
/// Mirrors C++ `ProtocolGame::sendOpenPrivateChannel` in
/// `forgottenserver/src/protocolgame.cpp` lines 1575–1581.  Full parity
/// since Tier 3.
///
/// Wire layout (opcode `0xAD`):
/// * `receiver` (length-prefixed string)
pub fn serialize_open_private_channel(receiver: &str) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xAD);
    out.add_string(receiver);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a tutorial packet.
///
/// Wire format (opcode 0xDC):
/// - tutorial_id (u8)
pub fn serialize_tutorial(tutorial_id: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xDC);
    out.add_u8(tutorial_id);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an add-marker packet.
///
/// Wire format (opcode 0xDD):
/// - 0x00 (unknown byte)
/// - pos_x (u16), pos_y (u16), pos_z (u8)
/// - mark_type (u8)
/// - desc (length-prefixed string)
pub fn serialize_add_marker(
    pos_x: u16,
    pos_y: u16,
    pos_z: u8,
    mark_type: u8,
    desc: &str,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xDD);
    out.add_u8(0x00);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_u8(mark_type);
    out.add_string(desc);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a relogin-window packet.
///
/// Wire format (opcode 0x28):
/// - 0x00 (padding)
/// - unfair_fight_reduction (u8)
/// - 0x00 (can use death redemption)
pub fn serialize_relogin_window(unfair_fight_reduction: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x28);
    out.add_u8(0x00);
    out.add_u8(unfair_fight_reduction);
    out.add_u8(0x00);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a VIP entry packet.
///
/// Mirrors C++ `ProtocolGame::sendVIP` in
/// `forgottenserver/src/protocolgame.cpp` lines 3255–3268.  Full parity
/// since Tier 3.
///
/// Wire layout (all little-endian):
/// * opcode `0xD2` (u8)
/// * `guid` (u32)
/// * `name` (u16 length + UTF-8 bytes)
/// * `description` (u16 length + UTF-8 bytes)
/// * `icon` (u32 — capped at 10, mirrors `std::min<uint32_t>(10, icon)`)
/// * notify flag (u8 `0x01` or `0x00`)
/// * `status` (u8 — `VipStatus_t`)
/// * literal byte `0x00` — `vipGroups` placeholder
pub fn serialize_vip(
    guid: u32,
    name: &str,
    description: &str,
    icon: u32,
    notify: bool,
    status: u8,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xD2);
    out.add_u32(guid);
    out.add_string(name);
    out.add_string(description);
    out.add_u32(icon.min(10));
    out.add_u8(if notify { 0x01 } else { 0x00 });
    out.add_u8(status);
    out.add_u8(0x00); // vip groups placeholder
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an updated-VIP-status packet.
///
/// Mirrors C++ `ProtocolGame::sendUpdatedVIPStatus` in
/// `forgottenserver/src/protocolgame.cpp` lines 3246–3253.  Full parity
/// since Tier 3.
///
/// Wire layout (little-endian):
/// * opcode `0xD3` (u8)
/// * `guid` (u32)
/// * `new_status` (u8 — `VipStatus_t`)
pub fn serialize_updated_vip_status(guid: u32, new_status: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xD3);
    out.add_u32(guid);
    out.add_u8(new_status);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a spell-cooldown packet.
///
/// Wire format (opcode 0xA4):
/// - spell_id (u16)
/// - time (u32)
pub fn serialize_spell_cooldown(spell_id: u8, time: u32) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xA4);
    out.add_u16(spell_id as u16);
    out.add_u32(time);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a spell-group-cooldown packet.
///
/// Wire format (opcode 0xA5):
/// - group_id (u8)
/// - time (u32)
pub fn serialize_spell_group_cooldown(group_id: u8, time: u32) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xA5);
    out.add_u8(group_id);
    out.add_u32(time);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a use-item-cooldown packet.
///
/// Wire format (opcode 0xA6):
/// - time (u32)
pub fn serialize_use_item_cooldown(time: u32) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xA6);
    out.add_u32(time);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a supply-used packet.
///
/// Wire format (opcode 0xCE):
/// - client_id (u16)
pub fn serialize_supply_used(client_id: u16) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xCE);
    out.add_u16(client_id);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a session-end packet.
///
/// Wire format (opcode 0x18):
/// - reason (u8)
pub fn serialize_session_end(reason: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x18);
    out.add_u8(reason);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a pending-state-entered packet (opcode 0x0A).
pub fn serialize_pending_state_entered() -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x0A);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an enter-world packet (opcode 0x0F).
pub fn serialize_enter_world() -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x0F);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a fight-modes packet.
///
/// Wire format (opcode 0xA7):
/// - fight_mode (u8)
/// - chase_mode (u8)
/// - secure_mode (u8)
/// - pvp_mode (u8, always 0 = dove)
pub fn serialize_fight_modes(fight_mode: u8, chase_mode: u8, secure_mode: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xA7);
    out.add_u8(fight_mode);
    out.add_u8(chase_mode);
    out.add_u8(secure_mode);
    out.add_u8(0x00); // PVP_MODE_DOVE
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a FYI-box (informational popup) packet.
///
/// Mirrors C++ `ProtocolGame::sendFYIBox` in
/// `forgottenserver/src/protocolgame.cpp` lines 2590–2596.  Full parity
/// since Tier 3.
///
/// Wire layout (opcode `0x15`):
/// * `message` (length-prefixed string)
pub fn serialize_fyi_box(message: &str) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x15);
    out.add_string(message);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an experience-tracker packet.
///
/// Mirrors C++ `ProtocolGame::sendExperienceTracker` in
/// `forgottenserver/src/protocolgame.cpp` lines 1715–1722 — which calls
/// `msg.add<int64_t>(rawExp)` / `msg.add<int64_t>(finalExp)`.  The Rust
/// port casts the signed inputs to `u64` (bit-pattern preservation),
/// matching the on-wire bytes exactly.  Full parity since Tier 3.
///
/// Wire layout (all little-endian):
/// * opcode `0xAF` (u8)
/// * `raw_exp` (i64 — written via `u64` bit pattern)
/// * `final_exp` (i64 — written via `u64` bit pattern)
pub fn serialize_experience_tracker(raw_exp: i64, final_exp: i64) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xAF);
    out.add_u64(raw_exp as u64);
    out.add_u64(final_exp as u64);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize `sendCreatureLight` (opcode `0x8D`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureLight` in
/// `forgottenserver/src/protocolgame.cpp` lines 1606–1620.
///
/// The C++ source overrides `level` to `0xFF` when the viewing player is
/// an access player (`player->isAccessPlayer()`); callers must apply that
/// override before passing the value here.
///
/// Wire layout:
/// * opcode `0x8D` (u8)
/// * `creature_id` (u32)
/// * `level` (u8 — `LightInfo::level`, or `0xFF` for access players)
/// * `color` (u8 — `LightInfo::color`)
pub fn serialize_creature_light(creature_id: u32, level: u8, color: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x8D);
    out.add_u32(creature_id);
    out.add_u8(level);
    out.add_u8(color);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize `sendCreatureSkull` (opcode `0x90`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureSkull` in
/// `forgottenserver/src/protocolgame.cpp` lines 1648–1663.
///
/// The C++ source early-returns when the world type is not PVP
/// (`g_game.getWorldType() != WORLD_TYPE_PVP`); callers must perform that
/// world-type check before invoking this serializer.
///
/// Wire layout:
/// * opcode `0x90` (u8)
/// * `creature_id` (u32)
/// * `skull` (u8 — `player->getSkullClient(creature)`)
pub fn serialize_creature_skull(creature_id: u32, skull: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x90);
    out.add_u32(creature_id);
    out.add_u8(skull);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize `sendCreatureSquare` (opcode `0x93`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureSquare` in
/// `forgottenserver/src/protocolgame.cpp` lines 1665–1677.
///
/// Wire layout:
/// * opcode `0x93` (u8)
/// * `creature_id` (u32)
/// * `0x01` (u8 — square type literal in C++)
/// * `color` (u8 — `SquareColor_t` enum value)
pub fn serialize_creature_square(creature_id: u32, color: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x93);
    out.add_u32(creature_id);
    out.add_u8(0x01);
    out.add_u8(color);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize `sendCreatureWalkthrough` (opcode `0x92`).
///
/// Mirrors C++ `ProtocolGame::sendCreatureWalkthrough` in
/// `forgottenserver/src/protocolgame.cpp` lines 1622–1633.
///
/// Wire layout:
/// * opcode `0x92` (u8)
/// * `creature_id` (u32)
/// * `walkthrough` byte (u8 — `0x00` if walkthrough, else `0x01`)
pub fn serialize_creature_walkthrough(creature_id: u32, walkthrough: bool) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x92);
    out.add_u32(creature_id);
    out.add_u8(if walkthrough { 0x00 } else { 0x01 });
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a map-description packet.
///
/// Wire format (opcode 0x64):
/// - pos_x (u16), pos_y (u16), pos_z (u8)
///
/// Note: In a full server this includes the full tile map payload.
/// This stub serializes the header position only.
pub fn serialize_map_description_header(pos_x: u16, pos_y: u16, pos_z: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x64);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an add-tile-item packet.
///
/// Wire format (opcode 0x6A — mirrors C++ `ProtocolGame::sendAddTileItem`
/// in `forgottenserver/src/protocolgame.cpp` lines 2609–2621):
/// - opcode `0x6A` (u8)
/// - position (5 bytes: x u16, y u16, z u8)
/// - stackpos (u8)
/// - full `addItem(item)` payload — see `NetworkMessage::add_item_payload`
///   for the exact byte layout (client_id + sub-type byte(s) + optional
///   podium block).
///
/// `count` is the per-instance item count (clamped to 0xFF by the caller for
/// stackable items, or the fluid type for splash/fluid items).
pub fn serialize_add_tile_item(
    pos_x: u16,
    pos_y: u16,
    pos_z: u8,
    stackpos: u8,
    count: u8,
    meta: ItemTypeMeta,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x6A);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_u8(stackpos);
    append_item_payload(&mut out, count, meta);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an update-tile-item packet.
///
/// Wire format (opcode 0x6B — mirrors C++ `ProtocolGame::sendUpdateTileItem`
/// in `forgottenserver/src/protocolgame.cpp` lines 2623–2635):
/// - opcode `0x6B` (u8)
/// - position (5 bytes: x u16, y u16, z u8)
/// - stackpos (u8)
/// - full `addItem(const Item*)` payload — see
///   `NetworkMessage::add_item_instance` for the exact byte layout
///   (client_id + sub-type byte(s) + charges/duration + container quiver
///   bytes + podium block).
///
/// `count` is `item->getItemCount()` (clamped to `0xFF`) or
/// `item->getFluidType()` for splash/fluid items.  `charges` is
/// `item->getCharges()` and `duration_seconds` is `item->getDuration() / 1000`.
#[allow(clippy::too_many_arguments)]
pub fn serialize_update_tile_item(
    pos_x: u16,
    pos_y: u16,
    pos_z: u8,
    stackpos: u8,
    count: u8,
    charges: u32,
    duration_seconds: u32,
    ammo_count: Option<u32>,
    podium: Option<PodiumMeta>,
    meta: ItemTypeMeta,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x6B);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_u8(stackpos);
    append_item_instance(
        &mut out,
        count,
        charges,
        duration_seconds,
        ammo_count,
        podium,
        meta,
    );
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a remove-tile-thing packet.
///
/// Wire format (opcode 0x6C):
/// - pos_x (u16), pos_y (u16), pos_z (u8)
/// - stackpos (u8)
pub fn serialize_remove_tile_thing(pos_x: u16, pos_y: u16, pos_z: u8, stackpos: u8) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x6C);
    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_u8(stackpos);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an inventory-item packet.
///
/// Wire format (mirrors C++ `ProtocolGame::sendInventoryItem` in
/// `forgottenserver/src/protocolgame.cpp` lines 2862–2874):
/// - When `item` is `Some`:
///   * opcode `0x78` (u8)
///   * slot (u8)
///   * full `addItem(item)` payload — see `NetworkMessage::add_item_payload`
///     for the exact byte layout.
/// - When `item` is `None`:
///   * opcode `0x79` (u8)
///   * slot (u8)
pub fn serialize_inventory_item(slot: u8, item: Option<(u8, ItemTypeMeta)>) -> Vec<u8> {
    let mut out = OutputMessage::new();
    match item {
        Some((count, meta)) => {
            out.add_u8(0x78);
            out.add_u8(slot);
            append_item_payload(&mut out, count, meta);
        }
        None => {
            out.add_u8(0x79);
            out.add_u8(slot);
        }
    }
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

// ---------------------------------------------------------------------------
// Player-cluster wire serializers (sendBasicData / sendStats / sendSkills /
// sendIcons / sendClientFeatures / sendStoreBalance / sendVIPEntries)
// ---------------------------------------------------------------------------
//
// Each function mirrors the byte layout of the corresponding C++
// `ProtocolGame::send*` method in
// `forgottenserver/src/protocolgame.cpp`.  Source lines
// are cited per function.  Signatures take plain data so this crate has
// no cross-crate `Player` dependency.

/// Writes an `OutputMessage` equivalent of `NetworkMessage::addDouble`.
///
/// Mirrors the C++ fixed-point double encoding:
/// `[precision u8][((value * 10^precision) + i32::MAX) as u32 LE]`.
/// See `crates/common/src/networkmessage.rs` `add_double` (lines 435–440).
fn append_double(out: &mut OutputMessage, value: f64, precision: u8) {
    out.add_u8(precision);
    let scale = 10f64.powi(precision as i32);
    let scaled = value * scale + i32::MAX as f64;
    out.add_u32(scaled as u32);
}

/// Serialize `sendBasicData` (opcode `0x9F`).
///
/// Mirrors C++ `ProtocolGame::sendBasicData` in
/// `forgottenserver/src/protocolgame.cpp` lines 1751–1774.
///
/// Wire layout:
/// * opcode `0x9F` (u8)
/// * premium flag (u8 `0x01` or `0x00`)
/// * `premium_end` (u32 LE; `0` if non-premium or free-premium server flag)
/// * `vocation_client_id` (u8 — `Vocation::getClientId()`)
/// * prey-system-enabled flag (u8 `0x01` or `0x00`)
/// * spell-list count (u16 LE) followed by `count` × u16 LE spell ids
/// * magic-shield-active flag (u8 `0x01` or `0x00`)
///
/// To match the C++ literal output exactly (which always writes 255
/// entries `0x0000..=0x00FE`), pass `&(0u16..255).collect::<Vec<_>>()`
/// as `spell_ids`.
pub fn serialize_basic_data(
    msg: &mut OutputMessage,
    is_premium: bool,
    premium_end: u32,
    vocation_client_id: u8,
    prey_enabled: bool,
    spell_ids: &[u16],
    magic_shield: bool,
) {
    msg.add_u8(0x9F);
    if is_premium {
        msg.add_u8(0x01);
        msg.add_u32(premium_end);
    } else {
        msg.add_u8(0x00);
        msg.add_u32(0);
    }
    msg.add_u8(vocation_client_id);
    msg.add_u8(if prey_enabled { 0x01 } else { 0x00 });

    msg.add_u16(spell_ids.len() as u16);
    for &spell_id in spell_ids {
        msg.add_u16(spell_id);
    }

    msg.add_u8(if magic_shield { 0x01 } else { 0x00 });
}

/// Serialize `sendStats` / `AddPlayerStats` (opcode `0xA0`).
///
/// Mirrors C++ `ProtocolGame::AddPlayerStats` in
/// `forgottenserver/src/protocolgame.cpp` lines 3499–3540 (invoked from
/// `sendStats` at line 1708).
///
/// Wire layout (all little-endian):
/// * opcode `0xA0` (u8)
/// * `hp` (u32), `hp_max` (u32)
/// * `free_capacity` (u32)
/// * `experience` (u64)
/// * `level` (u16), `level_pct` (u8)
/// * `base_xp_gain`, `voucher_xp_gain`, `grinding_xp_gain`,
///   `store_xp_gain`, `hunting_xp_gain` (5 × u16)
/// * `mp` (u32), `mp_max` (u32)
/// * `soul` (u8)
/// * `stamina_minutes` (u16)
/// * `base_speed_half` (u16 — caller passes `baseSpeed / 2`)
/// * `regen_seconds` (u16 — CONDITION_REGENERATION ticks / 1000)
/// * `offline_train_minutes` (u16 — offlineTrainingTime / 60 / 1000)
/// * `xp_boost_time` (u16 literal `0`)
/// * `xp_boost_enabled_in_store` (u8 literal `0x00`)
/// * `mana_shield` (u32), `mana_shield_max` (u32)
#[allow(clippy::too_many_arguments)]
pub fn serialize_stats(
    msg: &mut OutputMessage,
    hp: u32,
    hp_max: u32,
    free_capacity: u32,
    experience: u64,
    level: u16,
    level_pct: u8,
    base_xp_gain: u16,
    voucher_xp_gain: u16,
    grinding_xp_gain: u16,
    store_xp_gain: u16,
    hunting_xp_gain: u16,
    mp: u32,
    mp_max: u32,
    soul: u8,
    stamina_minutes: u16,
    base_speed_half: u16,
    regen_seconds: u16,
    offline_train_minutes: u16,
    mana_shield: u32,
    mana_shield_max: u32,
) {
    msg.add_u8(0xA0);

    msg.add_u32(hp);
    msg.add_u32(hp_max);

    msg.add_u32(free_capacity);
    msg.add_u64(experience);

    msg.add_u16(level);
    msg.add_u8(level_pct);

    msg.add_u16(base_xp_gain);
    msg.add_u16(voucher_xp_gain);
    msg.add_u16(grinding_xp_gain);
    msg.add_u16(store_xp_gain);
    msg.add_u16(hunting_xp_gain);

    msg.add_u32(mp);
    msg.add_u32(mp_max);

    msg.add_u8(soul);
    msg.add_u16(stamina_minutes);
    msg.add_u16(base_speed_half);

    msg.add_u16(regen_seconds);
    msg.add_u16(offline_train_minutes);

    msg.add_u16(0); // xp boost time (seconds)
    msg.add_u8(0x00); // exp-boost-in-store enabled

    msg.add_u32(mana_shield);
    msg.add_u32(mana_shield_max);
}

/// One regular skill row written by `sendSkills`:
/// `(level, base, base_loyalty, percent)`.
pub type SkillRow = (u16, u16, u16, u16);

/// One special-skill row written by `sendSkills`:
/// `(base_plus_bonus, base)`.
pub type SpecialSkillRow = (u16, u16);

/// Serialize `sendSkills` / `AddPlayerSkills` (opcode `0xA1`).
///
/// Mirrors C++ `ProtocolGame::AddPlayerSkills` in
/// `forgottenserver/src/protocolgame.cpp` lines 3542–3577 (invoked from
/// `sendSkills` at line 2526).
///
/// Wire layout (all little-endian):
/// * opcode `0xA1` (u8)
/// * magic block: `magic.level, magic.base, magic.base_loyalty, magic.pct`
///   (4 × u16)
/// * 7 skill rows (SKILL_FIST..=SKILL_FISHING), each 4 × u16
///   `(level, base, base_loyalty, percent)`
/// * 6 special-skill rows (SPECIALSKILL_FIRST..=SPECIALSKILL_LAST), each
///   2 × u16 `(base_plus_bonus, base)`
/// * 1 byte: element magic level count (literal `0`)
/// * 3 × 2 × u16: fatal, dodge, momentum rows (`(value, base)` each)
/// * `capacity` (u32) twice (base+bonus / base)
pub fn serialize_skills(
    msg: &mut OutputMessage,
    magic: SkillRow,
    skills: &[SkillRow; 7],
    special: &[SpecialSkillRow; 6],
    capacity: u32,
) {
    msg.add_u8(0xA1);
    let (m_level, m_base, m_base_loyalty, m_pct) = magic;
    msg.add_u16(m_level);
    msg.add_u16(m_base);
    msg.add_u16(m_base_loyalty);
    msg.add_u16(m_pct);

    for &(level, base, base_loyalty, percent) in skills {
        msg.add_u16(level);
        msg.add_u16(base);
        msg.add_u16(base_loyalty);
        msg.add_u16(percent);
    }

    for &(value, base) in special {
        msg.add_u16(value);
        msg.add_u16(base);
    }

    msg.add_u8(0); // element magic level count

    // fatal, dodge, momentum
    for _ in 0..3 {
        msg.add_u16(0);
        msg.add_u16(0);
    }

    msg.add_u32(capacity);
    msg.add_u32(capacity);
}

/// Serialize `sendIcons` (opcode `0xA2`).
///
/// Mirrors C++ `ProtocolGame::sendIcons` in
/// `forgottenserver/src/protocolgame.cpp` lines 1892–1898.
///
/// Wire layout:
/// * opcode `0xA2` (u8)
/// * `icons` (u32 LE — bitfield of `Icons_t` flags)
pub fn serialize_icons(msg: &mut OutputMessage, icons: u32) {
    msg.add_u8(0xA2);
    msg.add_u32(icons);
}

/// Serialize `sendClientFeatures` (opcode `0x17`).
///
/// Mirrors C++ `ProtocolGame::sendClientFeatures` in
/// `forgottenserver/src/protocolgame.cpp` lines 1724–1749.  The C++
/// signature is parameterless because all fields are pulled from the
/// `player` member and the global `Creature::speedA/B/C` constants; this
/// Rust port takes them as explicit arguments to keep the network crate
/// dependency-free.
///
/// Wire layout (all little-endian):
/// * opcode `0x17` (u8)
/// * `player_id` (u32)
/// * beat-duration (u16 literal `50`)
/// * `addDouble(speed_a, 3)` — u8 precision + u32 scaled value
/// * `addDouble(speed_b, 3)`
/// * `addDouble(speed_c, 3)`
/// * can-report-bugs flag (u8 `0x01` or `0x00`)
/// * literal byte `0x00` — can change PvP framing option
/// * literal byte `0x00` — expert-mode button enabled
/// * u16 literal `0x0000` — store images URL string
/// * u16 literal `25` — premium coin package size
/// * literal byte `0x00` — exiva button enabled
/// * literal byte `0x00` — Tournament button enabled
pub fn serialize_client_features(
    msg: &mut OutputMessage,
    player_id: u32,
    speed_a: f64,
    speed_b: f64,
    speed_c: f64,
    can_report_bugs: bool,
) {
    msg.add_u8(0x17);

    msg.add_u32(player_id);
    msg.add_u16(50); // beat duration

    append_double(msg, speed_a, 3);
    append_double(msg, speed_b, 3);
    append_double(msg, speed_c, 3);

    msg.add_u8(if can_report_bugs { 0x01 } else { 0x00 });

    msg.add_u8(0x00); // can change PvP framing
    msg.add_u8(0x00); // expert mode button enabled

    msg.add_u16(0x0000); // store images url (string or u16 0x00)
    msg.add_u16(25); // premium coin package size

    msg.add_u8(0x00); // exiva button enabled
    msg.add_u8(0x00); // tournament button enabled
}

/// Serialize `sendStoreBalance` (opcode `0xDF`).
///
/// Mirrors C++ `ProtocolGame::sendStoreBalance` in
/// `forgottenserver/src/protocolgame.cpp` lines 2081–2093.
///
/// Wire layout (all little-endian):
/// * opcode `0xDF` (u8)
/// * literal byte `0x01`
/// * `store_coins` (u32) — total (transferable + non-transferable)
/// * `transferable_coins` (u32)
/// * `auction_coins` (u32 — reserved)
/// * `tournament_coins` (u32)
pub fn serialize_store_balance(
    msg: &mut OutputMessage,
    store_coins: u32,
    transferable_coins: u32,
    auction_coins: u32,
    tournament_coins: u32,
) {
    msg.add_u8(0xDF);
    msg.add_u8(0x01);
    msg.add_u32(store_coins);
    msg.add_u32(transferable_coins);
    msg.add_u32(auction_coins);
    msg.add_u32(tournament_coins);
}

/// One row in the `sendVIPEntries` packet stream:
/// `(guid, name, description, icon, notify, status)`.
pub type VipEntryRow<'a> = (u32, &'a str, &'a str, u32, bool, u8);

/// Serialize `sendVIPEntries` as a sequence of `sendVIP` (opcode `0xD2`)
/// frames concatenated into one `OutputMessage`.
///
/// Mirrors C++ `ProtocolGame::sendVIPEntries` in
/// `forgottenserver/src/protocolgame.cpp` lines 3270–3285 — which simply
/// iterates the player's VIP list and calls `sendVIP` per entry (lines
/// 3255–3268).
///
/// Per-entry wire layout (all little-endian):
/// * opcode `0xD2` (u8)
/// * `guid` (u32)
/// * `name` (u16 length + UTF-8 bytes)
/// * `description` (u16 length + UTF-8 bytes)
/// * `icon` (u32 — `min(10, icon)`)
/// * notify flag (u8 `0x01` or `0x00`)
/// * `status` (u8 — `VipStatus_t`)
/// * literal byte `0x00` — `vipGroups` placeholder
pub fn serialize_vip_entries(msg: &mut OutputMessage, entries: &[VipEntryRow]) {
    for &(guid, name, description, icon, notify, status) in entries {
        msg.add_u8(0xD2);
        msg.add_u32(guid);
        msg.add_string(name);
        msg.add_string(description);
        msg.add_u32(icon.min(10));
        msg.add_u8(if notify { 0x01 } else { 0x00 });
        msg.add_u8(status);
        msg.add_u8(0x00); // vipGroups placeholder
    }
}

// ---------------------------------------------------------------------------
// Shop / merchant packets (Session 6 — wire-shop cluster)
// ---------------------------------------------------------------------------

/// One row in the `sendShop` packet stream — flat data resolved by the
/// caller from `ShopInfo` + `Item::items[itemId]`.
///
/// Tuple order matches the wire layout written by C++ `AddShopItem`:
/// `(client_id, subtype_byte, real_name, weight, buy_price, sell_price)`.
///
/// The `subtype_byte` is `serverFluidToClient(subType)` when the item is a
/// splash or fluid container, otherwise `0x00`.  Caller resolves this
/// because the `serverFluidToClient` table lives in `items.cpp` (game
/// crate concern, not network).  Likewise the caller has already clamped
/// `buy_price` / `sell_price` to non-negative (matching C++ `std::max<u32>`).
pub type ShopItemRow<'a> = (u16, u8, &'a str, u32, u32, u32);

/// Serialize `sendShop` (opcode `0x7A`).
///
/// Mirrors C++ `ProtocolGame::sendShop` in
/// `forgottenserver/src/protocolgame.cpp` lines 1959–1979 (with the
/// per-item `AddShopItem` block at lines 3725–3740).
///
/// Caller resolves the gold coin's `client_id` from
/// `Item::items[ITEM_GOLD_COIN].clientId` and the per-row subtype byte
/// from `serverFluidToClient(...)` (see `ShopItemRow`).
///
/// Wire layout (all little-endian):
/// * opcode `0x7A` (u8)
/// * `npc_name` (u16 length + UTF-8 bytes)
/// * `gold_coin_client_id` (u16 — currency item's client id)
/// * empty string (u16 length `0x0000` — currency-name placeholder)
/// * `items_to_send` (u16 — `min(items.len(), u16::MAX)`)
/// * for each of `items_to_send` rows:
///   * `client_id` (u16)
///   * `subtype_byte` (u8)
///   * `real_name` (u16 length + bytes)
///   * `weight` (u32)
///   * `buy_price` (u32)
///   * `sell_price` (u32)
pub fn serialize_shop(
    msg: &mut OutputMessage,
    npc_name: &str,
    gold_coin_client_id: u16,
    items: &[ShopItemRow],
) {
    msg.add_u8(0x7A);
    msg.add_string(npc_name);
    msg.add_u16(gold_coin_client_id);
    msg.add_string(""); // currency name placeholder
    let items_to_send = items.len().min(u16::MAX as usize) as u16;
    msg.add_u16(items_to_send);
    for &(client_id, subtype_byte, real_name, weight, buy_price, sell_price) in
        items.iter().take(items_to_send as usize)
    {
        msg.add_u16(client_id);
        msg.add_u8(subtype_byte);
        msg.add_string(real_name);
        msg.add_u32(weight);
        msg.add_u32(buy_price);
        msg.add_u32(sell_price);
    }
}

/// Serialize `sendCloseShop` (opcode `0x7C`).
///
/// Mirrors C++ `ProtocolGame::sendCloseShop` in
/// `forgottenserver/src/protocolgame.cpp` lines 1981–1986.
///
/// Wire layout:
/// * opcode `0x7C` (u8) — no payload
pub fn serialize_close_shop(msg: &mut OutputMessage) {
    msg.add_u8(0x7C);
}

/// Serialize `sendResourceBalance` (opcode `0xEE`).
///
/// Mirrors C++ `ProtocolGame::sendResourceBalance` in
/// `forgottenserver/src/protocolgame.cpp` lines 2072–2079.
///
/// Wire layout (all little-endian):
/// * opcode `0xEE` (u8)
/// * `resource_type` (u8 — `ResourceTypes_t`; e.g. `0x00`
///   `RESOURCE_BANK_BALANCE`, `0x01` `RESOURCE_GOLD_EQUIPPED`)
/// * `amount` (u64)
pub fn serialize_resource_balance(msg: &mut OutputMessage, resource_type: u8, amount: u64) {
    msg.add_u8(0xEE);
    msg.add_u8(resource_type);
    msg.add_u64(amount);
}

/// Serialize `sendSaleItemList` (two `sendResourceBalance` frames + opcode
/// `0x7B`).
///
/// Mirrors C++ `ProtocolGame::sendSaleItemList` in
/// `forgottenserver/src/protocolgame.cpp` lines 1988–2070.
///
/// The C++ source first calls
/// `sendResourceBalance(RESOURCE_BANK_BALANCE, playerBank)` and
/// `sendResourceBalance(RESOURCE_GOLD_EQUIPPED, playerMoney)` then writes
/// the sale-list packet.  This port emits all three frames in one call so
/// the parity test can lock the full byte stream.
///
/// The complex map building in C++ (which counts player inventory of
/// shop-purchasable items, accounting for subtype matching of fluid
/// containers and splashes) is the caller's concern — it needs Player
/// state.  The caller passes the already-resolved
/// `(client_id, count)` rows.
///
/// Wire layout (all little-endian):
/// 1. `sendResourceBalance(RESOURCE_BANK_BALANCE=0x00, bank_balance)` frame
///    (10 bytes: `0xEE 0x00` + u64)
/// 2. `sendResourceBalance(RESOURCE_GOLD_EQUIPPED=0x01, equipped_balance)`
///    frame (10 bytes: `0xEE 0x01` + u64)
/// 3. Sale-list frame:
///    * opcode `0x7B` (u8)
///    * `items_to_send` (u8 — `min(items.len(), u8::MAX)`)
///    * per row: `client_id` (u16) + `min(count, u16::MAX)` (u16)
pub fn serialize_sale_item_list(
    msg: &mut OutputMessage,
    bank_balance: u64,
    equipped_balance: u64,
    items: &[(u16, u32)],
) {
    // Two resource-balance frames precede the sale-list payload (C++:1992–1993).
    serialize_resource_balance(msg, 0x00, bank_balance); // RESOURCE_BANK_BALANCE
    serialize_resource_balance(msg, 0x01, equipped_balance); // RESOURCE_GOLD_EQUIPPED

    msg.add_u8(0x7B);
    let items_to_send = items.len().min(u8::MAX as usize) as u8;
    msg.add_u8(items_to_send);
    for &(client_id, count) in items.iter().take(items_to_send as usize) {
        msg.add_u16(client_id);
        msg.add_u16(count.min(u16::MAX as u32) as u16);
    }
}

// ---------------------------------------------------------------------------
// Market packets (Session 7 — wire-market cluster)
// ---------------------------------------------------------------------------

/// One depot-inventory row in the `sendMarketEnter` packet (opcode `0xF6`).
///
/// Tuple order matches the wire layout:
/// `(ware_id, classification, count)`.
///
/// * `ware_id` — `Item::items[itemId].wareId` (u16). Caller resolves this
///   from `ItemType` (game-crate concern, not network).
/// * `classification` — `Item::items[itemId].classification` (u8); when
///   non-zero, the C++ source writes an additional `0x00` byte (item tier).
/// * `count` — depot+inbox count for the item type, clamped to `u16::MAX`
///   by the caller via `std::min<uint32_t>(0xFFFF, ...)`.
pub type MarketDepotRow = (u16, u8, u16);

/// Serialize `sendMarketEnter` (opcode `0xF6`).
///
/// Mirrors C++ `ProtocolGame::sendMarketEnter` in
/// `forgottenserver/src/protocolgame.cpp` lines 2095–2158.
///
/// The C++ source iterates the player's depot chests + inbox, builds a
/// `(wareId → totalCount)` map, then emits the rows below.  After the
/// 0xF6 frame it also calls `sendResourceBalance` twice and
/// `sendStoreBalance` once — those are emitted by the caller (e.g. by
/// invoking [`serialize_resource_balance`] + [`serialize_store_balance`]).
///
/// Wire layout (all little-endian):
/// * opcode `0xF6` (u8)
/// * `offer_count` (u8 — `min(getPlayerOfferCount, u8::MAX)`)
/// * `items_to_send` (u16 — `min(depot_items.len(), u16::MAX)`)
/// * per row (length up to `items_to_send`):
///   * `ware_id` (u16)
///   * `0x00` (u8) — *only* when `classification > 0`
///   * `count` (u16 — already clamped to `u16::MAX` by the caller)
pub fn serialize_market_enter(
    msg: &mut OutputMessage,
    offer_count: u8,
    depot_items: &[MarketDepotRow],
) {
    msg.add_u8(0xF6);
    msg.add_u8(offer_count);
    let items_to_send = depot_items.len().min(u16::MAX as usize) as u16;
    msg.add_u16(items_to_send);
    for &(ware_id, classification, count) in depot_items.iter().take(items_to_send as usize) {
        msg.add_u16(ware_id);
        if classification > 0 {
            msg.add_u8(0x00); // item tier placeholder
        }
        msg.add_u16(count);
    }
}

/// Serialize `sendMarketLeave` (opcode `0xF7`).
///
/// Mirrors C++ `ProtocolGame::sendMarketLeave` in
/// `forgottenserver/src/protocolgame.cpp` lines 2160–2165.
///
/// Wire layout:
/// * opcode `0xF7` (u8) — no payload
pub fn serialize_market_leave(msg: &mut OutputMessage) {
    msg.add_u8(0xF7);
}

/// One row in a `sendMarketBrowseItem` offer list — flat data resolved by
/// the caller from `MarketOffer`.
///
/// Tuple order matches the wire layout:
/// `(timestamp, counter, amount, price, player_name)`.
pub type MarketOfferRow<'a> = (u32, u16, u16, u64, &'a str);

/// Serialize `sendMarketBrowseItem` (opcode `0xF9`, request `0x03`
/// `MARKETREQUEST_ITEM`).
///
/// Mirrors C++ `ProtocolGame::sendMarketBrowseItem` in
/// `forgottenserver/src/protocolgame.cpp` lines 2167–2200.
///
/// The C++ source emits a `sendStoreBalance` frame ahead of the 0xF9
/// payload — that frame is the caller's concern; this function only writes
/// the 0xF9 packet.
///
/// Caller resolves `client_id` from `Item::items[itemId].clientId` and
/// `has_classification` from `Item::items[itemId].classification > 0`.
///
/// Wire layout (all little-endian):
/// * opcode `0xF9` (u8)
/// * request type `0x03` `MARKETREQUEST_ITEM` (u8)
/// * `client_id` (u16)
/// * `0x00` (u8) — *only* when `has_classification` is true (item tier)
/// * `buy_offers.len()` (u32) and, per row: timestamp u32 | counter u16 |
///   amount u16 | price u64 | player_name (u16 length + bytes)
/// * `sell_offers.len()` (u32) and, per row: same shape as buy rows
pub fn serialize_market_browse_item(
    msg: &mut OutputMessage,
    client_id: u16,
    has_classification: bool,
    buy_offers: &[MarketOfferRow],
    sell_offers: &[MarketOfferRow],
) {
    msg.add_u8(0xF9);
    msg.add_u8(0x03); // MARKETREQUEST_ITEM
    msg.add_u16(client_id);
    if has_classification {
        msg.add_u8(0x00); // item tier
    }
    msg.add_u32(buy_offers.len() as u32);
    for &(timestamp, counter, amount, price, player_name) in buy_offers {
        msg.add_u32(timestamp);
        msg.add_u16(counter);
        msg.add_u16(amount);
        msg.add_u64(price);
        msg.add_string(player_name);
    }
    msg.add_u32(sell_offers.len() as u32);
    for &(timestamp, counter, amount, price, player_name) in sell_offers {
        msg.add_u32(timestamp);
        msg.add_u16(counter);
        msg.add_u16(amount);
        msg.add_u64(price);
        msg.add_string(player_name);
    }
}

/// Flat representation of `MarketOfferEx` for use with
/// [`serialize_market_accept_offer`] and [`serialize_market_cancel_offer`].
///
/// * `is_buy` — `offer.type == MARKETACTION_BUY` (`0x00 == MARKETACTION_BUY`,
///   `0x01 == MARKETACTION_SELL`).
/// * `client_id` — `Item::items[offer.itemId].clientId`.
/// * `has_classification` — `Item::items[offer.itemId].classification > 0`.
pub struct MarketOfferExFlat<'a> {
    pub is_buy: bool,
    pub client_id: u16,
    pub has_classification: bool,
    pub timestamp: u32,
    pub counter: u16,
    pub amount: u16,
    pub price: u64,
    pub player_name: &'a str,
}

/// Serialize `sendMarketAcceptOffer` (opcode `0xF9`, request `0x03`
/// `MARKETREQUEST_ITEM`).
///
/// Mirrors C++ `ProtocolGame::sendMarketAcceptOffer` in
/// `forgottenserver/src/protocolgame.cpp` lines 2202–2231.
///
/// The wire layout is identical to a "single-offer" browse-item frame,
/// with one of the two lists empty (`0x00000000`) depending on the offer
/// direction.
///
/// Wire layout (all little-endian):
/// * opcode `0xF9` (u8)
/// * request type `0x03` `MARKETREQUEST_ITEM` (u8)
/// * `client_id` (u16)
/// * `0x00` (u8) — *only* when `has_classification` is true (item tier)
/// * if `is_buy`:
///   * `0x00000001` (u32 buy count)
///   * timestamp u32 | counter u16 | amount u16 | price u64 |
///     player_name (u16 length + bytes)
///   * `0x00000000` (u32 sell count)
/// * else (is_sell):
///   * `0x00000000` (u32 buy count)
///   * `0x00000001` (u32 sell count)
///   * timestamp u32 | counter u16 | amount u16 | price u64 |
///     player_name (u16 length + bytes)
pub fn serialize_market_accept_offer(msg: &mut OutputMessage, offer: &MarketOfferExFlat) {
    msg.add_u8(0xF9);
    msg.add_u8(0x03); // MARKETREQUEST_ITEM
    msg.add_u16(offer.client_id);
    if offer.has_classification {
        msg.add_u8(0x00);
    }

    if offer.is_buy {
        msg.add_u32(0x01);
        msg.add_u32(offer.timestamp);
        msg.add_u16(offer.counter);
        msg.add_u16(offer.amount);
        msg.add_u64(offer.price);
        msg.add_string(offer.player_name);
        msg.add_u32(0x00);
    } else {
        msg.add_u32(0x00);
        msg.add_u32(0x01);
        msg.add_u32(offer.timestamp);
        msg.add_u16(offer.counter);
        msg.add_u16(offer.amount);
        msg.add_u64(offer.price);
        msg.add_string(offer.player_name);
    }
}

/// One row in a `sendMarketBrowseOwnOffers` list — flat data.
///
/// Tuple order matches the wire layout:
/// `(timestamp, counter, client_id, has_classification, amount, price)`.
pub type MarketOwnOfferRow = (u32, u16, u16, bool, u16, u64);

/// Serialize `sendMarketBrowseOwnOffers` (opcode `0xF9`, request `0x02`
/// `MARKETREQUEST_OWN_OFFERS`).
///
/// Mirrors C++ `ProtocolGame::sendMarketBrowseOwnOffers` in
/// `forgottenserver/src/protocolgame.cpp` lines 2233–2264.
///
/// Caller resolves `client_id` from `Item::items[offer.itemId].clientId`
/// and `has_classification` from `Item::items[offer.itemId].classification > 0`.
///
/// Wire layout (all little-endian):
/// * opcode `0xF9` (u8)
/// * request type `0x02` `MARKETREQUEST_OWN_OFFERS` (u8)
/// * `buy_offers.len()` (u32) and, per row:
///   * timestamp u32 | counter u16 | client_id u16
///   * `0x00` u8 — *only* when `has_classification` is true
///   * amount u16 | price u64
/// * `sell_offers.len()` (u32) and per-row: same shape
pub fn serialize_market_browse_own_offers(
    msg: &mut OutputMessage,
    buy_offers: &[MarketOwnOfferRow],
    sell_offers: &[MarketOwnOfferRow],
) {
    msg.add_u8(0xF9);
    msg.add_u8(0x02); // MARKETREQUEST_OWN_OFFERS

    msg.add_u32(buy_offers.len() as u32);
    for &(timestamp, counter, client_id, has_classification, amount, price) in buy_offers {
        msg.add_u32(timestamp);
        msg.add_u16(counter);
        msg.add_u16(client_id);
        if has_classification {
            msg.add_u8(0x00);
        }
        msg.add_u16(amount);
        msg.add_u64(price);
    }

    msg.add_u32(sell_offers.len() as u32);
    for &(timestamp, counter, client_id, has_classification, amount, price) in sell_offers {
        msg.add_u32(timestamp);
        msg.add_u16(counter);
        msg.add_u16(client_id);
        if has_classification {
            msg.add_u8(0x00);
        }
        msg.add_u16(amount);
        msg.add_u64(price);
    }
}

/// Serialize `sendMarketCancelOffer` (opcode `0xF9`, request `0x02`
/// `MARKETREQUEST_OWN_OFFERS`).
///
/// Mirrors C++ `ProtocolGame::sendMarketCancelOffer` in
/// `forgottenserver/src/protocolgame.cpp` lines 2266–2297.
///
/// Wire layout (all little-endian):
/// * opcode `0xF9` (u8)
/// * request type `0x02` `MARKETREQUEST_OWN_OFFERS` (u8)
/// * if `is_buy`:
///   * `0x00000001` (u32 buy count)
///   * timestamp u32 | counter u16 | client_id u16
///   * `0x00` u8 — *only* when `has_classification` is true
///   * amount u16 | price u64
///   * `0x00000000` (u32 sell count)
/// * else (is_sell):
///   * `0x00000000` (u32 buy count)
///   * `0x00000001` (u32 sell count)
///   * timestamp u32 | counter u16 | client_id u16
///   * `0x00` u8 — *only* when `has_classification` is true
///   * amount u16 | price u64
pub fn serialize_market_cancel_offer(msg: &mut OutputMessage, offer: &MarketOfferExFlat) {
    msg.add_u8(0xF9);
    msg.add_u8(0x02); // MARKETREQUEST_OWN_OFFERS

    if offer.is_buy {
        msg.add_u32(0x01);
        msg.add_u32(offer.timestamp);
        msg.add_u16(offer.counter);
        msg.add_u16(offer.client_id);
        if offer.has_classification {
            msg.add_u8(0x00);
        }
        msg.add_u16(offer.amount);
        msg.add_u64(offer.price);
        msg.add_u32(0x00);
    } else {
        msg.add_u32(0x00);
        msg.add_u32(0x01);
        msg.add_u32(offer.timestamp);
        msg.add_u16(offer.counter);
        msg.add_u16(offer.client_id);
        if offer.has_classification {
            msg.add_u8(0x00);
        }
        msg.add_u16(offer.amount);
        msg.add_u64(offer.price);
    }
}

/// One row in a `sendMarketBrowseOwnHistory` list — flat data resolved by
/// the caller from `HistoryMarketOffer`.
///
/// Tuple order matches the wire layout:
/// `(timestamp, client_id, has_classification, amount, price, state)`.
///
/// Note: per-timestamp counter (u16) is generated internally by
/// [`serialize_market_browse_own_history`] (mirrors the C++ `counterMap`).
pub type MarketHistoryRow = (u32, u16, bool, u16, u64, u8);

fn append_market_history_rows(msg: &mut OutputMessage, offers: &[MarketHistoryRow], max: usize) {
    let to_send = offers.len().min(max);
    msg.add_u32(to_send as u32);

    // Mirrors C++ counterMap (`std::map<uint32_t, uint16_t>`): per-timestamp
    // monotonically-increasing u16 starting at 0.  Distinct timestamps each
    // restart at 0; same-timestamp rows increment.  We do this in O(N²)
    // here (N ≤ 1620) by re-counting prior matches — the C++ code uses a
    // post-increment on `counterMap[ts]++`, which is functionally identical.
    for i in 0..to_send {
        let (ts, client_id, has_classification, amount, price, state) = offers[i];
        let mut counter: u16 = 0;
        for prior in offers.iter().take(i) {
            if prior.0 == ts {
                counter = counter.saturating_add(1);
            }
        }
        msg.add_u32(ts);
        msg.add_u16(counter);
        msg.add_u16(client_id);
        if has_classification {
            msg.add_u8(0x00);
        }
        msg.add_u16(amount);
        msg.add_u64(price);
        msg.add_u8(state);
    }
}

/// Serialize `sendMarketBrowseOwnHistory` (opcode `0xF9`, request `0x01`
/// `MARKETREQUEST_OWN_HISTORY`).
///
/// Mirrors C++ `ProtocolGame::sendMarketBrowseOwnHistory` in
/// `forgottenserver/src/protocolgame.cpp` lines 2299–2343.
///
/// Caller resolves `client_id` from `Item::items[offer.itemId].clientId`
/// and `has_classification` from `Item::items[offer.itemId].classification > 0`.
///
/// The C++ source caps each list at
/// `min(list.size(), 810 + max(0, 810 - other_list.size()))` so a total of
/// at most 1620 rows are emitted across both lists.
///
/// Wire layout (all little-endian):
/// * opcode `0xF9` (u8)
/// * request type `0x01` `MARKETREQUEST_OWN_HISTORY` (u8)
/// * `buy_to_send` (u32) and, per row:
///   * timestamp u32 | counter u16 (per-timestamp index) | client_id u16
///   * `0x00` u8 — *only* when `has_classification` is true
///   * amount u16 | price u64 | state u8
/// * `sell_to_send` (u32) and per-row: same shape
pub fn serialize_market_browse_own_history(
    msg: &mut OutputMessage,
    buy_offers: &[MarketHistoryRow],
    sell_offers: &[MarketHistoryRow],
) {
    msg.add_u8(0xF9);
    msg.add_u8(0x01); // MARKETREQUEST_OWN_HISTORY

    // C++ cap formula: min(self.size(), 810 + max(0, 810 - other.size())).
    let buy_max = 810usize.saturating_add(810usize.saturating_sub(sell_offers.len()));
    let sell_max = 810usize.saturating_add(810usize.saturating_sub(buy_offers.len()));

    append_market_history_rows(msg, buy_offers, buy_max);
    append_market_history_rows(msg, sell_offers, sell_max);
}

// ---------------------------------------------------------------------------
// Trade packets (Session 8 — wire-trade cluster)
// ---------------------------------------------------------------------------

/// Serialize `sendTradeItemRequest` (opcode `0x7D` own offer / `0x7E`
/// counterparty offer).
///
/// Mirrors C++ `ProtocolGame::sendTradeItemRequest` in
/// `forgottenserver/src/protocolgame.cpp` lines 2345–2382.  The C++ source
/// walks the trade container's nested `Container`s breadth-first and emits
/// each contained item via per-instance `addItem(const Item*)`.  For a
/// non-container item it sends a single-row list (`0x01` + one item).
///
/// This Rust port keeps `protocolgame.rs` free of container/item domain
/// types — the caller is responsible for flattening the trade item (and
/// any nested children) into the `items` slice using the same BFS order
/// as C++.  Each entry is a `(count, ItemTypeMeta)` pair forwarded to
/// `add_item_payload` (matches `addItem(id, count)` wire layout; per-
/// instance overrides like charges / duration are dropped, as no trade
/// item exposes them on the wire).
///
/// Wire layout (all little-endian):
/// * opcode `0x7D` if `ack` (own offer) else `0x7E` (counterparty offer)
/// * `trader_name` (u16 length + bytes, via `add_string`)
/// * `items_to_send` (u8 — `min(items.len(), u8::MAX)`)
/// * per row: item payload via `add_item_payload(count, meta)`
pub fn serialize_trade_item_request(
    msg: &mut OutputMessage,
    trader_name: &str,
    ack: bool,
    items: &[(u8, ItemTypeMeta)],
) {
    msg.add_u8(if ack { 0x7D } else { 0x7E });
    msg.add_string(trader_name);

    let items_to_send = items.len().min(u8::MAX as usize) as u8;
    msg.add_u8(items_to_send);
    for (count, meta) in items.iter().take(items_to_send as usize) {
        append_item_payload(msg, *count, *meta);
    }
}

/// Serialize `sendCloseTrade` (opcode `0x7F` own / `0x80` counterparty).
///
/// Mirrors C++ `ProtocolGame::sendCloseTrade` in
/// `forgottenserver/src/protocolgame.cpp` lines 2384–2389.  The C++ source
/// hard-codes `0x7F` for the local close; the counterparty close uses
/// `0x80` (called from the trade-controller path when the partner
/// abandons).  The `ack` flag here selects between the two so both
/// branches stay in one wire-parity-tested function.
///
/// Wire layout:
/// * opcode `0x7F` if `ack` (own close) else `0x80` (counterparty close)
///   — no payload
pub fn serialize_close_trade(msg: &mut OutputMessage, ack: bool) {
    msg.add_u8(if ack { 0x7F } else { 0x80 });
}

// ---------------------------------------------------------------------------
// Chat / channel dialog packets (Session 5)
// ---------------------------------------------------------------------------

/// Serialize a `sendChannelsDialog` packet (opcode `0xAB`).
///
/// Mirrors C++ `ProtocolGame::sendChannelsDialog` in
/// `forgottenserver/src/protocolgame.cpp` lines 1834–1847.  The C++ source
/// builds the list via `g_chat->getChannelList(*player)`; this port takes
/// the already-resolved `(channel_id, name)` rows so the network crate
/// stays free of game / chat dependencies.
///
/// Wire layout:
/// * opcode `0xAB` (u8)
/// * `channels.len()` (u8 — channel count)
/// * per channel:
///   * `channel_id` (u16)
///   * `name` (length-prefixed string)
pub fn serialize_channels_dialog(channels: &[(u16, &str)]) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xAB);
    out.add_u8(channels.len() as u8);
    for &(channel_id, name) in channels {
        out.add_u16(channel_id);
        out.add_string(name);
    }
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a `sendChannel` packet (opcode `0xAC`).
///
/// Mirrors C++ `ProtocolGame::sendChannel` in
/// `forgottenserver/src/protocolgame.cpp` lines 1849–1876.  Callers pass
/// already-resolved name lists for channel users and invited users; an
/// empty slice yields the C++ `nullptr` branch (zero count, no name list).
///
/// Wire layout:
/// * opcode `0xAC` (u8)
/// * `channel_id` (u16)
/// * `channel_name` (length-prefixed string)
/// * `channel_users.len()` (u16)
/// * `channel_users.len()` × (length-prefixed name)
/// * `invited_users.len()` (u16)
/// * `invited_users.len()` × (length-prefixed name)
pub fn serialize_channel(
    channel_id: u16,
    channel_name: &str,
    channel_users: &[&str],
    invited_users: &[&str],
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xAC);
    out.add_u16(channel_id);
    out.add_string(channel_name);
    out.add_u16(channel_users.len() as u16);
    for name in channel_users {
        out.add_string(name);
    }
    out.add_u16(invited_users.len() as u16);
    for name in invited_users {
        out.add_string(name);
    }
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a `sendChannelMessage` packet (opcode `0xAA`).
///
/// Mirrors C++ `ProtocolGame::sendChannelMessage` in
/// `forgottenserver/src/protocolgame.cpp` lines 1878–1890.  C++ writes a
/// zero `statementId` and zero `level` for system / channel info messages.
///
/// Wire layout:
/// * opcode `0xAA` (u8)
/// * `0x00000000` (u32 — statement id placeholder)
/// * `author` (length-prefixed string)
/// * `0x0000` (u16 — speaker level placeholder)
/// * `speak_class` (u8 — `SpeakClasses` enum)
/// * `channel_id` (u16)
/// * `text` (length-prefixed string)
pub fn serialize_channel_message(
    author: &str,
    text: &str,
    speak_class: u8,
    channel_id: u16,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xAA);
    out.add_u32(0x0000_0000);
    out.add_string(author);
    out.add_u16(0x0000);
    out.add_u8(speak_class);
    out.add_u16(channel_id);
    out.add_string(text);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a `sendToChannel` packet (opcode `0xAA`).
///
/// Mirrors C++ `ProtocolGame::sendToChannel` in
/// `forgottenserver/src/protocolgame.cpp` lines 2452–2479.
///
/// The C++ source uses a `static uint32_t statementId = 0; ++statementId`
/// counter local to the function.  This port takes the `stmt_id` as an
/// argument so the caller owns the monotonic counter and tests can
/// produce deterministic bytes.  Passing an empty `creature_name` slice
/// triggers the C++ `creature == nullptr` branch (u32 zero + u8 zero
/// "traded" suffix, no level, no name).
///
/// Wire layout:
/// * opcode `0xAA` (u8)
/// * `stmt_id` (u32)
/// * if `creature_name.is_empty()`:
///   * `0x00000000` (u32 — placeholder)
///   * `0x00` (u8 — "(Traded)" suffix)
/// * else:
///   * `creature_name` (length-prefixed string)
///   * `0x00` (u8 — "(Traded)" suffix)
///   * `level` (u16 — `0` for non-players)
/// * `speak_class` (u8)
/// * `channel_id` (u16)
/// * `text` (length-prefixed string)
pub fn serialize_to_channel(
    stmt_id: u32,
    creature_name: &str,
    level: u16,
    speak_class: u8,
    channel_id: u16,
    text: &str,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xAA);
    out.add_u32(stmt_id);
    if creature_name.is_empty() {
        out.add_u32(0x0000_0000);
        out.add_u8(0x00);
    } else {
        out.add_string(creature_name);
        out.add_u8(0x00);
        out.add_u16(level);
    }
    out.add_u8(speak_class);
    out.add_u16(channel_id);
    out.add_string(text);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a `sendPrivateMessage` packet (opcode `0xAA`).
///
/// Mirrors C++ `ProtocolGame::sendPrivateMessage` in
/// `forgottenserver/src/protocolgame.cpp` lines 2481–2498.  As with
/// `serialize_to_channel`, the caller owns the monotonic statement-id
/// counter to keep this function pure.  Passing an empty `speaker_name`
/// triggers the C++ `speaker == nullptr` branch.
///
/// Wire layout:
/// * opcode `0xAA` (u8)
/// * `stmt_id` (u32)
/// * if `speaker_name.is_empty()`:
///   * `0x00000000` (u32 — placeholder)
///   * `0x00` (u8 — "(Traded)" suffix)
/// * else:
///   * `speaker_name` (length-prefixed string)
///   * `0x00` (u8 — "(Traded)" suffix)
///   * `speaker_level` (u16)
/// * `speak_class` (u8)
/// * `text` (length-prefixed string)
pub fn serialize_private_message(
    stmt_id: u32,
    speaker_name: &str,
    speaker_level: u16,
    speak_class: u8,
    text: &str,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xAA);
    out.add_u32(stmt_id);
    if speaker_name.is_empty() {
        out.add_u32(0x0000_0000);
        out.add_u8(0x00);
    } else {
        out.add_string(speaker_name);
        out.add_u8(0x00);
        out.add_u16(speaker_level);
    }
    out.add_u8(speak_class);
    out.add_string(text);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

// ---------------------------------------------------------------------------
// sendTextMessage (Session 5)
// ---------------------------------------------------------------------------

/// `MessageClasses` enum values consumed by `serialize_text_message`.
///
/// Listed here for cross-reference; the wire opcode takes the raw `u8`
/// value, but the per-class branch selection inside `serialize_text_message`
/// uses these constants.  Source: `forgottenserver/src/const.h` lines
/// 278–293.
pub mod text_message_class {
    pub const MESSAGE_DAMAGE_DEALT: u8 = 23;
    pub const MESSAGE_DAMAGE_RECEIVED: u8 = 24;
    pub const MESSAGE_HEALED: u8 = 25;
    pub const MESSAGE_EXPERIENCE: u8 = 26;
    pub const MESSAGE_DAMAGE_OTHERS: u8 = 27;
    pub const MESSAGE_HEALED_OTHERS: u8 = 28;
    pub const MESSAGE_EXPERIENCE_OTHERS: u8 = 29;
    pub const MESSAGE_GUILD: u8 = 33;
    pub const MESSAGE_PARTY_MANAGEMENT: u8 = 34;
    pub const MESSAGE_PARTY: u8 = 35;
}

/// Position triple `(x, y, z)` used by `serialize_text_message`.
pub type TextMessagePosition = (u16, u16, u8);

/// Serialize a `sendTextMessage` packet (opcode `0xB4`).
///
/// Mirrors C++ `ProtocolGame::sendTextMessage` in
/// `forgottenserver/src/protocolgame.cpp` lines 1776–1812.
///
/// The C++ source switches over `message.type`:
/// * `MESSAGE_DAMAGE_DEALT/RECEIVED/OTHERS`: writes position + primary
///   (u32 value + u8 color) + secondary (u32 value + u8 color).
/// * `MESSAGE_HEALED/HEALED_OTHERS/EXPERIENCE/EXPERIENCE_OTHERS`: writes
///   position + primary (u32 value + u8 color) only.
/// * `MESSAGE_GUILD/PARTY_MANAGEMENT/PARTY`: writes `channel_id` (u16,
///   carried in `secondary_value` low half by the caller).
/// * default: no extra payload.
/// * Then `text` is always appended (length-prefixed string).
///
/// Branch selection is driven by `message_class` so the caller passes a
/// single `u8` enum value matching the C++ `MessageClasses`.  The
/// optional position / primary / secondary fields are only consumed by
/// the relevant branch; pass `None` (or any sentinel) for unused fields.
/// `channel_id` is conveyed via the `secondary_value` field for the
/// `GUILD/PARTY*` branch (matching `message.channelId` in C++).
#[allow(clippy::too_many_arguments)]
pub fn serialize_text_message(
    message_class: u8,
    text: &str,
    position: Option<TextMessagePosition>,
    primary_color: Option<u8>,
    primary_value: Option<u32>,
    secondary_color: Option<u8>,
    secondary_value: Option<u32>,
    channel_id: Option<u16>,
) -> Vec<u8> {
    use text_message_class::*;
    let mut out = OutputMessage::new();
    out.add_u8(0xB4);
    out.add_u8(message_class);

    match message_class {
        MESSAGE_DAMAGE_DEALT | MESSAGE_DAMAGE_RECEIVED | MESSAGE_DAMAGE_OTHERS => {
            let (x, y, z) = position.unwrap_or((0, 0, 0));
            out.add_u16(x);
            out.add_u16(y);
            out.add_u8(z);
            out.add_u32(primary_value.unwrap_or(0));
            out.add_u8(primary_color.unwrap_or(0));
            out.add_u32(secondary_value.unwrap_or(0));
            out.add_u8(secondary_color.unwrap_or(0));
        }
        MESSAGE_HEALED | MESSAGE_HEALED_OTHERS | MESSAGE_EXPERIENCE | MESSAGE_EXPERIENCE_OTHERS => {
            let (x, y, z) = position.unwrap_or((0, 0, 0));
            out.add_u16(x);
            out.add_u16(y);
            out.add_u8(z);
            out.add_u32(primary_value.unwrap_or(0));
            out.add_u8(primary_color.unwrap_or(0));
        }
        MESSAGE_GUILD | MESSAGE_PARTY_MANAGEMENT | MESSAGE_PARTY => {
            out.add_u16(channel_id.unwrap_or(0));
        }
        _ => {}
    }

    out.add_string(text);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

// ---------------------------------------------------------------------------
// sendModalWindow (Session 5)
// ---------------------------------------------------------------------------

/// Serialize a `sendModalWindow` packet (opcode `0xFA`).
///
/// Mirrors C++ `ProtocolGame::sendModalWindow` in
/// `forgottenserver/src/protocolgame.cpp` lines 3351–3377.
///
/// The C++ `ModalWindow` struct (from `enums.h:616`) stores buttons and
/// choices as `std::list<std::pair<std::string, uint8_t>>` — i.e. each
/// pair is `(text, id)`.  This Rust signature follows the same shape:
/// `(text, id)`.
///
/// Wire layout:
/// * opcode `0xFA` (u8)
/// * `modal_id` (u32)
/// * `title` (length-prefixed string)
/// * `message` (length-prefixed string)
/// * `buttons.len()` (u8)
/// * `buttons.len()` × (length-prefixed `text`, u8 `id`)
/// * `choices.len()` (u8)
/// * `choices.len()` × (length-prefixed `text`, u8 `id`)
/// * `default_escape_button` (u8)
/// * `default_enter_button` (u8)
/// * `priority` (u8 — `0x01` if true, else `0x00`)
#[allow(clippy::too_many_arguments)]
pub fn serialize_modal_window(
    modal_id: u32,
    title: &str,
    message: &str,
    buttons: &[(&str, u8)],
    choices: &[(&str, u8)],
    default_enter_button: u8,
    default_escape_button: u8,
    priority: bool,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xFA);
    out.add_u32(modal_id);
    out.add_string(title);
    out.add_string(message);

    out.add_u8(buttons.len() as u8);
    for &(btn_text, btn_id) in buttons {
        out.add_string(btn_text);
        out.add_u8(btn_id);
    }

    out.add_u8(choices.len() as u8);
    for &(ch_text, ch_id) in choices {
        out.add_string(ch_text);
        out.add_u8(ch_id);
    }

    out.add_u8(default_escape_button);
    out.add_u8(default_enter_button);
    out.add_u8(if priority { 0x01 } else { 0x00 });

    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

// ---------------------------------------------------------------------------
// sendTextWindow / sendHouseWindow (Session 5)
// ---------------------------------------------------------------------------

/// Serialize a writable text-window packet (opcode `0x96`).
///
/// Mirrors C++ `ProtocolGame::sendTextWindow(uint32_t, Item*, uint16_t, bool)`
/// in `forgottenserver/src/protocolgame.cpp` lines 2936–2969.
///
/// The C++ source pulls the item meta + `getText()` / `getWriter()` /
/// `getDate()` from the live `Item*`; this port takes the resolved values
/// as plain arguments so the network crate has no `Item` dependency.
/// `date` should be the already-formatted `formatDateShort(...)` string;
/// pass `None` when the C++ source writes the `0x0000` placeholder
/// (i.e. `writtenDate == 0`).  `writer` follows the same rule (`None`
/// means C++ wrote the empty `0x0000` placeholder).
///
/// Wire layout:
/// * opcode `0x96` (u8)
/// * `window_text_id` (u32)
/// * `addItem(client_id, count=1)` via `add_item_payload` (uses `item_meta`)
/// * if `can_write`:
///   * `maxlen` (u16)
///   * `text` (length-prefixed string)
/// * else:
///   * `text.len() as u16`
///   * `text` (length-prefixed string)
/// * writer block:
///   * if `writer.is_some()`: length-prefixed string
///   * else: `0x0000` (u16)
/// * `0x00` (u8 — "(traded)" suffix)
/// * date block:
///   * if `date.is_some()`: length-prefixed string
///   * else: `0x0000` (u16)
#[allow(clippy::too_many_arguments)]
pub fn serialize_text_window_writable(
    window_text_id: u32,
    item_meta: ItemTypeMeta,
    maxlen: u16,
    text: &str,
    can_write: bool,
    writer: Option<&str>,
    date: Option<&str>,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x96);
    out.add_u32(window_text_id);
    append_item_payload(&mut out, 1, item_meta);

    if can_write {
        out.add_u16(maxlen);
        out.add_string(text);
    } else {
        out.add_u16(text.len() as u16);
        out.add_string(text);
    }

    match writer {
        Some(w) => out.add_string(w),
        None => out.add_u16(0x0000),
    }

    out.add_u8(0x00); // "(traded)" suffix

    match date {
        Some(d) => out.add_string(d),
        None => out.add_u16(0x0000),
    }

    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a read-only text-window packet (opcode `0x96`).
///
/// Mirrors the second `ProtocolGame::sendTextWindow(uint32_t, uint32_t,
/// const std::string&)` overload in
/// `forgottenserver/src/protocolgame.cpp` lines 2971–2983.  This overload
/// is used to show short server-defined messages and writes a fixed empty
/// writer / "(traded)" / empty date block.  Item id is written via
/// `addItem(uint16_t id, uint8_t count=1)` — equivalent to a plain
/// `add_u16` of the client id since this overload never uses sub-type
/// bytes (the item is fabricated, not stackable/podium/etc).
///
/// Wire layout:
/// * opcode `0x96` (u8)
/// * `window_text_id` (u32)
/// * `item_client_id` (u16 — from `addItem(id, 1)` shortcut)
/// * `text.len() as u16` (u16)
/// * `text` (length-prefixed string)
/// * `0x0000` (u16 — empty writer name)
/// * `0x00` (u8 — "(traded)" suffix)
/// * `0x0000` (u16 — empty date)
pub fn serialize_text_window_readonly(
    window_text_id: u32,
    item_client_id: u16,
    text: &str,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x96);
    out.add_u32(window_text_id);
    out.add_u16(item_client_id);
    out.add_u16(text.len() as u16);
    out.add_string(text);
    out.add_u16(0x0000);
    out.add_u8(0x00);
    out.add_u16(0x0000);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a `sendHouseWindow` packet (opcode `0x97`).
///
/// Mirrors C++ `ProtocolGame::sendHouseWindow` in
/// `forgottenserver/src/protocolgame.cpp` lines 2985–2993.
///
/// Wire layout:
/// * opcode `0x97` (u8)
/// * `0x00` (u8 — window type / unknown, always zero in C++)
/// * `window_text_id` (u32)
/// * `text` (length-prefixed string)
pub fn serialize_house_window(window_text_id: u32, text: &str) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x97);
    out.add_u8(0x00);
    out.add_u32(window_text_id);
    out.add_string(text);
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

// ---------------------------------------------------------------------------
// sendOutfitWindow / sendPodiumWindow (Session 5)
// ---------------------------------------------------------------------------

/// One available-outfit row written by `serialize_outfit_window`.
///
/// Tuple shape matches the per-entry C++ wire layout in
/// `forgottenserver/src/protocolgame.cpp` lines 3076–3082:
/// `(look_type, name, addons, mode)`.
///
/// `mode` is the trailing `u8` byte: `0x00` available, `0x01` store
/// (which would also require a `u32` `store_offer_id` immediately after —
/// callers that need that branch must extend this type), `0x02` golden
/// outfit tooltip.
pub type OutfitWindowRow<'a> = (u16, &'a str, u8, u8);

/// One available-mount row written by `serialize_outfit_window`.
///
/// Tuple shape matches the C++ wire layout in
/// `forgottenserver/src/protocolgame.cpp` lines 3092–3095:
/// `(client_id, name, mode)`.
pub type OutfitWindowMountRow<'a> = (u16, &'a str, u8);

/// Serialize a `sendOutfitWindow` packet (opcode `0xC8`).
///
/// Mirrors C++ `ProtocolGame::sendOutfitWindow` in
/// `forgottenserver/src/protocolgame.cpp` lines 3019–3108.
///
/// The C++ source resolves the outfit list, current mount, "Gamemaster"
/// outfit (only for access players), mounts the player owns, and
/// `wasMounted` / `randomizeMount` from the `Player*`.  This port takes
/// all of that as plain arguments so the network crate stays free of
/// player / mount / outfit dependencies.
///
/// Intentional C++ simplifications (caller-resolved):
/// * Caller must already have applied the C++ `outfits.size() == 0`
///   early-return.
/// * Caller must already have applied the C++ "if `currentOutfit.lookType
///   == 0`, use `outfits.front().lookType`" fallback.
/// * Caller must already have applied the C++ "if `currentMount != null`,
///   set `currentOutfit.lookMount = currentMount->clientId`" override.
/// * Caller must already have added the "Gamemaster" outfit row for
///   access players.
///
/// Wire layout:
/// * opcode `0xC8` (u8)
/// * outfit block via [`append_outfit`] (current outfit)
/// * if `current_outfit.look_mount == 0`:
///   * `current_outfit.look_mount_head` (u8)
///   * `current_outfit.look_mount_body` (u8)
///   * `current_outfit.look_mount_legs` (u8)
///   * `current_outfit.look_mount_feet` (u8)
/// * `0x0000` (u16 — current familiar look type, unused)
/// * `available_outfits.len()` (u16)
/// * for each: `look_type` (u16), `name` (str), `addons` (u8), `mode` (u8)
/// * `mounts.len()` (u16)
/// * for each: `client_id` (u16), `name` (str), `mode` (u8)
/// * `0x0000` (u16 — familiars list size, unused)
/// * `0x00` (u8 — try-outfit mode)
/// * `mounted` (u8 — `0x01` or `0x00`)
/// * `randomize_mount` (u8 — `0x01` or `0x00`)
pub fn serialize_outfit_window(
    current_outfit: OutfitDescriptor,
    available_outfits: &[OutfitWindowRow<'_>],
    mounts: &[OutfitWindowMountRow<'_>],
    mounted: bool,
    randomize_mount: bool,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xC8);
    append_outfit(&mut out, current_outfit);

    if current_outfit.look_mount == 0 {
        out.add_u8(current_outfit.look_mount_head);
        out.add_u8(current_outfit.look_mount_body);
        out.add_u8(current_outfit.look_mount_legs);
        out.add_u8(current_outfit.look_mount_feet);
    }

    out.add_u16(0x0000); // current familiar look type

    out.add_u16(available_outfits.len() as u16);
    for &(look_type, name, addons, mode) in available_outfits {
        out.add_u16(look_type);
        out.add_string(name);
        out.add_u8(addons);
        out.add_u8(mode);
    }

    out.add_u16(mounts.len() as u16);
    for &(client_id, name, mode) in mounts {
        out.add_u16(client_id);
        out.add_string(name);
        out.add_u8(mode);
    }

    out.add_u16(0x0000); // familiars list size
    out.add_u8(0x00); // try-outfit mode
    out.add_u8(if mounted { 0x01 } else { 0x00 });
    out.add_u8(if randomize_mount { 0x01 } else { 0x00 });

    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a `sendPodiumWindow` packet (opcode `0xC8`).
///
/// Mirrors C++ `ProtocolGame::sendPodiumWindow` in
/// `forgottenserver/src/protocolgame.cpp` lines 3110–3244.
///
/// The C++ source resolves the podium outfit / player outfit fallback,
/// outfit + mount + addon visibility from `Podium*` flags, and tile
/// stackpos lookup from the live `Item*`.  This port takes the resolved
/// values as plain arguments.
///
/// Intentional C++ simplifications (caller-resolved):
/// * `podium_outfit` already has the "copy player outfit / mount when
///   lookType / lookMount == 0" fallback applied.
/// * `podium_outfit.look_type` already passed the C++ `canWear` check
///   (using the first available outfit when not unlocked).
/// * `available_outfits` already includes the optional Gamemaster row for
///   access players (same convention as `serialize_outfit_window`).
/// * `has_mount_visible` reflects the C++ ternary
///   `(isEmpty && playerOutfit.lookMount != 0) || podium->hasFlag(PODIUM_SHOW_MOUNT)`.
/// * `position` / `stackpos` / `item_client_id` come from the live item.
///
/// Wire layout:
/// * opcode `0xC8` (u8)
/// * current outfit (u16 look_type + 5 × u8 + u16 look_mount + 4 × u8) —
///   written inline (this is *not* the `append_outfit` shape because the
///   podium always writes the 4 mount-color bytes even when
///   `look_mount != 0`).
/// * `0x0000` (u16 — current familiar)
/// * `available_outfits.len()` (u16)
/// * for each: `look_type` (u16), `name` (str), `addons` (u8), `mode` (u8)
/// * `mounts.len()` (u16)
/// * for each: `client_id` (u16), `name` (str), `mode` (u8)
/// * `0x0000` (u16 — familiars list)
/// * `0x05` (u8 — "podium" set-outfit window mode)
/// * `has_mount_visible` (u8)
/// * `0x0000` (u16 — unknown)
/// * `position` (5 bytes)
/// * `item_client_id` (u16)
/// * `stackpos` (u8)
/// * `has_show_platform` (u8 — platform visible)
/// * `0x01` (u8 — outfit checkbox, ignored by client)
/// * `direction` (u8 — outfit direction)
#[allow(clippy::too_many_arguments)]
pub fn serialize_podium_window(
    pos_x: u16,
    pos_y: u16,
    pos_z: u8,
    stackpos: u8,
    item_client_id: u16,
    podium_outfit: OutfitDescriptor,
    direction: u8,
    available_outfits: &[OutfitWindowRow<'_>],
    mounts: &[OutfitWindowMountRow<'_>],
    has_mount_visible: bool,
    has_show_platform: bool,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xC8);

    // Current outfit — podium always writes the unconditional 6+5 layout
    // (look_type + 5 × u8 + look_mount + 4 × u8) regardless of look_type
    // or look_mount being zero.  See C++ lines 3193–3205.
    out.add_u16(podium_outfit.look_type);
    out.add_u8(podium_outfit.look_head);
    out.add_u8(podium_outfit.look_body);
    out.add_u8(podium_outfit.look_legs);
    out.add_u8(podium_outfit.look_feet);
    out.add_u8(podium_outfit.look_addons);

    out.add_u16(podium_outfit.look_mount);
    out.add_u8(podium_outfit.look_mount_head);
    out.add_u8(podium_outfit.look_mount_body);
    out.add_u8(podium_outfit.look_mount_legs);
    out.add_u8(podium_outfit.look_mount_feet);

    out.add_u16(0x0000); // familiar look type

    out.add_u16(available_outfits.len() as u16);
    for &(look_type, name, addons, mode) in available_outfits {
        out.add_u16(look_type);
        out.add_string(name);
        out.add_u8(addons);
        out.add_u8(mode);
    }

    out.add_u16(mounts.len() as u16);
    for &(client_id, name, mode) in mounts {
        out.add_u16(client_id);
        out.add_string(name);
        out.add_u8(mode);
    }

    out.add_u16(0x0000); // familiars list

    out.add_u8(0x05); // podium window mode
    out.add_u8(if has_mount_visible { 0x01 } else { 0x00 });
    out.add_u16(0x0000); // unknown

    out.add_u16(pos_x);
    out.add_u16(pos_y);
    out.add_u8(pos_z);
    out.add_u16(item_client_id);
    out.add_u8(stackpos);

    out.add_u8(if has_show_platform { 0x01 } else { 0x00 });
    out.add_u8(0x01); // outfit checkbox (ignored by client)
    out.add_u8(direction);

    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

// ---------------------------------------------------------------------------
// Combat / misc — Session 9 (wire-combat-misc cluster)
// ---------------------------------------------------------------------------

/// Serialize a distance-shoot packet.
///
/// Mirrors C++ `ProtocolGame::sendDistanceShoot` in
/// `forgottenserver/src/protocolgame.cpp` lines 2547–2558.
///
/// Wire layout (opcode `0x83`):
/// * opcode (u8)
/// * `from` position (u16 x, u16 y, u8 z)
/// * `MAGIC_EFFECTS_CREATE_DISTANCEEFFECT` byte (0x04)
/// * `effect_id` (u8)
/// * `delta_x = (to.x - from.x) as i8` cast to u8 (the C++ chain
///   `(uint8_t)(int8_t)(int32_t)to.x - (int32_t)from.x` preserves the
///   8-bit two's-complement bit pattern; the caller is responsible for
///   ensuring the delta fits in an `i8`)
/// * `delta_y = (to.y - from.y) as i8` cast to u8
/// * `MAGIC_EFFECTS_END_LOOP` byte (0x00)
pub fn serialize_distance_shoot(
    from_x: u16,
    from_y: u16,
    from_z: u8,
    to_x: u16,
    to_y: u16,
    effect_id: u8,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x83);
    out.add_u16(from_x);
    out.add_u16(from_y);
    out.add_u8(from_z);
    out.add_u8(0x04); // MAGIC_EFFECTS_CREATE_DISTANCEEFFECT
    out.add_u8(effect_id);
    let dx = (to_x as i32 - from_x as i32) as i8;
    let dy = (to_y as i32 - from_y as i32) as i8;
    out.add_u8(dx as u8);
    out.add_u8(dy as u8);
    out.add_u8(0x00); // MAGIC_EFFECTS_END_LOOP
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a player items list packet (opcode `0xF5`).
///
/// Mirrors C++ `ProtocolGame::sendItems` in
/// `forgottenserver/src/protocolgame.cpp` lines 2877–2900.
///
/// Wire layout:
/// * opcode `0xF5` (u8)
/// * `u16` (inventory.size() + 11) — total entry count
/// * 11 × default slot entries: `u16 slot_id (1..=11)` + `u8 0` +
///   `u16 1` (always 1)
/// * for each `(client_id, count)` pair in `inventory` (caller-provided
///   in the C++ source's `std::map<uint32_t, uint32_t>` ascending-key
///   iteration order): `u16 client_id` + `u8 0` + `u16 count`
///
/// Caller-side: the C++ source pulls `Item::items[item.first].clientId`
/// to translate item ids into client ids, and uses an ordered
/// `std::map<uint32_t, uint32_t>` iteration; both are pushed to the
/// caller.  Pass `inventory` already mapped + sorted.
pub fn serialize_items(inventory: &[(u16, u16)]) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xF5);
    let total = (inventory.len() as u16).saturating_add(11);
    out.add_u16(total);
    for i in 1u16..=11 {
        out.add_u16(i); // slotId
        out.add_u8(0); // always 0
        out.add_u16(1); // always 1
    }
    for &(client_id, count) in inventory {
        out.add_u16(client_id);
        out.add_u8(0);
        out.add_u16(count);
    }
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize an item-classes packet (opcode `0x86`).
///
/// Mirrors C++ `ProtocolGame::sendItemClasses` in
/// `forgottenserver/src/protocolgame.cpp` lines 3287–3314.  Both the
/// class count (4) and tier count (10) are compile-time constants in
/// the C++ source, as is the per-tier upgrade cost (10000).
///
/// Wire layout:
/// * opcode `0x86` (u8)
/// * `u8` `class_size = 4`
/// * for class in `0..4`:
///   - `u8 class_id = class + 1`
///   - `u8 tier_size = 10`
///   - for tier in `0..10`: `u8 tier_id = tier` + `u64 upgrade_cost = 10000`
/// * 11 padding bytes (`tier_size + 1 = 11` × `u8 0`)
pub fn serialize_item_classes() -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0x86);
    const CLASS_SIZE: u8 = 4;
    const TIERS_SIZE: u8 = 10;
    out.add_u8(CLASS_SIZE);
    for i in 0..CLASS_SIZE {
        out.add_u8(i + 1); // class id (1..=4)
        out.add_u8(TIERS_SIZE);
        for j in 0..TIERS_SIZE {
            out.add_u8(j); // tier id
            out.add_u64(10_000); // upgrade cost
        }
    }
    for _ in 0..(TIERS_SIZE + 1) {
        out.add_u8(0);
    }
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

/// Serialize a combat-analyzer packet (opcode `0xCC`).
///
/// Mirrors C++ `ProtocolGame::sendCombatAnalyzer` in
/// `forgottenserver/src/protocolgame.cpp` lines 2995–3017.
///
/// `impact_type` matches the C++ `DamageAnalyzerImpactType` enum:
/// * `0` (NONE) — opcode + impact_type + amount only (default branch)
/// * `1` (DEALT) — appends client_damage_type only
/// * `2` (RECEIVED) — appends client_damage_type + target string
///
/// Caller-side: the C++ source calls `getClientDamageType(combat_type)`
/// to map `CombatType_t` to the client's damage-type enum.  That
/// translation lives outside protocolgame, so pass the already-mapped
/// `client_damage_type` value here.
///
/// Wire layout:
/// * opcode `0xCC` (u8)
/// * `impact_type` (u8)
/// * `amount` (u32 — written via `u32` bit pattern of the input)
/// * if `impact_type == 2` (RECEIVED): `client_damage_type` (u8) +
///   `target` (u16-prefixed string)
/// * if `impact_type == 1` (DEALT): `client_damage_type` (u8) only
/// * else: nothing extra
pub fn serialize_combat_analyzer(
    impact_type: u8,
    amount: u32,
    client_damage_type: u8,
    target: &str,
) -> Vec<u8> {
    let mut out = OutputMessage::new();
    out.add_u8(0xCC);
    out.add_u8(impact_type);
    out.add_u32(amount);
    match impact_type {
        2 => {
            // RECEIVED
            out.add_u8(client_damage_type);
            out.add_string(target);
        }
        1 => {
            // DEALT
            out.add_u8(client_damage_type);
        }
        _ => {}
    }
    out.write_message_length();
    out.get_output_buffer()[2..].to_vec()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Write bytes into a NetworkMessage starting at the payload region and
    /// reset the read cursor ready for parsing.
    fn build_msg(bytes: &[u8]) -> NetworkMessage {
        let mut msg = NetworkMessage::new();
        msg.add_bytes(bytes);
        msg.set_buffer_position(0); // reset cursor to payload start
        msg
    }

    // -----------------------------------------------------------------------
    // parse_login_packet
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_login_packet() {
        // version=760, key=[1,2,3,4], account="acc", password="pass"
        let mut msg = NetworkMessage::new();
        msg.add_u16(760);
        msg.add_u32(1);
        msg.add_u32(2);
        msg.add_u32(3);
        msg.add_u32(4);
        msg.add_string("acc");
        msg.add_string("pass");
        msg.set_buffer_position(0);

        let pkt = parse_login_packet(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.client_version, 760);
        assert_eq!(pkt.xtea_key, [1, 2, 3, 4]);
        assert_eq!(pkt.account_name, "acc");
        assert_eq!(pkt.password, "pass");
    }

    // -----------------------------------------------------------------------
    // serialize_character_list
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_character_list_count_byte() {
        let chars = vec![CharacterEntry {
            name: "Hero".into(),
            world_name: "Antica".into(),
            world_ip: 0x01020304,
            world_port: 7171,
        }];
        let bytes = serialize_character_list(&chars);
        // First byte is the character count
        assert_eq!(bytes[0], 1);
    }

    #[test]
    fn test_serialize_character_list_round_trip() {
        let chars = vec![CharacterEntry {
            name: "Warrior".into(),
            world_name: "Testera".into(),
            world_ip: 0xC0A80001,
            world_port: 7171,
        }];
        let bytes = serialize_character_list(&chars);

        // Parse back manually via NetworkMessage
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);

        let count = msg.get_u8();
        assert_eq!(count, 1);
        let name = msg.get_string(0);
        let world_name = msg.get_string(0);
        let world_ip = msg.get_u32();
        let world_port = msg.get_u16();

        assert_eq!(name, "Warrior");
        assert_eq!(world_name, "Testera");
        assert_eq!(world_ip, 0xC0A80001);
        assert_eq!(world_port, 7171);
    }

    // -----------------------------------------------------------------------
    // parse_walk_packet
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_walk_packet() {
        let mut msg = build_msg(&[3]); // direction = 3
        let pkt = parse_walk_packet(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.direction, 3);
    }

    // -----------------------------------------------------------------------
    // parse_say_packet
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_say_packet() {
        let mut msg = NetworkMessage::new();
        msg.add_u8(1); // say_type = 1 (say)
        msg.add_string("Hello!");
        msg.set_buffer_position(0);

        let pkt = parse_say_packet(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.say_type, 1);
        assert_eq!(pkt.text, "Hello!");
    }

    // -----------------------------------------------------------------------
    // parse_use_item_packet
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_use_item_packet() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(100); // pos_x
        msg.add_u16(200); // pos_y
        msg.add_u8(7); // pos_z
        msg.add_u16(1234); // item_id
        msg.add_u8(0); // index
        msg.set_buffer_position(0);

        let pkt = parse_use_item_packet(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.pos_x, 100);
        assert_eq!(pkt.pos_y, 200);
        assert_eq!(pkt.pos_z, 7);
        assert_eq!(pkt.item_id, 1234);
        assert_eq!(pkt.index, 0);
    }

    // -----------------------------------------------------------------------
    // parse_look_at_packet
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_look_at_packet() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(50); // pos_x
        msg.add_u16(60); // pos_y
        msg.add_u8(2); // pos_z
        msg.add_u16(999); // item_id
        msg.add_u8(1); // stack_pos
        msg.set_buffer_position(0);

        let pkt = parse_look_at_packet(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.pos_x, 50);
        assert_eq!(pkt.pos_y, 60);
        assert_eq!(pkt.pos_z, 2);
        assert_eq!(pkt.item_id, 999);
        assert_eq!(pkt.stack_pos, 1);
    }

    // -----------------------------------------------------------------------
    // parse_trade_packet
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_trade_packet() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(10); // pos_x
        msg.add_u16(20); // pos_y
        msg.add_u8(1); // pos_z
        msg.add_u16(555); // item_id
        msg.add_u8(3); // count
        msg.set_buffer_position(0);

        let pkt = parse_trade_packet(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.pos_x, 10);
        assert_eq!(pkt.pos_y, 20);
        assert_eq!(pkt.pos_z, 1);
        assert_eq!(pkt.item_id, 555);
        assert_eq!(pkt.count, 3);
    }

    // -----------------------------------------------------------------------
    // parse_vip_packet
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_vip_packet() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(0xDEAD_BEEF);
        msg.set_buffer_position(0);

        let pkt = parse_vip_packet(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.player_id, 0xDEAD_BEEF);
    }

    // -----------------------------------------------------------------------
    // serialize_creature_move / serialize_add_creature
    //
    // Both signatures changed in Session 4 to take `&mut OutputMessage` +
    // plain-data args (was: `(&Packet) -> Vec<u8>`).  Byte-parity tests
    // live in `crates/network/tests/creature/mod.rs` alongside the rest
    // of the wire-creature cluster.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // parse_fight_modes
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_fight_modes() {
        let mut msg = build_msg(&[1, 0, 1]); // attack, stand, secure
        let pkt = parse_fight_modes(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.fight_mode, 1);
        assert_eq!(pkt.chase_mode, 0);
        assert_eq!(pkt.secure_mode, 1);
    }

    #[test]
    fn test_parse_fight_modes_defensive() {
        let mut msg = build_msg(&[3, 1, 0]);
        let pkt = parse_fight_modes(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.fight_mode, 3);
        assert_eq!(pkt.chase_mode, 1);
        assert_eq!(pkt.secure_mode, 0);
    }

    // -----------------------------------------------------------------------
    // parse_attack
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_attack() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(12345);
        msg.set_buffer_position(0);
        let pkt = parse_attack(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.creature_id, 12345);
    }

    // -----------------------------------------------------------------------
    // parse_follow
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_follow() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(99999);
        msg.set_buffer_position(0);
        let pkt = parse_follow(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.creature_id, 99999);
    }

    // -----------------------------------------------------------------------
    // parse_close_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_close_container() {
        let mut msg = build_msg(&[5]);
        let pkt = parse_close_container(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.container_id, 5);
    }

    // -----------------------------------------------------------------------
    // parse_up_arrow_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_up_arrow_container() {
        let mut msg = build_msg(&[3]);
        let pkt = parse_up_arrow_container(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.container_id, 3);
    }

    // -----------------------------------------------------------------------
    // parse_throw
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_throw() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(10); // from_x
        msg.add_u16(20); // from_y
        msg.add_u8(7); // from_z
        msg.add_u16(888); // sprite_id
        msg.add_u8(2); // from_stackpos
        msg.add_u16(11); // to_x
        msg.add_u16(20); // to_y
        msg.add_u8(7); // to_z
        msg.add_u8(1); // count
        msg.set_buffer_position(0);

        let pkt = parse_throw(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.from_x, 10);
        assert_eq!(pkt.from_y, 20);
        assert_eq!(pkt.from_z, 7);
        assert_eq!(pkt.sprite_id, 888);
        assert_eq!(pkt.from_stackpos, 2);
        assert_eq!(pkt.to_x, 11);
        assert_eq!(pkt.to_y, 20);
        assert_eq!(pkt.to_z, 7);
        assert_eq!(pkt.count, 1);
    }

    // -----------------------------------------------------------------------
    // parse_look_in_battle_list
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_look_in_battle_list() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(777);
        msg.set_buffer_position(0);
        let pkt = parse_look_in_battle_list(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.creature_id, 777);
    }

    // -----------------------------------------------------------------------
    // parse_invite_to_party
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_invite_to_party() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(42);
        msg.set_buffer_position(0);
        let pkt = parse_invite_to_party(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.target_id, 42);
    }

    // -----------------------------------------------------------------------
    // parse_join_party
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_join_party() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(43);
        msg.set_buffer_position(0);
        let pkt = parse_join_party(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.target_id, 43);
    }

    // -----------------------------------------------------------------------
    // parse_revoke_party_invite
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_revoke_party_invite() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(44);
        msg.set_buffer_position(0);
        let pkt = parse_revoke_party_invite(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.target_id, 44);
    }

    // -----------------------------------------------------------------------
    // parse_pass_party_leadership
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_pass_party_leadership() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(55);
        msg.set_buffer_position(0);
        let pkt = parse_pass_party_leadership(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.target_id, 55);
    }

    // -----------------------------------------------------------------------
    // parse_enable_shared_party_experience
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_enable_shared_party_experience_active() {
        let mut msg = build_msg(&[1]);
        let pkt = parse_enable_shared_party_experience(&mut msg).expect("parse should succeed");
        assert!(pkt.active);
    }

    #[test]
    fn test_parse_enable_shared_party_experience_inactive() {
        let mut msg = build_msg(&[0]);
        let pkt = parse_enable_shared_party_experience(&mut msg).expect("parse should succeed");
        assert!(!pkt.active);
    }

    // -----------------------------------------------------------------------
    // parse_modal_window_answer
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_modal_window_answer() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(100);
        msg.add_u8(2);
        msg.add_u8(3);
        msg.set_buffer_position(0);
        let pkt = parse_modal_window_answer(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.window_id, 100);
        assert_eq!(pkt.button, 2);
        assert_eq!(pkt.choice, 3);
    }

    // -----------------------------------------------------------------------
    // parse_browse_field
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_browse_field() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(500);
        msg.add_u16(600);
        msg.add_u8(7);
        msg.set_buffer_position(0);
        let pkt = parse_browse_field(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.pos_x, 500);
        assert_eq!(pkt.pos_y, 600);
        assert_eq!(pkt.pos_z, 7);
    }

    // -----------------------------------------------------------------------
    // parse_seek_in_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_seek_in_container() {
        let mut msg = NetworkMessage::new();
        msg.add_u8(2);
        msg.add_u16(15);
        msg.set_buffer_position(0);
        let pkt = parse_seek_in_container(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.container_id, 2);
        assert_eq!(pkt.index, 15);
    }

    // -----------------------------------------------------------------------
    // parse_market_browse (item)
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_market_browse_item() {
        let mut msg = NetworkMessage::new();
        msg.add_u8(0x01); // browse_id = item browse
        msg.add_u16(9999); // sprite_id
        msg.set_buffer_position(0);
        let pkt = parse_market_browse(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.browse_id, 0x01);
        assert_eq!(pkt.sprite_id, 9999);
    }

    #[test]
    fn test_parse_market_browse_own_offers() {
        let mut msg = build_msg(&[0xFE]); // MARKETREQUEST_OWN_OFFERS
        let pkt = parse_market_browse(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.browse_id, 0xFE);
        assert_eq!(pkt.sprite_id, 0); // not read
    }

    #[test]
    fn test_parse_market_browse_own_history() {
        let mut msg = build_msg(&[0xFF]); // MARKETREQUEST_OWN_HISTORY
        let pkt = parse_market_browse(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.browse_id, 0xFF);
        assert_eq!(pkt.sprite_id, 0); // not read
    }

    // -----------------------------------------------------------------------
    // parse_market_cancel_offer
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_market_cancel_offer() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(1234567890);
        msg.add_u16(42);
        msg.set_buffer_position(0);
        let pkt = parse_market_cancel_offer(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.timestamp, 1234567890);
        assert_eq!(pkt.counter, 42);
    }

    // -----------------------------------------------------------------------
    // parse_market_accept_offer
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_market_accept_offer() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(1000);
        msg.add_u16(7);
        msg.add_u16(50);
        msg.set_buffer_position(0);
        let pkt = parse_market_accept_offer(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.timestamp, 1000);
        assert_eq!(pkt.counter, 7);
        assert_eq!(pkt.amount, 50);
    }

    // -----------------------------------------------------------------------
    // parse_add_vip_by_name
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_add_vip_by_name() {
        let mut msg = NetworkMessage::new();
        msg.add_string("Godlike");
        msg.set_buffer_position(0);
        let pkt = parse_add_vip_by_name(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.name, "Godlike");
    }

    // -----------------------------------------------------------------------
    // parse_remove_vip
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_remove_vip() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(0xABCD_1234);
        msg.set_buffer_position(0);
        let pkt = parse_remove_vip(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.guid, 0xABCD_1234);
    }

    // -----------------------------------------------------------------------
    // parse_rotate_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_rotate_item() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(300);
        msg.add_u16(400);
        msg.add_u8(8);
        msg.add_u16(1500);
        msg.add_u8(0);
        msg.set_buffer_position(0);
        let pkt = parse_rotate_item(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.pos_x, 300);
        assert_eq!(pkt.pos_y, 400);
        assert_eq!(pkt.pos_z, 8);
        assert_eq!(pkt.sprite_id, 1500);
        assert_eq!(pkt.stackpos, 0);
    }

    // -----------------------------------------------------------------------
    // parse_equip_object
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_equip_object() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(2000);
        msg.set_buffer_position(0);
        let pkt = parse_equip_object(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.sprite_id, 2000);
    }

    // -----------------------------------------------------------------------
    // parse_text_window
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_text_window() {
        let mut msg = NetworkMessage::new();
        msg.add_u32(9);
        msg.add_string("Hello world");
        msg.set_buffer_position(0);
        let pkt = parse_text_window(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.window_text_id, 9);
        assert_eq!(pkt.text, "Hello world");
    }

    // -----------------------------------------------------------------------
    // parse_house_window
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_house_window() {
        let mut msg = NetworkMessage::new();
        msg.add_u8(1);
        msg.add_u32(42);
        msg.add_string("guests");
        msg.set_buffer_position(0);
        let pkt = parse_house_window(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.door_id, 1);
        assert_eq!(pkt.window_id, 42);
        assert_eq!(pkt.text, "guests");
    }

    // -----------------------------------------------------------------------
    // parse_open_channel / parse_close_channel
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_open_channel() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(5);
        msg.set_buffer_position(0);
        let pkt = parse_open_channel(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.channel_id, 5);
    }

    #[test]
    fn test_parse_close_channel() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(8);
        msg.set_buffer_position(0);
        let pkt = parse_close_channel(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.channel_id, 8);
    }

    // -----------------------------------------------------------------------
    // parse_channel_invite / parse_channel_exclude
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_channel_invite() {
        let mut msg = NetworkMessage::new();
        msg.add_string("Buddy");
        msg.set_buffer_position(0);
        let pkt = parse_channel_invite(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.name, "Buddy");
    }

    #[test]
    fn test_parse_channel_exclude() {
        let mut msg = NetworkMessage::new();
        msg.add_string("Foe");
        msg.set_buffer_position(0);
        let pkt = parse_channel_exclude(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.name, "Foe");
    }

    // -----------------------------------------------------------------------
    // parse_open_private_channel
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_open_private_channel() {
        let mut msg = NetworkMessage::new();
        msg.add_string("Alice");
        msg.set_buffer_position(0);
        let pkt = parse_open_private_channel(&mut msg).expect("parse should succeed");
        assert_eq!(pkt.receiver, "Alice");
    }

    // -----------------------------------------------------------------------
    // serialize_ping / serialize_ping_back
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_ping() {
        let bytes = serialize_ping();
        assert_eq!(bytes[0], 0x1D);
    }

    #[test]
    fn test_serialize_ping_back() {
        let bytes = serialize_ping_back();
        assert_eq!(bytes[0], 0x1E);
    }

    // -----------------------------------------------------------------------
    // serialize_creature_health
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_creature_health() {
        let bytes = serialize_creature_health(111, 75);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x8C);
        assert_eq!(msg.get_u32(), 111);
        assert_eq!(msg.get_u8(), 75);
    }

    // -----------------------------------------------------------------------
    // serialize_magic_effect
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_magic_effect() {
        let bytes = serialize_magic_effect(100, 200, 7, 5);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x83);
        assert_eq!(msg.get_u16(), 100); // pos_x
        assert_eq!(msg.get_u16(), 200); // pos_y
        assert_eq!(msg.get_u8(), 7); // pos_z
        assert_eq!(msg.get_u8(), 0x03); // MAGIC_EFFECTS_CREATE_EFFECT
        assert_eq!(msg.get_u8(), 5); // effect_type
        assert_eq!(msg.get_u8(), 0x00); // MAGIC_EFFECTS_END_LOOP
    }

    // -----------------------------------------------------------------------
    // serialize_close_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_close_container() {
        let bytes = serialize_close_container(3);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x6F);
        assert_eq!(msg.get_u8(), 3);
    }

    // -----------------------------------------------------------------------
    // serialize_cancel_walk
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_cancel_walk() {
        let bytes = serialize_cancel_walk(2);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xB5);
        assert_eq!(msg.get_u8(), 2);
    }

    // -----------------------------------------------------------------------
    // serialize_cancel_target
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_cancel_target() {
        let bytes = serialize_cancel_target();
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xA3);
        assert_eq!(msg.get_u32(), 0x00);
    }

    // -----------------------------------------------------------------------
    // serialize_change_speed
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_change_speed() {
        let bytes = serialize_change_speed(55, 100, 200);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x8F);
        assert_eq!(msg.get_u32(), 55);
        assert_eq!(msg.get_u16(), 100);
        assert_eq!(msg.get_u16(), 200);
    }

    // -----------------------------------------------------------------------
    // serialize_close_private / serialize_create_private_channel
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_close_private() {
        let bytes = serialize_close_private(7);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xB3);
        assert_eq!(msg.get_u16(), 7);
    }

    #[test]
    fn test_serialize_create_private_channel() {
        let bytes = serialize_create_private_channel(10, "My Channel", "Owner");
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xB2);
        assert_eq!(msg.get_u16(), 10);
        assert_eq!(msg.get_string(0), "My Channel");
        assert_eq!(msg.get_u16(), 0x0001);
        assert_eq!(msg.get_string(0), "Owner");
        assert_eq!(msg.get_u16(), 0x0000);
    }

    // -----------------------------------------------------------------------
    // serialize_channel_event
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_channel_event() {
        let bytes = serialize_channel_event(5, "Bob", 2);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xF3);
        assert_eq!(msg.get_u16(), 5);
        assert_eq!(msg.get_string(0), "Bob");
        assert_eq!(msg.get_u8(), 2);
    }

    // -----------------------------------------------------------------------
    // serialize_open_private_channel (send)
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_open_private_channel() {
        let bytes = serialize_open_private_channel("Carol");
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xAD);
        assert_eq!(msg.get_string(0), "Carol");
    }

    // -----------------------------------------------------------------------
    // serialize_tutorial
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_tutorial() {
        let bytes = serialize_tutorial(42);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xDC);
        assert_eq!(msg.get_u8(), 42);
    }

    // -----------------------------------------------------------------------
    // serialize_add_marker
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_add_marker() {
        let bytes = serialize_add_marker(100, 200, 7, 1, "treasure");
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xDD);
        assert_eq!(msg.get_u8(), 0x00); // unknown
        assert_eq!(msg.get_u16(), 100);
        assert_eq!(msg.get_u16(), 200);
        assert_eq!(msg.get_u8(), 7);
        assert_eq!(msg.get_u8(), 1);
        assert_eq!(msg.get_string(0), "treasure");
    }

    // -----------------------------------------------------------------------
    // serialize_relogin_window
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_relogin_window() {
        let bytes = serialize_relogin_window(30);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x28);
        assert_eq!(msg.get_u8(), 0x00);
        assert_eq!(msg.get_u8(), 30);
        assert_eq!(msg.get_u8(), 0x00);
    }

    // -----------------------------------------------------------------------
    // serialize_vip
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_vip() {
        let bytes = serialize_vip(1001, "Knight", "my ally", 3, true, 1);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xD2);
        assert_eq!(msg.get_u32(), 1001);
        assert_eq!(msg.get_string(0), "Knight");
        assert_eq!(msg.get_string(0), "my ally");
        assert_eq!(msg.get_u32(), 3); // icon (≤10)
        assert_eq!(msg.get_u8(), 0x01); // notify = true
        assert_eq!(msg.get_u8(), 1); // status
        assert_eq!(msg.get_u8(), 0x00); // vip groups placeholder
    }

    #[test]
    fn test_serialize_vip_icon_capped_at_10() {
        let bytes = serialize_vip(1, "X", "", 99, false, 0);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        msg.get_u8(); // opcode
        msg.get_u32(); // guid
        msg.get_string(0); // name
        msg.get_string(0); // description
        assert_eq!(msg.get_u32(), 10); // icon capped at 10
    }

    // -----------------------------------------------------------------------
    // serialize_updated_vip_status
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_updated_vip_status() {
        let bytes = serialize_updated_vip_status(500, 1);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xD3);
        assert_eq!(msg.get_u32(), 500);
        assert_eq!(msg.get_u8(), 1);
    }

    // -----------------------------------------------------------------------
    // serialize_spell_cooldown / group / use_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_spell_cooldown() {
        let bytes = serialize_spell_cooldown(10, 3000);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xA4);
        assert_eq!(msg.get_u16(), 10);
        assert_eq!(msg.get_u32(), 3000);
    }

    #[test]
    fn test_serialize_spell_group_cooldown() {
        let bytes = serialize_spell_group_cooldown(2, 5000);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xA5);
        assert_eq!(msg.get_u8(), 2);
        assert_eq!(msg.get_u32(), 5000);
    }

    #[test]
    fn test_serialize_use_item_cooldown() {
        let bytes = serialize_use_item_cooldown(2000);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xA6);
        assert_eq!(msg.get_u32(), 2000);
    }

    // -----------------------------------------------------------------------
    // serialize_supply_used
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_supply_used() {
        let bytes = serialize_supply_used(1234);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xCE);
        assert_eq!(msg.get_u16(), 1234);
    }

    // -----------------------------------------------------------------------
    // serialize_session_end
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_session_end() {
        let bytes = serialize_session_end(2);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x18);
        assert_eq!(msg.get_u8(), 2);
    }

    // -----------------------------------------------------------------------
    // serialize_pending_state_entered / serialize_enter_world
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_pending_state_entered() {
        let bytes = serialize_pending_state_entered();
        assert_eq!(bytes[0], 0x0A);
    }

    #[test]
    fn test_serialize_enter_world() {
        let bytes = serialize_enter_world();
        assert_eq!(bytes[0], 0x0F);
    }

    // -----------------------------------------------------------------------
    // serialize_fight_modes
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_fight_modes() {
        let bytes = serialize_fight_modes(1, 0, 1);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xA7);
        assert_eq!(msg.get_u8(), 1); // fight_mode
        assert_eq!(msg.get_u8(), 0); // chase_mode
        assert_eq!(msg.get_u8(), 1); // secure_mode
        assert_eq!(msg.get_u8(), 0); // pvp_mode (dove)
    }

    // -----------------------------------------------------------------------
    // serialize_fyi_box
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_fyi_box() {
        let bytes = serialize_fyi_box("Server going down!");
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x15);
        assert_eq!(msg.get_string(0), "Server going down!");
    }

    // -----------------------------------------------------------------------
    // serialize_experience_tracker
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_experience_tracker() {
        let bytes = serialize_experience_tracker(1000, 950);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0xAF);
        assert_eq!(msg.get_u64(), 1000);
        assert_eq!(msg.get_u64(), 950);
    }

    // -----------------------------------------------------------------------
    // serialize_creature_light
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_creature_light() {
        let bytes = serialize_creature_light(42, 7, 215);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x8D);
        assert_eq!(msg.get_u32(), 42);
        assert_eq!(msg.get_u8(), 7);
        assert_eq!(msg.get_u8(), 215);
    }

    // -----------------------------------------------------------------------
    // serialize_creature_skull
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_creature_skull() {
        let bytes = serialize_creature_skull(88, 3);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x90);
        assert_eq!(msg.get_u32(), 88);
        assert_eq!(msg.get_u8(), 3);
    }

    // -----------------------------------------------------------------------
    // serialize_creature_square
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_creature_square() {
        let bytes = serialize_creature_square(77, 0xFF);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x93);
        assert_eq!(msg.get_u32(), 77);
        assert_eq!(msg.get_u8(), 0x01); // square type
        assert_eq!(msg.get_u8(), 0xFF); // color
    }

    // -----------------------------------------------------------------------
    // serialize_creature_walkthrough
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_creature_walkthrough_true() {
        let bytes = serialize_creature_walkthrough(10, true);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x92);
        assert_eq!(msg.get_u32(), 10);
        assert_eq!(msg.get_u8(), 0x00); // walkthrough=true → 0x00
    }

    #[test]
    fn test_serialize_creature_walkthrough_false() {
        let bytes = serialize_creature_walkthrough(10, false);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x92);
        assert_eq!(msg.get_u32(), 10);
        assert_eq!(msg.get_u8(), 0x01); // walkthrough=false → 0x01 (solid)
    }

    // -----------------------------------------------------------------------
    // serialize_map_description_header
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_map_description_header() {
        let bytes = serialize_map_description_header(1000, 2000, 7);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x64);
        assert_eq!(msg.get_u16(), 1000);
        assert_eq!(msg.get_u16(), 2000);
        assert_eq!(msg.get_u8(), 7);
    }

    // -----------------------------------------------------------------------
    // serialize_add_tile_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_add_tile_item_plain_item() {
        // Non-stackable, non-special item: serializer emits opcode + position
        // + stackpos + just the client_id (u16) — no sub-type byte.
        let meta = ItemTypeMeta {
            client_id: 3456,
            ..Default::default()
        };
        let bytes = serialize_add_tile_item(100, 200, 7, 2, 0, meta);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x6A);
        assert_eq!(msg.get_u16(), 100);
        assert_eq!(msg.get_u16(), 200);
        assert_eq!(msg.get_u8(), 7);
        assert_eq!(msg.get_u8(), 2);
        assert_eq!(msg.get_u16(), 3456);
    }

    #[test]
    fn test_serialize_add_tile_item_stackable_includes_count_byte() {
        // Stackable items append a count byte after the client_id.
        let meta = ItemTypeMeta {
            client_id: 100,
            stackable: true,
            ..Default::default()
        };
        let bytes = serialize_add_tile_item(10, 20, 7, 5, 42, meta);
        // opcode (1) + pos (5) + stackpos (1) + client_id (2) + count (1) = 10
        assert_eq!(bytes.len(), 10);
        assert_eq!(bytes[0], 0x6A);
        // Position
        assert_eq!(&bytes[1..3], &[10, 0]);
        assert_eq!(&bytes[3..5], &[20, 0]);
        assert_eq!(bytes[5], 7);
        // stackpos
        assert_eq!(bytes[6], 5);
        // client_id (LE) + count
        assert_eq!(&bytes[7..10], &[100, 0, 42]);
    }

    #[test]
    fn test_serialize_add_tile_item_fluid_container_uses_fluid_map() {
        // Fluid container: count byte is FLUID_MAP[count & 7].
        let meta = ItemTypeMeta {
            client_id: 1,
            is_fluid_container: true,
            ..Default::default()
        };
        let bytes = serialize_add_tile_item(0, 0, 0, 0, 4, meta);
        // FLUID_MAP[4] = 6 (CLIENTFLUID_GREEN)
        let last = *bytes.last().unwrap();
        assert_eq!(last, 6);
    }

    #[test]
    fn test_serialize_add_tile_item_podium_appends_block() {
        let meta = ItemTypeMeta {
            client_id: 1,
            is_podium: true,
            ..Default::default()
        };
        let bytes = serialize_add_tile_item(0, 0, 0, 0, 0, meta);
        // opcode (1) + pos (5) + stackpos (1) + client_id (2) + podium block (6)
        assert_eq!(bytes.len(), 1 + 5 + 1 + 2 + 6);
        let tail = &bytes[bytes.len() - 6..];
        // lookType=0 + lookMount=0 + direction=2 + platform=0x01
        assert_eq!(tail, &[0, 0, 0, 0, 2, 0x01]);
    }

    // -----------------------------------------------------------------------
    // serialize_update_tile_item  (per-instance addItem path)
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_update_tile_item_plain_item() {
        let meta = ItemTypeMeta {
            client_id: 0x0064,
            ..Default::default()
        };
        let bytes = serialize_update_tile_item(50, 60, 5, 1, 0, 0, 0, None, None, meta);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x6B);
        assert_eq!(msg.get_u16(), 50);
        assert_eq!(msg.get_u16(), 60);
        assert_eq!(msg.get_u8(), 5);
        assert_eq!(msg.get_u8(), 1);
        assert_eq!(msg.get_u16(), 0x0064); // client_id only — no sub-type byte
    }

    #[test]
    fn test_serialize_update_tile_item_stackable_with_count() {
        let meta = ItemTypeMeta {
            client_id: 0x00C8,
            stackable: true,
            ..Default::default()
        };
        let bytes = serialize_update_tile_item(0, 0, 0, 0, 0xAA, 0, 0, None, None, meta);
        // tail: client_id LE + count
        assert_eq!(bytes[bytes.len() - 3..], [0xC8, 0x00, 0xAA]);
    }

    #[test]
    fn test_serialize_update_tile_item_classified_item() {
        let meta = ItemTypeMeta {
            client_id: 7,
            classification: 2,
            ..Default::default()
        };
        let bytes = serialize_update_tile_item(0, 0, 0, 0, 0, 0, 0, None, None, meta);
        // tail: client_id LE + 0x00 (tier byte)
        assert_eq!(bytes[bytes.len() - 3..], [7, 0, 0x00]);
    }

    #[test]
    fn test_serialize_update_tile_item_show_client_charges_with_explicit_value() {
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_charges: true,
            ..Default::default()
        };
        let bytes = serialize_update_tile_item(0, 0, 0, 0, 0, 0xCAFE, 0, None, None, meta);
        // tail: client_id LE + charges LE + 0x00
        assert_eq!(
            bytes[bytes.len() - 7..],
            [1, 0, 0xFE, 0xCA, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_serialize_update_tile_item_show_client_duration_uses_duration_seconds() {
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_duration: true,
            ..Default::default()
        };
        let bytes = serialize_update_tile_item(0, 0, 0, 0, 0, 0, 60, None, None, meta);
        // tail: client_id LE + duration LE + 0x00
        assert_eq!(bytes[bytes.len() - 7..], [1, 0, 60, 0, 0, 0, 0]);
    }

    #[test]
    fn test_serialize_update_tile_item_container_non_quiver() {
        let meta = ItemTypeMeta {
            client_id: 1,
            is_container: true,
            weapon_type: 0,
            ..Default::default()
        };
        let bytes = serialize_update_tile_item(0, 0, 0, 0, 0, 0, 0, None, None, meta);
        // tail: client_id LE + 0x00 (loot icon) + 0x00 (quiver byte)
        assert_eq!(bytes[bytes.len() - 4..], [1, 0, 0x00, 0x00]);
    }

    #[test]
    fn test_serialize_update_tile_item_container_quiver_with_ammo() {
        let meta = ItemTypeMeta {
            client_id: 1,
            is_container: true,
            weapon_type: forgottenserver_common::networkmessage::WEAPON_TYPE_QUIVER,
            ..Default::default()
        };
        let bytes = serialize_update_tile_item(0, 0, 0, 0, 0, 0, 0, Some(123), None, meta);
        // tail: client_id LE + 0x00 (loot icon) + 0x01 (quiver flag) + 4-byte ammo LE
        assert_eq!(bytes[bytes.len() - 8..], [1, 0, 0x00, 0x01, 123, 0, 0, 0]);
    }

    #[test]
    fn test_serialize_update_tile_item_podium_with_outfit_and_mount() {
        let meta = ItemTypeMeta {
            client_id: 1,
            is_podium: true,
            ..Default::default()
        };
        let podium = PodiumMeta {
            show_outfit: true,
            show_mount: true,
            show_platform: true,
            look_type: 0x0102,
            look_head: 1,
            look_body: 2,
            look_legs: 3,
            look_feet: 4,
            look_addons: 5,
            look_mount: 0x0203,
            look_mount_head: 10,
            look_mount_body: 20,
            look_mount_legs: 30,
            look_mount_feet: 40,
            direction: 2,
        };
        let bytes = serialize_update_tile_item(0, 0, 0, 0, 0, 0, 0, None, Some(podium), meta);
        // Item tail (17 bytes) after opcode+pos+stackpos:
        //   client_id LE (1 0) + look_type LE (02 01) + 5 outfit bytes (1 2 3 4 5)
        //   + look_mount LE (03 02) + 4 mount bytes (10 20 30 40)
        //   + direction (2) + platform (0x01)
        assert_eq!(
            bytes[bytes.len() - 17..],
            [1, 0, 0x02, 0x01, 1, 2, 3, 4, 5, 0x03, 0x02, 10, 20, 30, 40, 2, 0x01]
        );
    }

    // -----------------------------------------------------------------------
    // serialize_remove_tile_thing
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_remove_tile_thing() {
        let bytes = serialize_remove_tile_thing(50, 60, 5, 1);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x6C);
        assert_eq!(msg.get_u16(), 50);
        assert_eq!(msg.get_u16(), 60);
        assert_eq!(msg.get_u8(), 5);
        assert_eq!(msg.get_u8(), 1);
    }

    // -----------------------------------------------------------------------
    // serialize_inventory_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_inventory_item_with_plain_item() {
        // Non-stackable, non-special item: serializer emits opcode + slot +
        // just the client_id (u16).
        let meta = ItemTypeMeta {
            client_id: 1234,
            ..Default::default()
        };
        let bytes = serialize_inventory_item(1, Some((0, meta)));
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x78);
        assert_eq!(msg.get_u8(), 1);
        assert_eq!(msg.get_u16(), 1234);
    }

    #[test]
    fn test_serialize_inventory_item_empty_slot() {
        let bytes = serialize_inventory_item(3, None);
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&bytes);
        msg.set_buffer_position(0);
        assert_eq!(msg.get_u8(), 0x79);
        assert_eq!(msg.get_u8(), 3);
    }

    #[test]
    fn test_serialize_inventory_item_with_stackable_item() {
        // Stackable items include a count byte from add_item_payload.
        let meta = ItemTypeMeta {
            client_id: 50,
            stackable: true,
            ..Default::default()
        };
        let bytes = serialize_inventory_item(2, Some((99, meta)));
        // opcode + slot + client_id (2) + count (1) = 5
        assert_eq!(bytes, vec![0x78, 2, 50, 0, 99]);
    }

    #[test]
    fn test_serialize_inventory_item_with_container_item() {
        // Container items emit two trailing zero bytes.
        let meta = ItemTypeMeta {
            client_id: 30,
            is_container: true,
            ..Default::default()
        };
        let bytes = serialize_inventory_item(4, Some((0, meta)));
        assert_eq!(bytes, vec![0x78, 4, 30, 0, 0x00, 0x00]);
    }

    #[test]
    fn test_serialize_inventory_item_with_classified_item() {
        // Classified items emit the tier byte (0x00 for fresh items).
        let meta = ItemTypeMeta {
            client_id: 77,
            classification: 5,
            ..Default::default()
        };
        let bytes = serialize_inventory_item(0, Some((0, meta)));
        assert_eq!(bytes, vec![0x78, 0, 77, 0, 0x00]);
    }

    #[test]
    fn test_serialize_inventory_item_with_show_client_charges() {
        // showClientCharges items emit charges (u32) + brand-new byte.
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_charges: true,
            charges: 0x12345678,
            ..Default::default()
        };
        let bytes = serialize_inventory_item(0, Some((0, meta)));
        // opcode + slot + client_id (LE) + charges (LE) + 0x00
        assert_eq!(bytes, vec![0x78, 0, 1, 0, 0x78, 0x56, 0x34, 0x12, 0x00]);
    }

    #[test]
    fn test_serialize_inventory_item_with_show_client_duration() {
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_duration: true,
            decay_time_min: 0x00000042,
            ..Default::default()
        };
        let bytes = serialize_inventory_item(0, Some((0, meta)));
        assert_eq!(bytes, vec![0x78, 0, 1, 0, 0x42, 0x00, 0x00, 0x00, 0x00]);
    }

    // -----------------------------------------------------------------------
    // Overrun-path tests: every parse_* function returns Err when the inbound
    // NetworkMessage has no payload to read.
    //
    // A freshly-constructed NetworkMessage starts at position 8 (the post-
    // header region) with length=0, so the first read trips `is_overrun()`
    // via the `(pos + n) > (len + 8)` check in `NetworkMessage::can_read`.
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_login_packet_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_login_packet(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "login packet overrun");
    }

    #[test]
    fn test_parse_walk_packet_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_walk_packet(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "walk packet overrun");
    }

    #[test]
    fn test_parse_say_packet_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_say_packet(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "say packet overrun");
    }

    #[test]
    fn test_parse_use_item_packet_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_use_item_packet(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "use item packet overrun");
    }

    #[test]
    fn test_parse_look_at_packet_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_look_at_packet(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "look at packet overrun");
    }

    #[test]
    fn test_parse_trade_packet_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_trade_packet(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "trade packet overrun");
    }

    #[test]
    fn test_parse_vip_packet_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_vip_packet(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "vip packet overrun");
    }

    #[test]
    fn test_parse_fight_modes_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_fight_modes(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "fight modes packet overrun");
    }

    #[test]
    fn test_parse_attack_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_attack(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "attack packet overrun");
    }

    #[test]
    fn test_parse_follow_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_follow(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "follow packet overrun");
    }

    #[test]
    fn test_parse_close_container_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_close_container(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "close container packet overrun");
    }

    #[test]
    fn test_parse_up_arrow_container_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_up_arrow_container(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "up arrow container packet overrun");
    }

    #[test]
    fn test_parse_throw_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_throw(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "throw packet overrun");
    }

    #[test]
    fn test_parse_look_in_battle_list_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_look_in_battle_list(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "look in battle list packet overrun");
    }

    #[test]
    fn test_parse_invite_to_party_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_invite_to_party(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "invite to party packet overrun");
    }

    #[test]
    fn test_parse_join_party_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_join_party(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "join party packet overrun");
    }

    #[test]
    fn test_parse_revoke_party_invite_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_revoke_party_invite(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "revoke party invite packet overrun");
    }

    #[test]
    fn test_parse_pass_party_leadership_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_pass_party_leadership(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "pass party leadership packet overrun");
    }

    #[test]
    fn test_parse_enable_shared_party_experience_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_enable_shared_party_experience(&mut msg)
            .expect_err("empty buffer should overrun");
        assert_eq!(err, "enable shared party experience packet overrun");
    }

    #[test]
    fn test_parse_modal_window_answer_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_modal_window_answer(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "modal window answer packet overrun");
    }

    #[test]
    fn test_parse_browse_field_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_browse_field(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "browse field packet overrun");
    }

    #[test]
    fn test_parse_seek_in_container_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_seek_in_container(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "seek in container packet overrun");
    }

    #[test]
    fn test_parse_market_browse_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_market_browse(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "market browse packet overrun");
    }

    #[test]
    fn test_parse_market_cancel_offer_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_market_cancel_offer(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "market cancel offer packet overrun");
    }

    #[test]
    fn test_parse_market_accept_offer_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_market_accept_offer(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "market accept offer packet overrun");
    }

    #[test]
    fn test_parse_add_vip_by_name_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_add_vip_by_name(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "add vip by name packet overrun");
    }

    #[test]
    fn test_parse_remove_vip_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_remove_vip(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "remove vip packet overrun");
    }

    #[test]
    fn test_parse_rotate_item_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_rotate_item(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "rotate item packet overrun");
    }

    #[test]
    fn test_parse_equip_object_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_equip_object(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "equip object packet overrun");
    }

    #[test]
    fn test_parse_text_window_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_text_window(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "text window packet overrun");
    }

    #[test]
    fn test_parse_house_window_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_house_window(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "house window packet overrun");
    }

    #[test]
    fn test_parse_open_channel_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_open_channel(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "open channel packet overrun");
    }

    #[test]
    fn test_parse_close_channel_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_close_channel(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "close channel packet overrun");
    }

    #[test]
    fn test_parse_channel_invite_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_channel_invite(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "channel invite packet overrun");
    }

    #[test]
    fn test_parse_channel_exclude_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_channel_exclude(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "channel exclude packet overrun");
    }

    #[test]
    fn test_parse_open_private_channel_overrun() {
        let mut msg = NetworkMessage::new();
        let err = parse_open_private_channel(&mut msg).expect_err("empty buffer should overrun");
        assert_eq!(err, "open private channel packet overrun");
    }
}
