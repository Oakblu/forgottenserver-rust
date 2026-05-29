## 1. Cargo Feature Gate

- [x] 1.1 Add `perf_e2e = []` to `[features]` in `crates/e2e/Cargo.toml`
- [x] 1.2 Verify `cargo test --lib --workspace` still passes with no feature flags (nothing new compiles in)

## 2. Perf E2E Test Harness

- [x] 2.1 Create `crates/e2e/tests/perf.rs` with `#![cfg(feature = "perf_e2e")]` guard and module-level `#[ignore]` on all tests
- [x] 2.2 Implement `start_perf_stack()` helper: runs `docker compose -f docker-compose.perf.yml up --build -d` via `std::process::Command`
- [x] 2.3 Implement `wait_for_port(port: u16, timeout_secs: u64)` helper: polls TCP connect in a loop until success or timeout, returns `Result<(), String>`
- [x] 2.4 Implement `stop_perf_stack()` helper: runs `docker compose -f docker-compose.perf.yml down`
- [x] 2.5 Write `test_perf_stack_rust_login_flood` test: start stack → wait for port 7472 (Rust game port) → run perf-bot login_flood 5 bots 15s → assert successes > 0 and error_rate < 0.01 → stop stack
- [x] 2.6 Write `test_perf_stack_cpp_login_flood` test: start stack → wait for port 7372 (C++ game port) → run perf-bot login_flood 5 bots 15s → assert successes > 0 and error_rate < 0.01 → stop stack
- [x] 2.7 Ensure `stop_perf_stack()` is called in teardown even when assertions fail (use a drop guard or `std::panic::catch_unwind`)
- [ ] 2.8 Run `cargo test -p forgottenserver-e2e --features perf_e2e -- --ignored` with Docker available and confirm both tests pass

## 3. Performance Results Reference Data

- [x] 3.1 Create `docs/performance/` directory
- [x] 3.2 Start the perf stack (`docker compose -f docker-compose.perf.yml up --build -d`) and run `cargo run --release -p perf-bot -- --target both --scenario login_flood --bots 20 --duration 60 --output docs/performance/perf-results.json`
- [x] 3.3 Verify `docs/performance/perf-results.json` is valid JSON with `rust` and `cpp` top-level keys

## 4. Performance Comparison README

- [x] 4.1 Write `docs/performance/README.md` with an intro paragraph and a Markdown table sourced from the data in `perf-results.json`; table columns: Scenario | C++ actions/sec | Rust actions/sec | C++ login p50 | Rust login p50 | C++ login p99 | Rust login p99 | C++ error% | Rust error%
- [x] 4.2 Add a "How to update" section in `docs/performance/README.md` explaining the `scripts/update-perf-readme.sh` workflow

## 5. Update Script

- [x] 5.1 Write `scripts/update-perf-readme.sh`: check that the perf stack is reachable (TCP probe ports 7372 and 7472), exit with error if not; run perf-bot `--target both --scenario login_flood` with `--output docs/performance/perf-results.json`; regenerate the table section of `docs/performance/README.md` from the new JSON
- [x] 5.2 Make the script executable (`chmod +x scripts/update-perf-readme.sh`)
- [x] 5.3 Verify the script exits non-zero with a clear message when run without the stack

## 6. Root README Link

- [x] 6.1 Add a "## Performance" section to the root `README.md` with a one-line description and a link to `docs/performance/README.md`

## 7. Verification

- [ ] 7.1 `cargo test --lib --workspace` passes with zero failures (no perf e2e tests run)
- [ ] 7.2 `cargo clippy --workspace --lib --tests -- -D warnings` is clean
- [ ] 7.3 `docs/performance/README.md` renders correctly (table visible, numbers present)
- [ ] 7.4 `jq . docs/performance/perf-results.json` exits 0
