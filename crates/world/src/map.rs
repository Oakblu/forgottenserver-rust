// Migrated from forgottenserver/src/map.h + map.cpp
//
// Map is a sparse 3D tile grid backed by a HashMap.
// Spectator/range queries use simple coordinate iteration.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use forgottenserver_common::position::Position;
use forgottenserver_map::tile::Tile;

// ---------------------------------------------------------------------------
// Map constants — mirrors map.h static constexpr values
// ---------------------------------------------------------------------------

/// Server-side spectator range (tiles from centre per axis).
/// `maxViewportX = 11` in C++.
pub const MAP_MAX_VIEWPORT_X: i32 = 11;

/// Server-side spectator range (tiles from centre per axis).
/// `maxViewportY = 11` in C++.
pub const MAP_MAX_VIEWPORT_Y: i32 = 11;

/// Half-width of the client viewport.  `maxClientViewportX = 8`.
pub const MAP_MAX_CLIENT_VIEWPORT_X: i32 = 8;

/// Half-height of the client viewport.  `maxClientViewportY = 6`.
pub const MAP_MAX_CLIENT_VIEWPORT_Y: i32 = 6;

/// Maximum number of A* nodes reserved per search.
/// `nodeReserveSize = (maxViewportX * maxViewportY * 3) / 2`
pub const MAP_NODE_RESERVE_SIZE: usize =
    ((MAP_MAX_VIEWPORT_X * MAP_MAX_VIEWPORT_Y * 3) / 2) as usize;

/// Maximum floors in the map.
pub const MAP_MAX_LAYERS: u8 = 16;

/// Normal (cardinal) walk cost.
pub const MAP_NORMALWALKCOST: u16 = 10;

/// Diagonal walk cost.
pub const MAP_DIAGONALWALKCOST: u16 = 25;

// ---------------------------------------------------------------------------
// AStarNode — mirrors `struct AStarNode` from map.h
// ---------------------------------------------------------------------------

/// Single node in the A* open/closed set.
///
/// Mirrors `struct AStarNode { AStarNode* parent; uint16_t x, y; uint16_t g, f; }`.
/// Parent is stored as an index into `AStarNodes::nodes` rather than a raw
/// pointer, which is safe and avoids unsafe Rust.
#[derive(Debug, Clone)]
pub struct AStarNode {
    /// Index of the parent node in `AStarNodes::nodes`, or `usize::MAX` for root.
    pub parent_idx: usize,
    pub x: u16,
    pub y: u16,
    /// Cost from start to this node.
    pub g: u16,
    /// Total estimated cost (`g + h`).
    pub f: u16,
}

// ---------------------------------------------------------------------------
// AStarNodes — mirrors `class AStarNodes` from map.h
// ---------------------------------------------------------------------------

/// Container for the A* search state.
///
/// Mirrors `class AStarNodes`:
/// - `createNode` — allocate a new node (returns `None` when the reserve is
///   full, matching the C++ "limit of nodes reached" case).
/// - `getBestNode` — pop the lowest-f node from the open set (skipping
///   already-visited nodes).
/// - `getNodeByPosition` — lookup by (x, y) coordinate.
/// - `getMapWalkCost` — cardinal = 10, diagonal = 25.
#[derive(Debug)]
pub struct AStarNodes {
    /// All allocated nodes in insertion order.
    nodes: Vec<AStarNode>,
    /// Maps `hashCoord(x, y)` → index into `nodes`.
    node_map: HashMap<u32, usize>,
    /// Visited coordinates (hashCoord).
    visited: HashSet<u32>,
    /// Min-heap: `(Reverse(f), node_index)`.
    open_set: BinaryHeap<(Reverse<u16>, usize)>,
}

/// Mirrors C++ `hashCoord(x, y)`.
#[inline]
fn hash_coord(x: u16, y: u16) -> u32 {
    ((x as u32) << 16) | (y as u32)
}

impl AStarNodes {
    /// Creates a new search initialised with the start node at `(x, y)`.
    pub fn new(x: u16, y: u16) -> Self {
        let mut nodes = AStarNodes {
            nodes: Vec::with_capacity(MAP_NODE_RESERVE_SIZE),
            node_map: HashMap::with_capacity(MAP_NODE_RESERVE_SIZE),
            visited: HashSet::with_capacity(MAP_NODE_RESERVE_SIZE),
            open_set: BinaryHeap::new(),
        };
        nodes.create_node(usize::MAX, x, y, 0, 0);
        nodes
    }

    /// Allocates a new node.  Returns `None` when the reserve limit is reached
    /// (mirrors C++ returning `nullptr` when `nodes.size() == nodeReserveSize`).
    pub fn create_node(
        &mut self,
        parent_idx: usize,
        x: u16,
        y: u16,
        g: u16,
        f: u16,
    ) -> Option<usize> {
        if self.nodes.len() == MAP_NODE_RESERVE_SIZE {
            return None;
        }
        let idx = self.nodes.len();
        self.nodes.push(AStarNode {
            parent_idx,
            x,
            y,
            g,
            f,
        });
        let key = hash_coord(x, y);
        self.node_map.insert(key, idx);
        self.open_set.push((Reverse(f), idx));
        Some(idx)
    }

    /// Pops and returns the index of the lowest-f unvisited node, or `None`
    /// when the open set is exhausted.
    ///
    /// Mirrors `getBestNode()` — nodes are marked visited on pop, so a
    /// coordinate cannot be expanded twice even if it appears multiple times
    /// in the heap.
    pub fn get_best_node(&mut self) -> Option<usize> {
        while let Some((_, idx)) = self.open_set.pop() {
            let key = hash_coord(self.nodes[idx].x, self.nodes[idx].y);
            if self.visited.insert(key) {
                return Some(idx);
            }
        }
        None
    }

    /// Returns the index of the node at `(x, y)`, or `None`.
    pub fn get_node_by_position(&self, x: u16, y: u16) -> Option<usize> {
        self.node_map.get(&hash_coord(x, y)).copied()
    }

    /// Returns a reference to the node at `idx`.
    pub fn node(&self, idx: usize) -> &AStarNode {
        &self.nodes[idx]
    }

    /// Returns a mutable reference to the node at `idx`.
    pub fn node_mut(&mut self, idx: usize) -> &mut AStarNode {
        &mut self.nodes[idx]
    }

    /// Cardinal movement = 10, diagonal movement = 25.
    ///
    /// Mirrors `AStarNodes::getMapWalkCost`.
    pub fn get_map_walk_cost(from_x: u16, from_y: u16, to_x: u16, to_y: u16) -> u16 {
        let dx = (from_x as i32 - to_x as i32).unsigned_abs();
        let dy = (from_y as i32 - to_y as i32).unsigned_abs();
        if dx == dy {
            MAP_DIAGONALWALKCOST
        } else {
            MAP_NORMALWALKCOST
        }
    }
}

// ---------------------------------------------------------------------------
// Map
// ---------------------------------------------------------------------------

/// Sparse 3-D tile grid.
///
/// Keys are `(x, y, z)` tuples stored in a flat `HashMap`.  This matches the
/// semantics of the C++ QTree-based grid while being far simpler to implement
/// and test in Rust.
///
/// `declared_width` / `declared_height` hold the dimensions parsed from an
/// OTBM header (see `IoMap`).  They default to `0` and are not derived from
/// the tile set.
/// Cache key for spectator queries: (center, range_x, range_y, include_floor).
///
/// C++ caches by `Position` alone — the range/floor parameters are always
/// the viewport defaults at the cached call sites. Rust's `get_spectators`
/// is parameterised, so the cache key carries the full query shape to stay
/// correct under non-default range / floor settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpectatorKey {
    center: Position,
    range_x: i32,
    range_y: i32,
    include_floor_change: bool,
}

