// Migrated from forgottenserver/src/tile.h + tile.cpp
//
// Tile is the fundamental map unit. It holds a position, tile-state flags,
// an optional ground item, a list of stacked items, and a list of creature IDs.
//
// Two variants match the C++ DynamicTile / StaticTile:
//   - TileKind::Dynamic  — items stored in an always-allocated Vec
//   - TileKind::Static   — items stored in a lazily-allocated Option<Vec>

use forgottenserver_common::position::Position;
use forgottenserver_items::item::Item;

// ---------------------------------------------------------------------------
// TileFlags — mirrors tileflags_t from tile.h
// ---------------------------------------------------------------------------

/// Bit-flag constants that mirror C++ `tileflags_t`.
pub mod flags {
    pub const NONE: u32 = 0;

    // Floor-change directions
    pub const FLOORCHANGE_DOWN: u32 = 1 << 0;
    pub const FLOORCHANGE_NORTH: u32 = 1 << 1;
    pub const FLOORCHANGE_SOUTH: u32 = 1 << 2;
    pub const FLOORCHANGE_EAST: u32 = 1 << 3;
    pub const FLOORCHANGE_WEST: u32 = 1 << 4;
    pub const FLOORCHANGE_SOUTH_ALT: u32 = 1 << 5;
    pub const FLOORCHANGE_EAST_ALT: u32 = 1 << 6;

    // Zone flags
    pub const PROTECTIONZONE: u32 = 1 << 7;
    pub const NOPVPZONE: u32 = 1 << 8;
    pub const NOLOGOUT: u32 = 1 << 9;
    pub const PVPZONE: u32 = 1 << 10;

    // Special tile types
    pub const TELEPORT: u32 = 1 << 11;
    pub const MAGICFIELD: u32 = 1 << 12;
    pub const MAILBOX: u32 = 1 << 13;
    pub const TRASHHOLDER: u32 = 1 << 14;
    pub const BED: u32 = 1 << 15;
    pub const DEPOT: u32 = 1 << 16;

    // Blocking flags
    pub const BLOCKSOLID: u32 = 1 << 17;
    pub const BLOCKPATH: u32 = 1 << 18;
    pub const IMMOVABLEBLOCKSOLID: u32 = 1 << 19;
    pub const IMMOVABLEBLOCKPATH: u32 = 1 << 20;
    pub const IMMOVABLENOFIELDBLOCKPATH: u32 = 1 << 21;
    pub const NOFIELDBLOCKPATH: u32 = 1 << 22;

    // Hangable support
    pub const SUPPORTS_HANGABLE: u32 = 1 << 23;

    /// Composite: all floor-change bits (matches C++ TILESTATE_FLOORCHANGE).
    pub const FLOORCHANGE: u32 = FLOORCHANGE_DOWN
        | FLOORCHANGE_NORTH
        | FLOORCHANGE_SOUTH
        | FLOORCHANGE_EAST
        | FLOORCHANGE_WEST
        | FLOORCHANGE_SOUTH_ALT
        | FLOORCHANGE_EAST_ALT;
}

// ---------------------------------------------------------------------------
// TILE_UPDATE_THRESHOLD — mirrors `inline constexpr size_t TILE_UPDATE_THRESHOLD`
// from tile.h line 116.
//
// When a tile holds more items+creatures than this threshold the server sends a
// full-tile update packet to observers instead of a delta.  The constant is
// consumed by the protocol/game layer; it lives here so the map crate owns the
// canonical definition.
// ---------------------------------------------------------------------------

/// Maximum number of things on a tile before a full tile-update is sent to
/// observers.  Mirrors C++ `inline constexpr size_t TILE_UPDATE_THRESHOLD = 8`.
pub const TILE_UPDATE_THRESHOLD: usize = 8;

// ---------------------------------------------------------------------------
// ZoneType — mirrors ZoneType_t from tile.h
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Protection,
    NoPvp,
    Pvp,
    NoLogout,
    Normal,
}

// ---------------------------------------------------------------------------
// QueryFlags — mirrors FLAG_* constants used in queryAdd / queryRemove
// ---------------------------------------------------------------------------

/// Bit-flag constants for `query_add` / `query_remove` calls.
/// Mirrors the C++ FLAG_* constants (defined near Cylinder in game headers).
pub mod query_flags {
    /// No restrictions — bypass all checks.
    pub const NOLIMIT: u32 = 1 << 0;
    /// Pathfinding mode — avoids floor changes and teleports.
    pub const PATHFINDING: u32 = 1 << 1;
    /// Ignore blocking creatures.
    pub const IGNOREBLOCKCREATURE: u32 = 1 << 2;
    /// Ignore blocking items (still blocks unmoveable items).
    pub const IGNOREBLOCKITEM: u32 = 1 << 3;
    /// Ignore all checks (alias: always allow).
    pub const IGNORECHECKS: u32 = 1 << 4;
    /// Ignore the "not moveable" flag on items.
    pub const IGNORENOTMOVEABLE: u32 = 1 << 5;
    /// Ignore field damage (magic fields).
    pub const IGNOREFIELD: u32 = 1 << 6;
}

// ---------------------------------------------------------------------------
// ReturnValue — mirrors the C++ ReturnValue enum for queryAdd/queryRemove
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnValue {
    NoError,
    NotPossible,
    NotEnoughRoom,
    NotMoveable,
    ItemCannotBeMovedThere,
    NeedExchange,
}

impl ReturnValue {
    pub fn is_ok(self) -> bool {
        self == ReturnValue::NoError
    }
}

// ---------------------------------------------------------------------------
// TileKind — DynamicTile vs StaticTile storage strategy
// ---------------------------------------------------------------------------

/// Stores the variable-length parts of a tile.
///
/// - `Dynamic`: items Vec is always allocated (good for frequently-changed
///   tiles such as walkable ground).
/// - `Static`: items Vec is lazily allocated behind an `Option` (good for
///   blocking tiles that rarely contain items).
#[derive(Debug, Clone)]
pub enum TileKind {
    /// Items stored in an always-present Vec (DynamicTile equivalent).
    Dynamic {
        items: Vec<Item>,
        creatures: Vec<u32>,
    },
    /// Items stored in a lazily-allocated Vec (StaticTile equivalent).
    Static {
        items: Option<Vec<Item>>,
        creatures: Option<Vec<u32>>,
    },
}

impl TileKind {
    pub fn items(&self) -> &[Item] {
        match self {
            TileKind::Dynamic { items, .. } => items.as_slice(),
            TileKind::Static { items, .. } => items.as_deref().unwrap_or(&[]),
        }
    }

    pub fn items_mut(&mut self) -> &mut Vec<Item> {
        match self {
            TileKind::Dynamic { items, .. } => items,
            TileKind::Static { items, .. } => items.get_or_insert_with(Vec::new),
        }
    }

    pub fn creatures(&self) -> &[u32] {
        match self {
            TileKind::Dynamic { creatures, .. } => creatures.as_slice(),
            TileKind::Static { creatures, .. } => creatures.as_deref().unwrap_or(&[]),
        }
    }

    pub fn creatures_mut(&mut self) -> &mut Vec<u32> {
        match self {
            TileKind::Dynamic { creatures, .. } => creatures,
            TileKind::Static { creatures, .. } => creatures.get_or_insert_with(Vec::new),
        }
    }
}

// ---------------------------------------------------------------------------
// Tile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Tile {
    pub position: Position,
    pub flags: u32,
    ground: Option<Item>,
    kind: TileKind,
    /// When `Some(house_id)` this tile belongs to the house with the given ID.
    /// Mirrors C++ `HouseTile::getHouseID()` / `Tile::getHouseTile()`.
    house_id: Option<u32>,
}

impl Tile {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Create a `DynamicTile` at the given coordinates.
    pub fn new_dynamic(x: u16, y: u16, z: u8) -> Self {
        Tile {
            position: Position::new(x, y, z),
            flags: flags::NONE,
            ground: None,
            house_id: None,
            kind: TileKind::Dynamic {
                items: Vec::new(),
                creatures: Vec::new(),
            },
        }
    }

    /// Create a `StaticTile` at the given coordinates.
    pub fn new_static(x: u16, y: u16, z: u8) -> Self {
        Tile {
            position: Position::new(x, y, z),
            flags: flags::NONE,
            ground: None,
            house_id: None,
            kind: TileKind::Static {
                items: None,
                creatures: None,
            },
        }
    }

    /// Shorthand — creates a `DynamicTile` (most tests use this).
    pub fn new(x: u16, y: u16, z: u8) -> Self {
        Self::new_dynamic(x, y, z)
    }

    // -----------------------------------------------------------------------
    // Flag helpers — mirrors Tile::hasFlag / setFlag / resetFlag
    // -----------------------------------------------------------------------

