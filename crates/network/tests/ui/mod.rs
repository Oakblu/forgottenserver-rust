//! Byte-parity tests for the `wire-ui` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs`.
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  Tests cover all UI / chat packets:
//!   * `sendOpenPrivateChannel`    (opcode 0xAD)  C++ 1575
//!   * `sendChannelEvent`          (opcode 0xF3)  C++ 1583
//!   * `sendCreatePrivateChannel`  (opcode 0xB2)  C++ 1822
//!   * `sendClosePrivate`          (opcode 0xB3)  C++ 1814
//!   * `sendFYIBox`                (opcode 0x15)  C++ 2590
//!   * `sendChannelsDialog`        (opcode 0xAB)  C++ 1834
//!   * `sendChannel`               (opcode 0xAC)  C++ 1849
//!   * `sendChannelMessage`        (opcode 0xAA)  C++ 1878
//!   * `sendToChannel`             (opcode 0xAA)  C++ 2452
//!   * `sendPrivateMessage`        (opcode 0xAA)  C++ 2481
//!   * `sendTextMessage`           (opcode 0xB4)  C++ 1776
//!   * `sendModalWindow`           (opcode 0xFA)  C++ 3351
//!   * `sendTextWindow` (writable) (opcode 0x96)  C++ 2936
//!   * `sendTextWindow` (readonly) (opcode 0x96)  C++ 2971
//!   * `sendHouseWindow`           (opcode 0x97)  C++ 2985
//!   * `sendOutfitWindow`          (opcode 0xC8)  C++ 3019
//!   * `sendPodiumWindow`          (opcode 0xC8)  C++ 3110

use forgottenserver_network::protocolgame::{
    serialize_channel, serialize_channel_event, serialize_channel_message,
    serialize_channels_dialog, serialize_close_private, serialize_create_private_channel,
    serialize_fyi_box, serialize_house_window, serialize_modal_window,
    serialize_open_private_channel, serialize_outfit_window, serialize_podium_window,
    serialize_private_message, serialize_text_message, serialize_text_window_readonly,
    serialize_text_window_writable, serialize_to_channel, text_message_class, OutfitDescriptor,
};

use crate::fixtures::{assert_bytes_eq, make_item_type_meta};

// ---------------------------------------------------------------------------
// sendOpenPrivateChannel — opcode 0xAD  (C++ protocolgame.cpp:1575)
// ---------------------------------------------------------------------------

#[test]
fn parity_open_private_channel() {
    let bytes = serialize_open_private_channel("Bob");

    // C++ protocolgame.cpp:1575 sendOpenPrivateChannel
    //   opcode 0xAD | receiver (u16 len + bytes)
    assert_bytes_eq(
        &bytes,
        &[0xAD, 0x03, 0x00, b'B', b'o', b'b'],
        "protocolgame.cpp:1575 sendOpenPrivateChannel",
    );
}

// ---------------------------------------------------------------------------
// sendChannelEvent — opcode 0xF3  (C++ protocolgame.cpp:1583)
// ---------------------------------------------------------------------------

#[test]
fn parity_channel_event() {
    let bytes = serialize_channel_event(7, "Alice", 2);

    // C++ protocolgame.cpp:1583 sendChannelEvent
    //   opcode 0xF3 | channel_id u16 | player_name (u16 len + bytes) | event u8
    assert_bytes_eq(
        &bytes,
        &[
            0xF3, 0x07, 0x00, 0x05, 0x00, b'A', b'l', b'i', b'c', b'e', 0x02,
        ],
        "protocolgame.cpp:1583 sendChannelEvent",
    );
}

// ---------------------------------------------------------------------------
// sendCreatePrivateChannel — opcode 0xB2  (C++ protocolgame.cpp:1822)
// ---------------------------------------------------------------------------

