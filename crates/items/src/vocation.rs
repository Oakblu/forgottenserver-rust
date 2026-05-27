//! Migrated from forgottenserver/src/vocation.h and vocation.cpp
//!
//! Provides vocation definitions, skill/mana formulas, and XML loading.
//! Corresponds to the C++ `Vocation` class and `Vocations` class.

use std::collections::BTreeMap;

use forgottenserver_common::enums::VOCATION_NONE;

// ---------------------------------------------------------------------------
// Skill constants matching C++ arrays
// ---------------------------------------------------------------------------

/// C++ `SKILL_LAST` = 6 (Fishing). Skill ids 0..=6.
pub const SKILL_LAST: u8 = 6;

/// C++ `skillBase[SKILL_LAST + 1]` = {50, 50, 50, 50, 30, 100, 20}
pub const SKILL_BASE: [u32; 7] = [50, 50, 50, 50, 30, 100, 20];

/// Minimum skill level used in the skill-tries formula.
/// C++ `MINIMUM_SKILL_LEVEL` = 10.
pub const MINIMUM_SKILL_LEVEL: u16 = 10;

// ---------------------------------------------------------------------------
// Vocation
// ---------------------------------------------------------------------------

/// Corresponds to the C++ `Vocation` class.
///
/// All fields mirror the C++ private members with Rust naming conventions.
#[derive(Debug, Clone)]
pub struct Vocation {
    pub id: u16,
    pub client_id: u8,
    pub name: String,
    pub description: String,

    /// `skillMultipliers[SKILL_LAST + 1]` — default from C++: {1.5, 2.0, 2.0, 2.0, 2.0, 1.5, 1.1}
    pub skill_multipliers: [f64; 7],
    /// `manaMultiplier` — default 4.0
    pub mana_multiplier: f64,

    pub gain_health_ticks: u32,
    pub gain_health_amount: u32,
    pub gain_mana_ticks: u32,
    pub gain_mana_amount: u32,
    pub gain_cap: u32,
    pub gain_mana: u32,
    pub gain_hp: u32,
    pub from_vocation: u32,
    pub attack_speed: u32,
    pub base_speed: u32,
    pub no_pong_kick_time: u32,

    pub gain_soul_ticks: u16,
    pub soul_max: u8,

    pub allow_pvp: bool,
    pub magic_shield: bool,

    pub melee_damage_multiplier: f32,
    pub dist_damage_multiplier: f32,
    pub defense_multiplier: f32,
    pub armor_multiplier: f32,
}

impl Vocation {
    /// Create a new vocation with C++ defaults.
    pub fn new(id: u16) -> Self {
        Self {
            id,
            client_id: 0,
            name: "none".to_string(),
            description: String::new(),
            skill_multipliers: [1.5, 2.0, 2.0, 2.0, 2.0, 1.5, 1.1],
            mana_multiplier: 4.0,
            gain_health_ticks: 6,
            gain_health_amount: 1,
            gain_mana_ticks: 6,
            gain_mana_amount: 1,
            gain_cap: 500,
            gain_mana: 5,
            gain_hp: 5,
            from_vocation: VOCATION_NONE as u32,
            attack_speed: 1500,
            base_speed: 220,
            no_pong_kick_time: 60000,
            gain_soul_ticks: 120,
            soul_max: 100,
            allow_pvp: true,
            magic_shield: false,
            melee_damage_multiplier: 1.0,
            dist_damage_multiplier: 1.0,
            defense_multiplier: 1.0,
            armor_multiplier: 1.0,
        }
    }

    /// Required skill tries for a skill at a given level.
    ///
    /// Mirrors C++:
    /// ```cpp
    /// skillBase[skill] * pow(skillMultipliers[skill], level - (MINIMUM_SKILL_LEVEL + 1))
    /// ```
    ///
    /// Returns `0` if `skill > SKILL_LAST` or `level <= MINIMUM_SKILL_LEVEL`.
    pub fn required_skill_tries(&self, skill: u8, level: u16) -> u64 {
        if skill > SKILL_LAST {
            return 0;
        }
        // C++ casts the exponent to `int32_t`: level - (MINIMUM_SKILL_LEVEL + 1)
        // If level <= MINIMUM_SKILL_LEVEL the exponent is negative, pow returns a
        // fraction which gets truncated to 0 when cast to u64.
        let exponent = (level as i32) - (MINIMUM_SKILL_LEVEL as i32 + 1);
        if exponent < 0 {
            return 0;
        }
        let base = SKILL_BASE[skill as usize] as f64;
        let multiplier = self.skill_multipliers[skill as usize];
        (base * multiplier.powi(exponent)) as u64
    }

