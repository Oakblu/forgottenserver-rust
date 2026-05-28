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
    pub item_ids: Vec<i64>,
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
        methods.add_method_mut("id", |_, this, id: i64| {
            this.item_ids.push(id);
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
        methods.add_method_mut("register", |lua, this, ()| {
            let store = lua
                .app_data_ref::<crate::lua_bindings::LuaActionStore>()
                .ok_or_else(|| mlua::Error::runtime("LuaActionStore not initialized"))?;
            store
                .0
                .lock()
                .map_err(|_| mlua::Error::runtime("LuaActionStore lock poisoned"))?
                .push(this.clone());
            Ok(true)
        });
        methods.add_meta_method_mut("__newindex", |_, _this, (_k, _v): (Value, Value)| Ok(()));
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
    fn action_field_assignment_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load("local a = Action(); a.onUse = function() end")
            .exec();
        assert!(
            result.is_ok(),
            "field assignment on Action should not error: {result:?}"
        );
    }

    #[test]
    fn action_function_syntax_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load("local a = Action(); function a.onUse(p, i) end")
            .exec();
        assert!(
            result.is_ok(),
            "function-declaration syntax on Action should not error: {result:?}"
        );
    }

    #[test]
    fn action_id_setter_stores_item_id() {
        let lua = fresh_lua();
        lua.globals().set("a", LuaAction::default()).unwrap();
        lua.load(r#"a:id(1234)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("a").unwrap();
        let borrowed = ud.borrow::<LuaAction>().unwrap();
        assert!(borrowed.item_ids.contains(&1234));
    }

    #[test]
    fn action_id_setter_accepts_multiple_calls() {
        let lua = fresh_lua();
        lua.globals().set("a", LuaAction::default()).unwrap();
        lua.load(r#"a:id(100); a:id(200)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("a").unwrap();
        let borrowed = ud.borrow::<LuaAction>().unwrap();
        assert_eq!(borrowed.item_ids.len(), 2);
    }

    #[test]
    fn action_register_stores_in_lua_action_store() {
        use crate::lua_bindings::LuaActionStore;
        let lua = mlua::Lua::new();
        let store = LuaActionStore::default();
        lua.set_app_data(store.clone());
        crate::lua_bindings::install_bindings(&lua, crate::lua_bindings::GameStateHandle::default()).unwrap();

        lua.load(r#"
            local a = Action()
            a:id(1234)
            a:register()
        "#).exec().unwrap();

        let count = store.0.lock().unwrap().len();
        assert_eq!(count, 1, "register() must add the action to LuaActionStore");
        assert!(store.0.lock().unwrap()[0].item_ids.contains(&1234));
    }
}
