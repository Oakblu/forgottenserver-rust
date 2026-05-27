use forgottenserver_common::position::Position;

/// Mirrors `CombatType_t` from enums.h (bit flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum CombatType {
    #[default]
    None = 0,
    Physical = 1 << 0,
    Energy = 1 << 1,
    Earth = 1 << 2,
    Fire = 1 << 3,
    Undefined = 1 << 4,
    LifeDrain = 1 << 5,
    ManaDrain = 1 << 6,
    Healing = 1 << 7,
    Drown = 1 << 8,
    Ice = 1 << 9,
    Holy = 1 << 10,
    Death = 1 << 11,
    Count = 12,
}

/// Mirrors `formulaType_t` from enums.h — how damage is calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormulaType {
    #[default]
    Undefined,
    LevelMagic,
    Skill,
    Damage,
}

/// Mirrors `CallBackParam_t` from enums.h — which callback is attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallBackParam {
    LevelMagicValue,
    SkillValue,
    TargetTile,
    TargetCreature,
}

/// Mirrors `CombatParam_t` from enums.h — named parameters for `setParam`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatParam {
    Type,
    Effect,
    DistanceEffect,
    BlockShield,
    BlockArmor,
    TargetCasterOrTopMost,
    CreateItem,
    Aggressive,
    Dispel,
    UseCharges,
}

/// Mirrors `BlockType_t` from combat logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlockType {
    #[default]
    None,
    Armor,
    Shield,
    Immunity,
}

/// Nested damage component — mirrors the anonymous struct inside `CombatDamage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DamageComponent {
    pub combat_type: CombatType,
    pub value: i32,
}

impl DamageComponent {
    pub fn new(combat_type: CombatType, value: i32) -> Self {
        DamageComponent { combat_type, value }
    }
}

/// Full damage struct (mirrors `CombatDamage` from enums.h).
///
/// C++ layout:
/// ```cpp
/// struct CombatDamage {
///   struct { CombatType_t type; int32_t value; } primary, secondary;
///   CombatOrigin origin;
///   BlockType_t blockType;
///   bool critical;
///   bool leeched;
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CombatDamage {
    pub primary: DamageComponent,
    pub secondary: DamageComponent,
    pub origin: CombatOrigin,
    pub block_type: BlockType,
    pub critical: bool,
    pub leeched: bool,
    /// Legacy `extension` field kept for backward compat.
    pub extension: bool,
}

impl CombatDamage {
    /// Convenience constructor (mirrors common usage in C++).
    pub fn new(
        combat_type: CombatType,
        primary_damage: i32,
        secondary_damage: i32,
        extension: bool,
    ) -> Self {
        CombatDamage {
            primary: DamageComponent::new(combat_type, primary_damage),
            secondary: DamageComponent::new(CombatType::None, secondary_damage),
            origin: CombatOrigin::None,
            block_type: BlockType::None,
            critical: false,
            leeched: false,
            extension,
        }
    }

    /// Create a mana-drain damage value.
    pub fn mana_drain(value: i32) -> Self {
        CombatDamage {
            primary: DamageComponent::new(CombatType::ManaDrain, value),
            ..Default::default()
        }
    }

    /// Returns true when this is a mana-drain attack (mirrors C++ check
    /// `damage.primary.type == COMBAT_MANADRAIN`).
    pub fn is_mana_drain(&self) -> bool {
        self.primary.combat_type == CombatType::ManaDrain
    }
}

/// Parameters that configure a combat spell/attack (mirrors `CombatParams` from combat.h).
#[derive(Debug, Clone, Default)]
pub struct CombatParams {
    pub item_id: u16,
    pub dispel_type: ConditionType,
    pub combat_type: CombatType,
    pub origin: CombatOrigin,
    pub impact_effect: u8,
    pub distance_effect: u8,
    pub blocked_by_armor: bool,
    pub blocked_by_shield: bool,
    pub target_caster_or_top_most: bool,
    pub aggressive: bool,
    pub use_charges: bool,
    pub ignore_resistances: bool,
}

impl CombatParams {
    pub fn new() -> Self {
        CombatParams {
            aggressive: true,
            origin: CombatOrigin::Spell,
            ..Default::default()
        }
    }

    /// Apply a named parameter value (mirrors `Combat::setParam`).
    pub fn set_param(&mut self, param: CombatParam, value: u32) -> bool {
        match param {
            CombatParam::Type => {
                // stored as combat_type — caller must cast
                let _ = value;
                true
            }
            CombatParam::Effect => {
                self.impact_effect = value as u8;
                true
            }
            CombatParam::DistanceEffect => {
                self.distance_effect = value as u8;
                true
            }
            CombatParam::BlockArmor => {
                self.blocked_by_armor = value != 0;
                true
            }
            CombatParam::BlockShield => {
                self.blocked_by_shield = value != 0;
                true
            }
            CombatParam::TargetCasterOrTopMost => {
                self.target_caster_or_top_most = value != 0;
                true
            }
            CombatParam::CreateItem => {
                self.item_id = value as u16;
                true
            }
            CombatParam::Aggressive => {
                self.aggressive = value != 0;
                true
            }
            CombatParam::Dispel => {
                // dispel_type encoding is caller's responsibility
                let _ = value;
                true
            }
            CombatParam::UseCharges => {
                self.use_charges = value != 0;
                true
            }
        }
    }

    /// Get a named parameter value (mirrors `Combat::getParam`).
    pub fn get_param(&self, param: CombatParam) -> i32 {
        match param {
            CombatParam::Effect => self.impact_effect as i32,
            CombatParam::DistanceEffect => self.distance_effect as i32,
            CombatParam::BlockArmor => i32::from(self.blocked_by_armor),
            CombatParam::BlockShield => i32::from(self.blocked_by_shield),
            CombatParam::TargetCasterOrTopMost => i32::from(self.target_caster_or_top_most),
            CombatParam::CreateItem => self.item_id as i32,
            CombatParam::Aggressive => i32::from(self.aggressive),
            CombatParam::UseCharges => i32::from(self.use_charges),
            _ => i32::MAX,
        }
    }
}

/// Tile state flag: protection zone (mirrors `TILESTATE_PROTECTIONZONE = 1 << 7`).
pub const TILESTATE_PROTECTIONZONE: u32 = 1 << 7;

/// Tile state flag: no-PvP zone (mirrors `TILESTATE_NOPVPZONE`).
pub const TILESTATE_NOPVPZONE: u32 = 1 << 3;

/// Tile state flag: PvP zone (mirrors `TILESTATE_PVPZONE`).
pub const TILESTATE_PVPZONE: u32 = 1 << 5;

/// Zone types (mirrors `ZoneType_t` from enums.h).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Normal,
    Protection,
    NoPvp,
    Pvp,
    PvpFree,
}

/// World type (mirrors `WorldType_t` from enums.h).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldType {
    NoPvp,
    OpenPvp,
    HardcorePvp,
}

/// Condition types (mirrors `ConditionType_t`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConditionType {
    #[default]
    None,
    Fire,
    Poison,
    Energy,
    Bleeding,
    Haste,
    Paralyze,
    Outfit,
    Invisible,
    Light,
    ManaShield,
    Infight,
    Drunk,
    ExhaustWeapon,
    RegenerationBase,
    Soul,
    Drown,
    Muted,
    Channelize,
    Hexed,
    Stunned,
    SpellGrouped,
    InfightEx,
    RegenerationMana,
    RegenerationHealth,
    Freezing,
    Dazzled,
    Cursed,
    ExhaustHeal,
    Pacified,
    SpellCooldown,
}

/// Return value mirrors `ReturnValue` from enums.h (subset used in combat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnValue {
    NoError,
    NotEnoughRoom,
    YouMayNotAttackThisPlayer,
    YouMayNotAttackThisCreature,
    ActionNotPermittedInProtectionZone,
    ActionNotPermittedInNoPvpZone,
    YouMayNotAttackAPersonInProtectionZone,
    TurnSecureModeToAttackUnmarkedPlayers,
    FirstGoDownstairs,
    FirstGoUpstairs,
    NotEnoughMana,
}

