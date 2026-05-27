// Migrated from forgottenserver/src/inbox.h + inbox.cpp
//
// Inbox — a player's mail inbox.
//
// Key behaviours from the C++ source:
//   - Capacity is 30 (`Container(type, 30, false, true)`).
//   - Items can only be added when `FLAG_NOLIMIT` is set (i.e. only via
//     server-side mail delivery, not by drag-and-drop).  In our Rust model we
//     expose this as `query_add_inbox(item_type_id)` which blocks same-type
//     nesting and as `add_item` which always succeeds (assuming the caller
//     holds the appropriate flag — we leave flag checking to higher-level
//     game logic).
//   - `canRemove()` returns false.
//   - The locker parent is skipped (same pattern as DepotChest).

use crate::container::Container;
use crate::item::Item;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboxError {
    /// An inbox cannot be placed inside another inbox.
    NestedInbox,
    /// The inbox container is full.
    ContainerFull,
}

// ---------------------------------------------------------------------------
// Inbox
// ---------------------------------------------------------------------------

/// A player's mail inbox.
///
/// Wraps a 30-slot [`Container`] with `unlocked = false` and `pagination = true`.
/// Items can only be placed here by the server (mail delivery), not by players
/// via drag-and-drop.
#[derive(Debug, Clone)]
pub struct Inbox {
    container: Container,
}

impl Inbox {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create an inbox with its standard 30-slot capacity.
    pub fn new(item_type_id: u16) -> Self {
        // C++: Container(type, 30, false, true)
        let container = Container::with_flags(item_type_id, 30, false, true);
        Inbox { container }
    }

    // -----------------------------------------------------------------------
    // Policy
    // -----------------------------------------------------------------------

    /// Inboxes cannot be removed by a player.
    pub fn can_remove(&self) -> bool {
        false
    }

    /// Check whether the given item (identified by type ID) can be added to
    /// this inbox.
    ///
    /// Returns `Err(InboxError::NestedInbox)` if the item is itself an inbox
    /// (same type ID), mirroring the C++ guard that prevents placing an inbox
    /// inside another inbox.
    pub fn query_add_inbox(&self, item_type_id: u16) -> Result<(), InboxError> {
        if item_type_id == self.container.item_type_id() {
            return Err(InboxError::NestedInbox);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Item management (server-side — no player flag required here)
    // -----------------------------------------------------------------------

    /// Add an item to the inbox (server-side delivery).
    ///
    /// Returns `Err(InboxError::ContainerFull)` if the inbox is full.
    /// Does **not** enforce the nested-inbox guard — call `query_add_inbox`
    /// first when needed.
    pub fn add_item(&mut self, item: Item) -> Result<(), InboxError> {
        self.container
            .add_item(item)
            .map_err(|_| InboxError::ContainerFull)
    }

    // -----------------------------------------------------------------------
    // Delegation
    // -----------------------------------------------------------------------

    /// Immutable access to the inner container.
    pub fn inner_container(&self) -> &Container {
        &self.container
    }

    /// Mutable access to the inner container.
    pub fn inner_container_mut(&mut self) -> &mut Container {
        &mut self.container
    }
}

// ---------------------------------------------------------------------------
// Cross-crate Thing + Cylinder impls (delegate to the inner Container)
// ---------------------------------------------------------------------------

impl forgottenserver_common::thing::Thing for Inbox {
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

impl forgottenserver_common::cylinder::Cylinder for Inbox {
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
        // C++ Inbox::queryAdd: only permits adds when FLAG_NOLIMIT is set
        // (server-side mail delivery). Player-initiated drag-and-drop is
        // rejected.
        const FLAG_NOLIMIT: u32 = 1;
        if (flags & FLAG_NOLIMIT) == 0 {
            return forgottenserver_common::enums::ReturnValue::ContainerNotEnoughRoom;
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
        // C++ Inbox::canRemove() → false; players cannot remove items from
        // their inbox via cylinder dispatch.
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

    const INBOX_TYPE_ID: u16 = 1234;

    fn make_item(id: u16, weight: u32) -> Item {
        let td = ItemTypeData {
            id,
            client_id: id,
            weight,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        Item::new(Arc::new(td), 1)
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_creates_inbox_with_30_capacity() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert_eq!(inbox.inner_container().capacity(), 30);
        assert!(inbox.inner_container().is_empty());
    }

    #[test]
    fn test_new_inbox_is_not_unlocked() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert!(!inbox.inner_container().is_unlocked());
    }

    #[test]
    fn test_new_inbox_has_pagination() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert!(inbox.inner_container().has_pagination());
    }

    // -----------------------------------------------------------------------
    // can_remove
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_remove_always_false() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert!(!inbox.can_remove());
    }

    // -----------------------------------------------------------------------
    // query_add_inbox
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_add_inbox_blocks_same_type() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        // Trying to add another inbox (same type_id) must fail
        let result = inbox.query_add_inbox(INBOX_TYPE_ID);
        assert_eq!(result, Err(InboxError::NestedInbox));
    }

    #[test]
    fn test_query_add_inbox_allows_different_type() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        // A regular item type is fine
        assert!(inbox.query_add_inbox(999).is_ok());
    }

