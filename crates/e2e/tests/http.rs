#![cfg(feature = "e2e")]

mod common;

use std::net::TcpStream;
use std::sync::OnceLock;
use std::time::Duration;

use common::{tcp_roundtrip, ServerFixture};

static FIXTURE: OnceLock<ServerFixture> = OnceLock::new();

fn fixture() -> &'static ServerFixture {
    FIXTURE.get_or_init(ServerFixture::start)
}

#[test]
fn http_port_accepts_tcp_connection() {
    let addr = fixture().http_addr();
    let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5));
    assert!(
        stream.is_ok(),
        "HTTP port (8080) refused connection — port may not be bound: {:?}",
        stream.err()
    );
}

#[test]
fn http_port_returns_200_for_get_request() {
    let addr = fixture().http_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");
    assert!(!response.is_empty(), "HTTP port must return a response");
    assert!(
        response.starts_with(b"HTTP/1.0 200") || response.starts_with(b"HTTP/1.1 200"),
        "HTTP port must return 200 OK, got: {:?}",
        String::from_utf8_lossy(&response[..response.len().min(64)])
    );
}
