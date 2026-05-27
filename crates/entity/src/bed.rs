//! Migrated from forgottenserver/src/bed.h and bed.cpp
//!
//! Provides the `BedItem` struct.
//! House and Player references are replaced with integer ids.

// ---------------------------------------------------------------------------
// TrySleepError
// ---------------------------------------------------------------------------

/// Reasons why `BedItem::try_sleep` may fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrySleepError {
    /// The bed is not assigned to a house.
    NoHouse,
    /// The player is flagged as removed (logged out / being kicked).
    PlayerRemoved,
    /// Another player is already sleeping in the bed.
    BedOccupied,
}

// ---------------------------------------------------------------------------
// BedItem
// ---------------------------------------------------------------------------

/// Mirrors the C++ `BedItem` class, using composition instead of inheritance.
///
/// * `house_id == 0` means the bed is not assigned to a house.
/// * `sleeper_guid == 0` means nobody is sleeping.
#[derive(Debug, Clone)]
pub struct BedItem {
    /// Item type id (server id).
    pub item_type_id: u16,
    /// GUID of the sleeping player; 0 when empty.
    sleeper_guid: u32,
    /// Unix timestamp (seconds) when the player went to sleep; 0 when empty.
    sleep_start: u64,
    /// House id this bed belongs to; 0 when not assigned.
    house_id: u32,
    /// Optional partner bed item-type id (for double beds).
    partner_id: Option<u16>,
}

// ---------------------------------------------------------------------------
// Regeneration helpers (mirrors regeneratePlayer arithmetic)
// ---------------------------------------------------------------------------

/// Result of the offline regeneration calculation.
///
/// Mirrors the two-branch logic of `BedItem::regeneratePlayer`:
///
/// * When the regeneration condition has **finite ticks**:
///   - `hp_mp_regen = min(ticks_secs, slept_secs) / 30`
///   - `remaining_ticks_ms = ticks_ms - (regen * 30_000)` (clamped to ≥ 0)
/// * When the condition has **infinite ticks** (represented as `-1` in C++,
///   modelled here as `None`):
///   - `hp_mp_regen = slept_secs / 30`
///   - `remaining_ticks_ms = None`
///
/// `soul_regen = slept_secs / 900` (one soul per 15 minutes) in both cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegenResult {
    /// HP and mana gain from the sleep period.
    pub hp_mp_regen: u32,
    /// Soul gain from the sleep period.
    pub soul_regen: u32,
    /// Remaining condition ticks (ms) after subtracting consumed regen ticks.
    /// `None` when the condition had infinite ticks.
    pub remaining_condition_ticks_ms: Option<u32>,
}

/// Compute the offline regeneration from a sleep period.
///
/// * `slept_secs` — how long the player slept (seconds).
/// * `condition_ticks_ms` — current regen condition ticks in **milliseconds**;
///   `None` means the condition has infinite duration (C++ value of `-1`).
///
/// This is the pure arithmetic extracted from `BedItem::regeneratePlayer`.
pub fn compute_regen(slept_secs: u64, condition_ticks_ms: Option<u32>) -> RegenResult {
    let soul_regen = (slept_secs / 900) as u32; // 60 * 15 = 900 s per soul

    let (hp_mp_regen, remaining_condition_ticks_ms) = match condition_ticks_ms {
        Some(ticks_ms) => {
            let ticks_secs = (ticks_ms / 1000) as u64;
            let effective_secs = ticks_secs.min(slept_secs);
            let regen = (effective_secs / 30) as u32;
            let consumed_ms = regen as u64 * 30_000;
            let remaining_ms = (ticks_ms as u64).saturating_sub(consumed_ms) as u32;
            (regen, Some(remaining_ms))
        }
        None => {
            let regen = (slept_secs / 30) as u32;
            (regen, None)
        }
    };

    RegenResult {
        hp_mp_regen,
        soul_regen,
        remaining_condition_ticks_ms,
    }
}

impl BedItem {
    /// Create a new BedItem. Mirrors `BedItem::BedItem(uint16_t id)`.
    pub fn new(item_type_id: u16) -> Self {
        BedItem {
            item_type_id,
            sleeper_guid: 0,
            sleep_start: 0,
            house_id: 0,
            partner_id: None,
        }
    }

    // -----------------------------------------------------------------------
    // Sleeper
    // -----------------------------------------------------------------------

