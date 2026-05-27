use std::collections::HashMap;

pub type EntityId = u32;

#[derive(Debug, Clone)]
pub struct MarketOffer {
    pub player_id: EntityId,
    pub item_id: u16,
    pub amount: u16,
    pub price: u32,
}

pub struct MarketManager {
    buy_offers: Vec<MarketOffer>,
    sell_offers: Vec<MarketOffer>,
    player_sessions: HashMap<EntityId, bool>,
}

impl Default for MarketManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketManager {
    pub fn new() -> Self {
        MarketManager {
            buy_offers: Vec::new(),
            sell_offers: Vec::new(),
            player_sessions: HashMap::new(),
        }
    }

    pub fn place_buy_offer(&mut self, player_id: EntityId, item_id: u16, amount: u16, price: u32) {
        self.player_sessions.insert(player_id, true);
        self.buy_offers.push(MarketOffer {
            player_id,
            item_id,
            amount,
            price,
        });
    }

    pub fn place_sell_offer(&mut self, player_id: EntityId, item_id: u16, amount: u16, price: u32) {
        self.player_sessions.insert(player_id, true);
        self.sell_offers.push(MarketOffer {
            player_id,
            item_id,
            amount,
            price,
        });
    }

    /// Returns (buy_offers, sell_offers) for the given item.
    pub fn list_offers(&self, item_id: u16) -> (Vec<&MarketOffer>, Vec<&MarketOffer>) {
        let buys = self
            .buy_offers
            .iter()
            .filter(|o| o.item_id == item_id)
            .collect();
        let sells = self
            .sell_offers
            .iter()
            .filter(|o| o.item_id == item_id)
            .collect();
        (buys, sells)
    }

    pub fn close_session(&mut self, player_id: EntityId) {
        self.player_sessions.remove(&player_id);
    }

    pub fn has_session(&self, player_id: EntityId) -> bool {
        self.player_sessions.contains_key(&player_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_place_buy_offer_creates_pending_offer() {
        let mut mgr = MarketManager::new();
        mgr.place_buy_offer(1, 500, 10, 1000);
        let (buys, _) = mgr.list_offers(500);
        assert_eq!(buys.len(), 1);
        assert_eq!(buys[0].player_id, 1);
        assert_eq!(buys[0].amount, 10);
    }

    #[test]
    fn market_list_offers_returns_matching_buy_and_sell() {
        let mut mgr = MarketManager::new();
        mgr.place_buy_offer(1, 200, 5, 500);
        mgr.place_sell_offer(2, 200, 3, 450);
        mgr.place_buy_offer(3, 999, 1, 100); // different item
        let (buys, sells) = mgr.list_offers(200);
        assert_eq!(buys.len(), 1, "One buy offer for item 200");
        assert_eq!(sells.len(), 1, "One sell offer for item 200");
    }

    #[test]
    fn market_close_session_removes_player_session() {
        let mut mgr = MarketManager::new();
        mgr.place_buy_offer(1, 100, 1, 10);
        assert!(mgr.has_session(1));
        mgr.close_session(1);
        assert!(!mgr.has_session(1), "Session must be gone after close");
    }
}
