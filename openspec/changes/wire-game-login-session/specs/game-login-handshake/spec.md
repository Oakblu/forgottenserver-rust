## ADDED Requirements

### Requirement: Server loads RSA private key at boot
The server SHALL call `rsa::load_pem` with the default OTLand RSA private key exactly once at startup, before any game listener accepts connections. If the key fails to load the server SHALL abort with a descriptive error.

#### Scenario: Key loads successfully
- **WHEN** the server starts with the embedded default RSA PEM
- **THEN** `rsa::load_pem` returns `Ok(())` and subsequent `rsa::decrypt` calls succeed

#### Scenario: Key load failure aborts boot
- **WHEN** a malformed PEM string is passed to `rsa::load_pem`
- **THEN** boot returns `Err` and the server does not start

---

### Requirement: Server stores challenge values per connection
For each accepted game connection, the server SHALL store the `challengeTimestamp` (u32) and `challengeRandom` (u8) sent in the challenge packet so they can be validated against the client's echo.

#### Scenario: Challenge values are captured
- **WHEN** the server sends the `0x1F` challenge packet
- **THEN** the timestamp (current unix seconds as u32) and a random u8 are stored for that connection

---

### Requirement: Server parses TFS 13 first packet wire format
The server SHALL parse `onRecvFirstMessage` exactly as C++ TFS 1.4 does:
1. Read OS (u16)
2. Read client version (u16); reject with disconnect if outside [1310, 1311]
3. Skip 4 bytes (client build)
4. If version >= 1240 and buffer has remaining bytes > 132: skip one length-prefixed string
5. Skip 3 bytes (dat revision u16 + preview state u8)
6. RSA-decrypt the next 128 bytes; reject with disconnect if byte 0 ≠ 0x00 after decrypt
7. Read XTEA key (4 × u32 LE) from decrypted bytes [1..17)
8. Skip 1 byte (gm_flag)
9. Read session_token string; base64-decode it; reject with disconnect if empty
10. Read character_name string
11. Read challenge echo: timestamp (u32) + random (u8)
12. Reject with disconnect if echo does not match stored challenge values

#### Scenario: Valid packet accepted
- **WHEN** OTClient sends a well-formed TFS 13 login packet with correct version and valid session token
- **THEN** the server extracts the XTEA key, session token, and character name without error

#### Scenario: Wrong client version rejected
- **WHEN** the client sends version 1098 (outside [1310, 1311])
- **THEN** the server sends a disconnect message and closes the connection

#### Scenario: RSA decrypt failure rejects connection
- **WHEN** the RSA-encrypted block is garbage (wrong key)
- **THEN** `rsa::decrypt` returns Err and the server closes the connection without a response

#### Scenario: Challenge echo mismatch rejects connection
- **WHEN** the client echoes a timestamp or random that differs from the stored challenge
- **THEN** the server closes the connection immediately

#### Scenario: Empty session token rejects connection
- **WHEN** the base64-decoded session token is empty
- **THEN** the server sends disconnect message "Malformed session key."

---

### Requirement: Server validates session token against DB
After parsing the first packet the server SHALL execute one SQL query joining `sessions`, `accounts`, and `players` to resolve `account_id` and `character_id`.

The query SHALL require:
- `sessions.token` matches the decoded session token (binary comparison via `escape_blob`)
- `sessions.expired_at IS NULL`
- `players.name` matches `character_name`
- `players.deletion = 0`

If the query returns no rows the server SHALL disconnect with "Account name or password is not correct."

#### Scenario: Valid session resolves character
- **WHEN** a session token written by the HTTP login handler is presented with the matching character name
- **THEN** the query returns one row containing `account_id` and `character_id`

#### Scenario: Expired or missing session rejected
- **WHEN** the session token is not in the `sessions` table or has `expired_at` set
- **THEN** the server disconnects with the credentials error message

#### Scenario: Character name mismatch rejected
- **WHEN** the session token is valid but the character name does not match any player under that account
- **THEN** the server disconnects with the credentials error message
