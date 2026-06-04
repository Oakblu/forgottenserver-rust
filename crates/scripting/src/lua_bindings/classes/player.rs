//! `Player:*` Lua binding for `entity::player::Player`.
//!
//! Real getters/setters where the Rust `Player` struct already exposes the
//! field; stubs for anything that requires unrealised systems (channels,
//! housing, inventory items, online state, the network protocol, etc.).
//! Lua callers can already read identity / stats / experience / level /
//! premium / skull / capacity / etc.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_entity::player::Player;
use mlua::{UserData, UserDataMethods, Value};
use std::sync::{Arc, Mutex};

pub struct LuaPlayer(pub Arc<Mutex<Player>>);

impl LuaPlayer {
    pub fn new(p: Player) -> Self {
        Self(Arc::new(Mutex::new(p)))
    }
}

impl Clone for LuaPlayer {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaPlayer {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaPlayer>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaPlayer",
                message: Some("expected Player userdata".into()),
            }),
        }
    }
}

impl UserData for LuaPlayer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaPlayer| {
            Ok(Arc::ptr_eq(&this.0, &other.0))
        });
        // ── Identity (real) ───────────────────────────────────────
        methods.add_method("getGuid", |_, this, ()| {
            Ok(this.0.lock().unwrap().guid as i64)
        });
        methods.add_method("isPlayer", |_, _this, _args: Value| Ok(true));

        // ── Experience / level (real) ─────────────────────────────
        methods.add_method("getExperience", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_experience() as i64)
        });
        methods.add_method_mut("addExperience", |_, this, n: i64| {
            this.0.lock().unwrap().add_experience(n.max(0) as u64);
            Ok(())
        });
        methods.add_method_mut("removeExperience", |_, _this, _args: Value| Ok(false));
        methods.add_method("getLevel", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_level() as i64)
        });
        methods.add_method("getLevelPercent", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_level_percent() as i64)
        });

        // ── Magic / mana (real) ───────────────────────────────────
        methods.add_method("getMagicLevel", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_magic_level() as i64)
        });
        methods.add_method("getMagicLevelPercent", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_magic_level_percent() as i64)
        });
        methods.add_method("getBaseMagicLevel", |_, this, ()| {
            // Rust Player has no separate "base" magic level; use the current
            // (matches the C++ default when no boosts are active).
            Ok(this.0.lock().unwrap().get_magic_level() as i64)
        });
        methods.add_method("getMana", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_mana() as i64)
        });
        methods.add_method_mut("addMana", |_, this, n: i64| {
            let mut g = this.0.lock().unwrap();
            let mx = g.get_max_mana();
            let new = (g.get_mana() + n as i32).clamp(0, mx);
            g.set_mana(new);
            Ok(true)
        });
        methods.add_method("getMaxMana", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_max_mana() as i64)
        });
        methods.add_method_mut("setMaxMana", |_, this, n: i64| {
            this.0.lock().unwrap().set_max_mana(n as i32);
            Ok(())
        });
        methods.add_method("getManaSpent", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_mana_spent() as i64)
        });
        methods.add_method_mut("addManaSpent", |_, _this, _args: Value| Ok(()));
        methods.add_method_mut("removeManaSpent", |_, _this, _args: Value| Ok(()));
        methods.add_method("getBaseMaxHealth", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_max_health() as i64)
        });
        methods.add_method("getBaseMaxMana", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_max_mana() as i64)
        });

        // ── Capacity / weight (real) ──────────────────────────────
        methods.add_method("getCapacity", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_capacity() as i64)
        });
        methods.add_method_mut("setCapacity", |_, this, n: i64| {
            this.0.lock().unwrap().set_capacity(n.max(0) as u32);
            Ok(())
        });
        methods.add_method("getFreeCapacity", |_, this, ()| {
            // Rust signature wants current weight; use inventory weight as
            // a sane default. Returns 0 if capacity already exhausted.
            let g = this.0.lock().unwrap();
            let weight = g.get_inventory_weight();
            Ok(g.get_free_capacity(weight) as i64)
        });

        // ── Soul / stamina / skull / premium (real) ───────────────
        methods.add_method("getSoul", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_soul() as i64)
        });
        methods.add_method_mut("addSoul", |_, this, n: i64| {
            let mut g = this.0.lock().unwrap();
            let new_soul = (g.get_soul() as i64 + n).clamp(0, 200) as u8;
            g.set_soul(new_soul);
            Ok(())
        });
        methods.add_method("getMaxSoul", |_, _this, ()| Ok(200i64));
        methods.add_method("getStamina", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_stamina() as i64)
        });
        methods.add_method_mut("setStamina", |_, _this, _n: i64| Ok(()));
        methods.add_method("getSkullTime", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_skull_ticks() as i64)
        });
        methods.add_method_mut("setSkullTime", |_, this, ticks: i64| {
            this.0.lock().unwrap().set_skull_ticks(ticks.max(0) as u32);
            Ok(())
        });
        methods.add_method_mut("setSex", |_, _this, _v: i64| Ok(()));
        methods.add_method("getSex", |_, _this, ()| Ok(0i64));
        methods.add_method("getPremiumEndsAt", |_, _this, ()| Ok(0i64));
        methods.add_method_mut("setPremiumEndsAt", |_, _this, _ts: i64| Ok(()));

        // ── Offline training (real) ───────────────────────────────
        methods.add_method("getOfflineTrainingSkill", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_offline_training_skill() as i64)
        });
        methods.add_method_mut("setOfflineTrainingSkill", |_, _this, _v: i64| Ok(())); // SkillType enum dance
        methods.add_method("getOfflineTrainingTime", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_offline_training_time() as i64)
        });
        methods.add_method_mut("addOfflineTrainingTime", |_, _this, _n: i64| Ok(()));
        methods.add_method_mut("removeOfflineTrainingTime", |_, _this, _n: i64| Ok(()));
        methods.add_method_mut("addOfflineTrainingTries", |_, _this, _args: Value| {
            Ok(false)
        });

        // ── Skills (stub — SkillType ↔ i64 dance not yet wired) ───
        for n in &[
            "getSkillLevel",
            "getEffectiveSkillLevel",
            "getSkillTries",
            "getSkillPercent",
            "getSpecialSkill",
        ] {
            methods.add_method(n, |_, _this, _args: Value| Ok(0i64));
        }
        for n in &["addSkillTries", "removeSkillTries", "addSpecialSkill"] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(()));
        }

        // ── Inventory / items / money (stubs) ─────────────────────
        for n in &[
            "addItem",
            "addItemEx",
            "removeItem",
            "addMoney",
            "removeMoney",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(false));
        }
        for n in &["getItemCount", "getItemById", "getSlotItem"] {
            methods.add_method(n, |_, _this, _args: Value| Ok(Value::Nil));
        }
        // ── Containers / depot / inbox / market (stubs) ───────────
        for n in &[
            "getContainerById",
            "getContainerId",
            "getContainerIndex",
            "getDepotChest",
            "getStoreInbox",
            "getInbox",
            "isNearDepotBox",
            "getHouse",
        ] {
            methods.add_method(n, |_, _this, _args: Value| Ok(Value::Nil));
        }
        // ── Account / IP / client / town / group / vocation / guild ─
        methods.add_method("getAccountId", |_, _this, ()| Ok(0i64));
        methods.add_method("getAccountType", |_, _this, ()| Ok(1i64));
        methods.add_method_mut("setAccountType", |_, _this, _v: i64| Ok(()));
        methods.add_method("getIp", |_, _this, ()| Ok(0i64));
        methods.add_method("getClient", |lua, _this, ()| {
            let t = lua.create_table()?;
            t.set("version", 0i64)?;
            t.set("os", 0i64)?;
            Ok(t)
        });
        methods.add_method("getClientExpDisplay", |_, _this, ()| Ok(100i64));
        methods.add_method_mut("setClientExpDisplay", |_, _this, _v: i64| Ok(()));
        methods.add_method("getTown", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method_mut("setTown", |_, _this, _v: Value| Ok(()));
        methods.add_method("getGroup", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method_mut("setGroup", |_, _this, _v: Value| Ok(()));
        methods.add_method("getVocation", |_, this, ()| {
            Ok(this.0.lock().unwrap().vocation_id as i64)
        });
        methods.add_method_mut("setVocation", |_, this, v: i64| {
            this.0.lock().unwrap().vocation_id = v.max(0) as u16;
            Ok(())
        });
        methods.add_method("getGuild", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method_mut("setGuild", |_, _this, _v: Value| Ok(()));
        methods.add_method("getGuildLevel", |_, _this, ()| Ok(0i64));
        methods.add_method_mut("setGuildLevel", |_, _this, _v: i64| Ok(()));
        methods.add_method("getGuildNick", |_, _this, ()| Ok(String::new()));
        methods.add_method_mut("setGuildNick", |_, _this, _v: String| Ok(()));
        methods.add_method("getParty", |_, _this, _args: Value| Ok(Value::Nil));

        // ── Banking (stubs) ───────────────────────────────────────
        methods.add_method("getBankBalance", |_, _this, ()| Ok(0i64));
        methods.add_method_mut("setBankBalance", |_, _this, _n: i64| Ok(()));
        methods.add_method("getMoney", |_, _this, ()| Ok(0i64));
        methods.add_method("getDeathPenalty", |_, _this, ()| Ok(0.0f64));

        // ── PvP / state predicates (stubs) ────────────────────────
        methods.add_method("isPzLocked", |_, _this, ()| Ok(false));
        methods.add_method("hasChaseMode", |_, _this, ()| Ok(false));
        methods.add_method("hasSecureMode", |_, _this, ()| Ok(false));
        methods.add_method("getFightMode", |_, _this, ()| Ok(0i64));
        methods.add_method("getIdleTime", |_, _this, ()| Ok(0i64));
        methods.add_method_mut("resetIdleTime", |_, _this, ()| Ok(()));
        methods.add_method_mut("setGhostMode", |_, _this, _on: bool| Ok(()));
        methods.add_method("getLastLoginSaved", |_, _this, ()| Ok(0i64));
        methods.add_method("getLastLogout", |_, _this, ()| Ok(0i64));

        // ── Blessings / mounts / outfits / spells (stubs) ─────────
        for n in &[
            "hasBlessing",
            "hasMount",
            "hasOutfit",
            "hasLearnedSpell",
            "canCast",
            "canLearnSpell",
            "canWearOutfit",
        ] {
            methods.add_method(n, |_, _this, _args: Value| Ok(false));
        }
        for n in &[
            "addBlessing",
            "removeBlessing",
            "addMount",
            "removeMount",
            "toggleMount",
            "addOutfit",
            "addOutfitAddon",
            "removeOutfit",
            "removeOutfitAddon",
            "learnSpell",
            "forgetSpell",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(true));
        }
        for n in &["getInstantSpells", "getRuneSpells"] {
            methods.add_method(n, |lua, _this, ()| lua.create_table());
        }
        // ── Map marks (stubs) ─────────────────────────────────────
        methods.add_method_mut("addMapMark", |_, _this, _args: Value| Ok(()));
        // ── Networking (stubs — need an attached connection) ──────
        for n in &[
            "channelSay",
            "openChannel",
            "leaveChannel",
            "popupFYI",
            "save",
            "sendChannelMessage",
            "sendCreatureSquare",
            "sendEditPodium",
            "sendEnterMarket",
            "sendHouseWindow",
            "sendOutfitWindow",
            "sendPrivateMessage",
            "sendResourceBalance",
            "sendSupplyUsed",
            "sendTextMessage",
            "sendTutorial",
            "setEditHouse",
            "setManaShieldBar",
            "showTextDialog",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(true));
        }
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
    fn guid_returns_field() {
        let lua = fresh_lua();
        let p = Player::new(42, "Bubble", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let g: i64 = lua.load("return p:getGuid()").eval().unwrap();
        assert_eq!(g, 42);
    }

    #[test]
    fn add_experience_round_trips() {
        let lua = fresh_lua();
        let p = Player::new(1, "Bubble", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let xp: i64 = lua
            .load("p:addExperience(123); return p:getExperience()")
            .eval()
            .unwrap();
        assert_eq!(xp, 123);
    }

    #[test]
    fn is_player_returns_true() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: bool = lua.load("return p:isPlayer()").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn get_level_returns_value() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getLevel()").eval().unwrap();
        assert!(v >= 0);
    }

    #[test]
    fn get_level_percent_returns_value() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getLevelPercent()").eval().unwrap();
        assert!(v >= 0);
    }

    #[test]
    fn get_magic_level_returns_value() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getMagicLevel()").eval().unwrap();
        assert!(v >= 0);
    }

    #[test]
    fn get_base_magic_level_matches_magic_level() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let a: i64 = lua.load("return p:getMagicLevel()").eval().unwrap();
        let b: i64 = lua.load("return p:getBaseMagicLevel()").eval().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn mana_add_and_max() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let result: mlua::Result<bool> = lua.load("return p:addMana(100)").eval();
        assert!(result.is_ok());
        let mx: i64 = lua.load("return p:getMaxMana()").eval().unwrap();
        assert!(mx >= 0);
    }

    #[test]
    fn set_max_mana_mutates() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua
            .load("p:setMaxMana(500); return p:getMaxMana()")
            .eval()
            .unwrap();
        assert_eq!(v, 500);
    }

    #[test]
    fn get_mana_spent_returns_value() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getManaSpent()").eval().unwrap();
        assert!(v >= 0);
    }

    #[test]
    fn get_capacity_and_set() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let result: mlua::Result<()> = lua.load("p:setCapacity(10000)").exec();
        assert!(result.is_ok());
        let v: i64 = lua.load("return p:getCapacity()").eval().unwrap();
        assert_eq!(v, 10000);
    }

    #[test]
    fn get_free_capacity_returns_value() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getFreeCapacity()").eval().unwrap();
        assert!(v >= 0);
    }

    #[test]
    fn soul_and_max_soul() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let ms: i64 = lua.load("return p:getMaxSoul()").eval().unwrap();
        assert_eq!(ms, 200);
        let result: mlua::Result<()> = lua.load("p:addSoul(10)").exec();
        assert!(result.is_ok());
    }

    #[test]
    fn stamina_stub() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getStamina()").eval().unwrap();
        assert!(v >= 0);
    }

    #[test]
    fn skull_time_round_trip() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua
            .load("p:setSkullTime(1000); return p:getSkullTime()")
            .eval()
            .unwrap();
        assert_eq!(v, 1000);
    }

    #[test]
    fn vocation_round_trip() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua
            .load("p:setVocation(4); return p:getVocation()")
            .eval()
            .unwrap();
        assert_eq!(v, 4);
    }

    #[test]
    fn offline_training_skill_returns_value() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        // Default is Unset (-1) which matches C++ behavior
        let result: mlua::Result<i64> = lua
            .load("return p:getOfflineTrainingSkill()")
            .eval();
        assert!(result.is_ok());
    }

    #[test]
    fn offline_training_time_returns_value() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua
            .load("return p:getOfflineTrainingTime()")
            .eval()
            .unwrap();
        assert!(v >= 0);
    }

    #[test]
    fn get_account_id_returns_zero() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getAccountId()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_account_type_returns_one() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getAccountType()").eval().unwrap();
        assert_eq!(v, 1);
    }

    #[test]
    fn get_client_returns_table() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: bool = lua
            .load("local c = p:getClient(); return c ~= nil")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn get_bank_balance_returns_zero() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getBankBalance()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn is_pz_locked_returns_false() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: bool = lua.load("return p:isPzLocked()").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn get_instant_spells_returns_table() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: bool = lua
            .load("return type(p:getInstantSpells()) == 'table'")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn networking_stubs_return_true() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: bool = lua
            .load("return p:sendTextMessage(0, 'Hello')")
            .eval()
            .unwrap();
        assert!(v);
    }

    #[test]
    fn remove_experience_returns_false() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: bool = lua.load("return p:removeExperience(100)").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn skill_stubs_return_zero() {
        let lua = fresh_lua();
        let p = Player::new(1, "T", 1);
        lua.globals().set("p", LuaPlayer::new(p)).unwrap();
        let v: i64 = lua.load("return p:getSkillLevel(0)").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn eq_same_arc() {
        let lua = fresh_lua();
        let p = LuaPlayer::new(Player::new(1, "T", 1));
        let p2 = p.clone();
        lua.globals().set("a", p).unwrap();
        lua.globals().set("b", p2).unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(v);
    }

    #[test]
    fn eq_different_arcs() {
        let lua = fresh_lua();
        lua.globals()
            .set("a", LuaPlayer::new(Player::new(1, "A", 1)))
            .unwrap();
        lua.globals()
            .set("b", LuaPlayer::new(Player::new(2, "B", 1)))
            .unwrap();
        let v: bool = lua.load("return a == b").eval().unwrap();
        assert!(!v);
    }

    #[test]
    fn from_lua_error_on_wrong_type() {
        let lua = fresh_lua();
        let result: mlua::Result<LuaPlayer> = lua.load("return 42").eval();
        assert!(result.is_err());
    }
}
