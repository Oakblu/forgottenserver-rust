//! Byte-parity tests for the `wire-market` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs`.
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  Tests cover the 7 Session 7 market packets:
//!   * `sendMarketEnter`            (opcode 0xF6)                     — C++:2095
//!   * `sendMarketLeave`            (opcode 0xF7)                     — C++:2160
//!   * `sendMarketBrowseItem`       (opcode 0xF9 / req 0x03)          — C++:2167
//!   * `sendMarketAcceptOffer`      (opcode 0xF9 / req 0x03)          — C++:2202
//!   * `sendMarketBrowseOwnOffers`  (opcode 0xF9 / req 0x02)          — C++:2233
//!   * `sendMarketCancelOffer`      (opcode 0xF9 / req 0x02)          — C++:2266
//!   * `sendMarketBrowseOwnHistory` (opcode 0xF9 / req 0x01)          — C++:2299

use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_network::protocolgame::{
    serialize_market_accept_offer, serialize_market_browse_item,
    serialize_market_browse_own_history, serialize_market_browse_own_offers,
    serialize_market_cancel_offer, serialize_market_enter, serialize_market_leave,
    MarketOfferExFlat,
};

// ---------------------------------------------------------------------------
// sendMarketEnter — opcode 0xF6
// ---------------------------------------------------------------------------

#[test]
fn parity_market_enter_two_depot_items() {
    let mut out = OutputMessage::new();
    // offer_count = 7 (already clamped to u8::MAX by the caller).
    // Two depot rows:
    //   row 1: ware_id 0x0123 (291), classification 0, count 5
    //   row 2: ware_id 0x4567 (17767), classification 2 (=> +0x00 byte), count 65535
    serialize_market_enter(
        &mut out,
        /* offer_count */ 7,
        /* depot_items */
        &[(0x0123, 0, 5), (0x4567, 2, 0xFFFF)],
    );

    // C++ protocolgame.cpp:2095 sendMarketEnter:
    //   opcode 0xF6
    //   offer_count u8 = 7
    //   items_to_send u16 LE = 2
    //   row 1: ware_id u16 LE | count u16 LE
    //   row 2: ware_id u16 LE | 0x00 (classification > 0) | count u16 LE
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF6, 0x07, // items_to_send = 2
            0x02, 0x00, // row 1: ware_id 0x0123, count 5
            0x23, 0x01, 0x05, 0x00,
            // row 2: ware_id 0x4567, classification tier byte 0x00, count 0xFFFF
            0x67, 0x45, 0x00, 0xFF, 0xFF,
        ],
        "protocolgame.cpp:2095 sendMarketEnter",
    );
}

#[test]
fn parity_market_enter_empty() {
    let mut out = OutputMessage::new();
    serialize_market_enter(
        &mut out,
        /* offer_count */ 0,
        /* depot_items */ &[],
    );

    // Empty depot: opcode 0xF6, offer_count 0, items_to_send u16 LE = 0.
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[0xF6, 0x00, 0x00, 0x00],
        "protocolgame.cpp:2095 sendMarketEnter (empty)",
    );
}

// ---------------------------------------------------------------------------
// sendMarketLeave — opcode 0xF7
// ---------------------------------------------------------------------------

#[test]
fn parity_market_leave_opcode_only() {
    let mut out = OutputMessage::new();
    serialize_market_leave(&mut out);

    // C++ protocolgame.cpp:2160 sendMarketLeave: opcode 0xF7 only.
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[0xF7],
        "protocolgame.cpp:2160 sendMarketLeave",
    );
}

// ---------------------------------------------------------------------------
// sendMarketBrowseItem — opcode 0xF9, request 0x03 MARKETREQUEST_ITEM
// ---------------------------------------------------------------------------

