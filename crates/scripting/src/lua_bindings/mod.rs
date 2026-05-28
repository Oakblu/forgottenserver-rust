//! Lua-binding install surface.
//!
//! Entry point for every C++→Lua binding the Rust port exposes. Per-class
//! follow-up changes plug into [`install_bindings`] by adding one call per
//! class submodule. Built incrementally, one OpenSpec change per class.
//!
//! ## Architecture
//!
//! ```text
//! tfs::boot::initialise_modules
//!   └─ constructs GameState                       (Arc<Mutex<GameState>>)
//!   └─ constructs LuaEnvironment                  (owns mlua::Lua)
//!        └─ install_bindings(&lua, game_state)
//!             └─ lua.set_app_data(GameStateHandle(game_state.clone()))
//!             └─ position::install(&lua)
//!             └─ // future: creature::install(&lua)
//!             └─ // future: player::install(&lua)
//! ```
//!
//! UserData methods read `lua.app_data_ref::<GameStateHandle>()` to reach
//! the game state. The newtype is required because storing a raw
//! `Arc<Mutex<…>>` as app_data is fragile across crate boundaries (the
//! TypeId can collide and there's no type-name to look up).
//!
//! The engine change ships with `Position` as the reference class. The 989
//! enum constants land via the sibling `forgottenserver-rust-lua-bindings-
//! enums` change; every other class is its own per-class follow-up.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

pub mod classes;
pub mod enums;
pub mod misc_globals;
pub mod position;
pub mod table_enums;

/// Opaque handle stored on the Lua state via `set_app_data`.
///
/// The inner `Arc<Mutex<()>>` is a placeholder for the eventual
/// `Arc<Mutex<GameState>>`. The scripting crate sits at layer 9 and
/// cannot depend on `forgottenserver-server` (layer 11), so concrete
/// game-state types are resolved by the caller. The first per-class
/// change that needs real game state will introduce a trait the
/// caller implements.
///
/// What this newtype proves today: the install pipeline + the
/// `lua.app_data_ref::<GameStateHandle>()` lookup pattern are wired
/// end-to-end. Future bindings that need richer state extend the
/// inner type or store additional handles via `lua.set_named_registry`.
#[derive(Clone)]
pub struct GameStateHandle(pub(crate) Arc<Mutex<()>>);

impl Default for GameStateHandle {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(())))
    }
}

/// The single Lua environment for the server process.
///
/// Mirrors C++'s `LuaEnvironment` / `g_luaEnvironment`: one VM owns all
/// bindings and all loaded scripts. Created once at boot; lives for the
/// process lifetime.
pub struct LuaEnvironment {
    pub lua: mlua::Lua,
}

impl LuaEnvironment {
    /// Construct a fresh Lua state and install every Rust binding.
    pub fn new(game_state: GameStateHandle) -> mlua::Result<Self> {
        let lua = mlua::Lua::new();
        install_bindings(&lua, game_state)?;
        Ok(Self { lua })
    }

    /// Evaluate a Lua snippet — for tests and REPL use.
    pub fn eval<R: for<'lua> mlua::FromLua<'lua>>(&self, code: &str) -> mlua::Result<R> {
        self.lua.load(code).eval()
    }

