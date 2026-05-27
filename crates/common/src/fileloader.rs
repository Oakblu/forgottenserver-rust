//! Migrated from forgottenserver/src/fileloader.h + fileloader.cpp
//! Parses the OTB (Open Tibia Binary) file format — a tree of nodes.
//!
//! Format constants:
//!   NODE_START = 0xFE
//!   NODE_END   = 0xFF
//!   ESCAPE     = 0xFD
//!
//! Layout: 4-byte identifier | NODE_START | type_byte | prop_bytes... | NODE_END
//! Children appear inline between a parent's prop_bytes and its NODE_END.
//! Any 0xFD, 0xFE, or 0xFF occurring inside prop data is preceded by 0xFD.

use std::path::Path;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const NODE_START: u8 = 0xFE;
pub const NODE_END: u8 = 0xFF;
pub const ESCAPE: u8 = 0xFD;

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// A node in the OTB tree.
#[derive(Debug, Clone)]
pub struct Node {
    pub type_byte: u8,
    /// Unescaped property bytes for this node.
    pub props: Vec<u8>,
    pub children: Vec<Node>,
}

// ---------------------------------------------------------------------------
// FileLoader
// ---------------------------------------------------------------------------

/// Parses an OTB binary file into a tree of [`Node`]s.
pub struct FileLoader {
    root: Node,
}

impl FileLoader {
    /// Read and parse the OTB file at `path`.
    ///
    /// # Errors
    /// Returns a human-readable error string when the file cannot be read or
    /// the binary data does not conform to the OTB format.
    pub fn open(path: &Path) -> Result<FileLoader, String> {
        let data = std::fs::read(path)
            .map_err(|e| format!("Cannot open file '{}': {}", path.display(), e))?;

        let root = Self::parse(&data)?;
        Ok(FileLoader { root })
    }

    /// Return a reference to the root node.
    pub fn get_root_node(&self) -> Option<&Node> {
        Some(&self.root)
    }

