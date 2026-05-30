## ADDED Requirements

### Requirement: Database::getInstance returns the singleton connection
`Database::getInstance` SHALL return the global `Database` instance, creating it on first call.

#### Scenario: repeated calls return the same instance
- **WHEN** `getInstance` is called twice
- **THEN** both return the same object

### Requirement: Database::connect establishes a MariaDB connection
`Database::connect` SHALL read host, port, user, password, and database name from config and open a connection.

#### Scenario: connection fails with wrong credentials
- **WHEN** `connect` is called with invalid credentials
- **THEN** it returns false and logs an error

### Requirement: Database::getClientVersion returns the server-reported version string
`Database::getClientVersion` SHALL return the MariaDB server version string after a successful connection.

#### Scenario: version string is non-empty after connect
- **WHEN** `getClientVersion` is called after a successful connect
- **THEN** the returned string is non-empty

### Requirement: DatabaseManager::isDatabaseSetup checks schema existence
`DatabaseManager::isDatabaseSetup` SHALL query for the `server_config` table and return true only if it exists.

#### Scenario: missing table returns false
- **WHEN** the schema has not been applied
- **THEN** `isDatabaseSetup` returns false

### Requirement: DatabaseManager::updateDatabase applies pending migrations
`DatabaseManager::updateDatabase` SHALL compare the current DB version against the target version and run all pending migration scripts in order.

#### Scenario: up-to-date database runs no migrations
- **WHEN** the DB version equals the latest migration
- **THEN** no migration script is executed

### Requirement: DatabaseManager::optimizeTables runs OPTIMIZE TABLE on key tables
`DatabaseManager::optimizeTables` SHALL call `OPTIMIZE TABLE` on the player and item tables.

#### Scenario: optimize runs without error on valid connection
- **WHEN** `optimizeTables` is called
- **THEN** no error is returned

### Requirement: DatabaseTasks::start launches the async DB task thread
`DatabaseTasks::start` SHALL start the background thread that drains the DB task queue.

#### Scenario: tasks added after start are executed
- **WHEN** a task is added after `start`
- **THEN** the task is executed on the DB thread

### Requirement: DatabaseTasks::shutdown drains remaining tasks and joins the thread
`DatabaseTasks::shutdown` SHALL process any remaining queued tasks and join the background thread.

#### Scenario: no tasks run after shutdown
- **WHEN** tasks are added after `shutdown`
- **THEN** those tasks are not executed

### Requirement: Items::loadFromOtb parses the .otb item binary file
`Items::loadFromOtb` SHALL parse the OTB file and populate the item type registry with all item IDs.

#### Scenario: item type with known ID is accessible after load
- **WHEN** `loadFromOtb` is called with a valid .otb file
- **THEN** `getItemType(2160)` returns a non-default item type

### Requirement: Items::loadFromXml parses items.xml and enriches item types
`Items::loadFromXml` SHALL parse items.xml and populate name, weight, and attribute fields for each item type.

#### Scenario: item name is set from XML
- **WHEN** `loadFromXml` is called with a valid items.xml
- **THEN** `getItemType(2160).name` is non-empty

### Requirement: Monsters::loadFromXml parses all monster definition files
`Monsters::loadFromXml` SHALL read all `.xml` files in the monster directory and register them in the monster registry.

#### Scenario: a known monster is registered after load
- **WHEN** `loadFromXml` is called
- **THEN** `getMonsterType("Rat")` returns a valid definition

### Requirement: Outfits::loadFromXml parses outfit definitions
`Outfits::loadFromXml` SHALL parse outfit.xml and register all outfits with their properties.

#### Scenario: outfit with known ID is registered
- **WHEN** `loadFromXml` is called
- **THEN** a registered outfit with look type 128 can be retrieved

### Requirement: Outfits::getInstance returns the singleton
`Outfits::getInstance` SHALL return the global `Outfits` instance.

#### Scenario: repeated calls return the same instance
- **WHEN** `getInstance` is called twice
- **THEN** both return the same object

### Requirement: Vocations::loadFromXml parses vocation definitions
`Vocations::loadFromXml` SHALL parse vocations.xml and register all vocations with their stats and skill multipliers.

#### Scenario: known vocation is registered after load
- **WHEN** `loadFromXml` is called
- **THEN** vocation ID 1 (Sorcerer) is accessible with non-zero mana multiplier

