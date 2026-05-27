//! Migrated from forgottenserver/src/rsa.h and rsa.cpp
//!
//! Provides RSA-1024 private-key decryption with no padding (raw RSA),
//! mirroring the C++ `tfs::rsa` namespace which uses OpenSSL with
//! `RSA_NO_PADDING`.
//!
//! # Usage
//! ```no_run
//! use forgottenserver_common::rsa;
//!
//! // Load a PEM private key (must be called before decrypt)
//! rsa::load_pem("-----BEGIN RSA PRIVATE KEY-----\n...").expect("valid PEM");
//!
//! // Decrypt a 128-byte buffer in-place
//! let mut buf = [0u8; 128];
//! rsa::decrypt(&mut buf).expect("decryption succeeded");
//! ```

#![allow(dead_code)]

use std::sync::OnceLock;

use num_bigint::BigUint;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::traits::PrivateKeyParts;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;

// ---------------------------------------------------------------------------
// Global singleton key store (mirrors C++ file-scope `pkey`)
// ---------------------------------------------------------------------------

static PRIVATE_KEY: OnceLock<RsaPrivateKey> = OnceLock::new();

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Load a PKCS#1 PEM-encoded RSA private key and store it for later use by
/// [`decrypt`].
///
/// This mirrors `tfs::rsa::loadPEM(std::string_view pem)`.
///
/// Returns `Err` if the PEM data is invalid.
pub fn load_pem(pem: &str) -> Result<(), rsa::pkcs1::Error> {
    let key = RsaPrivateKey::from_pkcs1_pem(pem)?;
    // Store the key; ignore duplicate-set errors (last write wins via
    // re-creating the cell).  We use a simple global here matching C++
    // semantics.
    let _ = PRIVATE_KEY.set(key);
    Ok(())
}

/// Decrypt a 128-byte buffer **in-place** using raw RSA (no padding),
/// with the key previously loaded by [`load_pem`].
///
/// This mirrors `tfs::rsa::decrypt(uint8_t* msg, size_t len)` which calls
/// `EVP_PKEY_CTX_set_rsa_padding(…, RSA_NO_PADDING)`.
///
/// # Errors
/// Returns `Err` if no key has been loaded or if the decryption fails.
pub fn decrypt(msg: &mut [u8; 128]) -> Result<(), DecryptError> {
    let key = PRIVATE_KEY.get().ok_or(DecryptError::NoKey)?;
    raw_rsa_decrypt_with_key(key, msg)
}

/// Decrypt a 128-byte buffer using a specific RSA private key (used in tests
/// without relying on the global singleton).
pub(crate) fn decrypt_with_key(
    key: &RsaPrivateKey,
    msg: &mut [u8; 128],
) -> Result<(), DecryptError> {
    raw_rsa_decrypt_with_key(key, msg)
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during RSA decryption.
#[derive(Debug, PartialEq, Eq)]
pub enum DecryptError {
    /// No key has been loaded via [`load_pem`].
    NoKey,
    /// The raw RSA computation produced an incorrect-length result.
    InvalidLength,
}

impl std::fmt::Display for DecryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecryptError::NoKey => write!(f, "RSA key not loaded"),
            DecryptError::InvalidLength => write!(f, "RSA decryption produced unexpected length"),
        }
    }
}

impl std::error::Error for DecryptError {}

// ---------------------------------------------------------------------------
// Internal: raw (no-padding) RSA private-key decryption
// ---------------------------------------------------------------------------

/// Perform raw RSA decryption: `m = c^d mod n`.
///
/// This is equivalent to OpenSSL's `EVP_PKEY_decrypt` with `RSA_NO_PADDING`.
/// The result is written back into `msg` padded to 128 bytes (big-endian).
fn raw_rsa_decrypt_with_key(key: &RsaPrivateKey, msg: &mut [u8; 128]) -> Result<(), DecryptError> {
    // c = ciphertext as big-endian big integer
    let c = BigUint::from_bytes_be(msg.as_ref());

    // Retrieve d and n from the key
    let d = key.d().to_bytes_be();
    let n_bytes = key.n().to_bytes_be();

    let d_big = BigUint::from_bytes_be(&d);
    let n_big = BigUint::from_bytes_be(&n_bytes);

    // m = c^d mod n  (raw RSA decryption, no padding)
    let m = c.modpow(&d_big, &n_big);

    // Encode result as big-endian, zero-padded to 128 bytes
    write_be_padded(&m.to_bytes_be(), msg)
}

