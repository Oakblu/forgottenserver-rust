//! Byte-parity tests for the `wire-combat-misc` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs` (Session 9).
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  The 4 newly-added serializers (distance_shoot, items,
//! item_classes, combat_analyzer) are covered, as are the 12 existing
//! stubs (magic_effect was STUB-PROMOTED: byte 0x01 → 0x03 to match
//! `MAGIC_EFFECTS_CREATE_EFFECT = 3`).
//!
//! All 12 existing functions return `Vec<u8>` (Tier-3 legacy style), so
//! tests here use `crate::fixtures::assert_bytes_eq` directly rather
//! than the `expected_bytes!` macro (which is tied to OutputMessage).

use forgottenserver_network::protocolgame::{
    serialize_add_marker, serialize_combat_analyzer, serialize_distance_shoot,
    serialize_enter_world, serialize_fight_modes, serialize_item_classes, serialize_items,
    serialize_magic_effect, serialize_pending_state_entered, serialize_relogin_window,
    serialize_session_end, serialize_spell_cooldown, serialize_spell_group_cooldown,
    serialize_supply_used, serialize_tutorial, serialize_use_item_cooldown,
};

// ---------------------------------------------------------------------------
// sendDistanceShoot — opcode 0x83 (NEW, Session 9)
// ---------------------------------------------------------------------------

#[test]
fn parity_distance_shoot_positive_delta() {
    // from=(100,200,7) → to=(103,202,7): dx=+3, dy=+2; effect_id=5
    let bytes = serialize_distance_shoot(100, 200, 7, 103, 202, 5);

    // C++ protocolgame.cpp:2547 sendDistanceShoot
    //   opcode 0x83 | from pos (u16 x, u16 y, u8 z)
    //   | MAGIC_EFFECTS_CREATE_DISTANCEEFFECT (0x04) | effect_id u8
    //   | int8 dx | int8 dy | MAGIC_EFFECTS_END_LOOP (0x00)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0x83, 0x64, 0x00, // from.x = 100
            0xC8, 0x00, // from.y = 200
            0x07, // from.z
            0x04, // MAGIC_EFFECTS_CREATE_DISTANCEEFFECT
            0x05, // effect_id
            0x03, // dx = +3
            0x02, // dy = +2
            0x00, // MAGIC_EFFECTS_END_LOOP
        ],
        "protocolgame.cpp:2547 sendDistanceShoot",
    );
}

#[test]
fn parity_distance_shoot_negative_delta() {
    // from=(50,50,5) → to=(48,47,5): dx=-2, dy=-3 (i8 0xFE, 0xFD)
    let bytes = serialize_distance_shoot(50, 50, 5, 48, 47, 9);

    // C++ protocolgame.cpp:2547 sendDistanceShoot — negative delta path
    //   delta bytes preserve i8 two's-complement bit pattern as u8.
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0x83, 0x32, 0x00, // from.x = 50
            0x32, 0x00, // from.y = 50
            0x05, // from.z
            0x04, // MAGIC_EFFECTS_CREATE_DISTANCEEFFECT
            0x09, // effect_id
            0xFE, // dx = -2  (i8 → u8)
            0xFD, // dy = -3  (i8 → u8)
            0x00, // MAGIC_EFFECTS_END_LOOP
        ],
        "protocolgame.cpp:2547 sendDistanceShoot (negative delta)",
    );
}

// ---------------------------------------------------------------------------
// sendMagicEffect — opcode 0x83 (STUB-PROMOTED: 0x01 → 0x03)
// ---------------------------------------------------------------------------

