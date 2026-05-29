## Context

`start_http_listener` in `crates/server/src/boot.rs:254` calls:

```rust
TcpListener::bind(format!("0.0.0.0:{http_port}"))
```

This hard-codes the bind address to all interfaces. The `ConfigManager` in `crates/common/src/configmanager.rs` already handles `httpPort` (via `IntegerKey::HttpPort`) and `httpWorkers` (via `IntegerKey::HttpWorkers`) by mapping Lua config keys to enum variants. The same pattern can accommodate a new `httpLoginBindAddress` string key with a safe default.

The Docker Compose port mapping (`"127.0.0.1:8080:8080"`) only protects Docker deployments. Non-Docker deployments — bare-metal, VMs, or cloud instances running the binary directly — receive no protection from the compose config.

## Goals / Non-Goals

**Goals:**
- HTTP login listener bind address defaults to `127.0.0.1` in all deployment topologies.
- Operators can override via `httpLoginBindAddress` in `config.lua`.
- The change is minimal and contained: one new `StringKey` variant, one new Lua mapping, one changed `format!` call.
- Existing tests continue to pass without modification.

**Non-Goals:**
- Changing the admin, status, or game port bind addresses (separate concern, out of scope).
- Implementing IP allowlisting or firewall rules at the application layer.
- Changing the Docker Compose configuration (already correct).

## Decisions

### Decision 1: Add `StringKey::HttpBindAddress` rather than a separate `IpAddr` config type

**Chosen:** Add `StringKey::HttpBindAddress` to the existing `StringKey` enum, mapped from `"httpLoginBindAddress"` in the Lua key table. The string is passed directly to `TcpListener::bind(format!("{bind}:{port}"))`.

**Alternative considered:** Parse the value as `std::net::IpAddr` at config load time and store it typed. Rejected: overkill for a single string that `TcpListener::bind` already parses at bind time; a bad value produces a clear bind error at startup, not a silent failure.

**Default:** The `ConfigManager` returns an empty string for unknown `StringKey` variants. We intercept this in `start_http_listener`: if the configured value is empty, substitute `"127.0.0.1"`. This preserves the existing config-absent behavior while requiring no changes to `ConfigManager::defaults()`.

### Decision 2: Fallback logic lives in `boot.rs`, not in `ConfigManager`

**Chosen:** Apply the default inside `start_http_listener` with:
```rust
let bind_addr = {
    let s = config.get_string(StringKey::HttpBindAddress);
    if s.is_empty() { "127.0.0.1" } else { s }
};
```

**Alternative considered:** Add a default value for `HttpBindAddress` inside `ConfigManager`. Rejected: `ConfigManager` defaults are populated from C++ `configmanager.cpp` defaults; adding a new default there would require a ledger entry and a migration justification. The local fallback is simpler and self-documenting.

### Decision 3: Update fixture config files to document the new key

Add `httpLoginBindAddress = "127.0.0.1"` to:
- `docker/config.lua` — dev Docker fixture
- `crates/tfs/tests/fixtures/config.lua` — CLI smoke-test fixture

This makes the key discoverable for operators who look at example configs, and keeps the fixtures in sync with production behavior.

## Risks / Trade-offs

- **Operator surprise**: An operator running on bare metal who previously relied on `0.0.0.0` will find the login endpoint no longer reachable from the network after upgrading. They must explicitly add `httpLoginBindAddress = "0.0.0.0"` to their `config.lua`. This is intentional — the insecure behavior now requires an explicit opt-in — but it is a behavior change for existing non-Docker deployments.
  → Mitigation: Document the new key prominently in the server startup log (print the effective bind address alongside the existing port announcement).

- **Invalid address string**: A misconfigured value (e.g., `httpLoginBindAddress = "not-an-ip"`) will cause `TcpListener::bind` to fail at startup with a clear error message.
  → Mitigation: The existing bind-error path in `start_http_listener` already returns an `Err(String)` that propagates to the boot sequence and terminates the process with a descriptive message — no additional handling needed.

## Open Questions

None. The scope is fully bounded by the proposal and the existing `ConfigManager` extension pattern.
