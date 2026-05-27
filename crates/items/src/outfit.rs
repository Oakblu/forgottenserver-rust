//! Migrated from forgottenserver/src/outfit.h and outfit.cpp
//!
//! Provides outfit definitions and lookup functionality.
//! Corresponds to the C++ `Outfit` struct and `Outfits` class.

use forgottenserver_common::enums::PlayerSex;

/// Corresponds to the C++ `Outfit` struct (the outfit *definition*, not the
/// appearance data struct in `Outfit_t` / `enums.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitDefinition {
    pub name: String,
    pub look_type: u16,
    pub premium: bool,
    pub unlocked: bool,
}

/// Corresponds to the C++ `ProtocolOutfit` struct — a lightweight view used
/// when sending outfit lists over the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolOutfit {
    pub name: String,
    pub look_type: u16,
    pub addons: u8,
}

impl ProtocolOutfit {
    pub fn new(name: impl Into<String>, look_type: u16, addons: u8) -> Self {
        Self {
            name: name.into(),
            look_type,
            addons,
        }
    }
}

/// Corresponds to the C++ `Outfits` class.
///
/// Stores per-sex outfit lists loaded from XML.
#[derive(Debug, Default)]
pub struct Outfits {
    /// Index 0 = Female, index 1 = Male (mirrors C++ `outfits[PLAYERSEX_LAST + 1]`)
    outfits: [Vec<OutfitDefinition>; 2],
}

impl Outfits {
    /// Parse outfits from an XML string.
    ///
    /// XML format:
    /// ```xml
    /// <outfits>
    ///   <outfit type="0" looktype="128" name="Citizen" premium="0" unlocked="1"/>
    ///   <outfit type="1" looktype="136" name="Citizen" premium="0" unlocked="1"/>
    ///   <!-- enabled="0" skips an outfit -->
    /// </outfits>
    /// ```
    pub fn load_from_xml(xml: &str) -> Result<Outfits, String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;

        let root = doc
            .descendants()
            .find(|n| n.has_tag_name("outfits"))
            .ok_or_else(|| "Missing <outfits> root element".to_string())?;

        let mut outfits: [Vec<OutfitDefinition>; 2] = [Vec::new(), Vec::new()];

        for node in root.children().filter(|n| n.is_element()) {
            // Skip disabled outfits
            if let Some(enabled) = node.attribute("enabled") {
                if enabled == "0" || enabled.eq_ignore_ascii_case("false") {
                    continue;
                }
            }

            let type_attr = node
                .attribute("type")
                .ok_or_else(|| "Missing outfit type attribute".to_string())?;
            let sex_type: u8 = type_attr
                .parse()
                .map_err(|_| format!("Invalid outfit type: {type_attr}"))?;

            if sex_type > 1 {
                // C++ warns and continues on types > PLAYERSEX_LAST (which is 1)
                continue;
            }

            let look_type_str = node
                .attribute("looktype")
                .ok_or_else(|| "Missing looktype attribute on outfit".to_string())?;
            let look_type: u16 = look_type_str
                .parse()
                .map_err(|_| format!("Invalid looktype: {look_type_str}"))?;

            let name = node.attribute("name").unwrap_or("").to_string();
            let premium = node
                .attribute("premium")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            // C++ default for unlocked is `true` (as_bool(true))
            let unlocked = node
                .attribute("unlocked")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true);

