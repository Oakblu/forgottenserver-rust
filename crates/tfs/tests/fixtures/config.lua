-- Minimal fixture config for poketibia-server boot integration test.
-- Mirrors the C++ config.lua.dist subset required by ConfigManager.
worldType = "pvp"
ip = "127.0.0.1"
bindOnlyGlobalAddress = false
gameProtocolPort = 7172
statusProtocolPort = 7171
httpPort = 8080
httpWorkers = 1
httpLoginBindAddress = "127.0.0.1"
maxPlayers = 0

-- Database (test-only stub; no MariaDB required for these tests)
mysqlHost = "127.0.0.1"
mysqlUser = "forgottenserver"
mysqlPass = "forgottenserver"
mysqlDatabase = "forgottenserver"
mysqlPort = 3306

-- Admin / status / motd
serverName = "Poketibia (Rust port — integration test)"
ownerName = "Test"
ownerEmail = "test@example.com"
url = "http://localhost/"
location = "Test"
motd = "Welcome to Poketibia."
adminPassword = "test-admin"

-- Map / data
mapName = "forgotten"
mapAuthor = "Komic"
