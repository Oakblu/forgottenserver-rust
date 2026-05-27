use std::collections::HashMap;

/// Simplified weapon kind (game-logic layer, distinct from `weapon_registry.rs`).
/// Mirrors `WeaponType_t` from const.h.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Melee,
    Distance,
    Wand,
}

/// Element type for wands and elemental weapons (combat type identifier).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    None,
    Physical,
    Fire,
    Energy,
    Ice,
    Earth,
    Holy,
    Death,
}

// ---------------------------------------------------------------------------
// Static damage formulae (mirrors `Weapons::getMaxMeleeDamage` /
// `Weapons::getMaxWeaponDamage` from weapons.cpp)
// ---------------------------------------------------------------------------

/// Monster melee max-damage formula:
///   `ceil(attackSkill * attackValue * 0.05 + attackValue * 0.5)`
pub fn get_max_melee_damage(attack_skill: i32, attack_value: i32) -> i32 {
    let raw = (attack_skill as f64 * attack_value as f64 * 0.05) + (attack_value as f64 * 0.5);
    raw.ceil() as i32
}

/// Player weapon max-damage formula:
///   `round((level/5) + ((((attackSkill/4+1) * (attackValue/3)) * 1.03) / attackFactor))`
pub fn get_max_weapon_damage(
    level: u32,
    attack_skill: i32,
    attack_value: i32,
    attack_factor: f32,
) -> i32 {
    let inner = (((attack_skill as f64 / 4.0) + 1.0) * (attack_value as f64 / 3.0)) * 1.03
        / attack_factor as f64;
    let result = (level as f64 / 5.0) + inner;
    result.round() as i32
}

// ---------------------------------------------------------------------------
// Melee weapon damage formula
// ---------------------------------------------------------------------------

pub struct WeaponMelee;

impl WeaponMelee {
    /// `attack * skill_level / 20 + (seed % (attack / 3 + 1))`
    pub fn compute_damage(attack: u32, skill_level: u32, seed: u32) -> u32 {
        let base = attack * skill_level / 20;
        let rng_part = seed % (attack / 3 + 1);
        base + rng_part
    }
}

// ---------------------------------------------------------------------------
// Distance weapon damage formula
// ---------------------------------------------------------------------------

pub struct WeaponDistance;

impl WeaponDistance {
    /// Same formula as melee.
    pub fn compute_damage(attack: u32, dist_skill: u32, seed: u32) -> u32 {
        WeaponMelee::compute_damage(attack, dist_skill, seed)
    }

    /// Check whether a target is within `shoot_range` tiles (Chebyshev distance).
    /// Returns false when `player_distance > shoot_range`.
    pub fn check_range(
        player_x: i32,
        player_y: i32,
        target_x: i32,
        target_y: i32,
        shoot_range: u32,
    ) -> bool {
        let dx = (player_x - target_x).unsigned_abs();
        let dy = (player_y - target_y).unsigned_abs();
        let chebyshev = dx.max(dy);
        chebyshev <= shoot_range
    }

    /// Returns element damage for a distance weapon.
    /// When `element_type` is `ElementType::None` the bonus is 0.
    pub fn get_element_damage(element_type: ElementType, base_element_damage: i32) -> i32 {
        if element_type == ElementType::None {
            0
        } else {
            base_element_damage
        }
    }
}

// ---------------------------------------------------------------------------
// Generic weapon (melee / distance)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Weapon {
    pub item_id: u32,
    pub kind: WeaponKind,
    pub min_level: u32,
    pub min_mag_level: u32,
    pub attack: u32,
    pub defense: u32,
    pub shoot_range: u32,
    pub enabled: bool,
    pub element_type: ElementType,
    pub element_damage: u32,
}

impl Weapon {
    pub fn new(item_id: u32, kind: WeaponKind, min_level: u32, attack: u32, defense: u32) -> Self {
        Weapon {
            item_id,
            kind,
            min_level,
            min_mag_level: 0,
            attack,
            defense,
            shoot_range: 1,
            enabled: true,
            element_type: ElementType::None,
            element_damage: 0,
        }
    }

    /// Builder-style setter for `shoot_range`.
    pub fn with_shoot_range(mut self, range: u32) -> Self {
        self.shoot_range = range;
        self
    }

    /// Builder-style setter for element type / damage.
    pub fn with_element(mut self, element_type: ElementType, element_damage: u32) -> Self {
        self.element_type = element_type;
        self.element_damage = element_damage;
        self
    }

    /// Builder-style setter for minimum magic level.
    pub fn with_min_mag_level(mut self, mag_level: u32) -> Self {
        self.min_mag_level = mag_level;
        self
    }

    pub fn get_level_required(&self) -> u32 {
        self.min_level
    }

    pub fn meets_level(&self, player_level: u32) -> bool {
        player_level >= self.min_level
    }

    pub fn meets_magic_level(&self, player_mag_level: u32) -> bool {
        player_mag_level >= self.min_mag_level
    }

    /// Mirrors `Weapon::playerWeaponCheck`: returns the damage modifier (0 or 100).
    ///
    /// Returns 0 if:
    /// - weapon disabled
    /// - player level below `min_level` (and weapon is not wielded unproperly)
    /// - target outside `shoot_range` (Chebyshev distance, same Z)
    ///
    /// Returns 100 otherwise.
    pub fn player_weapon_check(
        &self,
        player_level: u32,
        player_x: i32,
        player_y: i32,
        target_x: i32,
        target_y: i32,
    ) -> i32 {
        if !self.enabled {
            return 0;
        }
        if player_level < self.min_level {
            return 0;
        }
        let dx = (player_x - target_x).unsigned_abs();
        let dy = (player_y - target_y).unsigned_abs();
        let chebyshev = dx.max(dy);
        if chebyshev > self.shoot_range {
            return 0;
        }
        100
    }

