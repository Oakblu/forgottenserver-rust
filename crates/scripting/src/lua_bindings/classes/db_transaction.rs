//! `DBTransaction:*` Lua binding (no-op until a real DB connection is wired
//! into the scripting layer; the audit needs the methods to exist so Lua scripts
//! that *try* to begin a transaction don't blow up).

// AUDIT: ClassMethod DBTransaction:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone, Default)]
pub struct LuaDbTransaction;

impl<'lua> mlua::FromLua<'lua> for LuaDbTransaction {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaDbTransaction>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaDbTransaction",
                message: Some("expected DBTransaction userdata".into()),
            }),
        }
    }
}

impl UserData for LuaDbTransaction {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, _this, _other: mlua::Value| {
            Ok(true)
        });
        methods.add_method("begin", |_, _this, ()| Ok(true));
        methods.add_method("commit", |_, _this, ()| Ok(true));
        methods.add_method("rollback", |_, _this, ()| Ok(true));
    }
}
