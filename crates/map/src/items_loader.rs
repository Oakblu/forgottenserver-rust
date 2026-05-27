use std::collections::HashSet;
use std::path::Path;

use forgottenserver_common::{
    fileloader::{FileLoader, Node},
    itemloader::{ItemAttr, ItemGroup, ITEM_ATTR_FIRST, ITEM_ATTR_LAST},
};
use forgottenserver_items::registry::{ItemType, ItemsRegistry};

pub fn load_items_otb(path: &Path) -> Result<ItemsRegistry, String> {
    let loader = FileLoader::open(path)?;
    let root = loader.get_root_node().ok_or("OTB: no root node")?;
    let mut registry = ItemsRegistry::new();
    // Dedupe unknown-attribute warnings across the whole OTB. A real OTB
    // file has tens of thousands of items, each potentially encoding the
    // same handful of unknown-to-us attributes — printing per-occurrence
    // floods stderr with ~27k duplicate lines and noticeably slows boot.
    let mut seen_unknown_attrs: HashSet<u8> = HashSet::new();

    for node in &root.children {
        match parse_item_node(node, &mut seen_unknown_attrs) {
            Ok(item) => registry.register(item),
            Err(e) => eprintln!("[Warning] Skipping OTB item node: {e}"),
        }
    }
    Ok(registry)
}

fn parse_item_node(node: &Node, seen_unknown_attrs: &mut HashSet<u8>) -> Result<ItemType, String> {
    let group = parse_group(node.type_byte);
    let props = &node.props;

    if props.len() < 4 {
        return Err("item props too short for flags field".into());
    }

    let flags = u32::from_le_bytes([props[0], props[1], props[2], props[3]]);
    let mut pos = 4;

    let mut server_id: u16 = 0;
    let mut client_id: u16 = 0;
    let mut speed: u16 = 0;
    let mut weight: u16 = 0;

    while pos < props.len() {
        let attrib = props[pos];
        pos += 1;

        if pos + 2 > props.len() {
            break;
        }
        let datalen = u16::from_le_bytes([props[pos], props[pos + 1]]) as usize;
        pos += 2;

        if pos + datalen > props.len() {
            break;
        }
        let data = &props[pos..pos + datalen];
        pos += datalen;

        if !(ITEM_ATTR_FIRST..ITEM_ATTR_LAST).contains(&attrib) {
            if seen_unknown_attrs.insert(attrib) {
                eprintln!(
                    "[Warning] Skipping unknown OTB item attribute: {attrib:#04x} \
                     (further occurrences of this attribute suppressed)"
                );
            }
            continue;
        }

        match attrib {
            x if x == ItemAttr::ServerId as u8 => {
                if datalen == 2 {
                    server_id = u16::from_le_bytes([data[0], data[1]]);
                }
            }
            x if x == ItemAttr::ClientId as u8 => {
                if datalen == 2 {
                    client_id = u16::from_le_bytes([data[0], data[1]]);
                }
            }
            x if x == ItemAttr::Speed as u8 => {
                if datalen == 2 {
                    speed = u16::from_le_bytes([data[0], data[1]]);
                }
            }
            x if x == ItemAttr::Weight as u8 => {
                if datalen == 2 {
                    weight = u16::from_le_bytes([data[0], data[1]]);
                }
            }
            _ => {}
        }
    }

    if server_id == 0 {
        return Err("item node has no server_id".into());
    }

    Ok(ItemType {
        server_id,
        client_id,
        group,
        flags,
        speed,
        weight,
    })
}

