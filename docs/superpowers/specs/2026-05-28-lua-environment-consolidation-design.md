# Design: Single Lua Environment Consolidation

**Date:** 2026-05-28  
**Status:** Approved

## Problem

Two separate `mlua::Lua` VMs exist at runtime and are never connected:

- `LuaBindingsState` — owns a VM with TFS globals installed (Position, enums, Game), but never loads any scripts.
- `LuaScriptEngine` (in `server::boot::load_lua_scripts`) — loads all scripts, but runs on a bare VM with zero TFS globals. Every script that calls `Spell()`, `Combat()`, `TalkAction()`, etc. fails immediately.

C++ uses exactly one environment (`g_luaEnvironment`) for both binding registration and script execution. The Rust port must do the same.

## Goal

Consolidate to a single `LuaEnvironment` that owns the VM, installs all TFS bindings, and loads all scripts — in the correct order.

## Architecture

```
tfs::boot::Modules
  └── lua: LuaEnvironment          ← single VM, all bindings + all scripts
        └── mlua::Lua
              ├── install_bindings()    ← Position, enums, Game + all class globals
              ├── load_lib_scripts()    ← data/scripts/lib/ (loaded first)
              └── load_scripts()        ← data/scripts/**/*.lua (skip lib/, skip #)
```

**Boot order in `tfs::boot::initialise_modules`:**
1. Load config
2. `server::boot::boot(data_dir)` → `GameData` (items, weapons, npcs) — no scripts here
3. `GameState::new()`
4. `LuaEnvironment::new(game_state_handle)` — creates VM, installs all bindings
5. `lua.load_lib_scripts(data_dir/scripts/lib)` — libs first (mirrors C++ `isLib=true` pass)
6. `lua.load_scripts(data_dir/scripts)` — game scripts (skip `lib/`, skip `#`-prefixed, sorted)
7. Return `Modules` with `scripts_loaded: usize`

## Components

### `crates/scripting/src/lua_bindings/mod.rs`

- Rename `LuaBindingsState` → `LuaEnvironment`
- Add `load_lib_scripts(dir: &Path) -> Result<usize, String>` — fatal on dir error
- Add `load_scripts(dir: &Path) -> usize` — log-and-continue per file
- Extend `install_bindings` to register all class globals:
  `Spell`, `Combat`, `TalkAction`, `Action`, `Condition`, `CreatureEvent`,
  `GlobalEvent`, `MoveEvent`, `XMLDocument`, `configManager`
- Each class uses its existing file in `classes/`; any unimplemented method is a stub
  returning `nil` or a descriptive error string (no panics)

### `crates/scripting/src/engine.rs`

- `LuaScriptEngine` and `collect_lua_files` remain unchanged
- `LuaEnvironment::load_scripts` delegates to `collect_lua_files` for the recursive walk
- `LuaScriptEngine` is no longer on the production boot path; kept as test/utility type

### `crates/server/src/boot.rs`

- Remove `load_lua_scripts` function
- Remove `scripts_loaded` field from `GameData`

### `crates/tfs/src/boot.rs`

- `Modules.lua` field type: `Option<LuaEnvironment>` (was `Option<LuaBindingsState>`)
- Add `Modules.scripts_loaded: usize`
- After `LuaEnvironment::new()`: call `load_lib_scripts`, then `load_scripts`; store count

### `crates/tfs/src/main.rs`

- Read `modules.scripts_loaded` (was `modules.game_data.scripts_loaded`)

## Error Handling

**Per-script errors** (non-fatal) — log and continue:
```
> [error] data/scripts/spells/fireball.lua:11: attempt to call a nil value (global 'Combat')
```
Format: `[error] <path-relative-to-data_dir>:<line>: <reason>`

**Lib load errors** (fatal) — abort boot:
```
[FATAL] Failed to load Lua lib: data/scripts/lib/spells.lua:3: <reason>
```

**Binding install errors** (fatal) — name the binding that failed:
```
[FATAL] Failed to install Lua binding 'Spell': <mlua error>
```

**Summary line** after all scripts attempted:
```
>> Loaded 312 Lua scripts (47 errors — run with RUST_LOG=debug for details)
```
Per-error lines always print. Debug logging adds full stack traces.

**Feature-gate:** `LuaEnvironment` is `#[cfg(feature = "lua-scripting")]`. When disabled,
`Modules.lua` is `None` and `scripts_loaded` is `0`.

## Tests

All tests follow TDD: failing test written first, then implementation.

### `LuaEnvironment` unit tests

| Test | Assertion |
|---|---|
| `lua_environment_new_installs_position_global` | `Position` is callable after construction |
| `lua_environment_new_installs_enum_constants` | spot-check one enum constant is non-nil |
| `lua_environment_new_installs_class_globals` | each of `Spell`, `Combat`, `TalkAction`, `Action`, `Condition`, `CreatureEvent`, `GlobalEvent`, `MoveEvent`, `XMLDocument` is non-nil |
| `lua_environment_load_lib_scripts_loads_recursively` | temp dir `lib/helpers.lua` is loaded |
| `lua_environment_load_lib_scripts_missing_dir_is_fatal` | returns `Err` |
| `lua_environment_load_scripts_skips_lib_and_hash` | lib/ and #-prefixed skipped |
| `lua_environment_load_scripts_error_continues_and_counts` | one bad script doesn't abort; count reflects successes |
| `lua_environment_load_scripts_error_message_includes_path_and_line` | error string contains filename and line number |

### Unchanged tests

- All existing `LuaScriptEngine` tests in `engine.rs` — stay green, type not removed
- `boot_all_four_loaders_called_before_game_loop` in `server/src/boot.rs` — updated to not assert `scripts_loaded`

## Out of Scope

- Full implementation of class methods (Spell:register, Combat:execute, etc.) — stubs are acceptable
- `PacketHandler` global — not in `classes/`; deferred
- `Event` global — not in `classes/`; deferred
- Async Lua dispatch, scheduler integration
