//! `CONDITION_*` base condition constants. Source: `game::condition::ConditionKind`.
//! `CONDITION_PARAM_*` and `CONDITION_EXHAUST_*` are deferred — would need
//! separate ConditionParam enum mappings.
#![cfg(feature = "lua-scripting")]

use forgottenserver_game::condition::ConditionKind;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CONDITION_BLEEDING", ConditionKind::Bleeding as i64)?;
    lua.globals()
        .set("CONDITION_DROWN", ConditionKind::Drown as i64)?;
    lua.globals()
        .set("CONDITION_DRUNK", ConditionKind::Drunk as i64)?;
    lua.globals()
        .set("CONDITION_ENERGY", ConditionKind::Energy as i64)?;
    lua.globals()
        .set("CONDITION_FIRE", ConditionKind::Fire as i64)?;
    lua.globals()
        .set("CONDITION_HASTE", ConditionKind::Haste as i64)?;
    lua.globals()
        .set("CONDITION_INVISIBLE", ConditionKind::Invisible as i64)?;
    lua.globals()
        .set("CONDITION_LIGHT", ConditionKind::Light as i64)?;
    lua.globals()
        .set("CONDITION_MANASHIELD", ConditionKind::ManaShield as i64)?;
    lua.globals()
        .set("CONDITION_OUTFIT", ConditionKind::Outfit as i64)?;
    lua.globals()
        .set("CONDITION_PACIFIED", ConditionKind::Pacified as i64)?;
    lua.globals()
        .set("CONDITION_PARALYZE", ConditionKind::Paralyze as i64)?;
    lua.globals()
        .set("CONDITION_POISON", ConditionKind::Poison as i64)?;
    lua.globals()
        .set("CONDITION_REGENERATION", ConditionKind::Regeneration as i64)?;
    lua.globals()
        .set("CONDITION_ROOT", ConditionKind::Root as i64)?;
    // ── Bit-flag style condition kinds (no Rust enum yet — values mirror
    //    C++ `ConditionType_t` in enums.h exactly).
    lua.globals().set("CONDITION_NONE", 0i64)?;
    lua.globals().set("CONDITION_INFIGHT", 1i64 << 10)?;
    lua.globals().set("CONDITION_EXHAUST_WEAPON", 1i64 << 12)?;
    lua.globals().set("CONDITION_SOUL", 1i64 << 14)?;
    lua.globals().set("CONDITION_MUTED", 1i64 << 16)?;
    lua.globals()
        .set("CONDITION_CHANNELMUTEDTICKS", 1i64 << 17)?;
    lua.globals().set("CONDITION_YELLTICKS", 1i64 << 18)?;
    lua.globals().set("CONDITION_ATTRIBUTES", 1i64 << 19)?;
    lua.globals().set("CONDITION_FREEZING", 1i64 << 20)?;
    lua.globals().set("CONDITION_DAZZLED", 1i64 << 21)?;
    lua.globals().set("CONDITION_CURSED", 1i64 << 22)?;
    lua.globals().set("CONDITION_EXHAUST_COMBAT", 1i64 << 23)?;
    lua.globals().set("CONDITION_EXHAUST_HEAL", 1i64 << 24)?;
    lua.globals().set("CONDITION_SPELLCOOLDOWN", 1i64 << 26)?;
    lua.globals()
        .set("CONDITION_SPELLGROUPCOOLDOWN", 1i64 << 27)?;
    lua.globals()
        .set("CONDITION_MANASHIELD_BREAKABLE", 1i64 << 29)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn condition_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        // Spot-check a few well-known conditions.
        for name in ["CONDITION_FIRE", "CONDITION_POISON", "CONDITION_HASTE"] {
            lua.load(format!("return {name}")).eval::<i64>().unwrap();
        }
    }
}
