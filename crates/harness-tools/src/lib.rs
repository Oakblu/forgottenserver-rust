//! harness-tools — utilities for the equivalence harness.
//!
//! Currently exposes the ledger writer (consumes per-lane JSON reports
//! and proposes `MIGRATION_LEDGER.yml` transitions for human review).
//! Future lanes (Tibia-protocol-aware diff, packet replayer, etc.) will
//! also live here.
//!
//! See `openspec/changes/forgottenserver-rust-equivalence-harness/` for
//! the change record.

pub mod ledger;
pub mod lua_audit;
pub mod report;
pub mod transitions;
