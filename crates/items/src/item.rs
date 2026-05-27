// Migrated from forgottenserver/src/item.h + item.cpp
//
// This module provides `Item` — a concrete, runtime item instance — together
// with the `AttributeValue` enum used by the generic attribute map.

use std::collections::HashMap;
use std::sync::Arc;

use forgottenserver_common::constants::WeaponType;
use forgottenserver_common::enums::{LightInfo, Reflect};
use forgottenserver_common::item_property::ItemProperty;

use crate::items_registry::{combat_type_to_index, ItemTypeData};

// ---------------------------------------------------------------------------
// ItemDecayState (mirrors ItemDecayState_t)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ItemDecayState {
    #[default]
    False = 0,
    True = 1,
    Pending = 2,
}

// ---------------------------------------------------------------------------
// ItemAttribute — discriminant enum for the attribute map key
//
// These mirror the `itemAttrTypes` bit-flags from the C++ codebase, but are
// used here as enum variants (HashMap keys) rather than bit masks.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemAttribute {
    ActionId,
    UniqueId,
    Description,
    Text,
    Date,
    Writer,
    Name,
    Article,
    PluralName,
    Weight,
    Attack,
    Defense,
    ExtraDefense,
    Armor,
    HitChance,
    ShootRange,
    Owner,
    Duration,
    DurationMax,
    DecayState,
    CorpseOwner,
    Charges,
    FluidType,
    DoorId,
    DecayTo,
    WrapId,
    StoreItem,
    AttackSpeed,
    OpenContainer,
}

// ---------------------------------------------------------------------------
// AttributeValue — the value side of the attribute map
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    Integer(i64),
    String(String),
    Float(f64),
    Boolean(bool),
}

// ---------------------------------------------------------------------------
// Item — a concrete item instance (mirrors C++ `Item`)
// ---------------------------------------------------------------------------

