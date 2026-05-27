//! Migrated from forgottenserver/src/player.h and player.cpp
//!
//! Player data model. Implements 16 sections (a–p) as data bags without
//! networked game logic, Lua scripting, or database I/O.

use forgottenserver_common::constants::WeaponType;
use forgottenserver_common::position::Position;
use forgottenserver_items::item::Item;
use forgottenserver_items::items_registry::slot_position;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const PLAYER_MAX_SPEED: i32 = 1500;
pub const PLAYER_MIN_SPEED: i32 = 10;
pub const MINIMUM_SKILL_LEVEL: u16 = 10;

pub const STAMINA_MAX: u16 = 2520; // 42 hours in minutes
pub const STAMINA_PREMIUM_THRESHOLD: u16 = 2400; // above this is premium bonus zone
pub const STAMINA_EXHAUSTED_THRESHOLD: u16 = 840; // below this: 0.5× XP
pub const STAMINA_BONUS_ABOVE: u16 = 2340; // above this + premium: 1.5× XP
pub const SOUL_MAX_DEFAULT: u8 = 100;
pub const CAPACITY_DEFAULT: u32 = 400;

// ---------------------------------------------------------------------------
// Free functions — XP / speed formulas
// ---------------------------------------------------------------------------

/// Total XP required to reach `level`.
///
/// Mirrors the C++ `Player::getExpForLevel` formula exactly:
/// `(((lv - 6) * lv + 17) * lv - 12) / 6 * 100`
///
/// Returns 0 for level ≤ 1.
pub fn xp_for_level(level: u32) -> u64 {
    if level <= 1 {
        return 0;
    }
    let lv = level as i64;
    let raw = (((lv - 6) * lv + 17) * lv - 12) / 6 * 100;
    raw.max(0) as u64
}

/// Base movement speed at `level`.
pub fn base_speed(level: u32) -> i32 {
    220 + (level as i32 - 1) * 2
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Mirrors C++ `skillsid_t`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SkillsId {
    Level = 0,
    Tries = 1,
    Percent = 2,
}

/// Mirrors C++ `fightMode_t`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FightMode {
    #[default]
    Attack = 1,
    Balanced = 2,
    Defense = 3,
}

/// Mirrors C++ `pvpMode_t`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum PvpMode {
    #[default]
    Dove = 0,
    WhiteHand = 1,
    YellowHand = 2,
    RedFist = 3,
}

/// Mirrors C++ `tradestate_t`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TradeState {
    #[default]
    None = 0,
    Initiated = 1,
    Accept = 2,
    Acknowledge = 3,
    Transfer = 4,
}

/// Skull types matching C++ `Skulls_t`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Skull {
    #[default]
    None = 0,
    Yellow = 1,
    Green = 2,
    White = 3,
    Red = 4,
    Black = 5,
    Orange = 6,
}

/// Skill IDs matching C++ `skills_t` order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SkillType {
    Fist = 0,
    Club = 1,
    Sword = 2,
    Axe = 3,
    Distance = 4,
    Shield = 5,
    Fishing = 6,
    MagicLevel = 7,
}

pub const SKILL_COUNT: usize = 8;

/// Offline training skill matches C++ values including -1 for Unset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum OfflineTrainingSkill {
    Shield = 0,
    Distance = 1,
    Club = 2,
    Sword = 3,
    Axe = 4,
    Fist = 5,
    #[default]
    Unset = -1,
}

/// Inventory slot IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InventorySlot {
    Head = 1,
    Necklace = 2,
    Backpack = 3,
    Armor = 4,
    Right = 5,
    Left = 6,
    Legs = 7,
    Feet = 8,
    Ring = 9,
    Ammo = 10,
    Depot = 11,
}

pub const INVENTORY_SLOT_COUNT: usize = 11;

// ---------------------------------------------------------------------------
// QueryAdd / QueryRemove flags (mirrors C++ FLAG_* constants)
// ---------------------------------------------------------------------------

pub mod query_flags {
    /// Item is being moved from a child container owned by this player.
    pub const CHILD_IS_OWNER: u32 = 1 << 0;
    /// Bypass capacity limit check.
    pub const NO_LIMIT: u32 = 1 << 1;
    /// Ignore the moveable flag check.
    pub const IGNORE_NOT_MOVEABLE: u32 = 1 << 2;
}

// ---------------------------------------------------------------------------
// ReturnValue — cylinder query result (mirrors C++ `ReturnValue`)
// ---------------------------------------------------------------------------

/// Return values for cylinder `query_add` / `query_remove`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnValue {
    NoError,
    NotPossible,
    CannotPickup,
    /// Item is a store item; cannot be moved.
    ItemCannotBeMovedThere,
    /// Wrong equipment slot for body-slot items.
    CannotBeDressed,
    /// Item requires both hands; place in left slot.
    PutThisObjectInBothHands,
    /// Item is a one-hand weapon; place in right or left slot.
    PutThisObjectInYourHand,
    /// Left-hand slot occupied and right-hand is a two-hander.
    BothHandsNeedToBeFree,
    /// Must drop two-handed item first.
    DropTwoHandedItem,
    /// Cannot equip two shields.
    CanOnlyUseOneShield,
    /// Cannot equip two melee weapons.
    CanOnlyUseOneWeapon,
    /// Not enough carry capacity.
    NotEnoughCapacity,
    /// Not enough room in this container.
    NotEnoughRoom,
    /// Count is zero or exceeds stack.
    NotMoveable,
    /// Source item and destination differ; need exchange.
    NeedExchange,
}

// ---------------------------------------------------------------------------
// Skill struct
// ---------------------------------------------------------------------------

/// Mirrors C++ `Skill` struct
#[derive(Debug, Clone)]
pub struct Skill {
    pub tries: u64,
    pub level: u16,
    pub percent: u16,
}

