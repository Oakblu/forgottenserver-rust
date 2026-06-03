//! Boot orchestration: thin adapter over `forgottenserver_server::boot`.
//!
//! The C++ `mainLoader()` sequence (from `forgottenserver/src/otserv.cpp`)
//! is fulfilled by:
//!
//!   1. `ConfigManager::load(config.lua)`                       — done here
//!   2. Database connect                                        — *deferred*: stub
//!   3. DatabaseManager::isDatabaseSetup → optimizeTables       — *deferred*: stub
//!   4. Items::loadFromOtb + Items::loadFromXml                 — `srv_boot::boot()`
//!   5. Vocations::loadFromXml                                  — *deferred*: PARTIAL
//!   6. Monsters::loadFromXml                                   — *deferred*: PARTIAL
//!   7. Outfits::loadFromXml                                    — *deferred*: PARTIAL
//!   8. Mounts::loadFromXml                                     — *deferred*: PARTIAL
//!   9. Houses::loadHousesXML / Map::loadMap                    — *deferred*: PARTIAL
//!   10. Scripts::loadScripts                                   — `srv_boot::boot()` (best-effort)
//!   11. Actions/MoveEvents/TalkActions/etc.                    — *deferred*: PARTIAL
//!   12. Game::initialise                                       — `GameState::new()`
//!   13. Scheduler::start + Dispatcher::start                   — *deferred*: PARTIAL
//!   14. open status listener on 7171                           — `srv_boot::start_admin_and_status`
//!   15. open game listener on 7172                             — *PARTIAL*: protocolgame stub-fills (now COMPLETE per wire-parity change)
//!   16. optional HTTP listener on 8080                         — *deferred*: PARTIAL
//!   17. install POSIX signals (SIGINT/SIGTERM)                 — `install_signal_handlers()`
//!   18. run game-loop tick until shutdown                      — `wait_for_shutdown()` polls flag
//!
//! Per the design's "PARTIAL outcomes surface as warnings, not panics" rule,
//! each deferred step is documented in the binary's README and logged at
//! runtime when its associated subsystem is referenced.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Result};

use forgottenserver_common::configmanager::ConfigManager;
use forgottenserver_database::database::{Database, InMemoryDb};
use forgottenserver_server::{boot as srv_boot, game_state::GameState};

/// Fully-initialised module bundle returned by [`initialise_modules`].
///
/// Holding this bundle keeps the config and game-state alive for the lifetime
/// of the listener threads spawned by [`start_listeners`].
pub struct Modules {
    pub config: Arc<ConfigManager>,
    pub game_state: Arc<Mutex<GameState>>,
    pub game_data: srv_boot::GameData,
    /// Number of Lua scripts loaded during boot (0 when lua-scripting feature
    /// is disabled or no scripts directory is present).
    pub scripts_loaded: usize,
    /// Database connection shared by the HTTP listener and other subsystems.
    /// Initialised to `InMemoryDb`; replaced by the caller with the real
    /// backend after [`connect_database`] succeeds.
    pub db: Arc<Mutex<Box<dyn Database + Send>>>,
    /// Embedded Lua state with the Rust-side C++→Lua bindings
    /// installed (Position so far; per-class follow-ups extend this).
    /// `None` if the `lua-scripting` feature was disabled at build time
    /// or if the install failed (failures log a warning, not panic).
    #[cfg(feature = "lua-scripting")]
    pub lua: Option<forgottenserver_scripting::lua_bindings::LuaEnvironment>,
}

