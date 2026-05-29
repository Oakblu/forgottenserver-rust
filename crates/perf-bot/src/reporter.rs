use crate::metrics::RunMetrics;
use serde::Serialize;
use std::io::Write as _;

/// A complete comparison report for one scenario run against both targets.
#[derive(Serialize)]
pub struct ComparisonReport {
    pub scenario: String,
    pub bot_count: usize,
    pub duration_secs: f64,
    pub cpp: TargetReport,
    pub rust: TargetReport,
}

/// Results for one target (cpp or rust).
#[derive(Serialize)]
pub struct TargetReport {
    pub target: String,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub actions_per_sec: f64,
    pub error_rate_pct: f64,
    pub errors: u64,
    pub rss_start_mb: f64,
    pub rss_end_mb: f64,
    pub peak_rss_mb: f64,
}

impl TargetReport {
    pub fn from_metrics(target: &str, m: &RunMetrics) -> Self {
        const MB: f64 = 1_048_576.0; // 1 MiB in bytes
        TargetReport {
            target: target.to_string(),
            p50_ms: m.percentile(50.0),
            p95_ms: m.percentile(95.0),
            p99_ms: m.percentile(99.0),
            actions_per_sec: m.throughput(),
            error_rate_pct: m.error_rate() * 100.0,
            errors: m.errors,
            rss_start_mb: m.rss_start_bytes as f64 / MB,
            rss_end_mb: m.rss_end_bytes as f64 / MB,
            peak_rss_mb: m.peak_rss_bytes as f64 / MB,
        }
    }
}

