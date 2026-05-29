-- C++ upstream server config for the perf comparison stack.
-- Uses tibia_cpp database so it stays isolated from dev data.
-- mysqlHost points to the "perf-db" compose service.
worldType = "pvp"
ip = "0.0.0.0"
bindOnlyGlobalAddress = false
gameProtocolPort = 7172
statusProtocolPort = 7171
httpPort = 8080
httpWorkers = 1
maxPlayers = 0

mysqlHost = "perf-db"
mysqlUser = "forgottenserver"
mysqlPass = "forgottenserver"
mysqlDatabase = "tibia_cpp"
mysqlPort = 3306
mysqlSock = ""

serverName = "Forgotten (C++ upstream)"
ownerName = ""
ownerEmail = ""
url = "https://otland.net/"
location = "Sweden"
motd = "Performance benchmark server (C++ upstream)."
adminPassword = "admin"

mapName = "forgotten"
mapAuthor = "Komic"
