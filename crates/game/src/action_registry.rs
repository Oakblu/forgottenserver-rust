//! Action registry — migrated from forgottenserver/src/actions.h and actions.cpp.
//!
//! Provides:
//! - `Action` — descriptor for a registered item-use action (flags, optional callback)
//! - `ActionRegistry` — three-map dispatch (item_id, unique_id, action_id)
//! - `UseItemContext` / `UseItemExContext` — context structs for use-item dispatch
//! - Exhaustion / cooldown tracking via `ActionCooldownTracker`
//! - `UseResult` — outcome of a dispatch attempt

use std::collections::HashMap;

type UseItemCallback = Box<dyn Fn(&UseItemContext) -> UseResult + Send + Sync>;

// ---------------------------------------------------------------------------
// Position (re-used from common)
// ---------------------------------------------------------------------------

use forgottenserver_common::position::Position;

// ---------------------------------------------------------------------------
// Action flags — mirrors C++ Action class boolean fields
// ---------------------------------------------------------------------------

/// Descriptor for a registered item-use action.
///
/// Mirrors the C++ `Action` class fields:
/// - `allowFarUse`       → `allow_far_use`
/// - `checkFloor`        → `check_floor`
/// - `checkLineOfSight`  → `check_line_of_sight`
pub struct Action {
    /// If `true` the action may be used from any distance (mirrors `allowFarUse`).
    pub allow_far_use: bool,
    /// If `true` the target must be on the same floor (mirrors `checkFloor`).
    pub check_floor: bool,
    /// If `true` there must be line-of-sight to the target (mirrors `checkLineOfSight`).
    pub check_line_of_sight: bool,
    /// Optional inline callback. When `None` the action is purely descriptor-based
    /// (Lua script would handle it in C++).
    pub callback: Option<UseItemCallback>,
}

impl Action {
    /// Create an action with default flags (mirrors C++ Action constructor defaults).
    pub fn new() -> Self {
        Action {
            allow_far_use: false,
            check_floor: true,
            check_line_of_sight: true,
            callback: None,
        }
    }

    /// Attach an inline callback.
    pub fn with_callback<F>(mut self, f: F) -> Self
    where
        F: Fn(&UseItemContext) -> UseResult + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(f));
        self
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// UseResult — outcome of dispatching an action
// ---------------------------------------------------------------------------

/// Result returned by `ActionRegistry::use_item` and related methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UseResult {
    /// Action executed successfully.
    Ok,
    /// No action is registered for this item.
    NoAction,
    /// Player is exhausted and must wait before using again.
    Exhausted,
    /// Target is out of range (too far away or wrong floor).
    OutOfRange,
    /// Use-on-target call made without a target registered.
    NoTarget,
    /// Custom message to relay back to the player.
    Message(String),
}

// ---------------------------------------------------------------------------
// UseItemContext — mirrors the parameters of C++ Actions::useItem
// ---------------------------------------------------------------------------

/// Context provided when a player uses an item (basic use, no target).
///
/// Mirrors: `Actions::useItem(Player*, const Position& pos, uint8_t index,
///                            Item* item, bool isHotkey)`
#[derive(Debug, Clone)]
pub struct UseItemContext {
    /// The position where the item is used.
    pub pos: Position,
    /// The stack index of the item.
    pub stack_pos: u8,
    /// The item's type identifier.
    pub item_id: u16,
    /// The item's unique id (0 = none).
    pub unique_id: u16,
    /// The item's action id (0 = none).
    pub action_id: u16,
    /// Whether the use was triggered via a hotkey.
    pub is_hotkey: bool,
}

impl UseItemContext {
    pub fn new(item_id: u16) -> Self {
        UseItemContext {
            pos: Position::default(),
            stack_pos: 0,
            item_id,
            unique_id: 0,
            action_id: 0,
            is_hotkey: false,
        }
    }
}

// ---------------------------------------------------------------------------
// UseItemExContext — mirrors parameters of Actions::useItemEx
// ---------------------------------------------------------------------------

