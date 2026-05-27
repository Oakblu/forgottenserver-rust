//! Migrated from forgottenserver/src/party.h and party.cpp
//!
//! Provides the `Party` struct for player group management.
//! Player references are replaced with player GUIDs (`u32`).
//! Level information is passed in as a `&[(u32, u32)]` slice (guid, level)
//! wherever the C++ would dereference a `Player*` to call `getLevel()`.

// ---------------------------------------------------------------------------
// Constants (mirrors party.h static constexpr)
// ---------------------------------------------------------------------------

pub const EXPERIENCE_SHARE_RANGE: i32 = 30;
pub const EXPERIENCE_SHARE_FLOORS: i32 = 1;

// ---------------------------------------------------------------------------
// SharedExpStatus
// ---------------------------------------------------------------------------

/// Mirrors the C++ `SharedExpStatus_t` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SharedExpStatus {
    Ok = 0,
    TooFarAway = 1,
    LevelDiffTooLarge = 2,
    MemberInactive = 3,
    EmptyParty = 4,
}

// ---------------------------------------------------------------------------
// Party
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Party` class, using player GUIDs instead of pointers.
///
/// # Leadership passover
/// When the leader leaves a party that still has members (or members + invitees),
/// leadership is automatically transferred to the first member in `member_list`
/// (`pass_party_leadership`) before the leader is removed, matching the C++
/// `leaveParty` / `passPartyLeadership` chain.
///
/// # Disband
/// A party disbands automatically when:
/// - the leader leaves and there are no members and no invitees, OR
/// - the last member leaves and no invitees remain.
///
/// `disband()` clears all lists and marks the party as disbanded; after that
/// `is_disbanded()` returns `true` and most mutations are no-ops.
#[derive(Debug)]
pub struct Party {
    /// `None` only after `disband()` has been called.
    leader_guid: Option<u32>,
    member_list: Vec<u32>,
    invite_list: Vec<u32>,
    shared_exp_active: bool,
    shared_exp_enabled: bool,
    disbanded: bool,
}

impl Party {
    /// Create a new party with the given leader.
    pub fn new(leader_guid: u32) -> Self {
        Party {
            leader_guid: Some(leader_guid),
            member_list: Vec::new(),
            invite_list: Vec::new(),
            shared_exp_active: false,
            shared_exp_enabled: false,
            disbanded: false,
        }
    }

    // -----------------------------------------------------------------------
    // Leader
    // -----------------------------------------------------------------------

    /// Returns the leader's GUID, or `None` if the party has been disbanded.
    pub fn get_leader_guid(&self) -> Option<u32> {
        self.leader_guid
    }

    /// Pass leadership to `new_leader_guid`.
    ///
    /// Mirrors `Party::passPartyLeadership`. Returns `false` when:
    /// - the party is disbanded,
    /// - `new_leader_guid` is already the leader, or
    /// - `new_leader_guid` is not in the member list.
    ///
    /// On success:
    /// - removes `new_leader_guid` from `member_list`,
    /// - inserts the old leader at the **front** of `member_list`, and
    /// - updates `leader_guid`.
    pub fn pass_party_leadership(&mut self, new_leader_guid: u32) -> bool {
        // A non-disbanded party always has `Some(leader_guid)`; the
        // single guard below covers both reachable error paths.
        let current_leader = match self.leader_guid {
            Some(g) if !self.disbanded => g,
            _ => return false,
        };
        if current_leader == new_leader_guid {
            return false;
        }
        // new leader must be an existing member
        let pos = match self.member_list.iter().position(|&g| g == new_leader_guid) {
            Some(p) => p,
            None => return false,
        };
        self.member_list.remove(pos);
        // old leader moves to front of member list
        self.member_list.insert(0, current_leader);
        self.leader_guid = Some(new_leader_guid);
        true
    }

    // -----------------------------------------------------------------------
    // Members
    // -----------------------------------------------------------------------

    pub fn get_member_count(&self) -> usize {
        self.member_list.len()
    }

    pub fn get_members(&self) -> &[u32] {
        &self.member_list
    }

    /// Add `guid` to the member list.
    ///
    /// Mirrors `joinParty`:
    /// - removes `guid` from the invite list first,
    /// - does nothing if `guid` is already a member.
    pub fn join_party(&mut self, guid: u32) {
        if self.disbanded {
            return;
        }
        self.invite_list.retain(|&g| g != guid);
        if !self.member_list.contains(&guid) {
            self.member_list.push(guid);
        }
    }

    /// Remove `guid` from the party.
    ///
    /// Mirrors `leaveParty`. Handles three cases:
    ///
    /// 1. **Regular member leaves** — removed from `member_list`; if the list
    ///    becomes empty and there are no invitees the party disbands.
    /// 2. **Leader leaves, members remain** — `pass_party_leadership` is called
    ///    with `member_list.front()` before the leader is removed.  If there is
    ///    only one member *and* the invite list is empty the party disbands
    ///    instead (matches C++ `missingLeader` path).
    /// 3. **Leader leaves, no members** — party disbands immediately.
    ///
    /// Returns `false` when `guid` is neither the leader nor a member.
    pub fn leave_party(&mut self, guid: u32) -> bool {
        if self.disbanded {
            return false;
        }
        let is_leader = self.leader_guid == Some(guid);
        let is_member = self.member_list.contains(&guid);

        if !is_leader && !is_member {
            return false;
        }

        if is_leader {
            if !self.member_list.is_empty() {
                // C++ missingLeader: only 1 member and no invites → disband
                if self.member_list.len() == 1 && self.invite_list.is_empty() {
                    self.disband();
                    return true;
                }
                // Pass leadership to the first member
                let new_leader = self.member_list[0];
                self.pass_party_leadership(new_leader);
                // After passover, old leader is now member_list[0]; remove them
                self.member_list.retain(|&g| g != guid);
            } else {
                // No members at all → disband
                self.disband();
                return true;
            }
        } else {
            self.member_list.retain(|&g| g != guid);
        }

        // If the party is now empty disband it
        if self.is_empty() {
            self.disband();
        }

        true
    }

    // -----------------------------------------------------------------------
    // Invitations
    // -----------------------------------------------------------------------

    pub fn get_invitation_count(&self) -> usize {
        self.invite_list.len()
    }

    /// Returns the read-only invite list (player GUIDs that have been
    /// invited but not yet joined). Mirrors C++ `Party::getInvitees`.
    pub fn get_invitees(&self) -> &[u32] {
        &self.invite_list
    }

    /// Recipient GUIDs for a party-wide broadcast. Mirrors the C++
    /// `Party::broadcastPartyMessage` iteration:
    ///   1. Every member receives.
    ///   2. The leader receives (deduplicated when the leader is also
    ///      in the member list — some C++ paths keep the leader
    ///      separately, the Rust port stores it via `get_leader_guid()`).
    ///   3. When `send_to_invitations` is true, pending invitees also
    ///      receive.
    ///
    /// Returned list is deduplicated so a caller iterating and calling
    /// `protocolgame::send_text_message(guid, msg)` doesn't double-send.
    /// Cross-crate caller does the actual protocol dispatch.
    pub fn broadcast_recipient_guids(&self, send_to_invitations: bool) -> Vec<u32> {
        let mut seen = std::collections::HashSet::new();
        let mut out: Vec<u32> = Vec::new();
        for guid in &self.member_list {
            if seen.insert(*guid) {
                out.push(*guid);
            }
        }
        if let Some(leader) = self.get_leader_guid() {
            if seen.insert(leader) {
                out.push(leader);
            }
        }
        if send_to_invitations {
            for guid in &self.invite_list {
                if seen.insert(*guid) {
                    out.push(*guid);
                }
            }
        }
        out
    }

    /// Add `guid` to the invite list.
    ///
    /// Mirrors `invitePlayer`. Returns `false` when `guid` is already invited.
    pub fn invite_player(&mut self, guid: u32) -> bool {
        if self.disbanded {
            return false;
        }
        if self.invite_list.contains(&guid) {
            return false;
        }
        self.invite_list.push(guid);
        true
    }

    /// Returns `true` when `guid` has a pending invitation.
    pub fn is_player_invited(&self, guid: u32) -> bool {
        self.invite_list.contains(&guid)
    }

    /// Remove `guid` from the invite list.
    ///
    /// Mirrors `revokeInvitation` / `removeInvite`. Returns `false` when `guid`
    /// is not in the invite list.
    pub fn revoke_invitation(&mut self, guid: u32) -> bool {
        if !self.invite_list.contains(&guid) {
            return false;
        }
        self.invite_list.retain(|&g| g != guid);
        // mirror removeInvite: disband if now empty
        if self.is_empty() {
            self.disband();
        }
        true
    }

    // -----------------------------------------------------------------------
    // Corpse access
    // -----------------------------------------------------------------------

    /// Returns `true` if `owner_guid` is the leader or a member.
    ///
    /// Mirrors `canOpenCorpse`.
    pub fn can_open_corpse(&self, owner_guid: u32) -> bool {
        if self.disbanded {
            return false;
        }
        self.leader_guid == Some(owner_guid) || self.member_list.contains(&owner_guid)
    }

    // -----------------------------------------------------------------------
    // Level-spread / shared-experience eligibility
    // -----------------------------------------------------------------------

    /// Compute the minimum level a player must have to qualify for shared XP.
    ///
    /// Mirrors the C++ formula `ceil((highestLevel * 2) / 3)`.
    /// `levels` is a slice of `(guid, level)` pairs for all party members
    /// (including the leader).  Returns `1` when the slice is empty.
    pub fn min_shared_exp_level(levels: &[(u32, u32)]) -> u32 {
        let highest = levels.iter().map(|&(_, lvl)| lvl).max().unwrap_or(0);
        if highest == 0 {
            return 1;
        }
        // ceil((highest * 2) / 3)
        (highest * 2).div_ceil(3)
    }

    /// Check whether a single player qualifies for shared experience based
    /// solely on the level-spread rule.
    ///
    /// `player_level` — the level of the player to test.
    /// `levels` — levels of all party members (including leader).
    ///
    /// Returns `SharedExpStatus::LevelDiffTooLarge` or `SharedExpStatus::Ok`.
    pub fn member_level_status(player_level: u32, levels: &[(u32, u32)]) -> SharedExpStatus {
        if levels.iter().all(|&(_, l)| l == 0) {
            return SharedExpStatus::EmptyParty;
        }
        let min_level = Self::min_shared_exp_level(levels);
        if player_level < min_level {
            SharedExpStatus::LevelDiffTooLarge
        } else {
            SharedExpStatus::Ok
        }
    }

    // -----------------------------------------------------------------------
    // Shared experience
    // -----------------------------------------------------------------------

    /// Returns `true` when both member and invite lists are empty.
    pub fn is_empty(&self) -> bool {
        self.member_list.is_empty() && self.invite_list.is_empty()
    }

    pub fn is_disbanded(&self) -> bool {
        self.disbanded
    }

    pub fn is_shared_experience_active(&self) -> bool {
        self.shared_exp_active
    }

    pub fn is_shared_experience_enabled(&self) -> bool {
        self.shared_exp_enabled
    }

    /// Enable or disable shared experience (leader-only in C++; enforcement is
    /// the caller's responsibility here).
    ///
    /// Mirrors `setSharedExperience`. Returns `false` when the value is
    /// unchanged, `true` when a change was made.
    pub fn set_shared_experience(&mut self, active: bool) -> bool {
        if self.shared_exp_active == active {
            return false;
        }
        self.shared_exp_active = active;
        if !active {
            self.shared_exp_enabled = false;
        }
        true
    }

    /// Synchronise `shared_exp_enabled` from an external status check.
    ///
    /// Mirrors `updateSharedExperience`: only acts when `shared_exp_active` is
    /// `true`; flips `shared_exp_enabled` to `result` if it differs.
    pub fn update_shared_experience(&mut self, result: bool) {
        if self.shared_exp_active && self.shared_exp_enabled != result {
            self.shared_exp_enabled = result;
        }
    }

    // -----------------------------------------------------------------------
    // Party size limit
    // -----------------------------------------------------------------------

    /// Maximum number of members (excluding the leader) allowed in a party.
    /// Matches the TFS hard limit used in several server scripts (20 total = 1
    /// leader + 19 members).  Not enforced by a C++ constant in the migrated
    /// sources, but referenced in server-side Lua as `MAX_PARTY_MEMBERS`.
    pub const MAX_MEMBERS: usize = 19;

    /// Returns `true` when no more members can join (`member_list` is full).
    pub fn is_full(&self) -> bool {
        self.member_list.len() >= Self::MAX_MEMBERS
    }

    // -----------------------------------------------------------------------
    // Disband
    // -----------------------------------------------------------------------

    /// Disband: clear all members and invitees, mark as disbanded.
    ///
    /// Mirrors `Party::disband`.
    pub fn disband(&mut self) {
        self.member_list.clear();
        self.invite_list.clear();
        self.leader_guid = None;
        self.disbanded = true;
        self.shared_exp_active = false;
        self.shared_exp_enabled = false;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Enum discriminant tests (existing, kept)
    // -----------------------------------------------------------------------

    #[test]
    fn test_shared_exp_status_variants() {
        assert_eq!(SharedExpStatus::Ok as u8, 0);
        assert_eq!(SharedExpStatus::TooFarAway as u8, 1);
        assert_eq!(SharedExpStatus::LevelDiffTooLarge as u8, 2);
        assert_eq!(SharedExpStatus::MemberInactive as u8, 3);
        assert_eq!(SharedExpStatus::EmptyParty as u8, 4);
    }

    #[test]
    fn test_experience_share_range_constant() {
        assert_eq!(EXPERIENCE_SHARE_RANGE, 30);
    }

    #[test]
    fn test_experience_share_floors_constant() {
        assert_eq!(EXPERIENCE_SHARE_FLOORS, 1);
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_new_leader_guid() {
        let p = Party::new(42);
        assert_eq!(p.get_leader_guid(), Some(42));
    }

    #[test]
    fn test_party_new_empty_member_list() {
        let p = Party::new(1);
        assert_eq!(p.get_member_count(), 0);
    }

    #[test]
    fn test_party_new_empty_invite_list() {
        let p = Party::new(1);
        assert_eq!(p.get_invitation_count(), 0);
    }

    #[test]
    fn test_party_new_not_disbanded() {
        let p = Party::new(1);
        assert!(!p.is_disbanded());
    }

    // -----------------------------------------------------------------------
    // invite_player — bool return value
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_invite_player_returns_true_on_new_invite() {
        let mut p = Party::new(1);
        assert!(p.invite_player(10));
        assert_eq!(p.get_invitation_count(), 1);
    }

    #[test]
    fn test_party_invite_player_returns_false_on_duplicate() {
        let mut p = Party::new(1);
        p.invite_player(10);
        assert!(!p.invite_player(10));
        assert_eq!(p.get_invitation_count(), 1);
    }

    #[test]
    fn test_party_is_player_invited_true() {
        let mut p = Party::new(1);
        p.invite_player(10);
        assert!(p.is_player_invited(10));
    }

    #[test]
    fn test_party_is_player_invited_false() {
        let p = Party::new(1);
        assert!(!p.is_player_invited(99));
    }

    #[test]
    fn test_party_invite_multiple_different_players() {
        let mut p = Party::new(1);
        assert!(p.invite_player(10));
        assert!(p.invite_player(11));
        assert!(p.invite_player(12));
        assert_eq!(p.get_invitation_count(), 3);
    }

    #[test]
    fn test_invite_player_returns_false_when_disbanded() {
        let mut p = Party::new(1);
        p.disband();
        assert!(!p.invite_player(10));
    }

    // -----------------------------------------------------------------------
    // join_party lifecycle
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_join_removes_from_invite() {
        let mut p = Party::new(1);
        p.invite_player(10);
        p.join_party(10);
        assert!(!p.is_player_invited(10));
        assert_eq!(p.get_member_count(), 1);
    }

    #[test]
    fn test_party_join_without_prior_invite() {
        let mut p = Party::new(1);
        p.join_party(99);
        assert_eq!(p.get_member_count(), 1);
    }

    #[test]
    fn test_party_join_idempotent() {
        let mut p = Party::new(1);
        p.invite_player(10);
        p.join_party(10);
        p.join_party(10); // second call must not duplicate
        assert_eq!(p.get_member_count(), 1);
    }

    #[test]
    fn test_party_join_multiple_members() {
        let mut p = Party::new(1);
        p.invite_player(10);
        p.invite_player(11);
        p.join_party(10);
        p.join_party(11);
        assert_eq!(p.get_member_count(), 2);
        assert_eq!(p.get_invitation_count(), 0);
    }

    #[test]
    fn test_join_party_no_op_when_disbanded() {
        let mut p = Party::new(1);
        p.disband();
        p.join_party(10);
        assert_eq!(p.get_member_count(), 0);
    }

    // -----------------------------------------------------------------------
    // leave_party — basic
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_leave_regular_member() {
        let mut p = Party::new(1);
        p.join_party(10);
        assert!(p.leave_party(10));
        assert_eq!(p.get_member_count(), 0);
    }

    #[test]
    fn test_party_leave_nonexistent_returns_false() {
        let mut p = Party::new(1);
        p.join_party(10);
        assert!(!p.leave_party(99));
        assert_eq!(p.get_member_count(), 1);
    }

    #[test]
    fn test_party_leave_returns_false_when_disbanded() {
        let mut p = Party::new(1);
        p.disband();
        assert!(!p.leave_party(1));
    }

    // -----------------------------------------------------------------------
    // leave_party — leader passover
    // -----------------------------------------------------------------------

    #[test]
    fn test_leader_leave_passes_leadership_to_first_member() {
        // Leader = 1, members = [10, 11]
        // When leader 1 leaves, 10 (first member) should become leader.
        let mut p = Party::new(1);
        p.join_party(10);
        p.join_party(11);
        p.leave_party(1);
        // Party should not be disbanded
        assert!(!p.is_disbanded());
        assert_eq!(p.get_leader_guid(), Some(10));
        // Old leader must not appear in member list
        assert!(!p.get_members().contains(&1));
    }

    #[test]
    fn test_leader_leave_new_leader_not_duplicated_in_members() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.join_party(11);
        p.leave_party(1);
        // 10 is now leader; only 11 should remain in member_list
        assert_eq!(p.get_member_count(), 1);
        assert!(p.get_members().contains(&11));
        assert!(!p.get_members().contains(&10));
    }

    #[test]
    fn test_leader_leave_old_leader_inserted_front_then_removed() {
        // After passover, old leader is at front of memberList; leaving removes them.
        let mut p = Party::new(1);
        p.join_party(10);
        p.join_party(11);
        p.join_party(12);
        p.leave_party(1);
        assert!(!p.get_members().contains(&1));
        assert_eq!(p.get_leader_guid(), Some(10));
        assert_eq!(p.get_member_count(), 2); // 11 and 12 remain
    }

    // -----------------------------------------------------------------------
    // leave_party — disband triggers
    // -----------------------------------------------------------------------

    #[test]
    fn test_leader_leave_no_members_disbands() {
        // Leader leaves with no members → disband
        let mut p = Party::new(1);
        p.leave_party(1);
        assert!(p.is_disbanded());
    }

    #[test]
    fn test_leader_leave_one_member_no_invites_disbands() {
        // C++ missingLeader path: 1 member, 0 invites → disband
        let mut p = Party::new(1);
        p.join_party(10);
        p.leave_party(1);
        assert!(p.is_disbanded());
    }

    #[test]
    fn test_leader_leave_one_member_with_invite_does_not_disband() {
        // 1 member + 1 invite → leadership passes, party survives
        let mut p = Party::new(1);
        p.join_party(10);
        p.invite_player(20);
        p.leave_party(1);
        assert!(!p.is_disbanded());
        assert_eq!(p.get_leader_guid(), Some(10));
    }

    #[test]
    fn test_last_member_leaves_disbands_party() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.leave_party(10);
        // now empty (no invites)
        assert!(p.is_disbanded());
    }

    #[test]
    fn test_member_leaves_with_remaining_invites_does_not_disband() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.invite_player(20);
        p.leave_party(10);
        assert!(!p.is_disbanded());
    }

    // -----------------------------------------------------------------------
    // revoke_invitation — bool return + disband side-effect
    // -----------------------------------------------------------------------

    #[test]
    fn test_revoke_invitation_returns_true_on_success() {
        let mut p = Party::new(1);
        p.invite_player(10);
        assert!(p.revoke_invitation(10));
    }

    #[test]
    fn test_revoke_invitation_returns_false_not_invited() {
        let mut p = Party::new(1);
        assert!(!p.revoke_invitation(99));
    }

    #[test]
    fn test_party_revoke_invitation_removes_from_list() {
        let mut p = Party::new(1);
        p.invite_player(10);
        p.revoke_invitation(10);
        assert_eq!(p.get_invitation_count(), 0);
        assert!(!p.is_player_invited(10));
    }

    #[test]
    fn test_revoke_last_invite_no_members_disbands() {
        // No members + revoke last invite → disband (mirrors removeInvite + empty check)
        let mut p = Party::new(1);
        p.invite_player(10);
        p.revoke_invitation(10);
        assert!(p.is_disbanded());
    }

    #[test]
    fn test_revoke_invite_with_remaining_members_does_not_disband() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.invite_player(20);
        p.revoke_invitation(20);
        assert!(!p.is_disbanded());
    }

    // -----------------------------------------------------------------------
    // pass_party_leadership
    // -----------------------------------------------------------------------

    #[test]
    fn test_pass_leadership_success() {
        let mut p = Party::new(1);
        p.join_party(10);
        assert!(p.pass_party_leadership(10));
        assert_eq!(p.get_leader_guid(), Some(10));
    }

    #[test]
    fn test_pass_leadership_old_leader_becomes_member() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.pass_party_leadership(10);
        assert!(p.get_members().contains(&1));
    }

    #[test]
    fn test_pass_leadership_new_leader_removed_from_members() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.pass_party_leadership(10);
        assert!(!p.get_members().contains(&10));
    }

    #[test]
    fn test_pass_leadership_old_leader_at_front_of_member_list() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.join_party(11);
        p.pass_party_leadership(10);
        // old leader (1) should now be at the front
        assert_eq!(p.get_members()[0], 1);
    }

    #[test]
    fn test_pass_leadership_to_self_returns_false() {
        let mut p = Party::new(1);
        p.join_party(10);
        assert!(!p.pass_party_leadership(1));
    }

    #[test]
    fn test_pass_leadership_to_non_member_returns_false() {
        let mut p = Party::new(1);
        p.join_party(10);
        assert!(!p.pass_party_leadership(99));
    }

    #[test]
    fn test_pass_leadership_when_disbanded_returns_false() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.disband();
        assert!(!p.pass_party_leadership(10));
    }

    // -----------------------------------------------------------------------
    // can_open_corpse
    // -----------------------------------------------------------------------

    #[test]
    fn test_can_open_corpse_leader() {
        let p = Party::new(1);
        assert!(p.can_open_corpse(1));
    }

    #[test]
    fn test_can_open_corpse_member() {
        let mut p = Party::new(1);
        p.join_party(10);
        assert!(p.can_open_corpse(10));
    }

    #[test]
    fn test_can_open_corpse_non_member() {
        let p = Party::new(1);
        assert!(!p.can_open_corpse(99));
    }

    #[test]
    fn test_can_open_corpse_invitee_not_allowed() {
        let mut p = Party::new(1);
        p.invite_player(20);
        assert!(!p.can_open_corpse(20));
    }

    #[test]
    fn test_can_open_corpse_disbanded_returns_false() {
        let mut p = Party::new(1);
        p.disband();
        assert!(!p.can_open_corpse(1));
    }

    // -----------------------------------------------------------------------
    // min_shared_exp_level (XP formula)
    // -----------------------------------------------------------------------

    #[test]
    fn test_min_shared_exp_level_empty_returns_one() {
        assert_eq!(Party::min_shared_exp_level(&[]), 1);
    }

    #[test]
    fn test_min_shared_exp_level_single_player() {
        // highest = 100, ceil(200/3) = 67
        let levels = [(1u32, 100u32)];
        assert_eq!(Party::min_shared_exp_level(&levels), 67);
    }

    #[test]
    fn test_min_shared_exp_level_mixed_group() {
        // highest = 90, ceil(180/3) = 60
        let levels = [(1, 50), (2, 90), (3, 70)];
        assert_eq!(Party::min_shared_exp_level(&levels), 60);
    }

    #[test]
    fn test_min_shared_exp_level_exact_divisible() {
        // highest = 60, ceil(120/3) = 40
        let levels = [(1, 60)];
        assert_eq!(Party::min_shared_exp_level(&levels), 40);
    }

    #[test]
    fn test_min_shared_exp_level_rounding_up() {
        // highest = 100: 200/3 = 66.67 → ceil = 67
        let levels = [(1, 100)];
        assert_eq!(Party::min_shared_exp_level(&levels), 67);
    }

    #[test]
    fn test_min_shared_exp_level_low_level() {
        // highest = 3: ceil(6/3) = 2
        let levels = [(1, 3)];
        assert_eq!(Party::min_shared_exp_level(&levels), 2);
    }

    // -----------------------------------------------------------------------
    // member_level_status
    // -----------------------------------------------------------------------

    #[test]
    fn test_member_level_status_ok_at_minimum() {
        let levels = [(1u32, 100u32), (2, 70)];
        // min_level = ceil(200/3) = 67; player at 67 → OK
        assert_eq!(Party::member_level_status(67, &levels), SharedExpStatus::Ok);
    }

    #[test]
    fn test_member_level_status_too_low() {
        let levels = [(1u32, 100u32), (2, 70)];
        // min_level = 67; player at 50 → LevelDiffTooLarge
        assert_eq!(
            Party::member_level_status(50, &levels),
            SharedExpStatus::LevelDiffTooLarge
        );
    }

    #[test]
    fn test_member_level_status_equal_to_highest() {
        let levels = [(1u32, 60u32)];
        assert_eq!(Party::member_level_status(60, &levels), SharedExpStatus::Ok);
    }

    #[test]
    fn test_member_level_status_all_zero_returns_empty_party() {
        // When every supplied level is zero we treat the cohort as empty
        // (mirrors C++ `getMemberSharedExperienceStatus` returning
        // `SHAREDEXP_EMPTYPARTY` when there are no members).
        let levels = [(1u32, 0u32), (2u32, 0u32)];
        assert_eq!(
            Party::member_level_status(0, &levels),
            SharedExpStatus::EmptyParty
        );
    }

    #[test]
    fn test_member_level_status_empty_slice_returns_empty_party() {
        // An empty slice means no party members at all; `all(|l| l == 0)`
        // is vacuously true so we expect `EmptyParty`.
        assert_eq!(
            Party::member_level_status(50, &[]),
            SharedExpStatus::EmptyParty
        );
    }

    // -----------------------------------------------------------------------
    // is_full / MAX_MEMBERS
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_max_members_constant() {
        assert_eq!(Party::MAX_MEMBERS, 19);
    }

    #[test]
    fn test_party_is_not_full_initially() {
        let p = Party::new(1);
        assert!(!p.is_full());
    }

    #[test]
    fn test_party_is_full_when_at_limit() {
        let mut p = Party::new(1);
        for i in 2..=(Party::MAX_MEMBERS as u32 + 1) {
            p.join_party(i);
        }
        assert!(p.is_full());
    }

    #[test]
    fn test_party_not_full_one_below_limit() {
        let mut p = Party::new(1);
        for i in 2..(Party::MAX_MEMBERS as u32 + 1) {
            p.join_party(i);
        }
        assert!(!p.is_full());
    }

    // -----------------------------------------------------------------------
    // set_shared_experience — bool return
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_shared_experience_false_by_default() {
        let p = Party::new(1);
        assert!(!p.is_shared_experience_active());
    }

    #[test]
    fn test_party_set_shared_experience_true_returns_true() {
        let mut p = Party::new(1);
        assert!(p.set_shared_experience(true));
        assert!(p.is_shared_experience_active());
    }

    #[test]
    fn test_party_set_shared_experience_same_value_returns_false() {
        let mut p = Party::new(1);
        p.set_shared_experience(true);
        // setting it to the same value → no change → false
        assert!(!p.set_shared_experience(true));
    }

    #[test]
    fn test_party_set_shared_experience_false_clears_enabled() {
        let mut p = Party::new(1);
        p.set_shared_experience(true);
        p.update_shared_experience(true); // force-enable
        p.set_shared_experience(false);
        assert!(!p.is_shared_experience_enabled());
    }

    // -----------------------------------------------------------------------
    // update_shared_experience
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_shared_experience_sets_enabled_when_active() {
        let mut p = Party::new(1);
        p.set_shared_experience(true);
        p.update_shared_experience(true);
        assert!(p.is_shared_experience_enabled());
    }

    #[test]
    fn test_update_shared_experience_no_op_when_inactive() {
        let mut p = Party::new(1);
        // shared_exp_active is false by default
        p.update_shared_experience(true);
        assert!(!p.is_shared_experience_enabled());
    }

    #[test]
    fn test_update_shared_experience_disables_on_false() {
        let mut p = Party::new(1);
        p.set_shared_experience(true);
        p.update_shared_experience(true);
        p.update_shared_experience(false);
        assert!(!p.is_shared_experience_enabled());
    }

    // -----------------------------------------------------------------------
    // disband
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_disband_clears_all() {
        let mut p = Party::new(1);
        p.join_party(10);
        p.join_party(11);
        p.invite_player(20);
        p.disband();
        assert!(p.is_empty());
        assert_eq!(p.get_member_count(), 0);
        assert_eq!(p.get_invitation_count(), 0);
    }

    #[test]
    fn test_party_disband_sets_disbanded_flag() {
        let mut p = Party::new(1);
        p.disband();
        assert!(p.is_disbanded());
    }

    #[test]
    fn test_party_disband_clears_leader() {
        let mut p = Party::new(1);
        p.disband();
        assert_eq!(p.get_leader_guid(), None);
    }

    #[test]
    fn test_party_disband_clears_shared_exp_flags() {
        let mut p = Party::new(1);
        p.set_shared_experience(true);
        p.update_shared_experience(true);
        p.disband();
        assert!(!p.is_shared_experience_active());
        assert!(!p.is_shared_experience_enabled());
    }

    // -----------------------------------------------------------------------
    // is_empty
    // -----------------------------------------------------------------------

    #[test]
    fn test_party_is_empty_initial() {
        let p = Party::new(1);
        assert!(p.is_empty());
    }

    #[test]
    fn test_party_is_not_empty_after_invite() {
        let mut p = Party::new(1);
        p.invite_player(10);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_party_is_not_empty_after_join() {
        let mut p = Party::new(1);
        p.join_party(10);
        assert!(!p.is_empty());
    }

    // ── broadcast_recipient_guids (Session 38) ──────────────────────────

    /// Members + leader receive; invitees are excluded by default.
    #[test]
    fn broadcast_recipients_default_excludes_invitees() {
        let mut p = Party::new(1); // leader=1
        p.join_party(10);
        p.join_party(20);
        p.invite_player(99);
        let r = p.broadcast_recipient_guids(false);
        assert!(r.contains(&1)); // leader
        assert!(r.contains(&10));
        assert!(r.contains(&20));
        assert!(!r.contains(&99), "invitees excluded when flag=false");
    }

    /// `send_to_invitations=true` includes the invite list.
    #[test]
    fn broadcast_recipients_with_invitations_flag_includes_invitees() {
        let mut p = Party::new(1);
        p.invite_player(99);
        p.invite_player(100);
        let r = p.broadcast_recipient_guids(true);
        assert!(r.contains(&1));
        assert!(r.contains(&99));
        assert!(r.contains(&100));
    }

    /// Deduplication: leader appearing in member_list shouldn't double.
    #[test]
    fn broadcast_recipients_dedup_leader_in_member_list() {
        let mut p = Party::new(1);
        // Force the leader into the member list too (some edge cases in
        // C++ where the leader is also tracked as a member).
        p.join_party(1);
        let r = p.broadcast_recipient_guids(false);
        assert_eq!(r.iter().filter(|&&g| g == 1).count(), 1);
    }

    /// Empty party (no members, no leader after disband) → empty vec.
    #[test]
    fn broadcast_recipients_after_disband_is_empty() {
        let mut p = Party::new(1);
        p.disband();
        let r = p.broadcast_recipient_guids(true);
        assert!(r.is_empty());
    }

    /// `get_invitees` returns the invite list.
    #[test]
    fn get_invitees_returns_pending_invites() {
        let mut p = Party::new(1);
        p.invite_player(99);
        p.invite_player(100);
        assert_eq!(p.get_invitees(), &[99, 100]);
    }
}
