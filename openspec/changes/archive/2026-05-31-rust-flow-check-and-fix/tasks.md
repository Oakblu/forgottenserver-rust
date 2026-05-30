## 1. Boot / Init — depth 0-2 (main, argumentsHandler, helpers)

- [x] 1.1 Verify `main` (src/main.cpp, depth 0): read C++ source, confirm Rust entry calls argumentsHandler then startServer, write test if missing
- [x] 1.2 Verify `argumentsHandler` (src/main.cpp, depth 1): read C++ source, confirm Rust parses --config/--data/--ip/--port flags correctly, write test if missing
- [x] 1.3 Verify `badAllocationHandler` (src/otserv.cpp, depth 2): confirm Rust registers an OOM handler equivalent, write test if missing
- [x] 1.4 Verify `printServerVersion` (src/otserv.cpp, depth 2): confirm Rust outputs version string in same format as C++, write test if missing
- [x] 1.5 Verify `startupErrorMessage` (src/otserv.cpp, depth 2): confirm Rust logs error and marks boot failed, write test if missing
- [x] 1.6 Verify `ConfigManager::setNumber` (src/configmanager.cpp, depth 2): confirm setter updates internal map and getter reads it back, write test if missing
- [x] 1.7 Verify `ConfigManager::setString` (src/configmanager.cpp, depth 2): confirm setter updates internal map and getter reads it back, write test if missing
- [x] 1.8 Verify `Scheduler::shutdown` (src/scheduler.cpp, depth 2): confirm shutdown drains queue and joins thread, no tasks run after, write test if missing
- [x] 1.9 Verify `Dispatcher::addTask` (src/tasks.cpp, depth 2): confirm added task is eventually executed, write test if missing
- [x] 1.10 Verify `Dispatcher::shutdown` (src/tasks.cpp, depth 2): confirm shutdown joins thread and no further tasks run, write test if missing

## 2. Boot / Init — depth 2-3 (startServer, mainLoader, ServiceManager)

- [x] 2.1 Verify `startServer` (src/otserv.cpp, depth 1): confirm it calls mainLoader then ServiceManager::run, and aborts on mainLoader error, write test if missing
- [x] 2.2 Verify `mainLoader` (src/otserv.cpp, depth 2): confirm boot subsystem ordering matches C++ (config→DB→game state→XML→scripting→service), write test if missing
- [x] 2.3 Verify `ServiceManager::run` (src/server.cpp, depth 2): confirm it blocks until shutdown signal, write test if missing
- [x] 2.4 Verify `ConfigManager::load` (src/configmanager.cpp, depth 3): confirm all numeric and string keys are populated from a Lua config file, write test if missing

## 3. Database / Storage — depth 3-4

- [x] 3.1 Verify `Database::getInstance` (src/database.h, depth 3): confirm singleton pattern, repeated calls return same object, write test if missing
- [x] 3.2 Verify `Database::getClientVersion` (src/database.h, depth 3): confirm non-empty version string after connect, write test if missing
- [x] 3.3 Verify `Database::connect` (src/database.cpp, depth 3): confirm returns false with invalid credentials and logs error, write test if missing
- [x] 3.4 Verify `DatabaseManager::isDatabaseSetup` (src/databasemanager.cpp, depth 3): confirm returns false when schema absent, write test if missing
- [x] 3.5 Verify `DatabaseManager::optimizeTables` (src/databasemanager.cpp, depth 3): confirm runs OPTIMIZE TABLE without error, write test if missing
- [x] 3.6 Verify `DatabaseManager::updateDatabase` (src/databasemanager.cpp, depth 3): confirm up-to-date DB runs no migrations, write test if missing
- [x] 3.7 Verify `DatabaseTasks::start` (src/databasetasks.cpp, depth 3): confirm tasks added after start are executed on the DB thread, write test if missing
- [x] 3.8 Verify `DatabaseTasks::shutdown` (src/databasetasks.cpp, depth 2): confirm tasks added after shutdown are not executed, write test if missing

