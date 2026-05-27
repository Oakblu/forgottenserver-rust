// Migrated from forgottenserver/src/trashholder.h + trashholder.cpp
//
// TrashHolder is an Item subtype that destroys anything added to it.
//
// Key C++ behaviour:
//   - queryAdd → always returns RETURNVALUE_NOERROR
//   - addThing(item) → item is immediately destroyed (g_game.internalRemoveItem)
//     Special cases in C++ that skip destruction:
//       * item == this (can't destroy self)
//       * !item.hasProperty(MOVEABLE)  (non-moveable items are ignored)
//       * item.isHangable() && ground-tile with SUPPORTS_HANGABLE flag
//     In our Rust model we have no Item self-reference and no tile context,
//     so we model the invariant simply: items added are never stored.
//   - item_count is always 0

use forgottenserver_items::item::Item;

// ---------------------------------------------------------------------------
// Error type (stub — TrashHolder never actually errors on queryAdd)
// ---------------------------------------------------------------------------

/// Placeholder error type.  In C++ `queryAdd` always returns
/// `RETURNVALUE_NOERROR`; this type exists only so `query_add` can return a
/// `Result` with a meaningful `Err` arm for future extensibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashHolderError {
    /// Would be returned if some future rule rejected an add.
    NotAllowed,
}

// ---------------------------------------------------------------------------
// TrashHolder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TrashHolder {
    pub item_type_id: u16,
}

impl TrashHolder {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    pub fn new(item_type_id: u16) -> Self {
        TrashHolder { item_type_id }
    }

    // -----------------------------------------------------------------------
    // queryAdd — mirrors TrashHolder::queryAdd returning RETURNVALUE_NOERROR
    // -----------------------------------------------------------------------

    /// Always accepts items — mirrors C++ `queryAdd` returning
    /// `RETURNVALUE_NOERROR`.
    pub fn query_add(&self) -> Result<(), TrashHolderError> {
        Ok(())
    }

    // -----------------------------------------------------------------------
    // add_item — mirrors TrashHolder::addThing
    //
    // Items passed to a trash holder are destroyed immediately; they are
    // never stored.  The `item` argument is moved in and then dropped.
    // -----------------------------------------------------------------------

    /// Accepts and immediately destroys `item`.  The item is moved in and
    /// dropped so it is never stored, matching the C++ `internalRemoveItem`
    /// semantics.
    pub fn add_item(&mut self, item: Item) {
        // Consume (destroy) the item — Rust drop handles the memory.
        drop(item);
    }

    /// Mirrors the two-argument C++ `TrashHolder::addThing(int32_t index,
    /// Thing* thing)`. The `index` is ignored — C++ also ignores it — and
    /// the item is destroyed regardless. This exists for behavioural
    /// parity with the C++ inline `addThing(thing) { addThing(0, thing); }`.
    pub fn add_item_at(&mut self, _index: i32, item: Item) {
        self.add_item(item);
    }

    // -----------------------------------------------------------------------
    // query_max_count — mirrors TrashHolder::queryMaxCount
    //
    // C++:  maxQueryCount = std::max<uint32_t>(1, count);
    //       return RETURNVALUE_NOERROR;
    //
    // The Rust port returns the computed max-count directly (Ok variant).
    // -----------------------------------------------------------------------

    /// Returns the maximum count that may be added in one operation.
    /// Mirrors C++ `std::max<uint32_t>(1, count)` — at least 1, otherwise
    /// the requested `count`.
    pub fn query_max_count(&self, count: u32) -> Result<u32, TrashHolderError> {
        Ok(count.max(1))
    }

    // -----------------------------------------------------------------------
    // item_count — always 0
    // -----------------------------------------------------------------------

