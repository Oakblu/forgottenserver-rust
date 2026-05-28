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
