## Why

The benchmarking system has no live integration test coverage and no human-readable summary of results — developers can't verify the test harness works end-to-end without manually spinning up Docker, and there's no reference document showing how Rust compares to C++ at a glance.

## What Changes

- Add a `tests/perf_e2e/` test suite (or `crates/e2e/` extension) that starts both servers via Docker and asserts basic perf-bot scenarios complete successfully; tests are gated behind a `perf_e2e` feature flag / `#[ignore]` so they are skipped in standard CI but runnable locally or in a dedicated job.
- Add a `docs/performance/README.md` with a canonical side-by-side table comparing C++ vs Rust on all perf-bot scenarios; table is generated from a `perf-results.json` fixture (committed reference run) so it renders correctly on GitHub without requiring a live server.

## Capabilities

### New Capabilities

- `perf-e2e-tests`: Docker-backed integration tests that start the perf stack, run a subset of perf-bot scenarios against both servers, and assert success thresholds (no crashes, error rate < 1%, p99 < 500ms).
- `perf-comparison-readme`: A maintained `docs/performance/README.md` with a Markdown table of C++ vs Rust benchmark results, sourced from a committed `docs/performance/perf-results.json` reference run, with instructions for regenerating.

### Modified Capabilities

## Impact

- New test file(s) in `crates/e2e/` or `tests/perf_e2e/` (Rust, `#[ignore]` or feature-gated)
- `Cargo.toml` workspace: no new crate needed if tests live in existing `e2e` crate under a feature flag
- New `docs/performance/README.md` and `docs/performance/perf-results.json`
- `.github/workflows/rust.yml` optional: a separate `perf-e2e` job that runs on-demand (not on every PR)
