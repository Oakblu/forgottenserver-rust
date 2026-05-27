use forgottenserver_common::enums::ReturnValue;
use forgottenserver_common::position::{Direction, Position};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Diagonal direction mask (mirrors DIRECTION_DIAGONAL_MASK in C++)
// ---------------------------------------------------------------------------

/// Bit mask for diagonal directions. A direction is diagonal when
/// `(direction as u8) & DIRECTION_DIAGONAL_MASK != 0`.
/// In the common crate Direction enum: Southwest=4, Southeast=5, Northwest=6, Northeast=7.
pub const DIRECTION_DIAGONAL_MASK: u8 = 0x04;

// ---------------------------------------------------------------------------
// Creature walk state — self-contained for pure logic (no networking)
// ---------------------------------------------------------------------------

/// Minimal creature walk state used by the movement subsystem.
/// Mirrors the relevant fields from C++ `Creature`.
#[derive(Debug, Clone)]
pub struct CreatureWalkState {
    pub id: u32,
    pub position: Position,
    pub direction: Direction,
    /// Whether the creature has the ROOT condition active.
    pub has_root_condition: bool,
    pub is_dead: bool,
    /// Number of times `on_walk` has been called (mirrors creature.onWalk()).
    pub walk_ticks: u32,
}

impl CreatureWalkState {
    pub fn new(id: u32, position: Position) -> Self {
        CreatureWalkState {
            id,
            position,
            direction: Direction::South,
            has_root_condition: false,
            is_dead: false,
            walk_ticks: 0,
        }
    }

    /// Simulate `creature.onWalk()` — increment walk tick counter.
    pub fn on_walk(&mut self) {
        self.walk_ticks += 1;
    }
}

// ---------------------------------------------------------------------------
// Tile query result — minimal tile abstraction for movement logic
// ---------------------------------------------------------------------------

/// Result of querying whether a creature can enter a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileQueryResult {
    /// Creature can enter (RETURNVALUE_NOERROR).
    Ok,
    /// Something blocks entry.
    Blocked(ReturnValue),
}

/// Minimal tile descriptor used by the movement system.
#[derive(Debug, Clone)]
pub struct TileDescriptor {
    pub position: Position,
    pub query_result: TileQueryResult,
}

impl TileDescriptor {
    pub fn passable(position: Position) -> Self {
        TileDescriptor {
            position,
            query_result: TileQueryResult::Ok,
        }
    }

    pub fn blocked(position: Position, rv: ReturnValue) -> Self {
        TileDescriptor {
            position,
            query_result: TileQueryResult::Blocked(rv),
        }
    }
}

// ---------------------------------------------------------------------------
// CreatureMover — pure-logic movement subsystem
// ---------------------------------------------------------------------------

/// Result of an `internal_move_creature` attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Ok,
    Err(ReturnValue),
}

/// The CreatureMover contains the stateless movement logic.
///
/// It mirrors the pure-logic portions of `Game::internalMoveCreature` and
/// `Game::internalCreatureTurn` from the C++ source, keeping them free of
/// network/map I/O so they can be tested in isolation.
#[derive(Debug, Default)]
pub struct CreatureMover;

impl CreatureMover {
    pub fn new() -> Self {
        CreatureMover
    }

    /// Attempt to move `creature` one step in `direction`.
    ///
    /// Mirrors `Game::internalMoveCreature(Creature*, Direction, uint32_t)`:
    /// 1. Computes `dest_pos` from the current position + direction delta.
    /// 2. If no tile is supplied (the map returned `None`), returns
    ///    `ReturnValue::NotPossible`.
    /// 3. Otherwise delegates to `internal_move_to_tile`.
    pub fn internal_move_by_direction(
        &self,
        creature: &mut CreatureWalkState,
        _direction: Direction,
        dest_tile: Option<TileDescriptor>,
    ) -> MoveResult {
        match dest_tile {
            None => MoveResult::Err(ReturnValue::NotPossible),
            Some(tile) => self.internal_move_to_tile(creature, tile),
        }
    }

    /// Attempt to move `creature` to the given tile.
    ///
    /// Mirrors `Game::internalMoveCreature(Creature&, Tile&, uint32_t)`:
    /// 1. If creature has ROOT condition → `RETURNVALUE_NOTPOSSIBLE`.
    /// 2. Query tile via `tile.query_result`.
    /// 3. If `Ok` → update creature position to tile position, return `Ok`.
    ///
    /// The C++ version also runs a loop over `queryDestination` (teleporters,
    /// ladders, etc.) and may call `internalCreatureTurn`. Those integration
    /// points need the map subsystem and are tested through `game.rs` methods.
    pub fn internal_move_to_tile(
        &self,
        creature: &mut CreatureWalkState,
        tile: TileDescriptor,
    ) -> MoveResult {
        // Root condition prevents all movement.
        if creature.has_root_condition {
            return MoveResult::Err(ReturnValue::NotPossible);
        }

        match tile.query_result {
            TileQueryResult::Ok => {
                creature.position = tile.position;
                MoveResult::Ok
            }
            TileQueryResult::Blocked(rv) => MoveResult::Err(rv),
        }
    }

    /// Turn a creature to face `new_dir`.
    ///
    /// Mirrors `Game::internalCreatureTurn(Creature*, Direction)`:
    /// - Returns `false` if the creature is already facing that direction
    ///   (no change needed).
    /// - Returns `true` and updates the direction if a change was made.
    ///
    /// The C++ version also notifies spectators; that I/O is not included here
    /// (pure logic only).
    pub fn internal_creature_turn(
        &self,
        creature: &mut CreatureWalkState,
        new_dir: Direction,
    ) -> bool {
        if creature.direction == new_dir {
            return false;
        }
        creature.direction = new_dir;
        true
    }

    /// Returns the destination position for a direction-based move.
    ///
    /// Wraps `Position::next_position` so callers do not need to import the
    /// common crate directly.
    pub fn get_dest_position(pos: Position, direction: Direction) -> Position {
        pos.next_position(direction)
    }

    /// Returns `true` if the direction is diagonal.
    pub fn is_diagonal(direction: Direction) -> bool {
        (direction as u8) & DIRECTION_DIAGONAL_MASK != 0
    }
}

// ---------------------------------------------------------------------------
// CreatureCheckList — mirrors addCreatureCheck / removeCreatureCheck
// ---------------------------------------------------------------------------

/// Tracks which creatures need periodic walk/think checks.
///
/// Mirrors the C++ fields `creatureCheck`, `inCheckCreaturesVector`, and the
/// `checkCreatureLists` array on the `Game` struct — distilled to a single
/// bucket for the pure-logic layer (multi-bucket sharding is an optimisation
/// detail not relevant to correctness testing).
#[derive(Debug, Default)]
pub struct CreatureCheckList {
    /// Creature IDs currently in the list.
    list: Vec<u32>,
    /// Whether each creature in `list` still wants to be checked.
    active: HashMap<u32, bool>,
}

impl CreatureCheckList {
    pub fn new() -> Self {
        CreatureCheckList {
            list: Vec::new(),
            active: HashMap::new(),
        }
    }

    /// Add a creature to the check list (idempotent — mirrors C++ dedup guard).
    ///
    /// Mirrors `Game::addCreatureCheck`: if already in the vector the flag is
    /// just set to `true` without a second insertion.
    pub fn add(&mut self, id: u32) {
        use std::collections::hash_map::Entry;
        match self.active.entry(id) {
            Entry::Occupied(mut e) => {
                *e.get_mut() = true;
            }
            Entry::Vacant(e) => {
                e.insert(true);
                self.list.push(id);
            }
        }
    }

