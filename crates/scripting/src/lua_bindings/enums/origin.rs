//! `ORIGIN_*` enum constants. Source: `common::enums::CombatOrigin`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::CombatOrigin;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("ORIGIN_CONDITION", CombatOrigin::Condition as i64)?;
    lua.globals()
        .set("ORIGIN_MELEE", CombatOrigin::Melee as i64)?;
    lua.globals()
        .set("ORIGIN_NONE", CombatOrigin::None as i64)?;
    lua.globals()
        .set("ORIGIN_RANGED", CombatOrigin::Ranged as i64)?;
    lua.globals()
        .set("ORIGIN_SPELL", CombatOrigin::Spell as i64)?;
    lua.globals()
        .set("ORIGIN_WAND", CombatOrigin::Wand as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn origin_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return ORIGIN_NONE").eval().unwrap();
        assert_eq!(v, 0);
    }
}
