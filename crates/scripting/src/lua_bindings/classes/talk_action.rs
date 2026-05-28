//! `TalkAction:*` Lua binding.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone, Default)]
pub struct LuaTalkAction {
    pub word: String,
    pub access: i64,
    pub account_type: i64,
    pub separator: String,
}

impl<'lua> mlua::FromLua<'lua> for LuaTalkAction {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaTalkAction>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaTalkAction",
                message: Some("expected TalkAction userdata".into()),
            }),
        }
    }
}

impl UserData for LuaTalkAction {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("access", |_, this, v: mlua::Value| {
            this.access = match v {
                mlua::Value::Boolean(b) => {
                    if b {
                        1
                    } else {
                        0
                    }
                }
                mlua::Value::Integer(n) => n,
                _ => 0,
            };
            Ok(())
        });
        methods.add_method_mut("accountType", |_, this, v: i64| {
            this.account_type = v;
            Ok(())
        });
        methods.add_method_mut("separator", |_, this, v: String| {
            this.separator = v;
            Ok(())
        });
        methods.add_method_mut("onSay", |_, _this, _cb: mlua::Value| Ok(()));
        methods.add_method_mut("register", |lua, this, ()| {
            let store = lua
                .app_data_ref::<crate::lua_bindings::LuaTalkActionStore>()
                .ok_or_else(|| mlua::Error::runtime("LuaTalkActionStore not initialized"))?;
            store
                .0
                .lock()
                .map_err(|_| mlua::Error::runtime("lock poisoned"))?
                .push(this.clone());
            Ok(true)
        });
        methods.add_meta_method_mut(
            "__newindex",
            |_, _this, (_k, _v): (mlua::Value, mlua::Value)| Ok(()),
        );
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
    fn talk_action_field_assignment_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local t = TalkAction("/ban"); t.onSay = function() end"#)
            .exec();
        assert!(
            result.is_ok(),
            "field assignment on TalkAction should not error: {result:?}"
        );
    }

    #[test]
    fn talk_action_function_syntax_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local t = TalkAction("/test"); function t.onSay(player, words, param) end"#)
            .exec();
        assert!(
            result.is_ok(),
            "function-declaration syntax on TalkAction should not error: {result:?}"
        );
    }

    #[test]
    fn access_accepts_boolean_true() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local ban = TalkAction("/ban"); ban:access(true)"#)
            .exec();
        assert!(
            result.is_ok(),
            "TalkAction:access(true) should not error: {result:?}"
        );
    }

    #[test]
    fn access_accepts_boolean_false() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local ban = TalkAction("/ban"); ban:access(false)"#)
            .exec();
        assert!(
            result.is_ok(),
            "TalkAction:access(false) should not error: {result:?}"
        );
    }

    #[test]
    fn access_accepts_integer_regression() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"local ban = TalkAction("/ban"); ban:access(1)"#)
            .exec();
        assert!(
            result.is_ok(),
            "TalkAction:access(1) should still work: {result:?}"
        );
    }

    #[test]
    fn talk_action_access_setter_bool_true() {
        let lua = fresh_lua();
        lua.globals()
            .set("t", LuaTalkAction::default())
            .unwrap();
        lua.load(r#"t:access(true)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("t").unwrap();
        let borrowed = ud.borrow::<LuaTalkAction>().unwrap();
        assert_eq!(borrowed.access, 1);
    }

    #[test]
    fn talk_action_separator_setter() {
        let lua = fresh_lua();
        lua.globals()
            .set("t", LuaTalkAction::default())
            .unwrap();
        lua.load(r#"t:separator(";")"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("t").unwrap();
        let borrowed = ud.borrow::<LuaTalkAction>().unwrap();
        assert_eq!(borrowed.separator, ";");
    }

    #[test]
    fn talk_action_register_stores_in_lua_talk_action_store() {
        use crate::lua_bindings::LuaTalkActionStore;
        let lua = mlua::Lua::new();
        let store = LuaTalkActionStore::default();
        lua.set_app_data(store.clone());
        crate::lua_bindings::install_bindings(
            &lua,
            crate::lua_bindings::GameStateHandle::default(),
        )
        .unwrap();
        lua.load(
            r#"
            local t = TalkAction("/ban")
            t:access(true)
            t:register()
        "#,
        )
        .exec()
        .unwrap();
        assert_eq!(store.0.lock().unwrap().len(), 1);
        assert_eq!(store.0.lock().unwrap()[0].word, "/ban");
    }
}
