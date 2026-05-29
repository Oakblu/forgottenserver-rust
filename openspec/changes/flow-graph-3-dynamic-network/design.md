## Context

`ProtocolGame::parsePacket` reads `recvbyte = msg.getByte()` then `switch (recvbyte)` with dozens of `case 0xNN: parseXxx(msg);` arms (e.g. `0x64`→`parseAutoWalk`). Some cases are commented out (e.g. bestiary, team finder). `ProtocolLogin::onRecvFirstMessage` and `ProtocolStatus` are the other client entrypoints. These opcode→handler mappings are dynamic dispatch that phase-2 static extraction cannot resolve.

## Goals / Non-Goals

**Goals:**
- A curated dynamic edge for every *active* opcode case → its handler, tagged with the opcode.
- A coverage check that ties curated edges to the actual switch arms so drift is detectable.

**Non-Goals:**
- Event/scheduler edges (phase 4), virtual dispatch (phase 5).
- Reimplementing protocol logic — this records the call graph only.

## Decisions

### D1: Edge encoding for opcodes
Each opcode edge: source = `ProtocolGame::parsePacket`, target = the `parseXxx` handler symbol, `kind: dynamic`, `confidence: curated`, `condition: "recvbyte == 0xNN"`, `order` = position in the switch. Disabled (commented-out) opcodes are NOT edged but are noted in `flow_graph/nodes/protocolgame.yml` under a `disabled_opcodes` annotation so the coverage check can distinguish "intentionally absent" from "missed".

### D2: Coverage check source of truth
A small script/test parses the `switch (recvbyte)` arms in `protocolgame.cpp` and asserts every active `case` has exactly one curated edge, and every disabled case is annotated. This keeps the curated set honest against the C++ without re-running full extraction.

### D3: Login/status entrypoints
`onRecvFirstMessage` and the status request path are curated as dynamic entrypoints from the connection accept path, so the gap report (phase 6) can reach them from `main` via the service manager.

## Risks / Trade-offs

- **Opcode list drifts vs. C++** → the coverage check fails loudly when a switch arm has no edge.
- **Mis-tagged opcode number** → coverage check compares the `condition` opcode to the parsed `case` value.
- **Handler resolves through a lambda/`addGameTask`** → record the edge to the ultimate handler symbol and note the indirection in `condition`.
