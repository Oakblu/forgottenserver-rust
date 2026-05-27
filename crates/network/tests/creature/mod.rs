//! Byte-parity tests for the `wire-creature` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs`.
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  Tests cover all 12 creature packets:
//!   * `sendCreatureOutfit`       (opcode 0x8E) — C++ line 1593
//!   * `sendCreatureLight`        (opcode 0x8D) — C++ line 1606
//!   * `sendCreatureWalkthrough`  (opcode 0x92) — C++ line 1622
//!   * `sendCreatureShield`       (opcode 0x91) — C++ line 1635
//!   * `sendCreatureSkull`        (opcode 0x90) — C++ line 1648
//!   * `sendCreatureSquare`       (opcode 0x93) — C++ line 1665
//!   * `sendCreatureTurn`         (opcode 0x6B) — C++ line 2399
//!   * `sendCreatureSay`          (opcode 0xAA) — C++ line 2422
//!   * `sendCreatureHealth`       (opcode 0x8C) — C++ line 2575
//!   * `sendUpdateCreatureIcons`  (opcode 0x8B + sub-op 14) — C++ line 2709
//!   * `sendAddCreature`          (opcode 0x6A) — C++ line 2749
//!   * `sendMoveCreature`         (opcode 0x6D) — C++ line 2788

use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_network::protocolgame::{
    serialize_add_creature, serialize_creature_health, serialize_creature_light,
    serialize_creature_move, serialize_creature_outfit, serialize_creature_say,
    serialize_creature_shield, serialize_creature_skull, serialize_creature_square,
    serialize_creature_turn, serialize_creature_walkthrough, serialize_update_creature_icons,
    AddCreatureMeta, OutfitDescriptor,
};

// ---------------------------------------------------------------------------
// sendCreatureOutfit — opcode 0x8E
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_outfit_full_human_with_mount() {
    let outfit = OutfitDescriptor {
        look_type: 0x0080, // 128
        look_head: 0x11,
        look_body: 0x22,
        look_legs: 0x33,
        look_feet: 0x44,
        look_addons: 0x03,
        look_type_ex: 0,
        look_mount: 0x0040, // 64
        look_mount_head: 0x55,
        look_mount_body: 0x66,
        look_mount_legs: 0x77,
        look_mount_feet: 0x88,
    };

    let mut out = OutputMessage::new();
    serialize_creature_outfit(&mut out, 0xDEAD_BEEF, outfit);

    // C++ protocolgame.cpp:1593 sendCreatureOutfit + AddOutfit (3579)
    //   opcode 0x8E | creature_id u32 LE | look_type u16
    //   (non-zero branch) head/body/legs/feet/addons (5×u8)
    //   look_mount u16 | (non-zero branch) head/body/legs/feet (4×u8)
    expected_bytes!(
        out,
        &[
            0x8E, 0xEF, 0xBE, 0xAD, 0xDE, 0x80, 0x00, 0x11, 0x22, 0x33, 0x44, 0x03, 0x40, 0x00,
            0x55, 0x66, 0x77, 0x88,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendCreatureLight — opcode 0x8D
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_light() {
    let bytes = serialize_creature_light(0x1234_5678, 7, 215);

    // C++ protocolgame.cpp:1606 sendCreatureLight
    //   opcode 0x8D | creature_id u32 LE | level u8 | color u8
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x8D, 0x78, 0x56, 0x34, 0x12, 0x07, 0xD7],
        "protocolgame.cpp:1606 sendCreatureLight",
    );
}

// ---------------------------------------------------------------------------
// sendCreatureWalkthrough — opcode 0x92
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_walkthrough_true_writes_zero() {
    let bytes = serialize_creature_walkthrough(0x0000_00FF, true);

    // C++ protocolgame.cpp:1622 sendCreatureWalkthrough
    //   opcode 0x92 | creature_id u32 LE | byte 0x00 (walkthrough)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x92, 0xFF, 0x00, 0x00, 0x00, 0x00],
        "protocolgame.cpp:1622 sendCreatureWalkthrough (true)",
    );
}

#[test]
fn parity_creature_walkthrough_false_writes_one() {
    let bytes = serialize_creature_walkthrough(0x0000_00FF, false);

    // Same source line; false branch writes 0x01.
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x92, 0xFF, 0x00, 0x00, 0x00, 0x01],
        "protocolgame.cpp:1622 sendCreatureWalkthrough (false)",
    );
}

// ---------------------------------------------------------------------------
// sendCreatureShield — opcode 0x91
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_shield() {
    let mut out = OutputMessage::new();
    serialize_creature_shield(&mut out, 0xCAFE_BABE, 4);

    // C++ protocolgame.cpp:1635 sendCreatureShield
    //   opcode 0x91 | creature_id u32 LE | party_shield u8
    expected_bytes!(out, &[0x91, 0xBE, 0xBA, 0xFE, 0xCA, 0x04],);
}

