//! `COMBAT_*` damage types + `COMBAT_FORMULA_*` formula types.
//! Sources: `game::combat::CombatType`, `common::enums::FormulaType`.
//! `COMBAT_PARAM_*` are deferred (would need a CombatParam enum lookup).
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::FormulaType;
use forgottenserver_game::combat::CombatType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("COMBAT_DEATHDAMAGE", CombatType::Death as i64)?;
    lua.globals()
        .set("COMBAT_DROWNDAMAGE", CombatType::Drown as i64)?;
    lua.globals()
        .set("COMBAT_EARTHDAMAGE", CombatType::Earth as i64)?;
    lua.globals()
        .set("COMBAT_ENERGYDAMAGE", CombatType::Energy as i64)?;
    lua.globals()
        .set("COMBAT_FIREDAMAGE", CombatType::Fire as i64)?;
    lua.globals()
        .set("COMBAT_HEALING", CombatType::Healing as i64)?;
    lua.globals()
        .set("COMBAT_HOLYDAMAGE", CombatType::Holy as i64)?;
    lua.globals()
        .set("COMBAT_ICEDAMAGE", CombatType::Ice as i64)?;
    lua.globals()
        .set("COMBAT_LIFEDRAIN", CombatType::LifeDrain as i64)?;
    lua.globals()
        .set("COMBAT_MANADRAIN", CombatType::ManaDrain as i64)?;
    lua.globals().set("COMBAT_NONE", CombatType::None as i64)?;
    lua.globals()
        .set("COMBAT_PHYSICALDAMAGE", CombatType::Physical as i64)?;
    lua.globals()
        .set("COMBAT_UNDEFINEDDAMAGE", CombatType::Undefined as i64)?;
    lua.globals()
        .set("COMBAT_FORMULA_DAMAGE", FormulaType::Damage as i64)?;
    lua.globals()
        .set("COMBAT_FORMULA_LEVELMAGIC", FormulaType::LevelMagic as i64)?;
    lua.globals()
        .set("COMBAT_FORMULA_SKILL", FormulaType::Skill as i64)?;
    lua.globals()
        .set("COMBAT_FORMULA_UNDEFINED", FormulaType::Undefined as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combat_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return COMBAT_NONE").eval().unwrap();
        assert_eq!(v, 0);
    }
}