## 4. Items / Map / Vocation / Outfit load — depth 3

- [x] 4.1 Verify `Items::loadFromOtb` (src/items.cpp, depth 3): confirm item type with known ID is accessible after load, write test if missing
- [x] 4.2 Verify `Items::loadFromXml` (src/items.cpp, depth 3): confirm item name is populated from XML, write test if missing
- [x] 4.3 Verify `Monsters::loadFromXml` (src/monsters.cpp, depth 3): confirm a known monster is registered after load, write test if missing
- [x] 4.4 Verify `Outfits::loadFromXml` (src/outfit.cpp, depth 3): confirm a known outfit is registered after load, write test if missing
- [x] 4.5 Verify `Outfits::getInstance` (src/outfit.h, depth 3): confirm singleton pattern, write test if missing
- [x] 4.6 Verify `Vocations::loadFromXml` (src/vocation.cpp, depth 3): confirm vocation ID 1 has non-zero mana multiplier, write test if missing

## 5. Scripting bootstrap — depth 3-4

- [x] 5.1 Verify `loadPEM` (src/rsa.cpp, depth 3): confirm valid PEM file initializes RSA key, write test if missing
- [x] 5.2 Verify `Scripts::loadScripts` (src/script.cpp, depth 3): confirm load returns true on valid data directory, write test if missing
- [x] 5.3 Verify `ScriptingManager::loadScriptSystems` (src/scriptmanager.cpp, depth 3): confirm loads without error, write test if missing
- [x] 5.4 Verify `ScriptingManager::getInstance` (src/scriptmanager.h, depth 3): confirm singleton, write test if missing
- [x] 5.5 Verify `tfs::lua::reserveScriptEnv` (src/luascript.cpp, depth 4): confirm returns non-null and increments env counter, write test if missing
- [x] 5.6 Verify `tfs::lua::resetScriptEnv` (src/luascript.cpp, depth 4): confirm decrements env counter to pre-reserve value, write test if missing
- [x] 5.7 Verify `tfs::lua::getBoolean` (src/luascript.cpp, depth 4): confirm returns true/false for true/false Lua values, write test if missing

## 6. Game state initialization — depth 3-4

- [x] 6.1 Verify `Game::loadMainMap` (src/game.cpp, depth 3): confirm tile at origin exists after map load, write test if missing
- [x] 6.2 Verify `Game::setGameState` (src/game.cpp, depth 3): confirm GAME_STATE_NORMAL fires GlobalEvents::startup, write test if missing — GlobalEvents integration lives in scripting crate; game crate tests confirm state storage via `set_game_state_normal`; intentional boundary
- [x] 6.3 Verify `Game::setWorldType` (src/game.cpp, depth 3): confirm setWorldType/getWorldType round-trips correctly, write test if missing
- [x] 6.4 Verify `Game::start` (src/game.cpp, depth 3): confirm checkCreatures task is scheduled after start, write test if missing — `Game::start` is intentionally_removed (MIGRATION_LEDGER); scheduling moved to tokio-based server crate; `tick_creature_bucket_advances_bucket_index` covers bucket advancement
- [x] 6.5 Verify `Houses::payHouses` (src/house.cpp, depth 3): confirm owner retains house when balance is sufficient; owner loses house when balance is insufficient, write tests if missing
- [x] 6.6 Verify `tfs::http::start` (src/http/http.cpp, depth 3): confirm binds on a free port without error, write test if missing — covered by `start_binds_loopback_when_pinned_and_spawns_requested_workers` in crates/server/src/http.rs
- [x] 6.7 Verify `tfs::http::stop` (src/http/http.cpp, depth 4): confirm stops cleanly after start, write test if missing — covered by `start_then_stop_joins_all_workers` in crates/server/src/http.rs
- [x] 6.8 Verify `tfs::iomarket::checkExpiredOffers` (src/iomarket.cpp, depth 3): confirm expired offer is removed, write test if missing
- [x] 6.9 Verify `tfs::iomarket::updateStatistics` (src/iomarket.cpp, depth 3): confirm runs without error, write test if missing

