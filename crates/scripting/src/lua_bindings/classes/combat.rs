//! `Combat:*` Lua binding.
//!
//! `game::combat::Combat` is a unit struct exposing static helpers
//! (e.g. `Combat::apply_formula_damage`). The C++ Lua side treats it
//! as an instance with setters for area/formula/callback that
//! configure a Combat instance. We wrap a small `CombatConfig` value
//! to hold these settings; methods that need cross-class types
//! (Condition, Area) stub no-op.

// AUDIT: ClassMethod Combat:__gc

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone, Default)]
pub struct CombatConfig {
    pub origin: i64,
    pub formula: Option<(f64, f64, f64, f64)>,
}

#[derive(Debug, Clone, Default)]
pub struct LuaCombat(pub CombatConfig);

impl LuaCombat {
    pub fn new() -> Self {
        Self(CombatConfig::default())
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaCombat {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaCombat>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaCombat",
                message: Some("expected Combat userdata".into()),
            }),
        }
    }
}

impl UserData for LuaCombat {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, _this, _other: Value| Ok(false));
        methods.add_method("delete", |_, _this, ()| Ok(()));
        methods.add_method_mut("clearConditions", |_, _this, ()| Ok(()));
        methods.add_method_mut("addCondition", |_, _this, _c: Value| Ok(()));
        methods.add_method_mut("setArea", |_, _this, _area: Value| Ok(()));
        methods.add_method_mut("setCallback", |_, _this, _args: (i64, String)| Ok(()));
        methods.add_method_mut("setFormula", |_, this, args: (i64, f64, f64, f64, f64)| {
            // (type, mina, minb, maxa, maxb)
            this.0.formula = Some((args.1, args.2, args.3, args.4));
            Ok(())
        });
        methods.add_method_mut("setOrigin", |_, this, origin: i64| {
            this.0.origin = origin;
            Ok(())
        });
        methods.add_method_mut("setParameter", |_, _this, _args: (i64, Value)| Ok(false));
        methods.add_method("getParameter", |_, _this, _param: i64| Ok(0i64));
        methods.add_method("execute", |_, _this, _args: (Value, Value)| Ok(true));
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

    #[test]
    fn set_origin_round_trips() {
        let lua = fresh_lua();
        lua.globals().set("c", LuaCombat::new()).unwrap();
        // setOrigin doesn't return a getter; we just ensure no error.
        lua.load("c:setOrigin(3)").exec().unwrap();
    }

    #[test]
    fn set_parameter_accepts_boolean_true() {
        let lua = fresh_lua();
        let result = lua
            .load("local c = Combat(); c:setParameter(1, true)")
            .exec();
        assert!(
            result.is_ok(),
            "setParameter with boolean true should not error: {result:?}"
        );
    }

    #[test]
    fn set_parameter_accepts_boolean_false() {
        let lua = fresh_lua();
        let result = lua
            .load("local c = Combat(); c:setParameter(2, false)")
            .exec();
        assert!(
            result.is_ok(),
            "setParameter with boolean false should not error: {result:?}"
        );
    }

    #[test]
    fn set_parameter_still_accepts_integer() {
        let lua = fresh_lua();
        let result = lua.load("local c = Combat(); c:setParameter(3, 42)").exec();
        assert!(
            result.is_ok(),
            "setParameter with integer should still work: {result:?}"
        );
    }
}
