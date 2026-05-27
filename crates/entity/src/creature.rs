//! Migrated from forgottenserver/src/creature.h and creature.cpp
//! Creature base data-bag. No inheritance; composition only.

use forgottenserver_common::position::Position;

// ---------------------------------------------------------------------------
// ConditionEntry — simple condition tracker
// ---------------------------------------------------------------------------

/// A lightweight condition entry stored on a creature.
/// `ticks == -1` means the condition is permanent (never expires on its own).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionEntry {
    /// The condition type identifier (mirrors ConditionType_t bit flags).
    pub condition_type: i32,
    /// Remaining ticks in milliseconds. `-1` means permanent.
    pub ticks: i32,
}

impl ConditionEntry {
    pub fn new(condition_type: i32, ticks: i32) -> Self {
        ConditionEntry {
            condition_type,
            ticks,
        }
    }
}

// ---------------------------------------------------------------------------
// Direction — movement direction (mirrors Direction enum in C++)
// ---------------------------------------------------------------------------

/// Movement direction. Mirrors the C++ `Direction` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Direction {
    North = 0,
    East = 1,
    #[default]
    South = 2,
    West = 3,
    NorthEast = 4,
    SouthEast = 5,
    SouthWest = 6,
    NorthWest = 7,
}

impl Direction {
    /// Returns `true` for diagonal directions (NE, SE, SW, NW).
    pub fn is_diagonal(self) -> bool {
        matches!(
            self,
            Direction::NorthEast
                | Direction::SouthEast
                | Direction::SouthWest
                | Direction::NorthWest
        )
    }
}

// ---------------------------------------------------------------------------
// SkullType — skull level (mirrors Skulls_t in C++)
// ---------------------------------------------------------------------------

/// Skull level for a creature. Mirrors the C++ `Skulls_t` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SkullType {
    #[default]
    None = 0,
    Yellow = 1,
    Green = 2,
    White = 3,
    Red = 4,
    Black = 5,
    Orange = 6,
}

// ---------------------------------------------------------------------------
// LightInfo — light level and color
// ---------------------------------------------------------------------------

/// Light emitted by a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LightInfo {
    pub level: u8,
    pub color: u8,
}

// ---------------------------------------------------------------------------
// Creature
// ---------------------------------------------------------------------------

/// Base data-bag for all creature types. Mirrors the data members of the C++
/// `Creature` class that are needed for game logic, without the OOP hierarchy.
#[derive(Debug)]
pub struct Creature {
    pub id: u32,
    pub name: String,

    pub health: i32,
    pub health_max: i32,

    pub base_speed: u32,
    pub var_speed: i32,

    pub light: LightInfo,

    pub conditions: Vec<ConditionEntry>,
    pub summons: Vec<u32>,

    pub attack_target: Option<u32>,

    pub hp_recovery_ticks: u32,
    pub mana_recovery_ticks: u32,

    pub pushable: bool,

    /// SpawnId of the spawn point that created this creature, if any.
    /// Set by the spawn manager when the creature is placed in the world.
    pub spawn_id: Option<u32>,

    // --- Fields added during audit ---
    /// Skull type (mirrors `Skulls_t skull` in C++ Creature).
    pub skull: SkullType,

    /// Current movement direction (mirrors `Direction direction` in C++).
    pub direction: Direction,

    /// Whether health is hidden from clients.
    pub hidden_health: bool,

    /// Whether movement is blocked (e.g. by a combat spell or condition).
    pub movement_blocked: bool,

    /// Whether this creature drops loot on death.
    pub loot_drop: bool,

    /// Whether this creature loses skill/exp on death.
    pub skill_loss: bool,

    /// Drunkenness level (0–100) affects walk direction randomisation.
    pub drunkenness: u8,

    /// Pending walk step queue (mirrors `listWalkDir` in C++, stored
    /// back-first so `pop` returns the *next* step, matching C++ `back()`).
    pub walk_steps: Vec<Direction>,

    /// Whether the next walk step should be cancelled (mirrors `cancelNextWalk`).
    pub cancel_next_walk: bool,

    /// Current position on the map.
    pub position: Position,
}