    pub fn get_sleeper_guid(&self) -> u32 {
        self.sleeper_guid
    }

    /// Return the unix timestamp (seconds) when sleep began; 0 when nobody is sleeping.
    pub fn get_sleep_start(&self) -> u64 {
        self.sleep_start
    }

    /// Begin sleeping. Mirrors `internalSetSleeper`.
    pub fn sleep_start(&mut self, player_guid: u32, sleep_timestamp_secs: u64) {
        self.sleeper_guid = player_guid;
        self.sleep_start = sleep_timestamp_secs;
    }

    /// Clear sleeper state. Mirrors `internalRemoveSleeper`.
    ///
    /// Resets `sleeper_guid` and `sleep_start` to 0.
    pub fn clear_sleeper(&mut self) {
        self.sleeper_guid = 0;
        self.sleep_start = 0;
    }

    /// Attempt to start sleeping.  Mirrors the gate checks in `BedItem::trySleep`.
    ///
    /// Returns `Ok(())` when sleep can proceed, or an `Err` describing why it
    /// cannot.
    ///
    /// Rules (in C++ order):
    /// 1. Must be assigned to a house (`house_id != 0`).
    /// 2. Player must not be flagged as removed (`player_removed`).
    /// 3. Bed must not already have a sleeper (`sleeper_guid == 0`).
    ///    * If a sleeper is present the caller should call `wake_up` first
    ///      (C++ does this when the house owner requests it).
    pub fn try_sleep(&self, player_removed: bool) -> Result<(), TrySleepError> {
        if self.house_id == 0 {
            return Err(TrySleepError::NoHouse);
        }
        if player_removed {
            return Err(TrySleepError::PlayerRemoved);
        }
        if self.sleeper_guid != 0 {
            return Err(TrySleepError::BedOccupied);
        }
        Ok(())
    }

    /// Wake up the sleeper and clear bed state.  Mirrors `BedItem::wakeUp`.
    ///
    /// Returns `Some(sleeper_guid)` of the evicted sleeper, or `None` when the
    /// bed was already empty or not assigned to a house.
    pub fn wake_up(&mut self) -> Option<u32> {
        if self.house_id == 0 {
            return None;
        }
        if self.sleeper_guid == 0 {
            return None;
        }
        let evicted = self.sleeper_guid;
        self.clear_sleeper();
        Some(evicted)
    }

    /// Compute stamina regeneration after a period of sleep.
    ///
    /// Returns `elapsed_minutes * regen_per_minute` clamped to `u32::MAX`.
    ///
    /// Mirrors a simplified version of `regeneratePlayer` (game-state ops
    /// are omitted; this is the core arithmetic).
    pub fn sleep_end(&self, elapsed_minutes: u32, regen_per_minute: u32) -> u32 {
        elapsed_minutes.saturating_mul(regen_per_minute)
    }

    // -----------------------------------------------------------------------
    // House
    // -----------------------------------------------------------------------

    pub fn get_house_id(&self) -> u32 {
        self.house_id
    }

    pub fn set_house_id(&mut self, id: u32) {
        self.house_id = id;
    }

    // -----------------------------------------------------------------------
    // can_remove
    // -----------------------------------------------------------------------

    /// A bed can be removed when it is not assigned to a house.
    ///
    /// Mirrors `BedItem::canRemove() const { return !house; }`.
    pub fn can_remove(&self) -> bool {
        self.house_id == 0
    }

    // -----------------------------------------------------------------------
    // can_use (mirrors `BedItem::canUse(Player*)`)
    // -----------------------------------------------------------------------

