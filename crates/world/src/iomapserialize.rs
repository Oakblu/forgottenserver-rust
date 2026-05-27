// Migrated from forgottenserver/src/iomapserialize.h + iomapserialize.cpp
//
// This module serializes/deserializes house tile data and house state.
//
// The C++ implementation persists to a MySQL database; the Rust port instead
// provides a self-contained binary serialization format suitable for unit
// testing and later integration with any storage back-end.
//
// ── House items (tile_store format) ────────────────────────────────────────
// Binary format for a single tile record:
//   [0..1]  x (u16 LE)
//   [2..3]  y (u16 LE)
//   [4]     z (u8)
//   [5..8]  item_count (u32 LE)
//   for each item:
//     [0..1]  item_id (u16 LE)
//     [2..]   attribute bytes (opaque, 0x00-terminated)
//
// A complete house-items payload is prefixed by:
//   [0..3]  house_id (u32 LE)
//   [4..7]  tile_count (u32 LE)
//   then tile records concatenated.
//
// ── House state (houses + house_lists format) ───────────────────────────────
// Binary format for house state:
//   [0..3]   house_id (u32 LE)
//   [4..7]   owner_guid (u32 LE)
//   [8..11]  paid_until (u32 LE)
//   [12..15] pay_rent_warnings (u32 LE)
//   [16..17] name_len (u16 LE)
//   [18..]   name bytes (UTF-8)
//
// ── Item property encoding (mirrors C++ PropWriteStream / serializeAttr) ───
// Binary format for a single item:
//   [0..1]  item_id (u16 LE)
//   [2..]   attribute bytes (raw key-value sequence)
//   [last]  0x00  (end-of-attributes sentinel)
// For a container item a ATTR_CONTAINER_ITEMS (0x14) attribute is inserted
// before the sentinel, followed by a u32 child-count and each child item.

use forgottenserver_common::position::Position;

// ---------------------------------------------------------------------------
// Item attribute constants  (C++ attr_t enum subset we need for encoding)
// ---------------------------------------------------------------------------

/// Sentinel byte that terminates the attribute list of an item.
pub const ATTR_END: u8 = 0x00;

/// Attribute: container items follow (u32 count, then items).
pub const ATTR_CONTAINER_ITEMS: u8 = 0x14; // == 20

// ---------------------------------------------------------------------------
// ItemRecord — in-memory representation for encoding
// ---------------------------------------------------------------------------

/// A serialisable item with optional child items (container).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRecord {
    /// The numeric item type ID.
    pub id: u16,
    /// Raw attribute bytes (key + value bytes, NOT terminated — we add the
    /// 0x00 sentinel automatically during encoding).
    pub attr_bytes: Vec<u8>,
    /// Child items if this is a container.  Empty for non-containers.
    pub children: Vec<ItemRecord>,
}