## 7. Weapons and Chat load — depth 4

- [x] 7.1 Verify `Weapons::loadDefaults` (src/weapons.cpp, depth 4): confirm sword weapon has non-zero attack after load, write test if missing
- [x] 7.2 Verify `Chat::load` (src/chat.cpp, depth 4): confirm Help channel exists after load, write test if missing
- [x] 7.3 Verify `tfs::events::load` (src/events.cpp, depth 4): confirm loads valid event script without error, write test if missing

## 8. GlobalEvents — depth 4

- [x] 8.1 Verify `GlobalEvents::startup` (src/globalevent.cpp, depth 4): confirm all STARTUP-type callbacks are invoked, write test if missing
- [x] 8.2 Verify `GlobalEvents::shutdown` (src/globalevent.cpp, depth 4): confirm all SHUTDOWN-type callbacks are invoked, write test if missing
- [x] 8.3 Verify `GlobalEvents::save` (src/globalevent.cpp, depth 4): confirm all SAVE-type callbacks are invoked, write test if missing

## 9. Game periodic loops — depth 4

- [x] 9.1 Verify `Game::checkCreatures` (src/game.cpp, depth 4): confirm reschedules itself each tick, write test if missing
- [x] 9.2 Verify `Game::checkDecay` (src/game.cpp, depth 4): confirm expired item is removed from tile, write test if missing
- [x] 9.3 Verify `Game::updateCreaturesPath` (src/game.cpp, depth 4): confirm creature movement queue updated, write test if missing
- [x] 9.4 Verify `Game::shutdown` (src/game.cpp, depth 4): confirm all logged-in players are saved before service stops, write test if missing

## 10. IOLoginData / IOBan — depth 4-6

- [x] 10.1 Verify `IOLoginData::loadPlayerById` (src/iologindata.cpp, depth 4): confirm player name matches DB record after load, write test if missing
- [x] 10.2 Verify `IOLoginData::preloadPlayer` (src/iologindata.cpp, depth 6): confirm player name is populated, write test if missing
- [x] 10.3 Verify `IOBan::getIpBanInfo` (src/ban.cpp, depth 5): confirm unbanned IP returns no active ban, write test if missing
- [x] 10.4 Verify `IOBan::getAccountBanInfo` (src/ban.cpp, depth 6): confirm unbanned account returns no active ban, write test if missing
- [x] 10.5 Verify `IOBan::isPlayerNamelocked` (src/ban.cpp, depth 6): confirm non-namelocked player returns false, write test if missing

## 11. Protocol — RSA and Connection — depth 3-5

- [x] 11.1 Verify `loadPEM` initializes RSA (src/rsa.cpp, depth 3): already covered in 5.1; verify RSA_decrypt succeeds on encrypted test vector
- [x] 11.2 Verify `Protocol::RSA_decrypt` (src/protocol.cpp, depth 5): confirm 128-byte block is decrypted and cursor advances; first decrypted byte != 0 returns false, write tests if missing
- [x] 11.3 Verify `Connection::parsePacket` (src/connection.cpp, depth 3): confirm first packet routes to onRecvFirstMessage; subsequent packets route to parsePacket, write tests if missing
- [x] 11.4 Verify `Connection::handleTimeout` (src/connection.cpp, depth 4): confirm timeout closes the connection, write test if missing
- [x] 11.5 Verify `ConnectionManager::getInstance` (src/connection.h, depth 5): confirm singleton, write test if missing — intentionally_removed; replaced by per-handler scoping (documented in MIGRATION_LEDGER.yml)

## 12. ProtocolStatus — depth 4-5

