// Copyright 2023 The Forgotten Server Authors. All rights reserved.
// Use of this source code is governed by the GPL-2.0 License that can be found in the LICENSE file.

//! Condition system — migrated from condition.h / condition.cpp
//!
//! Conditions are status effects applied to creatures (poison, fire, haste, etc.).
//! This module provides pure-data structs and logic functions decoupled from the
//! Creature/Player types (which are not yet migrated).  Creature-side effects
//! (applying damage, sending messages, etc.) are intentionally left as stubs /
//! no-ops until the creature layer exists.

use forgottenserver_common::enums::{ConditionId, ConditionParam, ConditionTypeFlags};

// ---------------------------------------------------------------------------
// ConditionAttr — serialisation attribute tags (mirrors ConditionAttr_t)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConditionAttr {
    Type = 1,
    Id = 2,
    Ticks = 3,
    HealthTicks = 4,
    HealthGain = 5,
    ManaTicks = 6,
    ManaGain = 7,
    Delayed = 8,
    Owner = 9,
    IntervalData = 10,
    SpeedDelta = 11,
    FormulaMina = 12,
    FormulaMinb = 13,
    FormulaMaxa = 14,
    FormulaMaxb = 15,
    LightColor = 16,
    LightLevel = 17,
    LightTicks = 18,
    LightInterval = 19,
    SoulTicks = 20,
    SoulGain = 21,
    Skills = 22,
    Stats = 23,
    Outfit = 24,
    PeriodDamage = 25,
    IsBuff = 26,
    SubId = 27,
    IsAggressive = 28,
    DisableDefense = 29,
    SpecialSkills = 30,
    ManaShieldBreakableMana = 31,
    ManaShieldBreakableMaxMana = 32,
    End = 254,
}

// ---------------------------------------------------------------------------
// IntervalInfo — one entry in a damage schedule
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntervalInfo {
    pub time_left: i32,
    pub value: i32,
    pub interval: i32,
}

// ---------------------------------------------------------------------------
// ConditionBase — fields common to every condition variant
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionBase {
    pub id: ConditionId,
    /// Stores one of the `ConditionTypeFlags` constants (i32 bit-field).
    pub condition_type: i32,
    /// Remaining ticks.  -1 = permanent (never expires).
    pub ticks: i32,
    /// Absolute end-time (milliseconds since epoch / OTSYS_TIME).
    /// Used only when ticks != -1.
    pub end_time: i64,
    pub sub_id: u32,
    pub is_buff: bool,
    pub aggressive: bool,
}

impl ConditionBase {
    pub fn new(
        id: ConditionId,
        condition_type: i32,
        ticks: i32,
        buff: bool,
        sub_id: u32,
        aggressive: bool,
    ) -> Self {
        let end_time = if ticks == -1 { i64::MAX } else { 0 };
        Self {
            id,
            condition_type,
            ticks,
            end_time,
            sub_id,
            is_buff: buff,
            aggressive,
        }
    }

    /// Returns true while the condition is still active.
    /// Mirrors `Condition::executeCondition` — reduces ticks by interval
    /// and returns false when the condition has expired.
    ///
    /// NOTE: We cannot compare against OTSYS_TIME() here (no wall-clock), so
    /// we use ticks-only logic.  The caller is responsible for driving the
    /// tick budget correctly.
    pub fn tick(&mut self, interval: i32) -> bool {
        if self.ticks == -1 {
            return true; // permanent condition
        }
        self.ticks = std::cmp::max(0, self.ticks - interval);
        self.ticks > 0
    }

    pub fn set_ticks(&mut self, new_ticks: i32) {
        self.ticks = new_ticks;
        // end_time would normally be set to new_ticks + OTSYS_TIME()
        // We skip wall-clock since we have no timer dependency here.
    }

    pub fn is_persistent(&self) -> bool {
        if self.ticks == -1 {
            return false;
        }
        matches!(self.id, ConditionId::Default | ConditionId::Combat)
            || self.condition_type == ConditionTypeFlags::MUTED
    }

    /// Returns the icon-flag bits the client should display for this
    /// condition. Mirrors C++ `Condition::getIcons() const { return
    /// isBuff ? ICON_PARTY_BUFF : 0; }`. The `ICON_PARTY_BUFF` constant
    /// is `1 << 12` (see C++ `const.h::PlayerIcon_t`).
    pub fn get_icons(&self) -> u32 {
        if self.is_buff {
            1 << 12 // ICON_PARTY_BUFF
        } else {
            0
        }
    }

    /// Generic update: returns true if the incoming condition should replace
    /// (extend) this one (mirrors `Condition::updateCondition`).
    pub fn should_update(&self, other: &ConditionBase) -> bool {
        if self.condition_type != other.condition_type {
            return false;
        }
        if self.ticks == -1 && other.ticks > 0 {
            return false;
        }
        // If the incoming condition lasts longer, update
        other.ticks >= 0 && other.ticks > self.ticks
    }

    // -- param helpers -------------------------------------------------------

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::Ticks => {
                self.ticks = value as i32;
                true
            }
            ConditionParam::BuffSpell => {
                self.is_buff = value != 0;
                true
            }
            ConditionParam::SubId => {
                self.sub_id = value as u32;
                true
            }
            ConditionParam::Aggressive => {
                self.aggressive = value != 0;
                true
            }
            _ => false,
        }
    }

    pub fn get_param(&self, param: ConditionParam) -> i64 {
        match param {
            ConditionParam::Ticks => self.ticks as i64,
            ConditionParam::BuffSpell => i64::from(self.is_buff),
            ConditionParam::SubId => self.sub_id as i64,
            _ => i32::MAX as i64,
        }
    }

    // -- serialise -----------------------------------------------------------

    pub fn serialize(&self, out: &mut Vec<u8>) {
        push_u8(out, ConditionAttr::Type as u8);
        push_i32(out, self.condition_type);
        push_u8(out, ConditionAttr::Id as u8);
        push_i32(out, self.id as i32);
        push_u8(out, ConditionAttr::Ticks as u8);
        push_i32(out, self.ticks);
        push_u8(out, ConditionAttr::IsBuff as u8);
        push_u8(out, self.is_buff as u8);
        push_u8(out, ConditionAttr::SubId as u8);
        push_u32(out, self.sub_id);
        push_u8(out, ConditionAttr::IsAggressive as u8);
        push_u8(out, self.aggressive as u8);
    }

    /// Returns (ConditionBase, bytes_consumed) or None on parse failure.
    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let mut pos = 0usize;

        macro_rules! read_u8 {
            () => {{
                if pos >= data.len() {
                    return None;
                }
                let v = data[pos];
                pos += 1;
                v
            }};
        }
        macro_rules! read_u32 {
            () => {{
                if pos + 4 > data.len() {
                    return None;
                }
                let v = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                pos += 4;
                v
            }};
        }
        macro_rules! read_i32 {
            () => {{
                if pos + 4 > data.len() {
                    return None;
                }
                let v = i32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                pos += 4;
                v
            }};
        }

        let mut condition_type = 0i32;
        let mut id_raw = 0i32;
        let mut ticks = 0i32;
        let mut is_buff = false;
        let mut sub_id = 0u32;
        let mut aggressive = false;

        loop {
            let attr = read_u8!();
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::Type as u8 => condition_type = read_i32!(),
                a if a == ConditionAttr::Id as u8 => id_raw = read_i32!(),
                a if a == ConditionAttr::Ticks as u8 => ticks = read_i32!(),
                a if a == ConditionAttr::IsBuff as u8 => is_buff = read_u8!() != 0,
                a if a == ConditionAttr::SubId as u8 => sub_id = read_u32!(),
                a if a == ConditionAttr::IsAggressive as u8 => aggressive = read_u8!() != 0,
                _ => {
                    // Unknown attr — put the byte back so the caller can read it
                    pos -= 1;
                    break;
                }
            }
        }

        let id = condition_id_from_i32(id_raw)?;
        let end_time = if ticks == -1 { i64::MAX } else { 0 };
        Some((
            ConditionBase {
                id,
                condition_type,
                ticks,
                end_time,
                sub_id,
                is_buff,
                aggressive,
            },
            pos,
        ))
    }
}

fn condition_id_from_i32(v: i32) -> Option<ConditionId> {
    match v {
        -1 => Some(ConditionId::Default),
        0 => Some(ConditionId::Combat),
        1 => Some(ConditionId::Head),
        2 => Some(ConditionId::Necklace),
        3 => Some(ConditionId::Backpack),
        4 => Some(ConditionId::Armor),
        5 => Some(ConditionId::Right),
        6 => Some(ConditionId::Left),
        7 => Some(ConditionId::Legs),
        8 => Some(ConditionId::Feet),
        9 => Some(ConditionId::Ring),
        10 => Some(ConditionId::Ammo),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// ConditionRegeneration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionRegeneration {
    pub base: ConditionBase,
    pub health_ticks: u32,
    pub mana_ticks: u32,
    pub health_gain: u32,
    pub mana_gain: u32,
    // internal accumulators (not serialised)
    pub internal_health_ticks: u32,
    pub internal_mana_ticks: u32,
}

impl ConditionRegeneration {
    pub fn new(id: ConditionId, ticks: i32, buff: bool, sub_id: u32, aggressive: bool) -> Self {
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::REGENERATION,
                ticks,
                buff,
                sub_id,
                aggressive,
            ),
            health_ticks: 1000,
            mana_ticks: 1000,
            health_gain: 0,
            mana_gain: 0,
            internal_health_ticks: 0,
            internal_mana_ticks: 0,
        }
    }

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::HealthGain => {
                self.health_gain = value as u32;
                true
            }
            ConditionParam::HealthTicks => {
                self.health_ticks = value as u32;
                true
            }
            ConditionParam::ManaGain => {
                self.mana_gain = value as u32;
                true
            }
            ConditionParam::ManaTicks => {
                self.mana_ticks = value as u32;
                true
            }
            p => self.base.set_param(p, value),
        }
    }

    pub fn get_param(&self, param: ConditionParam) -> i64 {
        match param {
            ConditionParam::HealthGain => self.health_gain as i64,
            ConditionParam::HealthTicks => self.health_ticks as i64,
            ConditionParam::ManaGain => self.mana_gain as i64,
            ConditionParam::ManaTicks => self.mana_ticks as i64,
            p => self.base.get_param(p),
        }
    }

    /// Advance one tick.  Returns (still_alive, hp_gain, mana_gain).
    /// The caller is responsible for actually applying hp/mana to the creature.
    pub fn tick(&mut self, interval: i32) -> (bool, u32, u32) {
        let interval_u = interval as u32;
        self.internal_health_ticks += interval_u;
        self.internal_mana_ticks += interval_u;

        let hp = if self.internal_health_ticks >= self.health_ticks {
            self.internal_health_ticks = 0;
            self.health_gain
        } else {
            0
        };

        let mana = if self.internal_mana_ticks >= self.mana_ticks {
            self.internal_mana_ticks = 0;
            self.mana_gain
        } else {
            0
        };

        let alive = self.base.tick(interval);
        (alive, hp, mana)
    }

    /// Merge incoming condition (mirrors addCondition / updateCondition).
    pub fn add_condition(&mut self, other: &ConditionRegeneration) {
        if self.base.should_update(&other.base) {
            self.base.set_ticks(other.base.ticks);
            self.health_ticks = other.health_ticks;
            self.mana_ticks = other.mana_ticks;
            self.health_gain = other.health_gain;
            self.mana_gain = other.mana_gain;
        }
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        push_u8(out, ConditionAttr::HealthTicks as u8);
        push_u32(out, self.health_ticks);
        push_u8(out, ConditionAttr::HealthGain as u8);
        push_u32(out, self.health_gain);
        push_u8(out, ConditionAttr::ManaTicks as u8);
        push_u32(out, self.mana_ticks);
        push_u8(out, ConditionAttr::ManaGain as u8);
        push_u32(out, self.mana_gain);
        push_u8(out, ConditionAttr::End as u8);
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut health_ticks = 1000u32;
        let mut health_gain = 0u32;
        let mut mana_ticks = 1000u32;
        let mut mana_gain = 0u32;

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::HealthTicks as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    health_ticks = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::HealthGain as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    health_gain = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::ManaTicks as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    mana_ticks = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::ManaGain as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    mana_gain = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                _ => break,
            }
        }
        Some((
            Self {
                base,
                health_ticks,
                mana_ticks,
                health_gain,
                mana_gain,
                internal_health_ticks: 0,
                internal_mana_ticks: 0,
            },
            pos,
        ))
    }
}

// ---------------------------------------------------------------------------
// ConditionSoul
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionSoul {
    pub base: ConditionBase,
    pub soul_ticks: u32,
    pub soul_gain: u32,
    pub internal_soul_ticks: u32,
}

impl ConditionSoul {
    pub fn new(id: ConditionId, ticks: i32, buff: bool, sub_id: u32, aggressive: bool) -> Self {
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::SOUL,
                ticks,
                buff,
                sub_id,
                aggressive,
            ),
            soul_ticks: 0,
            soul_gain: 0,
            internal_soul_ticks: 0,
        }
    }

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::SoulGain => {
                self.soul_gain = value as u32;
                true
            }
            ConditionParam::SoulTicks => {
                self.soul_ticks = value as u32;
                true
            }
            p => self.base.set_param(p, value),
        }
    }

    pub fn get_param(&self, param: ConditionParam) -> i64 {
        match param {
            ConditionParam::SoulGain => self.soul_gain as i64,
            ConditionParam::SoulTicks => self.soul_ticks as i64,
            p => self.base.get_param(p),
        }
    }

    /// Advance one tick.  Returns (still_alive, soul_gain_amount).
    pub fn tick(&mut self, interval: i32) -> (bool, u32) {
        self.internal_soul_ticks += interval as u32;
        let soul = if self.soul_ticks > 0 && self.internal_soul_ticks >= self.soul_ticks {
            self.internal_soul_ticks = 0;
            self.soul_gain
        } else {
            0
        };
        let alive = self.base.tick(interval);
        (alive, soul)
    }

    pub fn add_condition(&mut self, other: &ConditionSoul) {
        if self.base.should_update(&other.base) {
            self.base.set_ticks(other.base.ticks);
            self.soul_ticks = other.soul_ticks;
            self.soul_gain = other.soul_gain;
        }
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        push_u8(out, ConditionAttr::SoulGain as u8);
        push_u32(out, self.soul_gain);
        push_u8(out, ConditionAttr::SoulTicks as u8);
        push_u32(out, self.soul_ticks);
        push_u8(out, ConditionAttr::End as u8);
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut soul_gain = 0u32;
        let mut soul_ticks = 0u32;

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::SoulGain as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    soul_gain = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::SoulTicks as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    soul_ticks = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                _ => break,
            }
        }
        Some((
            Self {
                base,
                soul_gain,
                soul_ticks,
                internal_soul_ticks: 0,
            },
            pos,
        ))
    }
}

// ---------------------------------------------------------------------------
// ConditionDamage (poison, fire, energy, bleeding, etc.)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionDamage {
    pub base: ConditionBase,
    pub max_damage: i32,
    pub min_damage: i32,
    pub start_damage: i32,
    pub period_damage: i32,
    pub period_damage_tick: i32,
    pub tick_interval: i32,
    pub init_damage: i32,
    pub force_update: bool,
    pub delayed: bool,
    pub field: bool,
    pub owner: u32,
    pub damage_list: Vec<IntervalInfo>,
}

impl ConditionDamage {
    pub fn new(
        id: ConditionId,
        condition_type: i32,
        buff: bool,
        sub_id: u32,
        aggressive: bool,
    ) -> Self {
        Self {
            base: ConditionBase::new(id, condition_type, 0, buff, sub_id, aggressive),
            max_damage: 0,
            min_damage: 0,
            start_damage: 0,
            period_damage: 0,
            period_damage_tick: 0,
            tick_interval: 2000,
            init_damage: 0,
            force_update: false,
            delayed: false,
            field: false,
            owner: 0,
            damage_list: Vec::new(),
        }
    }

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::Owner => {
                self.owner = value as u32;
                true
            }
            ConditionParam::ForceUpdate => {
                self.force_update = value != 0;
                true
            }
            ConditionParam::Delayed => {
                self.delayed = value != 0;
                true
            }
            ConditionParam::MaxValue => {
                self.max_damage = value.unsigned_abs() as i32;
                true
            }
            ConditionParam::MinValue => {
                self.min_damage = value.unsigned_abs() as i32;
                true
            }
            ConditionParam::StartValue => {
                self.start_damage = value.unsigned_abs() as i32;
                true
            }
            ConditionParam::TickInterval => {
                self.tick_interval = value.unsigned_abs() as i32;
                true
            }
            ConditionParam::PeriodicDamage => {
                self.period_damage = value as i32;
                true
            }
            ConditionParam::Field => {
                self.field = value != 0;
                true
            }
            p => self.base.set_param(p, value),
        }
    }

    pub fn get_param(&self, param: ConditionParam) -> i64 {
        match param {
            ConditionParam::Owner => self.owner as i64,
            ConditionParam::ForceUpdate => i64::from(self.force_update),
            ConditionParam::Delayed => i64::from(self.delayed),
            ConditionParam::MaxValue => self.max_damage as i64,
            ConditionParam::MinValue => self.min_damage as i64,
            ConditionParam::StartValue => self.start_damage as i64,
            ConditionParam::TickInterval => self.tick_interval as i64,
            ConditionParam::PeriodicDamage => self.period_damage as i64,
            ConditionParam::Field => i64::from(self.field),
            p => self.base.get_param(p),
        }
    }

    /// Add damage entries.
    /// rounds == -1 → periodic damage; otherwise schedule `rounds` entries.
    pub fn add_damage(&mut self, rounds: i32, time: i32, value: i32) -> bool {
        let time = std::cmp::max(time, 500); // EVENT_CREATURE_THINK_INTERVAL = 500ms
        if rounds == -1 {
            self.period_damage = value;
            self.tick_interval = time;
            self.base.ticks = -1;
            return true;
        }

        if self.period_damage > 0 {
            return false;
        }

        for _ in 0..rounds {
            let info = IntervalInfo {
                interval: time,
                time_left: time,
                value,
            };
            self.damage_list.push(info);
            if self.base.ticks != -1 {
                self.base.ticks += time;
            }
        }
        true
    }

    pub fn get_total_damage(&self) -> i32 {
        if self.period_damage != 0 {
            return self.period_damage.abs();
        }
        self.damage_list.iter().map(|d| d.value.abs()).sum()
    }

    /// Mirror of inline C++ setter:
    /// `void setInitDamage(int32_t initDamage) { this->initDamage = initDamage; }`.
    /// Direct assignment, no sign normalisation — matches header.
    pub fn set_init_damage(&mut self, init_damage: i32) {
        self.init_damage = init_damage;
    }

    /// Static helper — port of C++
    /// `ConditionDamage::generateDamageList(amount, start, list)`. Fills `list`
    /// with the same decaying schedule the C++ algorithm produces (see
    /// `condition.cpp:1512`). Used by the `<attribute key="field">` XML loader
    /// (items.cpp:1716) to expand `damage`+`start` into a sequence of
    /// per-round damage values.
    pub fn generate_damage_list(amount: i32, start: i32, list: &mut Vec<i32>) {
        let amount = amount.abs();
        let mut sum: i32 = 0;
        if start <= 0 || amount == 0 {
            return;
        }
        let mut i = start;
        while i > 0 {
            let n = start + 1 - i;
            let med = (n * amount) / start;
            if med == 0 {
                // C++ would divide-by-zero on x1/x2 below; bail to match observable
                // behaviour (the inner do/while in C++ is gated on med != 0 in
                // practice because start >= amount in every datapack call site).
                i -= 1;
                continue;
            }
            loop {
                sum += i;
                list.push(i);

                let x1 = (1.0 - ((sum as f64) + (i as f64)) / (med as f64)).abs();
                let x2 = (1.0 - ((sum as f64) / (med as f64))).abs();
                if x1 >= x2 {
                    break;
                }
            }
            i -= 1;
        }
    }

    /// Merge update logic (mirrors ConditionDamage::updateCondition).
    pub fn should_update(&self, other: &ConditionDamage) -> bool {
        if other.force_update {
            return true;
        }
        if self.base.ticks == -1 && other.base.ticks > 0 {
            return false;
        }
        other.get_total_damage() > self.get_total_damage()
    }

    /// Mirrors `ConditionDamage::getIcons` (condition.cpp:1470).
    pub fn get_icons(&self) -> u32 {
        use ConditionTypeFlags as T;
        let icons = self.base.get_icons();
        let ct = self.base.condition_type;
        if ct == T::FIRE {
            icons | (1 << 1) // ICON_BURN
        } else if ct == T::ENERGY {
            icons | (1 << 2) // ICON_ENERGY
        } else if ct == T::DROWN {
            icons | (1 << 8) // ICON_DROWNING
        } else if ct == T::POISON {
            icons | (1 << 0) // ICON_POISON
        } else if ct == T::FREEZING {
            icons | (1 << 9) // ICON_FREEZING
        } else if ct == T::DAZZLED {
            icons | (1 << 10) // ICON_DAZZLED
        } else if ct == T::CURSED {
            icons | (1 << 11) // ICON_CURSED
        } else if ct == T::BLEEDING {
            icons | (1 << 15) // ICON_BLEEDING
        } else {
            icons
        }
    }

    /// Mirrors `ConditionDamage::init` (condition.cpp:1267).
    /// Builds the damage_list from min/max/start if it is not yet populated.
    pub fn init(&mut self) -> bool {
        if self.period_damage != 0 {
            return true;
        }
        if !self.damage_list.is_empty() {
            return true;
        }
        self.base.set_ticks(0);
        let amount =
            forgottenserver_common::tools::uniform_random(self.min_damage, self.max_damage);
        if amount != 0 {
            if self.start_damage > self.max_damage {
                self.start_damage = self.max_damage;
            } else if self.start_damage == 0 {
                self.start_damage = std::cmp::max(1, (amount as f64 / 20.0).ceil() as i32);
            }
            let mut list = Vec::new();
            Self::generate_damage_list(amount, self.start_damage, &mut list);
            let tick = self.tick_interval;
            for value in list {
                self.add_damage(1, tick, -value);
            }
        }
        !self.damage_list.is_empty()
    }

    /// Mirrors `ConditionDamage::getNextDamage` (condition.cpp:1355).
    /// Returns the next damage value without consuming the entry when ticks == -1.
    pub fn get_next_damage(&mut self) -> Option<i32> {
        if self.period_damage != 0 {
            return Some(self.period_damage);
        }
        if let Some(info) = self.damage_list.first() {
            let damage = info.value;
            if self.base.ticks != -1 {
                self.damage_list.remove(0);
            }
            return Some(damage);
        }
        None
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        push_u8(out, ConditionAttr::Delayed as u8);
        push_u8(out, self.delayed as u8);
        push_u8(out, ConditionAttr::PeriodDamage as u8);
        push_i32(out, self.period_damage);
        for info in &self.damage_list {
            push_u8(out, ConditionAttr::IntervalData as u8);
            push_i32(out, info.time_left);
            push_i32(out, info.value);
            push_i32(out, info.interval);
        }
        push_u8(out, ConditionAttr::End as u8);
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut delayed = false;
        let mut period_damage = 0i32;
        let mut damage_list: Vec<IntervalInfo> = Vec::new();

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::Delayed as u8 => {
                    if pos >= data.len() {
                        return None;
                    }
                    delayed = data[pos] != 0;
                    pos += 1;
                }
                a if a == ConditionAttr::PeriodDamage as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    period_damage = i32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::IntervalData as u8 => {
                    if pos + 12 > data.len() {
                        return None;
                    }
                    let time_left = i32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    let value = i32::from_le_bytes(data[pos + 4..pos + 8].try_into().ok()?);
                    let interval = i32::from_le_bytes(data[pos + 8..pos + 12].try_into().ok()?);
                    pos += 12;
                    damage_list.push(IntervalInfo {
                        time_left,
                        value,
                        interval,
                    });
                }
                _ => break,
            }
        }
        let mut cond = Self {
            base,
            max_damage: 0,
            min_damage: 0,
            start_damage: 0,
            period_damage,
            period_damage_tick: 0,
            tick_interval: 2000,
            init_damage: 0,
            force_update: false,
            delayed,
            field: false,
            owner: 0,
            damage_list,
        };
        // Restore ticks from damage list (mirrors C++ unserializeProp INTERVALDATA)
        if cond.base.ticks != -1 {
            for info in &cond.damage_list {
                cond.base.ticks += info.interval;
            }
        }
        Some((cond, pos))
    }
}

