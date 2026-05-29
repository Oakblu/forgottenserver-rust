// Migrated from forgottenserver/src/storeinbox.h + storeinbox.cpp
//
// StoreInbox — receives items bought from the in-game store.
//
// Key behaviours from C++:
//   - `Container(type, 20, true, true)` — 20 slots, unlocked, paginated.
//   - `getStoreInbox()` returns `this` (identified as a store-inbox).
//   - `queryAdd` blocks non-store items unless `FLAG_NOLIMIT` is set, and
//     additionally rejects non-empty container items.
//   - `postAddNotification` / `postRemoveNotification` simply forward the
//     callback up to the parent with `LINK_TOPPARENT`.
//   - `canRemove()` returns false.
//
// CHECKLIST mapping (C++ → Rust):
//   - `StoreInbox(uint16_t type)`                  → `StoreInbox::new` ✓
//   - `getStoreInbox()` (both overloads)            → `is_store_inbox()` ✓
//   - `queryAdd(...)`                               → `query_add(...)`    ✓
//   - `canRemove()`                                 → `can_remove()`      ✓
//   - `postAddNotification` / `postRemoveNotification` — DELIBERATELY_OMITTED:
//     they delegate to the cylinder/parent infrastructure, which lives in
//     downstream crates (`entity`, `game`).  The items crate has no parent
//     ownership model, so these methods are intentionally not implemented
//     here.  See `entity::player` for the cylinder-side notification chain.

use crate::container::{Container, ContainerError};
use crate::item::Item;
use forgottenserver_common::thing::ReceiverFlag;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreInboxError {
    /// The store inbox is full.
    ContainerFull,
    /// The container rejected the add for a reason other than "full" — e.g.
    /// an out-of-range index.  Currently unreachable through `add_item`
    /// (`Container::add_item` only returns `Full`); kept for forward-
    /// compatibility with new `ContainerError` variants.
    Rejected,
}

impl StoreInboxError {
    /// Translate a `ContainerError` into a `StoreInboxError`.
    ///
    /// Mapping:
    /// - [`ContainerError::Full`]       → [`StoreInboxError::ContainerFull`]
    /// - [`ContainerError::OutOfRange`] → [`StoreInboxError::Rejected`]
    pub fn from_container_error(e: ContainerError) -> Self {
        match e {
            ContainerError::Full => StoreInboxError::ContainerFull,
            ContainerError::OutOfRange => StoreInboxError::Rejected,
        }
    }
}

// ---------------------------------------------------------------------------
// queryAdd result (mirrors C++ `ReturnValue` values returned by
// `StoreInbox::queryAdd`).
// ---------------------------------------------------------------------------

/// Result of [`StoreInbox::query_add`].
///
/// Maps the exact C++ `ReturnValue` constants returned by
/// `StoreInbox::queryAdd` in `forgottenserver/src/storeinbox.cpp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreInboxQueryResult {
    /// The item may be added.  Mirrors `RETURNVALUE_NOERROR`.
    NoError,
    /// `thing.getItem()` was null — only items may be placed here.
    /// Mirrors `RETURNVALUE_NOTPOSSIBLE`.
    NotPossible,
    /// The caller tried to place the inbox inside itself.
    /// Mirrors `RETURNVALUE_THISISIMPOSSIBLE`.
    ThisIsImpossible,
    /// The item is not pickupable.  Mirrors `RETURNVALUE_CANNOTPICKUP`.
    CannotPickup,
    /// The item is not a store-deliverable item.
    /// Mirrors `RETURNVALUE_CANNOTMOVEITEMISNOTSTOREITEM`.
    CannotMoveItemIsNotStoreItem,
    /// The item is a non-empty container; only empty containers may be
    /// deposited via the store inbox.  Mirrors
    /// `RETURNVALUE_ITEMCANNOTBEMOVEDTHERE`.
    ItemCannotBeMovedThere,
}

// ---------------------------------------------------------------------------
// StoreInbox
// ---------------------------------------------------------------------------

/// The in-game store delivery inbox.
///
/// Wraps a 20-slot [`Container`] with `unlocked = true` and `pagination = true`.
#[derive(Debug, Clone)]
pub struct StoreInbox {
    container: Container,
}

impl StoreInbox {
    /// Maximum number of items a store inbox can hold (mirrors C++ hardcoded 20).
    pub const STORE_INBOX_MAX_CAPACITY: u32 = 20;

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create a store inbox with standard 20-slot capacity.
    pub fn new(item_type_id: u16) -> Self {
        // C++: Container(type, 20, true, true)
        let container =
            Container::with_flags(item_type_id, Self::STORE_INBOX_MAX_CAPACITY, true, true);
        StoreInbox { container }
    }