    /// Execute a single Lua file by path (equivalent to C++ `loadFile`).
    ///
    /// Returns `Err(String)` with `[FATAL]` prefix on read or execution failure.
    pub fn load_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            format!("[FATAL] Failed to read {}: {e}", path.display())
        })?;
        let name = path.to_string_lossy().into_owned();
        self.lua
            .load(&content)
            .set_name(name.as_str())
            .exec()
            .map_err(|e| format!("[FATAL] Failed to load {}: {e}", path.display()))
    }

    /// Load all `.lua` files from `lib_dir` recursively (including nested `lib/` dirs).
    ///
    /// Fatal: returns `Err(String)` with a `[FATAL]` prefix if the directory is
    /// missing or any file fails to load. Matches C++ `loadScripts(isLib=true)`.
    pub fn load_lib_scripts(&mut self, lib_dir: &std::path::Path) -> Result<usize, String> {
        if !lib_dir.exists() {
            return Err(format!(
                "[FATAL] Failed to load Lua libs: directory not found: {}",
                lib_dir.display()
            ));
        }
        let mut paths = Vec::new();
        crate::engine::collect_lua_files(lib_dir, &mut paths, false).map_err(|e| {
            format!("[FATAL] Failed to scan lib dir {}: {e}", lib_dir.display())
        })?;
        paths.sort();
        let mut loaded = 0usize;
        for path in &paths {
            let source = std::fs::read_to_string(path).map_err(|e| {
                format!("[FATAL] Failed to read Lua lib {}: {e}", path.display())
            })?;
            let name = path.to_string_lossy();
            self.lua
                .load(&source)
                .set_name(name.as_ref())
                .exec()
                .map_err(|e| {
                    format!("[FATAL] Failed to load Lua lib {}: {e}", path.display())
                })?;
            loaded += 1;
        }
        Ok(loaded)
    }

    /// Load `.lua` files from `scripts_dir` recursively (skip `lib/`, skip `#`-prefixed, sorted).
    ///
    /// Non-fatal per file: logs `> [error] <reason>` and continues.
    /// Returns count of successfully loaded scripts.
    /// `data_dir` is used to compute relative paths for error messages.
    pub fn load_scripts(
        &mut self,
        scripts_dir: &std::path::Path,
        data_dir: &std::path::Path,
    ) -> usize {
        if !scripts_dir.exists() {
            eprintln!(
                "> [warn] Script directory not found: {}",
                scripts_dir.display()
            );
            return 0;
        }
        let mut paths = Vec::new();
        if let Err(e) = crate::engine::collect_lua_files(scripts_dir, &mut paths, true) {
            eprintln!("> [warn] Failed to scan script dir: {e}");
            return 0;
        }
        paths.sort();
        let mut loaded = 0usize;
        let mut errors = 0usize;
        for path in &paths {
            let rel = path.strip_prefix(data_dir).unwrap_or(path);
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("> [error] {}: {e}", rel.display());
                    errors += 1;
                    continue;
                }
            };
            let name = rel.to_string_lossy();
            match self.lua.load(&source).set_name(name.as_ref()).exec() {
                Ok(()) => loaded += 1,
                Err(e) => {
                    let rel_str = rel.to_string_lossy();
                    let err_str = e.to_string();
                    let (line, reason) = parse_lua_error_location(&err_str);
                    let msg = match line {
                        Some(l) => format!("> [error] {}:{}: {}", rel_str, l, reason),
                        None => format!("> [error] {}: {}", rel_str, reason),
                    };
                    eprintln!("{msg}");
                    errors += 1;
                }
            }
        }
        if errors == 0 {
            eprintln!(">> Loaded {loaded} Lua scripts.");
        } else {
            eprintln!(
                ">> Loaded {loaded} Lua scripts ({errors} errors — run with RUST_LOG=debug for details)"
            );
        }
        loaded
    }
}

/// Parse the line number and reason out of an mlua error string.
///
/// mlua errors look like:
///   `[string "path"]:line: reason`
/// or prefixed with `runtime error: ` / `syntax error: `.
fn parse_lua_error_location(err_str: &str) -> (Option<u32>, String) {
    let stripped = err_str
        .strip_prefix("runtime error: ")
        .or_else(|| err_str.strip_prefix("syntax error: "))
        .unwrap_or(err_str);
    if let Some(colon_pos) = stripped.find("]:") {
        let after_bracket = &stripped[colon_pos + 2..];
        if let Some(next_colon) = after_bracket.find(':') {
            let line_str = &after_bracket[..next_colon];
            if let Ok(line) = line_str.trim().parse::<u32>() {
                let reason = after_bracket[next_colon + 1..].trim().to_string();
                return (Some(line), reason);
            }
        }
    }
    (None, err_str.to_string())
}

