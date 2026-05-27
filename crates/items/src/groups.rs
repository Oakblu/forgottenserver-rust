// Copyright 2023 The Forgotten Server Authors. All rights reserved.
// Use of this source code is governed by the GPL-2.0 License that can be found in the LICENSE file.

//! Player permission groups — migrated from groups.h / groups.cpp.
//!
//! A `Group` is a named permission profile (e.g. "Player", "Gamemaster").
//! Its `flags` field is a bitmask using the `PlayerFlags` constants from
//! `forgottenserver_common::constants`.

use forgottenserver_common::constants::PlayerFlags;

// ---------------------------------------------------------------------------
// Group
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Group` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub id: u16,
    pub name: String,
    /// Bitmask built from `PlayerFlags` constants.
    pub flags: u64,
    pub max_depot_items: u32,
    pub max_vip_entries: u32,
    pub access: bool,
}

impl Group {
    /// Returns `true` if this group has the given `PlayerFlags` bit set.
    pub fn has_flag(&self, flag: u64) -> bool {
        self.flags & flag != 0
    }
}

// ---------------------------------------------------------------------------
// Groups — registry
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Groups` class.
#[derive(Debug, Default)]
pub struct Groups {
    groups: Vec<Group>,
}

impl Groups {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse groups from an XML string.
    ///
    /// Expected format (mirrors data/XML/groups.xml):
    /// ```xml
    /// <groups>
    ///   <group id="1" name="Player" flags="0" maxdepotitems="2000" maxvipentries="200" access="0"/>
    ///   <group id="4" name="Gamemaster" flags="0" maxdepotitems="0" maxvipentries="0" access="1"/>
    /// </groups>
    /// ```
    ///
    /// `<flags>` child nodes with named boolean attributes are OR-ed into `flags`.
    pub fn load_from_xml(xml: &str) -> Result<Self, String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;

        let root = doc.root_element();
        if root.tag_name().name() != "groups" {
            return Err(format!(
                "expected root element <groups>, got <{}>",
                root.tag_name().name()
            ));
        }

        let mut groups = Vec::new();

        for node in root.children().filter(|n| n.is_element()) {
            let id: u16 = node
                .attribute("id")
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| "group missing or invalid 'id' attribute".to_string())?;

            let name = node.attribute("name").unwrap_or("").to_string();

            let access: bool = node
                .attribute("access")
                .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
                .unwrap_or(false);

            let max_depot_items: u32 = node
                .attribute("maxdepotitems")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);

            let max_vip_entries: u32 = node
                .attribute("maxvipentries")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);

            let mut flags: u64 = node
                .attribute("flags")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);

            // Parse named flag child elements (mirrors C++ ParsePlayerFlagMap).
            //
            // Two XML formats are supported:
            //
            // 1. Real groups.xml format (C++ canonical):
            //      <flags>
            //          <flag canbroadcast="1"/>
            //          <flag cannotbemuted="1"/>
            //      </flags>
            //    The element tag is "flag"; the *attribute name* is the flag key and
            //    the attribute value is truthy when non-zero / non-false.
            //
            // 2. Alternative format (tag name = flag key):
            //      <flags>
            //          <canbroadcast value="1"/>
            //      </flags>
            //    The element *tag name* is the flag key; a "value" (or first) attribute
            //    provides truthiness.
            if let Some(flags_node) = node.children().find(|n| n.tag_name().name() == "flags") {
                for flag_node in flags_node.children().filter(|n| n.is_element()) {
                    let tag = flag_node.tag_name().name();
                    if tag == "flag" {
                        // C++ canonical: attribute name = flag key, attribute value = "1"/"0".
                        for attr in flag_node.attributes() {
                            let flag_key = attr.name();
                            let enabled =
                                attr.value() != "0" && !attr.value().eq_ignore_ascii_case("false");
                            if enabled {
                                if let Some(bit) = flag_name_to_bit(flag_key) {
                                    flags |= bit;
                                }
                            }
                        }
                    } else {
                        // Alternative: tag name = flag key; "value" (or first) attr = truthiness.
                        let enabled = flag_node
                            .attribute("value")
                            .or_else(|| flag_node.attributes().next().map(|a| a.value()))
                            .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
                            .unwrap_or(true);
                        if enabled {
                            if let Some(bit) = flag_name_to_bit(tag) {
                                flags |= bit;
                            }
                        }
                    }
                }
            }

            groups.push(Group {
                id,
                name,
                flags,
                max_depot_items,
                max_vip_entries,
                access,
            });
        }

        Ok(Self { groups })
    }

    /// Returns a reference to the `Group` with the given `id`, or `None`.
    pub fn get_group(&self, id: u16) -> Option<&Group> {
        self.groups.iter().find(|g| g.id == id)
    }

    /// Returns a mutable reference to the `Group` with the given `id`, or `None`.
    pub fn get_group_mut(&mut self, id: u16) -> Option<&mut Group> {
        self.groups.iter_mut().find(|g| g.id == id)
    }

    /// Number of loaded groups.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Returns an iterator over all groups.
    pub fn iter(&self) -> impl Iterator<Item = &Group> {
        self.groups.iter()
    }
}

