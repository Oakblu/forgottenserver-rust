# Lua Environment Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the two separate Lua VMs (`LuaBindingsState` + `LuaScriptEngine`) with a single `LuaEnvironment` that installs all TFS bindings and loads all scripts, so game scripts no longer fail with "attempt to call a nil value".

**Architecture:** `LuaEnvironment` (rename of `LuaBindingsState`) owns the sole `mlua::Lua` instance. `install_bindings` is extended to register all class constructors. Script loading moves from `server::boot` into `tfs::boot`, called after bindings are installed. `LuaScriptEngine` remains as a test utility but is off the production boot path.

**Tech Stack:** Rust, mlua 0.9 (lua54 + vendored), forgottenserver-scripting crate, forgottenserver-server crate, forgottenserver-tfs crate.

---

## File Map

| File | Action | Purpose |
|---|---|---|
| `crates/scripting/src/engine.rs` | Modify | Make `collect_lua_files` `pub(crate)`, add `skip_lib: bool` param |
| `crates/scripting/src/lua_bindings/mod.rs` | Modify | Rename `LuaBindingsState` → `LuaEnvironment`; add `load_lib_scripts`, `load_scripts`; extend `install_bindings` with class globals |
| `crates/scripting/src/lua_bindings/classes/config_manager.rs` | Create | `LuaConfigManager` stub UserData |
| `crates/scripting/src/lua_bindings/classes/mod.rs` | Modify | `pub mod config_manager;` |
| `crates/server/src/boot.rs` | Modify | Remove `load_lua_scripts` fn; remove `scripts_loaded` from `GameData` |
| `crates/tfs/src/boot.rs` | Modify | `Modules.lua: Option<LuaEnvironment>`; add `scripts_loaded: usize`; call lib + game script loading |
| `crates/tfs/src/main.rs` | Modify | Read `modules.scripts_loaded` instead of `modules.game_data.scripts_loaded` |

---

## Task 1: Expose `collect_lua_files` as `pub(crate)` with `skip_lib` parameter

**Files:**
- Modify: `crates/scripting/src/engine.rs`

The `collect_lua_files` free function is currently `#[cfg(feature = "lua-scripting")]` private. `LuaEnvironment` (Task 2) needs to call it for both lib loading (skip_lib=false) and script loading (skip_lib=true).

- [ ] **Step 1: Write the failing test for `skip_lib=false`**

Add this test to the `lua_tests` module at the bottom of `engine.rs`, inside `#[cfg(feature = "lua-scripting")] mod lua_tests { ... }`:

```rust
#[test]
fn collect_lua_files_with_skip_lib_false_includes_lib_dir() {
    let dir = tempfile::TempDir::new().unwrap();
    let lib_dir = dir.path().join("lib");
    std::fs::create_dir(&lib_dir).unwrap();
    std::fs::write(lib_dir.join("helpers.lua"), "helpers = true").unwrap();
    std::fs::write(dir.path().join("main.lua"), "main = true").unwrap();

    let mut paths = Vec::new();
    crate::engine::collect_lua_files(dir.path(), &mut paths, false).unwrap();
    assert_eq!(paths.len(), 2, "skip_lib=false must include lib/ files");
}
```

- [ ] **Step 2: Run to confirm it fails**

```bash
cargo test --lib -p forgottenserver-scripting collect_lua_files_with_skip_lib_false
```

Expected: compile error — `collect_lua_files` is not accessible or has wrong signature.

- [ ] **Step 3: Update `collect_lua_files` in `engine.rs`**

Change the function signature from private to `pub(crate)` and add the `skip_lib` parameter. The function currently starts at the line beginning `fn collect_lua_files(`. Replace the entire function:

