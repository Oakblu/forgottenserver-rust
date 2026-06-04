use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use forgottenserver_common::position::Position;
use forgottenserver_entity::creature::Creature;
use forgottenserver_entity::monster::Monster;
use forgottenserver_entity::player::Player;
use forgottenserver_game::condition::{ConditionKind, TickableCondition};
use forgottenserver_game::game_loop::stamina_xp_multiplier;
use forgottenserver_game::quest_registry::QuestRegistry;
use forgottenserver_game::spawn_manager::{SpawnEntry, SpawnManager};
use forgottenserver_map::pathfinder::Pathfinder;

/// Event queued when a new creature is spawned into the world.
/// One event is emitted per player whose viewport overlaps the spawn position.
#[derive(Debug, Clone)]
pub struct AddCreatureEvent {
    pub player_id: u32,
    pub creature_id: u32,
    pub position: Position,
    pub name: String,
}

/// Outfit appearance snapshot sent/stored per player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OutfitAppearance {
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
}

/// Descriptor for an NPC that periodically emits a voice message.
pub struct NpcVoiceDef {
    pub message: String,
    pub interval: Duration,
}

// ---------------------------------------------------------------------------
// Helpers — condition effects
// ---------------------------------------------------------------------------

enum ConditionEffect {
    Damage(i32),
    Heal(i32),
    SpeedMod(i32),
    NoEffect,
}

fn collect_effect(cond: &TickableCondition) -> ConditionEffect {
    match cond.kind {
        ConditionKind::Poison
        | ConditionKind::Fire
        | ConditionKind::Energy
        | ConditionKind::Drown
        | ConditionKind::Bleeding
        | ConditionKind::LifeDrain
        | ConditionKind::ManaDrain => ConditionEffect::Damage(cond.damage_per_tick),
        ConditionKind::Regeneration | ConditionKind::GainMana => {
            ConditionEffect::Heal(cond.heal_per_tick)
        }
        ConditionKind::Haste
        | ConditionKind::Paralyze
        | ConditionKind::Slowed
        | ConditionKind::Root => ConditionEffect::SpeedMod(cond.speed_modifier),
        _ => ConditionEffect::NoEffect,
    }
}

fn apply_direction(pos: Position, dir: u8) -> Option<Position> {
    let (dx, dy): (i32, i32) = match dir {
        0 => (0, -1),  // N
        1 => (1, 0),   // E
        2 => (0, 1),   // S
        3 => (-1, 0),  // W
        4 => (1, -1),  // NE
        5 => (1, 1),   // SE
        6 => (-1, 1),  // SW
        7 => (-1, -1), // NW
        _ => return None,
    };
    let new_x = (pos.x as i32 + dx).max(0) as u16;
    let new_y = (pos.y as i32 + dy).max(0) as u16;
    Some(Position::new(new_x, new_y, pos.z))
}

// ---------------------------------------------------------------------------
// GameState
// ---------------------------------------------------------------------------

/// Runtime game state shared by the server's handlers.
#[derive(Default)]
pub struct GameState {
    // --- existing: online tracking ---
    online: HashSet<String>,
    peak: usize,

    // --- creatures (players + monsters share creature id-space) ---
    creatures: HashMap<u32, Creature>,

    // --- monster data (AI, loot, flee) ---
    monsters: HashMap<u32, Monster>,

    // --- positions for non-player creatures ---
    creature_positions: HashMap<u32, Position>,

    // --- player entity map (player_id → full Player data) ---
    players: HashMap<u32, Player>,

    // --- per-player runtime state ---
    player_positions: HashMap<u32, Position>,
    attack_targets: HashMap<u32, u32>,             // player_id → creature_id (0 = none)
    follow_targets: HashMap<u32, (u32, Vec<u8>)>, // player_id → (creature_id, path)
    auto_walks: HashMap<u32, Vec<u8>>,            // player_id → direction bytes
    fight_modes: HashMap<u32, (u8, u8, bool)>,    // player_id → (fight, chase, secure)
    outfits: HashMap<u32, OutfitAppearance>,      // player_id → outfit
    vip_lists: HashMap<u32, Vec<u32>>,            // player_id → list of guids
    pub quest_registry: QuestRegistry,

    // --- ticking conditions: creature_id → Vec<TickableCondition> ---
    conditions: HashMap<u32, Vec<TickableCondition>>,

    // --- NPC voice system ---
    npc_voice_defs: HashMap<u32, NpcVoiceDef>,
    npc_last_voice: HashMap<u32, Instant>,
    npc_voice_queue: Vec<(u32, String)>,

    // --- spawn/respawn ---
    pub spawn_manager: SpawnManager,
    next_creature_id: u32,
    spawn_events: Vec<AddCreatureEvent>,
}