- [x] 12.1 Verify `ProtocolStatus::onRecvFirstMessage` (src/protocolstatus.cpp, depth 4): confirm 0xFF routes to sendInfo, 0x01 routes to sendStatusString, write tests if missing
- [x] 12.2 Verify `ProtocolStatus::sendInfo` (src/protocolstatus.cpp, depth 5): confirm response XML contains `<players online="N">`, write test if missing
- [x] 12.3 Verify `ProtocolStatus::sendStatusString` (src/protocolstatus.cpp, depth 5): confirm byte string matches C++ format, write test if missing

## 13. ProtocolGame session — depth 4-5

- [x] 13.1 Verify `ProtocolGame::onRecvFirstMessage` (src/protocolgame.cpp, depth 4): confirm unsupported version rejected; IP-banned client rejected; valid credentials dispatch login task, write tests if missing
- [x] 13.2 Verify `ProtocolGame::parsePacket` (src/protocolgame.cpp, depth 4): confirm known opcode dispatches to correct handler; unknown opcode is discarded without disconnect, write tests if missing
- [x] 13.3 Verify `ProtocolGame::login` (src/protocolgame.cpp, depth 5): confirm account-banned player rejected; successful login sends initial game state, write tests if missing — intentionally_removed; login/logout logic lives in game crate, network crate is pure codec
- [x] 13.4 Verify `ProtocolGame::logout` (src/protocolgame.cpp, depth 5): confirm CreatureEvents::playerLogout is invoked before player removed, write test if missing — intentionally_removed; same as 13.3

## 14. Opcode handlers — movement and interaction — depth 5

- [x] 14.1 Verify `ProtocolGame::parseAutoWalk` (src/protocolgame.cpp, depth 5): direction sequence decoded correctly, write test if missing
- [x] 14.2 Verify `ProtocolGame::parseSay` (src/protocolgame.cpp, depth 5): private message type/receiver/text decoded correctly, write test if missing
- [x] 14.3 Verify `ProtocolGame::parseUseItem` (src/protocolgame.cpp, depth 5): position/itemId/stackpos decoded correctly, write test if missing
- [x] 14.4 Verify `ProtocolGame::parseUseItemEx` (src/protocolgame.cpp, depth 5): from-pos/itemId/stackpos/to-pos decoded correctly, write test if missing
- [x] 14.5 Verify `ProtocolGame::parseUseWithCreature` (src/protocolgame.cpp, depth 5): position/itemId/stackpos/creatureId decoded correctly, write test if missing
- [x] 14.6 Verify `ProtocolGame::parseLookAt` (src/protocolgame.cpp, depth 5): position/stackpos decoded correctly, write test if missing
- [x] 14.7 Verify `ProtocolGame::parseLookInBattleList` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 14.8 Verify `ProtocolGame::parseAttack` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 14.9 Verify `ProtocolGame::parseFollow` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 14.10 Verify `ProtocolGame::parseFightModes` (src/protocolgame.cpp, depth 5): attack/chase/secure bytes decoded correctly, write test if missing
- [x] 14.11 Verify `ProtocolGame::parseThrow` (src/protocolgame.cpp, depth 5): from-pos/to-pos/itemId/stackpos/count decoded correctly, write test if missing
- [x] 14.12 Verify `ProtocolGame::parseRotateItem` (src/protocolgame.cpp, depth 5): position/itemId/stackpos decoded correctly, write test if missing
- [x] 14.13 Verify `ProtocolGame::parseWrapItem` (src/protocolgame.cpp, depth 5): position/itemId/stackpos decoded correctly, write test if missing
- [x] 14.14 Verify `ProtocolGame::parseBrowseField` (src/protocolgame.cpp, depth 5): position decoded correctly, write test if missing
- [x] 14.15 Verify `ProtocolGame::parseEquipObject` (src/protocolgame.cpp, depth 5): itemId/subtype decoded correctly, write test if missing

