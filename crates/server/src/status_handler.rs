use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Instant,
};

use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
use forgottenserver_common::definitions::CLIENT_VERSION_STR;
use forgottenserver_network::protocolstatus::{
    parse_request, ProtocolStatus, ServerStatus, StatusRequest,
};

use crate::game_state::GameState;

pub struct StatusHandler {
    game_state: Arc<Mutex<GameState>>,
    config: Arc<ConfigManager>,
    start_time: Instant,
}

impl StatusHandler {
    pub fn new(game_state: Arc<Mutex<GameState>>, config: Arc<ConfigManager>) -> Self {
        Self {
            game_state,
            config,
            start_time: Instant::now(),
        }
    }

    /// Handle one status-port connection: read up to 512 bytes, dispatch to
    /// either the HTTP-XML or binary-protocol handler based on the first
    /// byte of the request, then write the response.
    pub fn handle_connection(&self, mut stream: TcpStream) {
        let mut request = [0u8; 512];
        let n = stream.read(&mut request).unwrap_or(0);
        let response = self.dispatch_request(&request[..n]);
        let _ = stream.write_all(&response);
    }

    /// Pure-function dispatcher — testable without sockets.
    ///
    /// Routing rule (matches `forgottenserver/src/protocolstatus.cpp`'s
    /// `parseFirstPacket` behaviour):
    ///
    /// - First byte is an ASCII uppercase letter (`'A'..='Z'`): treat as an
    ///   HTTP request line. Return `build_xml()` wrapped in `HTTP/1.0 200 OK`.
    /// - Empty buffer: same as HTTP (always-valid 200 OK fallback).
    /// - Otherwise: treat as a length-prefixed binary Tibia status frame.
    ///   Strip the 2-byte LE length prefix, hand the remaining payload to
    ///   [`parse_request`], and dispatch each `StatusRequest` variant:
    ///     * `Xml` → reply with the bare XML body (no HTTP headers — the
    ///       request wasn't HTTP).
    ///     * `Binary { requested, character_name }` → reply with
    ///       `ProtocolStatus::serialize_info(requested, &character_name)`.
    ///     * malformed → return empty `Vec` (matches C++'s
    ///       "fall-through-and-disconnect" behaviour).
    pub fn dispatch_request(&self, buf: &[u8]) -> Vec<u8> {
        if Self::looks_like_http(buf) {
            let body = self.build_xml();
            return format!(
                "HTTP/1.0 200 OK\r\nContent-Type: text/xml\r\n\
                 Content-Length: {}\r\n\r\n{}",
                body.len(),
                body
            )
            .into_bytes();
        }

        // Binary path: strip the 2-byte LE length prefix.
        if buf.len() < 2 {
            return Vec::new();
        }
        let payload = &buf[2..];
        match parse_request(payload) {
            Some(StatusRequest::Xml) => self.build_xml().into_bytes(),
            Some(StatusRequest::Binary {
                requested,
                character_name,
            }) => {
                let proto = ProtocolStatus::new(self.build_server_status());
                proto.serialize_info(requested, &character_name)
            }
            None => Vec::new(),
        }
    }

    /// True when the buffer's first byte looks like the start of an HTTP
    /// request line (any ASCII uppercase letter — `GET`, `POST`, `HEAD`,
    /// `PUT`, etc.). Empty buffers are treated as HTTP for safety: the HTTP
    /// branch always produces a valid 200 OK response.
    pub fn looks_like_http(buf: &[u8]) -> bool {
        match buf.first() {
            None => true,
            Some(b) => b.is_ascii_uppercase(),
        }
    }