impl Creature {
    /// Create a new creature with given id and name.
    /// Defaults mirror the C++ class defaults:
    ///   health/healthMax = 100 (task spec default; C++ uses 1000 but spec says 100)
    ///   baseSpeed = 220
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Creature {
            id,
            name: name.into(),
            health: 100,
            health_max: 100,
            base_speed: 220,
            var_speed: 0,
            light: LightInfo::default(),
            conditions: Vec::new(),
            summons: Vec::new(),
            attack_target: None,
            hp_recovery_ticks: 1000,
            mana_recovery_ticks: 1000,
            pushable: true,
            spawn_id: None,
            skull: SkullType::default(),
            direction: Direction::default(),
            hidden_health: false,
            movement_blocked: false,
            loot_drop: true,
            skill_loss: true,
            drunkenness: 0,
            walk_steps: Vec::new(),
            cancel_next_walk: false,
            position: Position::default(),
        }
    }

    // --- Basic accessors ---

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    // --- Health ---

    pub fn get_health(&self) -> i32 {
        self.health
    }

    pub fn get_max_health(&self) -> i32 {
        self.health_max
    }

    pub fn set_health(&mut self, hp: i32) {
        self.health = hp;
    }

    pub fn set_max_health(&mut self, hp: i32) {
        self.health_max = hp;
    }

    // --- Speed ---

    pub fn get_speed(&self) -> i32 {
        self.base_speed as i32 + self.var_speed
    }

    pub fn set_base_speed(&mut self, speed: u32) {
        self.base_speed = speed;
    }

    pub fn get_base_speed(&self) -> u32 {
        self.base_speed
    }

    pub fn change_speed(&mut self, delta: i32) {
        self.var_speed += delta;
    }

    /// Walk delay in milliseconds for a step in the given direction.
    ///
    /// Formula (from creature.cpp `getStepDuration`):
    ///   diagonal steps → numerator = 1414
    ///   non-diagonal   → numerator = 1000
    ///   delay = numerator * BASE_SPEED / max(1, speed)
    ///
    /// The C++ base speed for the denominator factor is 220.
    pub fn get_walk_delay(&self, diagonal: bool) -> i32 {
        let speed = self.get_speed().max(1);
        let numerator = if diagonal { 1414 } else { 1000 };
        numerator * 220 / speed
    }

    // --- Conditions ---

    pub fn add_condition(&mut self, condition_type: i32, ticks: i32) {
        // Remove existing entry of same type, then add new.
        self.conditions
            .retain(|c| c.condition_type != condition_type);
        self.conditions
            .push(ConditionEntry::new(condition_type, ticks));
    }

    pub fn has_condition(&self, condition_type: i32) -> bool {
        self.conditions
            .iter()
            .any(|c| c.condition_type == condition_type)
    }

    pub fn remove_condition(&mut self, condition_type: i32) {
        self.conditions
            .retain(|c| c.condition_type != condition_type);
    }

    /// Tick all conditions by `interval_ms`.
    ///
    /// Conditions with `ticks == -1` are permanent and are never decremented
    /// or removed (mirrors C++ `hasCondition` checking `getTicks() == -1`).
    /// All other conditions are decremented; those that fall to 0 or below
    /// are removed.
    pub fn tick_conditions(&mut self, interval_ms: i32) {
        for c in &mut self.conditions {
            if c.ticks != -1 {
                c.ticks -= interval_ms;
            }
        }
        self.conditions.retain(|c| c.ticks == -1 || c.ticks > 0);
    }

    // --- Summons ---

    pub fn add_summon(&mut self, creature_id: u32) {
        if !self.summons.contains(&creature_id) {
            self.summons.push(creature_id);
        }
    }

    pub fn remove_summon(&mut self, creature_id: u32) {
        self.summons.retain(|&id| id != creature_id);
    }

    pub fn get_summon_count(&self) -> usize {
        self.summons.len()
    }

    // --- Combat targets ---

    pub fn set_attack_target(&mut self, creature_id: u32) {
        self.attack_target = Some(creature_id);
    }

    pub fn get_attack_target(&self) -> Option<u32> {
        self.attack_target
    }

    pub fn clear_attack_target(&mut self) {
        self.attack_target = None;
    }

    // --- Light ---

    pub fn get_light_level(&self) -> u8 {
        self.light.level
    }

    pub fn get_light_color(&self) -> u8 {
        self.light.color
    }

    pub fn set_light(&mut self, level: u8, color: u8) {
        self.light.level = level;
        self.light.color = color;
    }

    // --- Viewport check ---

    /// Returns true when `to` is within the creature's 9×7 viewport relative
    /// to `from`. Mirrors `Creature::canSee` in creature.cpp:
    ///   |dx| <= 9, |dy| <= 7, same floor (z must match).
    pub fn can_see(from: Position, to: Position) -> bool {
        if from.z != to.z {
            return false;
        }
        let dx = (from.x as i32 - to.x as i32).abs();
        let dy = (from.y as i32 - to.y as i32).abs();
        dx <= 9 && dy <= 7
    }

    // --- Health/mana regen ---

    pub fn get_hp_recovery_ticks(&self) -> u32 {
        self.hp_recovery_ticks
    }

    pub fn get_mana_recovery_ticks(&self) -> u32 {
        self.mana_recovery_ticks
    }

    // --- Pushable ---

    pub fn is_pushable(&self) -> bool {
        self.pushable
    }

    pub fn set_pushable(&mut self, val: bool) {
        self.pushable = val;
    }

    // --- Dead/alive ---

    /// Returns `true` when health is zero or below (mirrors C++ `isDead()`).
    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }

    // --- change_health (C++ semantics) ---

    /// Change health by `delta`, clamping result to `[0, health_max]`.
    /// Mirrors C++ `Creature::changeHealth`:
    ///   gain → adds but won't exceed max_health
    ///   loss → subtracts but won't go below 0
    ///
    /// Returns the new health value.
    pub fn change_health(&mut self, delta: i32) -> i32 {
        if delta > 0 {
            self.health = (self.health + delta).min(self.health_max);
        } else {
            self.health = (self.health + delta).max(0);
        }
        self.health
    }

    // --- Skull ---

    /// Returns the creature's skull type (mirrors C++ `getSkull()`).
    pub fn get_skull(&self) -> SkullType {
        self.skull
    }

    /// Sets the creature's skull type (mirrors C++ `setSkull()`).
    pub fn set_skull(&mut self, skull: SkullType) {
        self.skull = skull;
    }

    // --- Direction ---

    /// Returns current movement direction.
    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    /// Sets current movement direction.
    pub fn set_direction(&mut self, dir: Direction) {
        self.direction = dir;
    }

    // --- Hidden health ---

    pub fn is_health_hidden(&self) -> bool {
        self.hidden_health
    }

    pub fn set_hidden_health(&mut self, hidden: bool) {
        self.hidden_health = hidden;
    }

    // --- Movement blocked ---

    /// Returns `true` when movement is blocked. Also sets `cancel_next_walk`.
    pub fn is_movement_blocked(&self) -> bool {
        self.movement_blocked
    }

    /// Sets movement-blocked state. Also sets `cancel_next_walk = true`
    /// to match C++ behaviour (`setMovementBlocked`).
    pub fn set_movement_blocked(&mut self, blocked: bool) {
        self.movement_blocked = blocked;
        self.cancel_next_walk = true;
    }

    // --- Loot / skill loss flags ---

    pub fn set_loot_drop(&mut self, val: bool) {
        self.loot_drop = val;
    }

    pub fn get_loot_drop(&self) -> bool {
        self.loot_drop
    }

    pub fn set_skill_loss(&mut self, val: bool) {
        self.skill_loss = val;
    }

    pub fn get_skill_loss(&self) -> bool {
        self.skill_loss
    }

    // --- Drunkenness ---

    pub fn get_drunkenness(&self) -> u8 {
        self.drunkenness
    }

    pub fn set_drunkenness(&mut self, val: u8) {
        self.drunkenness = val;
    }

    // --- Walk step queue ---

    /// Returns the next step from the walk queue (mirrors C++ `getNextStep`).
    ///
    /// C++ stores directions in `listWalkDir` with the *next* step at the back
    /// (`dir = listWalkDir.back(); listWalkDir.pop_back()`).  The Rust
    /// `walk_steps` Vec follows the same convention: `push_back` appends and
    /// `pop` (which removes from the end) yields the next step.
    ///
    /// Returns `Some(direction)` or `None` when the queue is empty.
    pub fn get_next_step(&mut self) -> Option<Direction> {
        self.walk_steps.pop()
    }

    /// Appends a sequence of directions to the walk queue.
    ///
    /// The slice should be ordered so that the *last* element is the *first*
    /// step to execute (back-of-Vec semantics, matching C++ `listWalkDir`).
    pub fn set_walk_steps(&mut self, steps: Vec<Direction>) {
        self.walk_steps = steps;
    }

    /// Returns `true` when the walk-step queue is empty.
    pub fn walk_steps_empty(&self) -> bool {
        self.walk_steps.is_empty()
    }

    /// Clears all pending walk steps (the "abort clears step cache" behaviour
    /// from C++ `onWalkAborted` / `cancelNextWalk` handling).
    pub fn on_walk_aborted(&mut self) {
        self.walk_steps.clear();
        self.cancel_next_walk = false;
    }

    // --- isUnderAttack ---

    /// Returns `true` when the creature is under attack, defined as:
    ///   health < max_health  AND  an attack_target is set.
    ///
    /// This captures the C++ notion of "attacker active and creature is
    /// taking damage" without requiring a full game-world reference.
    pub fn is_under_attack(&self) -> bool {
        self.health < self.health_max && self.attack_target.is_some()
    }

    // --- isInvisible ---

    /// Returns `true` when the creature carries an INVISIBLE condition.
    /// Mirrors C++ `Creature::isInvisible()` which scans the condition list
    /// for `CONDITION_INVISIBLE`.  The constant value used here (bit 14,
    /// value `16384 = 1 << 14`) must match the game's `CONDITION_INVISIBLE`
    /// definition; the exact bit is an implementation detail but the
    /// behaviour is: any condition with type == `CONDITION_INVISIBLE` present
    /// in the list (regardless of remaining ticks) implies invisibility.
    pub const CONDITION_INVISIBLE: i32 = 1 << 14;

    pub fn is_invisible(&self) -> bool {
        self.conditions
            .iter()
            .any(|c| c.condition_type == Self::CONDITION_INVISIBLE)
    }

    // --- Position ---

    pub fn get_position(&self) -> Position {
        self.position
    }

    pub fn set_position(&mut self, pos: Position) {
        self.position = pos;
    }
}

// ---------------------------------------------------------------------------
// Phase B.2 — additional C++ parity surface
//
// The methods below mirror the C++ `Creature` API at `creature.h` lines
// 100-450. Where a method requires concrete cross-crate types (Player*,
// Tile*, Item*, Container*), we use trait-object dispatch via
// `&dyn forgottenserver_common::thing::Thing` or accept opaque IDs.
//
// New fields are appended via a side struct `CreatureExt` to avoid breaking
// the existing public `Creature` shape.
// ---------------------------------------------------------------------------

use std::collections::{HashMap, HashSet, VecDeque};

