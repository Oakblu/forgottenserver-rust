//! `Monster:*` Lua binding for `entity::monster::Monster`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_entity::monster::Monster;
use mlua::{UserData, UserDataMethods, Value};
use std::sync::{Arc, Mutex};

pub struct LuaMonster(pub Arc<Mutex<Monster>>);

impl LuaMonster {
    pub fn new(m: Monster) -> Self {
        Self(Arc::new(Mutex::new(m)))
    }
}

impl Clone for LuaMonster {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaMonster {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaMonster>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaMonster",
                message: Some("expected Monster userdata".into()),
            }),
        }
    }
}

impl UserData for LuaMonster {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaMonster| {
            Ok(Arc::ptr_eq(&this.0, &other.0))
        });
        methods.add_method("getId", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_id() as i64)
        });
        methods.add_method("getTargetCount", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_target_count() as i64)
        });
        methods.add_method("getTargetList", |lua, this, ()| {
            let t = lua.create_table()?;
            for (i, id) in this.0.lock().unwrap().get_target_ids().iter().enumerate() {
                t.set(i as i64 + 1, *id as i64)?;
            }
            Ok(t)
        });
        methods.add_method_mut("selectTarget", |_, _this, _id: i64| Ok(false));
        methods.add_method_mut("searchTarget", |_, _this, _args: Value| Ok(false));
        methods.add_method_mut("setIdle", |_, _this, _v: bool| Ok(()));
        methods.add_method("isIdle", |_, _this, ()| Ok(false));
        // ── Stubs (Monster runtime ops need game-state plumbing) ──
        for n in &[
            "isOpponent",
            "isFriend",
            "isInSpawnRange",
            "isMonster",
            "getType",
            "rename",
            "getSpawnPosition",
            "addFriend",
            "removeFriend",
            "getFriendList",
            "getFriendCount",
            "getSpecialIcon",
            "hasSpecialIcon",
            "isTarget",
            "isWalkingToSpawn",
            "removeSpecialIcon",
            "setSpecialIcon",
            "walkToSpawn",
            "addTarget",
            "removeTarget",
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
    fn id_returns_field() {
        // C++ exposes `Monster:getId` via the inherited Creature class.
        // We bind `getId` directly on the Monster userdata since the
        // scripting crate doesn't have a per-class inheritance system.
        let lua = fresh_lua();
        let m = Monster::new(42, "Rat", 100);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let id: i64 = lua.load("return m:getId()").eval().unwrap();
        assert_eq!(id, 42);
    }

    #[test]
    fn get_target_count_returns_zero() {
        let lua = fresh_lua();
        let m = Monster::new(1, "Rat", 50);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let v: i64 = lua.load("return m:getTargetCount()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_target_list_returns_table() {
        let lua = fresh_lua();
        let m = Monster::new(1, "Rat", 50);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let v: bool = lua
            .load("return type(m:getTargetList()) == 'table'")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn select_target_returns_false() {
        let lua = fresh_lua();
        let m = Monster::new(1, "Rat", 50);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let v: bool = lua.load("return m:selectTarget(5)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn search_target_returns_false() {
        let lua = fresh_lua();
        let m = Monster::new(1, "Rat", 50);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let v: bool = lua.load("return m:searchTarget()").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn set_idle_does_not_error() {
        let lua = fresh_lua();
        let m = Monster::new(1, "Rat", 50);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let result: mlua::Result<()> = lua.load("m:setIdle(true)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn is_idle_returns_false() {
        let lua = fresh_lua();
        let m = Monster::new(1, "Rat", 50);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let v: bool = lua.load("return m:isIdle()").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn stub_methods_do_not_error() {
        let lua = fresh_lua();
        let m = Monster::new(1, "Rat", 50);
        lua.globals().set("m", LuaMonster::new(m)).unwrap();
        let result: mlua::Result<()> = lua.load("m:isOpponent(); m:isFriend(); m:isMonster()").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn eq_same_arc_is_true() {
        let lua = fresh_lua();
        let m = LuaMonster::new(Monster::new(1, "Rat", 50));
        let m2 = m.clone();
        lua.globals().set("a", m).unwrap();
        lua.globals().set("b", m2).unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn eq_different_arcs_is_false() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaMonster::new(Monster::new(1, "Rat", 50)))
            .unwrap();
        lua.globals()
            .set("b", LuaMonster::new(Monster::new(2, "Rat", 50)))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaMonster> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
