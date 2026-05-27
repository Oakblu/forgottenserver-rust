//! `RELOAD_TYPE_*` enum constants. Source: `common::enums::ReloadType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::ReloadType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("RELOAD_TYPE_ALL", ReloadType::All as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_ACTIONS", ReloadType::Actions as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_CHAT", ReloadType::Chat as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_CONFIG", ReloadType::Config as i64)?;
    lua.globals().set(
        "RELOAD_TYPE_CREATURESCRIPTS",
        ReloadType::CreatureScripts as i64,
    )?;
    lua.globals()
        .set("RELOAD_TYPE_EVENTS", ReloadType::Events as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_GLOBAL", ReloadType::Global as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_GLOBALEVENTS", ReloadType::GlobalEvents as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_ITEMS", ReloadType::Items as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_MONSTERS", ReloadType::Monsters as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_MOUNTS", ReloadType::Mounts as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_MOVEMENTS", ReloadType::Movements as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_NPCS", ReloadType::Npcs as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_QUESTS", ReloadType::Quests as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_SCRIPTS", ReloadType::Scripts as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_SPELLS", ReloadType::Spells as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_TALKACTIONS", ReloadType::TalkActions as i64)?;
    lua.globals()
        .set("RELOAD_TYPE_WEAPONS", ReloadType::Weapons as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reload_type_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return RELOAD_TYPE_ALL").eval().unwrap();
        assert_eq!(v, 0);
        let v: i64 = lua.load("return RELOAD_TYPE_WEAPONS").eval().unwrap();
        assert_eq!(v, 17);
    }
}
