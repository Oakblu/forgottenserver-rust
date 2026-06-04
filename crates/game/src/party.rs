use std::collections::HashMap;

use forgottenserver_entity::party::Party;

pub type EntityId = u32;

pub struct PartyManager {
    parties: HashMap<EntityId, Party>,
    player_party: HashMap<EntityId, EntityId>,
}

impl Default for PartyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PartyManager {
    pub fn new() -> Self {
        PartyManager {
            parties: HashMap::new(),
            player_party: HashMap::new(),
        }
    }

    fn find_leader(&self, player_id: EntityId) -> Option<EntityId> {
        if self.parties.contains_key(&player_id) {
            Some(player_id)
        } else {
            self.player_party.get(&player_id).copied()
        }
    }

    fn party_members(&self, leader_id: EntityId) -> Vec<EntityId> {
        self.parties
            .get(&leader_id)
            .map(|p| p.get_members().to_vec())
            .unwrap_or_default()
    }

    /// Invitee accepts invite from inviter. Returns `(recipient_id, leader_id)` for broadcast.
    pub fn accept_invite(
        &mut self,
        invitee_id: EntityId,
        inviter_id: EntityId,
    ) -> Vec<(EntityId, EntityId)> {
        let party = self
            .parties
            .entry(inviter_id)
            .or_insert_with(|| Party::new(inviter_id));
        party.join_party(invitee_id);
        self.player_party.insert(invitee_id, inviter_id);

        let members = party.get_members().to_vec();
        let mut recipients: Vec<EntityId> = std::iter::once(inviter_id).chain(members).collect();
        recipients.sort();
        recipients.dedup();
        recipients
            .into_iter()
            .map(|pid| (pid, inviter_id))
            .collect()
    }

    /// Leader revokes invite for `player_id`.
    pub fn revoke_invite(&mut self, leader_id: EntityId, player_id: EntityId) {
        if let Some(party) = self.parties.get_mut(&leader_id) {
            party.revoke_invitation(player_id);
        }
    }

    /// Transfer leadership. Returns `(recipient_id, new_leader_id)` for broadcast.
    pub fn pass_leadership(
        &mut self,
        old_leader_id: EntityId,
        new_leader_id: EntityId,
    ) -> Vec<(EntityId, EntityId)> {
        let Some(old_party) = self.parties.remove(&old_leader_id) else {
            return vec![];
        };

        let old_members = old_party.get_members().to_vec();
        let mut new_party = Party::new(new_leader_id);

        new_party.join_party(old_leader_id);
        self.player_party.insert(old_leader_id, new_leader_id);

        for &m in &old_members {
            if m != new_leader_id {
                new_party.join_party(m);
                self.player_party.insert(m, new_leader_id);
            } else {
                self.player_party.remove(&new_leader_id);
            }
        }

        self.parties.insert(new_leader_id, new_party);

        let mut recipients: Vec<EntityId> = std::iter::once(new_leader_id)
            .chain(self.party_members(new_leader_id))
            .collect();
        recipients.sort();
        recipients.dedup();
        recipients
            .into_iter()
            .map(|pid| (pid, new_leader_id))
            .collect()
    }

    /// Player leaves party. Returns `(recipient_id, remaining_members)`. Empty members = disbanded.
    pub fn leave(&mut self, player_id: EntityId) -> Vec<(EntityId, Vec<EntityId>)> {
        let leader_id = match self.find_leader(player_id) {
            Some(id) => id,
            None => return vec![],
        };

        if player_id == leader_id {
            if let Some(party) = self.parties.remove(&leader_id) {
                let members = party.get_members().to_vec();
                for &m in &members {
                    self.player_party.remove(&m);
                }
                let mut all: Vec<EntityId> = std::iter::once(leader_id).chain(members).collect();
                all.sort();
                all.dedup();
                return all.into_iter().map(|pid| (pid, vec![])).collect();
            }
            return vec![];
        }

        self.player_party.remove(&player_id);
        if let Some(party) = self.parties.get_mut(&leader_id) {
            party.leave_party(player_id);
            let remaining = party.get_members().to_vec();

            if remaining.is_empty() {
                self.parties.remove(&leader_id);
                return vec![(leader_id, vec![]), (player_id, vec![])];
            }

            let mut recipients: Vec<EntityId> = std::iter::once(leader_id)
                .chain(remaining.iter().copied())
                .collect();
            recipients.sort();
            recipients.dedup();
            recipients
                .into_iter()
                .map(|pid| (pid, remaining.clone()))
                .collect()
        } else {
            vec![]
        }
    }

