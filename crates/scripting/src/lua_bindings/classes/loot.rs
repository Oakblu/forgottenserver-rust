//! `Loot:*` Lua binding (data-only builder used by monster XML loaders).

// AUDIT: ClassMethod Loot:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaLoot;

impl<'lua> mlua::FromLua<'lua> for LuaLoot {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaLoot>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaLoot",
                message: Some("expected Loot userdata".into()),
            }),
        }
    }
}

impl UserData for LuaLoot {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        for n in &[
            "setId",
            "setSubType",
            "setChance",
            "setMaxCount",
            "setActionId",
            "setDescription",
            "addChildLoot",
            "delete",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(()));
        }
    }
}
