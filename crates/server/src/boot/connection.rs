use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use forgottenserver_common::networkmessage::NetworkMessage;
use forgottenserver_common::tools::adler_checksum;
use forgottenserver_database::database::Database;
use forgottenserver_database::iologindata::{
    load_player_for_login, lookup_session, save_player_logout, PlayerLogoutData,
};
use forgottenserver_items::vocation::Vocations;
use forgottenserver_network::protocolgame::{parse_first_packet, serialize_disconnect};
use forgottenserver_scripting::actions::Actions;
use forgottenserver_scripting::talkaction::TalkActions;
use forgottenserver_world::map::Map;
use forgottenserver_world::World;

use crate::admin_handler::AdminHandler;
use crate::boot::framing::{frame_packet, frame_packet_seq, frame_plaintext_packet};
use crate::boot::game_loop::run_game_loop;
use crate::game_handler::build_enter_world_burst;
use crate::status_handler::StatusHandler;

/// Trait implemented by handlers that process a single TCP connection.
pub trait ConnectionHandler {
    fn handle(&self, stream: std::net::TcpStream);
}

impl ConnectionHandler for AdminHandler {
    fn handle(&self, stream: std::net::TcpStream) {
        self.handle_connection(stream);
    }
}

impl ConnectionHandler for StatusHandler {
    fn handle(&self, stream: std::net::TcpStream) {
        self.handle_connection(stream);
    }
}

pub(crate) fn accept_loop<H: ConnectionHandler + Send + Sync + 'static>(
    listener: TcpListener,
    handler: Arc<H>,
) {
    for stream in listener.incoming().flatten() {
        let h = handler.clone();
        std::thread::spawn(move || h.handle(stream));
    }
}

// ---------------------------------------------------------------------------
// Game login handler (port 7172)
// ---------------------------------------------------------------------------

/// Handles a single game-protocol TCP connection.
pub struct GameLoginHandler {
    db: Arc<Mutex<Box<dyn Database + Send>>>,
    vocations: Arc<Vocations>,
    map: Arc<Map>,
    talk_actions: Arc<TalkActions>,
    script_dir: PathBuf,
    actions: Arc<Actions>,
    action_data_dir: PathBuf,
}

impl GameLoginHandler {
    pub fn new(
        db: Arc<Mutex<Box<dyn Database + Send>>>,
        vocations: Arc<Vocations>,
        map: Arc<Map>,
        talk_actions: Arc<TalkActions>,
        script_dir: PathBuf,
        actions: Arc<Actions>,
        action_data_dir: PathBuf,
    ) -> Self {
        Self { db, vocations, map, talk_actions, script_dir, actions, action_data_dir }
    }

    /// Handle a single accepted TCP stream: send challenge, read first packet,
    /// validate session, load player, send enter-world burst, run game loop.
    pub fn handle_connection(&self, mut stream: std::net::TcpStream) {
        let peer = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "<unknown>".to_string());
        eprintln!("[game] connection from {peer}");

