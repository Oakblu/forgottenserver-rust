// Migrated from forgottenserver/src/iomap.h + iomap.cpp
//
// OTBM (Open Tibia Binary Map) parser.
//
// The format consists of:
//   - 4-byte magic: `\x00 OTB` = `0x00 0x4D 0x42 0x4F`
//   - An OTB node tree (same NODE_START/NODE_END/ESCAPE bytes as OTB item files)
//   - Root node (type 0x00) carries a root-header attribute (attr type 0x01)
//     that contains: version(u32), width(u16), height(u16),
//     majorVersionItems(u32), minorVersionItems(u32)
//
// After the root-header the remaining children of OTBM_MAP_DATA are:
//   - OTBM_TILE_AREA  → contains OTBM_TILE / OTBM_HOUSETILE children
//   - OTBM_TOWNS      → town nodes (parsed but not stored in Map for now)
//   - OTBM_WAYPOINTS  → named waypoints registered into map.waypoints

use std::collections::HashMap;

use forgottenserver_common::position::Position;

use crate::map::Map;

// ---------------------------------------------------------------------------
// OTBM node-type constants (from iomap.h)
// ---------------------------------------------------------------------------

const OTBM_MAP_DATA: u8 = 2;
const OTBM_TILE_AREA: u8 = 4;
const OTBM_TILE: u8 = 5;
const OTBM_TOWNS: u8 = 12;
const OTBM_HOUSETILE: u8 = 14;
const OTBM_WAYPOINTS: u8 = 15;
const OTBM_WAYPOINT: u8 = 16;

// ---------------------------------------------------------------------------
// OTBM attribute-type constants (from iomap.h)
// ---------------------------------------------------------------------------

const OTBM_ATTR_DESCRIPTION: u8 = 1;
const OTBM_ATTR_EXT_SPAWN_FILE: u8 = 11;
const OTBM_ATTR_EXT_HOUSE_FILE: u8 = 13;
const OTBM_ATTR_TILE_FLAGS: u8 = 3;
const OTBM_ATTR_ITEM: u8 = 9;

// ---------------------------------------------------------------------------
// OTBM tile flags (from iomap.h — OTBM_TileFlag_t)
// ---------------------------------------------------------------------------

const OTBM_TILEFLAG_PROTECTIONZONE: u32 = 1 << 0;
const OTBM_TILEFLAG_NOPVPZONE: u32 = 1 << 2;
const OTBM_TILEFLAG_NOLOGOUT: u32 = 1 << 3;
const OTBM_TILEFLAG_PVPZONE: u32 = 1 << 4;

// ---------------------------------------------------------------------------
// Low-level binary constants
// ---------------------------------------------------------------------------

const OTBM_MAGIC: [u8; 4] = [0x00, 0x4D, 0x42, 0x4F];

const NODE_START: u8 = 0xFE;
const NODE_END: u8 = 0xFF;
const ESCAPE: u8 = 0xFD;

const OTBM_ATTR_MAP_VERSION: u8 = 0x01;

// ---------------------------------------------------------------------------
// Tile flags in our domain representation
// ---------------------------------------------------------------------------

/// Bit-flags set on a tile, corresponding to the C++ `tileflags_t` values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TileFlags {
    pub protection_zone: bool,
    pub no_pvp_zone: bool,
    pub no_logout: bool,
    pub pvp_zone: bool,
}

impl TileFlags {
    fn from_otbm(flags: u32) -> Self {
        TileFlags {
            protection_zone: (flags & OTBM_TILEFLAG_PROTECTIONZONE) != 0,
            no_pvp_zone: (flags & OTBM_TILEFLAG_NOPVPZONE) != 0,
            no_logout: (flags & OTBM_TILEFLAG_NOLOGOUT) != 0,
            pvp_zone: (flags & OTBM_TILEFLAG_PVPZONE) != 0,
        }
    }
}

// ---------------------------------------------------------------------------
// MapHeader — data decoded from the root node attribute
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapHeader {
    pub version: u32,
    pub width: u16,
    pub height: u16,
    pub major_version_items: u32,
    pub minor_version_items: u32,
}

// ---------------------------------------------------------------------------
// LoadedTile — a tile extracted by the parser (returned for inspection)
// ---------------------------------------------------------------------------

/// Represents a tile extracted during OTBM parsing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoadedTile {
    pub x: u16,
    pub y: u16,
    pub z: u8,
    pub flags: TileFlags,
    pub house_id: Option<u32>,
    /// Item IDs placed on this tile (from OTBM_ATTR_ITEM attributes).
    pub item_ids: Vec<u16>,
}

// ---------------------------------------------------------------------------
// ParsedMap — the full result of parsing an OTBM stream
// ---------------------------------------------------------------------------

/// Full result of parsing an OTBM binary buffer.
#[derive(Debug, Default)]
pub struct ParsedMap {
    pub header: Option<MapHeader>,
    pub tiles: Vec<LoadedTile>,
    /// Named waypoints: name → position.
    pub waypoints: HashMap<String, Position>,
    /// Town names keyed by their ID.
    pub town_names: HashMap<u32, String>,
    /// Spawn file path embedded in map-data attributes.
    pub spawn_file: Option<String>,
    /// House file path embedded in map-data attributes.
    pub house_file: Option<String>,
}

// ---------------------------------------------------------------------------
// IoMap
// ---------------------------------------------------------------------------

pub struct IoMap;

impl IoMap {
    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Parses `bytes` as OTBM binary data and returns a `Map` with its
    /// dimensions populated from the root-node header.
    ///
    /// # Errors
    /// Returns a `String` describing the parse failure.
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Map, String> {
        let header = Self::parse_header(bytes)?;

        let mut map = Map::new();
        map.set_declared_width(header.width);
        map.set_declared_height(header.height);
        Ok(map)
    }

    /// Extracts just the map version from the root-node header of `bytes`.
    pub fn parse_map_version(bytes: &[u8]) -> Result<u32, String> {
        let header = Self::parse_header(bytes)?;
        Ok(header.version)
    }