## 15. Opcode handlers — container and text — depth 5

- [x] 15.1 Verify `ProtocolGame::parseCloseContainer` (src/protocolgame.cpp, depth 5): container ID decoded correctly, write test if missing
- [x] 15.2 Verify `ProtocolGame::parseUpArrowContainer` (src/protocolgame.cpp, depth 5): container ID decoded correctly, write test if missing
- [x] 15.3 Verify `ProtocolGame::parseSeekInContainer` (src/protocolgame.cpp, depth 5): container ID and seek index decoded correctly, write test if missing
- [x] 15.4 Verify `ProtocolGame::parseUpdateContainer` (src/protocolgame.cpp, depth 5): container ID decoded correctly, write test if missing
- [x] 15.5 Verify `ProtocolGame::parseTextWindow` (src/protocolgame.cpp, depth 5): windowId/text decoded correctly, write test if missing
- [x] 15.6 Verify `ProtocolGame::parseHouseWindow` (src/protocolgame.cpp, depth 5): windowId/doorId/text decoded correctly, write test if missing

## 16. Opcode handlers — channel and VIP — depth 5

- [x] 16.1 Verify `ProtocolGame::parseOpenChannel` (src/protocolgame.cpp, depth 5): channel ID decoded correctly, write test if missing
- [x] 16.2 Verify `ProtocolGame::parseCloseChannel` (src/protocolgame.cpp, depth 5): channel ID decoded correctly, write test if missing
- [x] 16.3 Verify `ProtocolGame::parseChannelInvite` (src/protocolgame.cpp, depth 5): invitee name decoded correctly, write test if missing
- [x] 16.4 Verify `ProtocolGame::parseChannelExclude` (src/protocolgame.cpp, depth 5): excluded name decoded correctly, write test if missing
- [x] 16.5 Verify `ProtocolGame::parseOpenPrivateChannel` (src/protocolgame.cpp, depth 5): receiver name decoded correctly, write test if missing
- [x] 16.6 Verify `ProtocolGame::parseAddVip` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 16.7 Verify `ProtocolGame::parseEditVip` (src/protocolgame.cpp, depth 5): creature ID/description/icon/notify decoded correctly, write test if missing
- [x] 16.8 Verify `ProtocolGame::parseRemoveVip` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing

## 17. Opcode handlers — party — depth 5

- [x] 17.1 Verify `ProtocolGame::parseInviteToParty` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 17.2 Verify `ProtocolGame::parseJoinParty` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 17.3 Verify `ProtocolGame::parseRevokePartyInvite` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 17.4 Verify `ProtocolGame::parsePassPartyLeadership` (src/protocolgame.cpp, depth 5): creature ID decoded correctly, write test if missing
- [x] 17.5 Verify `ProtocolGame::parseEnableSharedPartyExperience` (src/protocolgame.cpp, depth 5): bool flag decoded correctly, write test if missing

## 18. Opcode handlers — shop and market — depth 5

- [x] 18.1 Verify `ProtocolGame::parseLookInShop` (src/protocolgame.cpp, depth 5): itemId/count decoded correctly, write test if missing
- [x] 18.2 Verify `ProtocolGame::parsePlayerPurchase` (src/protocolgame.cpp, depth 5): all purchase fields decoded correctly, write test if missing
- [x] 18.3 Verify `ProtocolGame::parsePlayerSale` (src/protocolgame.cpp, depth 5): all sale fields decoded correctly, write test if missing
- [x] 18.4 Verify `ProtocolGame::parseMarketBrowse` (src/protocolgame.cpp, depth 5): own-offers request and item-ID request both decoded correctly, write tests if missing
- [x] 18.5 Verify `ProtocolGame::parseMarketCreateOffer` (src/protocolgame.cpp, depth 5): type/itemId/amount/price decoded correctly, write test if missing
- [x] 18.6 Verify `ProtocolGame::parseMarketCancelOffer` (src/protocolgame.cpp, depth 5): offer ID decoded correctly, write test if missing
- [x] 18.7 Verify `ProtocolGame::parseMarketAcceptOffer` (src/protocolgame.cpp, depth 5): offer ID/amount decoded correctly, write test if missing
- [x] 18.8 Verify `ProtocolGame::parseMarketLeave` (src/protocolgame.cpp, depth 5): dispatches leave-market action, write test if missing

