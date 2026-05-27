//! `MoveEvent:*` Lua binding (registration-only stub).

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaMoveEvent {
    pub event_type: i64,
    pub slot: i64,
}

impl<'lua> mlua::FromLua<'lua> for LuaMoveEvent {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaMoveEvent>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaMoveEvent",
                message: Some("expected MoveEvent userdata".into()),
            }),
        }
    }
}

impl UserData for LuaMoveEvent {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("type", |_, this, t: i64| {
            this.event_type = t;
            Ok(())
        });
        methods.add_method_mut("slot", |_, this, s: i64| {
            this.slot = s;
            Ok(())
        });
        methods.add_method_mut("register", |_, _this, ()| Ok(true));
        for n in &[
            "id",
            "aid",
            "uid",
            "position",
            "premium",
            "vocation",
            "tileItem",
            "level",
            "magicLevel",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(()));
        }
        for n in &[
            "onEquip",
            "onDeEquip",
            "onAddItem",
            "onRemoveItem",
            "onStepIn",
            "onStepOut",
        ] {
            methods.add_method_mut(n, |_, _this, _cb: Value| Ok(()));
        }
    }
}