```rust
pub(crate) fn collect_lua_files(
    dir: &std::path::Path,
    out: &mut Vec<PathBuf>,
    skip_lib: bool,
) -> Result<(), crate::error::ScriptError> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| {
        crate::error::ScriptError::LoadFailed(format!("cannot read directory: {e}"))
    })?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if skip_lib && path.file_name().and_then(|n| n.to_str()) == Some("lib") {
                continue;
            }
            collect_lua_files(&path, out, skip_lib)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("lua") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('#') {
                out.push(path);
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Update `LuaScriptEngine::load_dir` to pass `skip_lib=true`**

Inside `LuaScriptEngine::load_dir`, the call to `collect_lua_files` currently reads:
```rust
let mut paths = Vec::new();
collect_lua_files(dir, &mut paths)?;
```

Change it to:
```rust
let mut paths = Vec::new();
collect_lua_files(dir, &mut paths, true)?;
```

- [ ] **Step 5: Run all `load_dir` and `collect_lua_files` tests**

```bash
cargo test --lib -p forgottenserver-scripting load_dir
cargo test --lib -p forgottenserver-scripting collect_lua_files
```

Expected: all pass.

- [ ] **Step 6: Run full workspace**

```bash
cargo test --lib --workspace
```

Expected: all pass, zero failures.

- [ ] **Step 7: Commit**

```bash
git add crates/scripting/src/engine.rs
git commit -m "refactor(scripting): make collect_lua_files pub(crate) with skip_lib param"
```

---

## Task 2: Rename `LuaBindingsState` → `LuaEnvironment`, add `load_lib_scripts` and `load_scripts`

**Files:**
- Modify: `crates/scripting/src/lua_bindings/mod.rs`

- [ ] **Step 1: Write failing tests**

Add these tests to the `tests` module at the bottom of `crates/scripting/src/lua_bindings/mod.rs`:

```rust
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
    use std::io::Write;
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("a.lua"), "lib_a = true").unwrap();
    std::fs::write(dir.path().join("b.lua"), "lib_b = true").unwrap();

    let mut env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
    let count = env.load_lib_scripts(dir.path()).unwrap();
    assert_eq!(count, 2, "load_lib_scripts must load all .lua files");

    let a: mlua::Value = env.lua.globals().get("lib_a").unwrap();
    assert_eq!(a, mlua::Value::Boolean(true));
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
```

- [ ] **Step 2: Run to confirm they fail**

```bash
cargo test --lib -p forgottenserver-scripting lua_environment_load
```

Expected: compile errors — `LuaEnvironment` doesn't have these methods yet (or doesn't exist yet).

- [ ] **Step 3: Rename `LuaBindingsState` → `LuaEnvironment` and add loading methods**

In `crates/scripting/src/lua_bindings/mod.rs`, replace the entire `LuaBindingsState` struct and impl with:

```rust
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
    /// Non-fatal per file: logs `> [error] <rel-path>: <reason>` and continues.
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
                    eprintln!("> [error] {e}");
                    errors += 1;
                }
            }
        }
        if errors > 0 {
            eprintln!(
                ">> {loaded} Lua scripts loaded ({errors} errors — run with RUST_LOG=debug for details)"
            );
        }
        loaded
    }
}
```

- [ ] **Step 4: Update the existing tests in `mod.rs` that reference `LuaBindingsState`**

In the `tests` module at the bottom of `mod.rs`, find any `LuaBindingsState::new(...)` calls and rename them to `LuaEnvironment::new(...)`.

Also add `use tempfile;` to the test imports if not present — it's already a dev-dependency.

- [ ] **Step 5: Run the new tests**

```bash
cargo test --lib -p forgottenserver-scripting lua_environment_load
```

Expected: all 4 new tests pass.

- [ ] **Step 6: Run full workspace**

```bash
cargo test --lib --workspace
```

Expected: all pass. If `tfs` crate fails because it still references `LuaBindingsState`, fix those references now (change to `LuaEnvironment`) — they'll be fully wired in Task 5.

- [ ] **Step 7: Commit**

```bash
git add crates/scripting/src/lua_bindings/mod.rs
git commit -m "feat(scripting): rename LuaBindingsState → LuaEnvironment; add load_lib_scripts + load_scripts"
```

---

## Task 3: Register class constructors and `configManager` in `install_bindings`

**Files:**
- Create: `crates/scripting/src/lua_bindings/classes/config_manager.rs`
- Modify: `crates/scripting/src/lua_bindings/classes/mod.rs`
- Modify: `crates/scripting/src/lua_bindings/mod.rs`

Scripts call `Spell()`, `Combat()`, `TalkAction()`, etc. as constructors. Each needs to be a callable Lua function that returns a UserData. The UserData types already exist in `classes/` — this task wires them as globals.

- [ ] **Step 1: Write the failing tests**

Add these tests to the `tests` module in `crates/scripting/src/lua_bindings/mod.rs`:

```rust
#[test]
fn lua_environment_new_installs_class_globals() {
    let env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
    for name in &[
        "Spell", "Combat", "TalkAction", "Action", "Condition",
        "CreatureEvent", "GlobalEvent", "MoveEvent", "XMLDocument",
        "configManager",
    ] {
        let val: mlua::Value = env.lua.globals().get(*name).unwrap();
        assert!(
            !matches!(val, mlua::Value::Nil),
            "expected {name} to be registered, got Nil"
        );
    }
}

