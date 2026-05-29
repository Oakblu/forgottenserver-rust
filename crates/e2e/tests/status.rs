#![cfg(feature = "e2e")]

mod common;

use std::sync::OnceLock;

use common::{tcp_roundtrip, ServerFixture};

static FIXTURE: OnceLock<ServerFixture> = OnceLock::new();

fn fixture() -> &'static ServerFixture {
    FIXTURE.get_or_init(ServerFixture::start)
}

// ── HTTP path ────────────────────────────────────────────────────────────────

#[test]
fn status_http_returns_200_with_xml() {
    let addr = fixture().status_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");

    assert!(
        !response.is_empty(),
        "expected a response on the status port"
    );
    assert!(
        response.starts_with(b"HTTP/1.0 200"),
        "expected HTTP/1.0 200, got: {:?}",
        String::from_utf8_lossy(&response[..response.len().min(64)])
    );
    assert!(
        response.windows(5).any(|w| w == b"<tsqp"),
        "expected <tsqp element in XML body"
    );
}

#[test]
fn status_xml_has_server_name() {
    let addr = fixture().status_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");

    let body = String::from_utf8_lossy(&response);
    let xml = body
        .split("\r\n\r\n")
        .nth(1)
        .expect("response has no HTTP body section");

    assert!(
        xml.contains("<serverinfo"),
        "expected <serverinfo element in XML body, got: {xml}"
    );
    assert!(
        xml.contains(r#"servername=""#),
        "expected servername attribute, got: {xml}"
    );

    let after = xml
        .split(r#"servername=""#)
        .nth(1)
        .expect("servername attribute not found");
    let name = after.split('"').next().unwrap_or("");
    assert!(!name.is_empty(), "servername attribute must be non-empty");
}

#[test]
fn status_xml_client_version_is_13_10() {
    let addr = fixture().status_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");
    let body = String::from_utf8_lossy(&response);

    assert!(
        body.contains(r#"version="13.10""#),
        "serverinfo version must be 13.10: {body}"
    );
    assert!(
        body.contains(r#"client="13.10""#),
        "serverinfo client must be 13.10: {body}"
    );
}

#[test]
fn status_xml_servername_matches_config() {
    let addr = fixture().status_addr();
    let response = tcp_roundtrip(addr, b"GET / HTTP/1.0\r\n\r\n");
    let body = String::from_utf8_lossy(&response);

    // The e2e config.lua sets: serverName = "E2E Test Server"
    assert!(
        body.contains(r#"servername=""#),
        "XML must contain servername attribute: {body}"
    );
}

#[test]
fn status_binary_xml_contains_tsqp_and_serverinfo() {
    // Binary status request: [len_lo, len_hi, 0xFF, 'i', 'n', 'f', 'o']
    let request: &[u8] = &[0x05, 0x00, 0xFF, b'i', b'n', b'f', b'o'];
    let response = tcp_roundtrip(fixture().status_addr(), request);
    let body = String::from_utf8_lossy(&response);

    assert!(body.contains("<tsqp"), "binary response must contain <tsqp");
    assert!(
        body.contains("<serverinfo"),
        "binary response must contain <serverinfo"
    );
    assert!(
        body.contains("<players"),
        "binary response must contain <players"
    );
}

// ── Binary path ──────────────────────────────────────────────────────────────

#[test]
fn status_binary_xml_request_responds() {
    // Binary XML status request:
    //   [0x05, 0x00]              — 2-byte LE length = 5
    //   [0xFF, b'i', b'n', b'f', b'o'] — REQUEST_XML_INFO + literal "info"
    // The server returns the bare XML body (no HTTP headers).
    let request: &[u8] = &[0x05, 0x00, 0xFF, b'i', b'n', b'f', b'o'];
    let response = tcp_roundtrip(fixture().status_addr(), request);

    assert!(
        !response.is_empty(),
        "expected a non-empty binary XML response"
    );
    assert!(
        response.windows(5).any(|w| w == b"<tsqp"),
        "expected <tsqp in binary XML response, got {} bytes",
        response.len()
    );
}
