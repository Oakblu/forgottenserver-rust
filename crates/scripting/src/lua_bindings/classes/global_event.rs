//! `GlobalEvent:*` Lua binding.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaGlobalEvent;

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
        methods.add_meta_method_mut("__newindex", |_, _this, (_k, _v): (Value, Value)| Ok(()));
        methods.add_method_mut("type", |_, _this, _v: Value| Ok(()));
        methods.add_method_mut("interval", |_, _this, _v: i64| Ok(()));
        methods.add_method_mut("time", |_, _this, _v: String| Ok(()));
        methods.add_method_mut("register", |_, _this, ()| Ok(true));
        for name in &[
            "onStartup",
            "onShutdown",
            "onRecord",
            "onSave",
            "onThink",
            "onTime",
        ] {
            methods.add_method_mut(*name, |_, _this, _cb: Value| Ok(()));
        }
    }
}

#[cfg(test)]
mod tests {
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
}
