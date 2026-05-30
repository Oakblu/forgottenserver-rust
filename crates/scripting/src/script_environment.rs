// Migrated from forgottenserver/src/luascript.h + luascript.cpp
//
// ScriptEnvironment — per-callback execution state for Lua scripts.
// Mirrors the C++ `LuaScriptInterface::ScriptEnvironment` class.
//
// Responsibilities:
//   * Track which script and callback are currently executing.
//   * Maintain a UID table for `Thing`/`Item`/`Container` handles so Lua
//     scripts can resolve `Player:addItem(uid)`, `Player:getStorageValue(key)`,
//     etc. against server-side objects.
//   * Recycle UIDs after use (released_uids pool).
//   * Provide an NPC binding slot for callbacks that target a specific NPC.
//   * Record timer-event IDs so timed Lua callbacks can be tracked and
//     cancelled.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use forgottenserver_common::thing::Thing;

// ---------------------------------------------------------------------------
// ThingHandle — opaque server-side reference stored by UID
// ---------------------------------------------------------------------------

/// Discriminator for what kind of object a UID points at.
///
/// Mirrors the C++ unions of `Thing*` / `Item*` / `Container*` resolved by
/// `getThingByUID` / `getItemByUID` / `getContainerByUID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleKind {
    /// A bare `Thing*` (creature, ground, etc).
    Thing,
    /// An `Item*` (also countable as a Thing).
    Item,
    /// A `Container*` (also countable as an Item and a Thing).
    Container,
}

/// A single entry in the UID table. Stores a boxed `dyn Thing` plus a
/// discriminator so the right Lua-side getter can succeed.
pub struct ThingHandle {
    pub kind: HandleKind,
    pub thing: Box<dyn Thing + Send + Sync>,
}

impl std::fmt::Debug for ThingHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThingHandle")
            .field("kind", &self.kind)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// TimerEvent — minimal record of a scheduled Lua callback
// ---------------------------------------------------------------------------

/// Per the C++ `TimerEventDesc`, a record of a scheduled callback. `function_ref`
/// is the Lua registry index for the callback function; `parameters` is a list
/// of integer parameters (mirrors C++ `std::vector<int32_t>`).
#[derive(Debug, Clone)]
pub struct TimerEventDesc {
    pub function_ref: i32,
    pub script_id: i32,
    pub script_name: String,
    pub parameters: Vec<i32>,
}

// ---------------------------------------------------------------------------
// ScriptEnvironment
// ---------------------------------------------------------------------------

/// Per-callback execution state for Lua scripts.
///
/// Mirrors C++ `LuaScriptInterface::ScriptEnvironment`. Reset between
/// callbacks via [`reset_env`].
pub struct ScriptEnvironment {
    /// The Lua registry-index of the script being executed
    /// (mirrors C++ `scriptId`).
    current_script_id: i32,
    /// The callback being executed (mirrors C++ `callbackId`).
    current_callback_id: i32,
    /// The NPC handle for the current callback, if any
    /// (mirrors C++ `curNpc`). Stored as an opaque ID; Lua resolves it
    /// via `getNpc`.
    current_npc: Option<u32>,
    /// The timer event currently being processed
    /// (mirrors C++ `eventInfo`).
    current_timer_event_id: u32,

    /// UID → handle table.
    thing_uid_table: HashMap<u32, ThingHandle>,
    /// Pool of recyclable UIDs (mirrors C++ `auto_id`/released-id pattern).
    released_uids: VecDeque<u32>,

    /// Next UID to allocate when the released pool is empty
    /// (mirrors C++ `lastUID` cursor; starts at 0x10000000 to avoid
    /// collisions with map-loaded UIDs which live in `[0x100, 0xFFFE]`).
    next_uid: u32,

    /// Map of timer event id → desc, used to resolve `setTimerEvent`
    /// callbacks (mirrors C++ `timerEvents`).
    timer_events: HashMap<u32, TimerEventDesc>,
    /// Next timer event id to allocate.
    next_timer_event_id: u32,
}