    /// Required mana for a given magic level.
    ///
    /// Mirrors C++:
    /// ```cpp
    /// 1600 * pow(manaMultiplier, magLevel - 1)
    /// ```
    ///
    /// Returns `0` if `mag_level == 0`.
    pub fn required_mana(&self, mag_level: u32) -> u64 {
        if mag_level == 0 {
            return 0;
        }
        let exponent = (mag_level as i32) - 1;
        (1600.0_f64 * self.mana_multiplier.powi(exponent)) as u64
    }

    /// Config-gated mana-shield accessor. Mirrors C++
    /// `Vocation::getMagicShield() const`: returns `false` whenever
    /// `MANASHIELD_BREAKABLE` is disabled in `config.lua`, regardless of
    /// the per-vocation flag. When the config is on, the per-vocation
    /// `magic_shield` field decides.
    ///
    /// Callers pass a `&ConfigManager` directly because the items crate
    /// owns no global config singleton — keeps the read sites explicit
    /// and the dependency inverted.
    pub fn get_magic_shield(
        &self,
        config: &forgottenserver_common::configmanager::ConfigManager,
    ) -> bool {
        if !config
            .get_boolean(forgottenserver_common::configmanager::BooleanKey::ManashieldBreakable)
        {
            return false;
        }
        self.magic_shield
    }
}

// ---------------------------------------------------------------------------
// Vocations
// ---------------------------------------------------------------------------

/// Corresponds to the C++ `Vocations` class.
pub struct Vocations {
    map: BTreeMap<u16, Vocation>,
}

impl Vocations {
    /// Parse vocations from an XML string.
    ///
    /// XML format (mirrors C++ parsing):
    /// ```xml
    /// <vocations>
    ///   <vocation id="1" name="Sorcerer" clientid="1" gaincap="10" ...>
    ///     <skill id="0" multiplier="1.5"/>
    ///     <formula meleeDamage="1.0" distDamage="1.0" defense="1.0" armor="1.0"/>
    ///   </vocation>
    /// </vocations>
    /// ```
    pub fn load_from_xml(xml: &str) -> Result<Vocations, String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;

        let root = doc
            .descendants()
            .find(|n| n.has_tag_name("vocations"))
            .ok_or_else(|| "Missing <vocations> root element".to_string())?;

        let mut map: BTreeMap<u16, Vocation> = BTreeMap::new();