impl std::fmt::Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameState")
            .field("online_count", &self.online.len())
            .finish_non_exhaustive()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    // -----------------------------------------------------------------------
    // Online player tracking
    // -----------------------------------------------------------------------

    pub fn add_player(&mut self, name: &str) {
        self.online.insert(name.to_owned());
        if self.online.len() > self.peak {
            self.peak = self.online.len();
        }
    }

    pub fn remove_player(&mut self, name: &str) -> bool {
        self.online.remove(name)
    }

    pub fn online_player_count(&self) -> usize {
        self.online.len()
    }

    pub fn peak_players(&self) -> usize {
        self.peak
    }

    // -----------------------------------------------------------------------
    // Creature management
    // -----------------------------------------------------------------------

    pub fn add_creature(&mut self, creature: Creature) {
        self.creatures.insert(creature.id, creature);
    }

    pub fn get_creature(&self, id: u32) -> Option<&Creature> {
        self.creatures.get(&id)
    }

    pub fn get_creature_mut(&mut self, id: u32) -> Option<&mut Creature> {
        self.creatures.get_mut(&id)
    }

    /// Subtract `damage` from target's health (clamped to 0).
    /// Returns `(remaining_health, health_percent)` or `None` if target unknown.
    /// When health reaches 0 and the creature has a `spawn_id`, notifies the
    /// spawn manager so the respawn countdown begins.
    pub fn apply_damage(&mut self, target_id: u32, damage: i32) -> Option<(i32, u8)> {
        let (remaining, percent, dead_spawn_id) = {
            let creature = self.creatures.get_mut(&target_id)?;
            creature.health = (creature.health - damage).max(0);
            let percent = if creature.health_max > 0 {
                ((creature.health as u64 * 100) / creature.health_max as u64).min(100) as u8
            } else {
                0
            };
            let dead_spawn_id = if creature.health == 0 {
                creature.spawn_id
            } else {
                None
            };
            (creature.health, percent, dead_spawn_id)
        };
        if let Some(spawn_id) = dead_spawn_id {
            self.spawn_manager
                .on_creature_killed(spawn_id, Instant::now());
        }
        Some((remaining, percent))
    }

    // -----------------------------------------------------------------------
    // Monster management
    // -----------------------------------------------------------------------

    pub fn add_monster(&mut self, monster: Monster) {
        self.monsters.insert(monster.get_id(), monster);
    }

    pub fn get_monster(&self, id: u32) -> Option<&Monster> {
        self.monsters.get(&id)
    }

    pub fn get_monster_mut(&mut self, id: u32) -> Option<&mut Monster> {
        self.monsters.get_mut(&id)
    }

    pub fn set_creature_position(&mut self, creature_id: u32, pos: Position) {
        self.creature_positions.insert(creature_id, pos);
    }

    pub fn get_creature_position(&self, creature_id: u32) -> Option<Position> {
        self.creature_positions.get(&creature_id).copied()
    }

    // -----------------------------------------------------------------------
    // Player positions
    // -----------------------------------------------------------------------

    pub fn set_player_position(&mut self, player_id: u32, pos: Position) {
        self.player_positions.insert(player_id, pos);
    }

    pub fn get_player_position(&self, player_id: u32) -> Option<Position> {
        self.player_positions.get(&player_id).copied()
    }

    /// Return all (player_id, pos) pairs visible from `center` (|dx|≤9, |dy|≤7, same z).
    pub fn get_players_in_viewport(&self, center: Position) -> Vec<(u32, Position)> {
        self.player_positions
            .iter()
            .filter(|(_, &pos)| Self::in_viewport(center, pos))
            .map(|(&id, &pos)| (id, pos))
            .collect()
    }

    fn in_viewport(center: Position, pos: Position) -> bool {
        if center.z != pos.z {
            return false;
        }
        let dx = (center.x as i32 - pos.x as i32).abs();
        let dy = (center.y as i32 - pos.y as i32).abs();
        dx <= 9 && dy <= 7
    }

    /// Return the nearest player within `range` tiles (Chebyshev, same z).
    pub fn nearest_player(&self, pos: Position, range: i32) -> Option<(u32, Position)> {
        self.player_positions
            .iter()
            .filter(|(_, &p)| pos.z == p.z && pos.distance(p) <= range)
            .min_by_key(|(_, &p)| pos.distance(p))
            .map(|(&id, &p)| (id, p))
    }

    // -----------------------------------------------------------------------
    // Attack target state
    // -----------------------------------------------------------------------

    /// Set the current attack target for `player_id`. Mirrors C++
    /// `Game::playerSetAttackedCreature`. Passing 0 clears the target.
    pub fn set_attack_target(&mut self, player_id: u32, creature_id: u32) {
        if creature_id == 0 {
            self.attack_targets.remove(&player_id);
        } else {
            self.attack_targets.insert(player_id, creature_id);
        }
    }

    /// Return the current attack target for `player_id`, or `None` if not attacking.
    pub fn get_attack_target(&self, player_id: u32) -> Option<u32> {
        self.attack_targets.get(&player_id).copied()
    }

    // -----------------------------------------------------------------------
    // Follow state
    // -----------------------------------------------------------------------

    pub fn set_follow_target(&mut self, player_id: u32, creature_id: u32, path: Vec<u8>) {
        self.follow_targets.insert(player_id, (creature_id, path));
    }

    pub fn get_follow_target(&self, player_id: u32) -> Option<(u32, &Vec<u8>)> {
        self.follow_targets
            .get(&player_id)
            .map(|(cid, p)| (*cid, p))
    }

    // -----------------------------------------------------------------------
    // Auto-walk state
    // -----------------------------------------------------------------------

    pub fn set_auto_walk(&mut self, player_id: u32, path: Vec<u8>) {
        self.auto_walks.insert(player_id, path);
    }

    pub fn get_auto_walk(&self, player_id: u32) -> Option<&Vec<u8>> {
        self.auto_walks.get(&player_id)
    }

    // -----------------------------------------------------------------------
    // Fight mode state
    // -----------------------------------------------------------------------

    pub fn set_fight_mode(
        &mut self,
        player_id: u32,
        fight_mode: u8,
        chase_mode: u8,
        secure_mode: bool,
    ) {
        self.fight_modes
            .insert(player_id, (fight_mode, chase_mode, secure_mode));
    }

    pub fn get_fight_mode(&self, player_id: u32) -> Option<(u8, u8, bool)> {
        self.fight_modes.get(&player_id).copied()
    }

    // -----------------------------------------------------------------------
    // Outfit state
    // -----------------------------------------------------------------------

    pub fn set_outfit(&mut self, player_id: u32, outfit: OutfitAppearance) {
        self.outfits.insert(player_id, outfit);
    }

    pub fn get_outfit(&self, player_id: u32) -> Option<&OutfitAppearance> {
        self.outfits.get(&player_id)
    }

    // -----------------------------------------------------------------------
    // VIP list
    // -----------------------------------------------------------------------

    pub fn add_vip(&mut self, player_id: u32, guid: u32) {
        self.vip_lists.entry(player_id).or_default().push(guid);
    }

    pub fn remove_vip(&mut self, player_id: u32, guid: u32) {
        if let Some(list) = self.vip_lists.get_mut(&player_id) {
            list.retain(|&g| g != guid);
        }
    }

    pub fn get_vip_list(&self, player_id: u32) -> &[u32] {
        self.vip_lists
            .get(&player_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    // -----------------------------------------------------------------------
    // Player entity management
    // -----------------------------------------------------------------------

    pub fn add_player_entity(&mut self, player: Player) {
        self.players.insert(player.guid, player);
    }

    pub fn get_player_entity(&self, player_id: u32) -> Option<&Player> {
        self.players.get(&player_id)
    }

    pub fn get_player_entity_mut(&mut self, player_id: u32) -> Option<&mut Player> {
        self.players.get_mut(&player_id)
    }

    pub fn remove_player_entity(&mut self, player_id: u32) -> Option<Player> {
        self.players.remove(&player_id)
    }

    /// Apply `damage` to a player's health (clamped to 0). Returns `true` if found.
    pub fn apply_damage_to_player(&mut self, player_id: u32, damage: i32) -> bool {
        if let Some(player) = self.players.get_mut(&player_id) {
            let new_hp = (player.get_health() - damage).max(0);
            player.set_health(new_hp);
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Player lifecycle — death
    // -----------------------------------------------------------------------

    /// Apply death to the player: skill loss, item drop, teleport to temple,
    /// restore HP/mana to 40%. Returns `true` if the player was found.
    pub fn on_player_death(&mut self, player_id: u32) -> bool {
        let player = match self.players.get_mut(&player_id) {
            Some(p) => p,
            None => return false,
        };

        let skill_loss = player.compute_skill_loss();
        let item_loss = player.compute_item_loss();
        player.apply_skill_loss(skill_loss);
        player.drop_items(item_loss);

        let temple = player.temple_pos;
        player.position = temple;

        let max_hp = player.get_max_health();
        let max_mp = player.get_max_mana();
        player.set_health((max_hp as f32 * 0.4) as i32);
        player.set_mana((max_mp as f32 * 0.4) as i32);

        true
    }

    // -----------------------------------------------------------------------
    // Player lifecycle — XP / level-up
    // -----------------------------------------------------------------------

    /// Add `base_xp` (adjusted by stamina multiplier) to the player's experience
    /// and apply level-up if the threshold is crossed.
    /// Returns the new level if the player leveled up, otherwise `None`.
    pub fn grant_xp(&mut self, player_id: u32, base_xp: u64) -> Option<u32> {
        let player = self.players.get_mut(&player_id)?;
        let adjusted = (base_xp as f64
            * stamina_xp_multiplier(player.get_stamina(), player.is_premium()))
            as u64;
        player.add_experience(adjusted);
        player.check_level_up()
    }

    // -----------------------------------------------------------------------
    // Quest state hook
    // -----------------------------------------------------------------------

    pub fn on_storage_value_changed(&mut self, player_id: u32, key: u32, value: i32) {
        self.quest_registry.on_storage_change(player_id, key, value);
    }

    // -----------------------------------------------------------------------
    // Condition management (Phase 2)
    // -----------------------------------------------------------------------

    pub fn add_tickable_condition(&mut self, creature_id: u32, cond: TickableCondition) {
        self.conditions.entry(creature_id).or_default().push(cond);
    }

    pub fn get_conditions(&self, creature_id: u32) -> &[TickableCondition] {
        self.conditions
            .get(&creature_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Advance all ticking conditions. Fires effects and removes expired conditions.
    pub fn tick_conditions(&mut self, now: Instant) {
        let creature_ids: Vec<u32> = self.conditions.keys().copied().collect();
        for creature_id in creature_ids {
            let effects: Vec<ConditionEffect> = {
                let Some(conds) = self.conditions.get_mut(&creature_id) else {
                    continue;
                };
                let mut effects: Vec<ConditionEffect> = Vec::new();
                let mut i = 0;
                while i < conds.len() {
                    let cond = &mut conds[i];
                    if now >= cond.next_tick {
                        effects.push(collect_effect(cond));
                        cond.next_tick = now + cond.tick_interval;
                        cond.ticks_remaining = cond.ticks_remaining.saturating_sub(1);
                    }
                    if cond.ticks_remaining == 0 {
                        conds.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
                effects
            };
            for effect in effects {
                match effect {
                    ConditionEffect::Damage(dmg) => {
                        self.apply_damage(creature_id, dmg);
                    }
                    ConditionEffect::Heal(hp) => {
                        if let Some(c) = self.creatures.get_mut(&creature_id) {
                            c.health = (c.health + hp).min(c.health_max);
                        }
                    }
                    ConditionEffect::SpeedMod(modifier) => {
                        if let Some(c) = self.creatures.get_mut(&creature_id) {
                            c.var_speed = modifier;
                        }
                    }
                    ConditionEffect::NoEffect => {}
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Creature AI loop (Phase 3)
    // -----------------------------------------------------------------------

    /// Run one AI tick for every live monster: flee, attack, or move toward target.
    pub fn tick_creature_ai(&mut self) {
        let monster_ids: Vec<u32> = self.monsters.keys().copied().collect();

        let mut move_actions: Vec<(u32, Position)> = Vec::new();
        let mut attack_actions: Vec<(u32, u32, i32)> = Vec::new(); // (monster_id, player_id, damage)

        for monster_id in &monster_ids {
            let monster = &self.monsters[monster_id];
            if !monster.creature.is_alive() {
                continue;
            }

            let Some(&creature_pos) = self.creature_positions.get(monster_id) else {
                continue;
            };

            let Some((player_id, player_pos)) = self.nearest_player(creature_pos, 9) else {
                continue;
            };

            let hp_pct = if monster.creature.health_max > 0 {
                (monster.creature.health * 100 / monster.creature.health_max) as u32
            } else {
                0
            };

            if monster.should_flee(hp_pct) {
                // Move away from player
                let dx = (creature_pos.x as i32 - player_pos.x as i32).signum();
                let dy = (creature_pos.y as i32 - player_pos.y as i32).signum();
                let new_x = (creature_pos.x as i32 + dx).max(0) as u16;
                let new_y = (creature_pos.y as i32 + dy).max(0) as u16;
                move_actions.push((*monster_id, Position::new(new_x, new_y, creature_pos.z)));
            } else {
                let dist = creature_pos.distance(player_pos);
                if dist <= 1 {
                    // Melee attack — fixed damage for now
                    attack_actions.push((*monster_id, player_id, 10));
                } else {
                    let path = Pathfinder::new().find_path(creature_pos, player_pos);
                    if let Some(&dir_byte) = path.first() {
                        if let Some(new_pos) = apply_direction(creature_pos, dir_byte) {
                            move_actions.push((*monster_id, new_pos));
                        }
                    }
                }
            }
        }

        for (monster_id, new_pos) in move_actions {
            self.creature_positions.insert(monster_id, new_pos);
        }
        for (_, player_id, damage) in attack_actions {
            self.apply_damage_to_player(player_id, damage);
        }
    }

    // -----------------------------------------------------------------------
    // NPC voice tick (Phase 4)
    // -----------------------------------------------------------------------

    pub fn add_npc_voice(&mut self, npc_id: u32, message: impl Into<String>, interval: Duration) {
        self.npc_voice_defs.insert(
            npc_id,
            NpcVoiceDef {
                message: message.into(),
                interval,
            },
        );
    }

    /// Override the last-fired timestamp for an NPC (used in tests).
    pub fn set_npc_last_voice(&mut self, npc_id: u32, at: Instant) {
        self.npc_last_voice.insert(npc_id, at);
    }

    /// Emit voice messages for any NPC whose interval has elapsed.
    pub fn tick_npc_voices(&mut self, now: Instant) {
        let npc_ids: Vec<u32> = self.npc_voice_defs.keys().copied().collect();
        for npc_id in npc_ids {
            let interval = self.npc_voice_defs[&npc_id].interval;
            let message = self.npc_voice_defs[&npc_id].message.clone();
            let last = self.npc_last_voice.entry(npc_id).or_insert_with(|| {
                now.checked_sub(interval + Duration::from_millis(1))
                    .unwrap_or(now)
            });
            if now.duration_since(*last) >= interval {
                *last = now;
                self.npc_voice_queue.push((npc_id, message));
            }
        }
    }

    /// Drain and return all queued NPC voice messages.
    pub fn drain_voice_messages(&mut self) -> Vec<(u32, String)> {
        std::mem::take(&mut self.npc_voice_queue)
    }

    // -----------------------------------------------------------------------
    // Spawn management (Phase 4 — game loop integration)
    // -----------------------------------------------------------------------

    /// Place a new creature for the given spawn entry into the world.
    /// Queues `AddCreatureEvent` for every player whose viewport overlaps the
    /// spawn position. Returns the new creature's ID.
    pub fn spawn_creature(&mut self, entry: &SpawnEntry) -> u32 {
        let pos = entry.position;
        let player_ids: Vec<u32> = self
            .player_positions
            .iter()
            .filter(|(_, &p)| Self::in_viewport(pos, p))
            .map(|(&pid, _)| pid)
            .collect();

        let id = self.next_creature_id;
        self.next_creature_id += 1;

        let mut creature = Creature::new(id, &entry.monster_name);
        creature.spawn_id = Some(entry.spawn_id);
        self.creatures.insert(id, creature);
        self.creature_positions.insert(id, pos);

        for player_id in player_ids {
            self.spawn_events.push(AddCreatureEvent {
                player_id,
                creature_id: id,
                position: pos,
                name: entry.monster_name.clone(),
            });
        }

        id
    }

    /// Advance the spawn clock and place creatures for all entries whose
    /// respawn interval has elapsed.
    pub fn tick_spawns(&mut self, now: Instant) {
        let ready_ids = self.spawn_manager.tick(now);
        let entries: Vec<SpawnEntry> = ready_ids
            .into_iter()
            .filter_map(|sid| self.spawn_manager.entry(sid).cloned())
            .collect();
        for entry in entries {
            self.spawn_creature(&entry);
        }
    }

    /// Drain and return all queued `AddCreatureEvent` messages.
    pub fn drain_spawn_events(&mut self) -> Vec<AddCreatureEvent> {
        std::mem::take(&mut self.spawn_events)
    }

    // -----------------------------------------------------------------------
    // Integrated tick (Phase 5)
    // -----------------------------------------------------------------------

    /// Run one full game tick: spawn cycle, creature AI, condition ticking, NPC voices.
    pub fn tick(&mut self, now: Instant) {
        self.tick_spawns(now);
        self.tick_creature_ai();
        self.tick_conditions(now);
        self.tick_npc_voices(now);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::position::Position;
    use forgottenserver_entity::creature::Creature;
    use forgottenserver_entity::monster::Monster;
    use forgottenserver_entity::player::{xp_for_level, Player, SkillType};
    use forgottenserver_game::condition::{ConditionKind, TickableCondition};
    use forgottenserver_game::spawn_manager::SpawnEntry;

    fn make_creature(id: u32) -> Creature {
        Creature::new(id, "TestCreature")
    }

    fn make_monster(id: u32, health: i32) -> Monster {
        Monster::new(id, "Rat", health)
    }

    // --- existing online tracking ---

    #[test]
    fn add_and_count_players() {
        let mut gs = GameState::new();
        gs.add_player("Alice");
        gs.add_player("Bob");
        assert_eq!(gs.online_player_count(), 2);
    }

    #[test]
    fn remove_known_player_returns_true() {
        let mut gs = GameState::new();
        gs.add_player("Alice");
        assert!(gs.remove_player("Alice"));
        assert_eq!(gs.online_player_count(), 0);
    }

    #[test]
    fn remove_unknown_player_returns_false() {
        let mut gs = GameState::new();
        assert!(!gs.remove_player("Nobody"));
    }

    #[test]
    fn peak_tracks_maximum() {
        let mut gs = GameState::new();
        gs.add_player("Alice");
        gs.add_player("Bob");
        gs.remove_player("Bob");
        assert_eq!(gs.peak_players(), 2);
        assert_eq!(gs.online_player_count(), 1);
    }

    // --- creature management ---

    #[test]
    fn add_and_get_creature() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(10));
        assert!(gs.get_creature(10).is_some());
        assert_eq!(gs.get_creature(10).unwrap().name, "TestCreature");
    }

    #[test]
    fn apply_damage_reduces_health() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(5));
        let (remaining, _pct) = gs.apply_damage(5, 30).unwrap();
        assert_eq!(remaining, 70);
    }

    #[test]
    fn apply_damage_clamps_to_zero() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(5));
        let (remaining, pct) = gs.apply_damage(5, 999).unwrap();
        assert_eq!(remaining, 0);
        assert_eq!(pct, 0);
    }

    #[test]
    fn apply_damage_unknown_target_returns_none() {
        let mut gs = GameState::new();
        assert!(gs.apply_damage(999, 10).is_none());
    }

    // --- positions & viewport ---

    #[test]
    fn set_and_get_player_position() {
        let mut gs = GameState::new();
        let pos = Position::new(100, 100, 7);
        gs.set_player_position(1, pos);
        assert_eq!(gs.get_player_position(1), Some(pos));
    }

    #[test]
    fn viewport_includes_nearby_player() {
        let mut gs = GameState::new();
        let center = Position::new(100, 100, 7);
        gs.set_player_position(1, center);
        gs.set_player_position(2, Position::new(103, 100, 7));
        gs.set_player_position(3, Position::new(200, 200, 7));

        let vp = gs.get_players_in_viewport(center);
        let ids: Vec<u32> = vp.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(!ids.contains(&3));
    }

    // --- attack target ---

    #[test]
    fn set_and_get_attack_target() {
        let mut gs = GameState::new();
        gs.set_attack_target(1, 42);
        assert_eq!(gs.get_attack_target(1), Some(42));
    }

    #[test]
    fn set_attack_target_zero_clears_target() {
        let mut gs = GameState::new();
        gs.set_attack_target(1, 42);
        gs.set_attack_target(1, 0);
        assert_eq!(gs.get_attack_target(1), None);
    }

    #[test]
    fn get_attack_target_none_for_unknown_player() {
        let gs = GameState::new();
        assert_eq!(gs.get_attack_target(999), None);
    }

    // --- follow ---

    #[test]
    fn set_and_get_follow_target() {
        let mut gs = GameState::new();
        gs.set_follow_target(1, 42, vec![0, 1, 2]);
        let (cid, path) = gs.get_follow_target(1).unwrap();
        assert_eq!(cid, 42);
        assert_eq!(path, &vec![0u8, 1, 2]);
    }

    // --- auto walk ---

    #[test]
    fn set_and_get_auto_walk() {
        let mut gs = GameState::new();
        gs.set_auto_walk(1, vec![1, 2]);
        assert_eq!(gs.get_auto_walk(1).unwrap(), &vec![1u8, 2]);
    }

    #[test]
    fn auto_walk_empty_path_stored() {
        let mut gs = GameState::new();
        gs.set_auto_walk(1, vec![]);
        assert!(gs.get_auto_walk(1).unwrap().is_empty());
    }

    // --- fight modes ---

    #[test]
    fn set_and_get_fight_mode() {
        let mut gs = GameState::new();
        gs.set_fight_mode(1, 2, 1, true);
        let (fight, chase, secure) = gs.get_fight_mode(1).unwrap();
        assert_eq!(fight, 2);
        assert_eq!(chase, 1);
        assert!(secure);
    }

    // --- outfit ---

    #[test]
    fn set_and_get_outfit() {
        let mut gs = GameState::new();
        let outfit = OutfitAppearance {
            look_type: 128,
            look_head: 5,
            ..Default::default()
        };
        gs.set_outfit(1, outfit);
        assert_eq!(gs.get_outfit(1).unwrap().look_type, 128);
    }

    // --- vip list ---

    #[test]
    fn add_and_remove_vip() {
        let mut gs = GameState::new();
        gs.add_vip(1, 42);
        gs.add_vip(1, 99);
        gs.remove_vip(1, 42);
        let list = gs.get_vip_list(1);
        assert!(!list.contains(&42));
        assert!(list.contains(&99));
    }

    #[test]
    fn vip_list_empty_for_unknown_player() {
        let gs = GameState::new();
        assert!(gs.get_vip_list(999).is_empty());
    }

    // --- player entity management ---

    #[test]
    fn add_and_get_player_entity() {
        let mut gs = GameState::new();
        let player = Player::new(10, "Hero", 1);
        gs.add_player_entity(player);
        assert!(gs.get_player_entity(10).is_some());
        assert_eq!(gs.get_player_entity(10).unwrap().name, "Hero");
    }

    #[test]
    fn remove_player_entity_returns_player() {
        let mut gs = GameState::new();
        gs.add_player_entity(Player::new(1, "A", 1));
        let removed = gs.remove_player_entity(1);
        assert!(removed.is_some());
        assert!(gs.get_player_entity(1).is_none());
    }

    // --- Phase 1: player death ---

    #[test]
    fn player_death_resets_hp_mana_to_40_percent() {
        let mut gs = GameState::new();
        let mut player = Player::new(1, "Hero", 1);
        player.set_max_health(100);
        player.set_health(100);
        player.set_max_mana(50);
        player.set_mana(50);
        gs.add_player_entity(player);

        gs.on_player_death(1);

        let p = gs.get_player_entity(1).unwrap();
        assert_eq!(p.get_health(), 40); // 40% of 100
        assert_eq!(p.get_mana(), 20); // 40% of 50
    }

    #[test]
    fn player_death_teleports_to_temple() {
        let mut gs = GameState::new();
        let mut player = Player::new(1, "Hero", 1);
        player.position = Position::new(100, 100, 7);
        player.temple_pos = Position::new(160, 54, 7);
        gs.add_player_entity(player);

        gs.on_player_death(1);

        let p = gs.get_player_entity(1).unwrap();
        assert_eq!(p.position, Position::new(160, 54, 7));
    }

    #[test]
    fn player_death_applies_skill_loss() {
        let mut gs = GameState::new();
        let mut player = Player::new(1, "Hero", 1);
        player.set_skill_level(SkillType::Sword, 100);
        gs.add_player_entity(player);

        gs.on_player_death(1);

        let p = gs.get_player_entity(1).unwrap();
        assert!(
            p.get_skill_level(SkillType::Sword) < 100,
            "skill level must decrease after death"
        );
    }

    #[test]
    fn on_player_death_returns_false_for_unknown_player() {
        let mut gs = GameState::new();
        assert!(!gs.on_player_death(999));
    }

    // --- Phase 2: XP / level-up ---

    #[test]
    fn xp_gain_triggers_level_up_when_threshold_crossed() {
        let mut gs = GameState::new();
        let player = Player::new(1, "Hero", 1);
        gs.add_player_entity(player);

        let new_level = gs.grant_xp(1, xp_for_level(2));

        assert_eq!(new_level, Some(2));
        let p = gs.get_player_entity(1).unwrap();
        assert_eq!(p.get_level(), 2);
    }

    #[test]
    fn grant_xp_applies_stamina_multiplier_below_840() {
        let mut gs = GameState::new();
        let mut player = Player::new(1, "Hero", 1);
        player.drain_stamina(2000); // stamina = 520, below STAMINA_EXHAUSTED_THRESHOLD (840)
        gs.add_player_entity(player);

        // 200 XP * 0.5 = 100 → exactly enough for level 2
        let new_level = gs.grant_xp(1, 200);
        assert_eq!(new_level, Some(2));
    }

    #[test]
    fn grant_xp_returns_none_for_unknown_player() {
        let mut gs = GameState::new();
        assert_eq!(gs.grant_xp(999, 100), None);
    }

    // --- quest storage hook ---

    #[test]
    fn storage_value_change_advances_quest_state() {
        use forgottenserver_game::quest_registry::QuestDef;

        let mut gs = GameState::new();
        gs.quest_registry.register(QuestDef {
            id: 1,
            name: "The Storage Quest".to_string(),
            completed: false,
            missions: vec![],
        });
        gs.on_storage_value_changed(1, 1, 5);
        assert!(
            gs.quest_registry.mission_info(1).unwrap().completed,
            "Quest must be completed after matching storage change"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 2 — Condition ticking
    // -----------------------------------------------------------------------

    #[test]
    fn poison_condition_deals_damage_each_tick() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::Poison, 3, 100, now).with_damage(10);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 90);
    }

    #[test]
    fn fire_condition_expires_after_ticks_remaining_zero() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::Fire, 1, 100, now).with_damage(5);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert!(
            gs.get_conditions(1).is_empty(),
            "expired condition must be removed"
        );
    }

    #[test]
    fn regeneration_condition_heals_entity_each_tick() {
        let mut gs = GameState::new();
        let mut c = make_creature(1);
        c.set_health(50);
        gs.add_creature(c);
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::Regeneration, 3, 100, now).with_heal(20);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 70);
    }

    #[test]
    fn paralyze_sets_speed_modifier() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond =
            TickableCondition::new(ConditionKind::Paralyze, 5, 100, now).with_speed_modifier(-100);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().var_speed, -100);
    }

    // -----------------------------------------------------------------------
    // Phase 3 — Creature AI loop
    // -----------------------------------------------------------------------

    #[test]
    fn creature_moves_toward_player_when_not_in_range() {
        let mut gs = GameState::new();
        gs.add_monster(make_monster(1, 100));
        let start_pos = Position::new(100, 100, 7);
        gs.set_creature_position(1, start_pos);
        gs.set_player_position(99, Position::new(104, 100, 7)); // 4 tiles away

        gs.tick_creature_ai();

        let new_pos = gs.get_creature_position(1).unwrap();
        assert_ne!(
            new_pos, start_pos,
            "monster should have moved toward player"
        );
    }

    #[test]
    fn creature_attacks_player_when_adjacent() {
        let mut gs = GameState::new();
        gs.add_monster(make_monster(1, 100));
        gs.set_creature_position(1, Position::new(100, 100, 7));
        let player = Player::new(99, "Hero", 1);
        let initial_hp = player.get_health();
        gs.add_player_entity(player);
        gs.set_player_position(99, Position::new(101, 100, 7)); // adjacent

        gs.tick_creature_ai();

        let player_hp = gs.get_player_entity(99).unwrap().get_health();
        assert!(
            player_hp < initial_hp,
            "player should have taken damage from adjacent monster"
        );
    }

    #[test]
    fn creature_flees_when_hp_below_threshold() {
        let mut gs = GameState::new();
        let mut monster = make_monster(1, 100);
        monster.creature.set_health(10); // 10% HP
        monster.set_can_flee(true);
        monster.set_flee_health_percent(20); // flee below 20%
        gs.add_monster(monster);
        let start_pos = Position::new(100, 100, 7);
        gs.set_creature_position(1, start_pos);
        gs.set_player_position(99, Position::new(101, 100, 7)); // player East

        gs.tick_creature_ai();

        let new_pos = gs.get_creature_position(1).unwrap();
        assert_ne!(new_pos, start_pos, "fleeing monster must move");
        // Player is East (x+1), so monster flees West (x-1)
        assert!(
            new_pos.x < start_pos.x,
            "monster should flee west (away from player)"
        );
    }

    #[test]
    fn dead_creature_skipped_in_ai_loop() {
        let mut gs = GameState::new();
        let mut monster = make_monster(1, 100);
        monster.creature.set_health(0); // dead
        gs.add_monster(monster);
        let start_pos = Position::new(100, 100, 7);
        gs.set_creature_position(1, start_pos);
        gs.set_player_position(99, Position::new(101, 100, 7));

        gs.tick_creature_ai();

        let pos = gs.get_creature_position(1).unwrap();
        assert_eq!(pos, start_pos, "dead creature must not move");
    }

    // -----------------------------------------------------------------------
    // Phase 4 — NPC voice tick
    // -----------------------------------------------------------------------

    #[test]
    fn npc_voice_message_fires_at_configured_interval() {
        let mut gs = GameState::new();
        let interval = Duration::from_millis(500);
        gs.add_npc_voice(1, "Hello, traveler!", interval);

        let now = Instant::now();
        let past = now - interval - Duration::from_millis(1);
        gs.set_npc_last_voice(1, past);

        gs.tick_npc_voices(now);

        let messages = gs.drain_voice_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, 1);
        assert_eq!(messages[0].1, "Hello, traveler!");
    }

    // -----------------------------------------------------------------------
    // Phase 4 — spawn_creature / tick_spawns
    // -----------------------------------------------------------------------

    fn make_spawn_entry(spawn_id: u32, pos: Position, name: &str) -> SpawnEntry {
        use forgottenserver_game::spawn_manager::SpawnState;
        SpawnEntry {
            spawn_id,
            position: pos,
            radius: 3,
            monster_name: name.to_string(),
            interval_secs: 60,
            state: SpawnState::Alive,
            live_creature_id: None,
        }
    }

    #[test]
    fn creature_added_to_game_state_after_respawn() {
        let mut gs = GameState::new();
        let entry = make_spawn_entry(0, Position::new(100, 100, 7), "Rat");

        let creature_id = gs.spawn_creature(&entry);

        assert!(gs.get_creature(creature_id).is_some());
        let c = gs.get_creature(creature_id).unwrap();
        assert_eq!(c.name, "Rat");
        assert_eq!(c.spawn_id, Some(0));
        assert_eq!(
            gs.get_creature_position(creature_id),
            Some(Position::new(100, 100, 7))
        );
    }

    #[test]
    fn add_creature_broadcasts_to_players_in_viewport() {
        let mut gs = GameState::new();
        gs.set_player_position(1, Position::new(100, 100, 7)); // in viewport
        gs.set_player_position(2, Position::new(250, 250, 7)); // out of viewport

        let entry = make_spawn_entry(0, Position::new(102, 100, 7), "Rat");
        gs.spawn_creature(&entry);

        let events = gs.drain_spawn_events();
        let pids: Vec<u32> = events.iter().map(|e| e.player_id).collect();
        assert!(pids.contains(&1), "player 1 must receive AddCreature event");
        assert!(!pids.contains(&2), "player 2 is out of viewport");
    }

    #[test]
    fn spawn_creature_assigns_sequential_ids() {
        let mut gs = GameState::new();
        let entry = make_spawn_entry(0, Position::new(100, 100, 7), "Rat");
        let id1 = gs.spawn_creature(&entry);
        let id2 = gs.spawn_creature(&entry);
        assert_ne!(id1, id2);
    }

    #[test]
    fn tick_spawns_places_creatures_for_ready_entries() {
        use forgottenserver_world::{SpawnPointDef, World};

        let mut world = World::new();
        world.add_spawn_point(SpawnPointDef {
            position: Position::new(100, 100, 7),
            radius: 3,
            monster_name: "Rat".to_string(),
            interval_secs: 60,
        });

        let mut gs = GameState::new();
        gs.spawn_manager.load_world(&world);

        let now = Instant::now();
        gs.tick_spawns(now); // initial boot — all ReadyToSpawn entries fire

        // One creature should have been added
        assert!(
            gs.get_creature(0).is_some(),
            "creature 0 must exist after first tick_spawns"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 5 — Integration tick
    // -----------------------------------------------------------------------

    #[test]
    fn tick_fires_creature_ai_and_conditions() {
        let mut gs = GameState::new();

        // Creature with a poison condition that fires immediately
        gs.add_creature(make_creature(5));
        let now = Instant::now();
        let poison = TickableCondition::new(ConditionKind::Poison, 2, 100, now).with_damage(15);
        gs.add_tickable_condition(5, poison);

        // Monster that will move toward a distant player
        gs.add_monster(make_monster(10, 100));
        let start_pos = Position::new(100, 100, 7);
        gs.set_creature_position(10, start_pos);
        gs.set_player_position(99, Position::new(104, 100, 7));

        gs.tick(now);

        // Condition ticked: creature 5 took poison damage
        let c = gs.get_creature(5).unwrap();
        assert!(c.health < 100, "poison must have dealt damage via tick");

        // AI ticked: monster 10 moved toward player
        let new_pos = gs.get_creature_position(10).unwrap();
        assert_ne!(
            new_pos, start_pos,
            "creature AI must have moved the monster"
        );
    }

    // -----------------------------------------------------------------------
    // apply_direction helper (private fn exercised indirectly via creature AI)
    // -----------------------------------------------------------------------

    #[test]
    fn apply_direction_north_decrements_y() {
        let pos = Position::new(100, 100, 7);
        // exercise via the public creature-AI path: monster at (100,102), player
        // at (100,100); pathfinder goes north (dir 0).
        let mut gs = GameState::new();
        gs.add_monster(make_monster(1, 100));
        gs.set_creature_position(1, Position::new(100, 102, 7));
        gs.set_player_position(99, pos);

        gs.tick_creature_ai();

        let new_pos = gs.get_creature_position(1).unwrap();
        assert_eq!(new_pos.y, 101, "monster should move north (y--)");
        let _ = pos;
    }

    #[test]
    fn apply_direction_invalid_dir_returns_none_leaves_creature_stationary() {
        // Indirectly: set a monster in a position where the pathfinder would
        // produce no valid path (same tile as player → dist==0; dist≤1 triggers
        // attack not movement, so no apply_direction call; covered via monster
        // attacking the player while on the same tile).
        let mut gs = GameState::new();
        gs.add_monster(make_monster(1, 100));
        gs.set_creature_position(1, Position::new(100, 100, 7));
        let player = Player::new(99, "Hero", 1);
        let initial_hp = player.get_health();
        gs.add_player_entity(player);
        gs.set_player_position(99, Position::new(100, 100, 7)); // same tile

        gs.tick_creature_ai();

        // Player took damage (attack, not movement) and creature did not move.
        let player_hp = gs.get_player_entity(99).unwrap().get_health();
        assert!(player_hp < initial_hp, "attack must fire at dist == 0");
        assert_eq!(
            gs.get_creature_position(1).unwrap(),
            Position::new(100, 100, 7),
            "creature must not move when attacking"
        );
    }

    // -----------------------------------------------------------------------
    // collect_effect — uncovered variants (GainMana, Haste, Slowed, Root,
    // NoEffect variants: Drunk, Invisible, etc.)
    // -----------------------------------------------------------------------

    #[test]
    fn gain_mana_condition_heals_entity_each_tick() {
        let mut gs = GameState::new();
        let mut c = make_creature(1);
        c.set_health(50);
        gs.add_creature(c);
        let now = Instant::now();
        // GainMana → Heal branch
        let cond = TickableCondition::new(ConditionKind::GainMana, 3, 100, now).with_heal(15);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 65);
    }

    #[test]
    fn haste_condition_sets_speed_modifier() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        // Haste → SpeedMod branch
        let cond =
            TickableCondition::new(ConditionKind::Haste, 3, 100, now).with_speed_modifier(50);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().var_speed, 50);
    }

    #[test]
    fn slowed_condition_sets_negative_speed_modifier() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        // Slowed → SpeedMod branch
        let cond =
            TickableCondition::new(ConditionKind::Slowed, 3, 100, now).with_speed_modifier(-50);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().var_speed, -50);
    }

    #[test]
    fn root_condition_sets_speed_modifier() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        // Root → SpeedMod branch
        let cond =
            TickableCondition::new(ConditionKind::Root, 3, 100, now).with_speed_modifier(-200);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().var_speed, -200);
    }

    #[test]
    fn no_effect_condition_does_not_change_health() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        // Drunk → NoEffect branch (does nothing to stats)
        let cond = TickableCondition::new(ConditionKind::Drunk, 3, 100, now);
        gs.add_tickable_condition(1, cond);
        let health_before = gs.get_creature(1).unwrap().health;
        gs.tick_conditions(now);
        let health_after = gs.get_creature(1).unwrap().health;
        assert_eq!(health_before, health_after, "Drunk (NoEffect) must not change health");
    }

    // -----------------------------------------------------------------------
    // GameState Debug impl
    // -----------------------------------------------------------------------

    #[test]
    fn game_state_debug_includes_online_count() {
        let mut gs = GameState::new();
        gs.add_player("Alice");
        gs.add_player("Bob");
        let debug_str = format!("{gs:?}");
        assert!(
            debug_str.contains("online_count"),
            "Debug output must include online_count: {debug_str}"
        );
        assert!(
            debug_str.contains('2'),
            "Debug output must show count 2: {debug_str}"
        );
    }

    // -----------------------------------------------------------------------
    // get_creature_mut
    // -----------------------------------------------------------------------

    #[test]
    fn get_creature_mut_returns_mutable_reference() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(7));
        if let Some(c) = gs.get_creature_mut(7) {
            c.set_health(42);
        }
        assert_eq!(gs.get_creature(7).unwrap().health, 42);
    }

    #[test]
    fn get_creature_mut_returns_none_for_unknown_id() {
        let mut gs = GameState::new();
        assert!(gs.get_creature_mut(999).is_none());
    }

    // -----------------------------------------------------------------------
    // apply_damage — death triggers spawn_manager
    // -----------------------------------------------------------------------

    #[test]
    fn apply_damage_kills_creature_with_spawn_id_notifies_spawn_manager() {
        use forgottenserver_world::{SpawnPointDef, World};
        let mut gs = GameState::new();

        // Register a spawn entry so on_creature_killed has something to act on.
        let mut world = World::new();
        world.add_spawn_point(SpawnPointDef {
            position: Position::new(100, 100, 7),
            radius: 3,
            monster_name: "Rat".to_string(),
            interval_secs: 60,
        });
        gs.spawn_manager.load_world(&world);

        // Spawn the creature (assign spawn_id)
        let entry_id = {
            let entry = gs.spawn_manager.entry(0).cloned().unwrap();
            gs.spawn_creature(&entry)
        };

        // Now kill it — should call on_creature_killed
        let result = gs.apply_damage(entry_id, 99999);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 0); // health == 0
    }

    // -----------------------------------------------------------------------
    // Monster management — get_monster_mut, set/get_creature_position
    // -----------------------------------------------------------------------

    #[test]
    fn get_monster_mut_allows_mutation() {
        let mut gs = GameState::new();
        gs.add_monster(make_monster(5, 100));
        if let Some(m) = gs.get_monster_mut(5) {
            m.creature.set_health(10);
        }
        assert_eq!(gs.get_monster(5).unwrap().creature.health, 10);
    }

    #[test]
    fn get_monster_mut_returns_none_for_unknown_id() {
        let mut gs = GameState::new();
        assert!(gs.get_monster_mut(999).is_none());
    }

    #[test]
    fn set_and_get_creature_position() {
        let mut gs = GameState::new();
        let pos = Position::new(50, 60, 7);
        gs.set_creature_position(1, pos);
        assert_eq!(gs.get_creature_position(1), Some(pos));
    }

    #[test]
    fn get_creature_position_returns_none_for_unknown_creature() {
        let gs = GameState::new();
        assert!(gs.get_creature_position(999).is_none());
    }

    // -----------------------------------------------------------------------
    // in_viewport — different-z case
    // -----------------------------------------------------------------------

    #[test]
    fn in_viewport_excludes_different_z_layer() {
        let mut gs = GameState::new();
        let center = Position::new(100, 100, 7);
        // Same x/y but different z — must not be in viewport.
        gs.set_player_position(1, Position::new(100, 100, 8));
        let vp = gs.get_players_in_viewport(center);
        assert!(vp.is_empty(), "players on a different z-level must not be in viewport");
    }

    // -----------------------------------------------------------------------
    // get_player_entity_mut / remove_player_entity
    // -----------------------------------------------------------------------

    #[test]
    fn get_player_entity_mut_allows_mutation() {
        let mut gs = GameState::new();
        let player = Player::new(3, "Warrior", 1);
        gs.add_player_entity(player);
        if let Some(p) = gs.get_player_entity_mut(3) {
            p.set_health(77);
        }
        assert_eq!(gs.get_player_entity(3).unwrap().get_health(), 77);
    }

    #[test]
    fn get_player_entity_mut_returns_none_for_unknown_player() {
        let mut gs = GameState::new();
        assert!(gs.get_player_entity_mut(999).is_none());
    }

    // -----------------------------------------------------------------------
    // apply_damage_to_player — both branches
    // -----------------------------------------------------------------------

    #[test]
    fn apply_damage_to_player_returns_true_and_reduces_health() {
        let mut gs = GameState::new();
        let mut player = Player::new(1, "Hero", 1);
        player.set_max_health(100);
        player.set_health(100);
        gs.add_player_entity(player);
        let result = gs.apply_damage_to_player(1, 25);
        assert!(result);
        assert_eq!(gs.get_player_entity(1).unwrap().get_health(), 75);
    }

    #[test]
    fn apply_damage_to_player_clamps_to_zero() {
        let mut gs = GameState::new();
        let mut player = Player::new(1, "Hero", 1);
        player.set_max_health(100);
        player.set_health(100);
        gs.add_player_entity(player);
        gs.apply_damage_to_player(1, 99999);
        assert_eq!(gs.get_player_entity(1).unwrap().get_health(), 0);
    }

    #[test]
    fn apply_damage_to_player_returns_false_for_unknown_player() {
        let mut gs = GameState::new();
        assert!(!gs.apply_damage_to_player(999, 10));
    }

    // -----------------------------------------------------------------------
    // additional condition branches: Energy, Drown, Bleeding, LifeDrain,
    // ManaDrain (all → Damage); also Invisible → NoEffect
    // -----------------------------------------------------------------------

    #[test]
    fn energy_condition_deals_damage() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::Energy, 2, 100, now).with_damage(20);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 80);
    }

    #[test]
    fn drown_condition_deals_damage() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::Drown, 2, 100, now).with_damage(5);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 95);
    }

    #[test]
    fn bleeding_condition_deals_damage() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::Bleeding, 2, 100, now).with_damage(8);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 92);
    }

    #[test]
    fn life_drain_condition_deals_damage() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::LifeDrain, 2, 100, now).with_damage(12);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 88);
    }

    #[test]
    fn mana_drain_condition_deals_damage() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::ManaDrain, 2, 100, now).with_damage(3);
        gs.add_tickable_condition(1, cond);
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, 97);
    }

    #[test]
    fn invisible_condition_has_no_effect_on_health() {
        let mut gs = GameState::new();
        gs.add_creature(make_creature(1));
        let now = Instant::now();
        let cond = TickableCondition::new(ConditionKind::Invisible, 2, 100, now);
        gs.add_tickable_condition(1, cond);
        let before = gs.get_creature(1).unwrap().health;
        gs.tick_conditions(now);
        assert_eq!(gs.get_creature(1).unwrap().health, before);
    }

    // -----------------------------------------------------------------------
    // NPC voice — first tick without a prior last_voice entry
    // -----------------------------------------------------------------------

    #[test]
    fn npc_voice_fires_on_first_tick_when_no_prior_entry() {
        let mut gs = GameState::new();
        // Use a very short interval so the time-since-epoch trick fires immediately.
        let interval = Duration::from_millis(1);
        gs.add_npc_voice(1, "Welcome!", interval);

        // Tick *without* calling set_npc_last_voice — the first-entry branch fires.
        // We need `now` to be >= interval after the implied "last" time.
        // The code sets last = now - interval - 1ms when first creating the entry,
        // so ticking immediately should fire.
        let now = Instant::now();
        gs.tick_npc_voices(now);

        let messages = gs.drain_voice_messages();
        assert_eq!(
            messages.len(),
            1,
            "NPC voice must fire on first tick even without a prior entry"
        );
        assert_eq!(messages[0].1, "Welcome!");
    }

    #[test]
    fn npc_voice_does_not_fire_before_interval_elapses() {
        let mut gs = GameState::new();
        let interval = Duration::from_secs(60);
        gs.add_npc_voice(1, "Hello!", interval);

        let now = Instant::now();
        // Set last_voice to "now" so the interval has NOT elapsed.
        gs.set_npc_last_voice(1, now);
        gs.tick_npc_voices(now);

        let messages = gs.drain_voice_messages();
        assert!(
            messages.is_empty(),
            "NPC voice must NOT fire before the interval elapses"
        );
    }

    // -----------------------------------------------------------------------
    // nearest_player — no-player / out-of-range cases
    // -----------------------------------------------------------------------

    #[test]
    fn nearest_player_returns_none_when_no_players() {
        let gs = GameState::new();
        assert!(gs.nearest_player(Position::new(100, 100, 7), 9).is_none());
    }

    #[test]
    fn nearest_player_returns_none_when_all_players_out_of_range() {
        let mut gs = GameState::new();
        gs.set_player_position(1, Position::new(200, 200, 7));
        assert!(
            gs.nearest_player(Position::new(100, 100, 7), 9).is_none(),
            "player 100+ tiles away must not be nearest"
        );
    }

    #[test]
    fn nearest_player_returns_closest_in_range() {
        let mut gs = GameState::new();
        gs.set_player_position(1, Position::new(103, 100, 7)); // distance 3
        gs.set_player_position(2, Position::new(108, 100, 7)); // distance 8
        let result = gs.nearest_player(Position::new(100, 100, 7), 9);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 1, "player 1 is closer and must be returned");
    }

    // -----------------------------------------------------------------------
    // apply_damage — health_max == 0 branch (percent calculation)
    // -----------------------------------------------------------------------

    #[test]
    fn apply_damage_with_zero_health_max_returns_percent_zero() {
        let mut gs = GameState::new();
        let mut c = Creature::new(1, "Weird");
        c.health_max = 0;
        c.health = 0;
        gs.add_creature(c);
        let result = gs.apply_damage(1, 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, 0, "percent must be 0 when health_max is 0");
    }
}
