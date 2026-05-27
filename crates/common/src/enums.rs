//! Migrated from forgottenserver/src/enums.h
//! Maps all enums, type aliases, and plain-data structs.

#![allow(dead_code)]
#![allow(non_camel_case_types)]

// ---------------------------------------------------------------------------
// RuleViolationType_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RuleViolationType {
    Name = 0,
    Statement = 1,
    Bot = 2,
}

// ---------------------------------------------------------------------------
// RuleViolationReasons_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RuleViolationReason {
    NameInappropriate = 0,
    NamePoorFormatted = 1,
    NameAdvertising = 2,
    NameUnfitting = 3,
    NameRuleViolation = 4,
    InsultingStatement = 5,
    Spamming = 6,
    AdvertisingStatement = 7,
    UnfittingStatement = 8,
    LanguageStatement = 9,
    Disclosure = 10,
    RuleViolation = 11,
    StatementBugAbuse = 12,
    UnofficialSoftware = 13,
    Pretending = 14,
    HarassingOwners = 15,
    FalseInfo = 16,
    AccountSharing = 17,
    StealingData = 18,
    ServiceAttacking = 19,
    ServiceAgreement = 20,
}

// ---------------------------------------------------------------------------
// ThreadState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThreadState {
    Running = 0,
    Closing = 1,
    Terminated = 2,
}

// ---------------------------------------------------------------------------
// itemAttrTypes — u32 bit flags
// ---------------------------------------------------------------------------

pub struct ItemAttrFlags;

impl ItemAttrFlags {
    pub const NONE: u32 = 0;
    pub const ACTIONID: u32 = 1 << 0;
    pub const UNIQUEID: u32 = 1 << 1;
    pub const DESCRIPTION: u32 = 1 << 2;
    pub const TEXT: u32 = 1 << 3;
    pub const DATE: u32 = 1 << 4;
    pub const WRITER: u32 = 1 << 5;
    pub const NAME: u32 = 1 << 6;
    pub const ARTICLE: u32 = 1 << 7;
    pub const PLURALNAME: u32 = 1 << 8;
    pub const WEIGHT: u32 = 1 << 9;
    pub const ATTACK: u32 = 1 << 10;
    pub const DEFENSE: u32 = 1 << 11;
    pub const EXTRADEFENSE: u32 = 1 << 12;
    pub const ARMOR: u32 = 1 << 13;
    pub const HITCHANCE: u32 = 1 << 14;
    pub const SHOOTRANGE: u32 = 1 << 15;
    pub const OWNER: u32 = 1 << 16;
    /// In C++ this bit (1 << 17) served double duty as both `ITEM_ATTRIBUTE_DURATION`
    /// and `ITEM_ATTRIBUTE_DURATION_MIN` (an alias). `DURATION_MIN` has been removed
    /// here because it is identical to `DURATION` — use `DURATION` directly.
    pub const DURATION: u32 = 1 << 17;
    pub const DECAYSTATE: u32 = 1 << 18;
    pub const CORPSEOWNER: u32 = 1 << 19;
    pub const CHARGES: u32 = 1 << 20;
    pub const FLUIDTYPE: u32 = 1 << 21;
    pub const DOORID: u32 = 1 << 22;
    pub const DECAYTO: u32 = 1 << 23;
    pub const WRAPID: u32 = 1 << 24;
    pub const STOREITEM: u32 = 1 << 25;
    pub const ATTACK_SPEED: u32 = 1 << 26;
    pub const OPENCONTAINER: u32 = 1 << 27;
    pub const DURATION_MAX: u32 = 1 << 28;
    pub const CUSTOM: u32 = 1u32 << 31;
}

// ---------------------------------------------------------------------------
// VipStatus_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VipStatus {
    Offline = 0,
    Online = 1,
    Pending = 2,
    Training = 3,
}

// ---------------------------------------------------------------------------
// MarketAction_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MarketAction {
    Buy = 0,
    Sell = 1,
}

// ---------------------------------------------------------------------------
// MarketRequest_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MarketRequest {
    OwnHistory = 1,
    OwnOffers = 2,
    Item = 3,
}

// ---------------------------------------------------------------------------
// MarketOfferState_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MarketOfferState {
    Active = 0,
    Cancelled = 1,
    Expired = 2,
    Accepted = 3,
    AcceptedEx = 255,
}

// ---------------------------------------------------------------------------
// ChannelEvent_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ChannelEvent {
    Join = 0,
    Leave = 1,
    Invite = 2,
    Exclude = 3,
}

// ---------------------------------------------------------------------------
// CreatureType_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CreatureType {
    Player = 0,
    Monster = 1,
    Npc = 2,
    SummonOwn = 3,
    SummonOthers = 4,
    Hidden = 5,
}

// ---------------------------------------------------------------------------
// OperatingSystem_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OperatingSystem {
    None = 0,
    Linux = 1,
    Windows = 2,
    Flash = 3,
    QtLinux = 4,
    QtWindows = 5,
    QtMac = 6,
    QtLinux2 = 7,
    OtClientLinux = 10,
    OtClientWindows = 11,
    OtClientMac = 12,
}

// ---------------------------------------------------------------------------
// SpellGroup_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellGroup {
    None = 0,
    Attack = 1,
    Healing = 2,
    Support = 3,
    Special = 4,
    Crippling = 6,
    Focus = 7,
    UltimateStrikes = 8,
}

// ---------------------------------------------------------------------------
// SpellType_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpellType {
    Undefined = 0,
    Instant = 1,
    Rune = 2,
}

// ---------------------------------------------------------------------------
// AccountType_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccountType {
    Normal = 1,
    Tutor = 2,
    SeniorTutor = 3,
    GameMaster = 4,
    CommunityManager = 5,
    God = 6,
}

// ---------------------------------------------------------------------------
// RaceType_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RaceType {
    None = 0,
    Venom = 1,
    Blood = 2,
    Undead = 3,
    Fire = 4,
    Energy = 5,
    Ink = 6,
}

// ---------------------------------------------------------------------------
// CombatType_t — u16 bit flags
// ---------------------------------------------------------------------------

pub struct CombatTypeFlags;

impl CombatTypeFlags {
    pub const NONE: u16 = 0;
    pub const PHYSICAL: u16 = 1 << 0;
    pub const ENERGY: u16 = 1 << 1;
    pub const EARTH: u16 = 1 << 2;
    pub const FIRE: u16 = 1 << 3;
    pub const UNDEFINED: u16 = 1 << 4;
    pub const LIFEDRAIN: u16 = 1 << 5;
    pub const MANADRAIN: u16 = 1 << 6;
    pub const HEALING: u16 = 1 << 7;
    pub const DROWN: u16 = 1 << 8;
    pub const ICE: u16 = 1 << 9;
    pub const HOLY: u16 = 1 << 10;
    pub const DEATH: u16 = 1 << 11;
    pub const COUNT: u16 = 12;
}

// ---------------------------------------------------------------------------
// CombatParam_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatParam {
    Type,
    Effect,
    DistanceEffect,
    BlockShield,
    BlockArmor,
    TargetCasterOrTopmost,
    CreateItem,
    Aggressive,
    Dispel,
    UseCharges,
}

// ---------------------------------------------------------------------------
// CallBackParam_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallBackParam {
    LevelMagicValue,
    SkillValue,
    TargetTile,
    TargetCreature,
}