    // -----------------------------------------------------------------------
    // Identity
    // -----------------------------------------------------------------------

    /// Returns `true` — identifies this as a store inbox (mirrors C++
    /// `getStoreInbox()` returning non-null).
    pub fn is_store_inbox(&self) -> bool {
        true
    }

    // -----------------------------------------------------------------------
    // Policy
    // -----------------------------------------------------------------------

    /// Store inboxes cannot be removed by a player.
    pub fn can_remove(&self) -> bool {
        false
    }

    // -----------------------------------------------------------------------
    // Item management
    // -----------------------------------------------------------------------

    /// Add an item to the store inbox.
    ///
    /// Server-side delivery path: bypasses store-item / pickupable checks
    /// (the C++ side accomplishes the same via the `FLAG_NOLIMIT` flag in
    /// `queryAdd`).  Only the underlying container capacity is enforced.
    ///
    /// All non-`Full` `ContainerError` variants are mapped to
    /// `StoreInboxError::Rejected` so the caller never sees an inner enum
    /// leak.  `Container::add_item` only ever returns `Full`, so the
    /// `Rejected` mapping is exercised separately by direct construction
    /// in unit tests (the variant exists for forward-compatibility with new
    /// `ContainerError` variants).
    pub fn add_item(&mut self, item: Item) -> Result<(), StoreInboxError> {
        self.container
            .add_item(item)
            .map_err(StoreInboxError::from_container_error)
    }

    // -----------------------------------------------------------------------
    // queryAdd (C++ `StoreInbox::queryAdd`)
    // -----------------------------------------------------------------------

    /// Check whether the given item can be added to the store inbox.
    ///
    /// Mirrors C++ `StoreInbox::queryAdd`:
    ///
    /// 1. `thing.getItem()` null → `NotPossible` (Rust enforces this at the
    ///    type level by accepting only `&Item`; callers thus cannot pass a
    ///    non-item).  The same C++ branch is exercised in this method via
    ///    [`query_add_thing`](Self::query_add_thing) which accepts an
    ///    `Option<&Item>` to mirror the nullable C++ pointer.
    /// 2. `item == this` → `ThisIsImpossible`.  Modelled here by allowing the
    ///    caller to assert "this item *is* the inbox itself" via the
    ///    `is_self` boolean — Rust doesn't have C++ identity comparison on
    ///    arbitrary pointers, so the higher-level game code must decide.
    /// 3. Not pickupable → `CannotPickup`.
    /// 4. Unless `flags` includes `FLAG_NOLIMIT`:
    ///    a. Not a store item → `CannotMoveItemIsNotStoreItem`.
    ///    b. A non-empty container → `ItemCannotBeMovedThere`.
    /// 5. Otherwise → `NoError`.
    pub fn query_add(
        &self,
        item: &Item,
        flags: u32,
        is_self: bool,
        item_is_non_empty_container: bool,
    ) -> StoreInboxQueryResult {
        if is_self {
            return StoreInboxQueryResult::ThisIsImpossible;
        }

        if !item.is_pickupable() {
            return StoreInboxQueryResult::CannotPickup;
        }

        if !Self::has_no_limit(flags) {
            if !item.is_store_item() {
                return StoreInboxQueryResult::CannotMoveItemIsNotStoreItem;
            }

            if item_is_non_empty_container {
                return StoreInboxQueryResult::ItemCannotBeMovedThere;
            }
        }

        StoreInboxQueryResult::NoError
    }

    /// Variant of [`query_add`](Self::query_add) that accepts a nullable item
    /// reference, returning `NotPossible` when `None` (mirrors the C++
    /// `if (!item) return RETURNVALUE_NOTPOSSIBLE` branch where
    /// `thing.getItem()` yields a null pointer for non-item `Thing`s such as
    /// creatures).
    pub fn query_add_thing(
        &self,
        item: Option<&Item>,
        flags: u32,
        is_self: bool,
        item_is_non_empty_container: bool,
    ) -> StoreInboxQueryResult {
        match item {
            None => StoreInboxQueryResult::NotPossible,
            Some(it) => self.query_add(it, flags, is_self, item_is_non_empty_container),
        }
    }