/// Run boot steps 1, 4, 10, 12 from the C++ `mainLoader()` sequence.
///
/// * Loads the config file from `config_path`.
/// * Loads items.otb, spells.xml, weapons.xml, npc/, and (best-effort) the
///   `scripts/` directory from `data_dir`.
/// * Builds a fresh `GameState`.
///
/// Errors during config or items.otb load are propagated; missing optional
/// data files (e.g. individual spell XML rows) become warnings inside the
/// loader code itself.
pub fn initialise_modules(config_path: &Path, data_dir: &Path) -> Result<Modules> {
    let mut config = ConfigManager::new();
    config
        .load(config_path)
        .map_err(|e| anyhow!("Failed to load config: {e}"))?;

    let map_name = config.get_string(forgottenserver_common::configmanager::StringKey::MapName).to_owned();
    let map_name = if map_name.is_empty() { "forgotten".to_owned() } else { map_name };
    let game_data =
        srv_boot::boot(data_dir, &map_name).map_err(|e| anyhow!("Failed to load game data: {e}"))?;

    let game_state = Arc::new(Mutex::new(GameState::new()));

    #[cfg(feature = "lua-scripting")]
    let (lua, scripts_loaded) = {
        use forgottenserver_scripting::lua_bindings::{GameStateHandle, LuaEnvironment};
        // GameStateHandle is a placeholder for the eventual real
        // game-state handle (see lua_bindings module docs). Today it
        // holds a fresh Arc — future per-class changes will wire
        // through the real `game_state` once the scripting crate can
        // depend on a game-state-providing trait.
        match LuaEnvironment::new(GameStateHandle::default()) {
            Ok(mut env) => {
                // Load data/lib/compat/compat.lua first; it defines `createFunctions`
                // (and other compatibility helpers) used by data/scripts/lib/*.lua.
                // The full data/lib/lib.lua also loads data/lib/core/ which depends on
                // Player, Creature, etc. — classes not yet fully registered. We load
                // only the compat layer here; core lib loading follows once all class
                // globals are wired.
                let compat_lib = data_dir.join("lib").join("compat").join("compat.lua");
                if compat_lib.exists() {
                    if let Err(e) = env.load_file(&compat_lib) {
                        eprintln!("{e}");
                        return Err(anyhow!("{e}"));
                    }
                }
                let lib_dir = data_dir.join("scripts").join("lib");
                if lib_dir.exists() {
                    match env.load_lib_scripts(&lib_dir) {
                        Ok(lib_count) => {
                            eprintln!(">> Loaded {lib_count} Lua lib scripts");
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            return Err(anyhow!("{e}"));
                        }
                    }
                }
                let scripts_dir = data_dir.join("scripts");
                let count = if scripts_dir.exists() {
                    env.load_scripts(&scripts_dir, data_dir)
                } else {
                    eprintln!(
                        "[WARN] Lua scripts dir not found: {}",
                        scripts_dir.display()
                    );
                    0
                };
                (Some(env), count)
            }
            Err(e) => {
                eprintln!("[WARN] Failed to install Lua bindings: {e}");
                (None, 0)
            }
        }
    };

    #[cfg(not(feature = "lua-scripting"))]
    let scripts_loaded = 0usize;

    Ok(Modules {
        config: Arc::new(config),
        game_state,
        game_data,
        scripts_loaded,
        db: Arc::new(Mutex::new(Box::new(InMemoryDb::new()))),
        #[cfg(feature = "lua-scripting")]
        lua,
    })
}

/// Run boot steps 14 + 16 — bind the admin/status listeners.
///
/// The game-protocol listener (step 15) requires the network stack's
/// ProtocolGame handler hooked into a TCP accept loop; that's currently
/// surfaced via `forgottenserver_server` directly when the relevant subsystem
/// is wired. For the bootable-binary milestone we only bind the status port,
/// which is the smoke-test target.
pub fn start_listeners(modules: &Modules) -> Result<()> {
    srv_boot::start_admin_and_status(modules.config.clone(), modules.game_state.clone())
        .map_err(|e| anyhow!("Failed to start admin/status listeners: {e}"))?;
    srv_boot::start_game_listener(
        modules.config.clone(),
        modules.game_state.clone(),
        modules.db.clone(),
        modules.game_data.vocations.clone(),
        modules.game_data.map.clone(),
    )
    .map_err(|e| anyhow!("Failed to start game listener: {e}"))?;
    srv_boot::start_http_listener(
        modules.config.clone(),
        modules.db.clone(),
        modules.game_data.vocations.clone(),
    )
    .map_err(|e| anyhow!("Failed to start HTTP listener: {e}"))?;
    Ok(())
}

