//! `Podium:*` Lua binding for `entity::podium::Podium`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_common::position::Direction;
use forgottenserver_entity::podium::{Podium, PodiumFlags};
use mlua::{UserData, UserDataMethods};

use crate::lua_bindings::classes::outfit::LuaOutfit;

fn flag_from_i64(v: i64) -> Option<PodiumFlags> {
    match v {
        0 => Some(PodiumFlags::ShowPlatform),
        1 => Some(PodiumFlags::ShowOutfit),
        2 => Some(PodiumFlags::ShowMount),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct LuaPodium(pub Podium);

impl LuaPodium {
    pub fn new(p: Podium) -> Self {
        Self(p)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaPodium {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaPodium>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaPodium",
                message: Some("expected Podium userdata".into()),
            }),
        }
    }
}

impl UserData for LuaPodium {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaPodium| {
            Ok(this.0.item_type_id == other.0.item_type_id)
        });
        methods.add_method("getDirection", |_, this, ()| Ok(this.0.direction as i64));
        methods.add_method_mut("setDirection", |_, this, d: i64| {
            // Cast to Direction enum; clamp to safe range.
            this.0.direction = match d {
                0 => Direction::North,
                1 => Direction::East,
                2 => Direction::South,
                3 => Direction::West,
                _ => Direction::South,
            };
            Ok(())
        });
        methods.add_method("getOutfit", |_, this, ()| {
            Ok(LuaOutfit::new(*this.0.get_outfit()))
        });
        methods.add_method_mut("setOutfit", |_, this, outfit: LuaOutfit| {
            this.0.set_outfit(outfit.0);
            Ok(())
        });
        methods.add_method("hasFlag", |_, this, flag: i64| {
            Ok(flag_from_i64(flag)
                .map(|f| this.0.has_flag(f))
                .unwrap_or(false))
        });
        methods.add_method_mut("setFlag", |_, this, (flag, value): (i64, bool)| {
            if let Some(f) = flag_from_i64(flag) {
                this.0.set_flag(f, value);
            }
            Ok(())
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
    fn get_direction_returns_default_south() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let v: i64 = lua.load("return p:getDirection()").eval().unwrap();
        // Direction::South = 2
        assert_eq!(v, 2);
    }

    #[test]
    fn set_direction_mutates() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let v: i64 = lua
            .load("p:setDirection(1); return p:getDirection()")
            .eval()
            .unwrap();
        assert_eq!(v, 1); // East
    }
}
