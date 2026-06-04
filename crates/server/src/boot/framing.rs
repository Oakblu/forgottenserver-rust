use forgottenserver_common::tools::adler_checksum;
use forgottenserver_common::xtea;

/// Frame a plaintext server→client payload with Adler32 crypto header.
///
/// Wire layout: `[outer_len:2 = 6+N][adler32:4][inner_len:2=N][payload:N]`
/// The adler32 covers `[inner_len:2][payload]`.
/// Used for all unencrypted outbound packets (disconnect, pre-XTEA).
pub(crate) fn frame_plaintext_packet(payload: &[u8]) -> Vec<u8> {
    let inner_len = payload.len() as u16;
    let mut checksummed = Vec::with_capacity(2 + payload.len());
    checksummed.extend_from_slice(&inner_len.to_le_bytes());
    checksummed.extend_from_slice(payload);
    let adler = adler_checksum(&checksummed);
    let outer_len = (4 + checksummed.len()) as u16;
    let mut frame = Vec::with_capacity(2 + outer_len as usize);
    frame.extend_from_slice(&outer_len.to_le_bytes());
    frame.extend_from_slice(&adler.to_le_bytes());
    frame.extend_from_slice(&checksummed);
    frame
}

/// Frame a server→client payload for sending over the wire.
///
/// Wire layout: `[outerLen:2][adler32:4][xtea_region]`
/// where `xtea_region` = XTEA-encrypt(`[innerLen:2][payload]` padded to a
/// multiple of 8 bytes).  This matches C++ `Protocol::onSendMessage` with
/// XTEA encryption enabled (non-sequenced mode).
pub(crate) fn frame_packet(payload: &[u8], xtea_key: [u32; 4]) -> Vec<u8> {
    frame_packet_inner(payload, xtea_key, None)
}

/// Frame a server→client payload using a sequence number instead of Adler-32.
///
/// Used when `sequence_checksum=true` (OTClient, os 4..=12, version >= 1111).
/// OTClient reads the 4-byte header field as a sequence number and treats
/// bit 31 as a decompression flag; sending Adler-32 here risks bit 31 being
/// set and causing OTClient to attempt (and fail) zlib decompression,
/// silently dropping the packet.  Using a monotonically increasing counter
/// (bit 31 never set for the first ~2 billion packets) avoids this.
pub(crate) fn frame_packet_seq(payload: &[u8], xtea_key: [u32; 4], seq: u32) -> Vec<u8> {
    frame_packet_inner(payload, xtea_key, Some(seq))
}

fn frame_packet_inner(payload: &[u8], xtea_key: [u32; 4], seq_override: Option<u32>) -> Vec<u8> {
    let inner_len = payload.len() as u16;
    let content_len = 2 + payload.len();
    let xtea_region_len = if content_len.is_multiple_of(8) {
        content_len
    } else {
        content_len + (8 - content_len % 8)
    };
    let mut xtea_region = vec![0u8; xtea_region_len];
    xtea_region[0..2].copy_from_slice(&inner_len.to_le_bytes());
    xtea_region[2..2 + payload.len()].copy_from_slice(payload);

    let key = xtea::Key(xtea_key);
    let round_keys = xtea::expand_key(&key);
    xtea::encrypt(&mut xtea_region, &round_keys);

    let header = seq_override.unwrap_or_else(|| adler_checksum(&xtea_region));
    let outer_len = (4 + xtea_region_len) as u16;

    let mut frame = Vec::with_capacity(2 + 4 + xtea_region_len);
    frame.extend_from_slice(&outer_len.to_le_bytes());
    frame.extend_from_slice(&header.to_le_bytes());
    frame.extend_from_slice(&xtea_region);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `frame_packet_seq` with small sequence numbers (0, 1, 2, ...) must
    /// never set bit 31 in the 4-byte header field.  OTClient in sequenced
    /// mode interprets bit 31 as a zlib-decompress flag; if it is set the
    /// packet is decompressed (fails) and silently dropped, causing a black
    /// map canvas.
    #[test]
    fn frame_packet_seq_header_bit31_never_set_for_small_sequences() {
        let xtea_key: [u32; 4] = [0x01, 0x02, 0x03, 0x04];
        let payload = [0xA0u8, 0x01, 0x02, 0x03]; // arbitrary payload
        for seq in 0u32..=255 {
            let frame = frame_packet_seq(&payload, xtea_key, seq);
            // frame = [outer_len:2][header:4][xtea_region]
            let header = u32::from_le_bytes([frame[2], frame[3], frame[4], frame[5]]);
            assert_eq!(
                header & (1 << 31),
                0,
                "frame_packet_seq(seq={seq}) has bit 31 set in header 0x{header:08X}"
            );
            assert_eq!(header, seq, "header must equal the sequence number");
        }
    }
}
