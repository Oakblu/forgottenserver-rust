use crate::luascript::{LuaEnvironment, LuaError, LuaValue};

#[derive(Debug, Clone)]
pub struct Script {
    name: String,
    path: String,
    loaded: bool,
}

impl Script {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            loaded: false,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn mark_loaded(&mut self) {
        self.loaded = true;
    }

    pub fn call_on_use<E: LuaEnvironment>(
        &self,
        env: &mut E,
        player_id: i64,
        item_id: i64,
    ) -> Result<Vec<LuaValue>, LuaError> {
        env.call_function(
            "onUse",
            vec![LuaValue::Integer(player_id), LuaValue::Integer(item_id)],
        )
    }

    pub fn call_on_step_in<E: LuaEnvironment>(
        &self,
        env: &mut E,
        creature_id: i64,
        x: i64,
        y: i64,
        z: i64,
    ) -> Result<Vec<LuaValue>, LuaError> {
        env.call_function(
            "onStepIn",
            vec![
                LuaValue::Integer(creature_id),
                LuaValue::Integer(x),
                LuaValue::Integer(y),
                LuaValue::Integer(z),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::luascript::MockLuaEnv;

    #[test]
    fn new_creates_script_entry() {
        let script = Script::new("test_script", "/scripts/test.lua");
        assert_eq!(script.get_name(), "test_script");
        assert_eq!(script.get_path(), "/scripts/test.lua");
    }

    #[test]
    fn get_name_returns_correct_value() {
        let script = Script::new("my_script", "/path/to/script.lua");
        assert_eq!(script.get_name(), "my_script");
    }

    #[test]
    fn get_path_returns_correct_value() {
        let script = Script::new("my_script", "/path/to/script.lua");
        assert_eq!(script.get_path(), "/path/to/script.lua");
    }

    #[test]
    fn is_loaded_returns_false_initially() {
        let script = Script::new("test", "/test.lua");
        assert!(!script.is_loaded());
    }

    #[test]
    fn mark_loaded_then_is_loaded_returns_true() {
        let mut script = Script::new("test", "/test.lua");
        script.mark_loaded();
        assert!(script.is_loaded());
    }

    #[test]
    fn call_on_use_invokes_env_call_function_with_player_and_item() {
        let script = Script::new("test", "/test.lua");
        let mut env = MockLuaEnv::new();
        env.register_function("onUse", |args| args);

        let result = script.call_on_use(&mut env, 100, 200);
        assert!(result.is_ok());
        let vals = result.unwrap();
        assert_eq!(vals[0], LuaValue::Integer(100));
        assert_eq!(vals[1], LuaValue::Integer(200));
    }

    #[test]
    fn call_on_use_fails_when_function_not_registered() {
        let script = Script::new("test", "/test.lua");
        let mut env = MockLuaEnv::new();
        let result = script.call_on_use(&mut env, 100, 200);
        assert!(matches!(result, Err(LuaError::RuntimeError(_))));
    }

    #[test]
    fn call_on_step_in_invokes_env_with_creature_and_position() {
        let script = Script::new("test", "/test.lua");
        let mut env = MockLuaEnv::new();
        env.register_function("onStepIn", |args| args);

        let result = script.call_on_step_in(&mut env, 42, 10, 20, 7);
        assert!(result.is_ok());
        let vals = result.unwrap();
        assert_eq!(vals[0], LuaValue::Integer(42));
        assert_eq!(vals[1], LuaValue::Integer(10));
        assert_eq!(vals[2], LuaValue::Integer(20));
        assert_eq!(vals[3], LuaValue::Integer(7));
    }

    #[test]
    fn call_on_step_in_fails_when_function_not_registered() {
        let script = Script::new("test", "/test.lua");
        let mut env = MockLuaEnv::new();
        let result = script.call_on_step_in(&mut env, 1, 0, 0, 0);
        assert!(matches!(result, Err(LuaError::RuntimeError(_))));
    }
}