    /// Mirrors `Weapon::ammoCheck` (simplified, no mana/soul).
    ///
    /// Returns false if:
    /// - weapon disabled
    /// - player level below `min_level`
    /// - player magic level below `min_mag_level`
    pub fn ammo_check(&self, player_level: u32, player_mag_level: u32) -> bool {
        if !self.enabled {
            return false;
        }
        if player_level < self.min_level {
            return false;
        }
        if player_mag_level < self.min_mag_level {
            return false;
        }
        true
    }

    /// Element damage for melee weapons.
    /// Returns 0 when `element_type` is `ElementType::None`.
    pub fn get_element_damage(&self) -> i32 {
        if self.element_type == ElementType::None {
            0
        } else {
            self.element_damage as i32
        }
    }

    /// Apply damage modifier (mirrors `damage * damageModifier / 100`).
    pub fn apply_damage_modifier(damage: i32, modifier: i32) -> i32 {
        (damage * modifier) / 100
    }
}

// ---------------------------------------------------------------------------
// Wand
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Wand {
    pub item_id: u32,
    pub mana_per_use: u32,
    pub min_damage: u32,
    pub max_damage: u32,
    pub element: ElementType,
}

impl Wand {
    pub fn new(
        item_id: u32,
        mana_per_use: u32,
        min_damage: u32,
        max_damage: u32,
        element: ElementType,
    ) -> Self {
        Wand {
            item_id,
            mana_per_use,
            min_damage,
            max_damage,
            element,
        }
    }

    pub fn mana_per_use(&self) -> u32 {
        self.mana_per_use
    }

    /// Seed-based damage roll in `[min_damage, max_damage]`.
    pub fn roll_damage(&self, seed: u32) -> u32 {
        let range = self.max_damage - self.min_damage + 1;
        seed % range + self.min_damage
    }

    /// Returns `max_damage` when `max_damage_mode` is true (mirrors C++ `maxDamage` param).
    pub fn get_weapon_damage(&self, seed: u32, max_damage_mode: bool) -> u32 {
        if max_damage_mode {
            self.max_damage
        } else {
            self.roll_damage(seed)
        }
    }

    /// Wand element damage is always 0 (mirrors `WeaponWand::getElementDamage`).
    pub fn get_element_damage(&self) -> i32 {
        0
    }
}

// ---------------------------------------------------------------------------
// Ammo state (simplified model of ammo stack)
// ---------------------------------------------------------------------------

/// Simplified ammo item: an item stack that can be decremented (consumed).
#[derive(Debug, Clone)]
pub struct AmmoItem {
    pub item_id: u32,
    pub count: u32,
}

impl AmmoItem {
    pub fn new(item_id: u32, count: u32) -> Self {
        AmmoItem { item_id, count }
    }

