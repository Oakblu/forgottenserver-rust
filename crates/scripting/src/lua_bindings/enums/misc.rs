//! Scattered single-constant `*_NONE`, `*_FIRST`/`*_LAST` markers,
//! house-list IDs, and other one-off C++ Lua-registered enums that don't
//! warrant their own module. Values mirror C++ exactly — see the inline
//! source-of-truth comments.

#![cfg(feature = "lua-scripting")]

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    // creature.h: `static constexpr uint32_t CREATURE_ID_MIN = 0x10000000;`
    lua.globals().set("CREATURE_ID_MIN", 0x1000_0000_i64)?;
    // creature.h: `static constexpr uint32_t CREATURE_ID_MAX = u32::MAX;`
    lua.globals().set("CREATURE_ID_MAX", u32::MAX as i64)?;
    // creatureevent.h: `CREATURE_EVENT_NONE = 0` (first variant).
    lua.globals().set("CREATURE_EVENT_NONE", 0i64)?;
    // const.h: CREATURE_ICON_FIRST = CrossWhite = 1, LAST = CrossRed = 21.
    lua.globals().set(
        "CREATURE_ICON_FIRST",
        forgottenserver_common::constants::CreatureIcon::FIRST as i64,
    )?;
    lua.globals().set(
        "CREATURE_ICON_LAST",
        forgottenserver_common::constants::CreatureIcon::LAST as i64,
    )?;
    // const.h: MONSTER_ICON_FIRST = Vulnerable = 1, LAST = Fiendish = 5.
    lua.globals().set(
        "MONSTER_ICON_FIRST",
        forgottenserver_common::constants::MonsterIcon::FIRST as i64,
    )?;
    lua.globals().set(
        "MONSTER_ICON_LAST",
        forgottenserver_common::constants::MonsterIcon::LAST as i64,
    )?;
    // item.h: DECAYING_FALSE = 0, DECAYING_TRUE = 1, DECAYING_PENDING = 2.
    lua.globals().set(
        "DECAYING_FALSE",
        forgottenserver_items::item::ItemDecayState::False as i64,
    )?;
    lua.globals().set(
        "DECAYING_TRUE",
        forgottenserver_items::item::ItemDecayState::True as i64,
    )?;
    lua.globals().set(
        "DECAYING_PENDING",
        forgottenserver_items::item::ItemDecayState::Pending as i64,
    )?;
    // game.h: GAME_STATE_SHUTDOWN (C++ sixth-after-MAINTAIN sequential value = 6).
    lua.globals().set("GAME_STATE_SHUTDOWN", 6i64)?;
    // house.h: GUEST_LIST = 0x100, SUBOWNER_LIST = 0x101.
    lua.globals().set("GUEST_LIST", 0x100_i64)?;
    lua.globals().set("SUBOWNER_LIST", 0x101_i64)?;
    // monsters.h: const uint32_t MAX_LOOTCHANCE = 100000.
    lua.globals().set("MAX_LOOTCHANCE", 100_000_i64)?;
    // enums.h: inline constexpr uint16_t VOCATION_NONE = 0.
    lua.globals().set("VOCATION_NONE", 0i64)?;
    // tile.h: ZONE_NOLOGOUT (bit-3 in C++ ZoneType bitmask).
    lua.globals().set("ZONE_NOLOGOUT", 3i64)?;
    Ok(())
}
