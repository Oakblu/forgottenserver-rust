//! MariaDB backend behind the `Database` trait.
//!
//! Minimal surface for the equivalence harness:
//!   - sqlx (mysql + rustls) connection pool
//!   - blocking `Database` trait impl via an internal tokio runtime
//!   - schema bootstrap (`schema.sql` applied if `players` table absent)
//!   - explicit `begin_transaction` / `commit` / `rollback`
//!
//! Scope explicitly OUT (per
//! `openspec/changes/forgottenserver-rust-equivalence-harness/design.md §5`):
//!   - DBInsert batched-insert builder
//!   - RAII DBTransaction wrapper
//!   - Async DatabaseTasks worker restoration
//!   - Connection-pool tuning beyond `max_connections`
//!   - Lua migration runner
//!   - Reconnect / retry logic
//!
//! These are deferred to `forgottenserver-rust-mariadb-adapter-prod`.

use crate::database::{Database, DbError, DbValue, Row};
use forgottenserver_common::configmanager::{ConfigManager, IntegerKey, StringKey};
use sqlx::mysql::MySqlConnection;
use sqlx::{Column, ConnectOptions, Row as SqlxRow, TypeInfo};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

/// Connection parameters for `MariaDbDatabase`.
#[derive(Debug, Clone)]
pub struct MariaDbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

impl MariaDbConfig {
    /// Read connection parameters from the running ConfigManager.
    ///
    /// Maps:
    ///   `mysqlHost`     → `host`
    ///   `mysqlPort`     → `port` (clamped to u16 range)
    ///   `mysqlUser`     → `user`
    ///   `mysqlPass`     → `password`
    ///   `mysqlDatabase` → `database`
    pub fn from_config_manager(cm: &ConfigManager) -> Self {
        let host = cm.get_string(StringKey::MysqlHost).to_string();
        let user = cm.get_string(StringKey::MysqlUser).to_string();
        let password = cm.get_string(StringKey::MysqlPass).to_string();
        let database = cm.get_string(StringKey::MysqlDb).to_string();
        let port_raw = cm.get_integer(IntegerKey::SqlPort);
        let port = if (1..=65_535).contains(&port_raw) {
            port_raw as u16
        } else {
            3306
        };
        Self {
            host,
            port,
            user,
            password,
            database,
            max_connections: 10,
        }
    }

    /// Build a sqlx connection URL.
    ///
    /// Format: `mysql://<user>:<password>@<host>:<port>/<database>`.
    /// Passwords are URL-encoded so `@`, `:`, `/` round-trip safely.
    pub fn to_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            url_encode(&self.user),
            url_encode(&self.password),
            self.host,
            self.port,
            self.database,
        )
    }
}