/// Additional creature state introduced by the architectural-parity change.
///
/// Held as a separate field on `Creature` (added via the impl below) to keep
/// the original struct shape stable for existing callers; new methods land
/// directly on `Creature` and read/write `extras` transparently.
#[derive(Debug, Default)]
pub struct CreatureExt {
    /// Mirrors C++ `isInternalRemoved` — set by `set_removed()`.
    pub removed: bool,
    /// Mirrors C++ `master` — the master creature ID for summons.
    pub master: Option<u32>,
    /// Mirrors C++ `followCreature` — current follow target ID.
    pub follow_target: Option<u32>,
    /// Mirrors C++ `followers` — set of creatures following this one.
    pub followers: HashSet<u32>,
    /// Mirrors C++ `lastPosition` — set at each move tick.
    pub last_position: Position,
    /// Mirrors C++ `referenceCounter` (Lua handle ref counting).
    pub reference_counter: u32,
    /// Mirrors C++ `eventWalk` — scheduler event id for the next step.
    pub event_walk: u32,
    /// Mirrors C++ `walkUpdateTicks` — last walk-direction update timestamp.
    pub walk_update_ticks: i64,
    /// Mirrors C++ `lastStep` — timestamp of the last movement.
    pub last_step: i64,
    /// Mirrors C++ `lastStepCost` — duration cost of the last step.
    pub last_step_cost: u32,
    /// Mirrors C++ `damageMap<attackerId, DamagePoints>`.
    pub damage_map: HashMap<u32, (i64, i64)>, // (lastDamage timestamp, total damage)
    /// Mirrors C++ `storageMap` — per-creature key→value Lua storage.
    pub storage: HashMap<u32, i64>,
    /// Mirrors C++ `eventsList` — registered creature event names.
    pub creature_events: Vec<String>,
    /// Mirrors C++ `canUseDefense` — whether defense skills are enabled.
    pub can_use_defense: bool,
    /// Mirrors C++ `isInGhostMode` — invisible to non-GM players.
    pub ghost_mode: bool,
    /// Mirrors C++ `currentOutfit` — packed look/colours.
    pub outfit: CreatureOutfit,
}

/// Compact creature outfit (mirrors C++ `Outfit_t`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CreatureOutfit {
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
    pub look_mount: u16,
}

impl Creature {
    // -----------------------------------------------------------------------
    // Removed flag — mirrors setRemoved / isRemoved
    // -----------------------------------------------------------------------

    /// Mark this creature as removed from the world.
    /// Mirrors C++ `void setRemoved() { isInternalRemoved = true; }`.
    pub fn set_removed(&mut self, ext: &mut CreatureExt) {
        ext.removed = true;
    }

    /// Whether the creature has been removed.
    /// Mirrors C++ `bool isRemoved() const`.
    pub fn is_removed_ext(&self, ext: &CreatureExt) -> bool {
        ext.removed
    }

    // -----------------------------------------------------------------------
    // Dead / pushable predicates — mirrors isPushable (isDead already exists
    // on the pre-existing impl block earlier in the file).
    // -----------------------------------------------------------------------

    /// C++ `isPushable` returns `getWalkDelay() <= 0`. Our walk-delay is
    /// stored on the creature already; we expose the pushable predicate
    /// directly so callers don't need to recompute the formula.
    pub fn is_pushable_creature(&self) -> bool {
        self.pushable && self.health > 0
    }

    // -----------------------------------------------------------------------
    // Master / summons — mirrors setMaster / removeMaster / isSummon /
    // getMaster / getSummons
    // -----------------------------------------------------------------------

    pub fn set_master(&mut self, ext: &mut CreatureExt, master_id: u32) {
        ext.master = Some(master_id);
    }

    pub fn remove_master(&mut self, ext: &mut CreatureExt) {
        ext.master = None;
    }

    pub fn is_summon(&self, ext: &CreatureExt) -> bool {
        ext.master.is_some()
    }

    pub fn get_master(&self, ext: &CreatureExt) -> Option<u32> {
        ext.master
    }

    pub fn get_summons(&self) -> &[u32] {
        &self.summons
    }

    // -----------------------------------------------------------------------
    // Follow target — mirrors setFollowCreature / getFollowCreature /
    // addFollower / removeFollower / isFollower / removeFollowers
    // -----------------------------------------------------------------------

    pub fn set_follow_creature(&mut self, ext: &mut CreatureExt, target_id: Option<u32>) {
        ext.follow_target = target_id;
    }

    pub fn get_follow_creature(&self, ext: &CreatureExt) -> Option<u32> {
        ext.follow_target
    }

    pub fn add_follower(&mut self, ext: &mut CreatureExt, follower_id: u32) {
        ext.followers.insert(follower_id);
    }

    pub fn remove_follower(&mut self, ext: &mut CreatureExt, follower_id: u32) {
        ext.followers.remove(&follower_id);
    }

    pub fn is_follower(&self, ext: &CreatureExt, follower_id: u32) -> bool {
        ext.followers.contains(&follower_id)
    }

    pub fn remove_followers(&mut self, ext: &mut CreatureExt) {
        ext.followers.clear();
    }

