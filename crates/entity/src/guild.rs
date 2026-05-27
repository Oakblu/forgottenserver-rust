//! Migrated from forgottenserver/src/guild.h and guild.cpp
//!
//! Provides the `Guild` struct and associated `GuildRank` type.

// ---------------------------------------------------------------------------
// GuildRank
// ---------------------------------------------------------------------------

/// Mirrors the C++ `GuildRank` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildRank {
    pub id: u32,
    pub name: String,
    pub level: u8,
}

impl GuildRank {
    pub fn new(id: u32, name: impl Into<String>, level: u8) -> Self {
        GuildRank {
            id,
            name: name.into(),
            level,
        }
    }
}

// ---------------------------------------------------------------------------
// Guild
// ---------------------------------------------------------------------------

/// Mirrors the C++ `Guild` class.
#[derive(Debug)]
pub struct Guild {
    id: u32,
    name: String,
    motd: String,
    ranks: Vec<GuildRank>,
    member_count: u32,
    /// Mirrors C++ `std::list<Player*> membersOnline`.
    ///
    /// In Rust we store player guids (`u32`) instead of raw `Player*` so the
    /// entity crate does not need to depend on game-level types. The order in
    /// which players are inserted is preserved (mirrors `push_back`), and
    /// `remove_member_online` returns whether the list is empty afterwards so
    /// callers can decide whether to perform the C++ `g_game.removeGuild(id)`
    /// callback (which itself lives outside the entity crate).
    members_online: Vec<u32>,
}

impl Guild {
    /// Default member rank level — mirrors `Guild::MEMBER_RANK_LEVEL_DEFAULT`.
    pub const MEMBER_RANK_LEVEL_DEFAULT: u8 = 1;

