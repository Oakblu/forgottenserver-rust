//! `Game:*` Lua binding (singleton).
//!
//! Mirrors the C++ `Game::lua*` family. Most calls need the live world
//! state, which the scripting crate cannot reach directly — these are
//! stubs returning C++ defaults (`nil`, `0`, empty tables) until the
//! game-state plumbing is extended through the binding install.
//!
//! Registered into Lua as the global `Game` so Lua code keeps writing
//! `Game.getPlayers()` exactly like in C++ (mlua exposes it as
//! `Game:getPlayers()` either way, since `Game` is userdata).

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Copy, Default)]
pub struct LuaGame;

impl<'lua> mlua::FromLua<'lua> for LuaGame {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<LuaGame>()?),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaGame",
                message: Some("expected Game userdata".into()),
            }),
        }
    }
}

impl UserData for LuaGame {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // ── Constructors / lookups (return Nil until plumbing exists) ──
        for n in &[
            "createContainer",
            "createItem",
            "createMonster",
            "createMonsterType",
            "createNpc",
            "createTile",
            "getItemTypeByClientId",
            "getMountIdByLookType",
            "getReturnMessage",
        ] {
            methods.add_method(n, |_, _this, _args: Value| Ok(Value::Nil));
        }
        // ── Aggregate getters (return empty table) ─────────────────
        for n in &[
            "getBestiary",
            "getCurrencyItems",
            "getHouses",
            "getInstantSpells",
            "getMonsterTypes",
            "getMonsters",
            "getMounts",
            "getNpcs",
            "getOutfits",
            "getPlayers",
            "getRuneSpells",
            "getSpectators",
            "getTowns",
        ] {
            methods.add_method(n, |lua, _this, _args: Value| lua.create_table());
        }
        // ── Scalar getters (defaults that match a fresh server) ────
        methods.add_method("getClientVersion", |lua, _this, ()| {
            let t = lua.create_table()?;
            t.set("min", 0i64)?;
            t.set("max", 0i64)?;
            t.set("string", "")?;
            Ok(t)
        });
        methods.add_method("getGameState", |_, _this, ()| Ok(0i64));
        methods.add_method("getMonsterCount", |_, _this, ()| Ok(0i64));
        methods.add_method("getNpcCount", |_, _this, ()| Ok(0i64));
        methods.add_method("getPlayerCount", |_, _this, ()| Ok(0i64));
        methods.add_method("getExperienceForLevel", |_, _this, level: i64| {
            // Mirrors C++ `Player::getExpForLevel`:
            //   50/3 * level^3 − 100 * level^2 + 850/3 * level − 200
            let l = level.max(1) as f64;
            let exp = (50.0 / 3.0) * l * l * l - 100.0 * l * l + (850.0 / 3.0) * l - 200.0;
            Ok(exp.round() as i64)
        });
        methods.add_method("getExperienceStage", |_, _this, _lvl: i64| Ok(1i64));
        methods.add_method("getItemAttributeByName", |_, _this, _name: String| Ok(0i64));
        methods.add_method("getWorldType", |_, _this, ()| Ok(0i64));
        methods.add_method("getVocations", |lua, _this, ()| lua.create_table());
        methods.add_method("reload", |_, _this, _args: Value| Ok(false));
        // ── Mutators (stub OK on a fresh world) ────────────────────
        for n in &["setGameState", "setWorldType", "loadMap", "startEvent"] {
            methods.add_method(n, |_, _this, _args: Value| Ok(false));
        }
    }
}
