// Migrated from forgottenserver/src/housetile.h + housetile.cpp
//
// HouseTile wraps a Tile (composition) with an associated house ID.
// The C++ HouseTile extends DynamicTile, so our HouseTile uses a Dynamic
// Tile internally.
//
// Key behaviour from C++:
//   - queryAdd for non-members (creatures not invited) → deny
//   - For items: if actor not invited → deny (optional config in C++,
//     modelled here as always-deny for non-members)

use forgottenserver_common::position::Position;

use crate::tile::Tile;

// ---------------------------------------------------------------------------
// Error type for queryAdd
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddError {
    /// Mirrors RETURNVALUE_PLAYERISNOTINVITED
    NotInvited,
    /// Mirrors RETURNVALUE_NOTPOSSIBLE
    NotPossible,
}

// ---------------------------------------------------------------------------
// HouseTile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HouseTile {
    tile: Tile,
    house_id: u32,
}

impl HouseTile {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    /// Creates a new HouseTile at (x, y, z) belonging to `house_id`.
    ///
    /// Internally backed by a DynamicTile (matching the C++ DynamicTile
    /// inheritance).
    pub fn new(x: u16, y: u16, z: u8, house_id: u32) -> Self {
        HouseTile {
            tile: Tile::new_dynamic(x, y, z),
            house_id,
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Returns the house ID this tile belongs to.
    pub fn house_id(&self) -> u32 {
        self.house_id
    }

    /// Returns the position of the underlying tile.
    pub fn get_position(&self) -> Position {
        self.tile.position
    }

    /// Read-only access to the underlying tile.
    pub fn inner_tile(&self) -> &Tile {
        &self.tile
    }

    /// Mutable access to the underlying tile.
    pub fn inner_tile_mut(&mut self) -> &mut Tile {
        &mut self.tile
    }

    // -----------------------------------------------------------------------
    // Query — mirrors HouseTile::queryAdd
    // -----------------------------------------------------------------------

    /// Non-member creatures are denied entry.
    ///
    /// In C++ this checks `house->isInvited(player)`.  Since House entities
    /// don't exist yet, we model this as a stub that always returns
    /// `AddError::NotInvited` — callers that hold a membership token can
    /// bypass this by calling the underlying `inner_tile_mut()` directly.
    pub fn query_add_for_non_member(&self) -> Result<(), AddError> {
        Err(AddError::NotInvited)
    }

    /// Non-creature (item) additions by non-members are also denied.
    pub fn query_add_item_for_non_member(&self) -> Result<(), AddError> {
        Err(AddError::NotInvited)
    }

    /// Non-player creatures (monsters, NPCs) are always denied.
    ///
    /// Mirrors C++ `queryAdd` branch: `if (!player) return RETURNVALUE_NOTPOSSIBLE`.
    pub fn query_add_non_player_creature(&self) -> Result<(), AddError> {
        Err(AddError::NotPossible)
    }

    /// Removal by a non-member actor is denied.
    ///
    /// Mirrors C++ `queryRemove` with `ONLY_INVITED_CAN_MOVE_HOUSE_ITEMS` logic:
    /// returns `RETURNVALUE_PLAYERISNOTINVITED` when the actor is not invited.
    pub fn query_remove_for_non_member(&self) -> Result<(), AddError> {
        Err(AddError::NotInvited)
    }

    /// Adds an item to the underlying tile, mirroring `HouseTile::addThing`
    /// and `internalAddThing`.
    ///
    /// In C++ both methods call `updateHouse(item)` which registers doors and
    /// beds with the house.  Since `House` entities are not yet modelled, the
    /// Rust equivalent simply delegates to the underlying tile.
    pub fn add_item(&mut self, item: forgottenserver_items::item::Item) {
        self.tile.add_item(item);
    }
}

// ---------------------------------------------------------------------------
// queryDestination decision helper (Session 22 ledger closure)
// ---------------------------------------------------------------------------

/// Outcome of `HouseTile::queryDestination` for the non-member branch.
/// The Rust port returns the routing decision as data so cross-crate
/// callers (which own the Map and can resolve `getTile(Position)`)
/// dispatch on it.
///
/// Mirrors the C++ control flow in `HouseTile::queryDestination`:
///   * invited → delegate to base Tile::queryDestination
///   * not invited + entry tile exists → entry
///   * not invited + entry missing + temple tile exists → temple
///   * not invited + both missing → null (caller falls back to `Tile::nullptr_tile`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryDestinationOutcome {
    /// Caller should defer to `Tile::queryDestination` on the actual tile.
    DelegateToTile,
    /// Caller should route the thing to the house's entry-position tile.
    RouteToEntry,
    /// Caller should route the thing to the player's temple-position
    /// tile (entry tile didn't resolve).
    RouteToTemple,
    /// Both entry and temple lookups failed; the C++ side substitutes
    /// `Tile::nullptr_tile` (a sentinel sink tile). Rust callers should
    /// drop the move or surface an error.
    RouteToNull,
}

/// Pure decision logic for `HouseTile::queryDestination`. Cross-crate
/// callers compute `invited`, then look up entry-pos / temple-pos tiles
/// on `g_game.map` and feed the boolean "tile resolved" flags here.
///
/// Mirrors the C++ inline branch chain exactly — the only loss of
/// fidelity is that the C++ method prints a console warning when the
/// entry tile is missing; that side-effect is the caller's job because
/// the error message references the house name/id, which this layer
/// doesn't know about.
pub fn query_destination_for_non_member(
    invited: bool,
    entry_tile_resolved: bool,
    temple_tile_resolved: bool,
) -> QueryDestinationOutcome {
    if invited {
        return QueryDestinationOutcome::DelegateToTile;
    }
    if entry_tile_resolved {
        return QueryDestinationOutcome::RouteToEntry;
    }
    if temple_tile_resolved {
        return QueryDestinationOutcome::RouteToTemple;
    }
    QueryDestinationOutcome::RouteToNull
}

// ---------------------------------------------------------------------------
// updateHouse classification helper (Session 22 ledger closure)
// ---------------------------------------------------------------------------

/// Side-effect a caller of `HouseTile::updateHouse` should perform.
///
/// Mirrors the C++ branch chain in `HouseTile::updateHouse(item)`:
///   * Door with non-zero `door_id` → `house->addDoor(door)`
///   * BedItem → `house->addBed(bed)`
///   * Otherwise no-op (Door with `door_id == 0`, or anything that's
///     not a Door/Bed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateHouseAction {
    /// Register a Door with the house. `door_id` is the C++
    /// `Door::getDoorId()` value.
    AddDoor { door_id: u32 },
    /// Register a Bed with the house.
    AddBed,
    /// No-op (the C++ method silently returns).
    None,
}