// ---------------------------------------------------------------------------
// sendCreatureSkull — opcode 0x90
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_skull() {
    let bytes = serialize_creature_skull(0x0000_0042, 5);

    // C++ protocolgame.cpp:1648 sendCreatureSkull
    //   opcode 0x90 | creature_id u32 LE | skull u8
    //   (caller is responsible for the WORLD_TYPE_PVP gate)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x90, 0x42, 0x00, 0x00, 0x00, 0x05],
        "protocolgame.cpp:1648 sendCreatureSkull",
    );
}

// ---------------------------------------------------------------------------
// sendCreatureSquare — opcode 0x93
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_square() {
    let bytes = serialize_creature_square(0x0000_0042, 0x07);

    // C++ protocolgame.cpp:1665 sendCreatureSquare
    //   opcode 0x93 | creature_id u32 LE | 0x01 (square type) | color u8
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x93, 0x42, 0x00, 0x00, 0x00, 0x01, 0x07],
        "protocolgame.cpp:1665 sendCreatureSquare",
    );
}

// ---------------------------------------------------------------------------
// sendCreatureTurn — opcode 0x6B
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_turn_in_range_stackpos() {
    let mut out = OutputMessage::new();
    serialize_creature_turn(
        &mut out,
        /* creature_id     */ 0x0000_00AA,
        /* pos_x           */ 100,
        /* pos_y           */ 200,
        /* pos_z           */ 7,
        /* stackpos        */ 3,
        /* direction       */ 2, // DIRECTION_SOUTH
        /* can_walkthrough */ false,
    );

    // C++ protocolgame.cpp:2399 sendCreatureTurn (stackpos < MAX_STACKPOS branch)
    //   opcode 0x6B | pos (u16 x, u16 y, u8 z) | stackpos u8
    //   0x63 u16 | creature_id u32 LE | direction u8 | walkthrough u8
    expected_bytes!(
        out,
        &[
            0x6B, 0x64, 0x00, // x=100
            0xC8, 0x00, // y=200
            0x07, // z=7
            0x03, // stackpos=3
            0x63, 0x00, // sub-opcode 0x63
            0xAA, 0x00, 0x00, 0x00, // creature_id
            0x02, // direction
            0x01, // can_walkthrough=false → 0x01
        ],
    );
}

#[test]
fn parity_creature_turn_overflow_stackpos() {
    let mut out = OutputMessage::new();
    serialize_creature_turn(
        &mut out,
        /* creature_id     */ 0x0000_00AA,
        /* pos_x           */ 100,
        /* pos_y           */ 200,
        /* pos_z           */ 7,
        /* stackpos        */ 10, // >= MAX_STACKPOS
        /* direction       */ 0,
        /* can_walkthrough */ true,
    );

    // C++ protocolgame.cpp:2407 (stackpos >= MAX_STACKPOS branch)
    //   opcode 0x6B | 0xFFFF u16 | creature_id u32
    //   0x63 u16 | creature_id u32 | direction | walkthrough
    expected_bytes!(
        out,
        &[
            0x6B, 0xFF, 0xFF, 0xAA, 0x00, 0x00, 0x00, 0x63, 0x00, 0xAA, 0x00, 0x00, 0x00, 0x00,
            0x00, // can_walkthrough=true → 0x00
        ],
    );
}

// ---------------------------------------------------------------------------
// sendCreatureSay — opcode 0xAA
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_say_player_with_pos() {
    let mut out = OutputMessage::new();
    serialize_creature_say(
        &mut out,
        /* statement_id */ 0x0001_0203,
        /* creature_name*/ "Bob",
        /* level        */ 42,
        /* speak_class  */ 1, // TALKTYPE_SAY
        /* pos_x        */ 100,
        /* pos_y        */ 200,
        /* pos_z        */ 7,
        /* text         */ "hi",
    );

    // C++ protocolgame.cpp:2422 sendCreatureSay
    //   opcode 0xAA | statement_id u32 LE | name (u16 len + bytes)
    //   0x00 ("(Traded)" suffix) | level u16 LE | speak_class u8
    //   pos (u16 x, u16 y, u8 z) | text (u16 len + bytes)
    expected_bytes!(
        out,
        &[
            0xAA, 0x03, 0x02, 0x01, 0x00, // statement_id
            // "Bob": u16 len=3 + bytes
            0x03, 0x00, b'B', b'o', b'b', 0x00, // (Traded) suffix
            0x2A, 0x00, // level=42
            0x01, // TALKTYPE_SAY
            0x64, 0x00, // pos_x=100
            0xC8, 0x00, // pos_y=200
            0x07, // pos_z=7
            // "hi": u16 len=2 + bytes
            0x02, 0x00, b'h', b'i',
        ],
    );
}

