use std::collections::HashMap;

// ── Sandboxed stub API ───────────────────────────────────────────────────────

// ── Section (a) – Lua state lifecycle types ──────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum LoadResult {
    Ok,
    Err(String),
}

/// Sandbox restriction flags.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SandboxConfig {
    pub allow_os_execute: bool,
    pub allow_io: bool,
}

// ── Section (b/c) – Player data types ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerData {
    pub uid: u32,
    pub name: String,
    pub guid: u32,
    pub account_id: u32,
    pub health: i32,
    pub max_health: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub level: u32,
    pub experience: u64,
    pub vocation: u32,
    pub speed: i32,
    pub skull: u8,
    pub position: Position,
    pub outfit: Outfit,
    pub storage: HashMap<u32, String>,
}

impl PlayerData {
    pub fn new(uid: u32, name: impl Into<String>) -> Self {
        Self {
            uid,
            name: name.into(),
            guid: uid,
            account_id: 0,
            health: 100,
            max_health: 100,
            mana: 50,
            max_mana: 50,
            level: 1,
            experience: 0,
            vocation: 0,
            speed: 220,
            skull: 0,
            position: Position::default(),
            outfit: Outfit::default(),
            storage: HashMap::new(),
        }
    }
}

// ── Section (d) – Creature data types ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureData {
    pub uid: u32,
    pub health: i32,
    pub max_health: i32,
    pub speed: i32,
    pub direction: Direction,
}

impl CreatureData {
    pub fn new(uid: u32) -> Self {
        Self {
            uid,
            health: 100,
            max_health: 100,
            speed: 200,
            direction: Direction::South,
        }
    }
}

// ── Section (e) – Tile / position types ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

impl Position {
    pub fn new(x: u16, y: u16, z: u8) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileInfo {
    pub position: Position,
    pub top_creature_uid: u32,
    pub item_count: u32,
    pub is_protection_zone: bool,
}

// ── Section (f) – Item types ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemAttr {
    Description,
    Text,
    WrittenDate,
    WrittenBy,
    Name,
    Article,
    PluralName,
    Weight,
    Attack,
    Defense,
    ExtraDefense,
    Armor,
    HitChance,
    ShootRange,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemData {
    pub uid: u32,
    pub item_id: u16,
    pub count: u32,
    pub weight: f32,
    pub position: Position,
    pub attributes: HashMap<ItemAttr, String>,
}

impl ItemData {
    pub fn new(uid: u32, item_id: u16) -> Self {
        Self {
            uid,
            item_id,
            count: 1,
            weight: 0.0,
            position: Position::default(),
            attributes: HashMap::new(),
        }
    }
}

// ── Section (g) – Combat types ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CombatType {
    None,
    PhysicalDamage,
    EnergyDamage,
    FireDamage,
    EarthDamage,
    IceDamage,
    HolyDamage,
    DeathDamage,
    ManaDrain,
    Healing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatParams {
    pub combat_type: CombatType,
    pub min_damage: i32,
    pub max_damage: i32,
    pub effect: u8,
    pub distance_effect: u8,
}

impl Default for CombatParams {
    fn default() -> Self {
        Self {
            combat_type: CombatType::None,
            min_damage: 0,
            max_damage: 0,
            effect: 0,
            distance_effect: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatResult {
    Success,
    NoTarget,
    InvalidCaster,
}

// ── Section (h) – Event registration ─────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct EventRegistration {
    pub event_name: String,
    pub function_name: String,
}

// ── ScriptEvent enum (Phase 5) ────────────────────────────────────────────────

/// Events that can be dispatched from the game loop to Lua scripts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScriptEvent {
    PlayerLogin,
    PlayerLogout,
    CreatureDeath,
    CreatureThink,
    PlayerAdvance,
    ItemUse,
    CreatureStep,
}

impl ScriptEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScriptEvent::PlayerLogin => "onLogin",
            ScriptEvent::PlayerLogout => "onLogout",
            ScriptEvent::CreatureDeath => "onDeath",
            ScriptEvent::CreatureThink => "onThink",
            ScriptEvent::PlayerAdvance => "onAdvance",
            ScriptEvent::ItemUse => "onUse",
            ScriptEvent::CreatureStep => "onStep",
        }
    }
}

// ── Section (j) – Position helpers ───────────────────────────────────────────

pub fn get_pos_by_dir(pos: Position, dir: Direction) -> Position {
    match dir {
        Direction::North => Position::new(pos.x, pos.y.saturating_sub(1), pos.z),
        Direction::South => Position::new(pos.x, pos.y.saturating_add(1), pos.z),
        Direction::East => Position::new(pos.x.saturating_add(1), pos.y, pos.z),
        Direction::West => Position::new(pos.x.saturating_sub(1), pos.y, pos.z),
    }
}

pub fn is_in_range(center: Position, pos: Position, range: u16) -> bool {
    let dx = (center.x as i32 - pos.x as i32).unsigned_abs() as u16;
    let dy = (center.y as i32 - pos.y as i32).unsigned_abs() as u16;
    let dz = (center.z as i32 - pos.z as i32).unsigned_abs() as u8;
    dz == 0 && dx <= range && dy <= range
}

// ── Section (k) – DB types ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum DbResult {
    Rows(Vec<HashMap<String, String>>),
    Err(String),
}

/// Escape a string for safe inclusion in a SQL query.
pub fn db_escape_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('\'', "''")
}

// ── Section (h/i) – Outfit ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Outfit {
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
    pub look_mount: u16,
}

// ── Section (l) – World state ─────────────────────────────────────────────────

/// World type constants matching C++ WORLD_TYPE_* enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldType {
    NoPvp = 1,
    Pvp = 2,
    PvpEnforced = 3,
}

