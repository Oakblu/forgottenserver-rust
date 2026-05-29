## 1. Tooling — Extend find_stubs.py with allowlist support

- [x] 1.1 Add `--allowlist <path>` flag to `find_stubs.py`; default to `scripts/confirmed_stubs.json` when the file exists
- [x] 1.2 Implement body-hash lookup: compute SHA-256 of each function body and compare against allowlist entries
- [x] 1.3 Partition detected stubs into `"unresolved"` and `"confirmed"` arrays in the JSON output; emit `WARN:` to stderr on hash mismatch
- [x] 1.4 Preserve backward-compat: continue writing the legacy flat-array `stub_report.json` alongside the new structured format
- [x] 1.5 Add unit tests in `scripts/test_find_stubs.py` for allowlist loading, hash matching, hash mismatch warning, and partition output
- [x] 1.6 Run `python3 scripts/test_find_stubs.py` — all tests pass

## 2. Triage — `game` and `server` crates (2 stubs)

- [x] 2.1 Classify `game/src/weapons.rs::get_element_damage` (line 285): check C++ `WeaponWand::getElementDamage` at `forgottenserver-upstream/src/weapons.h:219`; add confirming test asserting returns `0`; write triage entry
- [x] 2.2 Classify `server/src/http.rs::accept_loop` (line 218, `dropped_work`): confirm cross-crate deferred in `intentional_differences.yml` (id: `http-server-boost-asio-to-tokio`); write triage entry with `intentional_diff_id`
- [x] 2.3 Add both to `scripts/confirmed_stubs.json`
- [x] 2.4 Run `cargo test --lib -p game -p server` — all tests pass

## 3. Triage — `world` crate (1 stub)

- [x] 3.1 Classify `world/src/house.rs::can_transform` (line 1043): check C++ `Item::canTransform` virtual; add confirming test asserting returns `false` for a house-tile item; write triage entry
- [x] 3.2 Add to `scripts/confirmed_stubs.json`
- [x] 3.3 Run `cargo test --lib -p world` — all tests pass

## 4. Triage — `database` crate (4 stubs)

- [x] 4.1 Classify `database/src/database.rs::begin_transaction`, `commit`, `rollback` as `intentional-deferred` (id: `database-adapter-helpers-deferred-to-mariadb-adapter-prod`); write triage entries
- [x] 4.2 Classify `database/src/databasemanager.rs::optimize_tables` as `intentional-deferred` (id: `database-adapter-helpers-deferred-to-mariadb-adapter-prod`); write triage entry
- [x] 4.3 Add all 4 to `scripts/confirmed_stubs.json`
- [x] 4.4 Run `cargo test --lib -p database` — all tests pass

## 5. Triage — `scripting` crate (3 stubs)

- [x] 5.1 Classify `scripting/src/engine.rs::load_script` (line 71): `NoopScriptEngine` returns `Ok(())` — `correct-default` matching trait method no-op contract; add confirming test; write triage entry
- [x] 5.2 Classify `scripting/src/engine.rs::reset` (line 83): same as above — `correct-default`; add confirming test; write triage entry
- [x] 5.3 Classify `scripting/src/engine.rs::register_event` (line 87): same — `correct-default`; add confirming test; write triage entry
- [x] 5.4 Add all 3 to `scripts/confirmed_stubs.json`
- [x] 5.5 Run `cargo test --lib -p scripting` — all tests pass

## 6. Triage — `entity` crate (8 stubs)

- [x] 6.1 Classify `entity/src/creature.rs::is_attackable` (line 816): C++ `creature.h:269 virtual bool isAttackable() const { return true; }` — `correct-default`; add confirming test
- [x] 6.2 Classify `entity/src/creature.rs::can_see_invisibility` (line 821): C++ `creature.h:133 virtual bool canSeeInvisibility() const { return false; }` — `correct-default`; add confirming test
- [x] 6.3 Classify `entity/src/creature.rs::can_see_ghost_mode` (line 826): C++ `creature.h:135 virtual bool canSeeGhostMode(...) const { return false; }` — `correct-default`; add confirming test
- [x] 6.4 Classify `entity/src/creature.rs::is_creature` (line 983): C++ `Thing::is_creature` override returns `true` — `correct-default`; add confirming test
- [x] 6.5 Classify `entity/src/creature.rs::is_removable` (line 989): C++ `Thing::is_removable` default `true` — `correct-default`; add confirming test
- [x] 6.6 Classify `entity/src/creature.rs::get_throw_range` (line 999): C++ `Thing::get_throw_range` default `1` — `correct-default`; add confirming test
- [x] 6.7 Classify `entity/src/player.rs::post_add_notification` (line 1253, `empty_body`): `intentional-deferred` (id: `cross-crate-behavior-dispatch-deferred-to-game-glue`); write triage entry
- [x] 6.8 Classify `entity/src/player.rs::post_remove_notification` (line 1256, `empty_body`): same as above; write triage entry
- [x] 6.9 Add all 8 to `scripts/confirmed_stubs.json`
- [x] 6.10 Run `cargo test --lib -p entity` — all tests pass

## 7. Triage — `map` crate (13 stubs)

