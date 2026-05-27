use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellDef {
    pub name: String,
    pub words: String,
    pub mana_cost: u32,
    pub min_level: u32,
    pub vocations: Vec<String>,
    pub element: Option<String>,
}

#[derive(Debug, Default)]
pub struct SpellRegistry {
    spells: HashMap<String, SpellDef>,
}

impl SpellRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, spell: SpellDef) {
        self.spells.insert(spell.words.clone(), spell);
    }

    pub fn get_by_words(&self, words: &str) -> Option<&SpellDef> {
        self.spells.get(words)
    }

    pub fn len(&self) -> usize {
        self.spells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spells.is_empty()
    }
}

pub fn load_spells_xml(path: &Path) -> Result<SpellRegistry, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {e}", path.display()))?;
    let doc = roxmltree::Document::parse(&text)
        .map_err(|e| format!("XML parse error in '{}': {e}", path.display()))?;

    let root = doc
        .descendants()
        .find(|n| n.has_tag_name("spells"))
        .ok_or_else(|| format!("Missing <spells> root in '{}'", path.display()))?;

    let mut registry = SpellRegistry::new();

    for node in root.children().filter(|n| n.is_element()) {
        if !node.has_tag_name("spell") {
            eprintln!(
                "[Warning] load_spells_xml: unknown element <{}>, skipping",
                node.tag_name().name()
            );
            continue;
        }

        match parse_spell_node(&node) {
            Ok(spell) => registry.register(spell),
            Err(e) => eprintln!("[Warning] load_spells_xml: skipping spell: {e}"),
        }
    }

    Ok(registry)
}

fn parse_spell_node(node: &roxmltree::Node) -> Result<SpellDef, String> {
    let name = node
        .attribute("name")
        .ok_or("spell missing 'name'")?
        .to_string();
    let words = node
        .attribute("words")
        .ok_or_else(|| format!("spell '{name}' missing 'words'"))?
        .to_string();
    let mana_cost: u32 = node
        .attribute("mana")
        .unwrap_or("0")
        .parse()
        .map_err(|_| format!("spell '{name}' has invalid mana"))?;
    let min_level: u32 = node
        .attribute("level")
        .unwrap_or("0")
        .parse()
        .map_err(|_| format!("spell '{name}' has invalid level"))?;
    let element = node.attribute("element").map(str::to_string);

    let vocations: Vec<String> = node
        .children()
        .filter(|n| n.is_element() && n.has_tag_name("vocation"))
        .filter_map(|n| n.attribute("name").map(str::to_string))
        .collect();

    Ok(SpellDef {
        name,
        words,
        mana_cost,
        min_level,
        vocations,
        element,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(rel: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(rel)
    }

    // -----------------------------------------------------------------------
    // Test 1: spell loaded with mana cost and level requirement
    // -----------------------------------------------------------------------
    #[test]
    fn spells_xml_loads_spell_with_mana_cost_and_level_requirement() {
        let registry = load_spells_xml(&fixture("spells.xml")).expect("load ok");
        let spell = registry
            .get_by_words("exevo gran mas flam")
            .expect("Fire Ball should be registered");

        assert_eq!(spell.name, "Fire Ball");
        assert_eq!(spell.mana_cost, 110);
        assert_eq!(spell.min_level, 8);
        assert_eq!(spell.element.as_deref(), Some("fire"));
        assert!(spell.vocations.contains(&"Sorcerer".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test 2: unknown element logs warning and continues
    // -----------------------------------------------------------------------
    #[test]
    fn spells_xml_unknown_element_logs_warning_and_continues() {
        // The fixture has a valid spell + an <unknown-type/> child.
        // The loader must not fail; both valid spells must be present.
        let registry = load_spells_xml(&fixture("spells.xml")).expect("must not error");
        assert_eq!(
            registry.len(),
            2,
            "only the 2 <spell> entries should be registered"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3: multiple vocations parsed
    // -----------------------------------------------------------------------
    #[test]
    fn spells_xml_multiple_vocations_parsed() {
        let registry = load_spells_xml(&fixture("spells.xml")).expect("load ok");
        let spell = registry
            .get_by_words("exevo gran mas flam")
            .expect("Fire Ball");
        assert!(spell.vocations.contains(&"Master Sorcerer".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test 4: missing file returns Err
    // -----------------------------------------------------------------------
    #[test]
    fn spells_xml_missing_file_returns_error() {
        let result = load_spells_xml(Path::new("/tmp/nonexistent_spells_xyz.xml"));
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // SpellRegistry unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn spell_registry_new_is_empty() {
        let r = SpellRegistry::new();
        assert!(r.is_empty());
    }

    #[test]
    fn spell_registry_register_and_get() {
        let mut r = SpellRegistry::new();
        r.register(SpellDef {
            name: "Test".into(),
            words: "exura".into(),
            mana_cost: 20,
            min_level: 1,
            vocations: vec![],
            element: None,
        });
        let s = r.get_by_words("exura").expect("should exist");
        assert_eq!(s.name, "Test");
        assert_eq!(s.mana_cost, 20);
    }
}