        for voc_node in root.children().filter(|n| n.is_element()) {
            let id_str = voc_node
                .attribute("id")
                .ok_or_else(|| "Missing vocation id".to_string())?;
            let id: u16 = id_str
                .parse()
                .map_err(|_| format!("Invalid vocation id: {id_str}"))?;

            let mut voc = Vocation::new(id);

            // Parse all vocation attributes
            for attr in voc_node.attributes() {
                let name = attr.name().to_ascii_lowercase();
                let val = attr.value();
                match name.as_str() {
                    "id" => {} // already parsed
                    "name" => voc.name = val.to_string(),
                    "allowpvp" => voc.allow_pvp = parse_bool(val),
                    "clientid" => {
                        voc.client_id = val.parse().unwrap_or(0);
                    }
                    "description" => voc.description = val.to_string(),
                    "magicshield" => voc.magic_shield = parse_bool(val),
                    "gaincap" => {
                        // C++: gainCap = value * 100
                        let raw: u32 = val.parse().unwrap_or(0);
                        voc.gain_cap = raw * 100;
                    }
                    "gainhp" => voc.gain_hp = val.parse().unwrap_or(0),
                    "gainmana" => voc.gain_mana = val.parse().unwrap_or(0),
                    "gainhpticks" => voc.gain_health_ticks = val.parse().unwrap_or(0),
                    "gainhpamount" => voc.gain_health_amount = val.parse().unwrap_or(0),
                    "gainmanaticks" => voc.gain_mana_ticks = val.parse().unwrap_or(0),
                    "gainmanaamount" => voc.gain_mana_amount = val.parse().unwrap_or(0),
                    "manamultiplier" => {
                        voc.mana_multiplier = val.parse().unwrap_or(4.0);
                    }
                    "attackspeed" => voc.attack_speed = val.parse().unwrap_or(0),
                    "basespeed" => voc.base_speed = val.parse().unwrap_or(0),
                    "soulmax" => voc.soul_max = val.parse().unwrap_or(0),
                    "gainsoulticks" => voc.gain_soul_ticks = val.parse().unwrap_or(0),
                    "fromvoc" => voc.from_vocation = val.parse().unwrap_or(0),
                    "nopongkicktime" => {
                        // C++: noPongKickTime = value * 1000
                        let raw: u32 = val.parse().unwrap_or(0);
                        voc.no_pong_kick_time = raw * 1000;
                    }
                    _ => {} // unknown attribute — silently skip (C++ prints Notice)
                }
            }

            // Parse child elements: <skill> and <formula>
            for child in voc_node.children().filter(|n| n.is_element()) {
                match child.tag_name().name().to_ascii_lowercase().as_str() {
                    "skill" => {
                        if let Some(sid_str) = child.attribute("id") {
                            if let Ok(sid) = sid_str.parse::<u8>() {
                                if sid <= SKILL_LAST {
                                    if let Some(mult_str) = child.attribute("multiplier") {
                                        if let Ok(mult) = mult_str.parse::<f64>() {
                                            voc.skill_multipliers[sid as usize] = mult;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "formula" => {
                        if let Some(v) = child.attribute("meleeDamage") {
                            voc.melee_damage_multiplier = v.parse().unwrap_or(1.0);
                        }
                        if let Some(v) = child.attribute("distDamage") {
                            voc.dist_damage_multiplier = v.parse().unwrap_or(1.0);
                        }
                        if let Some(v) = child.attribute("defense") {
                            voc.defense_multiplier = v.parse().unwrap_or(1.0);
                        }
                        if let Some(v) = child.attribute("armor") {
                            voc.armor_multiplier = v.parse().unwrap_or(1.0);
                        }
                    }
                    _ => {}
                }
            }

            map.insert(id, voc);
        }

        Ok(Vocations { map })
    }

    /// Return a reference to the vocation with the given id, or `None`.
    ///
    /// Mirrors C++ `Vocations::getVocation(uint16_t)`.
    pub fn get_vocation(&self, id: u16) -> Option<&Vocation> {
        self.map.get(&id)
    }

    /// Return the id of the vocation with the given name (case-insensitive),
    /// or `None` if not found.
    ///
    /// Mirrors C++ `Vocations::getVocationId(string_view)`.
    pub fn get_vocation_id(&self, name: &str) -> Option<u16> {
        self.map
            .iter()
            .find(|(_, v)| v.name.eq_ignore_ascii_case(name))
            .map(|(id, _)| *id)
    }

    /// Return the promoted vocation id for the given base vocation, or
    /// `VOCATION_NONE` if no promotion exists.
    ///
    /// Mirrors C++ `Vocations::getPromotedVocation(uint16_t)`.
    pub fn get_promoted_vocation(&self, id: u16) -> u16 {
        self.map
            .iter()
            .find(|(promoted_id, v)| v.from_vocation == id as u32 && **promoted_id != id)
            .map(|(promoted_id, _)| *promoted_id)
            .unwrap_or(VOCATION_NONE)
    }

    /// Iterate over all vocations.
    pub fn vocations(&self) -> impl Iterator<Item = &Vocation> {
        self.map.values()
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn parse_bool(s: &str) -> bool {
    matches!(s, "1" | "true" | "True" | "TRUE")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Skill tries formula ----------------------------------------------

    /// C++ formula: skillBase[skill] * pow(multiplier, level - (MINIMUM_SKILL_LEVEL + 1))
    ///
    /// For Fist (skill=0): base=50, default multiplier=1.5
    /// At level=11: exponent = 11 - 11 = 0  →  50 * 1.5^0 = 50
    #[test]
    fn test_required_skill_tries_fist_level_11() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(0, 11), 50);
    }

    /// Fist level 12: exponent = 12 - 11 = 1  → 50 * 1.5^1 = 75
    #[test]
    fn test_required_skill_tries_fist_level_12() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(0, 12), 75);
    }

    /// Fist level 13: exponent = 2 → 50 * 1.5^2 = 112
    #[test]
    fn test_required_skill_tries_fist_level_13() {
        let voc = Vocation::new(1);
        // 50 * 2.25 = 112.5, truncated to 112
        assert_eq!(voc.required_skill_tries(0, 13), 112);
    }

    /// Sword (skill=2): base=50, default multiplier=2.0
    /// Level 11: 50 * 2.0^0 = 50
    #[test]
    fn test_required_skill_tries_sword_level_11() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(2, 11), 50);
    }

    /// Sword level 12: 50 * 2.0^1 = 100
    #[test]
    fn test_required_skill_tries_sword_level_12() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(2, 12), 100);
    }

    /// Level <= MINIMUM_SKILL_LEVEL returns 0
    #[test]
    fn test_required_skill_tries_level_too_low_returns_zero() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(0, 10), 0);
        assert_eq!(voc.required_skill_tries(0, 1), 0);
    }

    /// Skill > SKILL_LAST returns 0
    #[test]
    fn test_required_skill_tries_invalid_skill_returns_zero() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(7, 15), 0);
        assert_eq!(voc.required_skill_tries(255, 15), 0);
    }

    // ----- Required mana formula --------------------------------------------

    /// C++: 1600 * pow(manaMultiplier, magLevel - 1)
    /// Default multiplier = 4.0
    /// mag_level=0 → 0
    #[test]
    fn test_required_mana_zero_level() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_mana(0), 0);
    }

    /// mag_level=1 → 1600 * 4.0^0 = 1600
    #[test]
    fn test_required_mana_level_1() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_mana(1), 1600);
    }

    /// mag_level=2 → 1600 * 4.0^1 = 6400
    #[test]
    fn test_required_mana_level_2() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_mana(2), 6400);
    }

    /// mag_level=3 → 1600 * 4.0^2 = 25600
    #[test]
    fn test_required_mana_level_3() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_mana(3), 25600);
    }

    /// Custom mana multiplier = 1.0 → always returns 1600 regardless of level
    #[test]
    fn test_required_mana_multiplier_one() {
        let mut voc = Vocation::new(1);
        voc.mana_multiplier = 1.0;
        assert_eq!(voc.required_mana(1), 1600);
        assert_eq!(voc.required_mana(5), 1600);
    }

    // ----- load_from_xml: happy path ----------------------------------------

    fn minimal_xml() -> &'static str {
        r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Sorcerer" clientid="1" gaincap="10" gainhp="5" gainmana="30"
            gainhpticks="6" gainhpamount="1" gainmanaticks="6" gainmanaamount="2"
            manamultiplier="3.0" attackspeed="2000" basespeed="220"
            soulmax="200" gainsoulticks="30" fromvoc="0" nopongkicktime="60"
            allowpvp="1" magicshield="0">
    <skill id="0" multiplier="1.2"/>
    <formula meleeDamage="0.9" distDamage="0.9" defense="1.1" armor="1.1"/>
  </vocation>
  <vocation id="4" name="Elder Sorcerer" clientid="1" fromvoc="1"/>
