//! `RETURNVALUE_*` enum constants.
//! Source: `common::enums::ReturnValue`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::ReturnValue;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set(
        "RETURNVALUE_ACTIONNOTPERMITTEDINANOPVPZONE",
        ReturnValue::ActionNotPermittedInAnoPvpZone as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_ACTIONNOTPERMITTEDINPROTECTIONZONE",
        ReturnValue::ActionNotPermittedInProtectionZone as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_BOTHHANDSNEEDTOBEFREE",
        ReturnValue::BothHandsNeedToBeFree as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CANNOTBEDRESSED",
        ReturnValue::CannotBeDressed as i64,
    )?;
    lua.globals()
        .set("RETURNVALUE_CANNOTPICKUP", ReturnValue::CannotPickup as i64)?;
    lua.globals()
        .set("RETURNVALUE_CANNOTTHROW", ReturnValue::CannotThrow as i64)?;
    lua.globals().set(
        "RETURNVALUE_CANNOTUSETHISOBJECT",
        ReturnValue::CannotUseThisObject as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CANONLYUSEONESHIELD",
        ReturnValue::CanOnlyUseOneShield as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CANONLYUSEONEWEAPON",
        ReturnValue::CanOnlyUseOneWeapon as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CANONLYUSETHISRUNEONCREATURES",
        ReturnValue::CanOnlyUseThisRuneOnCreatures as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CONTAINERNOTENOUGHROOM",
        ReturnValue::ContainerNotEnoughRoom as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CREATUREBLOCK",
        ReturnValue::CreatureBlock as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CREATUREDOESNOTEXIST",
        ReturnValue::CreatureDoesNotExist as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_CREATUREISNOTREACHABLE",
        ReturnValue::CreatureIsNotReachable as i64,
    )?;
    lua.globals()
        .set("RETURNVALUE_DEPOTISFULL", ReturnValue::DepotIsFull as i64)?;
    lua.globals().set(
        "RETURNVALUE_DESTINATIONOUTOFREACH",
        ReturnValue::DestinationOutOfReach as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_DIRECTPLAYERSHOOT",
        ReturnValue::DirectPlayerShoot as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_DROPTWOHANDEDITEM",
        ReturnValue::DropTwoHandedItem as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_FIRSTGODOWNSTAIRS",
        ReturnValue::FirstGoDownstairs as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_FIRSTGOUPSTAIRS",
        ReturnValue::FirstGoUpstairs as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_NAMEISTOOAMBIGUOUS",
        ReturnValue::NameIsTooAmbiguous as i64,
    )?;
    lua.globals()
        .set("RETURNVALUE_NEEDEXCHANGE", ReturnValue::NeedExchange as i64)?;
    lua.globals()
        .set("RETURNVALUE_NOERROR", ReturnValue::NoError as i64)?;
    lua.globals().set(
        "RETURNVALUE_NOPARTYMEMBERSINRANGE",
        ReturnValue::NoPartyMembersInRange as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_NOTENOUGHCAPACITY",
        ReturnValue::NotEnoughCapacity as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_NOTENOUGHLEVEL",
        ReturnValue::NotEnoughLevel as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_NOTENOUGHMAGICLEVEL",
        ReturnValue::NotEnoughMagicLevel as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_NOTENOUGHMANA",
        ReturnValue::NotEnoughMana as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_NOTENOUGHROOM",
        ReturnValue::NotEnoughRoom as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_NOTENOUGHSOUL",
        ReturnValue::NotEnoughSoul as i64,
    )?;
    lua.globals()
        .set("RETURNVALUE_NOTMOVEABLE", ReturnValue::NotMoveable as i64)?;
    lua.globals()
        .set("RETURNVALUE_NOTPOSSIBLE", ReturnValue::NotPossible as i64)?;
    lua.globals().set(
        "RETURNVALUE_NOTREQUIREDLEVELTOUSERUNE",
        ReturnValue::NotRequiredLevelToUseRune as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PLAYERISNOTINVITED",
        ReturnValue::PlayerIsNotInvited as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PLAYERISNOTREACHABLE",
        ReturnValue::PlayerIsNotReachable as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PLAYERISPZLOCKED",
        ReturnValue::PlayerIsPzLocked as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PLAYERISPZLOCKEDENTERPVPZONE",
        ReturnValue::PlayerIsPzLockedEnterPvpZone as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PLAYERISPZLOCKEDLEAVEPVPZONE",
        ReturnValue::PlayerIsPzLockedLeavePvpZone as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PLAYERWITHTHISNAMEISNOTONLINE",
        ReturnValue::PlayerWithThisNameIsNotOnline as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PUTTHISOBJECTINBOTHHANDS",
        ReturnValue::PutThisObjectInBothHands as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_PUTTHISOBJECTINYOURHAND",
        ReturnValue::PutThisObjectInYourHand as i64,
    )?;
    lua.globals()
        .set("RETURNVALUE_THEREISNOWAY", ReturnValue::ThereIsNoWay as i64)?;
    lua.globals().set(
        "RETURNVALUE_THISISIMPOSSIBLE",
        ReturnValue::ThisIsImpossible as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_THISPLAYERISALREADYTRADING",
        ReturnValue::ThisPlayerIsAlreadyTrading as i64,
    )?;
    lua.globals()
        .set("RETURNVALUE_TOOFARAWAY", ReturnValue::TooFarAway as i64)?;
    lua.globals().set(
        "RETURNVALUE_TRADEPLAYERALREADYOWNSAHOUSE",
        ReturnValue::TradePlayerAlreadyOwnsAHouse as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_TRADEPLAYERFARAWAY",
        ReturnValue::TradePlayerFarAway as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_TRADEPLAYERHIGHESTBIDDER",
        ReturnValue::TradePlayerHighestBidder as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_TURNSECUREMODETOATTACKUNMARKEDPLAYERS",
        ReturnValue::TurnSecureModeToAttackUnmarkedPlayers as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUAREALREADYTRADING",
        ReturnValue::YouAreAlreadyTrading as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUAREEXHAUSTED",
        ReturnValue::YouAreExhausted as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUARENOTTHEOWNER",
        ReturnValue::YouAreNotTheOwner as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUCANNOTLOGOUTHERE",
        ReturnValue::YouCannotLogoutHere as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUCANNOTTRADETHISHOUSE",
        ReturnValue::YouCannotTradeThisHouse as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUCANNOTUSEOBJECTSTHATFAST",
        ReturnValue::YouCannotUseObjectsThatFast as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUCANNOTUSETHISBED",
        ReturnValue::YouCannotUseThisBed as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUCANONLYUSEITONCREATURES",
        ReturnValue::YouCanOnlyUseItOnCreatures as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUDONTHAVEREQUIREDPROFESSION",
        ReturnValue::YouDontHaveRequiredProfession as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUDONTOWNTHISHOUSE",
        ReturnValue::YouDontOwnThisHouse as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUMAYNOTATTACKAPERSONINPROTECTIONZONE",
        ReturnValue::YouMayNotAttackAPersonInProtectionZone as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUMAYNOTATTACKAPERSONWHILEINPROTECTIONZONE",
        ReturnValue::YouMayNotAttackAPersonWhileInProtectionZone as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUMAYNOTATTACKTHISCREATURE",
        ReturnValue::YouMayNotAttackThisCreature as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUMAYNOTATTACKTHISPLAYER",
        ReturnValue::YouMayNotAttackThisPlayer as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUMAYNOTLOGOUTDURINGAFIGHT",
        ReturnValue::YouMayNotLogoutDuringAFight as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUNEEDAMAGICITEMTOCASTSPELL",
        ReturnValue::YouNeedAMagicItemToCastSpell as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUNEEDAWEAPONTOUSETHISSPELL",
        ReturnValue::YouNeedAWeaponToUseThisSpell as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUNEEDPREMIUMACCOUNT",
        ReturnValue::YouNeedPremiumAccount as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOUNEEDTOLEARNTHISSPELL",
        ReturnValue::YouNeedToLearnThisSpell as i64,
    )?;
    lua.globals().set(
        "RETURNVALUE_YOURVOCATIONCANNOTUSETHISSPELL",
        ReturnValue::YourVocationCannotUseThisSpell as i64,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn return_value_noerror_is_zero() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return RETURNVALUE_NOERROR").eval().unwrap();
        assert_eq!(v, 0);
    }
    #[test]
    fn return_value_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        for name in [
            "RETURNVALUE_NOERROR",
            "RETURNVALUE_NOTPOSSIBLE",
            "RETURNVALUE_NOTENOUGHCAPACITY",
            "RETURNVALUE_PLAYERISPZLOCKED",
        ] {
            lua.load(format!("return {name}")).eval::<i64>().unwrap();
        }
    }
}
