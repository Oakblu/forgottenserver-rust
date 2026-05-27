use forgottenserver_common::position::{Direction, Position};
use forgottenserver_items::container::{Container, ReturnValue, INDEX_WHEREEVER};
use forgottenserver_items::item::Item;
use forgottenserver_items::items_registry::ItemTypeData;
use std::collections::HashMap;
use std::sync::Arc;

/// Game server state transitions.
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    Startup,
    Init,
    Normal,
    Closed,
    Closing,
    NoLogIn,
    Maintain,
}

/// Server-wide statistics snapshot.
#[derive(Debug, Clone, Default)]
pub struct ServerStats {
    pub players_online: u32,
    pub players_peak: u32,
    pub uptime_seconds: u64,
    pub monsters_alive: u32,
}

/// Message class for broadcast messages (mirrors C++ MessageClasses).
#[derive(Debug, Clone, PartialEq)]
pub enum MessageClass {
    StatusDefault,
    StatusWarning,
    EventAdvance,
    StatusSmall,
    InfoDescr,
    EventDefault,
    Loot,
}

/// A broadcast message delivered to all online players.
#[derive(Debug, Clone)]
pub struct BroadcastMessage {
    pub text: String,
    pub class: MessageClass,
}

// ── Map / spawn dispatch constants ───────────────────────────────────────────

/// Default Chebyshev spectator range for map dispatch.  Matches C++
/// `Map::maxClientViewportX` = 9 used in `Game::placeCreature` /
/// `Game::removeCreature` spectator gather calls.
pub const MAP_SPECTATOR_RANGE: u32 = 9;

// ── Speak class ───────────────────────────────────────────────────────────────

/// Speech class used in `creature_say`.  Mirrors the relevant subset of C++
/// `SpeakClasses` that affects the range calculation in `internalCreatureSay`.
#[derive(Debug, Clone, PartialEq)]
pub enum SpeakClass {
    /// Normal speech — standard viewport range.
    Say,
    /// Yell — double viewport + 2 (mirrors C++ TALKTYPE_YELL / TALKTYPE_MONSTER_YELL).
    Yell,
    /// Monster speech — standard range.
    MonsterSay,
}

// ── Result types ──────────────────────────────────────────────────────────────

/// Result of placing a creature on the map.
#[derive(Debug, Clone)]
pub struct PlaceResult {
    /// Assigned creature ID.
    pub creature_id: u32,
    /// IDs of spectators that would receive `onCreatureAppear`.
    pub spectators: Vec<u32>,
}

/// Result of removing a creature from the map.
#[derive(Debug, Clone)]
pub struct RemoveResult {
    /// Whether the creature existed and was removed.
    pub removed: bool,
    /// IDs of spectators that would receive `onRemoveCreature`.
    pub spectators: Vec<u32>,
}

/// Result of a `creature_say` dispatch.
#[derive(Debug, Clone)]
pub struct CreatureSayResult {
    pub creature_id: u32,
    pub text: String,
    pub speak_class: SpeakClass,
    /// IDs of spectators that would receive `sendCreatureSay`.
    pub spectators: Vec<u32>,
}

// ── Creature kind ─────────────────────────────────────────────────────────────

/// Lightweight record of a creature in the game world (ID + position + type tag).
#[derive(Debug, Clone, PartialEq)]
pub enum CreatureKind {
    Player,
    Monster,
    Npc,
}

#[derive(Debug, Clone)]
pub struct GameCreature {
    pub id: u32,
    pub name: String,
    pub position: Position,
    pub kind: CreatureKind,
    pub health: i32,
    pub max_health: i32,
    /// Account ID (only meaningful for Player kind; 0 means unset).
    pub account_id: u32,
    /// Current facing direction (mirrors C++ `Creature::direction`).
    pub direction: Direction,
    /// Walk tick counter — incremented each time `onWalk` is triggered.
    pub walk_ticks: u32,
}

