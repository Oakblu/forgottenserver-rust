//! `SLOTP_*` enum constants.
//!
//! Registered with **literal values matching the C++ enum**
//! (see `forgottenserver/src/items.h` enum SlotPositionBits). No Rust enum equivalent exists in
//! `common::` yet — future work can extract these into a typed
//! enum and switch this file to the `EnumKind::Variant as i64`
//! pattern used by the rest of `lua_bindings/enums/`.
#![cfg(feature = "lua-scripting")]

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("SLOTP_HEAD", 1_i64)?;
    lua.globals().set("SLOTP_NECKLACE", 2_i64)?;
    lua.globals().set("SLOTP_BACKPACK", 4_i64)?;
    lua.globals().set("SLOTP_ARMOR", 8_i64)?;
    lua.globals().set("SLOTP_RIGHT", 16_i64)?;
    lua.globals().set("SLOTP_LEFT", 32_i64)?;
    lua.globals().set("SLOTP_LEGS", 64_i64)?;
    lua.globals().set("SLOTP_FEET", 128_i64)?;
    lua.globals().set("SLOTP_RING", 256_i64)?;
    lua.globals().set("SLOTP_AMMO", 512_i64)?;
    lua.globals().set("SLOTP_DEPOT", 1024_i64)?;
    lua.globals().set("SLOTP_TWO_HAND", 2048_i64)?;
    lua.globals().set("SLOTP_WHEREEVER", 4294967295_i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slot_position_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return SLOTP_HEAD").eval().unwrap();
    }
}
