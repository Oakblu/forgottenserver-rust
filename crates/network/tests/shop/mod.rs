//! Byte-parity tests for the `wire-shop` cluster of `serialize_*`
//! functions in `crates/network/src/protocolgame.rs`.
//!
//! Each test calls a `serialize_*` against a deterministic fixture and
//! asserts the produced bytes match a hex literal taken from the C++
//! source.  Tests cover the 4 Session 6 shop packets:
//!   * `sendShop`            (opcode 0x7A)        — C++:1959
//!   * `sendCloseShop`       (opcode 0x7C)        — C++:1981
//!   * `sendSaleItemList`    (2× 0xEE + opcode 0x7B) — C++:1988
//!   * `sendResourceBalance` (opcode 0xEE)        — C++:2072
//!
//! (`sendStoreBalance` parity is covered in `player/mod.rs::parity_store_balance`.)

use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_network::protocolgame::{
    serialize_close_shop, serialize_resource_balance, serialize_sale_item_list, serialize_shop,
};

// ---------------------------------------------------------------------------
// sendShop — opcode 0x7A
// ---------------------------------------------------------------------------

#[test]
fn parity_shop_two_items() {
    let mut out = OutputMessage::new();
    // Npc "Bob", gold-coin client_id = 3031 (0x0BD7).
    // Row 1: a regular sword — client_id 2376 (0x0948), subtype 0x00,
    //   name "sword", weight=3500, buy=100, sell=25.
    // Row 2: a small mana potion — client_id 7620 (0x1DC4), subtype 0x07
    //   (serverFluidToClient(7) for mana-flask), name "mana potion",
    //   weight=120, buy=50, sell=0.
    serialize_shop(
        &mut out,
        /* npc_name             */ "Bob",
        /* gold_coin_client_id  */ 3031,
        /* items                */
        &[
            (2376, 0x00, "sword", 3500, 100, 25),
            (7620, 0x07, "mana potion", 120, 50, 0),
        ],
    );

    // C++ protocolgame.cpp:1959 sendShop (+ AddShopItem at 3725):
    //   opcode 0x7A
    //   npc_name "Bob" (u16 len=3 + bytes)
    //   gold_coin_client_id u16 = 3031 (0x0BD7) LE → D7 0B
    //   "" u16 len=0
    //   itemsToSend u16 LE = 2
    //   row 1: client_id u16 LE | subtype u8 | "sword" (u16 len=5 + bytes)
    //          weight u32 LE | buy_price u32 LE | sell_price u32 LE
    //   row 2: same shape
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            0x7A, // "Bob": u16 len=3 + bytes
            0x03, 0x00, b'B', b'o', b'b', // gold_coin_client_id = 3031 = 0x0BD7
            0xD7, 0x0B, // empty currency-name string
            0x00, 0x00, // itemsToSend = 2
            0x02, 0x00, // ---- row 1: sword ----
            // client_id 2376 = 0x0948
            0x48, 0x09, // subtype 0x00
            0x00, // "sword" u16 len=5 + bytes
            0x05, 0x00, b's', b'w', b'o', b'r', b'd', // weight 3500 = 0x00000DAC
            0xAC, 0x0D, 0x00, 0x00, // buy_price 100 = 0x64
            0x64, 0x00, 0x00, 0x00, // sell_price 25 = 0x19
            0x19, 0x00, 0x00, 0x00,
            // ---- row 2: mana potion ----
            // client_id 7620 = 0x1DC4
            0xC4, 0x1D, // subtype 0x07
            0x07, // "mana potion" u16 len=11
            0x0B, 0x00, b'm', b'a', b'n', b'a', b' ', b'p', b'o', b't', b'i', b'o', b'n',
            // weight 120 = 0x78
            0x78, 0x00, 0x00, 0x00, // buy_price 50 = 0x32
            0x32, 0x00, 0x00, 0x00, // sell_price 0
            0x00, 0x00, 0x00, 0x00,
        ],
        "protocolgame.cpp:1959 sendShop (+ AddShopItem at 3725)",
    );
}

// ---------------------------------------------------------------------------
// sendCloseShop — opcode 0x7C (no payload)
// ---------------------------------------------------------------------------

#[test]
fn parity_close_shop_opcode_only() {
    let mut out = OutputMessage::new();
    serialize_close_shop(&mut out);

    // C++ protocolgame.cpp:1981 sendCloseShop
    //   opcode 0x7C — no payload
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[0x7C],
        "protocolgame.cpp:1981 sendCloseShop",
    );
}

// ---------------------------------------------------------------------------
// sendResourceBalance — opcode 0xEE
// ---------------------------------------------------------------------------

#[test]
fn parity_resource_balance_gold_equipped() {
    let mut out = OutputMessage::new();
    // resource_type 0x01 RESOURCE_GOLD_EQUIPPED, amount=0x0102030405060708.
    serialize_resource_balance(&mut out, 0x01, 0x0102_0304_0506_0708);

    // C++ protocolgame.cpp:2072 sendResourceBalance
    //   opcode 0xEE | resource_type u8 | amount u64 LE
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[0xEE, 0x01, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
        "protocolgame.cpp:2072 sendResourceBalance",
    );
}

// ---------------------------------------------------------------------------
// sendSaleItemList — 2× 0xEE (resource balances) then opcode 0x7B
// ---------------------------------------------------------------------------

#[test]
fn parity_sale_item_list_two_rows() {
    let mut out = OutputMessage::new();
    // bank_balance = 1000 (0x03E8), equipped_balance = 25 (0x19).
    // Items: (client_id, count) — gold ingot client_id 9971 (0x26F3) count=3,
    //        magic shield client_id 2522 (0x09DA) count=70000 (capped u16=0xFFFF).
    serialize_sale_item_list(
        &mut out,
        /* bank_balance     */ 1000,
        /* equipped_balance */ 25,
        /* items            */
        &[(9971, 3), (2522, 70_000)],
    );

    // C++ protocolgame.cpp:1988 sendSaleItemList:
    //   1) sendResourceBalance(RESOURCE_BANK_BALANCE=0x00, 1000)
    //      → 0xEE 0x00 + u64 LE 1000
    //   2) sendResourceBalance(RESOURCE_GOLD_EQUIPPED=0x01, 25)
    //      → 0xEE 0x01 + u64 LE 25
    //   3) opcode 0x7B | item_count u8=2
    //      per row: client_id u16 LE | min(count, u16::MAX) u16 LE
    crate::fixtures::assert_bytes_eq(
        {
            out.write_message_length();
            &out.get_output_buffer()[2..]
        },
        &[
            // frame 1: bank balance (RESOURCE_BANK_BALANCE = 0x00)
            0xEE, 0x00, 0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // frame 2: equipped balance (RESOURCE_GOLD_EQUIPPED = 0x01)
            0xEE, 0x01, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // frame 3: sale-list opcode 0x7B
            0x7B, // itemsToSend = 2
            0x02, // row 1: client_id 9971 = 0x26F3, count=3
            0xF3, 0x26, 0x03, 0x00,
            // row 2: client_id 2522 = 0x09DA, count=70000 capped at 0xFFFF
            0xDA, 0x09, 0xFF, 0xFF,
        ],
        "protocolgame.cpp:1988 sendSaleItemList",
    );
}