## 19. Opcode handlers — trade — depth 5

- [x] 19.1 Verify `ProtocolGame::parseRequestTrade` (src/protocolgame.cpp, depth 5): position/itemId/stackpos/creatureId decoded correctly, write test if missing
- [x] 19.2 Verify `ProtocolGame::parseLookInTrade` (src/protocolgame.cpp, depth 5): otherPlayer flag and slot index decoded correctly, write test if missing

## 20. Opcode handlers — outfit, modal, misc — depth 5

- [x] 20.1 Verify `ProtocolGame::parseSetOutfit` (src/protocolgame.cpp, depth 5): all outfit color/mount fields decoded correctly, write test if missing
- [x] 20.2 Verify `ProtocolGame::parseEditPodiumRequest` (src/protocolgame.cpp, depth 5): position/outfit/direction decoded correctly, write test if missing
- [x] 20.3 Verify `ProtocolGame::parseModalWindowAnswer` (src/protocolgame.cpp, depth 5): windowId/buttonId/choiceId decoded correctly, write test if missing
- [x] 20.4 Verify `ProtocolGame::parseExtendedOpcode` (src/protocolgame.cpp, depth 5): channel/payload decoded correctly, write test if missing
- [x] 20.5 Verify `ProtocolGame::parseDebugAssert` (src/protocolgame.cpp, depth 5): packet consumed without panic, write test if missing
- [x] 20.6 Verify `ProtocolGame::parseRuleViolationReport` (src/protocolgame.cpp, depth 5): packet consumed without panic, write test if missing

## 21. Game player-action dispatchers — depth 5

- [x] 21.1 Verify `Game::playerMove` (src/game.cpp, depth 5): player position updates by one step in the given direction when tile is walkable, write test if missing — intentionally_removed (MIGRATION_LEDGER); covered by `test_start_auto_walk_single_queues_one_step` in entity crate
- [x] 21.2 Verify `Game::playerReceivePing` (src/game.cpp, depth 5): last-ping timestamp updated, write test if missing
- [x] 21.3 Verify `Game::playerReceivePingBack` (src/game.cpp, depth 5): latency field updated, write test if missing — intentionally_removed (MIGRATION_LEDGER); covered by `test_serialize_ping_back` in network crate
- [x] 21.4 Verify `Game::playerRequestChannels` (src/game.cpp, depth 5): at least one public channel sent to player, write test if missing — intentionally_removed (MIGRATION_LEDGER); covered by channel tests in network crate
- [x] 21.5 Verify `Game::playerRequestOutfit` (src/game.cpp, depth 5): at least one outfit option sent, write test if missing — intentionally_removed (MIGRATION_LEDGER); covered by outfit parse tests in network crate
- [x] 21.6 Verify `Game::playerTurn` (src/game.cpp, depth 5): player direction updated to the requested direction, write test if missing
- [x] 21.7 Verify `Game::playerStopAutoWalk` (src/game.cpp, depth 5): player walk path is cleared, write test if missing
- [x] 21.8 Verify `Game::playerCancelAttackAndFollow` (src/game.cpp, depth 5): both attack and follow targets cleared, write test if missing — intentionally_removed (MIGRATION_LEDGER); covered by `test_creature_clear_attack_target` and `test_follow_target_set_clear` in entity crate
- [x] 21.9 Verify `Game::playerLeaveParty` (src/game.cpp, depth 5): leader leaving disbands the party, write test if missing
- [x] 21.10 Verify `Game::playerCreatePrivateChannel` (src/game.cpp, depth 5): player receives channel-open packet, write test if missing — intentionally_removed (MIGRATION_LEDGER); covered by `test_serialize_create_private_channel` in network crate + `create_private_channel_owner_is_creating_player` in game crate
- [x] 21.11 Verify `Game::playerCloseNpcChannel` (src/game.cpp, depth 5): player has no active NPC conversation after call, write test if missing — intentionally_removed (MIGRATION_LEDGER); NPC channel state tracked in entity crate
- [x] 21.12 Verify `Game::playerCloseShop` (src/game.cpp, depth 5): player shop reference cleared, write test if missing — intentionally_removed (MIGRATION_LEDGER); shop state tracked in entity crate
- [x] 21.13 Verify `Game::playerAcceptTrade` (src/game.cpp, depth 5): items exchanged when both sides accept, write test if missing
- [x] 21.14 Verify `Game::playerCloseTrade` (src/game.cpp, depth 5): onTradeCompleted invoked with CANCELLED outcome, write test if missing

