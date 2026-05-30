## ADDED Requirements

### Requirement: main entry-point behavior is verified
The Rust `main` function SHALL match the C++ `main` behavior: parse CLI arguments, call `startServer`, and return exit code 0 on success.

#### Scenario: main calls argument handler before start
- **WHEN** the binary starts with no arguments
- **THEN** `argumentsHandler` is called and `startServer` is invoked

### Requirement: argumentsHandler produces correct config overrides
The Rust argument handler SHALL accept `--config`, `--data`, and `--ip`/`--port` flags and set the corresponding config values before boot.

#### Scenario: --config flag sets config path
- **WHEN** `--config /tmp/config.lua` is passed
- **THEN** the config file path used for loading is `/tmp/config.lua`

### Requirement: startServer orchestrates boot in C++ order
The Rust `startServer` SHALL call `mainLoader`, then `ServiceManager::run`. Any error in `mainLoader` SHALL invoke `startupErrorMessage` and abort.

#### Scenario: mainLoader failure triggers error message
- **WHEN** `mainLoader` returns an error
- **THEN** `startupErrorMessage` is called and `ServiceManager::run` is not called

### Requirement: badAllocationHandler is registered
The Rust boot sequence SHALL register a handler for allocation failures equivalent to `std::set_new_handler(badAllocationHandler)`.

#### Scenario: out-of-memory condition is handled gracefully
- **WHEN** memory allocation fails catastrophically
- **THEN** the process logs an OOM message before terminating

### Requirement: printServerVersion outputs correct version string
The Rust `printServerVersion` SHALL output the server name and version matching the C++ implementation.

#### Scenario: version string format matches C++
- **WHEN** `printServerVersion` is called
- **THEN** the output contains the server name and version number in the same format as the C++ implementation

### Requirement: ConfigManager::load parses all required keys
`ConfigManager::load` SHALL read a Lua config file and populate all keys enumerated in the C++ `ConfigManager` (integer and string variants).

#### Scenario: valid config file populates numeric keys
- **WHEN** a config file sets `worldType = "pvp"` and `maxPlayers = 1000`
- **THEN** `getNumber(MAX_PLAYERS)` returns 1000 and `getString(WORLD_TYPE)` returns `"pvp"`

### Requirement: ConfigManager::setNumber and setString update internal state
Both setters SHALL update the in-memory config map and be observable by the corresponding getters.

#### Scenario: setNumber round-trips via getNumber
- **WHEN** `setNumber(key, 42)` is called
- **THEN** `getNumber(key)` returns 42

### Requirement: mainLoader runs subsystem init in the correct order
`mainLoader` SHALL initialize subsystems in the same order as the C++: config → DB → game state → map/XML loads → scripting → service registration.

#### Scenario: map load is skipped if DB setup fails
- **WHEN** the database is not set up and `isDatabaseSetup` returns false
- **THEN** `loadMainMap` is not called and boot aborts

### Requirement: Scheduler::shutdown drains the scheduler queue
`Scheduler::shutdown` SHALL stop accepting new tasks and drain any pending events before returning.

#### Scenario: no tasks execute after shutdown
- **WHEN** `shutdown` is called
- **THEN** tasks added after shutdown are not executed

### Requirement: Dispatcher::addTask enqueues a task for execution
`Dispatcher::addTask` SHALL accept a `Task` and schedule it for execution on the dispatcher thread.

#### Scenario: added task eventually runs
- **WHEN** a task is added via `addTask`
- **THEN** the task's callback is invoked

### Requirement: Dispatcher::shutdown joins the dispatcher thread
`Dispatcher::shutdown` SHALL signal the dispatch loop to stop and join the thread.

#### Scenario: dispatcher thread exits after shutdown
- **WHEN** `shutdown` is called
- **THEN** the dispatcher thread completes and no further tasks run

### Requirement: ServiceManager::run blocks until shutdown signal
`ServiceManager::run` SHALL start all registered services and block until a shutdown signal is received.

#### Scenario: run returns after shutdown is requested
- **WHEN** `ServiceManager::run` is called and then shutdown is signaled
- **THEN** `run` returns without error
