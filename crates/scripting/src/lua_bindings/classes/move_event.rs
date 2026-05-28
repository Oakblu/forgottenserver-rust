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
        methods.add_method_mut("type", |_, this, t: Value| {
            this.event_type = match t {
                Value::Integer(n) => n,
                Value::String(s) => match s.to_str().unwrap_or("").to_lowercase().as_str() {
                    "stepin" => 0,
                    "stepout" => 1,
                    "equip" => 2,
                    "deequip" => 3,
                    "additem" => 4,
                    "removeitem" => 5,
                    _ => 0,
                },
                Value::Number(f) => f as i64,
                _ => 0,
            };
            Ok(())
        });
        methods.add_meta_method_mut("__newindex", |_, _this, (_k, _v): (Value, Value)| Ok(()));
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
    fn move_event_field_assignment_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load("local m = MoveEvent(); m.onStepIn = function() end")
            .exec();
        assert!(
            result.is_ok(),
            "field assignment on MoveEvent should not error: {result:?}"
        );
    }

    #[test]
    fn move_event_type_accepts_string_stepin() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local m = MoveEvent(); m:type("stepin")"#)
            .exec();
        assert!(
            result.is_ok(),
            "MoveEvent:type with string 'stepin' should not error: {result:?}"
        );
    }

    #[test]
    fn move_event_type_accepts_string_stepout() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local m = MoveEvent(); m:type("stepout")"#)
            .exec();
        assert!(
            result.is_ok(),
            "MoveEvent:type with string 'stepout' should not error: {result:?}"
        );
    }

    #[test]
    fn move_event_type_accepts_integer() {
        let lua = fresh_lua();
        let result = lua
            .load("local m = MoveEvent(); m:type(0)")
            .exec();
        assert!(
            result.is_ok(),
            "MoveEvent:type with integer should still work: {result:?}"
        );
    }
}
