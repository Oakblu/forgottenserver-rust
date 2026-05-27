//! Server status protocol.
//!
//! Migrated from forgottenserver protocolstatus.h / protocolstatus.cpp.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use forgottenserver_common::networkmessage::NetworkMessage;

// ---------------------------------------------------------------------------
// Protocol constants (mirrors ProtocolStatus enum fields in C++ header)
// ---------------------------------------------------------------------------

/// The protocol identifier byte sent by the client to select the status protocol.
pub const PROTOCOL_IDENTIFIER: u8 = 0xFF;

/// Whether the server sends the first message on a status connection.
pub const SERVER_SENDS_FIRST: bool = false;

/// Whether the status protocol uses a checksum.
pub const USE_CHECKSUM: bool = false;

/// Human-readable protocol name. Mirrors `ProtocolStatus::protocol_name()` in C++.
pub const PROTOCOL_NAME: &str = "status protocol";

/// First-byte selector for the legacy XML info request ("\xFFinfo" payload).
pub const REQUEST_XML_INFO: u8 = 0xFF;

/// First-byte selector for the binary server-info request.
pub const REQUEST_BINARY_INFO: u8 = 0x01;

// ---------------------------------------------------------------------------
// RequestedInfo bitflags (mirrors RequestedInfo_t in protocolstatus.cpp)
// ---------------------------------------------------------------------------

/// Bit flags controlling which server information fields are included in an
/// info response.  Mirrors `RequestedInfo_t` from protocolstatus.cpp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RequestedInfo(pub u16);

impl RequestedInfo {
    /// Basic server info: server name, IP, port.
    pub const BASIC_SERVER_INFO: u16 = 1 << 0;
    /// Owner info: owner name and email.
    pub const OWNER_SERVER_INFO: u16 = 1 << 1;
    /// Misc info: MOTD, location, URL, uptime.
    pub const MISC_SERVER_INFO: u16 = 1 << 2;
    /// Players info: online, max, peak.
    pub const PLAYERS_INFO: u16 = 1 << 3;
    /// Map info: name, author, dimensions.
    pub const MAP_INFO: u16 = 1 << 4;
    /// Extended players info: list of online players with levels.
    pub const EXT_PLAYERS_INFO: u16 = 1 << 5;
    /// Player status info: whether a named player is online.
    pub const PLAYER_STATUS_INFO: u16 = 1 << 6;
    /// Server software info: server name, version, client version.
    pub const SERVER_SOFTWARE_INFO: u16 = 1 << 7;

    pub fn new(flags: u16) -> Self {
        Self(flags)
    }

    pub fn has(&self, flag: u16) -> bool {
        self.0 & flag != 0
    }
}

// ---------------------------------------------------------------------------
// ServerStatus
// ---------------------------------------------------------------------------

/// Snapshot of the game server status sent in response to status requests.
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub server_name: String,
    pub map_name: String,
    pub players_online: u32,
    pub players_max: u32,
    /// All-time peak player count.
    pub players_peak: u32,
    pub uptime_seconds: u64,
    pub exp_rate: u32,
    /// Message of the day.
    pub motd: String,
    /// Server owner name.
    pub owner_name: String,
    /// Server owner email.
    pub owner_email: String,
    /// Server location description.
    pub location: String,
    /// Server website URL.
    pub url: String,
    /// Server IP/address string.
    pub server_ip: String,
    /// Server game port.
    pub server_port: u16,
    /// Map author name.
    pub map_author: String,
    /// Map width in tiles.
    pub map_width: u32,
    /// Map height in tiles.
    pub map_height: u32,
    /// Number of monsters currently online.
    pub monsters_online: u32,
    /// Number of NPCs currently online.
    pub npcs_online: u32,
    /// Skill rate multiplier.
    pub skill_rate: u32,
    /// Loot rate multiplier.
    pub loot_rate: u32,
    /// Magic rate multiplier.
    pub magic_rate: u32,
    /// Spawn rate multiplier.
    pub spawn_rate: u32,
    /// Server software version string.
    pub server_version: String,
    /// Client version string.
    pub client_version: String,
}

// ---------------------------------------------------------------------------
// ProtocolStatus
// ---------------------------------------------------------------------------

/// Handles inbound status requests and produces serialized responses.
pub struct ProtocolStatus {
    status: ServerStatus,
    /// Rate limiter: tracks the last-request timestamp per IPv4 address.
    last_request: HashMap<u32, Instant>,
}

