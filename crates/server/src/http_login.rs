// Mirrors `src/http/login.cpp`:`tfs::http::handle_login`.
//
// Two upgrades over the previous plaintext skeleton:
//   * Stored passwords are SHA-1 hashed (matches the C++
//     `transformToSHA1(password)` compare against `UNHEX(password)`).
//   * Accounts may carry a TOTP `secret`; when set, the request must include
//     a `token` matching the C++ ±1 30-second window check.

use forgottenserver_common::tools::{generate_token, transform_to_sha1};
use std::collections::HashMap;

/// 30-second TOTP window — mirror of C++ `AUTHENTICATOR_PERIOD`.
pub const AUTHENTICATOR_PERIOD: u64 = 30;

/// 6-digit TOTP tokens — mirror of C++ `generateToken(..., 6)`.
const TOTP_LENGTH: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub account_name: String,
    /// Plaintext password as sent by the client. The handler hashes
    /// this with SHA-1 before comparing to the stored hash.
    pub password: String,
    /// Optional TOTP token. Required when the account has a non-empty
    /// secret configured.
    pub token: Option<String>,
}

#[derive(Debug, Clone)]
struct AccountRecord {
    /// Raw SHA-1 digest of the password (20 bytes).
    password_sha1: [u8; 20],
    /// TOTP shared secret (raw bytes). Empty = no 2FA.
    secret: Vec<u8>,
}

pub struct LoginHandler {
    accounts: HashMap<String, AccountRecord>,
    request_counts: HashMap<u32, u32>,
    pub rate_limit: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoginResult {
    Success {
        character_list: Vec<String>,
    },
    BadCredentials,
    /// Account has 2FA enabled and the supplied token (if any) did not match
    /// any of the current / prev / next 30-second windows. Mirrors C++ error
    /// code 6 "Two-factor token required for authentication."
    TwoFactorRequired,
    RateLimited,
}

impl LoginHandler {
    pub fn new(rate_limit: u32) -> Self {
        Self {
            accounts: HashMap::new(),
            request_counts: HashMap::new(),
            rate_limit,
        }
    }

    /// Register an account. The plaintext password is hashed with SHA-1
    /// before storage, matching the C++ `UNHEX(password)` schema.
    pub fn add_account(&mut self, name: &str, password: &str) {
        self.accounts.insert(
            name.to_string(),
            AccountRecord {
                password_sha1: transform_to_sha1(password.as_bytes()),
                secret: Vec::new(),
            },
        );
    }

    /// Register an account that requires a TOTP token at login.
    pub fn add_account_with_2fa(&mut self, name: &str, password: &str, secret: &[u8]) {
        self.accounts.insert(
            name.to_string(),
            AccountRecord {
                password_sha1: transform_to_sha1(password.as_bytes()),
                secret: secret.to_vec(),
            },
        );
    }

    pub fn handle_login(&mut self, ip: u32, creds: &Credentials) -> LoginResult {
        self.handle_login_at(ip, creds, current_unix_seconds())
    }

    /// Test-friendly entry point that takes the "current time" explicitly,
    /// so the ±1 window check can be exercised without mocking the system
    /// clock.
    pub fn handle_login_at(&mut self, ip: u32, creds: &Credentials, now_secs: u64) -> LoginResult {
        if self.is_rate_limited(ip) {
            return LoginResult::RateLimited;
        }
        self.increment_ip_count(ip);

        let record = match self.accounts.get(&creds.account_name) {
            Some(r) => r.clone(),
            None => return LoginResult::BadCredentials,
        };

        let supplied_hash = transform_to_sha1(creds.password.as_bytes());
        if supplied_hash != record.password_sha1 {
            return LoginResult::BadCredentials;
        }

        if !record.secret.is_empty() {
            let token = match creds.token.as_deref() {
                Some(t) if !t.is_empty() => t,
                _ => return LoginResult::TwoFactorRequired,
            };
            let ticks = now_secs / AUTHENTICATOR_PERIOD;
            let candidates = [
                generate_token(&record.secret, ticks, TOTP_LENGTH),
                generate_token(&record.secret, ticks.saturating_sub(1), TOTP_LENGTH),
                generate_token(&record.secret, ticks + 1, TOTP_LENGTH),
            ];
            if !candidates.iter().any(|c| c == token) {
                return LoginResult::TwoFactorRequired;
            }
        }

        LoginResult::Success {
            character_list: vec![creds.account_name.clone()],
        }
    }

