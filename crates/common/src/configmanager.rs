//! Migrated from forgottenserver/src/configmanager.h + configmanager.cpp
//!
//! The original C++ reads a `config.lua` file via a Lua interpreter.
//! Since `mlua` is not available in the common crate, we implement a
//! lightweight parser that understands the Lua *assignment* subset used
//! by the game's configuration files:
//!
//! ```lua
//! serverName = "My Server"
//! gameProtocolPort = 7172
//! freePremium = true
//! ```
//!
//! Supported right-hand-side forms:
//!   * `"…"` or `'…'`  → string
//!   * bare integer literal (optional leading `-`)  → integer (also used as string)
//!   * `true` / `false`  → boolean (also exposed as string "true"/"false")
//!   * `nil`             → empty string / 0 / false
//!
//! All enum keys mirror the C++ `ConfigManager` namespace enums exactly.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Enum key types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BooleanKey {
    AllowChangeOutfit,
    OnePlayerOnAccount,
    AimbotHotkeyEnabled,
    RemoveRuneCharges,
    RemoveWeaponAmmo,
    RemoveWeaponCharges,
    RemovePotionCharges,
    ExperienceFromPlayers,
    FreePremium,
    ReplaceKickOnLogin,
    AllowClones,
    AllowWalkthrough,
    BindOnlyGlobalAddress,
    OptimizeDatabase,
    MarketPremium,
    EmoteSpells,
    StaminaSystem,
    WarnUnsafeScripts,
    ConvertUnsafeScripts,
    ClassicEquipmentSlots,
    ClassicAttackSpeed,
    ScriptsConsoleLogs,
    ServerSaveNotifyMessage,
    ServerSaveCleanMap,
    ServerSaveClose,
    ServerSaveShutdown,
    OnlineOfflineCharlist,
    YellAllowPremium,
    PremiumToSendPrivate,
    ForceMonsterTypeLoad,
    HouseOwnedByAccount,
    CleanProtectionZones,
    HouseDoorShowPrice,
    OnlyInvitedCanMoveHouseItems,
    RemoveOnDespawn,
    TwoFactorAuth,
    ManashieldBreakable,
    CheckDuplicateStorageKeys,
    MonsterOverspawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StringKey {
    MapName,
    HouseRentPeriod,
    ServerName,
    OwnerName,
    OwnerEmail,
    Url,
    Location,
    Ip,
    WorldType,
    MysqlHost,
    MysqlUser,
    MysqlPass,
    MysqlDb,
    MysqlSock,
    DefaultPriority,
    MapAuthor,
    ConfigFile,
    AdminPassword,
    Motd,
    HttpBindAddress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegerKey {
    SqlPort,
    MaxPlayers,
    PzLocked,
    DefaultDespawnRange,
    DefaultDespawnRadius,
    DefaultWalkToSpawnRadius,
    RateExperience,
    RateSkill,
    RateLoot,
    RateMagic,
    RateSpawn,
    HousePrice,
    KillsToRed,
    KillsToBlack,
    MaxMessageBuffer,
    ActionsDelayInterval,
    ExActionsDelayInterval,
    KickAfterMinutes,
    ProtectionLevel,
    DeathLosePercent,
    StatusQueryTimeout,
    StatusCountMaxPlayersPerIp,
    FragTime,
    WhiteSkullTime,
    GamePort,
    StatusPort,
    HttpPort,
    HttpWorkers,
    StairhopDelay,
    MarketOfferDuration,
    CheckExpiredMarketOffersEachMinutes,
    MaxMarketOffersAtATimePerPlayer,
    ExpFromPlayersLevelRange,
    MaxPacketsPerSecond,
    ServerSaveNotifyDuration,
    YellMinimumLevel,
    MinimumLevelToSendPrivate,
    VipFreeLimit,
    VipPremiumLimit,
    DepotFreeLimit,
    DepotPremiumLimit,
    QuestTrackerFreeLimit,
    QuestTrackerPremiumLimit,
    StaminaRegenMinute,
    StaminaRegenPremium,
    PathfindingInterval,
    PathfindingDelay,
    AdminPort,
}

/// Unified key type for the [`ConfigManager`] store.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigKey {
    Boolean(BooleanKey),
    String(StringKey),
    Integer(IntegerKey),
}

impl From<BooleanKey> for ConfigKey {
    fn from(k: BooleanKey) -> Self {
        ConfigKey::Boolean(k)
    }
}

impl From<StringKey> for ConfigKey {
    fn from(k: StringKey) -> Self {
        ConfigKey::String(k)
    }
}

impl From<IntegerKey> for ConfigKey {
    fn from(k: IntegerKey) -> Self {
        ConfigKey::Integer(k)
    }
}

// ---------------------------------------------------------------------------
// Stored value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum ConfigValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

// ---------------------------------------------------------------------------
// ConfigManager
// ---------------------------------------------------------------------------

/// Lightweight configuration manager that reads Lua-style assignment files.
#[derive(Debug, Default)]
pub struct ConfigManager {
    strings: HashMap<StringKey, String>,
    integers: HashMap<IntegerKey, i64>,
    booleans: HashMap<BooleanKey, bool>,
    /// Experience stages: tuples of `(min_level, max_level, multiplier)`.
    /// Mirrors the C++ `expStages` table populated by `loadXMLStages` /
    /// `loadLuaStages` and consumed by `getExperienceStage`.
    experience_stages: Vec<(u32, u32, f32)>,
}

impl ConfigManager {
    /// Create a new, empty `ConfigManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse the config file at `path`.
    ///
    /// Each non-empty, non-comment line is expected to be of the form:
    /// ```text
    /// key = value
    /// ```
    /// where `value` is a quoted string, integer, or boolean literal.
    ///
    /// Unknown keys are silently ignored.
    ///
    /// # Errors
    /// Returns `Err` when the file cannot be read.
    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot open config '{}': {}", path.display(), e))?;

