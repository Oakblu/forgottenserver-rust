//! `SKULL_*` enum constants. Source: `common::enums::Skull`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::Skull;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("SKULL_NONE", Skull::None as i64)?;
    lua.globals().set("SKULL_YELLOW", Skull::Yellow as i64)?;
    lua.globals().set("SKULL_GREEN", Skull::Green as i64)?;
    lua.globals().set("SKULL_WHITE", Skull::White as i64)?;
    lua.globals().set("SKULL_RED", Skull::Red as i64)?;
    lua.globals().set("SKULL_BLACK", Skull::Black as i64)?;
    lua.globals().set("SKULL_ORANGE", Skull::Orange as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_seven_skull_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        for (name, expected) in [
            ("SKULL_NONE", 0i64),
            ("SKULL_YELLOW", 1),
            ("SKULL_GREEN", 2),
            ("SKULL_WHITE", 3),
            ("SKULL_RED", 4),
            ("SKULL_BLACK", 5),
            ("SKULL_ORANGE", 6),
        ] {
            let v: i64 = lua.load(format!("return {name}")).eval().unwrap();
            assert_eq!(v, expected, "{name}");
        }
    }
}
