//! Migrated from forgottenserver/src/position.h and forgottenserver/src/tools.cpp
//! Position struct, Direction enum, and associated utility functions.

#![allow(dead_code)]

use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// Direction
// ---------------------------------------------------------------------------

/// Direction enum – mirrors C++ `Direction : uint8_t` from position.h.
///
/// Discriminants match the C++ values exactly so that serialised values are
/// wire-compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
    Southwest = 4, // DIRECTION_DIAGONAL_MASK | 0
    Southeast = 5, // DIRECTION_DIAGONAL_MASK | 1
    Northwest = 6, // DIRECTION_DIAGONAL_MASK | 2
    Northeast = 7, // DIRECTION_DIAGONAL_MASK | 3
    None = 8,
}

/// Names for each direction (mirrors an implicit DIRECTION_DEF table used
/// throughout the C++ code-base).
pub const DIRECTION_NAMES: [(&str, Direction); 8] = [
    ("North", Direction::North),
    ("East", Direction::East),
    ("South", Direction::South),
    ("West", Direction::West),
    ("Southwest", Direction::Southwest),
    ("Southeast", Direction::Southeast),
    ("Northwest", Direction::Northwest),
    ("Northeast", Direction::Northeast),
];

impl Direction {
    /// Returns the name string for this direction (empty string for `None`).
    pub fn name(self) -> &'static str {
        match self {
            Direction::North => "North",
            Direction::East => "East",
            Direction::South => "South",
            Direction::West => "West",
            Direction::Southwest => "Southwest",
            Direction::Southeast => "Southeast",
            Direction::Northwest => "Northwest",
            Direction::Northeast => "Northeast",
            Direction::None => "",
        }
    }

    /// Returns `(dx, dy)` deltas for this direction.
    ///
    /// * North     → (0, -1)  (y decreases)
    /// * South     → (0, +1)
    /// * East      → (+1, 0)
    /// * West      → (-1, 0)
    /// * NorthEast → (+1, -1)
    /// * NorthWest → (-1, -1)
    /// * SouthEast → (+1, +1)
    /// * SouthWest → (-1, +1)
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::Northeast => (1, -1),
            Direction::Northwest => (-1, -1),
            Direction::Southeast => (1, 1),
            Direction::Southwest => (-1, 1),
            Direction::None => (0, 0),
        }
    }

    /// Returns the opposite direction.
    pub fn reverse(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::Northwest => Direction::Southeast,
            Direction::Northeast => Direction::Southwest,
            Direction::Southwest => Direction::Northeast,
            Direction::Southeast => Direction::Northwest,
            Direction::None => Direction::None,
        }
    }
}

/// Error returned when parsing a [`Direction`] from a string fails.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseDirectionError(String);

impl fmt::Display for ParseDirectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown direction: {:?}", self.0)
    }
}

/// Case-insensitive parse of a direction name.
///
/// Accepted values (case-insensitive): `"North"`, `"East"`, `"South"`,
/// `"West"`, `"Southwest"`, `"Southeast"`, `"Northwest"`, `"Northeast"`,
/// `"None"`.
impl FromStr for Direction {
    type Err = ParseDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "north" => Ok(Direction::North),
            "east" => Ok(Direction::East),
            "south" => Ok(Direction::South),
            "west" => Ok(Direction::West),
            "southwest" => Ok(Direction::Southwest),
            "southeast" => Ok(Direction::Southeast),
            "northwest" => Ok(Direction::Northwest),
            "northeast" => Ok(Direction::Northeast),
            "none" => Ok(Direction::None),
            _ => Err(ParseDirectionError(s.to_owned())),
        }
    }
}

// ---------------------------------------------------------------------------
// Position
// ---------------------------------------------------------------------------

/// Game world position. Mirrors `struct Position` from position.h.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

impl Position {
    /// Creates a new position.
    pub const fn new(x: u16, y: u16, z: u8) -> Self {
        Position { x, y, z }
    }

    // --- offset helpers (signed) ---

    pub fn offset_x(self, other: Position) -> i32 {
        self.x as i32 - other.x as i32
    }

    pub fn offset_y(self, other: Position) -> i32 {
        self.y as i32 - other.y as i32
    }

    pub fn offset_z(self, other: Position) -> i16 {
        self.z as i16 - other.z as i16
    }

    // --- distance helpers (absolute values) ---

    pub fn distance_x(self, other: Position) -> i32 {
        self.offset_x(other).abs()
    }

    pub fn distance_y(self, other: Position) -> i32 {
        self.offset_y(other).abs()
    }

