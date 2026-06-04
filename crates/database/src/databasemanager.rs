use crate::database::{Database, InMemoryDb};

// ── DatabaseManager ───────────────────────────────────────────────────────────

/// Handles schema migrations and maintenance tasks.
///
/// The real C++ implementation queries MySQL's `information_schema` and runs
/// Lua migration scripts.  This Rust version wraps the `InMemoryDb` with the
/// same logical interface so higher layers can be tested without a real DB.
pub struct DatabaseManager {
    migrations_run: Vec<String>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            migrations_run: Vec::new(),
        }
    }

    /// Returns true if the named table exists in the database.
    pub fn table_exists(&self, db: &InMemoryDb, name: &str) -> bool {
        db.table_exists(name)
    }

    /// Returns the stored `db_version` config value, or 0 for a fresh database.
    pub fn get_database_version(&self, db: &InMemoryDb) -> i64 {
        db.get_config("db_version").unwrap_or(0)
    }

    /// Persist a new database version number.
    pub fn update_version(&self, db: &mut InMemoryDb, version: i64) {
        db.set_config("db_version", version);
    }

    /// No-op for in-memory databases; would run `OPTIMIZE TABLE` against MySQL.
    pub fn optimize_tables(&self, _db: &mut InMemoryDb) -> Result<(), String> {
        Ok(())
    }

    /// Record that a migration SQL script has been executed.
    pub fn run_migration(&mut self, db: &mut InMemoryDb, sql: &str) {
        // In a real DB we would execute the SQL and record the result.
        let _ = db.execute(sql);
        self.migrations_run.push(sql.to_string());
    }

    /// Returns the list of migration scripts that have been run.
    pub fn migrations_run(&self) -> &[String] {
        &self.migrations_run
    }

    /// No-op for in-memory databases.
    ///
    /// Mirrors C++ `DatabaseManager::updateDatabase()` which opens a Lua
    /// state, registers `db` and `result` tables, reads `db_version` from
    /// the database, and runs consecutive migration scripts
    /// `data/migrations/{version}.lua` until one returns `false` from its
    /// `onUpdateDatabase()` callback.
    ///
    /// The real Lua migration runner is deferred to
    /// `forgottenserver-rust-mariadb-adapter-prod` (intentional_differences:
    /// `lua-dispatch-deferred-to-cross-crate-glue`).  The observable
    /// contract tested here is: when the database is already at the latest
    /// version no migrations are run, which for an in-memory database that
    /// has no migration scripts is always the case.
    pub fn update_database(&self, db: &InMemoryDb) -> i64 {
        // Return the current version — no migrations to run in-memory.
        db.get_config("db_version").unwrap_or(0)
    }

    /// Mirrors C++ `DatabaseManager::isDatabaseSetup()`.
    ///
    /// C++ queries `information_schema.tables` for any table in the schema.
    /// Returns `true` if the schema has been bootstrapped (any table exists).
    pub fn is_database_setup(&self, db: &InMemoryDb) -> bool {
        db.has_any_table()
    }

    /// Mirrors C++ `DatabaseManager::getDatabaseConfig(config, value)`.
    ///
    /// C++ queries `SELECT value FROM server_config WHERE config = ?`.
    /// Returns the stored value for the key, or `None` if not found.
    pub fn get_database_config(&self, db: &InMemoryDb, key: &str) -> Option<i64> {
        db.get_config(key)
    }

    /// Mirrors C++ `DatabaseManager::registerDatabaseConfig(config, value)`.
    ///
    /// C++ does INSERT if key absent, UPDATE if key present (upsert).
    pub fn register_database_config(&self, db: &mut InMemoryDb, key: &str, value: i64) {
        db.set_config(key, value);
    }

    /// A single in-order migration step: a target version and the SQL to run.
    ///
    /// Used by `update_database_with_steps` to drive testable migration
    /// sequences without Lua scripting.
    pub fn update_database_with_steps(&self, db: &mut InMemoryDb, steps: &[MigrationStep]) -> i64 {
        let current = db.get_config("db_version").unwrap_or(0);
        let mut sorted: Vec<&MigrationStep> = steps
            .iter()
            .filter(|s| s.version > current)
            .collect();
        sorted.sort_by_key(|s| s.version);
        for step in sorted {
            let _ = db.execute(&step.sql);
            db.set_config("db_version", step.version);
        }
        db.get_config("db_version").unwrap_or(current)
    }
}