    /// Bridge `StatusHandler`'s `GameState` + `ConfigManager` state into the
    /// `ServerStatus` snapshot that `crates/network/src/protocolstatus.rs`
    /// expects. Keeps the network crate read-only (no `from_state`
    /// constructor needed there).
    fn build_server_status(&self) -> ServerStatus {
        let state = self.game_state.lock().unwrap();
        let server_name = self.config.get_string(StringKey::ServerName).to_owned();
        let map_name = self.config.get_string(StringKey::MapName).to_owned();
        let owner_name = self.config.get_string(StringKey::OwnerName).to_owned();
        let owner_email = self.config.get_string(StringKey::OwnerEmail).to_owned();
        let location = self.config.get_string(StringKey::Location).to_owned();
        let url = self.config.get_string(StringKey::Url).to_owned();
        let server_ip = self.config.get_string(StringKey::Ip).to_owned();
        let motd = self.config.get_string(StringKey::Motd).to_owned();
        let map_author = self.config.get_string(StringKey::MapAuthor).to_owned();

        ServerStatus {
            server_name,
            map_name,
            players_online: state.online_player_count() as u32,
            players_max: self.config.get_integer(IntegerKey::MaxPlayers) as u32,
            players_peak: state.peak_players() as u32,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            exp_rate: self.config.get_integer(IntegerKey::RateExperience) as u32,
            motd,
            owner_name,
            owner_email,
            location,
            url,
            server_ip,
            server_port: self.config.get_integer(IntegerKey::GamePort) as u16,
            map_author,
            map_width: 0,
            map_height: 0,
            monsters_online: 0,
            npcs_online: 0,
            skill_rate: self.config.get_integer(IntegerKey::RateSkill) as u32,
            loot_rate: self.config.get_integer(IntegerKey::RateLoot) as u32,
            magic_rate: self.config.get_integer(IntegerKey::RateMagic) as u32,
            spawn_rate: self.config.get_integer(IntegerKey::RateSpawn) as u32,
            server_version: "Rust port".to_owned(),
            client_version: CLIENT_VERSION_STR.to_owned(),
        }
    }

    /// Build the Tibia-compatible XML status document.
    pub fn build_xml(&self) -> String {
        let state = self.game_state.lock().unwrap();
        let uptime = self.start_time.elapsed().as_secs();
        let ip = self.config.get_string(StringKey::Ip);
        let server_name = self.config.get_string(StringKey::ServerName);
        let game_port = self.config.get_integer(IntegerKey::GamePort);
        let owner_name = self.config.get_string(StringKey::OwnerName);
        let owner_email = self.config.get_string(StringKey::OwnerEmail);
        let max_players = self.config.get_integer(IntegerKey::MaxPlayers);
        let motd = self.config.get_string(StringKey::Motd);
        let online = state.online_player_count();
        let peak = state.peak_players();

        let version_str = CLIENT_VERSION_STR;
        format!(
            r#"<?xml version="1.0"?>
<tsqp version="1.0">
<serverinfo uptime="{uptime}" ip="{ip}" servername="{server_name}" port="{game_port}" version="{version_str}" client="{version_str}"/>
<owner name="{owner_name}" email="{owner_email}"/>
<players online="{online}" max="{max_players}" peak="{peak}"/>
<motd>{motd}</motd>
</tsqp>"#
        )
    }
}