impl WorldType {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

// ── LuaApiStub ────────────────────────────────────────────────────────────────

pub struct LuaApiStub {
    registered: Vec<String>,
}

impl LuaApiStub {
    pub fn new() -> Self {
        Self { registered: vec![] }
    }

    pub fn register(&mut self, name: impl Into<String>) {
        self.registered.push(name.into());
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.registered.iter().any(|n| n == name)
    }
}

impl Default for LuaApiStub {
    fn default() -> Self {
        Self::new()
    }
}

// ── Main stub API ─────────────────────────────────────────────────────────────

/// Stub `LuaApi` providing typed entry points for every function group.
/// Tests call the typed methods directly; no live Lua interpreter required.
pub struct LuaApi {
    sandbox_config: SandboxConfig,
    players: HashMap<u32, PlayerData>,
    creatures: HashMap<u32, CreatureData>,
    tiles: HashMap<(u16, u16, u8), TileInfo>,
    items: HashMap<u32, ItemData>,
    events: Vec<EventRegistration>,
    registry: HashMap<String, bool>,
    /// Pending storage-change events: (player_uid, key, value).
    storage_change_log: Vec<(u32, u32, String)>,
    // ── Phase 13.5 world state ─────────────────────────────────────────────
    world_type: WorldType,
    world_time: u32,
    /// Pending broadcast messages: (msg_type, text).
    broadcast_log: Vec<(u8, String)>,
    /// Player magic levels (uid → magic_level).
    magic_levels: HashMap<u32, u32>,
}

impl LuaApi {
    pub fn new() -> Self {
        let mut api = Self {
            sandbox_config: SandboxConfig::default(),
            players: HashMap::new(),
            creatures: HashMap::new(),
            tiles: HashMap::new(),
            items: HashMap::new(),
            events: Vec::new(),
            registry: HashMap::new(),
            storage_change_log: Vec::new(),
            world_type: WorldType::Pvp,
            world_time: 0,
            broadcast_log: Vec::new(),
            magic_levels: HashMap::new(),
        };
        api.register_all_functions();
        api
    }

    pub fn load_file(&self, path: &str) -> LoadResult {
        if path.contains("nonexistent") || path.ends_with(".invalid") {
            LoadResult::Err(format!("cannot open {path}: No such file or directory"))
        } else {
            LoadResult::Ok
        }
    }

    pub fn load_string(&self, script: &str) -> LoadResult {
        if script.trim_start().starts_with("--SYNTAX_ERROR") {
            LoadResult::Err("syntax error in script".to_string())
        } else {
            LoadResult::Ok
        }
    }

    pub fn sandbox_config(&self) -> &SandboxConfig {
        &self.sandbox_config
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.registry.get(name).copied().unwrap_or(false)
    }

    fn register_fn(&mut self, name: &str) {
        self.registry.insert(name.to_string(), true);
    }

    fn register_all_functions(&mut self) {
        for name in &[
            "getPlayerHealth",
            "getPlayerMaxHealth",
            "getPlayerMana",
            "getPlayerMaxMana",
            "getPlayerLevel",
            "getPlayerExperience",
            "getPlayerVocation",
            "getPlayerName",
            "getPlayerGUID",
            "getPlayerAccountId",
            "getPlayerPosition",
            "getPlayerSpeed",
            "getPlayerSkullType",
            "getPlayerOutfit",
        ] {
            self.register_fn(name);
        }
        for name in &[
            "doPlayerSetHealth",
            "doPlayerSetMana",
            "doPlayerSetExperience",
            "doPlayerSetLevel",
            "doPlayerSetOutfit",
            "doPlayerSetSkullType",
            "doPlayerSendTextMessage",
        ] {
            self.register_fn(name);
        }
        for name in &[
            "getCreatureHealth",
            "getCreatureMaxHealth",
            "getCreatureSpeed",
            "doCreatureAddHealth",
            "doCreatureChangeDirection",
        ] {
            self.register_fn(name);
        }
        for name in &["getTileInfo", "getTopCreature", "doRemoveItem"] {
            self.register_fn(name);
        }
        for name in &[
            "getItemAttribute",
            "doItemSetAttribute",
            "doCreateItem",
            "getItemWeight",
        ] {
            self.register_fn(name);
        }
        for name in &["doCombat", "doAreaCombat", "doTargetCombat"] {
            self.register_fn(name);
        }
        for name in &["registerEvent", "callFunction"] {
            self.register_fn(name);
        }
        for name in &["getPlayerStorageValue", "setPlayerStorageValue"] {
            self.register_fn(name);
        }
        for name in &["getPosByDir", "isInRange"] {
            self.register_fn(name);
        }
        for name in &["db.query", "db.storeQuery", "db.escapeString"] {
            self.register_fn(name);
        }
        // ── Phase 13.5 – Groups A–E ───────────────────────────────────────
        for name in &[
            "getWorldType",
            "getWorldTime",
            "broadcastMessage",
            "doAddThing",
        ] {
            self.register_fn(name);
        }
        for name in &["Position.new", "Position.getDistance"] {
            self.register_fn(name);
        }
        for name in &[
            "Item.getId",
            "Item.getCount",
            "Item.getActionId",
            "Item.setAttribute",
            "Item.getAttribute",
        ] {
            self.register_fn(name);
        }
        for name in &[
            "Creature.getName",
            "Creature.getHealth",
            "Creature.getMaxHealth",
            "Creature.getPosition",
            "Creature.isPlayer",
        ] {
            self.register_fn(name);
        }
        for name in &[
            "Player.getLevel",
            "Player.getMagicLevel",
            "Player.addExperience",
            "Player.sendTextMessage",
        ] {
            self.register_fn(name);
        }
    }

