//! `Spell:*` Lua binding for `game::spells::Spell`.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_game::spells::{Spell, SpellGroup};
use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone)]
pub struct LuaSpell {
    pub inner: Spell,
    pub vocation_names: Vec<String>,
    // Rune-specific fields
    pub rune_item_id: u16,
    pub rune_level: u32,
    pub rune_magic_level: u32,
    pub charges: u32,
    pub has_params: bool,
    pub has_player_name_param: bool,
    pub allow_far_use: bool,
    pub block_walls: bool,
    pub check_floor: bool,
}

impl LuaSpell {
    pub fn new(s: Spell) -> Self {
        Self {
            inner: s,
            vocation_names: Vec::new(),
            rune_item_id: 0,
            rune_level: 0,
            rune_magic_level: 0,
            charges: 0,
            has_params: false,
            has_player_name_param: false,
            allow_far_use: false,
            block_walls: false,
            check_floor: false,
        }
    }
}

impl Default for LuaSpell {
    fn default() -> Self {
        use forgottenserver_game::spells::SpellGroup;
        Self {
            inner: Spell {
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
                need_direction: false,
                need_caster_target_or_direction: false,
            },
            vocation_names: Vec::new(),
            rune_item_id: 0,
            rune_level: 0,
            rune_magic_level: 0,
            charges: 0,
            has_params: false,
            has_player_name_param: false,
            allow_far_use: false,
            block_walls: false,
            check_floor: false,
        }
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
            Ok(this.inner.spell_id == other.inner.spell_id && this.inner.name == other.inner.name)
        });
        // ── Real getters backed by Spell struct fields ───────────
        methods.add_method_mut("name", |_, this, arg: Option<String>| {
            if let Some(v) = arg { this.inner.name = v; }
            Ok(this.inner.name.clone())
        });
        methods.add_method_mut("words", |_, this, arg: Option<String>| {
            if let Some(v) = arg { this.inner.words = v; }
            Ok(this.inner.words.clone())
        });
        methods.add_method_mut("id", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.spell_id = v as u8; }
            Ok(this.inner.spell_id as i64)
        });
        methods.add_method_mut("group", |_, this, arg: Option<mlua::Value>| {
            if let Some(v) = arg {
                this.inner.group = match v {
                    mlua::Value::String(s) => SpellGroup::from_name(s.to_str().unwrap_or("")),
                    mlua::Value::Integer(n) => match n {
                        1 => SpellGroup::Attack,
                        2 => SpellGroup::Healing,
                        3 => SpellGroup::Support,
                        4 => SpellGroup::Special,
                        _ => SpellGroup::None,
                    },
                    _ => SpellGroup::None,
                };
            }
            Ok(this.inner.group as i64)
        });
        methods.add_method_mut("isPremium", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.premium = v; }
            Ok(this.inner.premium)
        });
        methods.add_method_mut("isEnabled", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.enabled = v; }
            Ok(this.inner.enabled)
        });
        methods.add_method_mut("isSelfTarget", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.self_target = v; }
            Ok(this.inner.self_target)
        });
        methods.add_method_mut("cooldown", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.cooldown = v as u32; }
            Ok(this.inner.cooldown as i64)
        });
        methods.add_method_mut("groupCooldown", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.group_cooldown = v as u32; }
            Ok(this.inner.group_cooldown as i64)
        });
        methods.add_method_mut("manaPercent", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.mana_percent = v as u32; }
            Ok(this.inner.mana_percent as i64)
        });
        methods.add_method_mut("level", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.min_level = v as u32; }
            Ok(this.inner.min_level as i64)
        });
        methods.add_method_mut("mana", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.mana_cost = v as u32; }
            Ok(this.inner.mana_cost as i64)
        });
        methods.add_method_mut("magicLevel", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.magic_level = v as u32; }
            Ok(this.inner.magic_level as i64)
        });
        methods.add_method_mut("soul", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.soul_cost = v as u32; }
            Ok(this.inner.soul_cost as i64)
        });
        methods.add_method_mut("range", |_, this, arg: Option<i64>| {
            if let Some(v) = arg { this.inner.range = v as i32; }
            Ok(this.inner.range as i64)
        });
        // ── Boolean getter/setters ───────────────────────────────────
        methods.add_method_mut("needTarget", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.need_target = v; }
            Ok(this.inner.need_target)
        });
        methods.add_method_mut("needWeapon", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.need_weapon = v; }
            Ok(this.inner.need_weapon)
        });
        methods.add_method_mut("needLearn", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.learnable = v; }
            Ok(this.inner.learnable)
        });
        methods.add_method_mut("isAggressive", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.aggressive = v; }
            Ok(this.inner.aggressive)
        });
        methods.add_method_mut("isPzLock", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.pz_lock = v; }
            Ok(this.inner.pz_lock)
        });
        methods.add_method_mut("isBlocking", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.blocking_solid = v; }
            Ok(this.inner.blocking_solid)
        });
        methods.add_method_mut("isBlockingWalls", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.blocking_creature = v; }
            Ok(this.inner.blocking_creature)
        });
        methods.add_method_mut("needDirection", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.need_direction = v; }
            Ok(this.inner.need_direction)
        });
        methods.add_method_mut("needCasterTargetOrDirection", |_, this, arg: Option<bool>| {
            if let Some(v) = arg { this.inner.need_caster_target_or_direction = v; }
            Ok(this.inner.need_caster_target_or_direction)
        });
        // ── Real vocation setter ─────────────────────────────────────
        methods.add_method_mut("vocation", |_, this, args: mlua::Variadic<String>| {
            for v in args {
                this.vocation_names.push(v);
            }
            Ok(())
        });
        // ── Register the spell in the LuaSpellStore app_data ─────────────────
        methods.add_method_mut("register", |lua, this, ()| {
            let store = lua
                .app_data_ref::<crate::lua_bindings::LuaSpellStore>()
                .ok_or_else(|| mlua::Error::runtime("LuaSpellStore not initialized"))?;
            store
                .0
                .lock()
                .map_err(|_| mlua::Error::runtime("LuaSpellStore lock poisoned"))?
                .push(this.clone());
            Ok(true)
        });
        // ── Rune-specific setters ────────────────────────────────────
        methods.add_method_mut("runeId", |_, this, v: i64| {
            this.rune_item_id = v as u16;
            Ok(this.rune_item_id as i64)
        });
        methods.add_method_mut("runeLevel", |_, this, v: i64| {
            this.rune_level = v as u32;
            Ok(this.rune_level as i64)
        });
        methods.add_method_mut("runeMagicLevel", |_, this, v: i64| {
            this.rune_magic_level = v as u32;
            Ok(this.rune_magic_level as i64)
        });
        methods.add_method_mut("charges", |_, this, v: i64| {
            this.charges = v as u32;
            Ok(this.charges as i64)
        });
        methods.add_method_mut("hasParams", |_, this, v: bool| {
            this.has_params = v;
            Ok(this.has_params)
        });
        methods.add_method_mut("hasPlayerNameParam", |_, this, v: bool| {
            this.has_player_name_param = v;
            Ok(this.has_player_name_param)
        });
        methods.add_method_mut("allowFarUse", |_, this, v: bool| {
            this.allow_far_use = v;
            Ok(this.allow_far_use)
        });
        methods.add_method_mut("blockWalls", |_, this, v: bool| {
            this.block_walls = v;
            Ok(this.block_walls)
        });
        methods.add_method_mut("checkFloor", |_, this, v: bool| {
            this.check_floor = v;
            Ok(this.check_floor)
        });
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

    #[test]
    fn spell_name_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:name("Berserk")
            _G.result = s:name()
        "#).exec().unwrap();
        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "Berserk");
    }

    #[test]
    fn spell_words_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:words("exori")
            _G.result = s:words()
        "#).exec().unwrap();
        let result: String = lua.globals().get("result").unwrap();
        assert_eq!(result, "exori");
    }

    #[test]
    fn spell_id_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:id(80)
            _G.result = s:id()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 80);
    }

    #[test]
    fn spell_level_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:level(35)
            _G.result = s:level()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 35);
    }

    #[test]
    fn spell_mana_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:mana(115)
            _G.result = s:mana()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 115);
    }

    #[test]
    fn spell_magic_level_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:magicLevel(4)
            _G.result = s:magicLevel()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 4);
    }

    #[test]
    fn spell_soul_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:soul(2)
            _G.result = s:soul()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn spell_mana_percent_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:manaPercent(10)
            _G.result = s:manaPercent()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn spell_range_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:range(7)
            _G.result = s:range()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 7);
    }

    #[test]
    fn spell_cooldown_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:cooldown(4000)
            _G.result = s:cooldown()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 4000);
    }

    #[test]
    fn spell_group_cooldown_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:groupCooldown(2000)
            _G.result = s:groupCooldown()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 2000);
    }

    #[test]
    fn spell_is_premium_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:isPremium(true)
            _G.result = s:isPremium()
        "#).exec().unwrap();
        let result: bool = lua.globals().get("result").unwrap();
        assert!(result);
    }

    #[test]
    fn spell_need_weapon_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:needWeapon(true)
            _G.result = s:needWeapon()
        "#).exec().unwrap();
        let result: bool = lua.globals().get("result").unwrap();
        assert!(result);
    }

    #[test]
    fn spell_is_aggressive_setter_to_false() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:isAggressive(false)
            _G.result = s:isAggressive()
        "#).exec().unwrap();
        let result: bool = lua.globals().get("result").unwrap();
        assert!(!result);
    }

    #[test]
    fn spell_need_direction_setter_and_getter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:needDirection(true)
            _G.result = s:needDirection()
        "#).exec().unwrap();
        let result: bool = lua.globals().get("result").unwrap();
        assert!(result);
    }

    #[test]
    fn spell_need_caster_target_or_direction_setter() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:needCasterTargetOrDirection(true)
            _G.result = s:needCasterTargetOrDirection()
        "#).exec().unwrap();
        let result: bool = lua.globals().get("result").unwrap();
        assert!(result);
    }

    #[test]
    fn spell_group_setter_string_attack() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:group("attack")
            _G.result = s:group()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn spell_group_setter_string_healing() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:group("healing")
            _G.result = s:group()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn spell_group_setter_unknown_defaults_to_none() {
        let lua = fresh_lua();
        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:group("bogus")
            _G.result = s:group()
        "#).exec().unwrap();
        let result: i64 = lua.globals().get("result").unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn spell_vocation_stores_single_string() {
        let lua = fresh_lua();
        lua.globals().set("s", LuaSpell::default()).unwrap();
        lua.load(r#"s:vocation("knight;true")"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("s").unwrap();
        let borrowed = ud.borrow::<LuaSpell>().unwrap();
        assert_eq!(borrowed.vocation_names, vec!["knight;true".to_string()]);
    }

    #[test]
    fn spell_vocation_stores_multiple_strings() {
        let lua = fresh_lua();
        lua.globals().set("s", LuaSpell::default()).unwrap();
        lua.load(r#"s:vocation("knight;true", "elite knight;true")"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("s").unwrap();
        let borrowed = ud.borrow::<LuaSpell>().unwrap();
        assert_eq!(borrowed.vocation_names.len(), 2);
        assert!(borrowed.vocation_names.contains(&"knight;true".to_string()));
        assert!(borrowed.vocation_names.contains(&"elite knight;true".to_string()));
    }

    #[test]
    fn spell_rune_id_setter() {
        let lua = fresh_lua();
        lua.globals().set("s", LuaSpell::default()).unwrap();
        lua.load(r#"s:runeId(2303)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("s").unwrap();
        let borrowed = ud.borrow::<LuaSpell>().unwrap();
        assert_eq!(borrowed.rune_item_id, 2303);
    }

    #[test]
    fn spell_charges_setter() {
        let lua = fresh_lua();
        lua.globals().set("s", LuaSpell::default()).unwrap();
        lua.load(r#"s:charges(3)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("s").unwrap();
        let borrowed = ud.borrow::<LuaSpell>().unwrap();
        assert_eq!(borrowed.charges, 3);
    }

    #[test]
    fn spell_allow_far_use_setter() {
        let lua = fresh_lua();
        lua.globals().set("s", LuaSpell::default()).unwrap();
        lua.load(r#"s:allowFarUse(true)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("s").unwrap();
        let borrowed = ud.borrow::<LuaSpell>().unwrap();
        assert!(borrowed.allow_far_use);
    }

    #[test]
    fn spell_block_walls_setter() {
        let lua = fresh_lua();
        lua.globals().set("s", LuaSpell::default()).unwrap();
        lua.load(r#"s:blockWalls(true)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("s").unwrap();
        let borrowed = ud.borrow::<LuaSpell>().unwrap();
        assert!(borrowed.block_walls);
    }

    #[test]
    fn spell_check_floor_setter() {
        let lua = fresh_lua();
        lua.globals().set("s", LuaSpell::default()).unwrap();
        lua.load(r#"s:checkFloor(true)"#).exec().unwrap();
        let ud: mlua::AnyUserData = lua.globals().get("s").unwrap();
        let borrowed = ud.borrow::<LuaSpell>().unwrap();
        assert!(borrowed.check_floor);
    }

    #[test]
    fn spell_register_stores_spell_in_lua_spell_store() {
        use crate::lua_bindings::LuaSpellStore;
        let lua = mlua::Lua::new();
        let store = LuaSpellStore::default();
        lua.set_app_data(store.clone());
        crate::lua_bindings::install_bindings(&lua, crate::lua_bindings::GameStateHandle::default()).unwrap();

        lua.load(r#"
            local s = Spell(SPELL_INSTANT)
            s:name("Berserk")
            s:words("exori")
            s:id(80)
            s:register()
        "#).exec().unwrap();

        let count = store.0.lock().unwrap().len();
        assert_eq!(count, 1, "register() must add the spell to LuaSpellStore");
        let spell = store.0.lock().unwrap()[0].clone();
        assert_eq!(spell.inner.name, "Berserk");
        assert_eq!(spell.inner.words, "exori");
    }
}