            outfits[sex_type as usize].push(OutfitDefinition {
                name,
                look_type,
                premium,
                unlocked,
            });
        }

        Ok(Outfits { outfits })
    }

    /// Return all outfits for the given sex.
    pub fn get_outfits(&self, sex: PlayerSex) -> &[OutfitDefinition] {
        &self.outfits[sex as usize]
    }

    /// Return the outfit definition for a specific `look_type` and `sex`.
    ///
    /// Mirrors `Outfits::getOutfitByLookType(PlayerSex_t, uint16_t)`.
    pub fn get_outfit_by_look_type(
        &self,
        sex: PlayerSex,
        look_type: u16,
    ) -> Option<&OutfitDefinition> {
        self.outfits[sex as usize]
            .iter()
            .find(|o| o.look_type == look_type)
    }

    /// Search both sexes for a given `look_type`.
    ///
    /// Mirrors `Outfits::getOutfitByLookType(uint16_t)` (no sex parameter).
    pub fn find_outfit_by_look_type(&self, look_type: u16) -> Option<&OutfitDefinition> {
        for sex_outfits in &self.outfits {
            if let Some(o) = sex_outfits.iter().find(|o| o.look_type == look_type) {
                return Some(o);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_xml() -> &'static str {
        r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" looktype="128" name="Citizen" premium="0" unlocked="1"/>
  <outfit type="1" looktype="136" name="Citizen" premium="0" unlocked="1"/>
  <outfit type="0" looktype="200" name="Warrior" premium="1" unlocked="0"/>
  <outfit type="1" looktype="201" name="Warrior" premium="1" unlocked="0"/>
  <outfit type="0" looktype="999" name="Disabled" enabled="0"/>
</outfits>"#
    }

    // ----- load_from_xml: happy path ----------------------------------------

    #[test]
    fn test_load_from_xml_parses_female_outfits() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let female = outfits.get_outfits(PlayerSex::Female);
        assert_eq!(
            female.len(),
            2,
            "expected 2 female outfits (disabled one skipped)"
        );
    }

    #[test]
    fn test_load_from_xml_parses_male_outfits() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let male = outfits.get_outfits(PlayerSex::Male);
        assert_eq!(male.len(), 2);
    }

    #[test]
    fn test_load_from_xml_correct_fields() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let citizen = &outfits.get_outfits(PlayerSex::Female)[0];
        assert_eq!(citizen.look_type, 128);
        assert_eq!(citizen.name, "Citizen");
        assert!(!citizen.premium);
        assert!(citizen.unlocked);
    }

    #[test]
    fn test_load_from_xml_premium_and_locked_fields() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let warrior = &outfits.get_outfits(PlayerSex::Female)[1];
        assert_eq!(warrior.look_type, 200);
        assert_eq!(warrior.name, "Warrior");
        assert!(warrior.premium);
        assert!(!warrior.unlocked);
    }

    #[test]
    fn test_load_from_xml_skips_disabled_outfits() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let female = outfits.get_outfits(PlayerSex::Female);
        assert!(
            female.iter().all(|o| o.look_type != 999),
            "disabled outfit (looktype=999) must be skipped"
        );
    }

    // ----- get_outfit_by_look_type ------------------------------------------

    #[test]
    fn test_get_outfit_by_look_type_found_female() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let outfit = outfits.get_outfit_by_look_type(PlayerSex::Female, 128);
        assert!(outfit.is_some());
        assert_eq!(outfit.unwrap().name, "Citizen");
    }

    #[test]
    fn test_get_outfit_by_look_type_found_male() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let outfit = outfits.get_outfit_by_look_type(PlayerSex::Male, 136);
        assert!(outfit.is_some());
        assert_eq!(outfit.unwrap().name, "Citizen");
    }

    #[test]
    fn test_get_outfit_by_look_type_not_found() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let outfit = outfits.get_outfit_by_look_type(PlayerSex::Female, 9999);
        assert!(outfit.is_none());
    }

    #[test]
    fn test_get_outfit_by_look_type_wrong_sex_returns_none() {
        // looktype 136 is male-only; should not be found for female
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        let outfit = outfits.get_outfit_by_look_type(PlayerSex::Female, 136);
        assert!(outfit.is_none());
    }

    // ----- find_outfit_by_look_type (no sex param) --------------------------

    #[test]
    fn test_find_outfit_by_look_type_finds_female() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        assert!(outfits.find_outfit_by_look_type(128).is_some());
    }

    #[test]
    fn test_find_outfit_by_look_type_finds_male() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        assert!(outfits.find_outfit_by_look_type(136).is_some());
    }

    #[test]
    fn test_find_outfit_by_look_type_unknown_returns_none() {
        let outfits = Outfits::load_from_xml(minimal_xml()).unwrap();
        assert!(outfits.find_outfit_by_look_type(0xFFFF).is_none());
    }

    // ----- XML error cases --------------------------------------------------

    #[test]
    fn test_load_from_xml_invalid_xml_returns_err() {
        let result = Outfits::load_from_xml("<bad xml<<<");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_xml_missing_looktype_returns_err() {
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" name="NoBall"/>
</outfits>"#;
        let result = Outfits::load_from_xml(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_xml_invalid_sex_type_skipped() {
        // type > 1 (> PLAYERSEX_LAST) should be silently skipped
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="99" looktype="1" name="Bad"/>
</outfits>"#;
        let outfits = Outfits::load_from_xml(xml).unwrap();
        assert!(outfits.get_outfits(PlayerSex::Female).is_empty());
        assert!(outfits.get_outfits(PlayerSex::Male).is_empty());
    }

    // ----- ProtocolOutfit ---------------------------------------------------

    #[test]
    fn test_protocol_outfit_new() {
        let po = ProtocolOutfit::new("Citizen", 128, 3);
        assert_eq!(po.name, "Citizen");
        assert_eq!(po.look_type, 128);
        assert_eq!(po.addons, 3);
    }

    #[test]
    fn test_outfit_definition_equality() {
        let a = OutfitDefinition {
            name: "A".into(),
            look_type: 1,
            premium: false,
            unlocked: true,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ----- unlocked default is true -----------------------------------------
    #[test]
    fn test_unlocked_defaults_to_true_when_attribute_absent() {
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" looktype="5" name="NoUnlockedAttr"/>
</outfits>"#;
        let outfits = Outfits::load_from_xml(xml).unwrap();
        let o = &outfits.get_outfits(PlayerSex::Female)[0];
        assert!(
            o.unlocked,
            "unlocked should default to true when attribute absent"
        );
    }

    // ----- addon bitmask 0–3 coverage ---------------------------------------

    #[test]
    fn test_protocol_outfit_addons_no_addon() {
        let po = ProtocolOutfit::new("Citizen", 128, 0);
        assert_eq!(po.addons, 0, "addon bitmask 0 = no addons");
    }

    #[test]
    fn test_protocol_outfit_addons_first_addon() {
        let po = ProtocolOutfit::new("Citizen", 128, 1);
        assert_eq!(po.addons, 1, "addon bitmask 1 = first addon only");
    }

    #[test]
    fn test_protocol_outfit_addons_second_addon() {
        let po = ProtocolOutfit::new("Citizen", 128, 2);
        assert_eq!(po.addons, 2, "addon bitmask 2 = second addon only");
    }

    #[test]
    fn test_protocol_outfit_addons_both_addons() {
        let po = ProtocolOutfit::new("Citizen", 128, 3);
        assert_eq!(po.addons, 3, "addon bitmask 3 = both addons");
    }

    // ----- OutfitDefinition inequality (PartialEq asymmetric cases) ---------

    #[test]
    fn test_outfit_definition_inequality_different_name() {
        let a = OutfitDefinition {
            name: "A".into(),
            look_type: 1,
            premium: false,
            unlocked: true,
        };
        let b = OutfitDefinition {
            name: "B".into(),
            look_type: 1,
            premium: false,
            unlocked: true,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_outfit_definition_inequality_different_look_type() {
        let a = OutfitDefinition {
            name: "A".into(),
            look_type: 1,
            premium: false,
            unlocked: true,
        };
        let b = OutfitDefinition {
            name: "A".into(),
            look_type: 2,
            premium: false,
            unlocked: true,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_outfit_definition_inequality_different_premium() {
        let a = OutfitDefinition {
            name: "A".into(),
            look_type: 1,
            premium: false,
            unlocked: true,
        };
        let b = OutfitDefinition {
            name: "A".into(),
            look_type: 1,
            premium: true,
            unlocked: true,
        };
        assert_ne!(a, b);
    }

    #[test]
    fn test_outfit_definition_inequality_different_unlocked() {
        let a = OutfitDefinition {
            name: "A".into(),
            look_type: 1,
            premium: false,
            unlocked: true,
        };
        let b = OutfitDefinition {
            name: "A".into(),
            look_type: 1,
            premium: false,
            unlocked: false,
        };
        assert_ne!(a, b);
    }

    // ----- enabled="false" (string) skips outfit ----------------------------

    #[test]
    fn test_load_from_xml_skips_disabled_outfit_false_string() {
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" looktype="777" name="FalseDisabled" enabled="false"/>
  <outfit type="0" looktype="1" name="Active"/>
</outfits>"#;
        let outfits = Outfits::load_from_xml(xml).unwrap();
        let female = outfits.get_outfits(PlayerSex::Female);
        assert_eq!(female.len(), 1, "enabled=false outfit must be skipped");
        assert_eq!(female[0].look_type, 1);
    }

    // ----- premium defaults to false when attribute absent ------------------

    #[test]
    fn test_premium_defaults_to_false_when_attribute_absent() {
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" looktype="10" name="NoPremiumAttr"/>
</outfits>"#;
        let outfits = Outfits::load_from_xml(xml).unwrap();
        let o = &outfits.get_outfits(PlayerSex::Female)[0];
        assert!(
            !o.premium,
            "premium should default to false when attribute absent"
        );
    }

    // ----- missing type attribute returns Err -------------------------------

    #[test]
    fn test_load_from_xml_missing_type_attribute_returns_err() {
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit looktype="1" name="NoType"/>
</outfits>"#;
        let result = Outfits::load_from_xml(xml);
        assert!(result.is_err(), "missing type attribute must return Err");
    }

    // ----- find_outfit_by_look_type searches female (index 0) before male ---

    #[test]
    fn test_find_outfit_by_look_type_female_searched_first() {
        // Same look_type in both sexes — female entry is returned (index 0 searched first)
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" looktype="50" name="FemaleFirst"/>
  <outfit type="1" looktype="50" name="MaleSecond"/>
</outfits>"#;
        let outfits = Outfits::load_from_xml(xml).unwrap();
        let found = outfits.find_outfit_by_look_type(50).unwrap();
        assert_eq!(
            found.name, "FemaleFirst",
            "female (index 0) must be returned when both sexes share a look_type"
        );
    }

    // ----- get_outfits returns empty slice on empty registry ----------------

    #[test]
    fn test_get_outfits_empty_on_default() {
        let outfits = Outfits::default();
        assert!(outfits.get_outfits(PlayerSex::Female).is_empty());
        assert!(outfits.get_outfits(PlayerSex::Male).is_empty());
    }

    // ----- ProtocolOutfit equality (PartialEq) ------------------------------

    #[test]
    fn test_protocol_outfit_equality() {
        let a = ProtocolOutfit::new("Citizen", 128, 3);
        let b = ProtocolOutfit::new("Citizen", 128, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn test_protocol_outfit_inequality_different_addons() {
        let a = ProtocolOutfit::new("Citizen", 128, 1);
        let b = ProtocolOutfit::new("Citizen", 128, 2);
        assert_ne!(a, b);
    }

    // ----- enabled="1" (truthy) does NOT skip the outfit (fall-through path) -

    #[test]
    fn test_load_from_xml_enabled_attribute_truthy_does_not_skip() {
        // C++ behaviour: `attr.as_bool()` on "1"/"true" returns true, so the
        // outfit is NOT skipped. This exercises the fall-through branch where
        // the `enabled` attribute is present but the skip condition is false.
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" looktype="42" name="EnabledOne" enabled="1"/>
  <outfit type="0" looktype="43" name="EnabledTrue" enabled="true"/>
</outfits>"#;
        let outfits = Outfits::load_from_xml(xml).unwrap();
        let female = outfits.get_outfits(PlayerSex::Female);
        assert_eq!(
            female.len(),
            2,
            "outfits with truthy enabled attribute must NOT be skipped"
        );
        assert_eq!(female[0].look_type, 42);
        assert_eq!(female[1].look_type, 43);
    }

    // ----- non-numeric type attribute returns Err ---------------------------

    #[test]
    fn test_load_from_xml_non_numeric_type_attribute_returns_err() {
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="abc" looktype="1" name="BadType"/>
</outfits>"#;
        let result = Outfits::load_from_xml(xml);
        assert!(
            result.is_err(),
            "non-numeric type attribute must return Err"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("Invalid outfit type"),
            "error must mention invalid type: got `{msg}`"
        );
    }

    // ----- non-numeric looktype attribute returns Err -----------------------

    #[test]
    fn test_load_from_xml_non_numeric_looktype_attribute_returns_err() {
        let xml = r#"<?xml version="1.0"?>
<outfits>
  <outfit type="0" looktype="xyz" name="BadLook"/>
</outfits>"#;
        let result = Outfits::load_from_xml(xml);
        assert!(
            result.is_err(),
            "non-numeric looktype attribute must return Err"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("Invalid looktype"),
            "error must mention invalid looktype: got `{msg}`"
        );
    }

    // ----- missing <outfits> root element returns Err -----------------------

    #[test]
    fn test_load_from_xml_missing_root_element_returns_err() {
        // Well-formed XML but no <outfits> element anywhere.
        let xml = r#"<?xml version="1.0"?><something_else/>"#;
        let result = Outfits::load_from_xml(xml);
        assert!(result.is_err(), "missing <outfits> root must return Err");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("Missing <outfits> root element"),
            "error must mention missing root: got `{msg}`",
        );
    }
}
