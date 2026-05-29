-- Docker runtime config for forgottenserver-rust.
-- mysqlHost points to the "db" compose service, not localhost.
worldType = "pvp"
ip = "127.0.0.1"
bindOnlyGlobalAddress = false
gameProtocolPort = 7172
statusProtocolPort = 7171
httpPort = 8080
httpWorkers = 1
httpLoginBindAddress = "0.0.0.0"
maxPlayers = 0

mysqlHost = "db"
mysqlUser = "forgottenserver"
mysqlPass = "forgottenserver"
mysqlDatabase = "forgottenserver"
mysqlPort = 3306
mysqlSock = ""

serverName = "Forgotten (Rust port)"
ownerName = ""
ownerEmail = ""
url = "https://otland.net/"
location = "Sweden"
motd = "Welcome to Forgotten Server (Rust port)."
adminPassword = "admin"

mapName = "forgotten"
mapAuthor = "Komic"
