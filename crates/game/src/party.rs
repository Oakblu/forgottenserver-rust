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
}
