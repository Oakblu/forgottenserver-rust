//! `MonsterType:*` Lua binding for `entity::monsters::MonsterType`.
//!
//! Real bindings for fields the Rust struct has (name, health,
//! armor, defense, etc.); stubs for the rest until the monsters
//! XML loader fully migrates.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_entity::monsters::MonsterType;
use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone)]
pub struct LuaMonsterType(pub MonsterType);

impl LuaMonsterType {
    pub fn new(mt: MonsterType) -> Self {
        Self(mt)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaMonsterType {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaMonsterType>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaMonsterType",
                message: Some("expected MonsterType userdata".into()),
            }),
        }
    }
}

impl UserData for LuaMonsterType {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaMonsterType| {
            Ok(this.0.name == other.0.name)
        });

        // ── Real getters ──────────────────────────────────────────
        methods.add_method("name", |_, this, ()| Ok(this.0.name.clone()));
        methods.add_method("nameDescription", |_, this, ()| {
            Ok(this.0.name_description.clone())
        });
        methods.add_method("health", |_, this, ()| Ok(this.0.health as i64));
        methods.add_method("maxHealth", |_, this, ()| Ok(this.0.max_health as i64));
        methods.add_method("baseSpeed", |_, this, ()| Ok(this.0.base_speed as i64));
        methods.add_method("experience", |_, this, ()| Ok(this.0.experience as i64));
        methods.add_method("manaCost", |_, this, ()| Ok(this.0.mana_cost as i64));
        methods.add_method("armor", |_, this, ()| Ok(this.0.armor as i64));
        methods.add_method("defense", |_, this, ()| Ok(this.0.defense as i64));
        methods.add_method("targetDistance", |_, this, ()| {
            Ok(this.0.target_distance as i64)
        });
        methods.add_method("changeTargetSpeed", |_, this, ()| {
            Ok(this.0.change_target_speed as i64)
        });
        methods.add_method("changeTargetChance", |_, this, ()| {
            Ok(this.0.change_target_chance as i64)
        });
        methods.add_method("staticAttackChance", |_, this, ()| {
            Ok(this.0.static_attack_chance as i64)
        });
        methods.add_method("runHealth", |_, this, ()| Ok(this.0.run_away_health as i64));
        methods.add_method("maxSummons", |_, this, ()| Ok(this.0.max_summons as i64));
        methods.add_method("combatImmunities", |_, this, ()| {
            Ok(this.0.immunity_flags as i64)
        });

        // ── Stub setters (mutating monster type at runtime not yet wired) ─
        for_each_stub_unit(
            methods,
            &[
                "addAttack",
                "addDefense",
                "addElement",
                "addLoot",
                "addSummon",
                "addVoice",
                "registerEvent",
            ],
        );
        // ── Stub list-getters returning empty tables ──────────────
        methods.add_method("getAttackList", |lua, _this, ()| lua.create_table());
        methods.add_method("getDefenseList", |lua, _this, ()| lua.create_table());
        methods.add_method("getElementList", |lua, _this, ()| lua.create_table());
        methods.add_method("getLoot", |lua, _this, ()| lua.create_table());
        methods.add_method("getSummonList", |lua, _this, ()| lua.create_table());
        methods.add_method("getVoices", |lua, _this, ()| lua.create_table());
        methods.add_method("getCreatureEvents", |lua, _this, ()| lua.create_table());
        methods.add_method("outfit", |lua, _this, ()| lua.create_table());
        methods.add_method("light", |lua, _this, ()| lua.create_table());
        methods.add_method("bestiaryInfo", |lua, _this, ()| lua.create_table());

        // ── Stub event setters (Lua callbacks) ────────────────────
        for_each_stub_unit_arg(
            methods,
            &[
                "onAppear",
                "onDisappear",
                "onMove",
                "onSay",
                "onThink",
                "eventType",
            ],
        );

        // ── Stub other scalars ────────────────────────────────────
        methods.add_method("conditionImmunities", |_, _this, ()| Ok(0i64));
        methods.add_method("corpseId", |_, _this, ()| Ok(0i64));
        methods.add_method("race", |_, _this, ()| Ok(0i64));
        methods.add_method("skull", |_, _this, ()| Ok(0i64));
        methods.add_method("yellChance", |_, _this, ()| Ok(0i64));
        methods.add_method("yellSpeedTicks", |_, _this, ()| Ok(0i64));

        // ── Stub predicates ───────────────────────────────────────
        for_each_stub_bool(
            methods,
            &[
                "canPushCreatures",
                "canPushItems",
                "canWalkOnEnergy",
                "canWalkOnFire",
                "canWalkOnPoison",
                "isAttackable",
                "isBoss",
                "isChallengeable",
                "isConvinceable",
                "isHealthHidden",
                "isHostile",
                "isIgnoringSpawnBlock",
                "isIllusionable",
                "isPushable",
                "isSummonable",
            ],
        );
    }
}

fn for_each_stub_unit<'lua, M: UserDataMethods<'lua, LuaMonsterType>>(
    methods: &mut M,
    names: &[&'static str],
) {
    for name in names {
        methods.add_method_mut(name, |_, _this, _args: Value| Ok(()));
    }
}
fn for_each_stub_unit_arg<'lua, M: UserDataMethods<'lua, LuaMonsterType>>(
    methods: &mut M,
    names: &[&'static str],
) {
    for name in names {
        methods.add_method_mut(name, |_, _this, _args: Value| Ok(()));
    }
}
fn for_each_stub_bool<'lua, M: UserDataMethods<'lua, LuaMonsterType>>(
    methods: &mut M,
    names: &[&'static str],
) {
    for name in names {
        methods.add_method(name, |_, _this, ()| Ok(false));
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
    fn name_returns_struct_field() {
        let lua = fresh_lua();
        let mt = MonsterType {
            name: "Rat".into(),
            health: 20,
            ..MonsterType::default()
        };
        lua.globals().set("m", LuaMonsterType::new(mt)).unwrap();
        let s: String = lua.load("return m:name()").eval().unwrap();
        assert_eq!(s, "Rat");
        let v: i64 = lua.load("return m:health()").eval().unwrap();
        assert_eq!(v, 20);
    }
}
