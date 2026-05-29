//! HTTP connection session — one per accepted TCP connection on the HTTP port.
//!
//! Mirrors `forgottenserver/src/http/session.cpp`: read one HTTP request,
//! dispatch to the appropriate handler by the JSON `"type"` field, and write
//! one HTTP/1.1 response.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use forgottenserver_database::database::Database;
use forgottenserver_items::vocation::Vocations;

use crate::http_cacheinfo::handle_cacheinfo_db;
use crate::http_login::{handle_login_db, LoginConfig, LoginRequest};

const MAX_HEADER_BYTES: usize = 8_192;
const MAX_BODY_BYTES: usize = 4_096;
const READ_TIMEOUT_SECS: u64 = 30;

/// Shared state for the HTTP listener; one instance per listener, cloned via
/// `Arc` into each connection thread.
pub struct HttpConnectionSession {
    pub db: Arc<Mutex<Box<dyn Database + Send>>>,
    pub config: Arc<LoginConfig>,
    pub vocations: Arc<Vocations>,
}

impl HttpConnectionSession {
    pub fn new(
        db: Arc<Mutex<Box<dyn Database + Send>>>,
        config: Arc<LoginConfig>,
        vocations: Arc<Vocations>,
    ) -> Self {
        Self {
            db,
            config,
            vocations,
        }
    }

    /// Handle one accepted TCP connection: read request → dispatch → write
    /// response.  Matches the public interface of the C++ `Session::run()`.
    pub fn handle(&self, mut stream: TcpStream) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS)));

        let peer_ip = stream
            .peer_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_default();

        let (method, body) = match read_request(&mut stream) {
            Some(r) => r,
            None => return,
        };

        if method.eq_ignore_ascii_case("GET") {
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\n\
                  Content-Type: application/json\r\n\
                  Content-Length: 2\r\n\
                  Connection: close\r\n\
                  \r\n\
                  {}",
            );
            return;
        }

        let (status, resp_body) = self.dispatch(&body, &peer_ip);
        let response = format!(
            "HTTP/1.1 {status} {reason}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {len}\r\n\
             Connection: close\r\n\
             \r\n\
             {resp_body}",
            status = status,
            reason = reason_phrase(status),
            len = resp_body.len(),
            resp_body = resp_body,
        );
        let _ = stream.write_all(response.as_bytes());
    }

    fn dispatch(&self, body: &str, ip: &str) -> (u16, String) {
        let json: serde_json::Value = match serde_json::from_str(body) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "[HTTP] dispatch from {ip}: parse error ({:?})",
                    e.classify()
                );
                return (
                    200,
                    r#"{"errorCode":2,"errorMessage":"Invalid request body."}"#.to_string(),
                );
            }
        };

        let type_val = match json.get("type").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => {
                return (
                    200,
                    r#"{"errorCode":2,"errorMessage":"Invalid request type."}"#.to_string(),
                )
            }
        };

        eprintln!("[HTTP] dispatch from {ip}: type={type_val}");

        match type_val {
            "login" => self.dispatch_login(&json, ip),
            "cacheinfo" => {
                let guard = self.db.lock().unwrap();
                handle_cacheinfo_db(&**guard)
            }
            _ => (
                200,
                r#"{"errorCode":2,"errorMessage":"Invalid request type."}"#.to_string(),
            ),
        }
    }

    fn dispatch_login(&self, json: &serde_json::Value, ip: &str) -> (u16, String) {
        let req: LoginRequest = match serde_json::from_value(json.clone()) {
            Ok(r) => r,
            Err(_) => {
                return (
                    200,
                    r#"{"errorCode":3,"errorMessage":"Tibia account email address or Tibia password is not correct."}"#.to_string(),
                )
            }
        };
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut guard = self.db.lock().unwrap();
        handle_login_db(
            &mut **guard,
            &req,
            ip,
            &self.config,
            &self.vocations,
            now_secs,
        )
    }
}

