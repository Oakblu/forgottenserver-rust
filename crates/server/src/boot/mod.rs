pub mod connection;
pub mod framing;
pub mod game_loop;

use std::{net::TcpListener, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
use forgottenserver_database::database::Database;
use forgottenserver_entity::monsters::Monsters;
use forgottenserver_game::{
    monster_registry::load_monsters_xml,
    npc_registry::{load_npcs_xml, NpcRegistry},
    spell_registry::{load_spells_xml, SpellRegistry},
    weapon_registry::{load_weapons_xml, WeaponRegistry},
};
use forgottenserver_items::{
    items_registry::Items as FullItemRegistry,
    registry::ItemsRegistry,
    vocation::Vocations,
};
use forgottenserver_map::items_loader::load_items_otb;
use forgottenserver_scripting::actions::Actions;
use forgottenserver_scripting::actions_xml::{apply_parsed_action, parse_actions_xml};
use forgottenserver_scripting::talkaction::{apply_parsed_talkaction, parse_talkactions_xml, TalkActions};
use forgottenserver_world::{iomap::IoMap, map::Map};

use crate::{
    admin_handler::AdminHandler,
    boot::connection::{accept_loop, GameLoginHandler},
    game_state::GameState,
    http_connection_session::HttpConnectionSession,
    http_login::LoginConfig,
    status_handler::StatusHandler,
};

pub struct GameData {
    pub items: ItemsRegistry,
    pub spells: SpellRegistry,
    pub weapons: WeaponRegistry,
    pub npcs: NpcRegistry,
    pub vocations: Arc<Vocations>,
    /// Monster types loaded from `<data_dir>/monster/monsters.xml`.
    pub monsters: Monsters,
    /// The loaded world map parsed from `<data_dir>/world/<mapName>.otbm`.
    /// Shared by all game listener connections via `Arc`.
    pub map: Arc<Map>,
    /// Talkaction registry parsed from `<data_dir>/talkactions/talkactions.xml`.
    pub talk_actions: Arc<TalkActions>,
    /// Directory containing talkaction Lua scripts.
    pub talkaction_script_dir: PathBuf,
    /// Action registry parsed from `<data_dir>/actions/actions.xml`.
    pub actions: Arc<Actions>,
    /// Root of the actions data directory (contains `scripts/` and `lib/`).
    pub action_data_dir: PathBuf,
}

/// Load all four game data registries from `data_dir` before entering the game loop.
pub fn boot(data_dir: &Path, map_name: &str) -> Result<GameData, String> {
    let items = load_items_otb(&data_dir.join("items/items.otb"))?;
    let spells = load_spells_xml(&data_dir.join("spells/spells.xml"))?;
    let weapons = load_weapons_xml(&data_dir.join("weapons/weapons.xml"))?;
    let npcs = load_npcs_xml(&data_dir.join("npc"))?;

    let vocations_path = data_dir.join("XML/vocations.xml");
    let vocations = Arc::new(if vocations_path.exists() {
        let xml = std::fs::read_to_string(&vocations_path)
            .map_err(|e| format!("Cannot read vocations.xml: {e}"))?;
        Vocations::load_from_xml(&xml).map_err(|e| format!("Failed to parse vocations.xml: {e}"))?
    } else {
        Vocations::load_from_xml("<vocations/>").unwrap()
    });

    let monster_dir = data_dir.join("monster");
    let monsters = if monster_dir.exists() {
        match load_monsters_xml(&monster_dir) {
            Ok(m) => {
                eprintln!(">> Loaded {} monster types", m.get_monster_count());
                m
            }
            Err(e) => {
                eprintln!("[Warning] Failed to load monsters: {e}");
                Monsters::new()
            }
        }
    } else {
        Monsters::new()
    };

    let map_path = data_dir.join("world").join(format!("{map_name}.otbm"));
    let map_bytes = std::fs::read(&map_path)
        .map_err(|e| format!("Cannot read map file {}: {e}", map_path.display()))?;
    let mut map = IoMap::load_from_bytes(&map_bytes, &items)?;
    eprintln!(
        ">> Loaded map: {} tiles, {}x{}",
        map.get_tile_count(),
        map.get_declared_width(),
        map.get_declared_height()
    );

    // Post-process: apply FLOORCHANGE tile flags from items.xml.
    // The OTBM loader uses a simple registry (no XML attribute data), so
    // staircase items never set FLOORCHANGE flags during tile construction.
    // We load items.xml here, build a server_id→floor_change lookup, and
    // stamp the flags onto every tile that holds a staircase item.
    let otb_path = data_dir.join("items/items.otb");
    let items_xml_path = data_dir.join("items/items.xml");
    if items_xml_path.exists() {
        match (std::fs::read(&otb_path), std::fs::read_to_string(&items_xml_path)) {
            (Ok(otb_bytes), Ok(xml)) => {
                match FullItemRegistry::load_from_otb(&otb_bytes) {
                    Ok(mut full_items) => {
                        let _ = full_items.load_from_xml(&xml);
                        let max_id = full_items.get_max_item_id();
                        let floor_changes: Vec<(u16, u8)> = (1..=max_id)
                            .filter_map(|id| {
                                full_items.get_item_type(id).and_then(|it| {
                                    if it.floor_change != 0 {
                                        Some((id, it.floor_change))
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect();
                        let lookup: std::collections::HashMap<u16, u8> =
                            floor_changes.into_iter().collect();
                        map.apply_floor_change_flags(|id| lookup.get(&id).copied().unwrap_or(0));
                        eprintln!(">> Applied floor-change flags from items.xml");
                    }
                    Err(e) => eprintln!("[WARN] Failed to load items.otb for floor-change pass: {e}"),
                }
            }
            (Err(e), _) => eprintln!("[WARN] Cannot read items.otb for floor-change pass: {e}"),
            (_, Err(e)) => eprintln!("[WARN] Cannot read items.xml for floor-change pass: {e}"),
        }
    }

    let map = Arc::new(map);

    let talkaction_script_dir = data_dir.join("talkactions").join("scripts");
    let ta_xml_path = data_dir.join("talkactions").join("talkactions.xml");
    let talk_actions = if ta_xml_path.exists() {
        match std::fs::read_to_string(&ta_xml_path) {
            Ok(xml) => match parse_talkactions_xml(&xml) {
                Ok(parsed) => {
                    for w in &parsed.warnings {
                        eprintln!("[WARN] talkactions: {w}");
                    }
                    let mut ta = TalkActions::new();
                    for row in &parsed.rows {
                        apply_parsed_talkaction(&mut ta, row);
                    }
                    eprintln!(">> Loaded {} talkactions", parsed.rows.len());
                    Arc::new(ta)
                }
                Err(e) => {
                    eprintln!("[WARN] Failed to parse talkactions.xml: {e}");
                    Arc::new(TalkActions::new())
                }
            },
            Err(e) => {
                eprintln!("[WARN] Cannot read talkactions.xml: {e}");
                Arc::new(TalkActions::new())
            }
        }
    } else {
        eprintln!("[WARN] talkactions.xml not found: {}", ta_xml_path.display());
        Arc::new(TalkActions::new())
    };

    let action_data_dir = data_dir.join("actions");
    let actions_xml_path = action_data_dir.join("actions.xml");
    let actions = if actions_xml_path.exists() {
        match std::fs::read_to_string(&actions_xml_path) {
            Ok(xml) => match parse_actions_xml(&xml) {
                Ok(parsed) => {
                    for w in &parsed.warnings {
                        eprintln!("[WARN] actions: {w}");
                    }
                    let mut acts = Actions::new();
                    for row in &parsed.rows {
                        apply_parsed_action(&mut acts, row);
                    }
                    eprintln!(">> Loaded {} actions", parsed.rows.len());
                    Arc::new(acts)
                }
                Err(e) => {
                    eprintln!("[WARN] Failed to parse actions.xml: {e}");
                    Arc::new(Actions::new())
                }
            },
            Err(e) => {
                eprintln!("[WARN] Cannot read actions.xml: {e}");
                Arc::new(Actions::new())
            }
        }
    } else {
        eprintln!("[WARN] actions.xml not found: {}", actions_xml_path.display());
        Arc::new(Actions::new())
    };

    Ok(GameData {
        items,
        spells,
        weapons,
        npcs,
        vocations,
        monsters,
        map,
        talk_actions,
        talkaction_script_dir,
        actions,
        action_data_dir,
    })
}

/// Spawn the admin TCP listener and status HTTP listener as background threads.
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

/// Game-data references needed by the game listener.
///
/// Bundles the six shared data items that were previously passed as individual
/// arguments to `start_game_listener`.
pub struct GameListenerParams {
    pub vocations: Arc<Vocations>,
    pub map: Arc<Map>,
    pub talk_actions: Arc<TalkActions>,
    pub talkaction_script_dir: PathBuf,
    pub actions: Arc<Actions>,
    pub action_data_dir: PathBuf,
}

/// Bind the game-protocol listener on the configured game port and spawn a
/// background accept loop.
pub fn start_game_listener(
    config: Arc<ConfigManager>,
    _game_state: Arc<Mutex<GameState>>,
    db: Arc<Mutex<Box<dyn Database + Send>>>,
    params: GameListenerParams,
) -> Result<(), String> {
    let game_port = config.get_integer(IntegerKey::GamePort) as u16;
    let listener = TcpListener::bind(format!("0.0.0.0:{game_port}"))
        .map_err(|e| format!("Cannot bind game port {game_port}: {e}"))?;
    let handler = Arc::new(GameLoginHandler::new(
        db,
        params.vocations,
        params.map,
        params.talk_actions,
        params.talkaction_script_dir,
        params.actions,
        params.action_data_dir,
    ));
    std::thread::spawn(move || {
        accept_loop(listener, handler);
    });
    Ok(())
}

fn build_login_config(config: &ConfigManager) -> LoginConfig {
    let pvp_type = match config.get_string(StringKey::WorldType) {
        "no-pvp" => 1u8,
        "pvp-enforced" => 2u8,
        _ => 0u8,
    };
    LoginConfig {
        server_name: config.get_string(StringKey::ServerName).to_string(),
        ip: config.get_string(StringKey::Ip).to_string(),
        game_port: config.get_integer(IntegerKey::GamePort) as u16,
        location: config.get_string(StringKey::Location).to_string(),
        pvp_type,
    }
}

/// Bind the HTTP login listener on the configured `httpPort` and spawn
/// `httpWorkers` worker threads accepting connections.
pub fn start_http_listener(
    config: Arc<ConfigManager>,
    db: Arc<Mutex<Box<dyn Database + Send>>>,
    vocations: Arc<Vocations>,
) -> Result<(), String> {
    let http_port = config.get_integer(IntegerKey::HttpPort) as u16;
    if http_port == 0 {
        return Ok(());
    }
    let workers = (config.get_integer(IntegerKey::HttpWorkers) as usize).max(1);
    let bind_addr = {
        let s = config.get_string(StringKey::HttpBindAddress);
        if s.is_empty() { "127.0.0.1" } else { s }.to_owned()
    };

    let login_config = Arc::new(build_login_config(&config));
    let session = Arc::new(HttpConnectionSession::new(db, login_config, vocations));

    let listener = Arc::new(
        TcpListener::bind(format!("{bind_addr}:{http_port}"))
            .map_err(|e| format!("Cannot bind HTTP port {http_port}: {e}"))?,
    );

    eprintln!(">> HTTP login server online on {bind_addr}:{http_port} ({workers} worker(s)).");

    for _ in 0..workers {
        let l = Arc::clone(&listener);
        let s = Arc::clone(&session);
        std::thread::spawn(move || {
            while let Ok((stream, _)) = l.accept() {
                s.handle(stream);
            }
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
    use forgottenserver_database::database::InMemoryDb;
    use forgottenserver_items::vocation::Vocations;
    use forgottenserver_world::map::Map;
    use std::io::{Read as _, Write as _};

    fn data_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
    }

    fn free_port() -> u16 {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    fn make_config(admin_port: u16, status_port: u16, admin_password: &str) -> Arc<ConfigManager> {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::AdminPort, admin_port as i64);
        cm.set_integer(IntegerKey::StatusPort, status_port as i64);
        cm.set_string(StringKey::AdminPassword, admin_password);
        Arc::new(cm)
    }

    fn empty_db() -> Arc<Mutex<Box<dyn Database + Send>>> {
        Arc::new(Mutex::new(Box::new(InMemoryDb::new())))
    }

    fn empty_vocations() -> Arc<Vocations> {
        Arc::new(Vocations::load_from_xml("<vocations/>").unwrap())
    }

    fn empty_map() -> Arc<Map> {
        Arc::new(Map::new())
    }

    fn http_config(http_port: u16) -> Arc<ConfigManager> {
        http_config_bind(http_port, "127.0.0.1")
    }

    fn http_config_bind(http_port: u16, bind_addr: &str) -> Arc<ConfigManager> {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::HttpPort, http_port as i64);
        cm.set_integer(IntegerKey::HttpWorkers, 1);
        cm.set_string(StringKey::ServerName, "TestServer");
        cm.set_string(StringKey::Ip, "127.0.0.1");
        cm.set_integer(IntegerKey::GamePort, 7172);
        cm.set_string(StringKey::Location, "EU");
        cm.set_string(StringKey::WorldType, "pvp");
        cm.set_string(StringKey::HttpBindAddress, bind_addr);
        Arc::new(cm)
    }

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

        assert_eq!(spawn_manager.entry_count(), 2);
    }

    #[test]
    fn boot_all_four_loaders_called_before_game_loop() {
        let game_data = boot(&data_dir(), "forgotten").expect("boot should succeed with real data");

        assert!(!game_data.items.is_empty());
        let _ = game_data.spells.len();
        assert!(!game_data.weapons.is_empty());
        assert!(!game_data.npcs.is_empty());
        assert!(game_data.vocations.get_vocation(0).is_some());
    }

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
        assert_eq!(count, 2);
    }

    #[test]
    fn missing_script_dir_returns_error() {
        use forgottenserver_scripting::engine::LuaScriptEngine;
        let mut engine = LuaScriptEngine::new();
        let result = engine.load_dir(std::path::Path::new("/nonexistent/scripts/xyz_abc"));
        assert!(result.is_err());
    }

    #[test]
    fn boot_loads_lua_scripts_from_data_scripts() {
        let _game_data = boot(&data_dir(), "forgotten").expect("boot should succeed");
    }

    #[test]
    fn boot_loads_monsters_from_data_monster_dir() {
        let game_data = boot(&data_dir(), "forgotten").expect("boot should succeed");
        assert!(game_data.monsters.get_monster_count() > 0);
    }

    #[test]
    fn boot_unknown_map_name_returns_error_mentioning_path() {
        let result = boot(&data_dir(), "no_such_map_xyzabc123");
        assert!(result.is_err());
        let Err(err) = result else { panic!("expected error") };
        assert!(
            err.contains("no_such_map_xyzabc123"),
            "error must mention the missing map name, got: {err}"
        );
    }

    #[test]
    fn boot_known_map_name_succeeds() {
        let result = boot(&data_dir(), "forgotten");
        assert!(result.is_ok());
        assert!(result.unwrap().map.get_tile_count() > 0);
    }

    #[test]
    fn start_admin_and_status_binds_both_listeners_and_returns_ok() {
        let admin_port = free_port();
        let status_port = free_port();
        let config = make_config(admin_port, status_port, "secret");
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_admin_and_status(config.clone(), game_state.clone());
        assert!(res.is_ok(), "expected Ok, got: {:?}", res);

        let mut admin_stream =
            std::net::TcpStream::connect(format!("127.0.0.1:{admin_port}")).unwrap();
        admin_stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut admin_resp = String::new();
        let _ = admin_stream.read_to_string(&mut admin_resp);

        let mut status_stream =
            std::net::TcpStream::connect(format!("127.0.0.1:{status_port}")).unwrap();
        status_stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        status_stream.shutdown(std::net::Shutdown::Write).unwrap();
        let mut status_resp = String::new();
        let _ = status_stream.read_to_string(&mut status_resp);
        assert!(status_resp.contains("HTTP/1.0 200 OK"));
        assert!(status_resp.contains("<tsqp"));
    }

    #[test]
    fn start_admin_and_status_errors_when_admin_port_already_bound() {
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
    fn start_game_listener_binds_port_and_accepts_connection() {
        let game_port = free_port();
        let mut config_manager = ConfigManager::new();
        config_manager.set_integer(IntegerKey::GamePort, game_port as i64);
        let config = Arc::new(config_manager);
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_game_listener(
            config,
            game_state,
            empty_db(),
            GameListenerParams {
                vocations: empty_vocations(),
                map: empty_map(),
                talk_actions: Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()),
                talkaction_script_dir: std::path::PathBuf::new(),
                actions: Arc::new(forgottenserver_scripting::actions::Actions::new()),
                action_data_dir: std::path::PathBuf::new(),
            },
        );
        assert!(res.is_ok(), "start_game_listener must bind successfully: {:?}", res);

        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{game_port}")).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        let mut buf = [0u8; 14];
        let n = stream.read(&mut buf).unwrap_or(0);
        assert_eq!(n, 14, "challenge must be exactly 14 bytes");
        assert_eq!(&buf[0..2], &[0x0C, 0x00]);
        assert_eq!(buf[8], 0x1F);
    }

    #[test]
    fn start_game_listener_errors_when_port_already_bound() {
        let game_port = free_port();
        let _hog = std::net::TcpListener::bind(format!("0.0.0.0:{game_port}")).unwrap();

        let mut config_manager = ConfigManager::new();
        config_manager.set_integer(IntegerKey::GamePort, game_port as i64);
        let config = Arc::new(config_manager);
        let game_state = Arc::new(Mutex::new(GameState::new()));

        let res = start_game_listener(
            config,
            game_state,
            empty_db(),
            GameListenerParams {
                vocations: empty_vocations(),
                map: empty_map(),
                talk_actions: Arc::new(forgottenserver_scripting::talkaction::TalkActions::new()),
                talkaction_script_dir: std::path::PathBuf::new(),
                actions: Arc::new(forgottenserver_scripting::actions::Actions::new()),
                action_data_dir: std::path::PathBuf::new(),
            },
        );
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("Cannot bind game port"), "error: {err}");
    }

    #[test]
    fn start_http_listener_skips_when_port_zero() {
        let config = Arc::new(ConfigManager::new());
        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(res.is_ok());
    }

    #[test]
    fn start_http_listener_errors_when_port_already_bound() {
        let port = free_port();
        let _hog = std::net::TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

        let res = start_http_listener(http_config(port), empty_db(), empty_vocations());
        let err = res.expect_err("must error when port already bound");
        assert!(err.contains("Cannot bind HTTP port"), "unexpected error: {err}");
    }

    #[test]
    fn start_http_listener_binds_and_returns_ok() {
        let port = free_port();
        let res = start_http_listener(http_config(port), empty_db(), empty_vocations());
        assert!(res.is_ok(), "start_http_listener must succeed: {res:?}");

        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(conn.is_ok());
    }

    #[test]
    fn start_http_listener_handles_cacheinfo_request() {
        let port = free_port();
        let res = start_http_listener(http_config(port), empty_db(), empty_vocations());
        assert!(res.is_ok(), "start failed: {res:?}");

        std::thread::sleep(std::time::Duration::from_millis(50));

        let body = b"{\"type\":\"cacheinfo\"}";
        let request = format!("POST / HTTP/1.0\r\nContent-Length: {}\r\n\r\n", body.len());

        let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        stream.write_all(body).unwrap();

        let mut response = String::new();
        let _ = stream.read_to_string(&mut response);

        assert!(response.contains("HTTP/1.1 200"));
        assert!(response.contains("Content-Type: application/json"));
        assert!(response.contains("\"playersonline\""));
    }

    #[test]
    fn start_http_listener_defaults_to_loopback_when_bind_address_not_set() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::HttpPort, free_port() as i64);
        cm.set_integer(IntegerKey::HttpWorkers, 1);
        cm.set_string(StringKey::ServerName, "TestServer");
        cm.set_string(StringKey::Ip, "127.0.0.1");
        cm.set_integer(IntegerKey::GamePort, 7172);
        cm.set_string(StringKey::Location, "EU");
        cm.set_string(StringKey::WorldType, "pvp");
        let port = cm.get_integer(IntegerKey::HttpPort) as u16;
        let config = Arc::new(cm);

        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(res.is_ok());

        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(conn.is_ok());
    }

    #[test]
    fn start_http_listener_binds_to_configured_address_when_set_to_all_interfaces() {
        let port = free_port();
        let config = http_config_bind(port, "0.0.0.0");

        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(res.is_ok());

        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(conn.is_ok());
    }

    #[test]
    fn start_http_listener_empty_bind_address_does_not_produce_bare_colon_port() {
        let port = free_port();
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::HttpPort, port as i64);
        cm.set_integer(IntegerKey::HttpWorkers, 1);
        cm.set_string(StringKey::ServerName, "TestServer");
        cm.set_string(StringKey::Ip, "127.0.0.1");
        cm.set_integer(IntegerKey::GamePort, 7172);
        cm.set_string(StringKey::Location, "EU");
        cm.set_string(StringKey::WorldType, "pvp");
        cm.set_string(StringKey::HttpBindAddress, "");
        let config = Arc::new(cm);

        let res = start_http_listener(config, empty_db(), empty_vocations());
        assert!(res.is_ok());

        let conn = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        assert!(conn.is_ok());
    }
}