impl ItemRecord {
    /// Creates a simple item with no attributes and no children.
    pub fn new(id: u16) -> Self {
        ItemRecord {
            id,
            attr_bytes: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Creates an item with raw attribute bytes.
    pub fn with_attrs(id: u16, attr_bytes: Vec<u8>) -> Self {
        ItemRecord {
            id,
            attr_bytes,
            children: Vec::new(),
        }
    }

    /// Creates a container item with children.
    pub fn with_children(id: u16, children: Vec<ItemRecord>) -> Self {
        ItemRecord {
            id,
            attr_bytes: Vec::new(),
            children,
        }
    }
}

// ---------------------------------------------------------------------------
// TileRecord — one tile worth of saveable items
// ---------------------------------------------------------------------------

/// A tile position together with its list of saveable items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileRecord {
    pub x: u16,
    pub y: u16,
    pub z: u8,
    pub items: Vec<ItemRecord>,
}

impl TileRecord {
    pub fn new(x: u16, y: u16, z: u8, items: Vec<ItemRecord>) -> Self {
        TileRecord { x, y, z, items }
    }
}

// ---------------------------------------------------------------------------
// HouseState — plain data returned by deserialization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseState {
    pub house_id: u32,
    pub owner_guid: u32,
    /// Unix timestamp until rent is paid.
    pub paid_until: u32,
    /// Number of rent-payment warnings issued.
    pub pay_rent_warnings: u32,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Item encoding / decoding
// ---------------------------------------------------------------------------

/// Encodes a single `ItemRecord` into binary (id + attrs + 0x00 sentinel).
///
/// If the item has children a `ATTR_CONTAINER_ITEMS` section is prepended to
/// the sentinel, matching the C++ `IOMapSerialize::saveItem` pattern.
pub fn encode_item(item: &ItemRecord) -> Vec<u8> {
    let mut buf = Vec::new();

    // id (u16 LE)
    buf.extend_from_slice(&item.id.to_le_bytes());

    // attr bytes (raw, caller-controlled)
    buf.extend_from_slice(&item.attr_bytes);

    // container children
    if !item.children.is_empty() {
        buf.push(ATTR_CONTAINER_ITEMS);
        buf.extend_from_slice(&(item.children.len() as u32).to_le_bytes());
        // C++ iterates children in REVERSE (getReversedItems)
        for child in item.children.iter().rev() {
            buf.extend_from_slice(&encode_item(child));
        }
    }

    // end-of-attributes sentinel
    buf.push(ATTR_END);

    buf
}

/// Decodes one `ItemRecord` from `data` starting at `pos`.
/// Returns `(ItemRecord, new_pos)`.
pub fn decode_item(data: &[u8], pos: usize) -> Result<(ItemRecord, usize), String> {
    if data.len() < pos + 2 {
        return Err(format!(
            "decode_item: not enough bytes for item_id at pos {} (have {})",
            pos,
            data.len()
        ));
    }
    let id = u16::from_le_bytes([data[pos], data[pos + 1]]);
    let mut p = pos + 2;

    let mut attr_bytes = Vec::new();
    let mut children: Vec<ItemRecord> = Vec::new();

    loop {
        if p >= data.len() {
            return Err(format!(
                "decode_item: unexpected end reading attrs of item {}",
                id
            ));
        }
        let attr = data[p];
        p += 1;

        if attr == ATTR_END {
            break;
        }

        if attr == ATTR_CONTAINER_ITEMS {
            // u32 child count
            if data.len() < p + 4 {
                return Err(format!(
                    "decode_item: too short for container count at pos {}",
                    p
                ));
            }
            let count =
                u32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]) as usize;
            p += 4;

            // Children were written in reverse by encode_item; we read them in order
            let mut decoded_children = Vec::with_capacity(count);
            for _ in 0..count {
                let (child, np) = decode_item(data, p)?;
                decoded_children.push(child);
                p = np;
            }
            // Reverse to restore original order
            decoded_children.reverse();
            children = decoded_children;
        } else {
            // Unknown attribute — we store the raw byte in attr_bytes.
            // (In a real implementation each attribute has a fixed or
            // length-prefixed payload; for the purposes of this migration we
            // treat all non-container, non-sentinel bytes as opaque.)
            attr_bytes.push(attr);
        }
    }

    Ok((
        ItemRecord {
            id,
            attr_bytes,
            children,
        },
        p,
    ))
}

// ---------------------------------------------------------------------------
// Tile encoding / decoding
// ---------------------------------------------------------------------------

/// Encodes a `TileRecord` into the binary format used by tile_store.
///
/// Format: x(u16) + y(u16) + z(u8) + item_count(u32) + items…
pub fn encode_tile(tile: &TileRecord) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.extend_from_slice(&tile.x.to_le_bytes());
    buf.extend_from_slice(&tile.y.to_le_bytes());
    buf.push(tile.z);
    buf.extend_from_slice(&(tile.items.len() as u32).to_le_bytes());

    for item in &tile.items {
        buf.extend_from_slice(&encode_item(item));
    }

    buf
}