### Requirement: loadPEM loads the RSA private key from a PEM file
`loadPEM` SHALL read the PEM file specified in config and initialize the RSA private key used for game login decryption.

#### Scenario: valid PEM file initializes the key
- **WHEN** `loadPEM` is called with a valid PEM file
- **THEN** subsequent `RSA_decrypt` calls succeed on valid ciphertext

### Requirement: Scripts::loadScripts loads all Lua scripts from the data directory
`Scripts::loadScripts` SHALL enumerate and execute all `.lua` files in the configured scripts directory.

#### Scenario: script load returns true on success
- **WHEN** `loadScripts` is called with a valid data directory
- **THEN** it returns true and no script errors are logged

### Requirement: ScriptingManager::loadScriptSystems loads core script systems
`ScriptingManager::loadScriptSystems` SHALL load the global Lua script systems (actions, talk actions, move events, etc.).

#### Scenario: script systems load without error
- **WHEN** `loadScriptSystems` is called
- **THEN** it returns true

### Requirement: ScriptingManager::getInstance returns the singleton
`ScriptingManager::getInstance` SHALL return the global `ScriptingManager` instance.

#### Scenario: repeated calls return the same instance
- **WHEN** `getInstance` is called twice
- **THEN** both return the same object

### Requirement: Game::loadMainMap loads the OTBM world map
`Game::loadMainMap` SHALL load the `.otbm` map file and populate the `Map` with tiles, items, and spawn data.

#### Scenario: known tile exists after map load
- **WHEN** `loadMainMap` is called with a valid .otbm file
- **THEN** the tile at the map's origin position is non-null

### Requirement: Game::setGameState transitions the game state machine
`Game::setGameState` SHALL update the global game state and fire the appropriate GlobalEvents callbacks.

#### Scenario: transition to GAME fires startup events
- **WHEN** `setGameState(GAME_STATE_NORMAL)` is called
- **THEN** `GlobalEvents::startup` is invoked

### Requirement: Game::setWorldType stores the world type
`Game::setWorldType` SHALL set the world type (PVP, NOPVP, PVP-ENFORCED) used by combat checks.

#### Scenario: world type round-trips correctly
- **WHEN** `setWorldType(WORLD_TYPE_PVP)` is called
- **THEN** `getWorldType()` returns `WORLD_TYPE_PVP`

### Requirement: Game::start launches periodic game timers
`Game::start` SHALL schedule the first `checkCreatures`, `checkDecay`, and `updateCreaturesPath` calls.

#### Scenario: check-creatures is scheduled after start
- **WHEN** `Game::start` is called
- **THEN** a `checkCreatures` task is pending in the scheduler

### Requirement: Houses::payHouses deducts house rent from owner accounts
`Houses::payHouses` SHALL iterate all houses, query owner balance, deduct rent, and evict players who cannot pay.

#### Scenario: owner with sufficient balance retains house
- **WHEN** `payHouses` is called and the owner has enough gold
- **THEN** the house owner is unchanged

#### Scenario: owner with insufficient balance loses house
- **WHEN** `payHouses` is called and the owner has insufficient gold
- **THEN** the house is evicted (owner cleared)

### Requirement: tfs::http::start binds and starts the HTTP listener
`tfs::http::start` SHALL bind to the configured HTTP address and port and begin accepting connections.

#### Scenario: start succeeds on a free port
- **WHEN** `start` is called with an available port
- **THEN** it returns without error

### Requirement: tfs::http::stop closes the HTTP listener
`tfs::http::stop` SHALL close the listener socket and stop accepting new connections.

#### Scenario: stop after start runs without error
- **WHEN** `stop` is called after `start`
- **THEN** no error is returned

### Requirement: tfs::iomarket::checkExpiredOffers removes expired market offers
`tfs::iomarket::checkExpiredOffers` SHALL query for offers past their expiry time and remove them from the market.

#### Scenario: expired offer is removed from the market
- **WHEN** an offer's expiry time has passed and `checkExpiredOffers` is called
- **THEN** the offer is no longer retrievable

### Requirement: tfs::iomarket::updateStatistics recalculates market price statistics
`tfs::iomarket::updateStatistics` SHALL update the recent transaction price averages for all traded items.

