//! Minimal player and item userdata for action (item-use) Lua script execution.
//!
//! `execute_action` creates a fresh Lua state, loads any lib scripts in the
//! action `lib/` directory, loads the script that defines `onUse`, and calls
//! `onUse(player, item, fromPosition, target, toPosition, isHotkey)`.
//!
//! Outputs (messages, creature says, item removal) are captured via
//! `Arc<Mutex<…>>` and returned in [`ActionOutput`].

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use std::path::Path;
use std::sync::{Arc, Mutex};

use forgottenserver_common::position::Position;
use mlua::{UserData, UserDataFields, UserDataMethods, Value};

use crate::lua_bindings::position::LuaPosition;
use crate::lua_bindings::{GameStateHandle, LuaEnvironment};

// ── ActionOutput ──────────────────────────────────────────────────────────────

/// Result returned by [`execute_action`].
#[derive(Debug)]
pub struct ActionOutput {
    /// Text messages queued by `player:sendTextMessage` / `sendCancelMessage`.
    pub messages: Vec<(u8, String)>,
    /// Creature speech entries queued by `player:say(text, talktype)`.
    pub creature_says: Vec<(u8, String)>,
    /// Whether `item:remove(count)` was called (item was consumed).
    pub item_removed: bool,
    /// Magic effects queued by `position:sendMagicEffect(effect)`.
    pub magic_effects: Vec<(Position, u8)>,
}

// ── LuaActionGroup ────────────────────────────────────────────────────────────

struct LuaActionGroup {
    has_access: bool,
}

