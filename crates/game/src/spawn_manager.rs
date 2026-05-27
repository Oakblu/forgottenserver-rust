//! Spawn and respawn loop — tracks spawn points, responds to creature death,
//! and returns IDs of entries ready to re-populate on each game tick.
//!
//! Mirrors the C++ `Spawn` / `Spawns` classes from `spawn.cpp`:
//!  - Zone membership check (`is_in_zone`)
//!  - Player presence detection within a spawn zone (`has_player_in_zone`)
//!  - Interval validation (10 s – 86 400 s, matching C++ MINSPAWN_INTERVAL / MAXSPAWN_INTERVAL)
//!  - Multi-monster blocks with chance-based selection (`SpawnBlock`)
//!  - Cleanup of dead/removed creatures from the spawned tracking map

use std::collections::HashMap;
use std::time::Instant;

use forgottenserver_common::position::Position;
use forgottenserver_world::World;

pub type SpawnId = u32;
pub type CreatureId = u32;

// ── C++ constants ────────────────────────────────────────────────────────────
/// Minimum spawn interval (10 seconds), matches `MINSPAWN_INTERVAL` in C++.
pub const MIN_SPAWN_INTERVAL_SECS: u32 = 10;
/// Maximum spawn interval (24 h), matches `MAXSPAWN_INTERVAL` in C++.
pub const MAX_SPAWN_INTERVAL_SECS: u32 = 24 * 60 * 60;

// ── Monster type candidate ───────────────────────────────────────────────────

/// One candidate monster within a spawn block with an associated spawn chance
/// (0–100). Mirrors `std::pair<MonsterType*, uint16_t>` in C++ `spawnBlock_t`.
#[derive(Debug, Clone)]
pub struct MonsterCandidate {
    pub monster_name: String,
    /// Percentage chance (1–100). All candidates in a block should sum ≤ 100.
    pub chance: u8,
}

// ── Spawn block ──────────────────────────────────────────────────────────────

/// A single spawn slot within a `SpawnEntry`: a position, a respawn interval,
/// and one or more monster candidates (with chances).  Corresponds to
/// `spawnBlock_t` in C++.
#[derive(Debug, Clone)]
pub struct SpawnBlock {
    pub position: Position,
    pub interval_secs: u32,
    pub candidates: Vec<MonsterCandidate>,
}

impl SpawnBlock {
    /// Returns the first candidate whose cumulative chance exceeds `roll`.
    /// `roll` should be in 1..=100.  Falls back to the first candidate if
    /// nothing matches (mirrors C++ fallback path in `spawnMonster`).
    pub fn pick_monster(&self, roll: u8) -> Option<&MonsterCandidate> {
        if self.candidates.is_empty() {
            return None;
        }
        let mut cumulative: u8 = 0;
        for c in &self.candidates {
            cumulative = cumulative.saturating_add(c.chance);
            if roll <= cumulative {
                return Some(c);
            }
        }
        // Fallback — return first (mirrors C++ "just try to spawn something")
        self.candidates.first()
    }
}

// ── Spawn state ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SpawnState {
    /// Entry was just loaded from world data — spawn immediately on first tick.
    ReadyToSpawn,
    /// A live creature occupies this spawn point.
    Alive,
    /// The creature was killed; respawn after `interval_secs` from `killed_at`.
    Dead { killed_at: Instant },
}

// ── Spawn entry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SpawnEntry {
    pub spawn_id: SpawnId,
    pub position: Position,
    pub radius: u8,
    pub monster_name: String,
    pub interval_secs: u32,
    pub state: SpawnState,
    /// ID of the live creature currently occupying this slot (if any).
    pub live_creature_id: Option<CreatureId>,
}

// ── Spawn manager ────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct SpawnManager {
    entries: HashMap<SpawnId, SpawnEntry>,
    next_id: SpawnId,
}

impl SpawnManager {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Zone helpers ─────────────────────────────────────────────────────────

    /// Returns `true` when `pos` lies within the Chebyshev square centred on
    /// `center` with the given `radius`.  A negative radius means "entire map"
    /// (mirrors C++ `radius == -1` in `Spawns::isInZone`).
    pub fn is_in_zone(center: &Position, radius: i32, pos: &Position) -> bool {
        if radius < 0 {
            return true;
        }
        let r = radius;
        let dx = (pos.x as i32) - (center.x as i32);
        let dy = (pos.y as i32) - (center.y as i32);
        dx.abs() <= r && dy.abs() <= r
    }

