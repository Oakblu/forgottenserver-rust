//! Lightweight player userdata for talkaction Lua script execution.
//!
//! `LuaTalkPlayer` is a minimal player proxy passed as the `player` argument
//! to `onSay(player, words, param)`. It captures:
//!   - Current position (readable via `getPosition()`)
//!   - Message buffer (appended to by `sendTextMessage` / `sendCancelMessage`)
//!   - Position update slot (written by `teleportTo`)
//!   - Identity fields (name, level) for scripts that query them
//!   - Admin flag (group.getAccess())
//!
//! Create one with [`execute_talkaction`].

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use std::path::Path;
use std::sync::{Arc, Mutex};

use forgottenserver_common::position::Position;
use mlua::{UserData, UserDataMethods, Value};

use crate::lua_bindings::position::LuaPosition;
use crate::lua_bindings::LuaEnvironment;

// ── Output ─────────────────────────────────────────────────────────────────

/// Result returned by [`execute_talkaction`].
#[derive(Debug)]
pub struct TalkActionOutput {
    /// Text messages queued by `player:sendTextMessage` / `sendCancelMessage`,
    /// in call order. Each entry is `(message_type_byte, text)`.
    pub messages: Vec<(u8, String)>,
    /// New position set by `player:teleportTo(pos)`, if the script called it.
    pub new_pos: Option<Position>,
    /// Magic effects queued by `position:sendMagicEffect(effect)`.
    pub magic_effects: Vec<(Position, u8)>,
}

// ── LuaTalkGroup ────────────────────────────────────────────────────────────

struct LuaTalkGroup {
    has_access: bool,
}

impl UserData for LuaTalkGroup {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("getAccess", |_, this, ()| Ok(this.has_access));
        methods.add_method("getAccountType", |_, this, ()| {
            Ok(if this.has_access { 6i64 } else { 0i64 })
        });
        for n in &["getId", "getName", "getMaxDepotItems", "getMaxVipEntries"] {
            methods.add_method(n, |_, _this, _args: Value| Ok(Value::Nil));
        }
        methods.add_method("hasFlag", |_, _this, _args: Value| Ok(false));
    }
}

// ── LuaTalkPlayer ───────────────────────────────────────────────────────────

struct LuaTalkPlayer {
    pos: Position,
    messages: Arc<Mutex<Vec<(u8, String)>>>,
    new_pos: Arc<Mutex<Option<Position>>>,
    name: String,
    level: u16,
    has_access: bool,
}