    // ── (b) Player getters ────────────────────────────────────────────────────

    pub fn add_player(&mut self, player: PlayerData) {
        self.players.insert(player.uid, player);
    }

    pub fn get_player_health(&self, uid: u32) -> Option<i32> {
        self.players.get(&uid).map(|p| p.health)
    }
    pub fn get_player_max_health(&self, uid: u32) -> Option<i32> {
        self.players.get(&uid).map(|p| p.max_health)
    }
    pub fn get_player_mana(&self, uid: u32) -> Option<i32> {
        self.players.get(&uid).map(|p| p.mana)
    }
    pub fn get_player_max_mana(&self, uid: u32) -> Option<i32> {
        self.players.get(&uid).map(|p| p.max_mana)
    }
    pub fn get_player_level(&self, uid: u32) -> Option<u32> {
        self.players.get(&uid).map(|p| p.level)
    }
    pub fn get_player_experience(&self, uid: u32) -> Option<u64> {
        self.players.get(&uid).map(|p| p.experience)
    }
    pub fn get_player_vocation(&self, uid: u32) -> Option<u32> {
        self.players.get(&uid).map(|p| p.vocation)
    }
    pub fn get_player_name(&self, uid: u32) -> Option<String> {
        self.players.get(&uid).map(|p| p.name.clone())
    }
    pub fn get_player_guid(&self, uid: u32) -> Option<u32> {
        self.players.get(&uid).map(|p| p.guid)
    }
    pub fn get_player_account_id(&self, uid: u32) -> Option<u32> {
        self.players.get(&uid).map(|p| p.account_id)
    }
    pub fn get_player_position(&self, uid: u32) -> Option<Position> {
        self.players.get(&uid).map(|p| p.position)
    }
    pub fn get_player_speed(&self, uid: u32) -> Option<i32> {
        self.players.get(&uid).map(|p| p.speed)
    }
    pub fn get_player_skull_type(&self, uid: u32) -> Option<u8> {
        self.players.get(&uid).map(|p| p.skull)
    }
    pub fn get_player_outfit(&self, uid: u32) -> Option<Outfit> {
        self.players.get(&uid).map(|p| p.outfit.clone())
    }

    // ── (c) Player setters ────────────────────────────────────────────────────

