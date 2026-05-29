## Context

The C++ `ProtocolGame::onConnect()` sends a 12-byte challenge payload and then calls `send(output)`. `send` calls `write_protocol_header` which **prepends** a 2-byte outer TCP length (`outer_len = 12`) and writes the packet to the socket — producing 14 bytes on the wire:

```
[outer_len:2 = 12][adler32:4][inner_len:2 = 6][opcode:1 = 0x1F][timestamp:4][rand:1]
```

The Rust `handle_connection` sends only the 12-byte payload (no outer_len prefix). When the otclient (or Tibia 13.x client) receives those 12 bytes, its network framing layer reads the first 2 bytes as `outer_len`, getting `adler32[0:1]` — a large garbage value. The client then stalls or disconnects without sending a valid login packet.

The downstream symptom: some client (health-check tool, misconfigured probe, or the otclient in an error state) connects to port 7172 and sends ASCII text starting with `"Forgotten (Rust "` — the server name embedded in whatever fallback data the client sends. The game server reads those bytes as `outer_len = 28486`, attempts to read 28486 bytes, and logs:

```
[game] failed to read packet body: outer_len=28486 already_have=14 err=failed to fill whole buffer
```

## Goals / Non-Goals

**Goals:**
- Fix the challenge packet to include the `[outer_len:2 = 12]` prefix, matching C++ byte-for-byte.
- Update the unit test that incorrectly asserts 12 bytes.
- Add a TDD-first integration test (in-process TCP socket pair) that covers the full challenge → login round-trip.

**Non-Goals:**
- Fixing whatever downstream client is sending ASCII text (that is a separate client-side concern).
- Adding adler32 validation of the client's incoming first packet (not done in C++ either during the first plaintext exchange).
- Handling the QT-client (CLIENTOS_QT_LINUX) extra OS-name strings — out of scope for this change.

## Decisions

### Decision 1: wire format source of truth is the C++ `send()` path, not the manual comment

The existing Rust comment ("C++ wire format: 12 raw bytes, NO outer TCP length prefix") is incorrect. The C++ TFS `Protocol::send()` / `write_protocol_header()` always prepends `outer_len` before writing to the socket. The Rust challenge must match this exactly.

**Alternative considered**: Send 12 bytes and teach the client to handle it (patch otclient). Rejected — the server must be the source of truth for the protocol; clients should not be forked.

### Decision 2: challenge buffer is 14 bytes, outer_len written at `[0..2]`

```
buf[0..2]  = 12u16.to_le_bytes()          // outer_len
buf[2..6]  = adler32 of buf[6..14]        // checksum covers inner_len..rand
buf[6..8]  = 6u16.to_le_bytes()           // inner_len
buf[8]     = 0x1F                          // opcode
buf[9..13] = timestamp.to_le_bytes()
buf[13]    = rand_byte
```

**Alternative**: Use `frame_plaintext_packet()` helper. Rejected — that helper produces a different layout (it wraps an arbitrary payload), and the challenge uses its own fixed structure. Keeping it explicit prevents layout drift.

### Decision 3: update, not delete, the existing test

The test `challenge_packet_is_12_bytes_adler32_prefixed` verifies the challenge layout. It must be updated to assert 14 bytes and verify the outer_len field. Deleting it would leave the challenge layout untested.

## Risks / Trade-offs

- [If another in-flight session already sends 14-byte challenges] → No risk; the challenge is always the first packet and no sessions are preserved across restarts.
- [Test changes a public constant] → No risk; the challenge format is internal to `handle_connection`.
- [Outer_len field off-by-one] → Mitigation: the integration test sends the challenge through a real TCP socket pair and confirms the client-side parse succeeds.
