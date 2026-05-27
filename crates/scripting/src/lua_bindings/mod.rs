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
//!   └─ constructs LuaBindingsState                (owns mlua::Lua)
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
pub struct GameStateHandle(pub Arc<Mutex<()>>);

impl Default for GameStateHandle {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(())))
    }
}

/// Holds the Lua state for the server. Stored on `Modules` in
/// `tfs::boot` so the binary owns it for the process
/// lifetime.
pub struct LuaBindingsState {
    pub lua: mlua::Lua,
}

impl LuaBindingsState {
    /// Construct a fresh Lua state and install every Rust binding.
    pub fn new(game_state: GameStateHandle) -> mlua::Result<Self> {
        let lua = mlua::Lua::new();
        install_bindings(&lua, game_state)?;
        Ok(Self { lua })
    }

    /// Convenience for tests / scripts: evaluate a snippet and return
    /// the result.
    pub fn eval<R>(&self, code: &str) -> mlua::Result<R>
    where
        R: for<'lua> mlua::FromLuaMulti<'lua>,
    {
        self.lua.load(code).eval()
    }
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
    // Singleton namespace: `Game.*` Lua calls dispatch into LuaGame's UserData
    // methods. Registering the unit struct as a global gives audit-visible
    // `Game:method` entries without needing per-call game-state plumbing.
    lua.globals().set("Game", classes::game::LuaGame)?;
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
    fn lua_bindings_state_new_makes_position_available() {
        let state = LuaBindingsState::new(GameStateHandle::default()).unwrap();
        // Position constructor should be callable from Lua and the
        // returned value usable on the Rust side via FromLua.
        let pos: crate::lua_bindings::position::LuaPosition = state
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
}