// ---------------------------------------------------------------------------
// sendCreatureHealth — opcode 0x8C
// ---------------------------------------------------------------------------

#[test]
fn parity_creature_health() {
    let bytes = serialize_creature_health(0xDEAD_BEEF, 73);

    // C++ protocolgame.cpp:2575 sendCreatureHealth
    //   opcode 0x8C | creature_id u32 LE | health_percent u8
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x8C, 0xEF, 0xBE, 0xAD, 0xDE, 0x49],
        "protocolgame.cpp:2575 sendCreatureHealth",
    );
}

// ---------------------------------------------------------------------------
// sendUpdateCreatureIcons — opcode 0x8B + sub-op 14
// ---------------------------------------------------------------------------

#[test]
fn parity_update_creature_icons_two_rows() {
    let mut out = OutputMessage::new();
    serialize_update_creature_icons(
        &mut out,
        /* creature_id */ 0x0000_0042,
        /* icons       */
        &[
            (3, 0, 12),     // icon_id=3, type=0 (regular), level=12
            (7, 1, 0x0100), // icon_id=7, type=1 (monster), level=256
        ],
    );

    // C++ protocolgame.cpp:2709 sendUpdateCreatureIcons + AddCreatureIcons (3477)
    //   opcode 0x8B | creature_id u32 LE | 14 (sub-op)
    //   count u8 | per icon: icon_id u8 + type u8 + level u16 LE
    expected_bytes!(
        out,
        &[
            0x8B, 0x42, 0x00, 0x00, 0x00, 0x0E, // 14
            0x02, // 2 icons
            0x03, 0x00, 0x0C, 0x00, // icon 1
            0x07, 0x01, 0x00, 0x01, // icon 2
        ],
    );
}

// ---------------------------------------------------------------------------
// sendAddCreature — opcode 0x6A
// ---------------------------------------------------------------------------

#[test]
fn parity_add_creature_known_player_with_outfit() {
    // Known player branch: only writes 0x62 + creature_id (no name / type
    // byte / master id / guild emblem).
    let outfit = OutfitDescriptor {
        look_type: 0x0080,
        look_head: 0x11,
        look_body: 0x22,
        look_legs: 0x33,
        look_feet: 0x44,
        look_addons: 0x01,
        look_type_ex: 0,
        look_mount: 0,
        look_mount_head: 0,
        look_mount_body: 0,
        look_mount_legs: 0,
        look_mount_feet: 0,
    };

    let meta = AddCreatureMeta {
        creature_id: 0x0000_00AA,
        creature_type: 0, // CREATURETYPE_PLAYER
        master_id: 0,
        health_hidden: false,
        health_percent: 87,
        direction: 1,
        ghost_or_invisible: false,
        outfit,
        light_level: 0x07,
        light_color: 0xD7,
        step_speed_half: 110,
        skull: 0,
        party_shield: 0,
        guild_emblem: 0,
        player_vocation_client_id: 4,
        speech_bubble: 0,
        can_walkthrough: false,
    };

    let mut out = OutputMessage::new();
    serialize_add_creature(
        &mut out, /* pos_x         */ 100, /* pos_y         */ 200,
        /* pos_z         */ 7, /* stackpos      */ 3, /* known         */ true,
        /* removed_known */ 0, /* name          */ "Bob", meta,
    );

    // C++ protocolgame.cpp:2749 sendAddCreature + AddCreature (3388).
    // Known-player branch with outfit, no mount, walk-blocked.
    expected_bytes!(
        out,
        &[
            // sendAddCreature header
            0x6A, 0x64, 0x00, 0xC8, 0x00, 0x07, // pos
            0x03, // stackpos
            // known=true → 0x62 + id (no name, no creature_type, no emblem)
            0x62, 0x00, 0xAA, 0x00, 0x00, 0x00, // health_percent
            0x57, // direction
            0x01, // outfit (look_type 128, head/body/legs/feet/addons), mount=0
            0x80, 0x00, 0x11, 0x22, 0x33, 0x44, 0x01, 0x00, 0x00, // light level/color
            0x07, 0xD7, // step_speed_half=110
            0x6E, 0x00, // creature icons: empty
            0x00, // skull, party_shield
            0x00, 0x00,
            // (known=true → no guild emblem)
            // creature_type again = 0 (CREATURETYPE_PLAYER)
            0x00, // player vocation client id = 4
            0x04, // speech bubble
            0x00, // MARK_UNMARKED, inspection type
            0xFF, 0x00, // walk-through (false → 0x01)
            0x01,
        ],
    );
}

