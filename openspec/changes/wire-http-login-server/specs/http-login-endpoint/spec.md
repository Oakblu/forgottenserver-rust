## ADDED Requirements

### Requirement: Login requires email and password fields
The system SHALL reject login requests that are missing the `"email"` or `"password"` JSON fields with error code 3: `{"errorCode":3,"errorMessage":"Tibia account email address or Tibia password is not correct."}`.

#### Scenario: Missing email field returns error 3
- **WHEN** the request body is `{"type":"login","password":"x"}`
- **THEN** the response SHALL be `{"errorCode":3,"errorMessage":"Tibia account email address or Tibia password is not correct."}`

#### Scenario: Missing password field returns error 3
- **WHEN** the request body is `{"type":"login","email":"x@x.com"}`
- **THEN** the response SHALL be `{"errorCode":3,"errorMessage":"Tibia account email address or Tibia password is not correct."}`

---

### Requirement: Login validates credentials against the database
The system SHALL query the `accounts` table for a row matching the provided `email`. It SHALL compare the SHA-1 hash of the provided `password` against the stored `password` column (stored as a hex-encoded SHA-1 digest). A mismatch or missing account SHALL return error code 3.

#### Scenario: Unknown email returns error 3
- **WHEN** `email` does not match any row in `accounts`
- **THEN** the response SHALL contain `"errorCode":3`

#### Scenario: Wrong password returns error 3
- **WHEN** `email` matches an account but SHA-1(`password`) != stored hash
- **THEN** the response SHALL contain `"errorCode":3`

#### Scenario: Correct credentials return success
- **WHEN** `email` and `password` match a DB account
- **THEN** the response SHALL contain `"session"` and `"playdata"` top-level keys

---

### Requirement: TOTP two-factor authentication
When the matched account has a non-empty `secret` column, the system SHALL require a `"token"` field in the request body. The token SHALL be validated against the current, previous, and next 30-second TOTP windows (matching C++ `AUTHENTICATOR_PERIOD = 30`). A missing, empty, or out-of-window token SHALL return error code 6: `{"errorCode":6,"errorMessage":"Two-factor token required for authentication."}`.

#### Scenario: 2FA account missing token returns error 6
- **WHEN** the account has a non-empty `secret` and `"token"` is absent from the request
- **THEN** the response SHALL contain `"errorCode":6`

#### Scenario: 2FA account with wrong token returns error 6
- **WHEN** the account has a non-empty `secret` and the provided `"token"` does not match any of the ±1 TOTP windows
- **THEN** the response SHALL contain `"errorCode":6`

#### Scenario: 2FA account with correct token returns success
- **WHEN** the account has a non-empty `secret` and the provided `"token"` matches the current TOTP window
- **THEN** the response SHALL contain `"session"` and `"playdata"` top-level keys

---

### Requirement: Session token is persisted on successful login
On successful authentication, the system SHALL generate 16 cryptographically random bytes as a session token, Base64-encode them for the response, and INSERT the raw bytes into the `sessions` table (`token`, `account_id`, `ip` columns). A database error during INSERT SHALL return the generic internal error envelope.

#### Scenario: Session INSERT failure returns internal error
- **WHEN** the DB `execute()` for the session INSERT fails
- **THEN** the response SHALL contain `"errorCode":2` or similar internal error code

#### Scenario: Successful login session key is base64
- **WHEN** login succeeds
- **THEN** `response.session.sessionkey` SHALL be a non-empty base64-encoded string

---

### Requirement: Login response shape matches C++ exactly
On success the system SHALL return a JSON body with the following structure (field names and nesting must be exact):

```json
{
  "session": {
    "sessionkey": "<base64>",
    "lastlogintime": <uint>,
    "ispremium": <bool>,
    "premiumuntil": <int>,
    "status": "active",
    "returnernotification": false,
    "showrewardnews": true,
    "isreturner": true,
    "recoverysetupcomplete": true,
    "fpstracking": false,
    "optiontracking": false
  },
  "playdata": {
    "worlds": [
      {
        "id": 0,
        "name": "<serverName config>",
        "externaladdressprotected": "<ip config>",
        "externalportprotected": <gamePort config>,
        "externaladdressunprotected": "<ip config>",
        "externalportunprotected": <gamePort config>,
        "previewstate": 0,
        "location": "<location config>",
        "anticheatprotection": false,
        "pvptype": <0|1|2>
      }
    ],
    "characters": [
      {
        "worldid": 0,
        "name": "<name>",
        "level": <uint>,
        "vocation": "<vocation name>",
        "lastlogin": <uint>,
        "ismale": <bool>,
        "ishidden": false,
        "ismaincharacter": false,
        "tutorial": false,
        "outfitid": <uint>,
        "headcolor": <uint>,
        "torsocolor": <uint>,
        "legscolor": <uint>,
        "detailcolor": <uint>,
        "addonsflags": <uint>,
        "dailyrewardstate": 0
      }
    ]
  }
}
```

#### Scenario: Response contains session key
- **WHEN** login succeeds
- **THEN** `response.session.sessionkey` MUST be present and non-empty

#### Scenario: Response contains worlds array with one entry
- **WHEN** login succeeds
- **THEN** `response.playdata.worlds` MUST be an array with exactly one element containing `"id": 0`

#### Scenario: Response contains characters array
- **WHEN** login succeeds and the account has N characters
- **THEN** `response.playdata.characters` MUST be an array of N objects each containing `"name"` and `"level"`

#### Scenario: ispremium is true when premiumEndsAt is in the future
- **WHEN** login succeeds and `accounts.premium_ends_at` > current unix timestamp
- **THEN** `response.session.ispremium` SHALL be `true`

#### Scenario: ispremium is false when premiumEndsAt is in the past or zero
- **WHEN** login succeeds and `accounts.premium_ends_at` <= current unix timestamp
- **THEN** `response.session.ispremium` SHALL be `false`
