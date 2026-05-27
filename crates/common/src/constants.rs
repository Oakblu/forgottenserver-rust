//! Migrated from forgottenserver/src/const.h
//! Maps integer game constants and associated enums.

#![allow(dead_code)]
#![allow(non_camel_case_types)]

// ---------------------------------------------------------------------------
// Integer constants
// ---------------------------------------------------------------------------

pub const NETWORKMESSAGE_MAXSIZE: i32 = 24590;
pub const MIN_MARKET_FEE: i32 = 20;
pub const MAX_MARKET_FEE: i32 = 100000;

pub const CHANNEL_GUILD: i32 = 0x00;
pub const CHANNEL_PARTY: i32 = 0x01;
pub const CHANNEL_PRIVATE: i32 = 0xFFFF;

pub const PSTRG_RESERVED_RANGE_START: i32 = 10000000;
pub const PSTRG_RESERVED_RANGE_SIZE: i32 = 10000000;

// ---------------------------------------------------------------------------
// Fluid lookup tables (const arrays from const.h)
// ---------------------------------------------------------------------------

/// reverseFluidMap from const.h – maps server fluid index → FluidTypes_t value
pub const REVERSE_FLUID_MAP: [u8; 11] = [
    0, // FLUID_EMPTY
    1, // FLUID_WATER  (FLUID_BLUE)
    7, // FLUID_MANA   (FLUID_PURPLE)
    3, // FLUID_BEER   (FLUID_BROWN)
    0, // FLUID_EMPTY
    2, // FLUID_BLOOD  (FLUID_RED)
    4, // FLUID_SLIME  (FLUID_GREEN)
    0, // FLUID_EMPTY
    5, // FLUID_LEMONADE (FLUID_YELLOW)
    6, // FLUID_MILK   (FLUID_WHITE)
    8, // FLUID_INK    (FLUID_BLACK)
];

/// clientToServerFluidMap from const.h
pub const CLIENT_TO_SERVER_FLUID_MAP: [u8; 19] = [
    0,  // FLUID_EMPTY
    1,  // FLUID_WATER       (FLUID_BLUE)
    7,  // FLUID_MANA        (FLUID_PURPLE)
    3,  // FLUID_BEER        (FLUID_BROWN)
    19, // FLUID_MUD         (FLUID_BROWN + 16 = 3 + 16)
    2,  // FLUID_BLOOD       (FLUID_RED)
    4,  // FLUID_SLIME       (FLUID_GREEN)
    27, // FLUID_RUM         (FLUID_BROWN + 24 = 3 + 24)
    5,  // FLUID_LEMONADE    (FLUID_YELLOW)
    6,  // FLUID_MILK        (FLUID_WHITE)
    15, // FLUID_WINE        (FLUID_PURPLE + 8 = 7 + 8)
    10, // FLUID_LIFE        (FLUID_RED + 8 = 2 + 8)
    13, // FLUID_URINE       (FLUID_YELLOW + 8 = 5 + 8)
    11, // FLUID_OIL         (FLUID_BROWN + 8 = 3 + 8)
    21, // FLUID_FRUITJUICE  (FLUID_YELLOW + 16 = 5 + 16)
    14, // FLUID_COCONUTMILK (FLUID_WHITE + 8 = 6 + 8)
    35, // FLUID_TEA         (FLUID_BROWN + 32 = 3 + 32)
    43, // FLUID_MEAD        (FLUID_BROWN + 40 = 3 + 40)
    8,  // FLUID_INK         (FLUID_BLACK)
];

/// fluidMap from const.h – maps FluidColors_t index → ClientFluidTypes_t value
pub const FLUID_MAP: [u8; 9] = [
    0,  // CLIENTFLUID_EMPTY
    1,  // CLIENTFLUID_BLUE
    5,  // CLIENTFLUID_RED
    3,  // CLIENTFLUID_BROWN_1
    6,  // CLIENTFLUID_GREEN
    8,  // CLIENTFLUID_YELLOW
    9,  // CLIENTFLUID_WHITE
    2,  // CLIENTFLUID_PURPLE
    18, // CLIENTFLUID_BLACK
];

