## ADDED Requirements

### Requirement: Base virtuals are edged to their concrete overrides

The graph SHALL contain a `kind: dynamic`, `confidence: curated` edge from each in-scope base `Creature` virtual to each concrete override in `Player`/`Monster`/`Npc`, tagged `condition: "dyntype == <Subclass>"`.

#### Scenario: getType resolves to each subclass

- **WHEN** the graph is queried for out-edges of `Creature::getType`
- **THEN** dynamic edges to `Player::getType`, `Monster::getType`, and `Npc::getType` exist, each with a `condition` naming the subclass

#### Scenario: Behavioral virtual resolves to its overrides

- **WHEN** a behavioral virtual (e.g. `Creature::onWalk`) is overridden in a subclass
- **THEN** a dynamic edge from the base to that concrete override exists

### Requirement: Every pure-virtual has an override edge per concrete subclass

For each pure-virtual (`= 0`) in `Creature`, the graph SHALL contain an override edge to the corresponding method in each concrete subclass that the headers declare.

#### Scenario: Pure-virtual is fully resolved

- **WHEN** a pure-virtual (e.g. `Creature::setID`) is examined
- **THEN** a dynamic edge exists to its override in each concrete subclass declaring it

### Requirement: Override coverage is checkable against the headers

The change SHALL provide a check that parses `override`/`override final` declarations from the subclass headers, intersects them with the in-scope base virtuals (excluding an explicit allowlist of trivial accessors), and asserts each has a curated edge.

#### Scenario: Missing override edge fails the check

- **WHEN** an in-scope override declared in `player.h`/`monster.h`/`npc.h` has no curated edge
- **THEN** the check exits non-zero naming the override

#### Scenario: Trivial accessor is excluded, not required

- **WHEN** an override is on the trivial-accessor allowlist (e.g. `getPlayer()` returning `this`)
- **THEN** the check does not require an edge for it

#### Scenario: Complete in-scope coverage passes

- **WHEN** every in-scope override and every pure-virtual override has a curated edge
- **THEN** the check exits zero
