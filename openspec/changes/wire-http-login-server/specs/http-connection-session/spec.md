## ADDED Requirements

### Requirement: HTTP listener binds on configured port at boot
The system SHALL bind a TCP listener on `0.0.0.0:{httpPort}` during the boot sequence. `httpPort` is read from `config.lua` via `IntegerKey::HttpPort`. The listener SHALL spawn `httpWorkers` threads (read via `IntegerKey::HttpWorkers`) to accept connections.

#### Scenario: Server starts and port is reachable
- **WHEN** the server boots with `httpPort = 8080` and `httpWorkers = 1`
- **THEN** a TCP connection to port 8080 MUST succeed within 5 seconds of startup

#### Scenario: Port is not bound when httpPort is zero
- **WHEN** `httpPort = 0` in config
- **THEN** the HTTP listener SHALL NOT be started and no error SHALL be logged

---

### Requirement: Per-connection HTTP/1.1 request reading
For each accepted TCP connection, the system SHALL read the complete HTTP request (request line, headers, and body) using `httparse`. The read MUST be bounded: maximum 8 KB for headers, maximum 4 KB for body.

#### Scenario: Well-formed POST request is fully read
- **WHEN** a client sends `POST / HTTP/1.1\r\nContent-Length: N\r\n\r\n{body}`
- **THEN** the handler receives the complete body string of length N

#### Scenario: Request exceeding header size limit is rejected
- **WHEN** a client sends headers totalling more than 8 192 bytes
- **THEN** the server SHALL close the connection without sending a response

#### Scenario: Read timeout fires on slow client
- **WHEN** a client connects but sends no data within 30 seconds
- **THEN** the server SHALL close the connection without sending a response

---

### Requirement: JSON type-field dispatch
The system SHALL parse the request body as JSON and dispatch on the `"type"` string field. Supported type values are `"login"` and `"cacheinfo"`. Any other value, or a body that is not valid JSON, or a body missing the `"type"` field, SHALL return the error envelope `{"errorCode":2,"errorMessage":"Invalid request body."}` or `{"errorCode":2,"errorMessage":"Invalid request type."}` respectively.

#### Scenario: Valid type dispatches to login handler
- **WHEN** the request body is `{"type":"login", ...}`
- **THEN** the login handler is called and its response is returned

#### Scenario: Valid type dispatches to cacheinfo handler
- **WHEN** the request body is `{"type":"cacheinfo"}`
- **THEN** the cacheinfo handler is called and its response is returned

#### Scenario: Unknown type returns error
- **WHEN** the request body is `{"type":"unknown"}`
- **THEN** the response body SHALL be `{"errorCode":2,"errorMessage":"Invalid request type."}` with HTTP status 200

#### Scenario: Non-JSON body returns error
- **WHEN** the request body is `not json at all`
- **THEN** the response body SHALL be `{"errorCode":2,"errorMessage":"Invalid request body."}` with HTTP status 200

---

### Requirement: Well-formed HTTP/1.1 response
Every response the server sends SHALL be a valid HTTP/1.1 response with:
- Status line: `HTTP/1.1 200 OK\r\n` (all login/cacheinfo responses use 200 per C++ contract)
- `Content-Type: application/json\r\n`
- `Content-Length: <N>\r\n`
- `Connection: close\r\n`
- Blank line separator `\r\n`
- Response body

#### Scenario: GET / returns 200 OK
- **WHEN** a client sends `GET / HTTP/1.0\r\n\r\n`
- **THEN** the response MUST start with `HTTP/1.0 200` or `HTTP/1.1 200`

#### Scenario: Response includes Content-Type application/json
- **WHEN** any request is processed
- **THEN** the response headers MUST include `Content-Type: application/json`

#### Scenario: Content-Length matches body length
- **WHEN** the server sends a response with body of N bytes
- **THEN** the `Content-Length` header value MUST equal N
