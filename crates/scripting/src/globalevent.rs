use std::collections::HashMap;

/// Mirrors C++ GlobalEvent_t enum.
///
/// - `Think`    → "onThink"   (periodic, fires every `interval_ms`)
/// - `Time`     → "onTime"    (fires at a specific hour:minute:second each day)
/// - `Startup`  → "onStartup" (fires once when server starts)
/// - `Shutdown` → "onShutdown"(fires once when server shuts down)
/// - `Record`   → "onRecord"  (fires when player count beats previous record)
/// - `Save`     → "onSave"    (fires when server saves)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalEventType {
    Think,
    Time,
    Startup,
    Shutdown,
    Record,
    Save,
    /// Legacy alias kept for backward compatibility (maps to Think)
    Timer,
}

/// Scheduled time-of-day for `Time` events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl ScheduledTime {
    pub fn new(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }

    /// Returns true when `(hour, minute, second)` matches the scheduled time.
    pub fn matches(&self, hour: u8, minute: u8, second: u8) -> bool {
        self.hour == hour && self.minute == minute && self.second == second
    }
}

#[derive(Debug, Clone)]
pub struct GlobalEvent {
    pub name: String,
    pub event_type: GlobalEventType,
    pub script_name: String,
    /// Interval in milliseconds for Think/Timer events.
    pub interval_ms: Option<u64>,
    /// Time-of-day schedule for Time events.
    pub scheduled_time: Option<ScheduledTime>,
    /// Whether this event has already fired (used for Startup/Shutdown once-semantics).
    fired: bool,
    /// Tracks whether this is a once-only event that has fired.
    once: bool,
}

impl GlobalEvent {
    pub fn new(
        name: impl Into<String>,
        event_type: GlobalEventType,
        script_name: impl Into<String>,
    ) -> Self {
        let once = matches!(
            event_type,
            GlobalEventType::Startup | GlobalEventType::Shutdown
        );
        Self {
            name: name.into(),
            event_type,
            script_name: script_name.into(),
            interval_ms: None,
            scheduled_time: None,
            fired: false,
            once,
        }
    }

    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval_ms = Some(interval_ms);
        self
    }

    pub fn with_scheduled_time(mut self, hour: u8, minute: u8, second: u8) -> Self {
        self.scheduled_time = Some(ScheduledTime::new(hour, minute, second));
        self
    }

    /// Returns true when `now_ms - last_fire_ms >= interval_ms`.
    /// Returns false if no interval is set.
    pub fn is_due(&self, last_fire_ms: u64, now_ms: u64) -> bool {
        match self.interval_ms {
            Some(interval) => now_ms.saturating_sub(last_fire_ms) >= interval,
            None => false,
        }
    }

    /// Returns true when the current wall-clock time matches the scheduled hour:minute:second.
    /// Only relevant for `Time` events.
    pub fn is_time_due(&self, hour: u8, minute: u8, second: u8) -> bool {
        match &self.scheduled_time {
            Some(st) => st.matches(hour, minute, second),
            None => false,
        }
    }

    /// Mark this event as fired. For once-only events (Startup/Shutdown) this prevents
    /// re-firing. Returns `true` if the event should actually fire (has not fired yet for
    /// once-only events, or always true for repeating events).
    pub fn try_fire(&mut self) -> bool {
        if self.once {
            if self.fired {
                return false;
            }
            self.fired = true;
        }
        true
    }

    /// Whether this event has been fired at least once.
    pub fn has_fired(&self) -> bool {
        self.fired
    }

    /// Returns the Lua callback name for this event type.
    pub fn script_event_name(&self) -> &'static str {
        match self.event_type {
            GlobalEventType::Startup => "onStartup",
            GlobalEventType::Shutdown => "onShutdown",
            GlobalEventType::Record => "onRecord",
            GlobalEventType::Save => "onSave",
            GlobalEventType::Time | GlobalEventType::Timer => "onTime",
            GlobalEventType::Think => "onThink",
        }
    }
}

/// Result of attempting to fire a Record event.
#[derive(Debug, PartialEq, Eq)]
pub enum RecordFireResult {
    /// New record — event should fire with (current, old).
    NewRecord { current: u32, old: u32 },
    /// Count did not exceed the previous record; do not fire.
    NotARecord,
}

#[derive(Debug, Default)]
pub struct GlobalEvents {
    events: HashMap<String, GlobalEvent>,
    /// Tracks the all-time record player count for Record events.
    player_record: u32,
}