#[test]
fn parity_magic_effect_byte_layout() {
    // Verify the STUB-PROMOTED layout: the create-effect tag byte is the
    // C++ enum value MAGIC_EFFECTS_CREATE_EFFECT = 3 (was 0x01 pre-Session 9).
    let bytes = serialize_magic_effect(100, 200, 7, 5);

    // C++ protocolgame.cpp:2560 sendMagicEffect
    //   opcode 0x83 | pos (u16 x, u16 y, u8 z)
    //   | MAGIC_EFFECTS_CREATE_EFFECT (0x03) | type u8
    //   | MAGIC_EFFECTS_END_LOOP (0x00)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0x83, 0x64, 0x00, // pos.x = 100
            0xC8, 0x00, // pos.y = 200
            0x07, // pos.z
            0x03, // MAGIC_EFFECTS_CREATE_EFFECT
            0x05, // type = 5
            0x00, // MAGIC_EFFECTS_END_LOOP
        ],
        "protocolgame.cpp:2560 sendMagicEffect",
    );
}

// ---------------------------------------------------------------------------
// sendSpellCooldown — opcode 0xA4 (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_spell_cooldown() {
    // spell_id = 0x42 (written as u16); time = 0xDEAD_BEEF
    let bytes = serialize_spell_cooldown(0x42, 0xDEAD_BEEF);

    // C++ protocolgame.cpp:3316 sendSpellCooldown
    //   opcode 0xA4 | spell_id (u16 LE, cast from u8) | time u32 LE
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xA4, 0x42, 0x00, // spell_id u16 LE
            0xEF, 0xBE, 0xAD, 0xDE, // time u32 LE
        ],
        "protocolgame.cpp:3316 sendSpellCooldown",
    );
}

// ---------------------------------------------------------------------------
// sendSpellGroupCooldown — opcode 0xA5 (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_spell_group_cooldown() {
    let bytes = serialize_spell_group_cooldown(2, 5000);

    // C++ protocolgame.cpp:3325 sendSpellGroupCooldown
    //   opcode 0xA5 | group_id u8 | time u32 LE
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xA5, 0x02, 0x88, 0x13, 0x00, 0x00, // 5000 LE
        ],
        "protocolgame.cpp:3325 sendSpellGroupCooldown",
    );
}

// ---------------------------------------------------------------------------
// sendUseItemCooldown — opcode 0xA6 (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_use_item_cooldown() {
    let bytes = serialize_use_item_cooldown(0x0001_0000);

    // C++ protocolgame.cpp:3334 sendUseItemCooldown
    //   opcode 0xA6 | time u32 LE
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0xA6, 0x00, 0x00, 0x01, 0x00],
        "protocolgame.cpp:3334 sendUseItemCooldown",
    );
}

// ---------------------------------------------------------------------------
// sendSupplyUsed — opcode 0xCE (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_supply_used() {
    let bytes = serialize_supply_used(0xCAFE);

    // C++ protocolgame.cpp:3342 sendSupplyUsed
    //   opcode 0xCE | client_id u16 LE
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0xCE, 0xFE, 0xCA],
        "protocolgame.cpp:3342 sendSupplyUsed",
    );
}

// ---------------------------------------------------------------------------
// sendReLoginWindow — opcode 0x28 (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_relogin_window() {
    let bytes = serialize_relogin_window(30);

    // C++ protocolgame.cpp:1698 sendReLoginWindow
    //   opcode 0x28 | 0x00 (padding) | unfair_fight_reduction u8
    //   | 0x00 (can use death redemption)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x28, 0x00, 0x1E, 0x00],
        "protocolgame.cpp:1698 sendReLoginWindow",
    );
}

// ---------------------------------------------------------------------------
// sendSessionEnd — opcode 0x18 (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_session_end() {
    let bytes = serialize_session_end(2);

    // C++ protocolgame.cpp:3379 sendSessionEnd
    //   opcode 0x18 | reason u8 (SessionEndTypes_t)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x18, 0x02],
        "protocolgame.cpp:3379 sendSessionEnd",
    );
}

// ---------------------------------------------------------------------------
// sendTutorial — opcode 0xDC (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_tutorial() {
    let bytes = serialize_tutorial(42);

    // C++ protocolgame.cpp:1679 sendTutorial
    //   opcode 0xDC | tutorial_id u8
    crate::fixtures::assert_bytes_eq(&bytes, &[0xDC, 0x2A], "protocolgame.cpp:1679 sendTutorial");
}