#### Scenario: statistics update runs without error
- **WHEN** `updateStatistics` is called
- **THEN** no error is returned

### Requirement: Weapons::loadDefaults loads weapon attribute defaults from XML
`Weapons::loadDefaults` SHALL parse weapons.xml and register attack, defense, and element properties for each weapon item type.

#### Scenario: known weapon has non-zero attack after load
- **WHEN** `loadDefaults` is called
- **THEN** a registered sword-type weapon has a non-zero attack value

### Requirement: Chat::load initializes default channels
`Chat::load` SHALL create the default public channels (Advertise, Help, World, etc.) and register them in the channel registry.

#### Scenario: Help channel exists after load
- **WHEN** `Chat::load` is called
- **THEN** the Help channel is retrievable by ID

### Requirement: Chat::createChannel creates a new private channel
`Chat::createChannel` SHALL allocate a new channel with a unique ID and register the creating player as the owner.

#### Scenario: created channel has the player as owner
- **WHEN** `createChannel` is called with player P
- **THEN** the new channel's owner is P

### Requirement: IOLoginData::loadPlayerById loads full player data from the database
`IOLoginData::loadPlayerById` SHALL query all player tables and populate the `Player` struct completely.

#### Scenario: player with known ID has correct name after load
- **WHEN** `loadPlayerById` is called with a known player ID
- **THEN** the loaded player name matches the database record

### Requirement: IOLoginData::preloadPlayer loads the minimal set of player data for login checks
`IOLoginData::preloadPlayer` SHALL load name, account ID, and group without loading items, spells, or storage.

#### Scenario: preloaded player has name set
- **WHEN** `preloadPlayer` is called
- **THEN** the player name field is populated

### Requirement: IOBan::getIpBanInfo returns ban info for a banned IP
`IOBan::getIpBanInfo` SHALL query the database for an active ban on the given IP address.

#### Scenario: unbanned IP returns no active ban
- **WHEN** `getIpBanInfo` is called with an unbanned IP
- **THEN** the returned ban info indicates no active ban

### Requirement: IOBan::getAccountBanInfo returns ban info for a banned account
`IOBan::getAccountBanInfo` SHALL query for an active ban on the given account ID.

#### Scenario: unbanned account returns no active ban
- **WHEN** `getAccountBanInfo` is called with an unbanned account ID
- **THEN** the returned ban info indicates no active ban

### Requirement: IOBan::isPlayerNamelocked returns true if the player name is locked
`IOBan::isPlayerNamelocked` SHALL query for a namelock on the given player ID.

#### Scenario: player with no namelock returns false
- **WHEN** `isPlayerNamelocked` is called with a player not namelocked
- **THEN** it returns false

### Requirement: Game::getPlayerByName returns the correct player or null
`Game::getPlayerByName` SHALL perform a case-insensitive lookup in the online player registry.

#### Scenario: online player found by exact name
- **WHEN** a player named "Alice" is online and `getPlayerByName("Alice")` is called
- **THEN** the correct player is returned

#### Scenario: offline player returns null
- **WHEN** no player named "Bob" is online
- **THEN** `getPlayerByName("Bob")` returns None

### Requirement: tfs::net::make_output_message creates a framed output buffer
`tfs::net::make_output_message` SHALL allocate an output buffer with the correct header space reserved for length framing.

#### Scenario: output message has correct initial write position
- **WHEN** `make_output_message` is called
- **THEN** the write cursor starts after the length header bytes

### Requirement: tfs::net::insert_protocol_to_autosend registers a protocol for auto-send
`tfs::net::insert_protocol_to_autosend` SHALL add the protocol to the connection's pending-send list.

#### Scenario: protocol is in the send list after insertion
- **WHEN** a protocol is inserted via `insert_protocol_to_autosend`
- **THEN** it appears in the connection's pending-send list

### Requirement: startupErrorMessage logs a fatal error and marks boot as failed
`startupErrorMessage` SHALL log the error message to stderr/log and set a flag that prevents `ServiceManager::run` from starting.

#### Scenario: error message is logged
- **WHEN** `startupErrorMessage("fatal")` is called
- **THEN** "fatal" appears in the error log output

### Requirement: Game::playerMove processes a movement direction
`Game::playerMove` SHALL validate the direction, check walkability, move the player creature, and send the updated position to the client.