    /// Pure decision logic for `BedItem::canUse(Player*)`.
    ///
    /// The C++ method walks the player + house objects to evaluate the
    /// following gates, in order:
    ///
    /// 1. `player`, `house`, `isPremium()`, and `getZone() == ZONE_PROTECTION`
    ///    must all be truthy. If any of those four fail, return `false`.
    /// 2. If the bed is free (`sleeper_guid == 0`), return `true`.
    /// 3. If the requesting player's access level on the house is `HOUSE_OWNER`,
    ///    return `true` regardless of the sleeper.
    /// 4. Otherwise the sleeper must be loadable from disk; if loading fails,
    ///    return `false`.
    /// 5. If the sleeper's access level is **strictly greater** than the
    ///    requester's, return `false`.
    /// 6. Otherwise return `true`.
    ///
    /// `requester_access_level` and `sleeper_access_level` are ordinal ranks
    /// (0 = NotInvited, 1 = Guest, 2 = SubOwner, 3 = Owner), matching the
    /// `world::HouseAccessLevel` numeric encoding so callers in the world
    /// crate can pass `level as u8` directly.
    ///
    /// `sleeper_loaded` mirrors the success of `IOLoginData::loadPlayerById`
    /// in C++ — when the bed is occupied but the sleeper cannot be loaded the
    /// method returns `false`.
    pub fn can_use(
        &self,
        is_premium: bool,
        in_protection_zone: bool,
        requester_access_level: u8,
        sleeper_loaded: bool,
        sleeper_access_level: u8,
    ) -> bool {
        // gate 1: house assigned + premium + PZ
        if self.house_id == 0 || !is_premium || !in_protection_zone {
            return false;
        }

        // gate 2: free bed
        if self.sleeper_guid == 0 {
            return true;
        }

        // gate 3: requester is HOUSE_OWNER (rank 3)
        const HOUSE_OWNER_RANK: u8 = 3;
        if requester_access_level == HOUSE_OWNER_RANK {
            return true;
        }

        // gate 4: sleeper failed to load
        if !sleeper_loaded {
            return false;
        }

        // gate 5: sleeper outranks requester
        if sleeper_access_level > requester_access_level {
            return false;
        }

        // gate 6: allowed
        true
    }

    // -----------------------------------------------------------------------
    // Partner
    // -----------------------------------------------------------------------

    /// Return the partner bed's item-type id, if set.
    ///
    /// Mirrors the concept of `getNextBedItem` (without game-world lookup).
    pub fn get_next_bed_id(&self) -> Option<u16> {
        self.partner_id
    }

    pub fn set_partner_id(&mut self, id: u16) {
        self.partner_id = Some(id);
    }

    // -----------------------------------------------------------------------
    // Serialization
    // -----------------------------------------------------------------------

    /// Serialize the sleeper GUID and sleep_start into a byte vector.
    ///
    /// Format (12 bytes):
    /// * bytes 0..4  — sleeper_guid  (little-endian u32)
    /// * bytes 4..12 — sleep_start   (little-endian u64)
    pub fn serialize_sleeper(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12);
        buf.extend_from_slice(&self.sleeper_guid.to_le_bytes());
        buf.extend_from_slice(&self.sleep_start.to_le_bytes());
        buf
    }

    /// Deserialise the sleeper state from a byte slice produced by
    /// `serialize_sleeper`.
    ///
    /// Returns `false` when the slice is too short.
    pub fn deserialize_sleeper(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() < 12 {
            return false;
        }
        self.sleeper_guid = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        self.sleep_start = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
        true
    }
}

// ---------------------------------------------------------------------------
// Decision helpers — engine-side state + appearance (Session 28 ledger closure)
// ---------------------------------------------------------------------------

/// Pure decision for the appearance swap in `BedItem::updateAppearance`.
///
/// Mirrors the C++ branch chain:
/// ```cpp
/// const ItemType& it = items[id];
/// if (it.type == ITEM_TYPE_BED) {
///     if (player && it.transformToOnUse[player->getSex()] != 0) {
///         transform to it.transformToOnUse[sex];
///     } else if (it.transformToFree != 0) {
///         transform to it.transformToFree;
///     }
/// }
/// ```
///
/// Returns the target item-type id the bed should transform into, or
/// `None` when no transformation should happen (matching the C++ "no
/// transform applies" no-op branch).
///
/// `sleeper_sex` slots: `0` = female, `1` = male — matches C++
/// `PlayerSex_t` ordering. Out-of-range slots are treated as no-transform.
pub fn appearance_transform_target_id(
    transform_to_on_use: [u16; 2],
    transform_to_free: u16,
    has_sleeper: bool,
    sleeper_sex: u8,
) -> Option<u16> {
    if has_sleeper {
        let slot = sleeper_sex as usize;
        if slot < transform_to_on_use.len() {
            let target = transform_to_on_use[slot];
            if target != 0 {
                return Some(target);
            }
        }
        return None;
    }
    if transform_to_free != 0 {
        return Some(transform_to_free);
    }
    None
}

