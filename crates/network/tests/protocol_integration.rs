//! In-process protocol integration tests.
//!
//! These tests exercise full parse→serialize round-trips using NetworkMessage directly.
//! No TCP sockets or Docker required.

use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_network::protocolgame::{
    parse_first_packet, serialize_character_list, serialize_disconnect, CharacterEntry,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal first-packet header byte vector (os + version only).
///
/// The wire format for the unencrypted header region of `parse_first_packet` is:
///   os (u16) + version (u16) + ...
/// For version rejection tests we only need the first 4 bytes.
fn first_packet_header_payload(os: u16, version: u16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&os.to_le_bytes());
    out.extend_from_slice(&version.to_le_bytes());
    out
}

/// Stuff `payload` into a fresh `NetworkMessage` starting at position 0
/// (which `NetworkMessage::set_buffer_position(0)` maps to `INITIAL_BUFFER_POSITION`),
/// then hand it to `parse_first_packet`.
fn parse_from_bytes(
    payload: &[u8],
) -> Result<forgottenserver_network::protocolgame::FirstPacket, String> {
    let mut msg = NetworkMessage::new();
    msg.add_bytes(payload);
    // set_buffer_position(0) sets the absolute position to INITIAL_BUFFER_POSITION,
    // which is where add_bytes just wrote.
    msg.set_buffer_position(0);
    parse_first_packet(&mut msg)
}

// ---------------------------------------------------------------------------
// Game protocol — version rejection
// ---------------------------------------------------------------------------

#[test]
fn game_login_version_too_low_returns_disconnect_message() {
    let payload = first_packet_header_payload(3, 760);
    let result = parse_from_bytes(&payload);
    assert!(result.is_err(), "version 760 must be rejected");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("13.10"),
        "rejection message must mention allowed version: {msg}"
    );
    // Verify the message formats correctly into a disconnect packet
    let disconnect = serialize_disconnect(&msg);
    assert_eq!(disconnect[0], 0x14, "disconnect opcode must be 0x14");
}

#[test]
fn game_login_version_too_high_returns_disconnect_message() {
    let payload = first_packet_header_payload(3, 9999);
    let result = parse_from_bytes(&payload);
    assert!(result.is_err(), "version 9999 must be rejected");
    let msg = result.unwrap_err();
    assert!(msg.contains("13.10"), "rejection must mention 13.10: {msg}");
    let disconnect = serialize_disconnect(&msg);
    assert_eq!(disconnect[0], 0x14);
}

#[test]
fn game_login_version_1310_not_a_version_error() {
    // The truncated payload will fail later (e.g. on the RSA block), but
    // the error must NOT be about the version number.
    let payload = first_packet_header_payload(3, 1310);
    let result = parse_from_bytes(&payload);
    if let Err(e) = &result {
        assert!(
            !e.contains("protocol"),
            "version 1310 must not trigger a version error, got: {e}"
        );
    }
}

#[test]
fn game_login_version_1311_not_a_version_error() {
    let payload = first_packet_header_payload(3, 1311);
    let result = parse_from_bytes(&payload);
    if let Err(e) = &result {
        assert!(
            !e.contains("protocol"),
            "version 1311 must not trigger a version error, got: {e}"
        );
    }
}

// ---------------------------------------------------------------------------
// Game protocol — disconnect packet wire format
// ---------------------------------------------------------------------------

#[test]
fn disconnect_bad_credentials_message_encodes_correctly() {
    let message = "Account name or password is not correct.";
    let bytes = serialize_disconnect(message);
    assert_eq!(bytes[0], 0x14, "disconnect opcode must be 0x14");
    let str_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    assert_eq!(str_len, message.len());
    assert_eq!(&bytes[3..3 + str_len], message.as_bytes());
}

#[test]
fn disconnect_version_message_encodes_correctly() {
    let message = "Only clients with protocol 13.10 allowed!";
    let bytes = serialize_disconnect(message);
    assert_eq!(bytes[0], 0x14);
    let str_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    assert_eq!(&bytes[3..3 + str_len], message.as_bytes());
}

// ---------------------------------------------------------------------------
// Game protocol — character list packet shape
// ---------------------------------------------------------------------------

#[test]
fn serialize_character_list_first_byte_is_count() {
    let chars = vec![CharacterEntry {
        name: "Testchar".to_string(),
        world_name: "Test World".to_string(),
        world_ip: 0x7F000001,
        world_port: 7172,
    }];
    let bytes = serialize_character_list(&chars);
    // First byte is character count
    assert_eq!(bytes[0], 1, "first byte must be character count = 1");
    // Next 2 bytes are the name length prefix (u16 LE)
    let name_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
    assert_eq!(name_len, "Testchar".len());
    // Name bytes follow
    assert_eq!(&bytes[3..3 + name_len], b"Testchar");
}

#[test]
fn serialize_character_list_multiple_entries() {
    let chars = vec![
        CharacterEntry {
            name: "Alice".to_string(),
            world_name: "World".to_string(),
            world_ip: 0,
            world_port: 7172,
        },
        CharacterEntry {
            name: "Bob".to_string(),
            world_name: "World".to_string(),
            world_ip: 0,
            world_port: 7172,
        },
    ];
    let bytes = serialize_character_list(&chars);
    assert_eq!(bytes[0], 2, "character count must be 2");
}

// ---------------------------------------------------------------------------
// Round-trip: parse → serialize disconnect → opcode preserved
// ---------------------------------------------------------------------------

#[test]
fn rejected_version_round_trip_produces_valid_disconnect_wire_bytes() {
    // Parse a too-low version → get error message → serialize as disconnect
    let payload = first_packet_header_payload(1, 100);
    let err_msg = parse_from_bytes(&payload).unwrap_err();

    let wire = serialize_disconnect(&err_msg);
    // Opcode
    assert_eq!(wire[0], 0x14);
    // String length prefix (u16 LE)
    let str_len = u16::from_le_bytes([wire[1], wire[2]]) as usize;
    // String body must match the error message
    assert_eq!(&wire[3..3 + str_len], err_msg.as_bytes());
    // Total length: 1 (opcode) + 2 (len-prefix) + message bytes
    assert_eq!(wire.len(), 3 + err_msg.len());
}
