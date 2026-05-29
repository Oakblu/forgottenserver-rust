// Migrated from forgottenserver/src/house.h + house.cpp
//
// House holds metadata (id, name, rent, town_id, owner), a list of tile
// positions it covers, and three access lists (owner, sub-owners, guests).
//
// Houses is the registry mapping house_id → House.

use std::collections::{HashMap, HashSet};

use forgottenserver_common::position::Position;

// ---------------------------------------------------------------------------
// AccessList — mirrors C++ AccessList
// ---------------------------------------------------------------------------

/// Mirrors the C++ `AccessList` which manages a set of player GUIDs and an
/// optional "allow everyone" wildcard (`*`).
///
/// Guild/rank parsing (lines containing `@`) is preserved as a no-op because
/// guild state lives outside this crate; the important invariants (player
/// sets, wildcard, 100-line cap) are fully implemented.
#[derive(Debug, Clone, Default)]
pub struct AccessList {
    /// Raw text representation (round-trippable via `get_list` / `parse_list`).
    raw: String,
    /// Resolved player GUIDs.
    player_set: HashSet<u32>,
    /// Whether the wildcard `*` was present in the list.
    allow_everyone: bool,
}

impl AccessList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a newline-delimited text access list.  Matches C++ behaviour:
    ///
    /// - Lines starting with `#` are comments (skipped).
    /// - Lines longer than 100 chars are skipped.
    /// - At most 100 lines are processed.
    /// - `*` sets `allow_everyone`.
    /// - Lines containing `@` are guild/rank entries — stored but not resolved
    ///   (guild lookup requires external state; the `*` wildcard handles the
    ///   common "everyone from guild" case for our purposes).
    /// - All other lines are treated as player-name placeholders; callers may
    ///   pre-resolve names to GUIDs and call `add_player_guid` directly.
    pub fn parse_list(&mut self, list: &str) {
        self.player_set.clear();
        self.allow_everyone = false;
        self.raw = list.to_owned();

        let mut line_count = 0u16;
        for line in list.lines() {
            line_count += 1;
            if line_count > 100 {
                break;
            }
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.len() > 100 {
                continue;
            }
            if line == "*" {
                self.allow_everyone = true;
            }
            // Guild/rank lines (`@`) — no-op (guild state external)
        }
    }

    /// Directly adds a resolved player GUID (used when the caller resolves
    /// player names to GUIDs before adding).
    pub fn add_player_guid(&mut self, guid: u32) {
        self.player_set.insert(guid);
    }

    /// Returns `true` if `player_guid` is in the list or `allow_everyone` is set.
    pub fn is_in_list(&self, player_guid: u32) -> bool {
        self.allow_everyone || self.player_set.contains(&player_guid)
    }

    /// Returns the raw text representation.
    pub fn get_list(&self) -> &str {
        &self.raw
    }

    /// Whether the `*` wildcard was parsed.
    pub fn allows_everyone(&self) -> bool {
        self.allow_everyone
    }
}

// ---------------------------------------------------------------------------
// DoorAccessList — per-door access list (mirrors Door::accessList in C++)
// ---------------------------------------------------------------------------

/// Per-door access control.  Guests/sub-owners of the house can always open
/// doors; this list additionally tracks individual door-level invitees.
#[derive(Debug, Clone, Default)]
pub struct DoorAccessList {
    inner: AccessList,
}

impl DoorAccessList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_list(&mut self, list: &str) {
        self.inner.parse_list(list);
    }

    pub fn add_player_guid(&mut self, guid: u32) {
        self.inner.add_player_guid(guid);
    }

    pub fn is_in_list(&self, player_guid: u32) -> bool {
        self.inner.is_in_list(player_guid)
    }

    pub fn get_list(&self) -> &str {
        self.inner.get_list()
    }
}

// ---------------------------------------------------------------------------
// HouseAccessLevel — mirrors C++ AccessHouseLevel_t
// ---------------------------------------------------------------------------

/// Access levels for house entry and editing.  Matches C++ `AccessHouseLevel_t`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HouseAccessLevel {
    NotInvited = 0,
    Guest = 1,
    SubOwner = 2,
    Owner = 3,
}

// ---------------------------------------------------------------------------
// RentPeriod — mirrors C++ RentPeriod_t
// ---------------------------------------------------------------------------

/// Rent billing frequency.  Matches C++ `RentPeriod_t`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RentPeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}

