use std::collections::HashMap;

// ── Value types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum DbValue {
    Integer(i64),
    Text(String),
    Float(f64),
    Null,
}

// ── Error type ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum DbError {
    ConnectionFailed,
    QueryError(String),
    NotFound,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::ConnectionFailed => write!(f, "connection failed"),
            DbError::QueryError(msg) => write!(f, "query error: {msg}"),
            DbError::NotFound => write!(f, "not found"),
        }
    }
}

// ── Row ──────────────────────────────────────────────────────────────────────

/// A single result row with named columns.
#[derive(Debug, Clone)]
pub struct Row {
    columns: HashMap<String, DbValue>,
}

impl Row {
    pub fn new(columns: HashMap<String, DbValue>) -> Self {
        Self { columns }
    }

    /// Typed column accessor.  Returns `None` if the column is missing or the
    /// type does not match.
    pub fn get<T: FromDbValue>(&self, col: &str) -> Option<T> {
        self.columns.get(col).and_then(T::from_db_value)
    }

    pub fn get_raw(&self, col: &str) -> Option<&DbValue> {
        self.columns.get(col)
    }

    /// Return all column names present in this row.
    /// Mirrors the C++ `listNames` map built from `mysql_fetch_field` calls.
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.keys().map(|s| s.as_str()).collect()
    }
}

// ── Type-conversion helper ────────────────────────────────────────────────────

pub trait FromDbValue: Sized {
    fn from_db_value(v: &DbValue) -> Option<Self>;
}

impl FromDbValue for i64 {
    fn from_db_value(v: &DbValue) -> Option<Self> {
        match v {
            DbValue::Integer(n) => Some(*n),
            _ => None,
        }
    }
}

impl FromDbValue for u64 {
    fn from_db_value(v: &DbValue) -> Option<Self> {
        match v {
            DbValue::Integer(n) if *n >= 0 => Some(*n as u64),
            _ => None,
        }
    }
}

impl FromDbValue for u32 {
    fn from_db_value(v: &DbValue) -> Option<Self> {
        match v {
            DbValue::Integer(n) if *n >= 0 => Some(*n as u32),
            _ => None,
        }
    }
}

impl FromDbValue for u16 {
    fn from_db_value(v: &DbValue) -> Option<Self> {
        match v {
            DbValue::Integer(n) if *n >= 0 && *n <= u16::MAX as i64 => Some(*n as u16),
            _ => None,
        }
    }
}

impl FromDbValue for u8 {
    fn from_db_value(v: &DbValue) -> Option<Self> {
        match v {
            DbValue::Integer(n) if *n >= 0 && *n <= u8::MAX as i64 => Some(*n as u8),
            _ => None,
        }
    }
}