    /// Return the first child of `node`, if any.
    pub fn get_first_child<'a>(&self, node: &'a Node) -> Option<&'a Node> {
        node.children.first()
    }

    /// Return the next sibling after `node` within `parent`.
    pub fn get_next_node<'a>(&self, parent: &'a Node, node: &'a Node) -> Option<&'a Node> {
        let node_ptr = node as *const Node;
        let mut found = false;
        for child in &parent.children {
            if found {
                return Some(child);
            }
            if std::ptr::eq(child as *const Node, node_ptr) {
                found = true;
            }
        }
        None
    }

    /// Return the raw (already unescaped) prop bytes for `node`.
    pub fn get_props<'a>(&self, node: &'a Node) -> &'a [u8] {
        &node.props
    }

    // -----------------------------------------------------------------------
    // Internal parsing
    // -----------------------------------------------------------------------

    fn parse(data: &[u8]) -> Result<Node, String> {
        // Minimum layout: 4-byte identifier + START + type + END = 7 bytes
        if data.len() < 7 {
            return Err("OTB file too small".into());
        }

        let mut pos = 4; // skip the 4-byte file identifier

        if data[pos] != NODE_START {
            return Err("OTB: expected NODE_START at byte 4".into());
        }
        pos += 1;

        let root = Self::parse_node(data, &mut pos)?;
        Ok(root)
    }

    /// Parse one node starting just after the NODE_START byte.
    /// `pos` must point at the type byte on entry.
    /// Returns the parsed node and advances `pos` past the trailing NODE_END.
    fn parse_node(data: &[u8], pos: &mut usize) -> Result<Node, String> {
        if *pos >= data.len() {
            return Err("OTB: unexpected end of data (expected type byte)".into());
        }

        let type_byte = data[*pos];
        *pos += 1;

        let mut props: Vec<u8> = Vec::new();
        let mut children: Vec<Node> = Vec::new();

        loop {
            if *pos >= data.len() {
                return Err("OTB: unexpected end of data inside node".into());
            }

            match data[*pos] {
                NODE_START => {
                    // Start of a child node
                    *pos += 1;
                    let child = Self::parse_node(data, pos)?;
                    children.push(child);
                }
                NODE_END => {
                    *pos += 1;
                    break;
                }
                ESCAPE => {
                    // Next byte is a literal prop byte
                    *pos += 1;
                    if *pos >= data.len() {
                        return Err("OTB: unexpected end of data after ESCAPE".into());
                    }
                    props.push(data[*pos]);
                    *pos += 1;
                }
                byte => {
                    props.push(byte);
                    *pos += 1;
                }
            }
        }

        Ok(Node {
            type_byte,
            props,
            children,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Write `bytes` to a temporary file and call `FileLoader::open` on it.
    fn load_bytes(bytes: &[u8]) -> Result<FileLoader, String> {
        let mut tmp = NamedTempFile::new().expect("temp file");
        tmp.write_all(bytes).expect("write");
        FileLoader::open(tmp.path())
    }

    /// Minimal OTB: 4-byte identifier + START + type(0x00) + END
    fn minimal_otb(type_byte: u8, props: &[u8], children: &[Vec<u8>]) -> Vec<u8> {
        let mut buf = vec![0x00, 0x00, 0x00, 0x00]; // identifier (wildcard)
        buf.push(NODE_START);
        buf.push(type_byte);
        // props (escaped if needed)
        for &b in props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        // children
        for child_bytes in children {
            buf.extend_from_slice(child_bytes);
        }
        buf.push(NODE_END);
        buf
    }

    fn make_child_node(type_byte: u8, props: &[u8]) -> Vec<u8> {
        let mut buf = vec![NODE_START, type_byte];
        for &b in props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        buf.push(NODE_END);
        buf
    }

    // -------------------------------------------------------------------
    // Test 1: empty tree (root only, no children, no props)
    // -------------------------------------------------------------------
    #[test]
    fn test_empty_tree_root_only() {
        let data = minimal_otb(0x01, &[], &[]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().expect("root exists");
        assert_eq!(root.type_byte, 0x01);
        assert!(root.props.is_empty());
        assert!(root.children.is_empty());
    }

    // -------------------------------------------------------------------
    // Test 2: root with 3 prop bytes
    // -------------------------------------------------------------------
    #[test]
    fn test_root_with_three_props() {
        let data = minimal_otb(0x02, &[0x0A, 0x0B, 0x0C], &[]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().expect("root exists");
        assert_eq!(root.type_byte, 0x02);
        assert_eq!(loader.get_props(root), &[0x0A, 0x0B, 0x0C]);
        assert!(root.children.is_empty());
    }

    // -------------------------------------------------------------------
    // Test 3: root with one child (different type byte)
    // -------------------------------------------------------------------
    #[test]
    fn test_root_with_one_child() {
        let child_bytes = make_child_node(0x42, &[0xAA, 0xBB]);
        let data = minimal_otb(0x01, &[], &[child_bytes]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().expect("root exists");
        assert_eq!(root.children.len(), 1);

        let child = loader.get_first_child(root).expect("first child");
        assert_eq!(child.type_byte, 0x42);
        assert_eq!(loader.get_props(child), &[0xAA, 0xBB]);
    }

    // -------------------------------------------------------------------
    // Test 4: ESCAPE byte in prop data is correctly unescaped
    // -------------------------------------------------------------------
    #[test]
    fn test_escape_byte_unescaped_in_props() {
        // We want prop bytes [0x01, 0xFE, 0x02] — 0xFE must be escaped in the binary
        // minimal_otb applies escaping for us
        let data = minimal_otb(0x01, &[0x01, NODE_START, 0x02], &[]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().expect("root");
        // After unescaping the prop should contain the literal 0xFE
        assert_eq!(loader.get_props(root), &[0x01, NODE_START, 0x02]);
    }

    // -------------------------------------------------------------------
    // Test 5: ESCAPE byte 0xFD in props is unescaped
    // -------------------------------------------------------------------
    #[test]
    fn test_escape_byte_0xfd_unescaped() {
        let data = minimal_otb(0x01, &[ESCAPE, 0x05], &[]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().unwrap();
        assert_eq!(loader.get_props(root), &[ESCAPE, 0x05]);
    }

    // -------------------------------------------------------------------
    // Test 6: missing file → Err
    // -------------------------------------------------------------------
    #[test]
    fn test_missing_file_returns_error() {
        let result = FileLoader::open(Path::new("/tmp/nonexistent_otb_file_xyz_12345.otb"));
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------
    // Test 7: get_next_node returns the sibling after a node
    // -------------------------------------------------------------------
    #[test]
    fn test_get_next_node() {
        let child1 = make_child_node(0x10, &[]);
        let child2 = make_child_node(0x20, &[]);
        let data = minimal_otb(0x01, &[], &[child1, child2]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().unwrap();
        assert_eq!(root.children.len(), 2);

        let first = loader.get_first_child(root).unwrap();
        assert_eq!(first.type_byte, 0x10);

        let second = loader.get_next_node(root, first).unwrap();
        assert_eq!(second.type_byte, 0x20);

        // No node after the last child
        assert!(loader.get_next_node(root, second).is_none());
    }

    // -------------------------------------------------------------------
    // Test 8: file too small returns Err
    // -------------------------------------------------------------------
    #[test]
    fn test_too_small_file_returns_error() {
        let result = load_bytes(&[0x00, 0x01, 0x02]);
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------
    // Test 9: NODE_START missing at byte 4 → Err
    // -------------------------------------------------------------------
    #[test]
    fn test_no_node_start_at_byte_4_returns_error() {
        // 4-byte identifier followed by something that is NOT NODE_START (0xFE)
        let data = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, NODE_END];
        let result = load_bytes(&data);
        assert!(
            result.is_err(),
            "expected Err when byte 4 is not NODE_START"
        );
    }

    // -------------------------------------------------------------------
    // Test 10: unclosed node (NODE_END missing) → Err
    // -------------------------------------------------------------------
    #[test]
    fn test_unclosed_node_returns_error() {
        // Identifier + START + type byte — no matching NODE_END
        let data = vec![0x00, 0x00, 0x00, 0x00, NODE_START, 0x01, 0x0A];
        let result = load_bytes(&data);
        assert!(result.is_err(), "expected Err for unclosed node");
    }

    // -------------------------------------------------------------------
    // Test 11: ESCAPE byte at very end of data → Err
    // -------------------------------------------------------------------
    #[test]
    fn test_escape_at_eof_returns_error() {
        // Identifier + START + type + ESCAPE (no following byte)
        let data = vec![0x00, 0x00, 0x00, 0x00, NODE_START, 0x01, ESCAPE];
        let result = load_bytes(&data);
        assert!(result.is_err(), "expected Err when ESCAPE is the last byte");
    }

    // -------------------------------------------------------------------
    // Test 12: get_props returns empty slice when node has no props
    // -------------------------------------------------------------------
    #[test]
    fn test_get_props_empty_when_no_props() {
        let data = minimal_otb(0x01, &[], &[]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().unwrap();
        assert!(loader.get_props(root).is_empty());
    }

    // -------------------------------------------------------------------
    // Test 13: three children, iterator visits all in order
    // -------------------------------------------------------------------
    #[test]
    fn test_multi_node_iterator_three_children() {
        let child1 = make_child_node(0x10, &[0xAA]);
        let child2 = make_child_node(0x20, &[0xBB]);
        let child3 = make_child_node(0x30, &[0xCC]);
        let data = minimal_otb(0x01, &[], &[child1, child2, child3]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().unwrap();

        assert_eq!(root.children.len(), 3);

        let n1 = loader.get_first_child(root).unwrap();
        assert_eq!(n1.type_byte, 0x10);
        assert_eq!(loader.get_props(n1), &[0xAA]);

        let n2 = loader.get_next_node(root, n1).unwrap();
        assert_eq!(n2.type_byte, 0x20);
        assert_eq!(loader.get_props(n2), &[0xBB]);

        let n3 = loader.get_next_node(root, n2).unwrap();
        assert_eq!(n3.type_byte, 0x30);
        assert_eq!(loader.get_props(n3), &[0xCC]);

        // No fourth child
        assert!(loader.get_next_node(root, n3).is_none());
    }

    // -------------------------------------------------------------------
    // Test 14: ESCAPE followed by 0xFF literal in props is unescaped
    // -------------------------------------------------------------------
    #[test]
    fn test_escape_node_end_in_props() {
        // Want prop bytes [0xFF] — 0xFF must be escaped in the binary
        let data = minimal_otb(0x01, &[NODE_END], &[]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().unwrap();
        assert_eq!(loader.get_props(root), &[NODE_END]);
    }

    // -------------------------------------------------------------------
    // Test 15: NODE_START at EOF inside a node body → Err
    //   Covers the "expected type byte" branch inside parse_node: when a
    //   parent encounters NODE_START, advances past it, and the recursive
    //   call discovers `*pos >= data.len()` before it can read the child
    //   type byte. C++: `if (++it == fileContents.end()) throw`.
    // -------------------------------------------------------------------
    #[test]
    fn test_node_start_at_eof_returns_error() {
        // 4-byte identifier + root START + root type + child NODE_START
        // (no child type byte, no END bytes). parse_node recurses, finds
        // EOF where it expected a type byte.
        let data = vec![
            0x00, 0x00, 0x00, 0x00,       // identifier
            NODE_START, // root start
            0x01,       // root type
            NODE_START, // child start — no type byte follows
        ];
        let result = load_bytes(&data);
        assert!(
            result.is_err(),
            "expected Err when NODE_START is the last byte (no child type follows)"
        );
    }

    // -------------------------------------------------------------------
    // Test 16: error in a child node propagates through the parent
    //   Covers the `?` propagation on the recursive parse_node call: a
    //   parent that successfully consumes a child NODE_START must surface
    //   the inner error (here, ESCAPE-at-EOF inside the child) rather than
    //   swallow it.
    // -------------------------------------------------------------------
    #[test]
    fn test_child_parse_error_propagates() {
        // Root contains one child whose body ends with a dangling ESCAPE.
        let data = vec![
            0x00, 0x00, 0x00, 0x00,       // identifier
            NODE_START, // root start
            0x01,       // root type
            NODE_START, // child start
            0x02,       // child type
            ESCAPE,     // dangling ESCAPE — no byte follows
        ];
        let result = load_bytes(&data);
        assert!(
            result.is_err(),
            "expected Err propagated from inner ESCAPE-at-EOF"
        );
    }

    // -------------------------------------------------------------------
    // Test 17: helper `make_child_node` correctly escapes a raw 0xFE prop
    //   Exercises the escape branch inside the test helper AND verifies
    //   that a child's props are unescaped on parse, end-to-end. This
    //   guards against silent breakage of the test fixtures.
    // -------------------------------------------------------------------
    #[test]
    fn test_child_node_with_escaped_prop_bytes() {
        // Build a child whose props include all three special bytes —
        // make_child_node must inject an ESCAPE before each.
        let child = make_child_node(0x77, &[NODE_START, NODE_END, ESCAPE, 0x42]);
        let data = minimal_otb(0x01, &[], &[child]);
        let loader = load_bytes(&data).expect("parse ok");
        let root = loader.get_root_node().unwrap();
        let first = loader.get_first_child(root).expect("child exists");
        assert_eq!(first.type_byte, 0x77);
        // After unescaping, props recover the original literal byte sequence.
        assert_eq!(
            loader.get_props(first),
            &[NODE_START, NODE_END, ESCAPE, 0x42]
        );
    }
}
