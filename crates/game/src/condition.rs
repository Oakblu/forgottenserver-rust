use std::time::{Duration, Instant};

/// Identifies which kind of effect a condition applies each tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionKind {
    // Damage-over-time
    Poison,
    Fire,
    Energy,
    Drown,
    Bleeding,
    LifeDrain,
    ManaDrain,
    // Healing
    Regeneration,
    GainMana,
    // Speed modifiers
    Haste,
    Paralyze,
    Slowed,
    Root,
    // Status effects — modifier is applied once on add; no per-tick effect
    Drunk,
    Invisible,
    Light,
    ManaShield,
    MagicShield,
    Pacified,
    Outfit,
    // Combat flags
    InfightPlayer,
    InfightMonster,
    Hunting,
    Channeling,
    // Exhaust gates
    Exhaust,
    ExhaustHeal,
    ExhaustYell,
    ExhaustSpell,
    // Misc
    Strengthened,
}

/// A ticking condition carrying enough metadata to apply its effect each interval.
#[derive(Debug, Clone)]
pub struct TickableCondition {
    pub kind: ConditionKind,
    pub ticks_remaining: u32,
    pub tick_interval: Duration,
    /// Absolute time at which the next tick effect fires.
    pub next_tick: Instant,
    pub damage_per_tick: i32,
    pub heal_per_tick: i32,
    pub speed_modifier: i32,
}

impl TickableCondition {
    pub fn new(
        kind: ConditionKind,
        ticks_remaining: u32,
        tick_interval_ms: u64,
        next_tick: Instant,
    ) -> Self {
        TickableCondition {
            kind,
            ticks_remaining,
            tick_interval: Duration::from_millis(tick_interval_ms),
            next_tick,
            damage_per_tick: 0,
            heal_per_tick: 0,
            speed_modifier: 0,
        }
    }

    pub fn with_damage(mut self, dmg: i32) -> Self {
        self.damage_per_tick = dmg;
        self
    }

    pub fn with_heal(mut self, hp: i32) -> Self {
        self.heal_per_tick = hp;
        self
    }

