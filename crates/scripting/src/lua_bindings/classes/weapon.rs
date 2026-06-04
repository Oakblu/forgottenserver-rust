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

impl Default for LuaWeapon {
    fn default() -> Self {
        use forgottenserver_game::weapons::{ElementType, WeaponKind};
        Self(Weapon {
            item_id: 0,
            kind: WeaponKind::Melee,
            min_level: 0,
            min_mag_level: 0,
            attack: 0,
            defense: 0,
            shoot_range: 1,
            enabled: true,
            element_type: ElementType::None,
            element_damage: 0,
        })
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

    #[test]
    fn level_returns_min_level() {
        let lua = fresh_lua();
        let w = Weapon::new(2400, WeaponKind::Melee, 10, 30, 20);
        lua.globals().set("w", LuaWeapon::new(w)).unwrap();
        let v: i64 = lua.load("return w:level()").eval().unwrap();
        assert_eq!(v, 10);
    }

    #[test]
    fn magic_level_returns_min_mag_level() {
        let lua = fresh_lua();
        let mut w = LuaWeapon::default();
        w.0.min_mag_level = 5;
        lua.globals().set("w", w).unwrap();
        let v: i64 = lua.load("return w:magicLevel()").eval().unwrap();
        assert_eq!(v, 5);
    }

    #[test]
    fn attack_returns_attack() {
        let lua = fresh_lua();
        let w = Weapon::new(2400, WeaponKind::Melee, 10, 30, 20);
        lua.globals().set("w", LuaWeapon::new(w)).unwrap();
        let v: i64 = lua.load("return w:attack()").eval().unwrap();
        assert_eq!(v, 30);
    }

    #[test]
    fn defense_returns_defense() {
        let lua = fresh_lua();
        let w = Weapon::new(2400, WeaponKind::Melee, 10, 30, 20);
        lua.globals().set("w", LuaWeapon::new(w)).unwrap();
        let v: i64 = lua.load("return w:defense()").eval().unwrap();
        assert_eq!(v, 20);
    }

    #[test]
    fn range_returns_shoot_range() {
        let lua = fresh_lua();
        let mut w = LuaWeapon::default();
        w.0.shoot_range = 7;
        lua.globals().set("w", w).unwrap();
        let v: i64 = lua.load("return w:range()").eval().unwrap();
        assert_eq!(v, 7);
    }

    #[test]
    fn element_returns_element_type() {
        let lua = fresh_lua();
        let w = LuaWeapon::default(); // ElementType::None = 0
        lua.globals().set("w", w).unwrap();
        let v: i64 = lua.load("return w:element()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn stub_setters_do_not_error() {
        let lua = fresh_lua();
        lua.globals().set("w", LuaWeapon::default()).unwrap();
        let stubs = [
            "w:action(1)",
            "w:ammoType(1)",
            "w:breakChance(10)",
            "w:mana(50)",
            "w:manaPercent(5)",
            "w:soul(1)",
            "w:vocation('knight')",
            "w:premium(true)",
            "w:wieldUnproperly(false)",
            "w:register()",
            "w:onUseWeapon(function() end)",
            "w:shootType(1)",
            "w:charges(3)",
            "w:duration(1000)",
            "w:transformEquipTo(100)",
            "w:transformDeEquipTo(101)",
            "w:slotType(1)",
            "w:decayTo(0)",
            "w:damage(10, 20)",
            "w:extraElement(1, 5)",
            "w:health(10)",
            "w:healthPercent(5)",
            "w:hitChance(50)",
            "w:maxHitChance(100)",
        ];
        for stmt in &stubs {
            let result = lua.load(*stmt).exec();
            assert!(result.is_ok(), "stub '{}' should not error: {:?}", stmt, result);
        }
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaWeapon> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
