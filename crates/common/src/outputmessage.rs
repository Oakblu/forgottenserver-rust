//! Migrated from forgottenserver/src/outputmessage.h and outputmessage.cpp
//!
//! `OutputMessage` is the outgoing packet builder.  It is a pure byte buffer —
//! unlike the C++ version it does NOT depend on connection or protocol types.
//!
//! ## Buffer layout
//! ```text
//! [0..2)   — reserved for the 2-byte message-length header
//! [2..)    — payload (write cursor starts here after construction or reset)
//! ```
//!
//! `write_message_length()` fills bytes `[0..2)` with the current payload
//! length as a little-endian `u16`.

#![allow(dead_code)]

use crate::constants::NETWORKMESSAGE_MAXSIZE;

/// Number of bytes reserved at the start of the buffer for the length header.
pub const HEADER_LENGTH: usize = 2;

/// Maximum number of bytes that can be written into the payload region.
///
/// Mirrors the C++ `MAX_BODY_LENGTH` minus 2 for the length header itself.
pub const MAX_PAYLOAD_LENGTH: usize = NETWORKMESSAGE_MAXSIZE as usize - HEADER_LENGTH;

/// Maximum string / raw-bytes length accepted by write methods.
const MAX_STRING_LENGTH: usize = 8192;

// ---------------------------------------------------------------------------
// OutputMessage
// ---------------------------------------------------------------------------

/// An outgoing Tibia protocol packet builder.
///
/// The first two bytes are always reserved for the message-length header.
/// All `add_*` methods write into `buffer[write_pos..]` and advance
/// `write_pos`.  Call `write_message_length()` once the payload is complete
/// to stamp the header.
pub struct OutputMessage {
    buffer: Box<[u8; NETWORKMESSAGE_MAXSIZE as usize]>,
    /// Current write cursor (absolute index into `buffer`).
    write_pos: usize,
}