    /// Mark a creature as no longer needing checks (lazy removal).
    ///
    /// Mirrors `Game::removeCreatureCheck`: sets the flag to `false` without
    /// immediately removing it from the list.
    pub fn remove(&mut self, id: u32) {
        if let Some(flag) = self.active.get_mut(&id) {
            *flag = false;
        }
    }

    /// Returns `true` if the creature is in the list (regardless of active flag).
    pub fn contains(&self, id: u32) -> bool {
        self.active.contains_key(&id)
    }

    /// Returns `true` if the creature is in the list AND still active.
    pub fn is_active(&self, id: u32) -> bool {
        self.active.get(&id).copied().unwrap_or(false)
    }

    /// Drain dead entries and return active IDs (used by the game loop).
    ///
    /// Mirrors the C++ `checkCreatureLists` sweep: removes inactive entries
    /// and returns the IDs of those still active.
    pub fn drain_active(&mut self) -> Vec<u32> {
        let active_ids: Vec<u32> = self
            .list
            .iter()
            .copied()
            .filter(|id| self.active.get(id).copied().unwrap_or(false))
            .collect();

        // Compact: keep only the active ones.
        self.list
            .retain(|id| self.active.get(id).copied().unwrap_or(false));
        // Remove inactive entries from the map.
        self.active.retain(|_, v| *v);

        active_ids
    }
}

/// Mirrors `MoveEvent_t` from movement.h — all 8 event variants plus sentinels.
///
/// C++ source defines:
///   MOVE_EVENT_STEP_IN, MOVE_EVENT_STEP_OUT,
///   MOVE_EVENT_EQUIP, MOVE_EVENT_DEEQUIP,
///   MOVE_EVENT_ADD_ITEM, MOVE_EVENT_REMOVE_ITEM,
///   MOVE_EVENT_ADD_ITEM_ITEMTILE, MOVE_EVENT_REMOVE_ITEM_ITEMTILE,
///   MOVE_EVENT_LAST, MOVE_EVENT_NONE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveEventType {
    /// MOVE_EVENT_STEP_IN — creature enters a tile.
    StepIn,
    /// MOVE_EVENT_STEP_OUT — creature leaves a tile.
    StepOut,
    /// MOVE_EVENT_EQUIP — player equips an item.
    Equip,
    /// MOVE_EVENT_DEEQUIP — player removes an item.
    DeEquip,
    /// MOVE_EVENT_ADD_ITEM — item added to a tile.
    AddItem,
    /// MOVE_EVENT_REMOVE_ITEM — item removed from a tile.
    RemoveItem,
    /// MOVE_EVENT_ADD_ITEM_ITEMTILE — add item when another item already on tile.
    AddItemTile,
    /// MOVE_EVENT_REMOVE_ITEM_ITEMTILE — remove item when another item on tile.
    RemoveItemTile,
}

// ---------------------------------------------------------------------------
// Backwards-compat alias — the old public name was `MoveEvent`.
// Internal code now uses `MoveEventType`; existing tests reference `MoveEvent`.
// ---------------------------------------------------------------------------
/// Deprecated alias kept for backwards compatibility.  New code should use
/// `MoveEventType` directly.
pub use MoveEventType as MoveEvent;

// ---------------------------------------------------------------------------
// Equip slot bitmask — mirrors SLOTP_* constants from C++
// ---------------------------------------------------------------------------

/// Equip slot bit positions. A `MoveEventDescriptor` with `slot_mask = 0`
/// means "any slot" (matches SLOTP_WHEREEVER behaviour).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotMask(pub u32);

impl SlotMask {
    pub const WHEREEVER: SlotMask = SlotMask(0xFFFF_FFFF);
    pub const HEAD: SlotMask = SlotMask(1 << 0);
    pub const NECKLACE: SlotMask = SlotMask(1 << 1);
    pub const BACKPACK: SlotMask = SlotMask(1 << 2);
    pub const ARMOR: SlotMask = SlotMask(1 << 3);
    pub const RIGHT: SlotMask = SlotMask(1 << 4);
    pub const LEFT: SlotMask = SlotMask(1 << 5);
    pub const LEGS: SlotMask = SlotMask(1 << 6);
    pub const FEET: SlotMask = SlotMask(1 << 7);
    pub const RING: SlotMask = SlotMask(1 << 8);
    pub const AMMO: SlotMask = SlotMask(1 << 9);

    /// Returns true when `other` overlaps this slot mask (mirrors C++ `(slot & slotp) != 0`).
    pub fn matches(&self, other: SlotMask) -> bool {
        (self.0 & other.0) != 0
    }
}

// ---------------------------------------------------------------------------
// WieldInfo flags — mirrors WieldInfo_t from C++
// ---------------------------------------------------------------------------

bitflags::bitflags! {
    /// Requirement flags that must be satisfied to equip an item.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WieldInfo: u32 {
        const LEVEL    = 0x01;
        const MAGLV    = 0x02;
        const PREMIUM  = 0x04;
        const VOCREQ   = 0x08;
    }
}

// ---------------------------------------------------------------------------
// MoveEventDescriptor — per-event data (mirrors MoveEvent class fields)
// ---------------------------------------------------------------------------

type CallbackId = u32;

/// A registered move event — mirrors the data fields of C++ `MoveEvent` class.
#[derive(Debug, Clone)]
pub struct MoveEventDescriptor {
    pub event_type: MoveEventType,
    pub callback_id: CallbackId,
    /// Which equip slot(s) this event applies to (only relevant for equip/deequip).
    pub slot_mask: SlotMask,
    /// Minimum level required to equip (0 = no requirement).
    pub req_level: u32,
    /// Minimum magic level required to equip (0 = no requirement).
    pub req_mag_level: u32,
    /// Whether a premium account is required.
    pub premium: bool,
    /// Wield info flags (bitmask of level/maglevel/premium/vocation requirements).
    pub wield_info: WieldInfo,
}

impl MoveEventDescriptor {
    /// Create a step event descriptor with default (no-restriction) values.
    pub fn step(event_type: MoveEventType, callback_id: CallbackId) -> Self {
        MoveEventDescriptor {
            event_type,
            callback_id,
            slot_mask: SlotMask::WHEREEVER,
            req_level: 0,
            req_mag_level: 0,
            premium: false,
            wield_info: WieldInfo::empty(),
        }
    }

    /// Create an equip event descriptor with specific slot and requirements.
    pub fn equip(
        event_type: MoveEventType,
        callback_id: CallbackId,
        slot_mask: SlotMask,
        req_level: u32,
        req_mag_level: u32,
        premium: bool,
    ) -> Self {
        let mut wield_info = WieldInfo::empty();
        if req_level > 0 {
            wield_info |= WieldInfo::LEVEL;
        }
        if req_mag_level > 0 {
            wield_info |= WieldInfo::MAGLV;
        }
        if premium {
            wield_info |= WieldInfo::PREMIUM;
        }
        MoveEventDescriptor {
            event_type,
            callback_id,
            slot_mask,
            req_level,
            req_mag_level,
            premium,
            wield_info,
        }
    }
}

// ---------------------------------------------------------------------------
// EquipCheckResult — mirrors the subset of ReturnValue used by EquipItem()
// ---------------------------------------------------------------------------

/// Result of an equip requirement check.
/// Mirrors the `RETURNVALUE_*` constants returned by `MoveEvent::EquipItem`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipCheckResult {
    /// RETURNVALUE_NOERROR — requirements satisfied.
    Ok,
    /// RETURNVALUE_NOTENOUGHLEVEL — player level too low.
    NotEnoughLevel,
    /// RETURNVALUE_NOTENOUGHMAGICLEVEL — player magic level too low.
    NotEnoughMagicLevel,
    /// RETURNVALUE_YOUNEEDPREMIUMACCOUNT — premium required but player is not premium.
    NeedPremium,
    /// RETURNVALUE_YOUDONTHAVEREQUIREDPROFESSION — wrong vocation.
    WrongVocation,
}