</vocations>"#
    }

    #[test]
    fn test_load_from_xml_parses_vocation_name() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        assert_eq!(voc.name, "Sorcerer");
    }

    #[test]
    fn test_load_from_xml_parses_client_id() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().client_id, 1);
    }

    #[test]
    fn test_load_from_xml_gain_cap_multiplied_by_100() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        // gaincap="10" → stored as 10 * 100 = 1000
        assert_eq!(vocations.get_vocation(1).unwrap().gain_cap, 1000);
    }

    #[test]
    fn test_load_from_xml_mana_multiplier() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert!((vocations.get_vocation(1).unwrap().mana_multiplier - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_load_from_xml_nopongkicktime_multiplied_by_1000() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        // nopongkicktime="60" → 60 * 1000 = 60000
        assert_eq!(vocations.get_vocation(1).unwrap().no_pong_kick_time, 60000);
    }

    #[test]
    fn test_load_from_xml_skill_multiplier_override() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        // skill id=0 (Fist) was overridden to 1.2
        assert!((voc.skill_multipliers[0] - 1.2).abs() < 1e-9);
        // other skills remain default
        assert!((voc.skill_multipliers[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_load_from_xml_formula_multipliers() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        assert!((voc.melee_damage_multiplier - 0.9).abs() < 1e-6);
        assert!((voc.dist_damage_multiplier - 0.9).abs() < 1e-6);
        assert!((voc.defense_multiplier - 1.1).abs() < 1e-6);
        assert!((voc.armor_multiplier - 1.1).abs() < 1e-6);
    }

    #[test]
    fn test_load_from_xml_from_vocation() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(4).unwrap().from_vocation, 1);
    }

    // ----- get_vocation: not found ------------------------------------------

    #[test]
    fn test_get_vocation_unknown_returns_none() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert!(vocations.get_vocation(999).is_none());
    }

    // ----- get_vocation_id --------------------------------------------------

    #[test]
    fn test_get_vocation_id_found() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation_id("Sorcerer"), Some(1));
    }

    #[test]
    fn test_get_vocation_id_case_insensitive() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation_id("sorcerer"), Some(1));
        assert_eq!(vocations.get_vocation_id("SORCERER"), Some(1));
    }

    #[test]
    fn test_get_vocation_id_not_found() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert!(vocations.get_vocation_id("Druid").is_none());
    }

    // ----- get_promoted_vocation --------------------------------------------

    #[test]
    fn test_get_promoted_vocation_found() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        // id=4 has fromVoc=1, so voc 1 is promoted to voc 4
        assert_eq!(vocations.get_promoted_vocation(1), 4);
    }

    #[test]
    fn test_get_promoted_vocation_not_found_returns_vocation_none() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_promoted_vocation(99), VOCATION_NONE);
    }

    // ----- XML error cases --------------------------------------------------

    #[test]
    fn test_load_from_xml_invalid_xml_returns_err() {
        let result = Vocations::load_from_xml("<bad<<<");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_xml_missing_id_returns_err() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation name="NoId"/>
</vocations>"#;
        let result = Vocations::load_from_xml(xml);
        assert!(result.is_err());
    }

    // ----- vocations() iterator ---------------------------------------------

    #[test]
    fn test_vocations_iterator_count() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.vocations().count(), 2);
    }

    // ----- Vocation::new defaults -------------------------------------------

    #[test]
    fn test_vocation_new_defaults() {
        let voc = Vocation::new(42);
        assert_eq!(voc.id, 42);
        assert_eq!(voc.name, "none");
        assert_eq!(voc.gain_cap, 500);
        assert_eq!(voc.attack_speed, 1500);
        assert_eq!(voc.base_speed, 220);
        assert_eq!(voc.soul_max, 100);
        assert!(voc.allow_pvp);
        assert!(!voc.magic_shield);
        assert!((voc.mana_multiplier - 4.0).abs() < 1e-9);
    }

    // ----- allow_pvp parsing ------------------------------------------------

    #[test]
    fn test_allow_pvp_false_when_zero() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test" allowpvp="0"/>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        assert!(!vocations.get_vocation(1).unwrap().allow_pvp);
    }

    // ----- All 7 skill types: default multipliers and base values ----------

    /// Club (skill=1): base=50, default multiplier=2.0
    /// Level 11: exponent=0 → 50 * 2.0^0 = 50
    #[test]
    fn test_required_skill_tries_club_level_11() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(1, 11), 50);
    }

    /// Club level 12: 50 * 2.0^1 = 100
    #[test]
    fn test_required_skill_tries_club_level_12() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(1, 12), 100);
    }

    /// Axe (skill=3): base=50, default multiplier=2.0
    /// Level 11: 50 * 2.0^0 = 50
    #[test]
    fn test_required_skill_tries_axe_level_11() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(3, 11), 50);
    }

    /// Axe level 13: exponent=2 → 50 * 2.0^2 = 200
    #[test]
    fn test_required_skill_tries_axe_level_13() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(3, 13), 200);
    }

    /// Distance (skill=4): base=30, default multiplier=2.0
    /// Level 11: 30 * 2.0^0 = 30
    #[test]
    fn test_required_skill_tries_distance_level_11() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(4, 11), 30);
    }

    /// Distance level 12: 30 * 2.0^1 = 60
    #[test]
    fn test_required_skill_tries_distance_level_12() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(4, 12), 60);
    }

    /// Shield (skill=5): base=100, default multiplier=1.5
    /// Level 11: 100 * 1.5^0 = 100
    #[test]
    fn test_required_skill_tries_shield_level_11() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(5, 11), 100);
    }

    /// Shield level 12: 100 * 1.5^1 = 150
    #[test]
    fn test_required_skill_tries_shield_level_12() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(5, 12), 150);
    }

    /// Fishing (skill=6): base=20, default multiplier=1.1
    /// Level 11: 20 * 1.1^0 = 20
    #[test]
    fn test_required_skill_tries_fishing_level_11() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(6, 11), 20);
    }

    /// Fishing level 12: 20 * 1.1^1 = 22
    #[test]
    fn test_required_skill_tries_fishing_level_12() {
        let voc = Vocation::new(1);
        // 20 * 1.1 = 22.0, truncated to 22
        assert_eq!(voc.required_skill_tries(6, 12), 22);
    }

    /// Fishing level 13: 20 * 1.1^2 = 24 (20 * 1.21 = 24.2 → 24)
    #[test]
    fn test_required_skill_tries_fishing_level_13() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(6, 13), 24);
    }

    /// Skill id = SKILL_LAST (6) is valid — not out of bounds
    #[test]
    fn test_required_skill_tries_skill_last_is_valid() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_skill_tries(SKILL_LAST, 11), 20);
    }

    // ----- XML: gain_hp, gain_mana, health/mana ticks and amounts ----------

    #[test]
    fn test_load_from_xml_gain_hp() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().gain_hp, 5);
    }

    #[test]
    fn test_load_from_xml_gain_mana() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().gain_mana, 30);
    }

    #[test]
    fn test_load_from_xml_gain_health_ticks() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().gain_health_ticks, 6);
    }

    #[test]
    fn test_load_from_xml_gain_health_amount() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().gain_health_amount, 1);
    }

    #[test]
    fn test_load_from_xml_gain_mana_ticks() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().gain_mana_ticks, 6);
    }

    #[test]
    fn test_load_from_xml_gain_mana_amount() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().gain_mana_amount, 2);
    }

    // ----- XML: attack_speed, base_speed, soul_max, gain_soul_ticks --------

    #[test]
    fn test_load_from_xml_attack_speed() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().attack_speed, 2000);
    }

    #[test]
    fn test_load_from_xml_base_speed() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().base_speed, 220);
    }

    #[test]
    fn test_load_from_xml_soul_max() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().soul_max, 200);
    }

    #[test]
    fn test_load_from_xml_gain_soul_ticks() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        assert_eq!(vocations.get_vocation(1).unwrap().gain_soul_ticks, 30);
    }

    // ----- XML: magic_shield true -------------------------------------------

    #[test]
    fn test_load_from_xml_magic_shield_true() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test" magicshield="1"/>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        assert!(vocations.get_vocation(1).unwrap().magic_shield);
    }

    // ----- XML: allow_pvp with "true"/"false" strings -----------------------

    #[test]
    fn test_allow_pvp_true_with_string_true() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test" allowpvp="true"/>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        assert!(vocations.get_vocation(1).unwrap().allow_pvp);
    }

    #[test]
    fn test_allow_pvp_false_with_string_false() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test" allowpvp="false"/>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        assert!(!vocations.get_vocation(1).unwrap().allow_pvp);
    }

    // ----- XML: description field -------------------------------------------

    #[test]
    fn test_load_from_xml_description() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test" description="A test vocation"/>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        assert_eq!(
            vocations.get_vocation(1).unwrap().description,
            "A test vocation"
        );
    }

    // ----- XML: skill id > SKILL_LAST is silently ignored -------------------

    #[test]
    fn test_load_from_xml_skill_id_out_of_range_is_ignored() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test">
    <skill id="7" multiplier="9.9"/>
  </vocation>