// ---------------------------------------------------------------------------
// ConditionParam_t — explicit discriminants
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ConditionParam {
    Owner = 1,
    Ticks = 2,
    HealthGain = 4,
    HealthTicks = 5,
    ManaGain = 6,
    ManaTicks = 7,
    Delayed = 8,
    Speed = 9,
    LightLevel = 10,
    LightColor = 11,
    SoulGain = 12,
    SoulTicks = 13,
    MinValue = 14,
    MaxValue = 15,
    StartValue = 16,
    TickInterval = 17,
    ForceUpdate = 18,
    SkillMelee = 19,
    SkillFist = 20,
    SkillClub = 21,
    SkillSword = 22,
    SkillAxe = 23,
    SkillDistance = 24,
    SkillShield = 25,
    SkillFishing = 26,
    StatMaxHitPoints = 27,
    StatMaxManaPoints = 28,
    StatMagicPoints = 30,
    StatMaxHitPointsPercent = 31,
    StatMaxManaPointsPercent = 32,
    StatMagicPointsPercent = 34,
    PeriodicDamage = 35,
    SkillMeleePercent = 36,
    SkillFistPercent = 37,
    SkillClubPercent = 38,
    SkillSwordPercent = 39,
    SkillAxePercent = 40,
    SkillDistancePercent = 41,
    SkillShieldPercent = 42,
    SkillFishingPercent = 43,
    BuffSpell = 44,
    SubId = 45,
    Field = 46,
    DisableDefense = 47,
    SpecialSkillCriticalHitChance = 48,
    SpecialSkillCriticalHitAmount = 49,
    SpecialSkillLifeLeechChance = 50,
    SpecialSkillLifeLeechAmount = 51,
    SpecialSkillManaLeechChance = 52,
    SpecialSkillManaLeechAmount = 53,
    Aggressive = 54,
    Drunkenness = 55,
    ManaShieldBreakable = 56,
}

// ---------------------------------------------------------------------------
// BlockType_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockType {
    None = 0,
    Defense = 1,
    Armor = 2,
    Immunity = 3,
}

// ---------------------------------------------------------------------------
// skills_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Skill {
    Fist = 0,
    Club = 1,
    Sword = 2,
    Axe = 3,
    Distance = 4,
    Shield = 5,
    Fishing = 6,
    MagLevel = 7,
    Level = 8,
}

impl Skill {
    pub const FIRST: Skill = Skill::Fist;
    pub const LAST: Skill = Skill::Fishing;
}

// ---------------------------------------------------------------------------
// stats_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    MaxHitPoints,
    MaxManaPoints,
    SoulPoints, // unused
    MagicPoints,
}

impl Stat {
    pub const FIRST: Stat = Stat::MaxHitPoints;
    pub const LAST: Stat = Stat::MagicPoints;
}

// ---------------------------------------------------------------------------
// SpecialSkills_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialSkill {
    CriticalHitChance,
    CriticalHitAmount,
    LifeLeechChance,
    LifeLeechAmount,
    ManaLeechChance,
    ManaLeechAmount,
}

impl SpecialSkill {
    pub const FIRST: SpecialSkill = SpecialSkill::CriticalHitChance;
    pub const LAST: SpecialSkill = SpecialSkill::ManaLeechAmount;
}

// ---------------------------------------------------------------------------
// formulaType_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaType {
    Undefined,
    LevelMagic,
    Skill,
    Damage,
}

// ---------------------------------------------------------------------------
// ConditionType_t — i32 bit flags
// ---------------------------------------------------------------------------

pub struct ConditionTypeFlags;

impl ConditionTypeFlags {
    pub const NONE: i32 = 0;
    pub const POISON: i32 = 1 << 0;
    pub const FIRE: i32 = 1 << 1;
    pub const ENERGY: i32 = 1 << 2;
    pub const BLEEDING: i32 = 1 << 3;
    pub const HASTE: i32 = 1 << 4;
    pub const PARALYZE: i32 = 1 << 5;
    pub const OUTFIT: i32 = 1 << 6;
    pub const INVISIBLE: i32 = 1 << 7;
    pub const LIGHT: i32 = 1 << 8;
    pub const MANASHIELD: i32 = 1 << 9;
    pub const INFIGHT: i32 = 1 << 10;
    pub const DRUNK: i32 = 1 << 11;
    pub const EXHAUST_WEAPON: i32 = 1 << 12; // unused
    pub const REGENERATION: i32 = 1 << 13;
    pub const SOUL: i32 = 1 << 14;
    pub const DROWN: i32 = 1 << 15;
    pub const MUTED: i32 = 1 << 16;
    pub const CHANNELMUTEDTICKS: i32 = 1 << 17;
    pub const YELLTICKS: i32 = 1 << 18;
    pub const ATTRIBUTES: i32 = 1 << 19;
    pub const FREEZING: i32 = 1 << 20;
    pub const DAZZLED: i32 = 1 << 21;
    pub const CURSED: i32 = 1 << 22;
    pub const EXHAUST_COMBAT: i32 = 1 << 23; // unused
    pub const EXHAUST_HEAL: i32 = 1 << 24; // unused
    pub const PACIFIED: i32 = 1 << 25;
    pub const SPELLCOOLDOWN: i32 = 1 << 26;
    pub const SPELLGROUPCOOLDOWN: i32 = 1 << 27;
    pub const ROOT: i32 = 1 << 28;
    pub const MANASHIELD_BREAKABLE: i32 = 1 << 29;
}

// ---------------------------------------------------------------------------
// ConditionId_t — i8 with explicit values
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum ConditionId {
    Default = -1,
    Combat = 0,
    Head = 1,
    Necklace = 2,
    Backpack = 3,
    Armor = 4,
    Right = 5,
    Left = 6,
    Legs = 7,
    Feet = 8,
    Ring = 9,
    Ammo = 10,
}

// ---------------------------------------------------------------------------
// PlayerSex_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlayerSex {
    Female = 0,
    Male = 1,
}

impl PlayerSex {
    pub const LAST: PlayerSex = PlayerSex::Male;
}

/// VOCATION_NONE constant from enums.h
pub const VOCATION_NONE: u16 = 0;

// ---------------------------------------------------------------------------
// ReturnValue
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnValue {
    NoError,
    NotPossible,
    NotEnoughRoom,
    PlayerIsPzLocked,
    PlayerIsNotInvited,
    CannotThrow,
    ThereIsNoWay,
    DestinationOutOfReach,
    CreatureBlock,
    NotMoveable,
    DropTwoHandedItem,
    BothHandsNeedToBeFree,
    CanOnlyUseOneWeapon,
    NeedExchange,
    CannotBeDressed,
    PutThisObjectInYourHand,
    PutThisObjectInBothHands,
    TooFarAway,
    FirstGoDownstairs,
    FirstGoUpstairs,
    ContainerNotEnoughRoom,
    NotEnoughCapacity,
    CannotPickup,
    ThisIsImpossible,
    DepotIsFull,
    CreatureDoesNotExist,
    CannotUseThisObject,
    PlayerWithThisNameIsNotOnline,
    NotRequiredLevelToUseRune,
    YouAreAlreadyTrading,
    ThisPlayerIsAlreadyTrading,
    YouMayNotLogoutDuringAFight,
    DirectPlayerShoot,
    NotEnoughLevel,
    NotEnoughMagicLevel,
    NotEnoughMana,
    NotEnoughSoul,
    YouAreExhausted,
    YouCannotUseObjectsThatFast,
    PlayerIsNotReachable,
    CanOnlyUseThisRuneOnCreatures,
    ActionNotPermittedInProtectionZone,
    YouMayNotAttackThisPlayer,
    YouMayNotAttackAPersonInProtectionZone,
    YouMayNotAttackAPersonWhileInProtectionZone,
    YouMayNotAttackThisCreature,
    YouCanOnlyUseItOnCreatures,
    CreatureIsNotReachable,
    TurnSecureModeToAttackUnmarkedPlayers,
    YouNeedPremiumAccount,
    YouNeedToLearnThisSpell,
    YourVocationCannotUseThisSpell,
    YouNeedAWeaponToUseThisSpell,
    PlayerIsPzLockedLeavePvpZone,
    PlayerIsPzLockedEnterPvpZone,
    ActionNotPermittedInAnoPvpZone,
    YouCannotLogoutHere,
    YouNeedAMagicItemToCastSpell,
    NameIsTooAmbiguous,
    CanOnlyUseOneShield,
    NoPartyMembersInRange,
    YouAreNotTheOwner,
    TradePlayerFarAway,
    YouDontOwnThisHouse,
    TradePlayerAlreadyOwnsAHouse,
    TradePlayerHighestBidder,
    YouCannotTradeThisHouse,
    YouDontHaveRequiredProfession,
    CannotMoveItemIsNotStoreItem,
    ItemCannotBeMovedThere,
    YouCannotUseThisBed,
    QuiverAmmoOnly,
}

