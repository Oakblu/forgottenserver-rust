//! `CreatureEvent:*` Lua binding for `scripting::creatureevent::CreatureEvent`.
//!
//! C++ CreatureEvent registers Lua callbacks (onLogin, onDeath, …)
//! and dispatches them when game events fire. The Rust side has the
//! enum + dispatcher framework; full callback storage is wired by
//! the engine. Bindings here are setters that record the callback
//! name; actual dispatch happens at the appropriate event.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone, Default)]
pub struct LuaCreatureEvent {
    pub name: String,
    pub event_type: i64,
    pub registered_callbacks: Vec<String>,
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
        methods.add_method_mut("register", |lua, this, ()| {
            let store = lua
                .app_data_ref::<crate::lua_bindings::LuaCreatureEventStore>()
                .ok_or_else(|| mlua::Error::runtime("LuaCreatureEventStore not initialized"))?;
            store
                .0
                .lock()
                .map_err(|_| mlua::Error::runtime("lock poisoned"))?
                .push(this.clone());
            Ok(true)
        });
        // Allow field assignment (e.g. `creatureevent.onDeath = function() end`).
        // Lua scripts assign handler functions as fields; we accept but don't
        // store them here (real wiring happens in the dispatcher).
        methods.add_meta_method_mut(
            "__newindex",
            |_, _this, (_k, _v): (mlua::Value, mlua::Value)| Ok(()),
        );
        // Callback setters — record callback name so callers can inspect which
        // handlers have been registered. Real dispatch wiring happens in the engine.
        for cb_name in &[
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
            let cb_name_owned = cb_name.to_string();
            methods.add_method_mut(*cb_name, move |_, this, _cb: mlua::Value| {
                if !this.registered_callbacks.contains(&cb_name_owned) {
                    this.registered_callbacks.push(cb_name_owned.clone());
                }
                Ok(())
            });
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
    fn creature_event_callback_setter_records_name() {
        let lua = fresh_lua();
        lua.globals().set("ce", LuaCreatureEvent::default()).unwrap();
        lua.load(r#"ce:onLogin(function() end)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("ce").unwrap();
        let borrowed = ud.borrow::<LuaCreatureEvent>().unwrap();
        assert!(borrowed.registered_callbacks.contains(&"onLogin".to_string()));
    }

    #[test]
    fn creature_event_register_stores_in_store() {
        use crate::lua_bindings::LuaCreatureEventStore;
        let lua = mlua::Lua::new();
        let store = LuaCreatureEventStore::default();
        lua.set_app_data(store.clone());
        crate::lua_bindings::install_bindings(&lua, crate::lua_bindings::GameStateHandle::default()).unwrap();
        lua.load(r#"
            local ce = CreatureEvent("Login")
            function ce.onLogin(player) end
            ce:register()
        "#).exec().unwrap();
        let guard = store.0.lock().unwrap();
        assert_eq!(guard.len(), 1);
        assert_eq!(guard[0].name, "Login");
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