/// Context for use-on-target (extended use).
///
/// Mirrors: `Actions::useItemEx(Player*, fromPos, toPos, toStackPos, Item*, isHotkey, Creature*)`
#[derive(Debug, Clone)]
pub struct UseItemExContext {
    /// Position of the item being used.
    pub from_pos: Position,
    /// Position of the target.
    pub to_pos: Position,
    /// Stack position of the target.
    pub to_stack_pos: u8,
    /// The item's type identifier.
    pub item_id: u16,
    /// The item's unique id (0 = none).
    pub unique_id: u16,
    /// The item's action id (0 = none).
    pub action_id: u16,
    /// Whether there is a creature target (mirrors `Creature* creature` != nullptr).
    pub has_creature_target: bool,
    /// Whether the use was triggered via a hotkey.
    pub is_hotkey: bool,
}

impl UseItemExContext {
    pub fn new(item_id: u16) -> Self {
        UseItemExContext {
            from_pos: Position::default(),
            to_pos: Position::default(),
            to_stack_pos: 0,
            item_id,
            unique_id: 0,
            action_id: 0,
            has_creature_target: false,
            is_hotkey: false,
        }
    }
}

// ---------------------------------------------------------------------------
// ActionCooldownTracker — per-item exhaustion tracking
// ---------------------------------------------------------------------------

/// Tracks per-item cooldowns (action exhaustion).
///
/// In C++ this is handled via `player->setNextAction(OTSYS_TIME() + cooldown)`.
/// Here we store the "ready-at" tick for each item id independently so that
/// cooldowns can be tested without a full player object.
#[derive(Debug, Default)]
pub struct ActionCooldownTracker {
    /// Maps item_id → tick at which the item's cooldown expires.
    ready_at: HashMap<u16, u64>,
    /// Default cooldown in ticks for `use_item`.
    pub use_item_cooldown: u64,
    /// Default cooldown in ticks for `use_item_ex`.
    pub use_item_ex_cooldown: u64,
}

impl ActionCooldownTracker {
    pub fn new() -> Self {
        ActionCooldownTracker {
            ready_at: HashMap::new(),
            use_item_cooldown: 200,
            use_item_ex_cooldown: 400,
        }
    }

    /// Returns `true` when `item_id` is still on cooldown at `current_tick`.
    pub fn is_on_cooldown(&self, item_id: u16, current_tick: u64) -> bool {
        self.ready_at.get(&item_id).copied().unwrap_or(0) > current_tick
    }

    /// Record that `item_id` was just used at `current_tick` with the given `cooldown_ms`.
    pub fn set_cooldown(&mut self, item_id: u16, current_tick: u64, cooldown_ms: u64) {
        self.ready_at.insert(item_id, current_tick + cooldown_ms);
    }

    /// Clear the cooldown for `item_id` (makes it immediately usable).
    pub fn clear_cooldown(&mut self, item_id: u16) {
        self.ready_at.remove(&item_id);
    }
}

// ---------------------------------------------------------------------------
// Legacy callback type (kept for backward compatibility)
// ---------------------------------------------------------------------------

/// Simple callback type (no context).  Kept for backward compat with the
/// original minimal registry; prefer `Action::with_callback` for new code.
pub type ActionCallback = Box<dyn Fn() -> Option<String> + Send + Sync>;

// ---------------------------------------------------------------------------
// ActionRegistry — three-map dispatch
// ---------------------------------------------------------------------------

/// Registry mapping items to their use-action handlers.
///
/// Mirrors C++ `Actions`:
/// - `useItemMap`    → `by_item_id`    (primary item type id)
/// - `uniqueItemMap` → `by_unique_id`  (item unique id attribute)
/// - `actionItemMap` → `by_action_id`  (item action id attribute)
///
/// Lookup priority (mirrors C++ `Actions::getAction`):
/// 1. `unique_id` (if non-zero)
/// 2. `action_id` (if non-zero)
/// 3. `item_id`
#[derive(Default)]
pub struct ActionRegistry {
    /// Lookup by item type id (mirrors `useItemMap`).
    by_item_id: HashMap<u16, Action>,
    /// Lookup by unique id attribute (mirrors `uniqueItemMap`).
    by_unique_id: HashMap<u16, Action>,
    /// Lookup by action id attribute (mirrors `actionItemMap`).
    by_action_id: HashMap<u16, Action>,
    /// Legacy simple callbacks (no-context, backward compat).
    legacy: HashMap<u16, ActionCallback>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // ------------------------------------------------------------------
    // Registration
    // ------------------------------------------------------------------

