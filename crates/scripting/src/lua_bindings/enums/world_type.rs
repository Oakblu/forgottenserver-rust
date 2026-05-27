//! `WORLD_TYPE_*` enum constants. Source: `forgottenserver_game::combat::WorldType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_game::combat::WorldType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("WORLD_TYPE_NO_PVP", WorldType::NoPvp as i64)?;
    lua.globals()
        .set("WORLD_TYPE_PVP", WorldType::OpenPvp as i64)?;
    lua.globals()
        .set("WORLD_TYPE_PVP_ENFORCED", WorldType::HardcorePvp as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn world_type_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return WORLD_TYPE_NO_PVP").eval().unwrap();
    }
}
