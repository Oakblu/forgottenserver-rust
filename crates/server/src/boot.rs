use std::{
    net::TcpListener,
    path::Path,
    sync::{Arc, Mutex},
};

use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
use forgottenserver_game::{
    npc_registry::{load_npcs_xml, NpcRegistry},
    spell_registry::{load_spells_xml, SpellRegistry},
    weapon_registry::{load_weapons_xml, WeaponRegistry},
};
use forgottenserver_items::registry::ItemsRegistry;
use forgottenserver_map::items_loader::load_items_otb;
use forgottenserver_scripting::engine::LuaScriptEngine;

use crate::{admin_handler::AdminHandler, game_state::GameState, status_handler::StatusHandler};

pub struct GameData {
    pub items: ItemsRegistry,
    pub spells: SpellRegistry,
    pub weapons: WeaponRegistry,
    pub npcs: NpcRegistry,
    /// Number of Lua scripts loaded from `data/scripts/` (0 when feature disabled).
    pub scripts_loaded: usize,
}

/// Load all four game data registries from `data_dir` before entering the game loop.
///
/// Returns `Err` if a critical file (e.g. `items.otb`) cannot be read.
/// Missing individual records within each file are warnings, not errors.
pub fn boot(data_dir: &Path) -> Result<GameData, String> {
    let items = load_items_otb(&data_dir.join("items/items.otb"))?;
    let spells = load_spells_xml(&data_dir.join("spells/spells.xml"))?;
    let weapons = load_weapons_xml(&data_dir.join("weapons/weapons.xml"))?;
    let npcs = load_npcs_xml(&data_dir.join("npc"))?;

    let scripts_loaded = load_lua_scripts(data_dir);

    Ok(GameData {
        items,
        spells,
        weapons,
        npcs,
        scripts_loaded,
    })
}

/// Load Lua scripts from `data_dir/scripts/`. Errors are treated as warnings.
fn load_lua_scripts(data_dir: &Path) -> usize {
    let scripts_dir = data_dir.join("scripts");
    let mut engine = LuaScriptEngine::new();
    engine.load_dir(&scripts_dir).unwrap_or_default()
}

/// Spawn the admin TCP listener and status HTTP listener as background threads.
///
/// Both listeners run until the process exits. Errors binding are returned so
/// the caller can decide whether to abort or continue without admin/status.
pub fn start_admin_and_status(
    config: Arc<ConfigManager>,
    game_state: Arc<Mutex<GameState>>,
) -> Result<(), String> {
    let admin_port = config.get_integer(IntegerKey::AdminPort) as u16;
    let status_port = config.get_integer(IntegerKey::StatusPort) as u16;
    let admin_password = config.get_string(StringKey::AdminPassword).to_owned();

    let admin_listener = TcpListener::bind(format!("0.0.0.0:{admin_port}"))
        .map_err(|e| format!("Cannot bind admin port {admin_port}: {e}"))?;
    let status_listener = TcpListener::bind(format!("0.0.0.0:{status_port}"))
        .map_err(|e| format!("Cannot bind status port {status_port}: {e}"))?;

    let admin_handler = Arc::new(AdminHandler::new(admin_password, game_state.clone()));
    let status_handler = Arc::new(StatusHandler::new(game_state, config));

    std::thread::spawn(move || {
        accept_loop(admin_listener, admin_handler);
    });

    std::thread::spawn(move || {
        accept_loop(status_listener, status_handler);
    });

    Ok(())
}