    pub fn distance_z(self, other: Position) -> i16 {
        self.offset_z(other).abs()
    }

    /// Chebyshev distance: `max(|dx|, |dy|)`.
    pub fn distance(self, other: Position) -> i32 {
        self.distance_x(other).max(self.distance_y(other))
    }

    /// Returns `true` if `other` is within the given x/y range.
    pub fn is_in_range(self, other: Position, delta_x: i32, delta_y: i32) -> bool {
        self.distance_x(other) <= delta_x && self.distance_y(other) <= delta_y
    }

    /// Returns `true` if `other` is within the given x/y/z range.
    pub fn is_in_range_3d(self, other: Position, delta_x: i32, delta_y: i32, delta_z: i16) -> bool {
        self.distance_x(other) <= delta_x
            && self.distance_y(other) <= delta_y
            && self.distance_z(other) <= delta_z
    }

    /// Returns the position one step in the given direction (saturating on
    /// overflow so values remain valid `u16`/`u8`).
    pub fn next_position(self, dir: Direction) -> Position {
        let (dx, dy) = dir.delta();
        Position {
            x: (self.x as i32 + dx).clamp(0, u16::MAX as i32) as u16,
            y: (self.y as i32 + dy).clamp(0, u16::MAX as i32) as u16,
            z: self.z,
        }
    }

    /// Returns `true` if `other` is adjacent (Chebyshev distance exactly 1).
    ///
    /// Positions on different floors are never considered adjacent.
    pub fn is_adjacent(self, other: Position) -> bool {
        self.z == other.z && self.distance(other) == 1
    }

    /// Returns the position offset by the given direction delta.
    ///
    /// This is equivalent to [`next_position`] but exposed under the name used
    /// by several C++ callers (`getOffsetPosition`).  Values clamp to the
    /// valid `u16`/`u8` range.
    pub fn get_offset_position(self, dir: Direction) -> Position {
        self.next_position(dir)
    }
}

// --- Ordering mirrors C++: lexicographic on (z, y, x) ---
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.z, self.y, self.x).cmp(&(other.z, other.y, other.x))
    }
}

/// Display: mirrors the C++ `operator<<` format
/// `( 00100 / 00200 / 007 )`
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "( {:05} / {:05} / {:03} )",
            self.x, self.y, self.z as u16
        )
    }
}

// ---------------------------------------------------------------------------
// Free functions (from tools.cpp / tools.h)
// ---------------------------------------------------------------------------

/// Returns the direction to move from `from` towards `to`.
///
/// Mirrors `Direction getDirectionTo(const Position& from, const Position& to)`
/// in tools.cpp.
pub fn get_direction_to(from: Position, to: Position) -> Direction {
    if from == to {
        return Direction::None;
    }

    let x_raw = from.offset_x(to); // from.x - to.x
    let (dir_x, x_offset) = if x_raw < 0 {
        (Direction::East, x_raw.unsigned_abs() as i32)
    } else {
        (Direction::West, x_raw)
    };

    let y_raw = from.offset_y(to); // from.y - to.y
    if y_raw >= 0 {
        // to.y <= from.y → moving North (decreasing y)
        let y_offset = y_raw;
        if y_offset > x_offset {
            Direction::North
        } else if y_offset == x_offset {
            if dir_x == Direction::East {
                Direction::Northeast
            } else {
                Direction::Northwest
            }
        } else {
            dir_x
        }
    } else {
        let y_offset = -y_raw;
        if y_offset > x_offset {
            Direction::South
        } else if y_offset == x_offset {
            if dir_x == Direction::East {
                Direction::Southeast
            } else {
                Direction::Southwest
            }
        } else {
            dir_x
        }
    }
}

/// Returns the opposite direction. Mirrors `Game.getReverseDirection` (Lua)
/// and the standard inverse mapping.
pub fn get_reverse_direction(dir: Direction) -> Direction {
    dir.reverse()
}

/// Client viewport constants (from map.h).
///
/// The client viewport is 18 × 14 tiles centred on the player:
/// `maxClientViewportX = 8` → half-width  (total = 2*8+2 = 18)
/// `maxClientViewportY = 6` → half-height (total = 2*6+2 = 14)
pub const MAX_CLIENT_VIEWPORT_X: i32 = 8;
pub const MAX_CLIENT_VIEWPORT_Y: i32 = 6;

