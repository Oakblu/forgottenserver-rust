## 1. Remove Credential Logging

- [x] 1.1 Delete `eprintln!("[HTTP] dispatch from {ip}: {body}")` at `crates/server/src/http_connection_session.rs:87`
- [x] 1.2 Add a safe debug log after successful deserialization that emits only source IP and request `type` field (no credential fields)
- [x] 1.3 Add an error-level log on body parse failure that records only source IP and error category (not the raw body)

## 2. Restrict Docker Port Binding

- [x] 2.1 Change the HTTP login port in `docker-compose.yml` from `"8080:8080"` to `"127.0.0.1:8080:8080"`

## 3. Verification

- [x] 3.1 Run `cargo test --lib --workspace` and confirm all tests pass
- [x] 3.2 Run `cargo clippy --workspace --lib --tests -- -D warnings` and confirm zero warnings
- [x] 3.3 Run `docker compose up --build` and send a login request; confirm `docker logs` shows no password or token values in any log line
- [x] 3.4 Confirm the safe post-parse log line appears in docker logs with only IP and type field
