// Migrated from forgottenserver/src/container.h + container.cpp
//
// Container — a generic item container (bag, chest, backpack, etc.).
// In the C++ codebase `Container` extends `Item` via inheritance; here we
// use composition: the caller is responsible for the outer Item-level data
// while this struct manages the item list.
//
// NOTE: Tile-based construction is intentionally dropped to avoid a circular
// dependency (items crate ← map crate ← items crate is forbidden).

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::item::Item;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerError {
    /// Attempted to add an item when the container is already full.
    Full,
    /// Index was out of range.
    OutOfRange,
}

// ---------------------------------------------------------------------------
// ReturnValue — mirrors C++ ReturnValue for Cylinder interface
// ---------------------------------------------------------------------------

/// Return values for the Cylinder query methods (mirrors C++ `ReturnValue`).
///
/// Only the variants relevant to containers are included here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnValue {
    /// Operation is permitted.
    NoError,
    /// Generic denial.
    NotPossible,
    /// Container has no room.
    ContainerNotEnoughRoom,
    /// Container is locked (not unlocked).
    ContainerLocked,
    /// Item is not moveable.
    NotMoveable,
    /// Item is not at the given index.
    ItemNotFound,
}

// ---------------------------------------------------------------------------
// Special index sentinel
// ---------------------------------------------------------------------------

/// Equivalent of C++ `INDEX_WHEREEVER` (-1).
pub const INDEX_WHEREEVER: i32 = -1;

// ---------------------------------------------------------------------------
// Container
// ---------------------------------------------------------------------------

/// A generic item container.
///
/// Items are stored in insertion order.  `add_item` prepends to the front
/// (mirroring C++ `addThing` which calls `push_front`); `add_item_back`
/// appends (mirroring C++ `addItemBack` which calls `push_back`/`addItem`).
#[derive(Debug, Clone)]
pub struct Container {
    /// The server-side type ID for this container item.
    pub item_type_id: u16,
    /// Items currently held, front = index 0.
    items: VecDeque<Item>,
    /// Maximum number of direct children (C++ `maxSize`).
    capacity: u32,
    /// Cached total weight of all contained items.
    total_weight: u32,
    /// Running total of `Item::get_count()` across all held items
    /// (mirrors C++ `Container::ammoCount`).  Updated by every add/remove.
    ammo_count: u32,
    /// Whether items can be added/removed by the player.
    unlocked: bool,
    /// Whether the container uses client-side pagination.
    pagination: bool,
}

impl Container {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create a new container with an explicit capacity and default flags
    /// (`unlocked = true`, `pagination = false`).
    pub fn new(item_type_id: u16, capacity: u32) -> Self {
        Container {
            item_type_id,
            items: VecDeque::new(),
            capacity,
            total_weight: 0,
            ammo_count: 0,
            unlocked: true,
            pagination: false,
        }
    }

    /// Create a container with fully specified flags.
    pub fn with_flags(item_type_id: u16, capacity: u32, unlocked: bool, pagination: bool) -> Self {
        Container {
            item_type_id,
            items: VecDeque::new(),
            capacity,
            total_weight: 0,
            ammo_count: 0,
            unlocked,
            pagination,
        }
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// The item-type ID of this container.
    pub fn item_type_id(&self) -> u16 {
        self.item_type_id
    }

    /// Maximum number of items this container can hold.
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Current number of items directly inside this container.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// `true` when the container holds no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `true` when `size == capacity`.
    pub fn is_full(&self) -> bool {
        self.items.len() as u32 >= self.capacity
    }

    /// Whether items can be moved by a player (mirrors C++ `isUnlocked`).
    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }

    /// Whether client-side pagination is enabled (mirrors C++ `hasPagination`).
    pub fn has_pagination(&self) -> bool {
        self.pagination
    }

    /// Cached sum of the weights of all directly contained items.
    pub fn get_total_weight(&self) -> u32 {
        self.total_weight
    }

    /// Running total of every held item's `get_count()` (mirrors C++
    /// `Container::getAmmoCount`).  For non-stackable items each item
    /// contributes its own `count` (always 1 for non-stackables); for
    /// stackable items it contributes its stack count.
    pub fn get_ammo_count(&self) -> u32 {
        self.ammo_count
    }

    // -----------------------------------------------------------------------
    // Item management
    // -----------------------------------------------------------------------

    /// Prepend an item to the front of the list (mirrors C++ `addThing` /
    /// `internalAddThing` which both call `push_front`).
    ///
    /// Returns `Err(ContainerError::Full)` if the container is already at
    /// capacity.
    pub fn add_item(&mut self, item: Item) -> Result<(), ContainerError> {
        if self.is_full() {
            return Err(ContainerError::Full);
        }
        self.total_weight = self.total_weight.saturating_add(item.get_weight());
        self.ammo_count = self.ammo_count.saturating_add(item.get_count() as u32);
        self.items.push_front(item);
        Ok(())
    }

    /// Append an item to the back of the list (mirrors C++ `addItemBack`).
    ///
    /// Returns `Err(ContainerError::Full)` if the container is already at
    /// capacity.
    pub fn add_item_back(&mut self, item: Item) -> Result<(), ContainerError> {
        if self.is_full() {
            return Err(ContainerError::Full);
        }
        self.total_weight = self.total_weight.saturating_add(item.get_weight());
        self.ammo_count = self.ammo_count.saturating_add(item.get_count() as u32);
        self.items.push_back(item);
        Ok(())
    }

