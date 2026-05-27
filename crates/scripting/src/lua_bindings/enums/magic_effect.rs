//! `CONST_ME_*` magic effect enum constants.
//! Source: `common::constants::MagicEffectClass`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::MagicEffectClass;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CONST_ME_ASSASSIN", MagicEffectClass::Assassin as i64)?;
    lua.globals()
        .set("CONST_ME_BATS", MagicEffectClass::Bats as i64)?;
    lua.globals()
        .set("CONST_ME_BIGCLOUDS", MagicEffectClass::BigClouds as i64)?;
    lua.globals().set(
        "CONST_ME_BIGCLOUDSSINGLESPACE",
        MagicEffectClass::BigCloudsSingleSpace as i64,
    )?;
    lua.globals()
        .set("CONST_ME_BIGPLANTS", MagicEffectClass::BigPlants as i64)?;
    lua.globals().set(
        "CONST_ME_BIGSCRATCHING",
        MagicEffectClass::BigScratching as i64,
    )?;
    lua.globals()
        .set("CONST_ME_BITE", MagicEffectClass::Bite as i64)?;
    lua.globals()
        .set("CONST_ME_BLACKSMOKE", MagicEffectClass::BlackSmoke as i64)?;
    lua.globals()
        .set("CONST_ME_BLOCK", MagicEffectClass::Block as i64)?;
    lua.globals()
        .set("CONST_ME_BLOCKHIT", MagicEffectClass::BlockHit as i64)?;
    lua.globals()
        .set("CONST_ME_BLOODYSTEPS", MagicEffectClass::BloodySteps as i64)?;
    lua.globals()
        .set("CONST_ME_BLUECHAIN", MagicEffectClass::BlueChain as i64)?;
    lua.globals()
        .set("CONST_ME_BLUEGHOST", MagicEffectClass::BlueGhost as i64)?;
    lua.globals()
        .set("CONST_ME_BUBBLES", MagicEffectClass::Bubbles as i64)?;
    lua.globals()
        .set("CONST_ME_CAKE", MagicEffectClass::Cake as i64)?;
    lua.globals()
        .set("CONST_ME_CARNIPHILA", MagicEffectClass::Carniphila as i64)?;
    lua.globals().set(
        "CONST_ME_CONFETTI_HORIZONTAL",
        MagicEffectClass::ConfettiHorizontal as i64,
    )?;
    lua.globals().set(
        "CONST_ME_CONFETTI_VERTICAL",
        MagicEffectClass::ConfettiVertical as i64,
    )?;
    lua.globals()
        .set("CONST_ME_CRAPS", MagicEffectClass::Craps as i64)?;
    lua.globals().set(
        "CONST_ME_CRITICAL_DAMAGE",
        MagicEffectClass::CriticalDamage as i64,
    )?;
    lua.globals()
        .set("CONST_ME_DEVOVORGA", MagicEffectClass::Devovorga as i64)?;
    lua.globals().set(
        "CONST_ME_DIVINEDAZZLE",
        MagicEffectClass::DivineDazzle as i64,
    )?;
    lua.globals()
        .set("CONST_ME_DODGE", MagicEffectClass::Dodge as i64)?;
    lua.globals()
        .set("CONST_ME_DRAGONHEAD", MagicEffectClass::DragonHead as i64)?;
    lua.globals()
        .set("CONST_ME_DRAWBLOOD", MagicEffectClass::DrawBlood as i64)?;
    lua.globals()
        .set("CONST_ME_DRAWINK", MagicEffectClass::DrawInk as i64)?;
    lua.globals().set(
        "CONST_ME_EARLY_THUNDER",
        MagicEffectClass::EarlyThunder as i64,
    )?;
    lua.globals().set(
        "CONST_ME_ELECTRICALSPARK",
        MagicEffectClass::ElectricalSpark as i64,
    )?;
    lua.globals()
        .set("CONST_ME_ENERGYAREA", MagicEffectClass::EnergyArea as i64)?;
    lua.globals()
        .set("CONST_ME_ENERGYHIT", MagicEffectClass::EnergyHit as i64)?;
    lua.globals().set(
        "CONST_ME_EXPLOSIONAREA",
        MagicEffectClass::ExplosionArea as i64,
    )?;
    lua.globals().set(
        "CONST_ME_EXPLOSIONHIT",
        MagicEffectClass::ExplosionHit as i64,
    )?;
    lua.globals()
        .set("CONST_ME_FAECOMING", MagicEffectClass::FaeComing as i64)?;
    lua.globals().set(
        "CONST_ME_FAEEXPLOSION",
        MagicEffectClass::FaeExplosion as i64,
    )?;
    lua.globals()
        .set("CONST_ME_FAEGOING", MagicEffectClass::FaeGoing as i64)?;
    lua.globals()
        .set("CONST_ME_FATAL", MagicEffectClass::Fatal as i64)?;
    lua.globals()
        .set("CONST_ME_FERUMBRAS", MagicEffectClass::Ferumbras as i64)?;
    lua.globals()
        .set("CONST_ME_FERUMBRAS_1", MagicEffectClass::Ferumbras1 as i64)?;
    lua.globals()
        .set("CONST_ME_FERUMBRAS_2", MagicEffectClass::Ferumbras2 as i64)?;
    lua.globals()
        .set("CONST_ME_FIREAREA", MagicEffectClass::FireArea as i64)?;
    lua.globals()
        .set("CONST_ME_FIREATTACK", MagicEffectClass::FireAttack as i64)?;
    lua.globals().set(
        "CONST_ME_FIREWORKSCIRCLE",
        MagicEffectClass::FireworksCircle as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORKSSTAR",
        MagicEffectClass::FireworksStar as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORK_BLUE",
        MagicEffectClass::FireworkBlue as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORK_GREEN",
        MagicEffectClass::FireworkGreen as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORK_ORANGE",
        MagicEffectClass::FireworkOrange as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORK_PURPLE",
        MagicEffectClass::FireworkPurple as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORK_RED",
        MagicEffectClass::FireworkRed as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORK_TURQUOISE",
        MagicEffectClass::FireworkTurquoise as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FIREWORK_YELLOW",
        MagicEffectClass::FireworkYellow as i64,
    )?;
    lua.globals().set(
        "CONST_ME_FLOATINGBLOCK",
        MagicEffectClass::FloatingBlock as i64,
    )?;
    lua.globals()
        .set("CONST_ME_FOAM", MagicEffectClass::Foam as i64)?;
    lua.globals()
        .set("CONST_ME_GAZHARAGOTH", MagicEffectClass::Gazharagoth as i64)?;
    lua.globals()
        .set("CONST_ME_GHOSTLYBITE", MagicEffectClass::GhostlyBite as i64)?;
    lua.globals().set(
        "CONST_ME_GHOSTLYSCRATCH",
        MagicEffectClass::GhostlyScratch as i64,
    )?;
    lua.globals()
        .set("CONST_ME_GHOSTSMOKE", MagicEffectClass::GhostSmoke as i64)?;
    lua.globals()
        .set("CONST_ME_GIANTICE", MagicEffectClass::GiantIce as i64)?;
    lua.globals()
        .set("CONST_ME_GIFT_WRAPS", MagicEffectClass::GiftWraps as i64)?;
    lua.globals()
        .set("CONST_ME_GREENCHAIN", MagicEffectClass::GreenChain as i64)?;
    lua.globals()
        .set("CONST_ME_GREENSMOKE", MagicEffectClass::GreenSmoke as i64)?;
    lua.globals()
        .set("CONST_ME_GREEN_RINGS", MagicEffectClass::GreenRings as i64)?;
    lua.globals()
        .set("CONST_ME_GREYCHAIN", MagicEffectClass::GreyChain as i64)?;
    lua.globals().set(
        "CONST_ME_GREYTELEPORT",
        MagicEffectClass::GreyTeleport as i64,
    )?;
    lua.globals().set(
        "CONST_ME_GROUNDSHAKER",
        MagicEffectClass::GroundShaker as i64,
    )?;
    lua.globals()
        .set("CONST_ME_HEARTS", MagicEffectClass::Hearts as i64)?;
    lua.globals()
        .set("CONST_ME_HITAREA", MagicEffectClass::HitArea as i64)?;
    lua.globals()
        .set("CONST_ME_HITBYFIRE", MagicEffectClass::HitByFire as i64)?;
    lua.globals()
        .set("CONST_ME_HITBYPOISON", MagicEffectClass::HitByPoison as i64)?;
    lua.globals()
        .set("CONST_ME_HOLYAREA", MagicEffectClass::HolyArea as i64)?;
    lua.globals()
        .set("CONST_ME_HOLYDAMAGE", MagicEffectClass::HolyDamage as i64)?;
    lua.globals()
        .set("CONST_ME_HORESTIS", MagicEffectClass::Horestis as i64)?;
    lua.globals()
        .set("CONST_ME_HOURGLASS", MagicEffectClass::Hourglass as i64)?;
    lua.globals()
        .set("CONST_ME_ICEAREA", MagicEffectClass::IceArea as i64)?;
    lua.globals()
        .set("CONST_ME_ICEATTACK", MagicEffectClass::IceAttack as i64)?;
    lua.globals()
        .set("CONST_ME_ICETORNADO", MagicEffectClass::IceTornado as i64)?;
    lua.globals()
        .set("CONST_ME_INSECTS", MagicEffectClass::Insects as i64)?;
    lua.globals().set(
        "CONST_ME_LIGHTBLUETELEPORT",
        MagicEffectClass::LightBlueTeleport as i64,
    )?;
    lua.globals()
        .set("CONST_ME_LOSEENERGY", MagicEffectClass::LoseEnergy as i64)?;
    lua.globals()
        .set("CONST_ME_MAD_MAGE", MagicEffectClass::MadMage as i64)?;
    lua.globals()
        .set("CONST_ME_MAGIC_BLUE", MagicEffectClass::MagicBlue as i64)?;
    lua.globals()
        .set("CONST_ME_MAGIC_GREEN", MagicEffectClass::MagicGreen as i64)?;
    lua.globals()
        .set("CONST_ME_MAGIC_RED", MagicEffectClass::MagicRed as i64)?;
    lua.globals()
        .set("CONST_ME_MAPEFFECT", MagicEffectClass::MapEffect as i64)?;
    lua.globals().set(
        "CONST_ME_MIRRORHORIZONTAL",
        MagicEffectClass::MirrorHorizontal as i64,
    )?;
    lua.globals().set(
        "CONST_ME_MIRRORVERTICAL",
        MagicEffectClass::MirrorVertical as i64,
    )?;
    lua.globals()
        .set("CONST_ME_MORTAREA", MagicEffectClass::MortArea as i64)?;
    lua.globals()
        .set("CONST_ME_NONE", MagicEffectClass::None as i64)?;
    lua.globals()
        .set("CONST_ME_ORANGECHAIN", MagicEffectClass::OrangeChain as i64)?;
    lua.globals().set(
        "CONST_ME_ORANGETELEPORT",
        MagicEffectClass::OrangeTeleport as i64,
    )?;
    lua.globals()
        .set("CONST_ME_ORCSHAMAN", MagicEffectClass::OrcShaman as i64)?;
    lua.globals().set(
        "CONST_ME_ORCSHAMAN_FIRE",
        MagicEffectClass::OrcShamanFire as i64,
    )?;
    lua.globals()
        .set("CONST_ME_PINKSPARK", MagicEffectClass::PinkSpark as i64)?;
    lua.globals()
        .set("CONST_ME_PLANTATTACK", MagicEffectClass::PlantAttack as i64)?;
    lua.globals().set(
        "CONST_ME_PLUNGING_FISH",
        MagicEffectClass::PlungingFish as i64,
    )?;
    lua.globals()
        .set("CONST_ME_POFF", MagicEffectClass::Poff as i64)?;
    lua.globals().set(
        "CONST_ME_POINTOFINTEREST",
        MagicEffectClass::PointOfInterest as i64,
    )?;
    lua.globals()
        .set("CONST_ME_POISONAREA", MagicEffectClass::PoisonArea as i64)?;
    lua.globals().set(
        "CONST_ME_PRISMATICSPARKLES",
        MagicEffectClass::PrismaticSparkles as i64,
    )?;
    lua.globals()
        .set("CONST_ME_PURPLECHAIN", MagicEffectClass::PurpleChain as i64)?;
    lua.globals().set(
        "CONST_ME_PURPLEENERGY",
        MagicEffectClass::PurpleEnergy as i64,
    )?;
    lua.globals()
        .set("CONST_ME_PURPLESMOKE", MagicEffectClass::PurpleSmoke as i64)?;
    lua.globals().set(
        "CONST_ME_PURPLETELEPORT",
        MagicEffectClass::PurpleTeleport as i64,
    )?;
    lua.globals().set(
        "CONST_ME_RAGIAZ_BONECAPSULE",
        MagicEffectClass::RagiazBoneCapsule as i64,
    )?;
    lua.globals()
        .set("CONST_ME_REDSMOKE", MagicEffectClass::RedSmoke as i64)?;
    lua.globals()
        .set("CONST_ME_REDTELEPORT", MagicEffectClass::RedTeleport as i64)?;
    lua.globals()
        .set("CONST_ME_ROOTING", MagicEffectClass::Rooting as i64)?;
    lua.globals().set(
        "CONST_ME_SKULLHORIZONTAL",
        MagicEffectClass::SkullHorizontal as i64,
    )?;
    lua.globals().set(
        "CONST_ME_SKULLVERTICAL",
        MagicEffectClass::SkullVertical as i64,
    )?;
    lua.globals()
        .set("CONST_ME_SLASH", MagicEffectClass::Slash as i64)?;
    lua.globals()
        .set("CONST_ME_SLEEP", MagicEffectClass::Sleep as i64)?;
    lua.globals()
        .set("CONST_ME_SMALLCLOUDS", MagicEffectClass::SmallClouds as i64)?;
    lua.globals()
        .set("CONST_ME_SMALLPLANTS", MagicEffectClass::SmallPlants as i64)?;
    lua.globals()
        .set("CONST_ME_SMOKE", MagicEffectClass::Smoke as i64)?;
    lua.globals()
        .set("CONST_ME_SOUND_BLUE", MagicEffectClass::SoundBlue as i64)?;
    lua.globals()
        .set("CONST_ME_SOUND_GREEN", MagicEffectClass::SoundGreen as i64)?;
    lua.globals().set(
        "CONST_ME_SOUND_PURPLE",
        MagicEffectClass::SoundPurple as i64,
    )?;
    lua.globals()
        .set("CONST_ME_SOUND_RED", MagicEffectClass::SoundRed as i64)?;
    lua.globals()
        .set("CONST_ME_SOUND_WHITE", MagicEffectClass::SoundWhite as i64)?;
    lua.globals().set(
        "CONST_ME_SOUND_YELLOW",
        MagicEffectClass::SoundYellow as i64,
    )?;
    lua.globals().set(
        "CONST_ME_STEPSHORIZONTAL",
        MagicEffectClass::StepsHorizontal as i64,
    )?;
    lua.globals().set(
        "CONST_ME_STEPSVERTICAL",
        MagicEffectClass::StepsVertical as i64,
    )?;
    lua.globals()
        .set("CONST_ME_STONES", MagicEffectClass::Stones as i64)?;
    lua.globals().set(
        "CONST_ME_STONESSINGLESPACE",
        MagicEffectClass::StonesSingleSpace as i64,
    )?;
    lua.globals()
        .set("CONST_ME_STUN", MagicEffectClass::Stun as i64)?;
    lua.globals()
        .set("CONST_ME_TELEPORT", MagicEffectClass::Teleport as i64)?;
    lua.globals()
        .set("CONST_ME_THAIAN", MagicEffectClass::Thaian as i64)?;
    lua.globals()
        .set("CONST_ME_THAIANGHOST", MagicEffectClass::ThaianGhost as i64)?;
    lua.globals()
        .set("CONST_ME_THECUBE", MagicEffectClass::TheCube as i64)?;
    lua.globals()
        .set("CONST_ME_THUNDER", MagicEffectClass::Thunder as i64)?;
    lua.globals().set(
        "CONST_ME_TUTORIALARROW",
        MagicEffectClass::TutorialArrow as i64,
    )?;
    lua.globals().set(
        "CONST_ME_TUTORIALSQUARE",
        MagicEffectClass::TutorialSquare as i64,
    )?;
    lua.globals().set(
        "CONST_ME_WATERCREATURE",
        MagicEffectClass::WaterCreature as i64,
    )?;
    lua.globals()
        .set("CONST_ME_WATERSPLASH", MagicEffectClass::WaterSplash as i64)?;
    lua.globals().set(
        "CONST_ME_YALAHARIGHOST",
        MagicEffectClass::YalaharIGhost as i64,
    )?;
    lua.globals()
        .set("CONST_ME_YELLOWCHAIN", MagicEffectClass::YellowChain as i64)?;
    lua.globals().set(
        "CONST_ME_YELLOWENERGY",
        MagicEffectClass::YellowEnergy as i64,
    )?;
    lua.globals()
        .set("CONST_ME_YELLOWSMOKE", MagicEffectClass::YellowSmoke as i64)?;
    lua.globals().set(
        "CONST_ME_YELLOWSPARKLES",
        MagicEffectClass::YellowSparkles as i64,
    )?;
    lua.globals().set(
        "CONST_ME_YELLOW_RINGS",
        MagicEffectClass::YellowRings as i64,
    )?;
    // Spelling divergence: C++ has `CHIVALRIOUS` (typo for `CHIVALROUS`);
    // Rust common spells it correctly. Register under the C++ name so
    // existing data/*.lua scripts work unchanged.
    lua.globals().set(
        "CONST_ME_CHIVALRIOUSCHALLENGE",
        MagicEffectClass::ChivalrousChallenge as i64,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn const_me_fire_area_is_7() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let v: i64 = lua.load("return CONST_ME_FIREAREA").eval().unwrap();
        assert_eq!(v, 7);
    }
    #[test]
    fn const_me_chivalrious_uses_cpp_spelling() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        // C++ name (with typo) resolves to the (correctly-spelled)
        // Rust variant ChivalrousChallenge.
        let v: i64 = lua
            .load("return CONST_ME_CHIVALRIOUSCHALLENGE")
            .eval()
            .unwrap();
        assert_eq!(v, MagicEffectClass::ChivalrousChallenge as i64);
    }
    #[test]
    fn registers_at_least_140_effects() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        // Sanity: a sampling of well-known effects.
        for name in [
            "CONST_ME_NONE",
            "CONST_ME_DRAWBLOOD",
            "CONST_ME_FIREAREA",
            "CONST_ME_TELEPORT",
            "CONST_ME_HOLYDAMAGE",
            "CONST_ME_CRITICAL_DAMAGE",
        ] {
            lua.load(format!("return {name}")).eval::<i64>().unwrap();
        }
    }
}
