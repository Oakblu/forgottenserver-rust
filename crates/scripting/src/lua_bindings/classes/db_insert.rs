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
    fn add_row_returns_true() {
        let lua = fresh_lua();
        lua.globals().set("ins", LuaDbInsert).unwrap();
        let v: bool = lua.load("return ins:addRow('val1')").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn execute_returns_true() {
        let lua = fresh_lua();
        lua.globals().set("ins", LuaDbInsert).unwrap();
        let v: bool = lua.load("return ins:execute()").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaDbInsert> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
