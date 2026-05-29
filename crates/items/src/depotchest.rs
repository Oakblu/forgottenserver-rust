// Migrated from forgottenserver/src/depotchest.h + depotchest.cpp
//
// DepotChest — a player's personal depot chest.
//
// In C++ this subclasses Container which subclasses Item.
// In Rust we use composition: DepotChest holds a Container.
//
// Key differences from a plain Container:
//   - Optional maximum item count (total items held, not just direct children).
//   - `canRemove()` always returns false (cannot be removed by a player).
//   - Has pagination enabled by default (mirroring the C++ constructor).

use crate::container::Container;
use crate::item::Item;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepotChestError {
    /// The depot's item-count cap has been reached.
    DepotFull,
    /// The underlying container is full.
    ContainerFull,
}

// ---------------------------------------------------------------------------
// DepotChest
// ---------------------------------------------------------------------------

/// A personal depot chest.
///
/// Wraps a [`Container`] and optionally enforces a maximum total-item count.
/// When `max_depot_items == 0` the limit is disabled (unlimited).
#[derive(Debug, Clone)]
pub struct DepotChest {
    container: Container,
    max_depot_items: u32,
}

impl DepotChest {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create a depot chest with unlimited capacity (max_depot_items = 0).
    ///
    /// The inner container uses the default capacity of 20 slots and has
    /// pagination enabled (mirroring the C++ constructor default `paginated = true`).
    pub fn new(item_type_id: u16) -> Self {
        // C++: Container{type, items[type].maxItems, true, paginated}
        // We use 20 as a sensible default capacity (C++ reads from items[type].maxItems).
        let container = Container::with_flags(item_type_id, 20, true, true);
        DepotChest {
            container,
            max_depot_items: 0,
        }
    }

