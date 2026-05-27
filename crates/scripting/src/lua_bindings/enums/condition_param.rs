//! `CONDITION_PARAM_*` enum constants. Source: `common::enums::ConditionParam`.
#![cfg(feature = "lua-scripting")]

use forgottenserver_common::enums::ConditionParam;

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set(
        "CONDITION_PARAM_AGGRESSIVE",
        ConditionParam::Aggressive as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_BUFF_SPELL",
        ConditionParam::BuffSpell as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_DELAYED", ConditionParam::Delayed as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_DISABLE_DEFENSE",
        ConditionParam::DisableDefense as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_DRUNKENNESS",
        ConditionParam::Drunkenness as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_FIELD", ConditionParam::Field as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_FORCEUPDATE",
        ConditionParam::ForceUpdate as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_HEALTHGAIN",
        ConditionParam::HealthGain as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_HEALTHTICKS",
        ConditionParam::HealthTicks as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_LIGHT_COLOR",
        ConditionParam::LightColor as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_LIGHT_LEVEL",
        ConditionParam::LightLevel as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_MANAGAIN", ConditionParam::ManaGain as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_MANASHIELD_BREAKABLE",
        ConditionParam::ManaShieldBreakable as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_MANATICKS",
        ConditionParam::ManaTicks as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_MAXVALUE", ConditionParam::MaxValue as i64)?;
    lua.globals()
        .set("CONDITION_PARAM_MINVALUE", ConditionParam::MinValue as i64)?;
    lua.globals()
        .set("CONDITION_PARAM_OWNER", ConditionParam::Owner as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_PERIODICDAMAGE",
        ConditionParam::PeriodicDamage as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_SKILL_AXE", ConditionParam::SkillAxe as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_AXEPERCENT",
        ConditionParam::SkillAxePercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_CLUB",
        ConditionParam::SkillClub as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_CLUBPERCENT",
        ConditionParam::SkillClubPercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_DISTANCE",
        ConditionParam::SkillDistance as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_DISTANCEPERCENT",
        ConditionParam::SkillDistancePercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_FISHING",
        ConditionParam::SkillFishing as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_FISHINGPERCENT",
        ConditionParam::SkillFishingPercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_FIST",
        ConditionParam::SkillFist as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_FISTPERCENT",
        ConditionParam::SkillFistPercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_MELEE",
        ConditionParam::SkillMelee as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_MELEEPERCENT",
        ConditionParam::SkillMeleePercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_SHIELD",
        ConditionParam::SkillShield as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_SHIELDPERCENT",
        ConditionParam::SkillShieldPercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_SWORD",
        ConditionParam::SkillSword as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SKILL_SWORDPERCENT",
        ConditionParam::SkillSwordPercent as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_SOULGAIN", ConditionParam::SoulGain as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_SOULTICKS",
        ConditionParam::SoulTicks as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SPECIALSKILL_CRITICALHITAMOUNT",
        ConditionParam::SpecialSkillCriticalHitAmount as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SPECIALSKILL_CRITICALHITCHANCE",
        ConditionParam::SpecialSkillCriticalHitChance as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SPECIALSKILL_LIFELEECHAMOUNT",
        ConditionParam::SpecialSkillLifeLeechAmount as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SPECIALSKILL_LIFELEECHCHANCE",
        ConditionParam::SpecialSkillLifeLeechChance as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SPECIALSKILL_MANALEECHAMOUNT",
        ConditionParam::SpecialSkillManaLeechAmount as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_SPECIALSKILL_MANALEECHCHANCE",
        ConditionParam::SpecialSkillManaLeechChance as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_SPEED", ConditionParam::Speed as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_STARTVALUE",
        ConditionParam::StartValue as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_STAT_MAGICPOINTS",
        ConditionParam::StatMagicPoints as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_STAT_MAGICPOINTSPERCENT",
        ConditionParam::StatMagicPointsPercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_STAT_MAXHITPOINTS",
        ConditionParam::StatMaxHitPoints as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_STAT_MAXHITPOINTSPERCENT",
        ConditionParam::StatMaxHitPointsPercent as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_STAT_MAXMANAPOINTS",
        ConditionParam::StatMaxManaPoints as i64,
    )?;
    lua.globals().set(
        "CONDITION_PARAM_STAT_MAXMANAPOINTSPERCENT",
        ConditionParam::StatMaxManaPointsPercent as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_SUBID", ConditionParam::SubId as i64)?;
    lua.globals().set(
        "CONDITION_PARAM_TICKINTERVAL",
        ConditionParam::TickInterval as i64,
    )?;
    lua.globals()
        .set("CONDITION_PARAM_TICKS", ConditionParam::Ticks as i64)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn condition_param_constants_register() {
        let lua = mlua::Lua::new();
        install(&lua).unwrap();
        let _v: i64 = lua
            .load("return CONDITION_PARAM_AGGRESSIVE")
            .eval()
            .unwrap();
    }
}