impl UserData for LuaTalkPlayer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // ── Position ──────────────────────────────────────────────────
        methods.add_method("getPosition", |_, this, ()| Ok(LuaPosition(this.pos)));

        methods.add_method("teleportTo", |_, this, pos: LuaPosition| {
            *this.new_pos.lock().unwrap() = Some(pos.into_inner());
            Ok(true)
        });

        // ── Messaging ─────────────────────────────────────────────────
        methods.add_method(
            "sendTextMessage",
            |_, this, (msg_type, text): (i64, String)| {
                this.messages.lock().unwrap().push((msg_type as u8, text));
                Ok(true)
            },
        );

        methods.add_method("sendCancelMessage", |_, this, text: String| {
            // MESSAGE_STATUS_SMALL = 21 (matches C++ const.h MessageClasses)
            const MESSAGE_STATUS_SMALL: u8 = 21;
            this.messages
                .lock()
                .unwrap()
                .push((MESSAGE_STATUS_SMALL, text));
            Ok(true)
        });

        // ── Group / access ────────────────────────────────────────────
        methods.add_method("getGroup", |lua, this, ()| {
            lua.create_userdata(LuaTalkGroup {
                has_access: this.has_access,
            })
        });

        methods.add_method("getAccountType", |_, this, ()| {
            Ok(if this.has_access { 6i64 } else { 0i64 })
        });

        // ── Identity ──────────────────────────────────────────────────
        methods.add_method("getName", |_, this, ()| Ok(this.name.clone()));
        methods.add_method("getLevel", |_, this, ()| Ok(this.level as i64));
        methods.add_method("getMagicLevel", |_, _this, ()| Ok(0i64));
        methods.add_method("getSpeed", |_, _this, ()| Ok(220i64));
        methods.add_method("getIp", |_, _this, ()| Ok(std::string::String::from("127.0.0.1")));

        // ── Predicates (stubs) ────────────────────────────────────────
        methods.add_method("isPlayer", |_, _this, ()| Ok(true));
        methods.add_method("isCreature", |_, _this, ()| Ok(true));
        methods.add_method("isPremium", |_, _this, ()| Ok(false));
        methods.add_method("hasFlag", |_, _this, _args: Value| Ok(false));
        methods.add_method("canSeeCreature", |_, _this, _args: Value| Ok(true));
        methods.add_method("isInGhostMode", |_, _this, ()| Ok(false));
        methods.add_method("getGuid", |_, _this, ()| Ok(1i64));
        methods.add_method("getId", |_, _this, ()| Ok(1i64));

        // ── Town stub ─────────────────────────────────────────────────
        // Returns a minimal town-like table with getTemplePosition() so
        // teleport_home.lua can call player:getTown():getTemplePosition().
        methods.add_method("getTown", |lua, this, ()| {
            let pos = this.pos;
            let town = lua.create_table()?;
            town.set("getTemplePosition", lua.create_function(move |_, _: ()| Ok(LuaPosition(pos)))?)?;
            town.set("getId", lua.create_function(|_, _: ()| Ok(1i64))?)?;
            town.set("getName", lua.create_function(|_, _: ()| Ok("default"))?)?;
            Ok(town)
        });

        // ── Game-world interaction stubs ──────────────────────────────
        methods.add_method("addSummon", |_, _this, _args: Value| Ok(()));
        methods.add_method("getVocation", |lua, _this, ()| lua.create_table());
        methods.add_method("getSkillLevel", |_, _this, _args: Value| Ok(0i64));
        methods.add_method("addSkillTries", |_, _this, _args: Value| Ok(()));
        methods.add_method("addSkill", |_, _this, _args: Value| Ok(()));
        methods.add_method("addItem", |_, _this, _args: Value| Ok(mlua::Value::Nil));
        methods.add_method("addMana", |_, _this, _args: Value| Ok(()));
        methods.add_method("addHealth", |_, _this, _args: Value| Ok(()));
        methods.add_method("getHealth", |_, _this, ()| Ok(100i64));
        methods.add_method("getMaxHealth", |_, _this, ()| Ok(100i64));
        methods.add_method("getMana", |_, _this, ()| Ok(100i64));
        methods.add_method("getMaxMana", |_, _this, ()| Ok(100i64));
        methods.add_method("getStamina", |_, _this, ()| Ok(2520i64));
        methods.add_method("getSoul", |_, _this, ()| Ok(100i64));
        methods.add_method("getCapacity", |_, _this, ()| Ok(40000i64));
        methods.add_method("getFreeCapacity", |_, _this, ()| Ok(40000i64));
        methods.add_method("getExperience", |_, _this, ()| Ok(0i64));
        methods.add_method("addExperience", |_, _this, _args: Value| Ok(()));
        methods.add_method("removeExperience", |_, _this, _args: Value| Ok(()));
        methods.add_method("getCondition", |_, _this, _args: Value| Ok(mlua::Value::Nil));
        methods.add_method("addCondition", |_, _this, _args: Value| Ok(()));
        methods.add_method("removeCondition", |_, _this, _args: Value| Ok(()));
        methods.add_method("hasCondition", |_, _this, _args: Value| Ok(false));
        methods.add_method("getKillers", |lua, _this, ()| lua.create_table());
        methods.add_method("getBlessings", |_, _this, ()| Ok(0i64));
        methods.add_method("hasBlessing", |_, _this, _args: Value| Ok(false));
        methods.add_method("addBlessing", |_, _this, _args: Value| Ok(()));
        methods.add_method("getDepotChest", |_, _this, _args: Value| Ok(mlua::Value::Nil));
        methods.add_method("getInbox", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getStoreInbox", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getTile", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getHouse", |_, _this, ()| Ok(mlua::Value::Nil));
        methods.add_method("getPremiumEndsAt", |_, _this, ()| Ok(0i64));
        methods.add_method("setPremiumEndsAt", |_, _this, _args: Value| Ok(()));
        methods.add_method("save", |_, _this, ()| Ok(true));
        methods.add_method("kick", |_, _this, ()| Ok(()));
        methods.add_method("remove", |_, _this, ()| Ok(()));
        methods.add_method("getClosestFreePosition", |_, this, _args: Value| Ok(LuaPosition(this.pos)));
        methods.add_method("feed", |_, _this, _args: Value| Ok(()));
        methods.add_method("say", |_, _this, _args: Value| Ok(()));
        methods.add_method("registerCreatureEvent", |_, _this, _args: Value| Ok(()));
        methods.add_method("unregisterCreatureEvent", |_, _this, _args: Value| Ok(()));
        methods.add_method("getSpecialSkill", |_, _this, _args: Value| Ok(0i64));
        methods.add_method("setSpecialSkill", |_, _this, _args: Value| Ok(()));
        methods.add_method("sendLootStats", |_, _this, _args: Value| Ok(()));
        methods.add_method("sendSupplyUsed", |_, _this, _args: Value| Ok(()));
        methods.add_method("setMaxCombatValue", |_, _this, _args: Value| Ok(()));
        methods.add_method("getPzLocked", |_, _this, ()| Ok(false));

        // ── Networking stubs ──────────────────────────────────────────
        for n in &[
            "sendPrivateMessage",
            "openChannel",
            "sendChannelMessage",
            "sendTutorial",
            "sendResourceBalance",
        ] {
            methods.add_method_mut(n, |_, _this, _args: Value| Ok(true));
        }
    }
}

