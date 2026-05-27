/// A value that can be passed to or returned from a script.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    Str(String),
}

impl From<bool> for ScriptValue {
    fn from(v: bool) -> Self {
        ScriptValue::Bool(v)
    }
}

impl From<i64> for ScriptValue {
    fn from(v: i64) -> Self {
        ScriptValue::Integer(v)
    }
}

impl From<f64> for ScriptValue {
    fn from(v: f64) -> Self {
        ScriptValue::Float(v)
    }
}

impl From<String> for ScriptValue {
    fn from(v: String) -> Self {
        ScriptValue::Str(v)
    }
}

impl From<&str> for ScriptValue {
    fn from(v: &str) -> Self {
        ScriptValue::Str(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bool_true() {
        assert_eq!(ScriptValue::from(true), ScriptValue::Bool(true));
    }

    #[test]
    fn from_bool_false() {
        assert_eq!(ScriptValue::from(false), ScriptValue::Bool(false));
    }

    #[test]
    fn from_i64() {
        assert_eq!(ScriptValue::from(42i64), ScriptValue::Integer(42));
    }

    #[test]
    fn from_f64() {
        assert_eq!(
            ScriptValue::from(std::f64::consts::PI),
            ScriptValue::Float(std::f64::consts::PI)
        );
    }

    #[test]
    fn from_string() {
        assert_eq!(
            ScriptValue::from("hello".to_string()),
            ScriptValue::Str("hello".to_string())
        );
    }

    #[test]
    fn from_str_ref() {
        assert_eq!(
            ScriptValue::from("world"),
            ScriptValue::Str("world".to_string())
        );
    }

    #[test]
    fn nil_variant() {
        assert_eq!(ScriptValue::Nil, ScriptValue::Nil);
    }
}
