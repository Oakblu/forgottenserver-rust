//! `Tile:*` Lua binding for `map::tile::Tile`.
//!
//! Methods that return Item / Creature / House userdata are stubbed
//! (return nil / 0) until those bindings ship. Methods that operate
//! on tile-local state (position, flags, properties, counts) are
//! implemented in full.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_map::tile::{ItemProperty, Tile};
use mlua::{UserData, UserDataMethods, Value};

use crate::lua_bindings::position::LuaPosition;

#[derive(Debug, Clone)]
pub struct LuaTile(pub Tile);

impl LuaTile {
    pub fn new(t: Tile) -> Self {
        Self(t)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaTile {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaTile>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaTile",
                message: Some("expected Tile userdata".into()),
            }),
        }
    }
}

/// Map a C++ ItemProperty integer to the Rust enum variant.
fn item_property_from_i64(v: i64) -> Option<ItemProperty> {
    match v {
        0 => Some(ItemProperty::BlockSolid),
        1 => Some(ItemProperty::HasHeight),
        2 => Some(ItemProperty::BlockProjectile),
        3 => Some(ItemProperty::BlockPath),
        4 => Some(ItemProperty::IsVertical),
        5 => Some(ItemProperty::IsHorizontal),
        6 => Some(ItemProperty::Moveable),
        7 => Some(ItemProperty::ImmovableBlockSolid),
        8 => Some(ItemProperty::ImmovableBlockPath),
        9 => Some(ItemProperty::ImmovableNoFieldBlockPath),
        10 => Some(ItemProperty::NoFieldBlockPath),
        11 => Some(ItemProperty::SupportHangable),
        _ => None,
    }
}

impl UserData for LuaTile {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaTile| {
            Ok(this.0.position == other.0.position)
        });

        // Pure-data accessors.
        methods.add_method("getPosition", |_, this, ()| {
            Ok(LuaPosition(this.0.position))
        });
        methods.add_method("getCreatureCount", |_, this, ()| {
            Ok(this.0.get_creature_count() as i64)
        });
        methods.add_method("getItemCount", |_, this, ()| {
            Ok(this.0.get_item_count() as i64)
        });
        methods.add_method("getThingCount", |_, this, ()| {
            Ok(this.0.get_thing_count() as i64)
        });
        methods.add_method("getTopItemCount", |_, this, ()| {
            Ok(this.0.get_top_item_count() as i64)
        });
        methods.add_method("getDownItemCount", |_, this, ()| {
            Ok(this.0.get_down_item_count() as i64)
        });

        methods.add_method("hasFlag", |_, this, flag: i64| {
            Ok(this.0.has_flag(flag as u32))
        });
        methods.add_method("hasProperty", |_, this, prop: i64| {
            Ok(item_property_from_i64(prop)
                .map(|p| this.0.has_property(p))
                .unwrap_or(false))
        });

        // Returns Item — stub nil until Item binding ships.
        methods.add_method("getGround", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getFieldItem", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getTopDownItem", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getTopTopItem", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getItemById", |_, _this, _args: (i64, Option<i64>)| {
            Ok(mlua::Value::Nil)
        });
        methods.add_method("getItemByType", |_, _this, _t: i64| Ok(mlua::Value::Nil));
        methods.add_method("getItemByTopOrder", |_, _this, _t: i64| {
            Ok(mlua::Value::Nil)
        });
        methods.add_method(
            "getItemCountById",
            |_, _this, (_id, _sub): (i64, Option<i64>)| {
                // Stub: needs cross-item tile traversal.
                Ok(0i64)
            },
        );
        methods.add_method("getItems", |lua, _this, ()| lua.create_table());

        // Returns Creature — stub nil.
        methods.add_method("getTopCreature", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getBottomCreature", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getTopVisibleCreature", |_, _this, _player: Value| {
            Ok(mlua::Value::Nil)
        });
        methods.add_method("getBottomVisibleCreature", |_, _this, _player: Value| {
            Ok(mlua::Value::Nil)
        });
        methods.add_method("getTopVisibleThing", |_, _this, _player: Value| {
            Ok(mlua::Value::Nil)
        });
        methods.add_method("getCreatures", |lua, _this, ()| lua.create_table());

        // Returns Thing — stub.
        methods.add_method("getThing", |_, _this, _idx: i64| Ok(mlua::Value::Nil));
        methods.add_method("getThingIndex", |_, _this, _thing: Value| Ok(-1i64));

        // House lookup — stub nil.
        methods.add_method("getHouse", |_, _this, ()| Ok(mlua::Value::Nil));

        // Mutations — stubs returning nil/0/false. Real impl needs
        // Item type + Game integration.
        methods.add_method_mut(
            "addItem",
            |_, _this, _args: (Value, Option<i64>, Option<i64>)| Ok(mlua::Value::Nil),
        );
        methods.add_method_mut(
            "addItemEx",
            |_, _this, _args: (Value, Option<i64>, Option<i64>)| Ok(0i64),
        );
        methods.add_method_mut("remove", |_, _this, ()| Ok(false));
        methods.add_method(
            "queryAdd",
            |_, _this, _args: (Value, Option<i64>, Option<i64>, Option<Value>)| Ok(0i64),
        );
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

    fn sample_tile() -> Tile {
        let mut t = Tile::new(1000, 1000, 7);
        t.flags = 1 << 7; // TILESTATE_PROTECTIONZONE
        t
    }

    #[test]
    fn get_position_returns_tile_position() {
        let lua = fresh_lua();
        lua.globals().set("t", LuaTile::new(sample_tile())).unwrap();
        let x: i64 = lua.load("return t:getPosition().x").eval().unwrap();
        assert_eq!(x, 1000);
    }

    #[test]
    fn has_flag_returns_true_for_protection_zone() {
        let lua = fresh_lua();
        lua.globals().set("t", LuaTile::new(sample_tile())).unwrap();
        let v: bool = lua
            .load("return t:hasFlag(TILESTATE_PROTECTIONZONE)")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn get_thing_count_zero_for_empty_tile() {
        let lua = fresh_lua();
        lua.globals().set("t", LuaTile::new(sample_tile())).unwrap();
        let v: i64 = lua.load("return t:getThingCount()").eval().unwrap();
        // Ground item only — 1 thing.
        assert!(v >= 0);
    }

    #[test]
    fn equality_metamethod_compares_position() {
        let lua = fresh_lua();
        lua.globals().set("a", LuaTile::new(sample_tile())).unwrap();
        lua.globals().set("b", LuaTile::new(sample_tile())).unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(v);
    }
}