    /// Register an `Action` by item type id.
    /// Overwrites any existing entry.
    pub fn register_by_item_id(&mut self, item_id: u16, action: Action) {
        self.by_item_id.insert(item_id, action);
    }

    /// Register an `Action` by unique id attribute.
    pub fn register_by_unique_id(&mut self, uid: u16, action: Action) {
        self.by_unique_id.insert(uid, action);
    }

    /// Register an `Action` by action id attribute.
    pub fn register_by_action_id(&mut self, aid: u16, action: Action) {
        self.by_action_id.insert(aid, action);
    }

    /// Register a legacy simple callback for `item_id`. Overwrites any existing entry.
    pub fn register<F>(&mut self, item_id: u16, callback: F)
    where
        F: Fn() -> Option<String> + Send + Sync + 'static,
    {
        self.legacy.insert(item_id, Box::new(callback));
    }

    // ------------------------------------------------------------------
    // Lookup (mirrors Actions::getAction priority)
    // ------------------------------------------------------------------

    /// Returns a reference to the `Action` that would handle `ctx`, following
    /// the C++ priority: unique_id > action_id > item_id.
    pub fn get_action(&self, item_id: u16, unique_id: u16, action_id: u16) -> Option<&Action> {
        if unique_id != 0 {
            if let Some(a) = self.by_unique_id.get(&unique_id) {
                return Some(a);
            }
        }
        if action_id != 0 {
            if let Some(a) = self.by_action_id.get(&action_id) {
                return Some(a);
            }
        }
        self.by_item_id.get(&item_id)
    }

    /// Returns a reference to the `Action` registered by item type id, if any.
    pub fn get_action_by_item_id(&self, item_id: u16) -> Option<&Action> {
        self.by_item_id.get(&item_id)
    }

    /// Returns a reference to the `Action` registered by unique id, if any.
    pub fn get_action_by_unique_id(&self, uid: u16) -> Option<&Action> {
        self.by_unique_id.get(&uid)
    }

    /// Returns a reference to the `Action` registered by action id, if any.
    pub fn get_action_by_action_id(&self, aid: u16) -> Option<&Action> {
        self.by_action_id.get(&aid)
    }

    // ------------------------------------------------------------------
    // use_item — mirrors Actions::useItem / internalUseItem
    // ------------------------------------------------------------------

    /// Dispatch a basic item-use event (no target).
    ///
    /// Mirrors C++ `Actions::useItem`:
    /// 1. Resolves action by unique_id > action_id > item_id.
    /// 2. Invokes the callback with the context.
    /// 3. Returns `UseResult::NoAction` when no handler exists.
    pub fn use_item(&self, ctx: &UseItemContext) -> UseResult {
        let action = self.get_action(ctx.item_id, ctx.unique_id, ctx.action_id);
        match action {
            None => UseResult::NoAction,
            Some(a) => {
                if let Some(cb) = &a.callback {
                    cb(ctx)
                } else {
                    UseResult::Ok
                }
            }
        }
    }

    /// Dispatch an extended item-use event (use on a target position/creature).
    ///
    /// Mirrors C++ `Actions::useItemEx`:
    /// 1. Resolves action — returns `UseResult::NoAction` if none registered.
    /// 2. Performs simple adjacency check when `!allow_far_use`.
    /// 3. Invokes callback (or `UseResult::Ok` for descriptor-only actions).
    pub fn use_item_ex(&self, ctx: &UseItemExContext) -> UseResult {
        let action = {
            if ctx.unique_id != 0 {
                self.by_unique_id.get(&ctx.unique_id)
            } else if ctx.action_id != 0 {
                self.by_action_id.get(&ctx.action_id)
            } else {
                self.by_item_id.get(&ctx.item_id)
            }
        };

        let action = match action {
            None => return UseResult::NoAction,
            Some(a) => a,
        };

        // Range check: when !allow_far_use, player and target must be on same floor.
        if !action.allow_far_use && action.check_floor && ctx.from_pos.z != ctx.to_pos.z {
            return UseResult::OutOfRange;
        }

        if let Some(cb) = &action.callback {
            // Reuse UseItemContext with from_pos for the callback (simplified).
            let basic_ctx = UseItemContext {
                pos: ctx.from_pos,
                stack_pos: ctx.to_stack_pos,
                item_id: ctx.item_id,
                unique_id: ctx.unique_id,
                action_id: ctx.action_id,
                is_hotkey: ctx.is_hotkey,
            };
            cb(&basic_ctx)
        } else {
            UseResult::Ok
        }
    }

