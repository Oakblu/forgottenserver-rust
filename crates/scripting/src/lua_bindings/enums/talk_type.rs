//! `TALKTYPE_*` enum constants.
//! Source: `common::constants::SpeakClass`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::SpeakClass;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("TALKTYPE_BROADCAST", SpeakClass::Broadcast as i64)?;
    lua.globals()
        .set("TALKTYPE_CHANNEL_O", SpeakClass::ChannelO as i64)?;
    lua.globals()
        .set("TALKTYPE_CHANNEL_R1", SpeakClass::ChannelR1 as i64)?;
    lua.globals()
        .set("TALKTYPE_CHANNEL_Y", SpeakClass::ChannelY as i64)?;
    lua.globals()
        .set("TALKTYPE_MONSTER_SAY", SpeakClass::MonsterSay as i64)?;
    lua.globals()
        .set("TALKTYPE_MONSTER_YELL", SpeakClass::MonsterYell as i64)?;
    lua.globals()
        .set("TALKTYPE_POTION", SpeakClass::Potion as i64)?;
    lua.globals()
        .set("TALKTYPE_PRIVATE_FROM", SpeakClass::PrivateFrom as i64)?;
    lua.globals()
        .set("TALKTYPE_PRIVATE_NP", SpeakClass::PrivateNp as i64)?;
    lua.globals().set(
        "TALKTYPE_PRIVATE_NP_CONSOLE",
        SpeakClass::PrivateNpConsole as i64,
    )?;
    lua.globals()
        .set("TALKTYPE_PRIVATE_PN", SpeakClass::PrivatePn as i64)?;
    lua.globals().set(
        "TALKTYPE_PRIVATE_RED_FROM",
        SpeakClass::PrivateRedFrom as i64,
    )?;
    lua.globals()
        .set("TALKTYPE_PRIVATE_RED_TO", SpeakClass::PrivateRedTo as i64)?;
    lua.globals()
        .set("TALKTYPE_PRIVATE_TO", SpeakClass::PrivateTo as i64)?;
    lua.globals().set("TALKTYPE_SAY", SpeakClass::Say as i64)?;
    lua.globals()
        .set("TALKTYPE_SPELL", SpeakClass::Spell as i64)?;
    lua.globals()
        .set("TALKTYPE_WHISPER", SpeakClass::Whisper as i64)?;
    lua.globals()
        .set("TALKTYPE_YELL", SpeakClass::Yell as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn talk_type_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return TALKTYPE_SAY").eval().unwrap();
        assert_eq!(v, 1);
        for name in ["TALKTYPE_SAY", "TALKTYPE_WHISPER", "TALKTYPE_YELL"] {
            lua.load(format!("return {name}")).eval::<i64>().unwrap();
        }
    }
}