#[test]
fn parity_add_creature_unknown_monster_health_hidden() {
    // Unknown branch + health-hidden + non-player (monster) → name string
    // is "", health byte is 0, vocation byte is NOT written.
    let meta = AddCreatureMeta {
        creature_id: 0x0000_0042,
        creature_type: 1, // not player, not summon-own
        master_id: 0,
        health_hidden: true,
        health_percent: 99, // ignored
        direction: 0,
        ghost_or_invisible: false,
        outfit: OutfitDescriptor::default(),
        light_level: 0,
        light_color: 0,
        step_speed_half: 100,
        skull: 0,
        party_shield: 0,
        guild_emblem: 5,
        player_vocation_client_id: 0,
        speech_bubble: 0,
        can_walkthrough: true,
    };

    let mut out = OutputMessage::new();
    serialize_add_creature(
        &mut out,
        100,
        200,
        7,
        /* stackpos      */ 1,
        /* known         */ false,
        /* removed_known */ 0xDEAD_BEEF,
        /* name          */ "Demon", // overridden to "" because health_hidden
        meta,
    );

    // C++ protocolgame.cpp:2749 sendAddCreature + AddCreature (3388).
    // Unknown branch + CREATURETYPE_HIDDEN (12) replaces real type.
    expected_bytes!(
        out,
        &[
            0x6A, 0x64, 0x00, 0xC8, 0x00, 0x07, 0x01,
            // known=false → 0x61, removed_known, creature_id
            0x61, 0x00, 0xEF, 0xBE, 0xAD, 0xDE, 0x42, 0x00, 0x00, 0x00,
            // creature_type byte = CREATURETYPE_HIDDEN = 12 (0x0C)
            0x0C, // name = "" (length-prefixed): u16=0
            0x00, 0x00, // health byte = 0
            0x00, // direction = 0
            0x00, // outfit: default (all zero)
            0x00, 0x00, // look_type=0 → lookTypeEx u16
            0x00, 0x00, // look_type_ex=0
            0x00, 0x00, // look_mount=0
            // light
            0x00, 0x00, // step_speed_half=100
            0x64, 0x00, // creature icons: empty
            0x00, // skull, party_shield
            0x00, 0x00, // unknown branch → guild_emblem = 5
            0x05, // creature_type again = CREATURETYPE_HIDDEN = 12
            0x0C, // not player → no vocation byte
            // speech bubble
            0x00, // MARK_UNMARKED, inspection
            0xFF, 0x00, // walk-through = true → 0x00
            0x00,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendMoveCreature — opcode 0x6D
// ---------------------------------------------------------------------------

#[test]
fn parity_move_creature_in_range_stackpos() {
    let mut out = OutputMessage::new();
    serialize_creature_move(
        &mut out,
        /* creature_id */ 0xDEAD_BEEF,
        /* old_pos_x   */ 100,
        /* old_pos_y   */ 200,
        /* old_pos_z   */ 7,
        /* old_stack_pos*/ 3,
        /* new_pos_x   */ 101,
        /* new_pos_y   */ 200,
        /* new_pos_z   */ 7,
    );

    // C++ protocolgame.cpp:2788 sendMoveCreature (stackpos < MAX_STACKPOS branch)
    //   opcode 0x6D | old_pos (u16 x, u16 y, u8 z) | old_stack_pos u8
    //   | new_pos (u16 x, u16 y, u8 z)
    expected_bytes!(
        out,
        &[0x6D, 0x64, 0x00, 0xC8, 0x00, 0x07, 0x03, 0x65, 0x00, 0xC8, 0x00, 0x07,],
    );
}

#[test]
fn parity_move_creature_overflow_stackpos() {
    let mut out = OutputMessage::new();
    serialize_creature_move(
        &mut out,
        /* creature_id */ 0xDEAD_BEEF,
        /* old_pos_x   */ 100,
        /* old_pos_y   */ 200,
        /* old_pos_z   */ 7,
        /* old_stack_pos*/ 10, // >= MAX_STACKPOS
        /* new_pos_x   */ 101,
        /* new_pos_y   */ 200,
        /* new_pos_z   */ 7,
    );

    // C++ protocolgame.cpp:2804 (stackpos >= MAX_STACKPOS branch)
    //   opcode 0x6D | 0xFFFF u16 | creature_id u32
    //   | new_pos (u16 x, u16 y, u8 z)
    expected_bytes!(
        out,
        &[0x6D, 0xFF, 0xFF, 0xEF, 0xBE, 0xAD, 0xDE, 0x65, 0x00, 0xC8, 0x00, 0x07,],
    );
}