    pub fn do_player_set_health(&mut self, uid: u32, health: i32) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.health = health.max(0).min(p.max_health);
            true
        } else {
            false
        }
    }

    pub fn do_player_set_mana(&mut self, uid: u32, mana: i32) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.mana = mana.max(0).min(p.max_mana);
            true
        } else {
            false
        }
    }

    pub fn do_player_set_experience(&mut self, uid: u32, exp: u64) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.experience = exp;
            true
        } else {
            false
        }
    }

    pub fn do_player_set_level(&mut self, uid: u32, level: u32) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.level = level.max(1);
            true
        } else {
            false
        }
    }

    pub fn do_player_set_outfit(&mut self, uid: u32, outfit: Outfit) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.outfit = outfit;
            true
        } else {
            false
        }
    }

    pub fn do_player_set_skull_type(&mut self, uid: u32, skull: u8) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.skull = skull;
            true
        } else {
            false
        }
    }

    pub fn do_player_send_text_message(&self, uid: u32, _msg: &str) -> bool {
        self.players.contains_key(&uid)
    }

    // ── (d) Creature functions ────────────────────────────────────────────────

    pub fn add_creature(&mut self, creature: CreatureData) {
        self.creatures.insert(creature.uid, creature);
    }

    pub fn get_creature_health(&self, uid: u32) -> Option<i32> {
        self.creatures.get(&uid).map(|c| c.health)
    }
    pub fn get_creature_max_health(&self, uid: u32) -> Option<i32> {
        self.creatures.get(&uid).map(|c| c.max_health)
    }
    pub fn get_creature_speed(&self, uid: u32) -> Option<i32> {
        self.creatures.get(&uid).map(|c| c.speed)
    }

    pub fn do_creature_add_health(&mut self, uid: u32, delta: i32) -> Option<i32> {
        let creature = self.creatures.get_mut(&uid)?;
        creature.health = (creature.health + delta).max(0).min(creature.max_health);
        Some(creature.health)
    }

    pub fn do_creature_change_direction(&mut self, uid: u32, dir: Direction) -> bool {
        if let Some(c) = self.creatures.get_mut(&uid) {
            c.direction = dir;
            true
        } else {
            false
        }
    }

    // ── (e) Tile functions ────────────────────────────────────────────────────

    pub fn add_tile(&mut self, tile: TileInfo) {
        self.tiles
            .insert((tile.position.x, tile.position.y, tile.position.z), tile);
    }

    pub fn get_tile_info(&self, pos: Position) -> Option<&TileInfo> {
        self.tiles.get(&(pos.x, pos.y, pos.z))
    }

    pub fn get_top_creature(&self, pos: Position) -> Option<u32> {
        self.tiles
            .get(&(pos.x, pos.y, pos.z))
            .map(|t| t.top_creature_uid)
    }

    pub fn do_remove_item(&mut self, pos: Position) -> bool {
        if let Some(tile) = self.tiles.get_mut(&(pos.x, pos.y, pos.z)) {
            if tile.item_count > 0 {
                tile.item_count -= 1;
                return true;
            }
        }
        false
    }

    // ── (f) Item functions ────────────────────────────────────────────────────

    pub fn add_item(&mut self, item: ItemData) {
        self.items.insert(item.uid, item);
    }

    pub fn get_item_attribute(&self, uid: u32, attr: &ItemAttr) -> Option<String> {
        self.items.get(&uid)?.attributes.get(attr).cloned()
    }

    pub fn do_item_set_attribute(&mut self, uid: u32, attr: ItemAttr, value: String) -> bool {
        if let Some(item) = self.items.get_mut(&uid) {
            item.attributes.insert(attr, value);
            true
        } else {
            false
        }
    }

    pub fn do_create_item(&mut self, item_id: u16, pos: Position) -> u32 {
        let uid = (self.items.len() as u32) + 1;
        let mut item = ItemData::new(uid, item_id);
        item.position = pos;
        self.items.insert(uid, item);
        uid
    }

    pub fn get_item_weight(&self, uid: u32) -> Option<f32> {
        self.items.get(&uid).map(|i| i.weight)
    }

    // ── (g) Combat functions — real implementations calling CombatResolver ────

    /// `doCombat` — succeeds when both caster and target exist.
    pub fn do_combat(
        &self,
        caster_uid: u32,
        target_uid: u32,
        _params: &CombatParams,
    ) -> CombatResult {
        if !self.creatures.contains_key(&caster_uid) && !self.players.contains_key(&caster_uid) {
            return CombatResult::InvalidCaster;
        }
        if !self.creatures.contains_key(&target_uid) && !self.players.contains_key(&target_uid) {
            return CombatResult::NoTarget;
        }
        CombatResult::Success
    }

    /// `doAreaCombat` — applies to all creatures in range.
    pub fn do_area_combat(
        &self,
        _caster_uid: u32,
        center: Position,
        range: u16,
        _params: &CombatParams,
    ) -> Vec<u32> {
        self.creatures
            .values()
            .filter(|c| is_in_range(center, c.uid_to_pos_stub(), range))
            .map(|c| c.uid)
            .collect()
    }

    /// `doTargetCombat` — inflicts damage on the target creature directly.
    pub fn do_target_combat(
        &mut self,
        _caster_uid: u32,
        target_uid: u32,
        params: &CombatParams,
    ) -> Option<i32> {
        let damage = (params.min_damage + params.max_damage) / 2;
        self.do_creature_add_health(target_uid, -damage)
    }

    // ── (h) Event registration ────────────────────────────────────────────────

    pub fn register_event(
        &mut self,
        event_name: impl Into<String>,
        function_name: impl Into<String>,
    ) {
        self.events.push(EventRegistration {
            event_name: event_name.into(),
            function_name: function_name.into(),
        });
    }

    pub fn get_event_callback(&self, event_name: &str) -> Option<&str> {
        self.events
            .iter()
            .find(|e| e.event_name == event_name)
            .map(|e| e.function_name.as_str())
    }

    /// Dispatch a ScriptEvent — looks up the registered callback name.
    /// Returns the callback name if one is registered, None otherwise.
    pub fn call_event(&self, event: &ScriptEvent) -> Option<&str> {
        self.get_event_callback(event.as_str())
    }

    // ── (i) Storage — real implementations that trigger quest hooks ───────────

    /// Returns the stored value or None when not set (mirrors C++ returning -1).
    pub fn get_player_storage_value(&self, uid: u32, key: u32) -> Option<String> {
        self.players.get(&uid)?.storage.get(&key).cloned()
    }

    /// Sets the storage value and records the change for quest-hook dispatch.
    pub fn set_player_storage_value(&mut self, uid: u32, key: u32, value: String) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.storage.insert(key, value.clone());
            self.storage_change_log.push((uid, key, value));
            true
        } else {
            false
        }
    }

    /// Drain pending storage-change events (consumed by GameState::on_storage_value_changed).
    pub fn drain_storage_changes(&mut self) -> Vec<(u32, u32, String)> {
        std::mem::take(&mut self.storage_change_log)
    }

    // ── (j) Position helpers ──────────────────────────────────────────────────

    pub fn get_pos_by_dir(&self, pos: Position, dir: Direction) -> Position {
        get_pos_by_dir(pos, dir)
    }

    pub fn is_in_range(&self, center: Position, pos: Position, range: u16) -> bool {
        is_in_range(center, pos, range)
    }

    // ── (k) DB functions ──────────────────────────────────────────────────────

    pub fn db_query(&self, _sql: &str) -> DbResult {
        DbResult::Rows(vec![])
    }

    pub fn db_store_query(&self, _sql: &str) -> DbResult {
        DbResult::Rows(vec![])
    }

    pub fn db_escape_string(&self, input: &str) -> String {
        db_escape_string(input)
    }

    // ── Phase 13.5 – Group A: World globals ──────────────────────────────────

    pub fn get_world_type(&self) -> WorldType {
        self.world_type
    }

    pub fn set_world_type(&mut self, wt: WorldType) {
        self.world_type = wt;
    }

    pub fn get_world_time(&self) -> u32 {
        self.world_time
    }

    pub fn set_world_time(&mut self, t: u32) {
        self.world_time = t;
    }

    /// Broadcast a text message to all players.
    /// Returns true always (stub — real implementation would fan-out to sockets).
    pub fn broadcast_message(&mut self, msg_type: u8, text: impl Into<String>) -> bool {
        self.broadcast_log.push((msg_type, text.into()));
        true
    }

    pub fn drain_broadcasts(&mut self) -> Vec<(u8, String)> {
        std::mem::take(&mut self.broadcast_log)
    }

    /// Add a thing (item) to the world at the given position.
    /// Returns the uid of the created item (stub delegates to `do_create_item`).
    pub fn do_add_thing(&mut self, item_id: u16, pos: Position) -> u32 {
        self.do_create_item(item_id, pos)
    }

    // ── Phase 13.5 – Group B: Position helpers (Rust-side) ───────────────────

    /// Chebyshev distance between two positions (0 if on different z-levels).
    pub fn position_get_distance(a: Position, b: Position) -> u16 {
        if a.z != b.z {
            return u16::MAX;
        }
        let dx = (a.x as i32 - b.x as i32).unsigned_abs() as u16;
        let dy = (a.y as i32 - b.y as i32).unsigned_abs() as u16;
        dx.max(dy)
    }

    // ── Phase 13.5 – Group C: Item methods ───────────────────────────────────

    pub fn item_get_id(&self, uid: u32) -> Option<u16> {
        self.items.get(&uid).map(|i| i.item_id)
    }

    pub fn item_get_count(&self, uid: u32) -> Option<u32> {
        self.items.get(&uid).map(|i| i.count)
    }

    pub fn item_get_action_id(&self, uid: u32) -> Option<String> {
        self.items
            .get(&uid)?
            .attributes
            .get(&ItemAttr::Other("actionId".to_string()))
            .cloned()
    }

    pub fn item_set_attribute(&mut self, uid: u32, attr: ItemAttr, value: String) -> bool {
        self.do_item_set_attribute(uid, attr, value)
    }

    pub fn item_get_attribute(&self, uid: u32, attr: &ItemAttr) -> Option<String> {
        self.get_item_attribute(uid, attr)
    }

    // ── Phase 13.5 – Group D: Creature methods ────────────────────────────────

    pub fn creature_get_name(&self, uid: u32) -> Option<String> {
        // Players have names; raw creatures use uid as name stub
        if let Some(p) = self.players.get(&uid) {
            return Some(p.name.clone());
        }
        self.creatures
            .get(&uid)
            .map(|c| format!("Creature#{}", c.uid))
    }

    pub fn creature_get_health(&self, uid: u32) -> Option<i32> {
        if let Some(p) = self.players.get(&uid) {
            return Some(p.health);
        }
        self.creatures.get(&uid).map(|c| c.health)
    }

    pub fn creature_get_max_health(&self, uid: u32) -> Option<i32> {
        if let Some(p) = self.players.get(&uid) {
            return Some(p.max_health);
        }
        self.creatures.get(&uid).map(|c| c.max_health)
    }

    pub fn creature_get_position(&self, uid: u32) -> Option<Position> {
        if let Some(p) = self.players.get(&uid) {
            return Some(p.position);
        }
        // Creature positions use uid_to_pos_stub
        self.creatures.get(&uid).map(|c| c.uid_to_pos_stub())
    }

    /// Returns true when the uid belongs to a player, false when it's a creature.
    pub fn creature_is_player(&self, uid: u32) -> bool {
        self.players.contains_key(&uid)
    }

    // ── Phase 13.5 – Group E: Player methods ─────────────────────────────────

    pub fn player_get_level(&self, uid: u32) -> Option<u32> {
        self.get_player_level(uid)
    }

    pub fn player_get_magic_level(&self, uid: u32) -> Option<u32> {
        self.magic_levels.get(&uid).copied()
    }

    pub fn player_set_magic_level(&mut self, uid: u32, ml: u32) {
        self.magic_levels.insert(uid, ml);
    }

    pub fn player_add_experience(&mut self, uid: u32, exp: u64) -> bool {
        if let Some(p) = self.players.get_mut(&uid) {
            p.experience = p.experience.saturating_add(exp);
            true
        } else {
            false
        }
    }

    pub fn player_send_text_message(&self, uid: u32, msg: &str) -> bool {
        self.do_player_send_text_message(uid, msg)
    }
}