// ---------------------------------------------------------------------------
// ConditionSpeed (haste / paralyze)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionSpeed {
    pub base: ConditionBase,
    pub speed_delta: i32,
    pub formula_mina: f32,
    pub formula_minb: f32,
    pub formula_maxa: f32,
    pub formula_maxb: f32,
}

impl ConditionSpeed {
    pub fn new(
        id: ConditionId,
        condition_type: i32,
        ticks: i32,
        buff: bool,
        sub_id: u32,
        speed_delta: i32,
        aggressive: bool,
    ) -> Self {
        Self {
            base: ConditionBase::new(id, condition_type, ticks, buff, sub_id, aggressive),
            speed_delta,
            formula_mina: 0.0,
            formula_minb: 0.0,
            formula_maxa: 0.0,
            formula_maxb: 0.0,
        }
    }

    pub fn set_formula_vars(&mut self, mina: f32, minb: f32, maxa: f32, maxb: f32) {
        self.formula_mina = mina;
        self.formula_minb = minb;
        self.formula_maxa = maxa;
        self.formula_maxb = maxb;
    }

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::Speed => {
                self.speed_delta = value as i32;
                // Mirror C++: positive speed → HASTE, negative → PARALYZE
                if value > 0 {
                    self.base.condition_type = ConditionTypeFlags::HASTE;
                } else {
                    self.base.condition_type = ConditionTypeFlags::PARALYZE;
                }
                true
            }
            p => self.base.set_param(p, value),
        }
    }

    pub fn get_param(&self, param: ConditionParam) -> i64 {
        match param {
            ConditionParam::Speed => self.speed_delta as i64,
            p => self.base.get_param(p),
        }
    }

    /// Mirrors `ConditionSpeed::getIcons` (condition.cpp:1662).
    pub fn get_icons(&self) -> u32 {
        use ConditionTypeFlags as T;
        let icons = self.base.get_icons();
        if self.base.condition_type == T::HASTE {
            icons | (1 << 6) // ICON_HASTE
        } else if self.base.condition_type == T::PARALYZE {
            icons | (1 << 5) // ICON_PARALYZE
        } else {
            icons
        }
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    /// Merge an incoming speed condition.
    /// C++ always updates ticks (no "longer wins" guard) and applies the
    /// delta difference so the net speed change on the creature stays correct.
    /// Returns the net speed change that the caller should apply to the creature
    /// (positive = faster, negative = slower).
    pub fn add_condition(&mut self, other: &ConditionSpeed) -> i32 {
        if self.base.condition_type != other.base.condition_type {
            return 0;
        }
        if self.base.ticks == -1 && other.base.ticks > 0 {
            return 0;
        }
        let old_speed_delta = self.speed_delta;
        self.base.set_ticks(other.base.ticks);
        self.speed_delta = other.speed_delta;
        self.formula_mina = other.formula_mina;
        self.formula_minb = other.formula_minb;
        self.formula_maxa = other.formula_maxa;
        self.formula_maxb = other.formula_maxb;
        // Net creature speed change = new - old
        self.speed_delta - old_speed_delta
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        push_u8(out, ConditionAttr::SpeedDelta as u8);
        push_i32(out, self.speed_delta);
        push_u8(out, ConditionAttr::FormulaMina as u8);
        out.extend_from_slice(&self.formula_mina.to_le_bytes());
        push_u8(out, ConditionAttr::FormulaMinb as u8);
        out.extend_from_slice(&self.formula_minb.to_le_bytes());
        push_u8(out, ConditionAttr::FormulaMaxa as u8);
        out.extend_from_slice(&self.formula_maxa.to_le_bytes());
        push_u8(out, ConditionAttr::FormulaMaxb as u8);
        out.extend_from_slice(&self.formula_maxb.to_le_bytes());
        push_u8(out, ConditionAttr::End as u8);
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut speed_delta = 0i32;
        let mut formula_mina = 0.0f32;
        let mut formula_minb = 0.0f32;
        let mut formula_maxa = 0.0f32;
        let mut formula_maxb = 0.0f32;

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::SpeedDelta as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    speed_delta = i32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::FormulaMina as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    formula_mina = f32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::FormulaMinb as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    formula_minb = f32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::FormulaMaxa as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    formula_maxa = f32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::FormulaMaxb as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    formula_maxb = f32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                _ => break,
            }
        }
        Some((
            Self {
                base,
                speed_delta,
                formula_mina,
                formula_minb,
                formula_maxa,
                formula_maxb,
            },
            pos,
        ))
    }
}

// ---------------------------------------------------------------------------
// ConditionLight
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionLight {
    pub base: ConditionBase,
    pub light_level: u8,
    pub light_color: u8,
    pub internal_light_ticks: u32,
    pub light_change_interval: u32,
}

impl ConditionLight {
    pub fn new(
        id: ConditionId,
        ticks: i32,
        buff: bool,
        sub_id: u32,
        level: u8,
        color: u8,
        aggressive: bool,
    ) -> Self {
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::LIGHT,
                ticks,
                buff,
                sub_id,
                aggressive,
            ),
            light_level: level,
            light_color: color,
            internal_light_ticks: 0,
            light_change_interval: 0,
        }
    }

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::LightLevel => {
                self.light_level = value as u8;
                true
            }
            ConditionParam::LightColor => {
                self.light_color = value as u8;
                true
            }
            p => self.base.set_param(p, value),
        }
    }

    pub fn get_param(&self, param: ConditionParam) -> i64 {
        match param {
            ConditionParam::LightLevel => self.light_level as i64,
            ConditionParam::LightColor => self.light_color as i64,
            p => self.base.get_param(p),
        }
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    /// Merge incoming light condition (mirrors ConditionLight::addCondition).
    /// Recalculates `light_change_interval` from new ticks / level.
    pub fn add_condition(&mut self, other: &ConditionLight) {
        if !self.base.should_update(&other.base) {
            return;
        }
        self.base.set_ticks(other.base.ticks);
        self.light_level = other.light_level;
        self.light_color = other.light_color;
        let level = std::cmp::max(1u32, self.light_level as u32);
        self.light_change_interval = if self.base.ticks > 0 {
            self.base.ticks as u32 / level
        } else {
            0
        };
        self.internal_light_ticks = 0;
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        // C++ writes u32 for color and level (for forward compatibility)
        push_u8(out, ConditionAttr::LightColor as u8);
        push_u32(out, self.light_color as u32);
        push_u8(out, ConditionAttr::LightLevel as u8);
        push_u32(out, self.light_level as u32);
        push_u8(out, ConditionAttr::LightTicks as u8);
        push_u32(out, self.internal_light_ticks);
        push_u8(out, ConditionAttr::LightInterval as u8);
        push_u32(out, self.light_change_interval);
        push_u8(out, ConditionAttr::End as u8);
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut light_color = 0u8;
        let mut light_level = 0u8;
        let mut internal_light_ticks = 0u32;
        let mut light_change_interval = 0u32;

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::LightColor as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    light_color = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as u8;
                    pos += 4;
                }
                a if a == ConditionAttr::LightLevel as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    light_level = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?) as u8;
                    pos += 4;
                }
                a if a == ConditionAttr::LightTicks as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    internal_light_ticks = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                a if a == ConditionAttr::LightInterval as u8 => {
                    if pos + 4 > data.len() {
                        return None;
                    }
                    light_change_interval = u32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    pos += 4;
                }
                _ => break,
            }
        }
        Some((
            Self {
                base,
                light_level,
                light_color,
                internal_light_ticks,
                light_change_interval,
            },
            pos,
        ))
    }
}

// ---------------------------------------------------------------------------
// ConditionManaShield
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionManaShield {
    pub base: ConditionBase,
    pub mana_shield: u16,
    pub max_mana_shield: u16,
}

impl ConditionManaShield {
    pub fn new(id: ConditionId, ticks: i32, buff: bool, sub_id: u32) -> Self {
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::MANASHIELD_BREAKABLE,
                ticks,
                buff,
                sub_id,
                false,
            ),
            mana_shield: 0,
            max_mana_shield: 0,
        }
    }

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::ManaShieldBreakable => {
                self.mana_shield = value as u16;
                self.max_mana_shield = value as u16;
                true
            }
            p => self.base.set_param(p, value),
        }
    }

    pub fn absorb_damage(&mut self, damage: i32) -> i32 {
        // Returns the remaining damage after the shield absorbs what it can.
        if self.mana_shield == 0 {
            return damage;
        }
        let absorbed = std::cmp::min(self.mana_shield as i32, damage);
        self.mana_shield -= absorbed as u16;
        damage - absorbed
    }

    /// Mirrors ConditionManaShield::onDamageTaken.
    /// Returns the overflow damage that the shield could NOT absorb (excess
    /// over the current shield value).  Always drains shield to 0 on overflow.
    pub fn on_damage_taken(&mut self, mana_change: i32) -> i32 {
        if mana_change > self.mana_shield as i32 {
            let overflow = mana_change - self.mana_shield as i32;
            self.mana_shield = 0;
            overflow
        } else {
            self.mana_shield -= mana_change as u16;
            0
        }
    }

    /// Merge incoming mana shield condition.
    /// C++ always replaces the shield value (no should_update guard).
    pub fn add_condition(&mut self, other: &ConditionManaShield) {
        self.base.set_ticks(other.base.ticks);
        self.mana_shield = other.mana_shield;
        self.max_mana_shield = other.mana_shield;
    }

    /// Mirrors `ConditionManaShield::getIcons` (condition.cpp:2087).
    pub fn get_icons(&self) -> u32 {
        let icons = self.base.get_icons();
        if self.base.condition_type == ConditionTypeFlags::MANASHIELD_BREAKABLE {
            icons | (1 << 26) // ICON_MANASHIELD_BREAKABLE
        } else {
            icons
        }
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        push_u8(out, ConditionAttr::ManaShieldBreakableMana as u8);
        push_u16(out, self.mana_shield);
        push_u8(out, ConditionAttr::ManaShieldBreakableMaxMana as u8);
        push_u16(out, self.max_mana_shield);
        push_u8(out, ConditionAttr::End as u8);
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut mana_shield = 0u16;
        let mut max_mana_shield = 0u16;

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::ManaShieldBreakableMana as u8 => {
                    if pos + 2 > data.len() {
                        return None;
                    }
                    mana_shield = u16::from_le_bytes(data[pos..pos + 2].try_into().ok()?);
                    pos += 2;
                }
                a if a == ConditionAttr::ManaShieldBreakableMaxMana as u8 => {
                    if pos + 2 > data.len() {
                        return None;
                    }
                    max_mana_shield = u16::from_le_bytes(data[pos..pos + 2].try_into().ok()?);
                    pos += 2;
                }
                _ => break,
            }
        }
        Some((
            Self {
                base,
                mana_shield,
                max_mana_shield,
            },
            pos,
        ))
    }
}

// ---------------------------------------------------------------------------
// ConditionOutfit — stores a cosmetic outfit and restores it on end
// ---------------------------------------------------------------------------

/// Minimal representation of an Outfit_t (look type + addons).
/// Creature effects (g_game.internalCreatureChangeOutfit) are caller-side.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Outfit {
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
    pub look_mount: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionOutfit {
    pub base: ConditionBase,
    pub outfit: Outfit,
}

impl ConditionOutfit {
    pub fn new(id: ConditionId, ticks: i32, buff: bool, sub_id: u32, aggressive: bool) -> Self {
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::OUTFIT,
                ticks,
                buff,
                sub_id,
                aggressive,
            ),
            outfit: Outfit::default(),
        }
    }

    pub fn set_outfit(&mut self, outfit: Outfit) {
        self.outfit = outfit;
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    /// Merge incoming outfit condition (mirrors ConditionOutfit::addCondition).
    /// Returns true if the outfit was replaced.
    pub fn add_condition(&mut self, other: &ConditionOutfit) -> bool {
        if !self.base.should_update(&other.base) {
            return false;
        }
        self.base.set_ticks(other.base.ticks);
        self.outfit = other.outfit.clone();
        true
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        push_u8(out, ConditionAttr::Outfit as u8);
        // Outfit serialised as its fields (matches C++ Outfit_t layout)
        push_u16(out, self.outfit.look_type);
        push_u8(out, self.outfit.look_head);
        push_u8(out, self.outfit.look_body);
        push_u8(out, self.outfit.look_legs);
        push_u8(out, self.outfit.look_feet);
        push_u8(out, self.outfit.look_addons);
        push_u16(out, self.outfit.look_mount);
        push_u8(out, ConditionAttr::End as u8);
    }

    /// Mirror of C++ `ConditionOutfit::unserializeProp` — reads a
    /// `CONDITIONATTR_OUTFIT` block (Outfit_t raw bytes) into `self.outfit`.
    /// Falls through to `ConditionBase::deserialize` for shared attrs.
    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut outfit = Outfit::default();

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            if attr == ConditionAttr::Outfit as u8 {
                if pos + 9 > data.len() {
                    return None;
                }
                outfit.look_type = u16::from_le_bytes(data[pos..pos + 2].try_into().ok()?);
                pos += 2;
                outfit.look_head = data[pos];
                pos += 1;
                outfit.look_body = data[pos];
                pos += 1;
                outfit.look_legs = data[pos];
                pos += 1;
                outfit.look_feet = data[pos];
                pos += 1;
                outfit.look_addons = data[pos];
                pos += 1;
                outfit.look_mount = u16::from_le_bytes(data[pos..pos + 2].try_into().ok()?);
                pos += 2;
            } else {
                break;
            }
        }
        Some((Self { base, outfit }, pos))
    }
}

// ---------------------------------------------------------------------------
// ConditionInvisible — sets / clears the invisible flag on the creature
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionInvisible {
    pub base: ConditionBase,
}

impl ConditionInvisible {
    pub fn new(id: ConditionId, ticks: i32, buff: bool, sub_id: u32, aggressive: bool) -> Self {
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::INVISIBLE,
                ticks,
                buff,
                sub_id,
                aggressive,
            ),
        }
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    /// Merge (extends ticks like ConditionGeneric).
    pub fn add_condition(&mut self, other: &ConditionInvisible) {
        if self.base.should_update(&other.base) {
            self.base.set_ticks(other.base.ticks);
        }
    }
}

// ---------------------------------------------------------------------------
// ConditionDrunk — sets drunkenness on start, clears on end
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionDrunk {
    pub base: ConditionBase,
    /// Drunkenness level (0–255); default 25 mirrors C++.
    pub drunkenness: u8,
}

impl ConditionDrunk {
    pub fn new(
        id: ConditionId,
        ticks: i32,
        buff: bool,
        sub_id: u32,
        drunkenness: u8,
        aggressive: bool,
    ) -> Self {
        let actual = if drunkenness == 0 { 25 } else { drunkenness };
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::DRUNK,
                ticks,
                buff,
                sub_id,
                aggressive,
            ),
            drunkenness: actual,
        }
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    /// Merge: only replaces if the incoming drunkenness is HIGHER (C++ updateCondition).
    pub fn add_condition(&mut self, other: &ConditionDrunk) -> bool {
        if other.drunkenness > self.drunkenness {
            self.base.set_ticks(other.base.ticks);
            self.drunkenness = other.drunkenness;
            true
        } else {
            false
        }
    }

    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        match param {
            ConditionParam::Drunkenness => {
                self.drunkenness = value as u8;
                true
            }
            p => self.base.set_param(p, value),
        }
    }

    /// Mirrors `ConditionDrunk::getIcons` (condition.cpp:1977).
    pub fn get_icons(&self) -> u32 {
        1 << 3 // ICON_DRUNK
    }
}

// ---------------------------------------------------------------------------
// ConditionGeneric — covers infight, exhaust, muted, cooldowns, etc.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionGeneric {
    pub base: ConditionBase,
}

