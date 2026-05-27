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
}