    // -----------------------------------------------------------------------
    // add_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_regular_succeeds() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        let item = make_item(42, 100);
        assert!(inbox.add_item(item).is_ok());
        assert_eq!(inbox.inner_container().size(), 1);
    }

    #[test]
    fn test_add_item_fills_up_to_capacity() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        for i in 0..30 {
            inbox.add_item(make_item(i as u16 + 1, 10)).unwrap();
        }
        assert!(inbox.inner_container().is_full());
    }

    #[test]
    fn test_add_item_fails_when_full() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        for i in 0..30 {
            inbox.add_item(make_item(i as u16 + 1, 10)).unwrap();
        }
        let result = inbox.add_item(make_item(99, 10));
        assert_eq!(result, Err(InboxError::ContainerFull));
    }

    // -----------------------------------------------------------------------
    // inner_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_inner_container_mut_accessible() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        let item = make_item(7, 50);
        inbox.inner_container_mut().add_item(item).unwrap();
        assert_eq!(inbox.inner_container().size(), 1);
    }

    // -----------------------------------------------------------------------
    // Capacity boundary (C++: Container(type, 30, false, true))
    // -----------------------------------------------------------------------

    #[test]
    fn test_inbox_capacity_is_exactly_30() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert_eq!(inbox.inner_container().capacity(), 30);
    }

    #[test]
    fn test_add_29th_item_succeeds_and_30th_makes_it_full() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        for i in 0..29 {
            inbox.add_item(make_item(i as u16 + 1, 10)).unwrap();
        }
        assert!(!inbox.inner_container().is_full());
        inbox.add_item(make_item(30, 10)).unwrap();
        assert!(inbox.inner_container().is_full());
    }

    #[test]
    fn test_add_31st_item_fails_with_container_full() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        for i in 0..30 {
            inbox.add_item(make_item(i as u16 + 1, 10)).unwrap();
        }
        // 31st item must be rejected
        let result = inbox.add_item(make_item(31, 10));
        assert_eq!(result, Err(InboxError::ContainerFull));
        // Size stays at 30
        assert_eq!(inbox.inner_container().size(), 30);
    }

    // -----------------------------------------------------------------------
    // query_add_inbox flag semantics
    // (C++: queryAdd always returns CONTAINERNOTENOUGHROOM unless FLAG_NOLIMIT
    //  is set; Rust defers the flag check to callers — query_add_inbox only
    //  guards against nested-inbox placement)
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_add_inbox_allows_any_non_inbox_type_regardless_of_pickupable() {
        // Rust model: query_add_inbox does NOT enforce pickupable — that is a
        // concern for higher-level game logic (FLAG_NOLIMIT path in C++ already
        // bypasses pickupable check for server-side mail delivery).
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert!(inbox.query_add_inbox(999).is_ok());
    }

    #[test]
    fn test_query_add_inbox_blocks_same_type_as_container() {
        // Mirrors C++ guard: `if (item == this) return THISISIMPOSSIBLE`
        // and the structural rule that an inbox cannot hold another inbox.
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert_eq!(
            inbox.query_add_inbox(INBOX_TYPE_ID),
            Err(InboxError::NestedInbox)
        );
    }

    #[test]
    fn test_query_add_inbox_ok_does_not_prevent_add_item_failure_when_full() {
        // Verifies that query_add_inbox returning Ok does not bypass capacity:
        // both guards must be checked independently by the caller.
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        for i in 0..30 {
            inbox.add_item(make_item(i as u16 + 1, 10)).unwrap();
        }
        // query_add_inbox says OK for a non-inbox type
        assert!(inbox.query_add_inbox(999).is_ok());
        // But add_item still fails because the container is full
        let result = inbox.add_item(make_item(999, 10));
        assert_eq!(result, Err(InboxError::ContainerFull));
    }

    // -----------------------------------------------------------------------
    // Server-side add_item bypasses player restrictions
    // (C++: mail delivery uses FLAG_NOLIMIT which skips the regular queryAdd
    //  rejection path; in Rust add_item is the server-delivery entry point)
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_non_pickupable_succeeds_via_server_path() {
        // C++ mail delivery goes through the FLAG_NOLIMIT path, which skips
        // the pickupable check.  The Rust add_item method is the server-side
        // equivalent and does not enforce pickupable either.
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        let td = ItemTypeData {
            id: 42,
            client_id: 42,
            pickupable: false, // explicitly NOT pickupable
            moveable: false,
            ..Default::default()
        };
        let non_pickupable = Item::new(Arc::new(td), 1);
        // Should succeed — server can deliver anything
        assert!(inbox.add_item(non_pickupable).is_ok());
        assert_eq!(inbox.inner_container().size(), 1);
    }

    #[test]
    fn test_add_item_size_increments_correctly() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        for expected_size in 1..=5 {
            inbox.add_item(make_item(expected_size as u16, 10)).unwrap();
            assert_eq!(inbox.inner_container().size(), expected_size);
        }
    }

    // -----------------------------------------------------------------------
    // Inbox is not removable — confirmed idempotency
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_remove_is_false_regardless_of_contents() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        inbox.add_item(make_item(1, 10)).unwrap();
        assert!(!inbox.can_remove());
    }

    // -----------------------------------------------------------------------
    // Cylinder trait impl — cross-crate dispatch round-trip
    // -----------------------------------------------------------------------

    use forgottenserver_common::cylinder::Cylinder as CommonCylinder;
    use forgottenserver_common::enums::ReturnValue as CommonRv;
    use forgottenserver_common::thing::{DefaultThing, Thing as CommonThing};

    const FLAG_NOLIMIT: u32 = 1;

    #[test]
    fn test_inbox_cylinder_last_index_zero_when_empty() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert_eq!(CommonCylinder::cylinder_last_index(&inbox), 0);
    }

    #[test]
    fn test_inbox_cylinder_last_index_reflects_size() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        inbox.add_item(make_item(1, 10)).unwrap();
        inbox.add_item(make_item(2, 10)).unwrap();
        assert_eq!(CommonCylinder::cylinder_last_index(&inbox), 2);
    }

    #[test]
    fn test_inbox_cylinder_query_add_refuses_player_add() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        let dummy = DefaultThing;
        // No NOLIMIT flag → refused (player-initiated drag-and-drop).
        assert_eq!(
            CommonCylinder::cylinder_query_add(&inbox, 0, &dummy, 1, 0),
            CommonRv::ContainerNotEnoughRoom
        );
    }

    #[test]
    fn test_inbox_cylinder_query_add_accepts_server_delivery() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        let dummy = DefaultThing;
        // NOLIMIT flag set → server-side mail delivery permitted.
        assert_eq!(
            CommonCylinder::cylinder_query_add(&inbox, 0, &dummy, 1, FLAG_NOLIMIT),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_inbox_cylinder_query_remove_refuses_always() {
        let mut inbox = Inbox::new(INBOX_TYPE_ID);
        inbox.add_item(make_item(1, 10)).unwrap();
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&inbox, &dummy, 1, 0),
            CommonRv::NotPossible
        );
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&inbox, &dummy, 1, FLAG_NOLIMIT),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_inbox_implements_thing_is_container() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        assert!(CommonThing::is_container(&inbox));
        assert!(CommonThing::is_item(&inbox));
        assert!(!CommonThing::is_removed(&inbox));
    }

    #[test]
    fn test_inbox_via_dyn_cylinder_trait_object() {
        let inbox = Inbox::new(INBOX_TYPE_ID);
        let cyl: &dyn CommonCylinder = &inbox;
        assert_eq!(cyl.cylinder_first_index(), 0);
        let dummy = DefaultThing;
        assert_eq!(
            cyl.cylinder_query_add(0, &dummy, 1, 0),
            CommonRv::ContainerNotEnoughRoom
        );
    }
}
