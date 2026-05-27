use std::collections::HashMap;

/// Mirrors `SpellType_t` from enums.h.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellType {
    Undefined,
    Instant,
    Rune,
    Conjure,
}

/// Mirrors `SpellGroup_t` from enums.h.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellGroup {
    None,
    Attack,
    Healing,
    Support,
    Special,
}

/// Result of a `can_cast` check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellCastResult {
    Ok,
    NotEnoughMana,
    LevelTooLow,
    WrongVocation,
    TargetNotFound,
    /// Soul points insufficient.
    NotEnoughSoul,
    /// Spell or group cooldown is active.
    Exhausted,
    /// Caster's magic level is too low.
    MagicLevelTooLow,
    /// Spell requires premium account.
    NeedPremium,
    /// Spell is disabled.
    SpellDisabled,
    /// Spell must be learned first.
    NotLearned,
    /// Caster needs a melee weapon equipped.
    NeedWeapon,
}

/// Exhaustion/cooldown state passed to `check_exhaustion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExhaustionState {
    Ready,
    OnCooldown,
}

/// Target type for an InstantSpell cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstantTargetType {
    /// Targets the caster themselves.
    SelfTarget,
    /// Targets a creature.
    Creature,
    /// Targets a map position (direction-based).
    Position,
}

/// Data model for a castable spell, mirroring the C++ `Spell` base class.
#[derive(Debug, Clone)]
pub struct Spell {
    pub name: String,
    pub words: String,
    pub spell_id: u8,

    // Cost fields
    pub mana_cost: u32,
    pub mana_percent: u32,
    pub soul_cost: u32,

    // Requirement fields
    pub min_level: u32,
    pub magic_level: u32,
    /// Empty = all vocations can cast.
    pub required_vocations: Vec<u16>,

    // Cooldown fields (milliseconds)
    pub cooldown: u32,
    pub group_cooldown: u32,
    pub secondary_group_cooldown: u32,

    // Groups
    pub group: SpellGroup,
    pub secondary_group: SpellGroup,

    // Flags
    pub premium: bool,
    pub enabled: bool,
    pub learnable: bool,
    pub need_target: bool,
    pub need_weapon: bool,
    pub self_target: bool,
    pub blocking_solid: bool,
    pub blocking_creature: bool,
    pub aggressive: bool,
    pub pz_lock: bool,

    pub range: i32,
}

impl Spell {
    pub fn new(
        name: impl Into<String>,
        mana_cost: u32,
        min_level: u32,
        required_vocations: Vec<u16>,
    ) -> Self {
        let n = name.into();
        Spell {
            words: n.clone(),
            name: n,
            spell_id: 0,
            mana_cost,
            mana_percent: 0,
            soul_cost: 0,
            min_level,
            magic_level: 0,
            required_vocations,
            cooldown: 1000,
            group_cooldown: 1000,
            secondary_group_cooldown: 0,
            group: SpellGroup::None,
            secondary_group: SpellGroup::None,
            premium: false,
            enabled: true,
            learnable: false,
            need_target: false,
            need_weapon: false,
            self_target: false,
            blocking_solid: false,
            blocking_creature: false,
            aggressive: true,
            pz_lock: false,
            range: -1,
        }
    }

    /// Compute the mana cost for a given player's max mana.
    /// Matches C++ `Spell::getManaCost`: flat `mana`, else `manaPercent` of max_mana.
    pub fn get_mana_cost(&self, player_max_mana: u32) -> u32 {
        if self.mana_cost != 0 {
            return self.mana_cost;
        }
        if self.mana_percent != 0 {
            return (player_max_mana * self.mana_percent) / 100;
        }
        0
    }

    /// Check whether a player can cast this spell.
    pub fn can_cast(
        &self,
        player_level: u32,
        player_mana: u32,
        vocation_id: u16,
    ) -> SpellCastResult {
        if player_level < self.min_level {
            return SpellCastResult::LevelTooLow;
        }
        if player_mana < self.mana_cost {
            return SpellCastResult::NotEnoughMana;
        }
        if !self.required_vocations.is_empty() && !self.required_vocations.contains(&vocation_id) {
            return SpellCastResult::WrongVocation;
        }
        SpellCastResult::Ok
    }

