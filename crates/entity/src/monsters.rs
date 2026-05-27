//! Migrated from forgottenserver/src/monsters.h and monsters.cpp
//! MonsterType registry and minimal XML parser using roxmltree.

use std::collections::HashMap;

use crate::monster::LootBlock;

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// A single voice utterance for a monster.
/// Mirrors `voiceBlock_t` in C++.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceBlock {
    /// The text of the utterance.
    pub text: String,
    /// Whether the monster shouts (true) or speaks normally (false).
    pub yell: bool,
}

/// A summon entry — describes a creature the monster can summon.
/// Mirrors `summonBlock_t` in C++.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummonBlock {
    /// Name of the creature type to summon.
    pub name: String,
    /// Spawn interval in ms. Default 1000.
    pub speed: u32,
    /// Probability 0–100. Default 100.
    pub chance: u32,
    /// Maximum number of this type that can be summoned at once.
    pub max: u32,
    /// Whether the summon is forced even if the monster limit is reached.
    pub force: bool,
}

/// An attack or defense spell entry (simplified — only name + chance used for now).
/// Mirrors `spellBlock_t` in C++.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellBlock {
    /// Spell name (as read from XML `name` attribute).
    pub name: String,
    /// Probability 0–100. Default 100.
    pub chance: u32,
    /// Interval in ms. Default 2000.
    pub speed: u32,
    /// Minimum combat value (negative = damage).
    pub min_combat_value: i32,
    /// Maximum combat value.
    pub max_combat_value: i32,
    /// True when this is a melee attack.
    pub is_melee: bool,
}

/// A single loot item entry parsed from XML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootEntry {
    /// Item type id.
    pub id: u16,
    /// Maximum stack count to drop. Minimum 1.
    pub count_max: u32,
    /// Drop chance out of 100_000 (MAX_LOOTCHANCE).
    pub chance: u32,
    /// Optional sub-type / charges.
    pub sub_type: Option<i32>,
    /// Optional action id.
    pub action_id: Option<i32>,
    /// Optional text on the item.
    pub text: Option<String>,
    /// Child loot items when this entry is a container.
    pub child_loot: Vec<LootEntry>,
}

// ---------------------------------------------------------------------------
// MonsterType
// ---------------------------------------------------------------------------

/// Static description of a monster kind, loaded from XML.
/// Mirrors the relevant fields from the C++ `MonsterType` / `MonsterInfo`.
#[derive(Debug, Clone)]
pub struct MonsterType {
    // --- identity ---
    pub name: String,
    /// Human-readable description, e.g. "a rat". Defaults to "a <lowercase name>".
    pub name_description: String,

    // --- stats ---
    /// Current (initial) health when spawned. Mirrors `info.health`.
    pub health: i32,
    /// Maximum health. Mirrors `info.healthMax`.
    pub max_health: i32,
    /// Base movement speed. Mirrors `info.baseSpeed`. Default 200.
    pub base_speed: u32,
    /// Experience awarded on death.
    pub experience: u64,
    /// Mana cost to convince/summon this monster.
    pub mana_cost: u32,

    // --- combat ---
    /// Flat armor value. Mirrors `info.armor`.
    pub armor: i32,
    /// Flat defense value. Mirrors `info.defense`.
    pub defense: i32,
    /// Bitfield of damage immunity flags. Mirrors `info.damageImmunities`.
    pub immunity_flags: u32,
    /// Probability (0–100) to keep attacking the same target (staticAttackChance). Default 95.
    pub static_attack_chance: u32,
    /// Health threshold below which the monster flees. Mirrors `runAwayHealth`.
    pub run_away_health: i32,
    /// Preferred engagement distance (≥ 1). Default 1.
    pub target_distance: i32,
    /// Interval (ms) between target-change checks.
    pub change_target_speed: u32,
    /// Probability (0–100) of changing target on each check.
    pub change_target_chance: i32,

    // --- look ---
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_type_ex: u16,

    // --- voices ---
    /// Minimum ms between voice utterances.
    pub yell_speed_ticks: u32,
    /// Probability (0–100) of uttering a voice on each tick.
    pub yell_chance: u32,
    pub voices: Vec<VoiceBlock>,

    // --- summons ---
    /// Maximum concurrent summons (capped at 100 in C++).
    pub max_summons: u32,
    pub summons: Vec<SummonBlock>,

    // --- spells ---
    pub attack_spells: Vec<SpellBlock>,
    pub defense_spells: Vec<SpellBlock>,

    // --- loot ---
    pub loot_table: Vec<LootBlock>,
    /// Parsed loot entries from XML (richer than LootBlock).
    pub loot_entries: Vec<LootEntry>,

    // --- boolean flags ---
    pub is_summonable: bool,
    pub is_attackable: bool,
    pub is_hostile: bool,
    pub is_pushable: bool,
    pub can_push_items: bool,
    pub can_push_creatures: bool,
    pub is_boss: bool,
    pub is_challengeable: bool,
    pub is_convinceable: bool,
    pub is_illusionable: bool,
    pub is_ignoring_spawn_block: bool,
    pub hidden_health: bool,
    pub can_walk_on_energy: bool,
    pub can_walk_on_fire: bool,
    pub can_walk_on_poison: bool,
}

impl Default for MonsterType {
    fn default() -> Self {
        MonsterType {
            name: String::new(),
            name_description: String::new(),
            health: 100,
            max_health: 100,
            base_speed: 200,
            experience: 0,
            mana_cost: 0,
            armor: 0,
            defense: 0,
            immunity_flags: 0,
            static_attack_chance: 95,
            run_away_health: 0,
            target_distance: 1,
            change_target_speed: 0,
            change_target_chance: 0,
            look_type: 0,
            look_head: 0,
            look_body: 0,
            look_legs: 0,
            look_feet: 0,
            look_type_ex: 0,
            yell_speed_ticks: 0,
            yell_chance: 0,
            voices: Vec::new(),
            max_summons: 0,
            summons: Vec::new(),
            attack_spells: Vec::new(),
            defense_spells: Vec::new(),
            loot_table: Vec::new(),
            loot_entries: Vec::new(),
            is_summonable: false,
            is_attackable: true,
            is_hostile: true,
            is_pushable: true,
            can_push_items: false,
            can_push_creatures: false,
            is_boss: false,
            is_challengeable: true,
            is_convinceable: false,
            is_illusionable: false,
            is_ignoring_spawn_block: false,
            hidden_health: false,
            can_walk_on_energy: true,
            can_walk_on_fire: true,
            can_walk_on_poison: true,
        }
    }
}

impl MonsterType {
    pub fn new() -> Self {
        MonsterType::default()
    }
}

// ---------------------------------------------------------------------------
// XML parse error
// ---------------------------------------------------------------------------

