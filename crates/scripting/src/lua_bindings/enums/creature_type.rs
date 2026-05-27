//! `CREATURETYPE_*` enum constants. Source: `common::enums::CreatureType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::CreatureType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CREATURETYPE_MONSTER", CreatureType::Monster as i64)?;
    lua.globals()
        .set("CREATURETYPE_NPC", CreatureType::Npc as i64)?;
    lua.globals()
        .set("CREATURETYPE_PLAYER", CreatureType::Player as i64)?;
    lua.globals().set(
        "CREATURETYPE_SUMMON_OTHERS",
        CreatureType::SummonOthers as i64,
    )?;
    lua.globals()
        .set("CREATURETYPE_SUMMON_OWN", CreatureType::SummonOwn as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creature_type_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return CREATURETYPE_PLAYER").eval().unwrap();
        assert_eq!(v, 0);
    }
}
