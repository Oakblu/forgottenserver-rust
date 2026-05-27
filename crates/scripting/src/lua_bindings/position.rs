//! Lua binding for the `Position` class.
//!
//! Wraps `forgottenserver_common::position::Position` in `LuaPosition`
//! (newtype required by Rust orphan rules: both `mlua::UserData` and
//! `Position` are foreign to the scripting crate).
//!
//! Surface registered:
//!   - Global constructor: `Position(x, y, z)` returns userdata.
//!   - Fields: `pos.x`, `pos.y`, `pos.z` (read + write).
//!   - Methods (matching C++ `registerMethod(L, "Position", …)`):
//!     - `pos:isSightClear(other, floorCheck?)` — **stub** returning
//!       `true` (C++ default). Will be implemented once the Map
//!       binding lands and the scripting crate can resolve Map from
//!       app_data.
//!     - `pos:sendMagicEffect(effect, player?)` — **stub** returning
//!       `true`. Needs spectator broadcast.
//!     - `pos:sendDistanceEffect(target, effect, player?)` — **stub**
//!       returning `true`. Needs spectator broadcast.
//!
//! Stub methods document the call signature so existing `data/*.lua`
//! scripts that invoke them parse and run without erroring. The
//! divergence (no real effect dispatched) is recorded in
//! `MIGRATION_LEDGER.yml`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_common::position::Position;
use mlua::{Lua, UserData, UserDataFields, UserDataMethods, Value};

use crate::lua_bindings::GameStateHandle;

/// Newtype wrapper for `common::Position` that satisfies mlua's
/// orphan-rule constraints. Public so future bindings (Tile, Creature,
/// etc.) can convert between Position and LuaPosition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LuaPosition(pub Position);

impl LuaPosition {
    pub fn new(x: u16, y: u16, z: u8) -> Self {
        Self(Position::new(x, y, z))
    }

    pub fn into_inner(self) -> Position {
        self.0
    }
}

impl From<Position> for LuaPosition {
    fn from(p: Position) -> Self {
        Self(p)
    }
}

impl From<LuaPosition> for Position {
    fn from(p: LuaPosition) -> Self {
        p.0
    }
}

/// Allow `LuaPosition` to be received as a method argument or
/// returned from `eval` by extracting the wrapped value out of any
/// `LuaPosition` userdata. Falls back to a typed error for anything
/// else.
impl<'lua> mlua::FromLua<'lua> for LuaPosition {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => {
                let borrowed = ud.borrow::<LuaPosition>()?;
                Ok(*borrowed)
            }
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaPosition",
                message: Some("expected Position userdata".into()),
            }),
        }
    }
}

impl UserData for LuaPosition {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.0.x));
        fields.add_field_method_get("y", |_, this| Ok(this.0.y));
        fields.add_field_method_get("z", |_, this| Ok(this.0.z));
        fields.add_field_method_set("x", |_, this, v: u16| {
            this.0.x = v;
            Ok(())
        });
        fields.add_field_method_set("y", |_, this, v: u16| {
            this.0.y = v;
            Ok(())
        });
        fields.add_field_method_set("z", |_, this, v: u8| {
            this.0.z = v;
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // ── isSightClear ──────────────────────────────────────────────
        // C++: bool isSightClear(Position to, bool floorCheck)
        // Stub returns true (the C++ default for unobstructed paths).
        // Real impl needs Map::checkSightLine — wired when the Map
        // binding lands (forgottenserver-rust-lua-bindings-class-map).
        methods.add_method(
            "isSightClear",
            |lua, _this, (_other, _floor_check): (LuaPosition, Option<bool>)| {
                let _ = lua.app_data_ref::<GameStateHandle>();
                Ok(true)
            },
        );

        // ── sendMagicEffect ───────────────────────────────────────────
        // C++: bool sendMagicEffect(MagicEffectClasses effect, Player* player?)
        // Stub returns true. Real impl needs g_game.addMagicEffect +
        // spectator broadcast. Wired when the broadcast subsystem is
        // reachable from scripting (likely with the Creature class
        // follow-up).
        methods.add_method(
            "sendMagicEffect",
            |lua, _this, (_effect, _player): (u16, Option<Value>)| {
                let _ = lua.app_data_ref::<GameStateHandle>();
                Ok(true)
            },
        );

        // ── sendDistanceEffect ────────────────────────────────────────
        // C++: bool sendDistanceEffect(Position to, ShootType_t effect, Player* player?)
        // Stub returns true. Same backing requirement as
        // sendMagicEffect.
        methods.add_method(
            "sendDistanceEffect",
            |lua, _this, (_target, _effect, _player): (LuaPosition, u16, Option<Value>)| {
                let _ = lua.app_data_ref::<GameStateHandle>();
                Ok(true)
            },
        );

        // ── tostring meta-method ──────────────────────────────────────
        // Convenience for Lua debug prints; matches C++'s implicit
        // Position::toString output shape.
        methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, ()| {
            Ok(format!(
                "Position({}, {}, {})",
                this.0.x, this.0.y, this.0.z
            ))
        });
    }
}

