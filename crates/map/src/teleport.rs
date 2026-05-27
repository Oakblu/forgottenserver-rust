// Migrated from forgottenserver/src/teleport.h + teleport.cpp
//
// Teleport is an Item subtype that carries a destination Position.
//
// C++ serialization (serializeAttr / readAttr):
//   ATTR_TELE_DEST (u8 tag) followed by x(u16 LE) + y(u16 LE) + z(u8)
//
// We model Teleport as a standalone struct that holds:
//   - item_type_id: u16      (mirrors the `id` field from Item)
//   - dest_pos: Position     (default Position{0,0,0})

use forgottenserver_common::position::Position;

// ATTR_TELE_DEST tag value matches C++ AttrTypes_t::ATTR_TELE_DEST = 17
pub const ATTR_TELE_DEST: u8 = 17;

// ---------------------------------------------------------------------------
// Teleport
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Teleport {
    pub item_type_id: u16,
    dest_pos: Position,
}

impl Teleport {
    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    /// Creates a new teleport item of the given type with no destination set
    /// (destination defaults to Position{0,0,0}).
    pub fn new(item_type_id: u16) -> Self {
        Teleport {
            item_type_id,
            dest_pos: Position::new(0, 0, 0),
        }
    }

    // -----------------------------------------------------------------------
    // Destination accessors — mirrors getDestPos / setDestPos
    // -----------------------------------------------------------------------

    pub fn get_dest_pos(&self) -> Position {
        self.dest_pos
    }

    pub fn set_dest_pos(&mut self, pos: Position) {
        self.dest_pos = pos;
    }

    // -----------------------------------------------------------------------
    // Serialization — mirrors Teleport::serializeAttr + readAttr
    //
    // Format written: [ATTR_TELE_DEST][x_lo][x_hi][y_lo][y_hi][z]
    // (5 data bytes after the 1-byte tag)
    // -----------------------------------------------------------------------

    /// Serialise destination position into bytes.
    ///
    /// Matches C++ `serializeAttr`:
    ///   write ATTR_TELE_DEST tag (u8)
    ///   write x as u16 LE
    ///   write y as u16 LE
    ///   write z as u8
    pub fn serialize_dest(&self) -> [u8; 6] {
        let [x0, x1] = self.dest_pos.x.to_le_bytes();
        let [y0, y1] = self.dest_pos.y.to_le_bytes();
        [ATTR_TELE_DEST, x0, x1, y0, y1, self.dest_pos.z]
    }

    /// Deserialise a destination position from bytes produced by
    /// [`serialize_dest`].
    ///
    /// Returns `None` if the bytes are too short or the tag does not match
    /// `ATTR_TELE_DEST`.
    pub fn deserialize_dest(bytes: &[u8]) -> Option<Position> {
        if bytes.len() < 6 {
            return None;
        }
        if bytes[0] != ATTR_TELE_DEST {
            return None;
        }
        let x = u16::from_le_bytes([bytes[1], bytes[2]]);
        let y = u16::from_le_bytes([bytes[3], bytes[4]]);
        let z = bytes[5];
        Some(Position::new(x, y, z))
    }

    /// Mirrors C++ `Teleport* getTeleport() override final { return this; }`.
    /// Always returns `&self` — the C++ virtual override exists so a base-
    /// class `Item*` pointer can be downcast to `Teleport*`. Rust doesn't
    /// need the dispatch but the API surface keeps cross-crate callers
    /// tidy.
    pub fn get_teleport(&self) -> &Teleport {
        self
    }
}

// ---------------------------------------------------------------------------
// Teleport chain-loop detector (Session 23 ledger closure)
// ---------------------------------------------------------------------------