impl ConditionGeneric {
    pub fn new(
        id: ConditionId,
        condition_type: i32,
        ticks: i32,
        buff: bool,
        sub_id: u32,
        aggressive: bool,
    ) -> Self {
        Self {
            base: ConditionBase::new(id, condition_type, ticks, buff, sub_id, aggressive),
        }
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    pub fn add_condition(&mut self, other: &ConditionGeneric) {
        if self.base.should_update(&other.base) {
            self.base.set_ticks(other.base.ticks);
        }
    }

    /// Mirrors `ConditionGeneric::getIcons` (condition.cpp:353).
    pub fn get_icons(&self) -> u32 {
        use ConditionTypeFlags as T;
        let icons = self.base.get_icons();
        let ct = self.base.condition_type;
        if ct == T::MANASHIELD {
            icons | (1 << 4) // ICON_MANASHIELD
        } else if ct == T::INFIGHT {
            icons | (1 << 7) // ICON_SWORDS
        } else if ct == T::ROOT {
            icons | (1 << 19) // ICON_ROOT
        } else {
            icons
        }
    }
}

// ---------------------------------------------------------------------------
// ConditionAttributes — buffs/debuffs that adjust Player skills/stats/special-skills
// (mirrors C++ `class ConditionAttributes final : public ConditionGeneric`)
// ---------------------------------------------------------------------------
//
// The C++ subclass stores parallel arrays (skills, skillsPercent,
// specialSkills, stats, statsPercent) plus a disableDefense flag and three
// cursor ints (currentSkill/currentSpecialSkill/currentStat) that the
// unserializeProp dispatcher uses to walk the parallel SKILLS / SPECIALSKILLS
// / STATS attribute streams.  The live Player-state effects (updateSkills /
// updateStats / etc.) reach across to Player, so they are deferred to the
// cross-crate game-glue layer per the
// `cross-crate-behavior-dispatch-deferred-to-game-glue` intentional
// difference.  This struct owns the pure decision surface: the field carriers
// + setParam dispatch + serialize/deserialize round-trip.

/// Count of `Skill::FIRST..=Skill::Fishing` — matches C++ SKILL_LAST + 1.
pub const ATTR_SKILL_COUNT: usize = 7;
/// Count of `Stat::FIRST..=Stat::MagicPoints` — matches C++ STAT_LAST + 1.
pub const ATTR_STAT_COUNT: usize = 4;
/// Count of `SpecialSkill::FIRST..=SpecialSkill::ManaLeechAmount` — matches C++
/// SPECIALSKILL_LAST + 1.
pub const ATTR_SPECIALSKILL_COUNT: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionAttributes {
    pub base: ConditionBase,
    pub skills: [i32; ATTR_SKILL_COUNT],
    pub skills_percent: [i32; ATTR_SKILL_COUNT],
    pub special_skills: [i32; ATTR_SPECIALSKILL_COUNT],
    pub stats: [i32; ATTR_STAT_COUNT],
    pub stats_percent: [i32; ATTR_STAT_COUNT],
    pub disable_defense: bool,
    /// Cursor advanced by `deserialize` as it consumes each successive
    /// `CONDITIONATTR_SKILLS` block (mirrors C++ `currentSkill`).
    pub current_skill: usize,
    pub current_special_skill: usize,
    pub current_stat: usize,
}

impl ConditionAttributes {
    pub fn new(id: ConditionId, ticks: i32, buff: bool, sub_id: u32, aggressive: bool) -> Self {
        Self {
            base: ConditionBase::new(
                id,
                ConditionTypeFlags::ATTRIBUTES,
                ticks,
                buff,
                sub_id,
                aggressive,
            ),
            skills: [0; ATTR_SKILL_COUNT],
            skills_percent: [0; ATTR_SKILL_COUNT],
            special_skills: [0; ATTR_SPECIALSKILL_COUNT],
            stats: [0; ATTR_STAT_COUNT],
            stats_percent: [0; ATTR_STAT_COUNT],
            disable_defense: false,
            current_skill: 0,
            current_special_skill: 0,
            current_stat: 0,
        }
    }

    pub fn tick(&mut self, interval: i32) -> bool {
        self.base.tick(interval)
    }

    /// Mirrors C++ `ConditionAttributes::setParam` (condition.cpp:582).
    /// Returns `true` for the params owned by this subclass; falls through
    /// to `ConditionBase::set_param` (which already records SubId / Aggressive
    /// / Ticks / etc.) for everything else.
    pub fn set_param(&mut self, param: ConditionParam, value: i64) -> bool {
        use forgottenserver_common::enums::{Skill, SpecialSkill, Stat};

        let v = value as i32;
        let s = |k: Skill| k as usize;
        let st = |k: Stat| k as usize;
        let ss = |k: SpecialSkill| k as usize;

        match param {
            // Aggregate melee → CLUB + AXE + SWORD
            ConditionParam::SkillMelee => {
                self.skills[s(Skill::Club)] = v;
                self.skills[s(Skill::Axe)] = v;
                self.skills[s(Skill::Sword)] = v;
                true
            }
            ConditionParam::SkillMeleePercent => {
                self.skills_percent[s(Skill::Club)] = v;
                self.skills_percent[s(Skill::Axe)] = v;
                self.skills_percent[s(Skill::Sword)] = v;
                true
            }
            // Individual skills
            ConditionParam::SkillFist => {
                self.skills[s(Skill::Fist)] = v;
                true
            }
            ConditionParam::SkillFistPercent => {
                self.skills_percent[s(Skill::Fist)] = v;
                true
            }
            ConditionParam::SkillClub => {
                self.skills[s(Skill::Club)] = v;
                true
            }
            ConditionParam::SkillClubPercent => {
                self.skills_percent[s(Skill::Club)] = v;
                true
            }
            ConditionParam::SkillSword => {
                self.skills[s(Skill::Sword)] = v;
                true
            }
            ConditionParam::SkillSwordPercent => {
                self.skills_percent[s(Skill::Sword)] = v;
                true
            }
            ConditionParam::SkillAxe => {
                self.skills[s(Skill::Axe)] = v;
                true
            }
            ConditionParam::SkillAxePercent => {
                self.skills_percent[s(Skill::Axe)] = v;
                true
            }
            ConditionParam::SkillDistance => {
                self.skills[s(Skill::Distance)] = v;
                true
            }
            ConditionParam::SkillDistancePercent => {
                self.skills_percent[s(Skill::Distance)] = v;
                true
            }
            ConditionParam::SkillShield => {
                self.skills[s(Skill::Shield)] = v;
                true
            }
            ConditionParam::SkillShieldPercent => {
                self.skills_percent[s(Skill::Shield)] = v;
                true
            }
            ConditionParam::SkillFishing => {
                self.skills[s(Skill::Fishing)] = v;
                true
            }
            ConditionParam::SkillFishingPercent => {
                self.skills_percent[s(Skill::Fishing)] = v;
                true
            }

            // Stats
            ConditionParam::StatMaxHitPoints => {
                self.stats[st(Stat::MaxHitPoints)] = v;
                true
            }
            ConditionParam::StatMaxManaPoints => {
                self.stats[st(Stat::MaxManaPoints)] = v;
                true
            }
            ConditionParam::StatMagicPoints => {
                self.stats[st(Stat::MagicPoints)] = v;
                true
            }
            ConditionParam::StatMaxHitPointsPercent => {
                self.stats_percent[st(Stat::MaxHitPoints)] = v;
                true
            }
            ConditionParam::StatMaxManaPointsPercent => {
                self.stats_percent[st(Stat::MaxManaPoints)] = v;
                true
            }
            ConditionParam::StatMagicPointsPercent => {
                self.stats_percent[st(Stat::MagicPoints)] = v;
                true
            }

            // Special skills
            ConditionParam::SpecialSkillCriticalHitChance => {
                self.special_skills[ss(SpecialSkill::CriticalHitChance)] = v;
                true
            }
            ConditionParam::SpecialSkillCriticalHitAmount => {
                self.special_skills[ss(SpecialSkill::CriticalHitAmount)] = v;
                true
            }
            ConditionParam::SpecialSkillLifeLeechChance => {
                self.special_skills[ss(SpecialSkill::LifeLeechChance)] = v;
                true
            }
            ConditionParam::SpecialSkillLifeLeechAmount => {
                self.special_skills[ss(SpecialSkill::LifeLeechAmount)] = v;
                true
            }
            ConditionParam::SpecialSkillManaLeechChance => {
                self.special_skills[ss(SpecialSkill::ManaLeechChance)] = v;
                true
            }
            ConditionParam::SpecialSkillManaLeechAmount => {
                self.special_skills[ss(SpecialSkill::ManaLeechAmount)] = v;
                true
            }

            ConditionParam::DisableDefense => {
                self.disable_defense = v != 0;
                true
            }

            // Defer everything else to the shared base implementation (covers
            // SubId / Aggressive / Ticks / ForceUpdate / BuffSpell / etc.).
            p => self.base.set_param(p, value),
        }
    }

    /// Mirrors `ConditionAttributes::getParam` (condition.cpp:746).
    /// Reads back the stored skill/stat values without touching the creature.
    pub fn get_param(&self, param: ConditionParam) -> i64 {
        use forgottenserver_common::enums::{Skill, SpecialSkill, Stat};
        let s = |k: Skill| k as usize;
        let st = |k: Stat| k as usize;
        let ss = |k: SpecialSkill| k as usize;
        match param {
            ConditionParam::SkillFist => self.skills[s(Skill::Fist)] as i64,
            ConditionParam::SkillFistPercent => self.skills_percent[s(Skill::Fist)] as i64,
            ConditionParam::SkillClub | ConditionParam::SkillMelee => {
                self.skills[s(Skill::Club)] as i64
            }
            ConditionParam::SkillClubPercent | ConditionParam::SkillMeleePercent => {
                self.skills_percent[s(Skill::Club)] as i64
            }
            ConditionParam::SkillSword => self.skills[s(Skill::Sword)] as i64,
            ConditionParam::SkillSwordPercent => self.skills_percent[s(Skill::Sword)] as i64,
            ConditionParam::SkillAxe => self.skills[s(Skill::Axe)] as i64,
            ConditionParam::SkillAxePercent => self.skills_percent[s(Skill::Axe)] as i64,
            ConditionParam::SkillDistance => self.skills[s(Skill::Distance)] as i64,
            ConditionParam::SkillDistancePercent => self.skills_percent[s(Skill::Distance)] as i64,
            ConditionParam::SkillShield => self.skills[s(Skill::Shield)] as i64,
            ConditionParam::SkillShieldPercent => self.skills_percent[s(Skill::Shield)] as i64,
            ConditionParam::SkillFishing => self.skills[s(Skill::Fishing)] as i64,
            ConditionParam::SkillFishingPercent => self.skills_percent[s(Skill::Fishing)] as i64,
            ConditionParam::StatMaxHitPoints => self.stats[st(Stat::MaxHitPoints)] as i64,
            ConditionParam::StatMaxManaPoints => self.stats[st(Stat::MaxManaPoints)] as i64,
            ConditionParam::StatMagicPoints => self.stats[st(Stat::MagicPoints)] as i64,
            ConditionParam::StatMaxHitPointsPercent => {
                self.stats_percent[st(Stat::MaxHitPoints)] as i64
            }
            ConditionParam::StatMaxManaPointsPercent => {
                self.stats_percent[st(Stat::MaxManaPoints)] as i64
            }
            ConditionParam::StatMagicPointsPercent => {
                self.stats_percent[st(Stat::MagicPoints)] as i64
            }
            ConditionParam::DisableDefense => i64::from(self.disable_defense),
            ConditionParam::SpecialSkillCriticalHitChance => {
                self.special_skills[ss(SpecialSkill::CriticalHitChance)] as i64
            }
            ConditionParam::SpecialSkillCriticalHitAmount => {
                self.special_skills[ss(SpecialSkill::CriticalHitAmount)] as i64
            }
            ConditionParam::SpecialSkillLifeLeechChance => {
                self.special_skills[ss(SpecialSkill::LifeLeechChance)] as i64
            }
            ConditionParam::SpecialSkillLifeLeechAmount => {
                self.special_skills[ss(SpecialSkill::LifeLeechAmount)] as i64
            }
            ConditionParam::SpecialSkillManaLeechChance => {
                self.special_skills[ss(SpecialSkill::ManaLeechChance)] as i64
            }
            ConditionParam::SpecialSkillManaLeechAmount => {
                self.special_skills[ss(SpecialSkill::ManaLeechAmount)] as i64
            }
            p => self.base.get_param(p),
        }
    }

    /// Merge logic for an incoming ConditionAttributes — mirrors C++
    /// `ConditionAttributes::addCondition` (resets the cursor + copies the
    /// parallel arrays from the other instance).
    pub fn add_condition(&mut self, other: &ConditionAttributes) {
        if !self.base.should_update(&other.base) {
            return;
        }
        self.base.set_ticks(other.base.ticks);
        self.skills = other.skills;
        self.skills_percent = other.skills_percent;
        self.special_skills = other.special_skills;
        self.stats = other.stats;
        self.stats_percent = other.stats_percent;
        self.disable_defense = other.disable_defense;
    }

    pub fn serialize(&self, out: &mut Vec<u8>) {
        self.base.serialize(out);
        for v in &self.skills {
            push_u8(out, ConditionAttr::Skills as u8);
            push_i32(out, *v);
        }
        for v in &self.stats {
            push_u8(out, ConditionAttr::Stats as u8);
            push_i32(out, *v);
        }
        push_u8(out, ConditionAttr::DisableDefense as u8);
        push_u8(out, self.disable_defense as u8);
        for v in &self.special_skills {
            push_u8(out, ConditionAttr::SpecialSkills as u8);
            push_i32(out, *v);
        }
        push_u8(out, ConditionAttr::End as u8);
    }

    pub fn deserialize(data: &[u8]) -> Option<(Self, usize)> {
        let (base, mut pos) = ConditionBase::deserialize(data)?;
        let mut s = Self {
            base,
            skills: [0; ATTR_SKILL_COUNT],
            skills_percent: [0; ATTR_SKILL_COUNT],
            special_skills: [0; ATTR_SPECIALSKILL_COUNT],
            stats: [0; ATTR_STAT_COUNT],
            stats_percent: [0; ATTR_STAT_COUNT],
            disable_defense: false,
            current_skill: 0,
            current_special_skill: 0,
            current_stat: 0,
        };

        loop {
            if pos >= data.len() {
                break;
            }
            let attr = data[pos];
            pos += 1;
            if attr == ConditionAttr::End as u8 {
                break;
            }
            match attr {
                a if a == ConditionAttr::Skills as u8 => {
                    if pos + 4 > data.len() || s.current_skill >= ATTR_SKILL_COUNT {
                        return None;
                    }
                    let v = i32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    s.skills[s.current_skill] = v;
                    s.current_skill += 1;
                    pos += 4;
                }
                a if a == ConditionAttr::SpecialSkills as u8 => {
                    if pos + 4 > data.len() || s.current_special_skill >= ATTR_SPECIALSKILL_COUNT {
                        return None;
                    }
                    let v = i32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    s.special_skills[s.current_special_skill] = v;
                    s.current_special_skill += 1;
                    pos += 4;
                }
                a if a == ConditionAttr::Stats as u8 => {
                    if pos + 4 > data.len() || s.current_stat >= ATTR_STAT_COUNT {
                        return None;
                    }
                    let v = i32::from_le_bytes(data[pos..pos + 4].try_into().ok()?);
                    s.stats[s.current_stat] = v;
                    s.current_stat += 1;
                    pos += 4;
                }
                a if a == ConditionAttr::DisableDefense as u8 => {
                    if pos >= data.len() {
                        return None;
                    }
                    s.disable_defense = data[pos] != 0;
                    pos += 1;
                }
                _ => break,
            }
        }
        Some((s, pos))
    }
}

// ---------------------------------------------------------------------------
// Condition factory — mirror of C++ `Condition::createCondition` (condition.cpp:168).
// Dispatches on the `ConditionTypeFlags::*` bit to the matching subclass
// constructor. `param` carries subclass-specific data: ConditionSpeed gets
// the speed delta; ConditionLight packs (level | (color << 8)). Unsupported /
// unknown types return `None` (C++ returns nullptr through the switch's
// implicit default).
// ---------------------------------------------------------------------------

pub fn create_condition(
    id: ConditionId,
    condition_type: i32,
    ticks: i32,
    param: i32,
    buff: bool,
    sub_id: u32,
    aggressive: bool,
) -> Option<Condition> {
    use ConditionTypeFlags as T;
    // Damage family — every variant flows to ConditionDamage.
    if condition_type == T::POISON
        || condition_type == T::FIRE
        || condition_type == T::ENERGY
        || condition_type == T::DROWN
        || condition_type == T::FREEZING
        || condition_type == T::DAZZLED
        || condition_type == T::CURSED
        || condition_type == T::BLEEDING
    {
        return Some(Condition::Damage(ConditionDamage::new(
            id,
            condition_type,
            buff,
            sub_id,
            aggressive,
        )));
    }
    if condition_type == T::HASTE || condition_type == T::PARALYZE {
        // C++ ctor passes the `param` (speed delta) directly to ConditionSpeed.
        let c = ConditionSpeed::new(id, condition_type, ticks, buff, sub_id, param, aggressive);
        return Some(Condition::Speed(c));
    }
    if condition_type == T::INVISIBLE {
        return Some(Condition::Invisible(ConditionInvisible::new(
            id, ticks, buff, sub_id, aggressive,
        )));
    }
    if condition_type == T::OUTFIT {
        return Some(Condition::Outfit(ConditionOutfit::new(
            id, ticks, buff, sub_id, aggressive,
        )));
    }
    if condition_type == T::LIGHT {
        let level = (param & 0xFF) as u8;
        let color = ((param >> 8) & 0xFF) as u8;
        return Some(Condition::Light(ConditionLight::new(
            id, ticks, buff, sub_id, level, color, aggressive,
        )));
    }
    if condition_type == T::REGENERATION {
        return Some(Condition::Regeneration(ConditionRegeneration::new(
            id, ticks, buff, sub_id, aggressive,
        )));
    }
    if condition_type == T::SOUL {
        return Some(Condition::Soul(ConditionSoul::new(
            id, ticks, buff, sub_id, aggressive,
        )));
    }
    if condition_type == T::ATTRIBUTES {
        return Some(Condition::Attributes(ConditionAttributes::new(
            id, ticks, buff, sub_id, aggressive,
        )));
    }
    if condition_type == T::DRUNK {
        return Some(Condition::Drunk(ConditionDrunk::new(
            id, ticks, buff, sub_id, 0, aggressive,
        )));
    }
    // SPELLCOOLDOWN / SPELLGROUPCOOLDOWN / INFIGHT / EXHAUST_WEAPON / MUTED /
    // and any other "marker"-only condition flow through ConditionGeneric.
    if condition_type == T::SPELLCOOLDOWN
        || condition_type == T::SPELLGROUPCOOLDOWN
        || condition_type == T::INFIGHT
        || condition_type == T::EXHAUST_WEAPON
        || condition_type == T::MUTED
        || condition_type == T::CHANNELMUTEDTICKS
        || condition_type == T::YELLTICKS
        || condition_type == T::PACIFIED
        || condition_type == T::MANASHIELD
        || condition_type == T::ROOT
    {
        return Some(Condition::Generic(ConditionGeneric::new(
            id,
            condition_type,
            ticks,
            buff,
            sub_id,
            aggressive,
        )));
    }
    None
}

// ---------------------------------------------------------------------------
// Top-level Condition enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Condition {
    Generic(ConditionGeneric),
    Regeneration(ConditionRegeneration),
    Soul(ConditionSoul),
    Damage(ConditionDamage),
    Speed(ConditionSpeed),
    Light(ConditionLight),
    ManaShield(ConditionManaShield),
    Outfit(ConditionOutfit),
    Invisible(ConditionInvisible),
    Drunk(ConditionDrunk),
    Attributes(ConditionAttributes),
}

