## Context

The HTTP login server was transitioned from a dead stub to a live authentication endpoint in a recent PR. During that work, a debug `eprintln!` was left in the `dispatch` function that prints the full raw request body to stderr before any parsing. Because login request bodies contain plaintext passwords and TOTP tokens, every login attempt leaks credentials to stderr. In Docker Compose deployments (the project's primary runtime), all container stderr is captured and accessible via `docker logs`, log forwarders, and monitoring pipelines.

A second issue: the endpoint binds on `0.0.0.0` and the Docker Compose file publishes it as `"8080:8080"`, making it reachable from outside the host. The game's original C++ protocol uses cleartext for the login server (this is intentional for compatibility), but the Docker binding unnecessarily exposes this cleartext endpoint to the public internet.

## Goals / Non-Goals

**Goals:**
- Eliminate all logging of credential fields (password, token, email) from the HTTP dispatch path
- Restrict the Docker HTTP login port to loopback to prevent accidental internet exposure
- Preserve existing functional behavior: same request/response semantics, same error handling, same dispatch flow
- Add a safe post-parse debug log that records only the request type and source IP

**Non-Goals:**
- Adding TLS to the HTTP login server (the C++ original uses cleartext; TLS would be a protocol-breaking change out of scope for this fix)
- Changing the login protocol or request/response schema
- Rate limiting or IP allowlisting

## Decisions

### Remove the `eprintln!` rather than sanitize it

**Decision:** Delete `eprintln!("[HTTP] dispatch from {ip}: {body}")` entirely and replace it with a post-parse log that emits only `type` and `ip`.

**Rationale:** Sanitizing the raw body string by stripping JSON fields is fragile and error-prone (new fields added later could re-introduce leakage). Logging after deserialization is the correct boundary: the structured `LoginRequest` type exposes only non-sensitive fields, so there is nothing to accidentally include.

**Alternative considered:** Log only the first N bytes of the body. Rejected — a truncated log can still contain the password if it is short enough to fit within the limit.

### Restrict Docker port to loopback

**Decision:** Change `"8080:8080"` to `"127.0.0.1:8080:8080"` in `docker-compose.yml`.

**Rationale:** The HTTP login port is intended for local client connections (LAN game clients), not public internet access. Restricting to loopback is a low-risk change that eliminates the accidental internet-exposure scenario without affecting any intended use case. Operators who genuinely need external access can explicitly override this binding.

**Alternative considered:** Add a network-level firewall rule or nginx reverse proxy. Rejected — the Docker Compose binding change is simpler, closer to the source of the problem, and requires no additional infrastructure.

## Risks / Trade-offs

- **[Risk] Post-parse log is conditional on successful deserialization** → If the request body fails to parse, the new safe log won't fire. Mitigation: add a minimal error-level log on parse failure that records IP and error type (not the body).
- **[Trade-off] Loopback binding breaks external Docker network access** → Any Docker Compose override or external client expecting to reach port 8080 from outside the host will fail. This is intentional and correct; those configurations were insecure by default.

## Migration Plan

1. Edit `http_connection_session.rs`: remove the `eprintln!` at line 87; add a safe log after deserialization.
2. Edit `docker-compose.yml`: change port binding from `"8080:8080"` to `"127.0.0.1:8080:8080"`.
3. Run `cargo test --lib --workspace` to verify no regressions.
4. Run `docker compose up --build` and confirm login flow works and no credentials appear in `docker logs`.

No rollback strategy is needed — both changes are reversible one-line edits.