// ---------------------------------------------------------------------------
// Enums from const.h
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MagicEffectsType {
    EndLoop = 0,
    Delta = 1,
    Delay = 2,
    CreateEffect = 3,
    CreateDistanceEffect = 4,
    CreateDistanceEffectReversed = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MagicEffectClass {
    None = 0,
    DrawBlood = 1,
    LoseEnergy = 2,
    Poff = 3,
    BlockHit = 4,
    ExplosionArea = 5,
    ExplosionHit = 6,
    FireArea = 7,
    YellowRings = 8,
    GreenRings = 9,
    HitArea = 10,
    Teleport = 11,
    EnergyHit = 12,
    MagicBlue = 13,
    MagicRed = 14,
    MagicGreen = 15,
    HitByFire = 16,
    HitByPoison = 17,
    MortArea = 18,
    SoundGreen = 19,
    SoundRed = 20,
    PoisonArea = 21,
    SoundYellow = 22,
    SoundPurple = 23,
    SoundBlue = 24,
    SoundWhite = 25,
    Bubbles = 26,
    Craps = 27,
    GiftWraps = 28,
    FireworkYellow = 29,
    FireworkRed = 30,
    FireworkBlue = 31,
    Stun = 32,
    Sleep = 33,
    WaterCreature = 34,
    GroundShaker = 35,
    Hearts = 36,
    FireAttack = 37,
    EnergyArea = 38,
    SmallClouds = 39,
    HolyDamage = 40,
    BigClouds = 41,
    IceArea = 42,
    IceTornado = 43,
    IceAttack = 44,
    Stones = 45,
    SmallPlants = 46,
    Carniphila = 47,
    PurpleEnergy = 48,
    YellowEnergy = 49,
    HolyArea = 50,
    BigPlants = 51,
    Cake = 52,
    GiantIce = 53,
    WaterSplash = 54,
    PlantAttack = 55,
    TutorialArrow = 56,
    TutorialSquare = 57,
    MirrorHorizontal = 58,
    MirrorVertical = 59,
    SkullHorizontal = 60,
    SkullVertical = 61,
    Assassin = 62,
    StepsHorizontal = 63,
    BloodySteps = 64,
    StepsVertical = 65,
    YalaharIGhost = 66,
    Bats = 67,
    Smoke = 68,
    Insects = 69,
    DragonHead = 70,
    OrcShaman = 71,
    OrcShamanFire = 72,
    Thunder = 73,
    Ferumbras = 74,
    ConfettiHorizontal = 75,
    ConfettiVertical = 76,
    // 77-157 empty
    BlackSmoke = 158,
    // 159-166 empty
    RedSmoke = 167,
    YellowSmoke = 168,
    GreenSmoke = 169,
    PurpleSmoke = 170,
    EarlyThunder = 171,
    RagiazBoneCapsule = 172,
    CriticalDamage = 173,
    // 174 empty
    PlungingFish = 175,
    BlueChain = 176,
    OrangeChain = 177,
    GreenChain = 178,
    PurpleChain = 179,
    GreyChain = 180,
    YellowChain = 181,
    YellowSparkles = 182,
    // 183 empty
    FaeExplosion = 184,
    FaeComing = 185,
    FaeGoing = 186,
    // 187 empty
    BigCloudsSingleSpace = 188,
    StonesSingleSpace = 189,
    // 190 empty
    BlueGhost = 191,
    // 192 empty
    PointOfInterest = 193,
    MapEffect = 194,
    PinkSpark = 195,
    FireworkGreen = 196,
    FireworkOrange = 197,
    FireworkPurple = 198,
    FireworkTurquoise = 199,
    // 200 empty
    TheCube = 201,
    DrawInk = 202,
    PrismaticSparkles = 203,
    Thaian = 204,
    ThaianGhost = 205,
    GhostSmoke = 206,
    // 207 empty
    FloatingBlock = 208,
    Block = 209,
    Rooting = 210,
    // 211-212 removed from client
    GhostlyScratch = 213,
    GhostlyBite = 214,
    BigScratching = 215,
    Slash = 216,
    Bite = 217,
    // 218 empty
    ChivalrousChallenge = 219,
    DivineDazzle = 220,
    ElectricalSpark = 221,
    PurpleTeleport = 222,
    RedTeleport = 223,
    OrangeTeleport = 224,
    GreyTeleport = 225,
    LightBlueTeleport = 226,
    // 227-229 empty
    Fatal = 230,
    Dodge = 231,
    Hourglass = 232,
    FireworksStar = 233,
    FireworksCircle = 234,
    Ferumbras1 = 235,
    Gazharagoth = 236,
    MadMage = 237,
    Horestis = 238,
    Devovorga = 239,
    Ferumbras2 = 240,
    Foam = 241,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ShootType {
    None = 0,
    Spear = 1,
    Bolt = 2,
    Arrow = 3,
    Fire = 4,
    Energy = 5,
    PoisonArrow = 6,
    BurstArrow = 7,
    ThrowingStar = 8,
    ThrowingKnife = 9,
    SmallStone = 10,
    Death = 11,
    LargeRock = 12,
    Snowball = 13,
    PowerBolt = 14,
    Poison = 15,
    InfernalBolt = 16,
    HuntingSpear = 17,
    EnchantedSpear = 18,
    RedStar = 19,
    GreenStar = 20,
    RoyalSpear = 21,
    SniperArrow = 22,
    OnyxArrow = 23,
    PiercingBolt = 24,
    WhirlwindSword = 25,
    WhirlwindAxe = 26,
    WhirlwindClub = 27,
    EtherealSpear = 28,
    Ice = 29,
    Earth = 30,
    Holy = 31,
    SuddenDeath = 32,
    FlashArrow = 33,
    FlammingArrow = 34,
    ShiverArrow = 35,
    EnergyBall = 36,
    SmallIce = 37,
    SmallHoly = 38,
    SmallEarth = 39,
    EarthArrow = 40,
    Explosion = 41,
    Cake = 42,
    TarsalArrow = 44,
    VortexBolt = 45,
    PrismaticBolt = 48,
    CrystallineArrow = 49,
    DrillBolt = 50,
    EnvenomedArrow = 51,
    GloothSpear = 53,
    SimpleArrow = 54,
    LeafStar = 56,
    DiamondArrow = 57,
    SpectralBolt = 58,
    RoyalStar = 59,
    /// Internal use only — not sent to client
    WeaponType = 0xFE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpeakClass {
    Say = 1,
    Whisper = 2,
    Yell = 3,
    PrivateFrom = 4,
    PrivateTo = 5,
    ChannelY = 7,
    ChannelO = 8,
    Spell = 9,
    PrivateNp = 10,
    PrivateNpConsole = 11,
    PrivatePn = 12,
    Broadcast = 13,
    ChannelR1 = 14,
    PrivateRedFrom = 15,
    PrivateRedTo = 16,
    MonsterSay = 36,
    MonsterYell = 37,
    Potion = 52,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageClass {
    StatusDefault = 17,
    StatusWarning = 18,
    EventAdvance = 19,
    StatusWarning2 = 20,
    StatusSmall = 21,
    InfoDescr = 22,
    DamageDealt = 23,
    DamageReceived = 24,
    Healed = 25,
    Experience = 26,
    DamageOthers = 27,
    HealedOthers = 28,
    ExperienceOthers = 29,
    EventDefault = 30,
    Loot = 31,
    Trade = 32,
    Guild = 33,
    PartyManagement = 34,
    Party = 35,
    Report = 38,
    HotkeyPressed = 39,
    Market = 42,
    BeyondLast = 44,
    TournamentInfo = 45,
    Attention = 48,
    BoostedCreature = 49,
    OfflineTraining = 50,
    Transaction = 51,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FluidColor {
    Empty = 0,
    Blue = 1,
    Red = 2,
    Brown = 3,
    Green = 4,
    Yellow = 5,
    White = 6,
    Purple = 7,
    Black = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FluidType {
    None = 0,         // FLUID_EMPTY
    Water = 1,        // FLUID_BLUE
    Blood = 2,        // FLUID_RED
    Beer = 3,         // FLUID_BROWN
    Slime = 4,        // FLUID_GREEN
    Lemonade = 5,     // FLUID_YELLOW
    Milk = 6,         // FLUID_WHITE
    Mana = 7,         // FLUID_PURPLE
    Ink = 8,          // FLUID_BLACK
    Life = 10,        // FLUID_RED + 8
    Oil = 11,         // FLUID_BROWN + 8
    Urine = 13,       // FLUID_YELLOW + 8
    CoconutMilk = 14, // FLUID_WHITE + 8
    Wine = 15,        // FLUID_PURPLE + 8
    Mud = 19,         // FLUID_BROWN + 16
    FruitJuice = 21,  // FLUID_YELLOW + 16
    Lava = 26,        // FLUID_RED + 24
    Rum = 27,         // FLUID_BROWN + 24
    Swamp = 28,       // FLUID_GREEN + 24
    Tea = 35,         // FLUID_BROWN + 32
    Mead = 43,        // FLUID_BROWN + 40
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClientFluidType {
    Empty = 0,
    Blue = 1,
    Purple = 2,
    Brown1 = 3,
    Brown2 = 4,
    Red = 5,
    Green = 6,
    Brown = 7,
    Yellow = 8,
    White = 9,
    Black = 18,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SquareColor {
    Black = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TextColor {
    Blue = 5,
    LightGreen = 30,
    LightBlue = 35,
    DarkGrey = 86,
    MayaBlue = 95,
    DarkRed = 108,
    LightGrey = 129,
    SkyBlue = 143,
    Purple = 154,
    ElectricPurple = 155,
    Red = 180,
    PastelRed = 194,
    Orange = 198,
    Yellow = 210,
    WhiteExp = 215,
    None = 255,
}

/// Icon flags — bit flags, not repr(u8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconFlags(pub u32);

impl IconFlags {
    pub const POISON: u32 = 1 << 0;
    pub const BURN: u32 = 1 << 1;
    pub const ENERGY: u32 = 1 << 2;
    pub const DRUNK: u32 = 1 << 3;
    pub const MANASHIELD: u32 = 1 << 4;
    pub const PARALYZE: u32 = 1 << 5;
    pub const HASTE: u32 = 1 << 6;
    pub const SWORDS: u32 = 1 << 7;
    pub const DROWNING: u32 = 1 << 8;
    pub const FREEZING: u32 = 1 << 9;
    pub const DAZZLED: u32 = 1 << 10;
    pub const CURSED: u32 = 1 << 11;
    pub const PARTY_BUFF: u32 = 1 << 12;
    pub const REDSWORDS: u32 = 1 << 13;
    pub const PIGEON: u32 = 1 << 14;
    pub const BLEEDING: u32 = 1 << 15;
    pub const LESSERHEX: u32 = 1 << 16;
    pub const INTENSEHEX: u32 = 1 << 17;
    pub const GREATERHEX: u32 = 1 << 18;
    pub const ROOT: u32 = 1 << 19;
    pub const FEAR: u32 = 1 << 20;
    pub const GOSHNAR1: u32 = 1 << 21;
    pub const GOSHNAR2: u32 = 1 << 22;
    pub const GOSHNAR3: u32 = 1 << 23;
    pub const GOSHNAR4: u32 = 1 << 24;
    pub const GOSHNAR5: u32 = 1 << 25;
    pub const MANASHIELD_BREAKABLE: u32 = 1 << 26;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum WeaponType {
    #[default]
    None = 0,
    Sword = 1,
    Club = 2,
    Axe = 3,
    Shield = 4,
    Distance = 5,
    Wand = 6,
    Ammo = 7,
    Quiver = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Ammo {
    None = 0,
    Bolt = 1,
    Arrow = 2,
    Spear = 3,
    ThrowingStar = 4,
    ThrowingKnife = 5,
    Stone = 6,
    Snowball = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WeaponAction {
    None = 0,
    RemoveCount = 1,
    RemoveCharge = 2,
    Move = 3,
}

/// WieldInfo_t — bit flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WieldInfo(pub u32);

impl WieldInfo {
    pub const NONE: u32 = 0;
    pub const LEVEL: u32 = 1 << 0;
    pub const MAGLV: u32 = 1 << 1;
    pub const VOCREQ: u32 = 1 << 2;
    pub const PREMIUM: u32 = 1 << 3;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Skull {
    None = 0,
    Yellow = 1,
    Green = 2,
    White = 3,
    Red = 4,
    Black = 5,
    Orange = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PartyShield {
    None = 0,
    WhiteYellow = 1,
    WhiteBlue = 2,
    Blue = 3,
    Yellow = 4,
    BlueSharedExp = 5,
    YellowSharedExp = 6,
    BlueNoSharedExpBlink = 7,
    YellowNoSharedExpBlink = 8,
    BlueNoSharedExp = 9,
    YellowNoSharedExp = 10,
    Gray = 11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GuildEmblem {
    None = 0,
    Ally = 1,
    Enemy = 2,
    Neutral = 3,
    Member = 4,
    Other = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ItemId {
    BrowseField = 460,
    DecorationKit = 26054,
    FirefieldPvpFull = 1487,
    FirefieldPvpMedium = 1488,
    FirefieldPvpSmall = 1489,
    FirefieldPersistentFull = 1492,
    FirefieldPersistentMedium = 1493,
    FirefieldPersistentSmall = 1494,
    FirefieldNopvp = 1500,
    FirefieldNopvpMedium = 1501,
    PoisonfieldPvp = 1490,
    PoisonfieldPersistent = 1496,
    PoisonfieldNopvp = 1503,
    EnergyfieldPvp = 1491,
    EnergyfieldPersistent = 1495,
    EnergyfieldNopvp = 1504,
    Magicwall = 1497,
    MagicwallPersistent = 1498,
    MagicwallSafe = 11098,
    MagicwallNopvp = 20669,
    Wildgrowth = 1499,
    WildgrowthPersistent = 2721,
    WildgrowthSafe = 11099,
    WildgrowthNopvp = 20670,
    Bag = 1987,
    ShoppingBag = 23782,
    GoldCoin = 2148,
    PlatinumCoin = 2152,
    CrystalCoin = 2160,
    StoreCoin = 24774,
    Depot = 2594,
    Locker = 2589,
    Inbox = 14404,
    Market = 14405,
    StoreInbox = 26052,
    DepotBoxI = 25453,
    DepotBoxII = 25454,
    DepotBoxIII = 25455,
    DepotBoxIV = 25456,
    DepotBoxV = 25457,
    DepotBoxVI = 25458,
    DepotBoxVII = 25459,
    DepotBoxVIII = 25460,
    DepotBoxIX = 25461,
    DepotBoxX = 25462,
    DepotBoxXI = 25463,
    DepotBoxXII = 25464,
    DepotBoxXIII = 25465,
    DepotBoxXIV = 25466,
    DepotBoxXV = 25467,
    DepotBoxXVI = 25468,
    DepotBoxXVII = 25469,
    DepotBoxXVIII = 34571,
    DepotBoxXIX = 44714,
    DepotBoxXX = 44715,
    MaleCorpse = 3058,
    FemaleCorpse = 3065,
    FullSplash = 2016,
    SmallSplash = 2019,
    Parcel = 2595,
    Letter = 2597,
    LetterStamped = 2598,
    Label = 2599,
    AmuletOfLoss = 2173,
    DocumentRo = 1968,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceType {
    BankBalance = 0x00,
    GoldEquipped = 0x01,
    PreyWildcards = 0x0A,
    DailyRewardStreak = 0x14,
    DailyRewardJokers = 0x15,
}

/// PlayerFlags — u64 bit flags
pub struct PlayerFlags;

impl PlayerFlags {
    pub const CANNOT_USE_COMBAT: u64 = 1 << 0;
    pub const CANNOT_ATTACK_PLAYER: u64 = 1 << 1;
    pub const CANNOT_ATTACK_MONSTER: u64 = 1 << 2;
    pub const CANNOT_BE_ATTACKED: u64 = 1 << 3;
    pub const CAN_CONVINCE_ALL: u64 = 1 << 4;
    pub const CAN_SUMMON_ALL: u64 = 1 << 5;
    pub const CAN_ILLUSION_ALL: u64 = 1 << 6;
    pub const CAN_SENSE_INVISIBILITY: u64 = 1 << 7;
    pub const IGNORED_BY_MONSTERS: u64 = 1 << 8;
    pub const NOT_GAIN_IN_FIGHT: u64 = 1 << 9;
    pub const HAS_INFINITE_MANA: u64 = 1 << 10;
    pub const HAS_INFINITE_SOUL: u64 = 1 << 11;
    pub const HAS_NO_EXHAUSTION: u64 = 1 << 12;
    pub const CANNOT_USE_SPELLS: u64 = 1 << 13;
    pub const CANNOT_PICKUP_ITEM: u64 = 1 << 14;
    pub const CAN_ALWAYS_LOGIN: u64 = 1 << 15;
    pub const CAN_BROADCAST: u64 = 1 << 16;
    pub const CAN_EDIT_HOUSES: u64 = 1 << 17;
    pub const CANNOT_BE_BANNED: u64 = 1 << 18;
    pub const CANNOT_BE_PUSHED: u64 = 1 << 19;
    pub const HAS_INFINITE_CAPACITY: u64 = 1 << 20;
    pub const CAN_PUSH_ALL_CREATURES: u64 = 1 << 21;
    pub const CAN_TALK_RED_PRIVATE: u64 = 1 << 22;
    pub const CAN_TALK_RED_CHANNEL: u64 = 1 << 23;
    pub const TALK_ORANGE_HELP_CHANNEL: u64 = 1 << 24;
    pub const NOT_GAIN_EXPERIENCE: u64 = 1 << 25;
    pub const NOT_GAIN_MANA: u64 = 1 << 26;
    pub const NOT_GAIN_HEALTH: u64 = 1 << 27;
    pub const NOT_GAIN_SKILL: u64 = 1 << 28;
    pub const SET_MAX_SPEED: u64 = 1 << 29;
    pub const SPECIAL_VIP: u64 = 1 << 30;
    pub const NOT_GENERATE_LOOT: u64 = 1u64 << 31;
    // exponent 32 was deprecated
    pub const IGNORE_PROTECTION_ZONE: u64 = 1u64 << 33;
    pub const IGNORE_SPELL_CHECK: u64 = 1u64 << 34;
    pub const IGNORE_WEAPON_CHECK: u64 = 1u64 << 35;
    pub const CANNOT_BE_MUTED: u64 = 1u64 << 36;
    pub const IS_ALWAYS_PREMIUM: u64 = 1u64 << 37;
    pub const IGNORE_YELL_CHECK: u64 = 1u64 << 38;
    pub const IGNORE_SEND_PRIVATE_CHECK: u64 = 1u64 << 39;
}

/// PodiumFlags — values 0/1/2, not bit flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PodiumFlag {
    ShowPlatform = 0,
    ShowOutfit = 1,
    ShowMount = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ReloadType {
    All = 0,
    Actions = 1,
    Chat = 2,
    Config = 3,
    CreatureScripts = 4,
    Events = 5,
    Global = 6,
    GlobalEvents = 7,
    Items = 8,
    Monsters = 9,
    Mounts = 10,
    Movements = 11,
    Npcs = 12,
    Quests = 13,
    Scripts = 14,
    Spells = 15,
    TalkActions = 16,
    Weapons = 17,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MonsterIcon {
    Vulnerable = 1,
    Weakened = 2,
    Melee = 3,
    Influenced = 4,
    Fiendish = 5,
}

impl MonsterIcon {
    pub const FIRST: MonsterIcon = MonsterIcon::Vulnerable;
    pub const LAST: MonsterIcon = MonsterIcon::Fiendish;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CreatureIcon {
    CrossWhite = 1,
    CrossWhiteRed = 2,
    OrbRed = 3,
    OrbGreen = 4,
    OrbRedGreen = 5,
    GemGreen = 6,
    GemYellow = 7,
    GemBlue = 8,
    GemPurple = 9,
    GemRed = 10,
    Pigeon = 11,
    Energy = 12,
    Poison = 13,
    Water = 14,
    Fire = 15,
    Ice = 16,
    ArrowUp = 17,
    ArrowDown = 18,
    Warning = 19,
    Question = 20,
    CrossRed = 21,
}

impl CreatureIcon {
    pub const FIRST: CreatureIcon = CreatureIcon::CrossWhite;
    pub const LAST: CreatureIcon = CreatureIcon::CrossRed;
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Integer constants ---
    #[test]
    fn test_networkmessage_maxsize() {
        assert_eq!(NETWORKMESSAGE_MAXSIZE, 24590);
    }

    #[test]
    fn test_min_market_fee() {
        assert_eq!(MIN_MARKET_FEE, 20);
    }

    #[test]
    fn test_max_market_fee() {
        assert_eq!(MAX_MARKET_FEE, 100_000);
    }

    #[test]
    fn test_channel_guild() {
        assert_eq!(CHANNEL_GUILD, 0x00);
    }

    #[test]
    fn test_channel_party() {
        assert_eq!(CHANNEL_PARTY, 0x01);
    }

    #[test]
    fn test_channel_private() {
        assert_eq!(CHANNEL_PRIVATE, 0xFFFF);
    }

    #[test]
    fn test_pstrg_reserved_range_start() {
        assert_eq!(PSTRG_RESERVED_RANGE_START, 10_000_000);
    }

    #[test]
    fn test_pstrg_reserved_range_size() {
        assert_eq!(PSTRG_RESERVED_RANGE_SIZE, 10_000_000);
    }

    // --- MagicEffectsType discriminants ---
    #[test]
    fn test_magic_effects_type_discriminants() {
        assert_eq!(MagicEffectsType::EndLoop as u8, 0);
        assert_eq!(MagicEffectsType::Delta as u8, 1);
        assert_eq!(MagicEffectsType::Delay as u8, 2);
        assert_eq!(MagicEffectsType::CreateEffect as u8, 3);
        assert_eq!(MagicEffectsType::CreateDistanceEffect as u8, 4);
        assert_eq!(MagicEffectsType::CreateDistanceEffectReversed as u8, 5);
    }

    // --- MagicEffectClass key discriminants ---
    #[test]
    fn test_magic_effect_class_none() {
        assert_eq!(MagicEffectClass::None as u8, 0);
    }

    #[test]
    fn test_magic_effect_class_selected_values() {
        assert_eq!(MagicEffectClass::DrawBlood as u8, 1);
        assert_eq!(MagicEffectClass::ConfettiVertical as u8, 76);
        assert_eq!(MagicEffectClass::BlackSmoke as u8, 158);
        assert_eq!(MagicEffectClass::RedSmoke as u8, 167);
        assert_eq!(MagicEffectClass::CriticalDamage as u8, 173);
        assert_eq!(MagicEffectClass::PlungingFish as u8, 175);
        assert_eq!(MagicEffectClass::Foam as u8, 241);
    }

    // --- ShootType discriminants ---
    #[test]
    fn test_shoot_type_discriminants() {
        assert_eq!(ShootType::None as u8, 0);
        assert_eq!(ShootType::Spear as u8, 1);
        assert_eq!(ShootType::Arrow as u8, 3);
        assert_eq!(ShootType::TarsalArrow as u8, 44);
        assert_eq!(ShootType::VortexBolt as u8, 45);
        assert_eq!(ShootType::PrismaticBolt as u8, 48);
        assert_eq!(ShootType::RoyalStar as u8, 59);
        assert_eq!(ShootType::WeaponType as u8, 0xFE);
    }

    // --- SpeakClass discriminants ---
    #[test]
    fn test_speak_class_discriminants() {
        assert_eq!(SpeakClass::Say as u8, 1);
        assert_eq!(SpeakClass::Whisper as u8, 2);
        assert_eq!(SpeakClass::Yell as u8, 3);
        assert_eq!(SpeakClass::MonsterSay as u8, 36);
        assert_eq!(SpeakClass::MonsterYell as u8, 37);
        assert_eq!(SpeakClass::Potion as u8, 52);
    }

    // --- MessageClass discriminants ---
    #[test]
    fn test_message_class_discriminants() {
        assert_eq!(MessageClass::StatusDefault as u8, 17);
        assert_eq!(MessageClass::StatusWarning as u8, 18);
        assert_eq!(MessageClass::BeyondLast as u8, 44);
        assert_eq!(MessageClass::Transaction as u8, 51);
    }

    // --- FluidColor discriminants ---
    #[test]
    fn test_fluid_color_discriminants() {
        assert_eq!(FluidColor::Empty as u8, 0);
        assert_eq!(FluidColor::Blue as u8, 1);
        assert_eq!(FluidColor::Black as u8, 8);
    }

    // --- FluidType discriminants (incl. derived values) ---
    #[test]
    fn test_fluid_type_discriminants() {
        assert_eq!(FluidType::None as u8, 0); // FLUID_EMPTY
        assert_eq!(FluidType::Water as u8, 1); // FLUID_BLUE
        assert_eq!(FluidType::Blood as u8, 2); // FLUID_RED
        assert_eq!(FluidType::Beer as u8, 3); // FLUID_BROWN
        assert_eq!(FluidType::Ink as u8, 8); // FLUID_BLACK
        assert_eq!(FluidType::Life as u8, 10); // FLUID_RED + 8
        assert_eq!(FluidType::Oil as u8, 11); // FLUID_BROWN + 8
        assert_eq!(FluidType::Urine as u8, 13); // FLUID_YELLOW + 8
        assert_eq!(FluidType::CoconutMilk as u8, 14); // FLUID_WHITE + 8
        assert_eq!(FluidType::Wine as u8, 15); // FLUID_PURPLE + 8
        assert_eq!(FluidType::Mud as u8, 19); // FLUID_BROWN + 16
        assert_eq!(FluidType::FruitJuice as u8, 21); // FLUID_YELLOW + 16
        assert_eq!(FluidType::Lava as u8, 26); // FLUID_RED + 24
        assert_eq!(FluidType::Rum as u8, 27); // FLUID_BROWN + 24
        assert_eq!(FluidType::Swamp as u8, 28); // FLUID_GREEN + 24
        assert_eq!(FluidType::Tea as u8, 35); // FLUID_BROWN + 32
        assert_eq!(FluidType::Mead as u8, 43); // FLUID_BROWN + 40
    }

    // --- ClientFluidType discriminants ---
    #[test]
    fn test_client_fluid_type_discriminants() {
        assert_eq!(ClientFluidType::Empty as u8, 0);
        assert_eq!(ClientFluidType::Blue as u8, 1);
        assert_eq!(ClientFluidType::Black as u8, 18);
    }

    // --- TextColor discriminants ---
    #[test]
    fn test_text_color_discriminants() {
        assert_eq!(TextColor::Blue as u8, 5);
        assert_eq!(TextColor::None as u8, 255);
        assert_eq!(TextColor::Orange as u8, 198);
    }

    // --- IconFlags ---
    #[test]
    fn test_icon_flags_values() {
        assert_eq!(IconFlags::POISON, 1 << 0);
        assert_eq!(IconFlags::BURN, 1 << 1);
        assert_eq!(IconFlags::MANASHIELD_BREAKABLE, 1 << 26);
        assert_eq!(IconFlags::GOSHNAR5, 1 << 25);
    }

    // --- WeaponType ---
    #[test]
    fn test_weapon_type_discriminants() {
        assert_eq!(WeaponType::None as u8, 0);
        assert_eq!(WeaponType::Sword as u8, 1);
        assert_eq!(WeaponType::Club as u8, 2);
        assert_eq!(WeaponType::Axe as u8, 3);
        assert_eq!(WeaponType::Shield as u8, 4);
        assert_eq!(WeaponType::Distance as u8, 5);
        assert_eq!(WeaponType::Wand as u8, 6);
        assert_eq!(WeaponType::Ammo as u8, 7);
        assert_eq!(WeaponType::Quiver as u8, 8);
    }

    // --- Ammo ---
    #[test]
    fn test_ammo_variants() {
        assert_eq!(Ammo::None as u8, 0);
        assert_eq!(Ammo::Snowball as u8, 7);
    }

    // --- WeaponAction ---
    #[test]
    fn test_weapon_action_variants() {
        assert_eq!(WeaponAction::None as u8, 0);
        assert_eq!(WeaponAction::Move as u8, 3);
    }

    // --- WieldInfo flags ---
    #[test]
    fn test_wield_info_flags() {
        assert_eq!(WieldInfo::NONE, 0);
        assert_eq!(WieldInfo::LEVEL, 1 << 0);
        assert_eq!(WieldInfo::PREMIUM, 1 << 3);
    }

    // --- Skull ---
    #[test]
    fn test_skull_discriminants() {
        assert_eq!(Skull::None as u8, 0);
        assert_eq!(Skull::Orange as u8, 6);
    }

    // --- PartyShield ---
    #[test]
    fn test_party_shield_discriminants() {
        assert_eq!(PartyShield::None as u8, 0);
        assert_eq!(PartyShield::Gray as u8, 11);
    }

    // --- GuildEmblem ---
    #[test]
    fn test_guild_emblem_discriminants() {
        assert_eq!(GuildEmblem::None as u8, 0);
        assert_eq!(GuildEmblem::Other as u8, 5);
    }

    // --- ItemId ---
    #[test]
    fn test_item_id_key_values() {
        assert_eq!(ItemId::BrowseField as u16, 460);
        assert_eq!(ItemId::GoldCoin as u16, 2148);
        assert_eq!(ItemId::StoreCoin as u16, 24774);
        assert_eq!(ItemId::DepotBoxXX as u16, 44715);
        assert_eq!(ItemId::DocumentRo as u16, 1968);
    }

    // --- ResourceType ---
    #[test]
    fn test_resource_type_discriminants() {
        assert_eq!(ResourceType::BankBalance as u8, 0x00);
        assert_eq!(ResourceType::GoldEquipped as u8, 0x01);
        assert_eq!(ResourceType::PreyWildcards as u8, 0x0A);
        assert_eq!(ResourceType::DailyRewardStreak as u8, 0x14);
        assert_eq!(ResourceType::DailyRewardJokers as u8, 0x15);
    }

    // --- PlayerFlags ---
    #[test]
    fn test_player_flags_values() {
        assert_eq!(PlayerFlags::CANNOT_USE_COMBAT, 1 << 0);
        assert_eq!(PlayerFlags::NOT_GENERATE_LOOT, 1u64 << 31);
        assert_eq!(PlayerFlags::IGNORE_PROTECTION_ZONE, 1u64 << 33);
        assert_eq!(PlayerFlags::IGNORE_SEND_PRIVATE_CHECK, 1u64 << 39);
    }

    // --- PodiumFlag ---
    #[test]
    fn test_podium_flag_discriminants() {
        assert_eq!(PodiumFlag::ShowPlatform as u8, 0);
        assert_eq!(PodiumFlag::ShowOutfit as u8, 1);
        assert_eq!(PodiumFlag::ShowMount as u8, 2);
    }

    // --- ReloadType ---
    #[test]
    fn test_reload_type_discriminants() {
        assert_eq!(ReloadType::All as u8, 0);
        assert_eq!(ReloadType::Actions as u8, 1);
        assert_eq!(ReloadType::Chat as u8, 2);
        assert_eq!(ReloadType::Config as u8, 3);
        assert_eq!(ReloadType::CreatureScripts as u8, 4);
        assert_eq!(ReloadType::Events as u8, 5);
        assert_eq!(ReloadType::Global as u8, 6);
        assert_eq!(ReloadType::GlobalEvents as u8, 7);
        assert_eq!(ReloadType::Items as u8, 8);
        assert_eq!(ReloadType::Monsters as u8, 9);
        assert_eq!(ReloadType::Mounts as u8, 10);
        assert_eq!(ReloadType::Movements as u8, 11);
        assert_eq!(ReloadType::Npcs as u8, 12);
        assert_eq!(ReloadType::Quests as u8, 13);
        assert_eq!(ReloadType::Scripts as u8, 14);
        assert_eq!(ReloadType::Spells as u8, 15);
        assert_eq!(ReloadType::TalkActions as u8, 16);
        assert_eq!(ReloadType::Weapons as u8, 17);
    }

    // --- MonsterIcon ---
    #[test]
    fn test_monster_icon_discriminants() {
        assert_eq!(MonsterIcon::Vulnerable as u8, 1);
        assert_eq!(MonsterIcon::Fiendish as u8, 5);
        assert_eq!(MonsterIcon::FIRST as u8, 1);
        assert_eq!(MonsterIcon::LAST as u8, 5);
    }

    // --- CreatureIcon ---
    #[test]
    fn test_creature_icon_discriminants() {
        assert_eq!(CreatureIcon::CrossWhite as u8, 1);
        assert_eq!(CreatureIcon::CrossRed as u8, 21);
        assert_eq!(CreatureIcon::FIRST as u8, 1);
        assert_eq!(CreatureIcon::LAST as u8, 21);
    }

    // --- Fluid map arrays ---
    #[test]
    fn test_reverse_fluid_map_length() {
        assert_eq!(REVERSE_FLUID_MAP.len(), 11);
        assert_eq!(REVERSE_FLUID_MAP[0], 0); // FLUID_EMPTY
        assert_eq!(REVERSE_FLUID_MAP[1], 1); // FLUID_WATER
    }

    #[test]
    fn test_client_to_server_fluid_map_length() {
        assert_eq!(CLIENT_TO_SERVER_FLUID_MAP.len(), 19);
    }

    /// Verifies every entry in clientToServerFluidMap[] from const.h using the
    /// resolved FluidTypes_t numeric values (FluidColors_t aliases + offsets).
    #[test]
    fn test_client_to_server_fluid_map_exact_values() {
        // Order matches const.h clientToServerFluidMap[] verbatim:
        // FLUID_EMPTY, FLUID_WATER, FLUID_MANA, FLUID_BEER, FLUID_MUD,
        // FLUID_BLOOD, FLUID_SLIME, FLUID_RUM, FLUID_LEMONADE, FLUID_MILK,
        // FLUID_WINE, FLUID_LIFE, FLUID_URINE, FLUID_OIL, FLUID_FRUITJUICE,
        // FLUID_COCONUTMILK, FLUID_TEA, FLUID_MEAD, FLUID_INK
        let expected: [u8; 19] = [
            0,  // FLUID_EMPTY
            1,  // FLUID_WATER  (FLUID_BLUE)
            7,  // FLUID_MANA   (FLUID_PURPLE)
            3,  // FLUID_BEER   (FLUID_BROWN)
            19, // FLUID_MUD    (FLUID_BROWN + 16 = 3 + 16)
            2,  // FLUID_BLOOD  (FLUID_RED)
            4,  // FLUID_SLIME  (FLUID_GREEN)
            27, // FLUID_RUM    (FLUID_BROWN + 24 = 3 + 24)
            5,  // FLUID_LEMONADE (FLUID_YELLOW)
            6,  // FLUID_MILK   (FLUID_WHITE)
            15, // FLUID_WINE   (FLUID_PURPLE + 8 = 7 + 8)
            10, // FLUID_LIFE   (FLUID_RED + 8 = 2 + 8)
            13, // FLUID_URINE  (FLUID_YELLOW + 8 = 5 + 8)
            11, // FLUID_OIL    (FLUID_BROWN + 8 = 3 + 8)
            21, // FLUID_FRUITJUICE (FLUID_YELLOW + 16 = 5 + 16)
            14, // FLUID_COCONUTMILK (FLUID_WHITE + 8 = 6 + 8)
            35, // FLUID_TEA    (FLUID_BROWN + 32 = 3 + 32)
            43, // FLUID_MEAD   (FLUID_BROWN + 40 = 3 + 40)
            8,  // FLUID_INK    (FLUID_BLACK)
        ];
        assert_eq!(CLIENT_TO_SERVER_FLUID_MAP, expected);
    }

    #[test]
    fn test_fluid_map_length() {
        assert_eq!(FLUID_MAP.len(), 9);
        assert_eq!(FLUID_MAP[0], 0); // CLIENTFLUID_EMPTY
    }

    /// Verify every entry in fluidMap[] from const.h.
    /// Maps FluidColors_t index → ClientFluidTypes_t value.
    #[test]
    fn test_fluid_map_exact_values() {
        // const.h fluidMap[]:
        // CLIENTFLUID_EMPTY=0, CLIENTFLUID_BLUE=1, CLIENTFLUID_RED=5,
        // CLIENTFLUID_BROWN_1=3, CLIENTFLUID_GREEN=6, CLIENTFLUID_YELLOW=8,
        // CLIENTFLUID_WHITE=9, CLIENTFLUID_PURPLE=2, CLIENTFLUID_BLACK=18
        let expected: [u8; 9] = [0, 1, 5, 3, 6, 8, 9, 2, 18];
        assert_eq!(FLUID_MAP, expected);
    }

    /// Verify every entry in reverseFluidMap[] from const.h.
    #[test]
    fn test_reverse_fluid_map_exact_values() {
        // const.h reverseFluidMap[]:
        // FLUID_EMPTY, FLUID_WATER, FLUID_MANA, FLUID_BEER, FLUID_EMPTY,
        // FLUID_BLOOD, FLUID_SLIME, FLUID_EMPTY, FLUID_LEMONADE, FLUID_MILK, FLUID_INK
        // = [0, 1, 7, 3, 0, 2, 4, 0, 5, 6, 8]
        let expected: [u8; 11] = [0, 1, 7, 3, 0, 2, 4, 0, 5, 6, 8];
        assert_eq!(REVERSE_FLUID_MAP, expected);
    }

    // ---------------------------------------------------------------------
    // Exhaustive per-variant value asserts (re-audit gap-fill)
    // Each individual C++ constant must have a value-assertion test.
    // ---------------------------------------------------------------------

    #[test]
    fn test_magic_effect_class_all_variants_exact_values() {
        // Every value listed in const.h MagicEffectClasses
        assert_eq!(MagicEffectClass::None as u8, 0);
        assert_eq!(MagicEffectClass::DrawBlood as u8, 1);
        assert_eq!(MagicEffectClass::LoseEnergy as u8, 2);
        assert_eq!(MagicEffectClass::Poff as u8, 3);
        assert_eq!(MagicEffectClass::BlockHit as u8, 4);
        assert_eq!(MagicEffectClass::ExplosionArea as u8, 5);
        assert_eq!(MagicEffectClass::ExplosionHit as u8, 6);
        assert_eq!(MagicEffectClass::FireArea as u8, 7);
        assert_eq!(MagicEffectClass::YellowRings as u8, 8);
        assert_eq!(MagicEffectClass::GreenRings as u8, 9);
        assert_eq!(MagicEffectClass::HitArea as u8, 10);
        assert_eq!(MagicEffectClass::Teleport as u8, 11);
        assert_eq!(MagicEffectClass::EnergyHit as u8, 12);
        assert_eq!(MagicEffectClass::MagicBlue as u8, 13);
        assert_eq!(MagicEffectClass::MagicRed as u8, 14);
        assert_eq!(MagicEffectClass::MagicGreen as u8, 15);
        assert_eq!(MagicEffectClass::HitByFire as u8, 16);
        assert_eq!(MagicEffectClass::HitByPoison as u8, 17);
        assert_eq!(MagicEffectClass::MortArea as u8, 18);
        assert_eq!(MagicEffectClass::SoundGreen as u8, 19);
        assert_eq!(MagicEffectClass::SoundRed as u8, 20);
        assert_eq!(MagicEffectClass::PoisonArea as u8, 21);
        assert_eq!(MagicEffectClass::SoundYellow as u8, 22);
        assert_eq!(MagicEffectClass::SoundPurple as u8, 23);
        assert_eq!(MagicEffectClass::SoundBlue as u8, 24);
        assert_eq!(MagicEffectClass::SoundWhite as u8, 25);
        assert_eq!(MagicEffectClass::Bubbles as u8, 26);
        assert_eq!(MagicEffectClass::Craps as u8, 27);
        assert_eq!(MagicEffectClass::GiftWraps as u8, 28);
        assert_eq!(MagicEffectClass::FireworkYellow as u8, 29);
        assert_eq!(MagicEffectClass::FireworkRed as u8, 30);
        assert_eq!(MagicEffectClass::FireworkBlue as u8, 31);
        assert_eq!(MagicEffectClass::Stun as u8, 32);
        assert_eq!(MagicEffectClass::Sleep as u8, 33);
        assert_eq!(MagicEffectClass::WaterCreature as u8, 34);
        assert_eq!(MagicEffectClass::GroundShaker as u8, 35);
        assert_eq!(MagicEffectClass::Hearts as u8, 36);
        assert_eq!(MagicEffectClass::FireAttack as u8, 37);
        assert_eq!(MagicEffectClass::EnergyArea as u8, 38);
        assert_eq!(MagicEffectClass::SmallClouds as u8, 39);
        assert_eq!(MagicEffectClass::HolyDamage as u8, 40);
        assert_eq!(MagicEffectClass::BigClouds as u8, 41);
        assert_eq!(MagicEffectClass::IceArea as u8, 42);
        assert_eq!(MagicEffectClass::IceTornado as u8, 43);
        assert_eq!(MagicEffectClass::IceAttack as u8, 44);
        assert_eq!(MagicEffectClass::Stones as u8, 45);
        assert_eq!(MagicEffectClass::SmallPlants as u8, 46);
        assert_eq!(MagicEffectClass::Carniphila as u8, 47);
        assert_eq!(MagicEffectClass::PurpleEnergy as u8, 48);
        assert_eq!(MagicEffectClass::YellowEnergy as u8, 49);
        assert_eq!(MagicEffectClass::HolyArea as u8, 50);
        assert_eq!(MagicEffectClass::BigPlants as u8, 51);
        assert_eq!(MagicEffectClass::Cake as u8, 52);
        assert_eq!(MagicEffectClass::GiantIce as u8, 53);
        assert_eq!(MagicEffectClass::WaterSplash as u8, 54);
        assert_eq!(MagicEffectClass::PlantAttack as u8, 55);
        assert_eq!(MagicEffectClass::TutorialArrow as u8, 56);
        assert_eq!(MagicEffectClass::TutorialSquare as u8, 57);
        assert_eq!(MagicEffectClass::MirrorHorizontal as u8, 58);
        assert_eq!(MagicEffectClass::MirrorVertical as u8, 59);
        assert_eq!(MagicEffectClass::SkullHorizontal as u8, 60);
        assert_eq!(MagicEffectClass::SkullVertical as u8, 61);
        assert_eq!(MagicEffectClass::Assassin as u8, 62);
        assert_eq!(MagicEffectClass::StepsHorizontal as u8, 63);
        assert_eq!(MagicEffectClass::BloodySteps as u8, 64);
        assert_eq!(MagicEffectClass::StepsVertical as u8, 65);
        assert_eq!(MagicEffectClass::YalaharIGhost as u8, 66);
        assert_eq!(MagicEffectClass::Bats as u8, 67);
        assert_eq!(MagicEffectClass::Smoke as u8, 68);
        assert_eq!(MagicEffectClass::Insects as u8, 69);
        assert_eq!(MagicEffectClass::DragonHead as u8, 70);
        assert_eq!(MagicEffectClass::OrcShaman as u8, 71);
        assert_eq!(MagicEffectClass::OrcShamanFire as u8, 72);
        assert_eq!(MagicEffectClass::Thunder as u8, 73);
        assert_eq!(MagicEffectClass::Ferumbras as u8, 74);
        assert_eq!(MagicEffectClass::ConfettiHorizontal as u8, 75);
        assert_eq!(MagicEffectClass::ConfettiVertical as u8, 76);
        assert_eq!(MagicEffectClass::BlackSmoke as u8, 158);
        assert_eq!(MagicEffectClass::RedSmoke as u8, 167);
        assert_eq!(MagicEffectClass::YellowSmoke as u8, 168);
        assert_eq!(MagicEffectClass::GreenSmoke as u8, 169);
        assert_eq!(MagicEffectClass::PurpleSmoke as u8, 170);
        assert_eq!(MagicEffectClass::EarlyThunder as u8, 171);
        assert_eq!(MagicEffectClass::RagiazBoneCapsule as u8, 172);
        assert_eq!(MagicEffectClass::CriticalDamage as u8, 173);
        assert_eq!(MagicEffectClass::PlungingFish as u8, 175);
        assert_eq!(MagicEffectClass::BlueChain as u8, 176);
        assert_eq!(MagicEffectClass::OrangeChain as u8, 177);
        assert_eq!(MagicEffectClass::GreenChain as u8, 178);
        assert_eq!(MagicEffectClass::PurpleChain as u8, 179);
        assert_eq!(MagicEffectClass::GreyChain as u8, 180);
        assert_eq!(MagicEffectClass::YellowChain as u8, 181);
        assert_eq!(MagicEffectClass::YellowSparkles as u8, 182);
        assert_eq!(MagicEffectClass::FaeExplosion as u8, 184);
        assert_eq!(MagicEffectClass::FaeComing as u8, 185);
        assert_eq!(MagicEffectClass::FaeGoing as u8, 186);
        assert_eq!(MagicEffectClass::BigCloudsSingleSpace as u8, 188);
        assert_eq!(MagicEffectClass::StonesSingleSpace as u8, 189);
        assert_eq!(MagicEffectClass::BlueGhost as u8, 191);
        assert_eq!(MagicEffectClass::PointOfInterest as u8, 193);
        assert_eq!(MagicEffectClass::MapEffect as u8, 194);
        assert_eq!(MagicEffectClass::PinkSpark as u8, 195);
        assert_eq!(MagicEffectClass::FireworkGreen as u8, 196);
        assert_eq!(MagicEffectClass::FireworkOrange as u8, 197);
        assert_eq!(MagicEffectClass::FireworkPurple as u8, 198);
        assert_eq!(MagicEffectClass::FireworkTurquoise as u8, 199);
        assert_eq!(MagicEffectClass::TheCube as u8, 201);
        assert_eq!(MagicEffectClass::DrawInk as u8, 202);
        assert_eq!(MagicEffectClass::PrismaticSparkles as u8, 203);
        assert_eq!(MagicEffectClass::Thaian as u8, 204);
        assert_eq!(MagicEffectClass::ThaianGhost as u8, 205);
        assert_eq!(MagicEffectClass::GhostSmoke as u8, 206);
        assert_eq!(MagicEffectClass::FloatingBlock as u8, 208);
        assert_eq!(MagicEffectClass::Block as u8, 209);
        assert_eq!(MagicEffectClass::Rooting as u8, 210);
        assert_eq!(MagicEffectClass::GhostlyScratch as u8, 213);
        assert_eq!(MagicEffectClass::GhostlyBite as u8, 214);
        assert_eq!(MagicEffectClass::BigScratching as u8, 215);
        assert_eq!(MagicEffectClass::Slash as u8, 216);
        assert_eq!(MagicEffectClass::Bite as u8, 217);
        assert_eq!(MagicEffectClass::ChivalrousChallenge as u8, 219);
        assert_eq!(MagicEffectClass::DivineDazzle as u8, 220);
        assert_eq!(MagicEffectClass::ElectricalSpark as u8, 221);
        assert_eq!(MagicEffectClass::PurpleTeleport as u8, 222);
        assert_eq!(MagicEffectClass::RedTeleport as u8, 223);
        assert_eq!(MagicEffectClass::OrangeTeleport as u8, 224);
        assert_eq!(MagicEffectClass::GreyTeleport as u8, 225);
        assert_eq!(MagicEffectClass::LightBlueTeleport as u8, 226);
        assert_eq!(MagicEffectClass::Fatal as u8, 230);
        assert_eq!(MagicEffectClass::Dodge as u8, 231);
        assert_eq!(MagicEffectClass::Hourglass as u8, 232);
        assert_eq!(MagicEffectClass::FireworksStar as u8, 233);
        assert_eq!(MagicEffectClass::FireworksCircle as u8, 234);
        assert_eq!(MagicEffectClass::Ferumbras1 as u8, 235);
        assert_eq!(MagicEffectClass::Gazharagoth as u8, 236);
        assert_eq!(MagicEffectClass::MadMage as u8, 237);
        assert_eq!(MagicEffectClass::Horestis as u8, 238);
        assert_eq!(MagicEffectClass::Devovorga as u8, 239);
        assert_eq!(MagicEffectClass::Ferumbras2 as u8, 240);
        assert_eq!(MagicEffectClass::Foam as u8, 241);
    }

    #[test]
    fn test_shoot_type_all_variants_exact_values() {
        assert_eq!(ShootType::None as u8, 0);
        assert_eq!(ShootType::Spear as u8, 1);
        assert_eq!(ShootType::Bolt as u8, 2);
        assert_eq!(ShootType::Arrow as u8, 3);
        assert_eq!(ShootType::Fire as u8, 4);
        assert_eq!(ShootType::Energy as u8, 5);
        assert_eq!(ShootType::PoisonArrow as u8, 6);
        assert_eq!(ShootType::BurstArrow as u8, 7);
        assert_eq!(ShootType::ThrowingStar as u8, 8);
        assert_eq!(ShootType::ThrowingKnife as u8, 9);
        assert_eq!(ShootType::SmallStone as u8, 10);
        assert_eq!(ShootType::Death as u8, 11);
        assert_eq!(ShootType::LargeRock as u8, 12);
        assert_eq!(ShootType::Snowball as u8, 13);
        assert_eq!(ShootType::PowerBolt as u8, 14);
        assert_eq!(ShootType::Poison as u8, 15);
        assert_eq!(ShootType::InfernalBolt as u8, 16);
        assert_eq!(ShootType::HuntingSpear as u8, 17);
        assert_eq!(ShootType::EnchantedSpear as u8, 18);
        assert_eq!(ShootType::RedStar as u8, 19);
        assert_eq!(ShootType::GreenStar as u8, 20);
        assert_eq!(ShootType::RoyalSpear as u8, 21);
        assert_eq!(ShootType::SniperArrow as u8, 22);
        assert_eq!(ShootType::OnyxArrow as u8, 23);
        assert_eq!(ShootType::PiercingBolt as u8, 24);
        assert_eq!(ShootType::WhirlwindSword as u8, 25);
        assert_eq!(ShootType::WhirlwindAxe as u8, 26);
        assert_eq!(ShootType::WhirlwindClub as u8, 27);
        assert_eq!(ShootType::EtherealSpear as u8, 28);
        assert_eq!(ShootType::Ice as u8, 29);
        assert_eq!(ShootType::Earth as u8, 30);
        assert_eq!(ShootType::Holy as u8, 31);
        assert_eq!(ShootType::SuddenDeath as u8, 32);
        assert_eq!(ShootType::FlashArrow as u8, 33);
        assert_eq!(ShootType::FlammingArrow as u8, 34);
        assert_eq!(ShootType::ShiverArrow as u8, 35);
        assert_eq!(ShootType::EnergyBall as u8, 36);
        assert_eq!(ShootType::SmallIce as u8, 37);
        assert_eq!(ShootType::SmallHoly as u8, 38);
        assert_eq!(ShootType::SmallEarth as u8, 39);
        assert_eq!(ShootType::EarthArrow as u8, 40);
        assert_eq!(ShootType::Explosion as u8, 41);
        assert_eq!(ShootType::Cake as u8, 42);
        assert_eq!(ShootType::TarsalArrow as u8, 44);
        assert_eq!(ShootType::VortexBolt as u8, 45);
        assert_eq!(ShootType::PrismaticBolt as u8, 48);
        assert_eq!(ShootType::CrystallineArrow as u8, 49);
        assert_eq!(ShootType::DrillBolt as u8, 50);
        assert_eq!(ShootType::EnvenomedArrow as u8, 51);
        assert_eq!(ShootType::GloothSpear as u8, 53);
        assert_eq!(ShootType::SimpleArrow as u8, 54);
        assert_eq!(ShootType::LeafStar as u8, 56);
        assert_eq!(ShootType::DiamondArrow as u8, 57);
        assert_eq!(ShootType::SpectralBolt as u8, 58);
        assert_eq!(ShootType::RoyalStar as u8, 59);
        assert_eq!(ShootType::WeaponType as u8, 0xFE);
    }

    #[test]
    fn test_speak_class_all_variants_exact_values() {
        assert_eq!(SpeakClass::Say as u8, 1);
        assert_eq!(SpeakClass::Whisper as u8, 2);
        assert_eq!(SpeakClass::Yell as u8, 3);
        assert_eq!(SpeakClass::PrivateFrom as u8, 4);
        assert_eq!(SpeakClass::PrivateTo as u8, 5);
        assert_eq!(SpeakClass::ChannelY as u8, 7);
        assert_eq!(SpeakClass::ChannelO as u8, 8);
        assert_eq!(SpeakClass::Spell as u8, 9);
        assert_eq!(SpeakClass::PrivateNp as u8, 10);
        assert_eq!(SpeakClass::PrivateNpConsole as u8, 11);
        assert_eq!(SpeakClass::PrivatePn as u8, 12);
        assert_eq!(SpeakClass::Broadcast as u8, 13);
        assert_eq!(SpeakClass::ChannelR1 as u8, 14);
        assert_eq!(SpeakClass::PrivateRedFrom as u8, 15);
        assert_eq!(SpeakClass::PrivateRedTo as u8, 16);
        assert_eq!(SpeakClass::MonsterSay as u8, 36);
        assert_eq!(SpeakClass::MonsterYell as u8, 37);
        assert_eq!(SpeakClass::Potion as u8, 52);
    }

    #[test]
    fn test_message_class_all_variants_exact_values() {
        assert_eq!(MessageClass::StatusDefault as u8, 17);
        assert_eq!(MessageClass::StatusWarning as u8, 18);
        assert_eq!(MessageClass::EventAdvance as u8, 19);
        assert_eq!(MessageClass::StatusWarning2 as u8, 20);
        assert_eq!(MessageClass::StatusSmall as u8, 21);
        assert_eq!(MessageClass::InfoDescr as u8, 22);
        assert_eq!(MessageClass::DamageDealt as u8, 23);
        assert_eq!(MessageClass::DamageReceived as u8, 24);
        assert_eq!(MessageClass::Healed as u8, 25);
        assert_eq!(MessageClass::Experience as u8, 26);
        assert_eq!(MessageClass::DamageOthers as u8, 27);
        assert_eq!(MessageClass::HealedOthers as u8, 28);
        assert_eq!(MessageClass::ExperienceOthers as u8, 29);
        assert_eq!(MessageClass::EventDefault as u8, 30);
        assert_eq!(MessageClass::Loot as u8, 31);
        assert_eq!(MessageClass::Trade as u8, 32);
        assert_eq!(MessageClass::Guild as u8, 33);
        assert_eq!(MessageClass::PartyManagement as u8, 34);
        assert_eq!(MessageClass::Party as u8, 35);
        assert_eq!(MessageClass::Report as u8, 38);
        assert_eq!(MessageClass::HotkeyPressed as u8, 39);
        assert_eq!(MessageClass::Market as u8, 42);
        assert_eq!(MessageClass::BeyondLast as u8, 44);
        assert_eq!(MessageClass::TournamentInfo as u8, 45);
        assert_eq!(MessageClass::Attention as u8, 48);
        assert_eq!(MessageClass::BoostedCreature as u8, 49);
        assert_eq!(MessageClass::OfflineTraining as u8, 50);
        assert_eq!(MessageClass::Transaction as u8, 51);
    }

    #[test]
    fn test_fluid_color_all_variants_exact_values() {
        assert_eq!(FluidColor::Empty as u8, 0);
        assert_eq!(FluidColor::Blue as u8, 1);
        assert_eq!(FluidColor::Red as u8, 2);
        assert_eq!(FluidColor::Brown as u8, 3);
        assert_eq!(FluidColor::Green as u8, 4);
        assert_eq!(FluidColor::Yellow as u8, 5);
        assert_eq!(FluidColor::White as u8, 6);
        assert_eq!(FluidColor::Purple as u8, 7);
        assert_eq!(FluidColor::Black as u8, 8);
    }

    #[test]
    fn test_fluid_type_all_variants_exact_values() {
        assert_eq!(FluidType::None as u8, 0);
        assert_eq!(FluidType::Water as u8, 1);
        assert_eq!(FluidType::Blood as u8, 2);
        assert_eq!(FluidType::Beer as u8, 3);
        assert_eq!(FluidType::Slime as u8, 4);
        assert_eq!(FluidType::Lemonade as u8, 5);
        assert_eq!(FluidType::Milk as u8, 6);
        assert_eq!(FluidType::Mana as u8, 7);
        assert_eq!(FluidType::Ink as u8, 8);
        assert_eq!(FluidType::Life as u8, 10);
        assert_eq!(FluidType::Oil as u8, 11);
        assert_eq!(FluidType::Urine as u8, 13);
        assert_eq!(FluidType::CoconutMilk as u8, 14);
        assert_eq!(FluidType::Wine as u8, 15);
        assert_eq!(FluidType::Mud as u8, 19);
        assert_eq!(FluidType::FruitJuice as u8, 21);
        assert_eq!(FluidType::Lava as u8, 26);
        assert_eq!(FluidType::Rum as u8, 27);
        assert_eq!(FluidType::Swamp as u8, 28);
        assert_eq!(FluidType::Tea as u8, 35);
        assert_eq!(FluidType::Mead as u8, 43);
    }

    #[test]
    fn test_client_fluid_type_all_variants_exact_values() {
        assert_eq!(ClientFluidType::Empty as u8, 0);
        assert_eq!(ClientFluidType::Blue as u8, 1);
        assert_eq!(ClientFluidType::Purple as u8, 2);
        assert_eq!(ClientFluidType::Brown1 as u8, 3);
        assert_eq!(ClientFluidType::Brown2 as u8, 4);
        assert_eq!(ClientFluidType::Red as u8, 5);
        assert_eq!(ClientFluidType::Green as u8, 6);
        assert_eq!(ClientFluidType::Brown as u8, 7);
        assert_eq!(ClientFluidType::Yellow as u8, 8);
        assert_eq!(ClientFluidType::White as u8, 9);
        assert_eq!(ClientFluidType::Black as u8, 18);
    }

    #[test]
    fn test_square_color_all_variants_exact_values() {
        assert_eq!(SquareColor::Black as u8, 0);
    }

    #[test]
    fn test_text_color_all_variants_exact_values() {
        assert_eq!(TextColor::Blue as u8, 5);
        assert_eq!(TextColor::LightGreen as u8, 30);
        assert_eq!(TextColor::LightBlue as u8, 35);
        assert_eq!(TextColor::DarkGrey as u8, 86);
        assert_eq!(TextColor::MayaBlue as u8, 95);
        assert_eq!(TextColor::DarkRed as u8, 108);
        assert_eq!(TextColor::LightGrey as u8, 129);
        assert_eq!(TextColor::SkyBlue as u8, 143);
        assert_eq!(TextColor::Purple as u8, 154);
        assert_eq!(TextColor::ElectricPurple as u8, 155);
        assert_eq!(TextColor::Red as u8, 180);
        assert_eq!(TextColor::PastelRed as u8, 194);
        assert_eq!(TextColor::Orange as u8, 198);
        assert_eq!(TextColor::Yellow as u8, 210);
        assert_eq!(TextColor::WhiteExp as u8, 215);
        assert_eq!(TextColor::None as u8, 255);
    }

    #[test]
    fn test_icon_flags_all_constants_exact_values() {
        assert_eq!(IconFlags::POISON, 1 << 0);
        assert_eq!(IconFlags::BURN, 1 << 1);
        assert_eq!(IconFlags::ENERGY, 1 << 2);
        assert_eq!(IconFlags::DRUNK, 1 << 3);
        assert_eq!(IconFlags::MANASHIELD, 1 << 4);
        assert_eq!(IconFlags::PARALYZE, 1 << 5);
        assert_eq!(IconFlags::HASTE, 1 << 6);
        assert_eq!(IconFlags::SWORDS, 1 << 7);
        assert_eq!(IconFlags::DROWNING, 1 << 8);
        assert_eq!(IconFlags::FREEZING, 1 << 9);
        assert_eq!(IconFlags::DAZZLED, 1 << 10);
        assert_eq!(IconFlags::CURSED, 1 << 11);
        assert_eq!(IconFlags::PARTY_BUFF, 1 << 12);
        assert_eq!(IconFlags::REDSWORDS, 1 << 13);
        assert_eq!(IconFlags::PIGEON, 1 << 14);
        assert_eq!(IconFlags::BLEEDING, 1 << 15);
        assert_eq!(IconFlags::LESSERHEX, 1 << 16);
        assert_eq!(IconFlags::INTENSEHEX, 1 << 17);
        assert_eq!(IconFlags::GREATERHEX, 1 << 18);
        assert_eq!(IconFlags::ROOT, 1 << 19);
        assert_eq!(IconFlags::FEAR, 1 << 20);
        assert_eq!(IconFlags::GOSHNAR1, 1 << 21);
        assert_eq!(IconFlags::GOSHNAR2, 1 << 22);
        assert_eq!(IconFlags::GOSHNAR3, 1 << 23);
        assert_eq!(IconFlags::GOSHNAR4, 1 << 24);
        assert_eq!(IconFlags::GOSHNAR5, 1 << 25);
        assert_eq!(IconFlags::MANASHIELD_BREAKABLE, 1 << 26);
    }

    #[test]
    fn test_ammo_all_variants_exact_values() {
        assert_eq!(Ammo::None as u8, 0);
        assert_eq!(Ammo::Bolt as u8, 1);
        assert_eq!(Ammo::Arrow as u8, 2);
        assert_eq!(Ammo::Spear as u8, 3);
        assert_eq!(Ammo::ThrowingStar as u8, 4);
        assert_eq!(Ammo::ThrowingKnife as u8, 5);
        assert_eq!(Ammo::Stone as u8, 6);
        assert_eq!(Ammo::Snowball as u8, 7);
    }

    #[test]
    fn test_weapon_action_all_variants_exact_values() {
        assert_eq!(WeaponAction::None as u8, 0);
        assert_eq!(WeaponAction::RemoveCount as u8, 1);
        assert_eq!(WeaponAction::RemoveCharge as u8, 2);
        assert_eq!(WeaponAction::Move as u8, 3);
    }

    #[test]
    fn test_wield_info_all_constants_exact_values() {
        assert_eq!(WieldInfo::NONE, 0);
        assert_eq!(WieldInfo::LEVEL, 1 << 0);
        assert_eq!(WieldInfo::MAGLV, 1 << 1);
        assert_eq!(WieldInfo::VOCREQ, 1 << 2);
        assert_eq!(WieldInfo::PREMIUM, 1 << 3);
    }

    #[test]
    fn test_skull_all_variants_exact_values() {
        assert_eq!(Skull::None as u8, 0);
        assert_eq!(Skull::Yellow as u8, 1);
        assert_eq!(Skull::Green as u8, 2);
        assert_eq!(Skull::White as u8, 3);
        assert_eq!(Skull::Red as u8, 4);
        assert_eq!(Skull::Black as u8, 5);
        assert_eq!(Skull::Orange as u8, 6);
    }

    #[test]
    fn test_party_shield_all_variants_exact_values() {
        assert_eq!(PartyShield::None as u8, 0);
        assert_eq!(PartyShield::WhiteYellow as u8, 1);
        assert_eq!(PartyShield::WhiteBlue as u8, 2);
        assert_eq!(PartyShield::Blue as u8, 3);
        assert_eq!(PartyShield::Yellow as u8, 4);
        assert_eq!(PartyShield::BlueSharedExp as u8, 5);
        assert_eq!(PartyShield::YellowSharedExp as u8, 6);
        assert_eq!(PartyShield::BlueNoSharedExpBlink as u8, 7);
        assert_eq!(PartyShield::YellowNoSharedExpBlink as u8, 8);
        assert_eq!(PartyShield::BlueNoSharedExp as u8, 9);
        assert_eq!(PartyShield::YellowNoSharedExp as u8, 10);
        assert_eq!(PartyShield::Gray as u8, 11);
    }

    #[test]
    fn test_guild_emblem_all_variants_exact_values() {
        assert_eq!(GuildEmblem::None as u8, 0);
        assert_eq!(GuildEmblem::Ally as u8, 1);
        assert_eq!(GuildEmblem::Enemy as u8, 2);
        assert_eq!(GuildEmblem::Neutral as u8, 3);
        assert_eq!(GuildEmblem::Member as u8, 4);
        assert_eq!(GuildEmblem::Other as u8, 5);
    }

    #[test]
    fn test_item_id_all_variants_exact_values() {
        assert_eq!(ItemId::BrowseField as u16, 460);
        assert_eq!(ItemId::DecorationKit as u16, 26054);
        assert_eq!(ItemId::FirefieldPvpFull as u16, 1487);
        assert_eq!(ItemId::FirefieldPvpMedium as u16, 1488);
        assert_eq!(ItemId::FirefieldPvpSmall as u16, 1489);
        assert_eq!(ItemId::FirefieldPersistentFull as u16, 1492);
        assert_eq!(ItemId::FirefieldPersistentMedium as u16, 1493);
        assert_eq!(ItemId::FirefieldPersistentSmall as u16, 1494);
        assert_eq!(ItemId::FirefieldNopvp as u16, 1500);
        assert_eq!(ItemId::FirefieldNopvpMedium as u16, 1501);
        assert_eq!(ItemId::PoisonfieldPvp as u16, 1490);
        assert_eq!(ItemId::PoisonfieldPersistent as u16, 1496);
        assert_eq!(ItemId::PoisonfieldNopvp as u16, 1503);
        assert_eq!(ItemId::EnergyfieldPvp as u16, 1491);
        assert_eq!(ItemId::EnergyfieldPersistent as u16, 1495);
        assert_eq!(ItemId::EnergyfieldNopvp as u16, 1504);
        assert_eq!(ItemId::Magicwall as u16, 1497);
        assert_eq!(ItemId::MagicwallPersistent as u16, 1498);
        assert_eq!(ItemId::MagicwallSafe as u16, 11098);
        assert_eq!(ItemId::MagicwallNopvp as u16, 20669);
        assert_eq!(ItemId::Wildgrowth as u16, 1499);
        assert_eq!(ItemId::WildgrowthPersistent as u16, 2721);
        assert_eq!(ItemId::WildgrowthSafe as u16, 11099);
        assert_eq!(ItemId::WildgrowthNopvp as u16, 20670);
        assert_eq!(ItemId::Bag as u16, 1987);
        assert_eq!(ItemId::ShoppingBag as u16, 23782);
        assert_eq!(ItemId::GoldCoin as u16, 2148);
        assert_eq!(ItemId::PlatinumCoin as u16, 2152);
        assert_eq!(ItemId::CrystalCoin as u16, 2160);
        assert_eq!(ItemId::StoreCoin as u16, 24774);
        assert_eq!(ItemId::Depot as u16, 2594);
        assert_eq!(ItemId::Locker as u16, 2589);
        assert_eq!(ItemId::Inbox as u16, 14404);
        assert_eq!(ItemId::Market as u16, 14405);
        assert_eq!(ItemId::StoreInbox as u16, 26052);
        assert_eq!(ItemId::DepotBoxI as u16, 25453);
        assert_eq!(ItemId::DepotBoxII as u16, 25454);
        assert_eq!(ItemId::DepotBoxIII as u16, 25455);
        assert_eq!(ItemId::DepotBoxIV as u16, 25456);
        assert_eq!(ItemId::DepotBoxV as u16, 25457);
        assert_eq!(ItemId::DepotBoxVI as u16, 25458);
        assert_eq!(ItemId::DepotBoxVII as u16, 25459);
        assert_eq!(ItemId::DepotBoxVIII as u16, 25460);
        assert_eq!(ItemId::DepotBoxIX as u16, 25461);
        assert_eq!(ItemId::DepotBoxX as u16, 25462);
        assert_eq!(ItemId::DepotBoxXI as u16, 25463);
        assert_eq!(ItemId::DepotBoxXII as u16, 25464);
        assert_eq!(ItemId::DepotBoxXIII as u16, 25465);
        assert_eq!(ItemId::DepotBoxXIV as u16, 25466);
        assert_eq!(ItemId::DepotBoxXV as u16, 25467);
        assert_eq!(ItemId::DepotBoxXVI as u16, 25468);
        assert_eq!(ItemId::DepotBoxXVII as u16, 25469);
        assert_eq!(ItemId::DepotBoxXVIII as u16, 34571);
        assert_eq!(ItemId::DepotBoxXIX as u16, 44714);
        assert_eq!(ItemId::DepotBoxXX as u16, 44715);
        assert_eq!(ItemId::MaleCorpse as u16, 3058);
        assert_eq!(ItemId::FemaleCorpse as u16, 3065);
        assert_eq!(ItemId::FullSplash as u16, 2016);
        assert_eq!(ItemId::SmallSplash as u16, 2019);
        assert_eq!(ItemId::Parcel as u16, 2595);
        assert_eq!(ItemId::Letter as u16, 2597);
        assert_eq!(ItemId::LetterStamped as u16, 2598);
        assert_eq!(ItemId::Label as u16, 2599);
        assert_eq!(ItemId::AmuletOfLoss as u16, 2173);
        assert_eq!(ItemId::DocumentRo as u16, 1968);
    }

    #[test]
    fn test_player_flags_all_constants_exact_values() {
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
        assert_eq!(PlayerFlags::IGNORE_PROTECTION_ZONE, 1u64 << 33);
        assert_eq!(PlayerFlags::IGNORE_SPELL_CHECK, 1u64 << 34);
        assert_eq!(PlayerFlags::IGNORE_WEAPON_CHECK, 1u64 << 35);
        assert_eq!(PlayerFlags::CANNOT_BE_MUTED, 1u64 << 36);
        assert_eq!(PlayerFlags::IS_ALWAYS_PREMIUM, 1u64 << 37);
        assert_eq!(PlayerFlags::IGNORE_YELL_CHECK, 1u64 << 38);
        assert_eq!(PlayerFlags::IGNORE_SEND_PRIVATE_CHECK, 1u64 << 39);
    }

    #[test]
    fn test_monster_icon_all_variants_exact_values() {
        assert_eq!(MonsterIcon::Vulnerable as u8, 1);
        assert_eq!(MonsterIcon::Weakened as u8, 2);
        assert_eq!(MonsterIcon::Melee as u8, 3);
        assert_eq!(MonsterIcon::Influenced as u8, 4);
        assert_eq!(MonsterIcon::Fiendish as u8, 5);
        assert_eq!(MonsterIcon::FIRST as u8, 1);
        assert_eq!(MonsterIcon::LAST as u8, 5);
    }

    #[test]
    fn test_creature_icon_all_variants_exact_values() {
        assert_eq!(CreatureIcon::CrossWhite as u8, 1);
        assert_eq!(CreatureIcon::CrossWhiteRed as u8, 2);
        assert_eq!(CreatureIcon::OrbRed as u8, 3);
        assert_eq!(CreatureIcon::OrbGreen as u8, 4);
        assert_eq!(CreatureIcon::OrbRedGreen as u8, 5);
        assert_eq!(CreatureIcon::GemGreen as u8, 6);
        assert_eq!(CreatureIcon::GemYellow as u8, 7);
        assert_eq!(CreatureIcon::GemBlue as u8, 8);
        assert_eq!(CreatureIcon::GemPurple as u8, 9);
        assert_eq!(CreatureIcon::GemRed as u8, 10);
        assert_eq!(CreatureIcon::Pigeon as u8, 11);
        assert_eq!(CreatureIcon::Energy as u8, 12);
        assert_eq!(CreatureIcon::Poison as u8, 13);
        assert_eq!(CreatureIcon::Water as u8, 14);
        assert_eq!(CreatureIcon::Fire as u8, 15);
        assert_eq!(CreatureIcon::Ice as u8, 16);
        assert_eq!(CreatureIcon::ArrowUp as u8, 17);
        assert_eq!(CreatureIcon::ArrowDown as u8, 18);
        assert_eq!(CreatureIcon::Warning as u8, 19);
        assert_eq!(CreatureIcon::Question as u8, 20);
        assert_eq!(CreatureIcon::CrossRed as u8, 21);
        assert_eq!(CreatureIcon::FIRST as u8, 1);
        assert_eq!(CreatureIcon::LAST as u8, 21);
    }
}