// ---------------------------------------------------------------------------
// Tests (Phase 4)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::configmanager::{IntegerKey, StringKey};
    use std::{
        io::{BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        sync::{Arc, Mutex},
    };

    fn make_handler() -> StatusHandler {
        let gs = Arc::new(Mutex::new(GameState::new()));
        let cfg = Arc::new(ConfigManager::new());
        StatusHandler::new(gs, cfg)
    }

    fn make_handler_with(gs: Arc<Mutex<GameState>>, cfg: Arc<ConfigManager>) -> StatusHandler {
        StatusHandler::new(gs, cfg)
    }

    fn spawn_status_server(handler: Arc<StatusHandler>) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                handler.handle_connection(stream);
            }
        });
        port
    }

    fn http_get(port: u16) -> String {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        stream.write_all(b"GET /status HTTP/1.0\r\n\r\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut out = String::new();
        BufReader::new(stream).read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn status_xml_contains_tsqp_root() {
        let h = make_handler();
        let xml = h.build_xml();
        assert!(xml.contains("<tsqp"), "expected <tsqp in: {xml}");
    }

    #[test]
    fn status_xml_contains_server_name_and_player_count() {
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Alice");
        let mut cfg = ConfigManager::new();
        cfg.set_string(StringKey::ServerName, "PokéTibia");
        cfg.set_integer(IntegerKey::MaxPlayers, 200);
        let h = make_handler_with(gs, Arc::new(cfg));
        let xml = h.build_xml();
        assert!(xml.contains("PokéTibia"), "server name missing: {xml}");
        assert!(xml.contains("online=\"1\""), "player count missing: {xml}");
    }

    #[test]
    fn status_get_returns_200_with_xml_content_type() {
        let gs = Arc::new(Mutex::new(GameState::new()));
        let cfg = Arc::new(ConfigManager::new());
        let handler = Arc::new(make_handler_with(gs, cfg));
        let port = spawn_status_server(handler);
        let resp = http_get(port);
        assert!(resp.starts_with("HTTP/1.0 200 OK"), "got: {resp}");
        assert!(resp.contains("Content-Type: text/xml"), "got: {resp}");
    }

    #[test]
    fn status_xml_uptime_increases_over_time() {
        let h = make_handler();
        let xml1 = h.build_xml();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let xml2 = h.build_xml();
        // Both should have uptime= attribute; second should be >= first
        fn parse_uptime(xml: &str) -> u64 {
            xml.split("uptime=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        }
        let u1 = parse_uptime(&xml1);
        let u2 = parse_uptime(&xml2);
        assert!(u2 >= u1, "uptime should not decrease: {u1} -> {u2}");
    }

    // -----------------------------------------------------------------------
    // dispatch_request / looks_like_http — protocolstatus binary/HTTP demux
    // (forgottenserver-rust-protocolstatus-demux change)
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_request_routes_http_get_to_xml_body() {
        let h = make_handler();
        let response = h.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
        assert!(
            response.starts_with(b"HTTP/1.0 200 OK"),
            "HTTP GET must produce an HTTP 200 response, got: {:?}",
            String::from_utf8_lossy(&response[..response.len().min(80)])
        );
        assert!(
            response.windows(5).any(|w| w == b"<tsqp"),
            "HTTP response must contain <tsqp XML root"
        );
    }

    #[test]
    fn dispatch_request_routes_empty_buffer_to_http_for_safety() {
        let h = make_handler();
        let response = h.dispatch_request(b"");
        assert!(
            response.starts_with(b"HTTP/1.0 200 OK"),
            "empty buffer must fall through to the HTTP path"
        );
    }

    #[test]
    fn dispatch_request_routes_binary_xml_info_to_xml_body() {
        // Binary frame asking for the XML response: length=5, then `\xff
        // "info"` (5 bytes payload). This is the "give me the XML body via
        // the binary protocol" request — C++ returns the bare XML body, no
        // HTTP wrapping.
        let h = make_handler();
        let frame = b"\x05\x00\xff\x69\x6E\x66\x6F";
        let response = h.dispatch_request(frame);
        assert!(
            !response.is_empty(),
            "valid XML-info binary request must produce a non-empty response"
        );
        assert!(
            !response.starts_with(b"HTTP/"),
            "binary request must NOT produce an HTTP-wrapped response"
        );
        assert!(
            response.windows(5).any(|w| w == b"<tsqp"),
            "binary XML-info response must contain the <tsqp XML root"
        );
    }

    #[test]
    fn dispatch_request_routes_binary_serialize_info_to_binary_response() {
        // Binary frame asking for the binary status response: length=3, then
        // `\x01 <flags-le-u16>` for basic info (0x0001).
        let h = make_handler();
        let frame = b"\x03\x00\x01\x01\x00";
        let response = h.dispatch_request(frame);
        assert!(
            !response.is_empty(),
            "valid binary-info request must produce a non-empty response"
        );
        assert!(
            !response.starts_with(b"HTTP/"),
            "binary request must NOT produce an HTTP-wrapped response"
        );
        assert!(
            !response.starts_with(b"<?xml"),
            "binary-info response must NOT be the XML form"
        );
    }

    #[test]
    fn dispatch_request_routes_malformed_binary_to_empty_close() {
        // Single-byte buffer that's neither ASCII upper nor a valid binary
        // length prefix — matches C++'s "disconnect on garbage" behaviour.
        let h = make_handler();
        let response = h.dispatch_request(b"\x00");
        assert_eq!(
            response, b"",
            "malformed binary input must close cleanly with no response"
        );
    }

    #[test]
    fn looks_like_http_recognises_get_post_head() {
        assert!(StatusHandler::looks_like_http(b"GET / HTTP/1.0"));
        assert!(StatusHandler::looks_like_http(b"POST / HTTP/1.0"));
        assert!(StatusHandler::looks_like_http(b"HEAD / HTTP/1.0"));
        assert!(StatusHandler::looks_like_http(b"PUT / HTTP/1.0"));
    }

    #[test]
    fn looks_like_http_rejects_binary_first_byte() {
        assert!(!StatusHandler::looks_like_http(b"\x06\x00\xff\xff\x01\x1f"));
        assert!(!StatusHandler::looks_like_http(b"\xff"));
        assert!(!StatusHandler::looks_like_http(b"\x01\x00"));
        // Lowercase letters are NOT HTTP request verbs — fall through to
        // binary (which will reject as malformed and close cleanly).
        assert!(!StatusHandler::looks_like_http(b"get / HTTP/1.0"));
    }

    #[test]
    fn build_xml_version_field_matches_client_version_str() {
        use forgottenserver_common::configmanager::ConfigManager;

        let gs = Arc::new(Mutex::new(GameState::new()));
        let config = Arc::new(ConfigManager::new());
        let handler = StatusHandler::new(gs, config);
        let xml = handler.build_xml();

        // Must NOT contain the wrong hardcoded version
        assert!(
            !xml.contains(r#"version="860""#),
            "XML must not contain hardcoded version 860"
        );
        assert!(
            !xml.contains(r#"client="860""#),
            "XML must not contain hardcoded client 860"
        );

        // Must contain the correct version string from definitions
        assert!(
            xml.contains(r#"version="13.10""#),
            "XML version must be 13.10: {xml}"
        );
        assert!(
            xml.contains(r#"client="13.10""#),
            "XML client must be 13.10: {xml}"
        );
    }

    #[test]
    fn dispatch_request_round_trip_matches_handle_connection_for_http() {
        // The dispatch_request return-value contract: for HTTP input it
        // produces the same byte stream that handle_connection would write.
        let h = make_handler();
        let dispatch = h.dispatch_request(b"GET / HTTP/1.0\r\n\r\n");
        // The dispatch contract is "produces what handle_connection writes
        // back". We can't easily intercept handle_connection's writes
        // without a TcpStream, so we verify the structural contract: the
        // dispatch output is a valid HTTP/1.0 200 response with a non-zero
        // Content-Length and an XML body that starts with `<?xml`.
        let text = String::from_utf8_lossy(&dispatch);
        assert!(text.starts_with("HTTP/1.0 200 OK"));
        assert!(text.contains("Content-Length: "));
        let body_start = text
            .find("\r\n\r\n")
            .expect("HTTP response must have header/body separator");
        let body = &text[body_start + 4..];
        assert!(
            body.starts_with("<?xml"),
            "body must start with XML preamble, got: {body}"
        );
    }

    #[test]
    fn build_server_status_client_version_matches_definitions() {
        use forgottenserver_common::configmanager::ConfigManager;

        let gs = Arc::new(Mutex::new(GameState::new()));
        let config = Arc::new(ConfigManager::new());
        let handler = StatusHandler::new(gs, config);
        let status = handler.build_server_status();
        assert_eq!(
            status.client_version, "13.10",
            "client_version must be 13.10, got: {}",
            status.client_version
        );
    }
}
