//! `SKILL_*` enum constants. Source: `common::enums::Skill`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::Skill;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("SKILL_FIST", Skill::Fist as i64)?;
    lua.globals().set("SKILL_CLUB", Skill::Club as i64)?;
    lua.globals().set("SKILL_SWORD", Skill::Sword as i64)?;
    lua.globals().set("SKILL_AXE", Skill::Axe as i64)?;
    lua.globals()
        .set("SKILL_DISTANCE", Skill::Distance as i64)?;
    lua.globals().set("SKILL_SHIELD", Skill::Shield as i64)?;
    lua.globals().set("SKILL_FISHING", Skill::Fishing as i64)?;
    lua.globals()
        .set("SKILL_MAGLEVEL", Skill::MagLevel as i64)?;
    lua.globals().set("SKILL_LEVEL", Skill::Level as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_nine_skill_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return SKILL_FIST").eval().unwrap();
        assert_eq!(v, 0);
        let v: i64 = lua.load("return SKILL_LEVEL").eval().unwrap();
        assert_eq!(v, 8);
    }
}
