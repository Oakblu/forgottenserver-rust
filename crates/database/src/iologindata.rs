use std::collections::HashMap;

use forgottenserver_entity::player::Player;

use crate::database::Database;

// ── Records ───────────────────────────────────────────────────────────────────

/// Mirrors the `players` table row — all columns that `IOLoginData::loadPlayer`
/// reads or `IOLoginData::savePlayer` writes in the C++ original.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerRecord {
    // ── Identity / account ─────────────────────────────────────────────────
    pub guid: u32,
    pub name: String,
    pub account_id: u32,
    pub group_id: u16,

    // ── Core stats ─────────────────────────────────────────────────────────
    pub sex: u16,
    pub vocation_id: u8,
    pub level: u32,
    pub experience: u64,
    pub health: i32,
    pub health_max: i32,
    pub mana: i32,
    pub mana_max: i32,
    pub mana_spent: u64,
    pub magic_level: u32,
    pub soul: u8,
    pub blessings: u16,
    pub stamina: u16,

    // ── Position ───────────────────────────────────────────────────────────
    pub pos_x: u16,
    pub pos_y: u16,
    pub pos_z: u8,
    pub town_id: u32,
    pub direction: u16,

    // ── Skull ──────────────────────────────────────────────────────────────
    pub skull: u16,
    pub skull_time: i64,

    // ── Capacity (stored as cap*100) ───────────────────────────────────────
    pub capacity: u32,

    // ── Outfit ─────────────────────────────────────────────────────────────
    pub look_type: u16,
    pub look_head: u16,
    pub look_body: u16,
    pub look_legs: u16,
    pub look_feet: u16,
    pub look_addons: u16,
    pub look_mount: u16,
    pub look_mount_head: u16,
    pub look_mount_body: u16,
    pub look_mount_legs: u16,
    pub look_mount_feet: u16,
    pub current_mount: u16,
    pub randomize_mount: bool,

    // ── Skills (level + tries) for all 7 skills ────────────────────────────
    pub skill_fist: u16,
    pub skill_fist_tries: u64,
    pub skill_club: u16,
    pub skill_club_tries: u64,
    pub skill_sword: u16,
    pub skill_sword_tries: u64,
    pub skill_axe: u16,
    pub skill_axe_tries: u64,
    pub skill_dist: u16,
    pub skill_dist_tries: u64,
    pub skill_shielding: u16,
    pub skill_shielding_tries: u64,
    pub skill_fishing: u16,
    pub skill_fishing_tries: u64,

    // ── Conditions blob ─────────────────────────────────────────────────────
    pub conditions: Vec<u8>,

    // ── Offline training ───────────────────────────────────────────────────
    pub offline_training_time: i32,
    pub offline_training_skill: i32,

    // ── Timestamps / balance ───────────────────────────────────────────────
    pub last_login: i64,
    pub last_logout_at: i64,
    pub last_ip: String,
    pub balance: u64,
}

impl Default for PlayerRecord {
    fn default() -> Self {
        PlayerRecord {
            guid: 0,
            name: String::new(),
            account_id: 0,
            group_id: 1,
            sex: 0,
            vocation_id: 0,
            level: 1,
            experience: 0,
            health: 100,
            health_max: 100,
            mana: 0,
            mana_max: 0,
            mana_spent: 0,
            magic_level: 0,
            soul: 100,
            blessings: 0,
            stamina: 2520,
            pos_x: 0,
            pos_y: 0,
            pos_z: 0,
            town_id: 1,
            direction: 0,
            skull: 0,
            skull_time: 0,
            capacity: 400,
            look_type: 136,
            look_head: 0,
            look_body: 0,
            look_legs: 0,
            look_feet: 0,
            look_addons: 0,
            look_mount: 0,
            look_mount_head: 0,
            look_mount_body: 0,
            look_mount_legs: 0,
            look_mount_feet: 0,
            current_mount: 0,
            randomize_mount: false,
            skill_fist: 10,
            skill_fist_tries: 0,
            skill_club: 10,
            skill_club_tries: 0,
            skill_sword: 10,
            skill_sword_tries: 0,
            skill_axe: 10,
            skill_axe_tries: 0,
            skill_dist: 10,
            skill_dist_tries: 0,
            skill_shielding: 10,
            skill_shielding_tries: 0,
            skill_fishing: 10,
            skill_fishing_tries: 0,
            conditions: Vec::new(),
            offline_training_time: 0,
            offline_training_skill: -1,
            last_login: 0,
            last_logout_at: 0,
            last_ip: String::new(),
            balance: 0,
        }
    }
}

/// Mirrors the `accounts` table row.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccountRecord {
    pub id: u32,
    pub name: String,
    pub last_login: i64,
    /// Mirrors `accounts.type` — AccountType_t (0=Normal, 1=Tutor, etc.)
    pub account_type: u16,
    /// Unix timestamp when premium expires; 0 = no premium.
    pub premium_ends_at: i64,
    /// Hashed password — used by `account_login`.
    pub password_hash: String,
}

/// Mirrors a row from `account_viplist` joined with the player name.
#[derive(Debug, Clone, PartialEq)]
pub struct VipEntry {
    pub player_id: u32,
    pub name: String,
    pub description: String,
    pub icon: u32,
    pub notify: bool,
}

// ── PlayerLoginData ───────────────────────────────────────────────────────────

/// Minimal player data needed to construct the enter-world burst.
///
/// Mirrors the columns read by `load_player_for_login`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerLoginData {
    pub name: String,
    pub level: u32,
    pub health: u32,
    pub healthmax: u32,
    pub mana: u32,
    pub manamax: u32,
    pub stamina: u32,
    pub posx: u16,
    pub posy: u16,
    pub posz: u8,
}

/// Load the minimal player data needed for the enter-world burst.
///
/// Issues:
/// ```sql
/// SELECT name, level, health, healthmax, mana, manamax, stamina, posx, posy, posz
/// FROM players
/// WHERE id = {character_id}
/// ```
///
/// Returns `None` if no row is found.  If the returned position is `(0, 0, 0)`
/// the player has no saved position and the default temple coordinates
/// `(100, 100, 7)` are substituted.
pub fn load_player_for_login(db: &dyn Database, character_id: i64) -> Option<PlayerLoginData> {
    let sql = format!(
        "SELECT name, level, health, healthmax, mana, manamax, stamina, posx, posy, posz \
         FROM players \
         WHERE id = {character_id}"
    );

    let rows = db.query(&sql).ok()?;
    let row = rows.into_iter().next()?;

    let mut posx: u16 = row.get("posx").unwrap_or(0);
    let mut posy: u16 = row.get("posy").unwrap_or(0);
    let mut posz: u8 = row.get("posz").unwrap_or(0);

    if posx == 0 && posy == 0 && posz == 0 {
        posx = 100;
        posy = 100;
        posz = 7;
    }

    Some(PlayerLoginData {
        name: row.get("name").unwrap_or_default(),
        level: row.get("level").unwrap_or(1),
        health: row.get("health").unwrap_or(100),
        healthmax: row.get("healthmax").unwrap_or(100),
        mana: row.get("mana").unwrap_or(0),
        manamax: row.get("manamax").unwrap_or(0),
        stamina: row.get("stamina").unwrap_or(2520),
        posx,
        posy,
        posz,
    })
}

