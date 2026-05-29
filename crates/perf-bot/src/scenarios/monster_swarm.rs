//! Monster-swarm scenario: attack-heavy bots.
//!
//! Simulates a swarm of monsters/players in combat: heavy attack traffic
//! with some walking and spellcasting.

use crate::bot::ActionWeights;
use crate::metrics::RunMetrics;
use crate::scenarios::{run_bots, ScenarioConfig};

/// Run the monster-swarm scenario.
pub async fn run(config: &ScenarioConfig) -> anyhow::Result<RunMetrics> {
    let weights = ActionWeights {
        walk: 20,
        attack: 60,
        cast_spell: 20,
        chat: 0,
        use_item: 0,
        look: 0,
    };
    run_bots(config, weights).await
}
