//! `MAPMARK_*` enum constants. Source: `common::enums::MapMark`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::MapMark;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("MAPMARK_TICK", MapMark::Tick as i64)?;
    lua.globals()
        .set("MAPMARK_QUESTION", MapMark::Question as i64)?;
    lua.globals()
        .set("MAPMARK_EXCLAMATION", MapMark::Exclamation as i64)?;
    lua.globals().set("MAPMARK_STAR", MapMark::Star as i64)?;
    lua.globals().set("MAPMARK_CROSS", MapMark::Cross as i64)?;
    lua.globals()
        .set("MAPMARK_TEMPLE", MapMark::Temple as i64)?;
    lua.globals().set("MAPMARK_KISS", MapMark::Kiss as i64)?;
    lua.globals()
        .set("MAPMARK_SHOVEL", MapMark::Shovel as i64)?;
    lua.globals().set("MAPMARK_SWORD", MapMark::Sword as i64)?;
    lua.globals().set("MAPMARK_FLAG", MapMark::Flag as i64)?;
    lua.globals().set("MAPMARK_LOCK", MapMark::Lock as i64)?;
    lua.globals().set("MAPMARK_BAG", MapMark::Bag as i64)?;
    lua.globals().set("MAPMARK_SKULL", MapMark::Skull as i64)?;
    lua.globals()
        .set("MAPMARK_DOLLAR", MapMark::Dollar as i64)?;
    lua.globals()
        .set("MAPMARK_REDNORTH", MapMark::RedNorth as i64)?;
    lua.globals()
        .set("MAPMARK_REDSOUTH", MapMark::RedSouth as i64)?;
    lua.globals()
        .set("MAPMARK_REDEAST", MapMark::RedEast as i64)?;
    lua.globals()
        .set("MAPMARK_REDWEST", MapMark::RedWest as i64)?;
    lua.globals()
        .set("MAPMARK_GREENNORTH", MapMark::GreenNorth as i64)?;
    lua.globals()
        .set("MAPMARK_GREENSOUTH", MapMark::GreenSouth as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn map_mark_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return MAPMARK_TICK").eval().unwrap();
        assert_eq!(v, 0);
        let v: i64 = lua.load("return MAPMARK_GREENSOUTH").eval().unwrap();
        assert_eq!(v, 19);
    }
}
