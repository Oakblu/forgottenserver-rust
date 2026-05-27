//! Lua-binding enum-constant registration.
//!
//! Registers all C++ enum constants that `LuaScriptInterface::registerEnum`
//! exposes to Lua scripts. One submodule per binding family for review;
//! each submodule's `install(lua)` is mechanical (per-constant
//! `lua.globals().set(<NAME_LITERAL>, VALUE as i64)`).
//!
//! Sourced from existing Rust types in `forgottenserver_common`. C++→Rust
//! name mappings are inline next to each registration so future
//! contributors can find the source of any value.
//!
//! Files under this directory are **detected by `lua-static-audit`** as
//! `GlobalEnum` bindings (per the file-path heuristic in the parser).
//! Adding a constant here automatically shrinks the
//! `missing_in_rust` GlobalEnum count.
//!
//! ## Coverage map (audit baseline: 906 GlobalEnum C++ constants)
//!
//! ```text
//!   [x] account     (6 constants)   ACCOUNT_TYPE_*
//!   [x] direction   (8 constants)   DIRECTION_*
//!   [ ] ammo        (8)     ─┐
//!   [ ] skull       (7)      │  Follow-up changes:
//!   [ ] game_state  (7)      │  forgottenserver-rust-lua-bindings-
//!   [ ] weapon      (9)      │  enums-<family>
//!   [ ] weather     (5)      │
//!   [ ] return      (69)     │  Each family is mechanical; the engine
//!   [ ] condition   (84)     │  pattern is established here.
//!   [ ] item        (88)     │
//!   [ ] tilestate   (26)     │
//!   [ ] const       (218)    │  (Largest family — magic + animation
//!   [ ] talktype    (18)     │   effects; needs MagicEffectClass +
//!   [ ] message     (28)     │   ShootType mappings.)
//!   [ ] combat      (27)     │
//!   [ ] textcolor   (15)     │
//!   [ ] slot/p      (13)     │
//!   [ ] mapmark     (20)     │
//!   [ ] fluid       (20)     │
//!   [ ] reload      (18)     │
//!   [ ] report      (24)     │
//!   [ ] speechbubble (8)     │
//!   [ ] skill       (9)      │
//!   [ ] conditionid (12)     │
//!   [ ] creature    (39)     │
//!   [ ] flag        (8)      │
//!   [ ] origin      (6)      │
//!   [ ] clientos    (6)      │
//!   [ ] monster     (7)      │
//!   [ ] other       (~80)    ─┘
//! ```

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

pub mod account;
pub mod ammo;
pub mod callback_param;
pub mod client_os;
pub mod combat;
pub mod combat_param;
pub mod condition;
pub mod condition_id;
pub mod condition_param;
pub mod const_slot;
pub mod creature_event;
pub mod creature_icon;
pub mod creature_type;
pub mod direction;
pub mod distance_effect;
pub mod fight_mode;
pub mod flag;
pub mod fluid;
pub mod game_state;
pub mod item;
pub mod item_attribute;
pub mod item_group;
pub mod item_property;
pub mod magic_effect;
pub mod map_mark;
pub mod message;
pub mod misc;
pub mod monster_icon;
pub mod monsters_event;
pub mod origin;
pub mod player_flag;
pub mod player_sex;
pub mod podium;
pub mod reload;
pub mod report;
pub mod resource;
pub mod return_value;
pub mod skill;
pub mod skull;
pub mod slot_position;
pub mod special_skill;
pub mod speech_bubble;
pub mod spell_type;
pub mod stat;
pub mod talk_type;
pub mod text_color;
pub mod tile_state;
pub mod weapon;
pub mod wield_info;
pub mod world_type;
pub mod zone;

/// Register every global enum constant. Per-family follow-ups add a
/// new `<family>::install(lua)?;` line here when they land.
pub fn install_global_enums(lua: &mlua::Lua) -> mlua::Result<()> {
    account::install(lua)?;
    ammo::install(lua)?;
    callback_param::install(lua)?;
    client_os::install(lua)?;
    combat::install(lua)?;
    combat_param::install(lua)?;
    condition::install(lua)?;
    condition_id::install(lua)?;
    condition_param::install(lua)?;
    const_slot::install(lua)?;
    creature_event::install(lua)?;
    creature_icon::install(lua)?;
    creature_type::install(lua)?;
    direction::install(lua)?;
    distance_effect::install(lua)?;
    fight_mode::install(lua)?;
    flag::install(lua)?;
    fluid::install(lua)?;
    game_state::install(lua)?;
    item::install(lua)?;
    item_attribute::install(lua)?;
    item_group::install(lua)?;
    item_property::install(lua)?;
    magic_effect::install(lua)?;
    map_mark::install(lua)?;
    message::install(lua)?;
    misc::install(lua)?;
    monster_icon::install(lua)?;
    monsters_event::install(lua)?;
    origin::install(lua)?;
    player_flag::install(lua)?;
    player_sex::install(lua)?;
    podium::install(lua)?;
    reload::install(lua)?;
    report::install(lua)?;
    resource::install(lua)?;
    return_value::install(lua)?;
    skill::install(lua)?;
    skull::install(lua)?;
    slot_position::install(lua)?;
    special_skill::install(lua)?;
    speech_bubble::install(lua)?;
    spell_type::install(lua)?;
    stat::install(lua)?;
    talk_type::install(lua)?;
    text_color::install(lua)?;
    tile_state::install(lua)?;
    weapon::install(lua)?;
    wield_info::install(lua)?;
    world_type::install(lua)?;
    zone::install(lua)?;
    Ok(())
}
