//! `CREATURE_ICON_*` enum constants. Source: `common::constants::CreatureIcon`.
//! `CREATURE_ICON_FIRST`/`LAST` are markers without C++ value equivalents —
//! deferred until needed.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::CreatureIcon;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CREATURE_ICON_ARROW_DOWN", CreatureIcon::ArrowDown as i64)?;
    lua.globals()
        .set("CREATURE_ICON_ARROW_UP", CreatureIcon::ArrowUp as i64)?;
    lua.globals()
        .set("CREATURE_ICON_CROSS_RED", CreatureIcon::CrossRed as i64)?;
    lua.globals()
        .set("CREATURE_ICON_CROSS_WHITE", CreatureIcon::CrossWhite as i64)?;
    lua.globals().set(
        "CREATURE_ICON_CROSS_WHITE_RED",
        CreatureIcon::CrossWhiteRed as i64,
    )?;
    lua.globals()
        .set("CREATURE_ICON_ENERGY", CreatureIcon::Energy as i64)?;
    lua.globals()
        .set("CREATURE_ICON_FIRE", CreatureIcon::Fire as i64)?;
    lua.globals()
        .set("CREATURE_ICON_GEM_BLUE", CreatureIcon::GemBlue as i64)?;
    lua.globals()
        .set("CREATURE_ICON_GEM_GREEN", CreatureIcon::GemGreen as i64)?;
    lua.globals()
        .set("CREATURE_ICON_GEM_PURPLE", CreatureIcon::GemPurple as i64)?;
    lua.globals()
        .set("CREATURE_ICON_GEM_RED", CreatureIcon::GemRed as i64)?;
    lua.globals()
        .set("CREATURE_ICON_GEM_YELLOW", CreatureIcon::GemYellow as i64)?;
    lua.globals()
        .set("CREATURE_ICON_ICE", CreatureIcon::Ice as i64)?;
    lua.globals()
        .set("CREATURE_ICON_ORB_GREEN", CreatureIcon::OrbGreen as i64)?;
    lua.globals()
        .set("CREATURE_ICON_ORB_RED", CreatureIcon::OrbRed as i64)?;
    lua.globals().set(
        "CREATURE_ICON_ORB_RED_GREEN",
        CreatureIcon::OrbRedGreen as i64,
    )?;
    lua.globals()
        .set("CREATURE_ICON_PIGEON", CreatureIcon::Pigeon as i64)?;
    lua.globals()
        .set("CREATURE_ICON_POISON", CreatureIcon::Poison as i64)?;
    lua.globals()
        .set("CREATURE_ICON_QUESTION", CreatureIcon::Question as i64)?;
    lua.globals()
        .set("CREATURE_ICON_WARNING", CreatureIcon::Warning as i64)?;
    lua.globals()
        .set("CREATURE_ICON_WATER", CreatureIcon::Water as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creature_icon_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        for name in ["CREATURE_ICON_FIRE", "CREATURE_ICON_POISON"] {
            lua.load(format!("return {name}")).eval::<i64>().unwrap();
        }
    }
}