#[test]
fn parity_create_private_channel() {
    let bytes = serialize_create_private_channel(0x000A, "My Chan", "Owner");

    // C++ protocolgame.cpp:1822 sendCreatePrivateChannel
    //   opcode 0xB2 | channel_id u16 | channel_name (u16 len + bytes)
    //   u16 0x0001 (users count) | owner_name (u16 len + bytes)
    //   u16 0x0000 (invited count)
    assert_bytes_eq(
        &bytes,
        &[
            0xB2, 0x0A, 0x00, 0x07, 0x00, b'M', b'y', b' ', b'C', b'h', b'a', b'n', 0x01, 0x00,
            0x05, 0x00, b'O', b'w', b'n', b'e', b'r', 0x00, 0x00,
        ],
        "protocolgame.cpp:1822 sendCreatePrivateChannel",
    );
}

// ---------------------------------------------------------------------------
// sendClosePrivate — opcode 0xB3  (C++ protocolgame.cpp:1814)
// ---------------------------------------------------------------------------

#[test]
fn parity_close_private() {
    let bytes = serialize_close_private(0x00FE);

    // C++ protocolgame.cpp:1814 sendClosePrivate
    //   opcode 0xB3 | channel_id u16
    assert_bytes_eq(
        &bytes,
        &[0xB3, 0xFE, 0x00],
        "protocolgame.cpp:1814 sendClosePrivate",
    );
}

// ---------------------------------------------------------------------------
// sendFYIBox — opcode 0x15  (C++ protocolgame.cpp:2590)
// ---------------------------------------------------------------------------

#[test]
fn parity_fyi_box() {
    let bytes = serialize_fyi_box("Hi");

    // C++ protocolgame.cpp:2590 sendFYIBox
    //   opcode 0x15 | message (u16 len + bytes)
    assert_bytes_eq(
        &bytes,
        &[0x15, 0x02, 0x00, b'H', b'i'],
        "protocolgame.cpp:2590 sendFYIBox",
    );
}

// ---------------------------------------------------------------------------
// sendChannelsDialog — opcode 0xAB  (C++ protocolgame.cpp:1834)
// ---------------------------------------------------------------------------

#[test]
fn parity_channels_dialog_two_channels() {
    let bytes = serialize_channels_dialog(&[(0x0001, "Help"), (0x0002, "Trade")]);

    // C++ protocolgame.cpp:1834 sendChannelsDialog
    //   opcode 0xAB | count u8 | per entry: channel_id u16 + name (u16 len + bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xAB, 0x02, // Help
            0x01, 0x00, 0x04, 0x00, b'H', b'e', b'l', b'p', // Trade
            0x02, 0x00, 0x05, 0x00, b'T', b'r', b'a', b'd', b'e',
        ],
        "protocolgame.cpp:1834 sendChannelsDialog",
    );
}

// ---------------------------------------------------------------------------
// sendChannel — opcode 0xAC  (C++ protocolgame.cpp:1849)
// ---------------------------------------------------------------------------

#[test]
fn parity_channel_with_users_no_invited() {
    let bytes = serialize_channel(0x0005, "Guild", &["Bob", "Sue"], &[]);

    // C++ protocolgame.cpp:1849 sendChannel
    //   opcode 0xAC | channel_id u16 | name (u16 len + bytes)
    //   users.len() u16 | per user: name (u16 len + bytes)
    //   invited.len() u16 | per user: name (u16 len + bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xAC, 0x05, 0x00, 0x05, 0x00, b'G', b'u', b'i', b'l', b'd', // 2 users
            0x02, 0x00, 0x03, 0x00, b'B', b'o', b'b', 0x03, 0x00, b'S', b'u', b'e',
            // 0 invited
            0x00, 0x00,
        ],
        "protocolgame.cpp:1849 sendChannel",
    );
}

