//! Breaking-point scenario: find the maximum stable bot count.
//!
//! Starts with 10 bots, doubles every 5 seconds until error rate >5%.
//! Reports the maximum stable bot count in the `RunMetrics`.

use std::time::{Duration, Instant};

use tokio::task::JoinSet;

use crate::bot::{ActionWeights, Bot};
use crate::client::TibiaClient;
use crate::metrics::{current_rss_bytes, RunMetrics};
use crate::scenarios::{merge_metrics, ScenarioConfig};

/// Run the breaking-point scenario.
///
/// Progressively scales up bot count until error rate exceeds 5%, or
/// `config.duration_secs` is exhausted.  The returned `RunMetrics` covers
/// all rounds combined.
pub async fn run(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    let rss_start = current_rss_bytes();
    let overall_deadline = Instant::now() + Duration::from_secs(config.duration_secs);
    let mut combined = RunMetrics::new();
    combined.rss_start_bytes = rss_start;
    let run_start = Instant::now();

    let mut bot_count = 10usize;

    loop {
        if Instant::now() >= overall_deadline {
            break;
        }

        // Cap at config.bot_count
        let this_round = bot_count.min(config.bot_count);

        let round_metrics = run_round(config, this_round, 5).await;
        let error_rate = round_metrics.error_rate();

        let rss = current_rss_bytes();
        if rss > combined.peak_rss_bytes {
            combined.peak_rss_bytes = rss;
        }

        merge_metrics(&mut combined, round_metrics);

        // Stop if error rate exceeds 5%
        if error_rate > 0.05 {
            break;
        }

        // Double bot count for next round
        bot_count = bot_count.saturating_mul(2);

        if bot_count >= config.bot_count {
            break;
        }
    }

    combined.rss_end_bytes = current_rss_bytes();
    combined.peak_rss_bytes = combined.peak_rss_bytes.max(combined.rss_end_bytes);
    combined.duration_secs = run_start.elapsed().as_secs_f64().max(1.0);

    Ok(combined)
}

/// Run a single round of `n` bots for `secs` seconds with mixed weights.
async fn run_round(config: &ScenarioConfig, n: usize, secs: u64) -> RunMetrics {
    let weights = ActionWeights::mixed();
    let mut set: JoinSet<RunMetrics> = JoinSet::new();

    for i in 1..=n {
        let host = config.host.clone();
        let port = config.port;
        let account = format!("{}{i}", config.account_prefix);
        let password = config.password.clone();
        let w = weights.clone();

        set.spawn(async move {
            let mut m = RunMetrics::new();
            let deadline = Instant::now() + Duration::from_secs(secs);

            let client =
                match TibiaClient::connect(&host, port, &account, &password).await {
                    Ok(c) => c,
                    Err(_) => {
                        m.record_error();
                        m.duration_secs = secs as f64;
                        return m;
                    }
                };

            let mut bot = Bot::new(client, w);
            let bot_start = Instant::now();

            while Instant::now() < deadline {
                if !bot.tick().await {
                    break;
                }
            }

            let mut metrics = bot.finish();
            metrics.duration_secs = bot_start.elapsed().as_secs_f64();
            metrics
        });
    }

    let mut round_metrics = RunMetrics::new();
    while let Some(result) = set.join_next().await {
        let m = result.unwrap_or_else(|_| {
            let mut m = RunMetrics::new();
            m.record_error();
            m.duration_secs = secs as f64;
            m
        });
        merge_metrics(&mut round_metrics, m);
    }
    round_metrics
}
