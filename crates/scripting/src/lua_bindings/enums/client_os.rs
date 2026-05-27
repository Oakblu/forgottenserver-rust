//! `CLIENTOS_*` enum constants. Source: `common::enums::OperatingSystem`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::OperatingSystem;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CLIENTOS_FLASH", OperatingSystem::Flash as i64)?;
    lua.globals()
        .set("CLIENTOS_LINUX", OperatingSystem::Linux as i64)?;
    lua.globals().set(
        "CLIENTOS_OTCLIENT_LINUX",
        OperatingSystem::OtClientLinux as i64,
    )?;
    lua.globals()
        .set("CLIENTOS_OTCLIENT_MAC", OperatingSystem::OtClientMac as i64)?;
    lua.globals().set(
        "CLIENTOS_OTCLIENT_WINDOWS",
        OperatingSystem::OtClientWindows as i64,
    )?;
    lua.globals()
        .set("CLIENTOS_WINDOWS", OperatingSystem::Windows as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn client_os_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return CLIENTOS_LINUX").eval().unwrap();
        assert_eq!(v, 1);
    }
}
