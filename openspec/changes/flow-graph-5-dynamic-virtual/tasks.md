## 1. Scope the virtual set

- [ ] 1.1 Parse `Creature` virtuals from `creature.h`, separating pure-virtuals (`= 0`) from non-pure
- [ ] 1.2 Define the trivial-accessor allowlist (e.g. `getPlayer`/`getMonster`/`getNpc` returning `this`/`nullptr`) to exclude from the behavioral set

## 2. Curate override edges

- [ ] 2.1 For each pure-virtual, curate a `kind: dynamic` edge to its override in each concrete subclass that declares it (`player.h`/`monster.h`/`npc.h`), `condition: "dyntype == <Subclass>"`
- [ ] 2.2 For each in-scope behavioral virtual (think/walk lifecycle, combat/death, target selection, visibility/type predicates), curate edges to its concrete overrides
- [ ] 2.3 Place edges in `flow_graph/nodes/{creature,player,monster,npc}.yml`

## 3. Coverage check

- [ ] 3.1 Implement a check that parses `override`/`override final` from the subclass headers, intersects with in-scope base virtuals (minus the allowlist), and asserts each has a curated edge
- [ ] 3.2 Add tests: missing override edge fails, allowlisted accessor not required, complete coverage passes

## 4. Verification

- [ ] 4.1 Run `make flow` (+ the override coverage check) and confirm it passes with the curated virtual edges
- [ ] 4.2 Confirm no file under `forgottenserver-upstream/src/` or any Rust crate was modified
