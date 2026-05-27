//! `GAME_STATE_*` enum constants. Source: `game::game::GameState`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_game::game::GameState;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("GAME_STATE_CLOSED", GameState::Closed as i64)?;
    lua.globals()
        .set("GAME_STATE_CLOSING", GameState::Closing as i64)?;
    lua.globals()
        .set("GAME_STATE_INIT", GameState::Init as i64)?;
    lua.globals()
        .set("GAME_STATE_MAINTAIN", GameState::Maintain as i64)?;
    lua.globals()
        .set("GAME_STATE_NORMAL", GameState::Normal as i64)?;
    lua.globals()
        .set("GAME_STATE_STARTUP", GameState::Startup as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_state_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua.load("return GAME_STATE_NORMAL").eval().unwrap();
    }
}