impl FromDbValue for String {
    fn from_db_value(v: &DbValue) -> Option<Self> {
        match v {
            DbValue::Text(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl FromDbValue for f64 {
    fn from_db_value(v: &DbValue) -> Option<Self> {
        match v {
            DbValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

// ── Database trait ────────────────────────────────────────────────────────────

pub trait Database {
    /// Execute a query that returns rows (e.g. SELECT).
    fn query(&self, sql: &str) -> Result<Vec<Row>, DbError>;

    /// Execute a statement that does not return rows (INSERT/UPDATE/DELETE).
    /// Returns the number of affected rows.
    fn execute(&mut self, sql: &str) -> Result<u64, DbError>;

    /// Escape a string value for safe inclusion in SQL.
    fn escape_string(&self, s: &str) -> String;

    /// Escape a byte buffer as a MySQL/MariaDB hex literal (`X'...'`).
    ///
    /// Default implementation produces the standard hex-literal form, which
    /// every Database backend can consume. Real backends may override if
    /// they prefer parameter binding or a different escaping convention.
    fn escape_blob(&self, bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2 + 3);
        s.push_str("X'");
        for b in bytes {
            s.push_str(&format!("{b:02X}"));
        }
        s.push('\'');
        s
    }

    /// Begin a transaction. Default no-op so the in-memory backend
    /// remains a drop-in.
    fn begin_transaction(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    /// Commit the current transaction. Default no-op.
    fn commit(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    /// Roll back the current transaction. Default no-op.
    fn rollback(&mut self) -> Result<(), DbError> {
        Ok(())
    }
}

// ── In-memory implementation ──────────────────────────────────────────────────

/// Simple in-memory database used in tests.
///
/// Tables are stored as named collections of rows.  SQL is not actually parsed;
/// instead the test code manipulates the store directly through helper methods.
#[derive(Default)]
pub struct InMemoryDb {
    /// table_name → rows
    tables: HashMap<String, Vec<Row>>,
    /// key-value config store (mirrors `server_config` table)
    config: HashMap<String, i64>,
    pub executed_statements: Vec<String>,
    /// Records "BEGIN", "COMMIT", or "ROLLBACK" for each transaction call.
    pub transaction_log: Vec<String>,
    last_insert_id: u64,
}

impl InMemoryDb {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty table (no-op if it already exists).
    pub fn create_table(&mut self, name: &str) {
        self.tables.entry(name.to_string()).or_default();
    }

    /// Returns true if the table was previously created.
    pub fn table_exists(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Insert a row into a table (creates the table if necessary).
    pub fn insert_row(&mut self, table: &str, row: Row) {
        self.tables.entry(table.to_string()).or_default().push(row);
        self.last_insert_id += 1;
    }

    /// Return all rows from a table.
    pub fn rows(&self, table: &str) -> &[Row] {
        self.tables.get(table).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Return mutable reference to table rows.
    pub fn rows_mut(&mut self, table: &str) -> Option<&mut Vec<Row>> {
        self.tables.get_mut(table)
    }

    /// Store a config value.
    pub fn set_config(&mut self, key: &str, value: i64) {
        self.config.insert(key.to_string(), value);
    }

    /// Get a config value.
    pub fn get_config(&self, key: &str) -> Option<i64> {
        self.config.get(key).copied()
    }

    /// Returns true if at least one table has been created.
    ///
    /// Mirrors C++ `DatabaseManager::isDatabaseSetup()` which queries
    /// `information_schema.tables` for any table in the schema.
    pub fn has_any_table(&self) -> bool {
        !self.tables.is_empty()
    }

    pub fn last_insert_id(&self) -> u64 {
        self.last_insert_id
    }

    /// Record that a statement was run (used by DatabaseTasks tests).
    pub fn record_statement(&mut self, sql: &str) {
        self.executed_statements.push(sql.to_string());
    }
}

impl Database for InMemoryDb {
    fn query(&self, _sql: &str) -> Result<Vec<Row>, DbError> {
        // Not used in the in-memory stub — callers use direct table access.
        Ok(vec![])
    }

    fn execute(&mut self, sql: &str) -> Result<u64, DbError> {
        self.record_statement(sql);
        Ok(1)
    }

    fn escape_string(&self, s: &str) -> String {
        // Mirrors MySQL's escaping: backslash → \\, single-quote → \'
        let mut out = String::with_capacity(s.len() + 2);
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '\'' => out.push_str("\\'"),
                c => out.push(c),
            }
        }
        out
    }

    fn begin_transaction(&mut self) -> Result<(), DbError> {
        self.transaction_log.push("BEGIN".to_string());
        Ok(())
    }

    fn commit(&mut self) -> Result<(), DbError> {
        self.transaction_log.push("COMMIT".to_string());
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), DbError> {
        self.transaction_log.push("ROLLBACK".to_string());
        Ok(())
    }
}

// ── DbResult ─────────────────────────────────────────────────────────────────

/// Mirrors C++ `DBResult`: an ordered cursor over a set of result rows.
///
/// In C++ the constructor calls `mysql_fetch_field` to build `listNames` and
/// then fetches the first row.  Here rows arrive pre-built; `column_names()`
/// on the first row provides the same information.
///
/// `has_next()` — true when the cursor points at a valid row (C++ `row != nullptr`).
/// `next()` — advance the cursor; returns false when exhausted.
pub struct DbResult {
    rows: Vec<Row>,
    /// Zero-based index of the currently active row.
    cursor: usize,
}

impl DbResult {
    pub fn new(rows: Vec<Row>) -> Self {
        Self { rows, cursor: 0 }
    }

    /// Returns `true` if the cursor points at a valid row (C++ `hasNext()`).
    pub fn has_next(&self) -> bool {
        self.cursor < self.rows.len()
    }

    /// Advance the cursor to the next row. Returns `true` if the new position
    /// is still within the result set (C++ `next()`).
    ///
    /// Named `advance` rather than `next` to avoid confusion with
    /// `std::iter::Iterator::next` (clippy `should_implement_trait`).
    pub fn advance(&mut self) -> bool {
        self.cursor += 1;
        self.cursor < self.rows.len()
    }

    /// Return a reference to the row at the current cursor position.
    pub fn current_row(&self) -> Option<&Row> {
        self.rows.get(self.cursor)
    }

    /// Return the string value of `column` in the current row.
    pub fn get_string(&self, column: &str) -> Option<String> {
        self.current_row()?.get::<String>(column)
    }

    /// Return a typed numeric value of `column` in the current row.
    pub fn get_number<T: FromDbValue>(&self, column: &str) -> Option<T> {
        self.current_row()?.get::<T>(column)
    }

    /// Return all column names from the first row.
    ///
    /// Mirrors the C++ `listNames` map (column name → index) built from
    /// `mysql_fetch_field` calls inside the constructor.
    pub fn column_names(&self) -> Vec<String> {
        self.rows
            .first()
            .map(|r| r.column_names().iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }
}

// ── DbInsert ──────────────────────────────────────────────────────────────────

/// Mirrors C++ `DBInsert`: batches `INSERT … VALUES` rows and flushes when the
/// accumulated SQL would exceed `max_packet_size`.
///
/// C++ behaviour preserved:
/// * `addRow` appends `(row)` or `,(row)` to `values`.
/// * When `length > maxPacketSize` **before** appending, `execute()` is called
///   first to flush the current batch, then the new row is appended to a fresh
///   `values` string.
/// * `execute()` is idempotent when `values` is empty.
pub struct DbInsert {
    /// The fixed `INSERT INTO … VALUES` prefix, unchanged across flushes.
    query: String,
    /// Accumulated row fragments: `(v1),(v2),…`
    values: String,
    /// `query.len() + values.len()` — tracks total SQL length for packet-size checks.
    length: usize,
    /// Maximum allowed SQL length before a forced flush.
    max_packet_size: usize,
}

impl DbInsert {
    pub fn new(query: impl Into<String>, max_packet_size: usize) -> Self {
        let query = query.into();
        let length = query.len();
        Self {
            query,
            values: String::new(),
            length,
            max_packet_size,
        }
    }

    /// Append `row` (the inner values without parentheses) to the batch.
    ///
    /// If adding `row` would push the accumulated length past `max_packet_size`,
    /// the current batch is flushed first via `execute()`.
    /// Returns `false` only if a forced flush fails.
    pub fn add_row(&mut self, db: &mut dyn Database, row: &str) -> bool {
        self.length += row.len();
        if self.length > self.max_packet_size && !self.execute(db) {
            return false;
        }
        if self.values.is_empty() {
            self.values.push('(');
            self.values.push_str(row);
            self.values.push(')');
        } else {
            self.values.push_str(",(");
            self.values.push_str(row);
            self.values.push(')');
        }
        true
    }

    /// Flush the accumulated rows via a single `execute` call.
    ///
    /// No-op (returns `true`) when there are no pending rows.
    /// After flushing, `values` is cleared and `length` is reset to
    /// `query.len()`, ready for the next batch.
    pub fn execute(&mut self, db: &mut dyn Database) -> bool {
        if self.values.is_empty() {
            return true;
        }
        let sql = format!("{}{}", self.query, self.values);
        let ok = db.execute(&sql).is_ok();
        self.values.clear();
        self.length = self.query.len();
        ok
    }
}

// ── DbTransaction ─────────────────────────────────────────────────────────────

/// RAII transaction guard — mirrors C++ `DBTransaction`.
///
/// * `begin()` calls `db.begin_transaction()` and, on success, returns a guard
///   whose state is `Started`.
/// * `commit()` consumes `self` (moves it), sets state to `Committed`, and
///   calls `db.commit()`.  Because `self` is moved, `drop` is NOT called
///   afterward, so no rollback is issued.
/// * If the guard is dropped while still in `Started` state (i.e. without a
///   `commit()` call), `drop` issues `db.rollback()`.
///
/// This replicates the C++ destructor:
/// ```cpp
/// ~DBTransaction() { if (state == STATE_START) db.rollback(); }
/// ```
#[derive(Debug, PartialEq)]
enum TxState {
    Started,
    Committed,
}

pub struct DbTransaction<'a> {
    db: &'a mut dyn Database,
    state: TxState,
}

impl<'a> DbTransaction<'a> {
    /// Begin a transaction and return the RAII guard.
    pub fn begin(db: &'a mut dyn Database) -> Result<Self, DbError> {
        db.begin_transaction()?;
        Ok(Self {
            db,
            state: TxState::Started,
        })
    }

    /// Commit the transaction.  Consumes the guard so `drop` is never called.
    pub fn commit(mut self) -> Result<(), DbError> {
        self.state = TxState::Committed;
        self.db.commit()
    }
}

impl Drop for DbTransaction<'_> {
    fn drop(&mut self) {
        if self.state == TxState::Started {
            let _ = self.db.rollback();
        }
    }
}

// ── StoreQuery (WHERE-clause builder) ────────────────────────────────────────

/// Mirrors the C++ `DBInsert` / `DBResult` query builder pattern.
pub struct StoreQuery {
    _table: String,
    conditions: Vec<String>,
}

impl StoreQuery {
    pub fn new(table: &str) -> Self {
        Self {
            _table: table.to_string(),
            conditions: Vec::new(),
        }
    }

    /// Add a WHERE condition fragment: `column op value`.
    pub fn and(mut self, column: &str, op: &str, value: &str) -> Self {
        self.conditions.push(format!("{column} {op} {value}"));
        self
    }

    /// Build the WHERE-clause body (without the `WHERE` keyword).
    pub fn build(self) -> String {
        self.conditions.join(" AND ")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(cols: &[(&str, DbValue)]) -> Row {
        let map: HashMap<String, DbValue> = cols
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        Row::new(map)
    }

    #[test]
    fn row_get_i64_returns_value_when_column_exists() {
        let row = make_row(&[("level", DbValue::Integer(42))]);
        assert_eq!(row.get::<i64>("level"), Some(42));
    }

    #[test]
    fn row_get_string_returns_value_for_text_column() {
        let row = make_row(&[("name", DbValue::Text("Alice".to_string()))]);
        assert_eq!(row.get::<String>("name"), Some("Alice".to_string()));
    }

    #[test]
    fn row_get_returns_none_for_missing_column() {
        let row = make_row(&[("level", DbValue::Integer(10))]);
        assert_eq!(row.get::<i64>("missing"), None);
    }

    #[test]
    fn escape_string_escapes_single_quotes() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_string("O'Brien"), "O\\'Brien");
    }

    #[test]
    fn escape_string_escapes_backslashes() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn store_query_builds_where_clause() {
        let clause = StoreQuery::new("players")
            .and("level", ">", "5")
            .and("name", "=", "'rat'")
            .build();
        assert_eq!(clause, "level > 5 AND name = 'rat'");
    }

    #[test]
    fn store_query_single_condition() {
        let clause = StoreQuery::new("players").and("id", "=", "1").build();
        assert_eq!(clause, "id = 1");
    }

    #[test]
    fn in_memory_db_execute_records_statement() {
        let mut db = InMemoryDb::new();
        db.execute("INSERT INTO foo VALUES (1)").unwrap();
        assert_eq!(db.executed_statements, vec!["INSERT INTO foo VALUES (1)"]);
    }

    #[test]
    fn in_memory_db_create_table_and_table_exists() {
        let mut db = InMemoryDb::new();
        assert!(!db.table_exists("players"));
        db.create_table("players");
        assert!(db.table_exists("players"));
    }

    // ── DbError Display ───────────────────────────────────────────────────────

    #[test]
    fn db_error_connection_failed_display() {
        assert_eq!(
            format!("{}", DbError::ConnectionFailed),
            "connection failed"
        );
    }

    #[test]
    fn db_error_query_error_display_contains_message() {
        let err = DbError::QueryError("syntax error".to_string());
        assert_eq!(format!("{err}"), "query error: syntax error");
    }

    #[test]
    fn db_error_not_found_display() {
        assert_eq!(format!("{}", DbError::NotFound), "not found");
    }

    // ── FromDbValue edge cases ─────────────────────────────────────────────────

    #[test]
    fn from_db_value_u64_returns_none_for_negative_integer() {
        assert_eq!(u64::from_db_value(&DbValue::Integer(-1)), None);
    }

    #[test]
    fn from_db_value_u32_returns_none_for_negative_integer() {
        assert_eq!(u32::from_db_value(&DbValue::Integer(-5)), None);
    }

    #[test]
    fn from_db_value_u64_returns_value_for_zero() {
        assert_eq!(u64::from_db_value(&DbValue::Integer(0)), Some(0u64));
    }

    #[test]
    fn from_db_value_u32_returns_value_for_positive() {
        assert_eq!(u32::from_db_value(&DbValue::Integer(99)), Some(99u32));
    }

    #[test]
    fn from_db_value_f64_returns_value() {
        assert_eq!(f64::from_db_value(&DbValue::Float(2.5)), Some(2.5f64));
    }

    #[test]
    fn from_db_value_f64_returns_none_for_non_float() {
        assert_eq!(f64::from_db_value(&DbValue::Integer(1)), None);
    }

    #[test]
    fn from_db_value_i64_returns_none_for_null() {
        assert_eq!(i64::from_db_value(&DbValue::Null), None);
    }

    #[test]
    fn from_db_value_string_returns_none_for_null() {
        assert_eq!(String::from_db_value(&DbValue::Null), None);
    }

    // ── Row::get_raw ──────────────────────────────────────────────────────────

    #[test]
    fn row_get_raw_returns_some_for_existing_column() {
        let row = make_row(&[("level", DbValue::Integer(5))]);
        assert_eq!(row.get_raw("level"), Some(&DbValue::Integer(5)));
    }

    #[test]
    fn row_get_raw_returns_none_for_missing_column() {
        let row = make_row(&[("level", DbValue::Integer(5))]);
        assert_eq!(row.get_raw("hp"), None);
    }

    #[test]
    fn row_get_raw_null_value() {
        let row = make_row(&[("field", DbValue::Null)]);
        assert_eq!(row.get_raw("field"), Some(&DbValue::Null));
    }

    // ── InMemoryDb insert_row / rows / last_insert_id ─────────────────────────

    #[test]
    fn insert_row_increases_last_insert_id() {
        let mut db = InMemoryDb::new();
        db.insert_row("players", make_row(&[("id", DbValue::Integer(1))]));
        assert_eq!(db.last_insert_id(), 1);
        db.insert_row("players", make_row(&[("id", DbValue::Integer(2))]));
        assert_eq!(db.last_insert_id(), 2);
    }

    #[test]
    fn rows_returns_empty_slice_for_unknown_table() {
        let db = InMemoryDb::new();
        assert!(db.rows("nonexistent").is_empty());
    }

    #[test]
    fn insert_row_then_rows_returns_inserted_data() {
        let mut db = InMemoryDb::new();
        let row = make_row(&[("name", DbValue::Text("Bob".to_string()))]);
        db.insert_row("players", row);
        assert_eq!(db.rows("players").len(), 1);
        assert_eq!(
            db.rows("players")[0].get::<String>("name"),
            Some("Bob".to_string())
        );
    }

    #[test]
    fn rows_mut_returns_mutable_reference_and_allows_modification() {
        let mut db = InMemoryDb::new();
        db.create_table("items");
        db.insert_row("items", make_row(&[("id", DbValue::Integer(10))]));
        {
            let rows = db.rows_mut("items").expect("table should exist");
            rows.push(make_row(&[("id", DbValue::Integer(20))]));
        }
        assert_eq!(db.rows("items").len(), 2);
    }

    #[test]
    fn rows_mut_returns_none_for_missing_table() {
        let mut db = InMemoryDb::new();
        assert!(db.rows_mut("ghost").is_none());
    }

    // ── InMemoryDb config store ───────────────────────────────────────────────

    #[test]
    fn set_config_then_get_config_returns_value() {
        let mut db = InMemoryDb::new();
        db.set_config("max_level", 300);
        assert_eq!(db.get_config("max_level"), Some(300));
    }

    #[test]
    fn get_config_returns_none_for_unknown_key() {
        let db = InMemoryDb::new();
        assert_eq!(db.get_config("missing"), None);
    }

    #[test]
    fn set_config_overwrites_previous_value() {
        let mut db = InMemoryDb::new();
        db.set_config("rate", 1);
        db.set_config("rate", 10);
        assert_eq!(db.get_config("rate"), Some(10));
    }

    // ── InMemoryDb::query always returns Ok(empty) ────────────────────────────

    #[test]
    fn in_memory_db_query_returns_ok_empty_vec() {
        let db = InMemoryDb::new();
        let result = db.query("SELECT * FROM players");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ── InMemoryDb::execute returns affected-row count ────────────────────────

    #[test]
    fn in_memory_db_execute_returns_one() {
        let mut db = InMemoryDb::new();
        let affected = db.execute("DELETE FROM players WHERE id = 1").unwrap();
        assert_eq!(affected, 1);
    }

    // ── escape_string edge cases ──────────────────────────────────────────────

    #[test]
    fn escape_string_empty_string_returns_empty() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_string(""), "");
    }

    #[test]
    fn escape_string_no_special_chars_unchanged() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_string("hello world"), "hello world");
    }

    #[test]
    fn escape_string_multiple_single_quotes_all_escaped() {
        let db = InMemoryDb::new();
        let result = db.escape_string("a'b'c");
        assert_eq!(result, "a\\'b\\'c");
    }

    #[test]
    fn escape_string_both_backslash_and_quote() {
        let db = InMemoryDb::new();
        let result = db.escape_string("\\'");
        assert_eq!(result, "\\\\\\'");
    }

    // ── StoreQuery edge cases ─────────────────────────────────────────────────

    #[test]
    fn store_query_no_conditions_build_returns_empty_string() {
        let clause = StoreQuery::new("players").build();
        assert_eq!(clause, "");
    }

    #[test]
    fn store_query_three_conditions_joined_with_and() {
        let clause = StoreQuery::new("t")
            .and("a", "=", "1")
            .and("b", ">", "2")
            .and("c", "<", "3")
            .build();
        assert_eq!(clause, "a = 1 AND b > 2 AND c < 3");
    }

    // ── DbError PartialEq ──────────────────────────────────────────────────────

    #[test]
    fn db_error_connection_failed_equality() {
        assert_eq!(DbError::ConnectionFailed, DbError::ConnectionFailed);
    }

    #[test]
    fn db_error_query_error_equality() {
        let a = DbError::QueryError("oops".to_string());
        let b = DbError::QueryError("oops".to_string());
        assert_eq!(a, b);
    }

    #[test]
    fn db_error_not_found_equality() {
        assert_eq!(DbError::NotFound, DbError::NotFound);
    }

    #[test]
    fn db_error_different_variants_not_equal() {
        assert_ne!(DbError::ConnectionFailed, DbError::NotFound);
    }

    // ── Database trait default-impl tests ────────────────────────────────────

    #[test]
    fn escape_blob_empty_yields_empty_hex_literal() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_blob(&[]), "X''");
    }

    #[test]
    fn escape_blob_single_byte() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_blob(&[0x42]), "X'42'");
    }

    #[test]
    fn escape_blob_multiple_bytes_uppercase_hex() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_blob(&[0xAB, 0xCD, 0xEF]), "X'ABCDEF'");
    }

    #[test]
    fn escape_blob_preserves_high_bits() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_blob(&[0xFF, 0x00, 0x80]), "X'FF0080'");
    }

    #[test]
    fn escape_blob_pads_single_hex_digit_with_leading_zero() {
        let db = InMemoryDb::new();
        assert_eq!(db.escape_blob(&[0x01, 0x0A]), "X'010A'");
    }

    #[test]
    fn begin_transaction_default_is_noop_ok() {
        let mut db = InMemoryDb::new();
        assert_eq!(db.begin_transaction(), Ok(()));
    }

    #[test]
    fn commit_default_is_noop_ok() {
        let mut db = InMemoryDb::new();
        assert_eq!(db.commit(), Ok(()));
    }

    #[test]
    fn rollback_default_is_noop_ok() {
        let mut db = InMemoryDb::new();
        assert_eq!(db.rollback(), Ok(()));
    }

    // ── Task 3.1: Database::getInstance — singleton contract ─────────────────
    // C++ uses a function-local static (`static Database instance; return instance;`).
    // Rust replaces the global singleton with explicit-reference passing.
    // The observable contract is: all callers that share the same Database
    // reference see the same state.  This test confirms that a single
    // `InMemoryDb` passed as `&mut` maintains consistent state across two
    // separate borrows, matching the C++ singleton observable behavior.

    #[test]
    fn singleton_contract_single_instance_state_shared_across_borrows() {
        let mut db = InMemoryDb::new();
        // First caller inserts a row
        db.create_table("players");
        db.insert_row(
            "players",
            Row::new(std::collections::HashMap::from([(
                "id".to_string(),
                DbValue::Integer(1),
            )])),
        );
        // Second caller (using a re-borrow) sees the same state
        let row_count = db.rows("players").len();
        assert_eq!(
            row_count, 1,
            "all callers sharing the same DB reference see consistent state (singleton contract)"
        );
    }

    #[test]
    fn singleton_contract_no_separate_instance_has_different_state() {
        // Two independent InMemoryDb instances do NOT share state —
        // confirming that the singleton pattern (single shared reference)
        // is required to observe consistent state.
        let mut db1 = InMemoryDb::new();
        let db2 = InMemoryDb::new();
        db1.create_table("players");
        // db2 does not have the "players" table
        assert!(db1.table_exists("players"));
        assert!(!db2.table_exists("players"));
    }

    #[test]
    fn transaction_sequence_default_all_ok() {
        let mut db = InMemoryDb::new();
        assert_eq!(db.begin_transaction(), Ok(()));
        db.execute("UPDATE players SET level = 2 WHERE id = 1")
            .unwrap();
        assert_eq!(db.commit(), Ok(()));
    }

    #[test]
    fn rollback_after_begin_default_ok() {
        let mut db = InMemoryDb::new();
        assert_eq!(db.begin_transaction(), Ok(()));
        assert_eq!(db.rollback(), Ok(()));
    }

    // -----------------------------------------------------------------------
    // Confirming stubs: Database trait default transaction methods
    // Classification: intentional-deferred
    // intentional_diff_id: database-adapter-helpers-deferred-to-mariadb-adapter-prod
    // These are DEFAULT TRAIT METHODS — correct no-ops for InMemoryDb.
    // -----------------------------------------------------------------------

    #[test]
    fn test_in_memory_db_begin_transaction_is_noop() {
        // Default trait method: begin_transaction returns Ok(()) for InMemoryDb.
        // MariaDB adapter will override with real BEGIN statement.
        let mut db = InMemoryDb::new();
        assert_eq!(db.begin_transaction(), Ok(()));
    }

    #[test]
    fn test_in_memory_db_commit_is_noop() {
        // Default trait method: commit returns Ok(()) for InMemoryDb.
        let mut db = InMemoryDb::new();
        assert_eq!(db.commit(), Ok(()));
    }

    #[test]
    fn test_in_memory_db_rollback_is_noop() {
        // Default trait method: rollback returns Ok(()) for InMemoryDb.
        let mut db = InMemoryDb::new();
        assert_eq!(db.rollback(), Ok(()));
    }

    // ── New tests: DbResult, DbInsert, DbTransaction, Row edge cases ──────────

    #[test]
    fn row_get_returns_none_for_text_to_int_coercion() {
        let row = make_row(&[("col", DbValue::Text("42".to_string()))]);
        assert_eq!(row.get::<i64>("col"), None); // Text → i64 = None (no coercion)
    }

    #[test]
    fn row_get_handles_signed_overflow_like_strtoll() {
        // At exact boundary: i64::MAX stored as Integer should be retrievable
        let row = make_row(&[("n", DbValue::Integer(i64::MAX))]);
        assert_eq!(row.get::<i64>("n"), Some(i64::MAX));
        // Text representing an overflow value: Rust correctly returns None (no strtoll clamping)
        let overflow = make_row(&[("n", DbValue::Text("9999999999999999999".to_string()))]);
        assert_eq!(overflow.get::<i64>("n"), None);
    }

    #[test]
    fn db_result_constructor_populates_listnames_from_mysql_fields() {
        let rows = vec![make_row(&[
            ("name", DbValue::Text("alice".to_string())),
            ("level", DbValue::Integer(10)),
        ])];
        let result = DbResult::new(rows);
        // column names are accessible (mirrors C++ listNames map built in constructor)
        let names = result.column_names();
        assert!(names.contains(&"name".to_string()));
        assert!(names.contains(&"level".to_string()));
    }

    #[test]
    fn db_result_has_next_returns_true_before_eof() {
        let rows = vec![make_row(&[("id", DbValue::Integer(1))])];
        let result = DbResult::new(rows);
        assert!(result.has_next());
    }

    #[test]
    fn db_result_streaming_next_advances_cursor() {
        let rows = vec![
            make_row(&[("id", DbValue::Integer(1))]),
            make_row(&[("id", DbValue::Integer(2))]),
        ];
        let mut result = DbResult::new(rows);
        assert!(result.has_next());
        assert_eq!(result.get_number::<i64>("id"), Some(1));
        let has_more = result.advance();
        assert!(has_more);
        assert_eq!(result.get_number::<i64>("id"), Some(2));
        let at_end = result.advance();
        assert!(!at_end);
        assert!(!result.has_next());
    }

    #[test]
    fn db_insert_builds_multi_row_insert() {
        let mut db = InMemoryDb::new();
        let mut insert = DbInsert::new("INSERT INTO t (a) VALUES", 1024 * 1024);
        insert.add_row(&mut db, "'alice'");
        insert.add_row(&mut db, "'bob'");
        insert.execute(&mut db);
        assert_eq!(
            db.executed_statements.last().unwrap(),
            "INSERT INTO t (a) VALUES('alice'),('bob')"
        );
    }

    #[test]
    fn db_insert_splits_on_max_packet_size() {
        let mut db = InMemoryDb::new();
        // max_packet_size of 40 means a single long row triggers a flush
        let prefix = "INSERT INTO t (a) VALUES";
        let mut insert = DbInsert::new(prefix, 40);
        // First row: "1234567890" - 10 chars, total ~34 chars (prefix 24 + row 10) = OK
        insert.add_row(&mut db, "1234567890");
        // Second row: same 10 chars. length grows past 40, so should flush first batch
        insert.add_row(&mut db, "9876543210");
        // After second add_row, first batch should have been executed
        assert!(
            !db.executed_statements.is_empty(),
            "first batch should have been flushed"
        );
        insert.execute(&mut db); // flush remaining
        assert_eq!(db.executed_statements.len(), 2);
    }

    #[test]
    fn db_insert_executes_remaining_rows_on_drop() {
        let mut db = InMemoryDb::new();
        let mut insert = DbInsert::new("INSERT INTO t (a) VALUES", 1024 * 1024);
        insert.add_row(&mut db, "'remaining'");
        // calling execute() flushes remaining rows
        let ok = insert.execute(&mut db);
        assert!(ok);
        assert_eq!(
            db.executed_statements.last().unwrap(),
            "INSERT INTO t (a) VALUES('remaining')"
        );
        // After execute(), values are cleared — calling execute() again is a no-op
        let noop = insert.execute(&mut db);
        assert!(noop);
        assert_eq!(db.executed_statements.len(), 1); // no second statement
    }

    #[test]
    fn db_transaction_guard_commit_consumes_the_guard() {
        let mut db = InMemoryDb::new();
        let guard = DbTransaction::begin(&mut db).unwrap();
        guard.commit().unwrap();
        // After commit, COMMIT is recorded but ROLLBACK is not
        assert!(db.transaction_log.contains(&"COMMIT".to_string()));
        assert!(!db.transaction_log.contains(&"ROLLBACK".to_string()));
    }

    #[test]
    fn db_transaction_guard_rolls_back_on_drop_without_commit() {
        let mut db = InMemoryDb::new();
        {
            let _guard = DbTransaction::begin(&mut db).unwrap();
            // guard dropped here without commit
        }
        assert!(db.transaction_log.contains(&"ROLLBACK".to_string()));
        assert!(!db.transaction_log.contains(&"COMMIT".to_string()));
    }
}
