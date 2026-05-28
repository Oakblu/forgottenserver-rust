-- phpMyAdmin SQL Dump
-- version 5.0.4
-- https://www.phpmyadmin.net/
--
-- Host: 127.0.0.1
-- Generation Time: Dec 22, 2025 at 02:28 AM
-- Server version: 10.4.17-MariaDB
-- PHP Version: 7.4.13

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `otserv`
--

-- --------------------------------------------------------

--
-- Table structure for table `accounts`
--

CREATE TABLE `accounts` (
  `id` int(11) UNSIGNED NOT NULL,
  `name` varchar(32) DEFAULT NULL,
  `password` text NOT NULL,
  `email` varchar(255) NOT NULL DEFAULT '',
  `created` int(11) NOT NULL DEFAULT 0,
  `rlname` varchar(255) NOT NULL DEFAULT '',
  `location` varchar(255) NOT NULL DEFAULT '',
  `country` varchar(3) NOT NULL DEFAULT '',
  `web_lastlogin` int(11) NOT NULL DEFAULT 0,
  `web_flags` int(11) NOT NULL DEFAULT 0,
  `email_hash` varchar(32) NOT NULL DEFAULT '',
  `email_new` varchar(255) NOT NULL DEFAULT '',
  `email_new_time` int(11) NOT NULL DEFAULT 0,
  `email_code` varchar(255) NOT NULL DEFAULT '',
  `email_next` int(11) NOT NULL DEFAULT 0,
  `email_verified` tinyint(1) NOT NULL DEFAULT 0,
  `phone` varchar(15) DEFAULT NULL,
  `key` varchar(64) NOT NULL DEFAULT '',
  `premdays` int(11) NOT NULL DEFAULT 0,
  `premdays_purchased` int(11) NOT NULL DEFAULT 0,
  `lastday` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `type` tinyint(1) UNSIGNED NOT NULL DEFAULT 1,
  `coins` int(12) UNSIGNED NOT NULL DEFAULT 0,
  `coins_transferable` int(12) UNSIGNED NOT NULL DEFAULT 0,
  `tournament_coins` int(12) UNSIGNED NOT NULL DEFAULT 0,
  `creation` int(11) UNSIGNED NOT NULL DEFAULT 0,
  `recruiter` int(6) DEFAULT 0,
  `house_bid_id` int(11) NOT NULL DEFAULT 0,
  `vote` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `accounts`
--

INSERT INTO `accounts` (`id`, `name`, `password`, `email`, `created`, `rlname`, `location`, `country`, `web_lastlogin`, `web_flags`, `email_hash`, `email_new`, `email_new_time`, `email_code`, `email_next`, `email_verified`, `phone`, `key`, `premdays`, `premdays_purchased`, `lastday`, `type`, `coins`, `coins_transferable`, `tournament_coins`, `creation`, `recruiter`, `house_bid_id`, `vote`) VALUES
(1, 'god', '21298df8a3277357ee55b01df9530b535cf08ec1', '@god', 0, '', '', '', 0, 3, '', '', 0, '', 0, 0, NULL, '', 359, 360, 1797468388, 6, 0, 996099, 0, 1766344573, 0, 0, 0),
(2, 'sdfsdf', 'dfdsfdf', 'admin@gmail.com', 1766362992, '', '', 'us', 0, 0, '', '', 0, '', 0, 0, NULL, '', 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0);

--
-- Triggers `accounts`
--
DELIMITER $$
CREATE TRIGGER `oncreate_accounts` AFTER INSERT ON `accounts` FOR EACH ROW BEGIN
    INSERT INTO `account_vipgroups` (`account_id`, `name`, `customizable`) VALUES (NEW.`id`, 'Enemies', 0);
    INSERT INTO `account_vipgroups` (`account_id`, `name`, `customizable`) VALUES (NEW.`id`, 'Friends', 0);
    INSERT INTO `account_vipgroups` (`account_id`, `name`, `customizable`) VALUES (NEW.`id`, 'Trading Partner', 0);
END
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Table structure for table `account_bans`
--

CREATE TABLE `account_bans` (
  `account_id` int(11) UNSIGNED NOT NULL,
  `reason` varchar(255) NOT NULL,
  `banned_at` bigint(20) NOT NULL,
  `expires_at` bigint(20) NOT NULL,
  `banned_by` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `account_ban_history`
--

CREATE TABLE `account_ban_history` (
  `id` int(11) NOT NULL,
  `account_id` int(11) UNSIGNED NOT NULL,
  `reason` varchar(255) NOT NULL,
  `banned_at` bigint(20) NOT NULL,
  `expired_at` bigint(20) NOT NULL,
  `banned_by` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `account_sessions`
--

CREATE TABLE `account_sessions` (
  `id` varchar(191) NOT NULL,
  `account_id` int(10) UNSIGNED NOT NULL,
  `expires` bigint(20) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `account_vipgrouplist`
--

CREATE TABLE `account_vipgrouplist` (
  `account_id` int(11) UNSIGNED NOT NULL COMMENT 'id of account whose viplist entry it is',
  `player_id` int(11) NOT NULL COMMENT 'id of target player of viplist entry',
  `vipgroup_id` int(11) UNSIGNED NOT NULL COMMENT 'id of vip group that player belongs'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `account_vipgroups`
--

CREATE TABLE `account_vipgroups` (
  `id` int(11) UNSIGNED NOT NULL,
  `account_id` int(11) UNSIGNED NOT NULL COMMENT 'id of account whose vip group entry it is',
  `name` varchar(128) NOT NULL,
  `customizable` tinyint(1) NOT NULL DEFAULT 1
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `account_vipgroups`
--

INSERT INTO `account_vipgroups` (`id`, `account_id`, `name`, `customizable`) VALUES
(1, 1, 'Enemies', 0),
(2, 1, 'Friends', 0),
(3, 1, 'Trading Partner', 0),
(4, 2, 'Enemies', 0),
(5, 2, 'Friends', 0),
(6, 2, 'Trading Partner', 0);

-- --------------------------------------------------------

--
-- Table structure for table `account_viplist`
--

CREATE TABLE `account_viplist` (
  `account_id` int(11) UNSIGNED NOT NULL COMMENT 'id of account whose viplist entry it is',
  `player_id` int(11) NOT NULL COMMENT 'id of target player of viplist entry',
  `description` varchar(128) NOT NULL DEFAULT '',
  `icon` tinyint(2) UNSIGNED NOT NULL DEFAULT 0,
  `notify` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `boosted_boss`
--

CREATE TABLE `boosted_boss` (
  `boostname` text DEFAULT NULL,
  `date` varchar(250) NOT NULL DEFAULT '',
  `raceid` varchar(250) NOT NULL DEFAULT '',
  `looktypeEx` int(11) NOT NULL DEFAULT 0,
  `looktype` int(11) NOT NULL DEFAULT 136,
  `lookfeet` int(11) NOT NULL DEFAULT 0,
  `looklegs` int(11) NOT NULL DEFAULT 0,
  `lookhead` int(11) NOT NULL DEFAULT 0,
  `lookbody` int(11) NOT NULL DEFAULT 0,
  `lookaddons` int(11) NOT NULL DEFAULT 0,
  `lookmount` int(11) DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `boosted_boss`
--

INSERT INTO `boosted_boss` (`boostname`, `date`, `raceid`, `looktypeEx`, `looktype`, `lookfeet`, `looklegs`, `lookhead`, `lookbody`, `lookaddons`, `lookmount`) VALUES
('Tarbaz', '21', '1188', 0, 842, 3, 19, 0, 21, 2, 0);

-- --------------------------------------------------------

--
-- Table structure for table `boosted_creature`
--

CREATE TABLE `boosted_creature` (
  `boostname` text DEFAULT NULL,
  `date` varchar(250) NOT NULL DEFAULT '',
  `raceid` varchar(250) NOT NULL DEFAULT '',
  `looktype` int(11) NOT NULL DEFAULT 136,
  `lookfeet` int(11) NOT NULL DEFAULT 0,
  `looklegs` int(11) NOT NULL DEFAULT 0,
  `lookhead` int(11) NOT NULL DEFAULT 0,
  `lookbody` int(11) NOT NULL DEFAULT 0,
  `lookaddons` int(11) NOT NULL DEFAULT 0,
  `lookmount` int(11) DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `boosted_creature`
--

INSERT INTO `boosted_creature` (`boostname`, `date`, `raceid`, `looktype`, `lookfeet`, `looklegs`, `lookhead`, `lookbody`, `lookaddons`, `lookmount`) VALUES
('Dark Magician', '21', '371', 133, 131, 51, 58, 95, 2, 0);

-- --------------------------------------------------------

--
-- Table structure for table `coins_transactions`
--

CREATE TABLE `coins_transactions` (
  `id` int(11) UNSIGNED NOT NULL,
  `account_id` int(11) UNSIGNED NOT NULL,
  `type` tinyint(1) UNSIGNED NOT NULL,
  `coin_type` tinyint(1) UNSIGNED NOT NULL DEFAULT 1,
  `amount` int(12) UNSIGNED NOT NULL,
  `description` varchar(3500) NOT NULL,
  `timestamp` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `coins_transactions`
--

INSERT INTO `coins_transactions` (`id`, `account_id`, `type`, `coin_type`, `amount`, `description`, `timestamp`) VALUES
(1, 1, 2, 3, 3000, 'REMOVE Coins', '2025-12-22 00:46:28'),
(2, 1, 2, 3, 900, 'REMOVE Coins', '2025-12-22 00:48:37');

-- --------------------------------------------------------

--
-- Table structure for table `daily_reward_history`
--

CREATE TABLE `daily_reward_history` (
  `id` int(11) NOT NULL,
  `daystreak` smallint(2) NOT NULL DEFAULT 0,
  `player_id` int(11) NOT NULL,
  `timestamp` int(11) NOT NULL,
  `description` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `forge_history`
--

CREATE TABLE `forge_history` (
  `id` int(11) NOT NULL,
  `player_id` int(11) NOT NULL,
  `action_type` int(11) NOT NULL DEFAULT 0,
  `description` text NOT NULL,
  `is_success` tinyint(4) NOT NULL DEFAULT 0,
  `bonus` tinyint(4) NOT NULL DEFAULT 0,
  `done_at` bigint(20) NOT NULL,
  `done_at_date` datetime DEFAULT current_timestamp(),
  `cost` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `gained` bigint(20) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `global_storage`
--

CREATE TABLE `global_storage` (
  `key` varchar(32) NOT NULL,
  `value` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `global_storage`
--

INSERT INTO `global_storage` (`key`, `value`) VALUES
('14110', '1766364045'),
('40000', '4');

-- --------------------------------------------------------

--
-- Table structure for table `guilds`
--

CREATE TABLE `guilds` (
  `id` int(11) NOT NULL,
  `level` int(11) NOT NULL DEFAULT 1,
  `name` varchar(255) NOT NULL,
  `ownerid` int(11) NOT NULL,
  `creationdata` int(11) NOT NULL,
  `motd` varchar(255) NOT NULL DEFAULT '',
  `residence` int(11) NOT NULL DEFAULT 0,
  `balance` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `points` int(11) NOT NULL DEFAULT 0,
  `description` text NOT NULL,
  `logo_name` varchar(255) NOT NULL DEFAULT 'default.gif'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Triggers `guilds`
--
DELIMITER $$
CREATE TRIGGER `oncreate_guilds` AFTER INSERT ON `guilds` FOR EACH ROW BEGIN
    INSERT INTO `guild_ranks` (`name`, `level`, `guild_id`) VALUES ('The Leader', 3, NEW.`id`);
    INSERT INTO `guild_ranks` (`name`, `level`, `guild_id`) VALUES ('Vice-Leader', 2, NEW.`id`);
    INSERT INTO `guild_ranks` (`name`, `level`, `guild_id`) VALUES ('Member', 1, NEW.`id`);
END
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Table structure for table `guildwar_kills`
--

CREATE TABLE `guildwar_kills` (
  `id` int(11) NOT NULL,
  `killer` varchar(50) NOT NULL,
  `target` varchar(50) NOT NULL,
  `killerguild` int(11) NOT NULL DEFAULT 0,
  `targetguild` int(11) NOT NULL DEFAULT 0,
  `warid` int(11) NOT NULL DEFAULT 0,
  `time` bigint(15) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `guild_invites`
--

CREATE TABLE `guild_invites` (
  `player_id` int(11) NOT NULL DEFAULT 0,
  `guild_id` int(11) NOT NULL DEFAULT 0,
  `date` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `guild_membership`
--

CREATE TABLE `guild_membership` (
  `player_id` int(11) NOT NULL,
  `guild_id` int(11) NOT NULL,
  `rank_id` int(11) NOT NULL,
  `nick` varchar(15) NOT NULL DEFAULT ''
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `guild_ranks`
--

CREATE TABLE `guild_ranks` (
  `id` int(11) NOT NULL,
  `guild_id` int(11) NOT NULL COMMENT 'guild',
  `name` varchar(255) NOT NULL COMMENT 'rank name',
  `level` int(11) NOT NULL COMMENT 'rank level - leader, vice, member, maybe something else'
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `guild_wars`
--

CREATE TABLE `guild_wars` (
  `id` int(11) NOT NULL,
  `guild1` int(11) NOT NULL DEFAULT 0,
  `guild2` int(11) NOT NULL DEFAULT 0,
  `name1` varchar(255) NOT NULL,
  `name2` varchar(255) NOT NULL,
  `status` tinyint(2) UNSIGNED NOT NULL DEFAULT 0,
  `started` bigint(15) NOT NULL DEFAULT 0,
  `ended` bigint(15) NOT NULL DEFAULT 0,
  `frags_limit` smallint(4) UNSIGNED NOT NULL DEFAULT 0,
  `payment` bigint(13) UNSIGNED NOT NULL DEFAULT 0,
  `duration_days` tinyint(3) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `houses`
--

CREATE TABLE `houses` (
  `id` int(11) NOT NULL,
  `owner` int(11) NOT NULL,
  `new_owner` int(11) NOT NULL DEFAULT -1,
  `paid` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `warnings` int(11) NOT NULL DEFAULT 0,
  `name` varchar(255) NOT NULL,
  `rent` int(11) NOT NULL DEFAULT 0,
  `town_id` int(11) NOT NULL DEFAULT 0,
  `size` int(11) NOT NULL DEFAULT 0,
  `guildid` int(11) DEFAULT NULL,
  `beds` int(11) NOT NULL DEFAULT 0,
  `bidder` int(11) NOT NULL DEFAULT 0,
  `bidder_name` varchar(255) NOT NULL DEFAULT '',
  `highest_bid` int(11) NOT NULL DEFAULT 0,
  `internal_bid` int(11) NOT NULL DEFAULT 0,
  `bid_end_date` int(11) NOT NULL DEFAULT 0,
  `state` smallint(5) UNSIGNED NOT NULL DEFAULT 0,
  `transfer_status` tinyint(1) DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `houses`
--

INSERT INTO `houses` (`id`, `owner`, `new_owner`, `paid`, `warnings`, `name`, `rent`, `town_id`, `size`, `guildid`, `beds`, `bidder`, `bidder_name`, `highest_bid`, `internal_bid`, `bid_end_date`, `state`, `transfer_status`) VALUES
(2628, 0, -1, 0, 0, 'Castle of the Winds', 500000, 5, 514, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2629, 0, -1, 0, 0, 'Ab\'Dendriel Clanhall', 250000, 5, 326, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2630, 0, -1, 0, 0, 'Underwood 9', 50000, 5, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2631, 0, -1, 0, 0, 'Treetop 13', 100000, 5, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2632, 0, -1, 0, 0, 'Underwood 8', 50000, 5, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2633, 0, -1, 0, 0, 'Treetop 11', 50000, 5, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2635, 0, -1, 0, 0, 'Great Willow 2a', 50000, 5, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2637, 0, -1, 0, 0, 'Great Willow 2b', 50000, 5, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2638, 0, -1, 0, 0, 'Great Willow Western Wing', 100000, 5, 61, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2640, 0, -1, 0, 0, 'Great Willow 1', 100000, 5, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2642, 0, -1, 0, 0, 'Great Willow 3a', 50000, 5, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2644, 0, -1, 0, 0, 'Great Willow 3b', 80000, 5, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2645, 0, -1, 0, 0, 'Great Willow 4a', 25000, 5, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2648, 0, -1, 0, 0, 'Great Willow 4b', 25000, 5, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2649, 0, -1, 0, 0, 'Underwood 6', 100000, 5, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2650, 0, -1, 0, 0, 'Underwood 3', 100000, 5, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2651, 0, -1, 0, 0, 'Underwood 5', 80000, 5, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2652, 0, -1, 0, 0, 'Underwood 2', 100000, 5, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2653, 0, -1, 0, 0, 'Underwood 1', 100000, 5, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2654, 0, -1, 0, 0, 'Prima Arbor', 400000, 5, 200, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2655, 0, -1, 0, 0, 'Underwood 7', 200000, 5, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2656, 0, -1, 0, 0, 'Underwood 10', 25000, 5, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2657, 0, -1, 0, 0, 'Underwood 4', 100000, 5, 50, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2658, 0, -1, 0, 0, 'Treetop 9', 50000, 5, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2659, 0, -1, 0, 0, 'Treetop 10', 80000, 5, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2660, 0, -1, 0, 0, 'Treetop 8', 25000, 5, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2661, 0, -1, 0, 0, 'Treetop 7', 50000, 5, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2662, 0, -1, 0, 0, 'Treetop 6', 25000, 5, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2663, 0, -1, 0, 0, 'Treetop 5 (Shop)', 80000, 5, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2664, 0, -1, 0, 0, 'Treetop 12 (Shop)', 100000, 5, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2665, 0, -1, 0, 0, 'Treetop 4 (Shop)', 80000, 5, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2666, 0, -1, 0, 0, 'Treetop 3 (Shop)', 80000, 5, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2687, 0, -1, 0, 0, 'Northern Street 1a', 100000, 6, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2688, 0, -1, 0, 0, 'Park Lane 3a', 100000, 6, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2689, 0, -1, 0, 0, 'Park Lane 1a', 150000, 6, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2690, 0, -1, 0, 0, 'Park Lane 4', 150000, 6, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2691, 0, -1, 0, 0, 'Park Lane 2', 150000, 6, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2692, 0, -1, 0, 0, 'Theater Avenue 7, Flat 04', 50000, 6, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2693, 0, -1, 0, 0, 'Theater Avenue 7, Flat 03', 25000, 6, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2694, 0, -1, 0, 0, 'Theater Avenue 7, Flat 05', 50000, 6, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2695, 0, -1, 0, 0, 'Theater Avenue 7, Flat 06', 25000, 6, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2696, 0, -1, 0, 0, 'Theater Avenue 7, Flat 02', 25000, 6, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2697, 0, -1, 0, 0, 'Theater Avenue 7, Flat 01', 25000, 6, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2698, 0, -1, 0, 0, 'Northern Street 5', 200000, 6, 52, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2699, 0, -1, 0, 0, 'Northern Street 7', 150000, 6, 44, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2700, 0, -1, 0, 0, 'Theater Avenue 6e', 80000, 6, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2701, 0, -1, 0, 0, 'Theater Avenue 6c', 25000, 6, 9, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2702, 0, -1, 0, 0, 'Theater Avenue 6a', 80000, 6, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2703, 0, -1, 0, 0, 'Theater Avenue, Tower', 300000, 6, 80, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2705, 0, -1, 0, 0, 'East Lane 2', 300000, 6, 93, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2706, 0, -1, 0, 0, 'Harbour Lane 2a (Shop)', 80000, 6, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2707, 0, -1, 0, 0, 'Harbour Lane 2b (Shop)', 80000, 6, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2708, 0, -1, 0, 0, 'Harbour Lane 3', 400000, 6, 92, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2709, 0, -1, 0, 0, 'Magician\'s Alley 8', 150000, 6, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2710, 0, -1, 0, 0, 'Lonely Sea Side Hostel', 400000, 6, 331, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2711, 0, -1, 0, 0, 'Suntower', 500000, 6, 306, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2712, 0, -1, 0, 0, 'House of Recreation', 500000, 6, 469, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2713, 0, -1, 0, 0, 'Carlin Clanhall', 250000, 6, 287, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2714, 0, -1, 0, 0, 'Magician\'s Alley 4', 200000, 6, 60, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2715, 0, -1, 0, 0, 'Theater Avenue 14 (Shop)', 200000, 6, 54, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2716, 0, -1, 0, 0, 'Theater Avenue 12', 80000, 6, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2717, 0, -1, 0, 0, 'Magician\'s Alley 1', 100000, 6, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2718, 0, -1, 0, 0, 'Theater Avenue 10', 100000, 6, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2719, 0, -1, 0, 0, 'Magician\'s Alley 1b', 25000, 6, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2720, 0, -1, 0, 0, 'Magician\'s Alley 1a', 25000, 6, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2721, 0, -1, 0, 0, 'Magician\'s Alley 1c', 25000, 6, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2722, 0, -1, 0, 0, 'Magician\'s Alley 1d', 25000, 6, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2723, 0, -1, 0, 0, 'Magician\'s Alley 5c', 100000, 6, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2724, 0, -1, 0, 0, 'Magician\'s Alley 5f', 80000, 6, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2725, 0, -1, 0, 0, 'Magician\'s Alley 5b', 50000, 6, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2727, 0, -1, 0, 0, 'Magician\'s Alley 5a', 50000, 6, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2729, 0, -1, 0, 0, 'Central Plaza 3 (Shop)', 50000, 6, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2730, 0, -1, 0, 0, 'Central Plaza 2 (Shop)', 25000, 6, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2731, 0, -1, 0, 0, 'Central Plaza 1 (Shop)', 50000, 6, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2732, 0, -1, 0, 0, 'Theater Avenue 8b', 100000, 6, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2733, 0, -1, 0, 0, 'Harbour Lane 1 (Shop)', 100000, 6, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2734, 0, -1, 0, 0, 'Theater Avenue 6f', 80000, 6, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2735, 0, -1, 0, 0, 'Theater Avenue 6d', 25000, 6, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2736, 0, -1, 0, 0, 'Theater Avenue 6b', 50000, 6, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2737, 0, -1, 0, 0, 'Northern Street 3a', 80000, 6, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2738, 0, -1, 0, 0, 'Northern Street 3b', 80000, 6, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2739, 0, -1, 0, 0, 'Northern Street 1b', 80000, 6, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2740, 0, -1, 0, 0, 'Northern Street 1c', 80000, 6, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2741, 0, -1, 0, 0, 'Theater Avenue 7, Flat 14', 25000, 6, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2742, 0, -1, 0, 0, 'Theater Avenue 7, Flat 13', 25000, 6, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2743, 0, -1, 0, 0, 'Theater Avenue 7, Flat 15', 25000, 6, 12, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2744, 0, -1, 0, 0, 'Theater Avenue 7, Flat 12', 25000, 6, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2745, 0, -1, 0, 0, 'Theater Avenue 7, Flat 11', 50000, 6, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2746, 0, -1, 0, 0, 'Theater Avenue 7, Flat 16', 25000, 6, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2747, 0, -1, 0, 0, 'Theater Avenue 5', 200000, 6, 113, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2751, 0, -1, 0, 0, 'Harbour Flats, Flat 11', 25000, 6, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2752, 0, -1, 0, 0, 'Harbour Flats, Flat 13', 25000, 6, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2753, 0, -1, 0, 0, 'Harbour Flats, Flat 15', 50000, 6, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2755, 0, -1, 0, 0, 'Harbour Flats, Flat 12', 50000, 6, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2757, 0, -1, 0, 0, 'Harbour Flats, Flat 16', 50000, 6, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2759, 0, -1, 0, 0, 'Harbour Flats, Flat 21', 50000, 6, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2760, 0, -1, 0, 0, 'Harbour Flats, Flat 22', 80000, 6, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2761, 0, -1, 0, 0, 'Harbour Flats, Flat 23', 25000, 6, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2763, 0, -1, 0, 0, 'Park Lane 1b', 200000, 6, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2764, 0, -1, 0, 0, 'Theater Avenue 8a', 100000, 6, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2765, 0, -1, 0, 0, 'Theater Avenue 11a', 100000, 6, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2767, 0, -1, 0, 0, 'Theater Avenue 11b', 100000, 6, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2768, 0, -1, 0, 0, 'Caretaker\'s Residence', 600000, 6, 298, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2769, 0, -1, 0, 0, 'Moonkeep', 250000, 6, 298, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2770, 0, -1, 0, 0, 'Mangrove 1', 80000, 5, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2771, 0, -1, 0, 0, 'Coastwood 2', 50000, 5, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2772, 0, -1, 0, 0, 'Coastwood 1', 50000, 5, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2773, 0, -1, 0, 0, 'Coastwood 3', 50000, 5, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2774, 0, -1, 0, 0, 'Coastwood 4', 50000, 5, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2775, 0, -1, 0, 0, 'Mangrove 4', 50000, 5, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2776, 0, -1, 0, 0, 'Coastwood 10', 80000, 5, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2777, 0, -1, 0, 0, 'Coastwood 5', 50000, 5, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2778, 0, -1, 0, 0, 'Coastwood 6 (Shop)', 80000, 5, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2779, 0, -1, 0, 0, 'Coastwood 7', 25000, 5, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2780, 0, -1, 0, 0, 'Coastwood 8', 50000, 5, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2781, 0, -1, 0, 0, 'Coastwood 9', 50000, 5, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2782, 0, -1, 0, 0, 'Treetop 2', 25000, 5, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2783, 0, -1, 0, 0, 'Treetop 1', 25000, 5, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2784, 0, -1, 0, 0, 'Mangrove 3', 80000, 5, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2785, 0, -1, 0, 0, 'Mangrove 2', 50000, 5, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2786, 0, -1, 0, 0, 'The Hideout', 250000, 5, 449, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2787, 0, -1, 0, 0, 'Shadow Towers', 250000, 5, 429, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2788, 0, -1, 0, 0, 'Druids Retreat A', 50000, 6, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2789, 0, -1, 0, 0, 'Druids Retreat C', 50000, 6, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2790, 0, -1, 0, 0, 'Druids Retreat B', 50000, 6, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2791, 0, -1, 0, 0, 'Druids Retreat D', 80000, 6, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2792, 0, -1, 0, 0, 'East Lane 1b', 150000, 6, 43, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2793, 0, -1, 0, 0, 'East Lane 1a', 200000, 6, 62, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2794, 0, -1, 0, 0, 'Senja Village 11', 80000, 6, 59, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2795, 0, -1, 0, 0, 'Senja Village 10', 50000, 6, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2796, 0, -1, 0, 0, 'Senja Village 9', 80000, 6, 55, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2797, 0, -1, 0, 0, 'Senja Village 8', 50000, 6, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2798, 0, -1, 0, 0, 'Senja Village 7', 25000, 6, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2799, 0, -1, 0, 0, 'Senja Village 6b', 25000, 6, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2800, 0, -1, 0, 0, 'Senja Village 6a', 50000, 6, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2801, 0, -1, 0, 0, 'Senja Village 5', 50000, 6, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2802, 0, -1, 0, 0, 'Senja Village 4', 50000, 6, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2803, 0, -1, 0, 0, 'Senja Village 3', 50000, 6, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2804, 0, -1, 0, 0, 'Senja Village 1b', 50000, 6, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2805, 0, -1, 0, 0, 'Senja Village 1a', 25000, 6, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2806, 0, -1, 0, 0, 'Rosebud C', 100000, 6, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2807, 0, -1, 0, 0, 'Rosebud B', 80000, 6, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2808, 0, -1, 0, 0, 'Rosebud A', 50000, 6, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2809, 0, -1, 0, 0, 'Park Lane 3b', 100000, 6, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2810, 0, -1, 0, 0, 'Northport Village 6', 80000, 6, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2811, 0, -1, 0, 0, 'Northport Village 5', 80000, 6, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2812, 0, -1, 0, 0, 'Northport Village 4', 100000, 6, 50, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2813, 0, -1, 0, 0, 'Northport Village 3', 150000, 6, 104, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2814, 0, -1, 0, 0, 'Northport Village 2', 50000, 6, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2815, 0, -1, 0, 0, 'Northport Village 1', 50000, 6, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2816, 0, -1, 0, 0, 'Nautic Observer', 200000, 6, 220, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2817, 0, -1, 0, 0, 'Nordic Stronghold', 250000, 6, 536, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2818, 0, -1, 0, 0, 'Senja Clanhall', 250000, 6, 228, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2819, 0, -1, 0, 0, 'Seawatch', 250000, 6, 434, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2820, 0, -1, 0, 0, 'Dwarven Magnate\'s Estate', 300000, 7, 269, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2821, 0, -1, 0, 0, 'Forge Master\'s Quarters', 300000, 7, 79, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2822, 0, -1, 0, 0, 'Upper Barracks 13', 25000, 7, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2823, 0, -1, 0, 0, 'Upper Barracks 5', 80000, 7, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2824, 0, -1, 0, 0, 'Upper Barracks 3', 80000, 7, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2825, 0, -1, 0, 0, 'Upper Barracks 4', 50000, 7, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2826, 0, -1, 0, 0, 'Upper Barracks 2', 80000, 7, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2827, 0, -1, 0, 0, 'Upper Barracks 1', 50000, 7, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2828, 0, -1, 0, 0, 'Tunnel Gardens 9', 150000, 7, 74, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2829, 0, -1, 0, 0, 'Tunnel Gardens 8', 25000, 7, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2830, 0, -1, 0, 0, 'Tunnel Gardens 7', 50000, 7, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2831, 0, -1, 0, 0, 'Tunnel Gardens 6', 25000, 7, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2832, 0, -1, 0, 0, 'Tunnel Gardens 5', 25000, 7, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2835, 0, -1, 0, 0, 'Tunnel Gardens 4', 80000, 7, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2836, 0, -1, 0, 0, 'Tunnel Gardens 2', 80000, 7, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2837, 0, -1, 0, 0, 'Tunnel Gardens 1', 80000, 7, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2838, 0, -1, 0, 0, 'Tunnel Gardens 3', 80000, 7, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2839, 0, -1, 0, 0, 'The Market 4 (Shop)', 80000, 7, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2840, 0, -1, 0, 0, 'The Market 3 (Shop)', 80000, 7, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2841, 0, -1, 0, 0, 'The Market 2 (Shop)', 50000, 7, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2842, 0, -1, 0, 0, 'The Market 1 (Shop)', 25000, 7, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2843, 0, -1, 0, 0, 'The Farms 6, Fishing Hut', 50000, 7, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2844, 0, -1, 0, 0, 'The Farms 5', 50000, 7, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2845, 0, -1, 0, 0, 'The Farms 4', 25000, 7, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2846, 0, -1, 0, 0, 'The Farms 3', 80000, 7, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2847, 0, -1, 0, 0, 'The Farms 2', 50000, 7, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2849, 0, -1, 0, 0, 'The Farms 1', 80000, 7, 57, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2850, 0, -1, 0, 0, 'Outlaw Camp 14 (Shop)', 25000, 7, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2852, 0, -1, 0, 0, 'Outlaw Camp 13 (Shop)', 50000, 7, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2853, 0, -1, 0, 0, 'Outlaw Camp 9', 80000, 7, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2854, 0, -1, 0, 0, 'Outlaw Camp 7', 25000, 7, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2855, 0, -1, 0, 0, 'Outlaw Camp 4', 50000, 7, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2856, 0, -1, 0, 0, 'Outlaw Camp 2', 50000, 7, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2857, 0, -1, 0, 0, 'Outlaw Camp 3', 50000, 7, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2858, 0, -1, 0, 0, 'Outlaw Camp 1', 80000, 7, 47, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2859, 0, -1, 0, 0, 'Nobility Quarter 5', 100000, 7, 141, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2860, 0, -1, 0, 0, 'Nobility Quarter 4', 50000, 7, 65, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2861, 0, -1, 0, 0, 'Nobility Quarter 3', 80000, 7, 51, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2862, 0, -1, 0, 0, 'Nobility Quarter 2', 50000, 7, 58, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2863, 0, -1, 0, 0, 'Nobility Quarter 1', 80000, 7, 63, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2864, 0, -1, 0, 0, 'Lower Barracks 10', 80000, 7, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2865, 0, -1, 0, 0, 'Lower Barracks 9', 80000, 7, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2866, 0, -1, 0, 0, 'Lower Barracks 8', 80000, 7, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2867, 0, -1, 0, 0, 'Lower Barracks 1', 80000, 7, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2868, 0, -1, 0, 0, 'Lower Barracks 2', 80000, 7, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2869, 0, -1, 0, 0, 'Lower Barracks 3', 80000, 7, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2870, 0, -1, 0, 0, 'Lower Barracks 4', 50000, 7, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2871, 0, -1, 0, 0, 'Lower Barracks 5', 100000, 7, 63, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2872, 0, -1, 0, 0, 'Lower Barracks 6', 100000, 7, 63, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2873, 0, -1, 0, 0, 'Lower Barracks 7', 80000, 7, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2874, 0, -1, 0, 0, 'Wolftower', 500000, 7, 402, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2875, 0, -1, 0, 0, 'Riverspring', 250000, 7, 371, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2876, 0, -1, 0, 0, 'Outlaw Castle', 250000, 7, 302, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2877, 0, -1, 0, 0, 'Marble Guildhall', 250000, 7, 410, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2878, 0, -1, 0, 0, 'Iron Guildhall', 250000, 7, 379, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2879, 0, -1, 0, 0, 'Hill Hideout', 250000, 7, 247, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2880, 0, -1, 0, 0, 'Granite Guildhall', 250000, 7, 506, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2881, 0, -1, 0, 0, 'Alai Flats, Flat 01', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2882, 0, -1, 0, 0, 'Alai Flats, Flat 02', 50000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2883, 0, -1, 0, 0, 'Alai Flats, Flat 03', 50000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2884, 0, -1, 0, 0, 'Alai Flats, Flat 04', 80000, 8, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2885, 0, -1, 0, 0, 'Alai Flats, Flat 05', 100000, 8, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2886, 0, -1, 0, 0, 'Alai Flats, Flat 06', 100000, 8, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2887, 0, -1, 0, 0, 'Alai Flats, Flat 07', 25000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2888, 0, -1, 0, 0, 'Alai Flats, Flat 08', 50000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2889, 0, -1, 0, 0, 'Alai Flats, Flat 11', 80000, 8, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2890, 0, -1, 0, 0, 'Alai Flats, Flat 12', 25000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2891, 0, -1, 0, 0, 'Alai Flats, Flat 13', 50000, 8, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2892, 0, -1, 0, 0, 'Alai Flats, Flat 14', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2893, 0, -1, 0, 0, 'Alai Flats, Flat 15', 100000, 8, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2894, 0, -1, 0, 0, 'Alai Flats, Flat 16', 100000, 8, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2895, 0, -1, 0, 0, 'Alai Flats, Flat 17', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2896, 0, -1, 0, 0, 'Alai Flats, Flat 18', 50000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2897, 0, -1, 0, 0, 'Alai Flats, Flat 21', 50000, 8, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2898, 0, -1, 0, 0, 'Alai Flats, Flat 22', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2899, 0, -1, 0, 0, 'Alai Flats, Flat 23', 25000, 8, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2900, 0, -1, 0, 0, 'Alai Flats, Flat 28', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2901, 0, -1, 0, 0, 'Alai Flats, Flat 27', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2902, 0, -1, 0, 0, 'Alai Flats, Flat 26', 100000, 8, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2903, 0, -1, 0, 0, 'Alai Flats, Flat 25', 100000, 8, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2904, 0, -1, 0, 0, 'Alai Flats, Flat 24', 80000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2905, 0, -1, 0, 0, 'Beach Home Apartments, Flat 01', 50000, 8, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2906, 0, -1, 0, 0, 'Beach Home Apartments, Flat 02', 80000, 8, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2907, 0, -1, 0, 0, 'Beach Home Apartments, Flat 03', 80000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2908, 0, -1, 0, 0, 'Beach Home Apartments, Flat 04', 50000, 8, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2909, 0, -1, 0, 0, 'Beach Home Apartments, Flat 05', 80000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2910, 0, -1, 0, 0, 'Beach Home Apartments, Flat 06', 100000, 8, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2911, 0, -1, 0, 0, 'Beach Home Apartments, Flat 11', 25000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2912, 0, -1, 0, 0, 'Beach Home Apartments, Flat 12', 50000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2913, 0, -1, 0, 0, 'Beach Home Apartments, Flat 13', 80000, 8, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2914, 0, -1, 0, 0, 'Beach Home Apartments, Flat 14', 25000, 8, 8, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2915, 0, -1, 0, 0, 'Beach Home Apartments, Flat 15', 25000, 8, 9, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2916, 0, -1, 0, 0, 'Beach Home Apartments, Flat 16', 80000, 8, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2917, 0, -1, 0, 0, 'Demon Tower', 100000, 8, 75, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2918, 0, -1, 0, 0, 'Farm Lane, 1st floor (Shop)', 80000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2919, 0, -1, 0, 0, 'Farm Lane, 2nd Floor (Shop)', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2920, 0, -1, 0, 0, 'Farm Lane, Basement (Shop)', 50000, 8, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2921, 0, -1, 0, 0, 'Fibula Village 1', 25000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2922, 0, -1, 0, 0, 'Fibula Village 2', 25000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2923, 0, -1, 0, 0, 'Fibula Village 4', 25000, 8, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2924, 0, -1, 0, 0, 'Fibula Village 5', 50000, 8, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2925, 0, -1, 0, 0, 'Fibula Village 3', 80000, 8, 60, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2926, 0, -1, 0, 0, 'Fibula Village, Tower Flat', 100000, 8, 94, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2927, 0, -1, 0, 0, 'Guildhall of the Red Rose', 250000, 8, 396, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2928, 0, -1, 0, 0, 'Fibula Village, Bar (Shop)', 100000, 8, 74, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2929, 0, -1, 0, 0, 'Fibula Village, Villa', 200000, 8, 264, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2930, 0, -1, 0, 0, 'Greenshore Village 1', 80000, 8, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2931, 0, -1, 0, 0, 'Greenshore Clanhall', 250000, 8, 176, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2932, 0, -1, 0, 0, 'Castle of Greenshore', 250000, 8, 325, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2933, 0, -1, 0, 0, 'Greenshore Village, Shop', 80000, 8, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2934, 0, -1, 0, 0, 'Greenshore Village, Villa', 300000, 8, 178, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2935, 0, -1, 0, 0, 'Greenshore Village 7', 25000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2936, 0, -1, 0, 0, 'Greenshore Village 3', 50000, 8, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2939, 0, -1, 0, 0, 'Greenshore Village 2', 50000, 8, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2940, 0, -1, 0, 0, 'Greenshore Village 6', 150000, 8, 79, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2941, 0, -1, 0, 0, 'Harbour Place 1 (Shop)', 800000, 8, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2942, 0, -1, 0, 0, 'Harbour Place 2 (Shop)', 600000, 8, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2943, 0, -1, 0, 0, 'Harbour Place 3', 800000, 8, 88, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2944, 0, -1, 0, 0, 'Harbour Place 4', 80000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2945, 0, -1, 0, 0, 'Lower Swamp Lane 1', 400000, 8, 80, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2946, 0, -1, 0, 0, 'Lower Swamp Lane 3', 400000, 8, 80, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2947, 0, -1, 0, 0, 'Main Street 9, 1st floor (Shop)', 200000, 8, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2948, 0, -1, 0, 0, 'Main Street 9a, 2nd floor (Shop)', 100000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2949, 0, -1, 0, 0, 'Main Street 9b, 2nd floor (Shop)', 150000, 8, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2950, 0, -1, 0, 0, 'Mill Avenue 1 (Shop)', 200000, 8, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2951, 0, -1, 0, 0, 'Mill Avenue 2 (Shop)', 200000, 8, 47, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2952, 0, -1, 0, 0, 'Mill Avenue 3', 100000, 8, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2953, 0, -1, 0, 0, 'Mill Avenue 4', 100000, 8, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2954, 0, -1, 0, 0, 'Mill Avenue 5', 300000, 8, 69, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2955, 0, -1, 0, 0, 'Open-Air Theatre', 150000, 8, 81, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2956, 0, -1, 0, 0, 'Smuggler\'s Den', 400000, 8, 226, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2957, 0, -1, 0, 0, 'Sorcerer\'s Avenue 1a', 100000, 8, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2958, 0, -1, 0, 0, 'Sorcerer\'s Avenue 5 (Shop)', 150000, 8, 54, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2959, 0, -1, 0, 0, 'Sorcerer\'s Avenue 1b', 80000, 8, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2960, 0, -1, 0, 0, 'Sorcerer\'s Avenue 1c', 100000, 8, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2961, 0, -1, 0, 0, 'Sorcerer\'s Avenue Labs 2a', 50000, 8, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2962, 0, -1, 0, 0, 'Sorcerer\'s Avenue Labs 2c', 50000, 8, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2963, 0, -1, 0, 0, 'Sorcerer\'s Avenue Labs 2b', 50000, 8, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2964, 0, -1, 0, 0, 'Sunset Homes, Flat 01', 100000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2965, 0, -1, 0, 0, 'Sunset Homes, Flat 02', 80000, 8, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2966, 0, -1, 0, 0, 'Sunset Homes, Flat 03', 80000, 8, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2967, 0, -1, 0, 0, 'Sunset Homes, Flat 11', 80000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2968, 0, -1, 0, 0, 'Sunset Homes, Flat 12', 50000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2969, 0, -1, 0, 0, 'Sunset Homes, Flat 13', 100000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2970, 0, -1, 0, 0, 'Sunset Homes, Flat 14', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2971, 0, -1, 0, 0, 'Sunset Homes, Flat 21', 50000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2972, 0, -1, 0, 0, 'Sunset Homes, Flat 22', 50000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2973, 0, -1, 0, 0, 'Sunset Homes, Flat 23', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2974, 0, -1, 0, 0, 'Sunset Homes, Flat 24', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2975, 0, -1, 0, 0, 'Thais Hostel', 200000, 8, 129, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2976, 0, -1, 0, 0, 'The City Wall 1a', 150000, 8, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2977, 0, -1, 0, 0, 'The City Wall 1b', 100000, 8, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2978, 0, -1, 0, 0, 'The City Wall 3a', 100000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2979, 0, -1, 0, 0, 'The City Wall 3b', 100000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2980, 0, -1, 0, 0, 'The City Wall 3c', 100000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2981, 0, -1, 0, 0, 'The City Wall 3d', 100000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2982, 0, -1, 0, 0, 'The City Wall 3e', 100000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2983, 0, -1, 0, 0, 'The City Wall 3f', 100000, 8, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2984, 0, -1, 0, 0, 'Upper Swamp Lane 12', 300000, 8, 76, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2985, 0, -1, 0, 0, 'Upper Swamp Lane 10', 150000, 8, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2986, 0, -1, 0, 0, 'Upper Swamp Lane 8', 600000, 8, 159, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2987, 0, -1, 0, 0, 'Upper Swamp Lane 4', 600000, 8, 100, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2988, 0, -1, 0, 0, 'Upper Swamp Lane 2', 600000, 8, 100, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2989, 0, -1, 0, 0, 'The City Wall 9', 80000, 8, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2990, 0, -1, 0, 0, 'The City Wall 7h', 50000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2991, 0, -1, 0, 0, 'The City Wall 7b', 25000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2992, 0, -1, 0, 0, 'The City Wall 7d', 50000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2993, 0, -1, 0, 0, 'The City Wall 7f', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2994, 0, -1, 0, 0, 'The City Wall 7c', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2995, 0, -1, 0, 0, 'The City Wall 7a', 80000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2996, 0, -1, 0, 0, 'The City Wall 7g', 50000, 8, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2997, 0, -1, 0, 0, 'The City Wall 7e', 80000, 8, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2998, 0, -1, 0, 0, 'The City Wall 5b', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(2999, 0, -1, 0, 0, 'The City Wall 5d', 50000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3000, 0, -1, 0, 0, 'The City Wall 5f', 25000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3001, 0, -1, 0, 0, 'The City Wall 5a', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3002, 0, -1, 0, 0, 'The City Wall 5c', 50000, 8, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3003, 0, -1, 0, 0, 'The City Wall 5e', 50000, 8, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3004, 0, -1, 0, 0, 'Warriors\' Guildhall', 5000000, 8, 334, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3005, 0, -1, 0, 0, 'The Tibianic', 500000, 8, 809, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3006, 0, -1, 0, 0, 'Bloodhall', 500000, 8, 321, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3007, 0, -1, 0, 0, 'Fibula Clanhall', 250000, 8, 178, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3008, 0, -1, 0, 0, 'Dark Mansion', 1000000, 8, 375, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3009, 0, -1, 0, 0, 'Halls of the Adventurers', 250000, 8, 317, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3010, 0, -1, 0, 0, 'Mercenary Tower', 250000, 8, 619, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3011, 0, -1, 0, 0, 'Snake Tower', 500000, 8, 627, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3012, 0, -1, 0, 0, 'Southern Thais Guildhall', 1000000, 8, 374, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3013, 0, -1, 0, 0, 'Spiritkeep', 500000, 8, 289, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3014, 0, -1, 0, 0, 'Thais Clanhall', 500000, 8, 206, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3015, 0, -1, 0, 0, 'The Lair', 200000, 9, 259, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3016, 0, -1, 0, 0, 'Silver Street 4', 300000, 9, 119, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3017, 0, -1, 0, 0, 'Dream Street 1 (Shop)', 600000, 9, 149, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3018, 0, -1, 0, 0, 'Dagger Alley 1', 200000, 9, 103, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3019, 0, -1, 0, 0, 'Dream Street 2', 400000, 9, 113, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3020, 0, -1, 0, 0, 'Dream Street 3', 300000, 9, 104, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3021, 0, -1, 0, 0, 'Elm Street 1', 300000, 9, 99, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3022, 0, -1, 0, 0, 'Elm Street 3', 300000, 9, 107, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3023, 0, -1, 0, 0, 'Elm Street 2', 300000, 9, 98, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3024, 0, -1, 0, 0, 'Elm Street 4', 300000, 9, 108, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3025, 0, -1, 0, 0, 'Seagull Walk 1', 800000, 9, 169, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3026, 0, -1, 0, 0, 'Seagull Walk 2', 300000, 9, 102, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3027, 0, -1, 0, 0, 'Dream Street 4', 400000, 9, 128, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3028, 0, -1, 0, 0, 'Old Lighthouse', 200000, 9, 157, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3029, 0, -1, 0, 0, 'Market Street 1', 600000, 9, 220, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3030, 0, -1, 0, 0, 'Market Street 3', 600000, 9, 127, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3031, 0, -1, 0, 0, 'Market Street 4 (Shop)', 800000, 9, 176, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3032, 0, -1, 0, 0, 'Market Street 5 (Shop)', 800000, 9, 230, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3033, 0, -1, 0, 0, 'Market Street 2', 600000, 9, 173, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3034, 0, -1, 0, 0, 'Loot Lane 1 (Shop)', 600000, 9, 159, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3035, 0, -1, 0, 0, 'Mystic Lane 1', 300000, 9, 92, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3036, 0, -1, 0, 0, 'Mystic Lane 2', 200000, 9, 119, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3037, 0, -1, 0, 0, 'Lucky Lane 2 (Tower)', 600000, 9, 216, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3038, 0, -1, 0, 0, 'Lucky Lane 3 (Tower)', 600000, 9, 216, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3039, 0, -1, 0, 0, 'Iron Alley 1', 300000, 9, 101, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3040, 0, -1, 0, 0, 'Iron Alley 2', 300000, 9, 128, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3041, 0, -1, 0, 0, 'Swamp Watch', 500000, 9, 379, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3042, 0, -1, 0, 0, 'Golden Axe Guildhall', 500000, 9, 344, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3043, 0, -1, 0, 0, 'Silver Street 1', 200000, 9, 108, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3044, 0, -1, 0, 0, 'Valorous Venore', 500000, 9, 457, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3045, 0, -1, 0, 0, 'Salvation Street 2', 300000, 9, 113, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3046, 0, -1, 0, 0, 'Salvation Street 3', 300000, 9, 143, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3047, 0, -1, 0, 0, 'Silver Street 2', 200000, 9, 76, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3048, 0, -1, 0, 0, 'Silver Street 3', 200000, 9, 82, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3049, 0, -1, 0, 0, 'Mystic Lane 3 (Tower)', 800000, 9, 214, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3050, 0, -1, 0, 0, 'Market Street 7', 200000, 9, 90, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3051, 0, -1, 0, 0, 'Market Street 6', 600000, 9, 186, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3052, 0, -1, 0, 0, 'Iron Alley Watch, Upper', 600000, 9, 215, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3053, 0, -1, 0, 0, 'Iron Alley Watch, Lower', 600000, 9, 217, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3054, 0, -1, 0, 0, 'Blessed Shield Guildhall', 500000, 9, 250, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3055, 0, -1, 0, 0, 'Steel Home', 500000, 9, 388, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3056, 0, -1, 0, 0, 'Salvation Street 1 (Shop)', 600000, 9, 215, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3057, 0, -1, 0, 0, 'Lucky Lane 1 (Shop)', 800000, 9, 220, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3058, 0, -1, 0, 0, 'Paupers Palace, Flat 34', 100000, 9, 59, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3059, 0, -1, 0, 0, 'Paupers Palace, Flat 33', 50000, 9, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3060, 0, -1, 0, 0, 'Paupers Palace, Flat 32', 100000, 9, 50, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3061, 0, -1, 0, 0, 'Paupers Palace, Flat 31', 80000, 9, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3062, 0, -1, 0, 0, 'Paupers Palace, Flat 28', 25000, 9, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3063, 0, -1, 0, 0, 'Paupers Palace, Flat 26', 25000, 9, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3064, 0, -1, 0, 0, 'Paupers Palace, Flat 24', 25000, 9, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3065, 0, -1, 0, 0, 'Paupers Palace, Flat 22', 25000, 9, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3066, 0, -1, 0, 0, 'Paupers Palace, Flat 21', 25000, 9, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3067, 0, -1, 0, 0, 'Paupers Palace, Flat 27', 50000, 9, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3068, 0, -1, 0, 0, 'Paupers Palace, Flat 25', 50000, 9, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3069, 0, -1, 0, 0, 'Paupers Palace, Flat 23', 50000, 9, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3070, 0, -1, 0, 0, 'Paupers Palace, Flat 11', 25000, 9, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3071, 0, -1, 0, 0, 'Paupers Palace, Flat 13', 50000, 9, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3072, 0, -1, 0, 0, 'Paupers Palace, Flat 15', 50000, 9, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3073, 0, -1, 0, 0, 'Paupers Palace, Flat 17', 25000, 9, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3074, 0, -1, 0, 0, 'Paupers Palace, Flat 18', 25000, 9, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3075, 0, -1, 0, 0, 'Paupers Palace, Flat 12', 50000, 9, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3076, 0, -1, 0, 0, 'Paupers Palace, Flat 14', 50000, 9, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3077, 0, -1, 0, 0, 'Paupers Palace, Flat 16', 50000, 9, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3078, 0, -1, 0, 0, 'Paupers Palace, Flat 06', 25000, 9, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3079, 0, -1, 0, 0, 'Paupers Palace, Flat 05', 25000, 9, 9, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3080, 0, -1, 0, 0, 'Paupers Palace, Flat 04', 25000, 9, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3081, 0, -1, 0, 0, 'Paupers Palace, Flat 07', 50000, 9, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3082, 0, -1, 0, 0, 'Paupers Palace, Flat 03', 25000, 9, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3083, 0, -1, 0, 0, 'Paupers Palace, Flat 02', 25000, 9, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3084, 0, -1, 0, 0, 'Paupers Palace, Flat 01', 25000, 9, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3085, 0, -1, 0, 0, 'Castle, Residence', 600000, 11, 104, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3086, 0, -1, 0, 0, 'Castle, 3rd Floor, Flat 07', 80000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3087, 0, -1, 0, 0, 'Castle, 3rd Floor, Flat 04', 25000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3088, 0, -1, 0, 0, 'Castle, 3rd Floor, Flat 03', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3089, 0, -1, 0, 0, 'Castle, 3rd Floor, Flat 06', 100000, 11, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3090, 0, -1, 0, 0, 'Castle, 3rd Floor, Flat 05', 80000, 11, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3091, 0, -1, 0, 0, 'Castle, 3rd Floor, Flat 02', 80000, 11, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3092, 0, -1, 0, 0, 'Castle, 3rd Floor, Flat 01', 50000, 11, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3093, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 09', 50000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3094, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 08', 80000, 11, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3095, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 07', 80000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3096, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 04', 50000, 11, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3097, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 03', 50000, 11, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3098, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 06', 100000, 11, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3099, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 05', 80000, 11, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3100, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 02', 80000, 11, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3101, 0, -1, 0, 0, 'Castle, 4th Floor, Flat 01', 50000, 11, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3102, 0, -1, 0, 0, 'Castle Street 2', 150000, 11, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3103, 0, -1, 0, 0, 'Castle Street 3', 150000, 11, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3104, 0, -1, 0, 0, 'Castle Street 4', 150000, 11, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3105, 0, -1, 0, 0, 'Castle Street 5', 150000, 11, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3106, 0, -1, 0, 0, 'Castle Street 1', 300000, 11, 71, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3107, 0, -1, 0, 0, 'Edron Flats, Flat 08', 25000, 11, 10, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3108, 0, -1, 0, 0, 'Edron Flats, Flat 05', 25000, 11, 10, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3109, 0, -1, 0, 0, 'Edron Flats, Flat 04', 25000, 11, 10, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3110, 0, -1, 0, 0, 'Edron Flats, Flat 01', 50000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3111, 0, -1, 0, 0, 'Edron Flats, Flat 07', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3112, 0, -1, 0, 0, 'Edron Flats, Flat 06', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3113, 0, -1, 0, 0, 'Edron Flats, Flat 03', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3114, 0, -1, 0, 0, 'Edron Flats, Flat 02', 100000, 11, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3115, 0, -1, 0, 0, 'Edron Flats, Basement Flat 2', 100000, 11, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3116, 0, -1, 0, 0, 'Edron Flats, Basement Flat 1', 100000, 11, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3119, 0, -1, 0, 0, 'Edron Flats, Flat 13', 80000, 11, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3121, 0, -1, 0, 0, 'Edron Flats, Flat 14', 100000, 11, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3123, 0, -1, 0, 0, 'Edron Flats, Flat 12', 80000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3124, 0, -1, 0, 0, 'Edron Flats, Flat 11', 100000, 11, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3125, 0, -1, 0, 0, 'Edron Flats, Flat 25', 80000, 11, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3127, 0, -1, 0, 0, 'Edron Flats, Flat 24', 80000, 11, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3128, 0, -1, 0, 0, 'Edron Flats, Flat 21', 80000, 11, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3131, 0, -1, 0, 0, 'Edron Flats, Flat 23', 80000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3133, 0, -1, 0, 0, 'Castle Shop 1', 400000, 11, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3134, 0, -1, 0, 0, 'Castle Shop 2', 400000, 11, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3135, 0, -1, 0, 0, 'Castle Shop 3', 300000, 11, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3136, 0, -1, 0, 0, 'Central Circle 1', 800000, 11, 76, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3137, 0, -1, 0, 0, 'Central Circle 2', 800000, 11, 90, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3138, 0, -1, 0, 0, 'Central Circle 3', 800000, 11, 99, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3139, 0, -1, 0, 0, 'Central Circle 4', 800000, 11, 97, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3140, 0, -1, 0, 0, 'Central Circle 5', 800000, 11, 99, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3141, 0, -1, 0, 0, 'Central Circle 8 (Shop)', 400000, 11, 101, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3142, 0, -1, 0, 0, 'Central Circle 7 (Shop)', 400000, 11, 101, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3143, 0, -1, 0, 0, 'Central Circle 6 (Shop)', 400000, 11, 101, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3144, 0, -1, 0, 0, 'Central Circle 9a', 150000, 11, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3145, 0, -1, 0, 0, 'Central Circle 9b', 150000, 11, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3146, 0, -1, 0, 0, 'Sky Lane, Guild 1', 1000000, 11, 459, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3147, 0, -1, 0, 0, 'Sky Lane, Sea Tower', 300000, 11, 106, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3148, 0, -1, 0, 0, 'Sky Lane, Guild 3', 1000000, 11, 391, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3149, 0, -1, 0, 0, 'Sky Lane, Guild 2', 1000000, 11, 440, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3150, 0, -1, 0, 0, 'Wood Avenue 11', 600000, 11, 165, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3151, 0, -1, 0, 0, 'Wood Avenue 8', 800000, 11, 147, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3152, 0, -1, 0, 0, 'Wood Avenue 7', 800000, 11, 145, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3153, 0, -1, 0, 0, 'Wood Avenue 10a', 200000, 11, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3154, 0, -1, 0, 0, 'Wood Avenue 9a', 200000, 11, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3155, 0, -1, 0, 0, 'Wood Avenue 6a', 300000, 11, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3156, 0, -1, 0, 0, 'Wood Avenue 6b', 200000, 11, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3157, 0, -1, 0, 0, 'Wood Avenue 9b', 200000, 11, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3158, 0, -1, 0, 0, 'Wood Avenue 10b', 200000, 11, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3159, 0, -1, 0, 0, 'Stronghold', 800000, 11, 194, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3160, 0, -1, 0, 0, 'Wood Avenue 5', 300000, 11, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3161, 0, -1, 0, 0, 'Wood Avenue 3', 200000, 11, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3162, 0, -1, 0, 0, 'Wood Avenue 4', 200000, 11, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3163, 0, -1, 0, 0, 'Wood Avenue 2', 200000, 11, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3164, 0, -1, 0, 0, 'Wood Avenue 1', 200000, 11, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3165, 0, -1, 0, 0, 'Wood Avenue 4c', 200000, 11, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3166, 0, -1, 0, 0, 'Wood Avenue 4a', 150000, 11, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3167, 0, -1, 0, 0, 'Wood Avenue 4b', 150000, 11, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3168, 0, -1, 0, 0, 'Stonehome Village 1', 150000, 11, 45, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3169, 0, -1, 0, 0, 'Stonehome Flats, Flat 04', 80000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3171, 0, -1, 0, 0, 'Stonehome Flats, Flat 03', 80000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3173, 0, -1, 0, 0, 'Stonehome Flats, Flat 02', 25000, 11, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3174, 0, -1, 0, 0, 'Stonehome Flats, Flat 01', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3175, 0, -1, 0, 0, 'Stonehome Flats, Flat 13', 80000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3177, 0, -1, 0, 0, 'Stonehome Flats, Flat 11', 50000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3178, 0, -1, 0, 0, 'Stonehome Flats, Flat 14', 80000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3180, 0, -1, 0, 0, 'Stonehome Flats, Flat 12', 50000, 11, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3181, 0, -1, 0, 0, 'Stonehome Village 2', 50000, 11, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3182, 0, -1, 0, 0, 'Stonehome Village 3', 50000, 11, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3183, 0, -1, 0, 0, 'Stonehome Village 4', 80000, 11, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3184, 0, -1, 0, 0, 'Stonehome Village 6', 100000, 11, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3185, 0, -1, 0, 0, 'Stonehome Village 5', 80000, 11, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3186, 0, -1, 0, 0, 'Stonehome Village 7', 100000, 11, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3187, 0, -1, 0, 0, 'Stonehome Village 8', 25000, 11, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3188, 0, -1, 0, 0, 'Stonehome Village 9', 50000, 11, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3189, 0, -1, 0, 0, 'Stonehome Clanhall', 250000, 11, 205, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3190, 0, -1, 0, 0, 'Mad Scientist\'s Lab', 600000, 17, 63, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3191, 0, -1, 0, 0, 'Radiant Plaza 4', 800000, 17, 197, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3192, 0, -1, 0, 0, 'Radiant Plaza 3', 800000, 17, 126, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3193, 0, -1, 0, 0, 'Radiant Plaza 2', 600000, 17, 99, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3194, 0, -1, 0, 0, 'Radiant Plaza 1', 800000, 17, 138, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3195, 0, -1, 0, 0, 'Aureate Court 3', 400000, 17, 131, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3196, 0, -1, 0, 0, 'Aureate Court 4', 400000, 17, 104, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3197, 0, -1, 0, 0, 'Aureate Court 5', 600000, 17, 138, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3198, 0, -1, 0, 0, 'Aureate Court 2', 400000, 17, 125, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3199, 0, -1, 0, 0, 'Aureate Court 1', 600000, 17, 131, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3205, 0, -1, 0, 0, 'Halls of Serenity', 5000000, 17, 478, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3206, 0, -1, 0, 0, 'Fortune Wing 3', 600000, 17, 148, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3207, 0, -1, 0, 0, 'Fortune Wing 4', 600000, 17, 147, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3208, 0, -1, 0, 0, 'Fortune Wing 2', 600000, 17, 148, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3209, 0, -1, 0, 0, 'Fortune Wing 1', 800000, 17, 254, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3211, 0, -1, 0, 0, 'Cascade Towers', 5000000, 17, 419, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3212, 0, -1, 0, 0, 'Luminous Arc 5', 800000, 17, 145, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3213, 0, -1, 0, 0, 'Luminous Arc 2', 600000, 17, 161, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3214, 0, -1, 0, 0, 'Luminous Arc 1', 800000, 17, 167, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3215, 0, -1, 0, 0, 'Luminous Arc 3', 600000, 17, 139, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3216, 0, -1, 0, 0, 'Luminous Arc 4', 800000, 17, 200, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3217, 0, -1, 0, 0, 'Harbour Promenade 1', 800000, 17, 137, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3218, 0, -1, 0, 0, 'Sun Palace', 5000000, 17, 533, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3219, 0, -1, 0, 0, 'Haggler\'s Hangout 3', 300000, 15, 186, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3220, 0, -1, 0, 0, 'Haggler\'s Hangout 7', 400000, 15, 155, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3221, 0, -1, 0, 0, 'Big Game Hunter\'s Lodge', 600000, 15, 164, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3222, 0, -1, 0, 0, 'Haggler\'s Hangout 6', 400000, 15, 143, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3223, 0, -1, 0, 0, 'Haggler\'s Hangout 5 (Shop)', 200000, 15, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3224, 0, -1, 0, 0, 'Haggler\'s Hangout 4b (Shop)', 150000, 15, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3225, 0, -1, 0, 0, 'Haggler\'s Hangout 4a (Shop)', 200000, 15, 44, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3226, 0, -1, 0, 0, 'Haggler\'s Hangout 2', 100000, 15, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3227, 0, -1, 0, 0, 'Haggler\'s Hangout 1', 100000, 15, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3228, 0, -1, 0, 0, 'Bamboo Garden 3', 150000, 15, 44, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3229, 0, -1, 0, 0, 'Bamboo Fortress', 500000, 15, 531, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3230, 0, -1, 0, 0, 'Bamboo Garden 2', 80000, 15, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3231, 0, -1, 0, 0, 'Bamboo Garden 1', 100000, 15, 44, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3232, 0, -1, 0, 0, 'Banana Bay 4', 25000, 15, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3233, 0, -1, 0, 0, 'Banana Bay 2', 50000, 15, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3234, 0, -1, 0, 0, 'Banana Bay 3', 50000, 15, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3235, 0, -1, 0, 0, 'Banana Bay 1', 25000, 15, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3236, 0, -1, 0, 0, 'Crocodile Bridge 1', 80000, 15, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3237, 0, -1, 0, 0, 'Crocodile Bridge 2', 80000, 15, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3238, 0, -1, 0, 0, 'Crocodile Bridge 3', 100000, 15, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3239, 0, -1, 0, 0, 'Crocodile Bridge 4', 300000, 15, 119, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3240, 0, -1, 0, 0, 'Crocodile Bridge 5', 200000, 15, 102, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3241, 0, -1, 0, 0, 'Woodway 1', 80000, 15, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3242, 0, -1, 0, 0, 'Woodway 2', 50000, 15, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3243, 0, -1, 0, 0, 'Woodway 3', 150000, 15, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3244, 0, -1, 0, 0, 'Woodway 4', 25000, 15, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3245, 0, -1, 0, 0, 'Flamingo Flats 5', 150000, 15, 53, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3246, 0, -1, 0, 0, 'Flamingo Flats 4', 80000, 15, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3247, 0, -1, 0, 0, 'Flamingo Flats 1', 50000, 15, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3248, 0, -1, 0, 0, 'Flamingo Flats 2', 80000, 15, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3249, 0, -1, 0, 0, 'Flamingo Flats 3', 50000, 15, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3250, 0, -1, 0, 0, 'Jungle Edge 1', 200000, 15, 63, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3251, 0, -1, 0, 0, 'Jungle Edge 2', 200000, 15, 89, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3252, 0, -1, 0, 0, 'Jungle Edge 4', 80000, 15, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3253, 0, -1, 0, 0, 'Jungle Edge 5', 80000, 15, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3254, 0, -1, 0, 0, 'Jungle Edge 6', 25000, 15, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3255, 0, -1, 0, 0, 'Jungle Edge 3', 80000, 15, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3256, 0, -1, 0, 0, 'River Homes 3', 200000, 15, 111, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3257, 0, -1, 0, 0, 'River Homes 2b', 150000, 15, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3258, 0, -1, 0, 0, 'River Homes 2a', 100000, 15, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3259, 0, -1, 0, 0, 'River Homes 1', 300000, 15, 96, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3260, 0, -1, 0, 0, 'Coconut Quay 4', 150000, 15, 52, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3261, 0, -1, 0, 0, 'Coconut Quay 3', 200000, 15, 50, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3262, 0, -1, 0, 0, 'Coconut Quay 2', 100000, 15, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0);
INSERT INTO `houses` (`id`, `owner`, `new_owner`, `paid`, `warnings`, `name`, `rent`, `town_id`, `size`, `guildid`, `beds`, `bidder`, `bidder_name`, `highest_bid`, `internal_bid`, `bid_end_date`, `state`, `transfer_status`) VALUES
(3263, 0, -1, 0, 0, 'Coconut Quay 1', 150000, 15, 47, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3264, 0, -1, 0, 0, 'Shark Manor', 250000, 15, 173, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3265, 0, -1, 0, 0, 'Glacier Side 2', 300000, 16, 102, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3266, 0, -1, 0, 0, 'Glacier Side 1', 150000, 16, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3267, 0, -1, 0, 0, 'Glacier Side 3', 150000, 16, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3268, 0, -1, 0, 0, 'Glacier Side 4', 150000, 16, 46, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3269, 0, -1, 0, 0, 'Shelf Site', 300000, 16, 98, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3270, 0, -1, 0, 0, 'Spirit Homes 5', 150000, 16, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3271, 0, -1, 0, 0, 'Spirit Homes 4', 80000, 16, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3272, 0, -1, 0, 0, 'Spirit Homes 1', 150000, 16, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3273, 0, -1, 0, 0, 'Spirit Homes 2', 150000, 16, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3274, 0, -1, 0, 0, 'Spirit Homes 3', 300000, 16, 90, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3275, 0, -1, 0, 0, 'Arena Walk 3', 300000, 16, 74, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3276, 0, -1, 0, 0, 'Arena Walk 2', 150000, 16, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3277, 0, -1, 0, 0, 'Arena Walk 1', 300000, 16, 67, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3278, 0, -1, 0, 0, 'Bears Paw 2', 300000, 16, 54, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3279, 0, -1, 0, 0, 'Bears Paw 1', 200000, 16, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3280, 0, -1, 0, 0, 'Crystal Glance', 1000000, 16, 321, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3281, 0, -1, 0, 0, 'Shady Rocks 2', 200000, 16, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3282, 0, -1, 0, 0, 'Shady Rocks 1', 300000, 16, 79, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3283, 0, -1, 0, 0, 'Shady Rocks 3', 300000, 16, 94, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3284, 0, -1, 0, 0, 'Shady Rocks 4 (Shop)', 200000, 16, 61, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3285, 0, -1, 0, 0, 'Shady Rocks 5', 300000, 16, 66, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3286, 0, -1, 0, 0, 'Tusk Flats 2', 80000, 16, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3287, 0, -1, 0, 0, 'Tusk Flats 1', 80000, 16, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3288, 0, -1, 0, 0, 'Tusk Flats 3', 80000, 16, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3289, 0, -1, 0, 0, 'Tusk Flats 4', 25000, 16, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3290, 0, -1, 0, 0, 'Tusk Flats 6', 50000, 16, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3291, 0, -1, 0, 0, 'Tusk Flats 5', 25000, 16, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3292, 0, -1, 0, 0, 'Corner Shop (Shop)', 200000, 16, 50, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3293, 0, -1, 0, 0, 'Bears Paw 5', 200000, 16, 45, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3294, 0, -1, 0, 0, 'Bears Paw 4', 400000, 16, 119, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3295, 0, -1, 0, 0, 'Trout Plaza 2', 150000, 16, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3296, 0, -1, 0, 0, 'Trout Plaza 1', 200000, 16, 56, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3297, 0, -1, 0, 0, 'Trout Plaza 5 (Shop)', 300000, 16, 89, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3298, 0, -1, 0, 0, 'Trout Plaza 3', 80000, 16, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3299, 0, -1, 0, 0, 'Trout Plaza 4', 80000, 16, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3300, 0, -1, 0, 0, 'Skiffs End 2', 80000, 16, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3301, 0, -1, 0, 0, 'Skiffs End 1', 100000, 16, 35, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3302, 0, -1, 0, 0, 'Furrier Quarter 3', 100000, 16, 40, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3303, 0, -1, 0, 0, 'Fimbul Shelf 4', 100000, 16, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3304, 0, -1, 0, 0, 'Fimbul Shelf 3', 100000, 16, 49, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3305, 0, -1, 0, 0, 'Furrier Quarter 2', 80000, 16, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3306, 0, -1, 0, 0, 'Furrier Quarter 1', 150000, 16, 53, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3307, 0, -1, 0, 0, 'Fimbul Shelf 2', 100000, 16, 43, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3308, 0, -1, 0, 0, 'Fimbul Shelf 1', 80000, 16, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3309, 0, -1, 0, 0, 'Bears Paw 3', 200000, 16, 47, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3310, 0, -1, 0, 0, 'Raven Corner 2', 150000, 16, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3311, 0, -1, 0, 0, 'Raven Corner 1', 80000, 16, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3312, 0, -1, 0, 0, 'Raven Corner 3', 100000, 16, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3313, 0, -1, 0, 0, 'Mammoth Belly', 1000000, 16, 404, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3314, 0, -1, 0, 0, 'Darashia 3, Flat 01', 150000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3315, 0, -1, 0, 0, 'Darashia 3, Flat 05', 150000, 13, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3316, 0, -1, 0, 0, 'Darashia 3, Flat 02', 200000, 13, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3317, 0, -1, 0, 0, 'Darashia 3, Flat 04', 150000, 13, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3318, 0, -1, 0, 0, 'Darashia 3, Flat 03', 150000, 13, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3319, 0, -1, 0, 0, 'Darashia 3, Flat 12', 200000, 13, 56, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3320, 0, -1, 0, 0, 'Darashia 3, Flat 11', 100000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3321, 0, -1, 0, 0, 'Darashia 3, Flat 14', 200000, 13, 59, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3322, 0, -1, 0, 0, 'Darashia 3, Flat 13', 100000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3323, 0, -1, 0, 0, 'Darashia 8, Flat 01', 300000, 13, 55, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3325, 0, -1, 0, 0, 'Darashia 8, Flat 05', 300000, 13, 58, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3326, 0, -1, 0, 0, 'Darashia 8, Flat 04', 200000, 13, 63, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3327, 0, -1, 0, 0, 'Darashia 8, Flat 03', 300000, 13, 105, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3328, 0, -1, 0, 0, 'Darashia 8, Flat 12', 150000, 13, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3329, 0, -1, 0, 0, 'Darashia 8, Flat 11', 200000, 13, 46, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3330, 0, -1, 0, 0, 'Darashia 8, Flat 14', 150000, 13, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3331, 0, -1, 0, 0, 'Darashia 8, Flat 13', 150000, 13, 46, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3332, 0, -1, 0, 0, 'Darashia, Villa', 800000, 13, 120, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3333, 0, -1, 0, 0, 'Darashia, Eastern Guildhall', 1000000, 13, 272, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3334, 0, -1, 0, 0, 'Darashia, Western Guildhall', 500000, 13, 223, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3335, 0, -1, 0, 0, 'Darashia 2, Flat 03', 100000, 13, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3336, 0, -1, 0, 0, 'Darashia 2, Flat 02', 100000, 13, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3337, 0, -1, 0, 0, 'Darashia 2, Flat 01', 150000, 13, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3338, 0, -1, 0, 0, 'Darashia 2, Flat 04', 80000, 13, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3339, 0, -1, 0, 0, 'Darashia 2, Flat 05', 150000, 13, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3340, 0, -1, 0, 0, 'Darashia 2, Flat 06', 80000, 13, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3341, 0, -1, 0, 0, 'Darashia 2, Flat 07', 150000, 13, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3342, 0, -1, 0, 0, 'Darashia 2, Flat 13', 100000, 13, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3343, 0, -1, 0, 0, 'Darashia 2, Flat 14', 50000, 13, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3344, 0, -1, 0, 0, 'Darashia 2, Flat 15', 100000, 13, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3345, 0, -1, 0, 0, 'Darashia 2, Flat 16', 80000, 13, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3346, 0, -1, 0, 0, 'Darashia 2, Flat 17', 100000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3347, 0, -1, 0, 0, 'Darashia 2, Flat 18', 100000, 13, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3348, 0, -1, 0, 0, 'Darashia 2, Flat 11', 100000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3349, 0, -1, 0, 0, 'Darashia 2, Flat 12', 80000, 13, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3350, 0, -1, 0, 0, 'Darashia 1, Flat 03', 300000, 13, 65, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3351, 0, -1, 0, 0, 'Darashia 1, Flat 04', 100000, 13, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3352, 0, -1, 0, 0, 'Darashia 1, Flat 02', 100000, 13, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3353, 0, -1, 0, 0, 'Darashia 1, Flat 01', 100000, 13, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3354, 0, -1, 0, 0, 'Darashia 1, Flat 05', 100000, 13, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3355, 0, -1, 0, 0, 'Darashia 1, Flat 12', 150000, 13, 46, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3356, 0, -1, 0, 0, 'Darashia 1, Flat 13', 150000, 13, 50, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3357, 0, -1, 0, 0, 'Darashia 1, Flat 14', 200000, 13, 69, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3358, 0, -1, 0, 0, 'Darashia 1, Flat 11', 100000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3359, 0, -1, 0, 0, 'Darashia 5, Flat 02', 150000, 13, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3360, 0, -1, 0, 0, 'Darashia 5, Flat 01', 150000, 13, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3361, 0, -1, 0, 0, 'Darashia 5, Flat 05', 100000, 13, 29, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3362, 0, -1, 0, 0, 'Darashia 5, Flat 04', 150000, 13, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3363, 0, -1, 0, 0, 'Darashia 5, Flat 03', 150000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3364, 0, -1, 0, 0, 'Darashia 5, Flat 11', 150000, 13, 46, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3365, 0, -1, 0, 0, 'Darashia 5, Flat 12', 150000, 13, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3366, 0, -1, 0, 0, 'Darashia 5, Flat 13', 150000, 13, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3367, 0, -1, 0, 0, 'Darashia 5, Flat 14', 150000, 13, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3368, 0, -1, 0, 0, 'Darashia 6a', 300000, 13, 67, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3369, 0, -1, 0, 0, 'Darashia 6b', 300000, 13, 80, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3370, 0, -1, 0, 0, 'Darashia 4, Flat 02', 200000, 13, 44, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3371, 0, -1, 0, 0, 'Darashia 4, Flat 03', 150000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3372, 0, -1, 0, 0, 'Darashia 4, Flat 04', 200000, 13, 45, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3373, 0, -1, 0, 0, 'Darashia 4, Flat 05', 150000, 13, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3374, 0, -1, 0, 0, 'Darashia 4, Flat 01', 100000, 13, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3375, 0, -1, 0, 0, 'Darashia 4, Flat 12', 200000, 13, 64, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3376, 0, -1, 0, 0, 'Darashia 4, Flat 11', 100000, 13, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3377, 0, -1, 0, 0, 'Darashia 4, Flat 13', 200000, 13, 44, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3378, 0, -1, 0, 0, 'Darashia 4, Flat 14', 150000, 13, 46, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3379, 0, -1, 0, 0, 'Darashia 7, Flat 01', 100000, 13, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3380, 0, -1, 0, 0, 'Darashia 7, Flat 02', 100000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3381, 0, -1, 0, 0, 'Darashia 7, Flat 03', 200000, 13, 65, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3382, 0, -1, 0, 0, 'Darashia 7, Flat 05', 150000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3383, 0, -1, 0, 0, 'Darashia 7, Flat 04', 150000, 13, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3384, 0, -1, 0, 0, 'Darashia 7, Flat 12', 200000, 13, 60, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3385, 0, -1, 0, 0, 'Darashia 7, Flat 11', 100000, 13, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3386, 0, -1, 0, 0, 'Darashia 7, Flat 14', 200000, 13, 60, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3387, 0, -1, 0, 0, 'Darashia 7, Flat 13', 100000, 13, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3388, 0, -1, 0, 0, 'Pirate Shipwreck 1', 800000, 13, 187, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3389, 0, -1, 0, 0, 'Pirate Shipwreck 2', 800000, 13, 276, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3390, 0, -1, 0, 0, 'The Shelter', 250000, 14, 422, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3391, 0, -1, 0, 0, 'Litter Promenade 1', 25000, 14, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3392, 0, -1, 0, 0, 'Litter Promenade 2', 50000, 14, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3394, 0, -1, 0, 0, 'Litter Promenade 3', 25000, 14, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3395, 0, -1, 0, 0, 'Litter Promenade 4', 25000, 14, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3396, 0, -1, 0, 0, 'Rum Alley 3', 25000, 14, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3397, 0, -1, 0, 0, 'Straycat\'s Corner 5', 80000, 14, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3398, 0, -1, 0, 0, 'Straycat\'s Corner 6', 25000, 14, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3399, 0, -1, 0, 0, 'Litter Promenade 5', 25000, 14, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3401, 0, -1, 0, 0, 'Straycat\'s Corner 4', 50000, 14, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3402, 0, -1, 0, 0, 'Straycat\'s Corner 2', 50000, 14, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3403, 0, -1, 0, 0, 'Straycat\'s Corner 1', 25000, 14, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3404, 0, -1, 0, 0, 'Rum Alley 2', 25000, 14, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3405, 0, -1, 0, 0, 'Rum Alley 1', 25000, 14, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3406, 0, -1, 0, 0, 'Smuggler Backyard 3', 50000, 14, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3407, 0, -1, 0, 0, 'Shady Trail 3', 25000, 14, 16, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3408, 0, -1, 0, 0, 'Shady Trail 1', 100000, 14, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3409, 0, -1, 0, 0, 'Shady Trail 2', 25000, 14, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3410, 0, -1, 0, 0, 'Smuggler Backyard 4', 25000, 14, 22, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3411, 0, -1, 0, 0, 'Smuggler Backyard 2', 25000, 14, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3412, 0, -1, 0, 0, 'Smuggler Backyard 1', 25000, 14, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3413, 0, -1, 0, 0, 'Smuggler Backyard 5', 25000, 14, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3414, 0, -1, 0, 0, 'Sugar Street 1', 200000, 14, 60, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3415, 0, -1, 0, 0, 'Sugar Street 2', 150000, 14, 51, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3416, 0, -1, 0, 0, 'Sugar Street 3a', 100000, 14, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3417, 0, -1, 0, 0, 'Sugar Street 3b', 150000, 14, 41, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3418, 0, -1, 0, 0, 'Sugar Street 4d', 50000, 14, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3419, 0, -1, 0, 0, 'Sugar Street 4c', 25000, 14, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3420, 0, -1, 0, 0, 'Sugar Street 4b', 100000, 14, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3421, 0, -1, 0, 0, 'Sugar Street 4a', 80000, 14, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3422, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 01', 50000, 14, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3423, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 03', 50000, 14, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3424, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 05', 50000, 14, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3425, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 06', 50000, 14, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3426, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 04', 50000, 14, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3427, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 02', 50000, 14, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3428, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 07', 80000, 14, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3429, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 09', 50000, 14, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3430, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 11', 25000, 14, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3431, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 08', 50000, 14, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3432, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 10', 50000, 14, 18, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3433, 0, -1, 0, 0, 'Harvester\'s Haven, Flat 12', 25000, 14, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3434, 0, -1, 0, 0, 'Marble Lane 3', 600000, 14, 163, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3435, 0, -1, 0, 0, 'Marble Lane 2', 400000, 14, 141, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3436, 0, -1, 0, 0, 'Marble Lane 4', 400000, 14, 134, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3437, 0, -1, 0, 0, 'Admiral\'s Avenue 1', 400000, 14, 97, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3438, 0, -1, 0, 0, 'Admiral\'s Avenue 2', 400000, 14, 111, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3439, 0, -1, 0, 0, 'Admiral\'s Avenue 3', 300000, 14, 99, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3440, 0, -1, 0, 0, 'Ivory Circle 1', 400000, 14, 101, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3441, 0, -1, 0, 0, 'Sugar Street 5', 150000, 14, 25, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3442, 0, -1, 0, 0, 'Freedom Street 1', 200000, 14, 47, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3443, 0, -1, 0, 0, 'Trader\'s Point 1', 200000, 14, 42, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3444, 0, -1, 0, 0, 'Trader\'s Point 2 (Shop)', 600000, 14, 122, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3445, 0, -1, 0, 0, 'Trader\'s Point 3 (Shop)', 600000, 14, 130, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3446, 0, -1, 0, 0, 'Ivory Mansion', 800000, 14, 319, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3447, 0, -1, 0, 0, 'Ivory Circle 2', 400000, 14, 142, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3448, 0, -1, 0, 0, 'Ivy Cottage', 500000, 14, 587, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3449, 0, -1, 0, 0, 'Marble Lane 1', 600000, 14, 228, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3450, 0, -1, 0, 0, 'Freedom Street 2', 400000, 14, 123, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3452, 0, -1, 0, 0, 'Meriana Beach', 150000, 14, 172, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3453, 0, -1, 0, 0, 'The Tavern 1a', 150000, 14, 52, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3454, 0, -1, 0, 0, 'The Tavern 1b', 100000, 14, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3455, 0, -1, 0, 0, 'The Tavern 1c', 200000, 14, 85, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3456, 0, -1, 0, 0, 'The Tavern 1d', 100000, 14, 33, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3457, 0, -1, 0, 0, 'The Tavern 2a', 300000, 14, 111, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3458, 0, -1, 0, 0, 'The Tavern 2b', 100000, 14, 36, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3459, 0, -1, 0, 0, 'The Tavern 2d', 100000, 14, 27, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3460, 0, -1, 0, 0, 'The Tavern 2c', 50000, 14, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3461, 0, -1, 0, 0, 'The Yeah Beach Project', 150000, 14, 155, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3462, 0, -1, 0, 0, 'Mountain Hideout', 500000, 14, 321, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3463, 0, -1, 0, 0, 'Darashia 8, Flat 02', 300000, 13, 76, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3464, 0, -1, 0, 0, 'Castle, Basement, Flat 01', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3465, 0, -1, 0, 0, 'Castle, Basement, Flat 02', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3466, 0, -1, 0, 0, 'Castle, Basement, Flat 03', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3467, 0, -1, 0, 0, 'Castle, Basement, Flat 05', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3468, 0, -1, 0, 0, 'Castle, Basement, Flat 04', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3469, 0, -1, 0, 0, 'Castle, Basement, Flat 06', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3470, 0, -1, 0, 0, 'Castle, Basement, Flat 07', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3471, 0, -1, 0, 0, 'Castle, Basement, Flat 09', 25000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3472, 0, -1, 0, 0, 'Castle, Basement, Flat 08', 50000, 11, 13, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3473, 0, -1, 0, 0, 'Cormaya 1', 150000, 11, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3474, 0, -1, 0, 0, 'Cormaya Flats, Flat 01', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3475, 0, -1, 0, 0, 'Cormaya Flats, Flat 02', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3476, 0, -1, 0, 0, 'Cormaya Flats, Flat 03', 50000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3477, 0, -1, 0, 0, 'Cormaya Flats, Flat 06', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3478, 0, -1, 0, 0, 'Cormaya Flats, Flat 05', 25000, 11, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3479, 0, -1, 0, 0, 'Cormaya Flats, Flat 04', 50000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3480, 0, -1, 0, 0, 'Cormaya Flats, Flat 11', 100000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3482, 0, -1, 0, 0, 'Cormaya Flats, Flat 13', 25000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3483, 0, -1, 0, 0, 'Cormaya Flats, Flat 12', 100000, 11, 24, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3485, 0, -1, 0, 0, 'Cormaya Flats, Flat 14', 25000, 11, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3486, 0, -1, 0, 0, 'Cormaya 2', 300000, 11, 84, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3487, 0, -1, 0, 0, 'Cormaya 4', 150000, 11, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3488, 0, -1, 0, 0, 'Cormaya 3', 200000, 11, 47, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3489, 0, -1, 0, 0, 'Cormaya 6', 200000, 11, 56, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3490, 0, -1, 0, 0, 'Cormaya 7', 200000, 11, 54, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3491, 0, -1, 0, 0, 'Cormaya 8', 200000, 11, 65, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3492, 0, -1, 0, 0, 'Cormaya 5', 300000, 11, 123, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3493, 0, -1, 0, 0, 'Castle of the White Dragon', 1000000, 11, 532, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3494, 0, -1, 0, 0, 'Cormaya 9b', 150000, 11, 58, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3495, 0, -1, 0, 0, 'Cormaya 9a', 80000, 11, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3496, 0, -1, 0, 0, 'Cormaya 9d', 150000, 11, 60, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3497, 0, -1, 0, 0, 'Cormaya 9c', 80000, 11, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3498, 0, -1, 0, 0, 'Cormaya 10', 300000, 11, 85, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3499, 0, -1, 0, 0, 'Cormaya 11', 150000, 11, 47, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3500, 0, -1, 0, 0, 'Edron Flats, Flat 22', 50000, 11, 12, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3501, 0, -1, 0, 0, 'Magic Academy, Shop', 150000, 11, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3502, 0, -1, 0, 0, 'Magic Academy, Flat 1', 100000, 11, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3503, 0, -1, 0, 0, 'Magic Academy, Guild', 500000, 11, 195, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3504, 0, -1, 0, 0, 'Magic Academy, Flat 2', 80000, 11, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3505, 0, -1, 0, 0, 'Magic Academy, Flat 3', 100000, 11, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3506, 0, -1, 0, 0, 'Magic Academy, Flat 4', 100000, 11, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3507, 0, -1, 0, 0, 'Magic Academy, Flat 5', 80000, 11, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3508, 0, -1, 0, 0, 'Oskahl I f', 100000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3509, 0, -1, 0, 0, 'Oskahl I g', 100000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3510, 0, -1, 0, 0, 'Oskahl I h', 150000, 10, 39, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3511, 0, -1, 0, 0, 'Oskahl I i', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3512, 0, -1, 0, 0, 'Oskahl I j', 80000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3513, 0, -1, 0, 0, 'Oskahl I b', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3514, 0, -1, 0, 0, 'Oskahl I d', 100000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3515, 0, -1, 0, 0, 'Oskahl I e', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3516, 0, -1, 0, 0, 'Oskahl I c', 80000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3517, 0, -1, 0, 0, 'Chameken I', 100000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3518, 0, -1, 0, 0, 'Chameken II', 80000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3519, 0, -1, 0, 0, 'Charsirakh III', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3520, 0, -1, 0, 0, 'Charsirakh II', 100000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3521, 0, -1, 0, 0, 'Murkhol I a', 80000, 10, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3523, 0, -1, 0, 0, 'Murkhol I c', 50000, 10, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3524, 0, -1, 0, 0, 'Murkhol I b', 50000, 10, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3525, 0, -1, 0, 0, 'Charsirakh I b', 150000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3526, 0, -1, 0, 0, 'Harrah I', 250000, 10, 124, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3527, 0, -1, 0, 0, 'Thanah I d', 200000, 10, 52, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3528, 0, -1, 0, 0, 'Thanah I c', 200000, 10, 61, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3529, 0, -1, 0, 0, 'Thanah I b', 150000, 10, 56, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3530, 0, -1, 0, 0, 'Thanah I a', 25000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3531, 0, -1, 0, 0, 'Othehothep I c', 150000, 10, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3532, 0, -1, 0, 0, 'Othehothep I d', 150000, 10, 43, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3533, 0, -1, 0, 0, 'Othehothep I b', 100000, 10, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3534, 0, -1, 0, 0, 'Othehothep II c', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3535, 0, -1, 0, 0, 'Othehothep II d', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3536, 0, -1, 0, 0, 'Othehothep II e', 150000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3537, 0, -1, 0, 0, 'Othehothep II f', 100000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3538, 0, -1, 0, 0, 'Othehothep II b', 150000, 10, 43, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3539, 0, -1, 0, 0, 'Othehothep II a', 25000, 10, 10, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3540, 0, -1, 0, 0, 'Mothrem I', 80000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3541, 0, -1, 0, 0, 'Arakmehn I', 100000, 10, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3542, 0, -1, 0, 0, 'Arakmehn II', 80000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3543, 0, -1, 0, 0, 'Arakmehn III', 100000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3544, 0, -1, 0, 0, 'Arakmehn IV', 100000, 10, 28, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3545, 0, -1, 0, 0, 'Unklath II b', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3546, 0, -1, 0, 0, 'Unklath II c', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3547, 0, -1, 0, 0, 'Unklath II d', 100000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3548, 0, -1, 0, 0, 'Unklath II a', 50000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3549, 0, -1, 0, 0, 'Rathal I b', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3550, 0, -1, 0, 0, 'Rathal I c', 25000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3551, 0, -1, 0, 0, 'Rathal I d', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3552, 0, -1, 0, 0, 'Rathal I e', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3553, 0, -1, 0, 0, 'Rathal I a', 80000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3554, 0, -1, 0, 0, 'Rathal II b', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3555, 0, -1, 0, 0, 'Rathal II c', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3556, 0, -1, 0, 0, 'Rathal II d', 100000, 10, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3557, 0, -1, 0, 0, 'Rathal II a', 80000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3558, 0, -1, 0, 0, 'Esuph I', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3559, 0, -1, 0, 0, 'Esuph II b', 100000, 10, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3560, 0, -1, 0, 0, 'Esuph II a', 25000, 10, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3561, 0, -1, 0, 0, 'Esuph III b', 100000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3562, 0, -1, 0, 0, 'Esuph III a', 25000, 10, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3564, 0, -1, 0, 0, 'Esuph IV c', 80000, 10, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3565, 0, -1, 0, 0, 'Esuph IV d', 25000, 10, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3566, 0, -1, 0, 0, 'Esuph IV a', 25000, 10, 10, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3567, 0, -1, 0, 0, 'Horakhal', 250000, 10, 205, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3568, 0, -1, 0, 0, 'Botham II d', 100000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3569, 0, -1, 0, 0, 'Botham II e', 100000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3570, 0, -1, 0, 0, 'Botham II f', 80000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3571, 0, -1, 0, 0, 'Botham II g', 80000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3572, 0, -1, 0, 0, 'Botham II c', 100000, 10, 23, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3573, 0, -1, 0, 0, 'Botham II b', 100000, 10, 30, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3574, 0, -1, 0, 0, 'Botham II a', 25000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3575, 0, -1, 0, 0, 'Botham III f', 150000, 10, 43, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3576, 0, -1, 0, 0, 'Botham III h', 200000, 10, 71, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3577, 0, -1, 0, 0, 'Botham III g', 100000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3578, 0, -1, 0, 0, 'Botham III b', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3579, 0, -1, 0, 0, 'Botham III c', 25000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3581, 0, -1, 0, 0, 'Botham III e', 100000, 10, 38, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3582, 0, -1, 0, 0, 'Botham III a', 80000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3583, 0, -1, 0, 0, 'Botham IV f', 100000, 10, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3584, 0, -1, 0, 0, 'Botham IV h', 100000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3585, 0, -1, 0, 0, 'Botham IV i', 150000, 10, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3586, 0, -1, 0, 0, 'Botham IV g', 100000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3587, 0, -1, 0, 0, 'Botham IV e', 100000, 10, 85, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3591, 0, -1, 0, 0, 'Botham IV a', 100000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3592, 0, -1, 0, 0, 'Ramen Tah', 250000, 10, 125, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3593, 0, -1, 0, 0, 'Botham I c', 150000, 10, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3594, 0, -1, 0, 0, 'Botham I e', 80000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3595, 0, -1, 0, 0, 'Botham I d', 150000, 10, 57, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3596, 0, -1, 0, 0, 'Botham I b', 150000, 10, 56, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3597, 0, -1, 0, 0, 'Botham I a', 50000, 10, 19, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3598, 0, -1, 0, 0, 'Charsirakh I a', 25000, 10, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3599, 0, -1, 0, 0, 'Low Waters Observatory', 400000, 10, 525, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3600, 0, -1, 0, 0, 'Oskahl I a', 150000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3601, 0, -1, 0, 0, 'Othehothep I a', 25000, 10, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3602, 0, -1, 0, 0, 'Othehothep III a', 25000, 10, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3603, 0, -1, 0, 0, 'Othehothep III b', 80000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3604, 0, -1, 0, 0, 'Othehothep III c', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3605, 0, -1, 0, 0, 'Othehothep III d', 80000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3606, 0, -1, 0, 0, 'Othehothep III e', 50000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3607, 0, -1, 0, 0, 'Othehothep III f', 50000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3608, 0, -1, 0, 0, 'Unklath I f', 100000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3609, 0, -1, 0, 0, 'Unklath I g', 100000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3610, 0, -1, 0, 0, 'Unklath I d', 150000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3611, 0, -1, 0, 0, 'Unklath I e', 150000, 10, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3612, 0, -1, 0, 0, 'Unklath I b', 100000, 10, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3613, 0, -1, 0, 0, 'Unklath I c', 100000, 10, 34, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3614, 0, -1, 0, 0, 'Unklath I a', 100000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3615, 0, -1, 0, 0, 'Thanah II a', 25000, 10, 17, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3616, 0, -1, 0, 0, 'Thanah II b', 50000, 10, 9, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3617, 0, -1, 0, 0, 'Thanah II d', 50000, 10, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3618, 0, -1, 0, 0, 'Thanah II e', 25000, 10, 7, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3619, 0, -1, 0, 0, 'Thanah II c', 25000, 10, 9, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3620, 0, -1, 0, 0, 'Thanah II f', 150000, 10, 53, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3621, 0, -1, 0, 0, 'Thanah II g', 100000, 10, 31, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3622, 0, -1, 0, 0, 'Thanah II h', 100000, 10, 26, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3623, 0, -1, 0, 0, 'Thrarhor I a (Shop)', 50000, 10, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3624, 0, -1, 0, 0, 'Thrarhor I c (Shop)', 50000, 10, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3625, 0, -1, 0, 0, 'Thrarhor I d (Shop)', 80000, 10, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3626, 0, -1, 0, 0, 'Thrarhor I b (Shop)', 50000, 10, 15, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3627, 0, -1, 0, 0, 'Uthemath I a', 25000, 10, 10, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3628, 0, -1, 0, 0, 'Uthemath I b', 50000, 10, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3629, 0, -1, 0, 0, 'Uthemath I c', 80000, 10, 20, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3630, 0, -1, 0, 0, 'Uthemath I d', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3631, 0, -1, 0, 0, 'Uthemath I e', 80000, 10, 21, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3632, 0, -1, 0, 0, 'Uthemath I f', 150000, 10, 56, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3633, 0, -1, 0, 0, 'Uthemath II', 250000, 10, 93, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3634, 0, -1, 0, 0, 'Marketplace 1', 400000, 22, 79, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3635, 0, -1, 0, 0, 'Marketplace 2', 400000, 22, 92, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3636, 0, -1, 0, 0, 'Quay 1', 200000, 22, 81, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3637, 0, -1, 0, 0, 'Quay 2', 200000, 22, 130, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3638, 0, -1, 0, 0, 'Halls of Sun and Sea', 1000000, 22, 423, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3639, 0, -1, 0, 0, 'Palace Vicinity', 200000, 22, 132, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3640, 0, -1, 0, 0, 'Wave Tower', 400000, 22, 212, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3641, 0, -1, 0, 0, 'Old Sanctuary of God King Qjell', 300000, 18, 699, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3642, 0, -1, 0, 0, 'Old Heritage Estate', 600000, 20, 335, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3643, 0, -1, 0, 0, 'Rathleton Plaza 4', 400000, 20, 144, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3644, 0, -1, 0, 0, 'Rathleton Plaza 3', 400000, 20, 157, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3645, 0, -1, 0, 0, 'Rathleton Plaza 2', 400000, 20, 77, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3646, 0, -1, 0, 0, 'Rathleton Plaza 1', 300000, 20, 80, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3647, 0, -1, 0, 0, 'Antimony Lane 2', 400000, 20, 127, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3648, 0, -1, 0, 0, 'Antimony Lane 1', 400000, 20, 189, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3649, 0, -1, 0, 0, 'Wallside Residence', 400000, 20, 182, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3650, 0, -1, 0, 0, 'Wallside Lane 1', 800000, 20, 216, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3651, 0, -1, 0, 0, 'Wallside Lane 2', 600000, 20, 227, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3652, 0, -1, 0, 0, 'Vanward Flats B', 400000, 20, 179, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3653, 0, -1, 0, 0, 'Vanward Flats A', 400000, 20, 189, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3654, 0, -1, 0, 0, 'Bronze Brothers Bastion', 5000000, 20, 945, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3655, 0, -1, 0, 0, 'Cistern Ave', 300000, 20, 111, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3656, 0, -1, 0, 0, 'Antimony Lane 4', 400000, 20, 159, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3657, 0, -1, 0, 0, 'Antimony Lane 3', 400000, 20, 101, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3658, 0, -1, 0, 0, 'Rathleton Hills Residence', 400000, 20, 186, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3659, 0, -1, 0, 0, 'Rathleton Hills Estate', 1000000, 20, 534, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3660, 0, -1, 0, 0, 'Lion\'s Head Reef', 400000, 14, 166, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3661, 0, -1, 0, 0, 'Shadow Caves 1', 50000, 5, 32, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3662, 0, -1, 0, 0, 'Shadow Caves 2', 50000, 5, 37, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3663, 0, -1, 0, 0, 'Shadow Caves 3', 100000, 5, 61, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3664, 0, -1, 0, 0, 'Shadow Caves 4', 100000, 5, 53, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3665, 0, -1, 0, 0, 'Shadow Caves 5', 100000, 5, 61, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3666, 0, -1, 0, 0, 'Shadow Caves 6', 100000, 5, 50, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3667, 0, -1, 0, 0, 'Northport Clanhall', 250000, 6, 172, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3668, 0, -1, 0, 0, 'The Treehouse', 250000, 15, 972, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3669, 0, -1, 0, 0, 'Frost Manor', 500000, 16, 505, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3670, 0, -1, 0, 0, 'Hare\'s Den', 150000, 7, 304, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3671, 0, -1, 0, 0, 'Lost Cavern', 200000, 7, 705, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3673, 0, -1, 0, 0, 'Caveman Shelter', 150000, 12, 137, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3674, 0, -1, 0, 0, 'Eastern House of Tranquility', 200000, 12, 313, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3675, 0, -1, 0, 0, 'Lakeside Mansion', 300000, 16, 136, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3676, 0, -1, 0, 0, 'Pilchard Bin 1', 80000, 16, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3677, 0, -1, 0, 0, 'Pilchard Bin 2', 50000, 16, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3678, 0, -1, 0, 0, 'Pilchard Bin 3', 50000, 16, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3679, 0, -1, 0, 0, 'Pilchard Bin 4', 50000, 16, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3680, 0, -1, 0, 0, 'Pilchard Bin 5', 80000, 16, 14, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3681, 0, -1, 0, 0, 'Pilchard Bin 6', 25000, 16, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3682, 0, -1, 0, 0, 'Pilchard Bin 7', 25000, 16, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3683, 0, -1, 0, 0, 'Pilchard Bin 8', 25000, 16, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3684, 0, -1, 0, 0, 'Pilchard Bin 9', 50000, 16, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3685, 0, -1, 0, 0, 'Pilchard Bin 10', 50000, 16, 11, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3686, 0, -1, 0, 0, 'Mammoth House', 300000, 16, 280, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3687, 0, -1, 0, 0, 'Cherry Cake Tower', 800000, 29, 189, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3688, 0, -1, 0, 0, 'Blueberry Bay', 600000, 29, 130, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3689, 0, -1, 0, 0, 'Vanilla Beach', 600000, 29, 129, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3690, 0, -1, 0, 0, 'Centre 1', 600000, 30, 126, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3691, 0, -1, 0, 0, 'Centre 2', 600000, 30, 139, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3693, 0, -1, 0, 0, 'Cliffside', 600000, 31, 203, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3694, 0, -1, 0, 0, 'House of the Rising Moon', 1000000, 31, 340, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3695, 0, -1, 0, 0, 'Marketplace 3', 400000, 22, 130, NULL, 0, 0, '', 0, 0, 0, 0, 0),
(3696, 0, -1, 0, 0, 'Hanging Gardens 1', 400000, 22, 178, NULL, 0, 0, '', 0, 0, 0, 0, 0);

-- --------------------------------------------------------

--
-- Table structure for table `house_lists`
--

CREATE TABLE `house_lists` (
  `house_id` int(11) NOT NULL,
  `listid` int(11) NOT NULL,
  `version` bigint(20) NOT NULL DEFAULT 0,
  `list` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `ip_bans`
--

CREATE TABLE `ip_bans` (
  `ip` int(11) NOT NULL,
  `reason` varchar(255) NOT NULL,
  `banned_at` bigint(20) NOT NULL,
  `expires_at` bigint(20) NOT NULL,
  `banned_by` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `kv_store`
--

CREATE TABLE `kv_store` (
  `key_name` varchar(191) NOT NULL,
  `timestamp` bigint(20) NOT NULL,
  `value` longblob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `kv_store`
--

INSERT INTO `kv_store` (`key_name`, `timestamp`, `value`) VALUES
('migrations.20231128213158_move_hireling_data_to_kv', 1766344574111, 0x3001),
('migrations.20241708000535_move_achievement_to_kv', 1766344574214, 0x3001),
('migrations.20241708362079_move_vip_system_to_kv', 1766344574306, 0x3001),
('migrations.20241708485868_move_some_storages_to_kv', 1766344574401, 0x3001),
('migrations.20241715984279_move_wheel_scrolls_from_storagename_to_kv', 1766344574495, 0x3001),
('migrations.20241715984294_quests_storages_to_kv', 1766344574582, 0x3001),
('migrations.20251737599334_reset_charms', 1766344574671, 0x3001),
('player.7.combat-protection', 1766364222569, 0x19000000000000f03f),
('player.7.features.autoloot', 1766364531498, 0x19000000000000f03f),
('player.7.features.chainSystem', 1766364569221, 0x190000000000000000),
('player.7.summary.hirelings.amount', 1766364045578, 0x1000),
('player.7.titles.unlocked.Admirer of the Crown', 1766364569218, 0x1099aba2ca06),
('player.7.titles.unlocked.Beaststrider (Grade 1)', 1766364569217, 0x1099aba2ca06),
('player.7.titles.unlocked.Beaststrider (Grade 2)', 1766364569217, 0x1099aba2ca06),
('player.7.titles.unlocked.Beaststrider (Grade 3)', 1766364569217, 0x1099aba2ca06),
('player.7.titles.unlocked.Beaststrider (Grade 4)', 1766364569217, 0x1099aba2ca06),
('player.7.titles.unlocked.Beaststrider (Grade 5)', 1766364569217, 0x1099aba2ca06),
('player.7.titles.unlocked.Big Spender', 1766364569218, 0x1099aba2ca06),
('player.7.titles.unlocked.Royal Bounacean Advisor', 1766364569218, 0x1099aba2ca06),
('player.7.tracker-new-quest', 1766364569407, 0x19000000000000f03f),
('player.7.untracker-quest', 1766364569407, 0x19000000000000f03f),
('quest.soul-war.ebb-and-flow-maps.is-active', 1766364489826, 0x3000),
('quest.soul-war.ebb-and-flow-maps.is-loaded-empty-map', 1766364489826, 0x3001),
('raids.candia.sugarmommy.checks-today', 1766345914115, 0x190000000000003740),
('raids.candia.sugarmommy.failed-attempts', 1766364069819, 0x190000000000000000),
('raids.candia.sugarmommy.last-occurrence', 1766364069820, 0x19000040e92452da41),
('raids.candia.sugarmommy.trigger-when-possible', 1766364069819, 0x3000),
('raids.thais.wild-horses.checks-today', 1766344654112, 0x190000000000000040),
('raids.thais.wild-horses.failed-attempts', 1766344654112, 0x190000000000000040);

-- --------------------------------------------------------

--
-- Table structure for table `market_history`
--

CREATE TABLE `market_history` (
  `id` int(11) NOT NULL,
  `player_id` int(11) NOT NULL,
  `sale` tinyint(1) NOT NULL DEFAULT 0,
  `itemtype` int(10) UNSIGNED NOT NULL,
  `amount` smallint(5) UNSIGNED NOT NULL,
  `price` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `expires_at` bigint(20) UNSIGNED NOT NULL,
  `inserted` bigint(20) UNSIGNED NOT NULL,
  `state` tinyint(1) UNSIGNED NOT NULL,
  `tier` tinyint(3) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `market_offers`
--

CREATE TABLE `market_offers` (
  `id` int(11) NOT NULL,
  `player_id` int(11) NOT NULL,
  `sale` tinyint(1) NOT NULL DEFAULT 0,
  `itemtype` int(10) UNSIGNED NOT NULL,
  `amount` smallint(5) UNSIGNED NOT NULL,
  `created` bigint(20) UNSIGNED NOT NULL,
  `anonymous` tinyint(1) NOT NULL DEFAULT 0,
  `price` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `tier` tinyint(3) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_account_actions`
--

CREATE TABLE `myaac_account_actions` (
  `account_id` int(11) NOT NULL,
  `ip` varchar(16) NOT NULL DEFAULT '0.0.0.0',
  `ipv6` binary(16) NOT NULL DEFAULT '0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0',
  `date` int(11) NOT NULL DEFAULT 0,
  `action` varchar(255) NOT NULL DEFAULT ''
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_account_actions`
--

INSERT INTO `myaac_account_actions` (`account_id`, `ip`, `ipv6`, `date`, `action`) VALUES
(2, '0', 0x00000000000000000000000000000001, 1766362992, 'Account created.'),
(3, '0', 0x00000000000000000000000000000001, 1766364820, 'Account created.'),
(3, '0', 0x00000000000000000000000000000001, 1766364821, 'Created character <b>Pudinzao</b>.');

-- --------------------------------------------------------

--
-- Table structure for table `myaac_admin_menu`
--

CREATE TABLE `myaac_admin_menu` (
  `id` int(11) NOT NULL,
  `name` varchar(255) NOT NULL DEFAULT '',
  `page` varchar(255) NOT NULL DEFAULT '',
  `ordering` int(11) NOT NULL DEFAULT 0,
  `flags` int(11) NOT NULL DEFAULT 0,
  `enabled` int(1) NOT NULL DEFAULT 1
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_bugtracker`
--

CREATE TABLE `myaac_bugtracker` (
  `account` varchar(255) NOT NULL,
  `type` int(11) NOT NULL DEFAULT 0,
  `status` int(11) NOT NULL DEFAULT 0,
  `text` text NOT NULL,
  `id` int(11) NOT NULL DEFAULT 0,
  `subject` varchar(255) NOT NULL DEFAULT '',
  `reply` int(11) NOT NULL DEFAULT 0,
  `who` int(11) NOT NULL DEFAULT 0,
  `uid` int(11) NOT NULL,
  `tag` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_changelog`
--

CREATE TABLE `myaac_changelog` (
  `id` int(11) NOT NULL,
  `body` varchar(500) NOT NULL DEFAULT '',
  `type` tinyint(1) NOT NULL DEFAULT 0 COMMENT '1 - added, 2 - removed, 3 - changed, 4 - fixed',
  `where` tinyint(1) NOT NULL DEFAULT 0 COMMENT '1 - server, 2 - site',
  `date` int(11) NOT NULL DEFAULT 0,
  `player_id` int(11) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_changelog`
--

INSERT INTO `myaac_changelog` (`id`, `body`, `type`, `where`, `date`, `player_id`, `hidden`) VALUES
(1, 'MyAAC installed. (:', 3, 2, 1766362978, 0, 0);

-- --------------------------------------------------------

--
-- Table structure for table `myaac_charbazaar`
--

CREATE TABLE `myaac_charbazaar` (
  `id` int(11) NOT NULL,
  `account_old` int(11) NOT NULL,
  `account_new` int(11) NOT NULL,
  `player_id` int(11) NOT NULL,
  `price` int(11) NOT NULL,
  `date_end` datetime NOT NULL,
  `date_start` datetime NOT NULL,
  `bid_account` int(11) NOT NULL,
  `bid_price` int(11) NOT NULL,
  `status` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_charbazaar_bid`
--

CREATE TABLE `myaac_charbazaar_bid` (
  `id` int(11) NOT NULL,
  `account_id` int(11) NOT NULL,
  `auction_id` int(11) NOT NULL,
  `bid` int(11) NOT NULL,
  `date` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_config`
--

CREATE TABLE `myaac_config` (
  `id` int(11) NOT NULL,
  `name` varchar(30) NOT NULL,
  `value` varchar(1000) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_config`
--

INSERT INTO `myaac_config` (`id`, `name`, `value`) VALUES
(1, 'database_version', '35'),
(2, 'status_online', ''),
(3, 'status_players', '0'),
(4, 'status_playersMax', '0'),
(5, 'status_lastCheck', '1766364393'),
(6, 'status_uptime', '383'),
(7, 'status_monsters', '86039'),
(8, 'views_counter', '58'),
(9, 'status_uptimeReadable', '12 months, 31 days, 21h 06m'),
(10, 'status_motd', 'Welcome to the Crystal Server!'),
(11, 'status_mapAuthor', ''),
(12, 'status_mapName', 'world'),
(13, 'status_mapWidth', '35143'),
(14, 'status_mapHeight', '34812'),
(15, 'status_server', 'Crystal Server'),
(16, 'status_serverVersion', '4.1.7'),
(17, 'status_clientVersion', '15.11');

-- --------------------------------------------------------

--
-- Table structure for table `myaac_faq`
--

CREATE TABLE `myaac_faq` (
  `id` int(11) NOT NULL,
  `question` varchar(255) NOT NULL DEFAULT '',
  `answer` varchar(1020) NOT NULL DEFAULT '',
  `ordering` int(11) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_forum`
--

CREATE TABLE `myaac_forum` (
  `id` int(11) NOT NULL,
  `first_post` int(11) NOT NULL DEFAULT 0,
  `last_post` int(11) NOT NULL DEFAULT 0,
  `section` int(3) NOT NULL DEFAULT 0,
  `replies` int(20) NOT NULL DEFAULT 0,
  `views` int(20) NOT NULL DEFAULT 0,
  `author_aid` int(20) NOT NULL DEFAULT 0,
  `author_guid` int(20) NOT NULL DEFAULT 0,
  `post_text` text NOT NULL,
  `post_topic` varchar(255) NOT NULL DEFAULT '',
  `post_smile` tinyint(1) NOT NULL DEFAULT 0,
  `post_html` tinyint(1) NOT NULL DEFAULT 0,
  `post_date` int(20) NOT NULL DEFAULT 0,
  `last_edit_aid` int(20) NOT NULL DEFAULT 0,
  `edit_date` int(20) NOT NULL DEFAULT 0,
  `post_ip` varchar(32) NOT NULL DEFAULT '0.0.0.0',
  `sticked` tinyint(1) NOT NULL DEFAULT 0,
  `closed` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_forum_boards`
--

CREATE TABLE `myaac_forum_boards` (
  `id` int(11) NOT NULL,
  `name` varchar(32) NOT NULL,
  `description` varchar(255) NOT NULL DEFAULT '',
  `ordering` int(11) NOT NULL DEFAULT 0,
  `guild` int(11) NOT NULL DEFAULT 0,
  `access` int(11) NOT NULL DEFAULT 0,
  `closed` tinyint(1) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_forum_boards`
--

INSERT INTO `myaac_forum_boards` (`id`, `name`, `description`, `ordering`, `guild`, `access`, `closed`, `hidden`) VALUES
(1, 'News', 'News commenting', 0, 0, 0, 1, 0),
(2, 'Trade', 'Trade offers.', 1, 0, 0, 0, 0),
(3, 'Quests', 'Quest making.', 2, 0, 0, 0, 0),
(4, 'Pictures', 'Your pictures.', 3, 0, 0, 0, 0),
(5, 'Bug Report', 'Report bugs there.', 4, 0, 0, 0, 0);

-- --------------------------------------------------------

--
-- Table structure for table `myaac_gallery`
--

CREATE TABLE `myaac_gallery` (
  `id` int(11) NOT NULL,
  `comment` varchar(255) NOT NULL DEFAULT '',
  `image` varchar(255) NOT NULL,
  `thumb` varchar(255) NOT NULL,
  `author` varchar(50) NOT NULL DEFAULT '',
  `ordering` int(11) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_gallery`
--

INSERT INTO `myaac_gallery` (`id`, `comment`, `image`, `thumb`, `author`, `ordering`, `hidden`) VALUES
(1, 'Demon', 'images/gallery/demon.jpg', 'images/gallery/demon_thumb.gif', 'MyAAC', 1, 0);

-- --------------------------------------------------------

--
-- Table structure for table `myaac_menu`
--

CREATE TABLE `myaac_menu` (
  `id` int(11) NOT NULL,
  `template` varchar(255) NOT NULL,
  `name` varchar(255) NOT NULL,
  `link` varchar(255) NOT NULL,
  `blank` tinyint(1) NOT NULL DEFAULT 0,
  `color` varchar(6) NOT NULL DEFAULT '',
  `category` int(11) NOT NULL DEFAULT 1,
  `ordering` int(11) NOT NULL DEFAULT 0,
  `enabled` int(1) NOT NULL DEFAULT 1
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_menu`
--

INSERT INTO `myaac_menu` (`id`, `template`, `name`, `link`, `blank`, `color`, `category`, `ordering`, `enabled`) VALUES
(90, 'tibiacom', 'Latest News', 'news', 0, '', 1, 0, 1),
(91, 'tibiacom', 'News Archive', 'news/archive', 0, '', 1, 1, 1),
(92, 'tibiacom', 'Event Schedule', 'eventcalendar', 0, '', 1, 2, 1),
(93, 'tibiacom', 'Account Management', 'account/manage', 0, '', 2, 0, 1),
(94, 'tibiacom', 'Create Account', 'account/create', 0, '', 2, 1, 1),
(95, 'tibiacom', 'Lost Account?', 'account/lost', 0, '', 2, 2, 1),
(96, 'tibiacom', 'Server Rules', 'rules', 0, '', 2, 3, 1),
(97, 'tibiacom', 'Downloads', 'downloadclient', 0, '', 2, 4, 1),
(98, 'tibiacom', 'Characters', 'characters', 0, '', 3, 0, 1),
(99, 'tibiacom', 'Who Is Online?', 'online', 0, '', 3, 1, 1),
(100, 'tibiacom', 'Highscores', 'highscores', 0, '', 3, 2, 1),
(101, 'tibiacom', 'Last Kills', 'lastkills', 0, '', 3, 3, 1),
(102, 'tibiacom', 'Houses', 'houses', 0, '', 3, 4, 1),
(103, 'tibiacom', 'Guilds', 'guilds', 0, '', 3, 5, 1),
(104, 'tibiacom', 'Bans', 'bans', 0, '', 3, 7, 1),
(105, 'tibiacom', 'Support List', 'team', 0, '', 3, 8, 1),
(106, 'tibiacom', 'Creatures', 'creatures', 0, '', 5, 0, 1),
(107, 'tibiacom', 'Spells', 'spells', 0, '', 5, 1, 1),
(108, 'tibiacom', 'Server Info', 'serverInfo', 0, '', 5, 2, 1),
(109, 'tibiacom', 'Current Auctions', 'currentcharactertrades', 0, '', 7, 0, 1),
(110, 'tibiacom', 'Auction History', 'pastcharactertrades', 0, '', 7, 1, 1),
(111, 'tibiacom', 'My Bids', 'ownbids', 0, '', 7, 2, 1),
(112, 'tibiacom', 'My Auctions', 'owncharactertrades', 0, '', 7, 3, 1),
(113, 'tibiacom', 'Create Auction', 'createcharacterauction', 0, '', 7, 4, 1),
(114, 'tibiacom', 'Donate', 'donate', 0, 'ffff00', 6, 0, 1);

-- --------------------------------------------------------

--
-- Table structure for table `myaac_monsters`
--

CREATE TABLE `myaac_monsters` (
  `id` int(11) NOT NULL,
  `hidden` tinyint(1) NOT NULL DEFAULT 0,
  `name` varchar(255) NOT NULL,
  `mana` int(11) NOT NULL DEFAULT 0,
  `exp` int(11) NOT NULL,
  `health` int(11) NOT NULL,
  `speed_lvl` int(11) NOT NULL DEFAULT 1,
  `use_haste` tinyint(1) NOT NULL,
  `voices` text NOT NULL,
  `immunities` varchar(255) NOT NULL,
  `elements` text NOT NULL,
  `summonable` tinyint(1) NOT NULL,
  `convinceable` tinyint(1) NOT NULL,
  `pushable` tinyint(1) NOT NULL DEFAULT 0,
  `canpushitems` tinyint(1) NOT NULL DEFAULT 0,
  `canwalkonenergy` tinyint(1) NOT NULL DEFAULT 0,
  `canwalkonpoison` tinyint(1) NOT NULL DEFAULT 0,
  `canwalkonfire` tinyint(1) NOT NULL DEFAULT 0,
  `runonhealth` tinyint(1) NOT NULL DEFAULT 0,
  `hostile` tinyint(1) NOT NULL DEFAULT 0,
  `attackable` tinyint(1) NOT NULL DEFAULT 0,
  `rewardboss` tinyint(1) NOT NULL DEFAULT 0,
  `defense` int(11) NOT NULL DEFAULT 0,
  `armor` int(11) NOT NULL DEFAULT 0,
  `canpushcreatures` tinyint(1) NOT NULL DEFAULT 0,
  `race` varchar(255) NOT NULL,
  `loot` text NOT NULL,
  `summons` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_news`
--

CREATE TABLE `myaac_news` (
  `id` int(11) NOT NULL,
  `title` varchar(100) NOT NULL,
  `body` text NOT NULL,
  `type` tinyint(1) NOT NULL DEFAULT 0 COMMENT '1 - news, 2 - ticker, 3 - article',
  `date` int(11) NOT NULL DEFAULT 0,
  `category` tinyint(1) NOT NULL DEFAULT 0,
  `player_id` int(11) NOT NULL DEFAULT 0,
  `last_modified_by` int(11) NOT NULL DEFAULT 0,
  `last_modified_date` int(11) NOT NULL DEFAULT 0,
  `comments` varchar(50) NOT NULL DEFAULT '',
  `article_text` varchar(300) NOT NULL DEFAULT '',
  `article_image` varchar(100) NOT NULL DEFAULT '',
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_news`
--

INSERT INTO `myaac_news` (`id`, `title`, `body`, `type`, `date`, `category`, `player_id`, `last_modified_by`, `last_modified_date`, `comments`, `article_text`, `article_image`, `hidden`) VALUES
(1, 'Eu sou um News!', '<p>Fazendo Tibia, quero agradecer a todos que acompanham e apoiam o canal. Se inscreva e deixe o like!</p>\r\n<p>&nbsp;</p>\r\n<p>www.youtube.com/@fazendotibia/videos</p>', 1, 1766362992, 2, 8, 8, 1766363847, 'https://github.com/jprzimba/crystalserver-aac', '', 'images/news/announcement.jpg', 0),
(2, 'Hello tickets!', '<p>N&atilde;o percam as &uacute;ltimas atualiza&ccedil;&otilde;es entrando em nosso canal no youtube!</p>\r\n<p>www.youtube.com/@fazendotibia/videos</p>', 2, 1766362992, 4, 8, 8, 1766363884, '', '', 'images/news/announcement.jpg', 0);

-- --------------------------------------------------------

--
-- Table structure for table `myaac_news_categories`
--

CREATE TABLE `myaac_news_categories` (
  `id` int(11) NOT NULL,
  `name` varchar(50) NOT NULL DEFAULT '',
  `description` varchar(50) NOT NULL DEFAULT '',
  `icon_id` int(2) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_news_categories`
--

INSERT INTO `myaac_news_categories` (`id`, `name`, `description`, `icon_id`, `hidden`) VALUES
(1, '', '', 0, 0),
(2, '', '', 1, 0),
(3, '', '', 2, 0),
(4, '', '', 3, 0),
(5, '', '', 4, 0);

-- --------------------------------------------------------

--
-- Table structure for table `myaac_notepad`
--

CREATE TABLE `myaac_notepad` (
  `id` int(11) NOT NULL,
  `account_id` int(11) NOT NULL,
  `content` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_pages`
--

CREATE TABLE `myaac_pages` (
  `id` int(11) NOT NULL,
  `name` varchar(30) NOT NULL,
  `title` varchar(30) NOT NULL,
  `body` longtext NOT NULL,
  `date` int(11) NOT NULL DEFAULT 0,
  `player_id` int(11) NOT NULL DEFAULT 0,
  `php` tinyint(1) NOT NULL DEFAULT 0 COMMENT '0 - plain html, 1 - php',
  `enable_tinymce` tinyint(1) NOT NULL DEFAULT 1 COMMENT '1 - enabled, 0 - disabled',
  `access` tinyint(2) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_pages`
--

INSERT INTO `myaac_pages` (`id`, `name`, `title`, `body`, `date`, `player_id`, `php`, `enable_tinymce`, `access`, `hidden`) VALUES
(1, 'downloads', 'Downloads', '<p>&nbsp;</p>\n<p>&nbsp;</p>\n<div style=\"text-align: center;\">We\'re using official Tibia Client <strong>{{ config.client / 100 }}</strong><br />\n<p>Download Tibia Client <strong>{{ config.client / 100 }}</strong>&nbsp;for Windows <a href=\"https://drive.google.com/drive/folders/0B2-sMQkWYzhGSFhGVlY2WGk5czQ\" target=\"_blank\" rel=\"noopener\">HERE</a>.</p>\n<h2>IP Changer:</h2>\n<a href=\"https://static.otland.net/ipchanger.exe\" target=\"_blank\" rel=\"noopener\">HERE</a></div>', 0, 1, 0, 1, 1, 0),
(2, 'commands', 'Commands', '<table style=\"border-collapse: collapse; width: 87.8471%; height: 57px;\" border=\"1\">\n<tbody>\n<tr style=\"height: 18px;\">\n<td style=\"width: 33.3333%; background-color: #505050; height: 18px;\"><span style=\"color: #ffffff;\"><strong>Words</strong></span></td>\n<td style=\"width: 33.3333%; background-color: #505050; height: 18px;\"><span style=\"color: #ffffff;\"><strong>Description</strong></span></td>\n</tr>\n<tr style=\"height: 18px; background-color: #f1e0c6;\">\n<td style=\"width: 33.3333%; height: 18px;\"><em>!example</em></td>\n<td style=\"width: 33.3333%; height: 18px;\">This is just an example</td>\n</tr>\n<tr style=\"height: 18px; background-color: #d4c0a1;\">\n<td style=\"width: 33.3333%; height: 18px;\"><em>!buyhouse</em></td>\n<td style=\"width: 33.3333%; height: 18px;\">Buy house you are looking at</td>\n</tr>\n<tr style=\"height: 18px; background-color: #f1e0c6;\">\n<td style=\"width: 33.3333%; height: 18px;\"><em>!aol</em></td>\n<td style=\"width: 33.3333%; height: 18px;\">Buy AoL</td>\n</tr>\n</tbody>\n</table>', 0, 1, 0, 1, 1, 0),
(3, 'rules_on_the_page', 'Rules', '1. Names\na) Names which contain insulting (e.g. \"Bastard\"), racist (e.g. \"Nigger\"), extremely right-wing (e.g. \"Hitler\"), sexist (e.g. \"Bitch\") or offensive (e.g. \"Copkiller\") language.\nb) Names containing parts of sentences (e.g. \"Mike returns\"), nonsensical combinations of letters (e.g. \"Fgfshdsfg\") or invalid formattings (e.g. \"Thegreatknight\").\nc) Names that obviously do not describe a person (e.g. \"Christmastree\", \"Matrix\"), names of real life celebrities (e.g. \"Britney Spears\"), names that refer to real countries (e.g. \"Swedish Druid\"), names which were created to fake other players\' identities (e.g. \"Arieswer\" instead of \"Arieswar\") or official positions (e.g. \"System Admin\").\n\n2. Cheating\na) Exploiting obvious errors of the game (\"bugs\"), for instance to duplicate items. If you find an error you must report it to CipSoft immediately.\nb) Intentional abuse of weaknesses in the gameplay, for example arranging objects or players in a way that other players cannot move them.\nc) Using tools to automatically perform or repeat certain actions without any interaction by the player (\"macros\").\nd) Manipulating the client program or using additional software to play the game.\ne) Trying to steal other players\' account data (\"hacking\").\nf) Playing on more than one account at the same time (\"multi-clienting\").\ng) Offering account data to other players or accepting other players\' account data (\"account-trading/sharing\").\n\n3. Gamemasters\na) Threatening a gamemaster because of his or her actions or position as a gamemaster.\nb) Pretending to be a gamemaster or to have influence on the decisions of a gamemaster.\nc) Intentionally giving wrong or misleading information to a gamemaster concerning his or her investigations or making false reports about rule violations.\n\n4. Player Killing\na) Excessive killing of characters who are not marked with a \"skull\" on worlds which are not PvP-enforced. Please note that killing marked characters is not a reason for a banishment.\n\nA violation of the Tibia Rules may lead to temporary banishment of characters and accounts. In severe cases removal or modification of character skills, attributes and belongings, as well as the permanent removal of accounts without any compensation may be considered. The sanction is based on the seriousness of the rule violation and the previous record of the player. It is determined by the gamemaster imposing the banishment.\n\nThese rules may be changed at any time. All changes will be announced on the official website.', 0, 1, 0, 0, 1, 0);

-- --------------------------------------------------------

--
-- Table structure for table `myaac_polls`
--

CREATE TABLE `myaac_polls` (
  `id` int(11) NOT NULL,
  `question` varchar(255) NOT NULL,
  `description` varchar(255) NOT NULL,
  `end` int(11) NOT NULL,
  `start` int(11) NOT NULL,
  `answers` int(11) NOT NULL,
  `votes_all` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_polls_answers`
--

CREATE TABLE `myaac_polls_answers` (
  `poll_id` int(11) NOT NULL,
  `answer_id` int(11) NOT NULL,
  `answer` varchar(255) NOT NULL,
  `votes` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_spells`
--

CREATE TABLE `myaac_spells` (
  `id` int(11) NOT NULL,
  `spell` varchar(255) NOT NULL DEFAULT '',
  `name` varchar(255) NOT NULL,
  `words` varchar(255) NOT NULL DEFAULT '',
  `category` tinyint(1) NOT NULL DEFAULT 0 COMMENT '1 - attack, 2 - healing, 3 - summon, 4 - supply, 5 - support',
  `type` tinyint(1) NOT NULL DEFAULT 0 COMMENT '1 - instant, 2 - conjure, 3 - rune',
  `level` int(11) NOT NULL DEFAULT 0,
  `maglevel` int(11) NOT NULL DEFAULT 0,
  `mana` int(11) NOT NULL DEFAULT 0,
  `soul` tinyint(3) NOT NULL DEFAULT 0,
  `conjure_id` int(11) NOT NULL DEFAULT 0,
  `conjure_count` tinyint(3) NOT NULL DEFAULT 0,
  `reagent` int(11) NOT NULL DEFAULT 0,
  `item_id` int(11) NOT NULL DEFAULT 0,
  `premium` tinyint(1) NOT NULL DEFAULT 0,
  `vocations` varchar(100) NOT NULL DEFAULT '',
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_videos`
--

CREATE TABLE `myaac_videos` (
  `id` int(11) NOT NULL,
  `title` varchar(100) NOT NULL DEFAULT '',
  `youtube_id` varchar(20) NOT NULL,
  `author` varchar(50) NOT NULL DEFAULT '',
  `ordering` int(11) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `myaac_visitors`
--

CREATE TABLE `myaac_visitors` (
  `ip` varchar(45) NOT NULL,
  `lastvisit` int(11) NOT NULL DEFAULT 0,
  `page` varchar(2048) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `myaac_visitors`
--

INSERT INTO `myaac_visitors` (`ip`, `lastvisit`, `page`) VALUES
('::1', 1766364828, '/?characters/Pudinzao');

-- --------------------------------------------------------

--
-- Table structure for table `myaac_weapons`
--

CREATE TABLE `myaac_weapons` (
  `id` int(11) NOT NULL,
  `level` int(11) NOT NULL DEFAULT 0,
  `maglevel` int(11) NOT NULL DEFAULT 0,
  `vocations` varchar(100) NOT NULL DEFAULT ''
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `players`
--

CREATE TABLE `players` (
  `id` int(11) NOT NULL,
  `name` varchar(255) NOT NULL,
  `group_id` int(11) NOT NULL DEFAULT 1,
  `account_id` int(11) UNSIGNED NOT NULL DEFAULT 0,
  `level` int(11) NOT NULL DEFAULT 1,
  `vocation` int(11) NOT NULL DEFAULT 0,
  `health` int(11) NOT NULL DEFAULT 150,
  `healthmax` int(11) NOT NULL DEFAULT 150,
  `experience` bigint(20) NOT NULL DEFAULT 0,
  `lookbody` int(11) NOT NULL DEFAULT 0,
  `lookfeet` int(11) NOT NULL DEFAULT 0,
  `lookhead` int(11) NOT NULL DEFAULT 0,
  `looklegs` int(11) NOT NULL DEFAULT 0,
  `looktype` int(11) NOT NULL DEFAULT 136,
  `lookaddons` int(11) NOT NULL DEFAULT 0,
  `maglevel` int(11) NOT NULL DEFAULT 0,
  `mana` int(11) NOT NULL DEFAULT 0,
  `manamax` int(11) NOT NULL DEFAULT 0,
  `manaspent` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `soul` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `town_id` int(11) NOT NULL DEFAULT 1,
  `posx` int(11) NOT NULL DEFAULT 0,
  `posy` int(11) NOT NULL DEFAULT 0,
  `posz` int(11) NOT NULL DEFAULT 0,
  `conditions` mediumblob NOT NULL,
  `cap` int(11) NOT NULL DEFAULT 0,
  `sex` int(11) NOT NULL DEFAULT 0,
  `pronoun` int(11) NOT NULL DEFAULT 0,
  `lastlogin` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `lastip` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `save` tinyint(1) NOT NULL DEFAULT 1,
  `skull` tinyint(1) NOT NULL DEFAULT 0,
  `skulltime` bigint(20) NOT NULL DEFAULT 0,
  `lastlogout` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `blessings` tinyint(2) NOT NULL DEFAULT 0,
  `blessings1` tinyint(4) NOT NULL DEFAULT 0,
  `blessings2` tinyint(4) NOT NULL DEFAULT 0,
  `blessings3` tinyint(4) NOT NULL DEFAULT 0,
  `blessings4` tinyint(4) NOT NULL DEFAULT 0,
  `blessings5` tinyint(4) NOT NULL DEFAULT 0,
  `blessings6` tinyint(4) NOT NULL DEFAULT 0,
  `blessings7` tinyint(4) NOT NULL DEFAULT 0,
  `blessings8` tinyint(4) NOT NULL DEFAULT 0,
  `onlinetime` int(11) NOT NULL DEFAULT 0,
  `deletion` bigint(15) NOT NULL DEFAULT 0,
  `balance` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `offlinetraining_time` smallint(5) UNSIGNED NOT NULL DEFAULT 43200,
  `offlinetraining_skill` tinyint(2) NOT NULL DEFAULT -1,
  `stamina` smallint(5) UNSIGNED NOT NULL DEFAULT 2520,
  `skill_fist` int(10) UNSIGNED NOT NULL DEFAULT 10,
  `skill_fist_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_club` int(10) UNSIGNED NOT NULL DEFAULT 10,
  `skill_club_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_sword` int(10) UNSIGNED NOT NULL DEFAULT 10,
  `skill_sword_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_axe` int(10) UNSIGNED NOT NULL DEFAULT 10,
  `skill_axe_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_dist` int(10) UNSIGNED NOT NULL DEFAULT 10,
  `skill_dist_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_shielding` int(10) UNSIGNED NOT NULL DEFAULT 10,
  `skill_shielding_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_fishing` int(10) UNSIGNED NOT NULL DEFAULT 10,
  `skill_fishing_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_critical_hit_chance` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `skill_critical_hit_chance_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_critical_hit_damage` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `skill_critical_hit_damage_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_life_leech_chance` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `skill_life_leech_chance_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_life_leech_amount` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `skill_life_leech_amount_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_mana_leech_chance` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `skill_mana_leech_chance_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_mana_leech_amount` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `skill_mana_leech_amount_tries` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_criticalhit_chance` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_criticalhit_damage` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_lifeleech_chance` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_lifeleech_amount` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_manaleech_chance` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `skill_manaleech_amount` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `manashield` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `max_manashield` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `xpboost_stamina` smallint(5) UNSIGNED DEFAULT NULL,
  `xpboost_value` tinyint(4) UNSIGNED DEFAULT NULL,
  `marriage_status` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `marriage_spouse` int(11) NOT NULL DEFAULT -1,
  `bonus_rerolls` bigint(21) NOT NULL DEFAULT 0,
  `prey_wildcard` bigint(21) NOT NULL DEFAULT 0,
  `task_points` bigint(21) NOT NULL DEFAULT 0,
  `quickloot_fallback` tinyint(1) DEFAULT 0,
  `lookmountbody` tinyint(3) UNSIGNED NOT NULL DEFAULT 0,
  `lookmountfeet` tinyint(3) UNSIGNED NOT NULL DEFAULT 0,
  `lookmounthead` tinyint(3) UNSIGNED NOT NULL DEFAULT 0,
  `lookmountlegs` tinyint(3) UNSIGNED NOT NULL DEFAULT 0,
  `currentmount` smallint(5) UNSIGNED NOT NULL DEFAULT 0,
  `lookfamiliarstype` int(11) UNSIGNED NOT NULL DEFAULT 0,
  `isreward` tinyint(1) NOT NULL DEFAULT 1,
  `istutorial` tinyint(1) NOT NULL DEFAULT 0,
  `ismain` tinyint(1) NOT NULL DEFAULT 0,
  `forge_dusts` bigint(21) NOT NULL DEFAULT 0,
  `forge_dust_level` bigint(21) NOT NULL DEFAULT 100,
  `randomize_mount` tinyint(1) NOT NULL DEFAULT 0,
  `boss_points` int(11) NOT NULL DEFAULT 0,
  `loyalty_points` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `animus_mastery` mediumblob DEFAULT NULL,
  `virtue` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `harmony` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `weapon_proficiencies` mediumblob DEFAULT NULL,
  `created` int(11) NOT NULL DEFAULT 0,
  `hidden` tinyint(1) NOT NULL DEFAULT 0,
  `comment` text NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `players`
--

INSERT INTO `players` (`id`, `name`, `group_id`, `account_id`, `level`, `vocation`, `health`, `healthmax`, `experience`, `lookbody`, `lookfeet`, `lookhead`, `looklegs`, `looktype`, `lookaddons`, `maglevel`, `mana`, `manamax`, `manaspent`, `soul`, `town_id`, `posx`, `posy`, `posz`, `conditions`, `cap`, `sex`, `pronoun`, `lastlogin`, `lastip`, `save`, `skull`, `skulltime`, `lastlogout`, `blessings`, `blessings1`, `blessings2`, `blessings3`, `blessings4`, `blessings5`, `blessings6`, `blessings7`, `blessings8`, `onlinetime`, `deletion`, `balance`, `offlinetraining_time`, `offlinetraining_skill`, `stamina`, `skill_fist`, `skill_fist_tries`, `skill_club`, `skill_club_tries`, `skill_sword`, `skill_sword_tries`, `skill_axe`, `skill_axe_tries`, `skill_dist`, `skill_dist_tries`, `skill_shielding`, `skill_shielding_tries`, `skill_fishing`, `skill_fishing_tries`, `skill_critical_hit_chance`, `skill_critical_hit_chance_tries`, `skill_critical_hit_damage`, `skill_critical_hit_damage_tries`, `skill_life_leech_chance`, `skill_life_leech_chance_tries`, `skill_life_leech_amount`, `skill_life_leech_amount_tries`, `skill_mana_leech_chance`, `skill_mana_leech_chance_tries`, `skill_mana_leech_amount`, `skill_mana_leech_amount_tries`, `skill_criticalhit_chance`, `skill_criticalhit_damage`, `skill_lifeleech_chance`, `skill_lifeleech_amount`, `skill_manaleech_chance`, `skill_manaleech_amount`, `manashield`, `max_manashield`, `xpboost_stamina`, `xpboost_value`, `marriage_status`, `marriage_spouse`, `bonus_rerolls`, `prey_wildcard`, `task_points`, `quickloot_fallback`, `lookmountbody`, `lookmountfeet`, `lookmounthead`, `lookmountlegs`, `currentmount`, `lookfamiliarstype`, `isreward`, `istutorial`, `ismain`, `forge_dusts`, `forge_dust_level`, `randomize_mount`, `boss_points`, `loyalty_points`, `animus_mastery`, `virtue`, `harmony`, `weapon_proficiencies`, `created`, `hidden`, `comment`) VALUES
(1, 'Rook Sample', 1, 2, 2, 0, 155, 155, 100, 113, 115, 95, 39, 128, 0, 2, 60, 60, 5936, 0, 1, 32069, 31901, 6, '', 410, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43200, -1, 2520, 10, 0, 12, 155, 12, 155, 12, 155, 12, 93, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 100, 0, 0, 0, '', 0, 0, 0x0000, 0, 0, ''),
(2, 'Sorcerer Sample', 1, 2, 8, 1, 185, 185, 4200, 113, 115, 95, 39, 130, 0, 0, 90, 90, 0, 0, 8, 32369, 32241, 7, '', 470, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43200, -1, 2520, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 100, 0, 0, 0, '', 0, 0, 0x0000, 0, 0, ''),
(3, 'Druid Sample', 1, 2, 8, 2, 185, 185, 4200, 113, 115, 95, 39, 144, 0, 0, 90, 90, 0, 0, 8, 32369, 32241, 7, '', 470, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43200, -1, 2520, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 100, 0, 0, 0, '', 0, 0, 0x0000, 0, 0, ''),
(4, 'Paladin Sample', 1, 2, 8, 3, 185, 185, 4200, 113, 115, 95, 39, 129, 0, 0, 90, 90, 0, 0, 8, 32369, 32241, 7, '', 470, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43200, -1, 2520, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 100, 0, 0, 0, '', 0, 0, 0x0000, 0, 0, ''),
(5, 'Knight Sample', 1, 2, 8, 4, 185, 185, 4200, 113, 115, 95, 39, 131, 0, 0, 90, 90, 0, 0, 8, 32369, 32241, 7, '', 470, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43200, -1, 2520, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 100, 0, 0, 0, '', 0, 0, 0x0000, 0, 0, ''),
(6, 'Monk Sample', 1, 2, 8, 9, 185, 185, 4200, 113, 115, 95, 39, 1824, 0, 0, 90, 90, 0, 0, 8, 32369, 32241, 7, '', 470, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43200, -1, 2520, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 100, 0, 0, 0, '', 0, 0, 0x0000, 0, 0, ''),
(7, 'ADM', 6, 1, 2, 0, 155, 155, 100, 76, 94, 40, 94, 302, 3, 0, 60, 60, 0, 2, 8, 32369, 32241, 7, '', 410, 1, 0, 1766364569, 16777343, 1, 0, 0, 1766364575, 0, 1, 1, 1, 1, 1, 1, 1, 1, 471, 0, 0, 43200, -1, 2520, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 100, 0, 0, 0, '', 0, 0, 0x0000, 0, 0, '');

--
-- Triggers `players`
--
DELIMITER $$
CREATE TRIGGER `ondelete_players` BEFORE DELETE ON `players` FOR EACH ROW BEGIN
    UPDATE `houses` SET `owner` = 0 WHERE `owner` = OLD.`id`;
END
$$
DELIMITER ;

-- --------------------------------------------------------

--
-- Table structure for table `players_online`
--

CREATE TABLE `players_online` (
  `player_id` int(11) NOT NULL
) ENGINE=MEMORY DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_bosstiary`
--

CREATE TABLE `player_bosstiary` (
  `player_id` int(11) NOT NULL,
  `bossIdSlotOne` int(11) NOT NULL DEFAULT 0,
  `bossIdSlotTwo` int(11) NOT NULL DEFAULT 0,
  `removeTimes` int(11) NOT NULL DEFAULT 1,
  `tracker` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_bosstiary`
--

INSERT INTO `player_bosstiary` (`player_id`, `bossIdSlotOne`, `bossIdSlotTwo`, `removeTimes`, `tracker`) VALUES
(1, 0, 0, 1, ''),
(2, 0, 0, 1, ''),
(3, 0, 0, 1, ''),
(4, 0, 0, 1, ''),
(5, 0, 0, 1, ''),
(6, 0, 0, 1, ''),
(7, 0, 0, 1, '');

-- --------------------------------------------------------

--
-- Table structure for table `player_charms`
--

CREATE TABLE `player_charms` (
  `player_id` int(11) NOT NULL,
  `charm_points` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `minor_charm_echoes` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `max_charm_points` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `max_minor_charm_echoes` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `charm_expansion` tinyint(1) NOT NULL DEFAULT 0,
  `UsedRunesBit` int(11) NOT NULL DEFAULT 0,
  `UnlockedRunesBit` int(11) NOT NULL DEFAULT 0,
  `charms` blob DEFAULT NULL,
  `tracker_list` blob DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_charms`
--

INSERT INTO `player_charms` (`player_id`, `charm_points`, `minor_charm_echoes`, `max_charm_points`, `max_minor_charm_echoes`, `charm_expansion`, `UsedRunesBit`, `UnlockedRunesBit`, `charms`, `tracker_list`) VALUES
(1, 0, 0, 0, 0, 0, 0, 0, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, ''),
(2, 0, 0, 0, 0, 0, 0, 0, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, ''),
(3, 0, 0, 0, 0, 0, 0, 0, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, ''),
(4, 0, 0, 0, 0, 0, 0, 0, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, ''),
(5, 0, 0, 0, 0, 0, 0, 0, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, ''),
(6, 0, 0, 0, 0, 0, 0, 0, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, ''),
(7, 0, 0, 0, 0, 0, 0, 0, 0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000, '');

-- --------------------------------------------------------

--
-- Table structure for table `player_deaths`
--

CREATE TABLE `player_deaths` (
  `player_id` int(11) NOT NULL,
  `time` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `level` int(11) NOT NULL DEFAULT 1,
  `killed_by` varchar(255) NOT NULL,
  `is_player` tinyint(1) NOT NULL DEFAULT 1,
  `mostdamage_by` varchar(100) NOT NULL,
  `mostdamage_is_player` tinyint(1) NOT NULL DEFAULT 0,
  `unjustified` tinyint(1) NOT NULL DEFAULT 0,
  `mostdamage_unjustified` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_depotitems`
--

CREATE TABLE `player_depotitems` (
  `player_id` int(11) NOT NULL,
  `sid` int(11) NOT NULL COMMENT 'any given range eg 0-100 will be reserved for depot lockers and all > 100 will be then normal items inside depots',
  `pid` int(11) NOT NULL DEFAULT 0,
  `itemtype` int(11) NOT NULL DEFAULT 0,
  `count` int(11) NOT NULL DEFAULT 0,
  `attributes` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_hirelings`
--

CREATE TABLE `player_hirelings` (
  `id` int(11) NOT NULL,
  `player_id` int(11) NOT NULL,
  `name` varchar(255) DEFAULT NULL,
  `active` tinyint(3) UNSIGNED NOT NULL DEFAULT 0,
  `sex` tinyint(3) UNSIGNED NOT NULL DEFAULT 0,
  `posx` int(11) NOT NULL DEFAULT 0,
  `posy` int(11) NOT NULL DEFAULT 0,
  `posz` int(11) NOT NULL DEFAULT 0,
  `lookbody` int(11) NOT NULL DEFAULT 0,
  `lookfeet` int(11) NOT NULL DEFAULT 0,
  `lookhead` int(11) NOT NULL DEFAULT 0,
  `looklegs` int(11) NOT NULL DEFAULT 0,
  `looktype` int(11) NOT NULL DEFAULT 136
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_inboxitems`
--

CREATE TABLE `player_inboxitems` (
  `player_id` int(11) NOT NULL,
  `sid` int(11) NOT NULL,
  `pid` int(11) NOT NULL DEFAULT 0,
  `itemtype` int(11) NOT NULL DEFAULT 0,
  `count` int(11) NOT NULL DEFAULT 0,
  `attributes` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_items`
--

CREATE TABLE `player_items` (
  `player_id` int(11) NOT NULL DEFAULT 0,
  `pid` int(11) NOT NULL DEFAULT 0,
  `sid` int(11) NOT NULL DEFAULT 0,
  `itemtype` int(11) NOT NULL DEFAULT 0,
  `count` int(11) NOT NULL DEFAULT 0,
  `attributes` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_items`
--

INSERT INTO `player_items` (`player_id`, `pid`, `sid`, `itemtype`, `count`, `attributes`) VALUES
(1, 11, 101, 23396, 1, ''),
(2, 11, 101, 23396, 1, ''),
(3, 11, 101, 23396, 1, ''),
(4, 11, 101, 23396, 1, ''),
(5, 11, 101, 23396, 1, ''),
(6, 11, 101, 23396, 1, ''),
(7, 1, 101, 3387, 1, 0x280a),
(7, 2, 102, 3057, 1, ''),
(7, 3, 103, 9601, 1, 0x24022c00000080),
(7, 4, 104, 3388, 1, 0x280a),
(7, 5, 105, 3420, 1, ''),
(7, 6, 106, 3288, 1, 0x280a),
(7, 7, 107, 3389, 1, 0x280a),
(7, 8, 108, 3079, 1, ''),
(7, 9, 109, 3006, 1, ''),
(7, 10, 110, 2921, 1, 0x10bb0109001101),
(7, 11, 111, 23396, 1, 0x2401),
(7, 111, 112, 23721, 1, 0x01ca9487439b01000026000000802b07000000);

-- --------------------------------------------------------

--
-- Table structure for table `player_kills`
--

CREATE TABLE `player_kills` (
  `player_id` int(11) NOT NULL,
  `time` bigint(20) UNSIGNED NOT NULL DEFAULT 0,
  `target` int(11) NOT NULL,
  `unavenged` tinyint(1) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_mounts`
--

CREATE TABLE `player_mounts` (
  `player_id` int(11) NOT NULL DEFAULT 0,
  `mount_id` smallint(4) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_namelocks`
--

CREATE TABLE `player_namelocks` (
  `player_id` int(11) NOT NULL,
  `reason` varchar(255) NOT NULL,
  `namelocked_at` bigint(20) NOT NULL,
  `namelocked_by` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_oldnames`
--

CREATE TABLE `player_oldnames` (
  `id` int(11) NOT NULL,
  `player_id` int(11) NOT NULL,
  `former_name` varchar(255) NOT NULL DEFAULT '',
  `name` varchar(255) NOT NULL,
  `old_name` varchar(255) NOT NULL,
  `date` int(11) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_outfits`
--

CREATE TABLE `player_outfits` (
  `player_id` int(11) NOT NULL DEFAULT 0,
  `outfit_id` smallint(4) UNSIGNED NOT NULL DEFAULT 0,
  `addons` tinyint(1) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_outfits`
--

INSERT INTO `player_outfits` (`player_id`, `outfit_id`, `addons`) VALUES
(7, 251, 0),
(7, 252, 0),
(7, 288, 0),
(7, 289, 0);

-- --------------------------------------------------------

--
-- Table structure for table `player_prey`
--

CREATE TABLE `player_prey` (
  `player_id` int(11) NOT NULL,
  `slot` tinyint(1) NOT NULL,
  `state` tinyint(1) NOT NULL,
  `raceid` varchar(250) NOT NULL,
  `option` tinyint(1) NOT NULL,
  `bonus_type` tinyint(1) NOT NULL,
  `bonus_rarity` tinyint(1) NOT NULL,
  `bonus_percentage` varchar(250) NOT NULL,
  `bonus_time` varchar(250) NOT NULL,
  `free_reroll` bigint(20) NOT NULL,
  `monster_list` blob DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_prey`
--

INSERT INTO `player_prey` (`player_id`, `slot`, `state`, `raceid`, `option`, `bonus_type`, `bonus_rarity`, `bonus_percentage`, `bonus_time`, `free_reroll`, `monster_list`) VALUES
(7, 0, 3, '0', 0, 1, 3, '19', '0', 1766436045558, 0x65001b0775001a0048071700820316030500),
(7, 1, 3, '0', 0, 0, 2, '16', '0', 1766436045558, 0xc6060702f2046c005300d4032d0090037103),
(7, 2, 0, '0', 0, 2, 9, '37', '0', 1766436045558, '');

-- --------------------------------------------------------

--
-- Table structure for table `player_rewards`
--

CREATE TABLE `player_rewards` (
  `player_id` int(11) NOT NULL,
  `sid` int(11) NOT NULL,
  `pid` int(11) NOT NULL DEFAULT 0,
  `itemtype` int(11) NOT NULL DEFAULT 0,
  `count` int(11) NOT NULL DEFAULT 0,
  `attributes` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_spells`
--

CREATE TABLE `player_spells` (
  `player_id` int(11) NOT NULL,
  `name` varchar(255) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_stash`
--

CREATE TABLE `player_stash` (
  `player_id` int(16) NOT NULL,
  `item_id` int(16) NOT NULL,
  `item_count` int(32) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_statements`
--

CREATE TABLE `player_statements` (
  `id` int(11) NOT NULL,
  `player_id` int(11) NOT NULL,
  `receiver` text NOT NULL,
  `channel_id` int(11) NOT NULL DEFAULT 0,
  `text` varchar(255) NOT NULL,
  `date` bigint(20) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `player_storage`
--

CREATE TABLE `player_storage` (
  `player_id` int(11) NOT NULL DEFAULT 0,
  `key` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `value` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_storage`
--

INSERT INTO `player_storage` (`player_id`, `key`, `value`) VALUES
(7, 10134, 1),
(7, 10135, 1),
(7, 10136, 2),
(7, 10137, 1),
(7, 12330, 1),
(7, 12332, 13),
(7, 12333, 3),
(7, 12450, 6),
(7, 13414, 12),
(7, 14900, 1),
(7, 14903, 1),
(7, 30057, 1),
(7, 40430, 2),
(7, 40431, 2),
(7, 40432, 2),
(7, 40434, 1),
(7, 40435, 2),
(7, 40436, 2),
(7, 40437, 2),
(7, 40438, 3),
(7, 40439, 1),
(7, 40440, 1),
(7, 40441, 1),
(7, 40442, 3),
(7, 40443, 3),
(7, 40444, 3),
(7, 40445, 1),
(7, 40446, 1),
(7, 40612, 1),
(7, 40613, 18),
(7, 40629, 61),
(7, 40631, 1),
(7, 40638, 2),
(7, 40788, 1),
(7, 40789, 1),
(7, 40790, 1),
(7, 40791, 1),
(7, 40822, 1),
(7, 40823, 2),
(7, 40824, 2),
(7, 40825, 12),
(7, 40827, 3),
(7, 40828, 3),
(7, 40829, 2),
(7, 40830, 1),
(7, 40831, 3),
(7, 40832, 5),
(7, 40833, 1),
(7, 40834, 1),
(7, 40835, 2),
(7, 40836, 1),
(7, 40837, 4),
(7, 41153, 1),
(7, 41174, 8),
(7, 41175, 3),
(7, 41176, 3),
(7, 41177, 3),
(7, 41276, 40),
(7, 41277, 3),
(7, 41278, 5),
(7, 41279, 3),
(7, 41280, 2),
(7, 41281, 6),
(7, 41282, 8),
(7, 41283, 3),
(7, 41284, 4),
(7, 41285, 2),
(7, 41286, 2),
(7, 41287, 2),
(7, 41288, 6),
(7, 41300, 1),
(7, 41306, 1),
(7, 41311, 3),
(7, 41472, 1),
(7, 41476, 1),
(7, 41480, 2),
(7, 41481, 5),
(7, 41482, 3),
(7, 41483, 3),
(7, 41484, 3),
(7, 41485, 2),
(7, 41486, 1),
(7, 41492, 2),
(7, 41685, 2),
(7, 41690, 5),
(7, 41691, 25),
(7, 41692, 7),
(7, 41693, 3),
(7, 41694, 6),
(7, 41695, 3),
(7, 41696, 3),
(7, 41697, 3),
(7, 41698, 1),
(7, 41700, 1),
(7, 41701, 1),
(7, 41703, 1),
(7, 41704, 1),
(7, 41705, 1),
(7, 41710, 1),
(7, 41711, 2),
(7, 41712, 3),
(7, 41713, 3),
(7, 41714, 8),
(7, 41715, 2),
(7, 41716, 4),
(7, 41717, 2),
(7, 41718, 1),
(7, 41901, 1),
(7, 41902, 4),
(7, 41904, 2),
(7, 41905, 2),
(7, 41906, 2),
(7, 41907, 2),
(7, 41908, 2),
(7, 41909, 2),
(7, 41910, 3),
(7, 41911, 1),
(7, 41912, 1),
(7, 41950, 1),
(7, 41951, 51),
(7, 41952, 6),
(7, 41953, 8),
(7, 41954, 6),
(7, 41955, 6),
(7, 41956, 8),
(7, 41957, 5),
(7, 41958, 5),
(7, 41959, 4),
(7, 41960, 2),
(7, 41961, 1),
(7, 41962, 1),
(7, 41963, 1),
(7, 41964, 1),
(7, 41965, 1),
(7, 41966, 1),
(7, 41967, 1),
(7, 41968, 1),
(7, 41969, 1),
(7, 41970, 1),
(7, 41971, 1),
(7, 41972, 1),
(7, 41973, 1),
(7, 41974, 1),
(7, 41975, 1),
(7, 41976, 1),
(7, 41977, 1),
(7, 41978, 1),
(7, 41979, 1),
(7, 41980, 1),
(7, 41983, 1),
(7, 41984, 1),
(7, 41985, 1),
(7, 41986, 1),
(7, 41987, 1),
(7, 41988, 1),
(7, 41989, 1),
(7, 41994, 5),
(7, 41995, 1),
(7, 41996, 1),
(7, 41997, 1),
(7, 41998, 1),
(7, 41999, 1),
(7, 42000, 1),
(7, 42001, 1),
(7, 42002, 1),
(7, 42003, 1),
(7, 42004, 1),
(7, 42006, 1),
(7, 42166, 0),
(7, 42601, 21),
(7, 42602, 2),
(7, 42603, 3),
(7, 42604, 5),
(7, 42605, 3),
(7, 42606, 6),
(7, 42607, 3),
(7, 42608, 1),
(7, 42609, 1),
(7, 42610, 1),
(7, 42611, 1),
(7, 42701, 29),
(7, 42703, 3),
(7, 42704, 4),
(7, 42705, 1),
(7, 42706, 1),
(7, 42707, 1),
(7, 42708, 3),
(7, 42709, 2),
(7, 42710, 2),
(7, 42711, 1),
(7, 42712, 1),
(7, 42713, 1),
(7, 42714, 1),
(7, 42715, 1),
(7, 42716, 1),
(7, 42717, 5),
(7, 42718, 2),
(7, 42720, 2),
(7, 42721, 3),
(7, 42724, 2),
(7, 42725, 1),
(7, 42729, 12),
(7, 42731, 1),
(7, 43000, 29),
(7, 43001, 3),
(7, 43002, 3),
(7, 43003, 3),
(7, 43004, 3),
(7, 43005, 3),
(7, 43006, 4),
(7, 43007, 6),
(7, 43008, 2),
(7, 43009, 2),
(7, 43010, 1),
(7, 43026, 1),
(7, 43027, 1),
(7, 43028, 1),
(7, 43029, 2),
(7, 43030, 1),
(7, 43031, 1),
(7, 43032, 1),
(7, 43033, 1),
(7, 43851, 23),
(7, 43853, 5),
(7, 43854, 2),
(7, 43863, 1440),
(7, 43890, 2),
(7, 43891, 2),
(7, 43892, 2),
(7, 44551, 1),
(7, 44552, 1),
(7, 44553, 1),
(7, 44554, 1),
(7, 44555, 1),
(7, 44556, 1),
(7, 44557, 1),
(7, 44558, 1),
(7, 44559, 1),
(7, 44560, 1),
(7, 44561, 1),
(7, 44562, 1),
(7, 44563, 1),
(7, 44564, 1),
(7, 44565, 1),
(7, 44567, 1),
(7, 44568, 1),
(7, 44569, 1),
(7, 44571, 1),
(7, 44572, 1),
(7, 44579, 1),
(7, 44580, 1),
(7, 44581, 3000),
(7, 44582, 1),
(7, 44583, 1),
(7, 44584, 1),
(7, 44585, 1),
(7, 44586, 1),
(7, 44587, 1),
(7, 44588, 1),
(7, 44752, 1),
(7, 44753, 1),
(7, 44801, 5),
(7, 44802, 1),
(7, 44803, 1),
(7, 44804, 1),
(7, 44805, 1),
(7, 44806, 1),
(7, 44807, 1),
(7, 44808, 1),
(7, 44809, 1),
(7, 44810, 1),
(7, 44811, 1),
(7, 44812, 1),
(7, 45150, 1),
(7, 45151, 1),
(7, 45180, 2),
(7, 45183, 3),
(7, 45215, 1),
(7, 45216, 1),
(7, 45217, 1),
(7, 45218, 1),
(7, 45219, 1),
(7, 45226, 1),
(7, 45472, 1),
(7, 45473, 1),
(7, 45474, 1),
(7, 45475, 1),
(7, 45476, 1),
(7, 45477, 1),
(7, 45488, 1),
(7, 45489, 1),
(7, 45490, 1),
(7, 45491, 1),
(7, 45492, 1),
(7, 45493, 1),
(7, 45494, 1),
(7, 45495, 1),
(7, 45497, 1),
(7, 45500, 1),
(7, 45651, 7),
(7, 45652, 1),
(7, 45653, 1),
(7, 45654, 1),
(7, 45655, 1),
(7, 45656, 1),
(7, 45657, 1),
(7, 45658, 1),
(7, 45659, 1),
(7, 45660, 1),
(7, 45661, 1),
(7, 45662, 1),
(7, 45666, 1),
(7, 45667, 4),
(7, 45668, 3),
(7, 45669, 3),
(7, 45671, 1),
(7, 45672, 1),
(7, 45674, 1),
(7, 45675, 1),
(7, 45676, 1),
(7, 45677, 1),
(7, 45679, 1),
(7, 45680, 1),
(7, 45681, 1),
(7, 45682, 7),
(7, 45683, 1),
(7, 45684, 1),
(7, 45685, 1),
(7, 45686, 10),
(7, 45687, 10),
(7, 45688, 1),
(7, 45690, 1),
(7, 45691, 1),
(7, 45692, 5),
(7, 45693, 1),
(7, 45694, 1),
(7, 45695, 1),
(7, 45698, 1),
(7, 45751, 1),
(7, 45752, 16),
(7, 45764, 1),
(7, 45851, 1),
(7, 45852, 10),
(7, 45853, 2),
(7, 45854, 2),
(7, 45860, 10),
(7, 45861, 2),
(7, 45862, 2),
(7, 45875, 10),
(7, 45876, 3),
(7, 45877, 2),
(7, 45878, 2),
(7, 45899, 10),
(7, 45901, 10),
(7, 45903, 30),
(7, 46001, 1),
(7, 46002, 1),
(7, 46003, 1),
(7, 46017, 6),
(7, 46018, 6),
(7, 46030, 1),
(7, 46047, 4),
(7, 46060, 2),
(7, 46090, 7),
(7, 46091, 1),
(7, 46309, 1),
(7, 46401, 1),
(7, 46402, 1),
(7, 46403, 1),
(7, 46404, 1),
(7, 46851, 14),
(7, 46875, 1),
(7, 47008, 1),
(7, 47010, 1),
(7, 47012, 1),
(7, 47014, 1),
(7, 47016, 1),
(7, 47017, 1),
(7, 47019, 1),
(7, 47401, 1),
(7, 47402, 1),
(7, 47403, 1),
(7, 47501, 1),
(7, 47512, 1),
(7, 47514, 1),
(7, 47601, 1),
(7, 47902, 1),
(7, 47903, 1),
(7, 47904, 1),
(7, 47905, 1),
(7, 47952, 1),
(7, 50011, 5),
(7, 50043, 1),
(7, 50400, 1),
(7, 50401, 1),
(7, 50402, 1),
(7, 50403, 1),
(7, 50404, 1),
(7, 50405, 1),
(7, 50406, 1),
(7, 50960, 1),
(7, 51680, 1),
(7, 52148, 1),
(7, 55047, 1),
(7, 100157, 1);

-- --------------------------------------------------------

--
-- Table structure for table `player_taskhunt`
--

CREATE TABLE `player_taskhunt` (
  `player_id` int(11) NOT NULL,
  `slot` tinyint(1) NOT NULL,
  `state` tinyint(1) NOT NULL,
  `raceid` varchar(250) NOT NULL,
  `upgrade` tinyint(1) NOT NULL,
  `rarity` tinyint(1) NOT NULL,
  `kills` varchar(250) NOT NULL,
  `disabled_time` bigint(20) NOT NULL,
  `free_reroll` bigint(20) NOT NULL,
  `monster_list` blob DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_taskhunt`
--

INSERT INTO `player_taskhunt` (`player_id`, `slot`, `state`, `raceid`, `upgrade`, `rarity`, `kills`, `disabled_time`, `free_reroll`, `monster_list`) VALUES
(7, 0, 2, '0', 0, 1, '0', 0, 1766436045558, 0x6b0a2700d6035f000c008707d700e0007806),
(7, 1, 2, '0', 0, 1, '0', 0, 1766436045558, 0x8201d3030b002f08ff01dd00be064000f000),
(7, 2, 0, '0', 0, 1, '0', 0, 1766436045558, '');

-- --------------------------------------------------------

--
-- Table structure for table `player_wheeldata`
--

CREATE TABLE `player_wheeldata` (
  `player_id` int(11) NOT NULL,
  `slot` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `player_wheeldata`
--

INSERT INTO `player_wheeldata` (`player_id`, `slot`) VALUES
(1, 0x0100000200000300000400000500000600000700000800000900000a00000b00000c00000d00000e00000f00001000001100001200001300001400001500001600001700001800001900001a00001b00001c00001d00001e00001f0000200000210000220000230000240000),
(2, 0x0100000200000300000400000500000600000700000800000900000a00000b00000c00000d00000e00000f00001000001100001200001300001400001500001600001700001800001900001a00001b00001c00001d00001e00001f0000200000210000220000230000240000),
(3, 0x0100000200000300000400000500000600000700000800000900000a00000b00000c00000d00000e00000f00001000001100001200001300001400001500001600001700001800001900001a00001b00001c00001d00001e00001f0000200000210000220000230000240000),
(4, 0x0100000200000300000400000500000600000700000800000900000a00000b00000c00000d00000e00000f00001000001100001200001300001400001500001600001700001800001900001a00001b00001c00001d00001e00001f0000200000210000220000230000240000),
(5, 0x0100000200000300000400000500000600000700000800000900000a00000b00000c00000d00000e00000f00001000001100001200001300001400001500001600001700001800001900001a00001b00001c00001d00001e00001f0000200000210000220000230000240000),
(6, 0x0100000200000300000400000500000600000700000800000900000a00000b00000c00000d00000e00000f00001000001100001200001300001400001500001600001700001800001900001a00001b00001c00001d00001e00001f0000200000210000220000230000240000),
(7, 0x0100000200000300000400000500000600000700000800000900000a00000b00000c00000d00000e00000f00001000001100001200001300001400001500001600001700001800001900001a00001b00001c00001d00001e00001f0000200000210000220000230000240000);

-- --------------------------------------------------------

--
-- Table structure for table `server_config`
--

CREATE TABLE `server_config` (
  `config` varchar(50) NOT NULL,
  `value` varchar(256) NOT NULL DEFAULT '',
  `timestamp` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `server_config`
--

INSERT INTO `server_config` (`config`, `value`, `timestamp`) VALUES
('db_version', '60', '2025-12-22 00:22:59'),
('motd_hash', '7533b0eb1e46eee18600db20a84b8a7cfc29c89b', '2025-12-22 00:22:59'),
('motd_num', '1', '2025-12-22 00:22:59'),
('players_record', '1', '2025-12-22 00:40:45');

-- --------------------------------------------------------

--
-- Table structure for table `store_history`
--

CREATE TABLE `store_history` (
  `id` int(11) NOT NULL,
  `account_id` int(11) UNSIGNED NOT NULL,
  `mode` smallint(2) NOT NULL DEFAULT 0,
  `description` varchar(3500) NOT NULL,
  `coin_type` tinyint(1) NOT NULL DEFAULT 0,
  `coin_amount` int(12) NOT NULL,
  `time` bigint(20) UNSIGNED NOT NULL,
  `timestamp` int(11) NOT NULL DEFAULT 0,
  `coins` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `store_history`
--

INSERT INTO `store_history` (`id`, `account_id`, `mode`, `description`, `coin_type`, `coin_amount`, `time`, `timestamp`, `coins`) VALUES
(1, 1, 0, '360 Days of Premium Time', 1, -3000, 1766364388, 0, 0),
(2, 1, 0, 'Gold Pouch', 1, -900, 1766364517, 0, 0);

-- --------------------------------------------------------

--
-- Table structure for table `tile_store`
--

CREATE TABLE `tile_store` (
  `house_id` int(11) NOT NULL,
  `data` longblob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `tile_store`
--

INSERT INTO `tile_store` (`house_id`, `data`) VALUES
(2918, 0x7a7eeb7d0701000000231900),
(2918, 0x787eea7d07010000005b0b00),
(2918, 0x787ee97d07010000002d0a00),
(2918, 0x787ee87d07010000002d0a00),
(2918, 0x787ee77d07010000005b0b00),
(2919, 0x787eea7d06010000005b0b00),
(2919, 0x787ee97d06010000002d0a00),
(2919, 0x787ee87d06010000002d0a00),
(2919, 0x7b7ee87d0601000000220900),
(2919, 0x787ee77d06010000005b0b00),
(2947, 0x7a7ee47d0701000000660600),
(2947, 0x777ee37d07010000005b0b00),
(2947, 0x7a7ee37d07010000002d0a00),
(2947, 0x7a7ee27d07010000002d0a00),
(2947, 0x7a7ee17d07010000005b0b00),
(2947, 0x7a7ee07d0701000000660600),
(2948, 0x7a7ede7d0601000000660600),
(2948, 0x7a7edd7d06010000005b0b00),
(2949, 0x7b7ee57d06010000005d0b00),
(2949, 0x7a7ee47d0601000000660600),
(2949, 0x777ee37d06010000005b0b00),
(2949, 0x7a7ee37d06010000002d0a00),
(2949, 0x7a7ee27d06010000002d0a00),
(2949, 0x7a7ee17d06010000005b0b00),
(2949, 0x7a7ee07d0601000000660600),
(3220, 0x637fd37f0701000000391900),
(3220, 0x607fd37f0701000000391900);

-- --------------------------------------------------------

--
-- Table structure for table `towns`
--

CREATE TABLE `towns` (
  `id` int(11) NOT NULL,
  `name` varchar(255) NOT NULL,
  `posx` int(11) NOT NULL DEFAULT 0,
  `posy` int(11) NOT NULL DEFAULT 0,
  `posz` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Dumping data for table `towns`
--

INSERT INTO `towns` (`id`, `name`, `posx`, `posy`, `posz`) VALUES
(1, 'Dawnport Tutorial', 32069, 31901, 6),
(2, 'Dawnport', 32064, 31894, 6),
(3, 'Rookgaard', 32097, 32219, 7),
(4, 'Island of Destiny', 32091, 32027, 7),
(5, 'Ab\'Dendriel', 32732, 31634, 7),
(6, 'Carlin', 32360, 31782, 7),
(7, 'Kazordoon', 32649, 31925, 11),
(8, 'Thais', 32369, 32241, 7),
(9, 'Venore', 32957, 32076, 7),
(10, 'Ankrahmun', 33194, 32853, 8),
(11, 'Edron', 33217, 31814, 8),
(12, 'Farmine', 33023, 31521, 11),
(13, 'Darashia', 33213, 32454, 1),
(14, 'Liberty Bay', 32317, 32826, 7),
(15, 'Port Hope', 32594, 32745, 7),
(16, 'Svargrond', 32212, 31132, 7),
(17, 'Yalahar', 32787, 31276, 7),
(18, 'Gray Beach', 33447, 31323, 9),
(19, 'Krailos', 33657, 31665, 8),
(20, 'Rathleton', 33594, 31899, 6),
(21, 'Roshamuul', 33513, 32363, 6),
(22, 'Issavi', 33921, 31477, 5),
(24, 'Cobra Bastion', 33397, 32651, 7),
(25, 'Bounac', 32424, 32445, 7),
(26, 'Feyrist', 33490, 32221, 7),
(27, 'Gnomprona', 33517, 32856, 14),
(28, 'Marapur', 33842, 32853, 7),
(29, 'Candia', 33338, 32125, 7),
(30, 'Silvertides', 33776, 32842, 7),
(31, 'Moonfall', 33797, 32755, 5),
(32, 'Blue Valley', 33614, 31494, 7);

-- --------------------------------------------------------

--
-- Table structure for table `z_polls`
--

CREATE TABLE `z_polls` (
  `id` int(11) NOT NULL,
  `question` varchar(255) NOT NULL,
  `description` varchar(255) NOT NULL,
  `end` int(11) NOT NULL DEFAULT 0,
  `start` int(11) NOT NULL DEFAULT 0,
  `answers` int(11) NOT NULL DEFAULT 0,
  `votes_all` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

-- --------------------------------------------------------

--
-- Table structure for table `z_polls_answers`
--

CREATE TABLE `z_polls_answers` (
  `poll_id` int(11) NOT NULL,
  `answer_id` int(11) NOT NULL,
  `answer` varchar(255) NOT NULL,
  `votes` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `accounts`
--
ALTER TABLE `accounts`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `accounts_unique` (`name`);

--
-- Indexes for table `account_bans`
--
ALTER TABLE `account_bans`
  ADD PRIMARY KEY (`account_id`),
  ADD KEY `banned_by` (`banned_by`);

--
-- Indexes for table `account_ban_history`
--
ALTER TABLE `account_ban_history`
  ADD PRIMARY KEY (`id`),
  ADD KEY `account_id` (`account_id`),
  ADD KEY `banned_by` (`banned_by`);

--
-- Indexes for table `account_sessions`
--
ALTER TABLE `account_sessions`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `account_vipgrouplist`
--
ALTER TABLE `account_vipgrouplist`
  ADD UNIQUE KEY `account_vipgrouplist_unique` (`account_id`,`player_id`,`vipgroup_id`),
  ADD KEY `account_id` (`account_id`),
  ADD KEY `player_id` (`player_id`),
  ADD KEY `vipgroup_id` (`vipgroup_id`);

--
-- Indexes for table `account_vipgroups`
--
ALTER TABLE `account_vipgroups`
  ADD PRIMARY KEY (`id`),
  ADD KEY `account_vipgroups_accounts_fk` (`account_id`);

--
-- Indexes for table `account_viplist`
--
ALTER TABLE `account_viplist`
  ADD UNIQUE KEY `account_viplist_unique` (`account_id`,`player_id`),
  ADD KEY `account_id` (`account_id`),
  ADD KEY `player_id` (`player_id`);

--
-- Indexes for table `boosted_boss`
--
ALTER TABLE `boosted_boss`
  ADD PRIMARY KEY (`date`);

--
-- Indexes for table `boosted_creature`
--
ALTER TABLE `boosted_creature`
  ADD PRIMARY KEY (`date`);

--
-- Indexes for table `coins_transactions`
--
ALTER TABLE `coins_transactions`
  ADD PRIMARY KEY (`id`),
  ADD KEY `account_id` (`account_id`);

--
-- Indexes for table `daily_reward_history`
--
ALTER TABLE `daily_reward_history`
  ADD PRIMARY KEY (`id`),
  ADD KEY `player_id` (`player_id`);

--
-- Indexes for table `forge_history`
--
ALTER TABLE `forge_history`
  ADD PRIMARY KEY (`id`),
  ADD KEY `player_id` (`player_id`);

--
-- Indexes for table `global_storage`
--
ALTER TABLE `global_storage`
  ADD UNIQUE KEY `global_storage_unique` (`key`);

--
-- Indexes for table `guilds`
--
ALTER TABLE `guilds`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `guilds_name_unique` (`name`),
  ADD UNIQUE KEY `guilds_owner_unique` (`ownerid`);

--
-- Indexes for table `guildwar_kills`
--
ALTER TABLE `guildwar_kills`
  ADD PRIMARY KEY (`id`),
  ADD KEY `warid` (`warid`);

--
-- Indexes for table `guild_invites`
--
ALTER TABLE `guild_invites`
  ADD PRIMARY KEY (`player_id`,`guild_id`),
  ADD KEY `guild_id` (`guild_id`);

--
-- Indexes for table `guild_membership`
--
ALTER TABLE `guild_membership`
  ADD PRIMARY KEY (`player_id`),
  ADD KEY `guild_id` (`guild_id`),
  ADD KEY `rank_id` (`rank_id`);

--
-- Indexes for table `guild_ranks`
--
ALTER TABLE `guild_ranks`
  ADD PRIMARY KEY (`id`),
  ADD KEY `guild_id` (`guild_id`);

--
-- Indexes for table `guild_wars`
--
ALTER TABLE `guild_wars`
  ADD PRIMARY KEY (`id`),
  ADD KEY `guild1` (`guild1`),
  ADD KEY `guild2` (`guild2`);

--
-- Indexes for table `houses`
--
ALTER TABLE `houses`
  ADD PRIMARY KEY (`id`),
  ADD KEY `owner` (`owner`),
  ADD KEY `town_id` (`town_id`);

--
-- Indexes for table `house_lists`
--
ALTER TABLE `house_lists`
  ADD PRIMARY KEY (`house_id`,`listid`),
  ADD KEY `house_id_index` (`house_id`),
  ADD KEY `version` (`version`);

--
-- Indexes for table `ip_bans`
--
ALTER TABLE `ip_bans`
  ADD PRIMARY KEY (`ip`),
  ADD KEY `banned_by` (`banned_by`);

--
-- Indexes for table `kv_store`
--
ALTER TABLE `kv_store`
  ADD PRIMARY KEY (`key_name`);

--
-- Indexes for table `market_history`
--
ALTER TABLE `market_history`
  ADD PRIMARY KEY (`id`),
  ADD KEY `player_id` (`player_id`,`sale`);

--
-- Indexes for table `market_offers`
--
ALTER TABLE `market_offers`
  ADD PRIMARY KEY (`id`),
  ADD KEY `sale` (`sale`,`itemtype`),
  ADD KEY `created` (`created`),
  ADD KEY `player_id` (`player_id`);

--
-- Indexes for table `myaac_account_actions`
--
ALTER TABLE `myaac_account_actions`
  ADD KEY `account_id` (`account_id`);

--
-- Indexes for table `myaac_admin_menu`
--
ALTER TABLE `myaac_admin_menu`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_bugtracker`
--
ALTER TABLE `myaac_bugtracker`
  ADD PRIMARY KEY (`uid`);

--
-- Indexes for table `myaac_changelog`
--
ALTER TABLE `myaac_changelog`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_charbazaar`
--
ALTER TABLE `myaac_charbazaar`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_charbazaar_bid`
--
ALTER TABLE `myaac_charbazaar_bid`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_config`
--
ALTER TABLE `myaac_config`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `name` (`name`);

--
-- Indexes for table `myaac_faq`
--
ALTER TABLE `myaac_faq`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_forum`
--
ALTER TABLE `myaac_forum`
  ADD PRIMARY KEY (`id`),
  ADD KEY `section` (`section`);

--
-- Indexes for table `myaac_forum_boards`
--
ALTER TABLE `myaac_forum_boards`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_gallery`
--
ALTER TABLE `myaac_gallery`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_menu`
--
ALTER TABLE `myaac_menu`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_monsters`
--
ALTER TABLE `myaac_monsters`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_news`
--
ALTER TABLE `myaac_news`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_news_categories`
--
ALTER TABLE `myaac_news_categories`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_notepad`
--
ALTER TABLE `myaac_notepad`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_pages`
--
ALTER TABLE `myaac_pages`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `name` (`name`);

--
-- Indexes for table `myaac_polls`
--
ALTER TABLE `myaac_polls`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_spells`
--
ALTER TABLE `myaac_spells`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `name` (`name`);

--
-- Indexes for table `myaac_videos`
--
ALTER TABLE `myaac_videos`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `myaac_visitors`
--
ALTER TABLE `myaac_visitors`
  ADD UNIQUE KEY `ip` (`ip`);

--
-- Indexes for table `myaac_weapons`
--
ALTER TABLE `myaac_weapons`
  ADD PRIMARY KEY (`id`);

--
-- Indexes for table `players`
--
ALTER TABLE `players`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `players_unique` (`name`),
  ADD KEY `account_id` (`account_id`),
  ADD KEY `vocation` (`vocation`);

--
-- Indexes for table `players_online`
--
ALTER TABLE `players_online`
  ADD PRIMARY KEY (`player_id`);

--
-- Indexes for table `player_bosstiary`
--
ALTER TABLE `player_bosstiary`
  ADD KEY `player_bosstiary_players_fk` (`player_id`);

--
-- Indexes for table `player_charms`
--
ALTER TABLE `player_charms`
  ADD KEY `player_charms_players_fk` (`player_id`);

--
-- Indexes for table `player_deaths`
--
ALTER TABLE `player_deaths`
  ADD KEY `player_id` (`player_id`),
  ADD KEY `killed_by` (`killed_by`),
  ADD KEY `mostdamage_by` (`mostdamage_by`);

--
-- Indexes for table `player_depotitems`
--
ALTER TABLE `player_depotitems`
  ADD UNIQUE KEY `player_depotitems_unique` (`player_id`,`sid`);

--
-- Indexes for table `player_hirelings`
--
ALTER TABLE `player_hirelings`
  ADD PRIMARY KEY (`id`),
  ADD KEY `player_id` (`player_id`);

--
-- Indexes for table `player_inboxitems`
--
ALTER TABLE `player_inboxitems`
  ADD UNIQUE KEY `player_inboxitems_unique` (`player_id`,`sid`);

--
-- Indexes for table `player_items`
--
ALTER TABLE `player_items`
  ADD PRIMARY KEY (`player_id`,`pid`,`sid`),
  ADD KEY `player_id` (`player_id`),
  ADD KEY `sid` (`sid`);

--
-- Indexes for table `player_kills`
--
ALTER TABLE `player_kills`
  ADD KEY `player_kills_players_fk` (`player_id`);

--
-- Indexes for table `player_mounts`
--
ALTER TABLE `player_mounts`
  ADD PRIMARY KEY (`player_id`,`mount_id`);

--
-- Indexes for table `player_namelocks`
--
ALTER TABLE `player_namelocks`
  ADD UNIQUE KEY `player_namelocks_unique` (`player_id`),
  ADD KEY `namelocked_by` (`namelocked_by`);

--
-- Indexes for table `player_oldnames`
--
ALTER TABLE `player_oldnames`
  ADD PRIMARY KEY (`id`),
  ADD KEY `player_id_index` (`player_id`);

--
-- Indexes for table `player_outfits`
--
ALTER TABLE `player_outfits`
  ADD PRIMARY KEY (`player_id`,`outfit_id`);

--
-- Indexes for table `player_prey`
--
ALTER TABLE `player_prey`
  ADD PRIMARY KEY (`player_id`,`slot`);

--
-- Indexes for table `player_rewards`
--
ALTER TABLE `player_rewards`
  ADD UNIQUE KEY `player_rewards_unique` (`player_id`,`sid`);

--
-- Indexes for table `player_spells`
--
ALTER TABLE `player_spells`
  ADD PRIMARY KEY (`player_id`,`name`),
  ADD KEY `player_id` (`player_id`);

--
-- Indexes for table `player_stash`
--
ALTER TABLE `player_stash`
  ADD PRIMARY KEY (`player_id`,`item_id`);

--
-- Indexes for table `player_statements`
--
ALTER TABLE `player_statements`
  ADD PRIMARY KEY (`id`),
  ADD KEY `player_id` (`player_id`),
  ADD KEY `channel_id` (`channel_id`);

--
-- Indexes for table `player_storage`
--
ALTER TABLE `player_storage`
  ADD PRIMARY KEY (`player_id`,`key`);

--
-- Indexes for table `player_taskhunt`
--
ALTER TABLE `player_taskhunt`
  ADD PRIMARY KEY (`player_id`,`slot`);

--
-- Indexes for table `player_wheeldata`
--
ALTER TABLE `player_wheeldata`
  ADD PRIMARY KEY (`player_id`),
  ADD KEY `player_id` (`player_id`);

--
-- Indexes for table `server_config`
--
ALTER TABLE `server_config`
  ADD PRIMARY KEY (`config`);

--
-- Indexes for table `store_history`
--
ALTER TABLE `store_history`
  ADD PRIMARY KEY (`id`),
  ADD KEY `account_id` (`account_id`);

--
-- Indexes for table `tile_store`
--
ALTER TABLE `tile_store`
  ADD KEY `house_id` (`house_id`);

--
-- Indexes for table `towns`
--
ALTER TABLE `towns`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `name` (`name`);

--
-- Indexes for table `z_polls`
--
ALTER TABLE `z_polls`
  ADD PRIMARY KEY (`id`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `accounts`
--
ALTER TABLE `accounts`
  MODIFY `id` int(11) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=4;

--
-- AUTO_INCREMENT for table `account_ban_history`
--
ALTER TABLE `account_ban_history`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `account_vipgroups`
--
ALTER TABLE `account_vipgroups`
  MODIFY `id` int(11) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=10;

--
-- AUTO_INCREMENT for table `coins_transactions`
--
ALTER TABLE `coins_transactions`
  MODIFY `id` int(11) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=3;

--
-- AUTO_INCREMENT for table `daily_reward_history`
--
ALTER TABLE `daily_reward_history`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `forge_history`
--
ALTER TABLE `forge_history`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `guilds`
--
ALTER TABLE `guilds`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `guildwar_kills`
--
ALTER TABLE `guildwar_kills`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `guild_ranks`
--
ALTER TABLE `guild_ranks`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `guild_wars`
--
ALTER TABLE `guild_wars`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `houses`
--
ALTER TABLE `houses`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=3697;

--
-- AUTO_INCREMENT for table `market_history`
--
ALTER TABLE `market_history`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `market_offers`
--
ALTER TABLE `market_offers`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_admin_menu`
--
ALTER TABLE `myaac_admin_menu`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_bugtracker`
--
ALTER TABLE `myaac_bugtracker`
  MODIFY `uid` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_changelog`
--
ALTER TABLE `myaac_changelog`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=2;

--
-- AUTO_INCREMENT for table `myaac_charbazaar`
--
ALTER TABLE `myaac_charbazaar`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_charbazaar_bid`
--
ALTER TABLE `myaac_charbazaar_bid`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_config`
--
ALTER TABLE `myaac_config`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=18;

--
-- AUTO_INCREMENT for table `myaac_faq`
--
ALTER TABLE `myaac_faq`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_forum`
--
ALTER TABLE `myaac_forum`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_forum_boards`
--
ALTER TABLE `myaac_forum_boards`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=6;

--
-- AUTO_INCREMENT for table `myaac_gallery`
--
ALTER TABLE `myaac_gallery`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=2;

--
-- AUTO_INCREMENT for table `myaac_menu`
--
ALTER TABLE `myaac_menu`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=115;

--
-- AUTO_INCREMENT for table `myaac_monsters`
--
ALTER TABLE `myaac_monsters`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_news`
--
ALTER TABLE `myaac_news`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=3;

--
-- AUTO_INCREMENT for table `myaac_news_categories`
--
ALTER TABLE `myaac_news_categories`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=6;

--
-- AUTO_INCREMENT for table `myaac_notepad`
--
ALTER TABLE `myaac_notepad`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_pages`
--
ALTER TABLE `myaac_pages`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=4;

--
-- AUTO_INCREMENT for table `myaac_spells`
--
ALTER TABLE `myaac_spells`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `myaac_videos`
--
ALTER TABLE `myaac_videos`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `players`
--
ALTER TABLE `players`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=10;

--
-- AUTO_INCREMENT for table `player_hirelings`
--
ALTER TABLE `player_hirelings`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `player_oldnames`
--
ALTER TABLE `player_oldnames`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `player_statements`
--
ALTER TABLE `player_statements`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `store_history`
--
ALTER TABLE `store_history`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=3;

--
-- AUTO_INCREMENT for table `towns`
--
ALTER TABLE `towns`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=33;

--
-- AUTO_INCREMENT for table `z_polls`
--
ALTER TABLE `z_polls`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;

--
-- Constraints for dumped tables
--

--
-- Constraints for table `account_bans`
--
ALTER TABLE `account_bans`
  ADD CONSTRAINT `account_bans_account_fk` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `account_bans_player_fk` FOREIGN KEY (`banned_by`) REFERENCES `players` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `account_ban_history`
--
ALTER TABLE `account_ban_history`
  ADD CONSTRAINT `account_bans_history_account_fk` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `account_bans_history_player_fk` FOREIGN KEY (`banned_by`) REFERENCES `players` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `account_vipgrouplist`
--
ALTER TABLE `account_vipgrouplist`
  ADD CONSTRAINT `account_vipgrouplist_player_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE,
  ADD CONSTRAINT `account_vipgrouplist_vipgroup_fk` FOREIGN KEY (`vipgroup_id`) REFERENCES `account_vipgroups` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `account_vipgroups`
--
ALTER TABLE `account_vipgroups`
  ADD CONSTRAINT `account_vipgroups_accounts_fk` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `account_viplist`
--
ALTER TABLE `account_viplist`
  ADD CONSTRAINT `account_viplist_account_fk` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE,
  ADD CONSTRAINT `account_viplist_player_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `coins_transactions`
--
ALTER TABLE `coins_transactions`
  ADD CONSTRAINT `coins_transactions_account_fk` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `daily_reward_history`
--
ALTER TABLE `daily_reward_history`
  ADD CONSTRAINT `daily_reward_history_player_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `forge_history`
--
ALTER TABLE `forge_history`
  ADD CONSTRAINT `forge_history_ibfk_1` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `guilds`
--
ALTER TABLE `guilds`
  ADD CONSTRAINT `guilds_ownerid_fk` FOREIGN KEY (`ownerid`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `guildwar_kills`
--
ALTER TABLE `guildwar_kills`
  ADD CONSTRAINT `guildwar_kills_warid_fk` FOREIGN KEY (`warid`) REFERENCES `guild_wars` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `guild_invites`
--
ALTER TABLE `guild_invites`
  ADD CONSTRAINT `guild_invites_guild_fk` FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`) ON DELETE CASCADE,
  ADD CONSTRAINT `guild_invites_player_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `guild_membership`
--
ALTER TABLE `guild_membership`
  ADD CONSTRAINT `guild_membership_guild_fk` FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `guild_membership_player_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `guild_membership_rank_fk` FOREIGN KEY (`rank_id`) REFERENCES `guild_ranks` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `guild_ranks`
--
ALTER TABLE `guild_ranks`
  ADD CONSTRAINT `guild_ranks_fk` FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `house_lists`
--
ALTER TABLE `house_lists`
  ADD CONSTRAINT `houses_list_house_fk` FOREIGN KEY (`house_id`) REFERENCES `houses` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `ip_bans`
--
ALTER TABLE `ip_bans`
  ADD CONSTRAINT `ip_bans_players_fk` FOREIGN KEY (`banned_by`) REFERENCES `players` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `market_history`
--
ALTER TABLE `market_history`
  ADD CONSTRAINT `market_history_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `market_offers`
--
ALTER TABLE `market_offers`
  ADD CONSTRAINT `market_offers_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `players`
--
ALTER TABLE `players`
  ADD CONSTRAINT `players_account_fk` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_bosstiary`
--
ALTER TABLE `player_bosstiary`
  ADD CONSTRAINT `player_bosstiary_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_charms`
--
ALTER TABLE `player_charms`
  ADD CONSTRAINT `player_charms_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_deaths`
--
ALTER TABLE `player_deaths`
  ADD CONSTRAINT `player_deaths_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_depotitems`
--
ALTER TABLE `player_depotitems`
  ADD CONSTRAINT `player_depotitems_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_hirelings`
--
ALTER TABLE `player_hirelings`
  ADD CONSTRAINT `player_hirelings_ibfk_1` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_inboxitems`
--
ALTER TABLE `player_inboxitems`
  ADD CONSTRAINT `player_inboxitems_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_items`
--
ALTER TABLE `player_items`
  ADD CONSTRAINT `player_items_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_kills`
--
ALTER TABLE `player_kills`
  ADD CONSTRAINT `player_kills_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_mounts`
--
ALTER TABLE `player_mounts`
  ADD CONSTRAINT `player_mounts_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_namelocks`
--
ALTER TABLE `player_namelocks`
  ADD CONSTRAINT `player_namelocks_players2_fk` FOREIGN KEY (`namelocked_by`) REFERENCES `players` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  ADD CONSTRAINT `player_namelocks_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

--
-- Constraints for table `player_outfits`
--
ALTER TABLE `player_outfits`
  ADD CONSTRAINT `player_outfits_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_prey`
--
ALTER TABLE `player_prey`
  ADD CONSTRAINT `player_prey_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_rewards`
--
ALTER TABLE `player_rewards`
  ADD CONSTRAINT `player_rewards_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_spells`
--
ALTER TABLE `player_spells`
  ADD CONSTRAINT `player_spells_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_stash`
--
ALTER TABLE `player_stash`
  ADD CONSTRAINT `player_stash_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_statements`
--
ALTER TABLE `player_statements`
  ADD CONSTRAINT `player_statements_ibfk_1` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_storage`
--
ALTER TABLE `player_storage`
  ADD CONSTRAINT `player_storage_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_taskhunt`
--
ALTER TABLE `player_taskhunt`
  ADD CONSTRAINT `player_taskhunt_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `player_wheeldata`
--
ALTER TABLE `player_wheeldata`
  ADD CONSTRAINT `player_wheeldata_players_fk` FOREIGN KEY (`player_id`) REFERENCES `players` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `store_history`
--
ALTER TABLE `store_history`
  ADD CONSTRAINT `store_history_account_fk` FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE;

--
-- Constraints for table `tile_store`
--
ALTER TABLE `tile_store`
  ADD CONSTRAINT `tile_store_account_fk` FOREIGN KEY (`house_id`) REFERENCES `houses` (`id`) ON DELETE CASCADE;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