impl Default for Skill {
    fn default() -> Self {
        Skill {
            tries: 0,
            level: MINIMUM_SKILL_LEVEL,
            percent: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// VIPEntry struct
// ---------------------------------------------------------------------------

/// Mirrors C++ `VIPEntry` struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VIPEntry {
    pub guid: u32,
    pub name: String,
    pub description: String,
    pub icon: u32,
    pub notify: bool,
}

impl VIPEntry {
    pub fn new(
        guid: u32,
        name: impl Into<String>,
        description: impl Into<String>,
        icon: u32,
        notify: bool,
    ) -> Self {
        VIPEntry {
            guid,
            name: name.into(),
            description: description.into(),
            icon,
            notify,
        }
    }
}

// ---------------------------------------------------------------------------
// StatsSnapshot — sendStats data shape
// ---------------------------------------------------------------------------

/// Mirrors the data fields sent by C++ `sendStats()` / `AddPlayerStats`.
/// Network serialisation is out of scope for the entity crate; this provides
/// a pure-data view for testing and upper layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsSnapshot {
    pub health: i32,
    pub max_health: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub capacity: u32,
    pub experience: u64,
    pub level: u32,
    pub level_percent: u8,
    pub stamina: u16,
    pub magic_level: u32,
    pub magic_level_percent: u16,
    pub speed: i32,
    pub soul: u8,
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

/// Core player data-bag.
#[derive(Debug)]
pub struct Player {
    pub guid: u32,
    pub name: String,
    pub vocation_id: u16,

    // Runtime state
    health: i32,
    mana: i32,
    experience: u64,
    pub position: Position,
    pub temple_pos: Position,

    // Persistence positions (mirrors C++ loginPosition / tempPosition)
    login_position: Position,
    temp_position: Option<Position>,

    // (a) Stamina
    stamina: u16,

    // (b) Premium
    premium: bool,
    premium_days: u32,

    // (c) Skull / emblem
    skull: Skull,
    skull_ticks: u32,

    // (d) Vocation stats
    level: u32,
    level_percent: u8,
    max_health: i32,
    max_mana: i32,
    capacity: u32,

    // (d-3) Speed
    speed: i32,

    // (d-4) Magic level (separate from skills array)
    magic_level: u32,
    mana_spent: u64,
    magic_level_percent: u16,

    // (e) Skills
    skills: [Skill; SKILL_COUNT],

    // (f) Offline training
    offline_training_skill: OfflineTrainingSkill,
    offline_training_time: i32,

    // (g) Inventory
    inventory: [Option<Item>; INVENTORY_SLOT_COUNT],

    // (h) Depot
    max_depot_items: u32,
    /// Depot items keyed by depot_id (0..=99), each holding a list of items.
    /// Mirrors the C++ `depotChests` map used in IOLoginData::loadPlayer.
    depot_items: HashMap<u32, Vec<Item>>,

    // (i) Party
    party_id: Option<u32>,

    // (j) Trade state
    trade_state: TradeState,
    trade_partner_guid: Option<u32>,

    // (k) Weight (capacity tracked above)

    // (l) VIP list
    vip_list: Vec<VIPEntry>,
    max_vip_entries: u32,

    // (m) Blessings (5 blessings, stored as bitmask bits 0-4)
    blessings: u8,

    // (n) Unjustified kills
    unjustified_kills: u32,

    // (o) Soul
    soul: u8,
    soul_max: u8,

    // (p) Combat modes
    safe_mode: bool,
    fight_mode: FightMode,
    pvp_mode: PvpMode,

    // (q) Combat state — condition immunities and zone/fight flags
    /// Bitmask of ConditionTypeFlags values to which this player is immune.
    condition_immunities: i32,
    /// True when the player's current tile has the protection-zone flag.
    in_protection_zone: bool,
    /// True while the player is actively in a fight (set by on_attacking / on_attacked).
    in_fight: bool,

    // -----------------------------------------------------------------------
    // Task 8.5 — social & misc fields
    // -----------------------------------------------------------------------

    // (8.5-a) Friend list — player social contacts stored as GUIDs.
    // Mirrors the friend-list concept in many OT server forks.
    // Duplicate check and max_friends cap are enforced by add_friend().
    friend_list: Vec<u32>,
    max_friends: u32,

    // (8.5-b) Guild membership
    // Mirrors C++ `Guild_ptr guild` (presence == is guild member) + `guildNick`.
    // We store only guild_id so the data-bag stays self-contained.
    guild_id: Option<u32>,
    guild_nick: String,

    // (8.5-c) Pending party invitations received by this player.
    // Mirrors C++ `invitePartyList` (forward_list<Party*>).
    // We store party leader GUIDs since there is no Party object in the data-bag.
    pending_party_invites: Vec<u32>,

    // (8.5-d) Channel subscriptions — IDs of chat channels this player has joined.
    subscribed_channels: Vec<u16>,

    // (8.5-e) Client version reported during the login handshake.
    // Mirrors C++ `getProtocolVersion()` via ProtocolGame client.
    client_version: u16,

    // (8.5-f) Key-value storage for Lua script / quest state.
    // Mirrors C++ `Creature::storageMap: std::map<uint32_t, i32>`.
    // An absent key means "no value set"; setting `None` removes the key
    // (mirrors C++ `storageMap.erase(key)` path in setStorageValue).
    storage: HashMap<u32, i32>,
}

impl Player {
    /// Create a new player with the given guid, name, and vocation_id.
    pub fn new(guid: u32, name: impl Into<String>, vocation_id: u16) -> Self {
        Player {
            guid,
            name: name.into(),
            vocation_id,
            // Runtime state
            health: 100,
            mana: 0,
            experience: 0,
            position: Position { x: 0, y: 0, z: 7 },
            temple_pos: Position { x: 0, y: 0, z: 7 },
            login_position: Position { x: 0, y: 0, z: 0 },
            temp_position: None,
            // (a) Stamina — max 42 hours = 2520 minutes
            stamina: STAMINA_MAX,
            // (b) Premium
            premium: false,
            premium_days: 0,
            // (c) Skull
            skull: Skull::None,
            skull_ticks: 0,
            // (d) Vocation stats
            level: 1,
            level_percent: 0,
            max_health: 100,
            max_mana: 0,
            capacity: CAPACITY_DEFAULT,
            speed: base_speed(1),
            magic_level: 0,
            mana_spent: 0,
            magic_level_percent: 0,
            // (e) Skills — all default to MINIMUM_SKILL_LEVEL
            skills: std::array::from_fn(|_| Skill::default()),
            // (f) Offline training
            offline_training_skill: OfflineTrainingSkill::Unset,
            offline_training_time: 0,
            // (g) Inventory — all empty
            inventory: std::array::from_fn(|_| None),
            // (h) Depot
            max_depot_items: 0,
            depot_items: HashMap::new(),
            // (i) Party
            party_id: None,
            // (j) Trade
            trade_state: TradeState::None,
            trade_partner_guid: None,
            // (l) VIP
            vip_list: Vec::new(),
            max_vip_entries: 0,
            // (m) Blessings
            blessings: 0,
            // (n) Kills
            unjustified_kills: 0,
            // (o) Soul
            soul: SOUL_MAX_DEFAULT,
            soul_max: SOUL_MAX_DEFAULT,
            // (p) Combat modes
            safe_mode: true,
            fight_mode: FightMode::Attack,
            pvp_mode: PvpMode::Dove,
            // (q) Combat state
            condition_immunities: 0,
            in_protection_zone: false,
            in_fight: false,
            // (8.5) Social & misc
            friend_list: Vec::new(),
            max_friends: 200,
            guild_id: None,
            guild_nick: String::new(),
            pending_party_invites: Vec::new(),
            subscribed_channels: Vec::new(),
            client_version: 0,
            storage: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Position persistence (loginPosition / tempPosition / getSavedPosition)
    // Mirrors C++ player.h getLoginPosition() and IOLoginData loadPlayer/save.
    // -----------------------------------------------------------------------

    /// Returns the persisted login position (stored in `posx/posy/posz` DB columns).
    /// Mirrors C++ `getLoginPosition()`.
    pub fn get_login_position(&self) -> Position {
        self.login_position
    }

    /// Overwrite the persisted login position.
    /// Called by IOLoginData::loadPlayer when reading `posx/posy/posz`.
    /// If the stored position is (0,0,0) the C++ code falls back to the temple
    /// position; the same logic is exposed via `get_saved_position`.
    pub fn set_login_position(&mut self, pos: Position) {
        self.login_position = pos;
    }

    /// Returns the optional temporary (pre-teleport) position.
    pub fn get_temp_position(&self) -> Option<Position> {
        self.temp_position
    }

    /// Store a temporary teleport destination.  Cleared after teleport completes.
    pub fn set_temp_position(&mut self, pos: Position) {
        self.temp_position = Some(pos);
    }

    /// Clear the temporary position (after teleport has been applied).
    pub fn clear_temp_position(&mut self) {
        self.temp_position = None;
    }

    /// Mirrors C++ IOLoginData::loadPlayer logic:
    ///   - If `temp_position` is set, return it.
    ///   - Else if `login_position` is (0,0,0), return `temple_pos`.
    ///   - Else return `login_position`.
    pub fn get_saved_position(&self) -> Position {
        if let Some(temp) = self.temp_position {
            return temp;
        }
        if self.login_position.x == 0 && self.login_position.y == 0 && self.login_position.z == 0 {
            return self.temple_pos;
        }
        self.login_position
    }

    // -----------------------------------------------------------------------
    // (a) Stamina
    // -----------------------------------------------------------------------

    pub fn get_stamina(&self) -> u16 {
        self.stamina
    }

    /// Reduce stamina by `minutes`, clamped to 0.
    pub fn drain_stamina(&mut self, minutes: u16) {
        self.stamina = self.stamina.saturating_sub(minutes);
    }

    /// Increase stamina by `minutes`, clamped to STAMINA_MAX.
    pub fn recover_stamina(&mut self, minutes: u16) {
        self.stamina = (self.stamina + minutes).min(STAMINA_MAX);
    }

    /// Returns minutes of stamina above the premium threshold (2400).
    /// This is the "premium bonus zone".
    pub fn get_stamina_premium_minutes(&self) -> u16 {
        self.stamina.saturating_sub(STAMINA_PREMIUM_THRESHOLD)
    }

    // -----------------------------------------------------------------------
    // Runtime state — health, mana, experience
    // -----------------------------------------------------------------------

    pub fn get_health(&self) -> i32 {
        self.health
    }

    pub fn set_health(&mut self, hp: i32) {
        self.health = hp;
    }

    pub fn get_mana(&self) -> i32 {
        self.mana
    }

    pub fn set_mana(&mut self, mp: i32) {
        self.mana = mp;
    }

    pub fn set_max_health(&mut self, hp: i32) {
        self.max_health = hp;
    }

    pub fn set_max_mana(&mut self, mp: i32) {
        self.max_mana = mp;
    }

    pub fn get_experience(&self) -> u64 {
        self.experience
    }

    pub fn add_experience(&mut self, amount: u64) {
        self.experience = self.experience.saturating_add(amount);
    }

    // -----------------------------------------------------------------------
    // (a-2) Stamina recovery (offline)
    // -----------------------------------------------------------------------

    /// Apply stamina recovered while offline: 1 min per 3 min offline.
    pub fn apply_offline_stamina_recovery(&mut self, offline_seconds: u64) {
        let recovery = (offline_seconds / 180) as u16;
        self.recover_stamina(recovery);
    }

    // -----------------------------------------------------------------------
    // (b) Premium
    // -----------------------------------------------------------------------

    pub fn is_premium(&self) -> bool {
        self.premium
    }

    pub fn set_premium(&mut self, premium: bool) {
        self.premium = premium;
    }

    pub fn get_premium_days(&self) -> u32 {
        self.premium_days
    }

    pub fn add_premium_days(&mut self, n: u32) {
        self.premium_days += n;
    }

    // -----------------------------------------------------------------------
    // (c) Skull / emblem
    // -----------------------------------------------------------------------

    pub fn get_skull(&self) -> Skull {
        self.skull
    }

    pub fn set_skull(&mut self, skull: Skull) {
        self.skull = skull;
    }

    pub fn get_skull_ticks(&self) -> u32 {
        self.skull_ticks
    }

    pub fn set_skull_ticks(&mut self, ticks: u32) {
        self.skull_ticks = ticks;
    }

    // -----------------------------------------------------------------------
    // (d) Vocation stats
    // -----------------------------------------------------------------------

    pub fn get_level(&self) -> u32 {
        self.level
    }

    pub fn set_level(&mut self, level: u32) {
        self.level = level;
    }

    pub fn get_max_health(&self) -> i32 {
        self.max_health
    }

    pub fn get_max_mana(&self) -> i32 {
        self.max_mana
    }

    pub fn get_capacity(&self) -> u32 {
        self.capacity
    }

    pub fn set_capacity(&mut self, cap: u32) {
        self.capacity = cap;
    }

    /// Apply per-level stat gains (mirrors C++ addVocationSkillAdvance).
    pub fn apply_level_stats(&mut self, hp_gain: i32, mana_gain: i32, cap_gain: u32) {
        self.max_health += hp_gain;
        self.max_mana += mana_gain;
        self.capacity += cap_gain;
    }

    /// Returns the level percent (progress toward next level, 0-99).
    /// Mirrors C++ `getLevelPercent()`.
    pub fn get_level_percent(&self) -> u8 {
        self.level_percent
    }

    /// Set the level percent.
    pub fn set_level_percent(&mut self, pct: u8) {
        self.level_percent = pct.min(99);
    }

    /// Returns the current effective speed clamped to [PLAYER_MIN_SPEED, PLAYER_MAX_SPEED].
    /// Mirrors C++ `Player::getSpeed()` with the enforce-min/max guard.
    pub fn get_speed(&self) -> i32 {
        self.speed.clamp(PLAYER_MIN_SPEED, PLAYER_MAX_SPEED)
    }

    /// Set the movement speed (clamped to [PLAYER_MIN_SPEED, PLAYER_MAX_SPEED]).
    pub fn set_speed(&mut self, speed: i32) {
        self.speed = speed.clamp(PLAYER_MIN_SPEED, PLAYER_MAX_SPEED);
    }

    /// Apply a speed delta and clamp result to [PLAYER_MIN_SPEED, PLAYER_MAX_SPEED].
    /// Mirrors C++ game-level `changeSpeed` enforcement logic.
    pub fn change_speed(&mut self, delta: i32) {
        self.speed = (self.speed + delta).clamp(PLAYER_MIN_SPEED, PLAYER_MAX_SPEED);
    }

    /// Set base speed explicitly (e.g. on level-up), then enforce bounds.
    pub fn set_base_speed(&mut self, spd: i32) {
        self.speed = spd.clamp(PLAYER_MIN_SPEED, PLAYER_MAX_SPEED);
    }

    /// Returns the magic level (spell strength).
    /// Mirrors C++ `getBaseMagicLevel()`.
    pub fn get_magic_level(&self) -> u32 {
        self.magic_level
    }

    /// Set the magic level directly (e.g., loading from database).
    /// Resets `mana_spent` and `magic_level_percent` to 0.
    pub fn set_magic_level(&mut self, level: u32) {
        self.magic_level = level;
        self.mana_spent = 0;
        self.magic_level_percent = 0;
    }

    /// Returns mana spent (used to calculate magic level advancement).
    /// Mirrors C++ `getSpentMana()`.
    pub fn get_mana_spent(&self) -> u64 {
        self.mana_spent
    }

    /// Tries (mana) needed to advance from `magic_level` to the next magic level.
    ///
    /// Simplified baseline: `needed = 1600 * (magic_level + 1)^3`.
    fn magic_mana_needed(magic_level: u32) -> u64 {
        let next = (magic_level + 1) as u64;
        1600 * next * next * next
    }

    /// Add mana spent toward magic level advancement.
    /// Mirrors C++ `Player::addManaSpent`.
    ///
    /// Advances the magic level and resets `mana_spent` each time the threshold
    /// is crossed.  Updates `magic_level_percent` after each call.
    ///
    /// Returns `true` if the magic level advanced at least once.
    pub fn add_mana_spent_advance(&mut self, amount: u64) -> bool {
        if amount == 0 {
            return false;
        }
        let mut amount = amount;
        let mut advanced = false;
        loop {
            let needed = Self::magic_mana_needed(self.magic_level);
            if self.mana_spent + amount >= needed {
                amount -= needed - self.mana_spent;
                self.magic_level += 1;
                self.mana_spent = 0;
                self.magic_level_percent = 0;
                advanced = true;
            } else {
                self.mana_spent += amount;
                break;
            }
        }
        let needed = Self::magic_mana_needed(self.magic_level);
        if needed > 0 {
            self.magic_level_percent = ((self.mana_spent * 100 / needed).min(99)) as u16;
        }
        advanced
    }

    /// Recalculate magic-level percent from current `mana_spent` and `magic_level`.
    /// Equivalent to C++ `checkMagicLevel`.
    pub fn check_magic_level(&mut self) {
        // `magic_mana_needed(level)` = 1600 * (level + 1)^3, always >= 1600 (>0).
        let needed = Self::magic_mana_needed(self.magic_level);
        self.magic_level_percent = ((self.mana_spent * 100 / needed).min(99)) as u16;
    }

    /// Returns the magic level percent (progress toward next magic level, 0-99).
    pub fn get_magic_level_percent(&self) -> u16 {
        self.magic_level_percent
    }

    /// Set the magic level percent.
    pub fn set_magic_level_percent(&mut self, pct: u16) {
        self.magic_level_percent = pct.min(99);
    }

    /// Returns the condition immunity bitmask.
    /// Mirrors C++ `conditionImmunities` field.
    pub fn get_condition_immunities(&self) -> i32 {
        self.condition_immunities
    }

    /// Set the condition immunity bitmask.
    pub fn set_condition_immunities(&mut self, mask: i32) {
        self.condition_immunities = mask;
    }

    /// Returns true when the player is standing in a protection zone.
    pub fn is_in_protection_zone(&self) -> bool {
        self.in_protection_zone
    }

    /// Set whether the player is in a protection zone.
    pub fn set_in_protection_zone(&mut self, flag: bool) {
        self.in_protection_zone = flag;
    }

    /// Returns true when the player is actively engaged in combat.
    pub fn is_in_fight(&self) -> bool {
        self.in_fight
    }

    /// Set the combat engagement flag.
    pub fn set_in_fight(&mut self, flag: bool) {
        self.in_fight = flag;
    }

    // -----------------------------------------------------------------------
    // (a-3) Stamina XP multiplier
    // -----------------------------------------------------------------------

    /// Returns the XP multiplier for the current stamina level.
    ///
    /// - stamina == 0                                 -> 0.0 (no XP)
    /// - stamina <= STAMINA_EXHAUSTED_THRESHOLD (840) -> 0.5 (half XP)
    /// - stamina <= STAMINA_BONUS_ABOVE (2340)        -> 1.0 (normal XP)
    /// - stamina > STAMINA_BONUS_ABOVE AND premium    -> 1.5 (bonus XP)
    pub fn get_stamina_xp_multiplier(&self) -> f64 {
        if self.stamina == 0 {
            return 0.0;
        }
        if self.stamina <= STAMINA_EXHAUSTED_THRESHOLD {
            return 0.5;
        }
        if self.stamina > STAMINA_BONUS_ABOVE && self.premium {
            return 1.5;
        }
        1.0
    }

    // -----------------------------------------------------------------------
    // Health / mana regeneration tick
    // -----------------------------------------------------------------------

    /// Add `amount` HP, capped at `max_health`.  Mirrors C++ `addHPRegen`.
    pub fn add_hp_regen(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Add `amount` mana, capped at `max_mana`.  Mirrors C++ `addMPRegen`.
    pub fn add_mp_regen(&mut self, amount: i32) {
        self.mana = (self.mana + amount).min(self.max_mana);
    }

    // -----------------------------------------------------------------------
    // sendStats data shape
    // -----------------------------------------------------------------------

    /// Returns a snapshot of all fields that C++ `sendStats()` / `AddPlayerStats`
    /// transmits to the client.
    pub fn stats_snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            health: self.health,
            max_health: self.max_health,
            mana: self.mana,
            max_mana: self.max_mana,
            capacity: self.capacity,
            experience: self.experience,
            level: self.level,
            level_percent: self.level_percent,
            stamina: self.stamina,
            magic_level: self.magic_level,
            magic_level_percent: self.magic_level_percent,
            speed: self.get_speed(),
            soul: self.soul,
        }
    }

    // -----------------------------------------------------------------------
    // (e) Skill advance
    // -----------------------------------------------------------------------

    fn skill_index(skill: SkillType) -> usize {
        skill as usize
    }

    pub fn get_skill_level(&self, skill: SkillType) -> u16 {
        self.skills[Self::skill_index(skill)].level
    }

    pub fn get_skill_tries(&self, skill: SkillType) -> u64 {
        self.skills[Self::skill_index(skill)].tries
    }

    pub fn get_skill_percent(&self, skill: SkillType) -> u16 {
        self.skills[Self::skill_index(skill)].percent
    }

    /// Add `tries` to the skill, computing percent and advancing level when
    /// the threshold is met.
    ///
    /// Uses simplified formula: `tries_needed = 10 * level^3 * multiplier`
    /// (multiplier = 1.0 for MagicLevel; for combat skills use any multiplier ≥ 1.0).
    pub fn add_skill_tries(&mut self, skill: SkillType, tries: u64) {
        let idx = Self::skill_index(skill);
        self.skills[idx].tries += tries;
        self.update_skill_percent(skill);
    }

    /// Advance skill: compute level and percent from accumulated tries.
    /// Formula: tries_needed(level) = 10 * level^3 (multiplier=1.0 baseline).
    fn tries_needed(level: u16) -> u64 {
        10u64 * (level as u64).pow(3)
    }

    fn update_skill_percent(&mut self, skill: SkillType) {
        let idx = Self::skill_index(skill);
        let current_level = self.skills[idx].level;
        let needed = Self::tries_needed(current_level);
        // `current_level` is always >= MINIMUM_SKILL_LEVEL (>=10) by construction
        // (see Skill::default and set_skill_level), so `needed` is always >= 10_000;
        // no zero-divisor guard is required.
        // Check for level advance
        if self.skills[idx].tries >= needed {
            self.skills[idx].tries -= needed;
            self.skills[idx].level += 1;
            // Recalculate percent at new level
            self.update_skill_percent(skill);
        } else {
            let percent = (self.skills[idx].tries * 100 / needed).min(99) as u16;
            self.skills[idx].percent = percent;
        }
    }

    /// Set skill level directly (e.g., when loading from database).
    pub fn set_skill_level(&mut self, skill: SkillType, level: u16) {
        let idx = Self::skill_index(skill);
        self.skills[idx].level = level.max(MINIMUM_SKILL_LEVEL);
        self.skills[idx].tries = 0;
        self.skills[idx].percent = 0;
    }

    // -----------------------------------------------------------------------
    // (e-2) Death — skill loss, item loss
    // -----------------------------------------------------------------------

    /// Returns how many levels to remove from each skill (1% of current level).
    pub fn compute_skill_loss(&self) -> [u16; SKILL_COUNT] {
        let mut loss = [0u16; SKILL_COUNT];
        for (i, entry) in loss.iter_mut().enumerate() {
            let level = self.skills[i].level;
            if level > MINIMUM_SKILL_LEVEL {
                let raw = ((level as f32 * 0.01).ceil() as u16).max(1);
                *entry = raw.min(level - MINIMUM_SKILL_LEVEL);
            }
        }
        loss
    }

    /// Apply pre-computed skill loss; resets tries/percent to 0.
    pub fn apply_skill_loss(&mut self, loss: [u16; SKILL_COUNT]) {
        for (i, &reduction) in loss.iter().enumerate() {
            self.skills[i].level = self.skills[i]
                .level
                .saturating_sub(reduction)
                .max(MINIMUM_SKILL_LEVEL);
            self.skills[i].tries = 0;
            self.skills[i].percent = 0;
        }
    }

    /// Returns inventory slots that should be dropped on death (based on blessings).
    /// With 5 blessings or amulet of loss: no items dropped (simplified).
    pub fn compute_item_loss(&self) -> Vec<InventorySlot> {
        if self.get_blessing_count() >= 5 {
            return vec![];
        }
        // Simplified: drop visible body slots when unblessed
        vec![
            InventorySlot::Head,
            InventorySlot::Armor,
            InventorySlot::Legs,
            InventorySlot::Feet,
            InventorySlot::Right,
            InventorySlot::Left,
        ]
    }

    /// Remove and return items from the given slots.
    pub fn drop_items(&mut self, slots: Vec<InventorySlot>) -> Vec<Item> {
        slots
            .into_iter()
            .filter_map(|slot| self.remove_inventory_item(slot))
            .collect()
    }

    // -----------------------------------------------------------------------
    // (d-2) Level-up
    // -----------------------------------------------------------------------

    /// Check whether accumulated experience crosses the next level threshold.
    /// Returns the new level if the player advanced, otherwise `None`.
    pub fn check_level_up(&mut self) -> Option<u32> {
        let initial = self.level;
        while self.experience >= xp_for_level(self.level + 1) {
            self.level += 1;
            self.max_health += 5;
            self.max_mana += 5;
        }
        if self.level > initial {
            Some(self.level)
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------
    // (f) Offline training
    // -----------------------------------------------------------------------

    pub fn get_offline_training_skill(&self) -> OfflineTrainingSkill {
        self.offline_training_skill
    }

    pub fn set_offline_training_skill(&mut self, skill: OfflineTrainingSkill) {
        self.offline_training_skill = skill;
    }

    pub fn get_offline_training_time(&self) -> i32 {
        self.offline_training_time
    }

    pub fn set_offline_training_time(&mut self, n: i32) {
        self.offline_training_time = n;
    }

    // -----------------------------------------------------------------------
    // (g) Inventory slots
    // -----------------------------------------------------------------------

    fn slot_index(slot: InventorySlot) -> usize {
        (slot as usize) - 1 // slots 1..=11 → index 0..=10
    }

    pub fn get_inventory_item(&self, slot: InventorySlot) -> Option<&Item> {
        self.inventory[Self::slot_index(slot)].as_ref()
    }

    pub fn set_inventory_item(&mut self, slot: InventorySlot, item: Item) {
        self.inventory[Self::slot_index(slot)] = Some(item);
    }

    /// Remove and return the item in the given slot.
    pub fn remove_inventory_item(&mut self, slot: InventorySlot) -> Option<Item> {
        self.inventory[Self::slot_index(slot)].take()
    }

    /// Returns total weight of all inventory items. Also exposed as `get_carry_weight`.
    pub fn get_inventory_weight(&self) -> u32 {
        self.inventory
            .iter()
            .filter_map(|opt| opt.as_ref())
            .map(|item| item.get_weight())
            .sum()
    }

    /// Mirrors C++ `getCarryWeight()` — alias for `get_inventory_weight`.
    pub fn get_carry_weight(&self) -> u32 {
        self.get_inventory_weight()
    }

    /// True iff `extra_weight` fits within remaining capacity.
    pub fn has_extra_capacity(&self, extra_weight: u32) -> bool {
        self.get_inventory_weight().saturating_add(extra_weight) <= self.capacity
    }

    /// Place `item` in `slot`; returns the displaced item if any.
    pub fn add_item_to_inventory(&mut self, slot: InventorySlot, item: Item) -> Option<Item> {
        let idx = Self::slot_index(slot);
        let old = self.inventory[idx].take();
        self.inventory[idx] = Some(item);
        old
    }

    /// Remove and return item from `slot` (None if empty).
    pub fn remove_item_from_inventory(&mut self, slot: InventorySlot) -> Option<Item> {
        self.remove_inventory_item(slot)
    }

    fn is_melee_weapon(w: WeaponType) -> bool {
        matches!(
            w,
            WeaponType::Sword | WeaponType::Club | WeaponType::Axe | WeaponType::Wand
        )
    }

    /// Cylinder slot validation — mirrors C++ `Player::queryAdd`.
    pub fn query_add(
        &self,
        slot: InventorySlot,
        item: &Item,
        count: u8,
        flags: u32,
    ) -> ReturnValue {
        use query_flags::*;
        if flags & CHILD_IS_OWNER != 0 {
            return if flags & NO_LIMIT != 0 || self.has_extra_capacity(item.get_weight()) {
                ReturnValue::NoError
            } else {
                ReturnValue::NotEnoughCapacity
            };
        }
        if !item.is_pickupable() {
            return ReturnValue::CannotPickup;
        }
        if item.is_store_item() {
            return ReturnValue::ItemCannotBeMovedThere;
        }

        let sp = item.get_slot_position();
        let wt = item.get_weapon_type();

        let slot_ok = match slot {
            InventorySlot::Head => sp & slot_position::HEAD != 0,
            InventorySlot::Necklace => sp & slot_position::NECKLACE != 0,
            InventorySlot::Backpack => sp & slot_position::BACKPACK != 0,
            InventorySlot::Armor => sp & slot_position::ARMOR != 0,
            InventorySlot::Legs => sp & slot_position::LEGS != 0,
            InventorySlot::Feet => sp & slot_position::FEET != 0,
            InventorySlot::Ring => sp & slot_position::RING != 0,
            InventorySlot::Ammo => sp & slot_position::AMMO != 0,
            InventorySlot::Depot => true,
            InventorySlot::Right => {
                if sp & slot_position::RIGHT == 0 {
                    return ReturnValue::CannotBeDressed;
                }
                let li_opt = self.get_inventory_item(InventorySlot::Left);
                if sp & slot_position::TWO_HAND != 0 {
                    if let Some(li) = li_opt {
                        if !std::ptr::eq(li as *const Item, item as *const Item) {
                            return ReturnValue::BothHandsNeedToBeFree;
                        }
                    }
                    return ReturnValue::NoError;
                }
                if let Some(li) = li_opt {
                    let lw = li.get_weapon_type();
                    if li.get_slot_position() & slot_position::TWO_HAND != 0 {
                        if lw == WeaponType::Distance && wt == WeaponType::Quiver {
                            return ReturnValue::NoError;
                        }
                        return ReturnValue::DropTwoHandedItem;
                    }
                    if lw == WeaponType::Shield && wt == WeaponType::Shield {
                        return ReturnValue::CanOnlyUseOneShield;
                    }
                    if Self::is_melee_weapon(wt) && Self::is_melee_weapon(lw) {
                        return ReturnValue::CanOnlyUseOneWeapon;
                    }
                }
                true
            }
            InventorySlot::Left => {
                if sp & slot_position::LEFT == 0 {
                    return ReturnValue::CannotBeDressed;
                }
                let ri_opt = self.get_inventory_item(InventorySlot::Right);
                if sp & slot_position::TWO_HAND != 0 {
                    return match ri_opt {
                        Some(ri) if !std::ptr::eq(ri as *const Item, item as *const Item) => {
                            let rw = ri.get_weapon_type();
                            if wt == WeaponType::Distance && rw == WeaponType::Quiver {
                                ReturnValue::NoError
                            } else {
                                ReturnValue::BothHandsNeedToBeFree
                            }
                        }
                        _ => ReturnValue::NoError,
                    };
                }
                match ri_opt {
                    Some(ri) => {
                        let rw = ri.get_weapon_type();
                        if ri.get_slot_position() & slot_position::TWO_HAND != 0 {
                            if wt == WeaponType::Distance && rw == WeaponType::Quiver {
                                return ReturnValue::NoError;
                            }
                            return ReturnValue::DropTwoHandedItem;
                        }
                        if rw == WeaponType::Shield && wt == WeaponType::Shield {
                            return ReturnValue::CanOnlyUseOneShield;
                        }
                        if Self::is_melee_weapon(wt) && Self::is_melee_weapon(rw) {
                            return ReturnValue::CanOnlyUseOneWeapon;
                        }
                        true
                    }
                    None => true,
                }
            }
        };
        if !slot_ok {
            return ReturnValue::CannotBeDressed;
        }
        let item_weight = if item.is_stackable() {
            item.get_base_weight() * count as u32
        } else {
            item.get_base_weight()
        };
        if !self.can_carry_weight(item_weight, self.get_inventory_weight()) {
            return ReturnValue::NotEnoughCapacity;
        }
        ReturnValue::NoError
    }

    /// Cylinder remove validation — mirrors C++ `Player::queryRemove`.
    pub fn query_remove(&self, slot: InventorySlot, count: u8, flags: u32) -> ReturnValue {
        let item = match self.get_inventory_item(slot) {
            Some(i) => i,
            None => return ReturnValue::NotPossible,
        };
        if count == 0 {
            return ReturnValue::NotPossible;
        }
        if item.is_stackable() && count > item.get_item_count() {
            return ReturnValue::NotPossible;
        }
        if !item.is_moveable() && (flags & query_flags::IGNORE_NOT_MOVEABLE == 0) {
            return ReturnValue::NotMoveable;
        }
        ReturnValue::NoError
    }

    /// Mirrors C++ `postAddNotification`. Data-layer only; no side effects.
    pub fn post_add_notification(&self, _slot: InventorySlot) {}

    /// Mirrors C++ `postRemoveNotification`. Data-layer only; no side effects.
    pub fn post_remove_notification(&self, _slot: InventorySlot) {}

    // -----------------------------------------------------------------------
    // (h) Depot
    // -----------------------------------------------------------------------

    pub fn get_max_depot_items(&self) -> u32 {
        self.max_depot_items
    }

    pub fn set_max_depot_items(&mut self, n: u32) {
        self.max_depot_items = n;
    }

    /// Add an item to the depot chest identified by `depot_id`.
    /// Mirrors the C++ IOLoginData::loadPlayer depot-loading loop:
    ///   `depotChest->internalAddThing(item)`.
    pub fn add_depot_item(&mut self, depot_id: u32, item: Item) {
        self.depot_items.entry(depot_id).or_default().push(item);
    }

    /// Return all items in the given depot chest, or an empty slice if absent.
    pub fn get_depot_items(&self, depot_id: u32) -> &[Item] {
        self.depot_items
            .get(&depot_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Remove and return all items for `depot_id`.
    pub fn take_depot_items(&mut self, depot_id: u32) -> Vec<Item> {
        self.depot_items.remove(&depot_id).unwrap_or_default()
    }

    /// Returns the set of depot_ids that have been loaded.
    pub fn depot_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.depot_items.keys().copied()
    }

    // -----------------------------------------------------------------------
    // (i) Party state machine
    // -----------------------------------------------------------------------

    pub fn get_party_id(&self) -> Option<u32> {
        self.party_id
    }

    pub fn set_party_id(&mut self, party_id: Option<u32>) {
        self.party_id = party_id;
    }

    pub fn leave_party(&mut self) {
        self.party_id = None;
    }

    // -----------------------------------------------------------------------
    // (j) Trade state machine
    // -----------------------------------------------------------------------

    pub fn get_trade_state(&self) -> TradeState {
        self.trade_state
    }

    pub fn set_trade_state(&mut self, state: TradeState) {
        self.trade_state = state;
    }

    pub fn cancel_trade(&mut self) {
        self.trade_state = TradeState::None;
    }

    pub fn get_trade_partner_guid(&self) -> Option<u32> {
        self.trade_partner_guid
    }

    pub fn set_trade_partner_guid(&mut self, guid: u32) {
        self.trade_partner_guid = Some(guid);
    }

    // -----------------------------------------------------------------------
    // (k) Weight / capacity
    // -----------------------------------------------------------------------

    /// Returns free capacity (capacity - current_weight, clamped to 0).
    pub fn get_free_capacity(&self, current_weight: u32) -> u32 {
        self.capacity.saturating_sub(current_weight)
    }

    /// Returns true iff current_weight + extra <= capacity.
    pub fn can_carry_weight(&self, extra: u32, current_weight: u32) -> bool {
        current_weight.saturating_add(extra) <= self.capacity
    }

    // -----------------------------------------------------------------------
    // (l) VIP list
    // -----------------------------------------------------------------------

    pub fn add_vip(&mut self, entry: VIPEntry) {
        self.vip_list.push(entry);
    }

    pub fn is_in_vip_list(&self, guid: u32) -> bool {
        self.vip_list.iter().any(|e| e.guid == guid)
    }

    pub fn remove_vip(&mut self, guid: u32) {
        self.vip_list.retain(|e| e.guid != guid);
    }

    pub fn get_vip_count(&self) -> usize {
        self.vip_list.len()
    }

    pub fn get_max_vip_entries(&self) -> u32 {
        self.max_vip_entries
    }

    pub fn set_max_vip_entries(&mut self, n: u32) {
        self.max_vip_entries = n;
    }

    // -----------------------------------------------------------------------
    // (m) Blessings
    // -----------------------------------------------------------------------

    /// Returns true if blessing `n` (0..=4) is active.
    pub fn has_blessing(&self, n: u8) -> bool {
        n < 5 && (self.blessings & (1 << n)) != 0
    }

    /// Activate blessing `n` (0..=4).
    pub fn add_blessing(&mut self, n: u8) {
        if n < 5 {
            self.blessings |= 1 << n;
        }
    }

    /// Returns number of active blessings.
    pub fn get_blessing_count(&self) -> u8 {
        self.blessings.count_ones() as u8
    }

    /// XP loss percent for this player.
    ///
    /// Base = 10%. Each blessing reduces by 8%. Clamped to [0, 100].
    /// `has_amulet_of_loss` reduces remaining loss to 0 (full protection).
    pub fn get_xp_loss_percent(&self, has_amulet_of_loss: bool) -> u8 {
        let base: i32 = 10;
        let blessing_reduction: i32 = 8 * (self.get_blessing_count() as i32);
        let loss = (base - blessing_reduction).max(0);
        if has_amulet_of_loss {
            return 0;
        }
        loss as u8
    }

    /// Return the raw blessings bitmask byte.
    /// Mirrors C++ `player->blessings.to_ulong()` used in savePlayer SQL.
    /// Only bits 0–4 are meaningful (5 blessings).
    pub fn get_blessings_byte(&self) -> u8 {
        self.blessings
    }

    /// Restore the blessings bitmask from the value read out of the database.
    /// Mirrors C++ `player->blessings = result->getNumber<uint16_t>("blessings")`.
    /// Bits above 4 are masked off so out-of-range DB values cannot set
    /// non-existent blessings.
    pub fn set_blessings_from_byte(&mut self, byte: u8) {
        self.blessings = byte & 0b0001_1111; // keep only bits 0-4
    }

    /// Remove blessing `n` (0..=4).
    /// Mirrors C++ `blessings.reset(blessing)`.
    pub fn remove_blessing(&mut self, n: u8) {
        if n < 5 {
            self.blessings &= !(1 << n);
        }
    }

    // -----------------------------------------------------------------------
    // (n) Murder count / red skull
    // -----------------------------------------------------------------------

    pub fn get_unjustified_kills(&self) -> u32 {
        self.unjustified_kills
    }

    pub fn add_unjustified_kill(&mut self) {
        self.unjustified_kills += 1;
    }

    /// Returns true if the player has at least 1 unjustified kill.
    pub fn is_killer(&self) -> bool {
        self.unjustified_kills >= 1
    }

    // -----------------------------------------------------------------------
    // (o) Soul points
    // -----------------------------------------------------------------------

    pub fn get_soul(&self) -> u8 {
        self.soul
    }

    pub fn get_max_soul(&self) -> u8 {
        self.soul_max
    }

    pub fn set_soul(&mut self, n: u8) {
        self.soul = n;
    }

    /// Reduce soul by `n`, clamped to 0.
    pub fn consume_soul(&mut self, n: u8) {
        self.soul = self.soul.saturating_sub(n);
    }

    // -----------------------------------------------------------------------
    // (p) Combat modes
    // -----------------------------------------------------------------------

    pub fn get_safe_mode(&self) -> bool {
        self.safe_mode
    }

    pub fn set_safe_mode(&mut self, mode: bool) {
        self.safe_mode = mode;
    }

    pub fn get_fight_mode(&self) -> FightMode {
        self.fight_mode
    }

    pub fn set_fight_mode(&mut self, mode: FightMode) {
        self.fight_mode = mode;
    }

    pub fn get_pvp_mode(&self) -> PvpMode {
        self.pvp_mode
    }

    pub fn set_pvp_mode(&mut self, mode: PvpMode) {
        self.pvp_mode = mode;
    }

    // -----------------------------------------------------------------------
    // (8.5-a) Friend list
    // Mirrors the player social-contact list found in OT server forks.
    // add_friend: no-op on duplicate, returns false; capped at max_friends.
    // remove_friend: no-op if not present.
    // -----------------------------------------------------------------------

    /// Returns the current friend list (slice of GUIDs).
    pub fn get_friend_list(&self) -> &[u32] {
        &self.friend_list
    }

    /// Returns the maximum number of friends allowed.
    pub fn get_max_friends(&self) -> u32 {
        self.max_friends
    }

    /// Set the maximum number of friends.
    pub fn set_max_friends(&mut self, max: u32) {
        self.max_friends = max;
    }

    /// Add `guid` to the friend list.
    ///
    /// Returns `true` on success.
    /// Returns `false` if:
    /// - `guid` is already in the list (duplicate check), or
    /// - the list has reached `max_friends`.
    pub fn add_friend(&mut self, guid: u32) -> bool {
        if self.friend_list.contains(&guid) {
            return false;
        }
        if self.friend_list.len() as u32 >= self.max_friends {
            return false;
        }
        self.friend_list.push(guid);
        true
    }

    /// Remove `guid` from the friend list.
    /// Returns `true` if the guid was present and removed, `false` otherwise.
    pub fn remove_friend(&mut self, guid: u32) -> bool {
        let before = self.friend_list.len();
        self.friend_list.retain(|&g| g != guid);
        self.friend_list.len() < before
    }

    /// Returns true if `guid` is in the friend list.
    pub fn is_friend(&self, guid: u32) -> bool {
        self.friend_list.contains(&guid)
    }

    // -----------------------------------------------------------------------
    // (8.5-b) Guild membership
    // Mirrors C++ `isGuildMate()` and `guild` pointer.
    // A player is a guild member when guild_id is Some(_).
    // -----------------------------------------------------------------------

    /// Returns true when the player belongs to a guild.
    /// Mirrors C++ `guild != nullptr` check in `isGuildMate()`.
    pub fn is_guild_member(&self) -> bool {
        self.guild_id.is_some()
    }

    /// Returns the guild ID if the player belongs to one.
    pub fn get_guild_id(&self) -> Option<u32> {
        self.guild_id
    }

    /// Assign the player to a guild.
    pub fn set_guild_id(&mut self, id: u32) {
        self.guild_id = Some(id);
    }

    /// Remove the player from their guild.
    pub fn leave_guild(&mut self) {
        self.guild_id = None;
        self.guild_nick.clear();
    }

    /// Returns the guild nick (in-game title within the guild).
    /// Mirrors C++ `getGuildNick()`.
    pub fn get_guild_nick(&self) -> &str {
        &self.guild_nick
    }

    /// Set the guild nick.
    /// Mirrors C++ `setGuildNick(nick)`.
    pub fn set_guild_nick(&mut self, nick: impl Into<String>) {
        self.guild_nick = nick.into();
    }

    // -----------------------------------------------------------------------
    // (8.5-c) Party invitations (as seen from the invited player's side)
    // Mirrors C++ `addPartyInvitation` / `removePartyInvitation` on Player.
    // We track the inviting party leader's GUID.
    // -----------------------------------------------------------------------

    /// Returns true if a party invitation from `leader_guid` is pending.
    pub fn has_party_invite_from(&self, leader_guid: u32) -> bool {
        self.pending_party_invites.contains(&leader_guid)
    }

    /// Record an incoming party invitation from `leader_guid`.
    /// Returns `false` if the invitation is already present (duplicate guard,
    /// mirrors C++ `if (it != invitePartyList.end()) { return false; }`).
    pub fn add_party_invite(&mut self, leader_guid: u32) -> bool {
        if self.pending_party_invites.contains(&leader_guid) {
            return false;
        }
        self.pending_party_invites.push(leader_guid);
        true
    }

    /// Remove the party invitation from `leader_guid`.
    /// Mirrors C++ `removePartyInvitation(party)`.
    pub fn remove_party_invite(&mut self, leader_guid: u32) {
        self.pending_party_invites.retain(|&g| g != leader_guid);
    }

    /// Accept a party invitation: join the party identified by `leader_guid`.
    /// Removes the invitation and sets party_id to `leader_guid`.
    /// Returns `false` if no such invitation exists.
    ///
    /// Mirrors C++ `Party::joinParty(Player&)` side-effects on the player.
    pub fn join_party(&mut self, leader_guid: u32) -> bool {
        if !self.pending_party_invites.contains(&leader_guid) {
            return false;
        }
        self.pending_party_invites.retain(|&g| g != leader_guid);
        self.party_id = Some(leader_guid);
        true
    }

    /// Clear all pending party invitations.
    /// Mirrors C++ `clearPartyInvitations()`.
    pub fn clear_party_invites(&mut self) {
        self.pending_party_invites.clear();
    }

    // -----------------------------------------------------------------------
    // (8.5-d) Channel subscriptions
    // Mirrors the set of chat channels a player has open.
    // -----------------------------------------------------------------------

    /// Returns all subscribed channel IDs.
    pub fn get_subscribed_channels(&self) -> &[u16] {
        &self.subscribed_channels
    }

    /// Subscribe to a channel.  No-op if already subscribed.
    pub fn add_channel(&mut self, channel_id: u16) {
        if !self.subscribed_channels.contains(&channel_id) {
            self.subscribed_channels.push(channel_id);
        }
    }

    /// Unsubscribe from a channel.
    pub fn remove_channel(&mut self, channel_id: u16) {
        self.subscribed_channels.retain(|&id| id != channel_id);
    }

    /// Returns true if the player is subscribed to `channel_id`.
    pub fn is_subscribed_to_channel(&self, channel_id: u16) -> bool {
        self.subscribed_channels.contains(&channel_id)
    }

    // -----------------------------------------------------------------------
    // (8.5-e) Client version
    // Mirrors C++ `getProtocolVersion()`.
    // -----------------------------------------------------------------------

    /// Returns the client version reported at login.
    pub fn get_client_version(&self) -> u16 {
        self.client_version
    }

    /// Set the client version (called during login handshake).
    pub fn set_client_version(&mut self, version: u16) {
        self.client_version = version;
    }

    /// Returns true when the client version falls within the given inclusive range.
    /// Mirrors the version-check pattern in C++ protocolGame / config-driven
    /// min/max version checks.
    pub fn is_client_version_compatible(&self, min: u16, max: u16) -> bool {
        self.client_version >= min && self.client_version <= max
    }

    // -----------------------------------------------------------------------
    // (8.5-f) Key-value storage
    // Mirrors C++ `Creature::setStorageValue` / `getStorageValue`.
    // -----------------------------------------------------------------------

    /// Set a storage value for `key`.
    /// Passing `None` removes the key (mirrors C++ `storageMap.erase(key)`).
    ///
    /// Returns `false` and does nothing if `key` falls in the reserved range
    /// (`0x10_000_000..=0x1F_FF_FF_FF`), mirroring the C++ Player override that
    /// warns and returns early for reserved keys.
    pub fn set_storage_value(&mut self, key: u32, value: Option<i32>) -> bool {
        // Mirror C++ IS_IN_KEYRANGE(key, RESERVED_RANGE) check.
        // Reserved range: 0x10000000 <= key <= 0x1FFFFFFF (C++ macro).
        if (0x1000_0000..=0x1FFF_FFFF).contains(&key) {
            return false;
        }
        match value {
            Some(v) => {
                self.storage.insert(key, v);
            }
            None => {
                self.storage.remove(&key);
            }
        }
        true
    }

    /// Get the storage value for `key`, returning `None` if not set.
    /// Mirrors C++ `getStorageValue(key)`.
    pub fn get_storage_value(&self, key: u32) -> Option<i32> {
        self.storage.get(&key).copied()
    }

    /// Returns an iterator over all (key, value) storage pairs.
    pub fn storage_iter(&self) -> impl Iterator<Item = (u32, i32)> + '_ {
        self.storage.iter().map(|(&k, &v)| (k, v))
    }

    // -----------------------------------------------------------------------
    // (q) Task 8.3 — Combat: weapon type, attack speed, armor, raw defense
    // -----------------------------------------------------------------------

    /// Weapon type of the right-hand slot item (or None).
    /// Mirrors C++ `Player::getWeaponType()`.
    pub fn get_weapon_type(&self) -> WeaponType {
        match self.get_inventory_item(InventorySlot::Right) {
            Some(item) => item.get_weapon_type(),
            None => WeaponType::None,
        }
    }

    /// Effective attack speed: weapon speed if non-zero, else vocation default.
    /// Mirrors C++ `Player::getAttackSpeed()`.
    pub fn get_attack_speed(&self, vocation_attack_speed: u32) -> u32 {
        match self.get_inventory_item(InventorySlot::Right) {
            Some(item) => {
                let ws = item.get_attack_speed();
                if ws == 0 {
                    vocation_attack_speed
                } else {
                    ws
                }
            }
            None => vocation_attack_speed,
        }
    }

    /// Total armor from armor slots scaled by vocation `armor_multiplier`.
    /// Mirrors C++ `Player::getArmor()`.
    pub fn get_armor(&self, armor_multiplier: f32) -> i32 {
        const ARMOR_SLOTS: [InventorySlot; 6] = [
            InventorySlot::Head,
            InventorySlot::Necklace,
            InventorySlot::Armor,
            InventorySlot::Legs,
            InventorySlot::Feet,
            InventorySlot::Ring,
        ];
        let raw: i32 = ARMOR_SLOTS
            .iter()
            .filter_map(|&s| self.get_inventory_item(s))
            .map(|i| i.get_armor())
            .sum();
        (raw as f32 * armor_multiplier) as i32
    }

    /// Raw `(defense_value, has_shield)` from equipped shield/weapon.
    /// Callers apply defense-skill / fight-mode multipliers separately.
    /// Mirrors C++ `Player::getDefense()` shield/weapon inspection.
    pub fn get_raw_defense(&self) -> (i32, bool) {
        let shield = self
            .get_inventory_item(InventorySlot::Left)
            .filter(|i| matches!(i.get_weapon_type(), WeaponType::Shield | WeaponType::Quiver));
        let weapon = self.get_inventory_item(InventorySlot::Right).filter(|i| {
            !matches!(
                i.get_weapon_type(),
                WeaponType::None | WeaponType::Shield | WeaponType::Quiver
            )
        });

        match (shield, weapon) {
            (Some(s), Some(w)) => (s.get_defense() + w.get_extra_defense(), true),
            (Some(s), None) => (s.get_defense(), true),
            (None, Some(w)) => (w.get_defense() + w.get_extra_defense(), false),
            (None, None) => (7, false), // unarmed baseline
        }
    }

    // -----------------------------------------------------------------------
    // (q-2) Task 8.3 — on_attacking / on_attacked hooks
    // -----------------------------------------------------------------------

    /// Called when the player deals damage. Sets the in-fight flag.
    pub fn on_attacking(&mut self) {
        self.in_fight = true;
    }

    /// Called when the player receives damage. Sets the in-fight flag.
    pub fn on_attacked(&mut self) {
        self.in_fight = true;
    }

    /// Clear the in-fight flag.
    pub fn clear_in_fight(&mut self) {
        self.in_fight = false;
    }

    // -----------------------------------------------------------------------
    // (q-3) Task 8.3 — Condition immunity
    // -----------------------------------------------------------------------

    /// True if immune to `condition_flag` (a ConditionTypeFlags bit).
    /// Mirrors C++ `Player::isImmune(ConditionType_t)`.
    pub fn is_immune_to_condition(&self, condition_flag: i32) -> bool {
        self.condition_immunities & condition_flag != 0
    }

    /// Grant immunity to one or more condition types.
    pub fn add_condition_immunity(&mut self, condition_flag: i32) {
        self.condition_immunities |= condition_flag;
    }

    /// Remove immunity bits.
    pub fn remove_condition_immunity(&mut self, condition_flag: i32) {
        self.condition_immunities &= !condition_flag;
    }

    // -----------------------------------------------------------------------
    // (q-4) Task 8.3 — Skull upgrade via add_unjustified_points
    // -----------------------------------------------------------------------

    /// Record an unjustified kill and escalate skull if thresholds are met.
    /// Mirrors C++ `Player::addUnjustifiedDead`.
    pub fn add_unjustified_points(
        &mut self,
        frag_time_ms: u32,
        kills_to_red: u32,
        kills_to_black: u32,
    ) {
        self.unjustified_kills += 1;
        self.skull_ticks = self.skull_ticks.saturating_add(frag_time_ms);

        if self.skull != Skull::Black {
            if kills_to_black != 0
                && self.skull_ticks
                    > kills_to_black
                        .saturating_sub(1)
                        .saturating_mul(frag_time_ms)
            {
                self.skull = Skull::Black;
            } else if self.skull != Skull::Red
                && kills_to_red != 0
                && self.skull_ticks > kills_to_red.saturating_sub(1).saturating_mul(frag_time_ms)
            {
                self.skull = Skull::Red;
            }
        }
    }

    // -----------------------------------------------------------------------
    // (q-5) Task 8.3 — Protection zone alias
    // -----------------------------------------------------------------------

    /// True if current tile is a protection zone.
    /// Mirrors `tile && tile->hasFlag(TILESTATE_PROTECTIONZONE)`.
    pub fn is_protection_zone(&self) -> bool {
        self.in_protection_zone
    }

    /// Set the protection-zone flag.
    pub fn set_protection_zone(&mut self, pz: bool) {
        self.in_protection_zone = pz;
    }
}

// ---------------------------------------------------------------------------
// PlayerAttr — binary serialization of player attribute blobs (task 8.4)
//
// The format is a tagged byte stream:
//   [tag: u8][payload bytes] ...  terminated by PlayerAttr::End (0xFF).
// ---------------------------------------------------------------------------

/// Tag bytes used in the player attribute blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerAttr {
    Blessings = 0x01,
    LoginPosition = 0x02,
    VipList = 0x03,
    End = 0xFF,
}

/// Errors during attribute serialization / deserialization.
#[derive(Debug, PartialEq, Eq)]
pub enum AttrError {
    UnexpectedEof,
    UnknownTag(u8),
    InvalidStringLength,
}

impl std::fmt::Display for AttrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttrError::UnexpectedEof => write!(f, "unexpected end of attribute stream"),
            AttrError::UnknownTag(t) => write!(f, "unknown attribute tag: {:#04x}", t),
            AttrError::InvalidStringLength => {
                write!(f, "invalid string length in attribute stream")
            }
        }
    }
}

impl VIPEntry {
    /// Encode this VIP entry into bytes.
    /// Layout: guid(u32 LE) | icon(u32 LE) | notify(u8)
    ///         | name_len(u16 LE) | name bytes
    ///         | desc_len(u16 LE) | description bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let name_bytes = self.name.as_bytes();
        let desc_bytes = self.description.as_bytes();
        let mut buf = Vec::with_capacity(4 + 4 + 1 + 2 + name_bytes.len() + 2 + desc_bytes.len());
        buf.extend_from_slice(&self.guid.to_le_bytes());
        buf.extend_from_slice(&self.icon.to_le_bytes());
        buf.push(self.notify as u8);
        buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&(desc_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(desc_bytes);
        buf
    }