    /// Returns `true` when any creature in `creature_positions` is a player
    /// present inside the zone of `entry`.  Mirrors C++ `Spawn::findPlayer`.
    pub fn has_player_in_zone(entry: &SpawnEntry, player_positions: &[Position]) -> bool {
        for p in player_positions {
            if Self::is_in_zone(&entry.position, entry.radius as i32, p) {
                return true;
            }
        }
        false
    }

    // ── Interval validation ───────────────────────────────────────────────────

    /// Validates a spawn interval.  Returns `Err` with a human-readable reason
    /// when the value is outside the C++ bounds.
    pub fn validate_interval(interval_secs: u32) -> Result<(), String> {
        if interval_secs < MIN_SPAWN_INTERVAL_SECS {
            return Err(format!(
                "spawn interval {} s is below minimum {} s",
                interval_secs, MIN_SPAWN_INTERVAL_SECS
            ));
        }
        if interval_secs > MAX_SPAWN_INTERVAL_SECS {
            return Err(format!(
                "spawn interval {} s exceeds maximum {} s",
                interval_secs, MAX_SPAWN_INTERVAL_SECS
            ));
        }
        Ok(())
    }

    // ── World loading ─────────────────────────────────────────────────────────

    /// Register all spawn points from the loaded world. Each entry starts as
    /// `ReadyToSpawn` so the first call to `tick` spawns every creature.
    pub fn load_world(&mut self, world: &World) {
        for sp in world.spawn_points() {
            let id = self.next_id;
            self.next_id += 1;
            self.entries.insert(
                id,
                SpawnEntry {
                    spawn_id: id,
                    position: sp.position,
                    radius: sp.radius,
                    monster_name: sp.monster_name.clone(),
                    interval_secs: sp.interval_secs,
                    state: SpawnState::ReadyToSpawn,
                    live_creature_id: None,
                },
            );
        }
    }

    // ── Creature lifecycle ────────────────────────────────────────────────────

    /// Record a creature death so the entry starts counting its respawn interval.
    pub fn on_creature_killed(&mut self, spawn_id: SpawnId, now: Instant) {
        if let Some(entry) = self.entries.get_mut(&spawn_id) {
            entry.state = SpawnState::Dead { killed_at: now };
            entry.live_creature_id = None;
        }
    }

    /// Associate a live creature ID with a spawn entry (called after placement).
    pub fn on_creature_placed(&mut self, spawn_id: SpawnId, creature_id: CreatureId) {
        if let Some(entry) = self.entries.get_mut(&spawn_id) {
            entry.state = SpawnState::Alive;
            entry.live_creature_id = Some(creature_id);
        }
    }

    /// Remove stale entries from the spawned tracking map — analogous to
    /// C++ `Spawn::cleanup()`.  Entries whose `live_creature_id` matches any ID
    /// in the `removed_ids` set are transitioned back to `Dead` state.
    pub fn cleanup(&mut self, removed_ids: &[CreatureId], now: Instant) {
        for entry in self.entries.values_mut() {
            if let Some(cid) = entry.live_creature_id {
                if removed_ids.contains(&cid) {
                    entry.state = SpawnState::Dead { killed_at: now };
                    entry.live_creature_id = None;
                }
            }
        }
    }

    // ── Tick ──────────────────────────────────────────────────────────────────