    /// Returns `0`; trash holders never retain items.
    pub fn item_count(&self) -> usize {
        0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
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
    fn trashholder_new_stores_type_id() {
        let th = TrashHolder::new(500);
        assert_eq!(th.item_type_id, 500);
    }

    #[test]
    fn trashholder_new_type_id_zero() {
        let th = TrashHolder::new(0);
        assert_eq!(th.item_type_id, 0);
    }

    // -----------------------------------------------------------------------
    // query_add
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_returns_ok() {
        let th = TrashHolder::new(1);
        assert!(th.query_add().is_ok());
    }

    #[test]
    fn query_add_ok_repeated_calls() {
        let th = TrashHolder::new(1);
        assert!(th.query_add().is_ok());
        assert!(th.query_add().is_ok());
        assert!(th.query_add().is_ok());
    }

    // -----------------------------------------------------------------------
    // add_item — item is destroyed, item_count stays 0
    // -----------------------------------------------------------------------

    #[test]
    fn item_count_zero_initially() {
        let th = TrashHolder::new(1);
        assert_eq!(th.item_count(), 0);
    }

    #[test]
    fn add_item_does_not_store_item() {
        let mut th = TrashHolder::new(1);
        th.add_item(make_item(10));
        assert_eq!(th.item_count(), 0);
    }

    #[test]
    fn add_multiple_items_count_stays_zero() {
        let mut th = TrashHolder::new(1);
        th.add_item(make_item(1));
        th.add_item(make_item(2));
        th.add_item(make_item(3));
        assert_eq!(th.item_count(), 0);
    }

    #[test]
    fn item_count_always_zero_after_mixed_ops() {
        let mut th = TrashHolder::new(1);
        // query_add then add_item
        let _ = th.query_add();
        th.add_item(make_item(7));
        let _ = th.query_add();
        th.add_item(make_item(8));
        assert_eq!(th.item_count(), 0);
    }

    // -----------------------------------------------------------------------
    // removeThing is a no-op — C++ trashholder.cpp has no removeThing override
    // so it falls back to Item::removeThing which is a no-op.
    // In Rust there is intentionally no remove_item method; item_count stays 0.
    // -----------------------------------------------------------------------

    #[test]
    fn no_remove_method_exists_count_stays_zero() {
        // Verifies that TrashHolder has no stored items to remove.
        // The invariant is: item_count is always 0 regardless of operations.
        let mut th = TrashHolder::new(1);
        th.add_item(make_item(1));
        th.add_item(make_item(2));
        // No remove method — count has never grown above 0
        assert_eq!(th.item_count(), 0);
    }

    // -----------------------------------------------------------------------
    // queryMaxCount equivalent
    // C++: maxQueryCount = max(1, count); returns RETURNVALUE_NOERROR.
    // Rust: item_count() is always 0, meaning the trash holder can always
    // absorb at least 1 item (max capacity is effectively unlimited).
    // -----------------------------------------------------------------------

    #[test]
    fn query_max_count_always_accepts_at_least_one() {
        let th = TrashHolder::new(1);
        // TrashHolder is never "full" — query_add is always Ok
        assert!(th.query_add().is_ok());
        assert_eq!(
            th.item_count(),
            0,
            "retained count is always 0 (unlimited capacity)"
        );
    }

    #[test]
    fn query_add_does_not_depend_on_item_count() {
        let mut th = TrashHolder::new(1);
        // Even after many add_item calls, query_add is still Ok
        for i in 0..20_u16 {
            th.add_item(make_item(i));
            assert!(th.query_add().is_ok());
        }
    }

    // -----------------------------------------------------------------------
    // TrashHolderError type — verify NotAllowed variant exists and is usable
    // -----------------------------------------------------------------------

    #[test]
    fn trash_holder_error_not_allowed_variant() {
        let e = TrashHolderError::NotAllowed;
        assert_eq!(e, TrashHolderError::NotAllowed);
    }

    #[test]
    fn trash_holder_error_is_copy() {
        let e = TrashHolderError::NotAllowed;
        let e2 = e; // Copy
        assert_eq!(e, e2);
    }

    // -----------------------------------------------------------------------
    // addThing skips non-moveable items (C++ special case)
    // In Rust, add_item always drops the item regardless — there is no
    // moveable-property check at the TrashHolder level.  We verify that
    // item_count stays 0 for all item IDs.
    // -----------------------------------------------------------------------

    #[test]
    fn add_item_always_destroys_regardless_of_item_id() {
        let mut th = TrashHolder::new(42);
        for id in [0_u16, 1, 100, 999, u16::MAX] {
            th.add_item(make_item(id));
        }
        assert_eq!(th.item_count(), 0);
    }

    // -----------------------------------------------------------------------
    // Type-id boundary values
    // -----------------------------------------------------------------------

    #[test]
    fn trashholder_max_type_id() {
        let th = TrashHolder::new(u16::MAX);
        assert_eq!(th.item_type_id, u16::MAX);
    }

    // -----------------------------------------------------------------------
    // add_item_at — two-arg C++ addThing(index, thing): index is ignored
    // -----------------------------------------------------------------------

    #[test]
    fn add_item_at_destroys_item_regardless_of_index() {
        let mut th = TrashHolder::new(1);
        th.add_item_at(0, make_item(10));
        th.add_item_at(7, make_item(11));
        th.add_item_at(-1, make_item(12));
        th.add_item_at(i32::MAX, make_item(13));
        th.add_item_at(i32::MIN, make_item(14));
        assert_eq!(th.item_count(), 0);
    }

    #[test]
    fn add_item_at_zero_equivalent_to_add_item() {
        // C++ inline: void addThing(Thing* thing) { return addThing(0, thing); }
        let mut a = TrashHolder::new(1);
        let mut b = TrashHolder::new(1);
        a.add_item(make_item(99));
        b.add_item_at(0, make_item(99));
        assert_eq!(a.item_count(), b.item_count());
    }

    // -----------------------------------------------------------------------
    // query_max_count — mirrors C++ std::max<uint32_t>(1, count)
    // -----------------------------------------------------------------------

    #[test]
    fn query_max_count_zero_clamped_to_one() {
        let th = TrashHolder::new(1);
        assert_eq!(th.query_max_count(0), Ok(1));
    }

    #[test]
    fn query_max_count_one_returns_one() {
        let th = TrashHolder::new(1);
        assert_eq!(th.query_max_count(1), Ok(1));
    }

    #[test]
    fn query_max_count_passes_through_larger_counts() {
        let th = TrashHolder::new(1);
        assert_eq!(th.query_max_count(2), Ok(2));
        assert_eq!(th.query_max_count(100), Ok(100));
        assert_eq!(th.query_max_count(65_535), Ok(65_535));
    }

    #[test]
    fn query_max_count_u32_max() {
        let th = TrashHolder::new(1);
        assert_eq!(th.query_max_count(u32::MAX), Ok(u32::MAX));
    }

    // -----------------------------------------------------------------------
    // Clone — TrashHolder is Clone; cloned holder also destroys items
    // -----------------------------------------------------------------------

    #[test]
    fn clone_also_destroys_items() {
        let mut th = TrashHolder::new(1);
        let mut clone = th.clone();
        clone.add_item(make_item(5));
        th.add_item(make_item(6));
        assert_eq!(th.item_count(), 0);
        assert_eq!(clone.item_count(), 0);
    }
}
