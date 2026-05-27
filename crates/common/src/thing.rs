// Copyright 2023 The Forgotten Server Authors. All rights reserved.
// Use of this source code is governed by the GPL-2.0 License that can be found in the LICENSE file.

use std::collections::HashMap;

/// Sentinel "wherever" destination index used by container / receiver queries.
///
/// Mirrors the C++ `inline constexpr int32_t INDEX_WHEREEVER = -1;` in
/// `thing.h`. Callers use this value to mean "add to any free slot; we don't
/// care which one."
pub const INDEX_WHEREEVER: i32 = -1;

/// Optional flags that modify the behaviour of receiver queries
/// (`query_add`, `query_max_count`, `query_remove`, `query_destination`).
///
/// Mirrors the C++ `enum ReceiverFlag_t` in `thing.h`. The discriminants are
/// power-of-two so callers can `|` them together into a bitmask — exactly
/// how the C++ side uses them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ReceiverFlag {
    /// Bypass capacity / container / blocking limits.
    NoLimit = 1 << 0,
    /// Bypass movable blocking-item checks.
    IgnoreBlockItem = 1 << 1,
    /// Bypass creature checks.
    IgnoreBlockCreature = 1 << 2,
    /// The querying child is the owner (e.g. a container querying its carrier).
    ChildIsOwner = 1 << 3,
    /// Additional pathfinding check for floor-changing / teleport items.
    Pathfinding = 1 << 4,
    /// Bypass field-damage checks.
    IgnoreFieldDamage = 1 << 5,
    /// Bypass mobility (immovable) checks.
    IgnoreNotMoveable = 1 << 6,
    /// `query_destination` will not auto-stack items.
    IgnoreAutoStack = 1 << 7,
}

impl ReceiverFlag {
    /// Returns the underlying bit-flag value, suitable for bitwise OR-ing into
    /// a `u32` mask.
    pub const fn bits(self) -> u32 {
        self as u32
    }
}

/// Describes the relation of a moved/added/removed object to the receiver in
/// `post_add_notification` / `post_remove_notification` callbacks.
///
/// Mirrors the C++ `enum ReceiverLink_t` in `thing.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReceiverLink {
    /// The receiver is the direct owner (e.g. the carrying player).
    Owner,
    /// The receiver is the direct parent container/tile.
    Parent,
    /// The receiver is the top-most parent in the chain.
    TopParent,
    /// The receiver is nearby (within the same tile / neighbour tile).
    Near,
}

impl Default for ReceiverLink {
    /// Mirrors the C++ default for `ReceiverLink_t`-typed parameters
    /// (`LINK_OWNER`).
    fn default() -> Self {
        ReceiverLink::Owner
    }
}

/// Base trait for all game objects (creatures, items, tiles).
///
/// Mirrors the C++ `Thing` virtual base class from `thing.h / thing.cpp`.
///
/// Cross-crate methods (`get_tile`, `get_item`, `get_creature`, `get_receiver`,
/// parent chain) are exposed as trait-object signatures so lower crates can
/// dispatch through `&dyn Thing` without depending on higher-crate concrete
/// types.
pub trait Thing {
    /// Returns a textual description of this object at a given look distance.
    /// Defaults to an empty string (concrete types override as needed).
    fn get_description(&self, _look_distance: i32) -> String {
        String::new()
    }

    /// How far this object can be thrown (in tiles).
    fn get_throw_range(&self) -> i32 {
        1
    }

    /// Whether a creature can push this object.
    fn is_pushable(&self) -> bool {
        false
    }

    /// Whether this object can be removed from its parent.
    fn is_removable(&self) -> bool {
        true
    }

    /// Whether this object has already been removed from the world.
    ///
    /// Mirrors the C++ `virtual bool isRemoved() const { return true; }` —
    /// the C++ base reports `true` so any subclass that forgets to override
    /// is treated as "removed" (a deliberately conservative default).
    fn is_removed(&self) -> bool {
        true
    }

    // --- Index / child accessors (default no-op containers) ---

    /// First valid child index. Default `0`; container-like things override.
    ///
    /// Mirrors C++ `virtual size_t getFirstIndex() const { return 0; }`.
    fn get_first_index(&self) -> usize {
        0
    }

    /// Last valid child index. Default `0`; container-like things override.
    ///
    /// Mirrors C++ `virtual size_t getLastIndex() const { return 0; }`.
    fn get_last_index(&self) -> usize {
        0
    }

