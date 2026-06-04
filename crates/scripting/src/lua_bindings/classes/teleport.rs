//! `Teleport:*` Lua binding for `map::teleport::Teleport`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_common::position::Position;
use forgottenserver_map::teleport::Teleport;
use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone)]
pub struct LuaTeleport(pub Teleport);

impl LuaTeleport {
    pub fn new(t: Teleport) -> Self {
        Self(t)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaTeleport {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaTeleport>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaTeleport",
                message: Some("expected Teleport userdata".into()),
            }),
        }
    }
}

impl UserData for LuaTeleport {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaTeleport| {
            Ok(this.0.item_type_id == other.0.item_type_id)
        });
        methods.add_method("getDestination", |lua, this, ()| {
            let p = this.0.get_dest_pos();
            let t = lua.create_table()?;
            t.set("x", p.x as i64)?;
            t.set("y", p.y as i64)?;
            t.set("z", p.z as i64)?;
            Ok(t)
        });
        methods.add_method_mut("setDestination", |_, this, dest: mlua::Table| {
            let x: i64 = dest.get("x").unwrap_or(0);
            let y: i64 = dest.get("y").unwrap_or(0);
            let z: i64 = dest.get("z").unwrap_or(0);
            this.0
                .set_dest_pos(Position::new(x as u16, y as u16, z as u8));
            Ok(())
        });
        // `isTeleport` should always be true on this userdata
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
    fn round_trip_destination() {
        let lua = fresh_lua();
        let t = Teleport::new(1387);
        lua.globals().set("tp", LuaTeleport::new(t)).unwrap();
        let z: i64 = lua
            .load("tp:setDestination({x=100, y=200, z=7}); return tp:getDestination().z")
            .eval()
            .unwrap();
        assert_eq!(z, 7);
    }

    #[test]
    fn get_destination_x_y_z_fields() {
        let lua = fresh_lua();
        let mut t = Teleport::new(1387);
        t.set_dest_pos(Position::new(50, 60, 3));
        lua.globals().set("tp", LuaTeleport::new(t)).unwrap();
        let x: i64 = lua.load("return tp:getDestination().x").eval().unwrap();
        let y: i64 = lua.load("return tp:getDestination().y").eval().unwrap();
        let z: i64 = lua.load("return tp:getDestination().z").eval().unwrap();
        assert_eq!(x, 50);
        assert_eq!(y, 60);
        assert_eq!(z, 3);
    }

    #[test]
    fn set_destination_defaults_missing_fields() {
        let lua = fresh_lua();
        let t = Teleport::new(1387);
        lua.globals().set("tp", LuaTeleport::new(t)).unwrap();
        // Only x provided; y and z should default to 0
        let result: mlua::Result<()> = lua
            .load("tp:setDestination({x=100})")
            .exec();
        assert!(result.is_ok());
        let y: i64 = lua.load("return tp:getDestination().y").eval().unwrap();
        assert_eq!(y, 0);
    }

    #[test]
    fn eq_same_type_id() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaTeleport::new(Teleport::new(100)))
            .unwrap();
        lua.globals()
            .set("b", LuaTeleport::new(Teleport::new(100)))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn eq_different_type_id() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaTeleport::new(Teleport::new(100)))
            .unwrap();
        lua.globals()
            .set("b", LuaTeleport::new(Teleport::new(200)))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaTeleport> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
