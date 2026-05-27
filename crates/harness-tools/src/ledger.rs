//! Minimal `MIGRATION_LEDGER.yml` reader for the writer.
//!
//! We only parse the subset of the ledger needed to validate proposed
//! transitions: the per-file `status` field, indexed by `cpp` path. The
//! full schema lives in `MIGRATION_LEDGER.yml` itself; this is a
//! projection.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct LedgerDoc {
    files: Vec<FileEntry>,
}

#[derive(Debug, Deserialize)]
struct FileEntry {
    cpp: String,
    #[serde(default)]
    status: String,
}

/// In-memory projection: cpp path → current status.
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    by_cpp: HashMap<String, String>,
}

impl Ledger {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let doc: LedgerDoc = serde_yaml::from_str(&raw)?;
        let mut by_cpp = HashMap::with_capacity(doc.files.len());
        for f in doc.files {
            if !f.cpp.is_empty() && !f.status.is_empty() {
                by_cpp.insert(f.cpp, f.status);
            }
        }
        Ok(Self { by_cpp })
    }

    pub fn status(&self, cpp: &str) -> Option<&str> {
        self.by_cpp.get(cpp).map(String::as_str)
    }

    pub fn known_files(&self) -> impl Iterator<Item = &str> {
        self.by_cpp.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_parses_status_per_file() {
        let tmp = tempfile_with_content(
            "files:\n\
             - cpp: src/foo.cpp\n  status: DONE\n\
             - cpp: src/bar.cpp\n  status: PARTIAL\n",
        );
        let ledger = Ledger::load(&tmp).unwrap();
        assert_eq!(ledger.status("src/foo.cpp"), Some("DONE"));
        assert_eq!(ledger.status("src/bar.cpp"), Some("PARTIAL"));
    }

    #[test]
    fn status_unknown_returns_none() {
        let tmp = tempfile_with_content("files:\n- cpp: src/foo.cpp\n  status: DONE\n");
        let ledger = Ledger::load(&tmp).unwrap();
        assert_eq!(ledger.status("src/missing.cpp"), None);
    }

    #[test]
    fn load_skips_entries_with_empty_status() {
        let tmp = tempfile_with_content(
            "files:\n\
             - cpp: src/foo.cpp\n  status: DONE\n\
             - cpp: src/empty.cpp\n",
        );
        let ledger = Ledger::load(&tmp).unwrap();
        assert_eq!(ledger.status("src/foo.cpp"), Some("DONE"));
        assert_eq!(ledger.status("src/empty.cpp"), None);
    }

    fn tempfile_with_content(content: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "harness-tools-ledger-test-{}-{}",
            std::process::id(),
            id,
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ledger.yml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }
}
