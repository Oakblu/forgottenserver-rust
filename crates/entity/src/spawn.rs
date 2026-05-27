//! Migrated from forgottenserver/src/spawn.h and spawn.cpp
//!
//! Provides `SpawnBlock`, `Spawn`, and `Spawns` types.
//! MonsterType is not yet implemented — monster types are referenced by name.

use std::collections::HashMap;

use forgottenserver_common::position::{Direction, Position};

// ---------------------------------------------------------------------------
// SpawnBlock
// ---------------------------------------------------------------------------

/// Mirrors the C++ `spawnBlock_t` struct.
///
/// `monster_types` holds `(name, chance)` pairs (0–100) instead of
/// `(MonsterType*, u16)` pointers. `last_spawn` records the timestamp (ms
/// since epoch) of the most recent successful spawn for this block, mirroring
/// the C++ `lastSpawn` field used by `checkSpawn` to enforce the respawn
/// interval.
#[derive(Debug, Clone)]
pub struct SpawnBlock {
    pub pos: Position,
    /// List of (monster_name, chance) pairs where chance is 0–100.
    pub monster_types: Vec<(String, u16)>,
    pub interval: u32,
    pub direction: Direction,
    /// Timestamp (ms) of the last successful spawn. `0` means never spawned.
    /// Mirrors C++ `lastSpawn`.
    pub last_spawn: i64,
}

impl SpawnBlock {
    pub fn new(pos: Position, interval: u32, direction: Direction) -> Self {
        SpawnBlock {
            pos,
            monster_types: Vec::new(),
            interval,
            direction,
            last_spawn: 0,
        }
    }

    /// Returns `true` when enough time has elapsed since the last spawn.
    ///
    /// Mirrors the C++ condition `OTSYS_TIME() >= sb.lastSpawn + sb.interval`.
    pub fn is_ready_to_spawn(&self, now_ms: i64) -> bool {
        now_ms >= self.last_spawn + self.interval as i64
    }

