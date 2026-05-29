## ADDED Requirements

### Requirement: XTEA keys are enabled after burst
After the enter-world burst is flushed to the TCP stream the server SHALL enable XTEA encryption for all subsequent outbound packets and decryption for all subsequent inbound packets, using the key extracted from the login RSA block.

#### Scenario: Post-burst outbound packets are XTEA-encrypted
- **WHEN** the server sends any packet after the enter-world burst
- **THEN** the payload is encrypted with the session XTEA round keys before writing to the stream

#### Scenario: Post-burst inbound packets are XTEA-decrypted before dispatch
- **WHEN** the server reads a packet after the burst is sent
- **THEN** `decrypt_message` is called before the opcode is extracted

---

### Requirement: Game loop reads and dispatches packets
After the enter-world burst the server SHALL enter a blocking read loop:
1. Read 2-byte outer length
2. Read `outer_length` bytes into a buffer
3. Call `decrypt_message` with the session XTEA key
4. Validate Adler32 checksum (use `validate_adler32`); drop packet on failure
5. Extract inner length; validate it does not exceed outer length
6. Extract opcode (first byte of inner payload)
7. Dispatch to the appropriate handler from `game_handler.rs` based on opcode
8. Loop until read error or connection close

#### Scenario: Walk packet dispatched
- **WHEN** the client sends an encrypted walk packet (opcode `0x65`)
- **THEN** after XTEA decrypt the server calls the walk handler and sends the updated map description back

#### Scenario: Invalid checksum drops packet silently
- **WHEN** a received packet fails `validate_adler32`
- **THEN** the server discards the packet and continues the loop without disconnecting

#### Scenario: Connection close exits loop cleanly
- **WHEN** the TCP read returns 0 bytes (client disconnected)
- **THEN** the game loop exits and the thread terminates

---

### Requirement: Read timeout prevents thread starvation
The server SHALL set a read timeout of 30 seconds on the game connection socket. If no data arrives within 30 seconds of the last read the server SHALL close the connection and exit the loop.

#### Scenario: Idle connection is closed after timeout
- **WHEN** no packet is received for 30 seconds after the last read
- **THEN** the socket read returns a timeout error and the server closes the connection
