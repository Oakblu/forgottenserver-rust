//! `ModalWindow:*` Lua binding (in-memory; dispatch is server-side).
//!
//! Mirrors the C++ `ModalWindow` builder API used by Lua to assemble a window
//! before sending it. No game-state plumbing is required to construct one.

// AUDIT: ClassMethod ModalWindow:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaModalWindow {
    pub id: u32,
    pub title: String,
    pub message: String,
    pub priority: bool,
    pub default_enter: u32,
    pub default_escape: u32,
    pub buttons: Vec<(u32, String)>,
    pub choices: Vec<(u32, String)>,
}

impl LuaModalWindow {
    pub fn new(id: u32, title: String, message: String) -> Self {
        Self {
            id,
            title,
            message,
            ..Default::default()
        }
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaModalWindow {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaModalWindow>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaModalWindow",
                message: Some("expected ModalWindow userdata".into()),
            }),
        }
    }
}

impl UserData for LuaModalWindow {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaModalWindow| {
            Ok(this.id == other.id)
        });
        methods.add_method("getId", |_, this, ()| Ok(this.id as i64));
        methods.add_method("getTitle", |_, this, ()| Ok(this.title.clone()));
        methods.add_method("getMessage", |_, this, ()| Ok(this.message.clone()));
        methods.add_method_mut("setTitle", |_, this, t: String| {
            this.title = t;
            Ok(())
        });
        methods.add_method_mut("setMessage", |_, this, m: String| {
            this.message = m;
            Ok(())
        });
        methods.add_method(
            "getButtonCount",
            |_, this, ()| Ok(this.buttons.len() as i64),
        );
        methods.add_method(
            "getChoiceCount",
            |_, this, ()| Ok(this.choices.len() as i64),
        );
        methods.add_method_mut("addButton", |_, this, (id, text): (i64, String)| {
            this.buttons.push((id.max(0) as u32, text));
            Ok(())
        });
        methods.add_method_mut("addChoice", |_, this, (id, text): (i64, String)| {
            this.choices.push((id.max(0) as u32, text));
            Ok(())
        });
        methods.add_method("hasPriority", |_, this, ()| Ok(this.priority));
        methods.add_method_mut("setPriority", |_, this, p: bool| {
            this.priority = p;
            Ok(())
        });
        methods.add_method("getDefaultEnterButton", |_, this, ()| {
            Ok(this.default_enter as i64)
        });
        methods.add_method_mut("setDefaultEnterButton", |_, this, id: i64| {
            this.default_enter = id.max(0) as u32;
            Ok(())
        });
        // `sendToPlayer` needs game-state plumbing; pure stub for now.
        methods.add_method("sendToPlayer", |_, _this, _args: Value| Ok(false));
        methods.add_method_mut("delete", |_, _this, ()| Ok(()));
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
    fn add_buttons_and_choices() {
        let lua = fresh_lua();
        lua.globals()
            .set(
                "w",
                LuaModalWindow::new(1, "Choose".into(), "Pick a color".into()),
            )
            .unwrap();
        let counts: (i64, i64) = lua
            .load(
                r#"
                w:addButton(1, "Red")
                w:addButton(2, "Blue")
                w:addChoice(10, "A")
                return w:getButtonCount(), w:getChoiceCount()
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(counts, (2, 1));
    }
}
