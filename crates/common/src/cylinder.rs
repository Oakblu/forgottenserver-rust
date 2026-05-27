// Copyright 2023 The Forgotten Server Authors. All rights reserved.
// Use of this source code is governed by the GPL-2.0 License that can be found in the LICENSE file.

//! Cylinder trait — cross-crate dispatch contract for "receiver" things like
//! tiles and containers.
//!
//! Mirrors the C++ `Cylinder` aspect of `Thing` from `thing.h` (in the C++
//! code, `Thing` itself owns the cylinder methods; in Rust we lift them into
//! a separate trait to enable trait-object dispatch from lower crates without
//! pulling in higher-crate concrete types).
//!
//! Concrete implementations live in higher crates (`map::Tile`,
//! `items::Container`, `items::Inbox`, `items::StoreInbox`,
//! `items::DepotChest`, `items::DepotLocker`). Callers in `game`/`network`/
//! `server` dispatch through `&dyn Cylinder` to avoid hard cross-crate
//! dependencies.

use crate::enums::ReturnValue;
use crate::thing::{ReceiverLink, Thing};

/// Cross-crate receiver contract. Implemented by every "container-like" thing
/// (tiles, real containers, inboxes, depot chests/lockers).
///
/// All defaults are conservative: they refuse adds, report empty contents,
/// and treat the receiver as a non-cylinder. Concrete types override the
/// methods relevant to their behaviour.
pub trait Cylinder: Thing {
    // -----------------------------------------------------------------------
    // Read-only queries
    // -----------------------------------------------------------------------

    /// Returns the index for the first valid child slot.
    ///
    /// Mirrors C++ `virtual size_t getFirstIndex() const`. Defaults to `0`,
    /// matching the C++ base.
    fn cylinder_first_index(&self) -> usize {
        0
    }

    /// Returns the index for the last valid child slot.
    ///
    /// Mirrors C++ `virtual size_t getLastIndex() const`. Defaults to `0`,
    /// matching the C++ base.
    fn cylinder_last_index(&self) -> usize {
        0
    }

    /// Returns the count of items with `item_id` and (optionally) `sub_type`.
    ///
    /// Mirrors C++ `virtual uint32_t getItemTypeCount(uint16_t, int32_t = -1) const`.
    /// `sub_type == -1` means "any sub-type" (charges/fluid ignored).
    fn cylinder_item_type_count(&self, _item_id: u16, _sub_type: i32) -> u32 {
        0
    }

    /// Returns the index of `child` inside this cylinder, or `None` if absent.
    ///
    /// Mirrors C++ `virtual int32_t getThingIndex(const Thing*) const` which
    /// returns `-1` for "not found"; we use `Option<i32>` for clarity.
    fn cylinder_thing_index(&self, _child: &dyn Thing) -> Option<i32> {
        None
    }

    // -----------------------------------------------------------------------
    // Receiver-side queries (queryAdd / queryRemove / queryMaxCount / queryDestination)
    // -----------------------------------------------------------------------

    /// Whether `thing` can be added at `index` with the given `count` and
    /// `flags`. Defaults to `NotPossible` (matches C++ `Thing::queryAdd`).
    ///
    /// Flags are an OR'd bitmask of `ReceiverFlag::bits()`.
    fn cylinder_query_add(
        &self,
        _index: i32,
        _thing: &dyn Thing,
        _count: u32,
        _flags: u32,
    ) -> ReturnValue {
        ReturnValue::NotPossible
    }

    /// How many of `thing` can be accepted at `index`. Returns
    /// `(ReturnValue, max_count_acceptable)`.
    ///
    /// Mirrors C++ `virtual ReturnValue queryMaxCount(int32_t, const Thing&,
    /// uint32_t, uint32_t& maxQueryCount, uint32_t flags) const`. The out
    /// parameter is folded into the return tuple.
    fn cylinder_query_max_count(
        &self,
        _index: i32,
        _thing: &dyn Thing,
        _count: u32,
        _flags: u32,
    ) -> (ReturnValue, u32) {
        (ReturnValue::NotPossible, 0)
    }

    /// Whether `thing` can be removed from this cylinder. Defaults to
    /// `NotPossible` (matches C++).
    fn cylinder_query_remove(&self, _thing: &dyn Thing, _count: u32, _flags: u32) -> ReturnValue {
        ReturnValue::NotPossible
    }

    // -----------------------------------------------------------------------
    // Notifications (postAddNotification / postRemoveNotification)
    //
    // The full C++ signature includes a `Thing*` pointer to the moved object,
    // an int32_t index, and a `ReceiverLink_t` (which we model with
    // `ReceiverLink`). The Rust default implementations are no-ops; concrete
    // types override to wire spectator updates and decay handling.
    // -----------------------------------------------------------------------

    /// Post-add notification hook. Default: no-op.
    ///
    /// Mirrors C++ `virtual void postAddNotification(Thing*, const Thing*,
    /// int32_t, ReceiverLink_t = LINK_OWNER)`.
    fn cylinder_post_add(
        &self,
        _thing: &dyn Thing,
        _old_parent: Option<&dyn Thing>,
        _index: i32,
        _link: ReceiverLink,
    ) {
    }