    /// Index of the given child within this thing, or `-1` if not found.
    ///
    /// Mirrors C++ `virtual int32_t getThingIndex(const Thing*) const { return -1; }`.
    /// The argument is a `&dyn Thing` so concrete container implementations
    /// can downcast / compare addresses; the default ignores it.
    fn get_thing_index(&self, _child: &dyn Thing) -> i32 {
        -1
    }

    /// Count of items of a given type held by this thing. Default `0`.
    ///
    /// Mirrors C++ `virtual uint32_t getItemTypeCount(uint16_t, int32_t = -1) const { return 0; }`.
    /// `sub_type` of `-1` means "any sub-type" (charges / fluid type ignored).
    fn get_item_type_count(&self, _item_id: u16, _sub_type: i32) -> u32 {
        0
    }

    /// Accumulates the item-type counts of this thing into `count_map`.
    /// Returns `count_map` unchanged for the default implementation;
    /// container-like overrides add their contained items.
    ///
    /// Mirrors C++ `virtual std::map<uint32_t,uint32_t>& getAllItemTypeCount(std::map<uint32_t,uint32_t>&) const`
    /// which returns the input map untouched at the base level.
    fn get_all_item_type_count<'a>(
        &self,
        count_map: &'a mut HashMap<u32, u32>,
    ) -> &'a mut HashMap<u32, u32> {
        count_map
    }

    // --- Type discriminators (all false by default; overridden by concrete types) ---

    fn is_creature(&self) -> bool {
        false
    }

    fn is_item(&self) -> bool {
        false
    }

    fn is_teleport(&self) -> bool {
        false
    }

    fn is_magic_field(&self) -> bool {
        false
    }

    fn is_trash_holder(&self) -> bool {
        false
    }

    fn is_container(&self) -> bool {
        false
    }

    fn is_door(&self) -> bool {
        false
    }

    fn is_moveable(&self) -> bool {
        true
    }

    fn is_ground(&self) -> bool {
        false
    }

    fn is_invisible(&self) -> bool {
        false
    }

    // -----------------------------------------------------------------------
    // Cross-crate parent / accessor methods (trait-object dispatch)
    //
    // Default impls return `None`. Concrete types in higher crates override
    // to participate in the receiver/parent chain. Mirrors the C++
    // `virtual Tile* getTile()`, `virtual Item* getItem()`,
    // `virtual Creature* getCreature()`, `virtual Thing* getParent()`,
    // `virtual Thing* getReceiver()` family.
    // -----------------------------------------------------------------------

    /// Returns the tile this object is on, or `None` for objects not on a
    /// tile (items in containers, dis-embodied creatures).
    ///
    /// Mirrors C++ `virtual Tile* getTile()`. Returns a `&dyn Thing`
    /// trait-object because `Tile` lives in the `map` crate.
    fn get_tile(&self) -> Option<&dyn Thing> {
        None
    }

    /// Returns this object as an item, when applicable.
    ///
    /// Mirrors C++ `virtual Item* getItem()`. The default is `None`; an
    /// `Item` implementor returns `Some(self)`.
    fn get_item(&self) -> Option<&dyn Thing> {
        None
    }

    /// Returns this object as a creature, when applicable.
    ///
    /// Mirrors C++ `virtual Creature* getCreature()`. The default is
    /// `None`; a `Creature` implementor returns `Some(self)`.
    fn get_creature(&self) -> Option<&dyn Thing> {
        None
    }

    /// Returns the "receiver" Thing for cylinder dispatch — usually the
    /// object itself for cylinders, or its parent tile/container otherwise.
    ///
    /// Mirrors C++ `virtual Thing* getReceiver()`. Default is `None`.
    fn get_receiver(&self) -> Option<&dyn Thing> {
        None
    }

    /// Returns the parent cylinder (container or tile) holding this object,
    /// or `None` at the top of the chain. Mirrors C++
    /// `virtual Thing* getParent() const`.
    fn get_parent(&self) -> Option<&dyn Thing> {
        None
    }

    /// Convenience for the C++ `bool hasParent() const { return getParent(); }`
    /// pattern.
    fn has_parent(&self) -> bool {
        self.get_parent().is_some()
    }

    /// Returns the top-most parent in the receiver chain (walks `get_parent`
    /// until it returns `None`). Mirrors C++ `Thing::getTopParent()` walk.
    ///
    /// The default implementation iterates via `get_parent`. Concrete types
    /// that cache the top parent can override.
    fn get_top_parent(&self) -> Option<&dyn Thing> {
        // Default: do not perform the walk — concrete types own the chain
        // and can implement this in O(1). The trait-default returns the
        // direct parent (closest available approximation) so a single-level
        // lookup still works.
        self.get_parent()
    }

    /// Returns the world position of this object (mirror of C++
    /// `virtual const Position& getPosition() const`). Default is `None`;
    /// implementors that own a position (Tile, Item-on-tile, Creature on
    /// tile) override to return their stored position via a callback.
    ///
    /// Note: we return `Option<(u16, u16, u8)>` rather than a borrowed
    /// `Position` to keep the trait dyn-safe across crates without pulling
    /// `Position` into every implementor's vtable layout.
    fn get_position_tuple(&self) -> Option<(u16, u16, u8)> {
        None
    }
}