// ---------------------------------------------------------------------------
// SpeechBubble_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SpeechBubble {
    None = 0,
    Normal = 1,
    Trade = 2,
    Quest = 3,
    Compass = 4,
    Normal2 = 5,
    Normal3 = 6,
    Hireling = 7,
}

impl SpeechBubble {
    pub const LAST: SpeechBubble = SpeechBubble::Hireling;
}

// ---------------------------------------------------------------------------
// MapMark_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MapMark {
    Tick = 0,
    Question = 1,
    Exclamation = 2,
    Star = 3,
    Cross = 4,
    Temple = 5,
    Kiss = 6,
    Shovel = 7,
    Sword = 8,
    Flag = 9,
    Lock = 10,
    Bag = 11,
    Skull = 12,
    Dollar = 13,
    RedNorth = 14,
    RedSouth = 15,
    RedEast = 16,
    RedWest = 17,
    GreenNorth = 18,
    GreenSouth = 19,
}

// ---------------------------------------------------------------------------
// CombatOrigin
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatOrigin {
    None,
    Condition,
    Spell,
    Melee,
    Ranged,
    Wand,
    Reflect,
}

// ---------------------------------------------------------------------------
// MonstersEvent_t
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MonstersEvent {
    None = 0,
    Think = 1,
    Appear = 2,
    Disappear = 3,
    Move = 4,
    Say = 5,
}

// ---------------------------------------------------------------------------
// ClientDamageType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ClientDamageType {
    Physical = 0,
    Fire = 1,
    Earth = 2,
    Energy = 3,
    Ice = 4,
    Holy = 5,
    Death = 6,
    Healing = 7,
    Drown = 8,
    LifeDrain = 9,
    Undefined = 10,
}

// ---------------------------------------------------------------------------
// DamageAnalyzerImpactType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DamageAnalyzerImpactType {
    Healing = 0,
    Dealt = 1,
    Received = 2,
}

// ---------------------------------------------------------------------------
// Plain-data structs from enums.h
// ---------------------------------------------------------------------------

/// Outfit_t
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Outfit {
    pub look_type: u16,
    pub look_type_ex: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
    pub look_mount: u16,
    pub look_mount_head: u8,
    pub look_mount_body: u8,
    pub look_mount_legs: u8,
    pub look_mount_feet: u8,
}

/// LightInfo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightInfo {
    pub level: u8,
    pub color: u8,
}

impl Default for LightInfo {
    fn default() -> Self {
        LightInfo {
            level: 0,
            color: 215,
        }
    }
}

/// MarketStatistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarketStatistics {
    pub num_transactions: u32,
    pub highest_price: u32,
    pub total_price: u64,
    pub lowest_price: u32,
}

/// Reflect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Reflect {
    pub percent: u16,
    pub chance: u16,
}

impl std::ops::AddAssign for Reflect {
    /// Mirrors C++ `Reflect& operator+=(const Reflect& other)`:
    /// percent accumulates; chance is clamped to `[0, 100]`.
    fn add_assign(&mut self, other: Reflect) {
        self.percent = self.percent.saturating_add(other.percent);
        self.chance = (self.chance as u32 + other.chance as u32).min(100) as u16;
    }
}

