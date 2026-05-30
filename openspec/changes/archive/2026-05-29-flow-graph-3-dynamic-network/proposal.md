## Why

Client packets are dispatched by `ProtocolGame::parsePacket` (`protocolgame.cpp:503`) via `switch (recvbyte)` — each opcode (`0x14`, `0x64`→`parseAutoWalk`, …) jumps to a handler. Static extraction (phase 2) sees the function but not the opcode→handler mapping, yet that mapping IS the runtime contract: a missing or mis-numbered opcode handler is exactly the kind of silent divergence this whole effort targets. This phase curates those edges. Phase 3 of 6; context is bounded to the protocol files.

## What Changes

- Curate `kind: dynamic`, `confidence: curated` edges for every client opcode in `ProtocolGame::parsePacket` → its `parseXxx` handler, each tagged with the opcode (`condition: recvbyte == 0xNN`).
- Curate the login/status entrypoints (`ProtocolLogin::onRecvFirstMessage`, `ProtocolStatus`) → their handlers.
- Curate the server-bound send path entrypoints where dispatch is table/virtual driven, as applicable.
- Add a coverage check: every `case 0x..` handled in `parsePacket` has a corresponding curated edge (commented-out/disabled opcodes excluded with a note).
- **Non-goal:** event/scheduler edges (phase 4), virtual dispatch (phase 5), gap analysis (phase 6).

## Capabilities

### New Capabilities
- `flow-graph-dynamic-network`: Curated dynamic edges mapping each network opcode to its handler in the flow graph, with opcode-tagged conditions and a coverage check against the dispatch switch.

### Modified Capabilities
<!-- None. Adds data under the flow-graph-format contract. -->

## Impact

- New/expanded data: dynamic edges in `flow_graph/nodes/protocolgame.yml`, `protocollogin.yml`, `protocolstatus.yml`, `protocol.yml`.
- New tooling: an opcode-coverage check (script or test) over `parsePacket`.
- Reads, never modifies: `forgottenserver-upstream/src/protocol*.cpp/h`.
- **Depends on:** phases 1–2. **Unlocks:** phase 6 (gap report) sees network dynamic edges.