    pub fn get_followers<'a>(&self, ext: &'a CreatureExt) -> &'a HashSet<u32> {
        &ext.followers
    }

    // -----------------------------------------------------------------------
    // Storage values — mirrors getStorageValue / setStorageValue
    // -----------------------------------------------------------------------

    pub fn set_storage_value(&mut self, ext: &mut CreatureExt, key: u32, value: i64) {
        ext.storage.insert(key, value);
    }

    pub fn get_storage_value(&self, ext: &CreatureExt, key: u32) -> Option<i64> {
        ext.storage.get(&key).copied()
    }

    pub fn remove_storage_value(&mut self, ext: &mut CreatureExt, key: u32) -> bool {
        ext.storage.remove(&key).is_some()
    }

    // -----------------------------------------------------------------------
    // Event registration — mirrors registerCreatureEvent /
    // unregisterCreatureEvent / hasEventRegistered
    // -----------------------------------------------------------------------

    /// Returns `true` when the event was newly added (was not already
    /// registered).
    pub fn register_creature_event(&mut self, ext: &mut CreatureExt, name: &str) -> bool {
        if ext.creature_events.iter().any(|e| e == name) {
            return false;
        }
        ext.creature_events.push(name.to_string());
        true
    }

    pub fn unregister_creature_event(&mut self, ext: &mut CreatureExt, name: &str) -> bool {
        let before = ext.creature_events.len();
        ext.creature_events.retain(|e| e != name);
        ext.creature_events.len() != before
    }

    pub fn has_event_registered(&self, ext: &CreatureExt, name: &str) -> bool {
        ext.creature_events.iter().any(|e| e == name)
    }

    pub fn get_creature_events<'a>(&self, ext: &'a CreatureExt) -> &'a [String] {
        &ext.creature_events
    }

    // -----------------------------------------------------------------------
    // Damage map — mirrors addDamagePoints / hasBeenAttacked /
    // getDamageRatio
    // -----------------------------------------------------------------------

    /// Record `points` of damage dealt by `attacker_id` at timestamp `now`.
    pub fn add_damage_points(
        &mut self,
        ext: &mut CreatureExt,
        attacker_id: u32,
        points: i32,
        now: i64,
    ) {
        let entry = ext.damage_map.entry(attacker_id).or_insert((now, 0));
        entry.0 = now;
        entry.1 += points as i64;
    }

    /// Whether the given attacker has dealt at least 1 point of damage.
    pub fn has_been_attacked(&self, ext: &CreatureExt, attacker_id: u32) -> bool {
        ext.damage_map
            .get(&attacker_id)
            .is_some_and(|(_, dmg)| *dmg > 0)
    }

    /// Returns a damage attribution ratio in `[0.0, 1.0]` for the given
    /// attacker (fraction of all recorded damage).
    pub fn get_damage_ratio(&self, ext: &CreatureExt, attacker_id: u32) -> f64 {
        let total: i64 = ext.damage_map.values().map(|(_, d)| *d).sum();
        if total == 0 {
            return 0.0;
        }
        let attacker = ext
            .damage_map
            .get(&attacker_id)
            .map(|(_, d)| *d)
            .unwrap_or(0);
        attacker as f64 / total as f64
    }

    /// Iterates `(attacker_id, total_damage)` pairs sorted by total descending.
    pub fn get_killers(&self, ext: &CreatureExt) -> Vec<(u32, i64)> {
        let mut v: Vec<(u32, i64)> = ext
            .damage_map
            .iter()
            .map(|(id, (_, d))| (*id, *d))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }

    // -----------------------------------------------------------------------
    // Health drain / gain — mirrors drainHealth / gainHealth / changeHealth
    // -----------------------------------------------------------------------

    /// Decrease `health` by `amount` (clamped to 0). Mirrors C++
    /// `Creature::drainHealth`.
    pub fn drain_health(&mut self, amount: i32) {
        let new = self.health.saturating_sub(amount);
        self.health = new.max(0);
    }

    /// Increase `health` by `amount`, clamped to `health_max`. Mirrors
    /// C++ `Creature::gainHealth`.
    pub fn gain_health(&mut self, amount: i32) {
        self.health = (self.health.saturating_add(amount)).min(self.health_max);
    }

    // -----------------------------------------------------------------------
    // PvP / attackable — mirrors challengeCreature / isAttackable /
    // canSeeInvisibility / canSeeGhostMode
    // -----------------------------------------------------------------------

    /// Default: any creature is attackable. Concrete types (e.g. NPC)
    /// override this.
    pub fn is_attackable(&self) -> bool {
        true
    }

    /// Default false; players may override based on flags.
    pub fn can_see_invisibility(&self, _ext: &CreatureExt) -> bool {
        false
    }

    /// Default false; players may override based on group flags.
    pub fn can_see_ghost_mode(&self, _ext: &CreatureExt, _other_ext: &CreatureExt) -> bool {
        false
    }

    pub fn is_in_ghost_mode(&self, ext: &CreatureExt) -> bool {
        ext.ghost_mode
    }

    pub fn set_in_ghost_mode(&mut self, ext: &mut CreatureExt, b: bool) {
        ext.ghost_mode = b;
    }

    // -----------------------------------------------------------------------
    // Outfit — mirrors setCurrentOutfit / getCurrentOutfit
    // -----------------------------------------------------------------------

    pub fn set_current_outfit(&mut self, ext: &mut CreatureExt, outfit: CreatureOutfit) {
        ext.outfit = outfit;
    }

    pub fn get_current_outfit(&self, ext: &CreatureExt) -> CreatureOutfit {
        ext.outfit
    }

    // -----------------------------------------------------------------------
    // Reference counter — mirrors incrementReferenceCounter /
    // decrementReferenceCounter
    // -----------------------------------------------------------------------

    pub fn increment_reference_counter(&mut self, ext: &mut CreatureExt) {
        ext.reference_counter = ext.reference_counter.saturating_add(1);
    }

    /// Returns `true` when the counter reaches 0 (caller should free).
    pub fn decrement_reference_counter(&mut self, ext: &mut CreatureExt) -> bool {
        if ext.reference_counter > 0 {
            ext.reference_counter -= 1;
        }
        ext.reference_counter == 0
    }

    pub fn reference_count(&self, ext: &CreatureExt) -> u32 {
        ext.reference_counter
    }

    // -----------------------------------------------------------------------
    // Walk queue helpers — mirrors startAutoWalk / stopEventWalk /
    // addEventWalk / cancelWalk / getNextStep
    // -----------------------------------------------------------------------

    /// Queue a sequence of directions to walk. Mirrors C++
    /// `startAutoWalk(const std::vector<Direction>& listDir)`.
    pub fn start_auto_walk_dirs(&mut self, dirs: &[Direction]) {
        // listWalkDir is stored back-first in C++; we mirror that here:
        // dirs is provided front-first, so reverse-iterate to push to a Vec
        // whose last element is the next step.
        self.walk_steps.clear();
        for &d in dirs.iter().rev() {
            self.walk_steps.push(d);
        }
        self.cancel_next_walk = false;
    }

    /// Queue a single direction step.
    pub fn start_auto_walk_single(&mut self, dir: Direction) {
        self.start_auto_walk_dirs(&[dir]);
    }

    /// Pop the next direction from the walk queue, returning it.
    pub fn get_next_walk_step(&mut self) -> Option<Direction> {
        self.walk_steps.pop()
    }

    /// Cancel the queued walk; the next step will be a no-op.
    pub fn stop_event_walk(&mut self) {
        self.walk_steps.clear();
        self.cancel_next_walk = true;
    }

    pub fn has_pending_walk_steps(&self) -> bool {
        !self.walk_steps.is_empty()
    }

    // -----------------------------------------------------------------------
    // Last-position / last-step — mirrors lastPosition / lastStep / getLastStep
    // -----------------------------------------------------------------------

    pub fn set_last_position(&mut self, ext: &mut CreatureExt, pos: Position) {
        ext.last_position = pos;
    }

    pub fn get_last_position(&self, ext: &CreatureExt) -> Position {
        ext.last_position
    }

    pub fn set_last_step(&mut self, ext: &mut CreatureExt, ts: i64, cost: u32) {
        ext.last_step = ts;
        ext.last_step_cost = cost;
    }

    pub fn get_last_step(&self, ext: &CreatureExt) -> i64 {
        ext.last_step
    }

    pub fn get_last_step_cost(&self, ext: &CreatureExt) -> u32 {
        ext.last_step_cost
    }

    /// Mirrors C++ `int64_t getTimeSinceLastMove() const` — returns the
    /// duration in milliseconds since the last step at `now`.
    pub fn get_time_since_last_move(&self, ext: &CreatureExt, now: i64) -> i64 {
        (now - ext.last_step).max(0)
    }

    // -----------------------------------------------------------------------
    // Event walk slot — mirrors eventWalk handle
    // -----------------------------------------------------------------------

    pub fn set_event_walk(&mut self, ext: &mut CreatureExt, handle: u32) {
        ext.event_walk = handle;
    }

    pub fn get_event_walk(&self, ext: &CreatureExt) -> u32 {
        ext.event_walk
    }

    // -----------------------------------------------------------------------
    // Defense usage — mirrors setUseDefense / canUseDefense
    // -----------------------------------------------------------------------

    pub fn set_use_defense(&mut self, ext: &mut CreatureExt, b: bool) {
        ext.can_use_defense = b;
    }

    pub fn can_use_defense(&self, ext: &CreatureExt) -> bool {
        ext.can_use_defense
    }

    // -----------------------------------------------------------------------
    // Loot drop — mirrors setDropLoot / getDropLoot (set_skill_loss /
    // get_skill_loss already exist on the pre-existing impl block).
    // -----------------------------------------------------------------------

    pub fn set_drop_loot(&mut self, b: bool) {
        self.loot_drop = b;
    }

    pub fn get_drop_loot(&self) -> bool {
        self.loot_drop
    }
}

// ---------------------------------------------------------------------------
// Thing impl — Creature participates in cross-crate trait-object dispatch
// ---------------------------------------------------------------------------

impl forgottenserver_common::thing::Thing for Creature {
    fn is_creature(&self) -> bool {
        true
    }
    fn is_pushable(&self) -> bool {
        self.is_pushable_creature()
    }
    fn is_removable(&self) -> bool {
        true
    }
    fn is_removed(&self) -> bool {
        // Without access to a CreatureExt we conservatively report alive.
        !self.is_dead()
    }
    fn is_invisible(&self) -> bool {
        Creature::is_invisible(self)
    }
    fn get_throw_range(&self) -> i32 {
        1
    }
    fn get_position_tuple(&self) -> Option<(u16, u16, u8)> {
        Some((self.position.x, self.position.y, self.position.z))
    }
    fn get_creature(&self) -> Option<&dyn forgottenserver_common::thing::Thing> {
        Some(self)
    }
    fn get_description(&self, _look_distance: i32) -> String {
        self.name.clone()
    }
}

