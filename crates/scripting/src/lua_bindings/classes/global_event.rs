//! `GlobalEvent:*` Lua binding.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone, Default)]
pub struct LuaGlobalEvent {
    pub name: String,
    pub event_type: i64,
    pub interval: i64,
    pub time: String,
}

impl<'lua> mlua::FromLua<'lua> for LuaGlobalEvent {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaGlobalEvent>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaGlobalEvent",
                message: Some("expected GlobalEvent userdata".into()),
            }),
        }
    }
}

impl UserData for LuaGlobalEvent {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method_mut("__newindex", |_, _this, (_k, _v): (mlua::Value, mlua::Value)| Ok(()));
        methods.add_method_mut("type", |_, this, v: mlua::Value| {
            this.event_type = match v {
                mlua::Value::Integer(n) => n,
                mlua::Value::String(s) => s.to_str().ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
                _ => 0,
            };
            Ok(())
        });
        methods.add_method_mut("interval", |_, this, v: i64| {
            this.interval = v;
            Ok(())
        });
        methods.add_method_mut("time", |_, this, v: String| {
            this.time = v;
            Ok(())
        });
        methods.add_method_mut("register", |lua, this, ()| {
            let store = lua
                .app_data_ref::<crate::lua_bindings::LuaGlobalEventStore>()
                .ok_or_else(|| mlua::Error::runtime("LuaGlobalEventStore not initialized"))?;
            store.0.lock().map_err(|_| mlua::Error::runtime("lock poisoned"))?.push(this.clone());
            Ok(true)
        });
        for name in &[
            "onStartup",
            "onShutdown",
            "onRecord",
            "onSave",
            "onThink",
            "onTime",
        ] {
            methods.add_method_mut(*name, |_, _this, _cb: mlua::Value| Ok(()));
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
    fn global_event_field_assignment_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local e = GlobalEvent("Test"); e.onSave = function() end"#)
            .exec();
        assert!(
            result.is_ok(),
            "field assignment on GlobalEvent should not error: {result:?}"
        );
    }

    #[test]
    fn global_event_function_syntax_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local e = GlobalEvent("Test"); function e.onThink(interval) end"#)
            .exec();
        assert!(
            result.is_ok(),
            "function-declaration syntax on GlobalEvent should not error: {result:?}"
        );
    }

    #[test]
    fn global_event_type_setter() {
        let lua = fresh_lua();
        lua.globals().set("e", LuaGlobalEvent::default()).unwrap();
        lua.load(r#"e:type(1)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("e").unwrap();
        let borrowed = ud.borrow::<LuaGlobalEvent>().unwrap();
        assert_eq!(borrowed.event_type, 1);
    }

    #[test]
    fn global_event_interval_setter() {
        let lua = fresh_lua();
        lua.globals().set("e", LuaGlobalEvent::default()).unwrap();
        lua.load(r#"e:interval(60000)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("e").unwrap();
        let borrowed = ud.borrow::<LuaGlobalEvent>().unwrap();
        assert_eq!(borrowed.interval, 60000);
    }

    #[test]
    fn global_event_time_setter() {
        let lua = fresh_lua();
        lua.globals().set("e", LuaGlobalEvent::default()).unwrap();
        lua.load(r#"e:time("06:00")"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("e").unwrap();
        let borrowed = ud.borrow::<LuaGlobalEvent>().unwrap();
        assert_eq!(borrowed.time, "06:00");
    }

    #[test]
    fn global_event_register_stores_in_store() {
        use crate::lua_bindings::LuaGlobalEventStore;
        let lua = mlua::Lua::new();
        let store = LuaGlobalEventStore::default();
        lua.set_app_data(store.clone());
        crate::lua_bindings::install_bindings(&lua, crate::lua_bindings::GameStateHandle::default()).unwrap();
        lua.load(r#"
            local e = GlobalEvent("Save")
            e:type(1)
            e:register()
        "#).exec().unwrap();
        assert_eq!(store.0.lock().unwrap().len(), 1);
        assert_eq!(store.0.lock().unwrap()[0].name, "Save");
    }
}