</vocations>"#;
        // Should not error — invalid skill id is silently skipped
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        // Default multipliers remain unchanged for all valid skills
        for (i, &default) in [1.5_f64, 2.0, 2.0, 2.0, 2.0, 1.5, 1.1].iter().enumerate() {
            assert!((voc.skill_multipliers[i] - default).abs() < 1e-9);
        }
    }

    // ----- XML: skill multiplier override for all 7 skill types -------------

    #[test]
    fn test_load_from_xml_skill_multiplier_all_7_skills() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test">
    <skill id="0" multiplier="1.1"/>
    <skill id="1" multiplier="1.2"/>
    <skill id="2" multiplier="1.3"/>
    <skill id="3" multiplier="1.4"/>
    <skill id="4" multiplier="1.5"/>
    <skill id="5" multiplier="1.6"/>
    <skill id="6" multiplier="1.7"/>
  </vocation>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        let expected = [1.1_f64, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7];
        for (i, &exp) in expected.iter().enumerate() {
            assert!((voc.skill_multipliers[i] - exp).abs() < 1e-9);
        }
    }

    // ----- required_mana: default multiplier at higher levels ---------------

    /// mag_level=4 → 1600 * 4.0^3 = 102400
    #[test]
    fn test_required_mana_level_4_default_multiplier() {
        let voc = Vocation::new(1);
        assert_eq!(voc.required_mana(4), 102_400);
    }

    // ----- Vocation::new: default skill multipliers -------------------------

    /// Verify all 7 default skill multipliers from C++: {1.5, 2.0, 2.0, 2.0, 2.0, 1.5, 1.1}
    #[test]
    fn test_vocation_new_default_skill_multipliers() {
        let voc = Vocation::new(1);
        let expected = [1.5_f64, 2.0, 2.0, 2.0, 2.0, 1.5, 1.1];
        for (i, &exp) in expected.iter().enumerate() {
            assert!((voc.skill_multipliers[i] - exp).abs() < 1e-9);
        }
    }

    // ----- Vocation::new: remaining default fields --------------------------

    #[test]
    fn test_vocation_new_default_gain_fields() {
        let voc = Vocation::new(1);
        assert_eq!(voc.gain_hp, 5);
        assert_eq!(voc.gain_mana, 5);
        assert_eq!(voc.gain_health_ticks, 6);
        assert_eq!(voc.gain_health_amount, 1);
        assert_eq!(voc.gain_mana_ticks, 6);
        assert_eq!(voc.gain_mana_amount, 1);
        assert_eq!(voc.gain_soul_ticks, 120);
        assert_eq!(voc.soul_max, 100);
    }

    #[test]
    fn test_vocation_new_default_speed_and_timing() {
        let voc = Vocation::new(1);
        assert_eq!(voc.attack_speed, 1500);
        assert_eq!(voc.base_speed, 220);
        assert_eq!(voc.no_pong_kick_time, 60000);
    }

    #[test]
    fn test_vocation_new_default_multiplier_fields() {
        let voc = Vocation::new(1);
        assert!((voc.melee_damage_multiplier - 1.0).abs() < 1e-6);
        assert!((voc.dist_damage_multiplier - 1.0).abs() < 1e-6);
        assert!((voc.defense_multiplier - 1.0).abs() < 1e-6);
        assert!((voc.armor_multiplier - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vocation_new_default_client_id_and_from_vocation() {
        let voc = Vocation::new(5);
        assert_eq!(voc.client_id, 0);
        assert_eq!(voc.from_vocation, VOCATION_NONE as u32);
        assert_eq!(voc.description, "");
    }

    // ----- get_promoted_vocation: self-referential edge case ----------------

    /// A vocation with from_vocation == its own id should NOT promote to itself.
    /// C++ condition: `it.second.fromVocation == id && it.first != id`
    #[test]
    fn test_get_promoted_vocation_self_reference_returns_none() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test" fromvoc="1"/>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        // voc 1 has fromVoc=1 (itself) — same id excluded, so no promotion
        assert_eq!(vocations.get_promoted_vocation(1), VOCATION_NONE);
    }

    // ----- vocations() iterator returns correct vocation data ---------------

    #[test]
    fn test_vocations_iterator_yields_correct_names() {
        let vocations = Vocations::load_from_xml(minimal_xml()).unwrap();
        let mut names: Vec<&str> = vocations.vocations().map(|v| v.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["Elder Sorcerer", "Sorcerer"]);
    }

    // ----- SKILL_BASE array constants ---------------------------------------

    #[test]
    fn test_skill_base_constants() {
        // C++: static const uint32_t skillBase[SKILL_LAST + 1] = {50, 50, 50, 50, 30, 100, 20}
        assert_eq!(SKILL_BASE, [50u32, 50, 50, 50, 30, 100, 20]);
    }

    // ----- required_skill_tries with custom skill multiplier ----------------

    #[test]
    fn test_required_skill_tries_custom_multiplier() {
        let mut voc = Vocation::new(1);
        // Set Fist (0) multiplier to 2.0, base = 50
        voc.skill_multipliers[0] = 2.0;
        // Level 13: exponent=2 → 50 * 2.0^2 = 200
        assert_eq!(voc.required_skill_tries(0, 13), 200);
    }

    // ----- XML: unknown attribute is silently skipped -----------------------

    /// C++ prints a `[Notice - Vocations::loadFromXml] Unknown attribute: ...`
    /// for any attribute name that is not in the known set. The vocation must
    /// still be created with defaults, and other recognised attributes on the
    /// same node must still be applied.
    #[test]
    fn test_load_from_xml_unknown_attribute_is_ignored() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test" totallyBogus="42" gainhp="9"/>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        assert_eq!(voc.name, "Test");
        assert_eq!(voc.gain_hp, 9);
    }

    // ----- XML: unknown child element is silently skipped -------------------

    /// C++ only inspects `<skill>` and `<formula>` children; any other child
    /// element is ignored. The vocation must still parse cleanly.
    #[test]
    fn test_load_from_xml_unknown_child_element_is_ignored() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test">
    <bogus value="ignored"/>
    <skill id="0" multiplier="2.5"/>
  </vocation>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        // The valid <skill> sibling must still be applied.
        assert!((voc.skill_multipliers[0] - 2.5).abs() < 1e-9);
    }

    // ----- XML: <skill> without a multiplier attribute ----------------------

    /// C++: `voc.skillMultipliers[skillId] = cast<double>(childNode.attribute("multiplier").value())`.
    /// pugixml's `.value()` on a missing attribute returns `""`, and
    /// `pugi::cast<double>("")` yields 0.0. The Rust port skips the assignment
    /// entirely (safer), so the default multiplier remains. Either behaviour
    /// preserves the post-condition that the vocation parses without error;
    /// here we assert the Rust behaviour: default multiplier preserved.
    #[test]
    fn test_load_from_xml_skill_without_multiplier_attribute() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test">
    <skill id="0"/>
  </vocation>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        // Default Fist multiplier is 1.5; must remain so when no multiplier given.
        assert!((voc.skill_multipliers[0] - 1.5).abs() < 1e-9);
    }

    // ----- XML: <skill> with a non-numeric multiplier value -----------------

    /// `mult_str.parse::<f64>()` fails → branch on line for the `Ok` arm is
    /// skipped → default multiplier preserved. Exercises the inner-`if-let`
    /// failure path of <skill> parsing.
    #[test]
    fn test_load_from_xml_skill_with_invalid_multiplier_value() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test">
    <skill id="0" multiplier="not-a-number"/>
  </vocation>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        // Default Fist multiplier 1.5 must remain when the value fails to parse.
        assert!((voc.skill_multipliers[0] - 1.5).abs() < 1e-9);
    }

    // ----- XML: <skill> with a non-numeric id value -------------------------

    /// `sid_str.parse::<u8>()` fails → the outer `if let Ok(sid) = ...` arm is
    /// skipped → default multipliers preserved. Exercises the id-parse failure
    /// branch.
    #[test]
    fn test_load_from_xml_skill_with_invalid_id_value() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test">
    <skill id="not-a-number" multiplier="9.9"/>
  </vocation>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        for (i, &default) in [1.5_f64, 2.0, 2.0, 2.0, 2.0, 1.5, 1.1].iter().enumerate() {
            assert!((voc.skill_multipliers[i] - default).abs() < 1e-9);
        }
    }

    // ----- XML: missing <vocations> root element ----------------------------

    /// Well-formed XML without a <vocations> root must be rejected with an
    /// error. This exercises the `ok_or_else(|| "Missing <vocations> root ...")`
    /// closure on the parse path.
    #[test]
    fn test_load_from_xml_missing_vocations_root_returns_err() {
        let xml = r#"<?xml version="1.0"?><other/>"#;
        let err = Vocations::load_from_xml(xml).err().expect("must error");
        assert!(err.contains("vocations"));
    }

    // ----- XML: invalid vocation id value -----------------------------------

    /// A vocation id that cannot be parsed as `u16` must produce an error.
    /// Exercises the `map_err(|_| format!("Invalid vocation id: {id_str}"))`
    /// closure on the id-parse path.
    #[test]
    fn test_load_from_xml_invalid_vocation_id_value_returns_err() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="not-a-number" name="Bad"/>
