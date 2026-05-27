//! `STAT_*` enum constants. Source: `forgottenserver_common::enums::Stat`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::Stat;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("STAT_MAGICPOINTS", Stat::MagicPoints as i64)?;
    lua.globals()
        .set("STAT_MAXHITPOINTS", Stat::MaxHitPoints as i64)?;
    lua.globals()
        .set("STAT_MAXMANAPOINTS", Stat::MaxManaPoints as i64)?;
    lua.globals()
        .set("STAT_SOULPOINTS", Stat::SoulPoints as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stat_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return STAT_MAXHITPOINTS").eval().unwrap();
    }
}
