//! Table-scoped Lua bindings (`<table>.<NAME>` constants).
//!
//! Each submodule installs one Lua table and populates it with
//! integer constants. The harness static-audit parser tags these
//! as `TableEnum` bindings (file-path heuristic: any
//! `lua_bindings/table_enums/<name>.rs` qualifies).
#![cfg(feature = "lua-scripting")]

pub mod config_keys;

pub fn install_table_enums(lua: &mlua::Lua) -> mlua::Result<()> {
    config_keys::install(lua)?;
    Ok(())
}
