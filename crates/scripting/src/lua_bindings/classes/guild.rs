//! `Guild:*` Lua binding for `entity::guild::Guild`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_entity::guild::Guild;
use mlua::{UserData, UserDataMethods, Value};

pub struct LuaGuild(pub std::sync::Arc<std::sync::Mutex<Guild>>);

impl LuaGuild {
    pub fn new(g: Guild) -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(g)))
    }
}

impl Clone for LuaGuild {
    fn clone(&self) -> Self {
        Self(std::sync::Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaGuild {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaGuild>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaGuild",
                message: Some("expected Guild userdata".into()),
            }),
        }
    }
}

impl UserData for LuaGuild {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaGuild| {
            Ok(std::sync::Arc::ptr_eq(&this.0, &other.0))
        });

        methods.add_method("getId", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_id() as i64)
        });
        methods.add_method("getName", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_name().to_string())
        });
        methods.add_method("getMotd", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_motd().to_string())
        });
        methods.add_method_mut("setMotd", |_, this, motd: String| {
            this.0.lock().unwrap().set_motd(motd);
            Ok(())
        });
        methods.add_method("getMembersOnline", |lua, _this, ()| lua.create_table());
        // Stubs for rank-table access (need GuildRank userdata)
        methods.add_method("getRankById", |_, _this, _id: i64| Ok(Value::Nil));
        methods.add_method("getRankByLevel", |_, _this, _lvl: i64| Ok(Value::Nil));
        methods.add_method_mut("addRank", |_, _this, _args: (i64, String, i64)| Ok(false));
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
    fn get_id_returns_field() {
        let lua = fresh_lua();
        let g = Guild::new(7, "Knights of the Round".to_string());
        lua.globals().set("g", LuaGuild::new(g)).unwrap();
        let v: i64 = lua.load("return g:getId()").eval().unwrap();
        assert_eq!(v, 7);
    }

    #[test]
    fn motd_round_trips_via_set_then_get() {
        let lua = fresh_lua();
        let g = Guild::new(1, "X".to_string());
        lua.globals().set("g", LuaGuild::new(g)).unwrap();
        let s: String = lua
            .load("g:setMotd('hello'); return g:getMotd()")
            .eval()
            .unwrap();
        assert_eq!(s, "hello");
    }
}
