//! `REPORT_TYPE_*` and `REPORT_REASON_*` constants (bug-report / rule-violation
//! types). Source: C++ `enums.h::ReportType_t` / `ReportReason_t`. Values are
//! registered as literal `i64` since there is no Rust enum for them yet —
//! the Rust subsystems that handle reports haven't been migrated.
#![cfg(feature = "lua-scripting")]

pub fn install(lua: &mlua::Lua) -> mlua::Result<()> {
    // ReportType_t
    lua.globals().set("REPORT_TYPE_NAME", 0i64)?;
    lua.globals().set("REPORT_TYPE_STATEMENT", 1i64)?;
    lua.globals().set("REPORT_TYPE_BOT", 2i64)?;
    // ReportReason_t
    lua.globals().set("REPORT_REASON_NAMEINAPPROPRIATE", 0i64)?;
    lua.globals().set("REPORT_REASON_NAMEPOORFORMATTED", 1i64)?;
    lua.globals().set("REPORT_REASON_NAMEADVERTISING", 2i64)?;
    lua.globals().set("REPORT_REASON_NAMEUNFITTING", 3i64)?;
    lua.globals().set("REPORT_REASON_NAMERULEVIOLATION", 4i64)?;
    lua.globals()
        .set("REPORT_REASON_INSULTINGSTATEMENT", 5i64)?;
    lua.globals().set("REPORT_REASON_SPAMMING", 6i64)?;
    lua.globals()
        .set("REPORT_REASON_ADVERTISINGSTATEMENT", 7i64)?;
    lua.globals()
        .set("REPORT_REASON_UNFITTINGSTATEMENT", 8i64)?;
    lua.globals().set("REPORT_REASON_LANGUAGESTATEMENT", 9i64)?;
    lua.globals().set("REPORT_REASON_DISCLOSURE", 10i64)?;
    lua.globals().set("REPORT_REASON_RULEVIOLATION", 11i64)?;
    lua.globals()
        .set("REPORT_REASON_STATEMENT_BUGABUSE", 12i64)?;
    lua.globals()
        .set("REPORT_REASON_UNOFFICIALSOFTWARE", 13i64)?;
    lua.globals().set("REPORT_REASON_PRETENDING", 14i64)?;
    lua.globals().set("REPORT_REASON_HARASSINGOWNERS", 15i64)?;
    lua.globals().set("REPORT_REASON_FALSEINFO", 16i64)?;
    lua.globals().set("REPORT_REASON_ACCOUNTSHARING", 17i64)?;
    lua.globals().set("REPORT_REASON_STEALINGDATA", 18i64)?;
    lua.globals().set("REPORT_REASON_SERVICEATTACKING", 19i64)?;
    lua.globals().set("REPORT_REASON_SERVICEAGREEMENT", 20i64)?;
    Ok(())
}
