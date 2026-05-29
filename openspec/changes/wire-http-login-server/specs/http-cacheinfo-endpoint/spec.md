## ADDED Requirements

### Requirement: Cacheinfo queries players_online count from DB
The system SHALL execute `SELECT COUNT(*) AS count FROM players_online` against the database and return the result as `{"playersonline": N}` with HTTP 200. A DB failure SHALL return the error envelope `{"errorCode":2,"errorMessage":"Internal error. Please try again later or contact customer support if the problem persists."}`.

#### Scenario: Successful count returns playersonline field
- **WHEN** the DB query succeeds and returns count = 5
- **THEN** the response body SHALL be `{"playersonline":5}`

#### Scenario: Zero players online is valid
- **WHEN** `players_online` table is empty
- **THEN** the response body SHALL be `{"playersonline":0}`

#### Scenario: DB query failure returns error envelope
- **WHEN** the DB query fails (connection error or timeout)
- **THEN** the response body SHALL contain `"errorCode":2` and a non-empty `"errorMessage"`

---

### Requirement: Cacheinfo response is HTTP 200 in all cases
Per C++ behaviour, both success and error responses for `cacheinfo` use HTTP status 200 (the error is expressed in the JSON body, not the HTTP status code).

#### Scenario: HTTP status is always 200
- **WHEN** cacheinfo is called regardless of DB success or failure
- **THEN** the HTTP response status line SHALL be `HTTP/1.1 200 OK`