/// Outcome of the chain-walk inside `Teleport::addThing`. The Rust port
/// returns the routing decision as data so cross-crate callers (which
/// own `g_game.map.getTile` + `tile->getTeleportItem`) dispatch on it.
///
/// Mirrors the C++ control flow in `Teleport::addThing`:
///   * destination tile missing → caller drops the move
///   * destination tile is a teleport that chains back to an already-
///     visited position → infinite loop, abort
///   * otherwise → the resolved final destination position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleportChainOutcome {
    /// Caller should drop the move (C++ early `return` on null destTile).
    DestinationMissing,
    /// A teleport cycle was detected. `at` is the position that closed
    /// the loop — useful for logging (C++ prints
    /// `"Warning: possible infinite loop teleport. <pos>"`).
    InfiniteLoop { at: Position },
    /// Final destination after walking the teleport chain. When no
    /// chaining occurred this equals the immediate `dest_pos`.
    FinalDestination(Position),
}

/// Pure chain-walk for `Teleport::addThing`. The caller supplies:
///
/// * `start_pos` — the teleport's `getPosition()` (the cycle detector
///   seeds the visited set with it so a teleport that points to itself
///   is caught).
/// * `dest_pos` — the immediate `dest_pos` of *this* teleport.
/// * `dest_tile_resolved` — whether `g_game.map.getTile(dest_pos)`
///   returned a non-null tile. When false the C++ side aborts early.
/// * `next_teleport_at` — closure that, given a position, resolves to
///   `Some(next_dest_pos)` when that tile holds a teleport, else
///   `None`. Encodes `tile->getTeleportItem()->getDestPos()` from C++.
pub fn detect_teleport_chain_outcome<F>(
    start_pos: Position,
    dest_pos: Position,
    dest_tile_resolved: bool,
    mut next_teleport_at: F,
) -> TeleportChainOutcome
where
    F: FnMut(Position) -> Option<Position>,
{
    if !dest_tile_resolved {
        return TeleportChainOutcome::DestinationMissing;
    }
    let mut visited = vec![start_pos];
    let mut current = dest_pos;
    loop {
        match next_teleport_at(current) {
            Some(next_pos) => {
                if visited.contains(&next_pos) {
                    return TeleportChainOutcome::InfiniteLoop { at: next_pos };
                }
                visited.push(current);
                current = next_pos;
            }
            None => {
                // No teleport at `current`, or the underlying tile is
                // absent — the C++ loop terminates here too.
                return TeleportChainOutcome::FinalDestination(current);
            }
        }
    }
}

