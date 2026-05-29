-- bench-world-config.lua
-- Performance benchmark world configuration overrides.
-- Source this from config-cpp-perf.lua / config-rust-perf.lua via:
--   dofile("/srv/bench-world-config.lua")
--
-- Tuned for maximum stress-test throughput:
--   - Faster monster respawn (monsters stay in the world longer under load)
--   - More permissive PvP (bots can attack each other for combat benchmarks)
--   - Reduced login/save throttling
--   - NPC spawns and monster density rely on the existing forgotten-spawn.xml
--     which contains Dragon Lords, Wasps, and NPCs (The Oracle at 101,114,4)

-- Server identity
serverName   = "Forgotten Perf Bench"
motd         = "Performance benchmark — automated bots only."
maxPlayers   = 500

-- Faster rate limits for load testing
loginProtectionPeriod = 0
maxMessageBuffer      = 0

-- PvP: bots can attack each other freely in the combat scenario
worldType    = "pvp"
killsToRedSkull    = 1000
killsToBlackSkull  = 1000
protectionLevel    = 1

-- Keep saves minimal during benchmarks (reduces DB I/O noise)
savePlayerDataInterval = 300

-- Monster respawn — faster respawn keeps spawn density high under bot pressure
-- (TFS uses rateSpawn from config, not directly from this file in all versions;
--  set it here for versions that support it)
rateSpawn    = 10

-- Note: to increase spawn density in the test area, add extra spawn entries to
-- data/world/forgotten-spawn.xml and rebuild the image. The benchmark scenarios
-- that use monsters/NPCs rely on the existing spawn file which has:
--   Dragon Lords (149,574,2), (176,583,2), (186,572,3)
--   Wasps (102,67,4)
--   The Oracle NPC (101,114,4)
-- Bot characters spawn at (160,54,7) — main town — and walk/attack from there.
