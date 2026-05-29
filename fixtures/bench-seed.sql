-- bench-seed.sql
-- Creates 300 bot accounts (bot1..bot300) each with one character (Bot1..Bot300).
-- Applied to tibia_cpp and tibia_rs databases by docker-compose.perf.yml.
--
-- Accounts:   name = "bot{N}",   password = SHA1("botpass")
-- Characters: name = "Bot{N}",   level 50, vocation 1 (Sorcerer)
--             position = (160, 54, 7) — main town center of the forgotten map
--
-- perf-bot connects with: --account-prefix bot --password botpass

DELIMITER //

CREATE PROCEDURE create_bench_bots(IN n INT)
BEGIN
  DECLARE i INT DEFAULT 1;
  WHILE i <= n DO
    INSERT INTO `accounts` (`name`, `password`, `type`, `premium_ends_at`, `email`, `creation`)
    VALUES (
      CONCAT('bot', i),
      SHA1('botpass'),
      1,
      2147483647,
      '',
      UNIX_TIMESTAMP()
    );

    INSERT INTO `players` (
      `name`, `group_id`, `account_id`,
      `level`, `vocation`,
      `health`, `healthmax`,
      `mana`, `manamax`, `soul`,
      `town_id`, `posx`, `posy`, `posz`,
      `cap`, `sex`, `direction`,
      `stamina`,
      `skill_fist`, `skill_club`, `skill_sword`,
      `skill_axe`, `skill_dist`, `skill_shielding`, `skill_fishing`,
      `maglevel`, `experience`
    )
    VALUES (
      CONCAT('Bot', i), 1, LAST_INSERT_ID(),
      50, 1,
      1000, 1000,
      800, 800, 100,
      1, 160, 54, 7,
      400, 0, 2,
      2520,
      65, 65, 65,
      65, 30, 65, 10,
      20, 4000000
    );

    SET i = i + 1;
  END WHILE;
END //

DELIMITER ;

CALL create_bench_bots(300);
DROP PROCEDURE create_bench_bots;
