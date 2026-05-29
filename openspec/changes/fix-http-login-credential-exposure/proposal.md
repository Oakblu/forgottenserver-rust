## Why

The newly activated HTTP login server endpoint logs the full raw request body — including plaintext passwords and TOTP tokens — to stderr on every login request, and binds on all network interfaces without TLS, exposing credentials to anyone with log-reader access or network visibility. Both issues were introduced in the same PR that transitioned the endpoint from a dead stub to a live authentication path.

## What Changes

- Remove (or sanitize) the `eprintln!("[HTTP] dispatch from {ip}: {body}")` call in `http_connection_session.rs` that logs raw request bodies containing plaintext passwords before parsing
- Restrict the Docker Compose port binding for the HTTP login server from `"8080:8080"` (all interfaces) to `"127.0.0.1:8080:8080"` (loopback only) to prevent accidental internet exposure of the cleartext credential endpoint
- Add a post-parse debug log that emits only the safe `type` field (no credential fields)

## Capabilities

### New Capabilities

- `safe-http-dispatch-logging`: Logging of HTTP dispatch events that never includes credential fields; emits only connection metadata (IP, request type) after deserialization

### Modified Capabilities

- none

## Impact

- `crates/server/src/http_connection_session.rs` — remove credential-leaking `eprintln!`, add safe post-parse log
- `docker-compose.yml` — restrict HTTP login port to loopback binding
- No API or wire-format changes; no test-fixture changes required
