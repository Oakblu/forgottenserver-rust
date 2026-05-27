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
}
