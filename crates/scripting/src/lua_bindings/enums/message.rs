//! `MESSAGE_*` enum constants.
//! Source: `common::constants::MessageClass`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::MessageClass;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("MESSAGE_ATTENTION", MessageClass::Attention as i64)?;
    lua.globals()
        .set("MESSAGE_BEYOND_LAST", MessageClass::BeyondLast as i64)?;
    lua.globals().set(
        "MESSAGE_BOOSTED_CREATURE",
        MessageClass::BoostedCreature as i64,
    )?;
    lua.globals()
        .set("MESSAGE_DAMAGE_DEALT", MessageClass::DamageDealt as i64)?;
    lua.globals()
        .set("MESSAGE_DAMAGE_OTHERS", MessageClass::DamageOthers as i64)?;
    lua.globals().set(
        "MESSAGE_DAMAGE_RECEIVED",
        MessageClass::DamageReceived as i64,
    )?;
    lua.globals()
        .set("MESSAGE_EVENT_ADVANCE", MessageClass::EventAdvance as i64)?;
    lua.globals()
        .set("MESSAGE_EVENT_DEFAULT", MessageClass::EventDefault as i64)?;
    lua.globals()
        .set("MESSAGE_EXPERIENCE", MessageClass::Experience as i64)?;
    lua.globals().set(
        "MESSAGE_EXPERIENCE_OTHERS",
        MessageClass::ExperienceOthers as i64,
    )?;
    lua.globals()
        .set("MESSAGE_GUILD", MessageClass::Guild as i64)?;
    lua.globals()
        .set("MESSAGE_HEALED", MessageClass::Healed as i64)?;
    lua.globals()
        .set("MESSAGE_HEALED_OTHERS", MessageClass::HealedOthers as i64)?;
    lua.globals()
        .set("MESSAGE_HOTKEY_PRESSED", MessageClass::HotkeyPressed as i64)?;
    lua.globals()
        .set("MESSAGE_INFO_DESCR", MessageClass::InfoDescr as i64)?;
    lua.globals()
        .set("MESSAGE_LOOT", MessageClass::Loot as i64)?;
    lua.globals()
        .set("MESSAGE_MARKET", MessageClass::Market as i64)?;
    lua.globals().set(
        "MESSAGE_OFFLINE_TRAINING",
        MessageClass::OfflineTraining as i64,
    )?;
    lua.globals()
        .set("MESSAGE_PARTY", MessageClass::Party as i64)?;
    lua.globals().set(
        "MESSAGE_PARTY_MANAGEMENT",
        MessageClass::PartyManagement as i64,
    )?;
    lua.globals()
        .set("MESSAGE_REPORT", MessageClass::Report as i64)?;
    lua.globals()
        .set("MESSAGE_STATUS_DEFAULT", MessageClass::StatusDefault as i64)?;
    lua.globals()
        .set("MESSAGE_STATUS_SMALL", MessageClass::StatusSmall as i64)?;
    lua.globals()
        .set("MESSAGE_STATUS_WARNING", MessageClass::StatusWarning as i64)?;
    lua.globals().set(
        "MESSAGE_STATUS_WARNING2",
        MessageClass::StatusWarning2 as i64,
    )?;
    lua.globals().set(
        "MESSAGE_TOURNAMENT_INFO",
        MessageClass::TournamentInfo as i64,
    )?;
    lua.globals()
        .set("MESSAGE_TRADE", MessageClass::Trade as i64)?;
    lua.globals()
        .set("MESSAGE_TRANSACTION", MessageClass::Transaction as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn message_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        for name in [
            "MESSAGE_EVENT_ADVANCE",
            "MESSAGE_DAMAGE_DEALT",
            "MESSAGE_GUILD",
            "MESSAGE_MARKET",
        ] {
            lua.load(format!("return {name}")).eval::<i64>().unwrap();
        }
    }
}