/// Decodes a `TileRecord` from `data` starting at `pos`.
/// Returns `(TileRecord, new_pos)`.
pub fn decode_tile(data: &[u8], pos: usize) -> Result<(TileRecord, usize), String> {
    if data.len() < pos + 9 {
        return Err(format!(
            "decode_tile: too short at pos {} (have {}, need 9)",
            pos,
            data.len()
        ));
    }
    let x = u16::from_le_bytes([data[pos], data[pos + 1]]);
    let y = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
    let z = data[pos + 4];
    let count =
        u32::from_le_bytes([data[pos + 5], data[pos + 6], data[pos + 7], data[pos + 8]]) as usize;
    let mut p = pos + 9;

    let mut items = Vec::with_capacity(count);
    for i in 0..count {
        let (item, np) = decode_item(data, p)
            .map_err(|e| format!("decode_tile: error decoding item {} at pos {}: {}", i, p, e))?;
        items.push(item);
        p = np;
    }

    Ok((TileRecord::new(x, y, z, items), p))
}

// ---------------------------------------------------------------------------
// House items serialization (tile-list format, LEGACY — position-only)
// ---------------------------------------------------------------------------
//
// These functions are retained for backward compatibility.  New code should
// use `serialize_house_tiles` / `deserialize_house_tiles` which carry full
// item data.

/// Serializes the list of tile positions for house `house_id`.
///
/// This is the legacy (position-only) format.
pub fn serialize_house_items(house_id: u32, tile_positions: &[Position]) -> Vec<u8> {
    let count = tile_positions.len() as u32;
    let mut buf = Vec::with_capacity(4 + 4 + tile_positions.len() * 5);

    buf.extend_from_slice(&house_id.to_le_bytes());
    buf.extend_from_slice(&count.to_le_bytes());

    for pos in tile_positions {
        buf.extend_from_slice(&pos.x.to_le_bytes());
        buf.extend_from_slice(&pos.y.to_le_bytes());
        buf.push(pos.z);
    }

    buf
}

/// Deserializes a house-items payload produced by [`serialize_house_items`].
///
/// Returns `(house_id, tile_positions)` on success.
pub fn deserialize_house_items(bytes: &[u8]) -> Result<(u32, Vec<Position>), String> {
    if bytes.len() < 8 {
        return Err(format!(
            "house-items payload too short: {} bytes (need at least 8)",
            bytes.len()
        ));
    }

    let house_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;

    let required = 8 + count * 5;
    if bytes.len() < required {
        return Err(format!(
            "house-items payload too short for {} tiles: {} bytes (need {})",
            count,
            bytes.len(),
            required
        ));
    }

    let mut positions = Vec::with_capacity(count);
    let mut offset = 8usize;

    for _ in 0..count {
        let x = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        let y = u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]);
        let z = bytes[offset + 4];
        positions.push(Position::new(x, y, z));
        offset += 5;
    }

    Ok((house_id, positions))
}

// ---------------------------------------------------------------------------
// House tiles serialization (full item-data format)
// ---------------------------------------------------------------------------

/// Serialises a set of `TileRecord`s for house `house_id`.
///
/// Format: house_id(u32) + tile_count(u32) + encoded tiles…
pub fn serialize_house_tiles(house_id: u32, tiles: &[TileRecord]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&house_id.to_le_bytes());
    buf.extend_from_slice(&(tiles.len() as u32).to_le_bytes());
    for tile in tiles {
        buf.extend_from_slice(&encode_tile(tile));
    }
    buf
}

/// Deserialises a payload produced by [`serialize_house_tiles`].
///
/// Returns `(house_id, tiles)`.
pub fn deserialize_house_tiles(bytes: &[u8]) -> Result<(u32, Vec<TileRecord>), String> {
    if bytes.len() < 8 {
        return Err(format!(
            "house-tiles payload too short: {} bytes (need at least 8)",
            bytes.len()
        ));
    }
    let house_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let tile_count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let mut p = 8usize;

    let mut tiles = Vec::with_capacity(tile_count);
    for i in 0..tile_count {
        let (tile, np) = decode_tile(bytes, p)
            .map_err(|e| format!("deserialize_house_tiles: error decoding tile {}: {}", i, e))?;
        tiles.push(tile);
        p = np;
    }

    Ok((house_id, tiles))
}

// ---------------------------------------------------------------------------
// House state serialization
// ---------------------------------------------------------------------------

