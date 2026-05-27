//! `Variant:*` Lua binding for `common::luavariant::LuaVariant`.
//!
//! C++ Lua side:
//!   - `Variant:getNumber()`   → number or 0
//!   - `Variant:getPosition()` → Position or nil
//!   - `Variant:getString()`   → string or ""
//!
//! Newtype `LuaVariant` wraps the common `LuaVariant` so we can
//! impl `mlua::UserData` (orphan-rule workaround).

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_common::luavariant::LuaVariant as InnerVariant;
use mlua::{UserData, UserDataMethods};

use crate::lua_bindings::position::LuaPosition;

#[derive(Debug, Clone)]
pub struct LuaVariant(pub InnerVariant);

impl LuaVariant {
    pub fn new(v: InnerVariant) -> Self {
        Self(v)
    }
    pub fn into_inner(self) -> InnerVariant {
        self.0
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaVariant {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaVariant>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaVariant",
                message: Some("expected Variant userdata".into()),
            }),
        }
    }
}

impl UserData for LuaVariant {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // `getNumber` — C++ returns 0 when variant isn't a number.
        methods.add_method("getNumber", |_, this, ()| match &this.0 {
            InnerVariant::Number(n) => Ok(*n as i64),
            _ => Ok(0i64),
        });
        // `getPosition` — C++ returns a default Position (0,0,0) when not set;
        // we follow the same convention.
        methods.add_method("getPosition", |_, this, ()| match &this.0 {
            InnerVariant::Position(p) | InnerVariant::TargetPosition(p) => Ok(LuaPosition(*p)),
            _ => Ok(LuaPosition::new(0, 0, 0)),
        });
        // `getString` — C++ returns "" when not a string.
        methods.add_method("getString", |_, this, ()| match &this.0 {
            InnerVariant::String(s) => Ok(s.clone()),
            _ => Ok(String::new()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::position::Position;

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
    fn get_number_returns_inner_value() {
        let lua = fresh_lua();
        let v = LuaVariant::new(InnerVariant::Number(42));
        let g = lua.globals();
        g.set("v", v).unwrap();
        let n: i64 = lua.load("return v:getNumber()").eval().unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn get_number_returns_zero_for_non_number() {
        let lua = fresh_lua();
        let v = LuaVariant::new(InnerVariant::String("hello".into()));
        lua.globals().set("v", v).unwrap();
        let n: i64 = lua.load("return v:getNumber()").eval().unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn get_position_returns_inner_position() {
        let lua = fresh_lua();
        let v = LuaVariant::new(InnerVariant::Position(Position::new(10, 20, 5)));
        lua.globals().set("v", v).unwrap();
        let x: u16 = lua.load("return v:getPosition().x").eval().unwrap();
        assert_eq!(x, 10);
    }

    #[test]
    fn get_string_returns_inner_string() {
        let lua = fresh_lua();
        let v = LuaVariant::new(InnerVariant::String("hello".into()));
        lua.globals().set("v", v).unwrap();
        let s: String = lua.load("return v:getString()").eval().unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn get_string_returns_empty_for_non_string() {
        let lua = fresh_lua();
        let v = LuaVariant::new(InnerVariant::Number(7));
        lua.globals().set("v", v).unwrap();
        let s: String = lua.load("return v:getString()").eval().unwrap();
        assert_eq!(s, "");
    }
}
