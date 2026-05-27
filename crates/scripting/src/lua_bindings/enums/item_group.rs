//! `ITEM_GROUP_*` enum constants. Source: `common::itemloader::ItemGroup`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::itemloader::ItemGroup;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("ITEM_GROUP_AMMUNITION", ItemGroup::Ammunition as i64)?;
    lua.globals()
        .set("ITEM_GROUP_ARMOR", ItemGroup::Armor as i64)?;
    lua.globals()
        .set("ITEM_GROUP_CHARGES", ItemGroup::Charges as i64)?;
    lua.globals()
        .set("ITEM_GROUP_CONTAINER", ItemGroup::Container as i64)?;
    lua.globals()
        .set("ITEM_GROUP_DEPRECATED", ItemGroup::Deprecated as i64)?;
    lua.globals()
        .set("ITEM_GROUP_DOOR", ItemGroup::Door as i64)?;
    lua.globals()
        .set("ITEM_GROUP_FLUID", ItemGroup::Fluid as i64)?;
    lua.globals()
        .set("ITEM_GROUP_GROUND", ItemGroup::Ground as i64)?;
    lua.globals().set("ITEM_GROUP_KEY", ItemGroup::Key as i64)?;
    lua.globals()
        .set("ITEM_GROUP_MAGICFIELD", ItemGroup::MagicField as i64)?;
    lua.globals()
        .set("ITEM_GROUP_PODIUM", ItemGroup::Podium as i64)?;
    lua.globals()
        .set("ITEM_GROUP_SPLASH", ItemGroup::Splash as i64)?;
    lua.globals()
        .set("ITEM_GROUP_TELEPORT", ItemGroup::Teleport as i64)?;
    lua.globals()
        .set("ITEM_GROUP_WEAPON", ItemGroup::Weapon as i64)?;
    lua.globals()
        .set("ITEM_GROUP_WRITEABLE", ItemGroup::Writeable as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn item_group_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return ITEM_GROUP_GROUND").eval().unwrap();
    }
}
