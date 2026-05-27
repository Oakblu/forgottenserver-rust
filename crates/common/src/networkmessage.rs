//! Migrated from forgottenserver/src/networkmessage.h and networkmessage.cpp
//!
//! `NetworkMessage` is a byte buffer for reading/writing Tibia protocol packets.
//!
//! ## Buffer layout
//! ```text
//! [0..8)   — protocol headers (2 unencrypted len + 4 checksum + 2 encrypted len)
//! [8..)    — payload (read/write cursor starts here)
//! ```
//!
//! `info.length` counts bytes written into the payload region (excludes header).
//! `info.position` is an absolute index into `buffer`.

#![allow(dead_code)]

use crate::constants::{FLUID_MAP, NETWORKMESSAGE_MAXSIZE};
use crate::position::Position;

// ---------------------------------------------------------------------------
// Item serialization payload (mirrors C++ `NetworkMessage::addItem`)
// ---------------------------------------------------------------------------

/// `WEAPON_QUIVER` enum value from `const.h` (`WeaponType_t`).  Mirrored
/// here as a plain `u8` so the `common` crate stays free of cross-crate
/// item/enum dependencies.
pub const WEAPON_TYPE_QUIVER: u8 = 8;

/// Plain-data view of an `ItemType` containing only the fields that affect
/// the serialized bytes of `addItem` / `addItemId`.
///
/// Mirrors the subset of `ItemType` used by `NetworkMessage::addItem` in
/// `forgottenserver/src/networkmessage.cpp` (lines 90–193).  Callers in the
/// `network` crate are responsible for extracting these values from the
/// real `ItemType` registry and passing them in.
///
/// All fields are primitives so `common` does not need to depend on
/// `items` or `entity`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemTypeMeta {
    /// `ItemType::clientId` — first u16 written for every item.
    pub client_id: u16,
    /// `ItemType::stackable`
    pub stackable: bool,
    /// `ItemType::isSplash()` — `group == ITEM_GROUP_SPLASH`
    pub is_splash: bool,
    /// `ItemType::isFluidContainer()` — `group == ITEM_GROUP_FLUID`
    pub is_fluid_container: bool,
    /// `ItemType::isContainer()` — `group == ITEM_GROUP_CONTAINER`
    pub is_container: bool,
    /// `ItemType::classification` (0 for unclassified items)
    pub classification: u8,
    /// `ItemType::showClientCharges`
    pub show_client_charges: bool,
    /// `ItemType::showClientDuration`
    pub show_client_duration: bool,
    /// `ItemType::isPodium()` — `type == ITEM_TYPE_PODIUM`
    pub is_podium: bool,
    /// `ItemType::weaponType` (used only to detect `WEAPON_QUIVER`)
    pub weapon_type: u8,
    /// `ItemType::charges` — written when `showClientCharges` is set
    /// and the caller did not supply a per-instance charge count.
    pub charges: u32,
    /// `ItemType::decayTimeMin` — written when `showClientDuration` is set
    /// for the "fresh item by id" path.
    pub decay_time_min: u32,
}

/// Plain-data view of the podium block written when `ItemType::isPodium()`.
///
/// Mirrors `NetworkMessage::addItem(const Item*)` in
/// `forgottenserver/src/networkmessage.cpp` (lines 156–190).  All fields are
/// primitives — the caller resolves the `Podium` and `Outfit_t` structures
/// into this snapshot before passing it in.
#[derive(Debug, Clone, Copy, Default)]
pub struct PodiumMeta {
    /// `PODIUM_SHOW_OUTFIT` flag
    pub show_outfit: bool,
    /// `PODIUM_SHOW_MOUNT` flag
    pub show_mount: bool,
    /// `PODIUM_SHOW_PLATFORM` flag
    pub show_platform: bool,
    /// `outfit.lookType`
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
    /// `outfit.lookMount`
    pub look_mount: u16,
    pub look_mount_head: u8,
    pub look_mount_body: u8,
    pub look_mount_legs: u8,
    pub look_mount_feet: u8,
    /// `podium->getDirection()`
    pub direction: u8,
}

/// The 8-byte header block that precedes the usable payload.
///
/// Layout:
/// * `[0..2]` – unencrypted message size (u16 LE)
/// * `[2..6]` – checksum (u32 LE, Adler-32 or sequence-id)
/// * `[6..8]` – encrypted message size (u16 LE)
pub const INITIAL_BUFFER_POSITION: u16 = 8;

pub const HEADER_LENGTH: usize = 2;
pub const CHECKSUM_LENGTH: usize = 4;
pub const XTEA_MULTIPLE: usize = 8;

/// Maximum number of bytes that can form the message body (payload after headers).
pub const MAX_BODY_LENGTH: usize =
    NETWORKMESSAGE_MAXSIZE as usize - HEADER_LENGTH - CHECKSUM_LENGTH - XTEA_MULTIPLE;

/// Maximum protocol body length (10 bytes reserved for protocol overhead).
pub const MAX_PROTOCOL_BODY_LENGTH: usize = MAX_BODY_LENGTH - 10;

/// Maximum string length enforced by `add_string` / `add_bytes`.
const MAX_STRING_LENGTH: usize = 8192;

// ---------------------------------------------------------------------------
// NetworkMessageInfo
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub(crate) struct NetworkMessageInfo {
    /// Number of bytes written into the payload (does NOT include the 8-byte header block).
    length: u16,
    /// Absolute position inside `buffer` for the next read or write.
    position: u16,
    /// Set to `true` when a read or write would exceed the buffer bounds.
    overrun: bool,
}

impl Default for NetworkMessageInfo {
    fn default() -> Self {
        Self {
            length: 0,
            position: INITIAL_BUFFER_POSITION,
            overrun: false,
        }
    }
}

// ---------------------------------------------------------------------------
// NetworkMessage
// ---------------------------------------------------------------------------

/// A Tibia protocol packet buffer.
///
/// Provides separate read and write cursors that both start at byte 8 (after
/// the 8-byte header block).  Lengths track only the payload bytes — the
/// header region is managed by higher-level protocol code.
#[derive(Clone)]
pub struct NetworkMessage {
    pub(crate) info: NetworkMessageInfo,
    pub(crate) buffer: Box<[u8; NETWORKMESSAGE_MAXSIZE as usize]>,
}

impl Default for NetworkMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMessage {
    pub fn new() -> Self {
        Self {
            info: NetworkMessageInfo::default(),
            buffer: Box::new([0u8; NETWORKMESSAGE_MAXSIZE as usize]),
        }
    }