// ── Session lookup ────────────────────────────────────────────────────────────

/// Look up the `account_id` and `character_id` for a valid, non-expired session
/// that matches `token_blob` and is linked to a player named `character_name`.
///
/// Mirrors C++ `ProtocolGame::login` (forgottenserver/src/protocolgame.cpp ~line 437):
/// ```sql
/// SELECT a.id AS account_id, p.id AS character_id
/// FROM accounts a
/// JOIN sessions s ON a.id = s.account_id
/// JOIN players p ON a.id = p.account_id
/// WHERE s.token = {escaped_blob}
///   AND s.expired_at IS NULL
///   AND p.name = {escaped_string}
///   AND p.deletion = 0
/// ```
///
/// Returns `Some((account_id, character_id))` on success, `None` if no matching
/// row is found or if either id cannot be parsed from the result.
pub fn lookup_session(
    db: &dyn Database,
    token_blob: &[u8],
    character_name: &str,
) -> Option<(i64, i64)> {
    let escaped_token = db.escape_blob(token_blob);
    let escaped_name = db.escape_string(character_name);

    let sql = format!(
        "SELECT a.id AS account_id, p.id AS character_id \
         FROM accounts a \
         JOIN sessions s ON a.id = s.account_id \
         JOIN players p ON a.id = p.account_id \
         WHERE s.token = {escaped_token} \
           AND s.expired_at IS NULL \
           AND p.name = '{escaped_name}' \
           AND p.deletion = 0"
    );

    let rows = db.query(&sql).ok()?;
    let row = rows.into_iter().next()?;
    let account_id: i64 = row.get("account_id")?;
    let character_id: i64 = row.get("character_id")?;
    Some((account_id, character_id))
}

// ── In-memory store ───────────────────────────────────────────────────────────

/// Thin database façade used by `IoLoginData` tests.
///
/// In production this would be replaced by a real SQL backend.
#[derive(Default)]
pub struct LoginDb {
    players: HashMap<String, PlayerRecord>, // keyed by player name (lower-cased)
    players_by_guid: HashMap<u32, String>,  // guid → lower-cased name
    accounts: HashMap<u32, AccountRecord>,
    vip_list: HashMap<u32, Vec<VipEntry>>, // account_id → entries
    online_players: std::collections::HashSet<u32>, // guids currently online
}

impl LoginDb {
    pub fn new() -> Self {
        Self::default()
    }

    fn player_key(name: &str) -> String {
        name.to_lowercase()
    }

    pub fn put_player(&mut self, record: PlayerRecord) {
        let key = Self::player_key(&record.name);
        self.players_by_guid.insert(record.guid, key.clone());
        self.players.insert(key, record);
    }

    pub fn get_player(&self, name: &str) -> Option<&PlayerRecord> {
        self.players.get(&Self::player_key(name))
    }

    pub fn get_player_by_guid(&self, guid: u32) -> Option<&PlayerRecord> {
        let name = self.players_by_guid.get(&guid)?;
        self.players.get(name)
    }

    pub fn players_by_account(&self, account_id: u32) -> Vec<&PlayerRecord> {
        self.players
            .values()
            .filter(|p| p.account_id == account_id)
            .collect()
    }

    pub fn put_account(&mut self, record: AccountRecord) {
        self.accounts.insert(record.id, record);
    }

    pub fn get_account(&self, id: u32) -> Option<&AccountRecord> {
        self.accounts.get(&id)
    }

    pub fn set_online(&mut self, guid: u32, online: bool) {
        if online {
            self.online_players.insert(guid);
        } else {
            self.online_players.remove(&guid);
        }
    }

    pub fn is_online(&self, guid: u32) -> bool {
        self.online_players.contains(&guid)
    }

    pub fn add_vip_entry(&mut self, account_id: u32, entry: VipEntry) {
        self.vip_list.entry(account_id).or_default().push(entry);
    }

    pub fn get_vip_entries(&self, account_id: u32) -> &[VipEntry] {
        self.vip_list
            .get(&account_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn remove_vip_entry(&mut self, account_id: u32, player_id: u32) {
        if let Some(list) = self.vip_list.get_mut(&account_id) {
            list.retain(|e| e.player_id != player_id);
        }
    }

    pub fn update_vip_entry(
        &mut self,
        account_id: u32,
        player_id: u32,
        description: String,
        icon: u32,
        notify: bool,
    ) {
        if let Some(list) = self.vip_list.get_mut(&account_id) {
            if let Some(e) = list.iter_mut().find(|e| e.player_id == player_id) {
                e.description = description;
                e.icon = icon;
                e.notify = notify;
            }
        }
    }
}

// ── IOLoginData ───────────────────────────────────────────────────────────────

/// Handles player account and character serialization.
///
/// The C++ implementation issues raw SQL via the `Database` singleton.  Here we
/// accept a `LoginDb` reference so tests can run without a real database.
pub struct IoLoginData;

impl IoLoginData {
    pub fn new() -> Self {
        Self
    }

    // ── Look-up by name ───────────────────────────────────────────────────────

    /// Load a player record by name.  Returns `None` if not found.
    /// Mirrors C++ `IOLoginData::loadPlayerByName`.
    pub fn load_player_by_name<'a>(&self, db: &'a LoginDb, name: &str) -> Option<&'a PlayerRecord> {
        db.get_player(name)
    }

