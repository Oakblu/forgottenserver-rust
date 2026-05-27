//! Migrated from forgottenserver/src/xtea.h and xtea.cpp
//!
//! Provides XTEA (eXtended Tiny Encryption Algorithm) encrypt/decrypt
//! for 8-byte blocks.  Mirrors the `xtea` namespace from the C++ source.
//!
//! References:
//! - <https://wikipedia.org/wiki/XTEA>
//! - <https://wikipedia.org/wiki/Key_size> (128-bit key)
//!
//! # Usage
//! ```
//! use forgottenserver_common::xtea;
//!
//! let key = xtea::Key([0xdeadbeef, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef]);
//! let round_keys = xtea::expand_key(&key);
//!
//! let mut data = [0xef_u8, 0xbe, 0xad, 0xde, 0xef, 0xbe, 0xad, 0xde];
//! xtea::encrypt(&mut data, &round_keys);
//! ```

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A 128-bit XTEA key (four 32-bit words).
///
/// Mirrors `xtea::key = std::array<uint32_t, 4>` in the C++ header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(pub [u32; 4]);

/// 64 precomputed round keys (one per half-round across 32 full rounds).
///
/// Mirrors `xtea::round_keys = std::array<uint32_t, 64>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundKeys(pub [u32; 64]);

// ---------------------------------------------------------------------------
// Key expansion
// ---------------------------------------------------------------------------

/// Expand a 128-bit key into 64 round keys.
///
/// Mirrors `xtea::expand_key(const key& k)` in xtea.cpp.
///
/// The C++ implementation:
/// ```cpp
/// constexpr uint32_t delta = 0x9E3779B9;
/// for (uint32_t i = 0, sum = 0, next_sum = sum + delta; i < 64;
///      i += 2, sum = next_sum, next_sum += delta) {
///     expanded[i]     = sum      + k[sum & 3];
///     expanded[i + 1] = next_sum + k[(next_sum >> 11) & 3];
/// }
/// ```
pub fn expand_key(k: &Key) -> RoundKeys {
    const DELTA: u32 = 0x9E3779B9;
    let mut expanded = [0u32; 64];
    let key = k.0;

    let mut sum: u32 = 0;
    let mut next_sum: u32 = sum.wrapping_add(DELTA);
    let mut i = 0usize;
    while i < 64 {
        expanded[i] = sum.wrapping_add(key[(sum & 3) as usize]);
        expanded[i + 1] = next_sum.wrapping_add(key[((next_sum >> 11) & 3) as usize]);
        sum = next_sum;
        next_sum = next_sum.wrapping_add(DELTA);
        i += 2;
    }

    RoundKeys(expanded)
}

// ---------------------------------------------------------------------------
// Encrypt
// ---------------------------------------------------------------------------

/// Encrypt `data` in-place using XTEA with the precomputed `round_keys`.
///
/// `data` must contain a whole number of 8-byte blocks; any trailing bytes
/// that do not form a complete block are left unchanged (mirrors C++ which
/// iterates `it < last` over 8-byte strides).
///
/// Mirrors `xtea::encrypt(uint8_t* data, size_t length, const round_keys& k)`.
pub fn encrypt(data: &mut [u8], round_keys: &RoundKeys) {
    let k = &round_keys.0;
    let chunks = data.len() / 8;
    for chunk_idx in 0..chunks {
        let off = chunk_idx * 8;
        let mut left = u32::from_le_bytes(data[off..off + 4].try_into().unwrap());
        let mut right = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap());

        let mut i = 0usize;
        while i < 64 {
            left = left.wrapping_add(((right << 4 ^ right >> 5).wrapping_add(right)) ^ k[i]);
            right = right.wrapping_add(((left << 4 ^ left >> 5).wrapping_add(left)) ^ k[i + 1]);
            i += 2;
        }

        data[off..off + 4].copy_from_slice(&left.to_le_bytes());
        data[off + 4..off + 8].copy_from_slice(&right.to_le_bytes());
    }
}

// ---------------------------------------------------------------------------
// Decrypt
// ---------------------------------------------------------------------------

