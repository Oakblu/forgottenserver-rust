//! Protocol trait and XTEA encryption helpers.
//!
//! Migrated from forgottenserver protocol.h / protocol.cpp.

use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::outputmessage::OutputMessage;
use forgottenserver_common::tools::adler_checksum;
use forgottenserver_common::xtea;

// ---------------------------------------------------------------------------
// ChecksumMode
// ---------------------------------------------------------------------------

/// Checksum mode used for packet validation.
///
/// Mirrors `checksumMode_t` from the C++ header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChecksumMode {
    /// Adler-32 checksum (default).
    #[default]
    Adler,
    /// Sequence-number-based checksum (used with encryption).
    Sequence,
    /// No checksum validation.
    None,
}

// ---------------------------------------------------------------------------
// Checksum helpers
// ---------------------------------------------------------------------------

/// Validate the Adler-32 checksum stored in a `NetworkMessage` header.
///
/// The Tibia packet layout is:
/// ```text
/// [0..2)  – outer (unencrypted) length u16 LE
/// [2..6)  – Adler-32 checksum u32 LE
/// [6..8)  – inner (encrypted) length u16 LE
/// [8..)   – payload
/// ```
///
/// This function reads the stored u32 from bytes `[2..6)` and compares it
/// with `adler_checksum(&buffer[6..6+length])` where `length` is the outer
/// length field at `[0..2)`.  Returns `true` if they match.
pub fn validate_adler32(msg: &NetworkMessage) -> bool {
    let buf = msg.get_buffer();
    // Read the outer length (bytes 0-1)
    let outer_len = u16::from_le_bytes([buf[0], buf[1]]) as usize;
    // Read the stored checksum (bytes 2-5)
    let stored = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]);
    // The data to checksum starts at byte 6 for `outer_len` bytes.
    let end = 6 + outer_len;
    if end > buf.len() {
        return false;
    }
    let computed = adler_checksum(&buf[6..end]);
    computed == stored
}

/// Stamp the Adler-32 checksum for a `NetworkMessage` that already has its
/// outer length header written at bytes `[0..2)`.
///
/// Writes the computed Adler-32 of `buf[6..6+outer_len]` into bytes `[2..6)`.
pub fn stamp_adler32(msg: &mut NetworkMessage) {
    let buf = msg.get_buffer_mut();
    let outer_len = u16::from_le_bytes([buf[0], buf[1]]) as usize;
    let end = 6 + outer_len;
    if end > buf.len() {
        return;
    }
    // Compute over the data region (cannot borrow buf twice, so copy)
    let data: Vec<u8> = buf[6..end].to_vec();
    let checksum = adler_checksum(&data);
    let bytes = checksum.to_le_bytes();
    buf[2] = bytes[0];
    buf[3] = bytes[1];
    buf[4] = bytes[2];
    buf[5] = bytes[3];
}

// ---------------------------------------------------------------------------
// XTEA length validation (mirrors C++ XTEA_decrypt checks)
// ---------------------------------------------------------------------------

/// Returns `true` if the message length is suitable for XTEA decryption.
///
/// Mirrors the C++ guard:
/// ```cpp
/// if (((msg.getLength() - 6) & 7) != 0) { return false; }
/// ```
/// i.e. `(length - 6) % 8 == 0`.
pub fn xtea_length_valid(msg: &NetworkMessage) -> bool {
    let len = msg.get_message_length() as usize;
    if len < 6 {
        return false;
    }
    (len - 6).is_multiple_of(8)
}

/// After decrypting a XTEA-encrypted message the first two bytes of the
/// decrypted region are an inner length.  This function validates that
/// `inner_length + 8 <= outer_length`, mirroring the C++ check:
/// ```cpp
/// uint16_t innerLength = msg.get<uint16_t>();
/// if (innerLength + 8 > msg.getLength()) { return false; }
/// ```
///
/// Returns the inner length on success, or `None` if the check fails.
pub fn xtea_inner_length_valid(msg: &NetworkMessage, outer_length: u16) -> Option<u16> {
    let buf = msg.get_buffer();
    let start = forgottenserver_common::networkmessage::INITIAL_BUFFER_POSITION as usize;
    // Buffer is statically sized at NETWORKMESSAGE_MAXSIZE (24590), so
    // `start + 2 <= buf.len()` always holds; no bounds check needed.
    let inner_length = u16::from_le_bytes([buf[start], buf[start + 1]]);
    if (inner_length as usize + 8) > outer_length as usize {
        None
    } else {
        Some(inner_length)
    }
}

