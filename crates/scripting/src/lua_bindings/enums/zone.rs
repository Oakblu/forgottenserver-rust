//! `ZONE_*` enum constants. Source: `game::combat::ZoneType`.
//! `ZONE_NOLOGOUT` is deferred — not a separate ZoneType variant in Rust.
#![cfg(feature = "lua-scripting")]

use forgottenserver_game::combat::ZoneType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("ZONE_NOPVP", ZoneType::NoPvp as i64)?;
    lua.globals().set("ZONE_NORMAL", ZoneType::Normal as i64)?;
    lua.globals()
        .set("ZONE_PROTECTION", ZoneType::Protection as i64)?;
    lua.globals().set("ZONE_PVP", ZoneType::Pvp as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zone_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return ZONE_NORMAL").eval().unwrap();
        assert_eq!(v, 0);
    }
}
