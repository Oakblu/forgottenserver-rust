# scripts/

Utility scripts for the Rust port of forgottenserver.

| Script                            | Purpose                                                                                                    |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `smoke.sh`                        | Bring up the parity-test trio (MariaDB + C++ + Rust) in Docker, wait for both servers to come online, probe their status ports, report any divergence in `../SMOKE_DIVERGENCE.md`. Tears down on exit (idempotent across re-runs). Exit 0 = both booted; exit 1 = boot failure. |
| `audit_missing_methods.sh`        | Compare C++ `send*` methods to Rust counterparts (used during the wire-parity work; archived).             |
| `build_luascript_bindings.py`     | Generate Rust↔Lua type bindings from the C++ scripting headers (used during the base TDD migration).       |
| `extract_entity_symbols.py`       | Diff C++ entity-tree symbols against the Rust port (audit-only).                                           |
| `manifest_extractor/`             | Cargo binary that walks a forgottenserver source tree and emits a JSON manifest of files / classes / fields. |

## `smoke.sh` in detail

```bash
# From the repo root:
scripts/smoke.sh
```

The script:

1. `docker compose up -d db` and waits for `healthy` (max 60 s).
2. `docker compose up -d --build forgottenserver-cpp forgottenserver-rust`.
3. Waits for both servers' `Server Online` log lines (max 120 s).
4. Sends the binary Tibia info-request frame `06 00 ff ff 01 1f` to each
   server's status port (C++ on 7371, Rust on 7471) and captures the
   responses.
5. If responses match byte-for-byte: prints `PASS` and exits 0.
6. If responses differ: writes `SMOKE_DIVERGENCE.md` documenting
   the divergence, prints `WARN`, and exits 0 (the bootable-binary
   milestone's goal is that the stack runs; byte-parity is out-of-scope
   per `binary-and-docker`'s Non-Goals — see the `forgottenserver-rust-
   architectural-parity` change for the parity follow-up).
7. If either server failed to come online: prints `FAIL` and exits 1.

A trap at the end of the script tears the trio down via
`docker compose stop` + `rm -f`, so subsequent runs start from a clean
slate (idempotent across back-to-back invocations).

### Expected runtime

- **First run** (cold image cache, no MariaDB image pulled):
  3–5 minutes (Rust build dominates).
- **Subsequent runs** with both images already built:
  ~30 s end-to-end (MariaDB ~12 s + both servers `Online` in ~2 s + probe
  ~4 s + teardown ~10 s).

### Documented divergence

`SMOKE_DIVERGENCE.md` is regenerated on each run when responses differ.
It is **safe to commit** — the file's purpose is to make the current
parity state grep-able and reviewable.

To clear it after parity is reached: delete the file, then re-run
`smoke.sh` — if responses match, the script will not recreate it.
