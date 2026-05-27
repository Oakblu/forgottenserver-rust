//! `MONSTERS_EVENT_*` enum constants. Source: `common::enums::MonstersEvent`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::MonstersEvent;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("MONSTERS_EVENT_APPEAR", MonstersEvent::Appear as i64)?;
    lua.globals()
        .set("MONSTERS_EVENT_DISAPPEAR", MonstersEvent::Disappear as i64)?;
    lua.globals()
        .set("MONSTERS_EVENT_MOVE", MonstersEvent::Move as i64)?;
    lua.globals()
        .set("MONSTERS_EVENT_SAY", MonstersEvent::Say as i64)?;
    lua.globals()
        .set("MONSTERS_EVENT_THINK", MonstersEvent::Think as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn monsters_event_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return MONSTERS_EVENT_THINK").eval().unwrap();
        assert_eq!(v, 1);
    }
}
