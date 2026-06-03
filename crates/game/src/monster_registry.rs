use std::path::Path;

use forgottenserver_entity::monsters::Monsters;

/// Load all monster types from the index file at `<monster_dir>/monsters.xml`.
///
/// Mirrors C++ `Monsters::loadFromXml(false)`:
///   1. Parse `monsters.xml` index → name→file mappings
///   2. Load each individual `<file>` relative to `monster_dir`
///   3. Register successfully-parsed types; warn and skip on parse errors
///
/// Returns the populated registry, or an error if the index file is missing.
pub fn load_monsters_xml(monster_dir: &Path) -> Result<Monsters, String> {
    let index_path = monster_dir.join("monsters.xml");
    let xml = std::fs::read_to_string(&index_path)
        .map_err(|e| format!("Cannot read {}: {e}", index_path.display()))?;
    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| format!("XML error in {}: {e}", index_path.display()))?;

    let mut monsters = Monsters::new();

    let root = doc
        .descendants()
        .find(|n| n.has_tag_name("monsters"))
        .ok_or_else(|| format!("No <monsters> root in {}", index_path.display()))?;

    for node in root.children().filter(|n| n.has_tag_name("monster")) {
        let name = match node.attribute("name") {
            Some(n) => n.to_owned(),
            None => continue,
        };
        let file = match node.attribute("file") {
            Some(f) => f.to_owned(),
            None => continue,
        };

        let full_path = monster_dir.join(&file);
        let monster_xml = match std::fs::read_to_string(&full_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "[Warning] load_monsters_xml: skipping '{}': {e}",
                    full_path.display()
                );
                continue;
            }
        };

        match Monsters::parse_monster_xml(&monster_xml) {
            Ok(mt) => monsters.add_monster_type(&name, mt),
            Err(e) => eprintln!(
                "[Warning] load_monsters_xml: skipping '{}': {e}",
                full_path.display()
            ),
        }
    }

    Ok(monsters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(dir: &std::path::Path, name: &str, content: &str) {
        let mut f = std::fs::File::create(dir.join(name)).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn load_monsters_xml_missing_index_returns_error() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = load_monsters_xml(tmp.path());
        assert!(result.is_err(), "missing index must return error");
        let Err(e) = result else { panic!() };
        assert!(e.contains("monsters.xml"), "error should mention monsters.xml, got: {e}");
    }

    #[test]
    fn load_monsters_xml_empty_index_returns_empty_registry() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_file(tmp.path(), "monsters.xml", r#"<?xml version="1.0"?><monsters/>"#);
        let monsters = load_monsters_xml(tmp.path()).expect("empty index should succeed");
        assert_eq!(monsters.get_monster_count(), 0);
    }

    #[test]
    fn load_monsters_xml_loads_named_entries() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_file(
            tmp.path(),
            "monsters.xml",
            r#"<?xml version="1.0"?><monsters>
                <monster name="Rat" file="rat.xml"/>
            </monsters>"#,
        );
        write_file(
            tmp.path(),
            "rat.xml",
            r#"<?xml version="1.0"?>
            <monster name="Rat" experience="5" speed="160">
                <health now="20" max="20"/>
                <look type="21"/>
            </monster>"#,
        );
        let monsters = load_monsters_xml(tmp.path()).expect("rat load should succeed");
        assert_eq!(monsters.get_monster_count(), 1);
        assert!(monsters.get_monster_type("Rat").is_some());
    }

    #[test]
    fn load_monsters_xml_skips_missing_files_with_warning() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_file(
            tmp.path(),
            "monsters.xml",
            r#"<?xml version="1.0"?><monsters>
                <monster name="Ghost" file="ghost.xml"/>
            </monsters>"#,
        );
        // ghost.xml not created → should warn and return empty registry
        let monsters = load_monsters_xml(tmp.path()).expect("missing entry should not fail");
        assert_eq!(monsters.get_monster_count(), 0);
    }

    #[test]
    fn load_monsters_xml_skips_entries_with_bad_xml() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_file(
            tmp.path(),
            "monsters.xml",
            r#"<?xml version="1.0"?><monsters>
                <monster name="Bad" file="bad.xml"/>
            </monsters>"#,
        );
        write_file(tmp.path(), "bad.xml", "not xml at all <<<<");
        let monsters = load_monsters_xml(tmp.path()).expect("bad xml entry should not fail");
        assert_eq!(monsters.get_monster_count(), 0);
    }
}
