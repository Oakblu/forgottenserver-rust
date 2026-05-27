//! `COMBAT_PARAM_*` enum constants. Source: `common::enums::CombatParam`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::CombatParam;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("COMBAT_PARAM_AGGRESSIVE", CombatParam::Aggressive as i64)?;
    lua.globals()
        .set("COMBAT_PARAM_BLOCKARMOR", CombatParam::BlockArmor as i64)?;
    lua.globals()
        .set("COMBAT_PARAM_BLOCKSHIELD", CombatParam::BlockShield as i64)?;
    lua.globals()
        .set("COMBAT_PARAM_CREATEITEM", CombatParam::CreateItem as i64)?;
    lua.globals()
        .set("COMBAT_PARAM_DISPEL", CombatParam::Dispel as i64)?;
    lua.globals().set(
        "COMBAT_PARAM_DISTANCEEFFECT",
        CombatParam::DistanceEffect as i64,
    )?;
    lua.globals()
        .set("COMBAT_PARAM_EFFECT", CombatParam::Effect as i64)?;
    lua.globals().set(
        "COMBAT_PARAM_TARGETCASTERORTOPMOST",
        CombatParam::TargetCasterOrTopmost as i64,
    )?;
    lua.globals()
        .set("COMBAT_PARAM_TYPE", CombatParam::Type as i64)?;
    lua.globals()
        .set("COMBAT_PARAM_USECHARGES", CombatParam::UseCharges as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn combat_param_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return COMBAT_PARAM_AGGRESSIVE").eval().unwrap();
    }
}
