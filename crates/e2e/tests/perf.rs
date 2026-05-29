#![cfg(feature = "perf_e2e")]

use std::net::TcpStream;
use std::process::Command;
use std::time::{Duration, Instant};

fn repo_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists() && p.join("docker-compose.perf.yml").exists())
        .expect("could not locate repo root (expected Cargo.toml + docker-compose.perf.yml)")
        .to_path_buf()
}

fn start_perf_stack() {
    let root = repo_root();
    let status = Command::new("docker")
        .args([
            "compose",
            "-f",
            "docker-compose.perf.yml",
            "up",
            "--build",
            "-d",
        ])
        .current_dir(&root)
        .status()
        .expect("failed to spawn 'docker compose up' — is Docker running?");
    assert!(status.success(), "docker compose up failed: {status}");
}

fn stop_perf_stack() {
    let root = repo_root();
    let _ = Command::new("docker")
        .args(["compose", "-f", "docker-compose.perf.yml", "down"])
        .current_dir(&root)
        .status();
}

fn wait_for_port(port: u16, timeout_secs: u64) -> Result<(), String> {
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_secs(1)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Err(format!(
        "port {port} did not become reachable within {timeout_secs}s"
    ))
}

/// Drop guard: stops the perf stack even on panic or assertion failure.
struct StackGuard;

impl Drop for StackGuard {
    fn drop(&mut self) {
        stop_perf_stack();
    }
}

fn run_perf_bot_single(target: &str, port: u16, output_path: &str) {
    let root = repo_root();
    let (host_flag, port_flag) = match target {
        "rust" => ("--rust-host", "--rust-port"),
        "cpp" => ("--cpp-host", "--cpp-port"),
        other => panic!("unknown perf-bot target: {other}"),
    };
    let status = Command::new("cargo")
        .args([
            "run",
            "--release",
            "-p",
            "perf-bot",
            "--",
            "--target",
            target,
            "--scenario",
            "login_flood",
            "--bots",
            "5",
            "--duration",
            "15",
            host_flag,
            "127.0.0.1",
            port_flag,
            &port.to_string(),
            "--output",
            output_path,
        ])
        .current_dir(&root)
        .status()
        .expect("failed to spawn 'cargo run -p perf-bot'");
    assert!(status.success(), "perf-bot exited non-zero: {status}");
}

// ── Tests ─────────────────────────────────────────────────────────────────────
//
// Run with:
//   cargo test -p forgottenserver-e2e --features perf_e2e -- --ignored --test-threads=1
//
// --test-threads=1 is required: both tests start/stop the same Docker stack.
// First build of the C++ upstream image can take 10–15 minutes.

#[test]
#[ignore = "requires Docker + docker-compose.perf.yml stack (see module comment)"]
fn test_perf_stack_rust_login_flood() {
    start_perf_stack();
    let _guard = StackGuard; // stops stack on drop, even on panic

    wait_for_port(7472, 300).expect("Rust game port 7472 not reachable within 300s");

    let output = std::env::temp_dir().join("perf_e2e_rust.json");
    let output_str = output.to_str().unwrap();
    run_perf_bot_single("rust", 7472, output_str);

    let content = std::fs::read_to_string(output_str).expect("perf-bot did not write JSON output");
    let report: serde_json::Value =
        serde_json::from_str(&content).expect("perf-bot output is not valid JSON");

    // --target rust writes a bare TargetReport: { "actions_per_sec": ..., "error_rate_pct": ... }
    let actions = report["actions_per_sec"].as_f64().unwrap_or(0.0);
    let error_rate = report["error_rate_pct"].as_f64().unwrap_or(100.0);

    assert!(
        actions > 0.0,
        "expected >0 actions/sec for Rust, got {actions}; report: {report}"
    );
    assert!(
        error_rate < 1.0,
        "expected <1% error rate for Rust, got {error_rate}%; report: {report}"
    );
}

#[test]
#[ignore = "requires Docker + docker-compose.perf.yml stack (see module comment)"]
fn test_perf_stack_cpp_login_flood() {
    start_perf_stack();
    let _guard = StackGuard;

    wait_for_port(7372, 300).expect("C++ game port 7372 not reachable within 300s");

    let output = std::env::temp_dir().join("perf_e2e_cpp.json");
    let output_str = output.to_str().unwrap();
    run_perf_bot_single("cpp", 7372, output_str);

    let content = std::fs::read_to_string(output_str).expect("perf-bot did not write JSON output");
    let report: serde_json::Value =
        serde_json::from_str(&content).expect("perf-bot output is not valid JSON");

    // --target cpp writes a bare TargetReport: { "actions_per_sec": ..., "error_rate_pct": ... }
    let actions = report["actions_per_sec"].as_f64().unwrap_or(0.0);
    let error_rate = report["error_rate_pct"].as_f64().unwrap_or(100.0);

    assert!(
        actions > 0.0,
        "expected >0 actions/sec for C++, got {actions}; report: {report}"
    );
    assert!(
        error_rate < 1.0,
        "expected <1% error rate for C++, got {error_rate}%; report: {report}"
    );
}
