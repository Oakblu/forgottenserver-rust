//! In-memory ban management.
//!
//! Migrated from forgottenserver ban.h / ban.cpp.
//!
//! Uses a `HashMap`-backed in-memory store — no database dependency.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

// ---------------------------------------------------------------------------
// BanInfo
// ---------------------------------------------------------------------------

/// Full information about an active ban — mirrors the C++ `IOBan::BanInfo`.
#[derive(Debug, Clone)]
pub struct BanInfo {
    /// Human-readable ban reason.  Defaults to `"(none)"` when empty.
    pub reason: String,
    /// Name of the administrator who issued the ban.
    pub banned_by: String,
    /// `None` = permanent.  `Some(t)` = Unix timestamp when the ban expires.
    pub expires_at: Option<SystemTime>,
}

// ---------------------------------------------------------------------------
// BanEntry (internal)
// ---------------------------------------------------------------------------

/// A single ban record (internal storage).
#[derive(Debug, Clone)]
pub struct BanEntry {
    pub reason: String,
    pub banned_by: String,
    /// `None` = permanent ban.  `Some(t)` = ban expires at system time `t`.
    pub expires_at: Option<SystemTime>,
}

impl BanEntry {
    /// Returns `true` if this ban is still active (not yet expired).
    fn is_active(&self) -> bool {
        match self.expires_at {
            None => true, // permanent
            Some(expiry) => SystemTime::now() < expiry,
        }
    }

    /// Converts to a `BanInfo` if the ban is still active.
    fn to_ban_info(&self) -> Option<BanInfo> {
        if self.is_active() {
            Some(BanInfo {
                reason: if self.reason.is_empty() {
                    "(none)".to_string()
                } else {
                    self.reason.clone()
                },
                banned_by: self.banned_by.clone(),
                expires_at: self.expires_at,
            })
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// BanManager
// ---------------------------------------------------------------------------

/// Manages IP bans, account bans, and locked player names.
pub struct BanManager {
    ip_bans: HashMap<u32, BanEntry>,
    account_bans: HashMap<u32, BanEntry>,
    locked_names: HashSet<String>,
}

impl Default for BanManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BanManager {
    /// Creates a new empty `BanManager`.
    pub fn new() -> Self {
        Self {
            ip_bans: HashMap::new(),
            account_bans: HashMap::new(),
            locked_names: HashSet::new(),
        }
    }

    // -----------------------------------------------------------------------
    // IP bans
    // -----------------------------------------------------------------------

    /// Adds an IP ban.
    ///
    /// If `duration_secs` is `None`, the ban is permanent.
    /// If `duration_secs` is `Some(n)`, the ban expires in `n` seconds.
    pub fn add_ip_ban(
        &mut self,
        ip: u32,
        reason: String,
        banned_by: String,
        duration_secs: Option<u64>,
    ) {
        let expires_at = duration_secs.map(|secs| SystemTime::now() + Duration::from_secs(secs));
        self.ip_bans.insert(
            ip,
            BanEntry {
                reason,
                banned_by,
                expires_at,
            },
        );
    }

    /// Removes an IP ban.
    pub fn remove_ip_ban(&mut self, ip: u32) {
        self.ip_bans.remove(&ip);
    }

    /// Returns `true` if the IP is currently banned (and the ban has not expired).
    pub fn is_ip_banned(&self, ip: u32) -> bool {
        self.ip_bans
            .get(&ip)
            .map(|e| e.is_active())
            .unwrap_or(false)
    }

    /// Returns full `BanInfo` for the given IP if it is actively banned.
    ///
    /// Returns `None` if the IP is not banned or the ban has expired.
    pub fn get_ip_ban_info(&self, ip: u32) -> Option<BanInfo> {
        self.ip_bans.get(&ip).and_then(|e| e.to_ban_info())
    }

    // -----------------------------------------------------------------------
    // Account bans
    // -----------------------------------------------------------------------

    /// Adds an account ban.
    ///
    /// If `duration_secs` is `None`, the ban is permanent.
    pub fn add_account_ban(
        &mut self,
        account_id: u32,
        reason: String,
        banned_by: String,
        duration_secs: Option<u64>,
    ) {
        let expires_at = duration_secs.map(|secs| SystemTime::now() + Duration::from_secs(secs));
        self.account_bans.insert(
            account_id,
            BanEntry {
                reason,
                banned_by,
                expires_at,
            },
        );
    }

    /// Removes an account ban.
    pub fn remove_account_ban(&mut self, account_id: u32) {
        self.account_bans.remove(&account_id);
    }

    /// Returns `true` if the account is currently banned and the ban has not expired.
    pub fn is_account_banned(&self, account_id: u32) -> bool {
        self.account_bans
            .get(&account_id)
            .map(|e| e.is_active())
            .unwrap_or(false)
    }

    /// Returns full `BanInfo` for the given account if it is actively banned.
    ///
    /// Returns `None` if the account is not banned or the ban has expired.
    pub fn get_account_ban_info(&self, account_id: u32) -> Option<BanInfo> {
        self.account_bans
            .get(&account_id)
            .and_then(|e| e.to_ban_info())
    }

    // -----------------------------------------------------------------------
    // Player name locks
    // -----------------------------------------------------------------------

    /// Locks a player name (prevents it from being used).
    pub fn lock_player_name(&mut self, name: String) {
        self.locked_names.insert(name);
    }

    /// Unlocks a player name.
    pub fn unlock_player_name(&mut self, name: &str) {
        self.locked_names.remove(name);
    }

    /// Returns `true` if the player name is locked.
    pub fn is_player_name_locked(&self, name: &str) -> bool {
        self.locked_names.contains(name)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // IP bans
    // -----------------------------------------------------------------------

    #[test]
    fn test_ip_ban_add_then_is_banned() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x01010101, "spam".into(), "admin".into(), None);
        assert!(mgr.is_ip_banned(0x01010101));
    }

    #[test]
    fn test_ip_ban_remove_clears_ban() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x01010101, "spam".into(), "admin".into(), None);
        mgr.remove_ip_ban(0x01010101);
        assert!(!mgr.is_ip_banned(0x01010101));
    }

