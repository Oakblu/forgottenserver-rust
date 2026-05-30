## ADDED Requirements

### Requirement: GlobalEvents::startup fires all startup events
`GlobalEvents::startup` SHALL iterate all registered global events of type STARTUP and invoke their Lua callbacks in registration order.

#### Scenario: registered startup events are called
- **WHEN** `startup` is called with two registered STARTUP events
- **THEN** both Lua callbacks are invoked

### Requirement: GlobalEvents::shutdown fires all shutdown events
`GlobalEvents::shutdown` SHALL invoke all SHUTDOWN-type global event callbacks.

#### Scenario: shutdown events are called on shutdown
- **WHEN** `shutdown` is called with registered SHUTDOWN events
- **THEN** all SHUTDOWN callbacks are invoked

### Requirement: GlobalEvents::save fires all save events
`GlobalEvents::save` SHALL invoke all SAVE-type global event callbacks (triggered on world-save timer).

#### Scenario: save events are called when world save occurs
- **WHEN** `save` is called
- **THEN** all SAVE callbacks are invoked

### Requirement: tfs::events::load parses and registers all event scripts
`tfs::events::load` SHALL parse the events configuration script and register player, creature, and party event hooks.

#### Scenario: load succeeds with valid event script
- **WHEN** a valid events Lua script is loaded
- **THEN** no error is returned and event handlers are registered

### Requirement: tfs::events::player::onTradeAccept fires the Lua callback
`tfs::events::player::onTradeAccept` SHALL invoke the registered `onTradeAccept` Lua function with both player references.

#### Scenario: trade-accept event calls Lua callback
- **WHEN** `onTradeAccept` is called with two player references
- **THEN** the registered Lua `onTradeAccept` callback is invoked with both players

### Requirement: tfs::events::player::onTradeCompleted fires the Lua callback
`tfs::events::player::onTradeCompleted` SHALL invoke the registered `onTradeCompleted` Lua function with both player references and outcome.

#### Scenario: trade-completed event calls Lua callback
- **WHEN** `onTradeCompleted` is called
- **THEN** the registered Lua callback is invoked

### Requirement: tfs::events::player::onTurn fires the Lua callback
`tfs::events::player::onTurn` SHALL invoke the registered `onTurn` Lua function with the player reference and direction.

#### Scenario: turn event calls Lua callback
- **WHEN** `onTurn` is called with player and direction NORTH
- **THEN** the registered Lua `onTurn` callback is invoked with direction NORTH

### Requirement: Game::checkCreatures processes creature update tick
`Game::checkCreatures` SHALL iterate all creatures, call their `onCreatureThink`, and reschedule itself for the next tick.

#### Scenario: check-creatures reschedules after running
- **WHEN** `checkCreatures` is called
- **THEN** it reschedules itself for the next interval via the scheduler

### Requirement: Game::checkDecay removes expired items from the decay list
`Game::checkDecay` SHALL process the decay queue and remove any items whose decay timer has expired from the map.

#### Scenario: expired item is removed from the tile
- **WHEN** `checkDecay` is called with an item whose decay timer has expired
- **THEN** the item is removed from its tile

### Requirement: Game::updateCreaturesPath re-evaluates pathfinding for moving creatures
`Game::updateCreaturesPath` SHALL call `getNextStep` for all creatures with an active follow or auto-walk path and update their move queue.

#### Scenario: creature with active path receives next-step update
- **WHEN** `updateCreaturesPath` is called with a creature following a path
- **THEN** the creature's movement queue is updated

### Requirement: Game::shutdown persists world state and signals stop
`Game::shutdown` SHALL save all player data, stop decay/creature timers, and signal the `ServiceManager` to stop.

#### Scenario: shutdown saves all logged-in players
- **WHEN** `shutdown` is called with logged-in players
- **THEN** all players are saved to the database before the service stops

### Requirement: CreatureEvents::playerLogout fires logout callbacks
`CreatureEvents::playerLogout` SHALL invoke all registered LOGOUT creature event callbacks for the player.

#### Scenario: logout event fires registered callbacks
- **WHEN** `playerLogout` is called with a player
- **THEN** all LOGOUT callbacks for that player are invoked

### Requirement: tfs::lua::reserveScriptEnv acquires a Lua script environment slot
`tfs::lua::reserveScriptEnv` SHALL return a pointer to a free `ScriptEnvironment` slot, advancing the stack counter.

#### Scenario: reserve returns a non-null environment
- **WHEN** `reserveScriptEnv` is called
- **THEN** the returned pointer is non-null and the env counter increments

### Requirement: tfs::lua::resetScriptEnv releases the current Lua script environment
`tfs::lua::resetScriptEnv` SHALL decrement the env counter and call `resetEnv` on the top slot.

#### Scenario: reset decrements the env counter
- **WHEN** `resetScriptEnv` is called after a reserve
- **THEN** the env counter returns to its pre-reserve value

### Requirement: tfs::lua::getBoolean reads a boolean from the Lua stack
`tfs::lua::getBoolean` SHALL return the boolean value at the given stack index; non-boolean values are treated as true if non-nil/false.

#### Scenario: true value at stack index returns true
- **WHEN** Lua stack position -1 holds `true`
- **THEN** `getBoolean(L, -1)` returns true

#### Scenario: false value at stack index returns false
- **WHEN** Lua stack position -1 holds `false`
- **THEN** `getBoolean(L, -1)` returns false
