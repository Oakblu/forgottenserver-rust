use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::ScriptError;
use crate::value::ScriptValue;

/// The seven Lua event-handler categories that the ScriptingManager initialises.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum EventType {
    MoveEvent,
    Action,
    TalkAction,
    GlobalEvent,
    CreatureEvent,
    Spell,
    Weapon,
}

/// Metadata for a single `.xml` mod file.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ModBlock {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub contact: String,
    pub file: String,
    pub enabled: bool,
}

/// A named Lua library snippet loaded from a mod.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct LibBlock {
    pub first: String,
    pub second: String,
}

/// Trait for script engines.
pub trait ScriptEngine {
    fn load_script(&mut self, path: &Path) -> Result<(), ScriptError>;
    fn call_function(
        &mut self,
        name: &str,
        args: &[ScriptValue],
    ) -> Result<ScriptValue, ScriptError>;
    fn reset(&mut self) -> Result<(), ScriptError>;
    fn register_event(&mut self, event: &str, script_path: &Path) -> Result<(), ScriptError>;
    fn call_event(&mut self, event: &str, args: &[ScriptValue])
        -> Result<ScriptValue, ScriptError>;
}

/// A no-op script engine that always returns Ok.
pub struct NoopScriptEngine;

impl NoopScriptEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEngine for NoopScriptEngine {
    fn load_script(&mut self, _path: &Path) -> Result<(), ScriptError> {
        Ok(())
    }

    fn call_function(
        &mut self,
        _name: &str,
        _args: &[ScriptValue],
    ) -> Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }

    fn reset(&mut self) -> Result<(), ScriptError> {
        Ok(())
    }

    fn register_event(&mut self, _event: &str, _script_path: &Path) -> Result<(), ScriptError> {
        Ok(())
    }

    fn call_event(
        &mut self,
        _event: &str,
        _args: &[ScriptValue],
    ) -> Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }
}

/// Recursively collect `.lua` files under `dir`, skipping:
/// - Subdirectories named `lib` when `skip_lib=true` (loaded separately with isLib=true)
/// - Files whose name starts with `#` (disabled marker)
#[cfg(feature = "lua-scripting")]
pub(crate) fn collect_lua_files(
    dir: &std::path::Path,
    out: &mut Vec<PathBuf>,
    skip_lib: bool,
) -> Result<(), crate::error::ScriptError> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| {
        crate::error::ScriptError::LoadFailed(format!("cannot read directory: {e}"))
    })?;
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if skip_lib && path.file_name().and_then(|n| n.to_str()) == Some("lib") {
                continue;
            }
            collect_lua_files(&path, out, skip_lib)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("lua") {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('#') {
                out.push(path);
            }
        }
    }
    Ok(())
}

/// A real mlua-backed Lua script engine.
#[cfg(feature = "lua-scripting")]
pub struct LuaScriptEngine {
    pub(crate) lua: mlua::Lua,
    events: HashMap<String, PathBuf>,
}

#[cfg(feature = "lua-scripting")]
impl LuaScriptEngine {
    pub fn new() -> Self {
        Self {
            lua: mlua::Lua::new(),
            events: HashMap::new(),
        }
    }

