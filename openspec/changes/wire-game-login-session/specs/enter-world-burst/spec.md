## ADDED Requirements

### Requirement: Server loads player from DB before sending burst
After session validation the server SHALL query the `players` table by `character_id` to load: position (`posx`, `posy`, `posz`), `level`, `health`, `healthmax`, `mana`, `manamax`, `stamina`. If the query fails or returns no row the server SHALL disconnect with "Your character could not be loaded."

If `posx = posy = posz = 0` the server SHALL fall back to the temple position from config.

#### Scenario: Player loads successfully
- **WHEN** a valid `character_id` is resolved from the session query
- **THEN** the player row is fetched and its position and stats are available for the burst

#### Scenario: Missing player row causes disconnect
- **WHEN** the `character_id` does not match any row in `players`
- **THEN** the server disconnects with "Your character could not be loaded."

#### Scenario: Zero position falls back to temple position
- **WHEN** the player row has `posx = posy = posz = 0`
- **THEN** the server uses the temple position from `ConfigManager` instead

---

### Requirement: Server sends minimum enter-world burst
After loading the player the server SHALL send, in order, the following packets (unencrypted — XTEA is enabled after the burst):

1. Byte `0x0A` (pending state / enter world ack)
2. `FullMapDescription` (opcode `0x64`) — 18×14 tile viewport centred on login position
3. `PlayerStats` (opcode `0xA0`) — health, max_health, mana, max_mana, level, stamina

All three packets SHALL be written to the TCP stream before XTEA encryption is enabled for outbound messages.

#### Scenario: Client receives map on enter
- **WHEN** a player successfully logs in
- **THEN** the server writes a `0x64` map packet whose viewport origin equals `login_pos.x - 8`, `login_pos.y - 6`

#### Scenario: Client receives player stats on enter
- **WHEN** a player successfully logs in
- **THEN** the server writes a `0xA0` stats packet containing the player's level and HP values from the DB row

#### Scenario: Pending-state byte precedes map
- **WHEN** the enter-world burst is assembled
- **THEN** the first byte written is `0x0A`
