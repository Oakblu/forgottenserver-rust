// Migrated from forgottenserver/src/items.h + items.cpp
//
// This module provides the `ItemTypeData` struct (the compile-time blueprint for
// every item) and the `Items` registry, which loads and indexes all item type
// definitions from OTB binary files.

use std::collections::HashMap;
use std::sync::OnceLock;

use forgottenserver_common::constants::{Ammo, FluidType, MagicEffectClass, ShootType, WeaponType};
use forgottenserver_common::enums::{
    CombatTypeFlags, ConditionTypeFlags, PlayerSex, RaceType, Reflect, Skill, SpecialSkill, Stat,
};
use forgottenserver_common::itemloader::{
    item_flags, ItemAttr, ItemGroup, RootAttr, CLIENT_VERSION_LAST,
};
use forgottenserver_common::position::Direction;
use forgottenserver_common::tools::{
    boolean_string, get_ammo_type, get_magic_effect, get_shoot_type,
};

// ---------------------------------------------------------------------------
// SlotPosition bit-flags (mirrors SlotPositionBits)
// ---------------------------------------------------------------------------

pub mod slot_position {
    pub const WHEREEVER: u32 = 0xFFFF_FFFF;
    pub const HEAD: u32 = 1 << 0;
    pub const NECKLACE: u32 = 1 << 1;
    pub const BACKPACK: u32 = 1 << 2;
    pub const ARMOR: u32 = 1 << 3;
    pub const RIGHT: u32 = 1 << 4;
    pub const LEFT: u32 = 1 << 5;
    pub const LEGS: u32 = 1 << 6;
    pub const FEET: u32 = 1 << 7;
    pub const RING: u32 = 1 << 8;
    pub const AMMO: u32 = 1 << 9;
    pub const DEPOT: u32 = 1 << 10;
    pub const TWO_HAND: u32 = 1 << 11;
    pub const HAND: u32 = LEFT | RIGHT;
}

// ---------------------------------------------------------------------------
// TileState bit-flags (subset used by item floorChange — mirrors
// `TILESTATE_FLOORCHANGE_*` from tile.h)
// ---------------------------------------------------------------------------

pub mod tile_state_floor_change {
    pub const DOWN: u8 = 1 << 0;
    pub const NORTH: u8 = 1 << 1;
    pub const SOUTH: u8 = 1 << 2;
    pub const SOUTH_ALT: u8 = 1 << 3;
    pub const EAST: u8 = 1 << 4;
    pub const EAST_ALT: u8 = 1 << 5;
    pub const WEST: u8 = 1 << 6;
}

// ---------------------------------------------------------------------------
// OTBI — the OTB file identifier as a 4-byte tag (mirrors C++ `OTBI` const)
// ---------------------------------------------------------------------------

/// Four-byte OTB file identifier ("OTBI") used to validate the leading bytes
/// of an OTB binary file.
pub const OTBI: [u8; 4] = [b'O', b'T', b'B', b'I'];

// ---------------------------------------------------------------------------
// Reflect array index helper — mirrors C++ `combatTypeToIndex(CombatType_t)`.
//
// Items in C++ use a flag bit pattern (1 << index) for CombatType_t, and the
// `combatTypeToIndex` helper maps the bit position back to an array index.
// ---------------------------------------------------------------------------

/// Returns the slot index inside `Abilities::*[CombatTypeFlags::COUNT]` arrays
/// for a given `CombatTypeFlags::*` bit-mask value.  Mirrors C++
/// `combatTypeToIndex`.
pub fn combat_type_to_index(combat: u16) -> usize {
    // C++ uses `__builtin_ctz` (count trailing zeros).
    combat.trailing_zeros() as usize
}

// ---------------------------------------------------------------------------
// Abilities — mirrors C++ `Abilities` struct (items.h)
// ---------------------------------------------------------------------------

const COMBAT_COUNT: usize = CombatTypeFlags::COUNT as usize;
const STAT_COUNT: usize = Stat::LAST as usize + 1;
const SKILL_COUNT: usize = Skill::LAST as usize + 1;
const SPECIAL_SKILL_COUNT: usize = SpecialSkill::LAST as usize + 1;

/// Per-item magical abilities (regen, stat bonuses, resistances, reflect,
/// element damage, etc.).  Mirrors C++ `Abilities` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abilities {
    pub health_gain: u32,
    pub health_ticks: u32,
    pub mana_gain: u32,
    pub mana_ticks: u32,

    pub condition_immunities: u32,
    pub condition_suppressions: u32,

    pub stats: [i32; STAT_COUNT],
    pub stats_percent: [i32; STAT_COUNT],

    pub skills: [i32; SKILL_COUNT],
    pub special_skills: [i32; SPECIAL_SKILL_COUNT],
    pub special_magic_level_skill: [i16; COMBAT_COUNT],
    pub speed: i32,

    pub field_absorb_percent: [i16; COMBAT_COUNT],
    pub absorb_percent: [i16; COMBAT_COUNT],
    pub reflect: [Reflect; COMBAT_COUNT],
    pub boost_percent: [i16; COMBAT_COUNT],

    pub element_damage: u16,
    pub element_type: u16, // CombatTypeFlags::* value

    pub mana_shield: bool,
    pub invisible: bool,
    pub regeneration: bool,
}

impl Default for Abilities {
    fn default() -> Self {
        Abilities {
            health_gain: 0,
            health_ticks: 0,
            mana_gain: 0,
            mana_ticks: 0,
            condition_immunities: 0,
            condition_suppressions: 0,
            stats: [0; STAT_COUNT],
            stats_percent: [0; STAT_COUNT],
            skills: [0; SKILL_COUNT],
            special_skills: [0; SPECIAL_SKILL_COUNT],
            special_magic_level_skill: [0; COMBAT_COUNT],
            speed: 0,
            field_absorb_percent: [0; COMBAT_COUNT],
            absorb_percent: [0; COMBAT_COUNT],
            reflect: [Reflect {
                percent: 0,
                chance: 0,
            }; COMBAT_COUNT],
            boost_percent: [0; COMBAT_COUNT],
            element_damage: 0,
            element_type: CombatTypeFlags::NONE,
            mana_shield: false,
            invisible: false,
            regeneration: false,
        }
    }
}

// ---------------------------------------------------------------------------
// ItemParseAttribute — mirrors C++ `ItemParseAttributes_t` enum (~140 variants)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemParseAttribute {
    Type,
    Description,
    RuneSpellName,
    Weight,
    ShowCount,
    Armor,
    Defense,
    ExtraDef,
    Attack,
    AttackSpeed,
    RotateTo,
    Moveable,
    BlockProjectile,
    Pickupable,
    ForceSerialize,
    FloorChange,
    CorpseType,
    ContainerSize,
    FluidSource,
    Readable,
    Writeable,
    MaxTextLen,
    WriteOnceItemId,
    WeaponType,
    SlotType,
    AmmoType,
    ShootType,
    Effect,
    Range,
    StopDuration,
    DecayTo,
    TransformEquipTo,
    TransformDeEquipTo,
    Duration,
    ShowDuration,
    Charges,
    ShowCharges,
    ShowAttributes,
    HitChance,
    MaxHitChance,
    Invisible,
    Speed,
    HealthGain,
    HealthTicks,
    ManaGain,
    ManaTicks,
    ManaShield,
    SkillSword,
    SkillAxe,
    SkillClub,
    SkillDist,
    SkillFish,
    SkillShield,
    SkillFist,
    MaxHitPoints,
    MaxHitPointsPercent,
    MaxManaPoints,
    MaxManaPointsPercent,
    MagicPoints,
    MagicPointsPercent,
    CriticalHitChance,
    CriticalHitAmount,
    LifeLeechChance,
    LifeLeechAmount,
    ManaLeechChance,
    ManaLeechAmount,
    FieldAbsorbPercentEnergy,
    FieldAbsorbPercentFire,
    FieldAbsorbPercentPoison,
    AbsorbPercentAll,
    AbsorbPercentElements,
    AbsorbPercentMagic,
    AbsorbPercentEnergy,
    AbsorbPercentFire,
    AbsorbPercentPoison,
    AbsorbPercentIce,
    AbsorbPercentHoly,
    AbsorbPercentDeath,
    AbsorbPercentLifeDrain,
    AbsorbPercentManaDrain,
    AbsorbPercentDrown,
    AbsorbPercentPhysical,
    AbsorbPercentHealing,
    AbsorbPercentUndefined,
    MagicLevelEnergy,
    MagicLevelFire,
    MagicLevelPoison,
    MagicLevelIce,
    MagicLevelHoly,
    MagicLevelDeath,
    MagicLevelLifeDrain,
    MagicLevelManaDrain,
    MagicLevelDrown,
    MagicLevelPhysical,
    MagicLevelHealing,
    MagicLevelUndefined,
    SuppressDrunk,
    SuppressEnergy,
    SuppressFire,
    SuppressPoison,
    SuppressDrown,
    SuppressPhysical,
    SuppressFreeze,
    SuppressDazzle,
    SuppressCurse,
    Field,
    Replaceable,
    PartnerDirection,
    LevelDoor,
    MaleTransformTo,
    FemaleTransformTo,
    TransformTo,
    DestroyTo,
    ElementIce,
    ElementEarth,
    ElementFire,
    ElementEnergy,
    ElementDeath,
    ElementHoly,
    WalkStack,
    Blocking,
    AllowDistRead,
    StoreItem,
    Worth,
    ReflectPercentAll,
    ReflectPercentElements,
    ReflectPercentMagic,
    ReflectPercentEnergy,
    ReflectPercentFire,
    ReflectPercentEarth,
    ReflectPercentIce,
    ReflectPercentHoly,
    ReflectPercentDeath,
    ReflectPercentLifeDrain,
    ReflectPercentManaDrain,
    ReflectPercentDrown,
    ReflectPercentPhysical,
    ReflectPercentHealing,
    ReflectChanceAll,
    ReflectChanceElements,
    ReflectChanceMagic,
    ReflectChanceEnergy,
    ReflectChanceFire,
    ReflectChanceEarth,
    ReflectChanceIce,
    ReflectChanceHoly,
    ReflectChanceDeath,
    ReflectChanceLifeDrain,
    ReflectChanceManaDrain,
    ReflectChanceDrown,
    ReflectChancePhysical,
    ReflectChanceHealing,
    BoostPercentAll,
    BoostPercentElements,
    BoostPercentMagic,
    BoostPercentEnergy,
    BoostPercentFire,
    BoostPercentEarth,
    BoostPercentIce,
    BoostPercentHoly,
    BoostPercentDeath,
    BoostPercentLifeDrain,
    BoostPercentManaDrain,
    BoostPercentDrown,
    BoostPercentPhysical,
    BoostPercentHealing,
    Supply,
}

// ---------------------------------------------------------------------------
// String → ItemParseAttribute / enum lookup maps (mirrors the 7 maps in
// the anonymous namespace of items.cpp)
// ---------------------------------------------------------------------------

fn build_item_parse_attributes_map() -> HashMap<&'static str, ItemParseAttribute> {
    use ItemParseAttribute::*;
    let pairs: &[(&str, ItemParseAttribute)] = &[
        ("type", Type),
        ("description", Description),
        ("runespellname", RuneSpellName),
        ("weight", Weight),
        ("showcount", ShowCount),
        ("armor", Armor),
        ("defense", Defense),
        ("extradef", ExtraDef),
        ("attack", Attack),
        ("attackspeed", AttackSpeed),
        ("rotateto", RotateTo),
        ("moveable", Moveable),
        ("movable", Moveable),
        ("blockprojectile", BlockProjectile),
        ("allowpickupable", Pickupable),
        ("pickupable", Pickupable),
        ("forceserialize", ForceSerialize),
        ("forcesave", ForceSerialize),
        ("floorchange", FloorChange),
        ("corpsetype", CorpseType),
        ("containersize", ContainerSize),
        ("fluidsource", FluidSource),
        ("readable", Readable),
        ("writeable", Writeable),
        ("maxtextlen", MaxTextLen),
        ("writeonceitemid", WriteOnceItemId),
        ("weapontype", WeaponType),
        ("slottype", SlotType),
        ("ammotype", AmmoType),
        ("shoottype", ShootType),
        ("effect", Effect),
        ("range", Range),
        ("stopduration", StopDuration),
        ("decayto", DecayTo),
        ("transformequipto", TransformEquipTo),
        ("transformdeequipto", TransformDeEquipTo),
        ("duration", Duration),
        ("showduration", ShowDuration),
        ("charges", Charges),
        ("showcharges", ShowCharges),
        ("showattributes", ShowAttributes),
        ("hitchance", HitChance),
        ("maxhitchance", MaxHitChance),
        ("invisible", Invisible),
        ("speed", Speed),
        ("healthgain", HealthGain),
        ("healthticks", HealthTicks),
        ("managain", ManaGain),
        ("manaticks", ManaTicks),
        ("manashield", ManaShield),
        ("skillsword", SkillSword),
        ("skillaxe", SkillAxe),
        ("skillclub", SkillClub),
        ("skilldist", SkillDist),
        ("skillfish", SkillFish),
        ("skillshield", SkillShield),
        ("skillfist", SkillFist),
        ("maxhitpoints", MaxHitPoints),
        ("maxhitpointspercent", MaxHitPointsPercent),
        ("maxmanapoints", MaxManaPoints),
        ("maxmanapointspercent", MaxManaPointsPercent),
        ("magicpoints", MagicPoints),
        ("magiclevelpoints", MagicPoints),
        ("magicpointspercent", MagicPointsPercent),
        ("criticalhitchance", CriticalHitChance),
        ("criticalhitamount", CriticalHitAmount),
        ("lifeleechchance", LifeLeechChance),
        ("lifeleechamount", LifeLeechAmount),
        ("manaleechchance", ManaLeechChance),
        ("manaleechamount", ManaLeechAmount),
        ("fieldabsorbpercentenergy", FieldAbsorbPercentEnergy),
        ("fieldabsorbpercentfire", FieldAbsorbPercentFire),
        ("fieldabsorbpercentpoison", FieldAbsorbPercentPoison),
        ("fieldabsorbpercentearth", FieldAbsorbPercentPoison),
        ("absorbpercentall", AbsorbPercentAll),
        ("absorbpercentallelements", AbsorbPercentAll),
        ("absorbpercentelements", AbsorbPercentElements),
        ("absorbpercentmagic", AbsorbPercentMagic),
        ("absorbpercentenergy", AbsorbPercentEnergy),
        ("absorbpercentfire", AbsorbPercentFire),
        ("absorbpercentpoison", AbsorbPercentPoison),
        ("absorbpercentearth", AbsorbPercentPoison),
        ("absorbpercentice", AbsorbPercentIce),
        ("absorbpercentholy", AbsorbPercentHoly),
        ("absorbpercentdeath", AbsorbPercentDeath),
        ("absorbpercentlifedrain", AbsorbPercentLifeDrain),
        ("absorbpercentmanadrain", AbsorbPercentManaDrain),
        ("absorbpercentdrown", AbsorbPercentDrown),
        ("absorbpercentphysical", AbsorbPercentPhysical),
        ("absorbpercenthealing", AbsorbPercentHealing),
        ("absorbpercentundefined", AbsorbPercentUndefined),
        ("reflectpercentall", ReflectPercentAll),
        ("reflectpercentallelements", ReflectPercentAll),
        ("reflectpercentelements", ReflectPercentElements),
        ("reflectpercentmagic", ReflectPercentMagic),
        ("reflectpercentenergy", ReflectPercentEnergy),
        ("reflectpercentfire", ReflectPercentFire),
        ("reflectpercentpoison", ReflectPercentEarth),
        ("reflectpercentearth", ReflectPercentEarth),
        ("reflectpercentice", ReflectPercentIce),
        ("reflectpercentholy", ReflectPercentHoly),
        ("reflectpercentdeath", ReflectPercentDeath),
        ("reflectpercentlifedrain", ReflectPercentLifeDrain),
        ("reflectpercentmanadrain", ReflectPercentManaDrain),
        ("reflectpercentdrown", ReflectPercentDrown),
        ("reflectpercentphysical", ReflectPercentPhysical),
        ("reflectpercenthealing", ReflectPercentHealing),
        ("reflectchanceall", ReflectChanceAll),
        ("reflectchanceallelements", ReflectChanceAll),
        ("reflectchanceelements", ReflectChanceElements),
        ("reflectchancemagic", ReflectChanceMagic),
        ("reflectchanceenergy", ReflectChanceEnergy),
        ("reflectchancefire", ReflectChanceFire),
        ("reflectchancepoison", ReflectChanceEarth),
        ("reflectchanceearth", ReflectChanceEarth),
        ("reflectchanceice", ReflectChanceIce),
        ("reflectchanceholy", ReflectChanceHoly),
        ("reflectchancedeath", ReflectChanceDeath),
        ("reflectchancelifedrain", ReflectChanceLifeDrain),
        ("reflectchancemanadrain", ReflectChanceManaDrain),
        ("reflectchancedrown", ReflectChanceDrown),
        ("reflectchancephysical", ReflectChancePhysical),
        ("reflectchancehealing", ReflectChanceHealing),
        ("boostpercentall", BoostPercentAll),
        ("boostpercentallelements", BoostPercentAll),
        ("boostpercentelements", BoostPercentElements),
        ("boostpercentmagic", BoostPercentMagic),
        ("boostpercentenergy", BoostPercentEnergy),
        ("boostpercentfire", BoostPercentFire),
        ("boostpercentpoison", BoostPercentEarth),
        ("boostpercentearth", BoostPercentEarth),
        ("boostpercentice", BoostPercentIce),
        ("boostpercentholy", BoostPercentHoly),
        ("boostpercentdeath", BoostPercentDeath),
        ("boostpercentlifedrain", BoostPercentLifeDrain),
        ("boostpercentmanadrain", BoostPercentManaDrain),
        ("boostpercentdrown", BoostPercentDrown),
        ("boostpercentphysical", BoostPercentPhysical),
        ("boostpercenthealing", BoostPercentHealing),
        ("magiclevelenergy", MagicLevelEnergy),
        ("magiclevelfire", MagicLevelFire),
        ("magiclevelpoison", MagicLevelPoison),
        ("magiclevelearth", MagicLevelPoison),
        ("magiclevelice", MagicLevelIce),
        ("magiclevelholy", MagicLevelHoly),
        ("magicleveldeath", MagicLevelDeath),
        ("magiclevellifedrain", MagicLevelLifeDrain),
        ("magiclevelmanadrain", MagicLevelManaDrain),
        ("magicleveldrown", MagicLevelDrown),
        ("magiclevelphysical", MagicLevelPhysical),
        ("magiclevelhealing", MagicLevelHealing),
        ("magiclevelundefined", MagicLevelUndefined),
        ("suppressdrunk", SuppressDrunk),
        ("suppressenergy", SuppressEnergy),
        ("suppressfire", SuppressFire),
        ("suppresspoison", SuppressPoison),
        ("suppressdrown", SuppressDrown),
        ("suppressphysical", SuppressPhysical),
        ("suppressfreeze", SuppressFreeze),
        ("suppressdazzle", SuppressDazzle),
        ("suppresscurse", SuppressCurse),
        ("field", Field),
        ("replaceable", Replaceable),
        ("partnerdirection", PartnerDirection),
        ("leveldoor", LevelDoor),
        ("maletransformto", MaleTransformTo),
        ("malesleeper", MaleTransformTo),
        ("femaletransformto", FemaleTransformTo),
        ("femalesleeper", FemaleTransformTo),
        ("transformto", TransformTo),
        ("destroyto", DestroyTo),
        ("elementice", ElementIce),
        ("elementearth", ElementEarth),
        ("elementfire", ElementFire),
        ("elementenergy", ElementEnergy),
        ("elementdeath", ElementDeath),
        ("elementholy", ElementHoly),
        ("walkstack", WalkStack),
        ("blocking", Blocking),
        ("allowdistread", AllowDistRead),
        ("storeitem", StoreItem),
        ("worth", Worth),
        ("supply", Supply),
    ];
    pairs.iter().copied().collect()
}

fn item_parse_attributes_map() -> &'static HashMap<&'static str, ItemParseAttribute> {
    static MAP: OnceLock<HashMap<&'static str, ItemParseAttribute>> = OnceLock::new();
    MAP.get_or_init(build_item_parse_attributes_map)
}

fn item_types_map() -> &'static HashMap<&'static str, ItemTypeKind> {
    static MAP: OnceLock<HashMap<&'static str, ItemTypeKind>> = OnceLock::new();
    MAP.get_or_init(|| {
        let pairs: &[(&str, ItemTypeKind)] = &[
            ("key", ItemTypeKind::Key),
            ("magicfield", ItemTypeKind::MagicField),
            ("container", ItemTypeKind::Container),
            ("depot", ItemTypeKind::Depot),
            ("mailbox", ItemTypeKind::Mailbox),
            ("trashholder", ItemTypeKind::TrashHolder),
            ("teleport", ItemTypeKind::Teleport),
            ("door", ItemTypeKind::Door),
            ("bed", ItemTypeKind::Bed),
            ("rune", ItemTypeKind::Rune),
            ("podium", ItemTypeKind::Podium),
        ];
        pairs.iter().copied().collect()
    })
}

fn tile_states_map() -> &'static HashMap<&'static str, u8> {
    static MAP: OnceLock<HashMap<&'static str, u8>> = OnceLock::new();
    MAP.get_or_init(|| {
        use tile_state_floor_change::*;
        let pairs: &[(&str, u8)] = &[
            ("down", DOWN),
            ("north", NORTH),
            ("south", SOUTH),
            ("southalt", SOUTH_ALT),
            ("west", WEST),
            ("east", EAST),
            ("eastalt", EAST_ALT),
        ];
        pairs.iter().copied().collect()
    })
}

fn race_types_map() -> &'static HashMap<&'static str, RaceType> {
    static MAP: OnceLock<HashMap<&'static str, RaceType>> = OnceLock::new();
    MAP.get_or_init(|| {
        let pairs: &[(&str, RaceType)] = &[
            ("venom", RaceType::Venom),
            ("blood", RaceType::Blood),
            ("undead", RaceType::Undead),
            ("fire", RaceType::Fire),
            ("energy", RaceType::Energy),
            ("ink", RaceType::Ink),
        ];
        pairs.iter().copied().collect()
    })
}

fn weapon_types_map() -> &'static HashMap<&'static str, WeaponType> {
    static MAP: OnceLock<HashMap<&'static str, WeaponType>> = OnceLock::new();
    MAP.get_or_init(|| {
        let pairs: &[(&str, WeaponType)] = &[
            ("sword", WeaponType::Sword),
            ("club", WeaponType::Club),
            ("axe", WeaponType::Axe),
            ("shield", WeaponType::Shield),
            ("distance", WeaponType::Distance),
            ("wand", WeaponType::Wand),
            ("ammunition", WeaponType::Ammo),
            ("quiver", WeaponType::Quiver),
        ];
        pairs.iter().copied().collect()
    })
}

fn fluid_types_map() -> &'static HashMap<&'static str, FluidType> {
    static MAP: OnceLock<HashMap<&'static str, FluidType>> = OnceLock::new();
    MAP.get_or_init(|| {
        let pairs: &[(&str, FluidType)] = &[
            ("water", FluidType::Water),
            ("blood", FluidType::Blood),
            ("beer", FluidType::Beer),
            ("slime", FluidType::Slime),
            ("lemonade", FluidType::Lemonade),
            ("milk", FluidType::Milk),
            ("mana", FluidType::Mana),
            ("life", FluidType::Life),
            ("oil", FluidType::Oil),
            ("urine", FluidType::Urine),
            ("coconut", FluidType::CoconutMilk),
            ("wine", FluidType::Wine),
            ("mud", FluidType::Mud),
            ("fruitjuice", FluidType::FruitJuice),
            ("lava", FluidType::Lava),
            ("rum", FluidType::Rum),
            ("swamp", FluidType::Swamp),
            ("tea", FluidType::Tea),
            ("mead", FluidType::Mead),
            ("ink", FluidType::Ink),
        ];
        pairs.iter().copied().collect()
    })
}

fn directions_map() -> &'static HashMap<&'static str, Direction> {
    static MAP: OnceLock<HashMap<&'static str, Direction>> = OnceLock::new();
    MAP.get_or_init(|| {
        let pairs: &[(&str, Direction)] = &[
            ("north", Direction::North),
            ("n", Direction::North),
            ("0", Direction::North),
            ("east", Direction::East),
            ("e", Direction::East),
            ("1", Direction::East),
            ("south", Direction::South),
            ("s", Direction::South),
            ("2", Direction::South),
            ("west", Direction::West),
            ("w", Direction::West),
            ("3", Direction::West),
            ("southwest", Direction::Southwest),
            ("south west", Direction::Southwest),
            ("south-west", Direction::Southwest),
            ("sw", Direction::Southwest),
            ("4", Direction::Southwest),
            ("southeast", Direction::Southeast),
            ("south east", Direction::Southeast),
            ("south-east", Direction::Southeast),
            ("se", Direction::Southeast),
            ("5", Direction::Southeast),
            ("northwest", Direction::Northwest),
            ("north west", Direction::Northwest),
            ("north-west", Direction::Northwest),
            ("nw", Direction::Northwest),
            ("6", Direction::Northwest),
            ("northeast", Direction::Northeast),
            ("north east", Direction::Northeast),
            ("north-east", Direction::Northeast),
            ("ne", Direction::Northeast),
            ("7", Direction::Northeast),
        ];
        pairs.iter().copied().collect()
    })
}

/// Look up a `Direction` from a string. Defaults to `Direction::North` for
/// unknown input (mirrors C++ `getDirection`).
pub fn get_direction(s: &str) -> Direction {
    directions_map().get(s).copied().unwrap_or(Direction::North)
}

// ---------------------------------------------------------------------------
// ItemType_enum (mirrors ItemTypes_t)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemTypeKind {
    #[default]
    None,
    Depot,
    Mailbox,
    TrashHolder,
    Container,
    Door,
    MagicField,
    Teleport,
    Bed,
    Key,
    Rune,
    Podium,
}