impl std::ops::Add for Reflect {
    type Output = Reflect;
    fn add(mut self, other: Reflect) -> Reflect {
        self += other;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- RuleViolationType ---
    #[test]
    fn test_rule_violation_type_variants() {
        assert_eq!(RuleViolationType::Name as u8, 0);
        assert_eq!(RuleViolationType::Statement as u8, 1);
        assert_eq!(RuleViolationType::Bot as u8, 2);
    }

    // --- RuleViolationReason ---
    #[test]
    fn test_rule_violation_reason_discriminants() {
        assert_eq!(RuleViolationReason::NameInappropriate as u8, 0);
        assert_eq!(RuleViolationReason::NamePoorFormatted as u8, 1);
        assert_eq!(RuleViolationReason::NameAdvertising as u8, 2);
        assert_eq!(RuleViolationReason::NameUnfitting as u8, 3);
        assert_eq!(RuleViolationReason::NameRuleViolation as u8, 4);
        assert_eq!(RuleViolationReason::InsultingStatement as u8, 5);
        assert_eq!(RuleViolationReason::Spamming as u8, 6);
        assert_eq!(RuleViolationReason::AdvertisingStatement as u8, 7);
        assert_eq!(RuleViolationReason::UnfittingStatement as u8, 8);
        assert_eq!(RuleViolationReason::LanguageStatement as u8, 9);
        assert_eq!(RuleViolationReason::Disclosure as u8, 10);
        assert_eq!(RuleViolationReason::RuleViolation as u8, 11);
        assert_eq!(RuleViolationReason::StatementBugAbuse as u8, 12);
        assert_eq!(RuleViolationReason::UnofficialSoftware as u8, 13);
        assert_eq!(RuleViolationReason::Pretending as u8, 14);
        assert_eq!(RuleViolationReason::HarassingOwners as u8, 15);
        assert_eq!(RuleViolationReason::FalseInfo as u8, 16);
        assert_eq!(RuleViolationReason::AccountSharing as u8, 17);
        assert_eq!(RuleViolationReason::StealingData as u8, 18);
        assert_eq!(RuleViolationReason::ServiceAttacking as u8, 19);
        assert_eq!(RuleViolationReason::ServiceAgreement as u8, 20);
    }

    // --- ThreadState ---
    #[test]
    fn test_thread_state_discriminants() {
        assert_eq!(ThreadState::Running as u8, 0);
        assert_eq!(ThreadState::Closing as u8, 1);
        assert_eq!(ThreadState::Terminated as u8, 2);
    }

    // --- ItemAttrFlags ---
    #[test]
    fn test_item_attr_flags_values() {
        assert_eq!(ItemAttrFlags::NONE, 0);
        assert_eq!(ItemAttrFlags::ACTIONID, 1 << 0);
        assert_eq!(ItemAttrFlags::UNIQUEID, 1 << 1);
        assert_eq!(ItemAttrFlags::OPENCONTAINER, 1 << 27);
        assert_eq!(ItemAttrFlags::DURATION, 1 << 17); // DURATION_MIN was an alias; use DURATION directly
        assert_eq!(ItemAttrFlags::DURATION_MAX, 1 << 28);
        assert_eq!(ItemAttrFlags::CUSTOM, 1u32 << 31);
    }

    /// Pins every bit-flag in `itemAttrTypes` to its C++ shift offset.
    #[test]
    fn test_item_attr_flags_all_bits_match_cpp() {
        assert_eq!(ItemAttrFlags::DESCRIPTION, 1 << 2);
        assert_eq!(ItemAttrFlags::TEXT, 1 << 3);
        assert_eq!(ItemAttrFlags::DATE, 1 << 4);
        assert_eq!(ItemAttrFlags::WRITER, 1 << 5);
        assert_eq!(ItemAttrFlags::NAME, 1 << 6);
        assert_eq!(ItemAttrFlags::ARTICLE, 1 << 7);
        assert_eq!(ItemAttrFlags::PLURALNAME, 1 << 8);
        assert_eq!(ItemAttrFlags::WEIGHT, 1 << 9);
        assert_eq!(ItemAttrFlags::ATTACK, 1 << 10);
        assert_eq!(ItemAttrFlags::DEFENSE, 1 << 11);
        assert_eq!(ItemAttrFlags::EXTRADEFENSE, 1 << 12);
        assert_eq!(ItemAttrFlags::ARMOR, 1 << 13);
        assert_eq!(ItemAttrFlags::HITCHANCE, 1 << 14);
        assert_eq!(ItemAttrFlags::SHOOTRANGE, 1 << 15);
        assert_eq!(ItemAttrFlags::OWNER, 1 << 16);
        assert_eq!(ItemAttrFlags::DECAYSTATE, 1 << 18);
        assert_eq!(ItemAttrFlags::CORPSEOWNER, 1 << 19);
        assert_eq!(ItemAttrFlags::CHARGES, 1 << 20);
        assert_eq!(ItemAttrFlags::FLUIDTYPE, 1 << 21);
        assert_eq!(ItemAttrFlags::DOORID, 1 << 22);
        assert_eq!(ItemAttrFlags::DECAYTO, 1 << 23);
        assert_eq!(ItemAttrFlags::WRAPID, 1 << 24);
        assert_eq!(ItemAttrFlags::STOREITEM, 1 << 25);
        assert_eq!(ItemAttrFlags::ATTACK_SPEED, 1 << 26);
    }

    // --- VipStatus ---
    #[test]
    fn test_vip_status_discriminants() {
        assert_eq!(VipStatus::Offline as u8, 0);
        assert_eq!(VipStatus::Online as u8, 1);
        assert_eq!(VipStatus::Pending as u8, 2);
        assert_eq!(VipStatus::Training as u8, 3);
    }

    // --- MarketAction ---
    #[test]
    fn test_market_action_discriminants() {
        assert_eq!(MarketAction::Buy as i32, 0);
        assert_eq!(MarketAction::Sell as i32, 1);
    }

    // --- MarketRequest ---
    #[test]
    fn test_market_request_discriminants() {
        assert_eq!(MarketRequest::OwnHistory as i32, 1);
        assert_eq!(MarketRequest::OwnOffers as i32, 2);
        assert_eq!(MarketRequest::Item as i32, 3);
    }

    // --- MarketOfferState ---
    #[test]
    fn test_market_offer_state_discriminants() {
        assert_eq!(MarketOfferState::Active as i32, 0);
        assert_eq!(MarketOfferState::Cancelled as i32, 1);
        assert_eq!(MarketOfferState::Expired as i32, 2);
        assert_eq!(MarketOfferState::Accepted as i32, 3);
        assert_eq!(MarketOfferState::AcceptedEx as i32, 255);
    }

    // --- ChannelEvent ---
    #[test]
    fn test_channel_event_discriminants() {
        assert_eq!(ChannelEvent::Join as u8, 0);
        assert_eq!(ChannelEvent::Leave as u8, 1);
        assert_eq!(ChannelEvent::Invite as u8, 2);
        assert_eq!(ChannelEvent::Exclude as u8, 3);
    }

    // --- CreatureType ---
    #[test]
    fn test_creature_type_discriminants() {
        assert_eq!(CreatureType::Player as u8, 0);
        assert_eq!(CreatureType::Monster as u8, 1);
        assert_eq!(CreatureType::Npc as u8, 2);
        assert_eq!(CreatureType::SummonOwn as u8, 3);
        assert_eq!(CreatureType::SummonOthers as u8, 4);
        assert_eq!(CreatureType::Hidden as u8, 5);
    }

    // --- OperatingSystem ---
    #[test]
    fn test_operating_system_discriminants() {
        assert_eq!(OperatingSystem::None as u8, 0);
        assert_eq!(OperatingSystem::Linux as u8, 1);
        assert_eq!(OperatingSystem::Windows as u8, 2);
        assert_eq!(OperatingSystem::Flash as u8, 3);
        assert_eq!(OperatingSystem::QtLinux as u8, 4);
        assert_eq!(OperatingSystem::QtWindows as u8, 5);
        assert_eq!(OperatingSystem::QtMac as u8, 6);
        assert_eq!(OperatingSystem::QtLinux2 as u8, 7);
        assert_eq!(OperatingSystem::OtClientLinux as u8, 10);
        assert_eq!(OperatingSystem::OtClientWindows as u8, 11);
        assert_eq!(OperatingSystem::OtClientMac as u8, 12);
    }

    // --- SpellGroup ---
    #[test]
    fn test_spell_group_discriminants() {
        assert_eq!(SpellGroup::None as u8, 0);
        assert_eq!(SpellGroup::Attack as u8, 1);
        assert_eq!(SpellGroup::Healing as u8, 2);
        assert_eq!(SpellGroup::Support as u8, 3);
        assert_eq!(SpellGroup::Special as u8, 4);
        // C++ pin SPELLGROUP_CONJURE = 5 is commented-out; no Rust variant.
        assert_eq!(SpellGroup::Crippling as u8, 6);
        assert_eq!(SpellGroup::Focus as u8, 7);
        assert_eq!(SpellGroup::UltimateStrikes as u8, 8);
    }

    // --- SpellType ---
    #[test]
    fn test_spell_type_discriminants() {
        assert_eq!(SpellType::Undefined as u8, 0);
        assert_eq!(SpellType::Instant as u8, 1);
        assert_eq!(SpellType::Rune as u8, 2);
    }

    // --- AccountType ---
    #[test]
    fn test_account_type_discriminants() {
        assert_eq!(AccountType::Normal as u8, 1);
        assert_eq!(AccountType::Tutor as u8, 2);
        assert_eq!(AccountType::SeniorTutor as u8, 3);
        assert_eq!(AccountType::GameMaster as u8, 4);
        assert_eq!(AccountType::CommunityManager as u8, 5);
        assert_eq!(AccountType::God as u8, 6);
    }

    // --- RaceType ---
    /// C++ `RaceType_t` is unpinned; values are inferred from order
    /// (RACE_NONE = 0, RACE_VENOM = 1, ..., RACE_INK = 6).
    #[test]
    fn test_race_type_discriminants() {
        assert_eq!(RaceType::None as u8, 0);
        assert_eq!(RaceType::Venom as u8, 1);
        assert_eq!(RaceType::Blood as u8, 2);
        assert_eq!(RaceType::Undead as u8, 3);
        assert_eq!(RaceType::Fire as u8, 4);
        assert_eq!(RaceType::Energy as u8, 5);
        assert_eq!(RaceType::Ink as u8, 6);
    }

    // --- CombatTypeFlags ---
    #[test]
    fn test_combat_type_flags() {
        assert_eq!(CombatTypeFlags::NONE, 0);
        assert_eq!(CombatTypeFlags::PHYSICAL, 1 << 0);
        assert_eq!(CombatTypeFlags::DEATH, 1 << 11);
        assert_eq!(CombatTypeFlags::COUNT, 12);
    }

    /// Pins every bit-flag in `CombatType_t` to the C++ shift offset.
    #[test]
    fn test_combat_type_flags_all_bits_match_cpp() {
        assert_eq!(CombatTypeFlags::ENERGY, 1 << 1);
        assert_eq!(CombatTypeFlags::EARTH, 1 << 2);
        assert_eq!(CombatTypeFlags::FIRE, 1 << 3);
        assert_eq!(CombatTypeFlags::UNDEFINED, 1 << 4);
        assert_eq!(CombatTypeFlags::LIFEDRAIN, 1 << 5);
        assert_eq!(CombatTypeFlags::MANADRAIN, 1 << 6);
        assert_eq!(CombatTypeFlags::HEALING, 1 << 7);
        assert_eq!(CombatTypeFlags::DROWN, 1 << 8);
        assert_eq!(CombatTypeFlags::ICE, 1 << 9);
        assert_eq!(CombatTypeFlags::HOLY, 1 << 10);
    }

    // --- CombatParam ---
    /// `CombatParam_t` has no explicit C++ discriminants, so the variants are
    /// implicitly numbered 0..=9 in declaration order. Exercise pattern-match
    /// dispatch on every variant.
    #[test]
    fn test_combat_param_all_variants_dispatch() {
        fn name(p: CombatParam) -> &'static str {
            match p {
                CombatParam::Type => "type",
                CombatParam::Effect => "effect",
                CombatParam::DistanceEffect => "distance_effect",
                CombatParam::BlockShield => "block_shield",
                CombatParam::BlockArmor => "block_armor",
                CombatParam::TargetCasterOrTopmost => "target_caster_or_topmost",
                CombatParam::CreateItem => "create_item",
                CombatParam::Aggressive => "aggressive",
                CombatParam::Dispel => "dispel",
                CombatParam::UseCharges => "use_charges",
            }
        }
        assert_eq!(name(CombatParam::Type), "type");
        assert_eq!(name(CombatParam::Effect), "effect");
        assert_eq!(name(CombatParam::DistanceEffect), "distance_effect");
        assert_eq!(name(CombatParam::BlockShield), "block_shield");
        assert_eq!(name(CombatParam::BlockArmor), "block_armor");
        assert_eq!(
            name(CombatParam::TargetCasterOrTopmost),
            "target_caster_or_topmost"
        );
        assert_eq!(name(CombatParam::CreateItem), "create_item");
        assert_eq!(name(CombatParam::Aggressive), "aggressive");
        assert_eq!(name(CombatParam::Dispel), "dispel");
        assert_eq!(name(CombatParam::UseCharges), "use_charges");
    }