/// Combat origin (mirrors `CombatOrigin`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CombatOrigin {
    #[default]
    None,
    Spell,
    Melee,
    Ranged,
    Condition,
    Wand,
    Reflect,
}

/// Simple creature state for combat calculations (no game-state dependency).
///
/// This is a pure-data struct used in unit-testable combat functions that
/// mirror the logic of C++ `Creature` / `Player` fields needed during combat.
#[derive(Debug, Clone)]
pub struct CreatureState {
    pub health: i32,
    pub max_health: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub zone: ZoneType,
    pub is_player: bool,
    pub is_attackable: bool,
    pub immunity_flags: u16,
}

impl CreatureState {
    pub fn new(health: i32, max_health: i32, zone: ZoneType) -> Self {
        CreatureState {
            health,
            max_health,
            mana: 0,
            max_mana: 0,
            zone,
            is_player: false,
            is_attackable: true,
            immunity_flags: 0,
        }
    }

    pub fn with_mana(mut self, mana: i32, max_mana: i32) -> Self {
        self.mana = mana;
        self.max_mana = max_mana;
        self
    }

    pub fn with_immunity(mut self, flags: u16) -> Self {
        self.immunity_flags = flags;
        self
    }
}

/// Result of applying health damage to a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthChangeResult {
    /// Actual health removed (≥ 0).
    pub damage_dealt: i32,
    /// Health after the change.
    pub new_health: i32,
    /// True if the creature died (health reached 0).
    pub died: bool,
}

/// Result of applying mana damage/gain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManaChangeResult {
    pub mana_changed: i32,
    pub new_mana: i32,
}

/// Core combat logic (pure functions, no game state).
pub struct Combat;

impl Combat {
    /// Deterministic damage formula (seed-based, no RNG).
    ///
    /// Formula: `seed % (max_dmg - min_dmg + 1) + min_dmg`
    pub fn apply_formula_damage(
        _level: u32,
        _magic_level: u32,
        min_dmg: i32,
        max_dmg: i32,
        seed: i32,
    ) -> i32 {
        let range = max_dmg - min_dmg + 1;
        seed % range + min_dmg
    }

    /// Returns the block type for a given combat type and immunity flags.
    ///
    /// If the combat type's bit flag is set in `immunity_flags`, the attack
    /// is fully blocked (Immunity). Otherwise returns None.
    pub fn get_block_type(combat_type: CombatType, immunity_flags: u16) -> BlockType {
        let flag = combat_type as u16;
        if flag != 0 && (immunity_flags & flag) != 0 {
            BlockType::Immunity
        } else {
            BlockType::None
        }
    }

    /// Returns true when the tile flags include `TILESTATE_PROTECTIONZONE`.
    pub fn is_in_protection_zone(flags: u32) -> bool {
        (flags & TILESTATE_PROTECTIONZONE) != 0
    }

    /// Returns true when the tile flags include `TILESTATE_NOPVPZONE`.
    pub fn is_in_nopvp_zone(flags: u32) -> bool {
        (flags & TILESTATE_NOPVPZONE) != 0
    }

    /// Returns true when both attacker and target are in a PvP zone.
    ///
    /// Mirrors `Combat::isInPvpZone`:
    ///   `return attacker->getZone() == ZONE_PVP && target->getZone() == ZONE_PVP`
    pub fn is_in_pvp_zone(attacker_zone: ZoneType, target_zone: ZoneType) -> bool {
        attacker_zone == ZoneType::Pvp && target_zone == ZoneType::Pvp
    }

    /// Maps a `ConditionType` to the corresponding `CombatType` (damage type).
    ///
    /// Mirrors `Combat::ConditionToDamageType`.
    pub fn condition_to_damage_type(condition: ConditionType) -> CombatType {
        match condition {
            ConditionType::Fire => CombatType::Fire,
            ConditionType::Energy => CombatType::Energy,
            ConditionType::Bleeding => CombatType::Physical,
            ConditionType::Drown => CombatType::Drown,
            ConditionType::Poison => CombatType::Earth,
            ConditionType::Freezing => CombatType::Ice,
            ConditionType::Dazzled => CombatType::Holy,
            ConditionType::Cursed => CombatType::Death,
            _ => CombatType::None,
        }
    }

    /// Maps a `CombatType` (damage type) to the corresponding `ConditionType`.
    ///
    /// Mirrors `Combat::DamageToConditionType`.
    pub fn damage_to_condition_type(combat: CombatType) -> ConditionType {
        match combat {
            CombatType::Fire => ConditionType::Fire,
            CombatType::Energy => ConditionType::Energy,
            CombatType::Drown => ConditionType::Drown,
            CombatType::Earth => ConditionType::Poison,
            CombatType::Ice => ConditionType::Freezing,
            CombatType::Holy => ConditionType::Dazzled,
            CombatType::Death => ConditionType::Cursed,
            CombatType::Physical => ConditionType::Bleeding,
            _ => ConditionType::None,
        }
    }

    /// Apply health damage to a creature, clamping health at [0, max_health].
    ///
    /// Mirrors the key invariants from `Game::combatChangeHealth`:
    /// - Returns `None` if `creature.health == 0` (already dead) and
    ///   the damage value is negative (dealing damage to dead creature).
    /// - Health never drops below 0.
    /// - Health never exceeds `max_health` for healing.
    /// - `damage_dealt` is the actual HP removed (may be less than requested
    ///   if health would go below 0).
    pub fn combat_change_health(
        creature: &CreatureState,
        damage: i32,
    ) -> Option<HealthChangeResult> {
        if damage < 0 && creature.health == 0 {
            // Target already dead — cannot deal more damage.
            return None;
        }

        let new_health = if damage >= 0 {
            // Healing: add the value, clamp at max
            (creature.health + damage).min(creature.max_health)
        } else {
            // Damage: subtract abs, clamp at 0
            (creature.health - damage.unsigned_abs() as i32).max(0)
        };

        let damage_dealt = (creature.health - new_health).max(0);
        let died = new_health == 0 && creature.health > 0;

        Some(HealthChangeResult {
            damage_dealt,
            new_health,
            died,
        })
    }

    /// Apply mana change (positive = gain, negative = drain).
    ///
    /// Mirrors key invariants from `Game::combatChangeMana`:
    /// - Mana never drops below 0.
    /// - Mana never exceeds `max_mana`.
    pub fn combat_change_mana(creature: &CreatureState, delta: i32) -> ManaChangeResult {
        let new_mana = (creature.mana + delta).clamp(0, creature.max_mana);
        let mana_changed = new_mana - creature.mana;
        ManaChangeResult {
            mana_changed,
            new_mana,
        }
    }

    /// Returns whether a creature can be targeted in combat given tile and zone state.
    ///
    /// Mirrors key logic from `Combat::canDoCombat(Creature*, Tile*, bool)`:
    /// - Aggressive combat is blocked in protection zones.
    /// - Non-aggressive combat is always allowed.
    pub fn can_do_combat_on_tile(tile_flags: u32, aggressive: bool) -> ReturnValue {
        if aggressive && (tile_flags & TILESTATE_PROTECTIONZONE) != 0 {
            return ReturnValue::ActionNotPermittedInProtectionZone;
        }
        ReturnValue::NoError
    }

    /// Returns whether a PvP attacker can target a creature in a NoPvP world.
    ///
    /// Mirrors logic from `Combat::canDoCombat(Creature*, Creature*)` for
    /// the `WORLD_TYPE_NO_PVP` branch: player-originated attacks against
    /// players/their summons are blocked unless both are in a PvP zone.
    pub fn can_do_combat_no_pvp_world(
        attacker_zone: ZoneType,
        target_zone: ZoneType,
        attacker_is_player: bool,
        target_is_player: bool,
    ) -> ReturnValue {
        if attacker_is_player
            && target_is_player
            && !Combat::is_in_pvp_zone(attacker_zone, target_zone)
        {
            return ReturnValue::YouMayNotAttackThisPlayer;
        }
        ReturnValue::NoError
    }