// ── execute_talkaction ──────────────────────────────────────────────────────

/// Execute a talkaction Lua script file.
///
/// Uses the provided `env` Lua environment (with full bindings already
/// registered), registers a minimal player userdata, loads `script_path`,
/// and calls `onSay(player, words, param)`.
///
/// On success returns [`TalkActionOutput`] with collected messages and any
/// position update the script requested via `player:teleportTo(pos)`.
///
/// Returns `Err(String)` for IO / Lua errors — the caller should log and
/// treat as a no-op rather than terminating the connection.
#[allow(clippy::too_many_arguments)]
pub fn execute_talkaction(
    env: &mut LuaEnvironment,
    script_path: &Path,
    words: &str,
    param: &str,
    pos: Position,
    name: &str,
    level: u16,
    has_access: bool,
) -> Result<TalkActionOutput, String> {
    let messages: Arc<Mutex<Vec<(u8, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let new_pos: Arc<Mutex<Option<Position>>> = Arc::new(Mutex::new(None));
    let magic_effects_buf = Arc::new(Mutex::new(Vec::<(Position, u8)>::new()));

    let player = LuaTalkPlayer {
        pos,
        messages: Arc::clone(&messages),
        new_pos: Arc::clone(&new_pos),
        name: name.to_string(),
        level,
        has_access,
    };

    env.lua.set_app_data(crate::lua_bindings::MagicEffectsBuffer(Arc::clone(&magic_effects_buf)));

    // Load the script file — defines `onSay` globally.
    env.load_file(script_path)?;

    let on_say: mlua::Function = env
        .lua
        .globals()
        .get("onSay")
        .map_err(|e| format!("onSay missing in {}: {e}", script_path.display()))?;

    let player_ud = env
        .lua
        .create_userdata(player)
        .map_err(|e| format!("create_userdata failed: {e}"))?;

    let _result: Value = on_say
        .call((player_ud, words.to_owned(), param.to_owned()))
        .map_err(|e| format!("onSay error in {}: {e}", script_path.display()))?;

    let msgs = std::mem::take(&mut *messages.lock().unwrap());
    let pos_out = *new_pos.lock().unwrap();
    let effects = std::mem::take(&mut *magic_effects_buf.lock().unwrap());

    Ok(TalkActionOutput {
        messages: msgs,
        new_pos: pos_out,
        magic_effects: effects,
    })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_bindings::{GameStateHandle, LuaEnvironment};
    use std::io::Write as _;
    use tempfile::NamedTempFile;

    fn write_script(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::with_suffix(".lua").unwrap();
        write!(f, "{content}").unwrap();
        f
    }

    fn default_pos() -> Position {
        Position::new(1000, 1000, 7)
    }

    #[test]
    fn execute_talkaction_send_text_message() {
        let f = write_script(
            r#"function onSay(player, words, param)
                player:sendTextMessage(22, "hello from lua")
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out =
            execute_talkaction(&mut env, f.path(), "/test", "", default_pos(), "TestPlayer", 10, true)
                .unwrap();
        assert_eq!(out.messages.len(), 1);
        assert_eq!(out.messages[0].0, 22);
        assert_eq!(out.messages[0].1, "hello from lua");
        assert!(out.new_pos.is_none());
    }

    #[test]
    fn execute_talkaction_position_in_message() {
        let f = write_script(
            r#"function onSay(player, words, param)
                local pos = player:getPosition()
                player:sendTextMessage(22, "x=" .. pos.x .. ",y=" .. pos.y .. ",z=" .. pos.z)
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out = execute_talkaction(
            &mut env,
            f.path(),
            "/pos",
            "",
            Position::new(500, 600, 7),
            "Hero",
            50,
            true,
        )
        .unwrap();
        assert_eq!(out.messages.len(), 1);
        assert!(out.messages[0].1.contains("x=500"));
        assert!(out.messages[0].1.contains("y=600"));
        assert!(out.messages[0].1.contains("z=7"));
    }

    #[test]
    fn execute_talkaction_teleport_updates_position() {
        let f = write_script(
            r#"function onSay(player, words, param)
                local pos = player:getPosition()
                pos.z = pos.z - 1
                player:teleportTo(pos)
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out = execute_talkaction(
            &mut env,
            f.path(),
            "/up",
            "",
            Position::new(1000, 1000, 7),
            "Hero",
            1,
            true,
        )
        .unwrap();
        assert!(out.new_pos.is_some());
        let new = out.new_pos.unwrap();
        assert_eq!(new.z, 6);
        assert_eq!(new.x, 1000);
        assert_eq!(new.y, 1000);
    }

    #[test]
    fn execute_talkaction_group_access_true_allows_script() {
        let f = write_script(
            r#"function onSay(player, words, param)
                if not player:getGroup():getAccess() then
                    return true
                end
                player:sendTextMessage(22, "access granted")
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out =
            execute_talkaction(&mut env, f.path(), "/cmd", "", default_pos(), "Admin", 1, true).unwrap();
        assert_eq!(out.messages.len(), 1);
        assert_eq!(out.messages[0].1, "access granted");
    }

    #[test]
    fn execute_talkaction_group_access_false_blocks_script() {
        let f = write_script(
            r#"function onSay(player, words, param)
                if not player:getGroup():getAccess() then
                    return true
                end
                player:sendTextMessage(22, "access granted")
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out =
            execute_talkaction(&mut env, f.path(), "/cmd", "", default_pos(), "Player", 1, false).unwrap();
        assert!(out.messages.is_empty(), "non-admin should get no message");
    }

    #[test]
    fn execute_talkaction_send_cancel_message() {
        let f = write_script(
            r#"function onSay(player, words, param)
                player:sendCancelMessage("not allowed")
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out =
            execute_talkaction(&mut env, f.path(), "/x", "", default_pos(), "P", 1, false).unwrap();
        assert_eq!(out.messages.len(), 1);
        assert_eq!(out.messages[0].0, 21); // MESSAGE_STATUS_SMALL
        assert_eq!(out.messages[0].1, "not allowed");
    }

    #[test]
    fn execute_talkaction_multiple_messages_ordered() {
        let f = write_script(
            r#"function onSay(player, words, param)
                player:sendTextMessage(22, "first")
                player:sendTextMessage(22, "second")
                player:sendTextMessage(22, "third")
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out =
            execute_talkaction(&mut env, f.path(), "/multi", "", default_pos(), "P", 1, true).unwrap();
        assert_eq!(out.messages.len(), 3);
        assert_eq!(out.messages[0].1, "first");
        assert_eq!(out.messages[1].1, "second");
        assert_eq!(out.messages[2].1, "third");
    }

    #[test]
    fn execute_talkaction_missing_script_returns_err() {
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let result = execute_talkaction(
            &mut env,
            Path::new("/nonexistent/path/does_not_exist.lua"),
            "/x",
            "",
            default_pos(),
            "P",
            1,
            true,
        );
        assert!(result.is_err(), "missing script should return Err");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("[FATAL]") || msg.contains("No such file"),
            "error should mention missing file: {msg}"
        );
    }

    #[test]
    fn execute_talkaction_lua_error_returns_err() {
        let f = write_script(
            r#"function onSay(player, words, param)
                error("intentional lua error")
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let result = execute_talkaction(&mut env, f.path(), "/x", "", default_pos(), "P", 1, true);
        assert!(result.is_err(), "Lua error should propagate as Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("intentional lua error"), "error should contain script message: {msg}");
    }

    #[test]
    fn execute_talkaction_param_passed_to_script() {
        let f = write_script(
            r#"function onSay(player, words, param)
                player:sendTextMessage(22, "param=" .. param)
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out = execute_talkaction(&mut env, f.path(), "/test", "hello world", default_pos(), "P", 1, true)
            .unwrap();
        assert_eq!(out.messages[0].1, "param=hello world");
    }

    #[test]
    fn execute_talkaction_no_teleport_when_script_does_not_call_it() {
        let f = write_script(
            r#"function onSay(player, words, param)
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out = execute_talkaction(&mut env, f.path(), "/noop", "", default_pos(), "P", 1, true).unwrap();
        assert!(out.new_pos.is_none());
        assert!(out.messages.is_empty());
    }

    #[test]
    fn execute_talkaction_down_increases_z() {
        let f = write_script(
            r#"function onSay(player, words, param)
                if not player:getGroup():getAccess() then return true end
                local pos = player:getPosition()
                pos.z = pos.z + 1
                player:teleportTo(pos)
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out = execute_talkaction(
            &mut env,
            f.path(),
            "/down",
            "",
            Position::new(1000, 1000, 6),
            "Admin",
            1,
            true,
        )
        .unwrap();
        let new = out.new_pos.expect("teleportTo must have been called");
        assert_eq!(new.z, 7);
    }

    #[test]
    fn execute_real_place_summon_sends_cancel_message() {
        let script_path = std::path::Path::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/talkactions/scripts/place_summon.lua"),
        );
        if !script_path.exists() {
            eprintln!("Skipping: place_summon.lua not found at {}", script_path.display());
            return;
        }
        let pos = default_pos();
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out = execute_talkaction(&mut env, script_path, "/summon", "dog", pos, "Admin", 100, true)
            .expect("execute_talkaction should not error");
        assert_eq!(out.messages.len(), 1, "Expected 1 cancel message, got {:?}", out.messages);
        assert_eq!(out.messages[0].1, "There is not enough room.");
    }

    #[test]
    fn execute_talkaction_accepts_external_env() {
        let f = write_script(
            r#"function onSay(player, words, param)
                player:sendTextMessage(22, "env ok")
                return false
            end"#,
        );
        let mut env = LuaEnvironment::new(GameStateHandle::default()).expect("lua init");
        let out = execute_talkaction(
            &mut env,
            f.path(),
            "/test",
            "",
            default_pos(),
            "TestPlayer",
            10,
            true,
        )
        .expect("execute_talkaction must succeed");
        assert_eq!(out.messages[0].1, "env ok");
    }
}
