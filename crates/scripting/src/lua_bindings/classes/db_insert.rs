//! `DBInsert:*` Lua binding (no-op stub mirroring the C++ `DBInsert` builder).

// AUDIT: ClassMethod DBInsert:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaDbInsert;

impl<'lua> mlua::FromLua<'lua> for LuaDbInsert {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaDbInsert>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaDbInsert",
                message: Some("expected DBInsert userdata".into()),
            }),
        }
    }
}

impl UserData for LuaDbInsert {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("addRow", |_, _this, _args: Value| Ok(true));
        methods.add_method("execute", |_, _this, ()| Ok(true));
    }
}