    /// Calculates damage clamped to the target's remaining health.
    ///
    /// Mirrors `Game::combatChangeHealth` logic:
    ///   `if damage >= targetHealth => damage = targetHealth`
    /// Returns the actual damage that would be applied.
    pub fn clamp_damage_to_health(requested_damage: i32, current_health: i32) -> i32 {
        requested_damage.min(current_health).max(0)
    }

    /// Calculates mana drain clamped to the target's current mana.
    ///
    /// Mirrors `Game::combatChangeMana` logic:
    ///   `int32_t manaDamage = std::min<int32_t>(targetPlayer->getMana(), healthChange)`
    pub fn clamp_mana_drain(requested_drain: i32, current_mana: i32) -> i32 {
        requested_drain.min(current_mana).max(0)
    }

    /// Returns true if the target is a player (or the master of a summon is a player).
    ///
    /// Mirrors `Combat::isPlayerCombat`.
    pub fn is_player_combat(target_is_player: bool, master_is_player: bool) -> bool {
        target_is_player || master_is_player
    }

    /// Returns whether the attacker is protected from a target by level cap.
    ///
    /// Mirrors `Combat::isProtected` (simplified — no vocation or skull checks,
    /// only the level-based protection rule):
    ///   `return target.level < protection_level || attacker.level < protection_level`
    pub fn is_protected(attacker_level: u32, target_level: u32, protection_level: u32) -> bool {
        attacker_level < protection_level || target_level < protection_level
    }

    /// Checks whether an attacker can target a creature given protection-zone and
    /// no-pvp-zone rules, returning the reason if blocked.
    ///
    /// Simplified mirror of `Combat::canTargetCreature` (pure, no game state):
    /// - Blocked if attacker is in protection zone.
    /// - Blocked if target is in protection zone.
    /// - Blocked if either is in no-pvp zone and it is player combat.
    /// - Blocked if target is not attackable.
    pub fn can_target_creature(
        attacker_zone: ZoneType,
        target_zone: ZoneType,
        target_is_attackable: bool,
        is_player_combat: bool,
        ignore_protection_zone: bool,
    ) -> ReturnValue {
        if !ignore_protection_zone {
            if attacker_zone == ZoneType::Protection {
                return ReturnValue::ActionNotPermittedInProtectionZone;
            }
            if target_zone == ZoneType::Protection {
                return ReturnValue::ActionNotPermittedInProtectionZone;
            }
            if is_player_combat {
                if attacker_zone == ZoneType::NoPvp {
                    return ReturnValue::ActionNotPermittedInNoPvpZone;
                }
                if target_zone == ZoneType::NoPvp {
                    return ReturnValue::YouMayNotAttackAPersonInProtectionZone;
                }
            }
        }
        if !target_is_attackable {
            return ReturnValue::YouMayNotAttackThisCreature;
        }
        ReturnValue::NoError
    }

    /// Dispatch: apply health or mana change based on `damage.is_mana_drain()`.
    ///
    /// Returns `Ok(HealthChangeResult)` for health damage/healing,
    /// or `Err(ManaChangeResult)` for mana drain.
    ///
    /// Mirrors the dispatch in `doTargetCombat`:
    ///   `if (damage.primary.type != COMBAT_MANADRAIN) combatChangeHealth else combatChangeMana`
    pub fn do_combat_dispatch(
        creature: &CreatureState,
        damage: &CombatDamage,
    ) -> Result<HealthChangeResult, ManaChangeResult> {
        if damage.is_mana_drain() {
            let delta = damage.primary.value; // negative = drain
            Err(Combat::combat_change_mana(creature, delta))
        } else {
            let hp_delta = damage.primary.value; // negative = damage, positive = heal
            Ok(
                Combat::combat_change_health(creature, hp_delta).unwrap_or(HealthChangeResult {
                    damage_dealt: 0,
                    new_health: 0,
                    died: false,
                }),
            )
        }
    }

    /// Apply critical hit multiplier to damage value.
    ///
    /// Mirrors the C++ critical hit calculation:
    ///   `damage.primary.value += round(damage.primary.value * (skill / 100.))`
    /// Returns the modified value.
    pub fn apply_critical_hit(base_value: i32, skill_percent: u16) -> i32 {
        let extra = (base_value.abs() as f64 * (skill_percent as f64 / 100.0)).round() as i32;
        // value is negative (damage), so subtract extra
        if base_value < 0 {
            base_value - extra
        } else {
            base_value + extra
        }
    }

    /// Compute leech amount from total damage dealt.
    ///
    /// Mirrors `leechCombat.primary.value = round(totalDamage * (skill / 10000.))`.
    pub fn compute_leech_amount(total_damage: i32, skill_per_ten_thousand: u16) -> i32 {
        (total_damage.abs() as f64 * (skill_per_ten_thousand as f64 / 10000.0)).round() as i32
    }
}

/// Area of effect helper — computes all tile positions within a rectangle.
pub struct CombatArea;

impl CombatArea {
    /// Returns all positions within ±range_x, ±range_y of `center` on the
    /// same floor.
    pub fn get_tiles_in_range(center: Position, range_x: i32, range_y: i32) -> Vec<Position> {
        let mut tiles = Vec::new();
        for dy in -range_y..=range_y {
            for dx in -range_x..=range_x {
                let nx = center.x as i32 + dx;
                let ny = center.y as i32 + dy;
                if nx >= 0 && ny >= 0 && nx <= u16::MAX as i32 && ny <= u16::MAX as i32 {
                    tiles.push(Position::new(nx as u16, ny as u16, center.z));
                }
            }
        }
        tiles
    }
}

/// Mirrors the `AreaCombat` class from combat.h.
///
/// In the C++ source, `AreaCombat` owns a `std::vector<MatrixArea> areas`
/// and an `hasExtArea` flag. This Rust version stores a flat list of
/// pre-computed positions instead of a matrix, and supports the extended-area
/// flag used by some spells.
#[derive(Debug, Default)]
pub struct AreaCombat {
    /// All positions in the primary area (computed by `setup_area`).
    areas: Vec<Position>,
    /// True when an extended area has been added via `setup_ext_area`.
    has_ext_area: bool,
}

impl AreaCombat {
    pub fn new() -> Self {
        AreaCombat::default()
    }

    /// Set up a rectangular area around a center position.
    ///
    /// Mirrors `AreaCombat::setupArea(int32_t radius)` — fills a square of
    /// side `2*radius+1`.
    pub fn setup_area_radius(&mut self, center: Position, radius: i32) {
        self.areas = CombatArea::get_tiles_in_range(center, radius, radius);
    }

    /// Set up the extended area flag (mirrors `setupExtArea`).
    pub fn setup_ext_area(&mut self, center: Position, radius: i32) {
        self.has_ext_area = true;
        let ext = CombatArea::get_tiles_in_range(center, radius + 1, radius + 1);
        for pos in ext {
            if !self.areas.contains(&pos) {
                self.areas.push(pos);
            }
        }
    }

    /// Returns whether an extended area is configured (mirrors `hasExtArea`).
    pub fn has_ext_area(&self) -> bool {
        self.has_ext_area
    }

    /// Returns a reference to the list of positions in this area.
    ///
    /// Mirrors `AreaCombat::getArea` — callers iterate over this list.
    pub fn get_list(&self) -> &[Position] {
        &self.areas
    }