/// Typed Lua-scriptable custom attribute value. Mirrors the C++
/// `ItemAttributes::CustomAttribute::VariantAttribute`
/// (`variant<blank, string, int64_t, double, bool>`) minus the `blank`
/// variant — absence is encoded by HashMap-not-containing-key on the
/// Rust side rather than a distinct enum variant.
#[derive(Debug, Clone, PartialEq)]
pub enum CustomAttribute {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct Item {
    /// Shared reference to the compile-time type blueprint.
    pub item_type: Arc<ItemTypeData>,
    /// Stack count (or sub-type for fluids/charges).
    pub count: u8,
    /// Generic attribute storage.
    attributes: HashMap<ItemAttribute, AttributeValue>,
    /// Whether this item instance was placed by the map loader (mirrors
    /// C++ `bool loadedFromMap`). Map-placed items are exempt from the
    /// world's periodic ground-cleanup sweep — see `is_cleanable`.
    loaded_from_map: bool,
    /// Per-instance reflect overrides keyed by `CombatTypeFlags::*` bit
    /// values (mirrors C++ `ItemAttributes::reflect[combatType]`).
    /// Sparse: only combat types with a non-default override appear.
    reflect_overrides: HashMap<u16, Reflect>,
    /// Per-instance boost-percent overrides keyed by `CombatTypeFlags::*`
    /// bit values (mirrors C++ `ItemAttributes::boostPercent[combatType]`).
    boost_overrides: HashMap<u16, i16>,
    /// Lua-scriptable custom-attribute map (mirrors C++
    /// `ItemAttributes::CustomAttributeMap`). Keys are normalised to
    /// lowercase before storage — case-insensitive lookups match C++'s
    /// `boost::algorithm::to_lower_copy` step.
    custom_attributes: HashMap<String, CustomAttribute>,
}

impl Item {
    /// Construct a new item of the given type with the given count.
    pub fn new(item_type: Arc<ItemTypeData>, count: u8) -> Self {
        Item {
            item_type,
            count,
            attributes: HashMap::new(),
            loaded_from_map: false,
            reflect_overrides: HashMap::new(),
            boost_overrides: HashMap::new(),
            custom_attributes: HashMap::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Type-delegating accessors
    // -----------------------------------------------------------------------

    /// The server ID of this item.
    pub fn get_id(&self) -> u16 {
        self.item_type.id
    }

    /// The client (sprite) ID of this item.
    pub fn get_client_id(&self) -> u16 {
        self.item_type.client_id
    }

    /// The display name.  Returns the name override from the attribute map if
    /// set, otherwise delegates to the type blueprint.
    pub fn get_name(&self) -> &str {
        if let Some(AttributeValue::String(s)) = self.attributes.get(&ItemAttribute::Name) {
            return s.as_str();
        }
        self.item_type.name.as_str()
    }

    /// Computed weight: `base_weight * count` for stackable items.
    pub fn get_weight(&self) -> u32 {
        let base = self.get_base_weight();
        if self.is_stackable() {
            return base * (self.count.max(1) as u32);
        }
        base
    }

    /// Base weight from the type (or attribute override).
    pub fn get_base_weight(&self) -> u32 {
        if let Some(AttributeValue::Integer(w)) = self.attributes.get(&ItemAttribute::Weight) {
            return *w as u32;
        }
        self.item_type.weight
    }

    /// Stack count accessor.
    pub fn get_count(&self) -> u8 {
        self.count
    }

    // -----------------------------------------------------------------------
    // Boolean property delegation
    // -----------------------------------------------------------------------

    pub fn is_stackable(&self) -> bool {
        self.item_type.stackable
    }

    pub fn is_pickupable(&self) -> bool {
        self.item_type.is_pickupable()
    }

    pub fn is_moveable(&self) -> bool {
        self.item_type.moveable
    }

    pub fn is_useable(&self) -> bool {
        self.item_type.useable
    }

    pub fn is_rotatable(&self) -> bool {
        self.item_type.rotatable
    }

    pub fn is_hangable(&self) -> bool {
        self.item_type.is_hangable
    }

    pub fn block_solid(&self) -> bool {
        self.item_type.block_solid
    }

    pub fn block_path_find(&self) -> bool {
        self.item_type.block_path_find
    }

    pub fn block_projectile(&self) -> bool {
        self.item_type.block_projectile
    }

    pub fn has_height(&self) -> bool {
        self.item_type.has_height
    }

    pub fn look_through(&self) -> bool {
        self.item_type.look_through
    }

    pub fn always_on_top(&self) -> bool {
        self.item_type.always_on_top
    }

    pub fn allow_dist_read(&self) -> bool {
        self.item_type.allow_dist_read
    }

    pub fn is_readable(&self) -> bool {
        self.item_type.can_read_text
    }

    pub fn show_count(&self) -> bool {
        self.item_type.show_count
    }

    pub fn show_duration(&self) -> bool {
        self.item_type.show_duration
    }

    pub fn show_charges(&self) -> bool {
        self.item_type.show_charges
    }

    pub fn show_attributes(&self) -> bool {
        self.item_type.show_attributes
    }

    // -----------------------------------------------------------------------
    // Decay helpers
    // -----------------------------------------------------------------------

    /// Is this item currently decaying?
    pub fn is_decaying(&self) -> bool {
        matches!(
            self.get_attribute(ItemAttribute::DecayState),
            Some(AttributeValue::Integer(1)) | Some(AttributeValue::Integer(2))
        )
    }

    pub fn get_decay_state(&self) -> ItemDecayState {
        match self.get_attribute(ItemAttribute::DecayState) {
            Some(AttributeValue::Integer(1)) => ItemDecayState::True,
            Some(AttributeValue::Integer(2)) => ItemDecayState::Pending,
            _ => ItemDecayState::False,
        }
    }

    pub fn set_decaying(&mut self, state: ItemDecayState) {
        self.set_attribute(
            ItemAttribute::DecayState,
            AttributeValue::Integer(state as i64),
        );
    }

    pub fn get_duration(&self) -> u32 {
        match self.get_attribute(ItemAttribute::Duration) {
            Some(AttributeValue::Integer(v)) => *v as u32,
            _ => 0,
        }
    }

    pub fn set_duration(&mut self, secs: i32) {
        self.set_attribute(
            ItemAttribute::Duration,
            AttributeValue::Integer(secs.max(0) as i64),
        );
    }

    // -----------------------------------------------------------------------
    // Generic attribute map
    // -----------------------------------------------------------------------

    /// Retrieve an attribute value, if present.
    pub fn get_attribute(&self, attr: ItemAttribute) -> Option<&AttributeValue> {
        self.attributes.get(&attr)
    }

    /// Insert or replace an attribute value.
    pub fn set_attribute(&mut self, attr: ItemAttribute, value: AttributeValue) {
        self.attributes.insert(attr, value);
    }

    /// Remove an attribute.  Returns whether the attribute existed.
    pub fn remove_attribute(&mut self, attr: ItemAttribute) -> bool {
        self.attributes.remove(&attr).is_some()
    }

    // -----------------------------------------------------------------------
    // Description string (mirrors Item::getNameDescription)
    // -----------------------------------------------------------------------

    /// Format a human-readable description for this item.
    ///
    /// `look_distance` mirrors the C++ `lookDistance` parameter but is unused
    /// in the base description.
    pub fn get_description(&self, _look_distance: i32) -> String {
        let it = &self.item_type;
        let name = self.get_name();

        if name.is_empty() {
            return format!("an item of type {}", it.id);
        }

        if it.stackable && self.count > 1 {
            if it.show_count {
                format!("{} {}s", self.count, name)
            } else {
                name.to_string()
            }
        } else {
            let article = &it.article;
            if article.is_empty() {
                name.to_string()
            } else {
                format!("{} {}", article, name)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Convenience integer-attribute accessors (mirrors C++ API)
    // -----------------------------------------------------------------------

    pub fn get_action_id(&self) -> u16 {
        match self.get_attribute(ItemAttribute::ActionId) {
            Some(AttributeValue::Integer(v)) => *v as u16,
            _ => 0,
        }
    }

    pub fn set_action_id(&mut self, n: u16) {
        let n = if n < 100 { 100 } else { n };
        self.set_attribute(ItemAttribute::ActionId, AttributeValue::Integer(n as i64));
    }

    pub fn get_unique_id(&self) -> u16 {
        match self.get_attribute(ItemAttribute::UniqueId) {
            Some(AttributeValue::Integer(v)) => *v as u16,
            _ => 0,
        }
    }

    pub fn get_charges(&self) -> u16 {
        match self.get_attribute(ItemAttribute::Charges) {
            Some(AttributeValue::Integer(v)) => *v as u16,
            _ => 0,
        }
    }

    pub fn set_charges(&mut self, n: u16) {
        self.set_attribute(ItemAttribute::Charges, AttributeValue::Integer(n as i64));
    }

    pub fn get_fluid_type(&self) -> u16 {
        match self.get_attribute(ItemAttribute::FluidType) {
            Some(AttributeValue::Integer(v)) => *v as u16,
            _ => 0,
        }
    }

    pub fn set_fluid_type(&mut self, n: u16) {
        self.set_attribute(ItemAttribute::FluidType, AttributeValue::Integer(n as i64));
    }

    pub fn get_text(&self) -> &str {
        match self.get_attribute(ItemAttribute::Text) {
            Some(AttributeValue::String(s)) => s.as_str(),
            _ => "",
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.set_attribute(ItemAttribute::Text, AttributeValue::String(text));
    }

    pub fn get_writer(&self) -> &str {
        match self.get_attribute(ItemAttribute::Writer) {
            Some(AttributeValue::String(s)) => s.as_str(),
            _ => "",
        }
    }

    pub fn set_writer(&mut self, writer: String) {
        self.set_attribute(ItemAttribute::Writer, AttributeValue::String(writer));
    }

    pub fn get_date(&self) -> u32 {
        match self.get_attribute(ItemAttribute::Date) {
            Some(AttributeValue::Integer(v)) => *v as u32,
            _ => 0,
        }
    }

    pub fn set_date(&mut self, n: i32) {
        // Mirrors C++ `Item::setDate(int32_t)`. Negative epoch values are
        // valid (pre-1970 dates), so the signed signature must survive
        // through to the stored attribute integer.
        self.set_attribute(ItemAttribute::Date, AttributeValue::Integer(n as i64));
    }

    pub fn get_special_description(&self) -> &str {
        match self.get_attribute(ItemAttribute::Description) {
            Some(AttributeValue::String(s)) => s.as_str(),
            _ => "",
        }
    }

    pub fn set_special_description(&mut self, desc: String) {
        self.set_attribute(ItemAttribute::Description, AttributeValue::String(desc));
    }

    pub fn get_attack(&self) -> i32 {
        match self.get_attribute(ItemAttribute::Attack) {
            Some(AttributeValue::Integer(v)) => *v as i32,
            _ => self.item_type.attack,
        }
    }

    pub fn get_defense(&self) -> i32 {
        match self.get_attribute(ItemAttribute::Defense) {
            Some(AttributeValue::Integer(v)) => *v as i32,
            _ => self.item_type.defense,
        }
    }

    pub fn get_armor(&self) -> i32 {
        match self.get_attribute(ItemAttribute::Armor) {
            Some(AttributeValue::Integer(v)) => *v as i32,
            _ => self.item_type.armor,
        }
    }

    pub fn get_attack_speed(&self) -> u32 {
        match self.get_attribute(ItemAttribute::AttackSpeed) {
            Some(AttributeValue::Integer(v)) => *v as u32,
            _ => self.item_type.attack_speed,
        }
    }

    /// Mirrors C++ `Item::isLoadedFromMap()`.
    pub fn is_loaded_from_map(&self) -> bool {
        self.loaded_from_map
    }

    /// Mirrors C++ `Item::setLoadedFromMap(bool)`. The map loader flips
    /// this true for every item it places; the world's periodic cleanup
    /// sweep reads it via `is_cleanable` to skip those tiles.
    pub fn set_loaded_from_map(&mut self, value: bool) {
        self.loaded_from_map = value;
    }

    /// Mirrors C++ `Item::canRemove()`. Default is `true`; subclasses can
    /// override (when those land — house tiles, etc.).
    pub fn can_remove(&self) -> bool {
        true
    }

    // -----------------------------------------------------------------------
    // C++ `Item.cpp` parity wrappers (one per remaining ledger gap)
    // -----------------------------------------------------------------------

    /// Mirrors C++ `bool Item::equals(const Item* otherItem) const`. Two
    /// items are equal when they share the same item-type id *and* every
    /// per-instance attribute matches. Reference equality (Arc::ptr_eq on
    /// `item_type`) is NOT required — two distinct Arcs holding equivalent
    /// blueprints still equate.
    pub fn equals(&self, other: &Item) -> bool {
        if self.get_id() != other.get_id() {
            return false;
        }
        // Empty maps compare equal; differing sets are not equal even if
        // one is empty (C++ checks `attributeBits` equality first).
        if self.attributes.len() != other.attributes.len() {
            return false;
        }
        for (k, v) in &self.attributes {
            match other.attributes.get(k) {
                Some(ov) if ov == v => {}
                _ => return false,
            }
        }
        true
    }

    /// Mirrors C++ `uint16_t Item::getSubType() const`.
    ///
    /// Fluid containers / splashes → fluid type;
    /// stackables → count;
    /// items with default charges → current charges;
    /// otherwise count (matches C++ default fallthrough).
    pub fn get_sub_type(&self) -> u16 {
        if self.item_type.is_fluid_container() || self.item_type.is_splash() {
            return self.get_fluid_type();
        }
        if self.item_type.stackable {
            return self.count as u16;
        }
        if self.item_type.charges != 0 {
            return self.get_charges();
        }
        self.count as u16
    }

    /// Mirrors C++ `bool Item::hasProperty(ITEMPROPERTY) const`.
    pub fn has_property(&self, prop: ItemProperty) -> bool {
        use forgottenserver_common::item_property::ItemProperty as P;
        let it = &self.item_type;
        let has_uid = self.get_attribute(ItemAttribute::UniqueId).is_some();
        match prop {
            P::BlockSolid => it.block_solid,
            P::Moveable => it.moveable && !has_uid,
            P::HasHeight => it.has_height,
            P::BlockProjectile => it.block_projectile,
            P::BlockPath => it.block_path_find,
            P::IsVertical => it.is_vertical,
            P::IsHorizontal => it.is_horizontal,
            P::ImmovableBlockSolid => it.block_solid && (!it.moveable || has_uid),
            P::ImmovableBlockPath => it.block_path_find && (!it.moveable || has_uid),
            P::ImmovableNoFieldBlockPath => {
                !it.is_magic_field() && it.block_path_find && (!it.moveable || has_uid)
            }
            P::NoFieldBlockPath => !it.is_magic_field() && it.block_path_find,
            P::SupportHangable => it.is_horizontal || it.is_vertical,
        }
    }

    /// Mirrors C++ `uint32_t Item::getWorth() const { return items[id].worth * count; }`.
    pub fn get_worth(&self) -> u64 {
        self.item_type
            .worth
            .saturating_mul(self.count.max(1) as u64)
    }

    /// Mirrors C++ `LightInfo Item::getLightInfo() const`.
    pub fn get_light_info(&self) -> LightInfo {
        LightInfo {
            level: self.item_type.light_level,
            color: self.item_type.light_color,
        }
    }

    /// Mirrors C++ `bool Item::isPushable() const`. Aliased to `isMoveable`
    /// for parity with `Thing::isPushable`.
    pub fn is_pushable(&self) -> bool {
        self.is_moveable()
    }

    /// Mirrors C++ `int32_t Item::getThrowRange() const`.
    /// Pickupable items have a 15-tile throw range; world props 2 tiles.
    pub fn get_throw_range(&self) -> i32 {
        if self.is_pickupable() {
            15
        } else {
            2
        }
    }

    /// Mirrors C++ `Ammo_t Item::getAmmoType() const { return items[id].ammoType; }`.
    pub fn get_ammo_type(&self) -> forgottenserver_common::constants::Ammo {
        self.item_type.ammo_type
    }

    /// Mirrors C++ `int32_t Item::getDecayTimeMin() const`.
    ///
    /// Override attribute (`ItemAttribute::Duration`) first — C++ aliases
    /// `ITEM_ATTRIBUTE_DURATION_MIN = ITEM_ATTRIBUTE_DURATION` — else the
    /// item-type's static `decay_time_min`.
    pub fn get_decay_time_min(&self) -> i32 {
        match self.get_attribute(ItemAttribute::Duration) {
            Some(AttributeValue::Integer(v)) => *v as i32,
            _ => self.item_type.decay_time_min as i32,
        }
    }

    /// Mirrors C++ `int32_t Item::getDecayTimeMax() const`.
    pub fn get_decay_time_max(&self) -> i32 {
        match self.get_attribute(ItemAttribute::DurationMax) {
            Some(AttributeValue::Integer(v)) => *v as i32,
            _ => self.item_type.decay_time_max as i32,
        }
    }

    /// Mirrors C++ `uint32_t Item::getDefaultDurationMin() const`.
    /// `decayTimeMin` is in seconds on the C++ side; the runtime
    /// duration helpers (`setDuration`) take milliseconds, hence the
    /// ×1000 conversion.
    pub fn get_default_duration_min(&self) -> u32 {
        self.item_type.decay_time_min.saturating_mul(1000)
    }

    /// Mirrors C++ `uint32_t Item::getDefaultDurationMax() const`.
    pub fn get_default_duration_max(&self) -> u32 {
        self.item_type.decay_time_max.saturating_mul(1000)
    }

    /// Mirrors C++ `void Item::setDefaultDuration()`. Samples uniformly
    /// from `[defaultDurationMin, defaultDurationMax]` and assigns the
    /// resulting Duration attribute (when non-zero).
    pub fn set_default_duration(&mut self) {
        let min_ms = self.get_default_duration_min();
        let max_ms = self.get_default_duration_max();
        let duration = if max_ms > 0 && max_ms >= min_ms {
            forgottenserver_common::tools::normal_random(min_ms as i32, max_ms as i32) as u32
        } else {
            min_ms
        };
        if duration != 0 {
            self.set_duration(duration as i32);
        }
    }

    /// Mirrors C++ `bool Item::hasMarketAttributes() const`. An item is
    /// safe to list on the market when:
    ///   * no per-instance reflect or boost overrides are set
    ///     (any non-default value blocks listing — matches C++ loop), AND
    ///   * every remaining attribute is either an unchanged Charges or a
    ///     Duration strictly above the default min.
    pub fn has_market_attributes(&self) -> bool {
        if self.has_any_reflect_or_boost_override() {
            return false;
        }
        if self.attributes.is_empty() {
            return true;
        }
        for (k, v) in &self.attributes {
            match k {
                ItemAttribute::Charges => {
                    if let AttributeValue::Integer(n) = v {
                        if *n as u32 != self.item_type.charges {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                ItemAttribute::Duration => {
                    if let AttributeValue::Integer(n) = v {
                        if (*n as u32) <= self.get_default_duration_min() {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }

    /// Static helper mirroring the parameter-less branch of
    /// `Item::getNameDescription(const ItemType&, const Item* = nullptr,
    /// int32_t subType = -1, bool addArticle = true)`. The full overload
    /// (with `Item*`/`subType`/`addArticle`) lives on the Rust `Item`
    /// instance via `name_description` below.
    ///
    /// Returns the article + name + optional plural-or-count prefix.
    /// Matches C++'s `(it.stackable && subType > 1)` branch and the
    /// non-stackable article fallback.
    pub fn name_description_for_type(
        item_type: &crate::items_registry::ItemTypeData,
        sub_type: i32,
        add_article: bool,
    ) -> String {
        let name = &item_type.name;
        if name.is_empty() {
            return "an item of type ".to_string() + &item_type.id.to_string();
        }
        if item_type.stackable && sub_type > 1 {
            if item_type.show_count {
                return format!(
                    "{} {}",
                    sub_type,
                    if !item_type.plural_name.is_empty() {
                        &item_type.plural_name
                    } else {
                        name
                    }
                );
            }
            return name.clone();
        }
        if add_article && !item_type.article.is_empty() {
            return format!("{} {}", item_type.article, name);
        }
        name.clone()
    }

    /// Instance-bound version: pulls `subType` from this item, defers
    /// the rest to `name_description_for_type`.
    pub fn name_description(&self, add_article: bool) -> String {
        Self::name_description_for_type(&self.item_type, self.get_sub_type() as i32, add_article)
    }

    /// Mirrors C++ `Reflect Item::getReflect(CombatType_t, bool total) const`.
    ///
    /// The total form adds the per-instance override to the item-type's
    /// `abilities->reflect[idx]` value. When `total = false`, only the
    /// per-instance override is returned (matches the C++ branch used by
    /// `hasMarketAttributes`).
    pub fn get_reflect(&self, combat_type: u16, total: bool) -> Reflect {
        let mut acc = Reflect::default();
        if let Some(r) = self.reflect_overrides.get(&combat_type) {
            acc += *r;
        }
        if total {
            if let Some(abilities) = self.item_type.abilities.as_ref() {
                let idx = combat_type_to_index(combat_type);
                if idx < abilities.reflect.len() {
                    acc += abilities.reflect[idx];
                }
            }
        }
        acc
    }

    /// Mirrors C++ `void Item::setReflect(CombatType_t, const Reflect&)`.
    /// A zero Reflect (`percent == 0 && chance == 0`) removes the
    /// override entirely so `has_market_attributes` can detect a clean
    /// item.
    pub fn set_reflect(&mut self, combat_type: u16, reflect: Reflect) {
        if reflect.percent == 0 && reflect.chance == 0 {
            self.reflect_overrides.remove(&combat_type);
        } else {
            self.reflect_overrides.insert(combat_type, reflect);
        }
    }

    /// Mirrors C++ `uint16_t Item::getBoostPercent(CombatType_t, bool total) const`.
    pub fn get_boost_percent(&self, combat_type: u16, total: bool) -> u16 {
        let mut acc: i32 = self.boost_overrides.get(&combat_type).copied().unwrap_or(0) as i32;
        if total {
            if let Some(abilities) = self.item_type.abilities.as_ref() {
                let idx = combat_type_to_index(combat_type);
                if idx < abilities.boost_percent.len() {
                    acc += abilities.boost_percent[idx] as i32;
                }
            }
        }
        // C++ returns uint16_t; clamp the sum into the unsigned range.
        acc.clamp(0, u16::MAX as i32) as u16
    }

    /// Mirrors C++ `void Item::setBoostPercent(CombatType_t, uint16_t)`.
    /// Setting `0` removes the override (parity with the Reflect setter).
    pub fn set_boost_percent(&mut self, combat_type: u16, value: i16) {
        if value == 0 {
            self.boost_overrides.remove(&combat_type);
        } else {
            self.boost_overrides.insert(combat_type, value);
        }
    }

    /// Iterates every `CombatTypeFlags::*` bit and reports whether any
    /// per-instance reflect or boost override is set. Used by
    /// `has_market_attributes` so non-default reflect/boost values block
    /// market listing — matching C++'s reflect/boost loop in
    /// `hasMarketAttributes`.
    fn has_any_reflect_or_boost_override(&self) -> bool {
        !self.reflect_overrides.is_empty() || !self.boost_overrides.is_empty()
    }

    /// Static formatter mirroring C++
    /// `Item::getWeightDescription(const ItemType& it, uint32_t weight, uint32_t count = 1)`.
    ///
    /// Formats a weight as a tibia client-style string: weights below
    /// 100 are emitted as `0.XX` / `0.0X` decimal fractions, larger
    /// weights have a `.` inserted two digits from the right. Plural
    /// ("They weigh") vs singular ("It weighs") is chosen by
    /// `stackable && count > 1 && show_count`.
    pub fn weight_description_for_type(
        item_type: &crate::items_registry::ItemTypeData,
        weight: u32,
        count: u32,
    ) -> String {
        let prefix = if item_type.stackable && count > 1 && item_type.show_count {
            "They weigh "
        } else {
            "It weighs "
        };
        let formatted = if weight < 10 {
            format!("0.0{weight}")
        } else if weight < 100 {
            format!("0.{weight}")
        } else {
            // Insert a `.` two digits from the right (1234 → "12.34").
            let s = weight.to_string();
            let split_at = s.len() - 2;
            let (whole, frac) = s.split_at(split_at);
            format!("{whole}.{frac}")
        };
        format!("{prefix}{formatted} oz.")
    }

    /// Instance overload mirroring C++
    /// `Item::getWeightDescription(uint32_t weight) const`: takes the
    /// weight as input (caller already computed it; useful for inspect
    /// dialogs that show a hypothetical weight) and feeds the instance's
    /// item-type and count into the static formatter.
    pub fn weight_description_with(&self, weight: u32) -> String {
        Self::weight_description_for_type(&self.item_type, weight, self.count as u32)
    }

    /// Instance overload mirroring C++
    /// `Item::getWeightDescription() const`: returns empty when
    /// `getWeight() == 0` (matches the C++ short-circuit so weightless
    /// items get no descriptor line).
    pub fn weight_description(&self) -> String {
        let weight = self.get_weight();
        if weight == 0 {
            return String::new();
        }
        self.weight_description_with(weight)
    }

    /// Static helper mirroring C++
    /// `Item::countByType(const Item* i, int32_t subType)`. Returns the
    /// item's count when `sub_type < 0` (the "match any sub-type" path)
    /// or when the stored sub-type matches, else 0.
    ///
    /// `sub_type` is signed because C++ uses `-1` as the "ignore" sentinel,
    /// matching the API shape used by `Container::countByType` walks.
    pub fn count_by_type(&self, sub_type: i32) -> u32 {
        if sub_type < 0 || sub_type as u16 == self.get_sub_type() {
            self.count as u32
        } else {
            0
        }
    }

    // -----------------------------------------------------------------------
    // Custom attributes (mirror C++ `ItemAttributes::CustomAttribute*` API)
    // -----------------------------------------------------------------------

    /// Read-only access to the underlying custom-attribute map. Mirrors
    /// C++ `CustomAttributeMap* getCustomAttributeMap()` — returns `None`
    /// when no attributes have been set so callers can short-circuit.
    pub fn get_custom_attribute_map(&self) -> Option<&HashMap<String, CustomAttribute>> {
        if self.custom_attributes.is_empty() {
            None
        } else {
            Some(&self.custom_attributes)
        }
    }

    /// Mirrors C++ `setCustomAttribute(std::string_view key, R value)`:
    /// lowercases the key, replaces any existing entry under that key.
    /// `R` on the Rust side is the typed `CustomAttribute` enum (no
    /// templating needed — the C++ template only existed because
    /// `boost::variant` couldn't be constructed implicitly).
    pub fn set_custom_attribute(&mut self, key: &str, value: CustomAttribute) {
        let normalised = key.to_ascii_lowercase();
        self.custom_attributes.insert(normalised, value);
    }

    /// Mirrors C++ `setCustomAttribute(int64_t key, R value)`:
    /// stringifies the integer key and forwards to the string-keyed setter.
    pub fn set_custom_attribute_int_key(&mut self, key: i64, value: CustomAttribute) {
        self.set_custom_attribute(&key.to_string(), value);
    }

    /// Mirrors C++ `const CustomAttribute* getCustomAttribute(std::string_view)`.
    /// Returns `None` for unknown keys.
    pub fn get_custom_attribute(&self, key: &str) -> Option<&CustomAttribute> {
        let normalised = key.to_ascii_lowercase();
        self.custom_attributes.get(&normalised)
    }

    /// Mirrors C++ `const CustomAttribute* getCustomAttribute(int64_t key)`.
    pub fn get_custom_attribute_int_key(&self, key: i64) -> Option<&CustomAttribute> {
        self.get_custom_attribute(&key.to_string())
    }

    /// Mirrors C++ `bool removeCustomAttribute(std::string_view)`:
    /// returns `true` when an entry was removed. After removing the last
    /// entry the underlying map is the same as the never-populated case
    /// (`get_custom_attribute_map()` returns `None`).
    pub fn remove_custom_attribute(&mut self, key: &str) -> bool {
        let normalised = key.to_ascii_lowercase();
        self.custom_attributes.remove(&normalised).is_some()
    }

    /// Mirrors C++ `bool removeCustomAttribute(int64_t key)`.
    pub fn remove_custom_attribute_int_key(&mut self, key: i64) -> bool {
        self.remove_custom_attribute(&key.to_string())
    }

    /// Whether this item is cleanable (eligible for the world's periodic
    /// ground sweep). Mirrors C++ `Item::isCleanable` — gates on
    /// `!loadedFromMap && canRemove()` in addition to the pickupable +
    /// no-UID + no-AID checks.
    pub fn is_cleanable(&self) -> bool {
        !self.loaded_from_map
            && self.can_remove()
            && self.is_pickupable()
            && self.get_attribute(ItemAttribute::UniqueId).is_none()
            && self.get_attribute(ItemAttribute::ActionId).is_none()
    }

    /// Can this item decay?
    pub fn can_decay(&self) -> bool {
        if self.item_type.decay_to < 0
            && self.item_type.decay_time_min == 0
            && self.item_type.decay_time_max == 0
        {
            return false;
        }
        if self.get_attribute(ItemAttribute::UniqueId).is_some() {
            return false;
        }
        true
    }

    // -----------------------------------------------------------------------
    // `isBlocking` alias (mirrors C++ `Item::isBlocking`)
    // -----------------------------------------------------------------------

    /// Whether this item blocks solid movement.  Delegates to the type flag.
    pub fn is_blocking(&self) -> bool {
        self.item_type.block_solid
    }

    // -----------------------------------------------------------------------
    // Weapon / slot position accessors (mirrors C++ getWeaponType / getSlotPosition)
    // -----------------------------------------------------------------------

    /// Returns the weapon type of this item's type blueprint.
    pub fn get_weapon_type(&self) -> WeaponType {
        self.item_type.weapon_type
    }

    /// Returns the slot-position bitmask of this item's type blueprint.
    pub fn get_slot_position(&self) -> u32 {
        self.item_type.slot_position
    }

    /// Returns true if this is a store item (cannot be moved).
    /// Checks the per-instance `StoreItem` attribute first (set by `set_store_item`),
    /// then falls back to the type-level `store_item` flag.
    pub fn is_store_item(&self) -> bool {
        if let Some(AttributeValue::Integer(v)) = self.get_attribute(ItemAttribute::StoreItem) {
            return *v != 0;
        }
        self.item_type.store_item
    }

    // -----------------------------------------------------------------------
    // Stack count management
    // -----------------------------------------------------------------------

    /// Maximum stack count (mirrors the C++ game constant of 100).
    pub const MAX_STACK_COUNT: u8 = 100;

    /// Set the stack count, clamped to MAX_STACK_COUNT.
    pub fn set_item_count(&mut self, n: u8) {
        self.count = n.min(Self::MAX_STACK_COUNT);
    }

    /// Get the stack count.
    pub fn get_item_count(&self) -> u8 {
        self.count
    }

    // -----------------------------------------------------------------------
    // Duration helpers
    // -----------------------------------------------------------------------

    /// Decrease duration by `time` (clamped to zero).
    pub fn decrease_duration(&mut self, time: i32) {
        let current = self.get_duration() as i32;
        let new_val = (current - time).max(0);
        self.set_duration(new_val);
    }

    // -----------------------------------------------------------------------
    // Unique ID (idempotent set — mirrors C++ Item::setUniqueId)
    // -----------------------------------------------------------------------

    /// Set the unique ID.  Does nothing if already set (idempotent, same as C++).
    pub fn set_unique_id(&mut self, n: u16) {
        if self.get_attribute(ItemAttribute::UniqueId).is_none() {
            self.set_attribute(ItemAttribute::UniqueId, AttributeValue::Integer(n as i64));
        }
    }

    // -----------------------------------------------------------------------
    // Owner / CorpseOwner typed accessors
    // -----------------------------------------------------------------------

    pub fn get_owner(&self) -> u32 {
        match self.get_attribute(ItemAttribute::Owner) {
            Some(AttributeValue::Integer(v)) => *v as u32,
            _ => 0,
        }
    }

    pub fn set_owner(&mut self, owner: u32) {
        self.set_attribute(ItemAttribute::Owner, AttributeValue::Integer(owner as i64));
    }

    pub fn get_corpse_owner(&self) -> u32 {
        match self.get_attribute(ItemAttribute::CorpseOwner) {
            Some(AttributeValue::Integer(v)) => *v as u32,
            _ => 0,
        }
    }

    pub fn set_corpse_owner(&mut self, owner: u32) {
        self.set_attribute(
            ItemAttribute::CorpseOwner,
            AttributeValue::Integer(owner as i64),
        );
    }

    // -----------------------------------------------------------------------
    // HitChance / ShootRange typed accessors
    // -----------------------------------------------------------------------

    pub fn get_hit_chance(&self) -> i8 {
        match self.get_attribute(ItemAttribute::HitChance) {
            Some(AttributeValue::Integer(v)) => *v as i8,
            _ => 0,
        }
    }

    pub fn set_hit_chance(&mut self, n: i8) {
        self.set_attribute(ItemAttribute::HitChance, AttributeValue::Integer(n as i64));
    }

    pub fn get_shoot_range(&self) -> u8 {
        // Mirrors C++ `Item::getShootRange()`: override attribute first,
        // else fall back to `items[id].shootRange`. The old fallback path
        // returned `attack_speed`, which is a different ItemType field.
        match self.get_attribute(ItemAttribute::ShootRange) {
            Some(AttributeValue::Integer(v)) => *v as u8,
            _ => self.item_type.shoot_range,
        }
    }

    pub fn set_shoot_range(&mut self, n: u8) {
        self.set_attribute(ItemAttribute::ShootRange, AttributeValue::Integer(n as i64));
    }

    // -----------------------------------------------------------------------
    // ExtraDefense typed accessor
    // -----------------------------------------------------------------------

    pub fn get_extra_defense(&self) -> i32 {
        match self.get_attribute(ItemAttribute::ExtraDefense) {
            Some(AttributeValue::Integer(v)) => *v as i32,
            _ => self.item_type.extra_defense,
        }
    }

    pub fn set_extra_defense(&mut self, n: i32) {
        self.set_attribute(
            ItemAttribute::ExtraDefense,
            AttributeValue::Integer(n as i64),
        );
    }

    // -----------------------------------------------------------------------
    // DecayTo typed accessor (attribute override takes priority over type)
    // -----------------------------------------------------------------------

    pub fn get_decay_to(&self) -> i32 {
        match self.get_attribute(ItemAttribute::DecayTo) {
            Some(AttributeValue::Integer(v)) => *v as i32,
            _ => self.item_type.decay_to,
        }
    }

    pub fn set_decay_to(&mut self, n: i32) {
        self.set_attribute(ItemAttribute::DecayTo, AttributeValue::Integer(n as i64));
    }

    // -----------------------------------------------------------------------
    // AttackSpeed with minimum clamp (mirrors C++ setIntAttr ATTACK_SPEED clamping)
    // -----------------------------------------------------------------------

    /// Minimum attack speed (mirrors C++ clamping in `setIntAttr` for ATTACK_SPEED).
    pub const MIN_ATTACK_SPEED: u32 = 100;

    pub fn set_attack_speed(&mut self, n: u32) {
        let clamped = n.max(Self::MIN_ATTACK_SPEED);
        self.set_attribute(
            ItemAttribute::AttackSpeed,
            AttributeValue::Integer(clamped as i64),
        );
    }

    // -----------------------------------------------------------------------
    // Reset convenience methods (mirrors C++ resetText / resetDate / resetWriter)
    // -----------------------------------------------------------------------

    pub fn reset_text(&mut self) {
        self.remove_attribute(ItemAttribute::Text);
    }

    pub fn reset_date(&mut self) {
        self.remove_attribute(ItemAttribute::Date);
    }

    pub fn reset_writer(&mut self) {
        self.remove_attribute(ItemAttribute::Writer);
    }

    // -----------------------------------------------------------------------
    // has_attribute convenience (mirrors C++ Item::hasAttribute)
    // -----------------------------------------------------------------------

    /// Whether the given attribute is present in the map.
    pub fn has_attribute(&self, attr: ItemAttribute) -> bool {
        self.attributes.contains_key(&attr)
    }

    // -----------------------------------------------------------------------
    // `transform` — change the item type (mirrors C++ `Item::setID`)
    //
    // In C++ `setID` updates id and resets duration/corpse owner.  Here we
    // replace the `item_type` Arc and clear the relevant attributes.
    // -----------------------------------------------------------------------

    /// Change the item's type to `new_type`.
    ///
    /// Mirrors `Item::setID`: clears `CorpseOwner` and, when the new type has
    /// a decay duration, resets `DecayState` to `False` and sets the duration
    /// from the type blueprint.
    pub fn transform(&mut self, new_type: Arc<ItemTypeData>) {
        self.item_type = new_type;
        // Clear corpse owner (C++ always does this)
        self.remove_attribute(ItemAttribute::CorpseOwner);
        // Reset decay attributes if new type has a decay duration
        let new_duration_min = self.item_type.decay_time_min;
        let new_duration_max = self.item_type.decay_time_max;
        if new_duration_min == 0 && new_duration_max == 0 && self.item_type.decay_to < 0 {
            self.remove_attribute(ItemAttribute::DecayState);
            self.remove_attribute(ItemAttribute::Duration);
        } else {
            self.set_decaying(ItemDecayState::False);
            // Pick a duration within [min, max] — use max when nonzero, else min.
            let duration = if new_duration_max > 0 {
                new_duration_max
            } else {
                new_duration_min
            };
            self.set_duration(duration as i32);
        }
    }

    // -----------------------------------------------------------------------
    // `deep_copy` — creates a fully independent clone (mirrors C++ `Item::clone`)
    //
    // The derived `Clone` on `Item` already deep-copies the HashMap and the
    // `item_type` Arc (shared pointer, not cloned content — same as C++).
    // This method is an explicit alias used by callers that want the C++ semantic.
    // -----------------------------------------------------------------------

    /// Create an independent copy of this item with the same type and attributes.
    /// The returned item shares the same `Arc<ItemTypeData>` but has its own
    /// attribute map (mutations on the copy do not affect the original).
    pub fn deep_copy(&self) -> Item {
        self.clone()
    }

    // -----------------------------------------------------------------------
    // Serialize / unserialize (binary round-trip, mirrors serializeAttr /
    // unserializeAttr from C++)
    //
    // Format: sequence of (attr_tag: u8, payload) records, terminated by 0x00.
    //
    //  ATTR_ACTION_ID (0x04)     → u16 le
    //  ATTR_UNIQUE_ID (0x05)     → u16 le
    //  ATTR_TEXT      (0x06)     → u16-length-prefixed UTF-8 string
    //  ATTR_DESC      (0x07)     → u16-length-prefixed UTF-8 string
    //  ATTR_DURATION  (0x10)     → i32 le (clamped ≥ 0)
    //  ATTR_DECAYING_STATE(0x11) → u8 (0 = False, 1 = True, 2 = Pending)
    //  ATTR_COUNT     (0x0F)     → u8 (sub-type / stack count)
    //  ATTR_CHARGES   (0x16)     → u16 le
    //  ATTR_WRITTENDATE(0x12)    → u32 le
    //  ATTR_WRITTENBY (0x13)     → u16-length-prefixed UTF-8 string
    //  ATTR_NAME      (0x18)     → u16-length-prefixed UTF-8 string
    //  ATTR_ARTICLE   (0x19)     → u16-length-prefixed UTF-8 string
    //  ATTR_PLURALNAME(0x1A)     → u16-length-prefixed UTF-8 string
    //  ATTR_WEIGHT    (0x1B)     → u32 le
    //  ATTR_ATTACK    (0x1C)     → i32 le
    //  ATTR_DEFENSE   (0x1D)     → i32 le
    //  ATTR_EXTRADEFENSE(0x1E)   → i32 le
    //  ATTR_ARMOR     (0x1F)     → i32 le
    //  ATTR_HITCHANCE (0x20)     → i8
    //  ATTR_SHOOTRANGE(0x21)     → u8
    //  ATTR_DECAYTO   (0x23)     → i32 le
    //  ATTR_WRAPID    (0x24)     → u16 le
    //  ATTR_STOREITEM (0x25)     → u8
    //  ATTR_OPENCONTAINER(0x27)  → u8
    //  ATTR_ATTACK_SPEED(0x26)   → u32 le
    //
    // -----------------------------------------------------------------------

    /// Tags that mirror C++ `AttrTypes_t`.
    const TAG_COUNT: u8 = 0x0F;
    const TAG_ACTION_ID: u8 = 0x04;
    const TAG_UNIQUE_ID: u8 = 0x05;
    const TAG_TEXT: u8 = 0x06;
    const TAG_DESC: u8 = 0x07;
    const TAG_DURATION: u8 = 0x10;
    const TAG_DECAYING_STATE: u8 = 0x11;
    const TAG_WRITTENDATE: u8 = 0x12;
    const TAG_WRITTENBY: u8 = 0x13;
    const TAG_CHARGES: u8 = 0x16;
    const TAG_NAME: u8 = 0x18;
    const TAG_ARTICLE: u8 = 0x19;
    const TAG_PLURALNAME: u8 = 0x1A;
    const TAG_WEIGHT: u8 = 0x1B;
    const TAG_ATTACK: u8 = 0x1C;
    const TAG_DEFENSE: u8 = 0x1D;
    const TAG_EXTRADEFENSE: u8 = 0x1E;
    const TAG_ARMOR: u8 = 0x1F;
    const TAG_HITCHANCE: u8 = 0x20;
    const TAG_SHOOTRANGE: u8 = 0x21;
    const TAG_DECAYTO: u8 = 0x23;
    const TAG_WRAPID: u8 = 0x24;
    const TAG_STOREITEM: u8 = 0x25;
    const TAG_ATTACK_SPEED: u8 = 0x26;
    const TAG_OPENCONTAINER: u8 = 0x27;

    /// Serialize all attributes into a byte buffer.
    pub fn serialize_attr(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        // Count / sub-type (for stackable / fluid / splash items)
        let it = &self.item_type;
        if it.stackable || it.is_splash() || it.is_fluid_container() {
            buf.push(Self::TAG_COUNT);
            buf.push(self.count);
        }

        // Charges
        let charges = self.get_charges();
        if charges != 0 {
            buf.push(Self::TAG_CHARGES);
            buf.extend_from_slice(&charges.to_le_bytes());
        }

        // Action ID (only serialize for moveable items, like C++)
        if it.moveable {
            let action_id = self.get_action_id();
            if action_id != 0 {
                buf.push(Self::TAG_ACTION_ID);
                buf.extend_from_slice(&action_id.to_le_bytes());
            }
        }

        // Text
        let text = self.get_text();
        if !text.is_empty() {
            buf.push(Self::TAG_TEXT);
            write_string(&mut buf, text);
        }

        // Written date
        let date = self.get_date();
        if date != 0 {
            buf.push(Self::TAG_WRITTENDATE);
            buf.extend_from_slice(&date.to_le_bytes());
        }

        // Writer
        let writer = self.get_writer();
        if !writer.is_empty() {
            buf.push(Self::TAG_WRITTENBY);
            write_string(&mut buf, writer);
        }

        // Special description
        let desc = self.get_special_description();
        if !desc.is_empty() {
            buf.push(Self::TAG_DESC);
            write_string(&mut buf, desc);
        }

        // Duration
        if self.has_attribute(ItemAttribute::Duration) {
            buf.push(Self::TAG_DURATION);
            let d = self.get_duration();
            buf.extend_from_slice(&d.to_le_bytes());
        }

        // Decay state (only True or Pending are persisted)
        let ds = self.get_decay_state();
        if ds == ItemDecayState::True || ds == ItemDecayState::Pending {
            buf.push(Self::TAG_DECAYING_STATE);
            buf.push(ds as u8);
        }

        // Name override
        if self.has_attribute(ItemAttribute::Name) {
            if let Some(AttributeValue::String(s)) = self.get_attribute(ItemAttribute::Name) {
                buf.push(Self::TAG_NAME);
                write_string(&mut buf, s);
            }
        }

        // Article override
        if self.has_attribute(ItemAttribute::Article) {
            if let Some(AttributeValue::String(s)) = self.get_attribute(ItemAttribute::Article) {
                buf.push(Self::TAG_ARTICLE);
                write_string(&mut buf, s);
            }
        }

        // Plural name override
        if self.has_attribute(ItemAttribute::PluralName) {
            if let Some(AttributeValue::String(s)) = self.get_attribute(ItemAttribute::PluralName) {
                buf.push(Self::TAG_PLURALNAME);
                write_string(&mut buf, s);
            }
        }

        // Weight override
        if self.has_attribute(ItemAttribute::Weight) {
            if let Some(AttributeValue::Integer(w)) = self.get_attribute(ItemAttribute::Weight) {
                buf.push(Self::TAG_WEIGHT);
                buf.extend_from_slice(&(*w as u32).to_le_bytes());
            }
        }

        // Attack override
        if self.has_attribute(ItemAttribute::Attack) {
            if let Some(AttributeValue::Integer(a)) = self.get_attribute(ItemAttribute::Attack) {
                buf.push(Self::TAG_ATTACK);
                buf.extend_from_slice(&(*a as i32).to_le_bytes());
            }
        }

        // Attack speed override
        if self.has_attribute(ItemAttribute::AttackSpeed) {
            if let Some(AttributeValue::Integer(s)) = self.get_attribute(ItemAttribute::AttackSpeed)
            {
                buf.push(Self::TAG_ATTACK_SPEED);
                buf.extend_from_slice(&(*s as u32).to_le_bytes());
            }
        }

        // Defense override
        if self.has_attribute(ItemAttribute::Defense) {
            if let Some(AttributeValue::Integer(d)) = self.get_attribute(ItemAttribute::Defense) {
                buf.push(Self::TAG_DEFENSE);
                buf.extend_from_slice(&(*d as i32).to_le_bytes());
            }
        }

        // Extra defense override
        if self.has_attribute(ItemAttribute::ExtraDefense) {
            if let Some(AttributeValue::Integer(d)) =
                self.get_attribute(ItemAttribute::ExtraDefense)
            {
                buf.push(Self::TAG_EXTRADEFENSE);
                buf.extend_from_slice(&(*d as i32).to_le_bytes());
            }
        }

        // Armor override
        if self.has_attribute(ItemAttribute::Armor) {
            if let Some(AttributeValue::Integer(a)) = self.get_attribute(ItemAttribute::Armor) {
                buf.push(Self::TAG_ARMOR);
                buf.extend_from_slice(&(*a as i32).to_le_bytes());
            }
        }

        // Hit chance override
        if self.has_attribute(ItemAttribute::HitChance) {
            if let Some(AttributeValue::Integer(h)) = self.get_attribute(ItemAttribute::HitChance) {
                buf.push(Self::TAG_HITCHANCE);
                buf.push(*h as u8);
            }
        }

        // Shoot range override
        if self.has_attribute(ItemAttribute::ShootRange) {
            if let Some(AttributeValue::Integer(s)) = self.get_attribute(ItemAttribute::ShootRange)
            {
                buf.push(Self::TAG_SHOOTRANGE);
                buf.push(*s as u8);
            }
        }

        // Decay-to override
        if self.has_attribute(ItemAttribute::DecayTo) {
            if let Some(AttributeValue::Integer(d)) = self.get_attribute(ItemAttribute::DecayTo) {
                buf.push(Self::TAG_DECAYTO);
                buf.extend_from_slice(&(*d as i32).to_le_bytes());
            }
        }

        // Wrap ID override
        if self.has_attribute(ItemAttribute::WrapId) {
            if let Some(AttributeValue::Integer(w)) = self.get_attribute(ItemAttribute::WrapId) {
                buf.push(Self::TAG_WRAPID);
                buf.extend_from_slice(&(*w as u16).to_le_bytes());
            }
        }

        // Store item flag
        if self.has_attribute(ItemAttribute::StoreItem) {
            if let Some(AttributeValue::Integer(s)) = self.get_attribute(ItemAttribute::StoreItem) {
                buf.push(Self::TAG_STOREITEM);
                buf.push(*s as u8);
            }
        }

        // Open container flag
        if self.has_attribute(ItemAttribute::OpenContainer) {
            if let Some(AttributeValue::Integer(o)) =
                self.get_attribute(ItemAttribute::OpenContainer)
            {
                buf.push(Self::TAG_OPENCONTAINER);
                buf.push(*o as u8);
            }
        }

        // Terminator
        buf.push(0x00);
        buf
    }

    /// Deserialize attributes from a byte slice produced by `serialize_attr`.
    ///
    /// Returns `Err` with a message if the data is malformed.
    pub fn unserialize_attr(&mut self, data: &[u8]) -> Result<(), String> {
        let mut pos = 0usize;

        loop {
            if pos >= data.len() {
                return Err("unserialize_attr: unexpected end of data".into());
            }
            let tag = data[pos];
            pos += 1;

            if tag == 0x00 {
                break;
            }

            match tag {
                Self::TAG_COUNT => {
                    let v = read_u8(data, &mut pos)?;
                    self.count = v;
                }
                Self::TAG_ACTION_ID => {
                    let v = read_u16_le(data, &mut pos)?;
                    self.set_action_id(v);
                }
                Self::TAG_UNIQUE_ID => {
                    let v = read_u16_le(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::UniqueId, AttributeValue::Integer(v as i64));
                }
                Self::TAG_TEXT => {
                    let s = read_string(data, &mut pos)?;
                    self.set_text(s);
                }
                Self::TAG_DESC => {
                    let s = read_string(data, &mut pos)?;
                    self.set_special_description(s);
                }
                Self::TAG_DURATION => {
                    let v = read_i32_le(data, &mut pos)?;
                    self.set_duration(v.max(0));
                }
                Self::TAG_DECAYING_STATE => {
                    let v = read_u8(data, &mut pos)?;
                    // Any non-False decay state becomes Pending on load (matches C++)
                    if v != 0 {
                        self.set_decaying(ItemDecayState::Pending);
                    } else {
                        self.set_decaying(ItemDecayState::False);
                    }
                }
                Self::TAG_WRITTENDATE => {
                    let v = read_u32_le(data, &mut pos)?;
                    // C++ stores writtenDate as int32_t even though the
                    // wire is unsigned — preserve that signed reinterpret.
                    self.set_date(v as i32);
                }
                Self::TAG_WRITTENBY => {
                    let s = read_string(data, &mut pos)?;
                    self.set_writer(s);
                }
                Self::TAG_CHARGES => {
                    let v = read_u16_le(data, &mut pos)?;
                    self.set_charges(v);
                }
                Self::TAG_NAME => {
                    let s = read_string(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::Name, AttributeValue::String(s));
                }
                Self::TAG_ARTICLE => {
                    let s = read_string(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::Article, AttributeValue::String(s));
                }
                Self::TAG_PLURALNAME => {
                    let s = read_string(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::PluralName, AttributeValue::String(s));
                }
                Self::TAG_WEIGHT => {
                    let v = read_u32_le(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::Weight, AttributeValue::Integer(v as i64));
                }
                Self::TAG_ATTACK => {
                    let v = read_i32_le(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::Attack, AttributeValue::Integer(v as i64));
                }
                Self::TAG_ATTACK_SPEED => {
                    let v = read_u32_le(data, &mut pos)?;
                    self.set_attack_speed(v);
                }
                Self::TAG_DEFENSE => {
                    let v = read_i32_le(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::Defense, AttributeValue::Integer(v as i64));
                }
                Self::TAG_EXTRADEFENSE => {
                    let v = read_i32_le(data, &mut pos)?;
                    self.set_attribute(
                        ItemAttribute::ExtraDefense,
                        AttributeValue::Integer(v as i64),
                    );
                }
                Self::TAG_ARMOR => {
                    let v = read_i32_le(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::Armor, AttributeValue::Integer(v as i64));
                }
                Self::TAG_HITCHANCE => {
                    let v = read_u8(data, &mut pos)?;
                    self.set_attribute(
                        ItemAttribute::HitChance,
                        AttributeValue::Integer(v as i8 as i64),
                    );
                }
                Self::TAG_SHOOTRANGE => {
                    let v = read_u8(data, &mut pos)?;
                    self.set_attribute(
                        ItemAttribute::ShootRange,
                        AttributeValue::Integer(v as i64),
                    );
                }
                Self::TAG_DECAYTO => {
                    let v = read_i32_le(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::DecayTo, AttributeValue::Integer(v as i64));
                }
                Self::TAG_WRAPID => {
                    let v = read_u16_le(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::WrapId, AttributeValue::Integer(v as i64));
                }
                Self::TAG_STOREITEM => {
                    let v = read_u8(data, &mut pos)?;
                    self.set_attribute(ItemAttribute::StoreItem, AttributeValue::Integer(v as i64));
                }
                Self::TAG_OPENCONTAINER => {
                    let v = read_u8(data, &mut pos)?;
                    self.set_attribute(
                        ItemAttribute::OpenContainer,
                        AttributeValue::Integer(v as i64),
                    );
                }
                _ => {
                    return Err(format!(
                        "unserialize_attr: unknown attribute tag 0x{:02X}",
                        tag
                    ));
                }
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Attribute-override accessors (mirrors C++ getPluralName / getArticle)
    // -----------------------------------------------------------------------

    pub fn get_plural_name(&self) -> String {
        if let Some(AttributeValue::String(s)) = self.get_attribute(ItemAttribute::PluralName) {
            return s.clone();
        }
        self.item_type.get_plural_name()
    }

    pub fn get_article(&self) -> &str {
        if let Some(AttributeValue::String(s)) = self.get_attribute(ItemAttribute::Article) {
            return s.as_str();
        }
        self.item_type.article.as_str()
    }

    // -----------------------------------------------------------------------
    // set_store_item (mirrors C++ Item::setStoreItem)
    // -----------------------------------------------------------------------

    pub fn set_store_item(&mut self, value: bool) {
        self.set_attribute(
            ItemAttribute::StoreItem,
            AttributeValue::Integer(value as i64),
        );
    }

    // -----------------------------------------------------------------------
    // set_sub_type (mirrors C++ Item::setSubType)
    // -----------------------------------------------------------------------

    pub fn set_sub_type(&mut self, n: u16) {
        if self.item_type.is_fluid_container() || self.item_type.is_splash() {
            self.set_fluid_type(n);
        } else if self.item_type.stackable {
            self.set_item_count(n as u8);
        } else if self.item_type.charges != 0 {
            self.set_charges(n);
        } else {
            self.set_item_count(n as u8);
        }
    }

    // -----------------------------------------------------------------------
    // set_default_subtype (mirrors C++ Item::setDefaultSubtype)
    // -----------------------------------------------------------------------

    pub fn set_default_subtype(&mut self) {
        self.count = 1;
        if self.item_type.charges != 0 {
            if self.item_type.stackable {
                self.set_item_count(self.item_type.charges as u8);
            } else {
                self.set_charges(self.item_type.charges as u16);
            }
        }
    }

    // -----------------------------------------------------------------------
    // can_transform (mirrors C++ virtual bool Item::canTransform())
    // -----------------------------------------------------------------------

    pub fn can_transform(&self) -> bool {
        true
    }

    // -----------------------------------------------------------------------
    // Type-delegate predicates (mirrors C++ Item::isGroundTile etc.)
    // -----------------------------------------------------------------------

    pub fn is_ground_tile(&self) -> bool {
        self.item_type.is_ground_tile()
    }

    pub fn is_magic_field(&self) -> bool {
        self.item_type.is_magic_field()
    }

    pub fn is_podium(&self) -> bool {
        self.item_type.is_podium()
    }

    pub fn has_walk_stack(&self) -> bool {
        self.item_type.walk_stack
    }

    pub fn is_supply(&self) -> bool {
        self.item_type.is_supply()
    }
}

// ---------------------------------------------------------------------------
// Serialization helpers (module-private)
// ---------------------------------------------------------------------------

fn write_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len() as u16;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
}

fn read_u8(data: &[u8], pos: &mut usize) -> Result<u8, String> {
    if *pos >= data.len() {
        return Err("read_u8: unexpected end".into());
    }
    let v = data[*pos];
    *pos += 1;
    Ok(v)
}

fn read_u16_le(data: &[u8], pos: &mut usize) -> Result<u16, String> {
    if *pos + 2 > data.len() {
        return Err("read_u16_le: unexpected end".into());
    }
    let v = u16::from_le_bytes([data[*pos], data[*pos + 1]]);
    *pos += 2;
    Ok(v)
}

fn read_u32_le(data: &[u8], pos: &mut usize) -> Result<u32, String> {
    if *pos + 4 > data.len() {
        return Err("read_u32_le: unexpected end".into());
    }
    let v = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]]);
    *pos += 4;
    Ok(v)
}

fn read_i32_le(data: &[u8], pos: &mut usize) -> Result<i32, String> {
    read_u32_le(data, pos).map(|v| v as i32)
}

fn read_string(data: &[u8], pos: &mut usize) -> Result<String, String> {
    let len = read_u16_le(data, pos)? as usize;
    if *pos + len > data.len() {
        return Err("read_string: unexpected end".into());
    }
    let s = std::str::from_utf8(&data[*pos..*pos + len])
        .map_err(|e| format!("read_string: invalid UTF-8: {}", e))?
        .to_string();
    *pos += len;
    Ok(s)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items_registry::ItemTypeData;
    use crate::items_registry::ItemTypeKind;
    use forgottenserver_common::itemloader::ItemGroup;

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn make_item_type(
        id: u16,
        weight: u32,
        stackable: bool,
        pickupable: bool,
        moveable: bool,
        name: &str,
        article: &str,
    ) -> Arc<ItemTypeData> {
        let it = ItemTypeData {
            id,
            client_id: id + 100,
            weight,
            stackable,
            pickupable,
            moveable,
            name: name.to_string(),
            article: article.to_string(),
            show_count: true,
            ..Default::default()
        };
        Arc::new(it)
    }

    // -----------------------------------------------------------------------
    // Item::new / get_id / get_client_id / get_name
    // -----------------------------------------------------------------------

    #[test]
    fn test_item_new_basic() {
        let t = make_item_type(42, 100, false, true, true, "iron sword", "an");
        let item = Item::new(t.clone(), 1);
        assert_eq!(item.get_id(), 42);
        assert_eq!(item.get_client_id(), 142);
        assert_eq!(item.get_name(), "iron sword");
        assert_eq!(item.get_count(), 1);
    }

    #[test]
    fn test_item_name_override_via_attribute() {
        let t = make_item_type(1, 0, false, false, false, "generic", "a");
        let mut item = Item::new(t, 1);
        item.set_attribute(
            ItemAttribute::Name,
            AttributeValue::String("custom name".to_string()),
        );
        assert_eq!(item.get_name(), "custom name");
    }

    // -----------------------------------------------------------------------
    // Weight calculation
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_weight_non_stackable() {
        let t = make_item_type(10, 200, false, true, true, "shield", "a");
        let item = Item::new(t, 1);
        assert_eq!(item.get_weight(), 200);
    }

    #[test]
    fn test_get_weight_stackable_count_5() {
        let t = make_item_type(20, 50, true, true, true, "gold coin", "a");
        let item = Item::new(t, 5);
        // weight = base * count = 50 * 5 = 250
        assert_eq!(item.get_weight(), 250);
    }

    #[test]
    fn test_get_weight_stackable_count_1() {
        let t = make_item_type(21, 30, true, true, true, "arrow", "an");
        let item = Item::new(t, 1);
        assert_eq!(item.get_weight(), 30);
    }

    // -----------------------------------------------------------------------
    // Boolean flag delegation
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_stackable_delegate() {
        let t = make_item_type(1, 0, true, false, false, "coin", "a");
        let item = Item::new(t, 1);
        assert!(item.is_stackable());
    }

    #[test]
    fn test_is_not_stackable() {
        let t = make_item_type(2, 0, false, false, false, "sword", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_stackable());
    }

    #[test]
    fn test_is_pickupable_delegate() {
        let t = make_item_type(3, 0, false, true, true, "helmet", "a");
        let item = Item::new(t, 1);
        assert!(item.is_pickupable());
    }

    #[test]
    fn test_is_not_pickupable() {
        let t = make_item_type(4, 0, false, false, false, "wall", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_pickupable());
    }

    #[test]
    fn test_is_moveable_delegate() {
        let t = make_item_type(5, 0, false, false, true, "chest", "a");
        let item = Item::new(t, 1);
        assert!(item.is_moveable());
    }

    // -----------------------------------------------------------------------
    // Attribute set / get / remove round-trips
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_get_attribute_integer() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attribute(ItemAttribute::ActionId, AttributeValue::Integer(200));
        assert_eq!(
            item.get_attribute(ItemAttribute::ActionId),
            Some(&AttributeValue::Integer(200))
        );
    }

    #[test]
    fn test_set_get_attribute_string() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attribute(
            ItemAttribute::Text,
            AttributeValue::String("hello world".to_string()),
        );
        assert_eq!(
            item.get_attribute(ItemAttribute::Text),
            Some(&AttributeValue::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_set_get_attribute_float() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attribute(
            ItemAttribute::Weight,
            AttributeValue::Float(std::f64::consts::PI),
        );
        assert_eq!(
            item.get_attribute(ItemAttribute::Weight),
            Some(&AttributeValue::Float(std::f64::consts::PI))
        );
    }

    #[test]
    fn test_set_get_attribute_boolean() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attribute(ItemAttribute::StoreItem, AttributeValue::Boolean(true));
        assert_eq!(
            item.get_attribute(ItemAttribute::StoreItem),
            Some(&AttributeValue::Boolean(true))
        );
    }

    #[test]
    fn test_remove_attribute_returns_none_after() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attribute(ItemAttribute::UniqueId, AttributeValue::Integer(999));
        assert!(item.get_attribute(ItemAttribute::UniqueId).is_some());

        let removed = item.remove_attribute(ItemAttribute::UniqueId);
        assert!(removed);
        assert!(item.get_attribute(ItemAttribute::UniqueId).is_none());
    }

    #[test]
    fn test_remove_nonexistent_attribute_returns_false() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        assert!(!item.remove_attribute(ItemAttribute::Text));
    }

    #[test]
    fn test_missing_attribute_returns_none() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let item = Item::new(t, 1);
        assert!(item.get_attribute(ItemAttribute::ActionId).is_none());
    }

    // -----------------------------------------------------------------------
    // Description
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_description_returns_non_empty() {
        let t = make_item_type(1, 0, false, true, true, "golden key", "a");
        let item = Item::new(t, 1);
        let desc = item.get_description(0);
        assert!(!desc.is_empty());
        assert!(desc.contains("golden key"), "desc = {:?}", desc);
    }

    #[test]
    fn test_get_description_with_article() {
        let t = make_item_type(1, 0, false, true, true, "apple", "an");
        let item = Item::new(t, 1);
        let desc = item.get_description(0);
        assert_eq!(desc, "an apple");
    }

    #[test]
    fn test_get_description_stackable_plural() {
        let t = make_item_type(1, 10, true, true, true, "coin", "a");
        let item = Item::new(t, 5);
        let desc = item.get_description(0);
        // "5 coins"
        assert!(desc.contains("5"), "desc = {:?}", desc);
        assert!(desc.contains("coin"), "desc = {:?}", desc);
    }

    #[test]
    fn test_get_description_no_name_fallback() {
        let it = ItemTypeData {
            id: 7,
            name: String::new(),
            ..Default::default()
        };
        let t = Arc::new(it);
        let item = Item::new(t, 1);
        let desc = item.get_description(0);
        assert!(desc.contains("7"), "desc = {:?}", desc);
    }

    // -----------------------------------------------------------------------
    // Decay helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_decaying_false_by_default() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_decaying());
    }

    #[test]
    fn test_set_and_get_decay_state() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_decaying(ItemDecayState::True);
        assert!(item.is_decaying());
        assert_eq!(item.get_decay_state(), ItemDecayState::True);
    }

    #[test]
    fn test_set_decaying_pending() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_decaying(ItemDecayState::Pending);
        assert!(item.is_decaying());
        assert_eq!(item.get_decay_state(), ItemDecayState::Pending);
    }

    // -----------------------------------------------------------------------
    // Convenience accessors
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_and_get_action_id_clamped() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        // Values < 100 should be clamped to 100
        item.set_action_id(50);
        assert_eq!(item.get_action_id(), 100);

        item.set_action_id(200);
        assert_eq!(item.get_action_id(), 200);
    }

    #[test]
    fn test_set_and_get_charges() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_charges(10);
        assert_eq!(item.get_charges(), 10);
    }

    #[test]
    fn test_set_and_get_fluid_type() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_fluid_type(3);
        assert_eq!(item.get_fluid_type(), 3);
    }

    #[test]
    fn test_set_and_get_text() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_text("hello".to_string());
        assert_eq!(item.get_text(), "hello");
    }

    #[test]
    fn test_set_and_get_writer() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_writer("Bob".to_string());
        assert_eq!(item.get_writer(), "Bob");
    }

    #[test]
    fn test_set_and_get_date() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_date(1_600_000_000);
        assert_eq!(item.get_date(), 1_600_000_000);
    }

