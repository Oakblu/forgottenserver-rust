//! Login flood scenario: rapidly connect and immediately disconnect.
//!
//! Measures how many login handshakes the server can handle per second and
//! records per-connection latency.

use std::time::{Duration, Instant};

use tokio::task::JoinSet;

use crate::client::TibiaClient;
use crate::metrics::{current_rss_bytes, RunMetrics};
use crate::scenarios::{merge_metrics, ScenarioConfig};

/// Run the login-flood scenario.
///
/// Spawns `config.bot_count` concurrent tasks. Each task loops: connect →
/// record latency → disconnect, until `config.duration_secs` elapse.
pub async fn run(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    let rss_start = current_rss_bytes();

    let mut set: JoinSet<RunMetrics> = JoinSet::new();

    for i in 1..=config.bot_count {
        let host = config.host.clone();
        let port = config.port;
        let account = format!("{}{i}", config.account_prefix);
        let password = config.password.clone();
        let dur = config.duration_secs;

        set.spawn(async move {
            let mut m = RunMetrics::new();
            let task_deadline = Instant::now() + Duration::from_secs(dur);

            while Instant::now() < task_deadline {
                let t0 = Instant::now();
                match TibiaClient::connect(&host, port, &account, &password).await {
                    Ok(mut client) => {
                        let elapsed_ms = t0.elapsed().as_millis() as u64;
                        m.record_success(elapsed_ms.max(1));
                        let _ = client.disconnect().await;
                    }
                    Err(_) => {
                        m.record_error();
                    }
                }
            }

            m.duration_secs = dur as f64;
            m
        });
    }

    let mut combined = RunMetrics::new();
    combined.rss_start_bytes = rss_start;
    let run_start = Instant::now();

    while let Some(result) = set.join_next().await {
        let bot_metrics = result.unwrap_or_else(|_| {
            let mut m = RunMetrics::new();
            m.record_error();
            m.duration_secs = config.duration_secs as f64;
            m
        });
        let current_rss = current_rss_bytes();
        if current_rss > combined.peak_rss_bytes {
            combined.peak_rss_bytes = current_rss;
        }
        merge_metrics(&mut combined, bot_metrics);
    }

    combined.rss_end_bytes = current_rss_bytes();
    combined.peak_rss_bytes = combined.peak_rss_bytes.max(combined.rss_end_bytes);
    combined.duration_secs = run_start.elapsed().as_secs_f64().max(1.0);

    Ok(combined)
}
