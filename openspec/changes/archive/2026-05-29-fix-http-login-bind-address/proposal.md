## Why

The HTTP login listener in `crates/server/src/boot.rs` binds to `0.0.0.0`, exposing the credential endpoint on every network interface. The existing `safe-http-dispatch-logging` spec already requires the Docker Compose port mapping to be loopback-restricted, but the code-level bind address was never updated to match — so any non-Docker deployment (bare-metal, VM, cloud instance) silently exposes the live auth endpoint to the network.

## What Changes

- Change `TcpListener::bind(format!("0.0.0.0:{http_port}"))` in `boot.rs` to bind to a configurable address defaulting to `127.0.0.1`.
- Add `httpLoginBindAddress` to `config.lua` (default: `"127.0.0.1"`) so operators who intentionally need network exposure can opt in explicitly.
- Read the bind address from the server config in `boot.rs` alongside the existing `httpLoginPort` read.
- Add a test that verifies the default bind address is `127.0.0.1` when config is absent.

## Capabilities

### New Capabilities

- `http-login-bind-address`: Configurable bind address for the HTTP login listener, defaulting to `127.0.0.1` to prevent unintended network exposure.

### Modified Capabilities

- `safe-http-dispatch-logging`: Extend the existing spec to require that the HTTP login listener bind address in code (not just Docker Compose) defaults to `127.0.0.1` and is configurable via `config.lua`.

## Impact

- `crates/server/src/boot.rs` — bind address change at the `TcpListener::bind` call site.
- `crates/server/src/config.rs` (or equivalent config reader) — new `httpLoginBindAddress` key.
- `docker/config.lua` (dev fixture) — add `httpLoginBindAddress = "127.0.0.1"` to document the setting.
- `crates/tfs/tests/fixtures/config.lua` — add the key so the existing CLI smoke-test path continues to work.
- `openspec/specs/safe-http-dispatch-logging/spec.md` — new requirement and scenarios covering the code-level bind address.
