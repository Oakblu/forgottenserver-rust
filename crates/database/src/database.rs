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
}
