## ADDED Requirements

### Requirement: Perf e2e tests are feature-gated and skipped by default
The test module SHALL be compiled only when the `perf_e2e` Cargo feature is enabled. Tests SHALL additionally carry `#[ignore]` so they are skipped even when the feature is present unless the caller passes `-- --ignored`. Standard `cargo test --lib --workspace` and `cargo test -p forgottenserver-e2e --features e2e` SHALL complete without starting Docker or touching the perf stack.

#### Scenario: Standard workspace test run excludes perf e2e tests
- **WHEN** `cargo test --lib --workspace` is run without the `perf_e2e` feature
- **THEN** no Docker container is started and no perf-related test appears in output

#### Scenario: Explicit feature flag enables perf e2e tests
- **WHEN** `cargo test -p forgottenserver-e2e --features perf_e2e -- --ignored` is run with Docker available
- **THEN** perf e2e tests compile, run, and report pass or fail

### Requirement: Perf e2e test starts the perf Docker stack before running
The test harness SHALL bring up `docker-compose.perf.yml` (both `forgottenserver-rust` and `forgottenserver-cpp` services plus `perf-db`) before asserting anything. It SHALL poll both game ports (7372 for C++, 7472 for Rust) until they accept TCP connections or a 5-minute timeout elapses, failing the test on timeout.

#### Scenario: Both servers become reachable within timeout
- **WHEN** the perf stack starts successfully
- **THEN** TCP connects to 127.0.0.1:7372 and 127.0.0.1:7472 both succeed within 300 seconds

#### Scenario: Timeout produces a clear error
- **WHEN** a server does not become reachable within 300 seconds
- **THEN** the test fails with a message identifying which port timed out

### Requirement: Perf e2e smoke test runs login_flood scenario against both servers
After the stack is ready, the test SHALL invoke `cargo run --release -p perf-bot` with `--scenario login_flood --bots 5 --duration 15` targeting each server independently. The test SHALL assert: exit code 0, successes > 0, error_rate < 0.01 (1%).

#### Scenario: Rust server smoke passes
- **WHEN** perf-bot runs login_flood with 5 bots for 15 seconds against the Rust server (port 7472)
- **THEN** at least 1 action succeeds and error rate is below 1%

#### Scenario: C++ server smoke passes
- **WHEN** perf-bot runs login_flood with 5 bots for 15 seconds against the C++ server (port 7372)
- **THEN** at least 1 action succeeds and error rate is below 1%

### Requirement: Perf e2e test tears down the stack after completion
The test harness SHALL run `docker compose -f docker-compose.perf.yml down` after all assertions, regardless of pass or fail. Named volumes SHALL not be deleted (use `down` without `-v`) to allow debugging on failure.

#### Scenario: Stack is stopped on test pass
- **WHEN** the perf e2e test completes successfully
- **THEN** `docker compose -f docker-compose.perf.yml down` is called and all perf containers stop

#### Scenario: Stack is stopped on test failure
- **WHEN** the perf e2e test fails an assertion
- **THEN** `docker compose -f docker-compose.perf.yml down` is still called before the test exits

### Requirement: Cargo.toml declares the perf_e2e feature in the e2e crate
`crates/e2e/Cargo.toml` SHALL declare a `perf_e2e` feature. The feature SHALL have no required dependencies beyond what is already in `[dev-dependencies]` (Docker is a system dependency, not a Cargo one).

#### Scenario: Feature declared in Cargo.toml
- **WHEN** `cat crates/e2e/Cargo.toml` is inspected
- **THEN** `[features]` section contains `perf_e2e = []`