    /// Post-remove notification hook. Default: no-op.
    ///
    /// Mirrors C++ `virtual void postRemoveNotification(Thing*, const Thing*,
    /// int32_t, ReceiverLink_t = LINK_OWNER)`.
    fn cylinder_post_remove(
        &self,
        _thing: &dyn Thing,
        _new_parent: Option<&dyn Thing>,
        _index: i32,
        _link: ReceiverLink,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thing::{DefaultThing, ReceiverLink};

    /// Mark `DefaultThing` as a cylinder using all defaults, to verify the
    /// default-impl behaviour matches the C++ base.
    impl Cylinder for DefaultThing {}

    #[test]
    fn test_default_cylinder_first_index_returns_zero() {
        let c = DefaultThing;
        assert_eq!(c.cylinder_first_index(), 0);
    }

    #[test]
    fn test_default_cylinder_last_index_returns_zero() {
        let c = DefaultThing;
        assert_eq!(c.cylinder_last_index(), 0);
    }

    #[test]
    fn test_default_cylinder_item_type_count_returns_zero() {
        let c = DefaultThing;
        assert_eq!(c.cylinder_item_type_count(2160, -1), 0);
        assert_eq!(c.cylinder_item_type_count(2160, 7), 0);
        assert_eq!(c.cylinder_item_type_count(u16::MAX, i32::MAX), 0);
    }

    #[test]
    fn test_default_cylinder_thing_index_returns_none() {
        let c = DefaultThing;
        let child = DefaultThing;
        assert_eq!(c.cylinder_thing_index(&child), None);
    }

    #[test]
    fn test_default_cylinder_query_add_returns_not_possible() {
        let c = DefaultThing;
        let t = DefaultThing;
        assert_eq!(c.cylinder_query_add(0, &t, 1, 0), ReturnValue::NotPossible);
        assert_eq!(c.cylinder_query_add(-1, &t, 0, 0), ReturnValue::NotPossible);
        assert_eq!(
            c.cylinder_query_add(99, &t, 100, u32::MAX),
            ReturnValue::NotPossible
        );
    }

    #[test]
    fn test_default_cylinder_query_max_count_returns_not_possible_zero() {
        let c = DefaultThing;
        let t = DefaultThing;
        assert_eq!(
            c.cylinder_query_max_count(0, &t, 1, 0),
            (ReturnValue::NotPossible, 0)
        );
    }

    #[test]
    fn test_default_cylinder_query_remove_returns_not_possible() {
        let c = DefaultThing;
        let t = DefaultThing;
        assert_eq!(c.cylinder_query_remove(&t, 1, 0), ReturnValue::NotPossible);
    }

    #[test]
    fn test_default_cylinder_post_add_is_noop() {
        // No-op smoke test: must not panic and must not require state.
        let c = DefaultThing;
        let t = DefaultThing;
        c.cylinder_post_add(&t, None, 0, ReceiverLink::Owner);
        c.cylinder_post_add(&t, Some(&t), 5, ReceiverLink::TopParent);
    }

    #[test]
    fn test_default_cylinder_post_remove_is_noop() {
        let c = DefaultThing;
        let t = DefaultThing;
        c.cylinder_post_remove(&t, None, 0, ReceiverLink::Owner);
        c.cylinder_post_remove(&t, Some(&t), -1, ReceiverLink::Near);
    }

    // -----------------------------------------------------------------------
    // A non-trivial cylinder that overrides every method to confirm dispatch
    // honours the overrides instead of the trait defaults.
    // -----------------------------------------------------------------------

    struct AcceptingCylinder {
        capacity: u32,
        items: Vec<(u16, i32)>, // (item_id, sub_type)
    }

    impl Thing for AcceptingCylinder {
        fn is_container(&self) -> bool {
            true
        }
        fn is_removed(&self) -> bool {
            false
        }
    }

    impl Cylinder for AcceptingCylinder {
        fn cylinder_first_index(&self) -> usize {
            0
        }
        fn cylinder_last_index(&self) -> usize {
            self.items.len()
        }
        fn cylinder_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
            self.items
                .iter()
                .filter(|(id, st)| *id == item_id && (sub_type == -1 || *st == sub_type))
                .count() as u32
        }
        fn cylinder_thing_index(&self, _child: &dyn Thing) -> Option<i32> {
            // Toy: always reports the first slot.
            if self.items.is_empty() {
                None
            } else {
                Some(0)
            }
        }
        fn cylinder_query_add(
            &self,
            _index: i32,
            _thing: &dyn Thing,
            count: u32,
            _flags: u32,
        ) -> ReturnValue {
            if (self.items.len() as u32 + count) <= self.capacity {
                ReturnValue::NoError
            } else {
                ReturnValue::ContainerNotEnoughRoom
            }
        }
        fn cylinder_query_max_count(
            &self,
            _index: i32,
            _thing: &dyn Thing,
            count: u32,
            _flags: u32,
        ) -> (ReturnValue, u32) {
            let free = self.capacity.saturating_sub(self.items.len() as u32);
            let accepted = count.min(free);
            (ReturnValue::NoError, accepted)
        }
        fn cylinder_query_remove(
            &self,
            _thing: &dyn Thing,
            _count: u32,
            _flags: u32,
        ) -> ReturnValue {
            if self.items.is_empty() {
                ReturnValue::NotPossible
            } else {
                ReturnValue::NoError
            }
        }
    }