    /// Decode a VIP entry from `bytes` starting at `offset`.
    /// Returns `(entry, bytes_consumed)` or `AttrError`.
    pub fn from_bytes(bytes: &[u8], offset: usize) -> Result<(Self, usize), AttrError> {
        let mut pos = offset;
        let guid = read_u32_le(bytes, &mut pos)?;
        let icon = read_u32_le(bytes, &mut pos)?;
        let notify_byte = read_u8(bytes, &mut pos)?;
        let notify = notify_byte != 0;
        let name = read_string(bytes, &mut pos)?;
        let description = read_string(bytes, &mut pos)?;
        Ok((
            VIPEntry {
                guid,
                name,
                description,
                icon,
                notify,
            },
            pos - offset,
        ))
    }
}

impl Player {
    /// Serialize the player's persistence attributes into a compact byte blob.
    /// Encodes blessings, login_position, VIP list, and the End sentinel.
    pub fn serialize_attr(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.push(PlayerAttr::Blessings as u8);
        buf.push(self.blessings);

        buf.push(PlayerAttr::LoginPosition as u8);
        buf.extend_from_slice(&self.login_position.x.to_le_bytes());
        buf.extend_from_slice(&self.login_position.y.to_le_bytes());
        buf.push(self.login_position.z);

        buf.push(PlayerAttr::VipList as u8);
        buf.extend_from_slice(&(self.vip_list.len() as u32).to_le_bytes());
        for entry in &self.vip_list {
            buf.extend_from_slice(&entry.to_bytes());
        }

        buf.push(PlayerAttr::End as u8);
        buf
    }