    /// Resets the message: clears the length and resets the cursor to the
    /// start of the payload region.  Does NOT zero the buffer.
    pub fn reset(&mut self) {
        self.info = NetworkMessageInfo::default();
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Returns `true` if `n` more bytes can be written starting at the current position.
    fn can_add(&self, n: usize) -> bool {
        (n + self.info.position as usize) < MAX_BODY_LENGTH
    }

    /// Returns `true` if `n` more bytes can be read starting at the current position.
    ///
    /// Mirrors the C++ `canRead` check:
    /// `(position + size) > (length + 8)  ||  size >= (MAXSIZE - position)`
    fn can_read(&mut self, n: i32) -> bool {
        let pos = self.info.position as i32;
        let len = self.info.length as i32;
        let max = NETWORKMESSAGE_MAXSIZE;

        if (pos + n) > (len + 8) || n >= (max - pos) {
            self.info.overrun = true;
            return false;
        }
        true
    }

    // -----------------------------------------------------------------------
    // Read operations
    // -----------------------------------------------------------------------

    /// Reads one byte; returns `0` on overrun.
    pub fn get_u8(&mut self) -> u8 {
        if !self.can_read(1) {
            return 0;
        }
        let v = self.buffer[self.info.position as usize];
        self.info.position += 1;
        v
    }

    /// Reads the byte just before the current position (backs up by one).
    pub fn get_previous_u8(&mut self) -> u8 {
        self.info.position -= 1;
        self.buffer[self.info.position as usize]
    }

    /// Reads a little-endian `u16`; returns `0` on overrun.
    pub fn get_u16(&mut self) -> u16 {
        if !self.can_read(2) {
            return 0;
        }
        let pos = self.info.position as usize;
        let v = u16::from_le_bytes([self.buffer[pos], self.buffer[pos + 1]]);
        self.info.position += 2;
        v
    }

    /// Reads a little-endian `u32`; returns `0` on overrun.
    pub fn get_u32(&mut self) -> u32 {
        if !self.can_read(4) {
            return 0;
        }
        let pos = self.info.position as usize;
        let v = u32::from_le_bytes([
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
        ]);
        self.info.position += 4;
        v
    }

    /// Reads a little-endian `u64`; returns `0` on overrun.
    pub fn get_u64(&mut self) -> u64 {
        if !self.can_read(8) {
            return 0;
        }
        let pos = self.info.position as usize;
        let v = u64::from_le_bytes([
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
            self.buffer[pos + 4],
            self.buffer[pos + 5],
            self.buffer[pos + 6],
            self.buffer[pos + 7],
        ]);
        self.info.position += 8;
        v
    }

    /// Reads a length-prefixed UTF-8 string.
    ///
    /// First reads a `u16` length, then reads that many bytes and interprets
    /// them as UTF-8 (replacing invalid sequences with U+FFFD).  Returns an
    /// empty string on overrun.
    ///
    /// If `string_len` is `0` (default) the length is read from the stream;
    /// otherwise the provided length is used directly.
    pub fn get_string(&mut self, string_len: u16) -> String {
        let len = if string_len == 0 {
            self.get_u16()
        } else {
            string_len
        };

        if !self.can_read(len as i32) {
            return String::new();
        }

        let pos = self.info.position as usize;
        let bytes = &self.buffer[pos..pos + len as usize];
        self.info.position += len;
        String::from_utf8_lossy(bytes).into_owned()
    }

    /// Reads `n` raw bytes; returns an empty `Vec` on overrun.
    pub fn get_bytes(&mut self, n: usize) -> Vec<u8> {
        if !self.can_read(n as i32) {
            return Vec::new();
        }
        let pos = self.info.position as usize;
        let v = self.buffer[pos..pos + n].to_vec();
        self.info.position += n as u16;
        v
    }

    /// Advances the read cursor by `count` bytes without reading them.
    pub fn skip_bytes(&mut self, count: i16) {
        self.info.position = (self.info.position as i16 + count) as u16;
    }

    // -----------------------------------------------------------------------
    // Write operations
    // -----------------------------------------------------------------------

    /// Writes one byte; silently ignores the write on overflow.
    pub fn add_u8(&mut self, value: u8) {
        if !self.can_add(1) {
            return;
        }
        self.buffer[self.info.position as usize] = value;
        self.info.position += 1;
        self.info.length += 1;
    }

    /// Writes a little-endian `u16`.
    pub fn add_u16(&mut self, value: u16) {
        if !self.can_add(2) {
            return;
        }
        let pos = self.info.position as usize;
        let bytes = value.to_le_bytes();
        self.buffer[pos] = bytes[0];
        self.buffer[pos + 1] = bytes[1];
        self.info.position += 2;
        self.info.length += 2;
    }

    /// Writes a little-endian `u32`.
    pub fn add_u32(&mut self, value: u32) {
        if !self.can_add(4) {
            return;
        }
        let pos = self.info.position as usize;
        let bytes = value.to_le_bytes();
        self.buffer[pos..pos + 4].copy_from_slice(&bytes);
        self.info.position += 4;
        self.info.length += 4;
    }

    /// Writes a little-endian `u64`.
    pub fn add_u64(&mut self, value: u64) {
        if !self.can_add(8) {
            return;
        }
        let pos = self.info.position as usize;
        let bytes = value.to_le_bytes();
        self.buffer[pos..pos + 8].copy_from_slice(&bytes);
        self.info.position += 8;
        self.info.length += 8;
    }

    /// Writes a length-prefixed UTF-8 string (u16 length + bytes).
    ///
    /// Silently drops the write if the string exceeds `MAX_STRING_LENGTH`
    /// bytes or there is insufficient buffer space.
    pub fn add_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if !self.can_add(len + 2) || len > MAX_STRING_LENGTH {
            return;
        }
        self.add_u16(len as u16);
        let pos = self.info.position as usize;
        self.buffer[pos..pos + len].copy_from_slice(bytes);
        self.info.position += len as u16;
        self.info.length += len as u16;
    }

    /// Writes raw bytes; silently drops on overflow or if `data` exceeds `MAX_STRING_LENGTH`.
    pub fn add_bytes(&mut self, data: &[u8]) {
        let len = data.len();
        if !self.can_add(len) || len > MAX_STRING_LENGTH {
            return;
        }
        let pos = self.info.position as usize;
        self.buffer[pos..pos + len].copy_from_slice(data);
        self.info.position += len as u16;
        self.info.length += len as u16;
    }

    /// Writes `n` padding bytes (value `0x33`, matching C++ behaviour).
    pub fn add_padding_bytes(&mut self, n: usize) {
        if !self.can_add(n) {
            return;
        }
        let pos = self.info.position as usize;
        for i in 0..n {
            self.buffer[pos + i] = 0x33;
        }
        // Note: position is NOT advanced (matches C++ addPaddingBytes which
        // does not update position, only length).
        self.info.length += n as u16;
    }

    // -----------------------------------------------------------------------
    // Complex types (Position / double)
    // -----------------------------------------------------------------------

    /// Writes a 5-byte `Position`: `x` (u16 LE) + `y` (u16 LE) + `z` (u8).
    ///
    /// Mirrors `NetworkMessage::addPosition` in C++.
    pub fn add_position(&mut self, pos: Position) {
        self.add_u16(pos.x);
        self.add_u16(pos.y);
        self.add_u8(pos.z);
    }

    /// Reads a 5-byte `Position`: `x` (u16 LE) + `y` (u16 LE) + `z` (u8).
    ///
    /// Mirrors `NetworkMessage::getPosition` in C++.
    pub fn get_position(&mut self) -> Position {
        let x = self.get_u16();
        let y = self.get_u16();
        let z = self.get_u8();
        Position { x, y, z }
    }

    /// Writes a `double` value with a fixed-point encoding.
    ///
    /// Layout (matches C++ `NetworkMessage::addDouble`):
    /// * 1 byte — precision (number of decimal digits to preserve, default 2)
    /// * 4 bytes — scaled value as `u32` LE: `(value * 10^precision) + i32::MAX`
    pub fn add_double(&mut self, value: f64, precision: u8) {
        self.add_u8(precision);
        let scale = 10f64.powi(precision as i32);
        let scaled = value * scale + i32::MAX as f64;
        self.add_u32(scaled as u32);
    }

    // -----------------------------------------------------------------------
    // Item serialization (mirrors C++ NetworkMessage::addItem family)
    // -----------------------------------------------------------------------

