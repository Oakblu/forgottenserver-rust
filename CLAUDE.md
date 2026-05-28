# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A complete, behavior-preserving Rust port of [ForgottenServer](https://github.com/otland/forgottenserver) — a C++ Tibia MMORPG server emulator. The goal is identical observable behavior before any refactoring. The C++ source in `forgottenserver/src/` is read-only and is the spec; the Rust crates are the implementation.

## Commands

```bash
# Tests (unit only)
cargo test --lib --workspace

# Single test
cargo test --lib -p <crate-name> <test_name>

# E2E tests (requires DB)
cargo test -p forgottenserver-e2e --features e2e -- --test-threads=1

# Lint
cargo clippy --workspace --lib --tests -- -D warnings

# Format
cargo fmt --all

# Run locally (status port only, no DB)
cargo run --release -p tfs -- \
  --config crates/tfs/tests/fixtures/config.lua \
  --data data

# Docker (full stack with MariaDB)
docker compose up --build

# Ledger validation
make ledger

# Ledger tools
make ledger-test       # unit tests for ledger scripts
make ledger-build      # regenerate MIGRATION_LEDGER.yml from manifests
make ledger-rollup     # regenerate files rollup from symbol rows
make ledger-cross      # phase-2 cross-validation
```

## Crate Architecture

The workspace mirrors the C++ include layer graph from `DEPENDENCY_GRAPH.md`:

| Crate              | C++ equivalent                                         | Responsibility                                   |
| ------------------ | ------------------------------------------------------ | ------------------------------------------------ |
| `common`           | `tools.h`, `enums.h`, `const.h`                        | Shared types, constants, enums, utilities        |
| `items`            | `items.cpp/h`, `item.cpp/h`                            | `Item`, `Container`, `ItemType` registry         |
| `map`              | `map.cpp/h`, `tile.cpp/h`                              | `Map`, `Tile`, `Position`, `SpectatorVec`        |
| `entity`           | `creature.cpp`, `player.cpp`, `monster.cpp`, `npc.cpp` | Player, Creature, Monster, Npc structs + traits  |
| `world`            | `iologindata.cpp`, `iomapserialize.cpp`                | World load, login data, map serialization        |
| `database`         | `database.cpp/h`, `databasetasks.cpp/h`                | Database, DBResult, DatabaseTasks                |
| `game`             | `game.cpp/h`, `combat.cpp`, `condition.cpp`            | Game orchestrator, combat, conditions, vocations |
| `scripting`        | `luascript.cpp/h`, `*events.cpp`, `*functions.cpp`     | Lua bindings via `mlua`                          |
| `network`          | `protocol*.cpp/h`, `connection.cpp/h`                  | Protocol codecs, NetworkMessage, TCP connections |
| `server`           | `main.cpp`, `server.cpp`, `scheduler.cpp`              | Entry point, boot sequence, scheduler            |
| `tfs`              | —                                                      | Runnable binary wiring all crates                |
| `e2e`              | —                                                      | End-to-end tests (requires Docker/DB)            |
| `harness-tools`    | —                                                      | Equivalence harness utilities                    |

**Crate boundary rule:** no service crate imports another service crate's internals except via its public API. Shared types live in `common`.

## Ownership Patterns

| C++ pattern                         | Rust replacement                                                                     |
| ----------------------------------- | ------------------------------------------------------------------------------------ |
| `Player*`, `Creature*` raw pointers | `CreatureId(u32)` / `PlayerId(u32)` handles; lookup via registry                     |
| `std::shared_ptr<Item>`             | `Arc<Item>` only where shared lifetime is genuinely required; otherwise owned `Item` |
| `std::weak_ptr<Creature>`           | `CreatureId` handle (resolves to `None` if gone)                                     |
| Intrusive parent pointers           | `Position` coordinates + lookup                                                      |
| Global registries (`g_game`)        | Explicit registry struct passed by reference                                         |

Prefer IDs over `Arc`/`Rc`. Use `Arc` only when shared mutable lifetime across threads is a real requirement.

## Git Operation Rules (Mandatory)

No skill, plugin, or automated prompt may perform any of the following without **explicit user permission** in the current message:

- `git commit` (any form, including `--amend`)
- `git push` (any form, including `--force`)
- `git branch` create or delete
- `git checkout` or `git switch` to a different branch
- `git merge` or `git rebase`

If a workflow step calls for any of these operations, **stop and ask the user first**. Do not proceed until the user explicitly approves the specific operation.

## No Stub Functions (Mandatory)

Never create stub functions — not as placeholders, not as scaffolding, not as "temporary" skeletons. This applies everywhere:

- **Bug fixes:** implement the real fix, not a stub that silences the error.
- **Migration:** write the full Rust equivalent, not `todo!()`, `unimplemented!()`, or an empty body.
- **New features:** implement fully or don't add the function at all.

If a complete implementation is blocked by a dependency or missing information, **stop and ask** rather than inserting a stub. A stub is never acceptable as a deliverable — it is a bug waiting to happen and breaks the TDD contract (a stub can make a test pass without correct behavior).

