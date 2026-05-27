# config.lua / Docker Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align Docker credentials and database name to `config.lua` so `docker compose up` works out of the box, and give users a clear actionable error when `config.lua` is missing.

**Architecture:** Three independent edits — config file password alignment, Docker database name alignment, and a new `validate_config_path` guard in the boot sequence. No new crates, no new abstractions.

**Tech Stack:** Rust (anyhow, std::path), Bash (MariaDB init script), Lua (config files), Docker Compose

---

## File Map

| File | Change |
|---|---|
| `config.lua` | `mysqlPass = ""` → `mysqlPass = "forgottenserver"` |
| `config.lua.dist` | same |
| `crates/tfs/tests/fixtures/config.lua` | same |
| `docker-compose.yml` | `MARIADB_DATABASE: tibia_rs` → `MARIADB_DATABASE: forgottenserver` |
| `docker/mariadb-init/00-init-tibia-dbs.sh` | add `forgottenserver` to database loop |
| `crates/tfs/src/boot.rs` | add `pub fn validate_config_path(path: &Path) -> Result<()>` |
| `crates/tfs/src/main.rs` | call `validate_config_path` before `initialise_modules` |

---

## Task 1: Align config.lua passwords

**Files:**
- Modify: `config.lua:82`
- Modify: `config.lua.dist:82`
- Modify: `crates/tfs/tests/fixtures/config.lua` (the `mysqlPass` line)

- [ ] **Step 1: Update config.lua**

Open `config.lua`, find line 82:
```lua
mysqlPass = ""
```
Change to:
```lua
mysqlPass = "forgottenserver"
```

- [ ] **Step 2: Update config.lua.dist**

Open `config.lua.dist`, find the same line (`mysqlPass = ""`).
Change to:
```lua
mysqlPass = "forgottenserver"
```

- [ ] **Step 3: Update test fixture**

Open `crates/tfs/tests/fixtures/config.lua`, find:
```lua
mysqlPass = "forgottenserver"
```
It already says `"forgottenserver"` — verify this is the case. If it says `""`, change it to `"forgottenserver"`. No functional change either way for tests (no MariaDB connection made in unit tests).

- [ ] **Step 4: Verify no other config references the old empty password as a hard-coded credential**

```bash
grep -rn 'mysqlPass' config.lua config.lua.dist crates/tfs/tests/fixtures/config.lua
```

Expected output (all three lines should now say `"forgottenserver"`):
```
config.lua:82:mysqlPass = "forgottenserver"
config.lua.dist:82:mysqlPass = "forgottenserver"
crates/tfs/tests/fixtures/config.lua:8:mysqlPass = "forgottenserver"
```

- [ ] **Step 5: Commit**

```bash
git add config.lua config.lua.dist crates/tfs/tests/fixtures/config.lua
git commit -m "config: align mysqlPass to Docker credential (forgottenserver)"
```

---

## Task 2: Align Docker to forgottenserver database name

**Files:**
- Modify: `docker-compose.yml:10`
- Modify: `docker/mariadb-init/00-init-tibia-dbs.sh`

- [ ] **Step 1: Update docker-compose.yml**

Open `docker-compose.yml`, find line 10:
```yaml
      MARIADB_DATABASE: tibia_rs
```
Change to:
```yaml
      MARIADB_DATABASE: forgottenserver
```

- [ ] **Step 2: Update init script — header comment**

Open `docker/mariadb-init/00-init-tibia-dbs.sh`. Replace the header comment block and the echo/loop so the full file reads:

```bash
#!/bin/bash
#
# 00-init-tibia-dbs.sh — Initialize databases for the forgottenserver stack.
#
# Mounted into the db container at
# /docker-entrypoint-initdb.d/00-init-tibia-dbs.sh. Runs once on first
# container start, creates the databases used by the server and harness,
# grants permissions to the existing `forgottenserver` user, and applies
# schema.sql to each.
#
# forgottenserver — production/dev database (matches config.lua default)
# tibia_cpp       — C++ reference server database (harness lane)
# tibia_rs        — Rust port database (harness lane)
# tibia_test      — E2E test database (harness lane)
#
# Schema source is mounted at /opt/tfs-schema.sql by docker-compose.yml.

set -euo pipefail

SCHEMA="/opt/tfs-schema.sql"

if [ ! -f "$SCHEMA" ]; then
  echo "ERROR: schema not found at $SCHEMA" >&2
  exit 1
fi

echo "Initializing forgottenserver, tibia_cpp, tibia_rs, and tibia_test databases..."

for db in forgottenserver tibia_cpp tibia_rs tibia_test; do
  echo "  → creating database '$db'"
  mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" -e "CREATE DATABASE IF NOT EXISTS \`$db\` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
  echo "  → granting permissions on '$db' to 'forgottenserver'"
  mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" -e "GRANT ALL PRIVILEGES ON \`$db\`.* TO 'forgottenserver'@'%';"
  echo "  → applying schema to '$db'"
  mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" "$db" < "$SCHEMA"
done

mariadb -uroot -p"$MARIADB_ROOT_PASSWORD" -e "FLUSH PRIVILEGES;"
echo "Database initialization complete."
```

