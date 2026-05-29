## Context

Deferred tasks: `otserv.cpp:281` enqueues `mainLoader` onto the dispatcher; `game.cpp:68-71` schedules `checkCreatures`, decay, and other periodic ticks via `createSchedulerTask`; several `game.cpp` sites enqueue per-player autowalk continuations. Event callbacks: `creatureevent.cpp` selects by `getEventType()` and calls `executeOnLogin`/`executeOnLogout`/`executeOnReconnect`/`executeOnAdvance`/… which invoke the Lua script interface; `globalevent.cpp`, `events.cpp`, and talk-actions follow the same shape. Static extraction stops at the enqueue call or the C++ executor shell.

## Goals / Non-Goals

**Goals:**
- Edge each enqueue site to the lambda's effective callee.
- Edge each event type to its Lua dispatch point, with the Lua boundary marked.

**Non-Goals:**
- Enumerating individual user scripts in `data/` (the graph stops at the registered-callback boundary; which scripts register is runtime/data, not C++ flow).
- Network/virtual edges (phases 3/5).

## Decisions

### D1: Lambda/deferred-task edges
For `addTask`/`addEvent(createSchedulerTask(...))`, record `kind: dynamic`, `confidence: curated`, `condition` noting the trigger (e.g. `"scheduled: EVENT_CREATURE_THINK_INTERVAL"` or `"deferred: dispatcher"`), target = the function the lambda effectively calls. Periodic tasks that re-arm themselves get a self/`condition: "periodic"` note.

### D2: Event → Lua boundary node
Each event executor edges to a single representative "Lua dispatch" symbol (the `LuaScriptInterface` call entry) tagged `condition` with the event type (e.g. `"CREATURE_EVENT_LOGIN"`). This gives phase 6 a concrete node to check Rust registration against, without enumerating data scripts.

### D3: Scope of event types
Cover all `CREATURE_EVENT_*` types present in `creatureevent.cpp`, plus the global-event and talk-action/move-event executors that fan into Lua. Each is one curated edge.

## Risks / Trade-offs

- **Lambda callee ambiguity** → record the dominant/effective callee and note captures in `condition`; over-approximate rather than omit.
- **Event-type list drift** → cross-check curated edges against the `CREATURE_EVENT_*` enum values parsed from `creatureevent.cpp`.
- **Lua boundary is coarse** → intentional; per-script resolution is data-driven and out of scope for a C++ flow graph.
