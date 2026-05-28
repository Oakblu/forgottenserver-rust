//! Per-lane JSON report parsing.
//!
//! All harness lanes emit one or more JSON records per scenario, written
//! as one record per line (JSON-lines) to a single run report. This
//! module knows how to read that stream and surface the embedded
//! `ledger_entries` to the writer.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A single record in a lane's JSON-lines report.
///
/// Free-form by design: lanes attach lane-specific metadata under the
/// untyped `extra` map. The writer only consumes `lane`, `scenario`,
/// `status`, and `ledger_entries` directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneRecord {
    pub lane: String,
    #[serde(default)]
    pub scenario: Option<String>,
    pub status: String,
    #[serde(default)]
    pub ledger_entries: Vec<LedgerEntry>,
}

/// A proposed status transition emitted by a lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerEntry {
    pub cpp: String,
    pub transition: String,
    pub reason: String,
}

/// Parse a JSON-lines run report into a list of lane records.
///
/// The first line is the run-header metadata (timestamp, lanes
/// requested) — we skip it if it lacks a `lane` field.
pub fn parse_run_report(path: &Path) -> anyhow::Result<Vec<LaneRecord>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        // Try to parse as a LaneRecord; if it doesn't have `lane` /
        // `status`, treat it as the run header and skip.
        match serde_json::from_str::<LaneRecord>(&line) {
            Ok(rec) => records.push(rec),
            Err(_) if idx == 0 => continue, // run header
            Err(e) => return Err(anyhow::anyhow!("line {}: {e}", idx + 1)),
        }
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_skips_first_line_when_it_lacks_lane_field() {
        let tmp = tempdir_with_file(
            "{\"timestamp\":\"2026-05-25T08:00Z\",\"lanes_requested\":[\"wire_replay\"]}\n\
             {\"lane\":\"wire_replay\",\"status\":\"PASS\"}\n",
        );
        let records = parse_run_report(&tmp).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].lane, "wire_replay");
        assert_eq!(records[0].status, "PASS");
    }

    #[test]
    fn parse_empty_ledger_entries_defaults_to_empty_vec() {
        let tmp = tempdir_with_file("{\"lane\":\"x\",\"status\":\"PASS\"}\n");
        let records = parse_run_report(&tmp).unwrap();
        assert!(records[0].ledger_entries.is_empty());
    }

    #[test]
    fn parse_ledger_entries_present() {
        let tmp = tempdir_with_file(
            "{\"lane\":\"wire_replay\",\"scenario\":\"login\",\"status\":\"FAIL\",\
             \"ledger_entries\":[{\"cpp\":\"src/protocolgame.cpp\",\
             \"transition\":\"DONE → PARTIAL\",\"reason\":\"diverges at pkt 14\"}]}\n",
        );
        let records = parse_run_report(&tmp).unwrap();
        assert_eq!(records[0].ledger_entries.len(), 1);
        assert_eq!(records[0].ledger_entries[0].cpp, "src/protocolgame.cpp");
    }

    #[test]
    fn parse_blank_lines_are_skipped() {
        let tmp = tempdir_with_file(
            "{\"lane\":\"a\",\"status\":\"PASS\"}\n\
             \n\
             {\"lane\":\"b\",\"status\":\"PASS\"}\n",
        );
        let records = parse_run_report(&tmp).unwrap();
        assert_eq!(records.len(), 2);
    }

    fn tempdir_with_file(content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "harness-tools-test-{:?}-{}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("run.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }
}