    pub fn is_rate_limited(&self, ip: u32) -> bool {
        self.request_counts.get(&ip).copied().unwrap_or(0) >= self.rate_limit
    }

    pub fn increment_ip_count(&mut self, ip: u32) {
        let count = self.request_counts.entry(ip).or_insert(0);
        *count += 1;
    }
}

fn current_unix_seconds() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_creds(account_name: &str, password: &str) -> Credentials {
        Credentials {
            account_name: account_name.to_string(),
            password: password.to_string(),
            token: None,
        }
    }

    fn make_creds_with_token(account_name: &str, password: &str, token: &str) -> Credentials {
        Credentials {
            account_name: account_name.to_string(),
            password: password.to_string(),
            token: Some(token.to_string()),
        }
    }

    #[test]
    fn valid_credentials_returns_success_with_character_list() {
        let mut handler = LoginHandler::new(100);
        handler.add_account("alice", "secret");
        let result = handler.handle_login(1, &make_creds("alice", "secret"));
        assert_eq!(
            result,
            LoginResult::Success {
                character_list: vec!["alice".to_string()]
            }
        );
    }

    #[test]
    fn invalid_password_returns_bad_credentials() {
        let mut handler = LoginHandler::new(100);
        handler.add_account("alice", "secret");
        let result = handler.handle_login(1, &make_creds("alice", "wrongpassword"));
        assert_eq!(result, LoginResult::BadCredentials);
    }

    #[test]
    fn unknown_account_returns_bad_credentials() {
        let mut handler = LoginHandler::new(100);
        let result = handler.handle_login(1, &make_creds("nobody", "pass"));
        assert_eq!(result, LoginResult::BadCredentials);
    }

    #[test]
    fn exceeding_rate_limit_returns_rate_limited() {
        let mut handler = LoginHandler::new(2);
        handler.add_account("alice", "secret");
        let _ = handler.handle_login(42, &make_creds("alice", "secret"));
        let _ = handler.handle_login(42, &make_creds("alice", "secret"));
        let result = handler.handle_login(42, &make_creds("alice", "secret"));
        assert_eq!(result, LoginResult::RateLimited);
    }

    #[test]
    fn rate_limit_does_not_affect_other_ips() {
        let mut handler = LoginHandler::new(1);
        handler.add_account("alice", "secret");
        let _ = handler.handle_login(1, &make_creds("alice", "secret"));
        let result = handler.handle_login(2, &make_creds("alice", "secret"));
        assert_eq!(
            result,
            LoginResult::Success {
                character_list: vec!["alice".to_string()]
            }
        );
    }

    #[test]
    fn is_rate_limited_false_before_limit() {
        let handler = LoginHandler::new(5);
        assert!(!handler.is_rate_limited(99));
    }

    #[test]
    fn increment_ip_count_increases_count() {
        let mut handler = LoginHandler::new(3);
        handler.increment_ip_count(7);
        handler.increment_ip_count(7);
        assert_eq!(*handler.request_counts.get(&7).unwrap(), 2);
    }

    // -----------------------------------------------------------------------
    // S45 — SHA-1 + 2FA pipeline
    // -----------------------------------------------------------------------

    #[test]
    fn add_account_stores_sha1_hash_not_plaintext() {
        let mut handler = LoginHandler::new(100);
        handler.add_account("alice", "secret");
        let stored = &handler.accounts["alice"].password_sha1;
        assert_eq!(stored, &transform_to_sha1(b"secret"));
    }

