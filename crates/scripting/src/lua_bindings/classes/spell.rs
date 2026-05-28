//! `Spell:*` Lua binding for `game::spells::Spell`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_game::spells::Spell;
use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone)]
pub struct LuaSpell(pub Spell);

impl LuaSpell {
    pub fn new(s: Spell) -> Self {
        Self(s)
    }
}

impl Default for LuaSpell {
    fn default() -> Self {
        use forgottenserver_game::spells::SpellGroup;
        Self(Spell {
            name: String::new(),
            words: String::new(),
            spell_id: 0,
            mana_cost: 0,
            mana_percent: 0,
            soul_cost: 0,
            min_level: 0,
            magic_level: 0,
            required_vocations: Vec::new(),
            cooldown: 1000,
            group_cooldown: 1000,
            secondary_group_cooldown: 0,
            group: SpellGroup::None,
            secondary_group: SpellGroup::None,
            premium: false,
            enabled: true,
            learnable: false,
            need_target: false,
            need_weapon: false,
            self_target: false,
            blocking_solid: false,
            blocking_creature: false,
            aggressive: true,
            pz_lock: false,
            range: -1,
        })
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaSpell {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaSpell>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaSpell",
                message: Some("expected Spell userdata".into()),
            }),
        }
    }
}

impl UserData for LuaSpell {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaSpell| {
            Ok(this.0.spell_id == other.0.spell_id && this.0.name == other.0.name)
        });
        // ── Real getters backed by Spell struct fields ───────────
        methods.add_method("name", |_, this, ()| Ok(this.0.name.clone()));
        methods.add_method("words", |_, this, ()| Ok(this.0.words.clone()));
        methods.add_method("id", |_, this, ()| Ok(this.0.spell_id as i64));
        methods.add_method("group", |_, this, ()| Ok(this.0.group as i64));
        methods.add_method("isPremium", |_, this, ()| Ok(this.0.premium));
        methods.add_method("isEnabled", |_, this, ()| Ok(this.0.enabled));
        methods.add_method("isSelfTarget", |_, this, ()| Ok(!this.0.need_target));
        methods.add_method("cooldown", |_, this, ()| Ok(this.0.cooldown as i64));
        methods.add_method("groupCooldown", |_, this, ()| {
            Ok(this.0.group_cooldown as i64)
        });
        methods.add_method("manaPercent", |_, this, ()| Ok(this.0.mana_percent as i64));
        // ── Stub setters (loaders mutate Spell directly, not via script) ─
        for n in &[
            "register",
            "vocation",
            "blockWalls",
            "needTarget",
            "needLearn",
            "isAggressive",
            "runeId",
            "runeLevel",
            "runeMagicLevel",
            "charges",
            "hasParams",
            "hasPlayerNameParam",
            "allowFarUse",
            "checkFloor",
            "isBlocking",
            "isBlockingWalls",
            "isPzLock",
            "level",
            "magicLevel",
            "mana",
            "needCasterTargetOrDirection",
            "needDirection",
            "needWeapon",
            "range",
            "soul",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(()));
        }
        // ── Callback setters (recorded but not dispatched) ───────
        methods.add_method_mut("onCastSpell", |_, _this, _cb: Value| Ok(()));
        // ── Allow arbitrary field assignment (e.g. spell.onCastSpell = ...) ─
        methods.add_meta_method_mut("__newindex", |_, _this, (_k, _v): (Value, Value)| Ok(()));
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
    fn name_returns_field() {
        let lua = fresh_lua();
        let s = Spell::new("Light Healing", 20, 8, vec![]);
        lua.globals().set("s", LuaSpell::new(s)).unwrap();
        let n: String = lua.load("return s:name()").eval().unwrap();
        assert_eq!(n, "Light Healing");
    }

    #[test]
    fn spell_field_assignment_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load("local s = Spell(0); s.onCastSpell = function() end")
            .exec();
        assert!(
            result.is_ok(),
            "field assignment on Spell should not error: {result:?}"
        );
    }

    #[test]
    fn spell_function_syntax_does_not_error() {
        let lua = fresh_lua();
        let result = lua
            .load("local s = Spell(0); function s.onCastSpell(creature, var) end")
            .exec();
        assert!(
            result.is_ok(),
            "function-declaration syntax on Spell should not error: {result:?}"
        );
    }
}