    pub fn with_speed_modifier(mut self, modifier: i32) -> Self {
        self.speed_modifier = modifier;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    // ── ConditionKind variants ──────────────────────────────────────────────

    #[test]
    fn condition_kind_damage_over_time_variants_exist() {
        let kinds = [
            ConditionKind::Poison,
            ConditionKind::Fire,
            ConditionKind::Energy,
            ConditionKind::Drown,
            ConditionKind::Bleeding,
            ConditionKind::LifeDrain,
            ConditionKind::ManaDrain,
        ];
        assert_eq!(kinds.len(), 7);
    }

    #[test]
    fn condition_kind_healing_variants_exist() {
        let kinds = [ConditionKind::Regeneration, ConditionKind::GainMana];
        assert_eq!(kinds.len(), 2);
    }

    #[test]
    fn condition_kind_speed_modifier_variants_exist() {
        let kinds = [
            ConditionKind::Haste,
            ConditionKind::Paralyze,
            ConditionKind::Slowed,
            ConditionKind::Root,
        ];
        assert_eq!(kinds.len(), 4);
    }

    #[test]
    fn condition_kind_status_effect_variants_exist() {
        let kinds = [
            ConditionKind::Drunk,
            ConditionKind::Invisible,
            ConditionKind::Light,
            ConditionKind::ManaShield,
            ConditionKind::MagicShield,
            ConditionKind::Pacified,
            ConditionKind::Outfit,
        ];
        assert_eq!(kinds.len(), 7);
    }

    #[test]
    fn condition_kind_combat_flag_variants_exist() {
        let kinds = [
            ConditionKind::InfightPlayer,
            ConditionKind::InfightMonster,
            ConditionKind::Hunting,
            ConditionKind::Channeling,
        ];
        assert_eq!(kinds.len(), 4);
    }

    #[test]
    fn condition_kind_exhaust_variants_exist() {
        let kinds = [
            ConditionKind::Exhaust,
            ConditionKind::ExhaustHeal,
            ConditionKind::ExhaustYell,
            ConditionKind::ExhaustSpell,
        ];
        assert_eq!(kinds.len(), 4);
    }

    #[test]
    fn condition_kind_misc_variants_exist() {
        let _kind = ConditionKind::Strengthened;
    }

    #[test]
    fn condition_kind_derives_debug_clone_copy_eq() {
        let k = ConditionKind::Fire;
        let k2 = k;
        assert_eq!(k, k2);
        assert_eq!(format!("{k:?}"), "Fire");
    }

    // ── TickableCondition::new ─────────────────────────────────────────────

    #[test]
    fn tickable_condition_new_sets_kind_and_ticks() {
        let now = Instant::now();
        let tc = TickableCondition::new(ConditionKind::Fire, 10, 3000, now);
        assert_eq!(tc.kind, ConditionKind::Fire);
        assert_eq!(tc.ticks_remaining, 10);
        assert_eq!(tc.tick_interval, std::time::Duration::from_millis(3000));
        assert_eq!(tc.damage_per_tick, 0);
        assert_eq!(tc.heal_per_tick, 0);
        assert_eq!(tc.speed_modifier, 0);
    }

    #[test]
    fn tickable_condition_new_stores_next_tick() {
        let now = Instant::now();
        let tc = TickableCondition::new(ConditionKind::Poison, 5, 1000, now);
        // next_tick should equal the instant we passed in.
        assert_eq!(tc.next_tick, now);
    }

    // ── TickableCondition::with_damage ─────────────────────────────────────

    #[test]
    fn tickable_condition_with_damage_sets_value() {
        let tc = TickableCondition::new(ConditionKind::Fire, 5, 1000, Instant::now())
            .with_damage(42);
        assert_eq!(tc.damage_per_tick, 42);
        assert_eq!(tc.heal_per_tick, 0);
        assert_eq!(tc.speed_modifier, 0);
    }

    #[test]
    fn tickable_condition_with_damage_negative_value() {
        let tc = TickableCondition::new(ConditionKind::Energy, 3, 2000, Instant::now())
            .with_damage(-15);
        assert_eq!(tc.damage_per_tick, -15);
    }

    // ── TickableCondition::with_heal ───────────────────────────────────────

    #[test]
    fn tickable_condition_with_heal_sets_value() {
        let tc = TickableCondition::new(ConditionKind::Regeneration, 20, 1000, Instant::now())
            .with_heal(8);
        assert_eq!(tc.heal_per_tick, 8);
        assert_eq!(tc.damage_per_tick, 0);
    }

    #[test]
    fn tickable_condition_with_heal_zero() {
        let tc = TickableCondition::new(ConditionKind::GainMana, 10, 500, Instant::now())
            .with_heal(0);
        assert_eq!(tc.heal_per_tick, 0);
    }

    // ── TickableCondition::with_speed_modifier ─────────────────────────────

    #[test]
    fn tickable_condition_with_speed_modifier_positive() {
        let tc = TickableCondition::new(ConditionKind::Haste, 30, 1000, Instant::now())
            .with_speed_modifier(100);
        assert_eq!(tc.speed_modifier, 100);
        assert_eq!(tc.damage_per_tick, 0);
        assert_eq!(tc.heal_per_tick, 0);
    }

    #[test]
    fn tickable_condition_with_speed_modifier_negative() {
        let tc = TickableCondition::new(ConditionKind::Paralyze, 10, 1000, Instant::now())
            .with_speed_modifier(-50);
        assert_eq!(tc.speed_modifier, -50);
    }

    // ── Builder chaining ───────────────────────────────────────────────────

    #[test]
    fn tickable_condition_builder_chain_all() {
        let tc = TickableCondition::new(ConditionKind::Drown, 5, 2000, Instant::now())
            .with_damage(10)
            .with_heal(3)
            .with_speed_modifier(-20);
        assert_eq!(tc.damage_per_tick, 10);
        assert_eq!(tc.heal_per_tick, 3);
        assert_eq!(tc.speed_modifier, -20);
    }

    // ── Clone and Debug derives ────────────────────────────────────────────

    #[test]
    fn tickable_condition_debug_shows_kind() {
        let tc = TickableCondition::new(ConditionKind::Bleeding, 1, 1000, Instant::now());
        let debug = format!("{tc:?}");
        assert!(debug.contains("Bleeding"));
    }

    #[test]
    fn tickable_condition_clone_is_independent() {
        let now = Instant::now();
        let tc = TickableCondition::new(ConditionKind::Fire, 5, 1000, now).with_damage(20);
        let mut tc2 = tc.clone();
        tc2.damage_per_tick = 99;
        // Original is unchanged.
        assert_eq!(tc.damage_per_tick, 20);
        assert_eq!(tc2.damage_per_tick, 99);
    }
}