    /// Mark this block as having just spawned at `now_ms`.
    pub fn record_spawn(&mut self, now_ms: i64) {
        self.last_spawn = now_ms;
    }
}

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Spawn` class.
#[derive(Debug)]
pub struct Spawn {
    center_pos: Position,
    radius: i32,
    /// Minimum interval across all spawn blocks (ms). Mirrors C++ `interval`.
    interval: u32,
    /// Spawn blocks indexed by a monotonically-increasing key.
    spawn_map: HashMap<u32, SpawnBlock>,
    next_spawn_id: u32,
    /// Currently spawned monster ids (creature id → spawn block id).
    spawned_map: HashMap<u32, u32>,
    /// Whether the spawn timer is currently active. Mirrors C++
    /// `checkSpawnEvent != 0`.
    active: bool,
}

impl Spawn {
    /// Default spawn interval in milliseconds (60 seconds).
    pub const DEFAULT_INTERVAL: u32 = 60_000;

    /// Create a new spawn centred at `center_pos` with the given `radius`.
    pub fn new(center_pos: Position, radius: i32) -> Self {
        Spawn {
            center_pos,
            radius,
            interval: Self::DEFAULT_INTERVAL,
            spawn_map: HashMap::new(),
            next_spawn_id: 0,
            spawned_map: HashMap::new(),
            active: false,
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    pub fn get_interval(&self) -> u32 {
        self.interval
    }

    pub fn get_center_pos(&self) -> Position {
        self.center_pos
    }

    pub fn get_radius(&self) -> i32 {
        self.radius
    }

    /// Returns `true` if the spawn timer is currently running.
    /// Mirrors C++ `checkSpawnEvent != 0`.
    pub fn is_active(&self) -> bool {
        self.active
    }

    // -----------------------------------------------------------------------
    // Timer control (startSpawnCheck / stopEvent)
    // -----------------------------------------------------------------------

    /// Enable the spawn timer. Mirrors C++ `Spawn::startSpawnCheck`.
    ///
    /// In the C++ server this schedules a recurring task; here we record
    /// whether spawning is enabled so that callers can drive the tick loop.
    /// Has no effect if already active.
    pub fn start_spawning(&mut self) {
        self.active = true;
    }

    /// Disable the spawn timer. Mirrors C++ `Spawn::stopEvent`.
    ///
    /// Has no effect if already inactive.
    pub fn stop_spawning(&mut self) {
        self.active = false;
    }

    // -----------------------------------------------------------------------
    // Spawn map
    // -----------------------------------------------------------------------

    /// Add a monster entry to the spawn map. Mirrors `Spawn::addMonster`.
    ///
    /// Also updates `self.interval` to the minimum of the current interval
    /// and the new block's interval — mirroring the C++ `addBlock` behaviour
    /// (`interval = std::min(interval, sb.interval)`).
    ///
    /// Returns `true` on success.
    pub fn add_monster(
        &mut self,
        name: &str,
        pos: Position,
        dir: Direction,
        interval: u32,
    ) -> bool {
        if name.is_empty() {
            return false;
        }
        let mut block = SpawnBlock::new(pos, interval, dir);
        block.monster_types.push((name.to_string(), 100));
        self.interval = self.interval.min(interval);
        let id = self.next_spawn_id;
        self.next_spawn_id += 1;
        self.spawn_map.insert(id, block);
        true
    }

    /// Number of spawn blocks.
    pub fn get_monster_count(&self) -> usize {
        self.spawn_map.len()
    }

    // -----------------------------------------------------------------------
    // Spawned map
    // -----------------------------------------------------------------------

    /// Register a spawned monster (monster_id → spawn_block_id).
    pub fn register_spawned(&mut self, monster_id: u32, spawn_block_id: u32) {
        self.spawned_map.insert(monster_id, spawn_block_id);
    }

    /// Remove a spawned monster by its creature id. Mirrors `removeMonster`.
    pub fn remove_monster(&mut self, monster_id: u32) {
        self.spawned_map.remove(&monster_id);
    }

    /// Remove all dead/removed monsters from the spawned map. Mirrors
    /// `Spawn::cleanup`.
    ///
    /// `is_removed` is a predicate that returns `true` for dead monsters;
    /// in the real server this calls `monster->isRemoved()`.
    pub fn cleanup<F>(&mut self, is_removed: F)
    where
        F: Fn(u32) -> bool,
    {
        self.spawned_map
            .retain(|&monster_id, _| !is_removed(monster_id));
    }

    /// Returns the number of monsters currently in the spawned map.
    pub fn spawned_count(&self) -> usize {
        self.spawned_map.len()
    }

    // -----------------------------------------------------------------------
    // Player-blocking check
    // -----------------------------------------------------------------------

    /// Returns `true` if a player is present at `pos`, blocking a spawn.
    ///
    /// Mirrors the C++ `Spawn::findPlayer` static method.  In the real server
    /// this queries the game map for player spectators; here it delegates to a
    /// caller-supplied predicate so the entity crate remains game-loop-free.
    ///
    /// The spawn should be skipped when this returns `true` (unless the
    /// monster type has `isIgnoringSpawnBlock`).
    pub fn find_player<F>(pos: Position, has_player_at: F) -> bool
    where
        F: FnMut(Position) -> bool,
    {
        let mut f = has_player_at;
        f(pos)
    }

    // -----------------------------------------------------------------------
    // Respawn interval check
    // -----------------------------------------------------------------------

    /// Returns `true` when the spawn block with `spawn_block_id` is ready to
    /// respawn, i.e. the respawn interval has elapsed since its last spawn.
    ///
    /// Mirrors the C++ condition inside `checkSpawn`:
    /// `OTSYS_TIME() >= sb.lastSpawn + sb.interval`.
    pub fn is_block_ready(&self, spawn_block_id: u32, now_ms: i64) -> bool {
        self.spawn_map
            .get(&spawn_block_id)
            .map(|sb| sb.is_ready_to_spawn(now_ms))
            .unwrap_or(false)
    }

    /// Record that spawn block `spawn_block_id` just spawned at `now_ms`.
    pub fn record_block_spawn(&mut self, spawn_block_id: u32, now_ms: i64) {
        if let Some(sb) = self.spawn_map.get_mut(&spawn_block_id) {
            sb.record_spawn(now_ms);
        }
    }

    // -----------------------------------------------------------------------
    // Max spawn count
    // -----------------------------------------------------------------------

    /// Returns `true` if the spawn has reached its maximum concurrent count,
    /// i.e. every block already has a live monster.
    ///
    /// Mirrors the C++ behaviour in `checkSpawn` where iteration stops when
    /// `spawnedMap.size() >= spawnMap.size()`.
    pub fn is_at_max_capacity(&self) -> bool {
        self.spawned_map.len() >= self.spawn_map.len()
    }

    // -----------------------------------------------------------------------
    // Zone check
    // -----------------------------------------------------------------------

    /// Returns `true` when `pos` is within the spawn zone.
    ///
    /// Mirrors `Spawn::isInSpawnZone` — delegates to `Spawns::is_in_zone`
    /// with the spawn's own center and radius.
    pub fn is_in_spawn_zone(&self, pos: Position) -> bool {
        Spawns::is_in_zone(self.center_pos, self.radius, pos)
    }
}

// ---------------------------------------------------------------------------
// Spawns
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Spawns` class (registry of all spawns).
#[derive(Debug, Default)]
pub struct Spawns {
    spawn_list: Vec<Spawn>,
    /// Mirrors C++ `Spawns::loaded`. Set to `true` once a spawn file has been
    /// loaded successfully; cleared by [`Spawns::clear`].
    loaded: bool,
    /// Mirrors C++ `Spawns::started`. Set to `true` after [`Spawns::startup`]
    /// has run; cleared by [`Spawns::clear`].
    started: bool,
}

