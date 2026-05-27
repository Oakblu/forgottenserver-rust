use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CreatureEventType {
    Login,
    Logout,
    Reconnect,
    Think,
    PrepareDeath,
    Death,
    Kill,
    Advance,
    ModalWindow,
    TextEdit,
    HealthChange,
    ManaChange,
    ExtendedOpcode,
}

/// Arguments passed to the Lua function for each event type.
/// Mirrors the C++ execute* method signatures.
#[derive(Debug, Clone, PartialEq)]
pub enum CreatureEventArgs {
    /// onLogin(player)
    Login { player_id: u32 },
    /// onLogout(player)
    Logout { player_id: u32 },
    /// onReconnect(player)
    Reconnect { player_id: u32 },
    /// onThink(creature, interval)
    Think { creature_id: u32, interval: u32 },
    /// onPrepareDeath(creature, killer)
    PrepareDeath {
        creature_id: u32,
        killer_id: Option<u32>,
    },
    /// onDeath(creature, corpse, killer, mostDamageKiller, lastHitUnjustified, mostDamageUnjustified)
    Death {
        creature_id: u32,
        corpse_id: Option<u32>,
        killer_id: Option<u32>,
        most_damage_killer_id: Option<u32>,
        last_hit_unjustified: bool,
        most_damage_unjustified: bool,
    },
    /// onKill(creature, target)
    Kill { creature_id: u32, target_id: u32 },
    /// onAdvance(player, skill, oldLevel, newLevel)
    Advance {
        player_id: u32,
        skill: u32,
        old_level: u32,
        new_level: u32,
    },
    /// onModalWindow(player, modalWindowId, buttonId, choiceId)
    ModalWindow {
        player_id: u32,
        modal_window_id: u32,
        button_id: u8,
        choice_id: u8,
    },
    /// onTextEdit(player, item, text, windowTextId)
    TextEdit {
        player_id: u32,
        item_id: u32,
        text: String,
        window_text_id: u32,
    },
    /// onHealthChange(creature, attacker, primaryDamage, primaryType, secondaryDamage, secondaryType, origin)
    HealthChange {
        creature_id: u32,
        attacker_id: Option<u32>,
        primary_damage: i32,
        primary_type: u32,
        secondary_damage: i32,
        secondary_type: u32,
        origin: u32,
    },
    /// onManaChange(creature, attacker, primaryDamage, primaryType, secondaryDamage, secondaryType, origin)
    ManaChange {
        creature_id: u32,
        attacker_id: Option<u32>,
        primary_damage: i32,
        primary_type: u32,
        secondary_damage: i32,
        secondary_type: u32,
        origin: u32,
    },
    /// onExtendedOpcode(player, opcode, buffer)
    ExtendedOpcode {
        player_id: u32,
        opcode: u8,
        buffer: String,
    },
}

impl CreatureEventArgs {
    /// Returns the event type that matches these arguments.
    pub fn event_type(&self) -> CreatureEventType {
        match self {
            CreatureEventArgs::Login { .. } => CreatureEventType::Login,
            CreatureEventArgs::Logout { .. } => CreatureEventType::Logout,
            CreatureEventArgs::Reconnect { .. } => CreatureEventType::Reconnect,
            CreatureEventArgs::Think { .. } => CreatureEventType::Think,
            CreatureEventArgs::PrepareDeath { .. } => CreatureEventType::PrepareDeath,
            CreatureEventArgs::Death { .. } => CreatureEventType::Death,
            CreatureEventArgs::Kill { .. } => CreatureEventType::Kill,
            CreatureEventArgs::Advance { .. } => CreatureEventType::Advance,
            CreatureEventArgs::ModalWindow { .. } => CreatureEventType::ModalWindow,
            CreatureEventArgs::TextEdit { .. } => CreatureEventType::TextEdit,
            CreatureEventArgs::HealthChange { .. } => CreatureEventType::HealthChange,
            CreatureEventArgs::ManaChange { .. } => CreatureEventType::ManaChange,
            CreatureEventArgs::ExtendedOpcode { .. } => CreatureEventType::ExtendedOpcode,
        }
    }
}