#[test]
fn parity_market_browse_item_with_offers() {
    let mut out = OutputMessage::new();
    // client_id 0x0948 (2376), classification false (no tier byte).
    // buy_offers: 1 row at timestamp=0x01020304, counter=0x0011, amount=0x0022,
    //             price=0x000000000000_1234, player_name="Alice".
    // sell_offers: empty.
    serialize_market_browse_item(
        &mut out,
        /* client_id           */ 0x0948,
        /* has_classification  */ false,
        /* buy_offers          */
        &[(0x0102_0304, 0x0011, 0x0022, 0x0000_0000_0000_1234, "Alice")],
        /* sell_offers         */ &[],
    );

    // C++ protocolgame.cpp:2167 sendMarketBrowseItem:
    //   opcode 0xF9 | req 0x03 | client_id u16 LE
    //   buy_count u32 LE = 1
    //     timestamp u32 LE | counter u16 LE | amount u16 LE
    //     price u64 LE | name (u16 len + bytes)
    //   sell_count u32 LE = 0
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x03, // client_id 0x0948
            0x48, 0x09, // buy_count = 1
            0x01, 0x00, 0x00, 0x00, // timestamp 0x01020304
            0x04, 0x03, 0x02, 0x01, // counter 0x0011
            0x11, 0x00, // amount 0x0022
            0x22, 0x00, // price 0x0000_0000_0000_1234
            0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // name "Alice" (u16 len=5 + bytes)
            0x05, 0x00, b'A', b'l', b'i', b'c', b'e', // sell_count = 0
            0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:2167 sendMarketBrowseItem",
    );
}

#[test]
fn parity_market_browse_item_with_classification_tier() {
    let mut out = OutputMessage::new();
    // Classified item: client_id 0x1234, has_classification true (+0x00 tier
    // byte).  Empty lists.
    serialize_market_browse_item(
        &mut out,
        /* client_id           */ 0x1234,
        /* has_classification  */ true,
        /* buy_offers          */ &[],
        /* sell_offers         */ &[],
    );

    // Tier byte 0x00 sits between client_id and buy_count when
    // classification > 0 (C++:2177–2179).
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x03, // client_id 0x1234
            0x34, 0x12, // tier byte
            0x00, // buy_count = 0
            0x00, 0x00, 0x00, 0x00, // sell_count = 0
            0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:2167 sendMarketBrowseItem (classified tier byte)",
    );
}

// ---------------------------------------------------------------------------
// sendMarketAcceptOffer — opcode 0xF9, request 0x03 MARKETREQUEST_ITEM
// ---------------------------------------------------------------------------

#[test]
fn parity_market_accept_offer_buy() {
    let mut out = OutputMessage::new();
    let offer = MarketOfferExFlat {
        is_buy: true,
        client_id: 0x0BD7, // gold coin client_id
        has_classification: false,
        timestamp: 0x1122_3344,
        counter: 0x0007,
        amount: 0x000A,
        price: 0x0000_0000_0000_0064,
        player_name: "Bob",
    };
    serialize_market_accept_offer(&mut out, &offer);

    // C++ protocolgame.cpp:2202 sendMarketAcceptOffer (BUY branch):
    //   opcode 0xF9 | req 0x03 | client_id u16 LE
    //   buy_count u32 LE = 1
    //   timestamp u32 LE | counter u16 LE | amount u16 LE
    //   price u64 LE | name (u16 len + bytes)
    //   sell_count u32 LE = 0
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x03, // client_id 0x0BD7
            0xD7, 0x0B, // buy_count = 1
            0x01, 0x00, 0x00, 0x00, // timestamp 0x11223344
            0x44, 0x33, 0x22, 0x11, // counter 0x0007
            0x07, 0x00, // amount 0x000A
            0x0A, 0x00, // price 0x0064
            0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // name "Bob"
            0x03, 0x00, b'B', b'o', b'b', // sell_count = 0
            0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:2202 sendMarketAcceptOffer (BUY)",
    );
}