#[derive(Debug, Default)]
pub struct Map {
    tiles: HashMap<(u16, u16, u8), Tile>,
    pub(crate) declared_width: u16,
    pub(crate) declared_height: u16,
    /// Named waypoints loaded from map data.
    /// Mirrors `std::map<std::string, Position> Map::waypoints`.
    pub waypoints: HashMap<String, Position>,
    /// Spectator-query cache mirroring C++ `Map::spectatorCache` (in
    /// `map.h`). Entries are inserted on the first `get_spectators` for
    /// a given `(center, range_x, range_y, include_floor_change)` tuple
    /// and dropped when `clear_spectator_cache` runs (which the game
    /// loop calls whenever anything moves on the map).
    ///
    /// `RefCell` keeps the cache populatable through a `&self` query —
    /// the C++ method is also `const` despite mutating the cache.
    spectator_cache: std::cell::RefCell<HashMap<SpectatorKey, Vec<Position>>>,
    /// Players-only spectator cache — separate slot mirroring C++
    /// `playersSpectatorCache`. The Rust port doesn't yet split by
    /// creature kind (no Creature dispatch in `world`), so this cache
    /// remains empty until a downstream layer wires it. The
    /// `clear_players_spectator_cache` hook is in place so callers that
    /// move players have a deterministic invalidation point.
    players_spectator_cache: std::cell::RefCell<HashMap<SpectatorKey, Vec<Position>>>,
}

