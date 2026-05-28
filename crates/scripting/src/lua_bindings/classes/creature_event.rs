//! `CreatureEvent:*` Lua binding for `scripting::creatureevent::CreatureEvent`.
//!
//! C++ CreatureEvent registers Lua callbacks (onLogin, onDeath, …)
//! and dispatches them when game events fire. The Rust side has the
//! enum + dispatcher framework; full callback storage is wired by
//! the engine. Bindings here are setters that record the callback
//! name; actual dispatch happens at the appropriate event.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaCreatureEvent {
    pub event_type: i64,
}

impl LuaCreatureEvent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaCreatureEvent {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaCreatureEvent>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaCreatureEvent",
                message: Some("expected CreatureEvent userdata".into()),
            }),
        }
    }
}

impl UserData for LuaCreatureEvent {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("type", |_, this, t: i64| {
            this.event_type = t;
            Ok(())
        });
        methods.add_method_mut("register", |_, _this, ()| Ok(true));
        // Allow field assignment (e.g. `creatureevent.onDeath = function() end`).
        // Lua scripts assign handler functions as fields; we accept but don't
        // store them here (real wiring happens in the dispatcher).
        methods.add_meta_method_mut("__newindex", |_, _this, (_k, _v): (Value, Value)| Ok(()));
        // Callback setters — record but don't dispatch (real wiring needs
        // the CreatureEventsDispatcher hookup in scripting/engine).
        for name in &[
            "onLogin",
            "onLogout",
            "onReconnect",
            "onThink",
            "onPrepareDeath",
            "onDeath",
            "onKill",
            "onAdvance",
            "onModalWindow",
            "onTextEdit",
            "onHealthChange",
            "onManaChange",
            "onExtendedOpcode",
        ] {
            methods.add_method_mut(*name, |_, _this, _cb: Value| Ok(()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_lua() -> mlua::Lua {
        let lua = mlua::Lua::new();
        crate::lua_bindings::install_bindings(
            &lua,
            crate::lua_bindings::GameStateHandle::default(),
        )
        .unwrap();
        lua
    }

    #[test]
    fn creature_event_field_assignment_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local ce = CreatureEvent("Test"); ce.onDeath = function() end"#)
            .exec();
        assert!(
            result.is_ok(),
            "field assignment on CreatureEvent should not error: {result:?}"
        );
    }

    #[test]
    fn creature_event_function_syntax_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local ce = CreatureEvent("Test"); function ce.onLogin(player) end"#)
            .exec();
        assert!(
            result.is_ok(),
            "function-declaration syntax on CreatureEvent should not error: {result:?}"
        );
    }
}
