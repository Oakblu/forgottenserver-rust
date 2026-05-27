//! Per-class Lua bindings.
//!
//! Each submodule wraps a Rust type in a `LuaX` newtype (orphan-rule
//! workaround for `mlua::UserData`) and registers its methods +
//! meta-methods via mlua. The harness static-audit parser strips the
//! `Lua` prefix when reporting binding names, so audit output matches
//! C++'s `ClassName:method` convention.
//!
//! Classes that need access to long-lived game state read the
//! `GameStateHandle` newtype from `lua.app_data_ref` (see the engine
//! change's design doc for the pattern).

#![cfg(feature = "lua-scripting")]

pub mod action;
pub mod combat;
pub mod condition;
pub mod container;
pub mod creature;
pub mod creature_event;
pub mod db_insert;
pub mod db_transaction;
pub mod game;
pub mod global_event;
pub mod group;
pub mod guild;
pub mod house;
pub mod item;
pub mod item_type;
pub mod loot;
pub mod modal_window;
pub mod monster;
pub mod monster_spell;
pub mod monster_type;
pub mod move_event;
pub mod network_message;
pub mod npc;
pub mod outfit;
pub mod party;
pub mod player;
pub mod podium;
pub mod spell;
pub mod talk_action;
pub mod teleport;
pub mod tile;
pub mod variant;
pub mod vocation;
pub mod weapon;
pub mod xml_document;
pub mod xml_node;