/// Returns the Lua script event name for each event type (mirrors C++ getScriptEventName).
pub fn script_event_name(event_type: &CreatureEventType) -> &'static str {
    match event_type {
        CreatureEventType::Login => "onLogin",
        CreatureEventType::Logout => "onLogout",
        CreatureEventType::Reconnect => "onReconnect",
        CreatureEventType::Think => "onThink",
        CreatureEventType::PrepareDeath => "onPrepareDeath",
        CreatureEventType::Death => "onDeath",
        CreatureEventType::Kill => "onKill",
        CreatureEventType::Advance => "onAdvance",
        CreatureEventType::ModalWindow => "onModalWindow",
        CreatureEventType::TextEdit => "onTextEdit",
        CreatureEventType::HealthChange => "onHealthChange",
        CreatureEventType::ManaChange => "onManaChange",
        CreatureEventType::ExtendedOpcode => "onExtendedOpcode",
    }
}

#[derive(Debug, Clone)]
pub struct CreatureEvent {
    pub name: String,
    pub event_type: CreatureEventType,
    pub script_name: String,
    /// Whether this event has been fully loaded (has a valid script).
    pub loaded: bool,
    /// Whether this event was registered from Lua (vs XML).
    pub from_lua: bool,
}

impl CreatureEvent {
    pub fn new(
        name: impl Into<String>,
        event_type: CreatureEventType,
        script_name: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            event_type,
            script_name: script_name.into(),
            loaded: true,
            from_lua: false,
        }
    }

    pub fn new_unloaded(
        name: impl Into<String>,
        event_type: CreatureEventType,
        script_name: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            event_type,
            script_name: script_name.into(),
            loaded: false,
            from_lua: false,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Mirrors C++ clearEvent(): marks the event as unloaded.
    pub fn clear_event(&mut self) {
        self.loaded = false;
    }

    /// Mirrors C++ copyEvent(): copies script info from another event.
    pub fn copy_event(&mut self, other: &CreatureEvent) {
        self.script_name = other.script_name.clone();
        self.loaded = other.loaded;
        self.from_lua = other.from_lua;
    }

    /// Returns the Lua script event name for this event.
    pub fn script_event_name(&self) -> &'static str {
        script_event_name(&self.event_type)
    }
}

/// Optional dispatcher hook used by [`CreatureEvents`] when firing events.
///
/// In production this is `None` and the stub dispatcher always returns `true`
/// (real Lua dispatch is feature-gated and lives elsewhere). Tests may inject
/// a closure to simulate a Lua handler returning `false`, exercising the
/// early-exit paths in `player_login` / `player_logout` / `player_advance`
/// which mirror the C++ `for (...) if (!execute*) return false;` pattern.
pub type CreatureEventsDispatcher =
    Box<dyn Fn(&CreatureEvent, &CreatureEventArgs) -> bool + Send + Sync>;

#[derive(Default)]
pub struct CreatureEvents {
    events: HashMap<String, CreatureEvent>,
    dispatcher: Option<CreatureEventsDispatcher>,
}

