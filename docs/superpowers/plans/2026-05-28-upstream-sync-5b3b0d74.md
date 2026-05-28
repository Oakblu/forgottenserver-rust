# Upstream Sync — 2026-05-28 (5b3b0d74)

## Scope

Covers upstream `otland/forgottenserver` commits from the past **6 months**
(2025-11-28 → 2025-12-29). The default 3-month window had no new commits;
the last code-touching commit was `5b3b0d74` on 2025-12-29.

## Upstream Commits Being Addressed

| SHA | Date | Message | Files Changed |
|-----|------|---------|---------------|
| `afd91657` | 2025-11-28 | refactor(thing): remove Cylinder class and merge receiver logic into Thing (#5058) | 30+ files; `cylinder.cpp`, `cylinder.h` deleted |
| `133f91ea` | 2025-11-28 | refactor(player): rework onCreatureAppear to handle initial player world entry packets (#5030) | creature.cpp/.h, game.cpp, player.cpp/.h, monster.cpp/.h, npc.cpp/.h, protocolgame.cpp |
| `335e4da6` | 2025-12-03 | refactor(npc): remove generic creature speech bubble and restrict to NPCs only (#5066) | creature.h, npc.h, protocolgame.cpp |
| `3c684f91` | 2025-12-03 | fix(game): correct parent ref in transformItem postAddNotification (#5067) | game.cpp |
| `4d0b9794` | 2025-12-05 | refactor(player): remove onEquipInventory/onDeEquipInventory and inline inventory equip/unequip logic (#5065) | player.cpp, player.h |
| `5b3b0d74` | 2025-12-29 | fix(iomapserialize): add class Thing forward declaration (#5063) | iomapserialize.h |
| `cbc4d7bb` | 2026-01-28 | Fix item description to respect reading distance limits (#4912) | no .cpp/.h changes |

## Skipped Commits (no migration needed)

| SHA | Reason |
|-----|--------|
| `cbc4d7bb` | No .cpp/.h changes — nothing to port |
| `5b3b0d74` | Adds `class Thing;` forward declaration — compile-time only, no behavior change |

## Symbols Table (investigate targets)

All affected C++ files have ledger status `DONE`, meaning they were ported
as of the previous sync. The changes below represent **new upstream behavior**
applied to already-migrated code. Each must be cross-checked against the
current Rust implementation.

| Priority | Symbol / Area | C++ File | Upstream Commit | Ledger Status |
|----------|--------------|----------|-----------------|---------------|
| HIGH | `Game::transformItem` (bug fix: parent ref) | game.cpp | `3c684f91` | DONE |
| HIGH | `Player::onCreatureAppear` (signature + major rework) | player.cpp | `133f91ea` + `4d0b9794` | DONE |
| HIGH | Cylinder class deletion → merged into `Thing` | cylinder.cpp/.h (deleted); thing.h + 30 files | `afd91657` | cylinder: MISSING; rest: DONE |
| MEDIUM | `Creature::getSpeechBubble` removed from base class | creature.h | `335e4da6` | DONE |
| MEDIUM | `Player::onEquipInventory` / `onDeEquipInventory` removed | player.cpp/.h | `4d0b9794` | DONE |

## Per-Symbol Migration Steps

---

### 1. `Game::transformItem` — parent ref bug fix (`3c684f91`) — HIGH

**What changed upstream:**
```diff
-if (auto parent = item->getParent()) {
-    parent->postAddNotification(item, parent, parent->getThingIndex(item));
+if (auto itemParent = item->getParent()) {
+    itemParent->postAddNotification(item, parent, itemParent->getThingIndex(item));
     return item;
 }
```
The old code had a variable shadowing bug: `parent` was redeclared in the
inner scope, hiding the outer `parent`. The notification was being sent with
the wrong context: inner `parent` (same as `itemParent`) was used for both
the `cylinder` argument and the index lookup. The fix passes the **outer**
`parent` as the cylinder context and uses `itemParent` for the index.

**Steps:**
1. Read the relevant section of
   `forgottenserver-upstream/src/game.cpp` (search for `transformItem`) to
   confirm exact behavior at `postAddNotification` call site.
2. Locate the Rust equivalent in `crates/game/src/` — search for
   `transform_item` or `post_add_notification`.
3. Write a failing test that captures the correct context passed to
   `post_add_notification` when the item's parent differs from the
   destination parent.
4. Fix the Rust implementation to match.
5. Verify `cargo test --lib -p game` passes.
6. Update `MIGRATION_LEDGER.yml` `game.cpp` entry if needed (already DONE,
   but add a note about this fix).

---

### 2. `Player::onCreatureAppear` rework (`133f91ea` + `4d0b9794`) — HIGH

**What changed upstream (`133f91ea`):**
- Signature changed: `onCreatureAppear(Creature*, bool)` →
  `onCreatureAppear(Creature*, bool, MagicEffectClasses)`
- Early return if `creature != this`: just `sendAddCreature(creature, pos, effect)`
- Login path restructured: offline-time mute-condition decay now computed
  before `onEquipInventory`; `IOLoginData::updateOnlineStatus` moved inside
  this method.

**What changed upstream (`4d0b9794`):**
- `onEquipInventory()` body inlined into `onCreatureAppear` login path
- `onDeEquipInventory()` body inlined into `onRemoveCreature`
- Both helper methods removed from `player.cpp` and `player.h`

**Steps:**
1. Read `forgottenserver-upstream/src/player.cpp` around `onCreatureAppear`
   and `onRemoveCreature` (grep `onCreatureAppear`, ~60 lines).
2. Locate Rust equivalent — search for `on_creature_appear` in
   `crates/entity/src/` or `crates/game/src/`.
3. Write failing tests for:
   - Offline mute-condition tick-down on login
   - Inventory equip events fired during login in slot order
   - Inventory de-equip events fired during logout in slot order
4. Update Rust implementation to match inlined logic and new signature.
5. Verify `cargo test --lib --workspace` passes.

---

### 3. Cylinder class deletion → merged into `Thing` (`afd91657`) — HIGH

**What changed upstream:**
`Cylinder` (in `cylinder.cpp` / `cylinder.h`) was a base class providing
receiver/query methods (`queryAdd`, `queryMaxCount`, `queryRemove`,
`queryDestination`, `addThing`, `updateThing`, `replaceThing`,
`removeThing`, `postAddNotification`, `postRemoveNotification`,
`getThingIndex`, `getFirstIndex`, `getLastIndex`, `getThing`,
`getItemTypeCount`, `getAllItemTypeCount`, `internalRemoveThing`,
`internalAddThing`). These are now part of `Thing` directly.

`cylinder.cpp` and `cylinder.h` no longer exist in the upstream.

**Steps:**
1. Check whether the Rust port already has a `Cylinder` equivalent or
   merged these methods directly into a `Thing` trait/struct:
   ```bash
   grep -rn "cylinder\|Cylinder" crates/ --include="*.rs" | head -20
   ```
2. If Rust already has no `Cylinder` type and merges into `Thing`/a trait:
   verify all the methods listed above are present and correct. Add an
   entry to `intentional_differences.yml` recording that `Cylinder` was
   never a separate type in the Rust port (pre-dates this upstream removal).
3. If Rust still has a `Cylinder` type: plan its removal and merge into
   the `Thing` trait. Write tests first.
4. Update `MIGRATION_LEDGER.yml`:
   - Set `cylinder.cpp` and `cylinder.h` entries to `NON_GOAL` with note
     "Deleted upstream in afd91657; logic merged into Thing".

---

### 4. `Creature::getSpeechBubble` removed from base class (`335e4da6`) — MEDIUM

**What changed upstream:**
```diff
-virtual uint8_t getSpeechBubble() const { return SPEECHBUBBLE_NONE; }
```
Removed from `Creature`. `Npc::getSpeechBubble()` changed from `override`
to a non-virtual regular method. Speech bubbles are now NPC-only.

**Steps:**
1. Search for `get_speech_bubble` or `speech_bubble` in `crates/entity/src/`.
2. If `Creature` trait / base struct has `get_speech_bubble`: remove it;
   retain it only on the `Npc` struct.
3. Check `ProtocolGame` equivalent for any send-speech-bubble codepath that
   calls `creature.getSpeechBubble()` — update to NPC-only path.
4. Write a test: calling speech bubble on a non-NPC creature should not
   compile (type-level) or return a fixed NONE (if done via Option).
5. Verify `cargo test --lib --workspace` passes.

---

### 5. `Player::onEquipInventory` / `onDeEquipInventory` removal (`4d0b9794`) — MEDIUM

Covered above in step 2. If the Rust port has these as separate methods,
they should be inlined and removed. The behavior is identical — this is a
pure refactor.

---

## Verification Commands

```bash
cargo test --lib --workspace
python3 scripts/ledger/validate.py
python3 scripts/ledger/cross_validate.py
```

## Constraints (verbatim)

- **TDD always**: write a failing test before any implementation code
- **Wire format is a hard contract** — if any network opcode changes, flag immediately
- **Lua binding contract is strict** — function name, arg order, return shape must match C++ exactly
- **No git commits without explicit user permission**