    /// Create a new guild with the given id and name.
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Guild {
            id,
            name: name.into(),
            motd: String::new(),
            ranks: Vec::new(),
            member_count: 0,
            members_online: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Identity
    // -----------------------------------------------------------------------

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    // -----------------------------------------------------------------------
    // MOTD
    // -----------------------------------------------------------------------

    pub fn get_motd(&self) -> &str {
        &self.motd
    }

    pub fn set_motd(&mut self, motd: impl Into<String>) {
        self.motd = motd.into();
    }

    // -----------------------------------------------------------------------
    // Ranks
    // -----------------------------------------------------------------------

    /// Add a new rank. Mirrors `Guild::addRank`.
    pub fn add_rank(&mut self, id: u32, name: impl Into<String>, level: u8) {
        self.ranks.push(GuildRank::new(id, name, level));
    }

    /// Return all ranks. Mirrors `Guild::getRanks`.
    pub fn get_ranks(&self) -> &[GuildRank] {
        &self.ranks
    }

    /// Find a rank by its id. Mirrors `Guild::getRankById`.
    pub fn get_rank_by_id(&self, id: u32) -> Option<&GuildRank> {
        self.ranks.iter().find(|r| r.id == id)
    }

    /// Find a rank by its name (case-insensitive). Mirrors `Guild::getRankByName`
    /// which uses `caseInsensitiveEqual`.
    pub fn get_rank_by_name(&self, name: &str) -> Option<&GuildRank> {
        let lower = name.to_lowercase();
        self.ranks.iter().find(|r| r.name.to_lowercase() == lower)
    }

    /// Find a rank by its level. Mirrors `Guild::getRankByLevel`.
    pub fn get_rank_by_level(&self, level: u8) -> Option<&GuildRank> {
        self.ranks.iter().find(|r| r.level == level)
    }

    // -----------------------------------------------------------------------
    // Member count
    // -----------------------------------------------------------------------

    pub fn get_member_count(&self) -> u32 {
        self.member_count
    }

    pub fn set_member_count(&mut self, count: u32) {
        self.member_count = count;
    }

    // -----------------------------------------------------------------------
    // Online member tracking
    //
    // Mirrors C++ `Guild::addMember`, `Guild::removeMember` and
    // `Guild::getMembersOnline`. The C++ `removeMember` additionally calls
    // `g_game.removeGuild(id)` when the list becomes empty; that callback is
    // a game-layer concern and is intentionally not reproduced here. Instead
    // `remove_member_online` returns `true` when the list is empty after the
    // removal, so the game layer can perform the equivalent cleanup.
    // -----------------------------------------------------------------------

    /// Append a player guid to the online-members list. Mirrors
    /// `Guild::addMember(Player*)` which uses `push_back`.
    pub fn add_member_online(&mut self, player_id: u32) {
        self.members_online.push(player_id);
    }

    /// Remove the first occurrence of `player_id` from the online-members
    /// list (mirrors `std::list::remove`, which removes all equal elements;
    /// the C++ codepath only ever inserts a given pointer once, so we strip
    /// every occurrence for safety). Returns `true` when the list is empty
    /// after the removal so the caller can mirror the C++ `removeGuild`
    /// cleanup hook.
    pub fn remove_member_online(&mut self, player_id: u32) -> bool {
        self.members_online.retain(|&p| p != player_id);
        self.members_online.is_empty()
    }

    /// Return the list of online member guids. Mirrors
    /// `Guild::getMembersOnline()`.
    pub fn get_members_online(&self) -> &[u32] {
        &self.members_online
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guild_rank_struct() {
        let r = GuildRank::new(1, "Leader", 3);
        assert_eq!(r.id, 1);
        assert_eq!(r.name, "Leader");
        assert_eq!(r.level, 3);
    }

    #[test]
    fn test_guild_new_id_name() {
        let g = Guild::new(42, "TestGuild");
        assert_eq!(g.get_id(), 42);
        assert_eq!(g.get_name(), "TestGuild");
    }

    #[test]
    fn test_guild_new_empty_motd() {
        let g = Guild::new(1, "G");
        assert_eq!(g.get_motd(), "");
    }

    #[test]
    fn test_guild_new_empty_ranks() {
        let g = Guild::new(1, "G");
        assert!(g.get_ranks().is_empty());
    }

    #[test]
    fn test_guild_new_zero_member_count() {
        let g = Guild::new(1, "G");
        assert_eq!(g.get_member_count(), 0);
    }

    #[test]
    fn test_guild_set_get_motd() {
        let mut g = Guild::new(1, "G");
        g.set_motd("Hello guild!");
        assert_eq!(g.get_motd(), "Hello guild!");
    }

    #[test]
    fn test_guild_add_rank_and_get_ranks() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "Member", 1);
        g.add_rank(11, "Officer", 2);
        let ranks = g.get_ranks();
        assert_eq!(ranks.len(), 2);
        assert_eq!(ranks[0].name, "Member");
        assert_eq!(ranks[1].name, "Officer");
    }