    fn script_value_to_lua(&self, value: &ScriptValue) -> Result<mlua::Value<'_>, ScriptError> {
        match value {
            ScriptValue::Nil => Ok(mlua::Value::Nil),
            ScriptValue::Bool(b) => Ok(mlua::Value::Boolean(*b)),
            ScriptValue::Integer(i) => Ok(mlua::Value::Integer(*i)),
            ScriptValue::Float(f) => Ok(mlua::Value::Number(*f)),
            ScriptValue::Str(s) => {
                let lua_str = self
                    .lua
                    .create_string(s.as_bytes())
                    .map_err(|e| ScriptError::RuntimeError(e.to_string()))?;
                Ok(mlua::Value::String(lua_str))
            }
        }
    }

    fn lua_value_to_script(&self, value: mlua::Value<'_>) -> Result<ScriptValue, ScriptError> {
        match value {
            mlua::Value::Nil => Ok(ScriptValue::Nil),
            mlua::Value::Boolean(b) => Ok(ScriptValue::Bool(b)),
            mlua::Value::Integer(i) => Ok(ScriptValue::Integer(i)),
            mlua::Value::Number(f) => Ok(ScriptValue::Float(f)),
            mlua::Value::String(s) => {
                let owned = s
                    .to_str()
                    .map_err(|e| ScriptError::RuntimeError(e.to_string()))?
                    .to_owned();
                Ok(ScriptValue::Str(owned))
            }
            _ => Ok(ScriptValue::Nil),
        }
    }

    /// Load `.lua` files from `dir` recursively, matching C++ `Scripts::loadScripts(isLib=false)`:
    /// - Skips subdirectories named `lib` (those are loaded separately first)
    /// - Skips files whose name starts with `#` (disabled marker)
    /// - Loads files in sorted order for determinism
    /// - Logs per-file errors and continues (does not abort on a bad script)
    ///
    /// Returns the count of successfully loaded scripts.
    pub fn load_dir(&mut self, dir: &Path) -> Result<usize, ScriptError> {
        if !dir.exists() {
            return Err(ScriptError::LoadFailed(format!(
                "script directory not found: {}",
                dir.display()
            )));
        }
        let mut paths = Vec::new();
        collect_lua_files(dir, &mut paths, true)?;
        paths.sort();

        let mut loaded = 0usize;
        for path in &paths {
            match self.load_script(path) {
                Ok(()) => loaded += 1,
                Err(e) => eprintln!(
                    "> {} [error]: {e}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
            }
        }
        Ok(loaded)
    }
}

#[cfg(feature = "lua-scripting")]
impl Default for LuaScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "lua-scripting")]
impl ScriptEngine for LuaScriptEngine {
    fn load_script(&mut self, path: &Path) -> Result<(), ScriptError> {
        let source =
            std::fs::read_to_string(path).map_err(|e| ScriptError::LoadFailed(e.to_string()))?;
        self.lua
            .load(&source)
            .exec()
            .map_err(|e| ScriptError::LoadFailed(e.to_string()))
    }

    fn call_function(
        &mut self,
        name: &str,
        args: &[ScriptValue],
    ) -> Result<ScriptValue, ScriptError> {
        let func: mlua::Function = self
            .lua
            .globals()
            .get(name)
            .map_err(|e| ScriptError::CallFailed(e.to_string()))?;

        let lua_args: mlua::MultiValue = args
            .iter()
            .map(|v| self.script_value_to_lua(v))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .collect();

        let result: mlua::Value = func
            .call(lua_args)
            .map_err(|e| ScriptError::RuntimeError(e.to_string()))?;

        self.lua_value_to_script(result)
    }

    fn reset(&mut self) -> Result<(), ScriptError> {
        self.lua = mlua::Lua::new();
        self.events.clear();
        Ok(())
    }

    fn register_event(&mut self, event: &str, script_path: &Path) -> Result<(), ScriptError> {
        self.events
            .insert(event.to_string(), script_path.to_path_buf());
        Ok(())
    }

    fn call_event(
        &mut self,
        event: &str,
        args: &[ScriptValue],
    ) -> Result<ScriptValue, ScriptError> {
        let script_path = self.events.get(event).cloned().ok_or_else(|| {
            ScriptError::CallFailed(format!("no script registered for event: {event}"))
        })?;

        self.load_script(&script_path)?;
        self.call_function(event, args)
    }
}

/// Registry that maps script names to file paths and drives a `ScriptEngine`.
#[allow(dead_code)]
pub struct ScriptManager {
    engine: Box<dyn ScriptEngine>,
    scripts: HashMap<String, PathBuf>,
    script_order: Vec<String>,
    scripts_dir: Option<PathBuf>,
    mods_loaded: bool,
    mod_map: HashMap<String, ModBlock>,
    lib_map: HashMap<String, LibBlock>,
    load_errors: Vec<(String, String)>,
}

#[allow(dead_code)]
impl ScriptManager {
    pub fn new(engine: Box<dyn ScriptEngine>) -> Self {
        Self {
            engine,
            scripts: HashMap::new(),
            script_order: Vec::new(),
            scripts_dir: None,
            mods_loaded: false,
            mod_map: HashMap::new(),
            lib_map: HashMap::new(),
            load_errors: Vec::new(),
        }
    }