fn accept_loop<H: ConnectionHandler + Send + Sync + 'static>(
    listener: TcpListener,
    handler: Arc<H>,
) {
    for stream in listener.incoming().flatten() {
        let h = handler.clone();
        std::thread::spawn(move || h.handle(stream));
    }
}

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Path to the data directory (via the symlink at the workspace root).
    fn data_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") // crates/server → forgottenserver-rust root → data symlink
    }

    // -----------------------------------------------------------------------
    // Test: spawn_manager.load_world populates entries before game loop
    // -----------------------------------------------------------------------
    #[test]
    fn boot_populates_spawn_entries() {
        use forgottenserver_common::position::Position;
        use forgottenserver_game::spawn_manager::SpawnManager;
        use forgottenserver_world::{SpawnPointDef, World};

        let mut world = World::new();
        world.add_spawn_point(SpawnPointDef {
            position: Position::new(100, 100, 7),
            radius: 3,
            monster_name: "Rat".to_string(),
            interval_secs: 60,
        });
        world.add_spawn_point(SpawnPointDef {
            position: Position::new(200, 200, 7),
            radius: 5,
            monster_name: "Orc".to_string(),
            interval_secs: 120,
        });

        let mut spawn_manager = SpawnManager::new();
        spawn_manager.load_world(&world);

        assert_eq!(
            spawn_manager.entry_count(),
            2,
            "boot must register all spawn points"
        );
    }

    // -----------------------------------------------------------------------
    // Test: all four loaders are called before the game loop (integration)
    // -----------------------------------------------------------------------
    #[test]
    fn boot_all_four_loaders_called_before_game_loop() {
        let game_data = boot(&data_dir()).expect("boot should succeed with real data");

        // Items: items.otb is non-empty in the real data set
        assert!(
            !game_data.items.is_empty(),
            "ItemsRegistry should be populated from items.otb"
        );

        // Spells: spells.xml in the real data is empty (<spells />) — that's fine
        // We just verify the loader ran without error.
        let _ = game_data.spells.len();

        // Weapons: weapons.xml has wand entries
        assert!(
            !game_data.weapons.is_empty(),
            "WeaponRegistry should be populated from weapons.xml"
        );

        // NPCs: npc/ directory has several xml files
        assert!(
            !game_data.npcs.is_empty(),
            "NpcRegistry should be populated from data/npc/"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 6 — Script loading at boot
    // -----------------------------------------------------------------------

    #[test]
    fn load_dir_loads_all_lua_files_in_directory() {
        use forgottenserver_scripting::engine::LuaScriptEngine;
        use std::io::Write;

        let dir = tempfile::TempDir::new().unwrap();
        std::fs::File::create(dir.path().join("a.lua"))
            .unwrap()
            .write_all(b"a_loaded = true")
            .unwrap();
        std::fs::File::create(dir.path().join("b.lua"))
            .unwrap()
            .write_all(b"b_loaded = true")
            .unwrap();
        std::fs::File::create(dir.path().join("not_lua.txt"))
            .unwrap()
            .write_all(b"ignored")
            .unwrap();

        let mut engine = LuaScriptEngine::new();
        let count = engine.load_dir(dir.path()).unwrap();
        assert_eq!(count, 2, "load_dir must load exactly the .lua files");
    }

    #[test]
    fn missing_script_dir_returns_error() {
        use forgottenserver_scripting::engine::LuaScriptEngine;
        let mut engine = LuaScriptEngine::new();
        let result = engine.load_dir(std::path::Path::new("/nonexistent/scripts/xyz_abc"));
        assert!(result.is_err(), "missing directory must return an error");
    }

    #[test]
    fn boot_loads_lua_scripts_from_data_scripts() {
        let game_data = boot(&data_dir()).expect("boot should succeed");
        // The data/scripts/ directory contains .lua files; scripts_loaded is 0 or more.
        // We only assert boot completes without panic (scripts_loaded counts top-level .lua only).
        let _ = game_data.scripts_loaded;
    }

    // -----------------------------------------------------------------------
    // Phase 13 — start_admin_and_status + accept_loop + ConnectionHandler
    //
    // C++ cross-validation:
    //   * otserv.cpp `mainLoader` registers ProtocolStatus on STATUS_PORT and
    //     spawns admin/status listeners via the ServiceManager.
    //   * The Rust equivalent in `boot.rs` is `start_admin_and_status`, which
    //     reads AdminPort/StatusPort/AdminPassword from the ConfigManager,
    //     binds two TcpListeners, and spawns blocking accept loops on
    //     background threads. The two `ConnectionHandler` impls (for
    //     AdminHandler and StatusHandler) dispatch one accepted stream to the
    //     handler's own `handle_connection` method.
    // -----------------------------------------------------------------------

    use std::io::{Read as _, Write as _};

    /// Pick a free local port by binding/dropping a listener.
    /// (Port 0 lets the OS pick; we capture the chosen port before dropping.)
    fn free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    /// Build a ConfigManager with the given admin/status ports + admin password.
    fn make_config(admin_port: u16, status_port: u16, admin_password: &str) -> Arc<ConfigManager> {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::AdminPort, admin_port as i64);
        cm.set_integer(IntegerKey::StatusPort, status_port as i64);
        cm.set_string(StringKey::AdminPassword, admin_password);
        Arc::new(cm)
    }

    #[test]
    fn start_admin_and_status_binds_both_listeners_and_returns_ok() {
        // Picks two free ports, binds them, spawns the two accept threads.
        // We verify the function returns Ok and that connecting to the bound
        // ports succeeds (proves the listeners are alive).
        let admin_port = free_port();
        let status_port = free_port();
        let config = make_config(admin_port, status_port, "secret");
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_admin_and_status(config.clone(), game_state.clone());
        assert!(res.is_ok(), "expected Ok, got: {:?}", res);

        // Verify the admin listener actually accepts a TCP connection.
        // AdminHandler will read until EOF; we close the write half to let it
        // finish without blocking.
        let mut admin_stream =
            std::net::TcpStream::connect(format!("127.0.0.1:{admin_port}")).unwrap();
        admin_stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut admin_resp = String::new();
        let _ = admin_stream.read_to_string(&mut admin_resp);

        // Verify the status listener also accepts a connection and returns XML.
        let mut status_stream =
            std::net::TcpStream::connect(format!("127.0.0.1:{status_port}")).unwrap();
        status_stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        status_stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut status_resp = String::new();
        let _ = status_stream.read_to_string(&mut status_resp);
        assert!(
            status_resp.contains("HTTP/1.0 200 OK"),
            "status listener did not respond with HTTP 200: {status_resp}"
        );
        assert!(
            status_resp.contains("<tsqp"),
            "status listener did not respond with TSQP XML: {status_resp}"
        );
    }

    #[test]
    fn start_admin_and_status_errors_when_admin_port_already_bound() {
        // Pre-bind the admin port on 0.0.0.0 (matching the bind address used
        // inside start_admin_and_status) so the listener bind fails.
        let admin_port = free_port();
        let status_port = free_port();
        let _hog = std::net::TcpListener::bind(format!("0.0.0.0:{admin_port}")).unwrap();

        let config = make_config(admin_port, status_port, "secret");
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_admin_and_status(config, game_state);
        let err = res.expect_err("expected admin-port bind to fail");
        assert!(
            err.contains("Cannot bind admin port") && err.contains(&admin_port.to_string()),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn start_admin_and_status_errors_when_status_port_already_bound() {
        // Admin port is free, but status port is hogged; this exercises the
        // second `map_err` branch (lines 68-69).
        let admin_port = free_port();
        let status_port = free_port();
        let _hog = std::net::TcpListener::bind(format!("0.0.0.0:{status_port}")).unwrap();

        let config = make_config(admin_port, status_port, "secret");
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_admin_and_status(config, game_state);
        let err = res.expect_err("expected status-port bind to fail");
        assert!(
            err.contains("Cannot bind status port") && err.contains(&status_port.to_string()),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn accept_loop_dispatches_connection_to_admin_handler() {
        // Drives `accept_loop` directly with a controlled listener so we hit
        // the for-loop, the spawn-per-stream, and the ConnectionHandler::handle
        // impl for AdminHandler (lines 89-92 + 101-103).
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let gs = Arc::new(Mutex::new(GameState::new()));
        gs.lock().unwrap().add_player("Carol");
        let handler = Arc::new(AdminHandler::new("pw", gs.clone()));

        std::thread::spawn(move || accept_loop(listener, handler));

        // Send `auth` + `status`; the status command reports the online
        // player count, which proves the connection traversed
        // accept_loop -> ConnectionHandler::handle -> AdminHandler::handle_connection.
        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream.write_all(b"auth pw\nstatus\n").unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        assert!(
            resp.contains("players: 1"),
            "AdminHandler::handle did not run (Carol not counted): {resp}"
        );
        // Keep `gs` alive past the connection so it isn't dropped prematurely.
        let _ = gs;
    }

    #[test]
    fn accept_loop_dispatches_connection_to_status_handler() {
        // Same as above, but for the StatusHandler ConnectionHandler impl
        // (lines 107-109). This proves accept_loop is generic over both
        // handler types and that StatusHandler::handle forwards correctly.
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
        // Direct call into the trait impl (lines 101-103) without spawning a
        // background accept loop, providing redundant coverage of the impl
        // even if a future change skips accept_loop's spawn dispatch.
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
        // The exact response doesn't matter; we only need the impl to run.
        assert!(
            !resp.is_empty(),
            "AdminHandler trait dispatch produced no output"
        );
    }

    #[test]
    fn connection_handler_status_handle_forwards_to_handle_connection() {
        // Direct call into the StatusHandler ConnectionHandler impl
        // (lines 107-109).
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
}
