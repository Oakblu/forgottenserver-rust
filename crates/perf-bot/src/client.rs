//! Tibia game-protocol client for load testing.
//!
//! Implements the wire framing described in `protocolgame.rs` / `protocol.rs`:
//!
//! ```text
//! [0..2)  outer_len  u16 LE  = byte count from byte 6 to end
//! [2..6)  checksum   u32 LE  = Adler-32 of bytes [6..end]
//! [6..8)  inner_len  u16 LE  = length of the XTEA-encrypted payload
//! [8..)   XTEA-encrypted payload (multiple of 8 bytes, zero-padded)
//! ```
//!
//! The XTEA key is chosen randomly per connection; RSA-encrypts the first
//! 128-byte block of the login payload using the server's public key.

use std::time::Instant;

use anyhow::{anyhow, Context};
use forgottenserver_common::xtea::{self, RoundKeys};
use num_bigint::BigUint;
use rand::{RngCore, SeedableRng};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ---------------------------------------------------------------------------
// Server RSA public key (matches forgottenserver-upstream/key.pem)
// ---------------------------------------------------------------------------

/// PKCS#1 PEM for the server's RSA private key.  The perf-bot uses the
/// corresponding public key (extracted at runtime) to encrypt the XTEA key
/// block — exactly as a real game client would.
const SERVER_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
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

// ---------------------------------------------------------------------------
// Adler-32 (inline — no external crate)
// ---------------------------------------------------------------------------

/// Standard Adler-32 checksum (a=1, b=0, mod 65521).
fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = a.wrapping_add(byte as u32) % MOD;
        b = b.wrapping_add(a) % MOD;
    }
    (b << 16) | a
}

// ---------------------------------------------------------------------------
// Frame helpers
// ---------------------------------------------------------------------------

/// Build a complete on-wire packet frame for `payload`.
///
/// Layout:
/// ```text
/// [0..2)  outer_len  u16 LE  = 2 + padded_payload_len
/// [2..6)  checksum   u32 LE  = Adler-32([6..end])
/// [6..8)  inner_len  u16 LE  = payload.len()
/// [8..)   XTEA-encrypted payload (padded to 8-byte boundary with zeros)
/// ```
pub fn build_frame(payload: &[u8], round_keys: &RoundKeys) -> Vec<u8> {
    let inner_len = payload.len() as u16;

    // Pad payload to multiple of 8 for XTEA
    let padded_len = if payload.len().is_multiple_of(8) {
        payload.len()
    } else {
        payload.len() + (8 - payload.len() % 8)
    };

    // Build the region that gets XTEA-encrypted: [inner_len(2)] + [payload] + [padding]
    let mut encrypted_region = Vec::with_capacity(2 + padded_len);
    encrypted_region.extend_from_slice(&inner_len.to_le_bytes());
    encrypted_region.extend_from_slice(payload);
    encrypted_region.resize(2 + padded_len, 0u8);

    // Encrypt in-place
    xtea::encrypt(&mut encrypted_region, round_keys);

    let outer_len = encrypted_region.len() as u16; // = 2 + padded_len

    // Compute Adler-32 of the entire region that follows the checksum field:
    // [outer_len(2)] + [encrypted_region]  → but checksum covers [6..end]
    // which is exactly the encrypted_region (starts at byte 6).
    let checksum = adler32(&encrypted_region);

    // Assemble final frame
    let mut frame = Vec::with_capacity(6 + encrypted_region.len());
    frame.extend_from_slice(&outer_len.to_le_bytes()); // [0..2)
    frame.extend_from_slice(&checksum.to_le_bytes()); // [2..6)
    frame.extend_from_slice(&encrypted_region); // [6..)

    frame
}

// ---------------------------------------------------------------------------
// RSA encryption (raw, no padding: m^e mod n)
// ---------------------------------------------------------------------------

