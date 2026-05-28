//! `XMLDocument:*` Lua binding (parser-side stub).

// AUDIT: ClassMethod XMLDocument:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use super::xml_node::LuaXmlNode;
use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone, Default)]
pub struct LuaXmlDocument;

impl<'lua> mlua::FromLua<'lua> for LuaXmlDocument {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaXmlDocument>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaXmlDocument",
                message: Some("expected XMLDocument userdata".into()),
            }),
        }
    }
}

impl UserData for LuaXmlDocument {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("child", |lua, _this, _name: String| {
            lua.create_userdata(LuaXmlNode)
        });
        methods.add_method_mut("delete", |_, _this, ()| Ok(()));
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
    fn xml_doc_child_returns_non_nil() {
        let lua = fresh_lua();
        let result: mlua::Result<bool> = lua
            .load(r#"local doc = XMLDocument("test"); local c = doc:child("actions"); return c ~= nil"#)
            .eval();
        assert!(result.is_ok(), "child() should not error: {result:?}");
        assert!(result.unwrap(), "child() should return non-nil XMLNode");
    }

    #[test]
    fn xml_doc_child_children_iterable() {
        let lua = fresh_lua();
        let result = lua
            .load(
                r#"
                local doc = XMLDocument("test")
                local actions = doc:child("actions")
                local count = 0
                for node in actions:children() do
                    count = count + 1
                end
                return count
            "#,
            )
            .eval::<i64>();
        assert!(
            result.is_ok(),
            "iterating children() should not error: {result:?}"
        );
        assert_eq!(
            result.unwrap(),
            0,
            "stub children() should yield zero items"
        );
    }
}