    pub fn load(&mut self, name: impl Into<String>, path: &Path) -> Result<(), ScriptError> {
        self.engine.load_script(path)?;
        let name: String = name.into();
        self.script_order.push(name.clone());
        self.scripts.insert(name, path.to_path_buf());
        Ok(())
    }

    pub fn reload(&mut self, name: &str) -> Result<(), ScriptError> {
        let path = self
            .scripts
            .get(name)
            .cloned()
            .ok_or_else(|| ScriptError::LoadFailed(format!("no script registered: {name}")))?;
        self.engine.load_script(&path)
    }

    pub fn get_script_interface(&self) -> &dyn ScriptEngine {
        self.engine.as_ref()
    }

    pub fn get_script_interface_for_event(&self, _event: EventType) -> &dyn ScriptEngine {
        self.engine.as_ref()
    }

    pub fn clear(&mut self) -> Result<(), ScriptError> {
        self.scripts.clear();
        self.script_order.clear();
        self.load_errors.clear();
        self.engine.reset()
    }

    pub fn load_scripts(&mut self, dir: &Path) -> Result<usize, ScriptError> {
        if !dir.exists() {
            self.scripts_dir = Some(dir.to_path_buf());
            return Ok(0);
        }

        let read_dir = std::fs::read_dir(dir)
            .map_err(|e| ScriptError::LoadFailed(format!("cannot read directory: {e}")))?;

        self.scripts_dir = Some(dir.to_path_buf());
        let mut loaded = 0usize;

        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("lua") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            match self.engine.load_script(&path) {
                Ok(()) => {
                    if !self.scripts.contains_key(&name) {
                        self.script_order.push(name.clone());
                    }
                    self.scripts.insert(name, path);
                    loaded += 1;
                }
                Err(e) => {
                    self.load_errors
                        .push((path.to_string_lossy().into_owned(), e.to_string()));
                }
            }
        }

