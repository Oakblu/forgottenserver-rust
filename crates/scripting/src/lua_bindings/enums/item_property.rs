//! `CONST_PROP_*` enum constants. Source: `forgottenserver_map::tile::ItemProperty`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_map::tile::ItemProperty;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CONST_PROP_BLOCKPATH", ItemProperty::BlockPath as i64)?;
    lua.globals().set(
        "CONST_PROP_BLOCKPROJECTILE",
        ItemProperty::BlockProjectile as i64,
    )?;
    lua.globals()
        .set("CONST_PROP_BLOCKSOLID", ItemProperty::BlockSolid as i64)?;
    lua.globals()
        .set("CONST_PROP_HASHEIGHT", ItemProperty::HasHeight as i64)?;
    lua.globals().set(
        "CONST_PROP_IMMOVABLEBLOCKPATH",
        ItemProperty::ImmovableBlockPath as i64,
    )?;
    lua.globals().set(
        "CONST_PROP_IMMOVABLEBLOCKSOLID",
        ItemProperty::ImmovableBlockSolid as i64,
    )?;
    lua.globals().set(
        "CONST_PROP_IMMOVABLENOFIELDBLOCKPATH",
        ItemProperty::ImmovableNoFieldBlockPath as i64,
    )?;
    lua.globals()
        .set("CONST_PROP_ISHORIZONTAL", ItemProperty::IsHorizontal as i64)?;
    lua.globals()
        .set("CONST_PROP_ISVERTICAL", ItemProperty::IsVertical as i64)?;
    lua.globals()
        .set("CONST_PROP_MOVEABLE", ItemProperty::Moveable as i64)?;
    lua.globals().set(
        "CONST_PROP_NOFIELDBLOCKPATH",
        ItemProperty::NoFieldBlockPath as i64,
    )?;
    lua.globals().set(
        "CONST_PROP_SUPPORTHANGABLE",
        ItemProperty::SupportHangable as i64,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn item_property_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return CONST_PROP_BLOCKSOLID").eval().unwrap();
    }
}