    /// Returns `true` when *all* bits in `flag` are set.
    #[inline]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag && flag != 0
    }

    /// Sets all bits in `flag`.
    #[inline]
    pub fn set_flag(&mut self, flag: u32) {
        self.flags |= flag;
    }

    /// Clears all bits in `flag`.
    #[inline]
    pub fn reset_flag(&mut self, flag: u32) {
        self.flags &= !flag;
    }

    // -----------------------------------------------------------------------
    // Zone — mirrors Tile::getZone()
    // -----------------------------------------------------------------------

    /// Returns the zone type based on currently set flags.
    ///
    /// Priority: Protection > NoPvp > Pvp > NoLogout > Normal.
    pub fn get_zone(&self) -> ZoneType {
        if self.has_flag(flags::PROTECTIONZONE) {
            ZoneType::Protection
        } else if self.has_flag(flags::NOPVPZONE) {
            ZoneType::NoPvp
        } else if self.has_flag(flags::PVPZONE) {
            ZoneType::Pvp
        } else if self.has_flag(flags::NOLOGOUT) {
            ZoneType::NoLogout
        } else {
            ZoneType::Normal
        }
    }

    // -----------------------------------------------------------------------
    // Convenience zone/flag queries
    // -----------------------------------------------------------------------

    pub fn is_protection_zone(&self) -> bool {
        self.has_flag(flags::PROTECTIONZONE)
    }

    pub fn has_floor_change(&self) -> bool {
        self.flags & flags::FLOORCHANGE != 0
    }

    // -----------------------------------------------------------------------
    // Ground item — mirrors Tile::ground / getGround / setGround
    // -----------------------------------------------------------------------

    pub fn get_ground(&self) -> Option<&Item> {
        self.ground.as_ref()
    }

    pub fn set_ground(&mut self, item: Item) {
        self.ground = Some(item);
    }

    pub fn take_ground(&mut self) -> Option<Item> {
        self.ground.take()
    }

    // -----------------------------------------------------------------------
    // Item list operations
    // -----------------------------------------------------------------------

    /// Adds `item` to the tile's item list.
    pub fn add_item(&mut self, item: Item) {
        self.kind.items_mut().push(item);
    }

    /// Removes the item at the given index.  Returns the removed item if the
    /// index was valid.
    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        let items = self.kind.items_mut();
        if index < items.len() {
            Some(items.remove(index))
        } else {
            None
        }
    }

    /// Number of items on the tile (excluding the ground item).
    pub fn get_item_count(&self) -> usize {
        self.kind.items().len()
    }

    /// Read-only access to the item list.
    pub fn items(&self) -> &[Item] {
        self.kind.items()
    }

    // -----------------------------------------------------------------------
    // Creature list operations (creatures are stored as u32 IDs)
    // -----------------------------------------------------------------------

    /// Adds a creature ID to the front of the creature list.
    ///
    /// Mirrors C++ `creatures->insert(creatures->begin(), creature)` — the most
    /// recently added creature is always first (top creature).
    pub fn add_creature(&mut self, creature_id: u32) {
        self.kind.creatures_mut().insert(0, creature_id);
    }

    /// Removes the first occurrence of `creature_id`.  Returns `true` if it
    /// was present.
    pub fn remove_creature(&mut self, creature_id: u32) -> bool {
        let creatures = self.kind.creatures_mut();
        if let Some(pos) = creatures.iter().position(|&id| id == creature_id) {
            creatures.remove(pos);
            true
        } else {
            false
        }
    }

    /// Number of creatures on the tile.
    pub fn get_creature_count(&self) -> usize {
        self.kind.creatures().len()
    }

    /// Read-only access to the creature ID list.
    pub fn creatures(&self) -> &[u32] {
        self.kind.creatures()
    }

    // -----------------------------------------------------------------------
    // Thing count — mirrors Tile::getThingCount()
    // -----------------------------------------------------------------------

    /// Returns `creature_count + item_count + 1_if_ground`.
    pub fn get_thing_count(&self) -> usize {
        let mut count = self.get_creature_count() + self.get_item_count();
        if self.ground.is_some() {
            count += 1;
        }
        count
    }

    // -----------------------------------------------------------------------
    // TileKind queries (for tests that care about the storage strategy)
    // -----------------------------------------------------------------------

    pub fn is_dynamic(&self) -> bool {
        matches!(self.kind, TileKind::Dynamic { .. })
    }

    pub fn is_static(&self) -> bool {
        matches!(self.kind, TileKind::Static { .. })
    }

    // -----------------------------------------------------------------------
    // Top-thing queries — mirrors C++ getTopCreature / getTopItem / getTopDownItem
    // -----------------------------------------------------------------------

    /// Returns the ID of the top creature on the tile (the one at the front of
    /// the creature list), or `None` if there are no creatures.
    ///
    /// Mirrors C++ `Tile::getTopCreature` which returns `*creatures->begin()`.
    pub fn get_top_creature(&self) -> Option<u32> {
        self.kind.creatures().first().copied()
    }

    /// Returns the last item in the item list (top of the stacked items), or
    /// `None` if no items are present.
    ///
    /// Mirrors `TileItemVector::getTopTopItem` — returns the last top-order item.
    pub fn get_top_item(&self) -> Option<&Item> {
        self.kind.items().last()
    }

    /// Returns the first item in the item list (bottom-most / down-item at index
    /// 0), or `None` if no items are present.
    ///
    /// Mirrors `TileItemVector::getTopDownItem` — returns the first down-item.
    pub fn get_top_down_item(&self) -> Option<&Item> {
        self.kind.items().first()
    }

    // -----------------------------------------------------------------------
    // Block-property queries — mirrors isBlockSolid / isBlockProjectile / isBlockPathfinder
    // -----------------------------------------------------------------------

    /// Returns `true` when any item on this tile (ground or stacked) has
    /// `block_solid == true`.
    ///
    /// Mirrors C++ `Tile::hasFlag(TILESTATE_BLOCKSOLID)`.
    pub fn is_block_solid(&self) -> bool {
        self.has_flag(flags::BLOCKSOLID)
    }

    /// Returns `true` when any item on this tile has `block_projectile == true`.
    ///
    /// Derived by iterating items; in the C++ code this is determined via
    /// `ITEMPROPERTY` checks.  Here we check the item-type flag directly.
    pub fn is_block_projectile(&self) -> bool {
        if let Some(g) = &self.ground {
            if g.block_projectile() {
                return true;
            }
        }
        self.kind.items().iter().any(|item| item.block_projectile())
    }

    /// Returns `true` when any item blocks pathfinding (mirrors
    /// `TILESTATE_BLOCKPATH`).
    pub fn is_block_path_finder(&self) -> bool {
        self.has_flag(flags::BLOCKPATH)
    }

    // -----------------------------------------------------------------------
    // isMoveableBlocking — mirrors C++ Tile::isMoveableBlocking
    // -----------------------------------------------------------------------

    /// Returns `true` when the tile has no ground OR has `BLOCKSOLID` set.
    ///
    /// Mirrors C++ `Tile::isMoveableBlocking()`.
    pub fn is_moveable_blocking(&self) -> bool {
        self.ground.is_none() || self.has_flag(flags::BLOCKSOLID)
    }

    // -----------------------------------------------------------------------
    // House tile — mirrors HouseTile
    // -----------------------------------------------------------------------

    /// Returns the house ID stored on this tile, or `None` if this tile does
    /// not belong to a house.
    ///
    /// Mirrors C++ `Tile::getHouseTile()` — when non-null the tile is a house
    /// tile.
    pub fn get_house_id(&self) -> Option<u32> {
        self.house_id
    }

    /// Returns `true` when this tile belongs to a house.
    pub fn is_house_tile(&self) -> bool {
        self.house_id.is_some()
    }

    /// Mark this tile as belonging to a house.
    pub fn set_house_id(&mut self, house_id: u32) {
        self.house_id = Some(house_id);
    }

    /// Clear the house association.
    pub fn clear_house_id(&mut self) {
        self.house_id = None;
    }

    // -----------------------------------------------------------------------
    // query_add — mirrors Tile::queryAdd (simplified, creature/item-agnostic)
    //
    // Full C++ logic depends on Creature/Player/Monster hierarchies that don't
    // exist in this crate.  We implement the pure-tile logic that can be tested
    // without those types:
    //   - NOLIMIT  → always allow
    //   - IGNORECHECKS → always allow
    //   - PATHFINDING + (FLOORCHANGE | TELEPORT) → deny
    //   - no ground → deny (unless NOLIMIT/IGNORECHECKS)
    //   - BLOCKSOLID and !IGNOREBLOCKITEM → deny (NotEnoughRoom)
    //   - item count overflow (≥ 0xFFFF) → deny
    // -----------------------------------------------------------------------

    /// Query whether a thing can be added to this tile.
    ///
    /// `flags` is a bitmask of `query_flags::*` constants.
    ///
    /// Returns `ReturnValue::NoError` when the thing may be placed, or an
    /// appropriate denial code otherwise.
    pub fn query_add(&self, query_flags_arg: u32) -> ReturnValue {
        use query_flags::*;

        // NOLIMIT / IGNORECHECKS bypass everything
        if query_flags_arg & NOLIMIT != 0 || query_flags_arg & IGNORECHECKS != 0 {
            return ReturnValue::NoError;
        }

        // Pathfinding avoids floor changes and teleports.
        // Note: FLOORCHANGE is a composite mask — we check if ANY floorchange
        // bit is set (mirrors C++ `hasFlag(TILESTATE_FLOORCHANGE | TILESTATE_TELEPORT)`
        // which internally uses `hasBitSet` = any-bit-set semantics).
        if query_flags_arg & PATHFINDING != 0
            && (self.flags & (flags::FLOORCHANGE | flags::TELEPORT) != 0)
        {
            return ReturnValue::NotPossible;
        }

        // Item overflow guard (matches C++ `items->size() >= 0xFFFF`).
        // Checked before the no-ground guard so callers that bulk-fill the
        // items list (e.g. a tile carrying only stacked items) still get a
        // deterministic rejection.
        if self.get_item_count() >= 0xFFFF {
            return ReturnValue::NotPossible;
        }

        // Solid blocking
        if query_flags_arg & IGNOREBLOCKITEM == 0 {
            // No ground → not possible (only enforced on the non-IGNOREBLOCKITEM
            // branch, matching C++'s creature-add path which bails when
            // `!ground`).
            if self.ground.is_none() {
                return ReturnValue::NotPossible;
            }
            if self.has_flag(flags::BLOCKSOLID) {
                return ReturnValue::NotEnoughRoom;
            }
        } else {
            // FLAG_IGNOREBLOCKITEM is set — still block on immoveable solid
            // items (ground or stacked).  Mirrors C++ tile.cpp:600-614.
            if let Some(g) = &self.ground {
                if g.block_solid() && !g.is_moveable() {
                    return ReturnValue::NotPossible;
                }
            }
            for item in self.kind.items() {
                if item.block_solid() && !item.is_moveable() {
                    return ReturnValue::NotPossible;
                }
            }
        }

        ReturnValue::NoError
    }

    // -----------------------------------------------------------------------
    // query_remove — mirrors Tile::queryRemove
    //
    // Simplified: checks that count > 0 and (item is moveable OR
    // IGNORENOTMOVEABLE flag is set).
    // -----------------------------------------------------------------------

    /// Query whether a stacked item at `item_index` (0-based in `items()`)
    /// can be removed with the given `count` and `flags`.
    ///
    /// Returns `ReturnValue::NoError` when removal is allowed.
    pub fn query_remove(&self, item_index: usize, count: u32, flags: u32) -> ReturnValue {
        use query_flags::IGNORENOTMOVEABLE;

        if count == 0 {
            return ReturnValue::NotPossible;
        }

        let items = self.kind.items();
        let Some(item) = items.get(item_index) else {
            return ReturnValue::NotPossible;
        };

        // Stackable item: can't remove more than present
        if item.is_stackable() && count > item.get_item_count() as u32 {
            return ReturnValue::NotPossible;
        }

        // Moveable check
        if !item.is_moveable() && flags & IGNORENOTMOVEABLE == 0 {
            return ReturnValue::NotMoveable;
        }

        ReturnValue::NoError
    }

    // -----------------------------------------------------------------------
    // Serialize / Deserialize — tile-level round-trip
    //
    // Format (little-endian):
    //   flags:  u32  (tile flags)
    //   x/y/z:  u16, u16, u8  (position)
    //   house_id: u8 (0 = none, 1 = present) + optional u32
    //   has_ground: u8 (0/1)
    //   item_count: u16
    //   [item_id: u16 for each item]
    //   creature_count: u16
    //   [creature_id: u32 for each creature]
    // -----------------------------------------------------------------------

    /// Serialize this tile to a byte buffer.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // flags
        buf.extend_from_slice(&self.flags.to_le_bytes());

        // position
        buf.extend_from_slice(&self.position.x.to_le_bytes());
        buf.extend_from_slice(&self.position.y.to_le_bytes());
        buf.push(self.position.z);

        // house_id
        if let Some(hid) = self.house_id {
            buf.push(1u8);
            buf.extend_from_slice(&hid.to_le_bytes());
        } else {
            buf.push(0u8);
        }

        // ground item
        if let Some(g) = &self.ground {
            buf.push(1u8);
            buf.extend_from_slice(&g.get_id().to_le_bytes());
        } else {
            buf.push(0u8);
        }

        // items
        let items = self.kind.items();
        let item_count = items.len() as u16;
        buf.extend_from_slice(&item_count.to_le_bytes());
        for item in items {
            buf.extend_from_slice(&item.get_id().to_le_bytes());
        }

        // creatures
        let creatures = self.kind.creatures();
        let creature_count = creatures.len() as u16;
        buf.extend_from_slice(&creature_count.to_le_bytes());
        for &cid in creatures {
            buf.extend_from_slice(&cid.to_le_bytes());
        }

        buf
    }

    /// Deserialize a tile from a byte slice produced by `serialize`.
    ///
    /// The `item_factory` closure is called with each serialized item ID and
    /// should return an `Item` — this decouples the tile deserializer from the
    /// global items registry.
    pub fn deserialize<F>(data: &[u8], mut item_factory: F) -> Result<Self, String>
    where
        F: FnMut(u16) -> Item,
    {
        let mut pos = 0usize;

        macro_rules! read_u8 {
            () => {{
                if pos >= data.len() {
                    return Err("deserialize: unexpected end reading u8".into());
                }
                let v = data[pos];
                pos += 1;
                v
            }};
        }

        macro_rules! read_u16 {
            () => {{
                if pos + 2 > data.len() {
                    return Err("deserialize: unexpected end reading u16".into());
                }
                let v = u16::from_le_bytes([data[pos], data[pos + 1]]);
                pos += 2;
                v
            }};
        }

        macro_rules! read_u32 {
            () => {{
                if pos + 4 > data.len() {
                    return Err("deserialize: unexpected end reading u32".into());
                }
                let v =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
                pos += 4;
                v
            }};
        }

        let tile_flags = read_u32!();
        let x = read_u16!();
        let y = read_u16!();
        let z = read_u8!();

        let house_present = read_u8!();
        let house_id = if house_present != 0 {
            Some(read_u32!())
        } else {
            None
        };

        let has_ground = read_u8!();
        let ground = if has_ground != 0 {
            let gid = read_u16!();
            Some(item_factory(gid))
        } else {
            None
        };

        let item_count = read_u16!();
        let mut items: Vec<Item> = Vec::with_capacity(item_count as usize);
        for _ in 0..item_count {
            let iid = read_u16!();
            items.push(item_factory(iid));
        }

        let creature_count = read_u16!();
        let mut creatures: Vec<u32> = Vec::with_capacity(creature_count as usize);
        for _ in 0..creature_count {
            creatures.push(read_u32!());
        }

        Ok(Tile {
            position: Position::new(x, y, z),
            flags: tile_flags,
            ground,
            house_id,
            kind: TileKind::Dynamic { items, creatures },
        })
    }
}

// ---------------------------------------------------------------------------
// ItemProperty enum — mirrors C++ `ITEMPROPERTY` from `item.h`
// ---------------------------------------------------------------------------

// `ItemProperty` moved to `forgottenserver_common::item_property` so both
// `items::item::Item::has_property` and `map::tile::Tile::has_property`
// can share a single definition (items can't import map — they're layer
// 2 and 4 respectively). Re-exported here so existing callers keep
// working with `forgottenserver_map::tile::ItemProperty`.
pub use forgottenserver_common::item_property::ItemProperty;

// ---------------------------------------------------------------------------
// Tile — extended C++ parity API (Phase A.3 architectural rewrite)
//
// All methods below mirror the C++ `Tile` API as documented in
// `forgottenserver/src/tile.h` lines 118-256.  The existing TileKind storage
// (flat Vec<Item>) is preserved; top/down ordering is computed at query time
// from `Item::always_on_top()`.
// ---------------------------------------------------------------------------

impl Tile {
    // -----------------------------------------------------------------------
    // Top/down item counts — mirrors TileItemVector::getTopItemCount /
    // getDownItemCount
    // -----------------------------------------------------------------------

