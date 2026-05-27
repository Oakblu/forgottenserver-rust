//! Migrated from forgottenserver/src/npc.h and npc.cpp
//!
//! NPC data model without Lua scripting bindings. Focuses on the core data bag:
//! shop items, voiced messages, speech bubble, focus target, and idle movement.
//! Also models the behavioural logic that can be expressed without Lua:
//!   - Event-hook dispatch via `NpcEventHooks` (closure-based trait object stubs)
//!   - `do_sell_item` — stackable vs non-stackable sell-count algorithm
//!   - `turn_to_creature` / `turn_to_offset` — direction calculation
//!   - `get_next_step` — patrol guard (skips random walk while focused)
//!   - Shop-player set management
//!   - `set_idle` guard (no-op when already at target state, or dead)
//!   - `on_creature_say` self-filter (ignores self-talk)

use std::collections::HashSet;

use crate::creature::{Creature, Direction};

// ---------------------------------------------------------------------------
// SpeechBubble
// ---------------------------------------------------------------------------

/// Mirrors the C++ `SpeechBubble_t` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SpeechBubble {
    #[default]
    None = 0,
    Normal = 1,
    Trade = 2,
    Quest = 3,
    QuestTrader = 4,
    Hireling = 5,
}

// ---------------------------------------------------------------------------
// ShopItem
// ---------------------------------------------------------------------------

/// A single shop offer for an NPC. Mirrors the C++ `ShopInfo` struct.
#[derive(Debug, Clone, PartialEq)]
pub struct ShopItem {
    pub item_id: u16,
    pub sub_type: u32,
    pub buy_price: u32,
    pub sell_price: u32,
    pub name: String,
    pub weight: f64,
}

impl ShopItem {
    pub fn new(
        item_id: u16,
        sub_type: u32,
        buy_price: u32,
        sell_price: u32,
        name: impl Into<String>,
        weight: f64,
    ) -> Self {
        ShopItem {
            item_id,
            sub_type,
            buy_price,
            sell_price,
            name: name.into(),
            weight,
        }
    }
}

// ---------------------------------------------------------------------------
// SpeakClass — mirrors C++ SpeakClasses
// ---------------------------------------------------------------------------

/// Speak class used in `on_creature_say`. Mirrors `SpeakClasses` (subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpeakClass {
    Say = 1,
    Whisper = 2,
    Yell = 3,
    PrivateNp = 4,
    Other = 255,
}

// ---------------------------------------------------------------------------
// Type aliases for complex hook closures
// ---------------------------------------------------------------------------

/// Hook type for `on_creature_say(creature_id, speak_class, text)`.
type CreatureSayHook = Box<dyn Fn(u32, SpeakClass, String) + Send + Sync>;

/// Hook type for `on_player_trade(params)`.
type PlayerTradeHook = Box<dyn Fn(PlayerTradeParams) + Send + Sync>;

// ---------------------------------------------------------------------------
// NpcEventHooks — pure-Rust event dispatch stubs
// ---------------------------------------------------------------------------

/// Closure-based event hooks that mirror the C++ `NpcEventsHandler` dispatch.
///
/// Each field is `Option<Box<dyn Fn(…)>>` so callers can wire up test doubles
/// without a real Lua VM. When `None`, the hook is considered unregistered
/// (equivalent to `event_id == -1` in the C++ code).
///
/// NOTE: Lua script loading and Lua state management are intentionally absent
/// here; they belong in the scripting crate, not the entity crate.
#[derive(Default)]
pub struct NpcEventHooks {
    /// Fired when a creature appears near the NPC (including the NPC itself).
    pub on_creature_appear: Option<Box<dyn Fn(u32) + Send + Sync>>,

    /// Fired when a creature disappears / leaves range.
    pub on_creature_disappear: Option<Box<dyn Fn(u32) + Send + Sync>>,

    /// Fired when a creature (player) moves.
    pub on_creature_move: Option<Box<dyn Fn(u32) + Send + Sync>>,

    /// Fired when a player says something near the NPC (never for self-speak).
    pub on_creature_say: Option<CreatureSayHook>,

    /// Fired every think tick.
    pub on_think: Option<Box<dyn Fn() + Send + Sync>>,

    /// Fired when a trade callback needs invoking (buy or sell).
    /// Signature mirrors C++ `onPlayerTrade`: (player_id, item_id, count, amount, ignore, in_backpacks).
    pub on_player_trade: Option<PlayerTradeHook>,

    /// Fired when a player closes the NPC channel.
    pub on_player_close_channel: Option<Box<dyn Fn(u32) + Send + Sync>>,

    /// Fired when a player ends the trade session.
    pub on_player_end_trade: Option<Box<dyn Fn(u32) + Send + Sync>>,
}

impl std::fmt::Debug for NpcEventHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NpcEventHooks")
            .field("on_creature_appear", &self.on_creature_appear.is_some())
            .field(
                "on_creature_disappear",
                &self.on_creature_disappear.is_some(),
            )
            .field("on_creature_move", &self.on_creature_move.is_some())
            .field("on_creature_say", &self.on_creature_say.is_some())
            .field("on_think", &self.on_think.is_some())
            .field("on_player_trade", &self.on_player_trade.is_some())
            .field(
                "on_player_close_channel",
                &self.on_player_close_channel.is_some(),
            )
            .field("on_player_end_trade", &self.on_player_end_trade.is_some())
            .finish()
    }
}

impl NpcEventHooks {
    /// Returns `true` when at least one hook is registered (mirrors `isLoaded()`).
    pub fn is_loaded(&self) -> bool {
        self.on_creature_appear.is_some()
            || self.on_creature_disappear.is_some()
            || self.on_creature_move.is_some()
            || self.on_creature_say.is_some()
            || self.on_think.is_some()
            || self.on_player_trade.is_some()
            || self.on_player_close_channel.is_some()
            || self.on_player_end_trade.is_some()
    }

    /// Fire `on_creature_appear` if registered.
    pub fn fire_creature_appear(&self, creature_id: u32) {
        if let Some(f) = &self.on_creature_appear {
            f(creature_id);
        }
    }

    /// Fire `on_creature_disappear` if registered.
    pub fn fire_creature_disappear(&self, creature_id: u32) {
        if let Some(f) = &self.on_creature_disappear {
            f(creature_id);
        }
    }

    /// Fire `on_creature_move` if registered.
    pub fn fire_creature_move(&self, creature_id: u32) {
        if let Some(f) = &self.on_creature_move {
            f(creature_id);
        }
    }

    /// Fire `on_creature_say` if registered.
    /// Mirrors C++ `NpcEventsHandler::onCreatureSay`.
    pub fn fire_creature_say(&self, creature_id: u32, speak_class: SpeakClass, text: String) {
        if let Some(f) = &self.on_creature_say {
            f(creature_id, speak_class, text);
        }
    }

