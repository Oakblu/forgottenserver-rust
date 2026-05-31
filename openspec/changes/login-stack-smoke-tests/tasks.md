## 1. Stack bring-up

- [ ] 1.1 Run `docker compose down` to ensure a clean slate (stop any existing containers)
- [ ] 1.2 Run `docker compose up --build` and wait for `>> Forgotten Server Online!` in server logs; confirm no `ERROR` or `panic` lines appear during boot
- [ ] 1.3 Confirm both listeners are logged: look for HTTP listener (port 8080) and game listener (port 7172) start lines in the server log

## 2. Packet-framing verification (fix-game-login-packet-framing task 4.4)

- [ ] 2.1 With the stack running, attempt a game connection from OTClient (version 1310 or 1311) pointed at `127.0.0.1:7172`
- [ ] 2.2 Inspect server logs: confirm `failed to read packet body` does NOT appear for this connection attempt
- [ ] 2.3 If the error still appears: read the relevant C++ framing spec in `forgottenserver-upstream/src/connection.cpp`, identify the mismatch in `crates/network/src/connection.rs`, write a failing test, fix it, rebuild Docker, and re-run steps 2.1–2.2
- [ ] 2.4 Mark `fix-game-login-packet-framing` task 4.4 complete in `openspec/changes/fix-game-login-packet-framing/tasks.md`

## 3. HTTP login verification (wire-http-login-server task 7.7)

- [ ] 3.1 In OTClient, open the login dialog, enter any credentials, and submit
- [ ] 3.2 Confirm the dialog does NOT show `Failed to connect to server (HTTP)` — it must advance to either a character list or an account-not-found message
- [ ] 3.3 If the HTTP error still appears: check server logs for the incoming POST, inspect `crates/server/src/http_login.rs` and `crates/network/src/protocolgame.rs` for the routing issue, fix, rebuild, re-verify
- [ ] 3.4 Mark `wire-http-login-server` task 7.7 complete in `openspec/changes/wire-http-login-server/tasks.md`

## 4. Game session verification (wire-game-login-session tasks 10.4–10.5)

- [ ] 4.1 Confirm server logs show HTTP listener and game listener both starting (satisfies task 10.4)
- [ ] 4.2 In OTClient, complete the login flow: submit credentials → character list appears → select a character
- [ ] 4.3 Confirm the game world renders: map tiles visible, player HP/mana shown, no ERROR 60 and no unexpected disconnect within 10 seconds of entry (satisfies task 10.5)
- [ ] 4.4 If ERROR 60 or disconnect occurs: read server logs for the handshake phase, inspect `crates/network/src/protocolgame.rs` login/logout flow, write a failing test, fix, rebuild, re-verify
- [ ] 4.5 Mark `wire-game-login-session` tasks 10.4 and 10.5 complete in `openspec/changes/wire-game-login-session/tasks.md`

## 5. Fix loop (if blocking issues found in steps 2–4)

- [ ] 5.1 For each blocking issue found: identify the root cause crate and function using `cpp_symbol_manifest.json` and the C++ spec
- [ ] 5.2 Write a failing unit test capturing the incorrect behavior
- [ ] 5.3 Implement the fix (no stubs; full implementation)
- [ ] 5.4 Run `cargo test --lib --workspace` — confirm zero failures
- [ ] 5.5 Run `cargo clippy --workspace --lib --tests -- -D warnings` — confirm zero warnings
- [ ] 5.6 Rebuild Docker: `docker compose up --build` and re-run the failing verification step

## 6. Archive parent changes

- [ ] 6.1 Archive `fix-game-login-packet-framing` — run `/opsx:archive fix-game-login-packet-framing`
- [ ] 6.2 Archive `wire-http-login-server` — run `/opsx:archive wire-http-login-server`
- [ ] 6.3 Archive `wire-game-login-session` — run `/opsx:archive wire-game-login-session`
- [ ] 6.4 Archive `flow-graph-1-foundation` (already complete, no remaining tasks) — run `/opsx:archive flow-graph-1-foundation`