// ---------------------------------------------------------------------------
// Player profile — minimal data needed for equip requirement checks
// ---------------------------------------------------------------------------

/// Minimal player data needed by the equip check (no networking or map refs).
#[derive(Debug, Clone)]
pub struct PlayerProfile {
    pub id: u32,
    pub level: u32,
    pub magic_level: u32,
    pub is_premium: bool,
    /// Vocation id (0 = no vocation / all vocations pass).
    pub vocation_id: u16,
}

impl PlayerProfile {
    pub fn new(id: u32, level: u32, magic_level: u32, is_premium: bool, vocation_id: u16) -> Self {
        PlayerProfile {
            id,
            level,
            magic_level,
            is_premium,
            vocation_id,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-item event handler table
// ---------------------------------------------------------------------------

/// Per-item event handler table.
///
/// Stores a list of descriptors per (item_id, event_type) pair — mirrors
/// the `std::list<MoveEvent>` inside C++ `MoveEventList`.
#[derive(Debug, Default)]
pub struct MoveEventHandler {
    // (item_id, event_type) -> list of descriptors
    events: HashMap<(u32, MoveEventType), Vec<MoveEventDescriptor>>,
    // position -> list of descriptors per event_type (mirrors positionMap)
    pos_events: HashMap<(u16, u16, u8), Vec<MoveEventDescriptor>>,
}

impl MoveEventHandler {
    pub fn new() -> Self {
        MoveEventHandler {
            events: HashMap::new(),
            pos_events: HashMap::new(),
        }
    }

    /// Register a descriptor keyed by item id.
    pub fn register(&mut self, item_id: u32, desc: MoveEventDescriptor) {
        self.events
            .entry((item_id, desc.event_type))
            .or_default()
            .push(desc);
    }

    /// Register a descriptor keyed by position.
    pub fn register_pos(&mut self, pos: Position, desc: MoveEventDescriptor) {
        self.pos_events
            .entry((pos.x, pos.y, pos.z))
            .or_default()
            .push(desc);
    }

    /// Look up the first matching event for an item (ignores slot).
    pub fn get_event(
        &self,
        item_id: u32,
        event_type: MoveEventType,
    ) -> Option<&MoveEventDescriptor> {
        self.events
            .get(&(item_id, event_type))
            .and_then(|v| v.first())
    }

    /// Look up an equip/deequip event for an item that matches the slot mask.
    ///
    /// Mirrors C++ `MoveEvents::getEvent(Item*, MoveEvent_t, slots_t)`:
    /// iterates the list and returns the first descriptor whose `slot_mask`
    /// overlaps `slot`.
    pub fn get_equip_event(
        &self,
        item_id: u32,
        event_type: MoveEventType,
        slot: SlotMask,
    ) -> Option<&MoveEventDescriptor> {
        self.events
            .get(&(item_id, event_type))?
            .iter()
            .find(|d| d.slot_mask.matches(slot))
    }

    /// Look up a position-based event.
    pub fn get_pos_event(
        &self,
        pos: Position,
        event_type: MoveEventType,
    ) -> Option<&MoveEventDescriptor> {
        self.pos_events
            .get(&(pos.x, pos.y, pos.z))
            .and_then(|v| v.iter().find(|d| d.event_type == event_type))
    }
}

// ---------------------------------------------------------------------------
// Movement manager
// ---------------------------------------------------------------------------

/// Movement manager — fires step/equip/item events.
///
/// Mirrors the public API of C++ `MoveEvents`:
/// - `onCreatureMove` → `on_creature_move`
/// - `onPlayerEquip`  → `on_player_equip`
/// - `onPlayerDeEquip`→ `on_player_deequip`
/// - `onItemMove`     → `on_item_move`
#[derive(Debug, Default)]
pub struct Movement {
    handler: MoveEventHandler,
}

impl Movement {
    pub fn new() -> Self {
        Movement {
            handler: MoveEventHandler::new(),
        }
    }

    // ------------------------------------------------------------------
    // Registration
    // ------------------------------------------------------------------

    /// Register a move event descriptor by item id.
    pub fn add_move_event(&mut self, item_id: u32, desc: MoveEventDescriptor) {
        self.handler.register(item_id, desc);
    }

    /// Register a move event descriptor by position (mirrors positionMap).
    pub fn add_pos_event(&mut self, pos: Position, desc: MoveEventDescriptor) {
        self.handler.register_pos(pos, desc);
    }

    // ------------------------------------------------------------------
    // Legacy convenience registration (backwards-compat with old tests)
    // ------------------------------------------------------------------

    /// Register a simple step/equip event using the old (item_id, event_type, callback_id) API.
    pub fn add_move_event_simple(
        &mut self,
        item_id: u32,
        event_type: MoveEventType,
        callback_id: CallbackId,
    ) {
        self.handler
            .register(item_id, MoveEventDescriptor::step(event_type, callback_id));
    }

    // ------------------------------------------------------------------
    // Legacy fire helpers (backwards-compat with old tests)
    // ------------------------------------------------------------------

    pub fn fire_step_in(&self, item_id: u32) -> Option<CallbackId> {
        self.handler
            .get_event(item_id, MoveEventType::StepIn)
            .map(|d| d.callback_id)
    }

    pub fn fire_step_out(&self, item_id: u32) -> Option<CallbackId> {
        self.handler
            .get_event(item_id, MoveEventType::StepOut)
            .map(|d| d.callback_id)
    }

    pub fn fire_equip(&self, item_id: u32) -> Option<CallbackId> {
        self.handler
            .get_event(item_id, MoveEventType::Equip)
            .map(|d| d.callback_id)
    }

    pub fn fire_deequip(&self, item_id: u32) -> Option<CallbackId> {
        self.handler
            .get_event(item_id, MoveEventType::DeEquip)
            .map(|d| d.callback_id)
    }

    pub fn fire_add_item(&self, item_id: u32) -> Option<CallbackId> {
        self.handler
            .get_event(item_id, MoveEventType::AddItem)
            .map(|d| d.callback_id)
    }

    pub fn fire_remove_item(&self, item_id: u32) -> Option<CallbackId> {
        self.handler
            .get_event(item_id, MoveEventType::RemoveItem)
            .map(|d| d.callback_id)
    }

    // ------------------------------------------------------------------
    // onCreatureMove — mirrors MoveEvents::onCreatureMove
    // ------------------------------------------------------------------

    /// Fire step events when a creature enters (`StepIn`) or leaves (`StepOut`) a tile.
    ///
    /// Mirrors C++ `MoveEvents::onCreatureMove`:
    /// 1. Checks the position-based event registry.
    /// 2. Iterates tile items and fires their step events.
    ///
    /// Returns `true` if all fired callbacks succeeded (callback_id != 0),
    /// matching the C++ `ret &= ...` accumulation.
    pub fn on_creature_move(
        &self,
        pos: Position,
        tile_item_ids: &[u32],
        event_type: MoveEventType,
    ) -> bool {
        debug_assert!(
            event_type == MoveEventType::StepIn || event_type == MoveEventType::StepOut,
            "on_creature_move only fires StepIn / StepOut"
        );

        let mut ok = true;

        // Position-based event
        if let Some(desc) = self.handler.get_pos_event(pos, event_type) {
            ok &= desc.callback_id != 0;
        }

        // Item-based events on the tile
        for &item_id in tile_item_ids {
            if let Some(desc) = self.handler.get_event(item_id, event_type) {
                ok &= desc.callback_id != 0;
            }
        }

        ok
    }

    // ------------------------------------------------------------------
    // on_player_equip — mirrors MoveEvents::onPlayerEquip
    // ------------------------------------------------------------------

    /// Check and fire equip event for a player putting on an item.
    ///
    /// Mirrors C++ `MoveEvents::onPlayerEquip`:
    /// - Returns `Ok` if no equip event is registered for this item+slot.
    /// - Otherwise checks level, magic level, premium, and vocation requirements.
    pub fn on_player_equip(
        &self,
        item_id: u32,
        slot: SlotMask,
        player: &PlayerProfile,
        allowed_vocation_ids: &[u16],
    ) -> EquipCheckResult {
        let desc = match self
            .handler
            .get_equip_event(item_id, MoveEventType::Equip, slot)
        {
            None => return EquipCheckResult::Ok,
            Some(d) => d,
        };

        self.check_equip_requirements(desc, player, allowed_vocation_ids)
    }

    // ------------------------------------------------------------------
    // on_player_deequip — mirrors MoveEvents::onPlayerDeEquip
    // ------------------------------------------------------------------

    /// Fire deequip event for a player removing an item.
    ///
    /// Mirrors C++ `MoveEvents::onPlayerDeEquip`.
    /// Returns `Ok` if no event is registered.
    pub fn on_player_deequip(
        &self,
        item_id: u32,
        slot: SlotMask,
        _player: &PlayerProfile,
    ) -> EquipCheckResult {
        if self
            .handler
            .get_equip_event(item_id, MoveEventType::DeEquip, slot)
            .is_none()
        {
            return EquipCheckResult::Ok;
        }
        EquipCheckResult::Ok
    }

    // ------------------------------------------------------------------
    // on_item_move — mirrors MoveEvents::onItemMove
    // ------------------------------------------------------------------

    /// Fire add/remove item events when an item is placed on or removed from a tile.
    ///
    /// Mirrors C++ `MoveEvents::onItemMove`:
    /// - `is_add = true`  → fires `AddItem` events.
    /// - `is_add = false` → fires `RemoveItem` events.
    ///
    /// Returns `true` if all callbacks succeeded.
    pub fn on_item_move(
        &self,
        item_id: u32,
        pos: Position,
        tile_item_ids: &[u32],
        is_add: bool,
    ) -> bool {
        let (primary_event, tile_event) = if is_add {
            (MoveEventType::AddItem, MoveEventType::AddItemTile)
        } else {
            (MoveEventType::RemoveItem, MoveEventType::RemoveItemTile)
        };

        let mut ok = true;

        // Position-based
        if let Some(desc) = self.handler.get_pos_event(pos, primary_event) {
            ok &= desc.callback_id != 0;
        }

        // Item-based (the item being added/removed)
        if let Some(desc) = self.handler.get_event(item_id, primary_event) {
            ok &= desc.callback_id != 0;
        }

        // Tile items react to the item being added/removed (AddItemTile / RemoveItemTile)
        for &tile_id in tile_item_ids {
            if tile_id == item_id {
                continue;
            }
            if let Some(desc) = self.handler.get_event(tile_id, tile_event) {
                ok &= desc.callback_id != 0;
            }
        }

        ok
    }

    // ------------------------------------------------------------------
    // check_creature_move — position range validation
    // ------------------------------------------------------------------

    /// Returns `true` if a position-based step event exists at `pos`.
    ///
    /// Mirrors the C++ behaviour where `onCreatureMove` only fires if a
    /// `MoveEvent` is registered at the exact tile position or the tile
    /// has registered items. This helper checks the position registry only.
    pub fn check_creature_move(&self, pos: Position, event_type: MoveEventType) -> bool {
        self.handler.get_pos_event(pos, event_type).is_some()
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn check_equip_requirements(
        &self,
        desc: &MoveEventDescriptor,
        player: &PlayerProfile,
        allowed_vocation_ids: &[u16],
    ) -> EquipCheckResult {
        if desc.wield_info.is_empty() {
            return EquipCheckResult::Ok;
        }

        // Vocation check
        if desc.wield_info.contains(WieldInfo::VOCREQ) {
            let voc_ok = allowed_vocation_ids.is_empty()
                || allowed_vocation_ids.contains(&player.vocation_id);
            if !voc_ok {
                return EquipCheckResult::WrongVocation;
            }
        }

        // Level check
        if desc.wield_info.contains(WieldInfo::LEVEL) && player.level < desc.req_level {
            return EquipCheckResult::NotEnoughLevel;
        }

        // Magic level check
        if desc.wield_info.contains(WieldInfo::MAGLV) && player.magic_level < desc.req_mag_level {
            return EquipCheckResult::NotEnoughMagicLevel;
        }

        // Premium check
        if desc.wield_info.contains(WieldInfo::PREMIUM) && !player.is_premium {
            return EquipCheckResult::NeedPremium;
        }

        EquipCheckResult::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // MoveEvent (pre-existing)
    // -----------------------------------------------------------------------

    // 1. MoveEvent variants
    #[test]
    fn move_event_variants_exist() {
        let _ = MoveEvent::StepIn;
        let _ = MoveEvent::StepOut;
        let _ = MoveEvent::Equip;
        let _ = MoveEvent::DeEquip;
    }

    // 2a. Register and retrieve event
    #[test]
    fn handler_register_and_get_event() {
        let mut handler = MoveEventHandler::new();
        handler.register(10, MoveEventDescriptor::step(MoveEvent::StepIn, 42));
        assert_eq!(
            handler
                .get_event(10, MoveEvent::StepIn)
                .map(|d| d.callback_id),
            Some(42)
        );
    }

    // 2b. Returns None for unregistered event
    #[test]
    fn handler_get_event_returns_none_for_unregistered() {
        let mut handler = MoveEventHandler::new();
        handler.register(10, MoveEventDescriptor::step(MoveEvent::StepIn, 42));
        assert_eq!(
            handler
                .get_event(10, MoveEvent::StepOut)
                .map(|d| d.callback_id),
            None
        );
    }

    // 3a. Movement new creates empty manager
    #[test]
    fn movement_new_is_empty() {
        let m = Movement::new();
        assert!(m.fire_step_in(1).is_none());
    }

    // 3b. fire_step_in returns callback after add_move_event_simple
    #[test]
    fn movement_fire_step_in_returns_callback() {
        let mut m = Movement::new();
        m.add_move_event_simple(5, MoveEvent::StepIn, 99);
        assert_eq!(m.fire_step_in(5), Some(99));
    }

    // 3c. fire_step_out
    #[test]
    fn movement_fire_step_out_returns_callback() {
        let mut m = Movement::new();
        m.add_move_event_simple(5, MoveEvent::StepOut, 77);
        assert_eq!(m.fire_step_out(5), Some(77));
    }

    // 3d. fire_equip
    #[test]
    fn movement_fire_equip_returns_callback() {
        let mut m = Movement::new();
        m.add_move_event_simple(5, MoveEvent::Equip, 55);
        assert_eq!(m.fire_equip(5), Some(55));
    }

    // 3e. fire_deequip
    #[test]
    fn movement_fire_deequip_returns_callback() {
        let mut m = Movement::new();
        m.add_move_event_simple(5, MoveEvent::DeEquip, 33);
        assert_eq!(m.fire_deequip(5), Some(33));
    }

    // 3f. fire returns None for different item
    #[test]
    fn movement_fire_returns_none_for_different_item() {
        let mut m = Movement::new();
        m.add_move_event_simple(5, MoveEvent::StepIn, 99);
        assert!(m.fire_step_in(6).is_none());
    }

    // -----------------------------------------------------------------------
    // DIRECTION_DIAGONAL_MASK
    // -----------------------------------------------------------------------

    #[test]
    fn diagonal_mask_value_is_four() {
        assert_eq!(DIRECTION_DIAGONAL_MASK, 0x04);
    }

    #[test]
    fn is_diagonal_south_is_false() {
        assert!(!CreatureMover::is_diagonal(Direction::South));
    }

    #[test]
    fn is_diagonal_north_is_false() {
        assert!(!CreatureMover::is_diagonal(Direction::North));
    }

    #[test]
    fn is_diagonal_east_is_false() {
        assert!(!CreatureMover::is_diagonal(Direction::East));
    }

    #[test]
    fn is_diagonal_west_is_false() {
        assert!(!CreatureMover::is_diagonal(Direction::West));
    }

    #[test]
    fn is_diagonal_southwest_is_true() {
        assert!(CreatureMover::is_diagonal(Direction::Southwest));
    }

    #[test]
    fn is_diagonal_southeast_is_true() {
        assert!(CreatureMover::is_diagonal(Direction::Southeast));
    }

    #[test]
    fn is_diagonal_northwest_is_true() {
        assert!(CreatureMover::is_diagonal(Direction::Northwest));
    }

    #[test]
    fn is_diagonal_northeast_is_true() {
        assert!(CreatureMover::is_diagonal(Direction::Northeast));
    }

    // -----------------------------------------------------------------------
    // CreatureMover::internal_creature_turn
    // -----------------------------------------------------------------------

    #[test]
    fn creature_turn_to_same_direction_returns_false() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        c.direction = Direction::North;
        // turning to the same direction → no change, returns false
        assert!(!mover.internal_creature_turn(&mut c, Direction::North));
        assert_eq!(c.direction, Direction::North);
    }

    #[test]
    fn creature_turn_to_new_direction_returns_true() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        c.direction = Direction::South;
        assert!(mover.internal_creature_turn(&mut c, Direction::North));
        assert_eq!(c.direction, Direction::North);
    }

    #[test]
    fn creature_turn_updates_direction_field() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(50, 50, 7));
        c.direction = Direction::East;
        mover.internal_creature_turn(&mut c, Direction::West);
        assert_eq!(c.direction, Direction::West);
    }

    #[test]
    fn creature_turn_to_diagonal_direction() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        c.direction = Direction::North;
        assert!(mover.internal_creature_turn(&mut c, Direction::Northeast));
        assert_eq!(c.direction, Direction::Northeast);
    }