    /// Check level precondition alone.
    pub fn check_level(&self, player_level: u32) -> SpellCastResult {
        if player_level < self.min_level {
            SpellCastResult::LevelTooLow
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check mana precondition, supporting percent-based cost.
    pub fn check_mana(&self, player_mana: u32, player_max_mana: u32) -> SpellCastResult {
        let cost = self.get_mana_cost(player_max_mana);
        if player_mana < cost {
            SpellCastResult::NotEnoughMana
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check soul cost precondition.
    pub fn check_soul(&self, player_soul: u32) -> SpellCastResult {
        if player_soul < self.soul_cost {
            SpellCastResult::NotEnoughSoul
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check exhaustion/cooldown precondition.
    pub fn check_exhaustion(&self, state: ExhaustionState) -> SpellCastResult {
        if state == ExhaustionState::OnCooldown {
            SpellCastResult::Exhausted
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check magic level precondition.
    pub fn check_magic_level(&self, player_magic_level: u32) -> SpellCastResult {
        if player_magic_level < self.magic_level {
            SpellCastResult::MagicLevelTooLow
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check premium account precondition.
    pub fn check_premium(&self, player_is_premium: bool) -> SpellCastResult {
        if self.premium && !player_is_premium {
            SpellCastResult::NeedPremium
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check enabled flag.
    pub fn check_enabled(&self) -> SpellCastResult {
        if !self.enabled {
            SpellCastResult::SpellDisabled
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check whether the player has learned this spell (for learnable instant spells).
    pub fn check_learned(&self, player_has_learned: bool) -> SpellCastResult {
        if self.learnable && !player_has_learned {
            SpellCastResult::NotLearned
        } else {
            SpellCastResult::Ok
        }
    }

    /// Check weapon precondition.
    pub fn check_weapon(&self, player_has_weapon: bool) -> SpellCastResult {
        if self.need_weapon && !player_has_weapon {
            SpellCastResult::NeedWeapon
        } else {
            SpellCastResult::Ok
        }
    }

    /// Compute mana + soul deductions after a successful cast.
    /// Returns `(mana_spent, soul_spent)`.
    pub fn post_cast_costs(&self, player_max_mana: u32) -> (u32, u32) {
        (self.get_mana_cost(player_max_mana), self.soul_cost)
    }
}

/// Mirrors C++ `InstantSpell` extra fields.
#[derive(Debug, Clone)]
pub struct InstantSpell {
    pub spell: Spell,
    pub need_direction: bool,
    pub has_param: bool,
    pub has_player_name_param: bool,
    pub check_line_of_sight: bool,
    pub caster_target_or_direction: bool,
}

impl InstantSpell {
    pub fn new(spell: Spell) -> Self {
        InstantSpell {
            spell,
            need_direction: false,
            has_param: false,
            has_player_name_param: false,
            check_line_of_sight: true,
            caster_target_or_direction: false,
        }
    }

    /// Resolve the effective target type based on the C++ `playerCastInstant` logic.
    pub fn target_type(&self) -> InstantTargetType {
        if self.spell.self_target {
            InstantTargetType::SelfTarget
        } else if self.need_direction && !self.caster_target_or_direction {
            InstantTargetType::Position
        } else if self.spell.need_target || self.caster_target_or_direction {
            InstantTargetType::Creature
        } else {
            InstantTargetType::Position
        }
    }
}

/// Mirrors C++ `RuneSpell` extra fields.
#[derive(Debug, Clone)]
pub struct RuneSpell {
    pub spell: Spell,
    pub rune_item_id: u16,
    pub charges: u32,
    pub has_charges: bool,
}

impl RuneSpell {
    pub fn new(spell: Spell, rune_item_id: u16) -> Self {
        RuneSpell {
            spell,
            rune_item_id,
            charges: 0,
            has_charges: false,
        }
    }

    pub fn set_charges(&mut self, c: u32) {
        if c > 0 {
            self.has_charges = true;
        }
        self.charges = c;
    }
}

/// Simple healing spell.
#[derive(Debug, Clone, Copy)]
pub struct HealingSpell {
    pub base_heal: u32,
}

impl HealingSpell {
    pub fn new(base_heal: u32) -> Self {
        HealingSpell { base_heal }
    }

    pub fn compute_heal(&self, _seed: u32) -> u32 {
        self.base_heal
    }
}

/// Registry of all spells (game logic layer — distinct from `spell_registry.rs`).
#[derive(Debug, Default)]
pub struct Spells {
    by_name: HashMap<String, Spell>,
    runes: HashMap<u16, RuneSpell>,
    instants: HashMap<String, InstantSpell>,
}

impl Spells {
    pub fn new() -> Self {
        Spells {
            by_name: HashMap::new(),
            runes: HashMap::new(),
            instants: HashMap::new(),
        }
    }

    pub fn register(&mut self, spell_name: impl Into<String>, spell: Spell) {
        self.by_name.insert(spell_name.into(), spell);
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Spell> {
        self.by_name.get(name)
    }

    pub fn get_by_words(&self, words: &str) -> Option<&Spell> {
        self.by_name.values().find(|s| s.words == words)
    }

    /// Register an instant spell.
    pub fn register_instant(&mut self, instant: InstantSpell) {
        self.instants.insert(instant.spell.words.clone(), instant);
    }

    /// Get an instant spell by its cast words.
    pub fn get_instant_spell(&self, words: &str) -> Option<&InstantSpell> {
        self.instants.get(words)
    }

    /// Get an instant spell by name.
    pub fn get_instant_spell_by_name(&self, name: &str) -> Option<&InstantSpell> {
        self.instants.values().find(|i| i.spell.name == name)
    }

    /// Register a rune spell.
    pub fn register_rune(&mut self, rune: RuneSpell) {
        self.runes.insert(rune.rune_item_id, rune);
    }

    /// Get a rune spell by item id.
    pub fn get_rune_spell(&self, item_id: u16) -> Option<&RuneSpell> {
        self.runes.get(&item_id)
    }

    /// Get a rune spell by name.
    pub fn get_rune_spell_by_name(&self, name: &str) -> Option<&RuneSpell> {
        self.runes.values().find(|r| r.spell.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------ //
    // existing tests (preserved)                                          //
    // ------------------------------------------------------------------ //

    // 1. SpellType enum variants
    #[test]
    fn spell_type_variants_exist() {
        let _ = SpellType::Undefined;
        let _ = SpellType::Instant;
        let _ = SpellType::Rune;
        let _ = SpellType::Conjure;
    }

    // 2. SpellCastResult variants
    #[test]
    fn spell_cast_result_variants_exist() {
        let _ = SpellCastResult::Ok;
        let _ = SpellCastResult::NotEnoughMana;
        let _ = SpellCastResult::LevelTooLow;
        let _ = SpellCastResult::WrongVocation;
        let _ = SpellCastResult::TargetNotFound;
    }

    // 3a. Level too low
    #[test]
    fn can_cast_level_too_low() {
        let spell = Spell::new("Exura", 50, 10, vec![]);
        assert_eq!(spell.can_cast(5, 200, 1), SpellCastResult::LevelTooLow);
    }

    // 3b. Not enough mana
    #[test]
    fn can_cast_not_enough_mana() {
        let spell = Spell::new("Exura", 100, 1, vec![]);
        assert_eq!(spell.can_cast(5, 50, 1), SpellCastResult::NotEnoughMana);
    }

    // 3c. Wrong vocation
    #[test]
    fn can_cast_wrong_vocation() {
        let spell = Spell::new("Exura", 50, 1, vec![1, 2]);
        assert_eq!(spell.can_cast(5, 200, 3), SpellCastResult::WrongVocation);
    }

    // 3d. Empty vocation list = all vocations allowed
    #[test]
    fn can_cast_all_vocations_when_list_empty() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        assert_eq!(spell.can_cast(5, 200, 99), SpellCastResult::Ok);
    }

    // 3e. All conditions met
    #[test]
    fn can_cast_ok_when_all_conditions_met() {
        let spell = Spell::new("Exura", 50, 10, vec![1, 2]);
        assert_eq!(spell.can_cast(10, 50, 1), SpellCastResult::Ok);
    }

    // 4. HealingSpell
    #[test]
    fn healing_spell_returns_base_heal() {
        let hs = HealingSpell::new(100);
        assert_eq!(hs.compute_heal(42), 100);
    }

    // 5a. Empty registry
    #[test]
    fn spells_registry_new_is_empty() {
        let spells = Spells::new();
        assert!(spells.get_by_name("Exura").is_none());
    }

    // 5b. Register and get_by_name
    #[test]
    fn spells_registry_register_and_get_by_name() {
        let mut spells = Spells::new();
        let spell = Spell::new("Exura", 50, 1, vec![]);
        spells.register("Exura", spell);
        assert!(spells.get_by_name("Exura").is_some());
    }

    // 5c. get_by_name returns None for unknown
    #[test]
    fn spells_registry_get_by_name_unknown_returns_none() {
        let spells = Spells::new();
        assert!(spells.get_by_name("Unknown").is_none());
    }

    // 5d. get_by_words
    #[test]
    fn spells_registry_get_by_words() {
        let mut spells = Spells::new();
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.words = "exura".to_string();
        spells.register("Exura", spell);
        assert!(spells.get_by_words("exura").is_some());
    }

    // 5e. get_by_words returns None for unknown
    #[test]
    fn spells_registry_get_by_words_unknown_returns_none() {
        let spells = Spells::new();
        assert!(spells.get_by_words("unknown words").is_none());
    }

    // ------------------------------------------------------------------ //
    // NEW tests — Phase 12.2 gaps                                         //
    // ------------------------------------------------------------------ //

    // SpellGroup enum variants
    #[test]
    fn spell_group_variants_exist() {
        let _ = SpellGroup::None;
        let _ = SpellGroup::Attack;
        let _ = SpellGroup::Healing;
        let _ = SpellGroup::Support;
        let _ = SpellGroup::Special;
    }

    // SpellCastResult new variants
    #[test]
    fn spell_cast_result_new_variants_exist() {
        let _ = SpellCastResult::NotEnoughSoul;
        let _ = SpellCastResult::Exhausted;
        let _ = SpellCastResult::MagicLevelTooLow;
        let _ = SpellCastResult::NeedPremium;
        let _ = SpellCastResult::SpellDisabled;
        let _ = SpellCastResult::NotLearned;
        let _ = SpellCastResult::NeedWeapon;
    }

    // ExhaustionState enum
    #[test]
    fn exhaustion_state_variants_exist() {
        let _ = ExhaustionState::Ready;
        let _ = ExhaustionState::OnCooldown;
    }

    // InstantTargetType enum
    #[test]
    fn instant_target_type_variants_exist() {
        let _ = InstantTargetType::SelfTarget;
        let _ = InstantTargetType::Creature;
        let _ = InstantTargetType::Position;
    }

    // ---- check_level ----

    #[test]
    fn check_level_returns_level_too_low_when_below_min() {
        let spell = Spell::new("Exura", 50, 20, vec![]);
        assert_eq!(spell.check_level(10), SpellCastResult::LevelTooLow);
    }

    #[test]
    fn check_level_returns_ok_at_exact_min_level() {
        let spell = Spell::new("Exura", 50, 20, vec![]);
        assert_eq!(spell.check_level(20), SpellCastResult::Ok);
    }

    #[test]
    fn check_level_returns_ok_above_min_level() {
        let spell = Spell::new("Exura", 50, 20, vec![]);
        assert_eq!(spell.check_level(100), SpellCastResult::Ok);
    }

    // ---- check_mana (flat) ----

    #[test]
    fn check_mana_flat_returns_not_enough_mana() {
        let spell = Spell::new("Exura", 100, 1, vec![]);
        assert_eq!(spell.check_mana(50, 500), SpellCastResult::NotEnoughMana);
    }

    #[test]
    fn check_mana_flat_returns_ok_at_exact() {
        let spell = Spell::new("Exura", 100, 1, vec![]);
        assert_eq!(spell.check_mana(100, 500), SpellCastResult::Ok);
    }

    // ---- check_mana (percent-based) ----

    #[test]
    fn check_mana_percent_based_deducts_correctly() {
        // 10% of 200 max mana = 20 mana required
        let mut spell = Spell::new("Exura", 0, 1, vec![]);
        spell.mana_percent = 10;
        assert_eq!(spell.check_mana(10, 200), SpellCastResult::NotEnoughMana);
    }

    #[test]
    fn check_mana_percent_based_ok_when_sufficient() {
        let mut spell = Spell::new("Exura", 0, 1, vec![]);
        spell.mana_percent = 10;
        // 10% of 200 = 20, player has 20
        assert_eq!(spell.check_mana(20, 200), SpellCastResult::Ok);
    }

    #[test]
    fn get_mana_cost_returns_flat_mana_when_nonzero() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        assert_eq!(spell.get_mana_cost(1000), 50);
    }

    #[test]
    fn get_mana_cost_returns_percent_when_flat_is_zero() {
        let mut spell = Spell::new("Exura", 0, 1, vec![]);
        spell.mana_percent = 25;
        // 25% of 200 = 50
        assert_eq!(spell.get_mana_cost(200), 50);
    }

    #[test]
    fn get_mana_cost_returns_zero_when_both_zero() {
        let spell = Spell::new("Exura", 0, 1, vec![]);
        assert_eq!(spell.get_mana_cost(500), 0);
    }

    // ---- check_soul ----

    #[test]
    fn check_soul_returns_not_enough_soul() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.soul_cost = 5;
        assert_eq!(spell.check_soul(2), SpellCastResult::NotEnoughSoul);
    }

    #[test]
    fn check_soul_returns_ok_at_exact_soul() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.soul_cost = 5;
        assert_eq!(spell.check_soul(5), SpellCastResult::Ok);
    }

    #[test]
    fn check_soul_returns_ok_when_no_soul_cost() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        // soul_cost defaults to 0
        assert_eq!(spell.check_soul(0), SpellCastResult::Ok);
    }

    // ---- check_exhaustion ----

    #[test]
    fn check_exhaustion_on_cooldown_returns_exhausted() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        assert_eq!(
            spell.check_exhaustion(ExhaustionState::OnCooldown),
            SpellCastResult::Exhausted
        );
    }

    #[test]
    fn check_exhaustion_ready_returns_ok() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        assert_eq!(
            spell.check_exhaustion(ExhaustionState::Ready),
            SpellCastResult::Ok
        );
    }

    // ---- check_magic_level ----

    #[test]
    fn check_magic_level_too_low_returns_error() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.magic_level = 5;
        assert_eq!(
            spell.check_magic_level(3),
            SpellCastResult::MagicLevelTooLow
        );
    }

    #[test]
    fn check_magic_level_exact_returns_ok() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.magic_level = 5;
        assert_eq!(spell.check_magic_level(5), SpellCastResult::Ok);
    }

    #[test]
    fn check_magic_level_zero_requirement_always_ok() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        assert_eq!(spell.check_magic_level(0), SpellCastResult::Ok);
    }

    // ---- check_premium ----

    #[test]
    fn check_premium_fails_when_premium_required_and_player_not_premium() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.premium = true;
        assert_eq!(spell.check_premium(false), SpellCastResult::NeedPremium);
    }

    #[test]
    fn check_premium_ok_when_premium_required_and_player_is_premium() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.premium = true;
        assert_eq!(spell.check_premium(true), SpellCastResult::Ok);
    }

    #[test]
    fn check_premium_ok_when_not_required() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        // premium = false by default
        assert_eq!(spell.check_premium(false), SpellCastResult::Ok);
    }

    // ---- check_enabled ----

    #[test]
    fn check_enabled_disabled_spell_returns_error() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.enabled = false;
        assert_eq!(spell.check_enabled(), SpellCastResult::SpellDisabled);
    }

    #[test]
    fn check_enabled_enabled_spell_returns_ok() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        // enabled = true by default
        assert_eq!(spell.check_enabled(), SpellCastResult::Ok);
    }

    // ---- check_learned ----

    #[test]
    fn check_learned_learnable_and_not_learned_returns_not_learned() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.learnable = true;
        assert_eq!(spell.check_learned(false), SpellCastResult::NotLearned);
    }

    #[test]
    fn check_learned_learnable_and_learned_returns_ok() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.learnable = true;
        assert_eq!(spell.check_learned(true), SpellCastResult::Ok);
    }

