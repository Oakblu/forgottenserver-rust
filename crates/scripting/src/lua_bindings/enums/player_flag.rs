//! `PlayerFlag_*` enum constants. Source: `forgottenserver_common::constants::PlayerFlags`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::PlayerFlags;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set(
        "PlayerFlag_CanAlwaysLogin",
        PlayerFlags::CAN_ALWAYS_LOGIN as i64,
    )?;
    lua.globals()
        .set("PlayerFlag_CanBroadcast", PlayerFlags::CAN_BROADCAST as i64)?;
    lua.globals().set(
        "PlayerFlag_CanConvinceAll",
        PlayerFlags::CAN_CONVINCE_ALL as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CanEditHouses",
        PlayerFlags::CAN_EDIT_HOUSES as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CanIllusionAll",
        PlayerFlags::CAN_ILLUSION_ALL as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CanPushAllCreatures",
        PlayerFlags::CAN_PUSH_ALL_CREATURES as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CanSenseInvisibility",
        PlayerFlags::CAN_SENSE_INVISIBILITY as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CanSummonAll",
        PlayerFlags::CAN_SUMMON_ALL as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CanTalkRedChannel",
        PlayerFlags::CAN_TALK_RED_CHANNEL as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CanTalkRedPrivate",
        PlayerFlags::CAN_TALK_RED_PRIVATE as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotAttackMonster",
        PlayerFlags::CANNOT_ATTACK_MONSTER as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotAttackPlayer",
        PlayerFlags::CANNOT_ATTACK_PLAYER as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotBeAttacked",
        PlayerFlags::CANNOT_BE_ATTACKED as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotBeBanned",
        PlayerFlags::CANNOT_BE_BANNED as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotBeMuted",
        PlayerFlags::CANNOT_BE_MUTED as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotBePushed",
        PlayerFlags::CANNOT_BE_PUSHED as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotPickupItem",
        PlayerFlags::CANNOT_PICKUP_ITEM as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotUseCombat",
        PlayerFlags::CANNOT_USE_COMBAT as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_CannotUseSpells",
        PlayerFlags::CANNOT_USE_SPELLS as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_HasInfiniteCapacity",
        PlayerFlags::HAS_INFINITE_CAPACITY as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_HasInfiniteMana",
        PlayerFlags::HAS_INFINITE_MANA as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_HasInfiniteSoul",
        PlayerFlags::HAS_INFINITE_SOUL as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_HasNoExhaustion",
        PlayerFlags::HAS_NO_EXHAUSTION as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_IgnoreProtectionZone",
        PlayerFlags::IGNORE_PROTECTION_ZONE as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_IgnoreSendPrivateCheck",
        PlayerFlags::IGNORE_SEND_PRIVATE_CHECK as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_IgnoreSpellCheck",
        PlayerFlags::IGNORE_SPELL_CHECK as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_IgnoreWeaponCheck",
        PlayerFlags::IGNORE_WEAPON_CHECK as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_IgnoreYellCheck",
        PlayerFlags::IGNORE_YELL_CHECK as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_IgnoredByMonsters",
        PlayerFlags::IGNORED_BY_MONSTERS as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_IsAlwaysPremium",
        PlayerFlags::IS_ALWAYS_PREMIUM as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_NotGainExperience",
        PlayerFlags::NOT_GAIN_EXPERIENCE as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_NotGainHealth",
        PlayerFlags::NOT_GAIN_HEALTH as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_NotGainInFight",
        PlayerFlags::NOT_GAIN_IN_FIGHT as i64,
    )?;
    lua.globals()
        .set("PlayerFlag_NotGainMana", PlayerFlags::NOT_GAIN_MANA as i64)?;
    lua.globals().set(
        "PlayerFlag_NotGainSkill",
        PlayerFlags::NOT_GAIN_SKILL as i64,
    )?;
    lua.globals().set(
        "PlayerFlag_NotGenerateLoot",
        PlayerFlags::NOT_GENERATE_LOOT as i64,
    )?;
    lua.globals()
        .set("PlayerFlag_SetMaxSpeed", PlayerFlags::SET_MAX_SPEED as i64)?;
    lua.globals()
        .set("PlayerFlag_SpecialVIP", PlayerFlags::SPECIAL_VIP as i64)?;
    lua.globals().set(
        "PlayerFlag_TalkOrangeHelpChannel",
        PlayerFlags::TALK_ORANGE_HELP_CHANNEL as i64,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_flag_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return PlayerFlag_CanAlwaysLogin").eval().unwrap();
    }
}