/// Serializes the owning metadata for house `house_id`.
///
/// Format: house_id(u32) + owner_guid(u32) + paid_until(u32) +
///         pay_rent_warnings(u32) + name_len(u16) + name(UTF-8)
pub fn serialize_house_state(
    house_id: u32,
    owner_guid: u32,
    paid_until: u32,
    pay_rent_warnings: u32,
    name: &str,
) -> Vec<u8> {
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len() as u16;
    let mut buf = Vec::with_capacity(4 + 4 + 4 + 4 + 2 + name_bytes.len());

    buf.extend_from_slice(&house_id.to_le_bytes());
    buf.extend_from_slice(&owner_guid.to_le_bytes());
    buf.extend_from_slice(&paid_until.to_le_bytes());
    buf.extend_from_slice(&pay_rent_warnings.to_le_bytes());
    buf.extend_from_slice(&name_len.to_le_bytes());
    buf.extend_from_slice(name_bytes);

    buf
}

/// Deserializes a house-state payload produced by [`serialize_house_state`].
pub fn deserialize_house_state(bytes: &[u8]) -> Result<HouseState, String> {
    if bytes.len() < 18 {
        return Err(format!(
            "house-state payload too short: {} bytes (need at least 18)",
            bytes.len()
        ));
    }

    let house_id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let owner_guid = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let paid_until = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let pay_rent_warnings = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    let name_len = u16::from_le_bytes([bytes[16], bytes[17]]) as usize;

    let required = 18 + name_len;
    if bytes.len() < required {
        return Err(format!(
            "house-state payload too short for name of {} bytes: {} (need {})",
            name_len,
            bytes.len(),
            required
        ));
    }

    let name = std::str::from_utf8(&bytes[18..18 + name_len])
        .map_err(|e| format!("house-state: invalid UTF-8 in name: {e}"))?
        .to_owned();

    Ok(HouseState {
        house_id,
        owner_guid,
        paid_until,
        pay_rent_warnings,
        name,
    })
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

    fn make_positions() -> Vec<Position> {
        vec![
            Position::new(100, 200, 7),
            Position::new(101, 200, 7),
            Position::new(102, 201, 7),
        ]
    }

    fn simple_item(id: u16) -> ItemRecord {
        ItemRecord::new(id)
    }

    fn item_with_attrs(id: u16, attr_bytes: Vec<u8>) -> ItemRecord {
        ItemRecord::with_attrs(id, attr_bytes)
    }

    fn simple_tile(x: u16, y: u16, z: u8, items: Vec<ItemRecord>) -> TileRecord {
        TileRecord::new(x, y, z, items)
    }

    // -----------------------------------------------------------------------
    // House item serialization (legacy — position-only)
    // -----------------------------------------------------------------------

    #[test]
    fn serialize_house_items_produces_bytes() {
        let positions = make_positions();
        let bytes = serialize_house_items(42, &positions);
        // 4 (house_id) + 4 (count) + 3 * 5 (tiles) = 23
        assert_eq!(bytes.len(), 23);
    }

    #[test]
    fn serialize_house_items_empty_positions() {
        let bytes = serialize_house_items(1, &[]);
        // 4 + 4 = 8
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn deserialize_house_items_reads_house_id() {
        let bytes = serialize_house_items(99, &make_positions());
        let (house_id, _) = deserialize_house_items(&bytes).unwrap();
        assert_eq!(house_id, 99);
    }

    #[test]
    fn deserialize_house_items_reads_positions() {
        let positions = make_positions();
        let bytes = serialize_house_items(1, &positions);
        let (_, decoded) = deserialize_house_items(&bytes).unwrap();
        assert_eq!(decoded, positions);
    }

    #[test]
    fn house_items_round_trip() {
        let house_id = 7u32;
        let positions = make_positions();
        let bytes = serialize_house_items(house_id, &positions);
        let (decoded_id, decoded_positions) = deserialize_house_items(&bytes).unwrap();
        assert_eq!(decoded_id, house_id);
        assert_eq!(decoded_positions, positions);
    }

    #[test]
    fn house_items_round_trip_empty() {
        let house_id = 3u32;
        let positions: Vec<Position> = vec![];
        let bytes = serialize_house_items(house_id, &positions);
        let (decoded_id, decoded_positions) = deserialize_house_items(&bytes).unwrap();
        assert_eq!(decoded_id, house_id);
        assert!(decoded_positions.is_empty());
    }

    #[test]
    fn deserialize_house_items_error_on_too_short_payload() {
        let result = deserialize_house_items(&[0x01, 0x00]);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_house_items_error_when_count_exceeds_payload() {
        let mut bytes = vec![0x01, 0x00, 0x00, 0x00];
        bytes.extend_from_slice(&100u32.to_le_bytes());
        let result = deserialize_house_items(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn house_items_single_position_round_trip() {
        let positions = vec![Position::new(1, 2, 3)];
        let bytes = serialize_house_items(5, &positions);
        let (id, decoded) = deserialize_house_items(&bytes).unwrap();
        assert_eq!(id, 5);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].x, 1);
        assert_eq!(decoded[0].y, 2);
        assert_eq!(decoded[0].z, 3);
    }

    // -----------------------------------------------------------------------
    // encode_item / decode_item
    // -----------------------------------------------------------------------

    #[test]
    fn encode_item_simple_has_id_and_sentinel() {
        let item = simple_item(1234);
        let bytes = encode_item(&item);
        // id(2) + sentinel(1) = 3 bytes minimum
        assert_eq!(bytes.len(), 3);
        assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), 1234);
        assert_eq!(bytes[2], ATTR_END);
    }

    #[test]
    fn encode_item_includes_attr_bytes() {
        let item = item_with_attrs(10, vec![0x05, 0x2A]); // 2 attr bytes
        let bytes = encode_item(&item);
        // id(2) + attr(2) + sentinel(1) = 5
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[2], 0x05);
        assert_eq!(bytes[3], 0x2A);
        assert_eq!(bytes[4], ATTR_END);
    }

    #[test]
    fn decode_item_reads_simple_item() {
        let encoded = encode_item(&simple_item(42));
        let (item, pos) = decode_item(&encoded, 0).unwrap();
        assert_eq!(item.id, 42);
        assert!(item.attr_bytes.is_empty());
        assert!(item.children.is_empty());
        assert_eq!(pos, encoded.len());
    }

    #[test]
    fn encode_decode_item_round_trip() {
        let original = item_with_attrs(777, vec![0x01, 0xFF]);
        let bytes = encode_item(&original);
        let (decoded, _) = decode_item(&bytes, 0).unwrap();
        assert_eq!(decoded.id, 777);
        assert_eq!(decoded.attr_bytes, vec![0x01, 0xFF]);
        assert!(decoded.children.is_empty());
    }

    #[test]
    fn encode_item_container_includes_attr_container_items_marker() {
        let child = simple_item(5);
        let container = ItemRecord::with_children(100, vec![child]);
        let bytes = encode_item(&container);
        // Must contain ATTR_CONTAINER_ITEMS byte
        assert!(
            bytes.contains(&ATTR_CONTAINER_ITEMS),
            "encoded bytes should contain ATTR_CONTAINER_ITEMS (0x{:02X}): {:?}",
            ATTR_CONTAINER_ITEMS,
            bytes
        );
    }

    #[test]
    fn encode_decode_container_item_round_trip() {
        let child1 = simple_item(10);
        let child2 = simple_item(20);
        let container = ItemRecord::with_children(50, vec![child1.clone(), child2.clone()]);
        let bytes = encode_item(&container);
        let (decoded, _) = decode_item(&bytes, 0).unwrap();
        assert_eq!(decoded.id, 50);
        assert_eq!(decoded.children.len(), 2);
        // Order should be preserved after encode(reverse) + decode(reverse)
        assert_eq!(decoded.children[0].id, 10);
        assert_eq!(decoded.children[1].id, 20);
    }

    #[test]
    fn decode_item_error_on_too_short_data() {
        let result = decode_item(&[0x05], 0);
        assert!(result.is_err());
    }

    #[test]
    fn decode_item_error_on_missing_sentinel() {
        // id only, no sentinel
        let result = decode_item(&[0x01, 0x00], 0);
        assert!(result.is_err());
    }

    #[test]
    fn encode_item_deeply_nested_container_round_trip() {
        let leaf = simple_item(1);
        let inner = ItemRecord::with_children(2, vec![leaf]);
        let outer = ItemRecord::with_children(3, vec![inner]);
        let bytes = encode_item(&outer);
        let (decoded, _) = decode_item(&bytes, 0).unwrap();
        assert_eq!(decoded.id, 3);
        assert_eq!(decoded.children.len(), 1);
        assert_eq!(decoded.children[0].id, 2);
        assert_eq!(decoded.children[0].children.len(), 1);
        assert_eq!(decoded.children[0].children[0].id, 1);
    }

    // -----------------------------------------------------------------------
    // encode_tile / decode_tile
    // -----------------------------------------------------------------------

    #[test]
    fn encode_tile_no_items_has_correct_length() {
        let tile = simple_tile(10, 20, 7, vec![]);
        let bytes = encode_tile(&tile);
        // x(2) + y(2) + z(1) + count(4) = 9
        assert_eq!(bytes.len(), 9);
    }

    #[test]
    fn encode_tile_includes_position() {
        let tile = simple_tile(300, 400, 5, vec![]);
        let bytes = encode_tile(&tile);
        assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), 300);
        assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), 400);
        assert_eq!(bytes[4], 5);
    }

    #[test]
    fn decode_tile_reads_position() {
        let tile = simple_tile(300, 400, 5, vec![]);
        let bytes = encode_tile(&tile);
        let (decoded, _) = decode_tile(&bytes, 0).unwrap();
        assert_eq!(decoded.x, 300);
        assert_eq!(decoded.y, 400);
        assert_eq!(decoded.z, 5);
    }

    #[test]
    fn encode_decode_tile_round_trip_with_items() {
        let items = vec![simple_item(100), simple_item(200)];
        let tile = simple_tile(10, 20, 7, items.clone());
        let bytes = encode_tile(&tile);
        let (decoded, _) = decode_tile(&bytes, 0).unwrap();
        assert_eq!(decoded.x, 10);
        assert_eq!(decoded.y, 20);
        assert_eq!(decoded.z, 7);
        assert_eq!(decoded.items.len(), 2);
        assert_eq!(decoded.items[0].id, 100);
        assert_eq!(decoded.items[1].id, 200);
    }

    #[test]
    fn decode_tile_error_on_too_short_data() {
        let result = decode_tile(&[0x01, 0x00, 0x01, 0x00], 0);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // serialize_house_tiles / deserialize_house_tiles
    // -----------------------------------------------------------------------

    #[test]
    fn house_tiles_round_trip_empty() {
        let house_id = 10u32;
        let bytes = serialize_house_tiles(house_id, &[]);
        let (decoded_id, tiles) = deserialize_house_tiles(&bytes).unwrap();
        assert_eq!(decoded_id, house_id);
        assert!(tiles.is_empty());
    }

    #[test]
    fn house_tiles_round_trip_single_tile_no_items() {
        let tile = simple_tile(100, 200, 7, vec![]);
        let bytes = serialize_house_tiles(1, std::slice::from_ref(&tile));
        let (house_id, tiles) = deserialize_house_tiles(&bytes).unwrap();
        assert_eq!(house_id, 1);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].x, 100);
        assert_eq!(tiles[0].y, 200);
        assert_eq!(tiles[0].z, 7);
    }

    #[test]
    fn house_tiles_round_trip_with_items() {
        let items = vec![simple_item(42), simple_item(99)];
        let tile = simple_tile(10, 20, 7, items);
        let bytes = serialize_house_tiles(5, &[tile]);
        let (_, tiles) = deserialize_house_tiles(&bytes).unwrap();
        assert_eq!(tiles[0].items.len(), 2);
        assert_eq!(tiles[0].items[0].id, 42);
        assert_eq!(tiles[0].items[1].id, 99);
    }

    #[test]
    fn house_tiles_round_trip_multiple_tiles() {
        let t1 = simple_tile(1, 1, 7, vec![simple_item(10)]);
        let t2 = simple_tile(2, 1, 7, vec![simple_item(20), simple_item(30)]);
        let bytes = serialize_house_tiles(3, &[t1, t2]);
        let (house_id, tiles) = deserialize_house_tiles(&bytes).unwrap();
        assert_eq!(house_id, 3);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0].items.len(), 1);
        assert_eq!(tiles[1].items.len(), 2);
    }

    #[test]
    fn house_tiles_round_trip_with_container_item() {
        let child = simple_item(5);
        let container = ItemRecord::with_children(100, vec![child]);
        let tile = simple_tile(10, 10, 7, vec![container]);
        let bytes = serialize_house_tiles(1, &[tile]);
        let (_, tiles) = deserialize_house_tiles(&bytes).unwrap();
        assert_eq!(tiles[0].items[0].id, 100);
        assert_eq!(tiles[0].items[0].children.len(), 1);
        assert_eq!(tiles[0].items[0].children[0].id, 5);
    }

    #[test]
    fn house_tiles_error_on_too_short_payload() {
        let result = deserialize_house_tiles(&[0x01, 0x00]);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // House state serialization
    // -----------------------------------------------------------------------

    #[test]
    fn serialize_house_state_produces_correct_length() {
        let bytes = serialize_house_state(10, 555, 0, 0, "Grand Villa");
        // 4 + 4 + 4 + 4 + 2 + 11 = 29
        assert_eq!(bytes.len(), 29);
    }

    #[test]
    fn deserialize_house_state_reads_house_id() {
        let bytes = serialize_house_state(77, 0, 0, 0, "");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.house_id, 77);
    }

    #[test]
    fn deserialize_house_state_reads_owner_guid() {
        let bytes = serialize_house_state(1, 12345, 0, 0, "");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.owner_guid, 12345);
    }

    #[test]
    fn deserialize_house_state_reads_name() {
        let bytes = serialize_house_state(1, 0, 0, 0, "The Old Farm");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.name, "The Old Farm");
    }

    #[test]
    fn deserialize_house_state_reads_paid_until() {
        let paid_until: u32 = 1_700_000_000;
        let bytes = serialize_house_state(1, 0, paid_until, 0, "");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.paid_until, paid_until);
    }

    #[test]
    fn deserialize_house_state_reads_pay_rent_warnings() {
        let bytes = serialize_house_state(1, 0, 0, 3, "");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.pay_rent_warnings, 3);
    }

    #[test]
    fn house_state_round_trip() {
        let house_id = 42u32;
        let owner_guid = 9999u32;
        let paid_until = 1_700_000_000u32;
        let pay_rent_warnings = 2u32;
        let name = "Dragon's Lair";
        let bytes =
            serialize_house_state(house_id, owner_guid, paid_until, pay_rent_warnings, name);
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(
            state,
            HouseState {
                house_id,
                owner_guid,
                paid_until,
                pay_rent_warnings,
                name: name.to_owned()
            }
        );
    }

    #[test]
    fn house_state_round_trip_empty_name() {
        let bytes = serialize_house_state(1, 0, 0, 0, "");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.house_id, 1);
        assert_eq!(state.owner_guid, 0);
        assert!(state.name.is_empty());
    }

    #[test]
    fn deserialize_house_state_error_on_too_short_payload() {
        let result = deserialize_house_state(&[0x01]);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_house_state_error_when_name_exceeds_payload() {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&1u32.to_le_bytes()); // house_id
        bytes.extend_from_slice(&0u32.to_le_bytes()); // owner_guid
        bytes.extend_from_slice(&0u32.to_le_bytes()); // paid_until
        bytes.extend_from_slice(&0u32.to_le_bytes()); // pay_rent_warnings
        bytes.extend_from_slice(&100u16.to_le_bytes()); // name_len = 100
                                                        // No actual name bytes
        let result = deserialize_house_state(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn house_state_unicode_name_round_trip() {
        let name = "Bürgerhof";
        let bytes = serialize_house_state(3, 100, 0, 0, name);
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.name, name);
    }

    #[test]
    fn house_state_zero_paid_until_and_warnings() {
        let bytes = serialize_house_state(5, 42, 0, 0, "Empty");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.paid_until, 0);
        assert_eq!(state.pay_rent_warnings, 0);
    }

    #[test]
    fn house_state_max_warnings() {
        let bytes = serialize_house_state(1, 0, 0, u32::MAX, "");
        let state = deserialize_house_state(&bytes).unwrap();
        assert_eq!(state.pay_rent_warnings, u32::MAX);
    }

    // -----------------------------------------------------------------------
    // Tile item list preservation after round-trip (task 9.4 requirement)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Error-path coverage for nested decode error mappings
    // (mirror C++ "Unserialization error" logging in loadContainer / loadItem)
    // -----------------------------------------------------------------------

    #[test]
    fn decode_item_error_when_container_count_bytes_truncated() {
        // Hand-craft: id(2) + ATTR_CONTAINER_ITEMS(1) + only 2 bytes (need 4)
        // The C++ counterpart in loadContainer/loadItem fails here when the
        // PropStream has insufficient bytes for the u32 child count.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&123u16.to_le_bytes());
        bytes.push(ATTR_CONTAINER_ITEMS);
        bytes.push(0x00);
        bytes.push(0x00);
        // intentionally no more bytes — count truncated
        let result = decode_item(&bytes, 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("too short for container count"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn decode_tile_error_propagates_item_failure_with_index_and_pos() {
        // Tile header advertises 1 item, then provides bytes that fail to
        // decode as an item (only 1 byte — not enough for u16 id).
        // This exercises the closure that wraps the item-decode error with
        // tile context (lines 251-252 in the source).
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&10u16.to_le_bytes()); // x
        bytes.extend_from_slice(&20u16.to_le_bytes()); // y
        bytes.push(7); // z
        bytes.extend_from_slice(&1u32.to_le_bytes()); // item_count = 1
        bytes.push(0x00); // only 1 byte of item data
        let result = decode_tile(&bytes, 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("decode_tile"), "unexpected error: {msg}");
        assert!(
            msg.contains("error decoding item"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn deserialize_house_state_error_on_invalid_utf8_name() {
        // Construct a valid house-state payload but write invalid UTF-8 bytes
        // in the name section. Hits the from_utf8 error closure (line 423).
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_le_bytes()); // house_id
        bytes.extend_from_slice(&0u32.to_le_bytes()); // owner_guid
        bytes.extend_from_slice(&0u32.to_le_bytes()); // paid_until
        bytes.extend_from_slice(&0u32.to_le_bytes()); // pay_rent_warnings
        bytes.extend_from_slice(&3u16.to_le_bytes()); // name_len = 3
        bytes.extend_from_slice(&[0xC3, 0x28, 0xA0]); // invalid UTF-8 sequence
        let result = deserialize_house_state(&bytes);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("invalid UTF-8"), "unexpected error: {msg}");
    }

    #[test]
    fn deserialize_house_tiles_error_propagates_tile_failure_with_index() {
        // Payload: house_id(4) + tile_count=1 (4) + tile header bytes too short
        // (need 9, give 3). This exercises the closure on lines 359-360 that
        // wraps the tile-decode error with house-level context.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&5u32.to_le_bytes()); // house_id
        bytes.extend_from_slice(&1u32.to_le_bytes()); // tile_count = 1
        bytes.extend_from_slice(&[0x01, 0x00, 0x02]); // truncated tile header
        let result = deserialize_house_tiles(&bytes);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("deserialize_house_tiles"),
            "unexpected error: {msg}"
        );
        assert!(
            msg.contains("error decoding tile"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn tile_item_list_preservation_after_round_trip() {
        let items = vec![
            item_with_attrs(100, vec![0x05]),
            simple_item(200),
            ItemRecord::with_children(300, vec![simple_item(301), simple_item(302)]),
        ];
        let tile = simple_tile(50, 60, 7, items.clone());
        let bytes = serialize_house_tiles(99, &[tile]);
        let (_, tiles) = deserialize_house_tiles(&bytes).unwrap();

        assert_eq!(tiles[0].items.len(), 3);
        assert_eq!(tiles[0].items[0].id, 100);
        assert_eq!(tiles[0].items[0].attr_bytes, vec![0x05]);
        assert_eq!(tiles[0].items[1].id, 200);
        assert_eq!(tiles[0].items[2].id, 300);
        assert_eq!(tiles[0].items[2].children.len(), 2);
        assert_eq!(tiles[0].items[2].children[0].id, 301);
        assert_eq!(tiles[0].items[2].children[1].id, 302);
    }
}
