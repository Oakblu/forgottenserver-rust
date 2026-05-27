use crate::database::{Database, InMemoryDb, Row};
use std::collections::VecDeque;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TaskError {
    Stopped,
}

// ── Task ──────────────────────────────────────────────────────────────────────

/// Callback signature mirroring C++ `std::function<void(DBResult_ptr, bool)>`.
///
/// First arg is `Some(rows)` for store-mode tasks (the C++ `DBResult_ptr`)
/// and `None` for execute-mode tasks. Second arg is the success flag —
/// `true` when the query/execute returned `Ok`, `false` on `Err`.
pub type TaskCallback = Box<dyn FnOnce(Option<Vec<Row>>, bool) + Send>;

/// Single database task. Mirrors C++ `DatabaseTask { query, callback, store }`.
struct Task {
    sql: String,
    /// When `true`, the query returns rows (C++ `db.storeQuery`).
    /// When `false`, the query is a write/DDL (C++ `db.executeQuery`).
    store: bool,
    /// Optional callback invoked after the query completes. The C++
    /// side posts the callback through `g_dispatcher.addTask`; the
    /// Rust port runs it inline during `flush` (which the dispatcher
    /// drives — same observable model).
    callback: Option<TaskCallback>,
}

// ── DatabaseTasks ─────────────────────────────────────────────────────────────

/// Synchronous, FIFO task queue for deferred SQL execution.
///
/// The C++ original runs tasks on a dedicated thread using a mutex-protected
/// `std::list`.  This Rust version is intentionally single-threaded and
/// synchronous — no `tokio`/`async`, no OS threads.  The queue can be flushed
/// into any `InMemoryDb` for deterministic testing.
pub struct DatabaseTasks {
    queue: VecDeque<Task>,
    stopped: bool,
    executed_order: Vec<String>,
}

impl DatabaseTasks {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            stopped: false,
            executed_order: Vec::new(),
        }
    }

    /// Enqueue a SQL string. Returns `Err(TaskError::Stopped)` if the
    /// queue has been stopped. Equivalent to
    /// `add_task_with_callback(sql, None, false)`.
    pub fn add_task(&mut self, sql: impl Into<String>) -> Result<(), TaskError> {
        self.add_task_with_callback(sql, None, false)
    }

    /// Full-form enqueue mirroring C++ `DatabaseTasks::addTask(query,
    /// callback, store)`. When `store == true` the queue runs the SQL
    /// via `Database::query` (returns rows); otherwise via
    /// `Database::execute` (returns affected-row count, surfaced to
    /// the callback as the success flag).
    pub fn add_task_with_callback(
        &mut self,
        sql: impl Into<String>,
        callback: Option<TaskCallback>,
        store: bool,
    ) -> Result<(), TaskError> {
        if self.stopped {
            return Err(TaskError::Stopped);
        }
        self.queue.push_back(Task {
            sql: sql.into(),
            store,
            callback,
        });
        Ok(())
    }

    /// Number of pending tasks in the queue.
    pub fn task_count(&self) -> usize {
        self.queue.len()
    }

    /// Execute all queued tasks in FIFO order against `db`, then clear
    /// the queue. Mirrors the loop body of C++ `DatabaseTasks::runTask`
    /// for each dequeued task:
    ///
    ///   * `store == true`  → `db.query(sql)`; callback receives
    ///     `Some(rows)` + `success = is_ok()`.
    ///   * `store == false` → `db.execute(sql)`; callback receives
    ///     `None` + `success = is_ok()`.
    ///
    /// Callbacks run inline (the dispatcher already drives `flush`).
    pub fn flush(&mut self, db: &mut InMemoryDb) {
        while let Some(task) = self.queue.pop_front() {
            let success: bool;
            let rows: Option<Vec<Row>>;
            if task.store {
                match db.query(&task.sql) {
                    Ok(r) => {
                        rows = Some(r);
                        success = true;
                    }
                    Err(_) => {
                        rows = None;
                        success = false;
                    }
                }
            } else {
                rows = None;
                success = db.execute(&task.sql).is_ok();
            }
            self.executed_order.push(task.sql);
            if let Some(cb) = task.callback {
                cb(rows, success);
            }
        }
    }

    /// Mark the queue as stopped.  Subsequent calls to `add_task` will fail.
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// The order in which tasks were executed by `flush`.
    pub fn executed_order(&self) -> &[String] {
        &self.executed_order
    }
}

