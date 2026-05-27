//! Migrated from forgottenserver/src/itemloader.h
//!
//! Constants, enums, and structs used when loading item types from OTB
//! (Open Tibia Binary) files.  All discriminant values are kept identical to
//! the C++ originals so that serialised OTB data is wire-compatible.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// itemgroup_t
// ---------------------------------------------------------------------------

/// Item group classification.  Mirrors `itemgroup_t` from `itemloader.h`.
///
/// Several variants are marked deprecated in the original C++; they are
/// preserved here for full OTB compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ItemGroup {
    None = 0,
    Ground = 1,
    Container = 2,
    Weapon = 3,     // deprecated
    Ammunition = 4, // deprecated
    Armor = 5,      // deprecated
    Charges = 6,
    Teleport = 7,   // deprecated
    MagicField = 8, // deprecated
    Writeable = 9,  // deprecated
    Key = 10,       // deprecated
    Splash = 11,
    Fluid = 12,
    Door = 13, // deprecated
    Deprecated = 14,
    Podium = 15,
    Last = 16,
}

// ---------------------------------------------------------------------------
// clientVersion_t
// ---------------------------------------------------------------------------

/// OTB client-version identifiers.  Mirrors `clientVersion_t` from
/// `itemloader.h`.  Note that `V760` and `V770` share the same value (3) in
/// the C++, as do several other pairs — those aliases are exposed as
/// associated constants below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ClientVersion {
    V750 = 1,
    V755 = 2,
    V760 = 3,
    // V770 = 3 (alias — see CLIENT_VERSION_770 below)
    V780 = 4,
    V790 = 5,
    V792 = 6,
    V800 = 7,
    V810 = 8,
    V811 = 9,
    V820 = 10,
    V830 = 11,
    V840 = 12,
    V841 = 13,
    V842 = 14,
    V850 = 15,
    V854Bad = 16,
    V854 = 17,
    V855 = 18,
    V860Old = 19,
    V860 = 20,
    V861 = 21,
    V862 = 22,
    V870 = 23,
    V871 = 24,
    V872 = 25,
    V873 = 26,
    V900 = 27,
    V910 = 28,
    V920 = 29,
    V940 = 30,
    V944V1 = 31,
    V944V2 = 32,
    V944V3 = 33,
    V944V4 = 34,
    V946 = 35,
    V950 = 36,
    V952 = 37,
    V953 = 38,
    V954 = 39,
    V960 = 40,
    V961 = 41,
    V963 = 42,
    V970 = 43,
    V980 = 44,
    V981 = 45,
    V982 = 46,
    V983 = 47,
    V985 = 48,
    V986 = 49,
    V1010 = 50,
    V1020 = 51,
    V1021 = 52,
    V1030 = 53,
    V1031 = 54,
    V1035 = 55,
    V1076 = 56,
    V1098 = 57,
    V1100 = 58,
    V1272 = 59,
    V1281 = 60,
    V1285 = 61,
    V1286 = 62,
    V1287 = 63,
    V1290 = 64,
    V1310 = 65,
}

/// Alias: `CLIENT_VERSION_770 == CLIENT_VERSION_760 == 3`.
pub const CLIENT_VERSION_770: u32 = ClientVersion::V760 as u32;
/// The last defined client version value.
pub const CLIENT_VERSION_LAST: u32 = ClientVersion::V1310 as u32;

// ---------------------------------------------------------------------------
// rootattrib_ (root attribute tags)
// ---------------------------------------------------------------------------

/// Root-node attribute tags in the OTB format.  Mirrors `rootattrib_`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RootAttr {
    Version = 0x01,
}

// ---------------------------------------------------------------------------
// itemattrib_t (per-item attribute tags)
// ---------------------------------------------------------------------------