impl Default for ScriptEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ScriptEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptEnvironment")
            .field("current_script_id", &self.current_script_id)
            .field("current_callback_id", &self.current_callback_id)
            .field("current_npc", &self.current_npc)
            .field("current_timer_event_id", &self.current_timer_event_id)
            .field("uid_table_len", &self.thing_uid_table.len())
            .field("released_uids", &self.released_uids.len())
            .field("next_uid", &self.next_uid)
            .field("timer_events", &self.timer_events.len())
            .finish()
    }
}

impl ScriptEnvironment {
    /// UID-range start for dynamically-allocated handles. The C++ code
    /// uses `0x10000000` as the boundary between map-loaded UIDs
    /// (`[0x100, 0xFFFE]`) and runtime-allocated UIDs.
    pub const DYNAMIC_UID_START: u32 = 0x1000_0000;

    /// Reserved sentinel meaning "no callback active"
    /// (matches C++ `callbackId == 0`).
    pub const NO_CALLBACK: i32 = 0;

    /// Reserved sentinel meaning "no script active".
    pub const NO_SCRIPT: i32 = 0;

    /// Construct a fresh environment ready for first use.
    pub fn new() -> Self {
        ScriptEnvironment {
            current_script_id: Self::NO_SCRIPT,
            current_callback_id: Self::NO_CALLBACK,
            current_npc: None,
            current_timer_event_id: 0,
            thing_uid_table: HashMap::new(),
            released_uids: VecDeque::new(),
            next_uid: Self::DYNAMIC_UID_START,
            timer_events: HashMap::new(),
            next_timer_event_id: 1,
        }
    }

    // -----------------------------------------------------------------------
    // Script / callback IDs
    // -----------------------------------------------------------------------

    /// Set the currently-executing script id. Mirrors C++
    /// `setScriptId(scriptId)`.
    pub fn set_script_id(&mut self, script_id: i32) {
        self.current_script_id = script_id;
    }

    /// Get the currently-executing script id. Mirrors C++ `getScriptId()`.
    pub fn get_script_id(&self) -> i32 {
        self.current_script_id
    }

    /// Set the currently-executing callback id. Mirrors C++
    /// `setCallbackId(callbackId)`.
    pub fn set_callback_id(&mut self, callback_id: i32) {
        self.current_callback_id = callback_id;
    }

    /// Get the currently-executing callback id. Mirrors C++
    /// `getCallbackId()`.
    pub fn get_callback_id(&self) -> i32 {
        self.current_callback_id
    }

    // -----------------------------------------------------------------------
    // NPC binding
    // -----------------------------------------------------------------------

    /// Bind an NPC handle to the current callback. Mirrors C++ `setNpc(Npc*)`.
    /// Passing `None` clears the binding.
    pub fn set_npc(&mut self, npc: Option<u32>) {
        self.current_npc = npc;
    }

    /// Returns the NPC bound to the current callback, if any.
    pub fn get_npc(&self) -> Option<u32> {
        self.current_npc
    }

    // -----------------------------------------------------------------------
    // Timer events
    // -----------------------------------------------------------------------

    /// Insert a timer-event descriptor and return the new event id.
    /// Mirrors C++ `addTimerEvent(TimerEventDesc&&)`.
    pub fn add_timer_event(&mut self, desc: TimerEventDesc) -> u32 {
        let id = self.next_timer_event_id;
        self.next_timer_event_id = self.next_timer_event_id.saturating_add(1);
        self.timer_events.insert(id, desc);
        id
    }

    /// Fetch a timer-event descriptor by id. Mirrors C++ `getTimerEvent(id)`.
    pub fn get_timer_event(&self, id: u32) -> Option<&TimerEventDesc> {
        self.timer_events.get(&id)
    }

    /// Remove a timer-event descriptor, returning it. Mirrors the cancel /
    /// fire path in C++ that erases the entry after execution.
    pub fn remove_timer_event(&mut self, id: u32) -> Option<TimerEventDesc> {
        self.timer_events.remove(&id)
    }

    /// Set the currently-executing timer event id (for nested-callback
    /// resolution).
    pub fn set_timer_event(&mut self, id: u32) {
        self.current_timer_event_id = id;
    }

    /// Get the currently-executing timer event id.
    pub fn get_event_info(&self) -> u32 {
        self.current_timer_event_id
    }