    #[test]
    fn test_ip_ban_unknown_ip_not_banned() {
        let mgr = BanManager::new();
        assert!(!mgr.is_ip_banned(0xDEAD_BEEF));
    }

    #[test]
    fn test_ip_ban_permanent_is_always_active() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x7F000001, "permanent".into(), "gm".into(), None);
        assert!(mgr.is_ip_banned(0x7F000001));
    }

    #[test]
    fn test_ip_ban_timed_zero_secs_is_immediately_expired() {
        let mut mgr = BanManager::new();
        // A 0-second ban should have already expired (or expire immediately)
        mgr.add_ip_ban(0x02020202, "expired".into(), "gm".into(), Some(0));
        // We cannot guarantee the exact timing, but a 0-second ban is
        // effectively already expired.  The ban was set to now + 0, so
        // `now < expiry` is false.
        assert!(!mgr.is_ip_banned(0x02020202));
    }

    #[test]
    fn test_ip_ban_future_expiry_is_active() {
        let mut mgr = BanManager::new();
        // 1 hour in the future — definitely still active
        mgr.add_ip_ban(0x03030303, "temp".into(), "gm".into(), Some(3600));
        assert!(mgr.is_ip_banned(0x03030303));
    }

    #[test]
    fn test_get_ip_ban_info_returns_reason_and_banned_by() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x0A000001, "hacking".into(), "GameMaster".into(), None);
        let info = mgr
            .get_ip_ban_info(0x0A000001)
            .expect("should have ban info");
        assert_eq!(info.reason, "hacking");
        assert_eq!(info.banned_by, "GameMaster");
        assert!(info.expires_at.is_none());
    }

    #[test]
    fn test_get_ip_ban_info_none_for_unbanned_ip() {
        let mgr = BanManager::new();
        assert!(mgr.get_ip_ban_info(0x0A000002).is_none());
    }

    #[test]
    fn test_get_ip_ban_info_none_after_expiry() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x0A000003, "temp".into(), "gm".into(), Some(0));
        assert!(mgr.get_ip_ban_info(0x0A000003).is_none());
    }

    #[test]
    fn test_get_ip_ban_info_empty_reason_becomes_none_label() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x0A000004, "".into(), "gm".into(), None);
        let info = mgr
            .get_ip_ban_info(0x0A000004)
            .expect("should have ban info");
        assert_eq!(info.reason, "(none)");
    }

    #[test]
    fn test_ip_ban_overwrite_replaces_existing() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x0B000001, "spam".into(), "gm1".into(), None);
        mgr.add_ip_ban(0x0B000001, "hacking".into(), "gm2".into(), None);
        let info = mgr
            .get_ip_ban_info(0x0B000001)
            .expect("should have ban info");
        assert_eq!(info.reason, "hacking");
        assert_eq!(info.banned_by, "gm2");
    }

    #[test]
    fn test_multiple_ips_banned_independently() {
        let mut mgr = BanManager::new();
        mgr.add_ip_ban(0x01000001, "reason1".into(), "gm".into(), None);
        mgr.add_ip_ban(0x01000002, "reason2".into(), "gm".into(), None);
        assert!(mgr.is_ip_banned(0x01000001));
        assert!(mgr.is_ip_banned(0x01000002));
        mgr.remove_ip_ban(0x01000001);
        assert!(!mgr.is_ip_banned(0x01000001));
        assert!(mgr.is_ip_banned(0x01000002));
    }

    // -----------------------------------------------------------------------
    // Account bans
    // -----------------------------------------------------------------------

    #[test]
    fn test_account_ban_add_then_is_banned() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(42, "cheating".into(), "admin".into(), None);
        assert!(mgr.is_account_banned(42));
    }

    #[test]
    fn test_account_ban_remove_clears_ban() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(42, "cheating".into(), "admin".into(), None);
        mgr.remove_account_ban(42);
        assert!(!mgr.is_account_banned(42));
    }

    #[test]
    fn test_account_ban_unknown_account_not_banned() {
        let mgr = BanManager::new();
        assert!(!mgr.is_account_banned(9999));
    }

    #[test]
    fn test_account_ban_permanent_is_always_active() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(1, "perm".into(), "admin".into(), None);
        assert!(mgr.is_account_banned(1));
    }

    #[test]
    fn test_get_account_ban_info_returns_reason_and_banned_by() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(100, "botting".into(), "SeniorGM".into(), None);
        let info = mgr.get_account_ban_info(100).expect("should have ban info");
        assert_eq!(info.reason, "botting");
        assert_eq!(info.banned_by, "SeniorGM");
        assert!(info.expires_at.is_none());
    }

    #[test]
    fn test_get_account_ban_info_none_for_unbanned_account() {
        let mgr = BanManager::new();
        assert!(mgr.get_account_ban_info(999).is_none());
    }

    #[test]
    fn test_get_account_ban_info_none_after_expiry() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(200, "temp".into(), "gm".into(), Some(0));
        assert!(mgr.get_account_ban_info(200).is_none());
    }

    #[test]
    fn test_get_account_ban_info_empty_reason_becomes_none_label() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(300, "".into(), "gm".into(), None);
        let info = mgr.get_account_ban_info(300).expect("should have ban info");
        assert_eq!(info.reason, "(none)");
    }

    #[test]
    fn test_account_ban_with_future_expiry_has_expires_at() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(400, "temp ban".into(), "gm".into(), Some(3600));
        let info = mgr.get_account_ban_info(400).expect("should have ban info");
        assert!(info.expires_at.is_some());
        // The expiry should be in the future
        assert!(info.expires_at.unwrap() > SystemTime::now());
    }

    #[test]
    fn test_account_ban_overwrite_replaces_existing() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(500, "cheating".into(), "gm1".into(), None);
        mgr.add_account_ban(500, "botting".into(), "gm2".into(), None);
        let info = mgr.get_account_ban_info(500).expect("should have ban info");
        assert_eq!(info.reason, "botting");
        assert_eq!(info.banned_by, "gm2");
    }

    #[test]
    fn test_multiple_accounts_banned_independently() {
        let mut mgr = BanManager::new();
        mgr.add_account_ban(601, "reason1".into(), "gm".into(), None);
        mgr.add_account_ban(602, "reason2".into(), "gm".into(), None);
        assert!(mgr.is_account_banned(601));
        assert!(mgr.is_account_banned(602));
        mgr.remove_account_ban(601);
        assert!(!mgr.is_account_banned(601));
        assert!(mgr.is_account_banned(602));
    }

    // -----------------------------------------------------------------------
    // Player name locks
    // -----------------------------------------------------------------------

    #[test]
    fn test_name_lock_then_is_locked() {
        let mut mgr = BanManager::new();
        mgr.lock_player_name("BadName".into());
        assert!(mgr.is_player_name_locked("BadName"));
    }

    #[test]
    fn test_name_unlock_clears_lock() {
        let mut mgr = BanManager::new();
        mgr.lock_player_name("BadName".into());
        mgr.unlock_player_name("BadName");
        assert!(!mgr.is_player_name_locked("BadName"));
    }

    #[test]
    fn test_name_not_locked_by_default() {
        let mgr = BanManager::new();
        assert!(!mgr.is_player_name_locked("Hero"));
    }

    #[test]
    fn test_name_lock_is_case_sensitive() {
        let mut mgr = BanManager::new();
        mgr.lock_player_name("Hero".into());
        assert!(!mgr.is_player_name_locked("hero"));
    }

    #[test]
    fn test_name_unlock_nonexistent_does_not_panic() {
        let mut mgr = BanManager::new();
        // Should not panic
        mgr.unlock_player_name("NonExistent");
        assert!(!mgr.is_player_name_locked("NonExistent"));
    }

    #[test]
    fn test_multiple_names_locked_independently() {
        let mut mgr = BanManager::new();
        mgr.lock_player_name("BadName1".into());
        mgr.lock_player_name("BadName2".into());
        assert!(mgr.is_player_name_locked("BadName1"));
        assert!(mgr.is_player_name_locked("BadName2"));
        mgr.unlock_player_name("BadName1");
        assert!(!mgr.is_player_name_locked("BadName1"));
        assert!(mgr.is_player_name_locked("BadName2"));
    }

    #[test]
    fn test_lock_same_name_twice_idempotent() {
        let mut mgr = BanManager::new();
        mgr.lock_player_name("DupName".into());
        mgr.lock_player_name("DupName".into());
        assert!(mgr.is_player_name_locked("DupName"));
        // Unlocking once should clear it
        mgr.unlock_player_name("DupName");
        assert!(!mgr.is_player_name_locked("DupName"));
    }

    // -----------------------------------------------------------------------
    // BanManager default
    // -----------------------------------------------------------------------

    #[test]
    fn test_ban_manager_default_is_empty() {
        let mgr = BanManager::default();
        assert!(!mgr.is_ip_banned(0x01010101));
        assert!(!mgr.is_account_banned(1));
        assert!(!mgr.is_player_name_locked("Any"));
    }
}