impl std::fmt::Debug for CreatureEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreatureEvents")
            .field("events", &self.events)
            .field("dispatcher", &self.dispatcher.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl CreatureEvents {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            dispatcher: None,
        }
    }

    /// Install a dispatcher used to simulate the Lua handler return value.
    /// Primarily intended for tests; production code leaves this unset and
    /// `dispatch_bool` returns `true`.
    pub fn set_dispatcher(&mut self, dispatcher: CreatureEventsDispatcher) {
        self.dispatcher = Some(dispatcher);
    }

    pub fn register(&mut self, event: CreatureEvent) {
        self.events.insert(event.name.to_lowercase(), event);
    }

    /// Mirrors C++ getEventByName(name, forceLoaded).
    /// If `force_loaded` is true, only returns events that are loaded.
    pub fn get_event_by_name(&self, name: &str, force_loaded: bool) -> Option<&CreatureEvent> {
        let key = name.to_lowercase();
        match self.events.get(&key) {
            Some(ev) if !force_loaded || ev.is_loaded() => Some(ev),
            _ => None,
        }
    }

    /// Convenience: get loaded event by name (default force_loaded=true).
    pub fn get_event(&self, name: &str) -> Option<&CreatureEvent> {
        self.get_event_by_name(name, true)
    }

    /// Mirrors C++ playerLogin: fires all Login events globally.
    /// Returns false if any event handler returns false.
    pub fn player_login(&self, player_id: u32) -> bool {
        for ev in self.events.values() {
            if ev.event_type == CreatureEventType::Login
                && ev.is_loaded()
                && !self.dispatch_bool(ev, &CreatureEventArgs::Login { player_id })
            {
                return false;
            }
        }
        true
    }

    /// Mirrors C++ playerLogout: fires all Logout events globally.
    /// Returns false if any event handler returns false.
    pub fn player_logout(&self, player_id: u32) -> bool {
        for ev in self.events.values() {
            if ev.event_type == CreatureEventType::Logout
                && ev.is_loaded()
                && !self.dispatch_bool(ev, &CreatureEventArgs::Logout { player_id })
            {
                return false;
            }
        }
        true
    }

    /// Mirrors C++ playerReconnect: fires all Reconnect events globally.
    pub fn player_reconnect(&self, player_id: u32) {
        for ev in self.events.values() {
            if ev.event_type == CreatureEventType::Reconnect && ev.is_loaded() {
                self.dispatch_void(ev, &CreatureEventArgs::Reconnect { player_id });
            }
        }
    }

    /// Mirrors C++ playerAdvance: fires all Advance events globally.
    /// Returns false if any event handler returns false.
    pub fn player_advance(
        &self,
        player_id: u32,
        skill: u32,
        old_level: u32,
        new_level: u32,
    ) -> bool {
        for ev in self.events.values() {
            if ev.event_type == CreatureEventType::Advance && ev.is_loaded() {
                let args = CreatureEventArgs::Advance {
                    player_id,
                    skill,
                    old_level,
                    new_level,
                };
                if !self.dispatch_bool(ev, &args) {
                    return false;
                }
            }
        }
        true
    }

    /// Clear events, optionally only those matching from_lua flag.
    pub fn clear(&mut self, from_lua: bool) {
        for ev in self.events.values_mut() {
            if ev.from_lua == from_lua {
                ev.clear_event();
            }
        }
    }

    /// Removes events that are not loaded (scriptId == 0 equivalent).
    pub fn remove_invalid_events(&mut self) {
        self.events.retain(|_, ev| ev.is_loaded());
    }

    /// Stub dispatch that delegates to the optional dispatcher closure when
    /// installed and otherwise returns `true`. The real Lua dispatch is
    /// feature-gated and lives outside this module.
    fn dispatch_bool(&self, ev: &CreatureEvent, args: &CreatureEventArgs) -> bool {
        match &self.dispatcher {
            Some(d) => d(ev, args),
            None => true,
        }
    }

    fn dispatch_void(&self, ev: &CreatureEvent, args: &CreatureEventArgs) {
        if let Some(d) = &self.dispatcher {
            let _ = d(ev, args);
        }
    }
}

/// Per-creature event tracking
#[derive(Debug, Default)]
pub struct CreatureEventList {
    // maps event_name -> event_type for quick lookup
    events: HashMap<String, CreatureEventType>,
}