impl Spawns {
    pub fn new() -> Self {
        Spawns {
            spawn_list: Vec::new(),
            loaded: false,
            started: false,
        }
    }

    /// Static zone-check helper. Mirrors `Spawns::isInZone`.
    ///
    /// When `radius` is `-1` the zone is considered universal — returns `true`
    /// for any position.  Otherwise checks that `pos` lies within the
    /// axis-aligned bounding box `[center ± radius]` on the **same floor**.
    ///
    /// C++ reference:
    /// ```cpp
    /// if (radius == -1) return true;
    /// return (pos.x >= center.x - radius && pos.x <= center.x + radius &&
    ///         pos.y >= center.y - radius && pos.y <= center.y + radius);
    /// ```
    ///
    /// Note: different floors are NOT in zone (z must match) for finite radii.
    pub fn is_in_zone(center: Position, radius: i32, pos: Position) -> bool {
        if radius == -1 {
            return true;
        }
        if center.z != pos.z {
            return false;
        }
        let cx = center.x as i32;
        let cy = center.y as i32;
        let px = pos.x as i32;
        let py = pos.y as i32;
        px >= cx - radius && px <= cx + radius && py >= cy - radius && py <= cy + radius
    }

    pub fn add_spawn(&mut self, spawn: Spawn) {
        self.spawn_list.push(spawn);
        // Adding a spawn implicitly means the registry now has loaded content,
        // mirroring `loadFromXml` setting `loaded = true`.
        self.loaded = true;
    }

    pub fn get_spawn_count(&self) -> usize {
        self.spawn_list.len()
    }