## 22. Depth-6 nodes — events, chat, login helpers, output message

- [x] 22.1 Verify `IOBan::getAccountBanInfo` (src/ban.cpp, depth 6): unbanned account returns no active ban, write test if missing (may overlap with 10.4 — confirm coverage)
- [x] 22.2 Verify `IOBan::isPlayerNamelocked` (src/ban.cpp, depth 6): non-namelocked player returns false, write test if missing (may overlap with 10.5 — confirm coverage)
- [x] 22.3 Verify `Chat::createChannel` (src/chat.cpp, depth 6): new channel owner is the creating player, write test if missing
- [x] 22.4 Verify `CreatureEvents::playerLogout` (src/creatureevent.cpp, depth 6): all LOGOUT callbacks invoked, write test if missing
- [x] 22.5 Verify `tfs::events::player::onTradeAccept` (src/events.cpp, depth 6): Lua callback invoked with both player references, write test if missing
- [x] 22.6 Verify `tfs::events::player::onTradeCompleted` (src/events.cpp, depth 6): Lua callback invoked with both players and outcome, write test if missing
- [x] 22.7 Verify `tfs::events::player::onTurn` (src/events.cpp, depth 6): Lua callback invoked with player and direction, write test if missing
- [x] 22.8 Verify `Game::getPlayerByName` (src/game.cpp, depth 6): online player found by exact name; offline player returns None, write tests if missing
- [x] 22.9 Verify `IOLoginData::preloadPlayer` (src/iologindata.cpp, depth 6): player name populated after preload, write test if missing (may overlap with 10.2)
- [x] 22.10 Verify `tfs::net::make_output_message` (src/outputmessage.cpp, depth 6): write cursor starts after length header bytes, write test if missing — intentionally_removed (MIGRATION_LEDGER); `OutputMessage::new()` sets `write_pos = HEADER_LENGTH (2)`; covered by `test_new_write_pos_is_header_length` and `test_new_message_length_is_zero` in crates/common/src/outputmessage.rs
- [x] 22.11 Verify `tfs::net::insert_protocol_to_autosend` (src/outputmessage.cpp, depth 6): protocol appears in pending-send list after insertion, write test if missing — intentionally_removed (MIGRATION_LEDGER); autosend pipeline moved to server crate (tokio-based); `tests_required: []` in ledger

## 23. Final validation

- [x] 23.1 Run `cargo test --lib --workspace` and confirm zero failures
- [x] 23.2 Run `cargo clippy --workspace --lib --tests -- -D warnings` and confirm zero warnings
- [x] 23.3 Run `make flow-gap` and confirm GAP_REPORT still shows 0 actionable findings
- [x] 23.4 Run `make flow` and confirm all flow-graph validators pass
