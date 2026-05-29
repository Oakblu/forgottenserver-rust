//! Full-chaos scenario: maximum variety of packet types.
//!
//! Uses `ActionWeights::full_chaos()` which spreads traffic across all
//! action types with spellcasting weighted slightly higher.

use crate::bot::ActionWeights;
use crate::metrics::RunMetrics;
use crate::scenarios::{run_bots, ScenarioConfig};

/// Run the full-chaos scenario.
pub async fn run(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    run_bots(config, ActionWeights::full_chaos()).await
}
