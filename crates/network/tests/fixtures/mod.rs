//! Reusable fixture builders for `wire_parity` integration tests.
//!
//! All helpers here are deterministic so test byte literals stay stable.
//! Each builder mirrors a C++ data shape and is meant to be used from a
//! `tests/wire_parity.rs` per-cluster module (`container`, `creature`,
//! `player`, etc.).

#![allow(dead_code)]

use forgottenserver_common::networkmessage::{ItemTypeMeta, PodiumMeta};

/// Builds a minimal `ItemTypeMeta` for byte-parity tests.
///
/// Only `client_id` and `stackable` are configurable; every other flag
/// defaults to `false` / zero so each test's expected byte literal stays
/// short and unambiguous.  Pass-through to C++ `NetworkMessage::addItem`
/// follows the same defaults when an `ItemType` has none of the optional
/// fields set.
pub fn make_item_type_meta(client_id: u16, stackable: bool) -> ItemTypeMeta {
    ItemTypeMeta {
        client_id,
        stackable,
        is_splash: false,
        is_fluid_container: false,
        is_container: false,
        classification: 0,
        show_client_charges: false,
        show_client_duration: false,
        is_podium: false,
        weapon_type: 0,
        charges: 0,
        decay_time_min: 0,
    }
}

/// Builds an empty/visible podium meta block.
///
/// Mirrors a podium item with no outfit/mount selection: the C++ side
/// writes the literal `0x0000, 0x0000, 0x02, 0x01` block, matching the
/// defaults below.
pub fn make_podium_meta(direction: u8, _visible: bool) -> PodiumMeta {
    PodiumMeta {
        show_outfit: false,
        show_mount: false,
        show_platform: false,
        look_type: 0,
        look_head: 0,
        look_body: 0,
        look_legs: 0,
        look_feet: 0,
        look_addons: 0,
        look_mount: 0,
        look_mount_head: 0,
        look_mount_body: 0,
        look_mount_legs: 0,
        look_mount_feet: 0,
        direction,
    }
}

/// Outfit-shaped tuple builder for `sendCreatureOutfit` parity tests.
///
/// Returned tuple matches the wire-layout order:
/// `(look_type, look_head, look_body, look_legs, look_feet, look_addons,
///   look_mount, look_mount_head, look_mount_body, look_mount_legs,
///   look_mount_feet)`.
pub fn make_outfit_t(look_type: u16) -> (u16, u8, u8, u8, u8, u8, u16, u8, u8, u8, u8) {
    (look_type, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
}

/// Asserts two byte slices are equal and pretty-prints the offset of the
/// first divergence on failure.
///
/// Helps locate single-byte regressions in long wire packets without
/// scrolling through opaque "left/right" assert dumps.
pub fn assert_bytes_eq(actual: &[u8], expected: &[u8], cpp_citation: &str) {
    if actual == expected {
        return;
    }
    let mut diff_offset: Option<usize> = None;
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        if a != e {
            diff_offset = Some(i);
            break;
        }
    }
    let offset = diff_offset.unwrap_or_else(|| actual.len().min(expected.len()));
    panic!(
        "wire-parity mismatch ({cpp_citation})\n  first diff at offset {offset:#x}\n  actual   ({len_a:>3}): {actual:02x?}\n  expected ({len_e:>3}): {expected:02x?}",
        cpp_citation = cpp_citation,
        offset = offset,
        len_a = actual.len(),
        len_e = expected.len(),
        actual = actual,
        expected = expected,
    );
}

/// Compares an `OutputMessage` payload against a hex literal annotated
/// with its originating C++ source line for traceability.
///
/// Usage:
/// ```ignore
/// let mut out = OutputMessage::new();
/// serialize_icons(&mut out, 0x0000_BEEF);
/// expected_bytes!(
///     out,
///     // C++ protocolgame.cpp:1892 sendIcons: opcode 0xA2 + u32 LE icons
///     &[0xA2, 0xEF, 0xBE, 0x00, 0x00],
/// );
/// ```
#[macro_export]
#[allow(clippy::crate_in_macro_def)]
macro_rules! expected_bytes {
    ($out:expr, $expected:expr $(,)?) => {{
        $out.write_message_length();
        let __payload = &$out.get_output_buffer()[2..];
        crate::fixtures::assert_bytes_eq(__payload, $expected, concat!(file!(), ":", line!()));
    }};
}
