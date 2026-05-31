## Context

The three open changes (`fix-game-login-packet-framing`, `wire-http-login-server`, `wire-game-login-session`) each implemented or fixed a discrete piece of the login pipeline:

1. **HTTP login server** (port 8080) — OTClient POSTs account credentials; server responds with JSON character list.
2. **Game login framing** — server reads the initial game-protocol packet correctly (no `failed to read packet body` panic).
3. **Game session wiring** — after character selection, the client completes the game handshake and enters the world.

All three were unit-tested to the extent possible without a live stack. The remaining tasks are integration smoke tests that require Docker Compose (MariaDB + Rust server) and a real OTClient binary.

The Docker stack exposes:
- `7171` — status protocol
- `7172` — game protocol  
- `127.0.0.1:8080` — HTTP login (OTClient 1310+ uses this instead of the legacy login protocol)

## Goals / Non-Goals

**Goals:**
- Confirm the full login pipeline works end-to-end on the running stack.
- Fix any blocking issue discovered (framing, TLS, routing, DB schema) in-place.
- Archive the three parent changes once their remaining tasks are verified.

**Non-Goals:**
- Performance or load testing.
- In-game gameplay beyond the login screen and initial world render.
- Automated regression harness (that's the E2E crate — separate change).

## Decisions

**D1 — One stack bring-up, three verifications.** All three pending tasks target the same running stack. Bringing Docker up once and verifying in sequence is cheaper than three independent bring-up/tear-down cycles. Order: packet-framing check → HTTP login → game session (each depends on the previous succeeding).

**D2 — Fix in the appropriate crate, not the Docker config.** If a smoke test reveals a framing bug, fix it in `crates/network/`; if it's a boot-ordering issue, fix it in `crates/server/`. Do not paper over bugs by tweaking `config.lua`.

**D3 — Log-based verification first, then OTClient.** Server log output is deterministic and machine-readable. Check logs before reading OTClient UI, because log evidence disambiguates server-side vs client-side failures.

**D4 — Known pre-conditions.** The `docker/mariadb-init/` scripts seed the DB schema. No manual DB setup is needed. The OTClient must be version 1310 or 1311 (the only supported range) and pointed at `127.0.0.1:7172` game port with login server `127.0.0.1:8080`.

## Risks / Trade-offs

- **DB seed incomplete** → character list is empty or world fails to load. Mitigation: check `docker logs forgottenserver-rust-db-1` for migration errors before testing OTClient.
- **Port already bound on host** → compose fails. Mitigation: `docker compose down` before `up --build`.
- **OTClient version mismatch** → version rejection in `onRecvFirstMessage`. Mitigation: verify OTClient reports version 1310 or 1311 in its window title or `--version` output.
- **Multiple issues in sequence** → difficult to attribute. Mitigation: fix and re-test one issue at a time; keep commits small.

## Migration Plan

1. `docker compose down` (clean slate).
2. `docker compose up --build` — wait for `>> Forgotten Server Online!`.
3. Verify packet-framing task (4.4): no `failed to read packet body` in logs after a connection attempt.
4. Connect OTClient → verify HTTP login task (7.7): character list appears.
5. Select character → verify game session tasks (10.4–10.5): map visible, no disconnect.
6. For each blocking issue found: fix → `docker compose up --build` → re-verify.
7. Mark remaining tasks `[x]` in each parent change's `tasks.md`.
8. Archive `fix-game-login-packet-framing`, `wire-http-login-server`, `wire-game-login-session`.

## Open Questions

- Does the seeded DB contain at least one playable character, or does testing require creating an account first via the HTTP API?
- Is OTClient available on the host at a known path, or does the developer need to install it first?
