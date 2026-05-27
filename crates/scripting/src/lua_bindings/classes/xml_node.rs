//! `XMLNode:*` Lua binding (parser-side stub — used by Lua XML helpers).

// AUDIT: ClassMethod XMLNode:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaXmlNode;

impl<'lua> mlua::FromLua<'lua> for LuaXmlNode {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaXmlNode>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaXmlNode",
                message: Some("expected XMLNode userdata".into()),
            }),
        }
    }
}

impl UserData for LuaXmlNode {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Short-form readers (mirror C++ `XMLNode:name`/`:attribute`/etc.).
        methods.add_method("name", |_, _this, ()| Ok(String::new()));
        methods.add_method("attribute", |_, _this, _name: String| Ok(String::new()));
        methods.add_method("firstChild", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method("nextSibling", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method_mut("delete", |_, _this, ()| Ok(()));
    }
}
