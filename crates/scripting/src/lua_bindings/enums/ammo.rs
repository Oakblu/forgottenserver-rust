//! `AMMO_*` enum constants. Source: `common::enums::Ammo`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::Ammo;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("AMMO_NONE", Ammo::None as i64)?;
    lua.globals().set("AMMO_BOLT", Ammo::Bolt as i64)?;
    lua.globals().set("AMMO_ARROW", Ammo::Arrow as i64)?;
    lua.globals().set("AMMO_SPEAR", Ammo::Spear as i64)?;
    lua.globals()
        .set("AMMO_THROWINGSTAR", Ammo::ThrowingStar as i64)?;
    lua.globals()
        .set("AMMO_THROWINGKNIFE", Ammo::ThrowingKnife as i64)?;
    lua.globals().set("AMMO_STONE", Ammo::Stone as i64)?;
    lua.globals().set("AMMO_SNOWBALL", Ammo::Snowball as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_eight_ammo_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        for (name, expected) in [
            ("AMMO_NONE", 0i64),
            ("AMMO_BOLT", 1),
            ("AMMO_ARROW", 2),
            ("AMMO_SPEAR", 3),
            ("AMMO_THROWINGSTAR", 4),
            ("AMMO_THROWINGKNIFE", 5),
            ("AMMO_STONE", 6),
            ("AMMO_SNOWBALL", 7),
        ] {
            let v: i64 = lua.load(format!("return {name}")).eval().unwrap();
            assert_eq!(v, expected, "{name}");
        }
    }
}
