use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ScriptError {
    #[error("load failed: {0}")]
    LoadFailed(String),
    #[error("call failed: {0}")]
    CallFailed(String),
    #[error("runtime error: {0}")]
    RuntimeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_failed_formats() {
        assert_eq!(
            format!("{}", ScriptError::LoadFailed("file not found".to_string())),
            "load failed: file not found"
        );
    }

    #[test]
    fn call_failed_formats() {
        assert_eq!(
            format!(
                "{}",
                ScriptError::CallFailed("function not defined".to_string())
            ),
            "call failed: function not defined"
        );
    }

    #[test]
    fn runtime_error_formats() {
        assert_eq!(
            format!(
                "{}",
                ScriptError::RuntimeError("stack overflow".to_string())
            ),
            "runtime error: stack overflow"
        );
    }
}