    #[test]
    fn login_fails_when_2fa_required_and_token_missing() {
        let mut handler = LoginHandler::new(100);
        handler.add_account_with_2fa("alice", "secret", b"12345678901234567890");
        let result = handler.handle_login_at(1, &make_creds("alice", "secret"), 30);
        assert_eq!(result, LoginResult::TwoFactorRequired);
    }

    #[test]
    fn login_fails_when_2fa_token_wrong() {
        let mut handler = LoginHandler::new(100);
        handler.add_account_with_2fa("alice", "secret", b"12345678901234567890");
        let result =
            handler.handle_login_at(1, &make_creds_with_token("alice", "secret", "000000"), 30);
        assert_eq!(result, LoginResult::TwoFactorRequired);
    }

    #[test]
    fn login_succeeds_with_2fa_current_window_token() {
        let mut handler = LoginHandler::new(100);
        let secret = b"12345678901234567890";
        handler.add_account_with_2fa("alice", "secret", secret);
        let now = 1_500_000_000u64;
        let ticks = now / AUTHENTICATOR_PERIOD;
        let token = generate_token(secret, ticks, TOTP_LENGTH);
        let result =
            handler.handle_login_at(1, &make_creds_with_token("alice", "secret", &token), now);
        assert!(matches!(result, LoginResult::Success { .. }));
    }

    #[test]
    fn login_succeeds_with_2fa_previous_window_token() {
        let mut handler = LoginHandler::new(100);
        let secret = b"12345678901234567890";
        handler.add_account_with_2fa("alice", "secret", secret);
        let now = 1_500_000_000u64;
        let ticks = now / AUTHENTICATOR_PERIOD;
        let token = generate_token(secret, ticks - 1, TOTP_LENGTH);
        let result =
            handler.handle_login_at(1, &make_creds_with_token("alice", "secret", &token), now);
        assert!(matches!(result, LoginResult::Success { .. }));
    }

    #[test]
    fn login_succeeds_with_2fa_next_window_token() {
        let mut handler = LoginHandler::new(100);
        let secret = b"12345678901234567890";
        handler.add_account_with_2fa("alice", "secret", secret);
        let now = 1_500_000_000u64;
        let ticks = now / AUTHENTICATOR_PERIOD;
        let token = generate_token(secret, ticks + 1, TOTP_LENGTH);
        let result =
            handler.handle_login_at(1, &make_creds_with_token("alice", "secret", &token), now);
        assert!(matches!(result, LoginResult::Success { .. }));
    }

    #[test]
    fn login_rejects_token_two_windows_old() {
        let mut handler = LoginHandler::new(100);
        let secret = b"12345678901234567890";
        handler.add_account_with_2fa("alice", "secret", secret);
        let now = 1_500_000_000u64;
        let ticks = now / AUTHENTICATOR_PERIOD;
        let token = generate_token(secret, ticks - 2, TOTP_LENGTH);
        let result =
            handler.handle_login_at(1, &make_creds_with_token("alice", "secret", &token), now);
        assert_eq!(result, LoginResult::TwoFactorRequired);
    }

    #[test]
    fn login_succeeds_without_token_when_2fa_disabled() {
        let mut handler = LoginHandler::new(100);
        handler.add_account("alice", "secret");
        // Even with a stray token, no-2FA accounts succeed.
        let result =
            handler.handle_login_at(1, &make_creds_with_token("alice", "secret", "wrong"), 30);
        assert!(matches!(result, LoginResult::Success { .. }));
    }

    #[test]
    fn login_rejects_2fa_empty_token_string() {
        let mut handler = LoginHandler::new(100);
        handler.add_account_with_2fa("alice", "secret", b"12345678901234567890");
        let result = handler.handle_login_at(1, &make_creds_with_token("alice", "secret", ""), 30);
        assert_eq!(result, LoginResult::TwoFactorRequired);
    }
}