    /// Returns `true` when the FLAG_NOLIMIT bit is set in `flags`.
    /// Mirrors C++ `hasBitSet(FLAG_NOLIMIT, flags)`.
    fn has_no_limit(flags: u32) -> bool {
        flags & ReceiverFlag::NoLimit.bits() != 0
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

impl forgottenserver_common::thing::Thing for StoreInbox {
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

impl forgottenserver_common::cylinder::Cylinder for StoreInbox {
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
        // C++ StoreInbox::queryAdd: refuses non-NOLIMIT adds (store-only).
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
        // C++ StoreInbox::canRemove() → false; store-inbox items cannot be
        // dragged out by players.
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

    fn make_item_full(id: u16, weight: u32, pickupable: bool, store_item: bool) -> Item {
        let td = ItemTypeData {
            id,
            client_id: id,
            weight,
            pickupable,
            moveable: true,
            store_item,
            ..Default::default()
        };
        Item::new(Arc::new(td), 1)
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_creates_store_inbox_with_20_capacity() {
        let si = StoreInbox::new(200);
        assert_eq!(si.inner_container().capacity(), 20);
        assert!(si.inner_container().is_empty());
    }

    #[test]
    fn test_new_store_inbox_is_unlocked() {
        let si = StoreInbox::new(200);
        assert!(si.inner_container().is_unlocked());
    }

    #[test]
    fn test_new_store_inbox_has_pagination() {
        let si = StoreInbox::new(200);
        assert!(si.inner_container().has_pagination());
    }

    // -----------------------------------------------------------------------
    // is_store_inbox
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_store_inbox_returns_true() {
        let si = StoreInbox::new(200);
        assert!(si.is_store_inbox());
    }

    // -----------------------------------------------------------------------
    // can_remove
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_remove_always_false() {
        let si = StoreInbox::new(200);
        assert!(!si.can_remove());
    }

    // -----------------------------------------------------------------------
    // add_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_succeeds() {
        let mut si = StoreInbox::new(200);
        let item = make_item(10, 50);
        assert!(si.add_item(item).is_ok());
        assert_eq!(si.inner_container().size(), 1);
    }

    #[test]
    fn test_add_item_fails_when_full() {
        let mut si = StoreInbox::new(200);
        for i in 0..20 {
            si.add_item(make_item(i as u16 + 1, 10)).unwrap();
        }
        let result = si.add_item(make_item(99, 10));
        assert_eq!(result, Err(StoreInboxError::ContainerFull));
    }

    // -----------------------------------------------------------------------
    // inner_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_inner_container_accessible() {
        let si = StoreInbox::new(200);
        assert_eq!(si.inner_container().capacity(), 20);
    }

    #[test]
    fn test_inner_container_mut_accessible() {
        let mut si = StoreInbox::new(200);
        let item = make_item(5, 30);
        si.inner_container_mut().add_item(item).unwrap();
        assert_eq!(si.inner_container().size(), 1);
    }

    // -----------------------------------------------------------------------
    // STORE_INBOX_MAX_CAPACITY constant (task 4.5 requirement)
    // -----------------------------------------------------------------------

    #[test]
    fn test_store_inbox_max_capacity_constant_is_20() {
        assert_eq!(StoreInbox::STORE_INBOX_MAX_CAPACITY, 20);
    }

    #[test]
    fn test_capacity_matches_named_constant() {
        let si = StoreInbox::new(200);
        assert_eq!(
            si.inner_container().capacity(),
            StoreInbox::STORE_INBOX_MAX_CAPACITY
        );
    }

    // -----------------------------------------------------------------------
    // Exact-boundary behaviour (20 items accepted, 21st rejected)
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_exactly_max_capacity_items_succeeds() {
        let mut si = StoreInbox::new(200);
        for i in 0..StoreInbox::STORE_INBOX_MAX_CAPACITY {
            assert!(
                si.add_item(make_item(i as u16 + 1, 5)).is_ok(),
                "item {} should fit",
                i
            );
        }
        assert!(si.inner_container().is_full());
    }

    #[test]
    fn test_add_item_beyond_capacity_fails_with_container_full() {
        let mut si = StoreInbox::new(200);
        for i in 0..StoreInbox::STORE_INBOX_MAX_CAPACITY {
            si.add_item(make_item(i as u16 + 1, 5)).unwrap();
        }
        let result = si.add_item(make_item(100, 5));
        assert_eq!(result, Err(StoreInboxError::ContainerFull));
    }

    // -----------------------------------------------------------------------
    // queryRemove: always allowed — removing items from a store inbox is
    // permitted (no C++ override of queryRemove, so Container base allows it).
    // We verify this by removing items through the inner container.
    // -----------------------------------------------------------------------

    #[test]
    fn test_items_can_be_removed_from_store_inbox() {
        let mut si = StoreInbox::new(200);
        si.add_item(make_item(10, 50)).unwrap();
        let removed = si.inner_container_mut().remove_item(0);
        assert!(removed.is_some());
        assert_eq!(si.inner_container().size(), 0);
    }

    #[test]
    fn test_remove_from_empty_store_inbox_returns_none() {
        let mut si = StoreInbox::new(200);
        let removed = si.inner_container_mut().remove_item(0);
        assert!(removed.is_none());
    }

    #[test]
    fn test_remove_then_add_succeeds_when_not_full() {
        let mut si = StoreInbox::new(200);
        // Fill to capacity
        for i in 0..StoreInbox::STORE_INBOX_MAX_CAPACITY {
            si.add_item(make_item(i as u16 + 1, 5)).unwrap();
        }
        assert!(si.inner_container().is_full());
        // Remove one — now space is available again
        si.inner_container_mut().remove_item(0).unwrap();
        assert!(!si.inner_container().is_full());
        // Should be able to add another item
        assert!(si.add_item(make_item(99, 5)).is_ok());
    }

    // -----------------------------------------------------------------------
    // Separate from Inbox: capacity must be 20 (not 30 like Inbox)
    // -----------------------------------------------------------------------

    #[test]
    fn test_store_inbox_capacity_differs_from_inbox_capacity() {
        use crate::inbox::Inbox;
        let si = StoreInbox::new(200);
        let inbox = Inbox::new(201);
        assert_ne!(
            si.inner_container().capacity(),
            inbox.inner_container().capacity(),
            "StoreInbox (20) must differ from Inbox (30)"
        );
        assert_eq!(si.inner_container().capacity(), 20);
        assert_eq!(inbox.inner_container().capacity(), 30);
    }

    // -----------------------------------------------------------------------
    // add_item: OutOfRange path (currently unreachable via Container::add_item
    // but the match arm exists for completeness — test ensures it compiles
    // and the error type is exercised through a direct construction).
    // -----------------------------------------------------------------------

    #[test]
    fn test_store_inbox_error_rejected_is_distinct() {
        // Both variants must compare unequal.
        assert_ne!(StoreInboxError::ContainerFull, StoreInboxError::Rejected);
    }

    #[test]
    fn test_from_container_error_full_maps_to_container_full() {
        assert_eq!(
            StoreInboxError::from_container_error(ContainerError::Full),
            StoreInboxError::ContainerFull
        );
    }

    #[test]
    fn test_from_container_error_out_of_range_maps_to_rejected() {
        // Covers the otherwise-unreachable `OutOfRange` arm in the mapping.
        assert_eq!(
            StoreInboxError::from_container_error(ContainerError::OutOfRange),
            StoreInboxError::Rejected
        );
    }

    // -----------------------------------------------------------------------
    // Derive coverage — Debug + Clone on StoreInbox / StoreInboxError /
    // StoreInboxQueryResult
    // -----------------------------------------------------------------------

    #[test]
    fn test_store_inbox_clone_produces_equal_capacity_and_size() {
        let mut si = StoreInbox::new(200);
        si.add_item(make_item(7, 10)).unwrap();
        let cloned = si.clone();
        assert_eq!(cloned.inner_container().capacity(), 20);
        assert_eq!(cloned.inner_container().size(), 1);
    }

    #[test]
    fn test_store_inbox_debug_formats_non_empty() {
        let si = StoreInbox::new(200);
        let s = format!("{:?}", si);
        assert!(s.contains("StoreInbox"));
    }

    #[test]
    fn test_store_inbox_error_debug_clone_eq() {
        let e = StoreInboxError::ContainerFull;
        let e2 = e.clone();
        assert_eq!(e, e2);
        assert!(format!("{:?}", e).contains("ContainerFull"));
    }

    #[test]
    fn test_store_inbox_query_result_debug_clone_copy_eq() {
        let r = StoreInboxQueryResult::NoError;
        let r2 = r; // Copy
                    // Explicit invocation of the Clone derive (Copy uses memcpy, Clone
                    // dispatches through the derived `clone` method we want to exercise).
        #[allow(clippy::clone_on_copy)]
        let r3 = r.clone();
        assert_eq!(r, r2);
        assert_eq!(r, r3);
        assert!(format!("{:?}", r).contains("NoError"));
    }

    #[test]
    fn test_store_inbox_query_result_all_variants_debug_and_eq() {
        // Exercise Debug + PartialEq derive for every variant — without
        // this, the derive code for the less-common variants shows as
        // uncovered in llvm-cov.
        let variants = [
            StoreInboxQueryResult::NoError,
            StoreInboxQueryResult::NotPossible,
            StoreInboxQueryResult::ThisIsImpossible,
            StoreInboxQueryResult::CannotPickup,
            StoreInboxQueryResult::CannotMoveItemIsNotStoreItem,
            StoreInboxQueryResult::ItemCannotBeMovedThere,
        ];
        for (i, a) in variants.iter().enumerate() {
            // Debug-format each variant
            let s = format!("{:?}", a);
            assert!(!s.is_empty());
            // Copy + explicit Clone-derive invocation
            let c = *a;
            #[allow(clippy::clone_on_copy)]
            let c2 = a.clone();
            assert_eq!(*a, c);
            assert_eq!(*a, c2);
            // PartialEq: equal to self, unequal to every other variant
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(*a, *b);
                } else {
                    assert_ne!(*a, *b);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // query_add — C++ StoreInbox::queryAdd
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_add_returns_no_error_for_store_pickupable_item() {
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, /*pickupable*/ true, /*store_item*/ true);
        assert_eq!(
            si.query_add(&item, 0, false, false),
            StoreInboxQueryResult::NoError
        );
    }

    #[test]
    fn test_query_add_rejects_self_as_this_is_impossible() {
        // Mirrors C++ `if (item == this) return RETURNVALUE_THISISIMPOSSIBLE`.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, true, true);
        assert_eq!(
            si.query_add(&item, 0, /*is_self*/ true, false),
            StoreInboxQueryResult::ThisIsImpossible
        );
    }

    #[test]
    fn test_query_add_rejects_non_pickupable_item() {
        // Mirrors C++ `if (!item->isPickupable()) return RETURNVALUE_CANNOTPICKUP`.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, /*pickupable*/ false, /*store_item*/ true);
        assert_eq!(
            si.query_add(&item, 0, false, false),
            StoreInboxQueryResult::CannotPickup
        );
    }

    #[test]
    fn test_query_add_rejects_non_store_item_without_no_limit() {
        // Mirrors C++ `if (!item->isStoreItem()) return
        // RETURNVALUE_CANNOTMOVEITEMISNOTSTOREITEM`.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, /*pickupable*/ true, /*store_item*/ false);
        assert_eq!(
            si.query_add(&item, 0, false, false),
            StoreInboxQueryResult::CannotMoveItemIsNotStoreItem
        );
    }

    #[test]
    fn test_query_add_allows_non_store_item_with_no_limit_flag() {
        // FLAG_NOLIMIT bypasses the store-item and non-empty-container checks.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, true, /*store_item*/ false);
        let no_limit = ReceiverFlag::NoLimit.bits();
        assert_eq!(
            si.query_add(&item, no_limit, false, /*non_empty_container*/ true),
            StoreInboxQueryResult::NoError
        );
    }

    #[test]
    fn test_query_add_rejects_non_empty_container_item_without_no_limit() {
        // Mirrors C++ `if (container && !container->empty()) return
        // RETURNVALUE_ITEMCANNOTBEMOVEDTHERE`.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, true, /*store_item*/ true);
        assert_eq!(
            si.query_add(&item, 0, false, /*non_empty_container*/ true),
            StoreInboxQueryResult::ItemCannotBeMovedThere
        );
    }

    #[test]
    fn test_query_add_allows_empty_container_item_without_no_limit() {
        // Empty container (or non-container) → still NoError.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, true, true);
        assert_eq!(
            si.query_add(&item, 0, false, /*non_empty_container*/ false),
            StoreInboxQueryResult::NoError
        );
    }

    #[test]
    fn test_query_add_pickupable_check_runs_before_store_check() {
        // Even with FLAG_NOLIMIT, a non-pickupable item is rejected because
        // the C++ ordering does pickupable-check before the FLAG_NOLIMIT
        // bypass.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, /*pickupable*/ false, true);
        let no_limit = ReceiverFlag::NoLimit.bits();
        assert_eq!(
            si.query_add(&item, no_limit, false, false),
            StoreInboxQueryResult::CannotPickup
        );
    }

    #[test]
    fn test_query_add_is_self_check_takes_precedence_over_pickupable() {
        // C++ order: `item == this` check is before the pickupable check.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, /*pickupable*/ false, false);
        assert_eq!(
            si.query_add(&item, 0, /*is_self*/ true, false),
            StoreInboxQueryResult::ThisIsImpossible
        );
    }

    // -----------------------------------------------------------------------
    // query_add_thing — null pointer (nullable Thing → not an Item)
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_add_thing_returns_not_possible_for_none() {
        // Mirrors C++ `if (!item) return RETURNVALUE_NOTPOSSIBLE`.
        let si = StoreInbox::new(200);
        assert_eq!(
            si.query_add_thing(None, 0, false, false),
            StoreInboxQueryResult::NotPossible
        );
    }

    #[test]
    fn test_query_add_thing_delegates_to_query_add_for_some() {
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, true, true);
        assert_eq!(
            si.query_add_thing(Some(&item), 0, false, false),
            StoreInboxQueryResult::NoError
        );
    }

    #[test]
    fn test_query_add_thing_some_propagates_rejections() {
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, /*pickupable*/ false, true);
        assert_eq!(
            si.query_add_thing(Some(&item), 0, false, false),
            StoreInboxQueryResult::CannotPickup
        );
    }

    // -----------------------------------------------------------------------
    // FLAG_NOLIMIT bit semantics
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_add_other_flag_bits_do_not_enable_bypass() {
        // Only the NoLimit bit triggers the bypass — other flags must not.
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, true, /*store_item*/ false);
        let other_flags = ReceiverFlag::IgnoreBlockItem.bits()
            | ReceiverFlag::ChildIsOwner.bits()
            | ReceiverFlag::Pathfinding.bits();
        assert!(other_flags & ReceiverFlag::NoLimit.bits() == 0);
        assert_eq!(
            si.query_add(&item, other_flags, false, false),
            StoreInboxQueryResult::CannotMoveItemIsNotStoreItem
        );
    }

