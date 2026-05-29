//! In-process status handler integration tests.
//! No TCP sockets or Docker required.

use std::sync::{Arc, Mutex};

use forgottenserver_common::configmanager::{ConfigManager, StringKey};
use forgottenserver_server::game_state::GameState;
use forgottenserver_server::status_handler::StatusHandler;

fn make_handler(server_name: &str) -> StatusHandler {
    let gs = Arc::new(Mutex::new(GameState::new()));
    let mut config = ConfigManager::new();
    config.set_string(StringKey::ServerName, server_name);
    StatusHandler::new(gs, Arc::new(config))
}

#[test]
fn status_http_get_returns_200_and_xml() {
    let handler = make_handler("PortTest");
    let response = handler.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.starts_with("HTTP/1.0 200"),
        "must return HTTP 200: {text}"
    );
    assert!(
        text.contains("Content-Type: text/xml"),
        "must include XML content type: {text}"
    );
    assert!(
        text.contains("<tsqp"),
        "body must contain <tsqp element: {text}"
    );
}

#[test]
fn status_http_get_xml_contains_configured_server_name() {
    let handler = make_handler("PortTest");
    let response = handler.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.contains(r#"servername="PortTest""#),
        "XML must contain configured server name: {text}"
    );
}

#[test]
fn status_xml_contains_correct_client_version() {
    let handler = make_handler("PortTest");
    let response = handler.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.contains(r#"version="13.10""#),
        "XML version must be 13.10: {text}"
    );
    assert!(
        text.contains(r#"client="13.10""#),
        "XML client must be 13.10: {text}"
    );
}

#[test]
fn status_binary_xml_request_returns_raw_xml() {
    let handler = make_handler("PortTest");
    // Binary request: [len_lo, len_hi, 0xFF, 'i', 'n', 'f', 'o']
    // Length prefix = 5 (little-endian u16): covers the 5-byte payload `\xff info`
    let request: &[u8] = &[0x05, 0x00, 0xFF, b'i', b'n', b'f', b'o'];
    let response = handler.dispatch_request(request);
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.contains("<tsqp"),
        "binary XML request must return <tsqp XML: {text}"
    );
    assert!(
        !text.starts_with("HTTP"),
        "binary response must not have HTTP headers: {text}"
    );
}

#[test]
fn status_empty_buffer_falls_back_to_http_200() {
    let handler = make_handler("PortTest");
    let response = handler.dispatch_request(b"");
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.starts_with("HTTP/1.0 200"),
        "empty request must return HTTP 200 fallback: {text}"
    );
}

#[test]
fn status_http_post_also_returns_200() {
    let handler = make_handler("PortTest");
    let response = handler.dispatch_request(b"POST / HTTP/1.0\r\n\r\n");
    let text = String::from_utf8_lossy(&response);
    assert!(
        text.starts_with("HTTP/1.0 200"),
        "POST must also return HTTP 200: {text}"
    );
}
