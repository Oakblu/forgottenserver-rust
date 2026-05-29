// Mirrors `src/http/login.cpp`:`tfs::http::handle_login`.
//
// Two upgrades over the previous plaintext skeleton:
//   * Stored passwords are SHA-1 hashed (matches the C++
//     `transformToSHA1(password)` compare against `UNHEX(password)`).
//   * Accounts may carry a TOTP `secret`; when set, the request must include
//     a `token` matching the C++ ±1 30-second window check.

use forgottenserver_common::tools::{generate_token, transform_to_sha1, transform_to_sha1_hex};
use forgottenserver_common::base64;
use forgottenserver_database::database::Database;
use forgottenserver_items::vocation::Vocations;
use rand::random;
use serde::Serialize;
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

// ── DB-backed login handler (tasks 3.1–3.9) ──────────────────────────────────

/// Deserialized fields from the HTTP POST body for the login endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub token: Option<String>,
}

/// Server-level configuration forwarded into the login response.
pub struct LoginConfig {
    pub server_name: String,
    pub ip: String,
    pub game_port: u16,
    pub location: String,
    /// 0 = open-pvp, 1 = no-pvp, 2 = pvp-enforced (mirrors C++ `pvptype`).
    pub pvp_type: u8,
}

// ── Internal response shape (mirrors C++ JSON output exactly) ────────────────

#[derive(Serialize)]
struct SessionResponse {
    sessionkey: String,
    lastlogintime: u64,
    ispremium: bool,
    premiumuntil: i64,
    status: &'static str,
    returnernotification: bool,
    showrewardnews: bool,
    isreturner: bool,
    recoverysetupcomplete: bool,
    fpstracking: bool,
    optiontracking: bool,
}

#[derive(Serialize)]
struct WorldResponse {
    id: u8,
    name: String,
    externaladdressprotected: String,
    externalportprotected: u16,
    externaladdressunprotected: String,
    externalportunprotected: u16,
    previewstate: u8,
    location: String,
    anticheatprotection: bool,
    pvptype: u8,
}

#[derive(Serialize)]
struct CharacterResponse {
    worldid: u8,
    name: String,
    level: u32,
    vocation: String,
    lastlogin: u64,
    ismale: bool,
    ishidden: bool,
    ismaincharacter: bool,
    tutorial: bool,
    outfitid: u32,
    headcolor: u32,
    torsocolor: u32,
    legscolor: u32,
    detailcolor: u32,
    addonsflags: u32,
    dailyrewardstate: u8,
}

#[derive(Serialize)]
struct PlaydataResponse {
    worlds: Vec<WorldResponse>,
    characters: Vec<CharacterResponse>,
}

#[derive(Serialize)]
struct LoginResponse {
    session: SessionResponse,
    playdata: PlaydataResponse,
}

// ── Error helper ──────────────────────────────────────────────────────────────