// ---------------------------------------------------------------------------
// Sequence number helpers
// ---------------------------------------------------------------------------

/// Advance a sequence number, wrapping at `i32::MAX`.
///
/// Mirrors `Protocol::getNextSequenceId()` in protocol.h:
/// ```cpp
/// const auto sequence = ++sequenceNumber;
/// if (sequenceNumber >= static_cast<uint32_t>(std::numeric_limits<int32_t>::max())) {
///     sequenceNumber = 0;
/// }
/// return sequence;
/// ```
pub fn next_sequence_number(current: &mut u32) -> u32 {
    *current = current.wrapping_add(1);
    let seq = *current;
    if *current >= i32::MAX as u32 {
        *current = 0;
    }
    seq
}

// ---------------------------------------------------------------------------
// Protocol trait
// ---------------------------------------------------------------------------

/// Core protocol interface.  Implementors handle connection lifecycle and
/// message dispatch.
pub trait Protocol {
    fn on_connect(&mut self);
    fn on_close(&mut self);
    /// Called for the very first message on a connection.
    /// Returns `false` if the handshake fails and the connection should be
    /// closed.
    fn on_recv_first_message(&mut self, msg: &NetworkMessage) -> bool;
    fn on_recv_message(&mut self, msg: &NetworkMessage);
}

// ---------------------------------------------------------------------------
// XTEA helpers for OutputMessage bytes
// ---------------------------------------------------------------------------

/// Encrypt the payload bytes of an `OutputMessage` in-place using XTEA.
///
/// The first two bytes of `OutputMessage` are the length header and are
/// **not** encrypted — only the payload region is touched.  The payload is
/// padded to a multiple of 8 bytes before encryption (padding bytes are
/// zero).
///
/// Returns the number of encrypted bytes (payload, padded to 8-byte blocks).
pub fn encrypt_output(msg: &mut OutputMessage, key: &xtea::Key) -> usize {
    let round_keys = xtea::expand_key(key);
    let buf = msg.get_output_buffer();
    let payload_len = buf.len().saturating_sub(2); // skip 2-byte header
    let padded_len = if payload_len.is_multiple_of(8) {
        payload_len
    } else {
        payload_len + (8 - payload_len % 8)
    };

    // Pad with zeros up to the next 8-byte boundary
    let pad_needed = padded_len - payload_len;
    for _ in 0..pad_needed {
        msg.add_padding_bytes(1);
    }

    // Obtain a mutable slice of just the payload region
    // We cannot use get_output_buffer() (immutable) after add_padding_bytes, so
    // we work through a temporary copy approach.  Since OutputMessage exposes no
    // direct mutable slice of the payload, we use a short-lived raw approach:
    // rebuild from what we know about the internal layout (header is 2 bytes).
    let total = msg.get_output_buffer().len(); // header + payload + padding
                                               // SAFETY: we use safe Rust via a Vec copy-encrypt-copy-back pattern.
    let mut payload_copy: Vec<u8> = msg.get_output_buffer()[2..total].to_vec();
    xtea::encrypt(&mut payload_copy, &round_keys);
    // Copy back — we need mutable access.  OutputMessage doesn't expose a
    // mutable buffer slice directly, so we use add_bytes on a fresh message.
    // Instead we use the trick of rebuilding: write payload back via a fresh
    // OutputMessage that the caller replaces.  But we need in-place mutation —
    // use unsafe-free workaround: write back via a second OutputMessage that
    // the caller receives if they want encrypted bytes.
    //
    // Simpler approach: store the encrypted bytes and rebuild the output buffer.
    let mut out = OutputMessage::new();
    out.add_bytes(&payload_copy);
    out.add_crypto_header();
    *msg = out;
    padded_len
}