    /// Full parse: returns a `ParsedMap` with tile areas, waypoints, towns,
    /// and map-data attributes extracted from the entire OTBM tree.
    ///
    /// This mirrors the C++ `IOMap::loadMap` behaviour at the structural level.
    pub fn parse_full(bytes: &[u8]) -> Result<ParsedMap, String> {
        // ----------------------------------------------------------------
        // 1. Magic
        // ----------------------------------------------------------------
        if bytes.len() < 4 {
            return Err("OTBM: file too small (no magic bytes)".into());
        }
        if bytes[..4] != OTBM_MAGIC {
            return Err(format!(
                "OTBM: invalid magic bytes {:02X?} (expected {:02X?})",
                &bytes[..4],
                OTBM_MAGIC
            ));
        }

        // ----------------------------------------------------------------
        // 2. Root NODE_START
        // ----------------------------------------------------------------
        if bytes.len() < 5 || bytes[4] != NODE_START {
            return Err("OTBM: expected NODE_START after magic".into());
        }

        // ----------------------------------------------------------------
        // 3. Root node type byte (index 5; we accept any value like C++ does)
        // ----------------------------------------------------------------
        if bytes.len() < 6 {
            return Err("OTBM: missing root node type byte".into());
        }

        // ----------------------------------------------------------------
        // 4. Root node attribute → MapHeader
        // ----------------------------------------------------------------
        let (root_props, mut cursor) = Self::collect_props_with_end(bytes, 6)?;

        if root_props.is_empty() || root_props[0] != OTBM_ATTR_MAP_VERSION {
            return Err("OTBM: root node missing map-version attribute".into());
        }
        let data = &root_props[1..];
        if data.len() < 16 {
            return Err(format!(
                "OTBM: root node attribute too short ({} bytes, need 16)",
                data.len()
            ));
        }

        let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let width = u16::from_le_bytes([data[4], data[5]]);
        let height = u16::from_le_bytes([data[6], data[7]]);
        let major_version_items = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let minor_version_items = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        let header = MapHeader {
            version,
            width,
            height,
            major_version_items,
            minor_version_items,
        };

        // ----------------------------------------------------------------
        // 5. Version validation (mirrors C++ loadMap checks)
        // ----------------------------------------------------------------
        if version == 0 {
            return Err(
                "OTBM: map version 0 is not supported (upgrade with latest map editor)".into(),
            );
        }
        if version > 2 {
            return Err("OTBM: unknown OTBM version detected".into());
        }

        let mut result = ParsedMap {
            header: Some(header),
            ..Default::default()
        };

        // ----------------------------------------------------------------
        // 6. Expect child node: OTBM_MAP_DATA
        // ----------------------------------------------------------------
        if cursor >= bytes.len() || bytes[cursor] != NODE_START {
            // No child node present — valid minimal map
            return Ok(result);
        }
        cursor += 1; // consume NODE_START

        if cursor >= bytes.len() {
            return Err("OTBM: expected OTBM_MAP_DATA type byte".into());
        }
        let map_data_type = bytes[cursor];
        cursor += 1;

        if map_data_type != OTBM_MAP_DATA {
            return Err(format!(
                "OTBM: expected OTBM_MAP_DATA node (type 2), got {}",
                map_data_type
            ));
        }

        // Map-data attributes (description, spawn file, house file)
        let (map_attrs, c2) = Self::collect_props_with_end(bytes, cursor)?;
        cursor = c2;
        Self::parse_map_data_attributes(&map_attrs, &mut result)?;

        // ----------------------------------------------------------------
        // 7. Map-data child nodes
        //
        // After `collect_props_with_end` the cursor sits on NODE_START or
        // NODE_END; we don't need a catch-all arm.
        // ----------------------------------------------------------------
        while cursor < bytes.len() && bytes[cursor] != NODE_END {
            cursor += 1; // consume NODE_START
            if cursor >= bytes.len() {
                return Err("OTBM: unexpected end after NODE_START in map-data".into());
            }
            let child_type = bytes[cursor];
            cursor += 1;

            match child_type {
                OTBM_TILE_AREA => {
                    cursor = Self::parse_tile_area(bytes, cursor, &mut result)?;
                }
                OTBM_TOWNS => {
                    cursor = Self::parse_towns(bytes, cursor, &mut result)?;
                }
                OTBM_WAYPOINTS => {
                    cursor = Self::parse_waypoints(bytes, cursor, &mut result)?;
                }
                _ => {
                    return Err(format!(
                        "OTBM: unknown map-data child node type {}",
                        child_type
                    ));
                }
            }
        }

        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn parse_header(bytes: &[u8]) -> Result<MapHeader, String> {
        // Check magic
        if bytes.len() < 4 {
            return Err("OTBM: file too small (no magic bytes)".into());
        }
        if bytes[..4] != OTBM_MAGIC {
            return Err(format!(
                "OTBM: invalid magic bytes {:02X?} (expected {:02X?})",
                &bytes[..4],
                OTBM_MAGIC
            ));
        }

        // Expect NODE_START at byte 4
        if bytes.len() < 5 || bytes[4] != NODE_START {
            return Err("OTBM: expected NODE_START after magic".into());
        }

        // Root node type byte
        if bytes.len() < 6 {
            return Err("OTBM: missing root node type byte".into());
        }

        // Read props from the root node (unescaped)
        let props = Self::collect_props(bytes, 6)?;

        // Props must contain: attr_type(1) + version(4) + width(2) + height(2)
        //                     + majorVersionItems(4) + minorVersionItems(4) = 17 bytes minimum
        if props.is_empty() || props[0] != OTBM_ATTR_MAP_VERSION {
            return Err("OTBM: root node missing map-version attribute".into());
        }

        // Remaining bytes after attr-type byte
        let data = &props[1..];
        if data.len() < 16 {
            return Err(format!(
                "OTBM: root node attribute too short ({} bytes, need 16)",
                data.len()
            ));
        }

        let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let width = u16::from_le_bytes([data[4], data[5]]);
        let height = u16::from_le_bytes([data[6], data[7]]);
        let major_version_items = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let minor_version_items = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);

        Ok(MapHeader {
            version,
            width,
            height,
            major_version_items,
            minor_version_items,
        })
    }

    /// Collects unescaped prop bytes for the current node, starting at `start`
    /// inside `bytes`.  Stops (without consuming) at the first NODE_START or
    /// NODE_END byte it encounters.
    fn collect_props(bytes: &[u8], start: usize) -> Result<Vec<u8>, String> {
        let (props, _) = Self::collect_props_with_end(bytes, start)?;
        Ok(props)
    }

    /// Like `collect_props` but also returns the cursor position after the
    /// last byte consumed (i.e. pointing at the first NODE_START/NODE_END).
    fn collect_props_with_end(bytes: &[u8], start: usize) -> Result<(Vec<u8>, usize), String> {
        let mut pos = start;
        let mut props = Vec::new();

        while pos < bytes.len() {
            match bytes[pos] {
                NODE_START | NODE_END => break,
                ESCAPE => {
                    pos += 1;
                    if pos >= bytes.len() {
                        return Err("OTBM: unexpected end after ESCAPE byte".into());
                    }
                    props.push(bytes[pos]);
                    pos += 1;
                }
                b => {
                    props.push(b);
                    pos += 1;
                }
            }
        }

        Ok((props, pos))
    }

    // -----------------------------------------------------------------------
    // Map-data attribute parser
    // -----------------------------------------------------------------------