/// Install every Rust-side Lua binding onto the supplied state.
///
/// Idempotent within one Lua state — calling twice re-registers the
/// same globals (later wins). Per-class follow-up changes add their
/// `install` call here.
pub fn install_bindings(lua: &mlua::Lua, game_state: GameStateHandle) -> mlua::Result<()> {
    lua.set_app_data(game_state);
    position::install(lua)?;
    enums::install_global_enums(lua)?;
    table_enums::install_table_enums(lua)?;
    misc_globals::install(lua)?;
    // Class globals are Lua TABLES with a metatable that provides:
    //   __call     → creates a new UserData instance (so `Combat()` works)
    //   __newindex → rawset into the table so assignments like
    //                `Game.startRaid = fn` or `Combat.setCondition = fn`
    //                (from compat.lua) are actually stored and callable later.
    // This mirrors the C++ pattern where each class is a table/metatable pair.
    macro_rules! class_table {
        ($name:expr, $ctor:expr) => {{
            let tbl = lua.create_table()?;
            let mt = lua.create_table()?;
            mt.set("__call", lua.create_function($ctor)?)?;
            mt.set(
                "__newindex",
                lua.create_function(|_, (t, k, v): (mlua::Table, mlua::Value, mlua::Value)| {
                    t.raw_set(k, v)
                })?,
            )?;
            tbl.set_metatable(Some(mt));
            lua.globals().set($name, tbl)?;
        }};
    }

    // Game — singleton namespace; scripts call `Game:method()` via the table.
    // `Game()` is not a valid constructor so __call returns nil.
    class_table!("Game", |_, _: mlua::MultiValue| Ok(mlua::Value::Nil));
    // compat.lua calls Game.getMounts() at module level to build a lookup table.
    {
        let game_tbl: mlua::Table = lua.globals().get("Game")?;
        game_tbl.set("getMounts", lua.create_function(|lua, _: ()| lua.create_table())?)?;
    }

    class_table!("Spell", |_, _: mlua::MultiValue| Ok(
        classes::spell::LuaSpell::default()
    ));
    class_table!("Combat", |_, _: mlua::MultiValue| Ok(
        classes::combat::LuaCombat::default()
    ));
    class_table!("TalkAction", |_, _: mlua::MultiValue| Ok(
        classes::talk_action::LuaTalkAction
    ));
    class_table!("Action", |_, _: mlua::MultiValue| Ok(
        classes::action::LuaAction::default()
    ));
    class_table!("Condition", |_, _: mlua::MultiValue| Ok(
        classes::condition::LuaCondition::default()
    ));
    class_table!("CreatureEvent", |_, _: mlua::MultiValue| Ok(
        classes::creature_event::LuaCreatureEvent::default()
    ));
    class_table!("GlobalEvent", |_, _: mlua::MultiValue| Ok(
        classes::global_event::LuaGlobalEvent
    ));
    class_table!("MoveEvent", |_, _: mlua::MultiValue| Ok(
        classes::move_event::LuaMoveEvent::default()
    ));
    class_table!("XMLDocument", |_, _: mlua::MultiValue| Ok(
        classes::xml_document::LuaXmlDocument
    ));

    // configManager — singleton instance (lowercase, matches C++ g_config Lua name)
    lua.globals()
        .set("configManager", classes::config_manager::LuaConfigManager)?;

    // result — stub table for database result compat aliases used by compat.lua
    // (e.g. `result.getDataInt = result.getNumber`). Real methods are nil stubs.
    lua.globals().set("result", lua.create_table()?)?;

    // rawgetmetatable — C++ TFS custom global; returns the raw metatable of a
    // named class so compat.lua can extend it (e.g. `rawgetmetatable("Player").__index = fn`).
    // We return a throw-away table so assignments are silently discarded.
    lua.globals().set(
        "rawgetmetatable",
        lua.create_function(|lua, _: mlua::Value| lua.create_table())?,
    )?;

    // Stub class tables for entity/item globals used by compat.lua and scripts/lib.
    // compat.lua extends these with methods via `function Player:foo(...)` — that syntax
    // just sets table fields, which works fine on plain tables. Full UserData constructors
    // follow when each class is fully wired; for now nil-indexing errors are prevented.
    for name in &[
        "Player", "Creature", "Monster", "Npc",
        "Item", "Container", "Teleport", "Podium",
        "Tile", "ItemType", "Vocation",
        "Guild", "Group", "Party", "House",
        "MonsterType", "Weapon",
        // compat.lua module-level: `numberToVariant = Variant`, `Variant.getNumber` etc.
        "Variant",
        // Used by scripts (Town, Loot, MonsterSpell) as constructor/namespace tables
        "Town", "Loot", "MonsterSpell",
    ] {
        lua.globals().set(*name, lua.create_table()?)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_bindings_succeeds_with_default_handle() {
        let lua = mlua::Lua::new();
        let r = install_bindings(&lua, GameStateHandle::default());
        assert!(r.is_ok(), "install failed: {r:?}");
    }

    #[test]
    fn app_data_round_trip_preserves_handle_identity() {
        let lua = mlua::Lua::new();
        let handle = GameStateHandle::default();
        let inner_ptr = Arc::as_ptr(&handle.0) as usize;
        install_bindings(&lua, handle).unwrap();
        let recovered = lua
            .app_data_ref::<GameStateHandle>()
            .expect("app_data should contain GameStateHandle");
        assert_eq!(Arc::as_ptr(&recovered.0) as usize, inner_ptr);
    }

    #[test]
    fn lua_environment_new_makes_position_available() {
        let env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
        // Position constructor should be callable from Lua and the
        // returned value usable on the Rust side via FromLua.
        let pos: crate::lua_bindings::position::LuaPosition = env
            .eval("return Position(100, 200, 7)")
            .expect("Position(100, 200, 7) eval");
        assert_eq!(pos.0.x, 100);
        assert_eq!(pos.0.y, 200);
        assert_eq!(pos.0.z, 7);
    }

    #[test]
    fn install_bindings_is_idempotent() {
        let lua = mlua::Lua::new();
        let handle = GameStateHandle::default();
        install_bindings(&lua, handle.clone()).unwrap();
        // Second install: same global, no error.
        install_bindings(&lua, handle).unwrap();
    }

    #[test]
    fn lua_environment_load_lib_scripts_missing_dir_is_fatal() {
        let mut env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
        let result = env.load_lib_scripts(std::path::Path::new("/nonexistent/lib_xyz_abc"));
        assert!(result.is_err(), "missing lib dir must return Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("[FATAL]"), "error must contain [FATAL]: {msg}");
    }

    #[test]
    fn lua_environment_load_lib_scripts_loads_all_lua_files() {
        let tmp = tempfile::tempdir().unwrap();
        // Root-level lib file
        std::fs::write(tmp.path().join("helpers.lua"), "lib_helpers = true").unwrap();
        // Nested subdir — exercises skip_lib=false recursion
        let sub = tmp.path().join("subdir");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("utils.lua"), "lib_utils = true").unwrap();

        let mut env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
        let count = env.load_lib_scripts(tmp.path()).unwrap();
        assert_eq!(count, 2, "should load both root and nested lib files");
        let h: bool = env.lua.globals().get("lib_helpers").unwrap();
        assert!(h);
        let u: bool = env.lua.globals().get("lib_utils").unwrap();
        assert!(u);
    }

    #[test]
    fn lua_environment_load_scripts_skips_lib_and_hash() {
        let dir = tempfile::TempDir::new().unwrap();
        let lib = dir.path().join("lib");
        std::fs::create_dir(&lib).unwrap();
        std::fs::write(lib.join("helpers.lua"), "lib_helper = true").unwrap();
        std::fs::write(dir.path().join("#disabled.lua"), "disabled = true").unwrap();
        std::fs::write(dir.path().join("active.lua"), "active = true").unwrap();

        let mut env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
        let count = env.load_scripts(dir.path(), dir.path());
        assert_eq!(count, 1, "load_scripts must skip lib/ and #-prefixed files");

        let active: mlua::Value = env.lua.globals().get("active").unwrap();
        assert_eq!(active, mlua::Value::Boolean(true));
        let lib_helper: mlua::Value = env.lua.globals().get("lib_helper").unwrap();
        assert_eq!(lib_helper, mlua::Value::Nil);
    }

    #[test]
    fn lua_environment_load_scripts_error_continues_and_counts() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("good.lua"), "good_loaded = true").unwrap();
        std::fs::write(dir.path().join("bad.lua"), "error('intentional failure')").unwrap();

        let mut env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
        let count = env.load_scripts(dir.path(), dir.path());
        assert_eq!(count, 1, "one good script must be counted despite one bad script");

        let good: mlua::Value = env.lua.globals().get("good_loaded").unwrap();
        assert_eq!(good, mlua::Value::Boolean(true));
    }

    #[test]
    fn lua_environment_new_installs_class_globals() {
        let env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
        for global in &[
            "Spell", "Combat", "TalkAction", "Action", "Condition",
            "CreatureEvent", "GlobalEvent", "MoveEvent", "XMLDocument",
        ] {
            let val: mlua::Value = env.lua.globals().get(*global).unwrap();
            assert!(
                !matches!(val, mlua::Value::Nil),
                "expected global '{}' to be non-nil after install_bindings",
                global
            );
        }
    }

    #[test]
    fn lua_environment_new_installs_config_manager_global() {
        let env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
        let val: mlua::Value = env.lua.globals().get("configManager").unwrap();
        assert!(
            !matches!(val, mlua::Value::Nil),
            "expected global 'configManager' to be non-nil"
        );
    }
}