    /// Convenience alias kept for backwards compatibility with existing tests.
    pub fn load_player<'a>(&self, db: &'a LoginDb, name: &str) -> Option<&'a PlayerRecord> {
        self.load_player_by_name(db, name)
    }

    // ── Look-up by GUID ───────────────────────────────────────────────────────

    /// Load a player record by GUID.  Returns `None` if not found.
    /// Mirrors C++ `IOLoginData::loadPlayerById`.
    pub fn load_player_by_id<'a>(&self, db: &'a LoginDb, id: u32) -> Option<&'a PlayerRecord> {
        db.get_player_by_guid(id)
    }

    // ── GUID / name mapping helpers ───────────────────────────────────────────

    /// Return the GUID for a given player name, or 0 if not found.
    /// Mirrors C++ `IOLoginData::getGuidByName`.
    pub fn get_guid_by_name(&self, db: &LoginDb, name: &str) -> u32 {
        db.get_player(name).map(|p| p.guid).unwrap_or(0)
    }

    /// Return the player name for a given GUID, or an empty string.
    /// Mirrors C++ `IOLoginData::getNameByGuid`.
    pub fn get_name_by_guid(&self, db: &LoginDb, guid: u32) -> String {
        db.get_player_by_guid(guid)
            .map(|p| p.name.clone())
            .unwrap_or_default()
    }

    /// Return the account_id for a player with the given name, or 0.
    /// Mirrors C++ `IOLoginData::getAccountIdByPlayerName`.
    pub fn get_account_id_by_player_name(&self, db: &LoginDb, name: &str) -> u32 {
        db.get_player(name).map(|p| p.account_id).unwrap_or(0)
    }

    /// Return the account_id for a player with the given GUID, or 0.
    /// Mirrors C++ `IOLoginData::getAccountIdByPlayerId`.
    pub fn get_account_id_by_player_id(&self, db: &LoginDb, id: u32) -> u32 {
        db.get_player_by_guid(id).map(|p| p.account_id).unwrap_or(0)
    }

    /// Normalise the casing of a player name to match the stored value.
    /// Returns `false` if no player with that name exists.
    /// Mirrors C++ `IOLoginData::formatPlayerName`.
    pub fn format_player_name(&self, db: &LoginDb, name: &mut String) -> bool {
        if let Some(record) = db.get_player(name) {
            *name = record.name.clone();
            true
        } else {
            false
        }
    }

    // ── Preload ───────────────────────────────────────────────────────────────

    /// Fetch the minimal set of fields needed before the full load.
    /// Mirrors C++ `IOLoginData::preloadPlayer`.
    /// Returns `Some((name, group_id, account_id, account_type, premium_ends_at))`.
    pub fn preload_player(&self, db: &LoginDb, guid: u32) -> Option<(String, u16, u32, u16, i64)> {
        let player = db.get_player_by_guid(guid)?;
        let account = db.get_account(player.account_id)?;
        Some((
            player.name.clone(),
            player.group_id,
            player.account_id,
            account.account_type,
            account.premium_ends_at,
        ))
    }

    // ── Persist ───────────────────────────────────────────────────────────────

    /// Persist a raw `PlayerRecord`.
    pub fn save_player(&self, db: &mut LoginDb, record: PlayerRecord) {
        db.put_player(record);
    }

    /// Persist a live `Player` entity into the database, capturing all fields
    /// that `IOLoginData::savePlayer` writes in the C++ original.
    ///
    /// This replaces the old thin version which only stored a subset of fields.
    pub fn save_player_entity(
        &self,
        db: &mut LoginDb,
        player: &Player,
        account_id: u32,
        last_login: i64,
    ) {
        use forgottenserver_entity::player::SkillType;

        let record = PlayerRecord {
            guid: player.guid,
            name: player.name.clone(),
            account_id,
            group_id: 1, // default; caller may override after load
            sex: 0,      // default; caller may set
            vocation_id: player.vocation_id as u8,
            level: player.get_level(),
            experience: player.get_experience(),
            health: player.get_health(),
            health_max: player.get_max_health(),
            mana: player.get_mana(),
            mana_max: player.get_max_mana(),
            mana_spent: player.get_mana_spent(),
            magic_level: player.get_magic_level(),
            soul: player.get_soul(),
            blessings: player.get_blessings_byte() as u16,
            stamina: player.get_stamina(),
            pos_x: player.get_login_position().x,
            pos_y: player.get_login_position().y,
            pos_z: player.get_login_position().z,
            town_id: 1,
            direction: 0,
            skull: player.get_skull() as u16,
            skull_time: player.get_skull_ticks() as i64,
            capacity: player.get_capacity(),
            look_type: 0,
            look_head: 0,
            look_body: 0,
            look_legs: 0,
            look_feet: 0,
            look_addons: 0,
            look_mount: 0,
            look_mount_head: 0,
            look_mount_body: 0,
            look_mount_legs: 0,
            look_mount_feet: 0,
            current_mount: 0,
            randomize_mount: false,
            skill_fist: player.get_skill_level(SkillType::Fist),
            skill_fist_tries: player.get_skill_tries(SkillType::Fist),
            skill_club: player.get_skill_level(SkillType::Club),
            skill_club_tries: player.get_skill_tries(SkillType::Club),
            skill_sword: player.get_skill_level(SkillType::Sword),
            skill_sword_tries: player.get_skill_tries(SkillType::Sword),
            skill_axe: player.get_skill_level(SkillType::Axe),
            skill_axe_tries: player.get_skill_tries(SkillType::Axe),
            skill_dist: player.get_skill_level(SkillType::Distance),
            skill_dist_tries: player.get_skill_tries(SkillType::Distance),
            skill_shielding: player.get_skill_level(SkillType::Shield),
            skill_shielding_tries: player.get_skill_tries(SkillType::Shield),
            skill_fishing: player.get_skill_level(SkillType::Fishing),
            skill_fishing_tries: player.get_skill_tries(SkillType::Fishing),
            conditions: Vec::new(),
            offline_training_time: player.get_offline_training_time(),
            offline_training_skill: (player.get_offline_training_skill() as i8) as i32,
            last_login,
            last_logout_at: 0,
            last_ip: String::new(),
            balance: 0,
        };
        db.put_player(record);
    }

    // ── Account helpers ───────────────────────────────────────────────────────

    /// All players that belong to the given account.
    pub fn load_players_by_account<'a>(
        &self,
        db: &'a LoginDb,
        account_id: u32,
    ) -> Vec<&'a PlayerRecord> {
        db.players_by_account(account_id)
    }

    /// Persist an account record.
    pub fn save_account(&self, db: &mut LoginDb, record: AccountRecord) {
        db.put_account(record);
    }

    /// Load an account by id.
    pub fn load_account<'a>(&self, db: &'a LoginDb, id: u32) -> Option<&'a AccountRecord> {
        db.get_account(id)
    }

    /// Verify account credentials.  Returns the account id on success, 0 on failure.
    /// Mirrors C++ `IOLoginData::loginserverAuthentication` / `accountLogin`.
    ///
    /// The in-memory store holds plain hashes; production would use bcrypt/SHA1.
    pub fn account_login(&self, db: &LoginDb, name: &str, password_hash: &str) -> u32 {
        // Find account by name
        let account = db.accounts.values().find(|a| a.name == name);
        match account {
            Some(a) if a.password_hash == password_hash => a.id,
            _ => 0,
        }
    }

    /// Update the online status for the given player GUID.
    /// Mirrors C++ `IOLoginData::updateOnlineStatus`.
    pub fn update_online_status(&self, db: &mut LoginDb, guid: u32, login: bool) {
        db.set_online(guid, login);
    }

    /// Increase the bank balance for the given GUID.
    /// Mirrors C++ `IOLoginData::increaseBankBalance`.
    pub fn increase_bank_balance(&self, db: &mut LoginDb, guid: u32, amount: u64) {
        // Find the player by GUID and update balance.
        if let Some(name) = db.players_by_guid.get(&guid).cloned() {
            if let Some(record) = db.players.get_mut(&name) {
                record.balance = record.balance.saturating_add(amount);
            }
        }
    }

    /// Returns true if the player has bid on any house.
    /// Mirrors C++ `IOLoginData::hasBiddedOnHouse` (no house table in in-mem db).
    pub fn has_bidded_on_house(&self, _db: &LoginDb, _guid: u32) -> bool {
        // The in-memory stub has no houses table; always returns false.
        false
    }

    // ── VIP management ────────────────────────────────────────────────────────

    /// Return VIP entries for the given account.
    /// Mirrors C++ `IOLoginData::getVIPEntries`.
    pub fn get_vip_entries<'a>(&self, db: &'a LoginDb, account_id: u32) -> &'a [VipEntry] {
        db.get_vip_entries(account_id)
    }

    /// Add a VIP entry for the given account.
    /// Mirrors C++ `IOLoginData::addVIPEntry`.
    pub fn add_vip_entry(&self, db: &mut LoginDb, account_id: u32, entry: VipEntry) {
        db.add_vip_entry(account_id, entry);
    }

    /// Edit an existing VIP entry.
    /// Mirrors C++ `IOLoginData::editVIPEntry`.
    pub fn edit_vip_entry(
        &self,
        db: &mut LoginDb,
        account_id: u32,
        player_id: u32,
        description: impl Into<String>,
        icon: u32,
        notify: bool,
    ) {
        db.update_vip_entry(account_id, player_id, description.into(), icon, notify);
    }

    /// Remove a VIP entry.
    /// Mirrors C++ `IOLoginData::removeVIPEntry`.
    pub fn remove_vip_entry(&self, db: &mut LoginDb, account_id: u32, player_id: u32) {
        db.remove_vip_entry(account_id, player_id);
    }

    // ── Premium time ──────────────────────────────────────────────────────────

    /// Update the premium_ends_at timestamp for an account.
    /// Mirrors C++ `IOLoginData::updatePremiumTime`.
    pub fn update_premium_time(&self, db: &mut LoginDb, account_id: u32, end_time: i64) {
        if let Some(account) = db.accounts.get_mut(&account_id) {
            account.premium_ends_at = end_time;
        }
    }
}

