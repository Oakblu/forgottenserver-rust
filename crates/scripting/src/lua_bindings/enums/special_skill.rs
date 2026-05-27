//! `SPECIALSKILL_*` enum constants. Source: `common::enums::SpecialSkill`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::SpecialSkill;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set(
        "SPECIALSKILL_CRITICALHITAMOUNT",
        SpecialSkill::CriticalHitAmount as i64,
    )?;
    lua.globals().set(
        "SPECIALSKILL_CRITICALHITCHANCE",
        SpecialSkill::CriticalHitChance as i64,
    )?;
    lua.globals().set(
        "SPECIALSKILL_LIFELEECHAMOUNT",
        SpecialSkill::LifeLeechAmount as i64,
    )?;
    lua.globals().set(
        "SPECIALSKILL_LIFELEECHCHANCE",
        SpecialSkill::LifeLeechChance as i64,
    )?;
    lua.globals().set(
        "SPECIALSKILL_MANALEECHAMOUNT",
        SpecialSkill::ManaLeechAmount as i64,
    )?;
    lua.globals().set(
        "SPECIALSKILL_MANALEECHCHANCE",
        SpecialSkill::ManaLeechChance as i64,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn special_skill_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua
            .load("return SPECIALSKILL_CRITICALHITCHANCE")
            .eval()
            .unwrap();
        assert_eq!(v, 0);
    }
}
