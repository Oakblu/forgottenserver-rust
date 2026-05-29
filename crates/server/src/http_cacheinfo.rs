#[derive(Debug, Clone, PartialEq)]
pub struct CacheInfo {
    pub cache_control: String,
    pub etag: String,
    pub max_age_seconds: u32,
}

impl CacheInfo {
    pub fn new(max_age_seconds: u32) -> Self {
        Self {
            cache_control: format!("max-age={}", max_age_seconds),
            etag: format!("\"{}\"", max_age_seconds),
            max_age_seconds,
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"cache_control":"{}","etag":"{}","max_age_seconds":{}}}"#,
            self.cache_control, self.etag, self.max_age_seconds
        )
    }

    pub fn no_cache() -> Self {
        Self {
            cache_control: "no-cache".to_string(),
            etag: "\"0\"".to_string(),
            max_age_seconds: 0,
        }
    }
}

/// Rust counterpart of `tfs::http::handle_cacheinfo` from
/// `forgottenserver/src/http/cacheinfo.cpp`.
///
/// The C++ implementation queries the database for
/// `SELECT COUNT(*) AS count FROM players_online`, returning either
///   - `{status::ok, {{"playersonline", count}}}` on success, or
///   - `make_error_response()` (HTTP 200 with an error envelope) on DB failure.
///
/// In the Rust port the DB is not wired into the server crate, so the player
/// count is injected as a `Result<u32, _>`-shaped `Option<u32>`:
///   - `Some(count)` — query succeeded, returns the success body
///   - `None`        — query failed, returns the error envelope
///
/// `body` and `ip` are accepted (and currently unused) to mirror the C++
/// signature exactly. They are kept for future request-context wiring.
pub fn handle_cacheinfo(_body: &str, _ip: &str, player_count: Option<u32>) -> (u16, String) {
    match player_count {
        Some(count) => (200, format!(r#"{{"playersonline":{}}}"#, count)),
        None => (
            200,
            r#"{"errorCode":2,"errorMessage":"Internal error. Please try again later or contact customer support if the problem persists."}"#
                .to_string(),
        ),
    }
}

/// Live variant of `handle_cacheinfo` that queries the real database.
///
/// Executes `SELECT COUNT(*) AS count FROM players_online` and returns:
/// - `(200, {"playersonline":<n>})` on success, where `<n>` is the row count
///   (defaulting to 0 if the result set is empty or the column is missing).
/// - `(200, {"errorCode":2,"errorMessage":"..."})` on any `DbError`.
///
/// HTTP status is always 200 to match the C++ `status::ok` return in both
/// the success path and the `make_error_response()` error path.
pub fn handle_cacheinfo_db(
    db: &(dyn forgottenserver_database::database::Database + Send),
) -> (u16, String) {
    match db.query("SELECT COUNT(*) AS count FROM players_online") {
        Ok(rows) => {
            let count: u32 = rows
                .first()
                .and_then(|row| row.get::<u32>("count"))
                .unwrap_or(0);
            (200, format!(r#"{{"playersonline":{}}}"#, count))
        }
        Err(_) => (
            200,
            r#"{"errorCode":2,"errorMessage":"Internal error. Please try again later or contact customer support if the problem persists."}"#
                .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_json_contains_cache_control_field() {
        let info = CacheInfo::new(300);
        let json = info.to_json();
        assert!(
            json.contains("cache_control"),
            "JSON missing 'cache_control': {}",
            json
        );
    }

    #[test]
    fn to_json_contains_etag_field() {
        let info = CacheInfo::new(300);
        let json = info.to_json();
        assert!(json.contains("etag"), "JSON missing 'etag': {}", json);
    }

    #[test]
    fn to_json_contains_max_age_seconds_field() {
        let info = CacheInfo::new(300);
        let json = info.to_json();
        assert!(
            json.contains("max_age_seconds"),
            "JSON missing 'max_age_seconds': {}",
            json
        );
    }

    #[test]
    fn no_cache_has_max_age_zero() {
        let info = CacheInfo::no_cache();
        assert_eq!(info.max_age_seconds, 0);
    }

    #[test]
    fn no_cache_has_no_cache_string() {
        let info = CacheInfo::no_cache();
        assert_eq!(info.cache_control, "no-cache");
    }

    #[test]
    fn new_300_has_correct_max_age() {
        let info = CacheInfo::new(300);
        assert_eq!(info.max_age_seconds, 300);
    }

    #[test]
    fn new_sets_cache_control_with_max_age() {
        let info = CacheInfo::new(60);
        assert!(info.cache_control.contains("60"));
    }

    // --- handle_cacheinfo: mirrors tfs::http::handle_cacheinfo behaviour ---

    #[test]
    fn handle_cacheinfo_success_returns_status_ok() {
        let (status, _body) = handle_cacheinfo("{}", "127.0.0.1", Some(42));
        assert_eq!(
            status, 200,
            "C++ returns boost::beast::http::status::ok on success"
        );
    }

    #[test]
    fn handle_cacheinfo_success_body_contains_playersonline_key() {
        let (_status, body) = handle_cacheinfo("{}", "127.0.0.1", Some(42));
        assert!(
            body.contains("\"playersonline\""),
            "expected JSON key 'playersonline' in body: {}",
            body
        );
    }

    #[test]
    fn handle_cacheinfo_success_body_contains_player_count_value() {
        let (_status, body) = handle_cacheinfo("{}", "127.0.0.1", Some(137));
        assert!(body.contains("137"), "expected count 137 in body: {}", body);
    }

    #[test]
    fn handle_cacheinfo_success_with_zero_players() {
        let (status, body) = handle_cacheinfo("{}", "10.0.0.1", Some(0));
        assert_eq!(status, 200);
        assert!(body.contains("\"playersonline\":0"), "body was: {}", body);
    }

    #[test]
    fn handle_cacheinfo_db_failure_returns_error_envelope() {
        let (status, body) = handle_cacheinfo("{}", "127.0.0.1", None);
        // C++ make_error_response returns status::ok with error body
        assert_eq!(status, 200);
        assert!(
            body.contains("errorCode"),
            "expected 'errorCode' in error body: {}",
            body
        );
        assert!(
            body.contains("errorMessage"),
            "expected 'errorMessage' in error body: {}",
            body
        );
    }

    #[test]
    fn handle_cacheinfo_db_failure_error_code_is_2() {
        // C++ default ErrorResponseParams::code = 2
        let (_status, body) = handle_cacheinfo("{}", "127.0.0.1", None);
        assert!(
            body.contains("\"errorCode\":2"),
            "expected errorCode=2 in body: {}",
            body
        );
    }

    #[test]
    fn handle_cacheinfo_ignores_body_and_ip_arguments() {
        // C++ implementation takes both parameters but uses neither; verify
        // varying them does not change the observable response.
        let (s1, b1) = handle_cacheinfo("{}", "127.0.0.1", Some(5));
        let (s2, b2) = handle_cacheinfo(r#"{"unused":true}"#, "::1", Some(5));
        assert_eq!(s1, s2);
        assert_eq!(b1, b2);
    }

    #[test]
    fn handle_cacheinfo_success_body_is_valid_json_shape() {
        let (_status, body) = handle_cacheinfo("{}", "127.0.0.1", Some(9));
        assert!(
            body.starts_with('{') && body.ends_with('}'),
            "not JSON-shaped: {}",
            body
        );
    }

    // ── handle_cacheinfo_db tests ────────────────────────────────────────────

    use forgottenserver_database::database::{DbError, DbValue, Database, Row};
    use std::collections::HashMap;

    /// Minimal mock DB whose `query` returns a fixed result.
    struct MockDb {
        result: Result<Vec<Row>, DbError>,
    }

    impl MockDb {
        fn with_count(count: u32) -> Self {
            let mut cols = HashMap::new();
            cols.insert("count".to_string(), DbValue::Integer(count as i64));
            let row = Row::new(cols);
            Self { result: Ok(vec![row]) }
        }

        fn with_error(err: DbError) -> Self {
            Self { result: Err(err) }
        }

        fn empty() -> Self {
            Self { result: Ok(vec![]) }
        }
    }

    impl Database for MockDb {
        fn query(&self, _sql: &str) -> Result<Vec<Row>, DbError> {
            self.result.clone()
        }

        fn execute(&mut self, _sql: &str) -> Result<u64, DbError> {
            Ok(0)
        }

        fn escape_string(&self, s: &str) -> String {
            s.to_string()
        }
    }

    #[test]
    fn handle_cacheinfo_db_zero_players_returns_playersonline_zero() {
        let db = MockDb::with_count(0);
        let (status, body) = handle_cacheinfo_db(&db);
        assert_eq!(status, 200);
        assert_eq!(body, r#"{"playersonline":0}"#);
    }

    #[test]
    fn handle_cacheinfo_db_nonzero_count_returns_correct_value() {
        let db = MockDb::with_count(42);
        let (status, body) = handle_cacheinfo_db(&db);
        assert_eq!(status, 200);
        assert_eq!(body, r#"{"playersonline":42}"#);
    }

    #[test]
    fn handle_cacheinfo_db_large_count_returned_correctly() {
        let db = MockDb::with_count(1000);
        let (_status, body) = handle_cacheinfo_db(&db);
        assert_eq!(body, r#"{"playersonline":1000}"#);
    }

    #[test]
    fn handle_cacheinfo_db_query_error_returns_error_envelope() {
        let db = MockDb::with_error(DbError::QueryError("table not found".to_string()));
        let (status, body) = handle_cacheinfo_db(&db);
        assert_eq!(status, 200, "HTTP status must be 200 even on DB error");
        assert!(
            body.contains("\"errorCode\":2"),
            "expected errorCode=2 in body: {}",
            body
        );
        assert!(
            body.contains("\"errorMessage\""),
            "expected errorMessage in body: {}",
            body
        );
    }

    #[test]
    fn handle_cacheinfo_db_connection_error_returns_error_envelope() {
        let db = MockDb::with_error(DbError::ConnectionFailed);
        let (status, body) = handle_cacheinfo_db(&db);
        assert_eq!(status, 200);
        assert!(body.contains("\"errorCode\":2"), "body: {}", body);
    }

    #[test]
    fn handle_cacheinfo_db_empty_result_set_returns_zero() {
        // When the result set is empty (no rows), default to 0 online players.
        let db = MockDb::empty();
        let (status, body) = handle_cacheinfo_db(&db);
        assert_eq!(status, 200);
        assert_eq!(body, r#"{"playersonline":0}"#);
    }

    #[test]
    fn handle_cacheinfo_db_status_always_200_on_success() {
        let db = MockDb::with_count(5);
        let (status, _body) = handle_cacheinfo_db(&db);
        assert_eq!(status, 200);
    }

    #[test]
    fn handle_cacheinfo_db_status_always_200_on_error() {
        let db = MockDb::with_error(DbError::NotFound);
        let (status, _body) = handle_cacheinfo_db(&db);
        assert_eq!(status, 200);
    }
}