#[test]
fn parity_channel_empty_lists() {
    // Matches the C++ `nullptr` branch on both lists: both counts = 0.
    let bytes = serialize_channel(0x0003, "Help", &[], &[]);

    // C++ protocolgame.cpp:1849 sendChannel (null branches)
    assert_bytes_eq(
        &bytes,
        &[
            0xAC, 0x03, 0x00, 0x04, 0x00, b'H', b'e', b'l', b'p', 0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:1849 sendChannel (null users/invited)",
    );
}

// ---------------------------------------------------------------------------
// sendChannelMessage — opcode 0xAA  (C++ protocolgame.cpp:1878)
// ---------------------------------------------------------------------------

#[test]
fn parity_channel_message() {
    let bytes = serialize_channel_message(
        "System", "Hi", /* speak_class */ 0x05, /* channel_id  */ 0x000A,
    );

    // C++ protocolgame.cpp:1878 sendChannelMessage
    //   opcode 0xAA | stmt_id u32=0 | author (u16+bytes) | level u16=0
    //   speak_class u8 | channel_id u16 | text (u16+bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xAA, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, b'S', b'y', b's', b't', b'e', b'm', 0x00,
            0x00, 0x05, 0x0A, 0x00, 0x02, 0x00, b'H', b'i',
        ],
        "protocolgame.cpp:1878 sendChannelMessage",
    );
}

// ---------------------------------------------------------------------------
// sendToChannel — opcode 0xAA  (C++ protocolgame.cpp:2452)
// ---------------------------------------------------------------------------

#[test]
fn parity_to_channel_with_creature() {
    let bytes = serialize_to_channel(
        /* stmt_id      */ 0x1234_5678,
        /* creature     */ "Bob",
        /* level        */ 50,
        /* speak_class  */ 0x03,
        /* channel_id   */ 0x000A,
        /* text         */ "Hi",
    );

    // C++ protocolgame.cpp:2452 sendToChannel (creature branch)
    //   opcode 0xAA | stmt_id u32 | name (u16+bytes) | traded u8=0
    //   level u16 | speak_class u8 | channel u16 | text (u16+bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xAA, 0x78, 0x56, 0x34, 0x12, 0x03, 0x00, b'B', b'o', b'b', 0x00, 0x32, 0x00, 0x03,
            0x0A, 0x00, 0x02, 0x00, b'H', b'i',
        ],
        "protocolgame.cpp:2452 sendToChannel (creature branch)",
    );
}

#[test]
fn parity_to_channel_no_creature() {
    let bytes = serialize_to_channel(
        /* stmt_id      */ 1, /* creature     */ "", /* level        */ 0,
        /* speak_class  */ 0x01, /* channel_id   */ 0x0003, /* text         */ "Hi",
    );

    // C++ protocolgame.cpp:2452 sendToChannel (null branch)
    //   opcode 0xAA | stmt_id u32 | u32 0 | traded u8=0
    //   speak_class u8 | channel u16 | text (u16+bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xAA, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x03, 0x00, 0x02,
            0x00, b'H', b'i',
        ],
        "protocolgame.cpp:2452 sendToChannel (null branch)",
    );
}

// ---------------------------------------------------------------------------
// sendPrivateMessage — opcode 0xAA  (C++ protocolgame.cpp:2481)
// ---------------------------------------------------------------------------

#[test]
fn parity_private_message_with_speaker() {
    let bytes = serialize_private_message(
        /* stmt_id      */ 0xAABBCCDD, /* speaker      */ "Sue",
        /* speaker_lvl  */ 100, /* speak_class  */ 0x04, /* text         */ "Hi",
    );

    // C++ protocolgame.cpp:2481 sendPrivateMessage (speaker branch)
    //   opcode 0xAA | stmt_id u32 | name (u16+bytes) | traded u8=0
    //   level u16 | speak_class u8 | text (u16+bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xAA, 0xDD, 0xCC, 0xBB, 0xAA, 0x03, 0x00, b'S', b'u', b'e', 0x00, 0x64, 0x00, 0x04,
            0x02, 0x00, b'H', b'i',
        ],
        "protocolgame.cpp:2481 sendPrivateMessage (speaker branch)",
    );
}

#[test]
fn parity_private_message_no_speaker() {
    let bytes = serialize_private_message(
        /* stmt_id      */ 7, /* speaker      */ "", /* speaker_lvl  */ 0,
        /* speak_class  */ 0x02, /* text         */ "Hi",
    );

    // C++ protocolgame.cpp:2481 sendPrivateMessage (null branch)
    //   opcode 0xAA | stmt_id u32 | u32 0 | traded u8=0
    //   speak_class u8 | text (u16+bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xAA, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x02, 0x00, b'H',
            b'i',
        ],
        "protocolgame.cpp:2481 sendPrivateMessage (null branch)",
    );
}