#[test]
fn registered_class_constructors_are_callable() {
    let env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
    for (name, code) in &[
        ("Combat",        "local c = Combat()"),
        ("Action",        "local a = Action()"),
        ("TalkAction",    "local t = TalkAction()"),
        ("Spell",         "local s = Spell()"),
        ("Condition",     "local c = Condition(0)"),
        ("CreatureEvent", "local ce = CreatureEvent()"),
        ("GlobalEvent",   "local ge = GlobalEvent()"),
        ("MoveEvent",     "local me = MoveEvent()"),
        ("XMLDocument",   "local x = XMLDocument()"),
    ] {
        env.lua
            .load(*code)
            .exec()
            .unwrap_or_else(|e| panic!("{name}() constructor failed: {e}"));
    }
}

#[test]
fn config_manager_methods_are_callable() {
    let env = LuaEnvironment::new(GameStateHandle::default()).unwrap();
    env.lua.load("local s = configManager:getString(0)").exec().unwrap();
    env.lua.load("local n = configManager:getNumber(0)").exec().unwrap();
    env.lua.load("local b = configManager:getBoolean(0)").exec().unwrap();
}
```

- [ ] **Step 2: Run to confirm they fail**

```bash
cargo test --lib -p forgottenserver-scripting lua_environment_new_installs_class
cargo test --lib -p forgottenserver-scripting registered_class_constructors
cargo test --lib -p forgottenserver-scripting config_manager_methods
```

Expected: failures — globals return Nil.

- [ ] **Step 3: Create `classes/config_manager.rs`**

Create the file `crates/scripting/src/lua_bindings/classes/config_manager.rs`:

```rust
//! `configManager` Lua singleton stub.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

/// Stub for the C++ `ConfigManager` Lua singleton.
/// Returns zero-values for all getters until real config plumbing is wired.
#[derive(Debug, Clone, Copy)]
pub struct LuaConfigManager;

impl UserData for LuaConfigManager {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("getString", |_, _, _key: Value| Ok("".to_string()));
        methods.add_method("getNumber", |_, _, _key: Value| Ok(0i64));
        methods.add_method("getBoolean", |_, _, _key: Value| Ok(false));
        methods.add_method("getFloat", |_, _, _key: Value| Ok(0.0f64));
    }
}
```

- [ ] **Step 4: Add `pub mod config_manager;` to `classes/mod.rs`**

In `crates/scripting/src/lua_bindings/classes/mod.rs`, add the line:

```rust
pub mod config_manager;
```

(alphabetically, after `pub mod combat;` and before `pub mod condition;`)

- [ ] **Step 5: Extend `install_bindings` in `mod.rs`**

In `crates/scripting/src/lua_bindings/mod.rs`, add these imports at the top (after existing `use` statements):

```rust
use classes::{
    action::LuaAction,
    combat::LuaCombat,
    condition::LuaCondition,
    config_manager::LuaConfigManager,
    creature_event::LuaCreatureEvent,
    global_event::LuaGlobalEvent,
    move_event::LuaMoveEvent,
    spell::LuaSpell,
    talk_action::LuaTalkAction,
    xml_document::LuaXmlDocument,
};
use forgottenserver_common::enums::ConditionId;
use forgottenserver_items::condition::ConditionBase;
use forgottenserver_game::spells::Spell;
```

Then extend `install_bindings` — add these lines after the existing `lua.globals().set("Game", ...)` line:

```rust
// Class constructors — each returns a new UserData instance.
// Argument(s) accepted but ignored for stubs; scripts that pass a name or
// type constant will not error.
let globals = lua.globals();