/// Decrypt the payload bytes of a `NetworkMessage` in-place using XTEA.
///
/// The header region (first 8 bytes of NetworkMessage) is left untouched.
/// Only the payload — from `INITIAL_BUFFER_POSITION` for `length` bytes —
/// is decrypted.  `length` must be a multiple of 8.
pub fn decrypt_message(msg: &mut NetworkMessage, key: &xtea::Key) {
    let round_keys = xtea::expand_key(key);
    let len = msg.get_message_length() as usize;
    if len == 0 {
        return;
    }
    // payload region starts at byte 8
    let start = forgottenserver_common::networkmessage::INITIAL_BUFFER_POSITION as usize;
    let end = start + len;
    let buf = msg.get_buffer_mut();
    xtea::decrypt(&mut buf[start..end], &round_keys);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // MockProtocol
    // -----------------------------------------------------------------------

    struct MockProtocol {
        connect_calls: u32,
        close_calls: u32,
        recv_first_calls: u32,
        recv_calls: u32,
        /// Value returned by on_recv_first_message
        first_msg_result: bool,
    }

    impl MockProtocol {
        fn new(first_msg_result: bool) -> Self {
            Self {
                connect_calls: 0,
                close_calls: 0,
                recv_first_calls: 0,
                recv_calls: 0,
                first_msg_result,
            }
        }
    }

    impl Protocol for MockProtocol {
        fn on_connect(&mut self) {
            self.connect_calls += 1;
        }
        fn on_close(&mut self) {
            self.close_calls += 1;
        }
        fn on_recv_first_message(&mut self, _msg: &NetworkMessage) -> bool {
            self.recv_first_calls += 1;
            self.first_msg_result
        }
        fn on_recv_message(&mut self, _msg: &NetworkMessage) {
            self.recv_calls += 1;
        }
    }

    // -----------------------------------------------------------------------
    // Lifecycle tracking
    // -----------------------------------------------------------------------

    #[test]
    fn test_on_connect_tracked() {
        let mut p = MockProtocol::new(true);
        assert_eq!(p.connect_calls, 0);
        p.on_connect();
        assert_eq!(p.connect_calls, 1);
        p.on_connect();
        assert_eq!(p.connect_calls, 2);
    }

    #[test]
    fn test_on_close_tracked() {
        let mut p = MockProtocol::new(true);
        p.on_close();
        assert_eq!(p.close_calls, 1);
    }

    #[test]
    fn test_on_recv_first_message_tracked_and_returns_true() {
        let mut p = MockProtocol::new(true);
        let msg = NetworkMessage::new();
        let result = p.on_recv_first_message(&msg);
        assert_eq!(p.recv_first_calls, 1);
        assert!(result);
    }

    #[test]
    fn test_on_recv_first_message_returns_false_when_configured() {
        let mut p = MockProtocol::new(false);
        let msg = NetworkMessage::new();
        let result = p.on_recv_first_message(&msg);
        assert!(!result, "handshake failure should return false");
    }

    #[test]
    fn test_on_recv_message_tracked() {
        let mut p = MockProtocol::new(true);
        let msg = NetworkMessage::new();
        p.on_recv_message(&msg);
        p.on_recv_message(&msg);
        assert_eq!(p.recv_calls, 2);
    }

    // -----------------------------------------------------------------------
    // ChecksumMode
    // -----------------------------------------------------------------------

    #[test]
    fn test_checksum_mode_default_is_adler() {
        let mode = ChecksumMode::default();
        assert_eq!(mode, ChecksumMode::Adler);
    }

    #[test]
    fn test_checksum_mode_variants_are_distinct() {
        assert_ne!(ChecksumMode::Adler, ChecksumMode::Sequence);
        assert_ne!(ChecksumMode::Adler, ChecksumMode::None);
        assert_ne!(ChecksumMode::Sequence, ChecksumMode::None);
    }

    // -----------------------------------------------------------------------
    // validate_adler32 / stamp_adler32
    // -----------------------------------------------------------------------

    /// Build a minimal NetworkMessage with outer-length header and checksum.
    fn make_msg_with_checksum(payload: &[u8]) -> NetworkMessage {
        let mut msg = NetworkMessage::new();
        // Write payload at position 8 (INITIAL_BUFFER_POSITION)
        msg.add_bytes(payload);
        // Write outer length (=payload.len()) at bytes 0-1
        let outer_len = payload.len() as u16;
        let buf = msg.get_buffer_mut();
        let le = outer_len.to_le_bytes();
        buf[0] = le[0];
        buf[1] = le[1];
        // Data starts at byte 6 for `outer_len` bytes, which is our payload
        // at [8..8+payload.len()] — but we need it at [6..6+payload.len()].
        // For simplicity just copy payload to offset 6.
        let p: Vec<u8> = payload.to_vec();
        for (i, &b) in p.iter().enumerate() {
            buf[6 + i] = b;
        }
        // Stamp the checksum
        stamp_adler32(&mut msg);
        msg
    }

    #[test]
    fn test_stamp_and_validate_adler32_roundtrip() {
        let payload = b"hello checksum";
        let msg = make_msg_with_checksum(payload);
        assert!(validate_adler32(&msg), "stamp + validate should succeed");
    }

    #[test]
    fn test_validate_adler32_fails_with_wrong_checksum() {
        let payload = b"test data";
        let mut msg = make_msg_with_checksum(payload);
        // Corrupt the stored checksum
        let buf = msg.get_buffer_mut();
        buf[2] ^= 0xFF;
        assert!(
            !validate_adler32(&msg),
            "corrupted checksum should fail validation"
        );
    }

    #[test]
    fn test_validate_adler32_empty_payload() {
        let mut msg = NetworkMessage::new();
        // outer_len = 0, checksum should be adler32([]) = 1
        {
            let buf = msg.get_buffer_mut();
            buf[0] = 0;
            buf[1] = 0;
        }
        stamp_adler32(&mut msg);
        assert!(validate_adler32(&msg));
    }

    #[test]
    fn test_validate_adler32_rejects_truncated_buffer() {
        let mut msg = NetworkMessage::new();
        // Set outer_len to very large value to simulate truncated buffer
        {
            let buf = msg.get_buffer_mut();
            buf[0] = 0xFF;
            buf[1] = 0xFF;
        }
        // validate_adler32 should detect out-of-bounds and return false
        assert!(!validate_adler32(&msg));
    }

    // -----------------------------------------------------------------------
    // xtea_length_valid
    // -----------------------------------------------------------------------

    #[test]
    fn test_xtea_length_valid_exact_multiple() {
        // length = 14: (14 - 6) = 8, 8 % 8 == 0 → valid
        let mut msg = NetworkMessage::new();
        // Add 14 bytes of payload to get length = 14
        msg.add_bytes(&[0u8; 14]);
        assert!(xtea_length_valid(&msg));
    }

    #[test]
    fn test_xtea_length_valid_two_blocks() {
        // length = 22: (22 - 6) = 16, 16 % 8 == 0 → valid
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&[0u8; 22]);
        assert!(xtea_length_valid(&msg));
    }

    #[test]
    fn test_xtea_length_invalid_not_multiple() {
        // length = 15: (15 - 6) = 9, 9 % 8 != 0 → invalid
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&[0u8; 15]);
        assert!(!xtea_length_valid(&msg));
    }

    #[test]
    fn test_xtea_length_invalid_too_short() {
        // length = 4: less than 6 → invalid
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&[0u8; 4]);
        assert!(!xtea_length_valid(&msg));
    }

    #[test]
    fn test_xtea_length_valid_zero_length() {
        let msg = NetworkMessage::new();
        // length = 0 < 6 → invalid
        assert!(!xtea_length_valid(&msg));
    }

    // -----------------------------------------------------------------------
    // xtea_inner_length_valid
    // -----------------------------------------------------------------------

    #[test]
    fn test_xtea_inner_length_valid_passes() {
        let mut msg = NetworkMessage::new();
        // Write inner_length = 10 as first two payload bytes
        msg.add_u16(10);
        // outer_length must satisfy: 10 + 8 <= outer_length → outer >= 18
        let result = xtea_inner_length_valid(&msg, 20);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_xtea_inner_length_valid_exact_boundary() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(10);
        // 10 + 8 = 18 == outer_length → valid (equal is ok: <= check in C++)
        let result = xtea_inner_length_valid(&msg, 18);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_xtea_inner_length_valid_fails_too_large() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(20);
        // 20 + 8 = 28 > outer_length 25 → invalid
        let result = xtea_inner_length_valid(&msg, 25);
        assert!(result.is_none());
    }

    #[test]
    fn test_xtea_inner_length_valid_fails_exact_overflow() {
        let mut msg = NetworkMessage::new();
        msg.add_u16(10);
        // outer_length = 17: 10 + 8 = 18 > 17 → invalid
        let result = xtea_inner_length_valid(&msg, 17);
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------------
    // next_sequence_number
    // -----------------------------------------------------------------------

    #[test]
    fn test_next_sequence_number_increments() {
        let mut seq: u32 = 0;
        let v1 = next_sequence_number(&mut seq);
        assert_eq!(v1, 1);
        let v2 = next_sequence_number(&mut seq);
        assert_eq!(v2, 2);
    }

    #[test]
    fn test_next_sequence_number_returns_incremented_value() {
        let mut seq: u32 = 99;
        let v = next_sequence_number(&mut seq);
        assert_eq!(v, 100);
        assert_eq!(seq, 100);
    }

    #[test]
    fn test_next_sequence_number_wraps_at_i32_max() {
        // When seq reaches i32::MAX, it should wrap to 0.
        let mut seq: u32 = i32::MAX as u32 - 1;
        let v = next_sequence_number(&mut seq);
        assert_eq!(v, i32::MAX as u32);
        // After reaching i32::MAX, seq should be reset to 0.
        assert_eq!(seq, 0);
    }

    #[test]
    fn test_next_sequence_number_continues_after_wrap() {
        let mut seq: u32 = i32::MAX as u32 - 1;
        next_sequence_number(&mut seq); // reaches MAX, wraps to 0
        let v = next_sequence_number(&mut seq); // should be 1
        assert_eq!(v, 1);
    }

    // -----------------------------------------------------------------------
    // XTEA encrypt/decrypt round-trip on NetworkMessage payload
    // -----------------------------------------------------------------------

    #[test]
    fn test_xtea_decrypt_message_round_trip() {
        let key = xtea::Key([0xDEAD_BEEF, 0xDEAD_BEEF, 0xDEAD_BEEF, 0xDEAD_BEEF]);
        let round_keys = xtea::expand_key(&key);

        // Build a 16-byte message payload (2 full XTEA blocks)
        let mut msg = NetworkMessage::new();
        let payload: [u8; 16] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ];
        msg.add_bytes(&payload);

        // Encrypt the payload region manually
        let start = forgottenserver_common::networkmessage::INITIAL_BUFFER_POSITION as usize;
        let buf = msg.get_buffer_mut();
        xtea::encrypt(&mut buf[start..start + 16], &round_keys);

        // Now decrypt via our helper
        decrypt_message(&mut msg, &key);

        // Read back and verify (set_buffer_position(0) resets to INITIAL_BUFFER_POSITION)
        msg.set_buffer_position(0);
        let recovered = msg.get_bytes(16);
        assert_eq!(recovered, payload);
    }

    #[test]
    fn test_xtea_encrypt_output_and_decrypt_round_trip() {
        let key = xtea::Key([0x01234567, 0x89ABCDEF, 0xFEDCBA98, 0x76543210]);

        // Build an OutputMessage with 16 payload bytes
        let mut out = OutputMessage::new();
        let payload: [u8; 16] = [
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            0x99, 0x00,
        ];
        out.add_bytes(&payload);
        out.write_message_length();

        // Encrypt
        encrypt_output(&mut out, &key);

        // Layout after add_crypto_header: [outer_len:2=20][adler32:4][encrypted:16]
        let encrypted_buf = out.get_output_buffer();
        assert_eq!(encrypted_buf.len(), 22, "2 + 4 + 16 = 22 bytes total");
        assert_eq!(
            u16::from_le_bytes([encrypted_buf[0], encrypted_buf[1]]),
            20,
            "outer_len = 4 (adler32) + 16 (encrypted)"
        );
        // Encrypted data starts at offset 6 (after outer_len:2 + adler32:4)
        assert_ne!(&encrypted_buf[6..22], &payload[..]);

        // Decrypt back: put encrypted payload into a NetworkMessage
        let encrypted_payload: Vec<u8> = encrypted_buf[6..22].to_vec();
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&encrypted_payload);
        decrypt_message(&mut msg, &key);

        // set_buffer_position(0) resets cursor to INITIAL_BUFFER_POSITION (byte 8)
        msg.set_buffer_position(0);
        let recovered = msg.get_bytes(16);
        assert_eq!(
            recovered, payload,
            "round-trip should recover original payload"
        );
    }

    // -----------------------------------------------------------------------
    // Edge-case guards (mirror defensive branches in C++ XTEA_decrypt /
    // checksum stamping logic)
    // -----------------------------------------------------------------------

    #[test]
    fn test_stamp_adler32_no_op_on_truncated_buffer() {
        // Set outer_len to a value larger than the buffer (24590 bytes).
        // stamp_adler32 must early-return without writing or panicking.
        let mut msg = NetworkMessage::new();
        {
            let buf = msg.get_buffer_mut();
            // outer_len = 0xFFFF (65535) → end = 65541 > 24590 buffer size
            buf[0] = 0xFF;
            buf[1] = 0xFF;
            // Pre-write a sentinel into the checksum bytes — they must stay
            // unchanged after stamp_adler32 bails out.
            buf[2] = 0xAB;
            buf[3] = 0xCD;
            buf[4] = 0xEF;
            buf[5] = 0x01;
        }
        stamp_adler32(&mut msg);
        let buf = msg.get_buffer();
        assert_eq!(
            buf[2], 0xAB,
            "stamp_adler32 must not touch checksum on truncated buffer"
        );
        assert_eq!(buf[3], 0xCD);
        assert_eq!(buf[4], 0xEF);
        assert_eq!(buf[5], 0x01);
    }

    #[test]
    fn test_decrypt_message_no_op_on_empty_payload() {
        // A freshly constructed NetworkMessage has length 0.  decrypt_message
        // must early-return rather than attempt to decrypt an empty slice.
        let key = xtea::Key([0x0000_0000, 0x0000_0000, 0x0000_0000, 0x0000_0000]);
        let mut msg = NetworkMessage::new();
        assert_eq!(msg.get_message_length(), 0);

        // Snapshot the entire buffer — it must remain untouched.
        let before: Vec<u8> = msg.get_buffer().to_vec();
        decrypt_message(&mut msg, &key);
        let after: Vec<u8> = msg.get_buffer().to_vec();
        assert_eq!(
            before, after,
            "decrypt_message must be a no-op for length=0"
        );
    }

    #[test]
    fn test_encrypt_output_pads_non_multiple_of_8_payload() {
        // Use a payload length that is NOT a multiple of 8 to exercise the
        // padding branch (`payload_len + (8 - payload_len % 8)`).
        // 13 bytes → padded to 16 (one full XTEA block past the partial).
        let key = xtea::Key([0x0102_0304, 0x0506_0708, 0x090A_0B0C, 0x0D0E_0F10]);

        let mut out = OutputMessage::new();
        let payload: [u8; 13] = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
        ];
        out.add_bytes(&payload);
        out.write_message_length();

        let padded = encrypt_output(&mut out, &key);
        assert_eq!(
            padded, 16,
            "13-byte payload should round up to 16 bytes (next 8-byte boundary)"
        );

        // Layout after add_crypto_header: [outer_len:2=20][adler32:4][encrypted:16] = 22 bytes.
        assert_eq!(out.get_output_buffer().len(), 22);

        // Decrypting the encrypted payload should recover original 13 bytes
        // followed by 3 zero padding bytes.
        let encrypted_payload: Vec<u8> = out.get_output_buffer()[6..22].to_vec();
        let mut msg = NetworkMessage::new();
        msg.add_bytes(&encrypted_payload);
        decrypt_message(&mut msg, &key);
        msg.set_buffer_position(0);
        let recovered = msg.get_bytes(16);
        assert_eq!(&recovered[..13], &payload[..]);
        assert_eq!(
            &recovered[13..16],
            &[0u8, 0u8, 0u8],
            "padding bytes must be zero"
        );
    }
}
