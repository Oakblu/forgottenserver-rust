//! Migrated from forgottenserver/src/podium.h and podium.cpp
//!
//! Provides the `Podium` struct — an item variant that displays outfits on
//! a decorative platform.

use forgottenserver_common::enums::Outfit;
use forgottenserver_common::position::Direction;

// ---------------------------------------------------------------------------
// PodiumFlags
// ---------------------------------------------------------------------------

/// Mirrors the C++ `PodiumFlags` enum from const.h.
///
/// The flags are used as bit positions in a 3-bit mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PodiumFlags {
    /// Bit 0 — show the platform below the outfit.
    ShowPlatform = 0,
    /// Bit 1 — show the outfit on the podium.
    ShowOutfit = 1,
    /// Bit 2 — show the mount on the podium.
    ShowMount = 2,
}

// ---------------------------------------------------------------------------
// Podium
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Podium` class.
///
/// Composed from an item-type id rather than inheriting from `Item`.
#[derive(Debug, Clone)]
pub struct Podium {
    /// Item type id (server id).
    pub item_type_id: u16,
    /// The outfit currently displayed on the podium.
    pub outfit: Outfit,
    /// Direction the outfit faces.
    pub direction: Direction,
    /// 3-bit flag field (same layout as C++ `std::bitset<3>`).
    ///
    /// Bit 0 = ShowPlatform, Bit 1 = ShowOutfit, Bit 2 = ShowMount.
    flags: u8,
}

impl Podium {
    /// Create a new Podium with default values.
    ///
    /// Mirrors `Podium(uint16_t type) : Item(type) {}` plus the field
    /// initialisers in the C++ class:
    /// * `flags = {true}` → bit 0 (ShowPlatform) is set
    /// * `direction = DIRECTION_SOUTH`
    pub fn new(item_type_id: u16) -> Self {
        Podium {
            item_type_id,
            outfit: Outfit::default(),
            direction: Direction::South,
            // bit 0 (ShowPlatform) is set; mirrors `std::bitset<3> flags = {true}`
            flags: 1,
        }
    }

    // -----------------------------------------------------------------------
    // Outfit
    // -----------------------------------------------------------------------

    pub fn get_outfit(&self) -> &Outfit {
        &self.outfit
    }

    pub fn set_outfit(&mut self, outfit: Outfit) {
        self.outfit = outfit;
    }

    // -----------------------------------------------------------------------
    // Direction
    // -----------------------------------------------------------------------

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, dir: Direction) {
        self.direction = dir;
    }

    // -----------------------------------------------------------------------
    // Flags
    // -----------------------------------------------------------------------

    /// Returns `true` when the given flag bit is set.
    pub fn has_flag(&self, flag: PodiumFlags) -> bool {
        self.flags & (1 << flag as u8) != 0
    }

    /// Sets or clears the given flag bit.
    pub fn set_flag(&mut self, flag: PodiumFlags, value: bool) {
        if value {
            self.flags |= 1 << flag as u8;
        } else {
            self.flags &= !(1 << flag as u8);
        }
    }

    /// Set all flags at once from a raw byte (mirrors `setFlags(uint8_t)`).
    pub fn set_flags(&mut self, raw: u8) {
        self.flags = raw & 0b111; // only 3 bits
    }

    // -----------------------------------------------------------------------
    // Serialization helpers
    // -----------------------------------------------------------------------

    /// Serialize the flags byte (for wire format / persistence).
    pub fn serialize_flags(&self) -> u8 {
        self.flags
    }

    /// Serialize the direction byte (wire-compatible with C++ discriminants).
    pub fn serialize_direction(&self) -> u8 {
        self.direction as u8
    }

    // -----------------------------------------------------------------------
    // Wire-format attribute serialization (mirrors C++ `serializeAttr` and
    // `readAttr` for ATTR_PODIUMOUTFIT = 40)
    // -----------------------------------------------------------------------

    /// C++ `AttrTypes_t::ATTR_PODIUMOUTFIT` tag byte (value 40 / 0x28).
    pub const ATTR_PODIUMOUTFIT: u8 = 40;

    /// Total payload size after the tag byte: 1 flags + 1 direction +
    /// outfit (2+1+1+1+1+1 + 2+1+1+1+1) = 15 bytes. Mirrors the C++
    /// `propStream.size() < 15` guard in `Podium::readAttr`.
    pub const PODIUMOUTFIT_PAYLOAD_LEN: usize = 15;

    /// Serialize the podium attribute block to a byte vector, mirroring
    /// `Podium::serializeAttr` from podium.cpp. The output is:
    /// `[ATTR_PODIUMOUTFIT, flags, direction, lookType_le, lookHead,
    ///   lookBody, lookLegs, lookFeet, lookAddons, lookMount_le,
    ///   lookMountHead, lookMountBody, lookMountLegs, lookMountFeet]`
    /// (1 + 15 = 16 bytes).
    pub fn serialize_attr(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + Self::PODIUMOUTFIT_PAYLOAD_LEN);
        buf.push(Self::ATTR_PODIUMOUTFIT);
        buf.push(self.flags);
        buf.push(self.direction as u8);
        buf.extend_from_slice(&self.outfit.look_type.to_le_bytes());
        buf.push(self.outfit.look_head);
        buf.push(self.outfit.look_body);
        buf.push(self.outfit.look_legs);
        buf.push(self.outfit.look_feet);
        buf.push(self.outfit.look_addons);
        buf.extend_from_slice(&self.outfit.look_mount.to_le_bytes());
        buf.push(self.outfit.look_mount_head);
        buf.push(self.outfit.look_mount_body);
        buf.push(self.outfit.look_mount_legs);
        buf.push(self.outfit.look_mount_feet);
        buf
    }

    /// Parse a payload produced by `serialize_attr` (excluding the leading
    /// tag byte) and populate this `Podium`. Mirrors the `case
    /// ATTR_PODIUMOUTFIT` branch of `Podium::readAttr`. Returns `false` when
    /// `payload.len() < 15` (matches C++ `ATTR_READ_ERROR`).
    pub fn read_attr_podiumoutfit(&mut self, payload: &[u8]) -> bool {
        if payload.len() < Self::PODIUMOUTFIT_PAYLOAD_LEN {
            return false;
        }
        self.set_flags(payload[0]);
        // `Direction` is `#[repr(u8)]` with values 0..=7. Diagonal masks
        // beyond that range fall back to `South` to mirror the C++ cast
        // semantics for unknown discriminants in serialized data.
        self.direction = match payload[1] {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            4 => Direction::Southwest,
            5 => Direction::Southeast,
            6 => Direction::Northwest,
            7 => Direction::Northeast,
            _ => Direction::South,
        };
        let look_type = u16::from_le_bytes([payload[2], payload[3]]);
        let look_head = payload[4];
        let look_body = payload[5];
        let look_legs = payload[6];
        let look_feet = payload[7];
        let look_addons = payload[8];
        let look_mount = u16::from_le_bytes([payload[9], payload[10]]);
        let look_mount_head = payload[11];
        let look_mount_body = payload[12];
        let look_mount_legs = payload[13];
        let look_mount_feet = payload[14];
        self.outfit = forgottenserver_common::enums::Outfit {
            look_type,
            look_type_ex: self.outfit.look_type_ex, // preserved; C++ does not touch it
            look_head,
            look_body,
            look_legs,
            look_feet,
            look_addons,
            look_mount,
            look_mount_head,
            look_mount_body,
            look_mount_legs,
            look_mount_feet,
        };
        true
    }
}