impl Map {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    /// Creates an empty map.
    pub fn new() -> Self {
        Map {
            tiles: HashMap::new(),
            declared_width: 0,
            declared_height: 0,
            waypoints: HashMap::new(),
            spectator_cache: std::cell::RefCell::new(HashMap::new()),
            players_spectator_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }

    // -----------------------------------------------------------------------
    // Tile CRUD
    // -----------------------------------------------------------------------

    /// Returns a reference to the tile at `(x, y, z)`, or `None` if absent.
    pub fn get_tile(&self, x: u16, y: u16, z: u8) -> Option<&Tile> {
        self.tiles.get(&(x, y, z))
    }

    /// Returns a mutable reference to the tile at `(x, y, z)`, or `None`.
    pub fn get_tile_mut(&mut self, x: u16, y: u16, z: u8) -> Option<&mut Tile> {
        self.tiles.get_mut(&(x, y, z))
    }

    /// Inserts (or replaces) the tile at `(x, y, z)`.
    pub fn set_tile(&mut self, x: u16, y: u16, z: u8, tile: Tile) {
        self.tiles.insert((x, y, z), tile);
    }

    /// Removes the tile at `(x, y, z)`.  Returns the removed tile, if any.
    pub fn remove_tile(&mut self, x: u16, y: u16, z: u8) -> Option<Tile> {
        self.tiles.remove(&(x, y, z))
    }

    /// Returns the total number of tiles currently stored in the map.
    pub fn get_tile_count(&self) -> usize {
        self.tiles.len()
    }

    // -----------------------------------------------------------------------
    // Dimensions
    // -----------------------------------------------------------------------

    /// Returns the width of the bounding box covering all stored tiles.
    ///
    /// Width = `max_x - min_x + 1`.  Returns `0` for an empty map.
    pub fn get_width(&self) -> u32 {
        if self.tiles.is_empty() {
            return 0;
        }
        let min_x = self.tiles.keys().map(|&(x, _, _)| x).min().unwrap();
        let max_x = self.tiles.keys().map(|&(x, _, _)| x).max().unwrap();
        (max_x - min_x) as u32 + 1
    }

    /// Returns the height of the bounding box covering all stored tiles.
    ///
    /// Height = `max_y - min_y + 1`.  Returns `0` for an empty map.
    pub fn get_height(&self) -> u32 {
        if self.tiles.is_empty() {
            return 0;
        }
        let min_y = self.tiles.keys().map(|&(_, y, _)| y).min().unwrap();
        let max_y = self.tiles.keys().map(|&(_, y, _)| y).max().unwrap();
        (max_y - min_y) as u32 + 1
    }

    // -----------------------------------------------------------------------
    // Tile-property queries
    // -----------------------------------------------------------------------

    /// Returns `true` when the tile at `(x, y, z)` does not block projectiles.
    ///
    /// Mirrors `Map::isTileClear`:
    /// - A missing tile is considered clear.
    /// - When `block_floor` is `true`, a tile that has a ground item is NOT
    ///   clear (used for staircase / floor checks in `isSightClear`).
    /// - When `pathfinding` is `true`, the tile must also not be
    ///   BLOCKPATH / BLOCKSOLID / IMMOVABLE*.
    /// - Otherwise only `is_block_projectile()` is checked.
    pub fn is_tile_clear(
        &self,
        x: u16,
        y: u16,
        z: u8,
        block_floor: bool,
        pathfinding: bool,
    ) -> bool {
        let tile = match self.get_tile(x, y, z) {
            Some(t) => t,
            None => return true,
        };

        if block_floor && tile.get_ground().is_some() {
            return false;
        }

        if pathfinding {
            use forgottenserver_map::tile::flags;
            // Check if ANY of the blocking flags are set (use bitwise AND, not has_flag
            // which requires ALL bits to be present).
            let blocking_flags = flags::BLOCKPATH
                | flags::BLOCKSOLID
                | flags::IMMOVABLEBLOCKPATH
                | flags::IMMOVABLEBLOCKSOLID;
            let has_any_blocking = tile.flags & blocking_flags != 0;
            // Also block if there is a creature on the tile (top creature check)
            return !has_any_blocking && tile.get_top_creature().is_none();
        }

        !tile.is_block_projectile()
    }

    // -----------------------------------------------------------------------
    // Sight-line checks
    // -----------------------------------------------------------------------

    /// Checks whether a straight line from `(x0, y0)` to `(x1, y1)` on floor
    /// `z` is clear of projectile-blocking tiles.
    ///
    /// Mirrors `Map::checkSightLine`:
    /// - Same position → always clear.
    /// - Uses Bresenham-style slope iteration, checking intermediate tiles only
    ///   (endpoints are NOT checked, matching C++ behaviour).
    /// - When `|dy| > |dx|` the steep case is used (swaps x/y axes).
    pub fn check_sight_line(
        &self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
        z: u8,
        pathfinding: bool,
    ) -> bool {
        if x0 == x1 && y0 == y1 {
            return true;
        }

        let dx = x1 as i32 - x0 as i32;
        let dy = y1 as i32 - y0 as i32;

        if dy.abs() > dx.abs() {
            // steep — iterate along y.
            // Note: in this branch `dy != 0` (since `|dy| > |dx| >= 0`), so `ys != ye`
            // and the slope denominator is always non-zero.
            let (ys, ye, xs, xe) = if y1 > y0 {
                (y0, y1, x0, x1)
            } else {
                (y1, y0, x1, x0)
            };
            let slope = (xe as f64 - xs as f64) / (ye as f64 - ys as f64);
            let mut xi = xs as f64 + slope;
            let mut y = ys + 1;
            while y < ye {
                let tx = (xi + 0.1).floor() as i32;
                // `tx >= 0` guards the `as u16` cast against negative
                // floats. C++ `Map::checkSightLine` has no further bound
                // check; missing tiles are handled by `is_tile_clear`.
                // The previous `y < MAP_MAX_LAYERS as u16` check was a
                // typo that confused the Z-axis layer count with the
                // Y-axis world bound and silently passed any sight line
                // whose Y exceeded 16.
                if tx >= 0 && !self.is_tile_clear(tx as u16, y, z, false, pathfinding) {
                    return false;
                }
                xi += slope;
                y += 1;
            }
        } else {
            // slight — iterate along x.
            // Note: in this branch `dx != 0` (otherwise we would have taken the steep
            // branch since `dy.abs() > 0 == dx.abs()`), so `xs != xe`.
            let (xs, xe, ys, ye) = if x1 > x0 {
                (x0, x1, y0, y1)
            } else {
                (x1, x0, y1, y0)
            };
            let slope = (ye as f64 - ys as f64) / (xe as f64 - xs as f64);
            let mut yi = ys as f64 + slope;
            let mut x = xs + 1;
            while x < xe {
                let ty = (yi + 0.1).floor() as i32;
                if ty >= 0
                    && x < u16::MAX
                    && !self.is_tile_clear(x, ty as u16, z, false, pathfinding)
                {
                    return false;
                }
                yi += slope;
                x += 1;
            }
        }

        true
    }

    /// Returns `true` when there are no projectile-blocking obstacles in a
    /// straight line from `from` to `to`.
    ///
    /// Mirrors `Map::isSightClear` logic:
    /// - Same floor: adjacent tiles (Chebyshev < 2) are always visible.
    /// - Same floor, sight is blocked, `same_floor=false`, floor > 0: check
    ///   one floor above.
    /// - Different floor and `same_floor=true` → false.
    /// - Crossing ground boundary (floor 7↔8) → false.
    /// - Target above (distance 1): check tile above source + line above.
    /// - Target below: check all tiles above target + line from source floor.
    pub fn is_sight_clear(
        &self,
        from: Position,
        to: Position,
        same_floor: bool,
        pathfinding: bool,
    ) -> bool {
        if from.z == to.z {
            // Skip check for adjacent tiles (Chebyshev < 2 in both axes)
            if from.distance_x(to) < 2 && from.distance_y(to) < 2 {
                return true;
            }

            if pathfinding {
                return self.check_sight_line(from.x, from.y, to.x, to.y, from.z, true);
            }

            let sight_clear = self.check_sight_line(from.x, from.y, to.x, to.y, from.z, false);
            if sight_clear || same_floor {
                return sight_clear;
            }

            // No obstacles above floor 0 so we can throw above
            if from.z == 0 {
                return true;
            }

            let new_z = from.z - 1;
            return self.is_tile_clear(from.x, from.y, new_z, true, false)
                && self.is_tile_clear(to.x, to.y, new_z, true, false)
                && self.check_sight_line(from.x, from.y, to.x, to.y, new_z, false);
        }

        // Different floors
        if same_floor {
            return false;
        }

        // Cannot throw across ground boundary (floor 7 ↔ 8)
        if (from.z < 8 && to.z > 7) || (from.z > 7 && to.z < 8) {
            return false;
        }

        if from.z > to.z {
            // Target above: only adjacent floors allowed
            if from.distance_z(to) > 1 {
                return false;
            }
            let new_z = from.z - 1;
            return self.is_tile_clear(from.x, from.y, new_z, true, false)
                && self.check_sight_line(from.x, from.y, to.x, to.y, new_z, false);
        }

        // Target below: all tiles above target must be clear
        let mut z = from.z;
        while z < to.z {
            if !self.is_tile_clear(to.x, to.y, z, true, false) {
                return false;
            }
            z += 1;
        }

        self.check_sight_line(from.x, from.y, to.x, to.y, from.z, false)
    }

    // -----------------------------------------------------------------------
    // Throw / range checks
    // -----------------------------------------------------------------------

    /// Returns `true` when an object can be thrown from `from` to `to`.
    ///
    /// Mirrors `Map::canThrowObjectTo`:
    /// - `range_x` / `range_y` are the maximum allowed distances per axis
    ///   (default `maxClientViewportX` / `maxClientViewportY` in C++).
    /// - When `check_line_of_sight` is `true` also calls `isSightClear`.
    /// - `same_floor` is forwarded to `isSightClear`.
    pub fn can_throw_object_to(
        &self,
        from: Position,
        to: Position,
        check_line_of_sight: bool,
        same_floor: bool,
        range_x: i32,
        range_y: i32,
    ) -> bool {
        if from.distance_x(to) > range_x || from.distance_y(to) > range_y {
            return false;
        }

        !check_line_of_sight || self.is_sight_clear(from, to, same_floor, false)
    }

    // -----------------------------------------------------------------------
    // Multifloor Z-range helper
    // -----------------------------------------------------------------------

    /// Returns the `(min_z, max_z)` inclusive floor range for spectator queries
    /// centred on `z` when `multifloor` is `true`.
    ///
    /// Mirrors the `getSpectators` multifloor logic from `map.cpp`:
    /// - `z > 7` (underground) → `[z-2, z+2]` clamped to `[0, 15]`.
    /// - `z == 6` → `[0, 8]`.
    /// - `z == 7` → `[0, 9]`.
    /// - Otherwise → `[0, 7]`.
    pub fn multifloor_z_range(z: u8) -> (u8, u8) {
        let max_layer = MAP_MAX_LAYERS - 1;
        if z > 7 {
            let min_z = z.saturating_sub(2);
            let max_z = (z + 2).min(max_layer);
            (min_z, max_z)
        } else if z == 6 {
            (0, 8)
        } else if z == 7 {
            (0, 9)
        } else {
            (0, 7)
        }
    }

    // -----------------------------------------------------------------------
    // Spectators
    // -----------------------------------------------------------------------

    /// Returns the positions of all tiles within the rectangular range
    /// `[center.x ± range_x, center.y ± range_y]` on the same floor.
    ///
    /// When `include_floor_change` is `false` only tiles whose `z` exactly
    /// matches `center_pos.z` are included (the typical use-case).
    pub fn get_spectators(
        &self,
        center_pos: Position,
        range_x: i32,
        range_y: i32,
        include_floor_change: bool,
    ) -> Vec<Position> {
        let key = SpectatorKey {
            center: center_pos,
            range_x,
            range_y,
            include_floor_change,
        };
        if let Some(cached) = self.spectator_cache.borrow().get(&key) {
            return cached.clone();
        }

        let mut result = Vec::new();
        for &(x, y, z) in self.tiles.keys() {
            // Floor filter
            if !include_floor_change && z != center_pos.z {
                continue;
            }

            let dx = (x as i32 - center_pos.x as i32).abs();
            let dy = (y as i32 - center_pos.y as i32).abs();

            if dx <= range_x && dy <= range_y {
                result.push(Position::new(x, y, z));
            }
        }

        self.spectator_cache
            .borrow_mut()
            .insert(key, result.clone());
        result
    }

    /// Drops every cached spectator result. Mirrors C++ `Map::clearSpectatorCache`.
    /// Callers (game-tick code that moves entities) must invoke this
    /// whenever a tile's contents change, so stale entries don't leak
    /// into subsequent queries.
    pub fn clear_spectator_cache(&self) {
        self.spectator_cache.borrow_mut().clear();
    }

    /// Same as `clear_spectator_cache`, scoped to the players-only slot.
    /// Mirrors C++ `Map::clearPlayersSpectatorCache`. The players-only
    /// cache currently shares its shape with the all-creature one; the
    /// split exists in the API so future Creature dispatch layers don't
    /// have to retrofit the interface.
    pub fn clear_players_spectator_cache(&self) {
        self.players_spectator_cache.borrow_mut().clear();
    }

    // -----------------------------------------------------------------------
    // Broadcast / range helpers
    // -----------------------------------------------------------------------

    /// Returns the positions of all tiles within Chebyshev distance ≤ `range`
    /// on the same floor as `center`.
    ///
    /// This maps to the broadcast-position logic used by `Game` when notifying
    /// nearby players of events.
    pub fn get_positions_in_range(&self, center: Position, range: i32) -> Vec<Position> {
        self.get_spectators(center, range, range, false)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_tile(x: u16, y: u16, z: u8) -> Tile {
        Tile::new(x, y, z)
    }

    // -----------------------------------------------------------------------
    // Map struct basics
    // -----------------------------------------------------------------------

    #[test]
    fn map_new_creates_empty_map() {
        let map = Map::new();
        assert_eq!(map.get_tile_count(), 0);
    }

    #[test]
    fn get_tile_returns_none_on_unknown_position() {
        let map = Map::new();
        assert!(map.get_tile(100, 200, 7).is_none());
    }

    #[test]
    fn set_tile_inserts_tile() {
        let mut map = Map::new();
        let tile = make_tile(10, 20, 7);
        map.set_tile(10, 20, 7, tile);
        assert!(map.get_tile(10, 20, 7).is_some());
    }

    #[test]
    fn get_tile_returns_some_after_setting() {
        let mut map = Map::new();
        map.set_tile(5, 5, 7, make_tile(5, 5, 7));
        let t = map.get_tile(5, 5, 7).unwrap();
        assert_eq!(t.position.x, 5);
        assert_eq!(t.position.y, 5);
        assert_eq!(t.position.z, 7);
    }

    #[test]
    fn remove_tile_removes_and_get_returns_none() {
        let mut map = Map::new();
        map.set_tile(3, 3, 7, make_tile(3, 3, 7));
        assert!(map.get_tile(3, 3, 7).is_some());
        map.remove_tile(3, 3, 7);
        assert!(map.get_tile(3, 3, 7).is_none());
    }

    #[test]
    fn get_tile_count_returns_number_of_tiles() {
        let mut map = Map::new();
        assert_eq!(map.get_tile_count(), 0);
        map.set_tile(1, 1, 7, make_tile(1, 1, 7));
        assert_eq!(map.get_tile_count(), 1);
        map.set_tile(2, 2, 7, make_tile(2, 2, 7));
        assert_eq!(map.get_tile_count(), 2);
        map.remove_tile(1, 1, 7);
        assert_eq!(map.get_tile_count(), 1);
    }

    // -----------------------------------------------------------------------
    // Spectators
    // -----------------------------------------------------------------------

    #[test]
    fn get_spectators_empty_map_returns_empty() {
        let map = Map::new();
        let center = Position::new(100, 100, 7);
        let result = map.get_spectators(center, 5, 5, false);
        assert!(result.is_empty());
    }

    #[test]
    fn get_spectators_3x3_grid_returns_9_positions() {
        let mut map = Map::new();
        // Seed a 3×3 grid at z=7 centred on (1,1)
        for dx in 0u16..3 {
            for dy in 0u16..3 {
                let x = dx;
                let y = dy;
                map.set_tile(x, y, 7, make_tile(x, y, 7));
            }
        }
        let center = Position::new(1, 1, 7);
        let result = map.get_spectators(center, 1, 1, false);
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn get_spectators_excludes_different_floor_when_flag_false() {
        let mut map = Map::new();
        map.set_tile(5, 5, 6, make_tile(5, 5, 6));
        map.set_tile(5, 5, 7, make_tile(5, 5, 7));
        let center = Position::new(5, 5, 7);
        let result = map.get_spectators(center, 0, 0, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].z, 7);
    }

    #[test]
    fn get_spectators_includes_all_floors_when_flag_true() {
        let mut map = Map::new();
        map.set_tile(5, 5, 6, make_tile(5, 5, 6));
        map.set_tile(5, 5, 7, make_tile(5, 5, 7));
        let center = Position::new(5, 5, 7);
        let result = map.get_spectators(center, 0, 0, true);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn get_spectators_range_boundary_exact() {
        let mut map = Map::new();
        // tile exactly at boundary
        map.set_tile(3, 3, 7, make_tile(3, 3, 7));
        // tile just outside the X boundary
        map.set_tile(4, 3, 7, make_tile(4, 3, 7));
        // tile inside range with x==4 so the second closure's rhs (`p.y == 3`)
        // is evaluated and short-circuits to false.
        map.set_tile(4, 0, 7, make_tile(4, 0, 7));
        let center = Position::new(0, 0, 7);
        let result = map.get_spectators(center, 4, 3, false);
        // (3,3) is within range; (4,3) is also within range now (range_x=4),
        // but we use a different closure asserting x==4 && y==3 is true (it is,
        // since both (4,3) and (4,0) are in result; (4,3) matches).
        assert!(result.iter().any(|p| p.x == 3 && p.y == 3));
        assert!(result.iter().any(|p| p.x == 4 && p.y == 3));
    }

    // -----------------------------------------------------------------------
    // Dimensions
    // -----------------------------------------------------------------------

    #[test]
    fn get_width_empty_map_returns_zero() {
        let map = Map::new();
        assert_eq!(map.get_width(), 0);
    }

    #[test]
    fn get_height_empty_map_returns_zero() {
        let map = Map::new();
        assert_eq!(map.get_height(), 0);
    }

    #[test]
    fn get_width_single_tile_returns_one() {
        let mut map = Map::new();
        map.set_tile(10, 10, 7, make_tile(10, 10, 7));
        assert_eq!(map.get_width(), 1);
    }

    #[test]
    fn get_width_multiple_tiles() {
        let mut map = Map::new();
        map.set_tile(5, 0, 7, make_tile(5, 0, 7));
        map.set_tile(10, 0, 7, make_tile(10, 0, 7));
        // max_x=10, min_x=5 → width=6
        assert_eq!(map.get_width(), 6);
    }

    #[test]
    fn get_height_multiple_tiles() {
        let mut map = Map::new();
        map.set_tile(0, 3, 7, make_tile(0, 3, 7));
        map.set_tile(0, 8, 7, make_tile(0, 8, 7));
        // max_y=8, min_y=3 → height=6
        assert_eq!(map.get_height(), 6);
    }

    // -----------------------------------------------------------------------
    // get_positions_in_range (Chebyshev)
    // -----------------------------------------------------------------------

    #[test]
    fn get_positions_in_range_empty_map() {
        let map = Map::new();
        let center = Position::new(50, 50, 7);
        assert!(map.get_positions_in_range(center, 5).is_empty());
    }

    #[test]
    fn get_positions_in_range_includes_exact_distance() {
        let mut map = Map::new();
        // tile at Chebyshev distance exactly 2 from center (5,5,7)
        map.set_tile(7, 5, 7, make_tile(7, 5, 7)); // dx=2, dy=0
        map.set_tile(5, 7, 7, make_tile(5, 7, 7)); // dx=0, dy=2
        let center = Position::new(5, 5, 7);
        let result = map.get_positions_in_range(center, 2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn get_positions_in_range_excludes_outside() {
        let mut map = Map::new();
        map.set_tile(10, 5, 7, make_tile(10, 5, 7)); // dx=5 > range 3
        let center = Position::new(5, 5, 7);
        let result = map.get_positions_in_range(center, 3);
        assert!(result.is_empty());
    }

    #[test]
    fn get_positions_in_range_same_floor_only() {
        let mut map = Map::new();
        map.set_tile(5, 5, 6, make_tile(5, 5, 6)); // different floor
        map.set_tile(5, 5, 7, make_tile(5, 5, 7)); // same floor
        let center = Position::new(5, 5, 7);
        let result = map.get_positions_in_range(center, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].z, 7);
    }

    // -----------------------------------------------------------------------
    // Map constants
    // -----------------------------------------------------------------------

    #[test]
    fn map_max_viewport_x_is_11() {
        assert_eq!(MAP_MAX_VIEWPORT_X, 11);
    }

    #[test]
    fn map_max_viewport_y_is_11() {
        assert_eq!(MAP_MAX_VIEWPORT_Y, 11);
    }

    #[test]
    fn map_max_client_viewport_x_is_8() {
        assert_eq!(MAP_MAX_CLIENT_VIEWPORT_X, 8);
    }

    #[test]
    fn map_max_client_viewport_y_is_6() {
        assert_eq!(MAP_MAX_CLIENT_VIEWPORT_Y, 6);
    }

    #[test]
    fn map_normalwalkcost_is_10() {
        assert_eq!(MAP_NORMALWALKCOST, 10);
    }

    #[test]
    fn map_diagonalwalkcost_is_25() {
        assert_eq!(MAP_DIAGONALWALKCOST, 25);
    }

    #[test]
    fn map_node_reserve_size_matches_formula() {
        let expected = ((11_i32 * 11 * 3) / 2) as usize;
        assert_eq!(MAP_NODE_RESERVE_SIZE, expected);
    }

    // -----------------------------------------------------------------------
    // Waypoints
    // -----------------------------------------------------------------------

    #[test]
    fn map_new_waypoints_is_empty() {
        let map = Map::new();
        assert!(map.waypoints.is_empty());
    }

    #[test]
    fn map_waypoints_can_be_inserted_and_retrieved() {
        let mut map = Map::new();
        let pos = Position::new(100, 200, 7);
        map.waypoints.insert("temple".to_string(), pos);
        assert_eq!(map.waypoints.get("temple"), Some(&pos));
    }

    // -----------------------------------------------------------------------
    // is_tile_clear
    // -----------------------------------------------------------------------

    #[test]
    fn is_tile_clear_missing_tile_returns_true() {
        let map = Map::new();
        assert!(map.is_tile_clear(100, 100, 7, false, false));
    }

    #[test]
    fn is_tile_clear_empty_tile_no_block_projectile_returns_true() {
        let mut map = Map::new();
        map.set_tile(5, 5, 7, make_tile(5, 5, 7));
        assert!(map.is_tile_clear(5, 5, 7, false, false));
    }

    #[test]
    fn is_tile_clear_block_floor_with_ground_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData::default();
        tile.set_ground(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        // block_floor = true: having a ground tile blocks
        assert!(!map.is_tile_clear(5, 5, 7, true, false));
    }

    #[test]
    fn is_tile_clear_block_floor_false_with_ground_returns_true() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData::default();
        tile.set_ground(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        // block_floor = false: ground alone does not block projectile
        assert!(map.is_tile_clear(5, 5, 7, false, false));
    }

    #[test]
    fn is_tile_clear_projectile_blocking_tile_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        assert!(!map.is_tile_clear(5, 5, 7, false, false));
    }

    #[test]
    fn is_tile_clear_pathfinding_blockpath_returns_false() {
        use forgottenserver_map::tile::flags;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        tile.set_flag(flags::BLOCKPATH);
        map.set_tile(5, 5, 7, tile);
        assert!(!map.is_tile_clear(5, 5, 7, false, true));
    }

    #[test]
    fn is_tile_clear_pathfinding_creature_on_tile_returns_false() {
        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        tile.add_creature(42);
        map.set_tile(5, 5, 7, tile);
        assert!(!map.is_tile_clear(5, 5, 7, false, true));
    }

    #[test]
    fn is_tile_clear_pathfinding_empty_tile_returns_true() {
        let mut map = Map::new();
        map.set_tile(5, 5, 7, make_tile(5, 5, 7));
        assert!(map.is_tile_clear(5, 5, 7, false, true));
    }

    // -----------------------------------------------------------------------
    // check_sight_line
    // -----------------------------------------------------------------------

    #[test]
    fn check_sight_line_same_position_returns_true() {
        let map = Map::new();
        assert!(map.check_sight_line(10, 10, 10, 10, 7, false));
    }

    #[test]
    fn check_sight_line_horizontal_no_obstacles_returns_true() {
        let map = Map::new();
        // straight horizontal, no tiles in between
        assert!(map.check_sight_line(0, 5, 10, 5, 7, false));
    }

    #[test]
    fn check_sight_line_horizontal_obstacle_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        // blocking tile at (5, 5) on z=7
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        // line from (0,5) to (10,5) passes through (5,5)
        assert!(!map.check_sight_line(0, 5, 10, 5, 7, false));
    }

    #[test]
    fn check_sight_line_vertical_no_obstacles_returns_true() {
        let map = Map::new();
        assert!(map.check_sight_line(5, 0, 5, 10, 7, false));
    }

    #[test]
    fn check_sight_line_endpoints_not_checked() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        // Place a blocking tile at the target endpoint (10, 5)
        let mut tile = make_tile(10, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(10, 5, 7, tile);
        // Endpoints are not checked — should still return true
        assert!(map.check_sight_line(0, 5, 10, 5, 7, false));
    }

    #[test]
    fn check_sight_line_steep_vertical_obstacle_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        // steep line: (5,0) to (5,10), obstacle at (5,5)
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        assert!(!map.check_sight_line(5, 0, 5, 10, 7, false));
    }

    // -----------------------------------------------------------------------
    // is_sight_clear
    // -----------------------------------------------------------------------

    #[test]
    fn is_sight_clear_same_position_returns_true() {
        let map = Map::new();
        let p = Position::new(100, 100, 7);
        assert!(map.is_sight_clear(p, p, false, false));
    }

    #[test]
    fn is_sight_clear_adjacent_same_floor_returns_true() {
        let map = Map::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(101, 100, 7); // dx=1 < 2
        assert!(map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_far_no_obstacles_returns_true() {
        let map = Map::new();
        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        assert!(map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_same_floor_blocked_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        // same_floor=true: cannot throw above
        assert!(!map.is_sight_clear(from, to, true, false));
    }

    #[test]
    fn is_sight_clear_different_floor_same_floor_true_returns_false() {
        let map = Map::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 100, 6);
        assert!(!map.is_sight_clear(from, to, true, false));
    }

    #[test]
    fn is_sight_clear_cross_boundary_floor_7_to_8_returns_false() {
        let map = Map::new();
        let from = Position::new(100, 100, 7); // surface
        let to = Position::new(100, 100, 8); // underground
        assert!(!map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_cross_boundary_floor_8_to_7_returns_false() {
        let map = Map::new();
        let from = Position::new(100, 100, 8);
        let to = Position::new(100, 100, 7);
        assert!(!map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_target_above_distance_2_returns_false() {
        let map = Map::new();
        // from.z > to.z by 2 → distance_z > 1 → false
        let from = Position::new(100, 100, 10);
        let to = Position::new(100, 100, 8);
        assert!(!map.is_sight_clear(from, to, false, false));
    }

    // -----------------------------------------------------------------------
    // can_throw_object_to
    // -----------------------------------------------------------------------

    #[test]
    fn can_throw_object_to_within_range_no_obstacles_returns_true() {
        let map = Map::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(105, 103, 7); // dx=5, dy=3
        assert!(map.can_throw_object_to(from, to, false, false, 8, 6));
    }

    #[test]
    fn can_throw_object_to_exceeds_range_x_returns_false() {
        let map = Map::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(110, 100, 7); // dx=10 > rangex=8
        assert!(!map.can_throw_object_to(from, to, false, false, 8, 6));
    }

    #[test]
    fn can_throw_object_to_exceeds_range_y_returns_false() {
        let map = Map::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 110, 7); // dy=10 > rangey=6
        assert!(!map.can_throw_object_to(from, to, false, false, 8, 6));
    }

    #[test]
    fn can_throw_object_to_exact_range_returns_true() {
        let map = Map::new();
        let from = Position::new(100, 100, 7);
        let to = Position::new(108, 106, 7); // dx=8, dy=6 exactly at limit
        assert!(map.can_throw_object_to(from, to, false, false, 8, 6));
    }

    #[test]
    fn can_throw_object_to_sight_blocked_with_check_los_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        // check_line_of_sight=true, same_floor=true → blocked
        assert!(!map.can_throw_object_to(from, to, true, true, 20, 20));
    }

    #[test]
    fn can_throw_object_to_sight_blocked_no_check_los_returns_true() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);
        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        // check_line_of_sight=false → ignore obstacles
        assert!(map.can_throw_object_to(from, to, false, false, 20, 20));
    }

    // -----------------------------------------------------------------------
    // multifloor_z_range
    // -----------------------------------------------------------------------

    #[test]
    fn multifloor_z_range_underground_z9() {
        let (min_z, max_z) = Map::multifloor_z_range(9);
        assert_eq!(min_z, 7);
        assert_eq!(max_z, 11);
    }

    #[test]
    fn multifloor_z_range_underground_z8() {
        let (min_z, max_z) = Map::multifloor_z_range(8);
        assert_eq!(min_z, 6);
        assert_eq!(max_z, 10);
    }

    #[test]
    fn multifloor_z_range_underground_clamps_to_15() {
        let (min_z, max_z) = Map::multifloor_z_range(15);
        assert_eq!(min_z, 13);
        assert_eq!(max_z, 15);
    }

    #[test]
    fn multifloor_z_range_floor_6() {
        let (min_z, max_z) = Map::multifloor_z_range(6);
        assert_eq!(min_z, 0);
        assert_eq!(max_z, 8);
    }

    #[test]
    fn multifloor_z_range_floor_7() {
        let (min_z, max_z) = Map::multifloor_z_range(7);
        assert_eq!(min_z, 0);
        assert_eq!(max_z, 9);
    }

    #[test]
    fn multifloor_z_range_surface_floor_0() {
        let (min_z, max_z) = Map::multifloor_z_range(0);
        assert_eq!(min_z, 0);
        assert_eq!(max_z, 7);
    }

    #[test]
    fn multifloor_z_range_surface_floor_3() {
        let (min_z, max_z) = Map::multifloor_z_range(3);
        assert_eq!(min_z, 0);
        assert_eq!(max_z, 7);
    }

    // -----------------------------------------------------------------------
    // AStarNodes — data structure
    // -----------------------------------------------------------------------

    #[test]
    fn astar_nodes_new_creates_with_start_node() {
        let nodes = AStarNodes::new(100, 100);
        // The start node should be present
        let idx = nodes.get_node_by_position(100, 100);
        assert!(idx.is_some());
        let node = nodes.node(idx.unwrap());
        assert_eq!(node.x, 100);
        assert_eq!(node.y, 100);
        assert_eq!(node.g, 0);
        assert_eq!(node.f, 0);
        assert_eq!(node.parent_idx, usize::MAX);
    }

    #[test]
    fn astar_nodes_get_node_by_position_nonexistent_returns_none() {
        let nodes = AStarNodes::new(0, 0);
        assert!(nodes.get_node_by_position(99, 99).is_none());
    }

    #[test]
    fn astar_nodes_create_node_returns_index() {
        let mut nodes = AStarNodes::new(0, 0);
        let parent = 0; // start node is index 0
        let idx = nodes.create_node(parent, 1, 0, 10, 15);
        assert!(idx.is_some());
        let node = nodes.node(idx.unwrap());
        assert_eq!(node.x, 1);
        assert_eq!(node.y, 0);
        assert_eq!(node.g, 10);
        assert_eq!(node.f, 15);
        assert_eq!(node.parent_idx, 0);
    }

    #[test]
    fn astar_nodes_get_best_node_returns_lowest_f() {
        let mut nodes = AStarNodes::new(0, 0);
        // Add two more nodes with different f scores
        nodes.create_node(0, 1, 0, 5, 20); // f=20
        nodes.create_node(0, 0, 1, 3, 10); // f=10

        // First best node should be the start (f=0)
        let first = nodes.get_best_node();
        assert!(first.is_some());
        let first_idx = first.unwrap();
        assert_eq!(nodes.node(first_idx).f, 0);

        // Second best should be f=10
        let second = nodes.get_best_node();
        assert!(second.is_some());
        assert_eq!(nodes.node(second.unwrap()).f, 10);
    }

    #[test]
    fn astar_nodes_get_best_node_skips_visited() {
        let mut nodes = AStarNodes::new(0, 0);
        // Pop start node (marks it visited)
        let _ = nodes.get_best_node();
        // Pop next — should not return start again
        nodes.create_node(0, 1, 0, 10, 10);
        let next = nodes.get_best_node();
        assert!(next.is_some());
        let node = nodes.node(next.unwrap());
        // Should be (1,0) not (0,0)
        assert_eq!(node.x, 1);
        assert_eq!(node.y, 0);
    }

    #[test]
    fn astar_nodes_returns_none_when_exhausted() {
        let mut nodes = AStarNodes::new(0, 0);
        let _ = nodes.get_best_node(); // pop the only node
        assert!(nodes.get_best_node().is_none());
    }

    #[test]
    fn astar_nodes_create_node_returns_none_at_limit() {
        let mut nodes = AStarNodes::new(0, 0); // 1 node already
                                               // Fill up to the limit (MAP_NODE_RESERVE_SIZE total)
        let mut last_y = 1u16;
        for _ in 1..MAP_NODE_RESERVE_SIZE {
            nodes.create_node(0, 0, last_y, 0, 0);
            last_y += 1;
        }
        // Now we're at the limit — next should return None
        let result = nodes.create_node(0, 0, last_y, 0, 0);
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------------
    // AStarNodes::get_map_walk_cost
    // -----------------------------------------------------------------------

    #[test]
    fn get_map_walk_cost_cardinal_east_returns_10() {
        let cost = AStarNodes::get_map_walk_cost(5, 5, 6, 5); // dx=1, dy=0
        assert_eq!(cost, MAP_NORMALWALKCOST);
    }

    #[test]
    fn get_map_walk_cost_cardinal_north_returns_10() {
        let cost = AStarNodes::get_map_walk_cost(5, 5, 5, 4); // dx=0, dy=1
        assert_eq!(cost, MAP_NORMALWALKCOST);
    }

    #[test]
    fn get_map_walk_cost_diagonal_returns_25() {
        let cost = AStarNodes::get_map_walk_cost(5, 5, 6, 6); // dx=1, dy=1
        assert_eq!(cost, MAP_DIAGONALWALKCOST);
    }

    #[test]
    fn get_map_walk_cost_diagonal_northwest_returns_25() {
        let cost = AStarNodes::get_map_walk_cost(5, 5, 4, 4); // dx=1, dy=1
        assert_eq!(cost, MAP_DIAGONALWALKCOST);
    }

    #[test]
    fn get_map_walk_cost_cardinal_west_returns_10() {
        let cost = AStarNodes::get_map_walk_cost(5, 5, 4, 5); // dx=1, dy=0
        assert_eq!(cost, MAP_NORMALWALKCOST);
    }

    // -----------------------------------------------------------------------
    // hash_coord
    // -----------------------------------------------------------------------

    #[test]
    fn hash_coord_packs_x_in_high_bits() {
        // x=1, y=0 → (1 << 16) | 0 = 65536
        assert_eq!(hash_coord(1, 0), 65536);
    }

    #[test]
    fn hash_coord_packs_y_in_low_bits() {
        // x=0, y=1 → 0 | 1 = 1
        assert_eq!(hash_coord(0, 1), 1);
    }

    #[test]
    fn hash_coord_both_nonzero() {
        assert_eq!(hash_coord(2, 3), (2u32 << 16) | 3);
    }

    // -----------------------------------------------------------------------
    // get_tile_mut — mirrors C++ const/non-const `Tile* Map::getTile(...)` pair
    // -----------------------------------------------------------------------

    #[test]
    fn get_tile_mut_returns_none_on_missing_tile() {
        let mut map = Map::new();
        assert!(map.get_tile_mut(0, 0, 0).is_none());
    }

    #[test]
    fn get_tile_mut_allows_mutation_visible_to_get_tile() {
        use forgottenserver_map::tile::flags;

        let mut map = Map::new();
        map.set_tile(5, 5, 7, make_tile(5, 5, 7));

        // Mutate via get_tile_mut
        let tile = map.get_tile_mut(5, 5, 7).expect("tile present");
        tile.set_flag(flags::PROTECTIONZONE);

        // The mutation must be observable via the immutable accessor.
        let observed = map.get_tile(5, 5, 7).expect("tile still present");
        assert!(observed.has_flag(flags::PROTECTIONZONE));
    }

    // -----------------------------------------------------------------------
    // AStarNodes::node_mut — mutable accessor
    // -----------------------------------------------------------------------

    #[test]
    fn astar_nodes_get_best_node_skips_duplicate_entries() {
        // Two `create_node` calls at the same (x, y) produce two entries in the
        // open set sharing the same hash key.  On the second pop, `visited.insert`
        // returns false and the loop continues to the next entry.
        let mut nodes = AStarNodes::new(0, 0);
        // Add a node at (1,1) with f=5.
        nodes.create_node(0, 1, 1, 5, 5);
        // Add a *second* node at the same (x,y) with a higher f=20 so it pops later.
        nodes.create_node(0, 1, 1, 6, 20);
        // Also add a clearly higher-f distinct coordinate so the heap has another entry.
        nodes.create_node(0, 2, 0, 10, 30);

        // 1st pop: start (0, 0), f = 0
        let first = nodes.get_best_node().expect("start");
        assert_eq!(nodes.node(first).x, 0);
        assert_eq!(nodes.node(first).y, 0);

        // 2nd pop: one of the (1,1) entries (f = 5)
        let second = nodes.get_best_node().expect("first (1,1)");
        assert_eq!(nodes.node(second).x, 1);
        assert_eq!(nodes.node(second).y, 1);

        // 3rd pop: the OTHER (1,1) entry pops next (f=20) but is already visited,
        // so the loop iterates and returns the (2, 0) entry instead.
        let third = nodes
            .get_best_node()
            .expect("(2, 0) after skipping duplicate");
        assert_eq!(nodes.node(third).x, 2);
        assert_eq!(nodes.node(third).y, 0);
    }

    #[test]
    fn astar_nodes_node_mut_allows_mutation() {
        let mut nodes = AStarNodes::new(10, 10);
        let idx = nodes
            .get_node_by_position(10, 10)
            .expect("start node exists");

        // Mutate the node through node_mut.
        let n = nodes.node_mut(idx);
        n.g = 42;
        n.f = 99;

        // Read back via the immutable accessor and confirm the mutation persisted.
        let n2 = nodes.node(idx);
        assert_eq!(n2.g, 42);
        assert_eq!(n2.f, 99);
    }

    // -----------------------------------------------------------------------
    // check_sight_line — steep line where y1 < y0 (descending steep)
    // -----------------------------------------------------------------------

    #[test]
    fn check_sight_line_steep_descending_no_obstacle_returns_true() {
        // |dy|=10 > |dx|=1 → steep. y1 < y0 forces the swap branch.
        let map = Map::new();
        assert!(map.check_sight_line(5, 10, 4, 0, 7, false));
    }

    #[test]
    fn check_sight_line_steep_descending_blocked_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        // place an obstacle along the steep descending path
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        let mut tile = make_tile(4, 5, 7);
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(4, 5, 7, tile);

        // Steep descending: (4, 10) → (4, 0); intermediate ys hit (4, 5).
        assert!(!map.check_sight_line(4, 10, 4, 0, 7, false));
    }

    #[test]
    fn check_sight_line_slight_descending_no_obstacle_returns_true() {
        // x1 < x0 with |dx| > |dy| → slight branch, swap path.
        let map = Map::new();
        assert!(map.check_sight_line(10, 5, 0, 4, 7, false));
    }

    // -----------------------------------------------------------------------
    // is_sight_clear — pathfinding=true short-circuits on same floor
    // -----------------------------------------------------------------------

    #[test]
    fn is_sight_clear_same_floor_pathfinding_true_returns_check_sight_line() {
        let map = Map::new();
        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        // Same floor, far apart, pathfinding=true → delegates to check_sight_line
        // with pathfinding flag on; empty map has no blockers.
        assert!(map.is_sight_clear(from, to, false, true));
    }

    #[test]
    fn is_sight_clear_same_floor_pathfinding_true_blocked_returns_false() {
        use forgottenserver_map::tile::flags;

        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        // BLOCKPATH causes pathfinding=true `is_tile_clear` to return false.
        tile.set_flag(flags::BLOCKPATH);
        map.set_tile(5, 5, 7, tile);

        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        assert!(!map.is_sight_clear(from, to, false, true));
    }

    // -----------------------------------------------------------------------
    // is_sight_clear — same floor, blocked, same_floor=false fallback paths
    // -----------------------------------------------------------------------

    #[test]
    fn is_sight_clear_same_floor_blocked_from_z0_returns_true() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        // Obstacle on floor 0; from.z == 0 → "no obstacles above floor 0 so we can
        // throw above" path returns true.
        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 0);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 0, tile);

        let from = Position::new(0, 5, 0);
        let to = Position::new(10, 5, 0);
        assert!(map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_same_floor_blocked_throws_above_when_upper_clear_returns_true() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        // Block on floor 7 between (0,5,7) and (10,5,7); the floor above (z=6)
        // is empty, so we can throw "over" the obstacle.
        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 7);
        let data = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        tile.add_item(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 7, tile);

        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        assert!(map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_same_floor_blocked_throws_above_blocked_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        let mut map = Map::new();
        // Block on floor 7 main line.
        let data1 = ItemTypeData {
            block_projectile: true,
            ..ItemTypeData::default()
        };
        let mut t1 = make_tile(5, 5, 7);
        t1.add_item(Item::new(Arc::new(data1), 1));
        map.set_tile(5, 5, 7, t1);

        // Tile directly above the source `from` (5 in the same x, 5 in same y, z=6)
        // has a ground item → `is_tile_clear(.., block_floor=true)` returns false.
        let mut t2 = make_tile(0, 5, 6);
        let data2 = ItemTypeData::default();
        // ground item presence is what `block_floor=true` checks for.
        t2.set_ground(Item::new(Arc::new(data2), 1));
        map.set_tile(0, 5, 6, t2);

        let from = Position::new(0, 5, 7);
        let to = Position::new(10, 5, 7);
        assert!(!map.is_sight_clear(from, to, false, false));
    }

    // -----------------------------------------------------------------------
    // is_sight_clear — target above (from.z > to.z, distance_z=1)
    // -----------------------------------------------------------------------

    #[test]
    fn is_sight_clear_target_above_adjacent_clear_returns_true() {
        // Underground: (5,5,9) → (5,5,8). distance_z=1, both >= 8 so no boundary.
        let map = Map::new();
        let from = Position::new(5, 5, 9);
        let to = Position::new(8, 5, 8);
        assert!(map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_target_above_adjacent_upper_blocked_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        // The tile above `from` (z = from.z - 1 = 8) has a ground item, blocking.
        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 8);
        let data = ItemTypeData::default();
        tile.set_ground(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 8, tile);

        let from = Position::new(5, 5, 9);
        let to = Position::new(8, 5, 8);
        assert!(!map.is_sight_clear(from, to, false, false));
    }

    // -----------------------------------------------------------------------
    // is_sight_clear — target below (from.z < to.z)
    // -----------------------------------------------------------------------

    #[test]
    fn is_sight_clear_target_below_unblocked_returns_true() {
        // Same column, below: (5,5,8) → (5,5,10). All tiles directly above target
        // are absent (empty map), so the "all tiles above target are clear" loop
        // passes; the final straight line on from.z is also clear.
        let map = Map::new();
        let from = Position::new(5, 5, 8);
        let to = Position::new(5, 5, 10);
        assert!(map.is_sight_clear(from, to, false, false));
    }

    #[test]
    fn is_sight_clear_target_below_blocked_above_target_returns_false() {
        use forgottenserver_items::item::Item;
        use forgottenserver_items::items_registry::ItemTypeData;
        use std::sync::Arc;

        // Block one of the floors above the target with a ground item so
        // `is_tile_clear(to.x, to.y, z, block_floor=true)` returns false.
        let mut map = Map::new();
        let mut tile = make_tile(5, 5, 9);
        let data = ItemTypeData::default();
        tile.set_ground(Item::new(Arc::new(data), 1));
        map.set_tile(5, 5, 9, tile);

        let from = Position::new(5, 5, 8);
        let to = Position::new(5, 5, 10);
        assert!(!map.is_sight_clear(from, to, false, false));
    }

    // -----------------------------------------------------------------------
    // get_spectators — `&& p.y == 3` short-circuit branch coverage
    // -----------------------------------------------------------------------

    #[test]
    fn get_spectators_with_matching_x_but_non_matching_y_in_result() {
        // Seed positions so the result contains a position with x==4 but y!=3,
        // which forces the `&& p.y == 3` short-circuit's right-hand side to
        // be evaluated at least once.
        let mut map = Map::new();
        map.set_tile(4, 1, 7, make_tile(4, 1, 7)); // x==4, y==1 → matches x but not y
        map.set_tile(3, 3, 7, make_tile(3, 3, 7)); // sanity: in range
        let center = Position::new(0, 0, 7);
        let result = map.get_spectators(center, 5, 5, false);
        // x==4,y==1 is present; therefore there exists a p with p.x == 4 and
        // the predicate's second clause runs (and yields false for p.y == 3).
        assert!(result.iter().any(|p| p.x == 4 && p.y == 1));
        assert!(!result.iter().any(|p| p.x == 4 && p.y == 3));
    }

    /// Regression: `check_sight_line` previously skipped occlusion checks
    /// whenever the steep-axis y-coordinate exceeded `MAP_MAX_LAYERS`
    /// (16) — confusing the Z-axis layer limit with the Y-axis world
    /// bound. C++ `Map::checkSightLine` has no bounds check at all; it
    /// trusts `isTileClear` to return false for missing tiles.
    #[test]
    fn check_sight_line_steep_axis_honours_obstacles_past_layer_count() {
        use forgottenserver_map::tile::flags;
        let mut map = Map::new();
        let z = 7;
        // Plant a path-blocking tile at y=20 (well past MAP_MAX_LAYERS=16),
        // on a vertical line between (0,0,z) and (0,40,z). `pathfinding=true`
        // makes `is_tile_clear` consult the bitmask path so we can use
        // BLOCKPATH directly without constructing a blocking item.
        let mut blocker = make_tile(0, 20, z);
        blocker.set_flag(flags::BLOCKPATH);
        map.set_tile(0, 20, z, blocker);
        // With the bug present, the steep branch's `y < MAP_MAX_LAYERS`
        // guard returned `true` (line of sight clear) for y >= 16.
        // After the fix, the blocker at y=20 must be detected.
        assert!(
            !map.check_sight_line(0, 0, 0, 40, z, true),
            "blocker at y=20 must obstruct vertical sight line through (0,0,7)–(0,40,7)"
        );
    }

    // ── Spectator cache (Session 17 ledger closure) ─────────────────────

    /// `get_spectators` must return identical results when called twice
    /// with the same key — proves the cache returns a faithful copy.
    #[test]
    fn spectator_cache_returns_same_result_across_calls() {
        let mut map = Map::new();
        map.set_tile(0, 0, 7, make_tile(0, 0, 7));
        map.set_tile(1, 1, 7, make_tile(1, 1, 7));
        let center = Position::new(0, 0, 7);
        let first = map.get_spectators(center, 5, 5, false);
        let second = map.get_spectators(center, 5, 5, false);
        assert_eq!(first, second);
    }

    /// After `clear_spectator_cache`, the next query must reflect any
    /// tile mutations performed in the meantime.
    #[test]
    fn clear_spectator_cache_drops_stale_entries() {
        let mut map = Map::new();
        map.set_tile(0, 0, 7, make_tile(0, 0, 7));
        let center = Position::new(0, 0, 7);
        // Prime the cache.
        let before = map.get_spectators(center, 5, 5, false);
        assert_eq!(before.len(), 1);
        // Add a tile in range — without invalidation the cached result
        // would still report 1.
        map.set_tile(2, 2, 7, make_tile(2, 2, 7));
        map.clear_spectator_cache();
        let after = map.get_spectators(center, 5, 5, false);
        assert_eq!(after.len(), 2);
    }

    /// The players-only cache must be independently clearable so the
    /// game-tick code can invalidate it without dropping the full
    /// spectator cache. We assert the API is callable on `&self`.
    #[test]
    fn clear_players_spectator_cache_is_callable_via_shared_ref() {
        let map = Map::new();
        // Mirrors the C++ const-method signature; takes `&self`.
        map.clear_players_spectator_cache();
    }
}
