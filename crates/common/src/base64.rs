/// Standard Base64 alphabet (RFC 4648).
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes `input` bytes into a standard Base64 string (RFC 4648, with `=` padding).
///
/// Mirrors the behaviour of `tfs::base64::encode`, which uses OpenSSL's
/// `BIO_f_base64` with `BIO_FLAGS_BASE64_NO_NL` (no line breaks, standard
/// alphabet, padded output).
pub fn encode(input: &[u8]) -> String {
    if input.is_empty() {
        return String::new();
    }

    let output_len = input.len().div_ceil(3) * 4;
    let mut out = Vec::with_capacity(output_len);

    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let combined = (b0 << 16) | (b1 << 8) | b2;

        out.push(ALPHABET[((combined >> 18) & 0x3F) as usize]);
        out.push(ALPHABET[((combined >> 12) & 0x3F) as usize]);
        if chunk.len() > 1 {
            out.push(ALPHABET[((combined >> 6) & 0x3F) as usize]);
        } else {
            out.push(b'=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(combined & 0x3F) as usize]);
        } else {
            out.push(b'=');
        }
    }

    // SAFETY: every byte written is an ASCII Base64 character.
    unsafe { String::from_utf8_unchecked(out) }
}

/// Decodes a standard Base64 string into raw bytes.
///
/// Returns `Err` when the input length is not a multiple of 4, or when an
/// illegal character is encountered.  This mirrors the OpenSSL `BIO_read`
/// behaviour used by `tfs::base64::decode` (invalid input produces an empty /
/// truncated result; we surface that as `Err`).
pub fn decode(input: &str) -> Result<Vec<u8>, String> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    if !input.len().is_multiple_of(4) {
        return Err(format!(
            "base64 decode: input length {} is not a multiple of 4",
            input.len()
        ));
    }

    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(input.len() / 4 * 3);

    for chunk in bytes.chunks(4) {
        let c0 = decode_char(chunk[0])?;
        let c1 = decode_char(chunk[1])?;

        // Third character may be '='
        let (c2, pad2) = if chunk[2] == b'=' {
            (0u32, true)
        } else {
            (decode_char(chunk[2])? as u32, false)
        };

        // Fourth character may be '='
        let (c3, pad3) = if chunk[3] == b'=' {
            (0u32, true)
        } else {
            (decode_char(chunk[3])? as u32, false)
        };

        // Validate padding consistency: '=' in position 3 requires '=' in pos 4
        if pad2 && !pad3 {
            return Err("base64 decode: invalid padding".to_string());
        }

        let combined = ((c0 as u32) << 18) | ((c1 as u32) << 12) | (c2 << 6) | c3;

        out.push(((combined >> 16) & 0xFF) as u8);
        if !pad2 {
            out.push(((combined >> 8) & 0xFF) as u8);
        }
        if !pad3 {
            out.push((combined & 0xFF) as u8);
        }
    }

    Ok(out)
}

