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

    // ── createCombatArea(area) — accepts a 2-D pattern table, returns a stub ─
    // C++ builds a CombatArea from the matrix; here we return an empty table
    // so scripts that store the result and pass it to Combat:setArea still
    // get a non-nil value without requiring the full combat subsystem.
    let create_combat_area =
        lua.create_function(|lua, _area: mlua::MultiValue| lua.create_table())?;
    lua.globals().set("createCombatArea", create_combat_area)?;

    // ── isScriptsInterface() — returns true ─────────────────────────────────
    // TFS uses this to distinguish the scripts-interface path from the
    // legacy XML-event path.  This server always uses the scripts interface.
    let is_scripts_interface = lua.create_function(|_, _: ()| Ok(true))?;
    lua.globals().set("isScriptsInterface", is_scripts_interface)?;

    // ── PacketHandler(opcode) — returns a plain table ───────────────────────
    // C++ wires incoming packet opcodes to named Lua handler functions.
    // Scripts in data/scripts/network/ do:
    //   local handler = PacketHandler(0xE1)
    //   function handler.onReceive(player) ... end
    // A plain Lua table is returned so field assignment works naturally.
    // `register` and `clear` are no-op stubs matching packet_handler.lua.
    let packet_handler = lua.create_function(|lua, args: mlua::MultiValue| {
        let packet_type = match args.into_iter().next() {
            Some(mlua::Value::Integer(n)) => n,
            _ => 0,
        };
        let tbl = lua.create_table()?;
        tbl.set("packetType", packet_type)?;
        tbl.set("register", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
        tbl.set("clear", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
        Ok(tbl)
    })?;
    lua.globals().set("PacketHandler", packet_handler)?;

    // ── openLevelDoors / openQuestDoors — empty tables ──────────────────────
    // data/global.lua populates these tables at runtime; scripts in
    // data/scripts/movements/ iterate them with ipairs.  Register them as
    // empty tables so those scripts don't error when global.lua is absent.
    lua.globals().set("openLevelDoors", lua.create_table()?)?;
    lua.globals().set("openQuestDoors", lua.create_table()?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    fn fresh_lua() -> mlua::Lua {
        let lua = mlua::Lua::new();
        crate::lua_bindings::install_bindings(
            &lua,
            crate::lua_bindings::GameStateHandle::default(),
        )
        .unwrap();
        lua
    }

    #[test]
    fn create_combat_area_is_callable() {
        let lua = fresh_lua();
        let result = lua
            .load("local area = createCombatArea({{0,1,0},{1,1,1},{0,1,0}}); return area ~= nil")
            .eval::<bool>();
        assert!(result.is_ok(), "createCombatArea should not error: {result:?}");
        assert!(result.unwrap(), "createCombatArea should return non-nil");
    }

    #[test]
    fn is_scripts_interface_returns_true() {
        let lua = fresh_lua();
        let result = lua
            .load("return isScriptsInterface()")
            .eval::<bool>();
        assert!(result.is_ok(), "isScriptsInterface should not error: {result:?}");
        assert!(result.unwrap(), "isScriptsInterface should return true");
    }

    #[test]
    fn packet_handler_is_callable() {
        let lua = fresh_lua();
        let result = lua
            .load("PacketHandler(0x9F, function() end)")
            .exec();
        assert!(result.is_ok(), "PacketHandler should not error: {result:?}");
    }

    #[test]
    fn packet_handler_returns_non_nil() {
        let lua = fresh_lua();
        let result = lua
            .load("return PacketHandler(0xE1) ~= nil")
            .eval::<bool>();
        assert!(result.is_ok(), "PacketHandler(0xE1) should not error: {result:?}");
        assert!(result.unwrap(), "PacketHandler should return a non-nil value");
    }

    #[test]
    fn packet_handler_field_assignment_works() {
        let lua = fresh_lua();
        let result = lua
            .load("local h = PacketHandler(0xE1); function h.onReceive() end")
            .exec();
        assert!(result.is_ok(), "field assignment on PacketHandler result should not error: {result:?}");
    }

    #[test]
    fn open_level_doors_is_table() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"return type(openLevelDoors) == "table""#)
            .eval::<bool>();
        assert!(result.is_ok(), "openLevelDoors check should not error: {result:?}");
        assert!(result.unwrap(), "openLevelDoors should be a table");
    }

    #[test]
    fn open_quest_doors_is_table() {
        let lua = fresh_lua();
        let result = lua
            .load(r#"return type(openQuestDoors) == "table""#)
            .eval::<bool>();
        assert!(result.is_ok(), "openQuestDoors check should not error: {result:?}");
        assert!(result.unwrap(), "openQuestDoors should be a table");
    }
}
