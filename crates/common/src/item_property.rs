//! `ItemProperty` discriminator — mirrors C++ `ITEMPROPERTY` from `item.h`.
//!
//! Lives in `common` (layer 0) because both `items` (`Item::has_property`)
//! and `map` (`Tile::has_property`) need to consult the same enum. The
//! enum variant order matches the C++ declaration so the `as i64` cast
//! exposed to Lua via `CONST_PROP_*` constants stays stable.

/// Item-property discriminator used by `Item::has_property` (single item)
/// and `Tile::has_property` (any item on the tile).
///
/// Mirrors the C++ `ITEMPROPERTY` enum in `item.h`. Variant order is
/// load-bearing — the discriminant values are surfaced to Lua scripts as
/// `CONST_PROP_BLOCKSOLID = 0`, `CONST_PROP_HASHEIGHT = 1`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemProperty {
    BlockSolid,
    HasHeight,
    BlockProjectile,
    BlockPath,
    IsVertical,
    IsHorizontal,
    Moveable,
    ImmovableBlockSolid,
    ImmovableBlockPath,
    ImmovableNoFieldBlockPath,
    NoFieldBlockPath,
    SupportHangable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_match_cpp_const_prop_values() {
        // C++ `ITEMPROPERTY` is a plain enum starting at 0; the Lua-side
        // `CONST_PROP_*` constants are exposed via the same numbering.
        assert_eq!(ItemProperty::BlockSolid as i64, 0);
        assert_eq!(ItemProperty::HasHeight as i64, 1);
        assert_eq!(ItemProperty::BlockProjectile as i64, 2);
        assert_eq!(ItemProperty::BlockPath as i64, 3);
        assert_eq!(ItemProperty::IsVertical as i64, 4);
        assert_eq!(ItemProperty::IsHorizontal as i64, 5);
        assert_eq!(ItemProperty::Moveable as i64, 6);
        assert_eq!(ItemProperty::ImmovableBlockSolid as i64, 7);
        assert_eq!(ItemProperty::ImmovableBlockPath as i64, 8);
        assert_eq!(ItemProperty::ImmovableNoFieldBlockPath as i64, 9);
        assert_eq!(ItemProperty::NoFieldBlockPath as i64, 10);
        assert_eq!(ItemProperty::SupportHangable as i64, 11);
    }
}
