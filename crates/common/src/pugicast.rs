/// Type-casting helpers for XML attribute/text values (adapted for roxmltree instead of pugixml).
pub fn cast_str_to_u8(s: &str) -> Option<u8> {
    s.trim().parse().ok()
}
pub fn cast_str_to_u16(s: &str) -> Option<u16> {
    s.trim().parse().ok()
}
pub fn cast_str_to_u32(s: &str) -> Option<u32> {
    s.trim().parse().ok()
}
pub fn cast_str_to_i32(s: &str) -> Option<i32> {
    s.trim().parse().ok()
}
pub fn cast_str_to_f32(s: &str) -> Option<f32> {
    s.trim().parse().ok()
}
pub fn cast_str_to_bool(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

pub fn cast_str_to_u32_or(s: &str, default: u32) -> u32 {
    cast_str_to_u32(s).unwrap_or(default)
}
pub fn cast_str_to_i32_or(s: &str, default: i32) -> i32 {
    cast_str_to_i32(s).unwrap_or(default)
}
pub fn cast_str_to_bool_or(s: &str, default: bool) -> bool {
    cast_str_to_bool(s).unwrap_or(default)
}
pub fn cast_str_to_f32_or(s: &str, default: f32) -> f32 {
    cast_str_to_f32(s).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast_u8_valid() {
        assert_eq!(cast_str_to_u8("42"), Some(42));
    }

    #[test]
    fn cast_u8_invalid() {
        assert_eq!(cast_str_to_u8("abc"), None);
    }

    #[test]
    fn cast_u8_overflow() {
        assert_eq!(cast_str_to_u8("300"), None);
    }

    #[test]
    fn cast_u16_valid() {
        assert_eq!(cast_str_to_u16("1000"), Some(1000));
    }

    #[test]
    fn cast_u32_valid() {
        assert_eq!(cast_str_to_u32("99999"), Some(99999));
    }

    #[test]
    fn cast_u32_invalid() {
        assert_eq!(cast_str_to_u32("nope"), None);
    }

    #[test]
    fn cast_i32_negative() {
        assert_eq!(cast_str_to_i32("-5"), Some(-5));
    }

    #[test]
    fn cast_f32_valid() {
        // Use a non-approximated value so clippy doesn't flag this as an
        // approximate `PI` literal.
        let v = cast_str_to_f32("3.25").unwrap();
        assert!((v - 3.25_f32).abs() < 0.001);
    }

    #[test]
    fn cast_f32_invalid() {
        assert_eq!(cast_str_to_f32("xyz"), None);
    }

    #[test]
    fn cast_bool_true_variants() {
        assert_eq!(cast_str_to_bool("true"), Some(true));
        assert_eq!(cast_str_to_bool("1"), Some(true));
        assert_eq!(cast_str_to_bool("yes"), Some(true));
    }

    #[test]
    fn cast_bool_false_variants() {
        assert_eq!(cast_str_to_bool("false"), Some(false));
        assert_eq!(cast_str_to_bool("0"), Some(false));
        assert_eq!(cast_str_to_bool("no"), Some(false));
    }

    #[test]
    fn cast_bool_invalid() {
        assert_eq!(cast_str_to_bool("maybe"), None);
    }

    #[test]
    fn cast_bool_case_insensitive() {
        assert_eq!(cast_str_to_bool("TRUE"), Some(true));
    }

    #[test]
    fn cast_str_to_u32_or_uses_default_on_invalid() {
        assert_eq!(cast_str_to_u32_or("bad", 99), 99);
    }

    #[test]
    fn cast_str_to_u32_or_returns_value_on_valid() {
        assert_eq!(cast_str_to_u32_or("42", 99), 42);
    }

    #[test]
    fn cast_str_to_i32_or_uses_default() {
        assert_eq!(cast_str_to_i32_or("?", -1), -1);
    }

    #[test]
    fn cast_str_to_bool_or_uses_default() {
        assert!(cast_str_to_bool_or("?", true));
    }

    #[test]
    fn cast_str_to_f32_or_uses_default() {
        let v = cast_str_to_f32_or("?", 1.5);
        assert!((v - 1.5_f32).abs() < 0.001);
    }

    #[test]
    fn whitespace_trimmed() {
        assert_eq!(cast_str_to_u32("  7  "), Some(7));
    }

    // --- u16 edge cases -----------------------------------------------------
    #[test]
    fn cast_u16_invalid() {
        assert_eq!(cast_str_to_u16("not-a-number"), None);
    }

    #[test]
    fn cast_u16_overflow() {
        // u16::MAX = 65535; 70000 must fail to parse
        assert_eq!(cast_str_to_u16("70000"), None);
    }

    #[test]
    fn cast_u16_max_boundary() {
        assert_eq!(cast_str_to_u16("65535"), Some(u16::MAX));
    }

    // --- u32 edge cases -----------------------------------------------------
    #[test]
    fn cast_u32_overflow() {
        // u32::MAX = 4_294_967_295; 5_000_000_000 must overflow
        assert_eq!(cast_str_to_u32("5000000000"), None);
    }

    #[test]
    fn cast_u32_max_boundary() {
        assert_eq!(cast_str_to_u32("4294967295"), Some(u32::MAX));
    }

    #[test]
    fn cast_u32_negative_rejected() {
        // Mirrors C++ strtoul behavior: unsigned parse must not accept negatives in our Rust impl.
        assert_eq!(cast_str_to_u32("-1"), None);
    }

    // --- u8 boundary ---------------------------------------------------------
    #[test]
    fn cast_u8_max_boundary() {
        assert_eq!(cast_str_to_u8("255"), Some(u8::MAX));
    }

    // --- i32 happy + edge cases ---------------------------------------------
    #[test]
    fn cast_i32_positive() {
        assert_eq!(cast_str_to_i32("12345"), Some(12345));
    }

    #[test]
    fn cast_i32_invalid() {
        assert_eq!(cast_str_to_i32("abc"), None);
    }

    #[test]
    fn cast_i32_overflow_positive() {
        // i32::MAX = 2_147_483_647; one above must fail.
        assert_eq!(cast_str_to_i32("2147483648"), None);
    }

    #[test]
    fn cast_i32_overflow_negative() {
        // i32::MIN = -2_147_483_648; one below must fail.
        assert_eq!(cast_str_to_i32("-2147483649"), None);
    }

    #[test]
    fn cast_i32_min_boundary() {
        assert_eq!(cast_str_to_i32("-2147483648"), Some(i32::MIN));
    }

    #[test]
    fn cast_i32_max_boundary() {
        assert_eq!(cast_str_to_i32("2147483647"), Some(i32::MAX));
    }

    // --- f32 meaningful parses ----------------------------------------------
    #[test]
    fn cast_f32_scientific_notation() {
        // Mirrors strtof; e.g. "1.5e2" == 150.0
        let v = cast_str_to_f32("1.5e2").unwrap();
        assert!((v - 150.0_f32).abs() < 0.001);
    }

    #[test]
    fn cast_f32_negative() {
        let v = cast_str_to_f32("-2.5").unwrap();
        assert!((v + 2.5_f32).abs() < 0.001);
    }

    #[test]
    fn cast_f32_whitespace_trimmed() {
        let v = cast_str_to_f32("  4.25  ").unwrap();
        assert!((v - 4.25_f32).abs() < 0.001);
    }

    // --- bool edge cases ----------------------------------------------------
    #[test]
    fn cast_bool_empty_string() {
        assert_eq!(cast_str_to_bool(""), None);
    }

    #[test]
    fn cast_bool_mixed_case_false() {
        assert_eq!(cast_str_to_bool("False"), Some(false));
        assert_eq!(cast_str_to_bool("YES"), Some(true));
        assert_eq!(cast_str_to_bool("No"), Some(false));
    }

    #[test]
    fn cast_bool_whitespace_trimmed() {
        assert_eq!(cast_str_to_bool("  true  "), Some(true));
        assert_eq!(cast_str_to_bool("  no  "), Some(false));
    }

    // --- Empty-string edge cases (every numeric cast) -----------------------
    #[test]
    fn cast_u8_empty_string() {
        assert_eq!(cast_str_to_u8(""), None);
    }

    #[test]
    fn cast_u16_empty_string() {
        assert_eq!(cast_str_to_u16(""), None);
    }

    #[test]
    fn cast_u32_empty_string() {
        assert_eq!(cast_str_to_u32(""), None);
    }

    #[test]
    fn cast_i32_empty_string() {
        assert_eq!(cast_str_to_i32(""), None);
    }

    #[test]
    fn cast_f32_empty_string() {
        assert_eq!(cast_str_to_f32(""), None);
    }

    // --- *_or returning the parsed value branch -----------------------------
    #[test]
    fn cast_str_to_i32_or_returns_value_on_valid() {
        assert_eq!(cast_str_to_i32_or("-42", 7), -42);
    }

    #[test]
    fn cast_str_to_bool_or_returns_value_on_valid() {
        assert!(cast_str_to_bool_or("yes", false));
        assert!(!cast_str_to_bool_or("no", true));
    }

    #[test]
    fn cast_str_to_f32_or_returns_value_on_valid() {
        let v = cast_str_to_f32_or("2.5", 0.0);
        assert!((v - 2.5_f32).abs() < 0.001);
    }

    #[test]
    fn cast_str_to_u32_or_empty_string_uses_default() {
        assert_eq!(cast_str_to_u32_or("", 999), 999);
    }
}
