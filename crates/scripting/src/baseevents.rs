use std::collections::HashMap;

// ---------------------------------------------------------------------------
// BaseEvent — lightweight script reference (script filename only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BaseEvent {
    pub script_name: String,
}

impl BaseEvent {
    pub fn new(script_name: impl Into<String>) -> Self {
        Self {
            script_name: script_name.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// ScriptedEvent — mirrors C++ `Event`: tracks scripted/from_lua flags and
// the resolved Lua function id, plus the event name used for dispatch.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ScriptedEvent {
    /// Path to the Lua script file (relative).
    pub script_file: String,
    /// Lua function name used for dispatch (mirrors `getScriptEventName()`).
    pub event_name: String,
    /// True once the Lua script has been successfully loaded (mirrors `scripted`).
    pub scripted: bool,
    /// True when the event was registered from Lua rather than XML.
    pub from_lua: bool,
    /// Resolved Lua function id (mirrors `scriptId`).
    script_id: i32,
}

impl ScriptedEvent {
    pub fn new(script_file: impl Into<String>, event_name: impl Into<String>) -> Self {
        Self {
            script_file: script_file.into(),
            event_name: event_name.into(),
            scripted: false,
            from_lua: false,
            script_id: 0,
        }
    }

    /// Returns true if the underlying Lua script has been loaded successfully.
    pub fn is_scripted(&self) -> bool {
        self.scripted
    }

    /// Returns the resolved Lua function id (0 = not yet loaded).
    pub fn get_script_id(&self) -> i32 {
        self.script_id
    }

    /// Mark this event as scripted with the given resolved id.
    /// Mirrors the `scripted = true; scriptId = id;` assignment in `loadScript`.
    pub fn mark_scripted(&mut self, id: i32) {
        self.scripted = true;
        self.script_id = id;
    }
}

// ---------------------------------------------------------------------------
// NamedEventRegistry — maps event name → ScriptedEvent; models the per-type
// registry that subclasses of BaseEvents maintain in C++ (e.g. CreatureEvents).
// Provides named dispatch: execute(name) → Ok(name) | Err(EventError).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct NamedEventRegistry {
    events: HashMap<String, ScriptedEvent>,
}

/// Errors returned by event dispatch operations.
#[derive(Debug, PartialEq, Eq)]
pub enum EventError {
    /// No event with the given name has been registered.
    NotFound(String),
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::NotFound(name) => write!(f, "Event '{}' not found", name),
        }
    }
}

impl NamedEventRegistry {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    /// Register a scripted event under the given name.
    pub fn register(&mut self, name: String, event: ScriptedEvent) {
        self.events.insert(name, event);
    }

    /// Retrieve a registered event by name (read-only).
    pub fn get(&self, name: &str) -> Option<&ScriptedEvent> {
        self.events.get(name)
    }

