use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Minimum delay in milliseconds, matching C++ SCHEDULER_MINTICKS = 50.
pub const SCHEDULER_MINTICKS: u64 = 50;

pub struct ScheduledEvent {
    pub id: u64,
    pub fire_at_ms: u64,
    pub callback: Box<dyn FnOnce() + Send>,
}

pub struct Scheduler {
    /// Next event id to assign. IDs start at 1 (matches C++ pre-increment `++lastEventId`).
    next_id: u64,
    events: BinaryHeap<Reverse<(u64, u64)>>, // (fire_at_ms, id)
    pending: HashMap<u64, ScheduledEvent>,
    current_time_ms: u64,
    cancelled: HashSet<u64>,
    /// When true, `add_event` is a no-op (matches C++ THREAD_STATE_TERMINATED check).
    stopped: bool,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            events: BinaryHeap::new(),
            pending: HashMap::new(),
            current_time_ms: 0,
            cancelled: HashSet::new(),
            stopped: false,
        }
    }

    /// Enqueue a task to fire after `delay_ms` milliseconds.
    ///
    /// If the scheduler is stopped, the call is ignored and returns 0.
    /// Delays below `SCHEDULER_MINTICKS` are clamped to `SCHEDULER_MINTICKS`,
    /// matching the C++ minimum-tick enforcement.
    pub fn add_event(&mut self, delay_ms: u64, callback: Box<dyn FnOnce() + Send>) -> u64 {
        if self.stopped {
            return 0;
        }
        let effective_delay = delay_ms.max(SCHEDULER_MINTICKS);
        let id = self.next_id;
        self.next_id += 1;
        let fire_at_ms = self.current_time_ms + effective_delay;
        let event = ScheduledEvent {
            id,
            fire_at_ms,
            callback,
        };
        self.events.push(Reverse((fire_at_ms, id)));
        self.pending.insert(id, event);
        id
    }

    /// Cancel a pending event by id.  Matches C++ `stopEvent(eventId)`.
    /// Cancelling id 0 is a no-op (C++ guard: `if (eventId == 0) return`).
    pub fn stop_event(&mut self, event_id: u64) {
        if event_id == 0 {
            return;
        }
        self.cancelled.insert(event_id);
        self.pending.remove(&event_id);
    }

    /// Alias for `stop_event` kept for backward compat with earlier Rust code.
    pub fn cancel_event(&mut self, event_id: u64) {
        self.stop_event(event_id);
    }

    /// Stop the scheduler. After this, `add_event` is ignored.
    /// Matches C++ `Scheduler::shutdown()` / `setState(THREAD_STATE_TERMINATED)`.
    pub fn stop(&mut self) {
        self.stopped = true;
        // Cancel all pending events
        self.pending.clear();
        self.cancelled.clear();
        while self.events.pop().is_some() {}
    }

    /// Returns true if the scheduler has been stopped.
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Returns the number of milliseconds until the next event fires,
    /// or `None` if there are no pending events.
    /// Matches the spirit of C++ `getNextEventTime()`.
    pub fn get_next_event_time(&self) -> Option<u64> {
        // Peek at the heap; skip cancelled entries to find the first real one.
        // Because we eagerly remove from `pending` on cancel, we check pending membership.
        for Reverse((fire_at_ms, id)) in self.events.iter() {
            if self.pending.contains_key(id) {
                return Some(fire_at_ms.saturating_sub(self.current_time_ms));
            }
        }
        None
    }

    /// Advance time to `now_ms` and execute all callbacks whose deadline has passed.
    /// Events are fired in chronological order (earliest deadline first).
    pub fn tick(&mut self, now_ms: u64) {
        self.current_time_ms = now_ms;
        // Collect ids to fire in order
        let mut to_fire: Vec<(u64, u64)> = Vec::new();
        while let Some(Reverse((fire_at_ms, id))) = self.events.peek() {
            if *fire_at_ms <= now_ms {
                let fire_at_ms = *fire_at_ms;
                let id = *id;
                self.events.pop();
                if !self.cancelled.contains(&id) {
                    to_fire.push((fire_at_ms, id));
                }
            } else {
                break;
            }
        }
        // Sort by (fire_at_ms, id) to ensure chronological order
        to_fire.sort_by_key(|&(fire_at_ms, id)| (fire_at_ms, id));
        for (_, id) in to_fire {
            // Invariant: every id in `to_fire` was just popped from `events` and
            // passed the `cancelled` filter; `stop_event` (the only way to drop
            // from `pending`) also inserts into `cancelled`, so the entry is
            // guaranteed to be in `pending` here.
            let event = self
                .pending
                .remove(&id)
                .expect("to_fire id must be in pending");
            (event.callback)();
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SchedulerTask wrapper + factory (Session 24 ledger closure)
// ---------------------------------------------------------------------------

/// Pre-built task that mirrors the C++ `SchedulerTask` class.
///
/// In C++ callers build a task with `createSchedulerTask(delay, lambda)`
/// then pass the pointer to `g_scheduler.addEvent(task)`. The Rust port
/// previously took `(delay_ms, Box<dyn FnOnce>)` arguments directly via
/// `Scheduler::add_event`; this struct restores the C++ surface so
/// migrated call-sites read 1:1 with the original code.
///
/// `event_id` mirrors C++ `eventId`: it's 0 before the task has been
/// scheduled, and is overwritten with the real id when `add_event_task`
/// consumes the wrapper.
pub struct SchedulerTask {
    pub delay_ms: u64,
    pub event_id: u64,
    pub callback: Box<dyn FnOnce() + Send>,
}

impl SchedulerTask {
    /// Returns the assigned event id, or 0 when the task hasn't yet been
    /// passed to `Scheduler::add_event_task`. Mirrors C++ `getEventId()`.
    pub fn get_event_id(&self) -> u64 {
        self.event_id
    }

    /// Mirrors C++ `setEventId(uint32_t)` — used by tests and by
    /// `Scheduler::add_event_task` when it assigns a new id.
    pub fn set_event_id(&mut self, id: u64) {
        self.event_id = id;
    }

    /// Mirrors C++ `getDelay()`.
    pub fn get_delay(&self) -> u64 {
        self.delay_ms
    }
}

/// Factory mirroring C++ `createSchedulerTask(delay, std::move(f))`.
/// Returns a heap-allocated `SchedulerTask` (via `Box`) so the API
/// preserves the C++ "pointer ownership" contract — `Scheduler::add_event_task`
/// consumes it.
pub fn create_scheduler_task(delay_ms: u64, callback: Box<dyn FnOnce() + Send>) -> SchedulerTask {
    SchedulerTask {
        delay_ms,
        event_id: 0,
        callback,
    }
}

impl Scheduler {
    /// Mirrors C++ `Scheduler::addEvent(SchedulerTask*)`. Consumes the
    /// task wrapper, assigns it an event id (if not already set), and
    /// schedules the callback. Returns the assigned event id so callers
    /// can later `stop_event(id)`.
    ///
    /// The id-assignment branch mirrors the C++ behaviour:
    ///   * `task.event_id == 0` → assign a fresh id from `next_id`
    ///   * `task.event_id != 0` → keep the caller-provided id (used by
    ///     C++ replay paths that pre-assign ids)
    pub fn add_event_task(&mut self, mut task: SchedulerTask) -> u64 {
        if self.stopped {
            return 0;
        }
        if task.event_id == 0 {
            task.event_id = self.next_id;
            self.next_id += 1;
        } else {
            // Keep `next_id` ahead of any caller-provided id so future
            // auto-assigned ids don't collide.
            if task.event_id >= self.next_id {
                self.next_id = task.event_id + 1;
            }
        }
        let id = task.event_id;
        let effective_delay = task.delay_ms.max(SCHEDULER_MINTICKS);
        let fire_at_ms = self.current_time_ms + effective_delay;
        let event = ScheduledEvent {
            id,
            fire_at_ms,
            callback: task.callback,
        };
        self.events.push(Reverse((fire_at_ms, id)));
        self.pending.insert(id, event);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Tiny helper that yields a *no-op* boxed callback. Encapsulating this in a
    /// single function ensures llvm-cov sees exactly one closure-body region for
    /// "no-op" callbacks; every test that needs a no-op callback for an event
    /// that will eventually be fired (or for an event the test doesn't care
    /// about firing) shares this region, and the helper itself is exercised by
    /// `noop_callback_helper_is_callable`.
    fn noop_callback() -> Box<dyn FnOnce() + Send> {
        Box::new(|| {})
    }

    /// Sanity check that the `noop_callback()` helper produces an invokable
    /// closure — this is what makes the helper's closure-body region covered.
    /// All test sites that want a "no-op event" delegate to this helper instead
    /// of declaring a fresh `Box::new(|| {})`, so the closure body is only
    /// instantiated once.
    #[test]
    fn noop_callback_helper_is_callable() {
        (noop_callback())();
    }

    // ── existing tests (unchanged) ────────────────────────────────────────────

    #[test]
    fn add_event_increases_pending_count() {
        let mut sched = Scheduler::new();
        sched.add_event(100, noop_callback());
        assert_eq!(sched.pending_count(), 1);
        // Fire the callback to ensure its body region is exercised, then verify
        // pending_count returns to zero.
        sched.tick(100);
        assert_eq!(sched.pending_count(), 0);
    }

    #[test]
    fn tick_before_fire_time_does_not_fire() {
        let fired = Arc::new(Mutex::new(false));
        let fired_clone = fired.clone();
        let mut sched = Scheduler::new();
        sched.add_event(
            100,
            Box::new(move || {
                *fired_clone.lock().unwrap() = true;
            }),
        );
        sched.tick(50);
        assert!(!*fired.lock().unwrap());
        assert_eq!(sched.pending_count(), 1);
        // Fire to ensure the closure body region is exercised at some point.
        sched.tick(100);
        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn tick_at_fire_time_fires_callback() {
        let fired = Arc::new(Mutex::new(false));
        let fired_clone = fired.clone();
        let mut sched = Scheduler::new();
        sched.add_event(
            100,
            Box::new(move || {
                *fired_clone.lock().unwrap() = true;
            }),
        );
        sched.tick(100);
        assert!(*fired.lock().unwrap());
        assert_eq!(sched.pending_count(), 0);
    }

    #[test]
    fn cancel_event_prevents_callback_from_firing() {
        let fired = Arc::new(Mutex::new(0u32));
        let f_survivor = fired.clone();
        let mut sched = Scheduler::new();
        // Schedule two events; cancel one (with an empty closure, since its body
        // would otherwise be dead code) and let the other fire to prove the
        // un-cancelled event still runs.
        let id_cancelled = sched.add_event(100, noop_callback());
        let _id_alive = sched.add_event(
            100,
            Box::new(move || {
                *f_survivor.lock().unwrap() += 1;
            }),
        );
        sched.cancel_event(id_cancelled);
        sched.tick(200);
        // Only the un-cancelled event should have fired.
        assert_eq!(*fired.lock().unwrap(), 1);
    }

    #[test]
    fn multiple_events_fire_in_chronological_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut sched = Scheduler::new();

        let o1 = order.clone();
        let o2 = order.clone();
        let o3 = order.clone();

        sched.add_event(
            300,
            Box::new(move || {
                o3.lock().unwrap().push(300u64);
            }),
        );
        sched.add_event(
            100,
            Box::new(move || {
                o1.lock().unwrap().push(100u64);
            }),
        );
        sched.add_event(
            200,
            Box::new(move || {
                o2.lock().unwrap().push(200u64);
            }),
        );

        sched.tick(300);

        let result = order.lock().unwrap().clone();
        assert_eq!(result, vec![100, 200, 300]);
    }

    #[test]
    fn two_events_at_same_time_both_fire() {
        let count = Arc::new(Mutex::new(0u32));
        let c1 = count.clone();
        let c2 = count.clone();
        let mut sched = Scheduler::new();
        sched.add_event(
            50,
            Box::new(move || {
                *c1.lock().unwrap() += 1;
            }),
        );
        sched.add_event(
            50,
            Box::new(move || {
                *c2.lock().unwrap() += 1;
            }),
        );
        sched.tick(50);
        assert_eq!(*count.lock().unwrap(), 2);
        assert_eq!(sched.pending_count(), 0);
    }

    #[test]
    fn pending_count_is_zero_after_all_fired() {
        let mut sched = Scheduler::new();
        // Use delays >= SCHEDULER_MINTICKS (50) so they are not clamped.
        sched.add_event(50, noop_callback());
        sched.add_event(100, noop_callback());
        sched.tick(100);
        assert_eq!(sched.pending_count(), 0);
    }

    // ── new tests ─────────────────────────────────────────────────────────────

    /// Each call to `add_event` returns a unique, strictly increasing ID starting at 1.
    /// Matches C++ pre-increment `++lastEventId` (first ID is 1, not 0).
    #[test]
    fn add_event_returns_unique_increasing_id() {
        let counter = Arc::new(Mutex::new(0u32));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();
        let mut sched = Scheduler::new();
        let id1 = sched.add_event(
            100,
            Box::new(move || {
                *c1.lock().unwrap() += 1;
            }),
        );
        let id2 = sched.add_event(
            100,
            Box::new(move || {
                *c2.lock().unwrap() += 1;
            }),
        );
        let id3 = sched.add_event(
            100,
            Box::new(move || {
                *c3.lock().unwrap() += 1;
            }),
        );
        assert!(id1 >= 1, "first id must be >= 1 (C++ starts at 1)");
        assert!(id2 > id1, "ids must be strictly increasing");
        assert!(id3 > id2, "ids must be strictly increasing");
        // Fire all closures to exercise their bodies.
        sched.tick(100);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    /// `stop_event(id)` prevents the callback from firing — matches C++ `stopEvent`.
    #[test]
    fn stop_event_prevents_fire() {
        let count = Arc::new(Mutex::new(0u32));
        let c_alive = count.clone();
        let mut sched = Scheduler::new();
        // The cancelled event uses an empty closure (its body would be dead code
        // since stop_event removes it from pending before tick).
        let id_stopped = sched.add_event(100, noop_callback());
        let _id_alive = sched.add_event(
            100,
            Box::new(move || {
                *c_alive.lock().unwrap() += 1;
            }),
        );
        sched.stop_event(id_stopped);
        sched.tick(200);
        // Only the surviving event should have fired, exercising its closure body.
        assert_eq!(
            *count.lock().unwrap(),
            1,
            "stop_event must prevent the cancelled callback from firing"
        );
    }

    /// `stop_event(0)` is a no-op (C++ guard: `if (eventId == 0) return`).
    #[test]
    fn stop_event_zero_is_noop() {
        let fired = Arc::new(Mutex::new(false));
        let fc = fired.clone();
        let mut sched = Scheduler::new();
        sched.add_event(
            100,
            Box::new(move || {
                *fc.lock().unwrap() = true;
            }),
        );
        // Must not panic or corrupt state
        sched.stop_event(0);
        assert_eq!(sched.pending_count(), 1);
        // Fire to exercise the closure body and prove the event survived stop_event(0).
        sched.tick(100);
        assert!(*fired.lock().unwrap());
    }

    /// Events with earlier deadlines are returned (fired) before events with later deadlines.
    #[test]
    fn events_ordered_by_deadline() {
        let order = Arc::new(Mutex::new(Vec::<u32>::new()));
        let mut sched = Scheduler::new();

        let o_late = order.clone();
        let o_early = order.clone();

        // Add late event first, then early event — queue must reorder them.
        sched.add_event(
            200,
            Box::new(move || {
                o_late.lock().unwrap().push(2);
            }),
        );
        sched.add_event(
            100,
            Box::new(move || {
                o_early.lock().unwrap().push(1);
            }),
        );

        sched.tick(300);

        assert_eq!(
            *order.lock().unwrap(),
            vec![1, 2],
            "earlier deadline must fire first"
        );
    }

    /// `get_next_event_time()` returns `None` when the scheduler has no pending events.
    #[test]
    fn get_next_event_time_empty() {
        let sched = Scheduler::new();
        assert_eq!(sched.get_next_event_time(), None);
    }

    /// `get_next_event_time()` returns the delay to the nearest pending event.
    #[test]
    fn get_next_event_time_returns_nearest() {
        let count = Arc::new(Mutex::new(0u32));
        let c1 = count.clone();
        let c2 = count.clone();
        let mut sched = Scheduler::new();
        // current_time_ms is 0; add events at delays 200 and 100 → nearest is 100
        sched.add_event(
            200,
            Box::new(move || {
                *c1.lock().unwrap() += 1;
            }),
        );
        sched.add_event(
            100,
            Box::new(move || {
                *c2.lock().unwrap() += 1;
            }),
        );
        let t = sched.get_next_event_time();
        assert_eq!(t, Some(100), "nearest event is 100 ms away");
        // Fire both events to exercise their closure bodies.
        sched.tick(200);
        assert_eq!(*count.lock().unwrap(), 2);
    }

    /// `tick()` returns (fires) only events whose deadline is <= now; future events stay pending.
    #[test]
    fn tick_returns_ready_events_skips_future() {
        let fired_early = Arc::new(Mutex::new(false));
        let fired_late = Arc::new(Mutex::new(false));
        let fe = fired_early.clone();
        let fl = fired_late.clone();
        let mut sched = Scheduler::new();

        sched.add_event(
            50,
            Box::new(move || {
                *fe.lock().unwrap() = true;
            }),
        );
        sched.add_event(
            200,
            Box::new(move || {
                *fl.lock().unwrap() = true;
            }),
        );

        // Advance to 50 ms — only the first event should fire
        sched.tick(50);

        assert!(
            *fired_early.lock().unwrap(),
            "event at deadline 50 must fire at tick(50)"
        );
        assert!(
            !*fired_late.lock().unwrap(),
            "event at deadline 200 must NOT fire at tick(50)"
        );
        assert_eq!(sched.pending_count(), 1, "late event still pending");

        // Advance further to fire the late event, exercising its closure body region.
        sched.tick(200);
        assert!(
            *fired_late.lock().unwrap(),
            "late event must fire after its deadline is reached"
        );
        assert_eq!(sched.pending_count(), 0);
    }

    /// After `stop()`, calls to `add_event` are silently ignored and return 0.
    #[test]
    fn add_event_after_stop_is_ignored() {
        let mut sched = Scheduler::new();
        sched.stop();

        // Use an empty closure: no body region to cover. The point of this test
        // is that `add_event` returns 0 and queues nothing after `stop`.
        let id = sched.add_event(50, noop_callback());

        assert_eq!(id, 0, "add_event after stop must return 0");
        assert_eq!(
            sched.pending_count(),
            0,
            "no events must be queued after stop"
        );
        sched.tick(1000);
        // pending_count remains zero — proves the closure was never queued.
        assert_eq!(sched.pending_count(), 0);
    }

    /// `stop()` cancels all currently pending events.
    #[test]
    fn stop_cancels_all_pending_events() {
        // Use empty closures: the test asserts that pending events are cleared
        // by stop(), not that any specific callback ran (closures cancelled by
        // stop are never invoked, so their bodies would otherwise be dead code).
        let mut sched = Scheduler::new();
        sched.add_event(100, noop_callback());
        sched.add_event(200, noop_callback());
        assert_eq!(sched.pending_count(), 2);
        sched.stop();
        assert_eq!(
            sched.pending_count(),
            0,
            "stop must clear all pending events"
        );
        sched.tick(1000); // safe to tick after stop — no pending events
        assert_eq!(sched.pending_count(), 0);
    }

    /// Delays below SCHEDULER_MINTICKS are clamped to SCHEDULER_MINTICKS.
    #[test]
    fn delay_below_minticks_is_clamped() {
        let fired = Arc::new(Mutex::new(false));
        let fc = fired.clone();
        let mut sched = Scheduler::new();
        // Request delay of 10 ms, which is below SCHEDULER_MINTICKS (50).
        sched.add_event(
            10,
            Box::new(move || {
                *fc.lock().unwrap() = true;
            }),
        );
        sched.tick(10); // should NOT fire — clamped to 50
        assert!(
            !*fired.lock().unwrap(),
            "clamped event must not fire before SCHEDULER_MINTICKS"
        );
        sched.tick(50); // should fire now
        assert!(
            *fired.lock().unwrap(),
            "clamped event must fire at SCHEDULER_MINTICKS"
        );
    }

    /// `is_stopped()` reflects the stopped state correctly.
    #[test]
    fn is_stopped_reflects_state() {
        let mut sched = Scheduler::new();
        assert!(!sched.is_stopped());
        sched.stop();
        assert!(sched.is_stopped());
    }

    /// `Scheduler::default()` yields a fresh scheduler equivalent to `Scheduler::new()`.
    /// Exercises the `impl Default for Scheduler` block (matches the spirit of C++ default
    /// construction of the `Scheduler` aggregate / `g_scheduler` global).
    #[test]
    fn default_constructor_matches_new() {
        let sched: Scheduler = Scheduler::default();
        assert!(!sched.is_stopped());
        assert_eq!(sched.pending_count(), 0);
        assert_eq!(sched.get_next_event_time(), None);
    }

    /// `get_next_event_time()` skips cancelled entries that remain in the heap and
    /// returns the delay to the next *live* event. Exercises the loop-continue branch
    /// where `pending.contains_key(id)` is false (the entry was eagerly removed from
    /// `pending` by `stop_event` but still sits in the binary heap).
    #[test]
    fn get_next_event_time_skips_cancelled_entry() {
        let count = Arc::new(Mutex::new(0u32));
        let c_live = count.clone();
        let mut sched = Scheduler::new();
        // Add a near event (will be cancelled) and a later live event.
        let id_near = sched.add_event(50, noop_callback());
        sched.add_event(
            200,
            Box::new(move || {
                *c_live.lock().unwrap() += 1;
            }),
        );
        // Cancel the near event — it remains in the heap but is no longer in pending.
        sched.stop_event(id_near);
        // The next live event is the one at 200 ms.
        assert_eq!(sched.get_next_event_time(), Some(200));
        // Confirm only the live event fires when we tick past it.
        sched.tick(200);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    // ── SchedulerTask wrapper + factory (Session 24) ────────────────────

    /// `create_scheduler_task` returns a wrapper with `event_id == 0`
    /// (matches C++ `SchedulerTask::eventId = 0` default).
    #[test]
    fn create_scheduler_task_starts_with_event_id_zero() {
        let task = create_scheduler_task(123, noop_callback());
        assert_eq!(task.get_event_id(), 0);
        assert_eq!(task.get_delay(), 123);
    }

    /// `set_event_id` round-trips the assigned id.
    #[test]
    fn scheduler_task_set_event_id_round_trips() {
        let mut task = create_scheduler_task(50, noop_callback());
        task.set_event_id(42);
        assert_eq!(task.get_event_id(), 42);
    }

    /// `add_event_task` with id=0 assigns a fresh id, schedules the
    /// callback, and fires it on tick.
    #[test]
    fn add_event_task_assigns_id_and_fires_callback() {
        let count = Arc::new(Mutex::new(0u32));
        let c = count.clone();
        let mut sched = Scheduler::new();
        let task = create_scheduler_task(
            100,
            Box::new(move || {
                *c.lock().unwrap() += 1;
            }),
        );
        let id = sched.add_event_task(task);
        assert!(id > 0, "fresh id must be non-zero");
        assert_eq!(sched.pending_count(), 1);
        sched.tick(100);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    /// `add_event_task` with a caller-provided non-zero id keeps that
    /// id and bumps `next_id` past it so future auto-assignments don't
    /// collide.
    #[test]
    fn add_event_task_preserves_caller_id_and_bumps_next_id() {
        let mut sched = Scheduler::new();
        let mut task = create_scheduler_task(100, noop_callback());
        task.set_event_id(500);
        let id = sched.add_event_task(task);
        assert_eq!(id, 500, "caller-provided id must be preserved");
        // The next auto-assigned id should be 501, not the original next_id.
        let next_id = sched.add_event(50, noop_callback());
        assert_eq!(next_id, 501);
    }

    /// When the scheduler is stopped, `add_event_task` is a no-op and
    /// returns 0 (matches the `add_event` shutdown contract).
    #[test]
    fn add_event_task_no_op_after_stop() {
        let mut sched = Scheduler::new();
        sched.stop();
        let task = create_scheduler_task(100, noop_callback());
        assert_eq!(sched.add_event_task(task), 0);
        assert_eq!(sched.pending_count(), 0);
    }

    /// `SchedulerTask` honours the `SCHEDULER_MINTICKS` floor — a task
    /// scheduled with delay < 50 ms still fires only after 50 ms.
    #[test]
    fn add_event_task_clamps_delay_to_min_ticks() {
        let count = Arc::new(Mutex::new(0u32));
        let c = count.clone();
        let mut sched = Scheduler::new();
        let task = create_scheduler_task(
            10, // below SCHEDULER_MINTICKS (50)
            Box::new(move || {
                *c.lock().unwrap() += 1;
            }),
        );
        sched.add_event_task(task);
        // Tick at 49 ms (below min-ticks floor) — task must not fire.
        sched.tick(49);
        assert_eq!(*count.lock().unwrap(), 0);
        // Tick at the floor — task fires.
        sched.tick(50);
        assert_eq!(*count.lock().unwrap(), 1);
    }
}