    #[test]
    fn check_learned_not_learnable_always_ok() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        // learnable = false by default
        assert_eq!(spell.check_learned(false), SpellCastResult::Ok);
    }

    // ---- check_weapon ----

    #[test]
    fn check_weapon_needed_but_missing_returns_error() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.need_weapon = true;
        assert_eq!(spell.check_weapon(false), SpellCastResult::NeedWeapon);
    }

    #[test]
    fn check_weapon_needed_and_present_returns_ok() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.need_weapon = true;
        assert_eq!(spell.check_weapon(true), SpellCastResult::Ok);
    }

    #[test]
    fn check_weapon_not_needed_always_ok() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        assert_eq!(spell.check_weapon(false), SpellCastResult::Ok);
    }

    // ---- post_cast_costs ----

    #[test]
    fn post_cast_costs_returns_flat_mana_and_soul() {
        let mut spell = Spell::new("Exura", 80, 1, vec![]);
        spell.soul_cost = 3;
        assert_eq!(spell.post_cast_costs(500), (80, 3));
    }

    #[test]
    fn post_cast_costs_percent_mana_and_zero_soul() {
        let mut spell = Spell::new("Exura", 0, 1, vec![]);
        spell.mana_percent = 10; // 10% of 200 = 20
        assert_eq!(spell.post_cast_costs(200), (20, 0));
    }

    // ---- Spell fields ----

    #[test]
    fn spell_default_flags_match_cpp_defaults() {
        let spell = Spell::new("Exura", 0, 0, vec![]);
        assert!(spell.enabled);
        assert!(spell.aggressive);
        assert!(!spell.premium);
        assert!(!spell.learnable);
        assert!(!spell.need_weapon);
        assert!(!spell.self_target);
        assert!(!spell.blocking_solid);
        assert!(!spell.blocking_creature);
        assert!(!spell.pz_lock);
        assert_eq!(spell.cooldown, 1000);
        assert_eq!(spell.group_cooldown, 1000);
        assert_eq!(spell.secondary_group_cooldown, 0);
        assert_eq!(spell.range, -1);
        assert_eq!(spell.group, SpellGroup::None);
        assert_eq!(spell.secondary_group, SpellGroup::None);
    }

    // ---- InstantSpell ----

    #[test]
    fn instant_spell_new_has_correct_defaults() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        let instant = InstantSpell::new(spell);
        assert!(!instant.need_direction);
        assert!(!instant.has_param);
        assert!(!instant.has_player_name_param);
        assert!(instant.check_line_of_sight);
        assert!(!instant.caster_target_or_direction);
    }

    #[test]
    fn instant_target_type_self_target() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.self_target = true;
        let instant = InstantSpell::new(spell);
        assert_eq!(instant.target_type(), InstantTargetType::SelfTarget);
    }

    #[test]
    fn instant_target_type_creature_when_need_target() {
        let mut spell = Spell::new("Exura Vita", 50, 1, vec![]);
        spell.need_target = true;
        let instant = InstantSpell::new(spell);
        assert_eq!(instant.target_type(), InstantTargetType::Creature);
    }

    #[test]
    fn instant_target_type_creature_when_caster_target_or_direction() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        let mut instant = InstantSpell::new(spell);
        instant.caster_target_or_direction = true;
        assert_eq!(instant.target_type(), InstantTargetType::Creature);
    }

    #[test]
    fn instant_target_type_position_when_need_direction() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        let mut instant = InstantSpell::new(spell);
        instant.need_direction = true;
        assert_eq!(instant.target_type(), InstantTargetType::Position);
    }

    #[test]
    fn instant_target_type_position_when_no_special_flag() {
        let spell = Spell::new("Exura", 50, 1, vec![]);
        let instant = InstantSpell::new(spell);
        // no self_target, need_target, need_direction, caster_target_or_direction
        assert_eq!(instant.target_type(), InstantTargetType::Position);
    }

    // ---- RuneSpell ----

    #[test]
    fn rune_spell_new_has_correct_defaults() {
        let spell = Spell::new("Exevo", 50, 1, vec![]);
        let rune = RuneSpell::new(spell, 2265);
        assert_eq!(rune.rune_item_id, 2265);
        assert_eq!(rune.charges, 0);
        assert!(!rune.has_charges);
    }

    #[test]
    fn rune_spell_set_charges_marks_has_charges() {
        let spell = Spell::new("Exevo", 50, 1, vec![]);
        let mut rune = RuneSpell::new(spell, 2265);
        rune.set_charges(5);
        assert_eq!(rune.charges, 5);
        assert!(rune.has_charges);
    }

    #[test]
    fn rune_spell_set_charges_zero_does_not_mark_has_charges() {
        let spell = Spell::new("Exevo", 50, 1, vec![]);
        let mut rune = RuneSpell::new(spell, 2265);
        rune.set_charges(0);
        assert_eq!(rune.charges, 0);
        assert!(!rune.has_charges);
    }

    // ---- Spells registry — instant spells ----

    #[test]
    fn spells_register_and_get_instant_spell() {
        let mut registry = Spells::new();
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.words = "exura".to_string();
        let instant = InstantSpell::new(spell);
        registry.register_instant(instant);
        assert!(registry.get_instant_spell("exura").is_some());
    }

    #[test]
    fn spells_get_instant_spell_unknown_returns_none() {
        let registry = Spells::new();
        assert!(registry.get_instant_spell("exura").is_none());
    }

    #[test]
    fn spells_get_instant_spell_by_name() {
        let mut registry = Spells::new();
        let spell = Spell::new("Great Healing", 200, 1, vec![]);
        let instant = InstantSpell::new(spell);
        registry.register_instant(instant);
        assert!(registry
            .get_instant_spell_by_name("Great Healing")
            .is_some());
    }

    #[test]
    fn spells_get_instant_spell_by_name_unknown_returns_none() {
        let registry = Spells::new();
        assert!(registry
            .get_instant_spell_by_name("Great Healing")
            .is_none());
    }

    // ---- Spells registry — rune spells ----

    #[test]
    fn spells_register_and_get_rune_spell() {
        let mut registry = Spells::new();
        let spell = Spell::new("Exevo Gran Mas Vis", 200, 1, vec![]);
        let rune = RuneSpell::new(spell, 2311);
        registry.register_rune(rune);
        assert!(registry.get_rune_spell(2311).is_some());
    }

    #[test]
    fn spells_get_rune_spell_unknown_item_id_returns_none() {
        let registry = Spells::new();
        assert!(registry.get_rune_spell(9999).is_none());
    }

    #[test]
    fn spells_get_rune_spell_by_name() {
        let mut registry = Spells::new();
        let spell = Spell::new("Explosion Rune", 200, 1, vec![]);
        let rune = RuneSpell::new(spell, 2311);
        registry.register_rune(rune);
        assert!(registry.get_rune_spell_by_name("Explosion Rune").is_some());
    }

    #[test]
    fn spells_get_rune_spell_by_name_unknown_returns_none() {
        let registry = Spells::new();
        assert!(registry.get_rune_spell_by_name("Unknown Rune").is_none());
    }

    #[test]
    fn spells_get_rune_spell_returns_correct_item_id() {
        let mut registry = Spells::new();
        let spell = Spell::new("Great Fireball", 200, 1, vec![]);
        let rune = RuneSpell::new(spell, 2304);
        registry.register_rune(rune);
        let found = registry.get_rune_spell(2304).unwrap();
        assert_eq!(found.rune_item_id, 2304);
    }

    // ---- Spell group and cooldown fields ----

    #[test]
    fn spell_can_set_group_attack() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.group = SpellGroup::Attack;
        assert_eq!(spell.group, SpellGroup::Attack);
    }

    #[test]
    fn spell_can_set_cooldown_fields() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.cooldown = 2000;
        spell.group_cooldown = 3000;
        spell.secondary_group_cooldown = 500;
        assert_eq!(spell.cooldown, 2000);
        assert_eq!(spell.group_cooldown, 3000);
        assert_eq!(spell.secondary_group_cooldown, 500);
    }

    #[test]
    fn spell_id_field_accessible() {
        let mut spell = Spell::new("Exura", 50, 1, vec![]);
        spell.spell_id = 42;
        assert_eq!(spell.spell_id, 42);
    }
}