    // -----------------------------------------------------------------------
    // UID allocation & resolution
    // -----------------------------------------------------------------------

    /// Allocate a UID for a new handle, preferring recycled ids from the
    /// released pool. Mirrors the C++ `auto_id` allocation.
    fn allocate_uid(&mut self) -> u32 {
        if let Some(uid) = self.released_uids.pop_front() {
            return uid;
        }
        let uid = self.next_uid;
        self.next_uid = self.next_uid.saturating_add(1);
        uid
    }

    /// Insert a handle and return its UID. Mirrors C++ `addThing(Thing*)`
    /// which auto-allocates a UID and stores the handle.
    pub fn add_thing(&mut self, kind: HandleKind, thing: Box<dyn Thing + Send + Sync>) -> u32 {
        let uid = self.allocate_uid();
        self.thing_uid_table
            .insert(uid, ThingHandle { kind, thing });
        uid
    }

    /// Insert a handle at a specific UID. Mirrors C++ `insertItem(uid, item)`
    /// (used during map loading to register pre-assigned UIDs).
    ///
    /// Returns `false` when the UID is already in use.
    pub fn insert_item(
        &mut self,
        uid: u32,
        kind: HandleKind,
        thing: Box<dyn Thing + Send + Sync>,
    ) -> bool {
        if self.thing_uid_table.contains_key(&uid) {
            return false;
        }
        self.thing_uid_table
            .insert(uid, ThingHandle { kind, thing });
        true
    }

    /// Add an item assumed to be a temporary container/item (matches C++
    /// `addTempItem(item)`). Returns the assigned UID.
    pub fn add_temp_item(&mut self, kind: HandleKind, thing: Box<dyn Thing + Send + Sync>) -> u32 {
        self.add_thing(kind, thing)
    }

    /// Resolve a handle by UID for a generic `Thing` lookup.
    /// Mirrors C++ `getThingByUID(uid)`.
    pub fn get_thing_by_uid(&self, uid: u32) -> Option<&(dyn Thing + Send + Sync)> {
        self.thing_uid_table.get(&uid).map(|h| &*h.thing)
    }

    /// Resolve a handle by UID, but only if the stored kind is at least an
    /// Item. Mirrors C++ `getItemByUID(uid)`.
    pub fn get_item_by_uid(&self, uid: u32) -> Option<&(dyn Thing + Send + Sync)> {
        self.thing_uid_table.get(&uid).and_then(|h| match h.kind {
            HandleKind::Item | HandleKind::Container => Some(&*h.thing),
            HandleKind::Thing => None,
        })
    }

    /// Resolve a handle by UID, but only if the stored kind is a Container.
    /// Mirrors C++ `getContainerByUID(uid)`.
    pub fn get_container_by_uid(&self, uid: u32) -> Option<&(dyn Thing + Send + Sync)> {
        self.thing_uid_table.get(&uid).and_then(|h| match h.kind {
            HandleKind::Container => Some(&*h.thing),
            _ => None,
        })
    }

    /// Remove and recycle an item UID. Mirrors C++ `removeItemByUID(uid)`.
    /// Returns `true` when the UID existed and was removed.
    pub fn remove_item_by_uid(&mut self, uid: u32) -> bool {
        if self.thing_uid_table.remove(&uid).is_some() {
            self.released_uids.push_back(uid);
            true
        } else {
            false
        }
    }

    /// Number of currently-allocated UIDs (testing helper; not in C++).
    pub fn uid_table_len(&self) -> usize {
        self.thing_uid_table.len()
    }

    // -----------------------------------------------------------------------
    // Reset / lifecycle
    // -----------------------------------------------------------------------

    /// Reset all per-callback fields. Mirrors C++ `resetEnv()` which is
    /// called at the end of each Lua callback.
    pub fn reset_env(&mut self) {
        self.current_script_id = Self::NO_SCRIPT;
        self.current_callback_id = Self::NO_CALLBACK;
        self.current_npc = None;
        self.current_timer_event_id = 0;
        // Note: thing_uid_table, released_uids, next_uid, and timer_events
        // intentionally persist across callbacks — matches C++ semantics
        // (handles outlive a single callback).
    }
}

