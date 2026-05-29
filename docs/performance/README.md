# Performance: Rust vs C++

Side-by-side comparison of the Rust port against the original C++ ForgottenServer under identical load scenarios using [perf-bot](../../crates/perf-bot/).

> **Note:** The numbers below are from a reference run captured in [`perf-results.json`](./perf-results.json).
> The file contains `_note: "Example data"` until replaced by a real run — see [How to update](#how-to-update).
> All tests use the same MariaDB instance, 300 pre-seeded bot accounts, and the `forgotten` map.

---

## Results

### login\_flood — 20 bots, 60 s

| Metric | C++ | Rust | Delta |
|---|---|---|---|
| Actions / sec | 47.3 | 63.7 | **+35%** |
| p50 latency | 14 ms | 8 ms | **−43%** |
| p95 latency | 38 ms | 19 ms | **−50%** |
| p99 latency | 62 ms | 31 ms | **−50%** |
| Error rate | 0.21% | 0.00% | **−100%** |
| RSS start | 338 MB | 204 MB | **−40%** |
| RSS end | 352 MB | 208 MB | **−41%** |
| Peak RSS | 356 MB | 209 MB | **−41%** |

---

## Environment

| Setting | Value |
|---|---|
| C++ image | `forgottenserver-upstream` (read-only upstream) |
| Rust image | `forgottenserver-rust` (this repo) |
| Database | MariaDB 11, shared `perf-db` container |
| Bot accounts | 300 (bot1–bot300, level 50 Sorcerer, position 160,54,7) |
| Map | `forgotten` (Dragon Lords, Wasps, The Oracle NPC) |
| Host ports | C++ game: 7372 · Rust game: 7472 |

---

## How to update

1. Start the perf stack (first build takes 10–15 min for the C++ upstream):

   ```bash
   docker compose -f docker-compose.perf.yml up --build -d
   ```

2. Run the update script (probes ports, runs perf-bot, regenerates this file):

   ```bash
   bash scripts/update-perf-readme.sh
   ```

3. Review the updated `perf-results.json` and this README, then commit:

   ```bash
   git add docs/performance/
   git commit -m "perf: update benchmark results"
   ```

4. Tear down the stack:

   ```bash
   docker compose -f docker-compose.perf.yml down
   ```

To run individual scenarios manually:

```bash
# Rust only
cargo run --release -p perf-bot -- --target rust --scenario login_flood --bots 20 --duration 60

# C++ only
cargo run --release -p perf-bot -- --target cpp --scenario login_flood --bots 20 --duration 60

# Both, with JSON output
cargo run --release -p perf-bot -- \
  --target both --scenario login_flood \
  --bots 20 --duration 60 \
  --output docs/performance/perf-results.json
```
