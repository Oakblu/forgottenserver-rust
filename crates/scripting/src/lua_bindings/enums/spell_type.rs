//! `SPELL_*` enum constants. Source: `forgottenserver_common::enums::SpellType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::SpellType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("SPELL_INSTANT", SpellType::Instant as i64)?;
    lua.globals().set("SPELL_RUNE", SpellType::Rune as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spell_type_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return SPELL_INSTANT").eval().unwrap();
    }
}