globals.set("Combat", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaCombat::new())
})?)?;

globals.set("Action", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaAction::new())
})?)?;

globals.set("TalkAction", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaTalkAction)
})?)?;

globals.set("Spell", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaSpell(Spell::new("", 0, 0, vec![])))
})?)?;

globals.set("Condition", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaCondition(ConditionBase::new(
        ConditionId::Default,
        0,
        -1,
        false,
        0,
        false,
    )))
})?)?;

globals.set("CreatureEvent", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaCreatureEvent::new())
})?)?;

globals.set("GlobalEvent", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaGlobalEvent)
})?)?;

globals.set("MoveEvent", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaMoveEvent::default())
})?)?;

globals.set("XMLDocument", lua.create_function(|_, _: mlua::MultiValue| {
    Ok(LuaXmlDocument)
})?)?;

// Singleton — not a constructor; accessed as configManager:method(key).
globals.set("configManager", LuaConfigManager)?;
```

- [ ] **Step 6: Run the new tests**

```bash
cargo test --lib -p forgottenserver-scripting lua_environment_new_installs_class
cargo test --lib -p forgottenserver-scripting registered_class_constructors
cargo test --lib -p forgottenserver-scripting config_manager_methods
```

Expected: all three pass.

- [ ] **Step 7: Run full workspace**

```bash
cargo test --lib --workspace
```

Expected: all pass.

- [ ] **Step 8: Commit**

```bash
git add crates/scripting/src/lua_bindings/mod.rs \
        crates/scripting/src/lua_bindings/classes/config_manager.rs \
        crates/scripting/src/lua_bindings/classes/mod.rs
git commit -m "feat(scripting): register class constructors + configManager in install_bindings"
```

---

## Task 4: Remove script loading from `server::boot`

**Files:**
- Modify: `crates/server/src/boot.rs`

`GameData::scripts_loaded` and `load_lua_scripts` move out of `server::boot`. Script loading now happens in `tfs::boot` (Task 5).

- [ ] **Step 1: Remove `scripts_loaded` from `GameData` and delete `load_lua_scripts`**

In `crates/server/src/boot.rs`:

1. Remove `scripts_loaded: usize` from `GameData`.
2. Delete the `load_lua_scripts` function entirely.
3. Remove the `engine::LuaScriptEngine` import (line `use forgottenserver_scripting::engine::LuaScriptEngine;`).
4. In the `boot` function, remove the `let scripts_loaded = load_lua_scripts(data_dir);` line and remove `scripts_loaded` from the `Ok(GameData { ... })` struct literal.

After the edit, `GameData` looks like:

```rust
pub struct GameData {
    pub items: ItemsRegistry,
    pub spells: SpellRegistry,
    pub weapons: WeaponRegistry,
    pub npcs: NpcRegistry,
}
```

And `boot` returns:

```rust
Ok(GameData {
    items,
    spells,
    weapons,
    npcs,
})
```

- [ ] **Step 2: Fix the test that references `scripts_loaded`**

In the `tests` module at the bottom of `crates/server/src/boot.rs`, find the test `boot_loads_lua_scripts_from_data_scripts`. Delete it entirely — this behaviour moves to the tfs crate (Task 5).

Also find `boot_all_four_loaders_called_before_game_loop` and remove any assertion or reference to `game_data.scripts_loaded`.

- [ ] **Step 3: Compile check**

```bash
cargo build -p forgottenserver-server 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 4: Run server crate tests**

```bash
cargo test --lib -p forgottenserver-server
```

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/server/src/boot.rs
git commit -m "refactor(server): remove load_lua_scripts and scripts_loaded from GameData"
```

---

## Task 5: Wire `LuaEnvironment` into `tfs::boot` and `main.rs`

**Files:**
- Modify: `crates/tfs/src/boot.rs`
- Modify: `crates/tfs/src/main.rs`

- [ ] **Step 1: Update `Modules` struct and `initialise_modules` in `tfs/src/boot.rs`**

Replace the `Modules` struct to use `LuaEnvironment` and add `scripts_loaded`:

```rust
pub struct Modules {
    pub config: Arc<ConfigManager>,
    pub game_state: Arc<Mutex<GameState>>,
    pub game_data: srv_boot::GameData,
    pub scripts_loaded: usize,
    #[cfg(feature = "lua-scripting")]
    pub lua: Option<forgottenserver_scripting::lua_bindings::LuaEnvironment>,
}
```

Replace the `#[cfg(feature = "lua-scripting")]` block inside `initialise_modules` (the part that creates the Lua state) with:

```rust
#[cfg(feature = "lua-scripting")]
let (lua, scripts_loaded) = {
    use forgottenserver_scripting::lua_bindings::{GameStateHandle, LuaEnvironment};

    match LuaEnvironment::new(GameStateHandle::default()) {
        Ok(mut env) => {
            // Load lib scripts first (fatal on failure — mirrors C++ loadScriptSystems)
            let lib_dir = data_dir.join("scripts").join("lib");
            if lib_dir.exists() {
                if let Err(e) = env.load_lib_scripts(&lib_dir) {
                    eprintln!("{e}");
                    return Err(anyhow!("Lua lib load failed"));
                }
            }
            // Load game scripts (non-fatal per file)
            let scripts_dir = data_dir.join("scripts");
            let count = env.load_scripts(&scripts_dir, data_dir);
            (Some(env), count)
        }
        Err(e) => {
            eprintln!("[WARN] Failed to install Lua bindings: {e}");
            (None, 0)
        }
    }
};

#[cfg(not(feature = "lua-scripting"))]
let (lua, scripts_loaded) = ((), 0usize);
```

Update the `Ok(Modules { ... })` return to include the new fields:

```rust
Ok(Modules {
    config: Arc::new(config),
    game_state,
    game_data,
    scripts_loaded,
    #[cfg(feature = "lua-scripting")]
    lua,
})
```

Also remove the old standalone `lua` variable declaration for `LuaBindingsState` that existed before — the new block above replaces it entirely.

- [ ] **Step 2: Update `main.rs` to read `modules.scripts_loaded`**

In `crates/tfs/src/main.rs`, the startup message currently reads:

```rust
modules.game_data.scripts_loaded
```

Change it to:

```rust
modules.scripts_loaded
```

- [ ] **Step 3: Compile check**

```bash
cargo build -p tfs 2>&1 | grep "^error"
```

Expected: no errors. Fix any type mismatches.

- [ ] **Step 4: Run tfs crate tests**

```bash
cargo test --lib -p tfs
```

Expected: all pass.

- [ ] **Step 5: Run full workspace**

```bash
cargo test --lib --workspace
```

Expected: all pass, zero failures.

- [ ] **Step 6: Commit**

```bash
git add crates/tfs/src/boot.rs crates/tfs/src/main.rs
git commit -m "feat(tfs): wire LuaEnvironment into boot; load lib + game scripts after bindings"
```

---

## Task 6: Verify and rebuild Docker

- [ ] **Step 1: Run full workspace one final time**

```bash
cargo test --lib --workspace 2>&1 | grep -E "^test result"
```

Expected: every crate shows `ok. N passed; 0 failed`.

- [ ] **Step 2: Run clippy**

```bash
cargo clippy --workspace --lib --tests -- -D warnings 2>&1 | grep "^error"
```

Expected: no errors.

- [ ] **Step 3: Rebuild and restart Docker**

```bash
docker compose build && docker compose up -d
```

- [ ] **Step 4: Verify startup log**

```bash
docker compose logs server 2>&1 | grep -E "Loaded|error|FATAL|WARN"
```

Expected output (exact counts will vary):
```
>> N Lua scripts loaded (M errors — run with RUST_LOG=debug for details)
>> Loaded 38284 items, 0 spells, 316 weapons, 9 NPCs, N Lua scripts.
```

Scripts that still fail will log `> [error] scripts/path/file.lua:line: reason` — those are expected until the full API is implemented (PacketHandler, Event, etc. are deferred per the spec).

- [ ] **Step 5: Confirm no regression on items/weapons/NPCs count**

The counts for items (38284), weapons (316), and NPCs (9) must be unchanged.

- [ ] **Step 6: Commit if any formatting fixes were needed**

```bash
cargo fmt --all
git add -p
git commit -m "chore: fmt after lua environment consolidation"
```
