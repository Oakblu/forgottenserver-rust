//! Byte-parity tests for the `wire-container` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs`.
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  Tests cover all 6 container packets:
//!   * `sendContainer`           (opcode 0x6E)
//!   * `sendEmptyContainer`      (opcode 0x6E, hard-coded placeholder)
//!   * `sendCloseContainer`      (opcode 0x6F)
//!   * `sendAddContainerItem`    (opcode 0x70)
//!   * `sendUpdateContainerItem` (opcode 0x71)
//!   * `sendRemoveContainerItem` (opcode 0x72)

use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_network::protocolgame::{
    serialize_add_container_item, serialize_close_container, serialize_container,
    serialize_empty_container, serialize_remove_container_item, serialize_update_container_item,
};

use crate::fixtures::{assert_bytes_eq, make_item_type_meta};

// ---------------------------------------------------------------------------
// sendContainer — opcode 0x6E
// ---------------------------------------------------------------------------

#[test]
fn parity_container_bag_with_one_stackable_item() {
    // Container cid=3, a bag (client_id 1987 = 0x07C3) named "Bag",
    // capacity 8, has parent, unlocked, no pagination, contains 2 items
    // total, window starts at index 0, sending 1 visible item: 5 ropes
    // (stackable, client_id 2120 = 0x0848).
    let bag_meta = make_item_type_meta(1987, false);
    let rope_meta = make_item_type_meta(2120, true);

    let mut out = OutputMessage::new();
    serialize_container(
        &mut out,
        /* cid              */ 3,
        /* container_meta   */ bag_meta,
        /* container_name   */ "Bag",
        /* container_count  */ 1,
        /* capacity         */ 8,
        /* has_parent       */ true,
        /* is_unlocked      */ true,
        /* has_pagination   */ false,
        /* contained_count  */ 2,
        /* first_index      */ 0,
        /* items            */ &[(5, rope_meta)],
    );

    // C++ protocolgame.cpp:1900 sendContainer
    //   opcode | cid
    //   addItem(container)            -> u16 client_id (bag is plain)
    //   addString(name="Bag")         -> u16 len + 3 bytes
    //   capacity u8 | hasParent u8 | searchIcon=0 | unlocked u8 | pagination u8
    //   containerSize u16 | firstIndex u16
    //   itemsToSend u8
    //   per-item addItem -> u16 client_id + (stackable count u8)
    expected_bytes!(
        out,
        &[
            0x6E, 0x03, // bag: 0x07C3 LE
            0xC3, 0x07, // "Bag" length-prefixed: u16=3, then 'B','a','g'
            0x03, 0x00, 0x42, 0x61, 0x67, 0x08, // capacity
            0x01, // has_parent
            0x00, // show_search
            0x01, // is_unlocked
            0x00, // pagination
            0x02, 0x00, // contained_count
            0x00, 0x00, // first_index
            0x01, // items_to_send
            // rope: client_id 0x0848 + count 5 (stackable)
            0x48, 0x08, 0x05,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendEmptyContainer — opcode 0x6E (hard-coded placeholder bag)
// ---------------------------------------------------------------------------

#[test]
fn parity_empty_container_placeholder_layout() {
    let mut out = OutputMessage::new();
    serialize_empty_container(&mut out, /* cid */ 5);

    // C++ protocolgame.cpp:1938 sendEmptyContainer
    //   opcode 0x6E | cid u8
    //   addItem(ITEM_BAG=1987, 1)     -> bag is plain non-stackable: u16
    //   addString("Placeholder")      -> u16 len=11 + 11 bytes
    //   0x08, 0x00, 0x00, 0x01, 0x00  (capacity, parent, search, unlock, page)
    //   0x0000 u16 contained | 0x0000 u16 first | 0x00 u8 items-to-send
    expected_bytes!(
        out,
        &[
            0x6E, 0x05, 0xC3, 0x07, // "Placeholder": u16 len=11, then 11 bytes
            0x0B, 0x00, 0x50, 0x6C, 0x61, 0x63, 0x65, 0x68, 0x6F, 0x6C, 0x64, 0x65, 0x72, 0x08,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    );
}

// ---------------------------------------------------------------------------
// sendCloseContainer — opcode 0x6F
// ---------------------------------------------------------------------------

#[test]
fn parity_close_container_minimal_layout() {
    // serialize_close_container is an older-style helper that returns
    // Vec<u8> directly (no &mut OutputMessage).  Compare against the C++
    // literal layout: opcode + cid.
    let bytes = serialize_close_container(7);

    // C++ protocolgame.cpp:2391 sendCloseContainer
    //   opcode 0x6F | cid u8
    assert_bytes_eq(&bytes, &[0x6F, 0x07], "protocolgame.cpp:2391");
}

// ---------------------------------------------------------------------------
// sendAddContainerItem — opcode 0x70
// ---------------------------------------------------------------------------

#[test]
fn parity_add_container_item_stackable_rope() {
    let rope_meta = make_item_type_meta(2120, true);

    let mut out = OutputMessage::new();
    serialize_add_container_item(
        &mut out, /* cid        */ 3, /* slot       */ 2, /* item_count */ 10,
        /* item_meta  */ rope_meta,
    );

    // C++ protocolgame.cpp:2902 sendAddContainerItem
    //   opcode 0x70 | cid u8 | slot u16 | addItem(item)
    //   rope is stackable: u16 client_id 0x0848 + u8 count
    expected_bytes!(out, &[0x70, 0x03, 0x02, 0x00, 0x48, 0x08, 0x0A,],);
}

// ---------------------------------------------------------------------------
// sendUpdateContainerItem — opcode 0x71
// ---------------------------------------------------------------------------

#[test]
fn parity_update_container_item_stackable_rope() {
    let rope_meta = make_item_type_meta(2120, true);

    let mut out = OutputMessage::new();
    serialize_update_container_item(
        &mut out, /* cid        */ 3, /* slot       */ 2, /* item_count */ 10,
        /* item_meta  */ rope_meta,
    );

    // C++ protocolgame.cpp:2912 sendUpdateContainerItem
    //   identical to sendAddContainerItem except opcode byte = 0x71
    expected_bytes!(out, &[0x71, 0x03, 0x02, 0x00, 0x48, 0x08, 0x0A,],);
}

// ---------------------------------------------------------------------------
// sendRemoveContainerItem — opcode 0x72
// ---------------------------------------------------------------------------

#[test]
fn parity_remove_container_item_no_replacement() {
    let mut out = OutputMessage::new();
    serialize_remove_container_item(
        &mut out, /* cid       */ 3, /* slot      */ 2, /* last_item */ None,
    );

    // C++ protocolgame.cpp:2922 sendRemoveContainerItem (lastItem == nullptr branch)
    //   opcode 0x72 | cid u8 | slot u16 | u16 sentinel 0x0000
    expected_bytes!(out, &[0x72, 0x03, 0x02, 0x00, 0x00, 0x00,],);
}

#[test]
fn parity_remove_container_item_with_replacement() {
    let rope_meta = make_item_type_meta(2120, true);

    let mut out = OutputMessage::new();
    serialize_remove_container_item(
        &mut out,
        /* cid       */ 3,
        /* slot      */ 2,
        /* last_item */ Some((5, rope_meta)),
    );

    // C++ protocolgame.cpp:2922 sendRemoveContainerItem (lastItem != nullptr branch)
    //   opcode 0x72 | cid u8 | slot u16 | addItem(lastItem)
    //   rope stackable: u16 client_id 0x0848 + u8 count 5
    expected_bytes!(out, &[0x72, 0x03, 0x02, 0x00, 0x48, 0x08, 0x05,],);
}