/// Decode a single Base64 alphabet character to its 6-bit value.
fn decode_char(c: u8) -> Result<u32, String> {
    match c {
        b'A'..=b'Z' => Ok((c - b'A') as u32),
        b'a'..=b'z' => Ok((c - b'a' + 26) as u32),
        b'0'..=b'9' => Ok((c - b'0' + 52) as u32),
        b'+' => Ok(62),
        b'/' => Ok(63),
        other => Err(format!(
            "base64 decode: invalid character '{}'",
            other as char
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // RFC 4648 §10 test vectors (same set used by test_base64.cpp)
    // -------------------------------------------------------------------------

    #[test]
    fn test_encode_empty() {
        assert_eq!(encode(b""), "");
    }

    #[test]
    fn test_encode_f() {
        assert_eq!(encode(b"f"), "Zg==");
    }

    #[test]
    fn test_encode_fo() {
        assert_eq!(encode(b"fo"), "Zm8=");
    }

    #[test]
    fn test_encode_foo() {
        assert_eq!(encode(b"foo"), "Zm9v");
    }

    #[test]
    fn test_encode_foob() {
        assert_eq!(encode(b"foob"), "Zm9vYg==");
    }

    #[test]
    fn test_encode_fooba() {
        assert_eq!(encode(b"fooba"), "Zm9vYmE=");
    }

    #[test]
    fn test_encode_foobar() {
        assert_eq!(encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_encode_man() {
        // Classic example from Wikipedia / RFC
        assert_eq!(encode(b"Man"), "TWFu");
    }

    // -------------------------------------------------------------------------
    // Decode — RFC 4648 §10 vectors
    // -------------------------------------------------------------------------

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode("").unwrap(), b"");
    }

    #[test]
    fn test_decode_zg() {
        assert_eq!(decode("Zg==").unwrap(), b"f");
    }

    #[test]
    fn test_decode_zm8() {
        assert_eq!(decode("Zm8=").unwrap(), b"fo");
    }

    #[test]
    fn test_decode_zm9v() {
        assert_eq!(decode("Zm9v").unwrap(), b"foo");
    }

    #[test]
    fn test_decode_zm9vyg() {
        assert_eq!(decode("Zm9vYg==").unwrap(), b"foob");
    }

    #[test]
    fn test_decode_zm9vyme() {
        assert_eq!(decode("Zm9vYmE=").unwrap(), b"fooba");
    }

    #[test]
    fn test_decode_zm9vymfy() {
        assert_eq!(decode("Zm9vYmFy").unwrap(), b"foobar");
    }

    #[test]
    fn test_decode_man() {
        assert_eq!(decode("TWFu").unwrap(), b"Man");
    }

    // -------------------------------------------------------------------------
    // Round-trip: encode → decode for all 256 single-byte values
    // -------------------------------------------------------------------------

    #[test]
    fn test_round_trip_all_single_bytes() {
        for byte in 0u8..=255 {
            let encoded = encode(&[byte]);
            let decoded = decode(&encoded).expect("round-trip decode failed");
            assert_eq!(
                decoded,
                vec![byte],
                "round-trip failed for byte 0x{byte:02X}"
            );
        }
    }

    #[test]
    fn test_round_trip_all_two_byte_combinations_spot_check() {
        // Spot-check a selection of two-byte inputs for round-trip correctness
        for a in [0u8, 1, 127, 128, 255] {
            for b in [0u8, 1, 127, 128, 255] {
                let input = [a, b];
                let encoded = encode(&input);
                let decoded = decode(&encoded).expect("round-trip decode failed");
                assert_eq!(decoded, input.to_vec(), "round-trip failed for [{a}, {b}]");
            }
        }
    }

    #[test]
    fn test_round_trip_arbitrary_bytes() {
        let input: Vec<u8> = (0u8..=255).collect();
        let encoded = encode(&input);
        let decoded = decode(&encoded).expect("round-trip decode failed");
        assert_eq!(decoded, input);
    }

    // -------------------------------------------------------------------------
    // Error cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_decode_invalid_length_not_multiple_of_4() {
        // 1, 2, 3 chars — all invalid
        assert!(decode("Z").is_err());
        assert!(decode("Zg").is_err());
        assert!(decode("Zg=").is_err());
    }

    #[test]
    fn test_decode_invalid_character() {
        // '@' is not in the Base64 alphabet
        assert!(decode("Zg@=").is_err());
    }

    #[test]
    fn test_decode_invalid_padding_position() {
        // '=' in position 3 but not in position 4 is illegal
        assert!(decode("Zm=v").is_err());
    }

    #[test]
    fn test_encode_binary_zeros() {
        assert_eq!(encode(&[0u8, 0, 0]), "AAAA");
    }

    #[test]
    fn test_encode_all_ones() {
        assert_eq!(encode(&[0xFFu8, 0xFF, 0xFF]), "////");
    }

    #[test]
    fn test_decode_aaaa() {
        assert_eq!(decode("AAAA").unwrap(), vec![0u8, 0, 0]);
    }

    #[test]
    fn test_decode_slashes() {
        assert_eq!(decode("////").unwrap(), vec![0xFFu8, 0xFF, 0xFF]);
    }

    #[test]
    fn test_encode_longer_string() {
        // "Hello, World!" — well-known vector
        assert_eq!(encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_decode_hello_world() {
        assert_eq!(decode("SGVsbG8sIFdvcmxkIQ==").unwrap(), b"Hello, World!");
    }

    #[test]
    fn test_encode_newline() {
        assert_eq!(encode(b"\n"), "Cg==");
    }

    #[test]
    fn test_decode_newline() {
        assert_eq!(decode("Cg==").unwrap(), b"\n");
    }

    // -------------------------------------------------------------------------
    // Padding-only inputs: "", "=", "==" must be rejected or return empty
    // -------------------------------------------------------------------------

    /// decode("") → Ok(empty) — the empty string is the valid empty input.
    #[test]
    fn test_decode_empty_string_padding_variant() {
        // "" is the zero-byte base64 — already covered by test_decode_empty
        // but this test is explicit about the "padding variant" contract.
        let result = decode("");
        assert!(result.is_ok(), "empty string should decode to Ok");
        assert!(
            result.unwrap().is_empty(),
            "empty string should decode to empty bytes"
        );
    }

    /// decode("=") → Err — a lone "=" is not a valid multiple-of-4 input.
    #[test]
    fn test_decode_single_equals_is_error() {
        let result = decode("=");
        assert!(
            result.is_err(),
            "\"=\" alone is not valid base64 (length 1 is not multiple of 4)"
        );
    }

    /// decode("==") → Err — two equals signs is not a valid multiple-of-4 input.
    #[test]
    fn test_decode_double_equals_is_error() {
        let result = decode("==");
        assert!(
            result.is_err(),
            "\"==\" alone is not valid base64 (length 2 is not multiple of 4)"
        );
    }

    /// decode("===") → Err — three equals signs is not a valid multiple-of-4 input.
    #[test]
    fn test_decode_triple_equals_is_error() {
        let result = decode("===");
        assert!(
            result.is_err(),
            "\"===\" alone is not valid base64 (length 3 is not multiple of 4)"
        );
    }

    /// decode("====") → Err — four equals would be multiple of 4 but "=" is not
    /// a valid first or second character (only positions 3/4 may be padding).
    #[test]
    fn test_decode_quad_equals_is_error() {
        // '=' is not in the base64 alphabet so decode_char rejects it in positions 1/2.
        let result = decode("====");
        assert!(
            result.is_err(),
            "\"====\" should be rejected: '=' is not a valid base64 character in positions 1 or 2"
        );
    }

    // -------------------------------------------------------------------------
    // BIO_FLAGS_BASE64_NO_NL semantics — encoded output must never contain
    // line-break characters, regardless of input length. The C++ implementation
    // sets BIO_FLAGS_BASE64_NO_NL explicitly; the Rust port must match.
    // -------------------------------------------------------------------------

    #[test]
    fn test_encode_no_newlines_in_output_long_input() {
        // An input large enough that OpenSSL's default base64 BIO would emit
        // line breaks every 64 chars. The NO_NL flag suppresses these.
        let input: Vec<u8> = (0u8..=255).cycle().take(300).collect();
        let encoded = encode(&input);
        assert!(
            !encoded.contains('\n'),
            "encoded output must not contain newlines (BIO_FLAGS_BASE64_NO_NL)"
        );
        assert!(
            !encoded.contains('\r'),
            "encoded output must not contain carriage returns"
        );
        // Sanity: round-trip still works.
        assert_eq!(decode(&encoded).unwrap(), input);
    }

    // -------------------------------------------------------------------------
    // Invalid-character coverage at every position inside a 4-character chunk.
    // Position 0 (chunk[0]) and position 2 (chunk[2]) are already covered by
    // test_decode_invalid_character ("Zg@="). Positions 1 (chunk[1]) and 3
    // (chunk[3]) need their own tests so every `?` early-return in decode is
    // observably exercised.
    // -------------------------------------------------------------------------

    /// '@' at position 1 (chunk[1]) — decode_char on chunk[1] must propagate
    /// the error.
    #[test]
    fn test_decode_invalid_character_at_pos_1() {
        let result = decode("Z@g=");
        assert!(result.is_err(), "'@' at chunk position 1 must be rejected");
        let err = result.unwrap_err();
        assert!(
            err.contains("invalid character"),
            "error must describe an invalid character; got: {err}"
        );
    }

    /// '@' at position 3 (chunk[3], non-padding slot) — decode_char on chunk[3]
    /// must propagate the error when chunk[2] is a real alphabet character.
    #[test]
    fn test_decode_invalid_character_at_pos_3() {
        // "Zm9@" — first three are valid base64; fourth is '@' (invalid, not '=')
        let result = decode("Zm9@");
        assert!(
            result.is_err(),
            "'@' at chunk position 3 (non-padding slot) must be rejected"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("invalid character"),
            "error must describe an invalid character; got: {err}"
        );
    }
}