impl Default for DatabaseTasks {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_queue() {
        let tasks = DatabaseTasks::new();
        assert_eq!(tasks.task_count(), 0);
        assert!(!tasks.is_stopped());
    }

    #[test]
    fn add_task_enqueues_and_task_count_returns_one() {
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("SELECT 1").unwrap();
        assert_eq!(tasks.task_count(), 1);
    }

    #[test]
    fn flush_executes_tasks_and_empties_queue() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("SELECT 1").unwrap();
        tasks.add_task("SELECT 2").unwrap();
        tasks.flush(&mut db);
        assert_eq!(tasks.task_count(), 0);
        assert_eq!(db.executed_statements.len(), 2);
    }

    #[test]
    fn flush_preserves_fifo_order() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("task1").unwrap();
        tasks.add_task("task2").unwrap();
        tasks.add_task("task3").unwrap();
        tasks.flush(&mut db);
        assert_eq!(tasks.executed_order(), &["task1", "task2", "task3"]);
    }

    #[test]
    fn stop_marks_as_stopped() {
        let mut tasks = DatabaseTasks::new();
        tasks.stop();
        assert!(tasks.is_stopped());
    }

    #[test]
    fn add_task_after_stop_returns_err() {
        let mut tasks = DatabaseTasks::new();
        tasks.stop();
        assert_eq!(tasks.add_task("SELECT 1"), Err(TaskError::Stopped));
    }

    #[test]
    fn two_sequential_add_tasks_both_executed_after_flush() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("INSERT INTO a VALUES (1)").unwrap();
        tasks.add_task("INSERT INTO b VALUES (2)").unwrap();
        tasks.flush(&mut db);
        assert_eq!(db.executed_statements.len(), 2);
    }

    // ── flush on empty queue ──────────────────────────────────────────────────

    #[test]
    fn flush_on_empty_queue_is_a_noop() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.flush(&mut db);
        assert_eq!(tasks.task_count(), 0);
        assert!(db.executed_statements.is_empty());
        assert!(tasks.executed_order().is_empty());
    }

    // ── executed_order after two flushes ─────────────────────────────────────

    #[test]
    fn executed_order_accumulates_across_multiple_flushes() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("sql_a").unwrap();
        tasks.flush(&mut db);
        tasks.add_task("sql_b").unwrap();
        tasks.flush(&mut db);
        assert_eq!(tasks.executed_order(), &["sql_a", "sql_b"]);
    }

    // ── task_count after flush is zero ────────────────────────────────────────

    #[test]
    fn task_count_is_zero_after_flush() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("q1").unwrap();
        tasks.add_task("q2").unwrap();
        tasks.flush(&mut db);
        assert_eq!(tasks.task_count(), 0);
    }

    // ── stop + flush mirrors C++ shutdown behavior ────────────────────────────

    #[test]
    fn stop_then_flush_processes_remaining_tasks() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("pending_task").unwrap();
        tasks.stop();
        // The queue still has the task — flush should drain it
        tasks.flush(&mut db);
        assert_eq!(db.executed_statements, vec!["pending_task"]);
        assert_eq!(tasks.task_count(), 0);
    }

    #[test]
    fn stop_then_add_task_returns_stopped_error_and_queue_unchanged() {
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("early_task").unwrap();
        tasks.stop();
        let err = tasks.add_task("late_task");
        assert_eq!(err, Err(TaskError::Stopped));
        // The earlier task is still in the queue; the late one was rejected
        assert_eq!(tasks.task_count(), 1);
    }

    // ── is_stopped state transitions ─────────────────────────────────────────

    #[test]
    fn is_stopped_false_by_default() {
        let tasks = DatabaseTasks::new();
        assert!(!tasks.is_stopped());
    }

    #[test]
    fn is_stopped_true_after_stop_called() {
        let mut tasks = DatabaseTasks::new();
        tasks.stop();
        assert!(tasks.is_stopped());
    }

    // ── large batch stays in FIFO order ──────────────────────────────────────

    #[test]
    fn large_batch_fifo_order_preserved() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        let sqls: Vec<String> = (0..10).map(|i| format!("stmt_{i}")).collect();
        for sql in &sqls {
            tasks.add_task(sql.as_str()).unwrap();
        }
        tasks.flush(&mut db);
        assert_eq!(tasks.executed_order(), sqls.as_slice());
    }

    // ── default() is equivalent to new() ─────────────────────────────────────

    #[test]
    fn default_creates_empty_stopped_false_queue() {
        let tasks = DatabaseTasks::default();
        assert_eq!(tasks.task_count(), 0);
        assert!(!tasks.is_stopped());
        assert!(tasks.executed_order().is_empty());
    }

    // ── add_task accepts String and &str (Into<String>) ───────────────────────

    #[test]
    fn add_task_accepts_owned_string() {
        let mut tasks = DatabaseTasks::new();
        tasks.add_task(String::from("owned_sql")).unwrap();
        assert_eq!(tasks.task_count(), 1);
    }

    // ── TaskError equality and debug ──────────────────────────────────────────

    #[test]
    fn task_error_stopped_is_equal_to_itself() {
        assert_eq!(TaskError::Stopped, TaskError::Stopped);
    }

    #[test]
    fn task_error_stopped_debug_format() {
        let s = format!("{:?}", TaskError::Stopped);
        assert!(s.contains("Stopped"));
    }

    // ── store flag + callback (Session 32) ──────────────────────────────

    use std::sync::{Arc, Mutex};

    /// Captured-callback type alias used by the store/execute tests.
    type CapturedCallback = Arc<Mutex<Option<(Option<Vec<Row>>, bool)>>>;

    /// store=true → callback receives Some(rows) + success=true.
    #[test]
    fn callback_receives_rows_when_store_true() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        let captured: CapturedCallback = Arc::new(Mutex::new(None));
        let c = captured.clone();
        tasks
            .add_task_with_callback(
                "SELECT 1",
                Some(Box::new(move |rows, success| {
                    *c.lock().unwrap() = Some((rows, success));
                })),
                true,
            )
            .unwrap();
        tasks.flush(&mut db);
        let (rows, success) = captured.lock().unwrap().take().unwrap();
        assert!(success, "store-mode query on InMemoryDb returns Ok");
        assert!(
            rows.is_some(),
            "store-mode callback must receive Some(rows)"
        );
    }

    /// store=false → callback receives None + success based on Ok/Err.
    #[test]
    fn callback_receives_no_rows_when_store_false() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        let captured: CapturedCallback = Arc::new(Mutex::new(None));
        let c = captured.clone();
        tasks
            .add_task_with_callback(
                "INSERT INTO foo VALUES (1)",
                Some(Box::new(move |rows, success| {
                    *c.lock().unwrap() = Some((rows, success));
                })),
                false,
            )
            .unwrap();
        tasks.flush(&mut db);
        let (rows, success) = captured.lock().unwrap().take().unwrap();
        assert!(rows.is_none(), "execute-mode callback must receive None");
        assert!(success, "InMemoryDb::execute returns Ok");
    }

    /// Missing callback (None) → no panic during flush.
    #[test]
    fn flush_no_panic_when_callback_absent() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks
            .add_task_with_callback("SELECT 1", None, true)
            .unwrap();
        tasks.flush(&mut db);
        assert_eq!(tasks.task_count(), 0);
    }

    /// Stopped queue rejects the new full-form API too.
    #[test]
    fn add_task_with_callback_after_stop_returns_err() {
        let mut tasks = DatabaseTasks::new();
        tasks.stop();
        let err = tasks.add_task_with_callback("SELECT 1", None, true);
        assert_eq!(err, Err(TaskError::Stopped));
    }

    /// The legacy `add_task(sql)` path still works and matches the
    /// `add_task_with_callback(sql, None, false)` shape.
    #[test]
    fn legacy_add_task_equivalent_to_no_callback_no_store() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        tasks.add_task("UPDATE foo SET x = 1").unwrap();
        tasks.flush(&mut db);
        assert_eq!(tasks.executed_order(), &["UPDATE foo SET x = 1"]);
    }

    /// Multiple tasks fire their callbacks in FIFO order.
    #[test]
    fn callbacks_fire_in_fifo_order_during_flush() {
        let mut db = InMemoryDb::new();
        let mut tasks = DatabaseTasks::new();
        let log: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));
        for i in 1..=3 {
            let l = log.clone();
            tasks
                .add_task_with_callback(
                    format!("SELECT {i}"),
                    Some(Box::new(move |_rows, _success| {
                        l.lock().unwrap().push(i);
                    })),
                    true,
                )
                .unwrap();
        }
        tasks.flush(&mut db);
        assert_eq!(*log.lock().unwrap(), vec![1, 2, 3]);
    }
}
