//! `configManager` Lua binding — stub singleton.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaConfigManager;

impl UserData for LuaConfigManager {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("getString", |_, _, _key: Value| Ok("".to_string()));
        methods.add_method("getNumber", |_, _, _key: Value| Ok(0i64));
        methods.add_method("getBoolean", |_, _, _key: Value| Ok(false));
        methods.add_method("getFloat", |_, _, _key: Value| Ok(0.0f64));
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
    fn get_string_returns_empty() {
        let lua = fresh_lua();
        lua.globals()
            .set("cfg", LuaConfigManager)
            .unwrap();
        let v: String = lua.load("return cfg:getString(1)").eval().unwrap();
        assert_eq!(v, "");
    }

    #[test]
    fn get_number_returns_zero() {
        let lua = fresh_lua();
        lua.globals()
            .set("cfg", LuaConfigManager)
            .unwrap();
        let v: i64 = lua.load("return cfg:getNumber(1)").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_boolean_returns_false() {
        let lua = fresh_lua();
        lua.globals()
            .set("cfg", LuaConfigManager)
            .unwrap();
        let v: bool = lua.load("return cfg:getBoolean(1)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn get_float_returns_zero() {
        let lua = fresh_lua();
        lua.globals()
            .set("cfg", LuaConfigManager)
            .unwrap();
        let v: f64 = lua.load("return cfg:getFloat(1)").eval().unwrap();
        assert!((v - 0.0f64).abs() < f64::EPSILON);
    }
}
