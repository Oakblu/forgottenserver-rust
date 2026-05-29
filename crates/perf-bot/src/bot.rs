//! Single bot instance running against a connected server.
//!
//! A `Bot` wraps a `TibiaClient`, picks random game actions weighted by
//! `ActionWeights`, sends the corresponding packets, and records each
//! outcome into `RunMetrics`.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::client::TibiaClient;
use crate::metrics::RunMetrics;

// ---------------------------------------------------------------------------
// Action opcodes (verified from protocolgame.rs)
// ---------------------------------------------------------------------------

const OP_WALK_NORTH: u8 = 0x65;
const OP_WALK_EAST: u8 = 0x66;
const OP_WALK_SOUTH: u8 = 0x67;
const OP_WALK_WEST: u8 = 0x68;
const OP_SAY: u8 = 0x96;
const OP_ATTACK: u8 = 0xA1;
const OP_USE_ITEM: u8 = 0x82;
const OP_LOOK_AT: u8 = 0x8C;

// ---------------------------------------------------------------------------
// ActionWeights
// ---------------------------------------------------------------------------

/// Relative weights for each action type in a scenario.
///
/// The bot picks a random action each tick proportional to the weight value.
/// A weight of 0 means the action is never chosen.
#[derive(Clone, Debug)]
pub struct ActionWeights {
    pub walk: u32,
    pub chat: u32,
    pub attack: u32,
    pub cast_spell: u32,
    pub use_item: u32,
    pub look: u32,
}

impl ActionWeights {
    /// All zeros except `walk = 1` — effectively an idle bot that does
    /// only minimal walk traffic.
    pub fn idle() -> Self {
        ActionWeights {
            walk: 1,
            chat: 0,
            attack: 0,
            cast_spell: 0,
            use_item: 0,
            look: 0,
        }
    }

    /// Pure walking — 100% walk actions.
    pub fn walking() -> Self {
        ActionWeights {
            walk: 100,
            chat: 0,
            attack: 0,
            cast_spell: 0,
            use_item: 0,
            look: 0,
        }
    }

    /// Pure chat — 100% chat actions.
    pub fn chatting() -> Self {
        ActionWeights {
            walk: 0,
            chat: 100,
            attack: 0,
            cast_spell: 0,
            use_item: 0,
            look: 0,
        }
    }

    /// Pure combat — 100% attack actions.
    pub fn combat() -> Self {
        ActionWeights {
            walk: 0,
            chat: 0,
            attack: 100,
            cast_spell: 0,
            use_item: 0,
            look: 0,
        }
    }

    /// Pure spellcasting — 100% cast_spell actions.
    pub fn spellcasting() -> Self {
        ActionWeights {
            walk: 0,
            chat: 0,
            attack: 0,
            cast_spell: 100,
            use_item: 0,
            look: 0,
        }
    }

    /// Mixed realistic workload.
    pub fn mixed() -> Self {
        ActionWeights {
            walk: 30,
            chat: 15,
            attack: 20,
            cast_spell: 25,
            use_item: 10,
            look: 0,
        }
    }

    /// Full chaos — like mixed but with higher spellcasting weight.
    pub fn full_chaos() -> Self {
        ActionWeights {
            walk: 25,
            chat: 15,
            attack: 20,
            cast_spell: 30,
            use_item: 10,
            look: 0,
        }
    }

    /// Total weight (sum of all action weights).
    fn total(&self) -> u32 {
        self.walk + self.chat + self.attack + self.cast_spell + self.use_item + self.look
    }
}

// ---------------------------------------------------------------------------
// Action enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Action {
    Walk,
    Chat,
    Attack,
    CastSpell,
    UseItem,
    Look,
}

