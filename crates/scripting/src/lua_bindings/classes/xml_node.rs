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
        methods.add_method("children", |lua, _this, ()| {
            // Return an empty iterator function: `for x in node:children() do` loops zero times
            let f = lua.create_function(|_, _: ()| -> mlua::Result<mlua::Value> {
                Ok(mlua::Value::Nil)
            })?;
            Ok(mlua::Value::Function(f))
        });
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
    fn name_returns_empty_string() {
        let lua = fresh_lua();
        lua.globals().set("n", LuaXmlNode).unwrap();
        let v: String = lua.load("return n:name()").eval().unwrap();
        assert_eq!(v, "");
    }

    #[test]
    fn attribute_returns_empty_string() {
        let lua = fresh_lua();
        lua.globals().set("n", LuaXmlNode).unwrap();
        let v: String = lua.load("return n:attribute('id')").eval().unwrap();
        assert_eq!(v, "");
    }

    #[test]
    fn first_child_returns_nil() {
        let lua = fresh_lua();
        lua.globals().set("n", LuaXmlNode).unwrap();
        let v: mlua::Value = lua.load("return n:firstChild()").eval().unwrap();
        assert!(matches!(v, mlua::Value::Nil));
    }

    #[test]
    fn next_sibling_returns_nil() {
        let lua = fresh_lua();
        lua.globals().set("n", LuaXmlNode).unwrap();
        let v: mlua::Value = lua.load("return n:nextSibling()").eval().unwrap();
        assert!(matches!(v, mlua::Value::Nil));
    }

    #[test]
    fn delete_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("n", LuaXmlNode).unwrap();
        let result: mlua::Result<()> = lua.load("n:delete()").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn children_returns_iterator_that_loops_zero_times() {
        let lua = fresh_lua();
        lua.globals().set("n", LuaXmlNode).unwrap();
        let count: i64 = lua
            .load("local c = 0; for _ in n:children() do c = c + 1 end; return c")
            .eval()
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaXmlNode> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