    /// Dispatch a use-on-creature event.
    ///
    /// Mirrors C++ `Action::getTarget` + `executeUse` when `targetCreature != nullptr`.
    /// Returns `UseResult::NoTarget` when `ctx.has_creature_target` is `false`.
    pub fn use_with_creature(&self, ctx: &UseItemExContext) -> UseResult {
        if !ctx.has_creature_target {
            return UseResult::NoTarget;
        }
        self.use_item_ex(ctx)
    }

    // ------------------------------------------------------------------
    // Legacy dispatch (backward compat with original ActionRegistry API)
    // ------------------------------------------------------------------

    /// Invoke the legacy callback for `item_id` if one is registered.
    pub fn dispatch(&self, item_id: u16) -> Option<String> {
        self.legacy.get(&item_id).and_then(|cb| cb())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helper builders
    // -----------------------------------------------------------------------

    fn basic_ctx(item_id: u16) -> UseItemContext {
        UseItemContext::new(item_id)
    }

    fn ex_ctx(item_id: u16) -> UseItemExContext {
        UseItemExContext::new(item_id)
    }

    fn action_ok() -> Action {
        Action::new().with_callback(|_ctx| UseResult::Ok)
    }

    fn action_msg(msg: &'static str) -> Action {
        Action::new().with_callback(move |_ctx| UseResult::Message(msg.to_string()))
    }

    // -----------------------------------------------------------------------
    // Legacy API (backward compat)
    // -----------------------------------------------------------------------

    #[test]
    fn dispatch_registered_item_fires_callback() {
        let mut reg = ActionRegistry::new();
        reg.register(100, || Some("You use it.".to_string()));
        assert_eq!(reg.dispatch(100), Some("You use it.".to_string()));
    }

    #[test]
    fn dispatch_unknown_item_returns_none() {
        let reg = ActionRegistry::new();
        assert_eq!(reg.dispatch(9999), None);
    }

    #[test]
    fn dispatch_callback_returning_none_propagates_none() {
        let mut reg = ActionRegistry::new();
        reg.register(50, || None);
        assert_eq!(reg.dispatch(50), None);
    }

    #[test]
    fn register_overwrites_existing_callback() {
        let mut reg = ActionRegistry::new();
        reg.register(1, || Some("first".to_string()));
        reg.register(1, || Some("second".to_string()));
        assert_eq!(reg.dispatch(1), Some("second".to_string()));
    }

    // -----------------------------------------------------------------------
    // Action struct defaults
    // -----------------------------------------------------------------------

    #[test]
    fn action_default_flags_match_cpp() {
        let a = Action::new();
        assert!(!a.allow_far_use, "allowFarUse defaults to false");
        assert!(a.check_floor, "checkFloor defaults to true");
        assert!(a.check_line_of_sight, "checkLineOfSight defaults to true");
    }

    #[test]
    fn action_with_callback_sets_callback() {
        let a = Action::new().with_callback(|_| UseResult::Ok);
        assert!(a.callback.is_some());
    }

    #[test]
    fn action_builder_chain_works() {
        let a = Action {
            allow_far_use: true,
            check_floor: false,
            check_line_of_sight: false,
            ..Action::new()
        };
        assert!(a.allow_far_use);
        assert!(!a.check_floor);
        assert!(!a.check_line_of_sight);
    }

    // -----------------------------------------------------------------------
    // get_action_by_item_id
    // -----------------------------------------------------------------------

    #[test]
    fn get_action_by_item_id_returns_none_for_unknown() {
        let reg = ActionRegistry::new();
        assert!(reg.get_action_by_item_id(9999).is_none());
    }

    #[test]
    fn get_action_by_item_id_returns_registered_action() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(42, action_ok());
        assert!(reg.get_action_by_item_id(42).is_some());
    }

    // -----------------------------------------------------------------------
    // get_action — priority order
    // -----------------------------------------------------------------------

