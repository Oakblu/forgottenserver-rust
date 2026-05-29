## MODIFIED Requirements

### Requirement: Docker HTTP login port MUST be loopback-restricted
The Docker Compose configuration for the HTTP login server port SHALL bind to `127.0.0.1` only, not to `0.0.0.0`. Additionally, the HTTP login `TcpListener` in the server binary SHALL bind to `127.0.0.1` by default (configurable via `httpLoginBindAddress` in `config.lua`), ensuring loopback restriction applies to all deployment topologies, not only Docker.

#### Scenario: Port is not reachable from outside the host (Docker)
- **WHEN** the Docker Compose stack is started with default configuration
- **THEN** the HTTP login port is only reachable from the Docker host loopback interface
- **THEN** the port is not accessible from external network interfaces

#### Scenario: Listener binds to loopback in non-Docker deployment
- **WHEN** the server binary is started directly (not via Docker Compose) with no `httpLoginBindAddress` in `config.lua`
- **THEN** the HTTP login `TcpListener` is bound to `127.0.0.1:<httpPort>`
- **THEN** the port is not accessible from network interfaces other than loopback