    /// Writes the item bytes for the "fresh item by id" path.
    ///
    /// Mirrors C++ `NetworkMessage::addItem(uint16_t id, uint8_t count)` in
    /// `forgottenserver/src/networkmessage.cpp` lines 90–119.
    ///
    /// Byte layout (in order):
    /// * `client_id` (u16 LE) — always written
    /// * sub-type byte, exactly one of (mutually exclusive branches):
    ///   * `count` (u8) — when `meta.stackable`
    ///   * `FLUID_MAP[count & 7]` (u8) — when `meta.is_splash || meta.is_fluid_container`
    ///   * two `0x00` bytes — when `meta.is_container` (loot icon + quiver byte)
    ///   * `0x00` (u8) — when `meta.classification > 0`
    ///   * `charges` (u32 LE) + `0x00` (u8) — when `meta.show_client_charges`
    ///   * `decay_time_min` (u32 LE) + `0x00` (u8) — when `meta.show_client_duration`
    /// * podium block — appended when `meta.is_podium`:
    ///   * `0x0000` (u16 look type) + `0x0000` (u16 look mount) +
    ///     `0x02` (direction) + `0x01` (visible)
    pub fn add_item_payload(&mut self, count: u8, meta: ItemTypeMeta) {
        self.add_u16(meta.client_id);

        if meta.stackable {
            self.add_u8(count);
        } else if meta.is_splash || meta.is_fluid_container {
            self.add_u8(FLUID_MAP[(count & 7) as usize]);
        } else if meta.is_container {
            self.add_u8(0x00); // assigned loot container icon
            self.add_u8(0x00); // quiver ammo count
        } else if meta.classification > 0 {
            self.add_u8(0x00); // item tier (0-10)
        } else if meta.show_client_charges {
            self.add_u32(meta.charges);
            self.add_u8(0x00); // boolean (is brand new)
        } else if meta.show_client_duration {
            self.add_u32(meta.decay_time_min);
            self.add_u8(0x00); // boolean (is brand new)
        }

        if meta.is_podium {
            self.add_u16(0); // looktype
            self.add_u16(0); // lookmount
            self.add_u8(2); // direction
            self.add_u8(0x01); // is visible (bool)
        }
    }

    /// Writes the item bytes for the per-instance path.
    ///
    /// Mirrors C++ `NetworkMessage::addItem(const Item* item)` in
    /// `forgottenserver/src/networkmessage.cpp` lines 121–191.
    ///
    /// * `count` — `item->getItemCount()` (clamped to `0xFF` by the caller)
    ///   when stackable; otherwise `item->getFluidType()` for splash/fluid
    ///   items; ignored otherwise.
    /// * `charges` — `item->getCharges()` (written when `show_client_charges`)
    /// * `duration_seconds` — `item->getDuration() / 1000` (written when
    ///   `show_client_duration`).  Note the C++ source divides milliseconds
    ///   by 1000 at the call site; this method writes the value as-is.
    /// * `ammo_count` — when set and `meta.weapon_type == WEAPON_TYPE_QUIVER`,
    ///   writes `0x01` + `u32` ammo count for the quiver branch.
    /// * `podium` — when set, writes the full podium block in place of the
    ///   default one.
    pub fn add_item_instance(
        &mut self,
        count: u8,
        charges: u32,
        duration_seconds: u32,
        ammo_count: Option<u32>,
        podium: Option<PodiumMeta>,
        meta: ItemTypeMeta,
    ) {
        self.add_u16(meta.client_id);

        if meta.stackable {
            self.add_u8(count);
        } else if meta.is_splash || meta.is_fluid_container {
            self.add_u8(FLUID_MAP[(count & 7) as usize]);
        } else if meta.classification > 0 {
            self.add_u8(0x00); // item tier (0-10)
        }

        if meta.show_client_charges {
            self.add_u32(charges);
            self.add_u8(0); // boolean (is brand new)
        } else if meta.show_client_duration {
            self.add_u32(duration_seconds);
            self.add_u8(0); // boolean (is brand new)
        }

        if meta.is_container {
            self.add_u8(0x00); // assigned loot container icon
                               // quiver ammo count
            if meta.weapon_type == WEAPON_TYPE_QUIVER {
                if let Some(ammo) = ammo_count {
                    self.add_u8(0x01);
                    self.add_u32(ammo);
                } else {
                    self.add_u8(0x00);
                }
            } else {
                self.add_u8(0x00);
            }
        }

        // display outfit on the podium
        if meta.is_podium {
            let p = podium.unwrap_or_default();

            // add outfit
            if p.show_outfit {
                self.add_u16(p.look_type);
                if p.look_type != 0 {
                    self.add_u8(p.look_head);
                    self.add_u8(p.look_body);
                    self.add_u8(p.look_legs);
                    self.add_u8(p.look_feet);
                    self.add_u8(p.look_addons);
                }
            } else {
                self.add_u16(0);
            }

            // add mount
            if p.show_mount {
                self.add_u16(p.look_mount);
                if p.look_mount != 0 {
                    self.add_u8(p.look_mount_head);
                    self.add_u8(p.look_mount_body);
                    self.add_u8(p.look_mount_legs);
                    self.add_u8(p.look_mount_feet);
                }
            } else {
                self.add_u16(0);
            }

            self.add_u8(p.direction);
            self.add_u8(if p.show_platform { 0x01 } else { 0x00 });
        }
    }

    /// Writes only the client-id (`u16` LE) for an item.
    ///
    /// Mirrors C++ `NetworkMessage::addItemId(uint16_t)` in
    /// `forgottenserver/src/networkmessage.cpp` line 193.  Used by market /
    /// merchant packets that send a flat list of client-ids without any of
    /// the sub-type / podium / quiver bytes from `addItem`.
    pub fn add_item_id(&mut self, client_id: u16) {
        self.add_u16(client_id);
    }

    // -----------------------------------------------------------------------
    // Buffer access
    // -----------------------------------------------------------------------

    /// Returns the full internal buffer slice.
    pub fn get_buffer(&self) -> &[u8] {
        self.buffer.as_ref()
    }

    /// Returns a mutable pointer to the full internal buffer.
    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut()
    }

    /// Returns the slice starting at the current read/write position.
    pub fn get_remaining_buffer(&self) -> &[u8] {
        &self.buffer[self.info.position as usize..]
    }

    /// Returns the `length` field — bytes written into the payload region.
    pub fn get_message_length(&self) -> u16 {
        self.info.length
    }

    /// Sets the length field directly (used by protocol layers).
    pub fn set_message_length(&mut self, new_length: u16) {
        self.info.length = new_length;
    }

    /// Returns `true` when no bytes have been written.
    pub fn is_empty(&self) -> bool {
        self.info.length == 0
    }

    /// Returns the current absolute buffer position (read/write cursor).
    pub fn get_buffer_position(&self) -> u16 {
        self.info.position
    }

    /// Sets the buffer position as an offset from `INITIAL_BUFFER_POSITION`.
    ///
    /// Returns `false` if `pos` would exceed the buffer.
    pub fn set_buffer_position(&mut self, pos: u16) -> bool {
        if (pos as usize) < NETWORKMESSAGE_MAXSIZE as usize - INITIAL_BUFFER_POSITION as usize {
            self.info.position = pos + INITIAL_BUFFER_POSITION;
            return true;
        }
        false
    }

    /// Returns the remaining bytes available to read (`length - (position - 8)`).
    pub fn get_remaining_buffer_length(&self) -> u16 {
        self.info
            .length
            .saturating_sub(self.info.position - INITIAL_BUFFER_POSITION)
    }

    /// Reads the 2-byte unencrypted length header at position 0.
    pub fn get_length_header(&self) -> u16 {
        u16::from_le_bytes([self.buffer[0], self.buffer[1]])
    }

    /// Returns `true` if any read overran the buffer.
    pub fn is_overrun(&self) -> bool {
        self.info.overrun
    }

    /// Returns the header position constant.
    pub fn get_header_position(&self) -> usize {
        INITIAL_BUFFER_POSITION as usize
    }

    /// Returns a mutable reference to the body buffer, resetting position to 2.
    pub fn get_body_buffer(&mut self) -> &mut [u8] {
        self.info.position = 2;
        &mut self.buffer[HEADER_LENGTH..]
    }

    /// Returns a string-sized slice description: the body buffer from the
    /// current position through to the end of written data.  Used by the
    /// network crate for introspection.
    pub fn get_string_sized(&self) -> &[u8] {
        let start = self.info.position as usize;
        let end = (INITIAL_BUFFER_POSITION as usize) + self.info.length as usize;
        if start <= end && end <= self.buffer.len() {
            &self.buffer[start..end]
        } else {
            &[]
        }
    }
}