    /// Fire `on_think` if registered.
    pub fn fire_think(&self) {
        if let Some(f) = &self.on_think {
            f();
        }
    }

    /// Fire `on_player_trade` if registered.
    /// `callback == -1` equivalent: caller should skip firing when no callback
    /// is wired (mirrors the C++ `if (callback == -1) return` guard).
    pub fn fire_player_trade(&self, params: PlayerTradeParams) {
        if let Some(f) = &self.on_player_trade {
            f(params);
        }
    }

    /// Fire `on_player_close_channel` if registered.
    pub fn fire_player_close_channel(&self, player_id: u32) {
        if let Some(f) = &self.on_player_close_channel {
            f(player_id);
        }
    }

    /// Fire `on_player_end_trade` if registered.
    pub fn fire_player_end_trade(&self, player_id: u32) {
        if let Some(f) = &self.on_player_end_trade {
            f(player_id);
        }
    }
}

// ---------------------------------------------------------------------------
// PlayerTradeParams — groups parameters for on_player_trade
// ---------------------------------------------------------------------------

/// Parameters for a player trade event. Groups the arguments from C++
/// `NpcEventsHandler::onPlayerTrade` to avoid a too-many-arguments signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerTradeParams {
    pub player_id: u32,
    pub item_id: u16,
    pub count: u8,
    pub amount: u16,
    pub ignore: bool,
    pub in_backpacks: bool,
}

// ---------------------------------------------------------------------------
// DoSellResult — result of do_sell_item
// ---------------------------------------------------------------------------

/// Describes the outcome of a `do_sell_item` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoSellResult {
    /// Number of individual items successfully "added to the player".
    pub sold_count: u32,
    /// `true` when all requested items were delivered.
    pub complete: bool,
}

// ---------------------------------------------------------------------------
// Item capacity limit (mirrors ITEM_STACK_SIZE in C++)
// ---------------------------------------------------------------------------

/// Maximum stack size for stackable items (mirrors C++ `ITEM_STACK_SIZE`).
pub const ITEM_STACK_SIZE: u32 = 100;

// ---------------------------------------------------------------------------
// do_sell_item — pure logic (no Game/map coupling)
// ---------------------------------------------------------------------------

/// Models the `doSellItem` count algorithm from C++ without game I/O.
///
/// `can_accept` is a closure called once per chunk/unit; returning `false`
/// simulates `internalPlayerAddItem` failing (inventory full, etc.).
/// Mirrors the C++ function signature:
///   `doSellItem(player, itemid, amount, subtype, actionid, canDropOnMap)`
///
/// Returns a `DoSellResult` with `sold_count` and `complete`.
pub fn do_sell_item<F>(stackable: bool, amount: u32, mut can_accept: F) -> DoSellResult
where
    F: FnMut(u32) -> bool,
{
    let mut sold_count: u32 = 0;
    if stackable {
        let mut remaining = amount;
        while remaining > 0 {
            let chunk = remaining.min(ITEM_STACK_SIZE);
            if !can_accept(chunk) {
                return DoSellResult {
                    sold_count,
                    complete: false,
                };
            }
            sold_count += chunk;
            remaining -= chunk;
        }
    } else {
        for _ in 0..amount {
            if !can_accept(1) {
                return DoSellResult {
                    sold_count,
                    complete: false,
                };
            }
            sold_count += 1;
        }
    }
    DoSellResult {
        sold_count,
        complete: true,
    }
}

// ---------------------------------------------------------------------------
// turn_to_offset — direction calculation
// ---------------------------------------------------------------------------

/// Compute the direction an NPC should face to look at a target offset.
///
/// Mirrors C++ `Npc::turnToCreature`:
/// ```text
///   dx = myPos.getOffsetX(targetPos)   (positive = target is WEST of npc)
///   dy = myPos.getOffsetY(targetPos)   (positive = target is NORTH of npc)
///   tan = dy / dx  (or 10 if dx==0)
///   |tan| < 1  →  horizontal axis wins  →  East/West
///   |tan| >= 1 →  vertical axis wins    →  North/South
/// ```
///
/// `dx` = npc_x - target_x  (C++ `myPos.getOffsetX(targetPos)`)
/// `dy` = npc_y - target_y  (C++ `myPos.getOffsetY(targetPos)`)
pub fn turn_to_offset(dx: i32, dy: i32) -> Direction {
    let tan: f64 = if dx != 0 { dy as f64 / dx as f64 } else { 10.0 };

    if tan.abs() < 1.0 {
        // horizontal axis dominates
        if dx > 0 {
            Direction::West
        } else {
            Direction::East
        }
    } else {
        // vertical axis dominates
        if dy > 0 {
            Direction::North
        } else {
            Direction::South
        }
    }
}

// ---------------------------------------------------------------------------
// Npc
// ---------------------------------------------------------------------------

/// Core NPC data-bag. Does not include Lua scripting state.
///
/// Mirrors the data members of the C++ `Npc` class relevant to game logic.
#[derive(Debug)]
pub struct Npc {
    pub creature: Creature,

    script_name: String,

    shop_items: Vec<ShopItem>,

    focused_creature_id: Option<u32>,

    voiced_messages: Vec<String>,

    speech_bubble: SpeechBubble,

    /// Walk ticks (idle movement interval) in milliseconds. Default 1500 (C++ default).
    idle_interval: u32,

    /// Whether the NPC is currently idle (no spectators in range).
    /// Mirrors C++ `isIdle`.
    is_idle: bool,

    /// Whether the NPC is alive (not dead/removed). Used by `set_idle` guard.
    /// Mirrors C++ `isRemoved() || isDead()` guard.
    is_alive: bool,

    /// Set of player IDs currently viewing the shop window.
    /// Mirrors C++ `shopPlayerSet`.
    shop_player_ids: HashSet<u32>,

    /// Event hooks (Lua-less dispatch stubs).
    pub hooks: NpcEventHooks,
}

impl Npc {
    /// Create a new NPC with the given name and default values.
    pub fn new(name: impl Into<String>) -> Self {
        let name_str: String = name.into();
        Npc {
            creature: Creature::new(0, name_str),
            script_name: String::new(),
            shop_items: Vec::new(),
            focused_creature_id: None,
            voiced_messages: Vec::new(),
            speech_bubble: SpeechBubble::None,
            idle_interval: 1500,
            is_idle: true,
            is_alive: true,
            shop_player_ids: HashSet::new(),
            hooks: NpcEventHooks::default(),
        }
    }

    // --- Basic accessors ---

    pub fn get_name(&self) -> &str {
        self.creature.get_name()
    }

    pub fn get_script_name(&self) -> &str {
        &self.script_name
    }