/// Errors that can occur while parsing a monster XML fragment.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The XML itself is malformed.
    XmlError(String),
    /// A required attribute or element is missing.
    MissingAttribute(String),
    /// The root element is not `<monster>`.
    WrongRoot(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::XmlError(s) => write!(f, "XML parse error: {s}"),
            ParseError::MissingAttribute(s) => write!(f, "missing attribute: {s}"),
            ParseError::WrongRoot(s) => write!(f, "wrong root element: {s}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Monsters — registry
// ---------------------------------------------------------------------------

/// Registry of all loaded monster types.
pub struct Monsters {
    /// Keyed by lower-cased name for case-insensitive lookup.
    types: HashMap<String, MonsterType>,
}

impl Monsters {
    pub fn new() -> Self {
        Monsters {
            types: HashMap::new(),
        }
    }

    /// Register a monster type. The name is stored exactly as given but
    /// lookups are case-insensitive (key = name.to_lowercase()).
    pub fn register(&mut self, mt: MonsterType) {
        self.types.insert(mt.name.to_lowercase(), mt);
    }

    /// Add a monster type under an explicit key (case-insensitive).
    /// Mirrors `addMonsterType` in C++.
    pub fn add_monster_type(&mut self, name: &str, mt: MonsterType) {
        self.types.insert(name.to_lowercase(), mt);
    }

    /// Look up a monster type by name (case-insensitive).
    /// Mirrors `getMonsterType(name)` in C++.
    pub fn get_monster_type(&self, name: &str) -> Option<&MonsterType> {
        self.types.get(&name.to_lowercase())
    }

    /// Alias for `get_monster_type` — mirrors `getMonsterTypeByName` in C++.
    pub fn get_monster_type_by_name(&self, name: &str) -> Option<&MonsterType> {
        self.get_monster_type(name)
    }

    /// Total number of registered monster types.
    pub fn get_monster_count(&self) -> usize {
        self.types.len()
    }

    // -----------------------------------------------------------------------
    // XML parser — parses a single <monster> element from a string.
    // -----------------------------------------------------------------------

    /// Parse a `<monster>` XML fragment into a `MonsterType`.
    ///
    /// Recognised attributes on `<monster>`:
    ///   name, experience, speed, manacost, nameDescription
    ///
    /// Recognised child elements:
    ///   `<health now="N" max="N"/>` → health, max_health
    ///   `<look type="N" head="N" body="N" legs="N" feet="N" typeex="N"/>`
    ///   `<voices speed/interval="N" chance="N">` + `<voice sentence="…" yell="0/1"/>`
    ///   `<summons maxSummons="N">` + `<summon name="…" speed="N" chance="N" max="N" force="0/1"/>`
    ///   `<defenses defense="N" armor="N">` (+ optional spell children, stored by name)
    ///   `<attacks>` + `<attack name="…" chance="N" speed/interval="N" min="N" max="N"/>`
    ///   `<loot>` + `<item id="N" countmax="N" chance="N"/>`
    ///   `<flags>` + `<flag <attrname>="<val>"/>`
    ///   `<targetchange speed/interval="N" chance="N"/>`
    ///
    /// Returns `Err(ParseError)` on XML parse failure or missing required fields.
    pub fn parse_monster_type_from_xml(xml: &str) -> Result<MonsterType, String> {
        Self::parse_monster_xml(xml).map_err(|e| e.to_string())
    }

    /// Full parse returning a typed `ParseError`.
    pub fn parse_monster_xml(xml: &str) -> Result<MonsterType, ParseError> {
        let doc =
            roxmltree::Document::parse(xml).map_err(|e| ParseError::XmlError(e.to_string()))?;

        let root = doc.root_element();
        if root.tag_name().name() != "monster" {
            return Err(ParseError::WrongRoot(root.tag_name().name().to_string()));
        }

        let name = root
            .attribute("name")
            .ok_or_else(|| ParseError::MissingAttribute("name".to_string()))?
            .to_string();

        let experience = root
            .attribute("experience")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let base_speed = root
            .attribute("speed")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(200);

        let mana_cost = root
            .attribute("manacost")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        let name_description = root
            .attribute("nameDescription")
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("a {}", name.to_lowercase()));

        let mut mt = MonsterType {
            name,
            name_description,
            experience,
            base_speed,
            mana_cost,
            ..MonsterType::default()
        };

        for child in root.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "health" => {
                    parse_health(child, &mut mt);
                }
                "look" => {
                    parse_look(child, &mut mt);
                }
                "voices" => {
                    parse_voices(child, &mut mt);
                }
                "summons" => {
                    parse_summons(child, &mut mt);
                }
                "defenses" => {
                    parse_defenses(child, &mut mt);
                }
                "attacks" => {
                    parse_attacks(child, &mut mt);
                }
                "loot" => {
                    parse_loot(child, &mut mt);
                }
                "flags" => {
                    parse_flags(child, &mut mt);
                }
                "targetchange" => {
                    parse_targetchange(child, &mut mt);
                }
                _ => {}
            }
        }

        Ok(mt)
    }
}

// ---------------------------------------------------------------------------
// Private XML section parsers
// ---------------------------------------------------------------------------

fn parse_health(node: roxmltree::Node, mt: &mut MonsterType) {
    if let Some(v) = node.attribute("now").and_then(|s| s.parse::<i32>().ok()) {
        mt.health = v;
    }
    if let Some(v) = node.attribute("max").and_then(|s| s.parse::<i32>().ok()) {
        mt.max_health = v;
    }
    // C++ clamps health to max_health if health > max_health
    if mt.health > mt.max_health {
        mt.health = mt.max_health;
    }
}

fn parse_look(node: roxmltree::Node, mt: &mut MonsterType) {
    if let Some(v) = node.attribute("type").and_then(|s| s.parse::<u16>().ok()) {
        mt.look_type = v;
    } else if let Some(v) = node.attribute("typeex").and_then(|s| s.parse::<u16>().ok()) {
        mt.look_type_ex = v;
    }
    if let Some(v) = node.attribute("head").and_then(|s| s.parse::<u8>().ok()) {
        mt.look_head = v;
    }
    if let Some(v) = node.attribute("body").and_then(|s| s.parse::<u8>().ok()) {
        mt.look_body = v;
    }
    if let Some(v) = node.attribute("legs").and_then(|s| s.parse::<u8>().ok()) {
        mt.look_legs = v;
    }
    if let Some(v) = node.attribute("feet").and_then(|s| s.parse::<u8>().ok()) {
        mt.look_feet = v;
    }
}

fn parse_voices(node: roxmltree::Node, mt: &mut MonsterType) {
    // Read speed or interval attribute
    let speed = node
        .attribute("speed")
        .or_else(|| node.attribute("interval"))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    mt.yell_speed_ticks = speed;

    let chance = node
        .attribute("chance")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    mt.yell_chance = chance.min(100);

    for voice_node in node.children().filter(|n| n.is_element()) {
        if voice_node.tag_name().name() != "voice" {
            continue;
        }
        let text = voice_node.attribute("sentence").unwrap_or("").to_string();
        let yell = voice_node
            .attribute("yell")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        mt.voices.push(VoiceBlock { text, yell });
    }
}