#### Scenario: movement in valid direction updates player position
- **WHEN** `playerMove(player, DIRECTION_NORTH)` is called and the tile is walkable
- **THEN** the player's position is updated by one step north

### Requirement: Game::playerReceivePing acknowledges a client ping
`Game::playerReceivePing` SHALL reset the player's ping timer and optionally send a pong packet.

#### Scenario: ping resets idle timer
- **WHEN** `playerReceivePing` is called
- **THEN** the player's last-ping timestamp is updated

### Requirement: Game::playerReceivePingBack acknowledges a ping-back
`Game::playerReceivePingBack` SHALL record the round-trip time.

#### Scenario: ping-back updates latency
- **WHEN** `playerReceivePingBack` is called
- **THEN** the player's latency field is updated

### Requirement: Game::playerRequestChannels sends the channel list to the player
`Game::playerRequestChannels` SHALL send all available channels the player can join.

#### Scenario: channel list contains at least the public channels
- **WHEN** `playerRequestChannels` is called
- **THEN** the sent list includes at least one public channel

### Requirement: Game::playerRequestOutfit sends outfit options to the player
`Game::playerRequestOutfit` SHALL send the available outfits and mounts for the player's sex and premium status.

#### Scenario: outfit options are sent to the player
- **WHEN** `playerRequestOutfit` is called
- **THEN** at least one outfit option is sent

### Requirement: Game::playerTurn rotates the player to face the given direction
`Game::playerTurn` SHALL update the player creature's direction and broadcast the change to spectators.

#### Scenario: player direction is updated
- **WHEN** `playerTurn(player, DIRECTION_EAST)` is called
- **THEN** `player.direction` becomes EAST

### Requirement: Game::playerStopAutoWalk cancels the player's auto-walk path
`Game::playerStopAutoWalk` SHALL clear the player's pending walk path.

#### Scenario: auto-walk path is cleared
- **WHEN** `playerStopAutoWalk` is called
- **THEN** the player's walk path is empty

### Requirement: Game::playerCancelAttackAndFollow cancels both attack and follow targets
`Game::playerCancelAttackAndFollow` SHALL clear the attack target and follow target simultaneously.

#### Scenario: attack and follow targets are both cleared
- **WHEN** `playerCancelAttackAndFollow` is called
- **THEN** both `attackedCreature` and `followCreature` are None

### Requirement: Game::playerLeaveParty removes the player from their party
`Game::playerLeaveParty` SHALL call `Party::removePlayer` and disband the party if the player was the leader.

#### Scenario: leader leaving disbands the party
- **WHEN** the party leader calls `playerLeaveParty`
- **THEN** the party is disbanded

### Requirement: Game::playerCreatePrivateChannel creates a private chat channel
`Game::playerCreatePrivateChannel` SHALL call `Chat::createChannel` and send the new channel info to the player.

#### Scenario: new channel is sent to the creating player
- **WHEN** `playerCreatePrivateChannel` is called
- **THEN** the player receives a channel-open packet with the new channel ID

### Requirement: Game::playerCloseNpcChannel closes an NPC dialog channel
`Game::playerCloseNpcChannel` SHALL close any open NPC conversation for the player.

#### Scenario: NPC channel is closed
- **WHEN** `playerCloseNpcChannel` is called
- **THEN** the player has no active NPC conversation

### Requirement: Game::playerCloseShop closes the shop interface for the player
`Game::playerCloseShop` SHALL remove the player's active shop reference and notify the NPC.

#### Scenario: shop reference is cleared
- **WHEN** `playerCloseShop` is called
- **THEN** the player's shop reference is None

### Requirement: Game::playerAcceptTrade completes a pending trade
`Game::playerAcceptTrade` SHALL set the player's accept flag and complete the trade if both sides have accepted.

#### Scenario: trade completes when both sides accept
- **WHEN** both players have called `playerAcceptTrade`
- **THEN** items are exchanged and the trade is closed

### Requirement: Game::playerCloseTrade cancels and removes a trade
`Game::playerCloseTrade` SHALL call `tfs::events::player::onTradeCompleted` with outcome CANCELLED and return items.

#### Scenario: cancelled trade fires the trade-completed event
- **WHEN** `playerCloseTrade` is called
- **THEN** `onTradeCompleted` is invoked with outcome CANCELLED