    fn parse_map_data_attributes(props: &[u8], result: &mut ParsedMap) -> Result<(), String> {
        let mut pos = 0usize;
        while pos < props.len() {
            let attr = props[pos];
            pos += 1;
            match attr {
                OTBM_ATTR_DESCRIPTION => {
                    let (s, end) = Self::read_string(props, pos)?;
                    let _ = s; // description is stored nowhere (like C++)
                    pos = end;
                }
                OTBM_ATTR_EXT_SPAWN_FILE => {
                    let (s, end) = Self::read_string(props, pos)?;
                    result.spawn_file = Some(s);
                    pos = end;
                }
                OTBM_ATTR_EXT_HOUSE_FILE => {
                    let (s, end) = Self::read_string(props, pos)?;
                    result.house_file = Some(s);
                    pos = end;
                }
                _ => {
                    return Err(format!("OTBM: unknown map-data attribute type {}", attr));
                }
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Tile-area parser
    // -----------------------------------------------------------------------

    /// Parses an OTBM_TILE_AREA block.  `cursor` points at the first byte
    /// of the tile-area's props (the x coord of the base position).
    ///
    /// Returns the updated cursor after the tile area's NODE_END.
    fn parse_tile_area(
        bytes: &[u8],
        cursor: usize,
        result: &mut ParsedMap,
    ) -> Result<usize, String> {
        let (area_props, mut c) = Self::collect_props_with_end(bytes, cursor)?;

        // Area props: base_x(u16) + base_y(u16) + z(u8) = 5 bytes
        if area_props.len() < 5 {
            return Err(format!(
                "OTBM: tile-area props too short ({} bytes, need 5)",
                area_props.len()
            ));
        }
        let base_x = u16::from_le_bytes([area_props[0], area_props[1]]);
        let base_y = u16::from_le_bytes([area_props[2], area_props[3]]);
        let z = area_props[4];

        // Iterate child nodes (tiles).
        // After `collect_props_with_end` returns, `c` always points at
        // NODE_START or NODE_END (or past end-of-buffer).
        while c < bytes.len() && bytes[c] != NODE_END {
            c += 1; // consume NODE_START
            if c >= bytes.len() {
                return Err("OTBM: unexpected end after NODE_START in tile-area".into());
            }
            let tile_type = bytes[c];
            c += 1;

            if tile_type != OTBM_TILE && tile_type != OTBM_HOUSETILE {
                return Err(format!(
                    "OTBM: unknown tile node type {} in tile-area",
                    tile_type
                ));
            }

            let (tile_props, c2) = Self::collect_props_with_end(bytes, c)?;
            c = c2;

            let mut prop_pos = 0usize;

            // Tile coords: dx(u8) + dy(u8) = 2 bytes
            if tile_props.len() < 2 {
                return Err("OTBM: tile props too short (need 2 bytes for coords)".into());
            }
            let x = base_x + tile_props[0] as u16;
            let y = base_y + tile_props[1] as u16;
            prop_pos += 2;

            let mut tile = LoadedTile {
                x,
                y,
                z,
                ..Default::default()
            };

            // For house tiles read house_id (u32)
            if tile_type == OTBM_HOUSETILE {
                if tile_props.len() < prop_pos + 4 {
                    return Err(format!(
                        "OTBM: house tile [{},{},{}] too short to read house_id",
                        x, y, z
                    ));
                }
                let house_id = u32::from_le_bytes([
                    tile_props[prop_pos],
                    tile_props[prop_pos + 1],
                    tile_props[prop_pos + 2],
                    tile_props[prop_pos + 3],
                ]);
                tile.house_id = Some(house_id);
                prop_pos += 4;
            }

            // Read tile attributes
            while prop_pos < tile_props.len() {
                let attr = tile_props[prop_pos];
                prop_pos += 1;
                match attr {
                    OTBM_ATTR_TILE_FLAGS => {
                        if tile_props.len() < prop_pos + 4 {
                            return Err(format!(
                                "OTBM: tile [{},{},{}] too short for tile flags",
                                x, y, z
                            ));
                        }
                        let flags = u32::from_le_bytes([
                            tile_props[prop_pos],
                            tile_props[prop_pos + 1],
                            tile_props[prop_pos + 2],
                            tile_props[prop_pos + 3],
                        ]);
                        tile.flags = TileFlags::from_otbm(flags);
                        prop_pos += 4;
                    }
                    OTBM_ATTR_ITEM => {
                        // item_id u16
                        if tile_props.len() < prop_pos + 2 {
                            return Err(format!(
                                "OTBM: tile [{},{},{}] too short for item attr",
                                x, y, z
                            ));
                        }
                        let item_id =
                            u16::from_le_bytes([tile_props[prop_pos], tile_props[prop_pos + 1]]);
                        tile.item_ids.push(item_id);
                        prop_pos += 2;
                    }
                    _ => {
                        return Err(format!(
                            "OTBM: tile [{},{},{}] unknown attribute {}",
                            x, y, z, attr
                        ));
                    }
                }
            }

            // Consume tile child nodes (OTBM_ITEM children — we skip
            // their content for now but we must advance the cursor past them)
            c = Self::skip_children(bytes, c)?;

            result.tiles.push(tile);
        }

        // Skip closing NODE_END of the tile-area
        if c < bytes.len() && bytes[c] == NODE_END {
            c += 1;
        }

        Ok(c)
    }

    // -----------------------------------------------------------------------
    // Towns parser
    // -----------------------------------------------------------------------

    fn parse_towns(bytes: &[u8], cursor: usize, result: &mut ParsedMap) -> Result<usize, String> {
        // Skip towns props (should be empty)
        let (_, mut c) = Self::collect_props_with_end(bytes, cursor)?;

        // After `collect_props_with_end` returns, `c` always points at
        // NODE_START or NODE_END (or past end-of-buffer).
        while c < bytes.len() && bytes[c] != NODE_END {
            c += 1;
            if c >= bytes.len() {
                return Err("OTBM: unexpected end in towns".into());
            }
            // node type (should be OTBM_TOWN = 13, but we are lenient)
            c += 1;

            let (town_props, c2) = Self::collect_props_with_end(bytes, c)?;
            c = c2;

            // town_id(u32) + name(string) + coords(u16+u16+u8)
            if town_props.len() < 4 {
                return Err("OTBM: town props too short for town_id".into());
            }
            let town_id =
                u32::from_le_bytes([town_props[0], town_props[1], town_props[2], town_props[3]]);
            let (name, name_end) = Self::read_string(&town_props, 4)?;
            // After name: 5 bytes for coords (u16+u16+u8)
            let _ = name_end; // coords not stored in ParsedMap for now

            result.town_names.insert(town_id, name);

            // Consume any children
            c = Self::skip_children(bytes, c)?;
        }

        // Skip closing NODE_END of the towns node
        if c < bytes.len() && bytes[c] == NODE_END {
            c += 1;
        }

        Ok(c)
    }

    // -----------------------------------------------------------------------
    // Waypoints parser
    // -----------------------------------------------------------------------

    fn parse_waypoints(
        bytes: &[u8],
        cursor: usize,
        result: &mut ParsedMap,
    ) -> Result<usize, String> {
        let (_, mut c) = Self::collect_props_with_end(bytes, cursor)?;

        // After `collect_props_with_end` returns, `c` always points at
        // NODE_START or NODE_END (or past end-of-buffer).
        while c < bytes.len() && bytes[c] != NODE_END {
            c += 1;
            if c >= bytes.len() {
                return Err("OTBM: unexpected end in waypoints".into());
            }
            let node_type = bytes[c];
            c += 1;

            if node_type != OTBM_WAYPOINT {
                return Err(format!(
                    "OTBM: expected OTBM_WAYPOINT node (16), got {}",
                    node_type
                ));
            }

            let (wp_props, c2) = Self::collect_props_with_end(bytes, c)?;
            c = c2;

            // name (string) + coords u16+u16+u8 = 5 bytes
            let (name, name_end) = Self::read_string(&wp_props, 0)?;

            let coord_start = name_end;
            if wp_props.len() < coord_start + 5 {
                return Err(format!(
                    "OTBM: waypoint '{}' props too short for coords",
                    name
                ));
            }
            let wp_x = u16::from_le_bytes([wp_props[coord_start], wp_props[coord_start + 1]]);
            let wp_y = u16::from_le_bytes([wp_props[coord_start + 2], wp_props[coord_start + 3]]);
            let wp_z = wp_props[coord_start + 4];

            result
                .waypoints
                .insert(name, Position::new(wp_x, wp_y, wp_z));

            c = Self::skip_children(bytes, c)?;
        }

        // Skip closing NODE_END of the waypoints node
        if c < bytes.len() && bytes[c] == NODE_END {
            c += 1;
        }

        Ok(c)
    }

    // -----------------------------------------------------------------------
    // Generic child-node skipper
    // -----------------------------------------------------------------------

    /// Advances `cursor` past all child nodes of the current node, stopping
    /// when it encounters the NODE_END that closes the parent.  Returns the
    /// cursor position after that NODE_END.
    ///
    /// This is used when we need to skip over sub-tree content we don't parse.
    fn skip_children(bytes: &[u8], cursor: usize) -> Result<usize, String> {
        let mut c = cursor;
        let mut depth = 0i32;

        while c < bytes.len() {
            match bytes[c] {
                NODE_END if depth == 0 => {
                    // This is the NODE_END that closes the parent we started in
                    c += 1;
                    return Ok(c);
                }
                NODE_END => {
                    depth -= 1;
                    c += 1;
                }
                NODE_START => {
                    depth += 1;
                    c += 1;
                    // skip type byte
                    if c < bytes.len() {
                        c += 1;
                    }
                }
                ESCAPE => {
                    c += 2; // escape + escaped byte
                }
                _ => {
                    c += 1;
                }
            }
        }

        Err("OTBM: unexpected end while skipping children".into())
    }

    // -----------------------------------------------------------------------
    // String reader (u16-length-prefixed, as used in OTBM)
    // -----------------------------------------------------------------------

    fn read_string(data: &[u8], pos: usize) -> Result<(String, usize), String> {
        if data.len() < pos + 2 {
            return Err(format!(
                "OTBM: string too short (need 2-byte length at pos {})",
                pos
            ));
        }
        let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        let end = pos + 2 + len;
        if data.len() < end {
            return Err(format!(
                "OTBM: string body truncated (need {} bytes at pos {}, have {})",
                len,
                pos + 2,
                data.len()
            ));
        }
        let s = std::str::from_utf8(&data[pos + 2..end])
            .map_err(|e| format!("OTBM: invalid UTF-8 in string: {e}"))?
            .to_owned();
        Ok((s, end))
    }
}

// ---------------------------------------------------------------------------
// Map extension — declared dimensions from the OTBM header
// ---------------------------------------------------------------------------
//
// The sparse tile HashMap stores only tiles that have been explicitly set.
// The OTBM header contains *declared* dimensions that we keep separately so
// that `IoMap::load_from_bytes` can report them without needing to add any
// tiles.

impl Map {
    /// Sets the width declared in the OTBM header.
    pub fn set_declared_width(&mut self, w: u16) {
        self.declared_width = w;
    }

    /// Sets the height declared in the OTBM header.
    pub fn set_declared_height(&mut self, h: u16) {
        self.declared_height = h;
    }

    /// Returns the width declared in the OTBM header (0 if not set).
    pub fn get_declared_width(&self) -> u16 {
        self.declared_width
    }

    /// Returns the height declared in the OTBM header (0 if not set).
    pub fn get_declared_height(&self) -> u16 {
        self.declared_height
    }
}

// ---------------------------------------------------------------------------
// OTBM fixture builder (shared between tests below and external tests)
// ---------------------------------------------------------------------------

/// Builds the minimal valid OTBM header bytes (magic + root node) with the
/// given `version`, `width`, and `height`.  No map-data child node.
pub fn make_otbm_header(version: u32, width: u16, height: u16) -> Vec<u8> {
    let mut buf = vec![
        0x00, 0x4D, 0x42, 0x4F, // magic
        0xFE, // NODE_START
        0x00, // root node type
        0x01, // attr: map version
    ];
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&width.to_le_bytes());
    buf.extend_from_slice(&height.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // major
    buf.extend_from_slice(&0u32.to_le_bytes()); // minor
    buf.push(0xFF); // NODE_END
    buf
}

/// Encodes an OTBM string (u16 LE length + UTF-8 bytes).
pub fn encode_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut v = Vec::with_capacity(2 + bytes.len());
    v.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    v.extend_from_slice(bytes);
    v
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Minimal OTBM fixture helpers
    // -----------------------------------------------------------------------

    fn make_minimal_otbm() -> Vec<u8> {
        make_otbm_header(2, 1, 1)
    }

    fn make_otbm_with_dims(version: u32, width: u16, height: u16) -> Vec<u8> {
        make_otbm_header(version, width, height)
    }

    /// Builds a complete OTBM with:
    ///   - root header (version, width, height)
    ///   - one OTBM_MAP_DATA child (no attributes)
    ///   - one OTBM_TILE_AREA with one OTBM_TILE at (base_x + dx, base_y + dy, z)
    #[allow(clippy::too_many_arguments)]
    fn make_otbm_with_tile(
        version: u32,
        width: u16,
        height: u16,
        base_x: u16,
        base_y: u16,
        z: u8,
        dx: u8,
        dy: u8,
    ) -> Vec<u8> {
        // Root header
        let mut buf = vec![
            0x00, 0x4D, 0x42, 0x4F, // magic
            0xFE, // NODE_START (root)
            0x00, // root node type
            0x01, // attr: map version
        ];
        buf.extend_from_slice(&version.to_le_bytes());
        buf.extend_from_slice(&width.to_le_bytes());
        buf.extend_from_slice(&height.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes()); // major
        buf.extend_from_slice(&0u32.to_le_bytes()); // minor

        // OTBM_MAP_DATA child node
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        // No map-data attributes

        // OTBM_TILE_AREA child of MAP_DATA
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        // Area props: base_x(u16 LE) + base_y(u16 LE) + z(u8)
        buf.extend_from_slice(&base_x.to_le_bytes());
        buf.extend_from_slice(&base_y.to_le_bytes());
        buf.push(z);

        // One OTBM_TILE child
        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(dx); // tile x offset
        buf.push(dy); // tile y offset
        buf.push(NODE_END); // close tile

        buf.push(NODE_END); // close OTBM_TILE_AREA
        buf.push(NODE_END); // close OTBM_MAP_DATA
        buf.push(NODE_END); // close root

        buf
    }

    /// Builds OTBM with a house tile
    fn make_otbm_with_house_tile(house_id: u32) -> Vec<u8> {
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0x01];
        buf.extend_from_slice(&2u32.to_le_bytes()); // version
        buf.extend_from_slice(&100u16.to_le_bytes()); // width
        buf.extend_from_slice(&100u16.to_le_bytes()); // height
        buf.extend_from_slice(&0u32.to_le_bytes()); // major
        buf.extend_from_slice(&0u32.to_le_bytes()); // minor

        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);

        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&100u16.to_le_bytes()); // base_x
        buf.extend_from_slice(&100u16.to_le_bytes()); // base_y
        buf.push(7u8); // z