fn parse_summons(node: roxmltree::Node, mt: &mut MonsterType) {
    let max_summons = node
        .attribute("maxSummons")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
        .min(100);
    mt.max_summons = max_summons;

    for summon_node in node.children().filter(|n| n.is_element()) {
        if summon_node.tag_name().name() != "summon" {
            continue;
        }
        let name = match summon_node.attribute("name") {
            Some(n) => n.to_string(),
            None => continue,
        };
        let speed = summon_node
            .attribute("speed")
            .or_else(|| summon_node.attribute("interval"))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1000)
            .max(1);
        let chance = summon_node
            .attribute("chance")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(100)
            .min(100);
        let max = summon_node
            .attribute("max")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(max_summons);
        let force = summon_node
            .attribute("force")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        mt.summons.push(SummonBlock {
            name,
            speed,
            chance,
            max,
            force,
        });
    }
}

fn parse_defenses(node: roxmltree::Node, mt: &mut MonsterType) {
    if let Some(v) = node
        .attribute("defense")
        .and_then(|s| s.parse::<i32>().ok())
    {
        mt.defense = v;
    }
    if let Some(v) = node.attribute("armor").and_then(|s| s.parse::<i32>().ok()) {
        mt.armor = v;
    }
    // Parse defense spell children
    for spell_node in node.children().filter(|n| n.is_element()) {
        if let Some(sb) = parse_spell_block(spell_node) {
            mt.defense_spells.push(sb);
        }
    }
}

fn parse_attacks(node: roxmltree::Node, mt: &mut MonsterType) {
    for spell_node in node.children().filter(|n| n.is_element()) {
        if let Some(sb) = parse_spell_block(spell_node) {
            mt.attack_spells.push(sb);
        }
    }
}

/// Parse a spell child node (e.g. `<attack>` or `<defense>` spell entry).
fn parse_spell_block(node: roxmltree::Node) -> Option<SpellBlock> {
    let name = node.attribute("name")?.to_string();
    let chance = node
        .attribute("chance")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(100)
        .min(100);
    let speed = node
        .attribute("speed")
        .or_else(|| node.attribute("interval"))
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(2000)
        .max(1);

    let mut min_combat_value = node
        .attribute("min")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    let mut max_combat_value = node
        .attribute("max")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    // Normalise: |min| must be ≤ |max| (mirror C++ logic)
    if min_combat_value.unsigned_abs() > max_combat_value.unsigned_abs() {
        std::mem::swap(&mut min_combat_value, &mut max_combat_value);
    }

    let is_melee = name.to_lowercase() == "melee";
    Some(SpellBlock {
        name,
        chance,
        speed,
        min_combat_value,
        max_combat_value,
        is_melee,
    })
}

fn parse_loot(node: roxmltree::Node, mt: &mut MonsterType) {
    const MAX_LOOT_CHANCE: u32 = 100_000;

    for item_node in node.children().filter(|n| n.is_element()) {
        // Only handle <item> children
        if item_node.tag_name().name() != "item" {
            continue;
        }
        let id = match item_node
            .attribute("id")
            .and_then(|s| s.parse::<u16>().ok())
        {
            Some(v) if v > 0 => v,
            _ => continue,
        };
        let count_max = item_node
            .attribute("countmax")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1)
            .max(1);
        let chance = item_node
            .attribute("chance")
            .or_else(|| item_node.attribute("chance1"))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(MAX_LOOT_CHANCE)
            .min(MAX_LOOT_CHANCE);
        let sub_type = item_node
            .attribute("subtype")
            .and_then(|s| s.parse::<i32>().ok());
        let action_id = item_node
            .attribute("actionId")
            .and_then(|s| s.parse::<i32>().ok());
        let text = item_node.attribute("text").map(|s| s.to_string());

        mt.loot_entries.push(LootEntry {
            id,
            count_max,
            chance,
            sub_type,
            action_id,
            text,
            child_loot: Vec::new(),
        });
    }
}

fn parse_flags(node: roxmltree::Node, mt: &mut MonsterType) {
    for flag_node in node.children().filter(|n| n.is_element()) {
        // Each <flag> has one attribute — the flag name with its value.
        for attr in flag_node.attributes() {
            let key = attr.name().to_lowercase();
            let val = attr.value();
            let bool_val = val == "1" || val.eq_ignore_ascii_case("true");
            match key.as_str() {
                "summonable" => mt.is_summonable = bool_val,
                "attackable" => mt.is_attackable = bool_val,
                "hostile" => mt.is_hostile = bool_val,
                "ignorespawnblock" => mt.is_ignoring_spawn_block = bool_val,
                "illusionable" => mt.is_illusionable = bool_val,
                "challengeable" => mt.is_challengeable = bool_val,
                "convinceable" => mt.is_convinceable = bool_val,
                "pushable" => mt.is_pushable = bool_val,
                "isboss" => mt.is_boss = bool_val,
                "canpushitems" => mt.can_push_items = bool_val,
                "canpushcreatures" => mt.can_push_creatures = bool_val,
                "staticattack" => {
                    if let Ok(v) = val.parse::<u32>() {
                        mt.static_attack_chance = v.min(100);
                    }
                }
                "targetdistance" => {
                    if let Ok(v) = val.parse::<i32>() {
                        mt.target_distance = v.max(1);
                    }
                }
                "runonhealth" => {
                    if let Ok(v) = val.parse::<i32>() {
                        mt.run_away_health = v;
                    }
                }
                "hidehealth" => mt.hidden_health = bool_val,
                "canwalkonenergy" => mt.can_walk_on_energy = bool_val,
                "canwalkonfire" => mt.can_walk_on_fire = bool_val,
                "canwalkonpoison" => mt.can_walk_on_poison = bool_val,
                _ => {}
            }
        }
    }
    // C++ rule: if canPushCreatures, then pushable = false
    if mt.can_push_creatures {
        mt.is_pushable = false;
    }
}

fn parse_targetchange(node: roxmltree::Node, mt: &mut MonsterType) {
    if let Some(v) = node
        .attribute("speed")
        .or_else(|| node.attribute("interval"))
        .and_then(|s| s.parse::<u32>().ok())
    {
        mt.change_target_speed = v;
    }
    if let Some(v) = node.attribute("chance").and_then(|s| s.parse::<i32>().ok()) {
        mt.change_target_chance = v.min(100);
    }
}

