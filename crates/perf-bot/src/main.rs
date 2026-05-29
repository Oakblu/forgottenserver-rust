//! perf-bot — headless Tibia load testing binary.
//!
//! Run one or both of a C++ and Rust Tibia server under load using the
//! configured scenario, then print a comparison table and optionally write
//! a JSON report.
//!
//! # Usage
//! ```text
//! perf-bot [OPTIONS]
//!
//! Options:
//!   --target <TARGET>           cpp | rust | both  [default: both]
//!   --scenario <SCENARIO>       Scenario name      [default: full_chaos]
//!   --bots <N>                  Concurrent bots    [default: 50]
//!   --duration <SECS>           Run duration       [default: 60]
//!   --cpp-host <HOST>           C++ server host    [default: 127.0.0.1]
//!   --cpp-port <PORT>           C++ server port    [default: 7372]
//!   --rust-host <HOST>          Rust server host   [default: 127.0.0.1]
//!   --rust-port <PORT>          Rust server port   [default: 7472]
//!   --account-prefix <PREFIX>   Account prefix     [default: bot]
//!   --password <PASS>           Account password   [default: botpass]
//!   --output <PATH>             JSON output file   [optional]
//! ```

use perf_bot::metrics::RunMetrics;
use perf_bot::reporter::{ComparisonReport, TargetReport};
use perf_bot::scenarios::ScenarioConfig;

use clap::Parser;

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "perf-bot",
    about = "Headless Tibia load bot for benchmarking forgottenserver",
    long_about = None
)]
struct Cli {
    /// Which server target to benchmark: "cpp", "rust", or "both"
    #[arg(long, default_value = "both")]
    target: String,

    /// Scenario to run (login_flood | sustained_load | breaking_point |
    /// crowd | magic_storm | npc_flood | monster_swarm | full_chaos)
    #[arg(long, default_value = "full_chaos")]
    scenario: String,

    /// Number of concurrent bots
    #[arg(long, default_value_t = 50)]
    bots: usize,

    /// Duration of the run in seconds
    #[arg(long, default_value_t = 60)]
    duration: u64,

    /// C++ server host
    #[arg(long, default_value = "127.0.0.1")]
    cpp_host: String,

    /// C++ server port
    #[arg(long, default_value_t = 7372)]
    cpp_port: u16,

    /// Rust server host
    #[arg(long, default_value = "127.0.0.1")]
    rust_host: String,

    /// Rust server port
    #[arg(long, default_value_t = 7472)]
    rust_port: u16,

    /// Account name prefix (accounts are named "{prefix}1", "{prefix}2", …)
    #[arg(long, default_value = "bot")]
    account_prefix: String,

    /// Password used for all bot accounts
    #[arg(long, default_value = "botpass")]
    password: String,

    /// Optional path to write a JSON report
    #[arg(long)]
    output: Option<String>,
}

// ---------------------------------------------------------------------------
// Scenario dispatch
// ---------------------------------------------------------------------------

/// Run the named scenario against the given server config.
async fn run_scenario(
    scenario_name: &str,
    config: &ScenarioConfig,
) -> anyhow::Result<RunMetrics> {
    use perf_bot::scenarios;

    match scenario_name {
        "login_flood" => scenarios::login_flood::run(config).await,
        "sustained_load" => scenarios::sustained_load::run(config).await,
        "breaking_point" => scenarios::breaking_point::run(config).await,
        "crowd" | "crowd_walk" => scenarios::crowd::crowd_walk(config).await,
        "crowd_chat" => scenarios::crowd::crowd_chat(config).await,
        "crowd_combat" => scenarios::crowd::crowd_combat(config).await,
        "magic_storm" => scenarios::magic_storm::run(config).await,
        "npc_flood" => scenarios::npc_flood::run(config).await,
        "monster_swarm" => scenarios::monster_swarm::run(config).await,
        "full_chaos" => scenarios::full_chaos::run(config).await,
        other => Err(anyhow::anyhow!(
            "Unknown scenario '{other}'. Valid choices: \
             login_flood, sustained_load, breaking_point, crowd, \
             crowd_chat, crowd_combat, magic_storm, npc_flood, \
             monster_swarm, full_chaos"
        )),
    }
}

// ---------------------------------------------------------------------------
// Single-target summary print
// ---------------------------------------------------------------------------