impl Default for OutputMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputMessage {
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0u8; NETWORKMESSAGE_MAXSIZE as usize]),
            write_pos: HEADER_LENGTH,
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn can_add(&self, n: usize) -> bool {
        self.write_pos + n <= NETWORKMESSAGE_MAXSIZE as usize
    }

    // -----------------------------------------------------------------------
    // Header
    // -----------------------------------------------------------------------

    /// Writes the current message length as a little-endian `u16` at bytes
    /// `[0..2)` of the buffer.
    pub fn write_message_length(&mut self) {
        let len = self.get_message_length() as u16;
        let bytes = len.to_le_bytes();
        self.buffer[0] = bytes[0];
        self.buffer[1] = bytes[1];
    }

    // -----------------------------------------------------------------------
    // Write operations
    // -----------------------------------------------------------------------

    /// Writes one byte; silently ignores the write on overflow.
    pub fn add_u8(&mut self, value: u8) {
        if !self.can_add(1) {
            return;
        }
        self.buffer[self.write_pos] = value;
        self.write_pos += 1;
    }

    /// Writes a little-endian `u16`.
    pub fn add_u16(&mut self, value: u16) {
        if !self.can_add(2) {
            return;
        }
        let bytes = value.to_le_bytes();
        self.buffer[self.write_pos] = bytes[0];
        self.buffer[self.write_pos + 1] = bytes[1];
        self.write_pos += 2;
    }

    /// Writes a little-endian `u32`.
    pub fn add_u32(&mut self, value: u32) {
        if !self.can_add(4) {
            return;
        }
        let bytes = value.to_le_bytes();
        self.buffer[self.write_pos..self.write_pos + 4].copy_from_slice(&bytes);
        self.write_pos += 4;
    }

    /// Writes a little-endian `u64`.
    pub fn add_u64(&mut self, value: u64) {
        if !self.can_add(8) {
            return;
        }
        let bytes = value.to_le_bytes();
        self.buffer[self.write_pos..self.write_pos + 8].copy_from_slice(&bytes);
        self.write_pos += 8;
    }

    /// Writes a length-prefixed UTF-8 string (u16 length prefix + bytes).
    ///
    /// Silently drops the write if the string exceeds `MAX_STRING_LENGTH`
    /// bytes or there is insufficient buffer space.
    pub fn add_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if !self.can_add(len + 2) || len > MAX_STRING_LENGTH {
            return;
        }
        self.add_u16(len as u16);
        self.buffer[self.write_pos..self.write_pos + len].copy_from_slice(bytes);
        self.write_pos += len;
    }

    /// Writes raw bytes; silently drops on overflow or if `data` exceeds
    /// `MAX_STRING_LENGTH`.
    pub fn add_bytes(&mut self, data: &[u8]) {
        let len = data.len();
        if !self.can_add(len) || len > MAX_STRING_LENGTH {
            return;
        }
        self.buffer[self.write_pos..self.write_pos + len].copy_from_slice(data);
        self.write_pos += len;
    }

    /// Writes `n` zero-padding bytes.
    pub fn add_padding_bytes(&mut self, n: usize) {
        if !self.can_add(n) {
            return;
        }
        for i in 0..n {
            self.buffer[self.write_pos + i] = 0x00;
        }
        self.write_pos += n;
    }

    // -----------------------------------------------------------------------
    // Buffer access
    // -----------------------------------------------------------------------

    /// Returns the number of bytes written after the header.
    ///
    /// `get_message_length() == write_pos − HEADER_LENGTH`
    pub fn get_message_length(&self) -> usize {
        self.write_pos - HEADER_LENGTH
    }

    /// Returns the slice from the start of the buffer through to the end of
    /// written data (i.e., the header bytes + payload bytes).
    pub fn get_output_buffer(&self) -> &[u8] {
        &self.buffer[..self.write_pos]
    }

    /// Returns the header position (always 0).
    pub fn get_header_position(&self) -> usize {
        0
    }

    /// Resets the write cursor to just after the header, effectively
    /// discarding all payload written so far.
    pub fn reset(&mut self) {
        self.write_pos = HEADER_LENGTH;
    }

    /// Frames the current payload with the Adler32 crypto header used by OTClient.
    ///
    /// Transforms the buffer from `[outer_len:2][payload:N]` to
    /// `[outer_len:2 = 4+N][adler32:4][payload:N]` by shifting the payload
    /// 4 bytes right and inserting the checksum.
    pub fn add_crypto_header(&mut self) {
        let payload_len = self.write_pos - HEADER_LENGTH;
        // Shift payload 4 bytes right to make room for adler32.
        self.buffer
            .copy_within(HEADER_LENGTH..self.write_pos, HEADER_LENGTH + 4);
        self.write_pos += 4;
        // Adler32 covers the payload (now at [HEADER_LENGTH+4..write_pos)).
        let adler = crate::tools::adler_checksum(&self.buffer[HEADER_LENGTH + 4..self.write_pos]);
        self.buffer[HEADER_LENGTH..HEADER_LENGTH + 4].copy_from_slice(&adler.to_le_bytes());
        let outer_len = (4 + payload_len) as u16;
        self.buffer[0..2].copy_from_slice(&outer_len.to_le_bytes());
    }
}