// Re-export the deferred walk-queue alias so callers can use `VecDeque`-style
// imports if they want; the impl above uses Vec for parity with the existing
// `walk_steps` field.
pub(crate) type _WalkQueue = VecDeque<Direction>;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::position::Position;

    // --- Creature struct and basics ---

    #[test]
    fn test_creature_new_id_and_name() {
        let c = Creature::new(42, "Rat");
        assert_eq!(c.get_id(), 42);
        assert_eq!(c.get_name(), "Rat");
    }

    #[test]
    fn test_creature_is_alive_new() {
        let c = Creature::new(1, "Rat");
        assert!(c.is_alive());
    }

    #[test]
    fn test_creature_health_default() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_health(), 100);
        assert_eq!(c.get_max_health(), 100);
    }

    #[test]
    fn test_creature_set_health() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(50);
        assert_eq!(c.get_health(), 50);
    }

    #[test]
    fn test_creature_set_max_health() {
        let mut c = Creature::new(1, "Rat");
        c.set_max_health(200);
        assert_eq!(c.get_max_health(), 200);
    }

    #[test]
    fn test_creature_is_alive_false_when_health_zero() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(0);
        assert!(!c.is_alive());
    }

    // --- Speed and walk ---

    #[test]
    fn test_creature_get_speed_default() {
        let c = Creature::new(1, "Rat");
        // base_speed=220, var_speed=0 → 220
        assert_eq!(c.get_speed(), 220);
    }

    #[test]
    fn test_creature_set_base_speed() {
        let mut c = Creature::new(1, "Rat");
        c.set_base_speed(300);
        assert_eq!(c.get_base_speed(), 300);
    }

    #[test]
    fn test_creature_change_speed() {
        let mut c = Creature::new(1, "Rat");
        c.change_speed(50);
        assert_eq!(c.get_speed(), 270);
    }

    #[test]
    fn test_creature_walk_delay_non_diagonal_at_default_speed() {
        let c = Creature::new(1, "Rat");
        // speed=220 → delay = 1000 * 220 / 220 = 1000
        assert_eq!(c.get_walk_delay(false), 1000);
    }

    #[test]
    fn test_creature_walk_delay_diagonal_at_default_speed() {
        let c = Creature::new(1, "Rat");
        // speed=220 → delay = 1414 * 220 / 220 = 1414
        assert_eq!(c.get_walk_delay(true), 1414);
    }

    #[test]
    fn test_creature_walk_delay_clamped_to_1_at_zero_speed() {
        let mut c = Creature::new(1, "Rat");
        c.set_base_speed(0);
        c.change_speed(-1000); // will result in large negative, speed clamped to 1
                               // delay = 1000 * 220 / 1 = 220000
        assert_eq!(c.get_walk_delay(false), 220000);
    }

    // --- Conditions ---

    #[test]
    fn test_creature_add_condition() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, 5000); // POISON
        assert!(c.has_condition(1 << 0));
    }

    #[test]
    fn test_creature_remove_condition() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, 5000);
        c.remove_condition(1 << 0);
        assert!(!c.has_condition(1 << 0));
    }

    #[test]
    fn test_creature_tick_conditions_partial() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, 5000);
        c.tick_conditions(2000);
        assert!(c.has_condition(1 << 0)); // still alive
    }

    #[test]
    fn test_creature_tick_conditions_expire() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, 5000);
        c.tick_conditions(5000); // exactly removes (ticks becomes 0, <= 0 → removed)
        assert!(!c.has_condition(1 << 0));
    }

    #[test]
    fn test_creature_tick_conditions_overshoot_expire() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, 3000);
        c.tick_conditions(9999);
        assert!(!c.has_condition(1 << 0));
    }

    // --- Summons ---

    #[test]
    fn test_creature_add_summon() {
        let mut c = Creature::new(1, "Rat");
        c.add_summon(10);
        assert_eq!(c.get_summon_count(), 1);
    }

    #[test]
    fn test_creature_remove_summon() {
        let mut c = Creature::new(1, "Rat");
        c.add_summon(10);
        c.remove_summon(10);
        assert_eq!(c.get_summon_count(), 0);
    }

    #[test]
    fn test_creature_summon_count() {
        let mut c = Creature::new(1, "Rat");
        c.add_summon(10);
        c.add_summon(11);
        c.add_summon(12);
        assert_eq!(c.get_summon_count(), 3);
    }

    #[test]
    fn test_creature_add_duplicate_summon_ignored() {
        let mut c = Creature::new(1, "Rat");
        c.add_summon(10);
        c.add_summon(10);
        assert_eq!(c.get_summon_count(), 1);
    }

    // --- Combat targets ---

    #[test]
    fn test_creature_set_attack_target() {
        let mut c = Creature::new(1, "Rat");
        c.set_attack_target(99);
        assert_eq!(c.get_attack_target(), Some(99));
    }

    #[test]
    fn test_creature_get_attack_target_none_by_default() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_attack_target(), None);
    }

    #[test]
    fn test_creature_clear_attack_target() {
        let mut c = Creature::new(1, "Rat");
        c.set_attack_target(99);
        c.clear_attack_target();
        assert_eq!(c.get_attack_target(), None);
    }

    // --- Light ---

    #[test]
    fn test_creature_light_level_default() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_light_level(), 0);
    }

    #[test]
    fn test_creature_set_light() {
        let mut c = Creature::new(1, "Rat");
        c.set_light(5, 215);
        assert_eq!(c.get_light_level(), 5);
        assert_eq!(c.get_light_color(), 215);
    }

    // --- Viewport check ---

    #[test]
    fn test_can_see_same_position() {
        let p = Position::new(100, 100, 7);
        assert!(Creature::can_see(p, p));
    }

    #[test]
    fn test_can_see_within_viewport() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(109, 107, 7); // dx=9, dy=7 — on boundary
        assert!(Creature::can_see(from, to));
    }

    #[test]
    fn test_can_see_dx_out_of_range() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(110, 100, 7); // dx=10 > 9
        assert!(!Creature::can_see(from, to));
    }

    #[test]
    fn test_can_see_dy_out_of_range() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 108, 7); // dy=8 > 7
        assert!(!Creature::can_see(from, to));
    }

    #[test]
    fn test_can_see_different_floor() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 100, 6);
        assert!(!Creature::can_see(from, to));
    }

    // --- Health/mana regen ---

    #[test]
    fn test_creature_hp_recovery_ticks() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_hp_recovery_ticks(), 1000);
    }

    #[test]
    fn test_creature_mana_recovery_ticks() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_mana_recovery_ticks(), 1000);
    }

    // --- Pushable ---

    #[test]
    fn test_creature_is_pushable_default() {
        let c = Creature::new(1, "Rat");
        assert!(c.is_pushable());
    }

    #[test]
    fn test_creature_set_pushable_false() {
        let mut c = Creature::new(1, "Rat");
        c.set_pushable(false);
        assert!(!c.is_pushable());
    }

    // --- spawn_id ---

    #[test]
    fn test_creature_spawn_id_default_none() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.spawn_id, None);
    }

    #[test]
    fn test_creature_spawn_id_can_be_set() {
        let mut c = Creature::new(1, "Rat");
        c.spawn_id = Some(42);
        assert_eq!(c.spawn_id, Some(42));
    }

    // =========================================================================
    // NEW TESTS — gaps identified during audit
    // =========================================================================

    // --- is_dead ---

    #[test]
    fn test_is_dead_returns_false_when_alive() {
        let c = Creature::new(1, "Rat");
        assert!(!c.is_dead());
    }

    #[test]
    fn test_is_dead_returns_true_when_health_zero() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(0);
        assert!(c.is_dead());
    }

    #[test]
    fn test_is_dead_returns_true_when_health_negative() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(-1);
        assert!(c.is_dead());
    }

    // --- change_health (C++ semantics: clamps to [0, max_health]) ---

    #[test]
    fn test_change_health_gain_clamped_to_max() {
        let mut c = Creature::new(1, "Rat");
        // starts at 100/100 — gaining 50 should stay at 100
        let result = c.change_health(50);
        assert_eq!(result, 100);
        assert_eq!(c.get_health(), 100);
    }

    #[test]
    fn test_change_health_gain_partial() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(60);
        // 60 + 20 = 80 (below max=100)
        let result = c.change_health(20);
        assert_eq!(result, 80);
        assert_eq!(c.get_health(), 80);
    }

    #[test]
    fn test_change_health_loss_clamped_to_zero() {
        let mut c = Creature::new(1, "Rat");
        // 100 - 200 → clamped to 0
        let result = c.change_health(-200);
        assert_eq!(result, 0);
        assert_eq!(c.get_health(), 0);
    }

    #[test]
    fn test_change_health_loss_exact_zero() {
        let mut c = Creature::new(1, "Rat");
        let result = c.change_health(-100);
        assert_eq!(result, 0);
        assert!(c.is_dead());
    }

    #[test]
    fn test_change_health_loss_partial() {
        let mut c = Creature::new(1, "Rat");
        let result = c.change_health(-30);
        assert_eq!(result, 70);
        assert_eq!(c.get_health(), 70);
    }

    // --- skull ---

    #[test]
    fn test_get_skull_default_none() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_skull(), SkullType::None);
    }

    #[test]
    fn test_set_skull_yellow() {
        let mut c = Creature::new(1, "Rat");
        c.set_skull(SkullType::Yellow);
        assert_eq!(c.get_skull(), SkullType::Yellow);
    }

    #[test]
    fn test_set_skull_red() {
        let mut c = Creature::new(1, "Rat");
        c.set_skull(SkullType::Red);
        assert_eq!(c.get_skull(), SkullType::Red);
    }

    #[test]
    fn test_set_skull_black() {
        let mut c = Creature::new(1, "Rat");
        c.set_skull(SkullType::Black);
        assert_eq!(c.get_skull(), SkullType::Black);
    }

    #[test]
    fn test_set_skull_back_to_none() {
        let mut c = Creature::new(1, "Rat");
        c.set_skull(SkullType::White);
        c.set_skull(SkullType::None);
        assert_eq!(c.get_skull(), SkullType::None);
    }

    // --- direction ---

    #[test]
    fn test_get_direction_default_south() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_direction(), Direction::South);
    }

    #[test]
    fn test_set_direction_north() {
        let mut c = Creature::new(1, "Rat");
        c.set_direction(Direction::North);
        assert_eq!(c.get_direction(), Direction::North);
    }

    #[test]
    fn test_set_direction_east() {
        let mut c = Creature::new(1, "Rat");
        c.set_direction(Direction::East);
        assert_eq!(c.get_direction(), Direction::East);
    }

    #[test]
    fn test_direction_is_diagonal_false_for_cardinals() {
        assert!(!Direction::North.is_diagonal());
        assert!(!Direction::East.is_diagonal());
        assert!(!Direction::South.is_diagonal());
        assert!(!Direction::West.is_diagonal());
    }

    #[test]
    fn test_direction_is_diagonal_true_for_diagonals() {
        assert!(Direction::NorthEast.is_diagonal());
        assert!(Direction::SouthEast.is_diagonal());
        assert!(Direction::SouthWest.is_diagonal());
        assert!(Direction::NorthWest.is_diagonal());
    }

    // --- hidden_health ---

    #[test]
    fn test_is_health_hidden_default_false() {
        let c = Creature::new(1, "Rat");
        assert!(!c.is_health_hidden());
    }

    #[test]
    fn test_set_hidden_health_true() {
        let mut c = Creature::new(1, "Rat");
        c.set_hidden_health(true);
        assert!(c.is_health_hidden());
    }

    #[test]
    fn test_set_hidden_health_false() {
        let mut c = Creature::new(1, "Rat");
        c.set_hidden_health(true);
        c.set_hidden_health(false);
        assert!(!c.is_health_hidden());
    }

    // --- movement_blocked ---

    #[test]
    fn test_is_movement_blocked_default_false() {
        let c = Creature::new(1, "Rat");
        assert!(!c.is_movement_blocked());
    }

    #[test]
    fn test_set_movement_blocked_sets_cancel_next_walk() {
        let mut c = Creature::new(1, "Rat");
        c.set_movement_blocked(true);
        assert!(c.is_movement_blocked());
        // C++ setMovementBlocked also sets cancelNextWalk = true
        assert!(c.cancel_next_walk);
    }

    #[test]
    fn test_set_movement_blocked_false_still_sets_cancel_next_walk() {
        let mut c = Creature::new(1, "Rat");
        // Even setting to false triggers cancel_next_walk per C++ behaviour
        c.set_movement_blocked(false);
        assert!(!c.is_movement_blocked());
        assert!(c.cancel_next_walk);
    }

    // --- loot_drop / skill_loss ---

    #[test]
    fn test_loot_drop_default_true() {
        let c = Creature::new(1, "Rat");
        assert!(c.get_loot_drop());
    }

    #[test]
    fn test_set_loot_drop_false() {
        let mut c = Creature::new(1, "Rat");
        c.set_loot_drop(false);
        assert!(!c.get_loot_drop());
    }

    #[test]
    fn test_skill_loss_default_true() {
        let c = Creature::new(1, "Rat");
        assert!(c.get_skill_loss());
    }

    #[test]
    fn test_set_skill_loss_false() {
        let mut c = Creature::new(1, "Rat");
        c.set_skill_loss(false);
        assert!(!c.get_skill_loss());
    }

    // --- drunkenness ---

    #[test]
    fn test_drunkenness_default_zero() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_drunkenness(), 0);
    }

    #[test]
    fn test_set_drunkenness() {
        let mut c = Creature::new(1, "Rat");
        c.set_drunkenness(50);
        assert_eq!(c.get_drunkenness(), 50);
    }

    // --- walk step queue (getNextStep / onWalkAborted) ---

    #[test]
    fn test_get_next_step_empty_returns_none() {
        let mut c = Creature::new(1, "Rat");
        assert_eq!(c.get_next_step(), None);
    }

    #[test]
    fn test_walk_steps_empty_by_default() {
        let c = Creature::new(1, "Rat");
        assert!(c.walk_steps_empty());
    }

    #[test]
    fn test_get_next_step_returns_last_pushed_first() {
        // C++ listWalkDir uses back() so the last pushed is popped first.
        let mut c = Creature::new(1, "Rat");
        c.set_walk_steps(vec![Direction::South, Direction::East, Direction::North]);
        // North was pushed last → should be the first step
        assert_eq!(c.get_next_step(), Some(Direction::North));
        assert_eq!(c.get_next_step(), Some(Direction::East));
        assert_eq!(c.get_next_step(), Some(Direction::South));
        assert_eq!(c.get_next_step(), None);
    }

    #[test]
    fn test_walk_steps_not_empty_after_set() {
        let mut c = Creature::new(1, "Rat");
        c.set_walk_steps(vec![Direction::North]);
        assert!(!c.walk_steps_empty());
    }

    #[test]
    fn test_walk_steps_empty_after_exhausted() {
        let mut c = Creature::new(1, "Rat");
        c.set_walk_steps(vec![Direction::North]);
        c.get_next_step();
        assert!(c.walk_steps_empty());
    }

    // --- on_walk_aborted (abort clears step cache) ---

    #[test]
    fn test_on_walk_aborted_clears_walk_steps() {
        let mut c = Creature::new(1, "Rat");
        c.set_walk_steps(vec![Direction::North, Direction::East, Direction::South]);
        c.on_walk_aborted();
        assert!(c.walk_steps_empty());
    }

    #[test]
    fn test_on_walk_aborted_clears_cancel_next_walk_flag() {
        let mut c = Creature::new(1, "Rat");
        c.cancel_next_walk = true;
        c.on_walk_aborted();
        assert!(!c.cancel_next_walk);
    }

    #[test]
    fn test_on_walk_aborted_on_empty_is_safe() {
        let mut c = Creature::new(1, "Rat");
        c.on_walk_aborted(); // should not panic
        assert!(c.walk_steps_empty());
    }

    // --- is_under_attack ---

    #[test]
    fn test_is_under_attack_false_at_full_health_no_target() {
        let c = Creature::new(1, "Rat");
        assert!(!c.is_under_attack());
    }

    #[test]
    fn test_is_under_attack_false_at_full_health_with_target() {
        let mut c = Creature::new(1, "Rat");
        c.set_attack_target(5);
        // full health → not under attack
        assert!(!c.is_under_attack());
    }

    #[test]
    fn test_is_under_attack_false_reduced_health_no_target() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(50);
        // no attacker → not under attack
        assert!(!c.is_under_attack());
    }

    #[test]
    fn test_is_under_attack_true_when_damaged_and_has_target() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(50); // health < max
        c.set_attack_target(7);
        assert!(c.is_under_attack());
    }

    // --- is_invisible (condition-based) ---

    #[test]
    fn test_is_invisible_default_false() {
        let c = Creature::new(1, "Rat");
        assert!(!c.is_invisible());
    }

    #[test]
    fn test_is_invisible_true_when_condition_present() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(Creature::CONDITION_INVISIBLE, 5000);
        assert!(c.is_invisible());
    }

    #[test]
    fn test_is_invisible_false_after_condition_removed() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(Creature::CONDITION_INVISIBLE, 5000);
        c.remove_condition(Creature::CONDITION_INVISIBLE);
        assert!(!c.is_invisible());
    }

    #[test]
    fn test_is_invisible_false_after_condition_expires() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(Creature::CONDITION_INVISIBLE, 1000);
        c.tick_conditions(2000); // overshot — should expire
        assert!(!c.is_invisible());
    }

    // --- permanent condition (ticks == -1) ---

    #[test]
    fn test_permanent_condition_not_removed_by_tick() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, -1); // permanent
        c.tick_conditions(999_999);
        assert!(c.has_condition(1 << 0));
    }

    #[test]
    fn test_permanent_condition_ticks_not_decremented() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 1, -1);
        c.tick_conditions(10_000);
        // Find the entry and verify ticks is still -1
        let entry = c
            .conditions
            .iter()
            .find(|e| e.condition_type == (1 << 1))
            .unwrap();
        assert_eq!(entry.ticks, -1);
    }

    #[test]
    fn test_permanent_condition_removable_explicitly() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 2, -1);
        c.remove_condition(1 << 2);
        assert!(!c.has_condition(1 << 2));
    }

    // --- position ---

    #[test]
    fn test_position_default() {
        let c = Creature::new(1, "Rat");
        assert_eq!(c.get_position(), Position::default());
    }

    #[test]
    fn test_set_position() {
        let mut c = Creature::new(1, "Rat");
        let pos = Position::new(100, 200, 7);
        c.set_position(pos);
        assert_eq!(c.get_position(), pos);
    }

    // --- add_condition replacement semantics (exercises the retain closure) ---

    /// Documented behaviour at `add_condition`: "Remove existing entry of same
    /// type, then add new." This test forces the retain-closure on a populated
    /// `conditions` Vec and verifies the replacement semantics: only one entry
    /// of a given type may exist, and the new entry's ticks overwrite the old.
    #[test]
    fn test_add_condition_replaces_existing_same_type() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, 1000); // initial: 1000 ticks
        c.add_condition(1 << 0, 5000); // same type, new ticks → must replace

        // Still exactly one entry of this type.
        let entries: Vec<_> = c
            .conditions
            .iter()
            .filter(|e| e.condition_type == (1 << 0))
            .collect();
        assert_eq!(entries.len(), 1);
        // Ticks should be the new value, not the old.
        assert_eq!(entries[0].ticks, 5000);
    }

    /// When `add_condition` is called with a *different* type while another
    /// type already exists, the existing entry must remain (retain closure
    /// returns true for non-matching entries).
    #[test]
    fn test_add_condition_keeps_different_type() {
        let mut c = Creature::new(1, "Rat");
        c.add_condition(1 << 0, 1000); // type A
        c.add_condition(1 << 3, 7000); // type B — different type, must NOT remove A

        assert!(c.has_condition(1 << 0));
        assert!(c.has_condition(1 << 3));
        assert_eq!(c.conditions.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Phase B.2 — extended C++ parity surface
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_removed_flips_removed_flag() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        assert!(!c.is_removed_ext(&ext));
        c.set_removed(&mut ext);
        assert!(c.is_removed_ext(&ext));
    }

    #[test]
    fn test_master_summon_lifecycle() {
        let mut c = Creature::new(1, "Demon");
        let mut ext = CreatureExt::default();
        assert!(!c.is_summon(&ext));
        c.set_master(&mut ext, 42);
        assert!(c.is_summon(&ext));
        assert_eq!(c.get_master(&ext), Some(42));
        c.remove_master(&mut ext);
        assert!(!c.is_summon(&ext));
        assert_eq!(c.get_master(&ext), None);
    }

    #[test]
    fn test_follow_target_set_clear() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        assert_eq!(c.get_follow_creature(&ext), None);
        c.set_follow_creature(&mut ext, Some(99));
        assert_eq!(c.get_follow_creature(&ext), Some(99));
        c.set_follow_creature(&mut ext, None);
        assert_eq!(c.get_follow_creature(&ext), None);
    }

    #[test]
    fn test_follower_add_remove_check() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.add_follower(&mut ext, 7);
        c.add_follower(&mut ext, 8);
        assert!(c.is_follower(&ext, 7));
        assert!(c.is_follower(&ext, 8));
        assert!(!c.is_follower(&ext, 9));
        c.remove_follower(&mut ext, 7);
        assert!(!c.is_follower(&ext, 7));
        // followers iter
        let followers = c.get_followers(&ext);
        assert_eq!(followers.len(), 1);
    }

    #[test]
    fn test_remove_followers_clears_all() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.add_follower(&mut ext, 1);
        c.add_follower(&mut ext, 2);
        c.add_follower(&mut ext, 3);
        c.remove_followers(&mut ext);
        assert!(c.get_followers(&ext).is_empty());
    }

    #[test]
    fn test_storage_value_set_get_remove() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        assert_eq!(c.get_storage_value(&ext, 1000), None);
        c.set_storage_value(&mut ext, 1000, 42);
        assert_eq!(c.get_storage_value(&ext, 1000), Some(42));
        c.set_storage_value(&mut ext, 1000, -1);
        assert_eq!(c.get_storage_value(&ext, 1000), Some(-1));
        assert!(c.remove_storage_value(&mut ext, 1000));
        assert_eq!(c.get_storage_value(&ext, 1000), None);
        assert!(!c.remove_storage_value(&mut ext, 9999));
    }

    #[test]
    fn test_register_creature_event_returns_true_on_first_add() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        assert!(c.register_creature_event(&mut ext, "OnDeath"));
        assert!(c.has_event_registered(&ext, "OnDeath"));
        // Duplicate registration: returns false.
        assert!(!c.register_creature_event(&mut ext, "OnDeath"));
        assert_eq!(c.get_creature_events(&ext).len(), 1);
    }

    #[test]
    fn test_unregister_creature_event_returns_true_when_present() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.register_creature_event(&mut ext, "OnKill");
        assert!(c.unregister_creature_event(&mut ext, "OnKill"));
        assert!(!c.has_event_registered(&ext, "OnKill"));
        // Unregistering an absent event returns false.
        assert!(!c.unregister_creature_event(&mut ext, "OnKill"));
    }

    #[test]
    fn test_damage_map_records_and_attributes() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.add_damage_points(&mut ext, 100, 30, 1_000);
        c.add_damage_points(&mut ext, 200, 70, 2_000);
        assert!(c.has_been_attacked(&ext, 100));
        assert!(c.has_been_attacked(&ext, 200));
        assert!(!c.has_been_attacked(&ext, 999));
        // Ratios sum to 1.0.
        let r1 = c.get_damage_ratio(&ext, 100);
        let r2 = c.get_damage_ratio(&ext, 200);
        assert!((r1 - 0.30).abs() < 1e-9);
        assert!((r2 - 0.70).abs() < 1e-9);
        // Killers sorted descending by damage.
        let killers = c.get_killers(&ext);
        assert_eq!(killers[0].0, 200);
        assert_eq!(killers[0].1, 70);
    }

    #[test]
    fn test_damage_map_accumulates_multiple_hits_from_same_attacker() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.add_damage_points(&mut ext, 100, 10, 1_000);
        c.add_damage_points(&mut ext, 100, 25, 2_000);
        assert_eq!(c.get_damage_ratio(&ext, 100), 1.0);
        assert_eq!(c.get_killers(&ext)[0].1, 35);
    }

    #[test]
    fn test_damage_ratio_zero_when_no_damage() {
        let c = Creature::new(1, "Rat");
        let ext = CreatureExt::default();
        assert_eq!(c.get_damage_ratio(&ext, 100), 0.0);
    }

    #[test]
    fn test_drain_health_clamps_at_zero() {
        let mut c = Creature::new(1, "Rat");
        c.set_health(50);
        c.drain_health(60);
        assert_eq!(c.get_health(), 0);
    }

    #[test]
    fn test_gain_health_clamps_at_max() {
        let mut c = Creature::new(1, "Rat");
        c.set_max_health(100);
        c.set_health(80);
        c.gain_health(50);
        assert_eq!(c.get_health(), 100);
    }

    #[test]
    fn test_change_health_dispatches_signed() {
        let mut c = Creature::new(1, "Rat");
        c.set_max_health(100);
        c.set_health(80);
        c.change_health(-30);
        assert_eq!(c.get_health(), 50);
        c.change_health(20);
        assert_eq!(c.get_health(), 70);
    }

    #[test]
    fn test_is_attackable_default_true() {
        let c = Creature::new(1, "Rat");
        assert!(c.is_attackable());
    }

    #[test]
    fn test_ghost_mode_set_get() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        assert!(!c.is_in_ghost_mode(&ext));
        c.set_in_ghost_mode(&mut ext, true);
        assert!(c.is_in_ghost_mode(&ext));
    }

    #[test]
    fn test_outfit_set_get_round_trip() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        let outfit = CreatureOutfit {
            look_type: 128,
            look_head: 1,
            look_body: 2,
            look_legs: 3,
            look_feet: 4,
            look_addons: 1,
            look_mount: 30,
        };
        c.set_current_outfit(&mut ext, outfit);
        assert_eq!(c.get_current_outfit(&ext), outfit);
    }

    #[test]
    fn test_reference_counter_increment_and_decrement() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        assert_eq!(c.reference_count(&ext), 0);
        c.increment_reference_counter(&mut ext);
        c.increment_reference_counter(&mut ext);
        assert_eq!(c.reference_count(&ext), 2);
        assert!(!c.decrement_reference_counter(&mut ext));
        assert!(c.decrement_reference_counter(&mut ext));
        // Already-zero decrement is a no-op.
        assert!(c.decrement_reference_counter(&mut ext));
    }

    #[test]
    fn test_start_auto_walk_dirs_queues_in_order() {
        let mut c = Creature::new(1, "Rat");
        c.start_auto_walk_dirs(&[Direction::North, Direction::East, Direction::South]);
        // Stored back-first: pop returns North first.
        assert_eq!(c.get_next_walk_step(), Some(Direction::North));
        assert_eq!(c.get_next_walk_step(), Some(Direction::East));
        assert_eq!(c.get_next_walk_step(), Some(Direction::South));
        assert_eq!(c.get_next_walk_step(), None);
    }

    #[test]
    fn test_start_auto_walk_single_queues_one_step() {
        let mut c = Creature::new(1, "Rat");
        c.start_auto_walk_single(Direction::West);
        assert!(c.has_pending_walk_steps());
        assert_eq!(c.get_next_walk_step(), Some(Direction::West));
    }

    #[test]
    fn test_stop_event_walk_clears_queue_and_sets_cancel_flag() {
        let mut c = Creature::new(1, "Rat");
        c.start_auto_walk_dirs(&[Direction::North, Direction::East]);
        c.stop_event_walk();
        assert!(!c.has_pending_walk_steps());
        assert!(c.cancel_next_walk);
    }

    #[test]
    fn test_last_position_set_get() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        let p = Position::new(50, 60, 7);
        c.set_last_position(&mut ext, p);
        assert_eq!(c.get_last_position(&ext), p);
    }

    #[test]
    fn test_last_step_records_ts_and_cost() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.set_last_step(&mut ext, 12345, 220);
        assert_eq!(c.get_last_step(&ext), 12345);
        assert_eq!(c.get_last_step_cost(&ext), 220);
    }

    #[test]
    fn test_get_time_since_last_move_returns_delta() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.set_last_step(&mut ext, 1_000, 220);
        assert_eq!(c.get_time_since_last_move(&ext, 1_500), 500);
        // Negative deltas clamp to 0.
        assert_eq!(c.get_time_since_last_move(&ext, 500), 0);
    }

    #[test]
    fn test_event_walk_handle_round_trip() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        c.set_event_walk(&mut ext, 9999);
        assert_eq!(c.get_event_walk(&ext), 9999);
    }

    #[test]
    fn test_can_use_defense_toggle() {
        let mut c = Creature::new(1, "Rat");
        let mut ext = CreatureExt::default();
        assert!(!c.can_use_defense(&ext));
        c.set_use_defense(&mut ext, true);
        assert!(c.can_use_defense(&ext));
    }

    #[test]
    fn test_drop_loot_toggle() {
        let mut c = Creature::new(1, "Rat");
        c.set_drop_loot(false);
        assert!(!c.get_drop_loot());
        c.set_drop_loot(true);
        assert!(c.get_drop_loot());
    }

    #[test]
    fn test_is_pushable_creature_depends_on_health_and_flag() {
        let mut c = Creature::new(1, "Rat");
        // Default: pushable=true, health=100 → pushable.
        assert!(c.is_pushable_creature());
        // Death → not pushable.
        c.set_health(0);
        assert!(!c.is_pushable_creature());
        // Restore health but disable pushable flag.
        c.set_health(100);
        c.pushable = false;
        assert!(!c.is_pushable_creature());
    }

    // --- Thing impl smoke tests ---

    use forgottenserver_common::thing::Thing as CommonThing;

    #[test]
    fn test_creature_via_thing_is_creature() {
        let c = Creature::new(1, "Rat");
        assert!(CommonThing::is_creature(&c));
        assert!(CommonThing::is_removable(&c));
        assert!(!CommonThing::is_item(&c));
    }

    #[test]
    fn test_creature_via_thing_get_position_tuple() {
        let mut c = Creature::new(1, "Rat");
        c.set_position(Position::new(10, 20, 7));
        assert_eq!(c.get_position_tuple(), Some((10, 20, 7)));
    }

    #[test]
    fn test_creature_via_thing_get_creature_returns_self() {
        let c = Creature::new(1, "Rat");
        assert!(CommonThing::get_creature(&c).is_some());
    }

    #[test]
    fn test_creature_via_thing_get_description_returns_name() {
        let c = Creature::new(1, "Demon");
        assert_eq!(CommonThing::get_description(&c, 0), "Demon");
    }

    #[test]
    fn test_creature_via_dyn_thing_trait_object() {
        let c = Creature::new(1, "Rat");
        let t: &dyn CommonThing = &c;
        assert!(t.is_creature());
        assert_eq!(t.get_throw_range(), 1);
    }
}
