//! NPC-flood scenario: bots spam common NPC interaction phrases.
//!
//! Bots cycle through "hi", "trade", "bye" SAY packets, simulating a
//! crowd of players interacting with NPCs.

use std::time::{Duration, Instant};

use tokio::task::JoinSet;

use crate::client::TibiaClient;
use crate::metrics::{current_rss_bytes, RunMetrics};
use crate::scenarios::{merge_metrics, ScenarioConfig};

const OP_SAY: u8 = 0x96;
const NPC_PHRASES: &[&str] = &["hi", "trade", "bye"];

/// Run the NPC-flood scenario.
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

            let mut phrase_idx = 0usize;
            while Instant::now() < task_deadline {
                let phrase = NPC_PHRASES[phrase_idx % NPC_PHRASES.len()];
                phrase_idx += 1;

                let text_bytes = phrase.as_bytes();
                let text_len = text_bytes.len() as u16;
                let mut payload = Vec::with_capacity(3 + text_bytes.len());
                payload.push(1u8); // speak_type = TALKTYPE_SAY
                payload.extend_from_slice(&text_len.to_le_bytes());
                payload.extend_from_slice(text_bytes);

                match client.send_packet(OP_SAY, &payload).await {
                    Ok(latency_ms) => m.record_success(latency_ms),
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

    while let Some(result) = set.join_next().await {
        let bot_metrics = result.unwrap_or_else(|_| {
            let mut m = RunMetrics::new();
            m.record_error();
            m.duration_secs = config.duration_secs as f64;
            m
        });
        let rss = current_rss_bytes();
        if rss > combined.peak_rss_bytes {
            combined.peak_rss_bytes = rss;
        }
        merge_metrics(&mut combined, bot_metrics);
    }

    combined.rss_end_bytes = current_rss_bytes();
    combined.peak_rss_bytes = combined.peak_rss_bytes.max(combined.rss_end_bytes);
    combined.duration_secs = run_start.elapsed().as_secs_f64().max(1.0);

    Ok(combined)
}
