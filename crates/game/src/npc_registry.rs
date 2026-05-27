use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcDef {
    pub name: String,
    pub look_type: u16,
    pub walk_speed: u16,
    pub script_name: Option<String>,
}

#[derive(Debug, Default)]
pub struct NpcRegistry {
    npcs: HashMap<String, NpcDef>,
}

impl NpcRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, npc: NpcDef) {
        self.npcs.insert(npc.name.clone(), npc);
    }

    pub fn get(&self, name: &str) -> Option<&NpcDef> {
        self.npcs.get(name)
    }

    pub fn len(&self) -> usize {
        self.npcs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.npcs.is_empty()
    }
}

/// Load all NPC `.xml` files from `dir`.
/// Files that cannot be parsed log a warning; loading continues.
pub fn load_npcs_xml(dir: &Path) -> Result<NpcRegistry, String> {
    let mut registry = NpcRegistry::new();

    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Cannot read NPC dir '{}': {e}", dir.display()))?;

    for entry in entries {
        let path = entry
            .map_err(|e| format!("Error reading dir entry: {e}"))?
            .path();

        if path.extension().is_some_and(|e| e == "xml") {
            match parse_npc_file(&path) {
                Ok(npc) => registry.register(npc),
                Err(e) => eprintln!(
                    "[Warning] load_npcs_xml: skipping '{}': {e}",
                    path.display()
                ),
            }
        }
    }

    Ok(registry)
}

fn parse_npc_file(path: &Path) -> Result<NpcDef, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {e}", path.display()))?;
    let doc = roxmltree::Document::parse(&text)
        .map_err(|e| format!("XML error in '{}': {e}", path.display()))?;

    let npc_node = doc
        .descendants()
        .find(|n| n.has_tag_name("npc"))
        .ok_or_else(|| format!("No <npc> element in '{}'", path.display()))?;

    let name = npc_node
        .attribute("name")
        .ok_or("NPC missing 'name'")?
        .to_string();

    let script_name = npc_node.attribute("script").map(str::to_string);

    if script_name.is_none() {
        eprintln!(
            "[Warning] NPC '{name}' in '{}' has no script",
            path.display()
        );
    }

    let look_node = npc_node.children().find(|n| n.has_tag_name("look"));
    let look_type: u16 = look_node
        .and_then(|n| n.attribute("type"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let walk_speed: u16 = npc_node
        .attribute("walkinterval")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    Ok(NpcDef {
        name,
        look_type,
        walk_speed,
        script_name,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_dir(rel: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(rel)
    }

    /// Path to the forgottenserver data directory (via symlink at workspace root).
    fn data_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data") // crates/game → forgottenserver-rust root → data symlink
    }

    // -----------------------------------------------------------------------
    // Test 1: NPC loads name, look_type, and script_name
    // -----------------------------------------------------------------------
    #[test]
    fn npc_xml_loads_name_look_type_and_script_name() {
        let registry = load_npcs_xml(&fixture_dir("npc")).expect("load ok");
        let npc = registry
            .get("Blacksmith")
            .expect("Blacksmith should be registered");

        assert_eq!(npc.name, "Blacksmith");
        assert_eq!(npc.look_type, 40);
        assert_eq!(npc.script_name.as_deref(), Some("bless.lua"));
    }

    // -----------------------------------------------------------------------
    // Test 2: NPC without script logs warning, does NOT return error
    // -----------------------------------------------------------------------
    #[test]
    fn npc_xml_missing_script_logs_warning_not_error() {
        let registry = load_npcs_xml(&fixture_dir("npc")).expect("load ok — no error expected");
        // Wanderer has no script attribute; it must still be registered.
        let wanderer = registry
            .get("Wanderer")
            .expect("Wanderer should be registered despite no script");
        assert!(wanderer.script_name.is_none());
    }

    // -----------------------------------------------------------------------
    // Test 3: Lua script file exists in data directory (Lua scripts sidenote)
    //
    // The forgottenserver-rust/data symlink points to forgottenserver/data.
    // An NPC with script="bless.lua" should resolve to
    // data/npc/scripts/bless.lua — confirming the scripts are reachable.
    //
    // When the scripting crate gains Lua execution support (lua-scripting-wiring),
    // this test should be extended to actually load and run the script.
    // -----------------------------------------------------------------------
    #[test]
    fn npc_xml_referenced_lua_script_file_exists_in_data_dir() {
        let scripts_dir = data_dir().join("npc/scripts");
        let script_path = scripts_dir.join("bless.lua");
        assert!(
            script_path.exists(),
            "Expected Lua script at {}: symlink data/ must point to forgottenserver/data",
            script_path.display()
        );
    }

    // -----------------------------------------------------------------------
    // Test 4: missing directory returns Err
    // -----------------------------------------------------------------------
    #[test]
    fn npc_xml_missing_dir_returns_error() {
        let result = load_npcs_xml(Path::new("/tmp/nonexistent_npc_dir_xyz"));
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // NpcRegistry unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn npc_registry_new_is_empty() {
        assert!(NpcRegistry::new().is_empty());
    }

    #[test]
    fn npc_registry_register_and_get() {
        let mut r = NpcRegistry::new();
        r.register(NpcDef {
            name: "Test".into(),
            look_type: 99,
            walk_speed: 1000,
            script_name: Some("test.lua".into()),
        });
        let npc = r.get("Test").unwrap();
        assert_eq!(npc.look_type, 99);
        assert_eq!(npc.script_name.as_deref(), Some("test.lua"));
    }
}
