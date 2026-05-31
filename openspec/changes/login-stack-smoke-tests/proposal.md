## Why

Three open changes — `fix-game-login-packet-framing`, `wire-game-login-session`, and `wire-http-login-server` — are each one or two Docker/OTClient smoke-test tasks away from being archiveable. All three share the same verification environment (one `docker compose up --build` + one OTClient connection), so completing them is a single integrated session rather than three separate efforts.

## What Changes

- Bring the full login stack up via Docker Compose and verify it is healthy (MariaDB + Rust server).
- Confirm `fix-game-login-packet-framing` (task 4.4): the `failed to read packet body` error is absent from logs after an HTTP login + game connection; `Forgotten Server Online!` is still present.
- Confirm `wire-http-login-server` (task 7.7): OTClient login dialog advances past the `Failed to connect to server (HTTP)` error — character list loads or shows account-not-found, not a network error.
- Confirm `wire-game-login-session` (tasks 10.4–10.5): server logs show both HTTP and game listeners starting; OTClient completes full login, selects a character, and the game world renders (map visible, player stats shown, no ERROR 60 / disconnect).
- Fix any blocking issues discovered during the above smoke tests.
- Archive all three completed changes.

## Capabilities

### New Capabilities

- `login-stack-e2e`: End-to-end smoke-test contract for the full HTTP login → game session handshake. Covers what "working login" means observably: correct server logs, correct OTClient behaviour, absence of known error strings.

### Modified Capabilities

## Impact

- Docker Compose stack (`docker-compose.yml`, `docker/config.lua`).
- `crates/network/` — any framing or session fixes discovered during smoke testing.
- `crates/server/` — any boot or listener fixes.
- Three open OpenSpec changes closed and archived.
