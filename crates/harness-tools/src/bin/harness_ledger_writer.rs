//! harness-ledger-writer — propose `MIGRATION_LEDGER.yml` transitions
//! from harness JSON-lines run reports.
//!
//! Usage:
//!   harness-ledger-writer \
//!     --ledger MIGRATION_LEDGER.yml \
//!     --reports scripts/harness/reports/run-<ts>.json \
//!     --out     scripts/harness/reports/ledger_proposal-<ts>.diff
//!
//! Exit codes:
//!   0 — no transitions proposed (ledger stable)
//!   1 — at least one transition proposed (requires human review)
//!   2 — error (bad input, parse failure, etc.)

use anyhow::{Context, Result};
use harness_tools::{ledger::Ledger, report::parse_run_report, transitions::propose};
use std::path::PathBuf;

struct Args {
    ledger: PathBuf,
    reports: Vec<PathBuf>,
    out: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut ledger: Option<PathBuf> = None;
    let mut reports: Vec<PathBuf> = Vec::new();
    let mut out: Option<PathBuf> = None;
    let mut argv = std::env::args().skip(1);
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--ledger" => ledger = Some(argv.next().context("--ledger needs a path")?.into()),
            "--reports" => reports.push(argv.next().context("--reports needs a path")?.into()),
            "--out" => out = Some(argv.next().context("--out needs a path")?.into()),
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            unknown => anyhow::bail!("unknown argument: {unknown}"),
        }
    }
    Ok(Args {
        ledger: ledger.context("--ledger is required")?,
        reports,
        out,
    })
}

fn print_help() {
    eprintln!(
        "harness-ledger-writer\n\n\
         Usage:\n  \
           harness-ledger-writer --ledger <path> [--reports <path>]... [--out <path>]\n\n\
         Flags:\n  \
           --ledger <path>   Path to MIGRATION_LEDGER.yml\n  \
           --reports <path>  Path to a harness run report (repeat for multiple)\n  \
           --out <path>      Where to write the proposed diff (default: stdout)\n"
    );
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let ledger = Ledger::load(&args.ledger)
        .with_context(|| format!("loading ledger {}", args.ledger.display()))?;

    let mut all_records = Vec::new();
    for r in &args.reports {
        let mut recs =
            parse_run_report(r).with_context(|| format!("parsing report {}", r.display()))?;
        all_records.append(&mut recs);
    }

    let transitions = propose(&ledger, &all_records);

    let summary = render_summary(&transitions);
    match &args.out {
        Some(path) => {
            std::fs::write(path, &summary).with_context(|| format!("writing {}", path.display()))?
        }
        None => print!("{summary}"),
    }

    if transitions.is_empty() {
        std::process::exit(0);
    }
    std::process::exit(1);
}

fn render_summary(transitions: &[harness_tools::transitions::Transition]) -> String {
    if transitions.is_empty() {
        return "No transitions proposed — ledger is stable.\n".to_string();
    }
    let mut out = String::new();
    out.push_str("# Proposed MIGRATION_LEDGER.yml transitions\n\n");
    out.push_str(&format!(
        "{} transition(s) proposed:\n\n",
        transitions.len()
    ));
    for t in transitions {
        out.push_str(&format!(
            "- `{}`: {} → {}\n  reason: {}\n",
            t.cpp, t.from, t.to, t.reason
        ));
    }
    out.push_str("\nReview each proposal and edit `MIGRATION_LEDGER.yml` directly if accepted.\n");
    out
}
