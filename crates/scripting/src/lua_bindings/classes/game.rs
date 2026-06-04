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
    fn get_game_state_returns_zero() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: i64 = lua.load("return Game:getGameState()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_monster_count_returns_zero() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: i64 = lua.load("return Game:getMonsterCount()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_npc_count_returns_zero() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: i64 = lua.load("return Game:getNpcCount()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_player_count_returns_zero() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: i64 = lua.load("return Game:getPlayerCount()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_world_type_returns_zero() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: i64 = lua.load("return Game:getWorldType()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_experience_for_level_returns_value() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        // Level 2 should return a positive experience value
        let v: i64 = lua
            .load("return Game:getExperienceForLevel(2)")
            .eval()
            .unwrap();
        assert!(v > 0);
    }

    #[test]
    fn get_experience_stage_returns_one() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: i64 = lua
            .load("return Game:getExperienceStage(10)")
            .eval()
            .unwrap();
        assert_eq!(v, 1);
    }

    #[test]
    fn get_client_version_returns_table() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: bool = lua
            .load("local t = Game:getClientVersion(); return t ~= nil")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn constructor_stub_returns_nil() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: mlua::Value = lua.load("return Game:createItem(1)").eval().unwrap();
        assert!(matches!(v, mlua::Value::Nil));
    }

    #[test]
    fn aggregate_getter_returns_table() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: bool = lua
            .load("local t = Game:getPlayers(); return type(t) == 'table'")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn mutator_stub_returns_false() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: bool = lua.load("return Game:setGameState(4)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn get_item_attribute_by_name_returns_zero() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: i64 = lua
            .load("return Game:getItemAttributeByName('test')")
            .eval()
            .unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_vocations_returns_table() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: bool = lua
            .load("local t = Game:getVocations(); return type(t) == 'table'")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn reload_returns_false() {
        let lua = fresh_lua();
        lua.globals().set("Game", LuaGame).unwrap();
        let v: bool = lua.load("return Game:reload(1)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaGame> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