impl UserData for LuaActionGroup {
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

// ── LuaActionItem ─────────────────────────────────────────────────────────────

struct LuaActionItem {
    item_id: u16,
    pos: Position,
    consumed: Arc<Mutex<bool>>,
    new_item_id: Arc<Mutex<Option<u16>>>,
}

impl UserData for LuaActionItem {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        // food.lua accesses `item.itemid` (field access, not method call)
        fields.add_field_method_get("itemid", |_, this| Ok(this.item_id as i64));
        fields.add_field_method_get("uid", |_, _this| Ok(0i64));
        fields.add_field_method_get("aid", |_, _this| Ok(0i64));
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("getId", |_, this, ()| Ok(this.item_id as i64));
        methods.add_method("getActionId", |_, _this, ()| Ok(0i64));
        methods.add_method("getUniqueId", |_, _this, ()| Ok(0i64));
        methods.add_method("getCount", |_, _this, ()| Ok(1i64));
        methods.add_method("getSubType", |_, _this, ()| Ok(0i64));
        methods.add_method("getType", |_, this, ()| Ok(this.item_id as i64));
        methods.add_method("getPosition", |_, this, ()| Ok(LuaPosition(this.pos)));
        methods.add_method("getName", |_, _this, ()| Ok("".to_string()));
        methods.add_method("isContainer", |_, _this, ()| Ok(false));
        methods.add_method("isMovable", |_, _this, ()| Ok(true));
        methods.add_method("isStackable", |_, _this, ()| Ok(false));
        methods.add_method("hasProperty", |_, _this, _args: Value| Ok(false));
        methods.add_method("getAttribute", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method("setAttribute", |_, _this, _args: Value| Ok(true));
        methods.add_method("getItemHoldingCount", |_, _this, ()| Ok(0i64));

        methods.add_method("remove", |_, this, _args: Value| {
            *this.consumed.lock().unwrap() = true;
            Ok(true)
        });

        methods.add_method("transform", |_, this, new_id: i64| {
            *this.new_item_id.lock().unwrap() = Some(new_id as u16);
            Ok(true)
        });

        methods.add_method("decay", |_, _this, ()| Ok(true));
        methods.add_method("moveTo", |_, _this, _args: Value| Ok(true));
    }
}

// ── LuaActionPlayer ───────────────────────────────────────────────────────────

struct LuaActionPlayer {
    pos: Position,
    messages: Arc<Mutex<Vec<(u8, String)>>>,
    creature_says: Arc<Mutex<Vec<(u8, String)>>>,
    new_pos: Arc<Mutex<Option<Position>>>,
    name: String,
    level: u16,
    has_access: bool,
}

impl UserData for LuaActionPlayer {
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
            const MESSAGE_STATUS_SMALL: u8 = 21;
            this.messages
                .lock()
                .unwrap()
                .push((MESSAGE_STATUS_SMALL, text));
            Ok(true)
        });

        // ── Creature say ("Munch.", "Chomp.", …) ──────────────────────
        methods.add_method("say", |_, this, (text, say_type): (String, i64)| {
            this.creature_says
                .lock()
                .unwrap()
                .push((say_type as u8, text));
            Ok(true)
        });

        // ── Supply / analytics stubs ──────────────────────────────────
        methods.add_method_mut("sendSupplyUsed", |_, _this, _args: Value| Ok(true));
        methods.add_method_mut("sendLootStats", |_, _this, _args: Value| Ok(true));

        // ── Group / access ────────────────────────────────────────────
        methods.add_method("getGroup", |lua, this, ()| {
            lua.create_userdata(LuaActionGroup {
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
        methods.add_method("getIp", |_, _this, ()| Ok("127.0.0.1".to_string()));
        methods.add_method("getGuid", |_, _this, ()| Ok(0i64));
        methods.add_method("getId", |_, _this, ()| Ok(0i64));

        // ── Stats ─────────────────────────────────────────────────────
        methods.add_method("getHealth", |_, _this, ()| Ok(100i64));
        methods.add_method("getMaxHealth", |_, _this, ()| Ok(100i64));
        methods.add_method("getMana", |_, _this, ()| Ok(100i64));
        methods.add_method("getMaxMana", |_, _this, ()| Ok(100i64));
        methods.add_method("getStamina", |_, _this, ()| Ok(2520i64));
        methods.add_method("getSoul", |_, _this, ()| Ok(100i64));
        methods.add_method("getCapacity", |_, _this, ()| Ok(0i64));
        methods.add_method("getSkillLevel", |_, _this, _args: Value| Ok(10i64));
        methods.add_method("getSpecialSkill", |_, _this, _args: Value| Ok(0i64));
        methods.add_method("getFreeCapacity", |_, _this, ()| Ok(0i64));

        // ── Conditions ────────────────────────────────────────────────
        // Return nil → player is never "full" or condition-affected in scripts.
        methods.add_method("getCondition", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method("addCondition", |_, _this, _args: Value| Ok(true));
        methods.add_method("removeCondition", |_, _this, _args: Value| Ok(true));
        methods.add_method("hasCondition", |_, _this, _args: Value| Ok(false));

        // ── Food / regeneration ───────────────────────────────────────
        methods.add_method("feed", |_, _this, _ticks: Value| Ok(true));

        // ── Predicates ────────────────────────────────────────────────
        methods.add_method("isPlayer", |_, _this, ()| Ok(true));
        methods.add_method("isPremium", |_, _this, ()| Ok(false));
        methods.add_method("hasFlag", |_, _this, _args: Value| Ok(false));
        methods.add_method("canSeeCreature", |_, _this, _args: Value| Ok(true));
        methods.add_method("isInGhostMode", |_, _this, ()| Ok(false));

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

// ── execute_action ────────────────────────────────────────────────────────────

/// Parameters for [`execute_action`].
pub struct ActionContext<'a> {
    pub item_id: u16,
    pub item_pos: Position,
    pub player_pos: Position,
    pub player_name: &'a str,
    pub player_level: u16,
    pub has_access: bool,
}

/// Execute an action Lua script file.
///
/// Creates a fresh Lua VM, optionally loads lib scripts from `lib_dir` (e.g.
/// `data/actions/lib/`), loads `script_path`, then calls
/// `onUse(player, item, fromPosition, target, toPosition, isHotkey)`.
///
/// Returns [`ActionOutput`] on success, `Err(String)` on IO/Lua errors.
pub fn execute_action(
    script_path: &Path,
    lib_dir: Option<&Path>,
    ctx: ActionContext<'_>,
) -> Result<ActionOutput, String> {
    let messages: Arc<Mutex<Vec<(u8, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let creature_says: Arc<Mutex<Vec<(u8, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let new_pos: Arc<Mutex<Option<Position>>> = Arc::new(Mutex::new(None));
    let consumed: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let new_item_id_slot: Arc<Mutex<Option<u16>>> = Arc::new(Mutex::new(None));

    let player = LuaActionPlayer {
        pos: ctx.player_pos,
        messages: Arc::clone(&messages),
        creature_says: Arc::clone(&creature_says),
        new_pos: Arc::clone(&new_pos),
        name: ctx.player_name.to_string(),
        level: ctx.player_level,
        has_access: ctx.has_access,
    };

    let item = LuaActionItem {
        item_id: ctx.item_id,
        pos: ctx.item_pos,
        consumed: Arc::clone(&consumed),
        new_item_id: Arc::clone(&new_item_id_slot),
    };

    let magic_effects_buf = Arc::new(Mutex::new(Vec::<(Position, u8)>::new()));

    let mut env = LuaEnvironment::new(GameStateHandle::default())
        .map_err(|e| format!("Lua init failed: {e}"))?;

    env.lua.set_app_data(crate::lua_bindings::MagicEffectsBuffer(Arc::clone(&magic_effects_buf)));

    // Load lib scripts so action scripts can call shared helpers (e.g. onUseRope).
    if let Some(lib) = lib_dir {
        if lib.exists() {
            let _ = env.load_lib_scripts(lib);
        }
    }

    env.load_file(script_path)?;

    let on_use: mlua::Function = env
        .lua
        .globals()
        .get("onUse")
        .map_err(|e| format!("onUse missing in {}: {e}", script_path.display()))?;

    let player_ud = env
        .lua
        .create_userdata(player)
        .map_err(|e| format!("create_userdata(player) failed: {e}"))?;
    let item_ud = env
        .lua
        .create_userdata(item)
        .map_err(|e| format!("create_userdata(item) failed: {e}"))?;
    let from_pos_ud = env
        .lua
        .create_userdata(LuaPosition(ctx.item_pos))
        .map_err(|e| format!("create_userdata(pos) failed: {e}"))?;

    // onUse(player, item, fromPosition, target, toPosition, isHotkey)
    // target / toPosition are nil for basic (non-targeted) use.
    let _result: Value = on_use
        .call((
            player_ud,
            item_ud,
            from_pos_ud,
            Value::Nil,
            Value::Nil,
            false,
        ))
        .map_err(|e| format!("onUse error in {}: {e}", script_path.display()))?;

    let msgs = std::mem::take(&mut *messages.lock().unwrap());
    let says = std::mem::take(&mut *creature_says.lock().unwrap());
    let removed = *consumed.lock().unwrap();
    let effects = std::mem::take(&mut *magic_effects_buf.lock().unwrap());

    Ok(ActionOutput {
        messages: msgs,
        creature_says: says,
        item_removed: removed,
        magic_effects: effects,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
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

    fn run(script: &str, item_id: u16) -> ActionOutput {
        let f = write_script(script);
        execute_action(
            f.path(),
            None,
            ActionContext { item_id, item_pos: default_pos(), player_pos: default_pos(), player_name: "Hero", player_level: 10, has_access: true },
        )
        .expect("execute_action failed")
    }

    #[test]
    fn execute_action_send_text_message() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                player:sendTextMessage(22, "used!")
                return true
            end"#,
            100,
        );
        assert_eq!(out.messages.len(), 1);
        assert_eq!(out.messages[0].1, "used!");
        assert!(!out.item_removed);
    }

    #[test]
    fn execute_action_item_remove_sets_flag() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                item:remove(1)
                return true
            end"#,
            200,
        );
        assert!(out.item_removed);
    }

    #[test]
    fn execute_action_item_id_field_readable() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                player:sendTextMessage(22, "id=" .. item.itemid)
                return true
            end"#,
            42,
        );
        assert_eq!(out.messages[0].1, "id=42");
    }

    #[test]
    fn execute_action_item_get_id_method() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                player:sendTextMessage(22, "id=" .. item:getId())
                return true
            end"#,
            55,
        );
        assert_eq!(out.messages[0].1, "id=55");
    }

    #[test]
    fn execute_action_player_say_recorded() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                player:say("Munch.", 36)
                return true
            end"#,
            100,
        );
        assert_eq!(out.creature_says.len(), 1);
        assert_eq!(out.creature_says[0].1, "Munch.");
        assert_eq!(out.creature_says[0].0, 36);
    }

    #[test]
    fn execute_action_get_condition_returns_nil() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                local cond = player:getCondition(1, 0)
                if cond == nil then
                    player:sendTextMessage(22, "nil")
                else
                    player:sendTextMessage(22, "not nil")
                end
                return true
            end"#,
            100,
        );
        assert_eq!(out.messages[0].1, "nil");
    }

    #[test]
    fn execute_action_feed_is_stub() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                player:feed(60)
                player:sendTextMessage(22, "fed")
                return true
            end"#,
            100,
        );
        assert_eq!(out.messages[0].1, "fed");
    }

    #[test]
    fn execute_action_no_on_use_returns_err() {
        let f = write_script("-- no onUse defined");
        let result = execute_action(
            f.path(),
            None,
            ActionContext { item_id: 100, item_pos: default_pos(), player_pos: default_pos(), player_name: "P", player_level: 1, has_access: true },
        );
        assert!(result.is_err());
    }

    #[test]
    fn execute_action_missing_script_returns_err() {
        let result = execute_action(
            Path::new("/nonexistent/path/action.lua"),
            None,
            ActionContext { item_id: 100, item_pos: default_pos(), player_pos: default_pos(), player_name: "P", player_level: 1, has_access: true },
        );
        assert!(result.is_err());
    }

    #[test]
    fn execute_action_lua_error_returns_err() {
        let f = write_script(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                error("intentional error")
            end"#,
        );
        let result = execute_action(
            f.path(),
            None,
            ActionContext { item_id: 100, item_pos: default_pos(), player_pos: default_pos(), player_name: "P", player_level: 1, has_access: true },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("intentional error"));
    }

    #[test]
    fn execute_action_item_transform_not_crash() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                item:transform(9999)
                player:sendTextMessage(22, "transformed")
                return true
            end"#,
            2041,
        );
        assert_eq!(out.messages[0].1, "transformed");
    }

    #[test]
    fn execute_action_no_messages_when_script_is_silent() {
        let out = run(
            r#"function onUse(player, item, fromPos, target, toPos, isHotkey)
                return true
            end"#,
            100,
        );
        assert!(out.messages.is_empty());
        assert!(out.creature_says.is_empty());
        assert!(!out.item_removed);
    }
}