fn parse_group(type_byte: u8) -> ItemGroup {
    match type_byte {
        0 => ItemGroup::None,
        1 => ItemGroup::Ground,
        2 => ItemGroup::Container,
        3 => ItemGroup::Weapon,
        4 => ItemGroup::Ammunition,
        5 => ItemGroup::Armor,
        6 => ItemGroup::Charges,
        7 => ItemGroup::Teleport,
        8 => ItemGroup::MagicField,
        9 => ItemGroup::Writeable,
        10 => ItemGroup::Key,
        11 => ItemGroup::Splash,
        12 => ItemGroup::Fluid,
        13 => ItemGroup::Door,
        14 => ItemGroup::Deprecated,
        15 => ItemGroup::Podium,
        _ => ItemGroup::None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Build a minimal valid OTB binary.
    // `items`: list of (type_byte, attr_pairs).
    // Each attr_pair is (tag, data_bytes).
    type OtbItemAttr = (u8, Vec<u8>);
    type OtbItem = (u8, Vec<OtbItemAttr>);

    fn build_otb(items: &[OtbItem]) -> Vec<u8> {
        let mut buf = vec![0x00u8, 0x00, 0x00, 0x00]; // 4-byte file identifier

        // Root node
        buf.push(NODE_START);
        buf.push(0x00); // root type byte (ignored by loader)
                        // root has no props — just children

        for (type_byte, attrs) in items {
            buf.push(NODE_START);
            buf.push(*type_byte);

            // 4 bytes flags (FLAG_MOVEABLE = 0x40)
            push_escaped(&mut buf, &[0x40, 0x00, 0x00, 0x00]);

            for (tag, data) in attrs {
                let datalen = data.len() as u16;
                push_escaped(&mut buf, &[*tag]);
                push_escaped(&mut buf, &datalen.to_le_bytes());
                push_escaped(&mut buf, data);
            }

            buf.push(NODE_END);
        }

        buf.push(NODE_END);
        buf
    }

    fn push_escaped(buf: &mut Vec<u8>, data: &[u8]) {
        for &b in data {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
    }

    fn write_tmp_otb(data: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("temp file");
        f.write_all(data).expect("write");
        f
    }

    // -----------------------------------------------------------------------
    // Test 1: ground item with correct speed and weight
    // -----------------------------------------------------------------------
    #[test]
    fn items_otb_loads_ground_item_with_correct_speed_and_weight() {
        let otb = build_otb(&[(
            1, // Ground
            vec![
                (ItemAttr::ServerId as u8, vec![100, 0]), // server_id = 100
                (ItemAttr::Speed as u8, vec![220, 0]),    // speed = 220
                (ItemAttr::Weight as u8, vec![150, 0]),   // weight = 150
            ],
        )]);

        let tmp = write_tmp_otb(&otb);
        let registry = load_items_otb(tmp.path()).expect("load ok");

        assert_eq!(registry.len(), 1);
        let item = registry.get(100).expect("item 100");
        assert_eq!(item.group, ItemGroup::Ground);
        assert_eq!(item.speed, 220);
        assert_eq!(item.weight, 150);
        assert_eq!(item.server_id, 100);
    }

    // -----------------------------------------------------------------------
    // Test 2: unknown attribute is skipped, item still loads
    // -----------------------------------------------------------------------
    #[test]
    fn items_otb_unknown_attribute_is_skipped_not_panicked() {
        let otb = build_otb(&[(
            1, // Ground
            vec![
                (0x80, vec![0xDE, 0xAD, 0xBE, 0xEF]), // unknown attr — must be skipped
                (ItemAttr::ServerId as u8, vec![42, 0]),
                (ItemAttr::Speed as u8, vec![50, 0]),
            ],
        )]);

        let tmp = write_tmp_otb(&otb);
        let registry = load_items_otb(tmp.path()).expect("should not panic/error");
        let item = registry
            .get(42)
            .expect("item 42 should still be registered");
        assert_eq!(item.speed, 50);
    }

    // -----------------------------------------------------------------------
    // Test 3: multiple items
    // -----------------------------------------------------------------------
    #[test]
    fn items_otb_loads_multiple_items() {
        let otb = build_otb(&[
            (1, vec![(ItemAttr::ServerId as u8, vec![1, 0])]),
            (2, vec![(ItemAttr::ServerId as u8, vec![2, 0])]),
            (11, vec![(ItemAttr::ServerId as u8, vec![3, 0])]),
        ]);

        let tmp = write_tmp_otb(&otb);
        let registry = load_items_otb(tmp.path()).expect("load ok");
        assert_eq!(registry.len(), 3);
        assert_eq!(registry.get(1).unwrap().group, ItemGroup::Ground);
        assert_eq!(registry.get(2).unwrap().group, ItemGroup::Container);
        assert_eq!(registry.get(3).unwrap().group, ItemGroup::Splash);
    }

    // -----------------------------------------------------------------------
    // Test 4: missing file returns Err
    // -----------------------------------------------------------------------
    #[test]
    fn items_otb_missing_file_returns_error() {
        let result = load_items_otb(Path::new("/tmp/nonexistent_items_xyz.otb"));
        assert!(result.is_err());
    }
}