impl Default for IoLoginData {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_entity::player::{Player, SkillType, STAMINA_MAX};

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn alice_record() -> PlayerRecord {
        PlayerRecord {
            guid: 1,
            name: "Alice".to_string(),
            account_id: 100,
            group_id: 1,
            sex: 0,
            vocation_id: 1,
            level: 10,
            experience: 5000,
            health: 100,
            health_max: 185,
            mana: 60,
            mana_max: 90,
            mana_spent: 300,
            magic_level: 3,
            soul: 100,
            blessings: 0,
            stamina: STAMINA_MAX,
            pos_x: 100,
            pos_y: 200,
            pos_z: 7,
            town_id: 1,
            direction: 2,
            skull: 0,
            skull_time: 0,
            capacity: 40000,
            look_type: 136,
            look_head: 10,
            look_body: 20,
            look_legs: 30,
            look_feet: 40,
            look_addons: 1,
            look_mount: 0,
            look_mount_head: 0,
            look_mount_body: 0,
            look_mount_legs: 0,
            look_mount_feet: 0,
            current_mount: 0,
            randomize_mount: false,
            skill_fist: 10,
            skill_fist_tries: 0,
            skill_club: 10,
            skill_club_tries: 50,
            skill_sword: 15,
            skill_sword_tries: 2000,
            skill_axe: 10,
            skill_axe_tries: 0,
            skill_dist: 12,
            skill_dist_tries: 800,
            skill_shielding: 20,
            skill_shielding_tries: 5000,
            skill_fishing: 10,
            skill_fishing_tries: 0,
            conditions: vec![0xAB, 0xCD],
            offline_training_time: 43200,
            offline_training_skill: 2,
            last_login: 1_700_000_000,
            last_logout_at: 0,
            last_ip: "127.0.0.1".to_string(),
            balance: 5000,
        }
    }

    fn bob_record() -> PlayerRecord {
        PlayerRecord {
            guid: 2,
            name: "Bob".to_string(),
            account_id: 100,
            level: 5,
            vocation_id: 2,
            experience: 1000,
            health: 80,
            mana: 20,
            stamina: STAMINA_MAX,
            last_login: 1_700_000_100,
            ..Default::default()
        }
    }

    fn make_account(id: u32, name: &str) -> AccountRecord {
        AccountRecord {
            id,
            name: name.to_string(),
            last_login: 0,
            account_type: 0,
            premium_ends_at: 0,
            password_hash: "hash123".to_string(),
        }
    }

    // ── Existing tests preserved ─────────────────────────────────────────────

    #[test]
    fn io_login_data_new_creates_instance() {
        let _io = IoLoginData::new();
    }