impl Default for Monsters {
    fn default() -> Self {
        Monsters::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------------
    // Fixture XMLs
    // ---------------------------------------------------------------------------

    /// Minimal valid monster XML — just name + health.
    const MINIMAL_XML: &str = r#"<monster name="Rat">
  <health now="30" max="30"/>
</monster>"#;

    /// Full-featured monster XML exercising all parseable sections.
    const RAT_XML: &str = r#"<monster name="Rat" experience="5" speed="100" manacost="0">
  <health now="30" max="30"/>
  <look type="21" head="0" body="0" legs="0" feet="0" addons="0"/>
  <targetchange interval="2000" chance="10"/>
  <flags>
    <flag summonable="0"/>
    <flag attackable="1"/>
    <flag hostile="1"/>
    <flag pushable="1"/>
    <flag canpushitems="0"/>
    <flag canpushcreatures="0"/>
    <flag isboss="0"/>
  </flags>
  <voices speed="5000" chance="10">
    <voice sentence="Squeak!" yell="0"/>
    <voice sentence="SQUEAAAK!" yell="1"/>
  </voices>
  <summons maxSummons="3">
    <summon name="Cave Rat" speed="2000" chance="50" max="2" force="0"/>
    <summon name="Cave Rat" speed="1500" chance="30" max="1" force="1"/>
  </summons>
  <defenses defense="5" armor="3">
    <defense name="healing" interval="2000" chance="20" min="10" max="30"/>
  </defenses>
  <attacks>
    <attack name="melee" interval="2000" chance="100" skill="30" attack="25"/>
    <attack name="fire" interval="3000" chance="50" min="-50" max="-80"/>
  </attacks>
  <loot>
    <item id="2148" countmax="10" chance="50000"/>
    <item id="2152" countmax="1" chance="1000" subtype="50" actionId="100" text="Hello"/>
  </loot>
</monster>"#;

    /// XML with health now > max — C++ clamps now to max.
    const HEALTH_CLAMP_XML: &str = r#"<monster name="X">
  <health now="200" max="100"/>
</monster>"#;

    /// XML without the name attribute — must fail.
    const MISSING_NAME_XML: &str = r#"<monster experience="5"><health max="30"/></monster>"#;

    /// Malformed XML — must fail.
    const MALFORMED_XML: &str = "<monster name=\"Rat\"> <unclosed";

    /// Wrong root element.
    const WRONG_ROOT_XML: &str = r#"<foo name="Rat"/>"#;

    // ---------------------------------------------------------------------------
    // MonsterType struct defaults
    // ---------------------------------------------------------------------------

    #[test]
    fn test_monster_type_default_values() {
        let mt = MonsterType::default();
        assert_eq!(mt.name, "");
        assert_eq!(mt.health, 100);
        assert_eq!(mt.max_health, 100);
        assert_eq!(mt.base_speed, 200);
        assert_eq!(mt.experience, 0);
        assert_eq!(mt.mana_cost, 0);
        assert_eq!(mt.armor, 0);
        assert_eq!(mt.defense, 0);
        assert_eq!(mt.immunity_flags, 0);
        assert_eq!(mt.static_attack_chance, 95);
        assert_eq!(mt.target_distance, 1);
        assert_eq!(mt.run_away_health, 0);
        // look fields
        assert_eq!(mt.look_type, 0);
        assert_eq!(mt.look_head, 0);
        assert_eq!(mt.look_body, 0);
        assert_eq!(mt.look_legs, 0);
        assert_eq!(mt.look_feet, 0);
        assert_eq!(mt.look_type_ex, 0);
        // voices
        assert_eq!(mt.yell_speed_ticks, 0);
        assert_eq!(mt.yell_chance, 0);
        assert!(mt.voices.is_empty());
        // summons
        assert_eq!(mt.max_summons, 0);
        assert!(mt.summons.is_empty());
        // spells
        assert!(mt.attack_spells.is_empty());
        assert!(mt.defense_spells.is_empty());
        // loot
        assert!(mt.loot_table.is_empty());
        assert!(mt.loot_entries.is_empty());
        // flags
        assert!(!mt.is_summonable);
        assert!(mt.is_attackable);
        assert!(mt.is_hostile);
        assert!(mt.is_pushable);
        assert!(!mt.can_push_items);
        assert!(!mt.can_push_creatures);
        assert!(!mt.is_boss);
        assert!(mt.is_challengeable);
        assert!(!mt.is_convinceable);
        assert!(!mt.is_illusionable);
        assert!(!mt.is_ignoring_spawn_block);
        assert!(!mt.hidden_health);
        assert!(mt.can_walk_on_energy);
        assert!(mt.can_walk_on_fire);
        assert!(mt.can_walk_on_poison);
    }

    // ---------------------------------------------------------------------------
    // Monsters registry
    // ---------------------------------------------------------------------------

    #[test]
    fn test_monsters_new_is_empty() {
        let m = Monsters::new();
        assert_eq!(m.get_monster_count(), 0);
    }

    #[test]
    fn test_monsters_register_and_get() {
        let mut m = Monsters::new();
        m.register(MonsterType {
            name: "rat".to_string(),
            max_health: 30,
            ..Default::default()
        });
        let found = m.get_monster_type("rat");
        assert!(found.is_some());
        assert_eq!(found.unwrap().max_health, 30);
    }

    #[test]
    fn test_get_monster_type_unknown_returns_none() {
        let m = Monsters::new();
        assert!(m.get_monster_type("ghost").is_none());
    }

    #[test]
    fn test_get_monster_type_case_insensitive() {
        let mut m = Monsters::new();
        m.register(MonsterType {
            name: "Rat".to_string(),
            ..Default::default()
        });
        assert!(m.get_monster_type("RAT").is_some());
        assert!(m.get_monster_type("rat").is_some());
        assert!(m.get_monster_type("rAt").is_some());
    }

    #[test]
    fn test_get_monster_type_by_name_alias() {
        let mut m = Monsters::new();
        m.register(MonsterType {
            name: "Wolf".to_string(),
            max_health: 200,
            ..Default::default()
        });
        let a = m.get_monster_type("wolf");
        let b = m.get_monster_type_by_name("wolf");
        assert_eq!(a.map(|t| t.max_health), b.map(|t| t.max_health));
    }

    #[test]
    fn test_add_monster_type_explicit_key() {
        let mut m = Monsters::new();
        // name in struct doesn't match key
        let mt = MonsterType {
            name: "Internal Name".to_string(),
            max_health: 42,
            ..Default::default()
        };
        m.add_monster_type("wolf", mt);
        // lookup by key, not by internal name
        assert!(m.get_monster_type("Wolf").is_some());
        assert_eq!(m.get_monster_type("wolf").unwrap().max_health, 42);
    }

    #[test]
    fn test_monsters_count_increments() {
        let mut m = Monsters::new();
        assert_eq!(m.get_monster_count(), 0);
        m.register(MonsterType {
            name: "rat".to_string(),
            ..Default::default()
        });
        assert_eq!(m.get_monster_count(), 1);
        m.register(MonsterType {
            name: "wolf".to_string(),
            ..Default::default()
        });
        assert_eq!(m.get_monster_count(), 2);
    }

    #[test]
    fn test_register_overwrites_same_name() {
        let mut m = Monsters::new();
        m.register(MonsterType {
            name: "rat".to_string(),
            max_health: 10,
            ..Default::default()
        });
        m.register(MonsterType {
            name: "rat".to_string(),
            max_health: 99,
            ..Default::default()
        });
        assert_eq!(m.get_monster_count(), 1);
        assert_eq!(m.get_monster_type("rat").unwrap().max_health, 99);
    }

    // ---------------------------------------------------------------------------
    // XML parsing — error paths
    // ---------------------------------------------------------------------------

    #[test]
    fn test_parse_malformed_xml_returns_err() {
        assert!(Monsters::parse_monster_type_from_xml(MALFORMED_XML).is_err());
    }

    #[test]
    fn test_parse_wrong_root_returns_err() {
        assert!(Monsters::parse_monster_type_from_xml(WRONG_ROOT_XML).is_err());
    }

    #[test]
    fn test_parse_missing_name_returns_err() {
        assert!(Monsters::parse_monster_type_from_xml(MISSING_NAME_XML).is_err());
    }

    // ---------------------------------------------------------------------------
    // XML parsing — minimal
    // ---------------------------------------------------------------------------

    #[test]
    fn test_parse_minimal_name() {
        let mt = Monsters::parse_monster_type_from_xml(MINIMAL_XML).unwrap();
        assert_eq!(mt.name, "Rat");
    }

    #[test]
    fn test_parse_minimal_health() {
        let mt = Monsters::parse_monster_type_from_xml(MINIMAL_XML).unwrap();
        assert_eq!(mt.health, 30);
        assert_eq!(mt.max_health, 30);
    }

    #[test]
    fn test_parse_minimal_default_speed() {
        let mt = Monsters::parse_monster_type_from_xml(MINIMAL_XML).unwrap();
        assert_eq!(mt.base_speed, 200); // default
    }

    #[test]
    fn test_parse_minimal_default_experience() {
        let mt = Monsters::parse_monster_type_from_xml(MINIMAL_XML).unwrap();
        assert_eq!(mt.experience, 0); // default
    }

    #[test]
    fn test_parse_name_description_default() {
        let mt = Monsters::parse_monster_type_from_xml(MINIMAL_XML).unwrap();
        // default is "a <lowercase name>"
        assert_eq!(mt.name_description, "a rat");
    }

    // ---------------------------------------------------------------------------
    // XML parsing — health clamp
    // ---------------------------------------------------------------------------

    #[test]
    fn test_parse_health_now_clamped_to_max() {
        let mt = Monsters::parse_monster_type_from_xml(HEALTH_CLAMP_XML).unwrap();
        assert_eq!(mt.max_health, 100);
        assert_eq!(mt.health, 100); // was 200, clamped
    }

    // ---------------------------------------------------------------------------
    // XML parsing — RAT_XML full attributes
    // ---------------------------------------------------------------------------

    #[test]
    fn test_parse_rat_name() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.name, "Rat");
    }

    #[test]
    fn test_parse_rat_experience() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.experience, 5);
    }

    #[test]
    fn test_parse_rat_speed() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.base_speed, 100);
    }

    #[test]
    fn test_parse_rat_health() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.health, 30);
        assert_eq!(mt.max_health, 30);
    }

    #[test]
    fn test_parse_rat_look_type() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.look_type, 21);
    }

    #[test]
    fn test_parse_rat_look_components() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.look_head, 0);
        assert_eq!(mt.look_body, 0);
        assert_eq!(mt.look_legs, 0);
        assert_eq!(mt.look_feet, 0);
    }

    // --- targetchange ---

    #[test]
    fn test_parse_rat_targetchange_speed() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.change_target_speed, 2000);
    }

    #[test]
    fn test_parse_rat_targetchange_chance() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.change_target_chance, 10);
    }

    // --- flags ---

    #[test]
    fn test_parse_rat_flag_summonable_false() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert!(!mt.is_summonable);
    }

    #[test]
    fn test_parse_rat_flag_attackable_true() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert!(mt.is_attackable);
    }

    #[test]
    fn test_parse_rat_flag_hostile_true() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert!(mt.is_hostile);
    }

    #[test]
    fn test_parse_rat_flag_pushable_true() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert!(mt.is_pushable);
    }

    // --- voices ---

    #[test]
    fn test_parse_rat_voices_speed() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.yell_speed_ticks, 5000);
    }

    #[test]
    fn test_parse_rat_voices_chance() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.yell_chance, 10);
    }

    #[test]
    fn test_parse_rat_voices_count() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.voices.len(), 2);
    }

    #[test]
    fn test_parse_rat_voices_first_sentence() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.voices[0].text, "Squeak!");
        assert!(!mt.voices[0].yell);
    }

    #[test]
    fn test_parse_rat_voices_second_yell_true() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.voices[1].text, "SQUEAAAK!");
        assert!(mt.voices[1].yell);
    }

    // --- summons ---

    #[test]
    fn test_parse_rat_max_summons() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.max_summons, 3);
    }

    #[test]
    fn test_parse_rat_summons_count() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.summons.len(), 2);
    }

    #[test]
    fn test_parse_rat_summon_first_name() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.summons[0].name, "Cave Rat");
    }

    #[test]
    fn test_parse_rat_summon_first_speed() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.summons[0].speed, 2000);
    }

    #[test]
    fn test_parse_rat_summon_first_chance() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.summons[0].chance, 50);
    }

    #[test]
    fn test_parse_rat_summon_first_max() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.summons[0].max, 2);
    }

    #[test]
    fn test_parse_rat_summon_first_force_false() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert!(!mt.summons[0].force);
    }

    #[test]
    fn test_parse_rat_summon_second_force_true() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert!(mt.summons[1].force);
    }

    // --- defenses ---

    #[test]
    fn test_parse_rat_defense_value() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.defense, 5);
    }

    #[test]
    fn test_parse_rat_armor_value() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.armor, 3);
    }

    #[test]
    fn test_parse_rat_defense_spell_count() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.defense_spells.len(), 1);
    }

    #[test]
    fn test_parse_rat_defense_spell_name() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.defense_spells[0].name, "healing");
    }

    #[test]
    fn test_parse_rat_defense_spell_chance() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.defense_spells[0].chance, 20);
    }

    #[test]
    fn test_parse_rat_defense_spell_min_max() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        // min=10, max=30 — both positive, so normalised as is
        assert_eq!(mt.defense_spells[0].min_combat_value, 10);
        assert_eq!(mt.defense_spells[0].max_combat_value, 30);
    }

    // --- attacks ---

    #[test]
    fn test_parse_rat_attack_count() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.attack_spells.len(), 2);
    }

    #[test]
    fn test_parse_rat_attack_melee_name_and_flag() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.attack_spells[0].name, "melee");
        assert!(mt.attack_spells[0].is_melee);
    }

    #[test]
    fn test_parse_rat_attack_fire_name() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.attack_spells[1].name, "fire");
        assert!(!mt.attack_spells[1].is_melee);
    }

    #[test]
    fn test_parse_rat_attack_fire_normalised_values() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        // min=-50 max=-80 → |min|=50 < |max|=80, so normalised: min=-50, max=-80
        // After normalisation: |min_combat_value| ≤ |max_combat_value|
        let sp = &mt.attack_spells[1];
        assert!(sp.min_combat_value.unsigned_abs() <= sp.max_combat_value.unsigned_abs());
    }

    #[test]
    fn test_parse_rat_attack_fire_chance() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.attack_spells[1].chance, 50);
    }

    // --- loot ---

    #[test]
    fn test_parse_rat_loot_count() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.loot_entries.len(), 2);
    }

    #[test]
    fn test_parse_rat_loot_first_id() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.loot_entries[0].id, 2148);
    }

    #[test]
    fn test_parse_rat_loot_first_countmax() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.loot_entries[0].count_max, 10);
    }

    #[test]
    fn test_parse_rat_loot_first_chance() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        assert_eq!(mt.loot_entries[0].chance, 50_000);
    }

    #[test]
    fn test_parse_rat_loot_second_optional_fields() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        let e = &mt.loot_entries[1];
        assert_eq!(e.sub_type, Some(50));
        assert_eq!(e.action_id, Some(100));
        assert_eq!(e.text.as_deref(), Some("Hello"));
    }

    #[test]
    fn test_parse_rat_loot_first_no_optional_fields() {
        let mt = Monsters::parse_monster_type_from_xml(RAT_XML).unwrap();
        let e = &mt.loot_entries[0];
        assert_eq!(e.sub_type, None);
        assert_eq!(e.action_id, None);
        assert_eq!(e.text, None);
    }

    // ---------------------------------------------------------------------------
    // Edge cases
    // ---------------------------------------------------------------------------

    #[test]
    fn test_parse_voices_chance_capped_at_100() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <voices speed="1000" chance="200">
    <voice sentence="Hi" yell="0"/>
  </voices>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.yell_chance, 100);
    }

    #[test]
    fn test_parse_summons_max_capped_at_100() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <summons maxSummons="999"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.max_summons, 100);
    }

    #[test]
    fn test_parse_loot_chance_capped_at_100000() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <loot>
    <item id="1" countmax="1" chance="999999"/>
  </loot>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.loot_entries[0].chance, 100_000);
    }

    #[test]
    fn test_parse_loot_invalid_id_skipped() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <loot>
    <item id="0" countmax="1" chance="50000"/>
    <item id="2148" countmax="1" chance="50000"/>
  </loot>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // id=0 is skipped; only id=2148 is valid
        assert_eq!(mt.loot_entries.len(), 1);
        assert_eq!(mt.loot_entries[0].id, 2148);
    }

    #[test]
    fn test_parse_can_push_creatures_disables_pushable() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag pushable="1"/>
    <flag canpushcreatures="1"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // C++ rule: canPushCreatures → pushable = false
        assert!(!mt.is_pushable);
        assert!(mt.can_push_creatures);
    }

    #[test]
    fn test_parse_name_description_explicit() {
        let xml = r#"<monster name="Demon" nameDescription="a fierce demon">
  <health now="100" max="100"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.name_description, "a fierce demon");
    }

    #[test]
    fn test_parse_voice_interval_attribute() {
        // voices can use "interval" as alias for "speed"
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <voices interval="3000" chance="5">
    <voice sentence="Hello" yell="0"/>
  </voices>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.yell_speed_ticks, 3000);
        assert_eq!(mt.voices.len(), 1);
        assert_eq!(mt.voices[0].text, "Hello");
    }

    #[test]
    fn test_parse_targetchange_interval_alias() {
        // targetchange can use "speed" or "interval"
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <targetchange speed="4000" chance="25"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.change_target_speed, 4000);
        assert_eq!(mt.change_target_chance, 25);
    }

    #[test]
    fn test_parse_spell_chance_capped_at_100() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <attacks>
    <attack name="fire" interval="2000" chance="200" min="-10" max="-20"/>
  </attacks>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.attack_spells[0].chance, 100);
    }

    #[test]
    fn test_parse_look_typeex() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <look typeex="4526"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.look_type_ex, 4526);
        assert_eq!(mt.look_type, 0); // type not set
    }

    #[test]
    fn test_parse_loot_default_countmax_is_1() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <loot>
    <item id="5" chance="50000"/>
  </loot>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.loot_entries[0].count_max, 1);
    }

    #[test]
    fn test_parse_monster_xml_typed_error_wrong_root() {
        // Compare against the exact ParseError to avoid the un-hittable
        // "match fails" arm of `assert!(matches!(...))` macro expansion.
        let err = Monsters::parse_monster_xml(WRONG_ROOT_XML).err().unwrap();
        assert_eq!(err, ParseError::WrongRoot("foo".to_string()));
    }

    #[test]
    fn test_parse_monster_xml_typed_error_missing_name() {
        let err = Monsters::parse_monster_xml(MISSING_NAME_XML).err().unwrap();
        assert_eq!(err, ParseError::MissingAttribute("name".to_string()));
    }

    #[test]
    fn test_parse_monster_xml_typed_error_xml_error() {
        // The exact XmlError message depends on roxmltree's reporting; verify
        // the variant via the Display impl, which we already cover separately
        // for each variant. The "XML parse error: …" prefix is unique to
        // `ParseError::XmlError`.
        let err = Monsters::parse_monster_xml(MALFORMED_XML).err().unwrap();
        assert!(err.to_string().starts_with("XML parse error:"));
    }

    // ---------------------------------------------------------------------------
    // Coverage gap tests — added during Phase 6 audit.
    // ---------------------------------------------------------------------------

    #[test]
    fn test_monster_type_new_matches_default() {
        // Cover `MonsterType::new()` — must return defaults.
        let a = MonsterType::new();
        let b = MonsterType::default();
        assert_eq!(a.name, b.name);
        assert_eq!(a.health, b.health);
        assert_eq!(a.max_health, b.max_health);
        assert_eq!(a.base_speed, b.base_speed);
        assert_eq!(a.static_attack_chance, b.static_attack_chance);
        assert_eq!(a.target_distance, b.target_distance);
        assert!(a.voices.is_empty());
        assert!(a.summons.is_empty());
        assert!(a.attack_spells.is_empty());
        assert!(a.defense_spells.is_empty());
        assert!(a.loot_entries.is_empty());
    }

    #[test]
    fn test_monsters_default_matches_new() {
        // Cover `<Monsters as Default>::default()`.
        let m = Monsters::default();
        assert_eq!(m.get_monster_count(), 0);
        assert!(m.get_monster_type("anything").is_none());
    }

    #[test]
    fn test_parse_unknown_top_level_child_ignored() {
        // Cover the `_ => {}` arm in the top-level section dispatch.
        // Random unknown elements must be silently skipped.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <unknown_section foo="bar"/>
  <another>data</another>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.name, "X");
        assert_eq!(mt.health, 1);
    }

    #[test]
    fn test_parse_voices_non_voice_child_skipped() {
        // Cover `continue` for child whose tag_name != "voice".
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <voices speed="1000" chance="10">
    <voice sentence="ok" yell="0"/>
    <noise sentence="ignored" yell="1"/>
    <voice sentence="also ok" yell="1"/>
  </voices>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // Only the two actual <voice> children must be recorded.
        assert_eq!(mt.voices.len(), 2);
        assert_eq!(mt.voices[0].text, "ok");
        assert_eq!(mt.voices[1].text, "also ok");
    }

    #[test]
    fn test_parse_summons_non_summon_child_skipped() {
        // Cover `continue` for child whose tag_name != "summon".
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <summons maxSummons="5">
    <summon name="Cave Rat" speed="2000" chance="50" max="1"/>
    <notsummon name="ghost"/>
  </summons>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.summons.len(), 1);
        assert_eq!(mt.summons[0].name, "Cave Rat");
    }

    #[test]
    fn test_parse_summons_missing_name_skipped() {
        // Cover `None => continue` when <summon> has no `name` attribute.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <summons maxSummons="5">
    <summon speed="2000" chance="50" max="1"/>
    <summon name="Cave Rat" speed="2000" chance="50" max="1"/>
  </summons>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // The first summon (no name) must be skipped.
        assert_eq!(mt.summons.len(), 1);
        assert_eq!(mt.summons[0].name, "Cave Rat");
    }

    #[test]
    fn test_parse_spell_block_swaps_when_abs_min_greater_than_abs_max() {
        // Cover the `mem::swap` line in parse_spell_block.
        // |min|=80 > |max|=50 → values must be swapped so |min| ≤ |max|.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <attacks>
    <attack name="fire" interval="2000" chance="100" min="-80" max="-50"/>
  </attacks>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        let sp = &mt.attack_spells[0];
        // After swap: min_combat_value should be -50, max_combat_value should be -80.
        assert_eq!(sp.min_combat_value, -50);
        assert_eq!(sp.max_combat_value, -80);
        // Invariant: |min| ≤ |max|.
        assert!(sp.min_combat_value.unsigned_abs() <= sp.max_combat_value.unsigned_abs());
    }

    #[test]
    fn test_parse_loot_non_item_child_skipped() {
        // Cover `continue` for non-<item> children in <loot>.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <loot>
    <item id="2148" countmax="1" chance="50000"/>
    <noise id="2152" countmax="1" chance="50000"/>
    <comment>not an item</comment>
  </loot>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // Only the actual <item> entry is recorded.
        assert_eq!(mt.loot_entries.len(), 1);
        assert_eq!(mt.loot_entries[0].id, 2148);
    }

    #[test]
    fn test_parse_flag_staticattack_value_clamped() {
        // Cover `staticattack` flag branch incl. clamp at 100.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag staticattack="200"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.static_attack_chance, 100);
    }

    #[test]
    fn test_parse_flag_staticattack_normal_value() {
        // Cover `staticattack` flag branch with sub-100 value.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag staticattack="42"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.static_attack_chance, 42);
    }

    #[test]
    fn test_parse_flag_targetdistance_clamped_to_min_one() {
        // Cover `targetdistance` branch and `.max(1)` clamp.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag targetdistance="0"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // C++ clamps targetdistance to >= 1
        assert_eq!(mt.target_distance, 1);
    }

    #[test]
    fn test_parse_flag_targetdistance_normal_value() {
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag targetdistance="5"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.target_distance, 5);
    }

    #[test]
    fn test_parse_flag_runonhealth() {
        // Cover `runonhealth` branch.
        let xml = r#"<monster name="X">
  <health now="100" max="100"/>
  <flags>
    <flag runonhealth="20"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.run_away_health, 20);
    }

    #[test]
    fn test_parse_flag_hidehealth() {
        // Cover `hidehealth` branch.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag hidehealth="1"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(mt.hidden_health);
    }

    #[test]
    fn test_parse_flag_canwalkonenergy_false() {
        // Cover `canwalkonenergy` branch (default true, override to false).
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag canwalkonenergy="0"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(!mt.can_walk_on_energy);
    }

    #[test]
    fn test_parse_flag_canwalkonfire_false() {
        // Cover `canwalkonfire` branch.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag canwalkonfire="0"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(!mt.can_walk_on_fire);
    }

    #[test]
    fn test_parse_flag_canwalkonpoison_false() {
        // Cover `canwalkonpoison` branch.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag canwalkonpoison="0"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(!mt.can_walk_on_poison);
    }

    #[test]
    fn test_parse_flag_unknown_key_silently_ignored() {
        // Cover the `_ => {}` arm inside parse_flags.
        // Unknown flag names must not error and must not corrupt defaults.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag iliveinthecloud="1"/>
    <flag hostile="0"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // Known flag still parsed; unknown one ignored.
        assert!(!mt.is_hostile);
    }

    #[test]
    fn test_parse_flag_staticattack_invalid_value_ignored() {
        // Covers the `if let Ok(v) = val.parse::<u32>()` path failing.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag staticattack="notanumber"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // Invalid value silently dropped; default preserved.
        assert_eq!(mt.static_attack_chance, 95);
    }

    #[test]
    fn test_parse_error_display_xml_error() {
        // Cover Display impl for ParseError::XmlError.
        let e = ParseError::XmlError("boom".to_string());
        let s = format!("{e}");
        assert!(s.contains("XML parse error"));
        assert!(s.contains("boom"));
    }

    #[test]
    fn test_parse_error_display_missing_attribute() {
        // Cover Display impl for ParseError::MissingAttribute.
        let e = ParseError::MissingAttribute("name".to_string());
        let s = format!("{e}");
        assert!(s.contains("missing attribute"));
        assert!(s.contains("name"));
    }

    #[test]
    fn test_parse_error_display_wrong_root() {
        // Cover Display impl for ParseError::WrongRoot.
        let e = ParseError::WrongRoot("foo".to_string());
        let s = format!("{e}");
        assert!(s.contains("wrong root element"));
        assert!(s.contains("foo"));
    }

    #[test]
    fn test_parse_error_equality_for_same_variant() {
        // Cover derived PartialEq/Eq usage on ParseError.
        assert_eq!(
            ParseError::WrongRoot("foo".to_string()),
            ParseError::WrongRoot("foo".to_string())
        );
        assert_ne!(
            ParseError::WrongRoot("foo".to_string()),
            ParseError::MissingAttribute("foo".to_string())
        );
    }

    #[test]
    fn test_voice_block_equality_and_clone() {
        // Cover derived Clone/PartialEq on VoiceBlock.
        let a = VoiceBlock {
            text: "hi".to_string(),
            yell: false,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_summon_block_equality_and_clone() {
        // Cover derived Clone/PartialEq on SummonBlock.
        let a = SummonBlock {
            name: "rat".to_string(),
            speed: 1000,
            chance: 50,
            max: 2,
            force: false,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_spell_block_equality_and_clone() {
        // Cover derived Clone/PartialEq on SpellBlock.
        let a = SpellBlock {
            name: "fire".to_string(),
            chance: 50,
            speed: 2000,
            min_combat_value: -10,
            max_combat_value: -20,
            is_melee: false,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_loot_entry_equality_and_clone() {
        // Cover derived Clone/PartialEq on LootEntry.
        let a = LootEntry {
            id: 2148,
            count_max: 10,
            chance: 50_000,
            sub_type: Some(5),
            action_id: None,
            text: Some("hello".to_string()),
            child_loot: vec![],
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ---------------------------------------------------------------------------
    // Branch-completion tests — exercise the rare alternate arms that drove
    // the remaining line/region misses to 0 in baseline coverage.
    // ---------------------------------------------------------------------------

    #[test]
    fn test_parse_health_missing_attributes_uses_defaults() {
        // Cover the "None" branch of the two `if let Some(v) = node.attribute(...)` in parse_health.
        let xml = r#"<monster name="X">
  <health/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // No now/max — defaults of 100 preserved.
        assert_eq!(mt.health, 100);
        assert_eq!(mt.max_health, 100);
    }

    #[test]
    fn test_parse_look_typeex_then_other_attrs_skipped() {
        // After choosing the `typeex` branch, head/body/legs/feet should still
        // be parsed when present — exercising those `if let Some(v)` paths.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <look typeex="123" head="4" body="5" legs="6" feet="7"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.look_type_ex, 123);
        // type was not set so look_type stays 0
        assert_eq!(mt.look_type, 0);
        assert_eq!(mt.look_head, 4);
        assert_eq!(mt.look_body, 5);
        assert_eq!(mt.look_legs, 6);
        assert_eq!(mt.look_feet, 7);
    }

    #[test]
    fn test_parse_look_missing_optional_attrs_keeps_defaults() {
        // Cover the `else` (None) arms for head/body/legs/feet when not present.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <look type="9"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.look_type, 9);
        assert_eq!(mt.look_head, 0);
        assert_eq!(mt.look_body, 0);
        assert_eq!(mt.look_legs, 0);
        assert_eq!(mt.look_feet, 0);
    }

    #[test]
    fn test_parse_summon_interval_attribute_as_speed_alias() {
        // Cover the `.or_else(|| .attribute("interval"))` fallback in parse_summons.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <summons maxSummons="5">
    <summon name="Cave Rat" interval="2500" chance="50" max="1"/>
  </summons>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.summons.len(), 1);
        assert_eq!(mt.summons[0].speed, 2500);
    }

    #[test]
    fn test_parse_spell_block_missing_name_returns_none() {
        // Cover the `?` early-return in `parse_spell_block` when no name.
        // This means a defense/attack child with no name is silently dropped.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <attacks>
    <attack interval="2000" chance="100" min="-10" max="-20"/>
    <attack name="fire" interval="2000" chance="100" min="-10" max="-20"/>
  </attacks>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // First <attack> has no name → dropped.
        assert_eq!(mt.attack_spells.len(), 1);
        assert_eq!(mt.attack_spells[0].name, "fire");
    }

    #[test]
    fn test_parse_defense_spell_missing_name_dropped() {
        // Cover same `?` path through parse_defenses → parse_spell_block.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <defenses defense="5" armor="3">
    <defense interval="2000" chance="20" min="10" max="30"/>
    <defense name="healing" interval="2000" chance="20" min="10" max="30"/>
  </defenses>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.defense_spells.len(), 1);
        assert_eq!(mt.defense_spells[0].name, "healing");
    }

    #[test]
    fn test_parse_loot_chance1_attribute_fallback() {
        // Cover `.or_else(|| item_node.attribute("chance1"))` in parse_loot.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <loot>
    <item id="2148" countmax="1" chance1="12345"/>
  </loot>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.loot_entries.len(), 1);
        assert_eq!(mt.loot_entries[0].chance, 12_345);
    }

    #[test]
    fn test_parse_targetchange_missing_attributes_keeps_defaults() {
        // Cover the `else` (None) arms in parse_targetchange.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <targetchange/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.change_target_speed, 0);
        assert_eq!(mt.change_target_chance, 0);
    }

    #[test]
    fn test_parse_defenses_missing_defense_and_armor() {
        // Cover the `else` (None) arms in parse_defenses for missing defense/armor.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <defenses>
    <defense name="healing" interval="2000" chance="20" min="10" max="30"/>
  </defenses>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.defense, 0);
        assert_eq!(mt.armor, 0);
        assert_eq!(mt.defense_spells.len(), 1);
    }

    #[test]
    fn test_parse_targetchange_chance_clamped_to_100() {
        // Cover the `.min(100)` cap on targetchange chance.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <targetchange speed="1000" chance="500"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.change_target_chance, 100);
    }

    #[test]
    fn test_parse_flag_ignorespawnblock() {
        // Cover `ignorespawnblock` flag arm.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag ignorespawnblock="1"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(mt.is_ignoring_spawn_block);
    }

    #[test]
    fn test_parse_flag_illusionable() {
        // Cover `illusionable` flag arm.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag illusionable="1"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(mt.is_illusionable);
    }

    #[test]
    fn test_parse_flag_challengeable_false() {
        // Cover `challengeable` flag arm (default is true, override to false).
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag challengeable="0"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(!mt.is_challengeable);
    }

    #[test]
    fn test_parse_flag_convinceable() {
        // Cover `convinceable` flag arm.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag convinceable="1"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert!(mt.is_convinceable);
    }

    #[test]
    fn test_parse_flag_targetdistance_invalid_value_ignored() {
        // Cover the `Err` branch in `if let Ok(v) = val.parse::<i32>()` for targetdistance.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag targetdistance="notanumber"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // Invalid value silently dropped; default 1 preserved.
        assert_eq!(mt.target_distance, 1);
    }

    #[test]
    fn test_parse_flag_runonhealth_invalid_value_ignored() {
        // Cover the `Err` branch in `if let Ok(v) = val.parse::<i32>()` for runonhealth.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <flags>
    <flag runonhealth="notanumber"/>
  </flags>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        // Invalid value silently dropped; default 0 preserved.
        assert_eq!(mt.run_away_health, 0);
    }

    #[test]
    fn test_parse_look_neither_type_nor_typeex_keeps_defaults() {
        // Cover the implicit else of the `if .. else if` chain when neither
        // `type` nor `typeex` parses successfully — both look_type and
        // look_type_ex must remain 0.
        let xml = r#"<monster name="X">
  <health now="1" max="1"/>
  <look corpse="100"/>
</monster>"#;
        let mt = Monsters::parse_monster_type_from_xml(xml).unwrap();
        assert_eq!(mt.look_type, 0);
        assert_eq!(mt.look_type_ex, 0);
    }
}