    /// Get an immutable reference to the item at `index` (0 = front).
    pub fn get_item(&self, index: usize) -> Option<&Item> {
        self.items.get(index)
    }

    /// Get a mutable reference to the item at `index` (0 = front).
    pub fn get_item_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.items.get_mut(index)
    }

    /// Remove and return the item at `index`.
    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index >= self.items.len() {
            return None;
        }
        let item = self.items.remove(index)?;
        self.total_weight = self.total_weight.saturating_sub(item.get_weight());
        self.ammo_count = self.ammo_count.saturating_sub(item.get_count() as u32);
        Some(item)
    }

    /// `true` if any directly-held item has the given type ID.
    ///
    /// This is a shallow (non-recursive) check mirroring `isHoldingItem`
    /// when used for type-id matching.  The C++ version takes an `Item*`
    /// pointer; in Rust we compare by type-id since we own items by value.
    pub fn is_holding_item(&self, item_type_id: u16) -> bool {
        self.items.iter().any(|i| i.get_id() == item_type_id)
    }

    // -----------------------------------------------------------------------
    // Iteration
    // -----------------------------------------------------------------------

    /// Iterate over all directly-contained items in order (index 0 first).
    pub fn iter(&self) -> impl Iterator<Item = &crate::item::Item> {
        self.items.iter()
    }

    /// Mutable iteration over directly-contained items.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut crate::item::Item> {
        self.items.iter_mut()
    }

    // -----------------------------------------------------------------------
    // Cylinder interface — query methods (mirrors C++ Cylinder vtable)
    // -----------------------------------------------------------------------

    /// Determine whether `item` may be added to this container at `index`.
    ///
    /// Mirrors C++ `Container::queryAdd`.  Simplified for the Rust model (no
    /// actor / house-tile checks; those live in the game layer).
    ///
    /// Rules enforced here:
    /// - Container must be `unlocked`; otherwise `ContainerLocked`.
    /// - When `index == INDEX_WHEREEVER` and the container is full and does NOT
    ///   use pagination → `ContainerNotEnoughRoom`.
    pub fn query_add(&self, index: i32) -> ReturnValue {
        if !self.unlocked {
            return ReturnValue::ContainerLocked;
        }

        // When placing wherever and the container is full without pagination,
        // reject the addition (C++ line: `if index == INDEX_WHEREEVER &&
        // size() >= capacity() && !hasPagination()`)
        if index == INDEX_WHEREEVER && self.is_full() && !self.pagination {
            return ReturnValue::ContainerNotEnoughRoom;
        }

        ReturnValue::NoError
    }

    /// Determine whether the item at `index` may be removed.
    ///
    /// Mirrors C++ `Container::queryRemove`.
    ///
    /// Rules enforced:
    /// - `index` must be valid; otherwise `ItemNotFound`.
    /// - `count` must be > 0; otherwise `NotPossible`.
    /// - For stackable items: `count` must not exceed the item's stack count;
    ///   otherwise `NotPossible`.
    /// - Item must be moveable; otherwise `NotMoveable`.
    pub fn query_remove(&self, index: usize, count: u32) -> ReturnValue {
        let item = match self.items.get(index) {
            Some(i) => i,
            None => return ReturnValue::ItemNotFound,
        };

        if count == 0 {
            return ReturnValue::NotPossible;
        }

        if item.is_stackable() && count > item.get_count() as u32 {
            return ReturnValue::NotPossible;
        }

        if !item.is_moveable() {
            return ReturnValue::NotMoveable;
        }

        ReturnValue::NoError
    }

    /// Resolve the effective destination index for a placement operation.
    ///
    /// Mirrors C++ `Container::queryDestination`.
    ///
    /// - Index 254 → "move up"; reset to `INDEX_WHEREEVER`, return same
    ///   container (caller would use the parent — in the Rust model we just
    ///   signal this via the returned index).
    /// - Index 255 → "add wherever"; reset to `INDEX_WHEREEVER`.
    /// - Index ≥ capacity → beyond the visible slots; reset to `INDEX_WHEREEVER`.
    /// - Otherwise: index is unchanged.
    ///
    /// Returns the resolved index.
    pub fn query_destination(&self, index: i32) -> i32 {
        if !self.unlocked {
            return index;
        }

        if index == 254 {
            // "Move up" — treat as INDEX_WHEREEVER so the item lands in the
            // parent (or this container when no parent is tracked here).
            return INDEX_WHEREEVER;
        }

        if index == 255 {
            // "Add wherever"
            return INDEX_WHEREEVER;
        }

        if index >= 0 && index as u32 >= self.capacity {
            // Slot is within the grey (client-side overflow) area.
            return INDEX_WHEREEVER;
        }

        index
    }

    // -----------------------------------------------------------------------
    // Item count helpers
    // -----------------------------------------------------------------------

    /// Count of items directly held (non-recursive).
    ///
    /// Mirrors C++ `getItemHoldingCount` which is recursive via
    /// `ContainerIterator`; since the Rust container does not embed nested
    /// containers inside `Item`, this returns the flat item count.
    pub fn get_item_holding_count(&self) -> usize {
        self.items.len()
    }
    /// Count how many items of `item_type_id` are directly held. When `sub_type` is `Some(s)`, only items whose `count` equals `s` are counted (mirrors C++ `getItemTypeCount` with `subType != -1`). When `sub_type` is `None`, all items of that type are counted. Mirrors C++ `Container::getItemTypeCount`.
    pub fn get_item_type_count(&self, item_type_id: u16, sub_type: Option<u8>) -> u32 {
        let mut total: u32 = 0;
        for item in &self.items {
            if item.get_id() != item_type_id {
                continue;
            }
            match sub_type {
                Some(st) if item.get_count() == st => total += item.get_count() as u32,
                Some(_) => {}
                None => total += item.get_count() as u32,
            }
        }
        total
    }

    /// Accumulate item-type → total-count mapping for all directly-held items.
    ///
    /// Mirrors C++ `Container::getAllItemTypeCount`.
    pub fn get_all_item_type_count(&self) -> HashMap<u16, u32> {
        let mut map: HashMap<u16, u32> = HashMap::new();
        for item in &self.items {
            *map.entry(item.get_id()).or_insert(0) += item.get_count() as u32;
        }
        map
    }

    // -----------------------------------------------------------------------
    // Serialization helpers
    // -----------------------------------------------------------------------

    /// Return the number of items that would be serialised for this container.
    ///
    /// In the C++ OTB format the count is stored as `ATTR_CONTAINER_ITEMS`
    /// before writing child nodes.  This value equals `size()` for non-special
    /// containers.
    pub fn serialization_count(&self) -> u32 {
        self.items.len() as u32
    }

    /// Serialise the container header (item count) into `buf`.
    ///
    /// Format: `ATTR_CONTAINER_ITEMS (0x78)` tag followed by a `u32 le` count.
    /// This mirrors the C++ `readAttr(ATTR_CONTAINER_ITEMS, propStream)` path.
    pub fn serialize_container_header(&self, buf: &mut Vec<u8>) {
        const ATTR_CONTAINER_ITEMS: u8 = 0x78;
        buf.push(ATTR_CONTAINER_ITEMS);
        buf.extend_from_slice(&self.serialization_count().to_le_bytes());
    }

    /// Deserialise the container header from `data`, returning the item count.
    ///
    /// Expects: `ATTR_CONTAINER_ITEMS (0x78)` byte then `u32 le`.
    /// Returns `Err` if the data is malformed or the tag is missing.
    pub fn deserialize_container_header(data: &[u8]) -> Result<u32, String> {
        const ATTR_CONTAINER_ITEMS: u8 = 0x78;
        if data.len() < 5 {
            return Err("container header: too short".into());
        }
        if data[0] != ATTR_CONTAINER_ITEMS {
            return Err(format!(
                "container header: expected tag 0x78, got 0x{:02X}",
                data[0]
            ));
        }
        let count = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
        Ok(count)
    }
}