impl GlobalEvents {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            player_record: 0,
        }
    }

    pub fn register(&mut self, event: GlobalEvent) {
        self.events.insert(event.name.clone(), event);
    }

    pub fn get_event(&self, name: &str) -> Option<&GlobalEvent> {
        self.events.get(name)
    }

    pub fn get_event_mut(&mut self, name: &str) -> Option<&mut GlobalEvent> {
        self.events.get_mut(name)
    }

    pub fn get_startup_events(&self) -> Vec<&GlobalEvent> {
        self.events
            .values()
            .filter(|e| e.event_type == GlobalEventType::Startup)
            .collect()
    }

    pub fn get_shutdown_events(&self) -> Vec<&GlobalEvent> {
        self.events
            .values()
            .filter(|e| e.event_type == GlobalEventType::Shutdown)
            .collect()
    }

    pub fn get_save_events(&self) -> Vec<&GlobalEvent> {
        self.events
            .values()
            .filter(|e| e.event_type == GlobalEventType::Save)
            .collect()
    }

    pub fn get_record_events(&self) -> Vec<&GlobalEvent> {
        self.events
            .values()
            .filter(|e| e.event_type == GlobalEventType::Record)
            .collect()
    }

    /// Legacy: returns Timer and Think events that have an interval set.
    pub fn get_timer_events(&self) -> Vec<&GlobalEvent> {
        self.events
            .values()
            .filter(|e| {
                matches!(
                    e.event_type,
                    GlobalEventType::Timer | GlobalEventType::Think
                ) && e.interval_ms.is_some()
            })
            .collect()
    }

    /// Returns Think events (onThink — periodic).
    pub fn get_think_events(&self) -> Vec<&GlobalEvent> {
        self.events
            .values()
            .filter(|e| e.event_type == GlobalEventType::Think && e.interval_ms.is_some())
            .collect()
    }

    /// Returns Time events (onTime — time-of-day scheduled).
    pub fn get_time_events(&self) -> Vec<&GlobalEvent> {
        self.events
            .values()
            .filter(|e| {
                matches!(e.event_type, GlobalEventType::Time | GlobalEventType::Timer)
                    && e.scheduled_time.is_some()
            })
            .collect()
    }

    /// Check whether `player_count` beats the current record.
    /// Returns `RecordFireResult::NewRecord` if so (and updates internal record),
    /// or `RecordFireResult::NotARecord` otherwise.
    pub fn check_record(&mut self, player_count: u32) -> RecordFireResult {
        if player_count > self.player_record {
            let old = self.player_record;
            self.player_record = player_count;
            RecordFireResult::NewRecord {
                current: player_count,
                old,
            }
        } else {
            RecordFireResult::NotARecord
        }
    }

    pub fn player_record(&self) -> u32 {
        self.player_record
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── original tests (preserved) ───────────────────────────────────────────

    #[test]
    fn global_event_type_enum_variants_exist() {
        let _ = GlobalEventType::Startup;
        let _ = GlobalEventType::Shutdown;
        let _ = GlobalEventType::Record;
        let _ = GlobalEventType::Timer;
        let _ = GlobalEventType::Think;
        let _ = GlobalEventType::Time;
        let _ = GlobalEventType::Save;
    }

    #[test]
    fn global_event_struct_fields() {
        let event = GlobalEvent::new("startup", GlobalEventType::Startup, "startup.lua")
            .with_interval(5000);
        assert_eq!(event.name, "startup");
        assert_eq!(event.event_type, GlobalEventType::Startup);
        assert_eq!(event.script_name, "startup.lua");
        assert_eq!(event.interval_ms, Some(5000));
    }

    #[test]
    fn global_events_new_creates_empty_registry() {
        let events = GlobalEvents::new();
        assert!(events.get_event("any").is_none());
    }

    #[test]
    fn register_adds_event_by_name() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new(
            "on_start",
            GlobalEventType::Startup,
            "start.lua",
        ));
        assert!(events.get_event("on_start").is_some());
    }

    #[test]
    fn get_event_returns_some_when_registered() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new(
            "ev",
            GlobalEventType::Shutdown,
            "shutdown.lua",
        ));
        let e = events.get_event("ev").unwrap();
        assert_eq!(e.event_type, GlobalEventType::Shutdown);
    }

    #[test]
    fn get_event_returns_none_when_not_registered() {
        let events = GlobalEvents::new();
        assert!(events.get_event("nonexistent").is_none());
    }

    #[test]
    fn get_startup_events_returns_all_startup_events() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new("s1", GlobalEventType::Startup, "s1.lua"));
        events.register(GlobalEvent::new("s2", GlobalEventType::Startup, "s2.lua"));
        events
            .register(GlobalEvent::new("t1", GlobalEventType::Timer, "t1.lua").with_interval(1000));

        let startup = events.get_startup_events();
        assert_eq!(startup.len(), 2);
    }

    #[test]
    fn get_timer_events_returns_events_with_interval() {
        let mut events = GlobalEvents::new();
        events.register(
            GlobalEvent::new("timer1", GlobalEventType::Timer, "t1.lua").with_interval(1000),
        );
        events.register(
            GlobalEvent::new("timer2", GlobalEventType::Timer, "t2.lua").with_interval(2000),
        );
        events.register(GlobalEvent::new(
            "startup",
            GlobalEventType::Startup,
            "s.lua",
        ));

        let timers = events.get_timer_events();
        assert_eq!(timers.len(), 2);
    }

    #[test]
    fn is_due_returns_true_when_elapsed_exceeds_interval() {
        let event = GlobalEvent::new("t", GlobalEventType::Timer, "t.lua").with_interval(1000);
        assert!(event.is_due(0, 1000));
        assert!(event.is_due(0, 2000));
    }

    #[test]
    fn is_due_returns_false_when_elapsed_less_than_interval() {
        let event = GlobalEvent::new("t", GlobalEventType::Timer, "t.lua").with_interval(1000);
        assert!(!event.is_due(0, 999));
        assert!(!event.is_due(500, 1000));
    }

    #[test]
    fn is_due_returns_false_when_no_interval() {
        let event = GlobalEvent::new("s", GlobalEventType::Startup, "s.lua");
        assert!(!event.is_due(0, 99999));
    }

    // ── new tests: onTime ────────────────────────────────────────────────────

    /// onTime fires only when the current time exactly matches the scheduled time.
    #[test]
    fn on_time_fires_at_correct_time() {
        let event = GlobalEvent::new("morning", GlobalEventType::Time, "morning.lua")
            .with_scheduled_time(8, 30, 0);

        // Exact match → fires
        assert!(event.is_time_due(8, 30, 0));
        // Wrong hour
        assert!(!event.is_time_due(9, 30, 0));
        // Wrong minute
        assert!(!event.is_time_due(8, 31, 0));
        // Wrong second
        assert!(!event.is_time_due(8, 30, 1));
        // All wrong
        assert!(!event.is_time_due(0, 0, 0));
    }

    #[test]
    fn on_time_no_scheduled_time_never_fires() {
        // A Time event without a scheduled_time set should never fire.
        let event = GlobalEvent::new("bare", GlobalEventType::Time, "bare.lua");
        assert!(!event.is_time_due(8, 30, 0));
    }

    #[test]
    fn scheduled_time_struct_matches_correctly() {
        let st = ScheduledTime::new(12, 0, 0);
        assert!(st.matches(12, 0, 0));
        assert!(!st.matches(12, 0, 1));
        assert!(!st.matches(11, 59, 59));
    }

    // ── new tests: onThink ───────────────────────────────────────────────────

    /// onThink fires only when elapsed >= interval.
    #[test]
    fn on_think_fires_after_interval() {
        let event =
            GlobalEvent::new("think", GlobalEventType::Think, "think.lua").with_interval(5000);

        // Not yet due
        assert!(!event.is_due(0, 4999));
        // Exactly at interval boundary → due
        assert!(event.is_due(0, 5000));
        // Past interval → due
        assert!(event.is_due(0, 6000));
        // Offset last_fire
        assert!(!event.is_due(1000, 5999));
        assert!(event.is_due(1000, 6000));
    }

    #[test]
    fn on_think_no_interval_never_fires() {
        let event = GlobalEvent::new("think_bare", GlobalEventType::Think, "think.lua");
        assert!(!event.is_due(0, 99999));
    }

    #[test]
    fn get_think_events_returns_only_think_events_with_interval() {
        let mut events = GlobalEvents::new();
        events.register(
            GlobalEvent::new("th1", GlobalEventType::Think, "th1.lua").with_interval(1000),
        );
        events.register(
            GlobalEvent::new("th2", GlobalEventType::Think, "th2.lua").with_interval(2000),
        );
        // Think without interval → excluded
        events.register(GlobalEvent::new(
            "th3_bare",
            GlobalEventType::Think,
            "th3.lua",
        ));
        events.register(GlobalEvent::new("start", GlobalEventType::Startup, "s.lua"));

        let think = events.get_think_events();
        assert_eq!(think.len(), 2);
        assert!(think.iter().all(|e| e.event_type == GlobalEventType::Think));
    }

    // ── new tests: onStartup ─────────────────────────────────────────────────

    /// onStartup fires exactly once.
    #[test]
    fn on_startup_fires_once() {
        let mut event = GlobalEvent::new("startup", GlobalEventType::Startup, "startup.lua");

        // First fire → allowed
        assert!(event.try_fire());
        assert!(event.has_fired());

        // Subsequent fires → denied
        assert!(!event.try_fire());
        assert!(!event.try_fire());
    }

    // ── new tests: onShutdown ────────────────────────────────────────────────

    /// onShutdown fires exactly once.
    #[test]
    fn on_shutdown_fires_once() {
        let mut event = GlobalEvent::new("shutdown", GlobalEventType::Shutdown, "shutdown.lua");

        assert!(event.try_fire());
        assert!(!event.try_fire());
    }

    #[test]
    fn get_shutdown_events_returns_all_shutdown_events() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new(
            "sd1",
            GlobalEventType::Shutdown,
            "sd1.lua",
        ));
        events.register(GlobalEvent::new(
            "sd2",
            GlobalEventType::Shutdown,
            "sd2.lua",
        ));
        events.register(GlobalEvent::new("su1", GlobalEventType::Startup, "su1.lua"));

        assert_eq!(events.get_shutdown_events().len(), 2);
    }

    // ── new tests: onRecord ──────────────────────────────────────────────────

    /// onRecord fires only when player count exceeds the previous record.
    #[test]
    fn on_record_fires_when_new_record() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new(
            "rec",
            GlobalEventType::Record,
            "record.lua",
        ));

        // Initial record is 0; 100 beats it
        assert_eq!(
            events.check_record(100),
            RecordFireResult::NewRecord {
                current: 100,
                old: 0
            }
        );
        assert_eq!(events.player_record(), 100);

        // Same count → not a record
        assert_eq!(events.check_record(100), RecordFireResult::NotARecord);

        // Lower count → not a record
        assert_eq!(events.check_record(50), RecordFireResult::NotARecord);

        // Higher count → new record
        assert_eq!(
            events.check_record(101),
            RecordFireResult::NewRecord {
                current: 101,
                old: 100
            }
        );
        assert_eq!(events.player_record(), 101);
    }

    #[test]
    fn get_record_events_returns_all_record_events() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new("r1", GlobalEventType::Record, "r1.lua"));
        events.register(GlobalEvent::new("r2", GlobalEventType::Record, "r2.lua"));
        events.register(GlobalEvent::new("s1", GlobalEventType::Startup, "s1.lua"));

        assert_eq!(events.get_record_events().len(), 2);
    }

    // ── new tests: onSave ────────────────────────────────────────────────────

    #[test]
    fn save_event_type_exists_and_is_registered() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new("save1", GlobalEventType::Save, "save.lua"));
        let e = events.get_event("save1").unwrap();
        assert_eq!(e.event_type, GlobalEventType::Save);
    }

    #[test]
    fn get_save_events_returns_all_save_events() {
        let mut events = GlobalEvents::new();
        events.register(GlobalEvent::new("sv1", GlobalEventType::Save, "sv1.lua"));
        events.register(GlobalEvent::new("sv2", GlobalEventType::Save, "sv2.lua"));
        events.register(GlobalEvent::new("su1", GlobalEventType::Startup, "su1.lua"));

        assert_eq!(events.get_save_events().len(), 2);
    }

    // ── new tests: script_event_name ─────────────────────────────────────────

    #[test]
    fn script_event_name_returns_correct_lua_callback() {
        let cases = vec![
            (GlobalEventType::Startup, "onStartup"),
            (GlobalEventType::Shutdown, "onShutdown"),
            (GlobalEventType::Record, "onRecord"),
            (GlobalEventType::Save, "onSave"),
            (GlobalEventType::Time, "onTime"),
            (GlobalEventType::Timer, "onTime"),
            (GlobalEventType::Think, "onThink"),
        ];
        for (et, expected) in cases {
            let e = GlobalEvent::new("x", et, "x.lua");
            assert_eq!(e.script_event_name(), expected);
        }
    }

    // ── new tests: register_multiple_events_of_same_type ─────────────────────

    /// Both events fire independently — registering two events of the same type
    /// does not cause one to suppress the other.
    #[test]
    fn register_multiple_events_of_same_type() {
        let mut events = GlobalEvents::new();
        events.register(
            GlobalEvent::new("think_a", GlobalEventType::Think, "a.lua").with_interval(1000),
        );
        events.register(
            GlobalEvent::new("think_b", GlobalEventType::Think, "b.lua").with_interval(2000),
        );

        // Both are present and independently queryable
        let a = events.get_event("think_a").unwrap();
        let b = events.get_event("think_b").unwrap();

        assert_eq!(a.interval_ms, Some(1000));
        assert_eq!(b.interval_ms, Some(2000));

        // a is due at t=1000, b is not
        assert!(a.is_due(0, 1000));
        assert!(!b.is_due(0, 1000));

        // b is due at t=2000, a was already due earlier
        assert!(a.is_due(0, 2000));
        assert!(b.is_due(0, 2000));

        // Both show up in get_think_events
        assert_eq!(events.get_think_events().len(), 2);
    }

    #[test]
    fn register_multiple_time_events_of_same_type() {
        let mut events = GlobalEvents::new();
        events.register(
            GlobalEvent::new("morning", GlobalEventType::Time, "morning.lua")
                .with_scheduled_time(8, 0, 0),
        );
        events.register(
            GlobalEvent::new("noon", GlobalEventType::Time, "noon.lua")
                .with_scheduled_time(12, 0, 0),
        );

        let morning = events.get_event("morning").unwrap();
        let noon = events.get_event("noon").unwrap();

        // Fires independently at different times
        assert!(morning.is_time_due(8, 0, 0));
        assert!(!morning.is_time_due(12, 0, 0));
        assert!(noon.is_time_due(12, 0, 0));
        assert!(!noon.is_time_due(8, 0, 0));

        assert_eq!(events.get_time_events().len(), 2);
    }

    // ── new tests: think vs time separation ──────────────────────────────────

    /// Think events (interval-based) do not appear in get_time_events.
    #[test]
    fn think_events_not_in_time_events() {
        let mut events = GlobalEvents::new();
        events.register(
            GlobalEvent::new("think1", GlobalEventType::Think, "think1.lua").with_interval(1000),
        );
        events.register(
            GlobalEvent::new("time1", GlobalEventType::Time, "time1.lua")
                .with_scheduled_time(10, 0, 0),
        );

        assert_eq!(events.get_time_events().len(), 1);
        assert_eq!(events.get_think_events().len(), 1);
    }

    // ── new tests: try_fire for non-once events ───────────────────────────────

    /// Think, Time, Record, Save events are not once-only and always return true from try_fire.
    #[test]
    fn non_once_events_always_fire() {
        let repeating_types = vec![
            GlobalEventType::Think,
            GlobalEventType::Time,
            GlobalEventType::Record,
            GlobalEventType::Save,
            GlobalEventType::Timer,
        ];
        for et in repeating_types {
            let mut e = GlobalEvent::new("x", et, "x.lua");
            assert!(
                e.try_fire(),
                "Expected try_fire to return true for repeating event"
            );
            assert!(
                e.try_fire(),
                "Expected try_fire to return true on second call for repeating event"
            );
        }
    }

    // ── new tests: get_time_events with Timer alias ───────────────────────────

    #[test]
    fn get_time_events_includes_timer_alias_with_scheduled_time() {
        let mut events = GlobalEvents::new();
        // Legacy Timer type with scheduled_time should appear in time events
        events.register(
            GlobalEvent::new("legacy_timer", GlobalEventType::Timer, "t.lua")
                .with_scheduled_time(6, 0, 0),
        );

        assert_eq!(events.get_time_events().len(), 1);
    }

    // ── new tests: player_record initial state ────────────────────────────────

    #[test]
    fn player_record_starts_at_zero() {
        let events = GlobalEvents::new();
        assert_eq!(events.player_record(), 0);
    }

    #[test]
    fn check_record_zero_count_not_a_record() {
        let mut events = GlobalEvents::new();
        // 0 does not beat 0
        assert_eq!(events.check_record(0), RecordFireResult::NotARecord);
    }

    // ── new tests: get_event_mut ─────────────────────────────────────────────

    /// `get_event_mut` returns `None` for an unknown name and `Some` for a registered
    /// event, allowing in-place mutation of fields (mirrors C++ map lookup semantics
    /// that would let a caller alter `nextExecution` / `interval` on a registered event).
    #[test]
    fn get_event_mut_returns_some_for_registered_and_allows_mutation() {
        let mut events = GlobalEvents::new();
        events
            .register(GlobalEvent::new("mut", GlobalEventType::Think, "m.lua").with_interval(1000));

        // None for unknown
        assert!(events.get_event_mut("nope").is_none());

        // Some for registered — mutation is observable through subsequent lookup
        {
            let e = events.get_event_mut("mut").expect("event should exist");
            e.interval_ms = Some(2500);
        }
        let e = events.get_event("mut").unwrap();
        assert_eq!(e.interval_ms, Some(2500));
    }
}
