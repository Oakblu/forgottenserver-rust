//! Migrated from forgottenserver/src/monster.h and monster.cpp
//! Monster-specific data-bag. Contains a Creature plus monster fields.

use crate::creature::Creature;
use forgottenserver_common::position::Position;

// ---------------------------------------------------------------------------
// LootBlock
// ---------------------------------------------------------------------------

/// A single entry in a monster's loot table.
/// `chance` is out of 100_000 (MAX_LOOTCHANCE = 100000 in C++).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootBlock {
    pub item_type_id: u16,
    pub count_min: u32,
    pub count_max: u32,
    /// Out of 100_000. 100_000 = always drop, 0 = never drop.
    pub chance: u32,
}

impl LootBlock {
    pub fn new(item_type_id: u16, count_min: u32, count_max: u32, chance: u32) -> Self {
        LootBlock {
            item_type_id,
            count_min,
            count_max,
            chance,
        }
    }
}

// ---------------------------------------------------------------------------
// MonsterState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterState {
    Idle,
    Sleeping,
    InCombat,
    Fleeing,
}

// ---------------------------------------------------------------------------
// TargetSearchType — mirrors C++ TargetSearchType_t
// ---------------------------------------------------------------------------

/// Mirrors the C++ `TargetSearchType_t` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetSearchType {
    /// Default selection: random from in-range targets, then first in list.
    Default,
    /// Select a random valid target.
    Random,
    /// Only select targets within attack range.
    AttackRange,
    /// Select the nearest target.
    Nearest,
}

// ---------------------------------------------------------------------------
// ElementResistance — one entry in a monster's element map
// ---------------------------------------------------------------------------

/// A monster's modifier for a specific combat element.
/// `modifier` is a signed percent: positive = less damage, negative = more.
/// Mirrors `elementMap` in C++ `MonsterInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementResistance {
    /// Combat type identifier (mirrors `CombatType_t` int value in C++).
    pub combat_type: i32,
    /// Percentage modifier. Damage is multiplied by `(100 - modifier) / 100.0`.
    /// 100 = immune, 0 = no change, -100 = double damage.
    pub modifier: i32,
}

impl ElementResistance {
    pub fn new(combat_type: i32, modifier: i32) -> Self {
        ElementResistance {
            combat_type,
            modifier,
        }
    }
}

// ---------------------------------------------------------------------------
// TargetEntry — stores a creature_id with associated threat value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct TargetEntry {
    creature_id: u32,
    threat: i32,
}

// ---------------------------------------------------------------------------
// OnThinkTickets — AI tick counters (mirrors C++ per-monster tick fields)
// ---------------------------------------------------------------------------

/// Tick counters used by the monster AI loop.
/// All values are in milliseconds. They mirror the C++ member variables:
/// `attackTicks`, `defenseTicks`, `yellTicks`, `targetChangeTicks`,
/// `targetChangeCooldown`, `challengeFocusDuration`.
#[derive(Debug, Default, Clone)]
pub struct AiTicks {
    pub attack_ticks: u32,
    pub defense_ticks: u32,
    pub yell_ticks: u32,
    pub target_change_ticks: u32,
    pub target_change_cooldown: i32,
    pub challenge_focus_duration: i32,
}

// ---------------------------------------------------------------------------
// Monster
// ---------------------------------------------------------------------------

/// Monster-specific data. Wraps a `Creature` base via composition.
#[derive(Debug)]
pub struct Monster {
    pub creature: Creature,
    pub loot_table: Vec<LootBlock>,
    /// Health percentage (0–100) at which the monster starts fleeing.
    /// 0 means fleeing is never triggered by health threshold.
    pub flee_health_percent: u32,
    pub can_flee: bool,
    pub state: MonsterState,
    /// Absolute HP threshold below which the monster flees.
    /// Mirrors `mType->info.runAwayHealth` in C++.
    pub run_away_health: i32,
    /// Whether the monster is currently walking back to its spawn point.
    pub walking_to_spawn: bool,
    /// Master position — the spawn point the monster was placed at.
    /// Used for despawn-range checks and walk-to-spawn logic.
    pub master_pos: Position,
    /// Minimum and maximum combat values set by the last-used spell.
    pub min_combat_value: i32,
    pub max_combat_value: i32,
    /// Element resistances loaded from the monster type.
    pub element_resistances: Vec<ElementResistance>,
    /// AI tick counters.
    pub ai_ticks: AiTicks,
    targets: Vec<TargetEntry>,
}

impl Monster {
    /// Create a new monster.
    /// `base_health` sets both current and max health on the inner creature.
    pub fn new(id: u32, name: impl Into<String>, base_health: i32) -> Self {
        let mut creature = Creature::new(id, name);
        creature.set_health(base_health);
        creature.set_max_health(base_health);
        // Monsters are not pushable by default in C++ (requires pushable flag in mType)
        creature.set_pushable(false);

        Monster {
            creature,
            loot_table: Vec::new(),
            flee_health_percent: 0,
            can_flee: false,
            state: MonsterState::Idle,
            run_away_health: 0,
            walking_to_spawn: false,
            master_pos: Position::default(),
            min_combat_value: 0,
            max_combat_value: 0,
            element_resistances: Vec::new(),
            ai_ticks: AiTicks::default(),
            targets: Vec::new(),
        }
    }

    // --- Basic accessors ---

    pub fn get_id(&self) -> u32 {
        self.creature.get_id()
    }

    pub fn get_name(&self) -> &str {
        self.creature.get_name()
    }

    pub fn is_pushable(&self) -> bool {
        self.creature.is_pushable()
    }

    pub fn can_flee(&self) -> bool {
        self.can_flee
    }

    // --- Health percent utility ---

