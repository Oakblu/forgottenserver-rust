## 1. Opcode dispatch curation

- [x] 1.1 Enumerate the active and disabled `case` arms of `ProtocolGame::parsePacket` (`protocolgame.cpp:531+`)
- [x] 1.2 Add a `kind: dynamic`/`confidence: curated` edge per active opcode → its `parseXxx` handler in `flow_graph/nodes/protocolgame.yml`, with `condition: "recvbyte == 0xNN"` and `order`
- [x] 1.3 Record commented-out/disabled opcodes under a `disabled_opcodes` annotation in `flow_graph/nodes/protocolgame.yml`

## 2. Login/status entrypoints

- [x] 2.1 Curate `ProtocolLogin::onRecvFirstMessage` → its handlers in `flow_graph/nodes/protocollogin.yml`, reachable from the connection-accept path
- [x] 2.2 Curate the `ProtocolStatus` request path in `flow_graph/nodes/protocolstatus.yml`

## 3. Coverage check

- [x] 3.1 Implement a check (script or test) that parses `switch (recvbyte)` arms and asserts every active case has exactly one curated edge, every disabled case is annotated, and edge `condition` opcodes match the case values
- [x] 3.2 Wire the check into `make flow` (or a `make flow-check-network` sub-target) and add tests for: missing edge fails, opcode mismatch fails, full coverage passes

## 4. Verification

- [x] 4.1 Run `make flow` (+ the opcode coverage check) and confirm it passes with the curated network edges
- [x] 4.2 Confirm no file under `forgottenserver-upstream/src/` or any Rust crate was modified
