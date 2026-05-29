//! Byte-parity tests for the `wire-player` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs`.
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  Tests are added incrementally as Session 1+ work lands.

use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_network::protocolgame::{
    serialize_basic_data, serialize_cancel_target, serialize_cancel_walk, serialize_change_speed,
    serialize_client_features, serialize_experience_tracker, serialize_icons, serialize_ping,
    serialize_ping_back, serialize_skills, serialize_stats, serialize_store_balance,
    serialize_updated_vip_status, serialize_vip, serialize_vip_entries,
};

// ---------------------------------------------------------------------------
// sendBasicData — opcode 0x9F
// ---------------------------------------------------------------------------

#[test]
fn parity_basic_data_premium_short_spell_list() {
    // Premium player, vocation 4, magic-shield on, 3-entry spell list.
    let mut out = OutputMessage::new();
    serialize_basic_data(
        &mut out,
        /* is_premium     */ true,
        /* premium_end    */ 0x12345678,
        /* vocation_id    */ 4,
        /* prey_enabled   */ false,
        /* spell_ids      */ &[0x0001, 0x0002, 0x0003],
        /* magic_shield   */ true,
    );

    // C++ protocolgame.cpp:1751 sendBasicData
    //   opcode | premium=1 | premium_end u32 LE | vocation u8
    //   prey=0 | spell_count u16=3 | 3 × u16 spell ids | magic_shield=1
    expected_bytes!(
        out,
        &[
            0x9F, 0x01, 0x78, 0x56, 0x34, 0x12, 0x04, 0x00, 0x03, 0x00, 0x01, 0x00, 0x02, 0x00,
            0x03, 0x00, 0x01,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendStats / AddPlayerStats — opcode 0xA0
// ---------------------------------------------------------------------------

#[test]
fn parity_stats_full_payload() {
    let mut out = OutputMessage::new();
    serialize_stats(
        &mut out,
        /* hp                    */ 100,
        /* hp_max                */ 200,
        /* free_capacity         */ 500,
        /* experience            */ 0x0102_0304_0506_0708,
        /* level                 */ 8,
        /* level_pct             */ 50,
        /* exp_display           */ 100,
        /* low_level_bonus       */ 0,
        /* store_exp_bonus       */ 0,
        /* stamina_bonus         */ 0,
        /* mp                    */ 60,
        /* mp_max                */ 120,
        /* soul                  */ 5,
        /* stamina_minutes       */ 2520,
        /* base_speed_half       */ 110,
        /* regen_seconds         */ 60,
        /* offline_train_minutes */ 0,
        /* mana_shield           */ 0,
        /* mana_shield_max       */ 0,
    );

    // C++ protocolgame.cpp:3499 AddPlayerStats (invoked from sendStats at 1708).
    //   opcode 0xA0
    //   hp u32 | hp_max u32 | free_capacity u32 | experience u64
    //   level u16 | level_pct u8
    //   exp_display | low_level_bonus | store_exp_bonus | stamina_bonus (4 × u16)
    //   mp u32 | mp_max u32 | soul u8 | stamina u16 | base_speed/2 u16
    //   regen u16 | offline_train_minutes u16
    //   xp_boost_time u16=0 | xp_boost_in_store u8=0
    //   mana_shield u32 | mana_shield_max u32
    expected_bytes!(
        out,
        &[
            0xA0, 0x64, 0x00, 0x00, 0x00, 0xC8, 0x00, 0x00, 0x00, 0xF4, 0x01, 0x00, 0x00, 0x08,
            0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x08, 0x00, 0x32, 0x64, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x05, 0xD8,
            0x09, 0x6E, 0x00, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendSkills / AddPlayerSkills — opcode 0xA1
// ---------------------------------------------------------------------------

#[test]
fn parity_skills_full_payload() {
    let mut out = OutputMessage::new();
    serialize_skills(
        &mut out,
        /* magic (level, base, base_loyalty, pct) */ (10, 8, 8, 50),
        /* 7 skill rows (FIST..FISHING)           */
        &[
            (11, 10, 10, 25),
            (12, 11, 11, 30),
            (13, 12, 12, 35),
            (14, 13, 13, 40),
            (15, 14, 14, 45),
            (16, 15, 15, 50),
            (17, 16, 16, 55),
        ],
        /* 6 special skill rows (critical/leech)  */
        &[(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        /* capacity                                */ 12000,
    );

    // C++ protocolgame.cpp:3542 AddPlayerSkills (invoked from sendSkills at 2526).
    //   opcode 0xA1
    //   magic block: 4 × u16
    //   7 skill rows × (4 × u16)
    //   6 special skill rows × (2 × u16)
    //   u8 element ml count = 0
    //   3 × (2 × u16) fatal/dodge/momentum = 0
    //   capacity u32 (twice)
    expected_bytes!(
        out,
        &[
            0xA1, // magic: level=10, base=8, base_loyalty=8, pct=50
            0x0A, 0x00, 0x08, 0x00, 0x08, 0x00, 0x32, 0x00, // FIST
            0x0B, 0x00, 0x0A, 0x00, 0x0A, 0x00, 0x19, 0x00, // CLUB
            0x0C, 0x00, 0x0B, 0x00, 0x0B, 0x00, 0x1E, 0x00, // SWORD
            0x0D, 0x00, 0x0C, 0x00, 0x0C, 0x00, 0x23, 0x00, // AXE
            0x0E, 0x00, 0x0D, 0x00, 0x0D, 0x00, 0x28, 0x00, // DISTANCE
            0x0F, 0x00, 0x0E, 0x00, 0x0E, 0x00, 0x2D, 0x00, // SHIELD
            0x10, 0x00, 0x0F, 0x00, 0x0F, 0x00, 0x32, 0x00, // FISHING
            0x11, 0x00, 0x10, 0x00, 0x10, 0x00, 0x37, 0x00,
            // 6 × special (all zeros, 4 bytes each)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // element ml count
            0x00, // fatal/dodge/momentum (3 × 4 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // capacity 12000 = 0x2EE0 (twice)
            0xE0, 0x2E, 0x00, 0x00, 0xE0, 0x2E, 0x00, 0x00,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendIcons — opcode 0xA2
// ---------------------------------------------------------------------------

#[test]
fn parity_icons() {
    let mut out = OutputMessage::new();
    serialize_icons(&mut out, 0xCAFE_BABE);

    // C++ protocolgame.cpp:1892 sendIcons
    //   opcode 0xA2 | icons u32 LE
    expected_bytes!(out, &[0xA2, 0xBE, 0xBA, 0xFE, 0xCA],);
}

// ---------------------------------------------------------------------------
// sendClientFeatures — opcode 0x17
// ---------------------------------------------------------------------------

#[test]
fn parity_client_features() {
    let mut out = OutputMessage::new();
    // Use trivially-computable doubles so the expected bytes can be
    // verified by hand:
    //   speed_a =  0.0 → scaled =        0 + 2147483647 = 0x7FFFFFFF
    //   speed_b =  1.0 → scaled =     1000 + 2147483647 = 0x800003E7
    //   speed_c = -1.0 → scaled =    -1000 + 2147483647 = 0x7FFFFC17
    serialize_client_features(
        &mut out,
        /* player_id       */ 0x1000_0001,
        /* speed_a         */ 0.0,
        /* speed_b         */ 1.0,
        /* speed_c         */ -1.0,
        /* can_report_bugs */ true,
    );

    // C++ protocolgame.cpp:1724 sendClientFeatures
    //   opcode 0x17 | player_id u32 | beat=50 u16
    //   addDouble(speed_a, 3) | addDouble(speed_b, 3) | addDouble(speed_c, 3)
    //   can_report_bugs u8 | 0x00 | 0x00 (PvP, expert mode)
    //   0x0000 (store images url) | 25 (premium coin package size)
    //   0x00 | 0x00 (exiva, tournament)
    expected_bytes!(
        out,
        &[
            0x17, // player_id LE
            0x01, 0x00, 0x00, 0x10, // beat duration u16 = 50
            0x32, 0x00, // speed_a double: precision=3, scaled=0x7FFFFFFF
            0x03, 0xFF, 0xFF, 0xFF, 0x7F,
            // speed_b double: precision=3, scaled=0x800003E7
            0x03, 0xE7, 0x03, 0x00, 0x80,
            // speed_c double: precision=3, scaled=0x7FFFFC17
            0x03, 0x17, 0xFC, 0xFF, 0x7F, // can_report_bugs=1
            0x01, // PvP framing | expert mode
            0x00, 0x00, // store images url u16=0 | premium coin pkg u16=25
            0x00, 0x00, 0x19, 0x00, // exiva | tournament
            0x00, 0x00,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendStoreBalance — opcode 0xDF
// ---------------------------------------------------------------------------

#[test]
fn parity_store_balance() {
    let mut out = OutputMessage::new();
    serialize_store_balance(
        &mut out,
        /* store_coins         */ 0x0000_00FF,
        /* transferable_coins  */ 0x0000_00AA,
        /* auction_coins       */ 0x0000_0055,
        /* tournament_coins    */ 0x0000_0011,
    );

    // C++ protocolgame.cpp:2081 sendStoreBalance
    //   opcode 0xDF | 0x01 | total u32 | transferable u32 | auction u32 | tournament u32
    expected_bytes!(
        out,
        &[
            0xDF, 0x01, 0xFF, 0x00, 0x00, 0x00, 0xAA, 0x00, 0x00, 0x00, 0x55, 0x00, 0x00, 0x00,
            0x11, 0x00, 0x00, 0x00,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendVIPEntries — sequence of sendVIP (opcode 0xD2)
// ---------------------------------------------------------------------------

#[test]
fn parity_vip_entries_two_rows() {
    let mut out = OutputMessage::new();
    serialize_vip_entries(
        &mut out,
        &[
            (0x0000_0001, "Bob", "Friend", 3, true, 0x01),
            (0x0000_0002, "Sue", "", 99, false, 0x00),
        ],
    );

    // C++ protocolgame.cpp:3270 sendVIPEntries → loops sendVIP (3255):
    //   per entry:
    //     opcode 0xD2 | guid u32 | name (u16 len + bytes) | description (u16 len + bytes)
    //     icon u32 (capped at 10) | notify u8 | status u8 | 0x00 (vipGroups)
    expected_bytes!(
        out,
        &[
            // Row 1 — Bob
            0xD2, 0x01, 0x00, 0x00, 0x00, // "Bob": u16 len=3 + 'B','o','b'
            0x03, 0x00, b'B', b'o', b'b', // "Friend": u16 len=6 + bytes
            0x06, 0x00, b'F', b'r', b'i', b'e', b'n', b'd', // icon u32=3
            0x03, 0x00, 0x00, 0x00, // notify=1, status=0x01, groups=0x00
            0x01, 0x01, 0x00, // Row 2 — Sue (icon 99 capped at 10)
            0xD2, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, b'S', b'u', b'e', 0x00, 0x00, 0x0A, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    );
}

// ---------------------------------------------------------------------------
// Session 2 — Stub-promotion byte-parity tests
//
// The 8 functions below return `Vec<u8>` (legacy Tier 3 style) rather than
// taking `&mut OutputMessage`.  They were audited in Session 2:
//   * 7 were already byte-correct vs the C++ source — tests below assert
//     that and freeze the layout.
//   * 1 (`serialize_change_speed`) had parameter-name ambiguity around
//     "halve before passing"; renamed to `*_half` to match Session 1's
//     `base_speed_half` convention.  Bytes unchanged.
//   * `serialize_ping` / `serialize_ping_back` are legitimately one
//     opcode byte in C++ — full parity is a single-byte assert.
//
// Tests use `crate::fixtures::assert_bytes_eq` directly because the
// `expected_bytes!` macro is tied to the `OutputMessage` API.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// sendExperienceTracker — opcode 0xAF
// ---------------------------------------------------------------------------

#[test]
fn parity_experience_tracker() {
    // raw_exp = 0x0807_0605_0403_0201 → LE bytes 01 02 03 04 05 06 07 08
    // final_exp = -1 (i64) → LE bytes FF FF FF FF FF FF FF FF
    let bytes = serialize_experience_tracker(0x0807_0605_0403_0201, -1);

    // C++ protocolgame.cpp:1715 sendExperienceTracker
    //   opcode 0xAF | raw_exp i64 LE | final_exp i64 LE
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xAF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF,
        ],
        "protocolgame.cpp:1715 sendExperienceTracker",
    );
}

// ---------------------------------------------------------------------------
// sendChangeSpeed — opcode 0x8F
// ---------------------------------------------------------------------------

#[test]
fn parity_change_speed() {
    // creature_id = 0x1234_5678; base_speed_half = 110 (caller already
    // halved 220); new_speed_half = 150 (caller halved 300).
    let bytes = serialize_change_speed(0x1234_5678, 110, 150);

    // C++ protocolgame.cpp:2508 sendChangeSpeed
    //   opcode 0x8F | creature_id u32 LE | base_speed/2 u16 LE | speed/2 u16 LE
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0x8F, 0x78, 0x56, 0x34, 0x12, 0x6E, 0x00, // 110
            0x96, 0x00, // 150
        ],
        "protocolgame.cpp:2508 sendChangeSpeed",
    );
}

// ---------------------------------------------------------------------------
// sendCancelWalk — opcode 0xB5
// ---------------------------------------------------------------------------

#[test]
fn parity_cancel_walk() {
    // direction = 2 (DIRECTION_SOUTH in tibian Direction_t enum)
    let bytes = serialize_cancel_walk(2);

    // C++ protocolgame.cpp:2518 sendCancelWalk
    //   opcode 0xB5 | direction u8 (from player->getDirection())
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0xB5, 0x02],
        "protocolgame.cpp:2518 sendCancelWalk",
    );
}

// ---------------------------------------------------------------------------
// sendCancelTarget — opcode 0xA3
// ---------------------------------------------------------------------------

#[test]
fn parity_cancel_target() {
    let bytes = serialize_cancel_target();

    // C++ protocolgame.cpp:2500 sendCancelTarget
    //   opcode 0xA3 | u32 literal 0x00000000 (target creature id placeholder)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0xA3, 0x00, 0x00, 0x00, 0x00],
        "protocolgame.cpp:2500 sendCancelTarget",
    );
}

// ---------------------------------------------------------------------------
// sendVIP — opcode 0xD2
// ---------------------------------------------------------------------------

#[test]
fn parity_vip_single_row() {
    // guid=0x0000_0001 "Alice" / "Best friend" / icon=3 (no cap) / notify=true / status=0x01
    let bytes = serialize_vip(0x0000_0001, "Alice", "Best friend", 3, true, 0x01);

    // C++ protocolgame.cpp:3255 sendVIP
    //   opcode 0xD2 | guid u32 | name (u16 len + bytes) | description (u16 len + bytes)
    //   | icon u32 (min(10, icon)) | notify u8 | status u8 | 0x00 (vipGroups placeholder)
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[
            0xD2, 0x01, 0x00, 0x00, 0x00, // "Alice": u16 len=5
            0x05, 0x00, b'A', b'l', b'i', b'c', b'e', // "Best friend": u16 len=11
            0x0B, 0x00, b'B', b'e', b's', b't', b' ', b'f', b'r', b'i', b'e', b'n', b'd',
            // icon u32=3
            0x03, 0x00, 0x00, 0x00, // notify=1, status=0x01, vipGroups=0x00
            0x01, 0x01, 0x00,
        ],
        "protocolgame.cpp:3255 sendVIP",
    );
}

// ---------------------------------------------------------------------------
// sendUpdatedVIPStatus — opcode 0xD3
// ---------------------------------------------------------------------------

#[test]
fn parity_updated_vip_status() {
    // guid = 0xDEAD_BEEF, new_status = 0x02 (VIPSTATUS_PENDING)
    let bytes = serialize_updated_vip_status(0xDEAD_BEEF, 0x02);

    // C++ protocolgame.cpp:3246 sendUpdatedVIPStatus
    //   opcode 0xD3 | guid u32 LE | new_status u8
    crate::fixtures::assert_bytes_eq(
        &bytes,
        &[0xD3, 0xEF, 0xBE, 0xAD, 0xDE, 0x02],
        "protocolgame.cpp:3246 sendUpdatedVIPStatus",
    );
}

// ---------------------------------------------------------------------------
// sendPing — opcode 0x1D  (legitimately one byte; C++ has no payload)
// ---------------------------------------------------------------------------

#[test]
fn parity_ping_opcode_only() {
    let bytes = serialize_ping();

    // C++ protocolgame.cpp:2533 sendPing
    //   opcode 0x1D — no payload
    crate::fixtures::assert_bytes_eq(&bytes, &[0x1D], "protocolgame.cpp:2533 sendPing");
}

// ---------------------------------------------------------------------------
// sendPingBack — opcode 0x1E  (legitimately one byte; C++ has no payload)
// ---------------------------------------------------------------------------

#[test]
fn parity_ping_back_opcode_only() {
    let bytes = serialize_ping_back();

    // C++ protocolgame.cpp:2540 sendPingBack
    //   opcode 0x1E — no payload
    crate::fixtures::assert_bytes_eq(&bytes, &[0x1E], "protocolgame.cpp:2540 sendPingBack");
}