    // -----------------------------------------------------------------------
    // CreatureMover::internal_move_to_tile
    // -----------------------------------------------------------------------

    #[test]
    fn move_to_tile_updates_position_on_ok() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        let tile = TileDescriptor::passable(Position::new(101, 100, 7));
        let result = mover.internal_move_to_tile(&mut c, tile);
        assert_eq!(result, MoveResult::Ok);
        assert_eq!(c.position, Position::new(101, 100, 7));
    }

    #[test]
    fn move_to_tile_blocked_returns_error() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        let tile = TileDescriptor::blocked(Position::new(101, 100, 7), ReturnValue::NotEnoughRoom);
        let result = mover.internal_move_to_tile(&mut c, tile);
        assert_eq!(result, MoveResult::Err(ReturnValue::NotEnoughRoom));
        // position should be unchanged
        assert_eq!(c.position, Position::new(100, 100, 7));
    }

    #[test]
    fn move_to_tile_root_condition_prevents_move() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        c.has_root_condition = true;
        let tile = TileDescriptor::passable(Position::new(101, 100, 7));
        let result = mover.internal_move_to_tile(&mut c, tile);
        assert_eq!(result, MoveResult::Err(ReturnValue::NotPossible));
        // position should be unchanged
        assert_eq!(c.position, Position::new(100, 100, 7));
    }

    #[test]
    fn move_to_tile_root_overrides_passable_tile() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(50, 50, 7));
        c.has_root_condition = true;
        let tile = TileDescriptor::passable(Position::new(51, 50, 7));
        // root always blocks regardless of tile state
        assert_eq!(
            mover.internal_move_to_tile(&mut c, tile),
            MoveResult::Err(ReturnValue::NotPossible)
        );
    }

    // -----------------------------------------------------------------------
    // CreatureMover::internal_move_by_direction
    // -----------------------------------------------------------------------

    #[test]
    fn move_by_direction_no_tile_returns_not_possible() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        // dest_tile = None → tile not found on map → RETURNVALUE_NOTPOSSIBLE
        let result = mover.internal_move_by_direction(&mut c, Direction::North, None);
        assert_eq!(result, MoveResult::Err(ReturnValue::NotPossible));
    }

    #[test]
    fn move_by_direction_passable_tile_moves_creature() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        let dest = Position::new(100, 99, 7); // north
        let tile = TileDescriptor::passable(dest);
        let result = mover.internal_move_by_direction(&mut c, Direction::North, Some(tile));
        assert_eq!(result, MoveResult::Ok);
        assert_eq!(c.position, dest);
    }

    #[test]
    fn move_by_direction_blocked_tile_returns_error() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        let tile = TileDescriptor::blocked(Position::new(100, 99, 7), ReturnValue::CreatureBlock);
        let result = mover.internal_move_by_direction(&mut c, Direction::North, Some(tile));
        assert_eq!(result, MoveResult::Err(ReturnValue::CreatureBlock));
    }

    #[test]
    fn move_by_direction_root_condition_blocked_even_with_tile() {
        let mover = CreatureMover::new();
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        c.has_root_condition = true;
        let tile = TileDescriptor::passable(Position::new(100, 99, 7));
        let result = mover.internal_move_by_direction(&mut c, Direction::North, Some(tile));
        assert_eq!(result, MoveResult::Err(ReturnValue::NotPossible));
    }

    // -----------------------------------------------------------------------
    // CreatureMover::get_dest_position
    // -----------------------------------------------------------------------

    #[test]
    fn get_dest_position_north() {
        let pos = Position::new(100, 100, 7);
        assert_eq!(
            CreatureMover::get_dest_position(pos, Direction::North),
            Position::new(100, 99, 7)
        );
    }

    #[test]
    fn get_dest_position_east() {
        let pos = Position::new(100, 100, 7);
        assert_eq!(
            CreatureMover::get_dest_position(pos, Direction::East),
            Position::new(101, 100, 7)
        );
    }

    #[test]
    fn get_dest_position_south() {
        let pos = Position::new(100, 100, 7);
        assert_eq!(
            CreatureMover::get_dest_position(pos, Direction::South),
            Position::new(100, 101, 7)
        );
    }

    #[test]
    fn get_dest_position_west() {
        let pos = Position::new(100, 100, 7);
        assert_eq!(
            CreatureMover::get_dest_position(pos, Direction::West),
            Position::new(99, 100, 7)
        );
    }

    // -----------------------------------------------------------------------
    // CreatureWalkState helpers
    // -----------------------------------------------------------------------

    #[test]
    fn creature_walk_state_on_walk_increments_tick() {
        let mut c = CreatureWalkState::new(1, Position::new(100, 100, 7));
        assert_eq!(c.walk_ticks, 0);
        c.on_walk();
        assert_eq!(c.walk_ticks, 1);
        c.on_walk();
        assert_eq!(c.walk_ticks, 2);
    }

    #[test]
    fn creature_walk_state_default_direction_is_south() {
        let c = CreatureWalkState::new(1, Position::new(0, 0, 7));
        assert_eq!(c.direction, Direction::South);
    }

    #[test]
    fn creature_walk_state_default_not_dead_not_rooted() {
        let c = CreatureWalkState::new(2, Position::new(0, 0, 7));
        assert!(!c.is_dead);
        assert!(!c.has_root_condition);
    }

    // -----------------------------------------------------------------------
    // CreatureCheckList — addCreatureCheck / removeCreatureCheck
    // -----------------------------------------------------------------------

    #[test]
    fn check_list_add_creature_marks_active() {
        let mut list = CreatureCheckList::new();
        list.add(42);
        assert!(list.contains(42));
        assert!(list.is_active(42));
    }

    #[test]
    fn check_list_add_idempotent() {
        let mut list = CreatureCheckList::new();
        list.add(1);
        list.add(1); // second call must not duplicate
        let active = list.drain_active();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn check_list_remove_marks_inactive() {
        let mut list = CreatureCheckList::new();
        list.add(10);
        list.remove(10);
        // still in list (lazy), but flagged inactive
        assert!(list.contains(10));
        assert!(!list.is_active(10));
    }

    #[test]
    fn check_list_remove_not_present_is_noop() {
        let mut list = CreatureCheckList::new();
        // Should not panic; unknown IDs are silently ignored.
        list.remove(999);
        assert!(!list.contains(999));
    }

    #[test]
    fn check_list_drain_active_returns_only_active() {
        let mut list = CreatureCheckList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove(2);
        let active = list.drain_active();
        assert!(active.contains(&1));
        assert!(active.contains(&3));
        assert!(!active.contains(&2));
    }

    #[test]
    fn check_list_drain_active_compacts_list() {
        let mut list = CreatureCheckList::new();
        list.add(1);
        list.add(2);
        list.remove(1);
        list.drain_active();
        // After drain, inactive entry 1 should be gone
        assert!(!list.contains(1));
        assert!(list.contains(2));
    }

    #[test]
    fn check_list_re_add_after_remove_makes_active_again() {
        let mut list = CreatureCheckList::new();
        list.add(5);
        list.remove(5);
        assert!(!list.is_active(5));
        list.add(5); // re-add should mark active again
        assert!(list.is_active(5));
    }

    #[test]
    fn check_list_multiple_creatures_independent() {
        let mut list = CreatureCheckList::new();
        list.add(10);
        list.add(20);
        list.add(30);
        list.remove(20);
        assert!(list.is_active(10));
        assert!(!list.is_active(20));
        assert!(list.is_active(30));
    }

    // -----------------------------------------------------------------------
    // Phase 12.4 — new tests for expanded MoveEvent system
    // -----------------------------------------------------------------------

    // Helper
    fn player_profile(
        level: u32,
        magic_level: u32,
        is_premium: bool,
        vocation_id: u16,
    ) -> PlayerProfile {
        PlayerProfile::new(1, level, magic_level, is_premium, vocation_id)
    }

    fn sdesc(event_type: MoveEventType, cb: CallbackId) -> MoveEventDescriptor {
        MoveEventDescriptor::step(event_type, cb)
    }

    // ---
    // MoveEventType — all 8 variants
    // ---

    #[test]
    fn move_event_type_all_8_variants_exist() {
        let _ = MoveEventType::StepIn;
        let _ = MoveEventType::StepOut;
        let _ = MoveEventType::Equip;
        let _ = MoveEventType::DeEquip;
        let _ = MoveEventType::AddItem;
        let _ = MoveEventType::RemoveItem;
        let _ = MoveEventType::AddItemTile;
        let _ = MoveEventType::RemoveItemTile;
    }

    // ---
    // SlotMask
    // ---

    #[test]
    fn slot_mask_same_slot_matches() {
        assert!(SlotMask::ARMOR.matches(SlotMask::ARMOR));
        assert!(SlotMask::HEAD.matches(SlotMask::HEAD));
    }

    #[test]
    fn slot_mask_different_slots_do_not_match() {
        assert!(!SlotMask::ARMOR.matches(SlotMask::HEAD));
        assert!(!SlotMask::RING.matches(SlotMask::FEET));
    }

    #[test]
    fn slot_mask_whereever_matches_all_slots() {
        assert!(SlotMask::WHEREEVER.matches(SlotMask::HEAD));
        assert!(SlotMask::WHEREEVER.matches(SlotMask::ARMOR));
        assert!(SlotMask::WHEREEVER.matches(SlotMask::FEET));
        assert!(SlotMask::WHEREEVER.matches(SlotMask::RING));
        assert!(SlotMask::WHEREEVER.matches(SlotMask::AMMO));
    }

    // ---
    // WieldInfo
    // ---

    #[test]
    fn wield_info_level_flag_set_when_req_level_nonzero() {
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 50, 0, false);
        assert!(desc.wield_info.contains(WieldInfo::LEVEL));
        assert!(!desc.wield_info.contains(WieldInfo::MAGLV));
    }

    #[test]
    fn wield_info_maglv_flag_set_when_req_magic_level_nonzero() {
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 5, false);
        assert!(!desc.wield_info.contains(WieldInfo::LEVEL));
        assert!(desc.wield_info.contains(WieldInfo::MAGLV));
    }

    #[test]
    fn wield_info_premium_flag_set_when_premium_required() {
        let desc = MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, true);
        assert!(desc.wield_info.contains(WieldInfo::PREMIUM));
    }

    #[test]
    fn wield_info_empty_when_no_requirements() {
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, false);
        assert!(desc.wield_info.is_empty());
    }

    #[test]
    fn wield_info_all_flags_when_all_requirements() {
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 50, 5, true);
        assert!(desc.wield_info.contains(WieldInfo::LEVEL));
        assert!(desc.wield_info.contains(WieldInfo::MAGLV));
        assert!(desc.wield_info.contains(WieldInfo::PREMIUM));
    }

    // ---
    // MoveEventHandler — descriptor-based API
    // ---

    #[test]
    fn handler_register_pos_and_get_pos_event() {
        let mut handler = MoveEventHandler::new();
        let pos = Position::new(100, 200, 7);
        handler.register_pos(pos, sdesc(MoveEventType::StepIn, 77));
        let d = handler.get_pos_event(pos, MoveEventType::StepIn);
        assert!(d.is_some());
        assert_eq!(d.unwrap().callback_id, 77);
    }

    #[test]
    fn handler_get_pos_event_none_for_wrong_event_type() {
        let mut handler = MoveEventHandler::new();
        let pos = Position::new(10, 20, 7);
        handler.register_pos(pos, sdesc(MoveEventType::StepIn, 5));
        assert!(handler.get_pos_event(pos, MoveEventType::StepOut).is_none());
    }

    #[test]
    fn handler_get_pos_event_none_for_unregistered_position() {
        let handler = MoveEventHandler::new();
        let pos = Position::new(99, 99, 7);
        assert!(handler.get_pos_event(pos, MoveEventType::StepIn).is_none());
    }

    #[test]
    fn handler_equip_event_matches_slot() {
        let mut handler = MoveEventHandler::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 99, SlotMask::ARMOR, 0, 0, false);
        handler.register(5, desc);
        assert!(handler
            .get_equip_event(5, MoveEventType::Equip, SlotMask::ARMOR)
            .is_some());
        assert!(handler
            .get_equip_event(5, MoveEventType::Equip, SlotMask::HEAD)
            .is_none());
    }

    #[test]
    fn handler_equip_whereever_matches_any_slot() {
        let mut handler = MoveEventHandler::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::WHEREEVER, 0, 0, false);
        handler.register(7, desc);
        assert!(handler
            .get_equip_event(7, MoveEventType::Equip, SlotMask::HEAD)
            .is_some());
        assert!(handler
            .get_equip_event(7, MoveEventType::Equip, SlotMask::RING)
            .is_some());
    }

    #[test]
    fn handler_multiple_descriptors_returns_first() {
        let mut handler = MoveEventHandler::new();
        handler.register(10, sdesc(MoveEventType::StepIn, 100));
        handler.register(10, sdesc(MoveEventType::StepIn, 200));
        assert_eq!(
            handler
                .get_event(10, MoveEventType::StepIn)
                .unwrap()
                .callback_id,
            100
        );
    }

    // ---
    // Movement::add_move_event (descriptor-based) + fire helpers
    // ---

    #[test]
    fn movement_fire_add_item_returns_callback() {
        let mut m = Movement::new();
        m.add_move_event_simple(10, MoveEventType::AddItem, 11);
        assert_eq!(m.fire_add_item(10), Some(11));
    }

    #[test]
    fn movement_fire_remove_item_returns_callback() {
        let mut m = Movement::new();
        m.add_move_event_simple(20, MoveEventType::RemoveItem, 22);
        assert_eq!(m.fire_remove_item(20), Some(22));
    }

    #[test]
    fn add_item_and_remove_item_are_distinct() {
        let mut m = Movement::new();
        m.add_move_event_simple(30, MoveEventType::AddItem, 1);
        assert!(
            m.fire_remove_item(30).is_none(),
            "AddItem callback must not fire for RemoveItem"
        );
        m.add_move_event_simple(30, MoveEventType::RemoveItem, 2);
        assert_eq!(m.fire_add_item(30), Some(1));
        assert_eq!(m.fire_remove_item(30), Some(2));
    }

    #[test]
    fn step_in_does_not_fire_step_out_callback() {
        let mut m = Movement::new();
        m.add_move_event_simple(50, MoveEventType::StepIn, 7);
        assert!(m.fire_step_out(50).is_none());
        assert_eq!(m.fire_step_in(50), Some(7));
    }

    // ---
    // on_creature_move — mirrors MoveEvents::onCreatureMove
    // ---

    #[test]
    fn on_creature_move_step_in_fires_pos_event() {
        let mut m = Movement::new();
        let pos = Position::new(100, 100, 7);
        m.add_pos_event(pos, sdesc(MoveEventType::StepIn, 1));
        assert!(m.on_creature_move(pos, &[], MoveEventType::StepIn));
    }

    #[test]
    fn on_creature_move_step_out_fires_pos_event() {
        let mut m = Movement::new();
        let pos = Position::new(50, 50, 7);
        m.add_pos_event(pos, sdesc(MoveEventType::StepOut, 1));
        assert!(m.on_creature_move(pos, &[], MoveEventType::StepOut));
    }

    #[test]
    fn on_creature_move_fires_tile_item_step_in_events() {
        let mut m = Movement::new();
        let pos = Position::new(10, 10, 7);
        m.add_move_event_simple(42, MoveEventType::StepIn, 5);
        assert!(m.on_creature_move(pos, &[42], MoveEventType::StepIn));
    }

    #[test]
    fn on_creature_move_returns_true_when_no_events_registered() {
        let m = Movement::new();
        let pos = Position::new(1, 1, 7);
        assert!(m.on_creature_move(pos, &[], MoveEventType::StepIn));
    }

    #[test]
    fn on_creature_move_step_in_event_does_not_fire_for_step_out_call() {
        let mut m = Movement::new();
        let pos = Position::new(20, 20, 7);
        // Only StepIn event registered; firing StepOut must not match it
        m.add_pos_event(pos, sdesc(MoveEventType::StepIn, 1));
        // StepOut lookup finds nothing → stays true (no callback ran)
        assert!(m.on_creature_move(pos, &[], MoveEventType::StepOut));
    }

    #[test]
    fn on_creature_move_fires_multiple_tile_items() {
        let mut m = Movement::new();
        let pos = Position::new(5, 5, 7);
        m.add_move_event_simple(1, MoveEventType::StepIn, 10);
        m.add_move_event_simple(2, MoveEventType::StepIn, 11);
        assert!(m.on_creature_move(pos, &[1, 2], MoveEventType::StepIn));
    }

    // ---
    // check_creature_move — position range validation
    // ---

    #[test]
    fn check_creature_move_returns_true_when_event_at_pos() {
        let mut m = Movement::new();
        let pos = Position::new(100, 100, 7);
        m.add_pos_event(pos, sdesc(MoveEventType::StepIn, 1));
        assert!(m.check_creature_move(pos, MoveEventType::StepIn));
    }

    #[test]
    fn check_creature_move_returns_false_when_no_event_at_pos() {
        let m = Movement::new();
        let pos = Position::new(200, 200, 7);
        assert!(!m.check_creature_move(pos, MoveEventType::StepIn));
    }

    #[test]
    fn check_creature_move_different_positions_are_independent() {
        let mut m = Movement::new();
        let pos_a = Position::new(1, 1, 7);
        let pos_b = Position::new(2, 2, 7);
        m.add_pos_event(pos_a, sdesc(MoveEventType::StepIn, 1));
        assert!(m.check_creature_move(pos_a, MoveEventType::StepIn));
        assert!(!m.check_creature_move(pos_b, MoveEventType::StepIn));
    }

    #[test]
    fn check_creature_move_wrong_event_type_returns_false() {
        let mut m = Movement::new();
        let pos = Position::new(5, 5, 7);
        m.add_pos_event(pos, sdesc(MoveEventType::StepIn, 1));
        assert!(!m.check_creature_move(pos, MoveEventType::StepOut));
    }

    // ---
    // on_item_move — mirrors MoveEvents::onItemMove
    // ---

    #[test]
    fn on_item_move_add_fires_add_item_event() {
        let mut m = Movement::new();
        let pos = Position::new(10, 10, 7);
        m.add_move_event_simple(99, MoveEventType::AddItem, 1);
        assert!(m.on_item_move(99, pos, &[], true));
    }

    #[test]
    fn on_item_move_remove_fires_remove_item_event() {
        let mut m = Movement::new();
        let pos = Position::new(10, 10, 7);
        m.add_move_event_simple(99, MoveEventType::RemoveItem, 1);
        assert!(m.on_item_move(99, pos, &[], false));
    }

    #[test]
    fn on_item_move_add_fires_add_item_tile_for_existing_tile_items() {
        let mut m = Movement::new();
        let pos = Position::new(10, 10, 7);
        m.add_move_event_simple(55, MoveEventType::AddItemTile, 1);
        assert!(m.on_item_move(99, pos, &[55], true));
    }

    #[test]
    fn on_item_move_remove_fires_remove_item_tile_for_existing_tile_items() {
        let mut m = Movement::new();
        let pos = Position::new(10, 10, 7);
        m.add_move_event_simple(55, MoveEventType::RemoveItemTile, 1);
        assert!(m.on_item_move(99, pos, &[55], false));
    }

    #[test]
    fn on_item_move_skips_tile_event_for_the_same_item() {
        let mut m = Movement::new();
        let pos = Position::new(10, 10, 7);
        m.add_move_event_simple(99, MoveEventType::AddItemTile, 1);
        // tile_item_ids includes 99 (the item being added) — should be skipped
        assert!(m.on_item_move(99, pos, &[99], true));
    }

    #[test]
    fn on_item_move_returns_true_when_no_events_registered() {
        let m = Movement::new();
        let pos = Position::new(1, 1, 7);
        assert!(m.on_item_move(5, pos, &[], true));
        assert!(m.on_item_move(5, pos, &[], false));
    }

    // ---
    // on_player_equip — mirrors MoveEvents::onPlayerEquip
    // ---

    #[test]
    fn equip_no_event_registered_returns_ok() {
        let m = Movement::new();
        let p = player_profile(1, 0, false, 1);
        assert_eq!(
            m.on_player_equip(10, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_event_with_no_requirements_returns_ok() {
        let mut m = Movement::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, false);
        m.add_move_event(5, desc);
        let p = player_profile(1, 0, false, 1);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_wrong_slot_returns_ok_because_no_match() {
        let mut m = Movement::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 50, 0, false);
        m.add_move_event(5, desc);
        let p = player_profile(1, 0, false, 1);
        // HEAD slot doesn't match ARMOR → no event found → Ok
        assert_eq!(
            m.on_player_equip(5, SlotMask::HEAD, &p, &[]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_level_too_low_returns_not_enough_level() {
        let mut m = Movement::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 50, 0, false);
        m.add_move_event(5, desc);
        let p = player_profile(30, 0, false, 1);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::NotEnoughLevel
        );
    }

    #[test]
    fn equip_level_exactly_met_returns_ok() {
        let mut m = Movement::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 50, 0, false);
        m.add_move_event(5, desc);
        let p = player_profile(50, 0, false, 1);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_magic_level_too_low_returns_not_enough_magic_level() {
        let mut m = Movement::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 5, false);
        m.add_move_event(5, desc);
        let p = player_profile(100, 3, false, 1);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::NotEnoughMagicLevel
        );
    }

    #[test]
    fn equip_magic_level_exactly_met_returns_ok() {
        let mut m = Movement::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 5, false);
        m.add_move_event(5, desc);
        let p = player_profile(100, 5, false, 1);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_premium_required_non_premium_player_fails() {
        let mut m = Movement::new();
        let desc = MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, true);
        m.add_move_event(5, desc);
        let p = player_profile(100, 100, false, 1);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::NeedPremium
        );
    }

    #[test]
    fn equip_premium_required_premium_player_passes() {
        let mut m = Movement::new();
        let desc = MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, true);
        m.add_move_event(5, desc);
        let p = player_profile(100, 100, true, 1);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_wrong_vocation_returns_wrong_vocation() {
        let mut m = Movement::new();
        let mut desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, false);
        desc.wield_info |= WieldInfo::VOCREQ;
        m.add_move_event(5, desc);
        let p = player_profile(100, 100, false, 3);
        // allowed vocation = 2, player = 3
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[2]),
            EquipCheckResult::WrongVocation
        );
    }

    #[test]
    fn equip_correct_vocation_returns_ok() {
        let mut m = Movement::new();
        let mut desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, false);
        desc.wield_info |= WieldInfo::VOCREQ;
        m.add_move_event(5, desc);
        let p = player_profile(100, 100, false, 2);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[2]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_empty_allowed_vocations_passes_all() {
        let mut m = Movement::new();
        let mut desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 0, 0, false);
        desc.wield_info |= WieldInfo::VOCREQ;
        m.add_move_event(5, desc);
        // empty allowed list → all vocations pass
        let p = player_profile(100, 100, false, 99);
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn equip_applies_only_in_equip_slot_not_deequip() {
        let mut m = Movement::new();
        // Register equip event with level requirement
        let desc =
            MoveEventDescriptor::equip(MoveEventType::Equip, 1, SlotMask::ARMOR, 50, 0, false);
        m.add_move_event(5, desc);
        let p = player_profile(30, 0, false, 1);
        // equip fires and checks requirements
        assert_eq!(
            m.on_player_equip(5, SlotMask::ARMOR, &p, &[]),
            EquipCheckResult::NotEnoughLevel
        );
        // deequip has no event → always Ok
        assert_eq!(
            m.on_player_deequip(5, SlotMask::ARMOR, &p),
            EquipCheckResult::Ok
        );
    }

    // ---
    // on_player_deequip — mirrors MoveEvents::onPlayerDeEquip
    // ---

    #[test]
    fn deequip_no_event_registered_returns_ok() {
        let m = Movement::new();
        let p = player_profile(1, 0, false, 1);
        assert_eq!(
            m.on_player_deequip(10, SlotMask::ARMOR, &p),
            EquipCheckResult::Ok
        );
    }

    #[test]
    fn deequip_event_registered_returns_ok() {
        let mut m = Movement::new();
        let desc =
            MoveEventDescriptor::equip(MoveEventType::DeEquip, 1, SlotMask::ARMOR, 0, 0, false);
        m.add_move_event(5, desc);
        let p = player_profile(1, 0, false, 1);
        assert_eq!(
            m.on_player_deequip(5, SlotMask::ARMOR, &p),
            EquipCheckResult::Ok
        );
    }

    // ---
    // Phase 9 audit — final coverage closure (4 uncovered regions)
    // Mirrors uncovered branches in MoveEvents::onItemMove and onCreatureMove
    // ---

    /// Mirrors C++ `MoveEvents::onCreatureMove` iteration over tile items:
    /// when a tile contains an Item that has NO MoveEvent registered for the
    /// fired event type, the loop body's inner `if (moveEvent)` test is false
    /// and execution falls through to the next item. Exercises the
    /// `get_event(item_id, …) == None` branch in `on_creature_move`.
    #[test]
    fn on_creature_move_tile_item_without_event_does_not_change_result() {
        let m = Movement::new();
        let pos = Position::new(7, 8, 9);
        // tile contains item id 123 but nothing is registered for it.
        assert!(m.on_creature_move(pos, &[123], MoveEventType::StepIn));
    }

    /// Mirrors the position-based dispatch branch in C++
    /// `MoveEvents::onItemMove`:
    ///   moveEvent = getEvent(tile, eventType1);
    ///   if (moveEvent) { ret &= moveEvent->fireAddRemItem(...); }
    /// Registers a position-keyed AddItem event so the previously-uncovered
    /// pos_event Some-branch executes.
    #[test]
    fn on_item_move_add_fires_position_based_event() {
        let mut m = Movement::new();
        let pos = Position::new(10, 10, 7);
        // pos-keyed AddItem event with success callback (id != 0)
        m.add_pos_event(pos, MoveEventDescriptor::step(MoveEventType::AddItem, 1));
        assert!(m.on_item_move(99, pos, &[], true));
    }

    /// Same path as above but for the RemoveItem branch — also exercises the
    /// `callback_id == 0` accumulation, ensuring the position-based event
    /// can drive `ok` to `false` exactly as the C++ `ret &=` aggregation does.
    #[test]
    fn on_item_move_remove_position_event_with_zero_callback_returns_false() {
        let mut m = Movement::new();
        let pos = Position::new(11, 12, 5);
        m.add_pos_event(pos, MoveEventDescriptor::step(MoveEventType::RemoveItem, 0));
        // pos-event present but callback_id == 0 → ok becomes false
        assert!(!m.on_item_move(99, pos, &[], false));
    }

    /// Mirrors the inner loop of C++ `MoveEvents::onItemMove`:
    ///   moveEvent = getEvent(tileItem, eventType2);
    ///   if (moveEvent) { ret &= moveEvent->fireAddRemItem(...); }
    /// When the tile contains an item that has NO `*_ITEMTILE` event registered,
    /// the inner `if` is false and the loop continues. Covers the None-branch
    /// of `get_event(tile_id, tile_event)` in `on_item_move`.
    #[test]
    fn on_item_move_tile_item_without_tile_event_does_not_change_result() {
        let m = Movement::new();
        let pos = Position::new(3, 4, 7);
        // tile contains item id 55 but NO AddItemTile event is registered.
        assert!(m.on_item_move(99, pos, &[55], true));
    }
}