</vocations>"#;
        let err = Vocations::load_from_xml(xml).err().expect("must error");
        assert!(err.contains("Invalid vocation id"));
    }

    // ----- XML: <skill> without an id attribute -----------------------------

    /// C++ prints `[Notice] Missing skill id for vocation: <id>` and skips the
    /// child. The Rust port mirrors the silent-skip; the vocation must parse
    /// successfully and retain default multipliers.
    #[test]
    fn test_load_from_xml_skill_without_id_attribute() {
        let xml = r#"<?xml version="1.0"?>
<vocations>
  <vocation id="1" name="Test">
    <skill multiplier="9.9"/>
  </vocation>
</vocations>"#;
        let vocations = Vocations::load_from_xml(xml).unwrap();
        let voc = vocations.get_vocation(1).unwrap();
        for (i, &default) in [1.5_f64, 2.0, 2.0, 2.0, 2.0, 1.5, 1.1].iter().enumerate() {
            assert!((voc.skill_multipliers[i] - default).abs() < 1e-9);
        }
    }

    // ── get_magic_shield config-gating (Session 19) ─────────────────────

    /// Config disabled → return false regardless of per-vocation flag.
    #[test]
    fn get_magic_shield_config_off_overrides_vocation_flag() {
        use forgottenserver_common::configmanager::{BooleanKey, ConfigManager};
        let mut voc = Vocation::new(1);
        voc.magic_shield = true;
        let mut cm = ConfigManager::new();
        cm.set_boolean(BooleanKey::ManashieldBreakable, false);
        assert!(!voc.get_magic_shield(&cm), "config off must force false");
    }

    /// Config enabled + vocation flag true → true.
    #[test]
    fn get_magic_shield_config_on_returns_vocation_flag_true() {
        use forgottenserver_common::configmanager::{BooleanKey, ConfigManager};
        let mut voc = Vocation::new(1);
        voc.magic_shield = true;
        let mut cm = ConfigManager::new();
        cm.set_boolean(BooleanKey::ManashieldBreakable, true);
        assert!(voc.get_magic_shield(&cm));
    }

    /// Config enabled + vocation flag false → false.
    #[test]
    fn get_magic_shield_config_on_returns_vocation_flag_false() {
        use forgottenserver_common::configmanager::{BooleanKey, ConfigManager};
        let voc = Vocation::new(1); // default magic_shield = false
        let mut cm = ConfigManager::new();
        cm.set_boolean(BooleanKey::ManashieldBreakable, true);
        assert!(!voc.get_magic_shield(&cm));
    }
}