    /// Number of items on this tile marked `always_on_top` (top-order items).
    pub fn get_top_item_count(&self) -> u32 {
        self.kind
            .items()
            .iter()
            .filter(|i| i.always_on_top())
            .count() as u32
    }

    /// Number of items on this tile NOT marked `always_on_top` (down-order
    /// items like stair, decoration). Matches C++ `downItemCount`.
    pub fn get_down_item_count(&self) -> u32 {
        self.kind
            .items()
            .iter()
            .filter(|i| !i.always_on_top())
            .count() as u32
    }

    // -----------------------------------------------------------------------
    // First/last index — mirrors Tile::getFirstIndex / getLastIndex
    // -----------------------------------------------------------------------

    /// First valid index on the tile (always 0).
    ///
    /// Mirrors C++ `size_t Tile::getFirstIndex() const`. The C++ base inherits
    /// `getFirstIndex() = 0` from `Thing`.
    pub fn get_first_index(&self) -> usize {
        0
    }

    /// Last valid index on the tile — equals `get_thing_count()`.
    ///
    /// Mirrors C++ `size_t Tile::getLastIndex() const { return getThingCount(); }`.
    pub fn get_last_index_inclusive(&self) -> usize {
        self.get_thing_count()
    }

    // -----------------------------------------------------------------------
    // Special-item lookups (field / teleport / trash / mailbox / bed)
    //
    // Each returns `Option<&Item>` so callers can inspect or downcast as
    // needed. Implementation: linear scan of the item list checking the
    // item-type kind.
    // -----------------------------------------------------------------------

    /// Returns the first magic-field item on this tile, if any.
    ///
    /// Mirrors C++ `MagicField* Tile::getFieldItem() const`.
    pub fn get_field_item(&self) -> Option<&Item> {
        self.kind
            .items()
            .iter()
            .find(|i| i.item_type.is_magic_field())
    }

    /// Returns the first teleport item on this tile, if any.
    ///
    /// Mirrors C++ `Teleport* Tile::getTeleportItem() const`.
    pub fn get_teleport_item(&self) -> Option<&Item> {
        self.kind.items().iter().find(|i| i.item_type.is_teleport())
    }

    /// Returns the first trash-holder item on this tile, if any.
    ///
    /// Mirrors C++ `TrashHolder* Tile::getTrashHolder() const`.
    pub fn get_trash_holder(&self) -> Option<&Item> {
        self.kind
            .items()
            .iter()
            .find(|i| i.item_type.is_trash_holder())
    }

    /// Returns the first mailbox item on this tile, if any.
    ///
    /// Mirrors C++ `Mailbox* Tile::getMailbox() const`.
    pub fn get_mailbox(&self) -> Option<&Item> {
        self.kind.items().iter().find(|i| i.item_type.is_mailbox())
    }

    /// Returns the first bed item on this tile, if any.
    ///
    /// Mirrors C++ `BedItem* Tile::getBedItem() const`.
    pub fn get_bed_item(&self) -> Option<&Item> {
        self.kind.items().iter().find(|i| i.item_type.is_bed())
    }

    /// Returns the first item whose item-type `always_on_top_order` equals
    /// `top_order`. Mirrors C++ `Item* Tile::getItemByTopOrder(int32_t topOrder)`.
    pub fn get_item_by_top_order(&self, top_order: u8) -> Option<&Item> {
        self.kind
            .items()
            .iter()
            .find(|i| i.always_on_top() && i.item_type.always_on_top_order == top_order)
    }

    // -----------------------------------------------------------------------
    // Indexing — mirrors Tile::getThing(size_t), getThingIndex(Thing),
    // getUseItem(int32_t)
    // -----------------------------------------------------------------------

    /// Returns the item at the given linear thing index (creatures first,
    /// then ground, then stacked items).
    ///
    /// Mirrors C++ `Thing* Tile::getThing(size_t index) const`. Returns
    /// `None` when `index >= get_thing_count()`.
    pub fn get_thing_at_index(&self, index: usize) -> Option<TileThingRef<'_>> {
        let creature_count = self.get_creature_count();
        if index < creature_count {
            return Some(TileThingRef::Creature(self.kind.creatures()[index]));
        }
        let mut i = index - creature_count;
        if let Some(g) = self.ground.as_ref() {
            if i == 0 {
                return Some(TileThingRef::Ground(g));
            }
            i -= 1;
        }
        self.kind.items().get(i).map(TileThingRef::Item)
    }

    /// Returns the linear index of a creature on this tile, or `None` when
    /// the creature is absent. Mirrors C++ `getThingIndex` for the creature
    /// branch.
    pub fn get_creature_index(&self, creature_id: u32) -> Option<usize> {
        self.kind
            .creatures()
            .iter()
            .position(|&id| id == creature_id)
    }

    /// Returns the linear thing-index of an item by reference identity
    /// (matches on item-id and stack-count since `Item` does not implement
    /// `Eq` strictly by identity). Mirrors the C++ pointer-equality match.
    pub fn get_item_thing_index(&self, item_id: u16) -> Option<usize> {
        let creature_count = self.get_creature_count();
        let ground_offset = if self.ground.is_some() { 1 } else { 0 };
        // Check ground first.
        if let Some(g) = &self.ground {
            if g.get_id() == item_id {
                return Some(creature_count);
            }
        }
        // Then stacked items.
        self.kind
            .items()
            .iter()
            .position(|i| i.get_id() == item_id)
            .map(|pos| creature_count + ground_offset + pos)
    }

    /// Returns the item at the given thing-index suitable for "use this item"
    /// actions. Mirrors C++ `Item* Tile::getUseItem(int32_t index)`.
    pub fn get_use_item(&self, index: i32) -> Option<&Item> {
        if index < 0 {
            // C++ returns the top down-item when index is negative.
            return self.get_top_down_item();
        }
        let creature_count = self.get_creature_count();
        let idx = index as usize;
        if idx < creature_count {
            return None;
        }
        let i = idx - creature_count;
        if self.ground.is_some() {
            if i == 0 {
                return self.ground.as_ref();
            }
            return self.kind.items().get(i - 1);
        }
        self.kind.items().get(i)
    }

    /// Returns the count of items with the given `item_id` directly on this
    /// tile (ground + stacked items). When `sub_type != -1`, only items whose
    /// stack-count equals `sub_type` are counted.
    ///
    /// Mirrors C++ `uint32_t Tile::getItemTypeCount(uint16_t itemId, int32_t subType) const`.
    pub fn get_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
        let mut total: u32 = 0;
        if let Some(g) = &self.ground {
            if g.get_id() == item_id && (sub_type == -1 || g.get_count() as i32 == sub_type) {
                total += g.get_count() as u32;
            }
        }
        for item in self.kind.items() {
            if item.get_id() == item_id && (sub_type == -1 || item.get_count() as i32 == sub_type) {
                total += item.get_count() as u32;
            }
        }
        total
    }

    // -----------------------------------------------------------------------
    // Has-creature / remove-creature — mirrors Tile::hasCreature /
    // removeCreature
    // -----------------------------------------------------------------------

    /// Whether the given creature ID is present on this tile.
    ///
    /// Mirrors C++ `bool Tile::hasCreature(Creature* creature) const`.
    pub fn has_creature(&self, creature_id: u32) -> bool {
        self.kind.creatures().contains(&creature_id)
    }

    // -----------------------------------------------------------------------
    // Update / replace / explicit remove — additive surface
    //
    // C++ `updateThing` mutates an existing item's id+count; here we offer a
    // typed equivalent that operates on the items vector by index.
    // -----------------------------------------------------------------------

    /// Update the item at `item_index` in-place, replacing its id and count
    /// in the existing stacked-item slot.
    ///
    /// Mirrors C++ `Tile::updateThing(Thing*, uint16_t itemId, uint32_t count)`
    /// for the item branch. The caller supplies a freshly-built [`Item`].
    /// Returns the replaced item if the index was valid.
    pub fn update_item(&mut self, item_index: usize, new_item: Item) -> Option<Item> {
        let items = self.kind.items_mut();
        if item_index < items.len() {
            let old = std::mem::replace(&mut items[item_index], new_item);
            Some(old)
        } else {
            None
        }
    }

    /// Replace the item at `item_index` with `new_item`. Synonym for
    /// [`update_item`], matching the C++ `Tile::replaceThing(uint32_t index, Thing*)`
    /// signature shape (uint32 index).
    pub fn replace_item(&mut self, item_index: u32, new_item: Item) -> Option<Item> {
        self.update_item(item_index as usize, new_item)
    }

    /// Remove `count` units from the stacked item at `item_index`. For
    /// stackable items the stack count decreases; when it reaches 0 the item
    /// is removed.
    ///
    /// Mirrors C++ `Tile::removeThing(Thing*, uint32_t count)` for the
    /// stackable-item branch. Returns `true` when an item was modified or
    /// removed, `false` when the index was invalid.
    pub fn remove_item_count(&mut self, item_index: usize, count: u32) -> bool {
        let items = self.kind.items_mut();
        let Some(item) = items.get_mut(item_index) else {
            return false;
        };
        if item.is_stackable() {
            let current = item.get_count() as u32;
            if count >= current {
                items.remove(item_index);
            } else {
                let new_count = (current - count) as u8;
                item.set_item_count(new_count);
            }
        } else {
            items.remove(item_index);
        }
        true
    }

    // -----------------------------------------------------------------------
    // has_property / has_property_excluding / has_height — mirrors
    // C++ Tile::hasProperty(ITEMPROPERTY) and hasHeight(uint32_t)
    // -----------------------------------------------------------------------

    /// Whether any item on this tile (ground + stacked) satisfies the given
    /// property.
    ///
    /// Mirrors C++ `bool Tile::hasProperty(ITEMPROPERTY prop) const`.
    pub fn has_property(&self, prop: ItemProperty) -> bool {
        if let Some(g) = &self.ground {
            if item_has_property(g, prop) {
                return true;
            }
        }
        self.kind.items().iter().any(|i| item_has_property(i, prop))
    }

    /// Whether any item on this tile (ground + stacked) other than `exclude`
    /// satisfies `prop`.
    ///
    /// Mirrors C++ `bool Tile::hasProperty(const Item* exclude, ITEMPROPERTY prop) const`.
    /// `exclude_item_index` is the linear stacked-item index to skip; pass
    /// `usize::MAX` to skip nothing.
    pub fn has_property_excluding(&self, exclude_item_index: usize, prop: ItemProperty) -> bool {
        if let Some(g) = &self.ground {
            if item_has_property(g, prop) {
                return true;
            }
        }
        for (i, item) in self.kind.items().iter().enumerate() {
            if i == exclude_item_index {
                continue;
            }
            if item_has_property(item, prop) {
                return true;
            }
        }
        false
    }

    /// Returns `true` when the tile has at least `n` items with `has_height`
    /// stacked on it. Used by movement rules (creatures can't climb >1
    /// height-stack).
    ///
    /// Mirrors C++ `bool Tile::hasHeight(uint32_t n) const`.
    pub fn has_height(&self, n: u32) -> bool {
        let mut count: u32 = 0;
        if let Some(g) = &self.ground {
            if g.has_height() {
                count += 1;
            }
        }
        for item in self.kind.items() {
            if item.has_height() {
                count += 1;
            }
            if count >= n {
                return true;
            }
        }
        count >= n
    }

    // -----------------------------------------------------------------------
    // Visible creatures — mirrors Tile::getTopVisibleCreature /
    // getBottomVisibleCreature / getTopVisibleThing
    //
    // Visibility uses caller-supplied predicates so we don't need to pull
    // Creature into the map crate.
    // -----------------------------------------------------------------------

    /// Returns the bottom (last-added) creature ID on the tile, if any.
    ///
    /// Mirrors C++ `const Creature* Tile::getBottomCreature() const` which
    /// returns the back of the creature list. In Rust we insert at the front,
    /// so the bottom is `creatures().last()`.
    pub fn get_bottom_creature(&self) -> Option<u32> {
        self.kind.creatures().last().copied()
    }

    /// Returns the top creature ID that the supplied predicate accepts
    /// (typically: "can the observer see this creature?"). Iterates from the
    /// front of the creature list (most recently added = top of stack).
    ///
    /// Mirrors C++ `Creature* Tile::getTopVisibleCreature(const Creature* creature) const`
    /// (the visibility decision lives at the call site).
    pub fn get_top_visible_creature<F>(&self, can_see: F) -> Option<u32>
    where
        F: Fn(u32) -> bool,
    {
        self.kind
            .creatures()
            .iter()
            .copied()
            .find(|&id| can_see(id))
    }

    /// Returns the bottom creature ID that the supplied predicate accepts.
    ///
    /// Mirrors C++ `const Creature* Tile::getBottomVisibleCreature(const Creature* creature) const`.
    pub fn get_bottom_visible_creature<F>(&self, can_see: F) -> Option<u32>
    where
        F: Fn(u32) -> bool,
    {
        self.kind
            .creatures()
            .iter()
            .rev()
            .copied()
            .find(|&id| can_see(id))
    }

    // -----------------------------------------------------------------------
    // Stack-pos helpers — used by the wire protocol to address a specific
    // thing within a tile-update packet.
    //
    // The C++ versions take Player* (for client-version-gated visibility);
    // the Rust versions take a `can_see` predicate to keep map crate
    // independent of the Player type.
    // -----------------------------------------------------------------------

    /// Returns the client-side stack position of `creature_id` from the
    /// observer's perspective. Returns `None` when the creature is invisible
    /// to the observer or absent.
    ///
    /// Mirrors C++ `int32_t Tile::getClientIndexOfCreature(const Player*, const Creature*)`.
    pub fn get_client_index_of_creature<F>(&self, creature_id: u32, can_see: F) -> Option<i32>
    where
        F: Fn(u32) -> bool,
    {
        let mut stackpos: i32 = self.ground.is_some() as i32;
        // Top-order items push the creature below them.
        for item in self.kind.items() {
            if item.always_on_top() {
                stackpos += 1;
            }
        }
        // Walk the creature list from top to bottom (front to back).
        for &id in self.kind.creatures() {
            if id == creature_id {
                return Some(stackpos);
            }
            if can_see(id) {
                stackpos += 1;
            }
        }
        None
    }

    /// Returns the client-side stack position of the item at `item_index`
    /// (index into `items()`) from the observer's perspective.
    ///
    /// Mirrors C++ `int32_t Tile::getStackposOfItem(const Player*, const Item*)`.
    pub fn get_stackpos_of_item<F>(&self, item_index: usize, can_see: F) -> Option<i32>
    where
        F: Fn(u32) -> bool,
    {
        let items = self.kind.items();
        if item_index >= items.len() {
            return None;
        }
        let target_item = &items[item_index];
        // Ground takes stackpos 0 when present.
        let mut stackpos: i32 = self.ground.is_some() as i32;
        // Walk items in order; top-order items appear above creatures.
        for (i, item) in items.iter().enumerate() {
            if item.always_on_top() {
                if i == item_index {
                    return Some(stackpos);
                }
                stackpos += 1;
            }
        }
        // Creatures slot between top-order items and down-order items.
        for &id in self.kind.creatures() {
            if can_see(id) {
                stackpos += 1;
            }
        }
        // Then down-order items.
        for (i, item) in items.iter().enumerate() {
            if !item.always_on_top() {
                if i == item_index {
                    return Some(stackpos);
                }
                stackpos += 1;
            }
        }
        // Should be unreachable since we matched on i == item_index above.
        let _ = target_item;
        None
    }

    // -----------------------------------------------------------------------
    // Tile-flag mutation — mirrors private C++ Tile::setTileFlags /
    // resetTileFlags
    //
    // These are invoked internally by addThing/removeThing in C++; we expose
    // them so that callers in the game crate can wire up notification flows.
    // -----------------------------------------------------------------------

    /// Set the tile flags implied by the presence of `item` (e.g. TELEPORT,
    /// MAGICFIELD, MAILBOX, TRASHHOLDER, BED).
    ///
    /// Mirrors C++ `void Tile::setTileFlags(const Item*)`.
    pub fn set_tile_flags_for_item(&mut self, item: &Item) {
        if item.item_type.is_teleport() {
            self.set_flag(flags::TELEPORT);
        }
        if item.item_type.is_magic_field() {
            self.set_flag(flags::MAGICFIELD);
        }
        if item.item_type.is_mailbox() {
            self.set_flag(flags::MAILBOX);
        }
        if item.item_type.is_trash_holder() {
            self.set_flag(flags::TRASHHOLDER);
        }
        if item.item_type.is_bed() {
            self.set_flag(flags::BED);
        }
        if item.item_type.is_depot() {
            self.set_flag(flags::DEPOT);
        }
        if item.block_solid() {
            self.set_flag(flags::BLOCKSOLID);
        }
        if item.block_path_find() {
            self.set_flag(flags::BLOCKPATH);
        }
    }

    /// Reset the tile flags previously implied by `item`. Used when the item
    /// is removed.
    ///
    /// Mirrors C++ `void Tile::resetTileFlags(const Item*)`.
    pub fn reset_tile_flags_for_item(&mut self, item: &Item) {
        if item.item_type.is_teleport() {
            self.reset_flag(flags::TELEPORT);
        }
        if item.item_type.is_magic_field() {
            self.reset_flag(flags::MAGICFIELD);
        }
        if item.item_type.is_mailbox() {
            self.reset_flag(flags::MAILBOX);
        }
        if item.item_type.is_trash_holder() {
            self.reset_flag(flags::TRASHHOLDER);
        }
        if item.item_type.is_bed() {
            self.reset_flag(flags::BED);
        }
        if item.item_type.is_depot() {
            self.reset_flag(flags::DEPOT);
        }
        // Block flags require checking remaining items — only clear if NO
        // remaining item carries them.
        if item.block_solid()
            && !self.kind.items().iter().any(|i| i.block_solid())
            && self.ground.as_ref().is_none_or(|g| !g.block_solid())
        {
            self.reset_flag(flags::BLOCKSOLID);
        }
        if item.block_path_find()
            && !self.kind.items().iter().any(|i| i.block_path_find())
            && self.ground.as_ref().is_none_or(|g| !g.block_path_find())
        {
            self.reset_flag(flags::BLOCKPATH);
        }
    }

    // -----------------------------------------------------------------------
    // Internal add/remove (no spectator dispatch) — mirrors C++
    // Tile::internalAddThing(uint32_t, Thing*)
    // -----------------------------------------------------------------------

    /// Insert an item at the given index (creatures are skipped; index is
    /// into the stacked-item list).  Top-order items naturally land at the
    /// end of the items list. No spectator dispatch is performed.
    ///
    /// Mirrors C++ `void Tile::internalAddThing(uint32_t, Thing*)` for the
    /// item path.
    pub fn internal_add_item(&mut self, item: Item) {
        if item.always_on_top() {
            // Top-order items go at the end of the items vec.
            self.kind.items_mut().push(item);
        } else {
            // Down-order items go at the front.
            self.kind.items_mut().insert(0, item);
        }
    }

    // -----------------------------------------------------------------------
    // Cylinder dispatch (post_add / post_remove)
    //
    // C++ wires spectator updates here; we expose stub methods so callers
    // higher up can run the relevant pre/post hooks. The default impl is a
    // no-op; the spectator system lives in the world/game crates.
    // -----------------------------------------------------------------------

    /// Stub hook called after an item is added to this tile. Default no-op.
    pub fn post_add_item(&mut self, _item_id: u16, _index: i32) {}

    /// Stub hook called after an item is removed from this tile. Default no-op.
    pub fn post_remove_item(&mut self, _item_id: u16, _index: i32) {}
}