impl ProtocolStatus {
    /// Creates a new `ProtocolStatus` holding the given server status.
    pub fn new(status: ServerStatus) -> Self {
        Self {
            status,
            last_request: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Serialization
    // -----------------------------------------------------------------------

    /// Serializes the server status into a raw byte packet using
    /// `NetworkMessage` write primitives.
    ///
    /// Field order (all little-endian):
    /// 1. `server_name`    — length-prefixed string
    /// 2. `map_name`       — length-prefixed string
    /// 3. `players_online` — u32
    /// 4. `players_max`    — u32
    /// 5. `uptime_seconds` — u64
    /// 6. `exp_rate`       — u32
    ///
    /// Returns only the payload bytes (excludes the 8-byte NetworkMessage header).
    pub fn serialize_status(&self) -> Vec<u8> {
        let mut msg = NetworkMessage::new();
        msg.add_string(&self.status.server_name);
        msg.add_string(&self.status.map_name);
        msg.add_u32(self.status.players_online);
        msg.add_u32(self.status.players_max);
        msg.add_u64(self.status.uptime_seconds);
        msg.add_u32(self.status.exp_rate);

        // Extract only the payload bytes (after the 8-byte header)
        let start = forgottenserver_common::networkmessage::INITIAL_BUFFER_POSITION as usize;
        let end = start + msg.get_message_length() as usize;
        msg.get_buffer()[start..end].to_vec()
    }

    /// Returns an XML string representation of the server status.
    ///
    /// Matches the `sendStatusString()` XML format from protocolstatus.cpp,
    /// using a `<tsqp>` root with child elements:
    /// `<serverinfo>`, `<owner>`, `<players>`, `<monsters>`, `<npcs>`,
    /// `<rates>`, `<map>`, `<motd>`.
    pub fn get_xml_status(&self) -> String {
        let s = &self.status;
        format!(
            concat!(
                r#"<?xml version="1.0"?>"#,
                r#"<tsqp version="1.0">"#,
                r#"<serverinfo uptime="{uptime}" ip="{ip}" servername="{name}" port="{port}" location="{location}" url="{url}" server="{server_ver}" client="{client_ver}"/>"#,
                r#"<owner name="{owner_name}" email="{owner_email}"/>"#,
                r#"<players online="{online}" max="{max}" peak="{peak}"/>"#,
                r#"<monsters total="{monsters}"/>"#,
                r#"<npcs total="{npcs}"/>"#,
                r#"<rates experience="{exp}" skill="{skill}" loot="{loot}" magic="{magic}" spawn="{spawn}"/>"#,
                r#"<map name="{map_name}" author="{map_author}" width="{map_w}" height="{map_h}"/>"#,
                r#"<motd>{motd}</motd>"#,
                r#"</tsqp>"#,
            ),
            uptime = s.uptime_seconds,
            ip = xml_escape(&s.server_ip),
            name = xml_escape(&s.server_name),
            port = s.server_port,
            location = xml_escape(&s.location),
            url = xml_escape(&s.url),
            server_ver = xml_escape(&s.server_version),
            client_ver = xml_escape(&s.client_version),
            owner_name = xml_escape(&s.owner_name),
            owner_email = xml_escape(&s.owner_email),
            online = s.players_online,
            max = s.players_max,
            peak = s.players_peak,
            monsters = s.monsters_online,
            npcs = s.npcs_online,
            exp = s.exp_rate,
            skill = s.skill_rate,
            loot = s.loot_rate,
            magic = s.magic_rate,
            spawn = s.spawn_rate,
            map_name = xml_escape(&s.map_name),
            map_author = xml_escape(&s.map_author),
            map_w = s.map_width,
            map_h = s.map_height,
            motd = xml_escape(&s.motd),
        )
    }

    // -----------------------------------------------------------------------
    // Info serialization (binary protocol, request 0x01)
    // -----------------------------------------------------------------------

    /// Serializes the requested info fields into a byte vector.
    ///
    /// Mirrors `ProtocolStatus::sendInfo(uint16_t requestedInfo, ...)` from
    /// protocolstatus.cpp.  Each included section is prefixed by a tag byte
    /// followed by the section's fields.
    ///
    /// Tag bytes:
    /// - `0x10` — basic server info
    /// - `0x11` — owner info
    /// - `0x12` — misc info (MOTD, location, URL, uptime)
    /// - `0x20` — players info (online, max, peak)
    /// - `0x30` — map info
    /// - `0x21` — extended players info (empty list here — no runtime game state)
    /// - `0x22` — player status info (always offline — no runtime game state)
    /// - `0x23` — server software info
    pub fn serialize_info(&self, requested: RequestedInfo, character_name: &str) -> Vec<u8> {
        let s = &self.status;
        let mut msg = NetworkMessage::new();

        if requested.has(RequestedInfo::BASIC_SERVER_INFO) {
            msg.add_u8(0x10);
            msg.add_string(&s.server_name);
            msg.add_string(&s.server_ip);
            msg.add_string(&s.server_port.to_string());
        }

        if requested.has(RequestedInfo::OWNER_SERVER_INFO) {
            msg.add_u8(0x11);
            msg.add_string(&s.owner_name);
            msg.add_string(&s.owner_email);
        }

        if requested.has(RequestedInfo::MISC_SERVER_INFO) {
            msg.add_u8(0x12);
            msg.add_string(&s.motd);
            msg.add_string(&s.location);
            msg.add_string(&s.url);
            msg.add_u64(s.uptime_seconds);
        }

        if requested.has(RequestedInfo::PLAYERS_INFO) {
            msg.add_u8(0x20);
            msg.add_u32(s.players_online);
            msg.add_u32(s.players_max);
            msg.add_u32(s.players_peak);
        }

        if requested.has(RequestedInfo::MAP_INFO) {
            msg.add_u8(0x30);
            msg.add_string(&s.map_name);
            msg.add_string(&s.map_author);
            msg.add_u16(s.map_width as u16);
            msg.add_u16(s.map_height as u16);
        }

        if requested.has(RequestedInfo::EXT_PLAYERS_INFO) {
            msg.add_u8(0x21);
            // player count (0 — no runtime state available)
            msg.add_u32(0);
        }

        if requested.has(RequestedInfo::PLAYER_STATUS_INFO) {
            msg.add_u8(0x22);
            // Always "offline" — no runtime game state in the protocol layer
            let _ = character_name;
            msg.add_u8(0x00);
        }

        if requested.has(RequestedInfo::SERVER_SOFTWARE_INFO) {
            msg.add_u8(0x23);
            msg.add_string(&s.server_name);
            msg.add_string(&s.server_version);
            msg.add_string(&s.client_version);
        }

        let start = forgottenserver_common::networkmessage::INITIAL_BUFFER_POSITION as usize;
        let end = start + msg.get_message_length() as usize;
        msg.get_buffer()[start..end].to_vec()
    }

    // -----------------------------------------------------------------------
    // Rate limiter
    // -----------------------------------------------------------------------

    /// Checks whether the IP address is allowed to make a status request.
    ///
    /// Returns `true` if no request has been made by this IP within the last
    /// 60 seconds (and records the current timestamp).  Returns `false` if a
    /// request was made within the last 60 seconds.
    pub fn check_rate_limit(&mut self, ip: u32) -> bool {
        self.check_rate_limit_at(ip, Instant::now())
    }

    /// Same as `check_rate_limit` but takes the "current" instant as a parameter
    /// so tests can simulate the cooldown elapsing without sleeping.
    ///
    /// Mirrors the C++ `OTSYS_TIME() < (last + STATUSQUERY_TIMEOUT)` check from
    /// `ProtocolStatus::onRecvFirstMessage`.
    pub fn check_rate_limit_at(&mut self, ip: u32, now: Instant) -> bool {
        if let Some(&last) = self.last_request.get(&ip) {
            if now.duration_since(last) < Duration::from_secs(60) {
                return false;
            }
        }
        self.last_request.insert(ip, now);
        true
    }

    /// Clears all rate-limit records (useful for testing).
    pub fn clear_rate_limits(&mut self) {
        self.last_request.clear();
    }
}

// ---------------------------------------------------------------------------
// Request parsing (mirrors ProtocolStatus::onRecvFirstMessage in C++)
// ---------------------------------------------------------------------------

/// Parsed inbound status request.
///
/// `onRecvFirstMessage` in protocolstatus.cpp dispatches on the first byte of
/// the inbound NetworkMessage:
///
/// * `0xFF` followed by the 4-byte string `"info"` → XML status response.
/// * `0x01` followed by a `u16` info flags bitmask, and (when
///   `PLAYER_STATUS_INFO` is set) a length-prefixed character name → binary
///   info response.
///
/// Any other byte sequence is rejected (the C++ code calls `disconnect()`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusRequest {
    /// XML status response request (`0xFF "info"`).
    Xml,
    /// Binary info request with the parsed flags and optional character name.
    Binary {
        requested: RequestedInfo,
        character_name: String,
    },
}

/// Parses an inbound status request byte stream.
///
/// Returns `None` for malformed / unsupported requests, matching the C++
/// "fall through and disconnect" behaviour in `onRecvFirstMessage`.
pub fn parse_request(bytes: &[u8]) -> Option<StatusRequest> {
    if bytes.is_empty() {
        return None;
    }
    match bytes[0] {
        REQUEST_XML_INFO => {
            // Expect the literal ASCII "info" (4 bytes) after the selector byte.
            if bytes.len() < 5 {
                return None;
            }
            if &bytes[1..5] == b"info" {
                Some(StatusRequest::Xml)
            } else {
                None
            }
        }
        REQUEST_BINARY_INFO => {
            if bytes.len() < 3 {
                return None;
            }
            let requested = RequestedInfo::new(u16::from_le_bytes([bytes[1], bytes[2]]));
            let character_name =
                if requested.has(RequestedInfo::PLAYER_STATUS_INFO) && bytes.len() >= 5 {
                    let name_len = u16::from_le_bytes([bytes[3], bytes[4]]) as usize;
                    let name_start = 5;
                    let name_end = name_start + name_len;
                    if bytes.len() < name_end {
                        return None;
                    }
                    String::from_utf8_lossy(&bytes[name_start..name_end]).into_owned()
                } else {
                    String::new()
                };
            Some(StatusRequest::Binary {
                requested,
                character_name,
            })
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// Parse helper (used in tests for round-trip verification)
// ---------------------------------------------------------------------------

/// Parse a byte slice produced by `serialize_status` back into a
/// `ServerStatus`.  Used only in tests — not part of the production API.
pub fn parse_status(bytes: &[u8]) -> Option<ServerStatus> {
    // We write the bytes into a NetworkMessage and read them back.
    let mut msg = NetworkMessage::new();
    msg.add_bytes(bytes);
    // reset read cursor to start of payload
    msg.set_buffer_position(0);

    let server_name = msg.get_string(0);
    let map_name = msg.get_string(0);
    let players_online = msg.get_u32();
    let players_max = msg.get_u32();
    let uptime_seconds = msg.get_u64();
    let exp_rate = msg.get_u32();

    if msg.is_overrun() {
        return None;
    }

    Some(ServerStatus {
        server_name,
        map_name,
        players_online,
        players_max,
        players_peak: 0,
        uptime_seconds,
        exp_rate,
        motd: String::new(),
        owner_name: String::new(),
        owner_email: String::new(),
        location: String::new(),
        url: String::new(),
        server_ip: String::new(),
        server_port: 0,
        map_author: String::new(),
        map_width: 0,
        map_height: 0,
        monsters_online: 0,
        npcs_online: 0,
        skill_rate: 1,
        loot_rate: 1,
        magic_rate: 1,
        spawn_rate: 1,
        server_version: String::new(),
        client_version: String::new(),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_status() -> ServerStatus {
        ServerStatus {
            server_name: "TestServer".to_string(),
            map_name: "mymap".to_string(),
            players_online: 42,
            players_max: 1000,
            players_peak: 500,
            uptime_seconds: 86400,
            exp_rate: 2,
            motd: "Welcome!".to_string(),
            owner_name: "Admin".to_string(),
            owner_email: "admin@example.com".to_string(),
            location: "EU".to_string(),
            url: "https://example.com".to_string(),
            server_ip: "127.0.0.1".to_string(),
            server_port: 7171,
            map_author: "Mapmaker".to_string(),
            map_width: 256,
            map_height: 256,
            monsters_online: 1000,
            npcs_online: 50,
            skill_rate: 3,
            loot_rate: 2,
            magic_rate: 3,
            spawn_rate: 1,
            server_version: "1.4".to_string(),
            client_version: "10.98".to_string(),
        }
    }

    fn make_protocol() -> ProtocolStatus {
        ProtocolStatus::new(make_status())
    }

    // -----------------------------------------------------------------------
    // serialize_status round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_status_round_trip_server_name() {
        let proto = make_protocol();
        let bytes = proto.serialize_status();
        let parsed = parse_status(&bytes).expect("parse_status should succeed");
        assert_eq!(parsed.server_name, "TestServer");
    }

    #[test]
    fn test_serialize_status_round_trip_map_name() {
        let proto = make_protocol();
        let bytes = proto.serialize_status();
        let parsed = parse_status(&bytes).expect("parse_status should succeed");
        assert_eq!(parsed.map_name, "mymap");
    }

    #[test]
    fn test_serialize_status_round_trip_players_online() {
        let proto = make_protocol();
        let bytes = proto.serialize_status();
        let parsed = parse_status(&bytes).expect("parse_status should succeed");
        assert_eq!(parsed.players_online, 42);
    }

    #[test]
    fn test_serialize_status_round_trip_players_max() {
        let proto = make_protocol();
        let bytes = proto.serialize_status();
        let parsed = parse_status(&bytes).expect("parse_status should succeed");
        assert_eq!(parsed.players_max, 1000);
    }

    #[test]
    fn test_serialize_status_round_trip_uptime() {
        let proto = make_protocol();
        let bytes = proto.serialize_status();
        let parsed = parse_status(&bytes).expect("parse_status should succeed");
        assert_eq!(parsed.uptime_seconds, 86400);
    }

    #[test]
    fn test_serialize_status_round_trip_exp_rate() {
        let proto = make_protocol();
        let bytes = proto.serialize_status();
        let parsed = parse_status(&bytes).expect("parse_status should succeed");
        assert_eq!(parsed.exp_rate, 2);
    }

    // -----------------------------------------------------------------------
    // get_xml_status
    // -----------------------------------------------------------------------

    #[test]
    fn test_xml_status_contains_players_online() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("<players online="),
            "XML must contain players online tag"
        );
    }

    #[test]
    fn test_xml_status_contains_server_name() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        // New format uses <serverinfo ... servername="...">
        assert!(
            xml.contains("servername="),
            "XML must contain servername attribute"
        );
    }

    #[test]
    fn test_xml_status_contains_correct_values() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("TestServer"),
            "XML must include server name value"
        );
        assert!(xml.contains("42"), "XML must include players online value");
        assert!(xml.contains("1000"), "XML must include players max value");
    }