fn login_error(code: u16, message: &str) -> (u16, String) {
    (
        200,
        format!(r#"{{"errorCode":{},"errorMessage":"{}"}}"#, code, message),
    )
}

// ── Main DB-backed login function ─────────────────────────────────────────────

/// Handle a login request against the real database.
///
/// Always returns HTTP 200. Errors are encoded as JSON `errorCode` / `errorMessage`
/// fields, matching the C++ `tfs::http::handle_login` contract.
///
/// `now_secs` is injectable so that TOTP window tests do not depend on the wall
/// clock.
pub fn handle_login_db(
    db: &mut (dyn Database + Send),
    req: &LoginRequest,
    ip: &str,
    config: &LoginConfig,
    vocations: &Vocations,
    now_secs: u64,
) -> (u16, String) {
    // ── 1. Look up account by email ──────────────────────────────────────────
    let escaped_email = db.escape_string(&req.email);
    let account_sql = format!(
        "SELECT id, password, secret, premium_ends_at FROM accounts WHERE email = '{escaped_email}'"
    );

    let account_rows = match db.query(&account_sql) {
        Ok(rows) => rows,
        Err(_) => {
            return login_error(3, "Tibia account email address or Tibia password is not correct.")
        }
    };

    if account_rows.is_empty() {
        return login_error(
            3,
            "Tibia account email address or Tibia password is not correct.",
        );
    }

    let account_row = &account_rows[0];

    let stored_password_hex = match account_row.get::<String>("password") {
        Some(p) => p,
        None => {
            return login_error(
                3,
                "Tibia account email address or Tibia password is not correct.",
            )
        }
    };

    // ── 2. Verify password (hex SHA-1 comparison) ────────────────────────────
    let supplied_hex = transform_to_sha1_hex(req.password.as_bytes());
    if supplied_hex != stored_password_hex {
        return login_error(
            3,
            "Tibia account email address or Tibia password is not correct.",
        );
    }

    let account_id: i64 = match account_row.get::<i64>("id") {
        Some(id) => id,
        None => {
            return login_error(
                3,
                "Tibia account email address or Tibia password is not correct.",
            )
        }
    };

    let premium_ends_at: i64 = account_row.get::<i64>("premium_ends_at").unwrap_or(0);

    // ── 3. TOTP check ────────────────────────────────────────────────────────
    let secret_str = account_row.get::<String>("secret").unwrap_or_default();
    if !secret_str.is_empty() {
        let token = match req.token.as_deref() {
            Some(t) if !t.is_empty() => t,
            _ => {
                return login_error(
                    6,
                    "Two-factor token required for authentication.",
                )
            }
        };

        let secret_bytes = secret_str.as_bytes();
        let ticks = now_secs / AUTHENTICATOR_PERIOD;
        let candidates = [
            generate_token(secret_bytes, ticks, TOTP_LENGTH),
            generate_token(secret_bytes, ticks.saturating_sub(1), TOTP_LENGTH),
            generate_token(secret_bytes, ticks + 1, TOTP_LENGTH),
        ];
        if !candidates.iter().any(|c| c == token) {
            return login_error(6, "Two-factor token required for authentication.");
        }
    }

    // ── 4. Generate session token and INSERT into sessions ───────────────────
    let token_bytes: [u8; 16] = random();
    let escaped_token = db.escape_blob(&token_bytes);
    let escaped_ip = db.escape_string(ip);
    let session_sql = format!(
        "INSERT INTO sessions (token, account_id, ip) VALUES ({escaped_token}, {account_id}, '{escaped_ip}')"
    );

    if db.execute(&session_sql).is_err() {
        return login_error(255, "An unexpected error has occurred. Please try again later.");
    }

    let session_key = base64::encode(&token_bytes);

    // ── 5. Query characters for this account ────────────────────────────────
    let char_sql = format!(
        "SELECT id, name, level, vocation, lastlogin, sex, looktype, lookhead, lookbody, looklegs, lookfeet, lookaddons FROM players WHERE account_id = {account_id}"
    );

    let char_rows = db.query(&char_sql).unwrap_or_default();

    let mut characters: Vec<CharacterResponse> = Vec::with_capacity(char_rows.len());
    let mut max_lastlogin: u64 = 0;

    for row in &char_rows {
        let name = row.get::<String>("name").unwrap_or_default();
        let level = row.get::<u32>("level").unwrap_or(1);
        let voc_id = row.get::<u32>("vocation").unwrap_or(0) as u16;
        let lastlogin = row.get::<u64>("lastlogin").unwrap_or(0);
        let sex = row.get::<u32>("sex").unwrap_or(0);
        let looktype = row.get::<u32>("looktype").unwrap_or(0);
        let lookhead = row.get::<u32>("lookhead").unwrap_or(0);
        let lookbody = row.get::<u32>("lookbody").unwrap_or(0);
        let looklegs = row.get::<u32>("looklegs").unwrap_or(0);
        let lookfeet = row.get::<u32>("lookfeet").unwrap_or(0);
        let lookaddons = row.get::<u32>("lookaddons").unwrap_or(0);

        if lastlogin > max_lastlogin {
            max_lastlogin = lastlogin;
        }

        let vocation_name = vocations
            .get_vocation(voc_id)
            .map(|v| v.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        characters.push(CharacterResponse {
            worldid: 0,
            name,
            level,
            vocation: vocation_name,
            lastlogin,
            ismale: sex == 1,
            ishidden: false,
            ismaincharacter: false,
            tutorial: false,
            outfitid: looktype,
            headcolor: lookhead,
            torsocolor: lookbody,
            legscolor: looklegs,
            detailcolor: lookfeet,
            addonsflags: lookaddons,
            dailyrewardstate: 0,
        });
    }

    // ── 6. Build and serialize response ─────────────────────────────────────
    let is_premium = premium_ends_at > 0 && (premium_ends_at as u64) > now_secs;

    let response = LoginResponse {
        session: SessionResponse {
            sessionkey: session_key,
            lastlogintime: max_lastlogin,
            ispremium: is_premium,
            premiumuntil: premium_ends_at,
            status: "active",
            returnernotification: false,
            showrewardnews: true,
            isreturner: true,
            recoverysetupcomplete: true,
            fpstracking: false,
            optiontracking: false,
        },
        playdata: PlaydataResponse {
            worlds: vec![WorldResponse {
                id: 0,
                name: config.server_name.clone(),
                externaladdressprotected: config.ip.clone(),
                externalportprotected: config.game_port,
                externaladdressunprotected: config.ip.clone(),
                externalportunprotected: config.game_port,
                previewstate: 0,
                location: config.location.clone(),
                anticheatprotection: false,
                pvptype: config.pvp_type,
            }],
            characters,
        },
    };

    let json = serde_json::to_string(&response)
        .unwrap_or_else(|_| r#"{"errorCode":255,"errorMessage":"Serialization error."}"#.to_string());

    (200, json)
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

    // ── handle_login_db tests (tasks 3.10) ────────────────────────────────────

    use forgottenserver_database::database::{DbError, DbValue, Row};

    /// A test-only Database implementation that actually processes simple
    /// SELECT and INSERT queries against in-memory tables, so that
    /// `handle_login_db` can be exercised without a real MariaDB.
    struct TestDb {
        /// table_name → rows
        tables: HashMap<String, Vec<Row>>,
        pub executed: Vec<String>,
    }

    impl TestDb {
        fn new() -> Self {
            Self {
                tables: HashMap::new(),
                executed: Vec::new(),
            }
        }

        fn insert_row(&mut self, table: &str, cols: Vec<(&str, DbValue)>) {
            let map: HashMap<String, DbValue> =
                cols.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
            self.tables
                .entry(table.to_string())
                .or_default()
                .push(Row::new(map));
        }
    }

    impl Database for TestDb {
        fn query(&self, sql: &str) -> Result<Vec<Row>, DbError> {
            // Minimal parse: extract table name from "SELECT ... FROM <table> WHERE ..."
            // or "SELECT ... FROM <table>" (no WHERE).
            let upper = sql.to_uppercase();
            let from_pos = match upper.find(" FROM ") {
                Some(p) => p + 6,
                None => return Ok(vec![]),
            };
            let rest = sql[from_pos..].trim_start();
            let table_end = rest
                .find(|c: char| c.is_whitespace())
                .unwrap_or(rest.len());
            let table = &rest[..table_end];

            let rows = self.tables.get(table).cloned().unwrap_or_default();

            // For the accounts query: filter by email value extracted from WHERE clause.
            if table == "accounts" && upper.contains("WHERE") {
                // Extract the email value from: WHERE email = '<value>'
                if let Some(eq_pos) = sql.find("email = '") {
                    let val_start = eq_pos + "email = '".len();
                    let val_end = sql[val_start..].find('\'').map(|p| val_start + p);
                    if let Some(end) = val_end {
                        let email_val = &sql[val_start..end];
                        // Unescape \' → '
                        let email_val = email_val.replace("\\'", "'");
                        return Ok(rows
                            .into_iter()
                            .filter(|r| {
                                r.get::<String>("email")
                                    .as_deref()
                                    .unwrap_or("")
                                    == email_val
                            })
                            .collect());
                    }
                }
            }

            // For the players query: filter by account_id.
            if table == "players" && upper.contains("WHERE") {
                if let Some(pos) = sql.find("account_id = ") {
                    let id_start = pos + "account_id = ".len();
                    let id_end = sql[id_start..]
                        .find(|c: char| !c.is_ascii_digit())
                        .map(|p| id_start + p)
                        .unwrap_or(sql.len());
                    if let Ok(account_id) = sql[id_start..id_end].parse::<i64>() {
                        return Ok(rows
                            .into_iter()
                            .filter(|r| r.get::<i64>("account_id") == Some(account_id))
                            .collect());
                    }
                }
            }

            Ok(rows)
        }

        fn execute(&mut self, sql: &str) -> Result<u64, DbError> {
            self.executed.push(sql.to_string());
            Ok(1)
        }

        fn escape_string(&self, s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            for ch in s.chars() {
                match ch {
                    '\\' => out.push_str("\\\\"),
                    '\'' => out.push_str("\\'"),
                    c => out.push(c),
                }
            }
            out
        }
    }

    fn make_vocations_empty() -> Vocations {
        Vocations::load_from_xml("<vocations></vocations>").unwrap()
    }

    fn make_vocations_with_sorcerer() -> Vocations {
        Vocations::load_from_xml(
            r#"<vocations>
                <vocation id="1" name="Sorcerer" clientid="1"/>
            </vocations>"#,
        )
        .unwrap()
    }

    fn make_config() -> LoginConfig {
        LoginConfig {
            server_name: "TestWorld".to_string(),
            ip: "127.0.0.1".to_string(),
            game_port: 7172,
            location: "Europe".to_string(),
            pvp_type: 0,
        }
    }

    fn make_login_req(email: &str, password: &str) -> LoginRequest {
        LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
            token: None,
        }
    }

    fn make_login_req_with_token(email: &str, password: &str, token: &str) -> LoginRequest {
        LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
            token: Some(token.to_string()),
        }
    }

    /// Insert a minimal account row. Password is stored as a hex SHA-1 string.
    fn insert_account(
        db: &mut TestDb,
        id: i64,
        email: &str,
        password_plain: &str,
        secret: &str,
        premium_ends_at: i64,
    ) {
        let pwd_hex = transform_to_sha1_hex(password_plain.as_bytes());
        db.insert_row(
            "accounts",
            vec![
                ("id", DbValue::Integer(id)),
                ("email", DbValue::Text(email.to_string())),
                ("password", DbValue::Text(pwd_hex)),
                ("secret", DbValue::Text(secret.to_string())),
                ("premium_ends_at", DbValue::Integer(premium_ends_at)),
            ],
        );
    }

    // ── db login tests ────────────────────────────────────────────────────────

    #[test]
    fn db_login_unknown_account_returns_error_3() {
        let mut db = TestDb::new();
        let req = make_login_req("nobody@example.com", "pass");
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 0);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["errorCode"], 3);
    }

    #[test]
    fn db_login_wrong_password_returns_error_3() {
        let mut db = TestDb::new();
        insert_account(&mut db, 1, "alice@example.com", "correct", "", 0);
        let req = make_login_req("alice@example.com", "wrong");
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 0);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["errorCode"], 3);
    }

    #[test]
    fn db_login_totp_required_when_secret_present_and_token_absent() {
        let mut db = TestDb::new();
        insert_account(&mut db, 1, "alice@example.com", "pass", "12345678901234567890", 0);
        let req = make_login_req("alice@example.com", "pass");
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 0);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["errorCode"], 6);
    }

    #[test]
    fn db_login_totp_wrong_window_returns_error_6() {
        let mut db = TestDb::new();
        let secret = "12345678901234567890";
        insert_account(&mut db, 1, "alice@example.com", "pass", secret, 0);
        // Use a token from 2 windows ago (outside the ±1 acceptance window)
        let now = 1_500_000_000u64;
        let ticks = now / AUTHENTICATOR_PERIOD;
        let old_token = generate_token(secret.as_bytes(), ticks - 2, TOTP_LENGTH);
        let req = make_login_req_with_token("alice@example.com", "pass", &old_token);
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, now);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["errorCode"], 6);
    }

    #[test]
    fn db_login_totp_correct_current_window_succeeds() {
        let mut db = TestDb::new();
        let secret = "12345678901234567890";
        insert_account(&mut db, 1, "alice@example.com", "pass", secret, 0);
        let now = 1_500_000_000u64;
        let ticks = now / AUTHENTICATOR_PERIOD;
        let token = generate_token(secret.as_bytes(), ticks, TOTP_LENGTH);
        let req = make_login_req_with_token("alice@example.com", "pass", &token);
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, now);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(v.get("errorCode").is_none(), "expected success, got: {body}");
        assert!(v["session"].is_object());
    }

    #[test]
    fn db_login_session_key_is_nonempty_base64() {
        let mut db = TestDb::new();
        insert_account(&mut db, 1, "alice@example.com", "pass", "", 0);
        let req = make_login_req("alice@example.com", "pass");
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 0);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let key = v["session"]["sessionkey"].as_str().unwrap();
        assert!(!key.is_empty());
        // Verify it's valid base64 (16 raw bytes → 24 base64 chars with padding)
        let decoded = forgottenserver_common::base64::decode(key).unwrap();
        assert_eq!(decoded.len(), 16);
    }

    #[test]
    fn db_login_ispremium_true_when_premium_ends_at_in_future() {
        let mut db = TestDb::new();
        let now_secs = 1_000_000u64;
        let premium_ends = (now_secs + 86400) as i64; // 1 day in the future
        insert_account(&mut db, 1, "alice@example.com", "pass", "", premium_ends);
        let req = make_login_req("alice@example.com", "pass");
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) =
            handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, now_secs);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["session"]["ispremium"], true);
    }

    #[test]
    fn db_login_ispremium_false_when_premium_ends_at_zero() {
        let mut db = TestDb::new();
        insert_account(&mut db, 1, "alice@example.com", "pass", "", 0);
        let req = make_login_req("alice@example.com", "pass");
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 1000);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["session"]["ispremium"], false);
    }

    #[test]
    fn db_login_character_list_populated_with_player_rows() {
        let mut db = TestDb::new();
        insert_account(&mut db, 42, "alice@example.com", "pass", "", 0);
        // Insert a character
        db.insert_row(
            "players",
            vec![
                ("id", DbValue::Integer(1)),
                ("account_id", DbValue::Integer(42)),
                ("name", DbValue::Text("Alice".to_string())),
                ("level", DbValue::Integer(10)),
                ("vocation", DbValue::Integer(1)),
                ("lastlogin", DbValue::Integer(999)),
                ("sex", DbValue::Integer(1)),
                ("looktype", DbValue::Integer(128)),
                ("lookhead", DbValue::Integer(0)),
                ("lookbody", DbValue::Integer(0)),
                ("looklegs", DbValue::Integer(0)),
                ("lookfeet", DbValue::Integer(0)),
                ("lookaddons", DbValue::Integer(0)),
            ],
        );
        let req = make_login_req("alice@example.com", "pass");
        let vocs = make_vocations_with_sorcerer();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 0);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let chars = v["playdata"]["characters"].as_array().unwrap();
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0]["name"], "Alice");
        assert_eq!(chars[0]["level"], 10);
        assert_eq!(chars[0]["vocation"], "Sorcerer");
        assert_eq!(chars[0]["ismale"], true);
    }

    #[test]
    fn db_login_world_array_has_exactly_one_entry_with_correct_server_name() {
        let mut db = TestDb::new();
        insert_account(&mut db, 1, "alice@example.com", "pass", "", 0);
        let req = make_login_req("alice@example.com", "pass");
        let vocs = make_vocations_empty();
        let config = LoginConfig {
            server_name: "MySpecialWorld".to_string(),
            ip: "10.0.0.1".to_string(),
            game_port: 7172,
            location: "NA".to_string(),
            pvp_type: 1,
        };
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 0);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let worlds = v["playdata"]["worlds"].as_array().unwrap();
        assert_eq!(worlds.len(), 1);
        assert_eq!(worlds[0]["name"], "MySpecialWorld");
        assert_eq!(worlds[0]["pvptype"], 1);
    }

    #[test]
    fn db_login_missing_email_no_db_row_returns_error_3() {
        let mut db = TestDb::new();
        // Empty email, no row in DB
        let req = make_login_req("", "pass");
        let vocs = make_vocations_empty();
        let config = make_config();
        let (status, body) = handle_login_db(&mut db, &req, "127.0.0.1", &config, &vocs, 0);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["errorCode"], 3);
    }
}