impl Condition {
    pub fn base(&self) -> &ConditionBase {
        match self {
            Condition::Generic(c) => &c.base,
            Condition::Regeneration(c) => &c.base,
            Condition::Soul(c) => &c.base,
            Condition::Damage(c) => &c.base,
            Condition::Speed(c) => &c.base,
            Condition::Light(c) => &c.base,
            Condition::ManaShield(c) => &c.base,
            Condition::Outfit(c) => &c.base,
            Condition::Invisible(c) => &c.base,
            Condition::Drunk(c) => &c.base,
            Condition::Attributes(c) => &c.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut ConditionBase {
        match self {
            Condition::Generic(c) => &mut c.base,
            Condition::Regeneration(c) => &mut c.base,
            Condition::Soul(c) => &mut c.base,
            Condition::Damage(c) => &mut c.base,
            Condition::Speed(c) => &mut c.base,
            Condition::Light(c) => &mut c.base,
            Condition::ManaShield(c) => &mut c.base,
            Condition::Outfit(c) => &mut c.base,
            Condition::Invisible(c) => &mut c.base,
            Condition::Drunk(c) => &mut c.base,
            Condition::Attributes(c) => &mut c.base,
        }
    }

    pub fn condition_type(&self) -> i32 {
        self.base().condition_type
    }

    pub fn id(&self) -> ConditionId {
        self.base().id
    }

    pub fn ticks(&self) -> i32 {
        self.base().ticks
    }

    pub fn is_buff(&self) -> bool {
        self.base().is_buff
    }

    pub fn is_aggressive(&self) -> bool {
        self.base().aggressive
    }
}

// ---------------------------------------------------------------------------
// Helper serialisation functions
// ---------------------------------------------------------------------------

fn push_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

fn push_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- ConditionBase ------------------------------------------------------

    #[test]
    fn test_condition_base_permanent_never_expires() {
        let mut base = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            -1,
            false,
            0,
            true,
        );
        assert!(base.tick(10_000));
        assert!(base.tick(10_000));
        assert_eq!(base.ticks, -1, "permanent ticks must stay -1");
    }