    /// Returns current health as an integer percentage of max health (0–100).
    /// Returns 0 if max health is 0 (guard against divide-by-zero).
    ///
    /// Mirrors the computation used in C++ flee checks:
    ///   `getHealth() <= mType->info.runAwayHealth` where runAwayHealth is an
    ///   absolute value, but the Rust model also exposes the percent helper.
    pub fn get_health_percent(&self) -> u32 {
        let max = self.creature.get_max_health();
        if max <= 0 {
            return 0;
        }
        let hp = self.creature.get_health().max(0) as u64;
        let max = max as u64;
        ((hp * 100) / max) as u32
    }

    // --- Loot ---

    pub fn add_loot_block(&mut self, block: LootBlock) {
        self.loot_table.push(block);
    }

    /// Roll loot using a simple seeded xorshift64 PRNG.
    /// Returns a Vec of `(item_type_id, count)` pairs for items that drop.
    pub fn roll_loot(&self, rng_seed: u64) -> Vec<(u16, u32)> {
        let mut rng = rng_seed;
        let mut result = Vec::new();

        for block in &self.loot_table {
            // xorshift64
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;

            if block.chance == 0 {
                continue;
            }
            let roll = (rng % 100_000) as u32; // 0..99999
            if block.chance < 100_000 && roll >= block.chance {
                continue;
            }
            // Determine count: random in [count_min, count_max]
            let range = block.count_max.saturating_sub(block.count_min);
            let count = if range == 0 {
                block.count_min
            } else {
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                block.count_min + (rng % (range as u64 + 1)) as u32
            };
            result.push((block.item_type_id, count));
        }
        result
    }

    // --- Flee behavior ---

    pub fn set_flee_health_percent(&mut self, pct: u32) {
        self.flee_health_percent = pct;
    }

    pub fn set_can_flee(&mut self, val: bool) {
        self.can_flee = val;
    }

    /// Returns true when fleeing is enabled AND current health percentage is
    /// strictly less than the flee threshold.
    pub fn should_flee(&self, current_health_pct: u32) -> bool {
        self.can_flee && current_health_pct < self.flee_health_percent
    }

    /// Returns true when the monster is currently in a fleeing state.
    ///
    /// Mirrors the C++ `isFleeing()` check:
    ///   `!isSummon() && getHealth() <= mType->info.runAwayHealth && challengeFocusDuration <= 0`
    ///
    /// In the data-bag model there is no summon flag on Monster itself (summons
    /// are tracked through `Creature::summons`); `is_summon` must be passed in.
    pub fn is_fleeing(&self, is_summon: bool) -> bool {
        if is_summon {
            return false;
        }
        if self.ai_ticks.challenge_focus_duration > 0 {
            return false;
        }
        self.run_away_health > 0 && self.creature.get_health() <= self.run_away_health
    }

    // --- Element damage multipliers ---

    /// Add or replace an element resistance entry.
    pub fn add_element_resistance(&mut self, combat_type: i32, modifier: i32) {
        self.element_resistances
            .retain(|r| r.combat_type != combat_type);
        self.element_resistances
            .push(ElementResistance::new(combat_type, modifier));
    }

    /// Get the modifier for a specific combat type, or 0 if not found.
    pub fn get_element_modifier(&self, combat_type: i32) -> i32 {
        self.element_resistances
            .iter()
            .find(|r| r.combat_type == combat_type)
            .map(|r| r.modifier)
            .unwrap_or(0)
    }

    /// Apply element damage modifiers to an incoming damage value.
    ///
    /// Mirrors C++ `Monster::blockHit`:
    ///   `damage = round(damage * ((100 - elementMod) / 100.0))`
    ///
    /// Returns the modified damage. Returns 0 if the result is <= 0 (blocked).
    pub fn apply_element_modifier(&self, combat_type: i32, damage: i32) -> i32 {
        if damage == 0 {
            return 0;
        }
        let modifier = self.get_element_modifier(combat_type);
        if modifier == 0 {
            return damage;
        }
        let modified = (damage as f64 * ((100 - modifier) as f64 / 100.0)).round() as i32;
        modified.max(0)
    }

    // --- Target selection ---

    pub fn add_target(&mut self, creature_id: u32, threat: i32) {
        // Remove existing entry then add updated one
        self.targets.retain(|t| t.creature_id != creature_id);
        self.targets.push(TargetEntry {
            creature_id,
            threat,
        });
    }

    pub fn remove_target(&mut self, creature_id: u32) {
        self.targets.retain(|t| t.creature_id != creature_id);
    }

    pub fn get_target_count(&self) -> usize {
        self.targets.len()
    }

    /// Returns the creature_id of the highest-threat target, or None if empty.
    pub fn get_best_target(&self) -> Option<u32> {
        self.targets
            .iter()
            .max_by_key(|t| t.threat)
            .map(|t| t.creature_id)
    }

    /// Collect all target creature IDs in insertion order.
    pub fn get_target_ids(&self) -> Vec<u32> {
        self.targets.iter().map(|t| t.creature_id).collect()
    }

    /// Determine which target to select based on a search type and a provided
    /// distance resolver.
    ///
    /// This is the data-bag portion of C++ `Monster::searchTarget`. It cannot
    /// call into the game world, so callers supply `distances` — a slice of
    /// `(creature_id, manhattan_distance)` pairs for the targets that are
    /// currently valid (i.e. in attack range / can be followed).
    ///
    /// `search_type` controls the selection strategy:
    /// - `Nearest`  → pick the entry with the smallest distance.
    /// - `Random`   → pick any entry from `distances` at the given `random_index`.
    /// - `Default` / `AttackRange` → pick entry at `random_index` from `distances`.
    ///
    /// Returns `Some(creature_id)` of the chosen target, or `None` when the
    /// candidate list is empty.
    ///
    /// `random_index` is a caller-supplied pre-rolled index (modded internally
    /// to the slice length), enabling deterministic tests.
    pub fn select_target_from(
        &self,
        search_type: TargetSearchType,
        distances: &[(u32, i32)],
        random_index: usize,
    ) -> Option<u32> {
        if distances.is_empty() {
            return None;
        }
        match search_type {
            TargetSearchType::Nearest => {
                distances.iter().min_by_key(|(_, d)| *d).map(|(id, _)| *id)
            }
            TargetSearchType::Default
            | TargetSearchType::Random
            | TargetSearchType::AttackRange => {
                let idx = random_index % distances.len();
                Some(distances[idx].0)
            }
        }
    }