        Ok(loaded)
    }

    pub fn reload_scripts(&mut self) -> Result<usize, ScriptError> {
        let dir = self
            .scripts_dir
            .clone()
            .ok_or_else(|| ScriptError::LoadFailed("no scripts directory set".to_string()))?;
        self.clear()?;
        self.scripts_dir = Some(dir.clone());
        self.load_scripts(&dir)
    }

    pub fn get_script_id(&self, name: &str) -> Option<usize> {
        self.script_order.iter().position(|n| n == name)
    }

    pub fn get_script_base_name(&self, id: usize) -> Option<&str> {
        self.script_order.get(id).map(|s| s.as_str())
    }

    pub fn script_names(&self) -> impl Iterator<Item = &str> {
        self.scripts.keys().map(|k| k.as_str())
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.scripts.contains_key(name)
    }

    pub fn mods_loaded(&self) -> bool {
        self.mods_loaded
    }

    pub fn register_mod(&mut self, block: ModBlock) {
        self.mods_loaded = true;
        self.mod_map.insert(block.name.clone(), block);
    }

    pub fn register_lib(&mut self, name: impl Into<String>, block: LibBlock) -> bool {
        let name = name.into();
        if self.lib_map.contains_key(&name) {
            return false;
        }
        self.lib_map.insert(name, block);
        true
    }

    pub fn get_mod(&self, name: &str) -> Option<&ModBlock> {
        self.mod_map.get(name)
    }

    pub fn get_lib(&self, name: &str) -> Option<&LibBlock> {
        self.lib_map.get(name)
    }

    pub fn mods(&self) -> impl Iterator<Item = (&str, &ModBlock)> {
        self.mod_map.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn libs(&self) -> impl Iterator<Item = (&str, &LibBlock)> {
        self.lib_map.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn clear_mods(&mut self) {
        self.mod_map.clear();
        self.lib_map.clear();
        self.mods_loaded = false;
    }

    pub fn load_errors(&self) -> &[(String, String)] {
        &self.load_errors
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_engine_load_script_returns_ok() {
        let mut engine = NoopScriptEngine::new();
        assert!(engine.load_script(Path::new("some/script.lua")).is_ok());
    }

    #[test]
    fn noop_engine_call_function_returns_nil() {
        let mut engine = NoopScriptEngine::new();
        let result = engine.call_function("onDeath", &[]).unwrap();
        assert_eq!(result, ScriptValue::Nil);
    }

    #[test]
    fn noop_engine_call_with_args_returns_nil() {
        let mut engine = NoopScriptEngine::new();
        let args = vec![
            ScriptValue::Integer(42),
            ScriptValue::Str("player".to_string()),
        ];
        let result = engine.call_function("onSpawn", &args).unwrap();
        assert_eq!(result, ScriptValue::Nil);
    }

    #[test]
    fn noop_engine_reset_returns_ok() {
        let mut engine = NoopScriptEngine::new();
        assert!(engine.reset().is_ok());
    }

    #[test]
    fn noop_engine_as_trait_object() {
        let mut engine: Box<dyn ScriptEngine> = Box::new(NoopScriptEngine::new());
        assert!(engine.load_script(Path::new("x.lua")).is_ok());
        assert_eq!(engine.call_function("f", &[]).unwrap(), ScriptValue::Nil);
        assert!(engine.reset().is_ok());
    }

    #[test]
    fn noop_engine_default() {
        let mut engine = NoopScriptEngine;
        assert!(engine.reset().is_ok());
    }

    #[test]
    fn noop_engine_register_event_succeeds() {
        let mut engine = NoopScriptEngine::new();
        assert!(engine.register_event("onDeath", Path::new("x.lua")).is_ok());
    }

    #[test]
    fn call_unregistered_event_returns_nil() {
        let mut engine = NoopScriptEngine::new();
        let result = engine.call_event("onDeath", &[]).unwrap();
        assert_eq!(result, ScriptValue::Nil);
    }

    #[test]
    fn noop_engine_call_event_with_args_returns_nil() {
        let mut engine = NoopScriptEngine::new();
        let args = vec![ScriptValue::Integer(1), ScriptValue::Str("arg".to_string())];
        let result = engine.call_event("onSpawn", &args).unwrap();
        assert_eq!(result, ScriptValue::Nil);
    }

    #[test]
    fn noop_engine_trait_object_register_and_call_event() {
        let mut engine: Box<dyn ScriptEngine> = Box::new(NoopScriptEngine::new());
        assert!(engine
            .register_event("onLogin", Path::new("login.lua"))
            .is_ok());
        assert_eq!(engine.call_event("onLogin", &[]).unwrap(), ScriptValue::Nil);
    }

    #[test]
    fn script_manager_load_registers_script_by_name() {
        let engine = Box::new(NoopScriptEngine::new());
        let mut mgr = ScriptManager::new(engine);
        assert!(mgr
            .load("onDeath", Path::new("scripts/on_death.lua"))
            .is_ok());
        assert!(mgr.is_registered("onDeath"));
    }

    #[test]
    fn script_manager_reload_registered_script_succeeds() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        mgr.load("onLogin", Path::new("scripts/login.lua")).unwrap();
        assert!(mgr.reload("onLogin").is_ok());
    }

    #[test]
    fn script_manager_reload_unregistered_returns_error() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        let err = mgr.reload("nonexistent").unwrap_err();
        assert!(matches!(err, ScriptError::LoadFailed(_)));
        assert!(err.to_string().contains("no script registered"));
    }

    #[test]
    fn script_manager_clear_removes_all_registrations() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        mgr.load("onDeath", Path::new("a.lua")).unwrap();
        mgr.load("onSpawn", Path::new("b.lua")).unwrap();
        mgr.clear().unwrap();
        assert!(!mgr.is_registered("onDeath"));
        assert!(!mgr.is_registered("onSpawn"));
    }

    #[test]
    fn script_manager_is_registered_false_before_load() {
        let mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        assert!(!mgr.is_registered("onDeath"));
    }

    #[test]
    fn script_manager_get_script_id_returns_insertion_index() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        mgr.load("alpha", Path::new("a.lua")).unwrap();
        mgr.load("beta", Path::new("b.lua")).unwrap();
        assert_eq!(mgr.get_script_id("alpha"), Some(0));
        assert_eq!(mgr.get_script_id("beta"), Some(1));
    }

    #[test]
    fn script_manager_get_script_base_name_reverse_lookup() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        mgr.load("alpha", Path::new("a.lua")).unwrap();
        assert_eq!(mgr.get_script_base_name(0), Some("alpha"));
        assert_eq!(mgr.get_script_base_name(99), None);
    }

    #[test]
    fn load_scripts_nonexistent_directory_returns_ok_zero() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        let result = mgr.load_scripts(Path::new("/this/does/not/exist_xyz_12345"));
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn load_scripts_lua_files_are_loaded() {
        use std::io::Write;
        let dir = tempfile::TempDir::new().unwrap();
        let p1 = dir.path().join("on_death.lua");
        let p2 = dir.path().join("on_spawn.lua");
        std::fs::File::create(&p1).unwrap().write_all(b"").unwrap();
        std::fs::File::create(&p2).unwrap().write_all(b"").unwrap();

        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        let count = mgr.load_scripts(dir.path()).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn load_scripts_non_lua_files_are_ignored() {
        use std::io::Write;
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::File::create(dir.path().join("readme.txt"))
            .unwrap()
            .write_all(b"")
            .unwrap();
        std::fs::File::create(dir.path().join("script.lua"))
            .unwrap()
            .write_all(b"")
            .unwrap();

        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        let count = mgr.load_scripts(dir.path()).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn reload_scripts_without_prior_load_scripts_returns_error() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        let err = mgr.reload_scripts().unwrap_err();
        assert!(matches!(err, ScriptError::LoadFailed(_)));
        assert!(err.to_string().contains("no scripts directory"));
    }

    #[test]
    fn event_type_variants_are_distinct() {
        let all = [
            EventType::MoveEvent,
            EventType::Action,
            EventType::TalkAction,
            EventType::GlobalEvent,
            EventType::CreatureEvent,
            EventType::Spell,
            EventType::Weapon,
        ];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j]);
            }
        }
    }

    #[test]
    fn event_type_can_be_used_as_hash_key() {
        let mut map: HashMap<EventType, &str> = HashMap::new();
        map.insert(EventType::MoveEvent, "move");
        assert_eq!(map[&EventType::MoveEvent], "move");
    }

    #[test]
    fn register_mod_sets_mods_loaded() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        mgr.register_mod(ModBlock {
            name: "ModA".to_string(),
            enabled: true,
            ..Default::default()
        });
        assert!(mgr.mods_loaded());
    }

    #[test]
    fn register_lib_duplicate_returns_false() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        mgr.register_lib(
            "mylib",
            LibBlock {
                first: "first.xml".to_string(),
                second: "".to_string(),
            },
        );
        let ok = mgr.register_lib(
            "mylib",
            LibBlock {
                first: "second.xml".to_string(),
                second: "".to_string(),
            },
        );
        assert!(!ok);
        assert_eq!(mgr.get_lib("mylib").unwrap().first, "first.xml");
    }

    #[test]
    fn clear_mods_removes_all_mods_and_libs() {
        let mut mgr = ScriptManager::new(Box::new(NoopScriptEngine::new()));
        mgr.register_mod(ModBlock {
            name: "Mod1".to_string(),
            ..Default::default()
        });
        mgr.register_lib(
            "lib1",
            LibBlock {
                first: "f".to_string(),
                second: "s".to_string(),
            },
        );
        mgr.clear_mods();
        assert!(mgr.get_mod("Mod1").is_none());
        assert!(mgr.get_lib("lib1").is_none());
        assert!(!mgr.mods_loaded());
    }

    // ---------------------------------------------------------------------------
    // LuaScriptEngine tests (feature-gated)
    // ---------------------------------------------------------------------------

    #[cfg(feature = "lua-scripting")]
    mod lua_tests {
        use super::*;
        use std::io::Write;
        use tempfile::NamedTempFile;

        #[test]
        fn lua_engine_new_and_default() {
            let _ = LuaScriptEngine::new();
            let _ = LuaScriptEngine::default();
        }

        #[test]
        fn load_script_executes_lua() {
            let mut engine = LuaScriptEngine::new();
            let mut f = NamedTempFile::new().unwrap();
            writeln!(f, "x = 42").unwrap();
            assert!(engine.load_script(f.path()).is_ok());
        }

        #[test]
        fn lua_engine_load_script_missing_file_returns_err() {
            let mut engine = LuaScriptEngine::new();
            let result = engine.load_script(Path::new("/nonexistent/path/script.lua"));
            assert!(result.is_err());
        }

        #[test]
        fn call_function_returns_integer() {
            let mut engine = LuaScriptEngine::new();
            engine
                .lua
                .load("function add(a, b) return a + b end")
                .exec()
                .unwrap();
            let result = engine
                .call_function("add", &[ScriptValue::Integer(3), ScriptValue::Integer(4)])
                .unwrap();
            assert_eq!(result, ScriptValue::Integer(7));
        }

        #[test]
        fn call_function_returns_string() {
            let mut engine = LuaScriptEngine::new();
            engine
                .lua
                .load(r#"function greet(name) return "hello " .. name end"#)
                .exec()
                .unwrap();
            let result = engine
                .call_function("greet", &[ScriptValue::Str("world".to_string())])
                .unwrap();
            assert_eq!(result, ScriptValue::Str("hello world".to_string()));
        }

        #[test]
        fn call_function_returns_bool() {
            let mut engine = LuaScriptEngine::new();
            engine
                .lua
                .load("function is_true() return true end")
                .exec()
                .unwrap();
            let result = engine.call_function("is_true", &[]).unwrap();
            assert_eq!(result, ScriptValue::Bool(true));
        }

        #[test]
        fn call_function_returns_nil() {
            let mut engine = LuaScriptEngine::new();
            engine
                .lua
                .load("function nothing() return nil end")
                .exec()
                .unwrap();
            let result = engine.call_function("nothing", &[]).unwrap();
            assert_eq!(result, ScriptValue::Nil);
        }

        #[test]
        fn call_function_missing_name_returns_err() {
            let mut engine = LuaScriptEngine::new();
            let result = engine.call_function("does_not_exist", &[]);
            assert!(result.is_err());
        }

        #[test]
        fn reset_clears_all_globals() {
            let mut engine = LuaScriptEngine::new();
            engine.lua.load("myvar = 999").exec().unwrap();
            engine.reset().unwrap();
            let val: mlua::Value = engine.lua.globals().get("myvar").unwrap();
            assert_eq!(val, mlua::Value::Nil);
        }

        #[test]
        fn reset_clears_registered_events() {
            let mut engine = LuaScriptEngine::new();
            engine.register_event("onTest", Path::new("x.lua")).unwrap();
            engine.reset().unwrap();
            let result = engine.call_event("onTest", &[]);
            assert!(result.is_err());
        }

        #[test]
        fn call_event_with_matching_function_returns_value() {
            let mut engine = LuaScriptEngine::new();
            let mut f = NamedTempFile::new().unwrap();
            writeln!(f, "function onKill() return 42 end").unwrap();
            engine.register_event("onKill", f.path()).unwrap();
            let result = engine.call_event("onKill", &[]).unwrap();
            assert_eq!(result, ScriptValue::Integer(42));
        }

        #[test]
        fn call_event_unregistered_returns_err() {
            let mut engine = LuaScriptEngine::new();
            let result = engine.call_event("onMissing", &[]);
            assert!(result.is_err());
        }

        // Phase 6 tests — load_dir
        #[test]
        fn load_dir_loads_all_lua_files_in_directory() {
            let dir = tempfile::TempDir::new().unwrap();
            std::fs::write(dir.path().join("a.lua"), "a = 1").unwrap();
            std::fs::write(dir.path().join("b.lua"), "b = 2").unwrap();
            std::fs::write(dir.path().join("not_lua.txt"), "ignored").unwrap();

            let mut engine = LuaScriptEngine::new();
            let count = engine.load_dir(dir.path()).unwrap();
            assert_eq!(count, 2);

            let a: mlua::Value = engine.lua.globals().get("a").unwrap();
            assert_eq!(a, mlua::Value::Integer(1));
            let b: mlua::Value = engine.lua.globals().get("b").unwrap();
            assert_eq!(b, mlua::Value::Integer(2));
        }

        #[test]
        fn missing_script_dir_returns_error() {
            let mut engine = LuaScriptEngine::new();
            let result = engine.load_dir(Path::new("/nonexistent/scripts/dir_xyz"));
            assert!(result.is_err());
            if let Err(ScriptError::LoadFailed(msg)) = result {
                assert!(msg.contains("not found") || msg.contains("nonexistent"));
            } else {
                panic!("expected LoadFailed");
            }
        }

        #[test]
        fn load_dir_recurses_into_subdirectories() {
            let dir = tempfile::TempDir::new().unwrap();
            let subdir = dir.path().join("spells");
            std::fs::create_dir(&subdir).unwrap();
            std::fs::write(dir.path().join("top.lua"), "top_loaded = true").unwrap();
            std::fs::write(subdir.join("fireball.lua"), "fireball_loaded = true").unwrap();

            let mut engine = LuaScriptEngine::new();
            let count = engine.load_dir(dir.path()).unwrap();
            assert_eq!(count, 2, "load_dir must recurse into subdirectories");

            let top: mlua::Value = engine.lua.globals().get("top_loaded").unwrap();
            assert_eq!(top, mlua::Value::Boolean(true));
            let nested: mlua::Value = engine.lua.globals().get("fireball_loaded").unwrap();
            assert_eq!(
                nested,
                mlua::Value::Boolean(true),
                "script in subdir must be loaded"
            );
        }

        #[test]
        fn load_dir_skips_hash_prefixed_files() {
            let dir = tempfile::TempDir::new().unwrap();
            std::fs::write(dir.path().join("active.lua"), "active_loaded = true").unwrap();
            std::fs::write(dir.path().join("#disabled.lua"), "disabled_loaded = true").unwrap();

            let mut engine = LuaScriptEngine::new();
            let count = engine.load_dir(dir.path()).unwrap();
            assert_eq!(count, 1, "load_dir must skip #-prefixed files");

            let active: mlua::Value = engine.lua.globals().get("active_loaded").unwrap();
            assert_eq!(active, mlua::Value::Boolean(true));
            let disabled: mlua::Value = engine.lua.globals().get("disabled_loaded").unwrap();
            assert_eq!(
                disabled,
                mlua::Value::Nil,
                "#disabled.lua must not be loaded"
            );
        }

        #[test]
        fn load_dir_skips_lib_subdirectory() {
            let dir = tempfile::TempDir::new().unwrap();
            let lib_dir = dir.path().join("lib");
            std::fs::create_dir(&lib_dir).unwrap();
            std::fs::write(dir.path().join("main.lua"), "main_loaded = true").unwrap();
            std::fs::write(lib_dir.join("helpers.lua"), "helpers_loaded = true").unwrap();

            let mut engine = LuaScriptEngine::new();
            let count = engine.load_dir(dir.path()).unwrap();
            assert_eq!(count, 1, "load_dir must skip lib/ subdirectory");

            let main: mlua::Value = engine.lua.globals().get("main_loaded").unwrap();
            assert_eq!(main, mlua::Value::Boolean(true));
            let helpers: mlua::Value = engine.lua.globals().get("helpers_loaded").unwrap();
            assert_eq!(
                helpers,
                mlua::Value::Nil,
                "lib/helpers.lua must not be loaded"
            );
        }

        #[test]
        fn collect_lua_files_with_skip_lib_false_includes_lib_dir() {
            let dir = tempfile::TempDir::new().unwrap();
            let lib_dir = dir.path().join("lib");
            std::fs::create_dir(&lib_dir).unwrap();
            std::fs::write(lib_dir.join("helpers.lua"), "helpers = true").unwrap();
            std::fs::write(dir.path().join("main.lua"), "main = true").unwrap();

            let mut paths = Vec::new();
            crate::engine::collect_lua_files(dir.path(), &mut paths, false).unwrap();
            assert_eq!(paths.len(), 2, "skip_lib=false must include lib/ files");
        }
    }
}