    pub fn set_script_name(&mut self, name: impl Into<String>) {
        self.script_name = name.into();
    }

    // --- Shop ---

    pub fn add_shop_item(&mut self, item: ShopItem) {
        self.shop_items.push(item);
    }

    /// Returns a reference to the first ShopItem with the given `item_id`, or `None`.
    pub fn get_shop_item(&self, item_id: u16) -> Option<&ShopItem> {
        self.shop_items.iter().find(|s| s.item_id == item_id)
    }

    pub fn get_shop_count(&self) -> usize {
        self.shop_items.len()
    }

    // --- Shop player set ---

    /// Add a player to the set of players viewing this shop.
    /// Mirrors C++ `addShopPlayer`.
    pub fn add_shop_player(&mut self, player_id: u32) {
        self.shop_player_ids.insert(player_id);
    }

    /// Remove a player from the shop-player set.
    /// Mirrors C++ `removeShopPlayer`.
    pub fn remove_shop_player(&mut self, player_id: u32) {
        self.shop_player_ids.remove(&player_id);
    }

    /// Returns `true` if the player is currently tracked in the shop-player set.
    pub fn has_shop_player(&self, player_id: u32) -> bool {
        self.shop_player_ids.contains(&player_id)
    }

    /// Returns the number of players currently viewing the shop window.
    pub fn shop_player_count(&self) -> usize {
        self.shop_player_ids.len()
    }

    /// Drain the shop-player set and return the IDs so the caller can close
    /// each player's shop window. Mirrors C++ `closeAllShopWindows`.
    pub fn drain_shop_players(&mut self) -> Vec<u32> {
        self.shop_player_ids.drain().collect()
    }

    // --- Focus / unfocus ---

    pub fn get_focused_creature_id(&self) -> Option<u32> {
        self.focused_creature_id
    }

    pub fn set_focus(&mut self, creature_id: u32) {
        self.focused_creature_id = Some(creature_id);
    }

    pub fn clear_focus(&mut self) {
        self.focused_creature_id = None;
    }

    /// Whether the NPC is currently focused on a creature.
    /// When focused, random patrol steps are suppressed (mirrors C++ `getNextStep` guard).
    pub fn is_focused(&self) -> bool {
        self.focused_creature_id.is_some()
    }

    // --- turnToCreature (direction calculation) ---

    /// Compute the direction to face in order to look at `target_id`.
    /// The caller provides the NPC position offset relative to the target
    /// (`dx = npc_x - target_x`, `dy = npc_y - target_y`) as the C++ code does.
    ///
    /// Returns the direction the NPC should turn to, WITHOUT applying it to
    /// `creature.direction` so that callers can decide whether to send a
    /// network update. Use `apply_turn_to_offset` to also store the result.
    pub fn compute_turn_direction(&self, dx: i32, dy: i32) -> Direction {
        turn_to_offset(dx, dy)
    }

    /// Compute and store the direction the NPC faces toward the given offset.
    /// Mirrors C++ `Npc::turnToCreature` (updates internal direction).
    pub fn apply_turn_to_offset(&mut self, dx: i32, dy: i32) {
        self.creature.direction = turn_to_offset(dx, dy);
    }

    // --- Idle state management ---

    pub fn is_idle(&self) -> bool {
        self.is_idle
    }

    /// Mark the NPC as alive (default) or dead/removed.
    /// When dead, `set_idle` becomes a no-op, mirroring the C++ guard
    /// `if (isRemoved() || isDead()) return;`.
    pub fn set_alive(&mut self, alive: bool) {
        self.is_alive = alive;
    }

    /// Toggle idle state. Guards:
    ///   - No-op if already at the requested state (mirrors C++ `if (idle == isIdle) return`).
    ///   - No-op if the NPC is dead/removed (mirrors `if (isRemoved() || isDead()) return`).
    pub fn set_idle(&mut self, idle: bool) {
        if idle == self.is_idle {
            return;
        }
        if !self.is_alive {
            return;
        }
        self.is_idle = idle;
    }

    // --- get_next_step patrol guard ---

    /// Returns `false` (no random walk step) when the NPC is focused on a creature,
    /// mirroring the C++ `Npc::getNextStep` guard:
    ///   `if (focusCreature != 0) return false;`
    ///
    /// Also returns `false` when the elapsed time since last move is less than
    /// `idle_interval`. Callers supply `time_since_last_move_ms` to avoid
    /// coupling to a system clock here.
    ///
    /// When both guards pass, returns `true` meaning a random step should be taken.
    pub fn should_take_patrol_step(&self, time_since_last_move_ms: u32) -> bool {
        if self.is_focused() {
            return false;
        }
        if self.idle_interval == 0 {
            return false;
        }
        time_since_last_move_ms >= self.idle_interval
    }

    // --- on_creature_say self-filter ---

    /// Handle a creature saying something. Mirrors C++ `Npc::onCreatureSay`:
    ///   - If `speaker_id == self.creature.id`, return immediately (NPC ignores itself).
    ///   - Otherwise fires the `on_creature_say` hook.
    ///
    /// Returns `true` when the hook was fired, `false` when it was suppressed.
    pub fn on_creature_say(
        &self,
        speaker_id: u32,
        speak_class: SpeakClass,
        text: impl Into<String>,
    ) -> bool {
        if speaker_id == self.creature.id {
            return false; // NPC ignores its own speech
        }
        self.hooks
            .fire_creature_say(speaker_id, speak_class, text.into());
        true
    }

    // --- on_player_trade ---

    /// Trigger the trade callback (buy or sell) with the given parameters.
    /// Mirrors C++ `Npc::onPlayerTrade` → `npcEventHandler->onPlayerTrade(...)`.
    /// The `has_callback` flag mirrors the C++ `callback == -1` guard:
    /// if `false`, the hook is not fired.
    ///
    /// Also calls `player.sendSaleItemList()` in C++; callers of this method
    /// must handle that separately.
    pub fn on_player_trade(&self, params: PlayerTradeParams, has_callback: bool) {
        if has_callback {
            self.hooks.fire_player_trade(params);
        }
    }

    // --- on_think ---

    /// Fire the `on_think` hook and conditionally schedule a patrol step.
    /// Mirrors C++ `Npc::onThink`.
    ///
    /// Returns `true` when a patrol event should be added (i.e. the NPC is
    /// not idle and sufficient time has elapsed).
    pub fn on_think(&self, time_since_last_move_ms: u32) -> bool {
        self.hooks.fire_think();
        !self.is_idle && self.should_take_patrol_step(time_since_last_move_ms)
    }

    // --- Voiced messages ---

    pub fn add_voiced_message(&mut self, message: impl Into<String>) {
        self.voiced_messages.push(message.into());
    }

    pub fn get_voiced_messages(&self) -> &[String] {
        &self.voiced_messages
    }