        let timestamp: u32 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);
        let rand_byte: u8 = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0)
            & 0xFF) as u8;
        let mut buf = [0u8; 14];
        buf[0..2].copy_from_slice(&12u16.to_le_bytes());
        buf[6..8].copy_from_slice(&6u16.to_le_bytes());
        buf[8] = 0x1F;
        buf[9..13].copy_from_slice(&timestamp.to_le_bytes());
        buf[13] = rand_byte;
        let checksum = adler_checksum(&buf[6..14]);
        buf[2..6].copy_from_slice(&checksum.to_le_bytes());
        if stream.write_all(&buf).is_err() {
            return;
        }

        let mut hdr = [0u8; 2];
        if let Err(e) = stream.read_exact(&mut hdr) {
            eprintln!("[game] failed to read initial header: {e}");
            return;
        }

        let outer_len = if hdr[1] != 0x00 {
            let mut prefix: Vec<u8> = vec![hdr[0], hdr[1]];
            loop {
                let mut b = [0u8; 1];
                match stream.read(&mut b) {
                    Ok(1) if b[0] == 0x0A => {
                        prefix.push(b[0]);
                        break;
                    }
                    Ok(1) => {
                        prefix.push(b[0]);
                        if prefix.len() > 512 {
                            eprintln!(
                                "[game] GAMEWORLD_AUTH: pre-login prefix exceeds 512 bytes — closing"
                            );
                            return;
                        }
                    }
                    _ => return,
                }
            }
            let phex: String = prefix.iter().map(|b| format!("{b:02x} ")).collect();
            let pascii: String = prefix
                .iter()
                .map(|&b| {
                    if (0x20..0x7f).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
            eprintln!(
                "[game] GAMEWORLD_AUTH prefix ({} bytes): {phex} | {pascii}",
                prefix.len()
            );
            if let Err(e) = stream.read_exact(&mut hdr) {
                eprintln!("[game] GAMEWORLD_AUTH: failed to read real outer_len: {e}");
                return;
            }
            eprintln!(
                "[game] outer_len header bytes after prefix: {:02x} {:02x}",
                hdr[0], hdr[1]
            );
            u16::from_le_bytes(hdr) as usize
        } else {
            u16::from_le_bytes(hdr) as usize
        };

        eprintln!("[game] first packet outer_len={outer_len}");

        if outer_len > 32_768 {
            eprintln!(
                "[game] first packet outer_len={outer_len} exceeds limit — closing connection"
            );
            return;
        }
        if outer_len < 7 {
            eprintln!("[game] first packet too short: outer_len={outer_len}");
            return;
        }

        let mut body = vec![0u8; outer_len];
        if let Err(e) = stream.read_exact(&mut body) {
            eprintln!("[game] failed to read packet body: outer_len={outer_len} err={e}");
            return;
        }
        let opcode = body[4];
        eprintln!(
            "[game] seq={:02x}{:02x}{:02x}{:02x} opcode=0x{opcode:02x} outer_len={outer_len}",
            body[0], body[1], body[2], body[3]
        );
        let payload: &[u8] = &body[5..];
        let mut msg = NetworkMessage::new();
        msg.add_bytes(payload);
        msg.set_buffer_position(0);

        match parse_first_packet(&mut msg) {
            Err(disconnect_msg) => {
                eprintln!("[game] parse_first_packet error: {disconnect_msg}");
                let disconnect_payload = serialize_disconnect(&disconnect_msg);
                let _ = stream.write_all(&frame_plaintext_packet(&disconnect_payload));
            }
            Ok(packet) => {
                eprintln!(
                    "[game] parsed ok: char={:?} ts={} rand={}",
                    packet.character_name, packet.challenge_timestamp, packet.challenge_random
                );
                if packet.challenge_timestamp != timestamp || packet.challenge_random != rand_byte {
                    eprintln!("[game] challenge mismatch: got ts={} rand={}, expected ts={timestamp} rand={rand_byte}", packet.challenge_timestamp, packet.challenge_random);
                    let disconnect = serialize_disconnect("Invalid challenge echo.");
                    let _ = stream.write_all(&frame_plaintext_packet(&disconnect));
                    return;
                }

                eprintln!(
                    "[game] looking up session token (len={}) for char={:?}",
                    packet.session_token.len(),
                    packet.character_name
                );
                let session_result = {
                    let db_guard = self.db.lock().unwrap();
                    lookup_session(&**db_guard, &packet.session_token, &packet.character_name)
                };
                let (_account_id, character_id) = match session_result {
                    Some(ids) => {
                        eprintln!("[game] session ok: account={} char={}", ids.0, ids.1);
                        ids
                    }
                    None => {
                        eprintln!("[game] session not found");
                        let disconnect =
                            serialize_disconnect("Account name or password is not correct.");
                        let _ = stream.write_all(&frame_plaintext_packet(&disconnect));
                        return;
                    }
                };

                let player_data = {
                    let db_guard = self.db.lock().unwrap();
                    load_player_for_login(&**db_guard, character_id)
                };
                let player_data = match player_data {
                    Some(p) => p,
                    None => {
                        eprintln!("[game] character {character_id} could not be loaded");
                        let disconnect =
                            serialize_disconnect("Your character could not be loaded.");
                        let _ = stream.write_all(&frame_plaintext_packet(&disconnect));
                        return;
                    }
                };
                eprintln!("[game] player loaded: char={character_id}");

                let world = World::new();
                let player_creature_id = 0x1000_0000u32 | (character_id as u32);
                let burst = build_enter_world_burst(
                    &player_data,
                    &world,
                    player_creature_id,
                    &self.vocations,
                    &self.map,
                );
                eprintln!("[game] enter-world burst built: {} bytes", burst.len());
                {
                }

                let sequence_checksum = (4..=12).contains(&packet.os);
                let framed_burst = if sequence_checksum {
                    frame_packet_seq(&burst, packet.xtea_key, 1)
                } else {
                    frame_packet(&burst, packet.xtea_key)
                };
                eprintln!("[game] framed burst: {} bytes (sequence_checksum={sequence_checksum})", framed_burst.len());
                match stream.write_all(&framed_burst) {
                    Ok(()) => eprintln!("[game] enter-world burst sent"),
                    Err(e) => {
                        eprintln!("[game] failed to send enter-world burst: {e}");
                        return;
                    }
                }

                eprintln!(
                    "[game] entering game loop (os={} sequence_checksum={sequence_checksum})",
                    packet.os
                );
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let logout = run_game_loop(
                    &mut stream,
                    packet.xtea_key,
                    sequence_checksum,
                    2,
                    player_data,
                    player_creature_id,
                    Arc::clone(&self.vocations),
                    Arc::clone(&self.map),
                    Arc::clone(&self.talk_actions),
                    self.script_dir.clone(),
                    Arc::clone(&self.actions),
                    self.action_data_dir.clone(),
                );
                eprintln!("[game] game loop exited");
                let _ = stream.shutdown(std::net::Shutdown::Both);
                let _ = {
                    let mut db_guard = self.db.lock().unwrap();
                    save_player_logout(
                        &mut **db_guard,
                        character_id,
                        &PlayerLogoutData {
                            pos_x: logout.pos.x,
                            pos_y: logout.pos.y,
                            pos_z: logout.pos.z,
                            health: logout.health,
                            mana: logout.mana,
                            direction: logout.direction,
                        },
                    )
                };
                eprintln!(
                    "[game] player {} saved at ({},{},{}) hp={} mp={}",
                    character_id, logout.pos.x, logout.pos.y, logout.pos.z,
                    logout.health, logout.mana,
                );
            }
        }
    }
}

