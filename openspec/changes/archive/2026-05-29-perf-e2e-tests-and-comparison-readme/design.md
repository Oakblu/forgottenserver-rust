## Context

The benchmarking system (Criterion micro-benchmarks + perf-bot load bot + docker-compose.perf.yml) is fully implemented but has no automated test coverage that verifies it works end-to-end with real servers. The existing `crates/e2e/` crate already uses the `#[cfg(feature = "e2e")]` gate, has a `ServerFixture` abstraction, and runs via `cargo test -p forgottenserver-e2e --features e2e`. This is the natural home for Docker-backed perf integration tests. Additionally, there is no persistent record of actual benchmark results — developers must run the stack themselves to see numbers.

## Goals / Non-Goals

**Goals:**
- Add a `perf_e2e` feature-gated test module in `crates/e2e/tests/` that starts the `docker-compose.perf.yml` stack and runs a short perf-bot smoke scenario against both servers.
- Assert success thresholds (no panics, error rate < 1%, at least one successful action recorded) without requiring the test to be fast — these are inherently slow Docker tests.
- Add `docs/performance/README.md` with a Markdown table of C++ vs Rust comparison results, sourced from a committed `docs/performance/perf-results.json` reference run.
- Provide a `scripts/update-perf-readme.sh` script that regenerates the README table from a fresh run, so the process is reproducible.

**Non-Goals:**
- Running perf e2e tests on every PR in GitHub Actions (too slow; optional dedicated job only).
- Automating the README update in CI (committed reference results are intentionally static until manually updated).
- Modifying the C++ upstream or `forgottenserver-upstream/`.

## Decisions

**Decision: extend `crates/e2e/` rather than a new crate**
The existing e2e crate has the `#[cfg(feature = "e2e")]` pattern, `ServerFixture`, and test infrastructure already proven. Adding a `perf_e2e` feature flag alongside `e2e` keeps the Docker-gating idiom consistent and avoids a new crate member.
Alternative considered: a top-level `tests/perf_e2e/` integration test directory — rejected because it can't reuse the ServerFixture abstraction easily and doesn't align with how the project already gates Docker tests.

**Decision: use `#[ignore]` + feature flag for perf e2e tests**
`#[cfg(feature = "perf_e2e")]` ensures the tests aren't even compiled in standard workspace builds. A `#[ignore]` annotation on top provides a second opt-in layer (run with `-- --ignored`) for environments that have the feature but want to skip by default.
Alternative: a separate binary — rejected as over-engineering for what is a small smoke test.

**Decision: committed `perf-results.json` as the README source**
The README table needs to render on GitHub without a live server. Committing a reference run result file decouples the documentation from the live environment. The `update-perf-readme.sh` script regenerates both the JSON and the README, so updating is a two-command workflow.
Alternative: generate the README dynamically in CI — rejected because it creates a dependency on the full perf Docker stack in every main-branch CI run.

**Decision: `docs/performance/README.md` location**
Placing it under `docs/performance/` keeps the repo root clean and signals it is reference documentation rather than an operational README. The project `README.md` can link to it.

## Risks / Trade-offs

- **Docker availability on dev machines** → tests gate on `feature = "perf_e2e"` so they never run unless explicitly opted in; no impact on `cargo test --lib --workspace`.
- **Stale perf-results.json** → the file is intentionally a snapshot; the update script documents how to refresh it. Staleness is a documentation concern, not a correctness concern.
- **C++ upstream first-build time (~10-15 min)** → perf e2e tests should document the expected cold-start time and use a generous timeout (e.g., 5-minute server readiness poll).
- **Port conflicts** → perf-bot tests use the same ports as docker-compose.perf.yml (7372/7472); the test must ensure no other stack is running, documented in test output.
