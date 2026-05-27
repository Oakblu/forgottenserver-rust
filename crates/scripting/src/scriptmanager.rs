use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::script::Script;

#[cfg(feature = "lua-scripting")]
use crate::luascript::LuaEngine;

pub struct ScriptManager {
    scripts: HashMap<String, Script>,
    #[cfg(feature = "lua-scripting")]
    engine: Option<Arc<LuaEngine>>,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
            #[cfg(feature = "lua-scripting")]
            engine: None,
        }
    }

    #[cfg(feature = "lua-scripting")]
    pub fn with_lua(engine: Arc<LuaEngine>) -> Self {
        Self {
            scripts: HashMap::new(),
            engine: Some(engine),
        }
    }

    #[cfg(feature = "lua-scripting")]
    pub fn load_file(&mut self, path: &Path) -> mlua::Result<()> {
        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| mlua::Error::RuntimeError("no LuaEngine attached".into()))?;
        engine.load_file(path)
    }

    /// Replace the Lua engine with a fresh instance, clearing all global state.
    /// Equivalent to C++ `scriptInterface.reInitState()`.
    #[cfg(feature = "lua-scripting")]
    pub fn reload_state(&mut self) -> mlua::Result<()> {
        #[allow(clippy::arc_with_non_send_sync)]
        let engine = Arc::new(LuaEngine::new()?);
        self.engine = Some(engine);
        Ok(())
    }

    #[cfg(feature = "lua-scripting")]
    pub fn load_dir(&mut self, dir: &Path) -> mlua::Result<usize> {
        if !dir.exists() {
            return Err(mlua::Error::RuntimeError(format!(
                "script directory not found: {}",
                dir.display()
            )));
        }
        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| mlua::Error::RuntimeError("no LuaEngine attached".into()))?;
        let read_dir =
            std::fs::read_dir(dir).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        let mut loaded = 0usize;
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                engine.load_file(&path)?;
                loaded += 1;
            }
        }
        Ok(loaded)
    }

    pub fn register_script(&mut self, name: impl Into<String>, path: impl Into<String>) {
        let name = name.into();
        let script = Script::new(name.clone(), path);
        self.scripts.insert(name, script);
    }

    pub fn get_script(&self, name: &str) -> Option<&Script> {
        self.scripts.get(name)
    }

    pub fn get_script_mut(&mut self, name: &str) -> Option<&mut Script> {
        self.scripts.get_mut(name)
    }

    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }

    pub fn mark_all_loaded(&mut self) {
        for script in self.scripts.values_mut() {
            script.mark_loaded();
        }
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "lua-scripting")]
use mlua;

#[cfg(test)]
mod tests {
    // mlua's `Lua` is intentionally `!Send + !Sync` (single-threaded VM).
    // Wrapping it in `Arc` is legitimate for shared ownership within one thread
    // (matching the production API of `ScriptManager::with_lua(Arc<LuaEngine>)`),
    // so silence clippy across the test module rather than annotating each call site.
    #![allow(clippy::arc_with_non_send_sync)]

    use super::*;

    #[test]
    fn new_creates_empty_manager() {
        let manager = ScriptManager::new();
        assert_eq!(manager.script_count(), 0);
    }

    #[test]
    fn register_script_adds_script_by_name() {
        let mut manager = ScriptManager::new();
        manager.register_script("door_open", "/scripts/door_open.lua");
        assert_eq!(manager.script_count(), 1);
    }

    #[test]
    fn get_script_returns_some_after_registering() {
        let mut manager = ScriptManager::new();
        manager.register_script("my_script", "/scripts/my_script.lua");
        let script = manager.get_script("my_script");
        assert!(script.is_some());
        assert_eq!(script.unwrap().get_name(), "my_script");
    }

    #[test]
    fn get_script_unknown_returns_none() {
        let manager = ScriptManager::new();
        assert!(manager.get_script("unknown").is_none());
    }

    #[test]
    fn script_count_returns_count() {
        let mut manager = ScriptManager::new();
        manager.register_script("a", "/a.lua");
        manager.register_script("b", "/b.lua");
        manager.register_script("c", "/c.lua");
        assert_eq!(manager.script_count(), 3);
    }