/// Register the `Position` global constructor onto the Lua state.
pub fn install(lua: &Lua) -> mlua::Result<()> {
    let ctor = lua.create_function(|_, (x, y, z): (u16, u16, u8)| Ok(LuaPosition::new(x, y, z)))?;
    lua.globals().set("Position", ctor)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_bindings::install_bindings;

    fn fresh_lua() -> mlua::Lua {
        let lua = mlua::Lua::new();
        install_bindings(&lua, GameStateHandle::default()).unwrap();
        lua
    }

    // ── Constructor + field reads ────────────────────────────────────────────

    #[test]
    fn constructor_creates_userdata() {
        let lua = fresh_lua();
        let pos: LuaPosition = lua.load("return Position(100, 200, 7)").eval().unwrap();
        assert_eq!(pos.0.x, 100);
        assert_eq!(pos.0.y, 200);
        assert_eq!(pos.0.z, 7);
    }

    #[test]
    fn field_read_x_returns_constructor_value() {
        let lua = fresh_lua();
        let x: u16 = lua.load("return Position(42, 7, 5).x").eval().unwrap();
        assert_eq!(x, 42);
    }

    #[test]
    fn field_read_y_returns_constructor_value() {
        let lua = fresh_lua();
        let y: u16 = lua.load("return Position(42, 7, 5).y").eval().unwrap();
        assert_eq!(y, 7);
    }

    #[test]
    fn field_read_z_returns_constructor_value() {
        let lua = fresh_lua();
        let z: u8 = lua.load("return Position(42, 7, 5).z").eval().unwrap();
        assert_eq!(z, 5);
    }

    // ── Field writes ─────────────────────────────────────────────────────────

    #[test]
    fn field_write_x_mutates_userdata() {
        let lua = fresh_lua();
        let x: u16 = lua
            .load("local p = Position(1, 2, 3); p.x = 999; return p.x")
            .eval()
            .unwrap();
        assert_eq!(x, 999);
    }

    #[test]
    fn field_write_y_mutates_userdata() {
        let lua = fresh_lua();
        let y: u16 = lua
            .load("local p = Position(1, 2, 3); p.y = 555; return p.y")
            .eval()
            .unwrap();
        assert_eq!(y, 555);
    }

    #[test]
    fn field_write_z_mutates_userdata() {
        let lua = fresh_lua();
        let z: u8 = lua
            .load("local p = Position(1, 2, 3); p.z = 14; return p.z")
            .eval()
            .unwrap();
        assert_eq!(z, 14);
    }

    // ── Stub methods ─────────────────────────────────────────────────────────

    #[test]
    fn is_sight_clear_stub_returns_true() {
        let lua = fresh_lua();
        let v: bool = lua
            .load("return Position(0,0,7):isSightClear(Position(5,5,7))")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn is_sight_clear_accepts_optional_floor_check_arg() {
        let lua = fresh_lua();
        let v: bool = lua
            .load("return Position(0,0,7):isSightClear(Position(5,5,7), true)")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn send_magic_effect_stub_returns_true() {
        let lua = fresh_lua();
        let v: bool = lua
            .load("return Position(0,0,7):sendMagicEffect(5)")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn send_distance_effect_stub_returns_true() {
        let lua = fresh_lua();
        let v: bool = lua
            .load("return Position(0,0,7):sendDistanceEffect(Position(5,5,7), 3)")
            .eval()
            .unwrap();
        assert!(v);
    }

    // ── ToString meta-method ─────────────────────────────────────────────────

    #[test]
    fn tostring_formats_position_with_xyz() {
        let lua = fresh_lua();
        let s: String = lua
            .load("return tostring(Position(100, 200, 7))")
            .eval()
            .unwrap();
        assert_eq!(s, "Position(100, 200, 7)");
    }

    // ── Conversion helpers ───────────────────────────────────────────────────

    #[test]
    fn lua_position_into_inner_returns_common_position() {
        let lp = LuaPosition::new(10, 20, 3);
        let p: Position = lp.into_inner();
        assert_eq!(p, Position::new(10, 20, 3));
    }

    #[test]
    fn from_position_for_lua_position_round_trip() {
        let p = Position::new(7, 8, 9);
        let lp: LuaPosition = p.into();
        let back: Position = lp.into();
        assert_eq!(back, p);
    }

    // ── Argument-type passing ────────────────────────────────────────────────

    #[test]
    fn methods_accept_two_position_args() {
        let lua = fresh_lua();
        // sendDistanceEffect takes (target_pos, effect_id) — proves
        // LuaPosition userdata can be passed as a method argument.
        let v: bool = lua
            .load(
                "local from = Position(0,0,7)\n\
                 local to   = Position(10,10,7)\n\
                 return from:sendDistanceEffect(to, 3)",
            )
            .eval()
            .unwrap();
        assert!(v);
    }
}
