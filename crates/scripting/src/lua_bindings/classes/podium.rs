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
    fn from_lua_with_wrong_type_returns_error() {
        use mlua::FromLua as _;
        let lua = mlua::Lua::new();
        let result = LuaPodium::from_lua(mlua::Value::Integer(99), &lua);
        assert!(result.is_err(), "from_lua must fail for non-userdata");
        if let Err(mlua::Error::FromLuaConversionError { to, .. }) = result {
            assert_eq!(to, "LuaPodium");
        }
    }

    #[test]
    fn set_direction_north_east_west_all_work() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        // North=0, East=1, South=2, West=3
        for (d, expected) in [(0i64, 0i64), (1, 1), (3, 3)] {
            let code = format!("p:setDirection({d}); return p:getDirection()");
            let v: i64 = lua.load(&code).eval().unwrap();
            assert_eq!(v, expected, "direction {d} should round-trip");
        }
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

    #[test]
    fn set_direction_unknown_clamps_to_south() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let v: i64 = lua
            .load("p:setDirection(99); return p:getDirection()")
            .eval()
            .unwrap();
        assert_eq!(v, 2); // South fallback
    }

    #[test]
    fn get_outfit_returns_outfit_userdata() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let result: mlua::Result<bool> = lua
            .load("return p:getOutfit() ~= nil")
            .eval();
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn set_outfit_does_not_error() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let result: mlua::Result<()> = lua
            .load("local o = p:getOutfit(); p:setOutfit(o)")
            .exec();
        assert!(result.is_ok());
    }

    #[test]
    fn has_show_platform_flag_true_by_default() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        // Flag 0 = ShowPlatform — set by default in Podium::new
        let v: bool = lua.load("return p:hasFlag(0)").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn has_show_outfit_flag_false_by_default() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        // Flag 1 = ShowOutfit — not set by default
        let v: bool = lua.load("return p:hasFlag(1)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn set_flag_and_has_flag() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let v: bool = lua
            .load("p:setFlag(0, true); return p:hasFlag(0)")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn set_flag_with_false_clears_flag() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let v: bool = lua
            .load("p:setFlag(1, true); p:setFlag(1, false); return p:hasFlag(1)")
            .eval()
            .unwrap();
        assert!(!v);
    }

    #[test]
    fn has_flag_unknown_returns_false() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let v: bool = lua.load("return p:hasFlag(99)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn set_flag_unknown_does_not_error() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let result: mlua::Result<()> = lua.load("p:setFlag(99, true)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn eq_meta_same_type_id() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaPodium::new(Podium::new(100)))
            .unwrap();
        lua.globals()
            .set("b", LuaPodium::new(Podium::new(100)))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn eq_meta_different_type_id() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaPodium::new(Podium::new(100)))
            .unwrap();
        lua.globals()
            .set("b", LuaPodium::new(Podium::new(200)))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn all_valid_flag_indices_can_be_set() {
        let lua = fresh_lua();
        lua.globals()
            .set("p", LuaPodium::new(Podium::new(100)))
            .unwrap();
        // Test all three valid flag indices: 0=ShowPlatform, 1=ShowOutfit, 2=ShowMount
        // Each should be settable to true and readable back
        for i in 0..=2i64 {
            let code = format!("p:setFlag({i}, true); return p:hasFlag({i})");
            let v: bool = lua.load(&code).eval().unwrap();
            assert!(v, "flag {i} should be set");
        }
    }
}