    /// Deserialize a player attribute blob produced by `serialize_attr`.
    pub fn unserialize_attr(&mut self, data: &[u8]) -> Result<(), AttrError> {
        let mut pos = 0usize;
        loop {
            let tag = read_u8(data, &mut pos)?;
            if tag == PlayerAttr::End as u8 {
                break;
            } else if tag == PlayerAttr::Blessings as u8 {
                let b = read_u8(data, &mut pos)?;
                self.set_blessings_from_byte(b);
            } else if tag == PlayerAttr::LoginPosition as u8 {
                let x = read_u16_le(data, &mut pos)?;
                let y = read_u16_le(data, &mut pos)?;
                let z = read_u8(data, &mut pos)?;
                self.login_position = Position { x, y, z };
            } else if tag == PlayerAttr::VipList as u8 {
                let count = read_u32_le(data, &mut pos)?;
                self.vip_list.clear();
                for _ in 0..count {
                    let (entry, consumed) = VIPEntry::from_bytes(data, pos)?;
                    pos += consumed;
                    self.vip_list.push(entry);
                }
            } else {
                return Err(AttrError::UnknownTag(tag));
            }
        }
        Ok(())
    }
}

fn read_u8(buf: &[u8], pos: &mut usize) -> Result<u8, AttrError> {
    if *pos >= buf.len() {
        return Err(AttrError::UnexpectedEof);
    }
    let v = buf[*pos];
    *pos += 1;
    Ok(v)
}

fn read_u16_le(buf: &[u8], pos: &mut usize) -> Result<u16, AttrError> {
    if *pos + 2 > buf.len() {
        return Err(AttrError::UnexpectedEof);
    }
    let v = u16::from_le_bytes([buf[*pos], buf[*pos + 1]]);
    *pos += 2;
    Ok(v)
}

fn read_u32_le(buf: &[u8], pos: &mut usize) -> Result<u32, AttrError> {
    if *pos + 4 > buf.len() {
        return Err(AttrError::UnexpectedEof);
    }
    let v = u32::from_le_bytes([buf[*pos], buf[*pos + 1], buf[*pos + 2], buf[*pos + 3]]);
    *pos += 4;
    Ok(v)
}

fn read_string(buf: &[u8], pos: &mut usize) -> Result<String, AttrError> {
    let len = read_u16_le(buf, pos)? as usize;
    if *pos + len > buf.len() {
        return Err(AttrError::InvalidStringLength);
    }
    let s = std::str::from_utf8(&buf[*pos..*pos + len])
        .map_err(|_| AttrError::InvalidStringLength)?
        .to_owned();
    *pos += len;
    Ok(s)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::constants::WeaponType as WT;
    use forgottenserver_items::items_registry::{slot_position as sp, ItemTypeData};
    use std::sync::Arc;

    fn make_item(weight: u32) -> Item {
        let type_data = ItemTypeData {
            weight,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        Item::new(Arc::new(type_data), 1)
    }

    /// Build an item with a specific slot-position mask, weapon type, and weight.
    fn make_equip(slot_pos: u32, weapon: WT, weight: u32) -> Item {
        let td = ItemTypeData {
            pickupable: true,
            moveable: true,
            weight,
            slot_position: slot_pos,
            weapon_type: weapon,
            ..Default::default()
        };
        Item::new(Arc::new(td), 1)
    }

    fn make_equip_stackable(slot_pos: u32, weapon: WT, weight: u32, count: u8) -> Item {
        let td = ItemTypeData {
            pickupable: true,
            moveable: true,
            stackable: true,
            weight,
            slot_position: slot_pos,
            weapon_type: weapon,
            ..Default::default()
        };
        Item::new(Arc::new(td), count)
    }

    fn player_with_capacity(cap: u32) -> Player {
        let mut p = Player::new(1, "Test", 1);
        p.set_capacity(cap);
        p
    }

    // -----------------------------------------------------------------------
    // Section (a) — Stamina
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_new_stamina_is_max() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_stamina(), STAMINA_MAX);
        assert_eq!(p.get_stamina(), 2520);
    }