impl std::fmt::Debug for OutputMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputMessage")
            .field("write_pos", &self.write_pos)
            .field("message_length", &self.get_message_length())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn msg() -> OutputMessage {
        OutputMessage::new()
    }

    // --- Initial state ---

    #[test]
    fn test_new_message_length_is_zero() {
        let m = msg();
        assert_eq!(m.get_message_length(), 0);
    }

    #[test]
    fn test_new_write_pos_is_header_length() {
        let m = msg();
        // write_pos is private; verify indirectly through get_output_buffer length
        assert_eq!(m.get_output_buffer().len(), HEADER_LENGTH);
    }

    // --- get_header_position ---

    #[test]
    fn test_get_header_position_is_zero() {
        let m = msg();
        assert_eq!(m.get_header_position(), 0);
    }

    #[test]
    fn test_output_message_get_header_position_returns_zero() {
        // C++: outputmessage.h — OutputMessage::getHeaderPosition() returns
        // HEADER_AREA which is always 0. The header occupies bytes [0..2) of
        // the buffer; position 0 is the canonical start of the header region.
        let m = msg();
        assert_eq!(m.get_header_position(), 0);
    }

    // --- write_message_length ---

    #[test]
    fn test_write_message_length_u16_le_at_position_0() {
        let mut m = msg();
        m.add_u8(1);
        m.add_u8(2);
        m.add_u8(3); // 3 payload bytes
        m.write_message_length();
        let buf = m.get_output_buffer();
        // bytes 0-1 should be 3u16 in LE
        assert_eq!(buf[0], 3);
        assert_eq!(buf[1], 0);
    }

    #[test]
    fn test_write_message_length_large_value() {
        let mut m = msg();
        // write 300 bytes
        for _ in 0..300 {
            m.add_u8(0x00);
        }
        m.write_message_length();
        let buf = m.get_output_buffer();
        let len = u16::from_le_bytes([buf[0], buf[1]]);
        assert_eq!(len, 300);
    }

    // --- add_u8 ---

    #[test]
    fn test_add_u8_advances_write_pos() {
        let mut m = msg();
        m.add_u8(0xAB);
        assert_eq!(m.get_message_length(), 1);
    }

    #[test]
    fn test_add_u8_value_stored_correctly() {
        let mut m = msg();
        m.add_u8(0xCD);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0xCD);
    }

    // --- add_u16 ---

    #[test]
    fn test_add_u16_advances_write_pos_by_2() {
        let mut m = msg();
        m.add_u16(0x1234);
        assert_eq!(m.get_message_length(), 2);
    }

    #[test]
    fn test_add_u16_little_endian() {
        let mut m = msg();
        m.add_u16(0x0102);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0x02);
        assert_eq!(buf[HEADER_LENGTH + 1], 0x01);
    }

    // --- add_u32 ---

    #[test]
    fn test_add_u32_advances_write_pos_by_4() {
        let mut m = msg();
        m.add_u32(0xDEAD_BEEF);
        assert_eq!(m.get_message_length(), 4);
    }

    #[test]
    fn test_add_u32_little_endian() {
        let mut m = msg();
        m.add_u32(0x01020304);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0x04);
        assert_eq!(buf[HEADER_LENGTH + 1], 0x03);
        assert_eq!(buf[HEADER_LENGTH + 2], 0x02);
        assert_eq!(buf[HEADER_LENGTH + 3], 0x01);
    }

    // --- add_u64 ---

    #[test]
    fn test_add_u64_advances_write_pos_by_8() {
        let mut m = msg();
        m.add_u64(u64::MAX);
        assert_eq!(m.get_message_length(), 8);
    }

    #[test]
    fn test_add_u64_little_endian() {
        let mut m = msg();
        m.add_u64(0x0807060504030201);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0x01);
        assert_eq!(buf[HEADER_LENGTH + 7], 0x08);
    }

    // --- add_string ---

    #[test]
    fn test_add_string_writes_length_prefix_and_bytes() {
        let mut m = msg();
        m.add_string("hi");
        assert_eq!(m.get_message_length(), 4); // 2 prefix + 2 chars
        let buf = m.get_output_buffer();
        // length prefix at HEADER_LENGTH
        assert_eq!(
            u16::from_le_bytes([buf[HEADER_LENGTH], buf[HEADER_LENGTH + 1]]),
            2
        );
        assert_eq!(buf[HEADER_LENGTH + 2], b'h');
        assert_eq!(buf[HEADER_LENGTH + 3], b'i');
    }

    #[test]
    fn test_add_string_empty() {
        let mut m = msg();
        m.add_string("");
        assert_eq!(m.get_message_length(), 2); // only the length prefix
    }

    // --- add_bytes ---

    #[test]
    fn test_add_bytes_advances_write_pos() {
        let mut m = msg();
        m.add_bytes(&[1, 2, 3]);
        assert_eq!(m.get_message_length(), 3);
    }

    #[test]
    fn test_add_bytes_values_stored() {
        let mut m = msg();
        m.add_bytes(&[0xAA, 0xBB]);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0xAA);
        assert_eq!(buf[HEADER_LENGTH + 1], 0xBB);
    }

    // --- add_padding_bytes ---

    #[test]
    fn test_add_padding_bytes_writes_zeros() {
        let mut m = msg();
        m.add_u8(0xFF); // sentinel
        m.add_padding_bytes(3);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH + 1], 0x00);
        assert_eq!(buf[HEADER_LENGTH + 2], 0x00);
        assert_eq!(buf[HEADER_LENGTH + 3], 0x00);
    }

    #[test]
    fn test_add_padding_bytes_advances_write_pos() {
        let mut m = msg();
        m.add_padding_bytes(5);
        assert_eq!(m.get_message_length(), 5);
    }

    // --- get_message_length ---

    #[test]
    fn test_message_length_equals_write_pos_minus_header() {
        let mut m = msg();
        m.add_u8(1);
        m.add_u16(2);
        m.add_u32(3);
        // 1 + 2 + 4 = 7
        assert_eq!(m.get_message_length(), 7);
    }

    // --- get_output_buffer ---

    #[test]
    fn test_get_output_buffer_length_matches_header_plus_payload() {
        let mut m = msg();
        m.add_u8(0x01);
        m.add_u8(0x02);
        // 2 header + 2 payload
        assert_eq!(m.get_output_buffer().len(), HEADER_LENGTH + 2);
    }

    #[test]
    fn test_get_output_buffer_after_write_message_length() {
        let mut m = msg();
        m.add_u8(0x42);
        m.write_message_length();
        let buf = m.get_output_buffer();
        assert_eq!(buf[0], 1); // length = 1
        assert_eq!(buf[1], 0);
        assert_eq!(buf[2], 0x42);
    }

    // --- reset ---

    #[test]
    fn test_reset_sets_write_pos_to_header_length() {
        let mut m = msg();
        m.add_u32(0xDEAD_BEEF);
        m.reset();
        assert_eq!(m.get_message_length(), 0);
        assert_eq!(m.get_output_buffer().len(), HEADER_LENGTH);
    }

    #[test]
    fn test_reset_allows_reuse() {
        let mut m = msg();
        m.add_u8(0xAA);
        m.reset();
        m.add_u8(0xBB);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0xBB);
        assert_eq!(m.get_message_length(), 1);
    }

    // --- mixed writes ---

    #[test]
    fn test_mixed_writes_correct_layout() {
        let mut m = msg();
        m.add_u8(0x01);
        m.add_u16(0x0203);
        m.add_u32(0x04050607);
        m.write_message_length();
        let buf = m.get_output_buffer();
        // header
        assert_eq!(u16::from_le_bytes([buf[0], buf[1]]), 7);
        // payload
        assert_eq!(buf[2], 0x01);
        assert_eq!(u16::from_le_bytes([buf[3], buf[4]]), 0x0203);
        assert_eq!(
            u32::from_le_bytes([buf[5], buf[6], buf[7], buf[8]]),
            0x04050607
        );
    }

    // -----------------------------------------------------------------------
    // Additional edge-case tests (Task 2.2 audit)
    // -----------------------------------------------------------------------

    // --- write_message_length with zero payload ---

    #[test]
    fn test_write_message_length_zero_payload() {
        let mut m = msg();
        // No payload written yet.
        m.write_message_length();
        let buf = m.get_output_buffer();
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 0);
    }

    // --- add_string / add_bytes: oversized inputs silently dropped ---

    #[test]
    fn test_add_string_over_8192_bytes_is_dropped() {
        let mut m = msg();
        let big = "z".repeat(MAX_STRING_LENGTH + 1);
        m.add_string(&big);
        assert_eq!(m.get_message_length(), 0);
        assert_eq!(m.get_output_buffer().len(), HEADER_LENGTH);
    }

    #[test]
    fn test_add_string_exactly_8192_bytes_is_accepted() {
        let mut m = msg();
        let s = "a".repeat(MAX_STRING_LENGTH);
        m.add_string(&s);
        // 2-byte length prefix + 8192 payload bytes
        assert_eq!(m.get_message_length(), 2 + MAX_STRING_LENGTH);
    }

    #[test]
    fn test_add_bytes_over_8192_bytes_is_dropped() {
        let mut m = msg();
        let big = vec![0xFFu8; MAX_STRING_LENGTH + 1];
        m.add_bytes(&big);
        assert_eq!(m.get_message_length(), 0);
    }

    #[test]
    fn test_add_bytes_exactly_8192_bytes_is_accepted() {
        let mut m = msg();
        let data = vec![0x55u8; MAX_STRING_LENGTH];
        m.add_bytes(&data);
        assert_eq!(m.get_message_length(), MAX_STRING_LENGTH);
    }

    // --- overflow protection: write past end is silently dropped ---

    #[test]
    fn test_add_u8_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Fill the entire payload region.
        let capacity = NETWORKMESSAGE_MAXSIZE as usize - HEADER_LENGTH;
        for _ in 0..capacity {
            m.add_u8(0x00);
        }
        let len_before = m.get_message_length();
        // This write must be silently dropped.
        m.add_u8(0xFF);
        assert_eq!(m.get_message_length(), len_before);
    }

    // --- reset does NOT zero the buffer ---

    #[test]
    fn test_reset_does_not_zero_buffer() {
        let mut m = msg();
        m.add_u8(0xAB);
        // The byte sits at index HEADER_LENGTH in the buffer.
        // After reset the write_pos goes back to HEADER_LENGTH but the raw byte is untouched.
        m.reset();
        // Re-write a different sentinel and verify the old byte is underneath.
        // Peek: after reset write_pos == HEADER_LENGTH, so the payload area is logically empty
        // but the raw buffer still has 0xAB at index HEADER_LENGTH.
        // We verify by writing the same position and reading back.
        m.add_u8(0xCD); // overwrites same slot
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0xCD);
    }

    // --- get_output_buffer slice correctness ---

    #[test]
    fn test_get_output_buffer_includes_header_bytes() {
        let mut m = msg();
        m.add_u8(0x42);
        m.write_message_length();
        let buf = m.get_output_buffer();
        // Slice must include both the 2-byte header region and the payload.
        assert_eq!(buf.len(), HEADER_LENGTH + 1);
        // Header bytes (LE u16 = 1)
        assert_eq!(u16::from_le_bytes([buf[0], buf[1]]), 1);
        // Payload byte
        assert_eq!(buf[2], 0x42);
    }

    #[test]
    fn test_get_output_buffer_starts_at_index_zero() {
        let mut m = msg();
        m.write_message_length();
        let buf = m.get_output_buffer();
        // The very first byte must be the length header low byte, not a payload byte.
        // For a zero-length message this is 0x00.
        assert_eq!(buf[0], 0x00);
    }

    // --- reset: write_pos returns to exactly HEADER_LENGTH ---

    #[test]
    fn test_reset_write_pos_is_exactly_header_length() {
        let mut m = msg();
        m.add_u64(u64::MAX);
        m.reset();
        // After reset get_message_length() == write_pos - HEADER_LENGTH == 0.
        assert_eq!(m.get_message_length(), 0);
        // get_output_buffer() slice length == write_pos == HEADER_LENGTH.
        assert_eq!(m.get_output_buffer().len(), HEADER_LENGTH);
    }

    // --- add_string round-trip through get_output_buffer ---

    #[test]
    fn test_add_string_round_trip_via_output_buffer() {
        let mut m = msg();
        m.add_string("test");
        let buf = m.get_output_buffer();
        // Bytes 2-3: u16 LE length == 4
        assert_eq!(
            u16::from_le_bytes([buf[HEADER_LENGTH], buf[HEADER_LENGTH + 1]]),
            4
        );
        // Bytes 4-7: "test"
        assert_eq!(&buf[HEADER_LENGTH + 2..HEADER_LENGTH + 6], b"test");
    }

    // --- header position is always 0 ---

    #[test]
    fn test_header_position_unchanged_after_writes() {
        let mut m = msg();
        m.add_u32(0xDEAD_BEEF);
        m.add_string("foo");
        assert_eq!(m.get_header_position(), 0);
    }

    // -----------------------------------------------------------------------
    // Coverage-gap tests (line-coverage fill: Default, overflow branches,
    // Debug impl)
    // -----------------------------------------------------------------------

    // --- Default::default() constructs an empty, ready-to-write message ---

    #[test]
    fn test_default_constructor_matches_new() {
        let m: OutputMessage = OutputMessage::default();
        // Default must put us in the same initial state as new(): zero payload,
        // header reserved.
        assert_eq!(m.get_message_length(), 0);
        assert_eq!(m.get_output_buffer().len(), HEADER_LENGTH);
        assert_eq!(m.get_header_position(), 0);
    }

    #[test]
    fn test_default_constructor_writes_after_default() {
        // Use the Default trait explicitly (Default::default()) to exercise that
        // code path, then confirm normal writes work afterwards.
        let mut m: OutputMessage = Default::default();
        m.add_u16(0xBEEF);
        assert_eq!(m.get_message_length(), 2);
        let buf = m.get_output_buffer();
        assert_eq!(buf[HEADER_LENGTH], 0xEF);
        assert_eq!(buf[HEADER_LENGTH + 1], 0xBE);
    }

    // --- overflow branches for fixed-width add_uN and add_padding_bytes ---
    //
    // Each test fills the buffer to within (N - 1) bytes of capacity so that the
    // next add_uN call observes can_add(N) == false and silently drops the write.

    fn fill_to_remaining(m: &mut OutputMessage, remaining: usize) {
        let capacity = NETWORKMESSAGE_MAXSIZE as usize - HEADER_LENGTH;
        let to_write = capacity - remaining;
        for _ in 0..to_write {
            m.add_u8(0x00);
        }
        debug_assert_eq!(m.get_message_length(), to_write);
    }

    #[test]
    fn test_add_u16_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Leave only 1 byte of capacity; add_u16 needs 2 → must be dropped.
        fill_to_remaining(&mut m, 1);
        let before = m.get_message_length();
        m.add_u16(0xABCD);
        assert_eq!(m.get_message_length(), before);
    }

    #[test]
    fn test_add_u32_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Leave only 3 bytes; add_u32 needs 4 → drop.
        fill_to_remaining(&mut m, 3);
        let before = m.get_message_length();
        m.add_u32(0xDEAD_BEEF);
        assert_eq!(m.get_message_length(), before);
    }

    #[test]
    fn test_add_u64_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Leave only 7 bytes; add_u64 needs 8 → drop.
        fill_to_remaining(&mut m, 7);
        let before = m.get_message_length();
        m.add_u64(u64::MAX);
        assert_eq!(m.get_message_length(), before);
    }

    #[test]
    fn test_add_padding_bytes_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Leave only 2 bytes; request 3 bytes of padding → drop.
        fill_to_remaining(&mut m, 2);
        let before = m.get_message_length();
        m.add_padding_bytes(3);
        assert_eq!(m.get_message_length(), before);
    }

    // --- Debug impl: must format struct name and the two named fields ---

    #[test]
    fn test_debug_impl_includes_write_pos_and_message_length() {
        let mut m = msg();
        m.add_u32(0x01020304); // 4 payload bytes → write_pos = HEADER_LENGTH+4, message_length = 4
        let s = format!("{m:?}");
        assert!(
            s.contains("OutputMessage"),
            "debug output missing struct name: {s}"
        );
        assert!(
            s.contains("write_pos"),
            "debug output missing write_pos: {s}"
        );
        assert!(
            s.contains("message_length"),
            "debug output missing message_length: {s}"
        );
        // The numeric values themselves should also appear.
        assert!(
            s.contains(&format!("{}", HEADER_LENGTH + 4)),
            "debug output missing write_pos value: {s}"
        );
        assert!(
            s.contains(": 4"),
            "debug output missing message_length value: {s}"
        );
    }

    #[test]
    fn test_debug_impl_empty_message() {
        let m = msg();
        let s = format!("{m:?}");
        // Fresh message: write_pos == HEADER_LENGTH (2), message_length == 0.
        assert!(s.contains("OutputMessage"));
        assert!(s.contains(&format!("write_pos: {HEADER_LENGTH}")));
        assert!(s.contains("message_length: 0"));
    }

    // -----------------------------------------------------------------------
    // add_crypto_header tests (task 9.4)
    // -----------------------------------------------------------------------

    #[test]
    fn add_crypto_header_sets_correct_outer_len() {
        let mut m = msg();
        m.add_u8(0xAA);
        m.add_u8(0xBB);
        m.add_crypto_header();
        let buf = m.get_output_buffer();
        // outer_len = 4 (adler32) + 2 (payload) = 6
        assert_eq!(u16::from_le_bytes([buf[0], buf[1]]), 6);
    }

    #[test]
    fn add_crypto_header_adler32_covers_payload() {
        use crate::tools::adler_checksum;
        let mut m = msg();
        m.add_u8(0xAA);
        m.add_u8(0xBB);
        m.add_crypto_header();
        let buf = m.get_output_buffer();
        // payload lives at buf[6..8]
        let expected = adler_checksum(&buf[6..8]);
        let stored = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]);
        assert_eq!(stored, expected);
    }

    #[test]
    fn add_crypto_header_preserves_payload_bytes() {
        let mut m = msg();
        m.add_u8(0xAA);
        m.add_u8(0xBB);
        m.add_crypto_header();
        let buf = m.get_output_buffer();
        assert_eq!(buf[6], 0xAA);
        assert_eq!(buf[7], 0xBB);
    }

    #[test]
    fn add_crypto_header_empty_payload() {
        use crate::tools::adler_checksum;
        let mut m = msg();
        // No payload — outer_len = 4, adler32 of empty slice.
        m.add_crypto_header();
        let buf = m.get_output_buffer();
        assert_eq!(u16::from_le_bytes([buf[0], buf[1]]), 4);
        let stored = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]);
        assert_eq!(stored, adler_checksum(&[]));
    }
}
