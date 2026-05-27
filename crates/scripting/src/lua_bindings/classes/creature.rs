//! `Creature:*` Lua binding for `entity::creature::Creature` (the base class for
//! every monster / player / NPC instance).
//!
//! Real getters/setters where the Rust `Creature` struct already exposes the
//! field; stubs for anything that requires the live world (movement, vision,
//! conditions, etc.).

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_entity::creature::Creature;
use mlua::{UserData, UserDataMethods, Value};
use std::sync::{Arc, Mutex};

pub struct LuaCreature(pub Arc<Mutex<Creature>>);

impl LuaCreature {
    pub fn new(c: Creature) -> Self {
        Self(Arc::new(Mutex::new(c)))
    }
}

impl Clone for LuaCreature {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaCreature {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaCreature>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaCreature",
                message: Some("expected Creature userdata".into()),
            }),
        }
    }
}

impl UserData for LuaCreature {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaCreature| {
            Ok(Arc::ptr_eq(&this.0, &other.0))
        });
        // ── Real getters / setters ────────────────────────────────
        methods.add_method("getId", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_id() as i64)
        });
        methods.add_method("getName", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_name().to_string())
        });
        methods.add_method("getHealth", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_health() as i64)
        });
        methods.add_method("getMaxHealth", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_max_health() as i64)
        });
        methods.add_method_mut("setHealth", |_, this, hp: i64| {
            this.0.lock().unwrap().set_health(hp as i32);
            Ok(())
        });
        methods.add_method_mut("setMaxHealth", |_, this, hp: i64| {
            this.0.lock().unwrap().set_max_health(hp as i32);
            Ok(())
        });
        methods.add_method_mut("addHealth", |_, this, hp: i64| {
            let mut g = this.0.lock().unwrap();
            let cur = g.get_health();
            let mx = g.get_max_health();
            let new = (cur + hp as i32).clamp(0, mx);
            g.set_health(new);
            Ok(true)
        });
        methods.add_method("getSpeed", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_speed() as i64)
        });
        methods.add_method("getBaseSpeed", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_base_speed() as i64)
        });
        methods.add_method_mut("changeSpeed", |_, this, delta: i64| {
            let mut g = this.0.lock().unwrap();
            let base = g.get_base_speed() as i64 + delta;
            g.set_base_speed(base.max(0) as u32);
            Ok(())
        });
        methods.add_method("getDirection", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_direction() as i64)
        });
        methods.add_method_mut("setDirection", |_, this, d: i64| {
            use forgottenserver_entity::creature::Direction;
            let dir = match d {
                0 => Direction::North,
                1 => Direction::East,
                2 => Direction::South,
                3 => Direction::West,
                _ => Direction::South,
            };
            this.0.lock().unwrap().set_direction(dir);
            Ok(())
        });
        methods.add_method("getSkull", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_skull() as i64)
        });
        methods.add_method("getLight", |lua, this, ()| {
            let g = this.0.lock().unwrap();
            let t = lua.create_table()?;
            t.set("level", g.get_light_level() as i64)?;
            t.set("color", g.get_light_color() as i64)?;
            Ok(t)
        });
        methods.add_method_mut("setLight", |_, this, (level, color): (i64, i64)| {
            this.0
                .lock()
                .unwrap()
                .set_light(level.max(0) as u8, color.max(0) as u8);
            Ok(())
        });
        methods.add_method("isHealthHidden", |_, this, ()| {
            Ok(this.0.lock().unwrap().is_health_hidden())
        });
        methods.add_method_mut("setHiddenHealth", |_, this, on: bool| {
            this.0.lock().unwrap().set_hidden_health(on);
            Ok(())
        });
        methods.add_method("isMovementBlocked", |_, this, ()| {
            Ok(this.0.lock().unwrap().is_movement_blocked())
        });
        methods.add_method_mut("setMovementBlocked", |_, this, on: bool| {
            this.0.lock().unwrap().set_movement_blocked(on);
            Ok(())
        });
        methods.add_method_mut("setDropLoot", |_, this, on: bool| {
            this.0.lock().unwrap().set_loot_drop(on);
            Ok(())
        });
        methods.add_method_mut("setSkillLoss", |_, this, on: bool| {
            this.0.lock().unwrap().set_skill_loss(on);
            Ok(())
        });
        methods.add_method("isCreature", |_, _this, _args: Value| Ok(true));
        methods.add_method("isRemoved", |_, this, ()| {
            Ok(!this.0.lock().unwrap().is_alive())
        });
        methods.add_method("getTarget", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_attack_target().unwrap_or(0) as i64)
        });
        methods.add_method_mut("setTarget", |_, this, id: i64| {
            this.0.lock().unwrap().set_attack_target(id.max(0) as u32);
            Ok(true)
        });

        // ── Stubs (need game-state plumbing) ──────────────────────
        for n in &[
            "canSee",
            "canSeeCreature",
            "canSeeGhostMode",
            "canSeeInvisibility",
            "hasCondition",
            "isImmune",
            "isInGhostMode",
        ] {
            methods.add_method(n, |_, _this, _args: Value| Ok(false));
        }
        for n in &[
            "getCondition",
            "getDamageMap",
            "getEvents",
            "getFollowCreature",
            "getIcon",
            "getMaster",
            "getOutfit",
            "getParent",
            "getPathTo",
            "getPosition",
            "getSummons",
            "getTile",
            "getZone",
        ] {
            methods.add_method(n, |lua, _this, _args: Value| lua.create_table());
        }
        methods.add_method("getDescription", |_, _this, _args: Value| Ok(String::new()));
        methods.add_method("getStorageValue", |_, _this, _key: i64| Ok(-1i64));
        methods.add_method_mut("setStorageValue", |_, _this, _args: Value| Ok(()));
        methods.add_method("hasIcon", |_, _this, _id: i64| Ok(false));
        methods.add_method("hasParent", |_, _this, _args: Value| Ok(false));
        for n in &[
            "addCondition",
            "registerEvent",
            "removeCondition",
            "removeIcon",
            "say",
            "setFollowCreature",
            "setIcon",
            "setMaster",
            "setOutfit",
            "setSkull",
            "unregisterEvent",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(true));
        }
        methods.add_method_mut("move", |_, _this, _args: Value| Ok(false));
        methods.add_method_mut("remove", |_, _this, _args: Value| Ok(false));
        methods.add_method_mut("teleportTo", |_, _this, _args: Value| Ok(false));
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
    fn health_round_trip() {
        let lua = fresh_lua();
        let c = Creature::new(1u32, "Wolf".to_string());
        lua.globals().set("c", LuaCreature::new(c)).unwrap();
        let h: i64 = lua
            .load("c:setHealth(50); return c:getHealth()")
            .eval()
            .unwrap();
        assert_eq!(h, 50);
    }

    #[test]
    fn add_health_clamps_to_max() {
        let lua = fresh_lua();
        let mut c = Creature::new(1u32, "Wolf".to_string());
        c.set_max_health(100);
        c.set_health(20);
        lua.globals().set("c", LuaCreature::new(c)).unwrap();
        let h: i64 = lua
            .load("c:addHealth(1000); return c:getHealth()")
            .eval()
            .unwrap();
        assert_eq!(h, 100);
    }
}