// ---------------------------------------------------------------------------
// ItemTypeData — the blueprint for an item type (mirrors C++ `ItemType`)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ItemTypeData {
    // OTB-sourced fields
    pub id: u16,
    pub client_id: u16,
    pub group: ItemGroup,
    pub type_kind: ItemTypeKind,
    pub speed: u16,
    pub ware_id: u16,
    pub light_level: u8,
    pub light_color: u8,
    pub always_on_top_order: u8,
    pub classification: u8,

    // Boolean flags from OTB flags word
    pub block_solid: bool,
    pub block_projectile: bool,
    pub block_path_find: bool,
    pub has_height: bool,
    pub useable: bool,
    pub pickupable: bool,
    pub moveable: bool,
    pub stackable: bool,
    pub always_on_top: bool,
    pub is_vertical: bool,
    pub is_horizontal: bool,
    pub is_hangable: bool,
    pub allow_dist_read: bool,
    pub rotatable: bool,
    pub can_read_text: bool,
    pub can_write_text: bool,
    pub look_through: bool,
    pub is_animation: bool,
    pub force_use: bool,
    pub show_client_charges: bool,
    pub show_client_duration: bool,

    // XML-sourced fields
    pub name: String,
    pub article: String,
    pub plural_name: String,
    pub description: String,
    pub rune_spell_name: String,
    pub vocation_string: String,
    pub weight: u32,
    pub attack: i32,
    pub defense: i32,
    pub extra_defense: i32,
    pub armor: i32,
    pub attack_speed: u32,
    pub rotate_to: u16,
    pub slot_position: u32,
    pub show_count: bool,
    pub show_duration: bool,
    pub show_charges: bool,
    pub show_attributes: bool,
    pub allow_pickupable: bool,
    pub replaceable: bool,
    pub walk_stack: bool,
    pub force_serialize: bool,
    pub stop_time: bool,
    pub supply: bool,
    pub decay_time_min: u32,
    pub decay_time_max: u32,
    pub decay_to: i32,
    pub charges: u32,
    pub max_items: u16,
    pub max_text_len: u16,
    pub write_once_item_id: u16,
    pub transform_equip_to: u16,
    pub transform_de_equip_to: u16,
    pub transform_to_free: u16,
    pub destroy_to: u16,
    pub hit_chance: i8,
    pub max_hit_chance: i32,
    pub floor_change: u8,
    pub shoot_range: u8,
    pub level_door: u32,
    pub min_req_level: u32,
    pub min_req_magic_level: u32,
    pub wield_info: u32,
    pub rune_level: i32,
    pub rune_magic_level: i32,
    pub worth: u64,
    pub store_item: bool,
    pub block_pickupable: bool,
    pub weapon_type: WeaponType,

    // XML-sourced additional fields (mirrors the rest of C++ ItemType)
    pub abilities: Option<Box<Abilities>>,
    pub combat_type: u16, // CombatTypeFlags::* (NONE by default)
    pub bed_partner_dir: Direction,
    pub transform_to_on_use: [u16; 2], // [PLAYERSEX_FEMALE, PLAYERSEX_MALE]
    pub magic_effect: MagicEffectClass,
    pub ammo_type: Ammo,
    pub shoot_type: ShootType,
    pub corpse_type: RaceType,
    pub fluid_source: FluidType,
    /// Mirror of C++ `ItemType::conditionDamage` (`std::unique_ptr<ConditionDamage>`).
    /// Set by `<attribute key="field">` XML sub-children — see
    /// `apply_attribute_field`.
    pub condition_damage: Option<Box<crate::condition::ConditionDamage>>,
}

impl Default for ItemTypeData {
    fn default() -> Self {
        ItemTypeData {
            id: 0,
            client_id: 0,
            group: ItemGroup::None,
            type_kind: ItemTypeKind::None,
            speed: 0,
            ware_id: 0,
            light_level: 0,
            light_color: 0,
            always_on_top_order: 0,
            classification: 0,
            block_solid: false,
            block_projectile: false,
            block_path_find: false,
            has_height: false,
            useable: false,
            pickupable: false,
            moveable: false,
            stackable: false,
            always_on_top: false,
            is_vertical: false,
            is_horizontal: false,
            is_hangable: false,
            allow_dist_read: false,
            rotatable: false,
            can_read_text: false,
            can_write_text: false,
            look_through: false,
            is_animation: false,
            force_use: false,
            show_client_charges: false,
            show_client_duration: false,
            name: String::new(),
            article: String::new(),
            plural_name: String::new(),
            description: String::new(),
            rune_spell_name: String::new(),
            vocation_string: String::new(),
            weight: 0,
            attack: 0,
            defense: 0,
            extra_defense: 0,
            armor: 0,
            attack_speed: 0,
            rotate_to: 0,
            slot_position: slot_position::HAND,
            show_count: true,
            show_duration: false,
            show_charges: false,
            show_attributes: false,
            allow_pickupable: false,
            replaceable: true,
            walk_stack: true,
            force_serialize: false,
            stop_time: false,
            supply: false,
            decay_time_min: 0,
            decay_time_max: 0,
            decay_to: -1,
            charges: 0,
            max_items: 8,
            max_text_len: 0,
            write_once_item_id: 0,
            transform_equip_to: 0,
            transform_de_equip_to: 0,
            transform_to_free: 0,
            destroy_to: 0,
            hit_chance: 0,
            max_hit_chance: -1,
            floor_change: 0,
            shoot_range: 1,
            level_door: 0,
            min_req_level: 0,
            min_req_magic_level: 0,
            wield_info: 0,
            rune_level: 0,
            rune_magic_level: 0,
            worth: 0,
            store_item: false,
            block_pickupable: false,
            weapon_type: WeaponType::None,
            abilities: None,
            combat_type: CombatTypeFlags::NONE,
            bed_partner_dir: Direction::None,
            transform_to_on_use: [0u16; 2],
            magic_effect: MagicEffectClass::None,
            ammo_type: Ammo::None,
            shoot_type: ShootType::None,
            corpse_type: RaceType::None,
            fluid_source: FluidType::None,
            condition_damage: None,
        }
    }
}

impl ItemTypeData {
    /// Is this item a ground tile?
    pub fn is_ground_tile(&self) -> bool {
        self.group == ItemGroup::Ground
    }

    /// Is this item a container?
    pub fn is_container(&self) -> bool {
        self.group == ItemGroup::Container
    }

    /// Is this item a splash?
    pub fn is_splash(&self) -> bool {
        self.group == ItemGroup::Splash
    }

    /// Is this a fluid container?
    pub fn is_fluid_container(&self) -> bool {
        self.group == ItemGroup::Fluid
    }

    /// Is this a door?
    pub fn is_door(&self) -> bool {
        self.type_kind == ItemTypeKind::Door
    }

    /// Is this a magic field?
    pub fn is_magic_field(&self) -> bool {
        self.type_kind == ItemTypeKind::MagicField
    }

    /// Is this a teleport?
    pub fn is_teleport(&self) -> bool {
        self.type_kind == ItemTypeKind::Teleport
    }

    /// Is this a key?
    pub fn is_key(&self) -> bool {
        self.type_kind == ItemTypeKind::Key
    }

    /// Is this a depot?
    pub fn is_depot(&self) -> bool {
        self.type_kind == ItemTypeKind::Depot
    }

    /// Is this a mailbox?
    pub fn is_mailbox(&self) -> bool {
        self.type_kind == ItemTypeKind::Mailbox
    }

    /// Is this a trash holder?
    pub fn is_trash_holder(&self) -> bool {
        self.type_kind == ItemTypeKind::TrashHolder
    }

    /// Is this a bed?
    pub fn is_bed(&self) -> bool {
        self.type_kind == ItemTypeKind::Bed
    }

    /// Is this a rune?
    pub fn is_rune(&self) -> bool {
        self.type_kind == ItemTypeKind::Rune
    }

    /// Is this a podium?
    pub fn is_podium(&self) -> bool {
        self.type_kind == ItemTypeKind::Podium
    }

    /// Is this item useable?
    pub fn is_useable(&self) -> bool {
        self.useable
    }

    /// Is this a supply item?
    pub fn is_supply(&self) -> bool {
        self.supply
    }

    /// Does this item have a sub-type (fluid containers, splashes, stackables, charged items)?
    /// Mirrors C++: `isFluidContainer() || isSplash() || stackable || charges != 0`
    pub fn has_sub_type(&self) -> bool {
        self.is_fluid_container() || self.is_splash() || self.stackable || self.charges != 0
    }

    /// Is actually pickupable (either flag)?
    pub fn is_pickupable(&self) -> bool {
        self.allow_pickupable || self.pickupable
    }

    /// Get plural name, deriving it from `name` when not explicitly set.
    pub fn get_plural_name(&self) -> String {
        if !self.plural_name.is_empty() {
            return self.plural_name.clone();
        }
        if !self.show_count {
            return self.name.clone();
        }
        if self.name.is_empty() || self.name.ends_with('s') {
            return self.name.clone();
        }
        format!("{}s", self.name)
    }
}

// ---------------------------------------------------------------------------
// PropReader — a small cursor for reading typed little-endian values from a
// byte slice, mirroring C++ PropStream.
// ---------------------------------------------------------------------------

struct PropReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PropReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        PropReader { data, pos: 0 }
    }

    fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let v = self.data[self.pos];
            self.pos += 1;
            Some(v)
        } else {
            None
        }
    }

    fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 <= self.data.len() {
            let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
            self.pos += 2;
            Some(v)
        } else {
            None
        }
    }

    fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 <= self.data.len() {
            let v = u32::from_le_bytes([
                self.data[self.pos],
                self.data[self.pos + 1],
                self.data[self.pos + 2],
                self.data[self.pos + 3],
            ]);
            self.pos += 4;
            Some(v)
        } else {
            None
        }
    }

    fn skip(&mut self, n: usize) -> bool {
        if self.pos + n <= self.data.len() {
            self.pos += n;
            true
        } else {
            false
        }
    }

    fn remaining(&self) -> bool {
        self.pos < self.data.len()
    }
}

// ---------------------------------------------------------------------------
// Items — the registry (mirrors C++ `Items`)
// ---------------------------------------------------------------------------

/// The item-type registry.  Populated by [`Items::load_from_otb`].
#[derive(Debug, Default)]
pub struct Items {
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,

    /// Items indexed by server ID.  Index 0 is unused (sentinel).
    items: Vec<ItemTypeData>,
    /// Map client_id → server_id (first mapping wins).
    client_to_server: Vec<u16>,
    /// Map lowercase name → server_id.
    name_to_id: HashMap<String, u16>,
    /// Currency items: `(worth, item_id)` pairs, sorted by worth descending
    /// (mirrors C++ `std::map<uint64_t, uint16_t, std::greater<>>`).
    pub currency_items: Vec<(u64, u16)>,
}

impl Items {
    /// Create an empty registry.
    pub fn new() -> Self {
        Items::default()
    }

    /// Load item types from an OTB binary file.
    ///
    /// Returns `Err` with a human-readable message if the file cannot be
    /// parsed or if the version header is incompatible.
    pub fn load_from_otb(data: &[u8]) -> Result<Items, String> {
        Self::parse_otb(data)
    }

    fn parse_otb(data: &[u8]) -> Result<Items, String> {
        // OTB binary layout:
        //   4-byte file identifier (ignored)
        //   NODE_START (0xFE)
        //   root type byte
        //   root props (escaped)
        //   child nodes (escaped)
        //   NODE_END (0xFF)

        use forgottenserver_common::fileloader::NODE_START;

        if data.len() < 7 {
            return Err("OTB file too small".into());
        }

        // Skip 4-byte identifier
        let mut pos = 4usize;

        if data[pos] != NODE_START {
            return Err("OTB: expected NODE_START".into());
        }
        pos += 1;

        // Parse root node props
        let (_root_type, root_props, root_children_raw) = parse_node_raw(data, &mut pos)?;

        // Parse root version info from props
        let mut registry = Items::default();
        {
            let mut rp = PropReader::new(&root_props);
            // 4-byte flags
            let _flags = rp.read_u32().ok_or("OTB root: missing flags")?;
            // attr byte
            let attr = rp.read_u8().ok_or("OTB root: missing attr")?;
            if attr == RootAttr::Version as u8 {
                let datalen = rp.read_u16().ok_or("OTB root: missing datalen")?;
                // VersionInfo is 4+4+4+128 = 140 bytes
                const VERSIONINFO_SIZE: usize = 140;
                if datalen as usize != VERSIONINFO_SIZE {
                    return Err(format!(
                        "OTB root: VersionInfo size mismatch: expected {}, got {}",
                        VERSIONINFO_SIZE, datalen
                    ));
                }
                let major = rp.read_u32().ok_or("OTB: missing major version")?;
                let minor = rp.read_u32().ok_or("OTB: missing minor version")?;
                let build = rp.read_u32().ok_or("OTB: missing build number")?;
                // skip 128-byte CSD string
                if !rp.skip(128) {
                    return Err("OTB: truncated CSD string".into());
                }
                registry.major_version = major;
                registry.minor_version = minor;
                registry.build_number = build;
            }
        }

        // Validate version
        if registry.major_version == 0xFFFF_FFFF {
            // generic version — allowed
        } else if registry.major_version != 3 {
            return Err(format!(
                "OTB: incompatible major version {} (expected 3)",
                registry.major_version
            ));
        } else if registry.minor_version < CLIENT_VERSION_LAST {
            return Err(format!(
                "OTB: client version {} too old (need >= {})",
                registry.minor_version, CLIENT_VERSION_LAST
            ));
        }

        // Process item child nodes
        for child_props_bytes in &root_children_raw {
            let child_type_byte = child_props_bytes.0;
            let child_props = &child_props_bytes.1;

            let group = match child_type_byte {
                0 => ItemGroup::None,
                1 => ItemGroup::Ground,
                2 => ItemGroup::Container,
                3 => ItemGroup::Weapon,
                4 => ItemGroup::Ammunition,
                5 => ItemGroup::Armor,
                6 => ItemGroup::Charges,
                7 => ItemGroup::Teleport,
                8 => ItemGroup::MagicField,
                9 => ItemGroup::Writeable,
                10 => ItemGroup::Key,
                11 => ItemGroup::Splash,
                12 => ItemGroup::Fluid,
                13 => ItemGroup::Door,
                14 => ItemGroup::Deprecated,
                15 => ItemGroup::Podium,
                _ => return Err(format!("OTB: unknown item group {}", child_type_byte)),
            };

            let type_kind = match group {
                ItemGroup::Container => ItemTypeKind::Container,
                ItemGroup::Door => ItemTypeKind::Door,
                ItemGroup::MagicField => ItemTypeKind::MagicField,
                ItemGroup::Teleport => ItemTypeKind::Teleport,
                ItemGroup::Podium => ItemTypeKind::Podium,
                _ => ItemTypeKind::None,
            };

            let mut cp = PropReader::new(child_props);
            let flags = cp.read_u32().ok_or("OTB item: missing flags")?;

            let mut server_id: u16 = 0;
            let mut client_id: u16 = 0;
            let mut speed: u16 = 0;
            let mut ware_id: u16 = 0;
            let mut light_level: u8 = 0;
            let mut light_color: u8 = 0;
            let mut always_on_top_order: u8 = 0;
            let mut classification: u8 = 0;

            while cp.remaining() {
                // `remaining()` guarantees at least one byte is left, so
                // `read_u8` is infallible here.
                let attrib = cp
                    .read_u8()
                    .expect("PropReader::remaining guarantees a byte is available");
                let datalen = cp.read_u16().ok_or("OTB item: missing attr datalen")?;

                match attrib {
                    a if a == ItemAttr::ServerId as u8 => {
                        if datalen != 2 {
                            return Err("OTB item: ServerId wrong length".into());
                        }
                        server_id = cp.read_u16().ok_or("OTB item: missing server_id")?;
                    }
                    a if a == ItemAttr::ClientId as u8 => {
                        if datalen != 2 {
                            return Err("OTB item: ClientId wrong length".into());
                        }
                        client_id = cp.read_u16().ok_or("OTB item: missing client_id")?;
                    }
                    a if a == ItemAttr::Speed as u8 => {
                        if datalen != 2 {
                            return Err("OTB item: Speed wrong length".into());
                        }
                        speed = cp.read_u16().ok_or("OTB item: missing speed")?;
                    }
                    a if a == ItemAttr::Light2 as u8 => {
                        if datalen != 4 {
                            return Err("OTB item: Light2 wrong length".into());
                        }
                        light_level = cp.read_u16().ok_or("OTB item: missing light_level")? as u8;
                        light_color = cp.read_u16().ok_or("OTB item: missing light_color")? as u8;
                    }
                    a if a == ItemAttr::TopOrder as u8 => {
                        if datalen != 1 {
                            return Err("OTB item: TopOrder wrong length".into());
                        }
                        always_on_top_order = cp.read_u8().ok_or("OTB item: missing top_order")?;
                    }
                    a if a == ItemAttr::WareId as u8 => {
                        if datalen != 2 {
                            return Err("OTB item: WareId wrong length".into());
                        }
                        ware_id = cp.read_u16().ok_or("OTB item: missing ware_id")?;
                    }
                    a if a == ItemAttr::Classification as u8 => {
                        if datalen != 1 {
                            return Err("OTB item: Classification wrong length".into());
                        }
                        classification = cp.read_u8().ok_or("OTB item: missing classification")?;
                    }
                    _ => {
                        // Unknown attribute — skip
                        if !cp.skip(datalen as usize) {
                            return Err("OTB item: truncated unknown attr".into());
                        }
                    }
                }
            }

            // Map client_id → server_id (first mapping wins)
            if client_id as usize >= registry.client_to_server.len() {
                registry
                    .client_to_server
                    .resize(client_id as usize + 1, 0u16);
            }
            if registry.client_to_server[client_id as usize] == 0 {
                registry.client_to_server[client_id as usize] = server_id;
            }

            // Grow items vec if needed
            if server_id as usize >= registry.items.len() {
                registry
                    .items
                    .resize_with(server_id as usize + 1, ItemTypeData::default);
            }

            let it = &mut registry.items[server_id as usize];
            it.id = server_id;
            it.client_id = client_id;
            it.group = group;
            it.type_kind = type_kind;
            it.speed = speed;
            it.ware_id = ware_id;
            it.light_level = light_level;
            it.light_color = light_color;
            it.always_on_top_order = always_on_top_order;
            it.classification = classification;

            // Boolean flags
            it.block_solid = flags & item_flags::FLAG_BLOCK_SOLID != 0;
            it.block_projectile = flags & item_flags::FLAG_BLOCK_PROJECTILE != 0;
            it.block_path_find = flags & item_flags::FLAG_BLOCK_PATHFIND != 0;
            it.has_height = flags & item_flags::FLAG_HAS_HEIGHT != 0;
            it.useable = flags & item_flags::FLAG_USEABLE != 0;
            it.pickupable = flags & item_flags::FLAG_PICKUPABLE != 0;
            it.moveable = flags & item_flags::FLAG_MOVEABLE != 0;
            it.stackable = flags & item_flags::FLAG_STACKABLE != 0;
            it.always_on_top = flags & item_flags::FLAG_ALWAYSONTOP != 0;
            it.is_vertical = flags & item_flags::FLAG_VERTICAL != 0;
            it.is_horizontal = flags & item_flags::FLAG_HORIZONTAL != 0;
            it.is_hangable = flags & item_flags::FLAG_HANGABLE != 0;
            it.allow_dist_read = flags & item_flags::FLAG_ALLOWDISTREAD != 0;
            it.rotatable = flags & item_flags::FLAG_ROTATABLE != 0;
            it.can_read_text = flags & item_flags::FLAG_READABLE != 0;
            it.look_through = flags & item_flags::FLAG_LOOKTHROUGH != 0;
            it.is_animation = flags & item_flags::FLAG_ANIMATION != 0;
            it.force_use = flags & item_flags::FLAG_FORCEUSE != 0;
            it.show_client_charges = flags & item_flags::FLAG_CLIENTCHARGES != 0;
            it.show_client_duration = flags & item_flags::FLAG_CLIENTDURATION != 0;
        }

        Ok(registry)
    }

    /// Look up an item by server ID.
    pub fn get_item_type(&self, server_id: u16) -> Option<&ItemTypeData> {
        self.items.get(server_id as usize).filter(|it| it.id != 0)
    }

    /// Look up an item by client ID.
    ///
    /// Mirrors C++ `getItemIdByClientId`: client IDs < 100 are sprite-based and
    /// are never stored in the map, so they always return `None`.
    pub fn get_item_type_by_client_id(&self, client_id: u16) -> Option<&ItemTypeData> {
        if client_id < 100 {
            return None;
        }
        let server_id = self.client_to_server.get(client_id as usize).copied()?;
        if server_id == 0 {
            return None;
        }
        self.get_item_type(server_id)
    }

    /// Returns `true` if a valid item type exists for the given server ID.
    /// Mirrors C++ pattern of checking `id < items.size()` and `items[id].id != 0`.
    pub fn has_item_type(&self, server_id: u16) -> bool {
        self.get_item_type(server_id).is_some()
    }

    /// Look up an item by name (case-insensitive).
    pub fn get_item_type_by_name(&self, name: &str) -> Option<&ItemTypeData> {
        let lower = name.to_lowercase();
        let server_id = self.name_to_id.get(&lower).copied()?;
        self.get_item_type(server_id)
    }

    /// Register a name → server_id mapping (used after XML loading).
    /// First registration wins — duplicates are silently ignored (C++ behavior).
    pub fn register_name(&mut self, name: &str, server_id: u16) {
        let lower = name.to_lowercase();
        self.name_to_id.entry(lower).or_insert(server_id);
    }

    /// Return the maximum server ID present in the registry.
    pub fn get_max_item_id(&self) -> u16 {
        // items is indexed by server_id; find the last non-zero entry
        self.items
            .iter()
            .enumerate()
            .rev()
            .find(|(_, it)| it.id != 0)
            .map(|(i, _)| i as u16)
            .unwrap_or(0)
    }

    /// Total number of item types loaded.
    pub fn len(&self) -> usize {
        self.items.iter().filter(|it| it.id != 0).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the registry. Mirrors C++ `Items::clear`.
    pub fn clear(&mut self) {
        self.items.clear();
        self.client_to_server.clear();
        self.name_to_id.clear();
        self.currency_items.clear();
        self.major_version = 0;
        self.minor_version = 0;
        self.build_number = 0;
    }

    /// Reload a registry from OTB + XML inputs. Mirrors C++ `Items::reload`,
    /// minus the side-effect calls to `g_moveEvents`/`g_weapons` (those live
    /// in the game crate).
    ///
    /// Returns `Err` if either parser rejects the input.
    pub fn reload(&mut self, otb: &[u8], xml: &str) -> Result<(), String> {
        self.clear();
        let parsed = Self::parse_otb(otb)?;
        self.major_version = parsed.major_version;
        self.minor_version = parsed.minor_version;
        self.build_number = parsed.build_number;
        self.items = parsed.items;
        self.client_to_server = parsed.client_to_server;
        self.name_to_id = parsed.name_to_id;
        self.currency_items = parsed.currency_items;
        self.load_from_xml(xml)
    }

    /// Look up a mutable item type for in-place updates (used by the XML
    /// parser to fill in fields). Mirrors C++ non-const `Items::getItemType`.
    fn get_item_type_mut(&mut self, server_id: u16) -> Option<&mut ItemTypeData> {
        self.items.get_mut(server_id as usize)
    }

    /// Ensure `items` has a slot for `server_id` and return a mutable
    /// reference.  Used by `parse_item_node` when an XML-only id (e.g. 1..99)
    /// has not been pre-allocated by the OTB pass.
    fn ensure_item_slot(&mut self, server_id: u16) -> &mut ItemTypeData {
        if server_id as usize >= self.items.len() {
            self.items
                .resize_with(server_id as usize + 1, ItemTypeData::default);
        }
        &mut self.items[server_id as usize]
    }

    /// Load item-type XML attributes into the registry.
    ///
    /// XML format (matching C++ `data/items/items.xml`):
    /// ```xml
    /// <items>
    ///   <item id="2400" article="a" name="iron sword">
    ///     <attribute key="weight" value="3500"/>
    ///     <attribute key="attack" value="35"/>
    ///   </item>
    ///   <item fromid="2000" toid="2005" article="a" name="thing"/>
    /// </items>
    /// ```
    ///
    /// File I/O is the caller's responsibility — pass the XML string
    /// directly. This mirrors the Rust pattern in `outfit.rs` / `vocation.rs`.
    pub fn load_from_xml(&mut self, xml: &str) -> Result<(), String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;

        let root = doc
            .descendants()
            .find(|n| n.has_tag_name("items"))
            .ok_or_else(|| "Missing <items> root element".to_string())?;

        for node in root.children().filter(|n| n.is_element()) {
            if let Some(id_str) = node.attribute("id") {
                let id: u16 = id_str
                    .parse()
                    .map_err(|_| format!("Invalid item id: {id_str}"))?;
                self.parse_item_node(&node, id);
                continue;
            }

            let from_id_str = match node.attribute("fromid") {
                Some(s) => s,
                None => continue,
            };
            let to_id_str = match node.attribute("toid") {
                Some(s) => s,
                None => continue,
            };
            let from_id: u16 = from_id_str
                .parse()
                .map_err(|_| format!("Invalid fromid: {from_id_str}"))?;
            let to_id: u16 = to_id_str
                .parse()
                .map_err(|_| format!("Invalid toid: {to_id_str}"))?;
            let mut id = from_id;
            while id <= to_id {
                self.parse_item_node(&node, id);
                if id == u16::MAX {
                    break;
                }
                id += 1;
            }
        }

        Ok(())
    }

    /// Parse a single `<item>` XML node and merge its attributes into the
    /// `ItemTypeData` at `id`.  Mirrors C++ `Items::parseItemNode`.
    pub fn parse_item_node(&mut self, node: &roxmltree::Node, id: u16) {
        // C++ pre-allocates ids 1..99 with their own id set, since OTB doesn't
        // generally include them.
        if (1..100).contains(&id) {
            self.ensure_item_slot(id).id = id;
        }

        let exists = self
            .get_item_type_mut(id)
            .map(|it| it.id != 0)
            .unwrap_or(false);
        if !exists {
            return;
        }

        // Duplicate detection — if name was already set, warn and skip.
        let name_set = self
            .get_item_type_mut(id)
            .map(|it| !it.name.is_empty())
            .unwrap_or(false);
        if name_set {
            return;
        }

        // Apply name + article + plural up-front so subsequent attribute
        // handlers can see them.
        let name_attr = node.attribute("name").unwrap_or("");
        let article_attr = node.attribute("article").unwrap_or("");
        let plural_attr = node.attribute("plural").unwrap_or("");
        if !name_attr.is_empty() {
            self.name_to_id
                .entry(name_attr.to_lowercase())
                .or_insert(id);
        }
        let it = self.ensure_item_slot(id);
        if !name_attr.is_empty() {
            it.name = name_attr.to_string();
        }
        if !article_attr.is_empty() {
            it.article = article_attr.to_string();
        }
        if !plural_attr.is_empty() {
            it.plural_name = plural_attr.to_string();
        }

        // Iterate <attribute key="..." value="..."> children.
        for attr_node in node.children().filter(|n| n.is_element()) {
            let key = match attr_node.attribute("key") {
                Some(s) => s,
                None => continue,
            };
            // C++ allows `minvalue`+`maxvalue` instead of `value` for duration ranges.
            let (value_str, max_value_str) = match attr_node.attribute("value") {
                Some(v) => (v, None),
                None => match (
                    attr_node.attribute("minvalue"),
                    attr_node.attribute("maxvalue"),
                ) {
                    (Some(min), Some(max)) => (min, Some(max)),
                    _ => continue,
                },
            };

            let key_lower = key.to_lowercase();
            let parse_attr = match item_parse_attributes_map().get(key_lower.as_str()) {
                Some(p) => *p,
                None => continue,
            };

            self.apply_attribute(id, parse_attr, value_str, max_value_str, &attr_node);
        }
    }