// ---------------------------------------------------------------------------
// Flag name mapping (mirrors ParsePlayerFlagMap in groups.cpp)
// ---------------------------------------------------------------------------

fn flag_name_to_bit(name: &str) -> Option<u64> {
    match name {
        "cannotusecombat" => Some(PlayerFlags::CANNOT_USE_COMBAT),
        "cannotattackplayer" => Some(PlayerFlags::CANNOT_ATTACK_PLAYER),
        "cannotattackmonster" => Some(PlayerFlags::CANNOT_ATTACK_MONSTER),
        "cannotbeattacked" => Some(PlayerFlags::CANNOT_BE_ATTACKED),
        "canconvinceall" => Some(PlayerFlags::CAN_CONVINCE_ALL),
        "cansummonall" => Some(PlayerFlags::CAN_SUMMON_ALL),
        "canillusionall" => Some(PlayerFlags::CAN_ILLUSION_ALL),
        "cansenseinvisibility" => Some(PlayerFlags::CAN_SENSE_INVISIBILITY),
        "ignoredbymonsters" => Some(PlayerFlags::IGNORED_BY_MONSTERS),
        "notgaininfight" => Some(PlayerFlags::NOT_GAIN_IN_FIGHT),
        "hasinfinitemana" => Some(PlayerFlags::HAS_INFINITE_MANA),
        "hasinfinitesoul" => Some(PlayerFlags::HAS_INFINITE_SOUL),
        "hasnoexhaustion" => Some(PlayerFlags::HAS_NO_EXHAUSTION),
        "cannotusespells" => Some(PlayerFlags::CANNOT_USE_SPELLS),
        "cannotpickupitem" => Some(PlayerFlags::CANNOT_PICKUP_ITEM),
        "canalwayslogin" => Some(PlayerFlags::CAN_ALWAYS_LOGIN),
        "canbroadcast" => Some(PlayerFlags::CAN_BROADCAST),
        "canedithouses" => Some(PlayerFlags::CAN_EDIT_HOUSES),
        "cannotbebanned" => Some(PlayerFlags::CANNOT_BE_BANNED),
        "cannotbepushed" => Some(PlayerFlags::CANNOT_BE_PUSHED),
        "hasinfinitecapacity" => Some(PlayerFlags::HAS_INFINITE_CAPACITY),
        "canpushallcreatures" => Some(PlayerFlags::CAN_PUSH_ALL_CREATURES),
        "cantalkredprivate" => Some(PlayerFlags::CAN_TALK_RED_PRIVATE),
        "cantalkredchannel" => Some(PlayerFlags::CAN_TALK_RED_CHANNEL),
        "talkorangehelpchannel" => Some(PlayerFlags::TALK_ORANGE_HELP_CHANNEL),
        "notgainexperience" => Some(PlayerFlags::NOT_GAIN_EXPERIENCE),
        "notgainmana" => Some(PlayerFlags::NOT_GAIN_MANA),
        "notgainhealth" => Some(PlayerFlags::NOT_GAIN_HEALTH),
        "notgainskill" => Some(PlayerFlags::NOT_GAIN_SKILL),
        "setmaxspeed" => Some(PlayerFlags::SET_MAX_SPEED),
        "specialvip" => Some(PlayerFlags::SPECIAL_VIP),
        "notgenerateloot" => Some(PlayerFlags::NOT_GENERATE_LOOT),
        "ignoreprotectionzone" => Some(PlayerFlags::IGNORE_PROTECTION_ZONE),
        "ignorespellcheck" => Some(PlayerFlags::IGNORE_SPELL_CHECK),
        "ignoreweaponcheck" => Some(PlayerFlags::IGNORE_WEAPON_CHECK),
        "cannotbemuted" => Some(PlayerFlags::CANNOT_BE_MUTED),
        "isalwayspremium" => Some(PlayerFlags::IS_ALWAYS_PREMIUM),
        "ignoreyellcheck" => Some(PlayerFlags::IGNORE_YELL_CHECK),
        "ignoresendprivatecheck" => Some(PlayerFlags::IGNORE_SEND_PRIVATE_CHECK),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Group::has_flag ----------------------------------------------------

    #[test]
    fn test_has_flag_set_returns_true() {
        let group = Group {
            id: 1,
            name: "Player".into(),
            flags: PlayerFlags::CAN_BROADCAST,
            max_depot_items: 2000,
            max_vip_entries: 200,
            access: false,
        };
        assert!(group.has_flag(PlayerFlags::CAN_BROADCAST));
    }

    #[test]
    fn test_has_flag_not_set_returns_false() {
        let group = Group {
            id: 1,
            name: "Player".into(),
            flags: PlayerFlags::CAN_BROADCAST,
            max_depot_items: 2000,
            max_vip_entries: 200,
            access: false,
        };
        assert!(!group.has_flag(PlayerFlags::CAN_EDIT_HOUSES));
    }

    #[test]
    fn test_has_flag_combined_flags() {
        let flags = PlayerFlags::CAN_BROADCAST | PlayerFlags::CAN_EDIT_HOUSES;
        let group = Group {
            id: 4,
            name: "GM".into(),
            flags,
            max_depot_items: 0,
            max_vip_entries: 0,
            access: true,
        };
        assert!(group.has_flag(PlayerFlags::CAN_BROADCAST));
        assert!(group.has_flag(PlayerFlags::CAN_EDIT_HOUSES));
        // Third flag must remain absent
        assert!(!group.has_flag(PlayerFlags::CAN_ALWAYS_LOGIN));
    }

    #[test]
    fn test_has_flag_zero_flags() {
        let group = Group {
            id: 1,
            name: "Nobody".into(),
            flags: 0,
            max_depot_items: 0,
            max_vip_entries: 0,
            access: false,
        };
        assert!(!group.has_flag(PlayerFlags::CAN_BROADCAST));
    }

    #[test]
    fn test_has_flag_all_flags_set() {
        let group = Group {
            id: 255,
            name: "God".into(),
            flags: u64::MAX,
            max_depot_items: 0,
            max_vip_entries: 0,
            access: true,
        };
        assert!(group.has_flag(PlayerFlags::IGNORE_SEND_PRIVATE_CHECK));
        assert!(group.has_flag(PlayerFlags::CANNOT_USE_COMBAT));
    }

    // --- Groups::load_from_xml ---------------------------------------------

    const MINIMAL_XML: &str = r#"
        <groups>
            <group id="1" name="Player" flags="0" maxdepotitems="2000" maxvipentries="200" access="0"/>
            <group id="4" name="Gamemaster" flags="0" maxdepotitems="0" maxvipentries="0" access="1"/>
        </groups>
    "#;

    #[test]
    fn test_load_from_xml_parses_correct_count() {
        let groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_load_from_xml_player_group_fields() {
        let groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        let player = groups.get_group(1).expect("group 1 should exist");
        assert_eq!(player.name, "Player");
        assert!(!player.access);
        assert_eq!(player.max_depot_items, 2000);
        assert_eq!(player.max_vip_entries, 200);
        assert_eq!(player.flags, 0);
    }

    #[test]
    fn test_load_from_xml_gm_group_has_access() {
        let groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        let gm = groups.get_group(4).expect("group 4 should exist");
        assert!(gm.access);
        assert_eq!(gm.name, "Gamemaster");
    }

    #[test]
    fn test_load_from_xml_unknown_id_returns_none() {
        let groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        assert!(groups.get_group(99).is_none());
    }

    #[test]
    fn test_load_from_xml_invalid_xml_returns_err() {
        let result = Groups::load_from_xml("<groups><unclosed></groups>");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_xml_named_flags_parsed() {
        let xml = r#"
            <groups>
                <group id="5" name="Tutor" flags="0" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <canbroadcast value="1"/>
                        <canalwayslogin value="1"/>
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let tutor = groups.get_group(5).unwrap();
        assert!(tutor.has_flag(PlayerFlags::CAN_BROADCAST));
        assert!(tutor.has_flag(PlayerFlags::CAN_ALWAYS_LOGIN));
        assert!(!tutor.has_flag(PlayerFlags::CAN_EDIT_HOUSES));
    }

    #[test]
    fn test_load_from_xml_disabled_named_flag_not_set() {
        let xml = r#"
            <groups>
                <group id="2" name="Senior Tutor" flags="0" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <canbroadcast value="0"/>
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(2).unwrap();
        assert!(!g.has_flag(PlayerFlags::CAN_BROADCAST));
    }

    #[test]
    fn test_load_from_xml_base_flags_plus_named_flags_merged() {
        // flags attr = 1 (CANNOT_USE_COMBAT), plus canbroadcast via child
        let xml = r#"
            <groups>
                <group id="3" name="Mixed" flags="1" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <canbroadcast value="1"/>
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(3).unwrap();
        assert!(g.has_flag(PlayerFlags::CANNOT_USE_COMBAT)); // from numeric flags="1"
        assert!(g.has_flag(PlayerFlags::CAN_BROADCAST)); // from <flags> child
    }

    #[test]
    fn test_load_from_xml_empty_groups() {
        let xml = "<groups/>";
        let groups = Groups::load_from_xml(xml).unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_groups_iter_returns_all() {
        let groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        let ids: Vec<u16> = groups.iter().map(|g| g.id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&4));
    }

    // --- Groups::new / len / is_empty ------------------------------------------

    #[test]
    fn test_groups_new_is_empty() {
        let g = Groups::new();
        assert!(g.is_empty());
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_groups_len_after_load() {
        let groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        assert_eq!(groups.len(), 2);
        assert!(!groups.is_empty());
    }

    // --- Groups::get_group_mut ------------------------------------------------

    #[test]
    fn test_get_group_mut_returns_some_for_existing_id() {
        let mut groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        assert!(groups.get_group_mut(1).is_some());
    }

    #[test]
    fn test_get_group_mut_returns_none_for_unknown_id() {
        let mut groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        assert!(groups.get_group_mut(99).is_none());
    }

    #[test]
    fn test_get_group_mut_allows_field_mutation() {
        let mut groups = Groups::load_from_xml(MINIMAL_XML).unwrap();
        {
            let g = groups.get_group_mut(1).unwrap();
            g.max_depot_items = 9999;
        }
        assert_eq!(groups.get_group(1).unwrap().max_depot_items, 9999);
    }

    // --- Group Clone / PartialEq -----------------------------------------------

    #[test]
    fn test_group_clone_is_equal() {
        let g = Group {
            id: 1,
            name: "Player".into(),
            flags: PlayerFlags::CAN_BROADCAST,
            max_depot_items: 2000,
            max_vip_entries: 200,
            access: false,
        };
        let cloned = g.clone();
        assert_eq!(g, cloned);
    }

    #[test]
    fn test_group_partial_eq_differs_on_id() {
        let a = Group {
            id: 1,
            name: "A".into(),
            flags: 0,
            max_depot_items: 0,
            max_vip_entries: 0,
            access: false,
        };
        let b = Group {
            id: 2,
            name: "A".into(),
            flags: 0,
            max_depot_items: 0,
            max_vip_entries: 0,
            access: false,
        };
        assert_ne!(a, b);
    }

    // --- Real XML format: <flag attrname="1"/> ---------------------------------

    #[test]
    fn test_load_from_xml_real_flag_element_format() {
        // Mirrors the canonical groups.xml format used by C++ pugixml parser.
        let xml = r#"
            <groups>
                <group id="2" name="tutor" access="0" maxdepotitems="0" maxvipentries="0">
                    <flags>
                        <flag talkorangehelpchannel="1" />
                        <flag cannotbemuted="1" />
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let tutor = groups.get_group(2).unwrap();
        assert!(tutor.has_flag(PlayerFlags::TALK_ORANGE_HELP_CHANNEL));
        assert!(tutor.has_flag(PlayerFlags::CANNOT_BE_MUTED));
        assert!(!tutor.has_flag(PlayerFlags::CAN_BROADCAST));
    }

    #[test]
    fn test_load_from_xml_real_format_disabled_flag_not_set() {
        let xml = r#"
            <groups>
                <group id="4" name="gamemaster" access="1" maxdepotitems="0" maxvipentries="200">
                    <flags>
                        <flag canconvinceall="0" />
                        <flag cansummonall="0" />
                        <flag canillusionall="1" />
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let gm = groups.get_group(4).unwrap();
        assert!(!gm.has_flag(PlayerFlags::CAN_CONVINCE_ALL));
        assert!(!gm.has_flag(PlayerFlags::CAN_SUMMON_ALL));
        assert!(gm.has_flag(PlayerFlags::CAN_ILLUSION_ALL));
    }

    #[test]
    fn test_load_from_xml_real_format_all_gm_flags() {
        // Mirrors the full gamemaster group from groups.xml.
        let xml = r#"
            <groups>
                <group id="4" name="gamemaster" access="1" maxdepotitems="0" maxvipentries="200">
                    <flags>
                        <flag cannotusecombat="1" />
                        <flag cannotattackplayer="1" />
                        <flag cannotattackmonster="1" />
                        <flag cannotbeattacked="1" />
                        <flag canconvinceall="0" />
                        <flag cansummonall="0" />
                        <flag canillusionall="1" />
                        <flag cansenseinvisibility="1" />
                        <flag ignoredbymonsters="1" />
                        <flag notgaininfight="1" />
                        <flag hasinfinitemana="0" />
                        <flag hasinfinitesoul="0" />
                        <flag hasnoexhaustion="1" />
                        <flag cannotusespells="1" />
                        <flag cannotpickupitem="1" />
                        <flag canalwayslogin="1" />
                        <flag canbroadcast="1" />
                        <flag canedithouses="0" />
                        <flag cannotbebanned="1" />
                        <flag cannotbepushed="1" />
                        <flag hasinfinitecapacity="1" />
                        <flag canpushallcreatures="1" />
                        <flag cantalkredprivate="1" />
                        <flag cantalkredchannel="1" />
                        <flag talkorangehelpchannel="1" />
                        <flag notgainexperience="1" />
                        <flag notgainmana="1" />
                        <flag notgainhealth="1" />
                        <flag notgainskill="1" />
                        <flag setmaxspeed="1" />
                        <flag specialvip="1" />
                        <flag notgenerateloot="1" />
                        <flag ignoreprotectionzone="1" />
                        <flag ignorespellcheck="1" />
                        <flag ignoreweaponcheck="1" />
                        <flag cannotbemuted="1" />
                        <flag isalwayspremium="1" />
                        <flag ignoreyellcheck="1" />
                        <flag ignoresendprivatecheck="1" />
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let gm = groups.get_group(4).unwrap();
        // Flags explicitly set to "1"
        assert!(gm.has_flag(PlayerFlags::CANNOT_USE_COMBAT));
        assert!(gm.has_flag(PlayerFlags::CANNOT_ATTACK_PLAYER));
        assert!(gm.has_flag(PlayerFlags::CANNOT_ATTACK_MONSTER));
        assert!(gm.has_flag(PlayerFlags::CANNOT_BE_ATTACKED));
        assert!(gm.has_flag(PlayerFlags::CAN_ILLUSION_ALL));
        assert!(gm.has_flag(PlayerFlags::CAN_SENSE_INVISIBILITY));
        assert!(gm.has_flag(PlayerFlags::IGNORED_BY_MONSTERS));
        assert!(gm.has_flag(PlayerFlags::NOT_GAIN_IN_FIGHT));
        assert!(gm.has_flag(PlayerFlags::HAS_NO_EXHAUSTION));
        assert!(gm.has_flag(PlayerFlags::CANNOT_USE_SPELLS));
        assert!(gm.has_flag(PlayerFlags::CANNOT_PICKUP_ITEM));
        assert!(gm.has_flag(PlayerFlags::CAN_ALWAYS_LOGIN));
        assert!(gm.has_flag(PlayerFlags::CAN_BROADCAST));
        assert!(gm.has_flag(PlayerFlags::CANNOT_BE_BANNED));
        assert!(gm.has_flag(PlayerFlags::CANNOT_BE_PUSHED));
        assert!(gm.has_flag(PlayerFlags::HAS_INFINITE_CAPACITY));
        assert!(gm.has_flag(PlayerFlags::CAN_PUSH_ALL_CREATURES));
        assert!(gm.has_flag(PlayerFlags::CAN_TALK_RED_PRIVATE));
        assert!(gm.has_flag(PlayerFlags::CAN_TALK_RED_CHANNEL));
        assert!(gm.has_flag(PlayerFlags::TALK_ORANGE_HELP_CHANNEL));
        assert!(gm.has_flag(PlayerFlags::NOT_GAIN_EXPERIENCE));
        assert!(gm.has_flag(PlayerFlags::NOT_GAIN_MANA));
        assert!(gm.has_flag(PlayerFlags::NOT_GAIN_HEALTH));
        assert!(gm.has_flag(PlayerFlags::NOT_GAIN_SKILL));
        assert!(gm.has_flag(PlayerFlags::SET_MAX_SPEED));
        assert!(gm.has_flag(PlayerFlags::SPECIAL_VIP));
        assert!(gm.has_flag(PlayerFlags::NOT_GENERATE_LOOT));
        assert!(gm.has_flag(PlayerFlags::IGNORE_PROTECTION_ZONE));
        assert!(gm.has_flag(PlayerFlags::IGNORE_SPELL_CHECK));
        assert!(gm.has_flag(PlayerFlags::IGNORE_WEAPON_CHECK));
        assert!(gm.has_flag(PlayerFlags::CANNOT_BE_MUTED));
        assert!(gm.has_flag(PlayerFlags::IS_ALWAYS_PREMIUM));
        assert!(gm.has_flag(PlayerFlags::IGNORE_YELL_CHECK));
        assert!(gm.has_flag(PlayerFlags::IGNORE_SEND_PRIVATE_CHECK));
        // Flags explicitly set to "0"
        assert!(!gm.has_flag(PlayerFlags::CAN_CONVINCE_ALL));
        assert!(!gm.has_flag(PlayerFlags::CAN_SUMMON_ALL));
        assert!(!gm.has_flag(PlayerFlags::HAS_INFINITE_MANA));
        assert!(!gm.has_flag(PlayerFlags::HAS_INFINITE_SOUL));
        assert!(!gm.has_flag(PlayerFlags::CAN_EDIT_HOUSES));
        // max_vip_entries as in groups.xml
        assert_eq!(gm.max_vip_entries, 200);
        assert!(gm.access);
    }

    // --- flag_name_to_bit: verify all 40 flag keys ----------------------------

    #[test]
    fn test_flag_name_to_bit_all_known_keys() {
        // Every entry from ParsePlayerFlagMap in groups.cpp must map to the
        // correct PlayerFlags constant.
        let cases: &[(&str, u64)] = &[
            ("cannotusecombat", PlayerFlags::CANNOT_USE_COMBAT),
            ("cannotattackplayer", PlayerFlags::CANNOT_ATTACK_PLAYER),
            ("cannotattackmonster", PlayerFlags::CANNOT_ATTACK_MONSTER),
            ("cannotbeattacked", PlayerFlags::CANNOT_BE_ATTACKED),
            ("canconvinceall", PlayerFlags::CAN_CONVINCE_ALL),
            ("cansummonall", PlayerFlags::CAN_SUMMON_ALL),
            ("canillusionall", PlayerFlags::CAN_ILLUSION_ALL),
            ("cansenseinvisibility", PlayerFlags::CAN_SENSE_INVISIBILITY),
            ("ignoredbymonsters", PlayerFlags::IGNORED_BY_MONSTERS),
            ("notgaininfight", PlayerFlags::NOT_GAIN_IN_FIGHT),
            ("hasinfinitemana", PlayerFlags::HAS_INFINITE_MANA),
            ("hasinfinitesoul", PlayerFlags::HAS_INFINITE_SOUL),
            ("hasnoexhaustion", PlayerFlags::HAS_NO_EXHAUSTION),
            ("cannotusespells", PlayerFlags::CANNOT_USE_SPELLS),
            ("cannotpickupitem", PlayerFlags::CANNOT_PICKUP_ITEM),
            ("canalwayslogin", PlayerFlags::CAN_ALWAYS_LOGIN),
            ("canbroadcast", PlayerFlags::CAN_BROADCAST),
            ("canedithouses", PlayerFlags::CAN_EDIT_HOUSES),
            ("cannotbebanned", PlayerFlags::CANNOT_BE_BANNED),
            ("cannotbepushed", PlayerFlags::CANNOT_BE_PUSHED),
            ("hasinfinitecapacity", PlayerFlags::HAS_INFINITE_CAPACITY),
            ("canpushallcreatures", PlayerFlags::CAN_PUSH_ALL_CREATURES),
            ("cantalkredprivate", PlayerFlags::CAN_TALK_RED_PRIVATE),
            ("cantalkredchannel", PlayerFlags::CAN_TALK_RED_CHANNEL),
            (
                "talkorangehelpchannel",
                PlayerFlags::TALK_ORANGE_HELP_CHANNEL,
            ),
            ("notgainexperience", PlayerFlags::NOT_GAIN_EXPERIENCE),
            ("notgainmana", PlayerFlags::NOT_GAIN_MANA),
            ("notgainhealth", PlayerFlags::NOT_GAIN_HEALTH),
            ("notgainskill", PlayerFlags::NOT_GAIN_SKILL),
            ("setmaxspeed", PlayerFlags::SET_MAX_SPEED),
            ("specialvip", PlayerFlags::SPECIAL_VIP),
            ("notgenerateloot", PlayerFlags::NOT_GENERATE_LOOT),
            ("ignoreprotectionzone", PlayerFlags::IGNORE_PROTECTION_ZONE),
            ("ignorespellcheck", PlayerFlags::IGNORE_SPELL_CHECK),
            ("ignoreweaponcheck", PlayerFlags::IGNORE_WEAPON_CHECK),
            ("cannotbemuted", PlayerFlags::CANNOT_BE_MUTED),
            ("isalwayspremium", PlayerFlags::IS_ALWAYS_PREMIUM),
            ("ignoreyellcheck", PlayerFlags::IGNORE_YELL_CHECK),
            (
                "ignoresendprivatecheck",
                PlayerFlags::IGNORE_SEND_PRIVATE_CHECK,
            ),
        ];
        for (key, expected_bit) in cases {
            let bit = flag_name_to_bit(key).expect("flag_name_to_bit returned None for known key");
            assert_eq!(
                bit, *expected_bit,
                "flag_name_to_bit({key:?}) = {bit:#018x}, want {expected_bit:#018x}"
            );
        }
    }

    #[test]
    fn test_flag_name_to_bit_unknown_key_returns_none() {
        assert!(flag_name_to_bit("banishment").is_none());
        assert!(flag_name_to_bit("namelock").is_none());
        assert!(flag_name_to_bit("notation").is_none());
        assert!(flag_name_to_bit("").is_none());
        assert!(flag_name_to_bit("CANBROADCAST").is_none()); // case-sensitive
    }

    // --- PlayerFlag bit values match C++ enum (spot-checks) -------------------

    #[test]
    fn test_player_flags_bit_positions_match_cpp() {
        // C++ const.h: enum PlayerFlags : uint64_t { PlayerFlag_CannotUseCombat = 1 << 0, ... }
        assert_eq!(PlayerFlags::CANNOT_USE_COMBAT, 1u64 << 0);
        assert_eq!(PlayerFlags::CANNOT_ATTACK_PLAYER, 1u64 << 1);
        assert_eq!(PlayerFlags::CANNOT_ATTACK_MONSTER, 1u64 << 2);
        assert_eq!(PlayerFlags::CANNOT_BE_ATTACKED, 1u64 << 3);
        assert_eq!(PlayerFlags::CAN_CONVINCE_ALL, 1u64 << 4);
        assert_eq!(PlayerFlags::CAN_SUMMON_ALL, 1u64 << 5);
        assert_eq!(PlayerFlags::CAN_ILLUSION_ALL, 1u64 << 6);
        assert_eq!(PlayerFlags::CAN_SENSE_INVISIBILITY, 1u64 << 7);
        assert_eq!(PlayerFlags::IGNORED_BY_MONSTERS, 1u64 << 8);
        assert_eq!(PlayerFlags::NOT_GAIN_IN_FIGHT, 1u64 << 9);
        assert_eq!(PlayerFlags::HAS_INFINITE_MANA, 1u64 << 10);
        assert_eq!(PlayerFlags::HAS_INFINITE_SOUL, 1u64 << 11);
        assert_eq!(PlayerFlags::HAS_NO_EXHAUSTION, 1u64 << 12);
        assert_eq!(PlayerFlags::CANNOT_USE_SPELLS, 1u64 << 13);
        assert_eq!(PlayerFlags::CANNOT_PICKUP_ITEM, 1u64 << 14);
        assert_eq!(PlayerFlags::CAN_ALWAYS_LOGIN, 1u64 << 15);
        assert_eq!(PlayerFlags::CAN_BROADCAST, 1u64 << 16);
        assert_eq!(PlayerFlags::CAN_EDIT_HOUSES, 1u64 << 17);
        assert_eq!(PlayerFlags::CANNOT_BE_BANNED, 1u64 << 18);
        assert_eq!(PlayerFlags::CANNOT_BE_PUSHED, 1u64 << 19);
        assert_eq!(PlayerFlags::HAS_INFINITE_CAPACITY, 1u64 << 20);
        assert_eq!(PlayerFlags::CAN_PUSH_ALL_CREATURES, 1u64 << 21);
        assert_eq!(PlayerFlags::CAN_TALK_RED_PRIVATE, 1u64 << 22);
        assert_eq!(PlayerFlags::CAN_TALK_RED_CHANNEL, 1u64 << 23);
        assert_eq!(PlayerFlags::TALK_ORANGE_HELP_CHANNEL, 1u64 << 24);
        assert_eq!(PlayerFlags::NOT_GAIN_EXPERIENCE, 1u64 << 25);
        assert_eq!(PlayerFlags::NOT_GAIN_MANA, 1u64 << 26);
        assert_eq!(PlayerFlags::NOT_GAIN_HEALTH, 1u64 << 27);
        assert_eq!(PlayerFlags::NOT_GAIN_SKILL, 1u64 << 28);
        assert_eq!(PlayerFlags::SET_MAX_SPEED, 1u64 << 29);
        assert_eq!(PlayerFlags::SPECIAL_VIP, 1u64 << 30);
        assert_eq!(PlayerFlags::NOT_GENERATE_LOOT, 1u64 << 31);
        // Bit 32 was deprecated in C++ (no flag there)
        assert_eq!(PlayerFlags::IGNORE_PROTECTION_ZONE, 1u64 << 33);
        assert_eq!(PlayerFlags::IGNORE_SPELL_CHECK, 1u64 << 34);
        assert_eq!(PlayerFlags::IGNORE_WEAPON_CHECK, 1u64 << 35);
        assert_eq!(PlayerFlags::CANNOT_BE_MUTED, 1u64 << 36);
        assert_eq!(PlayerFlags::IS_ALWAYS_PREMIUM, 1u64 << 37);
        assert_eq!(PlayerFlags::IGNORE_YELL_CHECK, 1u64 << 38);
        assert_eq!(PlayerFlags::IGNORE_SEND_PRIVATE_CHECK, 1u64 << 39);
    }

    // --- Access level field parsing -------------------------------------------

    #[test]
    fn test_load_from_xml_access_false_string() {
        let xml = r#"<groups><group id="1" name="x" access="false" maxdepotitems="0" maxvipentries="0"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert!(!groups.get_group(1).unwrap().access);
    }

    #[test]
    fn test_load_from_xml_access_true_string() {
        let xml = r#"<groups><group id="1" name="x" access="true" maxdepotitems="0" maxvipentries="0"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert!(groups.get_group(1).unwrap().access);
    }

    #[test]
    fn test_load_from_xml_access_1_means_true() {
        let xml = r#"<groups><group id="1" name="x" access="1" maxdepotitems="0" maxvipentries="0"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert!(groups.get_group(1).unwrap().access);
    }

    #[test]
    fn test_load_from_xml_access_absent_defaults_false() {
        let xml =
            r#"<groups><group id="1" name="x" maxdepotitems="0" maxvipentries="0"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert!(!groups.get_group(1).unwrap().access);
    }

    // --- max_depot_items / max_vip_entries defaults and values ----------------

    #[test]
    fn test_load_from_xml_max_depot_absent_defaults_zero() {
        let xml = r#"<groups><group id="1" name="x" access="0" maxvipentries="0"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert_eq!(groups.get_group(1).unwrap().max_depot_items, 0);
    }

    #[test]
    fn test_load_from_xml_max_vip_absent_defaults_zero() {
        let xml = r#"<groups><group id="1" name="x" access="0" maxdepotitems="500"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert_eq!(groups.get_group(1).unwrap().max_vip_entries, 0);
    }

    #[test]
    fn test_load_from_xml_max_vip_200() {
        let xml = r#"<groups><group id="4" name="gm" access="1" maxdepotitems="0" maxvipentries="200"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert_eq!(groups.get_group(4).unwrap().max_vip_entries, 200);
    }

    // --- Wrong root element ---------------------------------------------------

    #[test]
    fn test_load_from_xml_wrong_root_returns_err() {
        let result = Groups::load_from_xml("<items><group id=\"1\"/></items>");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("groups"),
            "error should mention expected root: {msg}"
        );
    }

    // --- Missing id attribute returns Err ------------------------------------

    #[test]
    fn test_load_from_xml_missing_id_returns_err() {
        let xml =
            r#"<groups><group name="x" access="0" maxdepotitems="0" maxvipentries="0"/></groups>"#;
        let result = Groups::load_from_xml(xml);
        assert!(result.is_err());
    }

    // --- Group name field is stored correctly ---------------------------------

    #[test]
    fn test_load_from_xml_name_stored_exactly() {
        let xml = r#"<groups><group id="3" name="senior tutor" access="0" maxdepotitems="0" maxvipentries="0"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert_eq!(groups.get_group(3).unwrap().name, "senior tutor");
    }

    #[test]
    fn test_load_from_xml_empty_name_stored_as_empty() {
        let xml = r#"<groups><group id="1" name="" access="0" maxdepotitems="0" maxvipentries="0"/></groups>"#;
        let groups = Groups::load_from_xml(xml).unwrap();
        assert_eq!(groups.get_group(1).unwrap().name, "");
    }

    // --- Multiple groups, order preserved ------------------------------------

    #[test]
    fn test_load_from_xml_multiple_groups_order_preserved() {
        let xml = r#"
            <groups>
                <group id="6" name="god" access="1" maxdepotitems="0" maxvipentries="200"/>
                <group id="1" name="player" access="0" maxdepotitems="0" maxvipentries="0"/>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let ids: Vec<u16> = groups.iter().map(|g| g.id).collect();
        assert_eq!(ids, vec![6, 1]);
    }

    // --- Unknown flag names are silently ignored in BOTH XML formats ---------

    #[test]
    fn test_load_from_xml_real_format_unknown_flag_ignored() {
        // C++ canonical format: <flag <unknown>="1"/>. The unknown key must not
        // map to any bit and must NOT cause an error — it is silently skipped
        // (mirrors the C++ behaviour where ParsePlayerFlagMap.find() returns end()).
        let xml = r#"
            <groups>
                <group id="11" name="UnknownReal" flags="0" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <flag totallyfakeflag="1" />
                        <flag canbroadcast="1" />
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(11).unwrap();
        // Unknown key contributes nothing; canbroadcast is set.
        assert_eq!(g.flags, PlayerFlags::CAN_BROADCAST);
    }

    #[test]
    fn test_load_from_xml_alt_format_unknown_flag_ignored() {
        // Alternative format: tag name is the flag key. An unknown tag must
        // hit the `flag_name_to_bit -> None` branch and be silently skipped.
        let xml = r#"
            <groups>
                <group id="12" name="UnknownAlt" flags="0" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <totallyfakeflag value="1"/>
                        <canbroadcast value="1"/>
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(12).unwrap();
        assert_eq!(g.flags, PlayerFlags::CAN_BROADCAST);
    }

    // --- Alternative format fallback: tag-name = flag key, attribute missing -

    #[test]
    fn test_load_from_xml_alt_format_no_value_attr_uses_first_attr() {
        // Alternative format (tag name = flag key). When no `value=` attribute
        // is present, the parser falls back to the first attribute. Here the
        // first (and only) attribute is `enabled="1"`, so the flag should be set.
        let xml = r#"
            <groups>
                <group id="8" name="AltFirstAttr" flags="0" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <canbroadcast enabled="1"/>
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(8).unwrap();
        assert!(g.has_flag(PlayerFlags::CAN_BROADCAST));
    }

    #[test]
    fn test_load_from_xml_alt_format_no_value_attr_disabled() {
        // Same alt-format path, but the fallback first-attribute value is "0",
        // so the flag should NOT be set.
        let xml = r#"
            <groups>
                <group id="9" name="AltFirstAttrOff" flags="0" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <canbroadcast enabled="0"/>
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(9).unwrap();
        assert!(!g.has_flag(PlayerFlags::CAN_BROADCAST));
    }

    #[test]
    fn test_load_from_xml_alt_format_no_attrs_defaults_enabled() {
        // Alt-format with neither `value` nor any other attribute. The fallback
        // chain yields None for the truthiness probe and `unwrap_or(true)` kicks
        // in, so the flag is set.
        let xml = r#"
            <groups>
                <group id="10" name="AltNoAttrs" flags="0" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <canbroadcast/>
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(10).unwrap();
        assert!(g.has_flag(PlayerFlags::CAN_BROADCAST));
    }

    // --- Numeric flags= attribute is OR-ed with named flags ------------------

    #[test]
    fn test_load_from_xml_real_format_numeric_flags_plus_named_flags() {
        // flags="1" = CANNOT_USE_COMBAT (bit 0); then add canbroadcast via <flag>
        let xml = r#"
            <groups>
                <group id="7" name="Mixed" flags="1" maxdepotitems="0" maxvipentries="0" access="0">
                    <flags>
                        <flag canbroadcast="1" />
                    </flags>
                </group>
            </groups>
        "#;
        let groups = Groups::load_from_xml(xml).unwrap();
        let g = groups.get_group(7).unwrap();
        assert!(g.has_flag(PlayerFlags::CANNOT_USE_COMBAT));
        assert!(g.has_flag(PlayerFlags::CAN_BROADCAST));
        assert!(!g.has_flag(PlayerFlags::CAN_EDIT_HOUSES));
    }
}
