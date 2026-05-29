## Why

Two more classes of edges are invisible to static extraction but central to runtime behavior: (1) **deferred tasks** — `g_dispatcher.addTask([...]{ mainLoader })` (`otserv.cpp:281`) and `g_scheduler.addEvent(createSchedulerTask(... checkCreatures/checkDecay))` (`game.cpp:68`) — where the real callee lives in a lambda; and (2) **event → Lua callbacks** — `CreatureEvent::executeOnLogin/Logout/Advance/...` and the global/talk-action event systems that fan out into scripts. Without these edges the graph appears to "stop" at the enqueue site or the C++ event shell, hiding whole runtime paths. Phase 4 of 6; context is bounded to the event + scheduler files.

## What Changes

- Curate `kind: dynamic` edges from scheduler/dispatcher enqueue sites to the lambda's effective callee (e.g. `startServer`→`mainLoader`, `Game::start`→`checkCreatures`/`checkDecay`, deferred autowalk tasks).
- Curate `kind: dynamic` edges from each `CreatureEvent` type (`LOGIN`, `LOGOUT`, `RECONNECT`, `ADVANCE`, `DEATH`, `KILL`, `THINK`, `PREPAREDEATH`, `MODALWINDOW`, `TEXTEDIT`, …) and the global/talk-action event executors to their Lua-callback dispatch point.
- Mark the Lua boundary explicitly (target = the script-interface call symbol) so phase 6 can detect unregistered/unported callbacks.
- **Non-goal:** network opcode edges (phase 3), virtual dispatch (phase 5), gap analysis (phase 6).

## Capabilities

### New Capabilities
- `flow-graph-dynamic-events`: Curated dynamic edges for scheduler/dispatcher deferred tasks and for event-system → Lua-callback dispatch, so deferred and event-driven runtime paths are represented in the graph.

### Modified Capabilities
<!-- None. Adds data under the flow-graph-format contract. -->

## Impact

- New/expanded data: dynamic edges in `flow_graph/nodes/{otserv,game,creatureevent,globalevent,events,baseevents}.yml`.
- Reads, never modifies: the corresponding `forgottenserver-upstream/src/*.cpp/h`.
- **Depends on:** phases 1–2. **Unlocks:** phase 6 sees event/scheduler dynamic edges.