    /// Dispatch one parsed `<attribute>` to the matching `ItemTypeData`
    /// (and/or `Abilities`) field. Split out of `parse_item_node` to keep
    /// the cyclomatic complexity per function manageable.
    fn apply_attribute(
        &mut self,
        id: u16,
        attr: ItemParseAttribute,
        value: &str,
        max_value: Option<&str>,
        attr_node: &roxmltree::Node,
    ) {
        use ItemParseAttribute as P;

        // For Field / MaleTransformTo / FemaleTransformTo / Worth we need
        // multi-step mutation (sometimes touching `self.currency_items` or
        // a second item), so we keep those out of the closure.
        match attr {
            P::Field => {
                self.apply_attribute_field(id, value, attr_node);
                return;
            }
            P::MaleTransformTo => {
                let _ = self.apply_sex_transform(id, value, PlayerSex::Male, PlayerSex::Female);
                return;
            }
            P::FemaleTransformTo => {
                let _ = self.apply_sex_transform(id, value, PlayerSex::Female, PlayerSex::Male);
                return;
            }
            P::Worth => {
                if let Ok(w) = value.parse::<u64>() {
                    if !self
                        .currency_items
                        .iter()
                        .any(|(existing, _)| *existing == w)
                    {
                        self.currency_items.push((w, id));
                        // Keep sorted descending by worth.
                        self.currency_items.sort_by(|a, b| b.0.cmp(&a.0));
                        if let Some(it) = self.get_item_type_mut(id) {
                            it.worth = w;
                        }
                    }
                }
                return;
            }
            _ => {}
        }

        // `parse_item_node` already verified the item exists before calling
        // this method, so unwrap is safe.
        if let Some(it) = self.get_item_type_mut(id) {
            apply_simple_attribute(it, attr, value, max_value, attr_node);
        }
    }

    /// Common impl for `maletransformto` and `femaletransformto`.
    /// Sets `transform_to_on_use[primary]`, mirrors to `other` slot if that
    /// slot is 0, and back-links the target's `transform_to_free` if it is 0.
    fn apply_sex_transform(
        &mut self,
        id: u16,
        value: &str,
        primary: PlayerSex,
        other: PlayerSex,
    ) -> Option<()> {
        let v: u16 = value.parse().ok()?;
        let it = self.get_item_type_mut(id)?;
        it.transform_to_on_use[primary as usize] = v;
        if it.transform_to_on_use[other as usize] == 0 {
            it.transform_to_on_use[other as usize] = v;
        }
        let target = self.get_item_type_mut(v)?;
        if target.transform_to_free == 0 {
            target.transform_to_free = id;
        }
        Some(())
    }

    fn apply_attribute_field(&mut self, id: u16, value: &str, attr_node: &roxmltree::Node) {
        // Field attr sets group + type, then iterates sub-<attribute> nodes
        // to build a ConditionDamage. Mirror of items.cpp:1655-1742.
        use crate::condition::ConditionDamage;
        use forgottenserver_common::enums::{ConditionId, ConditionParam, ConditionTypeFlags};

        let (combat, condition_type) = match value.to_lowercase().as_str() {
            "fire" => (CombatTypeFlags::FIRE, ConditionTypeFlags::FIRE),
            "energy" => (CombatTypeFlags::ENERGY, ConditionTypeFlags::ENERGY),
            "poison" => (CombatTypeFlags::EARTH, ConditionTypeFlags::POISON),
            "drown" => (CombatTypeFlags::DROWN, ConditionTypeFlags::DROWN),
            "physical" => (CombatTypeFlags::PHYSICAL, ConditionTypeFlags::BLEEDING),
            _ => return,
        };

        let mut cd = ConditionDamage::new(ConditionId::Combat, condition_type, false, 0, true);

        let mut ticks: i32 = 0;
        let mut start: i32 = 0;
        let mut count: i32 = 1;
        let mut init_damage: i32 = -1;

        for sub in attr_node.children().filter(|n| n.is_element()) {
            let sub_key = match sub.attribute("key") {
                Some(s) => s,
                None => continue,
            };
            let sub_val = match sub.attribute("value") {
                Some(v) => v,
                None => continue,
            };
            match sub_key.to_lowercase().as_str() {
                "initdamage" => {
                    init_damage = sub_val.parse().unwrap_or(-1);
                }
                "ticks" => {
                    ticks = sub_val.parse().unwrap_or(0);
                }
                "count" => {
                    count = sub_val.parse().unwrap_or(1).max(1);
                }
                "start" => {
                    start = sub_val.parse().unwrap_or(0).max(0);
                }
                "damage" => {
                    let damage = -sub_val.parse::<i32>().unwrap_or(0);
                    if start > 0 {
                        let mut list: Vec<i32> = Vec::new();
                        ConditionDamage::generate_damage_list(damage, start, &mut list);
                        for v in list {
                            cd.add_damage(1, ticks, -v);
                        }
                        start = 0;
                    } else {
                        cd.add_damage(count, ticks, damage);
                    }
                }
                _ => {}
            }
        }

        // initDamage = 0 → leave as-is (don't override with damage)
        // initDamage = -1 → undefined; override with `start` if start != 0
        // initDamage > 0 or < -1 → override with -initDamage
        if !(-1..=0).contains(&init_damage) {
            cd.set_init_damage(-init_damage);
        } else if init_damage == -1 && start != 0 {
            cd.set_init_damage(start);
        }

        cd.set_param(ConditionParam::Field, 1);
        if cd.get_total_damage() > 0 {
            cd.set_param(ConditionParam::ForceUpdate, 1);
        }

        if let Some(it) = self.get_item_type_mut(id) {
            it.group = ItemGroup::MagicField;
            it.type_kind = ItemTypeKind::MagicField;
            it.combat_type = combat;
            it.condition_damage = Some(Box::new(cd));
        }
    }
}

/// Apply a "simple" attribute (one that only mutates the item itself, not
/// peer items, currency_items, or the global field-damage system).
#[allow(clippy::too_many_lines)]
fn apply_simple_attribute(
    it: &mut ItemTypeData,
    attr: ItemParseAttribute,
    value: &str,
    max_value: Option<&str>,
    _attr_node: &roxmltree::Node,
) {
    use ItemParseAttribute as P;

    // Helper lambdas
    let i = || value.parse::<i32>().unwrap_or(0);
    let u = || value.parse::<u32>().unwrap_or(0);
    let u16v = || value.parse::<u16>().unwrap_or(0);
    let i16v = || value.parse::<i16>().unwrap_or(0);
    let b = || boolean_string(value);
    let lower = || value.to_lowercase();

    match attr {
        P::Type => {
            if let Some(kind) = item_types_map().get(lower().as_str()).copied() {
                it.type_kind = kind;
                if kind == ItemTypeKind::Container {
                    it.group = ItemGroup::Container;
                }
            }
        }
        P::Description => it.description = value.to_string(),
        P::RuneSpellName => it.rune_spell_name = value.to_string(),
        P::Weight => it.weight = u(),
        P::ShowCount => it.show_count = b(),
        P::Supply => it.supply = b(),
        P::Armor => it.armor = i(),
        P::Defense => it.defense = i(),
        P::ExtraDef => it.extra_defense = i(),
        P::Attack => it.attack = i(),
        P::AttackSpeed => {
            let s = u();
            // C++ clamps attack_speed > 0 && < 100 → 100.
            it.attack_speed = if s > 0 && s < 100 { 100 } else { s };
        }
        P::RotateTo => it.rotate_to = u16v(),
        P::Moveable => it.moveable = b(),
        P::BlockProjectile => it.block_projectile = b(),
        P::Pickupable => it.allow_pickupable = b(),
        P::ForceSerialize => it.force_serialize = b(),
        P::FloorChange => {
            if let Some(flag) = tile_states_map().get(lower().as_str()).copied() {
                it.floor_change |= flag;
            }
        }
        P::CorpseType => {
            if let Some(rt) = race_types_map().get(lower().as_str()).copied() {
                it.corpse_type = rt;
            }
        }
        P::ContainerSize => it.max_items = u16v(),
        P::FluidSource => {
            if let Some(ft) = fluid_types_map().get(lower().as_str()).copied() {
                it.fluid_source = ft;
            }
        }
        P::Readable => it.can_read_text = b(),
        P::Writeable => {
            it.can_write_text = b();
            it.can_read_text = it.can_write_text;
        }
        P::MaxTextLen => it.max_text_len = u16v(),
        P::WriteOnceItemId => it.write_once_item_id = u16v(),
        P::WeaponType => {
            if let Some(wt) = weapon_types_map().get(lower().as_str()).copied() {
                it.weapon_type = wt;
            }
        }
        P::SlotType => {
            apply_slot_type(it, lower().as_str());
        }
        P::AmmoType => {
            let a = get_ammo_type(&lower());
            if a != Ammo::None {
                it.ammo_type = a;
            }
        }
        P::ShootType => {
            let st = get_shoot_type(&lower());
            if st != ShootType::None {
                it.shoot_type = st;
            }
        }
        P::Effect => {
            let eff = get_magic_effect(&lower());
            if eff != MagicEffectClass::None {
                it.magic_effect = eff;
            }
        }
        P::Range => it.shoot_range = u16v() as u8,
        P::StopDuration => it.stop_time = b(),
        P::DecayTo => it.decay_to = i(),
        P::TransformEquipTo => it.transform_equip_to = u16v(),
        P::TransformDeEquipTo => it.transform_de_equip_to = u16v(),
        P::Duration => {
            it.decay_time_min = u();
            it.decay_time_max = match max_value {
                Some(s) => s.parse::<u32>().unwrap_or(it.decay_time_min),
                None => it.decay_time_min,
            };
        }
        P::ShowDuration => it.show_duration = b(),
        P::Charges => it.charges = u(),
        P::ShowCharges => it.show_charges = b(),
        P::ShowAttributes => it.show_attributes = b(),
        P::HitChance => {
            let v = value.parse::<i16>().unwrap_or(0);
            it.hit_chance = v.clamp(-100, 100) as i8;
        }
        P::MaxHitChance => {
            let v = u();
            it.max_hit_chance = v.min(100) as i32;
        }
        P::Invisible => {
            it.abilities
                .get_or_insert_with(|| Box::new(Abilities::default()))
                .invisible = b();
        }
        P::Speed => {
            it.abilities
                .get_or_insert_with(|| Box::new(Abilities::default()))
                .speed = i();
        }
        P::HealthGain => {
            let a = it
                .abilities
                .get_or_insert_with(|| Box::new(Abilities::default()));
            a.regeneration = true;
            a.health_gain = u();
        }
        P::HealthTicks => {
            let a = it
                .abilities
                .get_or_insert_with(|| Box::new(Abilities::default()));
            a.regeneration = true;
            a.health_ticks = u();
        }
        P::ManaGain => {
            let a = it
                .abilities
                .get_or_insert_with(|| Box::new(Abilities::default()));
            a.regeneration = true;
            a.mana_gain = u();
        }
        P::ManaTicks => {
            let a = it
                .abilities
                .get_or_insert_with(|| Box::new(Abilities::default()));
            a.regeneration = true;
            a.mana_ticks = u();
        }
        P::ManaShield => {
            it.abilities
                .get_or_insert_with(|| Box::new(Abilities::default()))
                .mana_shield = b();
        }
        P::SkillSword => set_skill(it, Skill::Sword, i()),
        P::SkillAxe => set_skill(it, Skill::Axe, i()),
        P::SkillClub => set_skill(it, Skill::Club, i()),
        P::SkillDist => set_skill(it, Skill::Distance, i()),
        P::SkillFish => set_skill(it, Skill::Fishing, i()),
        P::SkillShield => set_skill(it, Skill::Shield, i()),
        P::SkillFist => set_skill(it, Skill::Fist, i()),
        P::CriticalHitAmount => set_special_skill(it, SpecialSkill::CriticalHitAmount, i()),
        P::CriticalHitChance => set_special_skill(it, SpecialSkill::CriticalHitChance, i()),
        P::ManaLeechAmount => set_special_skill(it, SpecialSkill::ManaLeechAmount, i()),
        P::ManaLeechChance => set_special_skill(it, SpecialSkill::ManaLeechChance, i()),
        P::LifeLeechAmount => set_special_skill(it, SpecialSkill::LifeLeechAmount, i()),
        P::LifeLeechChance => set_special_skill(it, SpecialSkill::LifeLeechChance, i()),
        P::MaxHitPoints => set_stat(it, Stat::MaxHitPoints, i()),
        P::MaxHitPointsPercent => set_stat_percent(it, Stat::MaxHitPoints, i()),
        P::MaxManaPoints => set_stat(it, Stat::MaxManaPoints, i()),
        P::MaxManaPointsPercent => set_stat_percent(it, Stat::MaxManaPoints, i()),
        P::MagicPoints => set_stat(it, Stat::MagicPoints, i()),
        P::MagicPointsPercent => set_stat_percent(it, Stat::MagicPoints, i()),
        P::FieldAbsorbPercentEnergy => add_field_absorb(it, CombatTypeFlags::ENERGY, i16v()),
        P::FieldAbsorbPercentFire => add_field_absorb(it, CombatTypeFlags::FIRE, i16v()),
        P::FieldAbsorbPercentPoison => add_field_absorb(it, CombatTypeFlags::EARTH, i16v()),
        P::AbsorbPercentAll => add_absorb_all(it, i16v()),
        P::AbsorbPercentElements => add_absorb_elements(it, i16v()),
        P::AbsorbPercentMagic => add_absorb_magic(it, i16v()),
        P::AbsorbPercentEnergy => add_absorb(it, CombatTypeFlags::ENERGY, i16v()),
        P::AbsorbPercentFire => add_absorb(it, CombatTypeFlags::FIRE, i16v()),
        P::AbsorbPercentPoison => add_absorb(it, CombatTypeFlags::EARTH, i16v()),
        P::AbsorbPercentIce => add_absorb(it, CombatTypeFlags::ICE, i16v()),
        P::AbsorbPercentHoly => add_absorb(it, CombatTypeFlags::HOLY, i16v()),
        P::AbsorbPercentDeath => add_absorb(it, CombatTypeFlags::DEATH, i16v()),
        P::AbsorbPercentLifeDrain => add_absorb(it, CombatTypeFlags::LIFEDRAIN, i16v()),
        P::AbsorbPercentManaDrain => add_absorb(it, CombatTypeFlags::MANADRAIN, i16v()),
        P::AbsorbPercentDrown => add_absorb(it, CombatTypeFlags::DROWN, i16v()),
        P::AbsorbPercentPhysical => add_absorb(it, CombatTypeFlags::PHYSICAL, i16v()),
        P::AbsorbPercentHealing => add_absorb(it, CombatTypeFlags::HEALING, i16v()),
        P::AbsorbPercentUndefined => add_absorb(it, CombatTypeFlags::UNDEFINED, i16v()),
        P::ReflectPercentAll => mutate_reflect_all(it, |r| r.percent += i16v() as u16),
        P::ReflectPercentElements => mutate_reflect_elements(it, |r| r.percent += i16v() as u16),
        P::ReflectPercentMagic => mutate_reflect_magic(it, |r| r.percent += i16v() as u16),
        P::ReflectPercentEnergy => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::ENERGY).percent += v;
        }
        P::ReflectPercentFire => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::FIRE).percent += v;
        }
        P::ReflectPercentEarth => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::EARTH).percent += v;
        }
        P::ReflectPercentIce => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::ICE).percent += v;
        }
        P::ReflectPercentHoly => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::HOLY).percent += v;
        }
        P::ReflectPercentDeath => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::DEATH).percent += v;
        }
        P::ReflectPercentLifeDrain => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::LIFEDRAIN).percent += v;
        }
        P::ReflectPercentManaDrain => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::MANADRAIN).percent += v;
        }
        P::ReflectPercentDrown => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::DROWN).percent += v;
        }
        P::ReflectPercentPhysical => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::PHYSICAL).percent += v;
        }
        P::ReflectPercentHealing => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::HEALING).percent += v;
        }
        P::ReflectChanceAll => mutate_reflect_all(it, |r| r.chance += i16v() as u16),
        P::ReflectChanceElements => mutate_reflect_elements(it, |r| r.chance += i16v() as u16),
        P::ReflectChanceMagic => mutate_reflect_magic(it, |r| r.chance += i16v() as u16),
        P::ReflectChanceEnergy => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::ENERGY).chance += v;
        }
        P::ReflectChanceFire => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::FIRE).chance += v;
        }
        P::ReflectChanceEarth => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::EARTH).chance += v;
        }
        P::ReflectChanceIce => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::ICE).chance += v;
        }
        P::ReflectChanceHoly => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::HOLY).chance += v;
        }
        P::ReflectChanceDeath => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::DEATH).chance += v;
        }
        P::ReflectChanceLifeDrain => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::LIFEDRAIN).chance += v;
        }
        P::ReflectChanceManaDrain => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::MANADRAIN).chance += v;
        }
        P::ReflectChanceDrown => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::DROWN).chance += v;
        }
        P::ReflectChancePhysical => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::PHYSICAL).chance += v;
        }
        P::ReflectChanceHealing => {
            let v = i16v() as u16;
            reflect_at(it, CombatTypeFlags::HEALING).chance += v;
        }
        P::BoostPercentAll => add_boost_all(it, i16v()),
        P::BoostPercentElements => add_boost_elements(it, i16v()),
        P::BoostPercentMagic => add_boost_magic(it, i16v()),
        P::BoostPercentEnergy => add_boost(it, CombatTypeFlags::ENERGY, i16v()),
        P::BoostPercentFire => add_boost(it, CombatTypeFlags::FIRE, i16v()),
        P::BoostPercentEarth => add_boost(it, CombatTypeFlags::EARTH, i16v()),
        P::BoostPercentIce => add_boost(it, CombatTypeFlags::ICE, i16v()),
        P::BoostPercentHoly => add_boost(it, CombatTypeFlags::HOLY, i16v()),
        P::BoostPercentDeath => add_boost(it, CombatTypeFlags::DEATH, i16v()),
        P::BoostPercentLifeDrain => add_boost(it, CombatTypeFlags::LIFEDRAIN, i16v()),
        P::BoostPercentManaDrain => add_boost(it, CombatTypeFlags::MANADRAIN, i16v()),
        P::BoostPercentDrown => add_boost(it, CombatTypeFlags::DROWN, i16v()),
        P::BoostPercentPhysical => add_boost(it, CombatTypeFlags::PHYSICAL, i16v()),
        P::BoostPercentHealing => add_boost(it, CombatTypeFlags::HEALING, i16v()),
        P::MagicLevelEnergy => add_magic_level(it, CombatTypeFlags::ENERGY, i16v()),
        P::MagicLevelFire => add_magic_level(it, CombatTypeFlags::FIRE, i16v()),
        P::MagicLevelPoison => add_magic_level(it, CombatTypeFlags::EARTH, i16v()),
        P::MagicLevelIce => add_magic_level(it, CombatTypeFlags::ICE, i16v()),
        P::MagicLevelHoly => add_magic_level(it, CombatTypeFlags::HOLY, i16v()),
        P::MagicLevelDeath => add_magic_level(it, CombatTypeFlags::DEATH, i16v()),
        P::MagicLevelLifeDrain => add_magic_level(it, CombatTypeFlags::LIFEDRAIN, i16v()),
        P::MagicLevelManaDrain => add_magic_level(it, CombatTypeFlags::MANADRAIN, i16v()),
        P::MagicLevelDrown => add_magic_level(it, CombatTypeFlags::DROWN, i16v()),
        P::MagicLevelPhysical => add_magic_level(it, CombatTypeFlags::PHYSICAL, i16v()),
        P::MagicLevelHealing => add_magic_level(it, CombatTypeFlags::HEALING, i16v()),
        P::MagicLevelUndefined => add_magic_level(it, CombatTypeFlags::UNDEFINED, i16v()),
        P::SuppressDrunk => suppress(it, ConditionTypeFlags::DRUNK as u32, b()),
        P::SuppressEnergy => suppress(it, ConditionTypeFlags::ENERGY as u32, b()),
        P::SuppressFire => suppress(it, ConditionTypeFlags::FIRE as u32, b()),
        P::SuppressPoison => suppress(it, ConditionTypeFlags::POISON as u32, b()),
        P::SuppressDrown => suppress(it, ConditionTypeFlags::DROWN as u32, b()),
        P::SuppressPhysical => suppress(it, ConditionTypeFlags::BLEEDING as u32, b()),
        P::SuppressFreeze => suppress(it, ConditionTypeFlags::FREEZING as u32, b()),
        P::SuppressDazzle => suppress(it, ConditionTypeFlags::DAZZLED as u32, b()),
        P::SuppressCurse => suppress(it, ConditionTypeFlags::CURSED as u32, b()),
        P::Replaceable => it.replaceable = b(),
        P::PartnerDirection => it.bed_partner_dir = get_direction(value),
        P::LevelDoor => it.level_door = u(),
        P::TransformTo => it.transform_to_free = u16v(),
        P::DestroyTo => it.destroy_to = u16v(),
        P::ElementIce => set_element(it, CombatTypeFlags::ICE, u16v()),
        P::ElementEarth => set_element(it, CombatTypeFlags::EARTH, u16v()),
        P::ElementFire => set_element(it, CombatTypeFlags::FIRE, u16v()),
        P::ElementEnergy => set_element(it, CombatTypeFlags::ENERGY, u16v()),
        P::ElementDeath => set_element(it, CombatTypeFlags::DEATH, u16v()),
        P::ElementHoly => set_element(it, CombatTypeFlags::HOLY, u16v()),
        P::WalkStack => it.walk_stack = b(),
        P::Blocking => it.block_solid = b(),
        P::AllowDistRead => it.allow_dist_read = boolean_string(value),
        P::StoreItem => it.store_item = boolean_string(value),
        // Multi-step variants are intercepted by `Items::apply_attribute`
        // and never reach this dispatcher. Listed explicitly to keep the
        // `match` exhaustive without an unreachable wildcard.
        P::Field | P::MaleTransformTo | P::FemaleTransformTo | P::Worth => {}
    }
}

fn set_skill(it: &mut ItemTypeData, s: Skill, v: i32) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .skills[s as usize] = v;
}

fn set_special_skill(it: &mut ItemTypeData, s: SpecialSkill, v: i32) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .special_skills[s as usize] = v;
}

fn set_stat(it: &mut ItemTypeData, s: Stat, v: i32) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .stats[s as usize] = v;
}

fn set_stat_percent(it: &mut ItemTypeData, s: Stat, v: i32) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .stats_percent[s as usize] = v;
}

fn add_absorb(it: &mut ItemTypeData, combat: u16, v: i16) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .absorb_percent[combat_type_to_index(combat)] += v;
}

fn add_field_absorb(it: &mut ItemTypeData, combat: u16, v: i16) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .field_absorb_percent[combat_type_to_index(combat)] += v;
}

fn add_boost(it: &mut ItemTypeData, combat: u16, v: i16) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .boost_percent[combat_type_to_index(combat)] += v;
}

fn add_magic_level(it: &mut ItemTypeData, combat: u16, v: i16) {
    it.abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .special_magic_level_skill[combat_type_to_index(combat)] += v;
}

fn add_absorb_all(it: &mut ItemTypeData, v: i16) {
    let a = it
        .abilities
        .get_or_insert_with(|| Box::new(Abilities::default()));
    for slot in a.absorb_percent.iter_mut() {
        *slot += v;
    }
}

fn add_absorb_elements(it: &mut ItemTypeData, v: i16) {
    for c in [
        CombatTypeFlags::ENERGY,
        CombatTypeFlags::FIRE,
        CombatTypeFlags::EARTH,
        CombatTypeFlags::ICE,
    ] {
        add_absorb(it, c, v);
    }
}

fn add_absorb_magic(it: &mut ItemTypeData, v: i16) {
    for c in [
        CombatTypeFlags::ENERGY,
        CombatTypeFlags::FIRE,
        CombatTypeFlags::EARTH,
        CombatTypeFlags::ICE,
        CombatTypeFlags::HOLY,
        CombatTypeFlags::DEATH,
    ] {
        add_absorb(it, c, v);
    }
}

fn add_boost_all(it: &mut ItemTypeData, v: i16) {
    let a = it
        .abilities
        .get_or_insert_with(|| Box::new(Abilities::default()));
    for slot in a.boost_percent.iter_mut() {
        *slot += v;
    }
}

fn add_boost_elements(it: &mut ItemTypeData, v: i16) {
    for c in [
        CombatTypeFlags::ENERGY,
        CombatTypeFlags::FIRE,
        CombatTypeFlags::EARTH,
        CombatTypeFlags::ICE,
    ] {
        add_boost(it, c, v);
    }
}

fn add_boost_magic(it: &mut ItemTypeData, v: i16) {
    for c in [
        CombatTypeFlags::ENERGY,
        CombatTypeFlags::FIRE,
        CombatTypeFlags::EARTH,
        CombatTypeFlags::ICE,
        CombatTypeFlags::HOLY,
        CombatTypeFlags::DEATH,
    ] {
        add_boost(it, c, v);
    }
}

fn reflect_at(it: &mut ItemTypeData, combat: u16) -> &mut Reflect {
    &mut it
        .abilities
        .get_or_insert_with(|| Box::new(Abilities::default()))
        .reflect[combat_type_to_index(combat)]
}

fn mutate_reflect_all(it: &mut ItemTypeData, mut f: impl FnMut(&mut Reflect)) {
    let a = it
        .abilities
        .get_or_insert_with(|| Box::new(Abilities::default()));
    for r in a.reflect.iter_mut() {
        f(r);
    }
}

fn mutate_reflect_elements(it: &mut ItemTypeData, mut f: impl FnMut(&mut Reflect)) {
    for c in [
        CombatTypeFlags::ENERGY,
        CombatTypeFlags::FIRE,
        CombatTypeFlags::EARTH,
        CombatTypeFlags::ICE,
    ] {
        f(reflect_at(it, c));
    }
}

fn mutate_reflect_magic(it: &mut ItemTypeData, mut f: impl FnMut(&mut Reflect)) {
    for c in [
        CombatTypeFlags::ENERGY,
        CombatTypeFlags::FIRE,
        CombatTypeFlags::EARTH,
        CombatTypeFlags::ICE,
        CombatTypeFlags::HOLY,
        CombatTypeFlags::DEATH,
    ] {
        f(reflect_at(it, c));
    }
}

