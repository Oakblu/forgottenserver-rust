//! `XMLDocument:*` Lua binding (parser-side stub).

// AUDIT: ClassMethod XMLDocument:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

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
        methods.add_method("child", |_, _this, _name: String| Ok(Value::Nil));
        methods.add_method_mut("delete", |_, _this, ()| Ok(()));
    }
}