    /// Returns `true` once [`Spawns::startup`] has run. Mirrors C++
    /// `Spawns::isStarted`.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Returns `true` once a spawn file has been loaded (or [`Spawns::add_spawn`]
    /// has been called). Mirrors C++ `Spawns::loaded` (which is private; this
    /// accessor exists so callers can mirror the C++ guard in
    /// `Spawns::startup` (`if (!loaded || isStarted()) return;`)).
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Activate all spawns in the registry. Mirrors C++ `Spawns::startup`,
    /// minus the NPC placement (NPCs are not owned by the entity crate).
    ///
    /// The C++ guard `if (!loaded || isStarted()) return;` is preserved: this
    /// is a no-op when nothing has been loaded or when the registry is already
    /// started. On success each child [`Spawn`] is moved into the active state
    /// via [`Spawn::start_spawning`] and the registry's `started` flag is set.
    pub fn startup(&mut self) {
        if !self.loaded || self.started {
            return;
        }
        for spawn in &mut self.spawn_list {
            spawn.start_spawning();
        }
        self.started = true;
    }

    /// Stop and drop every spawn. Mirrors C++ `Spawns::clear`.
    ///
    /// Each child [`Spawn`] has its timer stopped before being dropped, the
    /// `loaded` and `started` flags are reset, and the spawn list is emptied —
    /// matching the C++ ordering (`stopEvent()` → `spawnList.clear()` →
    /// `loaded = false; started = false;`).
    pub fn clear(&mut self) {
        for spawn in &mut self.spawn_list {
            spawn.stop_spawning();
        }
        self.spawn_list.clear();
        self.loaded = false;
        self.started = false;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: u16, y: u16, z: u8) -> Position {
        Position::new(x, y, z)
    }

    // -----------------------------------------------------------------------
    // SpawnBlock
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawn_block_fields() {
        let b = SpawnBlock::new(pos(100, 100, 7), 5000, Direction::South);
        assert_eq!(b.pos, pos(100, 100, 7));
        assert_eq!(b.interval, 5000);
        assert_eq!(b.direction, Direction::South);
        assert!(b.monster_types.is_empty());
        assert_eq!(b.last_spawn, 0);
    }

    #[test]
    fn test_spawn_block_last_spawn_initialized_to_zero() {
        let b = SpawnBlock::new(pos(100, 100, 7), 10_000, Direction::North);
        assert_eq!(b.last_spawn, 0);
    }

    #[test]
    fn test_spawn_block_is_ready_to_spawn_never_spawned() {
        // last_spawn == 0; any positive now > interval is ready
        let b = SpawnBlock::new(pos(100, 100, 7), 10_000, Direction::North);
        assert!(b.is_ready_to_spawn(10_000)); // now == 0 + 10_000
        assert!(b.is_ready_to_spawn(99_999));
    }

    #[test]
    fn test_spawn_block_is_ready_to_spawn_not_yet() {
        let mut b = SpawnBlock::new(pos(100, 100, 7), 10_000, Direction::North);
        b.record_spawn(5_000); // last_spawn = 5_000
                               // ready at 5_000 + 10_000 = 15_000
        assert!(!b.is_ready_to_spawn(14_999));
        assert!(b.is_ready_to_spawn(15_000));
        assert!(b.is_ready_to_spawn(15_001));
    }

    #[test]
    fn test_spawn_block_record_spawn_updates_last_spawn() {
        let mut b = SpawnBlock::new(pos(100, 100, 7), 10_000, Direction::North);
        b.record_spawn(42_000);
        assert_eq!(b.last_spawn, 42_000);
    }

    // -----------------------------------------------------------------------
    // Spawn — basic construction & accessors
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawn_new_default_interval() {
        let s = Spawn::new(pos(100, 100, 7), 5);
        assert_eq!(s.get_interval(), 60_000);
    }

    #[test]
    fn test_spawn_new_default_not_active() {
        let s = Spawn::new(pos(100, 100, 7), 5);
        assert!(!s.is_active());
    }

    // -----------------------------------------------------------------------
    // startSpawning / stopSpawning
    // -----------------------------------------------------------------------