    /// `is_target` check (pure data-bag portion).
    ///
    /// Mirrors the pure-data conditions from C++ `Monster::isTarget`:
    ///   - creature must not be dead
    ///   - must be on the same floor (z matches)
    ///   - must be within monster's viewport (can_see)
    ///
    /// The C++ version also checks `isRemoved`, `isAttackable`, and `ZONE_PROTECTION`
    /// which require game-world context — those are left to callers.
    ///
    /// Returns `true` when `target_pos` is on the same floor and within the
    /// 9×7 viewport centred at `self_pos`.
    pub fn is_valid_target_position(&self, self_pos: Position, target_pos: Position) -> bool {
        if self_pos.z != target_pos.z {
            return false;
        }
        Creature::can_see(self_pos, target_pos)
    }

    // --- Combat values ---

    /// Returns `(min, max)` combat values if at least one is non-zero.
    /// Mirrors C++ `Monster::getCombatValues`.
    pub fn get_combat_values(&self) -> Option<(i32, i32)> {
        if self.min_combat_value == 0 && self.max_combat_value == 0 {
            None
        } else {
            Some((self.min_combat_value, self.max_combat_value))
        }
    }

    /// Set the min/max combat values (typically set just before casting a spell).
    pub fn set_combat_values(&mut self, min: i32, max: i32) {
        self.min_combat_value = min;
        self.max_combat_value = max;
    }

    // --- Challenge creature ---

    /// Challenge the monster to focus on a specific creature.
    ///
    /// Mirrors C++ `Monster::challengeCreature`:
    ///   - Summons cannot be challenged.
    ///   - Sets `targetChangeCooldown = 8000` and `challengeFocusDuration = 8000`.
    ///   - Returns `true` when the challenge was accepted (target is valid).
    ///
    /// The `is_summon` flag must be passed in (data-bag cannot inspect the summon
    /// relationship itself). `target_is_valid` indicates whether `selectTarget`
    /// would succeed for that creature_id.
    pub fn challenge_creature(&mut self, is_summon: bool, target_is_valid: bool) -> bool {
        if is_summon {
            return false;
        }
        if !target_is_valid {
            return false;
        }
        self.ai_ticks.target_change_cooldown = 8000;
        self.ai_ticks.challenge_focus_duration = 8000;
        self.ai_ticks.target_change_ticks = 0;
        true
    }

    // --- onThink target tick (data-bag portion) ---

    /// Advance the target-change tick counters by `interval` milliseconds.
    ///
    /// Mirrors `Monster::onThinkTarget` logic (for non-summons):
    ///   - Decrements `challenge_focus_duration` by interval (clamped to 0).
    ///   - Decrements `target_change_cooldown` by interval when > 0.
    ///     When cooldown expires, resets `target_change_ticks` to `change_speed`.
    ///   - When no cooldown active, increments `target_change_ticks` by interval.
    ///   - Returns `true` when `target_change_ticks >= change_speed` (time to
    ///     search for a new target).
    ///
    /// `change_speed` is the monster type's `changeTargetSpeed` field (0 means
    /// target changing is disabled — returns `false` immediately).
    ///
    /// When this returns `true`, the caller should call `select_target_from` and
    /// then reset `ai_ticks.target_change_ticks = 0` and set the cooldown.
    pub fn tick_target_change(&mut self, interval: u32, change_speed: u32) -> bool {
        if change_speed == 0 {
            return false;
        }

        // Decrement challenge focus duration
        if self.ai_ticks.challenge_focus_duration > 0 {
            self.ai_ticks.challenge_focus_duration -= interval as i32;
            if self.ai_ticks.challenge_focus_duration < 0 {
                self.ai_ticks.challenge_focus_duration = 0;
            }
        }

        // Decrement cooldown
        if self.ai_ticks.target_change_cooldown > 0 {
            self.ai_ticks.target_change_cooldown -= interval as i32;
            if self.ai_ticks.target_change_cooldown <= 0 {
                self.ai_ticks.target_change_cooldown = 0;
                self.ai_ticks.target_change_ticks = change_speed;
            }
            // Still in cooldown — can't change target this tick
            return false;
        }

        // Advance ticks
        self.ai_ticks.target_change_ticks += interval;
        self.ai_ticks.target_change_ticks >= change_speed
    }

    // --- onThink yell tick ---

    /// Advance the yell-tick counter by `interval` ms.
    ///
    /// Mirrors C++ `Monster::onThinkYell`:
    ///   - Accumulates `yellTicks`.
    ///   - When `yellTicks >= yellSpeedTicks`, resets and returns `true`.
    ///
    /// `yell_speed_ticks` is `mType->info.yellSpeedTicks` (0 = disabled).
    /// Returns `true` when it's time for the monster to yell.
    pub fn tick_yell(&mut self, interval: u32, yell_speed_ticks: u32) -> bool {
        if yell_speed_ticks == 0 {
            return false;
        }
        self.ai_ticks.yell_ticks += interval;
        if self.ai_ticks.yell_ticks >= yell_speed_ticks {
            self.ai_ticks.yell_ticks = 0;
            true
        } else {
            false
        }
    }

    // --- onThink state machine transition ---