/// A single migration step: a target version and the SQL to execute.
///
/// Mirrors C++ `data/migrations/{version}.lua` Lua migration scripts that
/// call `onUpdateDatabase()`.  The Rust in-memory equivalent accepts plain
/// SQL strings so migrations can be tested without a Lua runtime.
#[derive(Debug, Clone)]
pub struct MigrationStep {
    pub version: i64,
    pub sql: String,
}

impl Default for DatabaseManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_manager_new_creates_instance() {
        let _mgr = DatabaseManager::new();
    }

    #[test]
    fn table_exists_returns_false_for_empty_db() {
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        assert!(!mgr.table_exists(&db, "players"));
    }

    #[test]
    fn table_exists_returns_true_after_create_table() {
        let mut db = InMemoryDb::new();
        db.create_table("players");
        let mgr = DatabaseManager::new();
        assert!(mgr.table_exists(&db, "players"));
    }

    #[test]
    fn get_database_version_returns_zero_for_fresh_db() {
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        assert_eq!(mgr.get_database_version(&db), 0);
    }

    #[test]
    fn update_version_then_get_returns_stored_value() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        mgr.update_version(&mut db, 42);
        assert_eq!(mgr.get_database_version(&db), 42);
    }

    #[test]
    fn optimize_tables_returns_ok() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        assert!(mgr.optimize_tables(&mut db).is_ok());
    }

    #[test]
    fn run_migration_records_migration() {
        let mut db = InMemoryDb::new();
        let mut mgr = DatabaseManager::new();
        mgr.run_migration(&mut db, "ALTER TABLE players ADD COLUMN foo INT");
        assert_eq!(mgr.migrations_run().len(), 1);
        assert_eq!(
            mgr.migrations_run()[0],
            "ALTER TABLE players ADD COLUMN foo INT"
        );
    }

    #[test]
    fn run_migration_multiple_migrations_all_recorded() {
        let mut db = InMemoryDb::new();
        let mut mgr = DatabaseManager::new();
        mgr.run_migration(&mut db, "migration_1");
        mgr.run_migration(&mut db, "migration_2");
        assert_eq!(mgr.migrations_run().len(), 2);
    }

    // ── table_exists after multiple tables ────────────────────────────────────

    #[test]
    fn table_exists_is_case_sensitive_distinct_tables() {
        let mut db = InMemoryDb::new();
        db.create_table("Players");
        let mgr = DatabaseManager::new();
        // "players" (lowercase) was not created — should return false
        assert!(!mgr.table_exists(&db, "players"));
        assert!(mgr.table_exists(&db, "Players"));
    }

    #[test]
    fn table_exists_returns_false_for_a_different_table() {
        let mut db = InMemoryDb::new();
        db.create_table("accounts");
        let mgr = DatabaseManager::new();
        assert!(!mgr.table_exists(&db, "players"));
        assert!(mgr.table_exists(&db, "accounts"));
    }

    // ── get_database_version with non-zero value ───────────────────────────────

    #[test]
    fn get_database_version_reflects_updated_version() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        mgr.update_version(&mut db, 7);
        assert_eq!(mgr.get_database_version(&db), 7);
    }

    #[test]
    fn get_database_version_after_multiple_updates_returns_last() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        mgr.update_version(&mut db, 1);
        mgr.update_version(&mut db, 2);
        mgr.update_version(&mut db, 10);
        assert_eq!(mgr.get_database_version(&db), 10);
    }

    // ── optimize_tables is a no-op for in-memory DB ────────────────────────────

    #[test]
    fn optimize_tables_does_not_modify_db_state() {
        let mut db = InMemoryDb::new();
        db.create_table("players");
        let mgr = DatabaseManager::new();
        let before_statements = db.executed_statements.len();
        mgr.optimize_tables(&mut db).unwrap();
        // No SQL should have been executed against the in-memory db
        assert_eq!(db.executed_statements.len(), before_statements);
    }

    // ── run_migration also executes SQL against db ─────────────────────────────

    #[test]
    fn run_migration_executes_sql_against_db() {
        let mut db = InMemoryDb::new();
        let mut mgr = DatabaseManager::new();
        mgr.run_migration(&mut db, "ALTER TABLE players ADD COLUMN vip TINYINT");
        assert_eq!(db.executed_statements.len(), 1);
        assert_eq!(
            db.executed_statements[0],
            "ALTER TABLE players ADD COLUMN vip TINYINT"
        );
    }

    #[test]
    fn run_migration_order_is_preserved_in_migrations_run() {
        let mut db = InMemoryDb::new();
        let mut mgr = DatabaseManager::new();
        mgr.run_migration(&mut db, "step_1");
        mgr.run_migration(&mut db, "step_2");
        mgr.run_migration(&mut db, "step_3");
        assert_eq!(mgr.migrations_run(), &["step_1", "step_2", "step_3"]);
    }

    // ── default() is equivalent to new() ─────────────────────────────────────

    #[test]
    fn default_creates_empty_manager() {
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::default();
        assert_eq!(mgr.get_database_version(&db), 0);
        assert!(mgr.migrations_run().is_empty());
    }

    // ── trigger_exists placeholder (not in C++ header, absent in Rust) ─────────
    // The C++ databasemanager.h does NOT expose triggerExists — it is not a
    // public method.  The audit task referenced it, but after reviewing the
    // header it is confirmed absent.  No implementation gap here.

    // ── is_database_setup (equivalent: table_exists on any table) ─────────────

    #[test]
    fn is_database_setup_false_when_no_tables_exist() {
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        // A fresh in-memory db has no tables → analogous to isDatabaseSetup() = false
        assert!(!mgr.table_exists(&db, "server_config"));
    }

    #[test]
    fn is_database_setup_true_when_server_config_exists() {
        let mut db = InMemoryDb::new();
        db.create_table("server_config");
        let mgr = DatabaseManager::new();
        assert!(mgr.table_exists(&db, "server_config"));
    }

    // ── getDatabaseConfig / registerDatabaseConfig equivalents ────────────────

    #[test]
    fn update_version_acts_as_register_database_config() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        // Mirrors registerDatabaseConfig("db_version", 3)
        mgr.update_version(&mut db, 3);
        // Mirrors getDatabaseConfig("db_version") == Some(3)
        assert_eq!(db.get_config("db_version"), Some(3));
    }

    #[test]
    fn get_database_version_returns_zero_when_config_absent() {
        // Fresh db has no "db_version" key → should default to 0 (not panic)
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        assert_eq!(mgr.get_database_version(&db), 0);
    }

    // ── Task 3.6: DatabaseManager::updateDatabase — up-to-date DB runs no migrations
    // C++ contract: when the db_version matches the latest migration script
    // index, the loop exits immediately (no scripts run, no SQL executed).
    // For the in-memory backend there are no migration scripts, so the
    // function always returns the current db_version unchanged.

    #[test]
    fn update_database_returns_current_version_when_up_to_date() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        // Simulate an already-updated database at version 5
        db.set_config("db_version", 5);
        let version = mgr.update_database(&db);
        assert_eq!(
            version, 5,
            "update_database must return the current version when no migrations are pending"
        );
    }

    #[test]
    fn update_database_returns_zero_for_fresh_db() {
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        // Fresh DB has no db_version key — update_database returns 0 (no migrations to run)
        let version = mgr.update_database(&db);
        assert_eq!(
            version, 0,
            "update_database on a fresh db must return 0 (no migrations pending)"
        );
    }

    #[test]
    fn update_database_does_not_modify_db_state() {
        let mut db = InMemoryDb::new();
        db.set_config("db_version", 3);
        let mgr = DatabaseManager::new();
        let before_stmts = db.executed_statements.len();
        mgr.update_database(&db);
        assert_eq!(
            db.executed_statements.len(),
            before_stmts,
            "update_database must not execute any SQL when no migrations are pending"
        );
        assert_eq!(
            db.get_config("db_version"),
            Some(3),
            "update_database must not mutate db_version when already up-to-date"
        );
    }

    // -----------------------------------------------------------------------
    // Confirming stub: DatabaseManager::optimize_tables
    // Classification: intentional-deferred
    // intentional_diff_id: database-adapter-helpers-deferred-to-mariadb-adapter-prod
    // In-memory DB has no tables to optimize; real MariaDB adapter sends
    // OPTIMIZE TABLE queries to the server.
    // -----------------------------------------------------------------------

    #[test]
    fn test_optimize_tables_is_noop_for_in_memory_db() {
        // DatabaseManager::optimize_tables is a no-op for InMemoryDb —
        // no SQL is executed and the return value is Ok(()).
        let mut db = InMemoryDb::new();
        db.create_table("players");
        let mgr = DatabaseManager::new();
        let before = db.executed_statements.len();
        let result = mgr.optimize_tables(&mut db);
        assert!(result.is_ok(), "optimize_tables must return Ok(())");
        assert_eq!(
            db.executed_statements.len(),
            before,
            "optimize_tables must not execute any SQL against InMemoryDb"
        );
    }

    // ── DatabaseManager::getDatabaseConfig / registerDatabaseConfig ────────────

    #[test]
    fn get_database_config_returns_none_for_missing_key() {
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        assert_eq!(mgr.get_database_config(&db, "nonexistent_key"), None);
    }

    #[test]
    fn get_database_config_returns_value_for_arbitrary_key() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        mgr.register_database_config(&mut db, "experience_rate", 3);
        assert_eq!(mgr.get_database_config(&db, "experience_rate"), Some(3));
    }

    #[test]
    fn register_database_config_persists_arbitrary_key() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        mgr.register_database_config(&mut db, "skill_rate", 7);
        assert_eq!(db.get_config("skill_rate"), Some(7));
    }

    #[test]
    fn register_database_config_upserts_existing_key() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        mgr.register_database_config(&mut db, "db_version", 1);
        // Second call with same key updates the value (upsert semantics)
        mgr.register_database_config(&mut db, "db_version", 5);
        assert_eq!(mgr.get_database_config(&db, "db_version"), Some(5));
    }

    // ── DatabaseManager::isDatabaseSetup ──────────────────────────────────────

    #[test]
    fn is_database_setup_explicit_method_returns_false_when_no_tables() {
        let db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        // C++ isDatabaseSetup() queries information_schema for any table.
        // In-memory equivalent: no tables → not set up.
        assert!(!mgr.is_database_setup(&db));
    }

    #[test]
    fn is_database_setup_explicit_method_returns_true_when_server_config_exists() {
        let mut db = InMemoryDb::new();
        db.create_table("server_config");
        let mgr = DatabaseManager::new();
        assert!(mgr.is_database_setup(&db));
    }

    // ── DatabaseManager::updateDatabase with migration steps ──────────────────

    #[test]
    fn update_database_bumps_db_version_after_each_step() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        db.set_config("db_version", 0);
        let steps = vec![
            MigrationStep { version: 1, sql: "ALTER TABLE players ADD COLUMN vip TINYINT".to_string() },
            MigrationStep { version: 2, sql: "ALTER TABLE players ADD COLUMN premium TINYINT".to_string() },
        ];
        let final_version = mgr.update_database_with_steps(&mut db, &steps);
        assert_eq!(final_version, 2);
        assert_eq!(db.get_config("db_version"), Some(2));
    }

    #[test]
    fn update_database_runs_lua_migrations_in_order() {
        let mut db = InMemoryDb::new();
        let mgr = DatabaseManager::new();
        db.set_config("db_version", 0);
        // Provide steps out-of-order in the slice; update_database_with_steps must sort by version.
        let steps = vec![
            MigrationStep { version: 3, sql: "step_3".to_string() },
            MigrationStep { version: 1, sql: "step_1".to_string() },
            MigrationStep { version: 2, sql: "step_2".to_string() },
        ];
        mgr.update_database_with_steps(&mut db, &steps);
        // Executed SQL must appear in version order: 1, 2, 3
        assert_eq!(db.executed_statements[0], "step_1");
        assert_eq!(db.executed_statements[1], "step_2");
        assert_eq!(db.executed_statements[2], "step_3");
    }
}
