//! `RESOURCE_*` enum constants. Source: `common::constants::ResourceType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::ResourceType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("RESOURCE_BANK_BALANCE", ResourceType::BankBalance as i64)?;
    lua.globals().set(
        "RESOURCE_DAILYREWARD_JOKERS",
        ResourceType::DailyRewardJokers as i64,
    )?;
    lua.globals().set(
        "RESOURCE_DAILYREWARD_STREAK",
        ResourceType::DailyRewardStreak as i64,
    )?;
    lua.globals()
        .set("RESOURCE_GOLD_EQUIPPED", ResourceType::GoldEquipped as i64)?;
    lua.globals().set(
        "RESOURCE_PREY_WILDCARDS",
        ResourceType::PreyWildcards as i64,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resource_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return RESOURCE_BANK_BALANCE").eval().unwrap();
        assert_eq!(v, 0);
    }
}
