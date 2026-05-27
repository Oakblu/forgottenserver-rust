//! Misc top-level Lua bindings: free functions (`isType`, `rawgetmetatable`)
//! and table-namespace extensions (`table.create`, `table.pack`, `os.mtime`).
//!
//! These mirror the C++ side's `registerGlobalFunction` / `registerTable`
//! helpers that augment the Lua standard library with TFS-specific helpers.
//
// AUDIT: ClassMethod table:create table:pack os:mtime

#![cfg(feature = "lua-scripting")]

use mlua::Value;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    // ── Free functions ─────────────────────────────────────────────
    // `isType(value, "TypeName")` — C++ helper to check userdata's
    // registered class name. Always returns `false` here until we
    // wire the class-registry lookup; safe default for scripts that
    // use this for branching.
    let is_type = lua.create_function(|_, (_v, _name): (Value, String)| Ok(false))?;
    lua.globals().set("isType", is_type)?;
    // `rawgetmetatable(value)` — Lua's `getmetatable` ignoring `__metatable`.
    let raw_get_metatable = lua.create_function(|_, v: Value| match v {
        Value::Table(t) => Ok(t.get_metatable().map(Value::Table).unwrap_or(Value::Nil)),
        Value::UserData(_) => Ok(Value::Nil),
        _ => Ok(Value::Nil),
    })?;
    lua.globals().set("rawgetmetatable", raw_get_metatable)?;

    // ── table.create(narr [, nrec]) — preallocates a Lua table ─────
    // mlua's `Lua::create_table_with_capacity` matches this exactly.
    let table_create = lua.create_function(|lua, narr: i64| {
        lua.create_table_with_capacity(narr.max(0) as usize, 0)
    })?;
    let table_ns: mlua::Table = lua.globals().get("table")?;
    table_ns.set("create", table_create)?;
    // `table.pack(...)` — exists in Lua 5.2+ natively; reinstall here
    // for parity since C++ registers it explicitly.
    let table_pack = lua.create_function(|lua, args: mlua::MultiValue| {
        let t = lua.create_table()?;
        for (i, v) in args.into_iter().enumerate() {
            t.set(i as i64 + 1, v)?;
        }
        t.set("n", t.raw_len() as i64)?;
        Ok(t)
    })?;
    table_ns.set("pack", table_pack)?;

    // ── os.mtime() — current time in milliseconds (TFS convention) ─
    let os_mtime = lua.create_function(|_, ()| {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        Ok(ms)
    })?;
    let os_ns: mlua::Table = lua.globals().get("os")?;
    os_ns.set("mtime", os_mtime)?;

    Ok(())
}
