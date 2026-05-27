// Migrated from forgottenserver/src/depotlocker.h + depotlocker.cpp
//
// DepotLocker — the top-level locker container a player sees when opening
// their depot.  It holds DepotChest / Inbox children.
//
// Key behaviours:
//   - `queryAdd` always returns `NotEnoughRoom` — you cannot drag items
//     directly into the locker.
//   - `canRemove()` returns false.
//   - Carries a `depot_id` that identifies which depot town/location this is.
//   - `readAttr(ATTR_DEPOT_ID, propStream)` reads a u16 LE into `depot_id`.

use crate::container::Container;

// ---------------------------------------------------------------------------
// Attribute tag — matches C++ AttrTypes_t::ATTR_DEPOT_ID = 10
// ---------------------------------------------------------------------------

/// Serialization tag for the depot identifier (mirrors C++
/// `AttrTypes_t::ATTR_DEPOT_ID` from `item.h`).
pub const ATTR_DEPOT_ID: u8 = 10;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepotLockerError {
    /// Items cannot be placed directly into a depot locker.
    NotEnoughRoom,
}

// ---------------------------------------------------------------------------
// DepotLocker
// ---------------------------------------------------------------------------

/// Top-level locker container.  Items cannot be added directly; they live
/// inside the nested DepotChest or Inbox sub-containers.
#[derive(Debug, Clone)]
pub struct DepotLocker {
    container: Container,
    depot_id: u16,
}