// ---------------------------------------------------------------------------
// sendTextMessage — opcode 0xB4  (C++ protocolgame.cpp:1776)
// ---------------------------------------------------------------------------

#[test]
fn parity_text_message_damage_dealt() {
    // MESSAGE_DAMAGE_DEALT = 23 → triggers position + primary + secondary.
    let bytes = serialize_text_message(
        text_message_class::MESSAGE_DAMAGE_DEALT,
        "Ouch",
        Some((0x0064, 0x00C8, 0x07)),
        Some(0x0A),
        Some(0x0000_0050),
        Some(0x0B),
        Some(0x0000_0019),
        None,
    );

    // C++ protocolgame.cpp:1776 sendTextMessage (damage branch)
    //   opcode 0xB4 | type u8 | pos (5 bytes)
    //   primary u32 + u8 color | secondary u32 + u8 color
    //   text (u16+bytes)
    assert_bytes_eq(
        &bytes,
        &[
            0xB4, 23, 0x64, 0x00, 0xC8, 0x00, 0x07, 0x50, 0x00, 0x00, 0x00, 0x0A, 0x19, 0x00, 0x00,
            0x00, 0x0B, 0x04, 0x00, b'O', b'u', b'c', b'h',
        ],
        "protocolgame.cpp:1776 sendTextMessage (damage)",
    );
}

#[test]
fn parity_text_message_healed() {
    // MESSAGE_HEALED = 25 → position + primary only.
    let bytes = serialize_text_message(
        text_message_class::MESSAGE_HEALED,
        "Heal",
        Some((0x000A, 0x000B, 0x07)),
        Some(0x05),
        Some(0x0000_0010),
        None,
        None,
        None,
    );

    // C++ protocolgame.cpp:1776 sendTextMessage (heal/experience branch)
    assert_bytes_eq(
        &bytes,
        &[
            0xB4, 25, 0x0A, 0x00, 0x0B, 0x00, 0x07, 0x10, 0x00, 0x00, 0x00, 0x05, 0x04, 0x00, b'H',
            b'e', b'a', b'l',
        ],
        "protocolgame.cpp:1776 sendTextMessage (healed)",
    );
}

#[test]
fn parity_text_message_guild() {
    // MESSAGE_GUILD = 33 → channel_id only (no position).
    let bytes = serialize_text_message(
        text_message_class::MESSAGE_GUILD,
        "Hi",
        None,
        None,
        None,
        None,
        None,
        Some(0x0042),
    );

    // C++ protocolgame.cpp:1776 sendTextMessage (guild/party branch)
    assert_bytes_eq(
        &bytes,
        &[0xB4, 33, 0x42, 0x00, 0x02, 0x00, b'H', b'i'],
        "protocolgame.cpp:1776 sendTextMessage (guild)",
    );
}

#[test]
fn parity_text_message_default_text_only() {
    // type = 0 (no branch matches) → only the text is appended.
    let bytes =
        serialize_text_message(/* class */ 0, "Hi", None, None, None, None, None, None);

    // C++ protocolgame.cpp:1776 sendTextMessage (default branch)
    assert_bytes_eq(
        &bytes,
        &[0xB4, 0x00, 0x02, 0x00, b'H', b'i'],
        "protocolgame.cpp:1776 sendTextMessage (default)",
    );
}

// ---------------------------------------------------------------------------
// sendModalWindow — opcode 0xFA  (C++ protocolgame.cpp:3351)
// ---------------------------------------------------------------------------

