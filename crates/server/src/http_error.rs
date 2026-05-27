#[derive(Debug, Clone, PartialEq)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: u16, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        // Wire field names mirror C++: the upstream tibia auth-protocol
        // payloads use `errorCode` / `errorMessage`. Keeping those exact
        // strings matters because real clients (and the sibling
        // `http_cacheinfo` module) deserialise on these keys. The Rust
        // struct keeps idiomatic `code` / `message` field names — only
        // the serialisation layer renames them.
        format!(
            r#"{{"errorCode":{},"errorMessage":"{}"}}"#,
            self.code, self.message
        )
    }

    pub fn not_found() -> Self {
        Self::new(404, "Not Found")
    }

    pub fn unauthorized() -> Self {
        Self::new(401, "Unauthorized")
    }

    pub fn internal_error() -> Self {
        Self::new(500, "Internal Server Error")
    }
}

pub fn format_error(e: &ErrorResponse) -> String {
    e.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_json_contains_code_and_message() {
        let e = ErrorResponse::new(418, "I'm a teapot");
        let json = e.to_json();
        assert!(json.contains("418"), "missing code: {}", json);
        assert!(json.contains("I'm a teapot"), "missing message: {}", json);
    }

    #[test]
    fn to_json_starts_with_brace() {
        let e = ErrorResponse::new(200, "OK");
        let json = e.to_json();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn not_found_has_code_404() {
        let e = ErrorResponse::not_found();
        assert_eq!(e.code, 404);
    }

    #[test]
    fn unauthorized_has_code_401() {
        let e = ErrorResponse::unauthorized();
        assert_eq!(e.code, 401);
    }

    #[test]
    fn internal_error_has_code_500() {
        let e = ErrorResponse::internal_error();
        assert_eq!(e.code, 500);
    }

    #[test]
    fn format_error_matches_to_json() {
        let e = ErrorResponse::new(403, "Forbidden");
        assert_eq!(format_error(&e), e.to_json());
    }

    #[test]
    fn not_found_message_is_not_found() {
        let e = ErrorResponse::not_found();
        assert_eq!(e.message, "Not Found");
    }

    #[test]
    fn unauthorized_message_is_unauthorized() {
        let e = ErrorResponse::unauthorized();
        assert_eq!(e.message, "Unauthorized");
    }

    #[test]
    fn internal_error_message_is_internal_server_error() {
        let e = ErrorResponse::internal_error();
        assert_eq!(e.message, "Internal Server Error");
    }

    #[test]
    fn new_preserves_code_and_message_fields() {
        let e = ErrorResponse::new(503, "Service Unavailable");
        assert_eq!(e.code, 503);
        assert_eq!(e.message, "Service Unavailable");
    }

    #[test]
    fn to_json_format_is_well_formed() {
        let e = ErrorResponse::new(400, "Bad Request");
        assert_eq!(
            e.to_json(),
            r#"{"errorCode":400,"errorMessage":"Bad Request"}"#
        );
    }

    /// Regression for the JSON field-name parity bug: the upstream tibia
    /// auth wire format uses `errorCode` / `errorMessage` exactly. Any
    /// drift here breaks the sibling `http_cacheinfo` module (which
    /// already expects those keys) and real clients deserialising the
    /// payload.
    #[test]
    fn to_json_uses_camel_case_error_field_names_for_wire_parity() {
        let json = ErrorResponse::new(500, "boom").to_json();
        assert!(
            json.contains(r#""errorCode""#),
            "must contain 'errorCode' for wire parity, got: {json}"
        );
        assert!(
            json.contains(r#""errorMessage""#),
            "must contain 'errorMessage' for wire parity, got: {json}"
        );
        // And the old (incorrect) shorter forms must NOT appear.
        assert!(!json.contains(r#""code""#));
        assert!(!json.contains(r#""message""#));
    }
}