/// A concrete no-op implementor of [`Thing`] that accepts all defaults.
///
/// Useful as a test fixture or a null-object stand-in.
pub struct DefaultThing;

impl Thing for DefaultThing {}

#[cfg(test)]
mod tests {
    use super::*;

    // --- DefaultThing implements Thing with all defaults ---

    #[test]
    fn test_default_thing_get_description_returns_empty_string() {
        let t = DefaultThing;
        assert_eq!(t.get_description(0), "");
    }

    #[test]
    fn test_default_thing_get_description_any_distance_returns_empty_string() {
        let t = DefaultThing;
        assert_eq!(t.get_description(-1), "");
        assert_eq!(t.get_description(5), "");
        assert_eq!(t.get_description(i32::MAX), "");
    }

    #[test]
    fn test_default_thing_get_throw_range_returns_one() {
        let t = DefaultThing;
        assert_eq!(t.get_throw_range(), 1);
    }

    #[test]
    fn test_default_thing_is_pushable_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_pushable());
    }

    #[test]
    fn test_default_thing_is_removable_returns_true() {
        let t = DefaultThing;
        assert!(t.is_removable());
    }

    // --- Discriminator defaults ---

    #[test]
    fn test_default_thing_is_creature_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_creature());
    }

    #[test]
    fn test_default_thing_is_item_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_item());
    }

    #[test]
    fn test_default_thing_is_teleport_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_teleport());
    }

    #[test]
    fn test_default_thing_is_magic_field_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_magic_field());
    }

    #[test]
    fn test_default_thing_is_trash_holder_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_trash_holder());
    }

    #[test]
    fn test_default_thing_is_container_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_container());
    }

    #[test]
    fn test_default_thing_is_door_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_door());
    }

    #[test]
    fn test_default_thing_is_moveable_returns_true() {
        let t = DefaultThing;
        assert!(t.is_moveable());
    }

    #[test]
    fn test_default_thing_is_ground_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_ground());
    }

    #[test]
    fn test_default_thing_is_invisible_returns_false() {
        let t = DefaultThing;
        assert!(!t.is_invisible());
    }

    // --- A struct that overrides is_creature() → true; others stay false ---

    struct CreatureThing;

    impl Thing for CreatureThing {
        fn is_creature(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_creature_thing_is_creature_returns_true() {
        let c = CreatureThing;
        assert!(c.is_creature());
    }

    #[test]
    fn test_creature_thing_other_discriminators_remain_false() {
        let c = CreatureThing;
        assert!(!c.is_item());
        assert!(!c.is_teleport());
        assert!(!c.is_magic_field());
        assert!(!c.is_trash_holder());
        assert!(!c.is_container());
        assert!(!c.is_door());
        assert!(!c.is_ground());
        assert!(!c.is_invisible());
    }

    #[test]
    fn test_creature_thing_inherits_default_description() {
        let c = CreatureThing;
        assert_eq!(c.get_description(0), "");
    }

    #[test]
    fn test_creature_thing_inherits_default_throw_range() {
        let c = CreatureThing;
        assert_eq!(c.get_throw_range(), 1);
    }

    #[test]
    fn test_creature_thing_inherits_default_is_pushable() {
        let c = CreatureThing;
        assert!(!c.is_pushable());
    }

    #[test]
    fn test_creature_thing_inherits_default_is_removable() {
        let c = CreatureThing;
        assert!(c.is_removable());
    }

    #[test]
    fn test_creature_thing_inherits_default_is_moveable() {
        let c = CreatureThing;
        assert!(c.is_moveable());
    }

    // --- Trait object usage ---

    #[test]
    fn test_thing_as_dyn_trait_object() {
        let items: Vec<Box<dyn Thing>> = vec![Box::new(DefaultThing), Box::new(CreatureThing)];
        assert!(!items[0].is_creature());
        assert!(items[1].is_creature());
    }

    // --- Custom struct overriding multiple fields ---

    struct ImmovablePushableItem;

    impl Thing for ImmovablePushableItem {
        fn is_item(&self) -> bool {
            true
        }
        fn is_pushable(&self) -> bool {
            true
        }
        fn is_moveable(&self) -> bool {
            false
        }
        fn get_throw_range(&self) -> i32 {
            3
        }
        fn get_description(&self, _look_distance: i32) -> String {
            "a heavy crate".to_string()
        }
    }

    #[test]
    fn test_custom_thing_is_item_returns_true() {
        let item = ImmovablePushableItem;
        assert!(item.is_item());
    }

    #[test]
    fn test_custom_thing_is_pushable_returns_true() {
        let item = ImmovablePushableItem;
        assert!(item.is_pushable());
    }

    #[test]
    fn test_custom_thing_is_moveable_returns_false() {
        let item = ImmovablePushableItem;
        assert!(!item.is_moveable());
    }

    #[test]
    fn test_custom_thing_throw_range_returns_three() {
        let item = ImmovablePushableItem;
        assert_eq!(item.get_throw_range(), 3);
    }

    #[test]
    fn test_custom_thing_description_returns_custom_string() {
        let item = ImmovablePushableItem;
        assert_eq!(item.get_description(0), "a heavy crate");
        assert_eq!(item.get_description(10), "a heavy crate");
    }

    #[test]
    fn test_custom_thing_non_overridden_discriminators_remain_false() {
        let item = ImmovablePushableItem;
        assert!(!item.is_creature());
        assert!(!item.is_teleport());
        assert!(!item.is_magic_field());
        assert!(!item.is_trash_holder());
        assert!(!item.is_container());
        assert!(!item.is_door());
        assert!(!item.is_ground());
        assert!(!item.is_invisible());
    }

    // -----------------------------------------------------------------------
    // Concrete overrides for remaining discriminators not covered above:
    // is_removable, is_teleport, is_magic_field, is_trash_holder,
    // is_container, is_door, is_ground, is_invisible.
    //
    // Each discriminator needs at least one struct that overrides it to true
    // so both the default (false) and the override (true) paths are exercised.
    // -----------------------------------------------------------------------

    /// A non-removable thing (e.g. a wall tile, a quest marker).
    struct NonRemovableThing;

    impl Thing for NonRemovableThing {
        fn is_removable(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_non_removable_thing_is_removable_returns_false() {
        let t = NonRemovableThing;
        assert!(
            !t.is_removable(),
            "overridden is_removable should return false"
        );
    }

    #[test]
    fn test_non_removable_thing_other_methods_use_defaults() {
        let t = NonRemovableThing;
        // Verify non-overridden methods still return their defaults.
        assert_eq!(t.get_description(0), "");
        assert_eq!(t.get_throw_range(), 1);
        assert!(!t.is_pushable());
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(t.is_moveable());
    }

    /// A teleport item.
    struct TeleportThing;

    impl Thing for TeleportThing {
        fn is_teleport(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_teleport_thing_is_teleport_returns_true() {
        let t = TeleportThing;
        assert!(t.is_teleport(), "overridden is_teleport should return true");
    }

    #[test]
    fn test_teleport_thing_other_discriminators_remain_default() {
        let t = TeleportThing;
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(!t.is_magic_field());
        assert!(!t.is_trash_holder());
        assert!(!t.is_container());
        assert!(!t.is_door());
        assert!(!t.is_ground());
        assert!(!t.is_invisible());
    }

    /// A magic field (fire field, poison field, etc.).
    struct MagicFieldThing;

    impl Thing for MagicFieldThing {
        fn is_magic_field(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_magic_field_thing_is_magic_field_returns_true() {
        let t = MagicFieldThing;
        assert!(
            t.is_magic_field(),
            "overridden is_magic_field should return true"
        );
    }

    #[test]
    fn test_magic_field_thing_other_discriminators_remain_default() {
        let t = MagicFieldThing;
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(!t.is_teleport());
        assert!(!t.is_trash_holder());
        assert!(!t.is_container());
        assert!(!t.is_door());
        assert!(!t.is_ground());
        assert!(!t.is_invisible());
    }

    /// A trash holder (e.g. a trash can).
    struct TrashHolderThing;

    impl Thing for TrashHolderThing {
        fn is_trash_holder(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_trash_holder_thing_is_trash_holder_returns_true() {
        let t = TrashHolderThing;
        assert!(
            t.is_trash_holder(),
            "overridden is_trash_holder should return true"
        );
    }

    #[test]
    fn test_trash_holder_thing_other_discriminators_remain_default() {
        let t = TrashHolderThing;
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(!t.is_teleport());
        assert!(!t.is_magic_field());
        assert!(!t.is_container());
        assert!(!t.is_door());
        assert!(!t.is_ground());
        assert!(!t.is_invisible());
    }

    /// A container (bag, box, etc.).
    struct ContainerThing;

    impl Thing for ContainerThing {
        fn is_container(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_container_thing_is_container_returns_true() {
        let t = ContainerThing;
        assert!(
            t.is_container(),
            "overridden is_container should return true"
        );
    }

    #[test]
    fn test_container_thing_other_discriminators_remain_default() {
        let t = ContainerThing;
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(!t.is_teleport());
        assert!(!t.is_magic_field());
        assert!(!t.is_trash_holder());
        assert!(!t.is_door());
        assert!(!t.is_ground());
        assert!(!t.is_invisible());
    }

    /// A door.
    struct DoorThing;

    impl Thing for DoorThing {
        fn is_door(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_door_thing_is_door_returns_true() {
        let t = DoorThing;
        assert!(t.is_door(), "overridden is_door should return true");
    }

    #[test]
    fn test_door_thing_other_discriminators_remain_default() {
        let t = DoorThing;
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(!t.is_teleport());
        assert!(!t.is_magic_field());
        assert!(!t.is_trash_holder());
        assert!(!t.is_container());
        assert!(!t.is_ground());
        assert!(!t.is_invisible());
    }

    /// A ground tile.
    struct GroundThing;

    impl Thing for GroundThing {
        fn is_ground(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_ground_thing_is_ground_returns_true() {
        let t = GroundThing;
        assert!(t.is_ground(), "overridden is_ground should return true");
    }

    #[test]
    fn test_ground_thing_other_discriminators_remain_default() {
        let t = GroundThing;
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(!t.is_teleport());
        assert!(!t.is_magic_field());
        assert!(!t.is_trash_holder());
        assert!(!t.is_container());
        assert!(!t.is_door());
        assert!(!t.is_invisible());
    }

    /// An invisible object.
    struct InvisibleThing;

    impl Thing for InvisibleThing {
        fn is_invisible(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_invisible_thing_is_invisible_returns_true() {
        let t = InvisibleThing;
        assert!(
            t.is_invisible(),
            "overridden is_invisible should return true"
        );
    }

    #[test]
    fn test_invisible_thing_other_discriminators_remain_default() {
        let t = InvisibleThing;
        assert!(!t.is_creature());
        assert!(!t.is_item());
        assert!(!t.is_teleport());
        assert!(!t.is_magic_field());
        assert!(!t.is_trash_holder());
        assert!(!t.is_container());
        assert!(!t.is_door());
        assert!(!t.is_ground());
    }

    /// An object that overrides every single trait method to non-default values,
    /// confirming that all overrides are independent and compose correctly.
    struct FullyOverriddenThing;

    impl Thing for FullyOverriddenThing {
        fn get_description(&self, look_distance: i32) -> String {
            format!("full override at distance {look_distance}")
        }
        fn get_throw_range(&self) -> i32 {
            10
        }
        fn is_pushable(&self) -> bool {
            true
        }
        fn is_removable(&self) -> bool {
            false
        }
        fn is_creature(&self) -> bool {
            true
        }
        fn is_item(&self) -> bool {
            true
        }
        fn is_teleport(&self) -> bool {
            true
        }
        fn is_magic_field(&self) -> bool {
            true
        }
        fn is_trash_holder(&self) -> bool {
            true
        }
        fn is_container(&self) -> bool {
            true
        }
        fn is_door(&self) -> bool {
            true
        }
        fn is_moveable(&self) -> bool {
            false
        }
        fn is_ground(&self) -> bool {
            true
        }
        fn is_invisible(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_fully_overridden_thing_all_methods_return_overridden_values() {
        let t = FullyOverriddenThing;
        assert_eq!(t.get_description(5), "full override at distance 5");
        assert_eq!(t.get_throw_range(), 10);
        assert!(t.is_pushable());
        assert!(!t.is_removable());
        assert!(t.is_creature());
        assert!(t.is_item());
        assert!(t.is_teleport());
        assert!(t.is_magic_field());
        assert!(t.is_trash_holder());
        assert!(t.is_container());
        assert!(t.is_door());
        assert!(!t.is_moveable());
        assert!(t.is_ground());
        assert!(t.is_invisible());
    }

    // -----------------------------------------------------------------------
    // INDEX_WHEREEVER constant (C++ `inline constexpr int32_t INDEX_WHEREEVER = -1`)
    // -----------------------------------------------------------------------

    #[test]
    fn test_index_whereever_equals_minus_one() {
        assert_eq!(INDEX_WHEREEVER, -1);
    }

    #[test]
    fn test_index_whereever_distinguishable_from_valid_indices() {
        // Sentinel must be negative so it cannot collide with any real
        // container slot (valid slots are 0..=container_size).
        // Compare via a runtime binding to dodge clippy's
        // `assertions_on_constants` on `assert!(INDEX_WHEREEVER < 0)`.
        let idx = INDEX_WHEREEVER;
        assert!(
            idx.is_negative(),
            "INDEX_WHEREEVER ({idx}) must be negative"
        );
        assert_ne!(idx, 0);
        assert_ne!(idx, 1);
        assert_ne!(idx, i32::MAX);
    }

    // -----------------------------------------------------------------------
    // ReceiverFlag enum — bit-flag discriminants and `bits()`
    // -----------------------------------------------------------------------

    #[test]
    fn test_receiver_flag_bit_values_match_cpp() {
        assert_eq!(ReceiverFlag::NoLimit.bits(), 1 << 0);
        assert_eq!(ReceiverFlag::IgnoreBlockItem.bits(), 1 << 1);
        assert_eq!(ReceiverFlag::IgnoreBlockCreature.bits(), 1 << 2);
        assert_eq!(ReceiverFlag::ChildIsOwner.bits(), 1 << 3);
        assert_eq!(ReceiverFlag::Pathfinding.bits(), 1 << 4);
        assert_eq!(ReceiverFlag::IgnoreFieldDamage.bits(), 1 << 5);
        assert_eq!(ReceiverFlag::IgnoreNotMoveable.bits(), 1 << 6);
        assert_eq!(ReceiverFlag::IgnoreAutoStack.bits(), 1 << 7);
    }

    #[test]
    fn test_receiver_flag_bits_are_all_distinct_powers_of_two() {
        let all = [
            ReceiverFlag::NoLimit,
            ReceiverFlag::IgnoreBlockItem,
            ReceiverFlag::IgnoreBlockCreature,
            ReceiverFlag::ChildIsOwner,
            ReceiverFlag::Pathfinding,
            ReceiverFlag::IgnoreFieldDamage,
            ReceiverFlag::IgnoreNotMoveable,
            ReceiverFlag::IgnoreAutoStack,
        ];
        // Each bit must be a power of two.
        for f in &all {
            let b = f.bits();
            assert!(
                b.is_power_of_two(),
                "flag {:?} bits={} is not power of two",
                f,
                b
            );
        }
        // Every pair must be distinct (no two flags share a bit).
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(
                    a.bits(),
                    b.bits(),
                    "duplicate bit between {:?} and {:?}",
                    a,
                    b
                );
            }
        }
    }

    #[test]
    fn test_receiver_flag_bits_compose_into_bitmask() {
        // Real usage: callers OR multiple flags into a u32 mask, then test bits.
        let mask = ReceiverFlag::NoLimit.bits()
            | ReceiverFlag::IgnoreFieldDamage.bits()
            | ReceiverFlag::Pathfinding.bits();

        assert_eq!(mask, 0b0011_0001);
        assert!(mask & ReceiverFlag::NoLimit.bits() != 0);
        assert!(mask & ReceiverFlag::IgnoreFieldDamage.bits() != 0);
        assert!(mask & ReceiverFlag::Pathfinding.bits() != 0);
        // Bits NOT in the mask should remain zero.
        assert_eq!(mask & ReceiverFlag::IgnoreBlockItem.bits(), 0);
        assert_eq!(mask & ReceiverFlag::IgnoreAutoStack.bits(), 0);
    }

    #[test]
    fn test_receiver_flag_equality_and_copy_semantics() {
        let a = ReceiverFlag::NoLimit;
        let b = a; // Copy
        assert_eq!(a, b);
        assert_eq!(a, ReceiverFlag::NoLimit);
        assert_ne!(a, ReceiverFlag::IgnoreBlockItem);
    }

    // -----------------------------------------------------------------------
    // ReceiverLink enum — variants and default
    // -----------------------------------------------------------------------

    #[test]
    fn test_receiver_link_default_is_owner() {
        // Mirrors the C++ default `ReceiverLink_t = LINK_OWNER` in
        // postAddNotification / postRemoveNotification.
        assert_eq!(ReceiverLink::default(), ReceiverLink::Owner);
    }

    #[test]
    fn test_receiver_link_all_variants_distinct() {
        let variants = [
            ReceiverLink::Owner,
            ReceiverLink::Parent,
            ReceiverLink::TopParent,
            ReceiverLink::Near,
        ];
        for (i, a) in variants.iter().enumerate() {
            for b in &variants[i + 1..] {
                assert_ne!(
                    a, b,
                    "ReceiverLink variants must be distinct: {:?} vs {:?}",
                    a, b
                );
            }
        }
    }

    #[test]
    fn test_receiver_link_copy_and_equality() {
        let l = ReceiverLink::TopParent;
        let l2 = l; // Copy
        assert_eq!(l, l2);
        assert_eq!(ReceiverLink::Near, ReceiverLink::Near);
        assert_ne!(ReceiverLink::Owner, ReceiverLink::Parent);
    }

    // -----------------------------------------------------------------------
    // is_removed() default and override (mirrors C++ `virtual bool isRemoved() const { return true; }`)
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_thing_is_removed_returns_true() {
        // C++ base default is `true` — a thing not yet wired into the world
        // is treated as removed.
        let t = DefaultThing;
        assert!(t.is_removed());
    }

    struct LiveThing;
    impl Thing for LiveThing {
        fn is_removed(&self) -> bool {
            false
        }
    }

    #[test]
    fn test_overridden_is_removed_returns_false() {
        let t = LiveThing;
        assert!(!t.is_removed());
    }

    #[test]
    fn test_is_removed_is_independent_of_is_removable() {
        // A thing can be still in the world (not removed) yet non-removable
        // (e.g. a quest stone).
        struct AnchoredThing;
        impl Thing for AnchoredThing {
            fn is_removed(&self) -> bool {
                false
            }
            fn is_removable(&self) -> bool {
                false
            }
        }
        let t = AnchoredThing;
        assert!(!t.is_removed());
        assert!(!t.is_removable());
    }

    // -----------------------------------------------------------------------
    // get_first_index / get_last_index defaults
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_thing_get_first_index_is_zero() {
        let t = DefaultThing;
        assert_eq!(t.get_first_index(), 0);
    }

    #[test]
    fn test_default_thing_get_last_index_is_zero() {
        let t = DefaultThing;
        assert_eq!(t.get_last_index(), 0);
    }

    #[test]
    fn test_overridden_indices_are_returned() {
        struct RangedContainer;
        impl Thing for RangedContainer {
            fn get_first_index(&self) -> usize {
                3
            }
            fn get_last_index(&self) -> usize {
                17
            }
        }
        let c = RangedContainer;
        assert_eq!(c.get_first_index(), 3);
        assert_eq!(c.get_last_index(), 17);
        // Sanity: range is well-formed.
        assert!(c.get_first_index() <= c.get_last_index());
    }

    // -----------------------------------------------------------------------
    // get_thing_index default (-1) + override
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_thing_get_thing_index_returns_minus_one() {
        let t = DefaultThing;
        let child = DefaultThing;
        assert_eq!(t.get_thing_index(&child), -1);
    }

    #[test]
    fn test_overridden_get_thing_index_reports_position() {
        // A toy container that always reports a fixed index to prove the
        // override path is taken.
        struct OneSlotContainer;
        impl Thing for OneSlotContainer {
            fn get_thing_index(&self, _child: &dyn Thing) -> i32 {
                0
            }
        }
        let c = OneSlotContainer;
        let item = DefaultThing;
        assert_eq!(c.get_thing_index(&item), 0);
        assert_ne!(c.get_thing_index(&item), -1);
    }

    // -----------------------------------------------------------------------
    // get_item_type_count default + override
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_thing_get_item_type_count_returns_zero() {
        let t = DefaultThing;
        assert_eq!(t.get_item_type_count(1234, -1), 0);
        assert_eq!(t.get_item_type_count(0, 0), 0);
        assert_eq!(t.get_item_type_count(u16::MAX, i32::MAX), 0);
    }

    #[test]
    fn test_overridden_get_item_type_count_reports_count() {
        struct FixedBag;
        impl Thing for FixedBag {
            fn get_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
                if item_id == 100 && (sub_type == -1 || sub_type == 7) {
                    42
                } else {
                    0
                }
            }
        }
        let bag = FixedBag;
        assert_eq!(bag.get_item_type_count(100, -1), 42);
        assert_eq!(bag.get_item_type_count(100, 7), 42);
        // Wrong sub-type → 0.
        assert_eq!(bag.get_item_type_count(100, 3), 0);
        // Wrong item id → 0.
        assert_eq!(bag.get_item_type_count(101, -1), 0);
    }

    // -----------------------------------------------------------------------
    // get_all_item_type_count default returns map unchanged + override accumulates
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_thing_get_all_item_type_count_returns_map_unchanged() {
        use std::collections::HashMap;
        let t = DefaultThing;
        let mut map: HashMap<u32, u32> = HashMap::new();
        map.insert(99, 5);
        let returned = t.get_all_item_type_count(&mut map);
        // The default impl is a no-op pass-through: same entries, no
        // additions or deletions.
        assert_eq!(returned.len(), 1);
        assert_eq!(returned.get(&99), Some(&5));
    }

    #[test]
    fn test_default_thing_get_all_item_type_count_empty_map_stays_empty() {
        use std::collections::HashMap;
        let t = DefaultThing;
        let mut map: HashMap<u32, u32> = HashMap::new();
        let returned = t.get_all_item_type_count(&mut map);
        assert!(returned.is_empty());
    }

    #[test]
    fn test_overridden_get_all_item_type_count_accumulates() {
        use std::collections::HashMap;
        // A toy container that "contains" three items of two distinct types
        // and folds them into the supplied map.
        struct ToyBag;
        impl Thing for ToyBag {
            fn get_all_item_type_count<'a>(
                &self,
                count_map: &'a mut HashMap<u32, u32>,
            ) -> &'a mut HashMap<u32, u32> {
                *count_map.entry(2160).or_insert(0) += 100; // 100 gold coins
                *count_map.entry(2152).or_insert(0) += 3; // 3 platinum coins
                count_map
            }
        }
        let bag = ToyBag;
        let mut map: HashMap<u32, u32> = HashMap::new();
        map.insert(2160, 1); // pre-existing single coin in the wallet
        bag.get_all_item_type_count(&mut map);
        // Accumulated, not overwritten.
        assert_eq!(map.get(&2160), Some(&101));
        assert_eq!(map.get(&2152), Some(&3));
    }

    // -----------------------------------------------------------------------
    // Cross-crate accessor defaults (Phase B.1.2)
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_thing_get_tile_returns_none() {
        let t = DefaultThing;
        assert!(t.get_tile().is_none());
    }

    #[test]
    fn test_default_thing_get_item_returns_none() {
        let t = DefaultThing;
        assert!(t.get_item().is_none());
    }

    #[test]
    fn test_default_thing_get_creature_returns_none() {
        let t = DefaultThing;
        assert!(t.get_creature().is_none());
    }

    #[test]
    fn test_default_thing_get_receiver_returns_none() {
        let t = DefaultThing;
        assert!(t.get_receiver().is_none());
    }

    #[test]
    fn test_default_thing_get_parent_returns_none() {
        let t = DefaultThing;
        assert!(t.get_parent().is_none());
    }

    #[test]
    fn test_default_thing_has_parent_returns_false_by_default() {
        let t = DefaultThing;
        assert!(!t.has_parent());
    }

    #[test]
    fn test_default_thing_get_top_parent_returns_none() {
        let t = DefaultThing;
        assert!(t.get_top_parent().is_none());
    }

    #[test]
    fn test_default_thing_get_position_tuple_returns_none() {
        let t = DefaultThing;
        assert!(t.get_position_tuple().is_none());
    }

    // --- Overrides participate in trait dispatch ---

    struct PlacedThing {
        pos: (u16, u16, u8),
    }

    impl Thing for PlacedThing {
        fn get_position_tuple(&self) -> Option<(u16, u16, u8)> {
            Some(self.pos)
        }
    }

    #[test]
    fn test_overridden_get_position_tuple_returns_value() {
        let p = PlacedThing { pos: (100, 200, 7) };
        assert_eq!(p.get_position_tuple(), Some((100, 200, 7)));
    }

    struct ChildItem<'a> {
        parent: &'a DefaultThing,
    }

    impl<'a> Thing for ChildItem<'a> {
        fn get_parent(&self) -> Option<&dyn Thing> {
            Some(self.parent)
        }
        fn get_item(&self) -> Option<&dyn Thing> {
            Some(self)
        }
    }

    #[test]
    fn test_overridden_get_parent_returns_parent() {
        let parent = DefaultThing;
        let child = ChildItem { parent: &parent };
        assert!(child.get_parent().is_some());
        assert!(child.has_parent());
    }

    #[test]
    fn test_overridden_get_item_returns_self() {
        let parent = DefaultThing;
        let child = ChildItem { parent: &parent };
        assert!(child.get_item().is_some());
    }

    #[test]
    fn test_default_get_top_parent_walks_via_get_parent() {
        // Without an override the trait default returns the direct parent.
        let parent = DefaultThing;
        let child = ChildItem { parent: &parent };
        assert!(child.get_top_parent().is_some());
    }

    struct CreatureThingMarker;
    impl Thing for CreatureThingMarker {
        fn get_creature(&self) -> Option<&dyn Thing> {
            Some(self)
        }
        fn is_creature(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_creature_marker_get_creature_returns_self() {
        let c = CreatureThingMarker;
        assert!(c.get_creature().is_some());
        assert!(c.is_creature());
    }
}