    #[test]
    fn test_accepting_cylinder_last_index_reflects_size() {
        let c = AcceptingCylinder {
            capacity: 8,
            items: vec![(2160, -1), (2152, -1)],
        };
        assert_eq!(c.cylinder_first_index(), 0);
        assert_eq!(c.cylinder_last_index(), 2);
    }

    #[test]
    fn test_accepting_cylinder_item_type_count_filters_by_subtype() {
        let c = AcceptingCylinder {
            capacity: 8,
            items: vec![(2160, -1), (2160, 7), (2152, -1)],
        };
        assert_eq!(c.cylinder_item_type_count(2160, -1), 2); // any sub-type
        assert_eq!(c.cylinder_item_type_count(2160, 7), 1); // exact sub-type
        assert_eq!(c.cylinder_item_type_count(99, -1), 0); // unknown id
    }

    #[test]
    fn test_accepting_cylinder_thing_index_reports_first_slot_when_nonempty() {
        let empty = AcceptingCylinder {
            capacity: 8,
            items: vec![],
        };
        let nonempty = AcceptingCylinder {
            capacity: 8,
            items: vec![(2160, -1)],
        };
        let t = DefaultThing;
        assert_eq!(empty.cylinder_thing_index(&t), None);
        assert_eq!(nonempty.cylinder_thing_index(&t), Some(0));
    }

    #[test]
    fn test_accepting_cylinder_query_add_succeeds_within_capacity() {
        let c = AcceptingCylinder {
            capacity: 8,
            items: vec![(2160, -1)],
        };
        let t = DefaultThing;
        assert_eq!(c.cylinder_query_add(0, &t, 1, 0), ReturnValue::NoError);
        assert_eq!(c.cylinder_query_add(0, &t, 7, 0), ReturnValue::NoError);
    }

    #[test]
    fn test_accepting_cylinder_query_add_refuses_overflow() {
        let c = AcceptingCylinder {
            capacity: 8,
            items: vec![(2160, -1); 8],
        };
        let t = DefaultThing;
        assert_eq!(
            c.cylinder_query_add(0, &t, 1, 0),
            ReturnValue::ContainerNotEnoughRoom
        );
    }

    #[test]
    fn test_accepting_cylinder_query_max_count_reports_free_slots() {
        let c = AcceptingCylinder {
            capacity: 8,
            items: vec![(2160, -1); 5],
        };
        let t = DefaultThing;
        // Want 10, have 3 free → accepts 3.
        assert_eq!(
            c.cylinder_query_max_count(0, &t, 10, 0),
            (ReturnValue::NoError, 3)
        );
        // Want 1, have 3 free → accepts 1.
        assert_eq!(
            c.cylinder_query_max_count(0, &t, 1, 0),
            (ReturnValue::NoError, 1)
        );
        // Want 0, accept 0.
        assert_eq!(
            c.cylinder_query_max_count(0, &t, 0, 0),
            (ReturnValue::NoError, 0)
        );
    }

    #[test]
    fn test_accepting_cylinder_query_remove_refuses_when_empty() {
        let c = AcceptingCylinder {
            capacity: 8,
            items: vec![],
        };
        let t = DefaultThing;
        assert_eq!(c.cylinder_query_remove(&t, 1, 0), ReturnValue::NotPossible);
    }

    #[test]
    fn test_accepting_cylinder_query_remove_succeeds_when_nonempty() {
        let c = AcceptingCylinder {
            capacity: 8,
            items: vec![(2160, -1)],
        };
        let t = DefaultThing;
        assert_eq!(c.cylinder_query_remove(&t, 1, 0), ReturnValue::NoError);
    }

    #[test]
    fn test_cylinder_as_dyn_trait_object_dispatch() {
        // A Vec<&dyn Cylinder> dispatches through the trait, confirming the
        // Rust trait-object machinery works for cross-crate use cases.
        let default = DefaultThing;
        let accepting = AcceptingCylinder {
            capacity: 4,
            items: vec![],
        };
        let cylinders: Vec<&dyn Cylinder> = vec![&default, &accepting];
        // Default refuses; accepting accepts.
        let t = DefaultThing;
        assert_eq!(
            cylinders[0].cylinder_query_add(0, &t, 1, 0),
            ReturnValue::NotPossible
        );
        assert_eq!(
            cylinders[1].cylinder_query_add(0, &t, 1, 0),
            ReturnValue::NoError
        );
    }
}
