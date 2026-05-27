//! `CONST_SLOT_*` enum constants.
//!
//! Registered with **literal values matching the C++ enum**
//! (see `forgottenserver/src/creature.h` enum slots_t). No Rust enum equivalent exists in
//! `common::` yet — future work can extract these into a typed
//! enum and switch this file to the `EnumKind::Variant as i64`
//! pattern used by the rest of `lua_bindings/enums/`.
#![cfg(feature = "lua-scripting")]

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("CONST_SLOT_HEAD", 1_i64)?;
    lua.globals().set("CONST_SLOT_NECKLACE", 2_i64)?;
    lua.globals().set("CONST_SLOT_BACKPACK", 3_i64)?;
    lua.globals().set("CONST_SLOT_ARMOR", 4_i64)?;
    lua.globals().set("CONST_SLOT_RIGHT", 5_i64)?;
    lua.globals().set("CONST_SLOT_LEFT", 6_i64)?;
    lua.globals().set("CONST_SLOT_LEGS", 7_i64)?;
    lua.globals().set("CONST_SLOT_FEET", 8_i64)?;
    lua.globals().set("CONST_SLOT_RING", 9_i64)?;
    lua.globals().set("CONST_SLOT_AMMO", 10_i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn const_slot_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return CONST_SLOT_HEAD").eval().unwrap();
    }
}
