use hdrhistogram::Histogram;
use sysinfo::{ProcessesToUpdate, System};

/// All metrics collected during a single scenario run against one server target.
pub struct RunMetrics {
    /// Latency histogram in milliseconds for successful requests.
    pub latency_ms: Histogram<u64>,
    /// Total number of successful actions recorded.
    pub successes: u64,
    /// Total number of failed/errored actions.
    pub errors: u64,
    /// Peak RSS (resident set size) in bytes during the run (polled at regular intervals).
    pub peak_rss_bytes: u64,
    /// RSS at start of run.
    pub rss_start_bytes: u64,
    /// RSS at end of run.
    pub rss_end_bytes: u64,
    /// Duration of the run in seconds.
    pub duration_secs: f64,
}

impl RunMetrics {
    pub fn new() -> Self {
        RunMetrics {
            // 3 significant figures; Histogram auto-resizes as needed.
            latency_ms: Histogram::<u64>::new(3).unwrap(),
            successes: 0,
            errors: 0,
            peak_rss_bytes: 0,
            rss_start_bytes: 0,
            rss_end_bytes: 0,
            duration_secs: 0.0,
        }
    }

    /// Record a successful action with given latency in milliseconds.
    pub fn record_success(&mut self, latency_ms: u64) {
        // Histogram::record returns Err only when the value exceeds the
        // configured max; with auto-resize enabled (new(3)) this never fires,
        // but saturate gracefully rather than panic.
        let _ = self.latency_ms.record(latency_ms);
        self.successes += 1;
    }

    /// Record a failed action.
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    /// Return latency at given percentile (0.0–100.0). Returns 0 if no data.
    pub fn percentile(&self, pct: f64) -> u64 {
        if self.latency_ms.is_empty() {
            return 0;
        }
        self.latency_ms.value_at_percentile(pct)
    }

    /// Throughput: successes per second.
    pub fn throughput(&self) -> f64 {
        if self.duration_secs <= 0.0 {
            return 0.0;
        }
        self.successes as f64 / self.duration_secs
    }

    /// Error rate as a fraction (0.0–1.0).
    pub fn error_rate(&self) -> f64 {
        let total = self.successes + self.errors;
        if total == 0 {
            return 0.0;
        }
        self.errors as f64 / total as f64
    }
}

impl Default for RunMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot RSS of the current process using sysinfo.
/// Returns bytes. Call this periodically during a run.
pub fn current_rss_bytes() -> u64 {
    let mut sys = System::new();
    if let Ok(pid) = sysinfo::get_current_pid() {
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
        if let Some(proc) = sys.process(pid) {
            // memory() returns bytes in sysinfo 0.33
            return proc.memory();
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_metrics_are_zeroed() {
        let m = RunMetrics::new();
        assert_eq!(m.successes, 0);
        assert_eq!(m.errors, 0);
        assert_eq!(m.peak_rss_bytes, 0);
        assert!(m.latency_ms.is_empty());
    }

    #[test]
    fn record_success_increments_and_stores_latency() {
        let mut m = RunMetrics::new();
        m.record_success(10);
        m.record_success(20);
        assert_eq!(m.successes, 2);
        assert_eq!(m.errors, 0);
        assert!(!m.latency_ms.is_empty());
    }

    #[test]
    fn record_error_increments_error_count() {
        let mut m = RunMetrics::new();
        m.record_error();
        m.record_error();
        assert_eq!(m.errors, 2);
        assert_eq!(m.successes, 0);
    }

    #[test]
    fn percentile_returns_zero_when_empty() {
        let m = RunMetrics::new();
        assert_eq!(m.percentile(99.0), 0);
    }

    #[test]
    fn percentile_returns_correct_value() {
        let mut m = RunMetrics::new();
        for ms in 1u64..=100 {
            m.record_success(ms);
        }
        // p50 of 1..=100 should be around 50
        let p50 = m.percentile(50.0);
        assert!((49..=51).contains(&p50), "p50={p50}");
        // p99 should be near 99
        let p99 = m.percentile(99.0);
        assert!((98..=100).contains(&p99), "p99={p99}");
    }

    #[test]
    fn throughput_is_zero_when_duration_is_zero() {
        let m = RunMetrics::new();
        assert_eq!(m.throughput(), 0.0);
    }

    #[test]
    fn throughput_divides_successes_by_duration() {
        let mut m = RunMetrics::new();
        m.successes = 600;
        m.duration_secs = 10.0;
        assert!((m.throughput() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn error_rate_is_zero_when_no_actions() {
        let m = RunMetrics::new();
        assert_eq!(m.error_rate(), 0.0);
    }

    #[test]
    fn error_rate_fraction() {
        let mut m = RunMetrics::new();
        m.successes = 9;
        m.errors = 1;
        assert!((m.error_rate() - 0.1).abs() < 1e-9);
    }

    #[test]
    fn current_rss_returns_nonzero() {
        // The current process must have some resident memory.
        let rss = current_rss_bytes();
        assert!(rss > 0, "expected nonzero RSS, got {rss}");
    }
}
