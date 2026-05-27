//! `Group:*` Lua binding for `items::groups::Group`.
//!
//! C++ Lua side:
//!   - `Group:__eq` — equality
//!   - `Group:getAccess` → bool
//!   - `Group:getFlags` → i64 (bitmask)
//!   - `Group:getId` → number
//!   - `Group:getMaxDepotItems` → number
//!   - `Group:getMaxVipEntries` → number
//!   - `Group:getName` → string
//!   - `Group:hasFlag(flag)` → bool

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_items::groups::Group;
use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone)]
pub struct LuaGroup(pub Group);

impl LuaGroup {
    pub fn new(g: Group) -> Self {
        Self(g)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaGroup {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaGroup>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaGroup",
                message: Some("expected Group userdata".into()),
            }),
        }
    }
}

impl UserData for LuaGroup {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaGroup| {
            Ok(this.0 == other.0)
        });
        methods.add_method("getAccess", |_, this, ()| Ok(this.0.access));
        methods.add_method("getFlags", |_, this, ()| Ok(this.0.flags as i64));
        methods.add_method("getId", |_, this, ()| Ok(this.0.id as i64));
        methods.add_method("getMaxDepotItems", |_, this, ()| {
            Ok(this.0.max_depot_items as i64)
        });
        methods.add_method("getMaxVipEntries", |_, this, ()| {
            Ok(this.0.max_vip_entries as i64)
        });
        methods.add_method("getName", |_, this, ()| Ok(this.0.name.clone()));
        methods.add_method("hasFlag", |_, this, flag: i64| {
            Ok(this.0.has_flag(flag as u64))
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

    fn sample_group() -> Group {
        Group {
            id: 2,
            name: "Tutor".to_string(),
            flags: 0b1010,
            max_depot_items: 1000,
            max_vip_entries: 50,
            access: true,
        }
    }

    #[test]
    fn get_id_returns_struct_field() {
        let lua = fresh_lua();
        lua.globals()
            .set("g", LuaGroup::new(sample_group()))
            .unwrap();
        let v: i64 = lua.load("return g:getId()").eval().unwrap();
        assert_eq!(v, 2);
    }

    #[test]
    fn get_name_returns_string() {
        let lua = fresh_lua();
        lua.globals()
            .set("g", LuaGroup::new(sample_group()))
            .unwrap();
        let v: String = lua.load("return g:getName()").eval().unwrap();
        assert_eq!(v, "Tutor");
    }

    #[test]
    fn get_access_returns_bool() {
        let lua = fresh_lua();
        lua.globals()
            .set("g", LuaGroup::new(sample_group()))
            .unwrap();
        let v: bool = lua.load("return g:getAccess()").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn has_flag_returns_true_when_bit_set() {
        let lua = fresh_lua();
        lua.globals()
            .set("g", LuaGroup::new(sample_group()))
            .unwrap();
        // flags = 0b1010, so bit 1 (=2) is set, bit 2 (=4) is not.
        let v: bool = lua.load("return g:hasFlag(2)").eval().unwrap();
        assert!(v);
        let v: bool = lua.load("return g:hasFlag(4)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn equality_metamethod_works_for_same_group() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaGroup::new(sample_group()))
            .unwrap();
        lua.globals()
            .set("b", LuaGroup::new(sample_group()))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn get_max_depot_items_returns_field() {
        let lua = fresh_lua();
        lua.globals()
            .set("g", LuaGroup::new(sample_group()))
            .unwrap();
        let v: i64 = lua.load("return g:getMaxDepotItems()").eval().unwrap();
        assert_eq!(v, 1000);
    }
}
