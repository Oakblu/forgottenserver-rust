## ADDED Requirements

### Requirement: Every active client opcode maps to its handler via a curated edge

The flow graph SHALL contain a `kind: dynamic`, `confidence: curated` edge from `ProtocolGame::parsePacket` to the handler of every *active* `case` in its `switch (recvbyte)`, each tagged `condition: "recvbyte == 0xNN"`.

#### Scenario: AutoWalk opcode is edged

- **WHEN** the graph is queried for out-edges of `ProtocolGame::parsePacket`
- **THEN** an edge to `ProtocolGame::parseAutoWalk` exists with `condition` referencing opcode `0x64`

#### Scenario: Disabled opcode is annotated, not edged

- **WHEN** an opcode case is commented out in `parsePacket` (e.g. bestiary tracker `0x2A`)
- **THEN** no edge exists for it AND it is listed under `disabled_opcodes` in `flow_graph/nodes/protocolgame.yml`

### Requirement: Login and status entrypoints are curated as dynamic edges

The graph SHALL record `ProtocolLogin::onRecvFirstMessage` and the `ProtocolStatus` request path as dynamic entrypoints reachable from the connection-accept path, so they are reachable from the root.

#### Scenario: Login first-message entrypoint is reachable

- **WHEN** the graph is traversed from the root
- **THEN** `ProtocolLogin::onRecvFirstMessage` is reachable via a curated dynamic edge

### Requirement: An opcode coverage check ties curated edges to the dispatch switch

The change SHALL provide a check that parses the `switch (recvbyte)` arms in `protocolgame.cpp` and asserts every active case has exactly one curated edge and every disabled case is annotated.

#### Scenario: Missing opcode edge fails the check

- **WHEN** an active `case 0xNN` in `parsePacket` has no curated edge
- **THEN** the coverage check exits non-zero naming opcode `0xNN`

#### Scenario: Opcode mismatch fails the check

- **WHEN** a curated edge's `condition` opcode does not match the `case` value it claims to handle
- **THEN** the coverage check exits non-zero naming the mismatch

#### Scenario: Fully-covered switch passes

- **WHEN** every active case has a matching curated edge and every disabled case is annotated
- **THEN** the coverage check exits zero
