//! `FIGHTMODE_*` enum constants. Source: `entity::player::FightMode`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_entity::player::FightMode;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("FIGHTMODE_ATTACK", FightMode::Attack as i64)?;
    lua.globals()
        .set("FIGHTMODE_BALANCED", FightMode::Balanced as i64)?;
    lua.globals()
        .set("FIGHTMODE_DEFENSE", FightMode::Defense as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fight_mode_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return FIGHTMODE_ATTACK").eval().unwrap();
    }
}