    #[test]
    fn test_condition_base_timed_expires() {
        let mut base = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            3000,
            false,
            0,
            false,
        );
        assert!(base.tick(1000));
        assert!(base.tick(1000));
        // After 3000 ms total the condition expires (ticks reaches 0)
        assert!(!base.tick(1000));
        assert_eq!(base.ticks, 0);
    }

    #[test]
    fn test_condition_base_tick_clamps_to_zero() {
        let mut base = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::ENERGY,
            500,
            false,
            0,
            false,
        );
        // interval larger than remaining ticks
        assert!(!base.tick(2000));
        assert_eq!(base.ticks, 0, "ticks must not go negative");
    }

    #[test]
    fn test_condition_base_set_param_ticks() {
        let mut base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            0,
            false,
        );
        assert!(base.set_param(ConditionParam::Ticks, 5000));
        assert_eq!(base.ticks, 5000);
    }

    #[test]
    fn test_condition_base_set_param_buff() {
        let mut base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            0,
            false,
        );
        assert!(base.set_param(ConditionParam::BuffSpell, 1));
        assert!(base.is_buff);
        assert!(base.set_param(ConditionParam::BuffSpell, 0));
        assert!(!base.is_buff);
    }

    #[test]
    fn test_condition_base_set_param_aggressive() {
        let mut base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            0,
            false,
        );
        assert!(base.set_param(ConditionParam::Aggressive, 1));
        assert!(base.aggressive);
    }

    #[test]
    fn test_condition_base_get_param_ticks() {
        let base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            2000,
            false,
            0,
            false,
        );
        assert_eq!(base.get_param(ConditionParam::Ticks), 2000);
    }

    #[test]
    fn test_condition_base_get_param_sub_id() {
        let base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            42,
            false,
        );
        assert_eq!(base.get_param(ConditionParam::SubId), 42);
    }

    #[test]
    fn test_condition_base_should_update_longer_wins() {
        let existing = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            1000,
            false,
            0,
            false,
        );
        let incoming = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            5000,
            false,
            0,
            false,
        );
        assert!(existing.should_update(&incoming));
    }

    #[test]
    fn test_condition_base_should_not_update_shorter() {
        let existing = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            5000,
            false,
            0,
            false,
        );
        let incoming = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            1000,
            false,
            0,
            false,
        );
        assert!(!existing.should_update(&incoming));
    }

    #[test]
    fn test_condition_base_should_not_update_different_type() {
        let existing = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            1000,
            false,
            0,
            false,
        );
        let incoming = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            5000,
            false,
            0,
            false,
        );
        assert!(!existing.should_update(&incoming));
    }

    #[test]
    fn test_condition_base_is_persistent_combat_id() {
        let base = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            1000,
            false,
            0,
            false,
        );
        assert!(base.is_persistent());
    }

    #[test]
    fn test_condition_base_is_not_persistent_when_permanent() {
        // ticks == -1 → NOT persistent (cannot be saved/restored)
        let base = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            -1,
            false,
            0,
            false,
        );
        assert!(!base.is_persistent());
    }

    // --- Serialize / Deserialize round-trip ---------------------------------

    #[test]
    fn test_condition_base_serialize_roundtrip() {
        let original = ConditionBase {
            id: ConditionId::Combat,
            condition_type: ConditionTypeFlags::FIRE,
            ticks: 3000,
            end_time: 0,
            sub_id: 7,
            is_buff: true,
            aggressive: false,
        };
        let mut buf = Vec::new();
        original.serialize(&mut buf);
        push_u8(&mut buf, ConditionAttr::End as u8);

        let (decoded, _) = ConditionBase::deserialize(&buf).expect("deserialize should succeed");
        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.condition_type, original.condition_type);
        assert_eq!(decoded.ticks, original.ticks);
        assert_eq!(decoded.sub_id, original.sub_id);
        assert_eq!(decoded.is_buff, original.is_buff);
        assert_eq!(decoded.aggressive, original.aggressive);
    }

    #[test]
    fn test_condition_base_deserialize_empty_returns_none() {
        assert!(ConditionBase::deserialize(&[]).is_none());
    }

    // --- ConditionRegeneration ---------------------------------------------

    #[test]
    fn test_regen_hp_gain_fires_at_health_tick_boundary() {
        let mut regen = ConditionRegeneration::new(ConditionId::Combat, 10_000, true, 0, false);
        regen.health_ticks = 1000;
        regen.health_gain = 5;

        // First 999 ms → no HP yet
        let (alive, hp, _) = regen.tick(999);
        assert!(alive);
        assert_eq!(hp, 0);

        // 1 more ms crosses threshold
        let (alive, hp, _) = regen.tick(1);
        assert!(alive);
        assert_eq!(hp, 5);
    }

    #[test]
    fn test_regen_mana_gain_fires_independently() {
        let mut regen = ConditionRegeneration::new(ConditionId::Combat, 10_000, true, 0, false);
        regen.health_ticks = 2000;
        regen.mana_ticks = 500;
        regen.health_gain = 3;
        regen.mana_gain = 10;

        let (_, hp, mana) = regen.tick(500);
        assert_eq!(hp, 0, "health tick not reached yet");
        assert_eq!(mana, 10, "mana should fire after 500 ms");
    }

    #[test]
    fn test_regen_expires_when_ticks_exhausted() {
        let mut regen = ConditionRegeneration::new(ConditionId::Combat, 2000, false, 0, false);
        regen.health_ticks = 5000; // won't fire during lifetime
        regen.health_gain = 1;

        let (alive, _, _) = regen.tick(1000);
        assert!(alive);
        let (alive, _, _) = regen.tick(1001); // crosses 2000 total
        assert!(!alive);
    }

    #[test]
    fn test_regen_add_condition_extends_ticks() {
        let mut existing = ConditionRegeneration::new(ConditionId::Combat, 1000, false, 0, false);
        existing.health_gain = 5;

        let mut incoming = ConditionRegeneration::new(ConditionId::Combat, 5000, false, 0, false);
        incoming.health_gain = 10;

        existing.add_condition(&incoming);

        assert_eq!(existing.base.ticks, 5000);
        assert_eq!(existing.health_gain, 10);
    }

    #[test]
    fn test_regen_add_condition_shorter_does_not_update() {
        let mut existing = ConditionRegeneration::new(ConditionId::Combat, 5000, false, 0, false);
        existing.health_gain = 5;

        let mut incoming = ConditionRegeneration::new(ConditionId::Combat, 1000, false, 0, false);
        incoming.health_gain = 10;

        existing.add_condition(&incoming);

        assert_eq!(
            existing.base.ticks, 5000,
            "should not shorten the condition"
        );
        assert_eq!(existing.health_gain, 5, "should keep original gain");
    }

    #[test]
    fn test_regen_set_get_params() {
        let mut regen = ConditionRegeneration::new(ConditionId::Combat, 5000, false, 0, false);
        regen.set_param(ConditionParam::HealthGain, 15);
        regen.set_param(ConditionParam::HealthTicks, 2000);
        regen.set_param(ConditionParam::ManaGain, 7);
        regen.set_param(ConditionParam::ManaTicks, 1500);

        assert_eq!(regen.get_param(ConditionParam::HealthGain), 15);
        assert_eq!(regen.get_param(ConditionParam::HealthTicks), 2000);
        assert_eq!(regen.get_param(ConditionParam::ManaGain), 7);
        assert_eq!(regen.get_param(ConditionParam::ManaTicks), 1500);
    }

    // --- ConditionSoul ------------------------------------------------------

    #[test]
    fn test_soul_gain_fires_at_soul_tick_boundary() {
        let mut soul = ConditionSoul::new(ConditionId::Combat, 10_000, false, 0, false);
        soul.soul_ticks = 2000;
        soul.soul_gain = 3;

        let (alive, gain) = soul.tick(1999);
        assert!(alive);
        assert_eq!(gain, 0);

        let (alive, gain) = soul.tick(1);
        assert!(alive);
        assert_eq!(gain, 3);
    }

    #[test]
    fn test_soul_no_gain_when_soul_ticks_is_zero() {
        let mut soul = ConditionSoul::new(ConditionId::Combat, 5000, false, 0, false);
        soul.soul_ticks = 0;
        soul.soul_gain = 5;

        let (_, gain) = soul.tick(1000);
        assert_eq!(gain, 0, "soul_ticks == 0 means no gain fires");
    }

    #[test]
    fn test_soul_add_condition_updates_params() {
        let mut existing = ConditionSoul::new(ConditionId::Combat, 1000, false, 0, false);
        existing.soul_gain = 1;
        existing.soul_ticks = 1000;

        let mut incoming = ConditionSoul::new(ConditionId::Combat, 8000, false, 0, false);
        incoming.soul_gain = 5;
        incoming.soul_ticks = 2000;

        existing.add_condition(&incoming);
        assert_eq!(existing.soul_gain, 5);
        assert_eq!(existing.soul_ticks, 2000);
    }

    #[test]
    fn test_soul_set_get_params() {
        let mut soul = ConditionSoul::new(ConditionId::Combat, 5000, false, 0, false);
        soul.set_param(ConditionParam::SoulGain, 4);
        soul.set_param(ConditionParam::SoulTicks, 3000);

        assert_eq!(soul.get_param(ConditionParam::SoulGain), 4);
        assert_eq!(soul.get_param(ConditionParam::SoulTicks), 3000);
    }

    // --- ConditionDamage ----------------------------------------------------

    #[test]
    fn test_damage_set_get_params() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            0,
            true,
        );
        dmg.set_param(ConditionParam::MaxValue, 100);
        dmg.set_param(ConditionParam::MinValue, 50);
        dmg.set_param(ConditionParam::TickInterval, 1000);
        dmg.set_param(ConditionParam::Delayed, 1);

        assert_eq!(dmg.get_param(ConditionParam::MaxValue), 100);
        assert_eq!(dmg.get_param(ConditionParam::MinValue), 50);
        assert_eq!(dmg.get_param(ConditionParam::TickInterval), 1000);
        assert_eq!(dmg.get_param(ConditionParam::Delayed), 1);
    }

    #[test]
    fn test_damage_add_damage_periodic() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        let ok = dmg.add_damage(-1, 2000, -10);
        assert!(ok);
        assert_eq!(dmg.period_damage, -10);
        assert_eq!(dmg.base.ticks, -1);
    }

    #[test]
    fn test_damage_add_damage_scheduled_rounds() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::ENERGY,
            false,
            0,
            true,
        );
        let ok = dmg.add_damage(3, 1000, -20);
        assert!(ok);
        assert_eq!(dmg.damage_list.len(), 3);
        assert_eq!(dmg.base.ticks, 3000, "ticks += 1000 per round");
    }

    #[test]
    fn test_damage_get_total_damage_from_list() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            0,
            true,
        );
        dmg.add_damage(3, 1000, -10);
        assert_eq!(dmg.get_total_damage(), 30);
    }

    #[test]
    fn test_damage_should_update_force_update() {
        let existing = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        let mut incoming = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        incoming.force_update = true;
        assert!(existing.should_update(&incoming));
    }

    #[test]
    fn test_damage_should_not_update_when_existing_has_more_damage() {
        let mut existing = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        existing.add_damage(5, 1000, -20); // total 100

        let mut incoming = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        incoming.add_damage(1, 1000, -10); // total 10

        assert!(!existing.should_update(&incoming));
    }

    // --- ConditionSpeed -----------------------------------------------------

    #[test]
    fn test_speed_set_get_speed_delta() {
        let mut speed = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            5000,
            false,
            0,
            100,
            false,
        );
        assert_eq!(speed.get_param(ConditionParam::Speed), 100);
        speed.set_param(ConditionParam::Speed, -50);
        assert_eq!(speed.get_param(ConditionParam::Speed), -50);
        assert_eq!(speed.speed_delta, -50);
    }

    #[test]
    fn test_speed_expires() {
        let mut speed = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::PARALYZE,
            2000,
            false,
            0,
            -30,
            true,
        );
        assert!(speed.tick(1000));
        assert!(!speed.tick(1001));
    }

    #[test]
    fn test_speed_formula_vars() {
        let mut speed = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            5000,
            false,
            0,
            0,
            false,
        );
        speed.set_formula_vars(1.0, 0.0, 1.5, 0.5);
        assert!((speed.formula_mina - 1.0).abs() < f32::EPSILON);
        assert!((speed.formula_maxb - 0.5).abs() < f32::EPSILON);
    }

    // --- ConditionLight -----------------------------------------------------

    #[test]
    fn test_light_set_get_params() {
        let mut light = ConditionLight::new(ConditionId::Combat, 5000, false, 0, 255, 200, false);
        assert_eq!(light.get_param(ConditionParam::LightLevel), 255);
        assert_eq!(light.get_param(ConditionParam::LightColor), 200);

        light.set_param(ConditionParam::LightLevel, 128);
        assert_eq!(light.light_level, 128);
    }

    #[test]
    fn test_light_expires() {
        let mut light = ConditionLight::new(ConditionId::Combat, 1000, false, 0, 10, 5, false);
        assert!(light.tick(500));
        assert!(!light.tick(501));
    }

    #[test]
    fn test_light_new_stores_level_and_color() {
        let light = ConditionLight::new(ConditionId::Default, 3000, true, 1, 200, 150, false);
        assert_eq!(light.light_level, 200);
        assert_eq!(light.light_color, 150);
        assert!(light.base.is_buff);
    }

    // --- ConditionManaShield ------------------------------------------------

    #[test]
    fn test_manashield_absorbs_damage() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, -1, false, 0);
        shield.mana_shield = 100;
        shield.max_mana_shield = 100;

        let remaining = shield.absorb_damage(60);
        assert_eq!(remaining, 0, "shield absorbed all damage");
        assert_eq!(shield.mana_shield, 40);
    }

    #[test]
    fn test_manashield_passes_excess_damage() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, -1, false, 0);
        shield.mana_shield = 30;
        shield.max_mana_shield = 100;

        let remaining = shield.absorb_damage(100);
        assert_eq!(remaining, 70, "shield drained, 70 damage passes through");
        assert_eq!(shield.mana_shield, 0);
    }

    #[test]
    fn test_manashield_no_shield_passes_all_damage() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, -1, false, 0);
        shield.mana_shield = 0;

        let remaining = shield.absorb_damage(50);
        assert_eq!(remaining, 50);
    }

    // --- ConditionGeneric ---------------------------------------------------

    #[test]
    fn test_generic_tick_expires() {
        let mut g = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::INFIGHT,
            3000,
            false,
            0,
            false,
        );
        assert!(g.tick(1000));
        assert!(g.tick(1000));
        assert!(!g.tick(1001));
    }

    #[test]
    fn test_generic_add_condition_extends_ticks() {
        let mut existing = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::INFIGHT,
            1000,
            false,
            0,
            false,
        );
        let incoming = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::INFIGHT,
            5000,
            false,
            0,
            false,
        );
        existing.add_condition(&incoming);
        assert_eq!(existing.base.ticks, 5000);
    }

    #[test]
    fn test_generic_add_condition_does_not_shorten() {
        let mut existing = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::INFIGHT,
            9000,
            false,
            0,
            false,
        );
        let incoming = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::INFIGHT,
            1000,
            false,
            0,
            false,
        );
        existing.add_condition(&incoming);
        assert_eq!(
            existing.base.ticks, 9000,
            "shorter incoming must not reduce"
        );
    }

    // --- Condition enum accessors -------------------------------------------

    #[test]
    fn test_condition_enum_accessors() {
        let c = Condition::Regeneration(ConditionRegeneration::new(
            ConditionId::Combat,
            5000,
            true,
            0,
            false,
        ));
        assert_eq!(c.id(), ConditionId::Combat);
        assert_eq!(c.ticks(), 5000);
        assert!(c.is_buff());
        assert!(!c.is_aggressive());
        assert_eq!(c.condition_type(), ConditionTypeFlags::REGENERATION);
    }

    #[test]
    fn test_condition_damage_enum_is_aggressive() {
        let c = Condition::Damage(ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        ));
        assert!(c.is_aggressive());
    }

    // --- ConditionSpeed stacking & serialize/deserialize --------------------

    #[test]
    fn test_speed_set_param_positive_sets_haste_type() {
        let mut speed = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            5000,
            false,
            0,
            0,
            false,
        );
        speed.set_param(ConditionParam::Speed, 100);
        assert_eq!(speed.base.condition_type, ConditionTypeFlags::HASTE);
        assert_eq!(speed.speed_delta, 100);
    }

    #[test]
    fn test_speed_set_param_negative_sets_paralyze_type() {
        let mut speed = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::PARALYZE,
            5000,
            false,
            0,
            0,
            true,
        );
        speed.set_param(ConditionParam::Speed, -50);
        assert_eq!(speed.base.condition_type, ConditionTypeFlags::PARALYZE);
        assert_eq!(speed.speed_delta, -50);
    }

    #[test]
    fn test_speed_add_condition_updates_ticks_and_delta() {
        let mut existing = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            2000,
            false,
            0,
            50,
            false,
        );
        let incoming = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            8000,
            false,
            0,
            120,
            false,
        );
        let net_change = existing.add_condition(&incoming);
        assert_eq!(existing.base.ticks, 8000);
        assert_eq!(existing.speed_delta, 120);
        assert_eq!(net_change, 70, "net change = 120 - 50");
    }

    #[test]
    fn test_speed_add_condition_different_type_noop() {
        let mut existing = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            2000,
            false,
            0,
            50,
            false,
        );
        let incoming = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::PARALYZE,
            8000,
            false,
            0,
            -100,
            false,
        );
        let net_change = existing.add_condition(&incoming);
        assert_eq!(net_change, 0, "different type must not stack");
        assert_eq!(existing.speed_delta, 50, "original unchanged");
    }

    #[test]
    fn test_speed_add_condition_permanent_ignores_timed() {
        let mut existing = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            -1,
            false,
            0,
            50,
            false,
        );
        let incoming = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            8000,
            false,
            0,
            120,
            false,
        );
        let net_change = existing.add_condition(&incoming);
        assert_eq!(net_change, 0, "permanent must not be overridden by timed");
        assert_eq!(existing.speed_delta, 50);
    }

    #[test]
    fn test_speed_serialize_deserialize_roundtrip() {
        let mut original = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            5000,
            true,
            3,
            75,
            false,
        );
        original.set_formula_vars(1.0, 0.5, 2.0, 1.0);

        let mut buf = Vec::new();
        original.serialize(&mut buf);

        let (decoded, _) = ConditionSpeed::deserialize(&buf).expect("deserialize should succeed");
        assert_eq!(decoded.base.ticks, 5000);
        assert_eq!(decoded.speed_delta, 75);
        assert!((decoded.formula_mina - 1.0f32).abs() < f32::EPSILON);
        assert!((decoded.formula_minb - 0.5f32).abs() < f32::EPSILON);
        assert!((decoded.formula_maxa - 2.0f32).abs() < f32::EPSILON);
        assert!((decoded.formula_maxb - 1.0f32).abs() < f32::EPSILON);
        assert_eq!(decoded.base.sub_id, 3);
        assert!(decoded.base.is_buff);
    }

    // --- ConditionLight add_condition & serialize/deserialize ---------------

    #[test]
    fn test_light_add_condition_updates_level_and_interval() {
        let mut existing = ConditionLight::new(ConditionId::Combat, 4000, false, 0, 4, 10, false);
        // Simulate start: set light_change_interval = ticks / level = 4000/4 = 1000
        existing.light_change_interval = 1000;

        let incoming = ConditionLight::new(ConditionId::Combat, 8000, false, 0, 8, 20, false);
        existing.add_condition(&incoming);

        assert_eq!(existing.base.ticks, 8000);
        assert_eq!(existing.light_level, 8);
        assert_eq!(existing.light_color, 20);
        assert_eq!(existing.light_change_interval, 1000, "8000/8 = 1000");
        assert_eq!(existing.internal_light_ticks, 0, "reset on update");
    }

    #[test]
    fn test_light_add_condition_shorter_noop() {
        let mut existing = ConditionLight::new(ConditionId::Combat, 8000, false, 0, 4, 10, false);
        let incoming = ConditionLight::new(ConditionId::Combat, 1000, false, 0, 2, 5, false);
        existing.add_condition(&incoming);
        assert_eq!(existing.base.ticks, 8000, "shorter must not override");
        assert_eq!(existing.light_level, 4, "level unchanged");
    }

    #[test]
    fn test_light_serialize_deserialize_roundtrip() {
        let mut original = ConditionLight::new(ConditionId::Combat, 6000, true, 1, 200, 150, false);
        original.light_change_interval = 500;
        original.internal_light_ticks = 250;

        let mut buf = Vec::new();
        original.serialize(&mut buf);

        let (decoded, _) = ConditionLight::deserialize(&buf).expect("deserialize");
        assert_eq!(decoded.light_level, 200);
        assert_eq!(decoded.light_color, 150);
        assert_eq!(decoded.light_change_interval, 500);
        assert_eq!(decoded.internal_light_ticks, 250);
        assert_eq!(decoded.base.ticks, 6000);
        assert!(decoded.base.is_buff);
    }

    // --- ConditionManaShield add_condition, on_damage_taken, deserialize ----

    #[test]
    fn test_manashield_add_condition_replaces_always() {
        let mut existing = ConditionManaShield::new(ConditionId::Combat, -1, false, 0);
        existing.mana_shield = 100;
        existing.max_mana_shield = 100;

        let mut incoming = ConditionManaShield::new(ConditionId::Combat, 5000, false, 0);
        incoming.mana_shield = 200;

        // C++ always replaces regardless of ticks
        existing.add_condition(&incoming);
        assert_eq!(existing.mana_shield, 200);
        assert_eq!(existing.max_mana_shield, 200);
        assert_eq!(existing.base.ticks, 5000);
    }

    #[test]
    fn test_manashield_on_damage_taken_partial_absorb() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, -1, false, 0);
        shield.mana_shield = 80;
        shield.max_mana_shield = 100;

        let overflow = shield.on_damage_taken(50);
        assert_eq!(overflow, 0, "shield covers 50, no overflow");
        assert_eq!(shield.mana_shield, 30);
    }

    #[test]
    fn test_manashield_on_damage_taken_overflow() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, -1, false, 0);
        shield.mana_shield = 30;
        shield.max_mana_shield = 100;

        let overflow = shield.on_damage_taken(100);
        assert_eq!(overflow, 70, "70 damage overflows the shield");
        assert_eq!(shield.mana_shield, 0, "shield fully drained");
    }

    #[test]
    fn test_manashield_serialize_deserialize_roundtrip() {
        let mut original = ConditionManaShield::new(ConditionId::Combat, 3000, false, 0);
        original.mana_shield = 150;
        original.max_mana_shield = 200;

        let mut buf = Vec::new();
        original.serialize(&mut buf);

        let (decoded, _) = ConditionManaShield::deserialize(&buf).expect("deserialize");
        assert_eq!(decoded.mana_shield, 150);
        assert_eq!(decoded.max_mana_shield, 200);
        assert_eq!(decoded.base.ticks, 3000);
    }

    // --- ConditionRegeneration serialize/deserialize roundtrip --------------

    #[test]
    fn test_regen_serialize_deserialize_roundtrip() {
        let mut original = ConditionRegeneration::new(ConditionId::Combat, 10_000, true, 2, false);
        original.health_ticks = 1500;
        original.health_gain = 8;
        original.mana_ticks = 2500;
        original.mana_gain = 12;

        let mut buf = Vec::new();
        original.serialize(&mut buf);

        let (decoded, _) = ConditionRegeneration::deserialize(&buf).expect("deserialize");
        assert_eq!(decoded.health_ticks, 1500);
        assert_eq!(decoded.health_gain, 8);
        assert_eq!(decoded.mana_ticks, 2500);
        assert_eq!(decoded.mana_gain, 12);
        assert_eq!(decoded.base.ticks, 10_000);
        assert!(decoded.base.is_buff);
        assert_eq!(decoded.base.sub_id, 2);
    }

    // --- ConditionSoul serialize/deserialize roundtrip ----------------------

    #[test]
    fn test_soul_serialize_deserialize_roundtrip() {
        let mut original = ConditionSoul::new(ConditionId::Combat, 20_000, false, 0, false);
        original.soul_gain = 5;
        original.soul_ticks = 3000;

        let mut buf = Vec::new();
        original.serialize(&mut buf);

        let (decoded, _) = ConditionSoul::deserialize(&buf).expect("deserialize");
        assert_eq!(decoded.soul_gain, 5);
        assert_eq!(decoded.soul_ticks, 3000);
        assert_eq!(decoded.base.ticks, 20_000);
    }

    // --- ConditionDamage serialize/deserialize roundtrip --------------------

    #[test]
    fn test_damage_serialize_deserialize_periodic() {
        let mut original = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        original.add_damage(-1, 2000, -15); // periodic

        let mut buf = Vec::new();
        original.serialize(&mut buf);

        let (decoded, _) = ConditionDamage::deserialize(&buf).expect("deserialize");
        assert_eq!(decoded.period_damage, -15);
        assert_eq!(decoded.base.ticks, -1);
    }

    #[test]
    fn test_damage_serialize_deserialize_scheduled() {
        let mut original = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            0,
            true,
        );
        original.add_damage(3, 1000, -20);

        let mut buf = Vec::new();
        original.serialize(&mut buf);

        let (decoded, _) = ConditionDamage::deserialize(&buf).expect("deserialize");
        assert_eq!(decoded.damage_list.len(), 3);
        assert_eq!(decoded.damage_list[0].value, -20);
        assert_eq!(decoded.damage_list[0].interval, 1000);
    }

    // --- ConditionDamage stacking (add_condition / should_update) -----------

    #[test]
    fn test_damage_add_condition_force_update_replaces() {
        let mut existing = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        existing.add_damage(5, 1000, -50); // total 250

        let mut incoming = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        incoming.add_damage(1, 1000, -10); // total 10, less damage
        incoming.force_update = true;

        assert!(
            existing.should_update(&incoming),
            "force_update must always win"
        );
    }

    #[test]
    fn test_damage_total_damage_periodic_returns_period_damage() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        dmg.period_damage = -50;
        // With period_damage set, get_total_damage should return it (abs)
        assert_eq!(dmg.get_total_damage(), 50);
    }

    #[test]
    fn test_damage_add_damage_periodic_rejects_when_period_damage_positive() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        dmg.period_damage = 10; // positive period set
                                // Adding scheduled rounds should fail when periodDamage > 0
        let ok = dmg.add_damage(3, 1000, -20);
        assert!(!ok, "must reject scheduled rounds when period_damage > 0");
    }

    #[test]
    fn test_damage_tick_interval_minimum_clamped_to_500() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            0,
            true,
        );
        // Time of 100 should be clamped to 500 (EVENT_CREATURE_THINK_INTERVAL)
        dmg.add_damage(1, 100, -5);
        assert_eq!(dmg.damage_list[0].interval, 500);
    }

    // --- ConditionOutfit ----------------------------------------------------

    #[test]
    fn test_outfit_new_stores_defaults() {
        let outfit_cond = ConditionOutfit::new(ConditionId::Combat, 5000, false, 0, false);
        assert_eq!(outfit_cond.outfit, Outfit::default());
        assert_eq!(outfit_cond.base.ticks, 5000);
    }

    #[test]
    fn test_outfit_set_outfit() {
        let mut outfit_cond = ConditionOutfit::new(ConditionId::Combat, 5000, false, 0, false);
        let new_outfit = Outfit {
            look_type: 75,
            look_head: 10,
            look_body: 20,
            look_legs: 30,
            look_feet: 40,
            look_addons: 3,
            look_mount: 0,
        };
        outfit_cond.set_outfit(new_outfit.clone());
        assert_eq!(outfit_cond.outfit, new_outfit);
    }

    #[test]
    fn test_outfit_add_condition_updates_when_longer() {
        let mut existing = ConditionOutfit::new(ConditionId::Combat, 1000, false, 0, false);
        existing.outfit = Outfit {
            look_type: 10,
            ..Outfit::default()
        };

        let mut incoming = ConditionOutfit::new(ConditionId::Combat, 9000, false, 0, false);
        incoming.outfit = Outfit {
            look_type: 75,
            ..Outfit::default()
        };

        let updated = existing.add_condition(&incoming);
        assert!(updated);
        assert_eq!(existing.base.ticks, 9000);
        assert_eq!(existing.outfit.look_type, 75);
    }

    #[test]
    fn test_outfit_add_condition_shorter_noop() {
        let mut existing = ConditionOutfit::new(ConditionId::Combat, 9000, false, 0, false);
        existing.outfit = Outfit {
            look_type: 75,
            ..Outfit::default()
        };

        let mut incoming = ConditionOutfit::new(ConditionId::Combat, 1000, false, 0, false);
        incoming.outfit = Outfit {
            look_type: 10,
            ..Outfit::default()
        };

        let updated = existing.add_condition(&incoming);
        assert!(!updated, "shorter condition must not replace");
        assert_eq!(existing.outfit.look_type, 75, "original outfit preserved");
    }

    #[test]
    fn test_outfit_tick_expires() {
        let mut outfit_cond = ConditionOutfit::new(ConditionId::Combat, 1000, false, 0, false);
        assert!(outfit_cond.tick(500));
        assert!(!outfit_cond.tick(501));
    }

    #[test]
    fn test_outfit_serialize_produces_bytes() {
        let mut outfit_cond = ConditionOutfit::new(ConditionId::Combat, 3000, false, 0, false);
        outfit_cond.outfit = Outfit {
            look_type: 128,
            ..Outfit::default()
        };
        let mut buf = Vec::new();
        outfit_cond.serialize(&mut buf);
        assert!(!buf.is_empty());
        // Last byte must be End marker
        assert_eq!(*buf.last().unwrap(), ConditionAttr::End as u8);
    }

    #[test]
    fn test_outfit_serialize_then_deserialize_roundtrip() {
        let mut outfit_cond = ConditionOutfit::new(ConditionId::Combat, 4000, true, 7, true);
        outfit_cond.outfit = Outfit {
            look_type: 130,
            look_head: 11,
            look_body: 22,
            look_legs: 33,
            look_feet: 44,
            look_addons: 3,
            look_mount: 257,
        };
        let mut buf = Vec::new();
        outfit_cond.serialize(&mut buf);

        let (restored, _) = ConditionOutfit::deserialize(&buf)
            .expect("ConditionOutfit::deserialize should succeed");
        assert_eq!(restored.outfit, outfit_cond.outfit);
        assert_eq!(restored.base.ticks, 4000);
        assert!(restored.base.is_buff);
        assert_eq!(restored.base.sub_id, 7);
        assert!(restored.base.aggressive);
    }

    #[test]
    fn test_outfit_deserialize_default_when_attr_absent() {
        // Base attrs followed directly by End — no OUTFIT block.
        let base = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::OUTFIT,
            5000,
            false,
            0,
            false,
        );
        let mut buf = Vec::new();
        base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        let (restored, _) =
            ConditionOutfit::deserialize(&buf).expect("base + End stream should decode");
        assert_eq!(restored.outfit, Outfit::default());
    }

    #[test]
    fn test_outfit_deserialize_truncated_outfit_block_returns_none() {
        let base = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::OUTFIT,
            5000,
            false,
            0,
            false,
        );
        let mut buf = Vec::new();
        base.serialize(&mut buf);
        buf.push(ConditionAttr::Outfit as u8);
        buf.extend_from_slice(&[1, 2, 3, 4, 5]); // only 5 bytes, need 9
        assert!(ConditionOutfit::deserialize(&buf).is_none());
    }

    // --- ConditionInvisible -------------------------------------------------

    #[test]
    fn test_invisible_new() {
        let inv = ConditionInvisible::new(ConditionId::Combat, 3000, false, 0, false);
        assert_eq!(inv.base.condition_type, ConditionTypeFlags::INVISIBLE);
        assert_eq!(inv.base.ticks, 3000);
    }

    #[test]
    fn test_invisible_tick_expires() {
        let mut inv = ConditionInvisible::new(ConditionId::Combat, 2000, false, 0, false);
        assert!(inv.tick(1000));
        assert!(!inv.tick(1001));
    }

    #[test]
    fn test_invisible_add_condition_extends_ticks() {
        let mut existing = ConditionInvisible::new(ConditionId::Combat, 1000, false, 0, false);
        let incoming = ConditionInvisible::new(ConditionId::Combat, 8000, false, 0, false);
        existing.add_condition(&incoming);
        assert_eq!(existing.base.ticks, 8000);
    }

    #[test]
    fn test_invisible_add_condition_shorter_noop() {
        let mut existing = ConditionInvisible::new(ConditionId::Combat, 8000, false, 0, false);
        let incoming = ConditionInvisible::new(ConditionId::Combat, 1000, false, 0, false);
        existing.add_condition(&incoming);
        assert_eq!(existing.base.ticks, 8000, "shorter must not override");
    }

    // --- ConditionDrunk -----------------------------------------------------

    #[test]
    fn test_drunk_default_drunkenness_is_25() {
        let drunk = ConditionDrunk::new(ConditionId::Combat, 5000, false, 0, 0, false);
        assert_eq!(drunk.drunkenness, 25, "zero drunkenness must default to 25");
    }

    #[test]
    fn test_drunk_custom_drunkenness_stored() {
        let drunk = ConditionDrunk::new(ConditionId::Combat, 5000, false, 0, 50, false);
        assert_eq!(drunk.drunkenness, 50);
    }

    #[test]
    fn test_drunk_add_condition_higher_wins() {
        let mut existing = ConditionDrunk::new(ConditionId::Combat, 5000, false, 0, 20, false);
        let incoming = ConditionDrunk::new(ConditionId::Combat, 8000, false, 0, 60, false);
        let updated = existing.add_condition(&incoming);
        assert!(updated);
        assert_eq!(existing.drunkenness, 60);
        assert_eq!(existing.base.ticks, 8000);
    }

    #[test]
    fn test_drunk_add_condition_lower_noop() {
        let mut existing = ConditionDrunk::new(ConditionId::Combat, 5000, false, 0, 60, false);
        let incoming = ConditionDrunk::new(ConditionId::Combat, 8000, false, 0, 20, false);
        let updated = existing.add_condition(&incoming);
        assert!(!updated, "lower drunkenness must not replace");
        assert_eq!(existing.drunkenness, 60, "original drunkenness preserved");
    }

    #[test]
    fn test_drunk_add_condition_equal_noop() {
        let mut existing = ConditionDrunk::new(ConditionId::Combat, 5000, false, 0, 30, false);
        let incoming = ConditionDrunk::new(ConditionId::Combat, 8000, false, 0, 30, false);
        let updated = existing.add_condition(&incoming);
        assert!(
            !updated,
            "equal drunkenness must not replace (strictly higher)"
        );
    }

    #[test]
    fn test_drunk_set_param_drunkenness() {
        let mut drunk = ConditionDrunk::new(ConditionId::Combat, 5000, false, 0, 25, false);
        assert!(drunk.set_param(ConditionParam::Drunkenness, 75));
        assert_eq!(drunk.drunkenness, 75);
    }

    #[test]
    fn test_drunk_tick_expires() {
        let mut drunk = ConditionDrunk::new(ConditionId::Combat, 2000, false, 0, 25, false);
        assert!(drunk.tick(1000));
        assert!(!drunk.tick(1001));
    }

    // --- Condition enum includes new variants -------------------------------

    #[test]
    fn test_condition_enum_outfit_variant() {
        let c = Condition::Outfit(ConditionOutfit::new(
            ConditionId::Combat,
            3000,
            false,
            0,
            false,
        ));
        assert_eq!(c.id(), ConditionId::Combat);
        assert_eq!(c.ticks(), 3000);
        assert_eq!(c.condition_type(), ConditionTypeFlags::OUTFIT);
    }

    #[test]
    fn test_condition_enum_invisible_variant() {
        let c = Condition::Invisible(ConditionInvisible::new(
            ConditionId::Combat,
            5000,
            false,
            0,
            false,
        ));
        assert_eq!(c.condition_type(), ConditionTypeFlags::INVISIBLE);
    }

    #[test]
    fn test_condition_enum_drunk_variant() {
        let c = Condition::Drunk(ConditionDrunk::new(
            ConditionId::Combat,
            4000,
            false,
            0,
            50,
            false,
        ));
        assert_eq!(c.condition_type(), ConditionTypeFlags::DRUNK);
    }

    // --- ConditionBase: branch coverage of helper APIs ---------------------

    #[test]
    fn test_condition_base_is_persistent_default_id() {
        // Mirrors C++: id == CONDITIONID_DEFAULT also makes the condition persistent.
        let base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::POISON,
            1000,
            false,
            0,
            false,
        );
        assert!(base.is_persistent());
    }

    #[test]
    fn test_condition_base_is_persistent_muted_type() {
        // Mirrors C++: even when id is neither Default nor Combat, MUTED type is persistent.
        let base = ConditionBase::new(
            ConditionId::Head,
            ConditionTypeFlags::MUTED,
            1000,
            false,
            0,
            false,
        );
        assert!(base.is_persistent());
    }

    #[test]
    fn test_condition_base_is_not_persistent_unrelated_id_and_type() {
        let base = ConditionBase::new(
            ConditionId::Head,
            ConditionTypeFlags::POISON,
            1000,
            false,
            0,
            false,
        );
        assert!(!base.is_persistent());
    }

    #[test]
    fn test_condition_base_should_update_permanent_blocks_timed() {
        // ticks == -1 + incoming.ticks > 0 → false (mirror C++ updateCondition)
        let existing = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            -1,
            false,
            0,
            false,
        );
        let incoming = ConditionBase::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            5000,
            false,
            0,
            false,
        );
        assert!(!existing.should_update(&incoming));
    }

    #[test]
    fn test_condition_base_set_param_sub_id_stores_value() {
        let mut base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            0,
            false,
        );
        assert!(base.set_param(ConditionParam::SubId, 999));
        assert_eq!(base.sub_id, 999);
    }

    #[test]
    fn test_condition_base_set_param_unknown_returns_false() {
        // ConditionParam::Drunkenness has no handler in ConditionBase → default false
        let mut base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            0,
            false,
        );
        assert!(!base.set_param(ConditionParam::Drunkenness, 50));
    }

    #[test]
    fn test_condition_base_get_param_buff_spell() {
        // BuffSpell branch returns 1 when buff is set, 0 otherwise
        let base_on = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            true,
            0,
            false,
        );
        assert_eq!(base_on.get_param(ConditionParam::BuffSpell), 1);
        let base_off = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(base_off.get_param(ConditionParam::BuffSpell), 0);
    }

    #[test]
    fn test_condition_base_get_param_unknown_returns_i32_max() {
        // Mirrors C++ default branch: std::numeric_limits<int32_t>::max()
        let base = ConditionBase::new(
            ConditionId::Default,
            ConditionTypeFlags::NONE,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(base.get_param(ConditionParam::Speed), i32::MAX as i64);
    }

    // --- condition_id_from_i32: every variant ------------------------------

    #[test]
    fn test_condition_id_from_i32_full_roundtrip() {
        // Exercise every arm of condition_id_from_i32 via ConditionBase::deserialize.
        for (raw, expected) in [
            (-1i32, ConditionId::Default),
            (0, ConditionId::Combat),
            (1, ConditionId::Head),
            (2, ConditionId::Necklace),
            (3, ConditionId::Backpack),
            (4, ConditionId::Armor),
            (5, ConditionId::Right),
            (6, ConditionId::Left),
            (7, ConditionId::Legs),
            (8, ConditionId::Feet),
            (9, ConditionId::Ring),
            (10, ConditionId::Ammo),
        ] {
            let original = ConditionBase {
                id: expected,
                condition_type: ConditionTypeFlags::POISON,
                ticks: 100,
                end_time: 0,
                sub_id: 0,
                is_buff: false,
                aggressive: false,
            };
            let mut buf = Vec::new();
            original.serialize(&mut buf);
            push_u8(&mut buf, ConditionAttr::End as u8);
            let (decoded, _) = ConditionBase::deserialize(&buf).expect("decode");
            assert_eq!(decoded.id, expected, "round-trip failed for raw {raw}");
        }
    }

    #[test]
    fn test_condition_id_from_i32_invalid_returns_none() {
        // Hand-craft a buffer with an unknown id discriminant (99) so we hit
        // the `_ => None` arm of condition_id_from_i32.
        let mut buf = Vec::new();
        push_u8(&mut buf, ConditionAttr::Type as u8);
        push_i32(&mut buf, ConditionTypeFlags::POISON);
        push_u8(&mut buf, ConditionAttr::Id as u8);
        push_i32(&mut buf, 99);
        push_u8(&mut buf, ConditionAttr::Ticks as u8);
        push_i32(&mut buf, 100);
        push_u8(&mut buf, ConditionAttr::End as u8);
        assert!(ConditionBase::deserialize(&buf).is_none());
    }

    // --- ConditionRegeneration: branch coverage ----------------------------

    #[test]
    fn test_regen_set_param_fallthrough_to_base() {
        // Setting a non-regen param should defer to ConditionBase::set_param.
        let mut regen = ConditionRegeneration::new(ConditionId::Combat, 5000, false, 0, false);
        assert!(regen.set_param(ConditionParam::SubId, 42));
        assert_eq!(regen.base.sub_id, 42);
    }

    #[test]
    fn test_regen_get_param_fallthrough_to_base() {
        let regen = ConditionRegeneration::new(ConditionId::Combat, 7500, false, 0, false);
        // Ticks delegates to base
        assert_eq!(regen.get_param(ConditionParam::Ticks), 7500);
    }

    #[test]
    fn test_regen_deserialize_breaks_on_unknown_attribute() {
        // Build a regen buffer ending with an unknown attribute byte (no End marker
        // after it). Deserialize should still succeed and return the data parsed
        // so far (the unknown attr triggers the `_ => break` arm).
        let mut original = ConditionRegeneration::new(ConditionId::Combat, 3000, false, 0, false);
        original.health_ticks = 1500;
        let mut buf = Vec::new();
        original.serialize(&mut buf);
        // Replace the trailing End marker with an unknown attribute byte to hit
        // the unknown-attribute break arm in deserialize.
        *buf.last_mut().unwrap() = 0xEE;
        let (decoded, _) = ConditionRegeneration::deserialize(&buf).expect("decode");
        assert_eq!(decoded.health_ticks, 1500);
    }

    // --- ConditionSoul: branch coverage ------------------------------------

    #[test]
    fn test_soul_set_param_fallthrough_to_base() {
        let mut soul = ConditionSoul::new(ConditionId::Combat, 5000, false, 0, false);
        assert!(soul.set_param(ConditionParam::BuffSpell, 1));
        assert!(soul.base.is_buff);
    }

    #[test]
    fn test_soul_get_param_fallthrough_to_base() {
        let soul = ConditionSoul::new(ConditionId::Combat, 4000, true, 0, false);
        assert_eq!(soul.get_param(ConditionParam::BuffSpell), 1);
    }

    #[test]
    fn test_soul_deserialize_breaks_on_unknown_attribute() {
        let mut original = ConditionSoul::new(ConditionId::Combat, 3000, false, 0, false);
        original.soul_gain = 2;
        original.soul_ticks = 1000;
        let mut buf = Vec::new();
        original.serialize(&mut buf);
        *buf.last_mut().unwrap() = 0xEE;
        let (decoded, _) = ConditionSoul::deserialize(&buf).expect("decode");
        assert_eq!(decoded.soul_gain, 2);
    }

    // --- ConditionDamage: full set/get_param + branch coverage --------------

    #[test]
    fn test_damage_set_get_owner_force_field_period() {
        // Exercise every ConditionDamage::set_param / get_param branch missed in
        // the existing tests.
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            0,
            true,
        );
        dmg.set_param(ConditionParam::Owner, 123);
        dmg.set_param(ConditionParam::ForceUpdate, 1);
        dmg.set_param(ConditionParam::StartValue, 5);
        dmg.set_param(ConditionParam::PeriodicDamage, -7);
        dmg.set_param(ConditionParam::Field, 1);

        assert_eq!(dmg.get_param(ConditionParam::Owner), 123);
        assert_eq!(dmg.get_param(ConditionParam::ForceUpdate), 1);
        assert_eq!(dmg.get_param(ConditionParam::StartValue), 5);
        assert_eq!(dmg.get_param(ConditionParam::PeriodicDamage), -7);
        assert_eq!(dmg.get_param(ConditionParam::Field), 1);
    }

    #[test]
    fn test_damage_set_param_fallthrough_to_base() {
        let mut dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            0,
            true,
        );
        // SubId is handled by ConditionBase
        assert!(dmg.set_param(ConditionParam::SubId, 77));
        assert_eq!(dmg.base.sub_id, 77);
    }

    #[test]
    fn test_damage_get_param_fallthrough_to_base() {
        let dmg = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            5,
            true,
        );
        assert_eq!(dmg.get_param(ConditionParam::SubId), 5);
    }

    #[test]
    fn test_damage_should_update_permanent_blocks_timed() {
        // existing has ticks == -1, incoming has ticks > 0 and no force_update
        let mut existing = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        existing.base.ticks = -1;
        existing.period_damage = 10;

        let mut incoming = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        incoming.base.ticks = 5000;
        incoming.period_damage = 100; // higher damage but permanent blocks

        assert!(!existing.should_update(&incoming));
    }

    #[test]
    fn test_damage_deserialize_breaks_on_unknown_attribute() {
        let mut original = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            true,
        );
        original.add_damage(-1, 1500, -5);
        let mut buf = Vec::new();
        original.serialize(&mut buf);
        *buf.last_mut().unwrap() = 0xEE;
        let (decoded, _) = ConditionDamage::deserialize(&buf).expect("decode");
        assert_eq!(decoded.period_damage, -5);
    }

    // --- ConditionSpeed: fallthrough branches ------------------------------

    #[test]
    fn test_speed_set_param_fallthrough_to_base() {
        let mut speed = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            5000,
            false,
            0,
            10,
            false,
        );
        // SubId handled by base
        assert!(speed.set_param(ConditionParam::SubId, 12));
        assert_eq!(speed.base.sub_id, 12);
    }

    #[test]
    fn test_speed_get_param_fallthrough_to_base() {
        let speed = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            5000,
            false,
            3,
            10,
            false,
        );
        assert_eq!(speed.get_param(ConditionParam::SubId), 3);
    }

    #[test]
    fn test_speed_deserialize_breaks_on_unknown_attribute() {
        let mut original = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            3000,
            false,
            0,
            60,
            false,
        );
        original.set_formula_vars(1.0, 2.0, 3.0, 4.0);
        let mut buf = Vec::new();
        original.serialize(&mut buf);
        *buf.last_mut().unwrap() = 0xEE;
        let (decoded, _) = ConditionSpeed::deserialize(&buf).expect("decode");
        assert_eq!(decoded.speed_delta, 60);
    }

    #[test]
    fn test_speed_deserialize_terminates_at_eof() {
        // Build a buffer that ends without an explicit End marker after the base
        // section.  Speed's deserialize loop should hit `pos >= data.len()` and
        // exit cleanly (line 923 of condition.rs).
        let base = ConditionBase {
            id: ConditionId::Combat,
            condition_type: ConditionTypeFlags::HASTE,
            ticks: 1000,
            end_time: 0,
            sub_id: 0,
            is_buff: false,
            aggressive: false,
        };
        let mut buf = Vec::new();
        base.serialize(&mut buf);
        push_u8(&mut buf, ConditionAttr::End as u8);
        // No speed/formula attrs after End — buffer ends here.
        let (decoded, _) = ConditionSpeed::deserialize(&buf).expect("decode");
        assert_eq!(decoded.base.ticks, 1000);
        assert_eq!(decoded.speed_delta, 0, "default when no SpeedDelta attr");
    }

    // --- ConditionLight: branch coverage -----------------------------------

    #[test]
    fn test_light_set_param_color_branch() {
        let mut light = ConditionLight::new(ConditionId::Combat, 5000, false, 0, 4, 10, false);
        assert!(light.set_param(ConditionParam::LightColor, 88));
        assert_eq!(light.light_color, 88);
    }

    #[test]
    fn test_light_set_param_fallthrough_to_base() {
        let mut light = ConditionLight::new(ConditionId::Combat, 5000, false, 0, 4, 10, false);
        assert!(light.set_param(ConditionParam::SubId, 99));
        assert_eq!(light.base.sub_id, 99);
    }

    #[test]
    fn test_light_get_param_fallthrough_to_base() {
        let light = ConditionLight::new(ConditionId::Combat, 5000, true, 7, 4, 10, false);
        assert_eq!(light.get_param(ConditionParam::SubId), 7);
    }

    #[test]
    fn test_light_add_condition_with_permanent_zeros_change_interval() {
        // existing must be timed (ticks > 0) so should_update accepts incoming.
        // incoming.ticks > existing.ticks AND incoming.ticks > 0 still triggers
        // the `else 0` branch only when self.base.ticks <= 0 AFTER set_ticks.
        // To force light_change_interval = 0, we set incoming.ticks = 0 and
        // existing.ticks negative (-1). But should_update guards against -1 →
        // we instead build manually and call add_condition directly with the
        // incoming ticks = 0 case by giving existing a smaller positive ticks.
        // Since should_update requires other.ticks > self.ticks AND ticks >= 0,
        // we use existing.ticks = -2 (which the guard does NOT special-case)
        // and incoming.ticks = 0 to trigger the else branch.
        //
        // Simpler: directly set internal state and re-invoke the post-conditions
        // we care about by simulating ticks=0 incoming.
        let mut existing = ConditionLight::new(ConditionId::Combat, 1000, false, 0, 4, 10, false);
        // Force ticks negative-but-not-permanent so should_update accepts
        // incoming.ticks=0.  The Rust guard is `other.ticks >= 0 && other.ticks > self.ticks`.
        existing.base.ticks = -5;
        existing.base.condition_type = ConditionTypeFlags::LIGHT;
        let mut incoming = ConditionLight::new(ConditionId::Combat, 0, false, 0, 4, 20, false);
        incoming.base.ticks = 0;
        existing.add_condition(&incoming);
        // After update, ticks = 0 → light_change_interval = 0 (else branch).
        assert_eq!(existing.light_change_interval, 0);
        assert_eq!(existing.light_color, 20);
    }

    #[test]
    fn test_light_deserialize_breaks_on_unknown_attribute() {
        let mut original = ConditionLight::new(ConditionId::Combat, 4000, false, 0, 50, 60, false);
        original.light_change_interval = 100;
        original.internal_light_ticks = 25;
        let mut buf = Vec::new();
        original.serialize(&mut buf);
        *buf.last_mut().unwrap() = 0xEE;
        let (decoded, _) = ConditionLight::deserialize(&buf).expect("decode");
        assert_eq!(decoded.light_level, 50);
    }

    // --- ConditionManaShield: branch coverage ------------------------------

    #[test]
    fn test_manashield_set_param_breakable_sets_shield_and_max() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, 5000, false, 0);
        assert!(shield.set_param(ConditionParam::ManaShieldBreakable, 250));
        assert_eq!(shield.mana_shield, 250);
        assert_eq!(shield.max_mana_shield, 250);
    }

    #[test]
    fn test_manashield_set_param_fallthrough_to_base() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, 5000, false, 0);
        assert!(shield.set_param(ConditionParam::SubId, 9));
        assert_eq!(shield.base.sub_id, 9);
    }

    #[test]
    fn test_manashield_tick_expires() {
        let mut shield = ConditionManaShield::new(ConditionId::Combat, 1500, false, 0);
        assert!(shield.tick(750));
        assert!(!shield.tick(751));
    }

    #[test]
    fn test_manashield_deserialize_breaks_on_unknown_attribute() {
        let mut original = ConditionManaShield::new(ConditionId::Combat, 3000, false, 0);
        original.mana_shield = 50;
        original.max_mana_shield = 100;
        let mut buf = Vec::new();
        original.serialize(&mut buf);
        *buf.last_mut().unwrap() = 0xEE;
        let (decoded, _) = ConditionManaShield::deserialize(&buf).expect("decode");
        assert_eq!(decoded.mana_shield, 50);
    }

    // --- ConditionDrunk: fallthrough -------------------------------------

    #[test]
    fn test_drunk_set_param_fallthrough_to_base() {
        // Non-Drunkenness param defers to ConditionBase::set_param.
        let mut drunk = ConditionDrunk::new(ConditionId::Combat, 5000, false, 0, 25, false);
        assert!(drunk.set_param(ConditionParam::BuffSpell, 1));
        assert!(drunk.base.is_buff);
    }

    // --- Condition enum dispatch: every variant ----------------------------

    #[test]
    fn test_condition_enum_base_dispatch_all_variants() {
        // Cover every arm of Condition::base() / base_mut() by iterating each
        // variant and confirming the dispatched base matches expectations.
        let mut variants: Vec<Condition> = vec![
            Condition::Generic(ConditionGeneric::new(
                ConditionId::Combat,
                ConditionTypeFlags::INFIGHT,
                1000,
                false,
                0,
                false,
            )),
            Condition::Regeneration(ConditionRegeneration::new(
                ConditionId::Combat,
                1000,
                false,
                0,
                false,
            )),
            Condition::Soul(ConditionSoul::new(
                ConditionId::Combat,
                1000,
                false,
                0,
                false,
            )),
            Condition::Damage(ConditionDamage::new(
                ConditionId::Combat,
                ConditionTypeFlags::FIRE,
                false,
                0,
                true,
            )),
            Condition::Speed(ConditionSpeed::new(
                ConditionId::Combat,
                ConditionTypeFlags::HASTE,
                1000,
                false,
                0,
                10,
                false,
            )),
            Condition::Light(ConditionLight::new(
                ConditionId::Combat,
                1000,
                false,
                0,
                5,
                5,
                false,
            )),
            Condition::ManaShield(ConditionManaShield::new(
                ConditionId::Combat,
                1000,
                false,
                0,
            )),
            Condition::Outfit(ConditionOutfit::new(
                ConditionId::Combat,
                1000,
                false,
                0,
                false,
            )),
            Condition::Invisible(ConditionInvisible::new(
                ConditionId::Combat,
                1000,
                false,
                0,
                false,
            )),
            Condition::Drunk(ConditionDrunk::new(
                ConditionId::Combat,
                1000,
                false,
                0,
                25,
                false,
            )),
        ];

        for c in &variants {
            // base() arm must return a ConditionBase whose id matches Combat.
            assert_eq!(c.base().id, ConditionId::Combat);
        }

        for c in &mut variants {
            // base_mut() arm — mutate via dispatch.
            c.base_mut().sub_id = 42;
            assert_eq!(c.base().sub_id, 42);
        }
    }

    // ── get_icons (Session 37) ──────────────────────────────────────────

    /// Buff condition → ICON_PARTY_BUFF (1 << 12 = 4096).
    #[test]
    fn get_icons_buff_returns_party_buff_bit() {
        let cb = ConditionBase::new(ConditionId::Combat, 0, 1000, true, 0, false);
        assert_eq!(cb.get_icons(), 1 << 12);
    }

    /// Non-buff condition → 0.
    #[test]
    fn get_icons_non_buff_returns_zero() {
        let cb = ConditionBase::new(ConditionId::Combat, 0, 1000, false, 0, false);
        assert_eq!(cb.get_icons(), 0);
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes (Session 47)
    // -----------------------------------------------------------------------

    fn make_attrs() -> ConditionAttributes {
        ConditionAttributes::new(ConditionId::Combat, 5000, false, 0, false)
    }

    #[test]
    fn attrs_new_defaults_zero() {
        let a = make_attrs();
        assert_eq!(a.skills, [0; ATTR_SKILL_COUNT]);
        assert_eq!(a.stats, [0; ATTR_STAT_COUNT]);
        assert_eq!(a.special_skills, [0; ATTR_SPECIALSKILL_COUNT]);
        assert!(!a.disable_defense);
        assert_eq!(a.base.condition_type, ConditionTypeFlags::ATTRIBUTES);
    }

    #[test]
    fn attrs_set_param_skill_melee_writes_club_axe_sword() {
        use forgottenserver_common::enums::Skill;
        let mut a = make_attrs();
        assert!(a.set_param(ConditionParam::SkillMelee, 7));
        assert_eq!(a.skills[Skill::Club as usize], 7);
        assert_eq!(a.skills[Skill::Axe as usize], 7);
        assert_eq!(a.skills[Skill::Sword as usize], 7);
        // Fist / Distance / Shield / Fishing untouched
        assert_eq!(a.skills[Skill::Fist as usize], 0);
        assert_eq!(a.skills[Skill::Distance as usize], 0);
    }

    #[test]
    fn attrs_set_param_skill_melee_percent_writes_club_axe_sword_percent() {
        use forgottenserver_common::enums::Skill;
        let mut a = make_attrs();
        assert!(a.set_param(ConditionParam::SkillMeleePercent, 15));
        assert_eq!(a.skills_percent[Skill::Club as usize], 15);
        assert_eq!(a.skills_percent[Skill::Axe as usize], 15);
        assert_eq!(a.skills_percent[Skill::Sword as usize], 15);
    }

    #[test]
    fn attrs_set_param_individual_skills() {
        use forgottenserver_common::enums::Skill;
        let mut a = make_attrs();
        a.set_param(ConditionParam::SkillFist, 1);
        a.set_param(ConditionParam::SkillDistance, 2);
        a.set_param(ConditionParam::SkillShield, 3);
        a.set_param(ConditionParam::SkillFishing, 4);
        assert_eq!(a.skills[Skill::Fist as usize], 1);
        assert_eq!(a.skills[Skill::Distance as usize], 2);
        assert_eq!(a.skills[Skill::Shield as usize], 3);
        assert_eq!(a.skills[Skill::Fishing as usize], 4);
    }

    #[test]
    fn attrs_set_param_stats() {
        use forgottenserver_common::enums::Stat;
        let mut a = make_attrs();
        a.set_param(ConditionParam::StatMaxHitPoints, 100);
        a.set_param(ConditionParam::StatMaxManaPoints, 200);
        a.set_param(ConditionParam::StatMagicPoints, 10);
        a.set_param(ConditionParam::StatMaxHitPointsPercent, 25);
        assert_eq!(a.stats[Stat::MaxHitPoints as usize], 100);
        assert_eq!(a.stats[Stat::MaxManaPoints as usize], 200);
        assert_eq!(a.stats[Stat::MagicPoints as usize], 10);
        assert_eq!(a.stats_percent[Stat::MaxHitPoints as usize], 25);
    }

    #[test]
    fn attrs_set_param_special_skills() {
        use forgottenserver_common::enums::SpecialSkill;
        let mut a = make_attrs();
        a.set_param(ConditionParam::SpecialSkillCriticalHitChance, 5);
        a.set_param(ConditionParam::SpecialSkillLifeLeechAmount, 9);
        assert_eq!(
            a.special_skills[SpecialSkill::CriticalHitChance as usize],
            5
        );
        assert_eq!(a.special_skills[SpecialSkill::LifeLeechAmount as usize], 9);
    }

    #[test]
    fn attrs_set_param_disable_defense_toggle() {
        let mut a = make_attrs();
        a.set_param(ConditionParam::DisableDefense, 1);
        assert!(a.disable_defense);
        a.set_param(ConditionParam::DisableDefense, 0);
        assert!(!a.disable_defense);
    }

    #[test]
    fn attrs_set_param_falls_through_to_base_for_sub_id() {
        let mut a = make_attrs();
        a.set_param(ConditionParam::SubId, 99);
        assert_eq!(a.base.sub_id, 99);
    }

    #[test]
    fn attrs_add_condition_copies_arrays_when_should_update() {
        let mut existing = make_attrs();
        existing.set_param(ConditionParam::SkillFist, 1);

        let mut incoming = ConditionAttributes::new(ConditionId::Combat, 9000, false, 0, false);
        incoming.set_param(ConditionParam::SkillFist, 5);
        incoming.set_param(ConditionParam::StatMaxHitPoints, 100);
        incoming.set_param(ConditionParam::DisableDefense, 1);

        existing.add_condition(&incoming);
        assert_eq!(existing.skills, incoming.skills);
        assert_eq!(existing.stats, incoming.stats);
        assert!(existing.disable_defense);
        assert_eq!(existing.base.ticks, 9000);
    }

    #[test]
    fn attrs_serialize_then_deserialize_roundtrips() {
        let mut a = make_attrs();
        a.set_param(ConditionParam::SkillFist, 3);
        a.set_param(ConditionParam::SkillFishing, 7);
        a.set_param(ConditionParam::StatMaxHitPoints, 100);
        a.set_param(ConditionParam::StatMagicPoints, 10);
        a.set_param(ConditionParam::SpecialSkillCriticalHitAmount, 4);
        a.set_param(ConditionParam::DisableDefense, 1);

        let mut buf = Vec::new();
        a.serialize(&mut buf);
        let (restored, _) =
            ConditionAttributes::deserialize(&buf).expect("attrs roundtrip should succeed");
        assert_eq!(restored.skills, a.skills);
        assert_eq!(restored.stats, a.stats);
        assert_eq!(restored.special_skills, a.special_skills);
        assert_eq!(restored.disable_defense, a.disable_defense);
        assert_eq!(restored.base.ticks, a.base.ticks);
    }

    // -----------------------------------------------------------------------
    // create_condition factory (Session 47)
    // -----------------------------------------------------------------------

    #[test]
    fn create_condition_damage_family() {
        for ct in [
            ConditionTypeFlags::POISON,
            ConditionTypeFlags::FIRE,
            ConditionTypeFlags::ENERGY,
            ConditionTypeFlags::DROWN,
            ConditionTypeFlags::FREEZING,
            ConditionTypeFlags::DAZZLED,
            ConditionTypeFlags::CURSED,
            ConditionTypeFlags::BLEEDING,
        ] {
            let c = create_condition(ConditionId::Combat, ct, 0, 0, false, 0, true);
            assert!(
                matches!(c, Some(Condition::Damage(_))),
                "type={ct} -> Damage"
            );
        }
    }

    #[test]
    fn create_condition_haste_paralyze_routes_speed_delta() {
        // HASTE → speed_delta = param
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            5000,
            42,
            false,
            0,
            false,
        )
        .expect("HASTE");
        if let Condition::Speed(s) = c {
            assert_eq!(s.speed_delta, 42);
            assert_eq!(s.base.condition_type, ConditionTypeFlags::HASTE);
        } else {
            panic!("expected Condition::Speed for HASTE");
        }
        // PARALYZE → preserves PARALYZE type
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::PARALYZE,
            5000,
            -10,
            false,
            0,
            true,
        )
        .expect("PARALYZE");
        if let Condition::Speed(s) = c {
            assert_eq!(s.speed_delta, -10);
            assert_eq!(s.base.condition_type, ConditionTypeFlags::PARALYZE);
        } else {
            panic!("expected Condition::Speed for PARALYZE");
        }
    }

    #[test]
    fn create_condition_light_packs_level_and_color() {
        let level = 7u8;
        let color = 215u8;
        let param = (level as i32) | ((color as i32) << 8);
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::LIGHT,
            5000,
            param,
            false,
            0,
            false,
        )
        .expect("LIGHT");
        if let Condition::Light(l) = c {
            assert_eq!(l.light_color, color);
            assert_eq!(l.light_level, level);
        } else {
            panic!("expected Condition::Light");
        }
    }

    #[test]
    fn create_condition_attributes_returns_attributes_variant() {
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::ATTRIBUTES,
            5000,
            0,
            true,
            0,
            false,
        )
        .expect("ATTRIBUTES");
        assert!(matches!(c, Condition::Attributes(_)));
    }

    #[test]
    fn create_condition_misc_marker_types_become_generic() {
        for ct in [
            ConditionTypeFlags::SPELLCOOLDOWN,
            ConditionTypeFlags::SPELLGROUPCOOLDOWN,
            ConditionTypeFlags::INFIGHT,
            ConditionTypeFlags::MUTED,
            ConditionTypeFlags::MANASHIELD,
        ] {
            let c = create_condition(ConditionId::Combat, ct, 5000, 0, false, 0, false);
            assert!(
                matches!(c, Some(Condition::Generic(_))),
                "type={ct} -> Generic"
            );
        }
    }

    #[test]
    fn create_condition_unknown_returns_none() {
        let unknown = 1 << 31;
        let c = create_condition(ConditionId::Combat, unknown, 5000, 0, false, 0, false);
        assert!(c.is_none());
    }

    // -----------------------------------------------------------------------
    // get_icons — subclass overrides
    // -----------------------------------------------------------------------

    #[test]
    fn damage_get_icons_fire_returns_icon_burn() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.base.is_buff = false;
        assert_eq!(d.get_icons(), 1 << 1); // ICON_BURN
    }

    #[test]
    fn damage_get_icons_poison_returns_icon_poison() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::POISON,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons(), 1 << 0); // ICON_POISON
    }

    #[test]
    fn damage_get_icons_energy_returns_icon_energy() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::ENERGY,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons(), 1 << 2); // ICON_ENERGY
    }

    #[test]
    fn damage_get_icons_bleeding_returns_icon_bleeding() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::BLEEDING,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons(), 1 << 15); // ICON_BLEEDING
    }

    #[test]
    fn damage_get_icons_buff_includes_party_buff() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.base.is_buff = true;
        assert_eq!(d.get_icons(), (1 << 12) | (1 << 1)); // ICON_PARTY_BUFF | ICON_BURN
    }

    #[test]
    fn speed_get_icons_haste_returns_icon_haste() {
        let s = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            1000,
            false,
            0,
            100,
            false,
        );
        assert_eq!(s.get_icons(), 1 << 6); // ICON_HASTE
    }

    #[test]
    fn speed_get_icons_paralyze_returns_icon_paralyze() {
        let s = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::PARALYZE,
            1000,
            false,
            0,
            -100,
            false,
        );
        assert_eq!(s.get_icons(), 1 << 5); // ICON_PARALYZE
    }

    #[test]
    fn manashield_get_icons_returns_manashield_breakable_bit() {
        let m = ConditionManaShield::new(ConditionId::Combat, 1000, false, 0);
        assert_eq!(m.get_icons(), 1 << 26); // ICON_MANASHIELD_BREAKABLE
    }

    #[test]
    fn drunk_get_icons_returns_icon_drunk() {
        let d = ConditionDrunk::new(ConditionId::Combat, 1000, false, 0, 25, false);
        assert_eq!(d.get_icons(), 1 << 3); // ICON_DRUNK
    }

    #[test]
    fn generic_get_icons_infight_returns_icon_swords() {
        let g = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::INFIGHT,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(g.get_icons(), 1 << 7); // ICON_SWORDS
    }

    #[test]
    fn generic_get_icons_manashield_returns_icon_manashield() {
        let g = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::MANASHIELD,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(g.get_icons(), 1 << 4); // ICON_MANASHIELD
    }

    // -----------------------------------------------------------------------
    // ConditionDamage::init / get_next_damage
    // -----------------------------------------------------------------------

    #[test]
    fn damage_init_builds_list_from_min_max() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.min_damage = 10;
        d.max_damage = 20;
        d.start_damage = 5;
        let result = d.init();
        assert!(result);
        assert!(!d.damage_list.is_empty());
    }

    #[test]
    fn damage_init_noop_if_period_damage_set() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.period_damage = 50;
        assert!(d.init());
        assert!(d.damage_list.is_empty()); // not populated from min/max
    }

    #[test]
    fn damage_init_noop_if_list_already_populated() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.damage_list.push(IntervalInfo {
            time_left: 1000,
            value: -5,
            interval: 1000,
        });
        d.min_damage = 10;
        d.max_damage = 20;
        let len_before = d.damage_list.len();
        assert!(d.init());
        assert_eq!(d.damage_list.len(), len_before); // unchanged
    }

    #[test]
    fn damage_get_next_damage_periodic() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.period_damage = -10;
        assert_eq!(d.get_next_damage(), Some(-10));
        assert_eq!(d.get_next_damage(), Some(-10)); // idempotent for periodic
    }

    #[test]
    fn damage_get_next_damage_list_pops_when_finite() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.base.ticks = 3000;
        d.damage_list.push(IntervalInfo {
            time_left: 1000,
            value: -5,
            interval: 1000,
        });
        d.damage_list.push(IntervalInfo {
            time_left: 1000,
            value: -3,
            interval: 1000,
        });
        assert_eq!(d.get_next_damage(), Some(-5));
        assert_eq!(d.damage_list.len(), 1);
    }

    #[test]
    fn damage_get_next_damage_empty_returns_none() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        assert_eq!(d.get_next_damage(), None);
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes::get_param
    // -----------------------------------------------------------------------

    #[test]
    fn attrs_get_param_reads_back_skills() {
        use forgottenserver_common::enums::Skill;
        let mut a = make_attrs();
        a.set_param(ConditionParam::SkillFist, 7);
        a.set_param(ConditionParam::SkillFistPercent, 50);
        a.set_param(ConditionParam::SkillClub, 3);
        assert_eq!(a.get_param(ConditionParam::SkillFist), 7);
        assert_eq!(a.get_param(ConditionParam::SkillFistPercent), 50);
        assert_eq!(a.get_param(ConditionParam::SkillClub), 3);
        let _ = Skill::Fist;
    }

    #[test]
    fn attrs_get_param_reads_back_stats() {
        let mut a = make_attrs();
        a.set_param(ConditionParam::StatMaxHitPoints, 100);
        a.set_param(ConditionParam::StatMagicPoints, 25);
        assert_eq!(a.get_param(ConditionParam::StatMaxHitPoints), 100);
        assert_eq!(a.get_param(ConditionParam::StatMagicPoints), 25);
    }

    #[test]
    fn attrs_get_param_reads_back_special_skills() {
        let mut a = make_attrs();
        a.set_param(ConditionParam::SpecialSkillCriticalHitChance, 15);
        a.set_param(ConditionParam::SpecialSkillManaLeechAmount, 8);
        assert_eq!(
            a.get_param(ConditionParam::SpecialSkillCriticalHitChance),
            15
        );
        assert_eq!(a.get_param(ConditionParam::SpecialSkillManaLeechAmount), 8);
    }

    #[test]
    fn attrs_get_param_disable_defense() {
        let mut a = make_attrs();
        a.set_param(ConditionParam::DisableDefense, 1);
        assert_eq!(a.get_param(ConditionParam::DisableDefense), 1);
        a.set_param(ConditionParam::DisableDefense, 0);
        assert_eq!(a.get_param(ConditionParam::DisableDefense), 0);
    }

    // -----------------------------------------------------------------------
    // ConditionRegeneration::deserialize truncated-data error paths (lines 434,
    // 444, 451, 458, 465)
    // -----------------------------------------------------------------------

    fn make_regen_base_bytes() -> Vec<u8> {
        let regen = ConditionRegeneration::new(ConditionId::Combat, 5000, false, 0, false);
        let mut buf = Vec::new();
        regen.base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        buf
    }

    #[test]
    fn regen_deserialize_pos_ge_data_len_breaks_loop() {
        let base_bytes = make_regen_base_bytes();
        let (decoded, _) =
            ConditionRegeneration::deserialize(&base_bytes).expect("should succeed with just base");
        assert_eq!(decoded.health_ticks, 1000);
        assert_eq!(decoded.mana_ticks, 1000);
    }

    #[test]
    fn regen_deserialize_truncated_health_ticks_returns_none() {
        let mut buf = make_regen_base_bytes();
        buf.push(ConditionAttr::HealthTicks as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionRegeneration::deserialize(&buf).is_none());
    }

    #[test]
    fn regen_deserialize_truncated_health_gain_returns_none() {
        let mut buf = make_regen_base_bytes();
        buf.push(ConditionAttr::HealthGain as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionRegeneration::deserialize(&buf).is_none());
    }

    #[test]
    fn regen_deserialize_truncated_mana_ticks_returns_none() {
        let mut buf = make_regen_base_bytes();
        buf.push(ConditionAttr::ManaTicks as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionRegeneration::deserialize(&buf).is_none());
    }

    #[test]
    fn regen_deserialize_truncated_mana_gain_returns_none() {
        let mut buf = make_regen_base_bytes();
        buf.push(ConditionAttr::ManaGain as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionRegeneration::deserialize(&buf).is_none());
    }

    // -----------------------------------------------------------------------
    // ConditionSoul::deserialize truncated-data error paths (lines 576, 586,
    // 593)
    // -----------------------------------------------------------------------

    fn make_soul_base_bytes() -> Vec<u8> {
        let soul = ConditionSoul::new(ConditionId::Combat, 5000, false, 0, false);
        let mut buf = Vec::new();
        soul.base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        buf
    }

    #[test]
    fn soul_deserialize_pos_ge_data_len_breaks_loop() {
        let base_bytes = make_soul_base_bytes();
        let (decoded, _) =
            ConditionSoul::deserialize(&base_bytes).expect("should succeed with just base");
        assert_eq!(decoded.soul_gain, 0);
        assert_eq!(decoded.soul_ticks, 0);
    }

    #[test]
    fn soul_deserialize_truncated_soul_gain_returns_none() {
        let mut buf = make_soul_base_bytes();
        buf.push(ConditionAttr::SoulGain as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionSoul::deserialize(&buf).is_none());
    }

    #[test]
    fn soul_deserialize_truncated_soul_ticks_returns_none() {
        let mut buf = make_soul_base_bytes();
        buf.push(ConditionAttr::SoulTicks as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionSoul::deserialize(&buf).is_none());
    }

    // -----------------------------------------------------------------------
    // ConditionDamage::set_init_damage (lines 755-756)
    // -----------------------------------------------------------------------

    #[test]
    fn damage_set_init_damage_stores_value() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.set_init_damage(42);
        assert_eq!(d.init_damage, 42);
        d.set_init_damage(-7);
        assert_eq!(d.init_damage, -7);
    }

    // -----------------------------------------------------------------------
    // ConditionDamage::generate_damage_list edge cases (lines 769, 779-780)
    // -----------------------------------------------------------------------

    #[test]
    fn generate_damage_list_zero_start_returns_empty() {
        let mut list = Vec::new();
        ConditionDamage::generate_damage_list(100, 0, &mut list);
        assert!(list.is_empty());
    }

    #[test]
    fn generate_damage_list_zero_amount_returns_empty() {
        let mut list = Vec::new();
        ConditionDamage::generate_damage_list(0, 10, &mut list);
        assert!(list.is_empty());
    }

    #[test]
    fn generate_damage_list_negative_start_returns_empty() {
        let mut list = Vec::new();
        ConditionDamage::generate_damage_list(50, -1, &mut list);
        assert!(list.is_empty());
    }

    #[test]
    fn generate_damage_list_start_much_larger_than_amount_hits_med_zero() {
        let mut list = Vec::new();
        ConditionDamage::generate_damage_list(1, 100, &mut list);
    }

    // -----------------------------------------------------------------------
    // ConditionDamage::get_icons remaining branches (lines 817, 821, 823, 825,
    // 829)
    // -----------------------------------------------------------------------

    #[test]
    fn damage_get_icons_drown() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::DROWN,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons() & (1 << 8), 1 << 8);
    }

    #[test]
    fn damage_get_icons_freezing() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FREEZING,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons() & (1 << 9), 1 << 9);
    }

    #[test]
    fn damage_get_icons_dazzled() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::DAZZLED,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons() & (1 << 10), 1 << 10);
    }

    #[test]
    fn damage_get_icons_cursed() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::CURSED,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons() & (1 << 11), 1 << 11);
    }

    #[test]
    fn damage_get_icons_bleeding() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::BLEEDING,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons() & (1 << 15), 1 << 15);
    }

    #[test]
    fn damage_get_icons_else_branch() {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::ATTRIBUTES,
            false,
            0,
            false,
        );
        assert_eq!(d.get_icons(), 0);
    }

    // -----------------------------------------------------------------------
    // ConditionDamage::init start_damage branches (lines 847, 849)
    // -----------------------------------------------------------------------

    #[test]
    fn damage_init_start_damage_exceeds_max_clamps_to_max() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.min_damage = 10;
        d.max_damage = 10;
        d.start_damage = 20;
        d.tick_interval = 2000;
        let result = d.init();
        assert!(d.start_damage <= 10);
        let _ = result;
    }

    #[test]
    fn damage_init_start_damage_zero_gets_computed() {
        let mut d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        d.min_damage = 20;
        d.max_damage = 20;
        d.start_damage = 0;
        d.tick_interval = 2000;
        let result = d.init();
        assert!(d.start_damage >= 1);
        let _ = result;
    }

    // -----------------------------------------------------------------------
    // ConditionDamage::deserialize truncated-data error paths (lines 900, 910,
    // 917, 924)
    // -----------------------------------------------------------------------

    fn make_damage_base_bytes() -> Vec<u8> {
        let d = ConditionDamage::new(
            ConditionId::Combat,
            ConditionTypeFlags::FIRE,
            false,
            0,
            false,
        );
        let mut buf = Vec::new();
        d.base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        buf
    }

    #[test]
    fn damage_deserialize_pos_ge_data_len_breaks_loop() {
        let base_bytes = make_damage_base_bytes();
        let (decoded, _) =
            ConditionDamage::deserialize(&base_bytes).expect("should succeed with just base");
        assert!(!decoded.delayed);
        assert_eq!(decoded.period_damage, 0);
    }

    #[test]
    fn damage_deserialize_truncated_delayed_returns_none() {
        let mut buf = make_damage_base_bytes();
        buf.push(ConditionAttr::Delayed as u8);
        assert!(ConditionDamage::deserialize(&buf).is_none());
    }

    #[test]
    fn damage_deserialize_truncated_period_damage_returns_none() {
        let mut buf = make_damage_base_bytes();
        buf.push(ConditionAttr::PeriodDamage as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionDamage::deserialize(&buf).is_none());
    }

    #[test]
    fn damage_deserialize_truncated_interval_data_returns_none() {
        let mut buf = make_damage_base_bytes();
        buf.push(ConditionAttr::IntervalData as u8);
        buf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        assert!(ConditionDamage::deserialize(&buf).is_none());
    }

    // -----------------------------------------------------------------------
    // ConditionSpeed::get_icons else branch (line 1037)
    // -----------------------------------------------------------------------

    #[test]
    fn speed_get_icons_else_branch_returns_base_icons() {
        let s = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::ATTRIBUTES,
            1000,
            false,
            0,
            10,
            false,
        );
        assert_eq!(s.get_icons(), 0);
    }

    // -----------------------------------------------------------------------
    // ConditionSpeed::deserialize truncated-data error paths (lines 1103, 1110,
    // 1117, 1124, 1131)
    // -----------------------------------------------------------------------

    fn make_speed_base_bytes() -> Vec<u8> {
        let s = ConditionSpeed::new(
            ConditionId::Combat,
            ConditionTypeFlags::HASTE,
            1000,
            false,
            0,
            0,
            false,
        );
        let mut buf = Vec::new();
        s.base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        buf
    }

    #[test]
    fn speed_deserialize_truncated_speed_delta_returns_none() {
        let mut buf = make_speed_base_bytes();
        buf.push(ConditionAttr::SpeedDelta as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionSpeed::deserialize(&buf).is_none());
    }

    #[test]
    fn speed_deserialize_truncated_formula_mina_returns_none() {
        let mut buf = make_speed_base_bytes();
        buf.push(ConditionAttr::FormulaMina as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionSpeed::deserialize(&buf).is_none());
    }

    #[test]
    fn speed_deserialize_truncated_formula_minb_returns_none() {
        let mut buf = make_speed_base_bytes();
        buf.push(ConditionAttr::FormulaMinb as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionSpeed::deserialize(&buf).is_none());
    }

    #[test]
    fn speed_deserialize_truncated_formula_maxa_returns_none() {
        let mut buf = make_speed_base_bytes();
        buf.push(ConditionAttr::FormulaMaxa as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionSpeed::deserialize(&buf).is_none());
    }

    #[test]
    fn speed_deserialize_truncated_formula_maxb_returns_none() {
        let mut buf = make_speed_base_bytes();
        buf.push(ConditionAttr::FormulaMaxb as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionSpeed::deserialize(&buf).is_none());
    }

    // -----------------------------------------------------------------------
    // ConditionLight::deserialize truncated-data error paths (lines 1259, 1269,
    // 1276, 1283, 1290)
    // -----------------------------------------------------------------------

    fn make_light_base_bytes() -> Vec<u8> {
        let l = ConditionLight::new(ConditionId::Combat, 1000, false, 0, 10, 5, false);
        let mut buf = Vec::new();
        l.base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        buf
    }

    #[test]
    fn light_deserialize_pos_ge_data_len_breaks_loop() {
        let base_bytes = make_light_base_bytes();
        let (decoded, _) =
            ConditionLight::deserialize(&base_bytes).expect("should succeed with just base");
        assert_eq!(decoded.light_color, 0);
        assert_eq!(decoded.light_level, 0);
    }

    #[test]
    fn light_deserialize_truncated_light_color_returns_none() {
        let mut buf = make_light_base_bytes();
        buf.push(ConditionAttr::LightColor as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionLight::deserialize(&buf).is_none());
    }

    #[test]
    fn light_deserialize_truncated_light_level_returns_none() {
        let mut buf = make_light_base_bytes();
        buf.push(ConditionAttr::LightLevel as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionLight::deserialize(&buf).is_none());
    }

    #[test]
    fn light_deserialize_truncated_light_ticks_returns_none() {
        let mut buf = make_light_base_bytes();
        buf.push(ConditionAttr::LightTicks as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionLight::deserialize(&buf).is_none());
    }

    #[test]
    fn light_deserialize_truncated_light_interval_returns_none() {
        let mut buf = make_light_base_bytes();
        buf.push(ConditionAttr::LightInterval as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionLight::deserialize(&buf).is_none());
    }

    // -----------------------------------------------------------------------
    // ConditionManaShield::get_icons else branch (line 1387)
    // -----------------------------------------------------------------------

    #[test]
    fn manashield_get_icons_non_breakable_returns_base_icons() {
        let mut ms = ConditionManaShield::new(ConditionId::Combat, 1000, false, 0);
        ms.base.condition_type = ConditionTypeFlags::MANASHIELD;
        assert_eq!(ms.get_icons(), 0);
    }

    // -----------------------------------------------------------------------
    // ConditionManaShield::deserialize truncated-data error paths (lines 1411,
    // 1421, 1428)
    // -----------------------------------------------------------------------

    fn make_manashield_base_bytes() -> Vec<u8> {
        let ms = ConditionManaShield::new(ConditionId::Combat, 1000, false, 0);
        let mut buf = Vec::new();
        ms.base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        buf
    }

    #[test]
    fn manashield_deserialize_pos_ge_data_len_breaks_loop() {
        let base_bytes = make_manashield_base_bytes();
        let (decoded, _) =
            ConditionManaShield::deserialize(&base_bytes).expect("should succeed with just base");
        assert_eq!(decoded.mana_shield, 0);
        assert_eq!(decoded.max_mana_shield, 0);
    }

    #[test]
    fn manashield_deserialize_truncated_mana_returns_none() {
        let mut buf = make_manashield_base_bytes();
        buf.push(ConditionAttr::ManaShieldBreakableMana as u8);
        buf.push(0x01);
        assert!(ConditionManaShield::deserialize(&buf).is_none());
    }

    #[test]
    fn manashield_deserialize_truncated_max_mana_returns_none() {
        let mut buf = make_manashield_base_bytes();
        buf.push(ConditionAttr::ManaShieldBreakableMaxMana as u8);
        buf.push(0x01);
        assert!(ConditionManaShield::deserialize(&buf).is_none());
    }

    // -----------------------------------------------------------------------
    // ConditionOutfit::deserialize unknown-attr break (line 1553)
    // -----------------------------------------------------------------------

    #[test]
    fn outfit_deserialize_unknown_attr_breaks_loop() {
        let outfit = ConditionOutfit::new(ConditionId::Combat, 1000, false, 0, false);
        let mut buf = Vec::new();
        outfit.base.serialize(&mut buf);
        buf.push(0xFF);
        let (decoded, _) = ConditionOutfit::deserialize(&buf).expect("should succeed");
        assert_eq!(decoded.outfit, Outfit::default());
    }

    // -----------------------------------------------------------------------
    // ConditionGeneric::get_icons branches (lines 1702-1703, 1705)
    // -----------------------------------------------------------------------

    #[test]
    fn generic_get_icons_manashield_branch() {
        let g = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::MANASHIELD,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(g.get_icons() & (1 << 4), 1 << 4);
    }

    #[test]
    fn generic_get_icons_infight_branch() {
        let g = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::INFIGHT,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(g.get_icons() & (1 << 7), 1 << 7);
    }

    #[test]
    fn generic_get_icons_root_branch() {
        let g = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::ROOT,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(g.get_icons() & (1 << 19), 1 << 19);
    }

    #[test]
    fn generic_get_icons_else_branch() {
        let g = ConditionGeneric::new(
            ConditionId::Combat,
            ConditionTypeFlags::DRUNK,
            1000,
            false,
            0,
            false,
        );
        assert_eq!(g.get_icons(), 0);
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes::tick (lines 1773-1774)
    // -----------------------------------------------------------------------

    #[test]
    fn attrs_tick_decrements_and_expires() {
        let mut a = make_attrs();
        a.base.ticks = 1000;
        let alive = a.tick(500);
        assert!(alive);
        assert_eq!(a.base.ticks, 500);
        let alive2 = a.tick(500);
        assert!(!alive2);
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes::set_param individual skill setters (lines 1817-1858)
    // -----------------------------------------------------------------------

    #[test]
    fn attrs_set_param_club_sword_axe_distance_shield_fishing_and_percents_extended() {
        use forgottenserver_common::enums::Skill;
        let mut a = make_attrs();
        assert!(a.set_param(ConditionParam::SkillClub, 5));
        assert_eq!(a.skills[Skill::Club as usize], 5);
        assert!(a.set_param(ConditionParam::SkillClubPercent, 10));
        assert_eq!(a.skills_percent[Skill::Club as usize], 10);
        assert!(a.set_param(ConditionParam::SkillSword, 6));
        assert_eq!(a.skills[Skill::Sword as usize], 6);
        assert!(a.set_param(ConditionParam::SkillSwordPercent, 11));
        assert_eq!(a.skills_percent[Skill::Sword as usize], 11);
        assert!(a.set_param(ConditionParam::SkillAxe, 7));
        assert_eq!(a.skills[Skill::Axe as usize], 7);
        assert!(a.set_param(ConditionParam::SkillAxePercent, 12));
        assert_eq!(a.skills_percent[Skill::Axe as usize], 12);
        assert!(a.set_param(ConditionParam::SkillDistance, 8));
        assert_eq!(a.skills[Skill::Distance as usize], 8);
        assert!(a.set_param(ConditionParam::SkillDistancePercent, 13));
        assert_eq!(a.skills_percent[Skill::Distance as usize], 13);
        assert!(a.set_param(ConditionParam::SkillShield, 9));
        assert_eq!(a.skills[Skill::Shield as usize], 9);
        assert!(a.set_param(ConditionParam::SkillShieldPercent, 14));
        assert_eq!(a.skills_percent[Skill::Shield as usize], 14);
        assert!(a.set_param(ConditionParam::SkillFishing, 11));
        assert_eq!(a.skills[Skill::Fishing as usize], 11);
        assert!(a.set_param(ConditionParam::SkillFishingPercent, 22));
        assert_eq!(a.skills_percent[Skill::Fishing as usize], 22);
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes::set_param stat setters (lines 1879-1884, 1897-1898,
    // 1905-1906)
    // -----------------------------------------------------------------------

    #[test]
    fn attrs_set_param_stats_and_special_skills_full() {
        use forgottenserver_common::enums::{SpecialSkill, Stat};
        let mut a = make_attrs();
        assert!(a.set_param(ConditionParam::StatMaxManaPoints, 200));
        assert_eq!(a.stats[Stat::MaxManaPoints as usize], 200);
        assert!(a.set_param(ConditionParam::StatMaxHitPointsPercent, 50));
        assert_eq!(a.stats_percent[Stat::MaxHitPoints as usize], 50);
        assert!(a.set_param(ConditionParam::StatMaxManaPointsPercent, 60));
        assert_eq!(a.stats_percent[Stat::MaxManaPoints as usize], 60);
        assert!(a.set_param(ConditionParam::StatMagicPointsPercent, 70));
        assert_eq!(a.stats_percent[Stat::MagicPoints as usize], 70);
        assert!(a.set_param(ConditionParam::SpecialSkillLifeLeechChance, 3));
        assert_eq!(a.special_skills[SpecialSkill::LifeLeechChance as usize], 3);
        assert!(a.set_param(ConditionParam::SpecialSkillLifeLeechAmount, 4));
        assert_eq!(a.special_skills[SpecialSkill::LifeLeechAmount as usize], 4);
        assert!(a.set_param(ConditionParam::SpecialSkillManaLeechChance, 5));
        assert_eq!(a.special_skills[SpecialSkill::ManaLeechChance as usize], 5);
        assert!(a.set_param(ConditionParam::SpecialSkillManaLeechAmount, 6));
        assert_eq!(a.special_skills[SpecialSkill::ManaLeechAmount as usize], 6);
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes::get_param remaining branches (lines 1938, 1940-1949,
    // 1951, 1954, 1957, 1960, 1967, 1970, 1973, 1976, 1981)
    // -----------------------------------------------------------------------

    #[test]
    fn attrs_get_param_all_remaining_branches() {
        use forgottenserver_common::enums::Skill;
        let mut a = make_attrs();
        a.set_param(ConditionParam::SkillMeleePercent, 99);
        assert_eq!(a.get_param(ConditionParam::SkillClubPercent), 99);
        assert_eq!(a.get_param(ConditionParam::SkillMeleePercent), 99);
        a.set_param(ConditionParam::SkillSword, 30);
        a.set_param(ConditionParam::SkillSwordPercent, 31);
        assert_eq!(a.get_param(ConditionParam::SkillSword), 30);
        assert_eq!(a.get_param(ConditionParam::SkillSwordPercent), 31);
        a.set_param(ConditionParam::SkillAxe, 40);
        a.set_param(ConditionParam::SkillAxePercent, 41);
        assert_eq!(a.get_param(ConditionParam::SkillAxe), 40);
        assert_eq!(a.get_param(ConditionParam::SkillAxePercent), 41);
        a.set_param(ConditionParam::SkillDistance, 50);
        a.set_param(ConditionParam::SkillDistancePercent, 51);
        assert_eq!(a.get_param(ConditionParam::SkillDistance), 50);
        assert_eq!(a.get_param(ConditionParam::SkillDistancePercent), 51);
        a.set_param(ConditionParam::SkillShield, 60);
        a.set_param(ConditionParam::SkillShieldPercent, 61);
        assert_eq!(a.get_param(ConditionParam::SkillShield), 60);
        assert_eq!(a.get_param(ConditionParam::SkillShieldPercent), 61);
        a.set_param(ConditionParam::SkillFishing, 70);
        a.set_param(ConditionParam::SkillFishingPercent, 71);
        assert_eq!(a.get_param(ConditionParam::SkillFishing), 70);
        assert_eq!(a.get_param(ConditionParam::SkillFishingPercent), 71);
        let _ = Skill::Fist;
    }

    #[test]
    fn attrs_get_param_stats_remaining_branches() {
        use forgottenserver_common::enums::Stat;
        let mut a = make_attrs();
        a.set_param(ConditionParam::StatMaxManaPoints, 200);
        a.set_param(ConditionParam::StatMagicPoints, 25);
        a.set_param(ConditionParam::StatMaxHitPointsPercent, 10);
        a.set_param(ConditionParam::StatMaxManaPointsPercent, 20);
        a.set_param(ConditionParam::StatMagicPointsPercent, 30);
        assert_eq!(a.get_param(ConditionParam::StatMaxManaPoints), 200);
        assert_eq!(a.get_param(ConditionParam::StatMagicPoints), 25);
        assert_eq!(a.get_param(ConditionParam::StatMaxHitPointsPercent), 10);
        assert_eq!(a.get_param(ConditionParam::StatMaxManaPointsPercent), 20);
        assert_eq!(a.get_param(ConditionParam::StatMagicPointsPercent), 30);
        let _ = Stat::MaxHitPoints;
    }

    #[test]
    fn attrs_get_param_special_skills_remaining() {
        use forgottenserver_common::enums::SpecialSkill;
        let mut a = make_attrs();
        a.set_param(ConditionParam::SpecialSkillCriticalHitAmount, 15);
        a.set_param(ConditionParam::SpecialSkillLifeLeechChance, 3);
        a.set_param(ConditionParam::SpecialSkillLifeLeechAmount, 4);
        a.set_param(ConditionParam::SpecialSkillManaLeechChance, 5);
        a.set_param(ConditionParam::SpecialSkillManaLeechAmount, 6);
        assert_eq!(
            a.get_param(ConditionParam::SpecialSkillCriticalHitAmount),
            15
        );
        assert_eq!(a.get_param(ConditionParam::SpecialSkillLifeLeechChance), 3);
        assert_eq!(a.get_param(ConditionParam::SpecialSkillLifeLeechAmount), 4);
        assert_eq!(a.get_param(ConditionParam::SpecialSkillManaLeechChance), 5);
        assert_eq!(a.get_param(ConditionParam::SpecialSkillManaLeechAmount), 6);
        let _ = SpecialSkill::CriticalHitChance;
    }

    #[test]
    fn attrs_get_param_base_fallthrough() {
        let a = make_attrs();
        assert_eq!(a.get_param(ConditionParam::Ticks), 5000);
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes::add_condition no-update path (line 1990)
    // -----------------------------------------------------------------------

    #[test]
    fn attrs_add_condition_no_update_when_shorter() {
        let mut existing = make_attrs();
        existing.base.ticks = 5000;
        existing.skills[0] = 99;

        let mut other = make_attrs();
        other.base.ticks = 1000;
        other.skills[0] = 1;

        existing.add_condition(&other);
        assert_eq!(
            existing.skills[0], 99,
            "skills unchanged when other is shorter"
        );
        assert_eq!(existing.base.ticks, 5000, "ticks unchanged");
    }

    // -----------------------------------------------------------------------
    // ConditionAttributes::deserialize truncated-data error paths (lines 2037,
    // 2047, 2056, 2065, 2074, 2079)
    // -----------------------------------------------------------------------

    fn make_attrs_base_bytes() -> Vec<u8> {
        let a = make_attrs();
        let mut buf = Vec::new();
        a.base.serialize(&mut buf);
        buf.push(ConditionAttr::End as u8);
        buf
    }

    #[test]
    fn attrs_deserialize_pos_ge_data_len_breaks_loop() {
        let base_bytes = make_attrs_base_bytes();
        let (decoded, _) =
            ConditionAttributes::deserialize(&base_bytes).expect("should succeed with just base");
        assert_eq!(decoded.skills, [0; ATTR_SKILL_COUNT]);
    }

    #[test]
    fn attrs_deserialize_truncated_skills_returns_none() {
        let mut buf = make_attrs_base_bytes();
        buf.push(ConditionAttr::Skills as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionAttributes::deserialize(&buf).is_none());
    }

    #[test]
    fn attrs_deserialize_skills_overflow_returns_none() {
        let mut buf = make_attrs_base_bytes();
        for _ in 0..=ATTR_SKILL_COUNT {
            buf.push(ConditionAttr::Skills as u8);
            buf.extend_from_slice(&42i32.to_le_bytes());
        }
        assert!(ConditionAttributes::deserialize(&buf).is_none());
    }

    #[test]
    fn attrs_deserialize_truncated_special_skills_returns_none() {
        let mut buf = make_attrs_base_bytes();
        buf.push(ConditionAttr::SpecialSkills as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionAttributes::deserialize(&buf).is_none());
    }

    #[test]
    fn attrs_deserialize_special_skills_overflow_returns_none() {
        let mut buf = make_attrs_base_bytes();
        for _ in 0..=ATTR_SPECIALSKILL_COUNT {
            buf.push(ConditionAttr::SpecialSkills as u8);
            buf.extend_from_slice(&7i32.to_le_bytes());
        }
        assert!(ConditionAttributes::deserialize(&buf).is_none());
    }

    #[test]
    fn attrs_deserialize_truncated_stats_returns_none() {
        let mut buf = make_attrs_base_bytes();
        buf.push(ConditionAttr::Stats as u8);
        buf.push(0x01);
        buf.push(0x00);
        assert!(ConditionAttributes::deserialize(&buf).is_none());
    }

    #[test]
    fn attrs_deserialize_stats_overflow_returns_none() {
        let mut buf = make_attrs_base_bytes();
        for _ in 0..=ATTR_STAT_COUNT {
            buf.push(ConditionAttr::Stats as u8);
            buf.extend_from_slice(&3i32.to_le_bytes());
        }
        assert!(ConditionAttributes::deserialize(&buf).is_none());
    }

    #[test]
    fn attrs_deserialize_truncated_disable_defense_returns_none() {
        let mut buf = make_attrs_base_bytes();
        buf.push(ConditionAttr::DisableDefense as u8);
        assert!(ConditionAttributes::deserialize(&buf).is_none());
    }

    #[test]
    fn attrs_deserialize_unknown_attr_breaks_loop() {
        let mut buf = make_attrs_base_bytes();
        buf.push(0xAB);
        let (decoded, _) =
            ConditionAttributes::deserialize(&buf).expect("unknown attr breaks loop gracefully");
        assert_eq!(decoded.skills, [0; ATTR_SKILL_COUNT]);
    }

    // -----------------------------------------------------------------------
    // create_condition branches (lines 2129-2130, 2134-2135, 2146-2147,
    // 2151-2152, 2161-2162)
    // -----------------------------------------------------------------------

    #[test]
    fn create_condition_invisible() {
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::INVISIBLE,
            2000,
            0,
            false,
            0,
            false,
        );
        assert!(matches!(c, Some(Condition::Invisible(_))));
    }

    #[test]
    fn create_condition_outfit() {
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::OUTFIT,
            2000,
            0,
            false,
            0,
            false,
        );
        assert!(matches!(c, Some(Condition::Outfit(_))));
    }

    #[test]
    fn create_condition_regeneration() {
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::REGENERATION,
            2000,
            0,
            false,
            0,
            false,
        );
        assert!(matches!(c, Some(Condition::Regeneration(_))));
    }

    #[test]
    fn create_condition_soul() {
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::SOUL,
            2000,
            0,
            false,
            0,
            false,
        );
        assert!(matches!(c, Some(Condition::Soul(_))));
    }

    #[test]
    fn create_condition_drunk() {
        let c = create_condition(
            ConditionId::Combat,
            ConditionTypeFlags::DRUNK,
            2000,
            0,
            false,
            0,
            false,
        );
        assert!(matches!(c, Some(Condition::Drunk(_))));
    }

    // -----------------------------------------------------------------------
    // Condition::base() and Condition::base_mut() Attributes arm (lines 2222,
    // 2238)
    // -----------------------------------------------------------------------

    #[test]
    fn condition_enum_base_and_base_mut_attributes_arm() {
        let mut c = Condition::Attributes(make_attrs());
        assert_eq!(c.base().condition_type, ConditionTypeFlags::ATTRIBUTES);
        c.base_mut().ticks = 9999;
        assert_eq!(c.base().ticks, 9999);
    }
}