// ---------------------------------------------------------------------------
// Thread-local single instance (mirrors C++ `static ScriptEnvironment env;`)
//
// The C++ code uses a single static ScriptEnvironment per Lua state. In Rust
// we expose a thread-local so unit tests can isolate state per thread.
// ---------------------------------------------------------------------------

thread_local! {
    static SCRIPT_ENV: RefCell<ScriptEnvironment> = RefCell::new(ScriptEnvironment::new());
}

/// Borrow the thread-local ScriptEnvironment immutably for the duration of `f`.
pub fn with_env<R>(f: impl FnOnce(&ScriptEnvironment) -> R) -> R {
    SCRIPT_ENV.with(|cell| f(&cell.borrow()))
}

/// Borrow the thread-local ScriptEnvironment mutably for the duration of `f`.
pub fn with_env_mut<R>(f: impl FnOnce(&mut ScriptEnvironment) -> R) -> R {
    SCRIPT_ENV.with(|cell| f(&mut cell.borrow_mut()))
}

// ---------------------------------------------------------------------------
// Script-environment stack
//
// Mirrors C++ `static std::array<ScriptEnvironment, 16> scriptEnv` and
// `static int32_t scriptEnvIndex = -1` in luascript.cpp.
//
// `reserve_script_env()` increments the index and returns true while there
// is still capacity (index < 16).  `reset_script_env()` resets the current
// slot and decrements the index.  This mirrors:
//
//   bool tfs::lua::reserveScriptEnv() {
//       return ++scriptEnvIndex < static_cast<int32_t>(scriptEnv.size());
//   }
//   void tfs::lua::resetScriptEnv() {
//       assert(scriptEnvIndex >= 0);
//       scriptEnv[scriptEnvIndex--].resetEnv();
//   }
// ---------------------------------------------------------------------------

/// Maximum nesting depth of Lua callbacks.  Mirrors the C++ array size of 16.
pub const SCRIPT_ENV_CAPACITY: i32 = 16;

thread_local! {
    /// Stack of per-callback ScriptEnvironment slots.  Mirrors C++
    /// `static std::array<ScriptEnvironment, 16> scriptEnv`.
    static SCRIPT_ENV_STACK: RefCell<Vec<ScriptEnvironment>> =
        RefCell::new(Vec::with_capacity(SCRIPT_ENV_CAPACITY as usize));

    /// Index of the currently-active slot, starting at -1 when idle.
    /// Mirrors C++ `static int32_t scriptEnvIndex = -1`.
    static SCRIPT_ENV_INDEX: RefCell<i32> = const { RefCell::new(-1) };
}

/// Reserve the next slot in the script-environment stack.
///
/// Mirrors `tfs::lua::reserveScriptEnv()`: increments the index and returns
/// `true` while there is still capacity.  Returns `false` if the call stack
/// would overflow the 16-slot limit (matches the C++ overflow check).
pub fn reserve_script_env() -> bool {
    SCRIPT_ENV_INDEX.with(|idx_cell| {
        let mut idx = idx_cell.borrow_mut();
        *idx += 1;
        let new_idx = *idx;
        if new_idx < SCRIPT_ENV_CAPACITY {
            SCRIPT_ENV_STACK.with(|stack| {
                let mut s = stack.borrow_mut();
                if new_idx as usize >= s.len() {
                    s.push(ScriptEnvironment::new());
                }
            });
            true
        } else {
            false
        }
    })
}

/// Reset the current slot and pop the stack.
///
/// Mirrors `tfs::lua::resetScriptEnv()`: calls `resetEnv()` on the active
/// slot, then decrements the index.  Panics (like the C++ `assert`) if
/// called when the stack is already empty (index < 0).
pub fn reset_script_env() {
    SCRIPT_ENV_INDEX.with(|idx_cell| {
        let mut idx = idx_cell.borrow_mut();
        assert!(
            *idx >= 0,
            "reset_script_env called with empty stack (mirrors C++ assert(scriptEnvIndex >= 0))"
        );
        let cur_idx = *idx as usize;
        SCRIPT_ENV_STACK.with(|stack| {
            let mut s = stack.borrow_mut();
            if let Some(env) = s.get_mut(cur_idx) {
                env.reset_env();
            }
        });
        *idx -= 1;
    });
}

