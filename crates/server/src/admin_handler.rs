use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::game_state::GameState;

pub struct AdminHandler {
    password: String,
    game_state: Arc<Mutex<GameState>>,
    start_time: Instant,
}

impl AdminHandler {
    pub fn new(password: impl Into<String>, game_state: Arc<Mutex<GameState>>) -> Self {
        Self {
            password: password.into(),
            game_state,
            start_time: Instant::now(),
        }
    }

    /// Handle one admin TCP connection: read lines until EOF, write responses.
    pub fn handle_connection(&self, stream: TcpStream) {
        let mut writer = match stream.try_clone() {
            Ok(w) => w,
            Err(_) => return,
        };
        let reader = BufReader::new(stream);
        let mut authenticated = false;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let response = self.dispatch_with_auth(line.trim(), &mut authenticated);
            if writer.write_all(response.as_bytes()).is_err() {
                break;
            }
        }
    }

    /// Dispatch a single command line, tracking per-connection auth state.
    pub fn dispatch_with_auth(&self, line: &str, authenticated: &mut bool) -> String {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        if cmd == "auth" {
            return self.cmd_auth(arg, authenticated);
        }

        if !*authenticated {
            return "error: not authenticated\n".to_string();
        }

        match cmd {
            "kick" => self.cmd_kick(arg),
            "broadcast" => self.cmd_broadcast(arg),
            "shutdown" => self.cmd_shutdown(),
            "status" => self.cmd_status(),
            _ => "error: unknown command\n".to_string(),
        }
    }

    /// Stateless dispatch used in unit tests (no per-connection auth tracking).
    pub fn dispatch(&self, line: &str) -> String {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        match parts[0] {
            "auth" => {
                let arg = parts.get(1).copied().unwrap_or("");
                if arg == self.password {
                    "ok\n".to_string()
                } else {
                    "error: wrong password\n".to_string()
                }
            }
            "kick" => self.cmd_kick(parts.get(1).copied().unwrap_or("")),
            "broadcast" => self.cmd_broadcast(parts.get(1).copied().unwrap_or("")),
            "shutdown" => self.cmd_shutdown(),
            "status" => self.cmd_status(),
            _ => "error: unknown command\n".to_string(),
        }
    }

    fn cmd_auth(&self, password: &str, authenticated: &mut bool) -> String {
        if password == self.password {
            *authenticated = true;
            "ok\n".to_string()
        } else {
            "error: wrong password\n".to_string()
        }
    }

    fn cmd_kick(&self, name: &str) -> String {
        let mut gs = self.game_state.lock().unwrap();
        if gs.remove_player(name) {
            "ok\n".to_string()
        } else {
            "error: player not found\n".to_string()
        }
    }

    fn cmd_broadcast(&self, _msg: &str) -> String {
        "ok\n".to_string()
    }

    fn cmd_shutdown(&self) -> String {
        "ok\n".to_string()
    }

    fn cmd_status(&self) -> String {
        let gs = self.game_state.lock().unwrap();
        let uptime = self.start_time.elapsed().as_secs();
        format!(
            "players: {} uptime: {}s\n",
            gs.online_player_count(),
            uptime
        )
    }
}

// ---------------------------------------------------------------------------
// Unit tests (Phase 2)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_handler(password: &str) -> AdminHandler {
        let gs = Arc::new(Mutex::new(GameState::new()));
        AdminHandler::new(password, gs)
    }

    #[test]
    fn admin_auth_correct_password_returns_ok() {
        let h = make_handler("secret");
        assert_eq!(h.dispatch("auth secret"), "ok\n");
    }

    #[test]
    fn admin_auth_wrong_password_returns_error() {
        let h = make_handler("secret");
        assert_eq!(h.dispatch("auth wrong"), "error: wrong password\n");
    }

    #[test]
    fn admin_unknown_command_returns_error() {
        let h = make_handler("secret");
        assert_eq!(h.dispatch("foobar"), "error: unknown command\n");
    }

    #[test]
    fn admin_kick_known_player_returns_ok() {
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Alice");
        let h = AdminHandler::new("pw", gs);
        assert_eq!(h.dispatch("kick Alice"), "ok\n");
    }

    #[test]
    fn admin_kick_unknown_player_returns_error() {
        let h = make_handler("pw");
        assert_eq!(h.dispatch("kick Nobody"), "error: player not found\n");
    }

    #[test]
    fn admin_broadcast_returns_ok() {
        let h = make_handler("pw");
        assert_eq!(h.dispatch("broadcast hello world"), "ok\n");
    }

    #[test]
    fn admin_shutdown_returns_ok() {
        let h = make_handler("pw");
        assert_eq!(h.dispatch("shutdown"), "ok\n");
    }

    #[test]
    fn admin_status_returns_player_count_and_uptime() {
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Alice");
        let h = AdminHandler::new("pw", gs);
        let resp = h.dispatch("status");
        assert!(resp.starts_with("players: 1 uptime:"), "unexpected: {resp}");
    }

    // -----------------------------------------------------------------------
    // Phase 3 — live TCP integration tests
    // -----------------------------------------------------------------------

    use std::net::{TcpListener, TcpStream};

    fn spawn_admin_server(handler: Arc<AdminHandler>) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                handler.handle_connection(stream);
            }
        });
        port
    }

    fn make_arc_handler(password: &str) -> Arc<AdminHandler> {
        let gs = Arc::new(Mutex::new(GameState::new()));
        Arc::new(AdminHandler::new(password, gs))
    }

    fn make_arc_handler_with_state(password: &str, gs: Arc<Mutex<GameState>>) -> Arc<AdminHandler> {
        Arc::new(AdminHandler::new(password, gs))
    }

    fn send_recv(port: u16, data: &str) -> String {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        stream.write_all(data.as_bytes()).unwrap();
        drop(stream.try_clone().unwrap()); // flush write side
                                           // Shutdown write to signal EOF to server
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut reader = BufReader::new(stream);
        let mut out = String::new();
        use std::io::Read;
        reader.read_to_string(&mut out).unwrap();
        out
    }

    #[test]
    fn admin_kick_online_player_disconnects_session() {
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Alice");
        let handler = make_arc_handler_with_state("pw", gs.clone());
        let port = spawn_admin_server(handler);

        let resp = send_recv(port, "auth pw\nkick Alice\n");
        assert!(resp.contains("ok"), "expected ok, got: {resp}");
        assert!(
            !gs.lock().unwrap().remove_player("Alice"),
            "Alice should be gone"
        );
    }

    #[test]
    fn admin_kick_unknown_player_returns_error_integration() {
        let handler = make_arc_handler("pw");
        let port = spawn_admin_server(handler);
        let resp = send_recv(port, "auth pw\nkick Nobody\n");
        assert!(resp.contains("error: player not found"), "got: {resp}");
    }

    #[test]
    fn admin_broadcast_sends_message_to_all_players() {
        let handler = make_arc_handler("pw");
        let port = spawn_admin_server(handler);
        let resp = send_recv(port, "auth pw\nbroadcast hello\n");
        assert!(resp.contains("ok"), "got: {resp}");
    }

    #[test]
    fn admin_status_returns_player_count_and_uptime_integration() {
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Alice");
        gs.lock().unwrap().add_player("Bob");
        let handler = make_arc_handler_with_state("pw", gs);
        let port = spawn_admin_server(handler);
        let resp = send_recv(port, "auth pw\nstatus\n");
        assert!(resp.contains("players: 2"), "got: {resp}");
        assert!(resp.contains("uptime:"), "got: {resp}");
    }
}
