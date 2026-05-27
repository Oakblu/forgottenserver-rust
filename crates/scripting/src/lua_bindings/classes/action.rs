//! `Action:*` Lua binding for `game::action_registry::Action`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaAction {
    pub action_id: i64,
    pub unique_id: i64,
    pub allow_far_use: bool,
    pub block_walls: bool,
    pub check_floor: bool,
}

impl LuaAction {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaAction {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaAction>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaAction",
                message: Some("expected Action userdata".into()),
            }),
        }
    }
}

impl UserData for LuaAction {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("id", |_, _this, _args: Value| Ok(()));
        methods.add_method_mut("aid", |_, this, v: i64| {
            this.action_id = v;
            Ok(())
        });
        methods.add_method_mut("uid", |_, this, v: i64| {
            this.unique_id = v;
            Ok(())
        });
        methods.add_method_mut("allowFarUse", |_, this, v: bool| {
            this.allow_far_use = v;
            Ok(())
        });
        methods.add_method_mut("blockWalls", |_, this, v: bool| {
            this.block_walls = v;
            Ok(())
        });
        methods.add_method_mut("checkFloor", |_, this, v: bool| {
            this.check_floor = v;
            Ok(())
        });
        methods.add_method_mut("onUse", |_, _this, _cb: Value| Ok(()));
        methods.add_method_mut("register", |_, _this, ()| Ok(true));
    }
}