/// Return the active script-environment index.  Starts at -1 (idle).
/// Mirrors C++ `scriptEnvIndex`.
pub fn script_env_index() -> i32 {
    SCRIPT_ENV_INDEX.with(|c| *c.borrow())
}

/// Get a copy of the active slot's script id.  Panics if called when
/// the stack is empty (mirrors C++ assert in `getScriptEnv`).
pub fn get_script_env_script_id() -> i32 {
    SCRIPT_ENV_INDEX.with(|idx_cell| {
        let idx = *idx_cell.borrow();
        assert!(
            idx >= 0,
            "get_script_env_script_id called with empty stack"
        );
        SCRIPT_ENV_STACK.with(|stack| {
            stack
                .borrow()
                .get(idx as usize)
                .map(|e| e.get_script_id())
                .unwrap_or(ScriptEnvironment::NO_SCRIPT)
        })
    })
}

// ---------------------------------------------------------------------------
// Lua-callback bridge helpers
//
// These functions show how a Lua callback would resolve a UID supplied by
// a script (e.g. `Player:addItem(uid)`) against the thread-local
// ScriptEnvironment. They are intentionally small wrappers so that hosting
// Lua bindings can simply call them in their FFI shim.
// ---------------------------------------------------------------------------

/// Resolve a UID to a generic Thing, returning `true` when found.
///
/// The C++ `LuaScriptInterface::getThing(L, uid)` Lua binding pattern
/// translates to this Rust helper: a Lua function pushes a UID, this helper
/// checks the env's UID table, and the binding returns success/failure.
pub fn lua_has_thing(uid: u32) -> bool {
    with_env(|env| env.get_thing_by_uid(uid).is_some())
}

/// Resolve a UID to an Item (kind ∈ {Item, Container}).
pub fn lua_has_item(uid: u32) -> bool {
    with_env(|env| env.get_item_by_uid(uid).is_some())
}

/// Resolve a UID to a Container.
pub fn lua_has_container(uid: u32) -> bool {
    with_env(|env| env.get_container_by_uid(uid).is_some())
}

/// Register a Thing handle and return its UID — the bridge a callback uses
/// to expose newly-created server-side objects to a running Lua script.
pub fn lua_register_thing(kind: HandleKind, thing: Box<dyn Thing + Send + Sync>) -> u32 {
    with_env_mut(|env| env.add_thing(kind, thing))
}

