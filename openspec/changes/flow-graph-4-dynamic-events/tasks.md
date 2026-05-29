## 1. Deferred-task edges

- [ ] 1.1 Curate `startServer`→`mainLoader` (dispatcher-deferred) edge in `flow_graph/nodes/otserv.yml` with `condition`
- [ ] 1.2 Curate `Game::start`→ scheduled ticks (`checkCreatures`, decay/`checkDecay`, and other `createSchedulerTask` callees in `game.cpp:68-71`) in `flow_graph/nodes/game.yml`, tagging the interval in `condition`
- [ ] 1.3 Curate the remaining `addTask`/`addEvent` enqueue sites (e.g. deferred autowalk continuations, shutdown task) to their effective callees

## 2. Event → Lua edges

- [ ] 2.1 Parse the `CREATURE_EVENT_*` enum values from `creatureevent.cpp` and curate a dynamic edge per type from its executor to the Lua dispatch symbol, tagging the type in `condition`
- [ ] 2.2 Curate the global-event and talk-action/move-event executors → Lua dispatch in their respective `flow_graph/nodes/*.yml`

## 3. Coverage check

- [ ] 3.1 Implement a check asserting every parsed non-`NONE` `CREATURE_EVENT_*` type and every known periodic scheduler task has a curated edge
- [ ] 3.2 Add tests: missing event-type edge fails, complete coverage passes

## 4. Verification

- [ ] 4.1 Run `make flow` (+ the event coverage check) and confirm it passes with the curated event/scheduler edges
- [ ] 4.2 Confirm no file under `forgottenserver-upstream/src/` or any Rust crate was modified