#[test]
fn parity_modal_window() {
    let bytes = serialize_modal_window(
        /* id     */ 0x0000_0001,
        /* title  */ "T",
        /* msg    */ "M",
        /* btns   */ &[("OK", 1), ("X", 2)],
        /* chs    */ &[("A", 10)],
        /* enter  */ 1,
        /* escape */ 2,
        /* prio   */ true,
    );

    // C++ protocolgame.cpp:3351 sendModalWindow
    //   opcode 0xFA | id u32 | title (str) | msg (str)
    //   btns.size() u8 + per btn (text str + id u8)
    //   chs.size() u8 + per ch (text str + id u8)
    //   default_escape u8 | default_enter u8 | priority u8
    assert_bytes_eq(
        &bytes,
        &[
            0xFA, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, b'T', 0x01, 0x00, b'M', 0x02, 0x02, 0x00,
            b'O', b'K', 0x01, 0x01, 0x00, b'X', 0x02, 0x01, 0x01, 0x00, b'A', 0x0A,
            // default_escape, default_enter, priority
            0x02, 0x01, 0x01,
        ],
        "protocolgame.cpp:3351 sendModalWindow",
    );
}

// ---------------------------------------------------------------------------
// sendTextWindow (writable overload) — opcode 0x96  (C++ protocolgame.cpp:2936)
// ---------------------------------------------------------------------------

#[test]
fn parity_text_window_writable() {
    // Use a plain item (client_id 1234, non-stackable) → addItem writes
    // only the u16 client_id (no sub-type bytes).  can_write=true means
    // the maxlen is written, not text.len().
    let item = make_item_type_meta(1234, false);
    let bytes = serialize_text_window_writable(
        /* window_id */ 0x000A,
        /* item meta */ item,
        /* maxlen    */ 100,
        /* text      */ "Hi",
        /* can_write */ true,
        /* writer    */ Some("Bob"),
        /* date      */ Some("01 Jan"),
    );

    // C++ protocolgame.cpp:2936 sendTextWindow (writable branch)
    //   opcode 0x96 | window_id u32 | addItem (u16 client_id)
    //   maxlen u16 | text (str)
    //   writer (str) | traded u8=0 | date (str)
    assert_bytes_eq(
        &bytes,
        &[
            0x96, 0x0A, 0x00, 0x00, 0x00, 0xD2, 0x04, // client_id 1234 = 0x04D2
            0x64, 0x00, // maxlen 100
            0x02, 0x00, b'H', b'i', 0x03, 0x00, b'B', b'o', b'b', 0x00, // traded
            0x06, 0x00, b'0', b'1', b' ', b'J', b'a', b'n',
        ],
        "protocolgame.cpp:2936 sendTextWindow (writable)",
    );
}

#[test]
fn parity_text_window_writable_readonly_branch_no_writer_no_date() {
    // can_write=false, no writer, no date — covers all "else" sub-branches.
    let item = make_item_type_meta(0x0500, false);
    let bytes = serialize_text_window_writable(
        /* window_id */ 1, /* item meta */ item, /* maxlen    */ 0,
        /* text      */ "Hi", /* can_write */ false, /* writer    */ None,
        /* date      */ None,
    );

    assert_bytes_eq(
        &bytes,
        &[
            0x96, 0x01, 0x00, 0x00, 0x00, 0x00, 0x05,
            // text.len() u16 = 2 (replaces maxlen branch)
            0x02, 0x00, 0x02, 0x00, b'H', b'i', // empty writer
            0x00, 0x00, // traded
            0x00, // empty date
            0x00, 0x00,
        ],
        "protocolgame.cpp:2936 sendTextWindow (writable, else branches)",
    );
}

// ---------------------------------------------------------------------------
// sendTextWindow (readonly overload) — opcode 0x96  (C++ protocolgame.cpp:2971)
// ---------------------------------------------------------------------------

