//! `WIELDINFO_*` enum constants. Source: `forgottenserver_common::constants::WieldInfo`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::WieldInfo;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("WIELDINFO_LEVEL", WieldInfo::LEVEL as i64)?;
    lua.globals()
        .set("WIELDINFO_MAGLV", WieldInfo::MAGLV as i64)?;
    lua.globals()
        .set("WIELDINFO_NONE", WieldInfo::NONE as i64)?;
    lua.globals()
        .set("WIELDINFO_PREMIUM", WieldInfo::PREMIUM as i64)?;
    lua.globals()
        .set("WIELDINFO_VOCREQ", WieldInfo::VOCREQ as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wield_info_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return WIELDINFO_LEVEL").eval().unwrap();
    }
}
