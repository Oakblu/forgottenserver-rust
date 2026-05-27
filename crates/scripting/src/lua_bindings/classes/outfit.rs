//! `Outfit:*` Lua binding for `common::enums::Outfit`.
//!
//! C++ Lua side exposes:
//!   - `Outfit:__eq` — equality comparison
//!
//! All Outfit fields (lookType, lookHead, …) are accessed via Lua
//! table-style indexing in C++ (`outfit.lookType`), not via methods.
//! Those field accessors are not in the C++ `registerMethod` list
//! and thus aren't required for parity — but adding them would
//! also satisfy Lua scripts using table notation.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_common::enums::Outfit;
use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone, Copy)]
pub struct LuaOutfit(pub Outfit);

impl LuaOutfit {
    pub fn new(o: Outfit) -> Self {
        Self(o)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaOutfit {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<LuaOutfit>()?),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaOutfit",
                message: Some("expected Outfit userdata".into()),
            }),
        }
    }
}

impl UserData for LuaOutfit {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaOutfit| {
            Ok(this.0 == other.0)
        });
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
    fn equal_outfits_are_equal_in_lua() {
        let lua = fresh_lua();
        let a = LuaOutfit::new(Outfit {
            look_type: 100,
            ..Outfit::default()
        });
        let b = LuaOutfit::new(Outfit {
            look_type: 100,
            ..Outfit::default()
        });
        lua.globals().set("a", a).unwrap();
        lua.globals().set("b", b).unwrap();
        let eq: bool = lua.load("return a == b").eval().unwrap();
        assert!(eq);
    }

    #[test]
    fn different_outfits_are_unequal_in_lua() {
        let lua = fresh_lua();
        let a = LuaOutfit::new(Outfit {
            look_type: 100,
            ..Outfit::default()
        });
        let b = LuaOutfit::new(Outfit {
            look_type: 200,
            ..Outfit::default()
        });
        lua.globals().set("a", a).unwrap();
        lua.globals().set("b", b).unwrap();
        let eq: bool = lua.load("return a == b").eval().unwrap();
        assert!(!eq);
    }
}
