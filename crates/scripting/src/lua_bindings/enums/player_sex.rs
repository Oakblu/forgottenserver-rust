//! `PLAYERSEX_*` enum constants. Source: `forgottenserver_common::enums::PlayerSex`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::PlayerSex;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("PLAYERSEX_FEMALE", PlayerSex::Female as i64)?;
    lua.globals()
        .set("PLAYERSEX_MALE", PlayerSex::Male as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_sex_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return PLAYERSEX_MALE").eval().unwrap();
    }
}
