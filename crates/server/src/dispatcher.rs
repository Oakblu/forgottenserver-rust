// Rust port of `tasks.cpp` / `tasks.h` (C++ `Dispatcher` + `Task`).
//
// The C++ implementation runs a dedicated worker thread with a mutex/condvar
// over a `std::vector<Task*>`.  The Rust port collapses that into a
// single-threaded queue plus an explicit `flush()` step, because the rest of
// the Rust server is event-loop driven rather than threaded.  Apart from the
// threading shim, every observable behaviour of the C++ `Dispatcher` /
// `Task` is preserved:
//
//   * `addTask` is a no-op after `shutdown` (C++ `THREAD_STATE_TERMINATED`
//     check in `addTask`).
//   * Tasks execute in FIFO order (C++ `taskList.swap` then iterate).
//   * The dispatcher cycle counter increments once per *non-expired* executed
//     task (C++ `++dispatcherCycle` inside `threadMain`).
//   * Tasks can carry an expiration deadline; expired tasks are dropped
//     without execution (C++ `if (!task->hasExpired())`).
//   * `Task::set_dont_expire` resets the deadline to "never expire"
//     (C++ `SYSTEM_TIME_ZERO`).
//   * Task execution is panic-safe: a panicking task does not poison the
//     dispatcher and subsequent tasks still run (C++ relies on the OS to
//     terminate; Rust catches the panic so the rest of the queue drains).

use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

/// Default expiration applied by helpers such as `add_task_with_expiration_default`.
/// Matches C++ `DISPATCHER_TASK_EXPIRATION = 2000`.
pub const DISPATCHER_TASK_EXPIRATION: u32 = 2000;

/// Boxed task closure.  Matches C++ `using TaskFunc = std::function<void(void)>`.
pub type TaskFunc = Box<dyn FnOnce() + Send>;

/// A unit of work enqueued onto the dispatcher.
///
/// Equivalent to the C++ `Task` class.  `expiration = None` corresponds to
/// `SYSTEM_TIME_ZERO` ("never expires"); `expiration = Some(deadline)` means
/// the task will be dropped without executing if the deadline has passed at
/// the moment the dispatcher would invoke it.
pub struct Task {
    func: TaskFunc,
    /// Deadline after which the task is considered expired.  `None` means
    /// "never expire" and matches C++ `SYSTEM_TIME_ZERO`.
    expiration: Option<Instant>,
}

impl Task {
    /// Build a task with no expiration.  Mirrors C++ `Task(TaskFunc&& f)`.
    pub fn new(func: TaskFunc) -> Self {
        Self {
            func,
            expiration: None,
        }
    }

    /// Build a task that expires `ms` milliseconds from now.
    /// Mirrors C++ `Task(uint32_t ms, TaskFunc&& f)`.
    pub fn with_expiration(ms: u32, func: TaskFunc) -> Self {
        let expiration = Instant::now()
            .checked_add(Duration::from_millis(ms as u64))
            .expect("expiration deadline overflowed Instant");
        Self {
            func,
            expiration: Some(expiration),
        }
    }

    /// Reset the deadline so this task never expires.
    /// Mirrors C++ `Task::setDontExpire()`.
    pub fn set_dont_expire(&mut self) {
        self.expiration = None;
    }

    /// `true` once the deadline has passed.  Tasks without a deadline never
    /// expire.  Mirrors C++ `Task::hasExpired() const`.
    pub fn has_expired(&self) -> bool {
        match self.expiration {
            None => false,
            Some(deadline) => deadline < Instant::now(),
        }
    }

    /// Consume the task and run its closure.  Mirrors C++ `Task::operator()()`.
    pub fn run(self) {
        (self.func)();
    }
}

/// Build a non-expiring task.  Mirrors free function C++ `createTask(TaskFunc&&)`.
pub fn create_task(func: TaskFunc) -> Task {
    Task::new(func)
}

/// Build an expiring task.  Mirrors free function C++ `createTask(uint32_t, TaskFunc&&)`.
pub fn create_task_with_expiration(ms: u32, func: TaskFunc) -> Task {
    Task::with_expiration(ms, func)
}

