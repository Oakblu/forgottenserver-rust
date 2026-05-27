//! `TEXTCOLOR_*` enum constants. Source: `common::enums::TextColor`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::TextColor;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("TEXTCOLOR_BLUE", TextColor::Blue as i64)?;
    lua.globals()
        .set("TEXTCOLOR_LIGHTGREEN", TextColor::LightGreen as i64)?;
    lua.globals()
        .set("TEXTCOLOR_LIGHTBLUE", TextColor::LightBlue as i64)?;
    lua.globals()
        .set("TEXTCOLOR_MAYABLUE", TextColor::MayaBlue as i64)?;
    lua.globals()
        .set("TEXTCOLOR_DARKRED", TextColor::DarkRed as i64)?;
    lua.globals()
        .set("TEXTCOLOR_LIGHTGREY", TextColor::LightGrey as i64)?;
    lua.globals()
        .set("TEXTCOLOR_SKYBLUE", TextColor::SkyBlue as i64)?;
    lua.globals()
        .set("TEXTCOLOR_PURPLE", TextColor::Purple as i64)?;
    lua.globals()
        .set("TEXTCOLOR_ELECTRICPURPLE", TextColor::ElectricPurple as i64)?;
    lua.globals().set("TEXTCOLOR_RED", TextColor::Red as i64)?;
    lua.globals()
        .set("TEXTCOLOR_PASTELRED", TextColor::PastelRed as i64)?;
    lua.globals()
        .set("TEXTCOLOR_ORANGE", TextColor::Orange as i64)?;
    lua.globals()
        .set("TEXTCOLOR_YELLOW", TextColor::Yellow as i64)?;
    lua.globals()
        .set("TEXTCOLOR_WHITE_EXP", TextColor::WhiteExp as i64)?;
    lua.globals()
        .set("TEXTCOLOR_NONE", TextColor::None as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_color_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return TEXTCOLOR_RED").eval().unwrap();
        assert_eq!(v, 180);
        let v: i64 = lua.load("return TEXTCOLOR_NONE").eval().unwrap();
        assert_eq!(v, 255);
    }
}