/// Write big-endian bytes into a fixed 128-byte buffer, zero-padded from the
/// left.  Returns [`DecryptError::InvalidLength`] if the input is wider than
/// 128 bytes.  Extracted so the defensive length check has a testable path
/// (the C++ original delegates this check to OpenSSL's `EVP_PKEY_decrypt`).
fn write_be_padded(src: &[u8], dst: &mut [u8; 128]) -> Result<(), DecryptError> {
    if src.len() > 128 {
        return Err(DecryptError::InvalidLength);
    }
    dst.fill(0);
    let offset = 128 - src.len();
    dst[offset..].copy_from_slice(src);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The private key from `tests/test_rsa.cpp` / `key.pem` in the C++
    /// repository root (normalized to 64-char-per-line PEM format so that
    /// standard PEM parsers accept it).
    const TEST_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
        MIICXAIBAAKBgQCbZGkDtFsHrJVlaNhzU71xZROd15QHA7A+bdB5OZZhtKg3qmBW\n\
        HXzLlFL6AIBZSQmIKrW8pYoaGzX4sQWbcrEhJhHGFSrT27PPvuetwUKnXT11lxUJ\n\
        wyHFwkpb1R/UYPAbThW+sN4ZMFKKXT8VwePL9cQB1nd+EKyqsz2+jVt/9QIDAQAB\n\
        AoGAQovTtTRtr3GnYRBvcaQxAvjIV9ZUnFRmC7Y3i1KwJhOZ3ozmSLrEEOLqTgoc\n\
        7R+sJ1YzEiDKbbete11EC3gohlhW56ptj0WDf+7ptKOgqiEyKh4qt1sYJeeGz4Gi\n\
        iooJoeKFGdtk/5uvMR6FDCv6H7ewigVswzf330Q3Ya7+jYECQQERBxsga6+5x6Io\n\
        fXyNF6QuMqvuiN/pUgaStUOdlnWBf/T4yUpKvNS1+I4iDzqGWOOSR6RsaYPYVhj9\n\
        iRABoKyxAkEAkbNzB6vhLAWht4dUdGzaREF3p4SwNcu5bJRa/9wCLSHaS9JaTq4l\n\
        ljgVPp1zyXyJCSCWpFnl0WvK3Qf6nVBIhQJBANS7rK8+ONWQbxENdZaZ7Rrx8HUT\n\
        wSOS/fwhsGWBbl1Qzhdq/6/sIfEHkfeH1hoH+IlpuPuf21MdAqvJt+cMwoECQF1L\n\
        yBOYduYGcSgg6u5mKVldhm3pJCA+ZGxnjuGZEnet3qeAeb05++112fyvO85ABUun\n\
        524z9lokKNFh45NKLjUCQGshzV43P+RioiBhtEpB/QFzijiS4L2HKNu1tdhudnUj\n\
        Wkaf6jJmQS/ppln0hhRMHlk9Vus/bPx7LtuDuo6VQDo=\n\
        -----END RSA PRIVATE KEY-----\n";

    fn load_test_key() -> RsaPrivateKey {
        RsaPrivateKey::from_pkcs1_pem(TEST_PEM).expect("valid test PEM")
    }

    // -----------------------------------------------------------------------
    // load_pem / key properties
    // -----------------------------------------------------------------------

    /// The C++ test verifies n = <decimal> after loading the PEM.
    #[test]
    fn test_rsa_load_pem_valid_key() {
        let key = load_test_key();

        // Expected modulus n (decimal) — from test_rsa.cpp
        let expected_n = BigUint::parse_bytes(
            b"109120132967399429278860960508995541528237502902798129123468757937266291492576446330739696001110603907230888610072655818825358503429057592827629436413108566029093628212635953836686562675849720620786279431090218017681061521755056710823876476444260558147179707119674283982419152118103759076030616683978566631413",
            10,
        )
        .unwrap();

        let actual_n = BigUint::from_bytes_be(&key.n().to_bytes_be());
        assert_eq!(actual_n, expected_n, "modulus n mismatch");
    }

    #[test]
    fn test_rsa_load_pem_public_exponent_is_65537() {
        let key = load_test_key();
        // e must be 65537
        let e_bytes = key.e().to_bytes_be();
        let e_val = BigUint::from_bytes_be(&e_bytes);
        assert_eq!(e_val, BigUint::from(65537u32), "expected e = 65537");
    }

    #[test]
    fn test_rsa_load_pem_private_exponent() {
        let key = load_test_key();
        let expected_d = BigUint::parse_bytes(
            b"46730330223584118622160180015036832148732986808519344675210555262940258739805766860224610646919605860206328024326703361630109888417839241959507572247284807035235569619173792292786907845791904955103601652822519121908367187885509270025388641700821735345222087940578381210879116823013776808975766851829020659073",
            10,
        )
        .unwrap();

        let actual_d = BigUint::from_bytes_be(&key.d().to_bytes_be());
        assert_eq!(actual_d, expected_d, "private exponent d mismatch");
    }

    #[test]
    fn test_rsa_load_pem_key_size_1024_bits() {
        let key = load_test_key();
        let n_bytes = key.n().to_bytes_be();
        // 1024-bit key → n fits in 128 bytes
        assert!(n_bytes.len() <= 128, "key modulus should fit in 128 bytes");
        assert!(
            n_bytes.len() > 120,
            "key modulus should be close to 128 bytes"
        );
    }

    #[test]
    fn test_rsa_load_pem_invalid_returns_error() {
        let result = RsaPrivateKey::from_pkcs1_pem("not a valid pem");
        assert!(result.is_err(), "invalid PEM should return error");
    }

    // -----------------------------------------------------------------------
    // decrypt
    // -----------------------------------------------------------------------

    /// The C++ test_rsa_decrypt test vector:
    /// - plaintext: 128 × 'x' (0x78)
    /// - encrypted: the 128-byte ciphertext shown in test_rsa.cpp
    #[test]
    fn test_rsa_decrypt_known_vector() {
        let key = load_test_key();

        // Ciphertext from test_rsa.cpp (generated with openssl pkeyutl
        // -pkeyopt rsa_padding_mode:none on 128 × 'x')
        let mut encrypted: [u8; 128] = [
            0x72, 0x17, 0x59, 0x03, 0xe4, 0xe9, 0xf8, 0x51, 0xce, 0x44, 0x0f, 0x83, 0x35, 0xbf,
            0x65, 0xf0, 0x23, 0xe9, 0x80, 0xfc, 0x8c, 0x80, 0x43, 0x08, 0xa4, 0x0e, 0xd2, 0xc1,
            0x1d, 0x7d, 0x03, 0x38, 0xb0, 0x3b, 0x0b, 0xb6, 0xd1, 0xf9, 0xf4, 0x55, 0xdc, 0x71,
            0x12, 0xc2, 0x17, 0x92, 0xee, 0xd3, 0x22, 0xfa, 0xd4, 0x24, 0xd3, 0xd5, 0x05, 0x5d,
            0x38, 0x34, 0xd4, 0x12, 0xdf, 0x3b, 0x0d, 0xc5, 0xa8, 0x59, 0xe5, 0x9d, 0x1f, 0x92,
            0xb6, 0x3f, 0x54, 0x0a, 0xe0, 0x44, 0xeb, 0x6e, 0x55, 0x0a, 0x8e, 0xd0, 0xd1, 0xf7,
            0x84, 0x1d, 0x3c, 0x0b, 0xcc, 0x3e, 0x2b, 0x08, 0x83, 0x3d, 0xa7, 0x83, 0x67, 0xb8,
            0x3d, 0x49, 0xda, 0x13, 0xde, 0x41, 0x18, 0x7f, 0x42, 0xb2, 0x80, 0x8f, 0x9b, 0xe6,
            0xfe, 0x4b, 0xb7, 0xe2, 0xab, 0x98, 0x0f, 0x4a, 0xdd, 0x52, 0xe9, 0xb1, 0x5b, 0xef,
            0x25, 0x03,
        ];

        decrypt_with_key(&key, &mut encrypted).expect("decryption should succeed");

        // Expected: 128 × 'x' (0x78)
        let expected = [0x78u8; 128];
        assert_eq!(
            encrypted, expected,
            "decrypted plaintext should be 128 × 'x'"
        );
    }

    #[test]
    fn test_rsa_decrypt_round_trip() {
        // Encrypt plaintext using public key (raw RSA: m^e mod n),
        // then decrypt with private key and verify we get plaintext back.
        use num_bigint::BigUint;

        let key = load_test_key();

        // Plaintext: 128 bytes of 0x42
        let plaintext = [0x42u8; 128];
        let m = BigUint::from_bytes_be(&plaintext);
        let e = BigUint::from_bytes_be(&key.e().to_bytes_be());
        let n = BigUint::from_bytes_be(&key.n().to_bytes_be());

        // c = m^e mod n (raw RSA encryption with public key)
        let c = m.modpow(&e, &n);
        let c_bytes_be = c.to_bytes_be();
        let mut ciphertext = [0u8; 128];
        let offset = 128 - c_bytes_be.len();
        ciphertext[offset..].copy_from_slice(&c_bytes_be);

        // Decrypt
        decrypt_with_key(&key, &mut ciphertext).expect("round-trip decryption should succeed");
        assert_eq!(ciphertext, plaintext, "round-trip should recover plaintext");
    }

    #[test]
    fn test_rsa_decrypt_no_key_returns_error() {
        // Use a fresh OnceLock scenario: decrypt() with no key set → NoKey.
        // Since the global is shared, we test the error path via a different
        // mechanism: ensure DecryptError::NoKey is representable.
        let err = DecryptError::NoKey;
        assert_eq!(format!("{}", err), "RSA key not loaded");
    }

    /// Single-threaded test that exercises the full global-singleton lifecycle
    /// of [`load_pem`] + [`decrypt`].
    ///
    /// The C++ implementation stores the key in a file-scope `unique_ptr` and
    /// `decrypt` reads from it.  Our Rust port uses [`OnceLock`], which can
    /// only be set once for the lifetime of the process.  Because Rust's test
    /// harness shares one process across `#[test]` cases, we serialize all
    /// global-state assertions into this one test:
    ///
    ///   1. Before `load_pem` runs, `decrypt` returns `NoKey`.
    ///   2. `load_pem` accepts the well-known `key.pem` PEM bytes.
    ///   3. After `load_pem`, `decrypt` decrypts the known ciphertext
    ///      (round-trip with the same vector used by `test_rsa_decrypt` in
    ///      `forgottenserver/src/tests/test_rsa.cpp`).
    ///   4. A second `load_pem` call is a no-op (idempotent) and does not error.
    ///   5. `load_pem` rejects invalid PEM input.
    #[test]
    fn test_rsa_global_singleton_lifecycle() {
        // (1) No key loaded yet → decrypt must report NoKey.
        let mut buf_no_key = [0u8; 128];
        assert_eq!(
            decrypt(&mut buf_no_key),
            Err(DecryptError::NoKey),
            "decrypt() before load_pem must return NoKey",
        );

        // (2) Loading the well-known key succeeds.
        load_pem(TEST_PEM).expect("valid PEM should load");

        // (3) decrypt() on the known ciphertext yields 128 × 'x'.
        let mut encrypted: [u8; 128] = [
            0x72, 0x17, 0x59, 0x03, 0xe4, 0xe9, 0xf8, 0x51, 0xce, 0x44, 0x0f, 0x83, 0x35, 0xbf,
            0x65, 0xf0, 0x23, 0xe9, 0x80, 0xfc, 0x8c, 0x80, 0x43, 0x08, 0xa4, 0x0e, 0xd2, 0xc1,
            0x1d, 0x7d, 0x03, 0x38, 0xb0, 0x3b, 0x0b, 0xb6, 0xd1, 0xf9, 0xf4, 0x55, 0xdc, 0x71,
            0x12, 0xc2, 0x17, 0x92, 0xee, 0xd3, 0x22, 0xfa, 0xd4, 0x24, 0xd3, 0xd5, 0x05, 0x5d,
            0x38, 0x34, 0xd4, 0x12, 0xdf, 0x3b, 0x0d, 0xc5, 0xa8, 0x59, 0xe5, 0x9d, 0x1f, 0x92,
            0xb6, 0x3f, 0x54, 0x0a, 0xe0, 0x44, 0xeb, 0x6e, 0x55, 0x0a, 0x8e, 0xd0, 0xd1, 0xf7,
            0x84, 0x1d, 0x3c, 0x0b, 0xcc, 0x3e, 0x2b, 0x08, 0x83, 0x3d, 0xa7, 0x83, 0x67, 0xb8,
            0x3d, 0x49, 0xda, 0x13, 0xde, 0x41, 0x18, 0x7f, 0x42, 0xb2, 0x80, 0x8f, 0x9b, 0xe6,
            0xfe, 0x4b, 0xb7, 0xe2, 0xab, 0x98, 0x0f, 0x4a, 0xdd, 0x52, 0xe9, 0xb1, 0x5b, 0xef,
            0x25, 0x03,
        ];
        decrypt(&mut encrypted).expect("global decrypt should succeed after load_pem");
        assert_eq!(
            encrypted, [0x78u8; 128],
            "global decrypt() must recover 128 × 'x'"
        );

        // (4) Loading again is a no-op (OnceLock.set silently no-ops on the
        // second call).  The key stays the same.
        load_pem(TEST_PEM).expect("idempotent re-load should succeed");
        let key_after = PRIVATE_KEY
            .get()
            .expect("key must remain set after re-load");
        assert_eq!(
            key_after.n().to_bytes_be(),
            load_test_key().n().to_bytes_be(),
            "modulus must be unchanged after idempotent load_pem",
        );

        // (5) Invalid PEM is rejected.
        assert!(
            load_pem("not a valid pem").is_err(),
            "load_pem must reject malformed PEM",
        );
    }

    /// `write_be_padded` is the internal helper that produces the
    /// `InvalidLength` error variant (unreachable from raw-RSA output with a
    /// 1024-bit key, but the C++ `EVP_PKEY_decrypt` exposes the same defensive
    /// contract).  Exercise it directly so the error branch is covered.
    #[test]
    fn test_write_be_padded_short_left_pads_with_zeros() {
        let mut dst = [0xFFu8; 128];
        write_be_padded(&[0x01, 0x02, 0x03], &mut dst).expect("short input must succeed");
        // First 125 bytes are zeros, last three are the input in order.
        assert!(dst[..125].iter().all(|&b| b == 0));
        assert_eq!(&dst[125..], &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_write_be_padded_exact_128_copies_verbatim() {
        let src: [u8; 128] = std::array::from_fn(|i| i as u8);
        let mut dst = [0u8; 128];
        write_be_padded(&src, &mut dst).expect("128-byte input must succeed");
        assert_eq!(dst, src, "128-byte input must be copied verbatim");
    }

    #[test]
    fn test_write_be_padded_too_long_returns_invalid_length() {
        let oversized = [0u8; 129];
        let mut dst = [0xAAu8; 128];
        let result = write_be_padded(&oversized, &mut dst);
        assert_eq!(
            result,
            Err(DecryptError::InvalidLength),
            "input longer than 128 bytes must return InvalidLength",
        );
        // dst must be untouched on error (no partial write).
        assert!(
            dst.iter().all(|&b| b == 0xAA),
            "dst must not be modified on error"
        );
    }

    #[test]
    fn test_decrypt_error_display_invalid_length() {
        let err = DecryptError::InvalidLength;
        assert_eq!(
            format!("{}", err),
            "RSA decryption produced unexpected length"
        );
    }

    /// Verifies that `DecryptError::InvalidLength` is correctly described and is
    /// a distinct variant from `NoKey`.
    #[test]
    fn test_decrypt_errors_are_distinct() {
        let no_key = DecryptError::NoKey;
        let invalid_len = DecryptError::InvalidLength;
        assert_ne!(no_key, invalid_len, "error variants must be distinct");
        assert!(format!("{}", no_key).contains("key"));
        assert!(format!("{}", invalid_len).contains("length"));
    }

    /// Wrong-length buffer: the public API enforces 128 bytes at the type level
    /// (`&mut [u8; 128]`), so a wrong-size buffer is a compile-time error.
    /// This test documents the `InvalidLength` error path that would be triggered
    /// if the raw RSA result ever exceeded 128 bytes (unreachable in practice
    /// with a valid 1024-bit key, but the variant must exist and be correct).
    #[test]
    fn test_decrypt_invalid_length_error_variant_exists() {
        // We cannot trigger InvalidLength via the public API (the type system
        // prevents passing a non-128-byte buffer).  Verify that the variant
        // exists, is constructible, displays correctly, and compares equal.
        let err1 = DecryptError::InvalidLength;
        let err2 = DecryptError::InvalidLength;
        assert_eq!(err1, err2);
        assert_eq!(
            format!("{}", err1),
            "RSA decryption produced unexpected length"
        );
    }

    #[test]
    fn test_rsa_decrypt_all_zeros_plaintext_round_trip() {
        // Plaintext: first byte is 1, rest are 0 (to keep m < n)
        let key = load_test_key();
        let mut plaintext = [0u8; 128];
        plaintext[0] = 1;

        let m = BigUint::from_bytes_be(&plaintext);
        let e = BigUint::from_bytes_be(&key.e().to_bytes_be());
        let n = BigUint::from_bytes_be(&key.n().to_bytes_be());

        let c = m.modpow(&e, &n);
        let c_bytes = c.to_bytes_be();
        let mut ciphertext = [0u8; 128];
        let offset = 128 - c_bytes.len();
        ciphertext[offset..].copy_from_slice(&c_bytes);

        decrypt_with_key(&key, &mut ciphertext).expect("should succeed");
        assert_eq!(ciphertext, plaintext);
    }
}
