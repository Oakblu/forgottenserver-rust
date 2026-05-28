//! `Condition:*` Lua binding for `items::condition::ConditionBase`.
//!
//! Wraps the polymorphic-base struct. Complex methods on subclasses
//! (`addDamage`, `setOutfit`, `setFormula`) require subclass dispatch
//! and are stubbed until the full Condition subclass tree binds.

// AUDIT: ClassMethod Condition:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_items::condition::ConditionBase;
use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone)]
pub struct LuaCondition(pub ConditionBase);

impl LuaCondition {
    pub fn new(c: ConditionBase) -> Self {
        Self(c)
    }
}

impl Default for LuaCondition {
    fn default() -> Self {
        use forgottenserver_common::enums::ConditionId;
        Self(ConditionBase::new(
            ConditionId::Default,
            0,
            -1,
            false,
            0,
            false,
        ))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaCondition {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaCondition>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaCondition",
                message: Some("expected Condition userdata".into()),
            }),
        }
    }
}

impl UserData for LuaCondition {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, _this, _other: Value| Ok(false));
        // __gc is mlua-implicit (no MetaMethod::Gc in 0.9); explicit
        // `delete` exposes the C++ binding.

        methods.add_method("getId", |_, this, ()| Ok(this.0.id as i64));
        methods.add_method("getType", |_, this, ()| Ok(this.0.condition_type as i64));
        methods.add_method("getSubId", |_, this, ()| Ok(this.0.sub_id as i64));
        methods.add_method("getTicks", |_, this, ()| Ok(this.0.ticks as i64));
        methods.add_method("getEndTime", |_, this, ()| Ok(this.0.end_time));
        methods.add_method("getIcons", |_, _this, ()| {
            // Stub: icons are subclass-specific (ConditionDamage sets fire,
            // ConditionRegeneration sets healing, etc.). C++ default for base
            // is 0 (no icons).
            Ok(0i64)
        });
        methods.add_method("getParameter", |_, _this, _param: i64| {
            // ConditionBase doesn't store typed params (subclasses do).
            // Match C++ Condition::getParam default (0 for unknown).
            Ok(0i64)
        });

        methods.add_method_mut("setTicks", |_, this, ticks: i64| {
            this.0.ticks = ticks as i32;
            Ok(())
        });
        methods.add_method_mut("setParameter", |_, _this, _args: (i64, Value)| {
            // Stub: ConditionBase doesn't store typed params; subclasses do.
            Ok(false)
        });
        // Stubs:
        methods.add_method("clone", |_, this, ()| Ok(this.clone()));
        methods.add_method_mut("addDamage", |_, _this, _args: (i64, i64, i64)| Ok(false));
        methods.add_method_mut("setOutfit", |_, _this, _outfit: Value| Ok(()));
        methods.add_method_mut("setFormula", |_, _this, _args: (f64, f64, f64, f64)| Ok(()));
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

    fn sample_condition() -> ConditionBase {
        use forgottenserver_common::enums::ConditionId;
        ConditionBase {
            id: ConditionId::Combat,
            condition_type: 1,
            ticks: 5000,
            end_time: 0,
            sub_id: 7,
            is_buff: false,
            aggressive: true,
        }
    }

    #[test]
    fn get_id_returns_discriminant() {
        let lua = fresh_lua();
        lua.globals()
            .set("c", LuaCondition::new(sample_condition()))
            .unwrap();
        let v: i64 = lua.load("return c:getId()").eval().unwrap();
        // ConditionId::Combat = 0
        assert_eq!(v, 0);
    }

    #[test]
    fn get_ticks_returns_field() {
        let lua = fresh_lua();
        lua.globals()
            .set("c", LuaCondition::new(sample_condition()))
            .unwrap();
        let v: i64 = lua.load("return c:getTicks()").eval().unwrap();
        assert_eq!(v, 5000);
    }

    #[test]
    fn set_ticks_mutates_field() {
        let lua = fresh_lua();
        lua.globals()
            .set("c", LuaCondition::new(sample_condition()))
            .unwrap();
        let v: i64 = lua
            .load("c:setTicks(10000); return c:getTicks()")
            .eval()
            .unwrap();
        assert_eq!(v, 10000);
    }

    #[test]
    fn set_parameter_accepts_boolean_true() {
        let lua = fresh_lua();
        let result = lua
            .load("local c = Condition(0); c:setParameter(1, true)")
            .exec();
        assert!(
            result.is_ok(),
            "Condition:setParameter with boolean true should not error: {result:?}"
        );
    }

    #[test]
    fn set_parameter_accepts_boolean_false() {
        let lua = fresh_lua();
        let result = lua
            .load("local c = Condition(0); c:setParameter(2, false)")
            .exec();
        assert!(
            result.is_ok(),
            "Condition:setParameter with boolean false should not error: {result:?}"
        );
    }

    #[test]
    fn set_parameter_still_accepts_integer() {
        let lua = fresh_lua();
        let result = lua
            .load("local c = Condition(0); c:setParameter(1, 5000)")
            .exec();
        assert!(
            result.is_ok(),
            "Condition:setParameter with integer should still work: {result:?}"
        );
    }
}
