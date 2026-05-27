//! Status-transition proposal validator.
//!
//! Per `AI_MIGRATION_CONTEXT.md §3.1` the only allowed transitions are:
//!   - `PENDING → PARTIAL → DONE`  (forward)
//!   - `DONE → PARTIAL`            (leader spot-check / harness demote)
//!
//! All other transitions are rejected so the writer cannot accidentally
//! propose nonsensical state changes (e.g. PARTIAL → PENDING).

use crate::ledger::Ledger;
use crate::report::{LaneRecord, LedgerEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub cpp: String,
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// Walk the lane records, look up each entry's current status in the
/// ledger, and emit valid transitions. Invalid transitions (e.g.
/// `DONE → DONE`, `PARTIAL → PENDING`) are dropped silently.
pub fn propose(ledger: &Ledger, records: &[LaneRecord]) -> Vec<Transition> {
    let mut out = Vec::new();
    for rec in records {
        for entry in &rec.ledger_entries {
            if let Some(t) = build_transition(ledger, entry) {
                out.push(t);
            }
        }
    }
    out
}

fn build_transition(ledger: &Ledger, entry: &LedgerEntry) -> Option<Transition> {
    let current = ledger.status(&entry.cpp)?;
    let to = parse_target(&entry.transition)?;
    if !is_valid_transition(current, &to) {
        return None;
    }
    Some(Transition {
        cpp: entry.cpp.clone(),
        from: current.to_string(),
        to,
        reason: entry.reason.clone(),
    })
}

/// Extract the target state from a transition string like
/// "DONE → PARTIAL" or "PARTIAL → DONE".
fn parse_target(transition: &str) -> Option<String> {
    let parts: Vec<&str> = transition.split('→').map(str::trim).collect();
    if parts.len() == 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

fn is_valid_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("PENDING", "PARTIAL")
            | ("PARTIAL", "DONE")
            | ("PENDING", "DONE")  // collapsing forward is OK
            | ("DONE", "PARTIAL") // leader spot-check / harness demote
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::LedgerEntry;

    fn ledger_with(entries: &[(&str, &str)]) -> Ledger {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "harness-tools-transitions-test-{}-{}",
            std::process::id(),
            id,
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ledger.yml");
        let mut yaml = String::from("files:\n");
        for (cpp, status) in entries {
            yaml.push_str(&format!("- cpp: {cpp}\n  status: {status}\n"));
        }
        std::fs::write(&path, yaml).unwrap();
        Ledger::load(&path).unwrap()
    }

    fn record_with_entry(cpp: &str, transition: &str, reason: &str) -> LaneRecord {
        LaneRecord {
            lane: "test".to_string(),
            scenario: None,
            status: "FAIL".to_string(),
            ledger_entries: vec![LedgerEntry {
                cpp: cpp.to_string(),
                transition: transition.to_string(),
                reason: reason.to_string(),
            }],
        }
    }

    #[test]
    fn done_to_partial_is_proposed() {
        let ledger = ledger_with(&[("src/protocolgame.cpp", "DONE")]);
        let records = vec![record_with_entry(
            "src/protocolgame.cpp",
            "DONE → PARTIAL",
            "byte diverges at pkt 14",
        )];
        let out = propose(&ledger, &records);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].from, "DONE");
        assert_eq!(out[0].to, "PARTIAL");
    }

    #[test]
    fn partial_to_done_is_proposed() {
        let ledger = ledger_with(&[("src/item.cpp", "PARTIAL")]);
        let records = vec![record_with_entry(
            "src/item.cpp",
            "PARTIAL → DONE",
            "all lanes pass",
        )];
        let out = propose(&ledger, &records);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].to, "DONE");
    }

    #[test]
    fn done_to_done_is_dropped() {
        let ledger = ledger_with(&[("src/x.cpp", "DONE")]);
        let records = vec![record_with_entry("src/x.cpp", "DONE → DONE", "noop")];
        assert!(propose(&ledger, &records).is_empty());
    }

    #[test]
    fn partial_to_pending_is_rejected() {
        let ledger = ledger_with(&[("src/x.cpp", "PARTIAL")]);
        let records = vec![record_with_entry("src/x.cpp", "PARTIAL → PENDING", "bad")];
        assert!(propose(&ledger, &records).is_empty());
    }

    #[test]
    fn unknown_file_is_dropped() {
        let ledger = ledger_with(&[("src/x.cpp", "DONE")]);
        let records = vec![record_with_entry("src/missing.cpp", "DONE → PARTIAL", "?")];
        assert!(propose(&ledger, &records).is_empty());
    }

    #[test]
    fn malformed_transition_string_is_dropped() {
        let ledger = ledger_with(&[("src/x.cpp", "DONE")]);
        let records = vec![record_with_entry(
            "src/x.cpp",
            "DONE PARTIAL",
            "missing arrow",
        )];
        assert!(propose(&ledger, &records).is_empty());
    }

    #[test]
    fn pending_to_done_collapses_forward() {
        let ledger = ledger_with(&[("src/new.cpp", "PENDING")]);
        let records = vec![record_with_entry(
            "src/new.cpp",
            "PENDING → DONE",
            "scaffold + tests landed in one shot",
        )];
        let out = propose(&ledger, &records);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].to, "DONE");
    }

    #[test]
    fn empty_records_yield_no_transitions() {
        let ledger = ledger_with(&[("src/x.cpp", "DONE")]);
        assert!(propose(&ledger, &[]).is_empty());
    }
}
