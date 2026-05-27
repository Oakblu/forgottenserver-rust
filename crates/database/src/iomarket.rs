// ── Offer types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum OfferType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OfferState {
    Active,
    Accepted,
    /// AcceptedEx is normalised to Accepted when reading history (C++ compat).
    AcceptedEx,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct MarketOffer {
    pub id: u32,
    pub player_guid: u32,
    pub offer_type: OfferType,
    pub item_type_id: u16,
    pub amount: u16,
    pub price: u64,
    pub created_at: i64,
    pub anonymous: bool,
    /// Player name resolved from player_guid; "Anonymous" when `anonymous` is true.
    pub player_name: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct HistoryMarketOffer {
    pub player_guid: u32,
    pub offer_type: OfferType,
    pub item_type_id: u16,
    pub amount: u16,
    pub price: u64,
    /// timestamp = created_at + market_offer_duration
    pub expires_at: i64,
    pub state: OfferState,
}

#[derive(Debug, Clone, Default)]
pub struct MarketStatistics {
    pub num_transactions: u32,
    pub lowest_price: u64,
    pub highest_price: u64,
    pub total_price: u64,
}

// ── IOMarket ──────────────────────────────────────────────────────────────────

/// Handles the in-game market (buy/sell offers).
///
/// The C++ implementation issues SQL via `Database`.  This Rust version keeps
/// in-memory `Vec`s so tests need no external database.
pub struct IoMarket {
    offers: Vec<MarketOffer>,
    history: Vec<HistoryMarketOffer>,
    next_id: u32,
    /// Duration in seconds before an offer expires (C++: MARKET_OFFER_DURATION).
    pub offer_duration: i64,
}

impl IoMarket {
    pub fn new() -> Self {
        Self {
            offers: Vec::new(),
            history: Vec::new(),
            next_id: 1,
            offer_duration: 2_592_000, // 30 days in seconds (typical default)
        }
    }

    // ── C++ `createOffer` ────────────────────────────────────────────────────

    /// Add a new offer to the market.  Returns the assigned offer id.
    ///
    /// Maps to C++ `createOffer(playerId, action, itemId, amount, price, anonymous)`.
    pub fn create_offer(
        &mut self,
        player_guid: u32,
        offer_type: OfferType,
        item_type_id: u16,
        amount: u16,
        price: u64,
        created_at: i64,
    ) -> u32 {
        self.create_offer_named(
            player_guid,
            offer_type,
            item_type_id,
            amount,
            price,
            created_at,
            false,
            String::new(),
        )
    }

    /// Extended variant that also sets `anonymous` and `player_name`.
    #[allow(clippy::too_many_arguments)]
    pub fn create_offer_named(
        &mut self,
        player_guid: u32,
        offer_type: OfferType,
        item_type_id: u16,
        amount: u16,
        price: u64,
        created_at: i64,
        anonymous: bool,
        player_name: String,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let display_name = if anonymous {
            "Anonymous".to_string()
        } else {
            player_name
        };
        self.offers.push(MarketOffer {
            id,
            player_guid,
            offer_type,
            item_type_id,
            amount,
            price,
            created_at,
            anonymous,
            player_name: display_name,
            active: true,
        });
        id
    }

    // ── C++ `getActiveOffers(action, itemId)` ────────────────────────────────

    /// Return all active offers for a specific item type filtered by offer type.
    ///
    /// Maps to C++ `getActiveOffers(action, itemId)`.
    /// Anonymous offers show `player_name` as "Anonymous".
    pub fn get_active_offers_for_item(
        &self,
        offer_type: &OfferType,
        item_type_id: u16,
    ) -> Vec<&MarketOffer> {
        self.offers
            .iter()
            .filter(|o| o.active && o.offer_type == *offer_type && o.item_type_id == item_type_id)
            .collect()
    }

    /// Return all currently active offers (both buy and sell, all items).
    ///
    /// Kept for backwards compatibility with existing callers.
    pub fn get_active_offers(&self) -> Vec<&MarketOffer> {
        self.offers.iter().filter(|o| o.active).collect()
    }

    /// Return all offers for a specific item type (regardless of offer_type / active state).
    ///
    /// Kept for backwards compatibility.
    pub fn get_offers_for_item(&self, item_type_id: u16) -> Vec<&MarketOffer> {
        self.offers
            .iter()
            .filter(|o| o.item_type_id == item_type_id)
            .collect()
    }

    // ── C++ `getOwnOffers(action, playerId)` ─────────────────────────────────

    /// Return all active offers for a specific player filtered by offer type.
    ///
    /// Maps to C++ `getOwnOffers(action, playerId)`.
    pub fn get_own_offers(&self, offer_type: &OfferType, player_guid: u32) -> Vec<&MarketOffer> {
        self.offers
            .iter()
            .filter(|o| o.active && o.player_guid == player_guid && o.offer_type == *offer_type)
            .collect()
    }

    // ── C++ `getOwnHistory(action, playerId)` ─────────────────────────────────

    /// Return history entries for a player filtered by offer type.
    ///
    /// Maps to C++ `getOwnHistory(action, playerId)`.
    /// C++ normalises `OFFERSTATE_ACCEPTEDEX` → `OFFERSTATE_ACCEPTED` on read.
    pub fn get_own_history(
        &self,
        offer_type: &OfferType,
        player_guid: u32,
    ) -> Vec<HistoryMarketOffer> {
        self.history
            .iter()
            .filter(|h| h.player_guid == player_guid && h.offer_type == *offer_type)
            .map(|h| {
                let mut entry = h.clone();
                // C++ normalises AcceptedEx → Accepted when reading history
                if entry.state == OfferState::AcceptedEx {
                    entry.state = OfferState::Accepted;
                }
                entry
            })
            .collect()
    }

    // ── C++ `getPlayerOfferCount(playerId)` ──────────────────────────────────

    /// Count all active offers for a given player (both buy and sell).
    ///
    /// Maps to C++ `getPlayerOfferCount(playerId)`.
    pub fn get_player_offer_count(&self, player_guid: u32) -> u32 {
        self.offers
            .iter()
            .filter(|o| o.active && o.player_guid == player_guid)
            .count() as u32
    }

    // ── C++ `getOfferByCounter(timestamp, counter)` ──────────────────────────

    /// Look up an offer by its timestamp and counter (low 16 bits of id).
    ///
    /// Maps to C++ `getOfferByCounter(timestamp, counter)`.
    /// The C++ `created` stored in the DB equals `timestamp - market_offer_duration`,
    /// so we reverse that before comparing.
    pub fn get_offer_by_counter(&self, timestamp: i64, counter: u16) -> Option<&MarketOffer> {
        let created = timestamp - self.offer_duration;
        self.offers
            .iter()
            .find(|o| o.active && o.created_at == created && (o.id as u16) == counter)
    }

    // ── C++ `acceptOffer(offerId, amount)` ───────────────────────────────────

    /// Decrement offer amount by `amount`.  If the resulting amount reaches 0
    /// the offer is deactivated.  Returns `true` if the offer was found.
    ///
    /// Maps to C++ `acceptOffer(offerId, amount)` which does:
    ///   `UPDATE market_offers SET amount = amount - :amount WHERE id = :id`
    ///
    /// The separate `_buyer_guid` parameter is kept for API compatibility but is
    /// not used in the pure in-memory implementation (the C++ also does not store it).
    pub fn accept_offer(&mut self, offer_id: u32, amount: u16) -> bool {
        if let Some(offer) = self
            .offers
            .iter_mut()
            .find(|o| o.id == offer_id && o.active)
        {
            if amount >= offer.amount {
                offer.amount = 0;
                offer.active = false;
            } else {
                offer.amount -= amount;
            }
            true
        } else {
            false
        }
    }

    // ── C++ `deleteOffer(offerId)` ───────────────────────────────────────────

    /// Remove an offer entirely (hard delete).
    ///
    /// Maps to C++ `deleteOffer(offerId)` which issues `DELETE FROM market_offers`.
    /// Returns `true` if the offer was found and removed.
    pub fn delete_offer(&mut self, offer_id: u32) -> bool {
        let before = self.offers.len();
        self.offers.retain(|o| o.id != offer_id);
        self.offers.len() < before
    }

    /// Cancel an offer (soft deactivate).  Kept for backwards compatibility.
    pub fn cancel_offer(&mut self, offer_id: u32) -> bool {
        if let Some(offer) = self.offers.iter_mut().find(|o| o.id == offer_id) {
            offer.active = false;
            true
        } else {
            false
        }
    }

    // ── C++ `appendHistory` ──────────────────────────────────────────────────

    /// Append a record to the market history.
    ///
    /// Maps to C++ `appendHistory(playerId, action, itemId, amount, price, timestamp, state)`.
    #[allow(clippy::too_many_arguments)]
    pub fn append_history(
        &mut self,
        player_guid: u32,
        offer_type: OfferType,
        item_type_id: u16,
        amount: u16,
        price: u64,
        expires_at: i64,
        state: OfferState,
    ) {
        self.history.push(HistoryMarketOffer {
            player_guid,
            offer_type,
            item_type_id,
            amount,
            price,
            expires_at,
            state,
        });
    }

    // ── C++ `moveOfferToHistory(offerId, state)` ─────────────────────────────

    /// Move an active offer to the history with the given state, then delete it.
    ///
    /// Maps to C++ `moveOfferToHistory(offerId, state)`:
    ///   1. SELECT offer
    ///   2. DELETE offer
    ///   3. appendHistory with `created + market_offer_duration` as `expires_at`
    ///
    /// Returns `true` on success, `false` if the offer was not found.
    pub fn move_offer_to_history(&mut self, offer_id: u32, state: OfferState) -> bool {
        let pos = self.offers.iter().position(|o| o.id == offer_id);
        if let Some(idx) = pos {
            let offer = self.offers.remove(idx);
            let expires_at = offer.created_at + self.offer_duration;
            self.append_history(
                offer.player_guid,
                offer.offer_type,
                offer.item_type_id,
                offer.amount,
                offer.price,
                expires_at,
                state,
            );
            true
        } else {
            false
        }
    }

    // ── C++ `checkExpiredOffers` / `processExpiredOffers` ────────────────────

    /// Remove all offers whose `created_at` is strictly less than `cutoff_timestamp`
    /// and move them to history with state `Expired`.
    ///
    /// Maps to C++ `checkExpiredOffers` + `processExpiredOffers`:
    ///   cutoff = now - market_offer_duration
    pub fn cleanup_expired_offers(&mut self, cutoff_timestamp: i64) {
        let expired_ids: Vec<u32> = self
            .offers
            .iter()
            .filter(|o| o.created_at < cutoff_timestamp)
            .map(|o| o.id)
            .collect();
        for id in expired_ids {
            self.move_offer_to_history(id, OfferState::Expired);
        }
    }

    // ── C++ `updateStatistics` / `getPurchaseStatistics` / `getSaleStatistics`

    /// Compute purchase and sale statistics from accepted history entries.
    ///
    /// Maps to C++ `updateStatistics()` + `getPurchaseStatistics` / `getSaleStatistics`.
    /// Returns `(purchase_stats, sale_stats)` keyed by item_type_id.
    pub fn compute_statistics(
        &self,
    ) -> (
        std::collections::HashMap<u16, MarketStatistics>,
        std::collections::HashMap<u16, MarketStatistics>,
    ) {
        let mut purchase: std::collections::HashMap<u16, MarketStatistics> =
            std::collections::HashMap::new();
        let mut sale: std::collections::HashMap<u16, MarketStatistics> =
            std::collections::HashMap::new();

        for entry in &self.history {
            // Only accepted offers count (AcceptedEx also counts in C++)
            if entry.state != OfferState::Accepted && entry.state != OfferState::AcceptedEx {
                continue;
            }

            let stats = match entry.offer_type {
                OfferType::Buy => purchase.entry(entry.item_type_id).or_default(),
                OfferType::Sell => sale.entry(entry.item_type_id).or_default(),
            };

            stats.num_transactions += 1;
            stats.total_price += entry.price;
            if stats.num_transactions == 1 || entry.price < stats.lowest_price {
                stats.lowest_price = entry.price;
            }
            if entry.price > stats.highest_price {
                stats.highest_price = entry.price;
            }
        }

        (purchase, sale)
    }
}

impl Default for IoMarket {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Expired-offer settlement decision helpers (Session 31 ledger closure)
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum stack size used by IOMarket's stackable-delivery loop.
/// Mirrors C++ `ITEM_STACK_SIZE = 100` (game.h).
pub const ITEM_STACK_SIZE: u16 = 100;

/// Routing decision for a single expired-offer row in C++
/// `IOMarket::processExpiredOffers`. The cross-crate caller (game
/// crate) reads each variant and dispatches the matching I/O:
///   * `DeliverStackable` / `DeliverNonStackable` → `Game::internalAddItem`
///     into the seller's inbox container.
///   * `RefundBank` → `Player::setBankBalance` (online) or
///     `IOLoginData::increaseBankBalance` (offline).
///   * `SkipInvalidType` → skip the row entirely (item-type was deleted
///     between offer creation and expiry).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpiredOfferSettlement {
    /// `sale == 1` but `items[itemtype].id == 0` — the item type no
    /// longer exists in the game data. Drop the row.
    SkipInvalidType,
    /// Sale-side delivery for a stackable item. The caller iterates
    /// `plan_stackable_delivery_chunks(total_amount, ITEM_STACK_SIZE)`
    /// and creates one item per chunk.
    DeliverStackable {
        item_type_id: u16,
        total_amount: u16,
    },
    /// Sale-side delivery for a non-stackable item. `sub_type` is
    /// `charges` when the item-type has charges, else `-1` (mirrors
    /// C++ `itemType.charges != 0 ? charges : -1`). Caller creates
    /// `count` individual items.
    DeliverNonStackable {
        item_type_id: u16,
        sub_type: i32,
        count: u16,
    },
    /// Buy-side refund: the offer expired without filling, so the
    /// reserved coins return to the player's bank.
    RefundBank { total_price: u64 },
}

/// Pure decision for `IOMarket::processExpiredOffers` per row. Inputs
/// mirror the columns the C++ SQL row exposes:
///
/// * `is_sale` — `result["sale"] == 1`
/// * `itemtype_id_zero` — `items[result["itemtype"]].id == 0` (only
///   meaningful for the sale branch; ignored when `is_sale == false`)
/// * `stackable` — `items[itemtype].stackable`
/// * `charges` — `items[itemtype].charges` (u32 in C++; non-zero ⇒
///   the non-stackable delivery uses charges as sub_type)
/// * `amount` — `result["amount"]` (u16)
/// * `price` — `result["price"]` (u64; only meaningful for buy refund)
///
/// Returns the matching `ExpiredOfferSettlement` variant. Branch
/// order matches the C++ source exactly.
pub fn expired_offer_settlement(
    is_sale: bool,
    itemtype_id_zero: bool,
    stackable: bool,
    charges: u32,
    amount: u16,
    price: u64,
) -> ExpiredOfferSettlement {
    if is_sale {
        if itemtype_id_zero {
            return ExpiredOfferSettlement::SkipInvalidType;
        }
        if stackable {
            return ExpiredOfferSettlement::DeliverStackable {
                item_type_id: 0, // caller already has the resolved id
                total_amount: amount,
            };
        }
        let sub_type: i32 = if charges != 0 { charges as i32 } else { -1 };
        return ExpiredOfferSettlement::DeliverNonStackable {
            item_type_id: 0, // caller already has the resolved id
            sub_type,
            count: amount,
        };
    }
    ExpiredOfferSettlement::RefundBank {
        total_price: price.saturating_mul(amount as u64),
    }
}

/// Plan the per-item-creation chunk counts for a stackable delivery.
/// Mirrors the C++ chunking loop:
///
/// ```cpp
/// uint16_t tmpAmount = amount;
/// while (tmpAmount > 0) {
///     uint16_t stackCount = std::min<uint16_t>(ITEM_STACK_SIZE, tmpAmount);
///     // CreateItem(id, stackCount); …
///     tmpAmount -= stackCount;
/// }
/// ```
///
/// Returns a vec of stack counts. The sum of the returned counts
/// equals `total_amount`; every element except possibly the last
/// equals `max_per_stack`. `total_amount == 0` returns `vec![]`.
pub fn plan_stackable_delivery_chunks(total_amount: u16, max_per_stack: u16) -> Vec<u16> {
    if total_amount == 0 || max_per_stack == 0 {
        return Vec::new();
    }
    let mut remaining = total_amount;
    let mut chunks = Vec::new();
    while remaining > 0 {
        let chunk = remaining.min(max_per_stack);
        chunks.push(chunk);
        remaining -= chunk;
    }
    chunks
}

/// Pure decision for the post-delivery cleanup branch in C++:
///
/// ```cpp
/// if (player->isOffline()) {
///     IOLoginData::savePlayer(player);
///     delete player;
/// }
/// ```
///
/// Returns `true` when the caller should persist the (loaded-offline)
/// player back to disk. The caller owns the `delete` step (Rust drops
/// the player automatically when the Box goes out of scope).
pub fn should_save_offline_loaded_player(was_offline: bool) -> bool {
    was_offline
}

/// Return the next-reschedule delay (in milliseconds) for the periodic
/// `checkExpiredOffers` task, or `None` when the operator disabled
/// periodic checks. Mirrors C++:
///
/// ```cpp
/// int32_t check = getNumber(ConfigManager::CHECK_EXPIRED_MARKET_OFFERS_EACH_MINUTES);
/// if (check <= 0) return;
/// g_scheduler.addEvent(createSchedulerTask(check * 60 * 1000, &checkExpiredOffers));
/// ```
pub fn expired_offer_check_reschedule_ms(check_each_minutes: i32) -> Option<u64> {
    if check_each_minutes <= 0 {
        return None;
    }
    Some(check_each_minutes as u64 * 60_000)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_700_000_000;
    const DURATION: i64 = 2_592_000; // 30 days

    fn market() -> IoMarket {
        IoMarket::new()
    }

    // ── create_offer ──────────────────────────────────────────────────────────

    #[test]
    fn new_creates_empty_market() {
        let m = market();
        assert!(m.get_active_offers().is_empty());
    }

    #[test]
    fn default_constructs_empty_market_with_default_duration() {
        let m: IoMarket = IoMarket::default();
        assert!(m.get_active_offers().is_empty());
        // Default `offer_duration` matches `IoMarket::new()` (30 days in seconds).
        assert_eq!(m.offer_duration, 2_592_000);
    }

    #[test]
    fn create_offer_returns_offer_id() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        assert_eq!(id, 1);
    }

    #[test]
    fn create_offer_increments_id() {
        let mut m = market();
        let id1 = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        let id2 = m.create_offer(2, OfferType::Buy, 100, 5, 400, NOW);
        assert_ne!(id1, id2);
    }

    #[test]
    fn create_offer_price_is_u64() {
        let mut m = market();
        let large_price: u64 = u64::MAX / 2;
        let id = m.create_offer(1, OfferType::Sell, 100, 1, large_price, NOW);
        let offer = m.get_active_offers();
        assert_eq!(offer[0].id, id);
        assert_eq!(offer[0].price, large_price);
    }

    #[test]
    fn create_offer_named_anonymous_sets_anonymous_player_name() {
        let mut m = market();
        m.create_offer_named(
            1,
            OfferType::Sell,
            100,
            10,
            500,
            NOW,
            true,
            "Alice".to_string(),
        );
        let active = m.get_active_offers();
        assert_eq!(active[0].player_name, "Anonymous");
        assert!(active[0].anonymous);
    }

    #[test]
    fn create_offer_named_non_anonymous_preserves_player_name() {
        let mut m = market();
        m.create_offer_named(
            1,
            OfferType::Sell,
            100,
            10,
            500,
            NOW,
            false,
            "Alice".to_string(),
        );
        let active = m.get_active_offers();
        assert_eq!(active[0].player_name, "Alice");
        assert!(!active[0].anonymous);
    }

    // ── get_active_offers_for_item ────────────────────────────────────────────

    #[test]
    fn get_active_offers_for_item_filters_by_type_and_item() {
        let mut m = market();
        m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.create_offer(2, OfferType::Buy, 100, 5, 400, NOW);
        m.create_offer(3, OfferType::Sell, 200, 1, 1000, NOW);

        let sell_100 = m.get_active_offers_for_item(&OfferType::Sell, 100);
        assert_eq!(sell_100.len(), 1);
        assert_eq!(sell_100[0].player_guid, 1);

        let buy_100 = m.get_active_offers_for_item(&OfferType::Buy, 100);
        assert_eq!(buy_100.len(), 1);
        assert_eq!(buy_100[0].player_guid, 2);
    }

    #[test]
    fn get_active_offers_for_item_excludes_inactive() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.cancel_offer(id);

        assert!(m
            .get_active_offers_for_item(&OfferType::Sell, 100)
            .is_empty());
    }

    #[test]
    fn get_active_offers_for_item_returns_empty_for_unknown_item() {
        let m = market();
        assert!(m
            .get_active_offers_for_item(&OfferType::Sell, 999)
            .is_empty());
    }

    // ── get_offers_for_item (backwards compat) ────────────────────────────────

    #[test]
    fn get_offers_for_item_returns_matching_offers() {
        let mut m = market();
        m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.create_offer(2, OfferType::Buy, 100, 5, 400, NOW);
        m.create_offer(3, OfferType::Sell, 200, 1, 1000, NOW);

        let offers = m.get_offers_for_item(100);
        assert_eq!(offers.len(), 2);
    }

    #[test]
    fn get_offers_for_item_returns_empty_for_unknown_item() {
        let m = market();
        assert!(m.get_offers_for_item(999).is_empty());
    }

    // ── get_own_offers ────────────────────────────────────────────────────────

    #[test]
    fn get_own_offers_returns_only_player_offers_of_given_type() {
        let mut m = market();
        m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.create_offer(1, OfferType::Buy, 200, 3, 200, NOW);
        m.create_offer(2, OfferType::Sell, 100, 5, 450, NOW);

        let own_sell = m.get_own_offers(&OfferType::Sell, 1);
        assert_eq!(own_sell.len(), 1);
        assert_eq!(own_sell[0].item_type_id, 100);

        let own_buy = m.get_own_offers(&OfferType::Buy, 1);
        assert_eq!(own_buy.len(), 1);
        assert_eq!(own_buy[0].item_type_id, 200);
    }

    #[test]
    fn get_own_offers_excludes_inactive_offers() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.cancel_offer(id);

        assert!(m.get_own_offers(&OfferType::Sell, 1).is_empty());
    }

    #[test]
    fn get_own_offers_returns_empty_for_unknown_player() {
        let m = market();
        assert!(m.get_own_offers(&OfferType::Sell, 999).is_empty());
    }

    // ── get_player_offer_count ────────────────────────────────────────────────

    #[test]
    fn get_player_offer_count_returns_total_active_offers() {
        let mut m = market();
        m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.create_offer(1, OfferType::Buy, 200, 3, 200, NOW);
        m.create_offer(2, OfferType::Sell, 100, 5, 450, NOW);

        assert_eq!(m.get_player_offer_count(1), 2);
        assert_eq!(m.get_player_offer_count(2), 1);
    }

    #[test]
    fn get_player_offer_count_excludes_inactive() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.cancel_offer(id);

        assert_eq!(m.get_player_offer_count(1), 0);
    }

    #[test]
    fn get_player_offer_count_returns_zero_for_unknown_player() {
        let m = market();
        assert_eq!(m.get_player_offer_count(999), 0);
    }

    // ── get_offer_by_counter ──────────────────────────────────────────────────

    #[test]
    fn get_offer_by_counter_finds_offer_with_matching_timestamp_and_counter() {
        let mut m = market();
        // created_at = NOW, timestamp used for lookup = NOW + DURATION
        m.offer_duration = DURATION;
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        let counter = id as u16; // id & 0xFFFF
        let lookup_ts = NOW + DURATION;

        let found = m.get_offer_by_counter(lookup_ts, counter);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[test]
    fn get_offer_by_counter_returns_none_for_wrong_timestamp() {
        let mut m = market();
        m.offer_duration = DURATION;
        m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);

        assert!(m.get_offer_by_counter(NOW, 1).is_none());
    }

    #[test]
    fn get_offer_by_counter_returns_none_for_wrong_counter() {
        let mut m = market();
        m.offer_duration = DURATION;
        m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);

        assert!(m.get_offer_by_counter(NOW + DURATION, 999).is_none());
    }

    #[test]
    fn get_offer_by_counter_returns_none_for_inactive_offer() {
        let mut m = market();
        m.offer_duration = DURATION;
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.cancel_offer(id);

        assert!(m.get_offer_by_counter(NOW + DURATION, id as u16).is_none());
    }

    // ── accept_offer ──────────────────────────────────────────────────────────

    #[test]
    fn accept_offer_decrements_amount() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        assert!(m.accept_offer(id, 3));

        let active = m.get_active_offers();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].amount, 7);
        assert!(active[0].active);
    }

    #[test]
    fn accept_offer_deactivates_when_amount_reaches_zero() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        assert!(m.accept_offer(id, 10));
        assert!(m.get_active_offers().is_empty());
    }

    #[test]
    fn accept_offer_deactivates_when_amount_exceeds_offer_amount() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 5, 500, NOW);
        assert!(m.accept_offer(id, 99));
        assert!(m.get_active_offers().is_empty());
    }

    #[test]
    fn accept_offer_returns_false_for_unknown_id() {
        let mut m = market();
        assert!(!m.accept_offer(999, 1));
    }

    #[test]
    fn accept_offer_partial_multiple_times() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.accept_offer(id, 4);
        m.accept_offer(id, 3);
        let active = m.get_active_offers();
        assert_eq!(active[0].amount, 3);
    }

    // ── delete_offer ──────────────────────────────────────────────────────────

    #[test]
    fn delete_offer_removes_offer_entirely() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        assert!(m.delete_offer(id));
        assert!(m.get_active_offers().is_empty());
        // Verify it is truly gone, not just marked inactive
        assert!(m.get_offers_for_item(100).is_empty());
    }

    #[test]
    fn delete_offer_returns_false_for_unknown_id() {
        let mut m = market();
        assert!(!m.delete_offer(999));
    }

    #[test]
    fn delete_offer_only_removes_targeted_offer() {
        let mut m = market();
        let id1 = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        let _id2 = m.create_offer(2, OfferType::Sell, 100, 5, 400, NOW);
        m.delete_offer(id1);
        assert_eq!(m.get_active_offers().len(), 1);
    }

    // ── cancel_offer ──────────────────────────────────────────────────────────

    #[test]
    fn cancel_offer_marks_offer_as_inactive() {
        let mut m = market();
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        assert!(m.cancel_offer(id));
        assert!(m.get_active_offers().is_empty());
    }

    #[test]
    fn cancel_offer_returns_false_for_unknown_id() {
        let mut m = market();
        assert!(!m.cancel_offer(999));
    }

    // ── get_active_offers ──────────────────────────────────────────────────────

    #[test]
    fn get_active_offers_excludes_inactive_offers() {
        let mut m = market();
        let id1 = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        let _id2 = m.create_offer(2, OfferType::Buy, 200, 5, 300, NOW);
        m.cancel_offer(id1);

        let active = m.get_active_offers();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].player_guid, 2);
    }

    // ── append_history ────────────────────────────────────────────────────────

    #[test]
    fn append_history_adds_record() {
        let mut m = market();
        m.append_history(
            1,
            OfferType::Sell,
            100,
            5,
            500,
            NOW + DURATION,
            OfferState::Accepted,
        );
        let history = m.get_own_history(&OfferType::Sell, 1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].item_type_id, 100);
        assert_eq!(history[0].amount, 5);
        assert_eq!(history[0].price, 500);
        assert_eq!(history[0].expires_at, NOW + DURATION);
    }

    // ── get_own_history ────────────────────────────────────────────────────────

    #[test]
    fn get_own_history_filters_by_player_and_type() {
        let mut m = market();
        m.append_history(
            1,
            OfferType::Sell,
            100,
            5,
            500,
            NOW + DURATION,
            OfferState::Accepted,
        );
        m.append_history(
            1,
            OfferType::Buy,
            200,
            3,
            300,
            NOW + DURATION,
            OfferState::Expired,
        );
        m.append_history(
            2,
            OfferType::Sell,
            100,
            1,
            600,
            NOW + DURATION,
            OfferState::Accepted,
        );

        let sell_1 = m.get_own_history(&OfferType::Sell, 1);
        assert_eq!(sell_1.len(), 1);
        assert_eq!(sell_1[0].item_type_id, 100);

        let buy_1 = m.get_own_history(&OfferType::Buy, 1);
        assert_eq!(buy_1.len(), 1);
        assert_eq!(buy_1[0].state, OfferState::Expired);
    }

    #[test]
    fn get_own_history_normalises_accepted_ex_to_accepted() {
        let mut m = market();
        m.append_history(
            1,
            OfferType::Sell,
            100,
            5,
            500,
            NOW + DURATION,
            OfferState::AcceptedEx,
        );
        let history = m.get_own_history(&OfferType::Sell, 1);
        assert_eq!(history[0].state, OfferState::Accepted);
    }

    #[test]
    fn get_own_history_returns_empty_for_unknown_player() {
        let m = market();
        assert!(m.get_own_history(&OfferType::Sell, 999).is_empty());
    }

    // ── move_offer_to_history ─────────────────────────────────────────────────

    #[test]
    fn move_offer_to_history_removes_offer_and_adds_history() {
        let mut m = market();
        m.offer_duration = DURATION;
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        assert!(m.move_offer_to_history(id, OfferState::Expired));

        // Offer is gone
        assert!(m.get_active_offers().is_empty());
        assert!(m.get_offers_for_item(100).is_empty());

        // History entry exists
        let history = m.get_own_history(&OfferType::Sell, 1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].expires_at, NOW + DURATION);
        assert_eq!(history[0].state, OfferState::Expired);
    }

    #[test]
    fn move_offer_to_history_returns_false_for_unknown_id() {
        let mut m = market();
        assert!(!m.move_offer_to_history(999, OfferState::Expired));
    }

    #[test]
    fn move_offer_to_history_sets_correct_expires_at() {
        let mut m = market();
        m.offer_duration = DURATION;
        let id = m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW);
        m.move_offer_to_history(id, OfferState::Accepted);

        let history = m.get_own_history(&OfferType::Sell, 1);
        // expires_at = created_at + offer_duration = NOW + DURATION
        assert_eq!(history[0].expires_at, NOW + DURATION);
    }

    // ── cleanup_expired_offers ────────────────────────────────────────────────

    #[test]
    fn cleanup_expired_offers_removes_old_offers_to_history() {
        let mut m = market();
        m.offer_duration = DURATION;
        m.create_offer(1, OfferType::Sell, 100, 10, 500, NOW - 1000);
        m.create_offer(2, OfferType::Sell, 100, 5, 400, NOW);
        m.create_offer(3, OfferType::Buy, 200, 1, 100, NOW + 1000);

        m.cleanup_expired_offers(NOW);

        // Only offers at NOW and NOW+1000 remain active
        assert_eq!(m.get_active_offers().len(), 2);

        // The expired offer is in history
        let history = m.get_own_history(&OfferType::Sell, 1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].state, OfferState::Expired);
    }

    #[test]
    fn cleanup_expired_offers_strict_less_than_cutoff() {
        let mut m = market();
        // created_at == cutoff should survive
        m.create_offer(1, OfferType::Sell, 100, 1, 100, NOW);
        m.cleanup_expired_offers(NOW);
        assert_eq!(m.get_active_offers().len(), 1);
    }

    #[test]
    fn cleanup_expired_offers_on_empty_market_is_noop() {
        let mut m = market();
        m.cleanup_expired_offers(NOW);
        assert!(m.get_active_offers().is_empty());
    }

    // ── compute_statistics ────────────────────────────────────────────────────

    #[test]
    fn compute_statistics_aggregates_accepted_history() {
        let mut m = market();
        m.append_history(1, OfferType::Buy, 100, 5, 100, NOW, OfferState::Accepted);
        m.append_history(2, OfferType::Buy, 100, 3, 200, NOW, OfferState::Accepted);
        m.append_history(3, OfferType::Sell, 100, 1, 150, NOW, OfferState::Accepted);

        let (purchase, sale) = m.compute_statistics();

        let buy_stats = &purchase[&100];
        assert_eq!(buy_stats.num_transactions, 2);
        assert_eq!(buy_stats.total_price, 300);
        assert_eq!(buy_stats.lowest_price, 100);
        assert_eq!(buy_stats.highest_price, 200);

        let sell_stats = &sale[&100];
        assert_eq!(sell_stats.num_transactions, 1);
        assert_eq!(sell_stats.total_price, 150);
        assert_eq!(sell_stats.lowest_price, 150);
        assert_eq!(sell_stats.highest_price, 150);
    }

    #[test]
    fn compute_statistics_excludes_expired_and_cancelled_entries() {
        let mut m = market();
        m.append_history(1, OfferType::Buy, 100, 5, 100, NOW, OfferState::Expired);
        m.append_history(2, OfferType::Buy, 100, 3, 200, NOW, OfferState::Cancelled);

        let (purchase, _sale) = m.compute_statistics();
        assert!(purchase.is_empty());
    }

    #[test]
    fn compute_statistics_includes_accepted_ex_entries() {
        let mut m = market();
        m.append_history(1, OfferType::Sell, 100, 5, 300, NOW, OfferState::AcceptedEx);

        let (_purchase, sale) = m.compute_statistics();
        assert_eq!(sale[&100].num_transactions, 1);
        assert_eq!(sale[&100].total_price, 300);
    }

    #[test]
    fn compute_statistics_returns_empty_maps_when_no_history() {
        let m = market();
        let (purchase, sale) = m.compute_statistics();
        assert!(purchase.is_empty());
        assert!(sale.is_empty());
    }

    #[test]
    fn compute_statistics_tracks_multiple_items_independently() {
        let mut m = market();
        m.append_history(1, OfferType::Buy, 100, 1, 50, NOW, OfferState::Accepted);
        m.append_history(1, OfferType::Buy, 200, 1, 80, NOW, OfferState::Accepted);

        let (purchase, _) = m.compute_statistics();
        assert_eq!(purchase[&100].total_price, 50);
        assert_eq!(purchase[&200].total_price, 80);
    }

    // ── Expired-offer settlement decision helpers (Session 31) ──────────

    /// Sale + itemtype.id == 0 → SkipInvalidType.
    #[test]
    fn settlement_skip_when_sale_and_itemtype_zero() {
        assert_eq!(
            expired_offer_settlement(true, true, true, 0, 10, 0),
            ExpiredOfferSettlement::SkipInvalidType
        );
        // Stackable / charges / non-zero amount don't override the skip.
        assert_eq!(
            expired_offer_settlement(true, true, false, 7, 100, 0),
            ExpiredOfferSettlement::SkipInvalidType
        );
    }

    /// Sale + stackable → DeliverStackable(total).
    #[test]
    fn settlement_deliver_stackable_carries_amount() {
        assert_eq!(
            expired_offer_settlement(true, false, true, 0, 250, 0),
            ExpiredOfferSettlement::DeliverStackable {
                item_type_id: 0,
                total_amount: 250,
            }
        );
    }

    /// Sale + non-stackable + charges>0 → DeliverNonStackable(sub_type=charges).
    #[test]
    fn settlement_deliver_non_stackable_with_charges_uses_charge_count() {
        assert_eq!(
            expired_offer_settlement(true, false, false, 250, 3, 0),
            ExpiredOfferSettlement::DeliverNonStackable {
                item_type_id: 0,
                sub_type: 250,
                count: 3,
            }
        );
    }

    /// Sale + non-stackable + charges==0 → DeliverNonStackable(sub_type=-1).
    #[test]
    fn settlement_deliver_non_stackable_without_charges_uses_minus_one() {
        assert_eq!(
            expired_offer_settlement(true, false, false, 0, 5, 0),
            ExpiredOfferSettlement::DeliverNonStackable {
                item_type_id: 0,
                sub_type: -1,
                count: 5,
            }
        );
    }

    /// Buy → RefundBank(price * amount).
    #[test]
    fn settlement_buy_refunds_price_times_amount() {
        assert_eq!(
            expired_offer_settlement(false, false, false, 0, 10, 50_000),
            ExpiredOfferSettlement::RefundBank {
                total_price: 500_000,
            }
        );
    }

    /// Buy refund saturates instead of overflowing.
    #[test]
    fn settlement_buy_refund_saturates_on_overflow() {
        // price * amount would overflow u64 at the extreme; saturating
        // multiply caps at u64::MAX.
        let outcome = expired_offer_settlement(false, false, false, 0, u16::MAX, u64::MAX);
        assert_eq!(
            outcome,
            ExpiredOfferSettlement::RefundBank {
                total_price: u64::MAX,
            }
        );
    }

    // ── Stackable-chunk planner ─────────────────────────────────────────

    /// Zero amount yields an empty chunk list.
    #[test]
    fn chunks_zero_amount_returns_empty() {
        assert_eq!(plan_stackable_delivery_chunks(0, 100), Vec::<u16>::new());
    }

    /// Zero max_per_stack returns empty (safety against divide-by-zero
    /// in alternative impls; we don't panic).
    #[test]
    fn chunks_zero_max_returns_empty() {
        assert_eq!(plan_stackable_delivery_chunks(10, 0), Vec::<u16>::new());
    }

    /// Amount ≤ max → single chunk equal to amount.
    #[test]
    fn chunks_amount_below_max_returns_single_full() {
        assert_eq!(plan_stackable_delivery_chunks(7, 100), vec![7]);
    }

    /// Amount exact multiple of max → equal full chunks.
    #[test]
    fn chunks_exact_multiple_returns_equal_full_chunks() {
        assert_eq!(
            plan_stackable_delivery_chunks(300, 100),
            vec![100, 100, 100]
        );
    }

    /// Amount not a multiple → last chunk is the remainder.
    #[test]
    fn chunks_partial_last_carries_remainder() {
        assert_eq!(plan_stackable_delivery_chunks(250, 100), vec![100, 100, 50]);
    }

    // ── Offline-save decision ──────────────────────────────────────────

    #[test]
    fn should_save_only_when_loaded_offline() {
        assert!(should_save_offline_loaded_player(true));
        assert!(!should_save_offline_loaded_player(false));
    }

    // ── Scheduler reschedule decision ──────────────────────────────────

    /// 0 minutes → None (disabled).
    #[test]
    fn reschedule_disabled_when_zero_minutes() {
        assert_eq!(expired_offer_check_reschedule_ms(0), None);
    }

    /// Negative → None (defensive; matches C++ `<= 0` guard).
    #[test]
    fn reschedule_disabled_when_negative_minutes() {
        assert_eq!(expired_offer_check_reschedule_ms(-5), None);
    }

    /// Positive → minutes × 60_000.
    #[test]
    fn reschedule_returns_minutes_in_ms() {
        assert_eq!(expired_offer_check_reschedule_ms(5), Some(300_000));
        assert_eq!(expired_offer_check_reschedule_ms(60), Some(3_600_000));
    }
}
