//! `ITEM_ATTRIBUTE_*` enum constants. Source: `items::item::ItemAttribute`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_items::item::ItemAttribute;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("ITEM_ATTRIBUTE_ACTIONID", ItemAttribute::ActionId as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_ARMOR", ItemAttribute::Armor as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_ARTICLE", ItemAttribute::Article as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_ATTACK", ItemAttribute::Attack as i64)?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_ATTACK_SPEED",
        ItemAttribute::AttackSpeed as i64,
    )?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_CHARGES", ItemAttribute::Charges as i64)?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_CORPSEOWNER",
        ItemAttribute::CorpseOwner as i64,
    )?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_DATE", ItemAttribute::Date as i64)?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_DECAYSTATE",
        ItemAttribute::DecayState as i64,
    )?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_DECAYTO", ItemAttribute::DecayTo as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_DEFENSE", ItemAttribute::Defense as i64)?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_DESCRIPTION",
        ItemAttribute::Description as i64,
    )?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_DOORID", ItemAttribute::DoorId as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_DURATION", ItemAttribute::Duration as i64)?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_DURATION_MAX",
        ItemAttribute::DurationMax as i64,
    )?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_EXTRADEFENSE",
        ItemAttribute::ExtraDefense as i64,
    )?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_FLUIDTYPE", ItemAttribute::FluidType as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_HITCHANCE", ItemAttribute::HitChance as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_NAME", ItemAttribute::Name as i64)?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_OPENCONTAINER",
        ItemAttribute::OpenContainer as i64,
    )?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_OWNER", ItemAttribute::Owner as i64)?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_PLURALNAME",
        ItemAttribute::PluralName as i64,
    )?;
    lua.globals().set(
        "ITEM_ATTRIBUTE_SHOOTRANGE",
        ItemAttribute::ShootRange as i64,
    )?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_STOREITEM", ItemAttribute::StoreItem as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_TEXT", ItemAttribute::Text as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_UNIQUEID", ItemAttribute::UniqueId as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_WEIGHT", ItemAttribute::Weight as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_WRAPID", ItemAttribute::WrapId as i64)?;
    lua.globals()
        .set("ITEM_ATTRIBUTE_WRITER", ItemAttribute::Writer as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn item_attribute_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return ITEM_ATTRIBUTE_NAME").eval().unwrap();
    }
}