    /// Select a voiced message deterministically: index = seed % count.
    /// Returns `None` if there are no messages.
    pub fn select_voiced_message(&self, seed: usize) -> Option<&str> {
        if self.voiced_messages.is_empty() {
            return None;
        }
        Some(&self.voiced_messages[seed % self.voiced_messages.len()])
    }

    // --- Speech bubble ---

    pub fn get_speech_bubble(&self) -> SpeechBubble {
        self.speech_bubble
    }

    pub fn set_speech_bubble(&mut self, bubble: SpeechBubble) {
        self.speech_bubble = bubble;
    }

    // --- Idle movement ---

    pub fn get_idle_interval(&self) -> u32 {
        self.idle_interval
    }

    pub fn set_idle_interval(&mut self, ms: u32) {
        self.idle_interval = ms;
    }

    // --- Description ---

    /// Returns the NPC description shown to inspecting players. Mirrors
    /// C++ `Npc::getDescription` which produces `"<name>."` regardless of
    /// the look-distance parameter (the C++ implementation ignores it).
    pub fn get_description(&self) -> String {
        let name = self.creature.get_name();
        let mut descr = String::with_capacity(name.len() + 1);
        descr.push_str(name);
        descr.push('.');
        descr
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // --- ShopItem struct ---

    #[test]
    fn test_shop_item_new_fields() {
        let item = ShopItem::new(101, 1, 500, 250, "Crystal Sword", 3.5);
        assert_eq!(item.item_id, 101);
        assert_eq!(item.sub_type, 1);
        assert_eq!(item.buy_price, 500);
        assert_eq!(item.sell_price, 250);
        assert_eq!(item.name, "Crystal Sword");
        assert!((item.weight - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_shop_item_zero_prices() {
        let item = ShopItem::new(1, 0, 0, 0, "Torch", 0.0);
        assert_eq!(item.buy_price, 0);
        assert_eq!(item.sell_price, 0);
    }

    // --- NPC basics ---

    #[test]
    fn test_npc_new_get_name() {
        let npc = Npc::new("Old Man");
        assert_eq!(npc.get_name(), "Old Man");
    }

    #[test]
    fn test_npc_new_script_name_empty() {
        let npc = Npc::new("Bob");
        assert_eq!(npc.get_script_name(), "");
    }

    #[test]
    fn test_npc_set_script_name_roundtrip() {
        let mut npc = Npc::new("Bob");
        npc.set_script_name("merchant.lua");
        assert_eq!(npc.get_script_name(), "merchant.lua");
    }

    #[test]
    fn test_npc_new_shop_empty() {
        let npc = Npc::new("Bob");
        assert_eq!(npc.get_shop_count(), 0);
    }

    // --- Shop ---

    #[test]
    fn test_npc_add_shop_item_increases_count() {
        let mut npc = Npc::new("Bob");
        npc.add_shop_item(ShopItem::new(10, 0, 100, 50, "Sword", 2.0));
        assert_eq!(npc.get_shop_count(), 1);
    }

    #[test]
    fn test_npc_get_shop_item_found() {
        let mut npc = Npc::new("Bob");
        npc.add_shop_item(ShopItem::new(10, 0, 100, 50, "Sword", 2.0));
        let item = npc.get_shop_item(10);
        assert!(item.is_some());
        assert_eq!(item.unwrap().name, "Sword");
    }

    #[test]
    fn test_npc_get_shop_item_not_found() {
        let npc = Npc::new("Bob");
        assert!(npc.get_shop_item(99).is_none());
    }

    #[test]
    fn test_npc_get_shop_count_multiple() {
        let mut npc = Npc::new("Bob");
        npc.add_shop_item(ShopItem::new(1, 0, 10, 5, "A", 1.0));
        npc.add_shop_item(ShopItem::new(2, 0, 20, 10, "B", 2.0));
        npc.add_shop_item(ShopItem::new(3, 0, 30, 15, "C", 3.0));
        assert_eq!(npc.get_shop_count(), 3);
    }

    // --- Focus / unfocus ---

    #[test]
    fn test_npc_focused_creature_id_none_initially() {
        let npc = Npc::new("Bob");
        assert_eq!(npc.get_focused_creature_id(), None);
    }

    #[test]
    fn test_npc_set_focus() {
        let mut npc = Npc::new("Bob");
        npc.set_focus(42);
        assert_eq!(npc.get_focused_creature_id(), Some(42));
    }

    #[test]
    fn test_npc_clear_focus() {
        let mut npc = Npc::new("Bob");
        npc.set_focus(42);
        npc.clear_focus();
        assert_eq!(npc.get_focused_creature_id(), None);
    }

    #[test]
    fn test_npc_is_focused_true_when_set() {
        let mut npc = Npc::new("Bob");
        assert!(!npc.is_focused());
        npc.set_focus(7);
        assert!(npc.is_focused());
    }

    #[test]
    fn test_npc_is_focused_false_after_clear() {
        let mut npc = Npc::new("Bob");
        npc.set_focus(7);
        npc.clear_focus();
        assert!(!npc.is_focused());
    }

    // --- Voiced messages ---

    #[test]
    fn test_npc_add_voiced_message() {
        let mut npc = Npc::new("Bob");
        npc.add_voiced_message("Hello traveler!");
        assert_eq!(npc.get_voiced_messages().len(), 1);
    }

    #[test]
    fn test_npc_get_voiced_messages_all() {
        let mut npc = Npc::new("Bob");
        npc.add_voiced_message("Hello!");
        npc.add_voiced_message("Goodbye!");
        npc.add_voiced_message("Come trade!");
        let msgs = npc.get_voiced_messages();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0], "Hello!");
        assert_eq!(msgs[1], "Goodbye!");
        assert_eq!(msgs[2], "Come trade!");
    }

    #[test]
    fn test_npc_select_voiced_message_deterministic() {
        let mut npc = Npc::new("Bob");
        npc.add_voiced_message("Msg0");
        npc.add_voiced_message("Msg1");
        npc.add_voiced_message("Msg2");
        assert_eq!(npc.select_voiced_message(0), Some("Msg0"));
        assert_eq!(npc.select_voiced_message(1), Some("Msg1"));
        assert_eq!(npc.select_voiced_message(2), Some("Msg2"));
        assert_eq!(npc.select_voiced_message(3), Some("Msg0")); // wraps
    }

    #[test]
    fn test_npc_select_voiced_message_empty_returns_none() {
        let npc = Npc::new("Bob");
        assert_eq!(npc.select_voiced_message(0), None);
    }

    // --- Speech bubble ---

    #[test]
    fn test_npc_speech_bubble_none_by_default() {
        let npc = Npc::new("Bob");
        assert_eq!(npc.get_speech_bubble(), SpeechBubble::None);
    }

    #[test]
    fn test_npc_set_speech_bubble() {
        let mut npc = Npc::new("Bob");
        npc.set_speech_bubble(SpeechBubble::Trade);
        assert_eq!(npc.get_speech_bubble(), SpeechBubble::Trade);
    }

    #[test]
    fn test_npc_set_speech_bubble_all_variants() {
        let variants = [
            SpeechBubble::None,
            SpeechBubble::Normal,
            SpeechBubble::Trade,
            SpeechBubble::Quest,
            SpeechBubble::QuestTrader,
            SpeechBubble::Hireling,
        ];
        for variant in variants {
            let mut npc = Npc::new("Bob");
            npc.set_speech_bubble(variant);
            assert_eq!(npc.get_speech_bubble(), variant);
        }
    }

    // --- Idle movement ---

    #[test]
    fn test_npc_idle_interval_default_1500() {
        let npc = Npc::new("Bob");
        // C++ default walkTicks = 1500
        assert_eq!(npc.get_idle_interval(), 1500);
    }

    #[test]
    fn test_npc_set_idle_interval() {
        let mut npc = Npc::new("Bob");
        npc.set_idle_interval(2000);
        assert_eq!(npc.get_idle_interval(), 2000);
    }

    // -----------------------------------------------------------------------
    // NEW: set_idle guard
    // -----------------------------------------------------------------------

    #[test]
    fn test_npc_is_idle_true_by_default() {
        let npc = Npc::new("Bob");
        assert!(npc.is_idle());
    }

    #[test]
    fn test_set_idle_toggles_state() {
        let mut npc = Npc::new("Bob");
        npc.set_idle(false);
        assert!(!npc.is_idle());
        npc.set_idle(true);
        assert!(npc.is_idle());
    }

    #[test]
    fn test_set_idle_no_op_when_already_at_target() {
        let mut npc = Npc::new("Bob");
        // Already idle; setting idle again should not change anything
        npc.set_idle(true);
        assert!(npc.is_idle());
        npc.set_idle(false);
        // Now not idle; setting not-idle again → no-op
        npc.set_idle(false);
        assert!(!npc.is_idle());
    }

    #[test]
    fn test_set_idle_no_op_when_dead() {
        let mut npc = Npc::new("Bob");
        npc.set_alive(false);
        // Attempting to un-idle while dead → no-op
        npc.set_idle(false);
        assert!(npc.is_idle()); // still idle
    }

    #[test]
    fn test_set_idle_allowed_when_alive() {
        let mut npc = Npc::new("Bob");
        npc.set_alive(true);
        npc.set_idle(false);
        assert!(!npc.is_idle());
    }

    // -----------------------------------------------------------------------
    // NEW: shop player set management
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_shop_player_tracked() {
        let mut npc = Npc::new("Bob");
        npc.add_shop_player(1001);
        assert!(npc.has_shop_player(1001));
    }

    #[test]
    fn test_remove_shop_player() {
        let mut npc = Npc::new("Bob");
        npc.add_shop_player(1001);
        npc.remove_shop_player(1001);
        assert!(!npc.has_shop_player(1001));
    }

    #[test]
    fn test_shop_player_count() {
        let mut npc = Npc::new("Bob");
        assert_eq!(npc.shop_player_count(), 0);
        npc.add_shop_player(1);
        npc.add_shop_player(2);
        assert_eq!(npc.shop_player_count(), 2);
    }

    #[test]
    fn test_drain_shop_players_clears_set() {
        let mut npc = Npc::new("Bob");
        npc.add_shop_player(10);
        npc.add_shop_player(20);
        let drained = npc.drain_shop_players();
        assert_eq!(drained.len(), 2);
        assert_eq!(npc.shop_player_count(), 0);
    }

    #[test]
    fn test_add_duplicate_shop_player_counted_once() {
        let mut npc = Npc::new("Bob");
        npc.add_shop_player(42);
        npc.add_shop_player(42);
        assert_eq!(npc.shop_player_count(), 1);
    }

    #[test]
    fn test_remove_nonexistent_shop_player_is_no_op() {
        let mut npc = Npc::new("Bob");
        npc.remove_shop_player(999); // must not panic
        assert_eq!(npc.shop_player_count(), 0);
    }

    // -----------------------------------------------------------------------
    // NEW: turn_to_offset direction calculation (mirrors turnToCreature)
    // -----------------------------------------------------------------------

    // dx > 0, |tan| < 1 → WEST
    #[test]
    fn test_turn_to_offset_west_dx_positive_small_dy() {
        // dx=5, dy=1 → tan=0.2, |tan|<1, dx>0 → West
        assert_eq!(turn_to_offset(5, 1), Direction::West);
    }

    // dx < 0, |tan| < 1 → EAST
    #[test]
    fn test_turn_to_offset_east_dx_negative_small_dy() {
        assert_eq!(turn_to_offset(-5, 1), Direction::East);
    }

    // dy > 0, |tan| >= 1 → NORTH
    #[test]
    fn test_turn_to_offset_north_dy_positive_large() {
        // dx=1, dy=5 → tan=5, |tan|>=1, dy>0 → North
        assert_eq!(turn_to_offset(1, 5), Direction::North);
    }

    // dy < 0, |tan| >= 1 → SOUTH
    #[test]
    fn test_turn_to_offset_south_dy_negative_large() {
        assert_eq!(turn_to_offset(1, -5), Direction::South);
    }

    // dx=0 → tan=10, |tan|>=1, dy>0 → NORTH
    #[test]
    fn test_turn_to_offset_dx_zero_north() {
        assert_eq!(turn_to_offset(0, 3), Direction::North);
    }

    // dx=0, dy<0 → SOUTH
    #[test]
    fn test_turn_to_offset_dx_zero_south() {
        assert_eq!(turn_to_offset(0, -3), Direction::South);
    }

    // diagonal: equal dx/dy → |tan|==1 which is NOT < 1, so vertical wins
    #[test]
    fn test_turn_to_offset_equal_dx_dy_positive_north() {
        // tan=1.0, not < 1 → North
        assert_eq!(turn_to_offset(3, 3), Direction::North);
    }

    #[test]
    fn test_compute_turn_direction_matches_turn_to_offset() {
        let npc = Npc::new("Bob");
        assert_eq!(npc.compute_turn_direction(5, 1), Direction::West);
    }

    #[test]
    fn test_apply_turn_to_offset_updates_direction() {
        let mut npc = Npc::new("Bob");
        npc.apply_turn_to_offset(0, 5); // → North
        assert_eq!(npc.creature.direction, Direction::North);
    }

    // -----------------------------------------------------------------------
    // NEW: get_next_step patrol guard
    // -----------------------------------------------------------------------

    #[test]
    fn test_should_take_patrol_step_not_focused_enough_time() {
        let mut npc = Npc::new("Bob");
        npc.set_idle_interval(1500);
        // Not focused, time >= interval
        assert!(npc.should_take_patrol_step(1500));
        assert!(npc.should_take_patrol_step(2000));
    }

    #[test]
    fn test_should_take_patrol_step_false_when_focused() {
        let mut npc = Npc::new("Bob");
        npc.set_focus(99);
        // Focused → no random walk even with enough time
        assert!(!npc.should_take_patrol_step(9999));
    }

    #[test]
    fn test_should_take_patrol_step_false_not_enough_time() {
        let npc = Npc::new("Bob");
        assert!(!npc.should_take_patrol_step(1000)); // interval=1500
    }

    #[test]
    fn test_should_take_patrol_step_false_when_walk_ticks_zero() {
        let mut npc = Npc::new("Bob");
        npc.set_idle_interval(0);
        // walkTicks == 0 → NPC never walks randomly
        assert!(!npc.should_take_patrol_step(99999));
    }

    // -----------------------------------------------------------------------
    // NEW: on_creature_say self-filter
    // -----------------------------------------------------------------------

    #[test]
    fn test_on_creature_say_ignores_self() {
        let mut npc = Npc::new("Bob");
        npc.creature.id = 55;
        let fired = Arc::new(Mutex::new(false));
        let fired_clone = Arc::clone(&fired);
        npc.hooks.on_creature_say = Some(Box::new(move |_id, _class, _text| {
            *fired_clone.lock().unwrap() = true
        }));

        // Self speaks — hook must NOT fire
        let result = npc.on_creature_say(55, SpeakClass::Say, "hello");
        assert!(!result);
        assert!(!*fired.lock().unwrap());

        // Sanity: prove the same closure DOES fire for a different speaker
        // (this also covers the closure body so the self-filter test does not
        // rely on a never-executed closure for branch coverage).
        let result2 = npc.on_creature_say(99, SpeakClass::Say, "hello");
        assert!(result2);
        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn test_on_creature_say_fires_for_other_creature() {
        let mut npc = Npc::new("Bob");
        npc.creature.id = 55;
        let captured = Arc::new(Mutex::new(None::<(u32, String)>));
        let cap_clone = Arc::clone(&captured);
        npc.hooks.on_creature_say = Some(Box::new(move |id, _class, text| {
            *cap_clone.lock().unwrap() = Some((id, text));
        }));

        let result = npc.on_creature_say(77, SpeakClass::Say, "hello");
        assert!(result);
        let val = captured.lock().unwrap();
        assert_eq!(*val, Some((77, "hello".to_string())));
    }

    #[test]
    fn test_on_creature_say_no_hook_returns_true_not_self() {
        let npc = Npc::new("Bob");
        // no hook, non-self speaker → should still return true (hook would have fired)
        assert!(npc.on_creature_say(99, SpeakClass::Say, "test"));
    }

    // -----------------------------------------------------------------------
    // NEW: NpcEventHooks dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn test_hooks_is_loaded_false_by_default() {
        let hooks = NpcEventHooks::default();
        assert!(!hooks.is_loaded());
    }

    #[test]
    fn test_hooks_is_loaded_true_when_any_hook_set() {
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let mut hooks = NpcEventHooks::default();
        // Use a closure with an observable body and fire it so the body is
        // covered (otherwise the closure-body lines remain uncovered).
        hooks.on_think = Some(Box::new(move || *c.lock().unwrap() += 1));
        assert!(hooks.is_loaded());
        hooks.fire_think();
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_hooks_fire_creature_appear() {
        let called = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&called);
        let mut hooks = NpcEventHooks::default();
        hooks.on_creature_appear = Some(Box::new(move |id| *c.lock().unwrap() = id));
        hooks.fire_creature_appear(42);
        assert_eq!(*called.lock().unwrap(), 42);
    }

    #[test]
    fn test_hooks_fire_creature_appear_no_hook_is_no_op() {
        let hooks = NpcEventHooks::default();
        hooks.fire_creature_appear(1); // must not panic
    }

    #[test]
    fn test_hooks_fire_creature_disappear() {
        let called = Arc::new(Mutex::new(false));
        let c = Arc::clone(&called);
        let mut hooks = NpcEventHooks::default();
        hooks.on_creature_disappear = Some(Box::new(move |_id| *c.lock().unwrap() = true));
        hooks.fire_creature_disappear(5);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_hooks_fire_creature_disappear_no_hook_is_no_op() {
        let hooks = NpcEventHooks::default();
        hooks.fire_creature_disappear(1); // must not panic
    }

    #[test]
    fn test_hooks_fire_creature_move_no_hook_is_no_op() {
        let hooks = NpcEventHooks::default();
        hooks.fire_creature_move(1); // must not panic
    }

    #[test]
    fn test_hooks_fire_player_trade_no_hook_is_no_op() {
        let hooks = NpcEventHooks::default();
        let params = PlayerTradeParams {
            player_id: 0,
            item_id: 0,
            count: 0,
            amount: 0,
            ignore: false,
            in_backpacks: false,
        };
        hooks.fire_player_trade(params); // must not panic
    }

    #[test]
    fn test_hooks_fire_player_close_channel_no_hook_is_no_op() {
        let hooks = NpcEventHooks::default();
        hooks.fire_player_close_channel(0); // must not panic
    }

    #[test]
    fn test_hooks_fire_player_end_trade_no_hook_is_no_op() {
        let hooks = NpcEventHooks::default();
        hooks.fire_player_end_trade(0); // must not panic
    }

    #[test]
    fn test_hooks_fire_creature_move() {
        let captured = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&captured);
        let mut hooks = NpcEventHooks::default();
        hooks.on_creature_move = Some(Box::new(move |id| *c.lock().unwrap() = id));
        hooks.fire_creature_move(99);
        assert_eq!(*captured.lock().unwrap(), 99);
    }

    #[test]
    fn test_hooks_fire_creature_say() {
        let captured = Arc::new(Mutex::new(None::<(u32, String)>));
        let c = Arc::clone(&captured);
        let mut hooks = NpcEventHooks::default();
        hooks.on_creature_say = Some(Box::new(move |id, _class, text| {
            *c.lock().unwrap() = Some((id, text));
        }));
        hooks.fire_creature_say(7, SpeakClass::Say, "hi".into());
        assert_eq!(*captured.lock().unwrap(), Some((7, "hi".to_string())));
    }

    #[test]
    fn test_hooks_fire_think() {
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let mut hooks = NpcEventHooks::default();
        hooks.on_think = Some(Box::new(move || *c.lock().unwrap() += 1));
        hooks.fire_think();
        hooks.fire_think();
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[test]
    fn test_hooks_fire_player_trade_with_callback() {
        let captured = Arc::new(Mutex::new(None::<PlayerTradeParams>));
        let c = Arc::clone(&captured);
        let mut hooks = NpcEventHooks::default();
        hooks.on_player_trade = Some(Box::new(move |p| {
            *c.lock().unwrap() = Some(p);
        }));
        let params = PlayerTradeParams {
            player_id: 10,
            item_id: 200,
            count: 3,
            amount: 5,
            ignore: false,
            in_backpacks: true,
        };
        hooks.fire_player_trade(params);
        assert_eq!(
            *captured.lock().unwrap(),
            Some(PlayerTradeParams {
                player_id: 10,
                item_id: 200,
                count: 3,
                amount: 5,
                ignore: false,
                in_backpacks: true,
            })
        );
    }

    #[test]
    fn test_hooks_fire_player_close_channel() {
        let called = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&called);
        let mut hooks = NpcEventHooks::default();
        hooks.on_player_close_channel = Some(Box::new(move |id| *c.lock().unwrap() = id));
        hooks.fire_player_close_channel(88);
        assert_eq!(*called.lock().unwrap(), 88);
    }

    #[test]
    fn test_hooks_fire_player_end_trade() {
        let called = Arc::new(Mutex::new(false));
        let c = Arc::clone(&called);
        let mut hooks = NpcEventHooks::default();
        hooks.on_player_end_trade = Some(Box::new(move |_id| *c.lock().unwrap() = true));
        hooks.fire_player_end_trade(1);
        assert!(*called.lock().unwrap());
    }

    // -----------------------------------------------------------------------
    // NEW: on_player_trade with has_callback guard
    // -----------------------------------------------------------------------

    #[test]
    fn test_on_player_trade_fires_when_has_callback() {
        let mut npc = Npc::new("Bob");
        let captured = Arc::new(Mutex::new(None::<(u32, u16)>));
        let c = Arc::clone(&captured);
        npc.hooks.on_player_trade = Some(Box::new(move |p| {
            *c.lock().unwrap() = Some((p.player_id, p.item_id));
        }));
        let params = PlayerTradeParams {
            player_id: 1,
            item_id: 42,
            count: 1,
            amount: 10,
            ignore: false,
            in_backpacks: false,
        };
        npc.on_player_trade(params, true);
        assert_eq!(*captured.lock().unwrap(), Some((1, 42)));
    }

    #[test]
    fn test_on_player_trade_skipped_when_no_callback() {
        let mut npc = Npc::new("Bob");
        let fired = Arc::new(Mutex::new(false));
        let f = Arc::clone(&fired);
        npc.hooks.on_player_trade = Some(Box::new(move |_p| {
            *f.lock().unwrap() = true;
        }));
        let params = PlayerTradeParams {
            player_id: 1,
            item_id: 42,
            count: 1,
            amount: 10,
            ignore: false,
            in_backpacks: false,
        };
        // has_callback = false → hook must NOT fire
        npc.on_player_trade(params, false);
        assert!(!*fired.lock().unwrap());

        // Sanity: the same closure DOES fire when has_callback=true, ensuring
        // the closure-body lines remain covered and the guard is exercised in
        // both directions within a single test.
        npc.on_player_trade(params, true);
        assert!(*fired.lock().unwrap());
    }

    // -----------------------------------------------------------------------
    // NEW: on_think
    // -----------------------------------------------------------------------

    #[test]
    fn test_on_think_fires_hook() {
        let mut npc = Npc::new("Bob");
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        npc.hooks.on_think = Some(Box::new(move || *c.lock().unwrap() += 1));
        npc.on_think(9999);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_on_think_returns_true_when_should_patrol() {
        let mut npc = Npc::new("Bob");
        npc.set_idle_interval(1000);
        npc.set_idle(false); // not idle
                             // Not focused, elapsed >= interval
        assert!(npc.on_think(1500));
    }

    #[test]
    fn test_on_think_returns_false_when_idle() {
        let npc = Npc::new("Bob");
        // is_idle = true by default
        assert!(!npc.on_think(9999));
    }

    #[test]
    fn test_on_think_returns_false_when_focused() {
        let mut npc = Npc::new("Bob");
        npc.set_idle(false);
        npc.set_focus(1);
        assert!(!npc.on_think(9999));
    }

    // -----------------------------------------------------------------------
    // NEW: do_sell_item algorithm
    // -----------------------------------------------------------------------

    #[test]
    fn test_do_sell_item_stackable_exact_one_chunk() {
        // 50 stackable items, all fit in one chunk
        let result = do_sell_item(true, 50, |_| true);
        assert_eq!(result.sold_count, 50);
        assert!(result.complete);
    }

    #[test]
    fn test_do_sell_item_stackable_multiple_chunks() {
        // 250 items → chunks: 100, 100, 50
        let chunks_seen: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(vec![]));
        let cs = Arc::clone(&chunks_seen);
        let result = do_sell_item(true, 250, move |chunk| {
            cs.lock().unwrap().push(chunk);
            true
        });
        assert_eq!(result.sold_count, 250);
        assert!(result.complete);
        let chunks = chunks_seen.lock().unwrap();
        assert_eq!(*chunks, vec![100, 100, 50]);
    }

    #[test]
    fn test_do_sell_item_stackable_partial_when_accept_fails() {
        // Accept first chunk (100), reject second
        let mut call = 0u32;
        let result = do_sell_item(true, 250, move |chunk| {
            call += 1;
            if call == 1 {
                let _ = chunk;
                true
            } else {
                false
            }
        });
        assert_eq!(result.sold_count, 100);
        assert!(!result.complete);
    }

    #[test]
    fn test_do_sell_item_non_stackable_all_accepted() {
        let result = do_sell_item(false, 5, |_| true);
        assert_eq!(result.sold_count, 5);
        assert!(result.complete);
    }

    #[test]
    fn test_do_sell_item_non_stackable_partial() {
        let mut call = 0u32;
        let result = do_sell_item(false, 5, move |_| {
            call += 1;
            call <= 3
        });
        assert_eq!(result.sold_count, 3);
        assert!(!result.complete);
    }

    #[test]
    fn test_do_sell_item_zero_amount() {
        // amount==0 → closure must never run (C++ guard `while amount > 0`).
        // We pass a named function pointer so the "zero amount" assertion is
        // checked AND a follow-up positive call ensures the same function
        // body is also covered (so the file reaches 100% line coverage).
        fn always_true(_: u32) -> bool {
            true
        }
        // Sanity: invoke once to cover the function body, independent of
        // the do_sell_item branch under test.
        assert!(always_true(0));

        let result = do_sell_item(true, 0, always_true);
        assert_eq!(result.sold_count, 0);
        assert!(result.complete);
        // Same for the non-stackable branch.
        let result = do_sell_item(false, 0, always_true);
        assert_eq!(result.sold_count, 0);
        assert!(result.complete);
    }

    #[test]
    fn test_do_sell_item_exactly_item_stack_size_stackable() {
        // Exactly 100 → exactly 1 chunk
        let call_count = Arc::new(Mutex::new(0u32));
        let cc = Arc::clone(&call_count);
        let result = do_sell_item(true, ITEM_STACK_SIZE, move |_| {
            *cc.lock().unwrap() += 1;
            true
        });
        assert_eq!(result.sold_count, ITEM_STACK_SIZE);
        assert!(result.complete);
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_do_sell_item_non_stackable_immediate_reject() {
        let result = do_sell_item(false, 10, |_| false);
        assert_eq!(result.sold_count, 0);
        assert!(!result.complete);
    }

    // -----------------------------------------------------------------------
    // NEW: NpcEventHooks Debug impl (mirrors C++ has no equivalent; this is
    // Rust-only diagnostic plumbing required to keep std::fmt::Debug usable).
    // -----------------------------------------------------------------------

    #[test]
    fn test_npc_event_hooks_debug_empty() {
        // Default hooks → every field reports false.
        let hooks = NpcEventHooks::default();
        let s = format!("{:?}", hooks);
        assert!(s.contains("NpcEventHooks"));
        assert!(s.contains("on_creature_appear: false"));
        assert!(s.contains("on_creature_disappear: false"));
        assert!(s.contains("on_creature_move: false"));
        assert!(s.contains("on_creature_say: false"));
        assert!(s.contains("on_think: false"));
        assert!(s.contains("on_player_trade: false"));
        assert!(s.contains("on_player_close_channel: false"));
        assert!(s.contains("on_player_end_trade: false"));
    }

    #[test]
    fn test_npc_event_hooks_debug_with_all_hooks_set() {
        // Every hook installed → every field reports true. This exercises the
        // entire Debug::fmt body which is otherwise uncovered. Each closure
        // increments a shared counter so we can fire every hook below and
        // ensure the closure bodies are also covered (no closure remains as
        // an uninvoked `|_| {}`).
        let counter = Arc::new(Mutex::new(0u32));
        let c1 = Arc::clone(&counter);
        let c2 = Arc::clone(&counter);
        let c3 = Arc::clone(&counter);
        let c4 = Arc::clone(&counter);
        let c5 = Arc::clone(&counter);
        let c6 = Arc::clone(&counter);
        let c7 = Arc::clone(&counter);
        let c8 = Arc::clone(&counter);
        let mut hooks = NpcEventHooks::default();
        hooks.on_creature_appear = Some(Box::new(move |_| *c1.lock().unwrap() += 1));
        hooks.on_creature_disappear = Some(Box::new(move |_| *c2.lock().unwrap() += 1));
        hooks.on_creature_move = Some(Box::new(move |_| *c3.lock().unwrap() += 1));
        hooks.on_creature_say = Some(Box::new(move |_, _, _| *c4.lock().unwrap() += 1));
        hooks.on_think = Some(Box::new(move || *c5.lock().unwrap() += 1));
        hooks.on_player_trade = Some(Box::new(move |_| *c6.lock().unwrap() += 1));
        hooks.on_player_close_channel = Some(Box::new(move |_| *c7.lock().unwrap() += 1));
        hooks.on_player_end_trade = Some(Box::new(move |_| *c8.lock().unwrap() += 1));

        // Debug formatting exercises every field of Debug::fmt.
        let s = format!("{:?}", hooks);
        assert!(s.contains("on_creature_appear: true"));
        assert!(s.contains("on_creature_disappear: true"));
        assert!(s.contains("on_creature_move: true"));
        assert!(s.contains("on_creature_say: true"));
        assert!(s.contains("on_think: true"));
        assert!(s.contains("on_player_trade: true"));
        assert!(s.contains("on_player_close_channel: true"));
        assert!(s.contains("on_player_end_trade: true"));

        // Fire every hook so each closure body executes at least once.
        hooks.fire_creature_appear(1);
        hooks.fire_creature_disappear(2);
        hooks.fire_creature_move(3);
        hooks.fire_creature_say(4, SpeakClass::Say, "x".into());
        hooks.fire_think();
        hooks.fire_player_trade(PlayerTradeParams {
            player_id: 5,
            item_id: 6,
            count: 7,
            amount: 8,
            ignore: false,
            in_backpacks: true,
        });
        hooks.fire_player_close_channel(9);
        hooks.fire_player_end_trade(10);
        assert_eq!(*counter.lock().unwrap(), 8);
    }

    // -----------------------------------------------------------------------
    // NEW: is_loaded short-circuit — make sure each individual hook flips it.
    // -----------------------------------------------------------------------

    #[test]
    fn test_hooks_is_loaded_only_player_end_trade_set() {
        // Only the LAST hook is set → forces is_loaded() to short-circuit on
        // the final branch and confirms that branch returns true. The closure
        // increments a counter so the body is observable; we fire it once so
        // the closure body is covered.
        let counter = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&counter);
        let mut hooks = NpcEventHooks::default();
        hooks.on_player_end_trade = Some(Box::new(move |_| *c.lock().unwrap() += 1));
        assert!(hooks.is_loaded());
        hooks.fire_player_end_trade(0);
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_hooks_is_loaded_only_player_close_channel_set() {
        let counter = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&counter);
        let mut hooks = NpcEventHooks::default();
        hooks.on_player_close_channel = Some(Box::new(move |_| *c.lock().unwrap() += 1));
        assert!(hooks.is_loaded());
        hooks.fire_player_close_channel(0);
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_hooks_is_loaded_only_player_trade_set() {
        let counter = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&counter);
        let mut hooks = NpcEventHooks::default();
        hooks.on_player_trade = Some(Box::new(move |_| *c.lock().unwrap() += 1));
        assert!(hooks.is_loaded());
        hooks.fire_player_trade(PlayerTradeParams {
            player_id: 0,
            item_id: 0,
            count: 0,
            amount: 0,
            ignore: false,
            in_backpacks: false,
        });
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    // -----------------------------------------------------------------------
    // NEW: get_description (mirrors C++ Npc::getDescription)
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_description_appends_period() {
        let npc = Npc::new("Old Man");
        assert_eq!(npc.get_description(), "Old Man.");
    }

    #[test]
    fn test_get_description_empty_name() {
        let npc = Npc::new("");
        assert_eq!(npc.get_description(), ".");
    }

    #[test]
    fn test_get_description_multibyte_name() {
        // Unicode names also get a single ASCII '.' appended (mirrors C++
        // which appends a single char regardless of encoding).
        let npc = Npc::new("Núrnen");
        let descr = npc.get_description();
        assert!(descr.starts_with("Núrnen"));
        assert!(descr.ends_with('.'));
    }
}