    #[test]
    fn load_player_returns_none_for_unknown_name() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(io.load_player(&db, "unknown").is_none());
    }

    #[test]
    fn save_player_then_load_player_returns_record() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        let record = io.load_player(&db, "Alice").expect("player should exist");
        assert_eq!(record.guid, 1);
        assert_eq!(record.name, "Alice");
    }

    #[test]
    fn load_player_is_case_insensitive() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        assert!(io.load_player(&db, "alice").is_some());
        assert!(io.load_player(&db, "ALICE").is_some());
    }

    #[test]
    fn load_players_by_account_returns_matching_players() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        io.save_player(&mut db, bob_record());

        let mut players = io.load_players_by_account(&db, 100);
        players.sort_by_key(|p| p.guid);
        assert_eq!(players.len(), 2);
        assert_eq!(players[0].name, "Alice");
        assert_eq!(players[1].name, "Bob");
    }

    #[test]
    fn load_players_by_account_empty_for_unknown_account() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(io.load_players_by_account(&db, 999).is_empty());
    }

    #[test]
    fn save_account_then_load_account_returns_record() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        let acc = AccountRecord {
            id: 100,
            name: "myaccount".to_string(),
            last_login: 1_700_000_000,
            account_type: 0,
            premium_ends_at: 0,
            password_hash: String::new(),
        };
        io.save_account(&mut db, acc.clone());
        let loaded = io.load_account(&db, 100).expect("account should exist");
        assert_eq!(loaded.id, 100);
        assert_eq!(loaded.name, "myaccount");
    }

    #[test]
    fn load_account_returns_none_for_unknown_id() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(io.load_account(&db, 999).is_none());
    }

    #[test]
    fn save_player_persists_full_state_to_db() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_health(80);
        player.set_mana(40);
        player.drain_stamina(100);

        io.save_player_entity(&mut db, &player, 100, 1_700_000_000);

        let record = io
            .load_player(&db, "Alice")
            .expect("player should be saved");
        assert_eq!(record.guid, 1);
        assert_eq!(record.health, 80);
        assert_eq!(record.mana, 40);
        assert_eq!(record.stamina, STAMINA_MAX - 100);
        assert_eq!(record.account_id, 100);
    }

    #[test]
    fn save_player_entity_updates_level_and_experience() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(5, "Bob", 2);
        player.add_experience(500);

        io.save_player_entity(&mut db, &player, 200, 0);

        let record = io.load_player(&db, "Bob").unwrap();
        assert_eq!(record.experience, 500);
        assert_eq!(record.level, 1); // not enough XP for level 2 with 500
    }

    // ── New tests: loadPlayerByName / loadPlayerById ──────────────────────────

    #[test]
    fn load_player_by_name_returns_correct_record() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        let r = io.load_player_by_name(&db, "Alice").expect("should exist");
        assert_eq!(r.guid, 1);
        assert_eq!(r.account_id, 100);
    }

    #[test]
    fn load_player_by_name_returns_none_for_missing() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(io.load_player_by_name(&db, "NoSuchPlayer").is_none());
    }

    #[test]
    fn load_player_by_id_returns_correct_record() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        let r = io.load_player_by_id(&db, 1).expect("should find by guid=1");
        assert_eq!(r.name, "Alice");
    }

    #[test]
    fn load_player_by_id_returns_none_for_missing_guid() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(io.load_player_by_id(&db, 9999).is_none());
    }

    // ── PlayerRecord covers all loadPlayer columns ────────────────────────────

    #[test]
    fn player_record_stores_health_and_healthmax() {
        let r = alice_record();
        assert_eq!(r.health, 100);
        assert_eq!(r.health_max, 185);
    }

    #[test]
    fn player_record_stores_mana_manamax_manaspent() {
        let r = alice_record();
        assert_eq!(r.mana, 60);
        assert_eq!(r.mana_max, 90);
        assert_eq!(r.mana_spent, 300);
    }

    #[test]
    fn player_record_stores_magic_level() {
        let r = alice_record();
        assert_eq!(r.magic_level, 3);
    }

    #[test]
    fn player_record_stores_blessings() {
        let mut r = alice_record();
        r.blessings = 0b0001_0101; // blessings 0, 2, 4
        assert_eq!(r.blessings, 0b0001_0101);
    }

    #[test]
    fn player_record_stores_soul() {
        let r = alice_record();
        assert_eq!(r.soul, 100);
    }

    #[test]
    fn player_record_stores_capacity() {
        let r = alice_record();
        assert_eq!(r.capacity, 40000); // 400 * 100
    }

    #[test]
    fn player_record_stores_position() {
        let r = alice_record();
        assert_eq!(r.pos_x, 100);
        assert_eq!(r.pos_y, 200);
        assert_eq!(r.pos_z, 7);
    }

    #[test]
    fn player_record_stores_town_id() {
        let r = alice_record();
        assert_eq!(r.town_id, 1);
    }

    #[test]
    fn player_record_stores_direction() {
        let r = alice_record();
        assert_eq!(r.direction, 2);
    }

    #[test]
    fn player_record_stores_skull_and_skull_time() {
        let mut r = alice_record();
        r.skull = 4; // SKULL_RED
        r.skull_time = 1_700_005_000;
        assert_eq!(r.skull, 4);
        assert_eq!(r.skull_time, 1_700_005_000);
    }

    #[test]
    fn player_record_stores_outfit_fields() {
        let r = alice_record();
        assert_eq!(r.look_type, 136);
        assert_eq!(r.look_head, 10);
        assert_eq!(r.look_body, 20);
        assert_eq!(r.look_legs, 30);
        assert_eq!(r.look_feet, 40);
        assert_eq!(r.look_addons, 1);
    }

    #[test]
    fn player_record_stores_mount_fields() {
        let mut r = alice_record();
        r.look_mount = 5;
        r.look_mount_head = 11;
        r.look_mount_body = 22;
        r.look_mount_legs = 33;
        r.look_mount_feet = 44;
        r.current_mount = 5;
        r.randomize_mount = true;
        assert_eq!(r.look_mount, 5);
        assert_eq!(r.look_mount_head, 11);
        assert_eq!(r.look_mount_body, 22);
        assert_eq!(r.look_mount_legs, 33);
        assert_eq!(r.look_mount_feet, 44);
        assert_eq!(r.current_mount, 5);
        assert!(r.randomize_mount);
    }

    #[test]
    fn player_record_stores_all_7_skill_levels_and_tries() {
        let r = alice_record();
        assert_eq!(r.skill_fist, 10);
        assert_eq!(r.skill_fist_tries, 0);
        assert_eq!(r.skill_club, 10);
        assert_eq!(r.skill_club_tries, 50);
        assert_eq!(r.skill_sword, 15);
        assert_eq!(r.skill_sword_tries, 2000);
        assert_eq!(r.skill_axe, 10);
        assert_eq!(r.skill_axe_tries, 0);
        assert_eq!(r.skill_dist, 12);
        assert_eq!(r.skill_dist_tries, 800);
        assert_eq!(r.skill_shielding, 20);
        assert_eq!(r.skill_shielding_tries, 5000);
        assert_eq!(r.skill_fishing, 10);
        assert_eq!(r.skill_fishing_tries, 0);
    }

    #[test]
    fn player_record_stores_conditions_blob() {
        let r = alice_record();
        assert_eq!(r.conditions, vec![0xAB, 0xCD]);
    }

    #[test]
    fn player_record_stores_offline_training_fields() {
        let r = alice_record();
        assert_eq!(r.offline_training_time, 43200);
        assert_eq!(r.offline_training_skill, 2);
    }

    #[test]
    fn player_record_stores_balance() {
        let r = alice_record();
        assert_eq!(r.balance, 5000);
    }

    #[test]
    fn player_record_stores_last_ip() {
        let r = alice_record();
        assert_eq!(r.last_ip, "127.0.0.1");
    }

    // ── save_player_entity captures all key fields ────────────────────────────

    #[test]
    fn save_player_entity_captures_health_max_and_mana_max() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_max_health(200);
        player.set_max_mana(150);

        io.save_player_entity(&mut db, &player, 100, 0);

        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.health_max, 200);
        assert_eq!(r.mana_max, 150);
    }

    #[test]
    fn save_player_entity_captures_magic_level() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_magic_level(5);

        io.save_player_entity(&mut db, &player, 100, 0);

        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.magic_level, 5);
    }

    #[test]
    fn save_player_entity_captures_skull() {
        use forgottenserver_entity::player::Skull;
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_skull(Skull::Red);

        io.save_player_entity(&mut db, &player, 100, 0);

        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.skull, Skull::Red as u16);
    }

    #[test]
    fn save_player_entity_captures_capacity() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_capacity(50000);

        io.save_player_entity(&mut db, &player, 100, 0);

        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.capacity, 50000);
    }

    #[test]
    fn save_player_entity_captures_all_skills() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_skill_level(SkillType::Sword, 25);
        player.add_skill_tries(SkillType::Shield, 100);
        player.set_skill_level(SkillType::Fist, 15);
        player.set_skill_level(SkillType::Club, 12);
        player.set_skill_level(SkillType::Axe, 18);
        player.set_skill_level(SkillType::Distance, 20);
        player.set_skill_level(SkillType::Fishing, 10);

        io.save_player_entity(&mut db, &player, 100, 0);

        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.skill_sword, 25);
        assert_eq!(r.skill_fist, 15);
        assert_eq!(r.skill_club, 12);
        assert_eq!(r.skill_axe, 18);
        assert_eq!(r.skill_dist, 20);
        assert_eq!(r.skill_fishing, 10);
    }

    #[test]
    fn save_player_entity_captures_offline_training() {
        use forgottenserver_entity::player::OfflineTrainingSkill;
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_offline_training_skill(OfflineTrainingSkill::Sword);
        player.set_offline_training_time(7200);

        io.save_player_entity(&mut db, &player, 100, 0);

        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.offline_training_time, 7200);
        assert_eq!(
            r.offline_training_skill,
            (OfflineTrainingSkill::Sword as i8) as i32
        );
    }

    #[test]
    fn save_player_entity_captures_blessings() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();

        let mut player = Player::new(1, "Alice", 1);
        player.add_blessing(0);
        player.add_blessing(2);
        player.add_blessing(4);

        io.save_player_entity(&mut db, &player, 100, 0);

        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.blessings, 0b0001_0101);
    }

    // ── GUID / name mapping ───────────────────────────────────────────────────

    #[test]
    fn get_guid_by_name_returns_correct_guid() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        assert_eq!(io.get_guid_by_name(&db, "Alice"), 1);
    }

    #[test]
    fn get_guid_by_name_returns_zero_for_unknown() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert_eq!(io.get_guid_by_name(&db, "Nobody"), 0);
    }

    #[test]
    fn get_name_by_guid_returns_name() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        assert_eq!(io.get_name_by_guid(&db, 1), "Alice");
    }

    #[test]
    fn get_name_by_guid_returns_empty_for_unknown() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert_eq!(io.get_name_by_guid(&db, 9999), "");
    }

    #[test]
    fn get_account_id_by_player_name_returns_correct_id() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        assert_eq!(io.get_account_id_by_player_name(&db, "Alice"), 100);
    }

    #[test]
    fn get_account_id_by_player_name_returns_zero_for_unknown() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert_eq!(io.get_account_id_by_player_name(&db, "Nobody"), 0);
    }

    #[test]
    fn get_account_id_by_player_id_returns_correct_id() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        assert_eq!(io.get_account_id_by_player_id(&db, 1), 100);
    }

    #[test]
    fn get_account_id_by_player_id_returns_zero_for_unknown_guid() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert_eq!(io.get_account_id_by_player_id(&db, 9999), 0);
    }

    // ── format_player_name ────────────────────────────────────────────────────

    #[test]
    fn format_player_name_normalises_case() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        let mut name = "ALICE".to_string();
        let found = io.format_player_name(&db, &mut name);
        assert!(found);
        assert_eq!(name, "Alice");
    }

    #[test]
    fn format_player_name_returns_false_for_unknown() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        let mut name = "Nobody".to_string();
        assert!(!io.format_player_name(&db, &mut name));
    }

    // ── preload_player ────────────────────────────────────────────────────────

    #[test]
    fn preload_player_returns_basic_fields() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record());
        io.save_account(&mut db, make_account(100, "alice_acc"));

        let result = io.preload_player(&db, 1);
        assert!(result.is_some());
        let (name, group_id, account_id, account_type, premium) = result.unwrap();
        assert_eq!(name, "Alice");
        assert_eq!(group_id, 1);
        assert_eq!(account_id, 100);
        assert_eq!(account_type, 0);
        assert_eq!(premium, 0);
    }

    #[test]
    fn preload_player_returns_none_for_unknown_guid() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(io.preload_player(&db, 9999).is_none());
    }

    /// `preload_player` must short-circuit to `None` when the player record
    /// exists but the referenced account record does not. Exercises the
    /// `db.get_account(player.account_id)?` early-return branch.
    #[test]
    fn preload_player_returns_none_when_account_missing() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        // Save player whose account_id is 100, but DO NOT save an account.
        io.save_player(&mut db, alice_record());
        assert!(io.preload_player(&db, 1).is_none());
    }

    // ── account_login ─────────────────────────────────────────────────────────

    #[test]
    fn account_login_returns_account_id_on_correct_credentials() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_account(
            &mut db,
            AccountRecord {
                id: 42,
                name: "testaccount".to_string(),
                password_hash: "correct_hash".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(io.account_login(&db, "testaccount", "correct_hash"), 42);
    }

    #[test]
    fn account_login_returns_zero_for_wrong_password() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_account(
            &mut db,
            AccountRecord {
                id: 42,
                name: "testaccount".to_string(),
                password_hash: "correct_hash".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(io.account_login(&db, "testaccount", "wrong_hash"), 0);
    }

    #[test]
    fn account_login_returns_zero_for_unknown_account() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert_eq!(io.account_login(&db, "noone", "anything"), 0);
    }

    // ── update_online_status ──────────────────────────────────────────────────

    #[test]
    fn update_online_status_login_marks_player_online() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.update_online_status(&mut db, 1, true);
        assert!(db.is_online(1));
    }

    #[test]
    fn update_online_status_logout_removes_player() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.update_online_status(&mut db, 1, true);
        io.update_online_status(&mut db, 1, false);
        assert!(!db.is_online(1));
    }

    // ── increase_bank_balance ─────────────────────────────────────────────────

    #[test]
    fn increase_bank_balance_adds_to_player_balance() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_player(&mut db, alice_record()); // starts with balance=5000
        io.increase_bank_balance(&mut db, 1, 1000);
        let r = io.load_player(&db, "Alice").unwrap();
        assert_eq!(r.balance, 6000);
    }

    #[test]
    fn increase_bank_balance_no_op_for_unknown_guid() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        // Should not panic
        io.increase_bank_balance(&mut db, 9999, 1000);
    }

    // ── VIP entries ───────────────────────────────────────────────────────────

    fn make_vip(
        player_id: u32,
        name: &str,
        description: &str,
        icon: u32,
        notify: bool,
    ) -> VipEntry {
        VipEntry {
            player_id,
            name: name.to_string(),
            description: description.to_string(),
            icon,
            notify,
        }
    }

    #[test]
    fn add_vip_entry_and_get_vip_entries_round_trip() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.add_vip_entry(&mut db, 100, make_vip(2, "Bob", "Friend", 3, true));
        let entries = io.get_vip_entries(&db, 100);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].player_id, 2);
        assert_eq!(entries[0].name, "Bob");
        assert_eq!(entries[0].description, "Friend");
        assert_eq!(entries[0].icon, 3);
        assert!(entries[0].notify);
    }

    #[test]
    fn get_vip_entries_returns_empty_for_unknown_account() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(io.get_vip_entries(&db, 999).is_empty());
    }

    #[test]
    fn edit_vip_entry_updates_description_icon_notify() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.add_vip_entry(&mut db, 100, make_vip(2, "Bob", "Old", 1, false));
        io.edit_vip_entry(&mut db, 100, 2, "Updated", 7, true);
        let entries = io.get_vip_entries(&db, 100);
        assert_eq!(entries[0].description, "Updated");
        assert_eq!(entries[0].icon, 7);
        assert!(entries[0].notify);
    }

    #[test]
    fn remove_vip_entry_removes_only_target() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.add_vip_entry(&mut db, 100, make_vip(2, "Bob", "", 0, false));
        io.add_vip_entry(&mut db, 100, make_vip(3, "Charlie", "", 0, false));
        io.remove_vip_entry(&mut db, 100, 2);
        let entries = io.get_vip_entries(&db, 100);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].player_id, 3);
    }

    // ── update_premium_time ───────────────────────────────────────────────────

    #[test]
    fn update_premium_time_sets_end_time() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.save_account(&mut db, make_account(100, "alice_acc"));
        io.update_premium_time(&mut db, 100, 9_999_999_999);
        let acc = io.load_account(&db, 100).unwrap();
        assert_eq!(acc.premium_ends_at, 9_999_999_999);
    }

    #[test]
    fn update_premium_time_no_op_for_unknown_account() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        // Should not panic
        io.update_premium_time(&mut db, 9999, 12345);
    }

    // ── has_bidded_on_house ───────────────────────────────────────────────────

    #[test]
    fn has_bidded_on_house_returns_false_in_stub() {
        let db = LoginDb::new();
        let io = IoLoginData::new();
        assert!(!io.has_bidded_on_house(&db, 1));
    }

    // ── Defensive paths for missing VIP list / players ────────────────────────

    /// `remove_vip_entry` must be a no-op (and not panic) when the account
    /// has no VIP list at all. Covers the implicit `else` arm in
    /// `LoginDb::remove_vip_entry`.
    #[test]
    fn remove_vip_entry_no_op_for_account_without_list() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        // No entries added at all for account 999.
        io.remove_vip_entry(&mut db, 999, 1);
        assert!(io.get_vip_entries(&db, 999).is_empty());
    }

    /// `edit_vip_entry` must be a no-op when the account's VIP list does not
    /// exist. Covers the implicit `else` arm in `LoginDb::update_vip_entry`.
    #[test]
    fn edit_vip_entry_no_op_for_account_without_list() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.edit_vip_entry(&mut db, 999, 1, "ignored", 0, false);
        assert!(io.get_vip_entries(&db, 999).is_empty());
    }

    /// `edit_vip_entry` must not modify any entry when the list exists but no
    /// entry matches the supplied `player_id`. Exercises the inner-`None` arm
    /// of `LoginDb::update_vip_entry` (list found, target id missing).
    #[test]
    fn edit_vip_entry_no_op_for_unknown_player_id() {
        let mut db = LoginDb::new();
        let io = IoLoginData::new();
        io.add_vip_entry(&mut db, 100, make_vip(2, "Bob", "Friend", 3, true));
        io.edit_vip_entry(&mut db, 100, 9999, "ShouldNotApply", 99, false);
        let entries = io.get_vip_entries(&db, 100);
        assert_eq!(entries.len(), 1);
        // Unchanged
        assert_eq!(entries[0].description, "Friend");
        assert_eq!(entries[0].icon, 3);
        assert!(entries[0].notify);
    }

    // ── Default impl for IoLoginData ──────────────────────────────────────────

    /// `IoLoginData::default()` must yield a usable instance equivalent to
    /// `IoLoginData::new()` so callers can rely on `Default`-derived containers
    /// or `Default::default()` constructions.
    #[test]
    fn io_login_data_default_is_usable() {
        let io: IoLoginData = Default::default();
        let db = LoginDb::new();
        // A freshly-defaulted instance should behave identically to ::new().
        assert!(io.load_player(&db, "anyone").is_none());
        assert_eq!(io.get_guid_by_name(&db, "anyone"), 0);
    }

    // ── lookup_session tests ──────────────────────────────────────────────────

    use crate::database::{DbError, DbValue, Row};

    /// A test-only `Database` that stores accounts, sessions, and players rows
    /// in-memory and answers the 3-table JOIN query issued by `lookup_session`.
    ///
    /// The `query` method parses the escaped token from `WHERE s.token = X'...'`
    /// and the player name from `AND p.name = '...'`, then performs the join
    /// manually so we never need a real SQL engine.
    struct SessionTestDb {
        /// accounts rows: (id, ...)
        accounts: Vec<(i64,)>,
        /// sessions rows: (token_hex, account_id, expired_at_is_null)
        sessions: Vec<(String, i64, bool)>,
        /// players rows: (id, account_id, name, deletion)
        players: Vec<(i64, i64, String, i64)>,
    }

    impl SessionTestDb {
        fn new() -> Self {
            Self {
                accounts: Vec::new(),
                sessions: Vec::new(),
                players: Vec::new(),
            }
        }

        fn add_account(&mut self, id: i64) {
            self.accounts.push((id,));
        }

        /// Add a session. `expired_at_is_null` = true means the session is still active.
        fn add_session(&mut self, token: &[u8], account_id: i64, expired_at_is_null: bool) {
            // Store the token as uppercase hex without the X'...' wrapper for easy comparison.
            let hex: String = token.iter().map(|b| format!("{b:02X}")).collect();
            self.sessions.push((hex, account_id, expired_at_is_null));
        }

        fn add_player(&mut self, id: i64, account_id: i64, name: &str, deletion: i64) {
            self.players
                .push((id, account_id, name.to_string(), deletion));
        }
    }

    impl crate::database::Database for SessionTestDb {
        fn query(&self, sql: &str) -> Result<Vec<Row>, DbError> {
            // Extract the hex token from: s.token = X'<HEX>'
            let token_hex = {
                let marker = "s.token = X'";
                let start = sql.find(marker).map(|p| p + marker.len());
                let token_hex =
                    start.and_then(|s| sql[s..].find('\'').map(|e| sql[s..s + e].to_uppercase()));
                match token_hex {
                    Some(h) => h,
                    None => return Ok(vec![]),
                }
            };

            // Extract the player name from: p.name = '<NAME>'
            let player_name = {
                let marker = "p.name = '";
                let start = sql.find(marker).map(|p| p + marker.len());
                let name = start.and_then(|s| {
                    sql[s..].find('\'').map(|e| {
                        // Unescape \' → '
                        sql[s..s + e].replace("\\'", "'")
                    })
                });
                match name {
                    Some(n) => n,
                    None => return Ok(vec![]),
                }
            };

            // Perform the join in memory.
            let mut result_rows = Vec::new();

            for (account_id,) in &self.accounts {
                // Find a matching, non-expired session for this account with the right token.
                let session_matches = self.sessions.iter().any(|(tok, acc_id, not_expired)| {
                    tok == &token_hex && acc_id == account_id && *not_expired
                });
                if !session_matches {
                    continue;
                }

                // Find a matching player for this account with the right name and no deletion.
                let player = self.players.iter().find(|(_, acc_id, name, deletion)| {
                    acc_id == account_id && name == &player_name && *deletion == 0
                });

                if let Some((player_id, _, _, _)) = player {
                    let mut cols = std::collections::HashMap::new();
                    cols.insert("account_id".to_string(), DbValue::Integer(*account_id));
                    cols.insert("character_id".to_string(), DbValue::Integer(*player_id));
                    result_rows.push(Row::new(cols));
                }
            }

            Ok(result_rows)
        }

        fn execute(&mut self, _sql: &str) -> Result<u64, DbError> {
            Ok(1)
        }

        fn escape_string(&self, s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            for ch in s.chars() {
                match ch {
                    '\\' => out.push_str("\\\\"),
                    '\'' => out.push_str("\\'"),
                    c => out.push(c),
                }
            }
            out
        }
    }

    #[test]
    fn lookup_session_valid_token_returns_account_and_character() {
        let mut db = SessionTestDb::new();
        db.add_account(1);
        db.add_session(b"valid_token_123456", 1, true);
        db.add_player(1, 1, "Alice", 0);

        let result = super::lookup_session(&db, b"valid_token_123456", "Alice");
        assert_eq!(result, Some((1, 1)));
    }

    #[test]
    fn lookup_session_wrong_character_name_returns_none() {
        let mut db = SessionTestDb::new();
        db.add_account(1);
        db.add_session(b"valid_token_123456", 1, true);
        db.add_player(1, 1, "Alice", 0);

        let result = super::lookup_session(&db, b"valid_token_123456", "Bob");
        assert_eq!(result, None);
    }

    #[test]
    fn lookup_session_expired_session_returns_none() {
        let mut db = SessionTestDb::new();
        db.add_account(1);
        // expired_at_is_null = false means the session is expired (expired_at IS NOT NULL)
        db.add_session(b"valid_token_123456", 1, false);
        db.add_player(1, 1, "Alice", 0);

        let result = super::lookup_session(&db, b"valid_token_123456", "Alice");
        assert_eq!(result, None);
    }

    #[test]
    fn lookup_session_unknown_token_returns_none() {
        let mut db = SessionTestDb::new();
        db.add_account(1);
        db.add_session(b"known_token_000000", 1, true);
        db.add_player(1, 1, "Alice", 0);

        let result = super::lookup_session(&db, b"unknown_token_xxxx", "Alice");
        assert_eq!(result, None);
    }

    // ── load_player_for_login tests ───────────────────────────────────────────

    /// Backing row for `LoginTestDb`.
    struct LoginTestRow {
        id: i64,
        name: String,
        level: i64,
        health: i64,
        healthmax: i64,
        mana: i64,
        manamax: i64,
        stamina: i64,
        posx: i64,
        posy: i64,
        posz: i64,
    }

    /// A test-only `Database` that holds `players` rows in memory and answers
    /// the `SELECT ... FROM players WHERE id = {id}` query issued by
    /// `load_player_for_login`.
    struct LoginTestDb {
        players: Vec<LoginTestRow>,
    }

    impl LoginTestDb {
        fn new() -> Self {
            Self {
                players: Vec::new(),
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn add_player(
            &mut self,
            id: i64,
            name: &str,
            level: i64,
            health: i64,
            healthmax: i64,
            mana: i64,
            manamax: i64,
            stamina: i64,
            posx: i64,
            posy: i64,
            posz: i64,
        ) {
            self.players.push(LoginTestRow {
                id,
                name: name.to_string(),
                level,
                health,
                healthmax,
                mana,
                manamax,
                stamina,
                posx,
                posy,
                posz,
            });
        }
    }

    impl crate::database::Database for LoginTestDb {
        fn query(&self, sql: &str) -> Result<Vec<crate::database::Row>, crate::database::DbError> {
            // Extract the id from: WHERE id = {id}
            let id_value: i64 = {
                let marker = "WHERE id = ";
                match sql.find(marker) {
                    Some(pos) => {
                        let rest = sql[pos + marker.len()..].trim();
                        match rest.parse::<i64>() {
                            Ok(n) => n,
                            Err(_) => return Ok(vec![]),
                        }
                    }
                    None => return Ok(vec![]),
                }
            };

            let mut result_rows = Vec::new();
            for row in &self.players {
                if row.id != id_value {
                    continue;
                }
                let mut cols = std::collections::HashMap::new();
                cols.insert(
                    "name".to_string(),
                    crate::database::DbValue::Text(row.name.clone()),
                );
                cols.insert(
                    "level".to_string(),
                    crate::database::DbValue::Integer(row.level),
                );
                cols.insert(
                    "health".to_string(),
                    crate::database::DbValue::Integer(row.health),
                );
                cols.insert(
                    "healthmax".to_string(),
                    crate::database::DbValue::Integer(row.healthmax),
                );
                cols.insert(
                    "mana".to_string(),
                    crate::database::DbValue::Integer(row.mana),
                );
                cols.insert(
                    "manamax".to_string(),
                    crate::database::DbValue::Integer(row.manamax),
                );
                cols.insert(
                    "stamina".to_string(),
                    crate::database::DbValue::Integer(row.stamina),
                );
                cols.insert(
                    "posx".to_string(),
                    crate::database::DbValue::Integer(row.posx),
                );
                cols.insert(
                    "posy".to_string(),
                    crate::database::DbValue::Integer(row.posy),
                );
                cols.insert(
                    "posz".to_string(),
                    crate::database::DbValue::Integer(row.posz),
                );
                result_rows.push(crate::database::Row::new(cols));
            }

            Ok(result_rows)
        }

        fn execute(&mut self, _sql: &str) -> Result<u64, crate::database::DbError> {
            Ok(1)
        }

        fn escape_string(&self, s: &str) -> String {
            s.to_string()
        }
    }

    #[test]
    fn load_player_valid_id_returns_data() {
        let mut db = LoginTestDb::new();
        db.add_player(1, "Hero", 5, 200, 300, 50, 100, 2000, 150, 200, 7);

        let result = super::load_player_for_login(&db, 1);
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.name, "Hero");
        assert_eq!(data.level, 5);
        assert_eq!(data.health, 200);
        assert_eq!(data.healthmax, 300);
        assert_eq!(data.mana, 50);
        assert_eq!(data.manamax, 100);
        assert_eq!(data.stamina, 2000);
        assert_eq!(data.posx, 150);
        assert_eq!(data.posy, 200);
        assert_eq!(data.posz, 7);
    }

    #[test]
    fn load_player_unknown_id_returns_none() {
        let db = LoginTestDb::new();
        let result = super::load_player_for_login(&db, 999);
        assert_eq!(result, None);
    }

    #[test]
    fn load_player_zero_position_uses_fallback() {
        let mut db = LoginTestDb::new();
        db.add_player(1, "Hero", 5, 200, 300, 50, 100, 2000, 0, 0, 0);

        let result = super::load_player_for_login(&db, 1);
        assert!(result.is_some());
        let data = result.unwrap();
        assert_eq!(data.posx, 100);
        assert_eq!(data.posy, 100);
        assert_eq!(data.posz, 7);
    }
}