    #[test]
    fn test_guild_get_rank_by_id_found() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "Member", 1);
        let rank = g.get_rank_by_id(10);
        assert!(rank.is_some());
        assert_eq!(rank.unwrap().name, "Member");
    }

    #[test]
    fn test_guild_get_rank_by_id_not_found() {
        let g = Guild::new(1, "G");
        assert!(g.get_rank_by_id(999).is_none());
    }

    #[test]
    fn test_guild_get_rank_by_name_found() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "Leader", 3);
        let rank = g.get_rank_by_name("Leader");
        assert!(rank.is_some());
        assert_eq!(rank.unwrap().id, 10);
    }

    #[test]
    fn test_guild_get_rank_by_name_not_found() {
        let g = Guild::new(1, "G");
        assert!(g.get_rank_by_name("Unknown").is_none());
    }

    #[test]
    fn test_guild_get_rank_by_level_found() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "Member", 1);
        g.add_rank(11, "Officer", 2);
        let rank = g.get_rank_by_level(2);
        assert!(rank.is_some());
        assert_eq!(rank.unwrap().name, "Officer");
    }

    #[test]
    fn test_guild_get_rank_by_level_not_found() {
        let g = Guild::new(1, "G");
        assert!(g.get_rank_by_level(99).is_none());
    }

    #[test]
    fn test_guild_set_member_count() {
        let mut g = Guild::new(1, "G");
        g.set_member_count(42);
        assert_eq!(g.get_member_count(), 42);
    }

    #[test]
    fn test_guild_member_rank_level_default() {
        assert_eq!(Guild::MEMBER_RANK_LEVEL_DEFAULT, 1);
    }

    // --- case-insensitive rank name lookup (C++ uses caseInsensitiveEqual) ---

    #[test]
    fn test_guild_get_rank_by_name_case_insensitive_lower() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "Leader", 3);
        // C++ getRankByName is case-insensitive via caseInsensitiveEqual
        assert!(g.get_rank_by_name("leader").is_some());
    }

    #[test]
    fn test_guild_get_rank_by_name_case_insensitive_upper() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "Leader", 3);
        assert!(g.get_rank_by_name("LEADER").is_some());
    }

    #[test]
    fn test_guild_get_rank_by_name_case_insensitive_mixed() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "Senior Officer", 5);
        let rank = g.get_rank_by_name("senior officer");
        assert!(rank.is_some());
        assert_eq!(rank.unwrap().id, 10);
    }

    // --- get_rank_by_id returns first match when duplicate ids are added ---

    #[test]
    fn test_guild_get_rank_by_id_returns_first_on_duplicate_ids() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "First", 1);
        g.add_rank(10, "Duplicate", 2);
        let rank = g.get_rank_by_id(10).unwrap();
        assert_eq!(rank.name, "First");
    }

    // --- get_rank_by_level returns first match when duplicate levels exist ---

    #[test]
    fn test_guild_get_rank_by_level_returns_first_on_duplicate_levels() {
        let mut g = Guild::new(1, "G");
        g.add_rank(10, "First", 1);
        g.add_rank(11, "Also Level One", 1);
        let rank = g.get_rank_by_level(1).unwrap();
        assert_eq!(rank.id, 10);
    }

    // --- member count: set_member_count / get_member_count round-trip ---

    #[test]
    fn test_guild_set_member_count_overwrite() {
        let mut g = Guild::new(1, "G");
        g.set_member_count(10);
        g.set_member_count(20);
        assert_eq!(g.get_member_count(), 20);
    }

    #[test]
    fn test_guild_set_member_count_zero() {
        let mut g = Guild::new(1, "G");
        g.set_member_count(5);
        g.set_member_count(0);
        assert_eq!(g.get_member_count(), 0);
    }

    // --- MOTD round-trip edge cases ---

    #[test]
    fn test_guild_motd_overwrite() {
        let mut g = Guild::new(1, "G");
        g.set_motd("First message");
        g.set_motd("Updated message");
        assert_eq!(g.get_motd(), "Updated message");
    }

    #[test]
    fn test_guild_motd_empty_string() {
        let mut g = Guild::new(1, "G");
        g.set_motd("Some motd");
        g.set_motd("");
        assert_eq!(g.get_motd(), "");
    }

    // --- add_rank builds the expected slice ---

    #[test]
    fn test_guild_add_multiple_ranks_ordering() {
        let mut g = Guild::new(1, "G");
        g.add_rank(1, "Member", 1);
        g.add_rank(2, "Officer", 2);
        g.add_rank(3, "Leader", 3);
        let ranks = g.get_ranks();
        assert_eq!(ranks.len(), 3);
        assert_eq!(ranks[0].level, 1);
        assert_eq!(ranks[1].level, 2);
        assert_eq!(ranks[2].level, 3);
    }

    // --- identity getters ---

    #[test]
    fn test_guild_get_id() {
        let g = Guild::new(99, "MyGuild");
        assert_eq!(g.get_id(), 99);
    }

    #[test]
    fn test_guild_get_name() {
        let g = Guild::new(1, "Dragons");
        assert_eq!(g.get_name(), "Dragons");
    }

    // -----------------------------------------------------------------------
    // Online member tracking (parity with C++ addMember / removeMember /
    // getMembersOnline)
    // -----------------------------------------------------------------------

    #[test]
    fn test_guild_new_members_online_is_empty() {
        let g = Guild::new(1, "G");
        assert!(g.get_members_online().is_empty());
    }

    #[test]
    fn test_guild_add_member_online_appends_in_order() {
        let mut g = Guild::new(1, "G");
        g.add_member_online(10);
        g.add_member_online(20);
        g.add_member_online(30);
        assert_eq!(g.get_members_online(), &[10, 20, 30]);
    }

    #[test]
    fn test_guild_add_member_online_allows_duplicates() {
        // C++ addMember uses push_back unconditionally — no de-dup.
        let mut g = Guild::new(1, "G");
        g.add_member_online(7);
        g.add_member_online(7);
        assert_eq!(g.get_members_online(), &[7, 7]);
    }

    #[test]
    fn test_guild_remove_member_online_removes_matching() {
        let mut g = Guild::new(1, "G");
        g.add_member_online(1);
        g.add_member_online(2);
        g.add_member_online(3);
        let empty = g.remove_member_online(2);
        assert!(!empty);
        assert_eq!(g.get_members_online(), &[1, 3]);
    }

    #[test]
    fn test_guild_remove_member_online_returns_true_when_list_empties() {
        // Mirrors the C++ branch where `membersOnline.empty()` is true and
        // `g_game.removeGuild(id)` would fire.
        let mut g = Guild::new(1, "G");
        g.add_member_online(42);
        let empty = g.remove_member_online(42);
        assert!(empty);
        assert!(g.get_members_online().is_empty());
    }

    #[test]
    fn test_guild_remove_member_online_returns_false_when_list_non_empty() {
        let mut g = Guild::new(1, "G");
        g.add_member_online(1);
        g.add_member_online(2);
        let empty = g.remove_member_online(1);
        assert!(!empty);
        assert_eq!(g.get_members_online(), &[2]);
    }

    #[test]
    fn test_guild_remove_member_online_absent_id_is_noop_but_signals_empty_when_empty() {
        // Removing an absent id from an empty list still leaves the list
        // empty, so the return value follows the same `is_empty()` rule as
        // the C++ `if (membersOnline.empty())` check.
        let mut g = Guild::new(1, "G");
        let empty = g.remove_member_online(999);
        assert!(empty);
        assert!(g.get_members_online().is_empty());
    }

    #[test]
    fn test_guild_remove_member_online_absent_id_in_populated_list_is_noop() {
        let mut g = Guild::new(1, "G");
        g.add_member_online(1);
        g.add_member_online(2);
        let empty = g.remove_member_online(999);
        assert!(!empty);
        assert_eq!(g.get_members_online(), &[1, 2]);
    }

    #[test]
    fn test_guild_remove_member_online_strips_all_duplicates() {
        // Mirrors `std::list::remove` semantics (removes all equal elements).
        let mut g = Guild::new(1, "G");
        g.add_member_online(5);
        g.add_member_online(5);
        g.add_member_online(6);
        let empty = g.remove_member_online(5);
        assert!(!empty);
        assert_eq!(g.get_members_online(), &[6]);
    }

    #[test]
    fn test_guild_remove_member_online_strips_duplicates_to_empty() {
        let mut g = Guild::new(1, "G");
        g.add_member_online(5);
        g.add_member_online(5);
        let empty = g.remove_member_online(5);
        assert!(empty);
        assert!(g.get_members_online().is_empty());
    }
}