- [x] 7.1 Classify `map/src/housetile.rs::query_add_for_non_member`, `query_add_item_for_non_member`, `query_add_non_player_creature`, `query_remove_for_non_member` (4 stubs): `intentional-deferred` — house entity cross-crate dispatch deferred; verify or add `intentional_differences.yml` entry; write triage entries
- [x] 7.2 Classify `map/src/tile.rs::post_add_item`, `post_remove_item` (2 stubs, `empty_body`): `intentional-deferred` — spectator system lives in game crate; verify `intentional_differences.yml` entry; write triage entries
- [x] 7.3 Classify `map/src/tile.rs::get_first_index` and `map/src/tile.rs::cylinder_first_index` (2 stubs): check C++ `Tile::getFirstIndex` — `correct-default` or `correct-default`; add confirming tests; write triage entries
- [x] 7.4 Classify `map/src/tile.rs::is_removed` (1 stub): verify Rust returns `false` is correct placeholder given no parent pointer; classify and add confirming test or document as deferred
- [x] 7.5 Classify `map/src/trashholder.rs::query_add` (1 stub, `Ok(())`): C++ `TrashHolder::queryAdd` inline returns `RETURNVALUE_NOERROR` — `correct-default`; add confirming test
- [x] 7.6 Classify `map/src/trashholder.rs::item_count` (1 stub, `0`): trash holders hold 0 items — `correct-default`; add confirming test
- [x] 7.7 Classify `map/src/trashholder.rs::add_item` (1 stub): `drop(item)` — check if C++ `addThing` cross-crate dispatch (calls `g_game.internalRemoveItem`) requires recording as deferred
- [x] 7.8 Add all classified map stubs to `scripts/confirmed_stubs.json`
- [x] 7.9 Run `cargo test --lib -p map` — all tests pass

## 8. Triage — `items` crate (27 stubs)

- [x] 8.1 Classify `items/src/container.rs::is_container`, `is_item`, `is_removed`, `get_first_index`, `cylinder_first_index` (5 stubs): check C++ Container header for inline overrides — `correct-default`; add confirming tests
- [x] 8.2 Classify `items/src/depotchest.rs::can_remove`, `is_container`, `is_item`, `is_removed` (4 stubs): C++ inline `canRemove() { return false; }` etc. — `correct-default`; add confirming tests
- [x] 8.3 Classify `items/src/depotlocker.rs::query_add`, `can_remove`, `is_container`, `is_item`, `is_removed` (5 stubs): C++ inline `queryAdd → NOTENOUGHROOM`, `canRemove → false` — `correct-default`; add confirming tests
- [x] 8.4 Classify `items/src/inbox.rs::can_remove`, `is_container`, `is_item`, `is_removed` (4 stubs): C++ inline `canRemove → false` — `correct-default`; add confirming tests
- [x] 8.5 Classify `items/src/item.rs::can_remove`, `can_transform` (2 stubs): check C++ `Item::canRemove → true` default and `Item::canTransform → true` default; add confirming tests
- [x] 8.6 Classify `items/src/mailbox.rs::store_item`, `is_mailbox` (2 stubs): `store_item → Err(CannotStore)` is cross-crate deferred routing (MailboxRouting); `is_mailbox → true` is `correct-default`; document store_item in `intentional_differences.yml` if not already covered; add confirming test for is_mailbox
- [x] 8.7 Classify `items/src/storeinbox.rs::is_store_inbox`, `can_remove`, `is_container`, `is_item`, `is_removed` (5 stubs): C++ inlines — `correct-default`; add confirming tests
- [x] 8.8 Add all items-crate stubs to `scripts/confirmed_stubs.json`
- [x] 8.9 Run `cargo test --lib -p items` — all tests pass

## 9. Triage — `common` crate (37 stubs)

- [x] 9.1 Classify `common/src/cylinder.rs::cylinder_post_add`, `cylinder_post_remove` (2 empty-body stubs): trait defaults — `intentional-deferred`; verify or add `intentional_differences.yml` entry (id: `cylinder-class-to-trait`)
- [x] 9.2 Classify `common/src/cylinder.rs::cylinder_first_index`, `cylinder_last_index`, `cylinder_item_type_count`, `cylinder_thing_index` (4 stubs): C++ Cylinder base returns 0/None — `correct-default`; add confirming tests
- [x] 9.3 Classify `common/src/luavariant.rs::get_number`, `get_position`, `get_target_position`, `get_string` (4 panic stubs): type-safety assertions — `panic-correct`; add confirming tests asserting correct variant returns value and wrong variant panics
- [x] 9.4 Classify `common/src/outputmessage.rs::get_header_position` (1 stub, `0`): C++ `OutputMessage::getHeaderPosition` returns `0` — `correct-default`; add confirming test
- [x] 9.5 Classify `common/src/thing.rs` stubs (25 stubs): all are trait default methods — `correct-default` for literal-value returns matching C++ `Thing` virtual defaults; add one confirming test per method
- [x] 9.6 Classify `common/src/thread_holder_base.rs::u8_to_state` (1 panic stub): panics on invalid u8 — `panic-correct`; add confirming test asserting valid values map correctly and invalid value panics
- [x] 9.7 Add all common-crate stubs to `scripts/confirmed_stubs.json`
- [x] 9.8 Run `cargo test --lib -p common` — all tests pass (Note: crate may be named `forgottenserver-common`)

## 10. Finalization

- [x] 10.1 Run `python3 scripts/find_stubs.py` — verify `"unresolved": []` or document any remaining entries with rationale
- [x] 10.2 Run `cargo test --lib --workspace` — all tests pass
- [x] 10.3 Run `cargo clippy --workspace --lib --tests -- -D warnings` — zero warnings
- [x] 10.4 Update `MIGRATION_LEDGER.yml` for all files where stubs have been resolved: advance status from `PARTIAL` to `DONE` where 100% line coverage is now achieved
- [x] 10.5 Verify `scripts/stub_triage.json` has 95 entries matching all `stub_report.json` entries
- [x] 10.6 Verify `scripts/confirmed_stubs.json` has entries for all non-`needs-implementation` stubs
