//! `NetworkMessage:*` Lua binding for `common::networkmessage::NetworkMessage`.
//!
//! Pure byte-buffer ops. Methods needing cross-class types
//! (`addItem`, `addItemId`, `sendToPlayer`) are stub no-ops until
//! the Item / Player bindings ship.

// AUDIT: ClassMethod NetworkMessage:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_common::networkmessage::NetworkMessage;
use mlua::{UserData, UserDataMethods, Value};

use crate::lua_bindings::position::LuaPosition;

#[derive(Debug, Clone)]
pub struct LuaNetworkMessage(pub NetworkMessage);

impl LuaNetworkMessage {
    pub fn new(m: NetworkMessage) -> Self {
        Self(m)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaNetworkMessage {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaNetworkMessage>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaNetworkMessage",
                message: Some("expected NetworkMessage userdata".into()),
            }),
        }
    }
}

impl UserData for LuaNetworkMessage {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, _this, _other: Value| Ok(false));
        // __gc is auto-implemented by mlua for Drop types; no explicit
        // MetaMethod::Gc variant exists in mlua 0.9 — we expose `delete`
        // for explicit cleanup which C++ scripts use.
        methods.add_method("delete", |_, _this, ()| Ok(()));

        // Read ops
        methods.add_method_mut("getByte", |_, this, ()| Ok(this.0.get_u8() as i64));
        methods.add_method_mut("getU16", |_, this, ()| Ok(this.0.get_u16() as i64));
        methods.add_method_mut("getU32", |_, this, ()| Ok(this.0.get_u32() as i64));
        methods.add_method_mut("getU64", |_, this, ()| Ok(this.0.get_u64() as i64));
        methods.add_method_mut("getString", |_, this, ()| {
            let len = this.0.get_u16();
            Ok(this.0.get_string(len))
        });
        methods.add_method_mut("getPosition", |_, this, ()| {
            Ok(LuaPosition(this.0.get_position()))
        });

        // Write ops
        methods.add_method_mut("addByte", |_, this, v: i64| {
            this.0.add_u8(v as u8);
            Ok(())
        });
        methods.add_method_mut("addU16", |_, this, v: i64| {
            this.0.add_u16(v as u16);
            Ok(())
        });
        methods.add_method_mut("addU32", |_, this, v: i64| {
            this.0.add_u32(v as u32);
            Ok(())
        });
        methods.add_method_mut("addU64", |_, this, v: i64| {
            this.0.add_u64(v as u64);
            Ok(())
        });
        methods.add_method_mut("addString", |_, this, s: String| {
            this.0.add_string(&s);
            Ok(())
        });
        methods.add_method_mut(
            "addDouble",
            |_, this, (v, precision): (f64, Option<i64>)| {
                this.0.add_double(v, precision.unwrap_or(4) as u8);
                Ok(())
            },
        );
        methods.add_method_mut("addPosition", |_, this, pos: LuaPosition| {
            this.0.add_position(pos.0);
            Ok(())
        });
        // Stubs: need Item binding (not yet shipped).
        methods.add_method_mut("addItem", |_, _this, _item: Value| Ok(()));
        methods.add_method_mut("addItemId", |_, _this, _item_id: i64| Ok(()));
        // Stub: needs Player binding + send pipeline.
        methods.add_method_mut("sendToPlayer", |_, _this, _player: Value| Ok(()));

        // Cursor / state
        methods.add_method("len", |_, this, ()| Ok(this.0.get_message_length() as i64));
        methods.add_method(
            "tell",
            |_, this, ()| Ok(this.0.get_buffer_position() as i64),
        );
        methods.add_method_mut("seek", |_, this, pos: i64| {
            this.0.set_buffer_position(pos as u16);
            Ok(())
        });
        methods.add_method_mut("skipBytes", |_, this, n: i64| {
            this.0.skip_bytes(n as i16);
            Ok(())
        });
        methods.add_method_mut("reset", |_, this, ()| {
            this.0.reset();
            Ok(())
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
    fn add_byte_then_get_byte_roundtrips() {
        let lua = fresh_lua();
        lua.globals()
            .set("m", LuaNetworkMessage::new(NetworkMessage::new()))
            .unwrap();
        lua.load("m:addByte(42); m:seek(0)").exec().unwrap();
        let v: i64 = lua.load("return m:getByte()").eval().unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn add_u32_then_get_u32_roundtrips() {
        let lua = fresh_lua();
        lua.globals()
            .set("m", LuaNetworkMessage::new(NetworkMessage::new()))
            .unwrap();
        lua.load("m:addU32(123456); m:seek(0)").exec().unwrap();
        let v: i64 = lua.load("return m:getU32()").eval().unwrap();
        assert_eq!(v, 123456);
    }

    #[test]
    fn add_position_round_trips() {
        let lua = fresh_lua();
        lua.globals()
            .set("m", LuaNetworkMessage::new(NetworkMessage::new()))
            .unwrap();
        lua.load("m:addPosition(Position(100, 200, 7)); m:seek(0)")
            .exec()
            .unwrap();
        let (x, y, z): (i64, i64, i64) = lua
            .load("local p = m:getPosition(); return p.x, p.y, p.z")
            .eval()
            .unwrap();
        assert_eq!((x, y, z), (100, 200, 7));
    }

    #[test]
    fn reset_clears_buffer() {
        let lua = fresh_lua();
        lua.globals()
            .set("m", LuaNetworkMessage::new(NetworkMessage::new()))
            .unwrap();
        lua.load("m:addU16(99); m:reset()").exec().unwrap();
        let v: i64 = lua.load("return m:len()").eval().unwrap();
        assert_eq!(v, 0);
    }
}
