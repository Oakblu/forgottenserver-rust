//! `configKeys.*` table-scoped enum constants.
//!
//! Maps the C++ `ConfigManager::ConfigManager_t` boolean/string/integer
//! enum constants to Rust's split `BooleanKey` / `StringKey` /
//! `IntegerKey` enums in `common::configmanager`. The actual integer
//! value exposed to Lua is the Rust enum variant discriminant — the
//! Lua API (`Game.getConfigBoolean`/`Game.getConfigString`/`Game.
//! getConfigInteger`) handles the type-dispatched lookup back into
//! the ConfigManager.
//!
//! C++: `registerEnumIn(L, "configKeys", ConfigManager::ALLOW_CHANGEOUTFIT);`
//! Rust: `table.set("ALLOW_CHANGEOUTFIT", BooleanKey::AllowChangeOutfit as i64);`
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::configmanager::{BooleanKey, IntegerKey, StringKey};

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    let table = lua.create_table()?;
    table.set(
        "AIMBOT_HOTKEY_ENABLED",
        BooleanKey::AimbotHotkeyEnabled as i64,
    )?;
    table.set("ALLOW_CHANGEOUTFIT", BooleanKey::AllowChangeOutfit as i64)?;
    table.set("ALLOW_CLONES", BooleanKey::AllowClones as i64)?;
    table.set(
        "BIND_ONLY_GLOBAL_ADDRESS",
        BooleanKey::BindOnlyGlobalAddress as i64,
    )?;
    table.set(
        "CHECK_DUPLICATE_STORAGE_KEYS",
        BooleanKey::CheckDuplicateStorageKeys as i64,
    )?;
    table.set(
        "CLASSIC_ATTACK_SPEED",
        BooleanKey::ClassicAttackSpeed as i64,
    )?;
    table.set(
        "CLASSIC_EQUIPMENT_SLOTS",
        BooleanKey::ClassicEquipmentSlots as i64,
    )?;
    table.set(
        "CONVERT_UNSAFE_SCRIPTS",
        BooleanKey::ConvertUnsafeScripts as i64,
    )?;
    table.set("EMOTE_SPELLS", BooleanKey::EmoteSpells as i64)?;
    table.set(
        "EXPERIENCE_FROM_PLAYERS",
        BooleanKey::ExperienceFromPlayers as i64,
    )?;
    table.set("FREE_PREMIUM", BooleanKey::FreePremium as i64)?;
    table.set(
        "HOUSE_DOOR_SHOW_PRICE",
        BooleanKey::HouseDoorShowPrice as i64,
    )?;
    table.set(
        "MANASHIELD_BREAKABLE",
        BooleanKey::ManashieldBreakable as i64,
    )?;
    table.set("MARKET_PREMIUM", BooleanKey::MarketPremium as i64)?;
    table.set("MONSTER_OVERSPAWN", BooleanKey::MonsterOverspawn as i64)?;
    table.set(
        "ONE_PLAYER_ON_ACCOUNT",
        BooleanKey::OnePlayerOnAccount as i64,
    )?;
    table.set(
        "ONLINE_OFFLINE_CHARLIST",
        BooleanKey::OnlineOfflineCharlist as i64,
    )?;
    table.set("OPTIMIZE_DATABASE", BooleanKey::OptimizeDatabase as i64)?;
    table.set("REMOVE_ON_DESPAWN", BooleanKey::RemoveOnDespawn as i64)?;
    table.set(
        "REMOVE_POTION_CHARGES",
        BooleanKey::RemovePotionCharges as i64,
    )?;
    table.set("REMOVE_RUNE_CHARGES", BooleanKey::RemoveRuneCharges as i64)?;
    table.set("REMOVE_WEAPON_AMMO", BooleanKey::RemoveWeaponAmmo as i64)?;
    table.set(
        "REMOVE_WEAPON_CHARGES",
        BooleanKey::RemoveWeaponCharges as i64,
    )?;
    table.set(
        "REPLACE_KICK_ON_LOGIN",
        BooleanKey::ReplaceKickOnLogin as i64,
    )?;
    table.set(
        "SERVER_SAVE_CLEAN_MAP",
        BooleanKey::ServerSaveCleanMap as i64,
    )?;
    table.set("SERVER_SAVE_CLOSE", BooleanKey::ServerSaveClose as i64)?;
    table.set(
        "SERVER_SAVE_NOTIFY_MESSAGE",
        BooleanKey::ServerSaveNotifyMessage as i64,
    )?;
    table.set(
        "SERVER_SAVE_SHUTDOWN",
        BooleanKey::ServerSaveShutdown as i64,
    )?;
    table.set("STAMINA_SYSTEM", BooleanKey::StaminaSystem as i64)?;
    table.set("TWO_FACTOR_AUTH", BooleanKey::TwoFactorAuth as i64)?;
    table.set("WARN_UNSAFE_SCRIPTS", BooleanKey::WarnUnsafeScripts as i64)?;
    table.set("DEFAULT_PRIORITY", StringKey::DefaultPriority as i64)?;
    table.set("HOUSE_RENT_PERIOD", StringKey::HouseRentPeriod as i64)?;
    table.set("IP", StringKey::Ip as i64)?;
    table.set("LOCATION", StringKey::Location as i64)?;
    table.set("MAP_AUTHOR", StringKey::MapAuthor as i64)?;
    table.set("MAP_NAME", StringKey::MapName as i64)?;
    table.set("MYSQL_DB", StringKey::MysqlDb as i64)?;
    table.set("MYSQL_HOST", StringKey::MysqlHost as i64)?;
    table.set("MYSQL_PASS", StringKey::MysqlPass as i64)?;
    table.set("MYSQL_SOCK", StringKey::MysqlSock as i64)?;
    table.set("MYSQL_USER", StringKey::MysqlUser as i64)?;
    table.set("OWNER_EMAIL", StringKey::OwnerEmail as i64)?;
    table.set("OWNER_NAME", StringKey::OwnerName as i64)?;
    table.set("SERVER_NAME", StringKey::ServerName as i64)?;
    table.set("URL", StringKey::Url as i64)?;
    table.set("WORLD_TYPE", StringKey::WorldType as i64)?;
    table.set(
        "ACTIONS_DELAY_INTERVAL",
        IntegerKey::ActionsDelayInterval as i64,
    )?;
    table.set(
        "CHECK_EXPIRED_MARKET_OFFERS_EACH_MINUTES",
        IntegerKey::CheckExpiredMarketOffersEachMinutes as i64,
    )?;
    table.set("DEATH_LOSE_PERCENT", IntegerKey::DeathLosePercent as i64)?;
    table.set(
        "DEFAULT_DESPAWNRADIUS",
        IntegerKey::DefaultDespawnRadius as i64,
    )?;
    table.set(
        "DEFAULT_DESPAWNRANGE",
        IntegerKey::DefaultDespawnRange as i64,
    )?;
    table.set(
        "DEFAULT_WALKTOSPAWNRADIUS",
        IntegerKey::DefaultWalkToSpawnRadius as i64,
    )?;
    table.set(
        "EXP_FROM_PLAYERS_LEVEL_RANGE",
        IntegerKey::ExpFromPlayersLevelRange as i64,
    )?;
    table.set(
        "EX_ACTIONS_DELAY_INTERVAL",
        IntegerKey::ExActionsDelayInterval as i64,
    )?;
    table.set("FRAG_TIME", IntegerKey::FragTime as i64)?;
    table.set("GAME_PORT", IntegerKey::GamePort as i64)?;
    table.set("HOUSE_PRICE", IntegerKey::HousePrice as i64)?;
    table.set("KICK_AFTER_MINUTES", IntegerKey::KickAfterMinutes as i64)?;
    table.set("KILLS_TO_BLACK", IntegerKey::KillsToBlack as i64)?;
    table.set("KILLS_TO_RED", IntegerKey::KillsToRed as i64)?;
    table.set(
        "MARKET_OFFER_DURATION",
        IntegerKey::MarketOfferDuration as i64,
    )?;
    table.set(
        "MAX_MARKET_OFFERS_AT_A_TIME_PER_PLAYER",
        IntegerKey::MaxMarketOffersAtATimePerPlayer as i64,
    )?;
    table.set("MAX_MESSAGEBUFFER", IntegerKey::MaxMessageBuffer as i64)?;
    table.set(
        "MAX_PACKETS_PER_SECOND",
        IntegerKey::MaxPacketsPerSecond as i64,
    )?;
    table.set("MAX_PLAYERS", IntegerKey::MaxPlayers as i64)?;
    table.set("PROTECTION_LEVEL", IntegerKey::ProtectionLevel as i64)?;
    table.set("PZ_LOCKED", IntegerKey::PzLocked as i64)?;
    table.set(
        "QUEST_TRACKER_FREE_LIMIT",
        IntegerKey::QuestTrackerFreeLimit as i64,
    )?;
    table.set(
        "QUEST_TRACKER_PREMIUM_LIMIT",
        IntegerKey::QuestTrackerPremiumLimit as i64,
    )?;
    table.set("RATE_EXPERIENCE", IntegerKey::RateExperience as i64)?;
    table.set("RATE_LOOT", IntegerKey::RateLoot as i64)?;
    table.set("RATE_MAGIC", IntegerKey::RateMagic as i64)?;
    table.set("RATE_SKILL", IntegerKey::RateSkill as i64)?;
    table.set("RATE_SPAWN", IntegerKey::RateSpawn as i64)?;
    table.set(
        "SERVER_SAVE_NOTIFY_DURATION",
        IntegerKey::ServerSaveNotifyDuration as i64,
    )?;
    table.set("SQL_PORT", IntegerKey::SqlPort as i64)?;
    table.set("STAIRHOP_DELAY", IntegerKey::StairhopDelay as i64)?;
    table.set(
        "STAMINA_REGEN_MINUTE",
        IntegerKey::StaminaRegenMinute as i64,
    )?;
    table.set(
        "STAMINA_REGEN_PREMIUM",
        IntegerKey::StaminaRegenPremium as i64,
    )?;
    table.set("STATUSQUERY_TIMEOUT", IntegerKey::StatusQueryTimeout as i64)?;
    table.set("STATUS_PORT", IntegerKey::StatusPort as i64)?;
    table.set("WHITE_SKULL_TIME", IntegerKey::WhiteSkullTime as i64)?;
    lua.globals().set("configKeys", table)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_keys_table_registers() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        // Sanity: table exists, sample keys reachable.
        let v: i64 = lua
            .load("return configKeys.ALLOW_CHANGEOUTFIT")
            .eval()
            .unwrap();
        assert_eq!(v, BooleanKey::AllowChangeOutfit as i64);
        let v: i64 = lua.load("return configKeys.IP").eval().unwrap();
        assert_eq!(v, StringKey::Ip as i64);
        let v: i64 = lua.load("return configKeys.MAX_PLAYERS").eval().unwrap();
        assert_eq!(v, IntegerKey::MaxPlayers as i64);
    }
}