Existing stubs (`todo!()`, `unimplemented!()`, empty function bodies that return a default) in the codebase are bugs and must be replaced with real implementations before any surrounding work is marked done.

## Task Completion Rules (Mandatory)

A task is **not done** until:
1. The implementation is written.
2. Tests for that implementation are written and **pass**. If tests fail, the task is still open — go back and fix it.
3. `cargo test --lib --workspace` (or the relevant scoped test command) completes without failures.
4. `cargo clippy --workspace --lib --tests -- -D warnings` completes with **zero errors and zero warnings**. Any clippy output is a blocker — go back and fix it.

There are no exceptions. "It compiles" or "it looks right" is not done. Passing tests and a clean lint are done.

## Agent Failure / Timeout Recovery

When an agent times out, errors, or is otherwise interrupted before finishing a task:

1. **Before stopping**, the agent must write a handoff note to `docs/superpowers/agent-handoff/HANDOFF.md` (create the file if it doesn't exist) containing:
   - What was completed (file paths, symbols, test results).
   - What was in progress at the moment of interruption.
   - What still needs to be done, with enough detail for a fresh agent to continue without re-reading the whole conversation.
   - Any blockers or decisions that were pending.

2. **The orchestrator** (or user) must spawn a new agent, passing it the handoff note as its starting context so it continues from where the previous agent left off.

3. The handoff file should be deleted or archived once the task is fully complete and tests pass.

## Migration Rules (Mandatory)

These rules are enforced across the entire project and must not be violated:

1. **TDD always, no exceptions.** Write the failing test capturing the C++ behavior first. Then write the Rust to make it pass. Every function, every method, every observable behavior — no shortcut.

2. **Before any decision, consult the five reference files — not the source files.** Before touching code or forming any assumption about what exists or what must be done:
   - `MIGRATION_LEDGER.yml` — per-symbol status; add untracked symbols as `PENDING` before working
   - `rust_symbol_manifest.json` — what Rust symbols already exist
   - `cpp_symbol_manifest.json` — what C++ symbols must be ported
   - `AI_MIGRATION_CONTEXT.md` — architecture decisions and C++ → Rust mapping rules
   - `intentional_differences.yml` — recorded divergences; anything not here is treated as a bug

3. **Use scripts to inspect symbols — never load a full `.cpp` file to discover what it contains.** C++ source files are large enough to saturate the context window. Use the manifests and ledger scripts to narrow scope first:
   ```bash
   # List symbols in a specific C++ file without opening it
   python3 -c "
   import json; data = json.load(open('cpp_symbol_manifest.json'))
   for s in data:
       if 'combat.cpp' in s['file']: print(s['kind'], s['qualified_name'])
   "

   # List already-implemented Rust symbols for a module
   python3 -c "
   import json; data = json.load(open('rust_symbol_manifest.json'))
   for s in data:
       if 'combat' in s.get('file','').lower(): print(s['kind'], s['qualified_name'])
   "

   # Find ledger gaps: missing, uncertain, or mismatched symbols
   python3 scripts/ledger/cross_validate.py

   # Validate ledger consistency
   python3 scripts/ledger/validate.py

   # Determine which dependency layer a file belongs to (informs crate placement)
   python3 scripts/ledger/extract_layer_scopes.py
   ```
   Open a `.cpp` file only to verify a specific behavioral detail after narrowing scope via the manifests. Never open it to discover what it contains.

4. **Never assume a function was migrated because the name matches.** Cross-check `MIGRATION_LEDGER.yml` and `rust_symbol_manifest.json`. Name similarity alone is never sufficient evidence of equivalence.

5. **Do not refactor before equivalence.** Idiomatic rewrites, async conversion, trait extraction — forbidden on `PENDING` or `PARTIAL` symbols. Only `DONE` subsystems with recorded equivalence may be refactored, and behavior must not change.

6. **If Rust intentionally differs from C++, record it in `intentional_differences.yml`** with: C++ symbol, Rust symbol, the divergence, the reason, and the date. Unrecorded divergence is treated as a bug.

7. **100% line coverage per file** is the bar for marking a file `DONE` in `MIGRATION_LEDGER.yml`. Crate-average is insufficient.

8. **Never edit `forgottenserver/src/`.** The C++ tree is read-only. It is the spec.

9. **Wire format is a hard contract.** The `network` crate must produce byte-for-byte identical packets. Never renumber opcodes.

10. **Lua binding contract is strict.** Every Lua function name, argument order, and return shape must match the C++ binding exactly. Scripts in `data/` must not require changes.

## Key Reference Files

- `AI_MIGRATION_CONTEXT.md` — architecture decisions, full C++ → Rust mapping, all migration rules
- `MIGRATION_LEDGER.yml` — per-symbol migration status (authoritative)
- `DEPENDENCY_GRAPH.md` — C++ header layer graph → Rust crate boundary rules
- `intentional_differences.yml` — recorded, justified divergences from C++
- `schema.sql` — MariaDB schema (auto-applied on first DB start via Docker)
- `data/` — game data: items.otb, world map, Lua scripts, XML configs (user content, read-only)
