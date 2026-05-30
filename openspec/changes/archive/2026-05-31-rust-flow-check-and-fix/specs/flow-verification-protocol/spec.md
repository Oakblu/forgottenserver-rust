## ADDED Requirements

### Requirement: Connection::parsePacket dispatches to the correct protocol handler
`Connection::parsePacket` SHALL read the first byte of an incoming buffer, look up the registered `Protocol` for that service, and forward the buffer to the protocol's `onRecvFirstMessage` (first connection) or `parsePacket` (subsequent packets).

#### Scenario: first packet is routed to onRecvFirstMessage
- **WHEN** a new TCP connection sends its first data
- **THEN** `Protocol::onRecvFirstMessage` is called with the raw message buffer

#### Scenario: subsequent packets are routed to parsePacket
- **WHEN** a connection that has already received its first message sends another packet
- **THEN** `Protocol::parsePacket` is called

### Requirement: Connection::handleTimeout closes the connection
`Connection::handleTimeout` SHALL close the connection and release associated resources when the read deadline expires.

#### Scenario: timeout closes the connection
- **WHEN** the read deadline expires without data received
- **THEN** the connection is closed and the protocol is notified

### Requirement: Protocol::RSA_decrypt decrypts a game login payload
`Protocol::RSA_decrypt` SHALL decrypt the RSA-encrypted block at the current buffer position using the private key, and advance the read cursor.

#### Scenario: valid RSA block is decrypted correctly
- **WHEN** a 128-byte RSA-encrypted block is in the message
- **THEN** the decrypted plaintext is placed at the same buffer position and the read cursor advances by 128

#### Scenario: first byte of decrypted block must be zero
- **WHEN** the first decrypted byte is not 0
- **THEN** `RSA_decrypt` returns false indicating a failed handshake

### Requirement: ProtocolGame::onRecvFirstMessage validates client version and sets up session
`ProtocolGame::onRecvFirstMessage` SHALL: RSA-decrypt the handshake, check the client version, verify account credentials, check IP ban, and dispatch a login task.

#### Scenario: unsupported client version is rejected
- **WHEN** a client sends a version outside the supported range
- **THEN** `onRecvFirstMessage` sends a disconnect message and does not queue a login task

#### Scenario: IP-banned client is rejected
- **WHEN** `IOBan::getIpBanInfo` returns an active ban
- **THEN** `onRecvFirstMessage` sends a ban message and closes the connection

#### Scenario: valid credentials proceed to login dispatch
- **WHEN** version is valid and no IP ban exists
- **THEN** a login task is dispatched to the game dispatcher

### Requirement: ProtocolGame::parsePacket routes opcodes to handler functions
`ProtocolGame::parsePacket` SHALL read one opcode byte from the message and dispatch to the corresponding `parse*` handler function via the opcode dispatch table.

#### Scenario: known opcode dispatches to correct handler
- **WHEN** an incoming message starts with opcode 0x64 (move north)
- **THEN** `parseAutoWalk` is called with the remaining message buffer

#### Scenario: unknown opcode is logged and discarded
- **WHEN** an incoming message starts with an unrecognized opcode byte
- **THEN** the packet is discarded and the connection remains open

### Requirement: ProtocolGame::login completes session establishment
`ProtocolGame::login` SHALL load player data, verify character name, check account ban, set up the player session, and send the initial map and player data to the client.

#### Scenario: account-banned player is rejected on login
- **WHEN** `IOBan::getAccountBanInfo` returns an active ban
- **THEN** `login` sends a ban message and disconnects

#### Scenario: successful login sends initial game state
- **WHEN** all checks pass and the player is loaded
- **THEN** the client receives the initial map description and player attributes

### Requirement: ProtocolGame::logout removes the player from the game
`ProtocolGame::logout` SHALL invoke `CreatureEvents::playerLogout`, save the player to the database, and remove the player creature from the game world.

#### Scenario: logout triggers playerLogout event
- **WHEN** `logout` is called
- **THEN** `CreatureEvents::playerLogout` is invoked before the player is removed

### Requirement: ConnectionManager::getInstance returns the singleton instance
`ConnectionManager::getInstance` SHALL return the global `ConnectionManager` instance, creating it on first call.

#### Scenario: repeated calls return the same instance
- **WHEN** `getInstance` is called twice
- **THEN** both calls return the same object

### Requirement: ProtocolStatus::onRecvFirstMessage handles status requests
`ProtocolStatus::onRecvFirstMessage` SHALL read the request type byte and dispatch to `sendInfo` or `sendStatusString`.

#### Scenario: request type 0xFF triggers sendInfo
- **WHEN** the first byte is 0xFF
- **THEN** `sendInfo` is called

#### Scenario: request type 0x01 triggers sendStatusString
- **WHEN** the first byte is 0x01
- **THEN** `sendStatusString` is called

### Requirement: ProtocolStatus::sendInfo sends server statistics XML
`ProtocolStatus::sendInfo` SHALL compose and send an XML document containing the server name, players online, max players, and uptime.

#### Scenario: response contains player count
- **WHEN** `sendInfo` is called with N players online
- **THEN** the response XML contains `<players online="N">`

### Requirement: ProtocolStatus::sendStatusString sends a plain-text status
`ProtocolStatus::sendStatusString` SHALL send a short plain-text status line with server name and online count.

#### Scenario: status string matches expected format
- **WHEN** `sendStatusString` is called
- **THEN** the response byte string matches the C++ format
