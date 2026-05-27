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

    let mut stream = stream.unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();

    // Either read some bytes (server sends a greeting) or time out / get EOF.
    // Both are acceptable — what's NOT acceptable is connection-refused above.
    let mut buf = [0u8; 64];
    let result = stream.read(&mut buf);
    match result {
        Ok(_) => {}
        Err(e)
            if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut =>
        {
            // read timed out — connection was accepted, server is just quiet
        }
        Err(e) => panic!("unexpected error reading from game port: {e}"),
    }
}

#[test]
fn game_login_records_server_response() {
    // Tibia 7.60 game login packet (see parse_login_packet in protocolgame.rs):
    //   [0x1E, 0x00]                    — 2-byte LE length prefix = 30
    //   [0x0a]                          — packet type: game login
    //   [0xF8, 0x02]                    — client version 760 (LE u16)
    //   [0x01,0x00,0x00,0x00]           — XTEA key[0] = 1
    //   [0x02,0x00,0x00,0x00]           — XTEA key[1] = 2
    //   [0x03,0x00,0x00,0x00]           — XTEA key[2] = 3
    //   [0x04,0x00,0x00,0x00]           — XTEA key[3] = 4
    //   [0x03,0x00, b'a',b'c',b'c']     — account "acc"
    //   [0x04,0x00, b'p',b'a',b's',b's'] — password "pass"
    #[rustfmt::skip]
    let packet: &[u8] = &[
        0x1E, 0x00,
        0x0a,
        0xF8, 0x02,
        0x01, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x00,
        0x04, 0x00, 0x00, 0x00,
        0x03, 0x00, b'a', b'c', b'c',
        0x04, 0x00, b'p', b'a', b's', b's',
    ];

    let addr = fixture().game_addr();
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .expect("game port refused connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();

    stream.write_all(packet).expect("failed to send login packet");
    let _ = stream.shutdown(std::net::Shutdown::Write);

    let mut response = Vec::new();
    let result = stream.read_to_end(&mut response);

    match result {
        Ok(0) => {
            // Server closed connection cleanly — game wiring not yet complete.
            eprintln!("[game_login_records_server_response] server closed with 0 bytes (expected at current scope)");
        }
        Ok(_) => {
            // Server responded — log bytes for future spec tightening.
            eprintln!(
                "[game_login_records_server_response] server responded with {} bytes: {:02X?}",
                response.len(),
                &response[..response.len().min(64)]
            );
        }
        Err(e)
            if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut =>
        {
            eprintln!("[game_login_records_server_response] read timed out (server accepted but sent nothing)");
        }
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
            // Server sent TCP RST — connection was accepted and reset without data.
            // Expected when game login handling is partially wired.
            eprintln!("[game_login_records_server_response] connection reset by server (partially wired, no response)");
        }
        Err(e) => {
            panic!("unexpected error reading game login response: {e}");
        }
    }
}