impl DepotLocker {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create a new DepotLocker with `depot_id = 0`.
    pub fn new(item_type_id: u16) -> Self {
        // C++: Container(type) — uses items[type].maxItems as capacity.
        // We default to 4 slots (inbox + depot chests), unlocked.
        let container = Container::with_flags(item_type_id, 4, true, false);
        DepotLocker {
            container,
            depot_id: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Depot ID
    // -----------------------------------------------------------------------

    /// The identifier for this locker's depot town/location.
    pub fn depot_id(&self) -> u16 {
        self.depot_id
    }

    /// Set the depot ID.
    pub fn set_depot_id(&mut self, id: u16) {
        self.depot_id = id;
    }

    // -----------------------------------------------------------------------
    // Policy
    // -----------------------------------------------------------------------

    /// Items cannot be added directly to a depot locker (mirrors C++
    /// `queryAdd` which always returns `RETURNVALUE_NOTENOUGHROOM`).
    pub fn query_add(&self) -> Result<(), DepotLockerError> {
        Err(DepotLockerError::NotEnoughRoom)
    }

    /// Depot lockers cannot be removed by a player.
    pub fn can_remove(&self) -> bool {
        false
    }

    // -----------------------------------------------------------------------
    // Delegation
    // -----------------------------------------------------------------------

    /// Immutable access to the inner container (holds depot chests / inbox).
    pub fn inner_container(&self) -> &Container {
        &self.container
    }

    /// Mutable access to the inner container.
    pub fn inner_container_mut(&mut self) -> &mut Container {
        &mut self.container
    }

    /// Remove the first child whose item-type ID matches `inbox_type_id`.
    ///
    /// Mirrors C++ `DepotLocker::removeInbox(Inbox*)` which does a linear scan
    /// of `itemlist` for the matching pointer and erases it.  In Rust we
    /// compare by type ID since items are owned by value.
    ///
    /// Returns `true` if a matching child was found and removed, `false`
    /// otherwise.
    pub fn remove_inbox(&mut self, inbox_type_id: u16) -> bool {
        let idx = self.container.iter().enumerate().find_map(|(i, item)| {
            if item.get_id() == inbox_type_id {
                Some(i)
            } else {
                None
            }
        });

        match idx {
            Some(i) => {
                self.container.remove_item(i);
                true
            }
            None => false,
        }
    }

    // -----------------------------------------------------------------------
    // Serialization — mirrors C++ DepotLocker::readAttr
    // -----------------------------------------------------------------------

    /// Parse a `[ATTR_DEPOT_ID, lo, hi]` byte sequence and update `depot_id`.
    ///
    /// Mirrors C++ `DepotLocker::readAttr(ATTR_DEPOT_ID, propStream)` which
    /// reads a `uint16_t` little-endian value off the prop stream into
    /// `depotId` and returns `ATTR_READ_CONTINUE`.  Any other tag is
    /// delegated to the base `Item::readAttr` (out of scope here — caller
    /// should route non-matching tags elsewhere).
    ///
    /// Returns `true` on success, `false` if `bytes` is too short or the
    /// leading tag is not `ATTR_DEPOT_ID`.
    pub fn read_depot_id_attr(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() < 3 {
            return false;
        }
        if bytes[0] != ATTR_DEPOT_ID {
            return false;
        }
        self.depot_id = u16::from_le_bytes([bytes[1], bytes[2]]);
        true
    }

    // -----------------------------------------------------------------------
    // Self-reference downcast — mirrors C++ getDepotLocker() override
    // -----------------------------------------------------------------------

    /// Returns a reference to this locker (mirrors C++
    /// `DepotLocker* getDepotLocker() override { return this; }`).
    pub fn get_depot_locker(&self) -> &Self {
        self
    }

    /// Mutable counterpart to [`get_depot_locker`].
    pub fn get_depot_locker_mut(&mut self) -> &mut Self {
        self
    }
}

// ---------------------------------------------------------------------------
// Cross-crate Thing + Cylinder impls (delegate to the inner Container)
// ---------------------------------------------------------------------------

impl forgottenserver_common::thing::Thing for DepotLocker {
    fn is_container(&self) -> bool {
        true
    }
    fn is_item(&self) -> bool {
        true
    }
    fn is_removed(&self) -> bool {
        false
    }
    fn get_first_index(&self) -> usize {
        self.container.get_first_index()
    }
    fn get_last_index(&self) -> usize {
        self.container.get_last_index()
    }
    fn get_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
        <Container as forgottenserver_common::thing::Thing>::get_item_type_count(
            &self.container,
            item_id,
            sub_type,
        )
    }
}

impl forgottenserver_common::cylinder::Cylinder for DepotLocker {
    fn cylinder_first_index(&self) -> usize {
        self.container.cylinder_first_index()
    }
    fn cylinder_last_index(&self) -> usize {
        self.container.cylinder_last_index()
    }
    fn cylinder_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
        self.container.cylinder_item_type_count(item_id, sub_type)
    }
    fn cylinder_query_add(
        &self,
        index: i32,
        thing: &dyn forgottenserver_common::thing::Thing,
        count: u32,
        flags: u32,
    ) -> forgottenserver_common::enums::ReturnValue {
        // C++ DepotLocker::queryAdd: refuses normal adds (locker is a wrapper
        // around the depot chest; players interact with the chest, not the
        // locker). Delegate to the inner container so server-side adds with
        // NOLIMIT can still set things up.
        const FLAG_NOLIMIT: u32 = 1;
        if (flags & FLAG_NOLIMIT) == 0 {
            return forgottenserver_common::enums::ReturnValue::NotPossible;
        }
        self.container
            .cylinder_query_add(index, thing, count, flags)
    }
    fn cylinder_query_max_count(
        &self,
        index: i32,
        thing: &dyn forgottenserver_common::thing::Thing,
        count: u32,
        flags: u32,
    ) -> (forgottenserver_common::enums::ReturnValue, u32) {
        self.container
            .cylinder_query_max_count(index, thing, count, flags)
    }
    fn cylinder_query_remove(
        &self,
        _thing: &dyn forgottenserver_common::thing::Thing,
        _count: u32,
        _flags: u32,
    ) -> forgottenserver_common::enums::ReturnValue {
        // C++ DepotLocker::canRemove() → false.
        forgottenserver_common::enums::ReturnValue::NotPossible
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;
    use crate::items_registry::ItemTypeData;
    use std::sync::Arc;

    const LOCKER_TYPE_ID: u16 = 50;
    const INBOX_TYPE_ID: u16 = 100;

    fn make_item(id: u16, weight: u32) -> Item {
        let td = ItemTypeData {
            id,
            client_id: id,
            weight,
            pickupable: true,
            moveable: true,
            ..ItemTypeData::default()
        };
        Item::new(Arc::new(td), 1)
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_creates_locker_with_depot_id_zero() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert_eq!(dl.depot_id(), 0);
    }

    #[test]
    fn test_inner_container_has_capacity_4() {
        // C++ DepotLocker is constructed with Container(type) whose maxItems
        // the Rust side defaults to 4 (inbox + up to 3 depot chests).
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert_eq!(dl.inner_container().capacity(), 4);
    }

    #[test]
    fn test_new_inner_container_is_empty() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert!(dl.inner_container().is_empty());
    }

