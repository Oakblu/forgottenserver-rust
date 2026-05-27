//! `MONSTER_ICON_*` enum constants. Source: `common::constants::MonsterIcon`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::MonsterIcon;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("MONSTER_ICON_FIENDISH", MonsterIcon::Fiendish as i64)?;
    lua.globals()
        .set("MONSTER_ICON_INFLUENCED", MonsterIcon::Influenced as i64)?;
    lua.globals()
        .set("MONSTER_ICON_MELEE", MonsterIcon::Melee as i64)?;
    lua.globals()
        .set("MONSTER_ICON_VULNERABLE", MonsterIcon::Vulnerable as i64)?;
    lua.globals()
        .set("MONSTER_ICON_WEAKENED", MonsterIcon::Weakened as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn monster_icon_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return MONSTER_ICON_MELEE").eval().unwrap();
    }
}
