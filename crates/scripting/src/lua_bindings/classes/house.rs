//! `House:*` Lua binding for `world::house::House`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_world::house::House;
use mlua::{UserData, UserDataMethods, Value};
use std::sync::{Arc, Mutex};

pub struct LuaHouse(pub Arc<Mutex<House>>);

impl LuaHouse {
    pub fn new(h: House) -> Self {
        Self(Arc::new(Mutex::new(h)))
    }
}

impl Clone for LuaHouse {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaHouse {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaHouse>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaHouse",
                message: Some("expected House userdata".into()),
            }),
        }
    }
}

impl UserData for LuaHouse {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaHouse| {
            Ok(Arc::ptr_eq(&this.0, &other.0))
        });
        methods.add_method("getId", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_id() as i64)
        });
        methods.add_method("getName", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_name().to_string())
        });
        methods.add_method("getRent", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_rent() as i64)
        });
        methods.add_method_mut("setRent", |_, this, r: i64| {
            this.0.lock().unwrap().set_rent(r.max(0) as u32);
            Ok(())
        });
        methods.add_method("getTown", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_town_id() as i64)
        });
        methods.add_method("getExitPosition", |lua, this, ()| {
            let p = this.0.lock().unwrap().get_entry_pos();
            let t = lua.create_table()?;
            t.set("x", p.x as i64)?;
            t.set("y", p.y as i64)?;
            t.set("z", p.z as i64)?;
            Ok(t)
        });
        methods.add_method("getOwnerGuid", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_owner_guid() as i64)
        });
        methods.add_method("getOwnerName", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_owner_name().to_string())
        });
        methods.add_method_mut("setOwnerGuid", |_, this, guid: i64| {
            this.0.lock().unwrap().set_owner(guid.max(0) as u32);
            Ok(())
        });
        methods.add_method("getTileCount", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_tile_count() as i64)
        });
        methods.add_method("getBedCount", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_bed_count() as i64)
        });
        methods.add_method("getDoorCount", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_door_count() as i64)
        });
        methods.add_method("getPaidUntil", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_paid_until())
        });
        methods.add_method_mut("setPaidUntil", |_, this, ts: i64| {
            this.0.lock().unwrap().set_paid_until(ts);
            Ok(())
        });
        methods.add_method("getPayRentWarnings", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_rent_warnings() as i64)
        });
        methods.add_method_mut("setPayRentWarnings", |_, this, w: i64| {
            this.0.lock().unwrap().set_rent_warnings(w.max(0) as u32);
            Ok(())
        });
        methods.add_method("kickPlayer", |_, _this, _args: Value| Ok(false));
        methods.add_method("canEditAccessList", |_, _this, _args: Value| Ok(false));
        methods.add_method("getDoorIdByPosition", |_, _this, _args: Value| Ok(0i64));
        methods.add_method("startTrade", |_, _this, _args: Value| Ok(false));
        // ── Stubs (need game-state plumbing) ─────────────────────
        for n in &[
            "getTiles",
            "getItems",
            "getBeds",
            "getDoors",
            "getAccessList",
            "setAccessList",
            "save",
        ] {
            methods.add_method(n, |lua, _this, _args: Value| lua.create_table());
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
    fn round_trip_name_and_rent() {
        let lua = fresh_lua();
        let h = House::new(1, "Lighthouse", 500, 1);
        lua.globals().set("h", LuaHouse::new(h)).unwrap();
        let n: String = lua.load("return h:getName()").eval().unwrap();
        assert_eq!(n, "Lighthouse");
        let r: i64 = lua
            .load("h:setRent(750); return h:getRent()")
            .eval()
            .unwrap();
        assert_eq!(r, 750);
    }
}