// ---------------------------------------------------------------------------
// Cross-crate Thing + Cylinder impls
//
// The C++ `Container` extends `Item` which extends `Thing`. In Rust we adapt
// the existing API to satisfy the cross-crate `Cylinder` contract from the
// `common` crate, allowing higher crates (game/network/server) to dispatch
// through `&dyn Cylinder` without depending on the concrete `Container` type.
// ---------------------------------------------------------------------------

impl forgottenserver_common::thing::Thing for Container {
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
        0
    }
    fn get_last_index(&self) -> usize {
        self.items.len()
    }
    fn get_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
        let sub = if sub_type == -1 {
            None
        } else {
            Some(sub_type as u8)
        };
        Container::get_item_type_count(self, item_id, sub)
    }
    fn get_all_item_type_count<'a>(
        &self,
        count_map: &'a mut HashMap<u32, u32>,
    ) -> &'a mut HashMap<u32, u32> {
        for (id, n) in Container::get_all_item_type_count(self) {
            *count_map.entry(id as u32).or_insert(0) += n;
        }
        count_map
    }
}

impl forgottenserver_common::cylinder::Cylinder for Container {
    fn cylinder_first_index(&self) -> usize {
        0
    }
    fn cylinder_last_index(&self) -> usize {
        self.items.len()
    }
    fn cylinder_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
        let sub = if sub_type == -1 {
            None
        } else {
            Some(sub_type as u8)
        };
        Container::get_item_type_count(self, item_id, sub)
    }
    fn cylinder_thing_index(
        &self,
        _child: &dyn forgottenserver_common::thing::Thing,
    ) -> Option<i32> {
        // Concrete pointer comparison is not possible across trait objects
        // without downcasting; for the cross-crate dispatch layer we return
        // None and let callers that know the concrete type use the inherent
        // `iter()` to find the index. This matches the C++ default which
        // returns -1 for things the container doesn't recognise.
        None
    }
    fn cylinder_query_add(
        &self,
        index: i32,
        _thing: &dyn forgottenserver_common::thing::Thing,
        _count: u32,
        flags: u32,
    ) -> forgottenserver_common::enums::ReturnValue {
        // FLAG_NOLIMIT (1 << 0): server-side adds bypass lock/capacity checks.
        const FLAG_NOLIMIT: u32 = 1;
        if (flags & FLAG_NOLIMIT) != 0 {
            return forgottenserver_common::enums::ReturnValue::NoError;
        }
        match Container::query_add(self, index) {
            ReturnValue::NoError => forgottenserver_common::enums::ReturnValue::NoError,
            ReturnValue::ContainerNotEnoughRoom => {
                forgottenserver_common::enums::ReturnValue::ContainerNotEnoughRoom
            }
            ReturnValue::ContainerLocked => forgottenserver_common::enums::ReturnValue::NotPossible,
            _ => forgottenserver_common::enums::ReturnValue::NotPossible,
        }
    }
    fn cylinder_query_max_count(
        &self,
        index: i32,
        thing: &dyn forgottenserver_common::thing::Thing,
        count: u32,
        flags: u32,
    ) -> (forgottenserver_common::enums::ReturnValue, u32) {
        let rv = self.cylinder_query_add(index, thing, count, flags);
        let free = (self.capacity as i64 - self.items.len() as i64).max(0) as u32;
        let accepted = count.min(free);
        (rv, accepted)
    }
    fn cylinder_query_remove(
        &self,
        _thing: &dyn forgottenserver_common::thing::Thing,
        count: u32,
        _flags: u32,
    ) -> forgottenserver_common::enums::ReturnValue {
        // Without index resolution, the most we can answer is whether the
        // container has any item the caller could remove. Matches C++ default
        // behaviour when the trait does not know the concrete index.
        if self.items.is_empty() || count == 0 {
            forgottenserver_common::enums::ReturnValue::NotPossible
        } else {
            forgottenserver_common::enums::ReturnValue::NoError
        }
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

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn make_item(id: u16, weight: u32, stackable: bool, count: u8) -> Item {
        let td = ItemTypeData {
            id,
            client_id: id,
            weight,
            stackable,
            pickupable: true,
            moveable: true,
            ..Default::default()
        };
        Item::new(Arc::new(td), count)
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_new_creates_empty_container_with_given_capacity() {
        let c = Container::new(10, 8);
        assert_eq!(c.item_type_id(), 10);
        assert_eq!(c.capacity(), 8);
        assert_eq!(c.size(), 0);
        assert!(c.is_empty());
    }

    #[test]
    fn test_new_container_is_not_full() {
        let c = Container::new(1, 4);
        assert!(!c.is_full());
    }

    // -----------------------------------------------------------------------
    // Flags
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_unlocked_true_by_default() {
        let c = Container::new(1, 4);
        assert!(c.is_unlocked());
    }

    #[test]
    fn test_has_pagination_false_by_default() {
        let c = Container::new(1, 4);
        assert!(!c.has_pagination());
    }

    #[test]
    fn test_with_flags_sets_unlocked_and_pagination() {
        let c = Container::with_flags(1, 4, false, true);
        assert!(!c.is_unlocked());
        assert!(c.has_pagination());
    }

    // -----------------------------------------------------------------------
    // add_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_to_empty_container_succeeds() {
        let mut c = Container::new(1, 4);
        let item = make_item(5, 100, false, 1);
        assert!(c.add_item(item).is_ok());
        assert_eq!(c.size(), 1);
    }

    #[test]
    fn test_add_item_when_full_returns_err() {
        let mut c = Container::new(1, 2);
        c.add_item(make_item(5, 10, false, 1)).unwrap();
        c.add_item(make_item(6, 10, false, 1)).unwrap();
        let result = c.add_item(make_item(7, 10, false, 1));
        assert_eq!(result, Err(ContainerError::Full));
    }

    #[test]
    fn test_add_item_prepends_to_front() {
        // C++ `addThing` calls `push_front`
        let mut c = Container::new(1, 4);
        let item_a = make_item(1, 10, false, 1);
        let item_b = make_item(2, 20, false, 1);
        c.add_item(item_a).unwrap();
        c.add_item(item_b).unwrap();
        // item_b was added last → it should be at index 0
        assert_eq!(c.get_item(0).unwrap().get_id(), 2);
        assert_eq!(c.get_item(1).unwrap().get_id(), 1);
    }

    #[test]
    fn test_is_full_when_size_equals_capacity() {
        let mut c = Container::new(1, 1);
        assert!(!c.is_full());
        c.add_item(make_item(5, 10, false, 1)).unwrap();
        assert!(c.is_full());
    }

    // -----------------------------------------------------------------------
    // get_item / remove_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_valid_index_returns_some() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(7, 50, false, 1)).unwrap();
        let item = c.get_item(0);
        assert!(item.is_some());
        assert_eq!(item.unwrap().get_id(), 7);
    }

    #[test]
    fn test_get_item_out_of_range_returns_none() {
        let c = Container::new(1, 4);
        assert!(c.get_item(0).is_none());
        assert!(c.get_item(99).is_none());
    }

    #[test]
    fn test_remove_item_returns_item_and_shrinks_list() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(3, 30, false, 1)).unwrap();
        let removed = c.remove_item(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get_id(), 3);
        assert_eq!(c.size(), 0);
    }

    #[test]
    fn test_remove_item_out_of_range_returns_none() {
        let mut c = Container::new(1, 4);
        assert!(c.remove_item(0).is_none());
    }

    // -----------------------------------------------------------------------
    // get_total_weight
    // -----------------------------------------------------------------------

    #[test]
    fn test_total_weight_empty_is_zero() {
        let c = Container::new(1, 4);
        assert_eq!(c.get_total_weight(), 0);
    }

    #[test]
    fn test_total_weight_accumulates_on_add() {
        let mut c = Container::new(1, 4);
        // item with weight 100 (non-stackable, count=1)
        c.add_item(make_item(1, 100, false, 1)).unwrap();
        assert_eq!(c.get_total_weight(), 100);
        // stackable item: weight 50 * count 3 = 150
        c.add_item(make_item(2, 50, true, 3)).unwrap();
        assert_eq!(c.get_total_weight(), 250);
    }

    #[test]
    fn test_total_weight_decreases_on_remove() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(1, 100, false, 1)).unwrap();
        c.add_item(make_item(2, 200, false, 1)).unwrap();
        c.remove_item(0); // removes most-recently-added (front)
        assert_eq!(c.get_total_weight(), 100);
    }

    // -----------------------------------------------------------------------
    // is_holding_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_holding_item_true_when_present() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(42, 10, false, 1)).unwrap();
        assert!(c.is_holding_item(42));
    }

    #[test]
    fn test_is_holding_item_false_when_absent() {
        let c = Container::new(1, 4);
        assert!(!c.is_holding_item(99));
    }

    // -----------------------------------------------------------------------
    // iter
    // -----------------------------------------------------------------------

    #[test]
    fn test_iter_yields_items_in_order() {
        let mut c = Container::new(1, 4);
        // add_item prepends, so: push 1 → [1], push 2 → [2, 1]
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        let ids: Vec<u16> = c.iter().map(|i| i.get_id()).collect();
        assert_eq!(ids, vec![2, 1]);
    }

    #[test]
    fn test_iter_empty_yields_nothing() {
        let c = Container::new(1, 4);
        assert_eq!(c.iter().count(), 0);
    }

    // -----------------------------------------------------------------------
    // add_item_back
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_back_appends_to_end() {
        let mut c = Container::new(1, 4);
        c.add_item_back(make_item(1, 10, false, 1)).unwrap();
        c.add_item_back(make_item(2, 10, false, 1)).unwrap();
        // Both appended → [1, 2]
        assert_eq!(c.get_item(0).unwrap().get_id(), 1);
        assert_eq!(c.get_item(1).unwrap().get_id(), 2);
    }

    #[test]
    fn test_add_item_back_full_returns_err() {
        let mut c = Container::new(1, 1);
        c.add_item_back(make_item(1, 10, false, 1)).unwrap();
        assert_eq!(
            c.add_item_back(make_item(2, 10, false, 1)),
            Err(ContainerError::Full)
        );
    }

    // -----------------------------------------------------------------------
    // query_add — Cylinder interface
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_add_unlocked_empty_whereever_ok() {
        let c = Container::new(1, 4);
        assert_eq!(c.query_add(INDEX_WHEREEVER), ReturnValue::NoError);
    }

    #[test]
    fn test_query_add_unlocked_specific_index_ok() {
        let c = Container::new(1, 4);
        assert_eq!(c.query_add(0), ReturnValue::NoError);
    }

    #[test]
    fn test_query_add_locked_container_returns_locked() {
        let c = Container::with_flags(1, 4, false, false);
        assert_eq!(c.query_add(INDEX_WHEREEVER), ReturnValue::ContainerLocked);
    }

    #[test]
    fn test_query_add_full_without_pagination_returns_no_room() {
        let mut c = Container::new(1, 2);
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        assert!(c.is_full());
        assert_eq!(
            c.query_add(INDEX_WHEREEVER),
            ReturnValue::ContainerNotEnoughRoom
        );
    }

    #[test]
    fn test_query_add_full_with_pagination_still_ok() {
        // With pagination enabled, full containers still accept items
        let mut c = Container::with_flags(1, 2, true, true);
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        assert!(c.is_full());
        // pagination = true → no room check is bypassed
        assert_eq!(c.query_add(INDEX_WHEREEVER), ReturnValue::NoError);
    }

    #[test]
    fn test_query_add_full_but_specific_index_ok() {
        // A specific index does not trigger the "full" rejection (C++ only
        // rejects when index == INDEX_WHEREEVER)
        let mut c = Container::new(1, 2);
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        assert!(c.is_full());
        assert_eq!(c.query_add(0), ReturnValue::NoError);
    }

    // -----------------------------------------------------------------------
    // query_remove — Cylinder interface
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_remove_valid_moveable_item_ok() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(7, 50, false, 1)).unwrap();
        assert_eq!(c.query_remove(0, 1), ReturnValue::NoError);
    }

    #[test]
    fn test_query_remove_invalid_index_returns_item_not_found() {
        let c = Container::new(1, 4);
        assert_eq!(c.query_remove(0, 1), ReturnValue::ItemNotFound);
    }

    #[test]
    fn test_query_remove_count_zero_returns_not_possible() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(7, 50, false, 1)).unwrap();
        assert_eq!(c.query_remove(0, 0), ReturnValue::NotPossible);
    }

    #[test]
    fn test_query_remove_stackable_count_exceeds_stack_returns_not_possible() {
        let mut c = Container::new(1, 4);
        // stackable item with count 5
        c.add_item(make_item(2, 10, true, 5)).unwrap();
        // Requesting more than the stack size should be rejected
        assert_eq!(c.query_remove(0, 10), ReturnValue::NotPossible);
    }

    #[test]
    fn test_query_remove_stackable_count_equals_stack_ok() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(2, 10, true, 5)).unwrap();
        assert_eq!(c.query_remove(0, 5), ReturnValue::NoError);
    }

    #[test]
    fn test_query_remove_non_moveable_returns_not_moveable() {
        // Build an immoveable item
        let td = crate::items_registry::ItemTypeData {
            id: 99,
            client_id: 99,
            weight: 100,
            stackable: false,
            pickupable: true,
            moveable: false, // <-- not moveable
            ..Default::default()
        };
        let item = Item::new(Arc::new(td), 1);

        let mut c = Container::new(1, 4);
        c.add_item(item).unwrap();
        assert_eq!(c.query_remove(0, 1), ReturnValue::NotMoveable);
    }

    // -----------------------------------------------------------------------
    // query_destination — Cylinder interface
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_destination_254_returns_whereever() {
        let c = Container::new(1, 8);
        assert_eq!(c.query_destination(254), INDEX_WHEREEVER);
    }

    #[test]
    fn test_query_destination_255_returns_whereever() {
        let c = Container::new(1, 8);
        assert_eq!(c.query_destination(255), INDEX_WHEREEVER);
    }

    #[test]
    fn test_query_destination_beyond_capacity_returns_whereever() {
        // capacity = 4; index 4 is beyond → INDEX_WHEREEVER
        let c = Container::new(1, 4);
        assert_eq!(c.query_destination(4), INDEX_WHEREEVER);
        assert_eq!(c.query_destination(10), INDEX_WHEREEVER);
    }

    #[test]
    fn test_query_destination_valid_index_unchanged() {
        let c = Container::new(1, 8);
        assert_eq!(c.query_destination(0), 0);
        assert_eq!(c.query_destination(3), 3);
        assert_eq!(c.query_destination(7), 7);
    }

    #[test]
    fn test_query_destination_index_whereever_unchanged() {
        let c = Container::new(1, 8);
        assert_eq!(c.query_destination(INDEX_WHEREEVER), INDEX_WHEREEVER);
    }

    #[test]
    fn test_query_destination_locked_container_returns_index_unchanged() {
        let c = Container::with_flags(1, 4, false, false);
        // When locked, queryDestination in C++ sets destItem=nullptr and
        // returns `this` unchanged — we mirror that by returning the index as-is.
        assert_eq!(c.query_destination(2), 2);
    }

    // -----------------------------------------------------------------------
    // get_item_holding_count
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_holding_count_empty() {
        let c = Container::new(1, 4);
        assert_eq!(c.get_item_holding_count(), 0);
    }

    #[test]
    fn test_get_item_holding_count_after_adds() {
        let mut c = Container::new(1, 8);
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        c.add_item(make_item(3, 10, false, 1)).unwrap();
        assert_eq!(c.get_item_holding_count(), 3);
    }

    #[test]
    fn test_get_item_holding_count_decreases_on_remove() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        c.remove_item(0);
        assert_eq!(c.get_item_holding_count(), 1);
    }

    // -----------------------------------------------------------------------
    // get_item_type_count
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_type_count_no_matching_items() {
        let c = Container::new(1, 4);
        assert_eq!(c.get_item_type_count(42, None), 0);
    }

    #[test]
    fn test_get_item_type_count_single_match() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(42, 10, false, 1)).unwrap();
        assert_eq!(c.get_item_type_count(42, None), 1);
    }

    #[test]
    fn test_get_item_type_count_multiple_matches_summed() {
        let mut c = Container::new(1, 8);
        // Two stackable items of the same type with count 3 each
        c.add_item(make_item(5, 10, true, 3)).unwrap();
        c.add_item(make_item(5, 10, true, 7)).unwrap();
        // Without sub_type filter: sum = 3 + 7 = 10
        assert_eq!(c.get_item_type_count(5, None), 10);
    }

    #[test]
    fn test_get_item_type_count_with_subtype_filter() {
        let mut c = Container::new(1, 8);
        c.add_item(make_item(5, 10, true, 3)).unwrap();
        c.add_item(make_item(5, 10, true, 7)).unwrap();
        // sub_type = Some(3) — only the first item matches
        assert_eq!(c.get_item_type_count(5, Some(3)), 3);
        // sub_type = Some(7) — only the second item matches
        assert_eq!(c.get_item_type_count(5, Some(7)), 7);
        // sub_type = Some(99) — no match
        assert_eq!(c.get_item_type_count(5, Some(99)), 0);
    }

    #[test]
    fn test_get_item_type_count_different_types_not_counted() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(10, 10, false, 1)).unwrap();
        c.add_item(make_item(20, 10, false, 1)).unwrap();
        assert_eq!(c.get_item_type_count(10, None), 1);
        assert_eq!(c.get_item_type_count(20, None), 1);
        assert_eq!(c.get_item_type_count(99, None), 0);
    }

    // -----------------------------------------------------------------------
    // get_all_item_type_count
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_all_item_type_count_empty() {
        let c = Container::new(1, 4);
        assert!(c.get_all_item_type_count().is_empty());
    }

    #[test]
    fn test_get_all_item_type_count_single_item() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(10, 10, false, 1)).unwrap();
        let map = c.get_all_item_type_count();
        assert_eq!(map.len(), 1);
        assert_eq!(map[&10], 1);
    }

    #[test]
    fn test_get_all_item_type_count_multiple_types() {
        let mut c = Container::new(1, 8);
        c.add_item(make_item(10, 10, true, 5)).unwrap();
        c.add_item(make_item(20, 10, false, 1)).unwrap();
        c.add_item(make_item(10, 10, true, 3)).unwrap();
        let map = c.get_all_item_type_count();
        // type 10: 5 + 3 = 8
        assert_eq!(map[&10], 8);
        // type 20: 1
        assert_eq!(map[&20], 1);
        assert_eq!(map.len(), 2);
    }

    // -----------------------------------------------------------------------
    // serialize_container_header / deserialize_container_header
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_container_header_empty() {
        let c = Container::new(1, 4);
        let mut buf = Vec::new();
        c.serialize_container_header(&mut buf);
        // tag(1) + count(4) = 5 bytes, count = 0
        assert_eq!(buf.len(), 5);
        assert_eq!(buf[0], 0x78);
        assert_eq!(u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]), 0);
    }

    #[test]
    fn test_serialize_container_header_with_items() {
        let mut c = Container::new(1, 8);
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        c.add_item(make_item(3, 10, false, 1)).unwrap();
        let mut buf = Vec::new();
        c.serialize_container_header(&mut buf);
        assert_eq!(buf[0], 0x78);
        let count = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_deserialize_container_header_ok() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(5, 10, false, 1)).unwrap();
        c.add_item(make_item(6, 10, false, 1)).unwrap();
        let mut buf = Vec::new();
        c.serialize_container_header(&mut buf);

        let count = Container::deserialize_container_header(&buf).expect("ok");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_deserialize_container_header_wrong_tag_returns_err() {
        // Wrong tag byte
        let buf = vec![0x77, 0x00, 0x00, 0x00, 0x00];
        let result = Container::deserialize_container_header(&buf);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("expected tag 0x78"), "msg: {}", msg);
    }

    #[test]
    fn test_deserialize_container_header_too_short_returns_err() {
        let buf = vec![0x78, 0x01]; // too short
        let result = Container::deserialize_container_header(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_count_equals_size() {
        let mut c = Container::new(1, 8);
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        assert_eq!(c.serialization_count(), c.size() as u32);
    }

    // -----------------------------------------------------------------------
    // Round-trip: serialize + deserialize container header
    // -----------------------------------------------------------------------

    #[test]
    fn test_container_header_round_trip() {
        let mut c = Container::new(42, 20);
        for i in 0..7u16 {
            c.add_item(make_item(i + 1, 10, false, 1)).unwrap();
        }
        assert_eq!(c.serialization_count(), 7);

        let mut buf = Vec::new();
        c.serialize_container_header(&mut buf);

        let recovered = Container::deserialize_container_header(&buf).expect("ok");
        assert_eq!(recovered, 7);
    }

    // -----------------------------------------------------------------------
    // get_item_mut
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_mut_valid_index_returns_some_and_allows_mutation() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(7, 50, true, 3)).unwrap();
        // Mutate via the returned reference and observe the change after.
        {
            let item = c.get_item_mut(0).expect("item exists");
            item.set_item_count(9);
        }
        assert_eq!(c.get_item(0).unwrap().get_count(), 9);
    }

    #[test]
    fn test_get_item_mut_out_of_range_returns_none() {
        let mut c = Container::new(1, 4);
        assert!(c.get_item_mut(0).is_none());
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        assert!(c.get_item_mut(5).is_none());
    }

    // -----------------------------------------------------------------------
    // iter_mut
    // -----------------------------------------------------------------------

    #[test]
    fn test_iter_mut_yields_each_item_and_allows_mutation() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(1, 10, true, 2)).unwrap();
        c.add_item(make_item(2, 10, true, 4)).unwrap();
        // Bump every item's stack count by 1 via the mutable iterator.
        for item in c.iter_mut() {
            let n = item.get_count();
            item.set_item_count(n + 1);
        }
        // add_item prepends, so insertion order [1(c=2), 2(c=4)] becomes
        // front-to-back [2(c=5), 1(c=3)] after +1 bumps.
        let counts: Vec<u8> = c.iter().map(|i| i.get_count()).collect();
        assert_eq!(counts, vec![5, 3]);
    }

    #[test]
    fn test_iter_mut_empty_yields_nothing() {
        let mut c = Container::new(1, 4);
        assert_eq!(c.iter_mut().count(), 0);
    }

    // -----------------------------------------------------------------------
    // get_ammo_count — mirrors C++ Container::ammoCount bookkeeping
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_ammo_count_empty_is_zero() {
        let c = Container::new(1, 4);
        assert_eq!(c.get_ammo_count(), 0);
    }

    #[test]
    fn test_get_ammo_count_add_item_increments_by_item_count() {
        let mut c = Container::new(1, 4);
        // Non-stackable item: count == 1
        c.add_item(make_item(1, 10, false, 1)).unwrap();
        assert_eq!(c.get_ammo_count(), 1);
        // Stackable item: count == 7
        c.add_item(make_item(2, 10, true, 7)).unwrap();
        assert_eq!(c.get_ammo_count(), 8);
    }

    #[test]
    fn test_get_ammo_count_add_item_back_increments_by_item_count() {
        let mut c = Container::new(1, 4);
        c.add_item_back(make_item(1, 10, true, 5)).unwrap();
        c.add_item_back(make_item(2, 10, true, 3)).unwrap();
        assert_eq!(c.get_ammo_count(), 8);
    }

    #[test]
    fn test_get_ammo_count_remove_item_decrements_by_item_count() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(1, 10, true, 4)).unwrap(); // ammo += 4
        c.add_item(make_item(2, 10, true, 6)).unwrap(); // ammo += 6 → 10
        assert_eq!(c.get_ammo_count(), 10);

        // remove_item(0) removes the most-recently-added (front, count=6)
        let removed = c.remove_item(0).expect("removed");
        assert_eq!(removed.get_count(), 6);
        assert_eq!(c.get_ammo_count(), 4);

        let removed2 = c.remove_item(0).expect("removed");
        assert_eq!(removed2.get_count(), 4);
        assert_eq!(c.get_ammo_count(), 0);
    }

    #[test]
    fn test_get_ammo_count_add_when_full_does_not_change() {
        let mut c = Container::new(1, 1);
        c.add_item(make_item(1, 10, true, 5)).unwrap();
        assert_eq!(c.get_ammo_count(), 5);
        // Container is now full — subsequent add must NOT bump ammo.
        let result = c.add_item(make_item(2, 10, true, 9));
        assert_eq!(result, Err(ContainerError::Full));
        assert_eq!(c.get_ammo_count(), 5);

        let result = c.add_item_back(make_item(3, 10, true, 9));
        assert_eq!(result, Err(ContainerError::Full));
        assert_eq!(c.get_ammo_count(), 5);
    }

    #[test]
    fn test_get_ammo_count_remove_out_of_range_does_not_change() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(1, 10, true, 3)).unwrap();
        assert_eq!(c.get_ammo_count(), 3);
        assert!(c.remove_item(99).is_none());
        assert_eq!(c.get_ammo_count(), 3);
    }

    // -----------------------------------------------------------------------
    // Cylinder trait impl — cross-crate dispatch round-trip
    // -----------------------------------------------------------------------

    use forgottenserver_common::cylinder::Cylinder as CommonCylinder;
    use forgottenserver_common::enums::ReturnValue as CommonRv;
    use forgottenserver_common::thing::{DefaultThing, Thing as CommonThing};

    #[test]
    fn test_cylinder_first_index_returns_zero() {
        let c = Container::new(1, 8);
        assert_eq!(CommonCylinder::cylinder_first_index(&c), 0);
    }

    #[test]
    fn test_cylinder_last_index_reflects_size() {
        let mut c = Container::new(1, 8);
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        c.add_item(make_item(3, 10, false, 1)).unwrap();
        assert_eq!(CommonCylinder::cylinder_last_index(&c), 2);
    }

    #[test]
    fn test_cylinder_item_type_count_matches_inherent() {
        let mut c = Container::new(1, 8);
        c.add_item(make_item(2160, 10, true, 5)).unwrap();
        c.add_item(make_item(2160, 10, true, 3)).unwrap();
        assert_eq!(CommonCylinder::cylinder_item_type_count(&c, 2160, -1), 8);
        assert_eq!(CommonCylinder::cylinder_item_type_count(&c, 9999, -1), 0);
    }

    #[test]
    fn test_cylinder_thing_index_returns_none_without_concrete_downcast() {
        let c = Container::new(1, 8);
        let dummy = DefaultThing;
        assert_eq!(CommonCylinder::cylinder_thing_index(&c, &dummy), None);
    }

    #[test]
    fn test_cylinder_query_add_accepts_when_room_available() {
        let c = Container::new(1, 8);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&c, INDEX_WHEREEVER, &dummy, 1, 0),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_cylinder_query_add_refuses_when_full() {
        let mut c = Container::new(1, 2);
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        c.add_item(make_item(3, 10, false, 1)).unwrap();
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&c, INDEX_WHEREEVER, &dummy, 1, 0),
            CommonRv::ContainerNotEnoughRoom
        );
    }

    #[test]
    fn test_cylinder_query_add_refuses_when_locked() {
        let c = Container::with_flags(1, 4, false, false);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&c, 0, &dummy, 1, 0),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_cylinder_query_max_count_reports_free_slots() {
        let mut c = Container::new(1, 5);
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        c.add_item(make_item(3, 10, false, 1)).unwrap();
        let dummy = DefaultThing;
        // 3 slots free, asking for 10 → accept 3.
        let (rv, accepted) =
            CommonCylinder::cylinder_query_max_count(&c, INDEX_WHEREEVER, &dummy, 10, 0);
        assert_eq!(rv, CommonRv::NoError);
        assert_eq!(accepted, 3);
    }

    #[test]
    fn test_cylinder_query_remove_refuses_when_empty() {
        let c = Container::new(1, 4);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&c, &dummy, 1, 0),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_cylinder_query_remove_accepts_when_nonempty() {
        let mut c = Container::new(1, 4);
        c.add_item(make_item(2, 10, false, 1)).unwrap();
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&c, &dummy, 1, 0),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_container_implements_thing_is_container() {
        let c = Container::new(1, 4);
        assert!(CommonThing::is_container(&c));
        assert!(CommonThing::is_item(&c));
        assert!(!CommonThing::is_creature(&c));
        assert!(!CommonThing::is_removed(&c));
    }

    #[test]
    fn test_container_get_all_item_type_count_via_thing_accumulates() {
        use std::collections::HashMap;
        let mut c = Container::new(1, 4);
        c.add_item(make_item(2160, 10, true, 5)).unwrap();
        c.add_item(make_item(2152, 10, true, 2)).unwrap();
        let mut map: HashMap<u32, u32> = HashMap::new();
        map.insert(2160, 1); // pre-existing entry
        CommonThing::get_all_item_type_count(&c, &mut map);
        assert_eq!(map.get(&2160), Some(&6));
        assert_eq!(map.get(&2152), Some(&2));
    }

    #[test]
    fn test_container_via_dyn_cylinder_trait_object() {
        // Round-trip the container through a &dyn Cylinder reference — proves
        // higher crates can dispatch without the concrete Container type.
        let c = Container::new(1, 4);
        let cyl: &dyn CommonCylinder = &c;
        assert_eq!(cyl.cylinder_first_index(), 0);
        assert_eq!(cyl.cylinder_last_index(), 0);
        let dummy = DefaultThing;
        assert_eq!(
            cyl.cylinder_query_add(INDEX_WHEREEVER, &dummy, 1, 0),
            CommonRv::NoError
        );
    }
}