impl CreatureEventList {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event_name: impl Into<String>, event_type: CreatureEventType) {
        self.events.insert(event_name.into(), event_type);
    }

    pub fn remove_event(&mut self, event_name: &str) {
        self.events.remove(event_name);
    }

    pub fn has_event(&self, event_name: &str) -> bool {
        self.events.contains_key(event_name)
    }

    pub fn get_events_of_type(&self, event_type: CreatureEventType) -> Vec<&str> {
        self.events
            .iter()
            .filter(|(_, t)| **t == event_type)
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── original tests (preserved) ──────────────────────────────────────────

    #[test]
    fn creature_event_type_enum_variants_exist() {
        let _ = CreatureEventType::Login;
        let _ = CreatureEventType::Logout;
        let _ = CreatureEventType::Think;
        let _ = CreatureEventType::PrepareDeath;
        let _ = CreatureEventType::Death;
        let _ = CreatureEventType::Kill;
        let _ = CreatureEventType::Advance;
        let _ = CreatureEventType::ModalWindow;
        let _ = CreatureEventType::TextEdit;
        let _ = CreatureEventType::HealthChange;
        let _ = CreatureEventType::ManaChange;
        let _ = CreatureEventType::ExtendedOpcode;
    }

    #[test]
    fn creature_event_struct_fields() {
        let event = CreatureEvent::new("login_event", CreatureEventType::Login, "player_login.lua");
        assert_eq!(event.name, "login_event");
        assert_eq!(event.event_type, CreatureEventType::Login);
        assert_eq!(event.script_name, "player_login.lua");
    }

    #[test]
    fn creature_events_new_creates_empty_registry() {
        let events = CreatureEvents::new();
        assert!(events.get_event("any").is_none());
    }

    #[test]
    fn register_adds_event_by_name() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        assert!(events.get_event("on_login").is_some());
    }

    #[test]
    fn get_event_returns_some_when_registered() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "ev",
            CreatureEventType::Death,
            "death.lua",
        ));
        let e = events.get_event("ev").unwrap();
        assert_eq!(e.event_type, CreatureEventType::Death);
    }

    #[test]
    fn get_event_returns_none_when_not_registered() {
        let events = CreatureEvents::new();
        assert!(events.get_event("nonexistent").is_none());
    }

    #[test]
    fn creature_event_list_new_creates_empty_list() {
        let list = CreatureEventList::new();
        assert!(!list.has_event("any"));
    }

    #[test]
    fn add_event_registers_event_for_creature() {
        let mut list = CreatureEventList::new();
        list.add_event("on_login", CreatureEventType::Login);
        assert!(list.has_event("on_login"));
    }

    #[test]
    fn remove_event_removes_it() {
        let mut list = CreatureEventList::new();
        list.add_event("on_login", CreatureEventType::Login);
        list.remove_event("on_login");
        assert!(!list.has_event("on_login"));
    }

    #[test]
    fn has_event_returns_false_for_missing() {
        let list = CreatureEventList::new();
        assert!(!list.has_event("missing"));
    }

    #[test]
    fn get_events_of_type_returns_matching_event_names() {
        let mut list = CreatureEventList::new();
        list.add_event("login_a", CreatureEventType::Login);
        list.add_event("login_b", CreatureEventType::Login);
        list.add_event("death_a", CreatureEventType::Death);

        let mut login_events = list.get_events_of_type(CreatureEventType::Login);
        login_events.sort();
        assert_eq!(login_events, vec!["login_a", "login_b"]);

        let death_events = list.get_events_of_type(CreatureEventType::Death);
        assert_eq!(death_events, vec!["death_a"]);
    }

    #[test]
    fn get_events_of_type_returns_empty_when_no_match() {
        let list = CreatureEventList::new();
        let results = list.get_events_of_type(CreatureEventType::Kill);
        assert!(results.is_empty());
    }

    // ── new tests: Reconnect variant ────────────────────────────────────────

    #[test]
    fn creature_event_type_reconnect_variant_exists() {
        let _ = CreatureEventType::Reconnect;
    }

    #[test]
    fn creature_event_reconnect_can_be_registered() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_reconnect",
            CreatureEventType::Reconnect,
            "reconnect.lua",
        ));
        let ev = events.get_event("on_reconnect").unwrap();
        assert_eq!(ev.event_type, CreatureEventType::Reconnect);
    }

    #[test]
    fn creature_event_list_add_reconnect() {
        let mut list = CreatureEventList::new();
        list.add_event("on_reconnect", CreatureEventType::Reconnect);
        assert!(list.has_event("on_reconnect"));
        let results = list.get_events_of_type(CreatureEventType::Reconnect);
        assert_eq!(results, vec!["on_reconnect"]);
    }

    // ── new tests: loaded / unloaded state ──────────────────────────────────

    #[test]
    fn creature_event_new_is_loaded() {
        let ev = CreatureEvent::new("ev", CreatureEventType::Login, "login.lua");
        assert!(ev.is_loaded());
    }

    #[test]
    fn creature_event_new_unloaded_is_not_loaded() {
        let ev = CreatureEvent::new_unloaded("ev", CreatureEventType::Login, "login.lua");
        assert!(!ev.is_loaded());
    }

    #[test]
    fn clear_event_marks_event_unloaded() {
        let mut ev = CreatureEvent::new("ev", CreatureEventType::Login, "login.lua");
        assert!(ev.is_loaded());
        ev.clear_event();
        assert!(!ev.is_loaded());
    }

    #[test]
    fn copy_event_copies_script_info() {
        let mut target = CreatureEvent::new_unloaded("target", CreatureEventType::Login, "");
        let source = CreatureEvent::new("source", CreatureEventType::Login, "login.lua");
        target.copy_event(&source);
        assert_eq!(target.script_name, "login.lua");
        assert!(target.is_loaded());
    }

    // ── new tests: get_event_by_name with force_loaded ───────────────────────

    #[test]
    fn get_event_by_name_force_loaded_true_skips_unloaded() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new_unloaded(
            "ev",
            CreatureEventType::Login,
            "login.lua",
        ));
        // force_loaded=true should not find it
        assert!(events.get_event_by_name("ev", true).is_none());
    }

    #[test]
    fn get_event_by_name_force_loaded_false_finds_unloaded() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new_unloaded(
            "ev",
            CreatureEventType::Login,
            "login.lua",
        ));
        // force_loaded=false should find it
        assert!(events.get_event_by_name("ev", false).is_some());
    }

    #[test]
    fn get_event_by_name_force_loaded_true_finds_loaded() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "ev",
            CreatureEventType::Login,
            "login.lua",
        ));
        assert!(events.get_event_by_name("ev", true).is_some());
    }

    #[test]
    fn get_event_by_name_is_case_insensitive() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "OnLogin",
            CreatureEventType::Login,
            "login.lua",
        ));
        assert!(events.get_event_by_name("onlogin", true).is_some());
        assert!(events.get_event_by_name("ONLOGIN", true).is_some());
    }

    // ── new tests: global dispatch methods ──────────────────────────────────

    #[test]
    fn player_login_returns_true_when_no_login_events() {
        let events = CreatureEvents::new();
        assert!(events.player_login(1));
    }

    #[test]
    fn player_login_returns_true_with_loaded_login_event() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        assert!(events.player_login(42));
    }

    #[test]
    fn player_login_skips_unloaded_events() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new_unloaded(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        // Should not fire unloaded events, still returns true
        assert!(events.player_login(1));
    }

    #[test]
    fn player_logout_returns_true_when_no_logout_events() {
        let events = CreatureEvents::new();
        assert!(events.player_logout(1));
    }

    #[test]
    fn player_logout_returns_true_with_loaded_logout_event() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_logout",
            CreatureEventType::Logout,
            "logout.lua",
        ));
        assert!(events.player_logout(42));
    }

    #[test]
    fn player_reconnect_fires_only_reconnect_events() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_reconnect",
            CreatureEventType::Reconnect,
            "reconnect.lua",
        ));
        events.register(CreatureEvent::new(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        // Should not panic; fires reconnect event for player 99
        events.player_reconnect(99);
    }

    #[test]
    fn player_reconnect_skips_unloaded_events() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new_unloaded(
            "on_reconnect",
            CreatureEventType::Reconnect,
            "reconnect.lua",
        ));
        // Should not fire unloaded events (no panic)
        events.player_reconnect(1);
    }

    #[test]
    fn player_advance_returns_true_when_no_advance_events() {
        let events = CreatureEvents::new();
        assert!(events.player_advance(1, 0, 10, 11));
    }

    #[test]
    fn player_advance_returns_true_with_loaded_advance_event() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_advance",
            CreatureEventType::Advance,
            "advance.lua",
        ));
        assert!(events.player_advance(42, 1, 5, 6));
    }

    #[test]
    fn player_advance_skips_non_advance_events() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        // Advance dispatch should not fire login events
        assert!(events.player_advance(1, 0, 1, 2));
    }

    // ── new tests: clear and remove_invalid_events ───────────────────────────

    #[test]
    fn clear_marks_matching_from_lua_events_unloaded() {
        let mut events = CreatureEvents::new();
        let mut ev = CreatureEvent::new("ev_lua", CreatureEventType::Login, "login.lua");
        ev.from_lua = true;
        events.register(ev);

        let ev2 = CreatureEvent::new("ev_xml", CreatureEventType::Login, "login2.lua");
        // from_lua defaults to false
        events.register(ev2);

        events.clear(true); // clear only from_lua events

        // ev_lua should now be unloaded
        assert!(!events.get_event_by_name("ev_lua", false).unwrap().loaded);
        // ev_xml should still be loaded
        assert!(events.get_event_by_name("ev_xml", false).unwrap().loaded);
    }

    #[test]
    fn remove_invalid_events_removes_unloaded() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "ev_loaded",
            CreatureEventType::Login,
            "a.lua",
        ));
        events.register(CreatureEvent::new_unloaded(
            "ev_unloaded",
            CreatureEventType::Login,
            "b.lua",
        ));

        events.remove_invalid_events();

        assert!(events.get_event_by_name("ev_loaded", false).is_some());
        assert!(events.get_event_by_name("ev_unloaded", false).is_none());
    }

    // ── new tests: script event names ───────────────────────────────────────

    #[test]
    fn script_event_name_login() {
        assert_eq!(script_event_name(&CreatureEventType::Login), "onLogin");
    }

    #[test]
    fn script_event_name_logout() {
        assert_eq!(script_event_name(&CreatureEventType::Logout), "onLogout");
    }

    #[test]
    fn script_event_name_reconnect() {
        assert_eq!(
            script_event_name(&CreatureEventType::Reconnect),
            "onReconnect"
        );
    }

    #[test]
    fn script_event_name_think() {
        assert_eq!(script_event_name(&CreatureEventType::Think), "onThink");
    }

    #[test]
    fn script_event_name_prepare_death() {
        assert_eq!(
            script_event_name(&CreatureEventType::PrepareDeath),
            "onPrepareDeath"
        );
    }

    #[test]
    fn script_event_name_death() {
        assert_eq!(script_event_name(&CreatureEventType::Death), "onDeath");
    }

    #[test]
    fn script_event_name_kill() {
        assert_eq!(script_event_name(&CreatureEventType::Kill), "onKill");
    }

    #[test]
    fn script_event_name_advance() {
        assert_eq!(script_event_name(&CreatureEventType::Advance), "onAdvance");
    }

    #[test]
    fn script_event_name_modal_window() {
        assert_eq!(
            script_event_name(&CreatureEventType::ModalWindow),
            "onModalWindow"
        );
    }

    #[test]
    fn script_event_name_text_edit() {
        assert_eq!(
            script_event_name(&CreatureEventType::TextEdit),
            "onTextEdit"
        );
    }

    #[test]
    fn script_event_name_health_change() {
        assert_eq!(
            script_event_name(&CreatureEventType::HealthChange),
            "onHealthChange"
        );
    }

    #[test]
    fn script_event_name_mana_change() {
        assert_eq!(
            script_event_name(&CreatureEventType::ManaChange),
            "onManaChange"
        );
    }

    #[test]
    fn script_event_name_extended_opcode() {
        assert_eq!(
            script_event_name(&CreatureEventType::ExtendedOpcode),
            "onExtendedOpcode"
        );
    }

    #[test]
    fn creature_event_script_event_name_method() {
        let ev = CreatureEvent::new("test", CreatureEventType::Kill, "kill.lua");
        assert_eq!(ev.script_event_name(), "onKill");
    }

    // ── new tests: CreatureEventArgs ─────────────────────────────────────────

    #[test]
    fn creature_event_args_login_event_type() {
        let args = CreatureEventArgs::Login { player_id: 1 };
        assert_eq!(args.event_type(), CreatureEventType::Login);
    }

    #[test]
    fn creature_event_args_logout_event_type() {
        let args = CreatureEventArgs::Logout { player_id: 2 };
        assert_eq!(args.event_type(), CreatureEventType::Logout);
    }

    #[test]
    fn creature_event_args_reconnect_event_type() {
        let args = CreatureEventArgs::Reconnect { player_id: 3 };
        assert_eq!(args.event_type(), CreatureEventType::Reconnect);
    }

    #[test]
    fn creature_event_args_think_event_type() {
        let args = CreatureEventArgs::Think {
            creature_id: 1,
            interval: 2000,
        };
        assert_eq!(args.event_type(), CreatureEventType::Think);
    }

    #[test]
    fn creature_event_args_prepare_death_event_type() {
        let args = CreatureEventArgs::PrepareDeath {
            creature_id: 1,
            killer_id: Some(2),
        };
        assert_eq!(args.event_type(), CreatureEventType::PrepareDeath);
    }

    #[test]
    fn creature_event_args_prepare_death_no_killer() {
        let args = CreatureEventArgs::PrepareDeath {
            creature_id: 1,
            killer_id: None,
        };
        assert_eq!(args.event_type(), CreatureEventType::PrepareDeath);
    }

    #[test]
    fn creature_event_args_death_event_type() {
        let args = CreatureEventArgs::Death {
            creature_id: 1,
            corpse_id: Some(10),
            killer_id: Some(2),
            most_damage_killer_id: None,
            last_hit_unjustified: false,
            most_damage_unjustified: true,
        };
        assert_eq!(args.event_type(), CreatureEventType::Death);
    }

    #[test]
    fn creature_event_args_kill_event_type() {
        let args = CreatureEventArgs::Kill {
            creature_id: 1,
            target_id: 2,
        };
        assert_eq!(args.event_type(), CreatureEventType::Kill);
    }

    #[test]
    fn creature_event_args_advance_event_type() {
        let args = CreatureEventArgs::Advance {
            player_id: 1,
            skill: 3,
            old_level: 10,
            new_level: 11,
        };
        assert_eq!(args.event_type(), CreatureEventType::Advance);
    }

    #[test]
    fn creature_event_args_modal_window_event_type() {
        let args = CreatureEventArgs::ModalWindow {
            player_id: 1,
            modal_window_id: 5,
            button_id: 1,
            choice_id: 0,
        };
        assert_eq!(args.event_type(), CreatureEventType::ModalWindow);
    }

    #[test]
    fn creature_event_args_text_edit_event_type() {
        let args = CreatureEventArgs::TextEdit {
            player_id: 1,
            item_id: 100,
            text: "hello".to_string(),
            window_text_id: 7,
        };
        assert_eq!(args.event_type(), CreatureEventType::TextEdit);
    }

    #[test]
    fn creature_event_args_health_change_event_type() {
        let args = CreatureEventArgs::HealthChange {
            creature_id: 1,
            attacker_id: Some(2),
            primary_damage: -50,
            primary_type: 1,
            secondary_damage: 0,
            secondary_type: 0,
            origin: 0,
        };
        assert_eq!(args.event_type(), CreatureEventType::HealthChange);
    }

    #[test]
    fn creature_event_args_health_change_no_attacker() {
        let args = CreatureEventArgs::HealthChange {
            creature_id: 1,
            attacker_id: None,
            primary_damage: -10,
            primary_type: 2,
            secondary_damage: 0,
            secondary_type: 0,
            origin: 1,
        };
        assert_eq!(args.event_type(), CreatureEventType::HealthChange);
    }

    #[test]
    fn creature_event_args_mana_change_event_type() {
        let args = CreatureEventArgs::ManaChange {
            creature_id: 1,
            attacker_id: Some(2),
            primary_damage: -20,
            primary_type: 1,
            secondary_damage: 0,
            secondary_type: 0,
            origin: 0,
        };
        assert_eq!(args.event_type(), CreatureEventType::ManaChange);
    }

    #[test]
    fn creature_event_args_extended_opcode_event_type() {
        let args = CreatureEventArgs::ExtendedOpcode {
            player_id: 1,
            opcode: 10,
            buffer: "data".to_string(),
        };
        assert_eq!(args.event_type(), CreatureEventType::ExtendedOpcode);
    }

    // ── new tests: from_lua field ────────────────────────────────────────────

    #[test]
    fn creature_event_new_from_lua_defaults_false() {
        let ev = CreatureEvent::new("ev", CreatureEventType::Login, "login.lua");
        assert!(!ev.from_lua);
    }

    #[test]
    fn creature_event_from_lua_field_settable() {
        let mut ev = CreatureEvent::new("ev", CreatureEventType::Login, "login.lua");
        ev.from_lua = true;
        assert!(ev.from_lua);
    }

    // ── new tests: dispatcher injection exercises false-return paths ─────────
    // These cover the C++ early-exit semantics in playerLogin/playerLogout/
    // playerAdvance: `for (...) if (!execute*(...)) return false;`.

    #[test]
    fn player_login_returns_false_when_dispatcher_returns_false() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        events.set_dispatcher(Box::new(|_ev, _args| false));
        assert!(!events.player_login(1));
    }

    #[test]
    fn player_logout_returns_false_when_dispatcher_returns_false() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_logout",
            CreatureEventType::Logout,
            "logout.lua",
        ));
        events.set_dispatcher(Box::new(|_ev, _args| false));
        assert!(!events.player_logout(1));
    }

    #[test]
    fn player_advance_returns_false_when_dispatcher_returns_false() {
        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_advance",
            CreatureEventType::Advance,
            "advance.lua",
        ));
        events.set_dispatcher(Box::new(|_ev, _args| false));
        assert!(!events.player_advance(1, 0, 10, 11));
    }

    #[test]
    fn player_login_dispatcher_receives_event_and_args() {
        use std::sync::{Arc, Mutex};
        let captured: Arc<Mutex<Option<(String, CreatureEventArgs)>>> = Arc::new(Mutex::new(None));
        let cap = Arc::clone(&captured);

        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        events.set_dispatcher(Box::new(move |ev, args| {
            *cap.lock().unwrap() = Some((ev.name.clone(), args.clone()));
            true
        }));
        assert!(events.player_login(77));
        let bound = captured.lock().unwrap();
        let (name, args) = bound.as_ref().unwrap();
        assert_eq!(name, "on_login");
        assert_eq!(args.event_type(), CreatureEventType::Login);
        assert_eq!(
            args,
            &CreatureEventArgs::Login { player_id: 77 },
            "dispatcher should receive the originating player id",
        );
    }

    #[test]
    fn player_reconnect_dispatcher_is_invoked() {
        use std::sync::{Arc, Mutex};
        let count = Arc::new(Mutex::new((0u32, 0u32)));
        let c = Arc::clone(&count);

        let mut events = CreatureEvents::new();
        events.register(CreatureEvent::new(
            "on_reconnect",
            CreatureEventType::Reconnect,
            "reconnect.lua",
        ));
        // also register a non-reconnect (Login) to ensure dispatcher is filtered by type
        events.register(CreatureEvent::new(
            "on_login",
            CreatureEventType::Login,
            "login.lua",
        ));
        events.set_dispatcher(Box::new(move |_ev, args| {
            let mut g = c.lock().unwrap();
            if args.event_type() == CreatureEventType::Reconnect {
                g.0 += 1;
            } else {
                g.1 += 1;
            }
            true
        }));
        // Reconnect dispatch fires the dispatcher for the Reconnect-typed event only.
        events.player_reconnect(5);
        // Login dispatch fires the dispatcher for the Login-typed event only.
        // This exercises both branches of the dispatcher's `if args.event_type() == Reconnect`
        // selector so all regions inside the closure are reached.
        assert!(events.player_login(7));
        let g = count.lock().unwrap();
        assert_eq!(g.0, 1, "reconnect path should fire exactly once");
        assert_eq!(g.1, 1, "non-reconnect path should fire exactly once");
    }

    #[test]
    fn creature_events_default_yields_empty_registry() {
        let events = CreatureEvents::default();
        assert!(events.get_event("any").is_none());
        assert!(events.player_login(1));
    }

    #[test]
    fn creature_event_list_default_yields_empty_registry() {
        let list = CreatureEventList::default();
        assert!(!list.has_event("any"));
    }

    #[test]
    fn debug_impl_renders_dispatcher_marker() {
        let mut events = CreatureEvents::new();
        let s_unset = format!("{:?}", events);
        assert!(s_unset.contains("None"));
        events.set_dispatcher(Box::new(|_ev, _args| true));
        let s_set = format!("{:?}", events);
        assert!(s_set.contains("<fn>"));
    }
}
