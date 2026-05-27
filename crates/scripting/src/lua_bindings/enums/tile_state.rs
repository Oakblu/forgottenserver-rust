//! `TILESTATE_*` enum constants.
//!
//! Registered with **literal values matching the C++ enum**
//! (see `forgottenserver/src/tile.h` enum TileFlags). No Rust enum equivalent exists in
//! `common::` yet — future work can extract these into a typed
//! enum and switch this file to the `EnumKind::Variant as i64`
//! pattern used by the rest of `lua_bindings/enums/`.
#![cfg(feature = "lua-scripting")]

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("TILESTATE_NONE", 0_i64)?;
    lua.globals().set("TILESTATE_FLOORCHANGE_DOWN", 1_i64)?;
    lua.globals().set("TILESTATE_FLOORCHANGE_NORTH", 2_i64)?;
    lua.globals().set("TILESTATE_FLOORCHANGE_SOUTH", 4_i64)?;
    lua.globals().set("TILESTATE_FLOORCHANGE_EAST", 8_i64)?;
    lua.globals().set("TILESTATE_FLOORCHANGE_WEST", 16_i64)?;
    lua.globals()
        .set("TILESTATE_FLOORCHANGE_SOUTH_ALT", 32_i64)?;
    lua.globals()
        .set("TILESTATE_FLOORCHANGE_EAST_ALT", 64_i64)?;
    lua.globals().set("TILESTATE_PROTECTIONZONE", 128_i64)?;
    lua.globals().set("TILESTATE_NOPVPZONE", 256_i64)?;
    lua.globals().set("TILESTATE_NOLOGOUT", 512_i64)?;
    lua.globals().set("TILESTATE_PVPZONE", 1024_i64)?;
    lua.globals().set("TILESTATE_TELEPORT", 2048_i64)?;
    lua.globals().set("TILESTATE_MAGICFIELD", 4096_i64)?;
    lua.globals().set("TILESTATE_MAILBOX", 8192_i64)?;
    lua.globals().set("TILESTATE_TRASHHOLDER", 16384_i64)?;
    lua.globals().set("TILESTATE_BED", 32768_i64)?;
    lua.globals().set("TILESTATE_DEPOT", 65536_i64)?;
    lua.globals().set("TILESTATE_BLOCKSOLID", 131072_i64)?;
    lua.globals().set("TILESTATE_BLOCKPATH", 262144_i64)?;
    lua.globals()
        .set("TILESTATE_IMMOVABLEBLOCKSOLID", 524288_i64)?;
    lua.globals()
        .set("TILESTATE_IMMOVABLEBLOCKPATH", 1048576_i64)?;
    lua.globals()
        .set("TILESTATE_IMMOVABLENOFIELDBLOCKPATH", 2097152_i64)?;
    lua.globals()
        .set("TILESTATE_NOFIELDBLOCKPATH", 4194304_i64)?;
    lua.globals()
        .set("TILESTATE_SUPPORTS_HANGABLE", 8388608_i64)?;
    lua.globals().set("TILESTATE_FLOORCHANGE", 127_i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tile_state_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return TILESTATE_NONE").eval().unwrap();
    }
}
