//! `DBTransaction:*` Lua binding (no-op until a real DB connection is wired
//! into the scripting layer; the audit needs the methods to exist so Lua scripts
//! that *try* to begin a transaction don't blow up).

// AUDIT: ClassMethod DBTransaction:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone, Default)]
pub struct LuaDbTransaction;

impl<'lua> mlua::FromLua<'lua> for LuaDbTransaction {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaDbTransaction>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaDbTransaction",
                message: Some("expected DBTransaction userdata".into()),
            }),
        }
    }
}

impl UserData for LuaDbTransaction {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, _this, _other: mlua::Value| {
            Ok(true)
        });
        methods.add_method("begin", |_, _this, ()| Ok(true));
        methods.add_method("commit", |_, _this, ()| Ok(true));
        methods.add_method("rollback", |_, _this, ()| Ok(true));
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
    fn begin_returns_true() {
        let lua = fresh_lua();
        lua.globals().set("tx", LuaDbTransaction).unwrap();
        let v: bool = lua.load("return tx:begin()").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn commit_returns_true() {
        let lua = fresh_lua();
        lua.globals().set("tx", LuaDbTransaction).unwrap();
        let v: bool = lua.load("return tx:commit()").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn rollback_returns_true() {
        let lua = fresh_lua();
        lua.globals().set("tx", LuaDbTransaction).unwrap();
        let v: bool = lua.load("return tx:rollback()").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn eq_meta_returns_true() {
        let lua = fresh_lua();
        lua.globals().set("tx1", LuaDbTransaction).unwrap();
        lua.globals().set("tx2", LuaDbTransaction).unwrap();
        let v: bool = lua.load("return tx1 == tx2").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaDbTransaction> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
