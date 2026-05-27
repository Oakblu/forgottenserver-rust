//! `CONDITIONID_*` enum constants. Source: `common::enums::ConditionId`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::ConditionId;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CONDITIONID_DEFAULT", ConditionId::Default as i64)?;
    lua.globals()
        .set("CONDITIONID_COMBAT", ConditionId::Combat as i64)?;
    lua.globals()
        .set("CONDITIONID_HEAD", ConditionId::Head as i64)?;
    lua.globals()
        .set("CONDITIONID_NECKLACE", ConditionId::Necklace as i64)?;
    lua.globals()
        .set("CONDITIONID_BACKPACK", ConditionId::Backpack as i64)?;
    lua.globals()
        .set("CONDITIONID_ARMOR", ConditionId::Armor as i64)?;
    lua.globals()
        .set("CONDITIONID_RIGHT", ConditionId::Right as i64)?;
    lua.globals()
        .set("CONDITIONID_LEFT", ConditionId::Left as i64)?;
    lua.globals()
        .set("CONDITIONID_LEGS", ConditionId::Legs as i64)?;
    lua.globals()
        .set("CONDITIONID_FEET", ConditionId::Feet as i64)?;
    lua.globals()
        .set("CONDITIONID_RING", ConditionId::Ring as i64)?;
    lua.globals()
        .set("CONDITIONID_AMMO", ConditionId::Ammo as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn condition_id_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return CONDITIONID_DEFAULT").eval().unwrap();
        assert_eq!(v, -1);
        let v: i64 = lua.load("return CONDITIONID_COMBAT").eval().unwrap();
        assert_eq!(v, 0);
        let v: i64 = lua.load("return CONDITIONID_AMMO").eval().unwrap();
        assert_eq!(v, 10);
    }
}