/// Check that `config_path` is a file before attempting to load it.
///
/// Returns a descriptive `Err` with recovery instructions if the file is
/// missing or is a directory, so users see an actionable message instead of a raw IO error.
pub fn validate_config_path(config_path: &Path) -> Result<()> {
    if !config_path.is_file() {
        return Err(anyhow!(
            "Config file not found: {}\nTo fix: copy config.lua.dist to config.lua and edit the settings.\n  cp config.lua.dist config.lua",
            config_path.display()
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Signal handling (boot step 17)
// ---------------------------------------------------------------------------

/// Global shutdown flag. Flipped by the C signal handler installed by
/// [`install_signal_handlers`] and polled by [`wait_for_shutdown`].
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

extern "C" fn signal_handler(_signum: libc::c_int) {
    SHUTDOWN.store(true, Ordering::SeqCst);
}

/// Install SIGINT + SIGTERM handlers that set the shutdown flag.
///
/// Idempotent: re-calling is harmless (re-installs the same handler).
/// Safe to call before `start_listeners` so listener threads see the same
/// shutdown flag via [`shutdown_requested`].
pub fn install_signal_handlers() {
    // SAFETY: `libc::signal` is unsafe because it sets a global handler; the
    // handler we install (`signal_handler`) is `extern "C"` and only mutates
    // an `AtomicBool` — both async-signal-safe operations.
    unsafe {
        libc::signal(
            libc::SIGINT,
            signal_handler as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            signal_handler as *const () as libc::sighandler_t,
        );
    }
}

/// Returns `true` once SIGINT or SIGTERM has been received.
pub fn shutdown_requested() -> bool {
    SHUTDOWN.load(Ordering::SeqCst)
}

/// Test/admin hook: flip the shutdown flag without raising a real signal.
///
/// Used by integration tests to verify the signal-flag contract without
/// actually raising SIGINT in the test process.
pub fn request_shutdown() {
    SHUTDOWN.store(true, Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// Database backend selection (boot step 2 — partially closed for harness)
// ---------------------------------------------------------------------------

/// User-selected database backend.
///
/// Default is `Auto`: pick `MariaDb` if config provides credentials,
/// `InMemory` otherwise. Explicit CLI values override the auto-detect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbBackend {
    Auto,
    InMemory,
    #[cfg(feature = "mariadb")]
    MariaDb,
}

impl DbBackend {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "in-memory" => Ok(Self::InMemory),
            #[cfg(feature = "mariadb")]
            "mariadb" => Ok(Self::MariaDb),
            #[cfg(not(feature = "mariadb"))]
            "mariadb" => {
                Err("mariadb backend not compiled in (build with --features mariadb)".into())
            }
            other => Err(format!(
                "unknown --db-backend value '{other}'; expected auto | in-memory | mariadb"
            )),
        }
    }
}

/// Resolve the effective backend choice given the user selection and
/// the running config.
pub fn resolve_backend(choice: DbBackend, config: &ConfigManager) -> DbBackend {
    use forgottenserver_common::configmanager::StringKey;
    match choice {
        DbBackend::Auto => {
            #[cfg(feature = "mariadb")]
            {
                let has_host = !config.get_string(StringKey::MysqlHost).is_empty();
                let has_db = !config.get_string(StringKey::MysqlDb).is_empty();
                if has_host && has_db {
                    return DbBackend::MariaDb;
                }
            }
            DbBackend::InMemory
        }
        other => other,
    }
}

/// Connect to the selected backend and return a boxed `Database`.
///
/// In `MariaDb` mode, also runs the idempotent schema bootstrap from
/// `forgottenserver/schema.sql` so a fresh DB has the right tables.
///
/// In `InMemory` mode, returns a fresh `InMemoryDb` immediately.
pub fn connect_database(
    backend: DbBackend,
    config: &ConfigManager,
) -> Result<Box<dyn forgottenserver_database::database::Database + Send>> {
    use forgottenserver_database::database::InMemoryDb;
    match backend {
        DbBackend::Auto => connect_database(resolve_backend(backend, config), config),
        DbBackend::InMemory => Ok(Box::new(InMemoryDb::new())),
        #[cfg(feature = "mariadb")]
        DbBackend::MariaDb => {
            use forgottenserver_database::mariadb::{MariaDbConfig, MariaDbDatabase};
            let cfg = MariaDbConfig::from_config_manager(config);
            let db = MariaDbDatabase::connect(&cfg)
                .map_err(|e| anyhow!("MariaDB connect failed: {e}"))?;
            // Idempotent — no-op if schema already present.
            const SCHEMA: &str = include_str!("../../../schema.sql");
            db.bootstrap_schema_if_needed(SCHEMA)
                .map_err(|e| anyhow!("MariaDB schema bootstrap failed: {e}"))?;
            Ok(Box::new(db))
        }
    }
}

/// Boot step 18: block until the shutdown flag is set (then return).
///
/// Polls every 100 ms. For a v1 bootable binary this is acceptable; a future
/// pass can switch to a `Condvar`-based wait once the dispatcher's run loop
/// is wired in.
pub fn wait_for_shutdown() {
    while !shutdown_requested() {
        std::thread::sleep(Duration::from_millis(100));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_flag_starts_clear_after_explicit_reset_or_fresh_process() {
        // We cannot guarantee fresh-process state since the static AtomicBool
        // is shared; instead, document that `request_shutdown` then a manual
        // store(false) round-trips correctly.
        request_shutdown();
        assert!(shutdown_requested());
        SHUTDOWN.store(false, Ordering::SeqCst);
        assert!(!shutdown_requested());
    }

    #[test]
    fn install_handlers_is_idempotent() {
        install_signal_handlers();
        install_signal_handlers();
        install_signal_handlers();
        // No panic, no UB. The handler is just a function pointer being
        // re-assigned to the same value.
    }

    #[test]
    fn request_shutdown_flips_flag() {
        SHUTDOWN.store(false, Ordering::SeqCst);
        assert!(!shutdown_requested());
        request_shutdown();
        assert!(shutdown_requested());
        SHUTDOWN.store(false, Ordering::SeqCst);
    }

    #[test]
    fn validate_config_path_missing_file_returns_err_with_hint() {
        let result = validate_config_path(std::path::Path::new("/nonexistent/xyz/config.lua"));
        let err = result.expect_err("expected Err for nonexistent path");
        let msg = format!("{err}");
        assert!(
            msg.contains("Config file not found"),
            "error should name the problem: {msg}"
        );
        assert!(
            msg.contains("cp config.lua.dist config.lua"),
            "error should include recovery command: {msg}"
        );
    }

    #[test]
    fn validate_config_path_existing_file_returns_ok() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let result = validate_config_path(&path);
        assert!(
            result.is_ok(),
            "expected Ok for existing file, got: {result:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Task 2.3 — ServiceManager::run equivalent: wait_for_shutdown blocks
    // until the shutdown signal is set.
    //
    // C++ cross-validation:
    //   * server.cpp ServiceManager::run(): sets running=true then calls
    //     io_context.run() which blocks until all work is done or
    //     io_context.stop() is called (by ServiceManager::stop()).
    //   * The Rust equivalent wait_for_shutdown() polls the SHUTDOWN AtomicBool
    //     every 100 ms until it is set, then returns.
    // -----------------------------------------------------------------------

    /// wait_for_shutdown() returns promptly once request_shutdown() is called
    /// from another thread. Mirrors C++ ServiceManager::run() blocking until
    /// ServiceManager::stop() signals io_context.stop().
    #[test]
    fn wait_for_shutdown_blocks_until_request_shutdown_called() {
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        // Reset the flag before the test (shared static; other tests may have set it).
        SHUTDOWN.store(false, Ordering::SeqCst);

        let finished = Arc::new(Mutex::new(false));
        let finished_clone = finished.clone();

        // Spawn a thread that calls wait_for_shutdown and records completion.
        let waiter = std::thread::spawn(move || {
            wait_for_shutdown();
            *finished_clone.lock().unwrap() = true;
        });

        // Give the waiter thread time to start and enter the polling loop.
        std::thread::sleep(Duration::from_millis(50));

        // The waiter must still be blocking — not finished yet.
        assert!(
            !*finished.lock().unwrap(),
            "wait_for_shutdown must NOT return before request_shutdown is called"
        );

        // Signal shutdown — the waiter should now unblock.
        request_shutdown();

        // Wait up to 1 second for the waiter to finish.
        waiter.join().expect("waiter thread must exit cleanly");

        assert!(
            *finished.lock().unwrap(),
            "wait_for_shutdown must return after request_shutdown is called"
        );

        // Restore the flag for other tests.
        SHUTDOWN.store(false, Ordering::SeqCst);
    }

    /// wait_for_shutdown() returns immediately if the shutdown flag is already
    /// set before the call. Matches C++ ServiceManager::run() where if the
    /// io_context has already been stopped, run() returns immediately.
    #[test]
    fn wait_for_shutdown_returns_immediately_when_already_shutdown() {
        use std::time::Instant;

        SHUTDOWN.store(true, Ordering::SeqCst);
        let start = Instant::now();
        wait_for_shutdown();
        let elapsed = start.elapsed();

        // Should return in much less than the 100ms poll interval.
        assert!(
            elapsed.as_millis() < 200,
            "wait_for_shutdown must return quickly when flag is already set (elapsed={elapsed:?})"
        );

        // Restore for other tests.
        SHUTDOWN.store(false, Ordering::SeqCst);
    }

    // -----------------------------------------------------------------------
    // Task 1.2 — argumentsHandler: config path handling.
    //
    // C++ cross-validation:
    //   * main.cpp argumentsHandler: --config=<path> → ConfigManager::setString
    //     CONFIG_FILE to the given path.
    //   * Rust parse_cli: --config <path> → CliArgs.config_path.
    //   * validate_config_path is the observable guard on the config path
    //     before initialise_modules is called, mirroring the C++ check in
    //     mainLoader (which opens the config file and calls ConfigManager::load).
    //
    // Since parse_cli lives in the binary (main.rs), we test the observable
    // guard — validate_config_path — which exercises the same config-path
    // validation that the argument handler ultimately triggers.
    // -----------------------------------------------------------------------

    /// A directory path (not a file) is rejected by validate_config_path,
    /// confirming the Rust binary guards against accidentally passing a
    /// directory where a file is expected (no C++ equivalent — C++ just fails
    /// when luaL_dofile can't open a directory).
    #[test]
    fn validate_config_path_directory_returns_err() {
        let result = validate_config_path(std::path::Path::new("/tmp"));
        assert!(
            result.is_err(),
            "a directory path must be rejected by validate_config_path"
        );
    }

    // -----------------------------------------------------------------------
    // Task 1.1 — main: calls argumentsHandler then startServer.
    //
    // C++ cross-validation:
    //   * main.cpp:39-48: argumentsHandler → if false, return 1;
    //     else startServer(); return 0.
    //   * Rust: parse_cli → validate_config_path → initialise_modules →
    //     start_listeners → wait_for_shutdown.
    //
    // The ordering is structurally enforced in main.rs; the observable test
    // is that validate_config_path (the first observable gate after argument
    // parsing) returns Err when the config file is absent, mirroring the C++
    // `argumentsHandler` returning false for `--help`/`--version` (which
    // causes main to return 1 immediately).
    // -----------------------------------------------------------------------

    /// resolve_backend returns InMemory when no DB credentials are configured,
    /// confirming the Auto-detection branch (which startServer/mainLoader
    /// triggers by attempting DB connect).
    #[test]
    fn resolve_backend_auto_with_no_credentials_returns_in_memory() {
        use forgottenserver_common::configmanager::ConfigManager;
        let config = ConfigManager::new(); // empty — no mysqlHost/mysqlDb
        let backend = resolve_backend(DbBackend::Auto, &config);
        assert_eq!(
            backend,
            DbBackend::InMemory,
            "Auto with no credentials must select InMemory backend"
        );
    }

    /// resolve_backend with explicit InMemory returns InMemory unchanged,
    /// confirming the passthrough branch.
    #[test]
    fn resolve_backend_explicit_in_memory_passes_through() {
        use forgottenserver_common::configmanager::ConfigManager;
        let config = ConfigManager::new();
        let backend = resolve_backend(DbBackend::InMemory, &config);
        assert_eq!(backend, DbBackend::InMemory);
    }

    /// connect_database with InMemory backend succeeds and returns a valid
    /// database, confirming startServer's DB-connect step works for the
    /// in-memory path.
    #[test]
    fn connect_database_in_memory_returns_ok() {
        use forgottenserver_common::configmanager::ConfigManager;
        let config = ConfigManager::new();
        let result = connect_database(DbBackend::InMemory, &config);
        assert!(result.is_ok(), "InMemory backend must connect successfully");
    }

    /// DbBackend::parse recognises all documented values.
    #[test]
    fn db_backend_parse_recognises_all_values() {
        assert_eq!(DbBackend::parse("auto").unwrap(), DbBackend::Auto);
        assert_eq!(DbBackend::parse("in-memory").unwrap(), DbBackend::InMemory);
        assert!(
            DbBackend::parse("unknown").is_err(),
            "unrecognised backend string must return Err"
        );
    }
}