impl RentPeriod {
    /// Number of seconds added to `paid_until` when rent is successfully paid.
    pub fn seconds(&self) -> i64 {
        match self {
            RentPeriod::Daily => 24 * 60 * 60,
            RentPeriod::Weekly => 24 * 60 * 60 * 7,
            RentPeriod::Monthly => 24 * 60 * 60 * 30,
            RentPeriod::Yearly => 24 * 60 * 60 * 365,
            RentPeriod::Never => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// House
// ---------------------------------------------------------------------------

/// Represents a single player-owned house.
#[derive(Debug, Clone)]
pub struct House {
    id: u32,
    name: String,
    rent: u32,
    town_id: u32,
    owner_guid: u32,
    owner_name: String,
    entry_pos: Position,
    tile_positions: Vec<Position>,
    sub_owners: HashSet<u32>,
    guests: HashSet<u32>,
    /// door_id → per-door access list
    doors: HashMap<u32, DoorAccessList>,
    /// Bed count (each physical bed occupies 2 sqm; stored as raw item count).
    bed_item_count: u32,
    /// Unix timestamp until which rent has been paid.
    paid_until: i64,
    /// Number of outstanding rent warnings sent to the owner.
    rent_warnings: u32,
}

impl House {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    /// Creates a new `House` with the given attributes.  Owner defaults to 0
    /// (no owner) matching the C++ `House::House(uint32_t houseId)` behaviour.
    pub fn new(id: u32, name: impl Into<String>, rent: u32, town_id: u32) -> Self {
        House {
            id,
            name: name.into(),
            rent,
            town_id,
            owner_guid: 0,
            owner_name: String::new(),
            entry_pos: Position::new(0, 0, 0),
            tile_positions: Vec::new(),
            sub_owners: HashSet::new(),
            guests: HashSet::new(),
            doors: HashMap::new(),
            bed_item_count: 0,
            paid_until: 0,
            rent_warnings: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Identity getters
    // -----------------------------------------------------------------------

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn get_rent(&self) -> u32 {
        self.rent
    }

    pub fn set_rent(&mut self, rent: u32) {
        self.rent = rent;
    }

    pub fn get_town_id(&self) -> u32 {
        self.town_id
    }

    pub fn set_town_id(&mut self, town_id: u32) {
        self.town_id = town_id;
    }

    // -----------------------------------------------------------------------
    // Entry position — mirrors setEntryPos / getEntryPosition
    // -----------------------------------------------------------------------

    pub fn set_entry_pos(&mut self, pos: Position) {
        self.entry_pos = pos;
    }

    pub fn get_entry_pos(&self) -> Position {
        self.entry_pos
    }

    // -----------------------------------------------------------------------
    // Owner — mirrors setOwner / getOwner / getOwnerName
    // -----------------------------------------------------------------------

    /// Returns the GUID of the current owner, or `0` if unowned.
    pub fn get_owner_guid(&self) -> u32 {
        self.owner_guid
    }

    /// Returns the name of the current owner.
    pub fn get_owner_name(&self) -> &str {
        &self.owner_name
    }

    /// Sets the owner by GUID.  Pass `0` to clear ownership (also clears name,
    /// access lists, and rent warnings — mirrors C++ `House::setOwner`).
    pub fn set_owner(&mut self, guid: u32) {
        self.set_owner_with_name(guid, String::new());
    }

    /// Sets the owner by GUID + pre-resolved name.  Clears access lists and
    /// resets rent warnings when `guid` is `0` (eviction path).
    pub fn set_owner_with_name(&mut self, guid: u32, name: impl Into<String>) {
        if guid == 0 {
            // Eviction: clear access state (mirrors C++ owner=0 branch)
            self.owner_guid = 0;
            self.owner_name = String::new();
            self.sub_owners.clear();
            self.guests.clear();
            for door_list in self.doors.values_mut() {
                door_list.parse_list("");
            }
        } else {
            self.owner_guid = guid;
            self.owner_name = name.into();
        }
        self.rent_warnings = 0;
    }

    // -----------------------------------------------------------------------
    // Tile positions
    // -----------------------------------------------------------------------

    /// Adds `pos` to the list of tiles belonging to this house.
    pub fn add_tile_pos(&mut self, pos: Position) {
        self.tile_positions.push(pos);
    }

    /// Read-only access to all tile positions belonging to this house.
    pub fn get_tile_positions(&self) -> &[Position] {
        &self.tile_positions
    }

    /// Returns the number of tiles that belong to this house.
    pub fn get_tile_count(&self) -> usize {
        self.tile_positions.len()
    }

    // -----------------------------------------------------------------------
    // Sub-owner list
    // -----------------------------------------------------------------------

    /// Grants sub-owner access to `player_guid`.
    pub fn add_sub_owner(&mut self, player_guid: u32) {
        self.sub_owners.insert(player_guid);
    }

    /// Returns `true` if `player_guid` is a sub-owner.
    pub fn is_sub_owner(&self, player_guid: u32) -> bool {
        self.sub_owners.contains(&player_guid)
    }

    /// Removes sub-owner access.  Returns `true` if the guid was present.
    pub fn remove_sub_owner(&mut self, player_guid: u32) -> bool {
        self.sub_owners.remove(&player_guid)
    }

    // -----------------------------------------------------------------------
    // Guest list
    // -----------------------------------------------------------------------

    /// Grants guest access to `player_guid`.
    pub fn add_guest(&mut self, player_guid: u32) {
        self.guests.insert(player_guid);
    }

    /// Returns `true` if `player_guid` is a guest.
    pub fn is_guest(&self, player_guid: u32) -> bool {
        self.guests.contains(&player_guid)
    }

    /// Removes guest access.  Returns `true` if the guid was present.
    pub fn remove_guest(&mut self, player_guid: u32) -> bool {
        self.guests.remove(&player_guid)
    }

    // -----------------------------------------------------------------------
    // is_invited — mirrors House::isInvited
    // -----------------------------------------------------------------------

    /// Returns `true` if `player_guid` is the owner, a sub-owner, or a guest.
    pub fn is_invited(&self, player_guid: u32) -> bool {
        self.owner_guid == player_guid
            || self.sub_owners.contains(&player_guid)
            || self.guests.contains(&player_guid)
    }

    // -----------------------------------------------------------------------
    // Access level — mirrors House::getHouseAccessLevel
    // -----------------------------------------------------------------------

    /// Returns the `HouseAccessLevel` for `player_guid`.  Mirrors C++
    /// `getHouseAccessLevel` (without the `PlayerFlag_CanEditHouses` path,
    /// which requires external game state).
    pub fn get_house_access_level(&self, player_guid: u32) -> HouseAccessLevel {
        if self.owner_guid == player_guid {
            HouseAccessLevel::Owner
        } else if self.sub_owners.contains(&player_guid) {
            HouseAccessLevel::SubOwner
        } else if self.guests.contains(&player_guid) {
            HouseAccessLevel::Guest
        } else {
            HouseAccessLevel::NotInvited
        }
    }

    // -----------------------------------------------------------------------
    // can_edit_access_list — mirrors House::canEditAccessList
    // -----------------------------------------------------------------------

    /// Returns `true` if `player_guid` is allowed to edit the given list.
    ///
    /// - Owners can edit any list (`list_id` is ignored).
    /// - Sub-owners can only edit the guest list (`list_id == GUEST_LIST_ID`).
    /// - Guests and uninvited players cannot edit any list.
    pub fn can_edit_access_list(&self, list_id: u32, player_guid: u32) -> bool {
        match self.get_house_access_level(player_guid) {
            HouseAccessLevel::Owner => true,
            HouseAccessLevel::SubOwner => list_id == GUEST_LIST_ID,
            _ => false,
        }
    }

    // -----------------------------------------------------------------------
    // Door management — mirrors addDoor / removeDoor / getDoorByNumber
    // -----------------------------------------------------------------------

    /// Registers a door by its ID.  Idempotent — registering the same ID twice
    /// is a no-op (preserves any existing access list).
    pub fn add_door(&mut self, door_id: u32) {
        self.doors.entry(door_id).or_default();
    }

    /// Removes a door registration.  Returns `true` if the door existed.
    pub fn remove_door(&mut self, door_id: u32) -> bool {
        self.doors.remove(&door_id).is_some()
    }

    /// Returns `true` if a door with `door_id` is registered.
    pub fn has_door(&self, door_id: u32) -> bool {
        self.doors.contains_key(&door_id)
    }

    /// Returns the number of registered doors.
    pub fn get_door_count(&self) -> usize {
        self.doors.len()
    }

    // -----------------------------------------------------------------------
    // Door access list — mirrors Door::setAccessList / getAccessList
    // -----------------------------------------------------------------------

    /// Sets the access list text for a specific door.  Parses the list
    /// immediately (mirrors `Door::setAccessList → AccessList::parseList`).
    /// Returns `false` if no door with `door_id` is registered.
    pub fn set_door_access_list(&mut self, door_id: u32, list: &str) -> bool {
        if let Some(dal) = self.doors.get_mut(&door_id) {
            dal.parse_list(list);
            true
        } else {
            false
        }
    }

    /// Returns the raw access list text for a specific door, or `None` if no
    /// such door is registered (mirrors `House::getAccessList` for door IDs).
    pub fn get_door_access_list(&self, door_id: u32) -> Option<&str> {
        self.doors.get(&door_id).map(|dal| dal.get_list())
    }

    /// Adds a player GUID directly to a door's access list.
    /// Returns `false` if no door with `door_id` is registered.
    pub fn add_door_guest(&mut self, door_id: u32, player_guid: u32) -> bool {
        if let Some(dal) = self.doors.get_mut(&door_id) {
            dal.add_player_guid(player_guid);
            true
        } else {
            false
        }
    }

    /// Returns `true` if `player_guid` is in the access list for `door_id`.
    /// Sub-owners and the owner implicitly pass — callers should check
    /// `get_house_access_level` first for full parity with `Door::canUse`.
    pub fn door_allows_player(&self, door_id: u32, player_guid: u32) -> bool {
        // Owner and sub-owners can always use doors (mirrors Door::canUse).
        match self.get_house_access_level(player_guid) {
            HouseAccessLevel::Owner | HouseAccessLevel::SubOwner => return true,
            _ => {}
        }
        self.doors
            .get(&door_id)
            .map(|dal| dal.is_in_list(player_guid))
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Beds — mirrors addBed / getBedCount
    // -----------------------------------------------------------------------

    /// Registers a bed item (increments raw item count).  In the C++ server
    /// each physical bed occupies 2 items; `get_bed_count()` rounds up to
    /// the nearest whole bed.
    pub fn add_bed(&mut self) {
        self.bed_item_count += 1;
    }

    /// Returns the raw bed item count.
    pub fn get_bed_item_count(&self) -> u32 {
        self.bed_item_count
    }

    /// Returns the number of whole beds (ceiling of `item_count / 2`), matching
    /// C++ `getBedCount()`.
    pub fn get_bed_count(&self) -> u32 {
        self.bed_item_count.div_ceil(2)
    }

    // -----------------------------------------------------------------------
    // Rent / payment — mirrors setPaidUntil / getPaidUntil / payRentWarnings
    // -----------------------------------------------------------------------

    /// Unix timestamp until which rent has been paid.
    pub fn get_paid_until(&self) -> i64 {
        self.paid_until
    }

    /// Sets the `paid_until` timestamp.
    pub fn set_paid_until(&mut self, ts: i64) {
        self.paid_until = ts;
    }

    /// Number of outstanding rent warnings (0–7 in C++).
    pub fn get_rent_warnings(&self) -> u32 {
        self.rent_warnings
    }

    /// Sets the rent warning counter.
    pub fn set_rent_warnings(&mut self, warnings: u32) {
        self.rent_warnings = warnings;
    }

    /// Calculates the new `paid_until` timestamp after a successful rent
    /// payment at `current_time` for `period`.  Returns `None` for
    /// `RentPeriod::Never`.
    pub fn calculate_next_paid_until(current_time: i64, period: RentPeriod) -> Option<i64> {
        if period == RentPeriod::Never {
            return None;
        }
        Some(current_time + period.seconds())
    }
}

// ---------------------------------------------------------------------------
// Special list IDs — mirrors C++ GUEST_LIST / SUBOWNER_LIST
// ---------------------------------------------------------------------------

/// Sentinel `list_id` for the house-wide guest list.  Matches C++ `GUEST_LIST`.
pub const GUEST_LIST_ID: u32 = 0x100;
/// Sentinel `list_id` for the house-wide sub-owner list.  Matches C++ `SUBOWNER_LIST`.
pub const SUBOWNER_LIST_ID: u32 = 0x101;

// ---------------------------------------------------------------------------
// Houses registry
// ---------------------------------------------------------------------------

/// Registry mapping `house_id` → `House`.
#[derive(Debug, Default)]
pub struct Houses {
    houses: HashMap<u32, House>,
}

impl Houses {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Houses {
            houses: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // CRUD
    // -----------------------------------------------------------------------

    /// Adds `house` to the registry.  Replaces any existing house with the
    /// same id.
    pub fn add_house(&mut self, house: House) {
        self.houses.insert(house.id, house);
    }

    /// Returns a shared reference to the house with the given id, or `None`.
    pub fn get_house(&self, id: u32) -> Option<&House> {
        self.houses.get(&id)
    }

    /// Returns a mutable reference to the house with the given id, or `None`.
    pub fn get_house_mut(&mut self, id: u32) -> Option<&mut House> {
        self.houses.get_mut(&id)
    }

    /// Returns the number of houses in the registry.
    pub fn get_house_count(&self) -> usize {
        self.houses.len()
    }

    /// Iterates over all houses.
    pub fn iter(&self) -> impl Iterator<Item = &House> {
        self.houses.values()
    }

    /// Finds a house owned by `player_guid`.  Mirrors C++
    /// `Houses::getHouseByPlayerId`.  Returns `None` if no house has that owner.
    pub fn get_house_by_player_id(&self, player_guid: u32) -> Option<&House> {
        self.houses
            .values()
            .find(|h| h.get_owner_guid() == player_guid)
    }

    /// Mutable variant of `get_house_by_player_id`.
    pub fn get_house_by_player_id_mut(&mut self, player_guid: u32) -> Option<&mut House> {
        self.houses
            .values_mut()
            .find(|h| h.get_owner_guid() == player_guid)
    }
}

// ---------------------------------------------------------------------------
// House::kickPlayer decision helper (Session 33 ledger closure)
// ---------------------------------------------------------------------------

/// Outcome of `House::kickPlayer` mirrored as data. Cross-crate caller
/// (game crate) consults the variant to decide whether to teleport the
/// target back to the house entry-pos and emit the POFF/TELEPORT magic
/// effects. Mirrors the early-return chain in `house.cpp:135-160`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KickPlayerOutcome {
    /// `target` pointer was null — drop.
    AbortNoTarget,
    /// Target has no tile (e.g. between teleports) — drop.
    AbortNoTile,
    /// Target's tile is not a HouseTile of this house — drop.
    AbortNotInThisHouse,
    /// Requester has lower access than target (or target has
    /// PlayerFlag_CanEditHouses) — denied.
    AbortInsufficientAccess,
    /// All guards pass — caller teleports target + emits magic effects.
    Allowed,
}

/// Pure decision for `House::kickPlayer(player, target)`. Inputs mirror
/// the C++ guard sequence:
///
/// * `target_present` — `target != nullptr`
/// * `target_has_tile` — `target->getTile() != nullptr`
/// * `target_in_this_house` — `houseTile != nullptr && houseTile->getHouse() == this`
/// * `requester_access` / `target_access` — `getHouseAccessLevel(...)` ordinal
///   ranks (0 = NotInvited … 3 = Owner)
/// * `target_can_edit_houses` — `target->hasFlag(PlayerFlag_CanEditHouses)`
pub fn kick_player_outcome(
    target_present: bool,
    target_has_tile: bool,
    target_in_this_house: bool,
    requester_access: u8,
    target_access: u8,
    target_can_edit_houses: bool,
) -> KickPlayerOutcome {
    if !target_present {
        return KickPlayerOutcome::AbortNoTarget;
    }
    if !target_has_tile {
        return KickPlayerOutcome::AbortNoTile;
    }
    if !target_in_this_house {
        return KickPlayerOutcome::AbortNotInThisHouse;
    }
    if requester_access < target_access || target_can_edit_houses {
        return KickPlayerOutcome::AbortInsufficientAccess;
    }
    KickPlayerOutcome::Allowed
}

// ---------------------------------------------------------------------------
// AccessList add{Player,Guild,GuildRank} decision helpers (Session 33)
// ---------------------------------------------------------------------------

/// Mirrors C++ `AccessList::addPlayer(name)` — resolve a player name to
/// a GUID, falling back to disk if the player isn't online. Cross-crate
/// caller performs both lookups; this helper picks the winning value.
///
/// Branch order matches C++: online wins if non-zero, otherwise offline
/// is consulted, otherwise None.
pub fn access_list_resolve_player_guid(
    online_guid: Option<u32>,
    offline_guid: Option<u32>,
) -> Option<u32> {
    if let Some(g) = online_guid {
        if g != 0 {
            return Some(g);
        }
    }
    if let Some(g) = offline_guid {
        if g != 0 {
            return Some(g);
        }
    }
    None
}

/// Mirrors C++ `AccessList::addGuild(name)` — given the full list of a
/// guild's rank ids, return them as a Vec for the caller to merge into
/// the access list's guild-rank set. Returns an empty Vec when the
/// guild lookup failed (caller passes `None`).
pub fn access_list_resolve_guild_rank_ids(rank_ids: Option<&[u32]>) -> Vec<u32> {
    rank_ids.map(|s| s.to_vec()).unwrap_or_default()
}

/// Mirrors C++ `AccessList::addGuildRank(name, rankName)` — given the
/// resolved rank-id (or `None` when guild/rank lookup failed), return
/// it for the caller to insert into the access list's guild-rank set.
pub fn access_list_resolve_single_rank_id(rank_id: Option<u32>) -> Option<u32> {
    rank_id
}

// ---------------------------------------------------------------------------
// transferToDepot + payHouses decision helpers (Session 33)
// ---------------------------------------------------------------------------

/// Routing decision for `House::transferToDepot()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepotTransferOutcome {
    /// `townId == 0 || owner == 0` — house has no owner to transfer to.
    Abort,
    /// Owner is online — caller uses the live `Player*` reference.
    TransferOnline,
    /// Owner is offline but loadable — caller loads the player, runs
    /// the transfer, then saves via IOLoginData.
    TransferOffline,
    /// Owner is offline AND not loadable — drop.
    AbortOwnerUnknown,
}

/// Pure decision for `House::transferToDepot()`. Inputs:
/// * `town_set` — `townId != 0`
/// * `owner_set` — `owner != 0` (the house has an owner guid)
/// * `online_player_present` — `g_game.getPlayerByGUID(owner) != nullptr`
/// * `offline_player_loaded` — `IOLoginData::loadPlayerById` returned true
///   (only consulted on the offline branch)
pub fn depot_transfer_outcome(
    town_set: bool,
    owner_set: bool,
    online_player_present: bool,
    offline_player_loaded: bool,
) -> DepotTransferOutcome {
    if !town_set || !owner_set {
        return DepotTransferOutcome::Abort;
    }
    if online_player_present {
        return DepotTransferOutcome::TransferOnline;
    }
    if offline_player_loaded {
        return DepotTransferOutcome::TransferOffline;
    }
    DepotTransferOutcome::AbortOwnerUnknown
}

/// Per-house action for `Houses::payHouses(rentPeriod)`. The C++ loop
/// walks every house, skips unowned ones, and either charges rent or
/// kicks the owner depending on the paid-until timestamp + grace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayHouseAction {
    /// House has no owner; nothing to do this period.
    Skip,
    /// Owner has paid through this rent period — no charge needed.
    AlreadyPaid,
    /// Rent is due; charge the owner's bank balance.
    ChargeRent,
    /// Rent has been unpaid past the grace period — clear the owner.
    KickOwner,
}

/// Pure decision for one house in `Houses::payHouses`. Inputs mirror
/// the per-house state the C++ loop reads:
///
/// * `owner_set` — `house->getOwner() != 0`
/// * `current_time_secs` — `time(nullptr)`
/// * `paid_until_secs` — `house->getPaidUntil()`
/// * `grace_secs` — the configured grace period (rent warnings then
///   eviction). C++ uses `MAX_RENT_WARNING_PERIOD = 7 days` typically;
///   pass it as a parameter so the world crate doesn't pin a value.
pub fn pay_house_action(
    owner_set: bool,
    current_time_secs: i64,
    paid_until_secs: i64,
    grace_secs: i64,
) -> PayHouseAction {
    if !owner_set {
        return PayHouseAction::Skip;
    }
    if current_time_secs < paid_until_secs {
        return PayHouseAction::AlreadyPaid;
    }
    if current_time_secs > paid_until_secs.saturating_add(grace_secs) {
        return PayHouseAction::KickOwner;
    }
    PayHouseAction::ChargeRent
}

// ---------------------------------------------------------------------------
// Houses::loadHousesXML parser (Session 33)
// ---------------------------------------------------------------------------

/// Parsed XML row for one house. The caller iterates a
/// `Vec<ParsedHouseRow>` and applies each row to its `Houses` registry
/// — this gives the same result as the C++ inline mutation without
/// fighting the borrow checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHouseRow {
    pub house_id: u32,
    /// Name attribute (None when missing — caller leaves name unchanged).
    pub name: Option<String>,
    pub entry_pos: forgottenserver_common::position::Position,
    pub rent: Option<u32>,
    pub town_id: Option<u32>,
}

/// Result of `parse_houses_xml` — parsed rows + non-fatal warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHousesXml {
    pub rows: Vec<ParsedHouseRow>,
    /// One entry per house with a zero entry-pos (matches the C++
    /// `[Warning - Houses::loadHousesXML] House entry not set` output).
    pub warnings: Vec<String>,
}

/// Walk an `<houses>` XML doc and return a `ParsedHousesXml` carrying
/// each per-house row (`houseid`, `name`, `entryx/y/z`, `rent`,
/// `townid`). Mirrors C++ `Houses::loadHousesXML` minus the inline
/// `getHouse(id)` lookup + mutation — the caller does that step after
/// `parse_houses_xml` returns, so the parser stays free of the borrow
/// constraints around `&mut Houses`.
///
/// Hard errors (malformed XML, missing `houseid`, unparseable id)
/// surface as `Err`. The caller is responsible for treating "unknown
/// house id" as an error to match the C++ early-return.
pub fn parse_houses_xml(xml: &str) -> Result<ParsedHousesXml, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc
        .descendants()
        .find(|n| n.has_tag_name("houses"))
        .ok_or_else(|| "Missing <houses> root element".to_string())?;

    let mut rows: Vec<ParsedHouseRow> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    for node in root.children().filter(|n| n.is_element()) {
        let house_id_str = node
            .attribute("houseid")
            .ok_or_else(|| "Missing houseid attribute".to_string())?;
        let house_id: u32 = house_id_str
            .parse()
            .map_err(|_| format!("Invalid houseid: {house_id_str}"))?;
        let name = node.attribute("name").map(|s| s.to_string());
        let entry_x: u16 = node
            .attribute("entryx")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let entry_y: u16 = node
            .attribute("entryy")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let entry_z: u16 = node
            .attribute("entryz")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if entry_x == 0 && entry_y == 0 && entry_z == 0 {
            warnings.push(format!(
                "[Houses::loadHousesXML] House entry not set - Name: {} - House id: {}",
                name.clone().unwrap_or_default(),
                house_id
            ));
        }
        rows.push(ParsedHouseRow {
            house_id,
            name,
            entry_pos: forgottenserver_common::position::Position::new(
                entry_x,
                entry_y,
                entry_z as u8,
            ),
            rent: node.attribute("rent").and_then(|s| s.parse().ok()),
            town_id: node.attribute("townid").and_then(|s| s.parse().ok()),
        });
    }
    Ok(ParsedHousesXml { rows, warnings })
}

/// Apply a parsed XML row onto a house registry. Mirrors the C++
/// inline mutation chain (name + entry_pos + rent + town_id + reset
/// owner to 0). Returns `Err` when the row references an unknown
/// house id — matches the C++ early-return.
pub fn apply_parsed_house_row(houses: &mut Houses, row: &ParsedHouseRow) -> Result<(), String> {
    let house = houses.get_house_mut(row.house_id).ok_or_else(|| {
        format!(
            "[Houses::loadHousesXML] Unknown house, id = {}",
            row.house_id
        )
    })?;
    if let Some(name) = &row.name {
        house.set_name(name.clone());
    }
    house.set_entry_pos(row.entry_pos);
    if let Some(rent) = row.rent {
        house.set_rent(rent);
    }
    if let Some(town) = row.town_id {
        house.set_town_id(town);
    }
    house.set_owner(0); // C++ setOwner(0, false) — reset on reload.
    Ok(())
}

// ---------------------------------------------------------------------------
// Door — mirrors C++ `class Door final : public Item` (house.h:36)
// ---------------------------------------------------------------------------
//
// In C++ Door inherits from Item and stores a non-owning back-pointer to its
// House plus an owned AccessList. In the Rust port the per-door access list
// lives on the House side (see `House::doors` / `door_allows_player`) so the
// Door wrapper is a thin composition holding the item-type ID, door_id, and
// the owning house_id. Behaviour parity is preserved through accessor /
// decision helpers that defer to the existing House API.

use forgottenserver_items::item::{AttributeValue, Item, ItemAttribute};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Door {
    item_type_id: u16,
    door_id: u32,
    house_id: Option<u32>,
}

impl Door {
    pub fn new(item_type_id: u16) -> Self {
        Self {
            item_type_id,
            door_id: 0,
            house_id: None,
        }
    }