    /// Number of registered events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true when no events are registered.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Dispatch: resolves the event by name and returns its event_name on success.
    /// Mirrors `executeEvent` in C++ — returns false (here `Err`) when the Lua
    /// function cannot be found.
    pub fn execute(&self, name: &str) -> Result<String, EventError> {
        match self.events.get(name) {
            Some(event) => Ok(event.event_name.clone()),
            None => Err(EventError::NotFound(name.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// XmlEventNode — represents a parsed <event name="X" script="Y.lua" /> node.
// Mirrors the XML parsing done in BaseEvents::loadFromXml in C++.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct XmlEventNode {
    /// Value of the `name` attribute.
    pub name: String,
    /// Value of the `script` attribute (optional).
    pub script: Option<String>,
    /// Value of the `function` attribute (optional).
    pub function: Option<String>,
}

impl XmlEventNode {
    /// Parse a minimal XML event node string.
    /// Supports: `<event name="X" script="Y.lua" function="Z" />`
    /// This is a pure-Rust parser that does not depend on any XML library;
    /// it covers the attribute subset used by BaseEvents::loadFromXml.
    pub fn parse(xml: &str) -> Option<Self> {
        let name = Self::extract_attr(xml, "name")?;
        let script = Self::extract_attr(xml, "script");
        let function = Self::extract_attr(xml, "function");
        Some(Self {
            name,
            script,
            function,
        })
    }

    fn extract_attr(xml: &str, attr: &str) -> Option<String> {
        let needle = format!("{}=\"", attr);
        let start = xml.find(&needle)? + needle.len();
        let rest = &xml[start..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    }
}

// ---------------------------------------------------------------------------
// EventHandler — groups BaseEvent entries under a single handler name.
// Used by the higher-level BaseEvents registry.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub name: String,
    pub events: Vec<BaseEvent>,
}

impl EventHandler {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            events: Vec::new(),
        }
    }

    pub fn register(&mut self, event: BaseEvent) {
        self.events.push(event);
    }

    pub fn get_event(&self, script_name: &str) -> Option<&BaseEvent> {
        self.events.iter().find(|e| e.script_name == script_name)
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

// ---------------------------------------------------------------------------
// BaseEvents — top-level registry, mirrors C++ BaseEvents class.
// Owns a `loaded` flag (mirrors `bool loaded`) and supports reload().
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct BaseEvents {
    pub handlers: HashMap<String, EventHandler>,
    loaded: bool,
}

impl BaseEvents {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            loaded: false,
        }
    }

    /// Returns true if events have been loaded from XML (mirrors `isLoaded()`).
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Set the loaded flag (used after a successful XML parse).
    pub fn set_loaded(&mut self, value: bool) {
        self.loaded = value;
    }

    /// Clear all handlers and reset the loaded flag, mirroring `reload()` in C++.
    /// In C++ reload() sets `loaded = false`, calls `clear(false)`, then
    /// calls `loadFromXml()`. Here we expose the reset step; callers drive
    /// the re-parse.
    pub fn reload(&mut self) {
        self.loaded = false;
        self.handlers.clear();
    }

    pub fn add_handler(&mut self, handler: EventHandler) {
        self.handlers.insert(handler.name.clone(), handler);
    }

    pub fn get_handler(&self, name: &str) -> Option<&EventHandler> {
        self.handlers.get(name)
    }
}

// ---------------------------------------------------------------------------
// CallBack — mirrors the C++ CallBack class: holds a resolved script id and
// a loaded flag. The name is the Lua function name resolved via getEvent().
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct CallBack {
    pub script_id: i32,
    pub name: Option<String>,
    loaded: bool,
}

impl CallBack {
    pub fn new() -> Self {
        Self {
            script_id: 0,
            name: None,
            loaded: false,
        }
    }

    /// Returns true if this callback has been successfully loaded.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Load the callback with the given resolved Lua function id and name.
    /// Mirrors `CallBack::loadCallBack` in C++ (success path).
    pub fn load(&mut self, id: i32, name: impl Into<String>) {
        self.script_id = id;
        self.name = Some(name.into());
        self.loaded = true;
    }

    /// Attempt to load; returns Err if id is negative (mirrors the `-1` sentinel
    /// that `scriptInterface->getEvent()` returns when the function is not found).
    pub fn try_load(&mut self, id: i32, name: impl Into<String>) -> Result<(), EventError> {
        if id < 0 {
            return Err(EventError::NotFound(name.into()));
        }
        self.script_id = id;
        self.name = Some(name.into());
        self.loaded = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_event_struct_has_script_name() {
        let event = BaseEvent::new("door_script");
        assert_eq!(event.script_name, "door_script");
    }

    #[test]
    fn event_handler_new_creates_empty_handler() {
        let handler = EventHandler::new("on_use");
        assert_eq!(handler.name, "on_use");
        assert_eq!(handler.event_count(), 0);
    }

    #[test]
    fn event_handler_register_adds_event() {
        let mut handler = EventHandler::new("on_use");
        handler.register(BaseEvent::new("script_a"));
        assert_eq!(handler.event_count(), 1);
    }

    #[test]
    fn event_handler_get_event_returns_some_when_found() {
        let mut handler = EventHandler::new("on_use");
        handler.register(BaseEvent::new("script_a"));
        let event = handler.get_event("script_a");
        assert!(event.is_some());
        assert_eq!(event.unwrap().script_name, "script_a");
    }

    #[test]
    fn event_handler_get_event_returns_none_when_not_found() {
        let handler = EventHandler::new("on_use");
        assert!(handler.get_event("nonexistent").is_none());
    }

    #[test]
    fn event_handler_event_count_returns_count() {
        let mut handler = EventHandler::new("on_use");
        handler.register(BaseEvent::new("a"));
        handler.register(BaseEvent::new("b"));
        assert_eq!(handler.event_count(), 2);
    }

    #[test]
    fn base_events_new_creates_empty_registry() {
        let events = BaseEvents::new();
        assert!(events.handlers.is_empty());
    }

    #[test]
    fn base_events_add_handler_adds_handler() {
        let mut base = BaseEvents::new();
        base.add_handler(EventHandler::new("on_use"));
        assert!(base.get_handler("on_use").is_some());
    }

    #[test]
    fn base_events_get_handler_returns_none_for_unknown() {
        let base = BaseEvents::new();
        assert!(base.get_handler("unknown").is_none());
    }

    #[test]
    fn base_events_get_handler_returns_correct_handler() {
        let mut base = BaseEvents::new();
        let mut handler = EventHandler::new("my_handler");
        handler.register(BaseEvent::new("my_script"));
        base.add_handler(handler);

        let h = base.get_handler("my_handler").unwrap();
        assert_eq!(h.name, "my_handler");
        assert_eq!(h.event_count(), 1);
    }

    // --- Phase 13.8 new tests ---

    /// register_event_stores_name: event name is retrievable after registration
    #[test]
    fn register_event_stores_name() {
        let mut registry = NamedEventRegistry::new();
        registry.register(
            "onLogin".to_string(),
            ScriptedEvent::new("login.lua", "onLogin"),
        );
        assert!(registry.get("onLogin").is_some());
        assert_eq!(registry.get("onLogin").unwrap().event_name, "onLogin");
    }

    /// execute_unregistered_event_returns_false: returns Err for unknown events
    #[test]
    fn execute_unregistered_event_returns_false() {
        let registry = NamedEventRegistry::new();
        let result = registry.execute("onUnknown");
        assert!(result.is_err());
    }

    /// register_multiple_events_by_name: each event name stored independently
    #[test]
    fn register_multiple_events_by_name() {
        let mut registry = NamedEventRegistry::new();
        registry.register(
            "onLogin".to_string(),
            ScriptedEvent::new("login.lua", "onLogin"),
        );
        registry.register(
            "onLogout".to_string(),
            ScriptedEvent::new("logout.lua", "onLogout"),
        );
        registry.register(
            "onDeath".to_string(),
            ScriptedEvent::new("death.lua", "onDeath"),
        );
        assert_eq!(registry.len(), 3);
        assert!(registry.get("onLogin").is_some());
        assert!(registry.get("onLogout").is_some());
        assert!(registry.get("onDeath").is_some());
    }

    /// xml_registration_parses_script_filename: parses <event name="X" script="Y.lua" />
    #[test]
    fn xml_registration_parses_script_filename() {
        let xml = r#"<event name="onLogin" script="login.lua" />"#;
        let parsed = XmlEventNode::parse(xml).unwrap();
        assert_eq!(parsed.name, "onLogin");
        assert_eq!(parsed.script.as_deref(), Some("login.lua"));
        assert!(parsed.function.is_none());
    }

    /// xml_registration_parses_function_attribute
    #[test]
    fn xml_registration_parses_function_attribute() {
        let xml = r#"<event name="onDeath" function="onDeathHandler" />"#;
        let parsed = XmlEventNode::parse(xml).unwrap();
        assert_eq!(parsed.name, "onDeath");
        assert!(parsed.script.is_none());
        assert_eq!(parsed.function.as_deref(), Some("onDeathHandler"));
    }

    /// xml_registration_parses_both_script_and_function
    #[test]
    fn xml_registration_parses_both_script_and_function() {
        let xml = r#"<event name="onMove" script="move.lua" function="onMoveHandler" />"#;
        let parsed = XmlEventNode::parse(xml).unwrap();
        assert_eq!(parsed.name, "onMove");
        assert_eq!(parsed.script.as_deref(), Some("move.lua"));
        assert_eq!(parsed.function.as_deref(), Some("onMoveHandler"));
    }

    /// base_event_execute_calls_correct_handler: dispatch calls the right function
    #[test]
    fn base_event_execute_calls_correct_handler() {
        let mut registry = NamedEventRegistry::new();
        registry.register(
            "onLogin".to_string(),
            ScriptedEvent::new("login.lua", "onLogin"),
        );
        registry.register(
            "onDeath".to_string(),
            ScriptedEvent::new("death.lua", "onDeath"),
        );

        // Executing a registered event returns Ok with the matched event name
        let result = registry.execute("onDeath");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "onDeath");

        // Wrong name returns error, not the other handler
        assert!(registry.execute("onLogin").is_ok());
        assert!(registry.execute("onUnknown").is_err());
    }

    // --- ScriptedEvent flag tests ---

    #[test]
    fn scripted_event_default_not_scripted() {
        let event = ScriptedEvent::new("test.lua", "onTest");
        assert!(!event.is_scripted());
        assert!(!event.from_lua);
        assert_eq!(event.get_script_id(), 0);
    }

    #[test]
    fn scripted_event_mark_scripted_sets_flag() {
        let mut event = ScriptedEvent::new("test.lua", "onTest");
        event.mark_scripted(42);
        assert!(event.is_scripted());
        assert_eq!(event.get_script_id(), 42);
    }

    // --- BaseEvents loaded flag tests ---

    #[test]
    fn base_events_not_loaded_by_default() {
        let base = BaseEvents::new();
        assert!(!base.is_loaded());
    }

    #[test]
    fn base_events_set_loaded() {
        let mut base = BaseEvents::new();
        base.set_loaded(true);
        assert!(base.is_loaded());
    }

    #[test]
    fn base_events_reload_resets_loaded_flag() {
        let mut base = BaseEvents::new();
        base.set_loaded(true);
        base.add_handler(EventHandler::new("h1"));
        base.reload();
        assert!(!base.is_loaded());
        assert!(base.handlers.is_empty());
    }

    // --- CallBack tests ---

    #[test]
    fn callback_new_not_loaded() {
        let cb = CallBack::new();
        assert!(!cb.is_loaded());
        assert_eq!(cb.script_id, 0);
    }

    #[test]
    fn callback_load_sets_loaded() {
        let mut cb = CallBack::new();
        cb.load(7, "onCallback");
        assert!(cb.is_loaded());
        assert_eq!(cb.script_id, 7);
        assert_eq!(cb.name.as_deref(), Some("onCallback"));
    }

    #[test]
    fn callback_load_fails_when_id_is_negative() {
        let mut cb = CallBack::new();
        let result = cb.try_load(-1, "onMissing");
        assert!(result.is_err());
        assert!(!cb.is_loaded());
    }

    // --- Phase 15 audit additions (Layer 10 re-audit) ---

    /// Covers `EventError::Display::fmt` (formatting branch — exercised via
    /// `format!`/`to_string`). Mirrors the C++ "[Warning] Event ... not found"
    /// log message produced when `scriptInterface->getEvent()` fails.
    #[test]
    fn event_error_display_includes_event_name() {
        let err = EventError::NotFound("onLogin".to_string());
        let rendered = format!("{}", err);
        assert_eq!(rendered, "Event 'onLogin' not found");
        // Sanity: `to_string` path also exercises the Display impl.
        assert_eq!(err.to_string(), "Event 'onLogin' not found");
    }

    /// Covers `NamedEventRegistry::is_empty`: true on construction, false after
    /// a registration, mirroring the implicit empty-check the C++ caller does
    /// before iterating `eventMap`.
    #[test]
    fn named_event_registry_is_empty_transitions_with_registration() {
        let mut registry = NamedEventRegistry::new();
        assert!(registry.is_empty());
        registry.register(
            "onLogin".to_string(),
            ScriptedEvent::new("login.lua", "onLogin"),
        );
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    /// Covers the success path of `CallBack::try_load` (id >= 0): mirrors
    /// `CallBack::loadCallBack` returning true when `getEvent(name)` resolves
    /// to a non-negative function id.
    #[test]
    fn callback_try_load_succeeds_with_non_negative_id() {
        let mut cb = CallBack::new();
        let result = cb.try_load(5, "onConfigured");
        assert!(result.is_ok());
        assert!(cb.is_loaded());
        assert_eq!(cb.script_id, 5);
        assert_eq!(cb.name.as_deref(), Some("onConfigured"));
    }

    /// Boundary: id == 0 is "non-negative" and should succeed (mirrors C++,
    /// which only treats `-1` as the missing-function sentinel).
    #[test]
    fn callback_try_load_succeeds_with_zero_id() {
        let mut cb = CallBack::new();
        let result = cb.try_load(0, "onZero");
        assert!(result.is_ok());
        assert!(cb.is_loaded());
        assert_eq!(cb.script_id, 0);
    }

    /// `XmlEventNode::parse` returns None when the required `name` attribute
    /// is absent (mirrors the C++ pugixml call where `node.attribute("name")`
    /// is missing and the event is skipped).
    #[test]
    fn xml_event_node_parse_returns_none_without_name_attribute() {
        let xml = r#"<event script="login.lua" />"#;
        assert!(XmlEventNode::parse(xml).is_none());
    }

    /// `XmlEventNode::extract_attr` returns None when the opening quote is
    /// present but the closing quote is missing (malformed XML guard).
    #[test]
    fn xml_event_node_parse_returns_none_with_unterminated_attribute() {
        // `name="onLogin` is missing its closing `"`; `rest.find('"')` is None.
        let xml = "<event name=\"onLogin";
        assert!(XmlEventNode::parse(xml).is_none());
    }
}
