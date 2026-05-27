//! `ACCOUNT_TYPE_*` enum constants.
//!
//! Source: `forgottenserver_common::enums::AccountType`.
//! C++: `enum AccountType_t { ACCOUNT_TYPE_NORMAL = 1, … };`

#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::AccountType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    // Inline `lua.globals().set(...)` form so the harness static-audit
    // parser detects each binding as a GlobalEnum.
    lua.globals()
        .set("ACCOUNT_TYPE_NORMAL", AccountType::Normal as i64)?;
    lua.globals()
        .set("ACCOUNT_TYPE_TUTOR", AccountType::Tutor as i64)?;
    lua.globals()
        .set("ACCOUNT_TYPE_SENIORTUTOR", AccountType::SeniorTutor as i64)?;
    lua.globals()
        .set("ACCOUNT_TYPE_GAMEMASTER", AccountType::GameMaster as i64)?;
    lua.globals().set(
        "ACCOUNT_TYPE_COMMUNITYMANAGER",
        AccountType::CommunityManager as i64,
    )?;
    lua.globals()
        .set("ACCOUNT_TYPE_GOD", AccountType::God as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_type_normal_registers_with_value_1() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return ACCOUNT_TYPE_NORMAL").eval().unwrap();
        assert_eq!(v, 1);
    }

    #[test]
    fn account_type_god_registers_with_value_6() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return ACCOUNT_TYPE_GOD").eval().unwrap();
        assert_eq!(v, 6);
    }

    #[test]
    fn all_six_account_types_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        for name in [
            "ACCOUNT_TYPE_NORMAL",
            "ACCOUNT_TYPE_TUTOR",
            "ACCOUNT_TYPE_SENIORTUTOR",
            "ACCOUNT_TYPE_GAMEMASTER",
            "ACCOUNT_TYPE_COMMUNITYMANAGER",
            "ACCOUNT_TYPE_GOD",
        ] {
            let v: i64 = lua
                .load(format!("return {name}"))
                .eval()
                .unwrap_or_else(|_| panic!("missing {name}"));
            assert!((1..=6).contains(&v), "{name} = {v}");
        }
    }
}