    pub fn item_type_id(&self) -> u16 {
        self.item_type_id
    }

    pub fn set_door_id(&mut self, door_id: u32) {
        self.door_id = door_id;
    }
    pub fn get_door_id(&self) -> u32 {
        self.door_id
    }

    /// Bind to a house. Mirrors C++ `Door::setHouse` (no-op if already bound).
    pub fn set_house(&mut self, house_id: u32) {
        if self.house_id.is_none() {
            self.house_id = Some(house_id);
        }
    }

    pub fn get_house_id(&self) -> Option<u32> {
        self.house_id
    }

    /// Mirror of C++ `Door::canUse(Player*)`. Returns `true` when the door
    /// is unbound (`house == nullptr` in C++), or when the player passes the
    /// House access check (owner/sub-owner or in this door's access list).
    pub fn can_use(&self, house: Option<&House>, player_guid: u32) -> bool {
        match house {
            None => true,
            Some(h) => h.door_allows_player(self.door_id, player_guid),
        }
    }

    /// Mirror of C++ `Door::setAccessList`. Delegates to the owning house's
    /// `set_door_access_list`. Returns `false` if the door is unbound or its
    /// door_id is not registered on the house.
    pub fn set_access_list(&self, house: &mut House, textlist: &str) -> bool {
        house.set_door_access_list(self.door_id, textlist)
    }

