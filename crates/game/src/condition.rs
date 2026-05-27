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