    // -----------------------------------------------------------------------
    // Rate limiter
    // -----------------------------------------------------------------------

    #[test]
    fn test_rate_limit_allows_first_request() {
        let mut proto = make_protocol();
        assert!(
            proto.check_rate_limit(0x7F000001),
            "first request should be allowed"
        );
    }

    #[test]
    fn test_rate_limit_blocks_second_request_within_60s() {
        let mut proto = make_protocol();
        let ip = 0xC0A80001_u32;
        assert!(proto.check_rate_limit(ip));
        assert!(
            !proto.check_rate_limit(ip),
            "second request within 60s should be blocked"
        );
    }

    #[test]
    fn test_rate_limit_allows_after_clearing() {
        let mut proto = make_protocol();
        let ip = 0xC0A80002_u32;
        proto.check_rate_limit(ip);
        proto.clear_rate_limits();
        assert!(
            proto.check_rate_limit(ip),
            "request should be allowed after clearing rate limits"
        );
    }

    #[test]
    fn test_rate_limit_different_ips_are_independent() {
        let mut proto = make_protocol();
        assert!(proto.check_rate_limit(0x01010101));
        assert!(proto.check_rate_limit(0x02020202));
    }

    // -----------------------------------------------------------------------
    // get_xml_status — extended coverage (C++ sendStatusString fields)
    // -----------------------------------------------------------------------

