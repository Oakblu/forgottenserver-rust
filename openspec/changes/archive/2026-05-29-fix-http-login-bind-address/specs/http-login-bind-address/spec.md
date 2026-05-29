## ADDED Requirements

### Requirement: HTTP login listener bind address MUST default to loopback
The HTTP login listener SHALL bind to `127.0.0.1` when no `httpLoginBindAddress` key is present in `config.lua`, ensuring non-Docker deployments are not unintentionally exposed.

#### Scenario: Listener binds to loopback when config key is absent
- **WHEN** `config.lua` does not contain an `httpLoginBindAddress` entry
- **THEN** `start_http_listener` binds the `TcpListener` to `127.0.0.1:<httpPort>`
- **THEN** the startup log line includes the effective bind address (`127.0.0.1`)

#### Scenario: Listener binds to loopback when config key is empty string
- **WHEN** `config.lua` sets `httpLoginBindAddress = ""`
- **THEN** `start_http_listener` treats the empty value as absent and binds to `127.0.0.1:<httpPort>`

### Requirement: HTTP login listener bind address MUST be overridable via config
The `httpLoginBindAddress` key in `config.lua` SHALL override the default bind address, allowing operators who require network access to opt in explicitly.

#### Scenario: Operator configures 0.0.0.0 explicitly
- **WHEN** `config.lua` sets `httpLoginBindAddress = "0.0.0.0"`
- **THEN** `start_http_listener` binds the `TcpListener` to `0.0.0.0:<httpPort>`

#### Scenario: Operator configures a specific interface address
- **WHEN** `config.lua` sets `httpLoginBindAddress = "192.168.1.10"`
- **THEN** `start_http_listener` binds the `TcpListener` to `192.168.1.10:<httpPort>`

### Requirement: Effective bind address MUST be logged at startup
The server SHALL print the effective HTTP login bind address (after applying defaults) to the startup output so operators can verify the configured behavior.

#### Scenario: Startup log shows effective bind address
- **WHEN** the HTTP login listener starts successfully
- **THEN** the startup log line contains both the bind address and the port (e.g., `127.0.0.1:8080`)