/// Per-item attribute tags stored in an OTB item node.  Mirrors `itemattrib_t`.
///
/// The first attribute starts at `0x10`; subsequent ones are sequential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ItemAttr {
    // ITEM_ATTR_FIRST = 0x10, ITEM_ATTR_SERVERID = ITEM_ATTR_FIRST
    ServerId = 0x10,
    ClientId = 0x11,
    Name = 0x12,
    Descr = 0x13,
    Speed = 0x14,
    Slot = 0x15,
    MaxItems = 0x16,
    Weight = 0x17,
    Weapon = 0x18,
    Amu = 0x19,
    Armor = 0x1A,
    MagLevel = 0x1B,
    MagFieldType = 0x1C,
    Writeable = 0x1D,
    RotateTo = 0x1E,
    Decay = 0x1F,
    SpriteHash = 0x20,
    MinimapColor = 0x21,
    Attr07 = 0x22,
    Attr08 = 0x23,
    Light = 0x24,
    // 1-byte aligned section
    Decay2 = 0x25,     // deprecated
    Weapon2 = 0x26,    // deprecated
    Amu2 = 0x27,       // deprecated
    Armor2 = 0x28,     // deprecated
    Writeable2 = 0x29, // deprecated
    Light2 = 0x2A,
    TopOrder = 0x2B,
    Writeable3 = 0x2C, // deprecated
    WareId = 0x2D,
    Classification = 0x2E,
    // ITEM_ATTR_LAST — sentinel (not a real attribute)
    Last = 0x2F,
}

/// The first valid item attribute tag value (mirrors `ITEM_ATTR_FIRST`).
pub const ITEM_ATTR_FIRST: u8 = ItemAttr::ServerId as u8;
/// One-past-the-last valid item attribute tag (mirrors `ITEM_ATTR_LAST`).
pub const ITEM_ATTR_LAST: u8 = ItemAttr::Last as u8;

// ---------------------------------------------------------------------------
// itemflags_t (bitfield flags)
// ---------------------------------------------------------------------------

/// Bit-flags describing item properties in an OTB file.  Mirrors `itemflags_t`.
///
/// Several flags are marked unused in the original C++; they are preserved for
/// bit-level compatibility with existing OTB data.
pub mod item_flags {
    pub const FLAG_BLOCK_SOLID: u32 = 1 << 0;
    pub const FLAG_BLOCK_PROJECTILE: u32 = 1 << 1;
    pub const FLAG_BLOCK_PATHFIND: u32 = 1 << 2;
    pub const FLAG_HAS_HEIGHT: u32 = 1 << 3;
    pub const FLAG_USEABLE: u32 = 1 << 4;
    pub const FLAG_PICKUPABLE: u32 = 1 << 5;
    pub const FLAG_MOVEABLE: u32 = 1 << 6;
    pub const FLAG_STACKABLE: u32 = 1 << 7;
    pub const FLAG_FLOORCHANGEDOWN: u32 = 1 << 8; // unused
    pub const FLAG_FLOORCHANGENORTH: u32 = 1 << 9; // unused
    pub const FLAG_FLOORCHANGEEAST: u32 = 1 << 10; // unused
    pub const FLAG_FLOORCHANGESOUTH: u32 = 1 << 11; // unused
    pub const FLAG_FLOORCHANGEWEST: u32 = 1 << 12; // unused
    pub const FLAG_ALWAYSONTOP: u32 = 1 << 13;
    pub const FLAG_READABLE: u32 = 1 << 14;
    pub const FLAG_ROTATABLE: u32 = 1 << 15;
    pub const FLAG_HANGABLE: u32 = 1 << 16;
    pub const FLAG_VERTICAL: u32 = 1 << 17;
    pub const FLAG_HORIZONTAL: u32 = 1 << 18;
    pub const FLAG_CANNOTDECAY: u32 = 1 << 19; // unused
    pub const FLAG_ALLOWDISTREAD: u32 = 1 << 20;
    pub const FLAG_CLIENTDURATION: u32 = 1 << 21;
    pub const FLAG_CLIENTCHARGES: u32 = 1 << 22;
    pub const FLAG_LOOKTHROUGH: u32 = 1 << 23;
    pub const FLAG_ANIMATION: u32 = 1 << 24;
    pub const FLAG_FULLTILE: u32 = 1 << 25; // unused
    pub const FLAG_FORCEUSE: u32 = 1 << 26;
    pub const FLAG_AMMO: u32 = 1 << 27; // unused
    pub const FLAG_REPORTABLE: u32 = 1 << 28; // unused
}

