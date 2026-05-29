#![cfg(feature = "e2e")]

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::OnceLock;
use std::time::Duration;

use common::ServerFixture;

static FIXTURE: OnceLock<ServerFixture> = OnceLock::new();

fn fixture() -> &'static ServerFixture {
    FIXTURE.get_or_init(ServerFixture::start)
}

#[test]
fn game_port_accepts_tcp_connection() {
    let addr = fixture().game_addr();
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5));
    assert!(
        stream.is_ok(),
        "game port refused connection — port may not be bound: {:?}",
        stream.err()
    );
}

#[test]
fn game_port_sends_challenge_on_connect() {
    // TFS 13.10 sends a challenge packet (opcode 0x1F) when client connects.
    // Wire: [len_lo, len_hi, 0x1F, ts0, ts1, ts2, ts3, rand]
    let addr = fixture().game_addr();
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("game port must accept");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();

    let mut buf = [0u8; 16];
    let n = stream.read(&mut buf).unwrap_or(0);
    assert!(n >= 3, "challenge packet must be at least 3 bytes, got {n}");
    assert_eq!(
        buf[2], 0x1F,
        "first payload byte (after 2-byte length prefix) must be challenge opcode 0x1F"
    );
}

#[test]
fn game_login_bad_version_gets_disconnect() {
    // Send a minimal login packet with version 760 (0xF8 0x02).
    // Server must respond with a disconnect packet (opcode 0x14).
    //
    // Wire format for our game listener: read 2-byte length, then body.
    // body[0] = opcode (0x0a), body[1..] = payload parsed by parse_login_packet:
    //   version (u16), xtea×4 (u32 each), account_name (string), password (string)
    //
    // Body length = 1 (opcode) + 2 (version) + 16 (xtea) + 2 (acc len=0) + 2 (pwd len=0) = 23
    #[rustfmt::skip]
    let login_packet: &[u8] = &[
        0x17, 0x00,                      // 2-byte LE body length = 23
        0x0a,                            // opcode: game login
        0xF8, 0x02,                      // client version 760 (LE)
        0x01, 0x00, 0x00, 0x00,          // XTEA key[0]
        0x02, 0x00, 0x00, 0x00,          // XTEA key[1]
        0x03, 0x00, 0x00, 0x00,          // XTEA key[2]
        0x04, 0x00, 0x00, 0x00,          // XTEA key[3]
        0x00, 0x00,                      // account_name: empty pascal string (length=0)
        0x00, 0x00,                      // password: empty pascal string (length=0)
    ];

    let addr = fixture().game_addr();
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_secs(5)).expect("game port must accept");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();

    // Read and discard the challenge packet first
    let mut challenge_buf = [0u8; 16];
    let _ = stream.read(&mut challenge_buf);

    // Send the bad-version login packet
    stream.write_all(login_packet).expect("write login packet");
    let _ = stream.shutdown(std::net::Shutdown::Write);

    // Read the disconnect response
    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response);

    assert!(
        !response.is_empty(),
        "server must send a disconnect packet for version 760"
    );
    assert!(
        response.len() >= 3,
        "disconnect packet must be at least 3 bytes"
    );
    // After 2-byte length prefix, first byte is opcode 0x14 (disconnect)
    assert_eq!(
        response[2], 0x14,
        "disconnect opcode must be 0x14 (after 2-byte length prefix)"
    );
}
