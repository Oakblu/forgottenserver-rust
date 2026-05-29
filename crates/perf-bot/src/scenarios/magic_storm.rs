//! Magic-storm scenario: pure spellcasting bots.
//!
//! All bots exclusively cast spells (send SAY packets with spell words).

use crate::bot::ActionWeights;
use crate::metrics::RunMetrics;
use crate::scenarios::{run_bots, ScenarioConfig};

/// Run the magic-storm scenario.
pub async fn run(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    run_bots(config, ActionWeights::spellcasting()).await
}