- [ ] **Step 3: Verify diff looks correct**

```bash
git diff docker-compose.yml docker/mariadb-init/00-init-tibia-dbs.sh
```

Expected: `MARIADB_DATABASE` changed to `forgottenserver`; loop now includes `forgottenserver tibia_cpp tibia_rs tibia_test`.

- [ ] **Step 4: Commit**

```bash
git add docker-compose.yml docker/mariadb-init/00-init-tibia-dbs.sh
git commit -m "docker: create forgottenserver database matching config.lua defaults"
```

---

## Task 3: Add validate_config_path with helpful missing-file error

**Files:**
- Modify: `crates/tfs/src/boot.rs` (add function + tests)
- Modify: `crates/tfs/src/main.rs` (call the new function)

- [ ] **Step 1: Write the failing tests in boot.rs**

Open `crates/tfs/src/boot.rs`. At the bottom of the `#[cfg(test)] mod tests` block (after the last `}` inside the tests module but before the closing `}` of the module), add:

```rust
    #[test]
    fn validate_config_path_missing_file_returns_err_with_hint() {
        let result = validate_config_path(std::path::Path::new("/nonexistent/xyz/config.lua"));
        let err = result.expect_err("expected Err for nonexistent path");
        let msg = format!("{err}");
        assert!(
            msg.contains("Config file not found"),
            "error should name the problem: {msg}"
        );
        assert!(
            msg.contains("cp config.lua.dist config.lua"),
            "error should include recovery command: {msg}"
        );
    }

    #[test]
    fn validate_config_path_existing_file_returns_ok() {
        // Use a path that is guaranteed to exist without extra dependencies.
        let result = validate_config_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        assert!(result.is_ok(), "expected Ok for existing path, got: {result:?}");
    }
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test --lib -p tfs validate_config_path 2>&1 | tail -20
```

Expected: compile error — `validate_config_path` is not yet defined.

- [ ] **Step 3: Implement validate_config_path in boot.rs**

In `crates/tfs/src/boot.rs`, add this function after the `initialise_modules` function (around line 103, before the signal-handling section):

```rust
/// Check that `config_path` exists before attempting to load it.
///
/// Returns a descriptive `Err` with recovery instructions if the file is
/// missing, so users see an actionable message instead of a raw IO error.
pub fn validate_config_path(config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        return Err(anyhow!(
            "Config file not found: {}\nTo fix: copy config.lua.dist to config.lua and edit the settings.\n  cp config.lua.dist config.lua",
            config_path.display()
        ));
    }
    Ok(())
}
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
cargo test --lib -p tfs validate_config_path 2>&1 | tail -10
```

Expected:
```
test boot::tests::validate_config_path_existing_file_returns_ok ... ok
test boot::tests::validate_config_path_missing_file_returns_err_with_hint ... ok
```

- [ ] **Step 5: Call validate_config_path from main.rs**

Open `crates/tfs/src/main.rs`. Find the `main()` function. After `let cli = parse_cli();` and before the `initialise_modules` call, add:

```rust
    if let Err(e) = boot::validate_config_path(&cli.config_path) {
        eprintln!("[FATAL] {e}");
        return ExitCode::from(1);
    }
```

The relevant section of `main()` should now read:

```rust
fn main() -> ExitCode {
    print_banner();
    let cli = parse_cli();

    if let Err(e) = boot::validate_config_path(&cli.config_path) {
        eprintln!("[FATAL] {e}");
        return ExitCode::from(1);
    }

    let modules = match boot::initialise_modules(&cli.config_path, &cli.data_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[FATAL] Failed to initialise modules: {e:#}");
            return ExitCode::from(1);
        }
    };
    // ... rest unchanged
```

- [ ] **Step 6: Run the full tfs test suite**

```bash
cargo test --lib -p tfs 2>&1 | tail -20
```

Expected: all tests pass, no compile errors.

- [ ] **Step 7: Run clippy**

```bash
cargo clippy -p tfs --lib --tests -- -D warnings 2>&1 | tail -20
```

Expected: no warnings.

- [ ] **Step 8: Commit**

```bash
git add crates/tfs/src/boot.rs crates/tfs/src/main.rs
git commit -m "feat(tfs): validate config path at startup with actionable error message"
```
