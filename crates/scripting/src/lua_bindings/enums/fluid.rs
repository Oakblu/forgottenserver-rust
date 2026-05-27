//! `FLUID_*` enum constants. Source: `common::enums::FluidType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::FluidType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set("FLUID_NONE", FluidType::None as i64)?;
    lua.globals().set("FLUID_WATER", FluidType::Water as i64)?;
    lua.globals().set("FLUID_BLOOD", FluidType::Blood as i64)?;
    lua.globals().set("FLUID_BEER", FluidType::Beer as i64)?;
    lua.globals().set("FLUID_SLIME", FluidType::Slime as i64)?;
    lua.globals()
        .set("FLUID_LEMONADE", FluidType::Lemonade as i64)?;
    lua.globals().set("FLUID_MILK", FluidType::Milk as i64)?;
    lua.globals().set("FLUID_MANA", FluidType::Mana as i64)?;
    lua.globals().set("FLUID_LIFE", FluidType::Life as i64)?;
    lua.globals().set("FLUID_OIL", FluidType::Oil as i64)?;
    lua.globals().set("FLUID_URINE", FluidType::Urine as i64)?;
    lua.globals()
        .set("FLUID_COCONUTMILK", FluidType::CoconutMilk as i64)?;
    lua.globals().set("FLUID_WINE", FluidType::Wine as i64)?;
    lua.globals().set("FLUID_MUD", FluidType::Mud as i64)?;
    lua.globals()
        .set("FLUID_FRUITJUICE", FluidType::FruitJuice as i64)?;
    lua.globals().set("FLUID_LAVA", FluidType::Lava as i64)?;
    lua.globals().set("FLUID_RUM", FluidType::Rum as i64)?;
    lua.globals().set("FLUID_SWAMP", FluidType::Swamp as i64)?;
    lua.globals().set("FLUID_TEA", FluidType::Tea as i64)?;
    lua.globals().set("FLUID_MEAD", FluidType::Mead as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fluid_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return FLUID_NONE").eval().unwrap();
        assert_eq!(v, 0);
        let v: i64 = lua.load("return FLUID_BLOOD").eval().unwrap();
        assert_eq!(v, 2);
        let v: i64 = lua.load("return FLUID_MEAD").eval().unwrap();
        assert_eq!(v, 43);
    }
}