#[test]
fn parity_market_accept_offer_sell_classified() {
    let mut out = OutputMessage::new();
    let offer = MarketOfferExFlat {
        is_buy: false,
        client_id: 0x09DA,
        has_classification: true, // emits the +0x00 tier byte
        timestamp: 0x0000_0001,
        counter: 0x0001,
        amount: 0x0001,
        price: 0x0000_0000_0000_0001,
        player_name: "Eve",
    };
    serialize_market_accept_offer(&mut out, &offer);

    // SELL branch with tier byte:
    //   opcode | req | client_id | 0x00 (tier)
    //   buy_count u32 = 0
    //   sell_count u32 = 1
    //   timestamp u32 | counter u16 | amount u16 | price u64 | name
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x03, 0xDA, 0x09, 0x00, // tier
            0x00, 0x00, 0x00, 0x00, // buy_count
            0x01, 0x00, 0x00, 0x00, // sell_count
            0x01, 0x00, 0x00, 0x00, // timestamp
            0x01, 0x00, // counter
            0x01, 0x00, // amount
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // price
            0x03, 0x00, b'E', b'v', b'e', // name
        ],
        "protocolgame.cpp:2202 sendMarketAcceptOffer (SELL classified)",
    );
}

// ---------------------------------------------------------------------------
// sendMarketBrowseOwnOffers — opcode 0xF9, request 0x02 MARKETREQUEST_OWN_OFFERS
// ---------------------------------------------------------------------------

#[test]
fn parity_market_browse_own_offers_mixed_classification() {
    let mut out = OutputMessage::new();
    // buy: 1 row, not classified.
    // sell: 1 row, classified (extra tier byte 0x00).
    serialize_market_browse_own_offers(
        &mut out,
        /* buy_offers  */
        &[(
            0x01020304,            // timestamp
            0x0010,                // counter
            0x0948,                // client_id
            false,                 // has_classification
            0x0005,                // amount
            0x0000_0000_0000_0064, // price
        )],
        /* sell_offers */
        &[(
            0x05060708,            // timestamp
            0x0020,                // counter
            0x09DA,                // client_id
            true,                  // has_classification → emits +0x00
            0x0003,                // amount
            0x0000_0000_0000_03E8, // price
        )],
    );

    // C++ protocolgame.cpp:2233 sendMarketBrowseOwnOffers:
    //   opcode 0xF9 | req 0x02
    //   buy_count u32 LE = 1
    //     timestamp u32 LE | counter u16 LE | client_id u16 LE
    //     [tier 0x00 if classified] | amount u16 LE | price u64 LE
    //   sell_count u32 LE = 1
    //     same row shape (classified → tier byte)
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x02, // buy_count = 1
            0x01, 0x00, 0x00, 0x00, // ---- buy row (not classified) ----
            0x04, 0x03, 0x02, 0x01, // timestamp
            0x10, 0x00, // counter
            0x48, 0x09, // client_id
            // no tier byte
            0x05, 0x00, // amount
            0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // price
            // sell_count = 1
            0x01, 0x00, 0x00, 0x00, // ---- sell row (classified) ----
            0x08, 0x07, 0x06, 0x05, // timestamp
            0x20, 0x00, // counter
            0xDA, 0x09, // client_id
            0x00, // tier byte
            0x03, 0x00, // amount
            0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // price
        ],
        "protocolgame.cpp:2233 sendMarketBrowseOwnOffers",
    );
}

// ---------------------------------------------------------------------------
// sendMarketCancelOffer — opcode 0xF9, request 0x02 MARKETREQUEST_OWN_OFFERS
// ---------------------------------------------------------------------------

#[test]
fn parity_market_cancel_offer_buy() {
    let mut out = OutputMessage::new();
    let offer = MarketOfferExFlat {
        is_buy: true,
        client_id: 0x0948,
        has_classification: false,
        timestamp: 0xDEAD_BEEF,
        counter: 0x0042,
        amount: 0x0009,
        price: 0x0000_0000_0000_00FF,
        // player_name is unused by sendMarketCancelOffer — see C++:2266.
        player_name: "ignored",
    };
    serialize_market_cancel_offer(&mut out, &offer);

    // C++ protocolgame.cpp:2266 sendMarketCancelOffer (BUY branch):
    //   opcode 0xF9 | req 0x02
    //   buy_count u32 LE = 1
    //   timestamp u32 | counter u16 | client_id u16
    //   [tier 0x00 if classified] | amount u16 | price u64
    //   sell_count u32 LE = 0
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x02, // buy_count = 1
            0x01, 0x00, 0x00, 0x00, // timestamp 0xDEADBEEF
            0xEF, 0xBE, 0xAD, 0xDE, // counter 0x0042
            0x42, 0x00, // client_id 0x0948
            0x48, 0x09, // amount 0x0009
            0x09, 0x00, // price 0xFF
            0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // sell_count = 0
            0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:2266 sendMarketCancelOffer (BUY)",
    );
}