    #[test]
    fn test_player_drain_stamina() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(60);
        assert_eq!(p.get_stamina(), 2460);
    }

    #[test]
    fn test_player_drain_stamina_clamped_to_zero() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(3000); // more than max
        assert_eq!(p.get_stamina(), 0);
    }

    #[test]
    fn test_player_recover_stamina() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(120);
        p.recover_stamina(60);
        assert_eq!(p.get_stamina(), 2460);
    }

    #[test]
    fn test_player_recover_stamina_clamped_to_max() {
        let mut p = Player::new(1, "Alon", 1);
        p.recover_stamina(100);
        assert_eq!(p.get_stamina(), STAMINA_MAX);
    }

    #[test]
    fn test_player_stamina_premium_minutes_above_threshold() {
        let mut p = Player::new(1, "Alon", 1);
        // 2520 - 2400 = 120 premium minutes
        assert_eq!(p.get_stamina_premium_minutes(), 120);
        // drain to exactly 2400 → 0 premium minutes
        p.drain_stamina(120);
        assert_eq!(p.get_stamina_premium_minutes(), 0);
    }

    #[test]
    fn test_player_stamina_premium_minutes_below_threshold() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(200); // stamina = 2320, below threshold
        assert_eq!(p.get_stamina_premium_minutes(), 0);
    }

    // -----------------------------------------------------------------------
    // Section (b) — Premium
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_is_premium_false_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_premium());
    }

    #[test]
    fn test_player_set_premium_true() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_premium(true);
        assert!(p.is_premium());
    }

    #[test]
    fn test_player_premium_days_zero_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_premium_days(), 0);
    }

    #[test]
    fn test_player_add_premium_days() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_premium_days(30);
        assert_eq!(p.get_premium_days(), 30);
    }

    // -----------------------------------------------------------------------
    // Section (c) — Skull / emblem
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_skull_none_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_skull(), Skull::None);
    }

    #[test]
    fn test_player_set_skull_red() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_skull(Skull::Red);
        assert_eq!(p.get_skull(), Skull::Red);
    }

    #[test]
    fn test_player_skull_ticks_zero_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_skull_ticks(), 0);
    }

    #[test]
    fn test_player_set_skull_ticks() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_skull_ticks(5000);
        assert_eq!(p.get_skull_ticks(), 5000);
    }

    #[test]
    fn test_skull_enum_all_variants() {
        let skulls = [
            Skull::None,
            Skull::Yellow,
            Skull::Green,
            Skull::White,
            Skull::Red,
            Skull::Black,
            Skull::Orange,
        ];
        for skull in skulls {
            let mut p = Player::new(1, "X", 0);
            p.set_skull(skull);
            assert_eq!(p.get_skull(), skull);
        }
    }

    // -----------------------------------------------------------------------
    // Section (d) — Vocation stats
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_level_one_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_level(), 1);
    }

    #[test]
    fn test_player_set_level() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_level(50);
        assert_eq!(p.get_level(), 50);
    }

    #[test]
    fn test_player_max_health_default_100() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_max_health(), 100);
    }

    #[test]
    fn test_player_apply_level_stats() {
        let mut p = Player::new(1, "Alon", 1);
        p.apply_level_stats(10, 5, 150);
        assert_eq!(p.get_max_health(), 110);
        assert_eq!(p.get_max_mana(), 5);
        assert_eq!(p.get_capacity(), CAPACITY_DEFAULT + 150);
    }

    // -----------------------------------------------------------------------
    // Section (e) — Skill advance
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_skill_level_minimum_default() {
        let p = Player::new(1, "Alon", 1);
        for skill in [
            SkillType::Fist,
            SkillType::Club,
            SkillType::Sword,
            SkillType::Axe,
            SkillType::Distance,
            SkillType::Shield,
            SkillType::Fishing,
            SkillType::MagicLevel,
        ] {
            assert_eq!(p.get_skill_level(skill), MINIMUM_SKILL_LEVEL);
        }
    }

    #[test]
    fn test_player_skill_tries_zero_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_skill_tries(SkillType::Sword), 0);
    }

    #[test]
    fn test_player_add_skill_tries_increases_tries() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_skill_tries(SkillType::Sword, 100);
        // 100 < tries_needed(10) = 10,000 → no level advance
        assert_eq!(p.get_skill_tries(SkillType::Sword), 100);
        assert_eq!(p.get_skill_level(SkillType::Sword), MINIMUM_SKILL_LEVEL);
    }

    #[test]
    fn test_player_skill_advance_level_up_at_10000_tries() {
        let mut p = Player::new(1, "Alon", 1);
        // tries_needed(10) = 10 * 10^3 = 10000
        p.add_skill_tries(SkillType::Sword, 10_000);
        assert_eq!(p.get_skill_level(SkillType::Sword), MINIMUM_SKILL_LEVEL + 1);
    }

    #[test]
    fn test_player_skill_percent_calculation() {
        let mut p = Player::new(1, "Alon", 1);
        // tries_needed(10) = 10,000
        // add 5000 tries → percent = 5000 * 100 / 10000 = 50
        p.add_skill_tries(SkillType::Club, 5_000);
        assert_eq!(p.get_skill_percent(SkillType::Club), 50);
    }

    #[test]
    fn test_player_skill_percent_zero_no_tries() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_skill_percent(SkillType::Axe), 0);
    }

    // -----------------------------------------------------------------------
    // Section (f) — Offline training
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_offline_training_skill_unset_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_offline_training_skill(), OfflineTrainingSkill::Unset);
    }

    #[test]
    fn test_player_set_offline_training_skill() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_offline_training_skill(OfflineTrainingSkill::Sword);
        assert_eq!(p.get_offline_training_skill(), OfflineTrainingSkill::Sword);
    }

    #[test]
    fn test_player_offline_training_time_zero_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_offline_training_time(), 0);
    }

    #[test]
    fn test_player_set_offline_training_time() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_offline_training_time(3600);
        assert_eq!(p.get_offline_training_time(), 3600);
    }

    #[test]
    fn test_offline_training_skill_all_variants() {
        let variants = [
            OfflineTrainingSkill::Shield,
            OfflineTrainingSkill::Distance,
            OfflineTrainingSkill::Club,
            OfflineTrainingSkill::Sword,
            OfflineTrainingSkill::Axe,
            OfflineTrainingSkill::Fist,
            OfflineTrainingSkill::Unset,
        ];
        for v in variants {
            let mut p = Player::new(1, "X", 0);
            p.set_offline_training_skill(v);
            assert_eq!(p.get_offline_training_skill(), v);
        }
    }

    // -----------------------------------------------------------------------
    // Section (g) — Inventory slots
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_inventory_all_slots_empty_by_default() {
        let p = Player::new(1, "Alon", 1);
        for slot in [
            InventorySlot::Head,
            InventorySlot::Necklace,
            InventorySlot::Backpack,
            InventorySlot::Armor,
            InventorySlot::Right,
            InventorySlot::Left,
            InventorySlot::Legs,
            InventorySlot::Feet,
            InventorySlot::Ring,
            InventorySlot::Ammo,
            InventorySlot::Depot,
        ] {
            assert!(p.get_inventory_item(slot).is_none());
        }
    }

    #[test]
    fn test_player_set_inventory_item() {
        let mut p = Player::new(1, "Alon", 1);
        let item = make_item(100);
        p.set_inventory_item(InventorySlot::Armor, item);
        assert!(p.get_inventory_item(InventorySlot::Armor).is_some());
    }

    #[test]
    fn test_player_remove_inventory_item() {
        let mut p = Player::new(1, "Alon", 1);
        let item = make_item(100);
        p.set_inventory_item(InventorySlot::Head, item);
        let removed = p.remove_inventory_item(InventorySlot::Head);
        assert!(removed.is_some());
        assert!(p.get_inventory_item(InventorySlot::Head).is_none());
    }

    #[test]
    fn test_player_inventory_weight_sum() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_inventory_item(InventorySlot::Head, make_item(50));
        p.set_inventory_item(InventorySlot::Armor, make_item(150));
        assert_eq!(p.get_inventory_weight(), 200);
    }

    #[test]
    fn test_player_inventory_weight_empty() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_inventory_weight(), 0);
    }

    // -----------------------------------------------------------------------
    // Section (g-ext) — Task 8.1 inventory: add/remove/query/notifications
    // -----------------------------------------------------------------------

    #[test]
    fn get_carry_weight_equals_inventory_weight() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(InventorySlot::Armor, make_item(200));
        assert_eq!(p.get_carry_weight(), p.get_inventory_weight());
    }

    #[test]
    fn has_extra_capacity_fits() {
        let mut p = player_with_capacity(500);
        p.add_item_to_inventory(InventorySlot::Head, make_item(100));
        assert!(p.has_extra_capacity(300));
    }

    #[test]
    fn has_extra_capacity_overweight() {
        let mut p = player_with_capacity(100);
        p.add_item_to_inventory(InventorySlot::Head, make_item(90));
        assert!(!p.has_extra_capacity(11));
    }

    #[test]
    fn has_extra_capacity_at_limit() {
        assert!(player_with_capacity(100).has_extra_capacity(100));
    }

    #[test]
    fn add_item_places_no_displaced() {
        let mut p = player_with_capacity(1000);
        assert!(p
            .add_item_to_inventory(InventorySlot::Armor, make_equip(sp::ARMOR, WT::None, 10))
            .is_none());
        assert!(p.get_inventory_item(InventorySlot::Armor).is_some());
    }

    #[test]
    fn add_item_returns_displaced() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(InventorySlot::Armor, make_equip(sp::ARMOR, WT::None, 10));
        let d = p.add_item_to_inventory(InventorySlot::Armor, make_equip(sp::ARMOR, WT::None, 20));
        assert!(d.is_some());
        assert_eq!(
            p.get_inventory_item(InventorySlot::Armor)
                .unwrap()
                .get_weight(),
            20
        );
    }

    #[test]
    fn remove_item_from_inv_some() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(InventorySlot::Head, make_equip(sp::HEAD, WT::None, 30));
        assert!(p.remove_item_from_inventory(InventorySlot::Head).is_some());
        assert!(p.get_inventory_item(InventorySlot::Head).is_none());
    }

    #[test]
    fn remove_item_from_inv_empty_none() {
        let mut p = player_with_capacity(1000);
        assert!(p.remove_item_from_inventory(InventorySlot::Ring).is_none());
    }

    #[test]
    fn query_add_non_pickupable() {
        let p = player_with_capacity(1000);
        let td = ItemTypeData {
            slot_position: sp::ARMOR,
            ..Default::default()
        };
        assert_eq!(
            p.query_add(InventorySlot::Armor, &Item::new(Arc::new(td), 1), 1, 0),
            ReturnValue::CannotPickup
        );
    }

    #[test]
    fn query_add_store_item_rejected() {
        let p = player_with_capacity(1000);
        let td = ItemTypeData {
            pickupable: true,
            store_item: true,
            slot_position: sp::ARMOR,
            ..Default::default()
        };
        assert_eq!(
            p.query_add(InventorySlot::Armor, &Item::new(Arc::new(td), 1), 1, 0),
            ReturnValue::ItemCannotBeMovedThere
        );
    }

    #[test]
    fn query_add_wrong_slot_rejected() {
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Armor,
                &make_equip(sp::HEAD, WT::None, 5),
                1,
                0
            ),
            ReturnValue::CannotBeDressed
        );
    }

    #[test]
    fn query_add_simple_slots_ok() {
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Head,
                &make_equip(sp::HEAD, WT::None, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Necklace,
                &make_equip(sp::NECKLACE, WT::None, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Backpack,
                &make_equip(sp::BACKPACK, WT::None, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Armor,
                &make_equip(sp::ARMOR, WT::None, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Legs,
                &make_equip(sp::LEGS, WT::None, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Feet,
                &make_equip(sp::FEET, WT::None, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Ring,
                &make_equip(sp::RING, WT::None, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Ammo,
                &make_equip(sp::AMMO, WT::Ammo, 1),
                1,
                0
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_overweight_rejected() {
        let mut p = player_with_capacity(50);
        p.add_item_to_inventory(InventorySlot::Head, make_item(40));
        assert_eq!(
            p.query_add(
                InventorySlot::Armor,
                &make_equip(sp::ARMOR, WT::None, 20),
                1,
                0
            ),
            ReturnValue::NotEnoughCapacity
        );
    }

    #[test]
    fn query_add_exactly_at_cap_ok() {
        let mut p = player_with_capacity(100);
        p.add_item_to_inventory(InventorySlot::Head, make_item(60));
        assert_eq!(
            p.query_add(
                InventorySlot::Armor,
                &make_equip(sp::ARMOR, WT::None, 40),
                1,
                0
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_child_is_owner_bypasses() {
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Armor,
                &make_equip(sp::HEAD, WT::None, 5),
                1,
                query_flags::CHILD_IS_OWNER
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_child_is_owner_no_limit_ok() {
        let p = player_with_capacity(0);
        assert_eq!(
            p.query_add(
                InventorySlot::Head,
                &make_equip(sp::HEAD, WT::None, 9999),
                1,
                query_flags::CHILD_IS_OWNER | query_flags::NO_LIMIT
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_right_one_hand_left_empty() {
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Sword, 5),
                1,
                0
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_two_hander_right_left_occupied() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Left,
            make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5),
                1,
                0
            ),
            ReturnValue::BothHandsNeedToBeFree
        );
    }

    #[test]
    fn query_add_two_hander_right_left_empty_ok() {
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5),
                1,
                0
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_shield_right_left_two_hander_drop() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Left,
            make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
                1,
                0
            ),
            ReturnValue::DropTwoHandedItem
        );
    }

    #[test]
    fn query_add_two_shields_blocked() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Left,
            make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
                1,
                0
            ),
            ReturnValue::CanOnlyUseOneShield
        );
    }

    #[test]
    fn query_add_two_melee_right_blocked() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Left,
            make_equip(sp::LEFT | sp::RIGHT, WT::Sword, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Axe, 5),
                1,
                0
            ),
            ReturnValue::CanOnlyUseOneWeapon
        );
    }

    #[test]
    fn query_add_shield_left_right_two_hander_drop() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
                1,
                0
            ),
            ReturnValue::DropTwoHandedItem
        );
    }

    #[test]
    fn query_add_two_hander_left_right_occupied() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5),
                1,
                0
            ),
            ReturnValue::BothHandsNeedToBeFree
        );
    }

    #[test]
    fn query_add_two_melee_left_blocked() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Club, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Sword, 5),
                1,
                0
            ),
            ReturnValue::CanOnlyUseOneWeapon
        );
    }

    #[test]
    fn query_add_shield_left_sword_right_ok() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Sword, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
                1,
                0
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_hand_item_wrong_slot() {
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Head,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
                1,
                0
            ),
            ReturnValue::CannotBeDressed
        );
    }

    #[test]
    fn query_add_stackable_overweight() {
        let p = player_with_capacity(50);
        // base_weight=5, count=20 → 100 > 50
        assert_eq!(
            p.query_add(
                InventorySlot::Ammo,
                &make_equip_stackable(sp::AMMO, WT::Ammo, 5, 1),
                20,
                0
            ),
            ReturnValue::NotEnoughCapacity
        );
    }

    #[test]
    fn query_add_stackable_within_cap() {
        let p = player_with_capacity(100);
        // base_weight=5, count=10 → 50 ≤ 100
        assert_eq!(
            p.query_add(
                InventorySlot::Ammo,
                &make_equip_stackable(sp::AMMO, WT::Ammo, 5, 1),
                10,
                0
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_remove_empty_not_possible() {
        assert_eq!(
            player_with_capacity(1000).query_remove(InventorySlot::Armor, 1, 0),
            ReturnValue::NotPossible
        );
    }

    #[test]
    fn query_remove_count_zero_not_possible() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(InventorySlot::Armor, make_equip(sp::ARMOR, WT::None, 5));
        assert_eq!(
            p.query_remove(InventorySlot::Armor, 0, 0),
            ReturnValue::NotPossible
        );
    }

    #[test]
    fn query_remove_moveable_ok() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(InventorySlot::Armor, make_equip(sp::ARMOR, WT::None, 5));
        assert_eq!(
            p.query_remove(InventorySlot::Armor, 1, 0),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_remove_not_moveable_no_flag() {
        let mut p = player_with_capacity(1000);
        let td = ItemTypeData {
            pickupable: true,
            moveable: false,
            slot_position: sp::ARMOR,
            ..Default::default()
        };
        p.add_item_to_inventory(InventorySlot::Armor, Item::new(Arc::new(td), 1));
        assert_eq!(
            p.query_remove(InventorySlot::Armor, 1, 0),
            ReturnValue::NotMoveable
        );
    }

    #[test]
    fn query_remove_not_moveable_with_flag_ok() {
        let mut p = player_with_capacity(1000);
        let td = ItemTypeData {
            pickupable: true,
            moveable: false,
            slot_position: sp::ARMOR,
            ..Default::default()
        };
        p.add_item_to_inventory(InventorySlot::Armor, Item::new(Arc::new(td), 1));
        assert_eq!(
            p.query_remove(InventorySlot::Armor, 1, query_flags::IGNORE_NOT_MOVEABLE),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_remove_stackable_count_exceeds() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Ammo,
            make_equip_stackable(sp::AMMO, WT::Ammo, 1, 5),
        );
        assert_eq!(
            p.query_remove(InventorySlot::Ammo, 10, 0),
            ReturnValue::NotPossible
        );
    }

    #[test]
    fn query_remove_stackable_valid() {
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Ammo,
            make_equip_stackable(sp::AMMO, WT::Ammo, 1, 10),
        );
        assert_eq!(
            p.query_remove(InventorySlot::Ammo, 5, 0),
            ReturnValue::NoError
        );
    }

    #[test]
    fn post_add_notification_no_panic() {
        player_with_capacity(1000).post_add_notification(InventorySlot::Armor);
    }

    #[test]
    fn post_remove_notification_no_panic() {
        player_with_capacity(1000).post_remove_notification(InventorySlot::Head);
    }

    // -----------------------------------------------------------------------
    // Section (h) — Depot
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_max_depot_items_zero_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_max_depot_items(), 0);
    }

    #[test]
    fn test_player_set_max_depot_items() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_max_depot_items(2000);
        assert_eq!(p.get_max_depot_items(), 2000);
    }

    // -----------------------------------------------------------------------
    // Section (i) — Party state machine
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_party_id_none_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_party_id(), None);
    }

    #[test]
    fn test_player_set_party_id() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_party_id(Some(42));
        assert_eq!(p.get_party_id(), Some(42));
    }

    #[test]
    fn test_player_leave_party() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_party_id(Some(42));
        p.leave_party();
        assert_eq!(p.get_party_id(), None);
    }

    // -----------------------------------------------------------------------
    // Section (j) — Trade state machine
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_trade_state_none_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_trade_state(), TradeState::None);
    }

    #[test]
    fn test_player_set_trade_state() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_trade_state(TradeState::Initiated);
        assert_eq!(p.get_trade_state(), TradeState::Initiated);
    }

    #[test]
    fn test_player_cancel_trade() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_trade_state(TradeState::Accept);
        p.cancel_trade();
        assert_eq!(p.get_trade_state(), TradeState::None);
    }

    #[test]
    fn test_player_trade_partner_guid_none_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_trade_partner_guid(), None);
    }

    #[test]
    fn test_player_set_trade_partner_guid() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_trade_partner_guid(99);
        assert_eq!(p.get_trade_partner_guid(), Some(99));
    }

    // -----------------------------------------------------------------------
    // Section (k) — Weight / capacity
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_capacity_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_capacity(), CAPACITY_DEFAULT);
        assert_eq!(p.get_capacity(), 400);
    }

    #[test]
    fn test_player_set_capacity() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_capacity(1000);
        assert_eq!(p.get_capacity(), 1000);
    }

    #[test]
    fn test_player_get_free_capacity() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_capacity(500);
        assert_eq!(p.get_free_capacity(200), 300);
    }

    #[test]
    fn test_player_get_free_capacity_clamped_to_zero() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_capacity(100);
        assert_eq!(p.get_free_capacity(200), 0); // overweight
    }

    #[test]
    fn test_player_can_carry_weight_true() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_capacity(500);
        assert!(p.can_carry_weight(100, 300)); // 300 + 100 = 400 <= 500
    }

    #[test]
    fn test_player_can_carry_weight_false() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_capacity(500);
        assert!(!p.can_carry_weight(201, 300)); // 300 + 201 = 501 > 500
    }

    #[test]
    fn test_player_can_carry_weight_exactly_at_capacity() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_capacity(500);
        assert!(p.can_carry_weight(200, 300)); // exactly 500
    }

    // -----------------------------------------------------------------------
    // Section (l) — VIP list
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_add_vip() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_vip(VIPEntry::new(10, "Friend", "", 0, false));
        assert!(p.is_in_vip_list(10));
    }

    #[test]
    fn test_player_is_in_vip_list_false_when_not_added() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_in_vip_list(99));
    }

    #[test]
    fn test_player_remove_vip() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_vip(VIPEntry::new(10, "Friend", "", 0, false));
        p.remove_vip(10);
        assert!(!p.is_in_vip_list(10));
    }

    #[test]
    fn test_player_get_vip_count() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_vip(VIPEntry::new(1, "A", "", 0, false));
        p.add_vip(VIPEntry::new(2, "B", "", 0, false));
        assert_eq!(p.get_vip_count(), 2);
    }

    #[test]
    fn test_player_max_vip_entries_zero_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_max_vip_entries(), 0);
    }

    #[test]
    fn test_player_set_max_vip_entries() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_max_vip_entries(200);
        assert_eq!(p.get_max_vip_entries(), 200);
    }

    // -----------------------------------------------------------------------
    // Section (m) — Blessings
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_has_blessing_false_initially() {
        let p = Player::new(1, "Alon", 1);
        for n in 0..5 {
            assert!(!p.has_blessing(n));
        }
    }

    #[test]
    fn test_player_add_blessing() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_blessing(2);
        assert!(p.has_blessing(2));
        assert!(!p.has_blessing(0));
    }

    #[test]
    fn test_player_get_blessing_count() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_blessing(0);
        p.add_blessing(1);
        p.add_blessing(4);
        assert_eq!(p.get_blessing_count(), 3);
    }

    #[test]
    fn test_player_xp_loss_no_blessings_10_percent() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_xp_loss_percent(false), 10);
    }

    #[test]
    fn test_player_xp_loss_5_blessings_zero_percent() {
        let mut p = Player::new(1, "Alon", 1);
        for n in 0..5 {
            p.add_blessing(n);
        }
        // 10 - 5*8 = -30, clamped to 0
        assert_eq!(p.get_xp_loss_percent(false), 0);
    }

    #[test]
    fn test_player_xp_loss_amulet_zero_percent() {
        let p = Player::new(1, "Alon", 1);
        // Even without blessings, amulet gives 0% loss
        assert_eq!(p.get_xp_loss_percent(true), 0);
    }

    #[test]
    fn test_player_xp_loss_one_blessing_2_percent() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_blessing(0);
        // 10 - 8 = 2
        assert_eq!(p.get_xp_loss_percent(false), 2);
    }

    // -----------------------------------------------------------------------
    // Section (n) — Murder count / red skull
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_unjustified_kills_zero_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_unjustified_kills(), 0);
    }

    #[test]
    fn test_player_add_unjustified_kill() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_unjustified_kill();
        assert_eq!(p.get_unjustified_kills(), 1);
    }

    #[test]
    fn test_player_is_killer_false_no_kills() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_killer());
    }

    #[test]
    fn test_player_is_killer_true_with_kill() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_unjustified_kill();
        assert!(p.is_killer());
    }

    // -----------------------------------------------------------------------
    // Section (o) — Soul points
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_soul_100_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_soul(), 100);
    }

    #[test]
    fn test_player_max_soul_100_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_max_soul(), 100);
    }

    #[test]
    fn test_player_set_soul() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_soul(50);
        assert_eq!(p.get_soul(), 50);
    }

    #[test]
    fn test_player_consume_soul() {
        let mut p = Player::new(1, "Alon", 1);
        p.consume_soul(30);
        assert_eq!(p.get_soul(), 70);
    }

    #[test]
    fn test_player_consume_soul_clamped_to_zero() {
        let mut p = Player::new(1, "Alon", 1);
        p.consume_soul(200); // more than max
        assert_eq!(p.get_soul(), 0);
    }

    // -----------------------------------------------------------------------
    // Section (p) — Combat modes
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_safe_mode_true_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(p.get_safe_mode());
    }

    #[test]
    fn test_player_set_safe_mode_false() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_safe_mode(false);
        assert!(!p.get_safe_mode());
    }

    #[test]
    fn test_player_fight_mode_attack_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_fight_mode(), FightMode::Attack);
    }

    #[test]
    fn test_player_set_fight_mode_defense() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_fight_mode(FightMode::Defense);
        assert_eq!(p.get_fight_mode(), FightMode::Defense);
    }

    #[test]
    fn test_player_pvp_mode_dove_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_pvp_mode(), PvpMode::Dove);
    }

    #[test]
    fn test_player_set_pvp_mode_red_fist() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_pvp_mode(PvpMode::RedFist);
        assert_eq!(p.get_pvp_mode(), PvpMode::RedFist);
    }

    // -----------------------------------------------------------------------
    // Runtime state — health / mana / experience
    // -----------------------------------------------------------------------

    #[test]
    fn test_player_health_default_100() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_health(), 100);
    }

    #[test]
    fn test_player_set_health() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_health(50);
        assert_eq!(p.get_health(), 50);
    }

    #[test]
    fn test_player_mana_default_zero() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_mana(), 0);
    }

    #[test]
    fn test_player_set_mana() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_mana(80);
        assert_eq!(p.get_mana(), 80);
    }

    #[test]
    fn test_player_experience_default_zero() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_experience(), 0);
    }

    #[test]
    fn test_player_add_experience() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_experience(500);
        assert_eq!(p.get_experience(), 500);
    }

    // -----------------------------------------------------------------------
    // XP formula
    // -----------------------------------------------------------------------

    #[test]
    fn xp_for_level_1_is_zero() {
        assert_eq!(xp_for_level(1), 0);
    }

    #[test]
    fn xp_for_level_2_is_100() {
        assert_eq!(xp_for_level(2), 100);
    }

    #[test]
    fn xp_for_level_3_is_200() {
        // C++ formula: (((lv-6)*lv+17)*lv-12)/6*100
        // lv=3: (((-3)*3+17)*3-12)/6*100 = ((8)*3-12)/6*100 = 12/6*100 = 200
        assert_eq!(xp_for_level(3), 200);
    }

    // -----------------------------------------------------------------------
    // Level-up
    // -----------------------------------------------------------------------

    #[test]
    fn level_up_updates_max_health_mana_speed() {
        let mut p = Player::new(1, "Alon", 1);
        let initial_hp = p.get_max_health();
        let initial_mana = p.get_max_mana();
        p.add_experience(xp_for_level(2)); // exactly 100 XP
        let new_level = p.check_level_up();
        assert_eq!(new_level, Some(2));
        assert!(
            p.get_max_health() > initial_hp,
            "max_health must increase on level-up"
        );
        assert!(
            p.get_max_mana() > initial_mana,
            "max_mana must increase on level-up"
        );
    }

    #[test]
    fn check_level_up_returns_none_when_not_enough_xp() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_experience(50); // not enough for level 2
        assert_eq!(p.check_level_up(), None);
        assert_eq!(p.get_level(), 1);
    }

    // -----------------------------------------------------------------------
    // Death — skill loss
    // -----------------------------------------------------------------------

    #[test]
    fn compute_skill_loss_one_percent_of_level() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_skill_level(SkillType::Sword, 100);
        let loss = p.compute_skill_loss();
        assert_eq!(loss[SkillType::Sword as usize], 1); // 1% of 100 = 1
    }

    #[test]
    fn compute_skill_loss_at_minimum_is_zero() {
        let p = Player::new(1, "Alon", 1);
        // Default skill = MINIMUM_SKILL_LEVEL (10) → no loss
        let loss = p.compute_skill_loss();
        assert_eq!(loss[SkillType::Sword as usize], 0);
    }

    #[test]
    fn apply_skill_loss_reduces_level() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_skill_level(SkillType::Sword, 100);
        let loss = p.compute_skill_loss();
        p.apply_skill_loss(loss);
        assert_eq!(p.get_skill_level(SkillType::Sword), 99);
    }

    #[test]
    fn apply_skill_loss_does_not_go_below_minimum() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_skill_level(SkillType::Sword, 11); // just above minimum
        let loss = [1u16; SKILL_COUNT];
        p.apply_skill_loss(loss);
        assert_eq!(p.get_skill_level(SkillType::Sword), MINIMUM_SKILL_LEVEL);
    }

    #[test]
    fn apply_skill_loss_resets_tries_and_percent() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_skill_level(SkillType::Sword, 50);
        p.add_skill_tries(SkillType::Sword, 5_000);
        let loss = p.compute_skill_loss();
        p.apply_skill_loss(loss);
        assert_eq!(p.get_skill_tries(SkillType::Sword), 0);
        assert_eq!(p.get_skill_percent(SkillType::Sword), 0);
    }

    // -----------------------------------------------------------------------
    // Death — item loss
    // -----------------------------------------------------------------------

    #[test]
    fn compute_item_loss_empty_with_five_blessings() {
        let mut p = Player::new(1, "Alon", 1);
        for n in 0..5 {
            p.add_blessing(n);
        }
        assert!(p.compute_item_loss().is_empty());
    }

    #[test]
    fn compute_item_loss_non_empty_without_blessings() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.compute_item_loss().is_empty());
    }

    #[test]
    fn drop_items_removes_from_inventory() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_inventory_item(InventorySlot::Head, make_item(100));
        let dropped = p.drop_items(vec![InventorySlot::Head]);
        assert_eq!(dropped.len(), 1);
        assert!(p.get_inventory_item(InventorySlot::Head).is_none());
    }

    // -----------------------------------------------------------------------
    // Offline stamina recovery
    // -----------------------------------------------------------------------

    #[test]
    fn stamina_recovers_offline_at_one_per_three_minutes() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(60);
        let before = p.get_stamina();
        // 180 seconds offline → 1 minute recovery
        p.apply_offline_stamina_recovery(180);
        assert_eq!(p.get_stamina(), before + 1);
    }

    #[test]
    fn offline_recovery_does_not_exceed_max() {
        let mut p = Player::new(1, "Alon", 1); // starts at STAMINA_MAX
        p.apply_offline_stamina_recovery(10_000);
        assert_eq!(p.get_stamina(), STAMINA_MAX);
    }

    #[test]
    fn offline_recovery_less_than_180s_is_zero() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(60);
        let before = p.get_stamina();
        p.apply_offline_stamina_recovery(179); // just under 180s
        assert_eq!(p.get_stamina(), before); // no recovery
    }

    // -----------------------------------------------------------------------
    // Task 8.5 — Friend list (8.5-a)
    // -----------------------------------------------------------------------

    #[test]
    fn friend_list_empty_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(p.get_friend_list().is_empty());
    }

    #[test]
    fn add_friend_returns_true_on_success() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(p.add_friend(42));
        assert!(p.is_friend(42));
    }

    #[test]
    fn add_friend_returns_false_on_duplicate() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_friend(42);
        assert!(!p.add_friend(42));
        assert_eq!(p.get_friend_list().len(), 1);
    }

    #[test]
    fn add_friend_returns_false_when_max_reached() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_max_friends(2);
        p.add_friend(1);
        p.add_friend(2);
        assert!(!p.add_friend(3));
        assert_eq!(p.get_friend_list().len(), 2);
    }

    #[test]
    fn remove_friend_removes_existing_returns_true() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_friend(42);
        assert!(p.remove_friend(42));
        assert!(!p.is_friend(42));
    }

    #[test]
    fn remove_friend_returns_false_when_not_present() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(!p.remove_friend(99));
    }

    #[test]
    fn is_friend_false_when_not_in_list() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_friend(7));
    }

    // -----------------------------------------------------------------------
    // Task 8.5 — Guild membership (8.5-b)
    // -----------------------------------------------------------------------

    #[test]
    fn is_guild_member_false_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_guild_member());
    }

    #[test]
    fn set_guild_id_makes_player_guild_member() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_guild_id(10);
        assert!(p.is_guild_member());
        assert_eq!(p.get_guild_id(), Some(10));
    }

    #[test]
    fn leave_guild_clears_guild_id_and_nick() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_guild_id(10);
        p.set_guild_nick("Knight");
        p.leave_guild();
        assert!(!p.is_guild_member());
        assert_eq!(p.get_guild_id(), None);
        assert_eq!(p.get_guild_nick(), "");
    }

    #[test]
    fn guild_nick_empty_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_guild_nick(), "");
    }

    #[test]
    fn set_guild_nick_stores_value() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_guild_nick("Elder");
        assert_eq!(p.get_guild_nick(), "Elder");
    }

    // -----------------------------------------------------------------------
    // Task 8.5 — Party invitations (8.5-c)
    // -----------------------------------------------------------------------

    #[test]
    fn no_party_invites_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.has_party_invite_from(5));
    }

    #[test]
    fn add_party_invite_returns_true_and_stores_invite() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(p.add_party_invite(5));
        assert!(p.has_party_invite_from(5));
    }

    #[test]
    fn add_party_invite_returns_false_on_duplicate() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_party_invite(5);
        assert!(!p.add_party_invite(5));
    }

    #[test]
    fn remove_party_invite_clears_invite() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_party_invite(5);
        p.remove_party_invite(5);
        assert!(!p.has_party_invite_from(5));
    }

    #[test]
    fn join_party_sets_party_id_and_removes_invite() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_party_invite(7);
        let result = p.join_party(7);
        assert!(result);
        assert_eq!(p.get_party_id(), Some(7));
        assert!(!p.has_party_invite_from(7));
    }

    #[test]
    fn join_party_returns_false_without_invite() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(!p.join_party(99));
        assert_eq!(p.get_party_id(), None);
    }

    #[test]
    fn clear_party_invites_removes_all() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_party_invite(1);
        p.add_party_invite(2);
        p.clear_party_invites();
        assert!(!p.has_party_invite_from(1));
        assert!(!p.has_party_invite_from(2));
    }

    // -----------------------------------------------------------------------
    // Task 8.5 — Channel subscriptions (8.5-d)
    // -----------------------------------------------------------------------

    #[test]
    fn no_channels_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(p.get_subscribed_channels().is_empty());
    }

    #[test]
    fn add_channel_subscribes_player() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_channel(1);
        assert!(p.is_subscribed_to_channel(1));
    }

    #[test]
    fn add_channel_no_duplicate() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_channel(1);
        p.add_channel(1);
        assert_eq!(p.get_subscribed_channels().len(), 1);
    }

    #[test]
    fn remove_channel_unsubscribes_player() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_channel(3);
        p.remove_channel(3);
        assert!(!p.is_subscribed_to_channel(3));
    }

    #[test]
    fn is_subscribed_to_channel_false_when_not_added() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_subscribed_to_channel(7));
    }

    #[test]
    fn multiple_channels_independent() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_channel(1);
        p.add_channel(2);
        p.add_channel(3);
        assert!(p.is_subscribed_to_channel(1));
        assert!(p.is_subscribed_to_channel(2));
        assert!(p.is_subscribed_to_channel(3));
        p.remove_channel(2);
        assert!(p.is_subscribed_to_channel(1));
        assert!(!p.is_subscribed_to_channel(2));
        assert!(p.is_subscribed_to_channel(3));
    }

    // -----------------------------------------------------------------------
    // Task 8.5 — Client version (8.5-e)
    // -----------------------------------------------------------------------

    #[test]
    fn client_version_zero_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_client_version(), 0);
    }

    #[test]
    fn set_client_version_stores_value() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_client_version(1240);
        assert_eq!(p.get_client_version(), 1240);
    }

    #[test]
    fn is_client_version_compatible_within_range() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_client_version(1240);
        assert!(p.is_client_version_compatible(1200, 1300));
    }

    #[test]
    fn is_client_version_compatible_at_min_boundary() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_client_version(1200);
        assert!(p.is_client_version_compatible(1200, 1300));
    }

    #[test]
    fn is_client_version_compatible_at_max_boundary() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_client_version(1300);
        assert!(p.is_client_version_compatible(1200, 1300));
    }

    #[test]
    fn is_client_version_compatible_below_min() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_client_version(1100);
        assert!(!p.is_client_version_compatible(1200, 1300));
    }

    #[test]
    fn is_client_version_compatible_above_max() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_client_version(1400);
        assert!(!p.is_client_version_compatible(1200, 1300));
    }

    // -----------------------------------------------------------------------
    // Task 8.5 — Key-value storage (8.5-f)
    // -----------------------------------------------------------------------

    #[test]
    fn storage_empty_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_storage_value(1000), None);
    }

    #[test]
    fn set_storage_value_stores_and_retrieves() {
        let mut p = Player::new(1, "Alon", 1);
        let ok = p.set_storage_value(1000, Some(42));
        assert!(ok);
        assert_eq!(p.get_storage_value(1000), Some(42));
    }

    #[test]
    fn set_storage_value_none_removes_key() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_storage_value(1000, Some(99));
        p.set_storage_value(1000, None);
        assert_eq!(p.get_storage_value(1000), None);
    }

    #[test]
    fn set_storage_value_overwrites_existing() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_storage_value(1000, Some(1));
        p.set_storage_value(1000, Some(2));
        assert_eq!(p.get_storage_value(1000), Some(2));
    }

    #[test]
    fn set_storage_value_rejects_reserved_range() {
        let mut p = Player::new(1, "Alon", 1);
        // 0x10000000 is the start of the reserved range
        let ok = p.set_storage_value(0x1000_0000, Some(1));
        assert!(!ok);
        assert_eq!(p.get_storage_value(0x1000_0000), None);
    }

    #[test]
    fn set_storage_value_rejects_upper_reserved_range() {
        let mut p = Player::new(1, "Alon", 1);
        let ok = p.set_storage_value(0x1FFF_FFFF, Some(5));
        assert!(!ok);
    }

    #[test]
    fn set_storage_value_accepts_key_just_below_reserved_range() {
        let mut p = Player::new(1, "Alon", 1);
        let ok = p.set_storage_value(0x0FFF_FFFF, Some(77));
        assert!(ok);
        assert_eq!(p.get_storage_value(0x0FFF_FFFF), Some(77));
    }

    #[test]
    fn set_storage_value_accepts_key_just_above_reserved_range() {
        let mut p = Player::new(1, "Alon", 1);
        let ok = p.set_storage_value(0x2000_0000, Some(88));
        assert!(ok);
        assert_eq!(p.get_storage_value(0x2000_0000), Some(88));
    }

    #[test]
    fn storage_negative_values() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_storage_value(500, Some(-10));
        assert_eq!(p.get_storage_value(500), Some(-10));
    }

    #[test]
    fn storage_iter_returns_all_entries() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_storage_value(1, Some(10));
        p.set_storage_value(2, Some(20));
        let mut pairs: Vec<(u32, i32)> = p.storage_iter().collect();
        pairs.sort_by_key(|&(k, _)| k);
        assert_eq!(pairs, vec![(1, 10), (2, 20)]);
    }

    // -----------------------------------------------------------------------
    // Task 8.4 — Player persistence: position, depot, VIP, blessings, attrs
    // -----------------------------------------------------------------------

    // --- login_position / temp_position / get_saved_position ---

    #[test]
    fn login_position_default_is_origin() {
        let p = Player::new(1, "Hero", 1);
        let lp = p.get_login_position();
        assert_eq!((lp.x, lp.y, lp.z), (0, 0, 0));
    }

    #[test]
    fn set_login_position_roundtrip() {
        let mut p = Player::new(1, "Hero", 1);
        let pos = Position {
            x: 100,
            y: 200,
            z: 7,
        };
        p.set_login_position(pos);
        let got = p.get_login_position();
        assert_eq!((got.x, got.y, got.z), (100, 200, 7));
    }

    #[test]
    fn temp_position_none_by_default() {
        let p = Player::new(1, "Hero", 1);
        assert!(p.get_temp_position().is_none());
    }

    #[test]
    fn set_and_clear_temp_position() {
        let mut p = Player::new(1, "Hero", 1);
        p.set_temp_position(Position { x: 50, y: 60, z: 3 });
        assert!(p.get_temp_position().is_some());
        p.clear_temp_position();
        assert!(p.get_temp_position().is_none());
    }

    #[test]
    fn get_saved_position_returns_temp_when_set() {
        let mut p = Player::new(1, "Hero", 1);
        p.set_login_position(Position {
            x: 100,
            y: 200,
            z: 7,
        });
        p.set_temp_position(Position { x: 50, y: 60, z: 3 });
        let sp = p.get_saved_position();
        assert_eq!((sp.x, sp.y, sp.z), (50, 60, 3));
    }

    #[test]
    fn get_saved_position_returns_login_when_no_temp() {
        let mut p = Player::new(1, "Hero", 1);
        p.set_login_position(Position {
            x: 100,
            y: 200,
            z: 7,
        });
        let sp = p.get_saved_position();
        assert_eq!((sp.x, sp.y, sp.z), (100, 200, 7));
    }

    #[test]
    fn get_saved_position_falls_back_to_temple_when_login_is_origin() {
        let mut p = Player::new(1, "Hero", 1);
        // login_position is (0,0,0) by default — should fall back to temple_pos
        p.temple_pos = Position {
            x: 300,
            y: 400,
            z: 7,
        };
        let sp = p.get_saved_position();
        assert_eq!((sp.x, sp.y, sp.z), (300, 400, 7));
    }

    // --- Depot loading ---

    #[test]
    fn depot_items_empty_by_default() {
        let p = Player::new(1, "Hero", 1);
        assert_eq!(p.get_depot_items(0).len(), 0);
        assert_eq!(p.depot_ids().count(), 0);
    }

    #[test]
    fn add_and_get_depot_item() {
        let mut p = Player::new(1, "Hero", 1);
        let item = make_item(100);
        p.add_depot_item(1, item);
        assert_eq!(p.get_depot_items(1).len(), 1);
        assert_eq!(p.get_depot_items(0).len(), 0); // different depot
    }

    #[test]
    fn take_depot_items_removes_them() {
        let mut p = Player::new(1, "Hero", 1);
        p.add_depot_item(2, make_item(50));
        p.add_depot_item(2, make_item(75));
        let taken = p.take_depot_items(2);
        assert_eq!(taken.len(), 2);
        assert_eq!(p.get_depot_items(2).len(), 0);
    }

    #[test]
    fn depot_ids_iterates_loaded_depots() {
        let mut p = Player::new(1, "Hero", 1);
        p.add_depot_item(0, make_item(10));
        p.add_depot_item(5, make_item(20));
        p.add_depot_item(99, make_item(30));
        let mut ids: Vec<u32> = p.depot_ids().collect();
        ids.sort();
        assert_eq!(ids, vec![0, 5, 99]);
    }

    // --- Blessings serialization helpers ---

    #[test]
    fn get_blessings_byte_reflects_bitmask() {
        let mut p = Player::new(1, "Hero", 1);
        p.add_blessing(0);
        p.add_blessing(2);
        p.add_blessing(4);
        assert_eq!(p.get_blessings_byte(), 0b0001_0101); // bits 0, 2, 4
    }

    #[test]
    fn set_blessings_from_byte_roundtrip() {
        let mut p = Player::new(1, "Hero", 1);
        p.set_blessings_from_byte(0b0001_1111); // all 5 blessings
        assert_eq!(p.get_blessing_count(), 5);
        assert!(p.has_blessing(0));
        assert!(p.has_blessing(4));
    }

    #[test]
    fn set_blessings_from_byte_masks_upper_bits() {
        let mut p = Player::new(1, "Hero", 1);
        p.set_blessings_from_byte(0xFF); // bits 5-7 should be stripped
        assert_eq!(p.get_blessings_byte(), 0b0001_1111); // only bits 0-4
    }

    #[test]
    fn remove_blessing_clears_bit() {
        let mut p = Player::new(1, "Hero", 1);
        p.add_blessing(3);
        assert!(p.has_blessing(3));
        p.remove_blessing(3);
        assert!(!p.has_blessing(3));
    }

    // --- VIPEntry binary round-trip ---

    #[test]
    fn vip_entry_to_bytes_from_bytes_roundtrip() {
        let original = VIPEntry::new(42, "Alice", "my friend", 7, true);
        let bytes = original.to_bytes();
        let (decoded, consumed) = VIPEntry::from_bytes(&bytes, 0).unwrap();
        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded, original);
    }

    #[test]
    fn vip_entry_from_bytes_notify_false() {
        let original = VIPEntry::new(99, "Bob", "", 0, false);
        let bytes = original.to_bytes();
        let (decoded, _) = VIPEntry::from_bytes(&bytes, 0).unwrap();
        assert!(!decoded.notify);
        assert_eq!(decoded.name, "Bob");
        assert_eq!(decoded.description, "");
    }

    #[test]
    fn vip_entry_from_bytes_truncated_returns_eof() {
        let entry = VIPEntry::new(1, "X", "", 0, false);
        let bytes = entry.to_bytes();
        // Truncate to 3 bytes (not even enough for guid)
        let result = VIPEntry::from_bytes(&bytes[..3], 0);
        assert_eq!(result, Err(AttrError::UnexpectedEof));
    }

    // --- Player serialize_attr / unserialize_attr round-trips ---

    #[test]
    fn serialize_attr_roundtrip_empty() {
        let p = Player::new(1, "Hero", 1);
        let blob = p.serialize_attr();
        let mut p2 = Player::new(2, "Zero", 1);
        p2.unserialize_attr(&blob).unwrap();
        // Default blessings = 0
        assert_eq!(p2.get_blessings_byte(), 0);
        // Default login_position = (0,0,0)
        let lp = p2.get_login_position();
        assert_eq!((lp.x, lp.y, lp.z), (0, 0, 0));
        // Empty VIP list
        assert_eq!(p2.get_vip_count(), 0);
    }

    #[test]
    fn serialize_attr_roundtrip_blessings() {
        let mut p = Player::new(1, "Hero", 1);
        p.add_blessing(0);
        p.add_blessing(3);
        let blob = p.serialize_attr();
        let mut p2 = Player::new(2, "Zero", 1);
        p2.unserialize_attr(&blob).unwrap();
        assert_eq!(p2.get_blessings_byte(), p.get_blessings_byte());
        assert!(p2.has_blessing(0));
        assert!(p2.has_blessing(3));
        assert!(!p2.has_blessing(1));
    }

    #[test]
    fn serialize_attr_roundtrip_login_position() {
        let mut p = Player::new(1, "Hero", 1);
        p.set_login_position(Position {
            x: 1000,
            y: 2000,
            z: 7,
        });
        let blob = p.serialize_attr();
        let mut p2 = Player::new(2, "Zero", 1);
        p2.unserialize_attr(&blob).unwrap();
        let lp = p2.get_login_position();
        assert_eq!((lp.x, lp.y, lp.z), (1000, 2000, 7));
    }

    #[test]
    fn serialize_attr_roundtrip_vip_list() {
        let mut p = Player::new(1, "Hero", 1);
        p.add_vip(VIPEntry::new(10, "Alice", "best friend", 2, true));
        p.add_vip(VIPEntry::new(20, "Bob", "", 0, false));
        let blob = p.serialize_attr();
        let mut p2 = Player::new(2, "Zero", 1);
        p2.unserialize_attr(&blob).unwrap();
        assert_eq!(p2.get_vip_count(), 2);
        assert!(p2.is_in_vip_list(10));
        assert!(p2.is_in_vip_list(20));
    }

    #[test]
    fn serialize_attr_roundtrip_full() {
        let mut p = Player::new(1, "Hero", 1);
        p.add_blessing(1);
        p.add_blessing(4);
        p.set_login_position(Position {
            x: 500,
            y: 700,
            z: 3,
        });
        p.add_vip(VIPEntry::new(55, "Carol", "ally", 1, true));
        let blob = p.serialize_attr();
        let mut p2 = Player::new(2, "Zero", 1);
        p2.unserialize_attr(&blob).unwrap();
        assert_eq!(p2.get_blessings_byte(), p.get_blessings_byte());
        let lp = p2.get_login_position();
        assert_eq!((lp.x, lp.y, lp.z), (500, 700, 3));
        assert_eq!(p2.get_vip_count(), 1);
        assert!(p2.is_in_vip_list(55));
    }

    #[test]
    fn unserialize_attr_unknown_tag_returns_error() {
        // Craft a blob with an unknown tag byte (0x99)
        let bad_blob = vec![0x99u8, 0xFF];
        let mut p = Player::new(1, "Hero", 1);
        let result = p.unserialize_attr(&bad_blob);
        assert_eq!(result, Err(AttrError::UnknownTag(0x99)));
    }

    #[test]
    fn unserialize_attr_truncated_returns_eof() {
        // Only a tag byte, no payload
        let bad_blob = vec![PlayerAttr::Blessings as u8]; // missing payload byte
        let mut p = Player::new(1, "Hero", 1);
        let result = p.unserialize_attr(&bad_blob);
        assert_eq!(result, Err(AttrError::UnexpectedEof));
    }

    #[test]
    fn unserialize_attr_empty_blob_returns_eof() {
        let mut p = Player::new(1, "Hero", 1);
        let result = p.unserialize_attr(&[]);
        assert_eq!(result, Err(AttrError::UnexpectedEof));
    }

    // -----------------------------------------------------------------------
    // Task 8.2 -- XP formula (C++ exact formula)
    // -----------------------------------------------------------------------

    #[test]
    fn xp_formula_level_4_is_400() {
        assert_eq!(xp_for_level(4), 400);
    }

    #[test]
    fn xp_formula_level_10_matches_cpp() {
        assert_eq!(xp_for_level(10), 9_300);
    }

    // -----------------------------------------------------------------------
    // Task 8.2 -- Speed: change_speed and set_base_speed
    // -----------------------------------------------------------------------

    #[test]
    fn player_new_speed_equals_220_at_level_1() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_speed(), 220);
    }

    #[test]
    fn change_speed_positive_delta_increases_speed() {
        let mut p = Player::new(1, "Alon", 1);
        p.change_speed(100);
        assert_eq!(p.get_speed(), 320);
    }

    #[test]
    fn change_speed_negative_delta_decreases_speed() {
        let mut p = Player::new(1, "Alon", 1);
        p.change_speed(-200);
        assert_eq!(p.get_speed(), 20);
    }

    #[test]
    fn change_speed_clamps_to_min() {
        let mut p = Player::new(1, "Alon", 1);
        p.change_speed(-10_000);
        assert_eq!(p.get_speed(), PLAYER_MIN_SPEED);
    }

    #[test]
    fn change_speed_clamps_to_max() {
        let mut p = Player::new(1, "Alon", 1);
        p.change_speed(10_000);
        assert_eq!(p.get_speed(), PLAYER_MAX_SPEED);
    }

    #[test]
    fn set_base_speed_clamps_to_min() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_base_speed(0);
        assert_eq!(p.get_speed(), PLAYER_MIN_SPEED);
    }

    #[test]
    fn set_base_speed_clamps_to_max() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_base_speed(99_999);
        assert_eq!(p.get_speed(), PLAYER_MAX_SPEED);
    }

    // -----------------------------------------------------------------------
    // Task 8.2 -- Magic level: add_mana_spent_advance and check_magic_level
    // -----------------------------------------------------------------------

    #[test]
    fn magic_level_zero_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_magic_level(), 0);
        assert_eq!(p.get_mana_spent(), 0);
        assert_eq!(p.get_magic_level_percent(), 0);
    }

    #[test]
    fn set_magic_level_resets_mana_spent_and_percent() {
        let mut p = Player::new(1, "Alon", 1);
        // Add partial mana (500 < 1600=needed(0)) to accumulate some mana_spent
        p.add_mana_spent_advance(500);
        p.set_magic_level(5);
        assert_eq!(p.get_magic_level(), 5);
        assert_eq!(p.get_mana_spent(), 0);
        assert_eq!(p.get_magic_level_percent(), 0);
    }

    #[test]
    fn add_mana_spent_advance_zero_returns_false() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(!p.add_mana_spent_advance(0));
        assert_eq!(p.get_magic_level(), 0);
    }

    #[test]
    fn add_mana_spent_advance_partial_updates_percent() {
        let mut p = Player::new(1, "Alon", 1);
        // magic_mana_needed(0) = 1600; 800 = 50%
        p.add_mana_spent_advance(800);
        assert_eq!(p.get_magic_level(), 0);
        assert_eq!(p.get_mana_spent(), 800);
        assert_eq!(p.get_magic_level_percent(), 50);
    }

    #[test]
    fn add_mana_spent_advance_exact_threshold_levels_up() {
        let mut p = Player::new(1, "Alon", 1);
        let advanced = p.add_mana_spent_advance(1600);
        assert!(advanced);
        assert_eq!(p.get_magic_level(), 1);
        assert_eq!(p.get_mana_spent(), 0);
    }

    #[test]
    fn add_mana_spent_advance_excess_carries_over() {
        let mut p = Player::new(1, "Alon", 1);
        // needed(0)=1600; add 2000: level-up + 400 carried over
        p.add_mana_spent_advance(2000);
        assert_eq!(p.get_magic_level(), 1);
        assert_eq!(p.get_mana_spent(), 400);
    }

    #[test]
    fn add_mana_spent_advance_multiple_levels_at_once() {
        let mut p = Player::new(1, "Alon", 1);
        // needed(0)=1600, needed(1)=12800; 14400 crosses both
        let advanced = p.add_mana_spent_advance(14_400);
        assert!(advanced);
        assert_eq!(p.get_magic_level(), 2);
    }

    #[test]
    fn check_magic_level_updates_percent_from_mana_spent() {
        let mut p = Player::new(1, "Alon", 1);
        // Accumulate 800 partial mana (needed(0)=1600)
        p.add_mana_spent_advance(800);
        // check_magic_level should agree with the percent already computed
        p.check_magic_level();
        // needed(0)=1600 -> 800*100/1600 = 50
        assert_eq!(p.get_magic_level_percent(), 50);
    }

    // -----------------------------------------------------------------------
    // Task 8.2 -- Stamina XP multiplier
    // -----------------------------------------------------------------------

    #[test]
    fn stamina_xp_zero_stamina_returns_zero() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(STAMINA_MAX);
        assert_eq!(p.get_stamina_xp_multiplier(), 0.0);
    }

    #[test]
    fn stamina_xp_exhausted_returns_half() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(STAMINA_MAX - STAMINA_EXHAUSTED_THRESHOLD);
        assert_eq!(p.get_stamina(), STAMINA_EXHAUSTED_THRESHOLD);
        assert_eq!(p.get_stamina_xp_multiplier(), 0.5);
    }

    #[test]
    fn stamina_xp_normal_range_returns_one() {
        let mut p = Player::new(1, "Alon", 1);
        p.drain_stamina(STAMINA_MAX - 1200);
        assert_eq!(p.get_stamina_xp_multiplier(), 1.0);
    }

    #[test]
    fn stamina_xp_premium_bonus_returns_one_point_five() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_premium(true);
        // Full stamina 2520 > STAMINA_BONUS_ABOVE (2340) + premium
        assert_eq!(p.get_stamina_xp_multiplier(), 1.5);
    }

    #[test]
    fn stamina_xp_above_bonus_non_premium_returns_one() {
        let p = Player::new(1, "Alon", 1);
        // No premium -> 1.0 even at full stamina
        assert_eq!(p.get_stamina_xp_multiplier(), 1.0);
    }

    // -----------------------------------------------------------------------
    // Task 8.2 -- HP/MP regen tick
    // -----------------------------------------------------------------------

    #[test]
    fn add_hp_regen_increases_health() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_health(50);
        p.add_hp_regen(20);
        assert_eq!(p.get_health(), 70);
    }

    #[test]
    fn add_hp_regen_capped_at_max_health() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_hp_regen(999);
        assert_eq!(p.get_health(), p.get_max_health());
    }

    #[test]
    fn add_mp_regen_increases_mana() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_max_mana(200);
        p.set_mana(50);
        p.add_mp_regen(30);
        assert_eq!(p.get_mana(), 80);
    }

    #[test]
    fn add_mp_regen_capped_at_max_mana() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_max_mana(100);
        p.set_mana(90);
        p.add_mp_regen(999);
        assert_eq!(p.get_mana(), 100);
    }

    // -----------------------------------------------------------------------
    // Task 8.2 -- StatsSnapshot (sendStats data shape)
    // -----------------------------------------------------------------------

    #[test]
    fn stats_snapshot_reflects_player_state() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_health(75);
        p.set_max_mana(200);
        p.set_mana(150);
        p.add_experience(100);
        let snap = p.stats_snapshot();
        assert_eq!(snap.health, 75);
        assert_eq!(snap.max_health, p.get_max_health());
        assert_eq!(snap.mana, 150);
        assert_eq!(snap.max_mana, 200);
        assert_eq!(snap.capacity, p.get_capacity());
        assert_eq!(snap.experience, 100);
        assert_eq!(snap.level, 1);
        assert_eq!(snap.level_percent, 0);
        assert_eq!(snap.stamina, STAMINA_MAX);
        assert_eq!(snap.magic_level, 0);
        assert_eq!(snap.speed, p.get_speed());
        assert_eq!(snap.soul, p.get_soul());
    }

    #[test]
    fn stats_snapshot_speed_is_clamped() {
        let p = Player::new(1, "Alon", 1);
        let snap = p.stats_snapshot();
        assert!(snap.speed >= PLAYER_MIN_SPEED);
        assert!(snap.speed <= PLAYER_MAX_SPEED);
    }

    // Task 8.3 tests — get_weapon_type

    #[test]
    fn get_weapon_type_no_weapon_returns_none() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_weapon_type(), WT::None);
    }

    #[test]
    fn get_weapon_type_sword_equipped() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_inventory_item(InventorySlot::Right, make_equip(sp::RIGHT, WT::Sword, 50));
        assert_eq!(p.get_weapon_type(), WT::Sword);
    }

    #[test]
    fn get_weapon_type_axe_equipped() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_inventory_item(InventorySlot::Right, make_equip(sp::RIGHT, WT::Axe, 60));
        assert_eq!(p.get_weapon_type(), WT::Axe);
    }

    #[test]
    fn get_weapon_type_distance_equipped() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_inventory_item(
            InventorySlot::Right,
            make_equip(sp::RIGHT, WT::Distance, 30),
        );
        assert_eq!(p.get_weapon_type(), WT::Distance);
    }

    // Task 8.3 tests — get_attack_speed

    #[test]
    fn get_attack_speed_no_weapon_uses_vocation() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_attack_speed(1500), 1500);
    }

    #[test]
    fn get_attack_speed_weapon_nonzero_speed() {
        let mut p = Player::new(1, "Alon", 1);
        let td = ItemTypeData {
            attack_speed: 800,
            weapon_type: WT::Sword,
            slot_position: sp::RIGHT,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Right, Item::new(Arc::new(td), 1));
        assert_eq!(p.get_attack_speed(1500), 800);
    }

    #[test]
    fn get_attack_speed_weapon_zero_speed_falls_back() {
        let mut p = Player::new(1, "Alon", 1);
        let td = ItemTypeData {
            weapon_type: WT::Sword,
            slot_position: sp::RIGHT,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Right, Item::new(Arc::new(td), 1));
        assert_eq!(p.get_attack_speed(1500), 1500);
    }

    // Task 8.3 tests — get_armor

    #[test]
    fn get_armor_empty_inventory_is_zero() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_armor(1.0), 0);
    }

    #[test]
    fn get_armor_sums_equipped_armor() {
        let mut p = Player::new(1, "Alon", 1);
        let head_td = ItemTypeData {
            armor: 3,
            slot_position: sp::HEAD,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Head, Item::new(Arc::new(head_td), 1));

        let body_td = ItemTypeData {
            armor: 10,
            slot_position: sp::ARMOR,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Armor, Item::new(Arc::new(body_td), 1));

        assert_eq!(p.get_armor(1.0), 13);
    }

    #[test]
    fn get_armor_applies_multiplier() {
        let mut p = Player::new(1, "Alon", 1);
        let td = ItemTypeData {
            armor: 10,
            slot_position: sp::ARMOR,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Armor, Item::new(Arc::new(td), 1));
        assert_eq!(p.get_armor(1.5), 15);
    }

    #[test]
    fn get_armor_ignores_weapon_slot() {
        let mut p = Player::new(1, "Alon", 1);
        let td = ItemTypeData {
            armor: 5,
            slot_position: sp::RIGHT,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Right, Item::new(Arc::new(td), 1));
        assert_eq!(p.get_armor(1.0), 0);
    }

    // Task 8.3 tests — get_raw_defense

    #[test]
    fn get_raw_defense_unarmed_returns_seven() {
        let p = Player::new(1, "Alon", 1);
        let (val, has_shield) = p.get_raw_defense();
        assert_eq!(val, 7);
        assert!(!has_shield);
    }

    #[test]
    fn get_raw_defense_weapon_only() {
        let mut p = Player::new(1, "Alon", 1);
        let td = ItemTypeData {
            defense: 15,
            extra_defense: 3,
            weapon_type: WT::Sword,
            slot_position: sp::RIGHT,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Right, Item::new(Arc::new(td), 1));
        let (val, has_shield) = p.get_raw_defense();
        assert_eq!(val, 18);
        assert!(!has_shield);
    }

    #[test]
    fn get_raw_defense_shield_only() {
        let mut p = Player::new(1, "Alon", 1);
        let td = ItemTypeData {
            defense: 20,
            weapon_type: WT::Shield,
            slot_position: sp::LEFT,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Left, Item::new(Arc::new(td), 1));
        let (val, has_shield) = p.get_raw_defense();
        assert_eq!(val, 20);
        assert!(has_shield);
    }

    #[test]
    fn get_raw_defense_shield_and_weapon() {
        let mut p = Player::new(1, "Alon", 1);
        let std_td = ItemTypeData {
            defense: 20,
            weapon_type: WT::Shield,
            slot_position: sp::LEFT,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Left, Item::new(Arc::new(std_td), 1));
        let w_td = ItemTypeData {
            defense: 10,
            extra_defense: 2,
            weapon_type: WT::Sword,
            slot_position: sp::RIGHT,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        p.set_inventory_item(InventorySlot::Right, Item::new(Arc::new(w_td), 1));
        let (val, has_shield) = p.get_raw_defense();
        assert_eq!(val, 22);
        assert!(has_shield);
    }

    // Task 8.3 tests — on_attacking / on_attacked

    #[test]
    fn on_attacking_sets_in_fight_flag() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(!p.is_in_fight());
        p.on_attacking();
        assert!(p.is_in_fight());
    }

    #[test]
    fn on_attacked_sets_in_fight_flag() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(!p.is_in_fight());
        p.on_attacked();
        assert!(p.is_in_fight());
    }

    #[test]
    fn clear_in_fight_resets_flag() {
        let mut p = Player::new(1, "Alon", 1);
        p.on_attacking();
        p.clear_in_fight();
        assert!(!p.is_in_fight());
    }

    // Task 8.3 tests — condition immunity

    #[test]
    fn no_condition_immunity_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_immune_to_condition(1 << 0));
    }

    #[test]
    fn add_condition_immunity_grants_it() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_condition_immunity(1 << 0);
        assert!(p.is_immune_to_condition(1 << 0));
    }

    #[test]
    fn add_multiple_condition_immunities() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_condition_immunity((1 << 0) | (1 << 1));
        assert!(p.is_immune_to_condition(1 << 0));
        assert!(p.is_immune_to_condition(1 << 1));
        assert!(!p.is_immune_to_condition(1 << 2));
    }

    #[test]
    fn remove_condition_immunity_removes_one() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_condition_immunity((1 << 0) | (1 << 1));
        p.remove_condition_immunity(1 << 0);
        assert!(!p.is_immune_to_condition(1 << 0));
        assert!(p.is_immune_to_condition(1 << 1));
    }

    #[test]
    fn get_condition_immunities_returns_bitmask() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_condition_immunity(1 << 5);
        assert_eq!(p.get_condition_immunities(), 1 << 5);
    }

    // Task 8.3 tests — add_unjustified_points

    #[test]
    fn add_unjustified_points_increments_kill_count() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_unjustified_points(1000, 3, 6);
        assert_eq!(p.get_unjustified_kills(), 1);
    }

    #[test]
    fn skull_none_below_red_threshold() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_unjustified_points(1000, 3, 6);
        assert_eq!(p.get_skull(), Skull::None);
    }

    #[test]
    fn skull_upgrades_to_red_at_threshold() {
        let mut p = Player::new(1, "Alon", 1);
        for _ in 0..3 {
            p.add_unjustified_points(1000, 3, 6);
        }
        assert_eq!(p.get_skull(), Skull::Red);
    }

    #[test]
    fn skull_upgrades_to_black_at_threshold() {
        let mut p = Player::new(1, "Alon", 1);
        for _ in 0..6 {
            p.add_unjustified_points(1000, 3, 6);
        }
        assert_eq!(p.get_skull(), Skull::Black);
    }

    #[test]
    fn skull_stays_black_after_reaching_it() {
        let mut p = Player::new(1, "Alon", 1);
        for _ in 0..7 {
            p.add_unjustified_points(1000, 3, 6);
        }
        assert_eq!(p.get_skull(), Skull::Black);
    }

    #[test]
    fn skull_disabled_when_kills_to_red_is_zero() {
        let mut p = Player::new(1, "Alon", 1);
        for _ in 0..10 {
            p.add_unjustified_points(1000, 0, 0);
        }
        assert_eq!(p.get_skull(), Skull::None);
    }

    // Task 8.3 tests — is_protection_zone / set_protection_zone

    #[test]
    fn is_protection_zone_false_by_default() {
        let p = Player::new(1, "Alon", 1);
        assert!(!p.is_protection_zone());
    }

    #[test]
    fn set_protection_zone_true_makes_is_true() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_protection_zone(true);
        assert!(p.is_protection_zone());
    }

    #[test]
    fn set_protection_zone_false_after_true() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_protection_zone(true);
        p.set_protection_zone(false);
        assert!(!p.is_protection_zone());
    }

    // -----------------------------------------------------------------------
    // Audit: tests for previously-uncovered getters/setters and edge paths
    // -----------------------------------------------------------------------

    #[test]
    fn set_max_health_stores_value() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_max_health(250);
        assert_eq!(p.get_max_health(), 250);
    }

    #[test]
    fn get_and_set_level_percent_roundtrip() {
        let mut p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_level_percent(), 0);
        p.set_level_percent(42);
        assert_eq!(p.get_level_percent(), 42);
    }

    #[test]
    fn set_level_percent_clamps_at_99() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_level_percent(200);
        assert_eq!(p.get_level_percent(), 99);
    }

    #[test]
    fn set_speed_within_range() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_speed(500);
        assert_eq!(p.get_speed(), 500);
    }

    #[test]
    fn set_speed_clamps_to_min() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_speed(1);
        assert_eq!(p.get_speed(), PLAYER_MIN_SPEED);
    }

    #[test]
    fn set_speed_clamps_to_max() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_speed(99_999);
        assert_eq!(p.get_speed(), PLAYER_MAX_SPEED);
    }

    #[test]
    fn set_magic_level_percent_roundtrip_and_clamp() {
        let mut p = Player::new(1, "Alon", 1);
        p.set_magic_level_percent(33);
        assert_eq!(p.get_magic_level_percent(), 33);
        p.set_magic_level_percent(500);
        assert_eq!(p.get_magic_level_percent(), 99);
    }

    #[test]
    fn set_condition_immunities_overwrites_bitmask() {
        let mut p = Player::new(1, "Alon", 1);
        p.add_condition_immunity(1 << 3);
        p.set_condition_immunities(1 << 7);
        assert_eq!(p.get_condition_immunities(), 1 << 7);
        assert!(!p.is_immune_to_condition(1 << 3));
        assert!(p.is_immune_to_condition(1 << 7));
    }

    #[test]
    fn is_in_protection_zone_and_set_in_protection_zone_roundtrip() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(!p.is_in_protection_zone());
        p.set_in_protection_zone(true);
        assert!(p.is_in_protection_zone());
        p.set_in_protection_zone(false);
        assert!(!p.is_in_protection_zone());
    }

    #[test]
    fn set_in_fight_roundtrip() {
        let mut p = Player::new(1, "Alon", 1);
        assert!(!p.is_in_fight());
        p.set_in_fight(true);
        assert!(p.is_in_fight());
        p.set_in_fight(false);
        assert!(!p.is_in_fight());
    }

    #[test]
    fn get_max_friends_default_is_200() {
        let p = Player::new(1, "Alon", 1);
        assert_eq!(p.get_max_friends(), 200);
    }

    // ------- query_add: previously-uncovered branches ---------

    #[test]
    fn query_add_child_is_owner_overweight_returns_not_enough_capacity() {
        // CHILD_IS_OWNER flag with no NO_LIMIT, item heavier than capacity ->
        // exercises the `ReturnValue::NotEnoughCapacity` branch (line 1115).
        let p = player_with_capacity(50);
        assert_eq!(
            p.query_add(
                InventorySlot::Armor,
                &make_equip(sp::ARMOR, WT::None, 100),
                1,
                query_flags::CHILD_IS_OWNER,
            ),
            ReturnValue::NotEnoughCapacity
        );
    }

    #[test]
    fn query_add_depot_slot_accepts_any_item() {
        // Exercises the `InventorySlot::Depot => true` arm (line 1133).
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Depot,
                &make_equip(sp::HEAD, WT::None, 5),
                1,
                0,
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_right_two_hander_same_item_in_left_ok() {
        // When left slot holds the *same* item pointer we're asking to add to
        // right (a two-hander), the ptr::eq check is satisfied and the call
        // returns NoError (covers the falsy branch of `!ptr::eq` on line 1140).
        let mut p = player_with_capacity(1000);
        let two_hander = make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5);
        p.add_item_to_inventory(InventorySlot::Left, two_hander);
        // Re-borrow the same item now stored in the left slot, then query_add it
        // for the right slot — same pointer.
        let li_ptr: *const Item = p.get_inventory_item(InventorySlot::Left).unwrap();
        // SAFETY: `p` is only borrowed immutably below, so the pointer remains
        // valid for the duration of the call.
        let item_ref: &Item = unsafe { &*li_ptr };
        assert_eq!(
            p.query_add(InventorySlot::Right, item_ref, 1, 0),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_right_two_hander_distance_left_with_quiver_ok() {
        // Right slot, item is a Quiver (not two-hand). Left slot has a
        // two-hander Distance bow. Quiver+Distance combo => NoError on line 1148.
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Left,
            make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Distance, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Quiver, 5),
                1,
                0,
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_left_two_hander_same_item_in_right_ok() {
        // Mirror of the right-side ptr::eq path on the left side (line 1173).
        let mut p = player_with_capacity(1000);
        let two_hander = make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5);
        p.add_item_to_inventory(InventorySlot::Right, two_hander);
        let ri_ptr: *const Item = p.get_inventory_item(InventorySlot::Right).unwrap();
        let item_ref: &Item = unsafe { &*ri_ptr };
        assert_eq!(
            p.query_add(InventorySlot::Left, item_ref, 1, 0),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_left_two_hander_distance_with_quiver_right_ok() {
        // Left slot, item is two-hand Distance. Right slot has a Quiver.
        // Distance+Quiver combo => NoError on line 1169.
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Quiver, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Distance, 5),
                1,
                0,
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_left_two_hander_non_quiver_right_blocked() {
        // Left slot, item is two-hand Sword. Right has a Shield (not Quiver).
        // Returns BothHandsNeedToBeFree on line 1171.
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Sword, 5),
                1,
                0,
            ),
            ReturnValue::BothHandsNeedToBeFree
        );
    }

    #[test]
    fn query_add_left_right_two_hander_distance_quiver_ok() {
        // Left slot. Right has a two-hander Quiver. Adding Distance to left
        // is accepted (wt==Distance, rw==Quiver) on line 1180.
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT | sp::TWO_HAND, WT::Quiver, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Distance, 5),
                1,
                0,
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_left_two_shields_blocked() {
        // Left slot, adding a Shield while Right already holds a Shield.
        // Returns CanOnlyUseOneShield on line 1186.
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
                1,
                0,
            ),
            ReturnValue::CanOnlyUseOneShield
        );
    }

    #[test]
    fn query_add_left_right_empty_ok() {
        // Left slot, item is one-hand (not TWO_HAND), Right slot empty —
        // exercises the `None => true` arm of the Left ri_opt match (line 1191).
        let p = player_with_capacity(1000);
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Shield, 5),
                1,
                0,
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_right_non_conflicting_left_ok() {
        // Right slot, with left containing a non-shield non-melee item (e.g. Ammo/Quiver).
        // Falls through both the shield-shield (1149) and melee-melee (1152) blocks,
        // reaches the closing of the `if let Some(li)` (line 1154) and the `true` arm.
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Left,
            make_equip(sp::LEFT | sp::RIGHT, WT::Quiver, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Right,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Distance, 5),
                1,
                0,
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_left_non_conflicting_right_ok() {
        // Mirror on the left side: right has a non-shield non-melee item, left adds
        // a non-conflicting item, falling through to the `true` arm at line 1187/1188.
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Quiver, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Distance, 5),
                1,
                0,
            ),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_left_two_melee_blocked() {
        // Left slot, melee weapon while Right has a melee weapon => CanOnlyUseOneWeapon (line 1191).
        let mut p = player_with_capacity(1000);
        p.add_item_to_inventory(
            InventorySlot::Right,
            make_equip(sp::LEFT | sp::RIGHT, WT::Sword, 5),
        );
        assert_eq!(
            p.query_add(
                InventorySlot::Left,
                &make_equip(sp::LEFT | sp::RIGHT, WT::Axe, 5),
                1,
                0,
            ),
            ReturnValue::CanOnlyUseOneWeapon
        );
    }

    // ------- AttrError Display + low-level decoder edge cases ---------

    #[test]
    fn attr_error_display_for_each_variant() {
        assert_eq!(
            format!("{}", AttrError::UnexpectedEof),
            "unexpected end of attribute stream"
        );
        assert_eq!(
            format!("{}", AttrError::UnknownTag(0x99)),
            "unknown attribute tag: 0x99"
        );
        assert_eq!(
            format!("{}", AttrError::InvalidStringLength),
            "invalid string length in attribute stream"
        );
    }

    #[test]
    fn unserialize_attr_login_position_truncated_returns_eof() {
        // A LoginPosition tag (0x02) followed by only 1 byte triggers the
        // read_u16_le EOF guard at line 1991.
        let bad_blob = vec![PlayerAttr::LoginPosition as u8, 0x00];
        let mut p = Player::new(1, "Hero", 1);
        assert_eq!(p.unserialize_attr(&bad_blob), Err(AttrError::UnexpectedEof));
    }

    #[test]
    fn vip_entry_from_bytes_invalid_string_length_returns_error() {
        // Build a VIPEntry blob then corrupt the name length so that
        // start+len > buf.len(), triggering `InvalidStringLength` at line 2010.
        let original = VIPEntry::new(1, "ABC", "", 0, false);
        let mut bytes = original.to_bytes();
        // Layout: guid(4) | icon(4) | notify(1) | name_len(2 LE) | name
        // name_len lives at bytes[9..11]. Set it to an absurd value.
        bytes[9] = 0xFF;
        bytes[10] = 0xFF;
        let result = VIPEntry::from_bytes(&bytes, 0);
        assert_eq!(result, Err(AttrError::InvalidStringLength));
    }
}
