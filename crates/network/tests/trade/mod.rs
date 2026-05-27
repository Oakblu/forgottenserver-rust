//! Byte-parity tests for the `wire-trade` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs`.
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  Tests cover the 2 Session 8 trade packets:
//!   * `sendTradeItemRequest` (opcode 0x7D own offer / 0x7E counterparty)
//!     — C++ protocolgame.cpp:2345
//!   * `sendCloseTrade`       (opcode 0x7F own / 0x80 counterparty)
//!     — C++ protocolgame.cpp:2384

use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_network::protocolgame::{serialize_close_trade, serialize_trade_item_request};

use crate::fixtures::make_item_type_meta;

// ---------------------------------------------------------------------------
// sendTradeItemRequest — opcode 0x7D (ack=own) / 0x7E (counterparty)
// ---------------------------------------------------------------------------

#[test]
fn parity_trade_item_request_own_single_stackable() {
    // Own-offer (ack=true) of a single stackable rope (client_id 2120 =
    // 0x0848) with count 5 — matches the C++ else-branch (non-container
    // item: writes a one-item list via 0x01 + addItem(item)).
    let rope_meta = make_item_type_meta(2120, true);

    let mut out = OutputMessage::new();
    serialize_trade_item_request(
        &mut out,
        /* trader_name */ "Alice",
        /* ack         */ true,
        /* items       */ &[(5, rope_meta)],
    );

    // C++ protocolgame.cpp:2345 sendTradeItemRequest (ack=true branch):
    //   opcode 0x7D
    //   addString("Alice") → u16 len=5 + 5 bytes
    //   itemList.size() = 1 → u8 0x01
    //   addItem(rope) → u16 client_id 0x0848 + u8 count 5 (stackable)
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0x7D, // "Alice" u16 len=5 + bytes
            0x05, 0x00, b'A', b'l', b'i', b'c', b'e', // itemList size = 1
            0x01, // rope: client_id 0x0848 + stackable count 5
            0x48, 0x08, 0x05,
        ],
        "protocolgame.cpp:2345 sendTradeItemRequest (own offer)",
    );
}

#[test]
fn parity_trade_item_request_counterparty_container_two_items() {
    // Counterparty-offer (ack=false) of a backpack containing two items.
    // Caller has already flattened the container's children in BFS order
    // (matches the C++ list_container walk at lines 2358–2371).
    //
    // Layout simulated:
    //   - backpack (client_id 1988 = 0x07C4, non-stackable)  ← root
    //   - 3 ropes  (client_id 2120 = 0x0848, stackable count=3)
    //   - 1 sword  (client_id 2376 = 0x0948, non-stackable)
    let backpack_meta = make_item_type_meta(1988, false);
    let rope_meta = make_item_type_meta(2120, true);
    let sword_meta = make_item_type_meta(2376, false);

    let mut out = OutputMessage::new();
    serialize_trade_item_request(
        &mut out,
        /* trader_name */ "Bob",
        /* ack         */ false,
        /* items       */
        &[(1, backpack_meta), (3, rope_meta), (1, sword_meta)],
    );

    // C++ protocolgame.cpp:2345 sendTradeItemRequest (ack=false branch,
    // container path at lines 2357–2376):
    //   opcode 0x7E
    //   addString("Bob") → u16 len=3 + 3 bytes
    //   itemList.size() = 3 → u8 0x03
    //   row 1: addItem(backpack) → u16 0x07C4 (non-stackable, no count)
    //   row 2: addItem(rope)     → u16 0x0848 + u8 count 3 (stackable)
    //   row 3: addItem(sword)    → u16 0x0948 (non-stackable, no count)
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0x7E, // "Bob" u16 len=3 + bytes
            0x03, 0x00, b'B', b'o', b'b', // itemList size = 3
            0x03, // backpack: client_id 0x07C4 (non-stackable)
            0xC4, 0x07, // rope: client_id 0x0848 + stackable count 3
            0x48, 0x08, 0x03, // sword: client_id 0x0948 (non-stackable)
            0x48, 0x09,
        ],
        "protocolgame.cpp:2345 sendTradeItemRequest (counterparty container)",
    );
}

// ---------------------------------------------------------------------------
// sendCloseTrade — opcode 0x7F (ack=own) / 0x80 (counterparty)
// ---------------------------------------------------------------------------

#[test]
fn parity_close_trade_own_opcode_only() {
    let mut out = OutputMessage::new();
    serialize_close_trade(&mut out, /* ack */ true);

    // C++ protocolgame.cpp:2384 sendCloseTrade (own close):
    //   opcode 0x7F — no payload
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[0x7F],
        "protocolgame.cpp:2384 sendCloseTrade (own)",
    );
}

#[test]
fn parity_close_trade_counterparty_opcode_only() {
    let mut out = OutputMessage::new();
    serialize_close_trade(&mut out, /* ack */ false);

    // C++ protocolgame.cpp:2384 sendCloseTrade (counterparty close path
    // uses opcode 0x80 — the partner-abandoned branch).
    //   opcode 0x80 — no payload
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[0x80],
        "protocolgame.cpp:2384 sendCloseTrade (counterparty)",
    );
}
