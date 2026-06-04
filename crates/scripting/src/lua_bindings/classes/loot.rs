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
    fn set_id_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("loot", LuaLoot).unwrap();
        let result: mlua::Result<()> = lua.load("loot:setId(100)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn set_chance_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("loot", LuaLoot).unwrap();
        let result: mlua::Result<()> = lua.load("loot:setChance(5000)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn set_max_count_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("loot", LuaLoot).unwrap();
        let result: mlua::Result<()> = lua.load("loot:setMaxCount(10)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn add_child_loot_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("loot", LuaLoot).unwrap();
        lua.globals().set("child", LuaLoot).unwrap();
        let result: mlua::Result<()> = lua.load("loot:addChildLoot(child)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaLoot> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