impl std::fmt::Debug for NetworkMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkMessage")
            .field("length", &self.info.length)
            .field("position", &self.info.position)
            .field("overrun", &self.info.overrun)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn msg() -> NetworkMessage {
        NetworkMessage::new()
    }

    // --- Initial state ---

    #[test]
    fn test_new_message_is_empty() {
        let m = msg();
        assert_eq!(m.get_message_length(), 0);
        assert!(m.is_empty());
        assert!(!m.is_overrun());
    }

    #[test]
    fn test_initial_position_is_header_offset() {
        let m = msg();
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
    }

    // --- reset ---

    #[test]
    fn test_reset_clears_length_and_position() {
        let mut m = msg();
        m.add_u32(0xDEAD_BEEF);
        assert_ne!(m.get_message_length(), 0);
        m.reset();
        assert_eq!(m.get_message_length(), 0);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
        assert!(!m.is_overrun());
    }

    // --- add_u8 / get_u8 round-trip ---

    #[test]
    fn test_add_get_u8_round_trip() {
        let mut m = msg();
        m.add_u8(0xAB);
        assert_eq!(m.get_message_length(), 1);
        // Reset position to re-read
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_u8(), 0xAB);
    }

    #[test]
    fn test_add_multiple_u8() {
        let mut m = msg();
        m.add_u8(1);
        m.add_u8(2);
        m.add_u8(3);
        assert_eq!(m.get_message_length(), 3);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_u8(), 1);
        assert_eq!(m.get_u8(), 2);
        assert_eq!(m.get_u8(), 3);
    }

    // --- add_u16 / get_u16 ---

    #[test]
    fn test_add_get_u16_round_trip() {
        let mut m = msg();
        m.add_u16(0x1234);
        assert_eq!(m.get_message_length(), 2);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_u16(), 0x1234);
    }

    #[test]
    fn test_u16_little_endian_layout() {
        let mut m = msg();
        m.add_u16(0x0102);
        // byte at position 8 is low byte, 9 is high
        assert_eq!(m.buffer[8], 0x02);
        assert_eq!(m.buffer[9], 0x01);
    }

    #[test]
    fn test_u16_max_value() {
        let mut m = msg();
        m.add_u16(u16::MAX);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_u16(), u16::MAX);
    }

    // --- add_u32 / get_u32 ---

    #[test]
    fn test_add_get_u32_round_trip() {
        let mut m = msg();
        m.add_u32(0xDEAD_BEEF);
        assert_eq!(m.get_message_length(), 4);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_u32(), 0xDEAD_BEEF);
    }

    #[test]
    fn test_u32_little_endian_layout() {
        let mut m = msg();
        m.add_u32(0x01020304);
        assert_eq!(m.buffer[8], 0x04);
        assert_eq!(m.buffer[9], 0x03);
        assert_eq!(m.buffer[10], 0x02);
        assert_eq!(m.buffer[11], 0x01);
    }

    // --- add_u64 / get_u64 ---

    #[test]
    fn test_add_get_u64_round_trip() {
        let mut m = msg();
        m.add_u64(0x0102_0304_0506_0708);
        assert_eq!(m.get_message_length(), 8);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_u64(), 0x0102_0304_0506_0708);
    }

    #[test]
    fn test_u64_max_value() {
        let mut m = msg();
        m.add_u64(u64::MAX);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_u64(), u64::MAX);
    }

    // --- add_string / get_string ---

    #[test]
    fn test_add_get_string_round_trip() {
        let mut m = msg();
        m.add_string("hello");
        // 2 (len) + 5 (chars) = 7
        assert_eq!(m.get_message_length(), 7);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_string(0), "hello");
    }

    #[test]
    fn test_add_string_empty() {
        let mut m = msg();
        m.add_string("");
        assert_eq!(m.get_message_length(), 2); // just the length prefix (0)
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_string(0), "");
    }

    #[test]
    fn test_add_string_with_explicit_len() {
        let mut m = msg();
        m.add_string("world");
        // skip the 2-byte length prefix, read raw 5 bytes as string
        m.info.position = INITIAL_BUFFER_POSITION + 2;
        assert_eq!(m.get_string(5), "world");
    }

    #[test]
    fn test_add_string_length_prefix_is_little_endian() {
        let mut m = msg();
        m.add_string("abc");
        // bytes 8-9 should be 3u16 in LE
        assert_eq!(m.buffer[8], 3);
        assert_eq!(m.buffer[9], 0);
    }

    // --- add_bytes / get_bytes ---

    #[test]
    fn test_add_get_bytes_round_trip() {
        let mut m = msg();
        let data = [0xAA, 0xBB, 0xCC];
        m.add_bytes(&data);
        assert_eq!(m.get_message_length(), 3);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_bytes(3), vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_get_bytes_empty_slice() {
        let mut m = msg();
        m.add_bytes(&[]);
        m.info.position = INITIAL_BUFFER_POSITION;
        assert_eq!(m.get_bytes(0), Vec::<u8>::new());
    }

    // --- add_padding_bytes ---

    #[test]
    fn test_add_padding_bytes_advances_length() {
        let mut m = msg();
        m.add_padding_bytes(4);
        assert_eq!(m.get_message_length(), 4);
    }

    #[test]
    fn test_add_padding_bytes_fills_with_0x33() {
        let mut m = msg();
        m.add_padding_bytes(3);
        // padding does NOT advance position (matches C++ behaviour)
        assert_eq!(m.buffer[8], 0x33);
        assert_eq!(m.buffer[9], 0x33);
        assert_eq!(m.buffer[10], 0x33);
    }

    // --- overrun on read ---

    #[test]
    fn test_get_u8_overrun_returns_zero() {
        let mut m = msg();
        // nothing written, so reading should overrun
        assert_eq!(m.get_u8(), 0);
        assert!(m.is_overrun());
    }

    #[test]
    fn test_get_u16_overrun_returns_zero() {
        let mut m = msg();
        assert_eq!(m.get_u16(), 0);
        assert!(m.is_overrun());
    }

    #[test]
    fn test_get_u32_overrun_returns_zero() {
        let mut m = msg();
        assert_eq!(m.get_u32(), 0);
        assert!(m.is_overrun());
    }

    #[test]
    fn test_get_u64_overrun_returns_zero() {
        let mut m = msg();
        assert_eq!(m.get_u64(), 0);
        assert!(m.is_overrun());
    }

    #[test]
    fn test_get_string_overrun_returns_empty() {
        let mut m = msg();
        // write a u16 len of 100 but no body
        m.add_u16(100);
        m.info.position = INITIAL_BUFFER_POSITION;
        // reading 100 bytes past the 2-byte length should overrun
        let s = m.get_string(0);
        assert!(s.is_empty() || m.is_overrun());
    }

    #[test]
    fn test_get_bytes_overrun_returns_empty() {
        let mut m = msg();
        assert_eq!(m.get_bytes(1), Vec::<u8>::new());
        assert!(m.is_overrun());
    }

    // --- skip_bytes ---

    #[test]
    fn test_skip_bytes_advances_position() {
        let mut m = msg();
        m.add_u8(1);
        m.add_u8(2);
        m.add_u8(3);
        m.info.position = INITIAL_BUFFER_POSITION;
        m.skip_bytes(2);
        assert_eq!(m.get_u8(), 3);
    }

    // --- get_message_length matches bytes written ---

    #[test]
    fn test_message_length_after_mixed_writes() {
        let mut m = msg();
        m.add_u8(1); // 1
        m.add_u16(2); // 2
        m.add_u32(3); // 4
        m.add_u64(4); // 8
        m.add_string("hi"); // 2 + 2
        assert_eq!(m.get_message_length(), 1 + 2 + 4 + 8 + 2 + 2);
    }

    // --- get_buffer ---

    #[test]
    fn test_get_buffer_length() {
        let m = msg();
        assert_eq!(m.get_buffer().len(), NETWORKMESSAGE_MAXSIZE as usize);
    }

    // --- set_buffer_position ---

    #[test]
    fn test_set_buffer_position_valid() {
        let mut m = msg();
        assert!(m.set_buffer_position(0));
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
    }

    #[test]
    fn test_set_buffer_position_invalid() {
        let mut m = msg();
        // attempting to set to exactly MAXSIZE - 8 (or beyond) should fail
        let too_large = (NETWORKMESSAGE_MAXSIZE - INITIAL_BUFFER_POSITION as i32) as u16;
        assert!(!m.set_buffer_position(too_large));
    }

    // --- get_length_header ---

    #[test]
    fn test_get_length_header_reads_bytes_0_1() {
        let mut m = msg();
        m.buffer[0] = 0x05;
        m.buffer[1] = 0x00;
        assert_eq!(m.get_length_header(), 5);
    }

    // --- get_header_position ---

    #[test]
    fn test_get_header_position_is_8() {
        let m = msg();
        assert_eq!(m.get_header_position(), 8);
    }

    // --- get_remaining_buffer_length ---

    #[test]
    fn test_remaining_buffer_length_after_write_and_reset_position() {
        let mut m = msg();
        m.add_u8(10);
        m.add_u8(20);
        m.info.position = INITIAL_BUFFER_POSITION; // rewind for reading
        assert_eq!(m.get_remaining_buffer_length(), 2);
        m.get_u8();
        assert_eq!(m.get_remaining_buffer_length(), 1);
    }

    // --- get_previous_u8 ---

    #[test]
    fn test_get_previous_u8_backs_up() {
        let mut m = msg();
        m.add_u8(0xFF);
        m.info.position = INITIAL_BUFFER_POSITION + 1; // just past what we wrote
        assert_eq!(m.get_previous_u8(), 0xFF);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
    }

    // --- constants ---

    #[test]
    fn test_max_body_length_value() {
        assert_eq!(
            MAX_BODY_LENGTH,
            NETWORKMESSAGE_MAXSIZE as usize - HEADER_LENGTH - CHECKSUM_LENGTH - XTEA_MULTIPLE
        );
    }

    #[test]
    fn test_max_protocol_body_length_value() {
        assert_eq!(MAX_PROTOCOL_BODY_LENGTH, MAX_BODY_LENGTH - 10);
    }

    // --- get_string_sized ---

    #[test]
    fn test_get_string_sized_returns_written_bytes() {
        let mut m = msg();
        m.add_u8(0xAA);
        m.add_u8(0xBB);
        m.info.position = INITIAL_BUFFER_POSITION;
        let slice = m.get_string_sized();
        assert_eq!(slice, &[0xAA, 0xBB]);
    }

    // -----------------------------------------------------------------------
    // Additional edge-case tests (Task 2.1 audit)
    // -----------------------------------------------------------------------

    // --- add_string / add_bytes: oversized inputs are silently dropped ---

    #[test]
    fn test_add_string_over_8192_bytes_is_dropped() {
        let mut m = msg();
        // Build a string of 8193 bytes — one over the MAX_STRING_LENGTH limit.
        let big = "x".repeat(MAX_STRING_LENGTH + 1);
        m.add_string(&big);
        // Nothing should have been written.
        assert_eq!(m.get_message_length(), 0);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
    }

    #[test]
    fn test_add_string_exactly_8192_bytes_is_accepted() {
        let mut m = msg();
        let s = "y".repeat(MAX_STRING_LENGTH);
        m.add_string(&s);
        // 2-byte length prefix + 8192 payload bytes
        assert_eq!(m.get_message_length() as usize, 2 + MAX_STRING_LENGTH);
    }

    #[test]
    fn test_add_bytes_over_8192_bytes_is_dropped() {
        let mut m = msg();
        let big = vec![0xFFu8; MAX_STRING_LENGTH + 1];
        m.add_bytes(&big);
        assert_eq!(m.get_message_length(), 0);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
    }

    #[test]
    fn test_add_bytes_exactly_8192_bytes_is_accepted() {
        let mut m = msg();
        let data = vec![0xABu8; MAX_STRING_LENGTH];
        m.add_bytes(&data);
        assert_eq!(m.get_message_length() as usize, MAX_STRING_LENGTH);
    }

    // --- write silently dropped when buffer is full (can_add boundary) ---

    #[test]
    fn test_add_u8_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Fill up to MAX_BODY_LENGTH - 1 bytes so the next add_u8 exactly
        // hits the boundary where can_add returns false.
        // MAX_BODY_LENGTH = NETWORKMESSAGE_MAXSIZE - HEADER_LENGTH - CHECKSUM_LENGTH - XTEA_MULTIPLE
        // position starts at 8; can_add(1) checks (1 + position) < MAX_BODY_LENGTH
        // So we fill until position == MAX_BODY_LENGTH - 1 (i.e. length == MAX_BODY_LENGTH - 9).
        let fill_count = MAX_BODY_LENGTH - 1 - INITIAL_BUFFER_POSITION as usize;
        for _ in 0..fill_count {
            m.add_u8(0x00);
        }
        let len_before = m.get_message_length();
        let pos_before = m.get_buffer_position();
        // This write should be silently dropped.
        m.add_u8(0xFF);
        assert_eq!(m.get_message_length(), len_before);
        assert_eq!(m.get_buffer_position(), pos_before);
    }

    // --- reset does NOT zero the buffer ---

    #[test]
    fn test_reset_does_not_zero_buffer() {
        let mut m = msg();
        m.add_u8(0xAB);
        // Buffer byte at position 8 is now 0xAB.
        m.reset();
        // After reset the cursor is back at 8 and length is 0,
        // but the raw byte is still in the buffer.
        assert_eq!(m.buffer[INITIAL_BUFFER_POSITION as usize], 0xAB);
    }

    // --- get_string: explicit non-zero len that overruns ---

    #[test]
    fn test_get_string_explicit_len_overrun() {
        let mut m = msg();
        // Write 3 bytes but try to read 10 with explicit len.
        m.add_bytes(&[1u8, 2, 3]);
        m.info.position = INITIAL_BUFFER_POSITION;
        let s = m.get_string(10);
        assert!(s.is_empty());
        assert!(m.is_overrun());
    }

    // --- get_string(0) where embedded length is 0 ---

    #[test]
    fn test_get_string_auto_len_zero_payload() {
        let mut m = msg();
        // write a u16(0) length prefix — no body bytes
        m.add_u16(0);
        m.info.position = INITIAL_BUFFER_POSITION;
        let s = m.get_string(0);
        // should succeed and return empty string (0-byte payload)
        assert_eq!(s, "");
        assert!(!m.is_overrun());
    }

    // --- overrun flag cleared by reset ---

    #[test]
    fn test_overrun_flag_cleared_by_reset() {
        let mut m = msg();
        m.get_u8(); // triggers overrun
        assert!(m.is_overrun());
        m.reset();
        assert!(!m.is_overrun());
    }

    // --- skip_bytes with negative count (backward skip) ---

    #[test]
    fn test_skip_bytes_negative_moves_position_backward() {
        let mut m = msg();
        m.add_u8(0x11);
        m.add_u8(0x22);
        // Position is now at INITIAL_BUFFER_POSITION + 2 after two writes.
        // Skip backward by 1 to re-read the last byte.
        m.skip_bytes(-1);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION + 1);
    }

    // --- add_padding_bytes: position NOT advanced ---

    #[test]
    fn test_add_padding_bytes_does_not_advance_position() {
        let mut m = msg();
        let pos_before = m.get_buffer_position();
        m.add_padding_bytes(5);
        // position unchanged; only length grows
        assert_eq!(m.get_buffer_position(), pos_before);
        assert_eq!(m.get_message_length(), 5);
    }

    // --- get_remaining_buffer starts at current position ---

    #[test]
    fn test_get_remaining_buffer_starts_at_current_position() {
        let mut m = msg();
        m.add_u8(0xAA);
        m.add_u8(0xBB);
        m.add_u8(0xCC);
        // Advance position past the first byte to simulate partial read.
        m.info.position = INITIAL_BUFFER_POSITION + 1;
        let rem = m.get_remaining_buffer();
        // First byte of the remaining slice should be 0xBB (second written byte).
        assert_eq!(rem[0], 0xBB);
    }

    // --- get_body_buffer resets position to 2 ---

    #[test]
    fn test_get_body_buffer_resets_position_to_2() {
        let mut m = msg();
        m.add_u32(0xDEAD_BEEF); // advances position to 12
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION + 4);
        let _ = m.get_body_buffer();
        assert_eq!(m.get_buffer_position(), 2);
    }

    // --- buffer position advances correctly after a sequence of writes ---

    #[test]
    fn test_buffer_position_advances_correctly_after_sequence() {
        let mut m = msg();
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
        m.add_u8(1);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION + 1);
        m.add_u16(2);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION + 3);
        m.add_u32(3);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION + 7);
        m.add_u64(4);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION + 15);
    }

    // --- set_buffer_position at exact boundary values ---

    #[test]
    fn test_set_buffer_position_max_valid() {
        let mut m = msg();
        // The largest valid value is NETWORKMESSAGE_MAXSIZE - INITIAL_BUFFER_POSITION - 1.
        let max_valid =
            (NETWORKMESSAGE_MAXSIZE as usize - INITIAL_BUFFER_POSITION as usize - 1) as u16;
        assert!(m.set_buffer_position(max_valid));
    }

    #[test]
    fn test_set_buffer_position_one_past_max_is_invalid() {
        let mut m = msg();
        let one_past = (NETWORKMESSAGE_MAXSIZE as usize - INITIAL_BUFFER_POSITION as usize) as u16;
        assert!(!m.set_buffer_position(one_past));
    }

    // --- get_length_header encodes multi-byte value correctly ---

    #[test]
    fn test_get_length_header_multi_byte_value() {
        let mut m = msg();
        // Manually stamp a 300 (0x012C) LE value at bytes 0-1.
        m.buffer[0] = 0x2C; // low byte of 300
        m.buffer[1] = 0x01; // high byte of 300
        assert_eq!(m.get_length_header(), 300);
    }

    // -----------------------------------------------------------------------
    // Phase 2 audit: cover remaining gaps (Default impl, overflow paths,
    // mutable buffer accessor, set_message_length, get_string_sized fallback,
    // Debug impl, and the C++-API gaps: add_position/get_position/add_double).
    // -----------------------------------------------------------------------

    // --- Default::default() constructs an equivalent empty message ---

    #[test]
    fn test_default_impl_constructs_empty_message() {
        let m: NetworkMessage = NetworkMessage::default();
        assert!(m.is_empty());
        assert_eq!(m.get_message_length(), 0);
        assert_eq!(m.get_buffer_position(), INITIAL_BUFFER_POSITION);
        assert!(!m.is_overrun());
        assert_eq!(m.get_buffer().len(), NETWORKMESSAGE_MAXSIZE as usize);
    }

    // --- add_u16 / add_u32 / add_u64 / add_padding_bytes overflow paths ---
    //
    // Fill the buffer up to a position where can_add(n) returns false for the
    // specific n, then verify the next write is silently dropped (length and
    // position unchanged).

    fn fill_to_position(m: &mut NetworkMessage, target_pos: usize) {
        while (m.get_buffer_position() as usize) < target_pos {
            // add_u8 advances position by 1 and length by 1.
            m.add_u8(0);
        }
    }

    #[test]
    fn test_add_u16_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Want: (2 + position) >= MAX_BODY_LENGTH, i.e. position >= MAX_BODY_LENGTH - 2.
        // Use position = MAX_BODY_LENGTH - 2 (smallest failing value).
        fill_to_position(&mut m, MAX_BODY_LENGTH - 2);
        let len_before = m.get_message_length();
        let pos_before = m.get_buffer_position();
        m.add_u16(0xFFFF);
        assert_eq!(m.get_message_length(), len_before);
        assert_eq!(m.get_buffer_position(), pos_before);
    }

    #[test]
    fn test_add_u32_silently_dropped_when_buffer_full() {
        let mut m = msg();
        fill_to_position(&mut m, MAX_BODY_LENGTH - 4);
        let len_before = m.get_message_length();
        let pos_before = m.get_buffer_position();
        m.add_u32(0xDEAD_BEEF);
        assert_eq!(m.get_message_length(), len_before);
        assert_eq!(m.get_buffer_position(), pos_before);
    }

    #[test]
    fn test_add_u64_silently_dropped_when_buffer_full() {
        let mut m = msg();
        fill_to_position(&mut m, MAX_BODY_LENGTH - 8);
        let len_before = m.get_message_length();
        let pos_before = m.get_buffer_position();
        m.add_u64(u64::MAX);
        assert_eq!(m.get_message_length(), len_before);
        assert_eq!(m.get_buffer_position(), pos_before);
    }

    #[test]
    fn test_add_padding_bytes_silently_dropped_when_buffer_full() {
        let mut m = msg();
        // Pick n = 4 padding bytes; fill until position is MAX_BODY_LENGTH - 4
        // so the next padding write fails can_add(4).
        fill_to_position(&mut m, MAX_BODY_LENGTH - 4);
        let len_before = m.get_message_length();
        let pos_before = m.get_buffer_position();
        m.add_padding_bytes(4);
        // can_add uses strict `<` so 4 + position == MAX_BODY_LENGTH fails.
        assert_eq!(m.get_message_length(), len_before);
        assert_eq!(m.get_buffer_position(), pos_before);
    }

    // --- get_buffer_mut returns a mutable view of the same backing buffer ---

    #[test]
    fn test_get_buffer_mut_allows_in_place_mutation() {
        let mut m = msg();
        {
            let buf = m.get_buffer_mut();
            assert_eq!(buf.len(), NETWORKMESSAGE_MAXSIZE as usize);
            buf[0] = 0xAB;
            buf[1] = 0xCD;
        }
        // The mutation is visible through the read-only accessor too.
        assert_eq!(m.get_buffer()[0], 0xAB);
        assert_eq!(m.get_buffer()[1], 0xCD);
        // ...and through get_length_header which reads bytes 0-1.
        assert_eq!(m.get_length_header(), u16::from_le_bytes([0xAB, 0xCD]));
    }

    // --- set_message_length overrides the internal length counter ---

    #[test]
    fn test_set_message_length_overrides_internal_length() {
        let mut m = msg();
        m.add_u8(1);
        m.add_u8(2);
        m.add_u8(3);
        assert_eq!(m.get_message_length(), 3);
        m.set_message_length(42);
        assert_eq!(m.get_message_length(), 42);
        // is_empty reflects the new length.
        m.set_message_length(0);
        assert!(m.is_empty());
    }

    // --- get_string_sized fallback: returns &[] when start > end ---

    #[test]
    fn test_get_string_sized_returns_empty_when_position_past_written_data() {
        let mut m = msg();
        m.add_u8(0xAA); // length = 1, end = 8 + 1 = 9
                        // Force position beyond end (which is 9) — start > end triggers fallback.
        m.info.position = 100;
        let slice = m.get_string_sized();
        assert!(slice.is_empty());
    }

    // --- Debug impl: includes the documented field names ---

    #[test]
    fn test_debug_impl_includes_length_position_and_overrun() {
        let mut m = msg();
        m.add_u32(0xDEAD_BEEF);
        let s = format!("{:?}", m);
        assert!(s.contains("NetworkMessage"));
        assert!(s.contains("length"));
        assert!(s.contains("position"));
        assert!(s.contains("overrun"));
    }

    // --- add_position / get_position round-trip (C++ NetworkMessage::addPosition) ---

    #[test]
    fn test_add_get_position_round_trip() {
        let mut m = msg();
        let pos = Position::new(0x1234, 0x5678, 7);
        m.add_position(pos);
        // 2 + 2 + 1 = 5 bytes written.
        assert_eq!(m.get_message_length(), 5);
        m.info.position = INITIAL_BUFFER_POSITION;
        let read = m.get_position();
        assert_eq!(read.x, 0x1234);
        assert_eq!(read.y, 0x5678);
        assert_eq!(read.z, 7);
    }

    #[test]
    fn test_add_position_little_endian_layout() {
        let mut m = msg();
        m.add_position(Position::new(0x0102, 0x0304, 0x05));
        // x LE → bytes 8-9 = 02 01; y LE → bytes 10-11 = 04 03; z = byte 12.
        assert_eq!(m.buffer[8], 0x02);
        assert_eq!(m.buffer[9], 0x01);
        assert_eq!(m.buffer[10], 0x04);
        assert_eq!(m.buffer[11], 0x03);
        assert_eq!(m.buffer[12], 0x05);
    }

    #[test]
    fn test_get_position_zero_on_overrun() {
        let mut m = msg();
        // Empty buffer: every read should overrun and return zeros.
        let p = m.get_position();
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
        assert_eq!(p.z, 0);
        assert!(m.is_overrun());
    }

    // --- add_double: precision byte + scaled u32 LE ---

    #[test]
    fn test_add_double_writes_precision_byte_and_scaled_u32() {
        let mut m = msg();
        // Pick a value/precision that has an exact representation.
        // value = 1.0, precision = 0 → scale = 1, scaled = 1.0 + i32::MAX = 2147483648.
        m.add_double(1.0, 0);
        // 1 byte precision + 4 bytes scaled value = 5 bytes.
        assert_eq!(m.get_message_length(), 5);
        // First byte: precision (0).
        assert_eq!(m.buffer[INITIAL_BUFFER_POSITION as usize], 0);
        // Next 4 bytes (LE): i32::MAX + 1 = 0x8000_0000.
        let scaled_bytes = &m.buffer
            [(INITIAL_BUFFER_POSITION as usize) + 1..(INITIAL_BUFFER_POSITION as usize) + 5];
        let scaled = u32::from_le_bytes([
            scaled_bytes[0],
            scaled_bytes[1],
            scaled_bytes[2],
            scaled_bytes[3],
        ]);
        assert_eq!(scaled, (i32::MAX as u32).wrapping_add(1));
    }

    #[test]
    fn test_add_double_default_precision_two() {
        let mut m = msg();
        // value = 1.25, precision = 2 → scale = 100, scaled = 125.0 + i32::MAX.
        m.add_double(1.25, 2);
        assert_eq!(m.get_message_length(), 5);
        assert_eq!(m.buffer[INITIAL_BUFFER_POSITION as usize], 2);
        let start = (INITIAL_BUFFER_POSITION as usize) + 1;
        let scaled = u32::from_le_bytes([
            m.buffer[start],
            m.buffer[start + 1],
            m.buffer[start + 2],
            m.buffer[start + 3],
        ]);
        assert_eq!(scaled, (i32::MAX as u32).wrapping_add(125));
    }

    // -----------------------------------------------------------------------
    // add_item_id  (mirrors C++ NetworkMessage::addItemId)
    // -----------------------------------------------------------------------

    /// Helper: extract the payload bytes written by the message.
    fn body(m: &NetworkMessage) -> &[u8] {
        let start = INITIAL_BUFFER_POSITION as usize;
        let end = start + m.info.length as usize;
        &m.buffer[start..end]
    }

    #[test]
    fn test_add_item_id_writes_two_bytes_le() {
        let mut m = msg();
        m.add_item_id(0x1234);
        assert_eq!(m.get_message_length(), 2);
        assert_eq!(body(&m), &[0x34, 0x12]);
    }

    // -----------------------------------------------------------------------
    // add_item_payload — fresh-item-by-id (NetworkMessage::addItem(id, count))
    // -----------------------------------------------------------------------

    fn meta_plain(client_id: u16) -> ItemTypeMeta {
        ItemTypeMeta {
            client_id,
            ..ItemTypeMeta::default()
        }
    }

    #[test]
    fn test_add_item_payload_plain_writes_only_client_id() {
        let mut m = msg();
        m.add_item_payload(7, meta_plain(0x0AB1));
        // Just the u16 client id, no sub-type byte.
        assert_eq!(body(&m), &[0xB1, 0x0A]);
    }

    #[test]
    fn test_add_item_payload_stackable_writes_count_byte() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 100,
            stackable: true,
            ..Default::default()
        };
        m.add_item_payload(42, meta);
        // client_id LE (100 = 0x64) + count byte
        assert_eq!(body(&m), &[0x64, 0x00, 42]);
    }

    #[test]
    fn test_add_item_payload_splash_writes_fluid_map_byte() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 0x0102,
            is_splash: true,
            ..Default::default()
        };
        // index 2 -> FLUID_MAP[2] = 5 (CLIENTFLUID_RED)
        m.add_item_payload(2, meta);
        assert_eq!(body(&m), &[0x02, 0x01, 5]);
    }

    #[test]
    fn test_add_item_payload_fluid_container_masks_count_low_3_bits() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_fluid_container: true,
            ..Default::default()
        };
        // count = 0b0000_1010 & 0b111 = 2 -> FLUID_MAP[2] = 5
        m.add_item_payload(0x0A, meta);
        assert_eq!(body(&m), &[0x01, 0x00, 5]);
    }

    #[test]
    fn test_add_item_payload_container_writes_two_zero_bytes() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 50,
            is_container: true,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        // client_id LE + 0x00 (loot icon) + 0x00 (quiver ammo)
        assert_eq!(body(&m), &[50, 0, 0x00, 0x00]);
    }

    #[test]
    fn test_add_item_payload_classification_writes_zero_tier_byte() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 9,
            classification: 3,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        assert_eq!(body(&m), &[9, 0, 0x00]);
    }

    #[test]
    fn test_add_item_payload_show_client_charges_writes_u32_charges_plus_zero() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_charges: true,
            charges: 0x12345678,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        // client_id (1 LE) + charges (LE) + 0x00
        assert_eq!(body(&m), &[1, 0, 0x78, 0x56, 0x34, 0x12, 0x00]);
    }

    #[test]
    fn test_add_item_payload_show_client_duration_writes_decay_time_min() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_duration: true,
            decay_time_min: 0x000000FF,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        assert_eq!(body(&m), &[1, 0, 0xFF, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_add_item_payload_podium_appends_default_block() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 0xABCD,
            is_podium: true,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        // client_id LE + lookType 0 (u16) + lookMount 0 (u16) + 2 (dir) + 1 (visible)
        assert_eq!(body(&m), &[0xCD, 0xAB, 0, 0, 0, 0, 2, 0x01]);
    }

    #[test]
    fn test_add_item_payload_stackable_and_podium_combined() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            stackable: true,
            is_podium: true,
            ..Default::default()
        };
        m.add_item_payload(5, meta);
        // client_id + count + podium tail
        assert_eq!(body(&m), &[1, 0, 5, 0, 0, 0, 0, 2, 0x01]);
    }

    #[test]
    fn test_add_item_payload_stackable_takes_priority_over_classification() {
        // The C++ code uses else-if, so stackable wins over classification.
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            stackable: true,
            classification: 5,
            ..Default::default()
        };
        m.add_item_payload(7, meta);
        // No tier byte should appear — only the count byte from the stackable branch.
        assert_eq!(body(&m), &[1, 0, 7]);
    }

    // -----------------------------------------------------------------------
    // add_item_instance — per-instance (NetworkMessage::addItem(const Item*))
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_item_instance_plain_writes_only_client_id() {
        let mut m = msg();
        m.add_item_instance(0, 0, 0, None, None, meta_plain(0x0099));
        assert_eq!(body(&m), &[0x99, 0x00]);
    }

    #[test]
    fn test_add_item_instance_stackable_writes_count() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            stackable: true,
            ..Default::default()
        };
        m.add_item_instance(42, 0, 0, None, None, meta);
        assert_eq!(body(&m), &[1, 0, 42]);
    }

    #[test]
    fn test_add_item_instance_fluid_container_uses_fluid_map() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_fluid_container: true,
            ..Default::default()
        };
        // fluid type index 3 -> FLUID_MAP[3] = 3 (CLIENTFLUID_BROWN_1)
        m.add_item_instance(3, 0, 0, None, None, meta);
        assert_eq!(body(&m), &[1, 0, 3]);
    }

    #[test]
    fn test_add_item_instance_classification_writes_zero_tier_byte() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            classification: 1,
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0, None, None, meta);
        assert_eq!(body(&m), &[1, 0, 0x00]);
    }

    #[test]
    fn test_add_item_instance_show_client_charges_writes_provided_charges() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_charges: true,
            ..Default::default()
        };
        m.add_item_instance(0, 0xAA, 0, None, None, meta);
        assert_eq!(body(&m), &[1, 0, 0xAA, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_add_item_instance_show_client_duration_writes_provided_duration_seconds() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            show_client_duration: true,
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0xBB, None, None, meta);
        assert_eq!(body(&m), &[1, 0, 0xBB, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_add_item_instance_container_non_quiver_writes_two_zero_bytes() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_container: true,
            weapon_type: 0, // not WEAPON_QUIVER
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0, None, None, meta);
        assert_eq!(body(&m), &[1, 0, 0x00, 0x00]);
    }

    #[test]
    fn test_add_item_instance_container_quiver_without_ammo_count() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_container: true,
            weapon_type: WEAPON_TYPE_QUIVER,
            ..Default::default()
        };
        // ammo_count = None -> still writes the trailing 0x00 byte
        m.add_item_instance(0, 0, 0, None, None, meta);
        assert_eq!(body(&m), &[1, 0, 0x00, 0x00]);
    }

    #[test]
    fn test_add_item_instance_container_quiver_with_ammo_count() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_container: true,
            weapon_type: WEAPON_TYPE_QUIVER,
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0, Some(0x01020304), None, meta);
        assert_eq!(body(&m), &[1, 0, 0x00, 0x01, 0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn test_add_item_instance_podium_without_outfit_or_mount_writes_zero_pairs() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_podium: true,
            ..Default::default()
        };
        // No PodiumMeta supplied → default (all false / 0).
        m.add_item_instance(0, 0, 0, None, None, meta);
        // lookType 0 (u16) + lookMount 0 (u16) + direction 0 + platform 0
        assert_eq!(body(&m), &[1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_add_item_instance_podium_show_outfit_nonzero_look_type_writes_outfit_fields() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_podium: true,
            ..Default::default()
        };
        let podium = PodiumMeta {
            show_outfit: true,
            show_mount: false,
            show_platform: false,
            look_type: 0x0102,
            look_head: 1,
            look_body: 2,
            look_legs: 3,
            look_feet: 4,
            look_addons: 5,
            direction: 3,
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0, None, Some(podium), meta);
        // client_id + lookType (LE) + 5 outfit bytes + lookMount 0 (u16) + dir + platform
        assert_eq!(body(&m), &[1, 0, 0x02, 0x01, 1, 2, 3, 4, 5, 0, 0, 3, 0x00]);
    }

    #[test]
    fn test_add_item_instance_podium_show_outfit_zero_look_type_skips_outfit_bytes() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_podium: true,
            ..Default::default()
        };
        let podium = PodiumMeta {
            show_outfit: true,
            look_type: 0,
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0, None, Some(podium), meta);
        // lookType 0 (u16) — no extra outfit bytes — + lookMount 0 (u16) + dir 0 + platform 0
        assert_eq!(body(&m), &[1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_add_item_instance_podium_show_mount_nonzero_writes_mount_fields() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_podium: true,
            ..Default::default()
        };
        let podium = PodiumMeta {
            show_outfit: false,
            show_mount: true,
            show_platform: true,
            look_mount: 0x0203,
            look_mount_head: 10,
            look_mount_body: 20,
            look_mount_legs: 30,
            look_mount_feet: 40,
            direction: 2,
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0, None, Some(podium), meta);
        // client_id + lookType 0 (u16, !show_outfit) + lookMount LE + 4 mount bytes +
        // dir 2 + platform 0x01
        assert_eq!(body(&m), &[1, 0, 0, 0, 0x03, 0x02, 10, 20, 30, 40, 2, 0x01]);
    }

    #[test]
    fn test_add_item_instance_podium_show_mount_zero_look_mount_skips_mount_bytes() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_podium: true,
            ..Default::default()
        };
        let podium = PodiumMeta {
            show_mount: true,
            look_mount: 0,
            ..Default::default()
        };
        m.add_item_instance(0, 0, 0, None, Some(podium), meta);
        // lookType 0 (u16) + lookMount 0 (u16) — no mount bytes — + dir 0 + platform 0
        assert_eq!(body(&m), &[1, 0, 0, 0, 0, 0, 0, 0]);
    }

    // -----------------------------------------------------------------------
    // C++ parity sanity checks — explicit "byte-for-byte" mirrors
    // -----------------------------------------------------------------------
    //
    // These tests document the exact wire bytes that the C++
    // NetworkMessage::addItem produces for the most common cases.  Changing
    // any byte here means the wire protocol has diverged from C++.

    #[test]
    fn test_cpp_parity_stackable_count_5() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 0x0064,
            stackable: true,
            ..Default::default()
        };
        m.add_item_payload(5, meta);
        // C++ would write: 64 00 05
        assert_eq!(body(&m), &[0x64, 0x00, 0x05]);
    }

    #[test]
    fn test_cpp_parity_fluid_map_index_0_is_empty() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_fluid_container: true,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        // FLUID_MAP[0] = 0 (CLIENTFLUID_EMPTY)
        assert_eq!(body(&m), &[1, 0, 0]);
    }

    #[test]
    fn test_cpp_parity_fluid_map_index_7_is_purple() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            is_splash: true,
            ..Default::default()
        };
        m.add_item_payload(7, meta);
        // FLUID_MAP[7] = 2 (CLIENTFLUID_PURPLE)
        assert_eq!(body(&m), &[1, 0, 2]);
    }

    #[test]
    fn test_cpp_parity_isolated_container_byte_layout() {
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 0x1A2B,
            is_container: true,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        assert_eq!(body(&m), &[0x2B, 0x1A, 0x00, 0x00]);
    }

    #[test]
    fn test_cpp_parity_classification_takes_precedence_over_show_charges() {
        // In C++ the else-if chain skips show_client_charges when
        // classification > 0.  Verify that here.
        let mut m = msg();
        let meta = ItemTypeMeta {
            client_id: 1,
            classification: 1,
            show_client_charges: true,
            charges: 0xDEAD,
            ..Default::default()
        };
        m.add_item_payload(0, meta);
        // Only the tier byte (0x00) — NOT the charges + brand-new bytes.
        assert_eq!(body(&m), &[1, 0, 0x00]);
    }
}