fn suppress(it: &mut ItemTypeData, mask: u32, on: bool) {
    if on {
        it.abilities
            .get_or_insert_with(|| Box::new(Abilities::default()))
            .condition_suppressions |= mask;
    }
}

fn set_element(it: &mut ItemTypeData, combat: u16, dmg: u16) {
    let a = it
        .abilities
        .get_or_insert_with(|| Box::new(Abilities::default()));
    a.element_damage = dmg;
    a.element_type = combat;
}

fn apply_slot_type(it: &mut ItemTypeData, lower_value: &str) {
    use slot_position::*;
    match lower_value {
        "head" => it.slot_position |= HEAD,
        "body" => it.slot_position |= ARMOR,
        "legs" => it.slot_position |= LEGS,
        "feet" => it.slot_position |= FEET,
        "backpack" => it.slot_position |= BACKPACK,
        "two-handed" => it.slot_position |= TWO_HAND,
        "right-hand" => it.slot_position &= !LEFT,
        "left-hand" => it.slot_position &= !RIGHT,
        "necklace" => it.slot_position |= NECKLACE,
        "ring" => it.slot_position |= RING,
        "ammo" => it.slot_position |= AMMO,
        "hand" => it.slot_position |= HAND,
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Raw OTB node parser (returns unescaped props + child (type, props) pairs)
// ---------------------------------------------------------------------------

/// `(child_type_byte, child_unescaped_props)` — returned by [`parse_node_raw`].
type ChildNode = (u8, Vec<u8>);

/// Parse one OTB node starting just *after* the leading NODE_START byte.
/// Returns `(type_byte, unescaped_props, children)` where each child is
/// `(child_type_byte, child_unescaped_props)`.
fn parse_node_raw(data: &[u8], pos: &mut usize) -> Result<(u8, Vec<u8>, Vec<ChildNode>), String> {
    use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};

    if *pos >= data.len() {
        return Err("OTB: unexpected end (expected type byte)".into());
    }

    let type_byte = data[*pos];
    *pos += 1;

    let mut props: Vec<u8> = Vec::new();
    let mut children: Vec<(u8, Vec<u8>)> = Vec::new();

    loop {
        if *pos >= data.len() {
            return Err("OTB: unexpected end inside node".into());
        }

        match data[*pos] {
            NODE_START => {
                *pos += 1;
                let (ct, cp, _) = parse_node_raw(data, pos)?;
                children.push((ct, cp));
            }
            NODE_END => {
                *pos += 1;
                break;
            }
            ESCAPE => {
                *pos += 1;
                if *pos >= data.len() {
                    return Err("OTB: unexpected end after ESCAPE".into());
                }
                props.push(data[*pos]);
                *pos += 1;
            }
            b => {
                props.push(b);
                *pos += 1;
            }
        }
    }

    Ok((type_byte, props, children))
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
mod helpers {
    use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};

    // VersionInfo wire size: 4+4+4+128 = 140 bytes
    const VERSIONINFO_SIZE: usize = 140;

    /// Build a full OTB binary blob from scratch.
    ///
    /// `major` / `minor` / `build` → version header
    /// `item_nodes` → list of (flags, server_id, client_id, extra_attrs) tuples
    pub fn build_otb(major: u32, minor: u32, build: u32, item_nodes: &[OtbItemNode]) -> Vec<u8> {
        // Root props: 4-byte flags + 0x01 (ROOT_ATTR_VERSION) + 2-byte datalen + 140-byte VERSIONINFO
        let mut root_props: Vec<u8> = Vec::new();
        // flags (4 bytes, always 0 for root)
        root_props.extend_from_slice(&0u32.to_le_bytes());
        // attr = ROOT_ATTR_VERSION = 0x01
        root_props.push(0x01);
        // datalen = 140
        root_props.extend_from_slice(&(VERSIONINFO_SIZE as u16).to_le_bytes());
        // VERSIONINFO: major(4) + minor(4) + build(4) + csd_string(128)
        root_props.extend_from_slice(&major.to_le_bytes());
        root_props.extend_from_slice(&minor.to_le_bytes());
        root_props.extend_from_slice(&build.to_le_bytes());
        root_props.extend_from_slice(&[0u8; 128]);

        // Build child item nodes
        let mut child_bytes: Vec<Vec<u8>> = Vec::new();
        for item in item_nodes {
            child_bytes.push(build_item_node(item));
        }

        build_raw_otb(&root_props, &child_bytes)
    }

    /// An item node descriptor for test OTB builder.
    pub struct OtbItemNode {
        pub group: u8, // ItemGroup discriminant
        pub flags: u32,
        pub server_id: u16,
        pub client_id: u16,
        pub speed: u16,
        pub light_level: u16,
        pub light_color: u16,
        pub ware_id: u16,
        pub top_order: u8,
        pub classification: u8,
    }

    impl Default for OtbItemNode {
        fn default() -> Self {
            OtbItemNode {
                group: 1, // Ground
                flags: 0,
                server_id: 100,
                client_id: 200,
                speed: 0,
                light_level: 0,
                light_color: 0,
                ware_id: 0,
                top_order: 0,
                classification: 0,
            }
        }
    }

    /// Serialise an OtbItemNode into raw OTB bytes (unescaped props will be
    /// wrapped in a proper node frame by the caller).
    fn build_item_node(item: &OtbItemNode) -> Vec<u8> {
        let mut props: Vec<u8> = Vec::new();
        // 4-byte flags
        props.extend_from_slice(&item.flags.to_le_bytes());
        // ITEM_ATTR_SERVERID = 0x10
        write_attr_u16(&mut props, 0x10, item.server_id);
        // ITEM_ATTR_CLIENTID = 0x11
        write_attr_u16(&mut props, 0x11, item.client_id);
        // ITEM_ATTR_SPEED = 0x14
        if item.speed != 0 {
            write_attr_u16(&mut props, 0x14, item.speed);
        }
        // ITEM_ATTR_LIGHT2 = 0x2A  (light_level:u16, light_color:u16)
        if item.light_level != 0 || item.light_color != 0 {
            props.push(0x2A);
            props.extend_from_slice(&4u16.to_le_bytes());
            props.extend_from_slice(&item.light_level.to_le_bytes());
            props.extend_from_slice(&item.light_color.to_le_bytes());
        }
        // ITEM_ATTR_TOPORDER = 0x2B
        if item.top_order != 0 {
            props.push(0x2B);
            props.extend_from_slice(&1u16.to_le_bytes());
            props.push(item.top_order);
        }
        // ITEM_ATTR_WAREID = 0x2D
        if item.ware_id != 0 {
            write_attr_u16(&mut props, 0x2D, item.ware_id);
        }
        // ITEM_ATTR_CLASSIFICATION = 0x2E
        if item.classification != 0 {
            props.push(0x2E);
            props.extend_from_slice(&1u16.to_le_bytes());
            props.push(item.classification);
        }

        make_node(item.group, &props)
    }

    fn write_attr_u16(buf: &mut Vec<u8>, attr: u8, val: u16) {
        buf.push(attr);
        buf.extend_from_slice(&2u16.to_le_bytes());
        buf.extend_from_slice(&val.to_le_bytes());
    }

    /// Wrap `props` in a NODE_START / type_byte / escaped_props / NODE_END frame.
    fn make_node(type_byte: u8, props: &[u8]) -> Vec<u8> {
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

    /// Build a full raw OTB file with 4-byte identifier + root node.
    fn build_raw_otb(root_props: &[u8], children: &[Vec<u8>]) -> Vec<u8> {
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49]; // "OTBI" identifier
        buf.push(NODE_START);
        buf.push(0x00); // root type byte
        for &b in root_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        for child in children {
            buf.extend_from_slice(child);
        }
        buf.push(NODE_END);
        buf
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::helpers::*;
    use super::*;
    use forgottenserver_common::itemloader::CLIENT_VERSION_LAST;

    // -----------------------------------------------------------------------
    // OTB Parsing — version validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_correct_version_ok() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let result = Items::load_from_otb(&otb);
        assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
    }

    #[test]
    fn test_load_wrong_major_version_err() {
        let otb = build_otb(2, CLIENT_VERSION_LAST, 1, &[]);
        let result = Items::load_from_otb(&otb);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("incompatible major version"), "msg: {}", msg);
    }

    #[test]
    fn test_load_generic_major_version_ok() {
        // 0xFFFFFFFF = generic client version, should be accepted
        let otb = build_otb(0xFFFF_FFFF, 0, 0, &[]);
        let result = Items::load_from_otb(&otb);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_minor_version_too_old_err() {
        // Minor version too old
        let otb = build_otb(3, CLIENT_VERSION_LAST - 1, 1, &[]);
        let result = Items::load_from_otb(&otb);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("too old"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // OTB Parsing — item node parsing
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_one_item_server_id_and_client_id() {
        let node = OtbItemNode {
            server_id: 100,
            client_id: 200,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");

        let it = registry.get_item_type(100).expect("item 100 exists");
        assert_eq!(it.id, 100);
        assert_eq!(it.client_id, 200);
        assert_eq!(it.group, ItemGroup::Ground);
    }

    #[test]
    fn test_parse_item_flags_stackable() {
        let node = OtbItemNode {
            server_id: 5,
            client_id: 10,
            group: ItemGroup::Ground as u8,
            flags: item_flags::FLAG_STACKABLE,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");

        let it = registry.get_item_type(5).expect("item 5 exists");
        assert!(it.stackable);
        assert!(!it.moveable);
    }

    #[test]
    fn test_parse_item_flags_moveable_and_pickupable() {
        let node = OtbItemNode {
            server_id: 42,
            client_id: 100,
            group: ItemGroup::Armor as u8,
            flags: item_flags::FLAG_MOVEABLE | item_flags::FLAG_PICKUPABLE,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");

        let it = registry.get_item_type(42).expect("item 42 exists");
        assert!(it.moveable);
        assert!(it.pickupable);
        assert!(!it.stackable);
    }

    #[test]
    fn test_parse_multiple_items() {
        let nodes = vec![
            OtbItemNode {
                server_id: 1,
                client_id: 10,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 2,
                client_id: 20,
                group: ItemGroup::Container as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 3,
                client_id: 30,
                group: ItemGroup::Splash as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let registry = Items::load_from_otb(&otb).expect("parse ok");

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.get_item_type(1).unwrap().group, ItemGroup::Ground);
        assert_eq!(
            registry.get_item_type(2).unwrap().group,
            ItemGroup::Container
        );
        assert_eq!(registry.get_item_type(3).unwrap().group, ItemGroup::Splash);
    }

    // -----------------------------------------------------------------------
    // Registry lookups
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_type_missing_returns_none() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert!(registry.get_item_type(999).is_none());
    }

    #[test]
    fn test_get_item_type_by_client_id() {
        let node = OtbItemNode {
            server_id: 77,
            client_id: 333,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");

        let it = registry
            .get_item_type_by_client_id(333)
            .expect("found by client id");
        assert_eq!(it.id, 77);
        assert_eq!(it.client_id, 333);
    }

    #[test]
    fn test_get_item_type_by_client_id_missing_returns_none() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert!(registry.get_item_type_by_client_id(9999).is_none());
    }

    #[test]
    fn test_get_max_item_id() {
        let nodes = vec![
            OtbItemNode {
                server_id: 10,
                client_id: 1,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 500,
                client_id: 2,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 250,
                client_id: 3,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_max_item_id(), 500);
    }

    #[test]
    fn test_get_max_item_id_empty_registry() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_max_item_id(), 0);
    }

    #[test]
    fn test_get_item_type_by_name_case_insensitive() {
        let node = OtbItemNode {
            server_id: 55,
            client_id: 55,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let mut registry = Items::load_from_otb(&otb).expect("parse ok");

        // Register name manually (XML step would do this)
        registry.register_name("Iron Sword", 55);

        assert!(registry.get_item_type_by_name("iron sword").is_some());
        assert!(registry.get_item_type_by_name("IRON SWORD").is_some());
        assert!(registry.get_item_type_by_name("Iron Sword").is_some());
        assert!(registry.get_item_type_by_name("wooden shield").is_none());
    }

    // -----------------------------------------------------------------------
    // ItemTypeData helper methods
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_type_data_is_ground_tile() {
        let mut it = ItemTypeData::default();
        it.group = ItemGroup::Ground;
        assert!(it.is_ground_tile());
        it.group = ItemGroup::Container;
        assert!(!it.is_ground_tile());
    }

    #[test]
    fn test_item_type_data_is_container() {
        let mut it = ItemTypeData::default();
        it.group = ItemGroup::Container;
        assert!(it.is_container());
    }

    #[test]
    fn test_item_type_data_is_pickupable_either_flag() {
        let mut it = ItemTypeData::default();
        it.pickupable = true;
        assert!(it.is_pickupable());

        let mut it2 = ItemTypeData::default();
        it2.allow_pickupable = true;
        assert!(it2.is_pickupable());

        let it3 = ItemTypeData::default();
        assert!(!it3.is_pickupable());
    }

    #[test]
    fn test_item_type_data_get_plural_name_explicit() {
        let mut it = ItemTypeData::default();
        it.name = "sword".into();
        it.plural_name = "swords of doom".into();
        assert_eq!(it.get_plural_name(), "swords of doom");
    }

    #[test]
    fn test_item_type_data_get_plural_name_derived() {
        let mut it = ItemTypeData::default();
        it.name = "sword".into();
        // show_count defaults to true
        assert_eq!(it.get_plural_name(), "swords");
    }

    #[test]
    fn test_item_type_data_get_plural_name_ends_with_s() {
        let mut it = ItemTypeData::default();
        it.name = "moss".into();
        assert_eq!(it.get_plural_name(), "moss");
    }

    #[test]
    fn test_item_type_data_get_plural_name_show_count_false() {
        let mut it = ItemTypeData::default();
        it.name = "apple".into();
        it.show_count = false;
        assert_eq!(it.get_plural_name(), "apple");
    }

    // -----------------------------------------------------------------------
    // Group → TypeKind mapping
    // -----------------------------------------------------------------------

    #[test]
    fn test_container_group_sets_container_type_kind() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 1,
            group: ItemGroup::Container as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry.get_item_type(1).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::Container);
    }

    #[test]
    fn test_podium_group_sets_podium_type_kind() {
        let node = OtbItemNode {
            server_id: 2,
            client_id: 2,
            group: ItemGroup::Podium as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry.get_item_type(2).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::Podium);
    }

    // -----------------------------------------------------------------------
    // Malformed OTB data
    // -----------------------------------------------------------------------

    #[test]
    fn test_too_small_returns_err() {
        let result = Items::load_from_otb(&[0x00, 0x01]);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_node_start_returns_err() {
        // 4 identifier bytes + wrong byte instead of NODE_START
        let data = [0x4F, 0x54, 0x42, 0x49, 0x00, 0x00, 0x00];
        let result = Items::load_from_otb(&data);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // has_item_type — existence check
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_item_type_present() {
        let node = OtbItemNode {
            server_id: 42,
            client_id: 100,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert!(registry.has_item_type(42));
    }

    #[test]
    fn test_has_item_type_absent_returns_false() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert!(!registry.has_item_type(999));
    }

    #[test]
    fn test_has_item_type_zero_returns_false() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert!(!registry.has_item_type(0));
    }

    // -----------------------------------------------------------------------
    // get_item_type_by_client_id — client_id < 100 guard
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_type_by_client_id_below_100_returns_none() {
        // Even if client_id is in the map, values < 100 must be rejected (C++ gate)
        let node = OtbItemNode {
            server_id: 10,
            client_id: 50,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        // client_id 50 is < 100, must return None
        assert!(registry.get_item_type_by_client_id(50).is_none());
    }

    #[test]
    fn test_get_item_type_by_client_id_exactly_100_ok() {
        let node = OtbItemNode {
            server_id: 77,
            client_id: 100,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry
            .get_item_type_by_client_id(100)
            .expect("client_id 100 ok");
        assert_eq!(it.id, 77);
    }

    // -----------------------------------------------------------------------
    // register_name — first-wins duplicate handling
    // -----------------------------------------------------------------------

    #[test]
    fn test_register_name_first_wins_duplicate() {
        let nodes = vec![
            OtbItemNode {
                server_id: 1,
                client_id: 101,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 2,
                client_id: 102,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut registry = Items::load_from_otb(&otb).expect("parse ok");

        // First registration wins
        registry.register_name("sword", 1);
        registry.register_name("sword", 2); // duplicate — must be ignored

        let it = registry
            .get_item_type_by_name("sword")
            .expect("sword found");
        assert_eq!(it.id, 1, "first registration should win");
    }

    // -----------------------------------------------------------------------
    // len / is_empty
    // -----------------------------------------------------------------------

    #[test]
    fn test_len_is_empty() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_len_with_items() {
        let nodes = vec![
            OtbItemNode {
                server_id: 1,
                client_id: 101,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 2,
                client_id: 102,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
    }

    // -----------------------------------------------------------------------
    // ItemTypeData type-kind predicates (is_door, is_magic_field, etc.)
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_door() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Door;
        assert!(it.is_door());
        assert!(!it.is_magic_field());
    }

    #[test]
    fn test_is_magic_field() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::MagicField;
        assert!(it.is_magic_field());
        assert!(!it.is_door());
    }

    #[test]
    fn test_is_teleport() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Teleport;
        assert!(it.is_teleport());
    }

    #[test]
    fn test_is_key() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Key;
        assert!(it.is_key());
    }

    #[test]
    fn test_is_depot() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Depot;
        assert!(it.is_depot());
    }

    #[test]
    fn test_is_mailbox() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Mailbox;
        assert!(it.is_mailbox());
    }

    #[test]
    fn test_is_trash_holder() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::TrashHolder;
        assert!(it.is_trash_holder());
    }

    #[test]
    fn test_is_bed() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Bed;
        assert!(it.is_bed());
    }

    #[test]
    fn test_is_rune() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Rune;
        assert!(it.is_rune());
    }

    #[test]
    fn test_is_podium() {
        let mut it = ItemTypeData::default();
        it.type_kind = ItemTypeKind::Podium;
        assert!(it.is_podium());
    }

    #[test]
    fn test_is_useable() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_useable());
        it.useable = true;
        assert!(it.is_useable());
    }

    #[test]
    fn test_is_supply() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_supply());
        it.supply = true;
        assert!(it.is_supply());
    }

    // -----------------------------------------------------------------------
    // has_sub_type — fluid container | splash | stackable | charges != 0
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_sub_type_fluid_container() {
        let mut it = ItemTypeData::default();
        it.group = ItemGroup::Fluid;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_has_sub_type_splash() {
        let mut it = ItemTypeData::default();
        it.group = ItemGroup::Splash;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_has_sub_type_stackable() {
        let mut it = ItemTypeData::default();
        it.stackable = true;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_has_sub_type_charges() {
        let mut it = ItemTypeData::default();
        it.charges = 5;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_has_sub_type_none() {
        let it = ItemTypeData::default();
        // Default group is None, stackable=false, charges=0
        assert!(!it.has_sub_type());
    }

    // -----------------------------------------------------------------------
    // OTB: unknown group byte → Err
    // -----------------------------------------------------------------------

    #[test]
    fn test_unknown_item_group_byte_returns_err() {
        // Build OTB with a custom item node using an out-of-range group byte (255)
        use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};

        // We build a minimal OTB by hand with an item node type byte = 255
        // Root props: flags(4) + attr(1) + datalen(2) + VERSIONINFO(140)
        let mut root_props: Vec<u8> = Vec::new();
        root_props.extend_from_slice(&0u32.to_le_bytes()); // flags
        root_props.push(0x01); // ROOT_ATTR_VERSION
        root_props.extend_from_slice(&140u16.to_le_bytes()); // datalen
        root_props.extend_from_slice(&3u32.to_le_bytes()); // major
        root_props.extend_from_slice(&CLIENT_VERSION_LAST.to_le_bytes()); // minor
        root_props.extend_from_slice(&1u32.to_le_bytes()); // build
        root_props.extend_from_slice(&[0u8; 128]); // csd

        // Item props: flags(4) + server_id attr + client_id attr.  Server id
        // is deliberately a value whose little-endian bytes coincide with the
        // framing markers NODE_START / NODE_END so the manual escape branches
        // below are exercised.  CSD bytes 0..3 are also set to the three
        // special bytes so the root-side escape branch is hit as well.
        root_props[100] = NODE_START;
        root_props[101] = NODE_END;
        root_props[102] = ESCAPE;
        let mut item_props: Vec<u8> = Vec::new();
        item_props.extend_from_slice(&0u32.to_le_bytes()); // flags
                                                           // server_id attr — payload contains NODE_START + NODE_END
        item_props.push(0x10);
        item_props.extend_from_slice(&2u16.to_le_bytes());
        item_props.push(NODE_START);
        item_props.push(NODE_END);
        // client_id attr — payload contains ESCAPE byte to hit that branch too
        item_props.push(0x11);
        item_props.extend_from_slice(&2u16.to_le_bytes());
        item_props.push(ESCAPE);
        item_props.push(0x00);

        // Build the full OTB
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49]; // OTBI
        buf.push(NODE_START);
        buf.push(0x00); // root type
                        // escape root_props
        for &b in &root_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        // child node with type byte = 255
        buf.push(NODE_START);
        buf.push(255u8); // unknown group
        for &b in &item_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        buf.push(NODE_END);
        buf.push(NODE_END);

        let result = Items::load_from_otb(&buf);
        assert!(result.is_err(), "unknown group byte should return Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown item group"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // OTB: attribute length mismatch → Err
    // -----------------------------------------------------------------------

    #[test]
    fn test_server_id_attr_wrong_datalen_returns_err() {
        use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};

        // Build a root with correct version, then an item node where
        // ITEM_ATTR_SERVERID declares datalen=3 instead of 2 → should Err
        let mut root_props: Vec<u8> = Vec::new();
        root_props.extend_from_slice(&0u32.to_le_bytes());
        root_props.push(0x01);
        root_props.extend_from_slice(&140u16.to_le_bytes());
        root_props.extend_from_slice(&3u32.to_le_bytes()); // major
        root_props.extend_from_slice(&CLIENT_VERSION_LAST.to_le_bytes());
        root_props.extend_from_slice(&1u32.to_le_bytes());
        root_props.extend_from_slice(&[0u8; 128]);
        // Append a special byte to exercise the escape branch in the buffer-build loop.
        root_props.push(ESCAPE);

        // Item node: flags(4) + SERVERID attr with wrong datalen=3
        let mut item_props: Vec<u8> = Vec::new();
        item_props.extend_from_slice(&0u32.to_le_bytes()); // flags
        item_props.push(0x10); // ITEM_ATTR_SERVERID
        item_props.extend_from_slice(&3u16.to_le_bytes()); // wrong length: 3 instead of 2
        item_props.extend_from_slice(&100u16.to_le_bytes());
        item_props.push(0x00); // pad byte for wrong length
                               // Append a special byte to exercise the escape branch in the item-props loop.
        item_props.push(ESCAPE);

        let mut buf = vec![0x4F, 0x54, 0x42, 0x49];
        buf.push(NODE_START);
        buf.push(0x00);
        for &b in &root_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        buf.push(NODE_START);
        buf.push(1u8); // Ground group
        for &b in &item_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        buf.push(NODE_END);
        buf.push(NODE_END);

        let result = Items::load_from_otb(&buf);
        assert!(result.is_err(), "wrong ServerId datalen should be Err");
        let msg = result.unwrap_err();
        assert!(msg.contains("ServerId wrong length"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // OTB: client_id first-wins on duplicate registrations
    // -----------------------------------------------------------------------

    #[test]
    fn test_client_id_first_wins_on_duplicate() {
        // Two items with the same client_id — first one should win
        let nodes = vec![
            OtbItemNode {
                server_id: 10,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 20,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let registry = Items::load_from_otb(&otb).expect("parse ok");

        let it = registry
            .get_item_type_by_client_id(200)
            .expect("found by client_id 200");
        assert_eq!(
            it.id, 10,
            "first server_id should win for duplicate client_id"
        );
    }

    // -----------------------------------------------------------------------
    // OTB: all OTB flag bits round-trip correctly
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_otb_flags_parsed() {
        let all_flags = item_flags::FLAG_BLOCK_SOLID
            | item_flags::FLAG_BLOCK_PROJECTILE
            | item_flags::FLAG_BLOCK_PATHFIND
            | item_flags::FLAG_HAS_HEIGHT
            | item_flags::FLAG_USEABLE
            | item_flags::FLAG_PICKUPABLE
            | item_flags::FLAG_MOVEABLE
            | item_flags::FLAG_STACKABLE
            | item_flags::FLAG_ALWAYSONTOP
            | item_flags::FLAG_VERTICAL
            | item_flags::FLAG_HORIZONTAL
            | item_flags::FLAG_HANGABLE
            | item_flags::FLAG_ALLOWDISTREAD
            | item_flags::FLAG_ROTATABLE
            | item_flags::FLAG_READABLE
            | item_flags::FLAG_LOOKTHROUGH
            | item_flags::FLAG_ANIMATION
            | item_flags::FLAG_FORCEUSE
            | item_flags::FLAG_CLIENTCHARGES
            | item_flags::FLAG_CLIENTDURATION;

        let node = OtbItemNode {
            server_id: 1,
            client_id: 101,
            group: ItemGroup::Ground as u8,
            flags: all_flags,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry.get_item_type(1).unwrap();

        assert!(it.block_solid);
        assert!(it.block_projectile);
        assert!(it.block_path_find);
        assert!(it.has_height);
        assert!(it.useable);
        assert!(it.pickupable);
        assert!(it.moveable);
        assert!(it.stackable);
        assert!(it.always_on_top);
        assert!(it.is_vertical);
        assert!(it.is_horizontal);
        assert!(it.is_hangable);
        assert!(it.allow_dist_read);
        assert!(it.rotatable);
        assert!(it.can_read_text);
        assert!(it.look_through);
        assert!(it.is_animation);
        assert!(it.force_use);
        assert!(it.show_client_charges);
        assert!(it.show_client_duration);
    }

    // -----------------------------------------------------------------------
    // OTB: speed / light / ware_id / top_order / classification attributes
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_item_speed_attribute() {
        let node = OtbItemNode {
            server_id: 5,
            client_id: 105,
            group: ItemGroup::Ground as u8,
            speed: 200,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(5).unwrap().speed, 200);
    }

    #[test]
    fn test_parse_item_light_attributes() {
        let node = OtbItemNode {
            server_id: 6,
            client_id: 106,
            group: ItemGroup::Ground as u8,
            light_level: 7,
            light_color: 215,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry.get_item_type(6).unwrap();
        assert_eq!(it.light_level, 7);
        assert_eq!(it.light_color, 215);
    }

    #[test]
    fn test_parse_item_ware_id_attribute() {
        let node = OtbItemNode {
            server_id: 7,
            client_id: 107,
            group: ItemGroup::Ground as u8,
            ware_id: 999,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(7).unwrap().ware_id, 999);
    }

    #[test]
    fn test_parse_item_top_order_attribute() {
        let node = OtbItemNode {
            server_id: 8,
            client_id: 108,
            group: ItemGroup::Ground as u8,
            top_order: 3,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(8).unwrap().always_on_top_order, 3);
    }

    #[test]
    fn test_parse_item_classification_attribute() {
        let node = OtbItemNode {
            server_id: 9,
            client_id: 109,
            group: ItemGroup::Ground as u8,
            classification: 4,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(9).unwrap().classification, 4);
    }

    // -----------------------------------------------------------------------
    // ItemTypeData — OTB group → type_kind mappings (door / magicfield / teleport)
    // -----------------------------------------------------------------------

    #[test]
    fn test_door_group_sets_door_type_kind() {
        let node = OtbItemNode {
            server_id: 3,
            client_id: 103,
            group: ItemGroup::Door as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry.get_item_type(3).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::Door);
        assert!(it.is_door());
    }

    #[test]
    fn test_magic_field_group_sets_magic_field_type_kind() {
        let node = OtbItemNode {
            server_id: 4,
            client_id: 104,
            group: ItemGroup::MagicField as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry.get_item_type(4).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::MagicField);
        assert!(it.is_magic_field());
    }

    #[test]
    fn test_teleport_group_sets_teleport_type_kind() {
        let node = OtbItemNode {
            server_id: 5,
            client_id: 105,
            group: ItemGroup::Teleport as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        let it = registry.get_item_type(5).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::Teleport);
        assert!(it.is_teleport());
    }

    // -----------------------------------------------------------------------
    // ItemTypeData — default field values match C++ defaults
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_field_values() {
        let it = ItemTypeData::default();
        assert_eq!(
            it.slot_position,
            slot_position::HAND,
            "default slot is HAND"
        );
        assert_eq!(it.max_items, 8, "default container size is 8");
        assert_eq!(it.shoot_range, 1, "default shoot range is 1");
        assert_eq!(it.decay_to, -1, "default decay_to is -1");
        assert_eq!(it.max_hit_chance, -1, "default max_hit_chance is -1");
        assert!(it.show_count, "default show_count is true");
        assert!(it.replaceable, "default replaceable is true");
        assert!(it.walk_stack, "default walk_stack is true");
        assert!(!it.supply, "default supply is false");
        assert!(!it.stop_time, "default stop_time is false");
    }

    // -----------------------------------------------------------------------
    // XML fields — register_name empty name guard
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_type_by_name_empty_returns_none() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert!(registry.get_item_type_by_name("").is_none());
    }

    // -----------------------------------------------------------------------
    // Items::new — exposed constructor
    // -----------------------------------------------------------------------

    #[test]
    fn test_items_new_creates_empty_registry() {
        let registry = Items::new();
        assert_eq!(registry.major_version, 0);
        assert_eq!(registry.minor_version, 0);
        assert_eq!(registry.build_number, 0);
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert!(registry.get_item_type(1).is_none());
        assert!(registry.get_item_type_by_client_id(100).is_none());
        assert!(registry.get_item_type_by_name("anything").is_none());
        assert_eq!(registry.get_max_item_id(), 0);
    }

    // -----------------------------------------------------------------------
    // PropReader — truncated-input None paths (read_u8 / u16 / u32 / skip)
    // -----------------------------------------------------------------------
    //
    // The PropReader is private; we exercise the same paths indirectly through
    // hand-rolled OTB blobs that produce truncated streams.

    #[test]
    fn test_prop_reader_read_u8_returns_none_on_truncation() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // Build an OTB where the root props are *empty*: only the 4-byte
        // identifier, NODE_START, root type byte (0x00), NODE_END.
        // PropReader::read_u32 (for flags) hits truncation immediately.
        let buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00, NODE_END];
        let result = Items::load_from_otb(&buf);
        assert!(result.is_err(), "empty root props must err");
        let msg = result.unwrap_err();
        assert!(msg.contains("OTB root: missing flags"), "msg: {}", msg);
    }

    #[test]
    fn test_prop_reader_read_u8_truncated_after_flags_err() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // Root props: only 4 bytes of flags, no attr byte.
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        buf.extend_from_slice(&0u32.to_le_bytes()); // flags only
        buf.push(NODE_END);
        let result = Items::load_from_otb(&buf);
        let msg = result.expect_err("must err");
        assert!(msg.contains("OTB root: missing attr"), "msg: {}", msg);
    }

    #[test]
    fn test_prop_reader_read_u16_returns_none_on_truncation() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // flags(4) + attr(0x01 = Version), but no datalen u16
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        buf.extend_from_slice(&0u32.to_le_bytes()); // flags
        buf.push(0x01); // attr = ROOT_ATTR_VERSION
        buf.push(NODE_END);
        let result = Items::load_from_otb(&buf);
        let msg = result.expect_err("must err");
        assert!(msg.contains("OTB root: missing datalen"), "msg: {}", msg);
    }

    #[test]
    fn test_prop_reader_read_u32_returns_none_on_truncation() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // flags(4) + attr(0x01) + datalen(140 = correct) but the VersionInfo
        // payload is truncated mid-u32 (only 3 bytes of major version).
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        buf.extend_from_slice(&0u32.to_le_bytes()); // flags
        buf.push(0x01); // attr
        buf.extend_from_slice(&140u16.to_le_bytes()); // datalen
        buf.extend_from_slice(&[0u8, 0, 0]); // 3-byte truncated u32
        buf.push(NODE_END);
        let result = Items::load_from_otb(&buf);
        let msg = result.expect_err("must err");
        assert!(msg.contains("missing major version"), "msg: {}", msg);
    }

    #[test]
    fn test_prop_reader_skip_returns_false_on_truncation() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // Build a root with valid header up to and including major/minor/build,
        // but the 128-byte CSD string is missing entirely.
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        buf.extend_from_slice(&0u32.to_le_bytes()); // flags
        buf.push(0x01); // attr
        buf.extend_from_slice(&140u16.to_le_bytes()); // datalen
        buf.extend_from_slice(&3u32.to_le_bytes()); // major
        buf.extend_from_slice(&(CLIENT_VERSION_LAST).to_le_bytes()); // minor
        buf.extend_from_slice(&1u32.to_le_bytes()); // build
                                                    // No CSD bytes — `skip(128)` returns false.
        buf.push(NODE_END);
        let result = Items::load_from_otb(&buf);
        let msg = result.expect_err("must err");
        assert!(msg.contains("truncated CSD string"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // Root VersionInfo: declared datalen != 140 must Err
    // -----------------------------------------------------------------------

    #[test]
    fn test_root_versioninfo_size_mismatch_err() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // datalen declared as 139 instead of 140
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.push(0x01);
        buf.extend_from_slice(&139u16.to_le_bytes()); // wrong
        buf.extend_from_slice(&[0u8; 139]);
        buf.push(NODE_END);
        let result = Items::load_from_otb(&buf);
        let msg = result.expect_err("must err");
        assert!(msg.contains("VersionInfo size mismatch"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // Root attribute != Version: version block is skipped, default values
    // remain (major_version=0) so subsequent validation rejects the file.
    // -----------------------------------------------------------------------

    #[test]
    fn test_root_attr_not_version_skips_block_then_invalid_major() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // attr = 0x99 (unknown) — the `if attr == Version` block is skipped,
        // so major_version stays 0 which fails the major-version check.
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        buf.extend_from_slice(&0u32.to_le_bytes()); // flags
        buf.push(0x99); // unknown attr → block skipped
        buf.push(NODE_END);
        let result = Items::load_from_otb(&buf);
        let msg = result.expect_err("must err");
        assert!(msg.contains("incompatible major version 0"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // ItemGroup match — every C++ itemgroup_t variant has a parser arm.
    // The existing tests already cover Ground / Container / Splash / Door /
    // MagicField / Teleport / Podium.  These tests cover the remaining arms:
    // None, Weapon, Ammunition, Armor, Charges, Writeable, Key, Fluid,
    // Deprecated.
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_group_none_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::None as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(1).unwrap().group, ItemGroup::None);
    }

    #[test]
    fn test_item_group_weapon_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Weapon as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(1).unwrap().group, ItemGroup::Weapon);
    }

    #[test]
    fn test_item_group_ammunition_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Ammunition as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(
            registry.get_item_type(1).unwrap().group,
            ItemGroup::Ammunition
        );
    }

    #[test]
    fn test_item_group_armor_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Armor as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(1).unwrap().group, ItemGroup::Armor);
    }

    #[test]
    fn test_item_group_charges_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Charges as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(1).unwrap().group, ItemGroup::Charges);
    }

    #[test]
    fn test_item_group_writeable_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Writeable as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(
            registry.get_item_type(1).unwrap().group,
            ItemGroup::Writeable
        );
    }

    #[test]
    fn test_item_group_key_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Key as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(1).unwrap().group, ItemGroup::Key);
    }

    #[test]
    fn test_item_group_fluid_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Fluid as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(registry.get_item_type(1).unwrap().group, ItemGroup::Fluid);
    }

    #[test]
    fn test_item_group_deprecated_arm() {
        let node = OtbItemNode {
            server_id: 1,
            client_id: 100,
            group: ItemGroup::Deprecated as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        assert_eq!(
            registry.get_item_type(1).unwrap().group,
            ItemGroup::Deprecated
        );
    }

    // -----------------------------------------------------------------------
    // ITEM_ATTR_* wrong-length Err arms (every numeric attribute checks
    // datalen and Errs if it does not match).
    // -----------------------------------------------------------------------

    /// Hand-build an OTB blob with a single item containing exactly the given
    /// raw `extra_attr_bytes` after the 4-byte flags word.  Used to inject
    /// malformed attributes for negative tests.
    fn build_otb_with_raw_item(extra_attr_bytes: &[u8]) -> Vec<u8> {
        use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};

        let mut root_props: Vec<u8> = Vec::new();
        root_props.extend_from_slice(&0u32.to_le_bytes());
        root_props.push(0x01);
        root_props.extend_from_slice(&140u16.to_le_bytes());
        root_props.extend_from_slice(&3u32.to_le_bytes());
        root_props.extend_from_slice(&CLIENT_VERSION_LAST.to_le_bytes());
        root_props.extend_from_slice(&1u32.to_le_bytes());
        // CSD bytes 0..3 carry the three framing markers so the root-side
        // escape branch in this helper is exercised on every call.
        let mut csd = [0u8; 128];
        csd[0] = NODE_START;
        csd[1] = NODE_END;
        csd[2] = ESCAPE;
        root_props.extend_from_slice(&csd);

        let mut item_props: Vec<u8> = Vec::new();
        item_props.extend_from_slice(&0u32.to_le_bytes()); // flags
        item_props.extend_from_slice(extra_attr_bytes);

        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        for &b in &root_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        buf.push(NODE_START);
        buf.push(1u8); // Ground group
        for &b in &item_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        buf.push(NODE_END);
        buf.push(NODE_END);
        buf
    }

    #[test]
    fn test_client_id_attr_wrong_datalen_err() {
        use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};

        // ClientId(0x11) declared with datalen 3 — payload also contains the
        // three "special" framing bytes so the helper's escape branches are
        // exercised on the way out.
        let mut attrs = vec![];
        attrs.push(0x11);
        attrs.extend_from_slice(&3u16.to_le_bytes());
        attrs.push(NODE_START);
        attrs.push(NODE_END);
        attrs.push(ESCAPE);
        let buf = build_otb_with_raw_item(&attrs);
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("ClientId wrong length"), "msg: {}", msg);
    }

    #[test]
    fn test_speed_attr_wrong_datalen_err() {
        // Speed(0x14) declared with datalen 1
        let mut attrs = vec![];
        attrs.push(0x14);
        attrs.extend_from_slice(&1u16.to_le_bytes());
        attrs.push(0u8);
        let buf = build_otb_with_raw_item(&attrs);
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("Speed wrong length"), "msg: {}", msg);
    }

    #[test]
    fn test_light2_attr_wrong_datalen_err() {
        // Light2(0x2A) declared with datalen 3 (expected 4)
        let mut attrs = vec![];
        attrs.push(0x2A);
        attrs.extend_from_slice(&3u16.to_le_bytes());
        attrs.extend_from_slice(&[0u8; 3]);
        let buf = build_otb_with_raw_item(&attrs);
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("Light2 wrong length"), "msg: {}", msg);
    }

    #[test]
    fn test_top_order_attr_wrong_datalen_err() {
        // TopOrder(0x2B) declared with datalen 2 (expected 1)
        let mut attrs = vec![];
        attrs.push(0x2B);
        attrs.extend_from_slice(&2u16.to_le_bytes());
        attrs.extend_from_slice(&[0u8; 2]);
        let buf = build_otb_with_raw_item(&attrs);
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("TopOrder wrong length"), "msg: {}", msg);
    }

    #[test]
    fn test_ware_id_attr_wrong_datalen_err() {
        // WareId(0x2D) declared with datalen 4 (expected 2)
        let mut attrs = vec![];
        attrs.push(0x2D);
        attrs.extend_from_slice(&4u16.to_le_bytes());
        attrs.extend_from_slice(&[0u8; 4]);
        let buf = build_otb_with_raw_item(&attrs);
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("WareId wrong length"), "msg: {}", msg);
    }

    #[test]
    fn test_classification_attr_wrong_datalen_err() {
        // Classification(0x2E) declared with datalen 2 (expected 1)
        let mut attrs = vec![];
        attrs.push(0x2E);
        attrs.extend_from_slice(&2u16.to_le_bytes());
        attrs.extend_from_slice(&[0u8; 2]);
        let buf = build_otb_with_raw_item(&attrs);
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("Classification wrong length"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // Unknown attribute byte → skip path
    // -----------------------------------------------------------------------

    #[test]
    fn test_unknown_attr_is_skipped_when_well_formed() {
        // Unknown attr 0x77 with datalen 2 and 2 payload bytes → must skip
        // without error, then item is otherwise valid.
        let mut attrs = vec![];
        attrs.push(0x77); // unknown
        attrs.extend_from_slice(&2u16.to_le_bytes());
        attrs.extend_from_slice(&0u16.to_le_bytes());
        // Real server/client id so we can look it up
        attrs.push(0x10);
        attrs.extend_from_slice(&2u16.to_le_bytes());
        attrs.extend_from_slice(&55u16.to_le_bytes());
        attrs.push(0x11);
        attrs.extend_from_slice(&2u16.to_le_bytes());
        attrs.extend_from_slice(&55u16.to_le_bytes());
        let buf = build_otb_with_raw_item(&attrs);
        let registry = Items::load_from_otb(&buf).expect("unknown attr should be skipped");
        assert!(registry.get_item_type(55).is_some());
    }

    #[test]
    fn test_unknown_attr_truncated_payload_err() {
        // Unknown attr 0x77 with declared datalen 5 but only 2 payload bytes
        // present → skip(5) returns false → "truncated unknown attr" Err.
        let mut attrs = vec![];
        attrs.push(0x77);
        attrs.extend_from_slice(&5u16.to_le_bytes());
        attrs.extend_from_slice(&[0u8; 2]); // only 2, need 5
        let buf = build_otb_with_raw_item(&attrs);
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("truncated unknown attr"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // get_item_type_by_client_id: client_id in range but mapped to 0 → None
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_type_by_client_id_in_range_but_unmapped_returns_none() {
        // Item A has client_id 300 → client_to_server is resized to 301 entries,
        // with all slots 0 except index 300.  Querying client_id 150 → in range,
        // value is 0 → None.
        let node = OtbItemNode {
            server_id: 7,
            client_id: 300,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("parse ok");
        // client_id 150 is >= 100, within vec range, but maps to 0
        assert!(registry.get_item_type_by_client_id(150).is_none());
    }

    // -----------------------------------------------------------------------
    // parse_node_raw error paths (recursive node parser)
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_node_raw_truncation_inside_node_err() {
        use forgottenserver_common::fileloader::NODE_START;

        // Root node never closes — no NODE_END after the type byte.
        // 4-byte ident + NODE_START + type byte + some prop bytes (no NODE_END).
        // Must be >= 7 bytes to bypass the "OTB file too small" early gate.
        let buf = vec![
            0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("unexpected end inside node"), "msg: {}", msg);
    }

    #[test]
    fn test_parse_node_raw_truncation_after_escape_err() {
        use forgottenserver_common::fileloader::{ESCAPE, NODE_START};

        // 4-byte ident + NODE_START + type byte + ESCAPE + EOF
        let buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00, ESCAPE];
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(msg.contains("unexpected end after ESCAPE"), "msg: {}", msg);
    }

    #[test]
    fn test_parse_node_raw_nested_node_start_at_eof_err() {
        use forgottenserver_common::fileloader::NODE_START;

        // Root opens, contains a NODE_START with no type byte after it.
        // 4-byte ident + NODE_START + type byte + NODE_START + EOF
        let buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00, NODE_START];
        let msg = Items::load_from_otb(&buf).expect_err("must err");
        assert!(
            msg.contains("unexpected end (expected type byte)"),
            "msg: {}",
            msg
        );
    }

    // -----------------------------------------------------------------------
    // Test helper escape paths — `make_node` and `build_raw_otb` must escape
    // bytes equal to NODE_START / NODE_END / ESCAPE.  We trigger them by
    // declaring an item attribute whose payload contains these bytes.
    // -----------------------------------------------------------------------

    #[test]
    fn test_test_helper_escapes_special_bytes_in_root_and_item() {
        // Build a custom OTB blob where the root props' VersionInfo contains
        // a 128-byte CSD string filled with 0xFE (NODE_START) — this forces
        // the build_raw_otb test helper to emit ESCAPE bytes.  Then verify
        // the parser still extracts the original VersionInfo intact.
        //
        // We can't easily do this with the existing `build_otb` helper because
        // it only escapes via build_raw_otb / make_node; we must call them
        // explicitly through a fresh blob.

        use forgottenserver_common::fileloader::{ESCAPE, NODE_END, NODE_START};

        // Hand-build with bytes that must be escaped on the way out.
        let mut root_props: Vec<u8> = Vec::new();
        root_props.extend_from_slice(&0u32.to_le_bytes());
        root_props.push(0x01); // ROOT_ATTR_VERSION
        root_props.extend_from_slice(&140u16.to_le_bytes()); // datalen
        root_props.extend_from_slice(&3u32.to_le_bytes()); // major
        root_props.extend_from_slice(&CLIENT_VERSION_LAST.to_le_bytes()); // minor
        root_props.extend_from_slice(&1u32.to_le_bytes()); // build
                                                           // CSD string contains NODE_START / NODE_END / ESCAPE bytes
        let mut csd = vec![0u8; 128];
        csd[0] = NODE_START;
        csd[1] = NODE_END;
        csd[2] = ESCAPE;
        root_props.extend_from_slice(&csd);

        // Build a child item where attr payload contains all three special bytes.
        let mut item_props: Vec<u8> = Vec::new();
        item_props.extend_from_slice(&0u32.to_le_bytes()); // flags
                                                           // ServerId attr: payload is 2 bytes — set them to NODE_START/NODE_END
        item_props.push(0x10);
        item_props.extend_from_slice(&2u16.to_le_bytes());
        item_props.push(NODE_START);
        item_props.push(NODE_END);
        // ClientId 100 (so it's >= 100 and queryable)
        item_props.push(0x11);
        item_props.extend_from_slice(&2u16.to_le_bytes());
        item_props.extend_from_slice(&100u16.to_le_bytes());

        // Assemble: 4-byte ident + NODE_START + root_type + ESCAPED root_props + ESCAPED child node + NODE_END
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49, NODE_START, 0x00];
        for &b in &root_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        // child node
        buf.push(NODE_START);
        buf.push(1u8); // Ground
        for &b in &item_props {
            if b == NODE_START || b == NODE_END || b == ESCAPE {
                buf.push(ESCAPE);
            }
            buf.push(b);
        }
        buf.push(NODE_END);
        buf.push(NODE_END);

        // Parse must round-trip: server_id was 0xFFFE = NODE_END|NODE_START LE.
        let registry = Items::load_from_otb(&buf).expect("parse ok despite escapes");
        let expected_server_id = u16::from_le_bytes([NODE_START, NODE_END]);
        assert!(registry.get_item_type(expected_server_id).is_some());
        assert_eq!(registry.major_version, 3);
        assert_eq!(registry.minor_version, CLIENT_VERSION_LAST);
        assert_eq!(registry.build_number, 1);
    }

    // -----------------------------------------------------------------------
    // helpers::build_otb internal-escape branches — exercise the make_node /
    // build_raw_otb escape branches by emitting item attr payloads containing
    // 0xFD / 0xFE / 0xFF *via the test helper API*.
    //
    // We must call `build_otb` (the public helper) — that means we configure
    // an OtbItemNode whose numeric fields encode the special bytes when
    // serialised little-endian.  Using server_id = 0xFE_FE produces two
    // 0xFE bytes that must be escaped by make_node.  Using top_order = 0xFE
    // produces one escape byte in the TopOrder attr value.
    // -----------------------------------------------------------------------

    #[test]
    fn test_test_helper_make_node_escapes_via_helper_api() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        let node = OtbItemNode {
            server_id: u16::from_le_bytes([NODE_START, NODE_END]),
            client_id: u16::from_le_bytes([NODE_END, NODE_START]),
            group: ItemGroup::Ground as u8,
            top_order: NODE_START,    // 0xFE in TopOrder payload
            classification: NODE_END, // 0xFF in Classification payload
            ware_id: u16::from_le_bytes([NODE_START, NODE_END]),
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let registry = Items::load_from_otb(&otb).expect("escapes round-trip");
        let sid = u16::from_le_bytes([NODE_START, NODE_END]);
        let item = registry
            .get_item_type(sid)
            .expect("item with escaped server_id");
        assert_eq!(item.always_on_top_order, NODE_START);
        assert_eq!(item.classification, NODE_END);
        assert_eq!(item.ware_id, u16::from_le_bytes([NODE_START, NODE_END]));
    }

    // ========================================================================
    // XML LOADER TESTS (mirrors `items.cpp` Items::loadFromXml/parseItemNode)
    // ========================================================================

    /// Tiny helper: build a 1-item OTB containing `server_id`, then return the
    /// pre-populated registry.
    fn registry_with(server_id: u16, client_id: u16) -> Items {
        let node = OtbItemNode {
            server_id,
            client_id,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        Items::load_from_otb(&otb).expect("parse ok")
    }

    // ----- Maps -----

    #[test]
    fn test_item_parse_attributes_map_known_keys() {
        let m = item_parse_attributes_map();
        assert_eq!(m.get("weight"), Some(&ItemParseAttribute::Weight));
        assert_eq!(m.get("attack"), Some(&ItemParseAttribute::Attack));
        assert_eq!(m.get("worth"), Some(&ItemParseAttribute::Worth));
        assert_eq!(m.get("type"), Some(&ItemParseAttribute::Type));
        // movable is an alias of moveable
        assert_eq!(m.get("movable"), Some(&ItemParseAttribute::Moveable));
        // forcesave alias of forceserialize
        assert_eq!(
            m.get("forcesave"),
            Some(&ItemParseAttribute::ForceSerialize)
        );
        // allowpickupable alias of pickupable
        assert_eq!(
            m.get("allowpickupable"),
            Some(&ItemParseAttribute::Pickupable)
        );
        // magiclevelpoints alias of magicpoints
        assert_eq!(
            m.get("magiclevelpoints"),
            Some(&ItemParseAttribute::MagicPoints)
        );
        assert!(m.get("not-a-real-key").is_none());
    }

    #[test]
    fn test_item_types_map_lookup() {
        let m = item_types_map();
        assert_eq!(m.get("container"), Some(&ItemTypeKind::Container));
        assert_eq!(m.get("depot"), Some(&ItemTypeKind::Depot));
        assert_eq!(m.get("rune"), Some(&ItemTypeKind::Rune));
        assert!(m.get("nope").is_none());
    }

    #[test]
    fn test_tile_states_map_lookup() {
        let m = tile_states_map();
        assert_eq!(m.get("down"), Some(&tile_state_floor_change::DOWN));
        assert_eq!(m.get("southalt"), Some(&tile_state_floor_change::SOUTH_ALT));
        assert!(m.get("nope").is_none());
    }

    #[test]
    fn test_race_types_map_lookup() {
        let m = race_types_map();
        assert_eq!(m.get("undead"), Some(&RaceType::Undead));
        assert_eq!(m.get("fire"), Some(&RaceType::Fire));
        assert_eq!(m.get("ink"), Some(&RaceType::Ink));
        assert!(m.get("none").is_none());
    }

    #[test]
    fn test_weapon_types_map_lookup() {
        let m = weapon_types_map();
        assert_eq!(m.get("sword"), Some(&WeaponType::Sword));
        // "ammunition" maps to Ammo (renamed in Rust)
        assert_eq!(m.get("ammunition"), Some(&WeaponType::Ammo));
        assert_eq!(m.get("quiver"), Some(&WeaponType::Quiver));
        assert!(m.get("none").is_none());
    }

    #[test]
    fn test_fluid_types_map_lookup() {
        let m = fluid_types_map();
        assert_eq!(m.get("water"), Some(&FluidType::Water));
        assert_eq!(m.get("coconut"), Some(&FluidType::CoconutMilk));
        assert_eq!(m.get("fruitjuice"), Some(&FluidType::FruitJuice));
        assert!(m.get("nope").is_none());
    }

    #[test]
    fn test_directions_map_lookup() {
        let m = directions_map();
        assert_eq!(m.get("north"), Some(&Direction::North));
        assert_eq!(m.get("sw"), Some(&Direction::Southwest));
        assert_eq!(m.get("0"), Some(&Direction::North));
        assert_eq!(m.get("7"), Some(&Direction::Northeast));
        assert!(m.get("xx").is_none());
    }

    #[test]
    fn test_get_direction_known() {
        assert_eq!(get_direction("north"), Direction::North);
        assert_eq!(get_direction("se"), Direction::Southeast);
        assert_eq!(get_direction("5"), Direction::Southeast);
    }

    #[test]
    fn test_get_direction_unknown_defaults_to_north() {
        assert_eq!(get_direction("xyz"), Direction::North);
        assert_eq!(get_direction(""), Direction::North);
    }

    // ----- combat_type_to_index -----

    #[test]
    fn test_combat_type_to_index_basic() {
        assert_eq!(combat_type_to_index(CombatTypeFlags::PHYSICAL), 0);
        assert_eq!(combat_type_to_index(CombatTypeFlags::ENERGY), 1);
        assert_eq!(combat_type_to_index(CombatTypeFlags::EARTH), 2);
        assert_eq!(combat_type_to_index(CombatTypeFlags::FIRE), 3);
        assert_eq!(combat_type_to_index(CombatTypeFlags::DEATH), 11);
    }

    // ----- Abilities defaults -----

    #[test]
    fn test_abilities_default_values() {
        let a = Abilities::default();
        assert_eq!(a.health_gain, 0);
        assert_eq!(a.speed, 0);
        assert_eq!(a.element_type, CombatTypeFlags::NONE);
        assert!(!a.regeneration);
        assert!(!a.invisible);
        assert!(!a.mana_shield);
        assert_eq!(a.skills.len(), SKILL_COUNT);
        assert_eq!(a.stats.len(), STAT_COUNT);
        assert_eq!(a.absorb_percent.len(), COMBAT_COUNT);
    }

    // ----- ItemParseAttribute is Copy + Eq + Hash -----

    #[test]
    fn test_item_parse_attribute_traits() {
        let a = ItemParseAttribute::Attack;
        let b = a; // Copy
        assert_eq!(a, b);
        let mut h = HashMap::new();
        h.insert(ItemParseAttribute::Weight, 1);
        assert_eq!(h.get(&ItemParseAttribute::Weight), Some(&1));
    }

    // ----- OTBI constant -----

    #[test]
    fn test_otbi_constant() {
        assert_eq!(OTBI, *b"OTBI");
    }

    // ----- Items::clear -----

    #[test]
    fn test_clear_resets_registry() {
        let mut reg = registry_with(100, 200);
        reg.register_name("apple", 100);
        reg.currency_items.push((1, 100));

        reg.clear();
        assert!(reg.is_empty());
        assert!(reg.get_item_type(100).is_none());
        assert!(reg.get_item_type_by_name("apple").is_none());
        assert!(reg.currency_items.is_empty());
        assert_eq!(reg.major_version, 0);
        assert_eq!(reg.minor_version, 0);
        assert_eq!(reg.build_number, 0);
    }

    // ----- Items::reload -----

    #[test]
    fn test_reload_replaces_state() {
        // initial state — different from final
        let mut reg = registry_with(1, 100);
        reg.register_name("old", 1);

        // Build a different OTB
        let node = OtbItemNode {
            server_id: 5,
            client_id: 105,
            group: ItemGroup::Container as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let xml = r#"<items><item id="5" name="bag"/></items>"#;
        reg.reload(&otb, xml).expect("reload ok");

        // Old state is gone
        assert!(reg.get_item_type_by_name("old").is_none());
        // New state is present
        assert!(reg.get_item_type(5).is_some());
        assert_eq!(reg.get_item_type(5).unwrap().name, "bag");
        assert!(reg.get_item_type_by_name("bag").is_some());
    }

    // ----- Items::load_from_xml — error paths -----

    #[test]
    fn test_load_from_xml_invalid_xml_err() {
        let mut reg = Items::new();
        let err = reg
            .load_from_xml("<<not-xml>>")
            .expect_err("must err on bad xml");
        assert!(err.contains("XML parse error"));
    }

    #[test]
    fn test_load_from_xml_missing_root_err() {
        let mut reg = Items::new();
        let err = reg
            .load_from_xml("<other></other>")
            .expect_err("must err on missing <items>");
        assert!(err.contains("Missing <items>"));
    }

    #[test]
    fn test_load_from_xml_invalid_id_err() {
        let mut reg = Items::new();
        let err = reg
            .load_from_xml(r#"<items><item id="abc" name="x"/></items>"#)
            .expect_err("must err");
        assert!(err.contains("Invalid item id"));
    }

    #[test]
    fn test_load_from_xml_fromid_toid_invalid_err() {
        let mut reg = Items::new();
        let err = reg
            .load_from_xml(r#"<items><item fromid="abc" toid="3"/></items>"#)
            .expect_err("must err");
        assert!(err.contains("Invalid fromid"));

        let err2 = reg
            .load_from_xml(r#"<items><item fromid="2" toid="xx"/></items>"#)
            .expect_err("must err");
        assert!(err2.contains("Invalid toid"));
    }

    // ----- Items::load_from_xml — happy paths -----

    #[test]
    fn test_load_from_xml_sets_name_article_plural() {
        let mut reg = registry_with(100, 200);
        let xml = r#"<items><item id="100" article="a" plural="swords" name="sword"/></items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.name, "sword");
        assert_eq!(it.article, "a");
        assert_eq!(it.plural_name, "swords");
        // name lookup is case-insensitive
        assert!(reg.get_item_type_by_name("SWORD").is_some());
    }

    #[test]
    fn test_load_from_xml_fromid_toid_range() {
        // Pre-populate items 10..=12 so parseItemNode finds them.
        let nodes = vec![
            OtbItemNode {
                server_id: 10,
                client_id: 100,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 11,
                client_id: 101,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 12,
                client_id: 102,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items><item fromid="10" toid="12" name="thing"/></items>"#;
        reg.load_from_xml(xml).expect("ok");
        // First wins for name → only id 10 is mapped by name
        assert_eq!(reg.get_item_type_by_name("thing").unwrap().id, 10);
        for id in 10u16..=12 {
            assert_eq!(reg.get_item_type(id).unwrap().name, "thing");
        }
    }

    #[test]
    fn test_load_from_xml_skips_fromid_without_toid() {
        let mut reg = registry_with(50, 150);
        let xml = r#"<items><item fromid="50"/></items>"#;
        reg.load_from_xml(xml).expect("ok");
        // Should be skipped (no name set)
        assert!(reg.get_item_type(50).unwrap().name.is_empty());
    }

    #[test]
    fn test_load_from_xml_skips_missing_id_node() {
        let mut reg = registry_with(50, 150);
        let xml = r#"<items><item article="a"/></items>"#;
        reg.load_from_xml(xml).expect("ok");
    }

    // ----- parse_item_node: pre-allocation of ids 1..100 -----

    #[test]
    fn test_parse_item_node_preallocates_low_ids_grows_vec() {
        // Empty OTB → items.len() == 0. XML refers to id=42 (in 1..100), so
        // `ensure_item_slot` must grow the vec.
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        assert_eq!(reg.items.len(), 0);
        let xml = r#"<items><item id="42" name="rock"/></items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(42).unwrap();
        assert_eq!(it.id, 42);
        assert_eq!(it.name, "rock");
    }

    #[test]
    fn test_parse_item_node_preallocates_low_ids() {
        let mut reg = registry_with(100, 200);
        // id=42 is in the 1..100 range — should be pre-allocated.
        let xml = r#"<items><item id="42" name="rock"/></items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(42).unwrap();
        assert_eq!(it.id, 42);
        assert_eq!(it.name, "rock");
    }

    #[test]
    fn test_parse_item_node_skips_unknown_id() {
        let mut reg = registry_with(100, 200);
        // id=200 is NOT pre-allocated by OTB (only id=100 is).
        // Since 200 >= 100, parse_item_node won't pre-allocate; it should skip.
        let xml = r#"<items><item id="200" name="unknown"/></items>"#;
        reg.load_from_xml(xml).expect("ok");
        assert!(reg.get_item_type_by_name("unknown").is_none());
    }

    #[test]
    fn test_parse_item_node_skips_duplicate_name() {
        let mut reg = registry_with(100, 200);
        let xml = r#"<items>
            <item id="100" name="first"/>
            <item id="100" name="second"/>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.name, "first", "first wins; duplicate skipped");
    }

    // ----- parse_item_node: simple attribute handlers -----

    fn parse_one(reg: &mut Items, id: u16, xml_attrs: &str) {
        let xml = format!(r#"<items><item id="{id}" name="t">{xml_attrs}</item></items>"#);
        reg.load_from_xml(&xml).expect("ok");
    }

    #[test]
    fn test_attr_weight() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="weight" value="1234"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().weight, 1234);
    }

    #[test]
    fn test_attr_armor_defense_attack_extradef() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="armor" value="5"/>
               <attribute key="defense" value="6"/>
               <attribute key="extradef" value="7"/>
               <attribute key="attack" value="42"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.armor, 5);
        assert_eq!(it.defense, 6);
        assert_eq!(it.extra_defense, 7);
        assert_eq!(it.attack, 42);
    }

    #[test]
    fn test_attr_attackspeed_clamps_below_100() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="attackspeed" value="50"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().attack_speed, 100);
    }

    #[test]
    fn test_attr_attackspeed_zero_stays_zero() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="attackspeed" value="0"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().attack_speed, 0);
    }

    #[test]
    fn test_attr_attackspeed_above_100_kept() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="attackspeed" value="250"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().attack_speed, 250);
    }

    #[test]
    fn test_attr_type_container_sets_group() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="type" value="container"/>"#);
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::Container);
        assert_eq!(it.group, ItemGroup::Container);
    }

    #[test]
    fn test_attr_type_unknown_value_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="type" value="bogus"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().type_kind, ItemTypeKind::None);
    }

    #[test]
    fn test_attr_description_and_runespellname() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="description" value="A test"/>
               <attribute key="runespellname" value="exori"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.description, "A test");
        assert_eq!(it.rune_spell_name, "exori");
    }

    #[test]
    fn test_attr_floorchange_or_combines() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="floorchange" value="down"/>
               <attribute key="floorchange" value="south"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(
            it.floor_change,
            tile_state_floor_change::DOWN | tile_state_floor_change::SOUTH
        );
    }

    #[test]
    fn test_attr_floorchange_unknown_value_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="floorchange" value="diagonal"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().floor_change, 0);
    }

    #[test]
    fn test_attr_corpsetype() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="corpsetype" value="undead"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().corpse_type, RaceType::Undead);
    }

    #[test]
    fn test_attr_corpsetype_unknown_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="corpsetype" value="bogus"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().corpse_type, RaceType::None);
    }

    #[test]
    fn test_attr_containersize_overrides_default() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="containersize" value="20"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().max_items, 20);
    }

    #[test]
    fn test_attr_fluidsource() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="fluidsource" value="mana"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().fluid_source, FluidType::Mana);
    }

    #[test]
    fn test_attr_fluidsource_unknown_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="fluidsource" value="bogus"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().fluid_source, FluidType::None);
    }

    #[test]
    fn test_attr_readable_writeable() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="readable" value="1"/>
               <attribute key="writeable" value="1"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(it.can_read_text);
        assert!(it.can_write_text);
    }

    #[test]
    fn test_attr_maxtextlen_writeonceitemid() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="maxtextlen" value="64"/>
               <attribute key="writeonceitemid" value="9999"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.max_text_len, 64);
        assert_eq!(it.write_once_item_id, 9999);
    }

    #[test]
    fn test_attr_weapontype() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="weapontype" value="sword"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().weapon_type, WeaponType::Sword);
    }

    #[test]
    fn test_attr_weapontype_unknown_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="weapontype" value="garbage"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().weapon_type, WeaponType::None);
    }

    #[test]
    fn test_attr_slottype_all() {
        let mut r = registry_with(100, 200);
        // Start with default slot = HAND. Set head + body + legs.
        parse_one(
            &mut r,
            100,
            r#"<attribute key="slottype" value="head"/>
               <attribute key="slottype" value="body"/>
               <attribute key="slottype" value="legs"/>
               <attribute key="slottype" value="feet"/>
               <attribute key="slottype" value="backpack"/>
               <attribute key="slottype" value="two-handed"/>
               <attribute key="slottype" value="necklace"/>
               <attribute key="slottype" value="ring"/>
               <attribute key="slottype" value="ammo"/>
               <attribute key="slottype" value="hand"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(it.slot_position & slot_position::HEAD != 0);
        assert!(it.slot_position & slot_position::ARMOR != 0);
        assert!(it.slot_position & slot_position::LEGS != 0);
        assert!(it.slot_position & slot_position::FEET != 0);
        assert!(it.slot_position & slot_position::BACKPACK != 0);
        assert!(it.slot_position & slot_position::TWO_HAND != 0);
        assert!(it.slot_position & slot_position::NECKLACE != 0);
        assert!(it.slot_position & slot_position::RING != 0);
        assert!(it.slot_position & slot_position::AMMO != 0);
    }

    #[test]
    fn test_attr_slottype_right_hand_clears_left() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="slottype" value="right-hand"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(it.slot_position & slot_position::LEFT == 0);
        assert!(it.slot_position & slot_position::RIGHT != 0);
    }

    #[test]
    fn test_attr_slottype_left_hand_clears_right() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="slottype" value="left-hand"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(it.slot_position & slot_position::RIGHT == 0);
        assert!(it.slot_position & slot_position::LEFT != 0);
    }

    #[test]
    fn test_attr_slottype_unknown_ignored() {
        let mut r = registry_with(100, 200);
        let before = r.get_item_type(100).unwrap().slot_position;
        parse_one(&mut r, 100, r#"<attribute key="slottype" value="bogus"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().slot_position, before);
    }

    #[test]
    fn test_attr_ammotype() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="ammotype" value="arrow"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().ammo_type, Ammo::Arrow);
    }

    #[test]
    fn test_attr_ammotype_unknown_keeps_default() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="ammotype" value="bogus"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().ammo_type, Ammo::None);
    }

    #[test]
    fn test_attr_shoottype() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="shoottype" value="spear"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().shoot_type, ShootType::Spear);
    }

    #[test]
    fn test_attr_shoottype_unknown_keeps_default() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="shoottype" value="bogus"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().shoot_type, ShootType::None);
    }

    #[test]
    fn test_attr_effect_unknown_keeps_default() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="effect" value="bogus"/>"#);
        assert_eq!(
            r.get_item_type(100).unwrap().magic_effect,
            MagicEffectClass::None
        );
    }

    #[test]
    fn test_attr_effect_known() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="effect" value="redspark"/>"#);
        assert_eq!(
            r.get_item_type(100).unwrap().magic_effect,
            MagicEffectClass::DrawBlood
        );
    }

    #[test]
    fn test_attr_range_decayto_charges() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="range" value="4"/>
               <attribute key="decayto" value="50"/>
               <attribute key="charges" value="100"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.shoot_range, 4);
        assert_eq!(it.decay_to, 50);
        assert_eq!(it.charges, 100);
    }

    #[test]
    fn test_attr_duration_with_minmax() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="duration" minvalue="100" maxvalue="200"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.decay_time_min, 100);
        assert_eq!(it.decay_time_max, 200);
    }

    #[test]
    fn test_attr_duration_with_only_value() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="duration" value="500"/>"#);
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.decay_time_min, 500);
        assert_eq!(it.decay_time_max, 500);
    }

    #[test]
    fn test_attr_hitchance_clamps() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="hitchance" value="200"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().hit_chance, 100);

        let mut r2 = registry_with(100, 200);
        parse_one(&mut r2, 100, r#"<attribute key="hitchance" value="-150"/>"#);
        assert_eq!(r2.get_item_type(100).unwrap().hit_chance, -100);

        let mut r3 = registry_with(100, 200);
        parse_one(&mut r3, 100, r#"<attribute key="hitchance" value="50"/>"#);
        assert_eq!(r3.get_item_type(100).unwrap().hit_chance, 50);
    }

    #[test]
    fn test_attr_maxhitchance_clamps_to_100() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="maxhitchance" value="200"/>"#,
        );
        assert_eq!(r.get_item_type(100).unwrap().max_hit_chance, 100);
    }

    #[test]
    fn test_attr_invisible_speed() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="invisible" value="1"/>
               <attribute key="speed" value="20"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        assert!(a.invisible);
        assert_eq!(a.speed, 20);
    }

    #[test]
    fn test_attr_health_mana_regen_sets_flag() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="healthgain" value="5"/>
               <attribute key="healthticks" value="3"/>
               <attribute key="managain" value="4"/>
               <attribute key="manaticks" value="2"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        assert!(a.regeneration);
        assert_eq!(a.health_gain, 5);
        assert_eq!(a.health_ticks, 3);
        assert_eq!(a.mana_gain, 4);
        assert_eq!(a.mana_ticks, 2);
    }

    #[test]
    fn test_attr_manashield() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="manashield" value="1"/>"#);
        assert!(
            r.get_item_type(100)
                .unwrap()
                .abilities
                .as_ref()
                .unwrap()
                .mana_shield
        );
    }

    #[test]
    fn test_attr_skills() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="skillsword" value="3"/>
               <attribute key="skillaxe" value="4"/>
               <attribute key="skillclub" value="5"/>
               <attribute key="skilldist" value="6"/>
               <attribute key="skillfish" value="7"/>
               <attribute key="skillshield" value="8"/>
               <attribute key="skillfist" value="9"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(a.skills[Skill::Sword as usize], 3);
        assert_eq!(a.skills[Skill::Axe as usize], 4);
        assert_eq!(a.skills[Skill::Club as usize], 5);
        assert_eq!(a.skills[Skill::Distance as usize], 6);
        assert_eq!(a.skills[Skill::Fishing as usize], 7);
        assert_eq!(a.skills[Skill::Shield as usize], 8);
        assert_eq!(a.skills[Skill::Fist as usize], 9);
    }

    #[test]
    fn test_attr_special_skills() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="criticalhitchance" value="1"/>
               <attribute key="criticalhitamount" value="2"/>
               <attribute key="lifeleechchance" value="3"/>
               <attribute key="lifeleechamount" value="4"/>
               <attribute key="manaleechchance" value="5"/>
               <attribute key="manaleechamount" value="6"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.special_skills[SpecialSkill::CriticalHitChance as usize],
            1
        );
        assert_eq!(
            a.special_skills[SpecialSkill::CriticalHitAmount as usize],
            2
        );
        assert_eq!(a.special_skills[SpecialSkill::LifeLeechChance as usize], 3);
        assert_eq!(a.special_skills[SpecialSkill::LifeLeechAmount as usize], 4);
        assert_eq!(a.special_skills[SpecialSkill::ManaLeechChance as usize], 5);
        assert_eq!(a.special_skills[SpecialSkill::ManaLeechAmount as usize], 6);
    }

    // Tests targeting the `get_or_insert_with` closures inside the
    // ability-init helpers. Calling these helpers on an item with no
    // pre-existing `abilities` triggers the closure that allocates a fresh
    // `Abilities` box. We exercise each helper in isolation.

    #[test]
    fn test_attr_speed_alone_initializes_abilities() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="speed" value="20"/>"#);
        assert_eq!(
            r.get_item_type(100)
                .unwrap()
                .abilities
                .as_ref()
                .unwrap()
                .speed,
            20
        );
    }

    #[test]
    fn test_attr_healthticks_alone_initializes_abilities() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="healthticks" value="7"/>"#);
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        assert!(a.regeneration);
        assert_eq!(a.health_ticks, 7);
    }

    #[test]
    fn test_attr_managain_alone_initializes_abilities() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="managain" value="6"/>"#);
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        assert!(a.regeneration);
        assert_eq!(a.mana_gain, 6);
    }

    #[test]
    fn test_attr_manaticks_alone_initializes_abilities() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="manaticks" value="9"/>"#);
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        assert!(a.regeneration);
        assert_eq!(a.mana_ticks, 9);
    }

    #[test]
    fn test_set_stat_percent_alone_initializes_abilities() {
        // Only MaxHitPointsPercent → set_stat_percent closure runs.
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="maxhitpointspercent" value="25"/>"#,
        );
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        assert_eq!(a.stats_percent[Stat::MaxHitPoints as usize], 25);
    }

    #[test]
    fn test_attr_stats_percent_alone_for_each_stat() {
        for (key, stat) in [
            ("maxmanapointspercent", Stat::MaxManaPoints),
            ("magicpointspercent", Stat::MagicPoints),
        ] {
            let mut r = registry_with(100, 200);
            let xml = format!(r#"<attribute key="{key}" value="42"/>"#);
            parse_one(&mut r, 100, &xml);
            let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
            assert_eq!(a.stats_percent[stat as usize], 42, "{key}");
        }
    }

    #[test]
    fn test_attr_stats_and_stats_percent() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="maxhitpoints" value="10"/>
               <attribute key="maxhitpointspercent" value="15"/>
               <attribute key="maxmanapoints" value="20"/>
               <attribute key="maxmanapointspercent" value="25"/>
               <attribute key="magicpoints" value="30"/>
               <attribute key="magicpointspercent" value="35"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(a.stats[Stat::MaxHitPoints as usize], 10);
        assert_eq!(a.stats_percent[Stat::MaxHitPoints as usize], 15);
        assert_eq!(a.stats[Stat::MaxManaPoints as usize], 20);
        assert_eq!(a.stats_percent[Stat::MaxManaPoints as usize], 25);
        assert_eq!(a.stats[Stat::MagicPoints as usize], 30);
        assert_eq!(a.stats_percent[Stat::MagicPoints as usize], 35);
    }

    #[test]
    fn test_attr_field_absorb_percent() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="fieldabsorbpercentenergy" value="10"/>
               <attribute key="fieldabsorbpercentfire" value="20"/>
               <attribute key="fieldabsorbpercentpoison" value="30"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.field_absorb_percent[combat_type_to_index(CombatTypeFlags::ENERGY)],
            10
        );
        assert_eq!(
            a.field_absorb_percent[combat_type_to_index(CombatTypeFlags::FIRE)],
            20
        );
        assert_eq!(
            a.field_absorb_percent[combat_type_to_index(CombatTypeFlags::EARTH)],
            30
        );
    }

    #[test]
    fn test_attr_absorb_percent_individual() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="absorbpercentenergy" value="5"/>
               <attribute key="absorbpercentfire" value="6"/>
               <attribute key="absorbpercentpoison" value="7"/>
               <attribute key="absorbpercentice" value="8"/>
               <attribute key="absorbpercentholy" value="9"/>
               <attribute key="absorbpercentdeath" value="10"/>
               <attribute key="absorbpercentlifedrain" value="11"/>
               <attribute key="absorbpercentmanadrain" value="12"/>
               <attribute key="absorbpercentdrown" value="13"/>
               <attribute key="absorbpercentphysical" value="14"/>
               <attribute key="absorbpercenthealing" value="15"/>
               <attribute key="absorbpercentundefined" value="16"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        let at = |c| a.absorb_percent[combat_type_to_index(c)];
        assert_eq!(at(CombatTypeFlags::ENERGY), 5);
        assert_eq!(at(CombatTypeFlags::FIRE), 6);
        assert_eq!(at(CombatTypeFlags::EARTH), 7);
        assert_eq!(at(CombatTypeFlags::ICE), 8);
        assert_eq!(at(CombatTypeFlags::HOLY), 9);
        assert_eq!(at(CombatTypeFlags::DEATH), 10);
        assert_eq!(at(CombatTypeFlags::LIFEDRAIN), 11);
        assert_eq!(at(CombatTypeFlags::MANADRAIN), 12);
        assert_eq!(at(CombatTypeFlags::DROWN), 13);
        assert_eq!(at(CombatTypeFlags::PHYSICAL), 14);
        assert_eq!(at(CombatTypeFlags::HEALING), 15);
        assert_eq!(at(CombatTypeFlags::UNDEFINED), 16);
    }

    #[test]
    fn test_attr_absorb_percent_all() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="absorbpercentall" value="3"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        for slot in a.absorb_percent.iter() {
            assert_eq!(*slot, 3);
        }
    }

    #[test]
    fn test_attr_absorb_percent_elements() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="absorbpercentelements" value="4"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
        ] {
            assert_eq!(a.absorb_percent[combat_type_to_index(c)], 4);
        }
        // Physical not touched
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::PHYSICAL)],
            0
        );
    }

    #[test]
    fn test_attr_absorb_percent_magic() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="absorbpercentmagic" value="2"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
            CombatTypeFlags::HOLY,
            CombatTypeFlags::DEATH,
        ] {
            assert_eq!(a.absorb_percent[combat_type_to_index(c)], 2);
        }
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::PHYSICAL)],
            0
        );
    }

    #[test]
    fn test_attr_reflect_percent_individual() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="reflectpercentenergy" value="5"/>
               <attribute key="reflectpercentfire" value="6"/>
               <attribute key="reflectpercentearth" value="7"/>
               <attribute key="reflectpercentice" value="8"/>
               <attribute key="reflectpercentholy" value="9"/>
               <attribute key="reflectpercentdeath" value="10"/>
               <attribute key="reflectpercentlifedrain" value="11"/>
               <attribute key="reflectpercentmanadrain" value="12"/>
               <attribute key="reflectpercentdrown" value="13"/>
               <attribute key="reflectpercentphysical" value="14"/>
               <attribute key="reflectpercenthealing" value="15"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        let at = |c| a.reflect[combat_type_to_index(c)].percent;
        assert_eq!(at(CombatTypeFlags::ENERGY), 5);
        assert_eq!(at(CombatTypeFlags::FIRE), 6);
        assert_eq!(at(CombatTypeFlags::EARTH), 7);
        assert_eq!(at(CombatTypeFlags::ICE), 8);
        assert_eq!(at(CombatTypeFlags::HOLY), 9);
        assert_eq!(at(CombatTypeFlags::DEATH), 10);
        assert_eq!(at(CombatTypeFlags::LIFEDRAIN), 11);
        assert_eq!(at(CombatTypeFlags::MANADRAIN), 12);
        assert_eq!(at(CombatTypeFlags::DROWN), 13);
        assert_eq!(at(CombatTypeFlags::PHYSICAL), 14);
        assert_eq!(at(CombatTypeFlags::HEALING), 15);
    }

    #[test]
    fn test_attr_reflect_percent_all_elements_magic() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="reflectpercentall" value="1"/>"#,
        );
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for slot in a.reflect.iter() {
            assert_eq!(slot.percent, 1);
        }

        let mut r2 = registry_with(100, 200);
        parse_one(
            &mut r2,
            100,
            r#"<attribute key="reflectpercentelements" value="2"/>"#,
        );
        let a2 = r2.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
        ] {
            assert_eq!(a2.reflect[combat_type_to_index(c)].percent, 2);
        }

        let mut r3 = registry_with(100, 200);
        parse_one(
            &mut r3,
            100,
            r#"<attribute key="reflectpercentmagic" value="3"/>"#,
        );
        let a3 = r3.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
            CombatTypeFlags::HOLY,
            CombatTypeFlags::DEATH,
        ] {
            assert_eq!(a3.reflect[combat_type_to_index(c)].percent, 3);
        }
    }

    #[test]
    fn test_attr_reflect_chance_individual() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="reflectchanceenergy" value="5"/>
               <attribute key="reflectchancefire" value="6"/>
               <attribute key="reflectchanceearth" value="7"/>
               <attribute key="reflectchanceice" value="8"/>
               <attribute key="reflectchanceholy" value="9"/>
               <attribute key="reflectchancedeath" value="10"/>
               <attribute key="reflectchancelifedrain" value="11"/>
               <attribute key="reflectchancemanadrain" value="12"/>
               <attribute key="reflectchancedrown" value="13"/>
               <attribute key="reflectchancephysical" value="14"/>
               <attribute key="reflectchancehealing" value="15"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let a = it.abilities.as_ref().unwrap();
        let at = |c| a.reflect[combat_type_to_index(c)].chance;
        assert_eq!(at(CombatTypeFlags::ENERGY), 5);
        assert_eq!(at(CombatTypeFlags::FIRE), 6);
        assert_eq!(at(CombatTypeFlags::EARTH), 7);
        assert_eq!(at(CombatTypeFlags::ICE), 8);
        assert_eq!(at(CombatTypeFlags::HOLY), 9);
        assert_eq!(at(CombatTypeFlags::DEATH), 10);
        assert_eq!(at(CombatTypeFlags::LIFEDRAIN), 11);
        assert_eq!(at(CombatTypeFlags::MANADRAIN), 12);
        assert_eq!(at(CombatTypeFlags::DROWN), 13);
        assert_eq!(at(CombatTypeFlags::PHYSICAL), 14);
        assert_eq!(at(CombatTypeFlags::HEALING), 15);
    }

    #[test]
    fn test_attr_reflect_chance_all_elements_magic() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="reflectchanceall" value="1"/>"#,
        );
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for slot in a.reflect.iter() {
            assert_eq!(slot.chance, 1);
        }

        let mut r2 = registry_with(100, 200);
        parse_one(
            &mut r2,
            100,
            r#"<attribute key="reflectchanceelements" value="2"/>"#,
        );
        let a2 = r2.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
        ] {
            assert_eq!(a2.reflect[combat_type_to_index(c)].chance, 2);
        }

        let mut r3 = registry_with(100, 200);
        parse_one(
            &mut r3,
            100,
            r#"<attribute key="reflectchancemagic" value="3"/>"#,
        );
        let a3 = r3.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
            CombatTypeFlags::HOLY,
            CombatTypeFlags::DEATH,
        ] {
            assert_eq!(a3.reflect[combat_type_to_index(c)].chance, 3);
        }
    }

    #[test]
    fn test_attr_boost_percent_individual() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="boostpercentenergy" value="5"/>
               <attribute key="boostpercentfire" value="6"/>
               <attribute key="boostpercentearth" value="7"/>
               <attribute key="boostpercentice" value="8"/>
               <attribute key="boostpercentholy" value="9"/>
               <attribute key="boostpercentdeath" value="10"/>
               <attribute key="boostpercentlifedrain" value="11"/>
               <attribute key="boostpercentmanadrain" value="12"/>
               <attribute key="boostpercentdrown" value="13"/>
               <attribute key="boostpercentphysical" value="14"/>
               <attribute key="boostpercenthealing" value="15"/>"#,
        );
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        let at = |c| a.boost_percent[combat_type_to_index(c)];
        assert_eq!(at(CombatTypeFlags::ENERGY), 5);
        assert_eq!(at(CombatTypeFlags::FIRE), 6);
        assert_eq!(at(CombatTypeFlags::EARTH), 7);
        assert_eq!(at(CombatTypeFlags::ICE), 8);
        assert_eq!(at(CombatTypeFlags::HOLY), 9);
        assert_eq!(at(CombatTypeFlags::DEATH), 10);
        assert_eq!(at(CombatTypeFlags::LIFEDRAIN), 11);
        assert_eq!(at(CombatTypeFlags::MANADRAIN), 12);
        assert_eq!(at(CombatTypeFlags::DROWN), 13);
        assert_eq!(at(CombatTypeFlags::PHYSICAL), 14);
        assert_eq!(at(CombatTypeFlags::HEALING), 15);
    }

    #[test]
    fn test_attr_boost_percent_all_elements_magic() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="boostpercentall" value="1"/>"#,
        );
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for slot in a.boost_percent.iter() {
            assert_eq!(*slot, 1);
        }

        let mut r2 = registry_with(100, 200);
        parse_one(
            &mut r2,
            100,
            r#"<attribute key="boostpercentelements" value="2"/>"#,
        );
        let a2 = r2.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
        ] {
            assert_eq!(a2.boost_percent[combat_type_to_index(c)], 2);
        }

        let mut r3 = registry_with(100, 200);
        parse_one(
            &mut r3,
            100,
            r#"<attribute key="boostpercentmagic" value="3"/>"#,
        );
        let a3 = r3.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
            CombatTypeFlags::HOLY,
            CombatTypeFlags::DEATH,
        ] {
            assert_eq!(a3.boost_percent[combat_type_to_index(c)], 3);
        }
    }

    #[test]
    fn test_attr_magic_level_per_element() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="magiclevelenergy" value="5"/>
               <attribute key="magiclevelfire" value="6"/>
               <attribute key="magiclevelpoison" value="7"/>
               <attribute key="magiclevelice" value="8"/>
               <attribute key="magiclevelholy" value="9"/>
               <attribute key="magicleveldeath" value="10"/>
               <attribute key="magiclevellifedrain" value="11"/>
               <attribute key="magiclevelmanadrain" value="12"/>
               <attribute key="magicleveldrown" value="13"/>
               <attribute key="magiclevelphysical" value="14"/>
               <attribute key="magiclevelhealing" value="15"/>
               <attribute key="magiclevelundefined" value="16"/>"#,
        );
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        let at = |c| a.special_magic_level_skill[combat_type_to_index(c)];
        assert_eq!(at(CombatTypeFlags::ENERGY), 5);
        assert_eq!(at(CombatTypeFlags::FIRE), 6);
        assert_eq!(at(CombatTypeFlags::EARTH), 7);
        assert_eq!(at(CombatTypeFlags::ICE), 8);
        assert_eq!(at(CombatTypeFlags::HOLY), 9);
        assert_eq!(at(CombatTypeFlags::DEATH), 10);
        assert_eq!(at(CombatTypeFlags::LIFEDRAIN), 11);
        assert_eq!(at(CombatTypeFlags::MANADRAIN), 12);
        assert_eq!(at(CombatTypeFlags::DROWN), 13);
        assert_eq!(at(CombatTypeFlags::PHYSICAL), 14);
        assert_eq!(at(CombatTypeFlags::HEALING), 15);
        assert_eq!(at(CombatTypeFlags::UNDEFINED), 16);
    }

    #[test]
    fn test_attr_suppress_conditions() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="suppressdrunk" value="1"/>
               <attribute key="suppressenergy" value="1"/>
               <attribute key="suppressfire" value="1"/>
               <attribute key="suppresspoison" value="1"/>
               <attribute key="suppressdrown" value="1"/>
               <attribute key="suppressphysical" value="1"/>
               <attribute key="suppressfreeze" value="1"/>
               <attribute key="suppressdazzle" value="1"/>
               <attribute key="suppresscurse" value="1"/>"#,
        );
        let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
        let s = a.condition_suppressions;
        assert!(s & ConditionTypeFlags::DRUNK as u32 != 0);
        assert!(s & ConditionTypeFlags::ENERGY as u32 != 0);
        assert!(s & ConditionTypeFlags::FIRE as u32 != 0);
        assert!(s & ConditionTypeFlags::POISON as u32 != 0);
        assert!(s & ConditionTypeFlags::DROWN as u32 != 0);
        assert!(s & ConditionTypeFlags::BLEEDING as u32 != 0);
        assert!(s & ConditionTypeFlags::FREEZING as u32 != 0);
        assert!(s & ConditionTypeFlags::DAZZLED as u32 != 0);
        assert!(s & ConditionTypeFlags::CURSED as u32 != 0);
    }

    #[test]
    fn test_attr_suppress_false_does_nothing() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="suppressfire" value="0"/>"#);
        // Abilities may not even be created since on=false short-circuits.
        let it = r.get_item_type(100).unwrap();
        let supp = it
            .abilities
            .as_ref()
            .map(|a| a.condition_suppressions)
            .unwrap_or(0);
        assert_eq!(supp & ConditionTypeFlags::FIRE as u32, 0);
    }

    #[test]
    fn test_attr_field_fire() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="field" value="fire"/>"#);
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.group, ItemGroup::MagicField);
        assert_eq!(it.type_kind, ItemTypeKind::MagicField);
        assert_eq!(it.combat_type, CombatTypeFlags::FIRE);
    }

    #[test]
    fn test_attr_field_energy_poison_drown_physical() {
        for (val, expected) in [
            ("energy", CombatTypeFlags::ENERGY),
            ("poison", CombatTypeFlags::EARTH),
            ("drown", CombatTypeFlags::DROWN),
            ("physical", CombatTypeFlags::PHYSICAL),
        ] {
            let mut r = registry_with(100, 200);
            let xml = format!(r#"<attribute key="field" value="{val}"/>"#);
            parse_one(&mut r, 100, &xml);
            let it = r.get_item_type(100).unwrap();
            assert_eq!(it.combat_type, expected, "field {val}");
            assert_eq!(it.type_kind, ItemTypeKind::MagicField);
        }
    }

    #[test]
    fn test_attr_field_fire_with_sub_children_builds_condition_damage() {
        use forgottenserver_common::enums::{ConditionParam, ConditionTypeFlags};
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="field" value="fire">
                  <attribute key="ticks" value="9000"/>
                  <attribute key="count" value="5"/>
                  <attribute key="damage" value="10"/>
                </attribute>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let cd = it.condition_damage.as_ref().expect("condition_damage set");
        assert_eq!(cd.base.condition_type, ConditionTypeFlags::FIRE);
        assert_eq!(cd.damage_list.len(), 5, "5 rounds scheduled");
        for d in &cd.damage_list {
            assert_eq!(d.value, -10, "each tick deals -10 damage");
            assert_eq!(d.interval, 9000, "tick interval matches ticks attr");
        }
        // Field param set + ForceUpdate because total damage > 0
        assert_eq!(cd.get_param(ConditionParam::Field), 1);
        assert_eq!(cd.get_param(ConditionParam::ForceUpdate), 1);
    }

    #[test]
    fn test_attr_field_uses_generate_damage_list_when_start_present() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="field" value="poison">
                  <attribute key="ticks" value="4000"/>
                  <attribute key="start" value="5"/>
                  <attribute key="damage" value="2"/>
                </attribute>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let cd = it.condition_damage.as_ref().expect("condition_damage set");
        // start>0 path: damage list filled from generate_damage_list with
        // decaying values; non-empty + all negative (because we negate before
        // calling add_damage(1, ticks, -damageValue))
        assert!(!cd.damage_list.is_empty(), "decay schedule non-empty");
        for d in &cd.damage_list {
            assert!(d.value < 0, "all scheduled values must be negative");
            assert_eq!(d.interval, 4000);
        }
    }

    #[test]
    fn test_attr_field_init_damage_overrides_with_negated_value() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="field" value="energy">
                  <attribute key="ticks" value="2000"/>
                  <attribute key="count" value="1"/>
                  <attribute key="damage" value="7"/>
                  <attribute key="initdamage" value="3"/>
                </attribute>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let cd = it.condition_damage.as_ref().expect("condition_damage set");
        // initDamage > 0 → setInitDamage(-initDamage)
        assert_eq!(cd.init_damage, -3);
    }

    #[test]
    fn test_generate_damage_list_decays_to_zero() {
        use crate::condition::ConditionDamage;
        let mut list: Vec<i32> = Vec::new();
        ConditionDamage::generate_damage_list(20, 10, &mut list);
        assert!(!list.is_empty(), "schedule must produce entries");
        // The schedule must end with `1` (the C++ loop pushes `i` until i = 1).
        assert_eq!(list.last(), Some(&1));
    }

    #[test]
    fn test_generate_damage_list_zero_amount_yields_empty() {
        use crate::condition::ConditionDamage;
        let mut list: Vec<i32> = Vec::new();
        ConditionDamage::generate_damage_list(0, 10, &mut list);
        assert!(list.is_empty());
    }

    #[test]
    fn test_attr_field_unknown_value_no_change() {
        let mut r = registry_with(100, 200);
        let before_group = r.get_item_type(100).unwrap().group;
        parse_one(&mut r, 100, r#"<attribute key="field" value="laser"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().group, before_group);
        assert_eq!(
            r.get_item_type(100).unwrap().combat_type,
            CombatTypeFlags::NONE
        );
    }

    #[test]
    fn test_attr_replaceable_partnerdirection_leveldoor() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="replaceable" value="0"/>
               <attribute key="partnerdirection" value="east"/>
               <attribute key="leveldoor" value="50"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(!it.replaceable);
        assert_eq!(it.bed_partner_dir, Direction::East);
        assert_eq!(it.level_door, 50);
    }

    #[test]
    fn test_attr_transformto_destroyto() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="transformto" value="105"/>
               <attribute key="destroyto" value="200"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.transform_to_free, 105);
        assert_eq!(it.destroy_to, 200);
    }

    #[test]
    fn test_attr_male_female_transformto() {
        // Need second item id to be a target. Pre-populate both 100 and 105.
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 105,
                client_id: 205,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="malebed">
                <attribute key="maletransformto" value="105"/>
            </item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.transform_to_on_use[PlayerSex::Male as usize], 105);
        // Female mirrors male because female was 0
        assert_eq!(it.transform_to_on_use[PlayerSex::Female as usize], 105);
        // The "other" item gets transform_to_free pointing back
        assert_eq!(reg.get_item_type(105).unwrap().transform_to_free, 100);
    }

    #[test]
    fn test_attr_female_transformto_mirrors_male() {
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 105,
                client_id: 205,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="femalebed">
                <attribute key="femaletransformto" value="105"/>
            </item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.transform_to_on_use[PlayerSex::Female as usize], 105);
        // Male mirrors female because male was 0
        assert_eq!(it.transform_to_on_use[PlayerSex::Male as usize], 105);
        assert_eq!(reg.get_item_type(105).unwrap().transform_to_free, 100);
    }

    #[test]
    fn test_attr_elements() {
        for (key, expected) in [
            ("elementice", CombatTypeFlags::ICE),
            ("elementearth", CombatTypeFlags::EARTH),
            ("elementfire", CombatTypeFlags::FIRE),
            ("elementenergy", CombatTypeFlags::ENERGY),
            ("elementdeath", CombatTypeFlags::DEATH),
            ("elementholy", CombatTypeFlags::HOLY),
        ] {
            let mut r = registry_with(100, 200);
            let xml = format!(r#"<attribute key="{key}" value="50"/>"#);
            parse_one(&mut r, 100, &xml);
            let a = r.get_item_type(100).unwrap().abilities.as_ref().unwrap();
            assert_eq!(a.element_damage, 50, "{key}");
            assert_eq!(a.element_type, expected, "{key}");
        }
    }

    #[test]
    fn test_attr_walkstack_blocking_allowdistread_storeitem() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="walkstack" value="0"/>
               <attribute key="blocking" value="1"/>
               <attribute key="allowdistread" value="yes"/>
               <attribute key="storeitem" value="yes"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(!it.walk_stack);
        assert!(it.block_solid);
        assert!(it.allow_dist_read);
        assert!(it.store_item);
    }

    #[test]
    fn test_attr_rotateto_decayto_showduration_showcharges_showattributes() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="rotateto" value="123"/>
               <attribute key="decayto" value="55"/>
               <attribute key="showduration" value="1"/>
               <attribute key="showcharges" value="1"/>
               <attribute key="showattributes" value="1"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.rotate_to, 123);
        assert_eq!(it.decay_to, 55);
        assert!(it.show_duration);
        assert!(it.show_charges);
        assert!(it.show_attributes);
    }

    #[test]
    fn test_attr_showcount_and_supply() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="showcount" value="0"/>
               <attribute key="supply" value="1"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(!it.show_count);
        assert!(it.supply);
    }

    #[test]
    fn test_attr_moveable_blockprojectile_pickupable_forceserialize_stopduration() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="moveable" value="1"/>
               <attribute key="blockprojectile" value="1"/>
               <attribute key="pickupable" value="1"/>
               <attribute key="forceserialize" value="1"/>
               <attribute key="stopduration" value="1"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert!(it.moveable);
        assert!(it.block_projectile);
        assert!(it.allow_pickupable);
        assert!(it.force_serialize);
        assert!(it.stop_time);
    }

    #[test]
    fn test_attr_transformequipto_deequipto() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="transformequipto" value="200"/>
               <attribute key="transformdeequipto" value="300"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.transform_equip_to, 200);
        assert_eq!(it.transform_de_equip_to, 300);
    }

    #[test]
    fn test_attr_worth_adds_to_currency_items() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="worth" value="100"/>"#);
        assert_eq!(r.currency_items, vec![(100u64, 100u16)]);
        assert_eq!(r.get_item_type(100).unwrap().worth, 100);
    }

    #[test]
    fn test_attr_worth_duplicate_ignored() {
        // Two items both claim worth=100; second is silently dropped.
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 101,
                client_id: 201,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="goldcoin"><attribute key="worth" value="1"/></item>
            <item id="101" name="goldcoin2"><attribute key="worth" value="1"/></item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        // Only the first registered.
        assert_eq!(reg.currency_items.len(), 1);
        assert_eq!(reg.currency_items[0], (1u64, 100u16));
        assert_eq!(reg.get_item_type(101).unwrap().worth, 0);
    }

    #[test]
    fn test_currency_items_sorted_descending() {
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 101,
                client_id: 201,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 102,
                client_id: 202,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="a"><attribute key="worth" value="1"/></item>
            <item id="101" name="b"><attribute key="worth" value="10000"/></item>
            <item id="102" name="c"><attribute key="worth" value="100"/></item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        assert_eq!(
            reg.currency_items,
            vec![(10000u64, 101u16), (100u64, 102u16), (1u64, 100u16)]
        );
    }

    #[test]
    fn test_unknown_attribute_key_skipped() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="totallyunknown" value="42"/>"#,
        );
        // No panic, item still exists, no fields touched.
        assert_eq!(r.get_item_type(100).unwrap().weight, 0);
    }

    #[test]
    fn test_attribute_without_key_skipped() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute value="42"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().weight, 0);
    }

    #[test]
    fn test_attribute_without_value_or_minmax_skipped() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="weight"/>"#);
        assert_eq!(r.get_item_type(100).unwrap().weight, 0);
    }

    #[test]
    fn test_attribute_minvalue_without_maxvalue_skipped() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="duration" minvalue="100"/>"#);
        // Skipped → decay_time_min stays 0
        assert_eq!(r.get_item_type(100).unwrap().decay_time_min, 0);
    }

    #[test]
    fn test_unknown_id_xml_skipped_no_panic() {
        // id=300 is NOT pre-allocated. Should skip with no name set.
        let mut r = registry_with(100, 200);
        let xml = r#"<items><item id="9999" name="ghost"/></items>"#;
        r.load_from_xml(xml).expect("ok");
        assert!(r.get_item_type(9999).is_none());
        assert!(r.get_item_type_by_name("ghost").is_none());
    }

    // ----- Items::load_from_xml empty doc -----

    #[test]
    fn test_load_from_xml_empty_items() {
        let mut r = Items::new();
        r.load_from_xml("<items></items>").expect("ok");
        assert_eq!(r.len(), 0);
    }

    // ----- Edge cases: u16::MAX wraparound -----

    #[test]
    fn test_load_from_xml_fromid_toid_includes_u16_max() {
        // Pre-allocate u16::MAX so parse_item_node sees an existing slot.
        let nodes = vec![OtbItemNode {
            server_id: u16::MAX,
            client_id: 200,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        }];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = format!(
            r#"<items><item fromid="{0}" toid="{0}" name="edge"/></items>"#,
            u16::MAX
        );
        reg.load_from_xml(&xml).expect("ok");
        assert_eq!(reg.get_item_type(u16::MAX).unwrap().name, "edge");
    }

    // ----- Male/Female transform: target already has transform_to_free set -----

    #[test]
    fn test_male_transformto_does_not_overwrite_existing_transform_to_free() {
        // 105's transform_to_free will be set by item 100; then item 200 also
        // declares maletransformto=105 → 105.transform_to_free must NOT be
        // overwritten (first wins).
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 105,
                client_id: 205,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 200,
                client_id: 300,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="bed1"><attribute key="maletransformto" value="105"/></item>
            <item id="200" name="bed2"><attribute key="maletransformto" value="105"/></item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        // First registrar wins.
        assert_eq!(reg.get_item_type(105).unwrap().transform_to_free, 100);
    }

    #[test]
    fn test_female_transformto_does_not_overwrite_existing_transform_to_free() {
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 105,
                client_id: 205,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 200,
                client_id: 300,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="bed1"><attribute key="femaletransformto" value="105"/></item>
            <item id="200" name="bed2"><attribute key="femaletransformto" value="105"/></item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        assert_eq!(reg.get_item_type(105).unwrap().transform_to_free, 100);
    }

    #[test]
    fn test_male_transformto_skips_if_female_already_set() {
        // If female slot is already set when male is being set, female is NOT
        // overwritten (covers `if female == 0` branch where female != 0).
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 105,
                client_id: 205,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 106,
                client_id: 206,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="bed">
                <attribute key="femaletransformto" value="106"/>
                <attribute key="maletransformto" value="105"/>
            </item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        // Male was set; female was already 106 → kept.
        assert_eq!(it.transform_to_on_use[PlayerSex::Male as usize], 105);
        assert_eq!(it.transform_to_on_use[PlayerSex::Female as usize], 106);
    }

    #[test]
    fn test_female_transformto_skips_if_male_already_set() {
        let nodes = vec![
            OtbItemNode {
                server_id: 100,
                client_id: 200,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 105,
                client_id: 205,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 106,
                client_id: 206,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let mut reg = Items::load_from_otb(&otb).expect("ok");
        let xml = r#"<items>
            <item id="100" name="bed">
                <attribute key="maletransformto" value="106"/>
                <attribute key="femaletransformto" value="105"/>
            </item>
        </items>"#;
        reg.load_from_xml(xml).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.transform_to_on_use[PlayerSex::Female as usize], 105);
        assert_eq!(it.transform_to_on_use[PlayerSex::Male as usize], 106);
    }

    // ----- Worth: invalid number (parse fail) -----

    #[test]
    fn test_attr_worth_invalid_number_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="worth" value="abc"/>"#);
        assert!(r.currency_items.is_empty());
    }

    // ----- Male/female transform: invalid number -----

    #[test]
    fn test_maletransformto_invalid_number_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="maletransformto" value="xx"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.transform_to_on_use[PlayerSex::Male as usize], 0);
    }

    #[test]
    fn test_femaletransformto_invalid_number_ignored() {
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="femaletransformto" value="xx"/>"#,
        );
        let it = r.get_item_type(100).unwrap();
        assert_eq!(it.transform_to_on_use[PlayerSex::Female as usize], 0);
    }

    // ----- apply_simple_attribute: multi-step variants are no-ops when called
    //       directly. They are intercepted by apply_attribute under normal
    //       flow, but the dispatcher must still exhaustively match them.
    // ------------------------------------------------------------------------

    #[test]
    fn test_apply_simple_attribute_multistep_variants_are_no_ops() {
        // Build a dummy XML doc just to obtain a `roxmltree::Node` for the
        // attr_node parameter (its content doesn't matter — these arms ignore
        // it).
        let doc = roxmltree::Document::parse("<root><x/></root>").unwrap();
        let attr_node = doc.descendants().find(|n| n.has_tag_name("x")).unwrap();

        for variant in [
            ItemParseAttribute::Field,
            ItemParseAttribute::MaleTransformTo,
            ItemParseAttribute::FemaleTransformTo,
            ItemParseAttribute::Worth,
        ] {
            let mut it = ItemTypeData::default();
            it.id = 100;
            let before = it.clone();
            super::apply_simple_attribute(&mut it, variant, "123", None, &attr_node);
            // None of the multi-step arms should touch the item.
            assert_eq!(it.id, before.id);
            assert_eq!(it.name, before.name);
            assert_eq!(it.worth, before.worth);
            assert_eq!(it.combat_type, before.combat_type);
            assert!(it.abilities.is_none());
        }
    }

    // ----- apply_attribute_field: sub-node edge cases -----

    #[test]
    fn test_field_sub_node_missing_key_skipped() {
        // Sub-node has no "key" attribute — the None => continue branch
        let mut r = registry_with(106, 206);
        let xml = r#"<items>
            <item id="106">
                <attribute key="field" value="fire">
                    <attribute value="9000"/>
                </attribute>
            </item>
        </items>"#;
        r.load_from_xml(xml).expect("ok");
        let it = r.get_item_type(106).unwrap();
        assert_eq!(it.group, ItemGroup::MagicField);
        assert_eq!(it.combat_type, CombatTypeFlags::FIRE);
        // No damage was added (sub-node skipped)
        let cd = it.condition_damage.as_ref().expect("cd set");
        assert!(cd.damage_list.is_empty());
    }

    #[test]
    fn test_field_sub_node_missing_value_skipped() {
        // Sub-node has "key" but no "value" — the None => continue branch
        let mut r = registry_with(107, 207);
        let xml = r#"<items>
            <item id="107">
                <attribute key="field" value="energy">
                    <attribute key="ticks"/>
                </attribute>
            </item>
        </items>"#;
        r.load_from_xml(xml).expect("ok");
        let it = r.get_item_type(107).unwrap();
        assert_eq!(it.group, ItemGroup::MagicField);
        assert_eq!(it.combat_type, CombatTypeFlags::ENERGY);
        let cd = it.condition_damage.as_ref().expect("cd set");
        assert!(cd.damage_list.is_empty());
    }

    #[test]
    fn test_field_sub_node_unknown_key_ignored() {
        // Sub-node has an unrecognised key — hits the `_ => {}` arm
        let mut r = registry_with(108, 208);
        let xml = r#"<items>
            <item id="108">
                <attribute key="field" value="fire">
                    <attribute key="unknownsubkey" value="42"/>
                    <attribute key="ticks" value="500"/>
                    <attribute key="count" value="2"/>
                    <attribute key="damage" value="6"/>
                </attribute>
            </item>
        </items>"#;
        r.load_from_xml(xml).expect("ok");
        let it = r.get_item_type(108).unwrap();
        assert_eq!(it.group, ItemGroup::MagicField);
        let cd = it.condition_damage.as_ref().expect("cd set");
        // Only the known sub-attrs contribute; unknown one was silently skipped
        assert_eq!(cd.damage_list.len(), 2);
    }

    #[test]
    fn test_field_init_damage_minus1_with_nonzero_start_sets_init_damage() {
        // Trigger the `else if init_damage == -1 && start != 0` branch.
        // To enter it: provide "start" but NOT "damage".
        // After the loop: init_damage=-1 (never set), start=50 (never reset) → branch fires.
        let mut r = registry_with(109, 209);
        let xml = r#"<items>
            <item id="109">
                <attribute key="field" value="fire">
                    <attribute key="ticks" value="5"/>
                    <attribute key="start" value="50"/>
                </attribute>
            </item>
        </items>"#;
        r.load_from_xml(xml).expect("ok");
        let it = r.get_item_type(109).unwrap();
        // start=50, init_damage=-1, no "damage" key → start never reset → set_init_damage(50)
        let cd = it
            .condition_damage
            .as_ref()
            .expect("condition_damage should be Some because fire field was set");
        assert_eq!(cd.init_damage, 50);
    }

    // ----- ItemTypeData new fields default values -----

    #[test]
    fn test_item_type_data_new_xml_fields_default() {
        let it = ItemTypeData::default();
        assert!(it.abilities.is_none());
        assert_eq!(it.combat_type, CombatTypeFlags::NONE);
        assert_eq!(it.bed_partner_dir, Direction::None);
        assert_eq!(it.transform_to_on_use, [0u16, 0u16]);
        assert_eq!(it.magic_effect, MagicEffectClass::None);
        assert_eq!(it.ammo_type, Ammo::None);
        assert_eq!(it.shoot_type, ShootType::None);
        assert_eq!(it.corpse_type, RaceType::None);
        assert_eq!(it.fluid_source, FluidType::None);
        assert!(!it.can_write_text);
    }

    // -----------------------------------------------------------------------
    // ItemTypeData predicate methods
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_type_data_is_ground_tile_false_for_default() {
        let it = ItemTypeData::default();
        assert!(!it.is_ground_tile());
    }

    #[test]
    fn test_item_type_data_is_container_false_for_default() {
        let it = ItemTypeData::default();
        assert!(!it.is_container());
    }

    #[test]
    fn test_item_type_data_is_splash() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_splash());
        it.group = ItemGroup::Splash;
        assert!(it.is_splash());
    }

    #[test]
    fn test_item_type_data_is_fluid_container() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_fluid_container());
        it.group = ItemGroup::Fluid;
        assert!(it.is_fluid_container());
    }

    #[test]
    fn test_item_type_data_is_door() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_door());
        it.type_kind = ItemTypeKind::Door;
        assert!(it.is_door());
    }

    #[test]
    fn test_item_type_data_is_magic_field() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_magic_field());
        it.type_kind = ItemTypeKind::MagicField;
        assert!(it.is_magic_field());
    }

    #[test]
    fn test_item_type_data_is_teleport() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_teleport());
        it.type_kind = ItemTypeKind::Teleport;
        assert!(it.is_teleport());
    }

    #[test]
    fn test_item_type_data_is_key() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_key());
        it.type_kind = ItemTypeKind::Key;
        assert!(it.is_key());
    }

    #[test]
    fn test_item_type_data_is_depot() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_depot());
        it.type_kind = ItemTypeKind::Depot;
        assert!(it.is_depot());
    }

    #[test]
    fn test_item_type_data_is_mailbox() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_mailbox());
        it.type_kind = ItemTypeKind::Mailbox;
        assert!(it.is_mailbox());
    }

    #[test]
    fn test_item_type_data_is_trash_holder() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_trash_holder());
        it.type_kind = ItemTypeKind::TrashHolder;
        assert!(it.is_trash_holder());
    }

    #[test]
    fn test_item_type_data_is_bed() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_bed());
        it.type_kind = ItemTypeKind::Bed;
        assert!(it.is_bed());
    }

    #[test]
    fn test_item_type_data_is_rune() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_rune());
        it.type_kind = ItemTypeKind::Rune;
        assert!(it.is_rune());
    }

    #[test]
    fn test_item_type_data_is_podium() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_podium());
        it.type_kind = ItemTypeKind::Podium;
        assert!(it.is_podium());
    }

    #[test]
    fn test_item_type_data_is_useable() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_useable());
        it.useable = true;
        assert!(it.is_useable());
    }

    #[test]
    fn test_item_type_data_is_supply() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_supply());
        it.supply = true;
        assert!(it.is_supply());
    }

    #[test]
    fn test_item_type_data_has_sub_type_fluid_container() {
        let mut it = ItemTypeData::default();
        assert!(!it.has_sub_type());
        it.group = ItemGroup::Fluid;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_item_type_data_has_sub_type_splash() {
        let mut it = ItemTypeData::default();
        it.group = ItemGroup::Splash;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_item_type_data_has_sub_type_stackable() {
        let mut it = ItemTypeData::default();
        it.stackable = true;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_item_type_data_has_sub_type_charges() {
        let mut it = ItemTypeData::default();
        it.charges = 10;
        assert!(it.has_sub_type());
    }

    #[test]
    fn test_item_type_data_is_pickupable_allow_pickupable() {
        let mut it = ItemTypeData::default();
        assert!(!it.is_pickupable());
        it.allow_pickupable = true;
        assert!(it.is_pickupable());
    }

    #[test]
    fn test_item_type_data_is_pickupable_pickupable() {
        let mut it = ItemTypeData::default();
        it.pickupable = true;
        assert!(it.is_pickupable());
    }

    #[test]
    fn test_item_type_data_get_plural_name_explicit_short() {
        let mut it = ItemTypeData::default();
        it.plural_name = "swords".to_string();
        it.name = "sword".to_string();
        it.show_count = true;
        assert_eq!(it.get_plural_name(), "swords");
    }

    #[test]
    fn test_item_type_data_get_plural_name_no_show_count() {
        let mut it = ItemTypeData::default();
        it.name = "sword".to_string();
        it.show_count = false;
        // No explicit plural, show_count=false → returns name unchanged
        assert_eq!(it.get_plural_name(), "sword");
    }

    #[test]
    fn test_item_type_data_get_plural_name_show_count_appends_s() {
        let mut it = ItemTypeData::default();
        it.name = "sword".to_string();
        it.show_count = true;
        assert_eq!(it.get_plural_name(), "swords");
    }

    #[test]
    fn test_item_type_data_get_plural_name_already_ends_in_s() {
        let mut it = ItemTypeData::default();
        it.name = "grass".to_string();
        it.show_count = true;
        // Name ends with 's' → return name as-is
        assert_eq!(it.get_plural_name(), "grass");
    }

    #[test]
    fn test_item_type_data_get_plural_name_empty_name() {
        let it = ItemTypeData::default();
        // Empty name + show_count=false → returns ""
        assert_eq!(it.get_plural_name(), "");
    }

    // -----------------------------------------------------------------------
    // Items utility methods
    // -----------------------------------------------------------------------

    #[test]
    fn test_items_new_is_empty() {
        let reg = Items::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_items_len_and_is_empty() {
        let reg = registry_with(100, 200);
        assert_eq!(reg.len(), 1);
        assert!(!reg.is_empty());
    }

    #[test]
    fn test_items_get_item_type_by_client_id_low_rejected() {
        let reg = registry_with(100, 200);
        // client_id < 100 always returns None
        assert!(reg.get_item_type_by_client_id(50).is_none());
        assert!(reg.get_item_type_by_client_id(99).is_none());
    }

    #[test]
    fn test_items_get_item_type_by_client_id_found() {
        let reg = registry_with(100, 200);
        let it = reg.get_item_type_by_client_id(200);
        assert!(it.is_some());
        assert_eq!(it.unwrap().id, 100);
    }

    #[test]
    fn test_items_get_item_type_by_client_id_zero_server_id() {
        // client_id not mapped (not in registry) → None
        let reg = registry_with(100, 200);
        assert!(reg.get_item_type_by_client_id(201).is_none());
    }

    #[test]
    fn test_items_has_item_type_true() {
        let reg = registry_with(100, 200);
        assert!(reg.has_item_type(100));
    }

    #[test]
    fn test_items_has_item_type_false() {
        let reg = registry_with(100, 200);
        assert!(!reg.has_item_type(101));
    }

    #[test]
    fn test_items_register_name() {
        let mut reg = registry_with(100, 200);
        reg.register_name("TestItem", 100);
        let it = reg.get_item_type_by_name("testitem");
        assert!(it.is_some());
        assert_eq!(it.unwrap().id, 100);
    }

    #[test]
    fn test_items_register_name_duplicate_ignored() {
        let mut reg = registry_with(100, 200);
        reg.register_name("coin", 100);
        // Second registration for same name is ignored
        reg.register_name("coin", 99);
        assert_eq!(reg.get_item_type_by_name("coin").unwrap().id, 100);
    }

    #[test]
    fn test_items_get_max_item_id() {
        let nodes = vec![
            OtbItemNode {
                server_id: 5,
                client_id: 105,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
            OtbItemNode {
                server_id: 10,
                client_id: 110,
                group: ItemGroup::Ground as u8,
                ..Default::default()
            },
        ];
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &nodes);
        let reg = Items::load_from_otb(&otb).expect("ok");
        assert_eq!(reg.get_max_item_id(), 10);
    }

    #[test]
    fn test_items_get_max_item_id_empty() {
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[]);
        let reg = Items::load_from_otb(&otb).expect("ok");
        assert_eq!(reg.get_max_item_id(), 0);
    }

    #[test]
    fn test_items_clear() {
        let mut reg = registry_with(100, 200);
        let xml = r#"<items><item id="100" name="coin"><attribute key="worth" value="1"/></item></items>"#;
        reg.load_from_xml(xml).expect("ok");
        assert_eq!(reg.len(), 1);
        assert!(!reg.currency_items.is_empty());

        reg.clear();

        assert_eq!(reg.len(), 0);
        assert!(reg.currency_items.is_empty());
        assert_eq!(reg.major_version, 0);
        assert_eq!(reg.minor_version, 0);
        assert_eq!(reg.build_number, 0);
        assert!(reg.get_item_type_by_name("coin").is_none());
    }

    #[test]
    fn test_items_reload() {
        let mut reg = registry_with(100, 200);
        let xml1 = r#"<items><item id="100" name="old"/></items>"#;
        reg.load_from_xml(xml1).expect("ok");
        assert_eq!(reg.get_item_type(100).unwrap().name, "old");

        // Build a new OTB with a different item
        let node = OtbItemNode {
            server_id: 50,
            client_id: 150,
            group: ItemGroup::Ground as u8,
            ..Default::default()
        };
        let new_otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let xml2 = r#"<items><item id="50" name="new"/></items>"#;
        reg.reload(&new_otb, xml2).expect("ok");

        // Old item gone, new item present
        assert!(reg.get_item_type(100).is_none());
        assert_eq!(reg.get_item_type(50).unwrap().name, "new");
    }

    #[test]
    fn test_items_reload_bad_otb_returns_err() {
        let mut reg = registry_with(100, 200);
        let result = reg.reload(b"bad", r#"<items/>"#);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // OTB parse_otb: error paths
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_otb_unknown_item_group_returns_err() {
        use forgottenserver_common::fileloader::{NODE_END, NODE_START};

        // Build an OTB with a child item node whose group byte is 255 (unknown).
        let mut root_props: Vec<u8> = Vec::new();
        root_props.extend_from_slice(&0u32.to_le_bytes()); // flags
        root_props.push(0x01); // ROOT_ATTR_VERSION
        root_props.extend_from_slice(&140u16.to_le_bytes()); // datalen
        root_props.extend_from_slice(&3u32.to_le_bytes()); // major
        root_props.extend_from_slice(&CLIENT_VERSION_LAST.to_le_bytes()); // minor
        root_props.extend_from_slice(&1u32.to_le_bytes()); // build
        root_props.extend_from_slice(&[0u8; 128]); // CSD
                                                   // Append a special byte to exercise the escape branch in the root-props loop.
        root_props.push(0xFD);

        // Item props: flags (4 bytes) + ServerId attr
        let mut item_props: Vec<u8> = Vec::new();
        item_props.extend_from_slice(&0u32.to_le_bytes()); // flags
        item_props.push(0x10); // ITEM_ATTR_SERVERID
        item_props.extend_from_slice(&2u16.to_le_bytes()); // datalen=2
        item_props.extend_from_slice(&100u16.to_le_bytes()); // server_id=100
                                                             // Append a special byte to exercise the escape branches in the buffer-build loops.
        item_props.push(0xFD);

        // Manually build the raw OTB blob with group byte = 255
        let mut buf = vec![0x4F, 0x54, 0x42, 0x49]; // OTBI
        buf.push(NODE_START); // root node start
        buf.push(0x00); // root type byte
                        // Root props (escaped)
        for &b in &root_props {
            if b == NODE_START || b == NODE_END || b == 0xFD {
                buf.push(0xFD); // ESCAPE
            }
            buf.push(b);
        }
        // Child node with group=255
        buf.push(NODE_START);
        buf.push(255u8); // unknown group byte
        for &b in &item_props {
            if b == NODE_START || b == NODE_END || b == 0xFD {
                buf.push(0xFD);
            }
            buf.push(b);
        }
        buf.push(NODE_END); // child end
        buf.push(NODE_END); // root end

        let result = Items::load_from_otb(&buf);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown item group"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // Helper function unit tests — direct calls (not through XML)
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_skill_initializes_abilities() {
        let mut it = ItemTypeData::default();
        assert!(it.abilities.is_none());
        super::set_skill(&mut it, Skill::Sword, 10);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(a.skills[Skill::Sword as usize], 10);
    }

    #[test]
    fn test_set_skill_all_variants() {
        for (skill, val) in [
            (Skill::Axe, 1),
            (Skill::Club, 2),
            (Skill::Distance, 3),
            (Skill::Fishing, 4),
            (Skill::Shield, 5),
            (Skill::Fist, 6),
        ] {
            let mut it = ItemTypeData::default();
            super::set_skill(&mut it, skill, val);
            assert_eq!(it.abilities.as_ref().unwrap().skills[skill as usize], val);
        }
    }

    #[test]
    fn test_set_special_skill_initializes_abilities() {
        let mut it = ItemTypeData::default();
        super::set_special_skill(&mut it, SpecialSkill::CriticalHitAmount, 50);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.special_skills[SpecialSkill::CriticalHitAmount as usize],
            50
        );
    }

    #[test]
    fn test_set_special_skill_all_variants() {
        for (ss, val) in [
            (SpecialSkill::CriticalHitChance, 10),
            (SpecialSkill::ManaLeechAmount, 20),
            (SpecialSkill::ManaLeechChance, 30),
            (SpecialSkill::LifeLeechAmount, 40),
            (SpecialSkill::LifeLeechChance, 50),
        ] {
            let mut it = ItemTypeData::default();
            super::set_special_skill(&mut it, ss, val);
            assert_eq!(
                it.abilities.as_ref().unwrap().special_skills[ss as usize],
                val
            );
        }
    }

    #[test]
    fn test_set_stat_initializes_abilities() {
        let mut it = ItemTypeData::default();
        super::set_stat(&mut it, Stat::MaxHitPoints, 100);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(a.stats[Stat::MaxHitPoints as usize], 100);
    }

    #[test]
    fn test_set_stat_all_variants() {
        for (stat, val) in [(Stat::MaxManaPoints, 200), (Stat::MagicPoints, 300)] {
            let mut it = ItemTypeData::default();
            super::set_stat(&mut it, stat, val);
            assert_eq!(it.abilities.as_ref().unwrap().stats[stat as usize], val);
        }
    }

    #[test]
    fn test_set_stat_percent_initializes_abilities() {
        let mut it = ItemTypeData::default();
        super::set_stat_percent(&mut it, Stat::MaxHitPoints, 25);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(a.stats_percent[Stat::MaxHitPoints as usize], 25);
    }

    #[test]
    fn test_add_absorb_single_combat_type() {
        let mut it = ItemTypeData::default();
        super::add_absorb(&mut it, CombatTypeFlags::FIRE, 15);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::FIRE)],
            15
        );
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::ENERGY)],
            0
        );
    }

    #[test]
    fn test_add_absorb_accumulates() {
        let mut it = ItemTypeData::default();
        super::add_absorb(&mut it, CombatTypeFlags::ICE, 10);
        super::add_absorb(&mut it, CombatTypeFlags::ICE, 5);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::ICE)],
            15
        );
    }

    #[test]
    fn test_add_field_absorb_single_combat_type() {
        let mut it = ItemTypeData::default();
        super::add_field_absorb(&mut it, CombatTypeFlags::FIRE, 20);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.field_absorb_percent[combat_type_to_index(CombatTypeFlags::FIRE)],
            20
        );
        assert_eq!(
            a.field_absorb_percent[combat_type_to_index(CombatTypeFlags::ENERGY)],
            0
        );
    }

    #[test]
    fn test_add_boost_single_combat_type() {
        let mut it = ItemTypeData::default();
        super::add_boost(&mut it, CombatTypeFlags::EARTH, 12);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::EARTH)],
            12
        );
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::FIRE)],
            0
        );
    }

    #[test]
    fn test_add_magic_level_single_combat_type() {
        let mut it = ItemTypeData::default();
        super::add_magic_level(&mut it, CombatTypeFlags::HOLY, 7);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.special_magic_level_skill[combat_type_to_index(CombatTypeFlags::HOLY)],
            7
        );
        assert_eq!(
            a.special_magic_level_skill[combat_type_to_index(CombatTypeFlags::FIRE)],
            0
        );
    }

    #[test]
    fn test_add_absorb_all_covers_all_slots() {
        let mut it = ItemTypeData::default();
        super::add_absorb_all(&mut it, 10);
        let a = it.abilities.as_ref().unwrap();
        assert!(a.absorb_percent.iter().all(|&v| v == 10));
    }

    #[test]
    fn test_add_absorb_all_accumulates() {
        let mut it = ItemTypeData::default();
        super::add_absorb_all(&mut it, 5);
        super::add_absorb_all(&mut it, 3);
        let a = it.abilities.as_ref().unwrap();
        assert!(a.absorb_percent.iter().all(|&v| v == 8));
    }

    #[test]
    fn test_add_absorb_elements_sets_energy_fire_earth_ice() {
        let mut it = ItemTypeData::default();
        super::add_absorb_elements(&mut it, 5);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::ENERGY)],
            5
        );
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::FIRE)],
            5
        );
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::EARTH)],
            5
        );
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::ICE)],
            5
        );
        // Non-element types should be untouched
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::HOLY)],
            0
        );
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::PHYSICAL)],
            0
        );
    }

    #[test]
    fn test_add_absorb_magic_sets_six_combat_types() {
        let mut it = ItemTypeData::default();
        super::add_absorb_magic(&mut it, 3);
        let a = it.abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
            CombatTypeFlags::HOLY,
            CombatTypeFlags::DEATH,
        ] {
            assert_eq!(
                a.absorb_percent[combat_type_to_index(c)],
                3,
                "failed for combat {c}"
            );
        }
        assert_eq!(
            a.absorb_percent[combat_type_to_index(CombatTypeFlags::PHYSICAL)],
            0
        );
    }

    #[test]
    fn test_add_boost_all_covers_all_slots() {
        let mut it = ItemTypeData::default();
        super::add_boost_all(&mut it, 6);
        let a = it.abilities.as_ref().unwrap();
        assert!(a.boost_percent.iter().all(|&v| v == 6));
    }

    #[test]
    fn test_add_boost_elements_sets_energy_fire_earth_ice() {
        let mut it = ItemTypeData::default();
        super::add_boost_elements(&mut it, 4);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::ENERGY)],
            4
        );
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::FIRE)],
            4
        );
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::EARTH)],
            4
        );
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::ICE)],
            4
        );
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::HOLY)],
            0
        );
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::PHYSICAL)],
            0
        );
    }

    #[test]
    fn test_add_boost_magic_sets_six_combat_types() {
        let mut it = ItemTypeData::default();
        super::add_boost_magic(&mut it, 2);
        let a = it.abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
            CombatTypeFlags::HOLY,
            CombatTypeFlags::DEATH,
        ] {
            assert_eq!(
                a.boost_percent[combat_type_to_index(c)],
                2,
                "failed for combat {c}"
            );
        }
        assert_eq!(
            a.boost_percent[combat_type_to_index(CombatTypeFlags::PHYSICAL)],
            0
        );
    }

    #[test]
    fn test_reflect_at_initializes_and_returns_mutable() {
        let mut it = ItemTypeData::default();
        assert!(it.abilities.is_none());
        let r = super::reflect_at(&mut it, CombatTypeFlags::FIRE);
        r.percent = 50;
        r.chance = 25;
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::FIRE)].percent,
            50
        );
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::FIRE)].chance,
            25
        );
    }

    #[test]
    fn test_mutate_reflect_all_sets_all_slots() {
        let mut it = ItemTypeData::default();
        super::mutate_reflect_all(&mut it, |r| {
            r.percent = 20;
            r.chance = 10;
        });
        let a = it.abilities.as_ref().unwrap();
        for r in a.reflect.iter() {
            assert_eq!(r.percent, 20);
            assert_eq!(r.chance, 10);
        }
    }

    #[test]
    fn test_mutate_reflect_elements_sets_four_slots() {
        let mut it = ItemTypeData::default();
        super::mutate_reflect_elements(&mut it, |r| r.percent = 15);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::ENERGY)].percent,
            15
        );
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::FIRE)].percent,
            15
        );
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::EARTH)].percent,
            15
        );
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::ICE)].percent,
            15
        );
        // Non-element types untouched
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::HOLY)].percent,
            0
        );
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::PHYSICAL)].percent,
            0
        );
    }

    #[test]
    fn test_mutate_reflect_magic_sets_six_slots() {
        let mut it = ItemTypeData::default();
        super::mutate_reflect_magic(&mut it, |r| r.chance = 8);
        let a = it.abilities.as_ref().unwrap();
        for c in [
            CombatTypeFlags::ENERGY,
            CombatTypeFlags::FIRE,
            CombatTypeFlags::EARTH,
            CombatTypeFlags::ICE,
            CombatTypeFlags::HOLY,
            CombatTypeFlags::DEATH,
        ] {
            assert_eq!(
                a.reflect[combat_type_to_index(c)].chance,
                8,
                "failed for combat {c}"
            );
        }
        assert_eq!(
            a.reflect[combat_type_to_index(CombatTypeFlags::PHYSICAL)].chance,
            0
        );
    }

    #[test]
    fn test_suppress_sets_bit_when_true() {
        let mut it = ItemTypeData::default();
        super::suppress(&mut it, ConditionTypeFlags::FIRE as u32, true);
        let a = it.abilities.as_ref().unwrap();
        assert!(a.condition_suppressions & ConditionTypeFlags::FIRE as u32 != 0);
    }

    #[test]
    fn test_suppress_does_nothing_when_false() {
        let mut it = ItemTypeData::default();
        super::suppress(&mut it, ConditionTypeFlags::FIRE as u32, false);
        // Abilities should NOT be initialized when on=false
        assert!(it.abilities.is_none());
    }

    #[test]
    fn test_suppress_multiple_bits() {
        let mut it = ItemTypeData::default();
        super::suppress(&mut it, ConditionTypeFlags::FIRE as u32, true);
        super::suppress(&mut it, ConditionTypeFlags::ENERGY as u32, true);
        let a = it.abilities.as_ref().unwrap();
        assert!(a.condition_suppressions & ConditionTypeFlags::FIRE as u32 != 0);
        assert!(a.condition_suppressions & ConditionTypeFlags::ENERGY as u32 != 0);
        assert_eq!(
            a.condition_suppressions & ConditionTypeFlags::POISON as u32,
            0
        );
    }

    #[test]
    fn test_set_element_sets_damage_and_type() {
        let mut it = ItemTypeData::default();
        super::set_element(&mut it, CombatTypeFlags::ICE, 42);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(a.element_damage, 42);
        assert_eq!(a.element_type, CombatTypeFlags::ICE);
    }

    #[test]
    fn test_set_element_overwrites_previous() {
        let mut it = ItemTypeData::default();
        super::set_element(&mut it, CombatTypeFlags::FIRE, 10);
        super::set_element(&mut it, CombatTypeFlags::EARTH, 20);
        let a = it.abilities.as_ref().unwrap();
        assert_eq!(a.element_damage, 20);
        assert_eq!(a.element_type, CombatTypeFlags::EARTH);
    }

    // -----------------------------------------------------------------------
    // OTB group type coverage — Teleport and Podium groups produce correct
    // ItemTypeKind assignments
    // -----------------------------------------------------------------------

    #[test]
    fn test_otb_teleport_group_sets_type_kind() {
        let node = OtbItemNode {
            server_id: 100,
            client_id: 200,
            group: ItemGroup::Teleport as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let reg = Items::load_from_otb(&otb).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.group, ItemGroup::Teleport);
        assert_eq!(it.type_kind, ItemTypeKind::Teleport);
        assert!(it.is_teleport());
    }

    #[test]
    fn test_otb_podium_group_sets_type_kind() {
        let node = OtbItemNode {
            server_id: 100,
            client_id: 200,
            group: ItemGroup::Podium as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let reg = Items::load_from_otb(&otb).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.group, ItemGroup::Podium);
        assert_eq!(it.type_kind, ItemTypeKind::Podium);
        assert!(it.is_podium());
    }

    #[test]
    fn test_otb_door_group_sets_type_kind() {
        let node = OtbItemNode {
            server_id: 100,
            client_id: 200,
            group: ItemGroup::Door as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let reg = Items::load_from_otb(&otb).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::Door);
        assert!(it.is_door());
    }

    #[test]
    fn test_otb_magic_field_group_sets_type_kind() {
        let node = OtbItemNode {
            server_id: 100,
            client_id: 200,
            group: ItemGroup::MagicField as u8,
            ..Default::default()
        };
        let otb = build_otb(3, CLIENT_VERSION_LAST, 1, &[node]);
        let reg = Items::load_from_otb(&otb).expect("ok");
        let it = reg.get_item_type(100).unwrap();
        assert_eq!(it.type_kind, ItemTypeKind::MagicField);
        assert!(it.is_magic_field());
    }

    // -----------------------------------------------------------------------
    // apply_attribute_field: initDamage < -1 path
    // -----------------------------------------------------------------------

    #[test]
    fn test_attr_field_init_damage_negative_below_minus_one() {
        // initDamage = -5 is < -1 → !(-1..=0).contains(-5) is true
        // → cd.set_init_damage(-(-5)) = 5
        let mut r = registry_with(100, 200);
        parse_one(
            &mut r,
            100,
            r#"<attribute key="field" value="fire">
                  <attribute key="ticks" value="1000"/>
                  <attribute key="count" value="1"/>
                  <attribute key="damage" value="5"/>
                  <attribute key="initdamage" value="-5"/>
               </attribute>"#,
        );
        let it = r.get_item_type(100).unwrap();
        let cd = it.condition_damage.as_ref().expect("condition_damage set");
        // set_init_damage(-(-5)) = set_init_damage(5)
        assert_eq!(cd.init_damage, 5);
    }

    // -----------------------------------------------------------------------
    // apply_slot_type: apply_slot_type helper via XML for all slot values
    // (the "hand" slot variant is not exercised by the existing slot test)
    // -----------------------------------------------------------------------

    #[test]
    fn test_attr_slottype_hand() {
        let mut r = registry_with(100, 200);
        parse_one(&mut r, 100, r#"<attribute key="slottype" value="hand"/>"#);
        let it = r.get_item_type(100).unwrap();
        assert!(
            it.slot_position & slot_position::HAND != 0,
            "HAND bits should be set"
        );
    }

    // -----------------------------------------------------------------------
    // Items::get_item_type_by_client_id: client_id mapped but server_id=0
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_item_type_by_client_id_out_of_range_returns_none() {
        let reg = registry_with(100, 200);
        // client_id=65535 has never been mapped → None
        assert!(reg.get_item_type_by_client_id(65535).is_none());
    }
}