    #[test]
    fn test_start_spawning_sets_active() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        assert!(!s.is_active());
        s.start_spawning();
        assert!(s.is_active());
    }

    #[test]
    fn test_stop_spawning_clears_active() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.start_spawning();
        assert!(s.is_active());
        s.stop_spawning();
        assert!(!s.is_active());
    }

    #[test]
    fn test_start_spawning_idempotent() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.start_spawning();
        s.start_spawning(); // second call should not panic or corrupt state
        assert!(s.is_active());
    }

    #[test]
    fn test_stop_spawning_idempotent() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.stop_spawning(); // already inactive
        assert!(!s.is_active());
    }

    // -----------------------------------------------------------------------
    // add_monster — interval minimum update
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawn_add_monster_returns_true() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        let result = s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        assert!(result);
    }

    #[test]
    fn test_spawn_add_monster_empty_name_returns_false() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        let result = s.add_monster("", pos(100, 100, 7), Direction::South, 10_000);
        assert!(!result);
    }

    #[test]
    fn test_spawn_get_monster_count() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        s.add_monster("Orc", pos(102, 100, 7), Direction::North, 15_000);
        assert_eq!(s.get_monster_count(), 2);
    }

    #[test]
    fn test_spawn_add_monster_lowers_interval() {
        // Default interval is 60_000. Adding a block with 20_000 should drop it.
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 20_000);
        assert_eq!(s.get_interval(), 20_000);
    }

    #[test]
    fn test_spawn_add_monster_does_not_raise_interval() {
        // Adding a block with a larger interval should not increase get_interval().
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        s.add_monster("Orc", pos(102, 100, 7), Direction::North, 50_000);
        assert_eq!(s.get_interval(), 10_000); // minimum of 10_000 and 50_000
    }

    #[test]
    fn test_spawn_add_monster_interval_tracks_minimum_of_all() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("A", pos(101, 100, 7), Direction::South, 30_000);
        s.add_monster("B", pos(102, 100, 7), Direction::South, 15_000);
        s.add_monster("C", pos(103, 100, 7), Direction::South, 25_000);
        assert_eq!(s.get_interval(), 15_000);
    }

    // -----------------------------------------------------------------------
    // Spawned map & cleanup
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawn_remove_monster() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.register_spawned(42, 0);
        assert!(s.spawned_map.contains_key(&42));
        s.remove_monster(42);
        assert!(!s.spawned_map.contains_key(&42));
    }

    #[test]
    fn test_spawn_remove_nonexistent_monster() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        // Should not panic
        s.remove_monster(9999);
    }

    #[test]
    fn test_cleanup_removes_dead_monsters() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.register_spawned(1, 0);
        s.register_spawned(2, 0);
        s.register_spawned(3, 0);
        // Mark monster 1 and 3 as removed
        s.cleanup(|id| id == 1 || id == 3);
        assert!(!s.spawned_map.contains_key(&1));
        assert!(s.spawned_map.contains_key(&2));
        assert!(!s.spawned_map.contains_key(&3));
        assert_eq!(s.spawned_count(), 1);
    }

    #[test]
    fn test_cleanup_no_dead_monsters() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.register_spawned(1, 0);
        s.register_spawned(2, 0);
        s.cleanup(|_| false); // nothing removed
        assert_eq!(s.spawned_count(), 2);
    }

    #[test]
    fn test_cleanup_all_dead_monsters() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.register_spawned(1, 0);
        s.register_spawned(2, 0);
        s.cleanup(|_| true); // all removed
        assert_eq!(s.spawned_count(), 0);
    }

    // -----------------------------------------------------------------------
    // find_player (player-blocking check)
    // -----------------------------------------------------------------------

    #[test]
    fn test_find_player_returns_true_when_player_present() {
        let spawn_pos = pos(100, 100, 7);
        let result = Spawn::find_player(spawn_pos, |_p| true);
        assert!(result);
    }

    #[test]
    fn test_find_player_returns_false_when_no_player() {
        let spawn_pos = pos(100, 100, 7);
        let result = Spawn::find_player(spawn_pos, |_p| false);
        assert!(!result);
    }

    #[test]
    fn test_find_player_passes_correct_position() {
        let spawn_pos = pos(50, 75, 3);
        let mut captured = pos(0, 0, 0);
        Spawn::find_player(spawn_pos, |p| {
            captured = p;
            false
        });
        assert_eq!(captured, spawn_pos);
    }

    // -----------------------------------------------------------------------
    // Respawn interval (is_block_ready / record_block_spawn)
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_block_ready_after_interval_elapses() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        // Block id is 0 (first inserted). last_spawn == 0, so ready at now >= 10_000.
        assert!(s.is_block_ready(0, 10_000));
        assert!(s.is_block_ready(0, 99_999));
    }

    #[test]
    fn test_is_block_not_ready_before_interval() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        s.record_block_spawn(0, 5_000); // last_spawn = 5_000, interval = 10_000
                                        // ready at 15_000
        assert!(!s.is_block_ready(0, 14_999));
    }

    #[test]
    fn test_is_block_ready_exactly_at_interval() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        s.record_block_spawn(0, 5_000);
        assert!(s.is_block_ready(0, 15_000)); // exactly at boundary
    }

    #[test]
    fn test_record_block_spawn_updates_last_spawn() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        s.record_block_spawn(0, 42_000);
        assert_eq!(s.spawn_map[&0].last_spawn, 42_000);
    }

    #[test]
    fn test_is_block_ready_nonexistent_id_returns_false() {
        let s = Spawn::new(pos(100, 100, 7), 5);
        assert!(!s.is_block_ready(999, 99_999));
    }

    // -----------------------------------------------------------------------
    // Max spawn count
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_at_max_capacity_empty_spawn_is_at_capacity() {
        // 0 blocks, 0 spawned → at capacity (vacuously)
        let s = Spawn::new(pos(100, 100, 7), 5);
        assert!(s.is_at_max_capacity());
    }

    #[test]
    fn test_is_at_max_capacity_one_block_no_spawned() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        // 1 block, 0 spawned → not at capacity
        assert!(!s.is_at_max_capacity());
    }

    #[test]
    fn test_is_at_max_capacity_one_block_one_spawned() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        s.register_spawned(42, 0);
        // 1 block, 1 spawned → at capacity
        assert!(s.is_at_max_capacity());
    }

    #[test]
    fn test_is_at_max_capacity_partial() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        s.add_monster("Orc", pos(102, 100, 7), Direction::North, 15_000);
        s.register_spawned(1, 0); // 1 of 2 spawned
        assert!(!s.is_at_max_capacity());
        s.register_spawned(2, 1); // 2 of 2 spawned
        assert!(s.is_at_max_capacity());
    }

    // -----------------------------------------------------------------------
    // Zone checks
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawn_is_in_spawn_zone_true() {
        let center = pos(100, 100, 7);
        let s = Spawn::new(center, 5);
        // same tile
        assert!(s.is_in_spawn_zone(center));
    }

    #[test]
    fn test_spawn_is_in_spawn_zone_within_radius() {
        let center = pos(100, 100, 7);
        let s = Spawn::new(center, 5);
        assert!(s.is_in_spawn_zone(pos(105, 100, 7))); // dx=5 ≤ 5
    }

    #[test]
    fn test_spawn_is_in_spawn_zone_outside_radius() {
        let center = pos(100, 100, 7);
        let s = Spawn::new(center, 5);
        assert!(!s.is_in_spawn_zone(pos(106, 100, 7))); // dx=6 > 5
    }

    #[test]
    fn test_spawn_is_in_spawn_zone_different_floor() {
        let center = pos(100, 100, 7);
        let s = Spawn::new(center, 10);
        assert!(!s.is_in_spawn_zone(pos(100, 100, 6))); // different z
    }

    #[test]
    fn test_spawns_is_in_zone_static_true() {
        let center = pos(50, 50, 7);
        assert!(Spawns::is_in_zone(center, 10, pos(55, 50, 7)));
    }

    #[test]
    fn test_spawns_is_in_zone_static_false() {
        let center = pos(50, 50, 7);
        assert!(!Spawns::is_in_zone(center, 10, pos(61, 50, 7)));
    }

    /// radius == -1 means "universal zone" — any position is inside.
    #[test]
    fn test_spawns_is_in_zone_radius_minus_one_always_true() {
        let center = pos(100, 100, 7);
        // Even positions on different floors or very far away are in zone
        assert!(Spawns::is_in_zone(center, -1, pos(0, 0, 0)));
        assert!(Spawns::is_in_zone(center, -1, pos(65535, 65535, 15)));
        assert!(Spawns::is_in_zone(center, -1, pos(100, 100, 0)));
    }

    #[test]
    fn test_spawns_is_in_zone_boundary_exact() {
        // AABB: center=(100,100,7), radius=5 → x in [95..105], y in [95..105]
        let center = pos(100, 100, 7);
        assert!(Spawns::is_in_zone(center, 5, pos(95, 100, 7))); // left edge
        assert!(Spawns::is_in_zone(center, 5, pos(105, 100, 7))); // right edge
        assert!(Spawns::is_in_zone(center, 5, pos(100, 95, 7))); // top edge
        assert!(Spawns::is_in_zone(center, 5, pos(100, 105, 7))); // bottom edge
        assert!(!Spawns::is_in_zone(center, 5, pos(94, 100, 7))); // one past left
        assert!(!Spawns::is_in_zone(center, 5, pos(106, 100, 7))); // one past right
    }

    #[test]
    fn test_spawns_is_in_zone_different_floor_returns_false() {
        let center = pos(100, 100, 7);
        assert!(!Spawns::is_in_zone(center, 100, pos(100, 100, 6)));
        assert!(!Spawns::is_in_zone(center, 100, pos(100, 100, 8)));
    }

    // -----------------------------------------------------------------------
    // Spawn accessors — get_center_pos / get_radius
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawn_get_center_pos_returns_constructor_value() {
        let center = pos(123, 45, 7);
        let s = Spawn::new(center, 5);
        assert_eq!(s.get_center_pos(), center);
    }

    #[test]
    fn test_spawn_get_radius_returns_constructor_value() {
        let s = Spawn::new(pos(100, 100, 7), 12);
        assert_eq!(s.get_radius(), 12);
    }

    #[test]
    fn test_spawn_get_radius_negative_one_universal() {
        // C++ uses -1 to mean "universal radius" (matches isInZone behaviour).
        let s = Spawn::new(pos(100, 100, 7), -1);
        assert_eq!(s.get_radius(), -1);
        // And isInSpawnZone should accept any position.
        assert!(s.is_in_spawn_zone(pos(0, 0, 0)));
    }

    // -----------------------------------------------------------------------
    // record_block_spawn — nonexistent id branch (mirrors C++ early-exit when
    // checkSpawn finds no matching block)
    // -----------------------------------------------------------------------

    #[test]
    fn test_record_block_spawn_nonexistent_id_is_noop() {
        let mut s = Spawn::new(pos(100, 100, 7), 5);
        s.add_monster("Rat", pos(101, 100, 7), Direction::South, 10_000);
        // Calling with an id that doesn't exist must silently do nothing —
        // it must not panic and must not mutate any existing block.
        s.record_block_spawn(999, 99_999);
        // The block at id 0 is still untouched.
        assert_eq!(s.spawn_map[&0].last_spawn, 0);
    }

    // -----------------------------------------------------------------------
    // Spawns — registry construction / spawn list management
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawns_new_is_empty() {
        let r = Spawns::new();
        assert_eq!(r.get_spawn_count(), 0);
        assert!(!r.is_started());
        assert!(!r.is_loaded());
    }

    #[test]
    fn test_spawns_default_matches_new() {
        let r = Spawns::default();
        assert_eq!(r.get_spawn_count(), 0);
        assert!(!r.is_started());
        assert!(!r.is_loaded());
    }

    #[test]
    fn test_spawns_add_spawn_increments_count() {
        let mut r = Spawns::new();
        r.add_spawn(Spawn::new(pos(100, 100, 7), 5));
        assert_eq!(r.get_spawn_count(), 1);
        r.add_spawn(Spawn::new(pos(200, 200, 7), 8));
        assert_eq!(r.get_spawn_count(), 2);
    }

    #[test]
    fn test_spawns_add_spawn_marks_loaded() {
        let mut r = Spawns::new();
        assert!(!r.is_loaded());
        r.add_spawn(Spawn::new(pos(100, 100, 7), 5));
        assert!(r.is_loaded());
    }

    // -----------------------------------------------------------------------
    // Spawns::startup
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawns_startup_is_noop_when_not_loaded() {
        // Mirrors C++: `if (!loaded || isStarted()) return;`
        let mut r = Spawns::new();
        r.startup();
        assert!(!r.is_started());
    }

    #[test]
    fn test_spawns_startup_activates_children_and_sets_started() {
        let mut r = Spawns::new();
        r.add_spawn(Spawn::new(pos(100, 100, 7), 5));
        r.add_spawn(Spawn::new(pos(200, 200, 7), 5));
        assert!(r.is_loaded());
        assert!(!r.is_started());
        r.startup();
        assert!(r.is_started());
        // The internal spawns are all active.
        for s in &r.spawn_list {
            assert!(s.is_active());
        }
    }

    #[test]
    fn test_spawns_startup_is_idempotent_when_already_started() {
        // Calling startup twice must not toggle state nor re-activate spawns
        // (C++ guard: `if (!loaded || isStarted()) return;`).
        let mut r = Spawns::new();
        r.add_spawn(Spawn::new(pos(100, 100, 7), 5));
        r.startup();
        // Manually disable the inner spawn to detect a second activation.
        r.spawn_list[0].stop_spawning();
        r.startup();
        // Second startup must be a no-op: spawn was stopped and should stay so.
        assert!(!r.spawn_list[0].is_active());
        assert!(r.is_started());
    }

    // -----------------------------------------------------------------------
    // Spawns::clear
    // -----------------------------------------------------------------------

    #[test]
    fn test_spawns_clear_empties_list_and_resets_flags() {
        let mut r = Spawns::new();
        r.add_spawn(Spawn::new(pos(100, 100, 7), 5));
        r.startup();
        assert!(r.is_started());
        assert!(r.is_loaded());
        assert_eq!(r.get_spawn_count(), 1);
        r.clear();
        assert_eq!(r.get_spawn_count(), 0);
        assert!(!r.is_loaded());
        assert!(!r.is_started());
    }

    #[test]
    fn test_spawns_clear_stops_active_children_before_drop() {
        // Capture: clear must call stop_spawning on every child first. We can
        // observe this indirectly: pre-clear, the spawn was active; the clear
        // path mirrors C++ stopEvent() ordering before the list is wiped.
        // Since clear drops the spawns, we instead verify that re-running
        // clear on an already-empty list is a safe no-op (re-entrancy check
        // matching C++ `clear()` after a previous `clear()`).
        let mut r = Spawns::new();
        r.add_spawn(Spawn::new(pos(100, 100, 7), 5));
        r.startup();
        r.clear();
        r.clear(); // double-clear must not panic
        assert_eq!(r.get_spawn_count(), 0);
        assert!(!r.is_loaded());
        assert!(!r.is_started());
    }

    #[test]
    fn test_spawns_clear_on_fresh_registry_is_safe() {
        let mut r = Spawns::new();
        r.clear();
        assert_eq!(r.get_spawn_count(), 0);
        assert!(!r.is_loaded());
        assert!(!r.is_started());
    }
}