/// Returns the turn direction a teleported creature should face on
/// arrival. Mirrors C++:
///   `g_game.internalCreatureTurn(creature,
///        origPos.x > destPos.x ? DIRECTION_WEST : DIRECTION_EAST);`
///
/// Note the strict `>`: equal x defaults to East to match the C++
/// branch order.
pub fn turn_direction_for_teleport(
    orig_x: u16,
    dest_x: u16,
) -> forgottenserver_common::position::Direction {
    use forgottenserver_common::position::Direction;
    if orig_x > dest_x {
        Direction::West
    } else {
        Direction::East
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Constructor
    // -----------------------------------------------------------------------

    #[test]
    fn teleport_new_stores_type_id() {
        let t = Teleport::new(1234);
        assert_eq!(t.item_type_id, 1234);
    }

    #[test]
    fn teleport_new_dest_pos_is_zero() {
        let t = Teleport::new(1);
        assert_eq!(t.get_dest_pos(), Position::new(0, 0, 0));
    }

    // -----------------------------------------------------------------------
    // set_dest_pos / get_dest_pos
    // -----------------------------------------------------------------------

    #[test]
    fn teleport_set_and_get_dest_pos() {
        let mut t = Teleport::new(1);
        let pos = Position::new(100, 200, 7);
        t.set_dest_pos(pos);
        assert_eq!(t.get_dest_pos(), pos);
    }

    #[test]
    fn teleport_overwrite_dest_pos() {
        let mut t = Teleport::new(1);
        t.set_dest_pos(Position::new(1, 2, 3));
        t.set_dest_pos(Position::new(50, 60, 4));
        assert_eq!(t.get_dest_pos(), Position::new(50, 60, 4));
    }

    // -----------------------------------------------------------------------
    // Serialization round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn serialize_then_deserialize_gives_same_position() {
        let mut t = Teleport::new(99);
        let original = Position::new(12345, 54321, 10);
        t.set_dest_pos(original);

        let bytes = t.serialize_dest();
        let recovered = Teleport::deserialize_dest(&bytes).expect("should deserialize");
        assert_eq!(recovered, original);
    }

    #[test]
    fn serialize_zero_pos_round_trip() {
        let t = Teleport::new(1);
        let bytes = t.serialize_dest();
        let recovered = Teleport::deserialize_dest(&bytes).expect("should deserialize");
        assert_eq!(recovered, Position::new(0, 0, 0));
    }

    #[test]
    fn serialize_max_xy_pos_round_trip() {
        let mut t = Teleport::new(1);
        t.set_dest_pos(Position::new(u16::MAX, u16::MAX, u8::MAX));
        let bytes = t.serialize_dest();
        let recovered = Teleport::deserialize_dest(&bytes).unwrap();
        assert_eq!(recovered, Position::new(u16::MAX, u16::MAX, u8::MAX));
    }

    #[test]
    fn serialized_first_byte_is_attr_tag() {
        let t = Teleport::new(1);
        let bytes = t.serialize_dest();
        assert_eq!(bytes[0], ATTR_TELE_DEST);
    }

    #[test]
    fn deserialize_wrong_tag_returns_none() {
        let bytes = [0xFF_u8, 0, 0, 0, 0, 0];
        assert!(Teleport::deserialize_dest(&bytes).is_none());
    }

    #[test]
    fn deserialize_too_short_returns_none() {
        let bytes = [ATTR_TELE_DEST, 0, 0, 0, 0];
        assert!(Teleport::deserialize_dest(&bytes).is_none());
    }

    #[test]
    fn deserialize_empty_returns_none() {
        assert!(Teleport::deserialize_dest(&[]).is_none());
    }

    #[test]
    fn attr_tele_dest_value() {
        // Matches C++ AttrTypes_t::ATTR_TELE_DEST = 17
        assert_eq!(ATTR_TELE_DEST, 17);
    }

    // -----------------------------------------------------------------------
    // Serialized byte layout — exact positions (mirrors C++ write order)
    // C++: write u8 tag, u16 x LE, u16 y LE, u8 z
    // -----------------------------------------------------------------------

    #[test]
    fn serialize_byte_layout_tag_at_index_0() {
        let mut t = Teleport::new(1);
        t.set_dest_pos(Position::new(0x0102, 0x0304, 0x05));
        let b = t.serialize_dest();
        assert_eq!(
            b[0], ATTR_TELE_DEST,
            "byte 0 must be the ATTR_TELE_DEST tag"
        );
    }

    #[test]
    fn serialize_byte_layout_x_little_endian() {
        let mut t = Teleport::new(1);
        // x = 0x0201 → LE bytes [0x01, 0x02]
        t.set_dest_pos(Position::new(0x0201, 0, 0));
        let b = t.serialize_dest();
        assert_eq!(b[1], 0x01, "byte 1 must be low byte of x");
        assert_eq!(b[2], 0x02, "byte 2 must be high byte of x");
    }

    #[test]
    fn serialize_byte_layout_y_little_endian() {
        let mut t = Teleport::new(1);
        // y = 0x0403 → LE bytes [0x03, 0x04]
        t.set_dest_pos(Position::new(0, 0x0403, 0));
        let b = t.serialize_dest();
        assert_eq!(b[3], 0x03, "byte 3 must be low byte of y");
        assert_eq!(b[4], 0x04, "byte 4 must be high byte of y");
    }

    #[test]
    fn serialize_byte_layout_z_at_index_5() {
        let mut t = Teleport::new(1);
        t.set_dest_pos(Position::new(0, 0, 7));
        let b = t.serialize_dest();
        assert_eq!(b[5], 7, "byte 5 must be z");
    }

    #[test]
    fn serialize_output_length_is_exactly_6() {
        let t = Teleport::new(1);
        assert_eq!(t.serialize_dest().len(), 6);
    }

    // -----------------------------------------------------------------------
    // deserialize_dest — extra trailing bytes are tolerated
    // Mirrors C++ readAttr which reads exactly 5 data bytes and stops.
    // -----------------------------------------------------------------------

    #[test]
    fn deserialize_tolerates_extra_trailing_bytes() {
        let mut t = Teleport::new(1);
        t.set_dest_pos(Position::new(10, 20, 3));
        let mut extended = t.serialize_dest().to_vec();
        extended.extend_from_slice(&[0xFF, 0xAB, 0xCD]); // trailing garbage
        let recovered = Teleport::deserialize_dest(&extended).expect("should deserialize");
        assert_eq!(recovered, Position::new(10, 20, 3));
    }

    // -----------------------------------------------------------------------
    // addThing hook simulation
    // C++ addThing(thing): looks up destPos tile and moves creature/item there.
    // In Rust we have no global game state, so we test the invariant:
    // after set_dest_pos, get_dest_pos returns the stored destination which
    // callers use as the teleport target.  We simulate the "hook fires →
    // destination is read" pattern.
    // -----------------------------------------------------------------------

    #[test]
    fn on_add_creature_hook_reads_stored_dest_pos() {
        // Simulate: teleport is placed at origin, destination is set.
        let mut tp = Teleport::new(10);
        let dest = Position::new(500, 600, 7);
        tp.set_dest_pos(dest);

        // Simulate the hook: caller reads get_dest_pos to know where to move.
        let hook_dest = tp.get_dest_pos();
        assert_eq!(hook_dest, dest, "hook must see the configured destination");
    }

    #[test]
    fn on_add_creature_hook_destination_can_be_changed_between_calls() {
        let mut tp = Teleport::new(5);
        tp.set_dest_pos(Position::new(1, 1, 1));

        // First "add" reads first dest
        assert_eq!(tp.get_dest_pos(), Position::new(1, 1, 1));

        // Destination is reconfigured
        tp.set_dest_pos(Position::new(999, 888, 5));

        // Second "add" reads new dest
        assert_eq!(tp.get_dest_pos(), Position::new(999, 888, 5));
    }

    // -----------------------------------------------------------------------
    // queryRemove — always returns no-error (mirrors C++ queryRemove)
    // Rust equivalent: serialization/deserialization never rejects; the struct
    // is always constructable and readable.  We verify the struct is usable
    // after any sequence of set/get operations (no panic, no error state).
    // -----------------------------------------------------------------------

    #[test]
    fn teleport_is_always_in_valid_state_after_set_dest_pos() {
        let mut t = Teleport::new(1);
        // Simulate setting and unsetting destination multiple times
        for i in 0..10_u16 {
            t.set_dest_pos(Position::new(i * 10, i * 20, (i % 15) as u8));
        }
        // Still accessible, no panic
        let _ = t.get_dest_pos();
        let _ = t.serialize_dest();
    }

    // -----------------------------------------------------------------------
    // item_type_id field is preserved through set_dest_pos operations
    // -----------------------------------------------------------------------

    #[test]
    fn item_type_id_unaffected_by_dest_pos_changes() {
        let mut t = Teleport::new(7777);
        t.set_dest_pos(Position::new(100, 200, 5));
        assert_eq!(t.item_type_id, 7777);
        t.set_dest_pos(Position::new(0, 0, 0));
        assert_eq!(t.item_type_id, 7777);
    }

    // -----------------------------------------------------------------------
    // Clone — Teleport implements Clone; dest_pos is deep-copied
    // -----------------------------------------------------------------------

    #[test]
    fn clone_preserves_dest_pos_independently() {
        let mut original = Teleport::new(3);
        original.set_dest_pos(Position::new(50, 50, 5));
        let mut clone = original.clone();
        clone.set_dest_pos(Position::new(99, 99, 9));
        // Original must not be affected
        assert_eq!(original.get_dest_pos(), Position::new(50, 50, 5));
        assert_eq!(clone.get_dest_pos(), Position::new(99, 99, 9));
    }

    // ── Cylinder-protocol decision helpers (Session 23) ─────────────────

    /// Destination tile missing → caller drops the move.
    #[test]
    fn chain_outcome_missing_destination_signals_drop() {
        let outcome = detect_teleport_chain_outcome(
            Position::new(0, 0, 7),
            Position::new(10, 10, 7),
            false, // dest tile NOT resolved
            |_| None,
        );
        assert_eq!(outcome, TeleportChainOutcome::DestinationMissing);
    }

    /// Destination tile present + no further teleport → final dest equals
    /// the immediate dest_pos.
    #[test]
    fn chain_outcome_no_chain_returns_immediate_dest() {
        let dest = Position::new(10, 10, 7);
        let outcome = detect_teleport_chain_outcome(Position::new(0, 0, 7), dest, true, |_| None);
        assert_eq!(outcome, TeleportChainOutcome::FinalDestination(dest));
    }

    /// Chain that walks through one or more teleports and terminates on
    /// a non-teleport tile → the final non-teleport position.
    #[test]
    fn chain_outcome_walks_chain_until_non_teleport() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(20, 20, 7);
        let c = Position::new(30, 30, 7);
        // a → b → c → (none)
        let outcome = detect_teleport_chain_outcome(Position::new(0, 0, 7), a, true, |p| {
            if p == a {
                Some(b)
            } else if p == b {
                Some(c)
            } else {
                None
            }
        });
        assert_eq!(outcome, TeleportChainOutcome::FinalDestination(c));
    }

    /// Cycle that returns to the start position is caught.
    #[test]
    fn chain_outcome_self_loop_caught() {
        let start = Position::new(0, 0, 7);
        let dest = Position::new(10, 10, 7);
        // dest → start (a teleport at dest that points back to the
        // teleport's own origin position).
        let outcome = detect_teleport_chain_outcome(start, dest, true, |p| {
            if p == dest {
                Some(start)
            } else {
                None
            }
        });
        assert_eq!(outcome, TeleportChainOutcome::InfiniteLoop { at: start });
    }

    /// Cycle midway through the chain is caught (not just the start
    /// position).
    #[test]
    fn chain_outcome_midchain_cycle_caught() {
        let a = Position::new(10, 10, 7);
        let b = Position::new(20, 20, 7);
        // dest_pos `a` → `b` → `a` (cycle back to `a` which is already
        // in the visited set after the first hop).
        let outcome = detect_teleport_chain_outcome(Position::new(0, 0, 7), a, true, |p| {
            if p == a {
                Some(b)
            } else if p == b {
                Some(a)
            } else {
                None
            }
        });
        assert_eq!(outcome, TeleportChainOutcome::InfiniteLoop { at: a });
    }

    /// `turn_direction_for_teleport`: orig.x > dest.x → west.
    #[test]
    fn turn_direction_west_when_orig_x_greater() {
        use forgottenserver_common::position::Direction;
        assert_eq!(turn_direction_for_teleport(100, 50), Direction::West);
    }

    /// orig.x < dest.x → east.
    #[test]
    fn turn_direction_east_when_orig_x_smaller() {
        use forgottenserver_common::position::Direction;
        assert_eq!(turn_direction_for_teleport(10, 50), Direction::East);
    }

    /// orig.x == dest.x → east (matches C++ branch order — strict `>`
    /// for west, else east).
    #[test]
    fn turn_direction_east_when_orig_x_equal() {
        use forgottenserver_common::position::Direction;
        assert_eq!(turn_direction_for_teleport(50, 50), Direction::East);
    }

    /// `get_teleport()` returns `&self` — identity accessor.
    #[test]
    fn get_teleport_returns_self_reference() {
        let mut t = Teleport::new(1);
        t.set_dest_pos(Position::new(7, 7, 7));
        let returned = t.get_teleport();
        assert_eq!(returned.get_dest_pos(), Position::new(7, 7, 7));
        // Pointer equality — the C++ `return this;` guarantees identity.
        assert!(std::ptr::eq(returned, &t));
    }
}