    /// Advance the spawn clock. Returns IDs of entries ready for a new creature:
    /// `ReadyToSpawn` entries fire immediately (first boot wave); `Dead` entries
    /// fire once their `interval_secs` have elapsed. All returned entries
    /// transition to `Alive`.
    pub fn tick(&mut self, now: Instant) -> Vec<SpawnId> {
        self.entries
            .iter_mut()
            .filter_map(|(id, entry)| match &entry.state {
                SpawnState::ReadyToSpawn => {
                    entry.state = SpawnState::Alive;
                    Some(*id)
                }
                SpawnState::Dead { killed_at } => {
                    if now.duration_since(*killed_at).as_secs() >= entry.interval_secs as u64 {
                        entry.state = SpawnState::Alive;
                        Some(*id)
                    } else {
                        None
                    }
                }
                SpawnState::Alive => None,
            })
            .collect()
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn entry(&self, spawn_id: SpawnId) -> Option<&SpawnEntry> {
        self.entries.get(&spawn_id)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Iterator over all entries.
    pub fn entries(&self) -> impl Iterator<Item = &SpawnEntry> {
        self.entries.values()
    }

    /// Number of entries currently in `Alive` state.
    pub fn alive_count(&self) -> usize {
        self.entries
            .values()
            .filter(|e| matches!(e.state, SpawnState::Alive))
            .count()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_world::SpawnPointDef;
    use std::time::Duration;

    fn pos(x: u16, y: u16, z: u8) -> Position {
        Position::new(x, y, z)
    }

    fn world_with_two_spawns() -> World {
        let mut w = World::new();
        w.add_spawn_point(SpawnPointDef {
            position: pos(100, 100, 7),
            radius: 3,
            monster_name: "Rat".to_string(),
            interval_secs: 60,
        });
        w.add_spawn_point(SpawnPointDef {
            position: pos(200, 200, 7),
            radius: 5,
            monster_name: "Orc".to_string(),
            interval_secs: 120,
        });
        w
    }

    // ── Phase 1 — load_world ─────────────────────────────────────────────────

    #[test]
    fn load_world_registers_all_spawn_points() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        assert_eq!(sm.entry_count(), 2);
    }

    #[test]
    fn load_world_entries_have_correct_fields() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);

        let rat = (0..2u32)
            .filter_map(|id| sm.entry(id))
            .find(|e| e.monster_name == "Rat")
            .expect("Rat entry must exist");

        assert_eq!(rat.position, pos(100, 100, 7));
        assert_eq!(rat.radius, 3);
        assert_eq!(rat.interval_secs, 60);
    }

