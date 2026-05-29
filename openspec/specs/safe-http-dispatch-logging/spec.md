# safe-http-dispatch-logging

## Requirement: HTTP dispatch MUST NOT log credential fields
The HTTP dispatch handler SHALL never write the raw request body to any output stream (stderr, stdout, tracing, or log file). Credential fields — specifically `password`, `token`, and `email` — SHALL NOT appear in any log output at any log level.

### Scenario: Login request body is not logged
- **WHEN** a client sends a POST request with a login body containing `password` and `email` fields
- **THEN** no log line contains the literal string from the `password` field
- **THEN** no log line contains the literal string from the `email` field

### Scenario: Parse failure does not log the raw body
- **WHEN** a client sends a POST request with a malformed or unparseable body
- **THEN** the error log contains only the source IP and error category, not the raw body content

## Requirement: HTTP dispatch MUST emit a safe post-parse log
After successfully deserializing an incoming request, the dispatch handler SHALL emit a debug-level log line containing the source IP address and the request `type` field only.

### Scenario: Successful dispatch logs type and IP
- **WHEN** a client sends a valid POST request of any type
- **THEN** a debug log line is emitted containing the source IP and the `type` field value
- **THEN** the log line does not contain any credential field values

## Requirement: Docker HTTP login port MUST be loopback-restricted
The Docker Compose configuration for the HTTP login server port SHALL bind to `127.0.0.1` only, not to `0.0.0.0`. Additionally, the HTTP login `TcpListener` in the server binary SHALL bind to `127.0.0.1` by default (configurable via `httpLoginBindAddress` in `config.lua`), ensuring loopback restriction applies to all deployment topologies, not only Docker.

### Scenario: Port is not reachable from outside the host (Docker)
- **WHEN** the Docker Compose stack is started with default configuration
- **THEN** the HTTP login port is only reachable from the Docker host loopback interface
- **THEN** the port is not accessible from external network interfaces

### Scenario: Listener binds to loopback in non-Docker deployment
- **WHEN** the server binary is started directly (not via Docker Compose) with no `httpLoginBindAddress` in `config.lua`
- **THEN** the HTTP login `TcpListener` is bound to `127.0.0.1:<httpPort>`
- **THEN** the port is not accessible from network interfaces other than loopback