    #[test]
    fn mark_all_loaded_marks_all_scripts_as_loaded() {
        let mut manager = ScriptManager::new();
        manager.register_script("x", "/x.lua");
        manager.register_script("y", "/y.lua");
        manager.mark_all_loaded();
        assert!(manager.get_script("x").unwrap().is_loaded());
        assert!(manager.get_script("y").unwrap().is_loaded());
    }

    #[test]
    fn scripts_not_loaded_before_mark_all_loaded() {
        let mut manager = ScriptManager::new();
        manager.register_script("x", "/x.lua");
        assert!(!manager.get_script("x").unwrap().is_loaded());
    }

    #[test]
    fn get_script_mut_allows_mutation_observable_via_get_script() {
        let mut manager = ScriptManager::new();
        manager.register_script("door", "/door.lua");
        // Mutate via get_script_mut: flip the loaded flag on the underlying Script.
        {
            let script_mut = manager
                .get_script_mut("door")
                .expect("registered script should be reachable via get_script_mut");
            assert!(
                !script_mut.is_loaded(),
                "freshly registered script must be unloaded"
            );
            script_mut.mark_loaded();
        }
        // Observe the mutation through the read-only accessor.
        assert!(
            manager.get_script("door").unwrap().is_loaded(),
            "mutation through get_script_mut must be visible via get_script"
        );
    }

    #[test]
    fn get_script_mut_returns_none_for_unknown_name() {
        let mut manager = ScriptManager::new();
        manager.register_script("known", "/known.lua");
        assert!(
            manager.get_script_mut("not_registered").is_none(),
            "get_script_mut must return None for unknown names"
        );
    }

    #[test]
    fn default_constructs_empty_usable_manager() {
        // Exercise the Default impl and prove it produces a working empty manager
        // equivalent to ScriptManager::new().
        let mut manager: ScriptManager = Default::default();
        assert_eq!(
            manager.script_count(),
            0,
            "Default must yield an empty registry"
        );
        // Manager must remain functional: register, look up, mutate.
        manager.register_script("alpha", "/alpha.lua");
        manager.register_script("beta", "/beta.lua");
        assert_eq!(manager.script_count(), 2);
        manager.mark_all_loaded();
        assert!(manager.get_script("alpha").unwrap().is_loaded());
        assert!(manager.get_script("beta").unwrap().is_loaded());
    }

    // -----------------------------------------------------------------------
    // LuaEngine integration tests (Phase 0.5.4)
    // -----------------------------------------------------------------------

    #[cfg(feature = "lua-scripting")]
    #[test]
    fn script_manager_load_nonexistent_file_returns_err() {
        use crate::luascript::LuaEngine;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        let result = mgr.load_file(std::path::Path::new("/nonexistent/does_not_exist.lua"));
        assert!(result.is_err());
    }