/// Pure classification for `HouseTile::updateHouse`. Cross-crate
/// callers (which can resolve `item->getDoor()` / `item->getBed()` via
/// virtual dispatch) feed the booleans + door-id here.
///
/// Precedence matches C++: door takes priority over bed when both
/// flags are true (the C++ `if (door) … else if (bed) …` ladder).
/// Door classification with `door_id == 0` returns `None` because the
/// C++ side guards `if (door->getDoorId() != 0)` before registering.
pub fn classify_item_for_update_house(
    is_door: bool,
    door_id: u32,
    is_bed: bool,
) -> UpdateHouseAction {
    if is_door {
        if door_id != 0 {
            return UpdateHouseAction::AddDoor { door_id };
        }
        return UpdateHouseAction::None;
    }
    if is_bed {
        return UpdateHouseAction::AddBed;
    }
    UpdateHouseAction::None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_items::item::Item;
    use forgottenserver_items::items_registry::ItemTypeData;
    use std::sync::Arc;

    fn make_item(id: u16) -> Item {
        let data = ItemTypeData {
            id,
            ..Default::default()
        };
        Item::new(Arc::new(data), 1)
    }

    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    #[test]
    fn house_tile_new_stores_house_id() {
        let ht = HouseTile::new(10, 20, 7, 42);
        assert_eq!(ht.house_id(), 42);
    }

    #[test]
    fn house_tile_new_position_correct() {
        let ht = HouseTile::new(100, 200, 5, 1);
        let pos = ht.get_position();
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
        assert_eq!(pos.z, 5);
    }

    #[test]
    fn house_tile_position_matches_inner_tile() {
        let ht = HouseTile::new(50, 60, 3, 99);
        assert_eq!(ht.get_position(), ht.inner_tile().position);
    }

    // -----------------------------------------------------------------------
    // inner_tile / inner_tile_mut
    // -----------------------------------------------------------------------

    #[test]
    fn inner_tile_starts_with_no_items() {
        let ht = HouseTile::new(0, 0, 0, 1);
        assert_eq!(ht.inner_tile().get_item_count(), 0);
    }

    #[test]
    fn inner_tile_mut_allows_adding_items() {
        let mut ht = HouseTile::new(0, 0, 0, 1);
        ht.inner_tile_mut().add_item(make_item(5));
        assert_eq!(ht.inner_tile().get_item_count(), 1);
    }

    #[test]
    fn inner_tile_flags_accessible_via_inner_tile_mut() {
        let mut ht = HouseTile::new(0, 0, 0, 1);
        ht.inner_tile_mut()
            .set_flag(crate::tile::flags::PROTECTIONZONE);
        assert!(ht.inner_tile().has_flag(crate::tile::flags::PROTECTIONZONE));
    }

    // -----------------------------------------------------------------------
    // query_add_for_non_member — always denies
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_non_member_returns_not_invited() {
        let ht = HouseTile::new(0, 0, 0, 1);
        assert_eq!(ht.query_add_for_non_member(), Err(AddError::NotInvited));
    }

    #[test]
    fn query_add_item_non_member_returns_not_invited() {
        let ht = HouseTile::new(0, 0, 0, 1);
        assert_eq!(
            ht.query_add_item_for_non_member(),
            Err(AddError::NotInvited)
        );
    }

    // -----------------------------------------------------------------------
    // house_id for different values
    // -----------------------------------------------------------------------

    #[test]
    fn house_tile_house_id_zero() {
        let ht = HouseTile::new(0, 0, 0, 0);
        assert_eq!(ht.house_id(), 0);
    }

    #[test]
    fn house_tile_house_id_max() {
        let ht = HouseTile::new(0, 0, 0, u32::MAX);
        assert_eq!(ht.house_id(), u32::MAX);
    }

    // -----------------------------------------------------------------------
    // query_add_non_player_creature — mirrors RETURNVALUE_NOTPOSSIBLE branch
    // C++: if (!player) return RETURNVALUE_NOTPOSSIBLE
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_non_player_creature_returns_not_possible() {
        let ht = HouseTile::new(0, 0, 0, 1);
        assert_eq!(
            ht.query_add_non_player_creature(),
            Err(AddError::NotPossible)
        );
    }

    #[test]
    fn add_error_not_possible_differs_from_not_invited() {
        assert_ne!(AddError::NotPossible, AddError::NotInvited);
    }

    // -----------------------------------------------------------------------
    // query_remove_for_non_member — mirrors queryRemove with invited check
    // -----------------------------------------------------------------------

    #[test]
    fn query_remove_non_member_returns_not_invited() {
        let ht = HouseTile::new(0, 0, 0, 5);
        assert_eq!(ht.query_remove_for_non_member(), Err(AddError::NotInvited));
    }

    #[test]
    fn query_remove_non_member_always_denies_regardless_of_house_id() {
        for id in [0_u32, 1, 100, u32::MAX] {
            let ht = HouseTile::new(0, 0, 0, id);
            assert_eq!(ht.query_remove_for_non_member(), Err(AddError::NotInvited));
        }
    }

    // -----------------------------------------------------------------------
    // add_item — mirrors addThing / internalAddThing
    // C++: Tile::addThing(item) then updateHouse(item)
    // Rust: delegates to inner tile; item count must increase.
    // -----------------------------------------------------------------------

    #[test]
    fn add_item_increases_inner_tile_item_count() {
        let mut ht = HouseTile::new(10, 20, 5, 7);
        ht.add_item(make_item(1));
        assert_eq!(ht.inner_tile().get_item_count(), 1);
    }

    #[test]
    fn add_multiple_items_reflected_in_inner_tile() {
        let mut ht = HouseTile::new(0, 0, 0, 1);
        ht.add_item(make_item(10));
        ht.add_item(make_item(20));
        ht.add_item(make_item(30));
        assert_eq!(ht.inner_tile().get_item_count(), 3);
    }

    #[test]
    fn add_item_does_not_change_house_id() {
        let mut ht = HouseTile::new(0, 0, 0, 42);
        ht.add_item(make_item(5));
        assert_eq!(ht.house_id(), 42);
    }

    #[test]
    fn add_item_does_not_change_position() {
        let mut ht = HouseTile::new(100, 200, 7, 1);
        ht.add_item(make_item(1));
        let pos = ht.get_position();
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
        assert_eq!(pos.z, 7);
    }

    // -----------------------------------------------------------------------
    // queryAdd / queryDestination — non-invited creature is redirected
    // C++: queryDestination returns house entry tile when player not invited.
    // We verify the not-invited result for both query_add variants.
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_non_member_and_item_non_member_both_deny() {
        let ht = HouseTile::new(0, 0, 0, 1);
        assert!(ht.query_add_for_non_member().is_err());
        assert!(ht.query_add_item_for_non_member().is_err());
    }

    #[test]
    fn all_query_deny_methods_return_add_error() {
        let ht = HouseTile::new(0, 0, 0, 1);
        let e1 = ht.query_add_for_non_member().unwrap_err();
        let e2 = ht.query_add_item_for_non_member().unwrap_err();
        let e3 = ht.query_add_non_player_creature().unwrap_err();
        let e4 = ht.query_remove_for_non_member().unwrap_err();
        // e1, e2, e4 are NotInvited; e3 is NotPossible
        assert_eq!(e1, AddError::NotInvited);
        assert_eq!(e2, AddError::NotInvited);
        assert_eq!(e3, AddError::NotPossible);
        assert_eq!(e4, AddError::NotInvited);
    }

    // -----------------------------------------------------------------------
    // Clone — HouseTile is Clone; clone has same house_id and items
    // -----------------------------------------------------------------------

    #[test]
    fn clone_preserves_house_id_and_item_count() {
        let mut ht = HouseTile::new(5, 5, 5, 99);
        ht.add_item(make_item(1));
        ht.add_item(make_item(2));
        let clone = ht.clone();
        assert_eq!(clone.house_id(), 99);
        assert_eq!(clone.inner_tile().get_item_count(), 2);
    }

    #[test]
    fn clone_is_independent_of_original() {
        let ht = HouseTile::new(0, 0, 0, 1);
        let mut clone = ht.clone();
        // Mutate clone; original must be unaffected
        clone.add_item(make_item(9));
        assert_eq!(ht.inner_tile().get_item_count(), 0);
        assert_eq!(clone.inner_tile().get_item_count(), 1);
    }

    // ── queryDestination decision helper (Session 22) ───────────────────

    /// Invited player → always delegate to base Tile::queryDestination.
    #[test]
    fn query_destination_invited_delegates_to_tile() {
        // Boolean inputs after `invited` are irrelevant for this branch.
        assert_eq!(
            query_destination_for_non_member(true, false, false),
            QueryDestinationOutcome::DelegateToTile
        );
        assert_eq!(
            query_destination_for_non_member(true, true, true),
            QueryDestinationOutcome::DelegateToTile
        );
    }

    /// Non-invited + entry tile present → route to entry.
    #[test]
    fn query_destination_non_invited_routes_to_entry_when_available() {
        assert_eq!(
            query_destination_for_non_member(false, true, false),
            QueryDestinationOutcome::RouteToEntry
        );
        // Entry takes precedence over temple when both resolve.
        assert_eq!(
            query_destination_for_non_member(false, true, true),
            QueryDestinationOutcome::RouteToEntry
        );
    }

    /// Non-invited + entry missing + temple present → route to temple.
    #[test]
    fn query_destination_non_invited_falls_back_to_temple_when_entry_missing() {
        assert_eq!(
            query_destination_for_non_member(false, false, true),
            QueryDestinationOutcome::RouteToTemple
        );
    }

    /// Non-invited + both missing → null sentinel.
    #[test]
    fn query_destination_non_invited_routes_to_null_when_both_missing() {
        assert_eq!(
            query_destination_for_non_member(false, false, false),
            QueryDestinationOutcome::RouteToNull
        );
    }

    // ── updateHouse classification helper (Session 22) ──────────────────

    /// Door with non-zero id → AddDoor.
    #[test]
    fn classify_door_with_id_returns_add_door() {
        assert_eq!(
            classify_item_for_update_house(true, 7, false),
            UpdateHouseAction::AddDoor { door_id: 7 }
        );
    }

    /// Door with id == 0 → no-op (C++ explicit guard).
    #[test]
    fn classify_door_with_zero_id_returns_none() {
        assert_eq!(
            classify_item_for_update_house(true, 0, false),
            UpdateHouseAction::None,
            "Door with door_id=0 must NOT be registered"
        );
    }

    /// Bed → AddBed.
    #[test]
    fn classify_bed_returns_add_bed() {
        assert_eq!(
            classify_item_for_update_house(false, 0, true),
            UpdateHouseAction::AddBed
        );
    }

    /// Plain item (neither door nor bed) → no-op.
    #[test]
    fn classify_neither_door_nor_bed_returns_none() {
        assert_eq!(
            classify_item_for_update_house(false, 0, false),
            UpdateHouseAction::None
        );
    }

    /// Door precedence: when both Door and Bed flags are true, Door wins
    /// (mirrors C++ `if (door) … else if (bed) …` branch order).
    #[test]
    fn classify_door_and_bed_door_takes_precedence() {
        assert_eq!(
            classify_item_for_update_house(true, 3, true),
            UpdateHouseAction::AddDoor { door_id: 3 }
        );
        // Door with id=0 + bed → still None (door branch short-circuits
        // before bed is considered).
        assert_eq!(
            classify_item_for_update_house(true, 0, true),
            UpdateHouseAction::None
        );
    }
}
