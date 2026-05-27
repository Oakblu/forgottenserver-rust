//! `Npc:*` Lua binding for `entity::npc::Npc`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_entity::npc::Npc;
use mlua::{UserData, UserDataMethods, Value};
use std::sync::{Arc, Mutex};

pub struct LuaNpc(pub Arc<Mutex<Npc>>);

impl LuaNpc {
    pub fn new(n: Npc) -> Self {
        Self(Arc::new(Mutex::new(n)))
    }
}

impl Clone for LuaNpc {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaNpc {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaNpc>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaNpc",
                message: Some("expected Npc userdata".into()),
            }),
        }
    }
}

impl UserData for LuaNpc {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaNpc| {
            Ok(Arc::ptr_eq(&this.0, &other.0))
        });
        // ── Stubs (need game-state + parser plumbing) ────────────
        for n in &[
            "setMasterPos",
            "isNpc",
            "getSpectators",
            "getSpeechBubble",
            "setSpeechBubble",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(()));
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
    fn npc_userdata_round_trips() {
        // Npc inherits everything from Creature in C++, so no method
        // names need to live directly on Npc. We just confirm the
        // userdata install is wired and registers without panicking.
        let lua = fresh_lua();
        let n = Npc::new("Banker");
        lua.globals().set("n", LuaNpc::new(n)).unwrap();
        // The stubs cover side-effecting setters; ensure one is callable.
        lua.load("n:setMasterPos({x=1,y=2,z=3})").exec().unwrap();
    }
}
