use std::collections::VecDeque;
use std::time::{Duration, Instant};

use forgottenserver_entity::player::{Player, STAMINA_BONUS_ABOVE, STAMINA_EXHAUSTED_THRESHOLD};

// ---------------------------------------------------------------------------
// C++ constants (mirrors creature.h / game.h)
// ---------------------------------------------------------------------------

/// Number of creature check buckets (mirrors `EVENT_CREATURECOUNT = 10`).
pub const EVENT_CREATURECOUNT: usize = 10;

/// Full creature think cycle in ms (mirrors `EVENT_CREATURE_THINK_INTERVAL = 1000`).
pub const EVENT_CREATURE_THINK_INTERVAL_MS: u32 = 1000;

/// Per-bucket check interval in ms (mirrors `EVENT_CHECK_CREATURE_INTERVAL = 100`).
pub const EVENT_CHECK_CREATURE_INTERVAL_MS: u32 =
    EVENT_CREATURE_THINK_INTERVAL_MS / EVENT_CREATURECOUNT as u32;

/// Decay tick interval in ms (mirrors `EVENT_DECAYINTERVAL = 250`).
pub const EVENT_DECAY_INTERVAL_MS: u32 = 250;

/// Number of decay buckets (mirrors `EVENT_DECAY_BUCKETS = 4`).
pub const EVENT_DECAY_BUCKETS: usize = 4;

// ---------------------------------------------------------------------------
// World light cycle constants
//
// The C++ TFS uses a 24-hour in-game day. Each real second advances the world
// clock by a configurable multiplier. In TFS 1.x, the default is one in-game
// hour = 2.5 real minutes. Full day = 1 hour real time.
//
// We model this with a simple tick counter:
//   - `world_light_tick` increments by 1 each call to `tick_world_light`
//   - A full day cycle takes `LIGHT_CYCLE_TICKS` ticks
//   - Day (max brightness) is the top half; night (min brightness) is the
//     bottom half
// ---------------------------------------------------------------------------

/// Total ticks in one full in-game day (arbitrary unit; tests use this).
pub const LIGHT_CYCLE_TICKS: u32 = 2400;

/// Maximum (daytime) light level (0–255 scale used by TFS).
pub const LIGHT_LEVEL_DAY: u8 = 250;

/// Minimum (nighttime) light level.
pub const LIGHT_LEVEL_NIGHT: u8 = 40;

// ---------------------------------------------------------------------------
// DecayItem — an item tracked in the decay subsystem
// ---------------------------------------------------------------------------

/// A simple in-memory item tracked by the decay subsystem.
///
/// Mirrors the essential state accessed by `Game::checkDecay` /
/// `Game::internalDecayItem` in C++.
#[derive(Debug, Clone)]
pub struct DecayItem {
    /// Remaining duration in milliseconds.
    pub duration_ms: i32,
    /// Item type ID (0 means no decay-to transform; item is removed when done).
    pub decay_to: i32,
    /// Whether this item can decay at all.
    pub can_decay: bool,
    /// Whether the item has been consumed / removed by internal decay.
    pub removed: bool,
    /// If `decay_to > 0` this records the transform that occurred.
    pub transformed_to: Option<i32>,
}

impl DecayItem {
    /// Create a new decaying item.
    pub fn new(duration_ms: i32, decay_to: i32) -> Self {
        DecayItem {
            duration_ms,
            decay_to,
            can_decay: true,
            removed: false,
            transformed_to: None,
        }
    }

    /// Decrement duration by `delta_ms`.  Returns the new remaining duration.
    pub fn decrease_duration(&mut self, delta_ms: i32) -> i32 {
        self.duration_ms -= delta_ms;
        self.duration_ms
    }
}

// ---------------------------------------------------------------------------
// DecayResult — outcome of `internal_decay_item`
// ---------------------------------------------------------------------------

/// The outcome of processing a single decaying item.
#[derive(Debug, Clone, PartialEq)]
pub enum DecayResult {
    /// Item had `decay_to > 0` — it transformed into the given type ID.
    Transformed(i32),
    /// Item had `decay_to <= 0` — it was removed.
    Removed,
    /// Item `can_decay` was false — nothing happened.
    Skipped,
}

// ---------------------------------------------------------------------------
// CreatureCheckEntry — minimal creature record for check_creatures
// ---------------------------------------------------------------------------

