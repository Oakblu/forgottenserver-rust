## Context

`creature.h` declares the base virtuals; `player.h` (`Player final : public Creature`), `monster.h` (`Monster final : public Creature`), `npc.h` (`Npc final : public Creature`) carry `override`/`override final` declarations. Pure virtuals (`= 0`: `getName`, `getType`, `setID`, `removeList`, `addList`, `goToFollowCreature`, …) MUST be overridden by every concrete subclass; non-pure virtuals (`onWalk`, `canSee`, `getMaxHealth`, …) may be overridden selectively. Phase-2 static extraction records a call to the base symbol; this phase adds the polymorphic targets.

## Goals / Non-Goals

**Goals:**
- For each base virtual that is overridden, an edge to each concrete override, tagged with the dynamic type.
- A coverage check derived from the subclasses' `override` declarations.

**Non-Goals:**
- Exhaustively edging trivial accessor overrides (e.g. `getPlayer()` returning `this`) — these add noise without runtime-flow value; the spec targets *behavioral* virtuals and all pure-virtuals.
- Network/event edges (phases 3/4).

## Decisions

### D1: Override edge encoding
Edge: source = base virtual symbol, target = the concrete override symbol, `kind: dynamic`, `confidence: curated`, `condition: "dyntype == Player|Monster|Npc"`. One edge per concrete override of a given base virtual.

### D2: Scope — behavioral virtuals + all pure-virtuals
Cover (a) every pure-virtual (`= 0`) since each demands an override in every concrete subclass, and (b) the runtime-hot behavioral virtuals: think/walk lifecycle (`onWalk`, `onWalkComplete`, `goToFollowCreature`), combat/death (`onDeath`, `onAttack*`, target selection), and visibility/type predicates (`getType`, `getRace`, ghost-mode). Trivial pointer-returning overrides are out of scope (D-NonGoal).

### D3: Coverage check from headers
Parse `override`/`override final` declarations from `player.h`/`monster.h`/`npc.h`, intersect with the in-scope base virtuals, and assert each has a curated edge. Trivial accessors are excluded via an allowlist so the check stays meaningful.

## Risks / Trade-offs

- **Override-set drift** → coverage check parses the headers and fails on any in-scope override lacking an edge.
- **Scope judgment (behavioral vs. trivial)** → the allowlist of excluded trivial accessors is explicit and reviewable; pure-virtuals are never excluded.
- **Multiple subclasses per virtual** → one edge per concrete override; `condition` distinguishes the dynamic type.