    // --- CallBackParam ---
    /// `CallBackParam_t` unpinned; exercise pattern-match dispatch on every variant.
    #[test]
    fn test_callback_param_all_variants_dispatch() {
        fn name(p: CallBackParam) -> &'static str {
            match p {
                CallBackParam::LevelMagicValue => "level_magic_value",
                CallBackParam::SkillValue => "skill_value",
                CallBackParam::TargetTile => "target_tile",
                CallBackParam::TargetCreature => "target_creature",
            }
        }
        assert_eq!(name(CallBackParam::LevelMagicValue), "level_magic_value");
        assert_eq!(name(CallBackParam::SkillValue), "skill_value");
        assert_eq!(name(CallBackParam::TargetTile), "target_tile");
        assert_eq!(name(CallBackParam::TargetCreature), "target_creature");
    }

    // --- ConditionParam discriminants ---
    #[test]
    fn test_condition_param_discriminants() {
        assert_eq!(ConditionParam::Owner as i32, 1);
        assert_eq!(ConditionParam::Ticks as i32, 2);
        assert_eq!(ConditionParam::ManaShieldBreakable as i32, 56);
        // Check skipped values
        assert_eq!(ConditionParam::StatMagicPoints as i32, 30); // skips 29
        assert_eq!(ConditionParam::StatMagicPointsPercent as i32, 34); // skips 33
    }

    /// Pin every `ConditionParam_t` variant to its C++ integer value. Several
    /// values are skipped in C++ (3 OUTFIT, 29 STAT_SOULPOINTS, 33 STAT_SOULPOINTSPERCENT)
    /// and are intentionally absent here.
    #[test]
    fn test_condition_param_all_discriminants_match_cpp() {
        assert_eq!(ConditionParam::HealthGain as i32, 4);
        assert_eq!(ConditionParam::HealthTicks as i32, 5);
        assert_eq!(ConditionParam::ManaGain as i32, 6);
        assert_eq!(ConditionParam::ManaTicks as i32, 7);
        assert_eq!(ConditionParam::Delayed as i32, 8);
        assert_eq!(ConditionParam::Speed as i32, 9);
        assert_eq!(ConditionParam::LightLevel as i32, 10);
        assert_eq!(ConditionParam::LightColor as i32, 11);
        assert_eq!(ConditionParam::SoulGain as i32, 12);
        assert_eq!(ConditionParam::SoulTicks as i32, 13);
        assert_eq!(ConditionParam::MinValue as i32, 14);
        assert_eq!(ConditionParam::MaxValue as i32, 15);
        assert_eq!(ConditionParam::StartValue as i32, 16);
        assert_eq!(ConditionParam::TickInterval as i32, 17);
        assert_eq!(ConditionParam::ForceUpdate as i32, 18);
        assert_eq!(ConditionParam::SkillMelee as i32, 19);
        assert_eq!(ConditionParam::SkillFist as i32, 20);
        assert_eq!(ConditionParam::SkillClub as i32, 21);
        assert_eq!(ConditionParam::SkillSword as i32, 22);
        assert_eq!(ConditionParam::SkillAxe as i32, 23);
        assert_eq!(ConditionParam::SkillDistance as i32, 24);
        assert_eq!(ConditionParam::SkillShield as i32, 25);
        assert_eq!(ConditionParam::SkillFishing as i32, 26);
        assert_eq!(ConditionParam::StatMaxHitPoints as i32, 27);
        assert_eq!(ConditionParam::StatMaxManaPoints as i32, 28);
        assert_eq!(ConditionParam::StatMaxHitPointsPercent as i32, 31);
        assert_eq!(ConditionParam::StatMaxManaPointsPercent as i32, 32);
        assert_eq!(ConditionParam::PeriodicDamage as i32, 35);
        assert_eq!(ConditionParam::SkillMeleePercent as i32, 36);
        assert_eq!(ConditionParam::SkillFistPercent as i32, 37);
        assert_eq!(ConditionParam::SkillClubPercent as i32, 38);
        assert_eq!(ConditionParam::SkillSwordPercent as i32, 39);
        assert_eq!(ConditionParam::SkillAxePercent as i32, 40);
        assert_eq!(ConditionParam::SkillDistancePercent as i32, 41);
        assert_eq!(ConditionParam::SkillShieldPercent as i32, 42);
        assert_eq!(ConditionParam::SkillFishingPercent as i32, 43);
        assert_eq!(ConditionParam::BuffSpell as i32, 44);
        assert_eq!(ConditionParam::SubId as i32, 45);
        assert_eq!(ConditionParam::Field as i32, 46);
        assert_eq!(ConditionParam::DisableDefense as i32, 47);
        assert_eq!(ConditionParam::SpecialSkillCriticalHitChance as i32, 48);
        assert_eq!(ConditionParam::SpecialSkillCriticalHitAmount as i32, 49);
        assert_eq!(ConditionParam::SpecialSkillLifeLeechChance as i32, 50);
        assert_eq!(ConditionParam::SpecialSkillLifeLeechAmount as i32, 51);
        assert_eq!(ConditionParam::SpecialSkillManaLeechChance as i32, 52);
        assert_eq!(ConditionParam::SpecialSkillManaLeechAmount as i32, 53);
        assert_eq!(ConditionParam::Aggressive as i32, 54);
        assert_eq!(ConditionParam::Drunkenness as i32, 55);
    }

    // --- BlockType ---
    #[test]
    fn test_block_type_discriminants() {
        assert_eq!(BlockType::None as u8, 0);
        assert_eq!(BlockType::Defense as u8, 1);
        assert_eq!(BlockType::Armor as u8, 2);
        assert_eq!(BlockType::Immunity as u8, 3);
    }

    // --- Skill ---
    #[test]
    fn test_skill_discriminants() {
        assert_eq!(Skill::Fist as u8, 0);
        assert_eq!(Skill::Fishing as u8, 6);
        assert_eq!(Skill::MagLevel as u8, 7);
        assert_eq!(Skill::Level as u8, 8);
        assert_eq!(Skill::FIRST as u8, 0);
        assert_eq!(Skill::LAST as u8, 6);
    }

    // --- Stat ---
    /// `stats_t` has no explicit C++ discriminants. Pattern-match dispatch
    /// over every variant to ensure none can be silently removed.
    #[test]
    fn test_stat_all_variants_dispatch() {
        fn name(s: Stat) -> &'static str {
            match s {
                Stat::MaxHitPoints => "max_hit_points",
                Stat::MaxManaPoints => "max_mana_points",
                Stat::SoulPoints => "soul_points",
                Stat::MagicPoints => "magic_points",
            }
        }
        assert_eq!(name(Stat::MaxHitPoints), "max_hit_points");
        assert_eq!(name(Stat::MaxManaPoints), "max_mana_points");
        assert_eq!(name(Stat::SoulPoints), "soul_points");
        assert_eq!(name(Stat::MagicPoints), "magic_points");
        // STAT_FIRST/LAST in C++ alias MaxHitPoints/MagicPoints respectively.
        assert_eq!(Stat::FIRST, Stat::MaxHitPoints);
        assert_eq!(Stat::LAST, Stat::MagicPoints);
    }

    // --- SpecialSkill ---
    /// `SpecialSkills_t` has no explicit C++ discriminants.
    /// Pattern-match dispatch every variant.
    #[test]
    fn test_special_skill_all_variants_dispatch() {
        fn name(s: SpecialSkill) -> &'static str {
            match s {
                SpecialSkill::CriticalHitChance => "critical_hit_chance",
                SpecialSkill::CriticalHitAmount => "critical_hit_amount",
                SpecialSkill::LifeLeechChance => "life_leech_chance",
                SpecialSkill::LifeLeechAmount => "life_leech_amount",
                SpecialSkill::ManaLeechChance => "mana_leech_chance",
                SpecialSkill::ManaLeechAmount => "mana_leech_amount",
            }
        }
        assert_eq!(name(SpecialSkill::CriticalHitChance), "critical_hit_chance");
        assert_eq!(name(SpecialSkill::CriticalHitAmount), "critical_hit_amount");
        assert_eq!(name(SpecialSkill::LifeLeechChance), "life_leech_chance");
        assert_eq!(name(SpecialSkill::LifeLeechAmount), "life_leech_amount");
        assert_eq!(name(SpecialSkill::ManaLeechChance), "mana_leech_chance");
        assert_eq!(name(SpecialSkill::ManaLeechAmount), "mana_leech_amount");
        assert_eq!(SpecialSkill::FIRST, SpecialSkill::CriticalHitChance);
        assert_eq!(SpecialSkill::LAST, SpecialSkill::ManaLeechAmount);
    }

    // --- FormulaType ---
    /// `formulaType_t` is unpinned; pattern-match dispatch over every variant.
    #[test]
    fn test_formula_type_all_variants_dispatch() {
        fn name(f: FormulaType) -> &'static str {
            match f {
                FormulaType::Undefined => "undefined",
                FormulaType::LevelMagic => "level_magic",
                FormulaType::Skill => "skill",
                FormulaType::Damage => "damage",
            }
        }
        assert_eq!(name(FormulaType::Undefined), "undefined");
        assert_eq!(name(FormulaType::LevelMagic), "level_magic");
        assert_eq!(name(FormulaType::Skill), "skill");
        assert_eq!(name(FormulaType::Damage), "damage");
    }

    // --- ConditionTypeFlags ---
    #[test]
    fn test_condition_type_flags() {
        assert_eq!(ConditionTypeFlags::NONE, 0);
        assert_eq!(ConditionTypeFlags::POISON, 1 << 0);
        assert_eq!(ConditionTypeFlags::ROOT, 1 << 28);
        assert_eq!(ConditionTypeFlags::MANASHIELD_BREAKABLE, 1 << 29);
    }

    /// Pins every bit-flag in `ConditionType_t` to the C++ shift offset.
    #[test]
    fn test_condition_type_flags_all_bits_match_cpp() {
        assert_eq!(ConditionTypeFlags::FIRE, 1 << 1);
        assert_eq!(ConditionTypeFlags::ENERGY, 1 << 2);
        assert_eq!(ConditionTypeFlags::BLEEDING, 1 << 3);
        assert_eq!(ConditionTypeFlags::HASTE, 1 << 4);
        assert_eq!(ConditionTypeFlags::PARALYZE, 1 << 5);
        assert_eq!(ConditionTypeFlags::OUTFIT, 1 << 6);
        assert_eq!(ConditionTypeFlags::INVISIBLE, 1 << 7);
        assert_eq!(ConditionTypeFlags::LIGHT, 1 << 8);
        assert_eq!(ConditionTypeFlags::MANASHIELD, 1 << 9);
        assert_eq!(ConditionTypeFlags::INFIGHT, 1 << 10);
        assert_eq!(ConditionTypeFlags::DRUNK, 1 << 11);
        assert_eq!(ConditionTypeFlags::EXHAUST_WEAPON, 1 << 12);
        assert_eq!(ConditionTypeFlags::REGENERATION, 1 << 13);
        assert_eq!(ConditionTypeFlags::SOUL, 1 << 14);
        assert_eq!(ConditionTypeFlags::DROWN, 1 << 15);
        assert_eq!(ConditionTypeFlags::MUTED, 1 << 16);
        assert_eq!(ConditionTypeFlags::CHANNELMUTEDTICKS, 1 << 17);
        assert_eq!(ConditionTypeFlags::YELLTICKS, 1 << 18);
        assert_eq!(ConditionTypeFlags::ATTRIBUTES, 1 << 19);
        assert_eq!(ConditionTypeFlags::FREEZING, 1 << 20);
        assert_eq!(ConditionTypeFlags::DAZZLED, 1 << 21);
        assert_eq!(ConditionTypeFlags::CURSED, 1 << 22);
        assert_eq!(ConditionTypeFlags::EXHAUST_COMBAT, 1 << 23);
        assert_eq!(ConditionTypeFlags::EXHAUST_HEAL, 1 << 24);
        assert_eq!(ConditionTypeFlags::PACIFIED, 1 << 25);
        assert_eq!(ConditionTypeFlags::SPELLCOOLDOWN, 1 << 26);
        assert_eq!(ConditionTypeFlags::SPELLGROUPCOOLDOWN, 1 << 27);
    }

    // --- ConditionId discriminants ---
    #[test]
    fn test_condition_id_discriminants() {
        assert_eq!(ConditionId::Default as i8, -1);
        assert_eq!(ConditionId::Combat as i8, 0);
        assert_eq!(ConditionId::Head as i8, 1);
        assert_eq!(ConditionId::Necklace as i8, 2);
        assert_eq!(ConditionId::Backpack as i8, 3);
        assert_eq!(ConditionId::Armor as i8, 4);
        assert_eq!(ConditionId::Right as i8, 5);
        assert_eq!(ConditionId::Left as i8, 6);
        assert_eq!(ConditionId::Legs as i8, 7);
        assert_eq!(ConditionId::Feet as i8, 8);
        assert_eq!(ConditionId::Ring as i8, 9);
        assert_eq!(ConditionId::Ammo as i8, 10);
    }

    // --- PlayerSex ---
    #[test]
    fn test_player_sex_discriminants() {
        assert_eq!(PlayerSex::Female as u8, 0);
        assert_eq!(PlayerSex::Male as u8, 1);
        assert_eq!(PlayerSex::LAST as u8, 1);
    }

    // --- VOCATION_NONE constant ---
    #[test]
    fn test_vocation_none() {
        assert_eq!(VOCATION_NONE, 0u16);
    }

    // --- ReturnValue variants exist ---
    #[test]
    fn test_return_value_variants_exist() {
        let _ = ReturnValue::NoError;
        let _ = ReturnValue::NotPossible;
        let _ = ReturnValue::QuiverAmmoOnly;
        assert_eq!(ReturnValue::NoError, ReturnValue::NoError);
    }

    /// `ReturnValue` is unpinned in C++. Exhaustive pattern-match dispatch
    /// ensures every variant exists and is reachable; the compiler enforces
    /// completeness via the absence of a wildcard arm.
    #[test]
    fn test_return_value_all_variants_dispatch() {
        fn ord(rv: ReturnValue) -> u32 {
            match rv {
                ReturnValue::NoError => 0,
                ReturnValue::NotPossible => 1,
                ReturnValue::NotEnoughRoom => 2,
                ReturnValue::PlayerIsPzLocked => 3,
                ReturnValue::PlayerIsNotInvited => 4,
                ReturnValue::CannotThrow => 5,
                ReturnValue::ThereIsNoWay => 6,
                ReturnValue::DestinationOutOfReach => 7,
                ReturnValue::CreatureBlock => 8,
                ReturnValue::NotMoveable => 9,
                ReturnValue::DropTwoHandedItem => 10,
                ReturnValue::BothHandsNeedToBeFree => 11,
                ReturnValue::CanOnlyUseOneWeapon => 12,
                ReturnValue::NeedExchange => 13,
                ReturnValue::CannotBeDressed => 14,
                ReturnValue::PutThisObjectInYourHand => 15,
                ReturnValue::PutThisObjectInBothHands => 16,
                ReturnValue::TooFarAway => 17,
                ReturnValue::FirstGoDownstairs => 18,
                ReturnValue::FirstGoUpstairs => 19,
                ReturnValue::ContainerNotEnoughRoom => 20,
                ReturnValue::NotEnoughCapacity => 21,
                ReturnValue::CannotPickup => 22,
                ReturnValue::ThisIsImpossible => 23,
                ReturnValue::DepotIsFull => 24,
                ReturnValue::CreatureDoesNotExist => 25,
                ReturnValue::CannotUseThisObject => 26,
                ReturnValue::PlayerWithThisNameIsNotOnline => 27,
                ReturnValue::NotRequiredLevelToUseRune => 28,
                ReturnValue::YouAreAlreadyTrading => 29,
                ReturnValue::ThisPlayerIsAlreadyTrading => 30,
                ReturnValue::YouMayNotLogoutDuringAFight => 31,
                ReturnValue::DirectPlayerShoot => 32,
                ReturnValue::NotEnoughLevel => 33,
                ReturnValue::NotEnoughMagicLevel => 34,
                ReturnValue::NotEnoughMana => 35,
                ReturnValue::NotEnoughSoul => 36,
                ReturnValue::YouAreExhausted => 37,
                ReturnValue::YouCannotUseObjectsThatFast => 38,
                ReturnValue::PlayerIsNotReachable => 39,
                ReturnValue::CanOnlyUseThisRuneOnCreatures => 40,
                ReturnValue::ActionNotPermittedInProtectionZone => 41,
                ReturnValue::YouMayNotAttackThisPlayer => 42,
                ReturnValue::YouMayNotAttackAPersonInProtectionZone => 43,
                ReturnValue::YouMayNotAttackAPersonWhileInProtectionZone => 44,
                ReturnValue::YouMayNotAttackThisCreature => 45,
                ReturnValue::YouCanOnlyUseItOnCreatures => 46,
                ReturnValue::CreatureIsNotReachable => 47,
                ReturnValue::TurnSecureModeToAttackUnmarkedPlayers => 48,
                ReturnValue::YouNeedPremiumAccount => 49,
                ReturnValue::YouNeedToLearnThisSpell => 50,
                ReturnValue::YourVocationCannotUseThisSpell => 51,
                ReturnValue::YouNeedAWeaponToUseThisSpell => 52,
                ReturnValue::PlayerIsPzLockedLeavePvpZone => 53,
                ReturnValue::PlayerIsPzLockedEnterPvpZone => 54,
                ReturnValue::ActionNotPermittedInAnoPvpZone => 55,
                ReturnValue::YouCannotLogoutHere => 56,
                ReturnValue::YouNeedAMagicItemToCastSpell => 57,
                ReturnValue::NameIsTooAmbiguous => 58,
                ReturnValue::CanOnlyUseOneShield => 59,
                ReturnValue::NoPartyMembersInRange => 60,
                ReturnValue::YouAreNotTheOwner => 61,
                ReturnValue::TradePlayerFarAway => 62,
                ReturnValue::YouDontOwnThisHouse => 63,
                ReturnValue::TradePlayerAlreadyOwnsAHouse => 64,
                ReturnValue::TradePlayerHighestBidder => 65,
                ReturnValue::YouCannotTradeThisHouse => 66,
                ReturnValue::YouDontHaveRequiredProfession => 67,
                ReturnValue::CannotMoveItemIsNotStoreItem => 68,
                ReturnValue::ItemCannotBeMovedThere => 69,
                ReturnValue::YouCannotUseThisBed => 70,
                ReturnValue::QuiverAmmoOnly => 71,
            }
        }
        // Spot-check a few from across the range to ensure dispatch works.
        assert_eq!(ord(ReturnValue::NoError), 0);
        assert_eq!(ord(ReturnValue::DropTwoHandedItem), 10);
        assert_eq!(ord(ReturnValue::DirectPlayerShoot), 32);
        assert_eq!(ord(ReturnValue::TurnSecureModeToAttackUnmarkedPlayers), 48);
        assert_eq!(ord(ReturnValue::QuiverAmmoOnly), 71);
        // Verify each distinct variant produces a distinct ordinal — collect all
        // and check the cardinality.
        let all = [
            ReturnValue::NoError,
            ReturnValue::NotPossible,
            ReturnValue::NotEnoughRoom,
            ReturnValue::PlayerIsPzLocked,
            ReturnValue::PlayerIsNotInvited,
            ReturnValue::CannotThrow,
            ReturnValue::ThereIsNoWay,
            ReturnValue::DestinationOutOfReach,
            ReturnValue::CreatureBlock,
            ReturnValue::NotMoveable,
            ReturnValue::DropTwoHandedItem,
            ReturnValue::BothHandsNeedToBeFree,
            ReturnValue::CanOnlyUseOneWeapon,
            ReturnValue::NeedExchange,
            ReturnValue::CannotBeDressed,
            ReturnValue::PutThisObjectInYourHand,
            ReturnValue::PutThisObjectInBothHands,
            ReturnValue::TooFarAway,
            ReturnValue::FirstGoDownstairs,
            ReturnValue::FirstGoUpstairs,
            ReturnValue::ContainerNotEnoughRoom,
            ReturnValue::NotEnoughCapacity,
            ReturnValue::CannotPickup,
            ReturnValue::ThisIsImpossible,
            ReturnValue::DepotIsFull,
            ReturnValue::CreatureDoesNotExist,
            ReturnValue::CannotUseThisObject,
            ReturnValue::PlayerWithThisNameIsNotOnline,
            ReturnValue::NotRequiredLevelToUseRune,
            ReturnValue::YouAreAlreadyTrading,
            ReturnValue::ThisPlayerIsAlreadyTrading,
            ReturnValue::YouMayNotLogoutDuringAFight,
            ReturnValue::DirectPlayerShoot,
            ReturnValue::NotEnoughLevel,
            ReturnValue::NotEnoughMagicLevel,
            ReturnValue::NotEnoughMana,
            ReturnValue::NotEnoughSoul,
            ReturnValue::YouAreExhausted,
            ReturnValue::YouCannotUseObjectsThatFast,
            ReturnValue::PlayerIsNotReachable,
            ReturnValue::CanOnlyUseThisRuneOnCreatures,
            ReturnValue::ActionNotPermittedInProtectionZone,
            ReturnValue::YouMayNotAttackThisPlayer,
            ReturnValue::YouMayNotAttackAPersonInProtectionZone,
            ReturnValue::YouMayNotAttackAPersonWhileInProtectionZone,
            ReturnValue::YouMayNotAttackThisCreature,
            ReturnValue::YouCanOnlyUseItOnCreatures,
            ReturnValue::CreatureIsNotReachable,
            ReturnValue::TurnSecureModeToAttackUnmarkedPlayers,
            ReturnValue::YouNeedPremiumAccount,
            ReturnValue::YouNeedToLearnThisSpell,
            ReturnValue::YourVocationCannotUseThisSpell,
            ReturnValue::YouNeedAWeaponToUseThisSpell,
            ReturnValue::PlayerIsPzLockedLeavePvpZone,
            ReturnValue::PlayerIsPzLockedEnterPvpZone,
            ReturnValue::ActionNotPermittedInAnoPvpZone,
            ReturnValue::YouCannotLogoutHere,
            ReturnValue::YouNeedAMagicItemToCastSpell,
            ReturnValue::NameIsTooAmbiguous,
            ReturnValue::CanOnlyUseOneShield,
            ReturnValue::NoPartyMembersInRange,
            ReturnValue::YouAreNotTheOwner,
            ReturnValue::TradePlayerFarAway,
            ReturnValue::YouDontOwnThisHouse,
            ReturnValue::TradePlayerAlreadyOwnsAHouse,
            ReturnValue::TradePlayerHighestBidder,
            ReturnValue::YouCannotTradeThisHouse,
            ReturnValue::YouDontHaveRequiredProfession,
            ReturnValue::CannotMoveItemIsNotStoreItem,
            ReturnValue::ItemCannotBeMovedThere,
            ReturnValue::YouCannotUseThisBed,
            ReturnValue::QuiverAmmoOnly,
        ];
        let ords: Vec<u32> = all.iter().copied().map(ord).collect();
        let mut sorted = ords.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ords.len(),
            "all ReturnValue variants must be distinct"
        );
        assert_eq!(ords.len(), 72);
    }

    // --- SpeechBubble ---
    #[test]
    fn test_speech_bubble_discriminants() {
        assert_eq!(SpeechBubble::None as i32, 0);
        assert_eq!(SpeechBubble::Normal as i32, 1);
        assert_eq!(SpeechBubble::Trade as i32, 2);
        assert_eq!(SpeechBubble::Quest as i32, 3);
        assert_eq!(SpeechBubble::Compass as i32, 4);
        assert_eq!(SpeechBubble::Normal2 as i32, 5);
        assert_eq!(SpeechBubble::Normal3 as i32, 6);
        assert_eq!(SpeechBubble::Hireling as i32, 7);
        assert_eq!(SpeechBubble::LAST as i32, 7);
    }

    // --- MapMark ---
    #[test]
    fn test_map_mark_discriminants() {
        assert_eq!(MapMark::Tick as i32, 0);
        assert_eq!(MapMark::Question as i32, 1);
        assert_eq!(MapMark::Exclamation as i32, 2);
        assert_eq!(MapMark::Star as i32, 3);
        assert_eq!(MapMark::Cross as i32, 4);
        assert_eq!(MapMark::Temple as i32, 5);
        assert_eq!(MapMark::Kiss as i32, 6);
        assert_eq!(MapMark::Shovel as i32, 7);
        assert_eq!(MapMark::Sword as i32, 8);
        assert_eq!(MapMark::Flag as i32, 9);
        assert_eq!(MapMark::Lock as i32, 10);
        assert_eq!(MapMark::Bag as i32, 11);
        assert_eq!(MapMark::Skull as i32, 12);
        assert_eq!(MapMark::Dollar as i32, 13);
        assert_eq!(MapMark::RedNorth as i32, 14);
        assert_eq!(MapMark::RedSouth as i32, 15);
        assert_eq!(MapMark::RedEast as i32, 16);
        assert_eq!(MapMark::RedWest as i32, 17);
        assert_eq!(MapMark::GreenNorth as i32, 18);
        assert_eq!(MapMark::GreenSouth as i32, 19);
    }

    // --- CombatOrigin ---
    /// `CombatOrigin` is unpinned in C++. Pattern-match dispatch over every variant.
    #[test]
    fn test_combat_origin_all_variants_dispatch() {
        fn name(o: CombatOrigin) -> &'static str {
            match o {
                CombatOrigin::None => "none",
                CombatOrigin::Condition => "condition",
                CombatOrigin::Spell => "spell",
                CombatOrigin::Melee => "melee",
                CombatOrigin::Ranged => "ranged",
                CombatOrigin::Wand => "wand",
                CombatOrigin::Reflect => "reflect",
            }
        }
        assert_eq!(name(CombatOrigin::None), "none");
        assert_eq!(name(CombatOrigin::Condition), "condition");
        assert_eq!(name(CombatOrigin::Spell), "spell");
        assert_eq!(name(CombatOrigin::Melee), "melee");
        assert_eq!(name(CombatOrigin::Ranged), "ranged");
        assert_eq!(name(CombatOrigin::Wand), "wand");
        assert_eq!(name(CombatOrigin::Reflect), "reflect");
    }

    // --- MonstersEvent ---
    #[test]
    fn test_monsters_event_discriminants() {
        assert_eq!(MonstersEvent::None as u8, 0);
        assert_eq!(MonstersEvent::Think as u8, 1);
        assert_eq!(MonstersEvent::Appear as u8, 2);
        assert_eq!(MonstersEvent::Disappear as u8, 3);
        assert_eq!(MonstersEvent::Move as u8, 4);
        assert_eq!(MonstersEvent::Say as u8, 5);
    }

    // --- ClientDamageType ---
    #[test]
    fn test_client_damage_type_discriminants() {
        assert_eq!(ClientDamageType::Physical as i32, 0);
        assert_eq!(ClientDamageType::Fire as i32, 1);
        assert_eq!(ClientDamageType::Earth as i32, 2);
        assert_eq!(ClientDamageType::Energy as i32, 3);
        assert_eq!(ClientDamageType::Ice as i32, 4);
        assert_eq!(ClientDamageType::Holy as i32, 5);
        assert_eq!(ClientDamageType::Death as i32, 6);
        assert_eq!(ClientDamageType::Healing as i32, 7);
        assert_eq!(ClientDamageType::Drown as i32, 8);
        assert_eq!(ClientDamageType::LifeDrain as i32, 9);
        assert_eq!(ClientDamageType::Undefined as i32, 10);
    }

    // --- DamageAnalyzerImpactType ---
    #[test]
    fn test_damage_analyzer_impact_type_discriminants() {
        assert_eq!(DamageAnalyzerImpactType::Healing as i32, 0);
        assert_eq!(DamageAnalyzerImpactType::Dealt as i32, 1);
        assert_eq!(DamageAnalyzerImpactType::Received as i32, 2);
    }

    // --- Structs ---
    #[test]
    fn test_outfit_default() {
        let o = Outfit::default();
        assert_eq!(o.look_type, 0);
        assert_eq!(o.look_mount, 0);
    }

    #[test]
    fn test_light_info_default_color() {
        let l = LightInfo::default();
        assert_eq!(l.level, 0);
        assert_eq!(l.color, 215);
    }

    #[test]
    fn test_market_statistics_default() {
        let ms = MarketStatistics::default();
        assert_eq!(ms.num_transactions, 0);
        assert_eq!(ms.total_price, 0);
    }

    #[test]
    fn test_reflect_default() {
        let r = Reflect::default();
        assert_eq!(r.percent, 0);
        assert_eq!(r.chance, 0);
    }
}