    #[test]
    fn test_set_and_get_duration() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_duration(3600);
        assert_eq!(item.get_duration(), 3600);
    }

    #[test]
    fn test_set_duration_negative_clamped_to_zero() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_duration(-100);
        assert_eq!(item.get_duration(), 0);
    }

    #[test]
    fn test_set_and_get_special_description() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_special_description("magic sword".to_string());
        assert_eq!(item.get_special_description(), "magic sword");
    }

    // -----------------------------------------------------------------------
    // Attack / defense / armor delegation
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_attack_defaults_to_type() {
        let it = ItemTypeData {
            id: 1,
            attack: 42,
            ..Default::default()
        };
        let t = Arc::new(it);
        let item = Item::new(t, 1);
        assert_eq!(item.get_attack(), 42);
    }

    #[test]
    fn test_get_attack_override() {
        let it = ItemTypeData {
            id: 1,
            attack: 10,
            ..Default::default()
        };
        let t = Arc::new(it);
        let mut item = Item::new(t, 1);
        item.set_attribute(ItemAttribute::Attack, AttributeValue::Integer(99));
        assert_eq!(item.get_attack(), 99);
    }

    // -----------------------------------------------------------------------
    // is_cleanable / can_decay
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_cleanable_pickupable_no_unique() {
        let t = make_item_type(1, 0, false, true, true, "x", "a");
        let item = Item::new(t, 1);
        assert!(item.is_cleanable());
    }

    #[test]
    fn test_is_not_cleanable_with_unique_id() {
        let t = make_item_type(1, 0, false, true, true, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attribute(ItemAttribute::UniqueId, AttributeValue::Integer(100));
        assert!(!item.is_cleanable());
    }

    #[test]
    fn test_is_not_cleanable_not_pickupable() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_cleanable());
    }

    // -----------------------------------------------------------------------
    // is_cleanable with action_id set (C++ also blocks cleanable when action_id set)
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_not_cleanable_with_action_id() {
        let t = make_item_type(1, 0, false, true, true, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_action_id(200);
        assert!(!item.is_cleanable());
    }

    // -----------------------------------------------------------------------
    // is_blocking (mirrors C++ isBlocking)
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_blocking_true_when_block_solid() {
        let it = ItemTypeData {
            id: 1,
            block_solid: true,
            ..Default::default()
        };
        let t = Arc::new(it);
        let item = Item::new(t, 1);
        assert!(item.is_blocking());
    }

    #[test]
    fn test_is_blocking_false_when_not_block_solid() {
        let it = ItemTypeData {
            id: 1,
            block_solid: false,
            ..Default::default()
        };
        let t = Arc::new(it);
        let item = Item::new(t, 1);
        assert!(!item.is_blocking());
    }

    // -----------------------------------------------------------------------
    // Stack count management (MAX_STACK_COUNT = 100)
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_item_count_normal() {
        let t = make_item_type(1, 10, true, true, true, "coin", "a");
        let mut item = Item::new(t, 1);
        item.set_item_count(50);
        assert_eq!(item.get_item_count(), 50);
    }

    #[test]
    fn test_set_item_count_clamped_at_100() {
        let t = make_item_type(1, 10, true, true, true, "coin", "a");
        let mut item = Item::new(t, 1);
        // 101 should be clamped to MAX_STACK_COUNT = 100
        item.set_item_count(101);
        assert_eq!(item.get_item_count(), 100);
    }

    #[test]
    fn test_set_item_count_exactly_100() {
        let t = make_item_type(1, 10, true, true, true, "coin", "a");
        let mut item = Item::new(t, 1);
        item.set_item_count(100);
        assert_eq!(item.get_item_count(), 100);
    }

    #[test]
    fn test_max_stack_count_constant() {
        assert_eq!(Item::MAX_STACK_COUNT, 100);
    }

    // -----------------------------------------------------------------------
    // decrease_duration
    // -----------------------------------------------------------------------

    #[test]
    fn test_decrease_duration_normal() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_duration(1000);
        item.decrease_duration(300);
        assert_eq!(item.get_duration(), 700);
    }

    #[test]
    fn test_decrease_duration_clamped_at_zero() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_duration(100);
        item.decrease_duration(500); // would go negative → clamp to 0
        assert_eq!(item.get_duration(), 0);
    }

    #[test]
    fn test_decrease_duration_exact_zero() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_duration(200);
        item.decrease_duration(200);
        assert_eq!(item.get_duration(), 0);
    }

    // -----------------------------------------------------------------------
    // set_unique_id idempotency (C++ Item::setUniqueId skips if already set)
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_unique_id_sets_when_not_present() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_unique_id(42);
        assert_eq!(item.get_unique_id(), 42);
    }

    #[test]
    fn test_set_unique_id_is_idempotent() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_unique_id(42);
        item.set_unique_id(99); // second call should be ignored
        assert_eq!(item.get_unique_id(), 42);
    }

    // -----------------------------------------------------------------------
    // Owner / CorpseOwner typed accessors
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_and_get_owner() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_owner(1234567);
        assert_eq!(item.get_owner(), 1234567);
    }

    #[test]
    fn test_get_owner_default_zero() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let item = Item::new(t, 1);
        assert_eq!(item.get_owner(), 0);
    }

    #[test]
    fn test_set_and_get_corpse_owner() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_corpse_owner(9876);
        assert_eq!(item.get_corpse_owner(), 9876);
    }

    #[test]
    fn test_get_corpse_owner_default_zero() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let item = Item::new(t, 1);
        assert_eq!(item.get_corpse_owner(), 0);
    }

    // -----------------------------------------------------------------------
    // HitChance / ShootRange typed accessors
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_and_get_hit_chance() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_hit_chance(-10);
        assert_eq!(item.get_hit_chance(), -10);
    }

    #[test]
    fn test_set_and_get_hit_chance_positive() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_hit_chance(20);
        assert_eq!(item.get_hit_chance(), 20);
    }

    #[test]
    fn test_set_and_get_shoot_range() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_shoot_range(7);
        assert_eq!(item.get_shoot_range(), 7);
    }

    // -----------------------------------------------------------------------
    // ExtraDefense typed accessor
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_and_get_extra_defense() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_extra_defense(5);
        assert_eq!(item.get_extra_defense(), 5);
    }

    #[test]
    fn test_get_extra_defense_fallback_to_type() {
        let it = ItemTypeData {
            id: 1,
            extra_defense: 8,
            ..Default::default()
        };
        let t = Arc::new(it);
        let item = Item::new(t, 1);
        assert_eq!(item.get_extra_defense(), 8);
    }

    // -----------------------------------------------------------------------
    // DecayTo typed accessor
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_and_get_decay_to() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_decay_to(999);
        assert_eq!(item.get_decay_to(), 999);
    }

    #[test]
    fn test_get_decay_to_fallback_to_type() {
        let it = ItemTypeData {
            id: 1,
            decay_to: 42,
            ..Default::default()
        };
        let t = Arc::new(it);
        let item = Item::new(t, 1);
        assert_eq!(item.get_decay_to(), 42);
    }

    // -----------------------------------------------------------------------
    // AttackSpeed minimum clamp (mirrors C++ setIntAttr ATTACK_SPEED clamping)
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_attack_speed_clamps_below_100() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attack_speed(50); // below 100 → should be clamped
        assert_eq!(item.get_attack_speed(), 100);
    }

    #[test]
    fn test_set_attack_speed_above_100_unchanged() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attack_speed(400);
        assert_eq!(item.get_attack_speed(), 400);
    }

    #[test]
    fn test_set_attack_speed_exactly_100() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_attack_speed(100);
        assert_eq!(item.get_attack_speed(), 100);
    }

    // -----------------------------------------------------------------------
    // Reset convenience methods
    // -----------------------------------------------------------------------

    #[test]
    fn test_reset_text() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_text("hello".to_string());
        assert_eq!(item.get_text(), "hello");
        item.reset_text();
        assert_eq!(item.get_text(), "");
    }

    #[test]
    fn test_reset_date() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_date(1234567);
        assert_eq!(item.get_date(), 1234567);
        item.reset_date();
        assert_eq!(item.get_date(), 0);
    }

    #[test]
    fn test_reset_writer() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_writer("Alice".to_string());
        assert_eq!(item.get_writer(), "Alice");
        item.reset_writer();
        assert_eq!(item.get_writer(), "");
    }

    // -----------------------------------------------------------------------
    // has_attribute
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_attribute_true_after_set() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        assert!(!item.has_attribute(ItemAttribute::Text));
        item.set_text("test".to_string());
        assert!(item.has_attribute(ItemAttribute::Text));
    }

    #[test]
    fn test_has_attribute_false_after_remove() {
        let t = make_item_type(1, 0, false, false, false, "x", "a");
        let mut item = Item::new(t, 1);
        item.set_date(100);
        assert!(item.has_attribute(ItemAttribute::Date));
        item.reset_date();
        assert!(!item.has_attribute(ItemAttribute::Date));
    }

    // -----------------------------------------------------------------------
    // deep_copy — independent clone (mirrors C++ Item::clone)
    // -----------------------------------------------------------------------

    #[test]
    fn test_deep_copy_produces_independent_item() {
        let t = make_item_type(1, 50, true, true, true, "coin", "a");
        let mut original = Item::new(t, 10);
        original.set_text("original".to_string());
        original.set_charges(5);

        let mut copy = original.deep_copy();

        // Mutations on copy do not affect original
        copy.set_text("modified".to_string());
        copy.set_charges(99);

        assert_eq!(original.get_text(), "original");
        assert_eq!(original.get_charges(), 5);
        assert_eq!(copy.get_text(), "modified");
        assert_eq!(copy.get_charges(), 99);
    }

    #[test]
    fn test_deep_copy_shares_item_type_arc() {
        let t = make_item_type(1, 50, true, true, true, "coin", "a");
        let original = Item::new(t, 5);
        let copy = original.deep_copy();

        // Both point to the same type blueprint (Arc)
        assert_eq!(original.get_id(), copy.get_id());
        assert!(Arc::ptr_eq(&original.item_type, &copy.item_type));
    }

    #[test]
    fn test_deep_copy_same_count() {
        let t = make_item_type(1, 10, true, true, true, "arrow", "an");
        let original = Item::new(t, 77);
        let copy = original.deep_copy();
        assert_eq!(copy.get_count(), 77);
    }

    // -----------------------------------------------------------------------
    // transform — change item type (mirrors C++ Item::setID)
    // -----------------------------------------------------------------------

    #[test]
    fn test_transform_changes_item_type() {
        let t1 = make_item_type(1, 100, false, true, true, "old item", "an");
        let t2 = make_item_type(2, 200, false, true, true, "new item", "a");
        let mut item = Item::new(t1, 1);
        assert_eq!(item.get_id(), 1);

        item.transform(t2);
        assert_eq!(item.get_id(), 2);
        assert_eq!(item.get_name(), "new item");
    }

    #[test]
    fn test_transform_clears_corpse_owner() {
        let t1 = make_item_type(1, 100, false, true, true, "old", "a");
        let t2 = make_item_type(2, 100, false, true, true, "new", "a");
        let mut item = Item::new(t1, 1);
        item.set_corpse_owner(12345);
        assert_eq!(item.get_corpse_owner(), 12345);

        item.transform(t2);
        assert_eq!(item.get_corpse_owner(), 0);
    }

    #[test]
    fn test_transform_with_decay_resets_state_to_false() {
        let t1 = make_item_type(1, 100, false, true, true, "old", "a");
        // New type has decay time
        let it2 = ItemTypeData {
            id: 2,
            decay_time_min: 60,
            decay_time_max: 0,
            decay_to: 0,
            ..Default::default()
        };
        let t2 = Arc::new(it2);

        let mut item = Item::new(t1, 1);
        item.set_decaying(ItemDecayState::True);

        item.transform(t2);
        assert_eq!(item.get_decay_state(), ItemDecayState::False);
        assert!(item.get_duration() > 0);
    }

    #[test]
    fn test_transform_to_non_decaying_clears_duration() {
        let t1 = make_item_type(1, 100, false, true, true, "old", "a");
        // New type has no decay (decay_to < 0 and no decay times)
        let it2 = ItemTypeData {
            id: 2,
            decay_time_min: 0,
            decay_time_max: 0,
            decay_to: -1,
            ..Default::default()
        };
        let t2 = Arc::new(it2);

        let mut item = Item::new(t1, 1);
        item.set_duration(5000);
        item.set_decaying(ItemDecayState::True);

        item.transform(t2);
        assert!(!item.has_attribute(ItemAttribute::Duration));
        assert!(!item.has_attribute(ItemAttribute::DecayState));
    }

    // -----------------------------------------------------------------------
    // Serialize / Unserialize round-trips
    // -----------------------------------------------------------------------

    fn make_moveable_item() -> Item {
        let it = ItemTypeData {
            id: 1,
            moveable: true,
            name: "test item".to_string(),
            ..Default::default()
        };
        Item::new(Arc::new(it), 1)
    }

    #[test]
    fn test_serialize_unserialize_action_id() {
        let mut item = make_moveable_item();
        item.set_action_id(200);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_action_id(), 200);
    }

    /// Unique ID is NOT serialized by C++ `serializeAttr` (it is a map-placed
    /// property, deserialized from OTB map nodes but never written back).
    /// This test confirms the Rust implementation preserves that behaviour.
    #[test]
    fn test_unique_id_is_not_serialized() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::UniqueId, AttributeValue::Integer(1234));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        // After round-trip, unique_id is not restored (not written by serialize_attr)
        assert_eq!(item2.get_unique_id(), 0);
    }

    #[test]
    fn test_serialize_unserialize_text() {
        let mut item = make_moveable_item();
        item.set_text("hello world".to_string());

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_text(), "hello world");
    }

    #[test]
    fn test_serialize_unserialize_special_description() {
        let mut item = make_moveable_item();
        item.set_special_description("magic sword".to_string());

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_special_description(), "magic sword");
    }

    #[test]
    fn test_serialize_unserialize_duration() {
        let mut item = make_moveable_item();
        item.set_duration(3600);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_duration(), 3600);
    }

    #[test]
    fn test_serialize_unserialize_decay_state_true_becomes_pending() {
        let mut item = make_moveable_item();
        item.set_duration(1000);
        item.set_decaying(ItemDecayState::True);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        // C++ behaviour: any non-False decay state becomes Pending on load
        assert_eq!(item2.get_decay_state(), ItemDecayState::Pending);
    }

    #[test]
    fn test_serialize_unserialize_decay_state_false_stays_false() {
        let mut item = make_moveable_item();
        item.set_decaying(ItemDecayState::False);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_decay_state(), ItemDecayState::False);
    }

    #[test]
    fn test_serialize_unserialize_written_date() {
        let mut item = make_moveable_item();
        item.set_date(1_700_000_000);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_date(), 1_700_000_000);
    }

    #[test]
    fn test_serialize_unserialize_writer() {
        let mut item = make_moveable_item();
        item.set_writer("Alice".to_string());

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_writer(), "Alice");
    }

    #[test]
    fn test_serialize_unserialize_charges() {
        let mut item = make_moveable_item();
        item.set_charges(42);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_charges(), 42);
    }

    #[test]
    fn test_serialize_unserialize_name_override() {
        let mut item = make_moveable_item();
        item.set_attribute(
            ItemAttribute::Name,
            AttributeValue::String("custom name".to_string()),
        );

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(
            item2.get_attribute(ItemAttribute::Name),
            Some(&AttributeValue::String("custom name".to_string()))
        );
    }

    #[test]
    fn test_serialize_unserialize_article_override() {
        let mut item = make_moveable_item();
        item.set_attribute(
            ItemAttribute::Article,
            AttributeValue::String("the".to_string()),
        );

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(
            item2.get_attribute(ItemAttribute::Article),
            Some(&AttributeValue::String("the".to_string()))
        );
    }

    #[test]
    fn test_serialize_unserialize_plural_name_override() {
        let mut item = make_moveable_item();
        item.set_attribute(
            ItemAttribute::PluralName,
            AttributeValue::String("swords".to_string()),
        );

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(
            item2.get_attribute(ItemAttribute::PluralName),
            Some(&AttributeValue::String("swords".to_string()))
        );
    }

    #[test]
    fn test_serialize_unserialize_weight_override() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::Weight, AttributeValue::Integer(500));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_base_weight(), 500);
    }

    #[test]
    fn test_serialize_unserialize_attack_override() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::Attack, AttributeValue::Integer(99));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_attack(), 99);
    }

    #[test]
    fn test_serialize_unserialize_attack_speed_with_clamp() {
        let mut item = make_moveable_item();
        item.set_attack_speed(300);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_attack_speed(), 300);
    }

    #[test]
    fn test_serialize_unserialize_defense_override() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::Defense, AttributeValue::Integer(15));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_defense(), 15);
    }

    #[test]
    fn test_serialize_unserialize_extra_defense() {
        let mut item = make_moveable_item();
        item.set_extra_defense(3);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_extra_defense(), 3);
    }

    #[test]
    fn test_serialize_unserialize_armor_override() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::Armor, AttributeValue::Integer(12));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_armor(), 12);
    }

    #[test]
    fn test_serialize_unserialize_hit_chance() {
        let mut item = make_moveable_item();
        item.set_hit_chance(-5);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_hit_chance(), -5);
    }

    #[test]
    fn test_serialize_unserialize_shoot_range() {
        let mut item = make_moveable_item();
        item.set_shoot_range(7);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_shoot_range(), 7);
    }

    #[test]
    fn test_serialize_unserialize_decay_to() {
        let mut item = make_moveable_item();
        item.set_decay_to(100);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_decay_to(), 100);
    }

    #[test]
    fn test_serialize_unserialize_wrap_id() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::WrapId, AttributeValue::Integer(55));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(
            item2.get_attribute(ItemAttribute::WrapId),
            Some(&AttributeValue::Integer(55))
        );
    }

    #[test]
    fn test_serialize_unserialize_store_item() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::StoreItem, AttributeValue::Integer(1));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(
            item2.get_attribute(ItemAttribute::StoreItem),
            Some(&AttributeValue::Integer(1))
        );
    }

    #[test]
    fn test_serialize_unserialize_open_container() {
        let mut item = make_moveable_item();
        item.set_attribute(ItemAttribute::OpenContainer, AttributeValue::Integer(1));

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(
            item2.get_attribute(ItemAttribute::OpenContainer),
            Some(&AttributeValue::Integer(1))
        );
    }

    #[test]
    fn test_serialize_unserialize_empty_item() {
        let item = make_moveable_item();
        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        // No attributes should be set on item2 (other than defaults from type)
        assert!(!item2.has_attribute(ItemAttribute::Text));
        assert!(!item2.has_attribute(ItemAttribute::ActionId));
    }

    #[test]
    fn test_serialize_unserialize_stackable_count() {
        let it = ItemTypeData {
            id: 1,
            stackable: true,
            ..Default::default()
        };
        let t = Arc::new(it);
        let mut item = Item::new(t.clone(), 1);
        item.set_item_count(77);

        let blob = item.serialize_attr();

        let mut item2 = Item::new(t, 1);
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_item_count(), 77);
    }

    #[test]
    fn test_unserialize_unknown_tag_returns_error() {
        // Build a buffer with an unknown tag (0xEE)
        let data = vec![0xEE, 0x00];
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("unknown attribute tag"), "msg: {}", msg);
    }

    #[test]
    fn test_unserialize_truncated_returns_error() {
        // Only partial data (action_id tag but no payload)
        let data = vec![0x04]; // TAG_ACTION_ID but no u16 follows
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Full multi-attribute round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_serialize_unserialize_multiple_attributes_round_trip() {
        let mut item = make_moveable_item();
        item.set_action_id(300);
        item.set_text("note text".to_string());
        item.set_writer("Bob".to_string());
        item.set_date(1_600_000_000);
        item.set_special_description("a fine item".to_string());
        item.set_duration(7200);
        item.set_decaying(ItemDecayState::Pending);
        item.set_charges(25);
        item.set_attribute(ItemAttribute::Armor, AttributeValue::Integer(10));
        item.set_attribute(ItemAttribute::Attack, AttributeValue::Integer(20));
        item.set_extra_defense(3);
        item.set_decay_to(0);

        let blob = item.serialize_attr();

        let mut item2 = make_moveable_item();
        item2.unserialize_attr(&blob).expect("ok");

        assert_eq!(item2.get_action_id(), 300);
        assert_eq!(item2.get_text(), "note text");
        assert_eq!(item2.get_writer(), "Bob");
        assert_eq!(item2.get_date(), 1_600_000_000);
        assert_eq!(item2.get_special_description(), "a fine item");
        assert_eq!(item2.get_duration(), 7200);
        // Pending → Pending (non-False stays Pending on load)
        assert_eq!(item2.get_decay_state(), ItemDecayState::Pending);
        assert_eq!(item2.get_charges(), 25);
        assert_eq!(item2.get_armor(), 10);
        assert_eq!(item2.get_attack(), 20);
        assert_eq!(item2.get_extra_defense(), 3);
        assert_eq!(item2.get_decay_to(), 0);
    }

    // -----------------------------------------------------------------------
    // Pass-through delegators (lines 163-221): is_useable, is_rotatable,
    // is_hangable, block_solid/path_find/projectile, has_height, look_through,
    // always_on_top, allow_dist_read, is_readable, show_count, show_duration,
    // show_charges, show_attributes
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_useable_delegates_true_and_false() {
        let t_true = Arc::new(ItemTypeData {
            useable: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).is_useable());

        let t_false = Arc::new(ItemTypeData {
            useable: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).is_useable());
    }

    #[test]
    fn test_is_rotatable_delegate() {
        let t_true = Arc::new(ItemTypeData {
            rotatable: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).is_rotatable());

        let t_false = Arc::new(ItemTypeData {
            rotatable: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).is_rotatable());
    }

    #[test]
    fn test_is_hangable_delegate() {
        let t_true = Arc::new(ItemTypeData {
            is_hangable: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).is_hangable());

        let t_false = Arc::new(ItemTypeData {
            is_hangable: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).is_hangable());
    }

    #[test]
    fn test_block_solid_delegate() {
        let t_true = Arc::new(ItemTypeData {
            block_solid: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).block_solid());

        let t_false = Arc::new(ItemTypeData {
            block_solid: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).block_solid());
    }

    #[test]
    fn test_block_path_find_delegate() {
        let t_true = Arc::new(ItemTypeData {
            block_path_find: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).block_path_find());

        let t_false = Arc::new(ItemTypeData {
            block_path_find: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).block_path_find());
    }

    #[test]
    fn test_block_projectile_delegate() {
        let t_true = Arc::new(ItemTypeData {
            block_projectile: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).block_projectile());

        let t_false = Arc::new(ItemTypeData {
            block_projectile: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).block_projectile());
    }

    #[test]
    fn test_has_height_delegate() {
        let t_true = Arc::new(ItemTypeData {
            has_height: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).has_height());

        let t_false = Arc::new(ItemTypeData {
            has_height: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).has_height());
    }

    #[test]
    fn test_look_through_delegate() {
        let t_true = Arc::new(ItemTypeData {
            look_through: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).look_through());

        let t_false = Arc::new(ItemTypeData {
            look_through: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).look_through());
    }

    #[test]
    fn test_always_on_top_delegate() {
        let t_true = Arc::new(ItemTypeData {
            always_on_top: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).always_on_top());

        let t_false = Arc::new(ItemTypeData {
            always_on_top: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).always_on_top());
    }

    #[test]
    fn test_allow_dist_read_delegate() {
        let t_true = Arc::new(ItemTypeData {
            allow_dist_read: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).allow_dist_read());

        let t_false = Arc::new(ItemTypeData {
            allow_dist_read: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).allow_dist_read());
    }

    #[test]
    fn test_is_readable_delegate() {
        let t_true = Arc::new(ItemTypeData {
            can_read_text: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).is_readable());

        let t_false = Arc::new(ItemTypeData {
            can_read_text: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).is_readable());
    }

    #[test]
    fn test_show_count_delegate() {
        let t_true = Arc::new(ItemTypeData {
            show_count: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).show_count());

        let t_false = Arc::new(ItemTypeData {
            show_count: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).show_count());
    }

    #[test]
    fn test_show_duration_delegate() {
        let t_true = Arc::new(ItemTypeData {
            show_duration: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).show_duration());

        let t_false = Arc::new(ItemTypeData {
            show_duration: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).show_duration());
    }

    #[test]
    fn test_show_charges_delegate() {
        let t_true = Arc::new(ItemTypeData {
            show_charges: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).show_charges());

        let t_false = Arc::new(ItemTypeData {
            show_charges: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).show_charges());
    }

    #[test]
    fn test_show_attributes_delegate() {
        let t_true = Arc::new(ItemTypeData {
            show_attributes: true,
            ..Default::default()
        });
        assert!(Item::new(t_true, 1).show_attributes());

        let t_false = Arc::new(ItemTypeData {
            show_attributes: false,
            ..Default::default()
        });
        assert!(!Item::new(t_false, 1).show_attributes());
    }

    // -----------------------------------------------------------------------
    // Fallback arms (no override attribute set, getter returns default)
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_duration_default_zero() {
        let t = Arc::new(ItemTypeData::default());
        let item = Item::new(t, 1);
        // No Duration attribute set → falls through `_` arm → returns 0
        assert_eq!(item.get_duration(), 0);
    }

    #[test]
    fn test_get_fluid_type_default_zero() {
        let t = Arc::new(ItemTypeData::default());
        let item = Item::new(t, 1);
        // No FluidType attribute → falls through to 0
        assert_eq!(item.get_fluid_type(), 0);
    }

    #[test]
    fn test_get_defense_falls_back_to_type() {
        let t = Arc::new(ItemTypeData {
            defense: 42,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        // No Defense attribute override → falls back to type
        assert_eq!(item.get_defense(), 42);
    }

    #[test]
    fn test_get_armor_falls_back_to_type() {
        let t = Arc::new(ItemTypeData {
            armor: 17,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        assert_eq!(item.get_armor(), 17);
    }

    #[test]
    fn test_get_attack_speed_falls_back_to_type() {
        let t = Arc::new(ItemTypeData {
            attack_speed: 250,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        assert_eq!(item.get_attack_speed(), 250);
    }

    #[test]
    fn test_get_hit_chance_default_zero() {
        let t = Arc::new(ItemTypeData::default());
        let item = Item::new(t, 1);
        // No HitChance attribute → falls through to 0
        assert_eq!(item.get_hit_chance(), 0);
    }

    #[test]
    fn test_get_shoot_range_uses_fallback_when_no_attr() {
        let t = Arc::new(ItemTypeData {
            shoot_range: 5,
            attack_speed: 200, // intentionally different — the old impl
            // incorrectly fell back to attack_speed.
            ..Default::default()
        });
        let item = Item::new(t, 1);
        // No ShootRange attribute → C++ falls back to `items[id].shootRange`.
        assert_eq!(item.get_shoot_range(), 5);
    }

    // -----------------------------------------------------------------------
    // get_description branches:
    //  - stackable + count > 1 + !show_count → just name
    //  - non-stackable + empty article → just name
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_description_stackable_no_show_count_returns_just_name() {
        let t = Arc::new(ItemTypeData {
            id: 1,
            name: "gold".to_string(),
            stackable: true,
            show_count: false,
            ..Default::default()
        });
        let item = Item::new(t, 5);
        // stackable + count > 1 + !show_count → return name only
        assert_eq!(item.get_description(0), "gold");
    }

    #[test]
    fn test_get_description_empty_article_returns_just_name() {
        let t = Arc::new(ItemTypeData {
            id: 1,
            name: "thing".to_string(),
            article: String::new(),
            stackable: false,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        // non-stackable + empty article → return name only
        assert_eq!(item.get_description(0), "thing");
    }

    // -----------------------------------------------------------------------
    // can_decay covers all branches
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_decay_false_when_type_has_no_decay() {
        // Default: decay_to=-1, min=0, max=0 → cannot decay
        let t = Arc::new(ItemTypeData::default());
        let item = Item::new(t, 1);
        assert!(!item.can_decay());
    }

    #[test]
    fn test_can_decay_true_with_decay_time_and_decay_to() {
        let t = Arc::new(ItemTypeData {
            decay_time_min: 60,
            decay_time_max: 120,
            decay_to: 100,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        assert!(item.can_decay());
    }

    #[test]
    fn test_can_decay_false_when_unique_id_set() {
        let t = Arc::new(ItemTypeData {
            decay_time_min: 60,
            decay_time_max: 120,
            decay_to: 100,
            ..Default::default()
        });
        let mut item = Item::new(t, 1);
        item.set_attribute(ItemAttribute::UniqueId, AttributeValue::Integer(42));
        assert!(!item.can_decay());
    }

    // -----------------------------------------------------------------------
    // get_weapon_type / get_slot_position / is_store_item delegates
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_weapon_type_delegates_to_type() {
        use forgottenserver_common::constants::WeaponType;
        let t = Arc::new(ItemTypeData {
            weapon_type: WeaponType::Sword,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        assert_eq!(item.get_weapon_type(), WeaponType::Sword);
    }

    #[test]
    fn test_get_slot_position_delegates_to_type() {
        let t = Arc::new(ItemTypeData {
            slot_position: 0x1234,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        assert_eq!(item.get_slot_position(), 0x1234);
    }

    #[test]
    fn test_is_store_item_delegates_true() {
        let t = Arc::new(ItemTypeData {
            store_item: true,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        assert!(item.is_store_item());
    }

    #[test]
    fn test_is_store_item_delegates_false() {
        let t = Arc::new(ItemTypeData {
            store_item: false,
            ..Default::default()
        });
        let item = Item::new(t, 1);
        assert!(!item.is_store_item());
    }

    // -----------------------------------------------------------------------
    // transform branch: new_duration_max > 0 picks max, not min
    // -----------------------------------------------------------------------

    #[test]
    fn test_transform_with_decay_max_picks_max_value() {
        let t1 = Arc::new(ItemTypeData::default());
        let t2 = Arc::new(ItemTypeData {
            decay_time_min: 60,
            decay_time_max: 999,
            decay_to: 1,
            ..Default::default()
        });
        let mut item = Item::new(t1, 1);
        item.transform(t2);
        // When max > 0, duration is set to max (not min)
        assert_eq!(item.get_duration(), 999);
        assert_eq!(item.get_decay_state(), ItemDecayState::False);
    }

    // -----------------------------------------------------------------------
    // unserialize_attr: error path when buffer ends without terminator
    // -----------------------------------------------------------------------

    #[test]
    fn test_unserialize_empty_buffer_returns_error() {
        // No bytes at all → expects 0x00 terminator → error
        let data: Vec<u8> = Vec::new();
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("unexpected end"), "msg: {}", msg);
    }

    #[test]
    fn test_unserialize_truncated_u16_returns_error() {
        // ACTION_ID tag (0x04) followed by only 1 byte (not 2)
        let data: Vec<u8> = vec![0x04, 0x01];
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_unserialize_truncated_u32_returns_error() {
        // DURATION tag (0x10) followed by only 2 bytes (not 4)
        let data: Vec<u8> = vec![0x10, 0x01, 0x00];
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_unserialize_truncated_string_length_returns_error() {
        // TEXT tag (0x06) followed by length prefix but no body
        let data: Vec<u8> = vec![0x06, 0x05, 0x00]; // claims 5 bytes, but none present
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_unserialize_invalid_utf8_string_returns_error() {
        // TEXT tag (0x06), length 2, then bytes that are not valid UTF-8 (0xFF, 0xFE)
        let data: Vec<u8> = vec![0x06, 0x02, 0x00, 0xFF, 0xFE, 0x00];
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("invalid UTF-8"), "msg: {}", msg);
    }

    // -----------------------------------------------------------------------
    // unserialize_attr: UniqueId tag (0x05) is restored
    // -----------------------------------------------------------------------

    #[test]
    fn test_unserialize_unique_id_tag_sets_attribute() {
        // TAG_UNIQUE_ID (0x05) + u16(1000) + terminator
        let data: Vec<u8> = vec![0x05, 0xE8, 0x03, 0x00];
        let mut item = make_moveable_item();
        item.unserialize_attr(&data).expect("ok");
        assert_eq!(item.get_unique_id(), 1000);
    }

    // -----------------------------------------------------------------------
    // unserialize_attr: DECAYING_STATE tag with value 0 stays False
    // -----------------------------------------------------------------------

    #[test]
    fn test_unserialize_decay_state_zero_stays_false() {
        // TAG_DECAYING_STATE (0x11) + 0 + terminator
        let data: Vec<u8> = vec![0x11, 0x00, 0x00];
        let mut item = make_moveable_item();
        // Pre-set state to True so we can verify it gets reset to False on load
        item.set_decaying(ItemDecayState::True);
        item.unserialize_attr(&data).expect("ok");
        assert_eq!(item.get_decay_state(), ItemDecayState::False);
    }

    // -----------------------------------------------------------------------
    // unserialize_attr: COUNT tag with no following byte → read_u8 error path
    // -----------------------------------------------------------------------

    #[test]
    fn test_unserialize_count_tag_truncated_triggers_read_u8_eof() {
        // TAG_COUNT (0x0F) but no u8 payload → read_u8 hits eof
        let data: Vec<u8> = vec![0x0F];
        let mut item = make_moveable_item();
        let result = item.unserialize_attr(&data);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("read_u8") || msg.contains("unexpected end"),
            "msg: {}",
            msg
        );
    }

    // ── Parity-bug fixes (previously documented in intentional_differences.yml) ─

    /// C++ `Item::setDate(int32_t n)` accepts a signed 32-bit value (epoch
    /// seconds). Rust mirrors that signature so negative timestamps round-trip
    /// without overflowing the unsigned u32 → i64 cast that the previous u32
    /// signature forced.
    #[test]
    fn set_date_accepts_signed_i32() {
        let t = Arc::new(ItemTypeData::default());
        let mut item = Item::new(t, 1);
        item.set_date(-1234i32);
        let v = match item.get_attribute(ItemAttribute::Date) {
            Some(AttributeValue::Integer(n)) => *n,
            _ => panic!("Date attribute should be set"),
        };
        assert_eq!(v, -1234);
    }

    /// C++ `Item::getShootRange()` falls back to `items[id].shootRange`
    /// when no override attribute is set. The Rust port previously fell back
    /// to `attack_speed` (a different ItemType field), making distance
    /// weapons report nonsense range values.
    #[test]
    fn get_shoot_range_fallback_uses_item_type_shoot_range() {
        let t = Arc::new(ItemTypeData {
            shoot_range: 7,
            attack_speed: 200, // intentionally different — proves we don't fall back here
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 1);
        assert_eq!(item.get_shoot_range(), 7);
    }

    /// C++ `Item::isCleanable()` is gated on `!loadedFromMap` — items placed
    /// directly by the map loader must NOT be auto-cleaned by the world's
    /// periodic ground sweep. The Rust port previously omitted this gate,
    /// so map decorations could be deleted on world tick.
    #[test]
    fn is_cleanable_returns_false_when_loaded_from_map() {
        let t = Arc::new(ItemTypeData {
            pickupable: true, // is_pickupable() checks pickupable || allow_pickupable
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        assert!(item.is_cleanable(), "non-map item should be cleanable");
        item.set_loaded_from_map(true);
        assert!(
            !item.is_cleanable(),
            "map-loaded item must be exempt from periodic cleanup"
        );
    }

    // ── item.cpp ledger-gap closure tests ─────────────────────────────────

    /// C++ `Item::equals` — same id, same attribute map.
    #[test]
    fn equals_true_when_id_and_attributes_match() {
        let t = Arc::new(ItemTypeData {
            id: 100,
            ..ItemTypeData::default()
        });
        let mut a = Item::new(t.clone(), 5);
        let mut b = Item::new(t, 5);
        // set_action_id clamps `n < 100 → 100`, so use values that won't
        // collapse to the same stored attribute.
        a.set_action_id(200);
        b.set_action_id(200);
        assert!(a.equals(&b));
    }

    #[test]
    fn equals_false_when_attribute_differs() {
        let t = Arc::new(ItemTypeData {
            id: 100,
            ..ItemTypeData::default()
        });
        let mut a = Item::new(t.clone(), 5);
        let mut b = Item::new(t, 5);
        a.set_action_id(200);
        b.set_action_id(300);
        assert!(!a.equals(&b));
    }

    #[test]
    fn equals_false_when_different_ids() {
        let t1 = Arc::new(ItemTypeData {
            id: 100,
            ..ItemTypeData::default()
        });
        let t2 = Arc::new(ItemTypeData {
            id: 200,
            ..ItemTypeData::default()
        });
        let a = Item::new(t1, 1);
        let b = Item::new(t2, 1);
        assert!(!a.equals(&b));
    }

    /// C++ `Item::getSubType` — stackable items report count.
    #[test]
    fn get_sub_type_stackable_returns_count() {
        let t = Arc::new(ItemTypeData {
            stackable: true,
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 42);
        assert_eq!(item.get_sub_type(), 42);
    }

    #[test]
    fn get_sub_type_fluid_container_returns_fluid_type() {
        // Splash and FluidContainer item-groups make `is_fluid_container()`
        // (or `is_splash()`) return true.
        let t = Arc::new(ItemTypeData {
            group: forgottenserver_common::itemloader::ItemGroup::Splash,
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        item.set_fluid_type(7);
        assert_eq!(item.get_sub_type(), 7);
    }

    /// C++ `Item::hasProperty(CONST_PROP_BLOCKSOLID)` dispatches to
    /// `it.blockSolid`.
    #[test]
    fn has_property_block_solid_reads_item_type_flag() {
        let blocking = Arc::new(ItemTypeData {
            block_solid: true,
            ..ItemTypeData::default()
        });
        let item = Item::new(blocking, 1);
        assert!(item.has_property(ItemProperty::BlockSolid));

        let walkable = Arc::new(ItemTypeData::default());
        let item = Item::new(walkable, 1);
        assert!(!item.has_property(ItemProperty::BlockSolid));
    }

    /// C++ `CONST_PROP_MOVEABLE` is false when the item has a UniqueId.
    #[test]
    fn has_property_moveable_false_with_unique_id() {
        let t = Arc::new(ItemTypeData {
            moveable: true,
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        assert!(item.has_property(ItemProperty::Moveable));
        item.set_unique_id(7);
        assert!(!item.has_property(ItemProperty::Moveable));
    }

    /// C++ `Item::getWorth() { return items[id].worth * count; }`.
    #[test]
    fn get_worth_multiplies_type_worth_by_count() {
        let gold = Arc::new(ItemTypeData {
            worth: 1,
            ..ItemTypeData::default()
        });
        assert_eq!(Item::new(gold.clone(), 50).get_worth(), 50);
        let crystal = Arc::new(ItemTypeData {
            worth: 10_000,
            ..ItemTypeData::default()
        });
        assert_eq!(Item::new(crystal, 7).get_worth(), 70_000);
    }

    /// C++ `Item::getLightInfo` returns `{lightLevel, lightColor}`
    /// straight from the item type.
    #[test]
    fn get_light_info_returns_item_type_fields() {
        let t = Arc::new(ItemTypeData {
            light_level: 7,
            light_color: 215,
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 1);
        let info = item.get_light_info();
        assert_eq!(info.level, 7);
        assert_eq!(info.color, 215);
    }

    /// C++ `Item::isPushable() const override final { return isMoveable(); }`.
    #[test]
    fn is_pushable_aliases_is_moveable() {
        let moveable = Arc::new(ItemTypeData {
            moveable: true,
            ..ItemTypeData::default()
        });
        assert!(Item::new(moveable, 1).is_pushable());
        let immovable = Arc::new(ItemTypeData {
            moveable: false,
            ..ItemTypeData::default()
        });
        assert!(!Item::new(immovable, 1).is_pushable());
    }

    /// C++ `Item::getThrowRange() { return (isPickupable() ? 15 : 2); }`.
    #[test]
    fn get_throw_range_pickupable_15_else_2() {
        let pickable = Arc::new(ItemTypeData {
            pickupable: true,
            ..ItemTypeData::default()
        });
        assert_eq!(Item::new(pickable, 1).get_throw_range(), 15);
        let prop = Arc::new(ItemTypeData::default());
        assert_eq!(Item::new(prop, 1).get_throw_range(), 2);
    }

    /// C++ `Item::getAmmoType() { return items[id].ammoType; }`.
    #[test]
    fn get_ammo_type_returns_item_type_field() {
        use forgottenserver_common::constants::Ammo;
        let arrow = Arc::new(ItemTypeData {
            ammo_type: Ammo::Arrow,
            ..ItemTypeData::default()
        });
        assert_eq!(Item::new(arrow, 1).get_ammo_type(), Ammo::Arrow);
    }

    /// C++ `Item::getDecayTimeMin/Max` — attribute override beats type
    /// default; absent attribute falls back to type fields.
    #[test]
    fn get_decay_time_min_uses_attribute_or_type_field() {
        let t = Arc::new(ItemTypeData {
            decay_time_min: 100,
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        assert_eq!(item.get_decay_time_min(), 100);
        item.set_attribute(ItemAttribute::Duration, AttributeValue::Integer(250));
        assert_eq!(item.get_decay_time_min(), 250);
    }

    #[test]
    fn get_decay_time_max_uses_attribute_or_type_field() {
        let t = Arc::new(ItemTypeData {
            decay_time_max: 200,
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        assert_eq!(item.get_decay_time_max(), 200);
        item.set_attribute(ItemAttribute::DurationMax, AttributeValue::Integer(500));
        assert_eq!(item.get_decay_time_max(), 500);
    }

    /// C++ `Item::getDefaultDuration{Min,Max}` convert seconds→milliseconds.
    #[test]
    fn get_default_duration_converts_seconds_to_ms() {
        let t = Arc::new(ItemTypeData {
            decay_time_min: 10,
            decay_time_max: 30,
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 1);
        assert_eq!(item.get_default_duration_min(), 10_000);
        assert_eq!(item.get_default_duration_max(), 30_000);
    }

    /// `set_default_duration` samples within [min, max] inclusive and
    /// stores Duration. Min=Max=0 is a no-op (matches C++).
    #[test]
    fn set_default_duration_samples_within_range() {
        let t = Arc::new(ItemTypeData {
            decay_time_min: 5,
            decay_time_max: 5,
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        item.set_default_duration();
        // min == max == 5s → 5000ms exactly.
        assert_eq!(item.get_duration(), 5_000);
    }

    #[test]
    fn set_default_duration_no_op_when_both_zero() {
        let t = Arc::new(ItemTypeData::default());
        let mut item = Item::new(t, 1);
        item.set_default_duration();
        assert!(item.get_attribute(ItemAttribute::Duration).is_none());
    }

    /// `has_market_attributes` true on a fresh item, false once any
    /// non-charge/duration attribute is set.
    #[test]
    fn has_market_attributes_true_on_fresh_item() {
        let t = Arc::new(ItemTypeData::default());
        let item = Item::new(t, 1);
        assert!(item.has_market_attributes());
    }

    #[test]
    fn has_market_attributes_false_when_arbitrary_attribute_set() {
        let t = Arc::new(ItemTypeData::default());
        let mut item = Item::new(t, 1);
        item.set_action_id(7);
        assert!(!item.has_market_attributes());
    }

    /// `name_description_for_type` with article — non-stackable case.
    #[test]
    fn name_description_for_type_non_stackable_uses_article() {
        let t = ItemTypeData {
            id: 1,
            name: "sword".into(),
            article: "a".into(),
            ..ItemTypeData::default()
        };
        assert_eq!(Item::name_description_for_type(&t, 1, true), "a sword");
        assert_eq!(Item::name_description_for_type(&t, 1, false), "sword");
    }

    /// Stackable with subType > 1 + showCount → plural + count.
    #[test]
    fn name_description_for_type_stackable_count_and_plural() {
        let t = ItemTypeData {
            id: 1,
            name: "gold coin".into(),
            plural_name: "gold coins".into(),
            stackable: true,
            show_count: true,
            ..ItemTypeData::default()
        };
        assert_eq!(
            Item::name_description_for_type(&t, 17, true),
            "17 gold coins"
        );
    }

    // ── Reflect / BoostPercent API (Session 18) ─────────────────────────

    use crate::items_registry::Abilities;
    use forgottenserver_common::enums::CombatTypeFlags;

    /// `set_reflect` then `get_reflect(total=false)` round-trips.
    #[test]
    fn set_reflect_round_trips_via_get_with_total_false() {
        let t = Arc::new(ItemTypeData::default());
        let mut item = Item::new(t, 1);
        let r = Reflect {
            percent: 20,
            chance: 30,
        };
        item.set_reflect(CombatTypeFlags::FIRE, r);
        assert_eq!(item.get_reflect(CombatTypeFlags::FIRE, false), r);
    }

    /// `get_reflect(total=true)` accumulates per-instance + item-type
    /// reflect; chance is clamped to 100.
    #[test]
    fn get_reflect_total_combines_instance_and_type_with_chance_clamp() {
        let mut abilities = Abilities::default();
        let fire_idx = combat_type_to_index(CombatTypeFlags::FIRE);
        abilities.reflect[fire_idx] = Reflect {
            percent: 5,
            chance: 70,
        };
        let t = Arc::new(ItemTypeData {
            abilities: Some(Box::new(abilities)),
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        item.set_reflect(
            CombatTypeFlags::FIRE,
            Reflect {
                percent: 7,
                chance: 60,
            },
        );
        let total = item.get_reflect(CombatTypeFlags::FIRE, true);
        // percent = 5 + 7 = 12
        // chance  = min(70 + 60, 100) = 100
        assert_eq!(
            total,
            Reflect {
                percent: 12,
                chance: 100
            }
        );
    }

    /// Default-zero `set_reflect` clears the override (so `has_market_attributes`
    /// can find a clean item again).
    #[test]
    fn set_reflect_default_clears_override() {
        let t = Arc::new(ItemTypeData::default());
        let mut item = Item::new(t, 1);
        item.set_reflect(
            CombatTypeFlags::ICE,
            Reflect {
                percent: 1,
                chance: 1,
            },
        );
        assert!(!item.has_market_attributes());
        item.set_reflect(CombatTypeFlags::ICE, Reflect::default());
        assert!(item.has_market_attributes());
    }

    /// `set_boost_percent` round-trip; default-zero clears.
    #[test]
    fn set_boost_percent_round_trips_and_zero_clears() {
        let t = Arc::new(ItemTypeData::default());
        let mut item = Item::new(t, 1);
        item.set_boost_percent(CombatTypeFlags::ENERGY, 15);
        assert_eq!(item.get_boost_percent(CombatTypeFlags::ENERGY, false), 15);
        item.set_boost_percent(CombatTypeFlags::ENERGY, 0);
        assert_eq!(item.get_boost_percent(CombatTypeFlags::ENERGY, false), 0);
    }

    /// `get_boost_percent(total=true)` adds instance override + item-type
    /// blueprint boost_percent.
    #[test]
    fn get_boost_percent_total_sums_instance_and_type() {
        let mut abilities = Abilities::default();
        let idx = combat_type_to_index(CombatTypeFlags::EARTH);
        abilities.boost_percent[idx] = 10;
        let t = Arc::new(ItemTypeData {
            abilities: Some(Box::new(abilities)),
            ..ItemTypeData::default()
        });
        let mut item = Item::new(t, 1);
        item.set_boost_percent(CombatTypeFlags::EARTH, 25);
        assert_eq!(item.get_boost_percent(CombatTypeFlags::EARTH, true), 35);
        // total=false → only the override.
        assert_eq!(item.get_boost_percent(CombatTypeFlags::EARTH, false), 25);
    }

    /// `has_market_attributes` is false when any reflect or boost
    /// override is set — matching the C++ reflect/boost loop.
    #[test]
    fn has_market_attributes_false_with_reflect_or_boost_override() {
        let t = Arc::new(ItemTypeData::default());
        let mut item = Item::new(t.clone(), 1);
        assert!(item.has_market_attributes());
        item.set_boost_percent(CombatTypeFlags::PHYSICAL, 1);
        assert!(!item.has_market_attributes());
        item.set_boost_percent(CombatTypeFlags::PHYSICAL, 0);
        assert!(item.has_market_attributes());
        item.set_reflect(
            CombatTypeFlags::PHYSICAL,
            Reflect {
                percent: 1,
                chance: 0,
            },
        );
        assert!(!item.has_market_attributes());
    }

    /// Reflect `+=` parity: percent accumulates, chance clamps at 100.
    #[test]
    fn reflect_add_assign_clamps_chance_at_100() {
        let mut a = Reflect {
            percent: 10,
            chance: 60,
        };
        a += Reflect {
            percent: 5,
            chance: 70,
        };
        assert_eq!(
            a,
            Reflect {
                percent: 15,
                chance: 100
            }
        );
    }

    // ── getWeightDescription parity (Session 19) ────────────────────────

    /// weight < 10 → "0.0X" form, singular prefix.
    #[test]
    fn weight_description_under_10_pads_with_zero() {
        let t = ItemTypeData::default();
        assert_eq!(
            Item::weight_description_for_type(&t, 7, 1),
            "It weighs 0.07 oz."
        );
    }

    /// 10 ≤ weight < 100 → "0.XX" form.
    #[test]
    fn weight_description_under_100_no_pad() {
        let t = ItemTypeData::default();
        assert_eq!(
            Item::weight_description_for_type(&t, 42, 1),
            "It weighs 0.42 oz."
        );
    }

    /// weight ≥ 100 → "XX.XX" form (insert dot two-from-right).
    #[test]
    fn weight_description_above_100_inserts_dot() {
        let t = ItemTypeData::default();
        // 1234 → "12.34"
        assert_eq!(
            Item::weight_description_for_type(&t, 1234, 1),
            "It weighs 12.34 oz."
        );
        // 100 → "1.00" (boundary case)
        assert_eq!(
            Item::weight_description_for_type(&t, 100, 1),
            "It weighs 1.00 oz."
        );
    }

    /// Plural "They weigh" requires stackable + count > 1 + show_count.
    #[test]
    fn weight_description_uses_plural_only_when_all_three_conditions_hold() {
        let plural_t = ItemTypeData {
            stackable: true,
            show_count: true,
            ..ItemTypeData::default()
        };
        assert_eq!(
            Item::weight_description_for_type(&plural_t, 1000, 5),
            "They weigh 10.00 oz."
        );

        // show_count=false → still singular even with stackable + count > 1
        let no_show = ItemTypeData {
            stackable: true,
            show_count: false,
            ..ItemTypeData::default()
        };
        assert_eq!(
            Item::weight_description_for_type(&no_show, 1000, 5),
            "It weighs 10.00 oz."
        );

        // count == 1 → singular
        assert_eq!(
            Item::weight_description_for_type(&plural_t, 1000, 1),
            "It weighs 10.00 oz."
        );
    }

    /// Parameter-less instance overload: returns empty when computed weight is zero.
    #[test]
    fn weight_description_empty_when_weight_zero() {
        let t = Arc::new(ItemTypeData::default());
        // Default ItemTypeData has weight=0 → get_weight returns 0.
        let item = Item::new(t, 1);
        assert_eq!(item.weight_description(), "");
    }

    /// Parameter-less instance overload delegates to the with-weight form.
    #[test]
    fn weight_description_delegates_to_static_via_get_weight() {
        let t = Arc::new(ItemTypeData {
            weight: 50,
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 1);
        assert_eq!(item.weight_description(), "It weighs 0.50 oz.");
    }

    // ── countByType parity (Session 19) ─────────────────────────────────

    /// sub_type == -1 → returns the raw count.
    #[test]
    fn count_by_type_sentinel_returns_count() {
        let t = Arc::new(ItemTypeData {
            stackable: true,
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 42);
        assert_eq!(item.count_by_type(-1), 42);
    }

    /// matching sub-type returns the count.
    #[test]
    fn count_by_type_matching_returns_count() {
        let t = Arc::new(ItemTypeData {
            stackable: true,
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 7);
        // stackable items report `count` as their sub-type.
        assert_eq!(item.count_by_type(7), 7);
    }

    /// non-matching sub-type returns 0.
    #[test]
    fn count_by_type_non_matching_returns_zero() {
        let t = Arc::new(ItemTypeData {
            stackable: true,
            ..ItemTypeData::default()
        });
        let item = Item::new(t, 7);
        assert_eq!(item.count_by_type(3), 0);
    }

    // ── customAttr API (Session 20) ─────────────────────────────────────

    fn fresh_item() -> Item {
        Item::new(Arc::new(ItemTypeData::default()), 1)
    }

    /// String set + get round-trips and the case-insensitive lookup
    /// matches C++ `to_lower_copy` normalisation.
    #[test]
    fn custom_attribute_string_round_trips_with_lowercase_key() {
        let mut item = fresh_item();
        item.set_custom_attribute("MyKey", CustomAttribute::String("hello".into()));
        assert_eq!(
            item.get_custom_attribute("mykey"),
            Some(&CustomAttribute::String("hello".into())),
            "lookup must be case-insensitive"
        );
        assert_eq!(
            item.get_custom_attribute("MYKEY"),
            Some(&CustomAttribute::String("hello".into()))
        );
    }

    /// All four variant types round-trip via the typed enum.
    #[test]
    fn custom_attribute_typed_variants_round_trip() {
        let mut item = fresh_item();
        item.set_custom_attribute("s", CustomAttribute::String("x".into()));
        item.set_custom_attribute("i", CustomAttribute::Integer(-42));
        item.set_custom_attribute("f", CustomAttribute::Float(3.5));
        item.set_custom_attribute("b", CustomAttribute::Boolean(true));
        assert_eq!(
            item.get_custom_attribute("s"),
            Some(&CustomAttribute::String("x".into()))
        );
        assert_eq!(
            item.get_custom_attribute("i"),
            Some(&CustomAttribute::Integer(-42))
        );
        assert_eq!(
            item.get_custom_attribute("f"),
            Some(&CustomAttribute::Float(3.5))
        );
        assert_eq!(
            item.get_custom_attribute("b"),
            Some(&CustomAttribute::Boolean(true))
        );
    }

    /// Int-keyed overload stringifies the key (C++ `to_string(int64_t)`
    /// happens before the string-keyed setter runs).
    #[test]
    fn custom_attribute_int_key_stringifies_for_lookup() {
        let mut item = fresh_item();
        item.set_custom_attribute_int_key(7, CustomAttribute::String("seven".into()));
        // Lookup via string "7" finds the value (stringification path).
        assert_eq!(
            item.get_custom_attribute("7"),
            Some(&CustomAttribute::String("seven".into()))
        );
        // Lookup via int-keyed getter also works.
        assert_eq!(
            item.get_custom_attribute_int_key(7),
            Some(&CustomAttribute::String("seven".into()))
        );
    }

    /// `remove_custom_attribute` reports success/failure correctly.
    #[test]
    fn remove_custom_attribute_returns_true_when_present_false_otherwise() {
        let mut item = fresh_item();
        item.set_custom_attribute("present", CustomAttribute::Integer(1));
        assert!(item.remove_custom_attribute("present"));
        assert!(!item.remove_custom_attribute("present"));
        assert!(!item.remove_custom_attribute("never-was-there"));
    }

    /// `get_custom_attribute_map()` is `None` until something is set,
    /// and again `None` after every entry is removed — matches C++ which
    /// frees the underlying map when it empties.
    #[test]
    fn custom_attribute_map_is_none_before_first_set_and_after_last_remove() {
        let mut item = fresh_item();
        assert!(item.get_custom_attribute_map().is_none());
        item.set_custom_attribute("only", CustomAttribute::Boolean(false));
        assert!(item.get_custom_attribute_map().is_some());
        assert!(item.remove_custom_attribute("only"));
        assert!(item.get_custom_attribute_map().is_none());
    }

    /// Re-`set` with the same key replaces the value (no duplicate entry).
    #[test]
    fn custom_attribute_reset_replaces_value() {
        let mut item = fresh_item();
        item.set_custom_attribute("k", CustomAttribute::Integer(1));
        item.set_custom_attribute("k", CustomAttribute::String("now-a-string".into()));
        assert_eq!(
            item.get_custom_attribute("k"),
            Some(&CustomAttribute::String("now-a-string".into()))
        );
        assert_eq!(item.get_custom_attribute_map().unwrap().len(), 1);
    }

    // -----------------------------------------------------------------------
    // get_plural_name / get_article
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_plural_name_falls_back_to_type() {
        let it = ItemTypeData { id: 1, plural_name: "swords".to_string(), ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert_eq!(item.get_plural_name(), "swords");
    }

    #[test]
    fn test_get_plural_name_auto_appends_s() {
        let it = ItemTypeData {
            id: 1,
            name: "sword".to_string(),
            show_count: true,
            ..Default::default()
        };
        // plural_name is empty → item_type.get_plural_name() returns "swords"
        let item = Item::new(Arc::new(it), 1);
        assert_eq!(item.get_plural_name(), "swords");
    }

    #[test]
    fn test_get_plural_name_attribute_override_wins() {
        let it = ItemTypeData { id: 1, plural_name: "swords".to_string(), ..Default::default() };
        let mut item = Item::new(Arc::new(it), 1);
        item.set_attribute(
            ItemAttribute::PluralName,
            AttributeValue::String("custom plural".to_string()),
        );
        assert_eq!(item.get_plural_name(), "custom plural");
    }

    #[test]
    fn test_get_article_falls_back_to_type() {
        let it = ItemTypeData { id: 1, article: "a".to_string(), ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert_eq!(item.get_article(), "a");
    }

    #[test]
    fn test_get_article_attribute_override_wins() {
        let it = ItemTypeData { id: 1, article: "a".to_string(), ..Default::default() };
        let mut item = Item::new(Arc::new(it), 1);
        item.set_attribute(
            ItemAttribute::Article,
            AttributeValue::String("the".to_string()),
        );
        assert_eq!(item.get_article(), "the");
    }

    // -----------------------------------------------------------------------
    // set_store_item
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_store_item_true() {
        let t = make_item_type(1, 0, false, false, false, "item", "an");
        let mut item = Item::new(t, 1);
        item.set_store_item(true);
        assert!(item.is_store_item());
    }

    #[test]
    fn test_set_store_item_false() {
        let t = make_item_type(1, 0, false, false, false, "item", "an");
        let mut item = Item::new(t, 1);
        item.set_store_item(true);
        item.set_store_item(false);
        assert!(!item.is_store_item());
    }

    // -----------------------------------------------------------------------
    // set_sub_type
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_sub_type_fluid_container() {
        let it = ItemTypeData { id: 1, group: ItemGroup::Fluid, ..Default::default() };
        let mut item = Item::new(Arc::new(it), 1);
        item.set_sub_type(7);
        assert_eq!(item.get_fluid_type(), 7);
    }

    #[test]
    fn test_set_sub_type_splash() {
        let it = ItemTypeData { id: 1, group: ItemGroup::Splash, ..Default::default() };
        let mut item = Item::new(Arc::new(it), 1);
        item.set_sub_type(3);
        assert_eq!(item.get_fluid_type(), 3);
    }

    #[test]
    fn test_set_sub_type_stackable() {
        let t = make_item_type(1, 0, true, true, true, "coin", "a");
        let mut item = Item::new(t, 1);
        item.set_sub_type(50);
        assert_eq!(item.get_item_count(), 50);
    }

    #[test]
    fn test_set_sub_type_charges() {
        let it = ItemTypeData { id: 1, charges: 10, ..Default::default() };
        let mut item = Item::new(Arc::new(it), 1);
        item.set_sub_type(5);
        assert_eq!(item.get_charges(), 5);
    }

    #[test]
    fn test_set_sub_type_default_path() {
        // Non-stackable, no charges, not fluid/splash → falls through to set_item_count
        let t = make_item_type(1, 0, false, true, true, "sword", "a");
        let mut item = Item::new(t, 1);
        item.set_sub_type(3);
        assert_eq!(item.get_item_count(), 3);
    }

    // -----------------------------------------------------------------------
    // set_default_subtype
    // -----------------------------------------------------------------------

    #[test]
    fn test_set_default_subtype_no_charges() {
        let it = ItemTypeData { id: 1, charges: 0, ..Default::default() };
        let mut item = Item::new(Arc::new(it), 5);
        item.set_default_subtype();
        assert_eq!(item.get_item_count(), 1);
        assert_eq!(item.get_charges(), 0);
    }

    #[test]
    fn test_set_default_subtype_stackable_with_charges() {
        let it = ItemTypeData { id: 1, stackable: true, charges: 20, ..Default::default() };
        let mut item = Item::new(Arc::new(it), 1);
        item.set_default_subtype();
        assert_eq!(item.get_item_count(), 20);
    }

    #[test]
    fn test_set_default_subtype_non_stackable_with_charges() {
        let it = ItemTypeData { id: 1, stackable: false, charges: 30, ..Default::default() };
        let mut item = Item::new(Arc::new(it), 1);
        item.set_default_subtype();
        assert_eq!(item.get_charges(), 30);
        assert_eq!(item.get_item_count(), 1);
    }

    // -----------------------------------------------------------------------
    // can_transform
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_transform_always_true() {
        let t = make_item_type(1, 0, false, false, false, "item", "an");
        let item = Item::new(t, 1);
        assert!(item.can_transform());
    }

    // -----------------------------------------------------------------------
    // Type-delegate predicates
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_ground_tile_true() {
        let it = ItemTypeData { id: 1, group: ItemGroup::Ground, ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert!(item.is_ground_tile());
    }

    #[test]
    fn test_is_ground_tile_false() {
        let t = make_item_type(1, 0, false, false, false, "sword", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_ground_tile());
    }

    #[test]
    fn test_is_magic_field_true() {
        let it = ItemTypeData { id: 1, type_kind: ItemTypeKind::MagicField, ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert!(item.is_magic_field());
    }

    #[test]
    fn test_is_magic_field_false() {
        let t = make_item_type(1, 0, false, false, false, "sword", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_magic_field());
    }

    #[test]
    fn test_is_podium_true() {
        let it = ItemTypeData { id: 1, type_kind: ItemTypeKind::Podium, ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert!(item.is_podium());
    }

    #[test]
    fn test_is_podium_false() {
        let t = make_item_type(1, 0, false, false, false, "sword", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_podium());
    }

    #[test]
    fn test_has_walk_stack_true() {
        let it = ItemTypeData { id: 1, walk_stack: true, ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert!(item.has_walk_stack());
    }

    #[test]
    fn test_has_walk_stack_false() {
        let it = ItemTypeData { id: 1, walk_stack: false, ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert!(!item.has_walk_stack());
    }

    #[test]
    fn test_is_supply_true() {
        let it = ItemTypeData { id: 1, supply: true, ..Default::default() };
        let item = Item::new(Arc::new(it), 1);
        assert!(item.is_supply());
    }

    #[test]
    fn test_is_supply_false() {
        let t = make_item_type(1, 0, false, false, false, "sword", "a");
        let item = Item::new(t, 1);
        assert!(!item.is_supply());
    }
}
