//! Migrated from forgottenserver/src/definitions.h
//! Maps constexpr string/integer constants to Rust pub const.

#[allow(dead_code)]
pub const STATUS_SERVER_NAME: &str = "The Forgotten Server";

#[allow(dead_code)]
pub const STATUS_SERVER_VERSION: &str = "1.7";

#[allow(dead_code)]
pub const STATUS_SERVER_DEVELOPERS: &str = "The Forgotten Server Team";

#[allow(dead_code)]
pub const CLIENT_VERSION_MIN: i32 = 1310;

#[allow(dead_code)]
pub const CLIENT_VERSION_MAX: i32 = 1311;

#[allow(dead_code)]
pub const CLIENT_VERSION_STR: &str = "13.10";

#[allow(dead_code)]
pub const AUTHENTICATOR_DIGITS: u32 = 6;

#[allow(dead_code)]
pub const AUTHENTICATOR_PERIOD: u32 = 30;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_server_name() {
        assert_eq!(STATUS_SERVER_NAME, "The Forgotten Server");
    }

    #[test]
    fn test_status_server_version() {
        assert_eq!(STATUS_SERVER_VERSION, "1.7");
    }

    #[test]
    fn test_status_server_developers() {
        assert_eq!(STATUS_SERVER_DEVELOPERS, "The Forgotten Server Team");
    }

    #[test]
    fn test_client_version_min() {
        assert_eq!(CLIENT_VERSION_MIN, 1310);
    }

    #[test]
    fn test_client_version_max() {
        assert_eq!(CLIENT_VERSION_MAX, 1311);
    }

    #[test]
    fn test_client_version_str() {
        assert_eq!(CLIENT_VERSION_STR, "13.10");
    }

    #[test]
    fn test_authenticator_digits() {
        assert_eq!(AUTHENTICATOR_DIGITS, 6u32);
    }

    #[test]
    fn test_authenticator_period() {
        assert_eq!(AUTHENTICATOR_PERIOD, 30u32);
    }

    #[test]
    fn test_client_version_min_le_max_invariant() {
        // C++ source guarantees CLIENT_VERSION_MIN (1310) <= CLIENT_VERSION_MAX (1311).
        // This invariant must hold across any future bump.
        const _: () = assert!(CLIENT_VERSION_MIN <= CLIENT_VERSION_MAX);
    }

    #[test]
    fn test_client_version_str_matches_numeric_range() {
        // CLIENT_VERSION_STR "13.10" is the human-readable form of CLIENT_VERSION_MIN 1310.
        // Asserts the documented C++ mapping (1310 -> "13.10") is preserved verbatim.
        assert_eq!(CLIENT_VERSION_STR, "13.10");
        assert_eq!(CLIENT_VERSION_MIN, 1310);
    }

    #[test]
    fn test_constant_types_are_exact() {
        // Pin the exact Rust types matching the C++ declarations.
        // C++ uses `auto` from int literals -> int (i32 in Rust).
        // C++ uses `6U` / `30U` -> unsigned int (u32 in Rust).
        let _min: i32 = CLIENT_VERSION_MIN;
        let _max: i32 = CLIENT_VERSION_MAX;
        let _digits: u32 = AUTHENTICATOR_DIGITS;
        let _period: u32 = AUTHENTICATOR_PERIOD;
        let _name: &str = STATUS_SERVER_NAME;
        let _ver: &str = STATUS_SERVER_VERSION;
        let _devs: &str = STATUS_SERVER_DEVELOPERS;
        let _vstr: &str = CLIENT_VERSION_STR;
    }
}
