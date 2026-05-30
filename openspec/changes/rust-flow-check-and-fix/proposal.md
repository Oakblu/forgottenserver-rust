## Why

The flow graph now captures all 142 C++ execution paths reachable from `main()`, and `MIGRATION_LEDGER.yml` claims every reachable symbol is `migrated` or `migrated_with_changes` — but "claimed migrated" is not the same as "behaviorally correct." Each Rust implementation must be verified against its C++ spec at the unit level, and any gaps (missing tests, stub bodies, wrong behavior) must be fixed. This work converts the flow graph from a coverage map into a correctness guarantee.

## What Changes

- For each of the 142 reachable C++ nodes: verify the Rust implementation has at least one test that exercises the behavior described by the C++ source; fix any stub or incorrect implementation found.
- Boot / init path (depth 0-3): config load, DB connect, game state init, map load, item/monster/vocation XML load, scripting bootstrap.
- Protocol / session path (depth 3-5): `Connection::parsePacket`, `ProtocolGame::onRecvFirstMessage`, `ProtocolGame::parsePacket`, `ProtocolStatus::onRecvFirstMessage`.
- Opcode handler path (depth 5): all 50 `ProtocolGame::parse*` handlers.
- Event / scheduler path (depth 4-6): `GlobalEvents`, `CreatureEvents`, tfs::events player callbacks, `Game::check*` loops.
- Supporting services (depth 3-5): `IOLoginData`, `IOBan`, `Chat`, `Weapons`, `Outfits`, `Vocations`, `Scripts`, `ScriptingManager`.

## Capabilities

### New Capabilities

- `flow-verification-boot`: Behavioral correctness tests for the boot/init chain (depth 0-3 nodes).
- `flow-verification-protocol`: Behavioral correctness tests for Connection and ProtocolGame session handling (depth 3-5).
- `flow-verification-opcodes`: Behavioral correctness tests for all ProtocolGame opcode handlers (depth 5).
- `flow-verification-events`: Behavioral correctness tests for GlobalEvents, CreatureEvents, and scheduler event loops (depth 4-6).
- `flow-verification-services`: Behavioral correctness tests for supporting service nodes (IOLoginData, IOBan, Chat, Weapons, Outfits, Vocations, Scripts).

### Modified Capabilities

## Impact

- All Rust crates: `common`, `items`, `map`, `entity`, `world`, `database`, `game`, `scripting`, `network`, `server`.
- `MIGRATION_LEDGER.yml`: stub entries will be promoted to fully tested as each node passes.
- `intentional_differences.yml`: any confirmed behavioral divergence found during verification must be recorded here.
- No changes to `forgottenserver-upstream/src/` (read-only spec).
