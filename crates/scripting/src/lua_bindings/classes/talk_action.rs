//! `TalkAction:*` Lua binding.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaTalkAction;

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
        methods.add_method_mut("access", |_, _this, _v: Value| Ok(()));
        methods.add_method_mut("accountType", |_, _this, _v: i64| Ok(()));
        methods.add_method_mut("separator", |_, _this, _v: String| Ok(()));
        methods.add_method_mut("onSay", |_, _this, _cb: Value| Ok(()));
        methods.add_method_mut("register", |_, _this, ()| Ok(true));
        methods.add_meta_method_mut("__newindex", |_, _this, (_k, _v): (Value, Value)| Ok(()));
    }
}

#[cfg(test)]
mod tests {
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
}