// ---------------------------------------------------------------------------
// sendAddMarker — opcode 0xDD (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_add_marker() {
    let bytes = serialize_add_marker(100, 200, 7, 1, "X");

    // C++ protocolgame.cpp:1687 sendAddMarker
    //   opcode 0xDD | 0x00 (unknown) | pos (u16 x, u16 y, u8 z)
    //   | mark_type u8 | desc (u16 len + UTF-8 bytes)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xDD, 0x00, // unknown
            0x64, 0x00, // pos.x = 100
            0xC8, 0x00, // pos.y = 200
            0x07, // pos.z
            0x01, // mark_type
            0x01, 0x00, b'X', // desc "X"
        ],
        "protocolgame.cpp:1687 sendAddMarker",
    );
}

// ---------------------------------------------------------------------------
// sendPendingStateEntered — opcode 0x0A (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_pending_state_entered() {
    let bytes = serialize_pending_state_entered();

    // C++ protocolgame.cpp:2724 sendPendingStateEntered
    //   opcode 0x0A — single byte payload
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0x0A],
        "protocolgame.cpp:2724 sendPendingStateEntered",
    );
}

// ---------------------------------------------------------------------------
// sendEnterWorld — opcode 0x0F (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_enter_world() {
    let bytes = serialize_enter_world();

    // C++ protocolgame.cpp:2731 sendEnterWorld
    //   opcode 0x0F — single byte payload
    crate::fixtures::assert_bytes_eq(&bytes, &[0x0F], "protocolgame.cpp:2731 sendEnterWorld");
}

// ---------------------------------------------------------------------------
// sendFightModes — opcode 0xA7 (WAS-COMPLETE)
// ---------------------------------------------------------------------------

#[test]
fn parity_fight_modes() {
    let bytes = serialize_fight_modes(1, 0, 1);

    // C++ protocolgame.cpp:2738 sendFightModes
    //   opcode 0xA7 | fight_mode u8 | chase_mode u8 | secure_mode u8
    //   | PVP_MODE_DOVE u8 (0x00)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0xA7, 0x01, 0x00, 0x01, 0x00],
        "protocolgame.cpp:2738 sendFightModes",
    );
}

// ---------------------------------------------------------------------------
// sendItems — opcode 0xF5 (NEW, Session 9)
// ---------------------------------------------------------------------------

#[test]
fn parity_items_empty_inventory() {
    // Inventory empty: total count = 11; 11 default slot rows; no inventory rows.
    let bytes = serialize_items(&[]);

    // C++ protocolgame.cpp:2877 sendItems
    //   opcode 0xF5 | count u16=11 | 11 × (slot u16 + 0x00 + 0x0001)
    let mut expected: Vec<u8> = Vec::new();
    expected.push(0xF5);
    expected.extend_from_slice(&11u16.to_le_bytes());
    for i in 1u16..=11 {
        expected.extend_from_slice(&i.to_le_bytes());
        expected.push(0x00);
        expected.extend_from_slice(&1u16.to_le_bytes());
    }
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &expected,
        "protocolgame.cpp:2877 sendItems (empty inventory)",
    );
}

#[test]
fn parity_items_two_entries() {
    // Two inventory rows: (client_id=0x2A48, count=5) and (0x2C48, 99).
    let inventory: &[(u16, u16)] = &[(0x2A48, 5), (0x2C48, 99)];
    let bytes = serialize_items(inventory);

    // C++ protocolgame.cpp:2877 sendItems
    //   opcode 0xF5 | count u16=13 | 11 default slot rows
    //   | (0x2A48, 0x00, 0x0005) | (0x2C48, 0x00, 0x0063)
    let mut expected: Vec<u8> = Vec::new();
    expected.push(0xF5);
    expected.extend_from_slice(&13u16.to_le_bytes());
    for i in 1u16..=11 {
        expected.extend_from_slice(&i.to_le_bytes());
        expected.push(0x00);
        expected.extend_from_slice(&1u16.to_le_bytes());
    }
    // entry 1
    expected.extend_from_slice(&0x2A48u16.to_le_bytes());
    expected.push(0x00);
    expected.extend_from_slice(&5u16.to_le_bytes());
    // entry 2
    expected.extend_from_slice(&0x2C48u16.to_le_bytes());
    expected.push(0x00);
    expected.extend_from_slice(&99u16.to_le_bytes());
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &expected,
        "protocolgame.cpp:2877 sendItems (two entries)",
    );
}