    #[test]
    fn load_world_entries_start_with_no_live_creature() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        for e in sm.entries() {
            assert!(
                e.live_creature_id.is_none(),
                "newly loaded entry must not have a live creature"
            );
        }
    }

    // ── Phase 2 — on_creature_killed ─────────────────────────────────────────

    #[test]
    fn on_creature_killed_transitions_entry_to_dead() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);

        let now = Instant::now();
        sm.tick(now);

        sm.on_creature_killed(0, now);

        let entry = sm.entry(0).unwrap();
        assert!(
            matches!(entry.state, SpawnState::Dead { .. }),
            "entry must be Dead after on_creature_killed"
        );
    }

    #[test]
    fn on_creature_killed_clears_live_creature_id() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);

        let now = Instant::now();
        sm.tick(now);
        sm.on_creature_placed(0, 42);
        sm.on_creature_killed(0, now);

        assert!(sm.entry(0).unwrap().live_creature_id.is_none());
    }

    #[test]
    fn on_creature_killed_unknown_id_is_noop() {
        let mut sm = SpawnManager::new();
        sm.on_creature_killed(999, Instant::now());
    }

    // ── Phase 3 — tick ───────────────────────────────────────────────────────

    #[test]
    fn tick_before_interval_returns_empty() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);

        let now = Instant::now();
        sm.tick(now);

        sm.on_creature_killed(0, now);

        let ready = sm.tick(now);
        assert!(
            !ready.contains(&0),
            "should not respawn before interval elapses"
        );
    }

    #[test]
    fn tick_after_interval_returns_spawn_id_and_marks_alive() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);

        let t0 = Instant::now();
        sm.tick(t0);

        sm.on_creature_killed(0, t0);

        let t1 = t0 + Duration::from_secs(61);
        let ready = sm.tick(t1);

        assert!(ready.contains(&0), "entry 0 must be ready after interval");

        let entry = sm.entry(0).unwrap();
        assert!(
            matches!(entry.state, SpawnState::Alive),
            "entry must be Alive after respawn tick"
        );
    }

    #[test]
    fn initial_boot_all_entries_alive_spawns_on_first_tick() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);

        let now = Instant::now();
        let ready = sm.tick(now);

        assert_eq!(
            ready.len(),
            2,
            "all entries must spawn on first tick after load_world"
        );
    }

    #[test]
    fn alive_entry_not_returned_by_tick() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);

        let t0 = Instant::now();
        sm.tick(t0);

        let t1 = t0 + Duration::from_secs(200);
        let ready = sm.tick(t1);
        assert!(
            ready.is_empty(),
            "Alive entries must not be returned by tick"
        );
    }

    // ── Phase 4 — on_creature_placed ─────────────────────────────────────────

    #[test]
    fn on_creature_placed_records_creature_id() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let now = Instant::now();
        sm.tick(now);

        sm.on_creature_placed(0, 77);
        assert_eq!(sm.entry(0).unwrap().live_creature_id, Some(77));
    }

    #[test]
    fn on_creature_placed_marks_entry_alive() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let now = Instant::now();
        sm.tick(now);
        sm.on_creature_killed(0, now);

        sm.on_creature_placed(0, 77);
        assert!(matches!(sm.entry(0).unwrap().state, SpawnState::Alive));
    }

    #[test]
    fn on_creature_placed_unknown_id_is_noop() {
        let mut sm = SpawnManager::new();
        sm.on_creature_placed(999, 42); // must not panic
    }

    // ── Phase 5 — cleanup ────────────────────────────────────────────────────

    #[test]
    fn cleanup_transitions_entry_with_removed_creature_to_dead() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let t0 = Instant::now();
        sm.tick(t0);
        sm.on_creature_placed(0, 55);
        sm.on_creature_placed(1, 66);

        sm.cleanup(&[55], t0);

        assert!(
            matches!(sm.entry(0).unwrap().state, SpawnState::Dead { .. }),
            "entry 0 creature 55 was removed — must be Dead"
        );
        // entry 1 creature still alive
        assert!(
            matches!(sm.entry(1).unwrap().state, SpawnState::Alive),
            "entry 1 not in removed list — must remain Alive"
        );
    }

    #[test]
    fn cleanup_clears_live_creature_id_for_removed_creature() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let t0 = Instant::now();
        sm.tick(t0);
        sm.on_creature_placed(0, 55);

        sm.cleanup(&[55], t0);

        assert!(sm.entry(0).unwrap().live_creature_id.is_none());
    }

    #[test]
    fn cleanup_with_empty_removed_list_changes_nothing() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let t0 = Instant::now();
        sm.tick(t0);
        sm.on_creature_placed(0, 55);

        sm.cleanup(&[], t0);

        assert!(matches!(sm.entry(0).unwrap().state, SpawnState::Alive));
    }

    // ── Phase 6 — is_in_zone ─────────────────────────────────────────────────

    #[test]
    fn is_in_zone_center_is_inside() {
        let center = pos(100, 100, 7);
        assert!(SpawnManager::is_in_zone(&center, 3, &center));
    }

    #[test]
    fn is_in_zone_edge_is_inside() {
        let center = pos(100, 100, 7);
        assert!(SpawnManager::is_in_zone(&center, 3, &pos(103, 100, 7)));
        assert!(SpawnManager::is_in_zone(&center, 3, &pos(100, 97, 7)));
    }

    #[test]
    fn is_in_zone_outside_is_false() {
        let center = pos(100, 100, 7);
        assert!(!SpawnManager::is_in_zone(&center, 3, &pos(104, 100, 7)));
        assert!(!SpawnManager::is_in_zone(&center, 3, &pos(100, 96, 7)));
    }

    #[test]
    fn is_in_zone_negative_radius_always_true() {
        let center = pos(100, 100, 7);
        assert!(SpawnManager::is_in_zone(&center, -1, &pos(0, 0, 7)));
        assert!(SpawnManager::is_in_zone(
            &center,
            -1,
            &pos(65535, 65535, 15)
        ));
    }

    #[test]
    fn is_in_zone_zero_radius_only_center() {
        let center = pos(100, 100, 7);
        assert!(SpawnManager::is_in_zone(&center, 0, &center));
        assert!(!SpawnManager::is_in_zone(&center, 0, &pos(101, 100, 7)));
    }

    // ── Phase 7 — has_player_in_zone ─────────────────────────────────────────

    #[test]
    fn has_player_in_zone_player_inside_returns_true() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let entry = sm.entry(0).unwrap().clone();
        // center=100,100 radius=3
        let players = vec![pos(101, 101, 7)];
        assert!(SpawnManager::has_player_in_zone(&entry, &players));
    }

    #[test]
    fn has_player_in_zone_player_outside_returns_false() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let entry = sm.entry(0).unwrap().clone();
        let players = vec![pos(200, 200, 7)];
        assert!(!SpawnManager::has_player_in_zone(&entry, &players));
    }

    #[test]
    fn has_player_in_zone_no_players_returns_false() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let entry = sm.entry(0).unwrap().clone();
        assert!(!SpawnManager::has_player_in_zone(&entry, &[]));
    }

    // ── Phase 8 — validate_interval ──────────────────────────────────────────

    #[test]
    fn validate_interval_min_boundary_ok() {
        assert!(SpawnManager::validate_interval(MIN_SPAWN_INTERVAL_SECS).is_ok());
    }

    #[test]
    fn validate_interval_max_boundary_ok() {
        assert!(SpawnManager::validate_interval(MAX_SPAWN_INTERVAL_SECS).is_ok());
    }

    #[test]
    fn validate_interval_below_min_is_err() {
        assert!(SpawnManager::validate_interval(MIN_SPAWN_INTERVAL_SECS - 1).is_err());
    }

    #[test]
    fn validate_interval_above_max_is_err() {
        assert!(SpawnManager::validate_interval(MAX_SPAWN_INTERVAL_SECS + 1).is_err());
    }

    #[test]
    fn validate_interval_zero_is_err() {
        assert!(SpawnManager::validate_interval(0).is_err());
    }

    // ── Phase 9 — SpawnBlock / pick_monster ──────────────────────────────────

    #[test]
    fn spawn_block_pick_monster_single_always_returns_it() {
        let block = SpawnBlock {
            position: pos(100, 100, 7),
            interval_secs: 60,
            candidates: vec![MonsterCandidate {
                monster_name: "Rat".into(),
                chance: 100,
            }],
        };
        assert_eq!(block.pick_monster(50).unwrap().monster_name, "Rat");
    }

    #[test]
    fn spawn_block_pick_monster_empty_returns_none() {
        let block = SpawnBlock {
            position: pos(100, 100, 7),
            interval_secs: 60,
            candidates: vec![],
        };
        assert!(block.pick_monster(50).is_none());
    }

    #[test]
    fn spawn_block_pick_monster_selects_by_cumulative_chance() {
        // Rat: 40%, Orc: 60%
        let block = SpawnBlock {
            position: pos(100, 100, 7),
            interval_secs: 60,
            candidates: vec![
                MonsterCandidate {
                    monster_name: "Rat".into(),
                    chance: 40,
                },
                MonsterCandidate {
                    monster_name: "Orc".into(),
                    chance: 60,
                },
            ],
        };
        assert_eq!(block.pick_monster(1).unwrap().monster_name, "Rat");
        assert_eq!(block.pick_monster(40).unwrap().monster_name, "Rat");
        assert_eq!(block.pick_monster(41).unwrap().monster_name, "Orc");
        assert_eq!(block.pick_monster(100).unwrap().monster_name, "Orc");
    }

    #[test]
    fn spawn_block_pick_monster_roll_over_total_falls_back_to_first() {
        // Chances sum to only 50 — roll of 99 overflows, fallback to first.
        let block = SpawnBlock {
            position: pos(100, 100, 7),
            interval_secs: 60,
            candidates: vec![
                MonsterCandidate {
                    monster_name: "Rat".into(),
                    chance: 25,
                },
                MonsterCandidate {
                    monster_name: "Orc".into(),
                    chance: 25,
                },
            ],
        };
        assert_eq!(block.pick_monster(99).unwrap().monster_name, "Rat");
    }

    // ── Phase 10 — alive_count ───────────────────────────────────────────────

    #[test]
    fn alive_count_after_first_tick_equals_total() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        sm.tick(Instant::now());
        assert_eq!(sm.alive_count(), 2);
    }

    #[test]
    fn alive_count_decreases_on_kill() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        let t0 = Instant::now();
        sm.tick(t0);
        sm.on_creature_killed(0, t0);
        assert_eq!(sm.alive_count(), 1);
    }

    #[test]
    fn alive_count_zero_before_first_tick() {
        let world = world_with_two_spawns();
        let mut sm = SpawnManager::new();
        sm.load_world(&world);
        // entries are ReadyToSpawn, not Alive
        assert_eq!(sm.alive_count(), 0);
    }
}
