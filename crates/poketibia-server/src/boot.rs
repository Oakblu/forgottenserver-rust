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
use forgottenserver_server::{boot as srv_boot, game_state::GameState};

/// Fully-initialised module bundle returned by [`initialise_modules`].
///
/// Holding this bundle keeps the config and game-state alive for the lifetime
/// of the listener threads spawned by [`start_listeners`].
pub struct Modules {
    pub config: Arc<ConfigManager>,
    pub game_state: Arc<Mutex<GameState>>,
    pub game_data: srv_boot::GameData,
    /// Embedded Lua state with the Rust-side C++→Lua bindings
    /// installed (Position so far; per-class follow-ups extend this).
    /// `None` if the `lua-scripting` feature was disabled at build time
    /// or if the install failed (failures log a warning, not panic).
    #[cfg(feature = "lua-scripting")]
    pub lua: Option<forgottenserver_scripting::lua_bindings::LuaBindingsState>,
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

    let game_data =
        srv_boot::boot(data_dir).map_err(|e| anyhow!("Failed to load game data: {e}"))?;

    let game_state = Arc::new(Mutex::new(GameState::new()));

    #[cfg(feature = "lua-scripting")]
    let lua = {
        use forgottenserver_scripting::lua_bindings::{GameStateHandle, LuaBindingsState};
        // GameStateHandle is a placeholder for the eventual real
        // game-state handle (see lua_bindings module docs). Today it
        // holds a fresh Arc — future per-class changes will wire
        // through the real `game_state` once the scripting crate can
        // depend on a game-state-providing trait.
        match LuaBindingsState::new(GameStateHandle::default()) {
            Ok(state) => {
                eprintln!(">> Lua bindings installed (Position + future classes)");
                Some(state)
            }
            Err(e) => {
                eprintln!("[WARN] Failed to install Lua bindings: {e}");
                None
            }
        }
    };

    Ok(Modules {
        config: Arc::new(config),
        game_state,
        game_data,
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
            const SCHEMA: &str = include_str!("../../../../forgottenserver/schema.sql");
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
}