/// Single-threaded dispatcher.  See module docs for the relationship to the
/// C++ `Dispatcher` class.
pub struct Dispatcher {
    queue: VecDeque<Task>,
    stopped: bool,
    /// Counts non-expired tasks executed by `flush`.  Matches the C++
    /// `dispatcherCycle` counter exposed via `getDispatcherCycle()`.
    dispatcher_cycle: u64,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            stopped: false,
            dispatcher_cycle: 0,
        }
    }

    /// Enqueue a pre-built `Task`.  Matches C++ `Dispatcher::addTask(Task*)`.
    /// After `stop()` (C++ `shutdown()`), the task is silently dropped.
    pub fn add_task(&mut self, task: Task) {
        if !self.stopped {
            self.queue.push_back(task);
        }
    }

    /// Convenience: enqueue a closure without an expiration.
    /// Matches C++ inline `addTask(TaskFunc&&)`.
    pub fn add_task_fn(&mut self, func: TaskFunc) {
        self.add_task(Task::new(func));
    }

    /// Convenience: enqueue a closure with an expiration in milliseconds.
    /// Matches C++ inline `addTask(uint32_t expiration, TaskFunc&&)`.
    pub fn add_task_with_expiration(&mut self, ms: u32, func: TaskFunc) {
        self.add_task(Task::with_expiration(ms, func));
    }

    /// Drain the queue, executing every non-expired task in FIFO order.
    /// Matches the body of C++ `Dispatcher::threadMain()`'s inner loop:
    /// swap the task list, then iterate; skip expired tasks; otherwise
    /// increment `dispatcherCycle` and invoke the task.  Tasks that panic
    /// do not abort the drain — the remaining tasks still execute.
    pub fn flush(&mut self) {
        while let Some(task) = self.queue.pop_front() {
            if task.has_expired() {
                // Drop without executing; matches C++ `if (!task->hasExpired())`.
                continue;
            }
            self.dispatcher_cycle += 1;
            let _ = catch_unwind(AssertUnwindSafe(move || task.run()));
        }
    }

    /// Mark the dispatcher as stopped so further `add_task` calls are ignored.
    /// Mirrors C++ `Dispatcher::shutdown()` (the C++ version enqueues a task
    /// that flips the thread state; in the single-threaded Rust port we just
    /// flip the flag directly).
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    /// `true` after `stop()` has been called.
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Number of tasks currently waiting in the queue.
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Cumulative non-expired tasks executed by `flush`.  Matches
    /// C++ `Dispatcher::getDispatcherCycle()`.
    pub fn get_dispatcher_cycle(&self) -> u64 {
        self.dispatcher_cycle
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    /// Returns a do-nothing closure.  Sharing this builder keeps the closure
    /// *body* line covered by tests that actually run a no-op task; tests
    /// that only need a placeholder closure (e.g. to fill the queue and
    /// observe that adds are ignored after `stop`) can call it without
    /// inflating their own line counts.
    fn noop_task() -> TaskFunc {
        Box::new(|| {})
    }

    #[test]
    fn add_task_increments_pending_count() {
        let ran = Arc::new(Mutex::new(0u32));
        let r1 = ran.clone();
        let r2 = ran.clone();
        let mut d = Dispatcher::new();
        assert_eq!(d.pending_count(), 0);
        d.add_task_fn(Box::new(move || {
            *r1.lock().unwrap() += 1;
        }));
        assert_eq!(d.pending_count(), 1);
        d.add_task_fn(Box::new(move || {
            *r2.lock().unwrap() += 1;
        }));
        assert_eq!(d.pending_count(), 2);
        // Drain so coverage tooling marks the closure bodies as executed.
        d.flush();
        assert_eq!(*ran.lock().unwrap(), 2);
    }

    #[test]
    fn flush_executes_tasks_in_fifo_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut d = Dispatcher::new();

        let o1 = order.clone();
        let o2 = order.clone();
        let o3 = order.clone();

        d.add_task_fn(Box::new(move || {
            o1.lock().unwrap().push(1u64);
        }));
        d.add_task_fn(Box::new(move || {
            o2.lock().unwrap().push(2u64);
        }));
        d.add_task_fn(Box::new(move || {
            o3.lock().unwrap().push(3u64);
        }));

        d.flush();

        let result = order.lock().unwrap().clone();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn flush_leaves_pending_count_zero() {
        let ran = Arc::new(Mutex::new(0u32));
        let r1 = ran.clone();
        let r2 = ran.clone();
        let mut d = Dispatcher::new();
        d.add_task_fn(Box::new(move || {
            *r1.lock().unwrap() += 1;
        }));
        d.add_task_fn(Box::new(move || {
            *r2.lock().unwrap() += 1;
        }));
        d.flush();
        assert_eq!(d.pending_count(), 0);
        assert_eq!(*ran.lock().unwrap(), 2);
    }

    #[test]
    fn stop_sets_is_stopped_true() {
        let mut d = Dispatcher::new();
        assert!(!d.is_stopped());
        d.stop();
        assert!(d.is_stopped());
    }

    #[test]
    fn add_task_after_stop_is_noop() {
        let mut d = Dispatcher::new();
        d.add_task_fn(noop_task());
        d.stop();
        d.add_task_fn(noop_task()); // should be ignored
        assert_eq!(d.pending_count(), 1);
        // Run the surviving task so the shared `noop_task` body is exercised.
        d.flush();
        assert_eq!(d.pending_count(), 0);
    }

    #[test]
    fn flush_after_stop_executes_pre_stop_tasks_only() {
        let count = Arc::new(Mutex::new(0u32));
        let c1 = count.clone();

        let mut d = Dispatcher::new();
        d.add_task_fn(Box::new(move || {
            *c1.lock().unwrap() += 1;
        }));
        d.stop();
        // Post-stop add is silently dropped; we don't need a non-empty body
        // because the assertion below validates only the pre-stop counter.
        d.add_task_fn(noop_task());
        d.flush();

        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn flush_on_empty_queue_is_safe() {
        let mut d = Dispatcher::new();
        d.flush();
        assert_eq!(d.pending_count(), 0);
    }

    // ---- New tests for parity with C++ tasks.{cpp,h} ----

    #[test]
    fn default_constructs_same_as_new() {
        let d: Dispatcher = Dispatcher::default();
        assert_eq!(d.pending_count(), 0);
        assert!(!d.is_stopped());
        assert_eq!(d.get_dispatcher_cycle(), 0);
    }

    #[test]
    fn dispatcher_task_expiration_constant_matches_cpp() {
        // C++ tasks.h: const int DISPATCHER_TASK_EXPIRATION = 2000;
        assert_eq!(DISPATCHER_TASK_EXPIRATION, 2000);
    }

    #[test]
    fn task_new_has_no_expiration() {
        let task = Task::new(noop_task());
        assert!(
            !task.has_expired(),
            "tasks without expiration must never expire"
        );
        // Consume the task so the closure body executes once.
        task.run();
    }

    #[test]
    fn create_task_helper_builds_non_expiring_task() {
        // Mirrors C++ free function `Task* createTask(TaskFunc&&)`.
        let task = create_task(noop_task());
        assert!(!task.has_expired());
        task.run();
    }

    #[test]
    fn create_task_with_expiration_helper_uses_given_ms() {
        // Mirrors C++ `Task* createTask(uint32_t expiration, TaskFunc&&)`.
        let task = create_task_with_expiration(50_000, noop_task());
        // 50s in the future — definitely not expired right now.
        assert!(!task.has_expired());
        task.run();
    }

    #[test]
    fn task_with_expiration_is_not_expired_immediately() {
        let task = Task::with_expiration(60_000, noop_task());
        assert!(!task.has_expired());
        task.run();
    }

    #[test]
    fn task_with_zero_expiration_is_expired_after_short_sleep() {
        // 0 ms means the deadline is "now"; by the time we sleep 1ms and
        // check, it must be in the past.
        let task = Task::with_expiration(0, noop_task());
        thread::sleep(Duration::from_millis(1));
        assert!(task.has_expired());
    }

    #[test]
    fn set_dont_expire_resets_deadline() {
        // Build an already-expired task, then clear its deadline.
        let mut task = Task::with_expiration(0, noop_task());
        thread::sleep(Duration::from_millis(1));
        assert!(task.has_expired());
        task.set_dont_expire();
        assert!(
            !task.has_expired(),
            "set_dont_expire must clear the deadline"
        );
    }

    #[test]
    fn task_run_invokes_closure() {
        let flag = Arc::new(Mutex::new(false));
        let f = flag.clone();
        let task = Task::new(Box::new(move || {
            *f.lock().unwrap() = true;
        }));
        task.run();
        assert!(*flag.lock().unwrap());
    }

    #[test]
    fn add_task_with_expiration_enqueues_task() {
        let mut d = Dispatcher::new();
        d.add_task_with_expiration(10_000, noop_task());
        assert_eq!(d.pending_count(), 1);
        // Flush so the enqueued non-expired noop body executes.
        d.flush();
    }

    #[test]
    fn flush_skips_expired_tasks_without_executing() {
        let mut d = Dispatcher::new();
        // Body is `noop_task` — execution-by-other-tests keeps the closure
        // body covered.  The cycle counter assertion below is the actual
        // proof that the task was skipped (C++ only increments
        // dispatcherCycle when `!task->hasExpired()`).
        d.add_task_with_expiration(0, noop_task());
        thread::sleep(Duration::from_millis(2));
        d.flush();
        // Expired tasks are still drained.
        assert_eq!(d.pending_count(), 0);
        // And do not advance the cycle counter (C++ only increments when the
        // task is *not* expired).
        assert_eq!(d.get_dispatcher_cycle(), 0);
    }

    #[test]
    fn flush_runs_non_expired_after_skipping_expired() {
        let order = Arc::new(Mutex::new(Vec::<u32>::new()));
        let o_live = order.clone();
        let mut d = Dispatcher::new();
        // Expired closure body is the shared no-op — the live closure below
        // proves the dispatcher continued past the skipped task.
        d.add_task_with_expiration(0, noop_task());
        thread::sleep(Duration::from_millis(2));
        d.add_task_fn(Box::new(move || {
            o_live.lock().unwrap().push(2);
        }));
        d.flush();
        assert_eq!(*order.lock().unwrap(), vec![2]);
        // Cycle counter only incremented once — for the live task.
        assert_eq!(d.get_dispatcher_cycle(), 1);
    }

    #[test]
    fn dispatcher_cycle_starts_at_zero() {
        let d = Dispatcher::new();
        assert_eq!(d.get_dispatcher_cycle(), 0);
    }

    #[test]
    fn dispatcher_cycle_increments_per_non_expired_task() {
        let mut d = Dispatcher::new();
        d.add_task_fn(noop_task());
        d.add_task_fn(noop_task());
        d.add_task_fn(noop_task());
        d.flush();
        assert_eq!(d.get_dispatcher_cycle(), 3);
    }

    #[test]
    fn dispatcher_cycle_accumulates_across_flushes() {
        let mut d = Dispatcher::new();
        d.add_task_fn(noop_task());
        d.flush();
        d.add_task_fn(noop_task());
        d.add_task_fn(noop_task());
        d.flush();
        assert_eq!(d.get_dispatcher_cycle(), 3);
    }

    #[test]
    fn flush_is_panic_safe_and_drains_remaining_tasks() {
        let after = Arc::new(Mutex::new(false));
        let a = after.clone();
        let mut d = Dispatcher::new();
        d.add_task_fn(Box::new(|| panic!("boom")));
        d.add_task_fn(Box::new(move || {
            *a.lock().unwrap() = true;
        }));
        d.flush();
        assert!(
            *after.lock().unwrap(),
            "tasks following a panicking task must still execute"
        );
        // Both attempts count toward the cycle: panic happens after the
        // counter increments, mirroring C++ where the panic would terminate
        // the worker thread (we deliberately recover and keep going).
        assert_eq!(d.get_dispatcher_cycle(), 2);
        assert_eq!(d.pending_count(), 0);
    }
}
