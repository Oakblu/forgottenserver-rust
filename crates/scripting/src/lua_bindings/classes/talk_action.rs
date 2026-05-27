//! `TalkAction:*` Lua binding.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaTalkAction;

impl<'lua> mlua::FromLua<'lua> for LuaTalkAction {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaTalkAction>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaTalkAction",
                message: Some("expected TalkAction userdata".into()),
            }),
        }
    }
}

impl UserData for LuaTalkAction {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("access", |_, _this, _v: i64| Ok(()));
        methods.add_method_mut("accountType", |_, _this, _v: i64| Ok(()));
        methods.add_method_mut("separator", |_, _this, _v: String| Ok(()));
        methods.add_method_mut("onSay", |_, _this, _cb: Value| Ok(()));
        methods.add_method_mut("register", |_, _this, ()| Ok(true));
    }
}