// ---------------------------------------------------------------------------
// Update-broadcast decision helper (Session 29 ledger closure)
// ---------------------------------------------------------------------------

/// Pure decision mirroring the guard chain in C++ `Game::updatePodium`:
///
/// ```cpp
/// void Game::updatePodium(Item* item) {
///     if (!item->getPodium()) return;
///     Tile* tile = item->getTile();
///     if (!tile) return;
///     // broadcast SendUpdateTileItem to all spectators of item->position
/// }
/// ```
///
/// The cross-crate caller (game crate) holds the item, the tile, and
/// the `Map::get_spectators` query; this helper just answers "should we
/// even start broadcasting?". Returns `true` only when both guards
/// pass — the spectator query + per-Player `sendUpdateTileItem` dispatch
/// is then the caller's responsibility.
pub fn should_broadcast_podium_update(is_podium: bool, tile_present: bool) -> bool {
    is_podium && tile_present
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_podium_flags_variants() {
        assert_eq!(PodiumFlags::ShowPlatform as u8, 0);
        assert_eq!(PodiumFlags::ShowOutfit as u8, 1);
        assert_eq!(PodiumFlags::ShowMount as u8, 2);
    }

    #[test]
    fn test_podium_new_direction_is_south() {
        let p = Podium::new(100);
        assert_eq!(p.get_direction(), Direction::South);
    }

    #[test]
    fn test_podium_new_has_show_platform_flag() {
        let p = Podium::new(100);
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
    }

    #[test]
    fn test_podium_new_show_outfit_false() {
        let p = Podium::new(100);
        assert!(!p.has_flag(PodiumFlags::ShowOutfit));
    }

    #[test]
    fn test_podium_new_show_mount_false() {
        let p = Podium::new(100);
        assert!(!p.has_flag(PodiumFlags::ShowMount));
    }

    #[test]
    fn test_podium_get_outfit_default() {
        let p = Podium::new(100);
        assert_eq!(*p.get_outfit(), Outfit::default());
    }

    #[test]
    fn test_podium_set_outfit() {
        let mut p = Podium::new(100);
        let o = Outfit {
            look_type: 128,
            look_head: 1,
            ..Outfit::default()
        };
        p.set_outfit(o);
        assert_eq!(p.get_outfit().look_type, 128);
        assert_eq!(p.get_outfit().look_head, 1);
    }

    #[test]
    fn test_podium_set_direction() {
        let mut p = Podium::new(100);
        p.set_direction(Direction::North);
        assert_eq!(p.get_direction(), Direction::North);
    }

    #[test]
    fn test_podium_set_flag_true() {
        let mut p = Podium::new(100);
        p.set_flag(PodiumFlags::ShowOutfit, true);
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
    }

    #[test]
    fn test_podium_set_flag_false() {
        let mut p = Podium::new(100);
        // ShowPlatform starts true
        p.set_flag(PodiumFlags::ShowPlatform, false);
        assert!(!p.has_flag(PodiumFlags::ShowPlatform));
    }

    #[test]
    fn test_podium_set_flag_mount_true() {
        let mut p = Podium::new(100);
        p.set_flag(PodiumFlags::ShowMount, true);
        assert!(p.has_flag(PodiumFlags::ShowMount));
    }

    #[test]
    fn test_podium_serialize_flags_default() {
        let p = Podium::new(100);
        // bit 0 is set → 1
        assert_eq!(p.serialize_flags(), 1);
    }

    #[test]
    fn test_podium_serialize_flags_all_set() {
        let mut p = Podium::new(100);
        p.set_flag(PodiumFlags::ShowOutfit, true);
        p.set_flag(PodiumFlags::ShowMount, true);
        assert_eq!(p.serialize_flags(), 0b111);
    }

    #[test]
    fn test_podium_serialize_direction_south() {
        let p = Podium::new(100);
        // Direction::South == 2 in C++ wire format
        assert_eq!(p.serialize_direction(), Direction::South as u8);
    }

    #[test]
    fn test_podium_serialize_direction_round_trip() {
        let mut p = Podium::new(100);
        p.set_direction(Direction::North);
        let raw = p.serialize_direction();
        // Raw 0 = North
        assert_eq!(raw, Direction::North as u8);
    }

    #[test]
    fn test_podium_set_flags_raw() {
        let mut p = Podium::new(100);
        p.set_flags(0b110); // ShowOutfit + ShowMount, not ShowPlatform
        assert!(!p.has_flag(PodiumFlags::ShowPlatform));
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
        assert!(p.has_flag(PodiumFlags::ShowMount));
    }

    // -----------------------------------------------------------------------
    // New tests — gaps identified during audit
    // -----------------------------------------------------------------------

    /// Constructor stores item_type_id correctly.
    #[test]
    fn test_podium_new_stores_item_type_id() {
        let p = Podium::new(42);
        assert_eq!(p.item_type_id, 42);
        let p2 = Podium::new(0xFFFF);
        assert_eq!(p2.item_type_id, 0xFFFF);
    }

    /// `set_flags` with a raw value of 0 clears all flags.
    #[test]
    fn test_podium_set_flags_zero_clears_all() {
        let mut p = Podium::new(100);
        // Default has ShowPlatform set
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        p.set_flags(0);
        assert!(!p.has_flag(PodiumFlags::ShowPlatform));
        assert!(!p.has_flag(PodiumFlags::ShowOutfit));
        assert!(!p.has_flag(PodiumFlags::ShowMount));
    }

    /// `set_flags` with 0xFF must only keep the lower 3 bits (mirrors
    /// `std::bitset<3>` which silently discards higher bits).
    #[test]
    fn test_podium_set_flags_masks_to_3_bits() {
        let mut p = Podium::new(100);
        p.set_flags(0xFF); // all 8 bits set
                           // All three valid flags must be set
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
        assert!(p.has_flag(PodiumFlags::ShowMount));
        // The raw serialized value must only contain the 3 valid bits
        assert_eq!(p.serialize_flags(), 0b111);
    }

    /// `set_flags(0b101)` sets ShowPlatform+ShowMount but not ShowOutfit.
    #[test]
    fn test_podium_set_flags_platform_and_mount_only() {
        let mut p = Podium::new(100);
        p.set_flags(0b101);
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        assert!(!p.has_flag(PodiumFlags::ShowOutfit));
        assert!(p.has_flag(PodiumFlags::ShowMount));
        assert_eq!(p.serialize_flags(), 0b101);
    }

    /// `set_flags(0b011)` sets ShowPlatform+ShowOutfit but not ShowMount.
    #[test]
    fn test_podium_set_flags_platform_and_outfit_only() {
        let mut p = Podium::new(100);
        p.set_flags(0b011);
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
        assert!(!p.has_flag(PodiumFlags::ShowMount));
        assert_eq!(p.serialize_flags(), 0b011);
    }

    /// Setting a flag that is already set is idempotent.
    #[test]
    fn test_podium_set_flag_true_idempotent() {
        let mut p = Podium::new(100);
        // ShowPlatform starts true
        p.set_flag(PodiumFlags::ShowPlatform, true);
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        // serialized flags still 1
        assert_eq!(p.serialize_flags(), 1);
    }

    /// Clearing a flag that is already clear is idempotent.
    #[test]
    fn test_podium_set_flag_false_idempotent() {
        let mut p = Podium::new(100);
        // ShowOutfit starts false
        p.set_flag(PodiumFlags::ShowOutfit, false);
        assert!(!p.has_flag(PodiumFlags::ShowOutfit));
        // ShowPlatform still set, serialized flags = 1
        assert_eq!(p.serialize_flags(), 1);
    }

    /// Setting ShowOutfit and ShowMount independently does not interfere with
    /// each other or with ShowPlatform.
    #[test]
    fn test_podium_set_flag_outfit_then_mount_independent() {
        let mut p = Podium::new(100);
        p.set_flag(PodiumFlags::ShowOutfit, true);
        p.set_flag(PodiumFlags::ShowMount, true);
        assert!(p.has_flag(PodiumFlags::ShowPlatform)); // unchanged
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
        assert!(p.has_flag(PodiumFlags::ShowMount));
        assert_eq!(p.serialize_flags(), 0b111);
    }

    /// Clearing ShowMount must not touch ShowPlatform or ShowOutfit.
    #[test]
    fn test_podium_clear_mount_leaves_others_intact() {
        let mut p = Podium::new(100);
        p.set_flag(PodiumFlags::ShowOutfit, true);
        p.set_flag(PodiumFlags::ShowMount, true);
        p.set_flag(PodiumFlags::ShowMount, false);
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
        assert!(!p.has_flag(PodiumFlags::ShowMount));
        assert_eq!(p.serialize_flags(), 0b011);
    }

    /// Full outfit round-trip: all fields set and retrieved correctly.
    #[test]
    fn test_podium_outfit_all_fields_round_trip() {
        use forgottenserver_common::enums::Outfit;
        let mut p = Podium::new(1);
        let o = Outfit {
            look_type: 128,
            look_type_ex: 0,
            look_head: 10,
            look_body: 20,
            look_legs: 30,
            look_feet: 40,
            look_addons: 3,
            look_mount: 200,
            look_mount_head: 50,
            look_mount_body: 60,
            look_mount_legs: 70,
            look_mount_feet: 80,
        };
        p.set_outfit(o);
        let got = p.get_outfit();
        assert_eq!(got.look_type, 128);
        assert_eq!(got.look_head, 10);
        assert_eq!(got.look_body, 20);
        assert_eq!(got.look_legs, 30);
        assert_eq!(got.look_feet, 40);
        assert_eq!(got.look_addons, 3);
        assert_eq!(got.look_mount, 200);
        assert_eq!(got.look_mount_head, 50);
        assert_eq!(got.look_mount_body, 60);
        assert_eq!(got.look_mount_legs, 70);
        assert_eq!(got.look_mount_feet, 80);
    }

    /// Replacing an outfit overwrites all previous values.
    #[test]
    fn test_podium_set_outfit_replaces_previous() {
        use forgottenserver_common::enums::Outfit;
        let mut p = Podium::new(1);
        let first = Outfit {
            look_type: 10,
            ..Outfit::default()
        };
        let second = Outfit {
            look_type: 999,
            look_head: 77,
            ..Outfit::default()
        };
        p.set_outfit(first);
        p.set_outfit(second);
        assert_eq!(p.get_outfit().look_type, 999);
        assert_eq!(p.get_outfit().look_head, 77);
    }

    /// serialize_direction returns correct wire values for all four cardinal
    /// directions (mirrors C++ Direction discriminants: North=0, East=1,
    /// South=2, West=3).
    #[test]
    fn test_podium_serialize_direction_all_cardinals() {
        let directions = [
            (Direction::North, 0u8),
            (Direction::East, 1u8),
            (Direction::South, 2u8),
            (Direction::West, 3u8),
        ];
        for (dir, expected) in directions {
            let mut p = Podium::new(1);
            p.set_direction(dir);
            assert_eq!(
                p.serialize_direction(),
                expected,
                "Expected {dir:?} → {expected}",
            );
        }
    }

    /// serialize_direction is consistent with get_direction for all diagonals.
    #[test]
    fn test_podium_serialize_direction_diagonals() {
        let directions = [
            Direction::Southwest,
            Direction::Southeast,
            Direction::Northwest,
            Direction::Northeast,
        ];
        for dir in directions {
            let mut p = Podium::new(1);
            p.set_direction(dir);
            assert_eq!(p.serialize_direction(), dir as u8);
            assert_eq!(p.get_direction(), dir);
        }
    }

    /// Clone produces an independent copy with identical state.
    #[test]
    fn test_podium_clone_is_independent() {
        let mut original = Podium::new(100);
        original.set_flag(PodiumFlags::ShowOutfit, true);
        original.set_direction(Direction::North);

        let mut cloned = original.clone();
        // Mutate clone — original must not change
        cloned.set_flag(PodiumFlags::ShowMount, true);
        cloned.set_direction(Direction::West);

        assert!(!original.has_flag(PodiumFlags::ShowMount));
        assert_eq!(original.get_direction(), Direction::North);
        // Clone has the new values
        assert!(cloned.has_flag(PodiumFlags::ShowMount));
        assert_eq!(cloned.get_direction(), Direction::West);
    }

    /// Debug formatting does not panic.
    #[test]
    fn test_podium_debug_does_not_panic() {
        let p = Podium::new(7);
        let _ = format!("{:?}", p);
    }

    /// Serialize-deserialize round-trip via set_flags/serialize_flags: the raw
    /// byte written by serialize_flags can be fed back into set_flags and
    /// produces identical flag state.
    #[test]
    fn test_podium_flags_serialize_deserialize_round_trip() {
        let mut original = Podium::new(1);
        original.set_flag(PodiumFlags::ShowOutfit, true);
        original.set_flag(PodiumFlags::ShowMount, false);

        let raw = original.serialize_flags();

        let mut restored = Podium::new(1);
        restored.set_flags(raw);

        assert_eq!(
            restored.has_flag(PodiumFlags::ShowPlatform),
            original.has_flag(PodiumFlags::ShowPlatform)
        );
        assert_eq!(
            restored.has_flag(PodiumFlags::ShowOutfit),
            original.has_flag(PodiumFlags::ShowOutfit)
        );
        assert_eq!(
            restored.has_flag(PodiumFlags::ShowMount),
            original.has_flag(PodiumFlags::ShowMount)
        );
        assert_eq!(restored.serialize_flags(), raw);
    }

    // -----------------------------------------------------------------------
    // serialize_attr / read_attr_podiumoutfit (C++ readAttr / serializeAttr)
    // -----------------------------------------------------------------------

    /// `serialize_attr` produces a 16-byte buffer beginning with the
    /// ATTR_PODIUMOUTFIT tag (40), mirroring podium.cpp `serializeAttr`.
    #[test]
    fn test_podium_serialize_attr_tag_and_length() {
        let p = Podium::new(1);
        let buf = p.serialize_attr();
        assert_eq!(buf.len(), 16);
        assert_eq!(buf[0], Podium::ATTR_PODIUMOUTFIT);
        assert_eq!(Podium::ATTR_PODIUMOUTFIT, 40);
    }

    /// `serialize_attr` lays out flags, direction and every outfit field in
    /// the exact byte order of C++ `Podium::serializeAttr`.
    #[test]
    fn test_podium_serialize_attr_layout() {
        use forgottenserver_common::enums::Outfit;
        let mut p = Podium::new(1);
        p.set_flags(0b101); // ShowPlatform + ShowMount
        p.set_direction(Direction::West); // 3
        p.set_outfit(Outfit {
            look_type: 0x1234,
            look_type_ex: 0xCAFE, // not serialized — should be ignored
            look_head: 0x11,
            look_body: 0x22,
            look_legs: 0x33,
            look_feet: 0x44,
            look_addons: 0x55,
            look_mount: 0xABCD,
            look_mount_head: 0x66,
            look_mount_body: 0x77,
            look_mount_legs: 0x88,
            look_mount_feet: 0x99,
        });
        let buf = p.serialize_attr();
        let expected: [u8; 16] = [
            40,    // tag
            0b101, // flags
            3,     // direction (West)
            0x34, 0x12, // look_type LE
            0x11, 0x22, 0x33, 0x44, 0x55, 0xCD, 0xAB, // look_mount LE
            0x66, 0x77, 0x88, 0x99,
        ];
        assert_eq!(buf, expected.to_vec());
    }

    /// `read_attr_podiumoutfit` rejects short buffers with `false` (mirrors
    /// the C++ `ATTR_READ_ERROR` early return).
    #[test]
    fn test_podium_read_attr_rejects_short_payload() {
        let mut p = Podium::new(1);
        for len in 0..15 {
            let payload = vec![0u8; len];
            assert!(
                !p.read_attr_podiumoutfit(&payload),
                "len={len} should be rejected",
            );
        }
    }

    /// `read_attr_podiumoutfit` accepts exactly 15-byte payloads and
    /// populates every field.
    #[test]
    fn test_podium_read_attr_populates_all_fields() {
        let payload: [u8; 15] = [
            0b011, // flags: ShowPlatform + ShowOutfit
            1,     // direction (East)
            0x34, 0x12, // look_type
            0x10, // look_head
            0x20, // look_body
            0x30, // look_legs
            0x40, // look_feet
            0x03, // look_addons
            0xCD, 0xAB, // look_mount
            0x50, // look_mount_head
            0x60, // look_mount_body
            0x70, // look_mount_legs
            0x80, // look_mount_feet
        ];
        let mut p = Podium::new(1);
        assert!(p.read_attr_podiumoutfit(&payload));
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
        assert!(!p.has_flag(PodiumFlags::ShowMount));
        assert_eq!(p.get_direction(), Direction::East);
        let o = p.get_outfit();
        assert_eq!(o.look_type, 0x1234);
        assert_eq!(o.look_head, 0x10);
        assert_eq!(o.look_body, 0x20);
        assert_eq!(o.look_legs, 0x30);
        assert_eq!(o.look_feet, 0x40);
        assert_eq!(o.look_addons, 0x03);
        assert_eq!(o.look_mount, 0xABCD);
        assert_eq!(o.look_mount_head, 0x50);
        assert_eq!(o.look_mount_body, 0x60);
        assert_eq!(o.look_mount_legs, 0x70);
        assert_eq!(o.look_mount_feet, 0x80);
    }

    /// Direction discriminants 0..=7 each decode to the corresponding
    /// `Direction` value (cardinal + diagonal).
    #[test]
    fn test_podium_read_attr_all_directions() {
        let expected = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
            Direction::Southwest,
            Direction::Southeast,
            Direction::Northwest,
            Direction::Northeast,
        ];
        for (raw, &dir) in expected.iter().enumerate() {
            let mut payload = [0u8; 15];
            payload[1] = raw as u8;
            let mut p = Podium::new(1);
            assert!(p.read_attr_podiumoutfit(&payload));
            assert_eq!(p.get_direction(), dir, "raw={raw}");
        }
    }

    /// Out-of-range direction bytes (>= 8) fall back to South — mirrors the
    /// `static_cast<Direction>(newDirection)` behaviour when feeding C++ an
    /// invalid discriminant.
    #[test]
    fn test_podium_read_attr_unknown_direction_falls_back_to_south() {
        let mut payload = [0u8; 15];
        payload[1] = 99; // out-of-range
        let mut p = Podium::new(1);
        assert!(p.read_attr_podiumoutfit(&payload));
        assert_eq!(p.get_direction(), Direction::South);
    }

    /// `read_attr_podiumoutfit` accepts buffers longer than 15 bytes and
    /// only consumes the first 15 — matching the C++ stream-based reader
    /// which leaves trailing bytes for the caller.
    #[test]
    fn test_podium_read_attr_extra_bytes_ignored() {
        let mut payload = vec![0u8; 32];
        payload[0] = 0b111;
        payload[1] = 2; // South
        let mut p = Podium::new(1);
        assert!(p.read_attr_podiumoutfit(&payload));
        assert!(p.has_flag(PodiumFlags::ShowPlatform));
        assert!(p.has_flag(PodiumFlags::ShowOutfit));
        assert!(p.has_flag(PodiumFlags::ShowMount));
    }

    /// `serialize_attr` → strip tag byte → `read_attr_podiumoutfit` is the
    /// identity for every field except `look_type_ex` (which the C++ wire
    /// format does not include).
    #[test]
    fn test_podium_attr_serialize_read_round_trip() {
        use forgottenserver_common::enums::Outfit;
        let mut original = Podium::new(7);
        original.set_flags(0b111);
        original.set_direction(Direction::Northeast);
        original.set_outfit(Outfit {
            look_type: 4096,
            look_type_ex: 0, // not in wire format
            look_head: 11,
            look_body: 22,
            look_legs: 33,
            look_feet: 44,
            look_addons: 1,
            look_mount: 600,
            look_mount_head: 55,
            look_mount_body: 66,
            look_mount_legs: 77,
            look_mount_feet: 88,
        });
        let buf = original.serialize_attr();
        assert_eq!(buf[0], Podium::ATTR_PODIUMOUTFIT);
        let mut restored = Podium::new(7);
        assert!(restored.read_attr_podiumoutfit(&buf[1..]));
        assert_eq!(restored.serialize_flags(), original.serialize_flags());
        assert_eq!(restored.get_direction(), original.get_direction());
        assert_eq!(restored.get_outfit(), original.get_outfit());
    }

    /// The `PODIUMOUTFIT_PAYLOAD_LEN` constant equals the C++ `15` literal
    /// from `Podium::readAttr`'s `propStream.size() < 15` guard.
    #[test]
    fn test_podium_payload_len_constant_matches_cpp() {
        assert_eq!(Podium::PODIUMOUTFIT_PAYLOAD_LEN, 15);
    }

    /// Direction serialize/deserialize round-trip: `serialize_direction`
    /// returns the `u8` discriminant, and `set_direction(dir)` round-trips
    /// back through `get_direction`.
    #[test]
    fn test_podium_direction_serialize_deserialize_round_trip() {
        let all_dirs = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
            Direction::Southwest,
            Direction::Southeast,
            Direction::Northwest,
            Direction::Northeast,
        ];
        for dir in all_dirs {
            let mut p = Podium::new(1);
            p.set_direction(dir);
            let raw = p.serialize_direction();
            assert_eq!(raw, dir as u8, "{dir:?} serialize mismatch");
            assert_eq!(p.get_direction(), dir, "{dir:?} get_direction mismatch");
        }
    }

    // ── Update-broadcast decision helper (Session 29) ───────────────────

    /// Non-podium items short-circuit — the C++ `!item->getPodium()` guard.
    #[test]
    fn should_broadcast_podium_update_returns_false_when_not_podium() {
        assert!(!should_broadcast_podium_update(false, true));
        assert!(!should_broadcast_podium_update(false, false));
    }

    /// Tile-less items short-circuit — the C++ `!tile` guard.
    #[test]
    fn should_broadcast_podium_update_returns_false_when_no_tile() {
        assert!(!should_broadcast_podium_update(true, false));
    }

    /// Both guards pass → broadcast.
    #[test]
    fn should_broadcast_podium_update_returns_true_only_when_both_guards_pass() {
        assert!(should_broadcast_podium_update(true, true));
    }
}
