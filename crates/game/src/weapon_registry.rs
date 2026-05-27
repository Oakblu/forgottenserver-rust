use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    Melee,
    Distance,
    Wand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponDef {
    pub item_id: u16,
    pub weapon_type: WeaponType,
    pub attack: i32,
    pub defense: i32,
    pub min_level: u32,
    pub vocations: Vec<String>,
    pub element: Option<String>,
}

#[derive(Debug, Default)]
pub struct WeaponRegistry {
    weapons: HashMap<u16, WeaponDef>,
}

impl WeaponRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, weapon: WeaponDef) {
        self.weapons.insert(weapon.item_id, weapon);
    }

    pub fn get(&self, item_id: u16) -> Option<&WeaponDef> {
        self.weapons.get(&item_id)
    }

    pub fn len(&self) -> usize {
        self.weapons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.weapons.is_empty()
    }
}

pub struct CombatResolver<'a> {
    weapon_registry: &'a WeaponRegistry,
}

impl<'a> CombatResolver<'a> {
    pub fn new(weapon_registry: &'a WeaponRegistry) -> Self {
        Self { weapon_registry }
    }

    /// Return the attack value for `item_id`, falling back to `default_attack`
    /// when the item is not registered or has no weapon stats.
    pub fn compute_melee(&self, item_id: u16, default_attack: i32) -> i32 {
        self.weapon_registry
            .get(item_id)
            .map(|w| w.attack)
            .unwrap_or(default_attack)
    }
}

pub fn load_weapons_xml(path: &Path) -> Result<WeaponRegistry, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {e}", path.display()))?;
    let doc = roxmltree::Document::parse(&text)
        .map_err(|e| format!("XML parse error in '{}': {e}", path.display()))?;

    let root = doc
        .descendants()
        .find(|n| n.has_tag_name("weapons"))
        .ok_or_else(|| format!("Missing <weapons> root in '{}'", path.display()))?;

    let mut registry = WeaponRegistry::new();

    for node in root.children().filter(|n| n.is_element()) {
        let weapon_type = match node.tag_name().name() {
            "melee" => WeaponType::Melee,
            "distance" => WeaponType::Distance,
            "wand" => WeaponType::Wand,
            other => {
                eprintln!("[Warning] load_weapons_xml: unknown element <{other}>, skipping");
                continue;
            }
        };

        match parse_weapon_node(&node, weapon_type) {
            Ok(weapon) => registry.register(weapon),
            Err(e) => eprintln!("[Warning] load_weapons_xml: skipping weapon: {e}"),
        }
    }

    Ok(registry)
}

fn parse_weapon_node(node: &roxmltree::Node, weapon_type: WeaponType) -> Result<WeaponDef, String> {
    let item_id: u16 = node
        .attribute("id")
        .ok_or("weapon missing 'id'")?
        .parse()
        .map_err(|_| "weapon 'id' is not a valid u16")?;

    let attack: i32 = node.attribute("attack").unwrap_or("0").parse().unwrap_or(0);
    let defense: i32 = node
        .attribute("defense")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    let min_level: u32 = node.attribute("level").unwrap_or("0").parse().unwrap_or(0);
    let element = node.attribute("type").map(str::to_string);

    let vocations: Vec<String> = node
        .children()
        .filter(|n| n.is_element() && n.has_tag_name("vocation"))
        .filter_map(|n| n.attribute("name").map(str::to_string))
        .collect();

    Ok(WeaponDef {
        item_id,
        weapon_type,
        attack,
        defense,
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
    // Test 1: sword (melee) with attack, defense, vocation gate
    // -----------------------------------------------------------------------
    #[test]
    fn weapons_xml_loads_sword_with_attack_defense_vocation_gate() {
        let registry = load_weapons_xml(&fixture("weapons.xml")).expect("load ok");
        let weapon = registry
            .get(2376)
            .expect("item 2376 (melee) should be loaded");

        assert_eq!(weapon.weapon_type, WeaponType::Melee);
        assert_eq!(weapon.attack, 30);
        assert_eq!(weapon.defense, 15);
        assert_eq!(weapon.min_level, 20);
        assert!(weapon.vocations.contains(&"Paladin".to_string()));
        assert!(weapon.vocations.contains(&"Royal Paladin".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test 2: bow (distance) loaded
    // -----------------------------------------------------------------------
    #[test]
    fn weapons_xml_loads_bow() {
        let registry = load_weapons_xml(&fixture("weapons.xml")).expect("load ok");
        let bow = registry
            .get(2456)
            .expect("item 2456 (distance) should be loaded");
        assert_eq!(bow.weapon_type, WeaponType::Distance);
        assert_eq!(bow.attack, 50);
        assert_eq!(bow.min_level, 30);
    }

    // -----------------------------------------------------------------------
    // Test 3: wand loaded with element type
    // -----------------------------------------------------------------------
    #[test]
    fn weapons_xml_loads_wand_with_element() {
        let registry = load_weapons_xml(&fixture("weapons.xml")).expect("load ok");
        let wand = registry
            .get(2180)
            .expect("item 2180 (wand) should be loaded");
        assert_eq!(wand.weapon_type, WeaponType::Wand);
        assert_eq!(wand.element.as_deref(), Some("earth"));
    }

    // -----------------------------------------------------------------------
    // Test 4: CombatResolver uses registry stats
    // -----------------------------------------------------------------------
    #[test]
    fn combat_resolver_compute_melee_uses_registry() {
        let registry = load_weapons_xml(&fixture("weapons.xml")).expect("load ok");
        let resolver = CombatResolver::new(&registry);

        // known item → uses weapon stats
        assert_eq!(resolver.compute_melee(2376, 5), 30);
        // unknown item → falls back to default
        assert_eq!(resolver.compute_melee(9999, 7), 7);
    }

    // -----------------------------------------------------------------------
    // Test 5: missing file returns Err
    // -----------------------------------------------------------------------
    #[test]
    fn weapons_xml_missing_file_returns_error() {
        let result = load_weapons_xml(Path::new("/tmp/nonexistent_weapons_xyz.xml"));
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // WeaponRegistry unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn weapon_registry_new_is_empty() {
        assert!(WeaponRegistry::new().is_empty());
    }

    #[test]
    fn weapon_registry_register_and_get() {
        let mut r = WeaponRegistry::new();
        r.register(WeaponDef {
            item_id: 1,
            weapon_type: WeaponType::Melee,
            attack: 25,
            defense: 10,
            min_level: 0,
            vocations: vec![],
            element: None,
        });
        assert_eq!(r.get(1).unwrap().attack, 25);
    }
}
