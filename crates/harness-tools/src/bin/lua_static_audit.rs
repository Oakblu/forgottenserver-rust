//! lua-static-audit — emit the static Lua-binding audit report.
//!
//! Parses C++ `luascript.cpp` for every binding registration site, parses
//! the Rust scripting tree for every mlua registration site, diffs the
//! two surfaces, and writes a JSON report consumable by the harness lane
//! driver and ledger writer.
//!
//! Usage:
//!   lua-static-audit \
//!     --cpp  forgottenserver/src/luascript.cpp \
//!     --rust crates/scripting/src \
//!     --out  scripts/harness/reports/lua_bindings-static.json
//!
//! Exit codes:
//!   0 — surfaces match (status PASS)
//!   1 — at least one binding missing or unexpected (status FAIL)
//!   2 — error (bad input, parse failure)

use anyhow::{Context, Result};
use harness_tools::lua_audit::audit;
use std::path::PathBuf;

struct Args {
    cpp: PathBuf,
    rust: PathBuf,
    out: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut cpp: Option<PathBuf> = None;
    let mut rust: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut argv = std::env::args().skip(1);
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--cpp" => cpp = Some(argv.next().context("--cpp needs a path")?.into()),
            "--rust" => rust = Some(argv.next().context("--rust needs a path")?.into()),
            "--out" => out = Some(argv.next().context("--out needs a path")?.into()),
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            unknown => anyhow::bail!("unknown argument: {unknown}"),
        }
    }
    Ok(Args {
        cpp: cpp.context("--cpp is required")?,
        rust: rust.context("--rust is required")?,
        out,
    })
}

fn print_help() {
    eprintln!(
        "lua-static-audit\n\n\
         Usage:\n  \
           lua-static-audit --cpp <path> --rust <dir> [--out <path>]\n\n\
         Flags:\n  \
           --cpp  <path>   Path to forgottenserver/src/luascript.cpp\n  \
           --rust <dir>    Root of the Rust scripting tree to scan\n  \
           --out  <path>   Where to write the JSON report (default: stdout)\n"
    );
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let report = audit(&args.cpp, &args.rust).with_context(|| {
        format!(
            "auditing cpp={} rust={}",
            args.cpp.display(),
            args.rust.display()
        )
    })?;

    let json = serde_json::to_string(&report)?;
    match &args.out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(path, &json)?;
        }
        None => println!("{json}"),
    }

    if report.status == "PASS" {
        std::process::exit(0);
    }
    std::process::exit(1);
}
