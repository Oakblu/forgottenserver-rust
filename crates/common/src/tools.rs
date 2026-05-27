//! Migrated from forgottenserver/src/tools.h and tools.cpp
//!
//! Provides string utilities, crypto helpers (SHA-1, HMAC-SHA1, TOTP),
//! random-number helpers, fluid-mapping, adler-32 checksum, and several
//! game-specific look-up functions.

#![allow(dead_code)]

use hmac::{Hmac, Mac};
use rand::Rng;
use sha1::Sha1;

use crate::constants::{
    Ammo, MagicEffectClass, ShootType, Skull, WeaponAction, CLIENT_TO_SERVER_FLUID_MAP,
};
use crate::enums::{CombatTypeFlags, ItemAttrFlags, ReturnValue, SpellGroup};
use crate::position::Direction;

// ---------------------------------------------------------------------------
// Type aliases (mirrors tools.h)
// ---------------------------------------------------------------------------

pub type StringVector = Vec<String>;
pub type IntegerVector = Vec<i32>;

// ---------------------------------------------------------------------------
// String utilities
// ---------------------------------------------------------------------------

/// Capitalise the first non-space character of `s`.
///
/// Mirrors `ucfirst(std::string str)` in tools.cpp.
pub fn ucfirst(mut s: String) -> String {
    for ch in s.chars() {
        if ch != ' ' {
            break;
        }
    }
    // find first non-space byte index and uppercase it
    let mut idx: Option<usize> = None;
    for (i, c) in s.char_indices() {
        if c != ' ' {
            idx = Some(i);
            break;
        }
    }
    if let Some(i) = idx {
        let c = s[i..].chars().next().unwrap();
        let upper: String = c.to_uppercase().collect();
        s.replace_range(i..i + c.len_utf8(), &upper);
    }
    s
}

/// Capitalise the first letter of each word (space-separated).
///
/// Mirrors `ucwords(std::string str)` in tools.cpp.
pub fn ucwords(mut s: String) -> String {
    if s.is_empty() {
        return s;
    }
    // Safety: we only operate on ASCII/single-byte boundaries produced by
    // to_uppercase() for ASCII letters, which is safe here.
    // Collect indices of chars that need uppercasing.
    let bytes = unsafe { s.as_bytes_mut() };
    if !bytes.is_empty() {
        bytes[0] = bytes[0].to_ascii_uppercase();
    }
    for i in 1..bytes.len() {
        if bytes[i - 1] == b' ' {
            bytes[i] = bytes[i].to_ascii_uppercase();
        }
    }
    s
}

/// Returns `true` if the string represents a truthy value.
///
/// Mirrors `booleanString(std::string_view str)` in tools.cpp:
/// - empty → false
/// - first char (lowercased) is 'f', 'n', or '0' → false
/// - otherwise → true
pub fn boolean_string(s: &str) -> bool {
    match s.chars().next() {
        None => false,
        Some(ch) => {
            let lower = ch.to_lowercase().next().unwrap_or(ch);
            lower != 'f' && lower != 'n' && lower != '0'
        }
    }
}

/// Case-insensitive equality check.
///
/// Mirrors `caseInsensitiveEqual` in tools.cpp.
pub fn case_insensitive_equal(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.chars()
            .zip(b.chars())
            .all(|(ca, cb)| ca.to_lowercase().eq(cb.to_lowercase()))
}

/// Case-insensitive prefix check.
///
/// Mirrors `caseInsensitiveStartsWith` in tools.cpp.
pub fn case_insensitive_starts_with(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len()
        && s.chars()
            .zip(prefix.chars())
            .all(|(cs, cp)| cs.to_lowercase().eq(cp.to_lowercase()))
}

/// Split `in_string` by `separator` into at most `limit` parts.
///
/// When `limit == -1` (or `limit == 0`) there is no limit.
/// Mirrors `explodeString` in tools.cpp.
pub fn explode_string<'a>(in_string: &'a str, separator: &str, limit: i32) -> Vec<&'a str> {
    let mut result: Vec<&str> = Vec::new();
    let mut start = 0usize;
    let mut remaining = limit;

    while remaining != 0 {
        remaining -= 1;
        if let Some(pos) = in_string[start..].find(separator) {
            result.push(&in_string[start..start + pos]);
            start += pos + separator.len();
        } else {
            break;
        }
        if remaining == 0 {
            break;
        }
    }
    result.push(&in_string[start..]);
    result
}

/// Parse a slice of string slices to i32 (like C `atoi`).
///
/// Mirrors `vectorAtoi` in tools.cpp.
pub fn vector_atoi(strings: &[&str]) -> IntegerVector {
    strings
        .iter()
        .map(|s| s.trim().parse::<i32>().unwrap_or(0))
        .collect()
}

/// Extract the first line (up to the first `\n`).
///
/// Mirrors `getFirstLine` in tools.cpp.
pub fn get_first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").to_string()
}

// ---------------------------------------------------------------------------
// Bit-test helper
// ---------------------------------------------------------------------------

/// Returns `true` if `flag` is set in `flags`.
///
/// Mirrors the `constexpr hasBitSet` in tools.h.
#[inline]
pub const fn has_bit_set(flag: u32, flags: u32) -> bool {
    (flags & flag) != 0
}

// ---------------------------------------------------------------------------
// Random helpers
// ---------------------------------------------------------------------------

/// Uniform random integer in `[min_number, max_number]` (inclusive).
///
/// If `min > max` the arguments are swapped, mirroring the C++ behaviour.
pub fn uniform_random(min_number: i32, max_number: i32) -> i32 {
    if min_number == max_number {
        return min_number;
    }
    let (lo, hi) = if min_number < max_number {
        (min_number, max_number)
    } else {
        (max_number, min_number)
    };
    rand::thread_rng().gen_range(lo..=hi)
}

/// Normal (Gaussian) random integer in `[min_number, max_number]` (inclusive).
///
/// Uses a truncated normal distribution with mean=0.5 and stddev=0.25
/// (via Box-Muller transform), then scales to the [min, max] range.
/// Arguments are automatically sorted so that `min <= max`.
///
/// Mirrors `normal_random(int32_t minNumber, int32_t maxNumber)` in tools.cpp.
pub fn normal_random(min_number: i32, max_number: i32) -> i32 {
    if min_number == max_number {
        return min_number;
    }
    let (lo, hi) = if min_number < max_number {
        (min_number, max_number)
    } else {
        (max_number, min_number)
    };
    // Mirror the C++ implementation: sample from N(0.5, 0.25²) truncated to
    // [0.0, 1.0] using rejection sampling, then round to [lo, hi].
    // We use the Box-Muller transform to generate Gaussian samples.
    use std::f64::consts::PI;
    let mut rng = rand::thread_rng();
    let v = loop {
        let u1: f64 = rng.gen::<f64>();
        let u2: f64 = rng.gen::<f64>();
        // Box-Muller: produces N(0,1). When u1 == 0.0 (probability ~2^-52),
        // u1.ln() == -inf which propagates to a non-finite `sample`; the
        // `(0.0..=1.0).contains(&sample)` check below rejects it and the
        // loop continues — no explicit guard needed.
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        // Scale to N(0.5, 0.25²): multiply by stddev (0.25) and add mean (0.5)
        let sample = 0.5 + 0.25 * z;
        if (0.0..=1.0).contains(&sample) {
            break sample;
        }
    };
    lo + (v * (hi - lo) as f64).round() as i32
}

/// Boolean random with given probability of returning `true`.
///
/// Mirrors `boolean_random(double probability = 0.5)` in tools.cpp.
pub fn boolean_random(probability: f64) -> bool {
    rand::thread_rng().gen_bool(probability.clamp(0.0, 1.0))
}

/// Generate `length` cryptographically-uniform random bytes (0–255).
///
/// Mirrors `randomBytes(size_t length)` in tools.cpp.
pub fn random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen::<u8>()).collect()
}

/// Convert a positive integer (1–3999) to a Roman numeral string.
///
/// Returns `None` for values outside the valid range [1, 3999].
///
/// This function is used throughout the codebase to label depot boxes
/// (DepotBoxI … DepotBoxXX) and similar enumerated game entities.
///
/// Examples:
/// - `to_roman_numeral(1)` → `Some("I")`
/// - `to_roman_numeral(1000)` → `Some("M")`
/// - `to_roman_numeral(3999)` → `Some("MMMCMXCIX")`
/// - `to_roman_numeral(0)` → `None`
/// - `to_roman_numeral(4000)` → `None`
pub fn to_roman_numeral(mut n: u32) -> Option<String> {
    if n == 0 || n > 3999 {
        return None;
    }
    const TABLE: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();
    for &(value, symbol) in TABLE {
        while n >= value {
            result.push_str(symbol);
            n -= value;
        }
    }
    Some(result)
}

/// Return shuffled cardinal + diagonal directions (North, West, East, South).
///
/// Mirrors `getShuffleDirections()` in tools.cpp.
pub fn get_shuffle_directions() -> Vec<Direction> {
    let mut dirs = vec![
        Direction::North,
        Direction::West,
        Direction::East,
        Direction::South,
    ];
    use rand::seq::SliceRandom;
    dirs.shuffle(&mut rand::thread_rng());
    dirs
}

// ---------------------------------------------------------------------------
// Time
// ---------------------------------------------------------------------------

/// Returns the current Unix time in **milliseconds**.
///
/// Mirrors `OTSYS_TIME()` in tools.cpp.
pub fn otsys_time() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Formats a Unix timestamp (seconds) as `"DD Mon YYYY"`.
///
/// Mirrors `formatDateShort(time_t time)` → `std::format("{:%d %b %Y}", ...)`.
pub fn format_date_short(timestamp_secs: i64) -> String {
    let secs = if timestamp_secs < 0 {
        0u64
    } else {
        timestamp_secs as u64
    };
    // Simple epoch → calendar conversion (Gregorian proleptic)
    epoch_secs_to_date_short(secs)
}