        for line in contents.lines() {
            let line = Self::strip_comment(line).trim().to_owned();
            if line.is_empty() {
                continue;
            }
            let Some((key_str, val_str)) = line.split_once('=') else {
                continue;
            };
            let key_str = key_str.trim();
            let val_str = val_str.trim();

            let value = Self::parse_value(val_str);
            self.apply(key_str, value);
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Getters
    // -----------------------------------------------------------------------

    /// Return the string value for `key`, or `""` if not set.
    pub fn get_string(&self, key: StringKey) -> &str {
        self.strings.get(&key).map(|s| s.as_str()).unwrap_or("")
    }

    /// Return the integer value for `key`, or `0` if not set.
    pub fn get_integer(&self, key: IntegerKey) -> i64 {
        self.integers.get(&key).copied().unwrap_or(0)
    }

    /// Return the boolean value for `key`, or `false` if not set.
    pub fn get_boolean(&self, key: BooleanKey) -> bool {
        self.booleans.get(&key).copied().unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Setters (runtime overrides)
    // -----------------------------------------------------------------------

    /// Override a string config value at runtime.
    pub fn set_string(&mut self, key: StringKey, value: &str) {
        self.strings.insert(key, value.to_owned());
    }

    /// Override an integer config value at runtime.
    pub fn set_integer(&mut self, key: IntegerKey, value: i64) {
        self.integers.insert(key, value);
    }

    /// Override a boolean config value at runtime.
    pub fn set_boolean(&mut self, key: BooleanKey, value: bool) {
        self.booleans.insert(key, value);
    }

    // -----------------------------------------------------------------------
    // Experience stages — mirrors C++ ConfigManager::getExperienceStage
    // -----------------------------------------------------------------------

    /// Append an experience stage `(min_level, max_level, multiplier)`.
    ///
    /// The stages are searched in insertion order by
    /// [`get_experience_stage`](Self::get_experience_stage); the first stage
    /// whose `[min_level, max_level]` band contains `level` wins. This matches
    /// the C++ behaviour where stages are pre-sorted by `min_level` and the
    /// first containing band is returned.
    pub fn add_experience_stage(&mut self, min_level: u32, max_level: u32, multiplier: f32) {
        self.experience_stages
            .push((min_level, max_level, multiplier));
    }

    /// Return the experience multiplier for the given character `level`.
    ///
    /// Behaviour mirrors C++ `ConfigManager::getExperienceStage`: if no stage
    /// covers `level`, the fallback is the integer `RateExperience` config
    /// value cast to `f32`.
    pub fn get_experience_stage(&self, level: u32) -> f32 {
        for &(min_level, max_level, multiplier) in &self.experience_stages {
            if level >= min_level && level <= max_level {
                return multiplier;
            }
        }
        self.get_integer(IntegerKey::RateExperience) as f32
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Strip a Lua `--` line comment (outside of strings — simplified).
    fn strip_comment(line: &str) -> &str {
        if let Some(pos) = line.find("--") {
            &line[..pos]
        } else {
            line
        }
    }

    /// Parse a Lua RHS value token into a typed [`ConfigValue`].
    fn parse_value(raw: &str) -> ConfigValue {
        // Quoted string
        if (raw.starts_with('"') && raw.ends_with('"'))
            || (raw.starts_with('\'') && raw.ends_with('\''))
        {
            let inner = &raw[1..raw.len() - 1];
            return ConfigValue::Str(inner.to_owned());
        }

        // Boolean literals
        if raw == "true" {
            return ConfigValue::Bool(true);
        }
        if raw == "false" {
            return ConfigValue::Bool(false);
        }

        // nil → zero / empty
        if raw == "nil" {
            return ConfigValue::Str(String::new());
        }

        // Integer
        if let Ok(n) = raw.parse::<i64>() {
            return ConfigValue::Int(n);
        }

        // Fallback: treat as bare string
        ConfigValue::Str(raw.to_owned())
    }

    /// Map the Lua global variable name to one of our typed keys and store the value.
    fn apply(&mut self, lua_key: &str, value: ConfigValue) {
        // String keys
        if let Some(sk) = Self::string_key(lua_key) {
            let s = match &value {
                ConfigValue::Str(s) => s.clone(),
                ConfigValue::Int(n) => n.to_string(),
                ConfigValue::Bool(b) => b.to_string(),
            };
            self.strings.insert(sk, s);
            return;
        }

        // Integer keys
        if let Some(ik) = Self::integer_key(lua_key) {
            let n = match &value {
                ConfigValue::Int(n) => *n,
                ConfigValue::Str(s) => s.parse().unwrap_or(0),
                ConfigValue::Bool(b) => *b as i64,
            };
            self.integers.insert(ik, n);
            return;
        }

        // Boolean keys
        if let Some(bk) = Self::boolean_key(lua_key) {
            let b = match &value {
                ConfigValue::Bool(b) => *b,
                ConfigValue::Int(n) => *n != 0,
                ConfigValue::Str(s) => matches!(s.as_str(), "true" | "yes" | "1"),
            };
            self.booleans.insert(bk, b);
        }
        // Unknown keys are silently ignored
    }

    /// Map Lua variable name → StringKey
    fn string_key(name: &str) -> Option<StringKey> {
        match name {
            "mapName" => Some(StringKey::MapName),
            "houseRentPeriod" => Some(StringKey::HouseRentPeriod),
            "serverName" => Some(StringKey::ServerName),
            "ownerName" => Some(StringKey::OwnerName),
            "ownerEmail" => Some(StringKey::OwnerEmail),
            "url" => Some(StringKey::Url),
            "location" => Some(StringKey::Location),
            "ip" => Some(StringKey::Ip),
            "worldType" => Some(StringKey::WorldType),
            "mysqlHost" => Some(StringKey::MysqlHost),
            "mysqlUser" => Some(StringKey::MysqlUser),
            "mysqlPass" => Some(StringKey::MysqlPass),
            "mysqlDatabase" => Some(StringKey::MysqlDb),
            "mysqlSock" => Some(StringKey::MysqlSock),
            "defaultPriority" => Some(StringKey::DefaultPriority),
            "mapAuthor" => Some(StringKey::MapAuthor),
            "adminPassword" => Some(StringKey::AdminPassword),
            "motd" => Some(StringKey::Motd),
            "configFile" => Some(StringKey::ConfigFile),
            "httpLoginBindAddress" => Some(StringKey::HttpBindAddress),
            _ => None,
        }
    }

    /// Map Lua variable name → IntegerKey
    fn integer_key(name: &str) -> Option<IntegerKey> {
        match name {
            "mysqlPort" => Some(IntegerKey::SqlPort),
            "maxPlayers" => Some(IntegerKey::MaxPlayers),
            "pzLocked" => Some(IntegerKey::PzLocked),
            "deSpawnRange" => Some(IntegerKey::DefaultDespawnRange),
            "deSpawnRadius" => Some(IntegerKey::DefaultDespawnRadius),
            "walkToSpawnRadius" => Some(IntegerKey::DefaultWalkToSpawnRadius),
            "rateExp" => Some(IntegerKey::RateExperience),
            "rateSkill" => Some(IntegerKey::RateSkill),
            "rateLoot" => Some(IntegerKey::RateLoot),
            "rateMagic" => Some(IntegerKey::RateMagic),
            "rateSpawn" => Some(IntegerKey::RateSpawn),
            "housePriceEachSQM" => Some(IntegerKey::HousePrice),
            "killsToRedSkull" => Some(IntegerKey::KillsToRed),
            "killsToBlackSkull" => Some(IntegerKey::KillsToBlack),
            "maxMessageBuffer" => Some(IntegerKey::MaxMessageBuffer),
            "timeBetweenActions" => Some(IntegerKey::ActionsDelayInterval),
            "timeBetweenExActions" => Some(IntegerKey::ExActionsDelayInterval),
            "kickIdlePlayerAfterMinutes" => Some(IntegerKey::KickAfterMinutes),
            "protectionLevel" => Some(IntegerKey::ProtectionLevel),
            "deathLosePercent" => Some(IntegerKey::DeathLosePercent),
            "statusTimeout" => Some(IntegerKey::StatusQueryTimeout),
            "statusCountMaxPlayersPerIp" => Some(IntegerKey::StatusCountMaxPlayersPerIp),
            "timeToDecreaseFrags" => Some(IntegerKey::FragTime),
            "whiteSkullTime" => Some(IntegerKey::WhiteSkullTime),
            "gameProtocolPort" => Some(IntegerKey::GamePort),
            "statusProtocolPort" => Some(IntegerKey::StatusPort),
            "httpPort" => Some(IntegerKey::HttpPort),
            "httpWorkers" => Some(IntegerKey::HttpWorkers),
            "stairJumpExhaustion" => Some(IntegerKey::StairhopDelay),
            "marketOfferDuration" => Some(IntegerKey::MarketOfferDuration),
            "checkExpiredMarketOffersEachMinutes" => {
                Some(IntegerKey::CheckExpiredMarketOffersEachMinutes)
            }
            "maxMarketOffersAtATimePerPlayer" => Some(IntegerKey::MaxMarketOffersAtATimePerPlayer),
            "expFromPlayersLevelRange" => Some(IntegerKey::ExpFromPlayersLevelRange),
            "maxPacketsPerSecond" => Some(IntegerKey::MaxPacketsPerSecond),
            "serverSaveNotifyDuration" => Some(IntegerKey::ServerSaveNotifyDuration),
            "yellMinimumLevel" => Some(IntegerKey::YellMinimumLevel),
            "minimumLevelToSendPrivate" => Some(IntegerKey::MinimumLevelToSendPrivate),
            "vipFreeLimit" => Some(IntegerKey::VipFreeLimit),
            "vipPremiumLimit" => Some(IntegerKey::VipPremiumLimit),
            "depotFreeLimit" => Some(IntegerKey::DepotFreeLimit),
            "depotPremiumLimit" => Some(IntegerKey::DepotPremiumLimit),
            "questTrackerFreeLimit" => Some(IntegerKey::QuestTrackerFreeLimit),
            "questTrackerPremiumLimit" => Some(IntegerKey::QuestTrackerPremiumLimit),
            "timeToRegenMinuteStamina" => Some(IntegerKey::StaminaRegenMinute),
            "timeToRegenMinutePremiumStamina" => Some(IntegerKey::StaminaRegenPremium),
            "pathfindingInterval" => Some(IntegerKey::PathfindingInterval),
            "pathfindingDelay" => Some(IntegerKey::PathfindingDelay),
            "adminPort" => Some(IntegerKey::AdminPort),
            "statusPort" => Some(IntegerKey::StatusPort),
            _ => None,
        }
    }

    /// Map Lua variable name → BooleanKey
    fn boolean_key(name: &str) -> Option<BooleanKey> {
        match name {
            "allowChangeOutfit" => Some(BooleanKey::AllowChangeOutfit),
            "onePlayerOnlinePerAccount" => Some(BooleanKey::OnePlayerOnAccount),
            "hotkeyAimbotEnabled" => Some(BooleanKey::AimbotHotkeyEnabled),
            "removeChargesFromRunes" => Some(BooleanKey::RemoveRuneCharges),
            "removeWeaponAmmunition" => Some(BooleanKey::RemoveWeaponAmmo),
            "removeWeaponCharges" => Some(BooleanKey::RemoveWeaponCharges),
            "removeChargesFromPotions" => Some(BooleanKey::RemovePotionCharges),
            "experienceByKillingPlayers" => Some(BooleanKey::ExperienceFromPlayers),
            "freePremium" => Some(BooleanKey::FreePremium),
            "replaceKickOnLogin" => Some(BooleanKey::ReplaceKickOnLogin),
            "allowClones" => Some(BooleanKey::AllowClones),
            "allowWalkthrough" => Some(BooleanKey::AllowWalkthrough),
            "bindOnlyGlobalAddress" => Some(BooleanKey::BindOnlyGlobalAddress),
            "startupDatabaseOptimization" => Some(BooleanKey::OptimizeDatabase),
            "premiumToCreateMarketOffer" => Some(BooleanKey::MarketPremium),
            "emoteSpells" => Some(BooleanKey::EmoteSpells),
            "staminaSystem" => Some(BooleanKey::StaminaSystem),
            "warnUnsafeScripts" => Some(BooleanKey::WarnUnsafeScripts),
            "convertUnsafeScripts" => Some(BooleanKey::ConvertUnsafeScripts),
            "classicEquipmentSlots" => Some(BooleanKey::ClassicEquipmentSlots),
            "classicAttackSpeed" => Some(BooleanKey::ClassicAttackSpeed),
            "showScriptsLogInConsole" => Some(BooleanKey::ScriptsConsoleLogs),
            "serverSaveNotifyMessage" => Some(BooleanKey::ServerSaveNotifyMessage),
            "serverSaveCleanMap" => Some(BooleanKey::ServerSaveCleanMap),
            "serverSaveClose" => Some(BooleanKey::ServerSaveClose),
            "serverSaveShutdown" => Some(BooleanKey::ServerSaveShutdown),
            "showOnlineStatusInCharlist" => Some(BooleanKey::OnlineOfflineCharlist),
            "yellAlwaysAllowPremium" => Some(BooleanKey::YellAllowPremium),
            "premiumToSendPrivate" => Some(BooleanKey::PremiumToSendPrivate),
            "forceMonsterTypesOnLoad" => Some(BooleanKey::ForceMonsterTypeLoad),
            "houseOwnedByAccount" => Some(BooleanKey::HouseOwnedByAccount),
            "cleanProtectionZones" => Some(BooleanKey::CleanProtectionZones),
            "houseDoorShowPrice" => Some(BooleanKey::HouseDoorShowPrice),
            "onlyInvitedCanMoveHouseItems" => Some(BooleanKey::OnlyInvitedCanMoveHouseItems),
            "removeOnDespawn" => Some(BooleanKey::RemoveOnDespawn),
            "enableTwoFactorAuth" => Some(BooleanKey::TwoFactorAuth),
            "useBreakableManaShield" => Some(BooleanKey::ManashieldBreakable),
            "checkDuplicateStorageKeys" => Some(BooleanKey::CheckDuplicateStorageKeys),
            "monsterOverspawn" => Some(BooleanKey::MonsterOverspawn),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_config(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("temp file");
        f.write_all(content.as_bytes()).expect("write");
        f
    }

    // -----------------------------------------------------------------------
    // Test 1: load on a temp file with known entries returns correct values
    // -----------------------------------------------------------------------
    #[test]
    fn test_load_known_string_entry() {
        let f = write_config(r#"serverName = "My Tibia""#);
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "My Tibia");
    }

    #[test]
    fn test_load_known_integer_entry() {
        let f = write_config("maxPlayers = 500\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 500);
    }

    #[test]
    fn test_load_known_boolean_true() {
        let f = write_config("freePremium = true\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert!(cm.get_boolean(BooleanKey::FreePremium));
    }

    #[test]
    fn test_load_known_boolean_false() {
        let f = write_config("freePremium = false\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert!(!cm.get_boolean(BooleanKey::FreePremium));
    }

    #[test]
    fn test_load_multiple_entries() {
        let f = write_config(
            r#"serverName = "Forgotten"
ip = "127.0.0.1"
maxPlayers = 1000
freePremium = true
"#,
        );
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "Forgotten");
        assert_eq!(cm.get_string(StringKey::Ip), "127.0.0.1");
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 1000);
        assert!(cm.get_boolean(BooleanKey::FreePremium));
    }

    // -----------------------------------------------------------------------
    // Test 2: get_string on missing key returns ""
    // -----------------------------------------------------------------------
    #[test]
    fn test_get_string_missing_returns_empty() {
        let cm = ConfigManager::new();
        assert_eq!(cm.get_string(StringKey::ServerName), "");
    }

    // -----------------------------------------------------------------------
    // Test 3: get_integer on missing key returns 0
    // -----------------------------------------------------------------------
    #[test]
    fn test_get_integer_missing_returns_zero() {
        let cm = ConfigManager::new();
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 0);
    }

    // -----------------------------------------------------------------------
    // Test 4: get_boolean on missing key returns false
    // -----------------------------------------------------------------------
    #[test]
    fn test_get_boolean_missing_returns_false() {
        let cm = ConfigManager::new();
        assert!(!cm.get_boolean(BooleanKey::FreePremium));
    }

    // -----------------------------------------------------------------------
    // Test 5: set_string overrides a value
    // -----------------------------------------------------------------------
    #[test]
    fn test_set_string_overrides() {
        let mut cm = ConfigManager::new();
        cm.set_string(StringKey::ServerName, "Initial");
        assert_eq!(cm.get_string(StringKey::ServerName), "Initial");
        cm.set_string(StringKey::ServerName, "Override");
        assert_eq!(cm.get_string(StringKey::ServerName), "Override");
    }

    // -----------------------------------------------------------------------
    // Test 6: set_integer overrides a value
    // -----------------------------------------------------------------------
    #[test]
    fn test_set_integer_overrides() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::MaxPlayers, 100);
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 100);
        cm.set_integer(IntegerKey::MaxPlayers, 999);
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 999);
    }

    // -----------------------------------------------------------------------
    // Test 7: set_boolean overrides a value
    // -----------------------------------------------------------------------
    #[test]
    fn test_set_boolean_overrides() {
        let mut cm = ConfigManager::new();
        cm.set_boolean(BooleanKey::FreePremium, true);
        assert!(cm.get_boolean(BooleanKey::FreePremium));
        cm.set_boolean(BooleanKey::FreePremium, false);
        assert!(!cm.get_boolean(BooleanKey::FreePremium));
    }

    // -----------------------------------------------------------------------
    // Test 8: load on missing file returns Err
    // -----------------------------------------------------------------------
    #[test]
    fn test_load_missing_file_returns_err() {
        let mut cm = ConfigManager::new();
        let result = cm.load(Path::new("/tmp/no_such_config_file_xyz_99.lua"));
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Test 9: comments are stripped
    // -----------------------------------------------------------------------
    #[test]
    fn test_comments_are_stripped() {
        let f = write_config(
            r#"-- This is a comment
serverName = "Test" -- inline comment
"#,
        );
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "Test");
    }

    // -----------------------------------------------------------------------
    // Test 10: unknown keys are silently ignored
    // -----------------------------------------------------------------------
    #[test]
    fn test_unknown_keys_ignored() {
        let f = write_config("unknownSetting = 42\n");
        let mut cm = ConfigManager::new();
        let result = cm.load(f.path());
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // Test 11: negative integer value
    // -----------------------------------------------------------------------
    #[test]
    fn test_negative_integer() {
        let f = write_config("deathLosePercent = -1\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::DeathLosePercent), -1);
    }

    // -----------------------------------------------------------------------
    // Phase 1 — admin.port, admin.password, status.port
    // -----------------------------------------------------------------------

    #[test]
    fn config_reads_admin_port() {
        let f = write_config("adminPort = 7172\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::AdminPort), 7172);
    }

    #[test]
    fn config_reads_admin_password() {
        let f = write_config(r#"adminPassword = "secret""#);
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::AdminPassword), "secret");
    }

    #[test]
    fn config_reads_status_port() {
        let f = write_config("statusPort = 7171\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::StatusPort), 7171);
    }

    #[test]
    fn config_reads_all_admin_and_status_keys_together() {
        let f = write_config("adminPort = 7172\nadminPassword = \"admin123\"\nstatusPort = 7171\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::AdminPort), 7172);
        assert_eq!(cm.get_string(StringKey::AdminPassword), "admin123");
        assert_eq!(cm.get_integer(IntegerKey::StatusPort), 7171);
    }

    // -----------------------------------------------------------------------
    // Test: single-quoted string value is parsed correctly
    // -----------------------------------------------------------------------
    #[test]
    fn test_single_quoted_string_value() {
        let f = write_config("serverName = 'My Server'\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "My Server");
    }

    // -----------------------------------------------------------------------
    // Test: nil value produces empty string / 0 / false
    // -----------------------------------------------------------------------
    #[test]
    fn test_nil_value_string_key() {
        let f = write_config("serverName = nil\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "");
    }

    #[test]
    fn test_nil_value_integer_key() {
        // nil → empty string → parse::<i64>() fails → 0
        let f = write_config("maxPlayers = nil\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 0);
    }

    // -----------------------------------------------------------------------
    // Test: configFile key is parsed from Lua
    // -----------------------------------------------------------------------
    #[test]
    fn test_config_file_key_loads_from_lua() {
        let f = write_config("configFile = \"config.lua\"\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ConfigFile), "config.lua");
    }

    // -----------------------------------------------------------------------
    // Test: every documented C++ boolean_config_t variant round-trips
    // -----------------------------------------------------------------------
    #[test]
    fn test_every_boolean_key_round_trips_via_set_get() {
        let all_keys = [
            BooleanKey::AllowChangeOutfit,
            BooleanKey::OnePlayerOnAccount,
            BooleanKey::AimbotHotkeyEnabled,
            BooleanKey::RemoveRuneCharges,
            BooleanKey::RemoveWeaponAmmo,
            BooleanKey::RemoveWeaponCharges,
            BooleanKey::RemovePotionCharges,
            BooleanKey::ExperienceFromPlayers,
            BooleanKey::FreePremium,
            BooleanKey::ReplaceKickOnLogin,
            BooleanKey::AllowClones,
            BooleanKey::AllowWalkthrough,
            BooleanKey::BindOnlyGlobalAddress,
            BooleanKey::OptimizeDatabase,
            BooleanKey::MarketPremium,
            BooleanKey::EmoteSpells,
            BooleanKey::StaminaSystem,
            BooleanKey::WarnUnsafeScripts,
            BooleanKey::ConvertUnsafeScripts,
            BooleanKey::ClassicEquipmentSlots,
            BooleanKey::ClassicAttackSpeed,
            BooleanKey::ScriptsConsoleLogs,
            BooleanKey::ServerSaveNotifyMessage,
            BooleanKey::ServerSaveCleanMap,
            BooleanKey::ServerSaveClose,
            BooleanKey::ServerSaveShutdown,
            BooleanKey::OnlineOfflineCharlist,
            BooleanKey::YellAllowPremium,
            BooleanKey::PremiumToSendPrivate,
            BooleanKey::ForceMonsterTypeLoad,
            BooleanKey::HouseOwnedByAccount,
            BooleanKey::CleanProtectionZones,
            BooleanKey::HouseDoorShowPrice,
            BooleanKey::OnlyInvitedCanMoveHouseItems,
            BooleanKey::RemoveOnDespawn,
            BooleanKey::TwoFactorAuth,
            BooleanKey::ManashieldBreakable,
            BooleanKey::CheckDuplicateStorageKeys,
            BooleanKey::MonsterOverspawn,
        ];
        let mut cm = ConfigManager::new();
        for key in all_keys {
            cm.set_boolean(key, true);
            assert!(
                cm.get_boolean(key),
                "set_boolean true → get_boolean should be true for {key:?}"
            );
            cm.set_boolean(key, false);
            assert!(
                !cm.get_boolean(key),
                "set_boolean false → get_boolean should be false for {key:?}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test: every documented C++ string_config_t variant round-trips
    // -----------------------------------------------------------------------
    #[test]
    fn test_every_string_key_round_trips_via_set_get() {
        let all_keys = [
            StringKey::MapName,
            StringKey::HouseRentPeriod,
            StringKey::ServerName,
            StringKey::OwnerName,
            StringKey::OwnerEmail,
            StringKey::Url,
            StringKey::Location,
            StringKey::Ip,
            StringKey::WorldType,
            StringKey::MysqlHost,
            StringKey::MysqlUser,
            StringKey::MysqlPass,
            StringKey::MysqlDb,
            StringKey::MysqlSock,
            StringKey::DefaultPriority,
            StringKey::MapAuthor,
            StringKey::ConfigFile,
            StringKey::HttpBindAddress,
        ];
        let mut cm = ConfigManager::new();
        for key in all_keys {
            cm.set_string(key, "test_value");
            assert_eq!(
                cm.get_string(key),
                "test_value",
                "round-trip failed for {key:?}"
            );
        }
    }

    #[test]
    fn test_http_bind_address_lua_name_maps_and_round_trips() {
        // Verifies the Lua key "httpLoginBindAddress" is recognized and that
        // StringKey::HttpBindAddress round-trips through set/get.
        let cfg = "httpLoginBindAddress = \"10.0.0.1\"\n";
        let f = write_config(cfg);
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(
            cm.get_string(StringKey::HttpBindAddress),
            "10.0.0.1",
            "httpLoginBindAddress Lua name must map to StringKey::HttpBindAddress"
        );

        // Also verify the set/get round-trip independent of file loading.
        let mut cm2 = ConfigManager::new();
        cm2.set_string(StringKey::HttpBindAddress, "192.168.1.5");
        assert_eq!(cm2.get_string(StringKey::HttpBindAddress), "192.168.1.5");
    }

    // -----------------------------------------------------------------------
    // Test: every documented C++ integer_config_t variant round-trips
    // -----------------------------------------------------------------------
    #[test]
    fn test_every_integer_key_round_trips_via_set_get() {
        let all_keys = [
            IntegerKey::SqlPort,
            IntegerKey::MaxPlayers,
            IntegerKey::PzLocked,
            IntegerKey::DefaultDespawnRange,
            IntegerKey::DefaultDespawnRadius,
            IntegerKey::DefaultWalkToSpawnRadius,
            IntegerKey::RateExperience,
            IntegerKey::RateSkill,
            IntegerKey::RateLoot,
            IntegerKey::RateMagic,
            IntegerKey::RateSpawn,
            IntegerKey::HousePrice,
            IntegerKey::KillsToRed,
            IntegerKey::KillsToBlack,
            IntegerKey::MaxMessageBuffer,
            IntegerKey::ActionsDelayInterval,
            IntegerKey::ExActionsDelayInterval,
            IntegerKey::KickAfterMinutes,
            IntegerKey::ProtectionLevel,
            IntegerKey::DeathLosePercent,
            IntegerKey::StatusQueryTimeout,
            IntegerKey::StatusCountMaxPlayersPerIp,
            IntegerKey::FragTime,
            IntegerKey::WhiteSkullTime,
            IntegerKey::GamePort,
            IntegerKey::StatusPort,
            IntegerKey::HttpPort,
            IntegerKey::HttpWorkers,
            IntegerKey::StairhopDelay,
            IntegerKey::MarketOfferDuration,
            IntegerKey::CheckExpiredMarketOffersEachMinutes,
            IntegerKey::MaxMarketOffersAtATimePerPlayer,
            IntegerKey::ExpFromPlayersLevelRange,
            IntegerKey::MaxPacketsPerSecond,
            IntegerKey::ServerSaveNotifyDuration,
            IntegerKey::YellMinimumLevel,
            IntegerKey::MinimumLevelToSendPrivate,
            IntegerKey::VipFreeLimit,
            IntegerKey::VipPremiumLimit,
            IntegerKey::DepotFreeLimit,
            IntegerKey::DepotPremiumLimit,
            IntegerKey::QuestTrackerFreeLimit,
            IntegerKey::QuestTrackerPremiumLimit,
            IntegerKey::StaminaRegenMinute,
            IntegerKey::StaminaRegenPremium,
            IntegerKey::PathfindingInterval,
            IntegerKey::PathfindingDelay,
        ];
        let mut cm = ConfigManager::new();
        for key in all_keys {
            cm.set_integer(key, 42);
            assert_eq!(cm.get_integer(key), 42, "round-trip failed for {key:?}");
        }
    }

    // -----------------------------------------------------------------------
    // Test: Lua boolean key mappings load from file (spot-check several)
    // -----------------------------------------------------------------------
    #[test]
    fn test_boolean_lua_keys_load_from_file() {
        let f = write_config(
            "allowChangeOutfit = true\nfreePremium = false\nstaminaSystem = true\nmonsterOverspawn = false\n",
        );
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert!(cm.get_boolean(BooleanKey::AllowChangeOutfit));
        assert!(!cm.get_boolean(BooleanKey::FreePremium));
        assert!(cm.get_boolean(BooleanKey::StaminaSystem));
        assert!(!cm.get_boolean(BooleanKey::MonsterOverspawn));
    }

    // -----------------------------------------------------------------------
    // Test: Lua integer key mappings load from file (spot-check several)
    // -----------------------------------------------------------------------
    #[test]
    fn test_integer_lua_keys_load_from_file() {
        let f = write_config(
            "gameProtocolPort = 7172\nstatusProtocolPort = 7171\nrateExp = 5\nrateLoot = 2\npathfindingDelay = 300\n",
        );
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::GamePort), 7172);
        assert_eq!(cm.get_integer(IntegerKey::StatusPort), 7171);
        assert_eq!(cm.get_integer(IntegerKey::RateExperience), 5);
        assert_eq!(cm.get_integer(IntegerKey::RateLoot), 2);
        assert_eq!(cm.get_integer(IntegerKey::PathfindingDelay), 300);
    }

    // -----------------------------------------------------------------------
    // Test: Lua string key mappings load from file (spot-check several)
    // -----------------------------------------------------------------------
    #[test]
    fn test_string_lua_keys_load_from_file() {
        let f = write_config(
            "mapName = \"forgotten\"\nhouseRentPeriod = \"never\"\nworldType = \"pvp\"\ndefaultPriority = \"high\"\n",
        );
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::MapName), "forgotten");
        assert_eq!(cm.get_string(StringKey::HouseRentPeriod), "never");
        assert_eq!(cm.get_string(StringKey::WorldType), "pvp");
        assert_eq!(cm.get_string(StringKey::DefaultPriority), "high");
    }

    // -----------------------------------------------------------------------
    // ConfigKey unified-key conversions — round-trip every typed key
    // through the [`From`] impls so the discriminant survives.
    // -----------------------------------------------------------------------
    #[test]
    fn test_config_key_from_boolean_key_preserves_variant() {
        let key: ConfigKey = BooleanKey::FreePremium.into();
        assert_eq!(key, ConfigKey::Boolean(BooleanKey::FreePremium));
    }

    #[test]
    fn test_config_key_from_string_key_preserves_variant() {
        let key: ConfigKey = StringKey::ServerName.into();
        assert_eq!(key, ConfigKey::String(StringKey::ServerName));
    }

    #[test]
    fn test_config_key_from_integer_key_preserves_variant() {
        let key: ConfigKey = IntegerKey::MaxPlayers.into();
        assert_eq!(key, ConfigKey::Integer(IntegerKey::MaxPlayers));
    }

    // -----------------------------------------------------------------------
    // Malformed line: a non-empty line with no '=' is silently skipped and
    // does NOT clobber adjacent valid entries (matches C++ Lua loader, which
    // ignores syntax-invalid free-standing identifiers).
    // -----------------------------------------------------------------------
    #[test]
    fn test_malformed_line_without_equals_is_skipped() {
        let f = write_config("serverName = \"Before\"\nthisLineHasNoEqualsSign\nmaxPlayers = 7\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        // both adjacent entries must still load
        assert_eq!(cm.get_string(StringKey::ServerName), "Before");
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 7);
    }

    // -----------------------------------------------------------------------
    // parse_value bare-identifier fallback: unquoted, non-bool, non-nil,
    // non-numeric RHS is treated as a bare string (matches C++ Lua loader,
    // where an undefined identifier resolves to nil — we degrade to "".
    // Here we exercise the fallback Str() arm via a known string key so the
    // value is actually stored).
    // -----------------------------------------------------------------------
    #[test]
    fn test_parse_value_bare_identifier_falls_back_to_string() {
        let f = write_config("serverName = bareword\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "bareword");
    }

    // -----------------------------------------------------------------------
    // apply() cross-type coercions — every (declared-key-type, value-type)
    // mismatch must coerce the value rather than silently dropping it.
    // -----------------------------------------------------------------------

    /// String key + integer RHS  → stored as decimal string ("42").
    #[test]
    fn test_apply_string_key_with_integer_value_stringifies() {
        let f = write_config("serverName = 42\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "42");
    }

    /// String key + bool RHS → stored as "true"/"false".
    #[test]
    fn test_apply_string_key_with_bool_value_stringifies() {
        let f = write_config("serverName = true\nownerName = false\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "true");
        assert_eq!(cm.get_string(StringKey::OwnerName), "false");
    }

    /// Integer key + quoted-string RHS that parses → numeric value.
    #[test]
    fn test_apply_integer_key_with_string_value_parses() {
        let f = write_config("maxPlayers = \"123\"\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 123);
    }

    /// Integer key + non-numeric quoted-string RHS → 0 (parse failure).
    #[test]
    fn test_apply_integer_key_with_nonnumeric_string_value_falls_back_to_zero() {
        let f = write_config("maxPlayers = \"abc\"\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 0);
    }

    /// Integer key + bool RHS → 1 (true) or 0 (false).
    #[test]
    fn test_apply_integer_key_with_bool_value_casts() {
        let f = write_config("maxPlayers = true\npzLocked = false\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::MaxPlayers), 1);
        assert_eq!(cm.get_integer(IntegerKey::PzLocked), 0);
    }

    /// Boolean key + integer RHS → false for 0, true for non-zero.
    #[test]
    fn test_apply_boolean_key_with_integer_value_casts() {
        let f = write_config("freePremium = 1\nallowClones = 0\nemoteSpells = 99\n");
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert!(cm.get_boolean(BooleanKey::FreePremium));
        assert!(!cm.get_boolean(BooleanKey::AllowClones));
        assert!(cm.get_boolean(BooleanKey::EmoteSpells));
    }

    /// Boolean key + string RHS — accepts "true", "yes", "1"; rejects others.
    #[test]
    fn test_apply_boolean_key_with_string_value_truthy_set() {
        let f = write_config(
            "freePremium = \"true\"\nallowClones = \"yes\"\nemoteSpells = \"1\"\nstaminaSystem = \"no\"\nallowWalkthrough = \"off\"\n",
        );
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert!(cm.get_boolean(BooleanKey::FreePremium));
        assert!(cm.get_boolean(BooleanKey::AllowClones));
        assert!(cm.get_boolean(BooleanKey::EmoteSpells));
        assert!(!cm.get_boolean(BooleanKey::StaminaSystem));
        assert!(!cm.get_boolean(BooleanKey::AllowWalkthrough));
    }

    // -----------------------------------------------------------------------
    // Remaining integer Lua-name mappings that lack a direct file-load test:
    // every mapped name must round-trip through load() with the canonical
    // value, so that any future rename in `integer_key` is caught.
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_remaining_integer_lua_names_load_from_file() {
        let cfg = "\
mysqlPort = 3307
pzLocked = 60001
deSpawnRange = 4
deSpawnRadius = 51
walkToSpawnRadius = 16
rateSkill = 7
rateMagic = 11
rateSpawn = 13
housePriceEachSQM = 1001
killsToRedSkull = 17
killsToBlackSkull = 19
maxMessageBuffer = 23
timeBetweenActions = 29
timeBetweenExActions = 31
kickIdlePlayerAfterMinutes = 37
protectionLevel = 41
deathLosePercent = 43
statusTimeout = 47
statusCountMaxPlayersPerIp = 53
timeToDecreaseFrags = 59
whiteSkullTime = 61
httpPort = 67
httpWorkers = 71
stairJumpExhaustion = 73
marketOfferDuration = 79
checkExpiredMarketOffersEachMinutes = 83
maxMarketOffersAtATimePerPlayer = 89
expFromPlayersLevelRange = 97
maxPacketsPerSecond = 101
serverSaveNotifyDuration = 103
yellMinimumLevel = 107
minimumLevelToSendPrivate = 109
vipFreeLimit = 113
vipPremiumLimit = 127
depotFreeLimit = 131
depotPremiumLimit = 137
questTrackerFreeLimit = 139
questTrackerPremiumLimit = 149
timeToRegenMinuteStamina = 151
timeToRegenMinutePremiumStamina = 157
pathfindingInterval = 163
";
        let f = write_config(cfg);
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_integer(IntegerKey::SqlPort), 3307);
        assert_eq!(cm.get_integer(IntegerKey::PzLocked), 60001);
        assert_eq!(cm.get_integer(IntegerKey::DefaultDespawnRange), 4);
        assert_eq!(cm.get_integer(IntegerKey::DefaultDespawnRadius), 51);
        assert_eq!(cm.get_integer(IntegerKey::DefaultWalkToSpawnRadius), 16);
        assert_eq!(cm.get_integer(IntegerKey::RateSkill), 7);
        assert_eq!(cm.get_integer(IntegerKey::RateMagic), 11);
        assert_eq!(cm.get_integer(IntegerKey::RateSpawn), 13);
        assert_eq!(cm.get_integer(IntegerKey::HousePrice), 1001);
        assert_eq!(cm.get_integer(IntegerKey::KillsToRed), 17);
        assert_eq!(cm.get_integer(IntegerKey::KillsToBlack), 19);
        assert_eq!(cm.get_integer(IntegerKey::MaxMessageBuffer), 23);
        assert_eq!(cm.get_integer(IntegerKey::ActionsDelayInterval), 29);
        assert_eq!(cm.get_integer(IntegerKey::ExActionsDelayInterval), 31);
        assert_eq!(cm.get_integer(IntegerKey::KickAfterMinutes), 37);
        assert_eq!(cm.get_integer(IntegerKey::ProtectionLevel), 41);
        assert_eq!(cm.get_integer(IntegerKey::DeathLosePercent), 43);
        assert_eq!(cm.get_integer(IntegerKey::StatusQueryTimeout), 47);
        assert_eq!(cm.get_integer(IntegerKey::StatusCountMaxPlayersPerIp), 53);
        assert_eq!(cm.get_integer(IntegerKey::FragTime), 59);
        assert_eq!(cm.get_integer(IntegerKey::WhiteSkullTime), 61);
        assert_eq!(cm.get_integer(IntegerKey::HttpPort), 67);
        assert_eq!(cm.get_integer(IntegerKey::HttpWorkers), 71);
        assert_eq!(cm.get_integer(IntegerKey::StairhopDelay), 73);
        assert_eq!(cm.get_integer(IntegerKey::MarketOfferDuration), 79);
        assert_eq!(
            cm.get_integer(IntegerKey::CheckExpiredMarketOffersEachMinutes),
            83
        );
        assert_eq!(
            cm.get_integer(IntegerKey::MaxMarketOffersAtATimePerPlayer),
            89
        );
        assert_eq!(cm.get_integer(IntegerKey::ExpFromPlayersLevelRange), 97);
        assert_eq!(cm.get_integer(IntegerKey::MaxPacketsPerSecond), 101);
        assert_eq!(cm.get_integer(IntegerKey::ServerSaveNotifyDuration), 103);
        assert_eq!(cm.get_integer(IntegerKey::YellMinimumLevel), 107);
        assert_eq!(cm.get_integer(IntegerKey::MinimumLevelToSendPrivate), 109);
        assert_eq!(cm.get_integer(IntegerKey::VipFreeLimit), 113);
        assert_eq!(cm.get_integer(IntegerKey::VipPremiumLimit), 127);
        assert_eq!(cm.get_integer(IntegerKey::DepotFreeLimit), 131);
        assert_eq!(cm.get_integer(IntegerKey::DepotPremiumLimit), 137);
        assert_eq!(cm.get_integer(IntegerKey::QuestTrackerFreeLimit), 139);
        assert_eq!(cm.get_integer(IntegerKey::QuestTrackerPremiumLimit), 149);
        assert_eq!(cm.get_integer(IntegerKey::StaminaRegenMinute), 151);
        assert_eq!(cm.get_integer(IntegerKey::StaminaRegenPremium), 157);
        assert_eq!(cm.get_integer(IntegerKey::PathfindingInterval), 163);
    }

    // -----------------------------------------------------------------------
    // Remaining boolean Lua-name mappings that lack a direct file-load test.
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_remaining_boolean_lua_names_load_from_file() {
        let cfg = "\
onePlayerOnlinePerAccount = true
hotkeyAimbotEnabled = true
removeChargesFromRunes = true
removeWeaponAmmunition = true
removeWeaponCharges = true
removeChargesFromPotions = true
experienceByKillingPlayers = true
replaceKickOnLogin = true
allowClones = true
allowWalkthrough = true
bindOnlyGlobalAddress = true
startupDatabaseOptimization = true
premiumToCreateMarketOffer = true
warnUnsafeScripts = true
convertUnsafeScripts = true
classicEquipmentSlots = true
classicAttackSpeed = true
showScriptsLogInConsole = true
serverSaveNotifyMessage = true
serverSaveCleanMap = true
serverSaveClose = true
serverSaveShutdown = true
showOnlineStatusInCharlist = true
yellAlwaysAllowPremium = true
premiumToSendPrivate = true
forceMonsterTypesOnLoad = true
houseOwnedByAccount = true
cleanProtectionZones = true
houseDoorShowPrice = true
onlyInvitedCanMoveHouseItems = true
removeOnDespawn = true
enableTwoFactorAuth = true
useBreakableManaShield = true
checkDuplicateStorageKeys = true
emoteSpells = true
";
        let f = write_config(cfg);
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert!(cm.get_boolean(BooleanKey::OnePlayerOnAccount));
        assert!(cm.get_boolean(BooleanKey::AimbotHotkeyEnabled));
        assert!(cm.get_boolean(BooleanKey::RemoveRuneCharges));
        assert!(cm.get_boolean(BooleanKey::RemoveWeaponAmmo));
        assert!(cm.get_boolean(BooleanKey::RemoveWeaponCharges));
        assert!(cm.get_boolean(BooleanKey::RemovePotionCharges));
        assert!(cm.get_boolean(BooleanKey::ExperienceFromPlayers));
        assert!(cm.get_boolean(BooleanKey::ReplaceKickOnLogin));
        assert!(cm.get_boolean(BooleanKey::AllowClones));
        assert!(cm.get_boolean(BooleanKey::AllowWalkthrough));
        assert!(cm.get_boolean(BooleanKey::BindOnlyGlobalAddress));
        assert!(cm.get_boolean(BooleanKey::OptimizeDatabase));
        assert!(cm.get_boolean(BooleanKey::MarketPremium));
        assert!(cm.get_boolean(BooleanKey::WarnUnsafeScripts));
        assert!(cm.get_boolean(BooleanKey::ConvertUnsafeScripts));
        assert!(cm.get_boolean(BooleanKey::ClassicEquipmentSlots));
        assert!(cm.get_boolean(BooleanKey::ClassicAttackSpeed));
        assert!(cm.get_boolean(BooleanKey::ScriptsConsoleLogs));
        assert!(cm.get_boolean(BooleanKey::ServerSaveNotifyMessage));
        assert!(cm.get_boolean(BooleanKey::ServerSaveCleanMap));
        assert!(cm.get_boolean(BooleanKey::ServerSaveClose));
        assert!(cm.get_boolean(BooleanKey::ServerSaveShutdown));
        assert!(cm.get_boolean(BooleanKey::OnlineOfflineCharlist));
        assert!(cm.get_boolean(BooleanKey::YellAllowPremium));
        assert!(cm.get_boolean(BooleanKey::PremiumToSendPrivate));
        assert!(cm.get_boolean(BooleanKey::ForceMonsterTypeLoad));
        assert!(cm.get_boolean(BooleanKey::HouseOwnedByAccount));
        assert!(cm.get_boolean(BooleanKey::CleanProtectionZones));
        assert!(cm.get_boolean(BooleanKey::HouseDoorShowPrice));
        assert!(cm.get_boolean(BooleanKey::OnlyInvitedCanMoveHouseItems));
        assert!(cm.get_boolean(BooleanKey::RemoveOnDespawn));
        assert!(cm.get_boolean(BooleanKey::TwoFactorAuth));
        assert!(cm.get_boolean(BooleanKey::ManashieldBreakable));
        assert!(cm.get_boolean(BooleanKey::CheckDuplicateStorageKeys));
        assert!(cm.get_boolean(BooleanKey::EmoteSpells));
    }

    // -----------------------------------------------------------------------
    // Remaining string Lua-name mappings that lack a direct file-load test.
    // -----------------------------------------------------------------------
    #[test]
    fn test_all_remaining_string_lua_names_load_from_file() {
        let cfg = "\
serverName = \"FS\"
ownerName = \"Pablo\"
ownerEmail = \"a@b.c\"
url = \"http://x\"
location = \"BR\"
ip = \"10.0.0.1\"
mapAuthor = \"Bob\"
mysqlHost = \"h\"
mysqlUser = \"u\"
mysqlPass = \"p\"
mysqlDatabase = \"d\"
mysqlSock = \"s\"
";
        let f = write_config(cfg);
        let mut cm = ConfigManager::new();
        cm.load(f.path()).expect("load ok");
        assert_eq!(cm.get_string(StringKey::ServerName), "FS");
        assert_eq!(cm.get_string(StringKey::OwnerName), "Pablo");
        assert_eq!(cm.get_string(StringKey::OwnerEmail), "a@b.c");
        assert_eq!(cm.get_string(StringKey::Url), "http://x");
        assert_eq!(cm.get_string(StringKey::Location), "BR");
        assert_eq!(cm.get_string(StringKey::Ip), "10.0.0.1");
        assert_eq!(cm.get_string(StringKey::MapAuthor), "Bob");
        assert_eq!(cm.get_string(StringKey::MysqlHost), "h");
        assert_eq!(cm.get_string(StringKey::MysqlUser), "u");
        assert_eq!(cm.get_string(StringKey::MysqlPass), "p");
        assert_eq!(cm.get_string(StringKey::MysqlDb), "d");
        assert_eq!(cm.get_string(StringKey::MysqlSock), "s");
    }

    // -----------------------------------------------------------------------
    // Experience stages — mirrors C++ `ConfigManager::getExperienceStage`.
    //
    // No stages configured  → fall back to `RateExperience` cast to `f32`.
    // Stage covers the level → return its multiplier.
    // No stage covers       → fall back to `RateExperience` cast to `f32`.
    // -----------------------------------------------------------------------
    #[test]
    fn test_get_experience_stage_no_stages_falls_back_to_rate_experience() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::RateExperience, 7);
        assert_eq!(cm.get_experience_stage(50), 7.0);
    }

    #[test]
    fn test_get_experience_stage_matching_stage_returns_multiplier() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::RateExperience, 1);
        cm.add_experience_stage(1, 50, 5.0);
        cm.add_experience_stage(51, 100, 3.5);
        cm.add_experience_stage(101, u32::MAX, 1.5);
        assert_eq!(cm.get_experience_stage(1), 5.0);
        assert_eq!(cm.get_experience_stage(25), 5.0);
        assert_eq!(cm.get_experience_stage(50), 5.0);
        assert_eq!(cm.get_experience_stage(51), 3.5);
        assert_eq!(cm.get_experience_stage(100), 3.5);
        assert_eq!(cm.get_experience_stage(101), 1.5);
        assert_eq!(cm.get_experience_stage(9_999_999), 1.5);
    }

    #[test]
    fn test_get_experience_stage_gap_falls_back_to_rate_experience() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::RateExperience, 4);
        // Stages cover [10, 20] only — querying 5 must fall back.
        cm.add_experience_stage(10, 20, 9.0);
        assert_eq!(cm.get_experience_stage(5), 4.0);
        assert_eq!(cm.get_experience_stage(15), 9.0);
        assert_eq!(cm.get_experience_stage(25), 4.0);
    }
}