/// Percent-encode reserved characters in a single URL segment.
///
/// Only escapes the characters that would break a sqlx mysql URL:
/// `:`, `/`, `@`, `?`, `#`, `&`, `=`, `%`, and space. Everything else
/// passes through.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ':' | '/' | '@' | '?' | '#' | '&' | '=' | '%' | ' ' => {
                let mut buf = [0u8; 4];
                let bytes = ch.encode_utf8(&mut buf).as_bytes();
                for b in bytes {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// MariaDB backend.
///
/// Owns a tokio runtime + a single sqlx `MySqlConnection` (held under
/// a mutex). Trait methods block on the runtime so callers stay in
/// synchronous code (matches the `Database` trait shape used throughout
/// the workspace).
///
/// **Why a single connection, not a pool?** Transactions are part of
/// the trait contract (`begin_transaction` / `commit` / `rollback`).
/// With a pool, each query checks out a fresh connection, so a
/// `BEGIN` on connection A is invisible to an `INSERT` on connection
/// B — the tx is broken. A single shared connection makes BEGIN /
/// INSERT / COMMIT atomic by construction. Concurrency tradeoff is
/// acceptable for the harness scope (single-threaded scenario
/// replay). Real connection pooling lives in
/// `forgottenserver-rust-mariadb-adapter-prod` (see design D8).
pub struct MariaDbDatabase {
    conn: Arc<Mutex<MySqlConnection>>,
    runtime: Arc<Runtime>,
}

impl MariaDbDatabase {
    /// Connect to MariaDB and return a ready adapter.
    ///
    /// Creates a fresh tokio runtime owned by the adapter and a single
    /// MySQL connection.
    pub fn connect(config: &MariaDbConfig) -> Result<Self, DbError> {
        let runtime = Runtime::new().map_err(|_| DbError::ConnectionFailed)?;
        let url = config.to_url();
        let conn = runtime.block_on(async {
            let options = sqlx::mysql::MySqlConnectOptions::from_str(&url)
                .map_err(|e| DbError::QueryError(format!("parse url: {e}")))?;
            options
                .connect()
                .await
                .map_err(|e| DbError::QueryError(format!("connect: {e}")))
        })?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            runtime: Arc::new(runtime),
        })
    }

    /// Apply `schema_sql` if and only if the `players` table is absent.
    ///
    /// Idempotent: running twice against an already-bootstrapped DB is
    /// a no-op. The schema is split on `;` and each statement is
    /// executed in order. Multi-statement SQL files like
    /// `forgottenserver/schema.sql` work as-is.
    pub fn bootstrap_schema_if_needed(&self, schema_sql: &str) -> Result<(), DbError> {
        let runtime = Arc::clone(&self.runtime);
        let conn = Arc::clone(&self.conn);
        let already = runtime.block_on(async {
            let mut g = conn.lock().await;
            sqlx::query("SHOW TABLES LIKE 'players'")
                .fetch_optional(&mut *g)
                .await
                .map_err(|e| DbError::QueryError(format!("check tables: {e}")))
        })?;
        if already.is_some() {
            return Ok(());
        }
        for stmt in schema_sql
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            runtime.block_on(async {
                let mut g = conn.lock().await;
                sqlx::query(stmt)
                    .execute(&mut *g)
                    .await
                    .map(|_| ())
                    .map_err(|e| DbError::QueryError(format!("bootstrap: {e}")))
            })?;
        }
        Ok(())
    }

    /// Return a clone of the runtime handle for integration test setup.
    /// Production callers should not need this.
    #[doc(hidden)]
    pub fn runtime(&self) -> Arc<Runtime> {
        Arc::clone(&self.runtime)
    }
}

impl Database for MariaDbDatabase {
    fn query(&self, sql: &str) -> Result<Vec<Row>, DbError> {
        let runtime = Arc::clone(&self.runtime);
        let conn = Arc::clone(&self.conn);
        let sql = sql.to_string();
        runtime.block_on(async move {
            let mut g = conn.lock().await;
            let rows = sqlx::query(&sql)
                .fetch_all(&mut *g)
                .await
                .map_err(|e| DbError::QueryError(e.to_string()))?;
            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                let mut cols = HashMap::with_capacity(r.columns().len());
                for (i, col) in r.columns().iter().enumerate() {
                    let val = sqlx_to_db_value(&r, i, col);
                    cols.insert(col.name().to_string(), val);
                }
                out.push(Row::new(cols));
            }
            Ok(out)
        })
    }

    fn execute(&mut self, sql: &str) -> Result<u64, DbError> {
        let runtime = Arc::clone(&self.runtime);
        let conn = Arc::clone(&self.conn);
        let sql = sql.to_string();
        runtime.block_on(async move {
            let mut g = conn.lock().await;
            sqlx::query(&sql)
                .execute(&mut *g)
                .await
                .map(|r| r.rows_affected())
                .map_err(|e| DbError::QueryError(e.to_string()))
        })
    }

    fn escape_string(&self, s: &str) -> String {
        // Same escaping as InMemoryDb (MySQL-style): backslash and
        // single-quote are doubled. Matches what callers expect.
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
        self.execute("START TRANSACTION").map(|_| ())
    }

    fn commit(&mut self) -> Result<(), DbError> {
        self.execute("COMMIT").map(|_| ())
    }

    fn rollback(&mut self) -> Result<(), DbError> {
        self.execute("ROLLBACK").map(|_| ())
    }
}

