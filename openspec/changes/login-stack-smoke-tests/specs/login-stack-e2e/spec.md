## ADDED Requirements

### Requirement: Docker stack starts cleanly
The composed stack (MariaDB + Rust server) SHALL start without errors and the server SHALL log `>> Forgotten Server Online!` within 60 seconds of `docker compose up`.

#### Scenario: clean stack start
- **WHEN** `docker compose up --build` is run with no prior running containers
- **THEN** server logs contain `>> Forgotten Server Online!` and no `ERROR` or `panic` lines

### Requirement: No packet-framing error on game connection
After the HTTP login and game handshake, the server SHALL NOT log `failed to read packet body` (or any equivalent framing error) for any connection attempt using a supported client version.

#### Scenario: game connection does not produce framing error
- **WHEN** OTClient (version 1310 or 1311) attempts a game connection to port 7172
- **THEN** server logs contain no `failed to read packet body` lines

### Requirement: HTTP login returns a character list or account-not-found
The HTTP login server (port 8080) SHALL respond to OTClient's login POST with either a character list (if the account exists in the DB) or a structured `account-not-found` error — not a network-level failure.

#### Scenario: OTClient login dialog advances past network error
- **WHEN** OTClient is pointed at the server and the user submits credentials
- **THEN** the login dialog does not show `Failed to connect to server (HTTP)` — it shows either a character list or an account-not-found message

### Requirement: Full game session reaches the world
After HTTP login returns a character list and the player selects a character, OTClient SHALL render the game world — map tiles visible, player stats shown — with no ERROR 60 or unexpected disconnect.

#### Scenario: character selection renders the game world
- **WHEN** a valid character is selected after a successful HTTP login
- **THEN** the game world is rendered (map tiles visible, player HP/mana shown) and no disconnect or ERROR 60 occurs within 10 seconds of entry

### Requirement: Both listeners are logged at startup
The server SHALL log the start of both the HTTP listener (port 8080) and the game listener (port 7172) during boot.

#### Scenario: listener start lines appear in server log
- **WHEN** the server starts
- **THEN** logs contain lines indicating both the HTTP and game listeners are accepting connections