impl ComparisonReport {
    /// Print the human-readable side-by-side table to stdout.
    pub fn print_table(&self) {
        let heavy = "\u{2501}"; // ━
        let light = "\u{2500}"; // ─
        let bar_len = 68usize;
        let heavy_bar = heavy.repeat(bar_len);
        let light_bar = light.repeat(bar_len);

        // Format duration as human-readable (e.g. "10m" or "90s")
        let dur_str = if self.duration_secs >= 60.0 {
            format!("{:.0}m", self.duration_secs / 60.0)
        } else {
            format!("{:.0}s", self.duration_secs)
        };

        println!("{heavy_bar}");
        println!(
            " Scenario: {} ({} bots, {})",
            self.scenario, self.bot_count, dur_str
        );
        println!(
            "{:25} {:>16} {:>16}",
            "",
            format!("C++ ({})", self.cpp.target),
            format!("Rust ({})", self.rust.target)
        );
        println!("{light_bar}");

        let row = |label: &str, cpp_val: String, rust_val: String| {
            println!(" {label:<24} {cpp_val:>16} {rust_val:>16}");
        };

        row(
            "Actions/sec",
            format!("{:.1}", self.cpp.actions_per_sec),
            format!("{:.1}", self.rust.actions_per_sec),
        );
        row(
            "p50 latency",
            format!("{}ms", self.cpp.p50_ms),
            format!("{}ms", self.rust.p50_ms),
        );
        row(
            "p95 latency",
            format!("{}ms", self.cpp.p95_ms),
            format!("{}ms", self.rust.p95_ms),
        );
        row(
            "p99 latency",
            format!("{}ms", self.cpp.p99_ms),
            format!("{}ms", self.rust.p99_ms),
        );
        row(
            "Error rate",
            format!("{:.2}%", self.cpp.error_rate_pct),
            format!("{:.2}%", self.rust.error_rate_pct),
        );
        row(
            "Errors",
            format!("{}", self.cpp.errors),
            format!("{}", self.rust.errors),
        );
        row(
            "RSS start",
            format!("{:.1} MB", self.cpp.rss_start_mb),
            format!("{:.1} MB", self.rust.rss_start_mb),
        );
        row(
            "RSS end",
            format!("{:.1} MB", self.cpp.rss_end_mb),
            format!("{:.1} MB", self.rust.rss_end_mb),
        );
        row(
            "Peak RSS",
            format!("{:.1} MB", self.cpp.peak_rss_mb),
            format!("{:.1} MB", self.rust.peak_rss_mb),
        );

        println!("{heavy_bar}");
        let _ = std::io::stdout().flush();
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self)
            .unwrap_or_else(|e| format!("{{\"error\": \"serialization failed: {e}\"}}"))
    }

    /// Write JSON to a file path.
    pub fn write_json(&self, path: &str) -> anyhow::Result<()> {
        let json = self.to_json();
        std::fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::RunMetrics;

    fn make_metrics(successes: u64, errors: u64, duration: f64, latencies: &[u64]) -> RunMetrics {
        let mut m = RunMetrics::new();
        m.successes = successes;
        m.errors = errors;
        m.duration_secs = duration;
        m.rss_start_bytes = 100 * 1_048_576; // 100 MB
        m.rss_end_bytes = 110 * 1_048_576; // 110 MB
        m.peak_rss_bytes = 115 * 1_048_576; // 115 MB
        for &lat in latencies {
            let _ = m.latency_ms.record(lat);
        }
        m
    }

    #[test]
    fn target_report_from_metrics_fields() {
        let m = make_metrics(900, 100, 10.0, &[10, 20, 30]);
        let r = TargetReport::from_metrics("cpp", &m);
        assert_eq!(r.target, "cpp");
        assert!((r.actions_per_sec - 90.0).abs() < 1e-6);
        assert!((r.error_rate_pct - 10.0).abs() < 1e-6);
        assert_eq!(r.errors, 100);
        assert!((r.rss_start_mb - 100.0).abs() < 0.01);
        assert!((r.rss_end_mb - 110.0).abs() < 0.01);
        assert!((r.peak_rss_mb - 115.0).abs() < 0.01);
    }

    #[test]
    fn comparison_report_to_json_is_valid() {
        let cpp_m = make_metrics(1000, 0, 10.0, &[5, 10, 15]);
        let rust_m = make_metrics(1200, 0, 10.0, &[3, 7, 10]);
        let report = ComparisonReport {
            scenario: "login_stress".to_string(),
            bot_count: 100,
            duration_secs: 10.0,
            cpp: TargetReport::from_metrics("cpp", &cpp_m),
            rust: TargetReport::from_metrics("rust", &rust_m),
        };
        let json = report.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("invalid JSON");
        assert_eq!(parsed["scenario"], "login_stress");
        assert_eq!(parsed["bot_count"], 100);
        assert_eq!(parsed["cpp"]["target"], "cpp");
        assert_eq!(parsed["rust"]["target"], "rust");
    }

    #[test]
    fn write_json_creates_file() {
        let cpp_m = make_metrics(500, 5, 5.0, &[8, 12, 25]);
        let rust_m = make_metrics(600, 0, 5.0, &[4, 6, 9]);
        let report = ComparisonReport {
            scenario: "walk_test".to_string(),
            bot_count: 50,
            duration_secs: 5.0,
            cpp: TargetReport::from_metrics("cpp", &cpp_m),
            rust: TargetReport::from_metrics("rust", &rust_m),
        };
        let path = std::env::temp_dir().join("perf_bot_test_report.json");
        let path_str = path.to_str().unwrap();
        report.write_json(path_str).expect("write_json failed");
        let content = std::fs::read_to_string(path_str).expect("read back failed");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("invalid JSON");
        assert_eq!(parsed["scenario"], "walk_test");
    }

    #[test]
    fn print_table_does_not_panic() {
        let cpp_m = make_metrics(4200, 42, 600.0, &[11, 31, 48]);
        let rust_m = make_metrics(5100, 0, 600.0, &[7, 14, 19]);
        let report = ComparisonReport {
            scenario: "full_chaos".to_string(),
            bot_count: 300,
            duration_secs: 600.0,
            cpp: TargetReport::from_metrics("cpp", &cpp_m),
            rust: TargetReport::from_metrics("rust", &rust_m),
        };
        // Must not panic; output goes to stdout.
        report.print_table();
    }
}
