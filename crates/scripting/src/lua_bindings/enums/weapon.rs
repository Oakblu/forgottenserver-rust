//! `WEAPON_*` enum constants. Source: `common::enums::WeaponType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::WeaponType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("WEAPON_NONE", WeaponType::None as i64)?;
    lua.globals()
        .set("WEAPON_SWORD", WeaponType::Sword as i64)?;
    lua.globals().set("WEAPON_CLUB", WeaponType::Club as i64)?;
    lua.globals().set("WEAPON_AXE", WeaponType::Axe as i64)?;
    lua.globals()
        .set("WEAPON_SHIELD", WeaponType::Shield as i64)?;
    lua.globals()
        .set("WEAPON_DISTANCE", WeaponType::Distance as i64)?;
    lua.globals().set("WEAPON_WAND", WeaponType::Wand as i64)?;
    lua.globals().set("WEAPON_AMMO", WeaponType::Ammo as i64)?;
    lua.globals()
        .set("WEAPON_QUIVER", WeaponType::Quiver as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_nine_weapon_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return WEAPON_NONE").eval().unwrap();
        assert_eq!(v, 0);
        let v: i64 = lua.load("return WEAPON_QUIVER").eval().unwrap();
        assert_eq!(v, 8);
    }
}