/// Convert a single sqlx column value to our backend-agnostic
/// `DbValue`. Unknown type names fall back to a stringified form so
/// the row at least carries the data through; callers using the
/// typed `Row::get<T>()` API are responsible for matching.
fn sqlx_to_db_value(
    row: &sqlx::mysql::MySqlRow,
    idx: usize,
    col: &sqlx::mysql::MySqlColumn,
) -> DbValue {
    let type_name = col.type_info().name();
    match type_name {
        "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" | "TINYINT UNSIGNED"
        | "SMALLINT UNSIGNED" | "MEDIUMINT UNSIGNED" | "INT UNSIGNED" | "BIGINT UNSIGNED" => {
            let v: Option<i64> = row.try_get(idx).ok();
            v.map(DbValue::Integer).unwrap_or(DbValue::Null)
        }
        "FLOAT" | "DOUBLE" | "DECIMAL" => {
            let v: Option<f64> = row.try_get(idx).ok();
            v.map(DbValue::Float).unwrap_or(DbValue::Null)
        }
        "VARCHAR" | "CHAR" | "TEXT" | "LONGTEXT" | "MEDIUMTEXT" | "TINYTEXT" => {
            let v: Option<String> = row.try_get(idx).ok();
            v.map(DbValue::Text).unwrap_or(DbValue::Null)
        }
        "BLOB" | "BINARY" | "VARBINARY" | "LONGBLOB" | "MEDIUMBLOB" | "TINYBLOB" => {
            // No Binary variant on DbValue yet; hex-encode so the data
            // round-trips through Text. See harness change design doc.
            let v: Option<Vec<u8>> = row.try_get(idx).ok();
            v.map(|b| DbValue::Text(b.iter().map(|x| format!("{x:02X}")).collect()))
                .unwrap_or(DbValue::Null)
        }
        _ => {
            // Fallback: try string. Worst case we drop the value to Null.
            let v: Option<String> = row.try_get(idx).ok();
            v.map(DbValue::Text).unwrap_or(DbValue::Null)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> MariaDbConfig {
        MariaDbConfig {
            host: "h".to_string(),
            port: 3306,
            user: "u".to_string(),
            password: "p".to_string(),
            database: "d".to_string(),
            max_connections: 4,
        }
    }

    #[test]
    fn to_url_includes_all_fields() {
        let url = cfg().to_url();
        assert_eq!(url, "mysql://u:p@h:3306/d");
    }

    #[test]
    fn to_url_percent_encodes_password_special_chars() {
        let mut c = cfg();
        c.password = "p@ss/word:1".to_string();
        let url = c.to_url();
        // @ → %40, / → %2F, : → %3A
        assert!(url.contains("p%40ss%2Fword%3A1"));
    }

    #[test]
    fn to_url_percent_encodes_user_at_sign() {
        let mut c = cfg();
        c.user = "alice@example".to_string();
        let url = c.to_url();
        assert!(url.contains("alice%40example:"));
    }

    #[test]
    fn from_config_manager_reads_string_keys() {
        let mut cm = ConfigManager::new();
        cm.set_string(StringKey::MysqlHost, "db.example");
        cm.set_string(StringKey::MysqlUser, "tibia");
        cm.set_string(StringKey::MysqlPass, "secret");
        cm.set_string(StringKey::MysqlDb, "tibia_rs");
        let c = MariaDbConfig::from_config_manager(&cm);
        assert_eq!(c.host, "db.example");
        assert_eq!(c.user, "tibia");
        assert_eq!(c.password, "secret");
        assert_eq!(c.database, "tibia_rs");
    }

    #[test]
    fn from_config_manager_uses_3306_when_port_out_of_range() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::SqlPort, 0);
        let c = MariaDbConfig::from_config_manager(&cm);
        assert_eq!(c.port, 3306);
    }

    #[test]
    fn from_config_manager_clamps_negative_port_to_default() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::SqlPort, -1);
        let c = MariaDbConfig::from_config_manager(&cm);
        assert_eq!(c.port, 3306);
    }

    #[test]
    fn from_config_manager_uses_3306_when_port_above_u16_max() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::SqlPort, 70_000);
        let c = MariaDbConfig::from_config_manager(&cm);
        assert_eq!(c.port, 3306);
    }

    #[test]
    fn from_config_manager_accepts_valid_port() {
        let mut cm = ConfigManager::new();
        cm.set_integer(IntegerKey::SqlPort, 3307);
        let c = MariaDbConfig::from_config_manager(&cm);
        assert_eq!(c.port, 3307);
    }

    #[test]
    fn url_encode_passes_through_safe_chars() {
        assert_eq!(url_encode("abc123_-."), "abc123_-.");
    }

    #[test]
    fn url_encode_escapes_each_reserved_char() {
        assert_eq!(url_encode(":"), "%3A");
        assert_eq!(url_encode("/"), "%2F");
        assert_eq!(url_encode("@"), "%40");
        assert_eq!(url_encode("?"), "%3F");
        assert_eq!(url_encode("#"), "%23");
        assert_eq!(url_encode("&"), "%26");
        assert_eq!(url_encode("="), "%3D");
        assert_eq!(url_encode("%"), "%25");
        assert_eq!(url_encode(" "), "%20");
    }
}
