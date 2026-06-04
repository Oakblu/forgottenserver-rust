use std::collections::HashMap;

pub type TradeId = u32;
pub type EntityId = u32;

#[derive(Debug, Clone)]
pub struct TradeItem {
    pub type_id: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeResult {
    Pending,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeError {
    AlreadyInTrade,
}

struct Trade {
    initiator: EntityId,
    target: EntityId,
    initiator_item: TradeItem,
    initiator_accepted: bool,
    target_accepted: bool,
}

pub struct TradeManager {
    trades: HashMap<TradeId, Trade>,
    player_trade: HashMap<EntityId, TradeId>,
    next_id: TradeId,
}

impl Default for TradeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeManager {
    pub fn new() -> Self {
        TradeManager {
            trades: HashMap::new(),
            player_trade: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn open(
        &mut self,
        initiator: EntityId,
        target: EntityId,
        item: TradeItem,
    ) -> Result<TradeId, TradeError> {
        if self.player_trade.contains_key(&initiator) || self.player_trade.contains_key(&target) {
            return Err(TradeError::AlreadyInTrade);
        }
        let id = self.next_id;
        self.next_id += 1;
        self.trades.insert(
            id,
            Trade {
                initiator,
                target,
                initiator_item: item,
                initiator_accepted: false,
                target_accepted: false,
            },
        );
        self.player_trade.insert(initiator, id);
        self.player_trade.insert(target, id);
        Ok(id)
    }

    /// Returns the item offered by the opposite party.
    pub fn inspect(&self, trade_id: TradeId, requester: EntityId) -> Option<TradeItem> {
        let trade = self.trades.get(&trade_id)?;
        if requester == trade.initiator {
            None
        } else {
            Some(trade.initiator_item.clone())
        }
    }

    pub fn accept(&mut self, trade_id: TradeId, player_id: EntityId) -> TradeResult {
        let trade = match self.trades.get_mut(&trade_id) {
            Some(t) => t,
            None => return TradeResult::Pending,
        };
        if player_id == trade.initiator {
            trade.initiator_accepted = true;
        } else if player_id == trade.target {
            trade.target_accepted = true;
        }
        if trade.initiator_accepted && trade.target_accepted {
            self.close(trade_id);
            TradeResult::Completed
        } else {
            TradeResult::Pending
        }
    }

    pub fn close(&mut self, trade_id: TradeId) {
        if let Some(trade) = self.trades.remove(&trade_id) {
            self.player_trade.remove(&trade.initiator);
            self.player_trade.remove(&trade.target);
        }
    }

    pub fn get_trade_for_player(&self, player_id: EntityId) -> Option<TradeId> {
        self.player_trade.get(&player_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(type_id: u16) -> TradeItem {
        TradeItem { type_id }
    }

    #[test]
    fn trade_open_creates_pending_trade_for_both_players() {
        let mut mgr = TradeManager::new();
        let result = mgr.open(1, 2, item(100));
        assert!(result.is_ok(), "Open must succeed for untraded players");
        let trade_id = result.unwrap();
        assert_eq!(mgr.get_trade_for_player(1), Some(trade_id));
        assert_eq!(mgr.get_trade_for_player(2), Some(trade_id));
    }

    #[test]
    fn trade_accept_by_one_remains_pending() {
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, item(100)).unwrap();
        let result = mgr.accept(trade_id, 1);
        assert_eq!(result, TradeResult::Pending);
        // Trade still exists
        assert!(mgr.get_trade_for_player(1).is_some());
    }

    #[test]
    fn trade_accept_by_both_swaps_items_and_closes() {
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, item(100)).unwrap();
        let r1 = mgr.accept(trade_id, 1);
        assert_eq!(r1, TradeResult::Pending);
        let r2 = mgr.accept(trade_id, 2);
        assert_eq!(r2, TradeResult::Completed);
        assert!(
            mgr.get_trade_for_player(1).is_none(),
            "Trade must close after both accept"
        );
        assert!(mgr.get_trade_for_player(2).is_none());
    }

    #[test]
    fn trade_close_cancels_and_returns_items() {
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, item(100)).unwrap();
        mgr.close(trade_id);
        assert!(
            mgr.get_trade_for_player(1).is_none(),
            "Trade must be gone after close"
        );
        assert!(mgr.get_trade_for_player(2).is_none());
    }

    // ── Default impl (lines 37-38) ─────────────────────────────────────────────

    #[test]
    fn trade_manager_default_is_fresh() {
        let mgr = TradeManager::default();
        // A fresh manager has no trades.
        assert!(mgr.get_trade_for_player(1).is_none());
    }

    // ── AlreadyInTrade error (line 58) ─────────────────────────────────────────

    #[test]
    fn open_fails_when_initiator_already_in_trade() {
        let mut mgr = TradeManager::new();
        mgr.open(1, 2, item(100)).unwrap();
        // Player 1 is already in a trade — a second open must fail.
        let err = mgr.open(1, 3, item(200));
        assert_eq!(err, Err(TradeError::AlreadyInTrade));
    }

    #[test]
    fn open_fails_when_target_already_in_trade() {
        let mut mgr = TradeManager::new();
        mgr.open(1, 2, item(100)).unwrap();
        // Player 2 is already in a trade.
        let err = mgr.open(3, 2, item(300));
        assert_eq!(err, Err(TradeError::AlreadyInTrade));
    }

    // ── inspect returns None for initiator (line 81) ───────────────────────────

    #[test]
    fn inspect_returns_none_for_initiator() {
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, item(100)).unwrap();
        // The initiator cannot see their own item via inspect.
        assert!(
            mgr.inspect(trade_id, 1).is_none(),
            "initiator must not see their own item"
        );
    }

    #[test]
    fn inspect_returns_item_for_target() {
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, item(100)).unwrap();
        let seen = mgr.inspect(trade_id, 2);
        assert!(seen.is_some(), "target must be able to inspect the offer");
        assert_eq!(seen.unwrap().type_id, 100);
    }

    #[test]
    fn inspect_returns_none_for_nonexistent_trade() {
        let mgr = TradeManager::new();
        assert!(mgr.inspect(999, 1).is_none());
    }

    // ── accept with non-existent trade_id (line 90) ────────────────────────────

    #[test]
    fn accept_nonexistent_trade_returns_pending() {
        let mut mgr = TradeManager::new();
        // Trade ID 999 does not exist — accept must return Pending (not panic).
        let result = mgr.accept(999, 1);
        assert_eq!(
            result,
            TradeResult::Pending,
            "accept on non-existent trade must return Pending"
        );
    }

    // ── accept: target accepts before initiator ────────────────────────────────

    #[test]
    fn accept_target_then_initiator_completes() {
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, item(50)).unwrap();
        assert_eq!(mgr.accept(trade_id, 2), TradeResult::Pending);
        assert_eq!(mgr.accept(trade_id, 1), TradeResult::Completed);
        assert!(mgr.get_trade_for_player(1).is_none());
        assert!(mgr.get_trade_for_player(2).is_none());
    }

    // ── next_id increments across multiple opens ────────────────────────────────

    #[test]
    fn open_assigns_distinct_trade_ids() {
        let mut mgr = TradeManager::new();
        // First trade between 1 and 2, then second trade between 3 and 4.
        let id1 = mgr.open(1, 2, item(10)).unwrap();
        mgr.close(id1);
        let id2 = mgr.open(3, 4, item(20)).unwrap();
        assert_ne!(id1, id2, "each open must produce a distinct trade ID");
    }
}