// ---------------------------------------------------------------------------
// sendItemClasses — opcode 0x86 (NEW, Session 9)
// ---------------------------------------------------------------------------

#[test]
fn parity_item_classes() {
    let bytes = serialize_item_classes();

    // C++ protocolgame.cpp:3287 sendItemClasses
    //   opcode 0x86
    //   | classSize u8 = 4
    //   | for each class (4):
    //       class_id u8 (1..=4)
    //       tier_size u8 = 10
    //       for each tier (10): tier_id u8 + upgrade_cost u64 = 10000
    //   | 11 padding bytes (tier_size + 1) = 0x00 × 11
    let mut expected: Vec<u8> = Vec::new();
    expected.push(0x86);
    expected.push(4); // classSize
    for class_id in 1u8..=4 {
        expected.push(class_id);
        expected.push(10); // tier_size
        for tier_id in 0u8..10 {
            expected.push(tier_id);
            expected.extend_from_slice(&10_000u64.to_le_bytes());
        }
    }
    expected.extend(std::iter::repeat_n(0x00u8, 11));
    crate::fixtures::assert_bytes_eq(&bytes, &expected, "protocolgame.cpp:3287 sendItemClasses");
}

// ---------------------------------------------------------------------------
// sendCombatAnalyzer — opcode 0xCC (NEW, Session 9)
// ---------------------------------------------------------------------------

#[test]
fn parity_combat_analyzer_received() {
    // RECEIVED = 2: emits client_damage_type + target string
    let bytes = serialize_combat_analyzer(2, 0xAABBCCDD, 7, "Bob");

    // C++ protocolgame.cpp:2995 sendCombatAnalyzer (RECEIVED branch)
    //   opcode 0xCC | impact_type u8=2 | amount u32 LE
    //   | client_damage_type u8 | target (u16 len + bytes)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xCC, 0x02, 0xDD, 0xCC, 0xBB, 0xAA, // amount LE
            0x07, // client_damage_type
            0x03, 0x00, b'B', b'o', b'b',
        ],
        "protocolgame.cpp:2995 sendCombatAnalyzer (RECEIVED)",
    );
}

#[test]
fn parity_combat_analyzer_dealt() {
    // DEALT = 1: emits client_damage_type, NO target string
    let bytes = serialize_combat_analyzer(1, 1234, 4, "ignored");

    // C++ protocolgame.cpp:2995 sendCombatAnalyzer (DEALT branch)
    //   opcode 0xCC | impact_type u8=1 | amount u32 LE
    //   | client_damage_type u8 — no target
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xCC, 0x01, 0xD2, 0x04, 0x00, 0x00, // 1234 LE
            0x04,
        ],
        "protocolgame.cpp:2995 sendCombatAnalyzer (DEALT)",
    );
}

#[test]
fn parity_combat_analyzer_default_branch() {
    // impact_type = 0 (NONE): only opcode + impact_type + amount written.
    let bytes = serialize_combat_analyzer(0, 0xDEAD, 9, "ignored");

    // C++ protocolgame.cpp:2995 sendCombatAnalyzer (default branch — no extra bytes)
    //   opcode 0xCC | impact_type u8=0 | amount u32 LE — switch default = no-op.
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xCC, 0x00, 0xAD, 0xDE, 0x00, 0x00, // 0xDEAD LE
        ],
        "protocolgame.cpp:2995 sendCombatAnalyzer (default)",
    );
}
