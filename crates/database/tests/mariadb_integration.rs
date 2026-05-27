//! Integration tests for the MariaDB backend.
//!
//! These tests require:
//!   1. The `mariadb-integration` feature enabled.
//!   2. A running `db` container with the `tibia_test`
//!      logical database (provisioned by
//!      `docker/mariadb-init/00-init-tibia-dbs.sh`).
//!   3. Port 13306 reachable from the test host (mapped by docker-
//!      compose).
//!
//! Run with:
//!   docker compose up -d db
//!   cargo test -p forgottenserver-database --features mariadb-integration \
//!       --test mariadb_integration -- --test-threads=1
//!
//! **`--test-threads=1` is required** because every test connects to the
//! same `tibia_test` database and several use `DELETE FROM accounts` as a
//! between-test reset. Running in parallel causes the reset of one test
//! to wipe the inserts of another mid-flight. The Phase 4 minimum-viable
//! adapter does not include test-level isolation primitives — the
//! follow-up `forgottenserver-rust-mariadb-adapter-prod` change will add
//! per-test isolation (e.g. per-test schema or `serial_test`).

#![cfg(feature = "mariadb-integration")]

use forgottenserver_database::database::{Database, DbValue};
use forgottenserver_database::mariadb::{MariaDbConfig, MariaDbDatabase};

fn test_config() -> MariaDbConfig {
    MariaDbConfig {
        host: "127.0.0.1".to_string(),
        port: 13306,
        user: "forgottenserver".to_string(),
        password: "forgottenserver".to_string(),
        database: "tibia_test".to_string(),
        max_connections: 4,
    }
}

fn connect() -> MariaDbDatabase {
    MariaDbDatabase::connect(&test_config())
        .expect("MariaDbDatabase::connect failed — is the db container up? (docker compose up -d db)")
}

/// Wipe a table; used to reset between tests that need committed state.
fn truncate(db: &mut MariaDbDatabase, table: &str) {
    db.execute(&format!("DELETE FROM `{table}`")).unwrap();
}

// ─── 4.5.1 Schema idempotency ────────────────────────────────────────────────

#[test]
fn schema_bootstrap_is_idempotent_when_schema_already_present() {
    let db = connect();
    // The test DB is bootstrapped by the docker init script. Re-running
    // bootstrap should detect the `players` table and short-circuit.
    let schema = include_str!("../../../schema.sql");
    db.bootstrap_schema_if_needed(schema).unwrap();
    // Second call: still no-op, still no error.
    db.bootstrap_schema_if_needed(schema).unwrap();
}

// ─── 4.5.2 Round-trip a row through the trait surface ───────────────────────

#[test]
fn execute_then_query_returns_inserted_row() {
    let mut db = connect();
    truncate(&mut db, "accounts");
    let affected = db
        .execute(
            "INSERT INTO accounts (name, password, email, creation) \
             VALUES ('rust_test_acct', 'sha1_hash_placeholder', 'rust@test.local', 1700000000)",
        )
        .unwrap();
    assert_eq!(affected, 1);

    let rows = db
        .query("SELECT name, email, creation FROM accounts WHERE name = 'rust_test_acct'")
        .unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.get::<String>("name").as_deref(), Some("rust_test_acct"));
    assert_eq!(
        row.get::<String>("email").as_deref(),
        Some("rust@test.local")
    );
    assert_eq!(row.get::<i64>("creation"), Some(1_700_000_000));

    // cleanup
    truncate(&mut db, "accounts");
}

// ─── 4.5.3 escape_string protects single quotes ─────────────────────────────

#[test]
fn escape_string_round_trip_protects_single_quote() {
    let mut db = connect();
    truncate(&mut db, "accounts");
    let raw = "O'Brien";
    let escaped = db.escape_string(raw);
    let sql = format!(
        "INSERT INTO accounts (name, password, email, creation) \
         VALUES ('{escaped}', 'pw', 'x@x', 0)"
    );
    db.execute(&sql).unwrap();
    let rows = db
        .query(&format!(
            "SELECT name FROM accounts WHERE name = '{}'",
            db.escape_string(raw)
        ))
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>("name").as_deref(), Some("O'Brien"));
    truncate(&mut db, "accounts");
}

// ─── 4.5.4 Transaction commit + read ────────────────────────────────────────

#[test]
fn transaction_commit_persists_row() {
    let mut db = connect();
    truncate(&mut db, "accounts");

    db.begin_transaction().unwrap();
    db.execute(
        "INSERT INTO accounts (name, password, email, creation) \
         VALUES ('tx_commit', 'pw', '@', 0)",
    )
    .unwrap();
    db.commit().unwrap();

    let rows = db
        .query("SELECT name FROM accounts WHERE name = 'tx_commit'")
        .unwrap();
    assert_eq!(rows.len(), 1);
    truncate(&mut db, "accounts");
}

// ─── 4.5.5 Transaction rollback discards row ────────────────────────────────

#[test]
fn transaction_rollback_discards_row() {
    let mut db = connect();
    truncate(&mut db, "accounts");

    db.begin_transaction().unwrap();
    db.execute(
        "INSERT INTO accounts (name, password, email, creation) \
         VALUES ('tx_rollback', 'pw', '@', 0)",
    )
    .unwrap();
    db.rollback().unwrap();

    let rows = db
        .query("SELECT name FROM accounts WHERE name = 'tx_rollback'")
        .unwrap();
    assert_eq!(
        rows.len(),
        0,
        "rolled-back row must NOT be visible after rollback"
    );
}

// ─── Bonus: escape_blob produces a hex literal accepted by MariaDB ──────────

#[test]
fn escape_blob_hex_literal_round_trips_through_database() {
    let mut db = connect();
    truncate(&mut db, "account_storage");
    // account_storage(account_id INT, key INT, value VARCHAR(255))
    // Insert a value whose textual form is the hex literal — proves the
    // server accepts the syntax.
    let blob = vec![0x01u8, 0x02, 0x03, 0xff];
    let hex_literal = db.escape_blob(&blob);
    assert_eq!(hex_literal, "X'010203FF'");
    // Validate MariaDB parses the syntax by asking it to evaluate the
    // literal — this would error if escape_blob produced bad syntax.
    let rows = db
        .query(&format!("SELECT HEX({hex_literal}) AS h"))
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String>("h").as_deref(), Some("010203FF"));
}

// ─── Sanity: query returns null for missing column type ─────────────────────

#[test]
fn query_returns_dbvalue_null_for_actual_null_column() {
    let mut db = connect();
    truncate(&mut db, "accounts");
    db.execute(
        "INSERT INTO accounts (name, password, email, creation) \
         VALUES ('null_test', 'pw', '@', 0)",
    )
    .unwrap();
    // `secret` defaults to NULL.
    let rows = db
        .query("SELECT secret FROM accounts WHERE name = 'null_test'")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_raw("secret"), Some(&DbValue::Null));
    truncate(&mut db, "accounts");
}