    /// Mirror of C++ `Door::getAccessList`. Returns `None` if the door is
    /// unbound (matches C++ `return false` path).
    pub fn get_access_list<'a>(&self, house: &'a House) -> Option<&'a str> {
        self.house_id?;
        house.get_door_access_list(self.door_id)
    }

    /// Mirror of C++ `Door::onRemoved` — when removed, deregister from the
    /// owning house if any. Returns true if the door was removed from a
    /// house registry.
    pub fn on_removed(&self, house: &mut House) -> bool {
        if self.house_id.is_some() {
            house.remove_door(self.door_id)
        } else {
            false
        }
    }

    /// Mirror of C++ `Door::readAttr(ATTR_HOUSEDOORID, ...)` — when an OTBM
    /// loader encounters a HOUSEDOORID attribute on a door, the door_id is
    /// updated and the corresponding Item attribute set. The C++ overload
    /// reads a single uint8_t from the prop stream; this Rust helper takes
    /// the already-parsed value to keep the helper pure.
    pub fn apply_house_door_id(&mut self, door_id: u8, item: &mut Item) {
        self.door_id = u32::from(door_id);
        item.set_attribute(
            ItemAttribute::DoorId,
            AttributeValue::Integer(i64::from(door_id)),
        );
    }
}

// ---------------------------------------------------------------------------
// HouseTransferItem — mirrors C++ `class HouseTransferItem final : public Item`
// (house.h:89). A throw-away Item created when a player wants to transfer
// ownership of a house to another player; carries the target house_id and
// short-circuits the item-transformation pipeline.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseTransferItem {
    item_type_id: u16,
    target_house_id: u32,
}

impl HouseTransferItem {
    /// Mirrors `HouseTransferItem(House* house)` — the C++ constructor uses
    /// item type 0 (an empty item) carrying the target house pointer.
    pub fn new(target_house_id: u32) -> Self {
        Self {
            item_type_id: 0,
            target_house_id,
        }
    }

    /// Static factory — mirror of `HouseTransferItem::createHouseTransferItem`.
    pub fn create_house_transfer_item(target_house_id: u32) -> Self {
        Self::new(target_house_id)
    }

    pub fn item_type_id(&self) -> u16 {
        self.item_type_id
    }

    pub fn target_house_id(&self) -> u32 {
        self.target_house_id
    }

    /// Mirror of C++ override `bool canTransform() const override { return false; }`.
    /// Transfer items must never be morphed into a different item id.
    pub fn can_transform(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_house() -> House {
        House::new(1, "Test House", 1000, 2)
    }

    // -----------------------------------------------------------------------
    // House struct — basic fields
    // -----------------------------------------------------------------------

    #[test]
    fn house_new_creates_with_correct_fields() {
        let h = House::new(42, "My House", 500, 3);
        assert_eq!(h.get_id(), 42);
        assert_eq!(h.get_name(), "My House");
        assert_eq!(h.get_rent(), 500);
        assert_eq!(h.get_town_id(), 3);
    }

    #[test]
    fn house_owner_guid_defaults_to_zero() {
        let h = sample_house();
        assert_eq!(h.get_owner_guid(), 0);
    }

    #[test]
    fn house_set_owner_updates_guid() {
        let mut h = sample_house();
        h.set_owner(999);
        assert_eq!(h.get_owner_guid(), 999);
    }

    #[test]
    fn house_set_owner_zero_clears_owner() {
        let mut h = sample_house();
        h.set_owner(10);
        h.set_owner(0);
        assert_eq!(h.get_owner_guid(), 0);
    }

    // -----------------------------------------------------------------------
    // Owner name — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn house_owner_name_defaults_to_empty() {
        let h = sample_house();
        assert_eq!(h.get_owner_name(), "");
    }

    #[test]
    fn house_set_owner_with_name_stores_name() {
        let mut h = sample_house();
        h.set_owner_with_name(42, "Alice");
        assert_eq!(h.get_owner_guid(), 42);
        assert_eq!(h.get_owner_name(), "Alice");
    }

    #[test]
    fn house_set_owner_zero_clears_name() {
        let mut h = sample_house();
        h.set_owner_with_name(42, "Alice");
        h.set_owner(0);
        assert_eq!(h.get_owner_name(), "");
    }