// ---------------------------------------------------------------------------
// VERSIONINFO (1-byte aligned struct)
// ---------------------------------------------------------------------------

/// OTB root version block.  Mirrors `struct VERSIONINFO` (`#pragma pack(1)`).
///
/// The C++ layout is 4+4+4+128 = 140 bytes, all little-endian on x86.
/// We keep the same field names and types; the CSD version string is stored
/// as a fixed-size byte array to match the wire layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    /// ASCII/CSD version string, NUL-padded to 128 bytes.
    pub csd_version: [u8; 128],
}

impl Default for VersionInfo {
    fn default() -> Self {
        VersionInfo {
            major_version: 0,
            minor_version: 0,
            build_number: 0,
            csd_version: [0u8; 128],
        }
    }
}

// ---------------------------------------------------------------------------
// lightBlock2 (1-byte aligned struct)
// ---------------------------------------------------------------------------

/// Light information embedded in item attribute blocks.
/// Mirrors `struct lightBlock2` (`#pragma pack(1)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LightBlock2 {
    pub light_level: u16,
    pub light_color: u16,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::item_flags::*;
    use super::*;

    // -----------------------------------------------------------------------
    // ItemGroup discriminants
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_group_none() {
        assert_eq!(ItemGroup::None as u32, 0);
    }

    #[test]
    fn test_item_group_ground() {
        assert_eq!(ItemGroup::Ground as u32, 1);
    }

    #[test]
    fn test_item_group_container() {
        assert_eq!(ItemGroup::Container as u32, 2);
    }

    #[test]
    fn test_item_group_weapon_deprecated() {
        assert_eq!(ItemGroup::Weapon as u32, 3);
    }

    #[test]
    fn test_item_group_ammunition_deprecated() {
        assert_eq!(ItemGroup::Ammunition as u32, 4);
    }

    #[test]
    fn test_item_group_armor_deprecated() {
        assert_eq!(ItemGroup::Armor as u32, 5);
    }

    #[test]
    fn test_item_group_charges() {
        assert_eq!(ItemGroup::Charges as u32, 6);
    }

    #[test]
    fn test_item_group_teleport_deprecated() {
        assert_eq!(ItemGroup::Teleport as u32, 7);
    }

    #[test]
    fn test_item_group_magic_field_deprecated() {
        assert_eq!(ItemGroup::MagicField as u32, 8);
    }

    #[test]
    fn test_item_group_writeable_deprecated() {
        assert_eq!(ItemGroup::Writeable as u32, 9);
    }

    #[test]
    fn test_item_group_key_deprecated() {
        assert_eq!(ItemGroup::Key as u32, 10);
    }

    #[test]
    fn test_item_group_splash() {
        assert_eq!(ItemGroup::Splash as u32, 11);
    }

    #[test]
    fn test_item_group_fluid() {
        assert_eq!(ItemGroup::Fluid as u32, 12);
    }

    #[test]
    fn test_item_group_door_deprecated() {
        assert_eq!(ItemGroup::Door as u32, 13);
    }

    #[test]
    fn test_item_group_deprecated_value() {
        assert_eq!(ItemGroup::Deprecated as u32, 14);
    }

    #[test]
    fn test_item_group_podium() {
        assert_eq!(ItemGroup::Podium as u32, 15);
    }

    #[test]
    fn test_item_group_last() {
        assert_eq!(ItemGroup::Last as u32, 16);
    }

    // -----------------------------------------------------------------------
    // ClientVersion discriminants (spot-check key versions + aliases)
    // -----------------------------------------------------------------------

    #[test]
    fn test_client_version_750() {
        assert_eq!(ClientVersion::V750 as u32, 1);
    }

    #[test]
    fn test_client_version_760() {
        assert_eq!(ClientVersion::V760 as u32, 3);
    }

    #[test]
    fn test_client_version_770_alias() {
        // CLIENT_VERSION_770 == CLIENT_VERSION_760 == 3
        assert_eq!(CLIENT_VERSION_770, 3);
        assert_eq!(CLIENT_VERSION_770, ClientVersion::V760 as u32);
    }

    #[test]
    fn test_client_version_854_bad() {
        assert_eq!(ClientVersion::V854Bad as u32, 16);
    }

    #[test]
    fn test_client_version_854() {
        assert_eq!(ClientVersion::V854 as u32, 17);
    }

    #[test]
    fn test_client_version_1310() {
        assert_eq!(ClientVersion::V1310 as u32, 65);
    }

    #[test]
    fn test_client_version_last_constant() {
        assert_eq!(CLIENT_VERSION_LAST, 65);
        assert_eq!(CLIENT_VERSION_LAST, ClientVersion::V1310 as u32);
    }

    #[test]
    fn test_client_version_944_variants() {
        assert_eq!(ClientVersion::V944V1 as u32, 31);
        assert_eq!(ClientVersion::V944V2 as u32, 32);
        assert_eq!(ClientVersion::V944V3 as u32, 33);
        assert_eq!(ClientVersion::V944V4 as u32, 34);
    }

    // -----------------------------------------------------------------------
    // RootAttr
    // -----------------------------------------------------------------------

    #[test]
    fn test_root_attr_version() {
        assert_eq!(RootAttr::Version as u8, 0x01);
    }

    // -----------------------------------------------------------------------
    // ItemAttr — first, last, and selected middle values
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_attr_first_constant() {
        assert_eq!(ITEM_ATTR_FIRST, 0x10);
    }

    #[test]
    fn test_item_attr_server_id_equals_first() {
        assert_eq!(ItemAttr::ServerId as u8, ITEM_ATTR_FIRST);
    }

    #[test]
    fn test_item_attr_client_id() {
        assert_eq!(ItemAttr::ClientId as u8, 0x11);
    }

    #[test]
    fn test_item_attr_name() {
        assert_eq!(ItemAttr::Name as u8, 0x12);
    }

    #[test]
    fn test_item_attr_light() {
        assert_eq!(ItemAttr::Light as u8, 0x24);
    }

    #[test]
    fn test_item_attr_light2() {
        assert_eq!(ItemAttr::Light2 as u8, 0x2A);
    }

    #[test]
    fn test_item_attr_ware_id() {
        assert_eq!(ItemAttr::WareId as u8, 0x2D);
    }

    #[test]
    fn test_item_attr_classification() {
        assert_eq!(ItemAttr::Classification as u8, 0x2E);
    }

    #[test]
    fn test_item_attr_last_constant() {
        assert_eq!(ITEM_ATTR_LAST, 0x2F);
    }

    #[test]
    fn test_item_attr_sequential_from_first() {
        // Verify the sequential layout starting at ITEM_ATTR_FIRST (0x10)
        let expected: &[(ItemAttr, u8)] = &[
            (ItemAttr::ServerId, 0x10),
            (ItemAttr::ClientId, 0x11),
            (ItemAttr::Name, 0x12),
            (ItemAttr::Descr, 0x13),
            (ItemAttr::Speed, 0x14),
            (ItemAttr::Slot, 0x15),
            (ItemAttr::MaxItems, 0x16),
            (ItemAttr::Weight, 0x17),
            (ItemAttr::Weapon, 0x18),
            (ItemAttr::Amu, 0x19),
            (ItemAttr::Armor, 0x1A),
            (ItemAttr::MagLevel, 0x1B),
            (ItemAttr::MagFieldType, 0x1C),
            (ItemAttr::Writeable, 0x1D),
            (ItemAttr::RotateTo, 0x1E),
            (ItemAttr::Decay, 0x1F),
            (ItemAttr::SpriteHash, 0x20),
            (ItemAttr::MinimapColor, 0x21),
            (ItemAttr::Attr07, 0x22),
            (ItemAttr::Attr08, 0x23),
            (ItemAttr::Light, 0x24),
            (ItemAttr::Decay2, 0x25),
            (ItemAttr::Weapon2, 0x26),
            (ItemAttr::Amu2, 0x27),
            (ItemAttr::Armor2, 0x28),
            (ItemAttr::Writeable2, 0x29),
            (ItemAttr::Light2, 0x2A),
            (ItemAttr::TopOrder, 0x2B),
            (ItemAttr::Writeable3, 0x2C),
            (ItemAttr::WareId, 0x2D),
            (ItemAttr::Classification, 0x2E),
            (ItemAttr::Last, 0x2F),
        ];
        for (attr, expected_val) in expected {
            assert_eq!(*attr as u8, *expected_val, "Mismatch for {:?}", attr);
        }
    }

    // -----------------------------------------------------------------------
    // Item flags — each bit position
    // -----------------------------------------------------------------------

    #[test]
    fn test_flag_block_solid() {
        assert_eq!(FLAG_BLOCK_SOLID, 1 << 0);
    }

    #[test]
    fn test_flag_block_projectile() {
        assert_eq!(FLAG_BLOCK_PROJECTILE, 1 << 1);
    }

    #[test]
    fn test_flag_block_pathfind() {
        assert_eq!(FLAG_BLOCK_PATHFIND, 1 << 2);
    }

    #[test]
    fn test_flag_has_height() {
        assert_eq!(FLAG_HAS_HEIGHT, 1 << 3);
    }

    #[test]
    fn test_flag_useable() {
        assert_eq!(FLAG_USEABLE, 1 << 4);
    }

    #[test]
    fn test_flag_pickupable() {
        assert_eq!(FLAG_PICKUPABLE, 1 << 5);
    }

    #[test]
    fn test_flag_moveable() {
        assert_eq!(FLAG_MOVEABLE, 1 << 6);
    }

    #[test]
    fn test_flag_stackable() {
        assert_eq!(FLAG_STACKABLE, 1 << 7);
    }

    #[test]
    fn test_flag_floorchange_down() {
        assert_eq!(FLAG_FLOORCHANGEDOWN, 1 << 8);
    }

    #[test]
    fn test_flag_floorchange_north() {
        assert_eq!(FLAG_FLOORCHANGENORTH, 1 << 9);
    }

    #[test]
    fn test_flag_floorchange_east() {
        assert_eq!(FLAG_FLOORCHANGEEAST, 1 << 10);
    }

    #[test]
    fn test_flag_floorchange_south() {
        assert_eq!(FLAG_FLOORCHANGESOUTH, 1 << 11);
    }

    #[test]
    fn test_flag_floorchange_west() {
        assert_eq!(FLAG_FLOORCHANGEWEST, 1 << 12);
    }

    #[test]
    fn test_flag_alwaysontop() {
        assert_eq!(FLAG_ALWAYSONTOP, 1 << 13);
    }

    #[test]
    fn test_flag_readable() {
        assert_eq!(FLAG_READABLE, 1 << 14);
    }

    #[test]
    fn test_flag_rotatable() {
        assert_eq!(FLAG_ROTATABLE, 1 << 15);
    }

    #[test]
    fn test_flag_hangable() {
        assert_eq!(FLAG_HANGABLE, 1 << 16);
    }

    #[test]
    fn test_flag_vertical() {
        assert_eq!(FLAG_VERTICAL, 1 << 17);
    }

    #[test]
    fn test_flag_horizontal() {
        assert_eq!(FLAG_HORIZONTAL, 1 << 18);
    }

    #[test]
    fn test_flag_cannotdecay() {
        assert_eq!(FLAG_CANNOTDECAY, 1 << 19);
    }

    #[test]
    fn test_flag_allowdistread() {
        assert_eq!(FLAG_ALLOWDISTREAD, 1 << 20);
    }

    #[test]
    fn test_flag_clientduration() {
        assert_eq!(FLAG_CLIENTDURATION, 1 << 21);
    }

    #[test]
    fn test_flag_clientcharges() {
        assert_eq!(FLAG_CLIENTCHARGES, 1 << 22);
    }

    #[test]
    fn test_flag_lookthrough() {
        assert_eq!(FLAG_LOOKTHROUGH, 1 << 23);
    }

    #[test]
    fn test_flag_animation() {
        assert_eq!(FLAG_ANIMATION, 1 << 24);
    }

    #[test]
    fn test_flag_fulltile() {
        assert_eq!(FLAG_FULLTILE, 1 << 25);
    }

    #[test]
    fn test_flag_forceuse() {
        assert_eq!(FLAG_FORCEUSE, 1 << 26);
    }

    #[test]
    fn test_flag_ammo() {
        assert_eq!(FLAG_AMMO, 1 << 27);
    }

    #[test]
    fn test_flag_reportable() {
        assert_eq!(FLAG_REPORTABLE, 1 << 28);
    }

    #[test]
    fn test_flags_no_overlap() {
        // All 29 flags are distinct powers-of-two — none should collide
        let flags = [
            FLAG_BLOCK_SOLID,
            FLAG_BLOCK_PROJECTILE,
            FLAG_BLOCK_PATHFIND,
            FLAG_HAS_HEIGHT,
            FLAG_USEABLE,
            FLAG_PICKUPABLE,
            FLAG_MOVEABLE,
            FLAG_STACKABLE,
            FLAG_FLOORCHANGEDOWN,
            FLAG_FLOORCHANGENORTH,
            FLAG_FLOORCHANGEEAST,
            FLAG_FLOORCHANGESOUTH,
            FLAG_FLOORCHANGEWEST,
            FLAG_ALWAYSONTOP,
            FLAG_READABLE,
            FLAG_ROTATABLE,
            FLAG_HANGABLE,
            FLAG_VERTICAL,
            FLAG_HORIZONTAL,
            FLAG_CANNOTDECAY,
            FLAG_ALLOWDISTREAD,
            FLAG_CLIENTDURATION,
            FLAG_CLIENTCHARGES,
            FLAG_LOOKTHROUGH,
            FLAG_ANIMATION,
            FLAG_FULLTILE,
            FLAG_FORCEUSE,
            FLAG_AMMO,
            FLAG_REPORTABLE,
        ];
        let combined: u32 = flags.iter().fold(0u32, |acc, &f| {
            assert_eq!(acc & f, 0, "Flag {:#010x} overlaps with a previous flag", f);
            acc | f
        });
        assert_eq!(combined.count_ones() as usize, flags.len());
    }

    // -----------------------------------------------------------------------
    // VersionInfo struct
    // -----------------------------------------------------------------------

    #[test]
    fn test_version_info_default() {
        let v = VersionInfo::default();
        assert_eq!(v.major_version, 0);
        assert_eq!(v.minor_version, 0);
        assert_eq!(v.build_number, 0);
        assert_eq!(v.csd_version, [0u8; 128]);
    }

    #[test]
    fn test_version_info_fields() {
        let v = VersionInfo {
            major_version: 3,
            minor_version: 57,
            build_number: 9001,
            ..Default::default()
        };
        assert_eq!(v.major_version, 3);
        assert_eq!(v.minor_version, 57);
        assert_eq!(v.build_number, 9001);
    }

    #[test]
    fn test_version_info_csd_version_size() {
        let v = VersionInfo::default();
        assert_eq!(v.csd_version.len(), 128);
    }

    #[test]
    fn test_version_info_clone() {
        let a = VersionInfo {
            major_version: 1,
            ..Default::default()
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // -----------------------------------------------------------------------
    // LightBlock2 struct
    // -----------------------------------------------------------------------

    #[test]
    fn test_light_block2_default() {
        let lb = LightBlock2::default();
        assert_eq!(lb.light_level, 0);
        assert_eq!(lb.light_color, 0);
    }

    #[test]
    fn test_light_block2_fields() {
        let lb = LightBlock2 {
            light_level: 7,
            light_color: 215,
        };
        assert_eq!(lb.light_level, 7);
        assert_eq!(lb.light_color, 215);
    }

    #[test]
    fn test_light_block2_equality() {
        let a = LightBlock2 {
            light_level: 10,
            light_color: 200,
        };
        let b = LightBlock2 {
            light_level: 10,
            light_color: 200,
        };
        let c = LightBlock2 {
            light_level: 11,
            light_color: 200,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_light_block2_copy() {
        let a = LightBlock2 {
            light_level: 5,
            light_color: 100,
        };
        let b = a; // Copy
        assert_eq!(a, b);
    }
}