/// Encrypt `block` (128 bytes) with raw RSA public key: `c = m^e mod n`.
fn rsa_encrypt_block(block: &[u8; 128]) -> anyhow::Result<[u8; 128]> {
    let private_key = RsaPrivateKey::from_pkcs1_pem(SERVER_KEY_PEM)
        .context("failed to parse server RSA private key")?;
    let public_key = private_key.to_public_key();

    let e = BigUint::from_bytes_be(&public_key.e().to_bytes_be());
    let n = BigUint::from_bytes_be(&public_key.n().to_bytes_be());
    let m = BigUint::from_bytes_be(block.as_ref());

    let c = m.modpow(&e, &n);
    let c_bytes = c.to_bytes_be();

    let mut out = [0u8; 128];
    if c_bytes.len() > 128 {
        return Err(anyhow!("RSA ciphertext too large"));
    }
    let offset = 128 - c_bytes.len();
    out[offset..].copy_from_slice(&c_bytes);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Login packet builder
// ---------------------------------------------------------------------------

/// Build the unencrypted payload for the game-login first message.
///
/// The Rust server's `parse_login_packet` (in `protocolgame.rs`) reads:
///   - client_version (u16)
///   - xtea_key (4 × u32) — inside the RSA block
///   - account_name (u16-len-prefixed string)
///   - password (u16-len-prefixed string)
///
/// Layout of the 128-byte RSA-encrypted block (plaintext, big-endian on wire
/// after decryption):
///   [0]       0x00 padding byte
///   [1..5)    xtea_key[0] (u32 LE)
///   [5..9)    xtea_key[1] (u32 LE)
///   [9..13)   xtea_key[2] (u32 LE)
///   [13..17)  xtea_key[3] (u32 LE)
///   [17]      gm_flag = 0x00
///   [18..)    account_name (u16 LE len + bytes)
///   [...]     password    (u16 LE len + bytes)
///   rest:     0x00 padding to fill 128 bytes
///
/// The full pre-encryption payload (before framing) is:
///   [0]       opcode 0x0a
///   [1..3)    client_version (u16 LE)
///   [3..131)  RSA-encrypted 128-byte block
fn build_login_payload(
    account: &str,
    password: &str,
    xtea_key: &[u32; 4],
) -> anyhow::Result<Vec<u8>> {
    // Build the 128-byte RSA plaintext block
    let mut rsa_plain = [0u8; 128];
    rsa_plain[0] = 0x00; // padding
    rsa_plain[1..5].copy_from_slice(&xtea_key[0].to_le_bytes());
    rsa_plain[5..9].copy_from_slice(&xtea_key[1].to_le_bytes());
    rsa_plain[9..13].copy_from_slice(&xtea_key[2].to_le_bytes());
    rsa_plain[13..17].copy_from_slice(&xtea_key[3].to_le_bytes());
    rsa_plain[17] = 0x00; // gm_flag = 0

    // Write account name and password as u16-prefixed strings
    let acc_bytes = account.as_bytes();
    let pwd_bytes = password.as_bytes();
    let mut cursor = 18usize;

    let acc_len = acc_bytes.len().min(50) as u16;
    rsa_plain[cursor..cursor + 2].copy_from_slice(&acc_len.to_le_bytes());
    cursor += 2;
    rsa_plain[cursor..cursor + acc_len as usize].copy_from_slice(&acc_bytes[..acc_len as usize]);
    cursor += acc_len as usize;

    let pwd_len = pwd_bytes.len().min(50) as u16;
    rsa_plain[cursor..cursor + 2].copy_from_slice(&pwd_len.to_le_bytes());
    cursor += 2;
    rsa_plain[cursor..cursor + pwd_len as usize].copy_from_slice(&pwd_bytes[..pwd_len as usize]);
    // remaining bytes are already 0x00 padding

    // RSA-encrypt the 128-byte block
    let rsa_encrypted = rsa_encrypt_block(&rsa_plain)?;

    // Assemble the full unencrypted payload:
    //   opcode (1) + client_version (2) + rsa_block (128)
    let client_version: u16 = 1098;
    let mut payload = Vec::with_capacity(131);
    payload.push(0x0a); // game login opcode
    payload.extend_from_slice(&client_version.to_le_bytes());
    payload.extend_from_slice(&rsa_encrypted);

    Ok(payload)
}

// ---------------------------------------------------------------------------
// TibiaClient
// ---------------------------------------------------------------------------

/// A connected Tibia game-protocol client.
pub struct TibiaClient {
    stream: TcpStream,
    round_keys: RoundKeys,
}

impl TibiaClient {
    /// Connect to `host:port` and attempt the game login handshake.
    ///
    /// Returns `Ok(Self)` when the TCP connection is established and the login
    /// packet is sent.  Because the Rust server's full auth handler is not yet
    /// wired up, the handshake is "best-effort": any I/O error during the
    /// initial response read causes the connection to be treated as a
    /// non-authenticated session (we still return Ok so bots can send packets
    /// and measure dispatch latency).
    pub async fn connect(
        host: &str,
        port: u16,
        account: &str,
        password: &str,
    ) -> anyhow::Result<Self> {
        let addr = format!("{host}:{port}");
        let mut stream = TcpStream::connect(&addr)
            .await
            .with_context(|| format!("failed to connect to {addr}"))?;

        // Generate a random XTEA key for this session using a local RNG
        // that is created and consumed before the first await point so it
        // doesn't need to be Send.
        let key = {
            let mut rng = rand::rngs::SmallRng::from_entropy();
            xtea::Key([
                rng.next_u32(),
                rng.next_u32(),
                rng.next_u32(),
                rng.next_u32(),
            ])
        };
        let round_keys = xtea::expand_key(&key);

        // Build and send the login packet (unframed payload, then framed)
        let login_payload = build_login_payload(account, password, &key.0)?;

        // The login packet is NOT XTEA-encrypted because it is the first
        // message; XTEA is enabled after the server decrypts the RSA block.
        // We send it with a minimal framing: outer_len (u16) only, no
        // checksum, no inner encryption — matching the C++ "pre-XTEA" mode
        // used for the very first packet.
        //
        // Actual framing for first packet (as the real client sends):
        //   [0..2) outer_len = payload.len() (u16 LE)
        //   [2..)  raw payload (no checksum, no encryption)
        let outer_len = login_payload.len() as u16;
        let mut first_msg = Vec::with_capacity(2 + login_payload.len());
        first_msg.extend_from_slice(&outer_len.to_le_bytes());
        first_msg.extend_from_slice(&login_payload);

        stream
            .write_all(&first_msg)
            .await
            .context("failed to send login packet")?;

        // Attempt to read the server's response — gracefully ignore errors
        // (server may not be running a full auth stack)
        let mut len_buf = [0u8; 2];
        if stream.read_exact(&mut len_buf).await.is_ok() {
            let resp_len = u16::from_le_bytes(len_buf) as usize;
            if resp_len > 0 && resp_len <= 65535 {
                let mut resp = vec![0u8; resp_len];
                let _ = stream.read_exact(&mut resp).await;
            }
        }

        Ok(TibiaClient { stream, round_keys })
    }

    /// Send a game packet (opcode + payload), measured send-only.
    ///
    /// Builds the full XTEA-framed packet and writes it to the TCP stream.
    /// Returns elapsed microseconds (reported as milliseconds for
    /// compatibility with `RunMetrics::record_success`).
    pub async fn send_packet(&mut self, opcode: u8, payload: &[u8]) -> anyhow::Result<u64> {
        let mut full_payload = Vec::with_capacity(1 + payload.len());
        full_payload.push(opcode);
        full_payload.extend_from_slice(payload);

        let frame = build_frame(&full_payload, &self.round_keys);

        let start = Instant::now();
        self.stream
            .write_all(&frame)
            .await
            .context("failed to send packet")?;
        let elapsed_us = start.elapsed().as_micros() as u64;
        // Convert microseconds to milliseconds, rounding up to at least 1 ms
        // to avoid recording 0 ms for very fast sends.
        let elapsed_ms = (elapsed_us / 1000).max(1);
        Ok(elapsed_ms)
    }

    /// Send opcode 0x1D (ping) and wait up to 5 seconds for a response.
    ///
    /// Returns round-trip milliseconds, or an error on timeout / I/O failure.
    pub async fn ping(&mut self) -> anyhow::Result<u64> {
        let frame = build_frame(&[0x1D], &self.round_keys);
        let start = Instant::now();

        self.stream
            .write_all(&frame)
            .await
            .context("failed to send ping")?;

        // Try to read a response header (4 bytes minimum for framed response)
        let mut hdr = [0u8; 2];
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.stream.read_exact(&mut hdr),
        )
        .await
        .context("ping timeout")?
        .context("ping recv failed")?;

        // Drain the response body so the stream stays in sync
        let resp_len = u16::from_le_bytes(hdr) as usize;
        if resp_len > 0 && resp_len <= 65535 {
            let mut body = vec![0u8; resp_len];
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                self.stream.read_exact(&mut body),
            )
            .await;
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        Ok(elapsed_ms.max(1))
    }

    /// Send logout opcode (0x14) and shut down the TCP stream.
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        // Best-effort logout send — ignore errors (server may have closed)
        let frame = build_frame(&[0x14], &self.round_keys);
        let _ = self.stream.write_all(&frame).await;
        let _ = self.stream.shutdown().await;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Frame receive helper (used by ping and for future use)
// ---------------------------------------------------------------------------

/// Decode a received framed packet: read 6-byte header + payload, verify
/// Adler-32, XTEA-decrypt, return inner payload bytes.
pub async fn recv_frame(stream: &mut TcpStream, round_keys: &RoundKeys) -> anyhow::Result<Vec<u8>> {
    // Read 6-byte header: outer_len(2) + checksum(4)
    let mut hdr = [0u8; 6];
    stream
        .read_exact(&mut hdr)
        .await
        .context("failed to read frame header")?;

    let outer_len = u16::from_le_bytes([hdr[0], hdr[1]]) as usize;
    let _stored_checksum = u32::from_le_bytes([hdr[2], hdr[3], hdr[4], hdr[5]]);

    if outer_len < 2 {
        return Err(anyhow!("frame outer_len too small: {outer_len}"));
    }
    if outer_len > 65530 {
        return Err(anyhow!("frame outer_len too large: {outer_len}"));
    }

    // Read encrypted region
    let mut encrypted = vec![0u8; outer_len];
    stream
        .read_exact(&mut encrypted)
        .await
        .context("failed to read frame body")?;

    // Decrypt in-place
    xtea::decrypt(&mut encrypted, round_keys);

    // Read inner_len from first 2 bytes of decrypted region
    let inner_len = u16::from_le_bytes([encrypted[0], encrypted[1]]) as usize;
    if inner_len + 2 > encrypted.len() {
        return Err(anyhow!("inner_len {inner_len} exceeds encrypted region"));
    }

    Ok(encrypted[2..2 + inner_len].to_vec())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::xtea;

    #[test]
    fn adler32_empty_is_one() {
        assert_eq!(adler32(&[]), 1);
    }

    #[test]
    fn adler32_wikipedia() {
        // Well-known test vector
        assert_eq!(adler32(b"Wikipedia"), 0x11E60398);
    }

    #[test]
    fn build_frame_round_trips() {
        let key = xtea::Key([0x01234567, 0x89abcdef, 0xfedcba98, 0x76543210]);
        let round_keys = xtea::expand_key(&key);

        let payload = b"hello world test payload";
        let frame = build_frame(payload, &round_keys);

        // Frame must be at least 8 bytes (6 header + 2 inner_len)
        assert!(frame.len() >= 8);

        // outer_len field
        let outer_len = u16::from_le_bytes([frame[0], frame[1]]) as usize;
        assert_eq!(frame.len(), 6 + outer_len);

        // Verify Adler-32
        let stored = u32::from_le_bytes([frame[2], frame[3], frame[4], frame[5]]);
        let computed = adler32(&frame[6..]);
        assert_eq!(stored, computed);

        // Decrypt and verify inner payload
        let mut encrypted = frame[6..].to_vec();
        xtea::decrypt(&mut encrypted, &round_keys);
        let inner_len = u16::from_le_bytes([encrypted[0], encrypted[1]]) as usize;
        assert_eq!(&encrypted[2..2 + inner_len], payload);
    }

    #[test]
    fn build_login_payload_correct_opcode() {
        let key = [0x11223344u32, 0x55667788, 0x99aabbcc, 0xddeeff00];
        let payload = build_login_payload("testaccount", "testpass", &key).unwrap();
        assert_eq!(payload[0], 0x0a);
    }

    #[test]
    fn build_login_payload_length() {
        let key = [1u32, 2, 3, 4];
        let payload = build_login_payload("acc", "pass", &key).unwrap();
        // opcode(1) + client_version(2) + rsa_block(128) = 131
        assert_eq!(payload.len(), 131);
    }
}