#[test]
fn parity_market_cancel_offer_sell_classified() {
    let mut out = OutputMessage::new();
    let offer = MarketOfferExFlat {
        is_buy: false,
        client_id: 0x09DA,
        has_classification: true,
        timestamp: 0x01,
        counter: 0x02,
        amount: 0x03,
        price: 0x04,
        player_name: "unused",
    };
    serialize_market_cancel_offer(&mut out, &offer);

    // SELL branch with tier byte (mirrors C++:2283 onwards).
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x02, // buy_count = 0
            0x00, 0x00, 0x00, 0x00, // sell_count = 1
            0x01, 0x00, 0x00, 0x00, // timestamp 0x01
            0x01, 0x00, 0x00, 0x00, // counter 0x02
            0x02, 0x00, // client_id 0x09DA
            0xDA, 0x09, // tier byte
            0x00, // amount 0x03
            0x03, 0x00, // price 0x04
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:2266 sendMarketCancelOffer (SELL classified)",
    );
}

// ---------------------------------------------------------------------------
// sendMarketBrowseOwnHistory — opcode 0xF9, request 0x01 MARKETREQUEST_OWN_HISTORY
// ---------------------------------------------------------------------------

#[test]
fn parity_market_browse_own_history_counter_map() {
    let mut out = OutputMessage::new();
    // Two buy rows share the same timestamp 0x1000 → counter increments
    // 0 then 1. One sell row, distinct timestamp, classified (extra 0x00).
    serialize_market_browse_own_history(
        &mut out,
        /* buy_offers */
        &[
            (
                0x1000,
                0x0948,
                false,
                0x000A,
                0x0000_0000_0000_0064,
                /* state */ 0x01,
            ),
            (0x1000, 0x0948, false, 0x000B, 0x0000_0000_0000_00C8, 0x02),
        ],
        /* sell_offers */
        &[(0x2000, 0x09DA, true, 0x000C, 0x0000_0000_0000_012C, 0x03)],
    );

    // C++ protocolgame.cpp:2299 sendMarketBrowseOwnHistory:
    //   opcode 0xF9 | req 0x01
    //   buy_to_send u32 LE = 2
    //   per row: timestamp u32 | counter u16 (counterMap[ts]++) | client_id u16
    //            [tier 0x00 if classified] | amount u16 | price u64 | state u8
    //   counterMap.clear()
    //   sell_to_send u32 LE = 1
    //   per row: same shape (classified → tier byte)
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0xF9, 0x01, // buy_to_send = 2
            0x02, 0x00, 0x00, 0x00, // ---- buy row 1 (counter = 0) ----
            0x00, 0x10, 0x00, 0x00, // timestamp 0x1000
            0x00, 0x00, // counter 0
            0x48, 0x09, // client_id 0x0948
            // no tier byte
            0x0A, 0x00, // amount
            0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // price
            0x01, // state
            // ---- buy row 2 (counter = 1, same timestamp) ----
            0x00, 0x10, 0x00, 0x00, // timestamp 0x1000
            0x01, 0x00, // counter 1
            0x48, 0x09, // client_id
            0x0B, 0x00, // amount
            0xC8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // price
            0x02, // state
            // sell_to_send = 1
            0x01, 0x00, 0x00, 0x00,
            // ---- sell row 1 (counter resets to 0, classified) ----
            0x00, 0x20, 0x00, 0x00, // timestamp 0x2000
            0x00, 0x00, // counter 0
            0xDA, 0x09, // client_id 0x09DA
            0x00, // tier byte
            0x0C, 0x00, // amount
            0x2C, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // price
            0x03, // state
        ],
        "protocolgame.cpp:2299 sendMarketBrowseOwnHistory",
    );
}
