//! `ITEM_*` item-id constants (excluding `ITEM_ATTRIBUTE_*` and `ITEM_GROUP_*`).
//! Source: `common::constants::ItemId`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::ItemId;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("ITEM_AMULETOFLOSS", ItemId::AmuletOfLoss as i64)?;
    lua.globals().set("ITEM_BAG", ItemId::Bag as i64)?;
    lua.globals()
        .set("ITEM_BROWSEFIELD", ItemId::BrowseField as i64)?;
    lua.globals()
        .set("ITEM_CRYSTAL_COIN", ItemId::CrystalCoin as i64)?;
    lua.globals()
        .set("ITEM_DECORATION_KIT", ItemId::DecorationKit as i64)?;
    lua.globals()
        .set("ITEM_ENERGYFIELD_NOPVP", ItemId::EnergyfieldNopvp as i64)?;
    lua.globals().set(
        "ITEM_ENERGYFIELD_PERSISTENT",
        ItemId::EnergyfieldPersistent as i64,
    )?;
    lua.globals()
        .set("ITEM_ENERGYFIELD_PVP", ItemId::EnergyfieldPvp as i64)?;
    lua.globals()
        .set("ITEM_FIREFIELD_NOPVP", ItemId::FirefieldNopvp as i64)?;
    lua.globals().set(
        "ITEM_FIREFIELD_NOPVP_MEDIUM",
        ItemId::FirefieldNopvpMedium as i64,
    )?;
    lua.globals().set(
        "ITEM_FIREFIELD_PERSISTENT_FULL",
        ItemId::FirefieldPersistentFull as i64,
    )?;
    lua.globals().set(
        "ITEM_FIREFIELD_PERSISTENT_MEDIUM",
        ItemId::FirefieldPersistentMedium as i64,
    )?;
    lua.globals().set(
        "ITEM_FIREFIELD_PERSISTENT_SMALL",
        ItemId::FirefieldPersistentSmall as i64,
    )?;
    lua.globals()
        .set("ITEM_FIREFIELD_PVP_FULL", ItemId::FirefieldPvpFull as i64)?;
    lua.globals().set(
        "ITEM_FIREFIELD_PVP_MEDIUM",
        ItemId::FirefieldPvpMedium as i64,
    )?;
    lua.globals()
        .set("ITEM_FIREFIELD_PVP_SMALL", ItemId::FirefieldPvpSmall as i64)?;
    lua.globals()
        .set("ITEM_GOLD_COIN", ItemId::GoldCoin as i64)?;
    lua.globals().set("ITEM_LABEL", ItemId::Label as i64)?;
    lua.globals()
        .set("ITEM_MAGICWALL", ItemId::Magicwall as i64)?;
    lua.globals().set(
        "ITEM_MAGICWALL_PERSISTENT",
        ItemId::MagicwallPersistent as i64,
    )?;
    lua.globals()
        .set("ITEM_MAGICWALL_SAFE", ItemId::MagicwallSafe as i64)?;
    lua.globals().set("ITEM_PARCEL", ItemId::Parcel as i64)?;
    lua.globals()
        .set("ITEM_PLATINUM_COIN", ItemId::PlatinumCoin as i64)?;
    lua.globals()
        .set("ITEM_POISONFIELD_NOPVP", ItemId::PoisonfieldNopvp as i64)?;
    lua.globals().set(
        "ITEM_POISONFIELD_PERSISTENT",
        ItemId::PoisonfieldPersistent as i64,
    )?;
    lua.globals()
        .set("ITEM_POISONFIELD_PVP", ItemId::PoisonfieldPvp as i64)?;
    lua.globals()
        .set("ITEM_SHOPPING_BAG", ItemId::ShoppingBag as i64)?;
    lua.globals()
        .set("ITEM_WILDGROWTH", ItemId::Wildgrowth as i64)?;
    lua.globals().set(
        "ITEM_WILDGROWTH_PERSISTENT",
        ItemId::WildgrowthPersistent as i64,
    )?;
    lua.globals()
        .set("ITEM_WILDGROWTH_SAFE", ItemId::WildgrowthSafe as i64)?;
    // ── Misc item-system constants (no Rust enum yet; values mirror C++).
    // C++ `static constexpr uint8_t ITEM_STACK_SIZE = 100;` (game.h).
    lua.globals().set("ITEM_STACK_SIZE", 100i64)?;
    // C++ `ItemAttributes_t::ITEM_ATTRIBUTE_NONE = 0` (enums.h, first variant).
    lua.globals().set("ITEM_ATTRIBUTE_NONE", 0i64)?;
    // C++ alias `ITEM_ATTRIBUTE_DURATION_MIN = ITEM_ATTRIBUTE_DURATION = 1 << 17`.
    lua.globals()
        .set("ITEM_ATTRIBUTE_DURATION_MIN", 1i64 << 17)?;
    // C++ `ItemTypes_t` sequential enum (items.h). NONE = 0, then DEPOT,
    // MAILBOX, TRASHHOLDER, CONTAINER, DOOR, MAGICFIELD, TELEPORT, BED,
    // KEY, RUNE, PODIUM.
    lua.globals().set("ITEM_TYPE_DEPOT", 1i64)?;
    lua.globals().set("ITEM_TYPE_MAILBOX", 2i64)?;
    lua.globals().set("ITEM_TYPE_TRASHHOLDER", 3i64)?;
    lua.globals().set("ITEM_TYPE_CONTAINER", 4i64)?;
    lua.globals().set("ITEM_TYPE_DOOR", 5i64)?;
    lua.globals().set("ITEM_TYPE_MAGICFIELD", 6i64)?;
    lua.globals().set("ITEM_TYPE_TELEPORT", 7i64)?;
    lua.globals().set("ITEM_TYPE_BED", 8i64)?;
    lua.globals().set("ITEM_TYPE_KEY", 9i64)?;
    lua.globals().set("ITEM_TYPE_RUNE", 10i64)?;
    lua.globals().set("ITEM_TYPE_PODIUM", 11i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn item_id_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return ITEM_GOLD_COIN").eval().unwrap();
    }
}