    #[cfg(feature = "lua-scripting")]
    #[test]
    fn script_manager_load_valid_lua_string_ok() {
        use crate::luascript::LuaEngine;
        use std::io::Write;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "x = 1").unwrap();
        let result = mgr.load_file(f.path());
        assert!(result.is_ok());
    }

    #[cfg(feature = "lua-scripting")]
    #[test]
    fn script_manager_load_dir_loads_all_lua_files() {
        use crate::luascript::LuaEngine;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.lua"), "a = 1").unwrap();
        std::fs::write(dir.path().join("b.lua"), "b = 2").unwrap();
        std::fs::write(dir.path().join("c.lua"), "c = 3").unwrap();
        std::fs::write(dir.path().join("ignored.txt"), "not lua").unwrap();
        let count = mgr.load_dir(dir.path()).unwrap();
        assert_eq!(count, 3);
    }

    // -----------------------------------------------------------------------
    // Phase 13.7 — C++ parity tests
    // -----------------------------------------------------------------------

    /// Error message from a file with a syntax error must include the filename.
    #[cfg(feature = "lua-scripting")]
    #[test]
    fn load_file_error_includes_filename() {
        use crate::luascript::LuaEngine;
        use std::io::Write;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        let mut f = tempfile::NamedTempFile::new().unwrap();
        // Syntax error on line 1
        writeln!(f, "this is @invalid@ lua syntax !!!").unwrap();
        let path = f.path().to_path_buf();
        let result = mgr.load_file(&path);
        assert!(result.is_err(), "expected Err for invalid Lua");
        let err_msg = result.unwrap_err().to_string();
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(
            err_msg.contains(filename),
            "error message should contain filename '{}', got: {}",
            filename,
            err_msg
        );
    }

    /// Error message from a file with a syntax error must include the line number.
    #[cfg(feature = "lua-scripting")]
    #[test]
    fn load_file_error_includes_line_number() {
        use crate::luascript::LuaEngine;
        use std::io::Write;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        let mut f = tempfile::NamedTempFile::new().unwrap();
        // Put two valid lines, then a syntax error on line 3
        writeln!(f, "x = 1").unwrap();
        writeln!(f, "y = 2").unwrap();
        writeln!(f, "this is @bad@ syntax !!!").unwrap();
        let result = mgr.load_file(f.path());
        assert!(result.is_err(), "expected Err for invalid Lua");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains('3'),
            "error message should contain line number '3', got: {}",
            err_msg
        );
    }

    /// load_dir must skip non-.lua files (only .lua files are loaded).
    #[cfg(feature = "lua-scripting")]
    #[test]
    fn load_dir_skips_non_lua_files() {
        use crate::luascript::LuaEngine;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("script.lua"), "ok = 1").unwrap();
        // These should all be skipped
        std::fs::write(dir.path().join("readme.txt"), "text").unwrap();
        std::fs::write(dir.path().join("config.xml"), "<root/>").unwrap();
        std::fs::write(dir.path().join("data.json"), "{}").unwrap();
        std::fs::write(dir.path().join("noext"), "no ext").unwrap();
        let count = mgr.load_dir(dir.path()).unwrap();
        assert_eq!(count, 1, "only the .lua file should have been loaded");
    }

    /// After reload_state(), a global set in the old state must not be visible.
    #[cfg(feature = "lua-scripting")]
    #[test]
    fn reload_clears_old_state() {
        use crate::luascript::LuaEngine;
        use std::io::Write;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        // Load a script that sets a global
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "old_global = 42").unwrap();
        mgr.load_file(f.path()).unwrap();
        // Confirm the global exists (non-nil integer value)
        {
            let eng = mgr.engine.as_ref().unwrap();
            let v: i64 = eng.get_global("old_global").unwrap();
            assert_eq!(v, 42, "old_global should be 42 before reload");
        }
        // Reload clears state — engine is replaced with a fresh one
        mgr.reload_state().unwrap();
        // After reload, old_global must be gone (nil) — we check by querying as Option<i64>
        {
            let eng = mgr.engine.as_ref().unwrap();
            let result: mlua::Result<Option<i64>> = eng.get_global("old_global");
            let is_nil = matches!(result, Ok(None));
            assert!(is_nil, "old_global should be nil after reload");
        }
    }

    /// load_file with invalid Lua must return Err, never panic.
    #[cfg(feature = "lua-scripting")]
    #[test]
    fn load_invalid_lua_returns_error() {
        use crate::luascript::LuaEngine;
        use std::io::Write;
        use std::sync::Arc;
        let engine = Arc::new(LuaEngine::new().unwrap());
        let mut mgr = ScriptManager::with_lua(engine);
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "function (((broken lua here").unwrap();
        let result = mgr.load_file(f.path());
        assert!(
            result.is_err(),
            "should return Err for broken Lua, not panic"
        );
        // Manager must still be usable after the error
        let mut f2 = tempfile::NamedTempFile::new().unwrap();
        writeln!(f2, "valid_after_error = 1").unwrap();
        assert!(
            mgr.load_file(f2.path()).is_ok(),
            "manager should remain usable after an error"
        );
    }
}
