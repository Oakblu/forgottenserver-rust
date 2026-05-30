## ADDED Requirements

### Requirement: Deferred scheduler/dispatcher tasks are edged to their effective callee

The graph SHALL contain a `kind: dynamic`, `confidence: curated` edge from each scheduler/dispatcher enqueue site to the function the enqueued lambda effectively calls, with `condition` noting the trigger.

#### Scenario: mainLoader deferred task is edged

- **WHEN** the graph is queried for out-edges of `startServer`
- **THEN** a dynamic edge to `mainLoader` exists with `condition` noting it is dispatcher-deferred

#### Scenario: Scheduled creature-think tick is edged

- **WHEN** the graph is queried for out-edges of `Game::start`
- **THEN** a dynamic edge to `Game::checkCreatures` exists with `condition` referencing `EVENT_CREATURE_THINK_INTERVAL`

### Requirement: Event types are edged to their Lua dispatch boundary

The graph SHALL contain a `kind: dynamic`, `confidence: curated` edge from each `CREATURE_EVENT_*` executor (and the global-event/talk-action executors) to a representative Lua dispatch symbol, tagged with the event type in `condition`.

#### Scenario: Login event reaches the Lua boundary

- **WHEN** the graph is traversed from the creature-event executor for `CREATURE_EVENT_LOGIN`
- **THEN** a dynamic edge to the Lua dispatch symbol exists with `condition` referencing `CREATURE_EVENT_LOGIN`

#### Scenario: All creature-event types are covered

- **WHEN** the curated event edges are compared to the `CREATURE_EVENT_*` enum values parsed from `creatureevent.cpp`
- **THEN** every non-`NONE` event type has at least one curated edge

### Requirement: Event/scheduler coverage is checkable

The change SHALL provide a check asserting curated edges exist for every parsed `CREATURE_EVENT_*` type and for the known periodic scheduler tasks, failing loudly on omissions.

#### Scenario: Missing event-type edge fails the check

- **WHEN** a `CREATURE_EVENT_*` type (other than `NONE`) has no curated edge
- **THEN** the check exits non-zero naming the event type

#### Scenario: Complete coverage passes

- **WHEN** every event type and known periodic task has a curated edge
- **THEN** the check exits zero
