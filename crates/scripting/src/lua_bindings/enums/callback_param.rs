//! `CALLBACK_PARAM_*` enum constants. Source: `forgottenserver_common::enums::CallBackParam`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::CallBackParam;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set(
        "CALLBACK_PARAM_LEVELMAGICVALUE",
        CallBackParam::LevelMagicValue as i64,
    )?;
    lua.globals().set(
        "CALLBACK_PARAM_SKILLVALUE",
        CallBackParam::SkillValue as i64,
    )?;
    lua.globals().set(
        "CALLBACK_PARAM_TARGETCREATURE",
        CallBackParam::TargetCreature as i64,
    )?;
    lua.globals().set(
        "CALLBACK_PARAM_TARGETTILE",
        CallBackParam::TargetTile as i64,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_param_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua
            .load("return CALLBACK_PARAM_TARGETCREATURE")
            .eval()
            .unwrap();
    }
}