impl ActionWeights {
    /// Pick a random action according to the configured weights.
    fn pick(&self, rng: &mut SmallRng) -> Option<Action> {
        let total = self.total();
        if total == 0 {
            return None;
        }
        let roll = rng.gen_range(0..total);
        let mut cumulative = 0u32;
        if self.walk > 0 {
            cumulative += self.walk;
            if roll < cumulative {
                return Some(Action::Walk);
            }
        }
        if self.chat > 0 {
            cumulative += self.chat;
            if roll < cumulative {
                return Some(Action::Chat);
            }
        }
        if self.attack > 0 {
            cumulative += self.attack;
            if roll < cumulative {
                return Some(Action::Attack);
            }
        }
        if self.cast_spell > 0 {
            cumulative += self.cast_spell;
            if roll < cumulative {
                return Some(Action::CastSpell);
            }
        }
        if self.use_item > 0 {
            cumulative += self.use_item;
            if roll < cumulative {
                return Some(Action::UseItem);
            }
        }
        if self.look > 0 {
            // Look is the final bucket — catch all remaining rolls
            return Some(Action::Look);
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Bot
// ---------------------------------------------------------------------------

/// One bot instance running against a connected server.
pub struct Bot {
    client: TibiaClient,
    weights: ActionWeights,
    metrics: RunMetrics,
    rng: SmallRng,
}

impl Bot {
    /// Create a new bot wrapping the given connected client.
    pub fn new(client: TibiaClient, weights: ActionWeights) -> Self {
        Bot {
            client,
            weights,
            metrics: RunMetrics::new(),
            rng: SmallRng::from_entropy(),
        }
    }

    /// Execute one random action tick.
    ///
    /// Returns `false` if the connection has dropped and the bot should stop.
    pub async fn tick(&mut self) -> bool {
        let action = match self.weights.pick(&mut self.rng) {
            Some(a) => a,
            None => return false,
        };

        let result = self.execute_action(action).await;
        match result {
            Ok(latency_ms) => {
                self.metrics.record_success(latency_ms);
                true
            }
            Err(_) => {
                self.metrics.record_error();
                // Connection-level errors stop this bot
                false
            }
        }
    }

    /// Consume `self` and return the collected metrics.
    pub fn finish(self) -> RunMetrics {
        self.metrics
    }

    // -----------------------------------------------------------------------
    // Private: action dispatch
    // -----------------------------------------------------------------------

    async fn execute_action(&mut self, action: Action) -> anyhow::Result<u64> {
        match action {
            Action::Walk => self.send_walk().await,
            Action::Chat => self.send_chat("hello").await,
            Action::Attack => self.send_attack(0).await,
            Action::CastSpell => self.send_chat("exori").await,
            Action::UseItem => self.send_use_item().await,
            Action::Look => self.send_look().await,
        }
    }

    /// Send a random cardinal direction walk packet.
    async fn send_walk(&mut self) -> anyhow::Result<u64> {
        let dir_opcode = match self.rng.gen_range(0..4u8) {
            0 => OP_WALK_NORTH,
            1 => OP_WALK_EAST,
            2 => OP_WALK_SOUTH,
            _ => OP_WALK_WEST,
        };
        // Walk opcodes have no payload — the opcode IS the direction
        self.client.send_packet(dir_opcode, &[]).await
    }

    /// Send a say packet: speak_type(u8) + text(u16-len-prefixed string).
    async fn send_chat(&mut self, text: &str) -> anyhow::Result<u64> {
        let text_bytes = text.as_bytes();
        let text_len = text_bytes.len() as u16;
        let mut payload = Vec::with_capacity(3 + text_bytes.len());
        payload.push(1u8); // speak_type = TALKTYPE_SAY
        payload.extend_from_slice(&text_len.to_le_bytes());
        payload.extend_from_slice(text_bytes);
        self.client.send_packet(OP_SAY, &payload).await
    }

    /// Send an attack packet: creature_id (u32 LE). 0 = no target (server ignores).
    async fn send_attack(&mut self, creature_id: u32) -> anyhow::Result<u64> {
        self.client
            .send_packet(OP_ATTACK, &creature_id.to_le_bytes())
            .await
    }

    /// Send a use-item packet targeting a dummy position.
    ///
    /// Wire format: pos_x(u16) + pos_y(u16) + pos_z(u8) + item_id(u16) + index(u8)
    async fn send_use_item(&mut self) -> anyhow::Result<u64> {
        let pos_x: u16 = 100;
        let pos_y: u16 = 100;
        let pos_z: u8 = 7;
        let sprite_id: u16 = 2160;
        let index: u8 = 0;

        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&pos_x.to_le_bytes());
        payload.extend_from_slice(&pos_y.to_le_bytes());
        payload.push(pos_z);
        payload.extend_from_slice(&sprite_id.to_le_bytes());
        payload.push(index);
        self.client.send_packet(OP_USE_ITEM, &payload).await
    }

    /// Send a look-at packet targeting a dummy position.
    ///
    /// Wire format: pos_x(u16) + pos_y(u16) + pos_z(u8) + item_id(u16) + stack_pos(u8)
    async fn send_look(&mut self) -> anyhow::Result<u64> {
        let pos_x: u16 = 100;
        let pos_y: u16 = 100;
        let pos_z: u8 = 7;
        let sprite_id: u16 = 2160;
        let stack_pos: u8 = 0;

        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&pos_x.to_le_bytes());
        payload.extend_from_slice(&pos_y.to_le_bytes());
        payload.push(pos_z);
        payload.extend_from_slice(&sprite_id.to_le_bytes());
        payload.push(stack_pos);
        self.client.send_packet(OP_LOOK_AT, &payload).await
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_weights_total() {
        let w = ActionWeights::mixed();
        assert_eq!(w.total(), 30 + 15 + 20 + 25 + 10);
    }

    #[test]
    fn action_weights_idle_walk_only() {
        let w = ActionWeights::idle();
        assert_eq!(w.walk, 1);
        assert_eq!(w.total(), 1);
    }

    #[test]
    fn action_weights_zero_total_returns_none() {
        let w = ActionWeights {
            walk: 0,
            chat: 0,
            attack: 0,
            cast_spell: 0,
            use_item: 0,
            look: 0,
        };
        let mut rng = SmallRng::seed_from_u64(42);
        assert!(w.pick(&mut rng).is_none());
    }

    #[test]
    fn action_weights_walking_always_walks() {
        let w = ActionWeights::walking();
        let mut rng = SmallRng::seed_from_u64(0);
        for _ in 0..20 {
            let action = w.pick(&mut rng).unwrap();
            assert!(matches!(action, Action::Walk));
        }
    }

    #[test]
    fn action_weights_combat_always_attacks() {
        let w = ActionWeights::combat();
        let mut rng = SmallRng::seed_from_u64(0);
        for _ in 0..20 {
            let action = w.pick(&mut rng).unwrap();
            assert!(matches!(action, Action::Attack));
        }
    }

    #[test]
    fn full_chaos_weights_are_valid() {
        let w = ActionWeights::full_chaos();
        assert_eq!(w.walk, 25);
        assert_eq!(w.cast_spell, 30);
        assert!(w.total() > 0);
    }
}
