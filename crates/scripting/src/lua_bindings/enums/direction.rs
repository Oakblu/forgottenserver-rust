//! `DIRECTION_*` enum constants.
//!
//! Source: `forgottenserver_common::position::Direction`.
//! C++: `enum Direction : uint8_t { DIRECTION_NORTH = 0, … };`
//!
//! `Direction::None = 8` is intentionally NOT registered — C++ does
//! not expose it via `registerEnum`. Lua scripts that need "no
//! direction" use `nil`.

#![cfg(feature = "lua-scripting")]

use forgottenserver_common::position::Direction;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    // Inline `lua.globals().set(...)` form so the harness static-audit
    // parser detects each binding as a GlobalEnum.
    lua.globals()
        .set("DIRECTION_NORTH", Direction::North as i64)?;
    lua.globals()
        .set("DIRECTION_EAST", Direction::East as i64)?;
    lua.globals()
        .set("DIRECTION_SOUTH", Direction::South as i64)?;
    lua.globals()
        .set("DIRECTION_WEST", Direction::West as i64)?;
    lua.globals()
        .set("DIRECTION_SOUTHWEST", Direction::Southwest as i64)?;
    lua.globals()
        .set("DIRECTION_SOUTHEAST", Direction::Southeast as i64)?;
    lua.globals()
        .set("DIRECTION_NORTHWEST", Direction::Northwest as i64)?;
    lua.globals()
        .set("DIRECTION_NORTHEAST", Direction::Northeast as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_north_registers_with_value_0() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return DIRECTION_NORTH").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn direction_northeast_registers_with_value_7() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return DIRECTION_NORTHEAST").eval().unwrap();
        assert_eq!(v, 7);
    }

    #[test]
    fn all_eight_directions_register_with_sequential_values() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let expected = [
            ("DIRECTION_NORTH", 0),
            ("DIRECTION_EAST", 1),
            ("DIRECTION_SOUTH", 2),
            ("DIRECTION_WEST", 3),
            ("DIRECTION_SOUTHWEST", 4),
            ("DIRECTION_SOUTHEAST", 5),
            ("DIRECTION_NORTHWEST", 6),
            ("DIRECTION_NORTHEAST", 7),
        ];
        for (name, val) in expected {
            let actual: i64 = lua
                .load(format!("return {name}"))
                .eval()
                .unwrap_or_else(|_| panic!("missing {name}"));
            assert_eq!(actual, val, "{name} should be {val}");
        }
    }
}