    #[test]
    fn test_xml_status_has_tsqp_root() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(xml.contains("<tsqp"), "XML must contain tsqp root element");
        assert!(xml.contains("</tsqp>"), "XML must close tsqp element");
    }

    #[test]
    fn test_xml_status_has_serverinfo_element() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("<serverinfo"),
            "XML must contain serverinfo element"
        );
    }

    #[test]
    fn test_xml_status_has_uptime() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("uptime=\"86400\""),
            "XML must contain uptime value"
        );
    }

    #[test]
    fn test_xml_status_has_owner_element() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(xml.contains("<owner"), "XML must contain owner element");
        assert!(xml.contains("Admin"), "XML must contain owner name");
        assert!(
            xml.contains("admin@example.com"),
            "XML must contain owner email"
        );
    }

    #[test]
    fn test_xml_status_has_players_peak() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("peak=\"500\""),
            "XML must contain peak player count"
        );
    }

    #[test]
    fn test_xml_status_has_monsters_element() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("<monsters"),
            "XML must contain monsters element"
        );
        assert!(
            xml.contains("total=\"1000\""),
            "XML must contain monster count"
        );
    }

    #[test]
    fn test_xml_status_has_npcs_element() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(xml.contains("<npcs"), "XML must contain npcs element");
        assert!(xml.contains("total=\"50\""), "XML must contain npc count");
    }

    #[test]
    fn test_xml_status_has_rates_element() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(xml.contains("<rates"), "XML must contain rates element");
        assert!(
            xml.contains("experience="),
            "XML must contain experience rate"
        );
        assert!(xml.contains("skill="), "XML must contain skill rate");
        assert!(xml.contains("loot="), "XML must contain loot rate");
        assert!(xml.contains("magic="), "XML must contain magic rate");
        assert!(xml.contains("spawn="), "XML must contain spawn rate");
    }

    #[test]
    fn test_xml_status_has_map_element() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(xml.contains("<map"), "XML must contain map element");
        assert!(xml.contains("mymap"), "XML must contain map name");
        assert!(xml.contains("Mapmaker"), "XML must contain map author");
        assert!(xml.contains("width=\"256\""), "XML must contain map width");
        assert!(
            xml.contains("height=\"256\""),
            "XML must contain map height"
        );
    }

    #[test]
    fn test_xml_status_has_motd_element() {
        let proto = make_protocol();
        let xml = proto.get_xml_status();
        assert!(xml.contains("<motd>"), "XML must contain motd element");
        assert!(xml.contains("Welcome!"), "XML must contain motd text");
    }

    #[test]
    fn test_xml_status_escapes_special_chars_in_server_name() {
        let status = ServerStatus {
            server_name: "Server & <Test>".to_string(),
            ..make_status()
        };
        let proto = ProtocolStatus::new(status);
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("Server &amp; &lt;Test&gt;"),
            "must XML-escape special chars"
        );
        assert!(
            !xml.contains("Server & <Test>"),
            "must not contain unescaped chars"
        );
    }

    #[test]
    fn test_xml_status_escapes_quotes_in_url() {
        let status = ServerStatus {
            url: r#"http://example.com?a="b""#.to_string(),
            ..make_status()
        };
        let proto = ProtocolStatus::new(status);
        let xml = proto.get_xml_status();
        assert!(
            xml.contains("&quot;"),
            "must XML-escape double quotes in URL"
        );
    }

    // -----------------------------------------------------------------------
    // Protocol constants
    // -----------------------------------------------------------------------

    #[test]
    fn test_protocol_identifier_is_0xff() {
        assert_eq!(PROTOCOL_IDENTIFIER, 0xFF);
    }

    #[test]
    fn test_server_sends_first_is_false() {
        // Use indirection via an extern fn to defeat clippy::assertions_on_constants;
        // the assert below is semantically "this const must remain false".
        fn check(v: bool) -> bool {
            v
        }
        assert!(!check(SERVER_SENDS_FIRST));
    }

    #[test]
    fn test_use_checksum_is_false() {
        fn check(v: bool) -> bool {
            v
        }
        assert!(!check(USE_CHECKSUM));
    }

    #[test]
    fn test_protocol_name_constant() {
        assert_eq!(PROTOCOL_NAME, "status protocol");
    }

    #[test]
    fn test_request_byte_constants() {
        assert_eq!(REQUEST_XML_INFO, 0xFF);
        assert_eq!(REQUEST_BINARY_INFO, 0x01);
    }

    // -----------------------------------------------------------------------
    // RequestedInfo flags
    // -----------------------------------------------------------------------

    #[test]
    fn test_requested_info_default_is_zero() {
        let ri = RequestedInfo::default();
        assert_eq!(ri.0, 0);
    }

    #[test]
    fn test_requested_info_has_flag() {
        let ri = RequestedInfo::new(RequestedInfo::BASIC_SERVER_INFO | RequestedInfo::PLAYERS_INFO);
        assert!(ri.has(RequestedInfo::BASIC_SERVER_INFO));
        assert!(ri.has(RequestedInfo::PLAYERS_INFO));
        assert!(!ri.has(RequestedInfo::OWNER_SERVER_INFO));
        assert!(!ri.has(RequestedInfo::MAP_INFO));
    }

    #[test]
    fn test_requested_info_all_flags_distinct() {
        let flags = [
            RequestedInfo::BASIC_SERVER_INFO,
            RequestedInfo::OWNER_SERVER_INFO,
            RequestedInfo::MISC_SERVER_INFO,
            RequestedInfo::PLAYERS_INFO,
            RequestedInfo::MAP_INFO,
            RequestedInfo::EXT_PLAYERS_INFO,
            RequestedInfo::PLAYER_STATUS_INFO,
            RequestedInfo::SERVER_SOFTWARE_INFO,
        ];
        for i in 0..flags.len() {
            for j in 0..flags.len() {
                if i != j {
                    assert_ne!(
                        flags[i], flags[j],
                        "all RequestedInfo flags must be distinct"
                    );
                }
            }
        }
    }

    #[test]
    fn test_requested_info_power_of_two() {
        // Each flag should be a power of two (single bit set)
        for &flag in &[
            RequestedInfo::BASIC_SERVER_INFO,
            RequestedInfo::OWNER_SERVER_INFO,
            RequestedInfo::MISC_SERVER_INFO,
            RequestedInfo::PLAYERS_INFO,
            RequestedInfo::MAP_INFO,
            RequestedInfo::EXT_PLAYERS_INFO,
            RequestedInfo::PLAYER_STATUS_INFO,
            RequestedInfo::SERVER_SOFTWARE_INFO,
        ] {
            assert!(
                flag.is_power_of_two(),
                "flag {flag:#06x} must be a power of two"
            );
        }
    }

    // -----------------------------------------------------------------------
    // serialize_info
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_info_basic_server_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::BASIC_SERVER_INFO);
        let bytes = proto.serialize_info(ri, "");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x10, "basic server info tag must be 0x10");
    }

    #[test]
    fn test_serialize_info_owner_server_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::OWNER_SERVER_INFO);
        let bytes = proto.serialize_info(ri, "");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x11, "owner server info tag must be 0x11");
    }

    #[test]
    fn test_serialize_info_misc_server_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::MISC_SERVER_INFO);
        let bytes = proto.serialize_info(ri, "");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x12, "misc server info tag must be 0x12");
    }

    #[test]
    fn test_serialize_info_players_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::PLAYERS_INFO);
        let bytes = proto.serialize_info(ri, "");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x20, "players info tag must be 0x20");
    }

    #[test]
    fn test_serialize_info_map_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::MAP_INFO);
        let bytes = proto.serialize_info(ri, "");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x30, "map info tag must be 0x30");
    }

    #[test]
    fn test_serialize_info_ext_players_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::EXT_PLAYERS_INFO);
        let bytes = proto.serialize_info(ri, "");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x21, "ext players info tag must be 0x21");
    }

    #[test]
    fn test_serialize_info_player_status_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::PLAYER_STATUS_INFO);
        let bytes = proto.serialize_info(ri, "SomePlayer");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x22, "player status info tag must be 0x22");
    }

    #[test]
    fn test_serialize_info_server_software_info_tag() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::SERVER_SOFTWARE_INFO);
        let bytes = proto.serialize_info(ri, "");
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x23, "server software info tag must be 0x23");
    }

    #[test]
    fn test_serialize_info_empty_flags_produces_empty() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(0);
        let bytes = proto.serialize_info(ri, "");
        assert!(bytes.is_empty(), "no flags → empty output");
    }

    #[test]
    fn test_serialize_info_players_info_contains_online_count() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::PLAYERS_INFO);
        let bytes = proto.serialize_info(ri, "");
        // bytes[0] = tag 0x20, bytes[1..5] = online u32 LE
        assert!(bytes.len() >= 5);
        let online = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        assert_eq!(online, 42, "online count must match status");
    }

    #[test]
    fn test_serialize_info_players_info_contains_max_and_peak() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::PLAYERS_INFO);
        let bytes = proto.serialize_info(ri, "");
        // [0]=tag, [1..5]=online, [5..9]=max, [9..13]=peak
        assert!(bytes.len() >= 13);
        let max = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
        let peak = u32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);
        assert_eq!(max, 1000, "max must match status");
        assert_eq!(peak, 500, "peak must match status");
    }

    #[test]
    fn test_serialize_info_player_status_offline_by_default() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::PLAYER_STATUS_INFO);
        let bytes = proto.serialize_info(ri, "AnyPlayer");
        // [0]=tag 0x22, [1]=0x00 (offline)
        assert!(bytes.len() >= 2);
        assert_eq!(
            bytes[1], 0x00,
            "player should be shown as offline (no runtime state)"
        );
    }

    #[test]
    fn test_serialize_info_multiple_flags_emit_multiple_sections() {
        let proto = make_protocol();
        let ri = RequestedInfo::new(RequestedInfo::BASIC_SERVER_INFO | RequestedInfo::PLAYERS_INFO);
        let bytes = proto.serialize_info(ri, "");
        // Must contain both tag 0x10 and tag 0x20
        assert!(bytes.contains(&0x10), "must contain basic server info tag");
        assert!(bytes.contains(&0x20), "must contain players info tag");
    }

    // -----------------------------------------------------------------------
    // check_rate_limit_at — cooldown elapsed path (covers the "else" branch
    // of the inner `if duration < 60s`)
    // -----------------------------------------------------------------------

    #[test]
    fn test_rate_limit_at_allows_request_after_cooldown_elapsed() {
        let mut proto = make_protocol();
        let ip = 0xDEAD_BEEF_u32;
        let t0 = Instant::now();
        assert!(
            proto.check_rate_limit_at(ip, t0),
            "first request at t0 allowed"
        );
        // Same IP, but 61s later → cooldown elapsed, must allow.
        let t1 = t0 + Duration::from_secs(61);
        assert!(
            proto.check_rate_limit_at(ip, t1),
            "request after 60s cooldown must be allowed"
        );
    }

    #[test]
    fn test_rate_limit_at_blocks_within_cooldown() {
        let mut proto = make_protocol();
        let ip = 0xCAFE_BABE_u32;
        let t0 = Instant::now();
        assert!(proto.check_rate_limit_at(ip, t0));
        let t1 = t0 + Duration::from_secs(30);
        assert!(
            !proto.check_rate_limit_at(ip, t1),
            "30s after first request is still within the 60s cooldown"
        );
    }

    #[test]
    fn test_rate_limit_at_exactly_60s_is_allowed() {
        let mut proto = make_protocol();
        let ip = 0x1234_5678_u32;
        let t0 = Instant::now();
        assert!(proto.check_rate_limit_at(ip, t0));
        // Exactly 60s — C++ uses `<`, so 60s is allowed.
        let t1 = t0 + Duration::from_secs(60);
        assert!(
            proto.check_rate_limit_at(ip, t1),
            "exactly 60s after the first request is allowed (strict `<` comparison)"
        );
    }

    // -----------------------------------------------------------------------
    // parse_status — overrun path (covers `return None` when reads underflow)
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_status_returns_none_on_short_input() {
        // A very short buffer cannot satisfy the read of two strings, two u32,
        // a u64, and another u32 — `parse_status` must return None.
        let bytes = vec![0u8; 3];
        let result = parse_status(&bytes);
        assert!(
            result.is_none(),
            "parse_status must return None when the buffer is too short to deserialize"
        );
    }

    // -----------------------------------------------------------------------
    // parse_request — covers `onRecvFirstMessage` dispatch in C++
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_request_empty_returns_none() {
        assert!(parse_request(&[]).is_none());
    }

    #[test]
    fn test_parse_request_unknown_selector_returns_none() {
        // 0x42 is not a recognised first byte
        assert!(parse_request(&[0x42, 0x00, 0x00]).is_none());
    }

    #[test]
    fn test_parse_request_xml_with_info_string() {
        let bytes = [0xFF, b'i', b'n', b'f', b'o'];
        assert_eq!(parse_request(&bytes), Some(StatusRequest::Xml));
    }

    #[test]
    fn test_parse_request_xml_truncated_returns_none() {
        // 0xFF followed by only 3 bytes (not enough for "info")
        let bytes = [0xFF, b'i', b'n', b'f'];
        assert!(parse_request(&bytes).is_none());
    }

    #[test]
    fn test_parse_request_xml_wrong_keyword_returns_none() {
        let bytes = [0xFF, b'd', b'a', b't', b'a'];
        assert!(parse_request(&bytes).is_none());
    }

    #[test]
    fn test_parse_request_binary_with_no_character_name() {
        // 0x01 + u16 flags = PLAYERS_INFO (1<<3 = 8)
        let bytes = [0x01, 0x08, 0x00];
        let parsed = parse_request(&bytes).expect("parse_request should succeed");
        assert_eq!(
            parsed,
            StatusRequest::Binary {
                requested: RequestedInfo::new(RequestedInfo::PLAYERS_INFO),
                character_name: String::new(),
            }
        );
    }

    #[test]
    fn test_parse_request_binary_truncated_returns_none() {
        // 0x01 followed by only one byte (not enough for u16 flags)
        let bytes = [0x01, 0x08];
        assert!(parse_request(&bytes).is_none());
    }

    #[test]
    fn test_parse_request_binary_with_character_name() {
        // 0x01 + flags (PLAYER_STATUS_INFO = 1<<6 = 64) + u16 name_len = 4 + "Test"
        let name = b"Test";
        let name_len = name.len() as u16;
        let mut bytes = vec![0x01, 0x40, 0x00];
        bytes.extend_from_slice(&name_len.to_le_bytes());
        bytes.extend_from_slice(name);
        let parsed = parse_request(&bytes).expect("parse_request should succeed");
        assert_eq!(
            parsed,
            StatusRequest::Binary {
                requested: RequestedInfo::new(RequestedInfo::PLAYER_STATUS_INFO),
                character_name: "Test".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_request_binary_player_status_truncated_name() {
        // PLAYER_STATUS_INFO set, but the name length claims 10 bytes that aren't there
        let bytes = [0x01, 0x40, 0x00, 0x0A, 0x00, b'A', b'B'];
        assert!(parse_request(&bytes).is_none());
    }

    #[test]
    fn test_parse_request_binary_player_status_no_name_field() {
        // PLAYER_STATUS_INFO set, but no name length / bytes at all → falls through
        // to empty name (matches the C++ "characterName remains empty" relaxed path
        // because msg.getString() would also return an empty string on overrun).
        let bytes = [0x01, 0x40, 0x00];
        let parsed = parse_request(&bytes).expect("parse_request should succeed");
        assert_eq!(
            parsed,
            StatusRequest::Binary {
                requested: RequestedInfo::new(RequestedInfo::PLAYER_STATUS_INFO),
                character_name: String::new(),
            }
        );
    }
}
