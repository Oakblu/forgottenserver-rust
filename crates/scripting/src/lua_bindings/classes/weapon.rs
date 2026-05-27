//! `Weapon:*` Lua binding for `game::weapons::Weapon`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_game::weapons::Weapon;
use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone)]
pub struct LuaWeapon(pub Weapon);

impl LuaWeapon {
    pub fn new(w: Weapon) -> Self {
        Self(w)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaWeapon {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaWeapon>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaWeapon",
                message: Some("expected Weapon userdata".into()),
            }),
        }
    }
}

impl UserData for LuaWeapon {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // ── Real getters ──────────────────────────────────────────
        methods.add_method("id", |_, this, ()| Ok(this.0.item_id as i64));
        methods.add_method("level", |_, this, ()| Ok(this.0.min_level as i64));
        methods.add_method("magicLevel", |_, this, ()| Ok(this.0.min_mag_level as i64));
        methods.add_method("attack", |_, this, ()| Ok(this.0.attack as i64));
        methods.add_method("defense", |_, this, ()| Ok(this.0.defense as i64));
        methods.add_method("range", |_, this, ()| Ok(this.0.shoot_range as i64));
        methods.add_method("element", |_, this, ()| Ok(this.0.element_type as i64));

        // ── Stub setters / config ────────────────────────────────
        for n in &[
            "action",
            "ammoType",
            "breakChance",
            "mana",
            "manaPercent",
            "soul",
            "vocation",
            "premium",
            "wieldUnproperly",
            "register",
            "onUseWeapon",
            "shootType",
            "charges",
            "duration",
            "transformEquipTo",
            "transformDeEquipTo",
            "slotType",
            "decayTo",
            "damage",
            "extraElement",
            "health",
            "healthPercent",
            "hitChance",
            "maxHitChance",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_game::weapons::WeaponKind;

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
    fn id_returns_item_id() {
        let lua = fresh_lua();
        let w = Weapon::new(2400, WeaponKind::Melee, 10, 30, 20);
        lua.globals().set("w", LuaWeapon::new(w)).unwrap();
        let id: i64 = lua.load("return w:id()").eval().unwrap();
        assert_eq!(id, 2400);
    }
}
