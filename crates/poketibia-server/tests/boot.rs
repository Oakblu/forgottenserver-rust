//! Integration tests for the `poketibia-server` binary crate's boot sequence.
//!
//! These tests exercise the 18-step `mainLoader()` equivalent end-to-end using
//! the real `data/` directory (symlinked to `apps/poketibia/forgottenserver/data`)
//! and a minimal fixture `config.lua`. They run without a MariaDB instance —
//! database wiring degrades gracefully per the design's "PARTIAL outcomes are
//! surfaced, not panics" rule.

use std::path::PathBuf;

use poketibia_server::boot::{
    initialise_modules, install_signal_handlers, request_shutdown, shutdown_requested,
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn workspace_data_dir() -> PathBuf {
    // crates/poketibia-server/Cargo.toml's parent's parent is the workspace
    // root; `data/` is the symlink to ../forgottenserver/data.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("data")
}

#[test]
fn initialise_modules_loads_config_from_fixture() {
    let config_path = fixtures_dir().join("config.lua");
    let data_dir = workspace_data_dir();

    let modules = initialise_modules(&config_path, &data_dir)
        .expect("initialise_modules must succeed against fixture config + real data dir");

    assert!(
        !modules.game_data.items.is_empty(),
        "items registry must contain at least one entry after items.otb load"
    );
}

#[test]
fn initialise_modules_loads_spells_registry() {
    let config_path = fixtures_dir().join("config.lua");
    let data_dir = workspace_data_dir();

    let modules =
        initialise_modules(&config_path, &data_dir).expect("initialise_modules must succeed");

    // Note: modern forgottenserver's `spells.xml` is intentionally empty
    // (entries moved into `data/scripts/spells/*.lua`). The loader must
    // still produce a valid empty registry — we assert is_empty rather than
    // is_non_empty, which is the actual contract for the bundled `data/`.
    assert!(
        modules.game_data.spells.is_empty(),
        "bundled spells.xml is empty by design; registry must reflect that"
    );
}

#[test]
fn initialise_modules_loads_weapons_registry() {
    let config_path = fixtures_dir().join("config.lua");
    let data_dir = workspace_data_dir();

    let modules =
        initialise_modules(&config_path, &data_dir).expect("initialise_modules must succeed");

    assert!(
        !modules.game_data.weapons.is_empty(),
        "weapon registry must have weapons loaded (weapons.xml is non-trivial; got {})",
        modules.game_data.weapons.len()
    );
}

#[test]
fn initialise_modules_missing_config_returns_clean_error() {
    let config_path = PathBuf::from("/nonexistent/config.lua");
    let data_dir = workspace_data_dir();

    let err = initialise_modules(&config_path, &data_dir)
        .err()
        .expect("missing config must error, not panic");

    let msg = err.to_string();
    assert!(
        msg.contains("config") || msg.contains("Config"),
        "error message must mention config (got: {msg})"
    );
}

#[test]
fn initialise_modules_missing_items_otb_returns_clean_error() {
    let config_path = fixtures_dir().join("config.lua");
    let data_dir = PathBuf::from("/nonexistent/data");

    let err = initialise_modules(&config_path, &data_dir)
        .err()
        .expect("missing items.otb must error, not panic");

    // The exact phrasing comes from forgottenserver-map; we only require the
    // path or 'items' is mentioned.
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("items") || msg.contains("otb") || msg.contains("nonexistent"),
        "error message must mention the missing data path or items.otb (got: {msg})"
    );
}

#[test]
fn signal_install_then_request_shutdown_flips_flag() {
    install_signal_handlers();
    assert!(
        !shutdown_requested(),
        "shutdown must not be requested before request_shutdown()"
    );
    request_shutdown();
    assert!(
        shutdown_requested(),
        "request_shutdown() must flip the global shutdown flag (mirrors SIGINT/SIGTERM)"
    );
}
