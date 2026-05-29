## ADDED Requirements

### Requirement: Challenge packet wire format
The game server SHALL send a 14-byte challenge packet immediately upon accepting a TCP game connection. The packet MUST conform to the Tibia protocol framing so that standard game clients (otclient, Tibia 13.x) can parse and respond.

Wire layout (all little-endian):
```
[0..2)  outer_len  u16 = 12   — bytes following this field
[2..6)  adler32    u32        — adler32 of bytes [6..14)
[6..8)  inner_len  u16 = 6    — bytes following inner_len
[8)     opcode     u8  = 0x1F — challenge opcode
[9..13) timestamp  u32        — current Unix seconds
[13)    rand       u8         — random byte [0, 255]
```

#### Scenario: Challenge packet is exactly 14 bytes
- **WHEN** the server accepts a TCP connection on the game port
- **THEN** the first bytes sent to the client are exactly 14 bytes

#### Scenario: Challenge outer_len is 12
- **WHEN** the challenge packet is read by the client
- **THEN** bytes [0..2) of the packet decode as little-endian u16 equal to 12

#### Scenario: Challenge adler32 covers inner_len through rand
- **WHEN** the challenge packet is inspected
- **THEN** bytes [2..6) equal adler32([6..14)) — the checksum of the 8-byte region starting at inner_len

#### Scenario: Challenge opcode is 0x1F
- **WHEN** the challenge packet is decoded
- **THEN** byte [8] equals 0x1F

### Requirement: First-packet read handles correct outer_len
The game server SHALL use the first 2 bytes of the client's response as the outer_len and read exactly outer_len bytes as the packet body.

#### Scenario: Valid outer_len under 32 KB succeeds
- **WHEN** the client sends a binary Tibia first packet with a well-formed outer_len (≤ 600 bytes)
- **THEN** the server reads the full packet body without error

#### Scenario: Implausibly large outer_len disconnects gracefully
- **WHEN** the client sends bytes whose first 2 bytes decode to outer_len > 32 768
- **THEN** the server closes the connection and logs a diagnostic message without panicking

### Requirement: Session-key round-trip
After a successful HTTP login, the game server SHALL accept the first game packet that contains the base64-decoded session token returned by the HTTP login response.

#### Scenario: Valid session token accepted
- **WHEN** the client sends a first packet containing the correct session token (base64-decoded 16 random bytes)
- **THEN** the server finds the session in the database and continues the login

#### Scenario: Unknown session token rejected
- **WHEN** the client sends a first packet with an unrecognised session token
- **THEN** the server sends a disconnect packet with an appropriate error message and closes the connection