    /// Compute the next `MonsterState` given current conditions.
    ///
    /// Mirrors the state transitions in C++ `Monster::onThink`:
    ///   - `Fleeing`  when `is_fleeing(is_summon)` is true and has targets.
    ///   - `InCombat` when there are targets and not fleeing.
    ///   - `Idle`     when there are no targets.
    ///
    /// `is_summon` is needed for the flee check.
    pub fn compute_state(&self, is_summon: bool) -> MonsterState {
        if self.targets.is_empty() {
            MonsterState::Idle
        } else if self.is_fleeing(is_summon) {
            MonsterState::Fleeing
        } else {
            MonsterState::InCombat
        }
    }

    // --- Walk to spawn ---

    /// Check and update the walk-to-spawn flag.
    ///
    /// Mirrors C++ `Monster::walkToSpawn`:
    ///   - Returns `false` (won't walk) when already walking or has targets.
    ///   - Otherwise sets `walking_to_spawn = true` when there is a spawn
    ///     (indicated by `has_spawn`).
    ///   - Distance to master_pos must be > 0.
    ///
    /// Returns `true` if the monster should start walking to spawn.
    pub fn try_walk_to_spawn(&mut self, has_spawn: bool, current_pos: Position) -> bool {
        if self.walking_to_spawn || !has_spawn || !self.targets.is_empty() {
            return false;
        }
        let dx = (current_pos.x as i32 - self.master_pos.x as i32).unsigned_abs();
        let dy = (current_pos.y as i32 - self.master_pos.y as i32).unsigned_abs();
        let distance = dx.max(dy);
        if distance == 0 {
            return false;
        }
        self.walking_to_spawn = true;
        true
    }

    /// Called when the walk-to-spawn path is complete (mirrors `onWalkComplete`).
    /// Clears the flag and triggers another walk-to-spawn attempt.
    pub fn on_walk_complete_spawn(&mut self) {
        if self.walking_to_spawn {
            self.walking_to_spawn = false;
        }
    }

    // --- Monster state ---

    pub fn get_state(&self) -> MonsterState {
        self.state
    }