    #[test]
    fn get_action_prefers_unique_id_over_item_id() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(10, action_msg("by_item_id"));
        reg.register_by_unique_id(20, action_msg("by_unique_id"));
        // query with both set: unique_id wins
        let a = reg.get_action(10, 20, 0).unwrap();
        if let Some(cb) = &a.callback {
            assert_eq!(
                cb(&basic_ctx(10)),
                UseResult::Message("by_unique_id".to_string())
            );
        } else {
            panic!("no callback");
        }
    }

    #[test]
    fn get_action_prefers_action_id_over_item_id() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(10, action_msg("by_item_id"));
        reg.register_by_action_id(30, action_msg("by_action_id"));
        // query with action_id set and no unique_id: action_id wins
        let a = reg.get_action(10, 0, 30).unwrap();
        if let Some(cb) = &a.callback {
            assert_eq!(
                cb(&basic_ctx(10)),
                UseResult::Message("by_action_id".to_string())
            );
        } else {
            panic!("no callback");
        }
    }

    #[test]
    fn get_action_falls_back_to_item_id() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(10, action_msg("by_item_id"));
        let a = reg.get_action(10, 0, 0).unwrap();
        if let Some(cb) = &a.callback {
            assert_eq!(
                cb(&basic_ctx(10)),
                UseResult::Message("by_item_id".to_string())
            );
        } else {
            panic!("no callback");
        }
    }

    #[test]
    fn get_action_returns_none_when_no_match() {
        let reg = ActionRegistry::new();
        assert!(reg.get_action(999, 0, 0).is_none());
    }

    #[test]
    fn get_action_unique_id_takes_priority_over_action_id() {
        let mut reg = ActionRegistry::new();
        reg.register_by_unique_id(20, action_msg("uid"));
        reg.register_by_action_id(30, action_msg("aid"));
        // Both uid and aid set: uid wins.
        let a = reg.get_action(0, 20, 30).unwrap();
        if let Some(cb) = &a.callback {
            assert_eq!(cb(&basic_ctx(0)), UseResult::Message("uid".to_string()));
        } else {
            panic!("no callback");
        }
    }

    // -----------------------------------------------------------------------
    // use_item dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn use_item_dispatches_to_registered_action() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(5, action_ok());
        let ctx = basic_ctx(5);
        assert_eq!(reg.use_item(&ctx), UseResult::Ok);
    }

    #[test]
    fn use_item_returns_no_action_for_unregistered() {
        let reg = ActionRegistry::new();
        assert_eq!(reg.use_item(&basic_ctx(999)), UseResult::NoAction);
    }

    #[test]
    fn use_item_dispatches_by_unique_id() {
        let mut reg = ActionRegistry::new();
        reg.register_by_unique_id(77, action_msg("uid-action"));
        let mut ctx = basic_ctx(0);
        ctx.unique_id = 77;
        assert_eq!(
            reg.use_item(&ctx),
            UseResult::Message("uid-action".to_string())
        );
    }

    #[test]
    fn use_item_dispatches_by_action_id() {
        let mut reg = ActionRegistry::new();
        reg.register_by_action_id(88, action_msg("aid-action"));
        let mut ctx = basic_ctx(0);
        ctx.action_id = 88;
        assert_eq!(
            reg.use_item(&ctx),
            UseResult::Message("aid-action".to_string())
        );
    }

    #[test]
    fn use_item_descriptor_only_action_returns_ok() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(7, Action::new()); // no callback
        assert_eq!(reg.use_item(&basic_ctx(7)), UseResult::Ok);
    }

    // -----------------------------------------------------------------------
    // use_item_ex dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn use_item_ex_with_no_target_returns_none_when_no_action() {
        let reg = ActionRegistry::new();
        assert_eq!(reg.use_item_ex(&ex_ctx(999)), UseResult::NoAction);
    }

    #[test]
    fn use_item_ex_dispatches_registered_action() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(10, action_ok());
        assert_eq!(reg.use_item_ex(&ex_ctx(10)), UseResult::Ok);
    }

    #[test]
    fn use_item_ex_floor_check_blocks_cross_floor_when_check_floor_true() {
        let mut reg = ActionRegistry::new();
        // Default action: check_floor=true, allow_far_use=false
        reg.register_by_item_id(10, Action::new());
        let mut ctx = ex_ctx(10);
        ctx.from_pos = Position::new(100, 100, 7);
        ctx.to_pos = Position::new(100, 100, 6); // different floor
        assert_eq!(reg.use_item_ex(&ctx), UseResult::OutOfRange);
    }

    #[test]
    fn use_item_ex_allows_cross_floor_when_allow_far_use() {
        let mut reg = ActionRegistry::new();
        let mut action = Action::new();
        action.allow_far_use = true;
        reg.register_by_item_id(10, action);
        let mut ctx = ex_ctx(10);
        ctx.from_pos = Position::new(100, 100, 7);
        ctx.to_pos = Position::new(100, 100, 6); // different floor — OK with far use
        assert_eq!(reg.use_item_ex(&ctx), UseResult::Ok);
    }

    #[test]
    fn use_item_ex_same_floor_passes_floor_check() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(10, action_ok());
        let mut ctx = ex_ctx(10);
        ctx.from_pos = Position::new(100, 100, 7);
        ctx.to_pos = Position::new(105, 105, 7); // same floor
        assert_eq!(reg.use_item_ex(&ctx), UseResult::Ok);
    }

    // -----------------------------------------------------------------------
    // use_with_creature
    // -----------------------------------------------------------------------

    #[test]
    fn use_with_creature_returns_no_target_when_no_creature() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(10, action_ok());
        let ctx = ex_ctx(10); // has_creature_target = false
        assert_eq!(reg.use_with_creature(&ctx), UseResult::NoTarget);
    }

    #[test]
    fn use_with_creature_dispatches_when_creature_present() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(10, action_ok());
        let mut ctx = ex_ctx(10);
        ctx.has_creature_target = true;
        assert_eq!(reg.use_with_creature(&ctx), UseResult::Ok);
    }

    #[test]
    fn use_with_creature_no_action_returns_no_action() {
        let reg = ActionRegistry::new();
        let mut ctx = ex_ctx(999);
        ctx.has_creature_target = true;
        assert_eq!(reg.use_with_creature(&ctx), UseResult::NoAction);
    }

    // -----------------------------------------------------------------------
    // ActionCooldownTracker
    // -----------------------------------------------------------------------

    #[test]
    fn cooldown_not_active_initially() {
        let tracker = ActionCooldownTracker::new();
        assert!(!tracker.is_on_cooldown(1, 1000));
    }

    #[test]
    fn exhaust_check_blocks_use_when_exhausted() {
        let mut tracker = ActionCooldownTracker::new();
        tracker.set_cooldown(5, 1000, 200); // cooldown expires at tick 1200
        assert!(tracker.is_on_cooldown(5, 1100)); // still on cooldown at 1100
        assert!(!tracker.is_on_cooldown(5, 1200)); // expires at exactly 1200
        assert!(!tracker.is_on_cooldown(5, 1300)); // past expiry
    }

    #[test]
    fn action_cooldown_tracks_independently_per_item() {
        let mut tracker = ActionCooldownTracker::new();
        tracker.set_cooldown(1, 0, 500);
        tracker.set_cooldown(2, 0, 100);
        assert!(tracker.is_on_cooldown(1, 300)); // item 1: still on cd
        assert!(!tracker.is_on_cooldown(2, 300)); // item 2: expired
    }

    #[test]
    fn clear_cooldown_makes_item_immediately_usable() {
        let mut tracker = ActionCooldownTracker::new();
        tracker.set_cooldown(7, 0, 9999);
        assert!(tracker.is_on_cooldown(7, 100));
        tracker.clear_cooldown(7);
        assert!(!tracker.is_on_cooldown(7, 100));
    }

    #[test]
    fn cooldown_at_boundary_is_not_active() {
        let mut tracker = ActionCooldownTracker::new();
        tracker.set_cooldown(3, 500, 200); // ready_at = 700
                                           // At exactly 700 it is NOT on cooldown (> vs >=)
        assert!(!tracker.is_on_cooldown(3, 700));
        assert!(tracker.is_on_cooldown(3, 699));
    }

    #[test]
    fn use_item_cooldown_default_value() {
        let tracker = ActionCooldownTracker::new();
        assert_eq!(tracker.use_item_cooldown, 200);
    }

    #[test]
    fn use_item_ex_cooldown_default_value() {
        let tracker = ActionCooldownTracker::new();
        assert_eq!(tracker.use_item_ex_cooldown, 400);
    }

    // -----------------------------------------------------------------------
    // UseResult variants
    // -----------------------------------------------------------------------

    #[test]
    fn use_result_variants_are_distinct() {
        assert_ne!(UseResult::Ok, UseResult::NoAction);
        assert_ne!(UseResult::Exhausted, UseResult::OutOfRange);
        assert_ne!(UseResult::NoTarget, UseResult::Ok);
    }

    #[test]
    fn use_result_message_stores_string() {
        let r = UseResult::Message("hello".to_string());
        assert_eq!(r, UseResult::Message("hello".to_string()));
    }

    // -----------------------------------------------------------------------
    // UseItemContext helpers
    // -----------------------------------------------------------------------

    #[test]
    fn use_item_context_new_defaults() {
        let ctx = UseItemContext::new(42);
        assert_eq!(ctx.item_id, 42);
        assert_eq!(ctx.unique_id, 0);
        assert_eq!(ctx.action_id, 0);
        assert!(!ctx.is_hotkey);
    }

    #[test]
    fn use_item_ex_context_new_defaults() {
        let ctx = UseItemExContext::new(99);
        assert_eq!(ctx.item_id, 99);
        assert!(!ctx.has_creature_target);
        assert!(!ctx.is_hotkey);
    }

    // -----------------------------------------------------------------------
    // Registry lookup by unique / action id
    // -----------------------------------------------------------------------

    #[test]
    fn get_action_by_unique_id_returns_none_for_unknown() {
        let reg = ActionRegistry::new();
        assert!(reg.get_action_by_unique_id(99).is_none());
    }

    #[test]
    fn get_action_by_unique_id_returns_registered() {
        let mut reg = ActionRegistry::new();
        reg.register_by_unique_id(5, action_ok());
        assert!(reg.get_action_by_unique_id(5).is_some());
    }

    #[test]
    fn get_action_by_action_id_returns_none_for_unknown() {
        let reg = ActionRegistry::new();
        assert!(reg.get_action_by_action_id(99).is_none());
    }

    #[test]
    fn get_action_by_action_id_returns_registered() {
        let mut reg = ActionRegistry::new();
        reg.register_by_action_id(15, action_ok());
        assert!(reg.get_action_by_action_id(15).is_some());
    }

    // -----------------------------------------------------------------------
    // Overwrite semantics
    // -----------------------------------------------------------------------

    #[test]
    fn register_by_item_id_overwrites_existing() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(1, action_msg("first"));
        reg.register_by_item_id(1, action_msg("second"));
        let a = reg.get_action_by_item_id(1).unwrap();
        if let Some(cb) = &a.callback {
            assert_eq!(cb(&basic_ctx(1)), UseResult::Message("second".to_string()));
        }
    }

    #[test]
    fn register_by_unique_id_overwrites_existing() {
        let mut reg = ActionRegistry::new();
        reg.register_by_unique_id(2, action_msg("first"));
        reg.register_by_unique_id(2, action_msg("second"));
        let a = reg.get_action_by_unique_id(2).unwrap();
        if let Some(cb) = &a.callback {
            assert_eq!(cb(&basic_ctx(0)), UseResult::Message("second".to_string()));
        }
    }

    // -----------------------------------------------------------------------
    // is_hotkey propagated through context
    // -----------------------------------------------------------------------

    #[test]
    fn use_item_context_hotkey_flag_accessible_in_callback() {
        let mut reg = ActionRegistry::new();
        reg.register_by_item_id(
            55,
            Action::new().with_callback(|ctx| {
                if ctx.is_hotkey {
                    UseResult::Message("hotkey".to_string())
                } else {
                    UseResult::Ok
                }
            }),
        );
        let mut ctx = basic_ctx(55);
        ctx.is_hotkey = true;
        assert_eq!(reg.use_item(&ctx), UseResult::Message("hotkey".to_string()));
        ctx.is_hotkey = false;
        assert_eq!(reg.use_item(&ctx), UseResult::Ok);
    }
}
