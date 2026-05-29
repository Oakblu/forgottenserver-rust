//! Sustained-load scenario: persistent connections sending ping packets.
//!
//! Spawns `config.bot_count` bots that each connect and then send only
//! pings for `config.duration_secs`. RSS is sampled every 10 seconds.

use std::time::{Duration, Instant};

use tokio::task::JoinSet;

use crate::client::TibiaClient;
use crate::metrics::{current_rss_bytes, RunMetrics};
use crate::scenarios::{merge_metrics, ScenarioConfig};

/// Run the sustained-load scenario.
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

            let mut client = match TibiaClient::connect(&host, port, &account, &password).await {
                Ok(c) => c,
                Err(_) => {
                    m.record_error();
                    m.duration_secs = dur as f64;
                    return m;
                }
            };

            while Instant::now() < task_deadline {
                match client.ping().await {
                    Ok(rtt_ms) => m.record_success(rtt_ms),
                    Err(_) => {
                        m.record_error();
                        break;
                    }
                }
            }

            let _ = client.disconnect().await;
            m.duration_secs = dur as f64;
            m
        });
    }

    let mut combined = RunMetrics::new();
    combined.rss_start_bytes = rss_start;
    let run_start = Instant::now();

    // Periodically sample RSS while waiting for bots to finish
    let mut last_rss_poll = Instant::now();
    while let Some(result) = set.join_next().await {
        if last_rss_poll.elapsed() >= Duration::from_secs(10) {
            let rss = current_rss_bytes();
            if rss > combined.peak_rss_bytes {
                combined.peak_rss_bytes = rss;
            }
            last_rss_poll = Instant::now();
        }
        let bot_metrics = result.unwrap_or_else(|_| {
            let mut m = RunMetrics::new();
            m.record_error();
            m.duration_secs = config.duration_secs as f64;
            m
        });
        merge_metrics(&mut combined, bot_metrics);
    }

    combined.rss_end_bytes = current_rss_bytes();
    combined.peak_rss_bytes = combined.peak_rss_bytes.max(combined.rss_end_bytes);
    combined.duration_secs = run_start.elapsed().as_secs_f64().max(1.0);

    Ok(combined)
}