/// Returns `true` if `to` is within the client viewport centred on `from`.
///
/// When `multifloor` is `false` (the normal case for client rendering), both
/// positions must be on the same floor (`z` must match).
///
/// Mirrors `Creature::canSee` in creature.cpp with the client-viewport range
/// (8, 6) — not the extended server range (11, 11).
pub fn is_in_viewport(from: Position, to: Position, multifloor: bool) -> bool {
    if !multifloor && from.z != to.z {
        return false;
    }

    // For the flat (non-multifloor) check we ignore z entirely once floors match.
    let dx = from.distance_x(to);
    let dy = from.distance_y(to);
    dx <= MAX_CLIENT_VIEWPORT_X && dy <= MAX_CLIENT_VIEWPORT_Y
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Direction discriminants ---

    #[test]
    fn test_direction_discriminants() {
        assert_eq!(Direction::North as u8, 0);
        assert_eq!(Direction::East as u8, 1);
        assert_eq!(Direction::South as u8, 2);
        assert_eq!(Direction::West as u8, 3);
        assert_eq!(Direction::Southwest as u8, 4);
        assert_eq!(Direction::Southeast as u8, 5);
        assert_eq!(Direction::Northwest as u8, 6);
        assert_eq!(Direction::Northeast as u8, 7);
        assert_eq!(Direction::None as u8, 8);
    }

    // --- Direction names ---

    #[test]
    fn test_direction_names() {
        assert_eq!(Direction::North.name(), "North");
        assert_eq!(Direction::East.name(), "East");
        assert_eq!(Direction::South.name(), "South");
        assert_eq!(Direction::West.name(), "West");
        assert_eq!(Direction::Southwest.name(), "Southwest");
        assert_eq!(Direction::Southeast.name(), "Southeast");
        assert_eq!(Direction::Northwest.name(), "Northwest");
        assert_eq!(Direction::Northeast.name(), "Northeast");
        assert_eq!(Direction::None.name(), "");
    }

    // --- Direction deltas ---

    #[test]
    fn test_direction_delta_north() {
        assert_eq!(Direction::North.delta(), (0, -1));
    }

    #[test]
    fn test_direction_delta_south() {
        assert_eq!(Direction::South.delta(), (0, 1));
    }

    #[test]
    fn test_direction_delta_east() {
        assert_eq!(Direction::East.delta(), (1, 0));
    }

    #[test]
    fn test_direction_delta_west() {
        assert_eq!(Direction::West.delta(), (-1, 0));
    }

    #[test]
    fn test_direction_delta_northeast() {
        assert_eq!(Direction::Northeast.delta(), (1, -1));
    }

    #[test]
    fn test_direction_delta_northwest() {
        assert_eq!(Direction::Northwest.delta(), (-1, -1));
    }

    #[test]
    fn test_direction_delta_southeast() {
        assert_eq!(Direction::Southeast.delta(), (1, 1));
    }

    #[test]
    fn test_direction_delta_southwest() {
        assert_eq!(Direction::Southwest.delta(), (-1, 1));
    }

    #[test]
    fn test_direction_delta_none() {
        assert_eq!(Direction::None.delta(), (0, 0));
    }

    // --- getReverseDirection ---

    #[test]
    fn test_reverse_direction_cardinal() {
        assert_eq!(get_reverse_direction(Direction::North), Direction::South);
        assert_eq!(get_reverse_direction(Direction::South), Direction::North);
        assert_eq!(get_reverse_direction(Direction::East), Direction::West);
        assert_eq!(get_reverse_direction(Direction::West), Direction::East);
    }

    #[test]
    fn test_reverse_direction_diagonal() {
        assert_eq!(
            get_reverse_direction(Direction::Northwest),
            Direction::Southeast
        );
        assert_eq!(
            get_reverse_direction(Direction::Northeast),
            Direction::Southwest
        );
        assert_eq!(
            get_reverse_direction(Direction::Southwest),
            Direction::Northeast
        );
        assert_eq!(
            get_reverse_direction(Direction::Southeast),
            Direction::Northwest
        );
    }

    #[test]
    fn test_reverse_direction_none() {
        assert_eq!(get_reverse_direction(Direction::None), Direction::None);
    }

    // --- Position struct basics ---

    #[test]
    fn test_position_default() {
        let p = Position::default();
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
        assert_eq!(p.z, 0);
    }

    #[test]
    fn test_position_new() {
        let p = Position::new(100, 200, 7);
        assert_eq!(p.x, 100);
        assert_eq!(p.y, 200);
        assert_eq!(p.z, 7);
    }

    // --- Equality operators ---

    #[test]
    fn test_position_equality() {
        let a = Position::new(10, 20, 5);
        let b = Position::new(10, 20, 5);
        let c = Position::new(10, 21, 5);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // --- Ordering: (z, y, x) ---

    #[test]
    fn test_position_ordering_by_z_first() {
        let low_z = Position::new(999, 999, 1);
        let high_z = Position::new(0, 0, 2);
        assert!(low_z < high_z);
    }

    #[test]
    fn test_position_ordering_by_y_second() {
        let low_y = Position::new(999, 5, 1);
        let high_y = Position::new(0, 10, 1);
        assert!(low_y < high_y);
    }

    #[test]
    fn test_position_ordering_by_x_last() {
        let low_x = Position::new(3, 5, 1);
        let high_x = Position::new(7, 5, 1);
        assert!(low_x < high_x);
    }

    // --- Display ---

    #[test]
    fn test_position_display() {
        let p = Position::new(100, 200, 7);
        assert_eq!(format!("{p}"), "( 00100 / 00200 / 007 )");
    }

    #[test]
    fn test_position_display_zeros() {
        let p = Position::new(0, 0, 0);
        assert_eq!(format!("{p}"), "( 00000 / 00000 / 000 )");
    }

    // --- distance (Chebyshev) ---

    #[test]
    fn test_distance_same_position() {
        let p = Position::new(10, 10, 7);
        assert_eq!(p.distance(p), 0);
    }

    #[test]
    fn test_distance_horizontal() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(15, 10, 7);
        assert_eq!(a.distance(b), 5);
    }

    #[test]
    fn test_distance_vertical() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(10, 3, 7);
        assert_eq!(a.distance(b), 7);
    }

    #[test]
    fn test_distance_diagonal_chebyshev() {
        // dx=3, dy=4 → max=4 (not Euclidean sqrt(25))
        let a = Position::new(10, 10, 7);
        let b = Position::new(13, 14, 7);
        assert_eq!(a.distance(b), 4);
    }

    #[test]
    fn test_distance_symmetric() {
        let a = Position::new(5, 3, 1);
        let b = Position::new(8, 7, 1);
        assert_eq!(a.distance(b), b.distance(a));
    }

    // --- is_in_range ---

    #[test]
    fn test_is_in_range_true() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(11, 11, 7);
        assert!(a.is_in_range(b, 1, 1));
    }

    #[test]
    fn test_is_in_range_false() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(12, 10, 7);
        assert!(!a.is_in_range(b, 1, 1));
    }

    #[test]
    fn test_is_in_range_exact_boundary() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(10, 12, 7);
        assert!(a.is_in_range(b, 5, 2));
        assert!(!a.is_in_range(b, 5, 1));
    }

    #[test]
    fn test_is_in_range_3d() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(10, 10, 8);
        assert!(a.is_in_range_3d(b, 0, 0, 1));
        assert!(!a.is_in_range_3d(b, 0, 0, 0));
    }

    // --- getNextPosition ---

    #[test]
    fn test_next_position_north() {
        let p = Position::new(100, 100, 7);
        assert_eq!(p.next_position(Direction::North), Position::new(100, 99, 7));
    }

    #[test]
    fn test_next_position_south() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.next_position(Direction::South),
            Position::new(100, 101, 7)
        );
    }

    #[test]
    fn test_next_position_east() {
        let p = Position::new(100, 100, 7);
        assert_eq!(p.next_position(Direction::East), Position::new(101, 100, 7));
    }

    #[test]
    fn test_next_position_west() {
        let p = Position::new(100, 100, 7);
        assert_eq!(p.next_position(Direction::West), Position::new(99, 100, 7));
    }

    #[test]
    fn test_next_position_northeast() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.next_position(Direction::Northeast),
            Position::new(101, 99, 7)
        );
    }

    #[test]
    fn test_next_position_northwest() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.next_position(Direction::Northwest),
            Position::new(99, 99, 7)
        );
    }

    #[test]
    fn test_next_position_southeast() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.next_position(Direction::Southeast),
            Position::new(101, 101, 7)
        );
    }

    #[test]
    fn test_next_position_southwest() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.next_position(Direction::Southwest),
            Position::new(99, 101, 7)
        );
    }

    #[test]
    fn test_next_position_none() {
        let p = Position::new(100, 100, 7);
        assert_eq!(p.next_position(Direction::None), p);
    }

    #[test]
    fn test_next_position_saturates_at_zero() {
        let p = Position::new(0, 0, 7);
        // Moving North or West from (0,0) should clamp, not wrap
        let north = p.next_position(Direction::North);
        assert_eq!(north.y, 0);
        let west = p.next_position(Direction::West);
        assert_eq!(west.x, 0);
    }

    // --- getDirectionTo ---

    #[test]
    fn test_get_direction_to_same_position() {
        let p = Position::new(100, 100, 7);
        assert_eq!(get_direction_to(p, p), Direction::None);
    }

    #[test]
    fn test_get_direction_to_north() {
        // to.y < from.y → North
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 90, 7);
        assert_eq!(get_direction_to(from, to), Direction::North);
    }

    #[test]
    fn test_get_direction_to_south() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 110, 7);
        assert_eq!(get_direction_to(from, to), Direction::South);
    }

    #[test]
    fn test_get_direction_to_east() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(110, 100, 7);
        assert_eq!(get_direction_to(from, to), Direction::East);
    }

    #[test]
    fn test_get_direction_to_west() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(90, 100, 7);
        assert_eq!(get_direction_to(from, to), Direction::West);
    }

    #[test]
    fn test_get_direction_to_northeast() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(105, 95, 7); // dx=5 east, dy=5 north → exactly diagonal
        assert_eq!(get_direction_to(from, to), Direction::Northeast);
    }

    #[test]
    fn test_get_direction_to_northwest() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(95, 95, 7);
        assert_eq!(get_direction_to(from, to), Direction::Northwest);
    }

    #[test]
    fn test_get_direction_to_southeast() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(105, 105, 7);
        assert_eq!(get_direction_to(from, to), Direction::Southeast);
    }

    #[test]
    fn test_get_direction_to_southwest() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(95, 105, 7);
        assert_eq!(get_direction_to(from, to), Direction::Southwest);
    }

    // Dominant-axis tests (not exact diagonal)
    #[test]
    fn test_get_direction_to_mostly_north() {
        // dy > dx → North
        let from = Position::new(100, 100, 7);
        let to = Position::new(101, 90, 7); // dx=1 east, dy=10 north
        assert_eq!(get_direction_to(from, to), Direction::North);
    }

    #[test]
    fn test_get_direction_to_mostly_east() {
        // dx > dy → East
        let from = Position::new(100, 100, 7);
        let to = Position::new(110, 101, 7); // dx=10 east, dy=1 south
        assert_eq!(get_direction_to(from, to), Direction::East);
    }

    // --- isInViewport ---

    #[test]
    fn test_is_in_viewport_same_position() {
        let p = Position::new(100, 100, 7);
        assert!(is_in_viewport(p, p, false));
    }

    #[test]
    fn test_is_in_viewport_within_bounds() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(108, 106, 7); // dx=8, dy=6 — exactly on boundary
        assert!(is_in_viewport(from, to, false));
    }

    #[test]
    fn test_is_in_viewport_x_out_of_bounds() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(109, 100, 7); // dx=9 > 8
        assert!(!is_in_viewport(from, to, false));
    }

    #[test]
    fn test_is_in_viewport_y_out_of_bounds() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 107, 7); // dy=7 > 6
        assert!(!is_in_viewport(from, to, false));
    }

    #[test]
    fn test_is_in_viewport_different_floor_without_multifloor() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 100, 6); // same xy, different floor
        assert!(!is_in_viewport(from, to, false));
    }

    #[test]
    fn test_is_in_viewport_different_floor_with_multifloor() {
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 100, 6);
        assert!(is_in_viewport(from, to, true));
    }

    // --- offset helpers ---

    #[test]
    fn test_offset_x() {
        let a = Position::new(10, 0, 0);
        let b = Position::new(3, 0, 0);
        assert_eq!(a.offset_x(b), 7);
        assert_eq!(b.offset_x(a), -7);
    }

    #[test]
    fn test_offset_y() {
        let a = Position::new(0, 20, 0);
        let b = Position::new(0, 5, 0);
        assert_eq!(a.offset_y(b), 15);
        assert_eq!(b.offset_y(a), -15);
    }

    #[test]
    fn test_offset_z() {
        let a = Position::new(0, 0, 10);
        let b = Position::new(0, 0, 3);
        assert_eq!(a.offset_z(b), 7);
        assert_eq!(b.offset_z(a), -7);
    }

    // --- DIRECTION_NAMES table ---

    #[test]
    fn test_direction_names_table_completeness() {
        // Verify that all 8 non-None directions appear in the table
        assert_eq!(DIRECTION_NAMES.len(), 8);
        let dirs: Vec<Direction> = DIRECTION_NAMES.iter().map(|(_, d)| *d).collect();
        assert!(dirs.contains(&Direction::North));
        assert!(dirs.contains(&Direction::East));
        assert!(dirs.contains(&Direction::South));
        assert!(dirs.contains(&Direction::West));
        assert!(dirs.contains(&Direction::Southwest));
        assert!(dirs.contains(&Direction::Southeast));
        assert!(dirs.contains(&Direction::Northwest));
        assert!(dirs.contains(&Direction::Northeast));
    }

    // --- Direction::from_str round-trip for all 8 variants ---

    #[test]
    fn test_direction_from_str_north() {
        let d: Direction = "North".parse().unwrap();
        assert_eq!(d, Direction::North);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::North);
    }

    #[test]
    fn test_direction_from_str_east() {
        let d: Direction = "East".parse().unwrap();
        assert_eq!(d, Direction::East);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::East);
    }

    #[test]
    fn test_direction_from_str_south() {
        let d: Direction = "South".parse().unwrap();
        assert_eq!(d, Direction::South);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::South);
    }

    #[test]
    fn test_direction_from_str_west() {
        let d: Direction = "West".parse().unwrap();
        assert_eq!(d, Direction::West);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::West);
    }

    #[test]
    fn test_direction_from_str_southwest() {
        let d: Direction = "Southwest".parse().unwrap();
        assert_eq!(d, Direction::Southwest);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::Southwest);
    }

    #[test]
    fn test_direction_from_str_southeast() {
        let d: Direction = "Southeast".parse().unwrap();
        assert_eq!(d, Direction::Southeast);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::Southeast);
    }

    #[test]
    fn test_direction_from_str_northwest() {
        let d: Direction = "Northwest".parse().unwrap();
        assert_eq!(d, Direction::Northwest);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::Northwest);
    }

    #[test]
    fn test_direction_from_str_northeast() {
        let d: Direction = "Northeast".parse().unwrap();
        assert_eq!(d, Direction::Northeast);
        assert_eq!(d.name().parse::<Direction>().unwrap(), Direction::Northeast);
    }

    #[test]
    fn test_direction_from_str_case_insensitive() {
        assert_eq!("NORTH".parse::<Direction>().unwrap(), Direction::North);
        assert_eq!("north".parse::<Direction>().unwrap(), Direction::North);
        assert_eq!("nOrTh".parse::<Direction>().unwrap(), Direction::North);
        assert_eq!(
            "northeast".parse::<Direction>().unwrap(),
            Direction::Northeast
        );
    }

    #[test]
    fn test_direction_from_str_invalid_returns_error() {
        assert!("unknown".parse::<Direction>().is_err());
        assert!("".parse::<Direction>().is_err());
        assert!("N".parse::<Direction>().is_err());
    }

    #[test]
    fn test_direction_from_str_none() {
        assert_eq!("none".parse::<Direction>().unwrap(), Direction::None);
        assert_eq!("None".parse::<Direction>().unwrap(), Direction::None);
    }

    // --- is_adjacent boundary: distance == 1 vs distance == 2 ---

    #[test]
    fn test_is_adjacent_cardinal_distance_1() {
        let center = Position::new(100, 100, 7);
        // All 4 cardinal neighbours at Chebyshev distance 1
        assert!(center.is_adjacent(Position::new(100, 99, 7))); // North
        assert!(center.is_adjacent(Position::new(100, 101, 7))); // South
        assert!(center.is_adjacent(Position::new(101, 100, 7))); // East
        assert!(center.is_adjacent(Position::new(99, 100, 7))); // West
    }

    #[test]
    fn test_is_adjacent_diagonal_distance_1() {
        let center = Position::new(100, 100, 7);
        // All 4 diagonal neighbours at Chebyshev distance 1
        assert!(center.is_adjacent(Position::new(101, 99, 7))); // NE
        assert!(center.is_adjacent(Position::new(99, 99, 7))); // NW
        assert!(center.is_adjacent(Position::new(101, 101, 7))); // SE
        assert!(center.is_adjacent(Position::new(99, 101, 7))); // SW
    }

    #[test]
    fn test_is_adjacent_distance_2_not_adjacent() {
        let center = Position::new(100, 100, 7);
        // Chebyshev distance == 2: not adjacent
        assert!(!center.is_adjacent(Position::new(102, 100, 7)));
        assert!(!center.is_adjacent(Position::new(100, 102, 7)));
        assert!(!center.is_adjacent(Position::new(102, 102, 7)));
    }

    #[test]
    fn test_is_adjacent_same_position_not_adjacent() {
        let p = Position::new(100, 100, 7);
        // Distance == 0: not adjacent (adjacency requires exactly 1)
        assert!(!p.is_adjacent(p));
    }

    #[test]
    fn test_is_adjacent_different_floor_not_adjacent() {
        let a = Position::new(100, 100, 7);
        let b = Position::new(100, 101, 6); // 1 step south but different floor
        assert!(!a.is_adjacent(b));
    }

    // --- get_offset_position for each direction delta ---

    #[test]
    fn test_get_offset_position_north() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::North),
            Position::new(100, 99, 7)
        );
    }

    #[test]
    fn test_get_offset_position_south() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::South),
            Position::new(100, 101, 7)
        );
    }

    #[test]
    fn test_get_offset_position_east() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::East),
            Position::new(101, 100, 7)
        );
    }

    #[test]
    fn test_get_offset_position_west() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::West),
            Position::new(99, 100, 7)
        );
    }

    #[test]
    fn test_get_offset_position_northeast() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::Northeast),
            Position::new(101, 99, 7)
        );
    }

    #[test]
    fn test_get_offset_position_northwest() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::Northwest),
            Position::new(99, 99, 7)
        );
    }

    #[test]
    fn test_get_offset_position_southeast() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::Southeast),
            Position::new(101, 101, 7)
        );
    }

    #[test]
    fn test_get_offset_position_southwest() {
        let p = Position::new(100, 100, 7);
        assert_eq!(
            p.get_offset_position(Direction::Southwest),
            Position::new(99, 101, 7)
        );
    }

    #[test]
    fn test_get_offset_position_none_stays_put() {
        let p = Position::new(100, 100, 7);
        assert_eq!(p.get_offset_position(Direction::None), p);
    }

    #[test]
    fn test_get_offset_position_saturates_at_zero() {
        let p = Position::new(0, 0, 7);
        assert_eq!(p.get_offset_position(Direction::North).y, 0);
        assert_eq!(p.get_offset_position(Direction::West).x, 0);
    }

    // --- can_see viewport math at exact 18×14 boundary ---
    // The client viewport is 18 wide × 14 tall, centred on the player.
    // Half-extents: MAX_CLIENT_VIEWPORT_X=8, MAX_CLIENT_VIEWPORT_Y=6.

    #[test]
    fn test_can_see_exact_x_boundary_inside() {
        // dx == MAX_CLIENT_VIEWPORT_X (8) → visible
        let from = Position::new(100, 100, 7);
        let to = Position::new(108, 100, 7);
        assert!(is_in_viewport(from, to, false));
    }

    #[test]
    fn test_can_see_exact_x_boundary_outside() {
        // dx == MAX_CLIENT_VIEWPORT_X + 1 (9) → not visible
        let from = Position::new(100, 100, 7);
        let to = Position::new(109, 100, 7);
        assert!(!is_in_viewport(from, to, false));
    }

    #[test]
    fn test_can_see_exact_y_boundary_inside() {
        // dy == MAX_CLIENT_VIEWPORT_Y (6) → visible
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 106, 7);
        assert!(is_in_viewport(from, to, false));
    }

    #[test]
    fn test_can_see_exact_y_boundary_outside() {
        // dy == MAX_CLIENT_VIEWPORT_Y + 1 (7) → not visible
        let from = Position::new(100, 100, 7);
        let to = Position::new(100, 107, 7);
        assert!(!is_in_viewport(from, to, false));
    }

    #[test]
    fn test_can_see_corner_boundary_inside() {
        // Both dx and dy at their maximum (8, 6) → visible (corner of viewport)
        let from = Position::new(100, 100, 7);
        let to = Position::new(108, 106, 7);
        assert!(is_in_viewport(from, to, false));
    }

    #[test]
    fn test_can_see_corner_boundary_x_over() {
        // dx=9, dy=6 → not visible (x exceeds half-width by 1)
        let from = Position::new(100, 100, 7);
        let to = Position::new(109, 106, 7);
        assert!(!is_in_viewport(from, to, false));
    }

    #[test]
    fn test_can_see_corner_boundary_y_over() {
        // dx=8, dy=7 → not visible (y exceeds half-height by 1)
        let from = Position::new(100, 100, 7);
        let to = Position::new(108, 107, 7);
        assert!(!is_in_viewport(from, to, false));
    }

    // --- ParseDirectionError ---
    // Covers the Display impl (uncovered in baseline), the PartialEq derive,
    // the Debug derive, and the round-trip from a failing FromStr parse.

    #[test]
    fn test_parse_direction_error_display_format() {
        // Reaching the error branch of FromStr should yield a
        // ParseDirectionError that round-trips through Display with the
        // original (unmodified) input wrapped in Rust's {:?} debug quoting.
        let err = "bogus".parse::<Direction>().unwrap_err();
        assert_eq!(format!("{err}"), "unknown direction: \"bogus\"");
    }

    #[test]
    fn test_parse_direction_error_display_preserves_input_casing() {
        // Display should echo the original input exactly (including case),
        // not the lowercased lookup key.
        let err = "WeIrD".parse::<Direction>().unwrap_err();
        assert_eq!(format!("{err}"), "unknown direction: \"WeIrD\"");
    }

    #[test]
    fn test_parse_direction_error_empty_string() {
        let err = "".parse::<Direction>().unwrap_err();
        assert_eq!(format!("{err}"), "unknown direction: \"\"");
    }

    #[test]
    fn test_parse_direction_error_debug_and_eq() {
        // PartialEq + Debug derives are part of the public API of
        // ParseDirectionError; assert they behave as documented.
        let a = "foo".parse::<Direction>().unwrap_err();
        let b = "foo".parse::<Direction>().unwrap_err();
        let c = "bar".parse::<Direction>().unwrap_err();
        assert_eq!(a, b);
        assert_ne!(a, c);
        // Debug formatting should at minimum mention the wrapped string.
        let dbg = format!("{a:?}");
        assert!(
            dbg.contains("foo"),
            "Debug output should contain the input: {dbg}"
        );
    }

    // --- direct distance_x / distance_y / distance_z assertions ---
    // The baseline tests only exercise these helpers transitively through
    // `is_in_range` and `distance`. Add direct assertions so each public
    // method has a test that meaningfully asserts on its observable output.

    #[test]
    fn test_distance_x_absolute_value() {
        // distance_x must be the absolute value of offset_x regardless of
        // which position is the receiver.
        let a = Position::new(10, 0, 0);
        let b = Position::new(3, 0, 0);
        assert_eq!(a.distance_x(b), 7);
        assert_eq!(b.distance_x(a), 7);
    }

    #[test]
    fn test_distance_y_absolute_value() {
        let a = Position::new(0, 20, 0);
        let b = Position::new(0, 5, 0);
        assert_eq!(a.distance_y(b), 15);
        assert_eq!(b.distance_y(a), 15);
    }

    #[test]
    fn test_distance_z_absolute_value() {
        let a = Position::new(0, 0, 10);
        let b = Position::new(0, 0, 3);
        assert_eq!(a.distance_z(b), 7);
        assert_eq!(b.distance_z(a), 7);
    }

    // --- Direction discriminants encode the DIRECTION_DIAGONAL_MASK ---

    #[test]
    fn test_diagonal_mask_property() {
        // C++ defines DIRECTION_DIAGONAL_MASK = 4 and the diagonal
        // directions as DIAGONAL_MASK | <cardinal>. Verify the
        // discriminants preserve that bit pattern.
        const DIAGONAL_MASK: u8 = 4;
        assert_eq!(
            Direction::Southwest as u8,
            DIAGONAL_MASK | (Direction::North as u8)
        );
        assert_eq!(
            Direction::Southeast as u8,
            DIAGONAL_MASK | (Direction::East as u8)
        );
        assert_eq!(
            Direction::Northwest as u8,
            DIAGONAL_MASK | (Direction::South as u8)
        );
        assert_eq!(
            Direction::Northeast as u8,
            DIAGONAL_MASK | (Direction::West as u8)
        );
        // And DIRECTION_LAST == NORTHEAST in C++ (= 7).
        assert_eq!(Direction::Northeast as u8, 7);
    }

    // --- get_direction_to: every non-zero x_raw branch ---
    // The branch where the y-axis dominates and x_raw is on the "else" side
    // (positive x_raw → dir_x = West) when y_raw < 0 needs an explicit test
    // to be sure each branch path returns the correct dominant direction.

    #[test]
    fn test_get_direction_to_mostly_south_with_west_x() {
        // dx=1 west (positive x_raw), dy=10 south → South (y dominates)
        let from = Position::new(100, 100, 7);
        let to = Position::new(99, 110, 7);
        assert_eq!(get_direction_to(from, to), Direction::South);
    }

    #[test]
    fn test_get_direction_to_mostly_west() {
        // dx=10 west, dy=1 north → West (x dominates, x_raw positive branch)
        let from = Position::new(100, 100, 7);
        let to = Position::new(90, 99, 7);
        assert_eq!(get_direction_to(from, to), Direction::West);
    }
}