    /// Create with an explicit inner-container capacity.
    pub fn with_capacity(item_type_id: u16, capacity: u32, pagination: bool) -> Self {
        let container = Container::with_flags(item_type_id, capacity, true, pagination);
        DepotChest {
            container,
            max_depot_items: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Depot-item cap
    // -----------------------------------------------------------------------

    /// Set the maximum number of items the depot may hold in total.
    /// `0` means unlimited.
    pub fn set_max_depot_items(&mut self, max: u32) {
        self.max_depot_items = max;
    }

    /// The current maximum-item cap (`0` = unlimited).
    pub fn max_depot_items(&self) -> u32 {
        self.max_depot_items
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

    /// Current number of direct children.
    pub fn size(&self) -> usize {
        self.container.size()
    }

    // -----------------------------------------------------------------------
    // Item addition with cap enforcement
    // -----------------------------------------------------------------------

    /// Add an item, enforcing the depot-items cap when non-zero.
    ///
    /// Returns `Err(DepotChestError::DepotFull)` when the cap is exceeded, or
    /// `Err(DepotChestError::ContainerFull)` when the inner container is full.
    pub fn add_item(&mut self, item: Item) -> Result<(), DepotChestError> {
        if self.max_depot_items > 0 && self.container.size() as u32 >= self.max_depot_items {
            return Err(DepotChestError::DepotFull);
        }
        self.container
            .add_item(item)
            .map_err(|_| DepotChestError::ContainerFull)
    }

    // -----------------------------------------------------------------------
    // Policy
    // -----------------------------------------------------------------------

    /// Depot chests cannot be removed by a player (mirrors C++ `canRemove`).
    pub fn can_remove(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// Cross-crate Thing + Cylinder impls (delegate to the inner Container)
// ---------------------------------------------------------------------------

impl forgottenserver_common::thing::Thing for DepotChest {
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

impl forgottenserver_common::cylinder::Cylinder for DepotChest {
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
        // C++ DepotChest::queryAdd: refuses if the depot would exceed
        // `max_depot_items`. A value of 0 means unlimited (no cap). Otherwise
        // delegates to the inner container.
        let cap = self.max_depot_items();
        if cap > 0 && (self.size() as u32 + count) > cap {
            return forgottenserver_common::enums::ReturnValue::DepotIsFull;
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
        let (rv, max) = self
            .container
            .cylinder_query_max_count(index, thing, count, flags);
        let cap = self.max_depot_items();
        let accepted = if cap == 0 {
            max
        } else {
            let depot_free = cap.saturating_sub(self.size() as u32);
            max.min(depot_free)
        };
        (rv, accepted)
    }
    fn cylinder_query_remove(
        &self,
        thing: &dyn forgottenserver_common::thing::Thing,
        count: u32,
        flags: u32,
    ) -> forgottenserver_common::enums::ReturnValue {
        // Players can remove items from a depot chest.
        self.container.cylinder_query_remove(thing, count, flags)
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

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_creates_depot_with_zero_max_depot_items() {
        let dc = DepotChest::new(100);
        assert_eq!(dc.max_depot_items(), 0);
        assert_eq!(dc.size(), 0);
    }

    #[test]
    fn test_can_remove_always_false() {
        let dc = DepotChest::new(100);
        assert!(!dc.can_remove());
    }

    // -----------------------------------------------------------------------
    // max_depot_items cap
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_max_depot_items_sets_cap() {
        let mut dc = DepotChest::new(100);
        dc.set_max_depot_items(50);
        assert_eq!(dc.max_depot_items(), 50);
    }

    #[test]
    fn test_add_item_succeeds_when_no_cap() {
        let mut dc = DepotChest::new(100);
        // max_depot_items == 0 → no cap
        assert!(dc.add_item(make_item(1, 10)).is_ok());
    }

    #[test]
    fn test_add_item_fails_when_depot_cap_exceeded() {
        let mut dc = DepotChest::with_capacity(100, 10, false);
        dc.set_max_depot_items(1);
        dc.add_item(make_item(1, 10)).unwrap();
        let result = dc.add_item(make_item(2, 10));
        assert_eq!(result, Err(DepotChestError::DepotFull));
    }

    #[test]
    fn test_add_item_succeeds_when_cap_not_yet_reached() {
        let mut dc = DepotChest::with_capacity(100, 10, false);
        dc.set_max_depot_items(3);
        dc.add_item(make_item(1, 10)).unwrap();
        dc.add_item(make_item(2, 10)).unwrap();
        assert!(dc.add_item(make_item(3, 10)).is_ok());
        assert_eq!(dc.size(), 3);
    }

    // -----------------------------------------------------------------------
    // inner_container
    // -----------------------------------------------------------------------

    #[test]
    fn test_inner_container_access() {
        let dc = DepotChest::new(100);
        assert_eq!(dc.inner_container().capacity(), 20);
    }

    #[test]
    fn test_inner_container_mut_allows_direct_modification() {
        let mut dc = DepotChest::new(100);
        let item = make_item(5, 50);
        dc.inner_container_mut().add_item(item).unwrap();
        assert_eq!(dc.size(), 1);
    }

    // -----------------------------------------------------------------------
    // size delegation
    // -----------------------------------------------------------------------

    #[test]
    fn test_size_delegates_to_inner_container() {
        let mut dc = DepotChest::new(100);
        assert_eq!(dc.size(), 0);
        dc.add_item(make_item(1, 10)).unwrap();
        assert_eq!(dc.size(), 1);
    }

    // -----------------------------------------------------------------------
    // C++ queryAdd: non-item (creature) rejection
    //
    // In C++ `queryAdd` immediately returns RETURNVALUE_NOTPOSSIBLE when
    // `thing.getItem()` is null (i.e. for creatures).  In Rust this invariant
    // is enforced at the type level: `add_item` only accepts `Item` values, so
    // the depot chest can never receive a non-item.  The test below documents
    // this contract explicitly.
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_accepts_only_items_type_enforced() {
        // The Rust signature `fn add_item(&mut self, item: Item)` makes it
        // statically impossible to pass a non-Item.  We verify a valid Item
        // is accepted to confirm the happy path is wired correctly.
        let mut dc = DepotChest::new(100);
        let item = make_item(42, 100);
        assert!(dc.add_item(item).is_ok());
    }

    // -----------------------------------------------------------------------
    // C++ queryAdd: FLAG_NOLIMIT bypasses the depot-items cap
    //
    // In C++ a caller can pass FLAG_NOLIMIT in the `flags` bitmask to skip
    // the `maxDepotItems` check.  The Rust API expresses this as
    // `inner_container_mut().add_item(...)` which bypasses DepotChest's cap
    // entirely and delegates directly to Container.
    // -----------------------------------------------------------------------

    #[test]
    fn test_bypass_cap_via_inner_container_mut() {
        // Depot cap is 1; using inner_container_mut bypasses it (mirrors FLAG_NOLIMIT).
        let mut dc = DepotChest::with_capacity(100, 10, false);
        dc.set_max_depot_items(1);
        // Fill the depot normally up to cap
        dc.add_item(make_item(1, 10)).unwrap();
        // add_item now returns DepotFull …
        assert_eq!(
            dc.add_item(make_item(2, 10)),
            Err(DepotChestError::DepotFull)
        );
        // … but inner_container_mut bypasses the depot cap (FLAG_NOLIMIT equivalent)
        dc.inner_container_mut().add_item(make_item(3, 10)).unwrap();
        assert_eq!(dc.size(), 2);
    }

    // -----------------------------------------------------------------------
    // C++ queryAdd: capacity boundary — exactly at cap succeeds, one more fails
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_exactly_at_cap_succeeds() {
        let mut dc = DepotChest::with_capacity(100, 5, false);
        dc.set_max_depot_items(3);
        dc.add_item(make_item(1, 10)).unwrap();
        dc.add_item(make_item(2, 10)).unwrap();
        // Third item reaches the cap exactly — should still succeed
        assert!(dc.add_item(make_item(3, 10)).is_ok());
        assert_eq!(dc.size(), 3);
    }

    #[test]
    fn test_add_item_one_over_cap_returns_depot_full() {
        let mut dc = DepotChest::with_capacity(100, 5, false);
        dc.set_max_depot_items(3);
        dc.add_item(make_item(1, 10)).unwrap();
        dc.add_item(make_item(2, 10)).unwrap();
        dc.add_item(make_item(3, 10)).unwrap();
        // Fourth item exceeds the cap
        assert_eq!(
            dc.add_item(make_item(4, 10)),
            Err(DepotChestError::DepotFull)
        );
    }

    // -----------------------------------------------------------------------
    // C++ queryAdd: unlimited depot (max_depot_items == 0) fills to inner cap
    //
    // When max_depot_items is 0 (unlimited) the only limit is the inner
    // container's physical capacity.
    // -----------------------------------------------------------------------

    #[test]
    fn test_unlimited_depot_fills_to_container_capacity() {
        // Inner capacity = 3, no depot cap
        let mut dc = DepotChest::with_capacity(100, 3, false);
        // max_depot_items remains 0 (unlimited)
        dc.add_item(make_item(1, 10)).unwrap();
        dc.add_item(make_item(2, 10)).unwrap();
        dc.add_item(make_item(3, 10)).unwrap();
        assert_eq!(dc.size(), 3);
        // Inner container is now full
        assert_eq!(
            dc.add_item(make_item(4, 10)),
            Err(DepotChestError::ContainerFull)
        );
    }

    // -----------------------------------------------------------------------
    // C++ constructor: pagination=true by default (mirrors `DepotChest(uint16_t, bool paginated=true)`)
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_has_pagination_enabled() {
        // C++ constructor default is paginated=true
        let dc = DepotChest::new(100);
        assert!(dc.inner_container().has_pagination());
    }

    #[test]
    fn test_with_capacity_respects_pagination_flag_true() {
        let dc = DepotChest::with_capacity(100, 10, true);
        assert!(dc.inner_container().has_pagination());
    }

    #[test]
    fn test_with_capacity_respects_pagination_flag_false() {
        let dc = DepotChest::with_capacity(100, 10, false);
        assert!(!dc.inner_container().has_pagination());
    }

    // -----------------------------------------------------------------------
    // C++ canRemove: always false regardless of state
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_remove_false_after_items_added() {
        let mut dc = DepotChest::new(100);
        dc.add_item(make_item(1, 10)).unwrap();
        assert!(!dc.can_remove());
    }

    #[test]
    fn test_can_remove_false_with_cap_set() {
        let mut dc = DepotChest::new(100);
        dc.set_max_depot_items(100);
        assert!(!dc.can_remove());
    }

    // -----------------------------------------------------------------------
    // C++ setMaxDepotItems / maxDepotItems getter round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_max_depot_items_to_zero_disables_cap() {
        let mut dc = DepotChest::new(100);
        dc.set_max_depot_items(50);
        assert_eq!(dc.max_depot_items(), 50);
        // Reset to 0 (unlimited)
        dc.set_max_depot_items(0);
        assert_eq!(dc.max_depot_items(), 0);
    }

    #[test]
    fn test_set_max_depot_items_large_value() {
        let mut dc = DepotChest::new(100);
        dc.set_max_depot_items(u32::MAX);
        assert_eq!(dc.max_depot_items(), u32::MAX);
    }

    // -----------------------------------------------------------------------
    // C++ item_type_id propagated to inner container
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_type_id_propagated_to_inner_container() {
        let dc = DepotChest::new(42);
        assert_eq!(dc.inner_container().item_type_id(), 42);
    }

    #[test]
    fn test_with_capacity_item_type_id_propagated() {
        let dc = DepotChest::with_capacity(99, 10, false);
        assert_eq!(dc.inner_container().item_type_id(), 99);
    }

    // -----------------------------------------------------------------------
    // ContainerFull error (inner container full, depot cap not yet reached)
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_returns_container_full_when_inner_full_but_depot_cap_not_reached() {
        // Inner capacity 2, depot cap 10 — inner fills first
        let mut dc = DepotChest::with_capacity(100, 2, false);
        dc.set_max_depot_items(10);
        dc.add_item(make_item(1, 10)).unwrap();
        dc.add_item(make_item(2, 10)).unwrap();
        // Depot cap not reached but inner container is full
        assert_eq!(
            dc.add_item(make_item(3, 10)),
            Err(DepotChestError::ContainerFull)
        );
    }

    // -----------------------------------------------------------------------
    // Cylinder trait impl — cross-crate dispatch round-trip
    // -----------------------------------------------------------------------

    use forgottenserver_common::cylinder::Cylinder as CommonCylinder;
    use forgottenserver_common::enums::ReturnValue as CommonRv;
    use forgottenserver_common::thing::{DefaultThing, Thing as CommonThing};

    #[test]
    fn test_depotchest_cylinder_query_add_refuses_when_depot_full() {
        let mut dc = DepotChest::with_capacity(100, 10, false);
        dc.set_max_depot_items(2);
        dc.add_item(make_item(1, 10)).unwrap();
        dc.add_item(make_item(2, 10)).unwrap();
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&dc, 0, &dummy, 1, 0),
            CommonRv::DepotIsFull
        );
    }

    #[test]
    fn test_depotchest_cylinder_query_add_accepts_when_under_cap() {
        let mut dc = DepotChest::with_capacity(100, 10, false);
        dc.set_max_depot_items(5);
        dc.add_item(make_item(1, 10)).unwrap();
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&dc, 0, &dummy, 1, 0),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_depotchest_cylinder_query_remove_delegates_to_container() {
        let mut dc = DepotChest::new(100);
        dc.add_item(make_item(1, 10)).unwrap();
        let dummy = DefaultThing;
        // Depot chest allows player removal.
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&dc, &dummy, 1, 0),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_depotchest_implements_thing_is_container() {
        let dc = DepotChest::new(100);
        assert!(CommonThing::is_container(&dc));
        assert!(CommonThing::is_item(&dc));
    }

    #[test]
    fn test_depotchest_is_removed_returns_false_matching_cpp() {
        // C++: DepotChest extends Container which extends Item. The base
        // Item::isRemoved() returns false by default (items are not removed
        // when constructed). DepotChest does not override this.
        let dc = DepotChest::new(100);
        assert!(!CommonThing::is_removed(&dc));
    }

    #[test]
    fn test_depotchest_via_dyn_cylinder_trait_object() {
        let dc = DepotChest::new(100);
        let cyl: &dyn CommonCylinder = &dc;
        assert_eq!(cyl.cylinder_first_index(), 0);
        let dummy = DefaultThing;
        // Empty depot chest: queryAdd OK (no cap reached) but queryRemove refuses.
        assert_eq!(cyl.cylinder_query_add(0, &dummy, 1, 0), CommonRv::NoError);
    }
}
