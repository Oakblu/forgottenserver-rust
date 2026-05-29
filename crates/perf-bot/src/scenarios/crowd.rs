//! Crowd scenario: walk/chat/combat specializations.
//!
//! Three entry points running bots with walk-only, chat-only, or
//! combat-only action weights.

use crate::bot::ActionWeights;
use crate::metrics::RunMetrics;
use crate::scenarios::{run_bots, ScenarioConfig};

/// Crowd scenario with pure walking bots.
pub async fn crowd_walk(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    run_bots(config, ActionWeights::walking()).await
}

/// Crowd scenario with pure chat bots.
pub async fn crowd_chat(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    run_bots(config, ActionWeights::chatting()).await
}

/// Crowd scenario with pure combat bots.
pub async fn crowd_combat(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    run_bots(config, ActionWeights::combat()).await
}

/// Default crowd entry point: runs mixed walk/chat/combat (uses `mixed()` weights).
pub async fn run(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    run_bots(config, ActionWeights::mixed()).await
}