/// Internal helper: convert Unix seconds to "DD Mon YYYY" string.
fn epoch_secs_to_date_short(secs: u64) -> String {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let days = secs / 86400;
    let mut remaining = days;

    let mut year = 1970u32;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days: [u32; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0usize;
    for &md in &month_days {
        if remaining < md as u64 {
            break;
        }
        remaining -= md as u64;
        month += 1;
    }

    let day = remaining + 1;
    format!("{:02} {} {}", day, months[month], year)
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

// ---------------------------------------------------------------------------
// Crypto — SHA-1
// ---------------------------------------------------------------------------

/// Compute SHA-1 of `input` and return the raw 20-byte digest.
///
/// Mirrors `transformToSHA1(std::string_view input)` in tools.cpp which
/// returns the raw bytes (not hex-encoded).
pub fn transform_to_sha1(input: &[u8]) -> [u8; 20] {
    use sha1::Digest;
    let mut hasher = Sha1::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// Hex-encoded SHA-1 (convenience wrapper, 40 hex chars).
pub fn transform_to_sha1_hex(input: &[u8]) -> String {
    let digest = transform_to_sha1(input);
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

// ---------------------------------------------------------------------------
// Crypto — HMAC-SHA1
// ---------------------------------------------------------------------------

/// Compute HMAC-SHA1(`key`, `message`) and return raw 20 bytes.
///
/// Mirrors `hmac("SHA1", key, message)` in tools.cpp.
pub fn hmac_sha1(key: &[u8], message: &[u8]) -> [u8; 20] {
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(message);
    mac.finalize().into_bytes().into()
}

// ---------------------------------------------------------------------------
// TOTP — generateToken
// ---------------------------------------------------------------------------

/// Generate a `length`-digit TOTP token.
///
/// Mirrors `generateToken(std::string_view key, uint64_t counter, size_t length)`
/// in tools.cpp.
///
/// The C++ code:
/// 1. Encodes `counter` as 8 big-endian bytes.
/// 2. Computes HMAC-SHA1(key, counter_bytes).
/// 3. Offset = last byte & 0x0f.
/// 4. Truncated = 4 bytes at offset, masked with 0x7fffffff.
/// 5. Token = last `length` decimal digits of `truncated`.
pub fn generate_token(key: &[u8], counter: u64, length: usize) -> String {
    // Step 1: big-endian 8-byte counter
    let counter_bytes = counter.to_be_bytes();

    // Step 2: HMAC-SHA1
    let mac = hmac_sha1(key, &counter_bytes);

    // Step 3: offset
    let offset = (mac[19] & 0x0f) as usize;

    // Step 4: dynamic truncation
    let p = ((mac[offset] as u32 & 0x7f) << 24)
        | ((mac[offset + 1] as u32) << 16)
        | ((mac[offset + 2] as u32) << 8)
        | (mac[offset + 3] as u32);

    // Step 5: last `length` digits
    let token = p.to_string();
    // Pad with leading zeros if needed, then take last `length` chars
    let padded = format!("{:0>width$}", token, width = length);
    let start = padded.len().saturating_sub(length);
    padded[start..].to_string()
}

// ---------------------------------------------------------------------------
// Adler-32 checksum
// ---------------------------------------------------------------------------

/// Compute Adler-32 checksum of `data`.
///
/// Returns 0 if `data.len() > NETWORKMESSAGE_MAXSIZE`.
/// Mirrors `adlerChecksum(const uint8_t* data, size_t length)` in tools.cpp.
pub fn adler_checksum(data: &[u8]) -> u32 {
    use crate::constants::NETWORKMESSAGE_MAXSIZE;
    if data.len() > NETWORKMESSAGE_MAXSIZE as usize {
        return 0;
    }

    const MOD_ADLER: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;

    let mut remaining = data;
    while !remaining.is_empty() {
        let chunk_size = remaining.len().min(5552);
        let (chunk, rest) = remaining.split_at(chunk_size);
        for &byte in chunk {
            a = a.wrapping_add(byte as u32);
            b = b.wrapping_add(a);
        }
        a %= MOD_ADLER;
        b %= MOD_ADLER;
        remaining = rest;
    }

    (b << 16) | a
}

// ---------------------------------------------------------------------------
// Fluid mapping
// ---------------------------------------------------------------------------

/// Server fluid → client fluid index.
///
/// Mirrors `serverFluidToClient(uint8_t serverFluid)` in tools.cpp.
pub fn server_fluid_to_client(server_fluid: u8) -> u8 {
    for (i, &sf) in CLIENT_TO_SERVER_FLUID_MAP.iter().enumerate() {
        if sf == server_fluid {
            return i as u8;
        }
    }
    0
}

/// Client fluid index → server fluid value.
///
/// Mirrors `clientFluidToServer(uint8_t clientFluid)` in tools.cpp.
pub fn client_fluid_to_server(client_fluid: u8) -> u8 {
    let map = &CLIENT_TO_SERVER_FLUID_MAP;
    if client_fluid as usize >= map.len() {
        return 0;
    }
    map[client_fluid as usize]
}

// ---------------------------------------------------------------------------
// Combat type index conversion
// ---------------------------------------------------------------------------

/// CombatType bit flag → index (0-based).
///
/// Mirrors `combatTypeToIndex(CombatType_t)` in tools.cpp.
pub fn combat_type_to_index(combat_type: u16) -> usize {
    match combat_type {
        CombatTypeFlags::PHYSICAL => 0,
        CombatTypeFlags::ENERGY => 1,
        CombatTypeFlags::EARTH => 2,
        CombatTypeFlags::FIRE => 3,
        CombatTypeFlags::UNDEFINED => 4,
        CombatTypeFlags::LIFEDRAIN => 5,
        CombatTypeFlags::MANADRAIN => 6,
        CombatTypeFlags::HEALING => 7,
        CombatTypeFlags::DROWN => 8,
        CombatTypeFlags::ICE => 9,
        CombatTypeFlags::HOLY => 10,
        CombatTypeFlags::DEATH => 11,
        _ => 0,
    }
}

/// Index → CombatType bit flag.
///
/// Mirrors `indexToCombatType(size_t v)` in tools.cpp → `1 << v`.
pub fn index_to_combat_type(v: usize) -> u16 {
    1u16 << v
}

// ---------------------------------------------------------------------------
// Item attribute string → flag
// ---------------------------------------------------------------------------

/// Map attribute name string to `ItemAttrFlags` constant.
///
/// Mirrors `stringToItemAttribute(const std::string& str)` in tools.cpp.
pub fn string_to_item_attribute(s: &str) -> u32 {
    match s {
        "aid" => ItemAttrFlags::ACTIONID,
        "uid" => ItemAttrFlags::UNIQUEID,
        "description" => ItemAttrFlags::DESCRIPTION,
        "text" => ItemAttrFlags::TEXT,
        "date" => ItemAttrFlags::DATE,
        "writer" => ItemAttrFlags::WRITER,
        "name" => ItemAttrFlags::NAME,
        "article" => ItemAttrFlags::ARTICLE,
        "pluralname" => ItemAttrFlags::PLURALNAME,
        "weight" => ItemAttrFlags::WEIGHT,
        "attack" => ItemAttrFlags::ATTACK,
        "defense" => ItemAttrFlags::DEFENSE,
        "extradefense" => ItemAttrFlags::EXTRADEFENSE,
        "armor" => ItemAttrFlags::ARMOR,
        "hitchance" => ItemAttrFlags::HITCHANCE,
        "shootrange" => ItemAttrFlags::SHOOTRANGE,
        "owner" => ItemAttrFlags::OWNER,
        "duration" => ItemAttrFlags::DURATION,
        "decaystate" => ItemAttrFlags::DECAYSTATE,
        "corpseowner" => ItemAttrFlags::CORPSEOWNER,
        "charges" => ItemAttrFlags::CHARGES,
        "fluidtype" => ItemAttrFlags::FLUIDTYPE,
        "doorid" => ItemAttrFlags::DOORID,
        "decayto" => ItemAttrFlags::DECAYTO,
        "wrapid" => ItemAttrFlags::WRAPID,
        "storeitem" => ItemAttrFlags::STOREITEM,
        "attackspeed" => ItemAttrFlags::ATTACK_SPEED,
        _ => ItemAttrFlags::NONE,
    }
}

// ---------------------------------------------------------------------------
// Spell group parsing
// ---------------------------------------------------------------------------

/// Parse a string to `SpellGroup`.
///
/// Mirrors `stringToSpellGroup(const std::string& value)` in tools.cpp.
pub fn string_to_spell_group(value: &str) -> SpellGroup {
    let lower = value.to_lowercase();
    match lower.as_str() {
        "attack" | "1" => SpellGroup::Attack,
        "healing" | "2" => SpellGroup::Healing,
        "support" | "3" => SpellGroup::Support,
        "special" | "4" => SpellGroup::Special,
        _ => SpellGroup::None,
    }
}

// ---------------------------------------------------------------------------
// Skill / SpecialSkill name helpers
// ---------------------------------------------------------------------------

/// Returns the human-readable name of a skill by its `Skill` enum variant index.
///
/// Mirrors `getSkillName(uint8_t skillid)` in tools.cpp.
pub fn get_skill_name(skill_id: u8) -> &'static str {
    match skill_id {
        0 => "fist fighting",
        1 => "club fighting",
        2 => "sword fighting",
        3 => "axe fighting",
        4 => "distance fighting",
        5 => "shielding",
        6 => "fishing",
        7 => "magic level",
        8 => "level",
        _ => "unknown",
    }
}

/// Returns the human-readable name of a special skill by its raw index.
///
/// Mirrors `getSpecialSkillName(uint8_t skillid)` in tools.cpp.
pub fn get_special_skill_name(skill_id: u8) -> &'static str {
    match skill_id {
        0 => "critical hit chance",
        1 => "critical extra damage",
        2 => "hitpoints leech chance",
        3 => "hitpoints leech amount",
        4 => "manapoints leech chance",
        5 => "mana points leech amount",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// Combat name
// ---------------------------------------------------------------------------

/// Returns the name string for a `CombatType` bit flag.
///
/// Mirrors `getCombatName(CombatType_t combatType)` in tools.cpp.
pub fn get_combat_name(combat_type: u16) -> &'static str {
    match combat_type {
        CombatTypeFlags::PHYSICAL => "physical",
        CombatTypeFlags::ENERGY => "energy",
        CombatTypeFlags::EARTH => "earth",
        CombatTypeFlags::FIRE => "fire",
        CombatTypeFlags::UNDEFINED => "undefined",
        CombatTypeFlags::LIFEDRAIN => "lifedrain",
        CombatTypeFlags::MANADRAIN => "manadrain",
        CombatTypeFlags::HEALING => "healing",
        CombatTypeFlags::DROWN => "drown",
        CombatTypeFlags::ICE => "ice",
        CombatTypeFlags::HOLY => "holy",
        CombatTypeFlags::DEATH => "death",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// Return message
// ---------------------------------------------------------------------------

/// Returns the player-visible error message for a `ReturnValue`.
///
/// Mirrors `getReturnMessage(ReturnValue value)` in tools.cpp.
pub fn get_return_message(value: ReturnValue) -> &'static str {
    match value {
        ReturnValue::DestinationOutOfReach => "Destination is out of range.",
        ReturnValue::NotMoveable => "You cannot move this object.",
        ReturnValue::DropTwoHandedItem => "Drop the double-handed object first.",
        ReturnValue::BothHandsNeedToBeFree => "Both hands need to be free.",
        ReturnValue::CannotBeDressed => "You cannot dress this object there.",
        ReturnValue::PutThisObjectInYourHand => "Put this object in your hand.",
        ReturnValue::PutThisObjectInBothHands => "Put this object in both hands.",
        ReturnValue::CanOnlyUseOneWeapon => "You may only use one weapon.",
        ReturnValue::TooFarAway => "You are too far away.",
        ReturnValue::FirstGoDownstairs => "First go downstairs.",
        ReturnValue::FirstGoUpstairs => "First go upstairs.",
        ReturnValue::NotEnoughCapacity => "This object is too heavy for you to carry.",
        ReturnValue::ContainerNotEnoughRoom => {
            "You cannot put more objects in this container."
        }
        ReturnValue::NeedExchange | ReturnValue::NotEnoughRoom => "There is not enough room.",
        ReturnValue::CannotPickup => "You cannot take this object.",
        ReturnValue::CannotThrow => "You cannot throw there.",
        ReturnValue::ThereIsNoWay => "There is no way.",
        ReturnValue::ThisIsImpossible => "This is impossible.",
        ReturnValue::PlayerIsPzLocked => {
            "You can not enter a protection zone after attacking another player."
        }
        ReturnValue::PlayerIsNotInvited => "You are not invited.",
        ReturnValue::CreatureDoesNotExist => "Creature does not exist.",
        ReturnValue::DepotIsFull => "You cannot put more items in this depot.",
        ReturnValue::CannotUseThisObject => "You cannot use this object.",
        ReturnValue::PlayerWithThisNameIsNotOnline => "A player with this name is not online.",
        ReturnValue::NotRequiredLevelToUseRune => {
            "You do not have the required magic level to use this rune."
        }
        ReturnValue::YouAreAlreadyTrading => {
            "You are already trading. Finish this trade first."
        }
        ReturnValue::ThisPlayerIsAlreadyTrading => "This player is already trading.",
        ReturnValue::YouMayNotLogoutDuringAFight => {
            "You may not logout during or immediately after a fight!"
        }
        ReturnValue::DirectPlayerShoot => {
            "You are not allowed to shoot directly on players."
        }
        ReturnValue::NotEnoughLevel => "Your level is too low.",
        ReturnValue::NotEnoughMagicLevel => "You do not have enough magic level.",
        ReturnValue::NotEnoughMana => "You do not have enough mana.",
        ReturnValue::NotEnoughSoul => "You do not have enough soul.",
        ReturnValue::YouAreExhausted => "You are exhausted.",
        ReturnValue::YouCannotUseObjectsThatFast => "You cannot use objects that fast.",
        ReturnValue::CanOnlyUseThisRuneOnCreatures => "You can only use it on creatures.",
        ReturnValue::PlayerIsNotReachable => "Player is not reachable.",
        ReturnValue::CreatureIsNotReachable => "Creature is not reachable.",
        ReturnValue::ActionNotPermittedInProtectionZone => {
            "This action is not permitted in a protection zone."
        }
        ReturnValue::YouMayNotAttackThisPlayer => "You may not attack this person.",
        ReturnValue::YouMayNotAttackThisCreature => "You may not attack this creature.",
        ReturnValue::YouMayNotAttackAPersonInProtectionZone => {
            "You may not attack a person in a protection zone."
        }
        ReturnValue::YouMayNotAttackAPersonWhileInProtectionZone => {
            "You may not attack a person while you are in a protection zone."
        }
        ReturnValue::YouCanOnlyUseItOnCreatures => "You can only use it on creatures.",
        ReturnValue::TurnSecureModeToAttackUnmarkedPlayers => {
            "Turn secure mode off if you really want to attack unmarked players."
        }
        ReturnValue::YouNeedPremiumAccount => "You need a premium account.",
        ReturnValue::YouNeedToLearnThisSpell => "You must learn this spell first.",
        ReturnValue::YourVocationCannotUseThisSpell => {
            "You have the wrong vocation to cast this spell."
        }
        ReturnValue::YouNeedAWeaponToUseThisSpell => {
            "You need to equip a weapon to use this spell."
        }
        ReturnValue::PlayerIsPzLockedLeavePvpZone => {
            "You can not leave a pvp zone after attacking another player."
        }
        ReturnValue::PlayerIsPzLockedEnterPvpZone => {
            "You can not enter a pvp zone after attacking another player."
        }
        ReturnValue::ActionNotPermittedInAnoPvpZone => {
            "This action is not permitted in a non pvp zone."
        }
        ReturnValue::YouCannotLogoutHere => "You can not logout here.",
        ReturnValue::YouNeedAMagicItemToCastSpell => {
            "You need a magic item to cast this spell."
        }
        ReturnValue::NameIsTooAmbiguous => "Player name is ambiguous.",
        ReturnValue::CanOnlyUseOneShield => "You may use only one shield.",
        ReturnValue::NoPartyMembersInRange => "No party members in range.",
        ReturnValue::YouAreNotTheOwner => "You are not the owner.",
        ReturnValue::TradePlayerFarAway => "Trade player is too far away.",
        ReturnValue::YouDontOwnThisHouse => "You don't own this house.",
        ReturnValue::TradePlayerAlreadyOwnsAHouse => "Trade player already owns a house.",
        ReturnValue::TradePlayerHighestBidder => {
            "Trade player is currently the highest bidder of an auctioned house."
        }
        ReturnValue::YouCannotTradeThisHouse => "You can not trade this house.",
        ReturnValue::YouDontHaveRequiredProfession => {
            "You don't have the required profession."
        }
        ReturnValue::CannotMoveItemIsNotStoreItem => {
            "You cannot move this item into your Store inbox as it was not bought in the Store."
        }
        ReturnValue::ItemCannotBeMovedThere => "This item cannot be moved there.",
        ReturnValue::YouCannotUseThisBed => {
            "This bed can't be used, but Premium Account players can rent houses and sleep in beds there to regain health and mana."
        }
        ReturnValue::QuiverAmmoOnly => {
            "This quiver only holds arrows and bolts.\nYou cannot put any other items in it."
        }
        // RETURNVALUE_NOTPOSSIBLE, NoError, CreatureBlock, etc.
        _ => "Sorry, not possible.",
    }
}

// ---------------------------------------------------------------------------
// Depot box ID
// ---------------------------------------------------------------------------

/// Map depot box index (0-based) to the corresponding item ID.
///
/// Mirrors `getDepotBoxId(uint16_t index)` in tools.cpp.
pub fn get_depot_box_id(index: u16) -> u16 {
    use crate::constants::ItemId;
    const DEPOT_BOXES: &[ItemId] = &[
        ItemId::DepotBoxI,
        ItemId::DepotBoxII,
        ItemId::DepotBoxIII,
        ItemId::DepotBoxIV,
        ItemId::DepotBoxV,
        ItemId::DepotBoxVI,
        ItemId::DepotBoxVII,
        ItemId::DepotBoxVIII,
        ItemId::DepotBoxIX,
        ItemId::DepotBoxX,
        ItemId::DepotBoxXI,
        ItemId::DepotBoxXII,
        ItemId::DepotBoxXIII,
        ItemId::DepotBoxXIV,
        ItemId::DepotBoxXV,
        ItemId::DepotBoxXVI,
        ItemId::DepotBoxXVII,
        ItemId::DepotBoxXVIII,
        ItemId::DepotBoxXIX,
        ItemId::DepotBoxXX,
    ];
    DEPOT_BOXES
        .get(index as usize)
        .map(|id| *id as u16)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Effect / shoot / skull / ammo / weapon-action name → enum
// ---------------------------------------------------------------------------

/// Parse a magic effect name to its `MagicEffectClass`.
///
/// Mirrors `getMagicEffect(const std::string& strValue)`.
pub fn get_magic_effect(s: &str) -> MagicEffectClass {
    match s {
        "redspark" => MagicEffectClass::DrawBlood,
        "bluebubble" => MagicEffectClass::LoseEnergy,
        "poff" => MagicEffectClass::Poff,
        "yellowspark" => MagicEffectClass::BlockHit,
        "explosionarea" => MagicEffectClass::ExplosionArea,
        "explosion" => MagicEffectClass::ExplosionHit,
        "firearea" => MagicEffectClass::FireArea,
        "yellowbubble" => MagicEffectClass::YellowRings,
        "greenbubble" => MagicEffectClass::GreenRings,
        "blackspark" => MagicEffectClass::HitArea,
        "teleport" => MagicEffectClass::Teleport,
        "energy" => MagicEffectClass::EnergyHit,
        "blueshimmer" => MagicEffectClass::MagicBlue,
        "redshimmer" => MagicEffectClass::MagicRed,
        "greenshimmer" => MagicEffectClass::MagicGreen,
        "fire" => MagicEffectClass::HitByFire,
        "greenspark" => MagicEffectClass::HitByPoison,
        "mortarea" => MagicEffectClass::MortArea,
        "greennote" => MagicEffectClass::SoundGreen,
        "rednote" => MagicEffectClass::SoundRed,
        "poison" => MagicEffectClass::PoisonArea,
        "yellownote" => MagicEffectClass::SoundYellow,
        "purplenote" => MagicEffectClass::SoundPurple,
        "bluenote" => MagicEffectClass::SoundBlue,
        "whitenote" => MagicEffectClass::SoundWhite,
        "bubbles" => MagicEffectClass::Bubbles,
        "dice" => MagicEffectClass::Craps,
        "giftwraps" => MagicEffectClass::GiftWraps,
        "yellowfirework" => MagicEffectClass::FireworkYellow,
        "redfirework" => MagicEffectClass::FireworkRed,
        "bluefirework" => MagicEffectClass::FireworkBlue,
        "stun" => MagicEffectClass::Stun,
        "sleep" => MagicEffectClass::Sleep,
        "watercreature" => MagicEffectClass::WaterCreature,
        "groundshaker" => MagicEffectClass::GroundShaker,
        "hearts" => MagicEffectClass::Hearts,
        "fireattack" => MagicEffectClass::FireAttack,
        "energyarea" => MagicEffectClass::EnergyArea,
        "smallclouds" => MagicEffectClass::SmallClouds,
        "holydamage" => MagicEffectClass::HolyDamage,
        "bigclouds" => MagicEffectClass::BigClouds,
        "icearea" => MagicEffectClass::IceArea,
        "icetornado" => MagicEffectClass::IceTornado,
        "iceattack" => MagicEffectClass::IceAttack,
        "stones" => MagicEffectClass::Stones,
        "smallplants" => MagicEffectClass::SmallPlants,
        "carniphila" => MagicEffectClass::Carniphila,
        "purpleenergy" => MagicEffectClass::PurpleEnergy,
        "yellowenergy" => MagicEffectClass::YellowEnergy,
        "holyarea" => MagicEffectClass::HolyArea,
        "bigplants" => MagicEffectClass::BigPlants,
        "cake" => MagicEffectClass::Cake,
        "giantice" => MagicEffectClass::GiantIce,
        "watersplash" => MagicEffectClass::WaterSplash,
        "plantattack" => MagicEffectClass::PlantAttack,
        "tutorialarrow" => MagicEffectClass::TutorialArrow,
        "tutorialsquare" => MagicEffectClass::TutorialSquare,
        "mirrorhorizontal" => MagicEffectClass::MirrorHorizontal,
        "mirrorvertical" => MagicEffectClass::MirrorVertical,
        "skullhorizontal" => MagicEffectClass::SkullHorizontal,
        "skullvertical" => MagicEffectClass::SkullVertical,
        "assassin" => MagicEffectClass::Assassin,
        "stepshorizontal" => MagicEffectClass::StepsHorizontal,
        "bloodysteps" => MagicEffectClass::BloodySteps,
        "stepsvertical" => MagicEffectClass::StepsVertical,
        "yalaharighost" => MagicEffectClass::YalaharIGhost,
        "bats" => MagicEffectClass::Bats,
        "smoke" => MagicEffectClass::Smoke,
        "insects" => MagicEffectClass::Insects,
        "dragonhead" => MagicEffectClass::DragonHead,
        "orcshaman" => MagicEffectClass::OrcShaman,
        "orcshamanfire" => MagicEffectClass::OrcShamanFire,
        "thunder" => MagicEffectClass::Thunder,
        "ferumbras" => MagicEffectClass::Ferumbras,
        "confettihorizontal" => MagicEffectClass::ConfettiHorizontal,
        "confettivertical" => MagicEffectClass::ConfettiVertical,
        "blacksmoke" => MagicEffectClass::BlackSmoke,
        "redsmoke" => MagicEffectClass::RedSmoke,
        "yellowsmoke" => MagicEffectClass::YellowSmoke,
        "greensmoke" => MagicEffectClass::GreenSmoke,
        "purplesmoke" => MagicEffectClass::PurpleSmoke,
        "earlythunder" => MagicEffectClass::EarlyThunder,
        "bonecapsule" => MagicEffectClass::RagiazBoneCapsule,
        "criticaldamage" => MagicEffectClass::CriticalDamage,
        "plungingfish" => MagicEffectClass::PlungingFish,
        "bluechain" => MagicEffectClass::BlueChain,
        "orangechain" => MagicEffectClass::OrangeChain,
        "greenchain" => MagicEffectClass::GreenChain,
        "purplechain" => MagicEffectClass::PurpleChain,
        "greychain" => MagicEffectClass::GreyChain,
        "yellowchain" => MagicEffectClass::YellowChain,
        "yellowsparkles" => MagicEffectClass::YellowSparkles,
        "faeexplosion" => MagicEffectClass::FaeExplosion,
        "faecoming" => MagicEffectClass::FaeComing,
        "faegoing" => MagicEffectClass::FaeGoing,
        "bigcloudssinglespace" => MagicEffectClass::BigCloudsSingleSpace,
        "stonessinglespace" => MagicEffectClass::StonesSingleSpace,
        "blueghost" => MagicEffectClass::BlueGhost,
        "pointofinterest" => MagicEffectClass::PointOfInterest,
        "mapeffect" => MagicEffectClass::MapEffect,
        "pinkspark" => MagicEffectClass::PinkSpark,
        "greenfirework" => MagicEffectClass::FireworkGreen,
        "orangefirework" => MagicEffectClass::FireworkOrange,
        "purplefirework" => MagicEffectClass::FireworkPurple,
        "turquoisefirework" => MagicEffectClass::FireworkTurquoise,
        "thecube" => MagicEffectClass::TheCube,
        "drawink" => MagicEffectClass::DrawInk,
        "prismaticsparkles" => MagicEffectClass::PrismaticSparkles,
        "thaian" => MagicEffectClass::Thaian,
        "thaianghost" => MagicEffectClass::ThaianGhost,
        "ghostsmoke" => MagicEffectClass::GhostSmoke,
        "floatingblock" => MagicEffectClass::FloatingBlock,
        "block" => MagicEffectClass::Block,
        "rooting" => MagicEffectClass::Rooting,
        "ghostlyscratch" => MagicEffectClass::GhostlyScratch,
        "ghostlybite" => MagicEffectClass::GhostlyBite,
        "bigscratching" => MagicEffectClass::BigScratching,
        "slash" => MagicEffectClass::Slash,
        "bite" => MagicEffectClass::Bite,
        "chivalriouschallenge" => MagicEffectClass::ChivalrousChallenge,
        "divinedazzle" => MagicEffectClass::DivineDazzle,
        "electricalspark" => MagicEffectClass::ElectricalSpark,
        "purpleteleport" => MagicEffectClass::PurpleTeleport,
        "redteleport" => MagicEffectClass::RedTeleport,
        "orangeteleport" => MagicEffectClass::OrangeTeleport,
        "greyteleport" => MagicEffectClass::GreyTeleport,
        "lightblueteleport" => MagicEffectClass::LightBlueTeleport,
        "fatal" => MagicEffectClass::Fatal,
        "dodge" => MagicEffectClass::Dodge,
        "hourglass" => MagicEffectClass::Hourglass,
        "fireworksstar" => MagicEffectClass::FireworksStar,
        "fireworkscircle" => MagicEffectClass::FireworksCircle,
        "ferumbras1" => MagicEffectClass::Ferumbras1,
        "gazharagoth" => MagicEffectClass::Gazharagoth,
        "madmage" => MagicEffectClass::MadMage,
        "horestis" => MagicEffectClass::Horestis,
        "devovorga" => MagicEffectClass::Devovorga,
        "ferumbras2" => MagicEffectClass::Ferumbras2,
        "foam" => MagicEffectClass::Foam,
        _ => MagicEffectClass::None,
    }
}

/// Parse a shoot-type name to its `ShootType` variant.
pub fn get_shoot_type(s: &str) -> ShootType {
    match s {
        "spear" => ShootType::Spear,
        "bolt" => ShootType::Bolt,
        "arrow" => ShootType::Arrow,
        "fire" => ShootType::Fire,
        "energy" => ShootType::Energy,
        "poisonarrow" => ShootType::PoisonArrow,
        "burstarrow" => ShootType::BurstArrow,
        "throwingstar" => ShootType::ThrowingStar,
        "throwingknife" => ShootType::ThrowingKnife,
        "smallstone" => ShootType::SmallStone,
        "death" => ShootType::Death,
        "largerock" => ShootType::LargeRock,
        "snowball" => ShootType::Snowball,
        "powerbolt" => ShootType::PowerBolt,
        "poison" => ShootType::Poison,
        "infernalbolt" => ShootType::InfernalBolt,
        "huntingspear" => ShootType::HuntingSpear,
        "enchantedspear" => ShootType::EnchantedSpear,
        "redstar" => ShootType::RedStar,
        "greenstar" => ShootType::GreenStar,
        "royalspear" => ShootType::RoyalSpear,
        "sniperarrow" => ShootType::SniperArrow,
        "onyxarrow" => ShootType::OnyxArrow,
        "piercingbolt" => ShootType::PiercingBolt,
        "whirlwindsword" => ShootType::WhirlwindSword,
        "whirlwindaxe" => ShootType::WhirlwindAxe,
        "whirlwindclub" => ShootType::WhirlwindClub,
        "etherealspear" => ShootType::EtherealSpear,
        "ice" => ShootType::Ice,
        "earth" => ShootType::Earth,
        "holy" => ShootType::Holy,
        "suddendeath" => ShootType::SuddenDeath,
        "flasharrow" => ShootType::FlashArrow,
        "flammingarrow" => ShootType::FlammingArrow,
        "shiverarrow" => ShootType::ShiverArrow,
        "energyball" => ShootType::EnergyBall,
        "smallice" => ShootType::SmallIce,
        "smallholy" => ShootType::SmallHoly,
        "smallearth" => ShootType::SmallEarth,
        "eartharrow" => ShootType::EarthArrow,
        "explosion" => ShootType::Explosion,
        "cake" => ShootType::Cake,
        "tarsalarrow" => ShootType::TarsalArrow,
        "vortexbolt" => ShootType::VortexBolt,
        "prismaticbolt" => ShootType::PrismaticBolt,
        "crystallinearrow" => ShootType::CrystallineArrow,
        "drillbolt" => ShootType::DrillBolt,
        "envenomedarrow" => ShootType::EnvenomedArrow,
        "gloothspear" => ShootType::GloothSpear,
        "simplearrow" => ShootType::SimpleArrow,
        "leafstar" => ShootType::LeafStar,
        "diamondarrow" => ShootType::DiamondArrow,
        "spectralbolt" => ShootType::SpectralBolt,
        "royalstar" => ShootType::RoyalStar,
        _ => ShootType::None,
    }
}

/// Parse an ammo type name to its `Ammo` variant.
pub fn get_ammo_type(s: &str) -> Ammo {
    match s {
        "spear" | "huntingspear" | "enchantedspear" | "royalspear" | "etherealspear"
        | "gloothspear" => Ammo::Spear,
        "bolt" | "powerbolt" | "infernalbolt" | "piercingbolt" | "vortexbolt" | "prismaticbolt"
        | "drillbolt" | "spectralbolt" => Ammo::Bolt,
        "arrow" | "poisonarrow" | "burstarrow" | "sniperarrow" | "onyxarrow" | "flasharrow"
        | "flammingarrow" | "shiverarrow" | "eartharrow" | "tarsalarrow" | "crystallinearrow"
        | "envenomedarrow" | "simplearrow" | "diamondarrow" => Ammo::Arrow,
        "throwingstar" | "redstar" | "greenstar" | "leafstar" | "royalstar" => Ammo::ThrowingStar,
        "throwingknife" => Ammo::ThrowingKnife,
        "smallstone" | "largerock" => Ammo::Stone,
        "snowball" => Ammo::Snowball,
        _ => Ammo::None,
    }
}

/// Parse a weapon action name to its `WeaponAction` variant.
pub fn get_weapon_action(s: &str) -> WeaponAction {
    match s {
        "move" => WeaponAction::Move,
        "removecharge" => WeaponAction::RemoveCharge,
        "removecount" => WeaponAction::RemoveCount,
        _ => WeaponAction::None,
    }
}

/// Parse a skull name to its `Skull` variant.
pub fn get_skull_type(s: &str) -> Skull {
    match s {
        "none" => Skull::None,
        "yellow" => Skull::Yellow,
        "green" => Skull::Green,
        "white" => Skull::White,
        "red" => Skull::Red,
        "black" => Skull::Black,
        "orange" => Skull::Orange,
        _ => Skull::None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // ucfirst
    // -----------------------------------------------------------------------

    #[test]
    fn test_ucfirst_capitalises_first_char() {
        assert_eq!(ucfirst("hello world".to_string()), "Hello world");
    }

    #[test]
    fn test_ucfirst_skips_leading_spaces() {
        assert_eq!(ucfirst("  hello".to_string()), "  Hello");
    }

    #[test]
    fn test_ucfirst_empty_string() {
        assert_eq!(ucfirst("".to_string()), "");
    }

    #[test]
    fn test_ucfirst_already_uppercase() {
        assert_eq!(ucfirst("HELLO".to_string()), "HELLO");
    }

    // -----------------------------------------------------------------------
    // ucwords
    // -----------------------------------------------------------------------

    #[test]
    fn test_ucwords_single_word() {
        assert_eq!(ucwords("hello".to_string()), "Hello");
    }

    #[test]
    fn test_ucwords_multiple_words() {
        assert_eq!(ucwords("hello world foo".to_string()), "Hello World Foo");
    }

    #[test]
    fn test_ucwords_empty_string() {
        assert_eq!(ucwords("".to_string()), "");
    }

    #[test]
    fn test_ucwords_already_ucwords() {
        assert_eq!(ucwords("Hello World".to_string()), "Hello World");
    }

    #[test]
    fn test_ucwords_leading_space() {
        // C++ sets str[0] = toupper(str[0]) regardless; space stays space.
        // Then i=1: prev=' ' → 'h'→'H'; i=7: prev=' ' → 'w'→'W'.
        // Result: " Hello World"
        let result = ucwords(" hello world".to_string());
        assert_eq!(result, " Hello World");
    }

    // -----------------------------------------------------------------------
    // boolean_string
    // -----------------------------------------------------------------------

    #[test]
    fn test_boolean_string_empty_is_false() {
        assert!(!boolean_string(""));
    }

    #[test]
    fn test_boolean_string_false_is_false() {
        assert!(!boolean_string("false"));
        assert!(!boolean_string("False"));
        assert!(!boolean_string("FALSE"));
    }

    #[test]
    fn test_boolean_string_no_is_false() {
        assert!(!boolean_string("no"));
        assert!(!boolean_string("No"));
    }

    #[test]
    fn test_boolean_string_zero_is_false() {
        assert!(!boolean_string("0"));
    }

    #[test]
    fn test_boolean_string_true_is_true() {
        assert!(boolean_string("true"));
        assert!(boolean_string("True"));
        assert!(boolean_string("yes"));
        assert!(boolean_string("1"));
        assert!(boolean_string("anything_else"));
    }

    // -----------------------------------------------------------------------
    // case_insensitive_equal
    // -----------------------------------------------------------------------

    #[test]
    fn test_case_insensitive_equal_same() {
        assert!(case_insensitive_equal("Hello", "hello"));
        assert!(case_insensitive_equal("ABC", "abc"));
    }

    #[test]
    fn test_case_insensitive_equal_different() {
        assert!(!case_insensitive_equal("Hello", "World"));
    }

    #[test]
    fn test_case_insensitive_equal_different_length() {
        assert!(!case_insensitive_equal("Hello", "hell"));
    }

    // -----------------------------------------------------------------------
    // case_insensitive_starts_with
    // -----------------------------------------------------------------------

    #[test]
    fn test_case_insensitive_starts_with_true() {
        assert!(case_insensitive_starts_with("HelloWorld", "hello"));
    }

    #[test]
    fn test_case_insensitive_starts_with_false() {
        assert!(!case_insensitive_starts_with("HelloWorld", "world"));
    }

    #[test]
    fn test_case_insensitive_starts_with_exact_match() {
        assert!(case_insensitive_starts_with("hello", "hello"));
    }

    // -----------------------------------------------------------------------
    // explode_string
    // -----------------------------------------------------------------------

    #[test]
    fn test_explode_string_no_limit() {
        let parts = explode_string("a,b,c,d", ",", -1);
        assert_eq!(parts, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_explode_string_limit_2() {
        // C++ --limit logic: limit=2 decrements twice before stopping,
        // producing 3 parts: ["a", "b", "c,d"]
        let parts = explode_string("a,b,c,d", ",", 2);
        assert_eq!(parts, vec!["a", "b", "c,d"]);
    }

    #[test]
    fn test_explode_string_no_separator() {
        let parts = explode_string("hello", ",", -1);
        assert_eq!(parts, vec!["hello"]);
    }

    #[test]
    fn test_explode_string_empty_string() {
        let parts = explode_string("", ",", -1);
        assert_eq!(parts, vec![""]);
    }

    // -----------------------------------------------------------------------
    // vector_atoi
    // -----------------------------------------------------------------------

    #[test]
    fn test_vector_atoi_basic() {
        assert_eq!(vector_atoi(&["1", "2", "3"]), vec![1, 2, 3]);
    }

    #[test]
    fn test_vector_atoi_negative() {
        assert_eq!(vector_atoi(&["-5", "10"]), vec![-5, 10]);
    }

    // -----------------------------------------------------------------------
    // get_first_line
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_first_line_single_line() {
        assert_eq!(get_first_line("hello world"), "hello world");
    }

    #[test]
    fn test_get_first_line_multi_line() {
        assert_eq!(get_first_line("first\nsecond\nthird"), "first");
    }

    #[test]
    fn test_get_first_line_empty() {
        assert_eq!(get_first_line(""), "");
    }

    // -----------------------------------------------------------------------
    // has_bit_set
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_bit_set_true() {
        assert!(has_bit_set(0b0100, 0b1110));
    }

    #[test]
    fn test_has_bit_set_false() {
        assert!(!has_bit_set(0b0001, 0b1110));
    }

    // -----------------------------------------------------------------------
    // uniform_random
    // -----------------------------------------------------------------------

    #[test]
    fn test_uniform_random_same_min_max() {
        assert_eq!(uniform_random(5, 5), 5);
    }

    #[test]
    fn test_uniform_random_in_range() {
        for _ in 0..100 {
            let v = uniform_random(0, 9);
            assert!((0..=9).contains(&v));
        }
    }

    #[test]
    fn test_uniform_random_swaps_when_min_greater() {
        for _ in 0..100 {
            let v = uniform_random(9, 0);
            assert!((0..=9).contains(&v));
        }
    }

    // -----------------------------------------------------------------------
    // boolean_random
    // -----------------------------------------------------------------------

    #[test]
    fn test_boolean_random_always_true_at_1() {
        for _ in 0..10 {
            assert!(boolean_random(1.0));
        }
    }

    #[test]
    fn test_boolean_random_always_false_at_0() {
        for _ in 0..10 {
            assert!(!boolean_random(0.0));
        }
    }

    // -----------------------------------------------------------------------
    // random_bytes
    // -----------------------------------------------------------------------

    #[test]
    fn test_random_bytes_length() {
        let bytes = random_bytes(16);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_random_bytes_zero_length() {
        assert!(random_bytes(0).is_empty());
    }

    // -----------------------------------------------------------------------
    // get_shuffle_directions
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_shuffle_directions_contains_all_four() {
        let dirs = get_shuffle_directions();
        assert_eq!(dirs.len(), 4);
        assert!(dirs.contains(&Direction::North));
        assert!(dirs.contains(&Direction::South));
        assert!(dirs.contains(&Direction::East));
        assert!(dirs.contains(&Direction::West));
    }

    // -----------------------------------------------------------------------
    // otsys_time
    // -----------------------------------------------------------------------

    #[test]
    fn test_otsys_time_is_positive() {
        assert!(otsys_time() > 0);
    }

    // -----------------------------------------------------------------------
    // format_date_short
    // -----------------------------------------------------------------------

    #[test]
    fn test_format_date_short_known_timestamp() {
        // 2023-01-01 00:00:00 UTC = 1672531200
        let s = format_date_short(1672531200);
        assert_eq!(s, "01 Jan 2023");
    }

    #[test]
    fn test_format_date_short_zero() {
        let s = format_date_short(0);
        assert_eq!(s, "01 Jan 1970");
    }

    #[test]
    fn test_format_date_short_leap_day() {
        // 2024-02-29 00:00:00 UTC = 1709164800
        let s = format_date_short(1709164800);
        assert_eq!(s, "29 Feb 2024");
    }

    // -----------------------------------------------------------------------
    // SHA-1 — transform_to_sha1
    // -----------------------------------------------------------------------

    #[test]
    fn test_sha1_empty_input() {
        // SHA1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709
        let digest = transform_to_sha1(b"");
        let expected: [u8; 20] = [
            0xda, 0x39, 0xa3, 0xee, 0x5e, 0x6b, 0x4b, 0x0d, 0x32, 0x55, 0xbf, 0xef, 0x95, 0x60,
            0x18, 0x90, 0xaf, 0xd8, 0x07, 0x09,
        ];
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_sha1_single_byte() {
        // SHA1("\x36") = c1dfd96eea8cc2b62785275bca38ac261256e278
        let digest = transform_to_sha1(&[0x36]);
        let expected: [u8; 20] = [
            0xc1, 0xdf, 0xd9, 0x6e, 0xea, 0x8c, 0xc2, 0xb6, 0x27, 0x85, 0x27, 0x5b, 0xca, 0x38,
            0xac, 0x26, 0x12, 0x56, 0xe2, 0x78,
        ];
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_sha1_two_bytes() {
        let digest = transform_to_sha1(&[0x19, 0x5a]);
        let expected: [u8; 20] = [
            0x0a, 0x1c, 0x2d, 0x55, 0x5b, 0xbe, 0x43, 0x1a, 0xd6, 0x28, 0x8a, 0xf5, 0xa5, 0x4f,
            0x93, 0xe0, 0x44, 0x9c, 0x92, 0x32,
        ];
        assert_eq!(digest, expected);
    }

    #[test]
    fn test_sha1_hex_empty() {
        let hex = transform_to_sha1_hex(b"");
        assert_eq!(hex, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    // -----------------------------------------------------------------------
    // HMAC-SHA1 — RFC 2202 test vectors
    // -----------------------------------------------------------------------

    #[test]
    fn test_hmac_sha1_rfc2202_vector1() {
        let key: Vec<u8> = vec![0x0bu8; 20];
        let msg = b"Hi There";
        let expected: [u8; 20] = [
            0xb6, 0x17, 0x31, 0x86, 0x55, 0x05, 0x72, 0x64, 0xe2, 0x8b, 0xc0, 0xb6, 0xfb, 0x37,
            0x8c, 0x8e, 0xf1, 0x46, 0xbe, 0x00,
        ];
        assert_eq!(hmac_sha1(&key, msg), expected);
    }

    #[test]
    fn test_hmac_sha1_rfc2202_vector2() {
        let key = b"Jefe";
        let msg = b"what do ya want for nothing?";
        let expected: [u8; 20] = [
            0xef, 0xfc, 0xdf, 0x6a, 0xe5, 0xeb, 0x2f, 0xa2, 0xd2, 0x74, 0x16, 0xd5, 0xf1, 0x84,
            0xdf, 0x9c, 0x25, 0x9a, 0x7c, 0x79,
        ];
        assert_eq!(hmac_sha1(key, msg), expected);
    }

    #[test]
    fn test_hmac_sha1_rfc2202_vector3() {
        let key: Vec<u8> = vec![0xaa; 20];
        let msg: Vec<u8> = vec![0xdd; 50];
        let expected: [u8; 20] = [
            0x12, 0x5d, 0x73, 0x42, 0xb9, 0xac, 0x11, 0xcd, 0x91, 0xa3, 0x9a, 0xf4, 0x8a, 0xa1,
            0x7b, 0x4f, 0x63, 0xf1, 0x75, 0xd3,
        ];
        assert_eq!(hmac_sha1(&key, &msg), expected);
    }

    #[test]
    fn test_hmac_sha1_rfc2202_vector4() {
        let key: &[u8] = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
        ];
        let msg: Vec<u8> = vec![0xcd; 50];
        let expected: [u8; 20] = [
            0x4c, 0x90, 0x07, 0xf4, 0x02, 0x62, 0x50, 0xc6, 0xbc, 0x84, 0x14, 0xf9, 0xbf, 0x50,
            0xc8, 0x6c, 0x2d, 0x72, 0x35, 0xda,
        ];
        assert_eq!(hmac_sha1(key, &msg), expected);
    }

    #[test]
    fn test_hmac_sha1_rfc2202_vector5() {
        let key: Vec<u8> = vec![0x0c; 20];
        let msg = b"Test With Truncation";
        let expected: [u8; 20] = [
            0x4c, 0x1a, 0x03, 0x42, 0x4b, 0x55, 0xe0, 0x7f, 0xe7, 0xf2, 0x7b, 0xe1, 0xd5, 0x8b,
            0xb9, 0x32, 0x4a, 0x9a, 0x5a, 0x04,
        ];
        assert_eq!(hmac_sha1(&key, msg), expected);
    }

    #[test]
    fn test_hmac_sha1_rfc2202_vector6() {
        let key: Vec<u8> = vec![0xaa; 80];
        let msg = b"Test Using Larger Than Block-Size Key - Hash Key First";
        let expected: [u8; 20] = [
            0xaa, 0x4a, 0xe5, 0xe1, 0x52, 0x72, 0xd0, 0x0e, 0x95, 0x70, 0x56, 0x37, 0xce, 0x8a,
            0x3b, 0x55, 0xed, 0x40, 0x21, 0x12,
        ];
        assert_eq!(hmac_sha1(&key, msg), expected);
    }

    #[test]
    fn test_hmac_sha1_rfc2202_vector7() {
        let key: Vec<u8> = vec![0xaa; 80];
        let msg = b"Test Using Larger Than Block-Size Key and Larger Than One Block-Size Data";
        let expected: [u8; 20] = [
            0xe8, 0xe9, 0x9d, 0x0f, 0x45, 0x23, 0x7d, 0x78, 0x6d, 0x6b, 0xba, 0xa7, 0x96, 0x5c,
            0x78, 0x08, 0xbb, 0xff, 0x1a, 0x91,
        ];
        assert_eq!(hmac_sha1(&key, msg), expected);
    }

    // -----------------------------------------------------------------------
    // TOTP — generate_token  (RFC 6238 Appendix B vectors, 8-digit)
    // -----------------------------------------------------------------------

    #[test]
    fn test_totp_rfc6238_time59() {
        let key = b"12345678901234567890";
        let token = generate_token(key, 59 / 30, 8);
        assert_eq!(token, "94287082");
    }

    #[test]
    fn test_totp_rfc6238_time1111111109() {
        let key = b"12345678901234567890";
        let token = generate_token(key, 1111111109 / 30, 8);
        assert_eq!(token, "07081804");
    }

    #[test]
    fn test_totp_rfc6238_time1111111111() {
        let key = b"12345678901234567890";
        let token = generate_token(key, 1111111111 / 30, 8);
        assert_eq!(token, "14050471");
    }

    #[test]
    fn test_totp_rfc6238_time1234567890() {
        let key = b"12345678901234567890";
        let token = generate_token(key, 1234567890 / 30, 8);
        assert_eq!(token, "89005924");
    }

    #[test]
    fn test_totp_rfc6238_time2000000000() {
        let key = b"12345678901234567890";
        let token = generate_token(key, 2000000000 / 30, 8);
        assert_eq!(token, "69279037");
    }

    #[test]
    fn test_totp_rfc6238_time20000000000() {
        let key = b"12345678901234567890";
        let token = generate_token(key, 20000000000u64 / 30, 8);
        assert_eq!(token, "65353130");
    }

    // -----------------------------------------------------------------------
    // Adler-32
    // -----------------------------------------------------------------------

    #[test]
    fn test_adler32_empty() {
        // adler32("") = 1 (a=1, b=0)
        assert_eq!(adler_checksum(&[]), 1);
    }

    #[test]
    fn test_adler32_wikipedia_example() {
        // adler32("Wikipedia") = 0x11E60398
        // Verified against reference implementation
        let result = adler_checksum(b"Wikipedia");
        assert_eq!(result, 0x11E6_0398);
    }

    #[test]
    fn test_adler32_exceeds_max_returns_zero() {
        use crate::constants::NETWORKMESSAGE_MAXSIZE;
        let oversized = vec![0u8; NETWORKMESSAGE_MAXSIZE as usize + 1];
        assert_eq!(adler_checksum(&oversized), 0);
    }

    // -----------------------------------------------------------------------
    // Fluid mapping
    // -----------------------------------------------------------------------

    #[test]
    fn test_server_fluid_to_client_water() {
        // Server fluid 1 (FLUID_WATER) → client index 1
        assert_eq!(server_fluid_to_client(1), 1);
    }

    #[test]
    fn test_server_fluid_to_client_unknown_returns_zero() {
        assert_eq!(server_fluid_to_client(255), 0);
    }

    #[test]
    fn test_client_fluid_to_server_water() {
        // Client index 1 → server fluid 1
        assert_eq!(client_fluid_to_server(1), 1);
    }

    #[test]
    fn test_client_fluid_to_server_out_of_range() {
        assert_eq!(client_fluid_to_server(200), 0);
    }

    // -----------------------------------------------------------------------
    // Combat type index
    // -----------------------------------------------------------------------

    #[test]
    fn test_combat_type_to_index_physical() {
        assert_eq!(combat_type_to_index(CombatTypeFlags::PHYSICAL), 0);
    }

    #[test]
    fn test_combat_type_to_index_death() {
        assert_eq!(combat_type_to_index(CombatTypeFlags::DEATH), 11);
    }

    #[test]
    fn test_index_to_combat_type_zero() {
        assert_eq!(index_to_combat_type(0), CombatTypeFlags::PHYSICAL);
    }

    #[test]
    fn test_index_to_combat_type_one() {
        assert_eq!(index_to_combat_type(1), CombatTypeFlags::ENERGY);
    }

    // -----------------------------------------------------------------------
    // Item attribute string
    // -----------------------------------------------------------------------

    #[test]
    fn test_string_to_item_attribute_aid() {
        assert_eq!(string_to_item_attribute("aid"), ItemAttrFlags::ACTIONID);
    }

    #[test]
    fn test_string_to_item_attribute_unknown() {
        assert_eq!(string_to_item_attribute("bogus"), ItemAttrFlags::NONE);
    }

    #[test]
    fn test_string_to_item_attribute_attackspeed() {
        assert_eq!(
            string_to_item_attribute("attackspeed"),
            ItemAttrFlags::ATTACK_SPEED
        );
    }

    // -----------------------------------------------------------------------
    // Spell group
    // -----------------------------------------------------------------------

    #[test]
    fn test_string_to_spell_group_attack() {
        assert_eq!(string_to_spell_group("attack"), SpellGroup::Attack);
        assert_eq!(string_to_spell_group("1"), SpellGroup::Attack);
    }

    #[test]
    fn test_string_to_spell_group_healing() {
        assert_eq!(string_to_spell_group("healing"), SpellGroup::Healing);
    }

    #[test]
    fn test_string_to_spell_group_support() {
        assert_eq!(string_to_spell_group("support"), SpellGroup::Support);
    }

    #[test]
    fn test_string_to_spell_group_special() {
        assert_eq!(string_to_spell_group("special"), SpellGroup::Special);
        assert_eq!(string_to_spell_group("SPECIAL"), SpellGroup::Special);
    }

    #[test]
    fn test_string_to_spell_group_unknown() {
        assert_eq!(string_to_spell_group("unknown"), SpellGroup::None);
    }

    // -----------------------------------------------------------------------
    // Skill name
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_skill_name_fist() {
        assert_eq!(get_skill_name(0), "fist fighting");
    }

    #[test]
    fn test_get_skill_name_level() {
        assert_eq!(get_skill_name(8), "level");
    }

    #[test]
    fn test_get_skill_name_unknown() {
        assert_eq!(get_skill_name(100), "unknown");
    }

    // -----------------------------------------------------------------------
    // Special skill name
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_special_skill_name_critical_hit_chance() {
        assert_eq!(get_special_skill_name(0), "critical hit chance");
    }

    #[test]
    fn test_get_special_skill_name_mana_leech_amount() {
        assert_eq!(get_special_skill_name(5), "mana points leech amount");
    }

    #[test]
    fn test_get_special_skill_name_unknown() {
        assert_eq!(get_special_skill_name(255), "unknown");
    }

    // -----------------------------------------------------------------------
    // Combat name
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_combat_name_physical() {
        assert_eq!(get_combat_name(CombatTypeFlags::PHYSICAL), "physical");
    }

    #[test]
    fn test_get_combat_name_unknown() {
        assert_eq!(get_combat_name(0xFFFF), "unknown");
    }

    // -----------------------------------------------------------------------
    // Return message
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_return_message_not_moveable() {
        assert_eq!(
            get_return_message(ReturnValue::NotMoveable),
            "You cannot move this object."
        );
    }

    #[test]
    fn test_get_return_message_default() {
        assert_eq!(
            get_return_message(ReturnValue::NotPossible),
            "Sorry, not possible."
        );
    }

    #[test]
    fn test_get_return_message_need_exchange() {
        // NeedExchange and NotEnoughRoom share the same message
        assert_eq!(
            get_return_message(ReturnValue::NeedExchange),
            "There is not enough room."
        );
        assert_eq!(
            get_return_message(ReturnValue::NotEnoughRoom),
            "There is not enough room."
        );
    }

    // -----------------------------------------------------------------------
    // Depot box ID
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_depot_box_id_zero() {
        use crate::constants::ItemId;
        assert_eq!(get_depot_box_id(0), ItemId::DepotBoxI as u16);
    }

    #[test]
    fn test_get_depot_box_id_out_of_range() {
        assert_eq!(get_depot_box_id(100), 0);
    }

    // -----------------------------------------------------------------------
    // Magic effect
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_magic_effect_redspark() {
        assert_eq!(get_magic_effect("redspark"), MagicEffectClass::DrawBlood);
    }

    #[test]
    fn test_get_magic_effect_unknown() {
        assert_eq!(get_magic_effect("UNKNOWN"), MagicEffectClass::None);
    }

    // -----------------------------------------------------------------------
    // Shoot type
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_shoot_type_spear() {
        assert_eq!(get_shoot_type("spear"), ShootType::Spear);
    }

    #[test]
    fn test_get_shoot_type_unknown() {
        assert_eq!(get_shoot_type("garbage"), ShootType::None);
    }

    // -----------------------------------------------------------------------
    // Ammo type
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_ammo_type_arrow() {
        assert_eq!(get_ammo_type("arrow"), Ammo::Arrow);
    }

    #[test]
    fn test_get_ammo_type_bolt() {
        assert_eq!(get_ammo_type("bolt"), Ammo::Bolt);
    }

    #[test]
    fn test_get_ammo_type_unknown() {
        assert_eq!(get_ammo_type("nope"), Ammo::None);
    }

    // -----------------------------------------------------------------------
    // Weapon action
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_weapon_action_move() {
        assert_eq!(get_weapon_action("move"), WeaponAction::Move);
    }

    #[test]
    fn test_get_weapon_action_unknown() {
        assert_eq!(get_weapon_action("jump"), WeaponAction::None);
    }

    // -----------------------------------------------------------------------
    // Skull type
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_skull_type_red() {
        assert_eq!(get_skull_type("red"), Skull::Red);
    }

    #[test]
    fn test_get_skull_type_none() {
        assert_eq!(get_skull_type("none"), Skull::None);
    }

    #[test]
    fn test_get_skull_type_unknown() {
        assert_eq!(get_skull_type("pink"), Skull::None);
    }

    // -----------------------------------------------------------------------
    // normal_random
    // -----------------------------------------------------------------------

    #[test]
    fn test_normal_random_same_min_max() {
        assert_eq!(normal_random(7, 7), 7);
    }

    #[test]
    fn test_normal_random_in_range() {
        for _ in 0..200 {
            let v = normal_random(0, 100);
            assert!(
                (0..=100).contains(&v),
                "normal_random(0,100) out of range: {v}"
            );
        }
    }

    #[test]
    fn test_normal_random_swaps_when_min_greater() {
        for _ in 0..200 {
            let v = normal_random(10, 0);
            assert!(
                (0..=10).contains(&v),
                "normal_random(10,0) out of range: {v}"
            );
        }
    }

    #[test]
    fn test_normal_random_biased_toward_middle() {
        // With N(0.5, 0.25²) most values should land in the middle half [25,75] of [0,100]
        let count_middle = (0..1000)
            .filter(|_| (25..=75).contains(&normal_random(0, 100)))
            .count();
        // The middle 50% of N(0.5, 0.25) after truncation is heavily concentrated
        // near the mean, so >60% should fall in the middle half.
        assert!(
            count_middle > 600,
            "expected >600/1000 in middle half, got {count_middle}"
        );
    }

    // -----------------------------------------------------------------------
    // to_roman_numeral
    // -----------------------------------------------------------------------

    #[test]
    fn test_roman_numeral_invalid_zero() {
        assert_eq!(to_roman_numeral(0), None);
    }

    #[test]
    fn test_roman_numeral_invalid_4000() {
        assert_eq!(to_roman_numeral(4000), None);
    }

    #[test]
    fn test_roman_numeral_invalid_large() {
        assert_eq!(to_roman_numeral(10000), None);
    }

    #[test]
    fn test_roman_numeral_1() {
        assert_eq!(to_roman_numeral(1), Some("I".to_string()));
    }

    #[test]
    fn test_roman_numeral_4() {
        assert_eq!(to_roman_numeral(4), Some("IV".to_string()));
    }

    #[test]
    fn test_roman_numeral_5() {
        assert_eq!(to_roman_numeral(5), Some("V".to_string()));
    }

    #[test]
    fn test_roman_numeral_9() {
        assert_eq!(to_roman_numeral(9), Some("IX".to_string()));
    }

    #[test]
    fn test_roman_numeral_14() {
        assert_eq!(to_roman_numeral(14), Some("XIV".to_string()));
    }

    #[test]
    fn test_roman_numeral_40() {
        assert_eq!(to_roman_numeral(40), Some("XL".to_string()));
    }

    #[test]
    fn test_roman_numeral_49() {
        assert_eq!(to_roman_numeral(49), Some("XLIX".to_string()));
    }

    #[test]
    fn test_roman_numeral_50() {
        assert_eq!(to_roman_numeral(50), Some("L".to_string()));
    }

    #[test]
    fn test_roman_numeral_90() {
        assert_eq!(to_roman_numeral(90), Some("XC".to_string()));
    }

    #[test]
    fn test_roman_numeral_100() {
        assert_eq!(to_roman_numeral(100), Some("C".to_string()));
    }

    #[test]
    fn test_roman_numeral_400() {
        assert_eq!(to_roman_numeral(400), Some("CD".to_string()));
    }

    #[test]
    fn test_roman_numeral_500() {
        assert_eq!(to_roman_numeral(500), Some("D".to_string()));
    }

    #[test]
    fn test_roman_numeral_900() {
        assert_eq!(to_roman_numeral(900), Some("CM".to_string()));
    }

    #[test]
    fn test_roman_numeral_1000() {
        assert_eq!(to_roman_numeral(1000), Some("M".to_string()));
    }

    #[test]
    fn test_roman_numeral_1994() {
        assert_eq!(to_roman_numeral(1994), Some("MCMXCIV".to_string()));
    }

    #[test]
    fn test_roman_numeral_2024() {
        assert_eq!(to_roman_numeral(2024), Some("MMXXIV".to_string()));
    }

    #[test]
    fn test_roman_numeral_3999() {
        assert_eq!(to_roman_numeral(3999), Some("MMMCMXCIX".to_string()));
    }

    /// Depot box labels I through XX — the primary use case in-game.
    #[test]
    fn test_roman_numeral_depot_boxes_i_through_xx() {
        let expected = [
            (1, "I"),
            (2, "II"),
            (3, "III"),
            (4, "IV"),
            (5, "V"),
            (6, "VI"),
            (7, "VII"),
            (8, "VIII"),
            (9, "IX"),
            (10, "X"),
            (11, "XI"),
            (12, "XII"),
            (13, "XIII"),
            (14, "XIV"),
            (15, "XV"),
            (16, "XVI"),
            (17, "XVII"),
            (18, "XVIII"),
            (19, "XIX"),
            (20, "XX"),
        ];
        for (n, s) in expected {
            assert_eq!(
                to_roman_numeral(n),
                Some(s.to_string()),
                "to_roman_numeral({n}) should be {s}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Additional coverage: combat_type_to_index — all 12 types
    // -----------------------------------------------------------------------

    #[test]
    fn test_combat_type_to_index_all() {
        assert_eq!(combat_type_to_index(CombatTypeFlags::PHYSICAL), 0);
        assert_eq!(combat_type_to_index(CombatTypeFlags::ENERGY), 1);
        assert_eq!(combat_type_to_index(CombatTypeFlags::EARTH), 2);
        assert_eq!(combat_type_to_index(CombatTypeFlags::FIRE), 3);
        assert_eq!(combat_type_to_index(CombatTypeFlags::UNDEFINED), 4);
        assert_eq!(combat_type_to_index(CombatTypeFlags::LIFEDRAIN), 5);
        assert_eq!(combat_type_to_index(CombatTypeFlags::MANADRAIN), 6);
        assert_eq!(combat_type_to_index(CombatTypeFlags::HEALING), 7);
        assert_eq!(combat_type_to_index(CombatTypeFlags::DROWN), 8);
        assert_eq!(combat_type_to_index(CombatTypeFlags::ICE), 9);
        assert_eq!(combat_type_to_index(CombatTypeFlags::HOLY), 10);
        assert_eq!(combat_type_to_index(CombatTypeFlags::DEATH), 11);
        // Unknown falls back to 0
        assert_eq!(combat_type_to_index(0xFFFF), 0);
    }

    #[test]
    fn test_index_to_combat_type_roundtrip_all() {
        // index_to_combat_type(i) == 1 << i for i in 0..12
        for i in 0u16..12 {
            assert_eq!(index_to_combat_type(i as usize), 1u16 << i);
        }
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_combat_name — all types
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_combat_name_all() {
        assert_eq!(get_combat_name(CombatTypeFlags::PHYSICAL), "physical");
        assert_eq!(get_combat_name(CombatTypeFlags::ENERGY), "energy");
        assert_eq!(get_combat_name(CombatTypeFlags::EARTH), "earth");
        assert_eq!(get_combat_name(CombatTypeFlags::FIRE), "fire");
        assert_eq!(get_combat_name(CombatTypeFlags::UNDEFINED), "undefined");
        assert_eq!(get_combat_name(CombatTypeFlags::LIFEDRAIN), "lifedrain");
        assert_eq!(get_combat_name(CombatTypeFlags::MANADRAIN), "manadrain");
        assert_eq!(get_combat_name(CombatTypeFlags::HEALING), "healing");
        assert_eq!(get_combat_name(CombatTypeFlags::DROWN), "drown");
        assert_eq!(get_combat_name(CombatTypeFlags::ICE), "ice");
        assert_eq!(get_combat_name(CombatTypeFlags::HOLY), "holy");
        assert_eq!(get_combat_name(CombatTypeFlags::DEATH), "death");
    }

    // -----------------------------------------------------------------------
    // Additional coverage: vector_atoi — invalid strings
    // -----------------------------------------------------------------------

    #[test]
    fn test_vector_atoi_invalid_string_returns_zero() {
        assert_eq!(vector_atoi(&["abc", "2"]), vec![0, 2]);
    }

    #[test]
    fn test_vector_atoi_empty_slice() {
        let result: Vec<i32> = vector_atoi(&[]);
        assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // Additional coverage: format_date_short — negative timestamp
    // -----------------------------------------------------------------------

    #[test]
    fn test_format_date_short_negative_clamps_to_epoch() {
        // Negative timestamps are clamped to 0 (Unix epoch = 01 Jan 1970)
        let s = format_date_short(-1);
        assert_eq!(s, "01 Jan 1970");
    }

    // -----------------------------------------------------------------------
    // Additional coverage: explode_string with limit=0 (same as -1: no limit)
    // -----------------------------------------------------------------------

    #[test]
    fn test_explode_string_limit_zero_same_as_unlimited() {
        // limit=0: the while loop condition `remaining != 0` is false immediately,
        // which breaks the loop — we just get ["a,b,c,d"] (the remainder).
        // NOTE: limit=0 is an edge case that returns the whole string as one part,
        // matching the C++ behaviour where --limit starts at 0 → immediately -1.
        let parts = explode_string("a,b,c,d", ",", 0);
        assert_eq!(parts, vec!["a,b,c,d"]);
    }

    #[test]
    fn test_explode_string_limit_1() {
        // limit=1: only one split allowed → produces 2 parts
        let parts = explode_string("a,b,c", ",", 1);
        assert_eq!(parts, vec!["a", "b,c"]);
    }

    #[test]
    fn test_explode_string_multi_char_separator() {
        let parts = explode_string("a::b::c", "::", -1);
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    // -----------------------------------------------------------------------
    // Additional coverage: boolean_string edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_boolean_string_uppercase_no() {
        assert!(!boolean_string("NO"));
    }

    #[test]
    fn test_boolean_string_single_char_n() {
        assert!(!boolean_string("n"));
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_skill_name — all skills
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_skill_name_all() {
        assert_eq!(get_skill_name(0), "fist fighting");
        assert_eq!(get_skill_name(1), "club fighting");
        assert_eq!(get_skill_name(2), "sword fighting");
        assert_eq!(get_skill_name(3), "axe fighting");
        assert_eq!(get_skill_name(4), "distance fighting");
        assert_eq!(get_skill_name(5), "shielding");
        assert_eq!(get_skill_name(6), "fishing");
        assert_eq!(get_skill_name(7), "magic level");
        assert_eq!(get_skill_name(8), "level");
        assert_eq!(get_skill_name(9), "unknown");
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_special_skill_name — all special skills
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_special_skill_name_all() {
        assert_eq!(get_special_skill_name(0), "critical hit chance");
        assert_eq!(get_special_skill_name(1), "critical extra damage");
        assert_eq!(get_special_skill_name(2), "hitpoints leech chance");
        assert_eq!(get_special_skill_name(3), "hitpoints leech amount");
        assert_eq!(get_special_skill_name(4), "manapoints leech chance");
        assert_eq!(get_special_skill_name(5), "mana points leech amount");
        assert_eq!(get_special_skill_name(6), "unknown");
    }

    // -----------------------------------------------------------------------
    // Additional coverage: fluid round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_fluid_round_trip() {
        // For each valid client index, server_fluid_to_client(client_fluid_to_server(i)) == i
        use crate::constants::CLIENT_TO_SERVER_FLUID_MAP;
        for i in 0..CLIENT_TO_SERVER_FLUID_MAP.len() as u8 {
            let server = client_fluid_to_server(i);
            let client_back = server_fluid_to_client(server);
            if server != 0 {
                // Non-zero server fluids should round-trip back to their client index
                assert_eq!(
                    client_back, i,
                    "fluid round-trip failed for client index {i}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Additional coverage: adler_checksum — known vectors
    // -----------------------------------------------------------------------

    #[test]
    fn test_adler32_abc() {
        // Adler-32("abc"):
        // Step 1: a=1+97=98,  b=0+98=98
        // Step 2: a=98+98=196, b=98+196=294
        // Step 3: a=196+99=295, b=294+295=589
        // (all values < 65521, so no modulo reduction changes them)
        // result = (b << 16) | a = (589 << 16) | 295 = 38_600_999
        let result = adler_checksum(b"abc");
        assert_eq!(result, 38_600_999);
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_depot_box_id — all 20 boxes
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_depot_box_id_all_valid() {
        use crate::constants::ItemId;
        // All 20 depot boxes should return non-zero item IDs
        let expected = [
            ItemId::DepotBoxI,
            ItemId::DepotBoxII,
            ItemId::DepotBoxIII,
            ItemId::DepotBoxIV,
            ItemId::DepotBoxV,
            ItemId::DepotBoxVI,
            ItemId::DepotBoxVII,
            ItemId::DepotBoxVIII,
            ItemId::DepotBoxIX,
            ItemId::DepotBoxX,
            ItemId::DepotBoxXI,
            ItemId::DepotBoxXII,
            ItemId::DepotBoxXIII,
            ItemId::DepotBoxXIV,
            ItemId::DepotBoxXV,
            ItemId::DepotBoxXVI,
            ItemId::DepotBoxXVII,
            ItemId::DepotBoxXVIII,
            ItemId::DepotBoxXIX,
            ItemId::DepotBoxXX,
        ];
        for (i, &id) in expected.iter().enumerate() {
            assert_eq!(
                get_depot_box_id(i as u16),
                id as u16,
                "depot box {i} mismatch"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Additional coverage: has_bit_set edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_bit_set_all_bits() {
        assert!(has_bit_set(0xFFFF_FFFF, 0xFFFF_FFFF));
    }

    #[test]
    fn test_has_bit_set_zero_flag() {
        // flag=0 always returns false (0 & anything == 0)
        assert!(!has_bit_set(0, 0xFFFF_FFFF));
    }

    // -----------------------------------------------------------------------
    // Additional coverage: case_insensitive_starts_with — empty prefix
    // -----------------------------------------------------------------------

    #[test]
    fn test_case_insensitive_starts_with_empty_prefix() {
        // Any string starts with an empty prefix
        assert!(case_insensitive_starts_with("hello", ""));
        assert!(case_insensitive_starts_with("", ""));
    }

    #[test]
    fn test_case_insensitive_starts_with_longer_prefix_than_string() {
        assert!(!case_insensitive_starts_with("hi", "hello"));
    }

    // -----------------------------------------------------------------------
    // Exhaustive coverage: get_return_message — every ReturnValue variant
    // -----------------------------------------------------------------------
    //
    // Mirrors the C++ switch in tools.cpp::getReturnMessage; each arm
    // (including the catch-all "Sorry, not possible.") is asserted here so
    // line coverage hits every match arm.
    //
    // Pairs that share a message string in C++ (e.g. NEEDEXCHANGE /
    // NOTENOUGHROOM) are tested in `test_get_return_message_need_exchange`
    // above; the cases below assert each distinct variant individually.

    #[test]
    fn test_get_return_message_destination_out_of_reach() {
        assert_eq!(
            get_return_message(ReturnValue::DestinationOutOfReach),
            "Destination is out of range."
        );
    }

    #[test]
    fn test_get_return_message_drop_two_handed_item() {
        assert_eq!(
            get_return_message(ReturnValue::DropTwoHandedItem),
            "Drop the double-handed object first."
        );
    }

    #[test]
    fn test_get_return_message_both_hands_need_to_be_free() {
        assert_eq!(
            get_return_message(ReturnValue::BothHandsNeedToBeFree),
            "Both hands need to be free."
        );
    }

    #[test]
    fn test_get_return_message_cannot_be_dressed() {
        assert_eq!(
            get_return_message(ReturnValue::CannotBeDressed),
            "You cannot dress this object there."
        );
    }

    #[test]
    fn test_get_return_message_put_this_object_in_your_hand() {
        assert_eq!(
            get_return_message(ReturnValue::PutThisObjectInYourHand),
            "Put this object in your hand."
        );
    }

    #[test]
    fn test_get_return_message_put_this_object_in_both_hands() {
        assert_eq!(
            get_return_message(ReturnValue::PutThisObjectInBothHands),
            "Put this object in both hands."
        );
    }

    #[test]
    fn test_get_return_message_can_only_use_one_weapon() {
        assert_eq!(
            get_return_message(ReturnValue::CanOnlyUseOneWeapon),
            "You may only use one weapon."
        );
    }

    #[test]
    fn test_get_return_message_too_far_away() {
        assert_eq!(
            get_return_message(ReturnValue::TooFarAway),
            "You are too far away."
        );
    }

    #[test]
    fn test_get_return_message_first_go_downstairs() {
        assert_eq!(
            get_return_message(ReturnValue::FirstGoDownstairs),
            "First go downstairs."
        );
    }

    #[test]
    fn test_get_return_message_first_go_upstairs() {
        assert_eq!(
            get_return_message(ReturnValue::FirstGoUpstairs),
            "First go upstairs."
        );
    }

    #[test]
    fn test_get_return_message_not_enough_capacity() {
        assert_eq!(
            get_return_message(ReturnValue::NotEnoughCapacity),
            "This object is too heavy for you to carry."
        );
    }

    #[test]
    fn test_get_return_message_container_not_enough_room() {
        assert_eq!(
            get_return_message(ReturnValue::ContainerNotEnoughRoom),
            "You cannot put more objects in this container."
        );
    }

    #[test]
    fn test_get_return_message_cannot_pickup() {
        assert_eq!(
            get_return_message(ReturnValue::CannotPickup),
            "You cannot take this object."
        );
    }

    #[test]
    fn test_get_return_message_cannot_throw() {
        assert_eq!(
            get_return_message(ReturnValue::CannotThrow),
            "You cannot throw there."
        );
    }

    #[test]
    fn test_get_return_message_there_is_no_way() {
        assert_eq!(
            get_return_message(ReturnValue::ThereIsNoWay),
            "There is no way."
        );
    }

    #[test]
    fn test_get_return_message_this_is_impossible() {
        assert_eq!(
            get_return_message(ReturnValue::ThisIsImpossible),
            "This is impossible."
        );
    }

    #[test]
    fn test_get_return_message_player_is_pz_locked() {
        assert_eq!(
            get_return_message(ReturnValue::PlayerIsPzLocked),
            "You can not enter a protection zone after attacking another player."
        );
    }

    #[test]
    fn test_get_return_message_player_is_not_invited() {
        assert_eq!(
            get_return_message(ReturnValue::PlayerIsNotInvited),
            "You are not invited."
        );
    }

    #[test]
    fn test_get_return_message_creature_does_not_exist() {
        assert_eq!(
            get_return_message(ReturnValue::CreatureDoesNotExist),
            "Creature does not exist."
        );
    }

    #[test]
    fn test_get_return_message_depot_is_full() {
        assert_eq!(
            get_return_message(ReturnValue::DepotIsFull),
            "You cannot put more items in this depot."
        );
    }

    #[test]
    fn test_get_return_message_cannot_use_this_object() {
        assert_eq!(
            get_return_message(ReturnValue::CannotUseThisObject),
            "You cannot use this object."
        );
    }

    #[test]
    fn test_get_return_message_player_with_this_name_is_not_online() {
        assert_eq!(
            get_return_message(ReturnValue::PlayerWithThisNameIsNotOnline),
            "A player with this name is not online."
        );
    }

    #[test]
    fn test_get_return_message_not_required_level_to_use_rune() {
        assert_eq!(
            get_return_message(ReturnValue::NotRequiredLevelToUseRune),
            "You do not have the required magic level to use this rune."
        );
    }

    #[test]
    fn test_get_return_message_you_are_already_trading() {
        assert_eq!(
            get_return_message(ReturnValue::YouAreAlreadyTrading),
            "You are already trading. Finish this trade first."
        );
    }

    #[test]
    fn test_get_return_message_this_player_is_already_trading() {
        assert_eq!(
            get_return_message(ReturnValue::ThisPlayerIsAlreadyTrading),
            "This player is already trading."
        );
    }

    #[test]
    fn test_get_return_message_you_may_not_logout_during_a_fight() {
        assert_eq!(
            get_return_message(ReturnValue::YouMayNotLogoutDuringAFight),
            "You may not logout during or immediately after a fight!"
        );
    }

    #[test]
    fn test_get_return_message_direct_player_shoot() {
        assert_eq!(
            get_return_message(ReturnValue::DirectPlayerShoot),
            "You are not allowed to shoot directly on players."
        );
    }

    #[test]
    fn test_get_return_message_not_enough_level() {
        assert_eq!(
            get_return_message(ReturnValue::NotEnoughLevel),
            "Your level is too low."
        );
    }

    #[test]
    fn test_get_return_message_not_enough_magic_level() {
        assert_eq!(
            get_return_message(ReturnValue::NotEnoughMagicLevel),
            "You do not have enough magic level."
        );
    }

    #[test]
    fn test_get_return_message_not_enough_mana() {
        assert_eq!(
            get_return_message(ReturnValue::NotEnoughMana),
            "You do not have enough mana."
        );
    }

    #[test]
    fn test_get_return_message_not_enough_soul() {
        assert_eq!(
            get_return_message(ReturnValue::NotEnoughSoul),
            "You do not have enough soul."
        );
    }

    #[test]
    fn test_get_return_message_you_are_exhausted() {
        assert_eq!(
            get_return_message(ReturnValue::YouAreExhausted),
            "You are exhausted."
        );
    }

    #[test]
    fn test_get_return_message_you_cannot_use_objects_that_fast() {
        assert_eq!(
            get_return_message(ReturnValue::YouCannotUseObjectsThatFast),
            "You cannot use objects that fast."
        );
    }

    #[test]
    fn test_get_return_message_can_only_use_this_rune_on_creatures() {
        assert_eq!(
            get_return_message(ReturnValue::CanOnlyUseThisRuneOnCreatures),
            "You can only use it on creatures."
        );
    }

    #[test]
    fn test_get_return_message_player_is_not_reachable() {
        assert_eq!(
            get_return_message(ReturnValue::PlayerIsNotReachable),
            "Player is not reachable."
        );
    }

    #[test]
    fn test_get_return_message_creature_is_not_reachable() {
        assert_eq!(
            get_return_message(ReturnValue::CreatureIsNotReachable),
            "Creature is not reachable."
        );
    }

    #[test]
    fn test_get_return_message_action_not_permitted_in_protection_zone() {
        assert_eq!(
            get_return_message(ReturnValue::ActionNotPermittedInProtectionZone),
            "This action is not permitted in a protection zone."
        );
    }

    #[test]
    fn test_get_return_message_you_may_not_attack_this_player() {
        assert_eq!(
            get_return_message(ReturnValue::YouMayNotAttackThisPlayer),
            "You may not attack this person."
        );
    }

    #[test]
    fn test_get_return_message_you_may_not_attack_this_creature() {
        assert_eq!(
            get_return_message(ReturnValue::YouMayNotAttackThisCreature),
            "You may not attack this creature."
        );
    }

    #[test]
    fn test_get_return_message_you_may_not_attack_person_in_protection_zone() {
        assert_eq!(
            get_return_message(ReturnValue::YouMayNotAttackAPersonInProtectionZone),
            "You may not attack a person in a protection zone."
        );
    }

    #[test]
    fn test_get_return_message_you_may_not_attack_person_while_in_protection_zone() {
        assert_eq!(
            get_return_message(ReturnValue::YouMayNotAttackAPersonWhileInProtectionZone),
            "You may not attack a person while you are in a protection zone."
        );
    }

    #[test]
    fn test_get_return_message_you_can_only_use_it_on_creatures() {
        assert_eq!(
            get_return_message(ReturnValue::YouCanOnlyUseItOnCreatures),
            "You can only use it on creatures."
        );
    }

    #[test]
    fn test_get_return_message_turn_secure_mode_to_attack_unmarked_players() {
        assert_eq!(
            get_return_message(ReturnValue::TurnSecureModeToAttackUnmarkedPlayers),
            "Turn secure mode off if you really want to attack unmarked players."
        );
    }

    #[test]
    fn test_get_return_message_you_need_premium_account() {
        assert_eq!(
            get_return_message(ReturnValue::YouNeedPremiumAccount),
            "You need a premium account."
        );
    }

    #[test]
    fn test_get_return_message_you_need_to_learn_this_spell() {
        assert_eq!(
            get_return_message(ReturnValue::YouNeedToLearnThisSpell),
            "You must learn this spell first."
        );
    }

    #[test]
    fn test_get_return_message_your_vocation_cannot_use_this_spell() {
        assert_eq!(
            get_return_message(ReturnValue::YourVocationCannotUseThisSpell),
            "You have the wrong vocation to cast this spell."
        );
    }

    #[test]
    fn test_get_return_message_you_need_a_weapon_to_use_this_spell() {
        assert_eq!(
            get_return_message(ReturnValue::YouNeedAWeaponToUseThisSpell),
            "You need to equip a weapon to use this spell."
        );
    }

    #[test]
    fn test_get_return_message_player_is_pz_locked_leave_pvp_zone() {
        assert_eq!(
            get_return_message(ReturnValue::PlayerIsPzLockedLeavePvpZone),
            "You can not leave a pvp zone after attacking another player."
        );
    }

    #[test]
    fn test_get_return_message_player_is_pz_locked_enter_pvp_zone() {
        assert_eq!(
            get_return_message(ReturnValue::PlayerIsPzLockedEnterPvpZone),
            "You can not enter a pvp zone after attacking another player."
        );
    }

    #[test]
    fn test_get_return_message_action_not_permitted_in_a_no_pvp_zone() {
        assert_eq!(
            get_return_message(ReturnValue::ActionNotPermittedInAnoPvpZone),
            "This action is not permitted in a non pvp zone."
        );
    }

    #[test]
    fn test_get_return_message_you_cannot_logout_here() {
        assert_eq!(
            get_return_message(ReturnValue::YouCannotLogoutHere),
            "You can not logout here."
        );
    }

    #[test]
    fn test_get_return_message_you_need_a_magic_item_to_cast_spell() {
        assert_eq!(
            get_return_message(ReturnValue::YouNeedAMagicItemToCastSpell),
            "You need a magic item to cast this spell."
        );
    }

    #[test]
    fn test_get_return_message_name_is_too_ambiguous() {
        assert_eq!(
            get_return_message(ReturnValue::NameIsTooAmbiguous),
            "Player name is ambiguous."
        );
    }

    #[test]
    fn test_get_return_message_can_only_use_one_shield() {
        assert_eq!(
            get_return_message(ReturnValue::CanOnlyUseOneShield),
            "You may use only one shield."
        );
    }

    #[test]
    fn test_get_return_message_no_party_members_in_range() {
        assert_eq!(
            get_return_message(ReturnValue::NoPartyMembersInRange),
            "No party members in range."
        );
    }

    #[test]
    fn test_get_return_message_you_are_not_the_owner() {
        assert_eq!(
            get_return_message(ReturnValue::YouAreNotTheOwner),
            "You are not the owner."
        );
    }

    #[test]
    fn test_get_return_message_trade_player_far_away() {
        assert_eq!(
            get_return_message(ReturnValue::TradePlayerFarAway),
            "Trade player is too far away."
        );
    }

    #[test]
    fn test_get_return_message_you_dont_own_this_house() {
        assert_eq!(
            get_return_message(ReturnValue::YouDontOwnThisHouse),
            "You don't own this house."
        );
    }

    #[test]
    fn test_get_return_message_trade_player_already_owns_a_house() {
        assert_eq!(
            get_return_message(ReturnValue::TradePlayerAlreadyOwnsAHouse),
            "Trade player already owns a house."
        );
    }

    #[test]
    fn test_get_return_message_trade_player_highest_bidder() {
        assert_eq!(
            get_return_message(ReturnValue::TradePlayerHighestBidder),
            "Trade player is currently the highest bidder of an auctioned house."
        );
    }

    #[test]
    fn test_get_return_message_you_cannot_trade_this_house() {
        assert_eq!(
            get_return_message(ReturnValue::YouCannotTradeThisHouse),
            "You can not trade this house."
        );
    }

    #[test]
    fn test_get_return_message_you_dont_have_required_profession() {
        assert_eq!(
            get_return_message(ReturnValue::YouDontHaveRequiredProfession),
            "You don't have the required profession."
        );
    }

    #[test]
    fn test_get_return_message_cannot_move_item_is_not_store_item() {
        assert_eq!(
            get_return_message(ReturnValue::CannotMoveItemIsNotStoreItem),
            "You cannot move this item into your Store inbox as it was not bought in the Store."
        );
    }

    #[test]
    fn test_get_return_message_item_cannot_be_moved_there() {
        assert_eq!(
            get_return_message(ReturnValue::ItemCannotBeMovedThere),
            "This item cannot be moved there."
        );
    }

    #[test]
    fn test_get_return_message_you_cannot_use_this_bed() {
        assert_eq!(
            get_return_message(ReturnValue::YouCannotUseThisBed),
            "This bed can't be used, but Premium Account players can rent houses and sleep in beds there to regain health and mana."
        );
    }

    #[test]
    fn test_get_return_message_quiver_ammo_only() {
        assert_eq!(
            get_return_message(ReturnValue::QuiverAmmoOnly),
            "This quiver only holds arrows and bolts.\nYou cannot put any other items in it."
        );
    }

    #[test]
    fn test_get_return_message_no_error_falls_through_to_default() {
        // NoError is not explicitly handled — it hits the catch-all.
        assert_eq!(
            get_return_message(ReturnValue::NoError),
            "Sorry, not possible."
        );
    }

    #[test]
    fn test_get_return_message_creature_block_falls_through_to_default() {
        // CreatureBlock is not explicitly handled — it hits the catch-all.
        assert_eq!(
            get_return_message(ReturnValue::CreatureBlock),
            "Sorry, not possible."
        );
    }

    // -----------------------------------------------------------------------
    // Additional coverage: stringToItemAttribute — every variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_string_to_item_attribute_all_variants() {
        assert_eq!(string_to_item_attribute("aid"), ItemAttrFlags::ACTIONID);
        assert_eq!(string_to_item_attribute("uid"), ItemAttrFlags::UNIQUEID);
        assert_eq!(
            string_to_item_attribute("description"),
            ItemAttrFlags::DESCRIPTION
        );
        assert_eq!(string_to_item_attribute("text"), ItemAttrFlags::TEXT);
        assert_eq!(string_to_item_attribute("date"), ItemAttrFlags::DATE);
        assert_eq!(string_to_item_attribute("writer"), ItemAttrFlags::WRITER);
        assert_eq!(string_to_item_attribute("name"), ItemAttrFlags::NAME);
        assert_eq!(string_to_item_attribute("article"), ItemAttrFlags::ARTICLE);
        assert_eq!(
            string_to_item_attribute("pluralname"),
            ItemAttrFlags::PLURALNAME
        );
        assert_eq!(string_to_item_attribute("weight"), ItemAttrFlags::WEIGHT);
        assert_eq!(string_to_item_attribute("attack"), ItemAttrFlags::ATTACK);
        assert_eq!(string_to_item_attribute("defense"), ItemAttrFlags::DEFENSE);
        assert_eq!(
            string_to_item_attribute("extradefense"),
            ItemAttrFlags::EXTRADEFENSE
        );
        assert_eq!(string_to_item_attribute("armor"), ItemAttrFlags::ARMOR);
        assert_eq!(
            string_to_item_attribute("hitchance"),
            ItemAttrFlags::HITCHANCE
        );
        assert_eq!(
            string_to_item_attribute("shootrange"),
            ItemAttrFlags::SHOOTRANGE
        );
        assert_eq!(string_to_item_attribute("owner"), ItemAttrFlags::OWNER);
        assert_eq!(
            string_to_item_attribute("duration"),
            ItemAttrFlags::DURATION
        );
        assert_eq!(
            string_to_item_attribute("decaystate"),
            ItemAttrFlags::DECAYSTATE
        );
        assert_eq!(
            string_to_item_attribute("corpseowner"),
            ItemAttrFlags::CORPSEOWNER
        );
        assert_eq!(string_to_item_attribute("charges"), ItemAttrFlags::CHARGES);
        assert_eq!(
            string_to_item_attribute("fluidtype"),
            ItemAttrFlags::FLUIDTYPE
        );
        assert_eq!(string_to_item_attribute("doorid"), ItemAttrFlags::DOORID);
        assert_eq!(string_to_item_attribute("decayto"), ItemAttrFlags::DECAYTO);
        assert_eq!(string_to_item_attribute("wrapid"), ItemAttrFlags::WRAPID);
        assert_eq!(
            string_to_item_attribute("storeitem"),
            ItemAttrFlags::STOREITEM
        );
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_magic_effect — every name in the C++ map
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_magic_effect_all_named() {
        // Mirrors the magicEffectNames map in tools.cpp.  Every entry must
        // map to its corresponding MagicEffectClass variant.
        let cases: &[(&str, MagicEffectClass)] = &[
            ("redspark", MagicEffectClass::DrawBlood),
            ("bluebubble", MagicEffectClass::LoseEnergy),
            ("poff", MagicEffectClass::Poff),
            ("yellowspark", MagicEffectClass::BlockHit),
            ("explosionarea", MagicEffectClass::ExplosionArea),
            ("explosion", MagicEffectClass::ExplosionHit),
            ("firearea", MagicEffectClass::FireArea),
            ("yellowbubble", MagicEffectClass::YellowRings),
            ("greenbubble", MagicEffectClass::GreenRings),
            ("blackspark", MagicEffectClass::HitArea),
            ("teleport", MagicEffectClass::Teleport),
            ("energy", MagicEffectClass::EnergyHit),
            ("blueshimmer", MagicEffectClass::MagicBlue),
            ("redshimmer", MagicEffectClass::MagicRed),
            ("greenshimmer", MagicEffectClass::MagicGreen),
            ("fire", MagicEffectClass::HitByFire),
            ("greenspark", MagicEffectClass::HitByPoison),
            ("mortarea", MagicEffectClass::MortArea),
            ("greennote", MagicEffectClass::SoundGreen),
            ("rednote", MagicEffectClass::SoundRed),
            ("poison", MagicEffectClass::PoisonArea),
            ("yellownote", MagicEffectClass::SoundYellow),
            ("purplenote", MagicEffectClass::SoundPurple),
            ("bluenote", MagicEffectClass::SoundBlue),
            ("whitenote", MagicEffectClass::SoundWhite),
            ("bubbles", MagicEffectClass::Bubbles),
            ("dice", MagicEffectClass::Craps),
            ("giftwraps", MagicEffectClass::GiftWraps),
            ("yellowfirework", MagicEffectClass::FireworkYellow),
            ("redfirework", MagicEffectClass::FireworkRed),
            ("bluefirework", MagicEffectClass::FireworkBlue),
            ("stun", MagicEffectClass::Stun),
            ("sleep", MagicEffectClass::Sleep),
            ("watercreature", MagicEffectClass::WaterCreature),
            ("groundshaker", MagicEffectClass::GroundShaker),
            ("hearts", MagicEffectClass::Hearts),
            ("fireattack", MagicEffectClass::FireAttack),
            ("energyarea", MagicEffectClass::EnergyArea),
            ("smallclouds", MagicEffectClass::SmallClouds),
            ("holydamage", MagicEffectClass::HolyDamage),
            ("bigclouds", MagicEffectClass::BigClouds),
            ("icearea", MagicEffectClass::IceArea),
            ("icetornado", MagicEffectClass::IceTornado),
            ("iceattack", MagicEffectClass::IceAttack),
            ("stones", MagicEffectClass::Stones),
            ("smallplants", MagicEffectClass::SmallPlants),
            ("carniphila", MagicEffectClass::Carniphila),
            ("purpleenergy", MagicEffectClass::PurpleEnergy),
            ("yellowenergy", MagicEffectClass::YellowEnergy),
            ("holyarea", MagicEffectClass::HolyArea),
            ("bigplants", MagicEffectClass::BigPlants),
            ("cake", MagicEffectClass::Cake),
            ("giantice", MagicEffectClass::GiantIce),
            ("watersplash", MagicEffectClass::WaterSplash),
            ("plantattack", MagicEffectClass::PlantAttack),
            ("tutorialarrow", MagicEffectClass::TutorialArrow),
            ("tutorialsquare", MagicEffectClass::TutorialSquare),
            ("mirrorhorizontal", MagicEffectClass::MirrorHorizontal),
            ("mirrorvertical", MagicEffectClass::MirrorVertical),
            ("skullhorizontal", MagicEffectClass::SkullHorizontal),
            ("skullvertical", MagicEffectClass::SkullVertical),
            ("assassin", MagicEffectClass::Assassin),
            ("stepshorizontal", MagicEffectClass::StepsHorizontal),
            ("bloodysteps", MagicEffectClass::BloodySteps),
            ("stepsvertical", MagicEffectClass::StepsVertical),
            ("yalaharighost", MagicEffectClass::YalaharIGhost),
            ("bats", MagicEffectClass::Bats),
            ("smoke", MagicEffectClass::Smoke),
            ("insects", MagicEffectClass::Insects),
            ("dragonhead", MagicEffectClass::DragonHead),
            ("orcshaman", MagicEffectClass::OrcShaman),
            ("orcshamanfire", MagicEffectClass::OrcShamanFire),
            ("thunder", MagicEffectClass::Thunder),
            ("ferumbras", MagicEffectClass::Ferumbras),
            ("confettihorizontal", MagicEffectClass::ConfettiHorizontal),
            ("confettivertical", MagicEffectClass::ConfettiVertical),
            ("blacksmoke", MagicEffectClass::BlackSmoke),
            ("redsmoke", MagicEffectClass::RedSmoke),
            ("yellowsmoke", MagicEffectClass::YellowSmoke),
            ("greensmoke", MagicEffectClass::GreenSmoke),
            ("purplesmoke", MagicEffectClass::PurpleSmoke),
            ("earlythunder", MagicEffectClass::EarlyThunder),
            ("bonecapsule", MagicEffectClass::RagiazBoneCapsule),
            ("criticaldamage", MagicEffectClass::CriticalDamage),
            ("plungingfish", MagicEffectClass::PlungingFish),
            ("bluechain", MagicEffectClass::BlueChain),
            ("orangechain", MagicEffectClass::OrangeChain),
            ("greenchain", MagicEffectClass::GreenChain),
            ("purplechain", MagicEffectClass::PurpleChain),
            ("greychain", MagicEffectClass::GreyChain),
            ("yellowchain", MagicEffectClass::YellowChain),
            ("yellowsparkles", MagicEffectClass::YellowSparkles),
            ("faeexplosion", MagicEffectClass::FaeExplosion),
            ("faecoming", MagicEffectClass::FaeComing),
            ("faegoing", MagicEffectClass::FaeGoing),
            (
                "bigcloudssinglespace",
                MagicEffectClass::BigCloudsSingleSpace,
            ),
            ("stonessinglespace", MagicEffectClass::StonesSingleSpace),
            ("blueghost", MagicEffectClass::BlueGhost),
            ("pointofinterest", MagicEffectClass::PointOfInterest),
            ("mapeffect", MagicEffectClass::MapEffect),
            ("pinkspark", MagicEffectClass::PinkSpark),
            ("greenfirework", MagicEffectClass::FireworkGreen),
            ("orangefirework", MagicEffectClass::FireworkOrange),
            ("purplefirework", MagicEffectClass::FireworkPurple),
            ("turquoisefirework", MagicEffectClass::FireworkTurquoise),
            ("thecube", MagicEffectClass::TheCube),
            ("drawink", MagicEffectClass::DrawInk),
            ("prismaticsparkles", MagicEffectClass::PrismaticSparkles),
            ("thaian", MagicEffectClass::Thaian),
            ("thaianghost", MagicEffectClass::ThaianGhost),
            ("ghostsmoke", MagicEffectClass::GhostSmoke),
            ("floatingblock", MagicEffectClass::FloatingBlock),
            ("block", MagicEffectClass::Block),
            ("rooting", MagicEffectClass::Rooting),
            ("ghostlyscratch", MagicEffectClass::GhostlyScratch),
            ("ghostlybite", MagicEffectClass::GhostlyBite),
            ("bigscratching", MagicEffectClass::BigScratching),
            ("slash", MagicEffectClass::Slash),
            ("bite", MagicEffectClass::Bite),
            (
                "chivalriouschallenge",
                MagicEffectClass::ChivalrousChallenge,
            ),
            ("divinedazzle", MagicEffectClass::DivineDazzle),
            ("electricalspark", MagicEffectClass::ElectricalSpark),
            ("purpleteleport", MagicEffectClass::PurpleTeleport),
            ("redteleport", MagicEffectClass::RedTeleport),
            ("orangeteleport", MagicEffectClass::OrangeTeleport),
            ("greyteleport", MagicEffectClass::GreyTeleport),
            ("lightblueteleport", MagicEffectClass::LightBlueTeleport),
            ("fatal", MagicEffectClass::Fatal),
            ("dodge", MagicEffectClass::Dodge),
            ("hourglass", MagicEffectClass::Hourglass),
            ("fireworksstar", MagicEffectClass::FireworksStar),
            ("fireworkscircle", MagicEffectClass::FireworksCircle),
            ("ferumbras1", MagicEffectClass::Ferumbras1),
            ("gazharagoth", MagicEffectClass::Gazharagoth),
            ("madmage", MagicEffectClass::MadMage),
            ("horestis", MagicEffectClass::Horestis),
            ("devovorga", MagicEffectClass::Devovorga),
            ("ferumbras2", MagicEffectClass::Ferumbras2),
            ("foam", MagicEffectClass::Foam),
        ];
        for &(name, expected) in cases {
            assert_eq!(
                get_magic_effect(name),
                expected,
                "get_magic_effect({name:?}) should be {expected:?}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_shoot_type — every name in the C++ map
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_shoot_type_all_named() {
        let cases: &[(&str, ShootType)] = &[
            ("spear", ShootType::Spear),
            ("bolt", ShootType::Bolt),
            ("arrow", ShootType::Arrow),
            ("fire", ShootType::Fire),
            ("energy", ShootType::Energy),
            ("poisonarrow", ShootType::PoisonArrow),
            ("burstarrow", ShootType::BurstArrow),
            ("throwingstar", ShootType::ThrowingStar),
            ("throwingknife", ShootType::ThrowingKnife),
            ("smallstone", ShootType::SmallStone),
            ("death", ShootType::Death),
            ("largerock", ShootType::LargeRock),
            ("snowball", ShootType::Snowball),
            ("powerbolt", ShootType::PowerBolt),
            ("poison", ShootType::Poison),
            ("infernalbolt", ShootType::InfernalBolt),
            ("huntingspear", ShootType::HuntingSpear),
            ("enchantedspear", ShootType::EnchantedSpear),
            ("redstar", ShootType::RedStar),
            ("greenstar", ShootType::GreenStar),
            ("royalspear", ShootType::RoyalSpear),
            ("sniperarrow", ShootType::SniperArrow),
            ("onyxarrow", ShootType::OnyxArrow),
            ("piercingbolt", ShootType::PiercingBolt),
            ("whirlwindsword", ShootType::WhirlwindSword),
            ("whirlwindaxe", ShootType::WhirlwindAxe),
            ("whirlwindclub", ShootType::WhirlwindClub),
            ("etherealspear", ShootType::EtherealSpear),
            ("ice", ShootType::Ice),
            ("earth", ShootType::Earth),
            ("holy", ShootType::Holy),
            ("suddendeath", ShootType::SuddenDeath),
            ("flasharrow", ShootType::FlashArrow),
            ("flammingarrow", ShootType::FlammingArrow),
            ("shiverarrow", ShootType::ShiverArrow),
            ("energyball", ShootType::EnergyBall),
            ("smallice", ShootType::SmallIce),
            ("smallholy", ShootType::SmallHoly),
            ("smallearth", ShootType::SmallEarth),
            ("eartharrow", ShootType::EarthArrow),
            ("explosion", ShootType::Explosion),
            ("cake", ShootType::Cake),
            ("tarsalarrow", ShootType::TarsalArrow),
            ("vortexbolt", ShootType::VortexBolt),
            ("prismaticbolt", ShootType::PrismaticBolt),
            ("crystallinearrow", ShootType::CrystallineArrow),
            ("drillbolt", ShootType::DrillBolt),
            ("envenomedarrow", ShootType::EnvenomedArrow),
            ("gloothspear", ShootType::GloothSpear),
            ("simplearrow", ShootType::SimpleArrow),
            ("leafstar", ShootType::LeafStar),
            ("diamondarrow", ShootType::DiamondArrow),
            ("spectralbolt", ShootType::SpectralBolt),
            ("royalstar", ShootType::RoyalStar),
        ];
        for &(name, expected) in cases {
            assert_eq!(
                get_shoot_type(name),
                expected,
                "get_shoot_type({name:?}) should be {expected:?}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_ammo_type — every name in the C++ map
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_ammo_type_all_named() {
        let cases: &[(&str, Ammo)] = &[
            ("spear", Ammo::Spear),
            ("bolt", Ammo::Bolt),
            ("arrow", Ammo::Arrow),
            ("poisonarrow", Ammo::Arrow),
            ("burstarrow", Ammo::Arrow),
            ("throwingstar", Ammo::ThrowingStar),
            ("throwingknife", Ammo::ThrowingKnife),
            ("smallstone", Ammo::Stone),
            ("largerock", Ammo::Stone),
            ("snowball", Ammo::Snowball),
            ("powerbolt", Ammo::Bolt),
            ("infernalbolt", Ammo::Bolt),
            ("huntingspear", Ammo::Spear),
            ("enchantedspear", Ammo::Spear),
            ("royalspear", Ammo::Spear),
            ("sniperarrow", Ammo::Arrow),
            ("onyxarrow", Ammo::Arrow),
            ("piercingbolt", Ammo::Bolt),
            ("etherealspear", Ammo::Spear),
            ("flasharrow", Ammo::Arrow),
            ("flammingarrow", Ammo::Arrow),
            ("shiverarrow", Ammo::Arrow),
            ("eartharrow", Ammo::Arrow),
            ("tarsalarrow", Ammo::Arrow),
            ("vortexbolt", Ammo::Bolt),
            ("prismaticbolt", Ammo::Bolt),
            ("crystallinearrow", Ammo::Arrow),
            ("drillbolt", Ammo::Bolt),
            ("envenomedarrow", Ammo::Arrow),
            ("gloothspear", Ammo::Spear),
            ("simplearrow", Ammo::Arrow),
            ("redstar", Ammo::ThrowingStar),
            ("greenstar", Ammo::ThrowingStar),
            ("leafstar", Ammo::ThrowingStar),
            ("diamondarrow", Ammo::Arrow),
            ("spectralbolt", Ammo::Bolt),
            ("royalstar", Ammo::ThrowingStar),
        ];
        for &(name, expected) in cases {
            assert_eq!(
                get_ammo_type(name),
                expected,
                "get_ammo_type({name:?}) should be {expected:?}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_weapon_action — every variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_weapon_action_all() {
        assert_eq!(get_weapon_action("move"), WeaponAction::Move);
        assert_eq!(
            get_weapon_action("removecharge"),
            WeaponAction::RemoveCharge
        );
        assert_eq!(get_weapon_action("removecount"), WeaponAction::RemoveCount);
        assert_eq!(get_weapon_action(""), WeaponAction::None);
    }

    // -----------------------------------------------------------------------
    // Additional coverage: get_skull_type — every variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_skull_type_all() {
        assert_eq!(get_skull_type("none"), Skull::None);
        assert_eq!(get_skull_type("yellow"), Skull::Yellow);
        assert_eq!(get_skull_type("green"), Skull::Green);
        assert_eq!(get_skull_type("white"), Skull::White);
        assert_eq!(get_skull_type("red"), Skull::Red);
        assert_eq!(get_skull_type("black"), Skull::Black);
        assert_eq!(get_skull_type("orange"), Skull::Orange);
    }
}
