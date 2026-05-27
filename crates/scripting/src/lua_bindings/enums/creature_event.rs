//! `CREATURE_EVENT_*` enum constants. Source: `scripting::creatureevent::CreatureEventType`.
//! `CREATURE_EVENT_NONE` is deferred — not a separate Rust variant.
#![cfg(feature = "lua-scripting")]

use crate::creatureevent::CreatureEventType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CREATURE_EVENT_ADVANCE", CreatureEventType::Advance as i64)?;
    lua.globals()
        .set("CREATURE_EVENT_DEATH", CreatureEventType::Death as i64)?;
    lua.globals().set(
        "CREATURE_EVENT_EXTENDED_OPCODE",
        CreatureEventType::ExtendedOpcode as i64,
    )?;
    lua.globals().set(
        "CREATURE_EVENT_HEALTHCHANGE",
        CreatureEventType::HealthChange as i64,
    )?;
    lua.globals()
        .set("CREATURE_EVENT_KILL", CreatureEventType::Kill as i64)?;
    lua.globals()
        .set("CREATURE_EVENT_LOGIN", CreatureEventType::Login as i64)?;
    lua.globals()
        .set("CREATURE_EVENT_LOGOUT", CreatureEventType::Logout as i64)?;
    lua.globals().set(
        "CREATURE_EVENT_MANACHANGE",
        CreatureEventType::ManaChange as i64,
    )?;
    lua.globals().set(
        "CREATURE_EVENT_MODALWINDOW",
        CreatureEventType::ModalWindow as i64,
    )?;
    lua.globals().set(
        "CREATURE_EVENT_PREPAREDEATH",
        CreatureEventType::PrepareDeath as i64,
    )?;
    lua.globals().set(
        "CREATURE_EVENT_RECONNECT",
        CreatureEventType::Reconnect as i64,
    )?;
    lua.globals().set(
        "CREATURE_EVENT_TEXTEDIT",
        CreatureEventType::TextEdit as i64,
    )?;
    lua.globals()
        .set("CREATURE_EVENT_THINK", CreatureEventType::Think as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creature_event_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return CREATURE_EVENT_LOGIN").eval().unwrap();
        assert_eq!(v, 0);
    }
}