    /// Toggle shared XP for the party the player belongs to.
    pub fn set_shared_xp(&mut self, player_id: EntityId, active: bool) {
        if let Some(leader_id) = self.find_leader(player_id) {
            if let Some(party) = self.parties.get_mut(&leader_id) {
                party.set_shared_experience(active);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_invite_adds_member() {
        let mut mgr = PartyManager::new();
        let broadcasts = mgr.accept_invite(2, 1);
        assert!(
            !broadcasts.is_empty(),
            "Must broadcast to at least one recipient"
        );
        assert!(
            broadcasts.iter().any(|(pid, _)| *pid == 2),
            "Invitee must receive update"
        );
        assert!(
            broadcasts.iter().any(|(pid, _)| *pid == 1),
            "Inviter must receive update"
        );
    }

    #[test]
    fn leave_disbands_when_leader_alone() {
        let mut mgr = PartyManager::new();
        mgr.accept_invite(2, 1);
        let broadcasts = mgr.leave(2);
        // After member leaves, leader (1) is alone → disband
        let has_leader_notified = broadcasts.iter().any(|(pid, _)| *pid == 1);
        assert!(has_leader_notified, "Leader must be notified on disband");
        let leader_remaining: Vec<EntityId> = broadcasts
            .iter()
            .filter(|(pid, _)| *pid == 1)
            .flat_map(|(_, m)| m.clone())
            .collect();
        assert!(
            leader_remaining.is_empty(),
            "Remaining members must be empty on disband"
        );
    }

    #[test]
    fn pass_leadership_updates_leader() {
        let mut mgr = PartyManager::new();
        mgr.accept_invite(2, 1);
        let broadcasts = mgr.pass_leadership(1, 2);
        assert!(!broadcasts.is_empty(), "Must broadcast leadership change");
        for (_, new_leader_id) in &broadcasts {
            assert_eq!(*new_leader_id, 2, "New leader must be player 2");
        }
    }

    // ── Task 21.9 — playerLeaveParty: leader leaving disbands the party ──────
    // Mirrors C++ `Party::leaveParty(player)` when the player is the leader:
    // the party is disbanded and all members are notified with empty remaining
    // member lists.

    #[test]
    fn leader_leaving_disbands_party_with_members() {
        let mut mgr = PartyManager::new();
        // Leader=1, members=2,3
        mgr.accept_invite(2, 1);
        mgr.accept_invite(3, 1);

        // Leader (1) leaves.
        let broadcasts = mgr.leave(1);

        // Must notify all three participants.
        let notified_ids: Vec<EntityId> = broadcasts.iter().map(|(pid, _)| *pid).collect();
        assert!(notified_ids.contains(&1), "leader must be notified of own leave");
        assert!(notified_ids.contains(&2), "member 2 must be notified");
        assert!(notified_ids.contains(&3), "member 3 must be notified");

        // All notifications must carry an empty remaining-members list (disbanded).
        for (_, remaining) in &broadcasts {
            assert!(
                remaining.is_empty(),
                "disbanded party must have empty remaining members in all notifications"
            );
        }
    }

    #[test]
    fn leader_leaving_solo_party_is_clean() {
        let mut mgr = PartyManager::new();
        // Party with only the leader — accept invite never called.
        // A solo leader is not tracked in `parties`, so leave returns empty.
        let broadcasts = mgr.leave(1);
        assert!(broadcasts.is_empty(), "solo player has no party to leave");
    }

    // ── Task 21.13 — playerAcceptTrade: items exchanged when both accept ──────
    // Mirrors C++ `Game::playerAcceptTrade`: both players must accept before
    // the trade completes.

    #[test]
    fn trade_both_accept_completes_and_closes() {
        use crate::trade::{TradeItem, TradeManager, TradeResult};
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, TradeItem { type_id: 100 }).unwrap();
        assert_eq!(mgr.accept(trade_id, 1), TradeResult::Pending);
        assert_eq!(
            mgr.accept(trade_id, 2),
            TradeResult::Completed,
            "trade must complete when both sides accept"
        );
        // After completion the trade is gone.
        assert!(mgr.get_trade_for_player(1).is_none());
        assert!(mgr.get_trade_for_player(2).is_none());
    }

    // ── Task 21.14 — playerCloseTrade: CANCELLED outcome (trade closed) ───────
    // Mirrors C++ `Game::playerCloseTrade` → `internalCloseTrade`:
    // calling close must remove the trade from both players.

    #[test]
    fn trade_close_removes_trade_for_both_players() {
        use crate::trade::{TradeItem, TradeManager};
        let mut mgr = TradeManager::new();
        let trade_id = mgr.open(1, 2, TradeItem { type_id: 200 }).unwrap();
        mgr.close(trade_id);
        assert!(
            mgr.get_trade_for_player(1).is_none(),
            "trade must be closed for player 1 after close"
        );
        assert!(
            mgr.get_trade_for_player(2).is_none(),
            "trade must be closed for player 2 after close"
        );
    }

    // ── Default impl (lines 13-14) ─────────────────────────────────────────────

    #[test]
    fn party_manager_default_is_same_as_new() {
        let mut mgr = PartyManager::default();
        // A fresh manager has no parties — leaving any player returns empty.
        assert!(mgr.leave(1).is_empty());
    }

    // ── revoke_invite (lines 65-67) ────────────────────────────────────────────

    #[test]
    fn revoke_invite_does_nothing_without_a_party() {
        // No party exists yet — should not panic.
        let mut mgr = PartyManager::new();
        mgr.revoke_invite(1, 2); // leader 1, player 2
    }

    #[test]
    fn revoke_invite_removes_pending_invitation() {
        let mut mgr = PartyManager::new();
        // Create a party by having player 2 accept player 1's invite.
        mgr.accept_invite(2, 1);
        // Now revoke an (additional) invitation for player 3.
        // Party exists for leader 1, so the inner `party.revoke_invitation` is called.
        mgr.revoke_invite(1, 3);
        // Party must still be intact (leader 1 still leads player 2).
        let broadcasts = mgr.leave(1);
        let notified: Vec<EntityId> = broadcasts.iter().map(|(id, _)| *id).collect();
        assert!(notified.contains(&1), "leader must be notified of own leave");
    }

    // ── pass_leadership with no existing party (line 78) ──────────────────────

    #[test]
    fn pass_leadership_returns_empty_when_leader_has_no_party() {
        let mut mgr = PartyManager::new();
        let broadcasts = mgr.pass_leadership(1, 2);
        assert!(
            broadcasts.is_empty(),
            "pass_leadership must return empty when the old leader has no party"
        );
    }

    // ── pass_leadership else branch (lines 89-90): new leader is in member list

    #[test]
    fn pass_leadership_when_new_leader_was_already_a_member() {
        let mut mgr = PartyManager::new();
        // Leader=1, members=2,3
        mgr.accept_invite(2, 1);
        mgr.accept_invite(3, 1);
        // Pass leadership to member 2 (who is in old_members, triggers the else branch).
        let broadcasts = mgr.pass_leadership(1, 2);
        assert!(!broadcasts.is_empty(), "Must broadcast leadership change");
        for (_, new_leader) in &broadcasts {
            assert_eq!(*new_leader, 2, "New leader must be player 2");
        }
    }

    // ── leave: member leaves with remaining members (lines 140-148) ───────────

    #[test]
    fn member_leaving_when_others_remain_broadcasts_to_all_remaining() {
        let mut mgr = PartyManager::new();
        // Leader=1, members=2,3
        mgr.accept_invite(2, 1);
        mgr.accept_invite(3, 1);

        // Member 3 leaves; leader 1 and member 2 remain.
        let broadcasts = mgr.leave(3);

        let notified_ids: Vec<EntityId> = broadcasts.iter().map(|(pid, _)| *pid).collect();
        // Leader and remaining member must be notified.
        assert!(notified_ids.contains(&1), "leader must be notified");
        assert!(notified_ids.contains(&2), "remaining member must be notified");
        // Remaining member list in each notification must be non-empty.
        for (_, remaining) in &broadcasts {
            assert!(
                !remaining.is_empty(),
                "remaining members must not be empty while party has members"
            );
        }
    }

    // ── leave: player who has no party (line 113 None branch) ─────────────────

    #[test]
    fn leave_for_player_with_no_party_returns_empty() {
        let mut mgr = PartyManager::new();
        assert!(mgr.leave(42).is_empty(), "unknown player has no party");
    }

    // ── set_shared_xp (lines 155-158) ─────────────────────────────────────────

    #[test]
    fn set_shared_xp_does_nothing_for_untracked_player() {
        // Player 1 has no party — must not panic.
        let mut mgr = PartyManager::new();
        mgr.set_shared_xp(1, true);
    }

    #[test]
    fn set_shared_xp_toggles_for_leader() {
        let mut mgr = PartyManager::new();
        mgr.accept_invite(2, 1); // leader=1
        // Enabling shared XP for the leader must not panic.
        mgr.set_shared_xp(1, true);
        mgr.set_shared_xp(1, false);
    }

    #[test]
    fn set_shared_xp_toggles_for_member() {
        let mut mgr = PartyManager::new();
        mgr.accept_invite(2, 1); // leader=1, member=2
        // Enabling shared XP via a member's id must not panic.
        mgr.set_shared_xp(2, true);
        mgr.set_shared_xp(2, false);
    }

    // ── leave: member leaving disbands when last member goes (line 137) ────────

    #[test]
    fn last_member_leaving_disbands_party_and_notifies_both() {
        let mut mgr = PartyManager::new();
        mgr.accept_invite(2, 1); // leader=1, member=2

        let broadcasts = mgr.leave(2);

        let notified_ids: Vec<EntityId> = broadcasts.iter().map(|(pid, _)| *pid).collect();
        assert!(notified_ids.contains(&1), "leader must be notified on disband");
        assert!(notified_ids.contains(&2), "leaving member must be notified");
        for (_, remaining) in &broadcasts {
            assert!(
                remaining.is_empty(),
                "disbanded party must send empty remaining list"
            );
        }
    }
}