    // -----------------------------------------------------------------------
    // depot_id
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_depot_id_updates_value() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.set_depot_id(7);
        assert_eq!(dl.depot_id(), 7);
    }

    #[test]
    fn test_set_depot_id_max_u16() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.set_depot_id(u16::MAX);
        assert_eq!(dl.depot_id(), u16::MAX);
    }

    #[test]
    fn test_set_depot_id_multiple_times_retains_last() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.set_depot_id(1);
        dl.set_depot_id(2);
        dl.set_depot_id(100);
        assert_eq!(dl.depot_id(), 100);
    }

    // -----------------------------------------------------------------------
    // query_add
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_add_always_returns_not_enough_room() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert_eq!(dl.query_add(), Err(DepotLockerError::NotEnoughRoom));
    }

    #[test]
    fn test_query_add_not_enough_room_error_variant() {
        // Verify the error compares equal to the specific variant
        // (mirrors C++ RETURNVALUE_NOTENOUGHROOM).  This test deliberately
        // unwraps the Err and tests the inner enum value directly so no
        // wildcard arm is generated.
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        let err = dl.query_add().expect_err("query_add must return Err");
        assert_eq!(err, DepotLockerError::NotEnoughRoom);
    }

    // -----------------------------------------------------------------------
    // can_remove
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_remove_always_false() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert!(!dl.can_remove());
    }

    // -----------------------------------------------------------------------
    // inner_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_inner_container_is_accessible() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        // Just check we can read from it without panicking
        let _ = dl.inner_container().capacity();
    }

    #[test]
    fn test_inner_container_mut_is_accessible() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        // Confirm mutable access compiles and works
        let cap = dl.inner_container_mut().capacity();
        assert!(cap > 0);
    }

    #[test]
    fn test_inner_container_mut_can_add_child() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        let item = make_item(INBOX_TYPE_ID, 0);
        dl.inner_container_mut().add_item(item).unwrap();
        assert_eq!(dl.inner_container().size(), 1);
    }

    // -----------------------------------------------------------------------
    // remove_inbox
    // -----------------------------------------------------------------------

    #[test]
    fn test_remove_inbox_returns_false_when_not_present() {
        // C++: if item not found in itemlist, just return (no-op).
        // Rust: we return false to indicate nothing was removed.
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert!(!dl.remove_inbox(INBOX_TYPE_ID));
    }

    #[test]
    fn test_remove_inbox_removes_matching_child_and_returns_true() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        let inbox_item = make_item(INBOX_TYPE_ID, 0);
        dl.inner_container_mut().add_item(inbox_item).unwrap();
        assert_eq!(dl.inner_container().size(), 1);

        let removed = dl.remove_inbox(INBOX_TYPE_ID);
        assert!(removed);
        assert_eq!(dl.inner_container().size(), 0);
    }

    #[test]
    fn test_remove_inbox_does_not_remove_non_matching_child() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        let other = make_item(200, 0); // type 200, not INBOX_TYPE_ID
        dl.inner_container_mut().add_item(other).unwrap();
        assert_eq!(dl.inner_container().size(), 1);

        let removed = dl.remove_inbox(INBOX_TYPE_ID);
        assert!(!removed);
        assert_eq!(dl.inner_container().size(), 1);
    }

    #[test]
    fn test_remove_inbox_removes_only_first_match() {
        // Two children with same type ID: only the first one found is removed.
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.inner_container_mut()
            .add_item_back(make_item(INBOX_TYPE_ID, 1))
            .unwrap();
        dl.inner_container_mut()
            .add_item_back(make_item(INBOX_TYPE_ID, 2))
            .unwrap();
        assert_eq!(dl.inner_container().size(), 2);

        let removed = dl.remove_inbox(INBOX_TYPE_ID);
        assert!(removed);
        assert_eq!(dl.inner_container().size(), 1);
    }

    #[test]
    fn test_remove_inbox_second_call_after_empty_returns_false() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.inner_container_mut()
            .add_item(make_item(INBOX_TYPE_ID, 0))
            .unwrap();

        assert!(dl.remove_inbox(INBOX_TYPE_ID));
        // Second call: no more inboxes to remove
        assert!(!dl.remove_inbox(INBOX_TYPE_ID));
    }

    // -----------------------------------------------------------------------
    // ATTR_DEPOT_ID constant
    // -----------------------------------------------------------------------

    #[test]
    fn test_attr_depot_id_value() {
        // Matches C++ AttrTypes_t::ATTR_DEPOT_ID = 10 (item.h line 61).
        assert_eq!(ATTR_DEPOT_ID, 10);
    }

    // -----------------------------------------------------------------------
    // read_depot_id_attr — mirrors C++ DepotLocker::readAttr(ATTR_DEPOT_ID, ...)
    // -----------------------------------------------------------------------

    #[test]
    fn test_read_depot_id_attr_parses_u16_le_and_updates_field() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        // [tag, 0x34, 0x12] -> depot_id = 0x1234
        let ok = dl.read_depot_id_attr(&[ATTR_DEPOT_ID, 0x34, 0x12]);
        assert!(ok);
        assert_eq!(dl.depot_id(), 0x1234);
    }

    #[test]
    fn test_read_depot_id_attr_low_byte_only() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        // Endianness check: [tag, 0xFF, 0x00] -> 0x00FF = 255
        assert!(dl.read_depot_id_attr(&[ATTR_DEPOT_ID, 0xFF, 0x00]));
        assert_eq!(dl.depot_id(), 0x00FF);
    }

    #[test]
    fn test_read_depot_id_attr_high_byte_only() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        // [tag, 0x00, 0xFF] -> 0xFF00 = 65280
        assert!(dl.read_depot_id_attr(&[ATTR_DEPOT_ID, 0x00, 0xFF]));
        assert_eq!(dl.depot_id(), 0xFF00);
    }

    #[test]
    fn test_read_depot_id_attr_max_value() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert!(dl.read_depot_id_attr(&[ATTR_DEPOT_ID, 0xFF, 0xFF]));
        assert_eq!(dl.depot_id(), u16::MAX);
    }

    #[test]
    fn test_read_depot_id_attr_too_short_returns_false() {
        // C++: PropStream::read<uint16_t> fails when fewer than 2 data bytes
        // remain → readAttr returns ATTR_READ_ERROR.  Rust counterpart: false.
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.set_depot_id(42);

        assert!(!dl.read_depot_id_attr(&[]));
        assert!(!dl.read_depot_id_attr(&[ATTR_DEPOT_ID]));
        assert!(!dl.read_depot_id_attr(&[ATTR_DEPOT_ID, 0x01]));
        // Field must remain unchanged on failure.
        assert_eq!(dl.depot_id(), 42);
    }

    #[test]
    fn test_read_depot_id_attr_wrong_tag_returns_false() {
        // Tag mismatch → caller should route to base Item::readAttr; we
        // signal "not handled" by returning false and leaving depot_id alone.
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.set_depot_id(99);
        let other_tag: u8 = ATTR_DEPOT_ID.wrapping_add(1);
        assert!(!dl.read_depot_id_attr(&[other_tag, 0x01, 0x02]));
        assert_eq!(dl.depot_id(), 99);
    }

    // -----------------------------------------------------------------------
    // get_depot_locker — mirrors C++ DepotLocker* getDepotLocker() override
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_depot_locker_returns_self_with_same_depot_id() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        dl.set_depot_id(77);
        let r = dl.get_depot_locker();
        // Observable behaviour: reading through the borrowed reference must
        // see the same field values as the original.
        assert_eq!(r.depot_id(), 77);
        assert_eq!(r.inner_container().capacity(), 4);
    }

    #[test]
    fn test_get_depot_locker_mut_allows_mutation_through_returned_ref() {
        let mut dl = DepotLocker::new(LOCKER_TYPE_ID);
        {
            let r = dl.get_depot_locker_mut();
            r.set_depot_id(123);
        }
        // Mutation through the self-reference is reflected on the original.
        assert_eq!(dl.depot_id(), 123);
    }

    // -----------------------------------------------------------------------
    // Cylinder trait impl — cross-crate dispatch round-trip
    // -----------------------------------------------------------------------

    use forgottenserver_common::cylinder::Cylinder as CommonCylinder;
    use forgottenserver_common::enums::ReturnValue as CommonRv;
    use forgottenserver_common::thing::{DefaultThing, Thing as CommonThing};

    const FLAG_NOLIMIT: u32 = 1;

    #[test]
    fn test_depotlocker_cylinder_query_add_refuses_player_add() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&dl, 0, &dummy, 1, 0),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_depotlocker_cylinder_query_add_accepts_server_setup() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        let dummy = DefaultThing;
        // NOLIMIT: server-side setup is permitted.
        assert_eq!(
            CommonCylinder::cylinder_query_add(&dl, 0, &dummy, 1, FLAG_NOLIMIT),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_depotlocker_cylinder_query_remove_refuses_always() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&dl, &dummy, 1, 0),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_depotlocker_implements_thing_is_container() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        assert!(CommonThing::is_container(&dl));
        assert!(CommonThing::is_item(&dl));
    }

    #[test]
    fn test_depotlocker_via_dyn_cylinder_trait_object() {
        let dl = DepotLocker::new(LOCKER_TYPE_ID);
        let cyl: &dyn CommonCylinder = &dl;
        let dummy = DefaultThing;
        assert_eq!(
            cyl.cylinder_query_add(0, &dummy, 1, 0),
            CommonRv::NotPossible
        );
        assert_eq!(cyl.cylinder_first_index(), 0);
    }
}