    /// Consume one ammo. Returns `true` if successful (count > 0).
    pub fn consume_one(&mut self) -> bool {
        if self.count > 0 {
            self.count -= 1;
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Registry of weapons keyed by item id (game-logic layer).
#[derive(Debug, Default)]
pub struct Weapons {
    map: HashMap<u32, Weapon>,
}

impl Weapons {
    pub fn new() -> Self {
        Weapons {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, weapon: Weapon) {
        self.map.insert(weapon.item_id, weapon);
    }

    pub fn get_by_item_id(&self, id: u32) -> Option<&Weapon> {
        self.map.get(&id)
    }
}

// ---------------------------------------------------------------------------
// C++ parity helpers — mana/health/soul/break/vocation/hit-chance
// (mirrors weapons.cpp pure-formula portions of Weapon::* and WeaponDistance::*)
// ---------------------------------------------------------------------------

/// Mirrors `Weapon::getManaCost`: flat `mana` overrides; else `manaPercent` of
/// `max_mana`. Returns 0 when both are 0.
pub fn weapon_mana_cost(mana: u32, mana_percent: u32, max_mana: u32) -> u32 {
    if mana != 0 {
        return mana;
    }
    if mana_percent == 0 {
        return 0;
    }
    (max_mana * mana_percent) / 100
}

/// Mirrors `Weapon::getHealthCost`: flat `health` overrides; else `healthPercent`
/// of `max_health`. Returns 0 when both are 0.
pub fn weapon_health_cost(health: u32, health_percent: u32, max_health: u32) -> u32 {
    if health != 0 {
        return health;
    }
    if health_percent == 0 {
        return 0;
    }
    (max_health * health_percent) / 100
}

/// Mirrors `Weapon::hasVocationWeaponSet`: returns true if the set is empty (no
/// vocation restriction) or contains the player's vocation id.
pub fn has_vocation_weapon_set(vocation_set: &[u16], vocation_id: u16) -> bool {
    vocation_set.is_empty() || vocation_set.contains(&vocation_id)
}

/// Mirrors break-chance roll in `Weapon::onUsedWeapon`:
/// returns true (weapon breaks) when `roll` in `1..=100` is `<= break_chance`.
/// `break_chance == 0` never breaks.
pub fn weapon_breaks(break_chance: u8, roll: u8) -> bool {
    if break_chance == 0 {
        return false;
    }
    roll >= 1 && roll <= break_chance
}

/// Outcome of `decrement_item_count` (mirrors `Weapon::decrementItemCount`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecrementItemOutcome {
    /// Stack reduced to `new_count` (>= 1).
    Transformed { new_count: u16 },
    /// Stack removed entirely (count was 1).
    Removed,
}

/// Mirrors `Weapon::decrementItemCount`: if count > 1, transform to `count - 1`;
/// otherwise mark for removal.
pub fn decrement_item_count(current_count: u16) -> DecrementItemOutcome {
    if current_count > 1 {
        DecrementItemOutcome::Transformed {
            new_count: current_count - 1,
        }
    } else {
        DecrementItemOutcome::Removed
    }
}

/// Mirrors the WEAPON_AMMO / WEAPON_DISTANCE branch in `Weapon::internalUseWeapon`
/// for selecting the damage origin tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponOrigin {
    Melee,
    Ranged,
    Wand,
}

/// Mirrors the origin selection in `Weapon::internalUseWeapon`.
pub fn weapon_damage_origin(kind: WeaponKind) -> WeaponOrigin {
    match kind {
        WeaponKind::Distance => WeaponOrigin::Ranged,
        WeaponKind::Wand => WeaponOrigin::Wand,
        WeaponKind::Melee => WeaponOrigin::Melee,
    }
}

/// Mirrors `WeaponDistance::useWeapon` hit-chance formulas. Returns the chance
/// in `[0, 100]` of a successful hit. `max_hit_chance` selects which family of
/// curves (75 / 90 / 100). For `distance` values not in the table, returns
/// `default_hit_chance` (the `it.hitChance` fallback).
pub fn distance_hit_chance(
    skill: u32,
    distance: u32,
    max_hit_chance: u32,
    default_hit_chance: i32,
) -> i32 {
    match max_hit_chance {
        75 => match distance {
            1 | 5 => (skill.min(74) + 1) as i32,
            2 => ((skill.min(28) as f32 * 2.40_f32) as u32 + 8) as i32,
            3 => ((skill.min(45) as f32 * 1.55_f32) as u32 + 6) as i32,
            4 => ((skill.min(58) as f32 * 1.25_f32) as u32 + 3) as i32,
            6 => ((skill.min(90) as f32 * 0.80_f32) as u32 + 3) as i32,
            7 => ((skill.min(104) as f32 * 0.70_f32) as u32 + 2) as i32,
            _ => default_hit_chance,
        },
        90 => match distance {
            1 | 5 => ((skill.min(74) as f32 * 1.20_f32) as u32 + 1) as i32,
            2 => (skill.min(28) as f32 * 3.20_f32) as i32,
            3 => (skill.min(45) * 2) as i32,
            4 => (skill.min(58) as f32 * 1.55_f32) as i32,
            6 | 7 => skill.min(90) as i32,
            _ => default_hit_chance,
        },
        100 => match distance {
            1 | 5 => ((skill.min(73) as f32 * 1.35_f32) as u32 + 1) as i32,
            2 => ((skill.min(30) as f32 * 3.20_f32) as u32 + 4) as i32,
            3 => ((skill.min(48) as f32 * 2.05_f32) as u32 + 2) as i32,
            4 => ((skill.min(65) as f32 * 1.50_f32) as u32 + 2) as i32,
            6 => ((skill.min(87) as f32 * 1.20_f32) as i32) - 4,
            7 => ((skill.min(90) as f32 * 1.10_f32) as u32 + 1) as i32,
            _ => default_hit_chance,
        },
        // Any other custom max_hit_chance is used directly (mirrors `chance = maxHitChance`).
        other => other as i32,
    }
}

/// Mirrors the `WeaponDistance::useWeapon` two-handed `maxHitChance` selection
/// when `it.maxHitChance == -1`:
///   - `AMMO_NONE` ⇒ 75 (one-handed)
///   - non-`AMMO_NONE` (e.g. bow) ⇒ 90 (two-handed)
pub fn default_max_hit_chance(has_ammo_type: bool, explicit_max: Option<u32>) -> u32 {
    if let Some(v) = explicit_max {
        return v;
    }
    if has_ammo_type {
        90
    } else {
        75
    }
}

/// Mirrors `Weapon::useFist` range check: fist range is exactly 1 (Chebyshev).
pub fn fist_in_range(player_x: i32, player_y: i32, target_x: i32, target_y: i32) -> bool {
    let dx = (player_x - target_x).unsigned_abs();
    let dy = (player_y - target_y).unsigned_abs();
    dx.max(dy) <= 1
}

/// Mirrors the "wielded unproperly" branch in `Weapon::playerWeaponCheck`:
/// when level/mag-level requirement is not met:
///   - if `wielded_unproperly` ⇒ damage_modifier / 2
///   - else                    ⇒ 0
pub fn wield_unproperly_modifier(damage_modifier: i32, wielded_unproperly: bool) -> i32 {
    if wielded_unproperly {
        damage_modifier / 2
    } else {
        0
    }
}

/// Full mirror of `Weapon::playerWeaponCheck` covering every C++ branch:
///   z mismatch, range, IgnoreWeaponCheck flag, disabled, mana, health, soul,
///   premium, vocation, level (with unproperly), mag-level (with unproperly).
#[allow(clippy::too_many_arguments)]
pub fn player_weapon_check_full(
    weapon: &Weapon,
    player_level: u32,
    player_mag_level: u32,
    player_mana: u32,
    player_health: u32,
    player_soul: u32,
    player_is_premium: bool,
    player_vocation_id: u16,
    ignore_weapon_check: bool,
    player_z: i32,
    target_z: i32,
    player_x: i32,
    player_y: i32,
    target_x: i32,
    target_y: i32,
    required_mana: u32,
    required_health: u32,
    required_soul: u32,
    requires_premium: bool,
    vocation_set: &[u16],
    wielded_unproperly: bool,
) -> i32 {
    // z mismatch (different floors)
    if player_z != target_z {
        return 0;
    }
    // out of range
    let dx = (player_x - target_x).unsigned_abs();
    let dy = (player_y - target_y).unsigned_abs();
    if dx.max(dy) > weapon.shoot_range {
        return 0;
    }
    // flag short-circuit
    if ignore_weapon_check {
        return 100;
    }
    if !weapon.enabled {
        return 0;
    }
    if player_mana < required_mana {
        return 0;
    }
    if player_health < required_health {
        return 0;
    }
    if player_soul < required_soul {
        return 0;
    }
    if requires_premium && !player_is_premium {
        return 0;
    }
    if !has_vocation_weapon_set(vocation_set, player_vocation_id) {
        return 0;
    }
    let mut damage_modifier: i32 = 100;
    if player_level < weapon.min_level {
        damage_modifier = wield_unproperly_modifier(damage_modifier, wielded_unproperly);
    }
    if player_mag_level < weapon.min_mag_level {
        damage_modifier = wield_unproperly_modifier(damage_modifier, wielded_unproperly);
    }
    damage_modifier
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Existing tests (unchanged)
    // -----------------------------------------------------------------------

    // 1. WeaponKind enum
    #[test]
    fn weapon_kind_variants_exist() {
        let _ = WeaponKind::Melee;
        let _ = WeaponKind::Distance;
        let _ = WeaponKind::Wand;
    }

    // 2a. Melee damage formula is deterministic
    #[test]
    fn melee_compute_damage_deterministic() {
        // attack=30, skill_level=50, seed=0
        // base = 30 * 50 / 20 = 75; rng = 0 % (10+1) = 0; result = 75
        let result = WeaponMelee::compute_damage(30, 50, 0);
        assert_eq!(result, 75);
    }

    // 2b. Distance formula matches melee
    #[test]
    fn distance_compute_damage_matches_melee() {
        let m = WeaponMelee::compute_damage(30, 50, 0);
        let d = WeaponDistance::compute_damage(30, 50, 0);
        assert_eq!(m, d);
    }

    // 3a. Weapon level accessors
    #[test]
    fn weapon_get_level_required() {
        let w = Weapon::new(100, WeaponKind::Melee, 20, 30, 10);
        assert_eq!(w.get_level_required(), 20);
    }

    // 3b. meets_level true when player level high enough
    #[test]
    fn weapon_meets_level_true() {
        let w = Weapon::new(100, WeaponKind::Melee, 20, 30, 10);
        assert!(w.meets_level(20));
        assert!(w.meets_level(50));
    }

    // 3c. meets_level false when level too low
    #[test]
    fn weapon_meets_level_false() {
        let w = Weapon::new(100, WeaponKind::Melee, 20, 30, 10);
        assert!(!w.meets_level(19));
    }

    // 4a. Wand mana_per_use
    #[test]
    fn wand_mana_per_use() {
        let wand = Wand::new(200, 13, 10, 30, ElementType::Fire);
        assert_eq!(wand.mana_per_use(), 13);
    }

    // 4b. Wand roll_damage in range
    #[test]
    fn wand_roll_damage_in_range() {
        let wand = Wand::new(200, 13, 10, 30, ElementType::Fire);
        let dmg = wand.roll_damage(0);
        assert!(
            (10..=30).contains(&dmg),
            "damage {dmg} out of range [10,30]"
        );
    }

    // 4c. Wand roll_damage is deterministic with seed
    #[test]
    fn wand_roll_damage_deterministic() {
        let wand = Wand::new(200, 13, 10, 30, ElementType::Fire);
        // seed=0: 0 % 21 + 10 = 10
        assert_eq!(wand.roll_damage(0), 10);
        // seed=20: 20 % 21 + 10 = 30
        assert_eq!(wand.roll_damage(20), 30);
    }

    // 5a. Weapons registry new is empty
    #[test]
    fn weapons_registry_new_is_empty() {
        let weapons = Weapons::new();
        assert!(weapons.get_by_item_id(1).is_none());
    }

    // 5b. Register and get_by_item_id
    #[test]
    fn weapons_registry_register_and_get() {
        let mut weapons = Weapons::new();
        let w = Weapon::new(42, WeaponKind::Melee, 0, 20, 10);
        weapons.register(w);
        assert!(weapons.get_by_item_id(42).is_some());
    }

    // 5c. get_by_item_id returns None for unknown
    #[test]
    fn weapons_registry_get_unknown_returns_none() {
        let weapons = Weapons::new();
        assert!(weapons.get_by_item_id(999).is_none());
    }

    // -----------------------------------------------------------------------
    // New tests — gaps identified from C++ source
    // -----------------------------------------------------------------------

    // 6. get_max_melee_damage (monster formula)
    // ceil(attackSkill * attackValue * 0.05 + attackValue * 0.5)
    // skill=50, value=30 => ceil(50*30*0.05 + 30*0.5) = ceil(75 + 15) = 90
    #[test]
    fn get_max_melee_damage_formula() {
        let result = get_max_melee_damage(50, 30);
        assert_eq!(result, 90);
    }

    // Edge: attackValue=0 => 0
    #[test]
    fn get_max_melee_damage_zero_attack() {
        assert_eq!(get_max_melee_damage(50, 0), 0);
    }

    // 7. get_max_weapon_damage (player formula)
    // level=100, skill=50, value=30, factor=1.0
    // inner = ((50/4+1) * (30/3)) * 1.03 / 1.0
    //       = (13.5 * 10) * 1.03 = 139.05
    // result = round(100/5 + 139.05) = round(20 + 139.05) = round(159.05) = 159
    #[test]
    fn get_max_weapon_damage_formula() {
        let result = get_max_weapon_damage(100, 50, 30, 1.0);
        assert_eq!(result, 159);
    }

    // Higher attackFactor reduces damage
    #[test]
    fn get_max_weapon_damage_higher_factor_reduces_damage() {
        let low_factor = get_max_weapon_damage(100, 50, 30, 1.0);
        let high_factor = get_max_weapon_damage(100, 50, 30, 2.0);
        assert!(
            low_factor > high_factor,
            "low_factor={low_factor} should be > high_factor={high_factor}"
        );
    }

    // Proportional to skill level: higher skill → higher damage
    #[test]
    fn weapon_melee_damage_proportional_to_skill() {
        let low_skill = WeaponMelee::compute_damage(30, 10, 0);
        let high_skill = WeaponMelee::compute_damage(30, 50, 0);
        assert!(
            high_skill > low_skill,
            "high_skill={high_skill} should be > low_skill={low_skill}"
        );
    }

    // 8. Distance range check
    #[test]
    fn weapon_distance_range_check_within_range() {
        // player at (0,0), target at (5,0), range=5 → ok
        assert!(WeaponDistance::check_range(0, 0, 5, 0, 5));
    }

    #[test]
    fn weapon_distance_range_check_exactly_at_range() {
        assert!(WeaponDistance::check_range(0, 0, 5, 5, 5)); // diagonal: max(5,5)=5
    }

    #[test]
    fn weapon_distance_range_check_outside_range() {
        // player at (0,0), target at (6,0), range=5 → fail
        assert!(!WeaponDistance::check_range(0, 0, 6, 0, 5));
    }

    #[test]
    fn weapon_distance_range_check_zero_range_adjacent() {
        // range=0 means only same tile
        assert!(WeaponDistance::check_range(3, 3, 3, 3, 0));
        assert!(!WeaponDistance::check_range(3, 3, 4, 3, 0));
    }

    // 9. Distance element damage
    #[test]
    fn weapon_distance_element_damage_no_element_returns_zero() {
        let dmg = WeaponDistance::get_element_damage(ElementType::None, 50);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn weapon_distance_element_damage_with_element_returns_value() {
        let dmg = WeaponDistance::get_element_damage(ElementType::Fire, 42);
        assert_eq!(dmg, 42);
    }

    // 10. Ammo consumption
    #[test]
    fn weapon_distance_ammo_consumption_reduces_count_by_one() {
        let mut ammo = AmmoItem::new(1001, 10);
        let consumed = ammo.consume_one();
        assert!(consumed);
        assert_eq!(ammo.count, 9);
    }

    #[test]
    fn weapon_distance_ammo_consumption_fails_when_empty() {
        let mut ammo = AmmoItem::new(1001, 0);
        let consumed = ammo.consume_one();
        assert!(!consumed);
        assert!(ammo.is_empty());
    }

    #[test]
    fn weapon_distance_ammo_consumption_empties_stack() {
        let mut ammo = AmmoItem::new(1001, 1);
        ammo.consume_one();
        assert!(ammo.is_empty());
    }

    // 11. Wand damage in [min, max]
    #[test]
    fn wand_damage_stays_in_min_max_range() {
        let wand = Wand::new(300, 10, 5, 20, ElementType::Energy);
        for seed in 0..100u32 {
            let dmg = wand.roll_damage(seed);
            assert!(
                dmg >= wand.min_damage && dmg <= wand.max_damage,
                "seed={seed}: damage {dmg} outside [{},{}]",
                wand.min_damage,
                wand.max_damage
            );
        }
    }

    // 12. Wand get_weapon_damage with max_damage_mode
    #[test]
    fn wand_get_weapon_damage_max_mode_returns_max() {
        let wand = Wand::new(300, 10, 5, 20, ElementType::Ice);
        assert_eq!(wand.get_weapon_damage(0, true), 20);
        assert_eq!(wand.get_weapon_damage(999, true), 20);
    }

    #[test]
    fn wand_get_weapon_damage_normal_mode_uses_seed() {
        let wand = Wand::new(300, 10, 5, 20, ElementType::Ice);
        // normal mode delegates to roll_damage
        assert_eq!(wand.get_weapon_damage(0, false), wand.roll_damage(0));
    }

    // 13. Wand element damage always 0
    #[test]
    fn wand_get_element_damage_always_zero() {
        let wand_fire = Wand::new(300, 10, 5, 20, ElementType::Fire);
        assert_eq!(wand_fire.get_element_damage(), 0);
        let wand_ice = Wand::new(301, 10, 5, 20, ElementType::Ice);
        assert_eq!(wand_ice.get_element_damage(), 0);
    }

    // 14. Melee weapon element damage
    #[test]
    fn melee_get_element_damage_no_element_returns_zero() {
        let w = Weapon::new(100, WeaponKind::Melee, 0, 30, 10);
        // default element_type is None
        assert_eq!(w.get_element_damage(), 0);
    }

    #[test]
    fn melee_get_element_damage_with_element_returns_value() {
        let w = Weapon::new(100, WeaponKind::Melee, 0, 30, 10).with_element(ElementType::Fire, 15);
        assert_eq!(w.get_element_damage(), 15);
    }

    // 15. player_weapon_check — range
    #[test]
    fn player_weapon_check_within_range_returns_100() {
        let w = Weapon::new(100, WeaponKind::Melee, 0, 30, 10).with_shoot_range(1);
        assert_eq!(w.player_weapon_check(10, 0, 0, 1, 0), 100);
    }

    #[test]
    fn player_weapon_check_outside_range_returns_zero() {
        let w = Weapon::new(100, WeaponKind::Distance, 0, 30, 0).with_shoot_range(5);
        assert_eq!(w.player_weapon_check(10, 0, 0, 10, 0), 0);
    }

    #[test]
    fn player_weapon_check_level_too_low_returns_zero() {
        let w = Weapon::new(100, WeaponKind::Melee, 50, 30, 10).with_shoot_range(1);
        assert_eq!(w.player_weapon_check(30, 0, 0, 1, 0), 0);
    }

    #[test]
    fn player_weapon_check_disabled_returns_zero() {
        let mut w = Weapon::new(100, WeaponKind::Melee, 0, 30, 10).with_shoot_range(1);
        w.enabled = false;
        assert_eq!(w.player_weapon_check(100, 0, 0, 1, 0), 0);
    }

    // 16. ammo_check
    #[test]
    fn ammo_check_passes_when_all_conditions_met() {
        let w = Weapon::new(100, WeaponKind::Distance, 20, 30, 0).with_min_mag_level(5);
        assert!(w.ammo_check(20, 5));
        assert!(w.ammo_check(50, 10));
    }

    #[test]
    fn ammo_check_fails_when_level_too_low() {
        let w = Weapon::new(100, WeaponKind::Distance, 20, 30, 0);
        assert!(!w.ammo_check(19, 0));
    }

    #[test]
    fn ammo_check_fails_when_mag_level_too_low() {
        let w = Weapon::new(100, WeaponKind::Distance, 0, 30, 0).with_min_mag_level(10);
        assert!(!w.ammo_check(100, 9));
    }

    #[test]
    fn ammo_check_fails_when_disabled() {
        let mut w = Weapon::new(100, WeaponKind::Distance, 0, 30, 0);
        w.enabled = false;
        assert!(!w.ammo_check(100, 100));
    }

    // 17. apply_damage_modifier
    #[test]
    fn apply_damage_modifier_full_modifier() {
        // modifier=100 → no change
        assert_eq!(Weapon::apply_damage_modifier(50, 100), 50);
    }

    #[test]
    fn apply_damage_modifier_half_modifier() {
        // modifier=50 → half
        assert_eq!(Weapon::apply_damage_modifier(100, 50), 50);
    }

    #[test]
    fn apply_damage_modifier_zero_modifier() {
        // modifier=0 → 0
        assert_eq!(Weapon::apply_damage_modifier(999, 0), 0);
    }

    // 18. meets_magic_level
    #[test]
    fn meets_magic_level_true() {
        let w = Weapon::new(100, WeaponKind::Wand, 0, 0, 0).with_min_mag_level(10);
        assert!(w.meets_magic_level(10));
        assert!(w.meets_magic_level(20));
    }

    #[test]
    fn meets_magic_level_false() {
        let w = Weapon::new(100, WeaponKind::Wand, 0, 0, 0).with_min_mag_level(10);
        assert!(!w.meets_magic_level(9));
    }

    // 19. ElementType::None default
    #[test]
    fn element_type_none_variant_exists() {
        let _ = ElementType::None;
    }

    // -----------------------------------------------------------------------
    // C++ parity gap tests — mana/health/soul/break/vocation/hit-chance
    // -----------------------------------------------------------------------

    // 20. weapon_mana_cost — flat overrides
    #[test]
    fn weapon_mana_cost_flat_overrides_percent() {
        assert_eq!(weapon_mana_cost(50, 99, 1000), 50);
    }

    #[test]
    fn weapon_mana_cost_percent_when_flat_zero() {
        // 25% of 200 = 50
        assert_eq!(weapon_mana_cost(0, 25, 200), 50);
    }

    #[test]
    fn weapon_mana_cost_zero_when_both_zero() {
        assert_eq!(weapon_mana_cost(0, 0, 1000), 0);
    }

    // 21. weapon_health_cost
    #[test]
    fn weapon_health_cost_flat_overrides_percent() {
        assert_eq!(weapon_health_cost(10, 99, 1000), 10);
    }

    #[test]
    fn weapon_health_cost_percent_when_flat_zero() {
        // 10% of 500 = 50
        assert_eq!(weapon_health_cost(0, 10, 500), 50);
    }

    #[test]
    fn weapon_health_cost_zero_when_both_zero() {
        assert_eq!(weapon_health_cost(0, 0, 1000), 0);
    }

    // 22. has_vocation_weapon_set
    #[test]
    fn has_vocation_weapon_set_empty_means_any_vocation() {
        assert!(has_vocation_weapon_set(&[], 1));
        assert!(has_vocation_weapon_set(&[], 42));
    }

    #[test]
    fn has_vocation_weapon_set_includes_known_vocation() {
        assert!(has_vocation_weapon_set(&[1, 4, 7], 4));
    }

    #[test]
    fn has_vocation_weapon_set_excludes_unknown_vocation() {
        assert!(!has_vocation_weapon_set(&[1, 4, 7], 2));
    }

    // 23. weapon_breaks (break chance roll)
    #[test]
    fn weapon_breaks_zero_chance_never_breaks() {
        for r in 1..=100u8 {
            assert!(!weapon_breaks(0, r));
        }
    }

    #[test]
    fn weapon_breaks_roll_within_chance() {
        // break_chance=10, roll=5 → break
        assert!(weapon_breaks(10, 5));
        // break_chance=10, roll=10 → break (inclusive)
        assert!(weapon_breaks(10, 10));
    }

    #[test]
    fn weapon_breaks_roll_above_chance_does_not_break() {
        assert!(!weapon_breaks(10, 11));
        assert!(!weapon_breaks(10, 100));
    }

    #[test]
    fn weapon_breaks_full_chance_always_breaks_when_roll_valid() {
        for r in 1..=100u8 {
            assert!(weapon_breaks(100, r));
        }
    }

    // 24. decrement_item_count
    #[test]
    fn decrement_item_count_transforms_when_stack_above_one() {
        assert_eq!(
            decrement_item_count(5),
            DecrementItemOutcome::Transformed { new_count: 4 }
        );
    }

    #[test]
    fn decrement_item_count_removes_when_stack_is_one() {
        assert_eq!(decrement_item_count(1), DecrementItemOutcome::Removed);
    }

    #[test]
    fn decrement_item_count_removes_when_stack_is_zero() {
        // Defensive: <=1 path triggers removal (mirrors `else` branch).
        assert_eq!(decrement_item_count(0), DecrementItemOutcome::Removed);
    }

    // 25. weapon_damage_origin
    #[test]
    fn weapon_damage_origin_melee_returns_melee() {
        assert_eq!(weapon_damage_origin(WeaponKind::Melee), WeaponOrigin::Melee);
    }

    #[test]
    fn weapon_damage_origin_distance_returns_ranged() {
        assert_eq!(
            weapon_damage_origin(WeaponKind::Distance),
            WeaponOrigin::Ranged
        );
    }

    #[test]
    fn weapon_damage_origin_wand_returns_wand() {
        assert_eq!(weapon_damage_origin(WeaponKind::Wand), WeaponOrigin::Wand);
    }

    // 26. distance_hit_chance — 75 family (one-handed)
    #[test]
    fn distance_hit_chance_75_distance_1_basic() {
        // dist=1, skill=10 → min(10,74)+1 = 11
        assert_eq!(distance_hit_chance(10, 1, 75, 0), 11);
    }

    #[test]
    fn distance_hit_chance_75_distance_5_same_as_1() {
        assert_eq!(
            distance_hit_chance(10, 5, 75, 0),
            distance_hit_chance(10, 1, 75, 0)
        );
    }

    #[test]
    fn distance_hit_chance_75_distance_2() {
        // skill=20 → (min(20,28) * 2.40 as u32) + 8 = (48u32) + 8 = 56
        assert_eq!(distance_hit_chance(20, 2, 75, 0), 56);
    }

    #[test]
    fn distance_hit_chance_75_distance_3() {
        // skill=40 → (min(40,45)*1.55 as u32) + 6 = 62 + 6 = 68
        assert_eq!(distance_hit_chance(40, 3, 75, 0), 68);
    }

    #[test]
    fn distance_hit_chance_75_distance_4() {
        // skill=50 → (min(50,58)*1.25 as u32) + 3 = 62 + 3 = 65
        assert_eq!(distance_hit_chance(50, 4, 75, 0), 65);
    }

    #[test]
    fn distance_hit_chance_75_distance_6() {
        // skill=80 → (min(80,90)*0.80 as u32) + 3 = 64 + 3 = 67
        assert_eq!(distance_hit_chance(80, 6, 75, 0), 67);
    }

    #[test]
    fn distance_hit_chance_75_distance_7() {
        // skill=100 → (min(100,104)*0.70 as u32) + 2 = 70 + 2 = 72
        assert_eq!(distance_hit_chance(100, 7, 75, 0), 72);
    }

    #[test]
    fn distance_hit_chance_75_distance_unknown_uses_default() {
        assert_eq!(distance_hit_chance(50, 99, 75, 42), 42);
    }

    // distance_hit_chance — 90 family (two-handed)
    #[test]
    fn distance_hit_chance_90_distance_1() {
        // skill=50 → (min(50,74)*1.20 as u32)+1 = 60+1 = 61
        assert_eq!(distance_hit_chance(50, 1, 90, 0), 61);
    }

    #[test]
    fn distance_hit_chance_90_distance_2() {
        // skill=10 → min(10,28)*3.20 as i32 = 32
        assert_eq!(distance_hit_chance(10, 2, 90, 0), 32);
    }

    #[test]
    fn distance_hit_chance_90_distance_3() {
        // skill=40 → min(40,45)*2 = 80
        assert_eq!(distance_hit_chance(40, 3, 90, 0), 80);
    }

    #[test]
    fn distance_hit_chance_90_distance_4() {
        // skill=50 → min(50,58)*1.55 as i32 = 77
        assert_eq!(distance_hit_chance(50, 4, 90, 0), 77);
    }

    #[test]
    fn distance_hit_chance_90_distance_6_and_7() {
        assert_eq!(distance_hit_chance(80, 6, 90, 0), 80);
        assert_eq!(distance_hit_chance(80, 7, 90, 0), 80);
        // capped at 90
        assert_eq!(distance_hit_chance(200, 6, 90, 0), 90);
    }

    #[test]
    fn distance_hit_chance_90_distance_unknown_uses_default() {
        assert_eq!(distance_hit_chance(50, 99, 90, 7), 7);
    }

    // distance_hit_chance — 100 family (custom)
    #[test]
    fn distance_hit_chance_100_distance_1() {
        // skill=50 → (min(50,73)*1.35 as u32)+1 = 67+1 = 68
        assert_eq!(distance_hit_chance(50, 1, 100, 0), 68);
    }

    #[test]
    fn distance_hit_chance_100_distance_2() {
        // skill=20 → (min(20,30)*3.20 as u32)+4 = 64+4 = 68
        assert_eq!(distance_hit_chance(20, 2, 100, 0), 68);
    }

    #[test]
    fn distance_hit_chance_100_distance_3() {
        // skill=40 → (min(40,48)*2.05 as u32)+2 = 82+2 = 84
        assert_eq!(distance_hit_chance(40, 3, 100, 0), 84);
    }

    #[test]
    fn distance_hit_chance_100_distance_4() {
        // skill=50 → (min(50,65)*1.50 as u32)+2 = 75+2 = 77
        assert_eq!(distance_hit_chance(50, 4, 100, 0), 77);
    }

    #[test]
    fn distance_hit_chance_100_distance_6() {
        // skill=80 → (min(80,87)*1.20 as i32)-4 = 96-4 = 92
        assert_eq!(distance_hit_chance(80, 6, 100, 0), 92);
    }

    #[test]
    fn distance_hit_chance_100_distance_7() {
        // skill=80 → (min(80,90)*1.10 as u32)+1 = 88+1 = 89
        assert_eq!(distance_hit_chance(80, 7, 100, 0), 89);
    }

    #[test]
    fn distance_hit_chance_100_distance_unknown_uses_default() {
        assert_eq!(distance_hit_chance(50, 99, 100, 12), 12);
    }

    #[test]
    fn distance_hit_chance_custom_max_chance_used_directly() {
        // Any other max_hit_chance (e.g. 55) is used verbatim
        assert_eq!(distance_hit_chance(50, 3, 55, 0), 55);
    }

    // 27. default_max_hit_chance
    #[test]
    fn default_max_hit_chance_explicit_overrides() {
        assert_eq!(default_max_hit_chance(false, Some(60)), 60);
        assert_eq!(default_max_hit_chance(true, Some(80)), 80);
    }

    #[test]
    fn default_max_hit_chance_two_handed_when_has_ammo_type() {
        // it.ammoType != AMMO_NONE → 90
        assert_eq!(default_max_hit_chance(true, None), 90);
    }

    #[test]
    fn default_max_hit_chance_one_handed_otherwise() {
        // it.ammoType == AMMO_NONE → 75
        assert_eq!(default_max_hit_chance(false, None), 75);
    }

    // 28. fist_in_range — Chebyshev distance 1
    #[test]
    fn fist_in_range_same_tile() {
        assert!(fist_in_range(5, 5, 5, 5));
    }

    #[test]
    fn fist_in_range_adjacent() {
        assert!(fist_in_range(5, 5, 6, 5));
        assert!(fist_in_range(5, 5, 6, 6));
        assert!(fist_in_range(5, 5, 4, 4));
    }

    #[test]
    fn fist_not_in_range_when_too_far() {
        assert!(!fist_in_range(5, 5, 7, 5));
        assert!(!fist_in_range(5, 5, 5, 7));
    }

    // 29. wield_unproperly_modifier
    #[test]
    fn wield_unproperly_modifier_halves_when_unproperly() {
        assert_eq!(wield_unproperly_modifier(100, true), 50);
        assert_eq!(wield_unproperly_modifier(50, true), 25);
    }

    #[test]
    fn wield_unproperly_modifier_zero_when_strict() {
        assert_eq!(wield_unproperly_modifier(100, false), 0);
        assert_eq!(wield_unproperly_modifier(1, false), 0);
    }

    // 30. player_weapon_check_full — every branch
    fn full_weapon() -> Weapon {
        Weapon::new(100, WeaponKind::Distance, 10, 30, 0)
            .with_shoot_range(5)
            .with_min_mag_level(0)
    }

    #[test]
    fn pwc_full_zero_when_z_mismatch() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            100,
            100,
            true,
            1,
            false,
            /* z */ 7,
            8,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_zero_when_out_of_range() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            10,
            0, // dx=10 > shoot_range=5
            0,
            0,
            0,
            false,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_100_when_ignore_weapon_check_flag_set() {
        let w = full_weapon();
        // disabled, but flag overrides
        let mut w = w;
        w.enabled = false;
        let r = player_weapon_check_full(
            &w,
            0,
            0,
            0,
            0,
            0,
            false,
            1,
            /* ignore */ true,
            7,
            7,
            0,
            0,
            1,
            0,
            999,
            999,
            999,
            true,
            &[2, 3],
            false,
        );
        assert_eq!(r, 100);
    }

    #[test]
    fn pwc_full_zero_when_disabled() {
        let mut w = full_weapon();
        w.enabled = false;
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_zero_when_mana_insufficient() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            /* mana */ 5,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            /* req_mana */ 10,
            0,
            0,
            false,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_zero_when_health_insufficient() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            /* hp */ 5,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            /* req_hp */ 10,
            0,
            false,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_zero_when_soul_insufficient() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            100,
            /* soul */ 0,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            /* req_soul */ 1,
            false,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_zero_when_premium_required_and_player_not_premium() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            100,
            100,
            /* premium */ false,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            /* req_premium */ true,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_zero_when_vocation_not_allowed() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            100,
            100,
            true,
            /* voc */ 1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[2, 3],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_strict_zero_when_level_too_low() {
        let w = full_weapon(); // min_level=10
        let r = player_weapon_check_full(
            &w,
            /* level */ 5,
            0,
            100,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[],
            /* unproperly */ false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_half_when_level_too_low_and_unproperly() {
        let w = full_weapon(); // min_level=10
        let r = player_weapon_check_full(
            &w,
            5,
            0,
            100,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[],
            /* unproperly */ true,
        );
        assert_eq!(r, 50);
    }

    #[test]
    fn pwc_full_strict_zero_when_mag_level_too_low() {
        let w = Weapon::new(100, WeaponKind::Distance, 0, 30, 0)
            .with_shoot_range(5)
            .with_min_mag_level(20);
        let r = player_weapon_check_full(
            &w,
            50,
            /* mag */ 5,
            100,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[],
            false,
        );
        assert_eq!(r, 0);
    }

    #[test]
    fn pwc_full_half_when_mag_level_too_low_and_unproperly() {
        let w = Weapon::new(100, WeaponKind::Distance, 0, 30, 0)
            .with_shoot_range(5)
            .with_min_mag_level(20);
        let r = player_weapon_check_full(
            &w,
            50,
            5,
            100,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[],
            true,
        );
        assert_eq!(r, 50);
    }

    #[test]
    fn pwc_full_100_when_all_requirements_met() {
        let w = full_weapon();
        let r = player_weapon_check_full(
            &w,
            50,
            0,
            100,
            100,
            100,
            true,
            1,
            false,
            7,
            7,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            false,
            &[1, 2],
            false,
        );
        assert_eq!(r, 100);
    }
}