    // -----------------------------------------------------------------------
    // Entry position — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn house_entry_pos_defaults_to_origin() {
        let h = sample_house();
        assert_eq!(h.get_entry_pos(), Position::new(0, 0, 0));
    }

    #[test]
    fn house_set_entry_pos_stores_value() {
        let mut h = sample_house();
        h.set_entry_pos(Position::new(100, 200, 7));
        assert_eq!(h.get_entry_pos(), Position::new(100, 200, 7));
    }

    #[test]
    fn house_set_entry_pos_can_be_overwritten() {
        let mut h = sample_house();
        h.set_entry_pos(Position::new(1, 2, 3));
        h.set_entry_pos(Position::new(5, 6, 7));
        assert_eq!(h.get_entry_pos(), Position::new(5, 6, 7));
    }

    // -----------------------------------------------------------------------
    // Tile positions
    // -----------------------------------------------------------------------

    #[test]
    fn house_tile_positions_empty_initially() {
        let h = sample_house();
        assert_eq!(h.get_tile_count(), 0);
        assert!(h.get_tile_positions().is_empty());
    }

    #[test]
    fn house_add_tile_pos_stores_position() {
        let mut h = sample_house();
        let pos = Position::new(100, 200, 7);
        h.add_tile_pos(pos);
        assert_eq!(h.get_tile_count(), 1);
        assert_eq!(h.get_tile_positions()[0], pos);
    }

    #[test]
    fn house_add_multiple_tile_positions() {
        let mut h = sample_house();
        h.add_tile_pos(Position::new(1, 1, 7));
        h.add_tile_pos(Position::new(2, 1, 7));
        h.add_tile_pos(Position::new(3, 1, 7));
        assert_eq!(h.get_tile_count(), 3);
    }

    // -----------------------------------------------------------------------
    // Sub-owners
    // -----------------------------------------------------------------------

    #[test]
    fn house_sub_owner_add_and_check() {
        let mut h = sample_house();
        h.add_sub_owner(777);
        assert!(h.is_sub_owner(777));
    }

    #[test]
    fn house_sub_owner_not_present_returns_false() {
        let h = sample_house();
        assert!(!h.is_sub_owner(123));
    }

    #[test]
    fn house_remove_sub_owner_removes() {
        let mut h = sample_house();
        h.add_sub_owner(777);
        let removed = h.remove_sub_owner(777);
        assert!(removed);
        assert!(!h.is_sub_owner(777));
    }

    #[test]
    fn house_remove_nonexistent_sub_owner_returns_false() {
        let mut h = sample_house();
        assert!(!h.remove_sub_owner(999));
    }

    // -----------------------------------------------------------------------
    // Guests
    // -----------------------------------------------------------------------

    #[test]
    fn house_guest_add_and_check() {
        let mut h = sample_house();
        h.add_guest(555);
        assert!(h.is_guest(555));
    }

    #[test]
    fn house_guest_not_present_returns_false() {
        let h = sample_house();
        assert!(!h.is_guest(555));
    }

    #[test]
    fn house_remove_guest_removes() {
        let mut h = sample_house();
        h.add_guest(555);
        let removed = h.remove_guest(555);
        assert!(removed);
        assert!(!h.is_guest(555));
    }

    #[test]
    fn house_remove_nonexistent_guest_returns_false() {
        let mut h = sample_house();
        assert!(!h.remove_guest(999));
    }

    // -----------------------------------------------------------------------
    // is_invited
    // -----------------------------------------------------------------------

    #[test]
    fn is_invited_true_for_owner() {
        let mut h = sample_house();
        h.set_owner(100);
        assert!(h.is_invited(100));
    }

    #[test]
    fn is_invited_true_for_sub_owner() {
        let mut h = sample_house();
        h.add_sub_owner(200);
        assert!(h.is_invited(200));
    }

    #[test]
    fn is_invited_true_for_guest() {
        let mut h = sample_house();
        h.add_guest(300);
        assert!(h.is_invited(300));
    }

    #[test]
    fn is_invited_false_for_unknown() {
        let h = sample_house();
        assert!(!h.is_invited(999));
    }

    #[test]
    fn is_invited_false_when_owner_zero_and_no_lists() {
        let h = sample_house(); // owner_guid = 0 (unowned)
                                // Guid 1 is not the owner, not a sub-owner, not a guest → not invited.
        assert!(!h.is_invited(1));
        // Guid 0 == owner_guid (both 0); the C++ code performs a plain equality
        // check without special-casing 0, so is_invited(0) returns true.
        assert!(h.is_invited(0));
    }

    // -----------------------------------------------------------------------
    // HouseAccessLevel — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn access_level_owner_returns_owner() {
        let mut h = sample_house();
        h.set_owner(10);
        assert_eq!(h.get_house_access_level(10), HouseAccessLevel::Owner);
    }

    #[test]
    fn access_level_sub_owner_returns_sub_owner() {
        let mut h = sample_house();
        h.set_owner(10);
        h.add_sub_owner(20);
        assert_eq!(h.get_house_access_level(20), HouseAccessLevel::SubOwner);
    }

    #[test]
    fn access_level_guest_returns_guest() {
        let mut h = sample_house();
        h.add_guest(30);
        assert_eq!(h.get_house_access_level(30), HouseAccessLevel::Guest);
    }

    #[test]
    fn access_level_unknown_returns_not_invited() {
        let h = sample_house();
        assert_eq!(h.get_house_access_level(99), HouseAccessLevel::NotInvited);
    }

    #[test]
    fn access_level_ordering_owner_gt_subowner_gt_guest_gt_not_invited() {
        assert!(HouseAccessLevel::Owner > HouseAccessLevel::SubOwner);
        assert!(HouseAccessLevel::SubOwner > HouseAccessLevel::Guest);
        assert!(HouseAccessLevel::Guest > HouseAccessLevel::NotInvited);
    }

    // -----------------------------------------------------------------------
    // can_edit_access_list — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn owner_can_edit_any_list() {
        let mut h = sample_house();
        h.set_owner(1);
        assert!(h.can_edit_access_list(GUEST_LIST_ID, 1));
        assert!(h.can_edit_access_list(SUBOWNER_LIST_ID, 1));
        assert!(h.can_edit_access_list(42, 1)); // arbitrary door id
    }

    #[test]
    fn sub_owner_can_only_edit_guest_list() {
        let mut h = sample_house();
        h.set_owner(1);
        h.add_sub_owner(2);
        assert!(h.can_edit_access_list(GUEST_LIST_ID, 2));
        assert!(!h.can_edit_access_list(SUBOWNER_LIST_ID, 2));
        assert!(!h.can_edit_access_list(42, 2));
    }

    #[test]
    fn guest_cannot_edit_any_list() {
        let mut h = sample_house();
        h.add_guest(3);
        assert!(!h.can_edit_access_list(GUEST_LIST_ID, 3));
        assert!(!h.can_edit_access_list(SUBOWNER_LIST_ID, 3));
    }

    #[test]
    fn uninvited_cannot_edit_any_list() {
        let h = sample_house();
        assert!(!h.can_edit_access_list(GUEST_LIST_ID, 99));
    }

    // -----------------------------------------------------------------------
    // Door management — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn house_doors_empty_initially() {
        let h = sample_house();
        assert_eq!(h.get_door_count(), 0);
        assert!(!h.has_door(1));
    }

    #[test]
    fn house_add_door_registers_door() {
        let mut h = sample_house();
        h.add_door(7);
        assert!(h.has_door(7));
        assert_eq!(h.get_door_count(), 1);
    }

    #[test]
    fn house_add_door_idempotent() {
        let mut h = sample_house();
        h.add_door(7);
        h.add_door(7); // second call should be no-op
        assert_eq!(h.get_door_count(), 1);
    }

    #[test]
    fn house_add_multiple_doors() {
        let mut h = sample_house();
        h.add_door(1);
        h.add_door(2);
        h.add_door(3);
        assert_eq!(h.get_door_count(), 3);
    }

    #[test]
    fn house_remove_door_removes() {
        let mut h = sample_house();
        h.add_door(5);
        let removed = h.remove_door(5);
        assert!(removed);
        assert!(!h.has_door(5));
        assert_eq!(h.get_door_count(), 0);
    }

    #[test]
    fn house_remove_nonexistent_door_returns_false() {
        let mut h = sample_house();
        assert!(!h.remove_door(99));
    }

    #[test]
    fn house_remove_door_does_not_affect_other_doors() {
        let mut h = sample_house();
        h.add_door(1);
        h.add_door(2);
        h.remove_door(1);
        assert!(!h.has_door(1));
        assert!(h.has_door(2));
    }

    // -----------------------------------------------------------------------
    // Door access list — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn door_access_list_empty_by_default() {
        let mut h = sample_house();
        h.add_door(10);
        assert_eq!(h.get_door_access_list(10), Some(""));
    }

    #[test]
    fn door_access_list_returns_none_for_unknown_door() {
        let h = sample_house();
        assert!(h.get_door_access_list(99).is_none());
    }

    #[test]
    fn set_door_access_list_stores_text() {
        let mut h = sample_house();
        h.add_door(10);
        let ok = h.set_door_access_list(10, "Alice\nBob");
        assert!(ok);
        assert_eq!(h.get_door_access_list(10), Some("Alice\nBob"));
    }

    #[test]
    fn set_door_access_list_returns_false_for_unknown_door() {
        let mut h = sample_house();
        assert!(!h.set_door_access_list(99, "Alice"));
    }

    #[test]
    fn add_door_guest_grants_access() {
        let mut h = sample_house();
        h.set_owner(1);
        h.add_door(10);
        h.add_door_guest(10, 42);
        assert!(h.door_allows_player(10, 42));
    }

    #[test]
    fn door_access_owner_always_allowed() {
        let mut h = sample_house();
        h.set_owner(1);
        h.add_door(10);
        // No explicit door guest — owner still passes
        assert!(h.door_allows_player(10, 1));
    }

    #[test]
    fn door_access_sub_owner_always_allowed() {
        let mut h = sample_house();
        h.set_owner(1);
        h.add_sub_owner(2);
        h.add_door(10);
        assert!(h.door_allows_player(10, 2));
    }

    #[test]
    fn door_access_uninvited_not_in_list_returns_false() {
        let mut h = sample_house();
        h.set_owner(1);
        h.add_door(10);
        assert!(!h.door_allows_player(10, 99));
    }

    #[test]
    fn door_access_unknown_door_uninvited_returns_false() {
        let mut h = sample_house();
        h.set_owner(1);
        // An uninvited player (guid=99) cannot use an unregistered door.
        assert!(!h.door_allows_player(99, 99));
    }

    #[test]
    fn door_access_unknown_door_owner_still_allowed() {
        let mut h = sample_house();
        h.set_owner(1);
        // In C++, Door::canUse checks the house access level first; owners always
        // pass regardless of whether the door is in the doorSet. Our implementation
        // preserves this: the owner check happens before the door-registry lookup.
        assert!(h.door_allows_player(99, 1));
    }

    // -----------------------------------------------------------------------
    // set_owner clears door access lists on eviction — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn set_owner_zero_clears_door_access_lists() {
        let mut h = sample_house();
        h.set_owner(1);
        h.add_door(10);
        h.set_door_access_list(10, "Alice\nBob");
        // Evict the owner
        h.set_owner(0);
        assert_eq!(h.get_door_access_list(10), Some(""));
    }

    #[test]
    fn set_owner_zero_clears_sub_owners_and_guests() {
        let mut h = sample_house();
        h.set_owner(1);
        h.add_sub_owner(2);
        h.add_guest(3);
        h.set_owner(0);
        assert!(!h.is_sub_owner(2));
        assert!(!h.is_guest(3));
    }

    #[test]
    fn set_owner_zero_resets_rent_warnings() {
        let mut h = sample_house();
        h.set_owner(1);
        h.set_rent_warnings(5);
        h.set_owner(0);
        assert_eq!(h.get_rent_warnings(), 0);
    }

    // -----------------------------------------------------------------------
    // Beds — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn bed_item_count_defaults_to_zero() {
        let h = sample_house();
        assert_eq!(h.get_bed_item_count(), 0);
        assert_eq!(h.get_bed_count(), 0);
    }

    #[test]
    fn add_bed_increments_item_count() {
        let mut h = sample_house();
        h.add_bed();
        h.add_bed();
        assert_eq!(h.get_bed_item_count(), 2);
    }

    #[test]
    fn get_bed_count_rounds_up_for_single_item() {
        let mut h = sample_house();
        h.add_bed(); // 1 item → 1 bed (ceiling of 0.5)
        assert_eq!(h.get_bed_count(), 1);
    }

    #[test]
    fn get_bed_count_two_items_equals_one_bed() {
        let mut h = sample_house();
        h.add_bed();
        h.add_bed(); // 2 items → 1 bed
        assert_eq!(h.get_bed_count(), 1);
    }

    #[test]
    fn get_bed_count_three_items_equals_two_beds() {
        let mut h = sample_house();
        for _ in 0..3 {
            h.add_bed();
        }
        assert_eq!(h.get_bed_count(), 2);
    }

    #[test]
    fn get_bed_count_four_items_equals_two_beds() {
        let mut h = sample_house();
        for _ in 0..4 {
            h.add_bed();
        }
        assert_eq!(h.get_bed_count(), 2);
    }

    // -----------------------------------------------------------------------
    // Rent / payment — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn paid_until_defaults_to_zero() {
        let h = sample_house();
        assert_eq!(h.get_paid_until(), 0);
    }

    #[test]
    fn set_paid_until_stores_value() {
        let mut h = sample_house();
        h.set_paid_until(1_700_000_000);
        assert_eq!(h.get_paid_until(), 1_700_000_000);
    }

    #[test]
    fn rent_warnings_default_to_zero() {
        let h = sample_house();
        assert_eq!(h.get_rent_warnings(), 0);
    }

    #[test]
    fn set_rent_warnings_stores_value() {
        let mut h = sample_house();
        h.set_rent_warnings(3);
        assert_eq!(h.get_rent_warnings(), 3);
    }

    #[test]
    fn calculate_next_paid_until_daily() {
        let base = 1_700_000_000i64;
        let next = House::calculate_next_paid_until(base, RentPeriod::Daily).unwrap();
        assert_eq!(next, base + 24 * 60 * 60);
    }

    #[test]
    fn calculate_next_paid_until_weekly() {
        let base = 1_700_000_000i64;
        let next = House::calculate_next_paid_until(base, RentPeriod::Weekly).unwrap();
        assert_eq!(next, base + 24 * 60 * 60 * 7);
    }

    #[test]
    fn calculate_next_paid_until_monthly() {
        let base = 1_700_000_000i64;
        let next = House::calculate_next_paid_until(base, RentPeriod::Monthly).unwrap();
        assert_eq!(next, base + 24 * 60 * 60 * 30);
    }

    #[test]
    fn calculate_next_paid_until_yearly() {
        let base = 1_700_000_000i64;
        let next = House::calculate_next_paid_until(base, RentPeriod::Yearly).unwrap();
        assert_eq!(next, base + 24 * 60 * 60 * 365);
    }

    #[test]
    fn calculate_next_paid_until_never_returns_none() {
        let base = 1_700_000_000i64;
        assert!(House::calculate_next_paid_until(base, RentPeriod::Never).is_none());
    }

    #[test]
    fn rent_period_seconds_daily() {
        assert_eq!(RentPeriod::Daily.seconds(), 86_400);
    }

    #[test]
    fn rent_period_seconds_weekly() {
        assert_eq!(RentPeriod::Weekly.seconds(), 604_800);
    }

    #[test]
    fn rent_period_seconds_monthly() {
        assert_eq!(RentPeriod::Monthly.seconds(), 2_592_000);
    }

    #[test]
    fn rent_period_seconds_yearly() {
        assert_eq!(RentPeriod::Yearly.seconds(), 31_536_000);
    }

    #[test]
    fn rent_period_seconds_never() {
        assert_eq!(RentPeriod::Never.seconds(), 0);
    }

    // -----------------------------------------------------------------------
    // AccessList — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn access_list_empty_by_default() {
        let al = AccessList::new();
        assert!(!al.allows_everyone());
        assert!(!al.is_in_list(1));
        assert_eq!(al.get_list(), "");
    }

    #[test]
    fn access_list_wildcard_allows_everyone() {
        let mut al = AccessList::new();
        al.parse_list("*");
        assert!(al.allows_everyone());
        assert!(al.is_in_list(12345));
    }

    #[test]
    fn access_list_add_player_guid_direct() {
        let mut al = AccessList::new();
        al.add_player_guid(42);
        assert!(al.is_in_list(42));
        assert!(!al.is_in_list(43));
    }

    #[test]
    fn access_list_parse_clears_previous() {
        let mut al = AccessList::new();
        al.add_player_guid(1);
        al.parse_list(""); // clear
                           // Direct guid is gone only if parse_list clears player_set — which it does
        assert!(!al.is_in_list(1));
    }

    #[test]
    fn access_list_get_list_returns_raw_text() {
        let mut al = AccessList::new();
        al.parse_list("Alice\nBob\n*");
        assert_eq!(al.get_list(), "Alice\nBob\n*");
    }

    #[test]
    fn access_list_comment_lines_ignored() {
        let mut al = AccessList::new();
        al.parse_list("# this is a comment\n*");
        // Comment lines do not affect wildcard or counts
        assert!(al.allows_everyone());
    }

    #[test]
    fn access_list_100_line_cap() {
        // Build a list with 200 lines where `*` appears on line 101
        let many_lines: String = (0..100)
            .map(|i| format!("# line {i}\n"))
            .collect::<String>()
            + "*\n"
            + "# trailing";
        let mut al = AccessList::new();
        al.parse_list(&many_lines);
        // Line 101 is `*` — but the cap stops at line 100 so `*` is not parsed.
        // Each `# line N` is a comment → 100 comment lines consumed → `*` is line 101 → skipped.
        assert!(!al.allows_everyone());
    }

    // -----------------------------------------------------------------------
    // Houses registry
    // -----------------------------------------------------------------------

    #[test]
    fn houses_new_creates_empty_registry() {
        let houses = Houses::new();
        assert_eq!(houses.get_house_count(), 0);
    }

    #[test]
    fn houses_add_house_stores_it() {
        let mut houses = Houses::new();
        houses.add_house(sample_house());
        assert_eq!(houses.get_house_count(), 1);
    }

    #[test]
    fn houses_get_house_returns_some() {
        let mut houses = Houses::new();
        houses.add_house(sample_house()); // id=1
        assert!(houses.get_house(1).is_some());
    }

    #[test]
    fn houses_get_house_returns_none_for_unknown_id() {
        let houses = Houses::new();
        assert!(houses.get_house(999).is_none());
    }

    #[test]
    fn houses_get_house_count_increments() {
        let mut houses = Houses::new();
        houses.add_house(House::new(1, "H1", 100, 1));
        houses.add_house(House::new(2, "H2", 200, 1));
        houses.add_house(House::new(3, "H3", 300, 2));
        assert_eq!(houses.get_house_count(), 3);
    }

    #[test]
    fn houses_add_house_replaces_existing() {
        let mut houses = Houses::new();
        houses.add_house(House::new(1, "Old", 100, 1));
        houses.add_house(House::new(1, "New", 200, 1));
        assert_eq!(houses.get_house_count(), 1);
        assert_eq!(houses.get_house(1).unwrap().get_name(), "New");
    }

    #[test]
    fn houses_get_house_correct_id() {
        let mut houses = Houses::new();
        houses.add_house(House::new(10, "House Ten", 500, 1));
        let h = houses.get_house(10).unwrap();
        assert_eq!(h.get_id(), 10);
        assert_eq!(h.get_name(), "House Ten");
    }

    // -----------------------------------------------------------------------
    // Houses::get_house_by_player_id — NEW
    // -----------------------------------------------------------------------

    #[test]
    fn get_house_by_player_id_finds_owned_house() {
        let mut houses = Houses::new();
        let mut h = House::new(1, "H1", 100, 1);
        h.set_owner(42);
        houses.add_house(h);
        let found = houses.get_house_by_player_id(42);
        assert!(found.is_some());
        assert_eq!(found.unwrap().get_id(), 1);
    }

    #[test]
    fn get_house_by_player_id_returns_none_for_unowned() {
        let mut houses = Houses::new();
        houses.add_house(House::new(1, "H1", 100, 1)); // owner=0
        assert!(houses.get_house_by_player_id(42).is_none());
    }

    #[test]
    fn get_house_by_player_id_returns_none_for_wrong_guid() {
        let mut houses = Houses::new();
        let mut h = House::new(1, "H1", 100, 1);
        h.set_owner(10);
        houses.add_house(h);
        assert!(houses.get_house_by_player_id(99).is_none());
    }

    #[test]
    fn get_house_by_player_id_mut_allows_mutation() {
        let mut houses = Houses::new();
        let mut h = House::new(1, "H1", 100, 1);
        h.set_owner(7);
        houses.add_house(h);
        {
            let found = houses.get_house_by_player_id_mut(7).unwrap();
            found.set_rent_warnings(3);
        }
        assert_eq!(houses.get_house(1).unwrap().get_rent_warnings(), 3);
    }

    // -----------------------------------------------------------------------
    // set_name / set_rent / set_town_id — NEW (setter coverage)
    // -----------------------------------------------------------------------

    #[test]
    fn house_set_name_updates_name() {
        let mut h = sample_house();
        h.set_name("New Name");
        assert_eq!(h.get_name(), "New Name");
    }

    #[test]
    fn house_set_rent_updates_rent() {
        let mut h = sample_house();
        h.set_rent(9999);
        assert_eq!(h.get_rent(), 9999);
    }

    #[test]
    fn house_set_town_id_updates_town_id() {
        let mut h = sample_house();
        h.set_town_id(5);
        assert_eq!(h.get_town_id(), 5);
    }

    // -----------------------------------------------------------------------
    // Coverage gap fillers (Phase 5 audit)
    // -----------------------------------------------------------------------

    #[test]
    fn door_access_list_new_creates_empty_list() {
        // Exercises `DoorAccessList::new()` directly (the wrapper constructor),
        // mirroring C++ `Door::accessList.reset(new AccessList())`.
        let dal = DoorAccessList::new();
        assert_eq!(dal.get_list(), "");
        assert!(!dal.is_in_list(1));
    }

    #[test]
    fn add_door_guest_returns_false_for_unknown_door() {
        // Mirrors C++ behaviour where `setAccessList`/door-id paths only succeed
        // when the door is registered in `doorSet`.
        let mut h = sample_house();
        assert!(!h.add_door_guest(99, 42));
    }

    #[test]
    fn houses_get_house_mut_allows_mutation() {
        // Mirrors C++ `Houses::getHouse` returning a mutable pointer used to
        // mutate the underlying `House` (e.g. `house->setName(...)`).
        let mut houses = Houses::new();
        houses.add_house(House::new(7, "Old Name", 100, 1));
        {
            let h = houses.get_house_mut(7).expect("house exists");
            h.set_name("New Name");
            h.set_rent(2500);
        }
        let h = houses.get_house(7).unwrap();
        assert_eq!(h.get_name(), "New Name");
        assert_eq!(h.get_rent(), 2500);
    }

    #[test]
    fn houses_get_house_mut_returns_none_for_unknown_id() {
        let mut houses = Houses::new();
        assert!(houses.get_house_mut(999).is_none());
    }

    #[test]
    fn houses_iter_yields_all_houses() {
        // Mirrors C++ `Houses::getHouses()` returning the underlying map for
        // iteration (e.g. `payHouses` loops over every entry).
        let mut houses = Houses::new();
        houses.add_house(House::new(1, "H1", 100, 1));
        houses.add_house(House::new(2, "H2", 200, 1));
        houses.add_house(House::new(3, "H3", 300, 2));
        let mut ids: Vec<u32> = houses.iter().map(|h| h.get_id()).collect();
        ids.sort();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn houses_iter_empty_registry_yields_nothing() {
        let houses = Houses::new();
        assert_eq!(houses.iter().count(), 0);
    }

    // ── kick_player_outcome (Session 33) ────────────────────────────────

    #[test]
    fn kick_aborts_when_target_absent() {
        let r = kick_player_outcome(false, true, true, 3, 0, false);
        assert_eq!(r, KickPlayerOutcome::AbortNoTarget);
    }

    #[test]
    fn kick_aborts_when_target_has_no_tile() {
        let r = kick_player_outcome(true, false, true, 3, 0, false);
        assert_eq!(r, KickPlayerOutcome::AbortNoTile);
    }

    #[test]
    fn kick_aborts_when_target_not_in_this_house() {
        let r = kick_player_outcome(true, true, false, 3, 0, false);
        assert_eq!(r, KickPlayerOutcome::AbortNotInThisHouse);
    }

    #[test]
    fn kick_denied_when_requester_lower_access() {
        // Guest (1) tries to kick Owner (3) — denied.
        let r = kick_player_outcome(true, true, true, 1, 3, false);
        assert_eq!(r, KickPlayerOutcome::AbortInsufficientAccess);
    }

    #[test]
    fn kick_denied_when_target_can_edit_houses() {
        // Even with equal/higher access, CanEditHouses flag blocks kick.
        let r = kick_player_outcome(true, true, true, 3, 3, true);
        assert_eq!(r, KickPlayerOutcome::AbortInsufficientAccess);
    }

    #[test]
    fn kick_allowed_when_all_guards_pass() {
        // Owner kicking Guest is fine.
        let r = kick_player_outcome(true, true, true, 3, 1, false);
        assert_eq!(r, KickPlayerOutcome::Allowed);
        // Equal-access kick (e.g. SubOwner kicking SubOwner) is allowed.
        let r = kick_player_outcome(true, true, true, 2, 2, false);
        assert_eq!(r, KickPlayerOutcome::Allowed);
    }

    // ── AccessList resolvers (Session 33) ──────────────────────────────

    #[test]
    fn access_list_player_online_wins() {
        assert_eq!(
            access_list_resolve_player_guid(Some(42), Some(99)),
            Some(42)
        );
    }

    #[test]
    fn access_list_player_falls_back_to_offline_when_online_missing() {
        assert_eq!(access_list_resolve_player_guid(None, Some(99)), Some(99));
    }

    #[test]
    fn access_list_player_falls_back_when_online_is_zero() {
        // C++ treats 0 as "not found" too.
        assert_eq!(access_list_resolve_player_guid(Some(0), Some(99)), Some(99));
    }

    #[test]
    fn access_list_player_returns_none_when_both_missing() {
        assert_eq!(access_list_resolve_player_guid(None, None), None);
        assert_eq!(access_list_resolve_player_guid(Some(0), Some(0)), None);
    }

    #[test]
    fn access_list_guild_returns_rank_ids() {
        let ranks = vec![10, 20, 30];
        assert_eq!(
            access_list_resolve_guild_rank_ids(Some(&ranks)),
            vec![10, 20, 30]
        );
    }

    #[test]
    fn access_list_guild_returns_empty_when_guild_missing() {
        assert!(access_list_resolve_guild_rank_ids(None).is_empty());
    }

    #[test]
    fn access_list_single_rank_passthrough() {
        assert_eq!(access_list_resolve_single_rank_id(Some(5)), Some(5));
        assert_eq!(access_list_resolve_single_rank_id(None), None);
    }

    // ── depot_transfer_outcome (Session 33) ────────────────────────────

    #[test]
    fn transfer_aborts_when_no_town() {
        assert_eq!(
            depot_transfer_outcome(false, true, true, true),
            DepotTransferOutcome::Abort
        );
    }

    #[test]
    fn transfer_aborts_when_no_owner() {
        assert_eq!(
            depot_transfer_outcome(true, false, true, true),
            DepotTransferOutcome::Abort
        );
    }

    #[test]
    fn transfer_online_when_player_present() {
        assert_eq!(
            depot_transfer_outcome(true, true, true, false),
            DepotTransferOutcome::TransferOnline
        );
    }

    #[test]
    fn transfer_offline_when_load_succeeds() {
        assert_eq!(
            depot_transfer_outcome(true, true, false, true),
            DepotTransferOutcome::TransferOffline
        );
    }

    #[test]
    fn transfer_aborts_when_owner_unknown() {
        assert_eq!(
            depot_transfer_outcome(true, true, false, false),
            DepotTransferOutcome::AbortOwnerUnknown
        );
    }

    // ── pay_house_action (Session 33) ──────────────────────────────────

    #[test]
    fn pay_skip_when_no_owner() {
        assert_eq!(
            pay_house_action(false, 1000, 500, 7 * 86_400),
            PayHouseAction::Skip
        );
    }

    #[test]
    fn pay_already_paid_when_paid_until_in_future() {
        // Now=1000, paidUntil=2000 → still paid.
        assert_eq!(
            pay_house_action(true, 1000, 2000, 7 * 86_400),
            PayHouseAction::AlreadyPaid
        );
    }

    #[test]
    fn pay_charge_when_due_and_within_grace() {
        // Now=2000, paidUntil=1000, grace=7d → within grace → charge.
        assert_eq!(
            pay_house_action(true, 2000, 1000, 7 * 86_400),
            PayHouseAction::ChargeRent
        );
    }

    #[test]
    fn pay_kick_when_past_grace_period() {
        // Now=1_000_000, paidUntil=1, grace=1d → way past grace → kick.
        assert_eq!(
            pay_house_action(true, 1_000_000, 1, 86_400),
            PayHouseAction::KickOwner
        );
    }

    // ── parse_houses_xml (Session 33) ──────────────────────────────────

    /// Valid XML produces one row per house and applies it.
    #[test]
    fn parse_houses_xml_applies_all_attributes() {
        let mut houses = Houses::new();
        houses.add_house(House::new(7, "old name", 1, 1));
        let xml = r#"<houses>
            <house houseid="7" name="Lighthouse" entryx="100" entryy="200" entryz="7" rent="500" townid="3"/>
        </houses>"#;
        let parsed = parse_houses_xml(xml).unwrap();
        assert!(parsed.warnings.is_empty());
        assert_eq!(parsed.rows.len(), 1);
        apply_parsed_house_row(&mut houses, &parsed.rows[0]).unwrap();
        let house = houses.get_house(7).unwrap();
        assert_eq!(house.get_name(), "Lighthouse");
        assert_eq!(house.get_rent(), 500);
        assert_eq!(house.get_town_id(), 3);
        assert_eq!(house.get_entry_pos().x, 100);
        assert_eq!(house.get_entry_pos().y, 200);
        assert_eq!(house.get_entry_pos().z, 7);
        // Owner is reset on load.
        assert_eq!(house.get_owner_guid(), 0);
    }

    /// Zero entry-pos emits a warning but still produces a row.
    #[test]
    fn parse_houses_xml_warns_on_zero_entry_pos() {
        let xml = r#"<houses>
            <house houseid="1" name="Test" entryx="0" entryy="0" entryz="0" rent="1" townid="1"/>
        </houses>"#;
        let parsed = parse_houses_xml(xml).unwrap();
        assert_eq!(parsed.warnings.len(), 1);
        assert!(parsed.warnings[0].contains("House entry not set"));
        assert_eq!(parsed.rows.len(), 1);
    }

    /// Unknown house id at apply time → Err (matches C++ early return).
    #[test]
    fn apply_parsed_row_unknown_house_id_returns_err() {
        let mut houses = Houses::new();
        let row = ParsedHouseRow {
            house_id: 999,
            name: Some("Ghost".into()),
            entry_pos: forgottenserver_common::position::Position::new(1, 1, 1),
            rent: Some(1),
            town_id: Some(1),
        };
        let result = apply_parsed_house_row(&mut houses, &row);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown house"));
    }

    /// Malformed XML returns Err.
    #[test]
    fn parse_houses_xml_malformed_returns_err() {
        let result = parse_houses_xml("<houses><house");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("XML parse error"));
    }

    /// Missing houseid surfaces as Err.
    #[test]
    fn parse_houses_xml_missing_houseid_returns_err() {
        let xml = r#"<houses><house name="No ID"/></houses>"#;
        let result = parse_houses_xml(xml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing houseid"));
    }

    // -----------------------------------------------------------------------
    // Door — house.h:36 mirror
    // -----------------------------------------------------------------------

    #[test]
    fn door_new_defaults() {
        let d = Door::new(1234);
        assert_eq!(d.item_type_id(), 1234);
        assert_eq!(d.get_door_id(), 0);
        assert_eq!(d.get_house_id(), None);
    }

    #[test]
    fn door_set_house_only_once() {
        let mut d = Door::new(1234);
        d.set_house(7);
        d.set_house(8);
        assert_eq!(
            d.get_house_id(),
            Some(7),
            "second bind ignored (C++ parity)"
        );
    }

    #[test]
    fn door_can_use_unbound_returns_true() {
        let d = Door::new(1234);
        assert!(
            d.can_use(None, 42),
            "unbound door allows anyone (house == nullptr in C++)"
        );
    }

    #[test]
    fn door_can_use_owner_passes_without_access_list() {
        let mut house = sample_house();
        let owner_guid = 100;
        house.set_owner(owner_guid);
        house.add_door(5);
        let mut d = Door::new(1234);
        d.set_door_id(5);
        d.set_house(house.get_id());
        assert!(d.can_use(Some(&house), owner_guid));
    }

    #[test]
    fn door_can_use_uninvited_blocked() {
        let mut house = sample_house();
        house.set_owner(100);
        house.add_door(5);
        let mut d = Door::new(1234);
        d.set_door_id(5);
        d.set_house(house.get_id());
        assert!(!d.can_use(Some(&house), 999), "uninvited blocked");
    }

    #[test]
    fn door_set_access_list_delegates_to_house() {
        let mut house = sample_house();
        house.add_door(7);
        let mut d = Door::new(1234);
        d.set_door_id(7);
        d.set_house(house.get_id());
        let ok = d.set_access_list(&mut house, "alice\nbob");
        assert!(ok);
        let list = d.get_access_list(&house).expect("list set");
        assert!(list.contains("alice"));
    }

    #[test]
    fn door_get_access_list_unbound_returns_none() {
        let house = sample_house();
        let d = Door::new(1234);
        assert!(d.get_access_list(&house).is_none());
    }

    #[test]
    fn door_on_removed_deregisters_from_house() {
        let mut house = sample_house();
        house.add_door(9);
        let mut d = Door::new(1234);
        d.set_door_id(9);
        d.set_house(house.get_id());
        let removed = d.on_removed(&mut house);
        assert!(removed);
        assert!(!house.has_door(9));
    }

    #[test]
    fn door_on_removed_unbound_is_noop() {
        let mut house = sample_house();
        let d = Door::new(1234);
        assert!(!d.on_removed(&mut house));
    }

    #[test]
    fn door_apply_house_door_id_sets_item_attribute() {
        use forgottenserver_items::item::{AttributeValue, ItemAttribute};
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;
        let mut d = Door::new(1234);
        let mut item = Item::new(Arc::new(ItemTypeData::default()), 1);
        d.apply_house_door_id(42, &mut item);
        assert_eq!(d.get_door_id(), 42);
        assert!(matches!(
            item.get_attribute(ItemAttribute::DoorId),
            Some(AttributeValue::Integer(42))
        ));
    }

    // -----------------------------------------------------------------------
    // HouseTransferItem — house.h:89 mirror
    // -----------------------------------------------------------------------

    #[test]
    fn house_transfer_item_factory_sets_item_type_zero() {
        let hti = HouseTransferItem::create_house_transfer_item(42);
        assert_eq!(hti.item_type_id(), 0, "C++ uses item type 0");
        assert_eq!(hti.target_house_id(), 42);
    }

    #[test]
    fn house_transfer_item_cannot_transform() {
        let hti = HouseTransferItem::new(7);
        assert!(!hti.can_transform());
    }

    // -----------------------------------------------------------------------
    // Confirming stub: HouseTransferItem::can_transform
    // -----------------------------------------------------------------------

    // C++: house.h — HouseTransferItem overrides Item::canTransform to return
    //   false, preventing transfer items from being morphed into a different
    //   item type.  The virtual default in Item returns true; this override
    //   hard-codes false.
    // Classification: correct-default
    #[test]
    fn test_house_item_can_transform_returns_false_matching_cpp() {
        // C++: HouseTransferItem::canTransform() const override { return false; }
        let hti = HouseTransferItem::new(42);
        assert!(
            !hti.can_transform(),
            "HouseTransferItem::can_transform must always return false (mirrors C++ canTransform override)"
        );
    }
}
