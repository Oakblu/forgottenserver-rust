## Why

`Creature` (`creature.h:81`) is an abstract base with many `virtual`/pure-virtual methods (`getType`, `setID`, `onWalk`, `goToFollowCreature`, …); `Player`, `Monster`, and `Npc` are `final` subclasses that override them. A C++ call like `creature->onThink()` statically resolves to `Creature::onThink`, but at runtime dispatches to the concrete override — so phase-2 static edges point at the wrong (base) target and the actual runtime behavior is invisible. If the Rust port implements the base but not a subclass override, nothing in the symbol layer flags it; the virtual edge does. Phase 5 of 6; context is bounded to the creature hierarchy headers.

## What Changes

- Curate `kind: dynamic`, `confidence: curated` edges from each overridden base virtual to its concrete override(s) in `Player`/`Monster`/`Npc`, tagged with the dynamic type in `condition` (e.g. `"dyntype == Player"`).
- Prioritize the runtime-hot virtuals (think/walk/death/attack/target-selection, `getType`, ghost/visibility predicates) and the pure-virtuals that MUST be overridden.
- Add a coverage check tying curated override edges to the `override` declarations parsed from the subclass headers.
- **Non-goal:** network (phase 3), event/scheduler (phase 4), gap analysis (phase 6).

## Capabilities

### New Capabilities
- `flow-graph-dynamic-virtual`: Curated dynamic edges from base-class virtual methods to their concrete `Player`/`Monster`/`Npc` overrides, so runtime polymorphic dispatch is represented in the graph.

### Modified Capabilities
<!-- None. Adds data under the flow-graph-format contract. -->

## Impact

- New/expanded data: dynamic edges in `flow_graph/nodes/{creature,player,monster,npc}.yml`.
- Reads, never modifies: `forgottenserver-upstream/src/{creature,player,monster,npc}.{cpp,h}`.
- **Depends on:** phases 1–2. **Unlocks:** phase 6 sees polymorphic dispatch edges.
