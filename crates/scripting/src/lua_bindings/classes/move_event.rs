//! `MoveEvent:*` Lua binding (registration-only stub).

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaMoveEvent {
    pub event_type: i64,
    pub slot: i64,
    pub item_id: i64,
    pub action_id: i64,
    pub unique_id: i64,
    pub premium: bool,
    pub level: i64,
    pub magic_level: i64,
    pub vocation_name: String,
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
        methods.add_method_mut("slot", |_, this, s: Value| {
            this.slot = match s {
                Value::Integer(n) => n,
                Value::String(s) => match s.to_str().unwrap_or("").to_lowercase().as_str() {
                    "head" => 1,
                    "necklace" => 2,
                    "backpack" => 3,
                    "armor" => 4,
                    "right" => 5,
                    "left" => 6,
                    "legs" => 7,
                    "feet" => 8,
                    "ring" => 9,
                    "ammo" => 10,
                    _ => 0,
                },
                Value::Number(f) => f as i64,
                _ => 0,
            };
            Ok(())
        });
        methods.add_method_mut("register", |lua, this, ()| {
            let store = lua
                .app_data_ref::<crate::lua_bindings::LuaMoveEventStore>()
                .ok_or_else(|| mlua::Error::runtime("LuaMoveEventStore not initialized"))?;
            store.0.lock().map_err(|_| mlua::Error::runtime("lock poisoned"))?.push(this.clone());
            Ok(true)
        });
        methods.add_method_mut("id", |_, this, v: i64| {
            this.item_id = v;
            Ok(())
        });
        methods.add_method_mut("aid", |_, this, v: i64| {
            this.action_id = v;
            Ok(())
        });
        methods.add_method_mut("uid", |_, this, v: i64| {
            this.unique_id = v;
            Ok(())
        });
        methods.add_method_mut("premium", |_, this, v: bool| {
            this.premium = v;
            Ok(())
        });
        methods.add_method_mut("level", |_, this, v: i64| {
            this.level = v;
            Ok(())
        });
        methods.add_method_mut("magicLevel", |_, this, v: i64| {
            this.magic_level = v;
            Ok(())
        });
        methods.add_method_mut("vocation", |_, this, v: String| {
            this.vocation_name = v;
            Ok(())
        });
        methods.add_method_mut("tileItem", |_, _this, _v: mlua::Value| Ok(()));
        methods.add_method_mut("position", |_, _this, _v: mlua::Value| Ok(()));
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
        let result = lua.load("local m = MoveEvent(); m:type(0)").exec();
        assert!(
            result.is_ok(),
            "MoveEvent:type with integer should still work: {result:?}"
        );
    }

    #[test]
    fn slot_accepts_string_ring() {
        let lua = fresh_lua();
        let result = lua.load(r#"local m = MoveEvent(); m:slot("ring")"#).exec();
        assert!(
            result.is_ok(),
            "MoveEvent:slot with string 'ring' should not error: {result:?}"
        );
    }

    #[test]
    fn slot_accepts_string_head() {
        let lua = fresh_lua();
        let result = lua.load(r#"local m = MoveEvent(); m:slot("head")"#).exec();
        assert!(
            result.is_ok(),
            "MoveEvent:slot with string 'head' should not error: {result:?}"
        );
    }

    #[test]
    fn slot_accepts_integer_regression() {
        let lua = fresh_lua();
        let result = lua.load("local m = MoveEvent(); m:slot(9)").exec();
        assert!(
            result.is_ok(),
            "MoveEvent:slot with integer should still work: {result:?}"
        );
    }

    #[test]
    fn move_event_id_setter() {
        let lua = fresh_lua();
        lua.globals().set("m", LuaMoveEvent::default()).unwrap();
        lua.load(r#"m:id(1200)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("m").unwrap();
        let borrowed = ud.borrow::<LuaMoveEvent>().unwrap();
        assert_eq!(borrowed.item_id, 1200);
    }

    #[test]
    fn move_event_aid_setter() {
        let lua = fresh_lua();
        lua.globals().set("m", LuaMoveEvent::default()).unwrap();
        lua.load(r#"m:aid(500)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("m").unwrap();
        let borrowed = ud.borrow::<LuaMoveEvent>().unwrap();
        assert_eq!(borrowed.action_id, 500);
    }

    #[test]
    fn move_event_level_setter() {
        let lua = fresh_lua();
        lua.globals().set("m", LuaMoveEvent::default()).unwrap();
        lua.load(r#"m:level(100)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("m").unwrap();
        let borrowed = ud.borrow::<LuaMoveEvent>().unwrap();
        assert_eq!(borrowed.level, 100);
    }

    #[test]
    fn move_event_register_stores_in_lua_move_event_store() {
        use crate::lua_bindings::LuaMoveEventStore;
        let lua = mlua::Lua::new();
        let store = LuaMoveEventStore::default();
        lua.set_app_data(store.clone());
        crate::lua_bindings::install_bindings(&lua, crate::lua_bindings::GameStateHandle::default()).unwrap();
        lua.load(r#"
            local m = MoveEvent()
            m:type("stepin")
            m:id(1234)
            m:register()
        "#).exec().unwrap();
        let guard = store.0.lock().unwrap();
        assert_eq!(guard.len(), 1);
        assert_eq!(guard[0].item_id, 1234);
    }
}
