//! `Party:*` Lua binding for `entity::party::Party`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_entity::party::Party;
use mlua::{UserData, UserDataMethods, Value};
use std::sync::{Arc, Mutex};

pub struct LuaParty(pub Arc<Mutex<Party>>);

impl LuaParty {
    pub fn new(p: Party) -> Self {
        Self(Arc::new(Mutex::new(p)))
    }
}

impl Clone for LuaParty {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaParty {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaParty>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaParty",
                message: Some("expected Party userdata".into()),
            }),
        }
    }
}

impl UserData for LuaParty {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaParty| {
            Ok(Arc::ptr_eq(&this.0, &other.0))
        });
        methods.add_method("getLeader", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_leader_guid().unwrap_or(0) as i64)
        });
        methods.add_method_mut("setLeader", |_, this, guid: i64| {
            Ok(this
                .0
                .lock()
                .unwrap()
                .pass_party_leadership(guid.max(0) as u32))
        });
        methods.add_method("getMemberCount", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_member_count() as i64)
        });
        methods.add_method("getMembers", |lua, this, ()| {
            let t = lua.create_table()?;
            for (i, guid) in this.0.lock().unwrap().get_members().iter().enumerate() {
                t.set(i as i64 + 1, *guid as i64)?;
            }
            Ok(t)
        });
        methods.add_method_mut("addInvite", |_, this, guid: i64| {
            Ok(this.0.lock().unwrap().invite_player(guid.max(0) as u32))
        });
        methods.add_method_mut("removeInvite", |_, this, guid: i64| {
            Ok(this.0.lock().unwrap().revoke_invitation(guid.max(0) as u32))
        });
        methods.add_method_mut("addMember", |_, this, guid: i64| {
            this.0.lock().unwrap().join_party(guid.max(0) as u32);
            Ok(true)
        });
        methods.add_method_mut("removeMember", |_, this, guid: i64| {
            Ok(this.0.lock().unwrap().leave_party(guid.max(0) as u32))
        });
        methods.add_method("isSharedExperienceActive", |_, this, ()| {
            Ok(this.0.lock().unwrap().is_shared_experience_active())
        });
        methods.add_method("isSharedExperienceEnabled", |_, this, ()| {
            Ok(this.0.lock().unwrap().is_shared_experience_enabled())
        });
        methods.add_method_mut("setSharedExperience", |_, this, on: bool| {
            Ok(this.0.lock().unwrap().set_shared_experience(on))
        });
        methods.add_method_mut("disband", |_, this, ()| {
            this.0.lock().unwrap().disband();
            Ok(true)
        });
        // ── Stubs (need plugin access to live world state) ───────
        for n in &["shareExperience", "isMemberSharingExp"] {
            methods.add_method(n, |_, _this, _args: Value| Ok(false));
        }
        methods.add_method("getInviteeCount", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_invitation_count() as i64)
        });
        methods.add_method("getInvitees", |lua, _this, ()| lua.create_table());
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
    fn leader_returns_field() {
        let lua = fresh_lua();
        let p = Party::new(7);
        lua.globals().set("p", LuaParty::new(p)).unwrap();
        let g: i64 = lua.load("return p:getLeader()").eval().unwrap();
        assert_eq!(g, 7);
    }

    #[test]
    fn invite_then_count() {
        let lua = fresh_lua();
        let p = Party::new(1);
        lua.globals().set("p", LuaParty::new(p)).unwrap();
        let n: i64 = lua
            .load("p:addInvite(99); return p:getInviteeCount()")
            .eval()
            .unwrap();
        assert_eq!(n, 1);
    }
}
