//! `CONST_ANI_*` distance/shoot effect constants.
//! Source: `common::constants::ShootType`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::constants::ShootType;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals()
        .set("CONST_ANI_ARROW", ShootType::Arrow as i64)?;
    lua.globals()
        .set("CONST_ANI_BOLT", ShootType::Bolt as i64)?;
    lua.globals()
        .set("CONST_ANI_BURSTARROW", ShootType::BurstArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_CAKE", ShootType::Cake as i64)?;
    lua.globals().set(
        "CONST_ANI_CRYSTALLINEARROW",
        ShootType::CrystallineArrow as i64,
    )?;
    lua.globals()
        .set("CONST_ANI_DEATH", ShootType::Death as i64)?;
    lua.globals()
        .set("CONST_ANI_DIAMONDARROW", ShootType::DiamondArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_DRILLBOLT", ShootType::DrillBolt as i64)?;
    lua.globals()
        .set("CONST_ANI_EARTH", ShootType::Earth as i64)?;
    lua.globals()
        .set("CONST_ANI_EARTHARROW", ShootType::EarthArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_ENCHANTEDSPEAR", ShootType::EnchantedSpear as i64)?;
    lua.globals()
        .set("CONST_ANI_ENERGY", ShootType::Energy as i64)?;
    lua.globals()
        .set("CONST_ANI_ENERGYBALL", ShootType::EnergyBall as i64)?;
    lua.globals()
        .set("CONST_ANI_ENVENOMEDARROW", ShootType::EnvenomedArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_ETHEREALSPEAR", ShootType::EtherealSpear as i64)?;
    lua.globals()
        .set("CONST_ANI_EXPLOSION", ShootType::Explosion as i64)?;
    lua.globals()
        .set("CONST_ANI_FIRE", ShootType::Fire as i64)?;
    lua.globals()
        .set("CONST_ANI_FLAMMINGARROW", ShootType::FlammingArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_FLASHARROW", ShootType::FlashArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_GLOOTHSPEAR", ShootType::GloothSpear as i64)?;
    lua.globals()
        .set("CONST_ANI_GREENSTAR", ShootType::GreenStar as i64)?;
    lua.globals()
        .set("CONST_ANI_HOLY", ShootType::Holy as i64)?;
    lua.globals()
        .set("CONST_ANI_HUNTINGSPEAR", ShootType::HuntingSpear as i64)?;
    lua.globals().set("CONST_ANI_ICE", ShootType::Ice as i64)?;
    lua.globals()
        .set("CONST_ANI_INFERNALBOLT", ShootType::InfernalBolt as i64)?;
    lua.globals()
        .set("CONST_ANI_LARGEROCK", ShootType::LargeRock as i64)?;
    lua.globals()
        .set("CONST_ANI_LEAFSTAR", ShootType::LeafStar as i64)?;
    lua.globals()
        .set("CONST_ANI_NONE", ShootType::None as i64)?;
    lua.globals()
        .set("CONST_ANI_ONYXARROW", ShootType::OnyxArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_PIERCINGBOLT", ShootType::PiercingBolt as i64)?;
    lua.globals()
        .set("CONST_ANI_POISON", ShootType::Poison as i64)?;
    lua.globals()
        .set("CONST_ANI_POISONARROW", ShootType::PoisonArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_POWERBOLT", ShootType::PowerBolt as i64)?;
    lua.globals()
        .set("CONST_ANI_PRISMATICBOLT", ShootType::PrismaticBolt as i64)?;
    lua.globals()
        .set("CONST_ANI_REDSTAR", ShootType::RedStar as i64)?;
    lua.globals()
        .set("CONST_ANI_ROYALSPEAR", ShootType::RoyalSpear as i64)?;
    lua.globals()
        .set("CONST_ANI_ROYALSTAR", ShootType::RoyalStar as i64)?;
    lua.globals()
        .set("CONST_ANI_SHIVERARROW", ShootType::ShiverArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_SIMPLEARROW", ShootType::SimpleArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_SMALLEARTH", ShootType::SmallEarth as i64)?;
    lua.globals()
        .set("CONST_ANI_SMALLHOLY", ShootType::SmallHoly as i64)?;
    lua.globals()
        .set("CONST_ANI_SMALLICE", ShootType::SmallIce as i64)?;
    lua.globals()
        .set("CONST_ANI_SMALLSTONE", ShootType::SmallStone as i64)?;
    lua.globals()
        .set("CONST_ANI_SNIPERARROW", ShootType::SniperArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_SNOWBALL", ShootType::Snowball as i64)?;
    lua.globals()
        .set("CONST_ANI_SPEAR", ShootType::Spear as i64)?;
    lua.globals()
        .set("CONST_ANI_SPECTRALBOLT", ShootType::SpectralBolt as i64)?;
    lua.globals()
        .set("CONST_ANI_SUDDENDEATH", ShootType::SuddenDeath as i64)?;
    lua.globals()
        .set("CONST_ANI_TARSALARROW", ShootType::TarsalArrow as i64)?;
    lua.globals()
        .set("CONST_ANI_THROWINGKNIFE", ShootType::ThrowingKnife as i64)?;
    lua.globals()
        .set("CONST_ANI_THROWINGSTAR", ShootType::ThrowingStar as i64)?;
    lua.globals()
        .set("CONST_ANI_VORTEXBOLT", ShootType::VortexBolt as i64)?;
    lua.globals()
        .set("CONST_ANI_WEAPONTYPE", ShootType::WeaponType as i64)?;
    lua.globals()
        .set("CONST_ANI_WHIRLWINDAXE", ShootType::WhirlwindAxe as i64)?;
    lua.globals()
        .set("CONST_ANI_WHIRLWINDCLUB", ShootType::WhirlwindClub as i64)?;
    lua.globals()
        .set("CONST_ANI_WHIRLWINDSWORD", ShootType::WhirlwindSword as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distance_effect_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        // Sanity: a few well-known ones.
        for name in ["CONST_ANI_NONE", "CONST_ANI_BOLT", "CONST_ANI_ARROW"] {
            lua.load(format!("return {name}")).eval::<i64>().unwrap();
        }
    }
}
