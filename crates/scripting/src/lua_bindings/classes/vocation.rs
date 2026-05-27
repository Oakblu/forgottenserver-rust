//! `Vocation:*` Lua binding for `items::vocation::Vocation`.
//!
//! C++ Lua side exposes 21 methods on Vocation, mostly getters
//! plus computed methods (`getRequiredSkillTries`,
//! `getRequiredManaSpent`) and the `allowsPvp` predicate.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_items::vocation::Vocation;
use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone)]
pub struct LuaVocation(pub Vocation);

impl LuaVocation {
    pub fn new(v: Vocation) -> Self {
        Self(v)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaVocation {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaVocation>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaVocation",
                message: Some("expected Vocation userdata".into()),
            }),
        }
    }
}

impl UserData for LuaVocation {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaVocation| {
            Ok(this.0.id == other.0.id)
        });
        methods.add_method("getId", |_, this, ()| Ok(this.0.id as i64));
        methods.add_method("getClientId", |_, this, ()| Ok(this.0.client_id as i64));
        methods.add_method("getName", |_, this, ()| Ok(this.0.name.clone()));
        methods.add_method("getDescription", |_, this, ()| {
            Ok(this.0.description.clone())
        });
        methods.add_method("getAttackSpeed", |_, this, ()| {
            Ok(this.0.attack_speed as i64)
        });
        methods.add_method("getBaseSpeed", |_, this, ()| Ok(this.0.base_speed as i64));
        methods.add_method("getCapacityGain", |_, this, ()| Ok(this.0.gain_cap as i64));
        methods.add_method("getHealthGain", |_, this, ()| Ok(this.0.gain_hp as i64));
        methods.add_method("getHealthGainAmount", |_, this, ()| {
            Ok(this.0.gain_health_amount as i64)
        });
        methods.add_method("getHealthGainTicks", |_, this, ()| {
            Ok(this.0.gain_health_ticks as i64)
        });
        methods.add_method("getManaGain", |_, this, ()| Ok(this.0.gain_mana as i64));
        methods.add_method("getManaGainAmount", |_, this, ()| {
            Ok(this.0.gain_mana_amount as i64)
        });
        methods.add_method("getManaGainTicks", |_, this, ()| {
            Ok(this.0.gain_mana_ticks as i64)
        });
        methods.add_method("getMaxSoul", |_, this, ()| Ok(this.0.soul_max as i64));
        methods.add_method("getSoulGainTicks", |_, this, ()| {
            Ok(this.0.gain_soul_ticks as i64)
        });
        methods.add_method("allowsPvp", |_, this, ()| Ok(this.0.allow_pvp));
        // Promotion/demotion return the from_vocation id (C++: returns the
        // promoted/demoted Vocation*; we expose just the id since we
        // don't have access to the Vocations registry from here).
        methods.add_method(
            "getPromotion",
            |_, this, ()| Ok(this.0.from_vocation as i64),
        );
        methods.add_method("getDemotion", |_, this, ()| Ok(this.0.from_vocation as i64));
        // Computed: matches C++ Vocation::getReqSkillTries formula
        // (10 * level^3 * skillMultipliers[skill]).
        methods.add_method(
            "getRequiredSkillTries",
            |_, this, (skill, level): (i64, i64)| {
                let idx = skill.clamp(0, 6) as usize;
                let mult = this.0.skill_multipliers[idx];
                let lvl = level.max(0) as f64;
                Ok((10.0 * lvl.powi(3) * mult) as i64)
            },
        );
        methods.add_method("getRequiredManaSpent", |_, this, level: i64| {
            // C++ getReqMana formula: pow(magic_multiplier, level-1) * 1600
            let lvl = level.max(1) as i32;
            let mana = this.0.mana_multiplier.powi(lvl - 1) * 1600.0;
            Ok(mana as i64)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_lua() -> mlua::Lua {
        let lua = mlua::Lua::new();
        crate::lua_bindings::install_bindings(
            &lua,
            crate::lua_bindings::GameStateHandle::default(),
        )
        .unwrap();
        lua
    }

    fn sample_vocation() -> Vocation {
        let mut v = Vocation::new(4);
        v.client_id = 4;
        v.name = "Knight".to_string();
        v.description = "a Knight".to_string();
        v.attack_speed = 2000;
        v.base_speed = 220;
        v.allow_pvp = true;
        v.skill_multipliers = [1.1, 1.1, 1.1, 1.1, 1.4, 1.1, 3.0];
        v.mana_multiplier = 3.0;
        v
    }

    #[test]
    fn get_id_returns_value() {
        let lua = fresh_lua();
        lua.globals()
            .set("v", LuaVocation::new(sample_vocation()))
            .unwrap();
        let n: i64 = lua.load("return v:getId()").eval().unwrap();
        assert_eq!(n, 4);
    }

    #[test]
    fn get_name_returns_string() {
        let lua = fresh_lua();
        lua.globals()
            .set("v", LuaVocation::new(sample_vocation()))
            .unwrap();
        let s: String = lua.load("return v:getName()").eval().unwrap();
        assert_eq!(s, "Knight");
    }

    #[test]
    fn allows_pvp_returns_bool() {
        let lua = fresh_lua();
        lua.globals()
            .set("v", LuaVocation::new(sample_vocation()))
            .unwrap();
        let b: bool = lua.load("return v:allowsPvp()").eval().unwrap();
        assert!(b);
    }

    #[test]
    fn required_skill_tries_uses_formula() {
        let lua = fresh_lua();
        lua.globals()
            .set("v", LuaVocation::new(sample_vocation()))
            .unwrap();
        // skill 0 (fist), level 10 → 10 * 1000 * 1.1 = 11000
        let n: i64 = lua
            .load("return v:getRequiredSkillTries(0, 10)")
            .eval()
            .unwrap();
        assert_eq!(n, 11000);
    }

    #[test]
    fn equality_metamethod_compares_by_id() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaVocation::new(sample_vocation()))
            .unwrap();
        lua.globals()
            .set("b", LuaVocation::new(sample_vocation()))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(v);
    }
}