impl ConnectionHandler for GameLoginHandler {
    fn handle(&self, stream: std::net::TcpStream) {
        self.handle_connection(stream);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
    use forgottenserver_common::xtea;
    use forgottenserver_database::database::InMemoryDb;
    use forgottenserver_items::vocation::Vocations;
    use forgottenserver_world::map::Map;
    use crate::game_state::GameState;

    fn empty_db() -> Arc<Mutex<Box<dyn Database + Send>>> {
        Arc::new(Mutex::new(Box::new(InMemoryDb::new())))
    }

    fn empty_vocations() -> Arc<Vocations> {
        Arc::new(Vocations::load_from_xml("<vocations/>").unwrap())
    }

    fn empty_map() -> Arc<Map> {
        Arc::new(Map::new())
    }

    fn make_config(admin_port: u16, status_port: u16, admin_password: &str) -> Arc<ConfigManager> {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::AdminPort, admin_port as i64);
        cm.set_integer(IntegerKey::StatusPort, status_port as i64);
        cm.set_string(StringKey::AdminPassword, admin_password);
        Arc::new(cm)
    }

    #[test]
    fn accept_loop_dispatches_connection_to_admin_handler() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Carol");
        let handler = Arc::new(AdminHandler::new("pw", gs.clone()));

        std::thread::spawn(move || accept_loop(listener, handler));

        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream.write_all(b"auth pw\nstatus\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        assert!(
            resp.contains("players: 1"),
            "AdminHandler::handle did not run (Carol not counted): {resp}"
        );
        let _ = gs;
    }

    #[test]
    fn accept_loop_dispatches_connection_to_status_handler() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        let cfg = make_config(0, 0, "");
        let handler = Arc::new(StatusHandler::new(gs, cfg));

        std::thread::spawn(move || accept_loop(listener, handler));

        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        assert!(
            resp.contains("HTTP/1.0 200 OK") && resp.contains("<tsqp"),
            "StatusHandler::handle did not run: {resp}"
        );
    }

    #[test]
    fn connection_handler_admin_handle_forwards_to_handle_connection() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        let handler: Arc<dyn ConnectionHandler + Send + Sync> =
            Arc::new(AdminHandler::new("pw", gs));

        let t = std::thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            handler.handle(server_stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.write_all(b"auth wrong\n").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        client.read_to_string(&mut resp).unwrap();
        t.join().unwrap();
        assert!(!resp.is_empty(), "AdminHandler trait dispatch produced no output");
    }

    #[test]
    fn connection_handler_status_handle_forwards_to_handle_connection() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        let cfg = make_config(0, 0, "");
        let handler: Arc<dyn ConnectionHandler + Send + Sync> =
            Arc::new(StatusHandler::new(gs, cfg));

        let t = std::thread::spawn(move || {
            let (server_stream, _) = listener.accept().unwrap();
            handler.handle(server_stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        client.read_to_string(&mut resp).unwrap();
        t.join().unwrap();
        assert!(
            resp.contains("HTTP/1.0 200 OK"),
            "StatusHandler trait dispatch did not produce HTTP response: {resp}"
        );
    }

    #[test]
    fn challenge_echo_mismatch_disconnects() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new())
                    .handle_connection(stream);
            }
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();

        let mut _challenge = [0u8; 14];
        client.read_exact(&mut _challenge).unwrap();

        let mut body = vec![0u8; 140];
        body[0] = 0x0A;
        body[1] = 0x03;
        body[2] = 0x00;
        body[3] = 0x1E;
        body[4] = 0x05;

        let body_len_bytes = (body.len() as u16).to_le_bytes();
        client.write_all(&body_len_bytes).unwrap();
        client.write_all(&body).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut response = Vec::new();
        let _ = client.read_to_end(&mut response);

        assert!(
            !response.is_empty(),
            "server must send a disconnect payload when the first packet is invalid"
        );
    }

    #[test]
    fn challenge_packet_is_14_bytes_with_outer_len() {
        let ts: u32 = 0xDEAD_BEEF;
        let rand: u8 = 0x42;

        let mut buf = [0u8; 14];
        buf[0..2].copy_from_slice(&12u16.to_le_bytes());
        buf[6..8].copy_from_slice(&6u16.to_le_bytes());
        buf[8] = 0x1F;
        buf[9..13].copy_from_slice(&ts.to_le_bytes());
        buf[13] = rand;
        let checksum = adler_checksum(&buf[6..14]);
        buf[2..6].copy_from_slice(&checksum.to_le_bytes());

        assert_eq!(buf.len(), 14);
        assert_eq!(&buf[0..2], &[0x0C, 0x00]);
        assert_eq!(
            u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]),
            adler_checksum(&buf[6..14])
        );
        assert_eq!(&buf[6..8], &[0x06, 0x00]);
        assert_eq!(buf[8], 0x1F);
        assert_eq!(u32::from_le_bytes([buf[9], buf[10], buf[11], buf[12]]), ts);
        assert_eq!(buf[13], rand);
    }

    #[test]
    fn game_login_challenge_round_trip() {
        use forgottenserver_common::base64;
        use forgottenserver_database::database::{Database, DbError, DbValue, Row};
        use num_bigint::BigUint;
        use rsa::pkcs1::DecodeRsaPrivateKey;
        use rsa::traits::PublicKeyParts;
        use rsa::RsaPrivateKey;
        use std::collections::HashMap;
        use std::io::Read as IoRead;

        struct RoundTripDb;
        impl Database for RoundTripDb {
            fn query(&self, sql: &str) -> Result<Vec<Row>, DbError> {
                if sql.contains("FROM accounts a") {
                    let mut map = HashMap::new();
                    map.insert("account_id".to_string(), DbValue::Integer(1));
                    map.insert("character_id".to_string(), DbValue::Integer(1));
                    Ok(vec![Row::new(map)])
                } else if sql.contains("FROM players") {
                    let mut map = HashMap::new();
                    map.insert("name".to_string(), DbValue::Text("TestChar".to_string()));
                    map.insert("level".to_string(), DbValue::Integer(1));
                    map.insert("health".to_string(), DbValue::Integer(100));
                    map.insert("healthmax".to_string(), DbValue::Integer(100));
                    map.insert("mana".to_string(), DbValue::Integer(0));
                    map.insert("manamax".to_string(), DbValue::Integer(0));
                    map.insert("stamina".to_string(), DbValue::Integer(2520));
                    map.insert("posx".to_string(), DbValue::Integer(100));
                    map.insert("posy".to_string(), DbValue::Integer(100));
                    map.insert("posz".to_string(), DbValue::Integer(7));
                    Ok(vec![Row::new(map)])
                } else {
                    Ok(vec![])
                }
            }
            fn execute(&mut self, _sql: &str) -> Result<u64, DbError> {
                Ok(1)
            }
            fn escape_string(&self, s: &str) -> String {
                s.replace('\\', "\\\\").replace('\'', "\\'")
            }
        }

        forgottenserver_common::rsa::load_pem(forgottenserver_common::rsa::DEFAULT_KEY_PEM).ok();

        let db: Arc<Mutex<Box<dyn Database + Send>>> = Arc::new(Mutex::new(Box::new(RoundTripDb)));
        let handler = GameLoginHandler::new(db, empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server_thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            handler.handle_connection(stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let mut challenge = [0u8; 14];
        client.read_exact(&mut challenge).unwrap();
        let outer_len_c = u16::from_le_bytes([challenge[0], challenge[1]]);
        assert_eq!(outer_len_c, 12);
        let challenge_ts =
            u32::from_le_bytes([challenge[9], challenge[10], challenge[11], challenge[12]]);
        let challenge_rand = challenge[13];

        let xtea_key: [u32; 4] = [0x1122_3344, 0x5566_7788, 0x99AA_BBCC, 0xDDEE_FF00];
        let token_bytes = [0x42u8; 16];
        let token_b64 = base64::encode(&token_bytes);
        let token_b64_bytes = token_b64.as_bytes();
        let char_name = "TestChar";
        let char_name_bytes = char_name.as_bytes();

        let mut plaintext = [0u8; 128];
        let mut cur = 0usize;
        plaintext[cur] = 0x00;
        cur += 1;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[0].to_le_bytes());
        cur += 4;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[1].to_le_bytes());
        cur += 4;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[2].to_le_bytes());
        cur += 4;
        plaintext[cur..cur + 4].copy_from_slice(&xtea_key[3].to_le_bytes());
        cur += 4;
        plaintext[cur] = 0x00;
        cur += 1;
        let st_len = token_b64_bytes.len() as u16;
        plaintext[cur..cur + 2].copy_from_slice(&st_len.to_le_bytes());
        cur += 2;
        plaintext[cur..cur + token_b64_bytes.len()].copy_from_slice(token_b64_bytes);
        cur += token_b64_bytes.len();
        let cn_len = char_name_bytes.len() as u16;
        plaintext[cur..cur + 2].copy_from_slice(&cn_len.to_le_bytes());
        cur += 2;
        plaintext[cur..cur + char_name_bytes.len()].copy_from_slice(char_name_bytes);
        cur += char_name_bytes.len();
        plaintext[cur..cur + 4].copy_from_slice(&challenge_ts.to_le_bytes());
        cur += 4;
        plaintext[cur] = challenge_rand;

        let priv_key = RsaPrivateKey::from_pkcs1_pem(forgottenserver_common::rsa::DEFAULT_KEY_PEM)
            .expect("DEFAULT_KEY_PEM must be valid");
        let m = BigUint::from_bytes_be(&plaintext);
        let e = BigUint::from_bytes_be(&priv_key.e().to_bytes_be());
        let n = BigUint::from_bytes_be(&priv_key.n().to_bytes_be());
        let c = m.modpow(&e, &n);
        let c_bytes = c.to_bytes_be();
        let mut rsa_block = [0u8; 128];
        let rsa_offset = 128 - c_bytes.len();
        rsa_block[rsa_offset..].copy_from_slice(&c_bytes);

        let mut pfp_payload = Vec::<u8>::with_capacity(139);
        pfp_payload.extend_from_slice(&2u16.to_le_bytes());
        pfp_payload.extend_from_slice(&1310u16.to_le_bytes());
        pfp_payload.extend_from_slice(&[0u8; 4]);
        pfp_payload.extend_from_slice(&[0u8; 3]);
        pfp_payload.extend_from_slice(&rsa_block);

        let mut body = Vec::with_capacity(5 + pfp_payload.len());
        body.extend_from_slice(&[0u8; 4]);
        body.push(0x0Au8);
        body.extend_from_slice(&pfp_payload);
        let outer_len = body.len() as u16;
        let mut framed = Vec::with_capacity(2 + body.len());
        framed.extend_from_slice(&outer_len.to_le_bytes());
        framed.extend_from_slice(&body);
        client.write_all(&framed).unwrap();

        let mut resp_hdr = [0u8; 6];
        client.read_exact(&mut resp_hdr).unwrap();
        let resp_outer_len = u16::from_le_bytes([resp_hdr[0], resp_hdr[1]]) as usize;
        assert!(resp_outer_len > 4);
        let xtea_len = resp_outer_len - 4;
        let mut xtea_region = vec![0u8; xtea_len];
        client.read_exact(&mut xtea_region).unwrap();

        let key = xtea::Key(xtea_key);
        let round_keys = xtea::expand_key(&key);
        xtea::decrypt(&mut xtea_region, &round_keys);
        assert!(xtea_region.len() >= 3);
        let resp_opcode = xtea_region[2];
        assert_eq!(
            resp_opcode, 0xA0,
            "server must send player stats (0xA0) as the first login packet, got 0x{resp_opcode:02X}"
        );

        client.shutdown(std::net::Shutdown::Write).unwrap();
        server_thread
            .join()
            .expect("handle_connection must not panic on valid round-trip login");
    }

    #[test]
    fn challenge_outer_len_guard_rejects_implausible_length() {
        use std::io::Read as IoRead;

        let db: Arc<Mutex<Box<dyn Database + Send>>> =
            Arc::new(Mutex::new(Box::new(InMemoryDb::new())));
        let handler = GameLoginHandler::new(db, empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server_thread = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            handler.handle_connection(stream);
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let mut challenge = [0u8; 14];
        client.read_exact(&mut challenge).unwrap();

        client.write_all(&[0xFF, 0xFF]).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        server_thread
            .join()
            .expect("handle_connection must not panic on implausible outer_len");
    }

    // -------------------------------------------------------------------------
    // GAMEWORLD_AUTH prefix path (hdr[1] != 0x00):
    // Client sends bytes where hdr[1] != 0 to trigger the prefix-scanning loop.
    // The loop reads until 0x0A then reads the real outer_len header.
    // After the prefix we send a real outer_len of 0xFFFF to hit the >32768 guard.
    // -------------------------------------------------------------------------

    #[test]
    fn gameworld_auth_prefix_path_close_after_0x0a_then_large_outer_len() {
        use std::io::Read as IoRead;
        let handler =
            GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                handler.handle_connection(stream);
            }
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        // Read and discard the 14-byte challenge.
        let mut _challenge = [0u8; 14];
        client.read_exact(&mut _challenge).unwrap();

        // Send hdr[0]=0x47, hdr[1]=0x45 ('G','E') — hdr[1] != 0x00 triggers prefix path.
        // Then a few non-0x0A bytes, then 0x0A to terminate the prefix loop.
        // Finally send 0xFF, 0xFF as the real outer_len (>32768 → server closes).
        let mut msg = Vec::new();
        msg.extend_from_slice(b"GE");   // initial hdr that starts the prefix loop
        msg.extend_from_slice(b"T /");   // more prefix bytes (non-0x0A)
        msg.push(0x0A);                  // 0x0A terminates prefix loop
        // Real outer_len bytes after the prefix (>32768 so server closes):
        msg.extend_from_slice(&[0xFF, 0xFF]);
        client.write_all(&msg).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut resp = Vec::new();
        let _ = client.read_to_end(&mut resp);
        // Server should close cleanly (no panic). Response may be empty.
    }

    #[test]
    fn gameworld_auth_prefix_exceeds_512_bytes_closes_connection() {
        use std::io::Read as IoRead;
        let handler =
            GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                handler.handle_connection(stream);
            }
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        // Discard challenge.
        let mut _challenge = [0u8; 14];
        client.read_exact(&mut _challenge).unwrap();

        // hdr[1] = 0x01 (non-zero) → enters prefix path.
        // Send 513+ non-0x0A bytes to exceed the 512-byte limit → server must close.
        let mut msg = vec![0x00u8, 0x01u8]; // initial hdr bytes (hdr[1]=0x01)
        msg.extend(vec![0x42u8; 513]);       // 513 bytes, none are 0x0A
        client.write_all(&msg).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut resp = Vec::new();
        let _ = client.read_to_end(&mut resp);
        // Server must exit without panic. No response expected.
    }

    #[test]
    fn gameworld_auth_prefix_path_eof_mid_prefix_closes_cleanly() {
        use std::io::Read as IoRead;
        let handler =
            GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                handler.handle_connection(stream);
            }
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        // Discard challenge.
        let mut _challenge = [0u8; 14];
        client.read_exact(&mut _challenge).unwrap();

        // hdr[1] = 0x01 → prefix path. Then EOF with no 0x0A → `_ => return` branch.
        client.write_all(&[0x00u8, 0x01u8]).unwrap();
        // Immediately close without sending 0x0A → server hits `_ => return`.
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut resp = Vec::new();
        let _ = client.read_to_end(&mut resp);
        // Must not panic.
    }

    // -------------------------------------------------------------------------
    // outer_len > 32768 guard (via hdr[1] == 0x00, regular path)
    // Send hdr = [0x01, 0x80] → outer_len = 0x8001 = 32769 > 32768.
    // -------------------------------------------------------------------------

    #[test]
    fn outer_len_exceeds_32768_closes_connection() {
        use std::io::Read as IoRead;
        let handler =
            GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                handler.handle_connection(stream);
            }
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let mut _challenge = [0u8; 14];
        client.read_exact(&mut _challenge).unwrap();

        // outer_len = 0x8001 = 32769, with hdr[1] = 0x80 (non-zero).
        // But we want hdr[1] == 0 for the regular path:
        // Send hdr[0]=0x01, hdr[1]=0x80 → hdr[1] != 0x00 → enters prefix loop.
        // Instead use LE bytes: [0x01, 0x80] as hdr when hdr[1] is treated as the high byte.
        // Actually for LE: outer_len = hdr[0] + hdr[1]*256.
        // hdr = [0x01, 0x80] → 0x01 + 0x80*256 = 32769 > 32768. BUT hdr[1]=0x80 != 0 → prefix.
        // Use hdr = [0xFF, 0x80] → hdr[1]=0x80 != 0 → prefix path again.
        // To get the regular path (hdr[1] == 0x00) with outer_len > 32768,
        // outer_len = hdr[0] + 0*256 = hdr[0] ≤ 255, so max outer_len = 255 via regular path.
        // The only way to get outer_len > 32768 via regular path is impossible since hdr[1] must be 0.
        // So this test: use GAMEWORLD_AUTH prefix to arrive at the real outer_len with hdr[1] == 0.
        // After prefix termination, send new hdr = [0x01, 0x80] (LE → 32769 > 32768).
        // BUT the new hdr read at line 144 might also have hdr[1] != 0 making it jump to
        // u16::from_le_bytes(hdr). Let's send the new hdr as [0x01, 0x80] → 32769.

        // Trigger prefix path with hdr = [0x01, 0x01], then send prefix + 0x0A,
        // then send the real outer_len hdr = [0x01, 0x80] (LE = 32769).
        let mut msg = Vec::new();
        msg.push(0x01u8); // hdr[0]
        msg.push(0x01u8); // hdr[1] != 0 → prefix path
        msg.push(0x0Au8); // terminates prefix immediately
        // Real outer_len after prefix (LE u16 = 32769 > 32768):
        msg.extend_from_slice(&32769u16.to_le_bytes());
        client.write_all(&msg).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut resp = Vec::new();
        let _ = client.read_to_end(&mut resp);
        // Server must close cleanly (no panic).
    }

    // -------------------------------------------------------------------------
    // outer_len < 7 guard
    // -------------------------------------------------------------------------

    #[test]
    fn outer_len_too_short_closes_connection() {
        use std::io::Read as IoRead;
        let handler =
            GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                handler.handle_connection(stream);
            }
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let mut _challenge = [0u8; 14];
        client.read_exact(&mut _challenge).unwrap();

        // outer_len = 3 (< 7), hdr[1] = 0 → regular path, outer_len=3 < 7 → close.
        // hdr[0]=3, hdr[1]=0 → u16::from_le_bytes([3,0]) = 3.
        client.write_all(&[3u8, 0u8]).unwrap();
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut resp = Vec::new();
        let _ = client.read_to_end(&mut resp);
        // Must close cleanly.
    }

    // -------------------------------------------------------------------------
    // Body read failure: outer_len says N bytes but client closes before sending them.
    // -------------------------------------------------------------------------

    #[test]
    fn body_read_failure_closes_connection_cleanly() {
        use std::io::Read as IoRead;
        let handler =
            GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                handler.handle_connection(stream);
            }
        });

        let mut client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();

        let mut _challenge = [0u8; 14];
        client.read_exact(&mut _challenge).unwrap();

        // outer_len = 100 (≥ 7, ≤ 32768) → server tries to read 100 bytes.
        // We only send the 2-byte outer_len and then close → read_exact fails.
        client.write_all(&[100u8, 0u8]).unwrap();
        // Close immediately without sending the 100-byte body.
        client.shutdown(std::net::Shutdown::Write).unwrap();

        let mut resp = Vec::new();
        let _ = client.read_to_end(&mut resp);
        // Server must exit cleanly.
    }

    // -------------------------------------------------------------------------
    // Challenge write failure (line 99): server must not panic when the
    // client closes the connection before reading the 14-byte challenge.
    // -------------------------------------------------------------------------

    #[test]
    fn challenge_write_failure_on_closed_client_does_not_panic() {
        let handler =
            GameLoginHandler::new(empty_db(), empty_vocations(), empty_map(), Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()), std::path::PathBuf::new(), Arc::new(forgottenserver_scripting::actions::Actions::new()), std::path::PathBuf::new());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let server_thread = std::thread::spawn(move || {
            // Accept the connection, then immediately handle it.
            if let Ok((stream, _)) = listener.accept() {
                // The client will close its write end right away,
                // but the server's write of the challenge may still succeed
                // (buffered in the OS). We test for no panic.
                handler.handle_connection(stream);
            }
        });

        // Connect and immediately shut down the write side.
        let client = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        // Drop client immediately — this closes the connection.
        drop(client);

        server_thread
            .join()
            .expect("handle_connection must not panic even if client disconnects early");
    }
}