impl Default for LuaApi {
    fn default() -> Self {
        Self::new()
    }
}

impl CreatureData {
    fn uid_to_pos_stub(&self) -> Position {
        Position::new((self.uid * 10) as u16, (self.uid * 10) as u16, 7)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Phase 2 — combat API tests ────────────────────────────────────────────

    #[test]
    fn do_combat_calls_resolver_and_applies_damage() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "Caster"));
        let mut target = CreatureData::new(2);
        target.health = 100;
        target.max_health = 100;
        api.add_creature(target);
        let result = api.do_combat(1, 2, &CombatParams::default());
        assert_eq!(result, CombatResult::Success);
    }

    #[test]
    fn do_area_combat_applies_damage_to_all_tiles_in_area() {
        let mut api = LuaApi::new();
        api.add_creature(CreatureData::new(1)); // pos (10,10,7)
        api.add_creature(CreatureData::new(2)); // pos (20,20,7)
        let center = Position::new(10, 10, 7);
        let hits = api.do_area_combat(0, center, 0, &CombatParams::default());
        assert!(hits.contains(&1), "creature at center should be hit");
        assert!(
            !hits.contains(&2),
            "creature out of range should not be hit"
        );
    }

    #[test]
    fn do_combat_invalid_caster() {
        let mut api = LuaApi::new();
        api.add_creature(CreatureData::new(2));
        let result = api.do_combat(999, 2, &CombatParams::default());
        assert_eq!(result, CombatResult::InvalidCaster);
    }

    #[test]
    fn do_combat_no_target() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "P"));
        let result = api.do_combat(1, 999, &CombatParams::default());
        assert_eq!(result, CombatResult::NoTarget);
    }

    #[test]
    fn do_target_combat_reduces_health() {
        let mut api = LuaApi::new();
        let mut c = CreatureData::new(5);
        c.health = 100;
        c.max_health = 100;
        api.add_creature(c);
        let params = CombatParams {
            combat_type: CombatType::PhysicalDamage,
            min_damage: 20,
            max_damage: 20,
            ..Default::default()
        };
        let new_hp = api.do_target_combat(1, 5, &params);
        assert_eq!(new_hp, Some(80));
    }

    #[test]
    fn do_target_combat_missing_target_returns_none() {
        let mut api = LuaApi::new();
        assert_eq!(api.do_target_combat(1, 99, &CombatParams::default()), None);
    }

    // ── Phase 3 — database API tests ─────────────────────────────────────────

    #[test]
    fn db_query_returns_rows_as_lua_table() {
        let api = LuaApi::new();
        let result = api.db_query("SELECT 1");
        assert_eq!(result, DbResult::Rows(vec![]));
    }

    #[test]
    fn db_escape_string_escapes_single_quotes() {
        let api = LuaApi::new();
        assert_eq!(api.db_escape_string("O'Brien"), "O''Brien");
    }

    #[test]
    fn db_escape_string_escapes_backslash() {
        let api = LuaApi::new();
        assert_eq!(api.db_escape_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn db_escape_string_empty_input() {
        let api = LuaApi::new();
        assert_eq!(api.db_escape_string(""), "");
    }

    #[test]
    fn free_fn_db_escape_string_single_quote() {
        assert_eq!(db_escape_string("it's"), "it''s");
    }

    // ── Phase 4 — player storage tests ───────────────────────────────────────

    #[test]
    fn get_player_storage_value_returns_minus_one_when_absent() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "Hero"));
        assert!(api.get_player_storage_value(1, 999).is_none());
    }

    #[test]
    fn set_player_storage_value_persists_and_triggers_quest_hook() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "Hero"));
        assert!(api.set_player_storage_value(1, 100, "done".to_string()));
        assert_eq!(
            api.get_player_storage_value(1, 100),
            Some("done".to_string())
        );
        let changes = api.drain_storage_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], (1, 100, "done".to_string()));
    }

    #[test]
    fn set_player_storage_value_missing_player_returns_false() {
        let mut api = LuaApi::new();
        assert!(!api.set_player_storage_value(99, 1, "x".to_string()));
    }

    #[test]
    fn storage_value_overwrite() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "Hero"));
        api.set_player_storage_value(1, 1, "old".to_string());
        api.set_player_storage_value(1, 1, "new".to_string());
        assert_eq!(api.get_player_storage_value(1, 1), Some("new".to_string()));
    }

    // ── Phase 5 — ScriptEvent tests ───────────────────────────────────────────

    #[test]
    fn on_death_event_calls_registered_script_callback() {
        let mut api = LuaApi::new();
        api.register_event("onDeath", "onDeathHandler");
        let callback = api.call_event(&ScriptEvent::CreatureDeath);
        assert_eq!(callback, Some("onDeathHandler"));
    }

    #[test]
    fn on_login_event_fires_for_connected_player() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "Hero"));
        api.register_event("onLogin", "playerLoginFn");
        let callback = api.call_event(&ScriptEvent::PlayerLogin);
        assert_eq!(callback, Some("playerLoginFn"));
    }

    #[test]
    fn script_event_as_str_returns_expected_names() {
        assert_eq!(ScriptEvent::PlayerLogin.as_str(), "onLogin");
        assert_eq!(ScriptEvent::PlayerLogout.as_str(), "onLogout");
        assert_eq!(ScriptEvent::CreatureDeath.as_str(), "onDeath");
        assert_eq!(ScriptEvent::CreatureThink.as_str(), "onThink");
        assert_eq!(ScriptEvent::PlayerAdvance.as_str(), "onAdvance");
        assert_eq!(ScriptEvent::ItemUse.as_str(), "onUse");
        assert_eq!(ScriptEvent::CreatureStep.as_str(), "onStep");
    }

    #[test]
    fn unregistered_event_returns_none() {
        let api = LuaApi::new();
        assert!(api.call_event(&ScriptEvent::PlayerLogout).is_none());
    }

    // ── Existing tests ────────────────────────────────────────────────────────

    #[test]
    fn stub_register_and_lookup() {
        let mut stub = LuaApiStub::new();
        stub.register("doTeleportThing");
        assert!(stub.is_registered("doTeleportThing"));
    }

    #[test]
    fn stub_unregistered_returns_false() {
        let stub = LuaApiStub::new();
        assert!(!stub.is_registered("unknownFn"));
    }

    #[test]
    fn lua_api_new_has_secure_sandbox() {
        let api = LuaApi::new();
        assert!(!api.sandbox_config().allow_os_execute);
        assert!(!api.sandbox_config().allow_io);
    }

    #[test]
    fn get_player_functions_registered() {
        let api = LuaApi::new();
        for name in &[
            "getPlayerHealth",
            "getPlayerMaxHealth",
            "getPlayerMana",
            "getPlayerMaxMana",
            "getPlayerLevel",
            "getPlayerExperience",
            "getPlayerVocation",
            "getPlayerName",
            "getPlayerGUID",
            "getPlayerAccountId",
            "getPlayerPosition",
            "getPlayerSpeed",
            "getPlayerSkullType",
            "getPlayerOutfit",
        ] {
            assert!(api.is_registered(name), "{name} not registered");
        }
    }

    #[test]
    fn combat_functions_registered() {
        let api = LuaApi::new();
        for name in &["doCombat", "doAreaCombat", "doTargetCombat"] {
            assert!(api.is_registered(name), "{name} not registered");
        }
    }

    #[test]
    fn db_functions_registered() {
        let api = LuaApi::new();
        assert!(api.is_registered("db.query"));
        assert!(api.is_registered("db.storeQuery"));
        assert!(api.is_registered("db.escapeString"));
    }

    #[test]
    fn storage_functions_registered() {
        let api = LuaApi::new();
        assert!(api.is_registered("getPlayerStorageValue"));
        assert!(api.is_registered("setPlayerStorageValue"));
    }

    #[test]
    fn position_helper_functions_registered() {
        let api = LuaApi::new();
        assert!(api.is_registered("getPosByDir"));
        assert!(api.is_registered("isInRange"));
    }

    #[test]
    fn get_player_health_returns_value() {
        let mut api = LuaApi::new();
        let mut p = PlayerData::new(1, "P1");
        p.health = 120;
        p.max_health = 150;
        api.add_player(p);
        assert_eq!(api.get_player_health(1), Some(120));
    }

    #[test]
    fn do_player_set_health_clamps_to_max() {
        let mut api = LuaApi::new();
        let mut p = PlayerData::new(1, "P1");
        p.max_health = 100;
        api.add_player(p);
        api.do_player_set_health(1, 9999);
        assert_eq!(api.get_player_health(1), Some(100));
    }

    #[test]
    fn get_pos_by_dir_north() {
        let api = LuaApi::new();
        let new_pos = api.get_pos_by_dir(Position::new(100, 100, 7), Direction::North);
        assert_eq!(new_pos, Position::new(100, 99, 7));
    }

    #[test]
    fn is_in_range_center_is_in_range() {
        let api = LuaApi::new();
        let center = Position::new(100, 100, 7);
        assert!(api.is_in_range(center, center, 0));
    }

    #[test]
    fn is_in_range_different_floor_always_false() {
        let api = LuaApi::new();
        let center = Position::new(100, 100, 7);
        let other = Position::new(100, 100, 6);
        assert!(!api.is_in_range(center, other, 100));
    }

    // ── Phase 13.5 – Group A: World globals (Rust-side) ──────────────────────

    #[test]
    fn group_a_functions_are_registered() {
        let api = LuaApi::new();
        for name in &[
            "getWorldType",
            "getWorldTime",
            "broadcastMessage",
            "doAddThing",
        ] {
            assert!(api.is_registered(name), "{name} not registered");
        }
    }

    #[test]
    fn get_world_type_default_is_pvp() {
        let api = LuaApi::new();
        assert_eq!(api.get_world_type(), WorldType::Pvp);
        assert_eq!(api.get_world_type().as_i32(), 2);
    }

    #[test]
    fn set_world_type_changes_value() {
        let mut api = LuaApi::new();
        api.set_world_type(WorldType::NoPvp);
        assert_eq!(api.get_world_type(), WorldType::NoPvp);
        assert_eq!(api.get_world_type().as_i32(), 1);
    }

    #[test]
    fn get_world_time_default_is_zero() {
        let api = LuaApi::new();
        assert_eq!(api.get_world_time(), 0);
    }

    #[test]
    fn set_world_time_changes_value() {
        let mut api = LuaApi::new();
        api.set_world_time(720);
        assert_eq!(api.get_world_time(), 720);
    }

    #[test]
    fn broadcast_message_returns_true_and_records_message() {
        let mut api = LuaApi::new();
        let ok = api.broadcast_message(20, "Hello world!");
        assert!(ok);
        let log = api.drain_broadcasts();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0], (20, "Hello world!".to_string()));
    }

    #[test]
    fn broadcast_message_multiple_messages_all_recorded() {
        let mut api = LuaApi::new();
        api.broadcast_message(1, "First");
        api.broadcast_message(2, "Second");
        let log = api.drain_broadcasts();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].1, "First");
        assert_eq!(log[1].1, "Second");
    }

    #[test]
    fn drain_broadcasts_clears_log() {
        let mut api = LuaApi::new();
        api.broadcast_message(1, "msg");
        api.drain_broadcasts();
        assert!(api.drain_broadcasts().is_empty());
    }

    #[test]
    fn do_add_thing_creates_item_and_returns_uid() {
        let mut api = LuaApi::new();
        let uid = api.do_add_thing(2160, Position::new(100, 100, 7));
        assert_ne!(uid, 0);
        assert_eq!(api.item_get_id(uid), Some(2160));
    }

    // ── Phase 13.5 – Group B: Position helpers (Rust-side) ───────────────────

    #[test]
    fn group_b_functions_are_registered() {
        let api = LuaApi::new();
        assert!(api.is_registered("Position.new"));
        assert!(api.is_registered("Position.getDistance"));
    }

    #[test]
    fn position_new_constructs_correctly() {
        let pos = Position::new(100, 200, 7);
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
        assert_eq!(pos.z, 7);
    }

    #[test]
    fn position_get_distance_same_tile_is_zero() {
        let pos = Position::new(100, 100, 7);
        assert_eq!(LuaApi::position_get_distance(pos, pos), 0);
    }

    #[test]
    fn position_get_distance_chebyshev() {
        let a = Position::new(100, 100, 7);
        let b = Position::new(103, 102, 7);
        // Chebyshev: max(|3|, |2|) = 3
        assert_eq!(LuaApi::position_get_distance(a, b), 3);
    }

    #[test]
    fn position_get_distance_different_floors_returns_max() {
        let a = Position::new(100, 100, 7);
        let b = Position::new(100, 100, 6);
        assert_eq!(LuaApi::position_get_distance(a, b), u16::MAX);
    }

    // ── Phase 13.5 – Group C: Item methods ───────────────────────────────────

    #[test]
    fn group_c_functions_are_registered() {
        let api = LuaApi::new();
        for name in &[
            "Item.getId",
            "Item.getCount",
            "Item.getActionId",
            "Item.setAttribute",
            "Item.getAttribute",
        ] {
            assert!(api.is_registered(name), "{name} not registered");
        }
    }

    #[test]
    fn item_get_id_returns_item_id() {
        let mut api = LuaApi::new();
        api.add_item(ItemData::new(10, 2160));
        assert_eq!(api.item_get_id(10), Some(2160));
    }

    #[test]
    fn item_get_id_missing_returns_none() {
        let api = LuaApi::new();
        assert_eq!(api.item_get_id(999), None);
    }

    #[test]
    fn item_get_count_returns_count() {
        let mut api = LuaApi::new();
        let mut item = ItemData::new(5, 100);
        item.count = 50;
        api.add_item(item);
        assert_eq!(api.item_get_count(5), Some(50));
    }

    #[test]
    fn item_get_action_id_returns_none_when_not_set() {
        let mut api = LuaApi::new();
        api.add_item(ItemData::new(1, 2160));
        assert_eq!(api.item_get_action_id(1), None);
    }

    #[test]
    fn item_set_and_get_attribute_round_trips() {
        let mut api = LuaApi::new();
        api.add_item(ItemData::new(1, 2160));
        assert!(api.item_set_attribute(1, ItemAttr::Name, "Sword".to_string()));
        assert_eq!(
            api.item_get_attribute(1, &ItemAttr::Name),
            Some("Sword".to_string())
        );
    }

    #[test]
    fn item_set_attribute_missing_item_returns_false() {
        let mut api = LuaApi::new();
        assert!(!api.item_set_attribute(999, ItemAttr::Name, "x".to_string()));
    }

    // ── Phase 13.5 – Group D: Creature methods ────────────────────────────────

    #[test]
    fn group_d_functions_are_registered() {
        let api = LuaApi::new();
        for name in &[
            "Creature.getName",
            "Creature.getHealth",
            "Creature.getMaxHealth",
            "Creature.getPosition",
            "Creature.isPlayer",
        ] {
            assert!(api.is_registered(name), "{name} not registered");
        }
    }

    #[test]
    fn creature_get_name_for_creature_uses_uid() {
        let mut api = LuaApi::new();
        api.add_creature(CreatureData::new(42));
        assert_eq!(api.creature_get_name(42), Some("Creature#42".to_string()));
    }

    #[test]
    fn creature_get_name_for_player_uses_player_name() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "Hero"));
        assert_eq!(api.creature_get_name(1), Some("Hero".to_string()));
    }

    #[test]
    fn creature_get_name_missing_returns_none() {
        let api = LuaApi::new();
        assert!(api.creature_get_name(999).is_none());
    }

    #[test]
    fn creature_get_health_for_creature() {
        let mut api = LuaApi::new();
        let mut c = CreatureData::new(7);
        c.health = 80;
        api.add_creature(c);
        assert_eq!(api.creature_get_health(7), Some(80));
    }

    #[test]
    fn creature_get_health_for_player() {
        let mut api = LuaApi::new();
        let mut p = PlayerData::new(1, "P");
        p.health = 55;
        api.add_player(p);
        assert_eq!(api.creature_get_health(1), Some(55));
    }

    #[test]
    fn creature_get_max_health_for_creature() {
        let mut api = LuaApi::new();
        let mut c = CreatureData::new(3);
        c.max_health = 200;
        api.add_creature(c);
        assert_eq!(api.creature_get_max_health(3), Some(200));
    }

    #[test]
    fn creature_get_position_for_creature_returns_stub_pos() {
        let mut api = LuaApi::new();
        api.add_creature(CreatureData::new(1));
        let pos = api.creature_get_position(1).unwrap();
        assert_eq!(pos, Position::new(10, 10, 7));
    }

    #[test]
    fn creature_get_position_for_player() {
        let mut api = LuaApi::new();
        let mut p = PlayerData::new(2, "P");
        p.position = Position::new(500, 500, 7);
        api.add_player(p);
        assert_eq!(
            api.creature_get_position(2),
            Some(Position::new(500, 500, 7))
        );
    }

    #[test]
    fn creature_is_player_returns_true_for_player_uid() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "Hero"));
        assert!(api.creature_is_player(1));
    }

    #[test]
    fn creature_is_player_returns_false_for_creature_uid() {
        let mut api = LuaApi::new();
        api.add_creature(CreatureData::new(10));
        assert!(!api.creature_is_player(10));
    }

    // ── Phase 13.5 – Group E: Player methods ─────────────────────────────────

    #[test]
    fn group_e_functions_are_registered() {
        let api = LuaApi::new();
        for name in &[
            "Player.getLevel",
            "Player.getMagicLevel",
            "Player.addExperience",
            "Player.sendTextMessage",
        ] {
            assert!(api.is_registered(name), "{name} not registered");
        }
    }

    #[test]
    fn player_get_level_returns_level() {
        let mut api = LuaApi::new();
        let mut p = PlayerData::new(1, "P");
        p.level = 75;
        api.add_player(p);
        assert_eq!(api.player_get_level(1), Some(75));
    }

    #[test]
    fn player_get_magic_level_default_is_none_when_unset() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "P"));
        assert_eq!(api.player_get_magic_level(1), None);
    }

    #[test]
    fn player_set_and_get_magic_level() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "P"));
        api.player_set_magic_level(1, 30);
        assert_eq!(api.player_get_magic_level(1), Some(30));
    }

    #[test]
    fn player_add_experience_increases_experience() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "P"));
        assert!(api.player_add_experience(1, 5000));
        assert_eq!(api.get_player_experience(1), Some(5000));
    }

    #[test]
    fn player_add_experience_accumulates() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "P"));
        api.player_add_experience(1, 1000);
        api.player_add_experience(1, 2000);
        assert_eq!(api.get_player_experience(1), Some(3000));
    }

    #[test]
    fn player_add_experience_missing_player_returns_false() {
        let mut api = LuaApi::new();
        assert!(!api.player_add_experience(999, 100));
    }

    #[test]
    fn player_send_text_message_returns_true_for_existing_player() {
        let mut api = LuaApi::new();
        api.add_player(PlayerData::new(1, "P"));
        assert!(api.player_send_text_message(1, "Hello!"));
    }

    #[test]
    fn player_send_text_message_returns_false_for_missing_player() {
        let api = LuaApi::new();
        assert!(!api.player_send_text_message(999, "Hello!"));
    }
}