/// Minimal data needed to run per-creature tick logic in `check_creatures`.
#[derive(Debug, Clone)]
pub struct CreatureCheckEntry {
    /// Creature ID.
    pub id: u32,
    /// Whether this creature should be ticked (mirrors `creatureCheck` flag).
    pub active: bool,
    /// Whether the creature is alive.
    pub alive: bool,
    /// Number of times `on_think` has been called on this entry.
    pub think_count: u32,
    /// Number of times `on_attacking` has been called on this entry.
    pub attacking_count: u32,
    /// Number of times `execute_conditions` has been called.
    pub conditions_count: u32,
}

impl CreatureCheckEntry {
    pub fn new(id: u32) -> Self {
        CreatureCheckEntry {
            id,
            active: true,
            alive: true,
            think_count: 0,
            attacking_count: 0,
            conditions_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// WorldLight — tracks the in-game day/night cycle
// ---------------------------------------------------------------------------

/// Tracks in-game world light level (day/night cycle).
#[derive(Debug, Clone)]
pub struct WorldLight {
    /// Current tick counter within the day cycle `[0, LIGHT_CYCLE_TICKS)`.
    pub tick: u32,
    /// Current computed light level.
    pub level: u8,
}

impl WorldLight {
    pub fn new() -> Self {
        WorldLight {
            tick: 0,
            level: LIGHT_LEVEL_DAY,
        }
    }

    /// Advance the world clock by one tick and recompute the light level.
    ///
    /// The day is split evenly: first half = daytime (level ramps from night
    /// at dawn to day at noon), second half = nighttime (ramps from day at
    /// noon back to night at midnight).
    ///
    /// For simplicity we use linear interpolation:
    ///   - tick 0         → LIGHT_LEVEL_DAY   (noon)
    ///   - tick HALF      → LIGHT_LEVEL_NIGHT  (midnight)
    ///   - tick HALF+1..  → rising back to day
    ///
    /// Returns the new light level.
    pub fn advance(&mut self) -> u8 {
        self.tick = (self.tick + 1) % LIGHT_CYCLE_TICKS;
        self.level = Self::compute_level(self.tick);
        self.level
    }

    /// Compute the light level for a given tick position.
    pub fn compute_level(tick: u32) -> u8 {
        let half = LIGHT_CYCLE_TICKS / 2;
        let day = LIGHT_LEVEL_DAY as i32;
        let night = LIGHT_LEVEL_NIGHT as i32;
        let range = day - night;

        if tick < half {
            // First half: day → night (decreasing)
            let fraction = tick as f32 / half as f32;
            (day as f32 - fraction * range as f32).round() as u8
        } else {
            // Second half: night → day (increasing)
            let fraction = (tick - half) as f32 / half as f32;
            (night as f32 + fraction * range as f32).round() as u8
        }
    }

    /// Returns `true` when the current tick is in the daytime half.
    pub fn is_day(&self) -> bool {
        self.tick < LIGHT_CYCLE_TICKS / 2
    }
}

impl Default for WorldLight {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// stamina_xp_multiplier
// ---------------------------------------------------------------------------

/// XP multiplier based on current stamina and premium status.
///
/// | Stamina range          | Multiplier |
/// |------------------------|-----------|
/// | < 840 min              | 0.5×      |
/// | 840–2340 min           | 1.0×      |
/// | > 2340 min + premium   | 1.5×      |
pub fn stamina_xp_multiplier(stamina: u16, is_premium: bool) -> f64 {
    if stamina < STAMINA_EXHAUSTED_THRESHOLD {
        0.5
    } else if is_premium && stamina > STAMINA_BONUS_ABOVE {
        1.5
    } else {
        1.0
    }
}

// ---------------------------------------------------------------------------
// internal_decay_item — mirrors Game::internalDecayItem
// ---------------------------------------------------------------------------

/// Process a single item whose duration has reached zero (or below).
///
/// Mirrors `Game::internalDecayItem`:
///   - If `decay_to > 0` → record the transformation and mark `transformed_to`.
///   - Otherwise → mark the item as `removed`.
///   - If `can_decay` is false → return `Skipped` (no changes).
pub fn internal_decay_item(item: &mut DecayItem) -> DecayResult {
    if !item.can_decay {
        return DecayResult::Skipped;
    }
    if item.decay_to > 0 {
        let to = item.decay_to;
        item.transformed_to = Some(to);
        DecayResult::Transformed(to)
    } else {
        item.removed = true;
        DecayResult::Removed
    }
}

// ---------------------------------------------------------------------------
// check_creature_attack — mirrors Game::checkCreatureAttack
// ---------------------------------------------------------------------------

/// Run an attack tick for all creatures in `list` that are alive.
///
/// For each alive creature:
///   - increments `attacking_count` (represents `creature->onAttacking(0)` in C++).
///
/// Dead creatures are silently skipped (mirrors C++ `!creature->isDead()` guard).
pub fn check_creature_attack(creatures: &mut [CreatureCheckEntry]) {
    for c in creatures.iter_mut() {
        if c.alive {
            c.attacking_count += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// check_creatures — mirrors Game::checkCreatures (one bucket)
// ---------------------------------------------------------------------------

/// Run a full think tick for all active, alive creatures in one bucket.
///
/// Mirrors C++ `Game::checkCreatures(index)`:
///   - For each creature with `active == true && alive == true`:
///     call `on_think`, `on_attacking`, `execute_conditions`
///     (represented as counter increments in our pure-memory model).
///   - Creatures with `active == false` are removed from the list.
///
/// Returns the number of creatures removed (no longer active).
pub fn check_creatures(bucket: &mut Vec<CreatureCheckEntry>) -> usize {
    let before = bucket.len();
    bucket.retain_mut(|c| {
        if !c.active {
            return false; // remove from list
        }
        if c.alive {
            c.think_count += 1;
            c.attacking_count += 1;
            c.conditions_count += 1;
        }
        true
    });
    before - bucket.len()
}

// ---------------------------------------------------------------------------
// CheckDecayState — result from process_decay_bucket
// ---------------------------------------------------------------------------

/// Summary of what happened when a decay bucket was processed.
#[derive(Debug, Default)]
pub struct DecayBucketResult {
    pub removed: usize,
    pub transformed: usize,
    pub rescheduled: usize,
}

// ---------------------------------------------------------------------------
// process_decay_bucket — mirrors Game::checkDecay (one bucket iteration)
// ---------------------------------------------------------------------------

/// Process one decay bucket: decrement durations, decay items that expire.
///
/// Mirrors the inner loop of `Game::checkDecay`:
///   - Each item's duration is decreased by `EVENT_DECAY_INTERVAL_MS *
///     EVENT_DECAY_BUCKETS` (= 1 000 ms, the full scan window).
///   - If duration ≤ 0 → call `internal_decay_item`.
///   - If duration < full window → reschedule (counted as rescheduled).
///   - Otherwise → leave in place.
///
/// Items that can no longer decay (`can_decay == false`) are removed from the
/// bucket without being decayed.
///
/// Returns a `DecayBucketResult` summary.
pub fn process_decay_bucket(bucket: &mut VecDeque<DecayItem>) -> DecayBucketResult {
    let decrease = (EVENT_DECAY_INTERVAL_MS * EVENT_DECAY_BUCKETS as u32) as i32;
    let window = decrease; // same value: 1000 ms

    let mut result = DecayBucketResult::default();
    let mut i = 0;

    while i < bucket.len() {
        let item = &mut bucket[i];

        if !item.can_decay {
            bucket.remove(i);
            continue;
        }

        let duration = item.duration_ms;
        let new_duration = duration - decrease;
        item.duration_ms = new_duration;

        if new_duration <= 0 {
            let item = bucket.remove(i).unwrap();
            let mut item = item;
            internal_decay_item(&mut item);
            if item.removed {
                result.removed += 1;
            } else if item.transformed_to.is_some() {
                result.transformed += 1;
            }
            // don't advance i
        } else if new_duration < window {
            // Should be rescheduled to a closer bucket (simplified: just count it)
            result.rescheduled += 1;
            i += 1;
        } else {
            i += 1;
        }
    }

    result
}

// ---------------------------------------------------------------------------
// tick_world_light — advance the world light cycle by one step
// ---------------------------------------------------------------------------

/// Advance the world light cycle by one tick.
///
/// Returns the new light level.  Callers should broadcast this to connected
/// players (not modelled here — pure state transition only).
pub fn tick_world_light(world_light: &mut WorldLight) -> u8 {
    world_light.advance()
}

// ---------------------------------------------------------------------------
// GameLoop
// ---------------------------------------------------------------------------

/// Drives periodic game-loop mechanics (stamina drain, creature AI, conditions, etc.).
pub struct GameLoop {
    pub clock: Box<dyn Fn() -> Instant>,
    pub last_stamina_tick: Instant,

    /// Creature check buckets (10 slots, one processed per tick).
    pub creature_buckets: [Vec<CreatureCheckEntry>; EVENT_CREATURECOUNT],

    /// Decay buckets (4 slots, one processed every 250 ms).
    pub decay_buckets: [VecDeque<DecayItem>; EVENT_DECAY_BUCKETS],

    /// Index of the last decay bucket processed.
    pub last_decay_bucket: usize,

    /// World light state.
    pub world_light: WorldLight,

    /// Tick counter for the current creature check bucket.
    pub creature_bucket_index: usize,
}

impl GameLoop {
    pub fn new() -> Self {
        let now = Instant::now();
        GameLoop {
            clock: Box::new(Instant::now),
            last_stamina_tick: now,
            creature_buckets: std::array::from_fn(|_| Vec::new()),
            decay_buckets: std::array::from_fn(|_| VecDeque::new()),
            last_decay_bucket: 0,
            world_light: WorldLight::new(),
            creature_bucket_index: 0,
        }
    }

    /// Construct a `GameLoop` with an injected clock — useful for deterministic tests.
    pub fn with_clock(clock: Box<dyn Fn() -> Instant>) -> Self {
        let now = (clock)();
        GameLoop {
            last_stamina_tick: now,
            clock,
            creature_buckets: std::array::from_fn(|_| Vec::new()),
            decay_buckets: std::array::from_fn(|_| VecDeque::new()),
            last_decay_bucket: 0,
            world_light: WorldLight::new(),
            creature_bucket_index: 0,
        }
    }

    /// Drain 1 stamina minute from each player once per real minute.
    pub fn tick_stamina(&mut self, players: &mut [Player], now: Instant) {
        if now.duration_since(self.last_stamina_tick) < Duration::from_secs(60) {
            return;
        }
        self.last_stamina_tick = now;
        for player in players.iter_mut() {
            player.drain_stamina(1);
        }
    }

    /// Add a creature to a random check bucket (mirrors `addCreatureCheck`).
    pub fn add_creature_check(&mut self, entry: CreatureCheckEntry) {
        // Simple round-robin assignment (production would use uniform_random).
        let bucket = (entry.id as usize) % EVENT_CREATURECOUNT;
        self.creature_buckets[bucket].push(entry);
    }

    /// Process the next creature check bucket (mirrors `checkCreatures`).
    ///
    /// Returns the number of inactive creatures removed from the bucket.
    pub fn tick_creature_bucket(&mut self) -> usize {
        let idx = self.creature_bucket_index;
        self.creature_bucket_index = (idx + 1) % EVENT_CREATURECOUNT;
        check_creatures(&mut self.creature_buckets[idx])
    }

    /// Process the next decay bucket (mirrors `checkDecay`).
    ///
    /// Returns a summary of what happened.
    pub fn tick_decay(&mut self) -> DecayBucketResult {
        let bucket = (self.last_decay_bucket + 1) % EVENT_DECAY_BUCKETS;
        let result = process_decay_bucket(&mut self.decay_buckets[bucket]);
        self.last_decay_bucket = bucket;
        result
    }

    /// Add an item to the appropriate decay bucket based on its duration.
    ///
    /// Mirrors `Game::cleanup` item-placement logic:
    ///   - duration >= full window → last bucket
    ///   - otherwise → `(last + 1 + dur/1000) % BUCKETS`
    pub fn start_decay(&mut self, item: DecayItem) {
        let dur = item.duration_ms as u32;
        let full_window = EVENT_DECAY_INTERVAL_MS * EVENT_DECAY_BUCKETS as u32;
        let bucket = if dur >= full_window {
            self.last_decay_bucket
        } else {
            (self.last_decay_bucket + 1 + (dur / 1000) as usize) % EVENT_DECAY_BUCKETS
        };
        self.decay_buckets[bucket].push_back(item);
    }

    /// Advance the world light cycle by one tick.
    ///
    /// Returns the new light level.
    pub fn tick_light(&mut self) -> u8 {
        tick_world_light(&mut self.world_light)
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ManualClock — test helper for deterministic timing
// ---------------------------------------------------------------------------

#[cfg(test)]
pub struct ManualClock {
    inner: std::sync::Arc<std::sync::Mutex<Instant>>,
}

#[cfg(test)]
impl Default for ManualClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl ManualClock {
    pub fn new() -> Self {
        ManualClock {
            inner: std::sync::Arc::new(std::sync::Mutex::new(Instant::now())),
        }
    }

    /// Advance the clock by `dur`.
    pub fn advance(&self, dur: Duration) {
        let mut t = self.inner.lock().unwrap();
        *t += dur;
    }

    /// Return a boxed clock function that reads from this `ManualClock`.
    pub fn as_clock_fn(&self) -> Box<dyn Fn() -> Instant> {
        let inner = std::sync::Arc::clone(&self.inner);
        Box::new(move || *inner.lock().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_entity::player::{Player, STAMINA_MAX};

    // -----------------------------------------------------------------------
    // Phase 1 — clock abstraction
    // -----------------------------------------------------------------------

    #[test]
    fn clock_can_be_injected() {
        let clock = ManualClock::new();
        let _gl = GameLoop::with_clock(clock.as_clock_fn());
    }

    // -----------------------------------------------------------------------
    // stamina_xp_multiplier
    // -----------------------------------------------------------------------

    #[test]
    fn stamina_xp_multiplier_halves_below_840() {
        assert_eq!(stamina_xp_multiplier(839, false), 0.5);
        assert_eq!(stamina_xp_multiplier(0, false), 0.5);
        assert_eq!(stamina_xp_multiplier(839, true), 0.5);
    }

    #[test]
    fn stamina_xp_multiplier_normal_at_840_to_2340() {
        assert_eq!(stamina_xp_multiplier(840, false), 1.0);
        assert_eq!(stamina_xp_multiplier(1500, false), 1.0);
        assert_eq!(stamina_xp_multiplier(2340, false), 1.0);
    }

    #[test]
    fn stamina_xp_multiplier_bonus_above_2340_premium() {
        assert_eq!(stamina_xp_multiplier(2341, true), 1.5);
        assert_eq!(stamina_xp_multiplier(STAMINA_MAX, true), 1.5);
    }

    #[test]
    fn stamina_xp_multiplier_no_bonus_above_2340_without_premium() {
        assert_eq!(stamina_xp_multiplier(2341, false), 1.0);
        assert_eq!(stamina_xp_multiplier(STAMINA_MAX, false), 1.0);
    }

    // -----------------------------------------------------------------------
    // tick_stamina
    // -----------------------------------------------------------------------

    #[test]
    fn stamina_drains_one_per_minute_while_online() {
        let mut game_loop = GameLoop::new();
        // set last tick to 61 seconds ago so the next tick fires immediately
        game_loop.last_stamina_tick = Instant::now() - Duration::from_secs(61);

        let mut players = vec![Player::new(1, "Hero", 1)];
        let initial = players[0].get_stamina();

        game_loop.tick_stamina(&mut players, Instant::now());

        assert_eq!(players[0].get_stamina(), initial - 1);
    }

    #[test]
    fn stamina_does_not_drain_before_one_minute() {
        let mut game_loop = GameLoop::new();
        // last tick was just now — should NOT drain
        let mut players = vec![Player::new(1, "Hero", 1)];
        let initial = players[0].get_stamina();

        game_loop.tick_stamina(&mut players, Instant::now());

        assert_eq!(players[0].get_stamina(), initial);
    }

    #[test]
    fn stamina_drain_updates_last_tick_timestamp() {
        let mut game_loop = GameLoop::new();
        let past = Instant::now() - Duration::from_secs(61);
        game_loop.last_stamina_tick = past;

        let mut players = vec![Player::new(1, "Hero", 1)];
        let now = Instant::now();
        game_loop.tick_stamina(&mut players, now);

        // last_stamina_tick should now be `now`, so a second call should not drain
        let after = players[0].get_stamina();
        game_loop.tick_stamina(&mut players, now); // same instant → no drain
        assert_eq!(players[0].get_stamina(), after);
    }

    // -----------------------------------------------------------------------
    // internal_decay_item
    // -----------------------------------------------------------------------

    #[test]
    fn internal_decay_item_removes_when_no_decay_to() {
        let mut item = DecayItem::new(0, 0);
        let result = internal_decay_item(&mut item);
        assert_eq!(result, DecayResult::Removed);
        assert!(item.removed);
    }

    #[test]
    fn internal_decay_item_transforms_when_decay_to_positive() {
        let mut item = DecayItem::new(0, 42);
        let result = internal_decay_item(&mut item);
        assert_eq!(result, DecayResult::Transformed(42));
        assert_eq!(item.transformed_to, Some(42));
        assert!(!item.removed);
    }

    #[test]
    fn internal_decay_item_skips_when_cannot_decay() {
        let mut item = DecayItem::new(0, 0);
        item.can_decay = false;
        let result = internal_decay_item(&mut item);
        assert_eq!(result, DecayResult::Skipped);
        assert!(!item.removed);
    }

    #[test]
    fn internal_decay_item_negative_decay_to_removes() {
        // decay_to <= 0 means removal (same as 0)
        let mut item = DecayItem::new(0, -1);
        let result = internal_decay_item(&mut item);
        assert_eq!(result, DecayResult::Removed);
        assert!(item.removed);
    }

    // -----------------------------------------------------------------------
    // DecayItem duration tracking
    // -----------------------------------------------------------------------

    #[test]
    fn decay_item_decrease_duration_decrements() {
        let mut item = DecayItem::new(5000, 0);
        let remaining = item.decrease_duration(1000);
        assert_eq!(remaining, 4000);
        assert_eq!(item.duration_ms, 4000);
    }

    #[test]
    fn decay_item_decrease_duration_can_go_negative() {
        let mut item = DecayItem::new(500, 0);
        let remaining = item.decrease_duration(1000);
        assert_eq!(remaining, -500);
    }

    #[test]
    fn decay_item_new_not_removed_by_default() {
        let item = DecayItem::new(1000, 0);
        assert!(!item.removed);
        assert!(item.can_decay);
        assert_eq!(item.transformed_to, None);
    }

    // -----------------------------------------------------------------------
    // process_decay_bucket
    // -----------------------------------------------------------------------

    #[test]
    fn check_decay_removes_item_with_zero_duration() {
        let mut bucket: VecDeque<DecayItem> = VecDeque::new();
        // Duration exactly equals the decrease window (1000 ms) — will hit 0
        bucket.push_back(DecayItem::new(1000, 0));
        let result = process_decay_bucket(&mut bucket);
        assert_eq!(result.removed, 1);
        assert!(bucket.is_empty());
    }

    #[test]
    fn check_decay_removes_item_with_duration_below_decrease() {
        let mut bucket: VecDeque<DecayItem> = VecDeque::new();
        bucket.push_back(DecayItem::new(500, 0)); // 500 < 1000 decrease
        let result = process_decay_bucket(&mut bucket);
        // 500 - 1000 = -500 ≤ 0 → removed
        assert_eq!(result.removed, 1);
        assert!(bucket.is_empty());
    }

    #[test]
    fn check_decay_transforms_item_with_decay_to() {
        let mut bucket: VecDeque<DecayItem> = VecDeque::new();
        bucket.push_back(DecayItem::new(1000, 99)); // decay_to=99
        let result = process_decay_bucket(&mut bucket);
        assert_eq!(result.transformed, 1);
        assert!(bucket.is_empty());
    }

    #[test]
    fn check_decay_leaves_item_with_long_duration() {
        let mut bucket: VecDeque<DecayItem> = VecDeque::new();
        // Duration >> decrease window — stays in bucket
        bucket.push_back(DecayItem::new(10_000, 0));
        let result = process_decay_bucket(&mut bucket);
        assert_eq!(result.removed, 0);
        assert_eq!(result.transformed, 0);
        assert!(!bucket.is_empty());
        // Duration should have been decremented
        assert_eq!(bucket[0].duration_ms, 9_000);
    }

    #[test]
    fn check_decay_skips_non_decaying_items() {
        let mut bucket: VecDeque<DecayItem> = VecDeque::new();
        let mut item = DecayItem::new(1000, 0);
        item.can_decay = false;
        bucket.push_back(item);
        let result = process_decay_bucket(&mut bucket);
        // Non-decaying items are removed from the bucket but NOT decayed
        assert_eq!(result.removed, 0);
        assert_eq!(result.transformed, 0);
        assert!(bucket.is_empty()); // removed from tracking
    }

    #[test]
    fn check_decay_processes_multiple_items() {
        let mut bucket: VecDeque<DecayItem> = VecDeque::new();
        bucket.push_back(DecayItem::new(1000, 0)); // expires → removed
        bucket.push_back(DecayItem::new(10_000, 0)); // survives
        bucket.push_back(DecayItem::new(500, 55)); // expires → transformed
        let result = process_decay_bucket(&mut bucket);
        assert_eq!(result.removed, 1);
        assert_eq!(result.transformed, 1);
        assert_eq!(bucket.len(), 1); // only the long-lived item remains
        assert_eq!(bucket[0].duration_ms, 9_000);
    }

    // -----------------------------------------------------------------------
    // check_creature_attack
    // -----------------------------------------------------------------------

    #[test]
    fn check_creature_attack_increments_alive_creature() {
        let mut creatures = vec![CreatureCheckEntry::new(1)];
        check_creature_attack(&mut creatures);
        assert_eq!(creatures[0].attacking_count, 1);
    }

    #[test]
    fn check_creature_attack_skips_dead_creature() {
        let mut creature = CreatureCheckEntry::new(1);
        creature.alive = false;
        let mut creatures = vec![creature];
        check_creature_attack(&mut creatures);
        assert_eq!(creatures[0].attacking_count, 0);
    }

    #[test]
    fn check_creature_attack_only_processes_alive_in_list() {
        let alive = CreatureCheckEntry::new(1);
        let mut dead = CreatureCheckEntry::new(2);
        dead.alive = false;
        let mut creatures = vec![alive.clone(), dead.clone()];
        check_creature_attack(&mut creatures);
        // alive: 1 attack tick; dead: 0
        assert_eq!(creatures[0].attacking_count, 1);
        assert_eq!(creatures[1].attacking_count, 0);
    }

    #[test]
    fn check_creature_attack_multiple_alive_all_incremented() {
        let c1 = CreatureCheckEntry::new(1);
        let c2 = CreatureCheckEntry::new(2);
        let c3 = CreatureCheckEntry::new(3);
        let mut creatures = vec![c1, c2, c3];
        check_creature_attack(&mut creatures);
        for c in &creatures {
            assert_eq!(c.attacking_count, 1);
        }
    }

    // -----------------------------------------------------------------------
    // check_creatures
    // -----------------------------------------------------------------------

    #[test]
    fn check_creatures_ticks_active_alive_creature() {
        let mut bucket = vec![CreatureCheckEntry::new(1)];
        check_creatures(&mut bucket);
        assert_eq!(bucket[0].think_count, 1);
        assert_eq!(bucket[0].attacking_count, 1);
        assert_eq!(bucket[0].conditions_count, 1);
    }

    #[test]
    fn check_creatures_does_not_tick_dead_creature() {
        let mut entry = CreatureCheckEntry::new(1);
        entry.alive = false;
        let mut bucket = vec![entry];
        check_creatures(&mut bucket);
        // Creature stays in bucket but is NOT ticked
        assert_eq!(bucket[0].think_count, 0);
        assert_eq!(bucket[0].attacking_count, 0);
    }

    #[test]
    fn check_creatures_removes_inactive_creature() {
        let mut entry = CreatureCheckEntry::new(1);
        entry.active = false;
        let mut bucket = vec![entry];
        let removed = check_creatures(&mut bucket);
        assert_eq!(removed, 1);
        assert!(bucket.is_empty());
    }

    #[test]
    fn check_creatures_keeps_active_creature_in_bucket() {
        let mut bucket = vec![CreatureCheckEntry::new(1)];
        let removed = check_creatures(&mut bucket);
        assert_eq!(removed, 0);
        assert_eq!(bucket.len(), 1);
    }

    #[test]
    fn check_creatures_mixed_active_inactive() {
        let active = CreatureCheckEntry::new(1); // active=true
        let mut inactive = CreatureCheckEntry::new(2);
        inactive.active = false;
        let mut bucket = vec![active, inactive];
        let removed = check_creatures(&mut bucket);
        assert_eq!(removed, 1);
        assert_eq!(bucket.len(), 1);
        assert_eq!(bucket[0].id, 1);
    }

    #[test]
    fn check_creatures_multiple_ticks_accumulate() {
        let mut bucket = vec![CreatureCheckEntry::new(1)];
        check_creatures(&mut bucket);
        check_creatures(&mut bucket);
        assert_eq!(bucket[0].think_count, 2);
    }

    // -----------------------------------------------------------------------
    // WorldLight
    // -----------------------------------------------------------------------

    #[test]
    fn world_light_initial_level_is_day() {
        let wl = WorldLight::new();
        assert_eq!(wl.level, LIGHT_LEVEL_DAY);
    }

    #[test]
    fn world_light_tick_zero_is_day() {
        // Tick 0 should compute to day level
        assert_eq!(WorldLight::compute_level(0), LIGHT_LEVEL_DAY);
    }

    #[test]
    fn world_light_halfway_tick_is_night() {
        let half = LIGHT_CYCLE_TICKS / 2;
        // At exactly half the cycle we are transitioning to night
        let level = WorldLight::compute_level(half);
        assert_eq!(level, LIGHT_LEVEL_NIGHT);
    }

    #[test]
    fn world_light_is_day_at_tick_zero() {
        let wl = WorldLight::new();
        assert!(wl.is_day());
    }

    #[test]
    fn world_light_is_not_day_at_half() {
        let mut wl = WorldLight::new();
        // Advance to the halfway point
        let half = LIGHT_CYCLE_TICKS / 2;
        for _ in 0..half {
            wl.advance();
        }
        assert!(!wl.is_day());
    }

    #[test]
    fn world_light_level_decreases_during_first_half() {
        let mut wl = WorldLight::new();
        let initial = wl.level;
        // After a few ticks into day, level should start decreasing
        for _ in 0..100 {
            wl.advance();
        }
        assert!(wl.level < initial);
    }

    #[test]
    fn world_light_level_increases_during_second_half() {
        let mut wl = WorldLight::new();
        // Advance to just past the halfway point
        let half = LIGHT_CYCLE_TICKS / 2;
        for _ in 0..=half {
            wl.advance();
        }
        let at_night = wl.level;
        // Advance further into the second half — level should rise
        for _ in 0..100 {
            wl.advance();
        }
        assert!(wl.level > at_night);
    }

    #[test]
    fn world_light_full_cycle_returns_to_day() {
        let mut wl = WorldLight::new();
        for _ in 0..LIGHT_CYCLE_TICKS {
            wl.advance();
        }
        // After a full cycle we are back at tick 0
        assert_eq!(wl.tick, 0);
        assert_eq!(wl.level, LIGHT_LEVEL_DAY);
    }

    #[test]
    fn tick_world_light_returns_new_level() {
        let mut wl = WorldLight::new();
        let level = tick_world_light(&mut wl);
        // After 1 tick from tick=0, we're at tick=1 which is slightly below day
        assert!(level <= LIGHT_LEVEL_DAY);
        assert!(level >= LIGHT_LEVEL_NIGHT);
    }

    // -----------------------------------------------------------------------
    // GameLoop integration — creature buckets
    // -----------------------------------------------------------------------

    #[test]
    fn game_loop_add_creature_and_tick_bucket() {
        let mut gl = GameLoop::new();
        let entry = CreatureCheckEntry::new(1);
        let bucket_idx = 1 % EVENT_CREATURECOUNT;
        gl.creature_buckets[bucket_idx].push(entry);

        // Advance to bucket 1
        gl.creature_bucket_index = bucket_idx;
        gl.tick_creature_bucket();

        assert_eq!(gl.creature_buckets[bucket_idx][0].think_count, 1);
    }

    #[test]
    fn game_loop_add_creature_check_assigns_to_bucket() {
        let mut gl = GameLoop::new();
        let entry = CreatureCheckEntry::new(5);
        gl.add_creature_check(entry);
        let expected_bucket = 5 % EVENT_CREATURECOUNT;
        assert!(!gl.creature_buckets[expected_bucket].is_empty());
    }

    // -----------------------------------------------------------------------
    // GameLoop integration — decay
    // -----------------------------------------------------------------------

    #[test]
    fn game_loop_start_decay_places_in_bucket() {
        let mut gl = GameLoop::new();
        let item = DecayItem::new(500, 0); // short duration
        gl.start_decay(item);
        // At least one bucket should have the item
        let total: usize = gl.decay_buckets.iter().map(|b| b.len()).sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn game_loop_tick_decay_removes_expired_item() {
        let mut gl = GameLoop::new();
        // Place a short-lived item directly into decay bucket 1 (which will be
        // the next bucket processed since last_decay_bucket starts at 0)
        gl.decay_buckets[1].push_back(DecayItem::new(1000, 0));
        let result = gl.tick_decay();
        assert_eq!(result.removed, 1);
    }

    // -----------------------------------------------------------------------
    // GameLoop integration — light
    // -----------------------------------------------------------------------

    #[test]
    fn game_loop_tick_light_changes_level() {
        let mut gl = GameLoop::new();
        let initial = gl.world_light.level;
        // Advance 100 ticks — level should have changed
        let mut last = initial;
        for _ in 0..100 {
            last = gl.tick_light();
        }
        // After 100 ticks of 2400 total, we're 100/1200 into first half → decreasing
        assert!(last < initial);
    }

    #[test]
    fn game_loop_tick_light_returns_consistent_level() {
        let mut gl = GameLoop::new();
        let returned = gl.tick_light();
        assert_eq!(returned, gl.world_light.level);
    }
}