impl GameCreature {
    pub fn new(
        id: u32,
        name: impl Into<String>,
        pos: Position,
        kind: CreatureKind,
        max_health: i32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            position: pos,
            kind,
            health: max_health,
            max_health,
            account_id: 0,
            direction: Direction::South,
            walk_ticks: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn apply_damage(&mut self, dmg: i32) {
        self.health = (self.health - dmg).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }
}

/// Central game orchestrator — manages world state, creatures, and game-loop transitions.
pub struct Game {
    pub state: GameState,
    creatures: HashMap<u32, GameCreature>,
    next_creature_id: u32,
    stats: ServerStats,
    experience_rate: u32,
    loot_rate: u32,
    spawn_rate: u32,
    // --- Player lifecycle registries ---
    player_names: HashMap<String, u32>,
    player_accounts: HashMap<u32, u32>,
    players_record: u32,
    broadcast_log: Vec<BroadcastMessage>,
    kicked_players: Vec<u32>,
    creature_check_list: Vec<u32>,
    creature_check_active: HashMap<u32, bool>,
    /// Currently loaded map path.
    map_path: Option<String>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::Startup,
            creatures: HashMap::new(),
            next_creature_id: 1,
            stats: ServerStats::default(),
            experience_rate: 1,
            loot_rate: 1,
            spawn_rate: 1,
            player_names: HashMap::new(),
            player_accounts: HashMap::new(),
            players_record: 0,
            broadcast_log: Vec::new(),
            kicked_players: Vec::new(),
            creature_check_list: Vec::new(),
            creature_check_active: HashMap::new(),
            map_path: None,
        }
    }

    // --- State machine ---

    pub fn set_game_state(&mut self, new_state: GameState) {
        self.state = new_state;
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, GameState::Normal)
    }

    pub fn is_shutdown(&self) -> bool {
        matches!(self.state, GameState::Closing | GameState::Closed)
    }

    // --- Creature management ---

    pub fn add_creature(
        &mut self,
        name: impl Into<String>,
        pos: Position,
        kind: CreatureKind,
        max_health: i32,
    ) -> u32 {
        let id = self.next_creature_id;
        self.next_creature_id += 1;
        let creature = GameCreature::new(id, name, pos, kind.clone(), max_health);
        self.creatures.insert(id, creature);
        if kind == CreatureKind::Player {
            self.stats.players_online += 1;
            if self.stats.players_online > self.stats.players_peak {
                self.stats.players_peak = self.stats.players_online;
            }
        } else {
            self.stats.monsters_alive += 1;
        }
        id
    }

    pub fn remove_creature(&mut self, id: u32) -> bool {
        if let Some(c) = self.creatures.remove(&id) {
            if c.kind == CreatureKind::Player {
                self.stats.players_online = self.stats.players_online.saturating_sub(1);
            } else {
                self.stats.monsters_alive = self.stats.monsters_alive.saturating_sub(1);
            }
            true
        } else {
            false
        }
    }

    pub fn get_creature(&self, id: u32) -> Option<&GameCreature> {
        self.creatures.get(&id)
    }

    pub fn get_creature_mut(&mut self, id: u32) -> Option<&mut GameCreature> {
        self.creatures.get_mut(&id)
    }

    /// Unified creature lookup by ID.
    pub fn get_creature_by_id(&self, id: u32) -> Option<&GameCreature> {
        self.creatures.get(&id)
    }

    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }

    pub fn move_creature(&mut self, id: u32, new_pos: Position) -> bool {
        if let Some(c) = self.creatures.get_mut(&id) {
            c.position = new_pos;
            true
        } else {
            false
        }
    }

    /// Returns IDs of all creatures within Chebyshev range on the same floor.
    pub fn get_spectators_of(&self, center: &Position, range: u32) -> Vec<u32> {
        self.creatures
            .values()
            .filter(|c| {
                c.position.z == center.z
                    && c.position.x.abs_diff(center.x) <= range as u16
                    && c.position.y.abs_diff(center.y) <= range as u16
            })
            .map(|c| c.id)
            .collect()
    }

    // --- Stats ---

    pub fn stats(&self) -> &ServerStats {
        &self.stats
    }

    pub fn tick_uptime(&mut self, seconds: u64) {
        self.stats.uptime_seconds += seconds;
    }

    // --- Server rates ---

    pub fn set_experience_rate(&mut self, rate: u32) {
        self.experience_rate = rate;
    }
    pub fn experience_rate(&self) -> u32 {
        self.experience_rate
    }

    pub fn set_loot_rate(&mut self, rate: u32) {
        self.loot_rate = rate;
    }
    pub fn loot_rate(&self) -> u32 {
        self.loot_rate
    }

    pub fn set_spawn_rate(&mut self, rate: u32) {
        self.spawn_rate = rate;
    }
    pub fn spawn_rate(&self) -> u32 {
        self.spawn_rate
    }

    // -------------------------------------------------------------------------
    // Player lifecycle
    // -------------------------------------------------------------------------

    pub fn add_player(
        &mut self,
        name: impl Into<String>,
        pos: Position,
        max_health: i32,
        account_id: u32,
    ) -> u32 {
        let name: String = name.into();
        let id = self.next_creature_id;
        self.next_creature_id += 1;
        let mut creature =
            GameCreature::new(id, name.clone(), pos, CreatureKind::Player, max_health);
        creature.account_id = account_id;
        self.creatures.insert(id, creature);

        self.player_names.insert(name.to_lowercase(), id);
        if account_id != 0 {
            self.player_accounts.insert(account_id, id);
        }

        self.stats.players_online += 1;
        if self.stats.players_online > self.stats.players_peak {
            self.stats.players_peak = self.stats.players_online;
        }
        if self.stats.players_online > self.players_record {
            self.players_record = self.stats.players_online;
        }

        id
    }

    pub fn remove_player(&mut self, id: u32) -> bool {
        if let Some(creature) = self.creatures.remove(&id) {
            if creature.kind == CreatureKind::Player {
                self.player_names.remove(&creature.name.to_lowercase());
                if creature.account_id != 0 {
                    self.player_accounts.remove(&creature.account_id);
                }
                self.stats.players_online = self.stats.players_online.saturating_sub(1);
            }
            true
        } else {
            false
        }
    }

    pub fn get_player_by_name(&self, name: &str) -> Option<&GameCreature> {
        if name.is_empty() {
            return None;
        }
        let id = self.player_names.get(&name.to_lowercase())?;
        self.creatures.get(id)
    }

    pub fn get_player_by_account(&self, account_id: u32) -> Option<&GameCreature> {
        if account_id == 0 {
            return None;
        }
        let id = self.player_accounts.get(&account_id)?;
        self.creatures.get(id)
    }

    pub fn kick_player(&mut self, id: u32) -> bool {
        if self
            .creatures
            .get(&id)
            .map(|c| c.kind == CreatureKind::Player)
            .unwrap_or(false)
        {
            self.kicked_players.push(id);
            self.remove_player(id)
        } else {
            false
        }
    }

    pub fn broadcast_message(&mut self, text: impl Into<String>, class: MessageClass) {
        self.broadcast_log.push(BroadcastMessage {
            text: text.into(),
            class,
        });
    }

    pub fn broadcast_log(&self) -> &[BroadcastMessage] {
        &self.broadcast_log
    }

    pub fn players_online_count(&self) -> usize {
        self.player_names.len()
    }

    pub fn players_record(&self) -> u32 {
        self.players_record
    }

    pub fn load_players_record(&mut self, record: u32) {
        if record > self.players_record {
            self.players_record = record;
        }
    }

    pub fn kicked_players(&self) -> &[u32] {
        &self.kicked_players
    }

    // -------------------------------------------------------------------------
    // Item manipulation
    // -------------------------------------------------------------------------

    pub fn internal_create_item(item_type: Arc<ItemTypeData>, count: u8) -> Option<Item> {
        if count == 0 {
            return None;
        }
        Some(Item::new(item_type, count))
    }

    pub fn internal_add_item(container: &mut Container, item: Item, index: i32) -> ReturnValue {
        let resolved = container.query_destination(index);
        let check = container.query_add(resolved);
        if check != ReturnValue::NoError {
            return check;
        }

        if item.is_stackable() {
            let item_id = item.get_id();
            let incoming_count = item.get_count();

            let stack_idx = (0..container.size()).find(|&i| {
                if let Some(existing) = container.get_item(i) {
                    existing.get_id() == item_id && existing.get_count() < Item::MAX_STACK_COUNT
                } else {
                    // Unreachable in practice: `i` always satisfies
                    // `i < container.size()`, so `get_item(i)` returns `Some`.
                    // Kept for parity with the C++ defensive `if (existingItem)`
                    // check in `Game::internalAddItem`.
                    false
                }
            });

            if let Some(idx) = stack_idx {
                let existing_count = container.get_item(idx).unwrap().get_count();
                let space = Item::MAX_STACK_COUNT - existing_count;
                let merge = incoming_count.min(space);
                container
                    .get_item_mut(idx)
                    .unwrap()
                    .set_item_count(existing_count + merge);

                let leftover = incoming_count - merge;
                if leftover > 0 {
                    let mut remainder = item.deep_copy();
                    remainder.set_item_count(leftover);
                    return Self::internal_add_item(container, remainder, INDEX_WHEREEVER);
                }
                return ReturnValue::NoError;
            }
        }

        match container.add_item(item) {
            Ok(()) => ReturnValue::NoError,
            Err(_) => ReturnValue::ContainerNotEnoughRoom,
        }
    }

    pub fn internal_remove_item(
        container: &mut Container,
        index: usize,
        count: u32,
    ) -> ReturnValue {
        if count == 0 {
            return ReturnValue::NotPossible;
        }

        let ret = container.query_remove(index, count);
        if ret != ReturnValue::NoError {
            return ret;
        }

        let item = match container.get_item(index) {
            Some(i) => i,
            // Unreachable in practice: `query_remove` already returned
            // `ItemNotFound` for invalid indices. Defensive match mirrors C++
            // `Game::internalRemoveItem` re-fetch.
            None => return ReturnValue::ItemNotFound,
        };

        if item.is_stackable() && count < item.get_count() as u32 {
            let new_count = item.get_count() - count as u8;
            container
                .get_item_mut(index)
                .unwrap()
                .set_item_count(new_count);
        } else {
            container.remove_item(index);
        }

        ReturnValue::NoError
    }

    pub fn internal_move_item(
        from: &mut Container,
        from_index: usize,
        to: &mut Container,
        to_index: i32,
        count: u32,
    ) -> ReturnValue {
        if count == 0 {
            return ReturnValue::NotPossible;
        }

        let remove_ret = from.query_remove(from_index, count);
        if remove_ret != ReturnValue::NoError {
            return remove_ret;
        }

        let source = match from.get_item(from_index) {
            Some(i) => i,
            // Unreachable in practice: `from.query_remove` above already
            // returned `ItemNotFound` for invalid indices. Defensive match
            // mirrors C++ `Game::internalMoveItem` re-fetch pattern.
            None => return ReturnValue::ItemNotFound,
        };

        let transfer_count = if source.is_stackable() {
            count.min(source.get_count() as u32) as u8
        } else {
            1u8
        };

        let mut item_to_add = source.deep_copy();
        item_to_add.set_item_count(transfer_count);

        let dest_check = to.query_add(to.query_destination(to_index));
        if dest_check != ReturnValue::NoError {
            return dest_check;
        }

        let rem_ret = Self::internal_remove_item(from, from_index, transfer_count as u32);
        if rem_ret != ReturnValue::NoError {
            // Unreachable in practice: `transfer_count` was clamped to the
            // source's available count, and both `query_remove` + `query_add`
            // already passed. Defensive return mirrors C++ rollback pattern in
            // `Game::internalMoveItem`.
            return rem_ret;
        }

        Self::internal_add_item(to, item_to_add, to_index)
    }

    pub fn internal_transform_item(
        container: &mut Container,
        index: usize,
        new_type: Arc<ItemTypeData>,
    ) -> ReturnValue {
        match container.get_item_mut(index) {
            Some(item) => {
                item.transform(new_type);
                ReturnValue::NoError
            }
            None => ReturnValue::ItemNotFound,
        }
    }

    pub fn internal_remove_items(
        container: &mut Container,
        indices: &[usize],
        amount: u32,
        stackable: bool,
    ) -> u32 {
        let mut remaining = amount;
        let mut removed = 0u32;

        if stackable {
            for &idx in indices {
                if remaining == 0 {
                    break;
                }
                if let Some(item) = container.get_item(idx) {
                    let available = item.get_count() as u32;
                    let take = available.min(remaining);
                    if Self::internal_remove_item(container, idx, take) == ReturnValue::NoError {
                        removed += take;
                        remaining -= take;
                    }
                }
            }
        } else {
            let mut sorted: Vec<usize> = indices.to_vec();
            sorted.sort_unstable_by(|a, b| b.cmp(a));
            for idx in sorted {
                if Self::internal_remove_item(container, idx, 1) == ReturnValue::NoError {
                    removed += 1;
                }
            }
        }
        removed
    }

    pub fn internal_player_add_item(
        player_container: &mut Container,
        overflow_container: &mut Container,
        item: Item,
        drop_on_overflow: bool,
    ) -> ReturnValue {
        let ret = Self::internal_add_item(player_container, item.deep_copy(), INDEX_WHEREEVER);
        if ret != ReturnValue::NoError && drop_on_overflow {
            match overflow_container.add_item_back(item) {
                Ok(()) => return ReturnValue::NoError,
                Err(_) => return ReturnValue::ContainerNotEnoughRoom,
            }
        }
        ret
    }

    // -------------------------------------------------------------------------
    // Creature walk / turn
    // -------------------------------------------------------------------------

    pub fn internal_creature_turn(&mut self, id: u32, dir: Direction) -> bool {
        if let Some(c) = self.creatures.get_mut(&id) {
            if c.direction == dir {
                return false;
            }
            c.direction = dir;
            true
        } else {
            false
        }
    }

    pub fn check_creature_walk(&mut self, id: u32) {
        if let Some(c) = self.creatures.get_mut(&id) {
            if c.is_alive() {
                c.walk_ticks += 1;
            }
        }
    }

    pub fn add_creature_check(&mut self, id: u32) {
        if self.creature_check_active.contains_key(&id) {
            self.creature_check_active.insert(id, true);
        } else {
            self.creature_check_list.push(id);
            self.creature_check_active.insert(id, true);
        }
    }

    pub fn remove_creature_check(&mut self, id: u32) {
        if let Some(flag) = self.creature_check_active.get_mut(&id) {
            *flag = false;
        }
    }

    pub fn is_creature_check_active(&self, id: u32) -> bool {
        self.creature_check_active
            .get(&id)
            .copied()
            .unwrap_or(false)
    }

    // -------------------------------------------------------------------------
    // Map / spawn dispatch
    // Mirrors C++ Game::placeCreature, Game::removeCreature (map path),
    // Game::internalCreatureSay, Game::loadMap.
    // -------------------------------------------------------------------------

    /// Place a creature at `pos` on the map.
    ///
    /// Mirrors C++ `Game::placeCreature`:
    ///  1. Add creature to the registry.
    ///  2. Collect spectators within `MAP_SPECTATOR_RANGE` (Chebyshev, same floor).
    ///  3. Return creature ID + spectator IDs (would receive `onCreatureAppear`).
    pub fn place_creature(
        &mut self,
        name: impl Into<String>,
        pos: Position,
        kind: CreatureKind,
        max_health: i32,
    ) -> PlaceResult {
        let id = self.add_creature(name, pos, kind, max_health);
        let spectators = self.get_spectators_of(&pos, MAP_SPECTATOR_RANGE);
        PlaceResult {
            creature_id: id,
            spectators,
        }
    }

    /// Place a monster. Mirrors the monster path of C++ `internalPlaceCreature`.
    pub fn place_monster(
        &mut self,
        name: impl Into<String>,
        pos: Position,
        max_health: i32,
    ) -> PlaceResult {
        self.place_creature(name, pos, CreatureKind::Monster, max_health)
    }

    /// Place an NPC. Mirrors C++ `Spawns::startup` → `g_game.placeCreature(npc, …)`.
    pub fn place_npc(
        &mut self,
        name: impl Into<String>,
        pos: Position,
        max_health: i32,
    ) -> PlaceResult {
        self.place_creature(name, pos, CreatureKind::Npc, max_health)
    }

    /// Remove a creature from the map and collect spectators for dispatch.
    ///
    /// Mirrors C++ `Game::removeCreature` (map-dispatch portion):
    ///  1. Collect spectators at the creature's position.
    ///  2. Remove the creature.
    ///  3. Return whether removal succeeded and the spectator IDs.
    pub fn remove_creature_with_spectators(&mut self, id: u32) -> RemoveResult {
        let spectators = if let Some(c) = self.creatures.get(&id) {
            self.get_spectators_of(&c.position, MAP_SPECTATOR_RANGE)
        } else {
            vec![]
        };
        let removed = self.remove_creature(id);
        RemoveResult {
            removed,
            spectators,
        }
    }

    /// Broadcast a creature's speech to nearby spectators.
    ///
    /// Mirrors C++ `Game::internalCreatureSay`:
    ///  - `SpeakClass::Say` / `SpeakClass::MonsterSay` → `MAP_SPECTATOR_RANGE`
    ///  - `SpeakClass::Yell` → `MAP_SPECTATOR_RANGE * 2 + 2`
    ///
    /// Returns `None` when `text` is empty or the creature does not exist.
    pub fn creature_say(
        &self,
        creature_id: u32,
        text: impl Into<String>,
        speak_class: SpeakClass,
    ) -> Option<CreatureSayResult> {
        let text: String = text.into();
        if text.is_empty() {
            return None;
        }
        let creature = self.creatures.get(&creature_id)?;
        let range = match speak_class {
            SpeakClass::Yell => MAP_SPECTATOR_RANGE * 2 + 2,
            _ => MAP_SPECTATOR_RANGE,
        };
        let spectators = self.get_spectators_of(&creature.position, range);
        Some(CreatureSayResult {
            creature_id,
            text,
            speak_class,
            spectators,
        })
    }

    /// Record the loaded map path.
    ///
    /// Mirrors C++ `Game::loadMainMap` / `Game::loadMap`.
    pub fn load_map(&mut self, path: impl Into<String>) {
        self.map_path = Some(path.into());
    }

    /// Returns the currently loaded map path (if any).
    pub fn map_path(&self) -> Option<&str> {
        self.map_path.as_deref()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: u16, y: u16, z: u8) -> Position {
        Position { x, y, z }
    }

    // --- State machine ---

    #[test]
    fn initial_state_is_startup() {
        let g = Game::new();
        assert_eq!(g.state, GameState::Startup);
    }

    #[test]
    fn set_game_state_normal() {
        let mut g = Game::new();
        g.set_game_state(GameState::Normal);
        assert_eq!(g.state, GameState::Normal);
        assert!(g.is_open());
    }

    #[test]
    fn is_open_false_on_startup() {
        let g = Game::new();
        assert!(!g.is_open());
    }

    #[test]
    fn is_shutdown_true_on_closing() {
        let mut g = Game::new();
        g.set_game_state(GameState::Closing);
        assert!(g.is_shutdown());
    }

    #[test]
    fn is_shutdown_true_on_closed() {
        let mut g = Game::new();
        g.set_game_state(GameState::Closed);
        assert!(g.is_shutdown());
    }

    #[test]
    fn is_shutdown_false_on_normal() {
        let mut g = Game::new();
        g.set_game_state(GameState::Normal);
        assert!(!g.is_shutdown());
    }

    // --- Creature management ---

    #[test]
    fn add_player_increments_online_count() {
        let mut g = Game::new();
        g.add_creature("Alice", pos(100, 100, 7), CreatureKind::Player, 200);
        assert_eq!(g.stats().players_online, 1);
    }

    #[test]
    fn add_monster_increments_monsters_alive() {
        let mut g = Game::new();
        g.add_creature("Troll", pos(100, 100, 7), CreatureKind::Monster, 50);
        assert_eq!(g.stats().monsters_alive, 1);
    }

    #[test]
    fn add_creature_returns_unique_ids() {
        let mut g = Game::new();
        let id1 = g.add_creature("A", pos(1, 1, 7), CreatureKind::Monster, 10);
        let id2 = g.add_creature("B", pos(2, 2, 7), CreatureKind::Monster, 10);
        assert_ne!(id1, id2);
    }

    #[test]
    fn get_creature_after_add() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(50, 50, 7), CreatureKind::Monster, 30);
        let c = g.get_creature(id).unwrap();
        assert_eq!(c.name, "Rat");
    }

    #[test]
    fn get_creature_unknown_returns_none() {
        let g = Game::new();
        assert!(g.get_creature(9999).is_none());
    }

    #[test]
    fn remove_creature_returns_true() {
        let mut g = Game::new();
        let id = g.add_creature("X", pos(1, 1, 7), CreatureKind::Monster, 10);
        assert!(g.remove_creature(id));
        assert!(g.get_creature(id).is_none());
    }

    #[test]
    fn remove_creature_unknown_returns_false() {
        let mut g = Game::new();
        assert!(!g.remove_creature(999));
    }

    #[test]
    fn remove_player_decrements_online_count() {
        let mut g = Game::new();
        let id = g.add_creature("Bob", pos(1, 1, 7), CreatureKind::Player, 100);
        g.remove_creature(id);
        assert_eq!(g.stats().players_online, 0);
    }

    #[test]
    fn players_peak_tracks_maximum() {
        let mut g = Game::new();
        let id1 = g.add_creature("A", pos(1, 1, 7), CreatureKind::Player, 100);
        let id2 = g.add_creature("B", pos(2, 2, 7), CreatureKind::Player, 100);
        g.remove_creature(id1);
        g.remove_creature(id2);
        assert_eq!(g.stats().players_peak, 2);
    }

    #[test]
    fn creature_count_tracks_both_types() {
        let mut g = Game::new();
        g.add_creature("P", pos(1, 1, 7), CreatureKind::Player, 100);
        g.add_creature("M", pos(2, 2, 7), CreatureKind::Monster, 50);
        assert_eq!(g.creature_count(), 2);
    }

    #[test]
    fn move_creature_updates_position() {
        let mut g = Game::new();
        let id = g.add_creature("R", pos(10, 10, 7), CreatureKind::Monster, 20);
        assert!(g.move_creature(id, pos(15, 15, 7)));
        assert_eq!(g.get_creature(id).unwrap().position.x, 15);
    }

    #[test]
    fn move_creature_unknown_returns_false() {
        let mut g = Game::new();
        assert!(!g.move_creature(999, pos(0, 0, 7)));
    }

    // --- Damage / heal ---

    #[test]
    fn apply_damage_reduces_health() {
        let mut c = GameCreature::new(1, "Rat", pos(1, 1, 7), CreatureKind::Monster, 100);
        c.apply_damage(30);
        assert_eq!(c.health, 70);
    }

    #[test]
    fn apply_damage_clamps_at_zero() {
        let mut c = GameCreature::new(1, "Rat", pos(1, 1, 7), CreatureKind::Monster, 100);
        c.apply_damage(200);
        assert_eq!(c.health, 0);
        assert!(!c.is_alive());
    }

    #[test]
    fn heal_increases_health() {
        let mut c = GameCreature::new(1, "Hero", pos(1, 1, 7), CreatureKind::Player, 100);
        c.apply_damage(50);
        c.heal(20);
        assert_eq!(c.health, 70);
    }

    #[test]
    fn heal_clamps_at_max_health() {
        let mut c = GameCreature::new(1, "Hero", pos(1, 1, 7), CreatureKind::Player, 100);
        c.heal(999);
        assert_eq!(c.health, 100);
    }

    // --- Spectators ---

    #[test]
    fn get_spectators_returns_nearby_ids() {
        let mut g = Game::new();
        let id1 = g.add_creature("Near", pos(100, 100, 7), CreatureKind::Monster, 10);
        g.add_creature("Far", pos(200, 200, 7), CreatureKind::Monster, 10);
        let center = pos(100, 100, 7);
        let nearby = g.get_spectators_of(&center, 5);
        assert!(nearby.contains(&id1));
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn get_spectators_excludes_different_floor() {
        let mut g = Game::new();
        g.add_creature("Other Floor", pos(100, 100, 6), CreatureKind::Monster, 10);
        let center = pos(100, 100, 7);
        assert!(g.get_spectators_of(&center, 100).is_empty());
    }

    // --- Stats & rates ---

    #[test]
    fn tick_uptime_accumulates() {
        let mut g = Game::new();
        g.tick_uptime(60);
        g.tick_uptime(60);
        assert_eq!(g.stats().uptime_seconds, 120);
    }

    #[test]
    fn experience_rate_default_one() {
        let g = Game::new();
        assert_eq!(g.experience_rate(), 1);
    }

    #[test]
    fn set_experience_rate() {
        let mut g = Game::new();
        g.set_experience_rate(5);
        assert_eq!(g.experience_rate(), 5);
    }

    #[test]
    fn loot_rate_default_one() {
        let g = Game::new();
        assert_eq!(g.loot_rate(), 1);
    }

    #[test]
    fn set_loot_rate() {
        let mut g = Game::new();
        g.set_loot_rate(3);
        assert_eq!(g.loot_rate(), 3);
    }

    #[test]
    fn spawn_rate_default_one() {
        let g = Game::new();
        assert_eq!(g.spawn_rate(), 1);
    }

    #[test]
    fn set_spawn_rate() {
        let mut g = Game::new();
        g.set_spawn_rate(2);
        assert_eq!(g.spawn_rate(), 2);
    }

    // ── Map / spawn dispatch ──────────────────────────────────────────────────

    // --- place_creature ---

    #[test]
    fn place_creature_adds_creature_to_registry() {
        let mut g = Game::new();
        let result = g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        assert!(g.get_creature(result.creature_id).is_some());
    }

    #[test]
    fn place_creature_increments_monsters_alive() {
        let mut g = Game::new();
        g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        assert_eq!(g.stats().monsters_alive, 1);
    }

    #[test]
    fn place_creature_returns_spectators_at_position() {
        let mut g = Game::new();
        let player_id = g.add_creature("Alice", pos(100, 100, 7), CreatureKind::Player, 200);
        let result = g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        assert!(result.spectators.contains(&player_id));
        assert!(result.spectators.contains(&result.creature_id));
    }

    #[test]
    fn place_creature_spectators_excludes_out_of_range() {
        let mut g = Game::new();
        let _far = g.add_creature("FarPlayer", pos(200, 200, 7), CreatureKind::Player, 200);
        let result = g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        assert_eq!(result.spectators.len(), 1);
        assert!(result.spectators.contains(&result.creature_id));
    }

    #[test]
    fn place_creature_spectators_excludes_different_floor() {
        let mut g = Game::new();
        let other_floor = g.add_creature("Player6", pos(100, 100, 6), CreatureKind::Player, 200);
        let result = g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        assert!(!result.spectators.contains(&other_floor));
    }

    // --- place_monster ---

    #[test]
    fn place_monster_creates_monster_kind_creature() {
        let mut g = Game::new();
        let result = g.place_monster("Orc", pos(50, 50, 7), 100);
        let c = g.get_creature(result.creature_id).unwrap();
        assert_eq!(c.kind, CreatureKind::Monster);
        assert_eq!(c.name, "Orc");
    }

    #[test]
    fn place_monster_increments_monsters_alive() {
        let mut g = Game::new();
        g.place_monster("Orc", pos(50, 50, 7), 100);
        assert_eq!(g.stats().monsters_alive, 1);
    }

    // --- place_npc ---

    #[test]
    fn place_npc_creates_npc_kind_creature() {
        let mut g = Game::new();
        let result = g.place_npc("Banker", pos(60, 60, 7), 100);
        let c = g.get_creature(result.creature_id).unwrap();
        assert_eq!(c.kind, CreatureKind::Npc);
        assert_eq!(c.name, "Banker");
    }

    #[test]
    fn place_npc_counted_in_monsters_alive_bucket() {
        let mut g = Game::new();
        g.place_npc("Banker", pos(60, 60, 7), 100);
        assert_eq!(g.stats().monsters_alive, 1);
    }

    // --- remove_creature_with_spectators ---

    #[test]
    fn remove_creature_with_spectators_returns_removed_true() {
        let mut g = Game::new();
        let result = g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        let rem = g.remove_creature_with_spectators(result.creature_id);
        assert!(rem.removed);
    }

    #[test]
    fn remove_creature_with_spectators_removes_from_registry() {
        let mut g = Game::new();
        let result = g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        let cid = result.creature_id;
        g.remove_creature_with_spectators(cid);
        assert!(g.get_creature(cid).is_none());
    }

    #[test]
    fn remove_creature_with_spectators_returns_false_for_unknown() {
        let mut g = Game::new();
        let rem = g.remove_creature_with_spectators(9999);
        assert!(!rem.removed);
        assert!(rem.spectators.is_empty());
    }

    #[test]
    fn remove_creature_with_spectators_includes_nearby_spectators() {
        let mut g = Game::new();
        let player_id = g.add_creature("Alice", pos(100, 100, 7), CreatureKind::Player, 200);
        let result = g.place_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        let rem = g.remove_creature_with_spectators(result.creature_id);
        assert!(rem.spectators.contains(&player_id));
    }

    // --- creature_say ---

    #[test]
    fn creature_say_returns_none_on_empty_text() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        assert!(g.creature_say(id, "", SpeakClass::Say).is_none());
    }

    #[test]
    fn creature_say_returns_none_for_unknown_creature() {
        let g = Game::new();
        assert!(g.creature_say(9999, "hello", SpeakClass::Say).is_none());
    }

    #[test]
    fn creature_say_returns_text_and_speaker() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        let result = g.creature_say(id, "squeak", SpeakClass::Say).unwrap();
        assert_eq!(result.creature_id, id);
        assert_eq!(result.text, "squeak");
        assert_eq!(result.speak_class, SpeakClass::Say);
    }

    #[test]
    fn creature_say_includes_nearby_spectators() {
        let mut g = Game::new();
        let speaker_id = g.add_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        let listener_id = g.add_creature("Alice", pos(101, 100, 7), CreatureKind::Player, 200);
        let result = g
            .creature_say(speaker_id, "hello", SpeakClass::Say)
            .unwrap();
        assert!(result.spectators.contains(&listener_id));
    }

    #[test]
    fn creature_say_excludes_out_of_range_for_normal_speech() {
        let mut g = Game::new();
        let speaker_id = g.add_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        let far_id = g.add_creature(
            "FarPlayer",
            pos(100 + MAP_SPECTATOR_RANGE as u16 + 1, 100, 7),
            CreatureKind::Player,
            200,
        );
        let result = g
            .creature_say(speaker_id, "hello", SpeakClass::Say)
            .unwrap();
        assert!(!result.spectators.contains(&far_id));
    }

    #[test]
    fn creature_say_yell_uses_double_range() {
        let mut g = Game::new();
        let speaker_id = g.add_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        // Place listener at MAP_SPECTATOR_RANGE + 5 — within yell range (2*9+2=20)
        let yell_listener_id = g.add_creature(
            "DistantPlayer",
            pos(100 + MAP_SPECTATOR_RANGE as u16 + 5, 100, 7),
            CreatureKind::Player,
            200,
        );
        let result = g
            .creature_say(speaker_id, "HELLO", SpeakClass::Yell)
            .unwrap();
        assert!(result.spectators.contains(&yell_listener_id));
    }

    #[test]
    fn creature_say_monster_say_uses_normal_range() {
        let mut g = Game::new();
        let speaker_id = g.add_creature("Rat", pos(100, 100, 7), CreatureKind::Monster, 50);
        let near_id = g.add_creature("Near", pos(105, 100, 7), CreatureKind::Player, 200);
        let far_id = g.add_creature("Far", pos(150, 100, 7), CreatureKind::Player, 200);
        let result = g
            .creature_say(speaker_id, "growl", SpeakClass::MonsterSay)
            .unwrap();
        assert!(result.spectators.contains(&near_id));
        assert!(!result.spectators.contains(&far_id));
    }

    // --- load_map ---

    #[test]
    fn load_map_records_path() {
        let mut g = Game::new();
        g.load_map("data/world/thais.otbm");
        assert_eq!(g.map_path(), Some("data/world/thais.otbm"));
    }

    #[test]
    fn map_path_none_before_load() {
        let g = Game::new();
        assert!(g.map_path().is_none());
    }

    #[test]
    fn load_map_overwrites_previous_path() {
        let mut g = Game::new();
        g.load_map("data/world/old.otbm");
        g.load_map("data/world/new.otbm");
        assert_eq!(g.map_path(), Some("data/world/new.otbm"));
    }

    // ── Default impl ──────────────────────────────────────────────────────────
    // Mirrors C++ Game's default constructor — `Game::Game()`.
    #[test]
    fn default_impl_matches_new() {
        let g: Game = Default::default();
        assert_eq!(g.state, GameState::Startup);
        assert_eq!(g.creature_count(), 0);
        assert_eq!(g.experience_rate(), 1);
        assert_eq!(g.loot_rate(), 1);
        assert_eq!(g.spawn_rate(), 1);
        assert_eq!(g.players_record(), 0);
        assert!(g.map_path().is_none());
    }

    // ── get_creature_mut / get_creature_by_id ────────────────────────────────
    // Mirrors C++ `Game::getCreatureByID` (mutable + unified lookup paths).

    #[test]
    fn get_creature_mut_allows_modification() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(10, 10, 7), CreatureKind::Monster, 50);
        let c = g.get_creature_mut(id).unwrap();
        c.health = 5;
        assert_eq!(g.get_creature(id).unwrap().health, 5);
    }

    #[test]
    fn get_creature_mut_unknown_returns_none() {
        let mut g = Game::new();
        assert!(g.get_creature_mut(9999).is_none());
    }

    #[test]
    fn get_creature_by_id_returns_creature() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(1, 1, 7), CreatureKind::Monster, 10);
        assert_eq!(g.get_creature_by_id(id).unwrap().id, id);
    }

    #[test]
    fn get_creature_by_id_unknown_returns_none() {
        let g = Game::new();
        assert!(g.get_creature_by_id(9999).is_none());
    }

    // ── Player lifecycle ──────────────────────────────────────────────────────
    // Mirrors C++ `Game::addPlayer`, `Game::removePlayer`, `Game::getPlayerByName`,
    // `Game::getPlayerByAccount`, `Game::kickPlayer`, `Game::broadcastMessage`,
    // `Game::loadPlayersRecord`, `Game::checkPlayersRecord`.

    #[test]
    fn add_player_registers_in_name_index() {
        let mut g = Game::new();
        let id = g.add_player("Alice", pos(100, 100, 7), 200, 1);
        assert_eq!(g.get_player_by_name("Alice").unwrap().id, id);
    }

    #[test]
    fn add_player_name_lookup_is_case_insensitive() {
        let mut g = Game::new();
        let id = g.add_player("Bob", pos(100, 100, 7), 200, 2);
        // Looked up via lowercase
        assert_eq!(g.get_player_by_name("bob").unwrap().id, id);
        assert_eq!(g.get_player_by_name("BOB").unwrap().id, id);
    }

    #[test]
    fn add_player_registers_in_account_index() {
        let mut g = Game::new();
        let id = g.add_player("Carol", pos(100, 100, 7), 200, 42);
        assert_eq!(g.get_player_by_account(42).unwrap().id, id);
    }

    #[test]
    fn add_player_account_zero_not_indexed() {
        let mut g = Game::new();
        // account 0 means "unset" → not registered in account index
        g.add_player("Anon", pos(100, 100, 7), 200, 0);
        assert!(g.get_player_by_account(0).is_none());
    }

    #[test]
    fn add_player_increments_online_count_and_peak_and_record() {
        let mut g = Game::new();
        g.add_player("A", pos(1, 1, 7), 100, 1);
        g.add_player("B", pos(2, 2, 7), 100, 2);
        assert_eq!(g.stats().players_online, 2);
        assert_eq!(g.stats().players_peak, 2);
        assert_eq!(g.players_record(), 2);
    }

    #[test]
    fn add_player_record_only_grows() {
        let mut g = Game::new();
        let a = g.add_player("A", pos(1, 1, 7), 100, 1);
        let b = g.add_player("B", pos(2, 2, 7), 100, 2);
        assert_eq!(g.players_record(), 2);
        // Removing them does NOT decrement record
        g.remove_player(a);
        g.remove_player(b);
        assert_eq!(g.players_record(), 2);
    }

    #[test]
    fn remove_player_unindexes_name_and_account() {
        let mut g = Game::new();
        let id = g.add_player("Dave", pos(1, 1, 7), 100, 5);
        assert!(g.remove_player(id));
        assert!(g.get_player_by_name("Dave").is_none());
        assert!(g.get_player_by_account(5).is_none());
    }

    #[test]
    fn remove_player_via_player_path_decrements_online_count() {
        let mut g = Game::new();
        let id = g.add_player("Eve", pos(1, 1, 7), 100, 1);
        g.remove_player(id);
        assert_eq!(g.stats().players_online, 0);
    }

    #[test]
    fn remove_player_unknown_returns_false() {
        let mut g = Game::new();
        assert!(!g.remove_player(9999));
    }

    #[test]
    fn remove_player_on_monster_returns_true_but_skips_player_path() {
        // remove_player removes anything from the registry — but the player
        // bookkeeping branch only fires when the creature was a Player.
        let mut g = Game::new();
        let mid = g.add_creature("Rat", pos(1, 1, 7), CreatureKind::Monster, 10);
        assert!(g.remove_player(mid)); // creature existed → returned true
                                       // monster bookkeeping unaffected by player paths
    }

    #[test]
    fn get_player_by_name_empty_returns_none() {
        let g = Game::new();
        assert!(g.get_player_by_name("").is_none());
    }

    #[test]
    fn get_player_by_name_unknown_returns_none() {
        let g = Game::new();
        assert!(g.get_player_by_name("Ghost").is_none());
    }

    #[test]
    fn get_player_by_account_zero_returns_none() {
        let g = Game::new();
        assert!(g.get_player_by_account(0).is_none());
    }

    #[test]
    fn get_player_by_account_unknown_returns_none() {
        let g = Game::new();
        assert!(g.get_player_by_account(9999).is_none());
    }

    #[test]
    fn kick_player_records_id_and_removes() {
        let mut g = Game::new();
        let id = g.add_player("Frank", pos(1, 1, 7), 100, 1);
        assert!(g.kick_player(id));
        assert_eq!(g.kicked_players(), &[id]);
        assert!(g.get_player_by_name("Frank").is_none());
    }

    #[test]
    fn kick_player_unknown_returns_false() {
        let mut g = Game::new();
        assert!(!g.kick_player(9999));
        assert!(g.kicked_players().is_empty());
    }

    #[test]
    fn kick_player_on_monster_returns_false() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(1, 1, 7), CreatureKind::Monster, 50);
        assert!(!g.kick_player(id));
        assert!(g.kicked_players().is_empty());
        // monster still in the world
        assert!(g.get_creature(id).is_some());
    }

    #[test]
    fn broadcast_message_records_in_log() {
        let mut g = Game::new();
        g.broadcast_message("Server restarting", MessageClass::StatusWarning);
        let log = g.broadcast_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].text, "Server restarting");
        assert_eq!(log[0].class, MessageClass::StatusWarning);
    }

    #[test]
    fn broadcast_message_appends_in_order() {
        let mut g = Game::new();
        g.broadcast_message("First", MessageClass::StatusDefault);
        g.broadcast_message("Second", MessageClass::EventAdvance);
        let log = g.broadcast_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].text, "First");
        assert_eq!(log[1].text, "Second");
    }

    #[test]
    fn players_online_count_reflects_name_index() {
        let mut g = Game::new();
        g.add_player("A", pos(1, 1, 7), 100, 1);
        g.add_player("B", pos(2, 2, 7), 100, 2);
        assert_eq!(g.players_online_count(), 2);
    }

    #[test]
    fn load_players_record_grows_record() {
        let mut g = Game::new();
        g.load_players_record(50);
        assert_eq!(g.players_record(), 50);
    }

    #[test]
    fn load_players_record_does_not_shrink_record() {
        let mut g = Game::new();
        g.load_players_record(100);
        g.load_players_record(20); // lower than current
        assert_eq!(g.players_record(), 100);
    }

    // ── Item manipulation ────────────────────────────────────────────────────
    // Mirrors C++ `Game::internalCreateItem`, `internalAddItem`, `internalRemoveItem`,
    // `internalMoveItem`, `internalTransformItem`, `internalRemoveItems`,
    // `internalPlayerAddItem`.

    fn make_item_type(id: u16, stackable: bool, moveable: bool) -> Arc<ItemTypeData> {
        Arc::new(ItemTypeData {
            id,
            client_id: id + 100,
            stackable,
            pickupable: true,
            moveable,
            name: format!("item-{}", id),
            article: "a".to_string(),
            show_count: true,
            ..Default::default()
        })
    }

    // --- internal_create_item ---

    #[test]
    fn internal_create_item_returns_some_when_count_positive() {
        let t = make_item_type(1, false, true);
        let item = Game::internal_create_item(t, 1);
        assert!(item.is_some());
        assert_eq!(item.unwrap().get_count(), 1);
    }

    #[test]
    fn internal_create_item_returns_none_for_zero_count() {
        let t = make_item_type(1, false, true);
        assert!(Game::internal_create_item(t, 0).is_none());
    }

    #[test]
    fn internal_create_item_stackable_keeps_count() {
        let t = make_item_type(2, true, true);
        let item = Game::internal_create_item(t, 50).unwrap();
        assert_eq!(item.get_count(), 50);
    }

    // --- internal_add_item ---

    #[test]
    fn internal_add_item_into_empty_succeeds() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        let item = Item::new(t, 1);
        let ret = Game::internal_add_item(&mut c, item, INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 1);
    }

    #[test]
    fn internal_add_item_locked_container_returns_error() {
        // Locked container → query_add returns ContainerLocked.
        let mut c = Container::with_flags(99, 10, false, false);
        let t = make_item_type(1, false, true);
        let item = Item::new(t, 1);
        let ret = Game::internal_add_item(&mut c, item, INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::ContainerLocked);
    }

    #[test]
    fn internal_add_item_full_container_returns_not_enough_room() {
        let mut c = Container::new(99, 1);
        let t = make_item_type(1, false, true);
        c.add_item(Item::new(t.clone(), 1)).unwrap();
        let ret = Game::internal_add_item(&mut c, Item::new(t, 1), INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::ContainerNotEnoughRoom);
    }

    #[test]
    fn internal_add_item_paginated_full_non_stackable_hits_add_err_branch() {
        // Paginated containers allow query_add to pass when full; the actual
        // add_item then returns Err(Full), exercising the fallback branch.
        let mut c = Container::with_flags(99, 1, true, true);
        let t = make_item_type(1, false, true);
        c.add_item(Item::new(t.clone(), 1)).unwrap();
        let ret = Game::internal_add_item(&mut c, Item::new(t, 1), INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::ContainerNotEnoughRoom);
    }

    #[test]
    fn internal_add_item_stackable_merges_into_existing() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        // Existing stack of 30
        c.add_item(Item::new(t.clone(), 30)).unwrap();
        // Add 20 more of same item id → merges into existing stack
        let ret = Game::internal_add_item(&mut c, Item::new(t, 20), INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 1);
        assert_eq!(c.get_item(0).unwrap().get_count(), 50);
    }

    #[test]
    fn internal_add_item_stackable_overflow_spills_to_new_stack() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(3, true, true);
        // existing stack of 80
        c.add_item(Item::new(t.clone(), 80)).unwrap();
        // add 40 more → 20 fits into stack, 20 leftover starts new stack
        let ret = Game::internal_add_item(&mut c, Item::new(t, 40), INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 2);
        // Order in the container: leftover (pushed last via recursion) at front.
        let total: u32 = c.iter().map(|i| i.get_count() as u32).sum();
        assert_eq!(total, 120);
    }

    #[test]
    fn internal_add_item_stackable_with_no_mergeable_existing_makes_new() {
        let mut c = Container::new(99, 10);
        let t_a = make_item_type(4, true, true);
        let t_b = make_item_type(5, true, true);
        c.add_item(Item::new(t_a, 10)).unwrap();
        // different item id → no merge → new entry
        let ret = Game::internal_add_item(&mut c, Item::new(t_b, 10), INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 2);
    }

    #[test]
    fn internal_add_item_stackable_existing_at_max_does_not_merge() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(6, true, true);
        // existing already at max stack
        c.add_item(Item::new(t.clone(), Item::MAX_STACK_COUNT))
            .unwrap();
        // 20 more → no merge possible → new entry
        let ret = Game::internal_add_item(&mut c, Item::new(t, 20), INDEX_WHEREEVER);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 2);
    }

    // --- internal_remove_item ---

    #[test]
    fn internal_remove_item_zero_count_returns_not_possible() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        c.add_item(Item::new(t, 1)).unwrap();
        let ret = Game::internal_remove_item(&mut c, 0, 0);
        assert_eq!(ret, ReturnValue::NotPossible);
    }

    #[test]
    fn internal_remove_item_invalid_index_propagates_remove_error() {
        let mut c = Container::new(99, 10);
        let ret = Game::internal_remove_item(&mut c, 5, 1);
        assert_eq!(ret, ReturnValue::ItemNotFound);
    }

    #[test]
    fn internal_remove_item_non_stackable_removes_entry() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        c.add_item(Item::new(t, 1)).unwrap();
        let ret = Game::internal_remove_item(&mut c, 0, 1);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 0);
    }

    #[test]
    fn internal_remove_item_stackable_reduces_count() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        c.add_item(Item::new(t, 30)).unwrap();
        let ret = Game::internal_remove_item(&mut c, 0, 10);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 1);
        assert_eq!(c.get_item(0).unwrap().get_count(), 20);
    }

    #[test]
    fn internal_remove_item_stackable_full_remove_drops_entry() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        c.add_item(Item::new(t, 5)).unwrap();
        let ret = Game::internal_remove_item(&mut c, 0, 5);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.size(), 0);
    }

    #[test]
    fn internal_remove_item_more_than_present_returns_not_possible() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        c.add_item(Item::new(t, 3)).unwrap();
        // query_remove rejects since count > available
        let ret = Game::internal_remove_item(&mut c, 0, 5);
        assert_eq!(ret, ReturnValue::NotPossible);
    }

    // --- internal_move_item ---

    #[test]
    fn internal_move_item_zero_count_returns_not_possible() {
        let mut a = Container::new(99, 10);
        let mut b = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        a.add_item(Item::new(t, 1)).unwrap();
        let ret = Game::internal_move_item(&mut a, 0, &mut b, INDEX_WHEREEVER, 0);
        assert_eq!(ret, ReturnValue::NotPossible);
    }

    #[test]
    fn internal_move_item_invalid_from_index_returns_item_not_found() {
        let mut a = Container::new(99, 10);
        let mut b = Container::new(99, 10);
        let ret = Game::internal_move_item(&mut a, 5, &mut b, INDEX_WHEREEVER, 1);
        assert_eq!(ret, ReturnValue::ItemNotFound);
    }

    #[test]
    fn internal_move_item_non_stackable_moves_single_unit() {
        let mut a = Container::new(99, 10);
        let mut b = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        a.add_item(Item::new(t, 1)).unwrap();
        let ret = Game::internal_move_item(&mut a, 0, &mut b, INDEX_WHEREEVER, 1);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(a.size(), 0);
        assert_eq!(b.size(), 1);
    }

    #[test]
    fn internal_move_item_stackable_moves_partial_count() {
        let mut a = Container::new(99, 10);
        let mut b = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        a.add_item(Item::new(t, 50)).unwrap();
        let ret = Game::internal_move_item(&mut a, 0, &mut b, INDEX_WHEREEVER, 20);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(a.get_item(0).unwrap().get_count(), 30);
        assert_eq!(b.get_item(0).unwrap().get_count(), 20);
    }

    #[test]
    fn internal_move_item_to_locked_destination_returns_locked() {
        let mut a = Container::new(99, 10);
        let mut b = Container::with_flags(99, 10, false, false);
        let t = make_item_type(1, false, true);
        a.add_item(Item::new(t, 1)).unwrap();
        let ret = Game::internal_move_item(&mut a, 0, &mut b, INDEX_WHEREEVER, 1);
        assert_eq!(ret, ReturnValue::ContainerLocked);
        // source unchanged
        assert_eq!(a.size(), 1);
    }

    #[test]
    fn internal_move_item_caps_stackable_to_available_count() {
        // count parameter exceeds actual stack size; query_remove fails first.
        let mut a = Container::new(99, 10);
        let mut b = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        a.add_item(Item::new(t, 3)).unwrap();
        // Try to move 50 from stack of 3 → query_remove returns NotPossible.
        let ret = Game::internal_move_item(&mut a, 0, &mut b, INDEX_WHEREEVER, 50);
        assert_eq!(ret, ReturnValue::NotPossible);
    }

    // --- internal_transform_item ---

    #[test]
    fn internal_transform_item_changes_type() {
        let mut c = Container::new(99, 10);
        let t1 = make_item_type(10, false, true);
        let t2 = make_item_type(20, false, true);
        c.add_item(Item::new(t1, 1)).unwrap();
        let ret = Game::internal_transform_item(&mut c, 0, t2);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(c.get_item(0).unwrap().get_id(), 20);
    }

    #[test]
    fn internal_transform_item_invalid_index_returns_not_found() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(10, false, true);
        let ret = Game::internal_transform_item(&mut c, 0, t);
        assert_eq!(ret, ReturnValue::ItemNotFound);
    }

    // --- internal_remove_items ---

    #[test]
    fn internal_remove_items_stackable_takes_from_each_index() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        // Container uses push_front; after three adds the order is
        // front-to-back: stack-7, stack-10, stack-5.
        c.add_item(Item::new(t.clone(), 5)).unwrap();
        c.add_item(Item::new(t.clone(), 10)).unwrap();
        c.add_item(Item::new(t, 7)).unwrap();
        // Walk indices in order — when a stack is fully removed the container
        // shifts, so later indices point at different stacks. We just verify
        // the function exercises the stackable-branch successfully and returns
        // a count ≤ requested.
        let removed = Game::internal_remove_items(&mut c, &[0], 5, true);
        assert_eq!(removed, 5);
        // index 0 was the count-7 stack → reduced by 5 → now 2
        assert_eq!(c.get_item(0).unwrap().get_count(), 2);
    }

    #[test]
    fn internal_remove_items_stackable_stops_when_remaining_zero() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        c.add_item(Item::new(t.clone(), 20)).unwrap();
        c.add_item(Item::new(t, 30)).unwrap();
        // request only 10 → break after first index consumed
        let removed = Game::internal_remove_items(&mut c, &[0, 1], 10, true);
        assert_eq!(removed, 10);
    }

    #[test]
    fn internal_remove_items_non_stackable_removes_each_index_once() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        c.add_item(Item::new(t.clone(), 1)).unwrap();
        c.add_item(Item::new(t.clone(), 1)).unwrap();
        c.add_item(Item::new(t, 1)).unwrap();
        let removed = Game::internal_remove_items(&mut c, &[0, 1, 2], 3, false);
        assert_eq!(removed, 3);
        assert_eq!(c.size(), 0);
    }

    #[test]
    fn internal_remove_items_stackable_handles_missing_index_silently() {
        let mut c = Container::new(99, 10);
        let t = make_item_type(2, true, true);
        c.add_item(Item::new(t, 5)).unwrap();
        // Index 9 doesn't exist → method skips it via `if let Some(item)`.
        let removed = Game::internal_remove_items(&mut c, &[9, 0], 3, true);
        assert_eq!(removed, 3);
    }

    // --- internal_player_add_item ---

    #[test]
    fn internal_player_add_item_into_inventory_succeeds() {
        let mut inv = Container::new(99, 10);
        let mut floor = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        let ret = Game::internal_player_add_item(&mut inv, &mut floor, Item::new(t, 1), false);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(inv.size(), 1);
        assert_eq!(floor.size(), 0);
    }

    #[test]
    fn internal_player_add_item_full_inventory_no_overflow_returns_error() {
        let mut inv = Container::new(99, 1);
        let mut floor = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        inv.add_item(Item::new(t.clone(), 1)).unwrap();
        let ret = Game::internal_player_add_item(&mut inv, &mut floor, Item::new(t, 1), false);
        assert_eq!(ret, ReturnValue::ContainerNotEnoughRoom);
        // overflow disabled → nothing ends up on the floor
        assert_eq!(floor.size(), 0);
    }

    #[test]
    fn internal_player_add_item_drops_on_overflow() {
        let mut inv = Container::new(99, 1);
        let mut floor = Container::new(99, 10);
        let t = make_item_type(1, false, true);
        inv.add_item(Item::new(t.clone(), 1)).unwrap();
        // overflow=true → item goes to floor when inv is full
        let ret = Game::internal_player_add_item(&mut inv, &mut floor, Item::new(t, 1), true);
        assert_eq!(ret, ReturnValue::NoError);
        assert_eq!(floor.size(), 1);
    }

    #[test]
    fn internal_player_add_item_drop_on_overflow_when_floor_also_full() {
        let mut inv = Container::new(99, 1);
        let mut floor = Container::new(99, 1);
        let t = make_item_type(1, false, true);
        inv.add_item(Item::new(t.clone(), 1)).unwrap();
        floor.add_item(Item::new(t.clone(), 1)).unwrap();
        // both full + drop_on_overflow=true → second call to floor.add_item_back fails
        let ret = Game::internal_player_add_item(&mut inv, &mut floor, Item::new(t, 1), true);
        assert_eq!(ret, ReturnValue::ContainerNotEnoughRoom);
    }

    // ── Creature walk / turn ─────────────────────────────────────────────────
    // Mirrors C++ `Game::internalCreatureTurn`, `Game::checkCreatureWalk`,
    // `Game::addCreatureCheck`, `Game::removeCreatureCheck`.

    #[test]
    fn internal_creature_turn_changes_direction() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(1, 1, 7), CreatureKind::Monster, 50);
        assert!(g.internal_creature_turn(id, Direction::North));
        assert_eq!(g.get_creature(id).unwrap().direction, Direction::North);
    }

    #[test]
    fn internal_creature_turn_same_direction_returns_false() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(1, 1, 7), CreatureKind::Monster, 50);
        // Default direction is South — same as requested → false (no change).
        assert!(!g.internal_creature_turn(id, Direction::South));
    }

    #[test]
    fn internal_creature_turn_unknown_returns_false() {
        let mut g = Game::new();
        assert!(!g.internal_creature_turn(9999, Direction::West));
    }

    #[test]
    fn check_creature_walk_increments_ticks_for_alive() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(1, 1, 7), CreatureKind::Monster, 50);
        g.check_creature_walk(id);
        g.check_creature_walk(id);
        assert_eq!(g.get_creature(id).unwrap().walk_ticks, 2);
    }

    #[test]
    fn check_creature_walk_dead_creature_does_not_tick() {
        let mut g = Game::new();
        let id = g.add_creature("Rat", pos(1, 1, 7), CreatureKind::Monster, 10);
        g.get_creature_mut(id).unwrap().apply_damage(100);
        assert!(!g.get_creature(id).unwrap().is_alive());
        g.check_creature_walk(id);
        assert_eq!(g.get_creature(id).unwrap().walk_ticks, 0);
    }

    #[test]
    fn check_creature_walk_unknown_is_noop() {
        let mut g = Game::new();
        // Should not panic.
        g.check_creature_walk(9999);
    }

    #[test]
    fn add_creature_check_registers_active_id() {
        let mut g = Game::new();
        g.add_creature_check(1);
        assert!(g.is_creature_check_active(1));
    }

    #[test]
    fn add_creature_check_existing_just_reactivates() {
        let mut g = Game::new();
        g.add_creature_check(7);
        g.remove_creature_check(7);
        assert!(!g.is_creature_check_active(7));
        // Adding again flips the existing entry back to active.
        g.add_creature_check(7);
        assert!(g.is_creature_check_active(7));
    }

    #[test]
    fn remove_creature_check_deactivates_existing() {
        let mut g = Game::new();
        g.add_creature_check(2);
        g.remove_creature_check(2);
        assert!(!g.is_creature_check_active(2));
    }

    #[test]
    fn remove_creature_check_unknown_is_noop() {
        let mut g = Game::new();
        // Removing an id never added is a silent no-op.
        g.remove_creature_check(9999);
        assert!(!g.is_creature_check_active(9999));
    }

    #[test]
    fn is_creature_check_active_unknown_returns_false() {
        let g = Game::new();
        assert!(!g.is_creature_check_active(9999));
    }
}