/// Read one complete HTTP request from `stream`.
///
/// Returns `(method, body_string)` on success, `None` if headers exceed
/// `MAX_HEADER_BYTES`, a parse error occurs, or the connection drops.
fn read_request(stream: &mut TcpStream) -> Option<(String, String)> {
    let mut buf = vec![0u8; MAX_HEADER_BYTES];
    let mut filled = 0;

    loop {
        if filled == buf.len() {
            return None; // Headers larger than MAX_HEADER_BYTES
        }

        let n = match stream.read(&mut buf[filled..]) {
            Ok(0) | Err(_) => return None,
            Ok(n) => n,
        };
        filled += n;

        let mut headers_storage = [httparse::EMPTY_HEADER; 32];
        let mut req = httparse::Request::new(&mut headers_storage);

        match req.parse(&buf[..filled]) {
            Ok(httparse::Status::Complete(header_end)) => {
                let method = req.method.unwrap_or("GET").to_string();

                let content_length: usize = headers_storage
                    .iter()
                    .take_while(|h| !h.name.is_empty())
                    .find(|h| h.name.eq_ignore_ascii_case("content-length"))
                    .and_then(|h| std::str::from_utf8(h.value).ok())
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(0)
                    .min(MAX_BODY_BYTES);

                let already = (filled - header_end).min(content_length);
                let mut body = vec![0u8; content_length];
                body[..already].copy_from_slice(&buf[header_end..header_end + already]);

                if already < content_length {
                    stream.read_exact(&mut body[already..]).ok()?;
                }

                return Some((method, String::from_utf8_lossy(&body).into_owned()));
            }
            Ok(httparse::Status::Partial) => {
                // Continue reading
            }
            Err(_) => return None,
        }
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_database::database::InMemoryDb;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream as Client};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn empty_db() -> Arc<Mutex<Box<dyn Database + Send>>> {
        Arc::new(Mutex::new(Box::new(InMemoryDb::new())))
    }

    fn empty_vocations() -> Arc<Vocations> {
        Arc::new(Vocations::load_from_xml("<vocations/>").unwrap())
    }

    fn test_config() -> Arc<LoginConfig> {
        Arc::new(LoginConfig {
            server_name: "TestServer".to_string(),
            ip: "127.0.0.1".to_string(),
            game_port: 7172,
            location: "EU".to_string(),
            pvp_type: 0,
        })
    }

    fn make_session(db: Arc<Mutex<Box<dyn Database + Send>>>) -> Arc<HttpConnectionSession> {
        Arc::new(HttpConnectionSession::new(
            db,
            test_config(),
            empty_vocations(),
        ))
    }

    /// Spin up a session on a free port; connect, write `request`, read full response.
    fn roundtrip(session: Arc<HttpConnectionSession>, request: &[u8]) -> Vec<u8> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let sess = session.clone();
        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                sess.handle(stream);
            }
        });

        let mut client = Client::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        client.write_all(request).unwrap();
        let mut response = Vec::new();
        let _ = client.read_to_end(&mut response);
        response
    }

    // ── 2.5  GET / returns 200 OK ────────────────────────────────────────────

    #[test]
    fn get_slash_returns_200_ok() {
        let resp = roundtrip(make_session(empty_db()), b"GET / HTTP/1.0\r\n\r\n");
        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.starts_with("HTTP/1.1 200") || text.starts_with("HTTP/1.0 200"),
            "expected 200, got: {text:?}"
        );
    }

    #[test]
    fn get_slash_response_has_json_content_type() {
        let resp = roundtrip(make_session(empty_db()), b"GET / HTTP/1.0\r\n\r\n");
        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.contains("Content-Type: application/json"),
            "missing Content-Type: application/json in: {text:?}"
        );
    }

    // ── 2.3  Oversized headers → connection closed without response ──────────

    #[test]
    fn oversized_headers_closes_connection_without_response() {
        let session = make_session(empty_db());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let sess = session.clone();
        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                sess.handle(stream);
            }
        });

        let mut client = Client::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();

        // Send MAX_HEADER_BYTES + 1 bytes without completing the headers.
        let big = vec![b'X'; MAX_HEADER_BYTES + 1];
        let _ = client.write_all(&big);

        let mut response = Vec::new();
        let _ = client.read_to_end(&mut response);
        assert!(
            response.is_empty(),
            "expected empty response for oversized headers, got {} bytes",
            response.len()
        );
    }

    // ── 2.7  Unknown type → error code 2 ────────────────────────────────────

    #[test]
    fn unknown_type_returns_error_code_2() {
        let resp = roundtrip(
            make_session(empty_db()),
            b"POST / HTTP/1.0\r\nContent-Length: 17\r\n\r\n{\"type\":\"unknown\"}",
        );
        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.contains("\"errorCode\":2"),
            "expected errorCode 2, got: {text:?}"
        );
    }

    // ── 2.6  Non-JSON body → error code 2 ───────────────────────────────────

    #[test]
    fn non_json_body_returns_error_code_2() {
        let body = b"not json at all";
        let req = format!("POST / HTTP/1.0\r\nContent-Length: {}\r\n\r\n", body.len());
        let mut full = req.into_bytes();
        full.extend_from_slice(body);
        let resp = roundtrip(make_session(empty_db()), &full);
        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.contains("\"errorCode\":2"),
            "expected errorCode 2 for non-JSON body, got: {text:?}"
        );
    }

    // ── 2.8  Content-Length header matches actual body ───────────────────────

    #[test]
    fn content_length_matches_body_length() {
        let resp = roundtrip(
            make_session(empty_db()),
            b"POST / HTTP/1.0\r\nContent-Length: 17\r\n\r\n{\"type\":\"unknown\"}",
        );
        let text = String::from_utf8_lossy(&resp);

        // Extract Content-Length value from response headers.
        let cl_val: usize = text
            .lines()
            .find(|l| l.to_lowercase().starts_with("content-length:"))
            .and_then(|l| l.split_once(':').map(|(_, v)| v))
            .and_then(|v| v.trim().parse().ok())
            .expect("Content-Length header must be present");

        // Split on \r\n\r\n to find body.
        let body_start = text
            .find("\r\n\r\n")
            .expect("must have header/body separator");
        let body = &text[body_start + 4..];
        assert_eq!(
            cl_val,
            body.len(),
            "Content-Length {cl_val} != actual body len {}",
            body.len()
        );
    }

    // ── 2.7  Missing type field → error code 2 ──────────────────────────────

    #[test]
    fn missing_type_field_returns_error_code_2() {
        let body = b"{\"foo\":\"bar\"}";
        let req = format!("POST / HTTP/1.0\r\nContent-Length: {}\r\n\r\n", body.len());
        let mut full = req.into_bytes();
        full.extend_from_slice(body);
        let resp = roundtrip(make_session(empty_db()), &full);
        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.contains("\"errorCode\":2"),
            "expected errorCode 2 for missing type, got: {text:?}"
        );
    }

    // ── 2.7  cacheinfo type → playersonline key ──────────────────────────────

    #[test]
    fn cacheinfo_type_returns_playersonline_key() {
        let body = b"{\"type\":\"cacheinfo\"}";
        let req = format!("POST / HTTP/1.0\r\nContent-Length: {}\r\n\r\n", body.len());
        let mut full = req.into_bytes();
        full.extend_from_slice(body);
        let resp = roundtrip(make_session(empty_db()), &full);
        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.contains("\"playersonline\""),
            "expected playersonline key, got: {text:?}"
        );
    }

    // ── 2.8  Response always has Content-Type: application/json ─────────────

    #[test]
    fn post_response_has_json_content_type() {
        let resp = roundtrip(
            make_session(empty_db()),
            b"POST / HTTP/1.0\r\nContent-Length: 17\r\n\r\n{\"type\":\"unknown\"}",
        );
        let text = String::from_utf8_lossy(&resp);
        assert!(
            text.contains("Content-Type: application/json"),
            "missing Content-Type header, got: {text:?}"
        );
    }
}