/// Release a UID so its handle can be reclaimed; the script no longer
/// references the object. Returns whether the UID was actually present.
pub fn lua_release_uid(uid: u32) -> bool {
    with_env_mut(|env| env.remove_item_by_uid(uid))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::thing::DefaultThing;

    fn fresh_env() -> ScriptEnvironment {
        ScriptEnvironment::new()
    }

    fn boxed_default() -> Box<dyn Thing + Send + Sync> {
        Box::new(DefaultThing)
    }

    // --- Script / callback IDs ---

    #[test]
    fn test_new_env_starts_with_no_script_or_callback() {
        let env = fresh_env();
        assert_eq!(env.get_script_id(), ScriptEnvironment::NO_SCRIPT);
        assert_eq!(env.get_callback_id(), ScriptEnvironment::NO_CALLBACK);
        assert!(env.get_npc().is_none());
        assert_eq!(env.get_event_info(), 0);
    }

    #[test]
    fn test_set_script_id_persists() {
        let mut env = fresh_env();
        env.set_script_id(42);
        assert_eq!(env.get_script_id(), 42);
    }

    #[test]
    fn test_set_callback_id_persists() {
        let mut env = fresh_env();
        env.set_callback_id(7);
        assert_eq!(env.get_callback_id(), 7);
    }

    // --- NPC binding ---

    #[test]
    fn test_set_npc_some_then_none() {
        let mut env = fresh_env();
        env.set_npc(Some(123));
        assert_eq!(env.get_npc(), Some(123));
        env.set_npc(None);
        assert!(env.get_npc().is_none());
    }

    // --- Timer events ---

    #[test]
    fn test_add_timer_event_returns_unique_ids() {
        let mut env = fresh_env();
        let id1 = env.add_timer_event(TimerEventDesc {
            function_ref: 1,
            script_id: 1,
            script_name: "scriptA".into(),
            parameters: vec![],
        });
        let id2 = env.add_timer_event(TimerEventDesc {
            function_ref: 2,
            script_id: 1,
            script_name: "scriptA".into(),
            parameters: vec![10, 20],
        });
        assert_ne!(id1, id2);
        assert!(env.get_timer_event(id1).is_some());
        assert!(env.get_timer_event(id2).is_some());
    }

    #[test]
    fn test_remove_timer_event_returns_desc() {
        let mut env = fresh_env();
        let id = env.add_timer_event(TimerEventDesc {
            function_ref: 1,
            script_id: 1,
            script_name: "scriptA".into(),
            parameters: vec![5],
        });
        let desc = env.remove_timer_event(id).expect("desc");
        assert_eq!(desc.function_ref, 1);
        assert_eq!(desc.parameters, vec![5]);
        // Second remove returns None.
        assert!(env.remove_timer_event(id).is_none());
    }

    #[test]
    fn test_set_timer_event_and_get_event_info() {
        let mut env = fresh_env();
        env.set_timer_event(99);
        assert_eq!(env.get_event_info(), 99);
    }

    // --- UID allocation & resolution ---

    #[test]
    fn test_add_thing_returns_unique_uids() {
        let mut env = fresh_env();
        let uid1 = env.add_thing(HandleKind::Thing, boxed_default());
        let uid2 = env.add_thing(HandleKind::Thing, boxed_default());
        assert_ne!(uid1, uid2);
        assert!(uid1 >= ScriptEnvironment::DYNAMIC_UID_START);
        assert_eq!(env.uid_table_len(), 2);
    }

    #[test]
    fn test_get_thing_by_uid_resolves_any_kind() {
        let mut env = fresh_env();
        let uid_t = env.add_thing(HandleKind::Thing, boxed_default());
        let uid_i = env.add_thing(HandleKind::Item, boxed_default());
        let uid_c = env.add_thing(HandleKind::Container, boxed_default());
        assert!(env.get_thing_by_uid(uid_t).is_some());
        assert!(env.get_thing_by_uid(uid_i).is_some());
        assert!(env.get_thing_by_uid(uid_c).is_some());
        assert!(env.get_thing_by_uid(0xDEAD).is_none());
    }

    #[test]
    fn test_get_item_by_uid_filters_thing_kind() {
        let mut env = fresh_env();
        let uid_t = env.add_thing(HandleKind::Thing, boxed_default());
        let uid_i = env.add_thing(HandleKind::Item, boxed_default());
        let uid_c = env.add_thing(HandleKind::Container, boxed_default());
        assert!(env.get_item_by_uid(uid_t).is_none());
        assert!(env.get_item_by_uid(uid_i).is_some());
        assert!(env.get_item_by_uid(uid_c).is_some());
    }

    #[test]
    fn test_get_container_by_uid_filters_to_container_only() {
        let mut env = fresh_env();
        let uid_t = env.add_thing(HandleKind::Thing, boxed_default());
        let uid_i = env.add_thing(HandleKind::Item, boxed_default());
        let uid_c = env.add_thing(HandleKind::Container, boxed_default());
        assert!(env.get_container_by_uid(uid_t).is_none());
        assert!(env.get_container_by_uid(uid_i).is_none());
        assert!(env.get_container_by_uid(uid_c).is_some());
    }

    #[test]
    fn test_insert_item_at_specific_uid() {
        let mut env = fresh_env();
        assert!(env.insert_item(1234, HandleKind::Item, boxed_default()));
        assert!(env.get_item_by_uid(1234).is_some());
        // Duplicate UID rejected.
        assert!(!env.insert_item(1234, HandleKind::Item, boxed_default()));
    }

    #[test]
    fn test_remove_item_by_uid_recycles_uid() {
        let mut env = fresh_env();
        let uid = env.add_thing(HandleKind::Item, boxed_default());
        assert_eq!(env.uid_table_len(), 1);
        assert!(env.remove_item_by_uid(uid));
        assert_eq!(env.uid_table_len(), 0);
        assert!(env.get_thing_by_uid(uid).is_none());
        // Next allocation reuses the released uid.
        let reused = env.add_thing(HandleKind::Item, boxed_default());
        assert_eq!(reused, uid, "released uid should be recycled first");
    }

    #[test]
    fn test_remove_item_by_uid_returns_false_for_unknown() {
        let mut env = fresh_env();
        assert!(!env.remove_item_by_uid(0xDEAD));
    }

    #[test]
    fn test_add_temp_item_uses_dynamic_uid_range() {
        let mut env = fresh_env();
        let uid = env.add_temp_item(HandleKind::Item, boxed_default());
        assert!(uid >= ScriptEnvironment::DYNAMIC_UID_START);
    }

    // --- reset_env ---

    #[test]
    fn test_reset_env_clears_callback_state_but_preserves_handles() {
        let mut env = fresh_env();
        env.set_script_id(5);
        env.set_callback_id(7);
        env.set_npc(Some(42));
        env.set_timer_event(100);
        let uid = env.add_thing(HandleKind::Item, boxed_default());

        env.reset_env();

        assert_eq!(env.get_script_id(), ScriptEnvironment::NO_SCRIPT);
        assert_eq!(env.get_callback_id(), ScriptEnvironment::NO_CALLBACK);
        assert!(env.get_npc().is_none());
        assert_eq!(env.get_event_info(), 0);
        // Handle table persists.
        assert!(env.get_item_by_uid(uid).is_some());
    }

    // --- Default impl ---

    #[test]
    fn test_default_env_matches_new() {
        let a = ScriptEnvironment::default();
        let b = ScriptEnvironment::new();
        assert_eq!(a.get_script_id(), b.get_script_id());
        assert_eq!(a.get_callback_id(), b.get_callback_id());
        assert_eq!(a.uid_table_len(), b.uid_table_len());
    }

    // --- Debug impl smoke test ---

    #[test]
    fn test_debug_impl_renders_without_panic() {
        let env = fresh_env();
        let s = format!("{:?}", env);
        assert!(s.contains("ScriptEnvironment"));
    }

    #[test]
    fn test_thing_handle_debug_renders() {
        let h = ThingHandle {
            kind: HandleKind::Container,
            thing: boxed_default(),
        };
        let s = format!("{:?}", h);
        assert!(s.contains("Container"));
    }

    // --- with_env / with_env_mut ---

    #[test]
    fn test_with_env_mut_can_set_and_with_env_can_read() {
        with_env_mut(|env| {
            env.set_script_id(999);
        });
        with_env(|env| {
            assert_eq!(env.get_script_id(), 999);
        });
        // Cleanup so the thread-local doesn't leak state to other tests.
        with_env_mut(|env| env.reset_env());
    }

    // --- Lua callback bridge (C.2) ---

    #[test]
    fn test_lua_register_thing_assigns_dynamic_uid() {
        // Fresh thread-local in this test (use a closure to reset on exit).
        with_env_mut(|env| {
            *env = ScriptEnvironment::new();
        });
        let uid = lua_register_thing(HandleKind::Item, boxed_default());
        assert!(uid >= ScriptEnvironment::DYNAMIC_UID_START);
        assert!(lua_has_thing(uid));
        assert!(lua_has_item(uid));
        assert!(!lua_has_container(uid));
        // Cleanup.
        assert!(lua_release_uid(uid));
    }

    #[test]
    fn test_lua_release_uid_unknown_returns_false() {
        with_env_mut(|env| {
            *env = ScriptEnvironment::new();
        });
        // 0xDEAD is outside the dynamic range and not registered.
        assert!(!lua_release_uid(0xDEAD));
    }

    #[test]
    fn test_lua_bridge_full_round_trip() {
        with_env_mut(|env| {
            *env = ScriptEnvironment::new();
        });
        // Simulate a Lua-side `Container:create(...)` registering a new
        // server-side container, then asking `getContainerByUID(uid)`.
        let uid = lua_register_thing(HandleKind::Container, boxed_default());
        assert!(lua_has_container(uid));
        // Now the script releases the handle.
        assert!(lua_release_uid(uid));
        assert!(!lua_has_thing(uid));
    }

    // --- reserve_script_env / reset_script_env (Tasks 5.5 and 5.6) ---
    //
    // C++: static std::array<ScriptEnvironment, 16> scriptEnv; int32_t scriptEnvIndex = -1;
    //   reserveScriptEnv() { return ++scriptEnvIndex < 16; }
    //   resetScriptEnv()   { assert(idx >= 0); scriptEnv[idx--].resetEnv(); }

    /// Task 5.5: reserve_script_env() returns true (non-null equivalent) and
    /// increments the env counter from its idle state (-1 → 0).
    #[test]
    fn test_reserve_script_env_returns_true_and_increments_counter() {
        // Reset stack to a known idle state.
        SCRIPT_ENV_INDEX.with(|c| *c.borrow_mut() = -1);
        SCRIPT_ENV_STACK.with(|s| s.borrow_mut().clear());

        let before = script_env_index();
        assert_eq!(before, -1, "stack should be idle (-1) before reserve");

        let ok = reserve_script_env();
        assert!(ok, "reserve_script_env() must return true on first call");
        assert_eq!(
            script_env_index(),
            0,
            "index must advance to 0 after first reserve"
        );

        // Cleanup: reset to idle.
        reset_script_env();
        SCRIPT_ENV_INDEX.with(|c| *c.borrow_mut() = -1);
    }

    /// Task 5.5: reserve_script_env() returns false when all 16 slots are
    /// exhausted (mirrors C++ `++scriptEnvIndex < 16` returning false).
    #[test]
    fn test_reserve_script_env_returns_false_at_capacity() {
        // Reset to idle.
        SCRIPT_ENV_INDEX.with(|c| *c.borrow_mut() = -1);
        SCRIPT_ENV_STACK.with(|s| s.borrow_mut().clear());

        // Reserve 16 slots (all should succeed).
        for i in 0..SCRIPT_ENV_CAPACITY {
            let ok = reserve_script_env();
            assert!(ok, "reserve #{i} should return true");
        }
        // The 17th reserve should return false (overflow).
        let overflow = reserve_script_env();
        assert!(
            !overflow,
            "reserve_script_env() must return false when stack is full"
        );

        // Cleanup.
        SCRIPT_ENV_INDEX.with(|c| *c.borrow_mut() = -1);
        SCRIPT_ENV_STACK.with(|s| s.borrow_mut().clear());
    }

    /// Task 5.6: reset_script_env() decrements the env counter back to the
    /// pre-reserve value, mirroring C++ `scriptEnv[scriptEnvIndex--].resetEnv()`.
    #[test]
    fn test_reset_script_env_decrements_counter_to_pre_reserve_value() {
        // Reset to idle.
        SCRIPT_ENV_INDEX.with(|c| *c.borrow_mut() = -1);
        SCRIPT_ENV_STACK.with(|s| s.borrow_mut().clear());

        assert_eq!(script_env_index(), -1);

        // Reserve a slot.
        assert!(reserve_script_env());
        assert_eq!(script_env_index(), 0);

        // Reset — index must return to -1.
        reset_script_env();
        assert_eq!(
            script_env_index(),
            -1,
            "index must return to -1 after reset (mirrors C++ scriptEnvIndex--)"
        );
    }

    /// Task 5.6: Two nested reserve/reset cycles work correctly (stack semantics).
    #[test]
    fn test_reserve_reset_nested_stack_semantics() {
        // Reset to idle.
        SCRIPT_ENV_INDEX.with(|c| *c.borrow_mut() = -1);
        SCRIPT_ENV_STACK.with(|s| s.borrow_mut().clear());

        // Outer callback.
        assert!(reserve_script_env());
        assert_eq!(script_env_index(), 0);

        // Inner callback (nested Lua call).
        assert!(reserve_script_env());
        assert_eq!(script_env_index(), 1);

        // Exit inner.
        reset_script_env();
        assert_eq!(script_env_index(), 0);

        // Exit outer.
        reset_script_env();
        assert_eq!(script_env_index(), -1);
    }
}
