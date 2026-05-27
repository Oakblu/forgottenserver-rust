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