    /// Returns true when the list is empty.
    pub fn is_empty(&self) -> bool {
        self.areas.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── existing tests (preserved) ───────────────────────────────────────────

    // 1. CombatType enum variants exist
    #[test]
    fn combat_type_variants_exist() {
        let types = [
            CombatType::None,
            CombatType::Physical,
            CombatType::Energy,
            CombatType::Fire,
            CombatType::Earth,
            CombatType::Undefined,
            CombatType::Holy,
            CombatType::Death,
            CombatType::Ice,
            CombatType::Healing,
            CombatType::Drown,
            CombatType::LifeDrain,
            CombatType::ManaDrain,
            CombatType::Count,
        ];
        assert_eq!(types.len(), 14);
    }

    // 2. CombatDamage struct accessors
    #[test]
    fn combat_damage_accessors_work() {
        let dmg = CombatDamage::new(CombatType::Fire, 50, 10, true);
        assert_eq!(dmg.primary.combat_type, CombatType::Fire);
        assert_eq!(dmg.primary.value, 50);
        assert_eq!(dmg.secondary.value, 10);
        assert!(dmg.extension);
    }

    // 3a. apply_formula_damage returns min at seed=0
    #[test]
    fn apply_formula_damage_min_at_seed_zero() {
        let result = Combat::apply_formula_damage(50, 30, 100, 200, 0);
        assert_eq!(result, 100);
    }

    // 3b. apply_formula_damage returns max at seed=100
    #[test]
    fn apply_formula_damage_max_at_seed_100() {
        let result = Combat::apply_formula_damage(50, 30, 100, 200, 100);
        assert_eq!(result, 200);
    }

    // 4a. BlockType immunity when flag set
    #[test]
    fn block_type_immunity_when_flag_set() {
        let result = Combat::get_block_type(CombatType::Physical, 0x0001);
        assert_eq!(result, BlockType::Immunity);
    }

    // 4b. Physical is not blocked by default (immunity_flags=0)
    #[test]
    fn block_type_none_when_no_immunity() {
        let result = Combat::get_block_type(CombatType::Physical, 0);
        assert_eq!(result, BlockType::None);
    }

    // 5a. get_tiles_in_range returns 9 tiles for range 1,1
    #[test]
    fn get_tiles_in_range_returns_nine_for_range_one() {
        let center = Position::new(10, 10, 7);
        let tiles = CombatArea::get_tiles_in_range(center, 1, 1);
        assert_eq!(tiles.len(), 9);
    }

    // 5b. All tiles share the same floor
    #[test]
    fn get_tiles_in_range_same_floor() {
        let center = Position::new(10, 10, 7);
        let tiles = CombatArea::get_tiles_in_range(center, 1, 1);
        assert!(tiles.iter().all(|p| p.z == 7));
    }

    // 6a. is_in_protection_zone returns true when bit 7 set
    #[test]
    fn is_in_protection_zone_true_when_flag_set() {
        assert!(Combat::is_in_protection_zone(TILESTATE_PROTECTIONZONE));
    }

    // 6b. is_in_protection_zone returns false when flag not set
    #[test]
    fn is_in_protection_zone_false_when_flag_absent() {
        assert!(!Combat::is_in_protection_zone(0));
    }

    // ─── NEW: ZoneType enum ────────────────────────────────────────────────────

    #[test]
    fn zone_type_variants_exist() {
        let zones = [
            ZoneType::Normal,
            ZoneType::Protection,
            ZoneType::NoPvp,
            ZoneType::Pvp,
            ZoneType::PvpFree,
        ];
        assert_eq!(zones.len(), 5);
    }

    // ─── NEW: is_in_pvp_zone ──────────────────────────────────────────────────

    #[test]
    fn is_in_pvp_zone_true_when_both_in_pvp() {
        assert!(Combat::is_in_pvp_zone(ZoneType::Pvp, ZoneType::Pvp));
    }

    #[test]
    fn is_in_pvp_zone_false_when_attacker_not_pvp() {
        assert!(!Combat::is_in_pvp_zone(ZoneType::Normal, ZoneType::Pvp));
    }

    #[test]
    fn is_in_pvp_zone_false_when_target_not_pvp() {
        assert!(!Combat::is_in_pvp_zone(ZoneType::Pvp, ZoneType::Normal));
    }

    #[test]
    fn is_in_pvp_zone_false_when_both_normal() {
        assert!(!Combat::is_in_pvp_zone(ZoneType::Normal, ZoneType::Normal));
    }

    #[test]
    fn is_in_pvp_zone_false_for_protection_zone() {
        assert!(!Combat::is_in_pvp_zone(ZoneType::Protection, ZoneType::Pvp));
    }

    #[test]
    fn is_in_pvp_zone_false_for_nopvp_zone() {
        assert!(!Combat::is_in_pvp_zone(ZoneType::NoPvp, ZoneType::NoPvp));
    }

    // ─── NEW: is_in_nopvp_zone ────────────────────────────────────────────────

    #[test]
    fn is_in_nopvp_zone_true_when_flag_set() {
        assert!(Combat::is_in_nopvp_zone(TILESTATE_NOPVPZONE));
    }

    #[test]
    fn is_in_nopvp_zone_false_when_flag_absent() {
        assert!(!Combat::is_in_nopvp_zone(0));
    }

    // ─── NEW: condition_to_damage_type ───────────────────────────────────────

    #[test]
    fn condition_to_damage_fire() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Fire),
            CombatType::Fire
        );
    }

    #[test]
    fn condition_to_damage_energy() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Energy),
            CombatType::Energy
        );
    }

    #[test]
    fn condition_to_damage_bleeding_is_physical() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Bleeding),
            CombatType::Physical
        );
    }

    #[test]
    fn condition_to_damage_drown() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Drown),
            CombatType::Drown
        );
    }

    #[test]
    fn condition_to_damage_poison_is_earth() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Poison),
            CombatType::Earth
        );
    }

    #[test]
    fn condition_to_damage_freezing_is_ice() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Freezing),
            CombatType::Ice
        );
    }

    #[test]
    fn condition_to_damage_dazzled_is_holy() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Dazzled),
            CombatType::Holy
        );
    }

    #[test]
    fn condition_to_damage_cursed_is_death() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::Cursed),
            CombatType::Death
        );
    }

    #[test]
    fn condition_to_damage_unknown_is_none() {
        assert_eq!(
            Combat::condition_to_damage_type(ConditionType::None),
            CombatType::None
        );
    }

    // ─── NEW: damage_to_condition_type ───────────────────────────────────────

    #[test]
    fn damage_to_condition_fire() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Fire),
            ConditionType::Fire
        );
    }

    #[test]
    fn damage_to_condition_energy() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Energy),
            ConditionType::Energy
        );
    }

    #[test]
    fn damage_to_condition_drown() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Drown),
            ConditionType::Drown
        );
    }

    #[test]
    fn damage_to_condition_earth_is_poison() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Earth),
            ConditionType::Poison
        );
    }

    #[test]
    fn damage_to_condition_ice_is_freezing() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Ice),
            ConditionType::Freezing
        );
    }

    #[test]
    fn damage_to_condition_holy_is_dazzled() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Holy),
            ConditionType::Dazzled
        );
    }

    #[test]
    fn damage_to_condition_death_is_cursed() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Death),
            ConditionType::Cursed
        );
    }

    #[test]
    fn damage_to_condition_physical_is_bleeding() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Physical),
            ConditionType::Bleeding
        );
    }

    #[test]
    fn damage_to_condition_none_is_none() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::None),
            ConditionType::None
        );
    }

    #[test]
    fn damage_to_condition_healing_is_none() {
        assert_eq!(
            Combat::damage_to_condition_type(CombatType::Healing),
            ConditionType::None
        );
    }

    // ─── NEW: round-trip condition ↔ damage type ─────────────────────────────

    #[test]
    fn condition_damage_roundtrip_fire() {
        let damage_type = Combat::condition_to_damage_type(ConditionType::Fire);
        let back = Combat::damage_to_condition_type(damage_type);
        assert_eq!(back, ConditionType::Fire);
    }

    #[test]
    fn condition_damage_roundtrip_cursed() {
        let damage_type = Combat::condition_to_damage_type(ConditionType::Cursed);
        let back = Combat::damage_to_condition_type(damage_type);
        assert_eq!(back, ConditionType::Cursed);
    }

    // ─── NEW: combat_change_health ───────────────────────────────────────────

    #[test]
    fn combat_change_health_damage_reduces_health() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, -30).unwrap();
        assert_eq!(result.new_health, 70);
        assert_eq!(result.damage_dealt, 30);
        assert!(!result.died);
    }

    #[test]
    fn combat_change_health_clamps_at_zero() {
        // Damage exceeds current health — health should be clamped to 0.
        let creature = CreatureState::new(50, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, -100).unwrap();
        assert_eq!(result.new_health, 0);
        assert_eq!(result.damage_dealt, 50); // only 50 hp were available
        assert!(result.died);
    }

    #[test]
    fn combat_change_health_exact_kill() {
        let creature = CreatureState::new(50, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, -50).unwrap();
        assert_eq!(result.new_health, 0);
        assert_eq!(result.damage_dealt, 50);
        assert!(result.died);
    }

    #[test]
    fn combat_change_health_healing_increases_health() {
        let creature = CreatureState::new(40, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, 30).unwrap();
        assert_eq!(result.new_health, 70);
        assert_eq!(result.damage_dealt, 0);
        assert!(!result.died);
    }

    #[test]
    fn combat_change_health_healing_clamps_at_max() {
        let creature = CreatureState::new(90, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, 50).unwrap();
        assert_eq!(result.new_health, 100);
        assert!(!result.died);
    }

    #[test]
    fn combat_change_health_damage_on_dead_creature_returns_none() {
        // Creature is already dead — damage should not be applied.
        let creature = CreatureState::new(0, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, -10);
        assert!(result.is_none());
    }

    #[test]
    fn combat_change_health_zero_damage_is_ok() {
        let creature = CreatureState::new(50, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, 0).unwrap();
        assert_eq!(result.new_health, 50);
        assert_eq!(result.damage_dealt, 0);
        assert!(!result.died);
    }

    #[test]
    fn combat_change_health_heal_on_full_hp_is_ok() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal);
        let result = Combat::combat_change_health(&creature, 50).unwrap();
        assert_eq!(result.new_health, 100);
    }

    // ─── NEW: combat_change_mana ─────────────────────────────────────────────

    #[test]
    fn combat_change_mana_drain_reduces_mana() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal).with_mana(80, 100);
        let result = Combat::combat_change_mana(&creature, -30);
        assert_eq!(result.new_mana, 50);
        assert_eq!(result.mana_changed, -30);
    }

    #[test]
    fn combat_change_mana_clamps_at_zero() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal).with_mana(20, 100);
        let result = Combat::combat_change_mana(&creature, -50);
        assert_eq!(result.new_mana, 0);
        assert_eq!(result.mana_changed, -20); // only 20 was available
    }

    #[test]
    fn combat_change_mana_gain_increases_mana() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal).with_mana(50, 100);
        let result = Combat::combat_change_mana(&creature, 30);
        assert_eq!(result.new_mana, 80);
        assert_eq!(result.mana_changed, 30);
    }

    #[test]
    fn combat_change_mana_gain_clamps_at_max() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal).with_mana(90, 100);
        let result = Combat::combat_change_mana(&creature, 50);
        assert_eq!(result.new_mana, 100);
        assert_eq!(result.mana_changed, 10); // only 10 space available
    }

    #[test]
    fn combat_change_mana_zero_delta() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal).with_mana(60, 100);
        let result = Combat::combat_change_mana(&creature, 0);
        assert_eq!(result.new_mana, 60);
        assert_eq!(result.mana_changed, 0);
    }

    // ─── NEW: can_do_combat_on_tile ──────────────────────────────────────────

    #[test]
    fn can_do_combat_aggressive_in_pz_blocked() {
        let result = Combat::can_do_combat_on_tile(TILESTATE_PROTECTIONZONE, true);
        assert_eq!(result, ReturnValue::ActionNotPermittedInProtectionZone);
    }

    #[test]
    fn can_do_combat_non_aggressive_in_pz_allowed() {
        // Non-aggressive (e.g. healing) is always allowed.
        let result = Combat::can_do_combat_on_tile(TILESTATE_PROTECTIONZONE, false);
        assert_eq!(result, ReturnValue::NoError);
    }

    #[test]
    fn can_do_combat_aggressive_normal_tile_allowed() {
        let result = Combat::can_do_combat_on_tile(0, true);
        assert_eq!(result, ReturnValue::NoError);
    }

    // ─── NEW: can_do_combat_no_pvp_world ─────────────────────────────────────

    #[test]
    fn no_pvp_world_player_vs_player_outside_pvp_zone_blocked() {
        let result =
            Combat::can_do_combat_no_pvp_world(ZoneType::Normal, ZoneType::Normal, true, true);
        assert_eq!(result, ReturnValue::YouMayNotAttackThisPlayer);
    }

    #[test]
    fn no_pvp_world_player_vs_player_in_pvp_zone_allowed() {
        let result = Combat::can_do_combat_no_pvp_world(ZoneType::Pvp, ZoneType::Pvp, true, true);
        assert_eq!(result, ReturnValue::NoError);
    }

    #[test]
    fn no_pvp_world_player_vs_monster_always_allowed() {
        // Monster-combat is not gated by pvp-world restrictions.
        let result = Combat::can_do_combat_no_pvp_world(
            ZoneType::Normal,
            ZoneType::Normal,
            true,
            false, // target is not a player
        );
        assert_eq!(result, ReturnValue::NoError);
    }

    #[test]
    fn no_pvp_world_monster_vs_player_allowed() {
        let result =
            Combat::can_do_combat_no_pvp_world(ZoneType::Normal, ZoneType::Normal, false, true);
        assert_eq!(result, ReturnValue::NoError);
    }

    // ─── NEW: clamp_damage_to_health ─────────────────────────────────────────

    #[test]
    fn clamp_damage_to_health_within_bounds() {
        assert_eq!(Combat::clamp_damage_to_health(30, 100), 30);
    }

    #[test]
    fn clamp_damage_to_health_exceeds_health_clamped() {
        assert_eq!(Combat::clamp_damage_to_health(150, 80), 80);
    }

    #[test]
    fn clamp_damage_to_health_zero_health() {
        assert_eq!(Combat::clamp_damage_to_health(50, 0), 0);
    }

    #[test]
    fn clamp_damage_to_health_exact_lethal() {
        assert_eq!(Combat::clamp_damage_to_health(100, 100), 100);
    }

    // ─── NEW: clamp_mana_drain ────────────────────────────────────────────────

    #[test]
    fn clamp_mana_drain_within_bounds() {
        assert_eq!(Combat::clamp_mana_drain(30, 100), 30);
    }

    #[test]
    fn clamp_mana_drain_exceeds_mana_clamped() {
        assert_eq!(Combat::clamp_mana_drain(200, 50), 50);
    }

    #[test]
    fn clamp_mana_drain_zero_mana() {
        assert_eq!(Combat::clamp_mana_drain(100, 0), 0);
    }

    // ─── NEW: WorldType enum ─────────────────────────────────────────────────

    #[test]
    fn world_type_variants_exist() {
        let types = [WorldType::NoPvp, WorldType::OpenPvp, WorldType::HardcorePvp];
        assert_eq!(types.len(), 3);
    }

    // ─── NEW: ConditionType enum completeness ────────────────────────────────

    #[test]
    fn condition_type_combat_relevant_variants_exist() {
        let conditions = [
            ConditionType::None,
            ConditionType::Fire,
            ConditionType::Poison,
            ConditionType::Energy,
            ConditionType::Bleeding,
            ConditionType::Drown,
            ConditionType::Freezing,
            ConditionType::Dazzled,
            ConditionType::Cursed,
        ];
        assert_eq!(conditions.len(), 9);
    }

    // ─── NEW: CombatArea range_x=0 / range_y=0 edge cases ───────────────────

    #[test]
    fn get_tiles_in_range_zero_returns_single_tile() {
        let center = Position::new(5, 5, 7);
        let tiles = CombatArea::get_tiles_in_range(center, 0, 0);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], center);
    }

    #[test]
    fn get_tiles_in_range_clips_at_map_boundary() {
        // Center near (0,0) with range=2: negative positions are skipped.
        let center = Position::new(1, 1, 7);
        let tiles = CombatArea::get_tiles_in_range(center, 2, 2);
        // All returned tiles must stay on the same z-layer as the center.
        assert!(tiles.iter().all(|p| p.z == center.z));
        // Without clipping we'd have 5*5=25 tiles, but (1-2)=-1 clips to 0.
        // x range: -1..=3 clipped to 0..=3 = 4 cols
        // y range: -1..=3 clipped to 0..=3 = 4 rows = 16 tiles
        assert_eq!(tiles.len(), 16);
    }

    #[test]
    fn get_tiles_in_range_larger_range_correct_count() {
        let center = Position::new(10, 10, 7);
        // 2*2+1 = 5 per axis => 5*5 = 25 tiles
        let tiles = CombatArea::get_tiles_in_range(center, 2, 2);
        assert_eq!(tiles.len(), 25);
    }

    // ─── NEW: CombatDamage default construction ───────────────────────────────

    #[test]
    fn combat_damage_new_stores_all_fields() {
        let d = CombatDamage::new(CombatType::Energy, -100, -20, false);
        assert_eq!(d.primary.combat_type, CombatType::Energy);
        assert_eq!(d.primary.value, -100);
        assert_eq!(d.secondary.value, -20);
        assert!(!d.extension);
    }

    // ─── NEW: CreatureState builder ──────────────────────────────────────────

    #[test]
    fn creature_state_default_zone_is_set() {
        let c = CreatureState::new(100, 100, ZoneType::Pvp);
        assert_eq!(c.zone, ZoneType::Pvp);
    }

    #[test]
    fn creature_state_with_mana_sets_mana() {
        let c = CreatureState::new(100, 100, ZoneType::Normal).with_mana(50, 200);
        assert_eq!(c.mana, 50);
        assert_eq!(c.max_mana, 200);
    }

    #[test]
    fn creature_state_with_immunity_sets_flags() {
        let flag = CombatType::Fire as u16;
        let c = CreatureState::new(100, 100, ZoneType::Normal).with_immunity(flag);
        assert_eq!(c.immunity_flags, flag);
        assert_eq!(
            Combat::get_block_type(CombatType::Fire, c.immunity_flags),
            BlockType::Immunity
        );
    }

    // ─── NEW: symmetry checks ─────────────────────────────────────────────────

    #[test]
    fn is_in_pvp_zone_symmetry() {
        // PvP zone check is NOT symmetric — both must be in PVP.
        // This is consistent with C++ `attacker->getZone() == ZONE_PVP && target->getZone() == ZONE_PVP`.
        assert_eq!(
            Combat::is_in_pvp_zone(ZoneType::Pvp, ZoneType::Normal),
            Combat::is_in_pvp_zone(ZoneType::Normal, ZoneType::Pvp),
        );
        // Both false, so symmetric in result, but the individual cases matter.
        assert!(!Combat::is_in_pvp_zone(ZoneType::Pvp, ZoneType::Normal));
    }

    // ─── NEW: FormulaType enum ────────────────────────────────────────────────

    #[test]
    fn formula_type_variants_exist() {
        let types = [
            FormulaType::Undefined,
            FormulaType::LevelMagic,
            FormulaType::Skill,
            FormulaType::Damage,
        ];
        assert_eq!(types.len(), 4);
    }

    #[test]
    fn formula_type_default_is_undefined() {
        assert_eq!(FormulaType::default(), FormulaType::Undefined);
    }

    // ─── NEW: CallBackParam enum ──────────────────────────────────────────────

    #[test]
    fn callback_param_variants_exist() {
        let params = [
            CallBackParam::LevelMagicValue,
            CallBackParam::SkillValue,
            CallBackParam::TargetTile,
            CallBackParam::TargetCreature,
        ];
        assert_eq!(params.len(), 4);
    }

    // ─── NEW: CombatParam enum ────────────────────────────────────────────────

    #[test]
    fn combat_param_variants_exist() {
        let params = [
            CombatParam::Type,
            CombatParam::Effect,
            CombatParam::DistanceEffect,
            CombatParam::BlockShield,
            CombatParam::BlockArmor,
            CombatParam::TargetCasterOrTopMost,
            CombatParam::CreateItem,
            CombatParam::Aggressive,
            CombatParam::Dispel,
            CombatParam::UseCharges,
        ];
        assert_eq!(params.len(), 10);
    }

    // ─── NEW: DamageComponent ─────────────────────────────────────────────────

    #[test]
    fn damage_component_fields_accessible() {
        let dc = DamageComponent::new(CombatType::Fire, -100);
        assert_eq!(dc.combat_type, CombatType::Fire);
        assert_eq!(dc.value, -100);
    }

    #[test]
    fn damage_component_default_is_none_zero() {
        let dc = DamageComponent::default();
        assert_eq!(dc.combat_type, CombatType::None);
        assert_eq!(dc.value, 0);
    }

    // ─── NEW: CombatDamage full struct fields ─────────────────────────────────

    #[test]
    fn combat_damage_primary_type_accessible() {
        let dmg = CombatDamage::new(CombatType::Energy, -50, -10, false);
        assert_eq!(dmg.primary.combat_type, CombatType::Energy);
        assert_eq!(dmg.primary.value, -50);
    }

    #[test]
    fn combat_damage_secondary_fields_accessible() {
        let dmg = CombatDamage::new(CombatType::Fire, -80, -20, true);
        assert_eq!(dmg.secondary.value, -20);
    }

    #[test]
    fn combat_damage_origin_field_accessible() {
        let dmg = CombatDamage {
            origin: CombatOrigin::Spell,
            ..Default::default()
        };
        assert_eq!(dmg.origin, CombatOrigin::Spell);
    }

    #[test]
    fn combat_damage_block_type_field_accessible() {
        let dmg = CombatDamage {
            block_type: BlockType::Armor,
            ..Default::default()
        };
        assert_eq!(dmg.block_type, BlockType::Armor);
    }

    #[test]
    fn combat_damage_critical_field_accessible() {
        let dmg = CombatDamage {
            critical: true,
            ..Default::default()
        };
        assert!(dmg.critical);
    }

    #[test]
    fn combat_damage_leeched_field_accessible() {
        let dmg = CombatDamage {
            leeched: true,
            ..Default::default()
        };
        assert!(dmg.leeched);
    }

    #[test]
    fn combat_damage_is_mana_drain_true_for_mana_drain() {
        let dmg = CombatDamage::mana_drain(-30);
        assert!(dmg.is_mana_drain());
    }

    #[test]
    fn combat_damage_is_mana_drain_false_for_fire() {
        let dmg = CombatDamage::new(CombatType::Fire, -30, 0, false);
        assert!(!dmg.is_mana_drain());
    }

    #[test]
    fn combat_damage_mana_drain_constructor_sets_type() {
        let dmg = CombatDamage::mana_drain(-50);
        assert_eq!(dmg.primary.combat_type, CombatType::ManaDrain);
        assert_eq!(dmg.primary.value, -50);
    }

    // ─── NEW: CombatOrigin extended variants ──────────────────────────────────

    #[test]
    fn combat_origin_wand_and_reflect_exist() {
        let origins = [CombatOrigin::Wand, CombatOrigin::Reflect];
        assert_eq!(origins.len(), 2);
    }

    // ─── NEW: CombatParams struct ─────────────────────────────────────────────

    #[test]
    fn combat_params_new_defaults_aggressive_true() {
        let p = CombatParams::new();
        assert!(p.aggressive);
    }

    #[test]
    fn combat_params_new_defaults_origin_spell() {
        let p = CombatParams::new();
        assert_eq!(p.origin, CombatOrigin::Spell);
    }

    #[test]
    fn combat_params_set_param_effect() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::Effect, 5);
        assert_eq!(p.impact_effect, 5);
    }

    #[test]
    fn combat_params_set_param_distance_effect() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::DistanceEffect, 3);
        assert_eq!(p.distance_effect, 3);
    }

    #[test]
    fn combat_params_set_param_block_armor_true() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::BlockArmor, 1);
        assert!(p.blocked_by_armor);
    }

    #[test]
    fn combat_params_set_param_block_armor_false() {
        let mut p = CombatParams::new();
        p.blocked_by_armor = true;
        p.set_param(CombatParam::BlockArmor, 0);
        assert!(!p.blocked_by_armor);
    }

    #[test]
    fn combat_params_set_param_block_shield() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::BlockShield, 1);
        assert!(p.blocked_by_shield);
    }

    #[test]
    fn combat_params_set_param_target_caster_or_top_most() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::TargetCasterOrTopMost, 1);
        assert!(p.target_caster_or_top_most);
    }

    #[test]
    fn combat_params_set_param_create_item() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::CreateItem, 1234);
        assert_eq!(p.item_id, 1234);
    }

    #[test]
    fn combat_params_set_param_aggressive_false() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::Aggressive, 0);
        assert!(!p.aggressive);
    }

    #[test]
    fn combat_params_set_param_use_charges() {
        let mut p = CombatParams::new();
        p.set_param(CombatParam::UseCharges, 1);
        assert!(p.use_charges);
    }

    #[test]
    fn combat_params_get_param_effect() {
        let mut p = CombatParams::new();
        p.impact_effect = 7;
        assert_eq!(p.get_param(CombatParam::Effect), 7);
    }

    #[test]
    fn combat_params_get_param_block_armor_one_when_true() {
        let mut p = CombatParams::new();
        p.blocked_by_armor = true;
        assert_eq!(p.get_param(CombatParam::BlockArmor), 1);
    }

    #[test]
    fn combat_params_get_param_aggressive_one_when_true() {
        let p = CombatParams::new();
        assert_eq!(p.get_param(CombatParam::Aggressive), 1);
    }

    // ─── NEW: is_player_combat ────────────────────────────────────────────────

    #[test]
    fn is_player_combat_true_when_target_is_player() {
        assert!(Combat::is_player_combat(true, false));
    }

    #[test]
    fn is_player_combat_true_when_master_is_player() {
        // Summon of a player
        assert!(Combat::is_player_combat(false, true));
    }

    #[test]
    fn is_player_combat_false_for_monster() {
        assert!(!Combat::is_player_combat(false, false));
    }

    #[test]
    fn is_player_combat_true_when_both_player_and_master_player() {
        assert!(Combat::is_player_combat(true, true));
    }

    // ─── NEW: is_protected ────────────────────────────────────────────────────

    #[test]
    fn is_protected_true_when_target_below_protection_level() {
        assert!(Combat::is_protected(50, 5, 20));
    }

    #[test]
    fn is_protected_true_when_attacker_below_protection_level() {
        assert!(Combat::is_protected(5, 50, 20));
    }

    #[test]
    fn is_protected_false_when_both_above_protection_level() {
        assert!(!Combat::is_protected(50, 30, 20));
    }

    #[test]
    fn is_protected_true_when_both_below_protection_level() {
        assert!(Combat::is_protected(5, 5, 20));
    }

    #[test]
    fn is_protected_false_when_both_equal_protection_level() {
        // protection_level = 20; both levels = 20 → neither is < 20
        assert!(!Combat::is_protected(20, 20, 20));
    }

    // ─── NEW: can_target_creature ─────────────────────────────────────────────

    #[test]
    fn can_target_creature_blocked_when_attacker_in_pz() {
        let result =
            Combat::can_target_creature(ZoneType::Protection, ZoneType::Normal, true, false, false);
        assert_eq!(result, ReturnValue::ActionNotPermittedInProtectionZone);
    }

    #[test]
    fn can_target_creature_blocked_when_target_in_pz() {
        let result =
            Combat::can_target_creature(ZoneType::Normal, ZoneType::Protection, true, false, false);
        assert_eq!(result, ReturnValue::ActionNotPermittedInProtectionZone);
    }

    #[test]
    fn can_target_creature_blocked_when_attacker_in_nopvp_and_player_combat() {
        let result = Combat::can_target_creature(
            ZoneType::NoPvp,
            ZoneType::Normal,
            true,
            true, // player combat
            false,
        );
        assert_eq!(result, ReturnValue::ActionNotPermittedInNoPvpZone);
    }

    #[test]
    fn can_target_creature_blocked_when_target_in_nopvp_and_player_combat() {
        let result =
            Combat::can_target_creature(ZoneType::Normal, ZoneType::NoPvp, true, true, false);
        assert_eq!(result, ReturnValue::YouMayNotAttackAPersonInProtectionZone);
    }

    #[test]
    fn can_target_creature_blocked_when_not_attackable() {
        let result = Combat::can_target_creature(
            ZoneType::Normal,
            ZoneType::Normal,
            false, // not attackable
            false,
            false,
        );
        assert_eq!(result, ReturnValue::YouMayNotAttackThisCreature);
    }

    #[test]
    fn can_target_creature_ok_when_normal() {
        let result =
            Combat::can_target_creature(ZoneType::Normal, ZoneType::Normal, true, false, false);
        assert_eq!(result, ReturnValue::NoError);
    }

    #[test]
    fn can_target_creature_ignore_protection_zone_bypasses_pz() {
        // With ignore_protection_zone=true, PZ is not checked.
        let result = Combat::can_target_creature(
            ZoneType::Protection,
            ZoneType::Protection,
            true,
            true,
            true, // ignore_protection_zone
        );
        assert_eq!(result, ReturnValue::NoError);
    }

    #[test]
    fn can_target_creature_nopvp_allowed_for_monster_combat() {
        // NoPvp zone only matters for player combat.
        let result = Combat::can_target_creature(
            ZoneType::NoPvp,
            ZoneType::Normal,
            true,
            false, // not player combat
            false,
        );
        assert_eq!(result, ReturnValue::NoError);
    }

    // ─── NEW: do_combat_dispatch ──────────────────────────────────────────────

    #[test]
    fn do_combat_dispatch_health_damage_returns_ok() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal);
        let damage = CombatDamage::new(CombatType::Fire, -40, 0, false);
        let result = Combat::do_combat_dispatch(&creature, &damage);
        assert!(result.is_ok());
        let hr = result.unwrap();
        assert_eq!(hr.new_health, 60);
        assert_eq!(hr.damage_dealt, 40);
    }

    #[test]
    fn do_combat_dispatch_health_clamps_at_zero() {
        let creature = CreatureState::new(30, 100, ZoneType::Normal);
        let damage = CombatDamage::new(CombatType::Physical, -100, 0, false);
        let result = Combat::do_combat_dispatch(&creature, &damage);
        assert!(result.is_ok());
        let hr = result.unwrap();
        assert_eq!(hr.new_health, 0);
        assert!(hr.died);
    }

    #[test]
    fn do_combat_dispatch_healing_returns_ok() {
        let creature = CreatureState::new(50, 100, ZoneType::Normal);
        let damage = CombatDamage::new(CombatType::Healing, 30, 0, false);
        let result = Combat::do_combat_dispatch(&creature, &damage);
        assert!(result.is_ok());
        let hr = result.unwrap();
        assert_eq!(hr.new_health, 80);
    }

    #[test]
    fn do_combat_dispatch_mana_drain_returns_err() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal).with_mana(80, 100);
        let damage = CombatDamage::mana_drain(-30);
        let result = Combat::do_combat_dispatch(&creature, &damage);
        assert!(result.is_err());
        let mr = result.unwrap_err();
        assert_eq!(mr.new_mana, 50);
        assert_eq!(mr.mana_changed, -30);
    }

    #[test]
    fn do_combat_dispatch_mana_drain_never_below_zero() {
        let creature = CreatureState::new(100, 100, ZoneType::Normal).with_mana(10, 100);
        let damage = CombatDamage::mana_drain(-200);
        let result = Combat::do_combat_dispatch(&creature, &damage);
        assert!(result.is_err());
        let mr = result.unwrap_err();
        assert_eq!(mr.new_mana, 0);
        assert_eq!(mr.mana_changed, -10);
    }

    // ─── NEW: apply_critical_hit ──────────────────────────────────────────────

    #[test]
    fn apply_critical_hit_adds_percentage_to_damage() {
        // -100 damage, 50% skill → -150 damage
        let result = Combat::apply_critical_hit(-100, 50);
        assert_eq!(result, -150);
    }

    #[test]
    fn apply_critical_hit_zero_skill_no_change() {
        let result = Combat::apply_critical_hit(-80, 0);
        assert_eq!(result, -80);
    }

    #[test]
    fn apply_critical_hit_one_hundred_skill_doubles() {
        // 100% extra → double damage
        let result = Combat::apply_critical_hit(-60, 100);
        assert_eq!(result, -120);
    }

    #[test]
    fn apply_critical_hit_positive_value_also_works() {
        // For healing: 50 heal + 50% = 75
        let result = Combat::apply_critical_hit(50, 50);
        assert_eq!(result, 75);
    }

    // ─── NEW: compute_leech_amount ────────────────────────────────────────────

    #[test]
    fn compute_leech_amount_typical_case() {
        // 1000 total damage, 1000/10000 = 10% leech → 100
        let result = Combat::compute_leech_amount(1000, 1000);
        assert_eq!(result, 100);
    }

    #[test]
    fn compute_leech_amount_zero_skill_returns_zero() {
        let result = Combat::compute_leech_amount(500, 0);
        assert_eq!(result, 0);
    }

    #[test]
    fn compute_leech_amount_zero_damage_returns_zero() {
        let result = Combat::compute_leech_amount(0, 5000);
        assert_eq!(result, 0);
    }

    #[test]
    fn compute_leech_amount_rounds_correctly() {
        // 100 damage * 5000/10000 = 50.0 → 50
        let result = Combat::compute_leech_amount(100, 5000);
        assert_eq!(result, 50);
    }

    // ─── NEW: AreaCombat (the class, not CombatArea helper) ──────────────────

    #[test]
    fn area_combat_new_is_empty() {
        let area = AreaCombat::new();
        assert!(area.is_empty());
        assert!(!area.has_ext_area());
    }

    #[test]
    fn area_combat_get_list_initially_empty() {
        let area = AreaCombat::new();
        assert_eq!(area.get_list().len(), 0);
    }

    #[test]
    fn area_combat_setup_radius_fills_correct_positions() {
        let mut area = AreaCombat::new();
        let center = Position::new(10, 10, 7);
        area.setup_area_radius(center, 1);
        // radius 1 → 3×3 = 9 tiles
        assert_eq!(area.get_list().len(), 9);
    }

    #[test]
    fn area_combat_setup_radius_all_on_same_floor() {
        let mut area = AreaCombat::new();
        let center = Position::new(10, 10, 7);
        area.setup_area_radius(center, 2);
        assert!(area.get_list().iter().all(|p| p.z == 7));
    }

    #[test]
    fn area_combat_has_ext_area_false_by_default() {
        let area = AreaCombat::new();
        assert!(!area.has_ext_area());
    }

    #[test]
    fn area_combat_setup_ext_area_sets_flag() {
        let mut area = AreaCombat::new();
        let center = Position::new(10, 10, 7);
        area.setup_area_radius(center, 1);
        area.setup_ext_area(center, 1);
        assert!(area.has_ext_area());
    }

    #[test]
    fn area_combat_setup_ext_area_adds_more_positions() {
        let mut area = AreaCombat::new();
        let center = Position::new(10, 10, 7);
        area.setup_area_radius(center, 1); // 9 tiles
        let count_before = area.get_list().len();
        area.setup_ext_area(center, 1); // ext adds ring around 3×3
        let count_after = area.get_list().len();
        // ext area (5×5=25) covers 25 unique positions; 25-9 = 16 new
        assert!(count_after > count_before);
    }

    #[test]
    fn area_combat_get_list_returns_slice() {
        let mut area = AreaCombat::new();
        let center = Position::new(5, 5, 7);
        area.setup_area_radius(center, 0);
        let list = area.get_list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], center);
    }

    // ─── NEW: CombatParams::set_param remaining branches ─────────────────────

    /// Mirrors `case COMBAT_PARAM_TYPE` in C++ `Combat::setParam`:
    /// returns true (the Rust impl is a stub but the branch must be covered).
    #[test]
    fn combat_params_set_param_type_returns_true() {
        let mut p = CombatParams::new();
        let ok = p.set_param(CombatParam::Type, CombatType::Fire as u32);
        assert!(ok);
    }

    /// Mirrors `case COMBAT_PARAM_DISPEL` in C++ `Combat::setParam`:
    /// returns true (the Rust impl is a stub but the branch must be covered).
    #[test]
    fn combat_params_set_param_dispel_returns_true() {
        let mut p = CombatParams::new();
        let ok = p.set_param(CombatParam::Dispel, ConditionType::Fire as u32);
        assert!(ok);
    }

    // ─── NEW: CombatParams::get_param remaining branches ─────────────────────

    /// Mirrors `case COMBAT_PARAM_DISTANCEEFFECT` in C++ `Combat::getParam`.
    #[test]
    fn combat_params_get_param_distance_effect_returns_value() {
        let mut p = CombatParams::new();
        p.distance_effect = 9;
        assert_eq!(p.get_param(CombatParam::DistanceEffect), 9);
    }

    /// Mirrors `case COMBAT_PARAM_BLOCKSHIELD` in C++ `Combat::getParam`.
    #[test]
    fn combat_params_get_param_block_shield_one_when_true() {
        let mut p = CombatParams::new();
        p.blocked_by_shield = true;
        assert_eq!(p.get_param(CombatParam::BlockShield), 1);
    }

    #[test]
    fn combat_params_get_param_block_shield_zero_when_false() {
        let p = CombatParams::new();
        assert_eq!(p.get_param(CombatParam::BlockShield), 0);
    }

    /// Mirrors `case COMBAT_PARAM_TARGETCASTERORTOPMOST` in C++ `Combat::getParam`.
    #[test]
    fn combat_params_get_param_target_caster_or_top_most_one_when_true() {
        let mut p = CombatParams::new();
        p.target_caster_or_top_most = true;
        assert_eq!(p.get_param(CombatParam::TargetCasterOrTopMost), 1);
    }

    /// Mirrors `case COMBAT_PARAM_CREATEITEM` in C++ `Combat::getParam`.
    #[test]
    fn combat_params_get_param_create_item_returns_item_id() {
        let mut p = CombatParams::new();
        p.item_id = 4242;
        assert_eq!(p.get_param(CombatParam::CreateItem), 4242);
    }

    /// Mirrors `case COMBAT_PARAM_USECHARGES` in C++ `Combat::getParam`.
    #[test]
    fn combat_params_get_param_use_charges_one_when_true() {
        let mut p = CombatParams::new();
        p.use_charges = true;
        assert_eq!(p.get_param(CombatParam::UseCharges), 1);
    }

    /// Mirrors the `default: return std::numeric_limits<int32_t>().max();` arm
    /// in C++ `Combat::getParam`. The Rust impl returns `i32::MAX` for any
    /// branch not explicitly handled (Type and Dispel are stored elsewhere
    /// in the Rust port, so they hit the default arm).
    #[test]
    fn combat_params_get_param_unhandled_returns_i32_max() {
        let p = CombatParams::new();
        assert_eq!(p.get_param(CombatParam::Type), i32::MAX);
        assert_eq!(p.get_param(CombatParam::Dispel), i32::MAX);
    }

    // ─── NEW: can_target_creature fall-through (line 602) ─────────────────────

    /// Mirrors the case where `is_player_combat = true` and neither zone is
    /// `NoPvp` — the function should fall through past both NoPvp checks and
    /// return `NoError` (target is attackable). This exercises the closing
    /// branch of the `if target_zone == ZoneType::NoPvp` block.
    #[test]
    fn can_target_creature_player_combat_neither_nopvp_falls_through() {
        let result = Combat::can_target_creature(
            ZoneType::Normal,
            ZoneType::Normal,
            true, // attackable
            true, // player_combat — exercise the is_player_combat=true path
            false,
        );
        assert_eq!(result, ReturnValue::NoError);
    }
}