    #[test]
    fn test_query_add_no_limit_combined_with_other_flags_still_bypasses() {
        let si = StoreInbox::new(200);
        let item = make_item_full(10, 5, true, /*store_item*/ false);
        let combined = ReceiverFlag::NoLimit.bits() | ReceiverFlag::IgnoreBlockItem.bits();
        assert_eq!(
            si.query_add(&item, combined, false, /*non_empty_container*/ true),
            StoreInboxQueryResult::NoError
        );
    }

    // -----------------------------------------------------------------------
    // Cylinder trait impl — cross-crate dispatch round-trip
    // -----------------------------------------------------------------------

    use forgottenserver_common::cylinder::Cylinder as CommonCylinder;
    use forgottenserver_common::enums::ReturnValue as CommonRv;
    use forgottenserver_common::thing::{DefaultThing, Thing as CommonThing};

    const FLAG_NOLIMIT: u32 = 1;

    #[test]
    fn test_storeinbox_cylinder_query_add_refuses_player_add() {
        let si = StoreInbox::new(200);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&si, 0, &dummy, 1, 0),
            CommonRv::ContainerNotEnoughRoom
        );
    }

    #[test]
    fn test_storeinbox_cylinder_query_add_accepts_server_delivery() {
        let si = StoreInbox::new(200);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&si, 0, &dummy, 1, FLAG_NOLIMIT),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_storeinbox_cylinder_query_remove_refuses_always() {
        let si = StoreInbox::new(200);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&si, &dummy, 1, 0),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_storeinbox_implements_thing_is_container() {
        let si = StoreInbox::new(200);
        assert!(CommonThing::is_container(&si));
        assert!(CommonThing::is_item(&si));
    }

    #[test]
    fn test_storeinbox_is_removed_returns_false_matching_cpp() {
        // C++: StoreInbox extends Container which extends Item. The base
        // Item::isRemoved() returns false by default. StoreInbox does not
        // override this — a freshly created store inbox is not removed.
        let si = StoreInbox::new(200);
        assert!(!CommonThing::is_removed(&si));
    }

    #[test]
    fn test_storeinbox_via_dyn_cylinder_trait_object() {
        let si = StoreInbox::new(200);
        let cyl: &dyn CommonCylinder = &si;
        assert_eq!(cyl.cylinder_first_index(), 0);
        let dummy = DefaultThing;
        assert_eq!(
            cyl.cylinder_query_remove(&dummy, 1, 0),
            CommonRv::NotPossible
        );
    }
}