/// Decrypt `data` in-place using XTEA with the precomputed `round_keys`.
///
/// `data` must contain a whole number of 8-byte blocks; any trailing bytes
/// that do not form a complete block are left unchanged.
///
/// Mirrors `xtea::decrypt(uint8_t* data, size_t length, const round_keys& k)`.
pub fn decrypt(data: &mut [u8], round_keys: &RoundKeys) {
    let k = &round_keys.0;
    let chunks = data.len() / 8;
    for chunk_idx in 0..chunks {
        let off = chunk_idx * 8;
        let mut left = u32::from_le_bytes(data[off..off + 4].try_into().unwrap());
        let mut right = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap());

        // Iterate round keys in reverse (from k[63] down to k[0])
        let mut i = 64usize;
        while i > 0 {
            right = right.wrapping_sub(((left << 4 ^ left >> 5).wrapping_add(left)) ^ k[i - 1]);
            left = left.wrapping_sub(((right << 4 ^ right >> 5).wrapping_add(right)) ^ k[i - 2]);
            i -= 2;
        }

        data[off..off + 4].copy_from_slice(&left.to_le_bytes());
        data[off + 4..off + 8].copy_from_slice(&right.to_le_bytes());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Known key used in test_xtea.cpp
    const KEY: Key = Key([0xdeadbeef, 0xdeadbeef, 0xdeadbeef, 0xdeadbeef]);

    // -----------------------------------------------------------------------
    // expand_key — exact round-key values from test_xtea.cpp
    // -----------------------------------------------------------------------

    #[test]
    fn test_expand_key_known_vector() {
        // Expected round keys from test_xtea.cpp
        let expected = RoundKeys([
            0xdeadbeef, 0x7ce538a8, 0x7ce538a8, 0x1b1cb261, 0x1b1cb261, 0xb9542c1a, 0xb9542c1a,
            0x578ba5d3, 0x578ba5d3, 0xf5c31f8c, 0xf5c31f8c, 0x93fa9945, 0x93fa9945, 0x323212fe,
            0x323212fe, 0xd0698cb7, 0xd0698cb7, 0x6ea10670, 0x6ea10670, 0x0cd88029, 0x0cd88029,
            0xab0ff9e2, 0xab0ff9e2, 0x4947739b, 0x4947739b, 0xe77eed54, 0xe77eed54, 0x85b6670d,
            0x85b6670d, 0x23ede0c6, 0x23ede0c6, 0xc2255a7f, 0xc2255a7f, 0x605cd438, 0x605cd438,
            0xfe944df1, 0xfe944df1, 0x9ccbc7aa, 0x9ccbc7aa, 0x3b034163, 0x3b034163, 0xd93abb1c,
            0xd93abb1c, 0x777234d5, 0x777234d5, 0x15a9ae8e, 0x15a9ae8e, 0xb3e12847, 0xb3e12847,
            0x5218a200, 0x5218a200, 0xf0501bb9, 0xf0501bb9, 0x8e879572, 0x8e879572, 0x2cbf0f2b,
            0x2cbf0f2b, 0xcaf688e4, 0xcaf688e4, 0x692e029d, 0x692e029d, 0x07657c56, 0x07657c56,
            0xa59cf60f,
        ]);

        let actual = expand_key(&KEY);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_expand_key_all_zeros() {
        // Expanding an all-zero key should not panic and produce deterministic output
        let key = Key([0, 0, 0, 0]);
        let rk = expand_key(&key);
        // First round key: sum=0, k[0&3]=k[0]=0 → expanded[0] = 0
        assert_eq!(rk.0[0], 0);
        // Second: next_sum = delta = 0x9E3779B9, k[(delta>>11)&3]
        // (delta >> 11) = 0x13C6EF3 → &3 = 3 → k[3] = 0
        // expanded[1] = 0x9E3779B9 + 0 = 0x9E3779B9
        assert_eq!(rk.0[1], 0x9E3779B9);
    }

    // -----------------------------------------------------------------------
    // encrypt — known vector from test_xtea.cpp
    // -----------------------------------------------------------------------

    #[test]
    fn test_encrypt_known_vector() {
        // From test_xtea.cpp:
        // input:    [0xef, 0xbe, 0xad, 0xde, 0xef, 0xbe, 0xad, 0xde]
        // expected: [0xb5, 0x8c, 0xf2, 0xfa, 0xe0, 0xc0, 0x40, 0x09]
        let rk = expand_key(&KEY);
        let mut data = [0xef_u8, 0xbe, 0xad, 0xde, 0xef, 0xbe, 0xad, 0xde];
        encrypt(&mut data, &rk);
        assert_eq!(data, [0xb5, 0x8c, 0xf2, 0xfa, 0xe0, 0xc0, 0x40, 0x09]);
    }

    // -----------------------------------------------------------------------
    // decrypt — known vector from test_xtea.cpp
    // -----------------------------------------------------------------------

    #[test]
    fn test_decrypt_known_vector() {
        // From test_xtea.cpp:
        // input:    [0xb5, 0x8c, 0xf2, 0xfa, 0xe0, 0xc0, 0x40, 0x09]
        // expected: [0xef, 0xbe, 0xad, 0xde, 0xef, 0xbe, 0xad, 0xde]
        let rk = expand_key(&KEY);
        let mut data = [0xb5_u8, 0x8c, 0xf2, 0xfa, 0xe0, 0xc0, 0x40, 0x09];
        decrypt(&mut data, &rk);
        assert_eq!(data, [0xef, 0xbe, 0xad, 0xde, 0xef, 0xbe, 0xad, 0xde]);
    }

    // -----------------------------------------------------------------------
    // round-trip: decrypt(encrypt(x)) == x
    // -----------------------------------------------------------------------

    #[test]
    fn test_round_trip_single_block() {
        let rk = expand_key(&KEY);
        let original = [0x01_u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut data = original;
        encrypt(&mut data, &rk);
        assert_ne!(
            data, original,
            "encrypted data should differ from plaintext"
        );
        decrypt(&mut data, &rk);
        assert_eq!(data, original, "round-trip should recover plaintext");
    }

    #[test]
    fn test_round_trip_multiple_blocks() {
        let rk = expand_key(&KEY);
        let original = [0xAB_u8; 32]; // 4 blocks
        let mut data = original;
        encrypt(&mut data, &rk);
        decrypt(&mut data, &rk);
        assert_eq!(data, original);
    }

    #[test]
    fn test_round_trip_all_zeros_block() {
        let rk = expand_key(&KEY);
        let original = [0u8; 8];
        let mut data = original;
        encrypt(&mut data, &rk);
        decrypt(&mut data, &rk);
        assert_eq!(data, original);
    }

    // -----------------------------------------------------------------------
    // edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_encrypt_empty_data_is_noop() {
        let rk = expand_key(&KEY);
        let mut data: [u8; 0] = [];
        encrypt(&mut data, &rk); // must not panic
    }

    #[test]
    fn test_decrypt_empty_data_is_noop() {
        let rk = expand_key(&KEY);
        let mut data: [u8; 0] = [];
        decrypt(&mut data, &rk); // must not panic
    }

    #[test]
    fn test_encrypt_partial_block_leaves_remainder_unchanged() {
        // 11 bytes = 1 full block (8) + 3 trailing bytes
        let rk = expand_key(&KEY);
        let mut data = [0x11_u8; 11];
        let tail_before = [data[8], data[9], data[10]];
        encrypt(&mut data, &rk);
        // The 3 trailing bytes must be untouched
        assert_eq!([data[8], data[9], data[10]], tail_before);
    }

    #[test]
    fn test_decrypt_partial_block_leaves_remainder_unchanged() {
        let rk = expand_key(&KEY);
        let mut data = [0x22_u8; 11];
        let tail_before = [data[8], data[9], data[10]];
        decrypt(&mut data, &rk);
        assert_eq!([data[8], data[9], data[10]], tail_before);
    }

    #[test]
    fn test_encrypt_different_keys_produce_different_ciphertext() {
        let key_a = Key([0x01, 0x02, 0x03, 0x04]);
        let key_b = Key([0x05, 0x06, 0x07, 0x08]);
        let rk_a = expand_key(&key_a);
        let rk_b = expand_key(&key_b);

        let plaintext = [0xAA_u8; 8];
        let mut ct_a = plaintext;
        let mut ct_b = plaintext;

        encrypt(&mut ct_a, &rk_a);
        encrypt(&mut ct_b, &rk_b);

        assert_ne!(
            ct_a, ct_b,
            "different keys must produce different ciphertext"
        );
    }

    #[test]
    fn test_wrong_key_decrypt_does_not_recover_plaintext() {
        let key_enc = Key([0xABCDEF01, 0x12345678, 0xDEADBEEF, 0xCAFEBABE]);
        let key_dec = Key([0x11111111, 0x22222222, 0x33333333, 0x44444444]);
        let rk_enc = expand_key(&key_enc);
        let rk_dec = expand_key(&key_dec);

        let original = [0x55_u8; 8];
        let mut data = original;
        encrypt(&mut data, &rk_enc);
        decrypt(&mut data, &rk_dec);
        assert_ne!(
            data, original,
            "wrong-key decrypt should not recover plaintext"
        );
    }

    // -----------------------------------------------------------------------
    // Mode: confirm the C++ behavior is ECB-style — each 8-byte block is
    // encrypted independently with no chaining between blocks. A chained
    // mode (CBC/CTR/etc.) would make this test fail.
    // -----------------------------------------------------------------------
    #[test]
    fn test_blocks_are_independent_ecb_mode() {
        let rk = expand_key(&KEY);

        // Two distinct plaintext blocks, A and B
        let block_a: [u8; 8] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let block_b: [u8; 8] = [0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80];

        // Encrypt them concatenated
        let mut combined = [0u8; 16];
        combined[..8].copy_from_slice(&block_a);
        combined[8..].copy_from_slice(&block_b);
        encrypt(&mut combined, &rk);

        // Encrypt each block independently
        let mut a_alone = block_a;
        let mut b_alone = block_b;
        encrypt(&mut a_alone, &rk);
        encrypt(&mut b_alone, &rk);

        // ECB invariant: enc(A||B) == enc(A) || enc(B)
        assert_eq!(
            &combined[..8],
            &a_alone,
            "block 0 must encrypt independently"
        );
        assert_eq!(
            &combined[8..],
            &b_alone,
            "block 1 must encrypt independently"
        );

        // And the two ciphertext blocks must themselves differ (distinct
        // plaintexts under the same key) — guards against accidentally
        // emitting plaintext through both blocks.
        assert_ne!(a_alone, b_alone);
    }

    // -----------------------------------------------------------------------
    // Multi-block round-trip with *distinct* per-block contents. Catches
    // bugs that the all-0xAB multi-block test cannot (e.g. swapping blocks
    // or encrypting the same block twice).
    // -----------------------------------------------------------------------
    #[test]
    fn test_round_trip_multiple_distinct_blocks_preserves_order() {
        let rk = expand_key(&KEY);
        // 3 blocks, each filled with a unique byte
        let original: [u8; 24] = [
            0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, // block 0
            0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, // block 1
            0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, // block 2
        ];
        let mut data = original;
        encrypt(&mut data, &rk);
        // Each ciphertext block must differ from its plaintext
        assert_ne!(&data[0..8], &original[0..8]);
        assert_ne!(&data[8..16], &original[8..16]);
        assert_ne!(&data[16..24], &original[16..24]);
        decrypt(&mut data, &rk);
        assert_eq!(
            data, original,
            "multi-block round-trip must preserve byte order"
        );
    }

    // -----------------------------------------------------------------------
    // Confirm that the implementation actually runs all 32 rounds (64 round
    // keys). Running fewer rounds against the known KEY would not produce
    // the published ciphertext — this is implicitly covered by
    // test_encrypt_known_vector, but we additionally assert that swapping a
    // single round key changes the output, proving every round key matters.
    // -----------------------------------------------------------------------
    #[test]
    fn test_every_round_key_is_used() {
        let rk_full = expand_key(&KEY);
        let plaintext = [0xef_u8, 0xbe, 0xad, 0xde, 0xef, 0xbe, 0xad, 0xde];

        let mut baseline = plaintext;
        encrypt(&mut baseline, &rk_full);

        // Perturb the last round key and confirm the ciphertext changes.
        // If the implementation stopped early (used <64 keys), perturbing
        // a late key would have no effect.
        let mut rk_perturbed = rk_full.clone();
        rk_perturbed.0[63] ^= 0x1;
        let mut perturbed = plaintext;
        encrypt(&mut perturbed, &rk_perturbed);
        assert_ne!(
            baseline, perturbed,
            "perturbing k[63] must change ciphertext"
        );

        // Same for k[0] — guards against the implementation skipping the
        // first round.
        let mut rk_perturbed_first = rk_full.clone();
        rk_perturbed_first.0[0] ^= 0x1;
        let mut perturbed_first = plaintext;
        encrypt(&mut perturbed_first, &rk_perturbed_first);
        assert_ne!(
            baseline, perturbed_first,
            "perturbing k[0] must change ciphertext"
        );
    }
}
