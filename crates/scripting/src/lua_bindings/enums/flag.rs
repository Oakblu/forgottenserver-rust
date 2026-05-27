//! `FLAG_*` enum constants.
//!
//! Registered with **literal values matching the C++ enum**
//! (see `forgottenserver/src/thing.h` enum cylinderflags_t). No Rust enum equivalent exists in
//! `common::` yet — future work can extract these into a typed
//! enum and switch this file to the `EnumKind::Variant as i64`
//! pattern used by the rest of `lua_bindings/enums/`.
#![cfg(feature = "lua-scripting")]

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("FLAG_NOLIMIT", 1_i64)?;
    lua.globals().set("FLAG_IGNOREBLOCKITEM", 2_i64)?;
    lua.globals().set("FLAG_IGNOREBLOCKCREATURE", 4_i64)?;
    lua.globals().set("FLAG_CHILDISOWNER", 8_i64)?;
    lua.globals().set("FLAG_PATHFINDING", 16_i64)?;
    lua.globals().set("FLAG_IGNOREFIELDDAMAGE", 32_i64)?;
    lua.globals().set("FLAG_IGNORENOTMOVEABLE", 64_i64)?;
    lua.globals().set("FLAG_IGNOREAUTOSTACK", 128_i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flag_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return FLAG_NOLIMIT").eval().unwrap();
    }
}
