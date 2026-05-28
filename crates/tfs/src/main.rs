//! `tfs` — binary entrypoint.
//!
//! Mirrors the C++ `forgottenserver/src/otserv.cpp` `main()` flow. Per the
//! design's "PARTIAL outcomes surface, not panics" rule, each subsystem's
//! init failure is surfaced as a clean error message + non-zero exit, not
//! a panic. See `boot.rs` for the per-step mapping.

use std::path::PathBuf;
use std::process::ExitCode;

use tfs::boot;
use tfs::boot::DbBackend;

fn print_banner() {
    println!("The Forgotten Server (Rust port)");
    println!();
    println!(
        "Compiled with rustc {} for {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    );
    println!();
}

struct CliArgs {
    config_path: PathBuf,
    data_dir: PathBuf,
    db_backend: DbBackend,
}

fn parse_cli() -> CliArgs {
    let mut args = std::env::args().skip(1);
    let mut config_path = PathBuf::from("config.lua");
    let mut data_dir = PathBuf::from("data");
    let mut db_backend = DbBackend::Auto;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" | "-c" => {
                if let Some(v) = args.next() {
                    config_path = PathBuf::from(v);
                }
            }
            "--data" | "-d" => {
                if let Some(v) = args.next() {
                    data_dir = PathBuf::from(v);
                }
            }
            "--db-backend" => {
                let v = args.next().unwrap_or_else(|| {
                    eprintln!("--db-backend needs a value (auto | in-memory | mariadb)");
                    std::process::exit(2);
                });
                match DbBackend::parse(&v) {
                    Ok(b) => db_backend = b,
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(2);
                    }
                }
            }
            "--help" | "-h" => {
                println!(
                    "Usage: tfs [--config <path>] [--data <dir>] \
                     [--db-backend auto|in-memory|mariadb]\n\
                     \n\
                     Options:\n\
                       --config <path>     Path to config.lua (default: ./config.lua)\n\
                       --data   <dir>      Path to data/ directory (default: ./data)\n\
                       --db-backend <kind> Database backend (default: auto — mariadb when\n\
                                           config provides credentials, in-memory otherwise)\n\
                       --help              Show this help and exit\n"
                );
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {arg}. Use --help for usage.");
                std::process::exit(2);
            }
        }
    }

    CliArgs {
        config_path,
        data_dir,
        db_backend,
    }
}

fn main() -> ExitCode {
    print_banner();
    let cli = parse_cli();

    if let Err(e) = boot::validate_config_path(&cli.config_path) {
        eprintln!("[FATAL] {e:#}");
        return ExitCode::from(1);
    }

    let modules = match boot::initialise_modules(&cli.config_path, &cli.data_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[FATAL] Failed to initialise modules: {e:#}");
            return ExitCode::from(1);
        }
    };

    let backend = boot::resolve_backend(cli.db_backend, &modules.config);
    println!(">> Using database backend: {backend:?}");
    let _database = match boot::connect_database(backend, &modules.config) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[FATAL] Failed to connect database: {e:#}");
            return ExitCode::from(1);
        }
    };

    println!(
        ">> Loaded {} items, {} spells, {} weapons, {} NPCs, {} Lua scripts.",
        modules.game_data.items.len(),
        modules.game_data.spells.len(),
        modules.game_data.weapons.len(),
        modules.game_data.npcs.len(),
        modules.scripts_loaded
    );

    boot::install_signal_handlers();

    if let Err(e) = boot::start_listeners(&modules) {
        eprintln!("[FATAL] Failed to start listeners: {e:#}");
        return ExitCode::from(1);
    }

    println!(">> Forgotten Server Online! (Rust port)");
    println!(">> Send SIGINT (Ctrl-C) or SIGTERM to shut down.");

    boot::wait_for_shutdown();

    println!(">> Shutdown requested; exiting cleanly.");
    ExitCode::from(0)
}