#[test]
fn parity_text_window_readonly() {
    let bytes = serialize_text_window_readonly(
        /* window_id */ 0x0000_0007,
        /* client_id */ 0x1234,
        /* text      */ "Hi",
    );

    // C++ protocolgame.cpp:2971 sendTextWindow (readonly overload)
    //   opcode 0x96 | window_id u32 | client_id u16 | text.len() u16 | text bytes
    //   u16 0x0000 (writer) | u8 0 (traded) | u16 0x0000 (date)
    assert_bytes_eq(
        &bytes,
        &[
            0x96, 0x07, 0x00, 0x00, 0x00, 0x34, 0x12, 0x02, 0x00, 0x02, 0x00, b'H', b'i', 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:2971 sendTextWindow (readonly)",
    );
}

// ---------------------------------------------------------------------------
// sendHouseWindow — opcode 0x97  (C++ protocolgame.cpp:2985)
// ---------------------------------------------------------------------------

#[test]
fn parity_house_window() {
    let bytes = serialize_house_window(0x0000_0042, "Hi");

    // C++ protocolgame.cpp:2985 sendHouseWindow
    //   opcode 0x97 | 0x00 | window_id u32 | text (u16+bytes)
    assert_bytes_eq(
        &bytes,
        &[0x97, 0x00, 0x42, 0x00, 0x00, 0x00, 0x02, 0x00, b'H', b'i'],
        "protocolgame.cpp:2985 sendHouseWindow",
    );
}

// ---------------------------------------------------------------------------
// sendOutfitWindow — opcode 0xC8  (C++ protocolgame.cpp:3019)
// ---------------------------------------------------------------------------

#[test]
fn parity_outfit_window_no_mount_one_outfit() {
    // Current outfit: lookType=128, head/body/legs/feet/addons=0, lookMount=0
    // → the "no mount" branch writes the 4 mount-color bytes.
    let current = OutfitDescriptor {
        look_type: 128,
        look_head: 1,
        look_body: 2,
        look_legs: 3,
        look_feet: 4,
        look_addons: 5,
        look_type_ex: 0,
        look_mount: 0,
        look_mount_head: 10,
        look_mount_body: 20,
        look_mount_legs: 30,
        look_mount_feet: 40,
    };
    let bytes = serialize_outfit_window(
        current,
        &[(128, "Citizen", 3, 0x00)],
        &[],
        /* mounted */ false,
        /* random  */ true,
    );

    // C++ protocolgame.cpp:3019 sendOutfitWindow
    //   opcode 0xC8
    //   AddOutfit(currentOutfit) — looktype != 0 path
    //     u16 lookType | head | body | legs | feet | addons | u16 lookMount=0
    //   (lookMount == 0): 4 × u8 mount-color
    //   u16 familiar=0
    //   outfits.len() u16 + per outfit (u16 looktype, str name, u8 addons, u8 mode)
    //   mounts.len() u16
    //   u16 familiars=0
    //   u8 try-outfit | u8 mounted | u8 randomize
    assert_bytes_eq(
        &bytes,
        &[
            0xC8, 0x80, 0x00, // lookType 128
            0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x00, // lookMount 0
            // mount-color bytes (because lookMount==0)
            0x0A, 0x14, 0x1E, 0x28, 0x00, 0x00, // familiar
            0x01, 0x00, // outfits count
            0x80, 0x00, // 128
            0x07, 0x00, b'C', b'i', b't', b'i', b'z', b'e', b'n', 0x03, // addons
            0x00, // mode
            0x00, 0x00, // mounts count
            0x00, 0x00, // familiars
            0x00, // try-outfit
            0x00, // mounted=false
            0x01, // randomize_mount=true
        ],
        "protocolgame.cpp:3019 sendOutfitWindow",
    );
}

#[test]
fn parity_outfit_window_with_mount_skips_color_bytes() {
    // lookMount != 0 → the "no mount" branch (4 color bytes) is skipped.
    let current = OutfitDescriptor {
        look_type: 200,
        look_head: 0,
        look_body: 0,
        look_legs: 0,
        look_feet: 0,
        look_addons: 0,
        look_type_ex: 0,
        look_mount: 50,
        look_mount_head: 0xAA, // ignored — written by append_outfit instead
        look_mount_body: 0xBB,
        look_mount_legs: 0xCC,
        look_mount_feet: 0xDD,
    };
    let bytes = serialize_outfit_window(
        current,
        &[],
        &[(50, "Pony", 0)],
        /* mounted */ true,
        /* random  */ false,
    );

    // append_outfit on lookMount!=0 also emits the 4 mount-color bytes —
    // those bytes are written there, and the secondary `if look_mount==0`
    // block here is suppressed.
    assert_bytes_eq(
        &bytes,
        &[
            0xC8, 0xC8, 0x00, // lookType 200
            0x00, 0x00, 0x00, 0x00, 0x00, 0x32, 0x00, // lookMount 50
            0xAA, 0xBB, 0xCC, 0xDD, // mount-color via append_outfit
            // NO extra 4 mount-color bytes (lookMount != 0 branch)
            0x00, 0x00, // familiar
            0x00, 0x00, // outfits count = 0
            0x01, 0x00, // mounts count = 1
            0x32, 0x00, 0x04, 0x00, b'P', b'o', b'n', b'y', 0x00, 0x00, 0x00, // familiars
            0x00, // try-outfit
            0x01, // mounted=true
            0x00, // randomize=false
        ],
        "protocolgame.cpp:3019 sendOutfitWindow (mounted branch)",
    );
}

// ---------------------------------------------------------------------------
// sendPodiumWindow — opcode 0xC8  (C++ protocolgame.cpp:3110)
// ---------------------------------------------------------------------------

#[test]
fn parity_podium_window() {
    let podium_outfit = OutfitDescriptor {
        look_type: 128,
        look_head: 1,
        look_body: 2,
        look_legs: 3,
        look_feet: 4,
        look_addons: 5,
        look_type_ex: 0,
        look_mount: 50,
        look_mount_head: 6,
        look_mount_body: 7,
        look_mount_legs: 8,
        look_mount_feet: 9,
    };

    let bytes = serialize_podium_window(
        /* pos_x          */ 0x0064,
        /* pos_y          */ 0x00C8,
        /* pos_z          */ 0x07,
        /* stackpos       */ 0x02,
        /* item_client_id */ 0x1234,
        /* podium_outfit  */ podium_outfit,
        /* direction      */ 0x03,
        /* outfits        */ &[(128, "Citizen", 3, 0x00)],
        /* mounts         */ &[(50, "Pony", 0)],
        /* mount_visible  */ true,
        /* show_platform  */ true,
    );

    // C++ protocolgame.cpp:3110 sendPodiumWindow
    //   opcode 0xC8
    //   current outfit: u16 lookType + 5 × u8 + u16 lookMount + 4 × u8 (always)
    //   u16 familiar=0
    //   outfits.len() u16 + per outfit (u16 looktype, str name, u8 addons, u8 mode)
    //   mounts.len() u16 + per mount (u16 client_id, str name, u8 mode)
    //   u16 familiars=0
    //   u8 0x05 (podium mode)
    //   u8 mount-visible
    //   u16 unknown=0
    //   position (5 bytes)
    //   item_client_id (u16)
    //   stackpos (u8)
    //   u8 show_platform
    //   u8 0x01 (outfit checkbox, ignored)
    //   u8 direction
    assert_bytes_eq(
        &bytes,
        &[
            0xC8, // current outfit (unconditional 6+5 layout)
            0x80, 0x00, // lookType 128
            0x01, 0x02, 0x03, 0x04, 0x05, 0x32, 0x00, // lookMount 50
            0x06, 0x07, 0x08, 0x09, // familiar
            0x00, 0x00, // outfits count = 1
            0x01, 0x00, 0x80, 0x00, 0x07, 0x00, b'C', b'i', b't', b'i', b'z', b'e', b'n', 0x03,
            0x00, // mounts count = 1
            0x01, 0x00, 0x32, 0x00, 0x04, 0x00, b'P', b'o', b'n', b'y', 0x00,
            // familiars
            0x00, 0x00, // podium-mode magic
            0x05, 0x01, // mount visible
            0x00, 0x00, // unknown
            // position
            0x64, 0x00, 0xC8, 0x00, 0x07, // item client id
            0x34, 0x12, // stackpos
            0x02, // show platform
            0x01, // outfit checkbox (always 0x01)
            0x01, // direction
            0x03,
        ],
        "protocolgame.cpp:3110 sendPodiumWindow",
    );
}
