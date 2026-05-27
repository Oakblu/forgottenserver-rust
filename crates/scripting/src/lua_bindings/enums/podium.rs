//! `PODIUM_*` enum constants. Source: `forgottenserver_common::constants::PodiumFlag`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::PodiumFlag;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("PODIUM_SHOW_MOUNT", PodiumFlag::ShowMount as i64)?;
    lua.globals()
        .set("PODIUM_SHOW_OUTFIT", PodiumFlag::ShowOutfit as i64)?;
    lua.globals()
        .set("PODIUM_SHOW_PLATFORM", PodiumFlag::ShowPlatform as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn podium_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return PODIUM_SHOW_OUTFIT").eval().unwrap();
    }
}
