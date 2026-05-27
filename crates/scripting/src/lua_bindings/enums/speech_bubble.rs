//! `SPEECHBUBBLE_*` enum constants. Source: `common::enums::SpeechBubble`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::SpeechBubble;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("SPEECHBUBBLE_NONE", SpeechBubble::None as i64)?;
    lua.globals()
        .set("SPEECHBUBBLE_NORMAL", SpeechBubble::Normal as i64)?;
    lua.globals()
        .set("SPEECHBUBBLE_TRADE", SpeechBubble::Trade as i64)?;
    lua.globals()
        .set("SPEECHBUBBLE_QUEST", SpeechBubble::Quest as i64)?;
    lua.globals()
        .set("SPEECHBUBBLE_COMPASS", SpeechBubble::Compass as i64)?;
    lua.globals()
        .set("SPEECHBUBBLE_NORMAL2", SpeechBubble::Normal2 as i64)?;
    lua.globals()
        .set("SPEECHBUBBLE_NORMAL3", SpeechBubble::Normal3 as i64)?;
    lua.globals()
        .set("SPEECHBUBBLE_HIRELING", SpeechBubble::Hireling as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn speech_bubble_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return SPEECHBUBBLE_NONE").eval().unwrap();
        assert_eq!(v, 0);
        let v: i64 = lua.load("return SPEECHBUBBLE_HIRELING").eval().unwrap();
        assert_eq!(v, 7);
    }
}