    pub fn set_state(&mut self, state: MonsterState) {
        self.state = state;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::position::Position;

    // --- Monster basics ---

    #[test]
    fn test_monster_new_id() {
        let m = Monster::new(1, "Rat", 30);
        assert_eq!(m.get_id(), 1);
    }

    #[test]
    fn test_monster_new_name() {
        let m = Monster::new(1, "Rat", 30);
        assert_eq!(m.get_name(), "Rat");
    }

    #[test]
    fn test_monster_new_health() {
        let m = Monster::new(1, "Rat", 30);
        assert_eq!(m.creature.get_health(), 30);
        assert_eq!(m.creature.get_max_health(), 30);
    }

    #[test]
    fn test_monster_is_not_pushable_by_default() {
        let m = Monster::new(1, "Rat", 30);
        assert!(!m.is_pushable());
    }

    #[test]
    fn test_monster_cannot_flee_by_default() {
        let m = Monster::new(1, "Rat", 30);
        assert!(!m.can_flee());
    }

    // --- Loot ---

    #[test]
    fn test_monster_add_loot_block() {
        let mut m = Monster::new(1, "Rat", 30);
        m.add_loot_block(LootBlock::new(100, 1, 1, 100_000));
        assert_eq!(m.loot_table.len(), 1);
    }

    #[test]
    fn test_monster_roll_loot_always_drops() {
        let mut m = Monster::new(1, "Rat", 30);
        // chance = 100_000 → always drops
        m.add_loot_block(LootBlock::new(100, 1, 1, 100_000));
        let drops = m.roll_loot(12345);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].0, 100);
        assert_eq!(drops[0].1, 1);
    }

    #[test]
    fn test_monster_roll_loot_never_drops() {
        let mut m = Monster::new(1, "Rat", 30);
        // chance = 0 → never drops
        m.add_loot_block(LootBlock::new(100, 1, 1, 0));
        let drops = m.roll_loot(12345);
        assert!(drops.is_empty());
    }

    #[test]
    fn test_monster_roll_loot_count_in_range() {
        let mut m = Monster::new(1, "Rat", 30);
        // Always drops, count 1..3
        m.add_loot_block(LootBlock::new(100, 1, 3, 100_000));
        let drops = m.roll_loot(99999);
        assert_eq!(drops.len(), 1);
        let count = drops[0].1;
        assert!((1..=3).contains(&count), "count out of range: {count}");
    }

    // --- Flee behavior ---

    #[test]
    fn test_monster_set_flee_health_percent() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_flee_health_percent(20);
        assert_eq!(m.flee_health_percent, 20);
    }

    #[test]
    fn test_monster_should_flee_when_below_threshold() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_can_flee(true);
        m.set_flee_health_percent(20);
        assert!(m.should_flee(10)); // 10 < 20
    }

    #[test]
    fn test_monster_should_not_flee_when_at_threshold() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_can_flee(true);
        m.set_flee_health_percent(20);
        assert!(!m.should_flee(20)); // 20 == 20, NOT < 20
    }

    #[test]
    fn test_monster_should_not_flee_when_disabled() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_flee_health_percent(20);
        // can_flee is false by default
        assert!(!m.should_flee(5));
    }

    #[test]
    fn test_monster_set_can_flee() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_can_flee(true);
        assert!(m.can_flee());
    }

    // --- Target selection ---

    #[test]
    fn test_monster_add_target() {
        let mut m = Monster::new(1, "Rat", 30);
        m.add_target(10, 50);
        assert_eq!(m.get_target_count(), 1);
    }

    #[test]
    fn test_monster_get_best_target() {
        let mut m = Monster::new(1, "Rat", 30);
        m.add_target(10, 50);
        m.add_target(11, 100);
        m.add_target(12, 25);
        assert_eq!(m.get_best_target(), Some(11)); // highest threat
    }

    #[test]
    fn test_monster_get_best_target_none_when_empty() {
        let m = Monster::new(1, "Rat", 30);
        assert_eq!(m.get_best_target(), None);
    }

    #[test]
    fn test_monster_remove_target() {
        let mut m = Monster::new(1, "Rat", 30);
        m.add_target(10, 50);
        m.remove_target(10);
        assert_eq!(m.get_target_count(), 0);
    }

    #[test]
    fn test_monster_target_count() {
        let mut m = Monster::new(1, "Rat", 30);
        m.add_target(10, 50);
        m.add_target(11, 100);
        assert_eq!(m.get_target_count(), 2);
    }

    // --- Monster state ---

    #[test]
    fn test_monster_state_idle_by_default() {
        let m = Monster::new(1, "Rat", 30);
        assert_eq!(m.get_state(), MonsterState::Idle);
    }

    #[test]
    fn test_monster_set_state() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_state(MonsterState::InCombat);
        assert_eq!(m.get_state(), MonsterState::InCombat);
    }

    #[test]
    fn test_monster_state_sleeping() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_state(MonsterState::Sleeping);
        assert_eq!(m.get_state(), MonsterState::Sleeping);
    }

    #[test]
    fn test_monster_state_fleeing() {
        let mut m = Monster::new(1, "Rat", 30);
        m.set_state(MonsterState::Fleeing);
        assert_eq!(m.get_state(), MonsterState::Fleeing);
    }

    // =========================================================================
    // NEW TESTS — gaps from C++ audit
    // =========================================================================

    // --- get_health_percent ---

    #[test]
    fn test_get_health_percent_full_health() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(m.get_health_percent(), 100);
    }

    #[test]
    fn test_get_health_percent_half_health() {
        let mut m = Monster::new(1, "Rat", 100);
        m.creature.set_health(50);
        assert_eq!(m.get_health_percent(), 50);
    }

    #[test]
    fn test_get_health_percent_zero_health() {
        let mut m = Monster::new(1, "Rat", 100);
        m.creature.set_health(0);
        assert_eq!(m.get_health_percent(), 0);
    }

    #[test]
    fn test_get_health_percent_zero_max_returns_zero() {
        let mut m = Monster::new(1, "Rat", 0);
        m.creature.set_health(0);
        assert_eq!(m.get_health_percent(), 0);
    }

    #[test]
    fn test_get_health_percent_rounding() {
        let mut m = Monster::new(1, "Rat", 1000);
        m.creature.set_health(333);
        // 333 * 100 / 1000 = 33 (integer division)
        assert_eq!(m.get_health_percent(), 33);
    }

    // --- is_fleeing ---

    #[test]
    fn test_is_fleeing_returns_false_when_summon() {
        let mut m = Monster::new(1, "Rat", 100);
        m.run_away_health = 30;
        m.creature.set_health(10); // below threshold
                                   // is_summon = true → never fleeing
        assert!(!m.is_fleeing(true));
    }

    #[test]
    fn test_is_fleeing_returns_false_when_health_above_threshold() {
        let mut m = Monster::new(1, "Rat", 100);
        m.run_away_health = 20;
        m.creature.set_health(50); // above run_away_health
        assert!(!m.is_fleeing(false));
    }

    #[test]
    fn test_is_fleeing_returns_true_when_health_at_threshold() {
        let mut m = Monster::new(1, "Rat", 100);
        m.run_away_health = 30;
        m.creature.set_health(30); // exactly at threshold: <= 30 → flee
        assert!(m.is_fleeing(false));
    }

    #[test]
    fn test_is_fleeing_returns_true_when_health_below_threshold() {
        let mut m = Monster::new(1, "Rat", 100);
        m.run_away_health = 30;
        m.creature.set_health(10); // below threshold
        assert!(m.is_fleeing(false));
    }

    #[test]
    fn test_is_fleeing_returns_false_when_challenge_focus_active() {
        let mut m = Monster::new(1, "Rat", 100);
        m.run_away_health = 30;
        m.creature.set_health(10); // below threshold
        m.ai_ticks.challenge_focus_duration = 5000; // challenged → no flee
        assert!(!m.is_fleeing(false));
    }

    #[test]
    fn test_is_fleeing_returns_false_when_run_away_health_zero() {
        let mut m = Monster::new(1, "Rat", 100);
        // run_away_health == 0 means no flee threshold configured
        m.run_away_health = 0;
        m.creature.set_health(1);
        assert!(!m.is_fleeing(false));
    }

    // --- element resistances ---

    #[test]
    fn test_add_element_resistance_stored() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_element_resistance(1, 50); // combat_type=1, modifier=50%
        assert_eq!(m.get_element_modifier(1), 50);
    }

    #[test]
    fn test_get_element_modifier_unknown_returns_zero() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(m.get_element_modifier(99), 0);
    }

    #[test]
    fn test_add_element_resistance_replaces_existing() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_element_resistance(1, 50);
        m.add_element_resistance(1, 75); // replace
        assert_eq!(m.get_element_modifier(1), 75);
        assert_eq!(m.element_resistances.len(), 1);
    }

    #[test]
    fn test_apply_element_modifier_no_resistance() {
        let m = Monster::new(1, "Rat", 100);
        // no resistance → damage unchanged
        assert_eq!(m.apply_element_modifier(1, 100), 100);
    }

    #[test]
    fn test_apply_element_modifier_50_percent() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_element_resistance(1, 50); // 50% resist
                                         // 100 * (100 - 50) / 100 = 50
        assert_eq!(m.apply_element_modifier(1, 100), 50);
    }

    #[test]
    fn test_apply_element_modifier_immune_100_percent() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_element_resistance(1, 100); // immune
        assert_eq!(m.apply_element_modifier(1, 200), 0);
    }

    #[test]
    fn test_apply_element_modifier_weakness_negative_modifier() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_element_resistance(1, -50); // 50% more damage
                                          // 100 * (100 - (-50)) / 100 = 100 * 150 / 100 = 150
        assert_eq!(m.apply_element_modifier(1, 100), 150);
    }

    #[test]
    fn test_apply_element_modifier_zero_damage_unchanged() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_element_resistance(1, 50);
        assert_eq!(m.apply_element_modifier(1, 0), 0);
    }

    #[test]
    fn test_apply_element_modifier_clamps_to_zero() {
        let mut m = Monster::new(1, "Rat", 100);
        // modifier > 100 would yield negative damage — clamp to 0
        m.add_element_resistance(1, 200);
        assert_eq!(m.apply_element_modifier(1, 50), 0);
    }

    // --- select_target_from (TargetSearchType) ---

    #[test]
    fn test_select_target_nearest_picks_closest() {
        let m = Monster::new(1, "Rat", 100);
        let distances = vec![(10u32, 15i32), (11u32, 5i32), (12u32, 20i32)];
        let result = m.select_target_from(TargetSearchType::Nearest, &distances, 0);
        assert_eq!(result, Some(11)); // smallest distance = 5
    }

    #[test]
    fn test_select_target_nearest_single_entry() {
        let m = Monster::new(1, "Rat", 100);
        let distances = vec![(42u32, 3i32)];
        assert_eq!(
            m.select_target_from(TargetSearchType::Nearest, &distances, 0),
            Some(42)
        );
    }

    #[test]
    fn test_select_target_random_uses_index() {
        let m = Monster::new(1, "Rat", 100);
        let distances = vec![(10u32, 5i32), (11u32, 10i32), (12u32, 3i32)];
        // index 1 → creature_id 11
        assert_eq!(
            m.select_target_from(TargetSearchType::Random, &distances, 1),
            Some(11)
        );
    }

    #[test]
    fn test_select_target_random_wraps_index() {
        let m = Monster::new(1, "Rat", 100);
        let distances = vec![(10u32, 5i32), (11u32, 10i32)];
        // index 5 % 2 = 1 → creature_id 11
        assert_eq!(
            m.select_target_from(TargetSearchType::Random, &distances, 5),
            Some(11)
        );
    }

    #[test]
    fn test_select_target_default_uses_index() {
        let m = Monster::new(1, "Rat", 100);
        let distances = vec![(10u32, 5i32), (11u32, 10i32)];
        assert_eq!(
            m.select_target_from(TargetSearchType::Default, &distances, 0),
            Some(10)
        );
    }

    #[test]
    fn test_select_target_empty_returns_none() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(
            m.select_target_from(TargetSearchType::Nearest, &[], 0),
            None
        );
    }

    // --- is_valid_target_position ---

    #[test]
    fn test_is_valid_target_position_same_floor_in_viewport() {
        let m = Monster::new(1, "Rat", 100);
        let self_pos = Position::new(100, 100, 7);
        let target_pos = Position::new(105, 103, 7);
        assert!(m.is_valid_target_position(self_pos, target_pos));
    }

    #[test]
    fn test_is_valid_target_position_different_floor_rejected() {
        let m = Monster::new(1, "Rat", 100);
        let self_pos = Position::new(100, 100, 7);
        let target_pos = Position::new(100, 100, 6);
        assert!(!m.is_valid_target_position(self_pos, target_pos));
    }

    #[test]
    fn test_is_valid_target_position_outside_viewport_rejected() {
        let m = Monster::new(1, "Rat", 100);
        let self_pos = Position::new(100, 100, 7);
        let target_pos = Position::new(115, 100, 7); // dx=15 > 9
        assert!(!m.is_valid_target_position(self_pos, target_pos));
    }

    #[test]
    fn test_is_valid_target_position_on_boundary() {
        let m = Monster::new(1, "Rat", 100);
        let self_pos = Position::new(100, 100, 7);
        // dx=9, dy=7 — exactly on boundary
        let target_pos = Position::new(109, 107, 7);
        assert!(m.is_valid_target_position(self_pos, target_pos));
    }

    // --- get_target_ids ---

    #[test]
    fn test_get_target_ids_empty() {
        let m = Monster::new(1, "Rat", 100);
        assert!(m.get_target_ids().is_empty());
    }

    #[test]
    fn test_get_target_ids_returns_all() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_target(10, 50);
        m.add_target(11, 80);
        let ids = m.get_target_ids();
        assert!(ids.contains(&10));
        assert!(ids.contains(&11));
    }

    // --- get_combat_values / set_combat_values ---

    #[test]
    fn test_get_combat_values_default_none() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(m.get_combat_values(), None);
    }

    #[test]
    fn test_set_combat_values_and_get() {
        let mut m = Monster::new(1, "Rat", 100);
        m.set_combat_values(-100, -50);
        assert_eq!(m.get_combat_values(), Some((-100, -50)));
    }

    #[test]
    fn test_get_combat_values_nonzero_min_only() {
        let mut m = Monster::new(1, "Rat", 100);
        m.set_combat_values(-10, 0);
        assert_eq!(m.get_combat_values(), Some((-10, 0)));
    }

    // --- challenge_creature ---

    #[test]
    fn test_challenge_creature_fails_for_summon() {
        let mut m = Monster::new(1, "Rat", 100);
        let accepted = m.challenge_creature(true, true);
        assert!(!accepted);
    }

    #[test]
    fn test_challenge_creature_fails_when_target_invalid() {
        let mut m = Monster::new(1, "Rat", 100);
        let accepted = m.challenge_creature(false, false);
        assert!(!accepted);
    }

    #[test]
    fn test_challenge_creature_sets_cooldown_and_focus() {
        let mut m = Monster::new(1, "Rat", 100);
        let accepted = m.challenge_creature(false, true);
        assert!(accepted);
        assert_eq!(m.ai_ticks.target_change_cooldown, 8000);
        assert_eq!(m.ai_ticks.challenge_focus_duration, 8000);
        assert_eq!(m.ai_ticks.target_change_ticks, 0);
    }

    // --- tick_target_change ---

    #[test]
    fn test_tick_target_change_disabled_when_speed_zero() {
        let mut m = Monster::new(1, "Rat", 100);
        assert!(!m.tick_target_change(1000, 0));
    }

    #[test]
    fn test_tick_target_change_accumulates_ticks() {
        let mut m = Monster::new(1, "Rat", 100);
        // speed = 3000 ms
        assert!(!m.tick_target_change(1000, 3000));
        assert!(!m.tick_target_change(1000, 3000));
        // Third tick: 3000 >= 3000 → fire
        assert!(m.tick_target_change(1000, 3000));
    }

    #[test]
    fn test_tick_target_change_cooldown_blocks_target_change() {
        let mut m = Monster::new(1, "Rat", 100);
        m.ai_ticks.target_change_cooldown = 2000;
        // During cooldown, should return false
        assert!(!m.tick_target_change(500, 1000));
        assert_eq!(m.ai_ticks.target_change_cooldown, 1500);
    }

    #[test]
    fn test_tick_target_change_cooldown_expires_resets_ticks() {
        let mut m = Monster::new(1, "Rat", 100);
        m.ai_ticks.target_change_cooldown = 500;
        // 600ms interval > 500ms cooldown → cooldown expires
        let result = m.tick_target_change(600, 2000);
        assert_eq!(m.ai_ticks.target_change_cooldown, 0);
        // target_change_ticks reset to change_speed on cooldown expiry
        assert_eq!(m.ai_ticks.target_change_ticks, 2000);
        // Still returns false (was in cooldown this tick)
        assert!(!result);
    }

    #[test]
    fn test_tick_target_change_challenge_focus_decremented() {
        let mut m = Monster::new(1, "Rat", 100);
        m.ai_ticks.challenge_focus_duration = 3000;
        m.tick_target_change(1000, 1000);
        assert_eq!(m.ai_ticks.challenge_focus_duration, 2000);
    }

    #[test]
    fn test_tick_target_change_challenge_focus_clamped_to_zero() {
        let mut m = Monster::new(1, "Rat", 100);
        m.ai_ticks.challenge_focus_duration = 500;
        m.tick_target_change(2000, 3000);
        assert_eq!(m.ai_ticks.challenge_focus_duration, 0);
    }

    // --- tick_yell ---

    #[test]
    fn test_tick_yell_disabled_when_speed_zero() {
        let mut m = Monster::new(1, "Rat", 100);
        assert!(!m.tick_yell(5000, 0));
    }

    #[test]
    fn test_tick_yell_accumulates_before_firing() {
        let mut m = Monster::new(1, "Rat", 100);
        assert!(!m.tick_yell(1000, 3000));
        assert!(!m.tick_yell(1000, 3000));
        assert!(m.tick_yell(1000, 3000)); // 3000 >= 3000 → fires
    }

    #[test]
    fn test_tick_yell_resets_after_firing() {
        let mut m = Monster::new(1, "Rat", 100);
        m.tick_yell(3000, 3000); // fires
        assert_eq!(m.ai_ticks.yell_ticks, 0);
    }

    #[test]
    fn test_tick_yell_single_large_interval() {
        let mut m = Monster::new(1, "Rat", 100);
        assert!(m.tick_yell(5000, 3000)); // 5000 >= 3000 → fires
    }

    // --- compute_state ---

    #[test]
    fn test_compute_state_idle_when_no_targets() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(m.compute_state(false), MonsterState::Idle);
    }

    #[test]
    fn test_compute_state_in_combat_when_targets_and_not_fleeing() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_target(5, 100);
        m.run_away_health = 0; // flee disabled
        assert_eq!(m.compute_state(false), MonsterState::InCombat);
    }

    #[test]
    fn test_compute_state_fleeing_when_health_below_run_away() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_target(5, 100);
        m.run_away_health = 30;
        m.creature.set_health(20); // below run_away_health
        assert_eq!(m.compute_state(false), MonsterState::Fleeing);
    }

    #[test]
    fn test_compute_state_in_combat_for_summon_even_below_threshold() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_target(5, 100);
        m.run_away_health = 50;
        m.creature.set_health(10); // below run_away_health
                                   // is_summon=true → is_fleeing returns false → InCombat
        assert_eq!(m.compute_state(true), MonsterState::InCombat);
    }

    // --- try_walk_to_spawn ---

    #[test]
    fn test_try_walk_to_spawn_no_spawn_returns_false() {
        let mut m = Monster::new(1, "Rat", 100);
        m.master_pos = Position::new(100, 100, 7);
        let result = m.try_walk_to_spawn(false, Position::new(110, 100, 7));
        assert!(!result);
        assert!(!m.walking_to_spawn);
    }

    #[test]
    fn test_try_walk_to_spawn_with_targets_returns_false() {
        let mut m = Monster::new(1, "Rat", 100);
        m.master_pos = Position::new(100, 100, 7);
        m.add_target(5, 100);
        let result = m.try_walk_to_spawn(true, Position::new(110, 100, 7));
        assert!(!result);
    }

    #[test]
    fn test_try_walk_to_spawn_already_walking_returns_false() {
        let mut m = Monster::new(1, "Rat", 100);
        m.master_pos = Position::new(100, 100, 7);
        m.walking_to_spawn = true;
        let result = m.try_walk_to_spawn(true, Position::new(110, 100, 7));
        assert!(!result);
    }

    #[test]
    fn test_try_walk_to_spawn_at_spawn_returns_false() {
        let mut m = Monster::new(1, "Rat", 100);
        let pos = Position::new(100, 100, 7);
        m.master_pos = pos;
        // distance = 0 → no need to walk
        let result = m.try_walk_to_spawn(true, pos);
        assert!(!result);
        assert!(!m.walking_to_spawn);
    }

    #[test]
    fn test_try_walk_to_spawn_starts_walking() {
        let mut m = Monster::new(1, "Rat", 100);
        m.master_pos = Position::new(100, 100, 7);
        let result = m.try_walk_to_spawn(true, Position::new(115, 100, 7));
        assert!(result);
        assert!(m.walking_to_spawn);
    }

    // --- on_walk_complete_spawn ---

    #[test]
    fn test_on_walk_complete_spawn_clears_flag() {
        let mut m = Monster::new(1, "Rat", 100);
        m.walking_to_spawn = true;
        m.on_walk_complete_spawn();
        assert!(!m.walking_to_spawn);
    }

    #[test]
    fn test_on_walk_complete_spawn_noop_when_not_walking() {
        let mut m = Monster::new(1, "Rat", 100);
        m.on_walk_complete_spawn(); // should not panic
        assert!(!m.walking_to_spawn);
    }

    // --- master_pos ---

    #[test]
    fn test_master_pos_default_is_origin() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(m.master_pos, Position::default());
    }

    #[test]
    fn test_master_pos_can_be_set() {
        let mut m = Monster::new(1, "Rat", 100);
        m.master_pos = Position::new(50, 75, 7);
        assert_eq!(m.master_pos, Position::new(50, 75, 7));
    }

    // --- ElementResistance struct ---

    #[test]
    fn test_element_resistance_new() {
        let r = ElementResistance::new(3, 25);
        assert_eq!(r.combat_type, 3);
        assert_eq!(r.modifier, 25);
    }

    // --- walking_to_spawn default ---

    #[test]
    fn test_walking_to_spawn_default_false() {
        let m = Monster::new(1, "Rat", 100);
        assert!(!m.walking_to_spawn);
    }

    // --- run_away_health default ---

    #[test]
    fn test_run_away_health_default_zero() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(m.run_away_health, 0);
    }

    // --- ai_ticks default ---

    #[test]
    fn test_ai_ticks_default_all_zero() {
        let m = Monster::new(1, "Rat", 100);
        assert_eq!(m.ai_ticks.attack_ticks, 0);
        assert_eq!(m.ai_ticks.defense_ticks, 0);
        assert_eq!(m.ai_ticks.yell_ticks, 0);
        assert_eq!(m.ai_ticks.target_change_ticks, 0);
        assert_eq!(m.ai_ticks.target_change_cooldown, 0);
        assert_eq!(m.ai_ticks.challenge_focus_duration, 0);
    }

    // --- TargetSearchType coverage ---

    #[test]
    fn test_target_search_type_attack_range_uses_index() {
        let m = Monster::new(1, "Rat", 100);
        let distances = vec![(20u32, 1i32), (21u32, 2i32)];
        assert_eq!(
            m.select_target_from(TargetSearchType::AttackRange, &distances, 1),
            Some(21)
        );
    }

    // --- multiple element resistances ---

    #[test]
    fn test_multiple_element_resistances() {
        let mut m = Monster::new(1, "Rat", 100);
        m.add_element_resistance(1, 50);
        m.add_element_resistance(2, 25);
        assert_eq!(m.get_element_modifier(1), 50);
        assert_eq!(m.get_element_modifier(2), 25);
        assert_eq!(m.element_resistances.len(), 2);
    }

    // --- partial-chance loot drop path ---
    //
    // The `roll_loot` function has three branches for a single block:
    //   1. `chance == 0`  → never drops (covered by `test_monster_roll_loot_never_drops`)
    //   2. `chance >= 100_000` → always drops (covered by `test_monster_roll_loot_always_drops`)
    //   3. `chance > 0 && chance < 100_000` → drops when `roll < chance`,
    //      skips when `roll >= chance`.
    //
    // These two tests exercise both sides of case (3) deterministically:
    // - chance = 99_999 drops for any seed where `roll` is in 0..99_998 (≈99.999%).
    // - chance = 1     skips for any seed where `roll` is in 1..99_999    (≈99.999%).
    //
    // Helper count below tracks across multiple seeds; the assertion uses a
    // count threshold rather than a break to keep llvm-cov line-coverage clean.
    #[test]
    fn test_monster_roll_loot_partial_chance_drops_when_roll_below_chance() {
        let mut m = Monster::new(1, "Rat", 30);
        // Partial chance: > 0 and < 100_000, very high so any seed lands a drop.
        m.add_loot_block(LootBlock::new(200, 2, 5, 99_999));

        let mut drop_count = 0u32;
        for seed in [1u64, 2, 3, 4, 5, 12345, 99999, 314159, 2718281] {
            let drops = m.roll_loot(seed);
            drop_count += drops.len() as u32;
            // Each drop must have the correct item_type_id and a count in [2,5].
            for (item_id, count) in &drops {
                assert_eq!(*item_id, 200);
                assert!((2..=5).contains(count));
            }
        }
        // With chance=99_999 and 9 seeds, drop_count must be > 0.
        assert!(
            drop_count > 0,
            "partial-chance drop never fired across 9 seeds"
        );
    }

    #[test]
    fn test_monster_roll_loot_partial_chance_skips_when_roll_above_chance() {
        let mut m = Monster::new(1, "Rat", 30);
        // Partial chance: very low (1 out of 100_000) so roll >= chance
        // for essentially any seed, covering the skip path.
        m.add_loot_block(LootBlock::new(202, 1, 1, 1));

        let mut skip_count = 0u32;
        for seed in [1u64, 2, 3, 4, 5, 12345, 99999, 314159, 2718281] {
            let drops = m.roll_loot(seed);
            if drops.is_empty() {
                skip_count += 1;
            }
        }
        // With chance=1 and 9 seeds, skip_count must be > 0.
        assert!(
            skip_count > 0,
            "partial-chance skip never fired across 9 seeds"
        );
    }
}
