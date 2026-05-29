//! Scenario modules for the perf-bot load tester.
//!
//! Each scenario implements:
//! ```rust,ignore
//! pub async fn run(config: &ScenarioConfig) -> anyhow::Result<RunMetrics>
//! ```
//!
//! Scenarios are orchestrated by `main.rs` based on the `--scenario` flag.

pub mod breaking_point;
pub mod crowd;
pub mod full_chaos;
pub mod login_flood;
pub mod magic_storm;
pub mod monster_swarm;
pub mod npc_flood;
pub mod sustained_load;

use std::time::{Duration, Instant};

use anyhow::anyhow;
use tokio::task::JoinSet;

use crate::bot::{ActionWeights, Bot};
use crate::client::TibiaClient;
use crate::metrics::{current_rss_bytes, RunMetrics};

// ---------------------------------------------------------------------------
// ScenarioConfig
// ---------------------------------------------------------------------------

/// Common configuration passed to every scenario.
#[derive(Clone)]
pub struct ScenarioConfig {
    /// Server hostname or IP.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// Account name prefix; accounts will be named `"{prefix}1"`, `"{prefix}2"`, …
    pub account_prefix: String,
    /// Password used for all accounts.
    pub password: String,
    /// Number of concurrent bots to spawn.
    pub bot_count: usize,
    /// How long to run the scenario (seconds).
    pub duration_secs: u64,
}

// ---------------------------------------------------------------------------
// Shared helper: merge two RunMetrics
// ---------------------------------------------------------------------------

/// Merge `src` into `dst`: sum counters, add histogram values, take max RSS.
pub fn merge_metrics(dst: &mut RunMetrics, src: RunMetrics) {
    dst.successes += src.successes;
    dst.errors += src.errors;
    dst.peak_rss_bytes = dst.peak_rss_bytes.max(src.peak_rss_bytes);
    let _ = dst.latency_ms.add(&src.latency_ms);
}

// ---------------------------------------------------------------------------
// Shared helper: run_bots
// ---------------------------------------------------------------------------

/// Spawn `config.bot_count` tokio tasks.  Each task connects with account
/// `"{account_prefix}{n}"`, creates a `Bot` with `weights`, and calls
/// `bot.tick()` in a loop until `config.duration_secs` elapse or the bot
/// disconnects.
///
/// Collects all per-bot `RunMetrics` and merges them into a single result.
pub async fn run_bots(
    config: &ScenarioConfig,
    weights: ActionWeights,
) -> anyhow::Result<RunMetrics> {
    if config.bot_count == 0 {
        return Err(anyhow!("bot_count must be > 0"));
    }

    let rss_start = current_rss_bytes();

    let mut set: JoinSet<RunMetrics> = JoinSet::new();

    for i in 1..=config.bot_count {
        let host = config.host.clone();
        let port = config.port;
        let account = format!("{}{i}", config.account_prefix);
        let password = config.password.clone();
        let w = weights.clone();
        let dur = config.duration_secs;

        set.spawn(async move {
            // Connect — best-effort; if the server is not reachable we record
            // all ticks as errors and return empty metrics.
            let client = match TibiaClient::connect(&host, port, &account, &password).await {
                Ok(c) => c,
                Err(_) => {
                    let mut m = RunMetrics::new();
                    m.record_error();
                    m.duration_secs = dur as f64;
                    return m;
                }
            };

            let mut bot = Bot::new(client, w);
            let bot_start = Instant::now();
            let bot_deadline = bot_start + Duration::from_secs(dur);

            while Instant::now() < bot_deadline {
                if !bot.tick().await {
                    break;
                }
            }

            let mut metrics = bot.finish();
            metrics.duration_secs = bot_start.elapsed().as_secs_f64();
            metrics
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
        // Track RSS periodically while collecting
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