// ---------------------------------------------------------------------------
// TileThingRef — typed enum for "the thing at index i on the tile"
// ---------------------------------------------------------------------------

/// A reference to one of the addressable things on a tile, returned by
/// [`Tile::get_thing_at_index`].
#[derive(Debug)]
pub enum TileThingRef<'a> {
    Creature(u32),
    Ground(&'a Item),
    Item(&'a Item),
}

/// Helper to test whether a single item satisfies an [`ItemProperty`].
///
/// Mirrors C++ inline checks against `ItemType` flags in `Item::hasProperty`.
fn item_has_property(item: &Item, prop: ItemProperty) -> bool {
    match prop {
        ItemProperty::BlockSolid => item.block_solid(),
        ItemProperty::HasHeight => item.has_height(),
        ItemProperty::BlockProjectile => item.block_projectile(),
        ItemProperty::BlockPath => item.block_path_find(),
        ItemProperty::Moveable => item.is_moveable(),
        ItemProperty::ImmovableBlockSolid => item.block_solid() && !item.is_moveable(),
        ItemProperty::ImmovableBlockPath => item.block_path_find() && !item.is_moveable(),
        ItemProperty::ImmovableNoFieldBlockPath => {
            item.block_path_find() && !item.is_moveable() && !item.item_type.is_magic_field()
        }
        ItemProperty::NoFieldBlockPath => {
            item.block_path_find() && !item.item_type.is_magic_field()
        }
        // IsVertical / IsHorizontal / SupportHangable are item-type flags
        // that are not yet modelled in ItemTypeData; surface as `false` for
        // now. The C++ defaults are also `false` for items lacking the flag.
        ItemProperty::IsVertical | ItemProperty::IsHorizontal | ItemProperty::SupportHangable => {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Cylinder trait impl — cross-crate dispatch for Tile
// ---------------------------------------------------------------------------

impl forgottenserver_common::thing::Thing for Tile {
    fn is_removed(&self) -> bool {
        false
    }
    fn is_ground(&self) -> bool {
        self.ground.is_some()
    }
    fn get_first_index(&self) -> usize {
        0
    }
    fn get_last_index(&self) -> usize {
        self.get_thing_count()
    }
    fn get_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
        Tile::get_item_type_count(self, item_id, sub_type)
    }
}

impl forgottenserver_common::cylinder::Cylinder for Tile {
    fn cylinder_first_index(&self) -> usize {
        0
    }
    fn cylinder_last_index(&self) -> usize {
        self.get_thing_count()
    }
    fn cylinder_item_type_count(&self, item_id: u16, sub_type: i32) -> u32 {
        Tile::get_item_type_count(self, item_id, sub_type)
    }
    fn cylinder_query_add(
        &self,
        _index: i32,
        _thing: &dyn forgottenserver_common::thing::Thing,
        _count: u32,
        flags: u32,
    ) -> forgottenserver_common::enums::ReturnValue {
        // Map common::ReturnValue from the existing tile-local enum via the
        // inherent `query_add`.
        match Tile::query_add(self, flags) {
            ReturnValue::NoError => forgottenserver_common::enums::ReturnValue::NoError,
            ReturnValue::NotPossible => forgottenserver_common::enums::ReturnValue::NotPossible,
            ReturnValue::NotEnoughRoom => forgottenserver_common::enums::ReturnValue::NotEnoughRoom,
            ReturnValue::NotMoveable => forgottenserver_common::enums::ReturnValue::NotMoveable,
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
        // Tiles do not have a fixed capacity for stacked items beyond the
        // 0xFFFF overflow guard; report up to 0xFFFF free slots.
        let free = (0xFFFFu32).saturating_sub(self.get_item_count() as u32);
        (rv, count.min(free))
    }
    fn cylinder_query_remove(
        &self,
        _thing: &dyn forgottenserver_common::thing::Thing,
        count: u32,
        flags: u32,
    ) -> forgottenserver_common::enums::ReturnValue {
        // Without index resolution at the trait level we report removability
        // of the top down-item, which matches the most common cylinder
        // dispatch use case.
        if self.kind.items().is_empty() || count == 0 {
            return forgottenserver_common::enums::ReturnValue::NotPossible;
        }
        match Tile::query_remove(self, 0, count, flags) {
            ReturnValue::NoError => forgottenserver_common::enums::ReturnValue::NoError,
            ReturnValue::NotPossible => forgottenserver_common::enums::ReturnValue::NotPossible,
            ReturnValue::NotEnoughRoom => forgottenserver_common::enums::ReturnValue::NotEnoughRoom,
            ReturnValue::NotMoveable => forgottenserver_common::enums::ReturnValue::NotMoveable,
            _ => forgottenserver_common::enums::ReturnValue::NotPossible,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::flags::*;
    use super::*;
    use std::sync::Arc;

    use forgottenserver_items::items_registry::ItemTypeData;

    fn make_item(id: u16) -> Item {
        let data = ItemTypeData {
            id,
            ..Default::default()
        };
        Item::new(Arc::new(data), 1)
    }

    // -----------------------------------------------------------------------
    // Flag bit values
    // -----------------------------------------------------------------------

    #[test]
    fn flag_protection_zone_bit() {
        assert_eq!(PROTECTIONZONE, 1 << 7);
    }

    #[test]
    fn flag_nopvpzone_bit() {
        assert_eq!(NOPVPZONE, 1 << 8);
    }

    #[test]
    fn flag_nologout_bit() {
        assert_eq!(NOLOGOUT, 1 << 9);
    }

    #[test]
    fn flag_pvpzone_bit() {
        assert_eq!(PVPZONE, 1 << 10);
    }

    #[test]
    fn flag_floorchange_down_bit() {
        assert_eq!(FLOORCHANGE_DOWN, 1 << 0);
    }

    #[test]
    fn flag_floorchange_north_bit() {
        assert_eq!(FLOORCHANGE_NORTH, 1 << 1);
    }

    #[test]
    fn flag_floorchange_south_bit() {
        assert_eq!(FLOORCHANGE_SOUTH, 1 << 2);
    }

    #[test]
    fn flag_floorchange_east_bit() {
        assert_eq!(FLOORCHANGE_EAST, 1 << 3);
    }

    #[test]
    fn flag_floorchange_west_bit() {
        assert_eq!(FLOORCHANGE_WEST, 1 << 4);
    }

    #[test]
    fn flag_floorchange_south_alt_bit() {
        assert_eq!(FLOORCHANGE_SOUTH_ALT, 1 << 5);
    }

    #[test]
    fn flag_floorchange_east_alt_bit() {
        assert_eq!(FLOORCHANGE_EAST_ALT, 1 << 6);
    }

    #[test]
    fn flag_teleport_bit() {
        assert_eq!(TELEPORT, 1 << 11);
    }

    #[test]
    fn flag_magicfield_bit() {
        assert_eq!(MAGICFIELD, 1 << 12);
    }

    #[test]
    fn flag_mailbox_bit() {
        assert_eq!(MAILBOX, 1 << 13);
    }

    #[test]
    fn flag_trashholder_bit() {
        assert_eq!(TRASHHOLDER, 1 << 14);
    }

    #[test]
    fn flag_bed_bit() {
        assert_eq!(BED, 1 << 15);
    }

    #[test]
    fn flag_depot_bit() {
        assert_eq!(DEPOT, 1 << 16);
    }

    #[test]
    fn flag_blocksolid_bit() {
        assert_eq!(BLOCKSOLID, 1 << 17);
    }

    #[test]
    fn flag_blockpath_bit() {
        assert_eq!(BLOCKPATH, 1 << 18);
    }

    #[test]
    fn flag_floorchange_composite() {
        let composite = FLOORCHANGE;
        assert!(composite & FLOORCHANGE_DOWN != 0);
        assert!(composite & FLOORCHANGE_NORTH != 0);
        assert!(composite & FLOORCHANGE_SOUTH != 0);
        assert!(composite & FLOORCHANGE_EAST != 0);
        assert!(composite & FLOORCHANGE_WEST != 0);
        assert!(composite & FLOORCHANGE_SOUTH_ALT != 0);
        assert!(composite & FLOORCHANGE_EAST_ALT != 0);
    }

    // -----------------------------------------------------------------------
    // Tile::new — position, no flags
    // -----------------------------------------------------------------------

    #[test]
    fn tile_new_creates_at_position() {
        let t = Tile::new(100, 200, 7);
        assert_eq!(t.position.x, 100);
        assert_eq!(t.position.y, 200);
        assert_eq!(t.position.z, 7);
    }

    #[test]
    fn tile_new_has_no_flags() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.flags, NONE);
    }

    #[test]
    fn tile_new_is_dynamic() {
        let t = Tile::new_dynamic(0, 0, 0);
        assert!(t.is_dynamic());
        assert!(!t.is_static());
    }

    #[test]
    fn tile_static_is_static() {
        let t = Tile::new_static(0, 0, 0);
        assert!(t.is_static());
        assert!(!t.is_dynamic());
    }

    // -----------------------------------------------------------------------
    // set_flag / has_flag / reset_flag
    // -----------------------------------------------------------------------

    #[test]
    fn set_and_has_flag() {
        let mut t = Tile::new(0, 0, 0);
        assert!(!t.has_flag(PROTECTIONZONE));
        t.set_flag(PROTECTIONZONE);
        assert!(t.has_flag(PROTECTIONZONE));
    }

    #[test]
    fn reset_flag_clears() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(PROTECTIONZONE);
        t.reset_flag(PROTECTIONZONE);
        assert!(!t.has_flag(PROTECTIONZONE));
    }

    #[test]
    fn set_multiple_flags_independently() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(BLOCKSOLID);
        t.set_flag(BLOCKPATH);
        assert!(t.has_flag(BLOCKSOLID));
        assert!(t.has_flag(BLOCKPATH));
        t.reset_flag(BLOCKSOLID);
        assert!(!t.has_flag(BLOCKSOLID));
        assert!(t.has_flag(BLOCKPATH));
    }

    // -----------------------------------------------------------------------
    // get_zone
    // -----------------------------------------------------------------------

    #[test]
    fn zone_protection_when_flag_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(PROTECTIONZONE);
        assert_eq!(t.get_zone(), ZoneType::Protection);
    }

    #[test]
    fn zone_nopvp_when_flag_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(NOPVPZONE);
        assert_eq!(t.get_zone(), ZoneType::NoPvp);
    }

    #[test]
    fn zone_pvp_when_flag_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(PVPZONE);
        assert_eq!(t.get_zone(), ZoneType::Pvp);
    }

    #[test]
    fn zone_nologout_when_flag_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(NOLOGOUT);
        assert_eq!(t.get_zone(), ZoneType::NoLogout);
    }

    #[test]
    fn zone_normal_when_no_zone_flags() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.get_zone(), ZoneType::Normal);
    }

    #[test]
    fn zone_protection_has_priority_over_nopvp() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(PROTECTIONZONE | NOPVPZONE);
        assert_eq!(t.get_zone(), ZoneType::Protection);
    }

    #[test]
    fn zone_nopvp_has_priority_over_pvp() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(NOPVPZONE | PVPZONE);
        assert_eq!(t.get_zone(), ZoneType::NoPvp);
    }

    // -----------------------------------------------------------------------
    // is_protection_zone / has_floor_change
    // -----------------------------------------------------------------------

    #[test]
    fn is_protection_zone_true_when_flag_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(PROTECTIONZONE);
        assert!(t.is_protection_zone());
    }

    #[test]
    fn is_protection_zone_false_when_not_set() {
        let t = Tile::new(0, 0, 0);
        assert!(!t.is_protection_zone());
    }

    #[test]
    fn has_floor_change_false_when_no_flags() {
        let t = Tile::new(0, 0, 0);
        assert!(!t.has_floor_change());
    }

    #[test]
    fn has_floor_change_true_when_down_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(FLOORCHANGE_DOWN);
        assert!(t.has_floor_change());
    }

    #[test]
    fn has_floor_change_true_when_north_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(FLOORCHANGE_NORTH);
        assert!(t.has_floor_change());
    }

    #[test]
    fn has_floor_change_true_when_south_alt_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(FLOORCHANGE_SOUTH_ALT);
        assert!(t.has_floor_change());
    }

    #[test]
    fn has_floor_change_true_when_east_alt_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(FLOORCHANGE_EAST_ALT);
        assert!(t.has_floor_change());
    }

    // -----------------------------------------------------------------------
    // Ground item
    // -----------------------------------------------------------------------

    #[test]
    fn get_ground_returns_none_by_default() {
        let t = Tile::new(0, 0, 0);
        assert!(t.get_ground().is_none());
    }

    #[test]
    fn set_ground_and_get_ground() {
        let mut t = Tile::new(0, 0, 0);
        let item = make_item(10);
        t.set_ground(item);
        assert!(t.get_ground().is_some());
        assert_eq!(t.get_ground().unwrap().get_id(), 10);
    }

    #[test]
    fn take_ground_returns_item_and_clears() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(5));
        let taken = t.take_ground();
        assert!(taken.is_some());
        assert!(t.get_ground().is_none());
    }

    // -----------------------------------------------------------------------
    // Item list
    // -----------------------------------------------------------------------

    #[test]
    fn get_item_count_zero_initially() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.get_item_count(), 0);
    }

    #[test]
    fn add_item_increases_count() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(1));
        assert_eq!(t.get_item_count(), 1);
        t.add_item(make_item(2));
        assert_eq!(t.get_item_count(), 2);
    }

    #[test]
    fn remove_item_at_index() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(1));
        t.add_item(make_item(2));
        let removed = t.remove_item(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get_id(), 1);
        assert_eq!(t.get_item_count(), 1);
    }

    #[test]
    fn remove_item_invalid_index_returns_none() {
        let mut t = Tile::new(0, 0, 0);
        assert!(t.remove_item(0).is_none());
    }

    // -----------------------------------------------------------------------
    // Creature list
    // -----------------------------------------------------------------------

    #[test]
    fn get_creature_count_zero_initially() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.get_creature_count(), 0);
    }

    #[test]
    fn add_creature_increases_count() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(42);
        assert_eq!(t.get_creature_count(), 1);
    }

    #[test]
    fn remove_creature_existing_returns_true() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(10);
        t.add_creature(20);
        assert!(t.remove_creature(10));
        assert_eq!(t.get_creature_count(), 1);
    }

    #[test]
    fn remove_creature_nonexistent_returns_false() {
        let mut t = Tile::new(0, 0, 0);
        assert!(!t.remove_creature(99));
    }

    // -----------------------------------------------------------------------
    // get_thing_count
    // -----------------------------------------------------------------------

    #[test]
    fn thing_count_empty_tile() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.get_thing_count(), 0);
    }

    #[test]
    fn thing_count_creatures_plus_items_plus_ground() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.add_item(make_item(2));
        t.add_item(make_item(3));
        t.add_creature(100);
        // ground(1) + items(2) + creatures(1) = 4
        assert_eq!(t.get_thing_count(), 4);
    }

    #[test]
    fn thing_count_no_ground() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(2));
        t.add_creature(1);
        assert_eq!(t.get_thing_count(), 2);
    }

    // -----------------------------------------------------------------------
    // StaticTile — lazy allocation
    // -----------------------------------------------------------------------

    #[test]
    fn static_tile_items_start_as_none() {
        let t = Tile::new_static(0, 0, 0);
        assert_eq!(t.get_item_count(), 0);
    }

    #[test]
    fn static_tile_allocates_on_first_add() {
        let mut t = Tile::new_static(0, 0, 0);
        t.add_item(make_item(1));
        assert_eq!(t.get_item_count(), 1);
    }

    #[test]
    fn static_tile_creatures_lazy() {
        let mut t = Tile::new_static(0, 0, 0);
        t.add_creature(7);
        assert_eq!(t.get_creature_count(), 1);
    }

    // -----------------------------------------------------------------------
    // add_creature inserts at front (mirrors C++ creatures->insert(begin(), …))
    // -----------------------------------------------------------------------

    #[test]
    fn add_creature_inserts_at_front() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(10);
        t.add_creature(20);
        t.add_creature(30);
        // Last added should be at index 0 (front)
        assert_eq!(t.creatures()[0], 30);
        assert_eq!(t.creatures()[1], 20);
        assert_eq!(t.creatures()[2], 10);
    }

    // -----------------------------------------------------------------------
    // get_top_creature — mirrors Tile::getTopCreature
    // -----------------------------------------------------------------------

    #[test]
    fn get_top_creature_none_when_empty() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.get_top_creature(), None);
    }

    #[test]
    fn get_top_creature_returns_first_in_list() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(1);
        assert_eq!(t.get_top_creature(), Some(1));
    }

    #[test]
    fn get_top_creature_is_last_added() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(1);
        t.add_creature(2);
        // Since add_creature inserts at front, last added = top creature
        assert_eq!(t.get_top_creature(), Some(2));
    }

    #[test]
    fn get_top_creature_after_remove() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(1);
        t.add_creature(2);
        t.remove_creature(2); // remove top
        assert_eq!(t.get_top_creature(), Some(1));
    }

    // -----------------------------------------------------------------------
    // get_top_item — last item in the list (top of stack)
    // -----------------------------------------------------------------------

    #[test]
    fn get_top_item_none_when_no_items() {
        let t = Tile::new(0, 0, 0);
        assert!(t.get_top_item().is_none());
    }

    #[test]
    fn get_top_item_returns_last_item() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(1));
        t.add_item(make_item(2));
        t.add_item(make_item(3));
        assert_eq!(t.get_top_item().unwrap().get_id(), 3);
    }

    #[test]
    fn get_top_down_item_returns_first_item() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(10));
        t.add_item(make_item(20));
        assert_eq!(t.get_top_down_item().unwrap().get_id(), 10);
    }

    #[test]
    fn get_top_down_item_none_when_no_items() {
        let t = Tile::new(0, 0, 0);
        assert!(t.get_top_down_item().is_none());
    }

    // -----------------------------------------------------------------------
    // is_block_solid / is_block_projectile / is_block_path_finder
    // -----------------------------------------------------------------------

    #[test]
    fn is_block_solid_false_by_default() {
        let t = Tile::new(0, 0, 0);
        assert!(!t.is_block_solid());
    }

    #[test]
    fn is_block_solid_true_when_flag_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(BLOCKSOLID);
        assert!(t.is_block_solid());
    }

    #[test]
    fn is_block_path_finder_false_by_default() {
        let t = Tile::new(0, 0, 0);
        assert!(!t.is_block_path_finder());
    }

    #[test]
    fn is_block_path_finder_true_when_flag_set() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(BLOCKPATH);
        assert!(t.is_block_path_finder());
    }

    #[test]
    fn is_block_projectile_false_when_no_blocking_item() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1)); // default item_type has block_projectile = false
        assert!(!t.is_block_projectile());
    }

    #[test]
    fn is_block_projectile_true_when_ground_blocks() {
        let mut t = Tile::new(0, 0, 0);
        let data = ItemTypeData {
            id: 99,
            block_projectile: true,
            ..Default::default()
        };
        let item = Item::new(Arc::new(data), 1);
        t.set_ground(item);
        assert!(t.is_block_projectile());
    }

    #[test]
    fn is_block_projectile_true_when_stacked_item_blocks() {
        let mut t = Tile::new(0, 0, 0);
        let data = ItemTypeData {
            id: 50,
            block_projectile: true,
            ..Default::default()
        };
        t.add_item(Item::new(Arc::new(data), 1));
        assert!(t.is_block_projectile());
    }

    // -----------------------------------------------------------------------
    // is_moveable_blocking — mirrors C++ Tile::isMoveableBlocking
    // -----------------------------------------------------------------------

    #[test]
    fn is_moveable_blocking_true_when_no_ground() {
        let t = Tile::new(0, 0, 0);
        assert!(t.is_moveable_blocking());
    }

    #[test]
    fn is_moveable_blocking_false_when_ground_and_no_blocksolid() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        assert!(!t.is_moveable_blocking());
    }

    #[test]
    fn is_moveable_blocking_true_when_blocksolid_even_with_ground() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.set_flag(BLOCKSOLID);
        assert!(t.is_moveable_blocking());
    }

    // -----------------------------------------------------------------------
    // House tile flag
    // -----------------------------------------------------------------------

    #[test]
    fn tile_no_house_by_default() {
        let t = Tile::new(0, 0, 0);
        assert!(!t.is_house_tile());
        assert_eq!(t.get_house_id(), None);
    }

    #[test]
    fn set_house_id_marks_as_house_tile() {
        let mut t = Tile::new(0, 0, 0);
        t.set_house_id(42);
        assert!(t.is_house_tile());
        assert_eq!(t.get_house_id(), Some(42));
    }

    #[test]
    fn clear_house_id_removes_house_flag() {
        let mut t = Tile::new(0, 0, 0);
        t.set_house_id(7);
        t.clear_house_id();
        assert!(!t.is_house_tile());
        assert_eq!(t.get_house_id(), None);
    }

    #[test]
    fn house_id_zero_is_valid() {
        let mut t = Tile::new(0, 0, 0);
        t.set_house_id(0);
        assert!(t.is_house_tile());
        assert_eq!(t.get_house_id(), Some(0));
    }

    // -----------------------------------------------------------------------
    // query_add — NOLIMIT / IGNORECHECKS bypass
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_nolimit_bypasses_no_ground() {
        let t = Tile::new(0, 0, 0); // no ground
        assert_eq!(t.query_add(query_flags::NOLIMIT), ReturnValue::NoError);
    }

    #[test]
    fn query_add_ignorechecks_bypasses_no_ground() {
        let t = Tile::new(0, 0, 0); // no ground
        assert_eq!(t.query_add(query_flags::IGNORECHECKS), ReturnValue::NoError);
    }

    #[test]
    fn query_add_no_flags_no_ground_returns_not_possible() {
        let t = Tile::new(0, 0, 0); // no ground
        assert_eq!(t.query_add(0), ReturnValue::NotPossible);
    }

    #[test]
    fn query_add_no_flags_with_ground_returns_ok() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        assert_eq!(t.query_add(0), ReturnValue::NoError);
    }

    // -----------------------------------------------------------------------
    // query_add — PATHFINDING blocks on floor change / teleport
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_pathfinding_blocked_by_floorchange() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.set_flag(FLOORCHANGE_DOWN);
        assert_eq!(
            t.query_add(query_flags::PATHFINDING),
            ReturnValue::NotPossible
        );
    }

    #[test]
    fn query_add_pathfinding_blocked_by_teleport() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.set_flag(TELEPORT);
        assert_eq!(
            t.query_add(query_flags::PATHFINDING),
            ReturnValue::NotPossible
        );
    }

    #[test]
    fn query_add_pathfinding_ok_when_no_floorchange_teleport() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        assert_eq!(t.query_add(query_flags::PATHFINDING), ReturnValue::NoError);
    }

    #[test]
    fn query_add_nolimit_bypasses_pathfinding_floorchange() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(FLOORCHANGE_DOWN);
        assert_eq!(
            t.query_add(query_flags::NOLIMIT | query_flags::PATHFINDING),
            ReturnValue::NoError
        );
    }

    // -----------------------------------------------------------------------
    // query_add — BLOCKSOLID denial
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_blocked_by_blocksolid() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.set_flag(BLOCKSOLID);
        assert_eq!(t.query_add(0), ReturnValue::NotEnoughRoom);
    }

    #[test]
    fn query_add_ignoreblockitem_bypasses_blocksolid_flag() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.set_flag(BLOCKSOLID);
        // With IGNOREBLOCKITEM, the flag is bypassed; items are checked individually
        // Ground item has block_solid=false by default, so should be ok
        assert_eq!(
            t.query_add(query_flags::IGNOREBLOCKITEM),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_ignoreblockitem_still_blocks_immoveable_solid_ground() {
        let data = ItemTypeData {
            id: 10,
            block_solid: true,
            moveable: false, // immoveable
            ..Default::default()
        };
        let ground_item = Item::new(Arc::new(data), 1);

        let mut t = Tile::new(0, 0, 0);
        t.set_ground(ground_item);
        // Even with IGNOREBLOCKITEM, immoveable solid ground still blocks
        assert_eq!(
            t.query_add(query_flags::IGNOREBLOCKITEM),
            ReturnValue::NotPossible
        );
    }

    #[test]
    fn query_add_ignoreblockitem_still_blocks_immoveable_solid_item() {
        let ground_data = ItemTypeData {
            id: 1,
            ..Default::default()
        };
        let ground = Item::new(Arc::new(ground_data), 1);

        let item_data = ItemTypeData {
            id: 99,
            block_solid: true,
            moveable: false,
            ..Default::default()
        };
        let blocking_item = Item::new(Arc::new(item_data), 1);

        let mut t = Tile::new(0, 0, 0);
        t.set_ground(ground);
        t.add_item(blocking_item);
        assert_eq!(
            t.query_add(query_flags::IGNOREBLOCKITEM),
            ReturnValue::NotPossible
        );
    }

    // -----------------------------------------------------------------------
    // query_remove — basic cases
    // -----------------------------------------------------------------------

    #[test]
    fn query_remove_count_zero_returns_not_possible() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(1));
        assert_eq!(t.query_remove(0, 0, 0), ReturnValue::NotPossible);
    }

    #[test]
    fn query_remove_invalid_index_returns_not_possible() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.query_remove(0, 1, 0), ReturnValue::NotPossible);
    }

    #[test]
    fn query_remove_moveable_item_ok() {
        let data = ItemTypeData {
            id: 5,
            moveable: true,
            ..Default::default()
        };
        let item = Item::new(Arc::new(data), 1);

        let mut t = Tile::new(0, 0, 0);
        t.add_item(item);
        assert_eq!(t.query_remove(0, 1, 0), ReturnValue::NoError);
    }

    #[test]
    fn query_remove_immoveable_returns_not_moveable() {
        let data = ItemTypeData {
            id: 5,
            moveable: false,
            ..Default::default()
        };
        let item = Item::new(Arc::new(data), 1);

        let mut t = Tile::new(0, 0, 0);
        t.add_item(item);
        assert_eq!(t.query_remove(0, 1, 0), ReturnValue::NotMoveable);
    }

    #[test]
    fn query_remove_immoveable_bypassed_by_ignorenotmoveable() {
        let data = ItemTypeData {
            id: 5,
            moveable: false,
            ..Default::default()
        };
        let item = Item::new(Arc::new(data), 1);

        let mut t = Tile::new(0, 0, 0);
        t.add_item(item);
        assert_eq!(
            t.query_remove(0, 1, query_flags::IGNORENOTMOVEABLE),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_remove_stackable_count_exceeds_stack_returns_not_possible() {
        let data = ItemTypeData {
            id: 5,
            stackable: true,
            moveable: true,
            ..Default::default()
        };
        let mut item = Item::new(Arc::new(data), 1);
        item.set_item_count(5);

        let mut t = Tile::new(0, 0, 0);
        t.add_item(item);
        // Trying to remove 10 when only 5 present
        assert_eq!(t.query_remove(0, 10, 0), ReturnValue::NotPossible);
    }

    #[test]
    fn query_remove_stackable_exact_count_ok() {
        let data = ItemTypeData {
            id: 5,
            stackable: true,
            moveable: true,
            ..Default::default()
        };
        let mut item = Item::new(Arc::new(data), 1);
        item.set_item_count(5);

        let mut t = Tile::new(0, 0, 0);
        t.add_item(item);
        assert_eq!(t.query_remove(0, 5, 0), ReturnValue::NoError);
    }

    // -----------------------------------------------------------------------
    // serialize / deserialize round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn serialize_deserialize_empty_tile_roundtrip() {
        let t = Tile::new(10, 20, 7);
        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("deserialize ok");
        assert_eq!(t2.position.x, 10);
        assert_eq!(t2.position.y, 20);
        assert_eq!(t2.position.z, 7);
        assert_eq!(t2.flags, 0);
        assert!(t2.get_ground().is_none());
        assert_eq!(t2.get_item_count(), 0);
        assert_eq!(t2.get_creature_count(), 0);
        assert_eq!(t2.get_house_id(), None);
    }

    #[test]
    fn serialize_deserialize_with_ground_roundtrip() {
        let mut t = Tile::new(1, 2, 3);
        t.set_ground(make_item(42));
        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("ok");
        assert!(t2.get_ground().is_some());
        assert_eq!(t2.get_ground().unwrap().get_id(), 42);
    }

    #[test]
    fn serialize_deserialize_with_items_roundtrip() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(10));
        t.add_item(make_item(20));
        t.add_item(make_item(30));
        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("ok");
        assert_eq!(t2.get_item_count(), 3);
        assert_eq!(t2.items()[0].get_id(), 10);
        assert_eq!(t2.items()[1].get_id(), 20);
        assert_eq!(t2.items()[2].get_id(), 30);
    }

    #[test]
    fn serialize_deserialize_with_creatures_roundtrip() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(100);
        t.add_creature(200);
        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("ok");
        assert_eq!(t2.get_creature_count(), 2);
        // creatures are serialized in stored order (front-first after insert-at-front)
        assert!(t2.creatures().contains(&100));
        assert!(t2.creatures().contains(&200));
    }

    #[test]
    fn serialize_deserialize_with_flags_roundtrip() {
        let mut t = Tile::new(0, 0, 0);
        t.set_flag(PROTECTIONZONE | BLOCKSOLID | BLOCKPATH);
        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("ok");
        assert!(t2.has_flag(PROTECTIONZONE));
        assert!(t2.has_flag(BLOCKSOLID));
        assert!(t2.has_flag(BLOCKPATH));
    }

    #[test]
    fn serialize_deserialize_house_id_present_roundtrip() {
        let mut t = Tile::new(5, 6, 7);
        t.set_house_id(99);
        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("ok");
        assert!(t2.is_house_tile());
        assert_eq!(t2.get_house_id(), Some(99));
    }

    #[test]
    fn serialize_deserialize_no_house_roundtrip() {
        let t = Tile::new(0, 0, 0);
        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("ok");
        assert!(!t2.is_house_tile());
    }

    #[test]
    fn deserialize_truncated_returns_error() {
        let blob = vec![0x01, 0x00]; // too short
        let result = Tile::deserialize(&blob, make_item);
        assert!(result.is_err());
    }

    #[test]
    fn serialize_deserialize_full_tile_roundtrip() {
        let mut t = Tile::new(100, 200, 7);
        t.set_ground(make_item(1));
        t.add_item(make_item(5));
        t.add_item(make_item(6));
        t.add_creature(42);
        t.add_creature(43);
        t.set_flag(PROTECTIONZONE | TELEPORT);
        t.set_house_id(77);

        let blob = t.serialize();
        let t2 = Tile::deserialize(&blob, make_item).expect("full roundtrip ok");

        assert_eq!(t2.position.x, 100);
        assert_eq!(t2.position.y, 200);
        assert_eq!(t2.position.z, 7);
        assert!(t2.get_ground().is_some());
        assert_eq!(t2.get_item_count(), 2);
        assert_eq!(t2.get_creature_count(), 2);
        assert!(t2.has_flag(PROTECTIONZONE));
        assert!(t2.has_flag(TELEPORT));
        assert_eq!(t2.get_house_id(), Some(77));
    }

    // -----------------------------------------------------------------------
    // ReturnValue helpers
    // -----------------------------------------------------------------------

    #[test]
    fn return_value_is_ok() {
        assert!(ReturnValue::NoError.is_ok());
        assert!(!ReturnValue::NotPossible.is_ok());
        assert!(!ReturnValue::NotEnoughRoom.is_ok());
        assert!(!ReturnValue::NotMoveable.is_ok());
    }

    // -----------------------------------------------------------------------
    // QueryFlag bit values
    // -----------------------------------------------------------------------

    #[test]
    fn query_flag_nolimit_bit() {
        assert_eq!(query_flags::NOLIMIT, 1 << 0);
    }

    #[test]
    fn query_flag_pathfinding_bit() {
        assert_eq!(query_flags::PATHFINDING, 1 << 1);
    }

    #[test]
    fn query_flag_ignoreblockcreature_bit() {
        assert_eq!(query_flags::IGNOREBLOCKCREATURE, 1 << 2);
    }

    #[test]
    fn query_flag_ignoreblockitem_bit() {
        assert_eq!(query_flags::IGNOREBLOCKITEM, 1 << 3);
    }

    #[test]
    fn query_flag_ignorechecks_bit() {
        assert_eq!(query_flags::IGNORECHECKS, 1 << 4);
    }

    #[test]
    fn query_flag_ignorenotmoveable_bit() {
        assert_eq!(query_flags::IGNORENOTMOVEABLE, 1 << 5);
    }

    // -----------------------------------------------------------------------
    // query_add — overflow guard, IGNOREBLOCKITEM no-ground / pass-through
    // (covers tile.rs lines 531, 545, 549)
    // -----------------------------------------------------------------------

    #[test]
    fn query_add_returns_not_possible_when_item_count_at_limit() {
        // C++ tile.cpp: `if (items && items->size() >= 0xFFFF) return NOTPOSSIBLE;`
        // We populate the items vector directly to avoid 65535 allocations,
        // using a single shared Arc<ItemTypeData> for all entries.
        let data = Arc::new(ItemTypeData::default());
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        // Inflate the items vec to exactly 0xFFFF entries.
        let items = t.kind.items_mut();
        items.reserve(0xFFFF);
        for _ in 0..0xFFFF {
            items.push(Item::new(Arc::clone(&data), 1));
        }
        assert_eq!(t.get_item_count(), 0xFFFF);
        assert_eq!(t.query_add(0), ReturnValue::NotPossible);
    }

    #[test]
    fn query_add_ignoreblockitem_with_no_ground_is_ok() {
        // C++ tile.cpp: when FLAG_IGNOREBLOCKITEM is set, the no-ground bail
        // does not fire (only the immoveable-solid checks run).  An empty
        // tile with IGNOREBLOCKITEM should therefore return NoError.
        let t = Tile::new(0, 0, 0);
        assert!(t.get_ground().is_none());
        assert_eq!(
            t.query_add(query_flags::IGNOREBLOCKITEM),
            ReturnValue::NoError
        );
    }

    #[test]
    fn query_add_ignoreblockitem_passes_when_item_is_moveable_solid() {
        // The items loop must visit every item.  A moveable solid item does
        // not satisfy `block_solid && !is_moveable`, so the loop body falls
        // through and the function returns NoError.
        let ground_data = ItemTypeData {
            id: 1,
            ..Default::default()
        };
        let stacked = ItemTypeData {
            id: 2,
            block_solid: true,
            moveable: true, // moveable solid is allowed under IGNOREBLOCKITEM
            ..Default::default()
        };
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(Item::new(Arc::new(ground_data), 1));
        t.add_item(Item::new(Arc::new(stacked), 1));
        assert_eq!(
            t.query_add(query_flags::IGNOREBLOCKITEM),
            ReturnValue::NoError
        );
    }

    // -----------------------------------------------------------------------
    // Phase A.3 — C++ parity API tests
    // -----------------------------------------------------------------------

    use forgottenserver_common::cylinder::Cylinder as CommonCylinder;
    use forgottenserver_common::enums::ReturnValue as CommonRv;
    use forgottenserver_common::thing::DefaultThing;
    use forgottenserver_items::items_registry::ItemTypeKind;

    fn item_with(
        id: u16,
        build: impl FnOnce(&mut forgottenserver_items::items_registry::ItemTypeData),
    ) -> Item {
        let mut data = forgottenserver_items::items_registry::ItemTypeData {
            id,
            ..Default::default()
        };
        build(&mut data);
        Item::new(Arc::new(data), 1)
    }

    fn item_with_count(
        id: u16,
        count: u8,
        build: impl FnOnce(&mut forgottenserver_items::items_registry::ItemTypeData),
    ) -> Item {
        let mut data = forgottenserver_items::items_registry::ItemTypeData {
            id,
            ..Default::default()
        };
        build(&mut data);
        Item::new(Arc::new(data), count)
    }

    fn _item_with_helper_alias() {}

    // --- top / down counts -------------------------------------------------

    #[test]
    fn test_get_top_item_count_zero_when_empty() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.get_top_item_count(), 0);
        assert_eq!(t.get_down_item_count(), 0);
    }

    #[test]
    fn test_get_top_item_count_counts_only_always_on_top() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(1, |d| d.always_on_top = true));
        t.add_item(item_with(2, |d| d.always_on_top = false));
        t.add_item(item_with(3, |d| d.always_on_top = true));
        assert_eq!(t.get_top_item_count(), 2);
        assert_eq!(t.get_down_item_count(), 1);
    }

    // --- get_first_index / get_last_index_inclusive ------------------------

    #[test]
    fn test_get_first_index_always_zero() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(t.get_first_index(), 0);
    }

    #[test]
    fn test_get_last_index_inclusive_matches_thing_count() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.add_item(make_item(2));
        t.add_creature(42);
        assert_eq!(t.get_last_index_inclusive(), t.get_thing_count());
    }

    // --- special-item lookups ---------------------------------------------

    #[test]
    fn test_get_field_item_returns_first_magic_field() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(1, |d| d.type_kind = ItemTypeKind::None));
        t.add_item(item_with(2, |d| d.type_kind = ItemTypeKind::MagicField));
        t.add_item(item_with(3, |d| d.type_kind = ItemTypeKind::MagicField));
        let f = t.get_field_item().expect("should find magic field");
        assert_eq!(f.get_id(), 2);
    }

    #[test]
    fn test_get_field_item_returns_none_without_magic_field() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(1, |d| d.type_kind = ItemTypeKind::None));
        assert!(t.get_field_item().is_none());
    }

    #[test]
    fn test_get_teleport_item_finds_teleport() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(7, |d| d.type_kind = ItemTypeKind::Teleport));
        assert!(t.get_teleport_item().is_some());
    }

    #[test]
    fn test_get_trash_holder_finds_trashholder() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(8, |d| d.type_kind = ItemTypeKind::TrashHolder));
        assert!(t.get_trash_holder().is_some());
    }

    #[test]
    fn test_get_mailbox_finds_mailbox() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(9, |d| d.type_kind = ItemTypeKind::Mailbox));
        assert!(t.get_mailbox().is_some());
    }

    #[test]
    fn test_get_bed_item_finds_bed() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(10, |d| d.type_kind = ItemTypeKind::Bed));
        assert!(t.get_bed_item().is_some());
    }

    #[test]
    fn test_get_item_by_top_order_filters_by_order() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(11, |d| {
            d.always_on_top = true;
            d.always_on_top_order = 1;
        }));
        t.add_item(item_with(12, |d| {
            d.always_on_top = true;
            d.always_on_top_order = 2;
        }));
        let found = t.get_item_by_top_order(2).expect("top_order=2 item");
        assert_eq!(found.get_id(), 12);
        assert!(t.get_item_by_top_order(99).is_none());
    }

    // --- get_thing_at_index -----------------------------------------------

    #[test]
    fn test_get_thing_at_index_creature_then_ground_then_item() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(7);
        t.set_ground(make_item(100));
        t.add_item(make_item(200));
        // 0 → creature, 1 → ground, 2 → item
        match t.get_thing_at_index(0).unwrap() {
            TileThingRef::Creature(id) => assert_eq!(id, 7),
            _ => panic!("expected creature"),
        }
        match t.get_thing_at_index(1).unwrap() {
            TileThingRef::Ground(i) => assert_eq!(i.get_id(), 100),
            _ => panic!("expected ground"),
        }
        match t.get_thing_at_index(2).unwrap() {
            TileThingRef::Item(i) => assert_eq!(i.get_id(), 200),
            _ => panic!("expected item"),
        }
        assert!(t.get_thing_at_index(99).is_none());
    }

    // --- creature index --------------------------------------------------

    #[test]
    fn test_get_creature_index_finds_creature() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(42);
        t.add_creature(7);
        assert_eq!(t.get_creature_index(7), Some(0)); // 7 was inserted at front
        assert_eq!(t.get_creature_index(42), Some(1));
        assert_eq!(t.get_creature_index(99), None);
    }

    #[test]
    fn test_get_item_thing_index_handles_ground_and_stack() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(5);
        t.set_ground(make_item(50));
        t.add_item(make_item(60));
        t.add_item(make_item(70));
        // creatures=1 + ground=1, so item 60 is at index 2.
        assert_eq!(t.get_item_thing_index(50), Some(1));
        assert_eq!(t.get_item_thing_index(60), Some(2));
        assert_eq!(t.get_item_thing_index(70), Some(3));
        assert_eq!(t.get_item_thing_index(999), None);
    }

    // --- get_use_item ----------------------------------------------------

    #[test]
    fn test_get_use_item_returns_top_down_when_negative() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(11));
        t.add_item(make_item(12));
        let i = t.get_use_item(-1).expect("top down item");
        assert_eq!(i.get_id(), 11);
    }

    #[test]
    fn test_get_use_item_returns_ground_at_correct_index() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(50));
        t.add_item(make_item(60));
        // creatures=0, ground at index 0, item at index 1.
        let g = t.get_use_item(0).expect("ground");
        assert_eq!(g.get_id(), 50);
        let item = t.get_use_item(1).expect("first item");
        assert_eq!(item.get_id(), 60);
    }

    // --- get_item_type_count ---------------------------------------------

    #[test]
    fn test_get_item_type_count_sums_stacks() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with_count(2160, 5, |d| d.stackable = true));
        t.add_item(item_with_count(2160, 3, |d| d.stackable = true));
        t.add_item(item_with_count(2152, 1, |_d| {}));
        assert_eq!(Tile::get_item_type_count(&t, 2160, -1), 8);
        assert_eq!(Tile::get_item_type_count(&t, 2152, -1), 1);
        assert_eq!(Tile::get_item_type_count(&t, 9999, -1), 0);
    }

    #[test]
    fn test_get_item_type_count_filters_by_subtype() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with_count(2160, 5, |d| d.stackable = true));
        t.add_item(item_with_count(2160, 3, |d| d.stackable = true));
        // sub_type filter: only count when stack_count matches
        assert_eq!(Tile::get_item_type_count(&t, 2160, 5), 5);
        assert_eq!(Tile::get_item_type_count(&t, 2160, 3), 3);
    }

    #[test]
    fn test_get_item_type_count_includes_ground() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(item_with_count(2160, 1, |_| {}));
        assert_eq!(Tile::get_item_type_count(&t, 2160, -1), 1);
    }

    // --- has_creature ----------------------------------------------------

    #[test]
    fn test_has_creature_finds_added_creature() {
        let mut t = Tile::new(0, 0, 0);
        assert!(!t.has_creature(99));
        t.add_creature(99);
        assert!(t.has_creature(99));
    }

    // --- update_item / replace_item / remove_item_count ------------------

    #[test]
    fn test_update_item_returns_old_when_index_valid() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(5));
        let old = t.update_item(0, make_item(6)).expect("old item");
        assert_eq!(old.get_id(), 5);
        assert_eq!(t.items()[0].get_id(), 6);
    }

    #[test]
    fn test_update_item_returns_none_when_index_invalid() {
        let mut t = Tile::new(0, 0, 0);
        assert!(t.update_item(99, make_item(1)).is_none());
    }

    #[test]
    fn test_replace_item_synonym_for_update_item() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(5));
        let old = t.replace_item(0, make_item(7)).expect("old");
        assert_eq!(old.get_id(), 5);
        assert_eq!(t.items()[0].get_id(), 7);
    }

    #[test]
    fn test_remove_item_count_decrements_stackable() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with_count(2160, 10, |d| d.stackable = true));
        assert!(t.remove_item_count(0, 4));
        assert_eq!(t.items()[0].get_count(), 6);
    }

    #[test]
    fn test_remove_item_count_removes_when_count_reaches_zero() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with_count(2160, 10, |d| d.stackable = true));
        assert!(t.remove_item_count(0, 10));
        assert!(t.items().is_empty());
    }

    #[test]
    fn test_remove_item_count_removes_non_stackable_outright() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(make_item(5));
        assert!(t.remove_item_count(0, 1));
        assert!(t.items().is_empty());
    }

    #[test]
    fn test_remove_item_count_returns_false_for_bad_index() {
        let mut t = Tile::new(0, 0, 0);
        assert!(!t.remove_item_count(99, 1));
    }

    // --- has_property / has_property_excluding / has_height --------------

    #[test]
    fn test_has_property_block_solid_finds_ground() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(item_with(1, |d| d.block_solid = true));
        assert!(t.has_property(ItemProperty::BlockSolid));
    }

    #[test]
    fn test_has_property_block_path_finds_stacked_item() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(1, |d| d.block_path_find = true));
        assert!(t.has_property(ItemProperty::BlockPath));
        assert!(!t.has_property(ItemProperty::BlockSolid));
    }

    #[test]
    fn test_has_property_excluding_skips_named_index() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(1, |d| d.block_path_find = true));
        t.add_item(item_with(2, |d| d.block_path_find = false));
        // Excluding index 0 (the blocker) → property no longer reports true.
        assert!(!t.has_property_excluding(0, ItemProperty::BlockPath));
        // Excluding index 1 (non-blocker) → still reports true.
        assert!(t.has_property_excluding(1, ItemProperty::BlockPath));
    }

    #[test]
    fn test_has_height_counts_only_height_items() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(item_with(1, |d| d.has_height = true));
        t.add_item(item_with(2, |d| d.has_height = true));
        t.add_item(item_with(3, |d| d.has_height = false));
        assert!(t.has_height(2));
        assert!(!t.has_height(3));
    }

    #[test]
    fn test_has_property_immovable_block_solid_requires_both() {
        let mut t = Tile::new(0, 0, 0);
        // Moveable solid: doesn't satisfy.
        t.add_item(item_with(1, |d| {
            d.block_solid = true;
            d.moveable = true;
        }));
        assert!(!t.has_property(ItemProperty::ImmovableBlockSolid));
        // Immovable solid: satisfies.
        t.add_item(item_with(2, |d| {
            d.block_solid = true;
            d.moveable = false;
        }));
        assert!(t.has_property(ItemProperty::ImmovableBlockSolid));
    }

    #[test]
    fn test_has_property_nofield_block_path_excludes_magic_fields() {
        let mut t = Tile::new(0, 0, 0);
        // Magic field that blocks pathfinding: does NOT satisfy NoFieldBlockPath.
        t.add_item(item_with(1, |d| {
            d.block_path_find = true;
            d.type_kind = ItemTypeKind::MagicField;
        }));
        assert!(!t.has_property(ItemProperty::NoFieldBlockPath));
        // Non-field path blocker: satisfies.
        t.add_item(item_with(2, |d| {
            d.block_path_find = true;
            d.type_kind = ItemTypeKind::None;
        }));
        assert!(t.has_property(ItemProperty::NoFieldBlockPath));
    }

    // --- visible-creature dispatch ---------------------------------------

    #[test]
    fn test_get_bottom_creature_returns_back_of_list() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(1);
        t.add_creature(2);
        t.add_creature(3); // most recently added — front
                           // bottom = back = first added = 1
        assert_eq!(t.get_bottom_creature(), Some(1));
    }

    #[test]
    fn test_get_top_visible_creature_uses_predicate() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(1);
        t.add_creature(2);
        t.add_creature(3);
        // Predicate: only id=2 visible
        assert_eq!(t.get_top_visible_creature(|id| id == 2), Some(2));
        // Predicate: none visible
        assert_eq!(t.get_top_visible_creature(|_| false), None);
    }

    #[test]
    fn test_get_bottom_visible_creature_iterates_from_back() {
        let mut t = Tile::new(0, 0, 0);
        t.add_creature(1);
        t.add_creature(2);
        t.add_creature(3);
        // Back of list is 1 (first added). Visible-1 returns 1.
        assert_eq!(t.get_bottom_visible_creature(|id| id == 1), Some(1));
        // Visible-3 returns 3 (since iterating from back, but 3 is at front).
        assert_eq!(t.get_bottom_visible_creature(|id| id == 3), Some(3));
    }

    // --- stackpos helpers ------------------------------------------------

    #[test]
    fn test_get_client_index_of_creature_with_ground() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(50));
        t.add_creature(7);
        // ground=1, top_items=0, creature 7 is first visible → stackpos=1
        assert_eq!(t.get_client_index_of_creature(7, |_| true), Some(1));
        assert_eq!(t.get_client_index_of_creature(99, |_| true), None);
    }

    #[test]
    fn test_get_client_index_skips_invisible_creatures_above() {
        let mut t = Tile::new(0, 0, 0);
        // No ground, no top items. 3 creatures: 1, 2, 3. Front=3.
        t.add_creature(1);
        t.add_creature(2);
        t.add_creature(3);
        // Predicate: id=1 invisible. Walking 3 → 2 → 1:
        //   3 (visible) → stackpos++  (now 1) → check 2: stackpos becomes 2.
        //   But we want the index OF 1 — so we walk past 3 and 2, stackpos
        //   ends at 2 (3 visible, 2 visible). 1 stackpos = 2.
        let idx = t.get_client_index_of_creature(1, |id| id != 1);
        assert_eq!(idx, Some(2));
    }

    #[test]
    fn test_get_stackpos_of_item_respects_top_order() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(50));
        t.add_item(item_with(99, |d| d.always_on_top = true));
        t.add_creature(7);
        t.add_item(item_with(200, |d| d.always_on_top = false));
        // Items vec layout: [99 (top), 200 (down)] (insert order)
        // ground=1, top_item=1 → top_item at stackpos=1
        let top_stackpos = t.get_stackpos_of_item(0, |_| true);
        assert_eq!(top_stackpos, Some(1));
        // down_item: ground=1 + top=1 + creature=1 = 3 → down_item at stackpos=3
        let down_stackpos = t.get_stackpos_of_item(1, |_| true);
        assert_eq!(down_stackpos, Some(3));
    }

    #[test]
    fn test_get_stackpos_of_item_returns_none_for_bad_index() {
        let t = Tile::new(0, 0, 0);
        assert!(t.get_stackpos_of_item(99, |_| true).is_none());
    }

    // --- set/reset_tile_flags --------------------------------------------

    #[test]
    fn test_set_tile_flags_for_teleport_sets_flag() {
        let mut t = Tile::new(0, 0, 0);
        let teleport = item_with(1, |d| d.type_kind = ItemTypeKind::Teleport);
        t.set_tile_flags_for_item(&teleport);
        assert!(t.has_flag(flags::TELEPORT));
    }

    #[test]
    fn test_reset_tile_flags_clears_teleport_flag() {
        let mut t = Tile::new(0, 0, 0);
        let teleport = item_with(1, |d| d.type_kind = ItemTypeKind::Teleport);
        t.set_tile_flags_for_item(&teleport);
        assert!(t.has_flag(flags::TELEPORT));
        t.reset_tile_flags_for_item(&teleport);
        assert!(!t.has_flag(flags::TELEPORT));
    }

    #[test]
    fn test_set_tile_flags_for_block_solid_sets_blocksolid() {
        let mut t = Tile::new(0, 0, 0);
        let block = item_with(1, |d| d.block_solid = true);
        t.set_tile_flags_for_item(&block);
        assert!(t.has_flag(flags::BLOCKSOLID));
    }

    #[test]
    fn test_reset_tile_flags_keeps_blocksolid_when_other_blocker_remains() {
        let mut t = Tile::new(0, 0, 0);
        let block1 = item_with(1, |d| d.block_solid = true);
        let block2 = item_with(2, |d| d.block_solid = true);
        t.add_item(block1.clone());
        t.add_item(block2.clone());
        t.set_tile_flags_for_item(&block1);
        // Remove the first blocker conceptually but keep block2 in items.
        t.reset_tile_flags_for_item(&block1);
        // block2 still on tile → flag should remain set.
        assert!(t.has_flag(flags::BLOCKSOLID));
    }

    #[test]
    fn test_set_tile_flags_for_each_kind() {
        for (kind, flag) in [
            (ItemTypeKind::Teleport, flags::TELEPORT),
            (ItemTypeKind::MagicField, flags::MAGICFIELD),
            (ItemTypeKind::Mailbox, flags::MAILBOX),
            (ItemTypeKind::TrashHolder, flags::TRASHHOLDER),
            (ItemTypeKind::Bed, flags::BED),
            (ItemTypeKind::Depot, flags::DEPOT),
        ] {
            let mut t = Tile::new(0, 0, 0);
            let it = item_with(1, |d| d.type_kind = kind);
            t.set_tile_flags_for_item(&it);
            assert!(t.has_flag(flag), "kind {:?} should set flag {}", kind, flag);
        }
    }

    // --- internal_add_item -----------------------------------------------

    #[test]
    fn test_internal_add_item_top_order_appended() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(1, |d| d.always_on_top = false));
        t.internal_add_item(item_with(2, |d| d.always_on_top = true));
        // Down-item first, top-item appended at end.
        assert_eq!(t.items()[1].get_id(), 2);
    }

    #[test]
    fn test_internal_add_item_down_order_prepended() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with(1, |d| d.always_on_top = false));
        t.internal_add_item(item_with(99, |d| d.always_on_top = false));
        // New down-item goes to the front.
        assert_eq!(t.items()[0].get_id(), 99);
    }

    // --- post_add / post_remove (default no-op) --------------------------

    #[test]
    fn test_post_add_item_is_noop_by_default() {
        let mut t = Tile::new(0, 0, 0);
        t.post_add_item(100, 0); // must not panic
    }

    #[test]
    fn test_post_remove_item_is_noop_by_default() {
        let mut t = Tile::new(0, 0, 0);
        t.post_remove_item(100, 0); // must not panic
    }

    // --- Cylinder trait impl ----------------------------------------------

    #[test]
    fn test_tile_cylinder_first_index_zero() {
        let t = Tile::new(0, 0, 0);
        assert_eq!(CommonCylinder::cylinder_first_index(&t), 0);
    }

    #[test]
    fn test_tile_cylinder_last_index_matches_thing_count() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.add_item(make_item(2));
        t.add_creature(3);
        assert_eq!(CommonCylinder::cylinder_last_index(&t), t.get_thing_count());
    }

    #[test]
    fn test_tile_cylinder_query_add_with_no_ground_refuses() {
        let t = Tile::new(0, 0, 0);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_add(&t, 0, &dummy, 1, 0),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_tile_cylinder_query_add_nolimit_bypasses() {
        let t = Tile::new(0, 0, 0);
        let dummy = DefaultThing;
        // NOLIMIT bit in tile.query_add (constant defined in query_flags::NOLIMIT)
        assert_eq!(
            CommonCylinder::cylinder_query_add(&t, 0, &dummy, 1, query_flags::NOLIMIT),
            CommonRv::NoError
        );
    }

    #[test]
    fn test_tile_cylinder_query_remove_refuses_when_empty() {
        let t = Tile::new(0, 0, 0);
        let dummy = DefaultThing;
        assert_eq!(
            CommonCylinder::cylinder_query_remove(&t, &dummy, 1, 0),
            CommonRv::NotPossible
        );
    }

    #[test]
    fn test_tile_implements_thing_is_not_removed() {
        use forgottenserver_common::thing::Thing as CommonThing;
        let t = Tile::new(0, 0, 0);
        assert!(!CommonThing::is_removed(&t));
        // is_ground reflects ground presence.
        assert!(!CommonThing::is_ground(&t));
    }

    #[test]
    fn test_tile_via_dyn_cylinder_trait_object() {
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        let cyl: &dyn CommonCylinder = &t;
        assert_eq!(cyl.cylinder_first_index(), 0);
        let dummy = DefaultThing;
        assert_eq!(cyl.cylinder_query_add(0, &dummy, 1, 0), CommonRv::NoError);
    }

    #[test]
    fn test_tile_cylinder_item_type_count_delegates() {
        let mut t = Tile::new(0, 0, 0);
        t.add_item(item_with_count(2160, 5, |d| d.stackable = true));
        assert_eq!(CommonCylinder::cylinder_item_type_count(&t, 2160, -1), 5);
    }

    // --- Thing::get_first_index (line 813) and Thing impl get_first_index (line 1406) --

    /// C++ evidence: `size_t Thing::getFirstIndex() const { return 0; }` in thing.h.
    /// Tiles (like all Things) return 0 as their first valid index.
    /// Tests the inherent method via the Thing trait interface.
    #[test]
    fn test_tile_thing_get_first_index_returns_zero() {
        use forgottenserver_common::thing::Thing as CommonThing;
        let t = Tile::new(0, 0, 0);
        // C++ Tile::getFirstIndex() returns 0 — not a container, index always starts at 0.
        assert_eq!(CommonThing::get_first_index(&t), 0);
    }

    /// C++ evidence: `size_t Thing::getFirstIndex() const { return 0; }` in thing.h,
    /// as called from within the Thing impl on Tile (line 1406).
    /// Verifies the Cylinder-context Thing impl returns 0 independently of item count.
    #[test]
    fn test_tile_cylinder_get_first_index_returns_zero() {
        use forgottenserver_common::thing::Thing as CommonThing;
        let mut t = Tile::new(0, 0, 0);
        t.set_ground(make_item(1));
        t.add_item(make_item(2));
        t.add_creature(99);
        // First index is always 0 regardless of how many things are on the tile.
        assert_eq!(CommonThing::get_first_index(&t), 0);
    }
}