fn print_single_target(target_name: &str, scenario: &str, metrics: &RunMetrics) {
    let heavy = "\u{2501}"; // ━
    let light = "\u{2500}"; // ─
    let bar_len = 50usize;

    println!("{}", heavy.repeat(bar_len));
    println!(
        " Scenario: {} — {} ({} bots, {:.0}s)",
        scenario, target_name, metrics.successes + metrics.errors, metrics.duration_secs
    );
    println!("{}", light.repeat(bar_len));
    println!(" p50 latency:    {}ms", metrics.percentile(50.0));
    println!(" p95 latency:    {}ms", metrics.percentile(95.0));
    println!(" p99 latency:    {}ms", metrics.percentile(99.0));
    println!(" Actions/sec:    {:.1}", metrics.throughput());
    println!(" Error rate:     {:.2}%", metrics.error_rate() * 100.0);
    println!(" Errors:         {}", metrics.errors);
    const MB: f64 = 1_048_576.0;
    println!(
        " RSS start:      {:.1} MB",
        metrics.rss_start_bytes as f64 / MB
    );
    println!(
        " RSS end:        {:.1} MB",
        metrics.rss_end_bytes as f64 / MB
    );
    println!(
        " Peak RSS:       {:.1} MB",
        metrics.peak_rss_bytes as f64 / MB
    );
    println!("{}", heavy.repeat(bar_len));
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let scenario_name = cli.scenario.as_str();

    match cli.target.as_str() {
        "both" => {
            // Run against C++ first, then Rust
            let cpp_config = ScenarioConfig {
                host: cli.cpp_host.clone(),
                port: cli.cpp_port,
                account_prefix: cli.account_prefix.clone(),
                password: cli.password.clone(),
                bot_count: cli.bots,
                duration_secs: cli.duration,
            };
            let rust_config = ScenarioConfig {
                host: cli.rust_host.clone(),
                port: cli.rust_port,
                account_prefix: cli.account_prefix.clone(),
                password: cli.password.clone(),
                bot_count: cli.bots,
                duration_secs: cli.duration,
            };

            println!("Running scenario '{scenario_name}' against C++ server …");
            let cpp_metrics = run_scenario(scenario_name, &cpp_config).await?;

            println!("Running scenario '{scenario_name}' against Rust server …");
            let rust_metrics = run_scenario(scenario_name, &rust_config).await?;

            let report = ComparisonReport {
                scenario: scenario_name.to_string(),
                bot_count: cli.bots,
                duration_secs: cli.duration as f64,
                cpp: TargetReport::from_metrics("cpp", &cpp_metrics),
                rust: TargetReport::from_metrics("rust", &rust_metrics),
            };

            report.print_table();

            if let Some(path) = &cli.output {
                report.write_json(path)?;
                println!("Report written to {path}");
            }
        }

        "cpp" => {
            let config = ScenarioConfig {
                host: cli.cpp_host.clone(),
                port: cli.cpp_port,
                account_prefix: cli.account_prefix.clone(),
                password: cli.password.clone(),
                bot_count: cli.bots,
                duration_secs: cli.duration,
            };

            println!("Running scenario '{scenario_name}' against C++ server …");
            let metrics = run_scenario(scenario_name, &config).await?;
            print_single_target("cpp", scenario_name, &metrics);

            if let Some(path) = &cli.output {
                let json = serde_json::to_string_pretty(&TargetReport::from_metrics("cpp", &metrics))?;
                std::fs::write(path, json)?;
                println!("Report written to {path}");
            }
        }

        "rust" => {
            let config = ScenarioConfig {
                host: cli.rust_host.clone(),
                port: cli.rust_port,
                account_prefix: cli.account_prefix.clone(),
                password: cli.password.clone(),
                bot_count: cli.bots,
                duration_secs: cli.duration,
            };

            println!("Running scenario '{scenario_name}' against Rust server …");
            let metrics = run_scenario(scenario_name, &config).await?;
            print_single_target("rust", scenario_name, &metrics);

            if let Some(path) = &cli.output {
                let json = serde_json::to_string_pretty(&TargetReport::from_metrics("rust", &metrics))?;
                std::fs::write(path, json)?;
                println!("Report written to {path}");
            }
        }

        other => {
            eprintln!("Unknown target '{other}'. Use: cpp, rust, or both");
            std::process::exit(1);
        }
    }

    Ok(())
}