        buf.push(NODE_START);
        buf.push(OTBM_HOUSETILE);
        buf.push(0u8); // dx
        buf.push(0u8); // dy
                       // house_id u32 LE
        buf.extend_from_slice(&house_id.to_le_bytes());
        buf.push(NODE_END); // close house tile

        buf.push(NODE_END); // close TILE_AREA
        buf.push(NODE_END); // close MAP_DATA
        buf.push(NODE_END); // close root

        buf
    }

    /// Builds OTBM with a tile that has tile flags
    fn make_otbm_with_tile_flags(otbm_flags: u32) -> Vec<u8> {
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0x01];
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());

        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);

        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&10u16.to_le_bytes());
        buf.extend_from_slice(&10u16.to_le_bytes());
        buf.push(7u8);

        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(0u8); // dx
        buf.push(0u8); // dy
                       // OTBM_ATTR_TILE_FLAGS
        buf.push(OTBM_ATTR_TILE_FLAGS);
        buf.extend_from_slice(&otbm_flags.to_le_bytes());
        buf.push(NODE_END);

        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);

        buf
    }

    /// Builds OTBM with a waypoint
    fn make_otbm_with_waypoint(name: &str, x: u16, y: u16, z: u8) -> Vec<u8> {
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0x01];
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());

        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);

        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINTS);
        // no waypoints-level props

        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINT);
        // name string
        buf.extend_from_slice(&encode_string(name));
        // coords
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.push(z);
        buf.push(NODE_END); // close waypoint

        buf.push(NODE_END); // close WAYPOINTS
        buf.push(NODE_END); // close MAP_DATA
        buf.push(NODE_END); // close root

        buf
    }

    /// Builds OTBM with a tile that has an OTBM_ATTR_ITEM
    fn make_otbm_with_item_on_tile(item_id: u16) -> Vec<u8> {
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0x01];
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());

        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);

        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&50u16.to_le_bytes());
        buf.extend_from_slice(&50u16.to_le_bytes());
        buf.push(7u8);

        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(0u8); // dx
        buf.push(0u8); // dy
        buf.push(OTBM_ATTR_ITEM);
        buf.extend_from_slice(&item_id.to_le_bytes());
        buf.push(NODE_END);

        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);

        buf
    }

    /// Builds OTBM with map-data attributes (spawn file, house file)
    fn make_otbm_with_map_attrs(spawn_file: &str, house_file: &str) -> Vec<u8> {
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0x01];
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());

        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        // spawn file attr
        buf.push(OTBM_ATTR_EXT_SPAWN_FILE);
        buf.extend_from_slice(&encode_string(spawn_file));
        // house file attr
        buf.push(OTBM_ATTR_EXT_HOUSE_FILE);
        buf.extend_from_slice(&encode_string(house_file));

        buf.push(NODE_END); // close MAP_DATA
        buf.push(NODE_END); // close root

        buf
    }

    // -----------------------------------------------------------------------
    // load_from_bytes (existing)
    // -----------------------------------------------------------------------

    #[test]
    fn load_from_bytes_returns_ok_on_valid_minimal_otbm() {
        let bytes = make_minimal_otbm();
        let result = IoMap::load_from_bytes(&bytes);
        assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    }

    #[test]
    fn load_from_bytes_returns_err_on_invalid_magic() {
        let mut bytes = make_minimal_otbm();
        bytes[0] = 0xFF; // corrupt magic
        let result = IoMap::load_from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn load_from_bytes_returns_err_when_too_short() {
        let result = IoMap::load_from_bytes(&[0x00, 0x4D]);
        assert!(result.is_err());
    }

    #[test]
    fn load_from_bytes_returns_err_on_missing_node_start() {
        let mut bytes = make_minimal_otbm();
        bytes[4] = 0x00; // not NODE_START
        let result = IoMap::load_from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn load_from_bytes_map_has_declared_width_from_header() {
        let bytes = make_otbm_with_dims(2, 300, 150);
        let map = IoMap::load_from_bytes(&bytes).unwrap();
        assert_eq!(map.get_declared_width(), 300);
    }

    #[test]
    fn load_from_bytes_map_has_declared_height_from_header() {
        let bytes = make_otbm_with_dims(2, 300, 150);
        let map = IoMap::load_from_bytes(&bytes).unwrap();
        assert_eq!(map.get_declared_height(), 150);
    }

    // -----------------------------------------------------------------------
    // parse_map_version (existing)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_map_version_returns_version_from_fixture() {
        let bytes = make_minimal_otbm();
        let version = IoMap::parse_map_version(&bytes).unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn parse_map_version_returns_correct_version() {
        let bytes = make_otbm_with_dims(5, 100, 100);
        let version = IoMap::parse_map_version(&bytes).unwrap();
        assert_eq!(version, 5);
    }

    #[test]
    fn parse_map_version_returns_err_on_bad_magic() {
        let result = IoMap::parse_map_version(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // MapHeader parsing (existing)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_header_decodes_all_fields() {
        let bytes = make_otbm_with_dims(3, 1000, 500);
        let header = IoMap::parse_header(&bytes).unwrap();
        assert_eq!(header.version, 3);
        assert_eq!(header.width, 1000);
        assert_eq!(header.height, 500);
        assert_eq!(header.major_version_items, 0);
        assert_eq!(header.minor_version_items, 0);
    }

    #[test]
    fn parse_header_with_escaped_bytes_in_props() {
        // version = 0xFE = 254 (LE) — must escape the 0xFE byte
        let mut buf = vec![
            0x00, 0x4D, 0x42, 0x4F, 0xFE, // NODE_START
            0x00, // root node type
            0x01, // attr: map version
            0xFD, 0xFE, // escaped 0xFE
            0x00, 0x00, 0x00,
        ];
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.push(0xFF);

        let header = IoMap::parse_header(&buf).unwrap();
        assert_eq!(header.version, 0xFE);
        assert_eq!(header.width, 1);
        assert_eq!(header.height, 1);
    }

    // -----------------------------------------------------------------------
    // parse_full — version validation
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_rejects_version_zero() {
        let bytes = make_otbm_header(0, 100, 100);
        let result = IoMap::parse_full(&bytes);
        assert!(result.is_err(), "expected Err for version 0");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("version 0") || msg.contains("not supported"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_rejects_version_greater_than_2() {
        let bytes = make_otbm_header(3, 100, 100);
        let result = IoMap::parse_full(&bytes);
        assert!(result.is_err(), "expected Err for version 3");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("unknown") || msg.contains("version"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_accepts_version_1() {
        let bytes = make_otbm_header(1, 50, 50);
        let result = IoMap::parse_full(&bytes);
        assert!(
            result.is_ok(),
            "expected Ok for version 1, got {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_full_accepts_version_2() {
        let bytes = make_otbm_header(2, 50, 50);
        let result = IoMap::parse_full(&bytes);
        assert!(
            result.is_ok(),
            "expected Ok for version 2, got {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_full_header_stored_in_result() {
        let bytes = make_otbm_header(2, 800, 600);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        let h = parsed.header.as_ref().unwrap();
        assert_eq!(h.version, 2);
        assert_eq!(h.width, 800);
        assert_eq!(h.height, 600);
    }

    // -----------------------------------------------------------------------
    // parse_full — tile-area parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_no_tiles_when_no_tile_area() {
        let bytes = make_otbm_header(2, 100, 100);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles.is_empty());
    }

    #[test]
    fn parse_full_tile_area_produces_one_tile() {
        let bytes = make_otbm_with_tile(2, 100, 100, 10, 20, 7, 0, 0);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.tiles.len(), 1);
    }

    #[test]
    fn parse_full_tile_coords_are_base_plus_offset() {
        let bytes = make_otbm_with_tile(2, 100, 100, 10, 20, 7, 3, 5);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        let tile = &parsed.tiles[0];
        assert_eq!(tile.x, 13);
        assert_eq!(tile.y, 25);
        assert_eq!(tile.z, 7);
    }

    #[test]
    fn parse_full_tile_has_no_house_id_for_regular_tile() {
        let bytes = make_otbm_with_tile(2, 100, 100, 10, 20, 7, 0, 0);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles[0].house_id.is_none());
    }

    // -----------------------------------------------------------------------
    // parse_full — house tile association
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_house_tile_has_house_id() {
        let bytes = make_otbm_with_house_tile(42);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.tiles.len(), 1);
        assert_eq!(parsed.tiles[0].house_id, Some(42));
    }

    #[test]
    fn parse_full_house_tile_coords_correct() {
        let bytes = make_otbm_with_house_tile(99);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        let tile = &parsed.tiles[0];
        assert_eq!(tile.x, 100);
        assert_eq!(tile.y, 100);
        assert_eq!(tile.z, 7);
    }

    #[test]
    fn parse_full_house_tile_id_zero_still_parsed() {
        // house_id=0 is unusual but we should parse it without error
        let bytes = make_otbm_with_house_tile(0);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.tiles[0].house_id, Some(0));
    }

    // -----------------------------------------------------------------------
    // parse_full — tile flags
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_tile_flags_protection_zone() {
        let bytes = make_otbm_with_tile_flags(OTBM_TILEFLAG_PROTECTIONZONE);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles[0].flags.protection_zone);
        assert!(!parsed.tiles[0].flags.no_logout);
    }

    #[test]
    fn parse_full_tile_flags_no_logout() {
        let bytes = make_otbm_with_tile_flags(OTBM_TILEFLAG_NOLOGOUT);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles[0].flags.no_logout);
        assert!(!parsed.tiles[0].flags.protection_zone);
    }

    #[test]
    fn parse_full_tile_flags_nopvpzone() {
        let bytes = make_otbm_with_tile_flags(OTBM_TILEFLAG_NOPVPZONE);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles[0].flags.no_pvp_zone);
    }

    #[test]
    fn parse_full_tile_flags_pvpzone() {
        let bytes = make_otbm_with_tile_flags(OTBM_TILEFLAG_PVPZONE);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles[0].flags.pvp_zone);
    }

    #[test]
    fn parse_full_tile_no_flags_all_false() {
        let bytes = make_otbm_with_tile(2, 100, 100, 10, 10, 7, 0, 0);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        let flags = parsed.tiles[0].flags;
        assert!(!flags.protection_zone);
        assert!(!flags.no_pvp_zone);
        assert!(!flags.no_logout);
        assert!(!flags.pvp_zone);
    }

    #[test]
    fn parse_full_tile_flags_combined() {
        let flags = OTBM_TILEFLAG_PROTECTIONZONE | OTBM_TILEFLAG_NOLOGOUT;
        let bytes = make_otbm_with_tile_flags(flags);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles[0].flags.protection_zone);
        assert!(parsed.tiles[0].flags.no_logout);
    }

    // -----------------------------------------------------------------------
    // parse_full — item attributes on tiles
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_tile_with_item_attr_stores_item_id() {
        let bytes = make_otbm_with_item_on_tile(1234);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.tiles.len(), 1);
        assert_eq!(parsed.tiles[0].item_ids, vec![1234u16]);
    }

    #[test]
    fn parse_full_tile_no_item_attrs_yields_empty_item_ids() {
        let bytes = make_otbm_with_tile(2, 100, 100, 10, 10, 7, 0, 0);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.tiles[0].item_ids.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse_full — map-data attributes
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_reads_spawn_file_from_map_data_attrs() {
        let bytes = make_otbm_with_map_attrs("map-spawn.xml", "map-house.xml");
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.spawn_file.as_deref(), Some("map-spawn.xml"));
    }

    #[test]
    fn parse_full_reads_house_file_from_map_data_attrs() {
        let bytes = make_otbm_with_map_attrs("map-spawn.xml", "map-house.xml");
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.house_file.as_deref(), Some("map-house.xml"));
    }

    #[test]
    fn parse_full_no_attrs_spawn_and_house_file_none() {
        let bytes = make_otbm_header(2, 100, 100);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.spawn_file.is_none());
        assert!(parsed.house_file.is_none());
    }

    // -----------------------------------------------------------------------
    // parse_full — waypoints
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_waypoint_registered_by_name() {
        let bytes = make_otbm_with_waypoint("Temple", 120, 130, 7);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(
            parsed.waypoints.contains_key("Temple"),
            "waypoints: {:?}",
            parsed.waypoints
        );
    }

    #[test]
    fn parse_full_waypoint_position_correct() {
        let bytes = make_otbm_with_waypoint("Temple", 120, 130, 7);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        let pos = &parsed.waypoints["Temple"];
        assert_eq!(pos.x, 120);
        assert_eq!(pos.y, 130);
        assert_eq!(pos.z, 7);
    }

    #[test]
    fn parse_full_no_waypoints_map_empty() {
        let bytes = make_otbm_header(2, 100, 100);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert!(parsed.waypoints.is_empty());
    }

    #[test]
    fn parse_full_multiple_waypoints_all_registered() {
        // Build OTBM with 2 waypoints
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0x01];
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&100u16.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());

        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);

        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINTS);

        // waypoint 1: "Alpha" at (1,2,3)
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINT);
        buf.extend_from_slice(&encode_string("Alpha"));
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&2u16.to_le_bytes());
        buf.push(3u8);
        buf.push(NODE_END);

        // waypoint 2: "Beta" at (4,5,6)
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINT);
        buf.extend_from_slice(&encode_string("Beta"));
        buf.extend_from_slice(&4u16.to_le_bytes());
        buf.extend_from_slice(&5u16.to_le_bytes());
        buf.push(6u8);
        buf.push(NODE_END);

        buf.push(NODE_END); // close WAYPOINTS
        buf.push(NODE_END); // close MAP_DATA
        buf.push(NODE_END); // close root

        let parsed = IoMap::parse_full(&buf).unwrap();
        assert_eq!(parsed.waypoints.len(), 2);
        assert_eq!(parsed.waypoints["Alpha"], Position::new(1, 2, 3));
        assert_eq!(parsed.waypoints["Beta"], Position::new(4, 5, 6));
    }

    // -----------------------------------------------------------------------
    // TileFlags unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn tile_flags_from_otbm_all_zero_means_no_flags() {
        let f = TileFlags::from_otbm(0);
        assert!(!f.protection_zone);
        assert!(!f.no_pvp_zone);
        assert!(!f.no_logout);
        assert!(!f.pvp_zone);
    }

    #[test]
    fn tile_flags_from_otbm_all_flags_set() {
        let all = OTBM_TILEFLAG_PROTECTIONZONE
            | OTBM_TILEFLAG_NOPVPZONE
            | OTBM_TILEFLAG_NOLOGOUT
            | OTBM_TILEFLAG_PVPZONE;
        let f = TileFlags::from_otbm(all);
        assert!(f.protection_zone);
        assert!(f.no_pvp_zone);
        assert!(f.no_logout);
        assert!(f.pvp_zone);
    }

    // -----------------------------------------------------------------------
    // parse_full — magic / NODE_START / header truncation error paths
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_rejects_empty_buffer() {
        let result = IoMap::parse_full(&[]);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("too small") || msg.contains("magic"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_rejects_short_buffer_under_4_bytes() {
        let result = IoMap::parse_full(&[0x00, 0x4D, 0x42]);
        let msg = result.unwrap_err();
        assert!(msg.contains("too small"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_invalid_magic() {
        let bytes = [0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0x00, 0x01];
        let result = IoMap::parse_full(&bytes);
        let msg = result.unwrap_err();
        assert!(msg.contains("invalid magic"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_missing_node_start() {
        let bytes = [0x00, 0x4D, 0x42, 0x4F, 0x00, 0x00];
        let result = IoMap::parse_full(&bytes);
        let msg = result.unwrap_err();
        assert!(msg.contains("NODE_START"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_short_after_node_start() {
        // magic + NODE_START but no root type byte
        let bytes = [0x00, 0x4D, 0x42, 0x4F, 0xFE];
        let result = IoMap::parse_full(&bytes);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("missing root node type") || msg.contains("NODE_START"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_rejects_missing_map_version_attribute() {
        // magic + NODE_START + root type + NODE_END (no attribute)
        let bytes = [0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0xFF];
        let result = IoMap::parse_full(&bytes);
        let msg = result.unwrap_err();
        assert!(msg.contains("map-version"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_attribute_too_short() {
        // magic + NODE_START + root type + attr 0x01 + only 5 bytes of data + NODE_END
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, 0xFE, 0x00, 0x01];
        buf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]); // only 5 bytes
        buf.push(0xFF);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("attribute too short"), "got: {}", msg);
    }

    // -----------------------------------------------------------------------
    // parse_full — map_data type byte and wrong-type error paths
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_rejects_eof_after_map_data_node_start() {
        // build valid header, then NODE_START with no type byte after
        let mut buf = make_otbm_header(2, 100, 100);
        // remove final NODE_END so we can append a dangling NODE_START
        buf.pop();
        buf.push(NODE_START);
        // intentionally no further bytes — EOF after NODE_START
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("expected OTBM_MAP_DATA type byte"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_rejects_wrong_map_data_node_type() {
        // build valid header then a NODE_START + bogus type
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop(); // remove final NODE_END
        buf.push(NODE_START);
        buf.push(99u8); // not OTBM_MAP_DATA
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("expected OTBM_MAP_DATA"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_unexpected_eof_after_node_start_in_map_data() {
        // header + map_data + NODE_START with no type byte
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        // no further bytes — EOF inside map-data
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("unexpected end after NODE_START"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_rejects_unknown_map_data_child_node_type() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(123u8); // unknown child type
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown map-data child"), "got: {}", msg);
    }

    // -----------------------------------------------------------------------
    // parse_full — map-data attribute error paths
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_reads_description_attribute() {
        // header + map_data + ATTR_DESCRIPTION string
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(OTBM_ATTR_DESCRIPTION);
        buf.extend_from_slice(&encode_string("Test map"));
        buf.push(NODE_END);
        buf.push(NODE_END);
        let parsed = IoMap::parse_full(&buf).unwrap();
        // description is intentionally not stored — just verify parse succeeds
        assert!(parsed.header.is_some());
    }

    #[test]
    fn parse_full_rejects_unknown_map_data_attribute() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(99u8); // unknown attribute type
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown map-data attribute"), "got: {}", msg);
    }

    // -----------------------------------------------------------------------
    // parse_full — tile-area error paths
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_rejects_tile_area_props_too_short() {
        // header + map_data + tile_area with <5 byte props
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        // only 3 props bytes instead of 5
        buf.push(0x01);
        buf.push(0x02);
        buf.push(0x03);
        buf.push(NODE_END); // close tile_area
        buf.push(NODE_END); // close map_data
        buf.push(NODE_END); // close root
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("tile-area props too short"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_unknown_tile_node_type_in_tile_area() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        buf.push(99u8); // unknown tile node type
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown tile node type"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_unexpected_eof_after_node_start_in_tile_area() {
        // header + map_data + tile_area + props + NODE_START with no type byte
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        // intentionally no further bytes — EOF after NODE_START in tile-area
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("unexpected end after NODE_START in tile-area"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_rejects_tile_props_too_short_for_coords() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(0x00); // only 1 byte instead of 2
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("tile props too short"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_house_tile_too_short_for_house_id() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        buf.push(OTBM_HOUSETILE);
        buf.push(0u8); // dx
        buf.push(0u8); // dy
                       // only 2 of the 4 bytes for house_id
        buf.push(0xAA);
        buf.push(0xBB);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("too short to read house_id"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_tile_flags_attribute_too_short() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(0u8);
        buf.push(0u8);
        buf.push(OTBM_ATTR_TILE_FLAGS);
        buf.push(0x01); // only 1 byte of flags instead of 4
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("too short for tile flags"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_item_attr_too_short() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(0u8);
        buf.push(0u8);
        buf.push(OTBM_ATTR_ITEM);
        buf.push(0xAA); // only 1 byte instead of 2 for item_id
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("too short for item attr"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_unknown_tile_attribute() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(0u8);
        buf.push(0u8);
        buf.push(99u8); // unknown tile attribute
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown attribute"), "got: {}", msg);
    }

    // -----------------------------------------------------------------------
    // parse_full — towns parsing (happy path + errors)
    // -----------------------------------------------------------------------

    /// Builds an OTBM with a single town inside an OTBM_TOWNS node.
    fn make_otbm_with_town(town_id: u32, name: &str, x: u16, y: u16, z: u8) -> Vec<u8> {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TOWNS);
        // no towns-level props
        buf.push(NODE_START);
        buf.push(13u8); // OTBM_TOWN
        buf.extend_from_slice(&town_id.to_le_bytes());
        buf.extend_from_slice(&encode_string(name));
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.push(z);
        buf.push(NODE_END); // close town
        buf.push(NODE_END); // close towns
        buf.push(NODE_END); // close map_data
        buf.push(NODE_END); // close root
        buf
    }

    #[test]
    fn parse_full_town_registers_in_town_names() {
        let bytes = make_otbm_with_town(7, "Thais", 1000, 1000, 7);
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.town_names.get(&7).map(String::as_str), Some("Thais"));
    }

    #[test]
    fn parse_full_multiple_towns_all_registered() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TOWNS);

        // town 1
        buf.push(NODE_START);
        buf.push(13u8);
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(&encode_string("Alpha"));
        buf.extend_from_slice(&10u16.to_le_bytes());
        buf.extend_from_slice(&20u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_END);

        // town 2
        buf.push(NODE_START);
        buf.push(13u8);
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&encode_string("Beta"));
        buf.extend_from_slice(&30u16.to_le_bytes());
        buf.extend_from_slice(&40u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_END);

        buf.push(NODE_END); // close towns
        buf.push(NODE_END); // close map_data
        buf.push(NODE_END); // close root

        let parsed = IoMap::parse_full(&buf).unwrap();
        assert_eq!(parsed.town_names.len(), 2);
        assert_eq!(parsed.town_names[&1], "Alpha");
        assert_eq!(parsed.town_names[&2], "Beta");
    }

    #[test]
    fn parse_full_rejects_unexpected_eof_in_towns() {
        // header + map_data + towns + NODE_START but no type byte
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TOWNS);
        buf.push(NODE_START); // dangling
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("unexpected end in towns"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_town_props_too_short_for_id() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TOWNS);
        buf.push(NODE_START);
        buf.push(13u8); // town type
        buf.push(0x01); // only 1 byte (need 4 for u32)
        buf.push(NODE_END); // close town
        buf.push(NODE_END); // close towns
        buf.push(NODE_END); // close map_data
        buf.push(NODE_END); // close root
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("town props too short for town_id"),
            "got: {}",
            msg
        );
    }

    // -----------------------------------------------------------------------
    // parse_full — waypoints error paths
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_rejects_unexpected_eof_in_waypoints() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINTS);
        buf.push(NODE_START); // dangling
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("unexpected end in waypoints"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_non_waypoint_node_inside_waypoints() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINTS);
        buf.push(NODE_START);
        buf.push(99u8); // not OTBM_WAYPOINT
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("expected OTBM_WAYPOINT"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_waypoint_coords_too_short() {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINTS);
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINT);
        // name "X" but no coords following
        buf.extend_from_slice(&encode_string("X"));
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("too short for coords"), "got: {}", msg);
    }

    // -----------------------------------------------------------------------
    // skip_children: nested children, ESCAPE, default byte, unexpected EOF
    // -----------------------------------------------------------------------

    /// Builds an OTBM with a tile whose child contains nested NODE_START /
    /// NODE_END / ESCAPE bytes, to exercise all `skip_children` branches.
    fn make_otbm_with_nested_tile_child() -> Vec<u8> {
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_TILE_AREA);
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.extend_from_slice(&0u16.to_le_bytes());
        buf.push(7u8);
        buf.push(NODE_START);
        buf.push(OTBM_TILE);
        buf.push(0u8); // dx
        buf.push(0u8); // dy
                       // Outer child = OTBM_ITEM with nested child + ESCAPE + arbitrary bytes
        buf.push(NODE_START);
        buf.push(6u8); // OTBM_ITEM (arbitrary type, skip_children doesn't check)
                       // a payload byte
        buf.push(0x11);
        // ESCAPE + escaped 0xFE byte (should NOT trigger NODE_START)
        buf.push(ESCAPE);
        buf.push(0xFE);
        // nested NODE_START + type + NODE_END (depth=1 -> depth=0)
        buf.push(NODE_START);
        buf.push(0x77);
        buf.push(NODE_END);
        buf.push(NODE_END); // close outer item
        buf.push(NODE_END); // close tile
        buf.push(NODE_END); // close tile_area
        buf.push(NODE_END); // close map_data
        buf.push(NODE_END); // close root
        buf
    }

    #[test]
    fn skip_children_handles_nested_node_start_node_end_and_escape() {
        let bytes = make_otbm_with_nested_tile_child();
        let parsed = IoMap::parse_full(&bytes).unwrap();
        assert_eq!(parsed.tiles.len(), 1);
    }

    #[test]
    fn skip_children_unexpected_end_when_truncated() {
        // Construct a buffer where skip_children is called and runs to EOF
        // without seeing the closing NODE_END.
        //
        // We build a waypoint that has a child NODE_START but the file is
        // truncated before the closing NODE_END is reached.
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINTS);
        buf.push(NODE_START);
        buf.push(OTBM_WAYPOINT);
        buf.extend_from_slice(&encode_string("T"));
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&2u16.to_le_bytes());
        buf.push(3u8);
        // Open a child node that we never close — skip_children will run off
        // the end of the buffer.
        buf.push(NODE_START);
        buf.push(6u8); // arbitrary child type
        buf.push(0xAA); // a non-special byte to exercise the catch-all arm
                        // intentionally NO closing bytes
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("unexpected end while skipping children"),
            "got: {}",
            msg
        );
    }

    // -----------------------------------------------------------------------
    // read_string: error paths (length truncation + UTF-8 invalid)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_full_rejects_truncated_string_length_in_attr() {
        // header + map_data + ATTR_DESCRIPTION but only 1 byte of length (need 2)
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(OTBM_ATTR_DESCRIPTION);
        buf.push(0x05); // only 1 of the 2 length bytes
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("string too short") || msg.contains("string body truncated"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_full_rejects_truncated_string_body_in_attr() {
        // length says 100 bytes but only 3 bytes follow
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(OTBM_ATTR_DESCRIPTION);
        buf.extend_from_slice(&100u16.to_le_bytes()); // length = 100
        buf.extend_from_slice(b"abc"); // only 3 bytes
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("string body truncated"), "got: {}", msg);
    }

    #[test]
    fn parse_full_rejects_invalid_utf8_in_string() {
        // length = 2, body = 0xFF 0xFF (invalid UTF-8)
        let mut buf = make_otbm_header(2, 100, 100);
        buf.pop();
        buf.push(NODE_START);
        buf.push(OTBM_MAP_DATA);
        buf.push(OTBM_ATTR_DESCRIPTION);
        buf.extend_from_slice(&2u16.to_le_bytes());
        // ESCAPE-protect 0xFF bytes because raw 0xFF is NODE_END
        buf.push(ESCAPE);
        buf.push(0xFF);
        buf.push(ESCAPE);
        buf.push(0xFF);
        buf.push(NODE_END);
        buf.push(NODE_END);
        let result = IoMap::parse_full(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("invalid UTF-8"), "got: {}", msg);
    }

    // -----------------------------------------------------------------------
    // collect_props_with_end: ESCAPE-at-EOF edge case (via parse_header)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_header_rejects_escape_at_end_of_buffer() {
        // magic + NODE_START + root type + ESCAPE then EOF
        let buf = [0x00, 0x4D, 0x42, 0x4F, NODE_START, 0x00, ESCAPE];
        let result = IoMap::parse_header(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("unexpected end after ESCAPE"), "got: {}", msg);
    }

    // -----------------------------------------------------------------------
    // load_from_bytes error propagation
    // -----------------------------------------------------------------------

    #[test]
    fn load_from_bytes_propagates_too_short_error() {
        let result = IoMap::load_from_bytes(&[]);
        let msg = result.unwrap_err();
        assert!(msg.contains("too small"), "got: {}", msg);
    }

    #[test]
    fn load_from_bytes_propagates_missing_root_type_byte_error() {
        // magic + NODE_START only (5 bytes total) -> bytes.len() < 6
        let bytes = [0x00, 0x4D, 0x42, 0x4F, NODE_START];
        let result = IoMap::load_from_bytes(&bytes);
        let msg = result.unwrap_err();
        assert!(
            msg.contains("missing root node type") || msg.contains("NODE_START"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_header_rejects_missing_root_type_byte_directly() {
        // magic (4) + NODE_START (1) = 5 bytes; need 6
        let bytes = [0x00, 0x4D, 0x42, 0x4F, NODE_START];
        let result = IoMap::parse_header(&bytes);
        let msg = result.unwrap_err();
        // Either branch order may catch it
        assert!(
            msg.contains("missing root node type") || msg.contains("NODE_START"),
            "got: {}",
            msg
        );
    }

    #[test]
    fn parse_header_rejects_root_attribute_too_short_directly() {
        // magic + NODE_START + root type + attr 0x01 + 3 bytes
        let mut buf = vec![0x00, 0x4D, 0x42, 0x4F, NODE_START, 0x00, 0x01];
        buf.extend_from_slice(&[0x01, 0x02, 0x03]);
        buf.push(NODE_END);
        let result = IoMap::parse_header(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("attribute too short"), "got: {}", msg);
    }

    #[test]
    fn parse_header_rejects_missing_map_version_attr_directly() {
        // magic + NODE_START + root type + NODE_END (no attr inside)
        let buf = [0x00, 0x4D, 0x42, 0x4F, NODE_START, 0x00, NODE_END];
        let result = IoMap::parse_header(&buf);
        let msg = result.unwrap_err();
        assert!(msg.contains("map-version"), "got: {}", msg);
    }
}
