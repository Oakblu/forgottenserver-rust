//! `MonsterSpell:*` Lua binding (data-only builder used by monster XML loaders).

// AUDIT: ClassMethod MonsterSpell:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct LuaMonsterSpell;

impl<'lua> mlua::FromLua<'lua> for LuaMonsterSpell {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaMonsterSpell>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaMonsterSpell",
                message: Some("expected MonsterSpell userdata".into()),
            }),
        }
    }
}

impl UserData for LuaMonsterSpell {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        for n in &[
            "setType",
            "setScriptName",
            "setInterval",
            "setChance",
            "setRange",
            "setNeedTarget",
            "setNeedDirection",
            "setCombatValue",
            "setCombatType",
            "setCombatLength",
            "setCombatSpread",
            "setCombatRadius",
            "setCombatRing",
            "setConditionType",
            "setConditionDamage",
            "setConditionDuration",
            "setCombatShootEffect",
            "setCombatEffect",
            "setAttackValue",
            "setOutfit",
            "delete",
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
    fn set_type_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("spell", LuaMonsterSpell).unwrap();
        let result: mlua::Result<()> = lua.load("spell:setType('melee')").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn set_interval_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("spell", LuaMonsterSpell).unwrap();
        let result: mlua::Result<()> = lua.load("spell:setInterval(2000)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn set_chance_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("spell", LuaMonsterSpell).unwrap();
        let result: mlua::Result<()> = lua.load("spell:setChance(100)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn set_combat_value_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("spell", LuaMonsterSpell).unwrap();
        let result: mlua::Result<()> = lua.load("spell:setCombatValue(-100, -200)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn delete_does_not_error() {
        let lua = fresh_lua();
        lua.globals().set("spell", LuaMonsterSpell).unwrap();
        let result: mlua::Result<()> = lua.load("spell:delete()").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaMonsterSpell> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