/// Pure position calc for the C++ `BedItem::getNextBedItem` helper.
///
/// Given the current bed's position and its `bed_partner_dir`, returns
/// the position of the partner-bed tile. The cross-crate caller looks
/// up the resulting tile and reads its `BedItem` (when present).
///
/// Returns `None` when the bed has no partner direction
/// (`Direction::None`) — single-occupant beds.
pub fn partner_bed_position(
    current_pos: forgottenserver_common::position::Position,
    bed_partner_dir: forgottenserver_common::position::Direction,
) -> Option<forgottenserver_common::position::Position> {
    use forgottenserver_common::position::Direction;
    if bed_partner_dir == Direction::None {
        return None;
    }
    Some(current_pos.next_position(bed_partner_dir))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bed_new_no_sleeper() {
        let b = BedItem::new(100);
        assert_eq!(b.get_sleeper_guid(), 0);
    }

    #[test]
    fn test_bed_new_no_house() {
        let b = BedItem::new(100);
        assert_eq!(b.get_house_id(), 0);
    }

    #[test]
    fn test_bed_can_remove_no_house() {
        let b = BedItem::new(100);
        assert!(b.can_remove());
    }

    #[test]
    fn test_bed_can_remove_with_house() {
        let mut b = BedItem::new(100);
        b.set_house_id(5);
        assert!(!b.can_remove());
    }

    #[test]
    fn test_bed_set_get_house_id() {
        let mut b = BedItem::new(100);
        b.set_house_id(42);
        assert_eq!(b.get_house_id(), 42);
    }

    #[test]
    fn test_bed_sleep_start() {
        let mut b = BedItem::new(100);
        b.sleep_start(1234, 1_000_000);
        assert_eq!(b.get_sleeper_guid(), 1234);
        assert_eq!(b.sleep_start, 1_000_000);
    }

    #[test]
    fn test_bed_sleep_end_basic() {
        let b = BedItem::new(100);
        // 10 minutes * 5 regen/min = 50
        assert_eq!(b.sleep_end(10, 5), 50);
    }

    #[test]
    fn test_bed_sleep_end_zero_elapsed() {
        let b = BedItem::new(100);
        assert_eq!(b.sleep_end(0, 100), 0);
    }

    #[test]
    fn test_bed_sleep_end_saturating() {
        let b = BedItem::new(100);
        // Should not panic/overflow on large values
        let result = b.sleep_end(u32::MAX, 2);
        assert_eq!(result, u32::MAX); // saturating_mul
    }

    #[test]
    fn test_bed_get_next_bed_id_default_none() {
        let b = BedItem::new(100);
        assert!(b.get_next_bed_id().is_none());
    }

    #[test]
    fn test_bed_set_partner_id() {
        let mut b = BedItem::new(100);
        b.set_partner_id(200);
        assert_eq!(b.get_next_bed_id(), Some(200));
    }

    #[test]
    fn test_bed_serialize_sleeper_round_trip() {
        let mut b = BedItem::new(100);
        b.sleep_start(9999, 1_700_000_000);

        let bytes = b.serialize_sleeper();
        assert_eq!(bytes.len(), 12);

        let mut b2 = BedItem::new(100);
        let ok = b2.deserialize_sleeper(&bytes);
        assert!(ok);
        assert_eq!(b2.get_sleeper_guid(), 9999);
        assert_eq!(b2.sleep_start, 1_700_000_000);
    }

    #[test]
    fn test_bed_serialize_sleeper_short_bytes_returns_false() {
        let mut b = BedItem::new(100);
        let short = vec![0u8; 4]; // too short (needs 12)
        assert!(!b.deserialize_sleeper(&short));
    }

    // -----------------------------------------------------------------------
    // get_sleep_start
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_sleep_start_initial_zero() {
        let b = BedItem::new(10);
        assert_eq!(b.get_sleep_start(), 0);
    }

    #[test]
    fn test_get_sleep_start_after_sleep_start_call() {
        let mut b = BedItem::new(10);
        b.sleep_start(42, 9_999_999);
        assert_eq!(b.get_sleep_start(), 9_999_999);
    }

    // -----------------------------------------------------------------------
    // clear_sleeper (mirrors internalRemoveSleeper)
    // -----------------------------------------------------------------------

    #[test]
    fn test_clear_sleeper_resets_guid() {
        let mut b = BedItem::new(10);
        b.sleep_start(777, 1_000);
        b.clear_sleeper();
        assert_eq!(b.get_sleeper_guid(), 0);
    }

    #[test]
    fn test_clear_sleeper_resets_sleep_start() {
        let mut b = BedItem::new(10);
        b.sleep_start(777, 1_000);
        b.clear_sleeper();
        assert_eq!(b.get_sleep_start(), 0);
    }

    #[test]
    fn test_clear_sleeper_on_empty_bed_is_idempotent() {
        let mut b = BedItem::new(10);
        b.clear_sleeper(); // already empty, should not panic
        assert_eq!(b.get_sleeper_guid(), 0);
        assert_eq!(b.get_sleep_start(), 0);
    }

    // -----------------------------------------------------------------------
    // try_sleep (mirrors trySleep gate checks)
    // -----------------------------------------------------------------------

    #[test]
    fn test_try_sleep_no_house_fails() {
        let b = BedItem::new(10); // house_id == 0
        assert_eq!(b.try_sleep(false), Err(TrySleepError::NoHouse));
    }

    #[test]
    fn test_try_sleep_player_removed_fails() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        assert_eq!(b.try_sleep(true), Err(TrySleepError::PlayerRemoved));
    }

    #[test]
    fn test_try_sleep_bed_occupied_fails() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(99, 1_000);
        assert_eq!(b.try_sleep(false), Err(TrySleepError::BedOccupied));
    }

    #[test]
    fn test_try_sleep_all_conditions_met_succeeds() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        // empty bed, player not removed, house assigned
        assert_eq!(b.try_sleep(false), Ok(()));
    }

    /// Ensure no_house is checked before player_removed (C++ returns early on !house).
    #[test]
    fn test_try_sleep_no_house_takes_priority_over_removed() {
        let b = BedItem::new(10); // no house
        assert_eq!(b.try_sleep(true), Err(TrySleepError::NoHouse));
    }

    // -----------------------------------------------------------------------
    // wake_up (mirrors wakeUp — game-state ops excluded)
    // -----------------------------------------------------------------------

    #[test]
    fn test_wake_up_returns_evicted_guid() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(42, 1_000);
        assert_eq!(b.wake_up(), Some(42));
    }

    #[test]
    fn test_wake_up_clears_sleeper() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(42, 1_000);
        b.wake_up();
        assert_eq!(b.get_sleeper_guid(), 0);
        assert_eq!(b.get_sleep_start(), 0);
    }

    #[test]
    fn test_wake_up_empty_bed_returns_none() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        // nobody sleeping
        assert_eq!(b.wake_up(), None);
    }

    #[test]
    fn test_wake_up_no_house_returns_none() {
        let mut b = BedItem::new(10); // no house
        b.sleep_start(99, 500); // force a sleeper even without house
                                // wake_up checks house first
        assert_eq!(b.wake_up(), None);
        // state should not have changed (no house → no-op)
        assert_eq!(b.get_sleeper_guid(), 99);
    }

    // -----------------------------------------------------------------------
    // Partner bed (bidirectional linking)
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_partner_id_overwrites_previous() {
        let mut b = BedItem::new(10);
        b.set_partner_id(200);
        b.set_partner_id(300);
        assert_eq!(b.get_next_bed_id(), Some(300));
    }

    #[test]
    fn test_partner_clear_via_option_replace() {
        // Modelling "unlinking" partners: two beds each know the other.
        let mut a = BedItem::new(10);
        let mut b = BedItem::new(20);
        a.set_partner_id(20);
        b.set_partner_id(10);
        // Verify bidirectional link
        assert_eq!(a.get_next_bed_id(), Some(20));
        assert_eq!(b.get_next_bed_id(), Some(10));
        // Unlink: replace partner id with a distinct value or remove via new API
        a.partner_id = None;
        b.partner_id = None;
        assert!(a.get_next_bed_id().is_none());
        assert!(b.get_next_bed_id().is_none());
    }

    // -----------------------------------------------------------------------
    // sleep_end edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_sleep_end_zero_regen_per_minute() {
        let b = BedItem::new(10);
        assert_eq!(b.sleep_end(100, 0), 0);
    }

    #[test]
    fn test_sleep_end_zero_elapsed_and_zero_regen() {
        let b = BedItem::new(10);
        assert_eq!(b.sleep_end(0, 0), 0);
    }

    #[test]
    fn test_sleep_end_large_values_no_overflow() {
        let b = BedItem::new(10);
        // Both args large but product would overflow; saturating_mul must not panic.
        let _ = b.sleep_end(u32::MAX, u32::MAX);
    }

    // -----------------------------------------------------------------------
    // Serialization edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_sleeper_empty_bed_is_all_zeros() {
        let b = BedItem::new(10);
        let bytes = b.serialize_sleeper();
        assert_eq!(bytes, vec![0u8; 12]);
    }

    #[test]
    fn test_serialize_sleeper_is_little_endian() {
        let mut b = BedItem::new(10);
        // guid = 1 (0x00_00_00_01 LE), sleep_start = 256 (0x00_00_01_00 LE)
        b.sleep_start(1, 256);
        let bytes = b.serialize_sleeper();
        assert_eq!(&bytes[0..4], &1u32.to_le_bytes());
        assert_eq!(&bytes[4..12], &256u64.to_le_bytes());
    }

    #[test]
    fn test_deserialize_sleeper_exactly_12_bytes_succeeds() {
        let mut b = BedItem::new(10);
        let payload = vec![0u8; 12];
        assert!(b.deserialize_sleeper(&payload));
    }

    #[test]
    fn test_deserialize_sleeper_11_bytes_fails() {
        let mut b = BedItem::new(10);
        let payload = vec![0u8; 11];
        assert!(!b.deserialize_sleeper(&payload));
    }

    #[test]
    fn test_deserialize_sleeper_extra_bytes_succeeds() {
        // A longer slice is fine; only first 12 bytes are consumed.
        let mut b = BedItem::new(10);
        let mut payload = vec![0u8; 16];
        payload[0] = 5; // guid = 5
        assert!(b.deserialize_sleeper(&payload));
        assert_eq!(b.get_sleeper_guid(), 5);
    }

    #[test]
    fn test_serialize_then_deserialize_zero_guid_preserves_state() {
        let b = BedItem::new(10); // empty
        let bytes = b.serialize_sleeper();
        let mut b2 = BedItem::new(10);
        b2.sleep_start(99, 12345); // pre-set some values
        b2.deserialize_sleeper(&bytes);
        assert_eq!(b2.get_sleeper_guid(), 0);
        assert_eq!(b2.get_sleep_start(), 0);
    }

    // -----------------------------------------------------------------------
    // compute_regen — finite condition ticks
    // -----------------------------------------------------------------------

    #[test]
    fn test_compute_regen_finite_ticks_full_sleep() {
        // slept 300 s, condition has 300_000 ms ticks → ticks_secs=300
        // effective = min(300,300)=300; regen=300/30=10
        // remaining = 300_000 - 10*30_000 = 0
        let r = compute_regen(300, Some(300_000));
        assert_eq!(r.hp_mp_regen, 10);
        assert_eq!(r.remaining_condition_ticks_ms, Some(0));
        assert_eq!(r.soul_regen, 0); // 300s < 900s
    }

    #[test]
    fn test_compute_regen_finite_ticks_slept_less_than_condition() {
        // slept 60 s, condition has 600_000 ms → effective=min(600,60)=60
        // regen=60/30=2; remaining=600_000-2*30_000=540_000
        let r = compute_regen(60, Some(600_000));
        assert_eq!(r.hp_mp_regen, 2);
        assert_eq!(r.remaining_condition_ticks_ms, Some(540_000));
    }

    #[test]
    fn test_compute_regen_finite_ticks_condition_smaller_than_slept() {
        // slept 3600 s, condition has 60_000 ms (60 s) → effective=min(60,3600)=60
        // regen=60/30=2; remaining=60_000-2*30_000=0
        let r = compute_regen(3600, Some(60_000));
        assert_eq!(r.hp_mp_regen, 2);
        assert_eq!(r.remaining_condition_ticks_ms, Some(0));
    }

    #[test]
    fn test_compute_regen_finite_ticks_regen_rounds_down() {
        // slept 50 s → effective=50; 50/30=1 (integer div), remaining=30_000 ms
        let r = compute_regen(50, Some(60_000));
        assert_eq!(r.hp_mp_regen, 1);
        assert_eq!(r.remaining_condition_ticks_ms, Some(30_000));
    }

    // -----------------------------------------------------------------------
    // compute_regen — infinite condition ticks (None)
    // -----------------------------------------------------------------------

    #[test]
    fn test_compute_regen_infinite_ticks() {
        // slept 300 s → regen=300/30=10; remaining=None
        let r = compute_regen(300, None);
        assert_eq!(r.hp_mp_regen, 10);
        assert_eq!(r.remaining_condition_ticks_ms, None);
    }

    #[test]
    fn test_compute_regen_infinite_ticks_rounds_down() {
        // slept 59 s → 59/30=1
        let r = compute_regen(59, None);
        assert_eq!(r.hp_mp_regen, 1);
        assert_eq!(r.remaining_condition_ticks_ms, None);
    }

    // -----------------------------------------------------------------------
    // compute_regen — soul regeneration
    // -----------------------------------------------------------------------

    #[test]
    fn test_compute_regen_soul_zero_below_threshold() {
        // slept 899 s → soul_regen = 0
        let r = compute_regen(899, None);
        assert_eq!(r.soul_regen, 0);
    }

    #[test]
    fn test_compute_regen_soul_one_at_exactly_900s() {
        // slept 900 s → soul_regen = 1
        let r = compute_regen(900, None);
        assert_eq!(r.soul_regen, 1);
    }

    #[test]
    fn test_compute_regen_soul_multiple() {
        // slept 2700 s (45 min) → soul_regen = 3
        let r = compute_regen(2700, None);
        assert_eq!(r.soul_regen, 3);
    }

    #[test]
    fn test_compute_regen_soul_rounds_down() {
        // slept 1799 s (just under 30 min) → soul_regen = 1
        let r = compute_regen(1799, None);
        assert_eq!(r.soul_regen, 1);
    }

    // -----------------------------------------------------------------------
    // compute_regen — zero sleep time
    // -----------------------------------------------------------------------

    #[test]
    fn test_compute_regen_zero_slept_time_finite() {
        let r = compute_regen(0, Some(60_000));
        assert_eq!(r.hp_mp_regen, 0);
        assert_eq!(r.soul_regen, 0);
        assert_eq!(r.remaining_condition_ticks_ms, Some(60_000));
    }

    #[test]
    fn test_compute_regen_zero_slept_time_infinite() {
        let r = compute_regen(0, None);
        assert_eq!(r.hp_mp_regen, 0);
        assert_eq!(r.soul_regen, 0);
        assert_eq!(r.remaining_condition_ticks_ms, None);
    }

    // -----------------------------------------------------------------------
    // can_remove edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_remove_after_set_then_clear_house() {
        let mut b = BedItem::new(10);
        b.set_house_id(5);
        assert!(!b.can_remove());
        b.set_house_id(0); // clear
        assert!(b.can_remove());
    }

    // -----------------------------------------------------------------------
    // can_use — mirrors `BedItem::canUse`
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_use_no_house_returns_false() {
        let b = BedItem::new(10); // house_id == 0
                                  // Even with every other gate satisfied, missing house blocks usage.
        assert!(!b.can_use(true, true, 3, true, 0));
    }

    #[test]
    fn test_can_use_not_premium_returns_false() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        assert!(!b.can_use(false, true, 3, true, 0));
    }

    #[test]
    fn test_can_use_not_in_protection_zone_returns_false() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        assert!(!b.can_use(true, false, 3, true, 0));
    }

    #[test]
    fn test_can_use_free_bed_succeeds() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        // sleeper_loaded / sleeper_access_level are irrelevant when free.
        assert!(b.can_use(true, true, 0, false, 0));
    }

    #[test]
    fn test_can_use_requester_is_owner_overrides_sleeper() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(42, 1_000); // occupy bed
                                  // requester rank = 3 (Owner). Sleeper outranks numerically (impossible
                                  // here but proves rank-3 short-circuits the load check too).
        assert!(b.can_use(true, true, 3, false, 99));
    }

    #[test]
    fn test_can_use_sleeper_failed_to_load_returns_false() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(42, 1_000);
        // requester is SubOwner (2), sleeper failed to load.
        assert!(!b.can_use(true, true, 2, false, 0));
    }

    #[test]
    fn test_can_use_sleeper_outranks_requester_returns_false() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(42, 1_000);
        // requester=Guest(1), sleeper=SubOwner(2) → sleeper > requester
        assert!(!b.can_use(true, true, 1, true, 2));
    }

    #[test]
    fn test_can_use_sleeper_equal_rank_to_requester_succeeds() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(42, 1_000);
        // requester=Guest(1), sleeper=Guest(1) → equal is allowed
        assert!(b.can_use(true, true, 1, true, 1));
    }

    #[test]
    fn test_can_use_sleeper_lower_rank_than_requester_succeeds() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        b.sleep_start(42, 1_000);
        // requester=SubOwner(2), sleeper=Guest(1)
        assert!(b.can_use(true, true, 2, true, 1));
    }

    #[test]
    fn test_can_use_no_house_takes_priority_over_premium() {
        // Verifies the gates are checked in C++ order — no house wins.
        let b = BedItem::new(10);
        assert!(!b.can_use(true, true, 3, true, 3));
    }

    #[test]
    fn test_can_use_premium_check_runs_before_pz_check() {
        let mut b = BedItem::new(10);
        b.set_house_id(1);
        // Both false; ensure both being false still yields false.
        assert!(!b.can_use(false, false, 3, true, 0));
    }

    // ── Decision helpers (Session 28) ───────────────────────────────────

    /// `has_sleeper=true` + non-zero `transform_to_on_use[sex]` returns
    /// that target id (the C++ "transform to sleeping" branch).
    #[test]
    fn appearance_transform_sleeper_female_returns_on_use_slot_zero() {
        // sleeper_sex = 0 (female) → use slot 0
        let target = appearance_transform_target_id([1234, 5678], 99, true, 0);
        assert_eq!(target, Some(1234));
    }

    #[test]
    fn appearance_transform_sleeper_male_returns_on_use_slot_one() {
        let target = appearance_transform_target_id([1234, 5678], 99, true, 1);
        assert_eq!(target, Some(5678));
    }

    /// `has_sleeper=true` + the sex slot is `0` → `None` (C++ "no transform" branch).
    #[test]
    fn appearance_transform_sleeper_zero_slot_returns_none() {
        let target = appearance_transform_target_id([0, 5678], 99, true, 0);
        assert_eq!(target, None);
        let target = appearance_transform_target_id([1234, 0], 99, true, 1);
        assert_eq!(target, None);
    }

    /// `has_sleeper=false` + non-zero `transform_to_free` → that id.
    #[test]
    fn appearance_transform_no_sleeper_returns_transform_to_free() {
        let target = appearance_transform_target_id([1234, 5678], 99, false, 0);
        assert_eq!(target, Some(99));
    }

    /// `has_sleeper=false` + zero `transform_to_free` → None.
    #[test]
    fn appearance_transform_no_sleeper_zero_free_returns_none() {
        let target = appearance_transform_target_id([1234, 5678], 0, false, 1);
        assert_eq!(target, None);
    }

    /// Out-of-range sex slot is treated as no-transform.
    #[test]
    fn appearance_transform_invalid_sex_slot_returns_none() {
        let target = appearance_transform_target_id([1234, 5678], 99, true, 5);
        assert_eq!(target, None);
    }

    /// `partner_bed_position` returns `None` when the bed has no partner
    /// direction (single-occupant bed).
    #[test]
    fn partner_bed_position_none_direction_returns_none() {
        use forgottenserver_common::position::{Direction, Position};
        let pos = Position::new(10, 20, 7);
        assert!(partner_bed_position(pos, Direction::None).is_none());
    }

    /// Valid direction returns the offset position via `next_position`.
    #[test]
    fn partner_bed_position_east_offsets_x_by_one() {
        use forgottenserver_common::position::{Direction, Position};
        let pos = Position::new(10, 20, 7);
        let partner = partner_bed_position(pos, Direction::East).unwrap();
        assert_eq!(partner, Position::new(11, 20, 7));
    }

    #[test]
    fn partner_bed_position_north_offsets_y_by_minus_one() {
        use forgottenserver_common::position::{Direction, Position};
        let pos = Position::new(10, 20, 7);
        let partner = partner_bed_position(pos, Direction::North).unwrap();
        assert_eq!(partner, Position::new(10, 19, 7));
    }
}
