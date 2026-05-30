# Flow Graph

_Generated 2026-05-30 — do not edit by hand. Re-run `make flow-render`._

This document is generated from the YAML node shards in `flow_graph/nodes/`. Each section corresponds to one shard file. Nodes are C++ functions/methods; edges encode the call paths (static calls extracted from source, dynamic/curated calls manually annotated).

## Boot Sequence

Entrypoint chain from `main` through initial loader:

1. `main` — `src/main.cpp`
2. `startServer` — `src/otserv.cpp`
3. `mainLoader` — `src/otserv.cpp`

- **Root:** `main`
- **Orphan policy:** `entrypoint-only`

## Node Shards

### `actions.h` (header declarations)

4 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_ACTIONS_H` | 0 | — |
| `Action` | 0 | — |
| `Action::Action` | 0 | — |
| `Action::canExecuteAction` | 0 | — |

### `actions`

16 node(s), 28 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Actions::clearMap` | 1 | `Actions::clearMap` (static) |
| `Actions::clear` | 1 | `Actions::clear` (static) |
| `Actions::getScriptInterface` | 1 | `Actions::getScriptInterface` (static) |
| `Actions::getEvent` | 1 | `Actions::getEvent` (static) |
| `Actions::registerLuaEvent` | 1 | `Actions::registerLuaEvent` (static) |
| `Actions::canUse` | 0 | — |
| `Actions::canUse` | 1 | `Actions::canUse` (static) |
| `Actions::canUseFar` | 2 | `Actions::canUseFar` (static); `Game::canThrowObjectTo` (static) |
| `Actions::getAction` | 2 | `Actions::getAction` (static); `Spells::getRuneSpell` (static) |
| `Actions::internalUseItem` | 2 | `Actions::internalUseItem` (static); `Game::sendOfflineTrainingDialog` (static) |
| `showUseHotkeyMessage` | 0 | — |
| `Actions::useItem` | 2 | `Actions::useItem` (static); `Game::internalCreatureSay` (static) |
| `Actions::useItemEx` | 1 | `Actions::useItemEx` (static) |
| `Action::canExecuteAction` | 3 | `Action::canExecuteAction` (static); `Actions::canUse` (static); `Actions::canUseFar` (static) |
| `Action::getTarget` | 2 | `Action::getTarget` (static); `Game::internalGetThing` (static) |
| `Action::executeUse` | 8 | `Action::executeUse` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::pushPosition` (static); `tfs::lua::pushThing` (static); … +3 more |

### `ban.h` (header declarations)

8 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `IOBan` | 0 | — |
| `IOBan::BanInfo` | 0 | — |
| `IOBan::BanInfo::bannedBy` | 0 | — |
| `IOBan::BanInfo::reason` | 0 | — |
| `IOBan::BanInfo::expiresAt` | 0 | — |
| `IOBan::getAccountBanInfo` | 0 | — |
| `IOBan::getIpBanInfo` | 0 | — |
| `IOBan::isPlayerNamelocked` | 0 | — |

### `ban`

3 node(s), 3 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `IOBan::getAccountBanInfo` | 1 | `Database::getInstance` (static) |
| `IOBan::getIpBanInfo` | 1 | `Database::getInstance` (static) |
| `IOBan::isPlayerNamelocked` | 1 | `Database::getInstance` (static) |

### `base64.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `TFS_BASE64_H` | 0 | — |

### `baseevents.h` (header declarations)

23 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_BASEEVENTS_H` | 0 | — |
| `Event` | 0 | — |
| `Event::Event` | 0 | — |
| `Event::Event` | 0 | — |
| `Event::configureEvent` | 0 | — |
| `Event::checkScript` | 0 | — |
| `Event::loadScript` | 0 | — |
| `Event::loadCallback` | 0 | — |
| `Event::getScriptEventName` | 0 | — |
| `BaseEvents` | 0 | — |
| `BaseEvents::BaseEvents` | 0 | — |
| `BaseEvents::BaseEvents` | 0 | — |
| `BaseEvents::loadFromXml` | 0 | — |
| `BaseEvents::reload` | 0 | — |
| `BaseEvents::reInitState` | 0 | — |
| `BaseEvents::getScriptInterface` | 0 | — |
| `BaseEvents::getScriptBaseName` | 0 | — |
| `BaseEvents::getEvent` | 0 | — |
| `BaseEvents::registerEvent` | 0 | — |
| `BaseEvents::clear` | 0 | — |
| `CallBack` | 0 | — |
| `CallBack::CallBack` | 0 | — |
| `CallBack::loadCallBack` | 0 | — |

### `baseevents`

7 node(s), 8 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `BaseEvents::loadFromXml` | 1 | `BaseEvents::loadFromXml` (static) |
| `BaseEvents::reload` | 1 | `BaseEvents::reload` (static) |
| `BaseEvents::reInitState` | 1 | `BaseEvents::reInitState` (static) |
| `Event::checkScript` | 2 | `Event::checkScript` (static); `LuaEnvironment::getTestInterface` (static) |
| `Event::loadScript` | 1 | `Event::loadScript` (static) |
| `Event::loadCallback` | 1 | `Event::loadCallback` (static) |
| `CallBack::loadCallBack` | 1 | `CallBack::loadCallBack` (static) |

### `bed.h` (header declarations)

11 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `BedItem` | 0 | — |
| `BedItem::house` | 0 | — |
| `BedItem::sleepStart` | 0 | — |
| `BedItem::sleeperGUID` | 0 | — |
| `BedItem::canRemove` | 0 | — |
| `BedItem::getBed (const)` | 0 | — |
| `BedItem::getBed (non-const)` | 0 | — |
| `BedItem::getHouse` | 0 | — |
| `BedItem::getSleeper` | 0 | — |
| `BedItem::setHouse` | 0 | — |
| `FS_BED_H` | 0 | — |

### `bed`

23 node(s), 31 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `BedItem::BedItem` | 1 | `BedItem::BedItem` (static) |
| `BedItem::canUse` | 0 | — |
| `BedItem::getNextBedItem` | 0 | — |
| `BedItem::internalRemoveSleeper` | 0 | — |
| `BedItem::internalSetSleeper` | 0 | — |
| `BedItem::readAttr` | 0 | — |
| `BedItem::regeneratePlayer` | 0 | — |
| `BedItem::serializeAttr` | 0 | — |
| `BedItem::sleep` | 0 | — |
| `BedItem::trySleep` | 0 | — |
| `BedItem::updateAppearance` | 0 | — |
| `BedItem::wakeUp` | 0 | — |
| `BedItem::readAttr` | 4 | `BedItem::readAttr` (static); `Game::setBedSleeper` (static); `IOLoginData::getNameByGuid` (static); `Item::readAttr` (static) |
| `BedItem::serializeAttr` | 1 | `BedItem::serializeAttr` (static) |
| `BedItem::getNextBedItem` | 1 | `BedItem::getNextBedItem` (static) |
| `BedItem::canUse` | 2 | `BedItem::canUse` (static); `IOLoginData::loadPlayerById` (static) |
| `BedItem::trySleep` | 2 | `BedItem::trySleep` (static); `Game::addMagicEffect` (static) |
| `BedItem::sleep` | 9 | `BedItem::sleep` (static); `BedItem::wakeUp` (static); `Game::addCreatureHealth` (static); `Game::addMagicEffect` (static); `Game::kickPlayer` (static); … +4 more |
| `BedItem::wakeUp` | 6 | `BedItem::regeneratePlayer` (static); `BedItem::wakeUp` (static); `Game::addCreatureHealth` (static); `Game::removeBedSleeper` (static); `IOLoginData::loadPlayerById` (static); … +1 more |
| `BedItem::regeneratePlayer` | 1 | `BedItem::regeneratePlayer` (static) |
| `BedItem::updateAppearance` | 2 | `BedItem::updateAppearance` (static); `Game::transformItem` (static) |
| `BedItem::internalSetSleeper` | 1 | `BedItem::internalSetSleeper` (static) |
| `BedItem::internalRemoveSleeper` | 1 | `BedItem::internalRemoveSleeper` (static) |

### `chat.h` (header declarations)

65 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Chat` | 0 | — |
| `ChatChannel` | 0 | — |
| `PrivateChatChannel` | 0 | — |
| `ChatChannel::ChatChannel (default)` | 0 | — |
| `ChatChannel::ChatChannel (id, name)` | 0 | — |
| `PrivateChatChannel::PrivateChatChannel` | 0 | — |
| `ChatChannel::~ChatChannel` | 0 | — |
| `Chat::dummyPrivate` | 0 | — |
| `Chat::guildChannels` | 0 | — |
| `Chat::normalChannels` | 0 | — |
| `Chat::partyChannels` | 0 | — |
| `Chat::privateChannels` | 0 | — |
| `Chat::scriptInterface` | 0 | — |
| `ChatChannel::canJoinEvent` | 0 | — |
| `ChatChannel::id` | 0 | — |
| `ChatChannel::name` | 0 | — |
| `ChatChannel::onJoinEvent` | 0 | — |
| `ChatChannel::onLeaveEvent` | 0 | — |
| `ChatChannel::onSpeakEvent` | 0 | — |
| `ChatChannel::publicChannel` | 0 | — |
| `ChatChannel::users` | 0 | — |
| `PrivateChatChannel::invites` | 0 | — |
| `PrivateChatChannel::owner` | 0 | — |
| `Chat::getScriptInterface` | 0 | — |
| `ChatChannel::getId` | 0 | — |
| `ChatChannel::getInvitedUsers` | 0 | — |
| `ChatChannel::getName` | 0 | — |
| `ChatChannel::getOwner` | 0 | — |
| `ChatChannel::getUsers` | 0 | — |
| `ChatChannel::isPublicChannel` | 0 | — |
| `PrivateChatChannel::getInvitedUsers` | 0 | — |
| `PrivateChatChannel::getOwner` | 0 | — |
| `PrivateChatChannel::setOwner` | 0 | — |
| `ChannelList` | 0 | — |
| `InvitedMap` | 0 | — |
| `UsersMap` | 0 | — |
| `FS_CHAT_H` | 0 | — |
| `ChatChannel` | 0 | — |
| `ChatChannel::ChatChannel` | 0 | — |
| `ChatChannel::ChatChannel` | 0 | — |
| `ChatChannel::ChatChannel` | 0 | — |
| `ChatChannel::addUser` | 0 | — |
| `ChatChannel::removeUser` | 0 | — |
| `ChatChannel::hasUser` | 0 | — |
| `ChatChannel::talk` | 0 | — |
| `ChatChannel::sendToAll` | 0 | — |
| `ChatChannel::executeOnJoinEvent` | 0 | — |
| `ChatChannel::executeCanJoinEvent` | 0 | — |
| `ChatChannel::executeOnLeaveEvent` | 0 | — |
| `ChatChannel::executeOnSpeakEvent` | 0 | — |
| `Chat` | 0 | — |
| `Chat::Chat` | 0 | — |
| `Chat::Chat` | 0 | — |
| `Chat::load` | 0 | — |
| `Chat::createChannel` | 0 | — |
| `Chat::deleteChannel` | 0 | — |
| `Chat::addUserToChannel` | 0 | — |
| `Chat::removeUserFromChannel` | 0 | — |
| `Chat::removeUserFromAllChannels` | 0 | — |
| `Chat::talkToChannel` | 0 | — |
| `Chat::getChannelList` | 0 | — |
| `Chat::getChannel` | 0 | — |
| `Chat::getChannelById` | 0 | — |
| `Chat::getGuildChannelById` | 0 | — |
| `Chat::getPrivateChannel` | 0 | — |

### `chat`

53 node(s), 53 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Chat::Chat` | 1 | `PrivateChatChannel::isInvited` (static) |
| `Chat::addUserToChannel` | 0 | — |
| `Chat::createChannel` | 0 | — |
| `Chat::deleteChannel` | 0 | — |
| `Chat::getChannel` | 0 | — |
| `Chat::getChannelById` | 0 | — |
| `Chat::getChannelList` | 0 | — |
| `Chat::getGuildChannelById` | 0 | — |
| `Chat::getPrivateChannel` | 0 | — |
| `Chat::load` | 0 | — |
| `Chat::removeUserFromAllChannels` | 0 | — |
| `Chat::removeUserFromChannel` | 0 | — |
| `Chat::talkToChannel` | 0 | — |
| `ChatChannel::addUser` | 0 | — |
| `ChatChannel::executeCanJoinEvent` | 0 | — |
| `ChatChannel::executeOnJoinEvent` | 0 | — |
| `ChatChannel::executeOnLeaveEvent` | 0 | — |
| `ChatChannel::executeOnSpeakEvent` | 0 | — |
| `ChatChannel::hasUser` | 0 | — |
| `ChatChannel::removeUser` | 0 | — |
| `ChatChannel::sendToAll` | 0 | — |
| `ChatChannel::talk` | 0 | — |
| `PrivateChatChannel::closeChannel` | 0 | — |
| `PrivateChatChannel::excludePlayer` | 0 | — |
| `PrivateChatChannel::invitePlayer` | 0 | — |
| `PrivateChatChannel::isInvited` | 0 | — |
| `PrivateChatChannel::removeInvite` | 0 | — |
| `PrivateChatChannel::isInvited` | 1 | `PrivateChatChannel::isInvited` (static) |
| `PrivateChatChannel::removeInvite` | 1 | `PrivateChatChannel::removeInvite` (static) |
| `PrivateChatChannel::invitePlayer` | 1 | `PrivateChatChannel::invitePlayer` (static) |
| `PrivateChatChannel::excludePlayer` | 1 | `PrivateChatChannel::excludePlayer` (static) |
| `PrivateChatChannel::closeChannel` | 1 | `PrivateChatChannel::closeChannel` (static) |
| `ChatChannel::addUser` | 2 | `ChatChannel::addUser` (static); `Game::sendGuildMotd` (static) |
| `ChatChannel::removeUser` | 1 | `ChatChannel::removeUser` (static) |
| `ChatChannel::hasUser` | 1 | `ChatChannel::hasUser` (static) |
| `ChatChannel::sendToAll` | 1 | `ChatChannel::sendToAll` (static) |
| `ChatChannel::talk` | 1 | `ChatChannel::talk` (static) |
| `ChatChannel::executeCanJoinEvent` | 6 | `ChatChannel::executeCanJoinEvent` (static); `Chat::getScriptInterface` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); … +1 more |
| `ChatChannel::executeOnJoinEvent` | 6 | `ChatChannel::executeOnJoinEvent` (static); `Chat::getScriptInterface` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); … +1 more |
| `ChatChannel::executeOnLeaveEvent` | 6 | `ChatChannel::executeOnLeaveEvent` (static); `Chat::getScriptInterface` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); … +1 more |
| `ChatChannel::executeOnSpeakEvent` | 11 | `ChatChannel::executeOnSpeakEvent` (static); `Chat::getScriptInterface` (static); `tfs::lua::getBoolean` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::popString` (static); … +6 more |
| `Chat::load` | 1 | `Chat::load` (static) |
| `Chat::createChannel` | 1 | `Chat::createChannel` (static) |
| `Chat::deleteChannel` | 1 | `Chat::deleteChannel` (static) |
| `Chat::addUserToChannel` | 1 | `Chat::addUserToChannel` (static) |
| `Chat::removeUserFromChannel` | 1 | `Chat::removeUserFromChannel` (static) |
| `Chat::removeUserFromAllChannels` | 1 | `Chat::removeUserFromAllChannels` (static) |
| `Chat::talkToChannel` | 1 | `Chat::talkToChannel` (static) |
| `Chat::getChannelList` | 1 | `Chat::getChannelList` (static) |
| `Chat::getChannel` | 1 | `Chat::getChannel` (static) |
| `Chat::getGuildChannelById` | 1 | `Chat::getGuildChannelById` (static) |
| `Chat::getChannelById` | 1 | `Chat::getChannelById` (static) |
| `Chat::getPrivateChannel` | 1 | `Chat::getPrivateChannel` (static) |

### `combat.h` (header declarations)

81 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ValueCallback` | 0 | — |
| `ValueCallback::ValueCallback` | 0 | — |
| `ValueCallback::getMinMaxValues` | 0 | — |
| `ValueCallback::type` | 0 | — |
| `TileCallback` | 0 | — |
| `TileCallback::onTileCombat` | 0 | — |
| `TargetCallback` | 0 | — |
| `TargetCallback::onTargetCombat` | 0 | — |
| `CombatParams` | 0 | — |
| `CombatParams::conditionList` | 0 | — |
| `CombatParams::valueCallback` | 0 | — |
| `CombatParams::tileCallback` | 0 | — |
| `CombatParams::targetCallback` | 0 | — |
| `CombatParams::itemId` | 0 | — |
| `CombatParams::dispelType` | 0 | — |
| `CombatParams::combatType` | 0 | — |
| `CombatParams::origin` | 0 | — |
| `CombatParams::impactEffect` | 0 | — |
| `CombatParams::distanceEffect` | 0 | — |
| `CombatParams::blockedByArmor` | 0 | — |
| `CombatParams::blockedByShield` | 0 | — |
| `CombatParams::targetCasterOrTopMost` | 0 | — |
| `CombatParams::aggressive` | 0 | — |
| `CombatParams::useCharges` | 0 | — |
| `CombatParams::ignoreResistances` | 0 | — |
| `AreaCombat` | 0 | — |
| `AreaCombat::setupArea(vector,rows)` | 0 | — |
| `AreaCombat::setupArea(length,spread)` | 0 | — |
| `AreaCombat::setupArea(radius)` | 0 | — |
| `AreaCombat::setupAreaRing` | 0 | — |
| `AreaCombat::setupExtArea` | 0 | — |
| `AreaCombat::getArea` | 0 | — |
| `AreaCombat::areas` | 0 | — |
| `AreaCombat::hasExtArea` | 0 | — |
| `Combat` | 0 | — |
| `Combat::Combat` | 0 | — |
| `Combat::Combat(const Combat&)` | 0 | — |
| `Combat::operator=` | 0 | — |
| `Combat::isInPvpZone` | 0 | — |
| `Combat::isProtected` | 0 | — |
| `Combat::isPlayerCombat` | 0 | — |
| `Combat::ConditionToDamageType` | 0 | — |
| `Combat::DamageToConditionType` | 0 | — |
| `Combat::canTargetCreature` | 0 | — |
| `Combat::canDoCombat(tile)` | 0 | — |
| `Combat::canDoCombat(target)` | 0 | — |
| `Combat::postCombatEffects` | 0 | — |
| `Combat::addDistanceEffect` | 0 | — |
| `Combat::doCombat(target)` | 0 | — |
| `Combat::doCombat(position)` | 0 | — |
| `Combat::doTargetCombat` | 0 | — |
| `Combat::doAreaCombat` | 0 | — |
| `Combat::setCallback` | 0 | — |
| `Combat::getCallback` | 0 | — |
| `Combat::setParam` | 0 | — |
| `Combat::getParam` | 0 | — |
| `Combat::setArea` | 0 | — |
| `Combat::hasArea` | 0 | — |
| `Combat::addCondition` | 0 | — |
| `Combat::clearConditions` | 0 | — |
| `Combat::setPlayerCombatValues` | 0 | — |
| `Combat::postCombatEffects(pos)` | 0 | — |
| `Combat::setOrigin` | 0 | — |
| `Combat::combatTileEffects` | 0 | — |
| `Combat::getCombatDamage` | 0 | — |
| `Combat::params` | 0 | — |
| `Combat::formulaType` | 0 | — |
| `Combat::mina` | 0 | — |
| `Combat::minb` | 0 | — |
| `Combat::maxa` | 0 | — |
| `Combat::maxb` | 0 | — |
| `Combat::area` | 0 | — |
| `MagicField` | 0 | — |
| `MagicField::MagicField` | 0 | — |
| `MagicField::getMagicField` | 0 | — |
| `MagicField::getMagicField const` | 0 | — |
| `MagicField::isReplaceable` | 0 | — |
| `MagicField::getCombatType` | 0 | — |
| `MagicField::getDamage` | 0 | — |
| `MagicField::onStepInField` | 0 | — |
| `MagicField::createTime` | 0 | — |

### `combat`

34 node(s), 67 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `getList` | 1 | `Game::isSightClear` (static) |
| `getCombatArea` | 0 | — |
| `Combat::getCombatDamage` | 2 | `Combat::getCombatDamage` (static); `Weapons::getWeapon` (static) |
| `Combat::ConditionToDamageType` | 1 | `Combat::ConditionToDamageType` (static) |
| `Combat::DamageToConditionType` | 1 | `Combat::DamageToConditionType` (static) |
| `Combat::isPlayerCombat` | 1 | `Combat::isPlayerCombat` (static) |
| `Combat::canTargetCreature` | 3 | `Combat::canDoCombat` (static); `Combat::canTargetCreature` (static); `Combat::isInPvpZone` (static) |
| `Combat::canDoCombat` | 0 | — |
| `Combat::isInPvpZone` | 1 | `Combat::isInPvpZone` (static) |
| `Combat::isProtected` | 1 | `Combat::isProtected` (static) |
| `Combat::canDoCombat` | 2 | `Combat::canDoCombat` (static); `tfs::events::creature::onTargetCombat` (static) |
| `Combat::setPlayerCombatValues` | 1 | `Combat::setPlayerCombatValues` (static) |
| `Combat::setParam` | 1 | `Combat::setParam` (static) |
| `Combat::getParam` | 1 | `Combat::getParam` (static) |
| `Combat::setArea` | 1 | `Combat::setArea` (static) |
| `Combat::setCallback` | 1 | `Combat::setCallback` (static) |
| `Combat::getCallback` | 1 | `Combat::getCallback` (static) |
| `Combat::combatTileEffects` | 5 | `Combat::combatTileEffects` (static); `Game::addMagicEffect` (static); `Game::internalAddItem` (static); `Game::startDecay` (static); `Item::CreateItem` (static) |
| `Combat::postCombatEffects` | 1 | `Combat::postCombatEffects` (static) |
| `Combat::addDistanceEffect` | 2 | `Combat::addDistanceEffect` (static); `Game::addDistanceEffect` (static) |
| `Combat::doCombat` | 0 | — |
| `Combat::doCombat` | 2 | `Combat::canDoCombat` (static); `Combat::doCombat` (static) |
| `Combat::doTargetCombat` | 5 | `Combat::doTargetCombat` (static); `Game::addMagicEffect` (static); `Game::combatBlockHit` (static); `Game::combatChangeHealth` (static); `Game::combatChangeMana` (static) |
| `Combat::doAreaCombat` | 1 | `Combat::doAreaCombat` (static) |
| `ValueCallback::getMinMaxValues` | 8 | `ValueCallback::getMinMaxValues` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::popString` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::resetScriptEnv` (static); … +3 more |
| `TileCallback::onTileCombat` | 7 | `TileCallback::onTileCombat` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushPosition` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::resetScriptEnv` (static); … +2 more |
| `TargetCallback::onTargetCombat` | 7 | `TargetCallback::onTargetCombat` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::popString` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::resetScriptEnv` (static); … +2 more |
| `AreaCombat::getArea` | 1 | `AreaCombat::getArea` (static) |
| `AreaCombat::setupArea` | 0 | — |
| `AreaCombat::setupArea` | 0 | — |
| `AreaCombat::setupArea` | 1 | `AreaCombat::setupArea` (static) |
| `AreaCombat::setupAreaRing` | 1 | `AreaCombat::setupAreaRing` (static) |
| `AreaCombat::setupExtArea` | 1 | `AreaCombat::setupExtArea` (static) |
| `MagicField::onStepInField` | 6 | `Combat::isProtected` (static); `MagicField::onStepInField` (static); `Game::addMagicEffect` (static); `Game::getCreatureByID` (static); `Game::getPlayerByID` (static); … +1 more |

### `condition.h` (header declarations)

60 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_CONDITION_H` | 0 | — |
| `ConditionAttr_t` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_TYPE` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_ID` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_TICKS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_HEALTHTICKS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_HEALTHGAIN` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_MANATICKS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_MANAGAIN` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_DELAYED` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_OWNER` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_INTERVALDATA` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_SPEEDDELTA` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_FORMULA_MINA` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_FORMULA_MINB` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_FORMULA_MAXA` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_FORMULA_MAXB` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_LIGHTCOLOR` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_LIGHTLEVEL` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_LIGHTTICKS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_LIGHTINTERVAL` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_SOULTICKS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_SOULGAIN` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_SKILLS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_STATS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_OUTFIT` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_PERIODDAMAGE` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_ISBUFF` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_SUBID` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_ISAGGRESSIVE` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_DISABLEDEFENSE` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_SPECIALSKILLS` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_MANASHIELD_BREAKABLE_MANA` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_MANASHIELD_BREAKABLE_MAXMANA` | 0 | — |
| `ConditionAttr_t::CONDITIONATTR_END` | 0 | — |
| `IntervalInfo` | 0 | — |
| `Condition` | 0 | — |
| `Condition::Condition` | 0 | — |
| `Condition::Condition` | 0 | — |
| `Condition::startCondition` | 0 | — |
| `Condition::executeCondition` | 0 | — |
| `Condition::endCondition` | 0 | — |
| `Condition::addCondition` | 0 | — |
| `Condition::getIcons` | 0 | — |
| `Condition::clone` | 0 | — |
| `Condition::setTicks` | 0 | — |
| `Condition::createCondition` | 0 | — |
| `Condition::setParam` | 0 | — |
| `Condition::getParam` | 0 | — |
| `Condition::unserialize` | 0 | — |
| `Condition::serialize` | 0 | — |
| `Condition::unserializeProp` | 0 | — |
| `Condition::isPersistent` | 0 | — |
| `Condition::updateCondition` | 0 | — |
| `ConditionGeneric` | 0 | — |
| `ConditionGeneric::startCondition` | 0 | — |
| `ConditionGeneric::executeCondition` | 0 | — |
| `ConditionGeneric::endCondition` | 0 | — |
| `ConditionGeneric::addCondition` | 0 | — |
| `ConditionGeneric::getIcons` | 0 | — |

### `condition`

103 node(s), 168 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Condition::setParam` | 1 | `Condition::setParam` (static) |
| `Condition::getParam` | 1 | `Condition::getParam` (static) |
| `Condition::unserialize` | 1 | `Condition::unserialize` (static) |
| `Condition::unserializeProp` | 1 | `Condition::unserializeProp` (static) |
| `Condition::serialize` | 1 | `Condition::serialize` (static) |
| `Condition::setTicks` | 1 | `Condition::setTicks` (static) |
| `Condition::executeCondition` | 1 | `Condition::executeCondition` (static) |
| `Condition::createCondition` | 0 | — |
| `Condition::createCondition` | 1 | `Condition::createCondition` (static) |
| `Condition::startCondition` | 1 | `Condition::startCondition` (static) |
| `Condition::isPersistent` | 1 | `Condition::isPersistent` (static) |
| `Condition::getIcons` | 1 | `Condition::getIcons` (static) |
| `Condition::updateCondition` | 1 | `Condition::updateCondition` (static) |
| `ConditionGeneric::startCondition` | 2 | `Condition::startCondition` (static); `ConditionGeneric::startCondition` (static) |
| `ConditionGeneric::executeCondition` | 2 | `Condition::executeCondition` (static); `ConditionGeneric::executeCondition` (static) |
| `ConditionGeneric::endCondition` | 1 | `ConditionGeneric::endCondition` (static) |
| `ConditionGeneric::addCondition` | 1 | `ConditionGeneric::addCondition` (static) |
| `ConditionGeneric::getIcons` | 2 | `Condition::getIcons` (static); `ConditionGeneric::getIcons` (static) |
| `ConditionAttributes::addCondition` | 1 | `ConditionAttributes::addCondition` (static) |
| `ConditionAttributes::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionAttributes::unserializeProp` (static) |
| `ConditionAttributes::serialize` | 2 | `Condition::serialize` (static); `ConditionAttributes::serialize` (static) |
| `ConditionAttributes::startCondition` | 2 | `Condition::startCondition` (static); `ConditionAttributes::startCondition` (static) |
| `ConditionAttributes::updatePercentStats` | 1 | `ConditionAttributes::updatePercentStats` (static) |
| `ConditionAttributes::updateStats` | 1 | `ConditionAttributes::updateStats` (static) |
| `ConditionAttributes::updatePercentSkills` | 1 | `ConditionAttributes::updatePercentSkills` (static) |
| `ConditionAttributes::updateSkills` | 1 | `ConditionAttributes::updateSkills` (static) |
| `ConditionAttributes::executeCondition` | 2 | `ConditionAttributes::executeCondition` (static); `ConditionGeneric::executeCondition` (static) |
| `ConditionAttributes::endCondition` | 1 | `ConditionAttributes::endCondition` (static) |
| `ConditionAttributes::setParam` | 1 | `ConditionAttributes::setParam` (static) |
| `ConditionAttributes::getParam` | 1 | `ConditionAttributes::getParam` (static) |
| `ConditionRegeneration::addCondition` | 1 | `ConditionRegeneration::addCondition` (static) |
| `ConditionRegeneration::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionRegeneration::unserializeProp` (static) |
| `ConditionRegeneration::serialize` | 2 | `Condition::serialize` (static); `ConditionRegeneration::serialize` (static) |
| `ConditionRegeneration::executeCondition` | 2 | `ConditionGeneric::executeCondition` (static); `ConditionRegeneration::executeCondition` (static) |
| `ConditionRegeneration::setParam` | 1 | `ConditionRegeneration::setParam` (static) |
| `ConditionRegeneration::getParam` | 1 | `ConditionRegeneration::getParam` (static) |
| `ConditionSoul::addCondition` | 1 | `ConditionSoul::addCondition` (static) |
| `ConditionSoul::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionSoul::unserializeProp` (static) |
| `ConditionSoul::serialize` | 2 | `Condition::serialize` (static); `ConditionSoul::serialize` (static) |
| `ConditionSoul::executeCondition` | 2 | `ConditionGeneric::executeCondition` (static); `ConditionSoul::executeCondition` (static) |
| `ConditionSoul::setParam` | 1 | `ConditionSoul::setParam` (static) |
| `ConditionSoul::getParam` | 1 | `ConditionSoul::getParam` (static) |
| `ConditionDamage::setParam` | 2 | `Condition::setParam` (static); `ConditionDamage::setParam` (static) |
| `ConditionDamage::getParam` | 2 | `Condition::getParam` (static); `ConditionDamage::getParam` (static) |
| `ConditionDamage::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionDamage::unserializeProp` (static) |
| `ConditionDamage::serialize` | 2 | `Condition::serialize` (static); `ConditionDamage::serialize` (static) |
| `ConditionDamage::updateCondition` | 1 | `ConditionDamage::updateCondition` (static) |
| `ConditionDamage::addDamage` | 1 | `ConditionDamage::addDamage` (static) |
| `ConditionDamage::init` | 2 | `ConditionDamage::generateDamageList` (static); `ConditionDamage::init` (static) |
| `ConditionDamage::startCondition` | 2 | `Condition::startCondition` (static); `ConditionDamage::startCondition` (static) |
| `ConditionDamage::executeCondition` | 2 | `Condition::executeCondition` (static); `ConditionDamage::executeCondition` (static) |
| `ConditionDamage::getNextDamage` | 1 | `ConditionDamage::getNextDamage` (static) |
| `ConditionDamage::doDamage` | 7 | `Combat::ConditionToDamageType` (static); `Combat::canDoCombat` (static); `ConditionDamage::doDamage` (static); `Game::addMagicEffect` (static); `Game::combatBlockHit` (static); … +2 more |
| `ConditionDamage::endCondition` | 1 | `ConditionDamage::endCondition` (static) |
| `ConditionDamage::addCondition` | 1 | `ConditionDamage::addCondition` (static) |
| `ConditionDamage::getTotalDamage` | 1 | `ConditionDamage::getTotalDamage` (static) |
| `ConditionDamage::getIcons` | 2 | `Condition::getIcons` (static); `ConditionDamage::getIcons` (static) |
| `ConditionDamage::generateDamageList` | 1 | `ConditionDamage::generateDamageList` (static) |
| `ConditionSpeed::setFormulaVars` | 1 | `ConditionSpeed::setFormulaVars` (static) |
| `ConditionSpeed::setParam` | 2 | `Condition::setParam` (static); `ConditionSpeed::setParam` (static) |
| `ConditionSpeed::getParam` | 2 | `Condition::getParam` (static); `ConditionSpeed::getParam` (static) |
| `ConditionSpeed::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionSpeed::unserializeProp` (static) |
| `ConditionSpeed::serialize` | 2 | `Condition::serialize` (static); `ConditionSpeed::serialize` (static) |
| `ConditionSpeed::startCondition` | 3 | `Condition::startCondition` (static); `ConditionSpeed::startCondition` (static); `Game::changeSpeed` (static) |
| `ConditionSpeed::executeCondition` | 2 | `Condition::executeCondition` (static); `ConditionSpeed::executeCondition` (static) |
| `ConditionSpeed::endCondition` | 2 | `ConditionSpeed::endCondition` (static); `Game::changeSpeed` (static) |
| `ConditionSpeed::addCondition` | 2 | `ConditionSpeed::addCondition` (static); `Game::changeSpeed` (static) |
| `ConditionSpeed::getIcons` | 2 | `Condition::getIcons` (static); `ConditionSpeed::getIcons` (static) |
| `ConditionInvisible::startCondition` | 3 | `Condition::startCondition` (static); `ConditionInvisible::startCondition` (static); `Game::internalCreatureChangeVisible` (static) |
| `ConditionInvisible::endCondition` | 2 | `ConditionInvisible::endCondition` (static); `Game::internalCreatureChangeVisible` (static) |
| `ConditionOutfit::setOutfit` | 1 | `ConditionOutfit::setOutfit` (static) |
| `ConditionOutfit::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionOutfit::unserializeProp` (static) |
| `ConditionOutfit::serialize` | 2 | `Condition::serialize` (static); `ConditionOutfit::serialize` (static) |
| `ConditionOutfit::startCondition` | 3 | `Condition::startCondition` (static); `ConditionOutfit::startCondition` (static); `Game::internalCreatureChangeOutfit` (static) |
| `ConditionOutfit::executeCondition` | 2 | `Condition::executeCondition` (static); `ConditionOutfit::executeCondition` (static) |
| `ConditionOutfit::endCondition` | 2 | `ConditionOutfit::endCondition` (static); `Game::internalCreatureChangeOutfit` (static) |
| `ConditionOutfit::addCondition` | 2 | `ConditionOutfit::addCondition` (static); `Game::internalCreatureChangeOutfit` (static) |
| `ConditionLight::startCondition` | 3 | `Condition::startCondition` (static); `ConditionLight::startCondition` (static); `Game::changeLight` (static) |
| `ConditionLight::executeCondition` | 3 | `Condition::executeCondition` (static); `ConditionLight::executeCondition` (static); `Game::changeLight` (static) |
| `ConditionLight::endCondition` | 2 | `ConditionLight::endCondition` (static); `Game::changeLight` (static) |
| `ConditionLight::addCondition` | 2 | `ConditionLight::addCondition` (static); `Game::changeLight` (static) |
| `ConditionLight::setParam` | 2 | `Condition::setParam` (static); `ConditionLight::setParam` (static) |
| `ConditionLight::getParam` | 2 | `Condition::getParam` (static); `ConditionLight::getParam` (static) |
| `ConditionLight::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionLight::unserializeProp` (static) |
| `ConditionLight::serialize` | 2 | `Condition::serialize` (static); `ConditionLight::serialize` (static) |
| `ConditionSpellCooldown::addCondition` | 1 | `ConditionSpellCooldown::addCondition` (static) |
| `ConditionSpellCooldown::startCondition` | 2 | `Condition::startCondition` (static); `ConditionSpellCooldown::startCondition` (static) |
| `ConditionSpellGroupCooldown::addCondition` | 1 | `ConditionSpellGroupCooldown::addCondition` (static) |
| `ConditionSpellGroupCooldown::startCondition` | 2 | `Condition::startCondition` (static); `ConditionSpellGroupCooldown::startCondition` (static) |
| `ConditionDrunk::startCondition` | 2 | `Condition::startCondition` (static); `ConditionDrunk::startCondition` (static) |
| `ConditionDrunk::updateCondition` | 1 | `ConditionDrunk::updateCondition` (static) |
| `ConditionDrunk::addCondition` | 1 | `ConditionDrunk::addCondition` (static) |
| `ConditionDrunk::endCondition` | 1 | `ConditionDrunk::endCondition` (static) |
| `ConditionDrunk::getIcons` | 1 | `ConditionDrunk::getIcons` (static) |
| `ConditionDrunk::setParam` | 2 | `Condition::setParam` (static); `ConditionDrunk::setParam` (static) |
| `ConditionManaShield::startCondition` | 2 | `Condition::startCondition` (static); `ConditionManaShield::startCondition` (static) |
| `ConditionManaShield::endCondition` | 1 | `ConditionManaShield::endCondition` (static) |
| `ConditionManaShield::addCondition` | 1 | `ConditionManaShield::addCondition` (static) |
| `ConditionManaShield::unserializeProp` | 2 | `Condition::unserializeProp` (static); `ConditionManaShield::unserializeProp` (static) |
| `ConditionManaShield::onDamageTaken` | 1 | `ConditionManaShield::onDamageTaken` (static) |
| `ConditionManaShield::serialize` | 2 | `Condition::serialize` (static); `ConditionManaShield::serialize` (static) |
| `ConditionManaShield::setParam` | 2 | `Condition::setParam` (static); `ConditionManaShield::setParam` (static) |
| `ConditionManaShield::getIcons` | 2 | `Condition::getIcons` (static); `ConditionManaShield::getIcons` (static) |

### `configmanager.h` (header declarations)

107 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_CONFIGMANAGER_H` | 0 | — |
| `boolean_config_t` | 0 | — |
| `boolean_config_t::ALLOW_CHANGEOUTFIT` | 0 | — |
| `boolean_config_t::ONE_PLAYER_ON_ACCOUNT` | 0 | — |
| `boolean_config_t::AIMBOT_HOTKEY_ENABLED` | 0 | — |
| `boolean_config_t::REMOVE_RUNE_CHARGES` | 0 | — |
| `boolean_config_t::REMOVE_WEAPON_AMMO` | 0 | — |
| `boolean_config_t::REMOVE_WEAPON_CHARGES` | 0 | — |
| `boolean_config_t::REMOVE_POTION_CHARGES` | 0 | — |
| `boolean_config_t::EXPERIENCE_FROM_PLAYERS` | 0 | — |
| `boolean_config_t::FREE_PREMIUM` | 0 | — |
| `boolean_config_t::REPLACE_KICK_ON_LOGIN` | 0 | — |
| `boolean_config_t::ALLOW_CLONES` | 0 | — |
| `boolean_config_t::ALLOW_WALKTHROUGH` | 0 | — |
| `boolean_config_t::BIND_ONLY_GLOBAL_ADDRESS` | 0 | — |
| `boolean_config_t::OPTIMIZE_DATABASE` | 0 | — |
| `boolean_config_t::MARKET_PREMIUM` | 0 | — |
| `boolean_config_t::EMOTE_SPELLS` | 0 | — |
| `boolean_config_t::STAMINA_SYSTEM` | 0 | — |
| `boolean_config_t::WARN_UNSAFE_SCRIPTS` | 0 | — |
| `boolean_config_t::CONVERT_UNSAFE_SCRIPTS` | 0 | — |
| `boolean_config_t::CLASSIC_EQUIPMENT_SLOTS` | 0 | — |
| `boolean_config_t::CLASSIC_ATTACK_SPEED` | 0 | — |
| `boolean_config_t::SCRIPTS_CONSOLE_LOGS` | 0 | — |
| `boolean_config_t::SERVER_SAVE_NOTIFY_MESSAGE` | 0 | — |
| `boolean_config_t::SERVER_SAVE_CLEAN_MAP` | 0 | — |
| `boolean_config_t::SERVER_SAVE_CLOSE` | 0 | — |
| `boolean_config_t::SERVER_SAVE_SHUTDOWN` | 0 | — |
| `boolean_config_t::ONLINE_OFFLINE_CHARLIST` | 0 | — |
| `boolean_config_t::YELL_ALLOW_PREMIUM` | 0 | — |
| `boolean_config_t::PREMIUM_TO_SEND_PRIVATE` | 0 | — |
| `boolean_config_t::FORCE_MONSTERTYPE_LOAD` | 0 | — |
| `boolean_config_t::HOUSE_OWNED_BY_ACCOUNT` | 0 | — |
| `boolean_config_t::CLEAN_PROTECTION_ZONES` | 0 | — |
| `boolean_config_t::HOUSE_DOOR_SHOW_PRICE` | 0 | — |
| `boolean_config_t::ONLY_INVITED_CAN_MOVE_HOUSE_ITEMS` | 0 | — |
| `boolean_config_t::REMOVE_ON_DESPAWN` | 0 | — |
| `boolean_config_t::TWO_FACTOR_AUTH` | 0 | — |
| `boolean_config_t::MANASHIELD_BREAKABLE` | 0 | — |
| `boolean_config_t::CHECK_DUPLICATE_STORAGE_KEYS` | 0 | — |
| `boolean_config_t::MONSTER_OVERSPAWN` | 0 | — |
| `string_config_t` | 0 | — |
| `string_config_t::MAP_NAME` | 0 | — |
| `string_config_t::HOUSE_RENT_PERIOD` | 0 | — |
| `string_config_t::SERVER_NAME` | 0 | — |
| `string_config_t::OWNER_NAME` | 0 | — |
| `string_config_t::OWNER_EMAIL` | 0 | — |
| `string_config_t::URL` | 0 | — |
| `string_config_t::LOCATION` | 0 | — |
| `string_config_t::IP` | 0 | — |
| `string_config_t::WORLD_TYPE` | 0 | — |
| `string_config_t::MYSQL_HOST` | 0 | — |
| `string_config_t::MYSQL_USER` | 0 | — |
| `string_config_t::MYSQL_PASS` | 0 | — |
| `string_config_t::MYSQL_DB` | 0 | — |
| `string_config_t::MYSQL_SOCK` | 0 | — |
| `string_config_t::DEFAULT_PRIORITY` | 0 | — |
| `string_config_t::MAP_AUTHOR` | 0 | — |
| `string_config_t::CONFIG_FILE` | 0 | — |
| `integer_config_t` | 0 | — |
| `integer_config_t::SQL_PORT` | 0 | — |
| `integer_config_t::MAX_PLAYERS` | 0 | — |
| `integer_config_t::PZ_LOCKED` | 0 | — |
| `integer_config_t::DEFAULT_DESPAWNRANGE` | 0 | — |
| `integer_config_t::DEFAULT_DESPAWNRADIUS` | 0 | — |
| `integer_config_t::DEFAULT_WALKTOSPAWNRADIUS` | 0 | — |
| `integer_config_t::RATE_EXPERIENCE` | 0 | — |
| `integer_config_t::RATE_SKILL` | 0 | — |
| `integer_config_t::RATE_LOOT` | 0 | — |
| `integer_config_t::RATE_MAGIC` | 0 | — |
| `integer_config_t::RATE_SPAWN` | 0 | — |
| `integer_config_t::HOUSE_PRICE` | 0 | — |
| `integer_config_t::KILLS_TO_RED` | 0 | — |
| `integer_config_t::KILLS_TO_BLACK` | 0 | — |
| `integer_config_t::MAX_MESSAGEBUFFER` | 0 | — |
| `integer_config_t::ACTIONS_DELAY_INTERVAL` | 0 | — |
| `integer_config_t::EX_ACTIONS_DELAY_INTERVAL` | 0 | — |
| `integer_config_t::KICK_AFTER_MINUTES` | 0 | — |
| `integer_config_t::PROTECTION_LEVEL` | 0 | — |
| `integer_config_t::DEATH_LOSE_PERCENT` | 0 | — |
| `integer_config_t::STATUSQUERY_TIMEOUT` | 0 | — |
| `integer_config_t::STATUS_COUNT_MAX_PLAYERS_PER_IP` | 0 | — |
| `integer_config_t::FRAG_TIME` | 0 | — |
| `integer_config_t::WHITE_SKULL_TIME` | 0 | — |
| `integer_config_t::GAME_PORT` | 0 | — |
| `integer_config_t::STATUS_PORT` | 0 | — |
| `integer_config_t::HTTP_PORT` | 0 | — |
| `integer_config_t::HTTP_WORKERS` | 0 | — |
| `integer_config_t::STAIRHOP_DELAY` | 0 | — |
| `integer_config_t::MARKET_OFFER_DURATION` | 0 | — |
| `integer_config_t::CHECK_EXPIRED_MARKET_OFFERS_EACH_MINUTES` | 0 | — |
| `integer_config_t::MAX_MARKET_OFFERS_AT_A_TIME_PER_PLAYER` | 0 | — |
| `integer_config_t::EXP_FROM_PLAYERS_LEVEL_RANGE` | 0 | — |
| `integer_config_t::MAX_PACKETS_PER_SECOND` | 0 | — |
| `integer_config_t::SERVER_SAVE_NOTIFY_DURATION` | 0 | — |
| `integer_config_t::YELL_MINIMUM_LEVEL` | 0 | — |
| `integer_config_t::MINIMUM_LEVEL_TO_SEND_PRIVATE` | 0 | — |
| `integer_config_t::VIP_FREE_LIMIT` | 0 | — |
| `integer_config_t::VIP_PREMIUM_LIMIT` | 0 | — |
| `integer_config_t::DEPOT_FREE_LIMIT` | 0 | — |
| `integer_config_t::DEPOT_PREMIUM_LIMIT` | 0 | — |
| `integer_config_t::QUEST_TRACKER_FREE_LIMIT` | 0 | — |
| `integer_config_t::QUEST_TRACKER_PREMIUM_LIMIT` | 0 | — |
| `integer_config_t::STAMINA_REGEN_MINUTE` | 0 | — |
| `integer_config_t::STAMINA_REGEN_PREMIUM` | 0 | — |
| `integer_config_t::PATHFINDING_INTERVAL` | 0 | — |
| `integer_config_t::PATHFINDING_DELAY` | 0 | — |

### `configmanager`

15 node(s), 9 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `getEnv` | 0 | — |
| `getEnv` | 0 | — |
| `getGlobalString` | 0 | — |
| `getGlobalNumber` | 0 | — |
| `getGlobalBoolean` | 0 | — |
| `loadLuaStages` | 1 | `tfs::lua::getField` (static) |
| `loadXMLStages` | 0 | — |
| `ConfigManager::load` | 1 | `ConfigManager::load` (static) |
| `ConfigManager::getString` | 1 | `ConfigManager::getString` (static) |
| `ConfigManager::getNumber` | 1 | `ConfigManager::getNumber` (static) |
| `ConfigManager::getBoolean` | 1 | `ConfigManager::getBoolean` (static) |
| `ConfigManager::getExperienceStage` | 1 | `ConfigManager::getExperienceStage` (static) |
| `ConfigManager::setString` | 1 | `ConfigManager::setString` (static) |
| `ConfigManager::setNumber` | 1 | `ConfigManager::setNumber` (static) |
| `ConfigManager::setBoolean` | 1 | `ConfigManager::setBoolean` (static) |

### `connection.h` (header declarations)

46 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ConnectionState_t` | 0 | — |
| `ConnectionState_t::CONNECTION_STATE_DISCONNECTED` | 0 | — |
| `ConnectionState_t::CONNECTION_STATE_REQUEST_CHARLIST` | 0 | — |
| `ConnectionState_t::CONNECTION_STATE_GAMEWORLD_AUTH` | 0 | — |
| `ConnectionState_t::CONNECTION_STATE_GAME` | 0 | — |
| `ConnectionState_t::CONNECTION_STATE_PENDING` | 0 | — |
| `checksumMode_t` | 0 | — |
| `checksumMode_t::CHECKSUM_DISABLED` | 0 | — |
| `checksumMode_t::CHECKSUM_ADLER` | 0 | — |
| `checksumMode_t::CHECKSUM_SEQUENCE` | 0 | — |
| `CONNECTION_WRITE_TIMEOUT` | 0 | — |
| `CONNECTION_READ_TIMEOUT` | 0 | — |
| `Protocol_ptr` | 0 | — |
| `OutputMessage_ptr` | 0 | — |
| `Connection_ptr` | 0 | — |
| `ConnectionWeak_ptr` | 0 | — |
| `Service_ptr` | 0 | — |
| `ServicePort_ptr` | 0 | — |
| `ConstServicePort_ptr` | 0 | — |
| `ConnectionManager` | 0 | — |
| `ConnectionManager::getInstance` | 0 | — |
| `ConnectionManager::ConnectionManager` | 0 | — |
| `ConnectionManager::connections` | 0 | — |
| `ConnectionManager::connectionManagerLock` | 0 | — |
| `Connection` | 0 | — |
| `Connection::Address` | 0 | — |
| `Connection::Connection(const Connection&)` | 0 | — |
| `Connection::operator=` | 0 | — |
| `Connection::FORCE_CLOSE` | 0 | — |
| `Connection::getIP` | 0 | — |
| `Connection::getSocket` | 0 | — |
| `Connection::msg` | 0 | — |
| `Connection::readTimer` | 0 | — |
| `Connection::writeTimer` | 0 | — |
| `Connection::connectionLock` | 0 | — |
| `Connection::messageQueue` | 0 | — |
| `Connection::service_port` | 0 | — |
| `Connection::protocol` | 0 | — |
| `Connection::socket` | 0 | — |
| `Connection::remoteAddress` | 0 | — |
| `Connection::timeConnected` | 0 | — |
| `Connection::packetsSent` | 0 | — |
| `Connection::connectionState` | 0 | — |
| `Connection::receivedFirst` | 0 | — |
| `Connection::receivedName` | 0 | — |
| `Connection::receivedLastChar` | 0 | — |

### `connection`

15 node(s), 20 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ConnectionManager::createConnection` | 1 | `ConnectionManager::createConnection` (static) |
| `ConnectionManager::releaseConnection` | 1 | `ConnectionManager::releaseConnection` (static) |
| `ConnectionManager::closeAll` | 1 | `ConnectionManager::closeAll` (static) |
| `Connection::Connection` | 1 | `Connection::Connection` (static) |
| `Connection::close` | 2 | `Connection::close` (static); `ConnectionManager::getInstance` (static) |
| `Connection::closeSocket` | 1 | `Connection::closeSocket` (static) |
| `Connection::~Connection` | 0 | — |
| `Connection::accept(Protocol_ptr)` | 0 | — |
| `Connection::accept()` | 1 | `Connection::handleTimeout` (static) |
| `Connection::parseHeader` | 2 | `Connection::handleTimeout` (static); `Connection::parseHeader` (static) |
| `Connection::parsePacket` | 5 | `Connection::handleTimeout` (static); `Connection::parsePacket` (static); `ProtocolGame::onRecvFirstMessage` (dynamic/curated); `ProtocolStatus::onRecvFirstMessage` (dynamic/curated); `ProtocolGame::parsePacket` (dynamic/curated) |
| `Connection::send` | 1 | `Connection::send` (static) |
| `Connection::internalSend` | 2 | `Connection::handleTimeout` (static); `Connection::internalSend` (static) |
| `Connection::onWriteOperation` | 1 | `Connection::onWriteOperation` (static) |
| `Connection::handleTimeout` | 1 | `Connection::handleTimeout` (static) |

### `const.h` (header declarations)

560 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_CONST_H` | 0 | — |
| `MagicEffectsType_t` | 0 | — |
| `MagicEffectsType_t::MAGIC_EFFECTS_END_LOOP` | 0 | — |
| `MagicEffectsType_t::MAGIC_EFFECTS_DELTA` | 0 | — |
| `MagicEffectsType_t::MAGIC_EFFECTS_DELAY` | 0 | — |
| `MagicEffectsType_t::MAGIC_EFFECTS_CREATE_EFFECT` | 0 | — |
| `MagicEffectsType_t::MAGIC_EFFECTS_CREATE_DISTANCEEFFECT` | 0 | — |
| `MagicEffectsType_t::MAGIC_EFFECTS_CREATE_DISTANCEEFFECT_REVERSED` | 0 | — |
| `MagicEffectClasses` | 0 | — |
| `MagicEffectClasses::CONST_ME_NONE` | 0 | — |
| `MagicEffectClasses::CONST_ME_DRAWBLOOD` | 0 | — |
| `MagicEffectClasses::CONST_ME_LOSEENERGY` | 0 | — |
| `MagicEffectClasses::CONST_ME_POFF` | 0 | — |
| `MagicEffectClasses::CONST_ME_BLOCKHIT` | 0 | — |
| `MagicEffectClasses::CONST_ME_EXPLOSIONAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_EXPLOSIONHIT` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_YELLOW_RINGS` | 0 | — |
| `MagicEffectClasses::CONST_ME_GREEN_RINGS` | 0 | — |
| `MagicEffectClasses::CONST_ME_HITAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_TELEPORT` | 0 | — |
| `MagicEffectClasses::CONST_ME_ENERGYHIT` | 0 | — |
| `MagicEffectClasses::CONST_ME_MAGIC_BLUE` | 0 | — |
| `MagicEffectClasses::CONST_ME_MAGIC_RED` | 0 | — |
| `MagicEffectClasses::CONST_ME_MAGIC_GREEN` | 0 | — |
| `MagicEffectClasses::CONST_ME_HITBYFIRE` | 0 | — |
| `MagicEffectClasses::CONST_ME_HITBYPOISON` | 0 | — |
| `MagicEffectClasses::CONST_ME_MORTAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_SOUND_GREEN` | 0 | — |
| `MagicEffectClasses::CONST_ME_SOUND_RED` | 0 | — |
| `MagicEffectClasses::CONST_ME_POISONAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_SOUND_YELLOW` | 0 | — |
| `MagicEffectClasses::CONST_ME_SOUND_PURPLE` | 0 | — |
| `MagicEffectClasses::CONST_ME_SOUND_BLUE` | 0 | — |
| `MagicEffectClasses::CONST_ME_SOUND_WHITE` | 0 | — |
| `MagicEffectClasses::CONST_ME_BUBBLES` | 0 | — |
| `MagicEffectClasses::CONST_ME_CRAPS` | 0 | — |
| `MagicEffectClasses::CONST_ME_GIFT_WRAPS` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORK_YELLOW` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORK_RED` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORK_BLUE` | 0 | — |
| `MagicEffectClasses::CONST_ME_STUN` | 0 | — |
| `MagicEffectClasses::CONST_ME_SLEEP` | 0 | — |
| `MagicEffectClasses::CONST_ME_WATERCREATURE` | 0 | — |
| `MagicEffectClasses::CONST_ME_GROUNDSHAKER` | 0 | — |
| `MagicEffectClasses::CONST_ME_HEARTS` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREATTACK` | 0 | — |
| `MagicEffectClasses::CONST_ME_ENERGYAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_SMALLCLOUDS` | 0 | — |
| `MagicEffectClasses::CONST_ME_HOLYDAMAGE` | 0 | — |
| `MagicEffectClasses::CONST_ME_BIGCLOUDS` | 0 | — |
| `MagicEffectClasses::CONST_ME_ICEAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_ICETORNADO` | 0 | — |
| `MagicEffectClasses::CONST_ME_ICEATTACK` | 0 | — |
| `MagicEffectClasses::CONST_ME_STONES` | 0 | — |
| `MagicEffectClasses::CONST_ME_SMALLPLANTS` | 0 | — |
| `MagicEffectClasses::CONST_ME_CARNIPHILA` | 0 | — |
| `MagicEffectClasses::CONST_ME_PURPLEENERGY` | 0 | — |
| `MagicEffectClasses::CONST_ME_YELLOWENERGY` | 0 | — |
| `MagicEffectClasses::CONST_ME_HOLYAREA` | 0 | — |
| `MagicEffectClasses::CONST_ME_BIGPLANTS` | 0 | — |
| `MagicEffectClasses::CONST_ME_CAKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_GIANTICE` | 0 | — |
| `MagicEffectClasses::CONST_ME_WATERSPLASH` | 0 | — |
| `MagicEffectClasses::CONST_ME_PLANTATTACK` | 0 | — |
| `MagicEffectClasses::CONST_ME_TUTORIALARROW` | 0 | — |
| `MagicEffectClasses::CONST_ME_TUTORIALSQUARE` | 0 | — |
| `MagicEffectClasses::CONST_ME_MIRRORHORIZONTAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_MIRRORVERTICAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_SKULLHORIZONTAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_SKULLVERTICAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_ASSASSIN` | 0 | — |
| `MagicEffectClasses::CONST_ME_STEPSHORIZONTAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_BLOODYSTEPS` | 0 | — |
| `MagicEffectClasses::CONST_ME_STEPSVERTICAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_YALAHARIGHOST` | 0 | — |
| `MagicEffectClasses::CONST_ME_BATS` | 0 | — |
| `MagicEffectClasses::CONST_ME_SMOKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_INSECTS` | 0 | — |
| `MagicEffectClasses::CONST_ME_DRAGONHEAD` | 0 | — |
| `MagicEffectClasses::CONST_ME_ORCSHAMAN` | 0 | — |
| `MagicEffectClasses::CONST_ME_ORCSHAMAN_FIRE` | 0 | — |
| `MagicEffectClasses::CONST_ME_THUNDER` | 0 | — |
| `MagicEffectClasses::CONST_ME_FERUMBRAS` | 0 | — |
| `MagicEffectClasses::CONST_ME_CONFETTI_HORIZONTAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_CONFETTI_VERTICAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_BLACKSMOKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_REDSMOKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_YELLOWSMOKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_GREENSMOKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_PURPLESMOKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_EARLY_THUNDER` | 0 | — |
| `MagicEffectClasses::CONST_ME_RAGIAZ_BONECAPSULE` | 0 | — |
| `MagicEffectClasses::CONST_ME_CRITICAL_DAMAGE` | 0 | — |
| `MagicEffectClasses::CONST_ME_PLUNGING_FISH` | 0 | — |
| `MagicEffectClasses::CONST_ME_BLUECHAIN` | 0 | — |
| `MagicEffectClasses::CONST_ME_ORANGECHAIN` | 0 | — |
| `MagicEffectClasses::CONST_ME_GREENCHAIN` | 0 | — |
| `MagicEffectClasses::CONST_ME_PURPLECHAIN` | 0 | — |
| `MagicEffectClasses::CONST_ME_GREYCHAIN` | 0 | — |
| `MagicEffectClasses::CONST_ME_YELLOWCHAIN` | 0 | — |
| `MagicEffectClasses::CONST_ME_YELLOWSPARKLES` | 0 | — |
| `MagicEffectClasses::CONST_ME_FAEEXPLOSION` | 0 | — |
| `MagicEffectClasses::CONST_ME_FAECOMING` | 0 | — |
| `MagicEffectClasses::CONST_ME_FAEGOING` | 0 | — |
| `MagicEffectClasses::CONST_ME_BIGCLOUDSSINGLESPACE` | 0 | — |
| `MagicEffectClasses::CONST_ME_STONESSINGLESPACE` | 0 | — |
| `MagicEffectClasses::CONST_ME_BLUEGHOST` | 0 | — |
| `MagicEffectClasses::CONST_ME_POINTOFINTEREST` | 0 | — |
| `MagicEffectClasses::CONST_ME_MAPEFFECT` | 0 | — |
| `MagicEffectClasses::CONST_ME_PINKSPARK` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORK_GREEN` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORK_ORANGE` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORK_PURPLE` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORK_TURQUOISE` | 0 | — |
| `MagicEffectClasses::CONST_ME_THECUBE` | 0 | — |
| `MagicEffectClasses::CONST_ME_DRAWINK` | 0 | — |
| `MagicEffectClasses::CONST_ME_PRISMATICSPARKLES` | 0 | — |
| `MagicEffectClasses::CONST_ME_THAIAN` | 0 | — |
| `MagicEffectClasses::CONST_ME_THAIANGHOST` | 0 | — |
| `MagicEffectClasses::CONST_ME_GHOSTSMOKE` | 0 | — |
| `MagicEffectClasses::CONST_ME_FLOATINGBLOCK` | 0 | — |
| `MagicEffectClasses::CONST_ME_BLOCK` | 0 | — |
| `MagicEffectClasses::CONST_ME_ROOTING` | 0 | — |
| `MagicEffectClasses::CONST_ME_GHOSTLYSCRATCH` | 0 | — |
| `MagicEffectClasses::CONST_ME_GHOSTLYBITE` | 0 | — |
| `MagicEffectClasses::CONST_ME_BIGSCRATCHING` | 0 | — |
| `MagicEffectClasses::CONST_ME_SLASH` | 0 | — |
| `MagicEffectClasses::CONST_ME_BITE` | 0 | — |
| `MagicEffectClasses::CONST_ME_CHIVALRIOUSCHALLENGE` | 0 | — |
| `MagicEffectClasses::CONST_ME_DIVINEDAZZLE` | 0 | — |
| `MagicEffectClasses::CONST_ME_ELECTRICALSPARK` | 0 | — |
| `MagicEffectClasses::CONST_ME_PURPLETELEPORT` | 0 | — |
| `MagicEffectClasses::CONST_ME_REDTELEPORT` | 0 | — |
| `MagicEffectClasses::CONST_ME_ORANGETELEPORT` | 0 | — |
| `MagicEffectClasses::CONST_ME_GREYTELEPORT` | 0 | — |
| `MagicEffectClasses::CONST_ME_LIGHTBLUETELEPORT` | 0 | — |
| `MagicEffectClasses::CONST_ME_FATAL` | 0 | — |
| `MagicEffectClasses::CONST_ME_DODGE` | 0 | — |
| `MagicEffectClasses::CONST_ME_HOURGLASS` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORKSSTAR` | 0 | — |
| `MagicEffectClasses::CONST_ME_FIREWORKSCIRCLE` | 0 | — |
| `MagicEffectClasses::CONST_ME_FERUMBRAS_1` | 0 | — |
| `MagicEffectClasses::CONST_ME_GAZHARAGOTH` | 0 | — |
| `MagicEffectClasses::CONST_ME_MAD_MAGE` | 0 | — |
| `MagicEffectClasses::CONST_ME_HORESTIS` | 0 | — |
| `MagicEffectClasses::CONST_ME_DEVOVORGA` | 0 | — |
| `MagicEffectClasses::CONST_ME_FERUMBRAS_2` | 0 | — |
| `MagicEffectClasses::CONST_ME_FOAM` | 0 | — |
| `ShootType_t` | 0 | — |
| `ShootType_t::CONST_ANI_NONE` | 0 | — |
| `ShootType_t::CONST_ANI_SPEAR` | 0 | — |
| `ShootType_t::CONST_ANI_BOLT` | 0 | — |
| `ShootType_t::CONST_ANI_ARROW` | 0 | — |
| `ShootType_t::CONST_ANI_FIRE` | 0 | — |
| `ShootType_t::CONST_ANI_ENERGY` | 0 | — |
| `ShootType_t::CONST_ANI_POISONARROW` | 0 | — |
| `ShootType_t::CONST_ANI_BURSTARROW` | 0 | — |
| `ShootType_t::CONST_ANI_THROWINGSTAR` | 0 | — |
| `ShootType_t::CONST_ANI_THROWINGKNIFE` | 0 | — |
| `ShootType_t::CONST_ANI_SMALLSTONE` | 0 | — |
| `ShootType_t::CONST_ANI_DEATH` | 0 | — |
| `ShootType_t::CONST_ANI_LARGEROCK` | 0 | — |
| `ShootType_t::CONST_ANI_SNOWBALL` | 0 | — |
| `ShootType_t::CONST_ANI_POWERBOLT` | 0 | — |
| `ShootType_t::CONST_ANI_POISON` | 0 | — |
| `ShootType_t::CONST_ANI_INFERNALBOLT` | 0 | — |
| `ShootType_t::CONST_ANI_HUNTINGSPEAR` | 0 | — |
| `ShootType_t::CONST_ANI_ENCHANTEDSPEAR` | 0 | — |
| `ShootType_t::CONST_ANI_REDSTAR` | 0 | — |
| `ShootType_t::CONST_ANI_GREENSTAR` | 0 | — |
| `ShootType_t::CONST_ANI_ROYALSPEAR` | 0 | — |
| `ShootType_t::CONST_ANI_SNIPERARROW` | 0 | — |
| `ShootType_t::CONST_ANI_ONYXARROW` | 0 | — |
| `ShootType_t::CONST_ANI_PIERCINGBOLT` | 0 | — |
| `ShootType_t::CONST_ANI_WHIRLWINDSWORD` | 0 | — |
| `ShootType_t::CONST_ANI_WHIRLWINDAXE` | 0 | — |
| `ShootType_t::CONST_ANI_WHIRLWINDCLUB` | 0 | — |
| `ShootType_t::CONST_ANI_ETHEREALSPEAR` | 0 | — |
| `ShootType_t::CONST_ANI_ICE` | 0 | — |
| `ShootType_t::CONST_ANI_EARTH` | 0 | — |
| `ShootType_t::CONST_ANI_HOLY` | 0 | — |
| `ShootType_t::CONST_ANI_SUDDENDEATH` | 0 | — |
| `ShootType_t::CONST_ANI_FLASHARROW` | 0 | — |
| `ShootType_t::CONST_ANI_FLAMMINGARROW` | 0 | — |
| `ShootType_t::CONST_ANI_SHIVERARROW` | 0 | — |
| `ShootType_t::CONST_ANI_ENERGYBALL` | 0 | — |
| `ShootType_t::CONST_ANI_SMALLICE` | 0 | — |
| `ShootType_t::CONST_ANI_SMALLHOLY` | 0 | — |
| `ShootType_t::CONST_ANI_SMALLEARTH` | 0 | — |
| `ShootType_t::CONST_ANI_EARTHARROW` | 0 | — |
| `ShootType_t::CONST_ANI_EXPLOSION` | 0 | — |
| `ShootType_t::CONST_ANI_CAKE` | 0 | — |
| `ShootType_t::CONST_ANI_TARSALARROW` | 0 | — |
| `ShootType_t::CONST_ANI_VORTEXBOLT` | 0 | — |
| `ShootType_t::CONST_ANI_PRISMATICBOLT` | 0 | — |
| `SpeakClasses` | 0 | — |
| `SpeakClasses::TALKTYPE_SAY` | 0 | — |
| `SpeakClasses::TALKTYPE_WHISPER` | 0 | — |
| `SpeakClasses::TALKTYPE_YELL` | 0 | — |
| `SpeakClasses::TALKTYPE_PRIVATE_FROM` | 0 | — |
| `SpeakClasses::TALKTYPE_PRIVATE_TO` | 0 | — |
| `SpeakClasses::TALKTYPE_CHANNEL_Y` | 0 | — |
| `SpeakClasses::TALKTYPE_CHANNEL_O` | 0 | — |
| `SpeakClasses::TALKTYPE_SPELL` | 0 | — |
| `SpeakClasses::TALKTYPE_PRIVATE_NP` | 0 | — |
| `SpeakClasses::TALKTYPE_PRIVATE_NP_CONSOLE` | 0 | — |
| `SpeakClasses::TALKTYPE_PRIVATE_PN` | 0 | — |
| `SpeakClasses::TALKTYPE_BROADCAST` | 0 | — |
| `SpeakClasses::TALKTYPE_CHANNEL_R1` | 0 | — |
| `SpeakClasses::TALKTYPE_PRIVATE_RED_FROM` | 0 | — |
| `SpeakClasses::TALKTYPE_PRIVATE_RED_TO` | 0 | — |
| `SpeakClasses::TALKTYPE_MONSTER_SAY` | 0 | — |
| `SpeakClasses::TALKTYPE_MONSTER_YELL` | 0 | — |
| `SpeakClasses::TALKTYPE_POTION` | 0 | — |
| `MessageClasses` | 0 | — |
| `MessageClasses::MESSAGE_STATUS_DEFAULT` | 0 | — |
| `MessageClasses::MESSAGE_STATUS_WARNING` | 0 | — |
| `MessageClasses::MESSAGE_EVENT_ADVANCE` | 0 | — |
| `MessageClasses::MESSAGE_STATUS_WARNING2` | 0 | — |
| `MessageClasses::MESSAGE_STATUS_SMALL` | 0 | — |
| `MessageClasses::MESSAGE_INFO_DESCR` | 0 | — |
| `MessageClasses::MESSAGE_DAMAGE_DEALT` | 0 | — |
| `MessageClasses::MESSAGE_DAMAGE_RECEIVED` | 0 | — |
| `MessageClasses::MESSAGE_HEALED` | 0 | — |
| `MessageClasses::MESSAGE_EXPERIENCE` | 0 | — |
| `MessageClasses::MESSAGE_DAMAGE_OTHERS` | 0 | — |
| `MessageClasses::MESSAGE_HEALED_OTHERS` | 0 | — |
| `MessageClasses::MESSAGE_EXPERIENCE_OTHERS` | 0 | — |
| `MessageClasses::MESSAGE_EVENT_DEFAULT` | 0 | — |
| `MessageClasses::MESSAGE_LOOT` | 0 | — |
| `MessageClasses::MESSAGE_TRADE` | 0 | — |
| `MessageClasses::MESSAGE_GUILD` | 0 | — |
| `MessageClasses::MESSAGE_PARTY_MANAGEMENT` | 0 | — |
| `MessageClasses::MESSAGE_PARTY` | 0 | — |
| `MessageClasses::MESSAGE_REPORT` | 0 | — |
| `MessageClasses::MESSAGE_HOTKEY_PRESSED` | 0 | — |
| `MessageClasses::MESSAGE_MARKET` | 0 | — |
| `MessageClasses::MESSAGE_BEYOND_LAST` | 0 | — |
| `MessageClasses::MESSAGE_TOURNAMENT_INFO` | 0 | — |
| `MessageClasses::MESSAGE_ATTENTION` | 0 | — |
| `MessageClasses::MESSAGE_BOOSTED_CREATURE` | 0 | — |
| `MessageClasses::MESSAGE_OFFLINE_TRAINING` | 0 | — |
| `MessageClasses::MESSAGE_TRANSACTION` | 0 | — |
| `FluidColors_t` | 0 | — |
| `FluidColors_t::FLUID_EMPTY` | 0 | — |
| `FluidColors_t::FLUID_BLUE` | 0 | — |
| `FluidColors_t::FLUID_RED` | 0 | — |
| `FluidColors_t::FLUID_BROWN` | 0 | — |
| `FluidColors_t::FLUID_GREEN` | 0 | — |
| `FluidColors_t::FLUID_YELLOW` | 0 | — |
| `FluidColors_t::FLUID_WHITE` | 0 | — |
| `FluidColors_t::FLUID_PURPLE` | 0 | — |
| `FluidColors_t::FLUID_BLACK` | 0 | — |
| `FluidTypes_t` | 0 | — |
| `FluidTypes_t::FLUID_NONE` | 0 | — |
| `FluidTypes_t::FLUID_WATER` | 0 | — |
| `FluidTypes_t::FLUID_BLOOD` | 0 | — |
| `FluidTypes_t::FLUID_BEER` | 0 | — |
| `FluidTypes_t::FLUID_SLIME` | 0 | — |
| `FluidTypes_t::FLUID_LEMONADE` | 0 | — |
| `FluidTypes_t::FLUID_MILK` | 0 | — |
| `FluidTypes_t::FLUID_MANA` | 0 | — |
| `FluidTypes_t::FLUID_INK` | 0 | — |
| `FluidTypes_t::FLUID_LIFE` | 0 | — |
| `FluidTypes_t::FLUID_OIL` | 0 | — |
| `FluidTypes_t::FLUID_URINE` | 0 | — |
| `FluidTypes_t::FLUID_COCONUTMILK` | 0 | — |
| `FluidTypes_t::FLUID_WINE` | 0 | — |
| `FluidTypes_t::FLUID_MUD` | 0 | — |
| `FluidTypes_t::FLUID_FRUITJUICE` | 0 | — |
| `FluidTypes_t::FLUID_LAVA` | 0 | — |
| `FluidTypes_t::FLUID_RUM` | 0 | — |
| `FluidTypes_t::FLUID_SWAMP` | 0 | — |
| `FluidTypes_t::FLUID_TEA` | 0 | — |
| `FluidTypes_t::FLUID_MEAD` | 0 | — |
| `ClientFluidTypes_t` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_EMPTY` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_BLUE` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_PURPLE` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_BROWN_1` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_BROWN_2` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_RED` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_GREEN` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_BROWN` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_YELLOW` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_WHITE` | 0 | — |
| `ClientFluidTypes_t::CLIENTFLUID_BLACK` | 0 | — |
| `SquareColor_t` | 0 | — |
| `SquareColor_t::SQ_COLOR_BLACK` | 0 | — |
| `TextColor_t` | 0 | — |
| `TextColor_t::TEXTCOLOR_BLUE` | 0 | — |
| `TextColor_t::TEXTCOLOR_LIGHTGREEN` | 0 | — |
| `TextColor_t::TEXTCOLOR_LIGHTBLUE` | 0 | — |
| `TextColor_t::TEXTCOLOR_DARKGREY` | 0 | — |
| `TextColor_t::TEXTCOLOR_MAYABLUE` | 0 | — |
| `TextColor_t::TEXTCOLOR_DARKRED` | 0 | — |
| `TextColor_t::TEXTCOLOR_LIGHTGREY` | 0 | — |
| `TextColor_t::TEXTCOLOR_SKYBLUE` | 0 | — |
| `TextColor_t::TEXTCOLOR_PURPLE` | 0 | — |
| `TextColor_t::TEXTCOLOR_ELECTRICPURPLE` | 0 | — |
| `TextColor_t::TEXTCOLOR_RED` | 0 | — |
| `TextColor_t::TEXTCOLOR_PASTELRED` | 0 | — |
| `TextColor_t::TEXTCOLOR_ORANGE` | 0 | — |
| `TextColor_t::TEXTCOLOR_YELLOW` | 0 | — |
| `TextColor_t::TEXTCOLOR_WHITE_EXP` | 0 | — |
| `TextColor_t::TEXTCOLOR_NONE` | 0 | — |
| `Icons_t` | 0 | — |
| `Icons_t::ICON_POISON` | 0 | — |
| `Icons_t::ICON_BURN` | 0 | — |
| `Icons_t::ICON_ENERGY` | 0 | — |
| `Icons_t::ICON_DRUNK` | 0 | — |
| `Icons_t::ICON_MANASHIELD` | 0 | — |
| `Icons_t::ICON_PARALYZE` | 0 | — |
| `Icons_t::ICON_HASTE` | 0 | — |
| `Icons_t::ICON_SWORDS` | 0 | — |
| `Icons_t::ICON_DROWNING` | 0 | — |
| `Icons_t::ICON_FREEZING` | 0 | — |
| `Icons_t::ICON_DAZZLED` | 0 | — |
| `Icons_t::ICON_CURSED` | 0 | — |
| `Icons_t::ICON_PARTY_BUFF` | 0 | — |
| `Icons_t::ICON_REDSWORDS` | 0 | — |
| `Icons_t::ICON_PIGEON` | 0 | — |
| `Icons_t::ICON_BLEEDING` | 0 | — |
| `Icons_t::ICON_LESSERHEX` | 0 | — |
| `Icons_t::ICON_INTENSEHEX` | 0 | — |
| `Icons_t::ICON_GREATERHEX` | 0 | — |
| `Icons_t::ICON_ROOT` | 0 | — |
| `Icons_t::ICON_FEAR` | 0 | — |
| `Icons_t::ICON_GOSHNAR1` | 0 | — |
| `Icons_t::ICON_GOSHNAR2` | 0 | — |
| `Icons_t::ICON_GOSHNAR3` | 0 | — |
| `Icons_t::ICON_GOSHNAR4` | 0 | — |
| `Icons_t::ICON_GOSHNAR5` | 0 | — |
| `Icons_t::ICON_MANASHIELD_BREAKABLE` | 0 | — |
| `WeaponType_t` | 0 | — |
| `WeaponType_t::WEAPON_NONE` | 0 | — |
| `WeaponType_t::WEAPON_SWORD` | 0 | — |
| `WeaponType_t::WEAPON_CLUB` | 0 | — |
| `WeaponType_t::WEAPON_AXE` | 0 | — |
| `WeaponType_t::WEAPON_SHIELD` | 0 | — |
| `WeaponType_t::WEAPON_DISTANCE` | 0 | — |
| `WeaponType_t::WEAPON_WAND` | 0 | — |
| `WeaponType_t::WEAPON_AMMO` | 0 | — |
| `WeaponType_t::WEAPON_QUIVER` | 0 | — |
| `Ammo_t` | 0 | — |
| `Ammo_t::AMMO_NONE` | 0 | — |
| `Ammo_t::AMMO_BOLT` | 0 | — |
| `Ammo_t::AMMO_ARROW` | 0 | — |
| `Ammo_t::AMMO_SPEAR` | 0 | — |
| `Ammo_t::AMMO_THROWINGSTAR` | 0 | — |
| `Ammo_t::AMMO_THROWINGKNIFE` | 0 | — |
| `Ammo_t::AMMO_STONE` | 0 | — |
| `Ammo_t::AMMO_SNOWBALL` | 0 | — |
| `WeaponAction_t` | 0 | — |
| `WeaponAction_t::WEAPONACTION_NONE` | 0 | — |
| `WeaponAction_t::WEAPONACTION_REMOVECOUNT` | 0 | — |
| `WeaponAction_t::WEAPONACTION_REMOVECHARGE` | 0 | — |
| `WeaponAction_t::WEAPONACTION_MOVE` | 0 | — |
| `WieldInfo_t` | 0 | — |
| `WieldInfo_t::WIELDINFO_NONE` | 0 | — |
| `WieldInfo_t::WIELDINFO_LEVEL` | 0 | — |
| `WieldInfo_t::WIELDINFO_MAGLV` | 0 | — |
| `WieldInfo_t::WIELDINFO_VOCREQ` | 0 | — |
| `WieldInfo_t::WIELDINFO_PREMIUM` | 0 | — |
| `Skulls_t` | 0 | — |
| `Skulls_t::SKULL_NONE` | 0 | — |
| `Skulls_t::SKULL_YELLOW` | 0 | — |
| `Skulls_t::SKULL_GREEN` | 0 | — |
| `Skulls_t::SKULL_WHITE` | 0 | — |
| `Skulls_t::SKULL_RED` | 0 | — |
| `Skulls_t::SKULL_BLACK` | 0 | — |
| `Skulls_t::SKULL_ORANGE` | 0 | — |
| `PartyShields_t` | 0 | — |
| `PartyShields_t::SHIELD_NONE` | 0 | — |
| `PartyShields_t::SHIELD_WHITEYELLOW` | 0 | — |
| `PartyShields_t::SHIELD_WHITEBLUE` | 0 | — |
| `PartyShields_t::SHIELD_BLUE` | 0 | — |
| `PartyShields_t::SHIELD_YELLOW` | 0 | — |
| `PartyShields_t::SHIELD_BLUE_SHAREDEXP` | 0 | — |
| `PartyShields_t::SHIELD_YELLOW_SHAREDEXP` | 0 | — |
| `PartyShields_t::SHIELD_BLUE_NOSHAREDEXP_BLINK` | 0 | — |
| `PartyShields_t::SHIELD_YELLOW_NOSHAREDEXP_BLINK` | 0 | — |
| `PartyShields_t::SHIELD_BLUE_NOSHAREDEXP` | 0 | — |
| `PartyShields_t::SHIELD_YELLOW_NOSHAREDEXP` | 0 | — |
| `PartyShields_t::SHIELD_GRAY` | 0 | — |
| `GuildEmblems_t` | 0 | — |
| `GuildEmblems_t::GUILDEMBLEM_NONE` | 0 | — |
| `GuildEmblems_t::GUILDEMBLEM_ALLY` | 0 | — |
| `GuildEmblems_t::GUILDEMBLEM_ENEMY` | 0 | — |
| `GuildEmblems_t::GUILDEMBLEM_NEUTRAL` | 0 | — |
| `GuildEmblems_t::GUILDEMBLEM_MEMBER` | 0 | — |
| `GuildEmblems_t::GUILDEMBLEM_OTHER` | 0 | — |
| `item_t` | 0 | — |
| `item_t::ITEM_BROWSEFIELD` | 0 | — |
| `item_t::ITEM_DECORATION_KIT` | 0 | — |
| `item_t::ITEM_FIREFIELD_PVP_FULL` | 0 | — |
| `item_t::ITEM_FIREFIELD_PVP_MEDIUM` | 0 | — |
| `item_t::ITEM_FIREFIELD_PVP_SMALL` | 0 | — |
| `item_t::ITEM_FIREFIELD_PERSISTENT_FULL` | 0 | — |
| `item_t::ITEM_FIREFIELD_PERSISTENT_MEDIUM` | 0 | — |
| `item_t::ITEM_FIREFIELD_PERSISTENT_SMALL` | 0 | — |
| `item_t::ITEM_FIREFIELD_NOPVP` | 0 | — |
| `item_t::ITEM_FIREFIELD_NOPVP_MEDIUM` | 0 | — |
| `item_t::ITEM_POISONFIELD_PVP` | 0 | — |
| `item_t::ITEM_POISONFIELD_PERSISTENT` | 0 | — |
| `item_t::ITEM_POISONFIELD_NOPVP` | 0 | — |
| `item_t::ITEM_ENERGYFIELD_PVP` | 0 | — |
| `item_t::ITEM_ENERGYFIELD_PERSISTENT` | 0 | — |
| `item_t::ITEM_ENERGYFIELD_NOPVP` | 0 | — |
| `item_t::ITEM_MAGICWALL` | 0 | — |
| `item_t::ITEM_MAGICWALL_PERSISTENT` | 0 | — |
| `item_t::ITEM_MAGICWALL_SAFE` | 0 | — |
| `item_t::ITEM_MAGICWALL_NOPVP` | 0 | — |
| `item_t::ITEM_WILDGROWTH` | 0 | — |
| `item_t::ITEM_WILDGROWTH_PERSISTENT` | 0 | — |
| `item_t::ITEM_WILDGROWTH_SAFE` | 0 | — |
| `item_t::ITEM_WILDGROWTH_NOPVP` | 0 | — |
| `item_t::ITEM_BAG` | 0 | — |
| `item_t::ITEM_SHOPPING_BAG` | 0 | — |
| `item_t::ITEM_GOLD_COIN` | 0 | — |
| `item_t::ITEM_PLATINUM_COIN` | 0 | — |
| `item_t::ITEM_CRYSTAL_COIN` | 0 | — |
| `item_t::ITEM_STORE_COIN` | 0 | — |
| `item_t::ITEM_DEPOT` | 0 | — |
| `item_t::ITEM_LOCKER` | 0 | — |
| `item_t::ITEM_INBOX` | 0 | — |
| `item_t::ITEM_MARKET` | 0 | — |
| `item_t::ITEM_STORE_INBOX` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_I` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_II` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_III` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_IV` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_V` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_VI` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_VII` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_VIII` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_IX` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_X` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XI` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XII` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XIII` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XIV` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XV` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XVI` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XVII` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XVIII` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XIX` | 0 | — |
| `item_t::ITEM_DEPOT_BOX_XX` | 0 | — |
| `item_t::ITEM_MALE_CORPSE` | 0 | — |
| `item_t::ITEM_FEMALE_CORPSE` | 0 | — |
| `item_t::ITEM_FULLSPLASH` | 0 | — |
| `item_t::ITEM_SMALLSPLASH` | 0 | — |
| `item_t::ITEM_PARCEL` | 0 | — |
| `item_t::ITEM_LETTER` | 0 | — |
| `item_t::ITEM_LETTER_STAMPED` | 0 | — |
| `item_t::ITEM_LABEL` | 0 | — |
| `item_t::ITEM_AMULETOFLOSS` | 0 | — |
| `item_t::ITEM_DOCUMENT_RO` | 0 | — |
| `ResourceTypes_t` | 0 | — |
| `ResourceTypes_t::RESOURCE_BANK_BALANCE` | 0 | — |
| `ResourceTypes_t::RESOURCE_GOLD_EQUIPPED` | 0 | — |
| `ResourceTypes_t::RESOURCE_PREY_WILDCARDS` | 0 | — |
| `ResourceTypes_t::RESOURCE_DAILYREWARD_STREAK` | 0 | — |
| `ResourceTypes_t::RESOURCE_DAILYREWARD_JOKERS` | 0 | — |
| `PlayerFlags` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotUseCombat` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotAttackPlayer` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotAttackMonster` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotBeAttacked` | 0 | — |
| `PlayerFlags::PlayerFlag_CanConvinceAll` | 0 | — |
| `PlayerFlags::PlayerFlag_CanSummonAll` | 0 | — |
| `PlayerFlags::PlayerFlag_CanIllusionAll` | 0 | — |
| `PlayerFlags::PlayerFlag_CanSenseInvisibility` | 0 | — |
| `PlayerFlags::PlayerFlag_IgnoredByMonsters` | 0 | — |
| `PlayerFlags::PlayerFlag_NotGainInFight` | 0 | — |
| `PlayerFlags::PlayerFlag_HasInfiniteMana` | 0 | — |
| `PlayerFlags::PlayerFlag_HasInfiniteSoul` | 0 | — |
| `PlayerFlags::PlayerFlag_HasNoExhaustion` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotUseSpells` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotPickupItem` | 0 | — |
| `PlayerFlags::PlayerFlag_CanAlwaysLogin` | 0 | — |
| `PlayerFlags::PlayerFlag_CanBroadcast` | 0 | — |
| `PlayerFlags::PlayerFlag_CanEditHouses` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotBeBanned` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotBePushed` | 0 | — |
| `PlayerFlags::PlayerFlag_HasInfiniteCapacity` | 0 | — |
| `PlayerFlags::PlayerFlag_CanPushAllCreatures` | 0 | — |
| `PlayerFlags::PlayerFlag_CanTalkRedPrivate` | 0 | — |
| `PlayerFlags::PlayerFlag_CanTalkRedChannel` | 0 | — |
| `PlayerFlags::PlayerFlag_TalkOrangeHelpChannel` | 0 | — |
| `PlayerFlags::PlayerFlag_NotGainExperience` | 0 | — |
| `PlayerFlags::PlayerFlag_NotGainMana` | 0 | — |
| `PlayerFlags::PlayerFlag_NotGainHealth` | 0 | — |
| `PlayerFlags::PlayerFlag_NotGainSkill` | 0 | — |
| `PlayerFlags::PlayerFlag_SetMaxSpeed` | 0 | — |
| `PlayerFlags::PlayerFlag_SpecialVIP` | 0 | — |
| `PlayerFlags::PlayerFlag_NotGenerateLoot` | 0 | — |
| `PlayerFlags::PlayerFlag_IgnoreProtectionZone` | 0 | — |
| `PlayerFlags::PlayerFlag_IgnoreSpellCheck` | 0 | — |
| `PlayerFlags::PlayerFlag_IgnoreWeaponCheck` | 0 | — |
| `PlayerFlags::PlayerFlag_CannotBeMuted` | 0 | — |
| `PlayerFlags::PlayerFlag_IsAlwaysPremium` | 0 | — |
| `PlayerFlags::PlayerFlag_IgnoreYellCheck` | 0 | — |
| `PlayerFlags::PlayerFlag_IgnoreSendPrivateCheck` | 0 | — |
| `PodiumFlags` | 0 | — |
| `PodiumFlags::PODIUM_SHOW_PLATFORM` | 0 | — |
| `PodiumFlags::PODIUM_SHOW_OUTFIT` | 0 | — |
| `PodiumFlags::PODIUM_SHOW_MOUNT` | 0 | — |
| `ReloadTypes_t` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_ALL` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_ACTIONS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_CHAT` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_CONFIG` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_CREATURESCRIPTS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_EVENTS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_GLOBAL` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_GLOBALEVENTS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_ITEMS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_MONSTERS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_MOUNTS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_MOVEMENTS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_NPCS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_QUESTS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_SCRIPTS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_SPELLS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_TALKACTIONS` | 0 | — |
| `ReloadTypes_t::RELOAD_TYPE_WEAPONS` | 0 | — |
| `MonsterIcon_t` | 0 | — |
| `MonsterIcon_t::MONSTER_ICON_VULNERABLE` | 0 | — |
| `MonsterIcon_t::MONSTER_ICON_WEAKENED` | 0 | — |
| `MonsterIcon_t::MONSTER_ICON_MELEE` | 0 | — |
| `MonsterIcon_t::MONSTER_ICON_INFLUENCED` | 0 | — |
| `MonsterIcon_t::MONSTER_ICON_FIENDISH` | 0 | — |
| `MonsterIcon_t::MONSTER_ICON_FIRST` | 0 | — |
| `MonsterIcon_t::MONSTER_ICON_LAST` | 0 | — |
| `CreatureIcon_t` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_CROSS_WHITE` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_CROSS_WHITE_RED` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_ORB_RED` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_ORB_GREEN` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_ORB_RED_GREEN` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_GEM_GREEN` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_GEM_YELLOW` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_GEM_BLUE` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_GEM_PURPLE` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_GEM_RED` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_PIGEON` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_ENERGY` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_POISON` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_WATER` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_FIRE` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_ICE` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_ARROW_UP` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_ARROW_DOWN` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_WARNING` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_QUESTION` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_CROSS_RED` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_FIRST` | 0 | — |
| `CreatureIcon_t::CREATURE_ICON_LAST` | 0 | — |

### `container.h` (header declarations)

37 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_CONTAINER_H` | 0 | — |
| `ContainerIterator` | 0 | — |
| `ContainerIterator::advance` | 0 | — |
| `Container` | 0 | — |
| `Container::Container` | 0 | — |
| `Container::Container` | 0 | — |
| `Container::Container` | 0 | — |
| `Container::Container` | 0 | — |
| `Container::Container` | 0 | — |
| `Container::clone` | 0 | — |
| `Container::hasContainerParent` | 0 | — |
| `Container::readAttr` | 0 | — |
| `Container::unserializeItemNode` | 0 | — |
| `Container::iterator` | 0 | — |
| `Container::getName` | 0 | — |
| `Container::addItem` | 0 | — |
| `Container::getItemByIndex` | 0 | — |
| `Container::isHoldingItem` | 0 | — |
| `Container::getItemHoldingCount` | 0 | — |
| `Container::getWeight` | 0 | — |
| `Container::queryDestination` | 0 | — |
| `Container::addThing` | 0 | — |
| `Container::addItemBack` | 0 | — |
| `Container::updateThing` | 0 | — |
| `Container::replaceThing` | 0 | — |
| `Container::removeThing` | 0 | — |
| `Container::getThingIndex` | 0 | — |
| `Container::getItemTypeCount` | 0 | — |
| `Container::getAllItemTypeCount` | 0 | — |
| `Container::getItems` | 0 | — |
| `Container::internalRemoveThing` | 0 | — |
| `Container::internalAddThing` | 0 | — |
| `Container::startDecaying` | 0 | — |
| `Container::onAddContainerItem` | 0 | — |
| `Container::onUpdateContainerItem` | 0 | — |
| `Container::onRemoveContainerItem` | 0 | — |
| `Container::updateItemWeight` | 0 | — |

### `container`

33 node(s), 58 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Container::clone` | 2 | `Container::clone` (static); `Item::clone` (static) |
| `Container::getName` | 1 | `Container::getName` (static) |
| `Container::hasContainerParent` | 1 | `Container::hasContainerParent` (static) |
| `Container::addItem` | 1 | `Container::addItem` (static) |
| `Container::readAttr` | 2 | `Container::readAttr` (static); `Item::readAttr` (static) |
| `Container::unserializeItemNode` | 3 | `Container::unserializeItemNode` (static); `Item::CreateItem` (static); `Item::unserializeItemNode` (static) |
| `Container::updateItemWeight` | 1 | `Container::updateItemWeight` (static) |
| `Container::getWeight` | 2 | `Container::getWeight` (static); `Item::getWeight` (static) |
| `Container::getItemByIndex` | 1 | `Container::getItemByIndex` (static) |
| `Container::getItemHoldingCount` | 1 | `Container::getItemHoldingCount` (static) |
| `Container::isHoldingItem` | 1 | `Container::isHoldingItem` (static) |
| `Container::onAddContainerItem` | 1 | `Container::onAddContainerItem` (static) |
| `Container::onUpdateContainerItem` | 1 | `Container::onUpdateContainerItem` (static) |
| `Container::onRemoveContainerItem` | 1 | `Container::onRemoveContainerItem` (static) |
| `Container::queryAdd` | 14 | `Container::addItemBack` (static); `Container::addThing` (static); `Container::getItemTypeCount` (static); `Container::getItems` (static); `Container::getThingIndex` (static); … +9 more |
| `Container::queryMaxCount` | 1 | `Container::queryMaxCount` (static) |
| `Container::queryRemove` | 1 | `Container::queryRemove` (static) |
| `Container::queryDestination` | 1 | `Container::queryDestination` (static) |
| `Container::addThing` | 1 | `Container::addThing` (static) |
| `Container::addItemBack` | 1 | `Container::addItemBack` (static) |
| `Container::updateThing` | 1 | `Container::updateThing` (static) |
| `Container::replaceThing` | 1 | `Container::replaceThing` (static) |
| `Container::removeThing` | 1 | `Container::removeThing` (static) |
| `Container::getThingIndex` | 1 | `Container::getThingIndex` (static) |
| `Container::getItemTypeCount` | 1 | `Container::getItemTypeCount` (static) |
| `Container::getItems` | 1 | `Container::getItems` (static) |
| `Container::postAddNotification` | 2 | `Container::postAddNotification` (static); `Container::postRemoveNotification` (static) |
| `Container::postRemoveNotification` | 6 | `Container::internalAddThing` (static); `Container::internalRemoveThing` (static); `Container::iterator` (static); `Container::postRemoveNotification` (static); `Container::startDecaying` (static); … +1 more |
| `Container::internalRemoveThing` | 1 | `Container::internalRemoveThing` (static) |
| `Container::internalAddThing` | 1 | `Container::internalAddThing` (static) |
| `Container::startDecaying` | 2 | `Container::startDecaying` (static); `Item::startDecaying` (static) |
| `Container::iterator` | 1 | `Container::iterator` (static) |
| `ContainerIterator::advance` | 1 | `ContainerIterator::advance` (static) |

### `creature.h` (header declarations)

285 node(s), 47 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ConditionList` | 0 | — |
| `CreatureEventList` | 0 | — |
| `CreatureIconHashMap` | 0 | — |
| `slots_t` | 0 | — |
| `slots_t::CONST_SLOT_WHEREEVER` | 0 | — |
| `slots_t::CONST_SLOT_HEAD` | 0 | — |
| `slots_t::CONST_SLOT_NECKLACE` | 0 | — |
| `slots_t::CONST_SLOT_BACKPACK` | 0 | — |
| `slots_t::CONST_SLOT_ARMOR` | 0 | — |
| `slots_t::CONST_SLOT_RIGHT` | 0 | — |
| `slots_t::CONST_SLOT_LEFT` | 0 | — |
| `slots_t::CONST_SLOT_LEGS` | 0 | — |
| `slots_t::CONST_SLOT_FEET` | 0 | — |
| `slots_t::CONST_SLOT_RING` | 0 | — |
| `slots_t::CONST_SLOT_AMMO` | 0 | — |
| `slots_t::CONST_SLOT_STORE_INBOX` | 0 | — |
| `slots_t::CONST_SLOT_FIRST` | 0 | — |
| `slots_t::CONST_SLOT_LAST` | 0 | — |
| `FindPathParams` | 0 | — |
| `FindPathParams::fullPathSearch` | 0 | — |
| `FindPathParams::clearSight` | 0 | — |
| `FindPathParams::allowDiagonal` | 0 | — |
| `FindPathParams::keepDistance` | 0 | — |
| `FindPathParams::summonTargetMaster` | 0 | — |
| `FindPathParams::maxSearchDist` | 0 | — |
| `FindPathParams::minTargetDist` | 0 | — |
| `FindPathParams::maxTargetDist` | 0 | — |
| `EVENT_CREATURECOUNT` | 0 | — |
| `EVENT_CREATURE_THINK_INTERVAL` | 0 | — |
| `EVENT_CHECK_CREATURE_INTERVAL` | 0 | — |
| `CREATURE_ID_MIN` | 0 | — |
| `CREATURE_ID_MAX` | 0 | — |
| `FrozenPathingConditionCall` | 0 | — |
| `FrozenPathingConditionCall::FrozenPathingConditionCall` | 0 | — |
| `FrozenPathingConditionCall::operator()` | 0 | — |
| `FrozenPathingConditionCall::isInRange` | 0 | — |
| `FrozenPathingConditionCall::targetPos` | 0 | — |
| `Creature` | 0 | — |
| `Creature::Creature` | 0 | — |
| `Creature::speedA` | 0 | — |
| `Creature::speedB` | 0 | — |
| `Creature::speedC` | 0 | — |
| `Creature::~Creature` | 0 | — |
| `Creature::Creature(const Creature&)` | 0 | — |
| `Creature::operator=` | 0 | — |
| `Creature::getCreature` | 0 | — |
| `Creature::getCreature_const` | 0 | — |
| `Creature::getPlayer` | 0 | — |
| `Creature::getPlayer_const` | 0 | — |
| `Creature::getNpc` | 0 | — |
| `Creature::getNpc_const` | 0 | — |
| `Creature::getMonster` | 0 | — |
| `Creature::getMonster_const` | 0 | — |
| `Creature::getName` | 1 | `Monster::getName` (dynamic/curated) |
| `Creature::getNameDescription` | 1 | `Monster::getNameDescription` (dynamic/curated) |
| `Creature::getDescription` | 2 | `Player::getDescription` (dynamic/curated); `Npc::getDescription` (dynamic/curated) |
| `Creature::getType` | 0 | — |
| `Creature::setID` | 1 | `Player::setID` (dynamic/curated) |
| `Creature::setRemoved` | 0 | — |
| `Creature::getID` | 0 | — |
| `Creature::removeList` | 3 | `Player::removeList` (dynamic/curated); `Monster::removeList` (dynamic/curated); `Npc::removeList` (dynamic/curated) |
| `Creature::addList` | 3 | `Player::addList` (dynamic/curated); `Monster::addList` (dynamic/curated); `Npc::addList` (dynamic/curated) |
| `Creature::canSee_pos` | 0 | — |
| `Creature::canSeeCreature` | 0 | — |
| `Creature::getRace` | 0 | — |
| `Creature::getSkull` | 1 | `Player::getSkull` (dynamic/curated) |
| `Creature::getSkullClient` | 1 | `Player::getSkullClient` (dynamic/curated) |
| `Creature::setSkull` | 0 | — |
| `Creature::getDirection` | 0 | — |
| `Creature::setDirection` | 0 | — |
| `Creature::isHealthHidden` | 0 | — |
| `Creature::setHiddenHealth` | 0 | — |
| `Creature::getThrowRange` | 0 | — |
| `Creature::isPushable` | 0 | — |
| `Creature::isRemoved` | 0 | — |
| `Creature::canSeeInvisibility` | 0 | — |
| `Creature::isInGhostMode` | 0 | — |
| `Creature::canSeeGhostMode` | 1 | `Player::canSeeGhostMode` (dynamic/curated) |
| `Creature::getWalkDelay_dir` | 0 | — |
| `Creature::getWalkDelay` | 0 | — |
| `Creature::getTimeSinceLastMove` | 0 | — |
| `Creature::getEventStepTicks` | 0 | — |
| `Creature::getStepDuration_dir` | 0 | — |
| `Creature::getStepDuration` | 0 | — |
| `Creature::getStepSpeed` | 0 | — |
| `Creature::getSpeed` | 0 | — |
| `Creature::setSpeed` | 0 | — |
| `Creature::setBaseSpeed` | 0 | — |
| `Creature::getBaseSpeed` | 0 | — |
| `Creature::getHealth` | 0 | — |
| `Creature::getMaxHealth` | 0 | — |
| `Creature::isDead` | 0 | — |
| `Creature::setDrunkenness` | 0 | — |
| `Creature::getDrunkenness` | 0 | — |
| `Creature::getCurrentOutfit` | 0 | — |
| `Creature::setCurrentOutfit` | 0 | — |
| `Creature::getDefaultOutfit` | 0 | — |
| `Creature::isInvisible` | 0 | — |
| `Creature::getZone` | 0 | — |
| `Creature::getIcons` | 0 | — |
| `Creature::getIcons_const` | 0 | — |
| `Creature::updateIcons` | 0 | — |
| `Creature::startAutoWalk` | 0 | — |
| `Creature::startAutoWalk_dir` | 0 | — |
| `Creature::startAutoWalk_list` | 0 | — |
| `Creature::addEventWalk` | 0 | — |
| `Creature::stopEventWalk` | 0 | — |
| `Creature::goToFollowCreature` | 3 | `Player::goToFollowCreature` (dynamic/curated); `Monster::goToFollowCreature` (dynamic/curated); `Npc::goToFollowCreature` (dynamic/curated) |
| `Creature::updateFollowCreaturePath` | 0 | — |
| `Creature::onWalk_dir` | 0 | — |
| `Creature::onWalkAborted` | 1 | `Player::onWalkAborted` (dynamic/curated) |
| `Creature::onWalkComplete` | 2 | `Player::onWalkComplete` (dynamic/curated); `Monster::onWalkComplete` (dynamic/curated) |
| `Creature::getFollowCreature` | 0 | — |
| `Creature::setFollowCreature` | 0 | — |
| `Creature::removeFollowCreature` | 0 | — |
| `Creature::canFollowCreature` | 0 | — |
| `Creature::isFollowingCreature` | 0 | — |
| `Creature::onFollowCreature` | 0 | — |
| `Creature::onUnfollowCreature` | 0 | — |
| `Creature::isFollower` | 0 | — |
| `Creature::addFollower` | 0 | — |
| `Creature::removeFollower` | 0 | — |
| `Creature::removeFollowers` | 0 | — |
| `Creature::releaseFollowers` | 0 | — |
| `Creature::updateFollowersPaths` | 0 | — |
| `Creature::getAttackedCreature` | 0 | — |
| `Creature::setAttackedCreature` | 0 | — |
| `Creature::removeAttackedCreature` | 0 | — |
| `Creature::canAttackCreature` | 0 | — |
| `Creature::isAttackingCreature` | 0 | — |
| `Creature::blockHit` | 0 | — |
| `Creature::setMaster` | 0 | — |
| `Creature::removeMaster` | 0 | — |
| `Creature::isSummon` | 0 | — |
| `Creature::getMaster` | 0 | — |
| `Creature::getSummons` | 0 | — |
| `Creature::getArmor` | 1 | `Player::getArmor` (dynamic/curated) |
| `Creature::getDefense` | 1 | `Player::getDefense` (dynamic/curated) |
| `Creature::getAttackFactor` | 1 | `Player::getAttackFactor` (dynamic/curated) |
| `Creature::getDefenseFactor` | 1 | `Player::getDefenseFactor` (dynamic/curated) |
| `Creature::addCondition` | 0 | — |
| `Creature::addCombatCondition` | 0 | — |
| `Creature::removeCondition_typeId` | 0 | — |
| `Creature::removeCondition_type` | 0 | — |
| `Creature::removeCondition_ptr` | 0 | — |
| `Creature::removeCombatCondition` | 0 | — |
| `Creature::getCondition_type` | 0 | — |
| `Creature::getCondition_typeIdSub` | 0 | — |
| `Creature::executeConditions` | 0 | — |
| `Creature::hasCondition` | 0 | — |
| `Creature::isImmune_condition` | 0 | — |
| `Creature::isImmune_combat` | 0 | — |
| `Creature::isSuppress` | 0 | — |
| `Creature::getDamageImmunities` | 0 | — |
| `Creature::getConditionImmunities` | 0 | — |
| `Creature::getConditionSuppressions` | 0 | — |
| `Creature::isAttackable` | 1 | `Player::isAttackable` (dynamic/curated) |
| `Creature::changeHealth` | 0 | — |
| `Creature::gainHealth` | 0 | — |
| `Creature::drainHealth` | 0 | — |
| `Creature::challengeCreature` | 1 | `Monster::challengeCreature` (dynamic/curated) |
| `Creature::getKillers` | 0 | — |
| `Creature::onDeath` | 0 | — |
| `Creature::getGainedExperience` | 0 | — |
| `Creature::addDamagePoints` | 0 | — |
| `Creature::hasBeenAttacked` | 0 | — |
| `Creature::onAddCondition` | 0 | — |
| `Creature::onAddCombatCondition` | 0 | — |
| `Creature::onEndCondition` | 0 | — |
| `Creature::onTickCondition` | 0 | — |
| `Creature::onCombatRemoveCondition` | 0 | — |
| `Creature::onAttackedCreature` | 1 | `Player::onAttackedCreature` (dynamic/curated) |
| `Creature::onAttacked` | 0 | — |
| `Creature::onAttackedCreatureDrainHealth` | 0 | — |
| `Creature::onTargetCreatureGainHealth` | 1 | `Player::onTargetCreatureGainHealth` (dynamic/curated) |
| `Creature::onKilledCreature` | 0 | — |
| `Creature::onGainExperience` | 0 | — |
| `Creature::onAttackedCreatureBlockHit` | 1 | `Player::onAttackedCreatureBlockHit` (dynamic/curated) |
| `Creature::onBlockHit` | 1 | `Player::onBlockHit` (dynamic/curated) |
| `Creature::onChangeZone` | 0 | — |
| `Creature::onAttackedCreatureChangeZone` | 0 | — |
| `Creature::onIdleStatus` | 0 | — |
| `Creature::getCreatureLight` | 0 | — |
| `Creature::setNormalCreatureLight` | 0 | — |
| `Creature::setCreatureLight` | 0 | — |
| `Creature::onThink` | 0 | — |
| `Creature::forceUpdatePath` | 0 | — |
| `Creature::onAttacking` | 0 | — |
| `Creature::onWalk` | 0 | — |
| `Creature::getNextStep` | 0 | — |
| `Creature::onUpdateTileItem` | 1 | `Player::onUpdateTileItem` (dynamic/curated) |
| `Creature::onRemoveTileItem` | 1 | `Player::onRemoveTileItem` (dynamic/curated) |
| `Creature::onCreatureAppear` | 3 | `Player::onCreatureAppear` (dynamic/curated); `Monster::onCreatureAppear` (dynamic/curated); `Npc::onCreatureAppear` (dynamic/curated) |
| `Creature::onRemoveCreature` | 0 | — |
| `Creature::onCreatureMove` | 0 | — |
| `Creature::onAttackedCreatureDisappear` | 2 | `Player::onAttackedCreatureDisappear` (dynamic/curated); `Monster::onAttackedCreatureDisappear` (dynamic/curated) |
| `Creature::onFollowCreatureDisappear` | 1 | `Player::onFollowCreatureDisappear` (dynamic/curated) |
| `Creature::onCreatureSay` | 2 | `Monster::onCreatureSay` (dynamic/curated); `Npc::onCreatureSay` (dynamic/curated) |
| `Creature::getCombatValues` | 1 | `Monster::getCombatValues` (dynamic/curated) |
| `Creature::getSummonCount` | 0 | — |
| `Creature::setDropLoot` | 0 | — |
| `Creature::setSkillLoss` | 0 | — |
| `Creature::setUseDefense` | 0 | — |
| `Creature::setMovementBlocked` | 0 | — |
| `Creature::isMovementBlocked` | 0 | — |
| `Creature::registerCreatureEvent` | 0 | — |
| `Creature::unregisterCreatureEvent` | 0 | — |
| `Creature::getParent` | 0 | — |
| `Creature::setParent` | 0 | — |
| `Creature::getPosition` | 0 | — |
| `Creature::getTile` | 0 | — |
| `Creature::getTile_const` | 0 | — |
| `Creature::getLastPosition` | 0 | — |
| `Creature::setLastPosition` | 0 | — |
| `Creature::canSee_static` | 0 | — |
| `Creature::getDamageRatio` | 0 | — |
| `Creature::getPathTo_fpp` | 0 | — |
| `Creature::getPathTo_basic` | 0 | — |
| `Creature::incrementReferenceCounter` | 0 | — |
| `Creature::decrementReferenceCounter` | 0 | — |
| `Creature::setStorageValue` | 0 | — |
| `Creature::getStorageValue` | 0 | — |
| `Creature::getStorageMap` | 0 | — |
| `Creature::CountBlock_t` | 0 | — |
| `Creature::CountBlock_t::total` | 0 | — |
| `Creature::CountBlock_t::ticks` | 0 | — |
| `Creature::position` | 0 | — |
| `Creature::CountMap` | 0 | — |
| `Creature::damageMap` | 0 | — |
| `Creature::summons` | 0 | — |
| `Creature::eventsList` | 0 | — |
| `Creature::conditions` | 0 | — |
| `Creature::creatureIcons` | 0 | — |
| `Creature::listWalkDir` | 0 | — |
| `Creature::tile` | 0 | — |
| `Creature::attackedCreature` | 0 | — |
| `Creature::master` | 0 | — |
| `Creature::followCreature` | 0 | — |
| `Creature::followers` | 0 | — |
| `Creature::lastStep` | 0 | — |
| `Creature::lastPathUpdate` | 0 | — |
| `Creature::referenceCounter` | 0 | — |
| `Creature::id` | 0 | — |
| `Creature::scriptEventsBitField` | 0 | — |
| `Creature::eventWalk` | 0 | — |
| `Creature::walkUpdateTicks` | 0 | — |
| `Creature::lastHitCreatureId` | 0 | — |
| `Creature::blockCount` | 0 | — |
| `Creature::blockTicks` | 0 | — |
| `Creature::lastStepCost` | 0 | — |
| `Creature::baseSpeed` | 0 | — |
| `Creature::varSpeed` | 0 | — |
| `Creature::health` | 0 | — |
| `Creature::healthMax` | 0 | — |
| `Creature::drunkenness` | 0 | — |
| `Creature::currentOutfit` | 0 | — |
| `Creature::defaultOutfit` | 0 | — |
| `Creature::currentMount` | 0 | — |
| `Creature::lastPosition` | 0 | — |
| `Creature::internalLight` | 0 | — |
| `Creature::direction` | 0 | — |
| `Creature::skull` | 0 | — |
| `Creature::isInternalRemoved` | 0 | — |
| `Creature::creatureCheck` | 0 | — |
| `Creature::inCheckCreaturesVector` | 0 | — |
| `Creature::skillLoss` | 0 | — |
| `Creature::lootDrop` | 0 | — |
| `Creature::cancelNextWalk` | 0 | — |
| `Creature::hasFollowPath` | 0 | — |
| `Creature::hiddenHealth` | 0 | — |
| `Creature::canUseDefense` | 0 | — |
| `Creature::movementBlocked` | 0 | — |
| `Creature::hasEventRegistered` | 0 | — |
| `Creature::getCreatureEvents` | 0 | — |
| `Creature::onCreatureDisappear` | 0 | — |
| `Creature::doAttacking` | 2 | `Player::doAttacking` (dynamic/curated); `Monster::doAttacking` (dynamic/curated) |
| `Creature::hasExtraSwing` | 0 | — |
| `Creature::getLostExperience` | 0 | — |
| `Creature::dropLoot` | 1 | `Monster::dropLoot` (dynamic/curated) |
| `Creature::getLookCorpse` | 1 | `Player::getLookCorpse` (dynamic/curated) |
| `Creature::getPathSearchParams` | 0 | — |
| `Creature::death` | 2 | `Player::death` (dynamic/curated); `Monster::death` (dynamic/curated) |
| `Creature::dropCorpse` | 0 | — |
| `Creature::getCorpse` | 0 | — |
| `Creature::storageMap` | 0 | — |

### `creature`

92 node(s), 160 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Creature::canSee` | 3 | `Player::canSee` (dynamic/curated); `Monster::canSee` (dynamic/curated); `Npc::canSee` (dynamic/curated) |
| `Creature::canSee` | 4 | `Creature::canSee` (static); `Player::canSee` (dynamic/curated); `Monster::canSee` (dynamic/curated); `Npc::canSee` (dynamic/curated) |
| `Creature::canSeeCreature` | 2 | `Creature::canSeeCreature` (static); `Player::canSeeCreature` (dynamic/curated) |
| `Creature::setSkull` | 2 | `Creature::setSkull` (static); `Game::updateCreatureSkull` (static) |
| `Creature::getTimeSinceLastMove` | 1 | `Creature::getTimeSinceLastMove` (static) |
| `Creature::getWalkDelay` | 0 | — |
| `Creature::getWalkDelay` | 1 | `Creature::getWalkDelay` (static) |
| `Creature::onThink` | 4 | `Creature::onThink` (static); `Player::onThink` (dynamic/curated); `Monster::onThink` (dynamic/curated); `Npc::onThink` (dynamic/curated) |
| `Creature::forceUpdatePath` | 2 | `Creature::forceUpdatePath` (static); `Game::updateCreatureWalk` (static) |
| `Creature::onAttacking` | 2 | `Creature::onAttacking` (static); `Game::isSightClear` (static) |
| `Creature::onIdleStatus` | 2 | `Creature::onIdleStatus` (static); `Player::onIdleStatus` (dynamic/curated) |
| `Creature::onWalk` | 2 | `Player::onWalk` (dynamic/curated); `Monster::onWalk` (dynamic/curated) |
| `Creature::onWalk` | 4 | `Creature::onWalk` (static); `Game::internalCreatureSay` (static); `Player::onWalk` (dynamic/curated); `Monster::onWalk` (dynamic/curated) |
| `Creature::getNextStep` | 3 | `Creature::getNextStep` (static); `Monster::getNextStep` (dynamic/curated); `Npc::getNextStep` (dynamic/curated) |
| `Creature::startAutoWalk` | 0 | — |
| `Creature::startAutoWalk` | 0 | — |
| `Creature::startAutoWalk` | 1 | `Creature::startAutoWalk` (static) |
| `Creature::addEventWalk` | 2 | `Creature::addEventWalk` (static); `Game::checkCreatureWalk` (static) |
| `Creature::stopEventWalk` | 1 | `Creature::stopEventWalk` (static) |
| `Creature::updateIcons` | 1 | `Creature::updateIcons` (static) |
| `Creature::onRemoveCreature` | 4 | `Creature::onRemoveCreature` (static); `Player::onRemoveCreature` (dynamic/curated); `Monster::onRemoveCreature` (dynamic/curated); `Npc::onRemoveCreature` (dynamic/curated) |
| `Creature::onCreatureDisappear` | 1 | `Creature::onCreatureDisappear` (static) |
| `Creature::updateFollowCreaturePath` | 1 | `Creature::updateFollowCreaturePath` (static) |
| `Creature::onChangeZone` | 2 | `Creature::onChangeZone` (static); `Player::onChangeZone` (dynamic/curated) |
| `Creature::onAttackedCreatureChangeZone` | 2 | `Creature::onAttackedCreatureChangeZone` (static); `Player::onAttackedCreatureChangeZone` (dynamic/curated) |
| `Creature::onCreatureMove` | 7 | `Creature::onCreatureMove` (static); `tfs::events::creature::onChangeZone` (static); `Game::checkCreatureAttack` (static); `Game::removeCreature` (static); `Player::onCreatureMove` (dynamic/curated); … +2 more |
| `Creature::getKillers` | 2 | `Creature::getKillers` (static); `Game::getCreatureByID` (static) |
| `Creature::onDeath` | 3 | `Creature::onDeath` (static); `Game::getCreatureByID` (static); `Game::removeCreature` (static) |
| `Creature::dropCorpse` | 6 | `Creature::dropCorpse` (static); `Game::addMagicEffect` (static); `Game::internalAddItem` (static); `Game::startDecay` (static); `Item::CreateItem` (static); … +1 more |
| `Creature::hasBeenAttacked` | 1 | `Creature::hasBeenAttacked` (static) |
| `Creature::getCorpse` | 4 | `Creature::getCorpse` (static); `Item::CreateItem` (static); `Player::getCorpse` (dynamic/curated); `Monster::getCorpse` (dynamic/curated) |
| `Creature::changeHealth` | 5 | `Creature::changeHealth` (static); `Game::addCreatureHealth` (static); `Game::executeDeath` (static); `Player::changeHealth` (dynamic/curated); `Monster::changeHealth` (dynamic/curated) |
| `Creature::gainHealth` | 1 | `Creature::gainHealth` (static) |
| `Creature::drainHealth` | 3 | `Creature::drainHealth` (static); `Player::drainHealth` (dynamic/curated); `Monster::drainHealth` (dynamic/curated) |
| `Creature::blockHit` | 3 | `Creature::blockHit` (static); `Player::blockHit` (dynamic/curated); `Monster::blockHit` (dynamic/curated) |
| `Creature::setAttackedCreature` | 2 | `Creature::setAttackedCreature` (static); `Player::setAttackedCreature` (dynamic/curated) |
| `Creature::removeAttackedCreature` | 2 | `Creature::removeAttackedCreature` (static); `Player::removeAttackedCreature` (dynamic/curated) |
| `Creature::canAttackCreature` | 1 | `Creature::canAttackCreature` (static) |
| `Creature::getPathSearchParams` | 3 | `Creature::getPathSearchParams` (static); `Player::getPathSearchParams` (dynamic/curated); `Monster::getPathSearchParams` (dynamic/curated) |
| `Creature::setFollowCreature` | 2 | `Creature::setFollowCreature` (static); `Player::setFollowCreature` (dynamic/curated) |
| `Creature::removeFollowCreature` | 1 | `Creature::removeFollowCreature` (static) |
| `Creature::canFollowCreature` | 1 | `Creature::canFollowCreature` (static) |
| `Creature::onFollowCreature` | 1 | `Creature::onFollowCreature` (static) |
| `Creature::onUnfollowCreature` | 2 | `Creature::onUnfollowCreature` (static); `Player::onUnfollowCreature` (dynamic/curated) |
| `Creature::isFollower` | 1 | `Creature::isFollower` (static) |
| `Creature::addFollower` | 1 | `Creature::addFollower` (static) |
| `Creature::removeFollower` | 1 | `Creature::removeFollower` (static) |
| `Creature::removeFollowers` | 1 | `Creature::removeFollowers` (static) |
| `Creature::releaseFollowers` | 1 | `Creature::releaseFollowers` (static) |
| `Creature::updateFollowersPaths` | 2 | `Creature::updateFollowersPaths` (static); `Game::updateCreatureWalk` (static) |
| `Creature::getDamageRatio` | 1 | `Creature::getDamageRatio` (static) |
| `Creature::getGainedExperience` | 2 | `Creature::getGainedExperience` (static); `Player::getGainedExperience` (dynamic/curated) |
| `Creature::addDamagePoints` | 1 | `Creature::addDamagePoints` (static) |
| `Creature::onAddCondition` | 3 | `Creature::onAddCondition` (static); `Player::onAddCondition` (dynamic/curated); `Monster::onAddCondition` (dynamic/curated) |
| `Creature::onAddCombatCondition` | 2 | `Creature::onAddCombatCondition` (static); `Player::onAddCombatCondition` (dynamic/curated) |
| `Creature::onEndCondition` | 3 | `Creature::onEndCondition` (static); `Player::onEndCondition` (dynamic/curated); `Monster::onEndCondition` (dynamic/curated) |
| `Creature::onTickCondition` | 1 | `Creature::onTickCondition` (static) |
| `Creature::onCombatRemoveCondition` | 2 | `Creature::onCombatRemoveCondition` (static); `Player::onCombatRemoveCondition` (dynamic/curated) |
| `Creature::onAttacked` | 2 | `Creature::onAttacked` (static); `Player::onAttacked` (dynamic/curated) |
| `Creature::onAttackedCreatureDrainHealth` | 2 | `Creature::onAttackedCreatureDrainHealth` (static); `Player::onAttackedCreatureDrainHealth` (dynamic/curated) |
| `Creature::onKilledCreature` | 2 | `Creature::onKilledCreature` (static); `Player::onKilledCreature` (dynamic/curated) |
| `Creature::onGainExperience` | 2 | `Creature::onGainExperience` (static); `Player::onGainExperience` (dynamic/curated) |
| `Creature::setMaster` | 1 | `Creature::setMaster` (static) |
| `Creature::addCondition` | 2 | `Creature::addCondition` (static); `Game::forceAddCondition` (static) |
| `Creature::addCombatCondition` | 1 | `Creature::addCombatCondition` (static) |
| `Creature::removeCondition` | 0 | — |
| `Creature::removeCondition` | 0 | — |
| `Creature::removeCombatCondition` | 1 | `Creature::removeCombatCondition` (static) |
| `Creature::removeCondition` | 2 | `Creature::removeCondition` (static); `Game::forceRemoveCondition` (static) |
| `Creature::getCondition` | 0 | — |
| `Creature::getCondition` | 1 | `Creature::getCondition` (static) |
| `Creature::executeConditions` | 1 | `Creature::executeConditions` (static) |
| `Creature::hasCondition` | 1 | `Creature::hasCondition` (static) |
| `Creature::isImmune` | 1 | `Player::isImmune` (dynamic/curated) |
| `Creature::isImmune` | 2 | `Creature::isImmune` (static); `Player::isImmune` (dynamic/curated) |
| `Creature::isSuppress` | 1 | `Creature::isSuppress` (static) |
| `Creature::getStepDuration` | 0 | — |
| `Creature::getStepDuration` | 1 | `Creature::getStepDuration` (static) |
| `Creature::getEventStepTicks` | 1 | `Creature::getEventStepTicks` (static) |
| `Creature::getCreatureLight` | 2 | `Creature::getCreatureLight` (static); `Player::getCreatureLight` (dynamic/curated) |
| `Creature::setCreatureLight` | 1 | `Creature::setCreatureLight` (static) |
| `Creature::setNormalCreatureLight` | 2 | `Creature::setNormalCreatureLight` (static); `Monster::setNormalCreatureLight` (dynamic/curated) |
| `Creature::registerCreatureEvent` | 2 | `Creature::registerCreatureEvent` (static); `CreatureEvents::getEventByName` (static) |
| `Creature::unregisterCreatureEvent` | 2 | `Creature::unregisterCreatureEvent` (static); `CreatureEvents::getEventByName` (static) |
| `Creature::getCreatureEvents` | 1 | `Creature::getCreatureEvents` (static) |
| `FrozenPathingConditionCall::isInRange` | 1 | `FrozenPathingConditionCall::isInRange` (static) |
| `FrozenPathingConditionCall::operator` | 0 | — |
| `Creature::isInvisible` | 1 | `Creature::isInvisible` (static) |
| `Creature::getPathTo` | 0 | — |
| `Creature::getPathTo` | 1 | `Creature::getPathTo` (static) |
| `Creature::setStorageValue` | 3 | `Creature::setStorageValue` (static); `tfs::events::creature::onUpdateStorage` (static); `Player::setStorageValue` (dynamic/curated) |
| `Creature::getStorageValue` | 1 | `Creature::getStorageValue` (static) |

### `creatureevent.h` (header declarations)

16 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_CREATUREEVENT_H` | 0 | — |
| `CreatureEventType_t` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_NONE` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_LOGIN` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_LOGOUT` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_RECONNECT` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_THINK` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_PREPAREDEATH` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_DEATH` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_KILL` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_ADVANCE` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_MODALWINDOW` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_TEXTEDIT` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_HEALTHCHANGE` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_MANACHANGE` | 0 | — |
| `CreatureEventType_t::CREATURE_EVENT_EXTENDED_OPCODE` | 0 | — |

### `creatureevent`

29 node(s), 104 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `CreatureEvents::clear` | 1 | `CreatureEvents::clear` (static) |
| `CreatureEvents::removeInvalidEvents` | 1 | `CreatureEvents::removeInvalidEvents` (static) |
| `CreatureEvents::getScriptInterface` | 1 | `CreatureEvents::getScriptInterface` (static) |
| `CreatureEvents::getEvent` | 1 | `CreatureEvents::getEvent` (static) |
| `CreatureEvents::registerEvent` | 1 | `CreatureEvents::registerEvent` (static) |
| `CreatureEvents::registerLuaEvent` | 1 | `CreatureEvents::registerLuaEvent` (static) |
| `CreatureEvents::getEventByName` | 1 | `CreatureEvents::getEventByName` (static) |
| `CreatureEvents::playerLogin` | 1 | `CreatureEvents::playerLogin` (static) |
| `CreatureEvents::playerLogout` | 1 | `CreatureEvents::playerLogout` (static) |
| `CreatureEvents::playerReconnect` | 1 | `CreatureEvents::playerReconnect` (static) |
| `CreatureEvents::playerAdvance` | 1 | `CreatureEvents::playerAdvance` (static) |
| `CreatureEvent::configureEvent` | 1 | `CreatureEvent::configureEvent` (static) |
| `CreatureEvent::getScriptEventName` | 1 | `CreatureEvent::getScriptEventName` (static) |
| `CreatureEvent::copyEvent` | 1 | `CreatureEvent::copyEvent` (static) |
| `CreatureEvent::clearEvent` | 1 | `CreatureEvent::clearEvent` (static) |
| `CreatureEvent::executeOnLogin` | 6 | `CreatureEvent::executeOnLogin` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeOnLogout` | 6 | `CreatureEvent::executeOnLogout` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeOnReconnect` | 6 | `CreatureEvent::executeOnReconnect` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeOnThink` | 6 | `CreatureEvent::executeOnThink` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeOnPrepareDeath` | 6 | `CreatureEvent::executeOnPrepareDeath` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeOnDeath` | 8 | `CreatureEvent::executeOnDeath` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::pushThing` (static); `tfs::lua::reserveScriptEnv` (static); … +3 more |
| `CreatureEvent::executeAdvance` | 6 | `CreatureEvent::executeAdvance` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeOnKill` | 6 | `CreatureEvent::executeOnKill` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeModalWindow` | 6 | `CreatureEvent::executeModalWindow` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `CreatureEvent::executeTextEdit` | 8 | `CreatureEvent::executeTextEdit` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushString` (static); `tfs::lua::pushThing` (static); `tfs::lua::reserveScriptEnv` (static); … +3 more |
| `pushCombatDamage` | 0 | — |
| `CreatureEvent::executeHealthChange` | 9 | `CreatureEvent::executeHealthChange` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::popString` (static); `tfs::lua::protectedCall` (static); `tfs::lua::reserveScriptEnv` (static); … +4 more |
| `CreatureEvent::executeManaChange` | 9 | `CreatureEvent::executeManaChange` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::popString` (static); `tfs::lua::protectedCall` (static); `tfs::lua::reserveScriptEnv` (static); … +4 more |
| `CreatureEvent::executeExtendedOpcode` | 7 | `CreatureEvent::executeExtendedOpcode` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushString` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); … +2 more |

### `database.h` (header declarations)

40 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DBResult_ptr` | 0 | — |
| `tfs::detail` | 0 | — |
| `tfs::detail::MysqlDeleter` | 0 | — |
| `tfs::detail::MysqlDeleter::operator()(MYSQL*)` | 0 | — |
| `tfs::detail::MysqlDeleter::operator()(MYSQL_RES*)` | 0 | — |
| `tfs::detail::Mysql_ptr` | 0 | — |
| `tfs::detail::MysqlResult_ptr` | 0 | — |
| `Database` | 0 | — |
| `Database::getInstance` | 0 | — |
| `Database::escapeString` | 0 | — |
| `Database::getLastInsertId` | 0 | — |
| `Database::getClientVersion` | 0 | — |
| `Database::getMaxPacketSize` | 0 | — |
| `Database::handle` | 0 | — |
| `Database::databaseLock` | 0 | — |
| `Database::maxPacketSize` | 0 | — |
| `Database::retryQueries` | 0 | — |
| `DBResult` | 0 | — |
| `DBResult::DBResult(const DBResult&)` | 0 | — |
| `DBResult::operator=` | 0 | — |
| `DBResult::getNumber` | 0 | — |
| `DBResult::handle` | 0 | — |
| `DBResult::row` | 0 | — |
| `DBResult::listNames` | 0 | — |
| `DBInsert` | 0 | — |
| `DBInsert::query` | 0 | — |
| `DBInsert::values` | 0 | — |
| `DBInsert::length` | 0 | — |
| `DBTransaction` | 0 | — |
| `DBTransaction::DBTransaction` | 0 | — |
| `DBTransaction::~DBTransaction` | 0 | — |
| `DBTransaction::DBTransaction(const DBTransaction&)` | 0 | — |
| `DBTransaction::operator=` | 0 | — |
| `DBTransaction::begin` | 0 | — |
| `DBTransaction::commit` | 0 | — |
| `DBTransaction::TransactionStates_t` | 0 | — |
| `DBTransaction::STATE_NO_START` | 0 | — |
| `DBTransaction::STATE_START` | 0 | — |
| `DBTransaction::STATE_COMMIT` | 0 | — |
| `DBTransaction::state` | 0 | — |

### `database`

18 node(s), 15 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `connectToDatabase` | 0 | — |
| `isLostConnectionError` | 0 | — |
| `executeQuery` | 0 | — |
| `Database::connect` | 1 | `Database::connect` (static) |
| `Database::beginTransaction` | 1 | `Database::beginTransaction` (static) |
| `Database::rollback` | 1 | `Database::rollback` (static) |
| `Database::commit` | 1 | `Database::commit` (static) |
| `Database::executeQuery` | 1 | `Database::executeQuery` (static) |
| `Database::storeQuery` | 1 | `Database::storeQuery` (static) |
| `Database::escapeBlob` | 1 | `Database::escapeBlob` (static) |
| `DBResult::DBResult` | 1 | `DBResult::DBResult` (static) |
| `DBResult::getString` | 1 | `DBResult::getString` (static) |
| `DBResult::hasNext` | 1 | `DBResult::hasNext` (static) |
| `DBResult::next` | 1 | `DBResult::next` (static) |
| `DBInsert::DBInsert` | 1 | `DBInsert::DBInsert` (static) |
| `DBInsert::addRow(const std::string&)` | 1 | `Database::getInstance` (static) |
| `DBInsert::addRow(std::ostringstream&)` | 0 | — |
| `DBInsert::execute` | 2 | `DBInsert::execute` (static); `Database::getInstance` (static) |

### `databasemanager.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DatabaseManager` | 0 | — |

### `databasemanager`

9 node(s), 16 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DatabaseManager::optimizeTables` | 2 | `Database::getInstance` (static); `DatabaseManager::optimizeTables` (static) |
| `DatabaseManager::tableExists` | 2 | `Database::getInstance` (static); `DatabaseManager::tableExists` (static) |
| `DatabaseManager::isDatabaseSetup` | 2 | `Database::getInstance` (static); `DatabaseManager::isDatabaseSetup` (static) |
| `DatabaseManager::getDatabaseVersion` | 2 | `Database::getInstance` (static); `DatabaseManager::getDatabaseVersion` (static) |
| `server_config.config` | 0 | — |
| `DatabaseManager::updateDatabase` | 4 | `DatabaseManager::updateDatabase` (static); `tfs::lua::getBoolean` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::resetScriptEnv` (static) |
| `DatabaseManager::getDatabaseConfig` | 2 | `Database::getInstance` (static); `DatabaseManager::getDatabaseConfig` (static) |
| `server_config.value` | 0 | — |
| `DatabaseManager::registerDatabaseConfig` | 2 | `Database::getInstance` (static); `DatabaseManager::registerDatabaseConfig` (static) |

### `databasetasks.h` (header declarations)

13 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DatabaseTask` | 0 | — |
| `DatabaseTask::DatabaseTask` | 0 | — |
| `DatabaseTask::query` | 0 | — |
| `DatabaseTask::callback` | 0 | — |
| `DatabaseTask::store` | 0 | — |
| `DatabaseTasks` | 0 | — |
| `DatabaseTasks::DatabaseTasks` | 0 | — |
| `DatabaseTasks::db` | 0 | — |
| `DatabaseTasks::thread` | 0 | — |
| `DatabaseTasks::tasks` | 0 | — |
| `DatabaseTasks::taskLock` | 0 | — |
| `DatabaseTasks::taskSignal` | 0 | — |
| `g_databaseTasks` | 0 | — |

### `databasetasks`

7 node(s), 7 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_dispatcher` | 0 | — |
| `DatabaseTasks::start` | 1 | `DatabaseTasks::start` (static) |
| `DatabaseTasks::threadMain` | 1 | `DatabaseTasks::threadMain` (static) |
| `DatabaseTasks::addTask` | 1 | `DatabaseTasks::addTask` (static) |
| `DatabaseTasks::runTask` | 2 | `DatabaseTasks::runTask` (static); `Dispatcher::addTask` (static) |
| `DatabaseTasks::flush` | 1 | `DatabaseTasks::flush` (static) |
| `DatabaseTasks::shutdown` | 1 | `DatabaseTasks::shutdown` (static) |

### `definitions.h` (header declarations)

5 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_DEFINITIONS_H` | 0 | — |
| `BOOST_ASIO_NO_DEPRECATED` | 0 | — |
| `NOMINMAX` | 0 | — |
| `WIN32_LEAN_AND_MEAN` | 0 | — |
| `HAS_ITERATOR_DEBUGGING` | 0 | — |

### `depotchest.h` (header declarations)

6 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DepotChest` | 0 | — |
| `DepotChest::maxDepotItems` | 0 | — |
| `DepotChest::canRemove` | 0 | — |
| `DepotChest::setMaxDepotItems` | 0 | — |
| `DepotChest_ptr` | 0 | — |
| `FS_DEPOTCHEST_H` | 0 | — |

### `depotchest`

9 node(s), 6 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DepotChest::DepotChest` | 1 | `DepotChest::DepotChest` (static) |
| `DepotChest::getParent` | 0 | — |
| `DepotChest::postAddNotification` | 0 | — |
| `DepotChest::postRemoveNotification` | 0 | — |
| `DepotChest::queryAdd` | 0 | — |
| `DepotChest::queryAdd` | 2 | `Container::queryAdd` (static); `DepotChest::queryAdd` (static) |
| `DepotChest::postAddNotification` | 1 | `DepotChest::postAddNotification` (static) |
| `DepotChest::postRemoveNotification` | 1 | `DepotChest::postRemoveNotification` (static) |
| `DepotChest::getParent` | 1 | `DepotChest::getParent` (static) |

### `depotlocker.h` (header declarations)

10 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DepotLocker` | 0 | — |
| `DepotLocker::depotId` | 0 | — |
| `DepotLocker::canRemove` | 0 | — |
| `DepotLocker::getDepotId` | 0 | — |
| `DepotLocker::getDepotLocker (const)` | 0 | — |
| `DepotLocker::getDepotLocker (non-const)` | 0 | — |
| `DepotLocker::queryAdd` | 0 | — |
| `DepotLocker::setDepotId` | 0 | — |
| `DepotLocker_ptr` | 0 | — |
| `FS_DEPOTLOCKER_H` | 0 | — |

### `depotlocker`

9 node(s), 6 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `DepotLocker::DepotLocker` | 1 | `DepotLocker::DepotLocker` (static) |
| `DepotLocker::postAddNotification` | 0 | — |
| `DepotLocker::postRemoveNotification` | 0 | — |
| `DepotLocker::readAttr` | 0 | — |
| `DepotLocker::removeInbox` | 0 | — |
| `DepotLocker::readAttr` | 2 | `DepotLocker::readAttr` (static); `Item::readAttr` (static) |
| `DepotLocker::postAddNotification` | 1 | `DepotLocker::postAddNotification` (static) |
| `DepotLocker::postRemoveNotification` | 1 | `DepotLocker::postRemoveNotification` (static) |
| `DepotLocker::removeInbox` | 1 | `DepotLocker::removeInbox` (static) |

### `enums.h` (header declarations)

457 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_ENUMS_H` | 0 | — |
| `RuleViolationType_t` | 0 | — |
| `RuleViolationType_t::REPORT_TYPE_NAME` | 0 | — |
| `RuleViolationType_t::REPORT_TYPE_STATEMENT` | 0 | — |
| `RuleViolationType_t::REPORT_TYPE_BOT` | 0 | — |
| `RuleViolationReasons_t` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_NAMEINAPPROPRIATE` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_NAMEPOORFORMATTED` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_NAMEADVERTISING` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_NAMEUNFITTING` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_NAMERULEVIOLATION` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_INSULTINGSTATEMENT` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_SPAMMING` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_ADVERTISINGSTATEMENT` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_UNFITTINGSTATEMENT` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_LANGUAGESTATEMENT` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_DISCLOSURE` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_RULEVIOLATION` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_STATEMENT_BUGABUSE` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_UNOFFICIALSOFTWARE` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_PRETENDING` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_HARASSINGOWNERS` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_FALSEINFO` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_ACCOUNTSHARING` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_STEALINGDATA` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_SERVICEATTACKING` | 0 | — |
| `RuleViolationReasons_t::REPORT_REASON_SERVICEAGREEMENT` | 0 | — |
| `ThreadState` | 0 | — |
| `ThreadState::THREAD_STATE_RUNNING` | 0 | — |
| `ThreadState::THREAD_STATE_CLOSING` | 0 | — |
| `ThreadState::THREAD_STATE_TERMINATED` | 0 | — |
| `itemAttrTypes` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_NONE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_ACTIONID` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_UNIQUEID` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DESCRIPTION` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_TEXT` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DATE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_WRITER` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_NAME` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_ARTICLE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_PLURALNAME` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_WEIGHT` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_ATTACK` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DEFENSE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_EXTRADEFENSE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_ARMOR` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_HITCHANCE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_SHOOTRANGE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_OWNER` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DURATION` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DECAYSTATE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_CORPSEOWNER` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_CHARGES` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_FLUIDTYPE` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DOORID` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DECAYTO` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_WRAPID` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_STOREITEM` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_ATTACK_SPEED` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_OPENCONTAINER` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DURATION_MIN` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_DURATION_MAX` | 0 | — |
| `itemAttrTypes::ITEM_ATTRIBUTE_CUSTOM` | 0 | — |
| `VipStatus_t` | 0 | — |
| `VipStatus_t::VIPSTATUS_OFFLINE` | 0 | — |
| `VipStatus_t::VIPSTATUS_ONLINE` | 0 | — |
| `VipStatus_t::VIPSTATUS_PENDING` | 0 | — |
| `VipStatus_t::VIPSTATUS_TRAINING` | 0 | — |
| `MarketAction_t` | 0 | — |
| `MarketAction_t::MARKETACTION_BUY` | 0 | — |
| `MarketAction_t::MARKETACTION_SELL` | 0 | — |
| `MarketRequest_t` | 0 | — |
| `MarketRequest_t::MARKETREQUEST_OWN_HISTORY` | 0 | — |
| `MarketRequest_t::MARKETREQUEST_OWN_OFFERS` | 0 | — |
| `MarketRequest_t::MARKETREQUEST_ITEM` | 0 | — |
| `MarketOfferState_t` | 0 | — |
| `MarketOfferState_t::OFFERSTATE_ACTIVE` | 0 | — |
| `MarketOfferState_t::OFFERSTATE_CANCELLED` | 0 | — |
| `MarketOfferState_t::OFFERSTATE_EXPIRED` | 0 | — |
| `MarketOfferState_t::OFFERSTATE_ACCEPTED` | 0 | — |
| `MarketOfferState_t::OFFERSTATE_ACCEPTEDEX` | 0 | — |
| `ChannelEvent_t` | 0 | — |
| `ChannelEvent_t::CHANNELEVENT_JOIN` | 0 | — |
| `ChannelEvent_t::CHANNELEVENT_LEAVE` | 0 | — |
| `ChannelEvent_t::CHANNELEVENT_INVITE` | 0 | — |
| `ChannelEvent_t::CHANNELEVENT_EXCLUDE` | 0 | — |
| `CreatureType_t` | 0 | — |
| `CreatureType_t::CREATURETYPE_PLAYER` | 0 | — |
| `CreatureType_t::CREATURETYPE_MONSTER` | 0 | — |
| `CreatureType_t::CREATURETYPE_NPC` | 0 | — |
| `CreatureType_t::CREATURETYPE_SUMMON_OWN` | 0 | — |
| `CreatureType_t::CREATURETYPE_SUMMON_OTHERS` | 0 | — |
| `CreatureType_t::CREATURETYPE_HIDDEN` | 0 | — |
| `OperatingSystem_t` | 0 | — |
| `OperatingSystem_t::CLIENTOS_NONE` | 0 | — |
| `OperatingSystem_t::CLIENTOS_LINUX` | 0 | — |
| `OperatingSystem_t::CLIENTOS_WINDOWS` | 0 | — |
| `OperatingSystem_t::CLIENTOS_FLASH` | 0 | — |
| `OperatingSystem_t::CLIENTOS_QT_LINUX` | 0 | — |
| `OperatingSystem_t::CLIENTOS_QT_WINDOWS` | 0 | — |
| `OperatingSystem_t::CLIENTOS_QT_MAC` | 0 | — |
| `OperatingSystem_t::CLIENTOS_QT_LINUX2` | 0 | — |
| `OperatingSystem_t::CLIENTOS_OTCLIENT_LINUX` | 0 | — |
| `OperatingSystem_t::CLIENTOS_OTCLIENT_WINDOWS` | 0 | — |
| `OperatingSystem_t::CLIENTOS_OTCLIENT_MAC` | 0 | — |
| `SpellGroup_t` | 0 | — |
| `SpellGroup_t::SPELLGROUP_NONE` | 0 | — |
| `SpellGroup_t::SPELLGROUP_ATTACK` | 0 | — |
| `SpellGroup_t::SPELLGROUP_HEALING` | 0 | — |
| `SpellGroup_t::SPELLGROUP_SUPPORT` | 0 | — |
| `SpellGroup_t::SPELLGROUP_SPECIAL` | 0 | — |
| `SpellGroup_t::SPELLGROUP_CRIPPLING` | 0 | — |
| `SpellGroup_t::SPELLGROUP_FOCUS` | 0 | — |
| `SpellGroup_t::SPELLGROUP_ULTIMATESTRIKES` | 0 | — |
| `SpellType_t` | 0 | — |
| `SpellType_t::SPELL_UNDEFINED` | 0 | — |
| `SpellType_t::SPELL_INSTANT` | 0 | — |
| `SpellType_t::SPELL_RUNE` | 0 | — |
| `AccountType_t` | 0 | — |
| `AccountType_t::ACCOUNT_TYPE_NORMAL` | 0 | — |
| `AccountType_t::ACCOUNT_TYPE_TUTOR` | 0 | — |
| `AccountType_t::ACCOUNT_TYPE_SENIORTUTOR` | 0 | — |
| `AccountType_t::ACCOUNT_TYPE_GAMEMASTER` | 0 | — |
| `AccountType_t::ACCOUNT_TYPE_COMMUNITYMANAGER` | 0 | — |
| `AccountType_t::ACCOUNT_TYPE_GOD` | 0 | — |
| `RaceType_t` | 0 | — |
| `RaceType_t::RACE_NONE` | 0 | — |
| `RaceType_t::RACE_VENOM` | 0 | — |
| `RaceType_t::RACE_BLOOD` | 0 | — |
| `RaceType_t::RACE_UNDEAD` | 0 | — |
| `RaceType_t::RACE_FIRE` | 0 | — |
| `RaceType_t::RACE_ENERGY` | 0 | — |
| `RaceType_t::RACE_INK` | 0 | — |
| `CombatType_t` | 0 | — |
| `CombatType_t::COMBAT_NONE` | 0 | — |
| `CombatType_t::COMBAT_PHYSICALDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_ENERGYDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_EARTHDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_FIREDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_UNDEFINEDDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_LIFEDRAIN` | 0 | — |
| `CombatType_t::COMBAT_MANADRAIN` | 0 | — |
| `CombatType_t::COMBAT_HEALING` | 0 | — |
| `CombatType_t::COMBAT_DROWNDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_ICEDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_HOLYDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_DEATHDAMAGE` | 0 | — |
| `CombatType_t::COMBAT_COUNT` | 0 | — |
| `CombatParam_t` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_TYPE` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_EFFECT` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_DISTANCEEFFECT` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_BLOCKSHIELD` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_BLOCKARMOR` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_TARGETCASTERORTOPMOST` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_CREATEITEM` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_AGGRESSIVE` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_DISPEL` | 0 | — |
| `CombatParam_t::COMBAT_PARAM_USECHARGES` | 0 | — |
| `CallBackParam_t` | 0 | — |
| `CallBackParam_t::CALLBACK_PARAM_LEVELMAGICVALUE` | 0 | — |
| `CallBackParam_t::CALLBACK_PARAM_SKILLVALUE` | 0 | — |
| `CallBackParam_t::CALLBACK_PARAM_TARGETTILE` | 0 | — |
| `CallBackParam_t::CALLBACK_PARAM_TARGETCREATURE` | 0 | — |
| `ConditionParam_t` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_OWNER` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_TICKS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_HEALTHGAIN` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_HEALTHTICKS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_MANAGAIN` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_MANATICKS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_DELAYED` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SPEED` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_LIGHT_LEVEL` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_LIGHT_COLOR` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SOULGAIN` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SOULTICKS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_MINVALUE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_MAXVALUE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_STARTVALUE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_TICKINTERVAL` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_FORCEUPDATE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_MELEE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_FIST` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_CLUB` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_SWORD` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_AXE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_DISTANCE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_SHIELD` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_FISHING` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_STAT_MAXHITPOINTS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_STAT_MAXMANAPOINTS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_STAT_MAGICPOINTS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_STAT_MAXHITPOINTSPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_STAT_MAXMANAPOINTSPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_STAT_MAGICPOINTSPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_PERIODICDAMAGE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_MELEEPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_FISTPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_CLUBPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_SWORDPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_AXEPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_DISTANCEPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_SHIELDPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SKILL_FISHINGPERCENT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_BUFF_SPELL` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SUBID` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_FIELD` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_DISABLE_DEFENSE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SPECIALSKILL_CRITICALHITCHANCE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SPECIALSKILL_CRITICALHITAMOUNT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SPECIALSKILL_LIFELEECHCHANCE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SPECIALSKILL_LIFELEECHAMOUNT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SPECIALSKILL_MANALEECHCHANCE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_SPECIALSKILL_MANALEECHAMOUNT` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_AGGRESSIVE` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_DRUNKENNESS` | 0 | — |
| `ConditionParam_t::CONDITION_PARAM_MANASHIELD_BREAKABLE` | 0 | — |
| `BlockType_t` | 0 | — |
| `BlockType_t::BLOCK_NONE` | 0 | — |
| `BlockType_t::BLOCK_DEFENSE` | 0 | — |
| `BlockType_t::BLOCK_ARMOR` | 0 | — |
| `BlockType_t::BLOCK_IMMUNITY` | 0 | — |
| `skills_t` | 0 | — |
| `skills_t::SKILL_FIST` | 0 | — |
| `skills_t::SKILL_CLUB` | 0 | — |
| `skills_t::SKILL_SWORD` | 0 | — |
| `skills_t::SKILL_AXE` | 0 | — |
| `skills_t::SKILL_DISTANCE` | 0 | — |
| `skills_t::SKILL_SHIELD` | 0 | — |
| `skills_t::SKILL_FISHING` | 0 | — |
| `skills_t::SKILL_MAGLEVEL` | 0 | — |
| `skills_t::SKILL_LEVEL` | 0 | — |
| `skills_t::SKILL_FIRST` | 0 | — |
| `skills_t::SKILL_LAST` | 0 | — |
| `stats_t` | 0 | — |
| `stats_t::STAT_MAXHITPOINTS` | 0 | — |
| `stats_t::STAT_MAXMANAPOINTS` | 0 | — |
| `stats_t::STAT_SOULPOINTS` | 0 | — |
| `stats_t::STAT_MAGICPOINTS` | 0 | — |
| `stats_t::STAT_FIRST` | 0 | — |
| `stats_t::STAT_LAST` | 0 | — |
| `SpecialSkills_t` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_CRITICALHITCHANCE` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_CRITICALHITAMOUNT` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_LIFELEECHCHANCE` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_LIFELEECHAMOUNT` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_MANALEECHCHANCE` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_MANALEECHAMOUNT` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_FIRST` | 0 | — |
| `SpecialSkills_t::SPECIALSKILL_LAST` | 0 | — |
| `formulaType_t` | 0 | — |
| `formulaType_t::COMBAT_FORMULA_UNDEFINED` | 0 | — |
| `formulaType_t::COMBAT_FORMULA_LEVELMAGIC` | 0 | — |
| `formulaType_t::COMBAT_FORMULA_SKILL` | 0 | — |
| `formulaType_t::COMBAT_FORMULA_DAMAGE` | 0 | — |
| `ConditionType_t` | 0 | — |
| `ConditionType_t::CONDITION_NONE` | 0 | — |
| `ConditionType_t::CONDITION_POISON` | 0 | — |
| `ConditionType_t::CONDITION_FIRE` | 0 | — |
| `ConditionType_t::CONDITION_ENERGY` | 0 | — |
| `ConditionType_t::CONDITION_BLEEDING` | 0 | — |
| `ConditionType_t::CONDITION_HASTE` | 0 | — |
| `ConditionType_t::CONDITION_PARALYZE` | 0 | — |
| `ConditionType_t::CONDITION_OUTFIT` | 0 | — |
| `ConditionType_t::CONDITION_INVISIBLE` | 0 | — |
| `ConditionType_t::CONDITION_LIGHT` | 0 | — |
| `ConditionType_t::CONDITION_MANASHIELD` | 0 | — |
| `ConditionType_t::CONDITION_INFIGHT` | 0 | — |
| `ConditionType_t::CONDITION_DRUNK` | 0 | — |
| `ConditionType_t::CONDITION_EXHAUST_WEAPON` | 0 | — |
| `ConditionType_t::CONDITION_REGENERATION` | 0 | — |
| `ConditionType_t::CONDITION_SOUL` | 0 | — |
| `ConditionType_t::CONDITION_DROWN` | 0 | — |
| `ConditionType_t::CONDITION_MUTED` | 0 | — |
| `ConditionType_t::CONDITION_CHANNELMUTEDTICKS` | 0 | — |
| `ConditionType_t::CONDITION_YELLTICKS` | 0 | — |
| `ConditionType_t::CONDITION_ATTRIBUTES` | 0 | — |
| `ConditionType_t::CONDITION_FREEZING` | 0 | — |
| `ConditionType_t::CONDITION_DAZZLED` | 0 | — |
| `ConditionType_t::CONDITION_CURSED` | 0 | — |
| `ConditionType_t::CONDITION_EXHAUST_COMBAT` | 0 | — |
| `ConditionType_t::CONDITION_EXHAUST_HEAL` | 0 | — |
| `ConditionType_t::CONDITION_PACIFIED` | 0 | — |
| `ConditionType_t::CONDITION_SPELLCOOLDOWN` | 0 | — |
| `ConditionType_t::CONDITION_SPELLGROUPCOOLDOWN` | 0 | — |
| `ConditionType_t::CONDITION_ROOT` | 0 | — |
| `ConditionType_t::CONDITION_MANASHIELD_BREAKABLE` | 0 | — |
| `ConditionId_t` | 0 | — |
| `ConditionId_t::CONDITIONID_DEFAULT` | 0 | — |
| `ConditionId_t::CONDITIONID_COMBAT` | 0 | — |
| `ConditionId_t::CONDITIONID_HEAD` | 0 | — |
| `ConditionId_t::CONDITIONID_NECKLACE` | 0 | — |
| `ConditionId_t::CONDITIONID_BACKPACK` | 0 | — |
| `ConditionId_t::CONDITIONID_ARMOR` | 0 | — |
| `ConditionId_t::CONDITIONID_RIGHT` | 0 | — |
| `ConditionId_t::CONDITIONID_LEFT` | 0 | — |
| `ConditionId_t::CONDITIONID_LEGS` | 0 | — |
| `ConditionId_t::CONDITIONID_FEET` | 0 | — |
| `ConditionId_t::CONDITIONID_RING` | 0 | — |
| `ConditionId_t::CONDITIONID_AMMO` | 0 | — |
| `PlayerSex_t` | 0 | — |
| `PlayerSex_t::PLAYERSEX_FEMALE` | 0 | — |
| `PlayerSex_t::PLAYERSEX_MALE` | 0 | — |
| `PlayerSex_t::PLAYERSEX_LAST` | 0 | — |
| `ReturnValue` | 0 | — |
| `ReturnValue::RETURNVALUE_NOERROR` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTPOSSIBLE` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTENOUGHROOM` | 0 | — |
| `ReturnValue::RETURNVALUE_PLAYERISPZLOCKED` | 0 | — |
| `ReturnValue::RETURNVALUE_PLAYERISNOTINVITED` | 0 | — |
| `ReturnValue::RETURNVALUE_CANNOTTHROW` | 0 | — |
| `ReturnValue::RETURNVALUE_THEREISNOWAY` | 0 | — |
| `ReturnValue::RETURNVALUE_DESTINATIONOUTOFREACH` | 0 | — |
| `ReturnValue::RETURNVALUE_CREATUREBLOCK` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTMOVEABLE` | 0 | — |
| `ReturnValue::RETURNVALUE_DROPTWOHANDEDITEM` | 0 | — |
| `ReturnValue::RETURNVALUE_BOTHHANDSNEEDTOBEFREE` | 0 | — |
| `ReturnValue::RETURNVALUE_CANONLYUSEONEWEAPON` | 0 | — |
| `ReturnValue::RETURNVALUE_NEEDEXCHANGE` | 0 | — |
| `ReturnValue::RETURNVALUE_CANNOTBEDRESSED` | 0 | — |
| `ReturnValue::RETURNVALUE_PUTTHISOBJECTINYOURHAND` | 0 | — |
| `ReturnValue::RETURNVALUE_PUTTHISOBJECTINBOTHHANDS` | 0 | — |
| `ReturnValue::RETURNVALUE_TOOFARAWAY` | 0 | — |
| `ReturnValue::RETURNVALUE_FIRSTGODOWNSTAIRS` | 0 | — |
| `ReturnValue::RETURNVALUE_FIRSTGOUPSTAIRS` | 0 | — |
| `ReturnValue::RETURNVALUE_CONTAINERNOTENOUGHROOM` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTENOUGHCAPACITY` | 0 | — |
| `ReturnValue::RETURNVALUE_CANNOTPICKUP` | 0 | — |
| `ReturnValue::RETURNVALUE_THISISIMPOSSIBLE` | 0 | — |
| `ReturnValue::RETURNVALUE_DEPOTISFULL` | 0 | — |
| `ReturnValue::RETURNVALUE_CREATUREDOESNOTEXIST` | 0 | — |
| `ReturnValue::RETURNVALUE_CANNOTUSETHISOBJECT` | 0 | — |
| `ReturnValue::RETURNVALUE_PLAYERWITHTHISNAMEISNOTONLINE` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTREQUIREDLEVELTOUSERUNE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUAREALREADYTRADING` | 0 | — |
| `ReturnValue::RETURNVALUE_THISPLAYERISALREADYTRADING` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUMAYNOTLOGOUTDURINGAFIGHT` | 0 | — |
| `ReturnValue::RETURNVALUE_DIRECTPLAYERSHOOT` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTENOUGHLEVEL` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTENOUGHMAGICLEVEL` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTENOUGHMANA` | 0 | — |
| `ReturnValue::RETURNVALUE_NOTENOUGHSOUL` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUAREEXHAUSTED` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUCANNOTUSEOBJECTSTHATFAST` | 0 | — |
| `ReturnValue::RETURNVALUE_PLAYERISNOTREACHABLE` | 0 | — |
| `ReturnValue::RETURNVALUE_CANONLYUSETHISRUNEONCREATURES` | 0 | — |
| `ReturnValue::RETURNVALUE_ACTIONNOTPERMITTEDINPROTECTIONZONE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUMAYNOTATTACKTHISPLAYER` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUMAYNOTATTACKAPERSONINPROTECTIONZONE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUMAYNOTATTACKAPERSONWHILEINPROTECTIONZONE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUMAYNOTATTACKTHISCREATURE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUCANONLYUSEITONCREATURES` | 0 | — |
| `ReturnValue::RETURNVALUE_CREATUREISNOTREACHABLE` | 0 | — |
| `ReturnValue::RETURNVALUE_TURNSECUREMODETOATTACKUNMARKEDPLAYERS` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUNEEDPREMIUMACCOUNT` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUNEEDTOLEARNTHISSPELL` | 0 | — |
| `ReturnValue::RETURNVALUE_YOURVOCATIONCANNOTUSETHISSPELL` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUNEEDAWEAPONTOUSETHISSPELL` | 0 | — |
| `ReturnValue::RETURNVALUE_PLAYERISPZLOCKEDLEAVEPVPZONE` | 0 | — |
| `ReturnValue::RETURNVALUE_PLAYERISPZLOCKEDENTERPVPZONE` | 0 | — |
| `ReturnValue::RETURNVALUE_ACTIONNOTPERMITTEDINANOPVPZONE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUCANNOTLOGOUTHERE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUNEEDAMAGICITEMTOCASTSPELL` | 0 | — |
| `ReturnValue::RETURNVALUE_NAMEISTOOAMBIGUOUS` | 0 | — |
| `ReturnValue::RETURNVALUE_CANONLYUSEONESHIELD` | 0 | — |
| `ReturnValue::RETURNVALUE_NOPARTYMEMBERSINRANGE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUARENOTTHEOWNER` | 0 | — |
| `ReturnValue::RETURNVALUE_TRADEPLAYERFARAWAY` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUDONTOWNTHISHOUSE` | 0 | — |
| `ReturnValue::RETURNVALUE_TRADEPLAYERALREADYOWNSAHOUSE` | 0 | — |
| `ReturnValue::RETURNVALUE_TRADEPLAYERHIGHESTBIDDER` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUCANNOTTRADETHISHOUSE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUDONTHAVEREQUIREDPROFESSION` | 0 | — |
| `ReturnValue::RETURNVALUE_CANNOTMOVEITEMISNOTSTOREITEM` | 0 | — |
| `ReturnValue::RETURNVALUE_ITEMCANNOTBEMOVEDTHERE` | 0 | — |
| `ReturnValue::RETURNVALUE_YOUCANNOTUSETHISBED` | 0 | — |
| `ReturnValue::RETURNVALUE_QUIVERAMMOONLY` | 0 | — |
| `SpeechBubble_t` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_NONE` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_NORMAL` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_TRADE` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_QUEST` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_COMPASS` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_NORMAL2` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_NORMAL3` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_HIRELING` | 0 | — |
| `SpeechBubble_t::SPEECHBUBBLE_LAST` | 0 | — |
| `MapMark_t` | 0 | — |
| `MapMark_t::MAPMARK_TICK` | 0 | — |
| `MapMark_t::MAPMARK_QUESTION` | 0 | — |
| `MapMark_t::MAPMARK_EXCLAMATION` | 0 | — |
| `MapMark_t::MAPMARK_STAR` | 0 | — |
| `MapMark_t::MAPMARK_CROSS` | 0 | — |
| `MapMark_t::MAPMARK_TEMPLE` | 0 | — |
| `MapMark_t::MAPMARK_KISS` | 0 | — |
| `MapMark_t::MAPMARK_SHOVEL` | 0 | — |
| `MapMark_t::MAPMARK_SWORD` | 0 | — |
| `MapMark_t::MAPMARK_FLAG` | 0 | — |
| `MapMark_t::MAPMARK_LOCK` | 0 | — |
| `MapMark_t::MAPMARK_BAG` | 0 | — |
| `MapMark_t::MAPMARK_SKULL` | 0 | — |
| `MapMark_t::MAPMARK_DOLLAR` | 0 | — |
| `MapMark_t::MAPMARK_REDNORTH` | 0 | — |
| `MapMark_t::MAPMARK_REDSOUTH` | 0 | — |
| `MapMark_t::MAPMARK_REDEAST` | 0 | — |
| `MapMark_t::MAPMARK_REDWEST` | 0 | — |
| `MapMark_t::MAPMARK_GREENNORTH` | 0 | — |
| `MapMark_t::MAPMARK_GREENSOUTH` | 0 | — |
| `Outfit_t` | 0 | — |
| `LightInfo` | 0 | — |
| `ShopInfo` | 0 | — |
| `ShopInfo::ShopInfo` | 0 | — |
| `MarketOffer` | 0 | — |
| `MarketOfferEx` | 0 | — |
| `MarketOfferEx::MarketOfferEx` | 0 | — |
| `HistoryMarketOffer` | 0 | — |
| `MarketStatistics` | 0 | — |
| `ModalWindow` | 0 | — |
| `CombatOrigin` | 0 | — |
| `CombatOrigin::ORIGIN_NONE` | 0 | — |
| `CombatOrigin::ORIGIN_CONDITION` | 0 | — |
| `CombatOrigin::ORIGIN_SPELL` | 0 | — |
| `CombatOrigin::ORIGIN_MELEE` | 0 | — |
| `CombatOrigin::ORIGIN_RANGED` | 0 | — |
| `CombatOrigin::ORIGIN_WAND` | 0 | — |
| `CombatOrigin::ORIGIN_REFLECT` | 0 | — |
| `CombatDamage` | 0 | — |
| `CombatDamage::type` | 0 | — |
| `CombatDamage::value` | 0 | — |
| `MonstersEvent_t` | 0 | — |
| `MonstersEvent_t::MONSTERS_EVENT_NONE` | 0 | — |
| `MonstersEvent_t::MONSTERS_EVENT_THINK` | 0 | — |
| `MonstersEvent_t::MONSTERS_EVENT_APPEAR` | 0 | — |
| `MonstersEvent_t::MONSTERS_EVENT_DISAPPEAR` | 0 | — |
| `MonstersEvent_t::MONSTERS_EVENT_MOVE` | 0 | — |
| `MonstersEvent_t::MONSTERS_EVENT_SAY` | 0 | — |
| `Reflect` | 0 | — |
| `Reflect::Reflect` | 0 | — |
| `Reflect::Reflect` | 0 | — |
| `ClientDamageType` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_PHYSICAL` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_FIRE` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_EARTH` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_ENERGY` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_ICE` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_HOLY` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_DEATH` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_HEALING` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_DROWN` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_LIFEDRAIN` | 0 | — |
| `ClientDamageType::CLIENT_DAMAGETYPE_UNDEFINED` | 0 | — |
| `DamageAnalyzerImpactType` | 0 | — |
| `DamageAnalyzerImpactType::HEALING` | 0 | — |
| `DamageAnalyzerImpactType::DEALT` | 0 | — |
| `DamageAnalyzerImpactType::RECEIVED` | 0 | — |

### `events.h` (header declarations)

46 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `EventInfoId` | 0 | — |
| `EventInfoId::CREATURE_ONHEAR` | 0 | — |
| `EventInfoId::MONSTER_ONSPAWN` | 0 | — |
| `tfs::events::load` | 0 | — |
| `tfs::events::reload` | 0 | — |
| `tfs::events::getScriptId` | 0 | — |
| `tfs::events::creature::onChangeOutfit` | 0 | — |
| `tfs::events::creature::onAreaCombat` | 0 | — |
| `tfs::events::creature::onTargetCombat` | 0 | — |
| `tfs::events::creature::onHear` | 0 | — |
| `tfs::events::creature::onChangeZone` | 0 | — |
| `tfs::events::creature::onUpdateStorage` | 0 | — |
| `tfs::events::party::onJoin` | 0 | — |
| `tfs::events::party::onLeave` | 0 | — |
| `tfs::events::party::onDisband` | 0 | — |
| `tfs::events::party::onShareExperience` | 0 | — |
| `tfs::events::party::onInvite` | 0 | — |
| `tfs::events::party::onRevokeInvitation` | 0 | — |
| `tfs::events::party::onPassLeadership` | 0 | — |
| `tfs::events::player::onBrowseField` | 0 | — |
| `tfs::events::player::onLook` | 0 | — |
| `tfs::events::player::onLookInBattleList` | 0 | — |
| `tfs::events::player::onLookInTrade` | 0 | — |
| `tfs::events::player::onLookInShop` | 0 | — |
| `tfs::events::player::onLookInMarket` | 0 | — |
| `tfs::events::player::onMoveItem` | 0 | — |
| `tfs::events::player::onItemMoved` | 0 | — |
| `tfs::events::player::onMoveCreature` | 0 | — |
| `tfs::events::player::onReportRuleViolation` | 0 | — |
| `tfs::events::player::onReportBug` | 0 | — |
| `tfs::events::player::onRotateItem` | 0 | — |
| `tfs::events::player::onTurn` | 0 | — |
| `tfs::events::player::onTradeRequest` | 0 | — |
| `tfs::events::player::onTradeAccept` | 0 | — |
| `tfs::events::player::onTradeCompleted` | 0 | — |
| `tfs::events::player::onPodiumRequest` | 0 | — |
| `tfs::events::player::onPodiumEdit` | 0 | — |
| `tfs::events::player::onGainExperience` | 0 | — |
| `tfs::events::player::onLoseExperience` | 0 | — |
| `tfs::events::player::onGainSkillTries` | 0 | — |
| `tfs::events::player::onWrapItem` | 0 | — |
| `tfs::events::player::onInventoryUpdate` | 0 | — |
| `tfs::events::player::onNetworkMessage` | 0 | — |
| `tfs::events::player::onSpellCheck` | 0 | — |
| `tfs::events::monster::onDropLoot` | 0 | — |
| `tfs::events::monster::onSpawn` | 0 | — |

### `events`

93 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `(anonymous)::scriptInterface` | 0 | — |
| `(anonymous)::CreatureHandlers` | 0 | — |
| `(anonymous)::CreatureHandlers::onChangeOutfit` | 0 | — |
| `(anonymous)::CreatureHandlers::onAreaCombat` | 0 | — |
| `(anonymous)::CreatureHandlers::onTargetCombat` | 0 | — |
| `(anonymous)::CreatureHandlers::onHear` | 0 | — |
| `(anonymous)::CreatureHandlers::onChangeZone` | 0 | — |
| `(anonymous)::CreatureHandlers::onUpdateStorage` | 0 | — |
| `(anonymous)::creatureHandlers` | 0 | — |
| `(anonymous)::PartyHandlers` | 0 | — |
| `(anonymous)::PartyHandlers::onJoin` | 0 | — |
| `(anonymous)::PartyHandlers::onLeave` | 0 | — |
| `(anonymous)::PartyHandlers::onDisband` | 0 | — |
| `(anonymous)::PartyHandlers::onShareExperience` | 0 | — |
| `(anonymous)::PartyHandlers::onInvite` | 0 | — |
| `(anonymous)::PartyHandlers::onRevokeInvitation` | 0 | — |
| `(anonymous)::PartyHandlers::onPassLeadership` | 0 | — |
| `(anonymous)::partyHandlers` | 0 | — |
| `(anonymous)::PlayerHandlers` | 0 | — |
| `(anonymous)::PlayerHandlers::onBrowseField` | 0 | — |
| `(anonymous)::PlayerHandlers::onLook` | 0 | — |
| `(anonymous)::PlayerHandlers::onLookInBattleList` | 0 | — |
| `(anonymous)::PlayerHandlers::onLookInTrade` | 0 | — |
| `(anonymous)::PlayerHandlers::onLookInShop` | 0 | — |
| `(anonymous)::PlayerHandlers::onLookInMarket` | 0 | — |
| `(anonymous)::PlayerHandlers::onMoveItem` | 0 | — |
| `(anonymous)::PlayerHandlers::onItemMoved` | 0 | — |
| `(anonymous)::PlayerHandlers::onMoveCreature` | 0 | — |
| `(anonymous)::PlayerHandlers::onReportRuleViolation` | 0 | — |
| `(anonymous)::PlayerHandlers::onReportBug` | 0 | — |
| `(anonymous)::PlayerHandlers::onRotateItem` | 0 | — |
| `(anonymous)::PlayerHandlers::onTurn` | 0 | — |
| `(anonymous)::PlayerHandlers::onTradeRequest` | 0 | — |
| `(anonymous)::PlayerHandlers::onTradeAccept` | 0 | — |
| `(anonymous)::PlayerHandlers::onTradeCompleted` | 0 | — |
| `(anonymous)::PlayerHandlers::onPodiumRequest` | 0 | — |
| `(anonymous)::PlayerHandlers::onPodiumEdit` | 0 | — |
| `(anonymous)::PlayerHandlers::onGainExperience` | 0 | — |
| `(anonymous)::PlayerHandlers::onLoseExperience` | 0 | — |
| `(anonymous)::PlayerHandlers::onGainSkillTries` | 0 | — |
| `(anonymous)::PlayerHandlers::onWrapItem` | 0 | — |
| `(anonymous)::PlayerHandlers::onInventoryUpdate` | 0 | — |
| `(anonymous)::PlayerHandlers::onNetworkMessage` | 0 | — |
| `(anonymous)::PlayerHandlers::onSpellCheck` | 0 | — |
| `(anonymous)::playerHandlers` | 0 | — |
| `(anonymous)::MonsterHandlers` | 0 | — |
| `(anonymous)::MonsterHandlers::onDropLoot` | 0 | — |
| `(anonymous)::MonsterHandlers::onSpawn` | 0 | — |
| `(anonymous)::monsterHandlers` | 0 | — |
| `(anonymous)::load_from_xml` | 0 | — |
| `tfs::events::getScriptId` | 0 | — |
| `tfs::events::load` | 0 | — |
| `tfs::events::reload` | 0 | — |
| `tfs::events::creature::onChangeOutfit` | 0 | — |
| `tfs::events::creature::onAreaCombat` | 0 | — |
| `tfs::events::creature::onTargetCombat` | 0 | — |
| `tfs::events::creature::onHear` | 0 | — |
| `tfs::events::creature::onChangeZone` | 0 | — |
| `tfs::events::creature::onUpdateStorage` | 0 | — |
| `tfs::events::party::onJoin` | 0 | — |
| `tfs::events::party::onLeave` | 0 | — |
| `tfs::events::party::onDisband` | 0 | — |
| `tfs::events::party::onInvite` | 0 | — |
| `tfs::events::party::onRevokeInvitation` | 0 | — |
| `tfs::events::party::onPassLeadership` | 0 | — |
| `tfs::events::party::onShareExperience` | 0 | — |
| `tfs::events::player::onBrowseField` | 0 | — |
| `tfs::events::player::onLook` | 0 | — |
| `tfs::events::player::onLookInBattleList` | 0 | — |
| `tfs::events::player::onLookInTrade` | 0 | — |
| `tfs::events::player::onLookInShop` | 0 | — |
| `tfs::events::player::onLookInMarket` | 0 | — |
| `tfs::events::player::onMoveItem` | 0 | — |
| `tfs::events::player::onItemMoved` | 0 | — |
| `tfs::events::player::onMoveCreature` | 0 | — |
| `tfs::events::player::onReportRuleViolation` | 0 | — |
| `tfs::events::player::onReportBug` | 0 | — |
| `tfs::events::player::onRotateItem` | 0 | — |
| `tfs::events::player::onTurn` | 0 | — |
| `tfs::events::player::onTradeRequest` | 0 | — |
| `tfs::events::player::onTradeAccept` | 0 | — |
| `tfs::events::player::onTradeCompleted` | 0 | — |
| `tfs::events::player::onPodiumRequest` | 0 | — |
| `tfs::events::player::onPodiumEdit` | 0 | — |
| `tfs::events::player::onGainExperience` | 0 | — |
| `tfs::events::player::onLoseExperience` | 0 | — |
| `tfs::events::player::onGainSkillTries` | 0 | — |
| `tfs::events::player::onWrapItem` | 0 | — |
| `tfs::events::player::onInventoryUpdate` | 0 | — |
| `tfs::events::player::onNetworkMessage` | 0 | — |
| `tfs::events::player::onSpellCheck` | 0 | — |
| `tfs::events::monster::onSpawn` | 0 | — |
| `tfs::events::monster::onDropLoot` | 0 | — |

### `fileloader.h` (header declarations)

38 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `OTB` | 0 | — |
| `OTB::MappedFile` | 0 | — |
| `OTB::ContentIt` | 0 | — |
| `OTB::Identifier` | 0 | — |
| `OTB::Node` | 0 | — |
| `OTB::Node::ChildrenVector` | 0 | — |
| `OTB::Node::children` | 0 | — |
| `OTB::Node::propsBegin` | 0 | — |
| `OTB::Node::propsEnd` | 0 | — |
| `OTB::Node::type` | 0 | — |
| `OTB::Node::NodeChar` | 0 | — |
| `OTB::Node::NodeChar::ESCAPE` | 0 | — |
| `OTB::Node::NodeChar::START` | 0 | — |
| `OTB::Node::NodeChar::END` | 0 | — |
| `OTB::LoadError` | 0 | — |
| `OTB::InvalidOTBFormat` | 0 | — |
| `OTB::Loader` | 0 | — |
| `OTB::Loader::fileContents` | 0 | — |
| `OTB::Loader::root` | 0 | — |
| `OTB::Loader::propBuffer` | 0 | — |
| `OTB::Loader::Loader` | 0 | — |
| `OTB::Loader::getProps` | 0 | — |
| `OTB::Loader::parseTree` | 0 | — |
| `PropStream` | 0 | — |
| `PropStream::init` | 0 | — |
| `PropStream::size` | 0 | — |
| `PropStream::read<T>` | 0 | — |
| `PropStream::readString` | 0 | — |
| `PropStream::skip` | 0 | — |
| `PropStream::p` | 0 | — |
| `PropStream::end` | 0 | — |
| `PropWriteStream` | 0 | — |
| `PropWriteStream::PropWriteStream` | 0 | — |
| `PropWriteStream::getStream` | 0 | — |
| `PropWriteStream::clear` | 0 | — |
| `PropWriteStream::write<T>` | 0 | — |
| `PropWriteStream::writeString` | 0 | — |
| `PropWriteStream::buffer` | 0 | — |

### `fileloader`

6 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `OTB::wildcard` | 0 | — |
| `OTB::Loader::Loader` | 0 | — |
| `OTB::NodeStack` | 0 | — |
| `OTB::getCurrentNode` | 0 | — |
| `OTB::Loader::parseTree` | 0 | — |
| `OTB::Loader::getProps` | 0 | — |

### `game.h` (header declarations)

33 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_GAME_H` | 0 | — |
| `stackPosType_t` | 0 | — |
| `stackPosType_t::STACKPOS_MOVE` | 0 | — |
| `stackPosType_t::STACKPOS_LOOK` | 0 | — |
| `stackPosType_t::STACKPOS_TOPDOWN_ITEM` | 0 | — |
| `stackPosType_t::STACKPOS_USEITEM` | 0 | — |
| `stackPosType_t::STACKPOS_USETARGET` | 0 | — |
| `WorldType_t` | 0 | — |
| `WorldType_t::WORLD_TYPE_NO_PVP` | 0 | — |
| `WorldType_t::WORLD_TYPE_PVP` | 0 | — |
| `WorldType_t::WORLD_TYPE_PVP_ENFORCED` | 0 | — |
| `GameState_t` | 0 | — |
| `GameState_t::GAME_STATE_STARTUP` | 0 | — |
| `GameState_t::GAME_STATE_INIT` | 0 | — |
| `GameState_t::GAME_STATE_NORMAL` | 0 | — |
| `GameState_t::GAME_STATE_CLOSED` | 0 | — |
| `GameState_t::GAME_STATE_SHUTDOWN` | 0 | — |
| `GameState_t::GAME_STATE_CLOSING` | 0 | — |
| `GameState_t::GAME_STATE_MAINTAIN` | 0 | — |
| `Game` | 0 | — |
| `Game::Game` | 0 | — |
| `Game::Game` | 0 | — |
| `Game::start` | 0 | — |
| `Game::forceAddCondition` | 0 | — |
| `Game::forceRemoveCondition` | 0 | — |
| `Game::loadMainMap` | 0 | — |
| `Game::loadMap` | 0 | — |
| `Game::setWorldType` | 0 | — |
| `Game::internalGetThing` | 0 | — |
| `Game::internalGetPosition` | 0 | — |
| `Game::getTradeErrorDescription` | 0 | — |
| `Game::getCreatureByID` | 0 | — |
| `Game::getMonsterByID` | 0 | — |

### `game`

185 node(s), 278 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Game::start` | 4 | `Game::start` (static); `Game::checkCreatures` (dynamic/curated); `Game::updateCreaturesPath` (dynamic/curated); `Game::checkDecay` (dynamic/curated) |
| `Game::getGameState` | 1 | `Game::getGameState` (static) |
| `Game::setWorldType` | 1 | `Game::setWorldType` (static) |
| `Game::setGameState` | 7 | `Chat::load` (static); `Game::setGameState` (static); `GlobalEvents::save` (static); `GlobalEvents::shutdown` (static); `GlobalEvents::startup` (static); … +2 more |
| `Game::saveGameState` | 3 | `Game::saveGameState` (static); `IOLoginData::savePlayer` (static); `Map::save` (static) |
| `Game::loadMainMap` | 1 | `Game::loadMainMap` (static) |
| `Game::loadMap` | 1 | `Game::loadMap` (static) |
| `Game::internalGetThing` | 0 | — |
| `Game::internalGetThing` | 1 | `Game::internalGetThing` (static) |
| `Game::internalGetPosition` | 1 | `Game::internalGetPosition` (static) |
| `Game::getCreatureByID` | 1 | `Game::getCreatureByID` (static) |
| `Game::getMonsterByID` | 1 | `Game::getMonsterByID` (static) |
| `Game::getNpcByID` | 1 | `Game::getNpcByID` (static) |
| `Game::getPlayerByID` | 1 | `Game::getPlayerByID` (static) |
| `Game::getCreatureByName` | 1 | `Game::getCreatureByName` (static) |
| `Game::getNpcByName` | 1 | `Game::getNpcByName` (static) |
| `Game::getPlayerByName` | 1 | `Game::getPlayerByName` (static) |
| `Game::getPlayerByGUID` | 1 | `Game::getPlayerByGUID` (static) |
| `Game::getPlayerByNameWildcard` | 1 | `Game::getPlayerByNameWildcard` (static) |
| `Game::getPlayerByAccount` | 1 | `Game::getPlayerByAccount` (static) |
| `Game::internalPlaceCreature` | 1 | `Game::internalPlaceCreature` (static) |
| `Game::placeCreature` | 1 | `Game::placeCreature` (static) |
| `Game::removeCreature` | 1 | `Game::removeCreature` (static) |
| `Game::executeDeath` | 1 | `Game::executeDeath` (static) |
| `Game::playerMoveThing` | 1 | `Game::playerMoveThing` (static) |
| `Game::playerMoveCreatureByID` | 1 | `Game::playerMoveCreatureByID` (static) |
| `Game::playerMoveCreature` | 3 | `tfs::events::player::onMoveCreature` (static); `Game::playerMoveCreature` (static); `Spawns::isInZone` (static) |
| `Game::internalMoveCreature` | 0 | — |
| `Game::internalMoveCreature` | 1 | `Game::internalMoveCreature` (static) |
| `Game::playerMoveItemByPlayerID` | 1 | `Game::playerMoveItemByPlayerID` (static) |
| `Game::playerMoveItem` | 1 | `Game::playerMoveItem` (static) |
| `Game::internalMoveItem` | 3 | `tfs::events::player::onItemMoved` (static); `tfs::events::player::onMoveItem` (static); `Game::internalMoveItem` (static) |
| `Game::internalAddItem` | 0 | — |
| `Game::internalAddItem` | 1 | `Game::internalAddItem` (static) |
| `Game::internalRemoveItem` | 1 | `Game::internalRemoveItem` (static) |
| `Game::internalPlayerAddItem` | 2 | `Game::internalPlayerAddItem` (static); `Item::CreateItem` (static) |
| `Game::findItemOfType` | 1 | `Game::findItemOfType` (static) |
| `Game::removeMoney` | 1 | `Game::removeMoney` (static) |
| `Game::addMoney` | 2 | `Game::addMoney` (static); `Item::CreateItem` (static) |
| `Game::transformItem` | 2 | `Game::transformItem` (static); `Item::CreateItem` (static) |
| `Game::internalTeleport` | 1 | `Game::internalTeleport` (static) |
| `searchForItem` | 0 | — |
| `getSlotType` | 0 | — |
| `Game::playerEquipItem` | 1 | `Game::playerEquipItem` (static) |
| `Game::playerMove` | 1 | `Game::playerMove` (static) |
| `Game::playerBroadcastMessage` | 1 | `Game::playerBroadcastMessage` (static) |
| `Game::playerCreatePrivateChannel` | 2 | `Chat::createChannel` (static); `Game::playerCreatePrivateChannel` (static) |
| `Game::playerChannelInvite` | 2 | `Chat::getPrivateChannel` (static); `Game::playerChannelInvite` (static) |
| `Game::playerChannelExclude` | 2 | `Chat::getPrivateChannel` (static); `Game::playerChannelExclude` (static) |
| `Game::playerRequestChannels` | 1 | `Game::playerRequestChannels` (static) |
| `Game::playerOpenChannel` | 2 | `Chat::addUserToChannel` (static); `Game::playerOpenChannel` (static) |
| `Game::playerCloseChannel` | 2 | `Chat::removeUserFromChannel` (static); `Game::playerCloseChannel` (static) |
| `Game::playerOpenPrivateChannel` | 2 | `Game::playerOpenPrivateChannel` (static); `IOLoginData::formatPlayerName` (static) |
| `Game::playerCloseNpcChannel` | 1 | `Game::playerCloseNpcChannel` (static) |
| `Game::playerReceivePing` | 1 | `Game::playerReceivePing` (static) |
| `Game::playerReceivePingBack` | 1 | `Game::playerReceivePingBack` (static) |
| `Game::playerAutoWalk` | 2 | `Game::playerAutoWalk` (static); `Game::internalMoveCreature` (dynamic/curated) |
| `Game::playerStopAutoWalk` | 1 | `Game::playerStopAutoWalk` (static) |
| `Game::playerUseItemEx` | 3 | `Actions::canUse` (static); `Actions::useItemEx` (static); `Game::playerUseItemEx` (static) |
| `Game::playerUseItem` | 3 | `Actions::canUse` (static); `Actions::useItem` (static); `Game::playerUseItem` (static) |
| `Game::playerUseWithCreature` | 3 | `Actions::canUse` (static); `Actions::useItemEx` (static); `Game::playerUseWithCreature` (static) |
| `Game::playerCloseContainer` | 1 | `Game::playerCloseContainer` (static) |
| `Game::playerMoveUpContainer` | 2 | `tfs::events::player::onBrowseField` (static); `Game::playerMoveUpContainer` (static) |
| `Game::playerUpdateContainer` | 1 | `Game::playerUpdateContainer` (static) |
| `Game::playerRotateItem` | 2 | `tfs::events::player::onRotateItem` (static); `Game::playerRotateItem` (static) |
| `Game::playerWriteItem` | 1 | `Game::playerWriteItem` (static) |
| `Game::playerBrowseField` | 2 | `tfs::events::player::onBrowseField` (static); `Game::playerBrowseField` (static) |
| `Game::playerSeekInContainer` | 1 | `Game::playerSeekInContainer` (static) |
| `Game::playerUpdateHouseWindow` | 1 | `Game::playerUpdateHouseWindow` (static) |
| `Game::playerWrapItem` | 2 | `tfs::events::player::onWrapItem` (static); `Game::playerWrapItem` (static) |
| `Game::playerRequestTrade` | 2 | `tfs::events::player::onTradeRequest` (static); `Game::playerRequestTrade` (static) |
| `Game::internalStartTrade` | 1 | `Game::internalStartTrade` (static) |
| `Game::playerAcceptTrade` | 3 | `tfs::events::player::onTradeAccept` (static); `tfs::events::player::onTradeCompleted` (static); `Game::playerAcceptTrade` (static) |
| `Game::getTradeErrorDescription` | 1 | `Game::getTradeErrorDescription` (static) |
| `Game::playerLookInTrade` | 2 | `tfs::events::player::onLookInTrade` (static); `Game::playerLookInTrade` (static) |
| `Game::playerCloseTrade` | 1 | `Game::playerCloseTrade` (static) |
| `Game::internalCloseTrade` | 1 | `Game::internalCloseTrade` (static) |
| `Game::playerPurchaseItem` | 1 | `Game::playerPurchaseItem` (static) |
| `Game::playerSellItem` | 1 | `Game::playerSellItem` (static) |
| `Game::playerCloseShop` | 1 | `Game::playerCloseShop` (static) |
| `Game::playerLookInShop` | 2 | `tfs::events::player::onLookInShop` (static); `Game::playerLookInShop` (static) |
| `Game::playerLookAt` | 2 | `tfs::events::player::onLook` (static); `Game::playerLookAt` (static) |
| `Game::playerLookInBattleList` | 2 | `tfs::events::player::onLookInBattleList` (static); `Game::playerLookInBattleList` (static) |
| `Game::playerCancelAttackAndFollow` | 1 | `Game::playerCancelAttackAndFollow` (static) |
| `Game::playerSetAttackedCreature` | 2 | `Combat::canTargetCreature` (static); `Game::playerSetAttackedCreature` (static) |
| `Game::playerFollowCreature` | 1 | `Game::playerFollowCreature` (static) |
| `Game::playerSetFightModes` | 1 | `Game::playerSetFightModes` (static) |
| `Game::playerRequestAddVip` | 2 | `Game::playerRequestAddVip` (static); `IOLoginData::getGuidByNameEx` (static) |
| `Game::playerRequestRemoveVip` | 1 | `Game::playerRequestRemoveVip` (static) |
| `Game::playerRequestEditVip` | 1 | `Game::playerRequestEditVip` (static) |
| `Game::playerTurn` | 2 | `tfs::events::player::onTurn` (static); `Game::playerTurn` (static) |
| `Game::playerRequestOutfit` | 1 | `Game::playerRequestOutfit` (static) |
| `Game::playerRequestEditPodium` | 2 | `tfs::events::player::onPodiumRequest` (static); `Game::playerRequestEditPodium` (static) |
| `Game::playerEditPodium` | 2 | `tfs::events::player::onPodiumEdit` (static); `Game::playerEditPodium` (static) |
| `Game::playerToggleMount` | 1 | `Game::playerToggleMount` (static) |
| `Game::playerChangeOutfit` | 2 | `Game::playerChangeOutfit` (static); `Outfits::getInstance` (static) |
| `Game::playerSay` | 2 | `Chat::talkToChannel` (static); `Game::playerSay` (static) |
| `Game::playerSaySpell` | 3 | `Game::playerSaySpell` (static); `Spells::playerSaySpell` (static); `TalkActions::playerSaySpell` (static) |
| `Game::playerWhisper` | 1 | `Game::playerWhisper` (static) |
| `Game::playerYell` | 2 | `Condition::createCondition` (static); `Game::playerYell` (static) |
| `Game::playerSpeakTo` | 1 | `Game::playerSpeakTo` (static) |
| `Game::playerSpeakToNpc` | 1 | `Game::playerSpeakToNpc` (static) |
| `Game::canThrowObjectTo` | 1 | `Game::canThrowObjectTo` (static) |
| `Game::isSightClear` | 1 | `Game::isSightClear` (static) |
| `Game::internalCreatureTurn` | 1 | `Game::internalCreatureTurn` (static) |
| `Game::internalCreatureSay` | 3 | `tfs::events::creature::onHear` (static); `Game::checkCreatureWalk` (static); `Game::internalCreatureSay` (static) |
| `Game::checkCreatureWalk` | 1 | `Game::checkCreatureWalk` (static) |
| `Game::updateCreatureWalk` | 1 | `Game::updateCreatureWalk` (static) |
| `Game::checkCreatureAttack` | 1 | `Game::checkCreatureAttack` (static) |
| `Game::addCreatureCheck` | 1 | `Game::addCreatureCheck` (static) |
| `Game::removeCreatureCheck` | 1 | `Game::removeCreatureCheck` (static) |
| `Game::checkCreatures` | 2 | `Game::checkCreatures` (static); `Game::checkCreatures` (dynamic/curated) |
| `Game::updateCreaturesPath` | 2 | `Game::updateCreaturesPath` (static); `Game::updateCreaturesPath` (dynamic/curated) |
| `Game::changeSpeed` | 1 | `Game::changeSpeed` (static) |
| `Game::internalCreatureChangeOutfit` | 2 | `tfs::events::creature::onChangeOutfit` (static); `Game::internalCreatureChangeOutfit` (static) |
| `Game::internalCreatureChangeVisible` | 1 | `Game::internalCreatureChangeVisible` (static) |
| `Game::changeLight` | 1 | `Game::changeLight` (static) |
| `Game::combatBlockHit` | 1 | `Game::combatBlockHit` (static) |
| `Game::combatGetTypeInfo` | 2 | `Game::combatGetTypeInfo` (static); `Item::CreateItem` (static) |
| `Game::combatChangeHealth` | 1 | `Game::combatChangeHealth` (static) |
| `Game::combatChangeMana` | 1 | `Game::combatChangeMana` (static) |
| `Game::addCreatureHealth` | 0 | — |
| `Game::addCreatureHealth` | 1 | `Game::addCreatureHealth` (static) |
| `Game::addMagicEffect` | 0 | — |
| `Game::addMagicEffect` | 1 | `Game::addMagicEffect` (static) |
| `Game::addDistanceEffect` | 0 | — |
| `Game::addDistanceEffect` | 1 | `Game::addDistanceEffect` (static) |
| `Game::startDecay` | 1 | `Game::startDecay` (static) |
| `Game::internalDecayItem` | 1 | `Game::internalDecayItem` (static) |
| `Game::checkDecay` | 2 | `Game::checkDecay` (static); `Game::checkDecay` (dynamic/curated) |
| `Game::shutdown` | 2 | `ConnectionManager::getInstance` (static); `Game::shutdown` (static) |
| `Game::cleanup` | 1 | `Game::cleanup` (static) |
| `Game::ReleaseCreature` | 1 | `Game::ReleaseCreature` (static) |
| `Game::ReleaseItem` | 1 | `Game::ReleaseItem` (static) |
| `Game::broadcastMessage` | 1 | `Game::broadcastMessage` (static) |
| `Game::updateCreatureWalkthrough` | 1 | `Game::updateCreatureWalkthrough` (static) |
| `Game::updateKnownCreature` | 1 | `Game::updateKnownCreature` (static) |
| `Game::updateCreatureSkull` | 1 | `Game::updateCreatureSkull` (static) |
| `Game::updatePlayerShield` | 1 | `Game::updatePlayerShield` (static) |
| `Game::checkPlayersRecord` | 2 | `Game::checkPlayersRecord` (static); `GlobalEvents::getEventMap` (static) |
| `Game::updatePlayersRecord` | 2 | `Database::getInstance` (static); `Game::updatePlayersRecord` (static) |
| `Game::loadPlayersRecord` | 2 | `Database::getInstance` (static); `Game::loadPlayersRecord` (static) |
| `Game::playerInviteToParty` | 2 | `tfs::events::party::onInvite` (static); `Game::playerInviteToParty` (static) |
| `Game::playerJoinParty` | 1 | `Game::playerJoinParty` (static) |
| `Game::playerRevokePartyInvitation` | 1 | `Game::playerRevokePartyInvitation` (static) |
| `Game::playerPassPartyLeadership` | 1 | `Game::playerPassPartyLeadership` (static) |
| `Game::playerLeaveParty` | 1 | `Game::playerLeaveParty` (static) |
| `Game::playerEnableSharedPartyExperience` | 1 | `Game::playerEnableSharedPartyExperience` (static) |
| `Game::sendGuildMotd` | 1 | `Game::sendGuildMotd` (static) |
| `Game::kickPlayer` | 1 | `Game::kickPlayer` (static) |
| `Game::playerReportRuleViolation` | 2 | `tfs::events::player::onReportRuleViolation` (static); `Game::playerReportRuleViolation` (static) |
| `Game::playerDebugAssert` | 1 | `Game::playerDebugAssert` (static) |
| `Game::playerLeaveMarket` | 1 | `Game::playerLeaveMarket` (static) |
| `Game::playerBrowseMarket` | 3 | `tfs::events::player::onLookInMarket` (static); `Game::playerBrowseMarket` (static); `tfs::iomarket::getActiveOffers` (static) |
| `Game::playerBrowseMarketOwnOffers` | 2 | `Game::playerBrowseMarketOwnOffers` (static); `tfs::iomarket::getOwnOffers` (static) |
| `Game::playerBrowseMarketOwnHistory` | 2 | `Game::playerBrowseMarketOwnHistory` (static); `tfs::iomarket::getOwnHistory` (static) |
| `Game::playerCreateMarketOffer` | 4 | `Game::playerCreateMarketOffer` (static); `tfs::iomarket::createOffer` (static); `tfs::iomarket::getActiveOffers` (static); `tfs::iomarket::getPlayerOfferCount` (static) |
| `Game::playerCancelMarketOffer` | 4 | `Game::playerCancelMarketOffer` (static); `tfs::iomarket::getOfferByCounter` (static); `tfs::iomarket::moveOfferToHistory` (static); `Item::CreateItem` (static) |
| `Game::playerAcceptMarketOffer` | 10 | `Game::playerAcceptMarketOffer` (static); `IOLoginData::getAccountIdByPlayerId` (static); `IOLoginData::increaseBankBalance` (static); `IOLoginData::loadPlayerById` (static); `IOLoginData::savePlayer` (static); … +5 more |
| `Game::parsePlayerExtendedOpcode` | 1 | `Game::parsePlayerExtendedOpcode` (static) |
| `Game::parsePlayerNetworkMessage` | 2 | `tfs::events::player::onNetworkMessage` (static); `Game::parsePlayerNetworkMessage` (static) |
| `Game::getMarketItemList` | 1 | `Game::getMarketItemList` (static) |
| `Game::forceAddCondition` | 1 | `Game::forceAddCondition` (static) |
| `Game::forceRemoveCondition` | 1 | `Game::forceRemoveCondition` (static) |
| `Game::sendOfflineTrainingDialog` | 1 | `Game::sendOfflineTrainingDialog` (static) |
| `Game::playerAnswerModalWindow` | 1 | `Game::playerAnswerModalWindow` (static) |
| `Game::addPlayer` | 1 | `Game::addPlayer` (static) |
| `Game::removePlayer` | 1 | `Game::removePlayer` (static) |
| `Game::addNpc` | 1 | `Game::addNpc` (static) |
| `Game::removeNpc` | 1 | `Game::removeNpc` (static) |
| `Game::addMonster` | 1 | `Game::addMonster` (static) |
| `Game::removeMonster` | 1 | `Game::removeMonster` (static) |
| `Game::getGuild` | 1 | `Game::getGuild` (static) |
| `Game::addGuild` | 1 | `Game::addGuild` (static) |
| `Game::removeGuild` | 1 | `Game::removeGuild` (static) |
| `Game::decreaseBrowseFieldRef` | 1 | `Game::decreaseBrowseFieldRef` (static) |
| `Game::internalRemoveItems` | 1 | `Game::internalRemoveItems` (static) |
| `Game::getBedBySleeper` | 1 | `Game::getBedBySleeper` (static) |
| `Game::setBedSleeper` | 1 | `Game::setBedSleeper` (static) |
| `Game::removeBedSleeper` | 1 | `Game::removeBedSleeper` (static) |
| `Game::updatePodium` | 1 | `Game::updatePodium` (static) |
| `Game::getUniqueItem` | 1 | `Game::getUniqueItem` (static) |
| `Game::addUniqueItem` | 1 | `Game::addUniqueItem` (static) |
| `Game::removeUniqueItem` | 1 | `Game::removeUniqueItem` (static) |
| `Game::reload` | 17 | `Actions::clear` (static); `Chat::load` (static); `ConfigManager::load` (static); `CreatureEvents::clear` (static); `CreatureEvents::removeInvalidEvents` (static); … +12 more |

### `globalevent.h` (header declarations)

8 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_GLOBALEVENT_H` | 0 | — |
| `GlobalEvent_t` | 0 | — |
| `GlobalEvent_t::GLOBALEVENT_NONE` | 0 | — |
| `GlobalEvent_t::GLOBALEVENT_TIMER` | 0 | — |
| `GlobalEvent_t::GLOBALEVENT_STARTUP` | 0 | — |
| `GlobalEvent_t::GLOBALEVENT_SHUTDOWN` | 0 | — |
| `GlobalEvent_t::GLOBALEVENT_RECORD` | 0 | — |
| `GlobalEvent_t::GLOBALEVENT_SAVE` | 0 | — |

### `globalevent`

16 node(s), 22 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `GlobalEvents::clearMap` | 1 | `GlobalEvents::clearMap` (static) |
| `GlobalEvents::clear` | 1 | `GlobalEvents::clear` (static) |
| `GlobalEvents::getEvent` | 1 | `GlobalEvents::getEvent` (static) |
| `GlobalEvents::registerEvent` | 1 | `GlobalEvents::registerEvent` (static) |
| `GlobalEvents::registerLuaEvent` | 1 | `GlobalEvents::registerLuaEvent` (static) |
| `GlobalEvents::startup` | 1 | `GlobalEvents::startup` (static) |
| `GlobalEvents::shutdown` | 1 | `GlobalEvents::shutdown` (static) |
| `GlobalEvents::save` | 1 | `GlobalEvents::save` (static) |
| `GlobalEvents::timer` | 1 | `GlobalEvents::timer` (static) |
| `GlobalEvents::think` | 1 | `GlobalEvents::think` (static) |
| `GlobalEvents::execute` | 1 | `GlobalEvents::execute` (static) |
| `GlobalEvents::getEventMap` | 1 | `GlobalEvents::getEventMap` (static) |
| `GlobalEvent::configureEvent` | 1 | `GlobalEvent::configureEvent` (static) |
| `GlobalEvent::getScriptEventName` | 1 | `GlobalEvent::getScriptEventName` (static) |
| `GlobalEvent::executeRecord` | 4 | `GlobalEvent::executeRecord` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `LuaScriptInterface::callFunction` (dynamic/curated) |
| `GlobalEvent::executeEvent` | 4 | `GlobalEvent::executeEvent` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `LuaScriptInterface::callFunction` (dynamic/curated) |

### `groups.h` (header declarations)

11 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Group` | 0 | — |
| `Group::name` | 0 | — |
| `Group::flags` | 0 | — |
| `Group::maxDepotItems` | 0 | — |
| `Group::maxVipEntries` | 0 | — |
| `Group::id` | 0 | — |
| `Group::access` | 0 | — |
| `Groups` | 0 | — |
| `Groups::load` | 0 | — |
| `Groups::getGroup` | 0 | — |
| `Groups::groups` | 0 | — |

### `groups`

3 node(s), 2 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ParsePlayerFlagMap` | 0 | — |
| `Groups::load` | 1 | `Groups::load` (static) |
| `Groups::getGroup` | 1 | `Groups::getGroup` (static) |

### `guild.h` (header declarations)

36 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Guild` | 0 | — |
| `Guild::MEMBER_RANK_LEVEL_DEFAULT` | 0 | — |
| `Guild::Guild` | 0 | — |
| `GuildRank::GuildRank` | 0 | — |
| `Guild::id` | 0 | — |
| `Guild::memberCount` | 0 | — |
| `Guild::membersOnline` | 0 | — |
| `Guild::motd` | 0 | — |
| `Guild::name` | 0 | — |
| `Guild::ranks` | 0 | — |
| `GuildRank::id` | 0 | — |
| `GuildRank::level` | 0 | — |
| `GuildRank::name` | 0 | — |
| `Guild::getId` | 0 | — |
| `Guild::getMemberCount` | 0 | — |
| `Guild::getMembersOnline` | 0 | — |
| `Guild::getMotd` | 0 | — |
| `Guild::getName` | 0 | — |
| `Guild::getRanks` | 0 | — |
| `Guild::setMemberCount` | 0 | — |
| `Guild::setMotd` | 0 | — |
| `GuildRank` | 0 | — |
| `GuildRank_ptr` | 0 | — |
| `GuildWarVector` | 0 | — |
| `Guild_ptr` | 0 | — |
| `FS_GUILD_H` | 0 | — |
| `GuildRank` | 0 | — |
| `GuildRank::GuildRank` | 0 | — |
| `Guild` | 0 | — |
| `Guild::Guild` | 0 | — |
| `Guild::addMember` | 0 | — |
| `Guild::removeMember` | 0 | — |
| `Guild::addRank` | 0 | — |
| `Guild::getRankById` | 0 | — |
| `Guild::getRankByName` | 0 | — |
| `Guild::getRankByLevel` | 0 | — |

### `guild`

16 node(s), 11 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `IOGuild::getGuildIdByName` | 0 | — |
| `IOGuild::loadGuild` | 0 | — |
| `Guild::addMember` | 0 | — |
| `Guild::addRank` | 0 | — |
| `Guild::getRankById` | 0 | — |
| `Guild::getRankByLevel` | 0 | — |
| `Guild::getRankByName` | 0 | — |
| `Guild::removeMember` | 0 | — |
| `Guild::addMember` | 1 | `Guild::addMember` (static) |
| `Guild::removeMember` | 2 | `Game::removeGuild` (static); `Guild::removeMember` (static) |
| `Guild::addRank` | 1 | `Guild::addRank` (static) |
| `Guild::getRankById` | 1 | `Guild::getRankById` (static) |
| `Guild::getRankByName` | 1 | `Guild::getRankByName` (static) |
| `Guild::getRankByLevel` | 1 | `Guild::getRankByLevel` (static) |
| `IOGuild::loadGuild` | 2 | `Database::getInstance` (static); `IOGuild::loadGuild` (static) |
| `IOGuild::getGuildIdByName` | 2 | `Database::getInstance` (static); `IOGuild::getGuildIdByName` (static) |

### `house.h` (header declarations)

136 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `AccessList` | 0 | — |
| `Door` | 0 | — |
| `House` | 0 | — |
| `HouseTransferItem` | 0 | — |
| `Houses` | 0 | — |
| `HouseTransferItem::HouseTransferItem` | 0 | — |
| `Houses::Houses` | 0 | — |
| `Houses::~Houses` | 0 | — |
| `AccessHouseLevel_t` | 0 | — |
| `AccessList_t` | 0 | — |
| `RentPeriod_t` | 0 | — |
| `AccessHouseLevel_t::HOUSE_GUEST` | 0 | — |
| `AccessHouseLevel_t::HOUSE_NOT_INVITED` | 0 | — |
| `AccessHouseLevel_t::HOUSE_OWNER` | 0 | — |
| `AccessHouseLevel_t::HOUSE_SUBOWNER` | 0 | — |
| `AccessList_t::GUEST_LIST` | 0 | — |
| `AccessList_t::SUBOWNER_LIST` | 0 | — |
| `RentPeriod_t::RENTPERIOD_DAILY` | 0 | — |
| `RentPeriod_t::RENTPERIOD_MONTHLY` | 0 | — |
| `RentPeriod_t::RENTPERIOD_NEVER` | 0 | — |
| `RentPeriod_t::RENTPERIOD_WEEKLY` | 0 | — |
| `RentPeriod_t::RENTPERIOD_YEARLY` | 0 | — |
| `AccessList::allowEveryone` | 0 | — |
| `AccessList::guildRankList` | 0 | — |
| `AccessList::list` | 0 | — |
| `AccessList::playerList` | 0 | — |
| `Door::accessList` | 0 | — |
| `Door::house` | 0 | — |
| `House::bedsList` | 0 | — |
| `House::doorSet` | 0 | — |
| `House::guestList` | 0 | — |
| `House::houseName` | 0 | — |
| `House::houseTiles` | 0 | — |
| `House::id` | 0 | — |
| `House::isLoaded` | 0 | — |
| `House::owner` | 0 | — |
| `House::ownerAccountId` | 0 | — |
| `House::ownerName` | 0 | — |
| `House::paidUntil` | 0 | — |
| `House::posEntry` | 0 | — |
| `House::rent` | 0 | — |
| `House::rentWarnings` | 0 | — |
| `House::subOwnerList` | 0 | — |
| `House::townId` | 0 | — |
| `House::transferItem` | 0 | — |
| `House::transfer_container` | 0 | — |
| `HouseTransferItem::house` | 0 | — |
| `Houses::houseMap` | 0 | — |
| `Door::getDoor (const)` | 0 | — |
| `Door::getDoor (non-const)` | 0 | — |
| `Door::getDoorId` | 0 | — |
| `Door::getHouse` | 0 | — |
| `Door::serializeAttr` | 0 | — |
| `Door::setDoorId` | 0 | — |
| `House::getBedCount` | 0 | — |
| `House::getBeds` | 0 | — |
| `House::getDoors` | 0 | — |
| `House::getEntryPosition` | 0 | — |
| `House::getId` | 0 | — |
| `House::getName` | 0 | — |
| `House::getOwner` | 0 | — |
| `House::getOwnerName` | 0 | — |
| `House::getPaidUntil` | 0 | — |
| `House::getPayRentWarnings` | 0 | — |
| `House::getRent` | 0 | — |
| `House::getTiles` | 0 | — |
| `House::getTownId` | 0 | — |
| `House::setEntryPos` | 0 | — |
| `House::setName` | 0 | — |
| `House::setPaidUntil` | 0 | — |
| `House::setPayRentWarnings` | 0 | — |
| `House::setRent` | 0 | — |
| `House::setTownId` | 0 | — |
| `HouseTransferItem::canTransform` | 0 | — |
| `Houses::addHouse` | 0 | — |
| `Houses::getHouse` | 0 | — |
| `Houses::getHouses` | 0 | — |
| `HouseBedItemList` | 0 | — |
| `HouseMap` | 0 | — |
| `HouseTileList` | 0 | — |
| `FS_HOUSE_H` | 0 | — |
| `AccessList` | 0 | — |
| `AccessList::parseList` | 0 | — |
| `AccessList::addPlayer` | 0 | — |
| `AccessList::addGuild` | 0 | — |
| `AccessList::addGuildRank` | 0 | — |
| `AccessList::isInList` | 0 | — |
| `AccessList::getList` | 0 | — |
| `AccessList_t` | 0 | — |
| `AccessList_t::GUEST_LIST` | 0 | — |
| `AccessList_t::SUBOWNER_LIST` | 0 | — |
| `AccessHouseLevel_t` | 0 | — |
| `AccessHouseLevel_t::HOUSE_NOT_INVITED` | 0 | — |
| `AccessHouseLevel_t::HOUSE_GUEST` | 0 | — |
| `AccessHouseLevel_t::HOUSE_SUBOWNER` | 0 | — |
| `AccessHouseLevel_t::HOUSE_OWNER` | 0 | — |
| `House` | 0 | — |
| `House::House` | 0 | — |
| `House::addTile` | 0 | — |
| `House::canEditAccessList` | 0 | — |
| `House::setAccessList` | 0 | — |
| `House::getAccessList` | 0 | — |
| `House::isInvited` | 0 | — |
| `House::getHouseAccessLevel` | 0 | — |
| `House::kickPlayer` | 0 | — |
| `House::setOwner` | 0 | — |
| `House::addDoor` | 0 | — |
| `House::removeDoor` | 0 | — |
| `House::getDoorByNumber` | 0 | — |
| `House::getDoorByPosition` | 0 | — |
| `House::getTransferItem` | 0 | — |
| `House::resetTransferItem` | 0 | — |
| `House::executeTransfer` | 0 | — |
| `House::addBed` | 0 | — |
| `House::transferToDepot` | 0 | — |
| `House::transferToDepot` | 0 | — |
| `RentPeriod_t` | 0 | — |
| `RentPeriod_t::RENTPERIOD_DAILY` | 0 | — |
| `RentPeriod_t::RENTPERIOD_WEEKLY` | 0 | — |
| `RentPeriod_t::RENTPERIOD_MONTHLY` | 0 | — |
| `RentPeriod_t::RENTPERIOD_YEARLY` | 0 | — |
| `RentPeriod_t::RENTPERIOD_NEVER` | 0 | — |
| `Houses` | 0 | — |
| `Houses::Houses` | 0 | — |
| `Houses::Houses` | 0 | — |
| `Houses::it` | 0 | — |
| `Houses::find` | 0 | — |
| `Houses::house` | 0 | — |
| `Houses::House` | 0 | — |
| `Houses::house` | 0 | — |
| `Houses::it` | 0 | — |
| `Houses::find` | 0 | — |
| `Houses::nullptr` | 0 | — |
| `Houses::getHouseByPlayerId` | 0 | — |
| `Houses::loadHousesXML` | 0 | — |
| `Houses::payHouses` | 0 | — |

### `house`

74 node(s), 52 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Door::Door` | 0 | — |
| `House::House` | 1 | `House::House` (static) |
| `anon::getGuildByName` | 0 | — |
| `AccessList::addGuild` | 0 | — |
| `AccessList::addGuildRank` | 0 | — |
| `AccessList::addPlayer` | 0 | — |
| `AccessList::getList` | 0 | — |
| `AccessList::isInList` | 0 | — |
| `AccessList::parseList` | 0 | — |
| `Door::canUse` | 0 | — |
| `Door::getAccessList` | 0 | — |
| `Door::onRemoved` | 0 | — |
| `Door::readAttr` | 0 | — |
| `Door::setAccessList` | 0 | — |
| `Door::setHouse` | 0 | — |
| `House::addBed` | 0 | — |
| `House::addDoor` | 0 | — |
| `House::addTile` | 0 | — |
| `House::canEditAccessList` | 0 | — |
| `House::executeTransfer` | 0 | — |
| `House::getAccessList` | 0 | — |
| `House::getDoorByNumber` | 0 | — |
| `House::getDoorByPosition` | 0 | — |
| `House::getHouseAccessLevel` | 0 | — |
| `House::getTransferItem` | 0 | — |
| `House::isInvited` | 0 | — |
| `House::kickPlayer` | 0 | — |
| `House::removeDoor` | 0 | — |
| `House::resetTransferItem` | 0 | — |
| `House::setAccessList` | 0 | — |
| `House::setOwner` | 0 | — |
| `House::transferToDepot (Player*)` | 0 | — |
| `House::transferToDepot (no args)` | 0 | — |
| `HouseTransferItem::createHouseTransferItem (static)` | 0 | — |
| `HouseTransferItem::onTradeEvent` | 0 | — |
| `Houses::getHouseByPlayerId` | 0 | — |
| `Houses::loadHousesXML` | 0 | — |
| `Houses::payHouses` | 0 | — |
| `House::addTile` | 1 | `House::addTile` (static) |
| `House::setOwner` | 4 | `Database::getInstance` (static); `House::setOwner` (static); `IOLoginData::getAccountIdByPlayerName` (static); `IOLoginData::getNameByGuid` (static) |
| `House::getHouseAccessLevel` | 1 | `House::getHouseAccessLevel` (static) |
| `House::kickPlayer` | 3 | `Game::addMagicEffect` (static); `Game::internalTeleport` (static); `House::kickPlayer` (static) |
| `House::setAccessList` | 1 | `House::setAccessList` (static) |
| `House::transferToDepot` | 0 | — |
| `House::transferToDepot` | 2 | `Game::internalMoveItem` (static); `House::transferToDepot` (static) |
| `House::getAccessList` | 1 | `House::getAccessList` (static) |
| `House::isInvited` | 1 | `House::isInvited` (static) |
| `House::addDoor` | 1 | `House::addDoor` (static) |
| `House::removeDoor` | 1 | `House::removeDoor` (static) |
| `House::addBed` | 1 | `House::addBed` (static) |
| `House::getDoorByNumber` | 1 | `House::getDoorByNumber` (static) |
| `House::getDoorByPosition` | 1 | `House::getDoorByPosition` (static) |
| `House::canEditAccessList` | 1 | `House::canEditAccessList` (static) |
| `House::getTransferItem` | 2 | `House::getTransferItem` (static); `HouseTransferItem::createHouseTransferItem` (static) |
| `House::resetTransferItem` | 2 | `Game::ReleaseItem` (static); `House::resetTransferItem` (static) |
| `HouseTransferItem::createHouseTransferItem` | 1 | `HouseTransferItem::createHouseTransferItem` (static) |
| `HouseTransferItem::onTradeEvent` | 2 | `Game::internalRemoveItem` (static); `HouseTransferItem::onTradeEvent` (static) |
| `House::executeTransfer` | 1 | `House::executeTransfer` (static) |
| `AccessList::parseList` | 1 | `AccessList::parseList` (static) |
| `AccessList::addPlayer` | 3 | `Game::getPlayerByName` (static); `AccessList::addPlayer` (static); `IOLoginData::getGuidByName` (static) |
| `getGuildByName` | 3 | `Game::getGuild` (static); `IOGuild::getGuildIdByName` (static); `IOGuild::loadGuild` (static) |
| `AccessList::addGuild` | 1 | `AccessList::addGuild` (static) |
| `AccessList::addGuildRank` | 1 | `AccessList::addGuildRank` (static) |
| `AccessList::isInList` | 1 | `AccessList::isInList` (static) |
| `AccessList::getList` | 1 | `AccessList::getList` (static) |
| `Door::readAttr` | 2 | `Door::readAttr` (static); `Item::readAttr` (static) |
| `Door::setHouse` | 1 | `Door::setHouse` (static) |
| `Door::canUse` | 1 | `Door::canUse` (static) |
| `Door::setAccessList` | 1 | `Door::setAccessList` (static) |
| `Door::getAccessList` | 1 | `Door::getAccessList` (static) |
| `Door::onRemoved` | 2 | `Door::onRemoved` (static); `Item::onRemoved` (static) |
| `Houses::getHouseByPlayerId` | 1 | `Houses::getHouseByPlayerId` (static) |
| `Houses::loadHousesXML` | 1 | `Houses::loadHousesXML` (static) |
| `Houses::payHouses` | 2 | `Houses::payHouses` (static); `IOLoginData::loadPlayerById` (static) |

### `housetile.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_HOUSETILE_H` | 0 | — |

### `housetile`

6 node(s), 11 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `HouseTile::addThing` | 2 | `HouseTile::addThing` (static); `Tile::addThing` (static) |
| `HouseTile::internalAddThing` | 2 | `HouseTile::internalAddThing` (static); `Tile::internalAddThing` (static) |
| `HouseTile::updateHouse` | 1 | `HouseTile::updateHouse` (static) |
| `HouseTile::queryAdd` | 2 | `HouseTile::queryAdd` (static); `Tile::queryAdd` (static) |
| `HouseTile::queryDestination` | 2 | `HouseTile::queryDestination` (static); `Tile::queryDestination` (static) |
| `HouseTile::queryRemove` | 2 | `HouseTile::queryRemove` (static); `Tile::queryRemove` (static) |

### `http__cacheinfo.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::handle_cacheinfo` | 0 | — |

### `http__cacheinfo`

1 node(s), 2 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::handle_cacheinfo` | 2 | `Database::getInstance` (static); `tfs::http::handle_cacheinfo` (static) |

### `http__error.h` (header declarations)

2 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::detail::ErrorResponseParams` | 0 | — |
| `tfs::http::make_error_response` | 0 | — |

### `http__error`

1 node(s), 1 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::make_error_response` | 1 | `tfs::http::make_error_response` (static) |

### `http__http.h` (header declarations)

2 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::start` | 0 | — |
| `tfs::http::stop` | 0 | — |

### `http__http`

2 node(s), 2 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::start` | 1 | `tfs::http::start` (static) |
| `tfs::http::stop` | 1 | `tfs::http::stop` (static) |

### `http__listener.h` (header declarations)

6 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::Listener` | 0 | — |
| `tfs::http::Listener::Listener` | 0 | — |
| `tfs::http::Listener::accept` | 0 | — |
| `tfs::http::Listener::run` | 0 | — |
| `tfs::http::Listener::on_accept` | 0 | — |
| `tfs::http::make_listener` | 0 | — |

### `http__listener`

4 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::Listener::Listener` | 0 | — |
| `tfs::http::Listener::accept` | 0 | — |
| `tfs::http::Listener::on_accept` | 0 | — |
| `tfs::http::make_listener` | 0 | — |

### `http__login.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::handle_login` | 0 | — |

### `http__login`

1 node(s), 3 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::handle_login` | 3 | `Database::getInstance` (static); `tfs::http::handle_login` (static); `Vocations::getVocation` (static) |

### `http__router.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::handle_request` | 0 | — |

### `http__router`

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::handle_request` | 0 | — |

### `http__session.h` (header declarations)

9 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::Session` | 0 | — |
| `tfs::http::Session::Session` | 0 | — |
| `tfs::http::Session::read` | 0 | — |
| `tfs::http::Session::write` | 0 | — |
| `tfs::http::Session::close` | 0 | — |
| `tfs::http::Session::run` | 0 | — |
| `tfs::http::Session::on_read` | 0 | — |
| `tfs::http::Session::on_write` | 0 | — |
| `tfs::http::make_session` | 0 | — |

### `http__session`

8 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::http::Session::Session` | 0 | — |
| `tfs::http::Session::read` | 0 | — |
| `tfs::http::Session::write` | 0 | — |
| `tfs::http::Session::close` | 0 | — |
| `tfs::http::Session::run` | 0 | — |
| `tfs::http::Session::on_read` | 0 | — |
| `tfs::http::Session::on_write` | 0 | — |
| `tfs::http::make_session` | 0 | — |

### `inbox.h` (header declarations)

4 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Inbox` | 0 | — |
| `Inbox::canRemove` | 0 | — |
| `Inbox_ptr` | 0 | — |
| `FS_INBOX_H` | 0 | — |

### `inbox`

9 node(s), 5 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Inbox::Inbox` | 1 | `Inbox::Inbox` (static) |
| `Inbox::getParent` | 0 | — |
| `Inbox::postAddNotification` | 0 | — |
| `Inbox::postRemoveNotification` | 0 | — |
| `Inbox::queryAdd` | 0 | — |
| `Inbox::queryAdd` | 1 | `Inbox::queryAdd` (static) |
| `Inbox::postAddNotification` | 1 | `Inbox::postAddNotification` (static) |
| `Inbox::postRemoveNotification` | 1 | `Inbox::postRemoveNotification` (static) |
| `Inbox::getParent` | 1 | `Inbox::getParent` (static) |

### `iologindata.h` (header declarations)

3 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ItemBlockList` | 0 | — |
| `IOLoginData` | 0 | — |
| `IOLoginData::ItemMap` | 0 | — |

### `iologindata`

127 node(s), 52 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_game (iologindata)` | 0 | — |
| `IOLoginData::getAccountIdByPlayerName` | 2 | `Database::getInstance` (static); `IOLoginData::getAccountIdByPlayerName` (static) |
| `players.account_id` | 0 | — |
| `IOLoginData::getAccountIdByPlayerId` | 2 | `Database::getInstance` (static); `IOLoginData::getAccountIdByPlayerId` (static) |
| `IOLoginData::getAccountType` | 2 | `Database::getInstance` (static); `IOLoginData::getAccountType` (static) |
| `accounts.id` | 0 | — |
| `accounts.type` | 0 | — |
| `IOLoginData::setAccountType` | 2 | `Database::getInstance` (static); `IOLoginData::setAccountType` (static) |
| `IOLoginData::updateOnlineStatus` | 2 | `Database::getInstance` (static); `IOLoginData::updateOnlineStatus` (static) |
| `players_online.player_id` | 0 | — |
| `IOLoginData::preloadPlayer` | 2 | `Database::getInstance` (static); `IOLoginData::preloadPlayer` (static) |
| `accounts.premium_ends_at` | 0 | — |
| `players.deletion` | 0 | — |
| `players.group_id` | 0 | — |
| `IOLoginData::loadPlayerById` | 2 | `Database::getInstance` (static); `IOLoginData::loadPlayerById` (static) |
| `players.balance` | 0 | — |
| `players.blessings` | 0 | — |
| `players.cap` | 0 | — |
| `players.conditions` | 0 | — |
| `players.currentmount` | 0 | — |
| `players.direction` | 0 | — |
| `players.experience` | 0 | — |
| `players.health` | 0 | — |
| `players.healthmax` | 0 | — |
| `players.lastip` | 0 | — |
| `players.lastlogin` | 0 | — |
| `players.lastlogout` | 0 | — |
| `players.level` | 0 | — |
| `players.lookaddons` | 0 | — |
| `players.lookbody` | 0 | — |
| `players.lookfeet` | 0 | — |
| `players.lookhead` | 0 | — |
| `players.looklegs` | 0 | — |
| `players.lookmount` | 0 | — |
| `players.lookmountbody` | 0 | — |
| `players.lookmountfeet` | 0 | — |
| `players.lookmounthead` | 0 | — |
| `players.lookmountlegs` | 0 | — |
| `players.looktype` | 0 | — |
| `players.maglevel` | 0 | — |
| `players.mana` | 0 | — |
| `players.manamax` | 0 | — |
| `players.manaspent` | 0 | — |
| `players.offlinetraining_skill` | 0 | — |
| `players.offlinetraining_time` | 0 | — |
| `players.posx` | 0 | — |
| `players.posy` | 0 | — |
| `players.posz` | 0 | — |
| `players.randomizemount` | 0 | — |
| `players.sex` | 0 | — |
| `players.skill_axe` | 0 | — |
| `players.skill_axe_tries` | 0 | — |
| `players.skill_club` | 0 | — |
| `players.skill_club_tries` | 0 | — |
| `players.skill_dist` | 0 | — |
| `players.skill_dist_tries` | 0 | — |
| `players.skill_fishing` | 0 | — |
| `players.skill_fishing_tries` | 0 | — |
| `players.skill_fist` | 0 | — |
| `players.skill_fist_tries` | 0 | — |
| `players.skill_shielding` | 0 | — |
| `players.skill_shielding_tries` | 0 | — |
| `players.skill_sword` | 0 | — |
| `players.skill_sword_tries` | 0 | — |
| `players.skull` | 0 | — |
| `players.skulltime` | 0 | — |
| `players.soul` | 0 | — |
| `players.stamina` | 0 | — |
| `players.town_id` | 0 | — |
| `players.vocation` | 0 | — |
| `IOLoginData::loadPlayerByName` | 2 | `Database::getInstance` (static); `IOLoginData::loadPlayerByName` (static) |
| `getWarList` | 1 | `Database::getInstance` (static) |
| `guild_wars.guild1` | 0 | — |
| `guild_wars.guild2` | 0 | — |
| `guild_wars.ended` | 0 | — |
| `guild_wars.status` | 0 | — |
| `IOLoginData::loadPlayer` | 7 | `Condition::createCondition` (static); `Database::getInstance` (static); `Game::addGuild` (static); `Game::getGuild` (static); `IOGuild::loadGuild` (static); … +2 more |
| `guild_membership.guild_id` | 0 | — |
| `guild_membership.nick` | 0 | — |
| `guild_membership.player_id` | 0 | — |
| `guild_membership.rank_id` | 0 | — |
| `guild_ranks.id` | 0 | — |
| `guild_ranks.level` | 0 | — |
| `guild_ranks.name` | 0 | — |
| `player_spells.player_id` | 0 | — |
| `player_items.attributes` | 0 | — |
| `player_items.count` | 0 | — |
| `player_items.itemtype` | 0 | — |
| `player_items.pid` | 0 | — |
| `player_items.sid` | 0 | — |
| `player_depotitems.player_id` | 0 | — |
| `player_inboxitems.player_id` | 0 | — |
| `player_storeinboxitems.player_id` | 0 | — |
| `player_storage.key` | 0 | — |
| `player_storage.player_id` | 0 | — |
| `player_storage.value` | 0 | — |
| `account_viplist.account_id` | 0 | — |
| `account_viplist.player_id` | 0 | — |
| `player_outfits.addons` | 0 | — |
| `player_outfits.outfit_id` | 0 | — |
| `player_outfits.player_id` | 0 | — |
| `player_mounts.mount_id` | 0 | — |
| `player_mounts.player_id` | 0 | — |
| `IOLoginData::saveItems` | 2 | `Database::getInstance` (static); `IOLoginData::saveItems` (static) |
| `IOLoginData::savePlayer` | 2 | `Database::getInstance` (static); `IOLoginData::savePlayer` (static) |
| `players.save` | 0 | — |
| `players.onlinetime` | 0 | — |
| `player_spells.name` | 0 | — |
| `player_items.player_id` | 0 | — |
| `IOLoginData::getNameByGuid` | 2 | `Database::getInstance` (static); `IOLoginData::getNameByGuid` (static) |
| `players.name` | 0 | — |
| `IOLoginData::getGuidByName` | 2 | `Database::getInstance` (static); `IOLoginData::getGuidByName` (static) |
| `players.id` | 0 | — |
| `IOLoginData::getGuidByNameEx` | 2 | `Database::getInstance` (static); `IOLoginData::getGuidByNameEx` (static) |
| `IOLoginData::formatPlayerName` | 2 | `Database::getInstance` (static); `IOLoginData::formatPlayerName` (static) |
| `IOLoginData::loadItems` | 2 | `IOLoginData::loadItems` (static); `Item::CreateItem` (static) |
| `IOLoginData::increaseBankBalance` | 2 | `Database::getInstance` (static); `IOLoginData::increaseBankBalance` (static) |
| `IOLoginData::hasBiddedOnHouse` | 2 | `Database::getInstance` (static); `IOLoginData::hasBiddedOnHouse` (static) |
| `houses.highest_bidder` | 0 | — |
| `IOLoginData::getVIPEntries` | 2 | `Database::getInstance` (static); `IOLoginData::getVIPEntries` (static) |
| `account_viplist.description` | 0 | — |
| `account_viplist.icon` | 0 | — |
| `account_viplist.notify` | 0 | — |
| `IOLoginData::addVIPEntry` | 2 | `Database::getInstance` (static); `IOLoginData::addVIPEntry` (static) |
| `IOLoginData::editVIPEntry` | 2 | `Database::getInstance` (static); `IOLoginData::editVIPEntry` (static) |
| `IOLoginData::removeVIPEntry` | 2 | `Database::getInstance` (static); `IOLoginData::removeVIPEntry` (static) |
| `IOLoginData::updatePremiumTime` | 2 | `Database::getInstance` (static); `IOLoginData::updatePremiumTime` (static) |

### `iomap.h` (header declarations)

59 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_IOMAP_H` | 0 | — |
| `OTBM_AttrTypes_t` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_DESCRIPTION` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_EXT_FILE` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_TILE_FLAGS` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_ACTION_ID` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_UNIQUE_ID` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_TEXT` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_DESC` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_TELE_DEST` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_ITEM` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_DEPOT_ID` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_EXT_SPAWN_FILE` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_RUNE_CHARGES` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_EXT_HOUSE_FILE` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_HOUSEDOORID` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_COUNT` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_DURATION` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_DECAYING_STATE` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_WRITTENDATE` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_WRITTENBY` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_SLEEPERGUID` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_SLEEPSTART` | 0 | — |
| `OTBM_AttrTypes_t::OTBM_ATTR_CHARGES` | 0 | — |
| `OTBM_NodeTypes_t` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_ROOTV1` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_MAP_DATA` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_ITEM_DEF` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_TILE_AREA` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_TILE` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_ITEM` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_TILE_SQUARE` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_TILE_REF` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_SPAWNS` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_SPAWN_AREA` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_MONSTER` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_TOWNS` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_TOWN` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_HOUSETILE` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_WAYPOINTS` | 0 | — |
| `OTBM_NodeTypes_t::OTBM_WAYPOINT` | 0 | — |
| `OTBM_TileFlag_t` | 0 | — |
| `OTBM_TileFlag_t::OTBM_TILEFLAG_PROTECTIONZONE` | 0 | — |
| `OTBM_TileFlag_t::OTBM_TILEFLAG_NOPVPZONE` | 0 | — |
| `OTBM_TileFlag_t::OTBM_TILEFLAG_NOLOGOUT` | 0 | — |
| `OTBM_TileFlag_t::OTBM_TILEFLAG_PVPZONE` | 0 | — |
| `OTBM_root_header` | 0 | — |
| `OTBM_Destination_coords` | 0 | — |
| `OTBM_Tile_coords` | 0 | — |
| `IOMap` | 0 | — |
| `IOMap::createTile` | 0 | — |
| `IOMap::loadMap` | 0 | — |
| `IOMap::getString` | 0 | — |
| `IOMap::loadFromXml` | 0 | — |
| `IOMap::getString` | 0 | — |
| `IOMap::loadHousesXML` | 0 | — |
| `IOMap::parseWaypoints` | 0 | — |
| `IOMap::parseTowns` | 0 | — |
| `IOMap::parseTileArea` | 0 | — |

### `iomap`

6 node(s), 7 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `IOMap::createTile` | 1 | `IOMap::createTile` (static) |
| `IOMap::loadMap` | 1 | `IOMap::loadMap` (static) |
| `IOMap::parseMapDataAttributes` | 1 | `IOMap::parseMapDataAttributes` (static) |
| `IOMap::parseTileArea` | 2 | `IOMap::parseTileArea` (static); `Item::CreateItem` (static) |
| `IOMap::parseTowns` | 1 | `IOMap::parseTowns` (static) |
| `IOMap::parseWaypoints` | 1 | `IOMap::parseWaypoints` (static) |

### `iomapserialize.h` (header declarations)

11 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_IOMAPSERIALIZE_H` | 0 | — |
| `IOMapSerialize` | 0 | — |
| `IOMapSerialize::loadHouseItems` | 0 | — |
| `IOMapSerialize::saveHouseItems` | 0 | — |
| `IOMapSerialize::loadHouseInfo` | 0 | — |
| `IOMapSerialize::saveHouseInfo` | 0 | — |
| `IOMapSerialize::saveHouse` | 0 | — |
| `IOMapSerialize::saveItem` | 0 | — |
| `IOMapSerialize::saveTile` | 0 | — |
| `IOMapSerialize::loadContainer` | 0 | — |
| `IOMapSerialize::loadItem` | 0 | — |

### `iomapserialize`

9 node(s), 17 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `IOMapSerialize::loadHouseItems` | 2 | `Database::getInstance` (static); `IOMapSerialize::loadHouseItems` (static) |
| `IOMapSerialize::saveHouseItems` | 2 | `Database::getInstance` (static); `IOMapSerialize::saveHouseItems` (static) |
| `IOMapSerialize::loadContainer` | 1 | `IOMapSerialize::loadContainer` (static) |
| `IOMapSerialize::loadItem` | 4 | `Game::removeBedSleeper` (static); `Game::transformItem` (static); `IOMapSerialize::loadItem` (static); `Item::CreateItem` (static) |
| `IOMapSerialize::saveItem` | 1 | `IOMapSerialize::saveItem` (static) |
| `IOMapSerialize::saveTile` | 1 | `IOMapSerialize::saveTile` (static) |
| `IOMapSerialize::loadHouseInfo` | 2 | `Database::getInstance` (static); `IOMapSerialize::loadHouseInfo` (static) |
| `IOMapSerialize::saveHouseInfo` | 2 | `Database::getInstance` (static); `IOMapSerialize::saveHouseInfo` (static) |
| `IOMapSerialize::saveHouse` | 2 | `Database::getInstance` (static); `IOMapSerialize::saveHouse` (static) |

### `iomarket.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `tfs::iomarket` | 0 | — |

### `iomarket`

34 node(s), 16 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_game` | 0 | — |
| `(anonymous)::purchaseStatistics` | 0 | — |
| `(anonymous)::saleStatistics` | 0 | — |
| `tfs::iomarket::getActiveOffers` | 1 | `Database::getInstance` (static) |
| `market_offers.amount` | 0 | — |
| `market_offers.anonymous` | 0 | — |
| `market_offers.created` | 0 | — |
| `market_offers.id` | 0 | — |
| `market_offers.itemtype` | 0 | — |
| `market_offers.price` | 0 | — |
| `market_offers.sale` | 0 | — |
| `tfs::iomarket::getOwnOffers` | 1 | `Database::getInstance` (static) |
| `market_offers.player_id` | 0 | — |
| `tfs::iomarket::getOwnHistory` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::processExpiredOffers` | 6 | `Game::getPlayerByGUID` (static); `Game::internalAddItem` (static); `IOLoginData::increaseBankBalance` (static); `IOLoginData::loadPlayerById` (static); `IOLoginData::savePlayer` (static); … +1 more |
| `tfs::iomarket::checkExpiredOffers` | 0 | — |
| `tfs::iomarket::getPlayerOfferCount` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::getOfferByCounter` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::createOffer` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::acceptOffer` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::deleteOffer` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::appendHistory` | 0 | — |
| `market_history.amount` | 0 | — |
| `market_history.expires_at` | 0 | — |
| `market_history.inserted` | 0 | — |
| `market_history.itemtype` | 0 | — |
| `market_history.player_id` | 0 | — |
| `market_history.price` | 0 | — |
| `market_history.sale` | 0 | — |
| `market_history.state` | 0 | — |
| `tfs::iomarket::moveOfferToHistory` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::updateStatistics` | 1 | `Database::getInstance` (static) |
| `tfs::iomarket::getPurchaseStatistics` | 0 | — |
| `tfs::iomarket::getSaleStatistics` | 0 | — |

### `item.h` (header declarations)

72 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ITEMPROPERTY` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_BLOCKSOLID` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_HASHEIGHT` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_BLOCKPROJECTILE` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_BLOCKPATH` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_ISVERTICAL` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_ISHORIZONTAL` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_MOVEABLE` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_IMMOVABLEBLOCKSOLID` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_IMMOVABLEBLOCKPATH` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_IMMOVABLENOFIELDBLOCKPATH` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_NOFIELDBLOCKPATH` | 0 | — |
| `ITEMPROPERTY::CONST_PROP_SUPPORTHANGABLE` | 0 | — |
| `TradeEvents_t` | 0 | — |
| `TradeEvents_t::ON_TRADE_TRANSFER` | 0 | — |
| `TradeEvents_t::ON_TRADE_CANCEL` | 0 | — |
| `ItemDecayState_t` | 0 | — |
| `ItemDecayState_t::DECAYING_FALSE` | 0 | — |
| `ItemDecayState_t::DECAYING_TRUE` | 0 | — |
| `ItemDecayState_t::DECAYING_PENDING` | 0 | — |
| `AttrTypes_t` | 0 | — |
| `AttrTypes_t::ATTR_TILE_FLAGS` | 0 | — |
| `AttrTypes_t::ATTR_ACTION_ID` | 0 | — |
| `AttrTypes_t::ATTR_UNIQUE_ID` | 0 | — |
| `AttrTypes_t::ATTR_TEXT` | 0 | — |
| `AttrTypes_t::ATTR_DESC` | 0 | — |
| `AttrTypes_t::ATTR_TELE_DEST` | 0 | — |
| `AttrTypes_t::ATTR_ITEM` | 0 | — |
| `AttrTypes_t::ATTR_DEPOT_ID` | 0 | — |
| `AttrTypes_t::ATTR_RUNE_CHARGES` | 0 | — |
| `AttrTypes_t::ATTR_HOUSEDOORID` | 0 | — |
| `AttrTypes_t::ATTR_COUNT` | 0 | — |
| `AttrTypes_t::ATTR_DURATION` | 0 | — |
| `AttrTypes_t::ATTR_DECAYING_STATE` | 0 | — |
| `AttrTypes_t::ATTR_WRITTENDATE` | 0 | — |
| `AttrTypes_t::ATTR_WRITTENBY` | 0 | — |
| `AttrTypes_t::ATTR_SLEEPERGUID` | 0 | — |
| `AttrTypes_t::ATTR_SLEEPSTART` | 0 | — |
| `AttrTypes_t::ATTR_CHARGES` | 0 | — |
| `AttrTypes_t::ATTR_CONTAINER_ITEMS` | 0 | — |
| `AttrTypes_t::ATTR_NAME` | 0 | — |
| `AttrTypes_t::ATTR_ARTICLE` | 0 | — |
| `AttrTypes_t::ATTR_PLURALNAME` | 0 | — |
| `AttrTypes_t::ATTR_WEIGHT` | 0 | — |
| `AttrTypes_t::ATTR_ATTACK` | 0 | — |
| `AttrTypes_t::ATTR_DEFENSE` | 0 | — |
| `AttrTypes_t::ATTR_EXTRADEFENSE` | 0 | — |
| `AttrTypes_t::ATTR_ARMOR` | 0 | — |
| `AttrTypes_t::ATTR_HITCHANCE` | 0 | — |
| `AttrTypes_t::ATTR_SHOOTRANGE` | 0 | — |
| `AttrTypes_t::ATTR_CUSTOM_ATTRIBUTES` | 0 | — |
| `AttrTypes_t::ATTR_DECAYTO` | 0 | — |
| `AttrTypes_t::ATTR_WRAPID` | 0 | — |
| `AttrTypes_t::ATTR_STOREITEM` | 0 | — |
| `AttrTypes_t::ATTR_ATTACK_SPEED` | 0 | — |
| `AttrTypes_t::ATTR_OPENCONTAINER` | 0 | — |
| `AttrTypes_t::ATTR_PODIUMOUTFIT` | 0 | — |
| `AttrTypes_t::ATTR_REFLECT` | 0 | — |
| `AttrTypes_t::ATTR_BOOST` | 0 | — |
| `Attr_ReadValue` | 0 | — |
| `Attr_ReadValue::ATTR_READ_CONTINUE` | 0 | — |
| `Attr_ReadValue::ATTR_READ_ERROR` | 0 | — |
| `Attr_ReadValue::ATTR_READ_END` | 0 | — |
| `Item::getPluralName` | 0 | — |
| `Item::getArticle` | 0 | — |
| `Item::setStoreItem` | 0 | — |
| `Item::canTransform` | 0 | — |
| `Item::isGroundTile` | 0 | — |
| `Item::isMagicField` | 0 | — |
| `Item::isPodium` | 0 | — |
| `Item::hasWalkStack` | 0 | — |
| `Item::isSupply` | 0 | — |

### `item`

43 node(s), 42 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Item::CreateItem` | 0 | — |
| `Item::CreateItemAsContainer` | 1 | `Item::CreateItemAsContainer` (static) |
| `Item::CreateItem` | 1 | `Item::CreateItem` (static) |
| `Item::clone` | 2 | `Item::CreateItem` (static); `Item::clone` (static) |
| `Item::equals` | 1 | `Item::equals` (static) |
| `Item::setDefaultSubtype` | 1 | `Item::setDefaultSubtype` (static) |
| `Item::onRemoved` | 3 | `Game::removeUniqueItem` (static); `Item::onRemoved` (static); `tfs::lua::removeTempItem` (static) |
| `Item::setID` | 1 | `Item::setID` (static) |
| `Item::getTopParent` | 0 | — |
| `Item::getTopParent` | 1 | `Item::getTopParent` (static) |
| `Item::getTile` | 0 | — |
| `Item::getTile` | 1 | `Item::getTile` (static) |
| `Item::getSubType` | 1 | `Item::getSubType` (static) |
| `Item::getHoldingPlayer` | 1 | `Item::getHoldingPlayer` (static) |
| `Item::setSubType` | 1 | `Item::setSubType` (static) |
| `Item::readAttr` | 1 | `Item::readAttr` (static) |
| `Item::unserializeAttr` | 1 | `Item::unserializeAttr` (static) |
| `Item::unserializeItemNode` | 1 | `Item::unserializeItemNode` (static) |
| `Item::serializeAttr` | 1 | `Item::serializeAttr` (static) |
| `Item::hasProperty` | 1 | `Item::hasProperty` (static) |
| `Item::getWeight` | 1 | `Item::getWeight` (static) |
| `Item::getNameDescription` | 0 | — |
| `Item::getNameDescription` | 1 | `Item::getNameDescription` (static) |
| `Item::getWeightDescription` | 0 | — |
| `Item::getWeightDescription` | 0 | — |
| `Item::getWeightDescription` | 1 | `Item::getWeightDescription` (static) |
| `Item::setUniqueId` | 2 | `Game::addUniqueItem` (static); `Item::setUniqueId` (static) |
| `Item::setDefaultDuration` | 1 | `Item::setDefaultDuration` (static) |
| `Item::canDecay` | 1 | `Item::canDecay` (static) |
| `Item::getWorth` | 1 | `Item::getWorth` (static) |
| `Item::getLightInfo` | 1 | `Item::getLightInfo` (static) |
| `Item::getReflect` | 1 | `Item::getReflect` (static) |
| `Item::getBoostPercent` | 1 | `Item::getBoostPercent` (static) |
| `ItemAttributes::getStrAttr` | 1 | `ItemAttributes::getStrAttr` (static) |
| `ItemAttributes::setStrAttr` | 1 | `ItemAttributes::setStrAttr` (static) |
| `ItemAttributes::removeAttribute` | 1 | `ItemAttributes::removeAttribute` (static) |
| `ItemAttributes::getIntAttr` | 1 | `ItemAttributes::getIntAttr` (static) |
| `ItemAttributes::setIntAttr` | 1 | `ItemAttributes::setIntAttr` (static) |
| `ItemAttributes::increaseIntAttr` | 1 | `ItemAttributes::increaseIntAttr` (static) |
| `ItemAttributes::getExistingAttr` | 1 | `ItemAttributes::getExistingAttr` (static) |
| `ItemAttributes::getAttr` | 1 | `ItemAttributes::getAttr` (static) |
| `Item::startDecaying` | 2 | `Game::startDecay` (static); `Item::startDecaying` (static) |
| `Item::hasMarketAttributes` | 1 | `Item::hasMarketAttributes` (static) |

### `itemloader.h` (header declarations)

155 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_ITEMLOADER_H` | 0 | — |
| `itemgroup_t` | 0 | — |
| `itemgroup_t::ITEM_GROUP_NONE` | 0 | — |
| `itemgroup_t::ITEM_GROUP_GROUND` | 0 | — |
| `itemgroup_t::ITEM_GROUP_CONTAINER` | 0 | — |
| `itemgroup_t::ITEM_GROUP_WEAPON` | 0 | — |
| `itemgroup_t::ITEM_GROUP_AMMUNITION` | 0 | — |
| `itemgroup_t::ITEM_GROUP_ARMOR` | 0 | — |
| `itemgroup_t::ITEM_GROUP_CHARGES` | 0 | — |
| `itemgroup_t::ITEM_GROUP_TELEPORT` | 0 | — |
| `itemgroup_t::ITEM_GROUP_MAGICFIELD` | 0 | — |
| `itemgroup_t::ITEM_GROUP_WRITEABLE` | 0 | — |
| `itemgroup_t::ITEM_GROUP_KEY` | 0 | — |
| `itemgroup_t::ITEM_GROUP_SPLASH` | 0 | — |
| `itemgroup_t::ITEM_GROUP_FLUID` | 0 | — |
| `itemgroup_t::ITEM_GROUP_DOOR` | 0 | — |
| `itemgroup_t::ITEM_GROUP_DEPRECATED` | 0 | — |
| `itemgroup_t::ITEM_GROUP_PODIUM` | 0 | — |
| `itemgroup_t::ITEM_GROUP_LAST` | 0 | — |
| `clientVersion_t` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_750` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_755` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_760` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_770` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_780` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_790` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_792` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_800` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_810` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_811` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_820` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_830` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_840` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_841` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_842` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_850` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_854_BAD` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_854` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_855` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_860_OLD` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_860` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_861` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_862` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_870` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_871` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_872` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_873` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_900` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_910` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_920` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_940` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_944_V1` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_944_V2` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_944_V3` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_944_V4` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_946` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_950` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_952` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_953` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_954` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_960` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_961` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_963` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_970` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_980` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_981` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_982` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_983` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_985` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_986` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1010` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1020` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1021` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1030` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1031` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1035` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1076` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1098` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1100` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1272` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1281` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1285` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1286` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1287` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1290` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_1310` | 0 | — |
| `clientVersion_t::CLIENT_VERSION_LAST` | 0 | — |
| `rootattrib_` | 0 | — |
| `rootattrib_::ROOT_ATTR_VERSION` | 0 | — |
| `itemattrib_t` | 0 | — |
| `itemattrib_t::ITEM_ATTR_FIRST` | 0 | — |
| `itemattrib_t::ITEM_ATTR_SERVERID` | 0 | — |
| `itemattrib_t::ITEM_ATTR_CLIENTID` | 0 | — |
| `itemattrib_t::ITEM_ATTR_NAME` | 0 | — |
| `itemattrib_t::ITEM_ATTR_DESCR` | 0 | — |
| `itemattrib_t::ITEM_ATTR_SPEED` | 0 | — |
| `itemattrib_t::ITEM_ATTR_SLOT` | 0 | — |
| `itemattrib_t::ITEM_ATTR_MAXITEMS` | 0 | — |
| `itemattrib_t::ITEM_ATTR_WEIGHT` | 0 | — |
| `itemattrib_t::ITEM_ATTR_WEAPON` | 0 | — |
| `itemattrib_t::ITEM_ATTR_AMU` | 0 | — |
| `itemattrib_t::ITEM_ATTR_ARMOR` | 0 | — |
| `itemattrib_t::ITEM_ATTR_MAGLEVEL` | 0 | — |
| `itemattrib_t::ITEM_ATTR_MAGFIELDTYPE` | 0 | — |
| `itemattrib_t::ITEM_ATTR_WRITEABLE` | 0 | — |
| `itemattrib_t::ITEM_ATTR_ROTATETO` | 0 | — |
| `itemattrib_t::ITEM_ATTR_DECAY` | 0 | — |
| `itemattrib_t::ITEM_ATTR_SPRITEHASH` | 0 | — |
| `itemattrib_t::ITEM_ATTR_MINIMAPCOLOR` | 0 | — |
| `itemattrib_t::ITEM_ATTR_07` | 0 | — |
| `itemattrib_t::ITEM_ATTR_08` | 0 | — |
| `itemattrib_t::ITEM_ATTR_LIGHT` | 0 | — |
| `itemattrib_t::ITEM_ATTR_DECAY2` | 0 | — |
| `itemattrib_t::ITEM_ATTR_WEAPON2` | 0 | — |
| `itemattrib_t::ITEM_ATTR_AMU2` | 0 | — |
| `itemattrib_t::ITEM_ATTR_ARMOR2` | 0 | — |
| `itemattrib_t::ITEM_ATTR_WRITEABLE2` | 0 | — |
| `itemattrib_t::ITEM_ATTR_LIGHT2` | 0 | — |
| `itemattrib_t::ITEM_ATTR_TOPORDER` | 0 | — |
| `itemattrib_t::ITEM_ATTR_WRITEABLE3` | 0 | — |
| `itemattrib_t::ITEM_ATTR_WAREID` | 0 | — |
| `itemattrib_t::ITEM_ATTR_CLASSIFICATION` | 0 | — |
| `itemattrib_t::ITEM_ATTR_LAST` | 0 | — |
| `itemflags_t` | 0 | — |
| `itemflags_t::FLAG_BLOCK_SOLID` | 0 | — |
| `itemflags_t::FLAG_BLOCK_PROJECTILE` | 0 | — |
| `itemflags_t::FLAG_BLOCK_PATHFIND` | 0 | — |
| `itemflags_t::FLAG_HAS_HEIGHT` | 0 | — |
| `itemflags_t::FLAG_USEABLE` | 0 | — |
| `itemflags_t::FLAG_PICKUPABLE` | 0 | — |
| `itemflags_t::FLAG_MOVEABLE` | 0 | — |
| `itemflags_t::FLAG_STACKABLE` | 0 | — |
| `itemflags_t::FLAG_FLOORCHANGEDOWN` | 0 | — |
| `itemflags_t::FLAG_FLOORCHANGENORTH` | 0 | — |
| `itemflags_t::FLAG_FLOORCHANGEEAST` | 0 | — |
| `itemflags_t::FLAG_FLOORCHANGESOUTH` | 0 | — |
| `itemflags_t::FLAG_FLOORCHANGEWEST` | 0 | — |
| `itemflags_t::FLAG_ALWAYSONTOP` | 0 | — |
| `itemflags_t::FLAG_READABLE` | 0 | — |
| `itemflags_t::FLAG_ROTATABLE` | 0 | — |
| `itemflags_t::FLAG_HANGABLE` | 0 | — |
| `itemflags_t::FLAG_VERTICAL` | 0 | — |
| `itemflags_t::FLAG_HORIZONTAL` | 0 | — |
| `itemflags_t::FLAG_CANNOTDECAY` | 0 | — |
| `itemflags_t::FLAG_ALLOWDISTREAD` | 0 | — |
| `itemflags_t::FLAG_CLIENTDURATION` | 0 | — |
| `itemflags_t::FLAG_CLIENTCHARGES` | 0 | — |
| `itemflags_t::FLAG_LOOKTHROUGH` | 0 | — |
| `itemflags_t::FLAG_ANIMATION` | 0 | — |
| `itemflags_t::FLAG_FULLTILE` | 0 | — |
| `itemflags_t::FLAG_FORCEUSE` | 0 | — |
| `itemflags_t::FLAG_AMMO` | 0 | — |
| `itemflags_t::FLAG_REPORTABLE` | 0 | — |
| `VERSIONINFO` | 0 | — |
| `lightBlock2` | 0 | — |

### `items.h` (header declarations)

227 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_ITEMS_H` | 0 | — |
| `SlotPositionBits` | 0 | — |
| `SlotPositionBits::SLOTP_WHEREEVER` | 0 | — |
| `SlotPositionBits::SLOTP_HEAD` | 0 | — |
| `SlotPositionBits::SLOTP_NECKLACE` | 0 | — |
| `SlotPositionBits::SLOTP_BACKPACK` | 0 | — |
| `SlotPositionBits::SLOTP_ARMOR` | 0 | — |
| `SlotPositionBits::SLOTP_RIGHT` | 0 | — |
| `SlotPositionBits::SLOTP_LEFT` | 0 | — |
| `SlotPositionBits::SLOTP_LEGS` | 0 | — |
| `SlotPositionBits::SLOTP_FEET` | 0 | — |
| `SlotPositionBits::SLOTP_RING` | 0 | — |
| `SlotPositionBits::SLOTP_AMMO` | 0 | — |
| `SlotPositionBits::SLOTP_DEPOT` | 0 | — |
| `SlotPositionBits::SLOTP_TWO_HAND` | 0 | — |
| `SlotPositionBits::SLOTP_HAND` | 0 | — |
| `ItemTypes_t` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_NONE` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_DEPOT` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_MAILBOX` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_TRASHHOLDER` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_CONTAINER` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_DOOR` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_MAGICFIELD` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_TELEPORT` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_BED` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_KEY` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_RUNE` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_PODIUM` | 0 | — |
| `ItemTypes_t::ITEM_TYPE_LAST` | 0 | — |
| `ItemParseAttributes_t` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_TYPE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_DESCRIPTION` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_RUNESPELLNAME` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_WEIGHT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SHOWCOUNT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ARMOR` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_DEFENSE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_EXTRADEF` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ATTACK` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ATTACK_SPEED` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ROTATETO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MOVEABLE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BLOCKPROJECTILE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_PICKUPABLE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FORCESERIALIZE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FLOORCHANGE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_CORPSETYPE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_CONTAINERSIZE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FLUIDSOURCE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_READABLE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_WRITEABLE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAXTEXTLEN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_WRITEONCEITEMID` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_WEAPONTYPE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SLOTTYPE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_AMMOTYPE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SHOOTTYPE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_EFFECT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_RANGE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_STOPDURATION` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_DECAYTO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_TRANSFORMEQUIPTO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_TRANSFORMDEEQUIPTO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_DURATION` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SHOWDURATION` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_CHARGES` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SHOWCHARGES` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SHOWATTRIBUTES` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_HITCHANCE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAXHITCHANCE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_INVISIBLE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SPEED` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_HEALTHGAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_HEALTHTICKS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MANAGAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MANATICKS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MANASHIELD` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SKILLSWORD` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SKILLAXE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SKILLCLUB` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SKILLDIST` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SKILLFISH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SKILLSHIELD` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SKILLFIST` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAXHITPOINTS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAXHITPOINTSPERCENT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAXMANAPOINTS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAXMANAPOINTSPERCENT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICPOINTS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICPOINTSPERCENT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_CRITICALHITCHANCE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_CRITICALHITAMOUNT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_LIFELEECHCHANCE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_LIFELEECHAMOUNT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MANALEECHCHANCE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MANALEECHAMOUNT` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FIELDABSORBPERCENTENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FIELDABSORBPERCENTFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FIELDABSORBPERCENTPOISON` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTALL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTELEMENTS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTMAGIC` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTPOISON` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTICE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTHOLY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTDEATH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTLIFEDRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTMANADRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTDROWN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTPHYSICAL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTHEALING` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ABSORBPERCENTUNDEFINED` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELPOISON` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELICE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELHOLY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELDEATH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELLIFEDRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELMANADRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELDROWN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELPHYSICAL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELHEALING` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MAGICLEVELUNDEFINED` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSDRUNK` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSPOISON` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSDROWN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSPHYSICAL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSFREEZE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSDAZZLE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPRESSCURSE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FIELD` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REPLACEABLE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_PARTNERDIRECTION` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_LEVELDOOR` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_MALETRANSFORMTO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_FEMALETRANSFORMTO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_TRANSFORMTO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_DESTROYTO` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ELEMENTICE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ELEMENTEARTH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ELEMENTFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ELEMENTENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ELEMENTDEATH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ELEMENTHOLY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_WALKSTACK` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BLOCKING` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_ALLOWDISTREAD` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_STOREITEM` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_WORTH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTALL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTELEMENTS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTMAGIC` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTEARTH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTICE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTHOLY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTDEATH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTLIFEDRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTMANADRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTDROWN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTPHYSICAL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTPERCENTHEALING` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEALL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEELEMENTS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEMAGIC` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEEARTH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEICE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEHOLY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEDEATH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCELIFEDRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEMANADRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEDROWN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEPHYSICAL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_REFLECTCHANCEHEALING` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTALL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTELEMENTS` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTMAGIC` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTENERGY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTFIRE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTEARTH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTICE` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTHOLY` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTDEATH` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTLIFEDRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTMANADRAIN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTDROWN` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTPHYSICAL` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_BOOSTPERCENTHEALING` | 0 | — |
| `ItemParseAttributes_t::ITEM_PARSE_SUPPLY` | 0 | — |
| `Abilities` | 0 | — |
| `ItemType` | 0 | — |
| `ItemType::ItemType` | 0 | — |
| `ItemType::ItemType` | 0 | — |
| `ItemType::ItemType` | 0 | — |
| `ItemType::reset` | 0 | — |
| `ItemType::pluralName` | 0 | — |
| `ItemType::name` | 0 | — |
| `ItemType::name` | 0 | — |
| `ItemType::str` | 0 | — |
| `ItemType::reserve` | 0 | — |
| `ItemType::assign` | 0 | — |
| `ItemType::str` | 0 | — |
| `Items` | 0 | — |
| `Items::Items` | 0 | — |
| `Items::Items` | 0 | — |
| `Items::reload` | 0 | — |
| `Items::clear` | 0 | — |
| `Items::loadFromOtb` | 0 | — |
| `Items::getItemType` | 0 | — |
| `Items::getItemType` | 0 | — |
| `Items::getItemIdByClientId` | 0 | — |
| `Items::getItemIdByName` | 0 | — |
| `Items::loadFromXml` | 0 | — |
| `Items::parseItemNode` | 0 | — |
| `Items::resize` | 0 | — |
| `Items::serverId` | 0 | — |
| `Items::serverId` | 0 | — |
| `Items::vec` | 0 | — |

### `items`

10 node(s), 9 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `getDirection` | 0 | — |
| `Items::clear` | 1 | `Items::clear` (static) |
| `Items::reload` | 2 | `Items::reload` (static); `Weapons::loadDefaults` (static) |
| `Items::loadFromOtb` | 1 | `Items::loadFromOtb` (static) |
| `Items::loadFromXml` | 1 | `Items::loadFromXml` (static) |
| `Items::parseItemNode` | 1 | `Items::parseItemNode` (static) |
| `Items::getItemType` | 0 | — |
| `Items::getItemType` | 1 | `Items::getItemType` (static) |
| `Items::getItemIdByClientId` | 1 | `Items::getItemIdByClientId` (static) |
| `Items::getItemIdByName` | 1 | `Items::getItemIdByName` (static) |

### `lockfree.h` (header declarations)

14 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_LOCKFREE_H` | 0 | — |
| `LockfreeFreeList` | 0 | — |
| `LockfreeFreeList::freeList` | 0 | — |
| `LockfreeFreeList::freeList` | 0 | — |
| `LockfreePoolingAllocator` | 0 | — |
| `LockfreePoolingAllocator::other` | 0 | — |
| `LockfreePoolingAllocator::LockfreePoolingAllocator` | 0 | — |
| `LockfreePoolingAllocator::inst` | 0 | — |
| `LockfreePoolingAllocator::sizeof` | 0 | — |
| `LockfreePoolingAllocator::p` | 0 | — |
| `LockfreePoolingAllocator::operator` | 0 | — |
| `LockfreePoolingAllocator::inst` | 0 | — |
| `LockfreePoolingAllocator::sizeof` | 0 | — |
| `LockfreePoolingAllocator::operator` | 0 | — |

### `luascript.h` (header declarations)

151 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `luaL_register_compat` | 0 | — |
| `Combat_ptr` | 0 | — |
| `EVENT_ID_USER` | 0 | — |
| `LuaTimerEventDesc` | 0 | — |
| `LuaTimerEventDesc::scriptId` | 0 | — |
| `LuaTimerEventDesc::function` | 0 | — |
| `LuaTimerEventDesc::parameters` | 0 | — |
| `LuaTimerEventDesc::eventId` | 0 | — |
| `LuaTimerEventDesc::LuaTimerEventDesc` | 0 | — |
| `LuaTimerEventDesc::LuaTimerEventDesc(LuaTimerEventDesc&&)` | 0 | — |
| `ScriptEnvironment` | 0 | — |
| `ScriptEnvironment::ScriptEnvironment` | 0 | — |
| `ScriptEnvironment::~ScriptEnvironment` | 0 | — |
| `ScriptEnvironment::ScriptEnvironment(const ScriptEnvironment&)` | 0 | — |
| `ScriptEnvironment::operator=` | 0 | — |
| `ScriptEnvironment::resetEnv` | 0 | — |
| `ScriptEnvironment::setScriptId` | 0 | — |
| `ScriptEnvironment::setCallbackId` | 0 | — |
| `ScriptEnvironment::getScriptId` | 0 | — |
| `ScriptEnvironment::getScriptInterface` | 0 | — |
| `ScriptEnvironment::setTimerEvent` | 0 | — |
| `ScriptEnvironment::getEventInfo` | 0 | — |
| `ScriptEnvironment::addThing` | 0 | — |
| `ScriptEnvironment::insertItem` | 0 | — |
| `ScriptEnvironment::setNpc` | 0 | — |
| `ScriptEnvironment::getNpc` | 0 | — |
| `ScriptEnvironment::getThingByUID` | 0 | — |
| `ScriptEnvironment::getItemByUID` | 0 | — |
| `ScriptEnvironment::getContainerByUID` | 0 | — |
| `ScriptEnvironment::removeItemByUID` | 0 | — |
| `ScriptEnvironment::interface` | 0 | — |
| `ScriptEnvironment::curNpc` | 0 | — |
| `ScriptEnvironment::localMap` | 0 | — |
| `ScriptEnvironment::lastUID` | 0 | — |
| `ScriptEnvironment::scriptId` | 0 | — |
| `ScriptEnvironment::callbackId` | 0 | — |
| `ScriptEnvironment::timerEvent` | 0 | — |
| `ErrorCode_t` | 0 | — |
| `ErrorCode_t::LUA_ERROR_PLAYER_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_CREATURE_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_ITEM_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_THING_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_TILE_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_HOUSE_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_COMBAT_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_CONDITION_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_AREA_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_CONTAINER_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_VARIANT_NOT_FOUND` | 0 | — |
| `ErrorCode_t::LUA_ERROR_VARIANT_UNKNOWN` | 0 | — |
| `ErrorCode_t::LUA_ERROR_SPELL_NOT_FOUND` | 0 | — |
| `LuaScriptInterface` | 0 | — |
| `LuaScriptInterface::LuaScriptInterface` | 0 | — |
| `LuaScriptInterface::~LuaScriptInterface` | 0 | — |
| `LuaScriptInterface::LuaScriptInterface(const LuaScriptInterface&)` | 0 | — |
| `LuaScriptInterface::operator=` | 0 | — |
| `LuaScriptInterface::initState` | 0 | — |
| `LuaScriptInterface::reInitState` | 0 | — |
| `LuaScriptInterface::loadFile` | 0 | — |
| `LuaScriptInterface::getFileById` | 0 | — |
| `LuaScriptInterface::getEvent(std::string_view)` | 0 | — |
| `LuaScriptInterface::getEvent()` | 0 | — |
| `LuaScriptInterface::getMetaEvent` | 0 | — |
| `LuaScriptInterface::getInterfaceName` | 0 | — |
| `LuaScriptInterface::getLastLuaError` | 0 | — |
| `LuaScriptInterface::getLuaState` | 0 | — |
| `LuaScriptInterface::pushFunction` | 0 | — |
| `LuaScriptInterface::callFunction` | 0 | — |
| `LuaScriptInterface::callVoidFunction` | 0 | — |
| `LuaScriptInterface::luaBitReg` | 0 | — |
| `LuaScriptInterface::luaConfigManagerTable` | 0 | — |
| `LuaScriptInterface::luaDatabaseTable` | 0 | — |
| `LuaScriptInterface::luaResultTable` | 0 | — |
| `LuaScriptInterface::closeState` | 0 | — |
| `LuaScriptInterface::registerFunctions` | 0 | — |
| `LuaScriptInterface::L` | 0 | — |
| `LuaScriptInterface::eventTableRef` | 0 | — |
| `LuaScriptInterface::runningEventId` | 0 | — |
| `LuaScriptInterface::cacheFiles` | 0 | — |
| `LuaScriptInterface::lastLuaError` | 0 | — |
| `LuaScriptInterface::interfaceName` | 0 | — |
| `LuaScriptInterface::loadingFile` | 0 | — |
| `LuaEnvironment` | 0 | — |
| `LuaEnvironment::LuaEnvironment` | 0 | — |
| `LuaEnvironment::~LuaEnvironment` | 0 | — |
| `LuaEnvironment::LuaEnvironment(const LuaEnvironment&)` | 0 | — |
| `LuaEnvironment::operator=` | 0 | — |
| `LuaEnvironment::initState` | 0 | — |
| `LuaEnvironment::reInitState` | 0 | — |
| `LuaEnvironment::closeState` | 0 | — |
| `LuaEnvironment::getTestInterface` | 0 | — |
| `LuaEnvironment::getCombatObject` | 0 | — |
| `LuaEnvironment::createCombatObject` | 0 | — |
| `LuaEnvironment::clearCombatObjects` | 0 | — |
| `LuaEnvironment::getAreaObject` | 0 | — |
| `LuaEnvironment::createAreaObject` | 0 | — |
| `LuaEnvironment::clearAreaObjects` | 0 | — |
| `LuaEnvironment::executeTimerEvent` | 0 | — |
| `LuaEnvironment::timerEvents` | 0 | — |
| `LuaEnvironment::combatMap` | 0 | — |
| `LuaEnvironment::areaMap` | 0 | — |
| `LuaEnvironment::combatIdMap` | 0 | — |
| `LuaEnvironment::areaIdMap` | 0 | — |
| `LuaEnvironment::testInterface` | 0 | — |
| `LuaEnvironment::lastEventTimerId` | 0 | — |
| `LuaEnvironment::lastCombatId` | 0 | — |
| `LuaEnvironment::lastAreaId` | 0 | — |
| `LuaEnvironment::friend_LuaScriptInterface` | 0 | — |
| `LuaEnvironment::friend_CombatSpell` | 0 | — |
| `tfs::lua` | 0 | — |
| `tfs::lua::removeTempItem` | 0 | — |
| `tfs::lua::getScriptEnv` | 0 | — |
| `tfs::lua::reserveScriptEnv` | 0 | — |
| `tfs::lua::resetScriptEnv` | 0 | — |
| `tfs::lua::reportError` | 0 | — |
| `reportErrorFunc` | 0 | — |
| `tfs::lua::pushThing` | 0 | — |
| `tfs::lua::pushVariant` | 0 | — |
| `tfs::lua::pushString` | 0 | — |
| `tfs::lua::pushCallback` | 0 | — |
| `tfs::lua::popString` | 0 | — |
| `tfs::lua::popCallback` | 0 | — |
| `tfs::lua::pushUserdata` | 0 | — |
| `tfs::lua::setMetatable` | 0 | — |
| `tfs::lua::setItemMetatable` | 0 | — |
| `tfs::lua::setCreatureMetatable` | 0 | — |
| `tfs::lua::getNumber<enum T>` | 0 | — |
| `tfs::lua::getNumber<unsigned T>` | 0 | — |
| `tfs::lua::getNumber<signed/floating T>` | 0 | — |
| `tfs::lua::getNumber<T,default>` | 0 | — |
| `tfs::lua::getRawUserdata` | 0 | — |
| `tfs::lua::getUserdata` | 0 | — |
| `tfs::lua::getBoolean` | 0 | — |
| `tfs::lua::getBoolean(default)` | 0 | — |
| `tfs::lua::getString` | 0 | — |
| `tfs::lua::getPosition` | 0 | — |
| `tfs::lua::getPosition(stackpos)` | 0 | — |
| `tfs::lua::getThing` | 0 | — |
| `tfs::lua::getCreature` | 0 | — |
| `tfs::lua::getPlayer` | 0 | — |
| `tfs::lua::getField` | 0 | — |
| `tfs::lua::getField(default)` | 0 | — |
| `tfs::lua::getFieldString` | 0 | — |
| `tfs::lua::pushBoolean` | 0 | — |
| `tfs::lua::pushSpell` | 0 | — |
| `tfs::lua::pushPosition` | 0 | — |
| `tfs::lua::pushOutfit(Outfit_t)` | 0 | — |
| `tfs::lua::pushOutfit(Outfit*)` | 0 | — |
| `tfs::lua::protectedCall` | 0 | — |
| `tfs::lua::registerMethod` | 0 | — |
| `tfs::lua::getErrorDesc` | 0 | — |

### `luascript`

240 node(s), 188 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_luaEnvironment` | 0 | — |
| `EVENT_ID_LOADING` | 0 | — |
| `(anonymous)::LuaDataType` | 0 | — |
| `LuaDataType::LuaData_Unknown` | 0 | — |
| `LuaDataType::LuaData_Item` | 0 | — |
| `LuaDataType::LuaData_Container` | 0 | — |
| `LuaDataType::LuaData_Teleport` | 0 | — |
| `LuaDataType::LuaData_Podium` | 0 | — |
| `LuaDataType::LuaData_Player` | 0 | — |
| `LuaDataType::LuaData_Monster` | 0 | — |
| `LuaDataType::LuaData_Npc` | 0 | — |
| `LuaDataType::LuaData_Tile` | 0 | — |
| `(anonymous)::tempItems` | 0 | — |
| `(anonymous)::lastResultId` | 0 | — |
| `(anonymous)::tempResults` | 0 | — |
| `(anonymous)::isNumber` | 0 | — |
| `(anonymous)::setField(number)` | 0 | — |
| `(anonymous)::setField(string)` | 1 | `tfs::lua::pushString` (static) |
| `(anonymous)::registerClass` | 0 | — |
| `(anonymous)::registerTable` | 0 | — |
| `(anonymous)::registerMetaMethod` | 0 | — |
| `(anonymous)::registerGlobalMethod` | 0 | — |
| `(anonymous)::registerVariable` | 0 | — |
| `(anonymous)::registerGlobalVariable` | 0 | — |
| `(anonymous)::registerGlobalBoolean` | 1 | `tfs::lua::pushBoolean` (static) |
| `(anonymous)::getStackTrace` | 1 | `tfs::lua::popString` (static) |
| `(anonymous)::luaErrorHandler` | 2 | `tfs::lua::popString` (static); `tfs::lua::pushString` (static) |
| `(anonymous)::getArea` | 0 | — |
| `(anonymous)::getSharedPtr` | 0 | — |
| `(anonymous)::pushSharedPtr` | 0 | — |
| `ScriptEnvironment::ScriptEnvironment` | 1 | `ScriptEnvironment::ScriptEnvironment` (static) |
| `ScriptEnvironment::~ScriptEnvironment` | 0 | — |
| `ScriptEnvironment::resetEnv` | 2 | `Game::ReleaseItem` (static); `ScriptEnvironment::resetEnv` (static) |
| `ScriptEnvironment::setCallbackId` | 1 | `ScriptEnvironment::setCallbackId` (static) |
| `ScriptEnvironment::addThing` | 1 | `ScriptEnvironment::addThing` (static) |
| `ScriptEnvironment::insertItem` | 1 | `ScriptEnvironment::insertItem` (static) |
| `ScriptEnvironment::getThingByUID` | 3 | `Game::getCreatureByID` (static); `Game::getUniqueItem` (static); `ScriptEnvironment::getThingByUID` (static) |
| `ScriptEnvironment::getItemByUID` | 1 | `ScriptEnvironment::getItemByUID` (static) |
| `ScriptEnvironment::getContainerByUID` | 1 | `ScriptEnvironment::getContainerByUID` (static) |
| `ScriptEnvironment::removeItemByUID` | 2 | `Game::removeUniqueItem` (static); `ScriptEnvironment::removeItemByUID` (static) |
| `addTempItem` | 1 | `tfs::lua::getScriptEnv` (static) |
| `tfs::lua::removeTempItem` | 1 | `tfs::lua::removeTempItem` (static) |
| `addResult` | 0 | — |
| `removeResult` | 0 | — |
| `getResultByID` | 0 | — |
| `tfs::lua::getErrorDesc` | 1 | `tfs::lua::getErrorDesc` (static) |
| `(file-static)::scriptEnv` | 0 | — |
| `(file-static)::scriptEnvIndex` | 0 | — |
| `LuaScriptInterface::LuaScriptInterface` | 2 | `LuaEnvironment::initState` (static); `LuaScriptInterface::LuaScriptInterface` (static) |
| `LuaScriptInterface::~LuaScriptInterface` | 0 | — |
| `LuaScriptInterface::reInitState` | 3 | `LuaEnvironment::clearAreaObjects` (static); `LuaEnvironment::clearCombatObjects` (static); `LuaScriptInterface::reInitState` (static) |
| `tfs::lua::protectedCall` | 1 | `tfs::lua::protectedCall` (static) |
| `LuaScriptInterface::loadFile` | 6 | `LuaScriptInterface::loadFile` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::popString` (static); `tfs::lua::protectedCall` (static); `tfs::lua::reserveScriptEnv` (static); … +1 more |
| `LuaScriptInterface::getEvent(std::string_view)` | 0 | — |
| `LuaScriptInterface::getEvent()` | 0 | — |
| `LuaScriptInterface::getMetaEvent` | 1 | `LuaScriptInterface::getMetaEvent` (static) |
| `LuaScriptInterface::getFileById` | 1 | `LuaScriptInterface::getFileById` (static) |
| `tfs::lua::reportError` | 1 | `tfs::lua::reportError` (static) |
| `LuaScriptInterface::pushFunction` | 1 | `LuaScriptInterface::pushFunction` (static) |
| `LuaScriptInterface::initState` | 1 | `LuaScriptInterface::initState` (static) |
| `LuaScriptInterface::closeState` | 1 | `LuaScriptInterface::closeState` (static) |
| `LuaScriptInterface::callFunction` | 5 | `LuaScriptInterface::callFunction` (static); `tfs::lua::getBoolean` (static); `tfs::lua::getString` (static); `tfs::lua::protectedCall` (static); `tfs::lua::resetScriptEnv` (static) |
| `LuaScriptInterface::callVoidFunction` | 4 | `LuaScriptInterface::callVoidFunction` (static); `tfs::lua::popString` (static); `tfs::lua::protectedCall` (static); `tfs::lua::resetScriptEnv` (static) |
| `tfs::lua::pushVariant` | 1 | `tfs::lua::pushVariant` (static) |
| `tfs::lua::pushThing` | 1 | `tfs::lua::pushThing` (static) |
| `tfs::lua::pushString` | 1 | `tfs::lua::pushString` (static) |
| `tfs::lua::pushCallback` | 1 | `tfs::lua::pushCallback` (static) |
| `tfs::lua::popString` | 1 | `tfs::lua::popString` (static) |
| `tfs::lua::popCallback` | 1 | `tfs::lua::popCallback` (static) |
| `tfs::lua::setMetatable` | 1 | `tfs::lua::setMetatable` (static) |
| `(file-static)::setWeakMetatable` | 0 | — |
| `tfs::lua::setItemMetatable` | 1 | `tfs::lua::setItemMetatable` (static) |
| `tfs::lua::setCreatureMetatable` | 1 | `tfs::lua::setCreatureMetatable` (static) |
| `tfs::lua::getString` | 1 | `tfs::lua::getString` (static) |
| `tfs::lua::getPosition(stackpos)` | 1 | `tfs::lua::getPosition` (static) |
| `tfs::lua::getPosition` | 1 | `tfs::lua::getPosition` (static) |
| `(file-static)::getOutfit` | 1 | `tfs::lua::getField` (static) |
| `(file-static)::getOutfitClass` | 2 | `tfs::lua::getFieldString` (static); `tfs::lua::getField` (static) |
| `(file-static)::getVariant` | 3 | `tfs::lua::getFieldString` (static); `tfs::lua::getPosition` (static); `tfs::lua::getField` (static) |
| `tfs::lua::getThing` | 1 | `tfs::lua::getThing` (static) |
| `tfs::lua::getCreature` | 2 | `Game::getCreatureByID` (static); `tfs::lua::getCreature` (static) |
| `tfs::lua::getPlayer` | 2 | `Game::getPlayerByID` (static); `tfs::lua::getPlayer` (static) |
| `tfs::lua::getFieldString` | 1 | `tfs::lua::getFieldString` (static) |
| `(file-static)::getUserdataType` | 0 | — |
| `tfs::lua::pushBoolean` | 1 | `tfs::lua::pushBoolean` (static) |
| `tfs::lua::pushSpell` | 1 | `tfs::lua::pushSpell` (static) |
| `tfs::lua::pushPosition` | 1 | `tfs::lua::pushPosition` (static) |
| `tfs::lua::pushOutfit(Outfit_t)` | 0 | — |
| `tfs::lua::pushOutfit(Outfit*)` | 0 | — |
| `(file-static)::pushLoot` | 0 | — |
| `(file-static)::pushTown` | 1 | `tfs::lua::pushPosition` (static) |
| `registerEnum` | 0 | — |
| `registerEnumIn` | 0 | — |
| `LuaScriptInterface::registerFunctions` | 1 | `LuaScriptInterface::registerFunctions` (static) |
| `lua:doPlayerAddItem` | 0 | — |
| `lua:isValidUID` | 0 | — |
| `lua:isDepot` | 0 | — |
| `lua:isMovable` | 0 | — |
| `lua:getDepotId` | 0 | — |
| `lua:getWorldUpTime` | 0 | — |
| `lua:getSubTypeName` | 0 | — |
| `lua:createCombatArea` | 0 | — |
| `lua:doAreaCombat` | 0 | — |
| `lua:doTargetCombat` | 0 | — |
| `lua:doChallengeCreature` | 0 | — |
| `lua:addEvent` | 0 | — |
| `lua:stopEvent` | 0 | — |
| `lua:saveServer` | 0 | — |
| `lua:cleanMap` | 0 | — |
| `lua:debugPrint` | 0 | — |
| `lua:isInWar` | 0 | — |
| `lua:getWaypointPositionByName` | 0 | — |
| `lua:sendChannelMessage` | 0 | — |
| `lua:sendGuildChannelMessage` | 0 | — |
| `lua:isScriptsInterface` | 0 | — |
| `lua:INDEX_WHEREEVER` | 0 | — |
| `lua:VIRTUAL_PARENT` | 0 | — |
| `lua:isType` | 0 | — |
| `lua:rawgetmetatable` | 0 | — |
| `lua:configKeys` | 0 | — |
| `lua:DBInsert` | 0 | — |
| `lua:DBInsert.__gc` | 0 | — |
| `lua:DBTransaction` | 0 | — |
| `lua:DBTransaction.__eq` | 0 | — |
| `lua:DBTransaction.__gc` | 0 | — |
| `lua:Game` | 0 | — |
| `lua:Variant` | 0 | — |
| `lua:Position` | 0 | — |
| `lua:Tile` | 0 | — |
| `lua:Tile.__eq` | 0 | — |
| `lua:NetworkMessage` | 0 | — |
| `lua:NetworkMessage.__eq` | 0 | — |
| `lua:NetworkMessage.__gc` | 0 | — |
| `lua:ModalWindow` | 0 | — |
| `lua:ModalWindow.__eq` | 0 | — |
| `lua:ModalWindow.__gc` | 0 | — |
| `lua:Item` | 0 | — |
| `lua:Item.__eq` | 0 | — |
| `lua:Container` | 0 | — |
| `lua:Container.__eq` | 0 | — |
| `lua:Teleport` | 0 | — |
| `lua:Teleport.__eq` | 0 | — |
| `lua:Podium` | 0 | — |
| `lua:Podium.__eq` | 0 | — |
| `lua:Creature` | 0 | — |
| `lua:Creature.__eq` | 0 | — |
| `lua:Player` | 0 | — |
| `lua:Player.__eq` | 0 | — |
| `lua:Monster` | 0 | — |
| `lua:Monster.__eq` | 0 | — |
| `lua:Npc` | 0 | — |
| `lua:Npc.__eq` | 0 | — |
| `lua:Guild` | 0 | — |
| `lua:Guild.__eq` | 0 | — |
| `lua:Group` | 0 | — |
| `lua:Group.__eq` | 0 | — |
| `lua:Vocation` | 0 | — |
| `lua:Vocation.__eq` | 0 | — |
| `lua:House` | 0 | — |
| `lua:House.__eq` | 0 | — |
| `lua:ItemType` | 0 | — |
| `lua:ItemType.__eq` | 0 | — |
| `lua:Combat` | 0 | — |
| `lua:Combat.__eq` | 0 | — |
| `lua:Combat.__gc` | 0 | — |
| `lua:Condition` | 0 | — |
| `lua:Condition.__eq` | 0 | — |
| `lua:Condition.__gc` | 0 | — |
| `lua:Outfit` | 0 | — |
| `lua:Outfit.__eq` | 0 | — |
| `lua:MonsterType` | 0 | — |
| `lua:MonsterType.__eq` | 0 | — |
| `lua:Loot` | 0 | — |
| `lua:Loot.__gc` | 0 | — |
| `lua:MonsterSpell` | 0 | — |
| `lua:MonsterSpell.__gc` | 0 | — |
| `lua:Party` | 0 | — |
| `lua:Party.__eq` | 0 | — |
| `lua:Spell` | 0 | — |
| `lua:Spell.__eq` | 0 | — |
| `lua:Action` | 0 | — |
| `lua:TalkAction` | 0 | — |
| `lua:CreatureEvent` | 0 | — |
| `lua:MoveEvent` | 0 | — |
| `lua:GlobalEvent` | 0 | — |
| `lua:Weapon` | 0 | — |
| `lua:XMLDocument` | 0 | — |
| `lua:XMLDocument.__gc` | 0 | — |
| `lua:XMLNode` | 0 | — |
| `lua:XMLNode.__gc` | 0 | — |
| `tfs::lua::getScriptEnv` | 1 | `tfs::lua::getScriptEnv` (static) |
| `tfs::lua::reserveScriptEnv` | 1 | `tfs::lua::reserveScriptEnv` (static) |
| `tfs::lua::resetScriptEnv` | 1 | `tfs::lua::resetScriptEnv` (static) |
| `tfs::lua::getBoolean` | 1 | `tfs::lua::getBoolean` (static) |
| `tfs::lua::getBoolean(default)` | 1 | `tfs::lua::getBoolean` (static) |
| `tfs::lua::registerMethod` | 1 | `tfs::lua::registerMethod` (static) |
| `LuaScriptInterface::luaDoPlayerAddItem` | 8 | `Game::internalPlayerAddItem` (static); `Item::CreateItem` (static); `LuaScriptInterface::luaDoPlayerAddItem` (static); `tfs::lua::getBoolean` (static); `tfs::lua::getErrorDesc` (static); … +3 more |
| `LuaScriptInterface::luaDebugPrint` | 2 | `LuaScriptInterface::luaDebugPrint` (static); `tfs::lua::getString` (static) |
| `LuaScriptInterface::luaGetWorldUpTime` | 1 | `LuaScriptInterface::luaGetWorldUpTime` (static) |
| `LuaScriptInterface::luaGetSubTypeName` | 2 | `LuaScriptInterface::luaGetSubTypeName` (static); `tfs::lua::pushString` (static) |
| `LuaScriptInterface::luaCreateCombatArea` | 5 | `LuaEnvironment::createAreaObject` (static); `LuaEnvironment::getAreaObject` (static); `LuaScriptInterface::luaCreateCombatArea` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaDoAreaCombat` | 8 | `Combat::doAreaCombat` (static); `LuaEnvironment::getAreaObject` (static); `LuaScriptInterface::luaDoAreaCombat` (static); `tfs::lua::getBoolean` (static); `tfs::lua::getCreature` (static); … +3 more |
| `LuaScriptInterface::luaDoTargetCombat` | 6 | `Combat::doTargetCombat` (static); `LuaScriptInterface::luaDoTargetCombat` (static); `tfs::lua::getBoolean` (static); `tfs::lua::getCreature` (static); `tfs::lua::getErrorDesc` (static); … +1 more |
| `LuaScriptInterface::luaDoChallengeCreature` | 5 | `LuaScriptInterface::luaDoChallengeCreature` (static); `tfs::lua::getBoolean` (static); `tfs::lua::getCreature` (static); `tfs::lua::getErrorDesc` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaIsValidUID` | 3 | `LuaScriptInterface::luaIsValidUID` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaIsDepot` | 3 | `LuaScriptInterface::luaIsDepot` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaIsMoveable` | 3 | `LuaScriptInterface::luaIsMoveable` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaGetDepotId` | 4 | `LuaScriptInterface::luaGetDepotId` (static); `tfs::lua::getErrorDesc` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaAddEvent` | 5 | `ConfigManager::getBoolean` (static); `LuaEnvironment::executeTimerEvent` (static); `LuaScriptInterface::luaAddEvent` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaStopEvent` | 2 | `LuaScriptInterface::luaStopEvent` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaSaveServer` | 4 | `Game::saveGameState` (static); `GlobalEvents::save` (static); `LuaScriptInterface::luaSaveServer` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaCleanMap` | 1 | `LuaScriptInterface::luaCleanMap` (static) |
| `LuaScriptInterface::luaIsInWar` | 4 | `LuaScriptInterface::luaIsInWar` (static); `tfs::lua::getErrorDesc` (static); `tfs::lua::getPlayer` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaGetWaypointPositionByName` | 4 | `LuaScriptInterface::luaGetWaypointPositionByName` (static); `tfs::lua::getString` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::pushPosition` (static) |
| `LuaScriptInterface::luaSendChannelMessage` | 4 | `Chat::getChannelById` (static); `LuaScriptInterface::luaSendChannelMessage` (static); `tfs::lua::getString` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaSendGuildChannelMessage` | 4 | `Chat::getGuildChannelById` (static); `LuaScriptInterface::luaSendGuildChannelMessage` (static); `tfs::lua::getString` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaIsScriptsInterface` | 4 | `LuaScriptInterface::luaIsScriptsInterface` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `Scripts::getScriptInterface` (static) |
| `LuaScriptInterface::luaBitReg` | 0 | — |
| `LuaScriptInterface::luaBitNot` | 1 | `LuaScriptInterface::luaBitNot` (static) |
| `MULTIOP` | 0 | — |
| `SHIFTOP` | 0 | — |
| `LuaScriptInterface::luaConfigManagerTable` | 0 | — |
| `LuaScriptInterface::luaConfigManagerGetString` | 3 | `ConfigManager::getString` (static); `LuaScriptInterface::luaConfigManagerGetString` (static); `tfs::lua::pushString` (static) |
| `LuaScriptInterface::luaConfigManagerGetNumber` | 2 | `ConfigManager::getNumber` (static); `LuaScriptInterface::luaConfigManagerGetNumber` (static) |
| `LuaScriptInterface::luaConfigManagerGetBoolean` | 3 | `ConfigManager::getBoolean` (static); `LuaScriptInterface::luaConfigManagerGetBoolean` (static); `tfs::lua::pushBoolean` (static) |
| `LuaScriptInterface::luaDatabaseTable` | 0 | — |
| `LuaScriptInterface::luaResultTable` | 0 | — |
| `LuaEnvironment::LuaEnvironment` | 1 | `LuaEnvironment::LuaEnvironment` (static) |
| `LuaEnvironment::~LuaEnvironment` | 0 | — |
| `LuaEnvironment::initState` | 1 | `LuaEnvironment::initState` (static) |
| `LuaEnvironment::reInitState` | 1 | `LuaEnvironment::reInitState` (static) |
| `LuaEnvironment::closeState` | 1 | `LuaEnvironment::closeState` (static) |
| `LuaEnvironment::getTestInterface` | 1 | `LuaEnvironment::getTestInterface` (static) |
| `LuaEnvironment::getCombatObject` | 1 | `LuaEnvironment::getCombatObject` (static) |
| `LuaEnvironment::createCombatObject` | 1 | `LuaEnvironment::createCombatObject` (static) |
| `LuaEnvironment::clearCombatObjects` | 1 | `LuaEnvironment::clearCombatObjects` (static) |
| `LuaEnvironment::getAreaObject` | 1 | `LuaEnvironment::getAreaObject` (static) |
| `LuaEnvironment::createAreaObject` | 1 | `LuaEnvironment::createAreaObject` (static) |
| `LuaEnvironment::clearAreaObjects` | 1 | `LuaEnvironment::clearAreaObjects` (static) |
| `LuaEnvironment::executeTimerEvent` | 3 | `LuaEnvironment::executeTimerEvent` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static) |

### `luavariant.h` (header declarations)

21 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `LuaVariantType_t` | 0 | — |
| `LuaVariantType_t::VARIANT_NUMBER` | 0 | — |
| `LuaVariantType_t::VARIANT_POSITION` | 0 | — |
| `LuaVariantType_t::VARIANT_TARGETPOSITION` | 0 | — |
| `LuaVariantType_t::VARIANT_STRING` | 0 | — |
| `LuaVariantType_t::VARIANT_NONE` | 0 | — |
| `LuaVariant` | 0 | — |
| `LuaVariant::getNumber` | 0 | — |
| `LuaVariant::getPosition` | 0 | — |
| `LuaVariant::getTargetPosition` | 0 | — |
| `LuaVariant::getString` | 0 | — |
| `LuaVariant::isNumber` | 0 | — |
| `LuaVariant::isPosition` | 0 | — |
| `LuaVariant::isTargetPosition` | 0 | — |
| `LuaVariant::isString` | 0 | — |
| `LuaVariant::setNumber` | 0 | — |
| `LuaVariant::setPosition` | 0 | — |
| `LuaVariant::setTargetPosition` | 0 | — |
| `LuaVariant::setString` | 0 | — |
| `LuaVariant::type` | 0 | — |
| `LuaVariant::variant` | 0 | — |

### `mailbox.h` (header declarations)

9 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Mailbox` | 0 | — |
| `Mailbox::Mailbox` | 0 | — |
| `Mailbox::addThing (Thing*)` | 0 | — |
| `Mailbox::getMailbox (const)` | 0 | — |
| `Mailbox::getMailbox (non-const)` | 0 | — |
| `Mailbox::getReceiver (const)` | 0 | — |
| `Mailbox::getReceiver (non-const)` | 0 | — |
| `Mailbox::queryDestination` | 0 | — |
| `FS_MAILBOX_H` | 0 | — |

### `mailbox`

16 node(s), 15 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Mailbox::addThing (int32_t, Thing*)` | 0 | — |
| `Mailbox::canSend (static)` | 0 | — |
| `Mailbox::getReceiver (private)` | 0 | — |
| `Mailbox::postAddNotification` | 0 | — |
| `Mailbox::postRemoveNotification` | 0 | — |
| `Mailbox::queryAdd` | 0 | — |
| `Mailbox::queryMaxCount` | 0 | — |
| `Mailbox::sendItem` | 0 | — |
| `Mailbox::queryAdd` | 2 | `Mailbox::canSend` (static); `Mailbox::queryAdd` (static) |
| `Mailbox::queryMaxCount` | 1 | `Mailbox::queryMaxCount` (static) |
| `Mailbox::addThing` | 2 | `Mailbox::addThing` (static); `Mailbox::canSend` (static) |
| `Mailbox::postAddNotification` | 1 | `Mailbox::postAddNotification` (static) |
| `Mailbox::postRemoveNotification` | 1 | `Mailbox::postRemoveNotification` (static) |
| `Mailbox::sendItem` | 6 | `Game::getPlayerByName` (static); `Game::internalMoveItem` (static); `Game::transformItem` (static); `IOLoginData::loadPlayerByName` (static); `IOLoginData::savePlayer` (static); … +1 more |
| `Mailbox::getReceiver` | 1 | `Mailbox::getReceiver` (static) |
| `Mailbox::canSend` | 1 | `Mailbox::canSend` (static) |

### `main`

2 node(s), 5 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `argumentsHandler` | 3 | `printServerVersion` (static/curated); `ConfigManager::setNumber` (static); `ConfigManager::setString` (static) |
| `main` | 2 | `argumentsHandler` (static/curated); `startServer` (static/curated) |

### `map.h` (header declarations)

34 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_MAP_H` | 0 | — |
| `AStarNode` | 0 | — |
| `AStarNodes` | 0 | — |
| `AStarNodes::AStarNodes` | 0 | — |
| `AStarNodes::createNode` | 0 | — |
| `AStarNodes::getBestNode` | 0 | — |
| `AStarNodes::getNodeByPosition` | 0 | — |
| `AStarNodes::getMapWalkCost` | 0 | — |
| `AStarNodes::getTileWalkCost` | 0 | — |
| `Floor` | 0 | — |
| `Floor::Floor` | 0 | — |
| `Floor::Floor` | 0 | — |
| `Floor::Floor` | 0 | — |
| `QTreeNode` | 0 | — |
| `QTreeNode::QTreeNode` | 0 | — |
| `QTreeNode::QTreeNode` | 0 | — |
| `QTreeNode::QTreeNode` | 0 | — |
| `QTreeNode::getLeaf` | 0 | — |
| `QTreeNode::nullptr` | 0 | — |
| `QTreeNode::while` | 0 | — |
| `QTreeNode::createLeaf` | 0 | — |
| `Map` | 0 | — |
| `Map::clean` | 0 | — |
| `Map::loadMap` | 0 | — |
| `Map::save` | 0 | — |
| `Map::getTile` | 0 | — |
| `Map::setTile` | 0 | — |
| `Map::removeTile` | 0 | — |
| `Map::moveCreature` | 0 | — |
| `Map::clearSpectatorCache` | 0 | — |
| `Map::clearPlayersSpectatorCache` | 0 | — |
| `Map::isTileClear` | 0 | — |
| `Map::checkSightLine` | 0 | — |
| `Map::canWalkTo` | 0 | — |

### `map`

31 node(s), 39 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Map::loadMap` | 3 | `IOMapSerialize::loadHouseInfo` (static); `IOMapSerialize::loadHouseItems` (static); `Map::loadMap` (static) |
| `Map::save` | 3 | `IOMapSerialize::saveHouseInfo` (static); `IOMapSerialize::saveHouseItems` (static); `Map::save` (static) |
| `Map::getTile` | 1 | `Map::getTile` (static) |
| `Map::setTile` | 1 | `Map::setTile` (static) |
| `Map::removeTile` | 4 | `Game::internalRemoveItem` (static); `Game::internalTeleport` (static); `Game::removeCreature` (static); `Map::removeTile` (static) |
| `Map::placeCreature` | 1 | `Map::placeCreature` (static) |
| `Map::moveCreature` | 1 | `Map::moveCreature` (static) |
| `Map::getSpectatorsInternal` | 1 | `Map::getSpectatorsInternal` (static) |
| `Map::getSpectators` | 1 | `Map::getSpectators` (static) |
| `Map::clearSpectatorCache` | 1 | `Map::clearSpectatorCache` (static) |
| `Map::clearPlayersSpectatorCache` | 1 | `Map::clearPlayersSpectatorCache` (static) |
| `Map::canThrowObjectTo` | 1 | `Map::canThrowObjectTo` (static) |
| `Map::isTileClear` | 1 | `Map::isTileClear` (static) |
| `checkSteepLine` | 0 | — |
| `checkSlightLine` | 0 | — |
| `Map::checkSightLine` | 1 | `Map::checkSightLine` (static) |
| `Map::isSightClear` | 1 | `Map::isSightClear` (static) |
| `Map::canWalkTo` | 1 | `Map::canWalkTo` (static) |
| `calculateHeuristic` | 0 | — |
| `Map::getPathMatching` | 1 | `Map::getPathMatching` (static) |
| `AStarNodes::createNode` | 1 | `AStarNodes::createNode` (static) |
| `AStarNodes::getBestNode` | 1 | `AStarNodes::getBestNode` (static) |
| `AStarNodes::getNodeByPosition` | 1 | `AStarNodes::getNodeByPosition` (static) |
| `AStarNodes::getMapWalkCost` | 1 | `AStarNodes::getMapWalkCost` (static) |
| `AStarNodes::getTileWalkCost` | 2 | `Combat::DamageToConditionType` (static); `AStarNodes::getTileWalkCost` (static) |
| `QTreeNode::getLeaf` | 1 | `QTreeNode::getLeaf` (static) |
| `QTreeNode::createLeaf` | 1 | `QTreeNode::createLeaf` (static) |
| `QTreeLeafNode::createFloor` | 1 | `QTreeLeafNode::createFloor` (static) |
| `QTreeLeafNode::addCreature` | 1 | `QTreeLeafNode::addCreature` (static) |
| `QTreeLeafNode::removeCreature` | 1 | `QTreeLeafNode::removeCreature` (static) |
| `Map::clean` | 4 | `Game::getGameState` (static); `Game::internalRemoveItem` (static); `Game::setGameState` (static); `Map::clean` (static) |

### `matrixarea.h` (header declarations)

21 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `MatrixArea` | 0 | — |
| `MatrixArea::Center` | 0 | — |
| `MatrixArea::Container` | 0 | — |
| `MatrixArea::MatrixArea()` | 0 | — |
| `MatrixArea::MatrixArea(uint32_t,uint32_t)` | 0 | — |
| `MatrixArea::operator()(uint32_t,uint32_t) const` | 0 | — |
| `MatrixArea::operator()(uint32_t,uint32_t)` | 0 | — |
| `MatrixArea::setCenter` | 0 | — |
| `MatrixArea::getCenter` | 0 | — |
| `MatrixArea::getRows` | 0 | — |
| `MatrixArea::getCols` | 0 | — |
| `MatrixArea::rotate90` | 0 | — |
| `MatrixArea::rotate180` | 0 | — |
| `MatrixArea::rotate270` | 0 | — |
| `MatrixArea::operator bool` | 0 | — |
| `MatrixArea::MatrixArea(Center,uint32_t,uint32_t,Container&&)` | 0 | — |
| `MatrixArea::arr` | 0 | — |
| `MatrixArea::center` | 0 | — |
| `MatrixArea::cols` | 0 | — |
| `MatrixArea::rows` | 0 | — |
| `createArea` | 0 | — |

### `matrixarea`

4 node(s), 3 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `MatrixArea::rotate90` | 1 | `MatrixArea::rotate90` (static) |
| `MatrixArea::rotate180` | 1 | `MatrixArea::rotate180` (static) |
| `MatrixArea::rotate270` | 1 | `MatrixArea::rotate270` (static) |
| `createArea` | 0 | — |

### `monster.h` (header declarations)

6 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_MONSTER_H` | 0 | — |
| `TargetSearchType_t` | 0 | — |
| `TargetSearchType_t::TARGETSEARCH_DEFAULT` | 0 | — |
| `TargetSearchType_t::TARGETSEARCH_RANDOM` | 0 | — |
| `TargetSearchType_t::TARGETSEARCH_ATTACKRANGE` | 0 | — |
| `TargetSearchType_t::TARGETSEARCH_NEAREST` | 0 | — |

### `monster`

66 node(s), 136 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Monster::createMonster` | 2 | `Monster::createMonster` (static); `Monsters::getMonsterType` (static) |
| `Monster::addList` | 2 | `Game::addMonster` (static); `Monster::addList` (static) |
| `Monster::removeList` | 2 | `Game::removeMonster` (static); `Monster::removeList` (static) |
| `Monster::getName` | 1 | `Monster::getName` (static) |
| `Monster::setName` | 2 | `Game::updateKnownCreature` (static); `Monster::setName` (static) |
| `Monster::getNameDescription` | 1 | `Monster::getNameDescription` (static) |
| `Monster::canSee` | 2 | `Creature::canSee` (static); `Monster::canSee` (static) |
| `Monster::canWalkOnFieldType` | 1 | `Monster::canWalkOnFieldType` (static) |
| `Monster::onAttackedCreatureDisappear` | 1 | `Monster::onAttackedCreatureDisappear` (static) |
| `Monster::onCreatureAppear` | 6 | `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `Monster::onRemoveCreature` | 7 | `Creature::onRemoveCreature` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::setMetatable` (static); … +2 more |
| `Monster::onCreatureMove` | 8 | `Creature::onCreatureMove` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushPosition` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); … +3 more |
| `Monster::onCreatureSay` | 8 | `Creature::onCreatureSay` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushString` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); … +3 more |
| `Monster::addFriend` | 1 | `Monster::addFriend` (static) |
| `Monster::removeFriend` | 1 | `Monster::removeFriend` (static) |
| `Monster::addTarget` | 1 | `Monster::addTarget` (static) |
| `Monster::removeTarget` | 1 | `Monster::removeTarget` (static) |
| `Monster::updateTargetList` | 1 | `Monster::updateTargetList` (static) |
| `Monster::clearTargetList` | 1 | `Monster::clearTargetList` (static) |
| `Monster::clearFriendList` | 1 | `Monster::clearFriendList` (static) |
| `Monster::onCreatureFound` | 1 | `Monster::onCreatureFound` (static) |
| `Monster::onCreatureEnter` | 1 | `Monster::onCreatureEnter` (static) |
| `Monster::isFriend` | 1 | `Monster::isFriend` (static) |
| `Monster::isOpponent` | 1 | `Monster::isOpponent` (static) |
| `Monster::onCreatureLeave` | 1 | `Monster::onCreatureLeave` (static) |
| `Monster::searchTarget` | 1 | `Monster::searchTarget` (static) |
| `Monster::goToFollowCreature` | 2 | `Monster::goToFollowCreature` (static); `Monster::onFollowCreatureComplete` (static) |
| `Monster::onFollowCreatureComplete` | 1 | `Monster::onFollowCreatureComplete` (static) |
| `Monster::blockHit` | 2 | `Creature::blockHit` (static); `Monster::blockHit` (static) |
| `Monster::isTarget` | 1 | `Monster::isTarget` (static) |
| `Monster::selectTarget` | 2 | `Game::checkCreatureAttack` (static); `Monster::selectTarget` (static) |
| `Monster::setIdle` | 3 | `Game::addCreatureCheck` (static); `Game::removeCreatureCheck` (static); `Monster::setIdle` (static) |
| `Monster::updateIdleStatus` | 1 | `Monster::updateIdleStatus` (static) |
| `Monster::onAddCondition` | 1 | `Monster::onAddCondition` (static) |
| `Monster::onEndCondition` | 1 | `Monster::onEndCondition` (static) |
| `Monster::onThink` | 9 | `Creature::onThink` (static); `Game::addMagicEffect` (static); `Game::internalTeleport` (static); `Game::removeCreature` (static); `tfs::lua::getScriptEnv` (static); … +4 more |
| `Monster::doAttacking` | 1 | `Monster::doAttacking` (static) |
| `Monster::canUseAttack` | 2 | `Game::isSightClear` (static); `Monster::canUseAttack` (static) |
| `Monster::canUseSpell` | 1 | `Monster::canUseSpell` (static) |
| `Monster::onThinkTarget` | 1 | `Monster::onThinkTarget` (static) |
| `Monster::onThinkDefense` | 4 | `Game::addMagicEffect` (static); `Game::placeCreature` (static); `Monster::createMonster` (static); `Monster::onThinkDefense` (static) |
| `Monster::onThinkYell` | 2 | `Game::internalCreatureSay` (static); `Monster::onThinkYell` (static) |
| `Monster::walkToSpawn` | 1 | `Monster::walkToSpawn` (static) |
| `Monster::onWalk` | 2 | `Creature::onWalk` (static); `Monster::onWalk` (static) |
| `Monster::onWalkComplete` | 1 | `Monster::onWalkComplete` (static) |
| `Monster::pushItem` | 3 | `Game::canThrowObjectTo` (static); `Game::internalMoveItem` (static); `Monster::pushItem` (static) |
| `Monster::pushItems` | 4 | `Game::addMagicEffect` (static); `Game::internalRemoveItem` (static); `Monster::pushItem` (static); `Monster::pushItems` (static) |
| `Monster::pushCreature` | 3 | `Game::internalMoveCreature` (static); `Monster::pushCreature` (static); `Spells::getCasterPosition` (static) |
| `Monster::pushCreatures` | 3 | `Game::addMagicEffect` (static); `Monster::pushCreature` (static); `Monster::pushCreatures` (static) |
| `Monster::getNextStep` | 5 | `Creature::getNextStep` (static); `Monster::getNextStep` (static); `Monster::pushCreatures` (static); `Monster::pushItems` (static); `Spells::getCasterPosition` (static) |
| `Monster::getRandomStep` | 1 | `Monster::getRandomStep` (static) |
| `Monster::getDanceStep` | 1 | `Monster::getDanceStep` (static) |
| `Monster::getDistanceStep` | 2 | `Game::isSightClear` (static); `Monster::getDistanceStep` (static) |
| `Monster::canWalkTo` | 1 | `Monster::canWalkTo` (static) |
| `Monster::death` | 1 | `Monster::death` (static) |
| `Monster::getCorpse` | 2 | `Creature::getCorpse` (static); `Monster::getCorpse` (static) |
| `Monster::isInSpawnRange` | 2 | `Monster::isInSpawnRange` (static); `Spawns::isInZone` (static) |
| `Monster::getCombatValues` | 1 | `Monster::getCombatValues` (static) |
| `Monster::updateLookDirection` | 2 | `Game::internalCreatureTurn` (static); `Monster::updateLookDirection` (static) |
| `Monster::dropLoot` | 2 | `tfs::events::monster::onDropLoot` (static); `Monster::dropLoot` (static) |
| `Monster::setNormalCreatureLight` | 1 | `Monster::setNormalCreatureLight` (static) |
| `Monster::drainHealth` | 2 | `Creature::drainHealth` (static); `Monster::drainHealth` (static) |
| `Monster::changeHealth` | 2 | `Creature::changeHealth` (static); `Monster::changeHealth` (static) |
| `Monster::challengeCreature` | 1 | `Monster::challengeCreature` (static) |
| `Monster::getPathSearchParams` | 2 | `Creature::getPathSearchParams` (static); `Monster::getPathSearchParams` (static) |
| `Monster::canPushItems` | 1 | `Monster::canPushItems` (static) |

### `monsters.h` (header declarations)

84 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_MONSTERS_H` | 0 | — |
| `LootBlock` | 0 | — |
| `Loot` | 0 | — |
| `Loot::Loot` | 0 | — |
| `Loot::Loot` | 0 | — |
| `summonBlock_t` | 0 | — |
| `spellBlock_t` | 0 | — |
| `spellBlock_t::spellBlock_t` | 0 | — |
| `spellBlock_t::spellBlock_t` | 0 | — |
| `spellBlock_t::spellBlock_t` | 0 | — |
| `voiceBlock_t` | 0 | — |
| `BestiaryInfo` | 0 | — |
| `MonsterType` | 0 | — |
| `MonsterType::scriptInterface` | 0 | — |
| `MonsterType::voiceVector` | 0 | — |
| `MonsterType::lootItems` | 0 | — |
| `MonsterType::scripts` | 0 | — |
| `MonsterType::attackSpells` | 0 | — |
| `MonsterType::defenseSpells` | 0 | — |
| `MonsterType::summons` | 0 | — |
| `MonsterType::skull` | 0 | — |
| `MonsterType::outfit` | 0 | — |
| `MonsterType::race` | 0 | — |
| `MonsterType::light` | 0 | — |
| `MonsterType::lookcorpse` | 0 | — |
| `MonsterType::experience` | 0 | — |
| `MonsterType::manaCost` | 0 | — |
| `MonsterType::yellChance` | 0 | — |
| `MonsterType::yellSpeedTicks` | 0 | — |
| `MonsterType::staticAttackChance` | 0 | — |
| `MonsterType::maxSummons` | 0 | — |
| `MonsterType::changeTargetSpeed` | 0 | — |
| `MonsterType::conditionImmunities` | 0 | — |
| `MonsterType::damageImmunities` | 0 | — |
| `MonsterType::baseSpeed` | 0 | — |
| `MonsterType::creatureAppearEvent` | 0 | — |
| `MonsterType::creatureDisappearEvent` | 0 | — |
| `MonsterType::creatureMoveEvent` | 0 | — |
| `MonsterType::creatureSayEvent` | 0 | — |
| `MonsterType::thinkEvent` | 0 | — |
| `MonsterType::targetDistance` | 0 | — |
| `MonsterType::runAwayHealth` | 0 | — |
| `MonsterType::health` | 0 | — |
| `MonsterType::healthMax` | 0 | — |
| `MonsterType::changeTargetChance` | 0 | — |
| `MonsterType::defense` | 0 | — |
| `MonsterType::armor` | 0 | — |
| `MonsterType::canPushItems` | 0 | — |
| `MonsterType::canPushCreatures` | 0 | — |
| `MonsterType::pushable` | 0 | — |
| `MonsterType::isAttackable` | 0 | — |
| `MonsterType::isBoss` | 0 | — |
| `MonsterType::isChallengeable` | 0 | — |
| `MonsterType::isConvinceable` | 0 | — |
| `MonsterType::isHostile` | 0 | — |
| `MonsterType::isIgnoringSpawnBlock` | 0 | — |
| `MonsterType::isIllusionable` | 0 | — |
| `MonsterType::isSummonable` | 0 | — |
| `MonsterType::hiddenHealth` | 0 | — |
| `MonsterType::canWalkOnEnergy` | 0 | — |
| `MonsterType::canWalkOnFire` | 0 | — |
| `MonsterType::canWalkOnPoison` | 0 | — |
| `MonsterType::eventType` | 0 | — |
| `MonsterType::MonsterType` | 0 | — |
| `MonsterType::MonsterType` | 0 | — |
| `MonsterType::loadCallback` | 0 | — |
| `MonsterType::loadLoot` | 0 | — |
| `MonsterSpell` | 0 | — |
| `MonsterSpell::MonsterSpell` | 0 | — |
| `MonsterSpell::MonsterSpell` | 0 | — |
| `Monsters` | 0 | — |
| `Monsters::Monsters` | 0 | — |
| `Monsters::Monsters` | 0 | — |
| `Monsters::loadFromXml` | 0 | — |
| `Monsters::reload` | 0 | — |
| `Monsters::getMonsterType` | 0 | — |
| `Monsters::deserializeSpell` | 0 | — |
| `Monsters::getMonsterType` | 0 | — |
| `Monsters::addBestiaryMonsterType` | 0 | — |
| `Monsters::isValidBestiaryInfo` | 0 | — |
| `Monsters::deserializeSpell` | 0 | — |
| `Monsters::loadMonster` | 0 | — |
| `Monsters::loadLootContainer` | 0 | — |
| `Monsters::loadLootItem` | 0 | — |

### `monsters`

14 node(s), 16 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `MonsterType::loadLoot` | 1 | `MonsterType::loadLoot` (static) |
| `Monsters::loadFromXml` | 1 | `Monsters::loadFromXml` (static) |
| `Monsters::reload` | 1 | `Monsters::reload` (static) |
| `Monsters::getDamageCondition` | 2 | `Condition::createCondition` (static); `Monsters::getDamageCondition` (static) |
| `Monsters::deserializeSpell` | 0 | — |
| `Monsters::deserializeSpell` | 4 | `Condition::createCondition` (static); `Monsters::deserializeSpell` (static); `Spells::getSpellByName` (static); `Weapons::getMaxMeleeDamage` (static) |
| `Monsters::loadMonster` | 1 | `Monsters::loadMonster` (static) |
| `MonsterType::loadCallback` | 1 | `MonsterType::loadCallback` (static) |
| `Monsters::loadLootItem` | 1 | `Monsters::loadLootItem` (static) |
| `Monsters::loadLootContainer` | 1 | `Monsters::loadLootContainer` (static) |
| `Monsters::getMonsterType` | 0 | — |
| `Monsters::getMonsterType` | 1 | `Monsters::getMonsterType` (static) |
| `Monsters::addBestiaryMonsterType` | 1 | `Monsters::addBestiaryMonsterType` (static) |
| `Monsters::isValidBestiaryInfo` | 1 | `Monsters::isValidBestiaryInfo` (static) |

### `mounts.h` (header declarations)

15 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Mount` | 0 | — |
| `Mount::Mount` | 0 | — |
| `Mount::name` | 0 | — |
| `Mount::speed` | 0 | — |
| `Mount::clientId` | 0 | — |
| `Mount::id` | 0 | — |
| `Mount::premium` | 0 | — |
| `Mounts` | 0 | — |
| `Mounts::reload` | 0 | — |
| `Mounts::loadFromXml` | 0 | — |
| `Mounts::getMountByID` | 0 | — |
| `Mounts::getMountByName` | 0 | — |
| `Mounts::getMountByClientID` | 0 | — |
| `Mounts::getMounts` | 0 | — |
| `Mounts::mounts` | 0 | — |

### `mounts`

5 node(s), 5 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Mounts::reload` | 1 | `Mounts::reload` (static) |
| `Mounts::loadFromXml` | 1 | `Mounts::loadFromXml` (static) |
| `Mounts::getMountByID` | 1 | `Mounts::getMountByID` (static) |
| `Mounts::getMountByName` | 1 | `Mounts::getMountByName` (static) |
| `Mounts::getMountByClientID` | 1 | `Mounts::getMountByClientID` (static) |

### `movement.h` (header declarations)

13 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_MOVEMENT_H` | 0 | — |
| `MoveEvent_t` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_STEP_IN` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_STEP_OUT` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_EQUIP` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_DEEQUIP` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_ADD_ITEM` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_REMOVE_ITEM` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_ADD_ITEM_ITEMTILE` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_REMOVE_ITEM_ITEMTILE` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_LAST` | 0 | — |
| `MoveEvent_t::MOVE_EVENT_NONE` | 0 | — |
| `MoveEventList` | 0 | — |

### `movement`

34 node(s), 57 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `MoveEvents::clearMap` | 1 | `MoveEvents::clearMap` (static) |
| `MoveEvents::clearPosMap` | 1 | `MoveEvents::clearPosMap` (static) |
| `MoveEvents::clear` | 1 | `MoveEvents::clear` (static) |
| `MoveEvents::getScriptInterface` | 1 | `MoveEvents::getScriptInterface` (static) |
| `MoveEvents::getEvent` | 0 | — |
| `MoveEvents::registerEvent` | 1 | `MoveEvents::registerEvent` (static) |
| `MoveEvents::registerLuaFunction` | 1 | `MoveEvents::registerLuaFunction` (static) |
| `MoveEvents::registerLuaEvent` | 1 | `MoveEvents::registerLuaEvent` (static) |
| `MoveEvents::addEvent` | 0 | — |
| `MoveEvents::getEvent` | 0 | — |
| `MoveEvents::getEvent` | 0 | — |
| `MoveEvents::addEvent` | 1 | `MoveEvents::addEvent` (static) |
| `MoveEvents::getEvent` | 1 | `MoveEvents::getEvent` (static) |
| `MoveEvents::onCreatureMove` | 1 | `MoveEvents::onCreatureMove` (static) |
| `MoveEvents::onPlayerEquip` | 1 | `MoveEvents::onPlayerEquip` (static) |
| `MoveEvents::onPlayerDeEquip` | 1 | `MoveEvents::onPlayerDeEquip` (static) |
| `MoveEvents::onItemMove` | 1 | `MoveEvents::onItemMove` (static) |
| `MoveEvent::getScriptEventName` | 1 | `MoveEvent::getScriptEventName` (static) |
| `MoveEvent::configureEvent` | 2 | `MoveEvent::configureEvent` (static); `Vocations::getVocationId` (static) |
| `MoveEvent::StepInField` | 1 | `MoveEvent::StepInField` (static) |
| `MoveEvent::StepOutField` | 1 | `MoveEvent::StepOutField` (static) |
| `MoveEvent::AddItemField` | 1 | `MoveEvent::AddItemField` (static) |
| `MoveEvent::RemoveItemField` | 1 | `MoveEvent::RemoveItemField` (static) |
| `MoveEvent::EquipItem` | 5 | `Condition::createCondition` (static); `Game::changeSpeed` (static); `Game::startDecay` (static); `Game::transformItem` (static); `MoveEvent::EquipItem` (static) |
| `MoveEvent::DeEquipItem` | 4 | `Game::changeSpeed` (static); `Game::startDecay` (static); `Game::transformItem` (static); `MoveEvent::DeEquipItem` (static) |
| `MoveEvent::loadFunction` | 1 | `MoveEvent::loadFunction` (static) |
| `MoveEvent::getEventType` | 1 | `MoveEvent::getEventType` (static) |
| `MoveEvent::setEventType` | 1 | `MoveEvent::setEventType` (static) |
| `MoveEvent::fireStepEvent` | 1 | `MoveEvent::fireStepEvent` (static) |
| `MoveEvent::executeStep` | 8 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushPosition` (static); `tfs::lua::pushThing` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); … +3 more |
| `MoveEvent::fireEquip` | 1 | `MoveEvent::fireEquip` (static) |
| `MoveEvent::executeEquip` | 8 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::pushThing` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); … +3 more |
| `MoveEvent::fireAddRemItem` | 1 | `MoveEvent::fireAddRemItem` (static) |
| `MoveEvent::executeAddRemItem` | 6 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushPosition` (static); `tfs::lua::pushThing` (static); `tfs::lua::reserveScriptEnv` (static); `MoveEvent::executeAddRemItem` (static); … +1 more |

### `networkmessage.h` (header declarations)

37 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `NetworkMessage_ptr` | 0 | — |
| `NetworkMessage` | 0 | — |
| `NetworkMessage::MsgSize_t` | 0 | — |
| `NetworkMessage::INITIAL_BUFFER_POSITION` | 0 | — |
| `NetworkMessage::HEADER_LENGTH` | 0 | — |
| `NetworkMessage::CHECKSUM_LENGTH` | 0 | — |
| `NetworkMessage::XTEA_MULTIPLE` | 0 | — |
| `NetworkMessage::MAX_BODY_LENGTH` | 0 | — |
| `NetworkMessage::MAX_PROTOCOL_BODY_LENGTH` | 0 | — |
| `NetworkMessage::NetworkMessage` | 0 | — |
| `NetworkMessage::reset` | 0 | — |
| `NetworkMessage::getByte` | 0 | — |
| `NetworkMessage::getPreviousByte` | 0 | — |
| `NetworkMessage::get` | 0 | — |
| `NetworkMessage::skipBytes` | 0 | — |
| `NetworkMessage::addByte` | 0 | — |
| `NetworkMessage::add` | 0 | — |
| `NetworkMessage::getLength` | 0 | — |
| `NetworkMessage::isEmpty` | 0 | — |
| `NetworkMessage::setLength` | 0 | — |
| `NetworkMessage::getBufferPosition` | 0 | — |
| `NetworkMessage::getRemainingBufferLength` | 0 | — |
| `NetworkMessage::setBufferPosition` | 0 | — |
| `NetworkMessage::getLengthHeader` | 0 | — |
| `NetworkMessage::isOverrun` | 0 | — |
| `NetworkMessage::getBuffer` | 0 | — |
| `NetworkMessage::getBuffer() const` | 0 | — |
| `NetworkMessage::getRemainingBuffer` | 0 | — |
| `NetworkMessage::getBodyBuffer` | 0 | — |
| `NetworkMessage::NetworkMessageInfo` | 0 | — |
| `NetworkMessage::NetworkMessageInfo::length` | 0 | — |
| `NetworkMessage::NetworkMessageInfo::position` | 0 | — |
| `NetworkMessage::NetworkMessageInfo::overrun` | 0 | — |
| `NetworkMessage::info` | 0 | — |
| `NetworkMessage::buffer` | 0 | — |
| `NetworkMessage::canAdd` | 0 | — |
| `NetworkMessage::canRead` | 0 | — |

### `networkmessage`

10 node(s), 8 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `NetworkMessage::getString` | 1 | `NetworkMessage::getString` (static) |
| `NetworkMessage::getPosition` | 1 | `NetworkMessage::getPosition` (static) |
| `NetworkMessage::addString` | 1 | `NetworkMessage::addString` (static) |
| `NetworkMessage::addDouble` | 1 | `NetworkMessage::addDouble` (static) |
| `NetworkMessage::addBytes` | 1 | `NetworkMessage::addBytes` (static) |
| `NetworkMessage::addPaddingBytes` | 1 | `NetworkMessage::addPaddingBytes` (static) |
| `NetworkMessage::addPosition` | 1 | `NetworkMessage::addPosition` (static) |
| `NetworkMessage::addItem(uint16_t,uint8_t)` | 0 | — |
| `NetworkMessage::addItem(const Item*)` | 0 | — |
| `NetworkMessage::addItemId` | 1 | `NetworkMessage::addItemId` (static) |

### `npc.h` (header declarations)

13 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_NPC_H` | 0 | — |
| `Npcs` | 0 | — |
| `Npcs::reload` | 0 | — |
| `NpcEventsHandler` | 0 | — |
| `NpcEventsHandler::NpcEventsHandler` | 0 | — |
| `NpcEventsHandler::onCreatureAppear` | 0 | — |
| `NpcEventsHandler::onCreatureDisappear` | 0 | — |
| `NpcEventsHandler::onCreatureMove` | 0 | — |
| `NpcEventsHandler::onCreatureSay` | 0 | — |
| `NpcEventsHandler::onPlayerCloseChannel` | 0 | — |
| `NpcEventsHandler::onPlayerEndTrade` | 0 | — |
| `NpcEventsHandler::onThink` | 0 | — |
| `NpcEventsHandler::isLoaded` | 0 | — |

### `npc`

60 node(s), 161 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Npcs::reload` | 1 | `Npcs::reload` (static) |
| `Npc::createNpc` | 1 | `Npc::createNpc` (static) |
| `Npc::addList` | 2 | `Game::addNpc` (static); `Npc::addList` (static) |
| `Npc::removeList` | 2 | `Game::removeNpc` (static); `Npc::removeList` (static) |
| `Npc::load` | 1 | `Npc::load` (static) |
| `Npc::reset` | 1 | `Npc::reset` (static) |
| `Npc::reload` | 1 | `Npc::reload` (static) |
| `Npc::loadFromXml` | 1 | `Npc::loadFromXml` (static) |
| `Npc::canSee` | 2 | `Creature::canSee` (static); `Npc::canSee` (static) |
| `Npc::getDescription` | 1 | `Npc::getDescription` (static) |
| `Npc::goToFollowCreature` | 1 | `Npc::goToFollowCreature` (static) |
| `Npc::onCreatureAppear` | 1 | `Npc::onCreatureAppear` (static) |
| `Npc::onRemoveCreature` | 2 | `Creature::onRemoveCreature` (static); `Npc::onRemoveCreature` (static) |
| `Npc::onCreatureMove` | 2 | `Creature::onCreatureMove` (static); `Npc::onCreatureMove` (static) |
| `Npc::onCreatureSay` | 1 | `Npc::onCreatureSay` (static) |
| `Npc::onPlayerCloseChannel` | 1 | `Npc::onPlayerCloseChannel` (static) |
| `Npc::onThink` | 2 | `Creature::onThink` (static); `Npc::onThink` (static) |
| `Npc::doSay` | 2 | `Game::internalCreatureSay` (static); `Npc::doSay` (static) |
| `Npc::doSayToPlayer` | 1 | `Npc::doSayToPlayer` (static) |
| `Npc::onPlayerTrade` | 1 | `Npc::onPlayerTrade` (static) |
| `Npc::onPlayerEndTrade` | 1 | `Npc::onPlayerEndTrade` (static) |
| `Npc::getNextStep` | 2 | `Creature::getNextStep` (static); `Npc::getNextStep` (static) |
| `Npc::setIdle` | 1 | `Npc::setIdle` (static) |
| `Npc::canWalkTo` | 2 | `Npc::canWalkTo` (static); `Spawns::isInZone` (static) |
| `Npc::getRandomStep` | 1 | `Npc::getRandomStep` (static) |
| `Npc::doMoveTo` | 1 | `Npc::doMoveTo` (static) |
| `Npc::turnToCreature` | 2 | `Game::internalCreatureTurn` (static); `Npc::turnToCreature` (static) |
| `Npc::setCreatureFocus` | 1 | `Npc::setCreatureFocus` (static) |
| `Npc::addShopPlayer` | 1 | `Npc::addShopPlayer` (static) |
| `Npc::removeShopPlayer` | 1 | `Npc::removeShopPlayer` (static) |
| `Npc::closeAllShopWindows` | 1 | `Npc::closeAllShopWindows` (static) |
| `NpcScriptInterface::initState` | 1 | `NpcScriptInterface::initState` (static) |
| `NpcScriptInterface::closeState` | 2 | `LuaScriptInterface::closeState` (static); `NpcScriptInterface::closeState` (static) |
| `NpcScriptInterface::loadNpcLib` | 1 | `NpcScriptInterface::loadNpcLib` (static) |
| `NpcScriptInterface::registerFunctions` | 2 | `tfs::lua::registerMethod` (static); `NpcScriptInterface::registerFunctions` (static) |
| `NpcScriptInterface::luaActionSay` | 4 | `tfs::lua::getPlayer` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::getString` (static); `NpcScriptInterface::luaActionSay` (static) |
| `NpcScriptInterface::luaActionMove` | 3 | `Game::internalMoveCreature` (static); `tfs::lua::getScriptEnv` (static); `NpcScriptInterface::luaActionMove` (static) |
| `NpcScriptInterface::luaActionMoveTo` | 5 | `tfs::lua::getBoolean` (static); `tfs::lua::getPosition` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `NpcScriptInterface::luaActionMoveTo` (static) |
| `NpcScriptInterface::luaActionTurn` | 3 | `Game::internalCreatureTurn` (static); `tfs::lua::getScriptEnv` (static); `NpcScriptInterface::luaActionTurn` (static) |
| `NpcScriptInterface::luaActionFollow` | 4 | `tfs::lua::getPlayer` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `NpcScriptInterface::luaActionFollow` (static) |
| `NpcScriptInterface::luagetDistanceTo` | 3 | `tfs::lua::getErrorDesc` (static); `tfs::lua::getScriptEnv` (static); `NpcScriptInterface::luagetDistanceTo` (static) |
| `NpcScriptInterface::luaSetNpcFocus` | 3 | `tfs::lua::getCreature` (static); `tfs::lua::getScriptEnv` (static); `NpcScriptInterface::luaSetNpcFocus` (static) |
| `NpcScriptInterface::luaGetNpcCid` | 2 | `tfs::lua::getScriptEnv` (static); `NpcScriptInterface::luaGetNpcCid` (static) |
| `NpcScriptInterface::luaGetNpcParameter` | 4 | `tfs::lua::getScriptEnv` (static); `tfs::lua::getString` (static); `tfs::lua::pushString` (static); `NpcScriptInterface::luaGetNpcParameter` (static) |
| `NpcScriptInterface::luaOpenShopWindow` | 8 | `tfs::lua::getErrorDesc` (static); `tfs::lua::getFieldString` (static); `tfs::lua::getPlayer` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::popCallback` (static); … +3 more |
| `NpcScriptInterface::luaCloseShopWindow` | 5 | `tfs::lua::getErrorDesc` (static); `tfs::lua::getPlayer` (static); `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `NpcScriptInterface::luaCloseShopWindow` (static) |
| `NpcScriptInterface::luaDoSellItem` | 7 | `Game::internalPlayerAddItem` (static); `Item::CreateItem` (static); `tfs::lua::getBoolean` (static); `tfs::lua::getErrorDesc` (static); `tfs::lua::getPlayer` (static); … +2 more |
| `NpcScriptInterface::luaNpcGetParameter` | 4 | `tfs::lua::getString` (static); `tfs::lua::pushString` (static); `tfs::lua::getUserdata` (static); `NpcScriptInterface::luaNpcGetParameter` (static) |
| `NpcScriptInterface::luaNpcSetFocus` | 4 | `tfs::lua::getCreature` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::getUserdata` (static); `NpcScriptInterface::luaNpcSetFocus` (static) |
| `NpcScriptInterface::luaNpcOpenShopWindow` | 7 | `tfs::lua::getErrorDesc` (static); `tfs::lua::getFieldString` (static); `tfs::lua::getPlayer` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::getField` (static); … +2 more |
| `NpcScriptInterface::luaNpcCloseShopWindow` | 5 | `tfs::lua::getErrorDesc` (static); `tfs::lua::getPlayer` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::getUserdata` (static); `NpcScriptInterface::luaNpcCloseShopWindow` (static) |
| `NpcEventsHandler::isLoaded` | 1 | `NpcEventsHandler::isLoaded` (static) |
| `NpcEventsHandler::onCreatureAppear` | 5 | `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); `NpcEventsHandler::onCreatureAppear` (static) |
| `NpcEventsHandler::onCreatureDisappear` | 5 | `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); `NpcEventsHandler::onCreatureDisappear` (static) |
| `NpcEventsHandler::onCreatureMove` | 6 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushPosition` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `NpcEventsHandler::onCreatureSay` | 6 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushString` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `NpcEventsHandler::onPlayerTrade` | 7 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::pushCallback` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); … +2 more |
| `NpcEventsHandler::onPlayerCloseChannel` | 5 | `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); `NpcEventsHandler::onPlayerCloseChannel` (static) |
| `NpcEventsHandler::onPlayerEndTrade` | 5 | `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); `NpcEventsHandler::onPlayerEndTrade` (static) |
| `NpcEventsHandler::onThink` | 3 | `tfs::lua::getScriptEnv` (static); `tfs::lua::reserveScriptEnv` (static); `NpcEventsHandler::onThink` (static) |

### `otpch.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_OTPCH_H` | 0 | — |

### `otserv.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_OTSERV_H` | 0 | — |

### `otserv`

5 node(s), 51 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `startupErrorMessage` | 0 | — |
| `printServerVersion` | 0 | — |
| `badAllocationHandler` | 0 | — |
| `startServer` | 7 | `badAllocationHandler` (dynamic/curated); `mainLoader` (dynamic/curated); `DatabaseTasks::shutdown` (static); `Scheduler::shutdown` (static); `Dispatcher::addTask` (static); … +2 more |
| `mainLoader` | 44 | `Game::setGameState` (static/curated); `printServerVersion` (static/curated); `ConfigManager::load` (static/curated); `startupErrorMessage` (static/curated); `loadPEM` (static/curated); … +39 more |

### `outfit.h` (header declarations)

18 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Outfit` | 0 | — |
| `Outfit::name` | 0 | — |
| `Outfit::lookType` | 0 | — |
| `Outfit::premium` | 0 | — |
| `Outfit::unlocked` | 0 | — |
| `operator==(const Outfit&, const Outfit&)` | 0 | — |
| `ProtocolOutfit` | 0 | — |
| `ProtocolOutfit::ProtocolOutfit` | 0 | — |
| `ProtocolOutfit::name` | 0 | — |
| `ProtocolOutfit::lookType` | 0 | — |
| `ProtocolOutfit::addons` | 0 | — |
| `Outfits` | 0 | — |
| `Outfits::getInstance` | 0 | — |
| `Outfits::loadFromXml` | 0 | — |
| `Outfits::getOutfitByLookType(PlayerSex_t, uint16_t)` | 0 | — |
| `Outfits::getOutfitByLookType(uint16_t)` | 0 | — |
| `Outfits::getOutfits` | 0 | — |
| `Outfits::outfits` | 0 | — |

### `outfit`

3 node(s), 1 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Outfits::loadFromXml` | 1 | `Outfits::loadFromXml` (static) |
| `Outfits::getOutfitByLookType(PlayerSex_t, uint16_t)` | 0 | — |
| `Outfits::getOutfitByLookType(uint16_t)` | 0 | — |

### `outputmessage.h` (header declarations)

15 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `OutputMessage` | 0 | — |
| `OutputMessage::OutputMessage` | 0 | — |
| `OutputMessage::OutputMessage(const OutputMessage&)` | 0 | — |
| `OutputMessage::operator=` | 0 | — |
| `OutputMessage::getOutputBuffer` | 0 | — |
| `OutputMessage::writeMessageLength` | 0 | — |
| `OutputMessage::addCryptoHeader` | 0 | — |
| `OutputMessage::append(const NetworkMessage&)` | 0 | — |
| `OutputMessage::append(const OutputMessage_ptr&)` | 0 | — |
| `OutputMessage::setSequenceId` | 0 | — |
| `OutputMessage::getSequenceId` | 0 | — |
| `OutputMessage::add_header` | 0 | — |
| `OutputMessage::outputBufferStart` | 0 | — |
| `OutputMessage::sequenceId` | 0 | — |
| `tfs::net` | 0 | — |

### `outputmessage`

9 node(s), 4 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_scheduler` | 0 | — |
| `(anonymous)::OUTPUTMESSAGE_FREE_LIST_CAPACITY` | 0 | — |
| `(anonymous)::OUTPUTMESSAGE_AUTOSEND_DELAY` | 0 | — |
| `(anonymous)::bufferedProtocols` | 0 | — |
| `(anonymous)::scheduleSendAll` | 1 | `Scheduler::addEvent` (static) |
| `(anonymous)::sendAll` | 0 | — |
| `tfs::net::make_output_message` | 1 | `tfs::net::make_output_message` (static) |
| `tfs::net::insert_protocol_to_autosend` | 1 | `tfs::net::insert_protocol_to_autosend` (static) |
| `tfs::net::remove_protocol_from_autosend` | 1 | `tfs::net::remove_protocol_from_autosend` (static) |

### `party.h` (header declarations)

52 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Party` | 0 | — |
| `EXPERIENCE_SHARE_FLOORS` | 0 | — |
| `EXPERIENCE_SHARE_RANGE` | 0 | — |
| `SharedExpStatus_t` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_EMPTYPARTY` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_LEVELDIFFTOOLARGE` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_MEMBERINACTIVE` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_OK` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_TOOFARAWAY` | 0 | — |
| `Party::inviteList` | 0 | — |
| `Party::leader` | 0 | — |
| `Party::memberList` | 0 | — |
| `Party::sharedExpActive` | 0 | — |
| `Party::sharedExpEnabled` | 0 | — |
| `Party::ticksMap` | 0 | — |
| `Party::empty` | 0 | — |
| `Party::getInvitationCount` | 0 | — |
| `Party::getInvitees` | 0 | — |
| `Party::getLeader` | 0 | — |
| `Party::getMemberCount` | 0 | — |
| `Party::getMembers` | 0 | — |
| `Party::isSharedExperienceActive` | 0 | — |
| `Party::isSharedExperienceEnabled` | 0 | — |
| `PlayerVector` | 0 | — |
| `FS_PARTY_H` | 0 | — |
| `SharedExpStatus_t` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_OK` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_TOOFARAWAY` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_LEVELDIFFTOOLARGE` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_MEMBERINACTIVE` | 0 | — |
| `SharedExpStatus_t::SHAREDEXP_EMPTYPARTY` | 0 | — |
| `Party` | 0 | — |
| `Party::Party` | 0 | — |
| `Party::disband` | 0 | — |
| `Party::invitePlayer` | 0 | — |
| `Party::joinParty` | 0 | — |
| `Party::revokeInvitation` | 0 | — |
| `Party::passPartyLeadership` | 0 | — |
| `Party::leaveParty` | 0 | — |
| `Party::removeInvite` | 0 | — |
| `Party::isPlayerInvited` | 0 | — |
| `Party::updateAllPartyIcons` | 0 | — |
| `Party::broadcastPartyMessage` | 0 | — |
| `Party::canOpenCorpse` | 0 | — |
| `Party::shareExperience` | 0 | — |
| `Party::setSharedExperience` | 0 | — |
| `Party::canUseSharedExperience` | 0 | — |
| `Party::getMemberSharedExperienceStatus` | 0 | — |
| `Party::updateSharedExperience` | 0 | — |
| `Party::updatePlayerTicks` | 0 | — |
| `Party::clearPlayerPoints` | 0 | — |
| `Party::getSharedExperienceStatus` | 0 | — |

### `party`

41 node(s), 31 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Party::Party` | 1 | `Party::Party` (static) |
| `anon::getSharedExpReturnMessage` | 0 | — |
| `Party::broadcastPartyMessage` | 0 | — |
| `Party::canOpenCorpse` | 0 | — |
| `Party::canUseSharedExperience` | 0 | — |
| `Party::clearPlayerPoints` | 0 | — |
| `Party::disband` | 0 | — |
| `Party::getMemberSharedExperienceStatus` | 0 | — |
| `Party::getSharedExperienceStatus` | 0 | — |
| `Party::invitePlayer` | 0 | — |
| `Party::isPlayerInvited` | 0 | — |
| `Party::joinParty` | 0 | — |
| `Party::leaveParty` | 0 | — |
| `Party::passPartyLeadership` | 0 | — |
| `Party::removeInvite` | 0 | — |
| `Party::revokeInvitation` | 0 | — |
| `Party::setSharedExperience` | 0 | — |
| `Party::shareExperience` | 0 | — |
| `Party::updateAllPartyIcons` | 0 | — |
| `Party::updatePlayerTicks` | 0 | — |
| `Party::updateSharedExperience` | 0 | — |
| `Party::disband` | 3 | `tfs::events::party::onDisband` (static); `Game::updatePlayerShield` (static); `Party::disband` (static) |
| `Party::leaveParty` | 3 | `tfs::events::party::onLeave` (static); `Game::updatePlayerShield` (static); `Party::leaveParty` (static) |
| `Party::passPartyLeadership` | 2 | `tfs::events::party::onPassLeadership` (static); `Party::passPartyLeadership` (static) |
| `Party::joinParty` | 3 | `tfs::events::party::onJoin` (static); `Game::updatePlayerShield` (static); `Party::joinParty` (static) |
| `Party::removeInvite` | 1 | `Party::removeInvite` (static) |
| `Party::revokeInvitation` | 2 | `tfs::events::party::onRevokeInvitation` (static); `Party::revokeInvitation` (static) |
| `Party::invitePlayer` | 2 | `Game::updatePlayerShield` (static); `Party::invitePlayer` (static) |
| `Party::isPlayerInvited` | 1 | `Party::isPlayerInvited` (static) |
| `Party::updateAllPartyIcons` | 1 | `Party::updateAllPartyIcons` (static) |
| `Party::broadcastPartyMessage` | 1 | `Party::broadcastPartyMessage` (static) |
| `Party::updateSharedExperience` | 1 | `Party::updateSharedExperience` (static) |
| `getSharedExpReturnMessage` | 0 | — |
| `Party::setSharedExperience` | 1 | `Party::setSharedExperience` (static) |
| `Party::shareExperience` | 2 | `tfs::events::party::onShareExperience` (static); `Party::shareExperience` (static) |
| `Party::canUseSharedExperience` | 1 | `Party::canUseSharedExperience` (static) |
| `Party::getMemberSharedExperienceStatus` | 1 | `Party::getMemberSharedExperienceStatus` (static) |
| `Party::getSharedExperienceStatus` | 1 | `Party::getSharedExperienceStatus` (static) |
| `Party::updatePlayerTicks` | 1 | `Party::updatePlayerTicks` (static) |
| `Party::clearPlayerPoints` | 1 | `Party::clearPlayerPoints` (static) |
| `Party::canOpenCorpse` | 2 | `Game::getPlayerByID` (static); `Party::canOpenCorpse` (static) |

### `player.h` (header declarations)

39 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `skillsid_t` | 0 | — |
| `skillsid_t::SKILLVALUE_LEVEL` | 0 | — |
| `skillsid_t::SKILLVALUE_TRIES` | 0 | — |
| `skillsid_t::SKILLVALUE_PERCENT` | 0 | — |
| `fightMode_t` | 0 | — |
| `fightMode_t::FIGHTMODE_ATTACK` | 0 | — |
| `fightMode_t::FIGHTMODE_BALANCED` | 0 | — |
| `fightMode_t::FIGHTMODE_DEFENSE` | 0 | — |
| `pvpMode_t` | 0 | — |
| `pvpMode_t::PVP_MODE_DOVE` | 0 | — |
| `pvpMode_t::PVP_MODE_WHITE_HAND` | 0 | — |
| `pvpMode_t::PVP_MODE_YELLOW_HAND` | 0 | — |
| `pvpMode_t::PVP_MODE_RED_FIST` | 0 | — |
| `tradestate_t` | 0 | — |
| `tradestate_t::TRADE_NONE` | 0 | — |
| `tradestate_t::TRADE_INITIATED` | 0 | — |
| `tradestate_t::TRADE_ACCEPT` | 0 | — |
| `tradestate_t::TRADE_ACKNOWLEDGE` | 0 | — |
| `tradestate_t::TRADE_TRANSFER` | 0 | — |
| `VIPEntry` | 0 | — |
| `VIPEntry::VIPEntry` | 0 | — |
| `VIPEntry::guid` | 0 | — |
| `VIPEntry::name` | 0 | — |
| `VIPEntry::description` | 0 | — |
| `VIPEntry::icon` | 0 | — |
| `VIPEntry::notify` | 0 | — |
| `OpenContainer` | 0 | — |
| `OpenContainer::container` | 0 | — |
| `OpenContainer::index` | 0 | — |
| `MINIMUM_SKILL_LEVEL` | 0 | — |
| `Skill` | 0 | — |
| `Skill::tries` | 0 | — |
| `Skill::level` | 0 | — |
| `Skill::percent` | 0 | — |
| `MuteCountMap` | 0 | — |
| `PLAYER_MAX_SPEED` | 0 | — |
| `PLAYER_MIN_SPEED` | 0 | — |
| `NOTIFY_DEPOT_BOX_RANGE` | 0 | — |
| `Player` | 0 | — |

### `player`

214 node(s), 331 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Player::Player` | 1 | `Player::Player` (static) |
| `Player::~Player` | 0 | — |
| `Player::setID` | 1 | `Player::setID` (static) |
| `Player::setVocation` | 3 | `Game::changeSpeed` (static); `Player::setVocation` (static); `Vocations::getVocation` (static) |
| `Player::isPushable` | 2 | `Creature::isPushable` (static); `Player::isPushable` (static) |
| `Player::getDescription` | 1 | `Player::getDescription` (static) |
| `Player::getInventoryItem` | 1 | `Player::getInventoryItem` (static) |
| `Player::addConditionSuppressions` | 0 | — |
| `Player::removeConditionSuppressions` | 0 | — |
| `Player::getWeapon` | 0 | — |
| `Player::getWeapon` | 1 | `Player::getWeapon` (static) |
| `Player::getWeaponType` | 1 | `Player::getWeaponType` (static) |
| `Player::getWeaponSkill` | 1 | `Player::getWeaponSkill` (static) |
| `Player::getArmor` | 1 | `Player::getArmor` (static) |
| `Player::getShieldAndWeapon` | 1 | `Player::getShieldAndWeapon` (static) |
| `Player::getDefense` | 1 | `Player::getDefense` (static) |
| `Player::getAttackSpeed` | 1 | `Player::getAttackSpeed` (static) |
| `Player::getAttackFactor` | 1 | `Player::getAttackFactor` (static) |
| `Player::getDefenseFactor` | 1 | `Player::getDefenseFactor` (static) |
| `Player::getClientIcons` | 1 | `Player::getClientIcons` (static) |
| `Player::updateInventoryWeight` | 1 | `Player::updateInventoryWeight` (static) |
| `Player::addSkillAdvance` | 4 | `CreatureEvents::playerAdvance` (static); `tfs::events::player::onGainSkillTries` (static); `Player::addSkillAdvance` (static); `Player::getBasisPointLevel` (static) |
| `Player::removeSkillTries` | 2 | `Player::getBasisPointLevel` (static); `Player::removeSkillTries` (static) |
| `Player::setVarStats` | 3 | `Creature::changeHealth` (static); `Game::addCreatureHealth` (static); `Player::setVarStats` (static) |
| `Player::getDefaultStats` | 1 | `Player::getDefaultStats` (static) |
| `Player::addContainer` | 1 | `Player::addContainer` (static) |
| `Player::closeContainer` | 1 | `Player::closeContainer` (static) |
| `Player::setContainerIndex` | 1 | `Player::setContainerIndex` (static) |
| `Player::getContainerByID` | 1 | `Player::getContainerByID` (static) |
| `Player::getContainerID` | 1 | `Player::getContainerID` (static) |
| `Player::getContainerIndex` | 1 | `Player::getContainerIndex` (static) |
| `Player::canOpenCorpse` | 1 | `Player::canOpenCorpse` (static) |
| `Player::getLookCorpse` | 1 | `Player::getLookCorpse` (static) |
| `Player::setStorageValue` | 2 | `Creature::setStorageValue` (static); `Player::setStorageValue` (static) |
| `Player::canSee` | 1 | `Player::canSee` (static) |
| `Player::canSeeCreature` | 1 | `Player::canSeeCreature` (static) |
| `Player::canSeeGhostMode` | 0 | — |
| `Player::canWalkthrough` | 1 | `Player::canWalkthrough` (static) |
| `Player::canWalkthroughEx` | 1 | `Player::canWalkthroughEx` (static) |
| `Player::onReceiveMail` | 1 | `Player::onReceiveMail` (static) |
| `Player::isNearDepotBox` | 1 | `Player::isNearDepotBox` (static) |
| `Player::getDepotChest` | 1 | `Player::getDepotChest` (static) |
| `Player::getDepotLocker` | 2 | `Item::CreateItem` (static); `Player::getDepotLocker` (static) |
| `Player::sendCancelMessage` | 0 | — |
| `Player::sendStats` | 1 | `Player::sendStats` (static) |
| `Player::sendPing` | 3 | `CreatureEvents::playerLogout` (static); `Game::removeCreature` (static); `Player::sendPing` (static) |
| `Player::getWriteItem` | 1 | `Player::getWriteItem` (static) |
| `Player::setWriteItem` | 1 | `Player::setWriteItem` (static) |
| `Player::getEditHouse` | 1 | `Player::getEditHouse` (static) |
| `Player::setEditHouse` | 1 | `Player::setEditHouse` (static) |
| `Player::sendHouseWindow` | 1 | `Player::sendHouseWindow` (static) |
| `Player::sendAddContainerItem` | 1 | `Player::sendAddContainerItem` (static) |
| `Player::sendUpdateContainerItem` | 1 | `Player::sendUpdateContainerItem` (static) |
| `Player::sendRemoveContainerItem` | 1 | `Player::sendRemoveContainerItem` (static) |
| `Player::openSavedContainers` | 1 | `Player::openSavedContainers` (static) |
| `Player::onUpdateTileItem` | 3 | `Creature::onUpdateTileItem` (static); `Game::internalCloseTrade` (static); `Player::onUpdateTileItem` (static) |
| `Player::onRemoveTileItem` | 3 | `Creature::onRemoveTileItem` (static); `Game::internalCloseTrade` (static); `Player::onRemoveTileItem` (static) |
| `Player::onCreatureAppear` | 9 | `CreatureEvents::playerLogin` (static); `tfs::events::player::onInventoryUpdate` (static); `Game::changeSpeed` (static); `Game::checkPlayersRecord` (static); `Game::getBedBySleeper` (static); … +4 more |
| `Player::onAttackedCreatureDisappear` | 1 | `Player::onAttackedCreatureDisappear` (static) |
| `Player::onFollowCreatureDisappear` | 1 | `Player::onFollowCreatureDisappear` (static) |
| `Player::onChangeZone` | 3 | `Game::internalCreatureChangeOutfit` (static); `Game::updateCreatureWalkthrough` (static); `Player::onChangeZone` (static) |
| `Player::onAttackedCreatureChangeZone` | 1 | `Player::onAttackedCreatureChangeZone` (static) |
| `Player::onRemoveCreature` | 8 | `Chat::removeUserFromAllChannels` (static); `Creature::onRemoveCreature` (static); `tfs::events::player::onInventoryUpdate` (static); `Game::internalCloseTrade` (static); `IOLoginData::savePlayer` (static); … +3 more |
| `Player::openShopWindow` | 1 | `Player::openShopWindow` (static) |
| `Player::closeShopWindow` | 1 | `Player::closeShopWindow` (static) |
| `Player::onWalk` | 2 | `Creature::onWalk` (static); `Player::onWalk` (static) |
| `Player::onCreatureMove` | 5 | `Condition::createCondition` (static); `Creature::onCreatureMove` (static); `Game::internalCloseTrade` (static); `Game::updateCreatureWalk` (static); `Player::onCreatureMove` (static) |
| `Player::onAddContainerItem` | 0 | — |
| `Player::onUpdateContainerItem` | 1 | `Player::onUpdateContainerItem` (static) |
| `Player::onRemoveContainerItem` | 2 | `Game::internalCloseTrade` (static); `Player::onRemoveContainerItem` (static) |
| `Player::onCloseContainer` | 1 | `Player::onCloseContainer` (static) |
| `Player::onSendContainer` | 1 | `Player::onSendContainer` (static) |
| `Player::onUpdateInventoryItem` | 1 | `Player::onUpdateInventoryItem` (static) |
| `Player::onRemoveInventoryItem` | 2 | `Game::internalCloseTrade` (static); `Player::onRemoveInventoryItem` (static) |
| `Player::checkTradeState` | 2 | `Game::internalCloseTrade` (static); `Player::checkTradeState` (static) |
| `Player::setNextWalkActionTask` | 1 | `Player::setNextWalkActionTask` (static) |
| `Player::setNextActionTask` | 1 | `Player::setNextActionTask` (static) |
| `Player::getNextActionTime` | 0 | — |
| `Player::onThink` | 2 | `Creature::onThink` (static); `Player::onThink` (static) |
| `Player::isMuted` | 1 | `Player::isMuted` (static) |
| `Player::addMessageBuffer` | 1 | `Player::addMessageBuffer` (static) |
| `Player::removeMessageBuffer` | 2 | `Condition::createCondition` (static); `Player::removeMessageBuffer` (static) |
| `Player::drainHealth` | 2 | `Creature::drainHealth` (static); `Player::drainHealth` (static) |
| `Player::drainMana` | 1 | `Player::drainMana` (static) |
| `Player::addManaSpent` | 4 | `CreatureEvents::playerAdvance` (static); `tfs::events::player::onGainSkillTries` (static); `Player::addManaSpent` (static); `Player::getBasisPointLevel` (static) |
| `Player::removeManaSpent` | 2 | `Player::getBasisPointLevel` (static); `Player::removeManaSpent` (static) |
| `Player::addExperience` | 8 | `ConfigManager::getExperienceStage` (static); `CreatureEvents::playerAdvance` (static); `tfs::events::player::onGainExperience` (static); `Game::addCreatureHealth` (static); `Game::changeSpeed` (static); … +3 more |
| `Player::removeExperience` | 6 | `tfs::events::player::onLoseExperience` (static); `Game::addCreatureHealth` (static); `Game::changeSpeed` (static); `Game::updateCreatureWalkthrough` (static); `Player::getBasisPointLevel` (static); … +1 more |
| `Player::getBasisPointLevel` | 1 | `Player::getBasisPointLevel` (static) |
| `Player::onBlockHit` | 1 | `Player::onBlockHit` (static) |
| `Player::onAttackedCreatureBlockHit` | 1 | `Player::onAttackedCreatureBlockHit` (static) |
| `Player::hasShield` | 1 | `Player::hasShield` (static) |
| `Player::blockHit` | 4 | `Creature::blockHit` (static); `Game::combatChangeHealth` (static); `Game::transformItem` (static); `Player::blockHit` (static) |
| `Player::getIP` | 1 | `Player::getIP` (static) |
| `Player::death` | 7 | `tfs::events::player::onLoseExperience` (static); `Game::addCreatureHealth` (static); `Game::getPlayerByID` (static); `Game::internalTeleport` (static); `Player::death` (static); … +2 more |
| `Player::dropCorpse` | 3 | `Creature::dropCorpse` (static); `Player::dropCorpse` (static); `Player::lastHitIsPlayer` (static) |
| `Player::getCorpse` | 2 | `Creature::getCorpse` (static); `Player::getCorpse` (static) |
| `Player::addInFightTicks` | 2 | `Condition::createCondition` (static); `Player::addInFightTicks` (static) |
| `Player::removeList` | 2 | `Game::removePlayer` (static); `Player::removeList` (static) |
| `Player::addList` | 2 | `Game::addPlayer` (static); `Player::addList` (static) |
| `Player::kickPlayer` | 3 | `CreatureEvents::playerLogout` (static); `Game::removeCreature` (static); `Player::kickPlayer` (static) |
| `Player::notifyStatusChange` | 1 | `Player::notifyStatusChange` (static) |
| `Player::removeVIP` | 2 | `IOLoginData::removeVIPEntry` (static); `Player::removeVIP` (static) |
| `Player::addVIP` | 2 | `IOLoginData::addVIPEntry` (static); `Player::addVIP` (static) |
| `Player::addVIPInternal` | 1 | `Player::addVIPInternal` (static) |
| `Player::editVIP` | 2 | `IOLoginData::editVIPEntry` (static); `Player::editVIP` (static) |
| `Player::autoCloseContainers` | 1 | `Player::autoCloseContainers` (static) |
| `Player::hasCapacity` | 1 | `Player::hasCapacity` (static) |
| `Player::queryAdd` | 2 | `MoveEvents::onPlayerEquip` (static); `Player::queryAdd` (static) |
| `Player::queryMaxCount` | 1 | `Player::queryMaxCount` (static) |
| `Player::queryRemove` | 1 | `Player::queryRemove` (static) |
| `Player::queryDestination` | 1 | `Player::queryDestination` (static) |
| `Player::addThing` | 1 | `Player::addThing` (static) |
| `Player::updateThing` | 1 | `Player::updateThing` (static) |
| `Player::replaceThing` | 1 | `Player::replaceThing` (static) |
| `Player::removeThing` | 1 | `Player::removeThing` (static) |
| `Player::getThingIndex` | 1 | `Player::getThingIndex` (static) |
| `Player::getItemTypeCount` | 1 | `Player::getItemTypeCount` (static) |
| `Player::removeItemOfType` | 2 | `Game::internalRemoveItems` (static); `Player::removeItemOfType` (static) |
| `Player::getAllItemTypeCount` | 1 | `Player::getAllItemTypeCount` (static) |
| `Player::getThing` | 1 | `Player::getThing` (static) |
| `Player::postAddNotification` | 3 | `tfs::events::player::onInventoryUpdate` (static); `MoveEvents::onPlayerEquip` (static); `Player::postAddNotification` (static) |
| `Player::postRemoveNotification` | 3 | `tfs::events::player::onInventoryUpdate` (static); `MoveEvents::onPlayerDeEquip` (static); `Player::postRemoveNotification` (static) |
| `Player::updateSaleShopList` | 1 | `Player::updateSaleShopList` (static) |
| `Player::hasShopItemForSale` | 1 | `Player::hasShopItemForSale` (static) |
| `Player::internalAddThing` | 1 | `Player::internalAddThing` (static) |
| `Player::setFollowCreature` | 2 | `Creature::setFollowCreature` (static); `Player::setFollowCreature` (static) |
| `Player::setAttackedCreature` | 3 | `Creature::setAttackedCreature` (static); `Game::checkCreatureAttack` (static); `Player::setAttackedCreature` (static) |
| `Player::removeAttackedCreature` | 2 | `Creature::removeAttackedCreature` (static); `Player::removeAttackedCreature` (static) |
| `Player::goToFollowCreature` | 1 | `Player::goToFollowCreature` (static) |
| `Player::getPathSearchParams` | 2 | `Creature::getPathSearchParams` (static); `Player::getPathSearchParams` (static) |
| `Player::doAttacking` | 4 | `Game::checkCreatureAttack` (static); `Player::doAttacking` (static); `Weapon::useFist` (static); `Weapons::getWeapon` (static) |
| `Player::getGainedExperience` | 1 | `Player::getGainedExperience` (static) |
| `Player::onUnfollowCreature` | 2 | `Creature::onUnfollowCreature` (static); `Player::onUnfollowCreature` (static) |
| `Player::setChaseMode` | 1 | `Player::setChaseMode` (static) |
| `Player::onWalkAborted` | 1 | `Player::onWalkAborted` (static) |
| `Player::onWalkComplete` | 1 | `Player::onWalkComplete` (static) |
| `Player::stopWalk` | 1 | `Player::stopWalk` (static) |
| `Player::getCreatureLight` | 1 | `Player::getCreatureLight` (static) |
| `Player::updateItemsLight` | 2 | `Game::changeLight` (static); `Player::updateItemsLight` (static) |
| `Player::onAddCondition` | 2 | `Creature::onAddCondition` (static); `Player::onAddCondition` (static) |
| `Player::onAddCombatCondition` | 1 | `Player::onAddCombatCondition` (static) |
| `Player::onEndCondition` | 2 | `Creature::onEndCondition` (static); `Player::onEndCondition` (static) |
| `Player::onCombatRemoveCondition` | 3 | `Creature::onCombatRemoveCondition` (static); `Game::internalRemoveItem` (static); `Player::onCombatRemoveCondition` (static) |
| `Player::onAttackedCreature` | 3 | `Combat::isInPvpZone` (static); `Creature::onAttackedCreature` (static); `Player::onAttackedCreature` (static) |
| `Player::onAttacked` | 2 | `Creature::onAttacked` (static); `Player::onAttacked` (static) |
| `Player::onIdleStatus` | 2 | `Creature::onIdleStatus` (static); `Player::onIdleStatus` (static) |
| `Player::onAttackedCreatureDrainHealth` | 3 | `Combat::isPlayerCombat` (static); `Creature::onAttackedCreatureDrainHealth` (static); `Player::onAttackedCreatureDrainHealth` (static) |
| `Player::onTargetCreatureGainHealth` | 1 | `Player::onTargetCreatureGainHealth` (static) |
| `Player::onKilledCreature` | 4 | `Combat::isInPvpZone` (static); `Condition::createCondition` (static); `Creature::onKilledCreature` (static); `Player::onKilledCreature` (static) |
| `Player::gainExperience` | 1 | `Player::gainExperience` (static) |
| `Player::onGainExperience` | 2 | `Creature::onGainExperience` (static); `Player::onGainExperience` (static) |
| `Player::onGainSharedExperience` | 1 | `Player::onGainSharedExperience` (static) |
| `Player::isImmune` | 0 | — |
| `Player::isImmune` | 2 | `Creature::isImmune` (static); `Player::isImmune` (static) |
| `Player::isAttackable` | 1 | `Player::isAttackable` (static) |
| `Player::lastHitIsPlayer` | 1 | `Player::lastHitIsPlayer` (static) |
| `Player::changeHealth` | 2 | `Creature::changeHealth` (static); `Player::changeHealth` (static) |
| `Player::changeMana` | 1 | `Player::changeMana` (static) |
| `Player::changeSoul` | 1 | `Player::changeSoul` (static) |
| `Player::canWear` | 2 | `Outfits::getInstance` (static); `Player::canWear` (static) |
| `Player::hasOutfit` | 2 | `Outfits::getInstance` (static); `Player::hasOutfit` (static) |
| `Player::addOutfit` | 1 | `Player::addOutfit` (static) |
| `Player::removeOutfit` | 1 | `Player::removeOutfit` (static) |
| `Player::removeOutfitAddon` | 1 | `Player::removeOutfitAddon` (static) |
| `Player::getOutfitAddons` | 1 | `Player::getOutfitAddons` (static) |
| `Player::setSex` | 1 | `Player::setSex` (static) |
| `Player::getSkull` | 1 | `Player::getSkull` (static) |
| `Player::getSkullClient` | 2 | `Creature::getSkullClient` (static); `Player::getSkullClient` (static) |
| `Player::hasAttacked` | 1 | `Player::hasAttacked` (static) |
| `Player::addAttacked` | 1 | `Player::addAttacked` (static) |
| `Player::removeAttacked` | 1 | `Player::removeAttacked` (static) |
| `Player::clearAttacked` | 1 | `Player::clearAttacked` (static) |
| `Player::addUnjustifiedDead` | 1 | `Player::addUnjustifiedDead` (static) |
| `Player::checkSkullTicks` | 1 | `Player::checkSkullTicks` (static) |
| `Player::isPromoted` | 2 | `Player::isPromoted` (static); `Vocations::getPromotedVocation` (static) |
| `Player::getLostPercent` | 1 | `Player::getLostPercent` (static) |
| `Player::learnInstantSpell` | 1 | `Player::learnInstantSpell` (static) |
| `Player::forgetInstantSpell` | 1 | `Player::forgetInstantSpell` (static) |
| `Player::hasLearnedInstantSpell` | 1 | `Player::hasLearnedInstantSpell` (static) |
| `Player::isInWar` | 1 | `Player::isInWar` (static) |
| `Player::isInWarList` | 1 | `Player::isInWarList` (static) |
| `Player::isPremium` | 1 | `Player::isPremium` (static) |
| `Player::setPremiumTime` | 1 | `Player::setPremiumTime` (static) |
| `Player::getPartyShield` | 1 | `Player::getPartyShield` (static) |
| `Player::isInviting` | 1 | `Player::isInviting` (static) |
| `Player::isPartner` | 1 | `Player::isPartner` (static) |
| `Player::isGuildMate` | 1 | `Player::isGuildMate` (static) |
| `Player::sendPlayerPartyIcons` | 1 | `Player::sendPlayerPartyIcons` (static) |
| `Player::addPartyInvitation` | 1 | `Player::addPartyInvitation` (static) |
| `Player::removePartyInvitation` | 1 | `Player::removePartyInvitation` (static) |
| `Player::clearPartyInvitations` | 1 | `Player::clearPartyInvitations` (static) |
| `Player::getGuildEmblem` | 1 | `Player::getGuildEmblem` (static) |
| `Player::getRandomMount` | 1 | `Player::getRandomMount` (static) |
| `Player::getCurrentMount` | 1 | `Player::getCurrentMount` (static) |
| `Player::setCurrentMount` | 1 | `Player::setCurrentMount` (static) |
| `Player::toggleMount` | 4 | `Game::changeSpeed` (static); `Game::internalCreatureChangeOutfit` (static); `Outfits::getInstance` (static); `Player::toggleMount` (static) |
| `Player::tameMount` | 1 | `Player::tameMount` (static) |
| `Player::untameMount` | 2 | `Game::internalCreatureChangeOutfit` (static); `Player::untameMount` (static) |
| `Player::hasMount` | 1 | `Player::hasMount` (static) |
| `Player::hasMounts` | 1 | `Player::hasMounts` (static) |
| `Player::dismount` | 2 | `Game::changeSpeed` (static); `Player::dismount` (static) |
| `Player::addOfflineTrainingTries` | 4 | `CreatureEvents::playerAdvance` (static); `tfs::events::player::onGainSkillTries` (static); `Player::addOfflineTrainingTries` (static); `Player::getBasisPointLevel` (static) |
| `Player::hasModalWindowOpen` | 1 | `Player::hasModalWindowOpen` (static) |
| `Player::onModalWindowHandled` | 1 | `Player::onModalWindowHandled` (static) |
| `Player::sendModalWindow` | 1 | `Player::sendModalWindow` (static) |
| `Player::clearModalWindows` | 1 | `Player::clearModalWindows` (static) |
| `Player::sendClosePrivate` | 2 | `Chat::removeUserFromChannel` (static); `Player::sendClosePrivate` (static) |
| `Player::getMoney` | 1 | `Player::getMoney` (static) |
| `Player::getMaxVIPEntries` | 1 | `Player::getMaxVIPEntries` (static) |
| `Player::getMaxDepotItems` | 1 | `Player::getMaxDepotItems` (static) |
| `Player::getMuteConditions` | 1 | `Player::getMuteConditions` (static) |
| `Player::setGuild` | 1 | `Player::setGuild` (static) |
| `Player::updateRegeneration` | 1 | `Player::updateRegeneration` (static) |

### `podium.h` (header declarations)

15 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Podium` | 0 | — |
| `Podium::Podium` | 0 | — |
| `Podium::direction` | 0 | — |
| `Podium::flags` | 0 | — |
| `Podium::outfit` | 0 | — |
| `Podium::getDirection` | 0 | — |
| `Podium::getOutfit` | 0 | — |
| `Podium::getPodium (const)` | 0 | — |
| `Podium::getPodium (non-const)` | 0 | — |
| `Podium::hasFlag` | 0 | — |
| `Podium::setDirection` | 0 | — |
| `Podium::setFlagValue` | 0 | — |
| `Podium::setFlags` | 0 | — |
| `Podium::setOutfit` | 0 | — |
| `FS_PODIUM_H` | 0 | — |

### `podium`

4 node(s), 4 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Podium::readAttr` | 0 | — |
| `Podium::serializeAttr` | 0 | — |
| `Podium::readAttr` | 3 | `Game::updatePodium` (static); `Item::readAttr` (static); `Podium::readAttr` (static) |
| `Podium::serializeAttr` | 1 | `Podium::serializeAttr` (static) |

### `position.h` (header declarations)

35 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Direction` | 0 | — |
| `Direction::DIRECTION_NORTH` | 0 | — |
| `Direction::DIRECTION_EAST` | 0 | — |
| `Direction::DIRECTION_SOUTH` | 0 | — |
| `Direction::DIRECTION_WEST` | 0 | — |
| `Direction::DIRECTION_DIAGONAL_MASK` | 0 | — |
| `Direction::DIRECTION_SOUTHWEST` | 0 | — |
| `Direction::DIRECTION_SOUTHEAST` | 0 | — |
| `Direction::DIRECTION_NORTHWEST` | 0 | — |
| `Direction::DIRECTION_NORTHEAST` | 0 | — |
| `Direction::DIRECTION_LAST` | 0 | — |
| `Direction::DIRECTION_NONE` | 0 | — |
| `tfs` | 0 | — |
| `tfs::abs` | 0 | — |
| `Position` | 0 | — |
| `Position::Position()` | 0 | — |
| `Position::Position(uint16_t,uint16_t,uint8_t)` | 0 | — |
| `Position::isInRange(2D)` | 0 | — |
| `Position::isInRange(3D)` | 0 | — |
| `Position::getOffsetX` | 0 | — |
| `Position::getOffsetY` | 0 | — |
| `Position::getOffsetZ` | 0 | — |
| `Position::getDistanceX` | 0 | — |
| `Position::getDistanceY` | 0 | — |
| `Position::getDistanceZ` | 0 | — |
| `Position::x` | 0 | — |
| `Position::y` | 0 | — |
| `Position::z` | 0 | — |
| `Position::operator==` | 0 | — |
| `Position::operator!=` | 0 | — |
| `Position::operator<` | 0 | — |
| `Position::getX` | 0 | — |
| `Position::getY` | 0 | — |
| `Position::getZ` | 0 | — |
| `operator<<(std::ostream&,const Position&)` | 0 | — |

### `position`

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `operator<<(std::ostream&,const Position&)` | 0 | — |

### `protocol.h` (header declarations)

32 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Protocol` | 0 | — |
| `Protocol::Protocol` | 0 | — |
| `Protocol::~Protocol` | 0 | — |
| `Protocol::parsePacket` | 0 | — |
| `Protocol::onSendMessage` | 0 | — |
| `Protocol::onRecvMessage` | 0 | — |
| `Protocol::onRecvFirstMessage` | 0 | — |
| `Protocol::onConnect` | 0 | — |
| `Protocol::isConnectionExpired` | 0 | — |
| `Protocol::getConnection` | 0 | — |
| `Protocol::getIP` | 0 | — |
| `Protocol::getOutputBuffer` | 0 | — |
| `Protocol::getCurrentBuffer` | 0 | — |
| `Protocol::send` | 0 | — |
| `Protocol::getNextSequenceId` | 0 | — |
| `Protocol::RSA_BUFFER_LENGTH` | 0 | — |
| `Protocol::disconnect` | 0 | — |
| `Protocol::enableXTEAEncryption` | 0 | — |
| `Protocol::setXTEAKey` | 0 | — |
| `Protocol::setChecksumMode` | 0 | — |
| `Protocol::RSA_decrypt` | 0 | — |
| `Protocol::deflateMessage` | 0 | — |
| `Protocol::setRawMessages` | 0 | — |
| `Protocol::release` | 0 | — |
| `Protocol::outputBuffer` | 0 | — |
| `Protocol::connection` | 0 | — |
| `Protocol::key` | 0 | — |
| `Protocol::sequenceNumber` | 0 | — |
| `Protocol::encryptionEnabled` | 0 | — |
| `Protocol::checksumMode` | 0 | — |
| `Protocol::rawMessages` | 0 | — |
| `Protocol::zstream` | 0 | — |

### `protocol`

9 node(s), 7 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `(anonymous)::XTEA_encrypt` | 0 | — |
| `(anonymous)::XTEA_decrypt` | 0 | — |
| `Protocol::~Protocol` | 0 | — |
| `Protocol::onSendMessage` | 1 | `Protocol::onSendMessage` (static) |
| `Protocol::onRecvMessage` | 1 | `Protocol::onRecvMessage` (static) |
| `Protocol::getOutputBuffer` | 2 | `tfs::net::make_output_message` (static); `Protocol::getOutputBuffer` (static) |
| `Protocol::RSA_decrypt` | 1 | `Protocol::RSA_decrypt` (static) |
| `Protocol::deflateMessage` | 1 | `Protocol::deflateMessage` (static) |
| `Protocol::getIP` | 1 | `Protocol::getIP` (static) |

### `protocolgame.h` (header declarations)

32 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `SessionEndTypes_t` | 0 | — |
| `SessionEndTypes_t::SESSION_END_LOGOUT` | 0 | — |
| `SessionEndTypes_t::SESSION_END_UNKNOWN` | 0 | — |
| `SessionEndTypes_t::SESSION_END_FORCECLOSE` | 0 | — |
| `SessionEndTypes_t::SESSION_END_UNKNOWN2` | 0 | — |
| `ProtocolGame_ptr` | 0 | — |
| `g_game` | 0 | — |
| `TextMessage` | 0 | — |
| `TextMessage::type` | 0 | — |
| `TextMessage::text` | 0 | — |
| `TextMessage::position` | 0 | — |
| `TextMessage::channelId` | 0 | — |
| `TextMessage::primary` | 0 | — |
| `TextMessage::secondary` | 0 | — |
| `TextMessage::TextMessage` | 0 | — |
| `TextMessage::TextMessage` | 0 | — |
| `ProtocolGame` | 0 | — |
| `ProtocolGame::server_sends_first` | 0 | — |
| `ProtocolGame::protocol_identifier` | 0 | — |
| `ProtocolGame::use_checksum` | 0 | — |
| `ProtocolGame::protocol_name` | 0 | — |
| `ProtocolGame::ProtocolGame` | 0 | — |
| `ProtocolGame::getVersion` | 0 | — |
| `ProtocolGame::getThis` | 0 | — |
| `ProtocolGame::knownCreatureSet` | 0 | — |
| `ProtocolGame::player` | 0 | — |
| `ProtocolGame::eventConnect` | 0 | — |
| `ProtocolGame::challengeTimestamp` | 0 | — |
| `ProtocolGame::version` | 0 | — |
| `ProtocolGame::challengeRandom` | 0 | — |
| `ProtocolGame::debugAssertSent` | 0 | — |
| `ProtocolGame::acceptPackets` | 0 | — |

### `protocolgame`

252 node(s), 262 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_creatureEvents` | 0 | — |
| `g_chat` | 0 | — |
| `(anonymous)::waitList` | 0 | — |
| `(anonymous)::priorityEnd` | 0 | — |
| `(anonymous)::findClient` | 0 | — |
| `(anonymous)::getWaitTime` | 0 | — |
| `(anonymous)::getTimeout` | 0 | — |
| `(anonymous)::clientLogin` | 0 | — |
| `(anonymous)::getClientDamageType` | 0 | — |
| `ProtocolGame::release` | 3 | `tfs::net::remove_protocol_from_autosend` (static); `Protocol::release` (static); `ProtocolGame::release` (static) |
| `ProtocolGame::login` | 7 | `IOBan::getAccountBanInfo` (static); `IOBan::isPlayerNamelocked` (static); `IOLoginData::loadPlayerById` (static); `IOLoginData::preloadPlayer` (static); `tfs::net::insert_protocol_to_autosend` (static); … +2 more |
| `ProtocolGame::connect` | 4 | `Chat::removeUserFromAllChannels` (static); `CreatureEvents::playerReconnect` (static); `ProtocolGame::connect` (static); `ProtocolGame::release` (static) |
| `ProtocolGame::logout` | 2 | `CreatureEvents::playerLogout` (static); `ProtocolGame::logout` (static) |
| `ProtocolGame::onRecvFirstMessage` | 5 | `IOBan::getIpBanInfo` (static); `Database::getInstance` (static); `Protocol::RSA_decrypt` (static); `ProtocolGame::onRecvFirstMessage` (static); `ProtocolGame::login` (dynamic/curated) |
| `ProtocolGame::onConnect` | 2 | `tfs::net::make_output_message` (static); `ProtocolGame::onConnect` (static) |
| `ProtocolGame::disconnectClient` | 2 | `tfs::net::make_output_message` (static); `ProtocolGame::disconnectClient` (static) |
| `ProtocolGame::writeToOutputBuffer` | 1 | `ProtocolGame::writeToOutputBuffer` (static) |
| `ProtocolGame::parsePacket` | 76 | `ProtocolGame::parsePacket` (static); `ProtocolGame::logout` (dynamic/curated); `Game::playerReceivePingBack` (dynamic/curated); `Game::playerReceivePing` (dynamic/curated); `ProtocolGame::parseExtendedOpcode` (dynamic/curated); … +71 more |
| `ClientPacket::0x0F_QuitInLoading` | 0 | — |
| `ClientPacket::0x14_Logout` | 0 | — |
| `ClientPacket::0x1D_PingBack` | 0 | — |
| `ClientPacket::0x1E_Ping` | 0 | — |
| `ClientPacket::0x32_ExtendedOpcode` | 0 | — |
| `ClientPacket::0x64_AutoWalk` | 0 | — |
| `ClientPacket::0x65_MoveNorth` | 0 | — |
| `ClientPacket::0x66_MoveEast` | 0 | — |
| `ClientPacket::0x67_MoveSouth` | 0 | — |
| `ClientPacket::0x68_MoveWest` | 0 | — |
| `ClientPacket::0x69_StopAutoWalk` | 0 | — |
| `ClientPacket::0x6A_MoveNorthEast` | 0 | — |
| `ClientPacket::0x6B_MoveSouthEast` | 0 | — |
| `ClientPacket::0x6C_MoveSouthWest` | 0 | — |
| `ClientPacket::0x6D_MoveNorthWest` | 0 | — |
| `ClientPacket::0x6F_TurnNorth` | 0 | — |
| `ClientPacket::0x70_TurnEast` | 0 | — |
| `ClientPacket::0x71_TurnSouth` | 0 | — |
| `ClientPacket::0x72_TurnWest` | 0 | — |
| `ClientPacket::0x77_EquipObject` | 0 | — |
| `ClientPacket::0x78_Throw` | 0 | — |
| `ClientPacket::0x79_LookInShop` | 0 | — |
| `ClientPacket::0x7A_PlayerPurchase` | 0 | — |
| `ClientPacket::0x7B_PlayerSale` | 0 | — |
| `ClientPacket::0x7C_CloseShop` | 0 | — |
| `ClientPacket::0x7D_RequestTrade` | 0 | — |
| `ClientPacket::0x7E_LookInTrade` | 0 | — |
| `ClientPacket::0x7F_AcceptTrade` | 0 | — |
| `ClientPacket::0x80_CloseTrade` | 0 | — |
| `ClientPacket::0x82_UseItem` | 0 | — |
| `ClientPacket::0x83_UseItemEx` | 0 | — |
| `ClientPacket::0x84_UseWithCreature` | 0 | — |
| `ClientPacket::0x85_RotateItem` | 0 | — |
| `ClientPacket::0x86_EditPodiumRequest` | 0 | — |
| `ClientPacket::0x87_CloseContainer` | 0 | — |
| `ClientPacket::0x88_UpArrowContainer` | 0 | — |
| `ClientPacket::0x89_TextWindow` | 0 | — |
| `ClientPacket::0x8A_HouseWindow` | 0 | — |
| `ClientPacket::0x8B_WrapItem` | 0 | — |
| `ClientPacket::0x8C_LookAt` | 0 | — |
| `ClientPacket::0x8D_LookInBattleList` | 0 | — |
| `ClientPacket::0x8E_JoinAggression` | 0 | — |
| `ClientPacket::0x96_Say` | 0 | — |
| `ClientPacket::0x97_RequestChannels` | 0 | — |
| `ClientPacket::0x98_OpenChannel` | 0 | — |
| `ClientPacket::0x99_CloseChannel` | 0 | — |
| `ClientPacket::0x9A_OpenPrivateChannel` | 0 | — |
| `ClientPacket::0x9E_CloseNpcChannel` | 0 | — |
| `ClientPacket::0xA0_FightModes` | 0 | — |
| `ClientPacket::0xA1_Attack` | 0 | — |
| `ClientPacket::0xA2_Follow` | 0 | — |
| `ClientPacket::0xA3_InviteToParty` | 0 | — |
| `ClientPacket::0xA4_JoinParty` | 0 | — |
| `ClientPacket::0xA5_RevokePartyInvite` | 0 | — |
| `ClientPacket::0xA6_PassPartyLeadership` | 0 | — |
| `ClientPacket::0xA7_LeaveParty` | 0 | — |
| `ClientPacket::0xA8_EnableSharedPartyExperience` | 0 | — |
| `ClientPacket::0xAA_CreatePrivateChannel` | 0 | — |
| `ClientPacket::0xAB_ChannelInvite` | 0 | — |
| `ClientPacket::0xAC_ChannelExclude` | 0 | — |
| `ClientPacket::0xBE_CancelAttackAndFollow` | 0 | — |
| `ClientPacket::0xC9_UpdateTile` | 0 | — |
| `ClientPacket::0xCA_UpdateContainer` | 0 | — |
| `ClientPacket::0xCB_BrowseField` | 0 | — |
| `ClientPacket::0xCC_SeekInContainer` | 0 | — |
| `ClientPacket::0xD2_RequestOutfit` | 0 | — |
| `ClientPacket::0xD3_SetOutfit` | 0 | — |
| `ClientPacket::0xDC_AddVip` | 0 | — |
| `ClientPacket::0xDD_RemoveVip` | 0 | — |
| `ClientPacket::0xDE_EditVip` | 0 | — |
| `ClientPacket::0xE7_ThankYou` | 0 | — |
| `ClientPacket::0xE8_DebugAssert` | 0 | — |
| `ClientPacket::0xF2_RuleViolationReport` | 0 | — |
| `ClientPacket::0xF3_GetObjectInfo` | 0 | — |
| `ClientPacket::0xF4_MarketLeave` | 0 | — |
| `ClientPacket::0xF5_MarketBrowse` | 0 | — |
| `ClientPacket::0xF6_MarketCreateOffer` | 0 | — |
| `ClientPacket::0xF7_MarketCancelOffer` | 0 | — |
| `ClientPacket::0xF8_MarketAcceptOffer` | 0 | — |
| `ClientPacket::0xF9_ModalWindowAnswer` | 0 | — |
| `ProtocolGame::GetTileDescription` | 1 | `ProtocolGame::GetTileDescription` (static) |
| `ProtocolGame::GetMapDescription` | 1 | `ProtocolGame::GetMapDescription` (static) |
| `ProtocolGame::GetFloorDescription` | 1 | `ProtocolGame::GetFloorDescription` (static) |
| `ProtocolGame::checkCreatureAsKnown` | 1 | `ProtocolGame::checkCreatureAsKnown` (static) |
| `ProtocolGame::canSee` | 1 | `ProtocolGame::canSee` (static) |
| `ProtocolGame::canSee(Position)` | 1 | `ProtocolGame::canSee` (static) |
| `ProtocolGame::canSee(xyz)` | 1 | `ProtocolGame::canSee` (static) |
| `ProtocolGame::parseChannelInvite` | 1 | `ProtocolGame::parseChannelInvite` (static) |
| `ProtocolGame::parseChannelExclude` | 1 | `ProtocolGame::parseChannelExclude` (static) |
| `ProtocolGame::parseOpenChannel` | 1 | `ProtocolGame::parseOpenChannel` (static) |
| `ProtocolGame::parseCloseChannel` | 1 | `ProtocolGame::parseCloseChannel` (static) |
| `ProtocolGame::parseOpenPrivateChannel` | 1 | `ProtocolGame::parseOpenPrivateChannel` (static) |
| `ProtocolGame::parseAutoWalk` | 1 | `ProtocolGame::parseAutoWalk` (static) |
| `ProtocolGame::parseSetOutfit` | 1 | `ProtocolGame::parseSetOutfit` (static) |
| `ProtocolGame::parseEditPodiumRequest` | 1 | `ProtocolGame::parseEditPodiumRequest` (static) |
| `ProtocolGame::parseUseItem` | 1 | `ProtocolGame::parseUseItem` (static) |
| `ProtocolGame::parseUseItemEx` | 1 | `ProtocolGame::parseUseItemEx` (static) |
| `ProtocolGame::parseUseWithCreature` | 1 | `ProtocolGame::parseUseWithCreature` (static) |
| `ProtocolGame::parseCloseContainer` | 1 | `ProtocolGame::parseCloseContainer` (static) |
| `ProtocolGame::parseUpArrowContainer` | 1 | `ProtocolGame::parseUpArrowContainer` (static) |
| `ProtocolGame::parseUpdateContainer` | 1 | `ProtocolGame::parseUpdateContainer` (static) |
| `ProtocolGame::parseThrow` | 1 | `ProtocolGame::parseThrow` (static) |
| `ProtocolGame::parseLookAt` | 1 | `ProtocolGame::parseLookAt` (static) |
| `ProtocolGame::parseLookInBattleList` | 1 | `ProtocolGame::parseLookInBattleList` (static) |
| `ProtocolGame::parseSay` | 1 | `ProtocolGame::parseSay` (static) |
| `ProtocolGame::parseFightModes` | 1 | `ProtocolGame::parseFightModes` (static) |
| `ProtocolGame::parseAttack` | 1 | `ProtocolGame::parseAttack` (static) |
| `ProtocolGame::parseFollow` | 1 | `ProtocolGame::parseFollow` (static) |
| `ProtocolGame::parseEquipObject` | 1 | `ProtocolGame::parseEquipObject` (static) |
| `ProtocolGame::parseTextWindow` | 1 | `ProtocolGame::parseTextWindow` (static) |
| `ProtocolGame::parseHouseWindow` | 1 | `ProtocolGame::parseHouseWindow` (static) |
| `ProtocolGame::parseWrapItem` | 1 | `ProtocolGame::parseWrapItem` (static) |
| `ProtocolGame::parseLookInShop` | 1 | `ProtocolGame::parseLookInShop` (static) |
| `ProtocolGame::parsePlayerPurchase` | 1 | `ProtocolGame::parsePlayerPurchase` (static) |
| `ProtocolGame::parsePlayerSale` | 1 | `ProtocolGame::parsePlayerSale` (static) |
| `ProtocolGame::parseRequestTrade` | 1 | `ProtocolGame::parseRequestTrade` (static) |
| `ProtocolGame::parseLookInTrade` | 1 | `ProtocolGame::parseLookInTrade` (static) |
| `ProtocolGame::parseAddVip` | 1 | `ProtocolGame::parseAddVip` (static) |
| `ProtocolGame::parseRemoveVip` | 1 | `ProtocolGame::parseRemoveVip` (static) |
| `ProtocolGame::parseEditVip` | 1 | `ProtocolGame::parseEditVip` (static) |
| `ProtocolGame::parseRotateItem` | 1 | `ProtocolGame::parseRotateItem` (static) |
| `ProtocolGame::parseRuleViolationReport` | 1 | `ProtocolGame::parseRuleViolationReport` (static) |
| `ProtocolGame::parseDebugAssert` | 1 | `ProtocolGame::parseDebugAssert` (static) |
| `ProtocolGame::parseInviteToParty` | 1 | `ProtocolGame::parseInviteToParty` (static) |
| `ProtocolGame::parseJoinParty` | 1 | `ProtocolGame::parseJoinParty` (static) |
| `ProtocolGame::parseRevokePartyInvite` | 1 | `ProtocolGame::parseRevokePartyInvite` (static) |
| `ProtocolGame::parsePassPartyLeadership` | 1 | `ProtocolGame::parsePassPartyLeadership` (static) |
| `ProtocolGame::parseEnableSharedPartyExperience` | 1 | `ProtocolGame::parseEnableSharedPartyExperience` (static) |
| `ProtocolGame::parseMarketLeave` | 1 | `ProtocolGame::parseMarketLeave` (static) |
| `ProtocolGame::parseMarketBrowse` | 1 | `ProtocolGame::parseMarketBrowse` (static) |
| `ProtocolGame::parseMarketCreateOffer` | 1 | `ProtocolGame::parseMarketCreateOffer` (static) |
| `ProtocolGame::parseMarketCancelOffer` | 1 | `ProtocolGame::parseMarketCancelOffer` (static) |
| `ProtocolGame::parseMarketAcceptOffer` | 1 | `ProtocolGame::parseMarketAcceptOffer` (static) |
| `ProtocolGame::parseModalWindowAnswer` | 1 | `ProtocolGame::parseModalWindowAnswer` (static) |
| `ProtocolGame::parseBrowseField` | 1 | `ProtocolGame::parseBrowseField` (static) |
| `ProtocolGame::parseSeekInContainer` | 1 | `ProtocolGame::parseSeekInContainer` (static) |
| `ProtocolGame::sendOpenPrivateChannel` | 1 | `ProtocolGame::sendOpenPrivateChannel` (static) |
| `ProtocolGame::sendChannelEvent` | 1 | `ProtocolGame::sendChannelEvent` (static) |
| `ProtocolGame::sendCreatureOutfit` | 1 | `ProtocolGame::sendCreatureOutfit` (static) |
| `ProtocolGame::sendCreatureLight` | 1 | `ProtocolGame::sendCreatureLight` (static) |
| `ProtocolGame::sendCreatureWalkthrough` | 1 | `ProtocolGame::sendCreatureWalkthrough` (static) |
| `ProtocolGame::sendCreatureShield` | 1 | `ProtocolGame::sendCreatureShield` (static) |
| `ProtocolGame::sendCreatureSkull` | 1 | `ProtocolGame::sendCreatureSkull` (static) |
| `ProtocolGame::sendCreatureSquare` | 1 | `ProtocolGame::sendCreatureSquare` (static) |
| `ProtocolGame::sendTutorial` | 1 | `ProtocolGame::sendTutorial` (static) |
| `ProtocolGame::sendAddMarker` | 1 | `ProtocolGame::sendAddMarker` (static) |
| `ProtocolGame::sendReLoginWindow` | 1 | `ProtocolGame::sendReLoginWindow` (static) |
| `ProtocolGame::sendStats` | 1 | `ProtocolGame::sendStats` (static) |
| `ProtocolGame::sendExperienceTracker` | 1 | `ProtocolGame::sendExperienceTracker` (static) |
| `ProtocolGame::sendClientFeatures` | 1 | `ProtocolGame::sendClientFeatures` (static) |
| `ProtocolGame::sendBasicData` | 1 | `ProtocolGame::sendBasicData` (static) |
| `ProtocolGame::sendTextMessage` | 1 | `ProtocolGame::sendTextMessage` (static) |
| `ProtocolGame::sendClosePrivate` | 1 | `ProtocolGame::sendClosePrivate` (static) |
| `ProtocolGame::sendCreatePrivateChannel` | 1 | `ProtocolGame::sendCreatePrivateChannel` (static) |
| `ProtocolGame::sendChannelsDialog` | 2 | `Chat::getChannelList` (static); `ProtocolGame::sendChannelsDialog` (static) |
| `ProtocolGame::sendChannel` | 1 | `ProtocolGame::sendChannel` (static) |
| `ProtocolGame::sendChannelMessage` | 1 | `ProtocolGame::sendChannelMessage` (static) |
| `ProtocolGame::sendIcons` | 1 | `ProtocolGame::sendIcons` (static) |
| `ProtocolGame::sendContainer` | 1 | `ProtocolGame::sendContainer` (static) |
| `ProtocolGame::sendEmptyContainer` | 1 | `ProtocolGame::sendEmptyContainer` (static) |
| `ProtocolGame::sendShop` | 1 | `ProtocolGame::sendShop` (static) |
| `ProtocolGame::sendCloseShop` | 1 | `ProtocolGame::sendCloseShop` (static) |
| `ProtocolGame::sendSaleItemList` | 1 | `ProtocolGame::sendSaleItemList` (static) |
| `ProtocolGame::sendResourceBalance` | 1 | `ProtocolGame::sendResourceBalance` (static) |
| `ProtocolGame::sendStoreBalance` | 1 | `ProtocolGame::sendStoreBalance` (static) |
| `ProtocolGame::sendMarketEnter` | 2 | `tfs::iomarket::getPlayerOfferCount` (static); `ProtocolGame::sendMarketEnter` (static) |
| `ProtocolGame::sendMarketLeave` | 1 | `ProtocolGame::sendMarketLeave` (static) |
| `ProtocolGame::sendMarketBrowseItem` | 1 | `ProtocolGame::sendMarketBrowseItem` (static) |
| `ProtocolGame::sendMarketAcceptOffer` | 1 | `ProtocolGame::sendMarketAcceptOffer` (static) |
| `ProtocolGame::sendMarketBrowseOwnOffers` | 1 | `ProtocolGame::sendMarketBrowseOwnOffers` (static) |
| `ProtocolGame::sendMarketCancelOffer` | 1 | `ProtocolGame::sendMarketCancelOffer` (static) |
| `ProtocolGame::sendMarketBrowseOwnHistory` | 1 | `ProtocolGame::sendMarketBrowseOwnHistory` (static) |
| `ProtocolGame::sendTradeItemRequest` | 1 | `ProtocolGame::sendTradeItemRequest` (static) |
| `ProtocolGame::sendCloseTrade` | 1 | `ProtocolGame::sendCloseTrade` (static) |
| `ProtocolGame::sendCloseContainer` | 1 | `ProtocolGame::sendCloseContainer` (static) |
| `ProtocolGame::sendCreatureTurn` | 1 | `ProtocolGame::sendCreatureTurn` (static) |
| `ProtocolGame::sendCreatureSay` | 1 | `ProtocolGame::sendCreatureSay` (static) |
| `ProtocolGame::sendToChannel` | 1 | `ProtocolGame::sendToChannel` (static) |
| `ProtocolGame::sendPrivateMessage` | 1 | `ProtocolGame::sendPrivateMessage` (static) |
| `ProtocolGame::sendCancelTarget` | 1 | `ProtocolGame::sendCancelTarget` (static) |
| `ProtocolGame::sendChangeSpeed` | 1 | `ProtocolGame::sendChangeSpeed` (static) |
| `ProtocolGame::sendCancelWalk` | 1 | `ProtocolGame::sendCancelWalk` (static) |
| `ProtocolGame::sendSkills` | 1 | `ProtocolGame::sendSkills` (static) |
| `ProtocolGame::sendPing` | 1 | `ProtocolGame::sendPing` (static) |
| `ProtocolGame::sendPingBack` | 1 | `ProtocolGame::sendPingBack` (static) |
| `ProtocolGame::sendDistanceShoot` | 1 | `ProtocolGame::sendDistanceShoot` (static) |
| `ProtocolGame::sendMagicEffect` | 1 | `ProtocolGame::sendMagicEffect` (static) |
| `ProtocolGame::sendCreatureHealth` | 1 | `ProtocolGame::sendCreatureHealth` (static) |
| `ProtocolGame::sendFYIBox` | 1 | `ProtocolGame::sendFYIBox` (static) |
| `ProtocolGame::sendMapDescription` | 1 | `ProtocolGame::sendMapDescription` (static) |
| `ProtocolGame::sendAddTileItem` | 1 | `ProtocolGame::sendAddTileItem` (static) |
| `ProtocolGame::sendUpdateTileItem` | 1 | `ProtocolGame::sendUpdateTileItem` (static) |
| `ProtocolGame::sendRemoveTileThing` | 1 | `ProtocolGame::sendRemoveTileThing` (static) |
| `ProtocolGame::sendUpdateTileCreature` | 1 | `ProtocolGame::sendUpdateTileCreature` (static) |
| `ProtocolGame::sendRemoveTileCreature` | 1 | `ProtocolGame::sendRemoveTileCreature` (static) |
| `ProtocolGame::sendUpdateTile` | 1 | `ProtocolGame::sendUpdateTile` (static) |
| `ProtocolGame::sendUpdateCreatureIcons` | 1 | `ProtocolGame::sendUpdateCreatureIcons` (static) |
| `ProtocolGame::sendPendingStateEntered` | 1 | `ProtocolGame::sendPendingStateEntered` (static) |
| `ProtocolGame::sendEnterWorld` | 1 | `ProtocolGame::sendEnterWorld` (static) |
| `ProtocolGame::sendFightModes` | 1 | `ProtocolGame::sendFightModes` (static) |
| `ProtocolGame::sendAddCreature` | 1 | `ProtocolGame::sendAddCreature` (static) |
| `ProtocolGame::sendMoveCreature` | 1 | `ProtocolGame::sendMoveCreature` (static) |
| `ProtocolGame::sendInventoryItem` | 1 | `ProtocolGame::sendInventoryItem` (static) |
| `ProtocolGame::sendItems` | 1 | `ProtocolGame::sendItems` (static) |
| `ProtocolGame::sendAddContainerItem` | 1 | `ProtocolGame::sendAddContainerItem` (static) |
| `ProtocolGame::sendUpdateContainerItem` | 1 | `ProtocolGame::sendUpdateContainerItem` (static) |
| `ProtocolGame::sendRemoveContainerItem` | 1 | `ProtocolGame::sendRemoveContainerItem` (static) |
| `ProtocolGame::sendTextWindow` | 1 | `ProtocolGame::sendTextWindow` (static) |
| `ProtocolGame::sendTextWindow(itemId)` | 1 | `ProtocolGame::sendTextWindow` (static) |
| `ProtocolGame::sendHouseWindow` | 1 | `ProtocolGame::sendHouseWindow` (static) |
| `ProtocolGame::sendCombatAnalyzer` | 1 | `ProtocolGame::sendCombatAnalyzer` (static) |
| `ProtocolGame::sendOutfitWindow` | 2 | `Outfits::getInstance` (static); `ProtocolGame::sendOutfitWindow` (static) |
| `ProtocolGame::sendPodiumWindow` | 2 | `Outfits::getInstance` (static); `ProtocolGame::sendPodiumWindow` (static) |
| `ProtocolGame::sendUpdatedVIPStatus` | 1 | `ProtocolGame::sendUpdatedVIPStatus` (static) |
| `ProtocolGame::sendVIP` | 1 | `ProtocolGame::sendVIP` (static) |
| `ProtocolGame::sendVIPEntries` | 2 | `IOLoginData::getVIPEntries` (static); `ProtocolGame::sendVIPEntries` (static) |
| `ProtocolGame::sendItemClasses` | 1 | `ProtocolGame::sendItemClasses` (static) |
| `ProtocolGame::sendSpellCooldown` | 1 | `ProtocolGame::sendSpellCooldown` (static) |
| `ProtocolGame::sendSpellGroupCooldown` | 1 | `ProtocolGame::sendSpellGroupCooldown` (static) |
| `ProtocolGame::sendUseItemCooldown` | 1 | `ProtocolGame::sendUseItemCooldown` (static) |
| `ProtocolGame::sendSupplyUsed` | 1 | `ProtocolGame::sendSupplyUsed` (static) |
| `ProtocolGame::sendModalWindow` | 1 | `ProtocolGame::sendModalWindow` (static) |
| `ProtocolGame::sendSessionEnd` | 2 | `tfs::net::make_output_message` (static); `ProtocolGame::sendSessionEnd` (static) |
| `ProtocolGame::AddCreature` | 1 | `ProtocolGame::AddCreature` (static) |
| `ProtocolGame::AddCreatureIcons` | 1 | `ProtocolGame::AddCreatureIcons` (static) |
| `ProtocolGame::AddPlayerStats` | 1 | `ProtocolGame::AddPlayerStats` (static) |
| `ProtocolGame::AddPlayerSkills` | 1 | `ProtocolGame::AddPlayerSkills` (static) |
| `ProtocolGame::AddOutfit` | 1 | `ProtocolGame::AddOutfit` (static) |
| `ProtocolGame::RemoveTileThing` | 1 | `ProtocolGame::RemoveTileThing` (static) |
| `ProtocolGame::RemoveTileCreature` | 1 | `ProtocolGame::RemoveTileCreature` (static) |
| `ProtocolGame::MoveUpCreature` | 1 | `ProtocolGame::MoveUpCreature` (static) |
| `ProtocolGame::MoveDownCreature` | 1 | `ProtocolGame::MoveDownCreature` (static) |
| `ProtocolGame::AddShopItem` | 1 | `ProtocolGame::AddShopItem` (static) |
| `ProtocolGame::parseExtendedOpcode` | 1 | `ProtocolGame::parseExtendedOpcode` (static) |

### `protocolstatus.h` (header declarations)

11 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ProtocolStatus` | 0 | — |
| `ProtocolStatus::server_sends_first` | 0 | — |
| `ProtocolStatus::protocol_identifier` | 0 | — |
| `ProtocolStatus::use_checksum` | 0 | — |
| `ProtocolStatus::protocol_name` | 0 | — |
| `ProtocolStatus::ProtocolStatus` | 0 | — |
| `ProtocolStatus::onRecvFirstMessage` | 0 | — |
| `ProtocolStatus::sendStatusString` | 0 | — |
| `ProtocolStatus::sendInfo` | 0 | — |
| `ProtocolStatus::start` | 0 | — |
| `ProtocolStatus::ipConnectMap` | 0 | — |

### `protocolstatus`

14 node(s), 8 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ProtocolStatus::ipConnectMap` | 0 | — |
| `ProtocolStatus::start` | 0 | — |
| `RequestedInfo_t` | 0 | — |
| `RequestedInfo_t::REQUEST_BASIC_SERVER_INFO` | 0 | — |
| `RequestedInfo_t::REQUEST_OWNER_SERVER_INFO` | 0 | — |
| `RequestedInfo_t::REQUEST_MISC_SERVER_INFO` | 0 | — |
| `RequestedInfo_t::REQUEST_PLAYERS_INFO` | 0 | — |
| `RequestedInfo_t::REQUEST_MAP_INFO` | 0 | — |
| `RequestedInfo_t::REQUEST_EXT_PLAYERS_INFO` | 0 | — |
| `RequestedInfo_t::REQUEST_PLAYER_STATUS_INFO` | 0 | — |
| `RequestedInfo_t::REQUEST_SERVER_SOFTWARE_INFO` | 0 | — |
| `ProtocolStatus::onRecvFirstMessage` | 3 | `ProtocolStatus::onRecvFirstMessage` (static); `ProtocolStatus::sendStatusString` (dynamic/curated); `ProtocolStatus::sendInfo` (dynamic/curated) |
| `ProtocolStatus::sendStatusString` | 2 | `tfs::net::make_output_message` (static); `ProtocolStatus::sendStatusString` (static) |
| `ProtocolStatus::sendInfo` | 3 | `Game::getPlayerByName` (static); `tfs::net::make_output_message` (static); `ProtocolStatus::sendInfo` (static) |

### `pugicast.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_PUGICAST_H` | 0 | — |

### `rsa.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_RSA_H` | 0 | — |

### `rsa`

2 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `decrypt` | 0 | — |
| `loadPEM` | 0 | — |

### `scheduler.h` (header declarations)

9 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_SCHEDULER_H` | 0 | — |
| `SchedulerTask` | 0 | — |
| `SchedulerTask::SchedulerTask` | 0 | — |
| `SchedulerTask::createSchedulerTask` | 0 | — |
| `Scheduler` | 0 | — |
| `Scheduler::addEvent` | 0 | — |
| `Scheduler::stopEvent` | 0 | — |
| `Scheduler::shutdown` | 0 | — |
| `Scheduler::get_executor` | 0 | — |

### `scheduler`

4 node(s), 3 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Scheduler::addEvent` | 1 | `Scheduler::addEvent` (static) |
| `Scheduler::stopEvent` | 1 | `Scheduler::stopEvent` (static) |
| `Scheduler::shutdown` | 1 | `Scheduler::shutdown` (static) |
| `createSchedulerTask` | 0 | — |

### `script.h` (header declarations)

6 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Scripts` | 0 | — |
| `Scripts::Scripts` | 0 | — |
| `Scripts::~Scripts` | 0 | — |
| `Scripts::loadScripts` | 0 | — |
| `Scripts::getScriptInterface` | 0 | — |
| `Scripts::scriptInterface` | 0 | — |

### `script`

4 node(s), 2 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_luaEnvironment` | 0 | — |
| `Scripts::Scripts` | 1 | `Scripts::Scripts` (static) |
| `Scripts::~Scripts` | 0 | — |
| `Scripts::loadScripts` | 1 | `Scripts::loadScripts` (static) |

### `scriptmanager.h` (header declarations)

7 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ScriptingManager` | 0 | — |
| `ScriptingManager::ScriptingManager` | 0 | — |
| `ScriptingManager::~ScriptingManager` | 0 | — |
| `ScriptingManager::ScriptingManager(const ScriptingManager&)` | 0 | — |
| `ScriptingManager::operator=` | 0 | — |
| `ScriptingManager::getInstance` | 0 | — |
| `ScriptingManager::loadScriptSystems` | 0 | — |

### `scriptmanager`

12 node(s), 4 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `g_actions` | 0 | — |
| `g_creatureEvents` | 0 | — |
| `g_chat` | 0 | — |
| `g_globalEvents` | 0 | — |
| `g_spells` | 0 | — |
| `g_talkActions` | 0 | — |
| `g_moveEvents` | 0 | — |
| `g_weapons` | 0 | — |
| `g_scripts` | 0 | — |
| `g_luaEnvironment_extern` | 0 | — |
| `ScriptingManager::~ScriptingManager` | 0 | — |
| `ScriptingManager::loadScriptSystems` | 4 | `tfs::events::load` (static); `Scripts::loadScripts` (static); `ScriptingManager::loadScriptSystems` (static); `Weapons::loadDefaults` (static) |

### `server.h` (header declarations)

33 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `ServiceBase` | 0 | — |
| `ServiceBase::~ServiceBase` | 0 | — |
| `ServiceBase::is_single_socket` | 0 | — |
| `ServiceBase::is_checksummed` | 0 | — |
| `ServiceBase::get_protocol_identifier` | 0 | — |
| `ServiceBase::get_protocol_name` | 0 | — |
| `ServiceBase::make_protocol` | 0 | — |
| `Service` | 0 | — |
| `Service::is_single_socket` | 0 | — |
| `Service::is_checksummed` | 0 | — |
| `Service::get_protocol_identifier` | 0 | — |
| `Service::get_protocol_name` | 0 | — |
| `Service::make_protocol` | 0 | — |
| `ServicePort` | 0 | — |
| `ServicePort::ServicePort` | 0 | — |
| `ServicePort::ServicePort(const ServicePort&)` | 0 | — |
| `ServicePort::operator=` | 0 | — |
| `ServicePort::io_context` | 0 | — |
| `ServicePort::acceptor` | 0 | — |
| `ServicePort::services` | 0 | — |
| `ServicePort::serverPort` | 0 | — |
| `ServicePort::pendingStart` | 0 | — |
| `ServiceManager` | 0 | — |
| `ServiceManager::ServiceManager` | 0 | — |
| `ServiceManager::ServiceManager(const ServiceManager&)` | 0 | — |
| `ServiceManager::operator=` | 0 | — |
| `ServiceManager::is_running` | 0 | — |
| `ServiceManager::acceptors` | 0 | — |
| `ServiceManager::io_context` | 0 | — |
| `ServiceManager::signals` | 0 | — |
| `ServiceManager::death_timer` | 0 | — |
| `ServiceManager::running` | 0 | — |
| `ServiceManager::add` | 0 | — |

### `server`

21 node(s), 14 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `(anonymous)::ConnectBlock` | 0 | — |
| `(anonymous)::ConnectBlock::lastAttempt` | 0 | — |
| `(anonymous)::ConnectBlock::blockTime` | 0 | — |
| `(anonymous)::ConnectBlock::count` | 0 | — |
| `(anonymous)::acceptConnection` | 0 | — |
| `(anonymous)::getListenAddress` | 0 | — |
| `(anonymous)::openAcceptor` | 0 | — |
| `ServiceManager::~ServiceManager` | 0 | — |
| `ServiceManager::die` | 1 | `ServiceManager::die` (static) |
| `ServiceManager::run` | 2 | `ServiceManager::run` (static); `Connection::parsePacket` (dynamic/curated) |
| `ServiceManager::stop` | 1 | `ServiceManager::stop` (static) |
| `ServicePort::~ServicePort` | 0 | — |
| `ServicePort::is_single_socket` | 1 | `ServicePort::is_single_socket` (static) |
| `ServicePort::get_protocol_names` | 1 | `ServicePort::get_protocol_names` (static) |
| `ServicePort::accept` | 2 | `ConnectionManager::getInstance` (static); `ServicePort::accept` (static) |
| `ServicePort::onAccept` | 1 | `ServicePort::onAccept` (static) |
| `ServicePort::make_protocol` | 1 | `ServicePort::make_protocol` (static) |
| `ServicePort::onStopServer` | 1 | `ServicePort::onStopServer` (static) |
| `ServicePort::open` | 1 | `ServicePort::open` (static) |
| `ServicePort::close` | 1 | `ServicePort::close` (static) |
| `ServicePort::add_service` | 1 | `ServicePort::add_service` (static) |

### `signals.h` (header declarations)

4 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_SIGNALS_H` | 0 | — |
| `Signals` | 0 | — |
| `Signals::Signals` | 0 | — |
| `Signals::asyncWait` | 0 | — |

### `signals`

7 node(s), 13 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `sigusr1Handler` | 2 | `Game::saveGameState` (static); `GlobalEvents::save` (static) |
| `sighupHandler` | 6 | `Chat::load` (static); `ConfigManager::load` (static); `tfs::events::reload` (static); `Monsters::reload` (static); `Npcs::reload` (static); … +1 more |
| `sigbreakHandler` | 1 | `Game::setGameState` (static) |
| `sigtermHandler` | 1 | `Game::setGameState` (static) |
| `sigintHandler` | 1 | `Game::setGameState` (static) |
| `dispatchSignalHandler` | 1 | `Dispatcher::addTask` (static) |
| `Signals::asyncWait` | 1 | `Signals::asyncWait` (static) |

### `spawn.h` (header declarations)

23 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_SPAWN_H` | 0 | — |
| `spawnBlock_t` | 0 | — |
| `Spawn` | 0 | — |
| `Spawn::Spawn` | 0 | — |
| `Spawn::Spawn` | 0 | — |
| `Spawn::Spawn` | 0 | — |
| `Spawn::addBlock` | 0 | — |
| `Spawn::addMonster` | 0 | — |
| `Spawn::removeMonster` | 0 | — |
| `Spawn::startup` | 0 | — |
| `Spawn::startSpawnCheck` | 0 | — |
| `Spawn::stopEvent` | 0 | — |
| `Spawn::isInSpawnZone` | 0 | — |
| `Spawn::cleanup` | 0 | — |
| `Spawn::findPlayer` | 0 | — |
| `Spawn::spawnMonster` | 0 | — |
| `Spawn::spawnMonster` | 0 | — |
| `Spawn::checkSpawn` | 0 | — |
| `Spawns` | 0 | — |
| `Spawns::isInZone` | 0 | — |
| `Spawns::loadFromXml` | 0 | — |
| `Spawns::startup` | 0 | — |
| `Spawns::clear` | 0 | — |

### `spawn`

15 node(s), 21 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Spawns::loadFromXml` | 3 | `Monsters::getMonsterType` (static); `Npc::createNpc` (static); `Spawns::loadFromXml` (static) |
| `Spawns::startup` | 2 | `Game::placeCreature` (static); `Spawns::startup` (static) |
| `Spawns::clear` | 1 | `Spawns::clear` (static) |
| `Spawns::isInZone` | 1 | `Spawns::isInZone` (static) |
| `Spawn::startSpawnCheck` | 1 | `Spawn::startSpawnCheck` (static) |
| `Spawn::findPlayer` | 1 | `Spawn::findPlayer` (static) |
| `Spawn::spawnMonster` | 0 | — |
| `Spawn::spawnMonster` | 4 | `tfs::events::monster::onSpawn` (static); `Game::internalPlaceCreature` (static); `Game::placeCreature` (static); `Spawn::spawnMonster` (static) |
| `Spawn::startup` | 1 | `Spawn::startup` (static) |
| `Spawn::checkSpawn` | 1 | `Spawn::checkSpawn` (static) |
| `Spawn::cleanup` | 1 | `Spawn::cleanup` (static) |
| `Spawn::addBlock` | 1 | `Spawn::addBlock` (static) |
| `Spawn::addMonster` | 2 | `Monsters::getMonsterType` (static); `Spawn::addMonster` (static) |
| `Spawn::removeMonster` | 1 | `Spawn::removeMonster` (static) |
| `Spawn::stopEvent` | 1 | `Spawn::stopEvent` (static) |

### `spectators.h` (header declarations)

13 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `SpectatorVec` | 0 | — |
| `SpectatorVec::Vec` | 0 | — |
| `SpectatorVec::Iterator` | 0 | — |
| `SpectatorVec::ConstIterator` | 0 | — |
| `SpectatorVec::SpectatorVec` | 0 | — |
| `SpectatorVec::addSpectators` | 0 | — |
| `SpectatorVec::erase` | 0 | — |
| `SpectatorVec::size` | 0 | — |
| `SpectatorVec::empty` | 0 | — |
| `SpectatorVec::begin` | 0 | — |
| `SpectatorVec::end` | 0 | — |
| `SpectatorVec::emplace_back` | 0 | — |
| `SpectatorVec::vec` | 0 | — |

### `spells.h` (header declarations)

19 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_SPELLS_H` | 0 | — |
| `BaseSpell` | 0 | — |
| `BaseSpell::BaseSpell` | 0 | — |
| `BaseSpell::BaseSpell` | 0 | — |
| `BaseSpell::castSpell` | 0 | — |
| `BaseSpell::castSpell` | 0 | — |
| `Spell` | 0 | — |
| `Spell::Spell` | 0 | — |
| `Spell::configureSpell` | 0 | — |
| `Spell::postCastSpell` | 0 | — |
| `Spell::postCastSpell` | 0 | — |
| `Spell::getManaCost` | 0 | — |
| `Spell::isInstant` | 0 | — |
| `Spell::vocationId` | 0 | — |
| `Spell::getVocationId` | 0 | — |
| `Spell::empty` | 0 | — |
| `Spell::playerSpellCheck` | 0 | — |
| `Spell::playerInstantSpellCheck` | 0 | — |
| `Spell::playerRuneSpellCheck` | 0 | — |

### `spells`

41 node(s), 73 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Spells::playerSaySpell` | 1 | `Spells::playerSaySpell` (static) |
| `Spells::clearMaps` | 1 | `Spells::clearMaps` (static) |
| `Spells::clear` | 1 | `Spells::clear` (static) |
| `Spells::getScriptInterface` | 1 | `Spells::getScriptInterface` (static) |
| `Spells::getEvent` | 1 | `Spells::getEvent` (static) |
| `Spells::registerEvent` | 1 | `Spells::registerEvent` (static) |
| `Spells::registerInstantLuaEvent` | 1 | `Spells::registerInstantLuaEvent` (static) |
| `Spells::registerRuneLuaEvent` | 1 | `Spells::registerRuneLuaEvent` (static) |
| `Spells::getSpellByName` | 1 | `Spells::getSpellByName` (static) |
| `Spells::getRuneSpell` | 1 | `Spells::getRuneSpell` (static) |
| `Spells::getRuneSpellByName` | 1 | `Spells::getRuneSpellByName` (static) |
| `Spells::getInstantSpell` | 1 | `Spells::getInstantSpell` (static) |
| `Spells::getInstantSpellByName` | 1 | `Spells::getInstantSpellByName` (static) |
| `Spells::getCasterPosition` | 1 | `Spells::getCasterPosition` (static) |
| `CombatSpell::loadScriptCombat` | 2 | `LuaEnvironment::getCombatObject` (static); `CombatSpell::loadScriptCombat` (static) |
| `CombatSpell::castSpell` | 0 | — |
| `CombatSpell::castSpell` | 2 | `CombatSpell::castSpell` (static); `Spells::getCasterPosition` (static) |
| `CombatSpell::executeCastSpell` | 6 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushVariant` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `Spell::configureSpell` | 1 | `Spell::configureSpell` (static) |
| `Spell::playerSpellCheck` | 3 | `tfs::events::player::onSpellCheck` (static); `Game::addMagicEffect` (static); `Spell::playerSpellCheck` (static) |
| `Spell::playerInstantSpellCheck` | 2 | `Game::addMagicEffect` (static); `Spell::playerInstantSpellCheck` (static) |
| `Spell::playerRuneSpellCheck` | 5 | `Combat::canDoCombat` (static); `Combat::isInPvpZone` (static); `Game::addMagicEffect` (static); `Game::canThrowObjectTo` (static); `Spell::playerRuneSpellCheck` (static) |
| `Spell::postCastSpell` | 0 | — |
| `Spell::postCastSpell` | 1 | `Spell::postCastSpell` (static) |
| `Spell::getManaCost` | 1 | `Spell::getManaCost` (static) |
| `InstantSpell::configureEvent` | 3 | `InstantSpell::configureEvent` (static); `Spell::configureSpell` (static); `TalkAction::configureEvent` (static) |
| `InstantSpell::playerCastInstant` | 5 | `Condition::createCondition` (static); `Game::addMagicEffect` (static); `Game::getPlayerByNameWildcard` (static); `InstantSpell::playerCastInstant` (static); `Spells::getCasterPosition` (static) |
| `InstantSpell::canThrowSpell` | 2 | `Game::canThrowObjectTo` (static); `InstantSpell::canThrowSpell` (static) |
| `InstantSpell::castSpell` | 0 | — |
| `InstantSpell::castSpell` | 1 | `InstantSpell::castSpell` (static) |
| `InstantSpell::internalCastSpell` | 1 | `InstantSpell::internalCastSpell` (static) |
| `InstantSpell::executeCastSpell` | 6 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushVariant` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `InstantSpell::canCast` | 1 | `InstantSpell::canCast` (static) |
| `RuneSpell::configureEvent` | 2 | `RuneSpell::configureEvent` (static); `Spell::configureSpell` (static) |
| `RuneSpell::canExecuteAction` | 2 | `Action::canExecuteAction` (static); `RuneSpell::canExecuteAction` (static) |
| `RuneSpell::executeUse` | 3 | `Game::getCreatureByID` (static); `Game::transformItem` (static); `RuneSpell::executeUse` (static) |
| `RuneSpell::castSpell` | 0 | — |
| `RuneSpell::castSpell` | 1 | `RuneSpell::castSpell` (static) |
| `RuneSpell::internalCastSpell` | 1 | `RuneSpell::internalCastSpell` (static) |
| `RuneSpell::executeCastSpell` | 7 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushBoolean` (static); `tfs::lua::pushVariant` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setCreatureMetatable` (static); … +2 more |
| `RuneSpell::canUse` | 1 | `RuneSpell::canUse` (static) |

### `storeinbox.h` (header declarations)

5 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `StoreInbox` | 0 | — |
| `StoreInbox::canRemove` | 0 | — |
| `StoreInbox::getStoreInbox (const)` | 0 | — |
| `StoreInbox::getStoreInbox (non-const)` | 0 | — |
| `FS_STOREINBOX_H` | 0 | — |

### `storeinbox`

7 node(s), 4 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `StoreInbox::StoreInbox` | 1 | `StoreInbox::StoreInbox` (static) |
| `StoreInbox::postAddNotification` | 0 | — |
| `StoreInbox::postRemoveNotification` | 0 | — |
| `StoreInbox::queryAdd` | 0 | — |
| `StoreInbox::queryAdd` | 1 | `StoreInbox::queryAdd` (static) |
| `StoreInbox::postAddNotification` | 1 | `StoreInbox::postAddNotification` (static) |
| `StoreInbox::postRemoveNotification` | 1 | `StoreInbox::postRemoveNotification` (static) |

### `talkaction.h` (header declarations)

15 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_TALKACTION_H` | 0 | — |
| `TalkActionResult_t` | 0 | — |
| `TalkActionResult_t::TALKACTION_CONTINUE` | 0 | — |
| `TalkActionResult_t::TALKACTION_BREAK` | 0 | — |
| `TalkActionResult_t::TALKACTION_FAILED` | 0 | — |
| `TalkAction` | 0 | — |
| `TalkAction::TalkAction` | 0 | — |
| `TalkAction::configureEvent` | 0 | — |
| `TalkAction::emplace_back` | 0 | — |
| `TalkAction::executeSay` | 0 | — |
| `TalkAction::TalkActions` | 0 | — |
| `TalkAction::~TalkAction` | 0 | — |
| `TalkAction::TalkActions` | 0 | — |
| `TalkAction::playerSaySpell` | 0 | — |
| `TalkAction::registerLuaEvent` | 0 | — |

### `talkaction`

8 node(s), 14 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `TalkActions::clear` | 1 | `TalkActions::clear` (static) |
| `TalkActions::getScriptInterface` | 1 | `TalkActions::getScriptInterface` (static) |
| `TalkActions::getEvent` | 1 | `TalkActions::getEvent` (static) |
| `TalkActions::registerEvent` | 1 | `TalkActions::registerEvent` (static) |
| `TalkActions::registerLuaEvent` | 1 | `TalkActions::registerLuaEvent` (static) |
| `TalkActions::playerSaySpell` | 1 | `TalkActions::playerSaySpell` (static) |
| `TalkAction::configureEvent` | 1 | `TalkAction::configureEvent` (static) |
| `TalkAction::executeSay` | 7 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushString` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +2 more |

### `tasks.h` (header declarations)

10 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_TASKS_H` | 0 | — |
| `Task` | 0 | — |
| `Task::Task` | 0 | — |
| `Task::Task` | 0 | — |
| `Task::false` | 0 | — |
| `Task::now` | 0 | — |
| `Dispatcher` | 0 | — |
| `Dispatcher::addTask` | 0 | — |
| `Dispatcher::shutdown` | 0 | — |
| `Dispatcher::threadMain` | 0 | — |

### `tasks`

5 node(s), 3 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `createTask` | 0 | — |
| `createTask` | 0 | — |
| `Dispatcher::threadMain` | 1 | `Dispatcher::threadMain` (static) |
| `Dispatcher::addTask` | 1 | `Dispatcher::addTask` (static) |
| `Dispatcher::shutdown` | 1 | `Dispatcher::shutdown` (static) |

### `teleport.h` (header declarations)

13 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Teleport` | 0 | — |
| `Teleport::Teleport` | 0 | — |
| `Teleport::destPos` | 0 | — |
| `Teleport::addThing (Thing*)` | 0 | — |
| `Teleport::getDestPos` | 0 | — |
| `Teleport::getReceiver (const)` | 0 | — |
| `Teleport::getReceiver (non-const)` | 0 | — |
| `Teleport::getTeleport (const)` | 0 | — |
| `Teleport::getTeleport (non-const)` | 0 | — |
| `Teleport::queryDestination` | 0 | — |
| `Teleport::queryRemove` | 0 | — |
| `Teleport::setDestPos` | 0 | — |
| `FS_TELEPORT_H` | 0 | — |

### `teleport`

10 node(s), 10 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Teleport::addThing (int32_t, Thing*)` | 0 | — |
| `Teleport::postAddNotification` | 0 | — |
| `Teleport::postRemoveNotification` | 0 | — |
| `Teleport::readAttr` | 0 | — |
| `Teleport::serializeAttr` | 0 | — |
| `Teleport::readAttr` | 2 | `Item::readAttr` (static); `Teleport::readAttr` (static) |
| `Teleport::serializeAttr` | 2 | `Item::serializeAttr` (static); `Teleport::serializeAttr` (static) |
| `Teleport::addThing` | 4 | `Game::addMagicEffect` (static); `Game::internalCreatureTurn` (static); `Game::internalMoveItem` (static); `Teleport::addThing` (static) |
| `Teleport::postAddNotification` | 1 | `Teleport::postAddNotification` (static) |
| `Teleport::postRemoveNotification` | 1 | `Teleport::postRemoveNotification` (static) |

### `thing.h` (header declarations)

51 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `INDEX_WHEREEVER` | 0 | — |
| `ReceiverFlag_t` | 0 | — |
| `ReceiverFlag_t::FLAG_NOLIMIT` | 0 | — |
| `ReceiverFlag_t::FLAG_IGNOREBLOCKITEM` | 0 | — |
| `ReceiverFlag_t::FLAG_IGNOREBLOCKCREATURE` | 0 | — |
| `ReceiverFlag_t::FLAG_CHILDISOWNER` | 0 | — |
| `ReceiverFlag_t::FLAG_PATHFINDING` | 0 | — |
| `ReceiverFlag_t::FLAG_IGNOREFIELDDAMAGE` | 0 | — |
| `ReceiverFlag_t::FLAG_IGNORENOTMOVEABLE` | 0 | — |
| `ReceiverFlag_t::FLAG_IGNOREAUTOSTACK` | 0 | — |
| `ReceiverLink_t` | 0 | — |
| `ReceiverLink_t::LINK_OWNER` | 0 | — |
| `ReceiverLink_t::LINK_PARENT` | 0 | — |
| `ReceiverLink_t::LINK_TOPPARENT` | 0 | — |
| `ReceiverLink_t::LINK_NEAR` | 0 | — |
| `Thing` | 0 | — |
| `Thing::Thing` | 0 | — |
| `Thing::~Thing` | 0 | — |
| `Thing::hasParent` | 0 | — |
| `Thing::getParent` | 0 | — |
| `Thing::getRealParent` | 0 | — |
| `Thing::setParent` | 0 | — |
| `Thing::getPosition` | 0 | — |
| `Thing::getThrowRange` | 0 | — |
| `Thing::isPushable` | 0 | — |
| `Thing::getTile` | 0 | — |
| `Thing::getItem` | 0 | — |
| `Thing::getCreature` | 0 | — |
| `Thing::isRemoved` | 0 | — |
| `Thing::getReceiver` | 0 | — |
| `Thing::queryAdd` | 0 | — |
| `Thing::queryMaxCount` | 0 | — |
| `Thing::queryRemove` | 0 | — |
| `Thing::queryDestination` | 0 | — |
| `Thing::addThing(Thing*)` | 0 | — |
| `Thing::addThing(int32_t,Thing*)` | 0 | — |
| `Thing::updateThing` | 0 | — |
| `Thing::replaceThing` | 0 | — |
| `Thing::removeThing` | 0 | — |
| `Thing::postAddNotification` | 0 | — |
| `Thing::postRemoveNotification` | 0 | — |
| `Thing::getThingIndex` | 0 | — |
| `Thing::getFirstIndex` | 0 | — |
| `Thing::getLastIndex` | 0 | — |
| `Thing::getThing` | 0 | — |
| `Thing::getItemTypeCount` | 0 | — |
| `Thing::getAllItemTypeCount` | 0 | — |
| `Thing::internalRemoveThing` | 0 | — |
| `Thing::internalAddThing(Thing*)` | 0 | — |
| `Thing::internalAddThing(uint32_t,Thing*)` | 0 | — |
| `Thing::parent` | 0 | — |

### `thing`

1 node(s), 1 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Thing::getPosition` | 1 | `Thing::getPosition` (static) |

### `thread_holder_base.h` (header declarations)

6 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_THREAD_HOLDER_BASE_H` | 0 | — |
| `ThreadHolder` | 0 | — |
| `ThreadHolder::ThreadHolder` | 0 | — |
| `ThreadHolder::setState` | 0 | — |
| `ThreadHolder::thread` | 0 | — |
| `ThreadHolder::join` | 0 | — |

### `tile.h` (header declarations)

47 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `CreatureVector` | 0 | — |
| `ItemVector` | 0 | — |
| `tileflags_t` | 0 | — |
| `tileflags_t::TILESTATE_NONE` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE_DOWN` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE_NORTH` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE_SOUTH` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE_EAST` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE_WEST` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE_SOUTH_ALT` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE_EAST_ALT` | 0 | — |
| `tileflags_t::TILESTATE_PROTECTIONZONE` | 0 | — |
| `tileflags_t::TILESTATE_NOPVPZONE` | 0 | — |
| `tileflags_t::TILESTATE_NOLOGOUT` | 0 | — |
| `tileflags_t::TILESTATE_PVPZONE` | 0 | — |
| `tileflags_t::TILESTATE_TELEPORT` | 0 | — |
| `tileflags_t::TILESTATE_MAGICFIELD` | 0 | — |
| `tileflags_t::TILESTATE_MAILBOX` | 0 | — |
| `tileflags_t::TILESTATE_TRASHHOLDER` | 0 | — |
| `tileflags_t::TILESTATE_BED` | 0 | — |
| `tileflags_t::TILESTATE_DEPOT` | 0 | — |
| `tileflags_t::TILESTATE_BLOCKSOLID` | 0 | — |
| `tileflags_t::TILESTATE_BLOCKPATH` | 0 | — |
| `tileflags_t::TILESTATE_IMMOVABLEBLOCKSOLID` | 0 | — |
| `tileflags_t::TILESTATE_IMMOVABLEBLOCKPATH` | 0 | — |
| `tileflags_t::TILESTATE_IMMOVABLENOFIELDBLOCKPATH` | 0 | — |
| `tileflags_t::TILESTATE_NOFIELDBLOCKPATH` | 0 | — |
| `tileflags_t::TILESTATE_SUPPORTS_HANGABLE` | 0 | — |
| `tileflags_t::TILESTATE_FLOORCHANGE` | 0 | — |
| `ZoneType_t` | 0 | — |
| `ZoneType_t::ZONE_PROTECTION` | 0 | — |
| `ZoneType_t::ZONE_NOPVP` | 0 | — |
| `ZoneType_t::ZONE_PVP` | 0 | — |
| `ZoneType_t::ZONE_NOLOGOUT` | 0 | — |
| `ZoneType_t::ZONE_NORMAL` | 0 | — |
| `TileItemVector` | 0 | — |
| `TileItemVector::getBeginDownItem` | 0 | — |
| `TileItemVector::getEndDownItem` | 0 | — |
| `TileItemVector::getBeginTopItem` | 0 | — |
| `TileItemVector::getEndTopItem` | 0 | — |
| `TileItemVector::getTopItemCount` | 0 | — |
| `TileItemVector::getDownItemCount` | 0 | — |
| `TileItemVector::getTopTopItem` | 0 | — |
| `TileItemVector::getTopDownItem` | 0 | — |
| `TileItemVector::addDownItemCount` | 0 | — |
| `TileItemVector::downItemCount` | 0 | — |
| `TILE_UPDATE_THRESHOLD` | 0 | — |

### `tile`

45 node(s), 51 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Tile::hasProperty` | 0 | — |
| `Tile::hasProperty` | 1 | `Tile::hasProperty` (static) |
| `Tile::hasHeight` | 1 | `Tile::hasHeight` (static) |
| `Tile::getCreatureCount` | 1 | `Tile::getCreatureCount` (static) |
| `Tile::getItemCount` | 1 | `Tile::getItemCount` (static) |
| `Tile::getTopItemCount` | 1 | `Tile::getTopItemCount` (static) |
| `Tile::getDownItemCount` | 1 | `Tile::getDownItemCount` (static) |
| `Tile::getTeleportItem` | 1 | `Tile::getTeleportItem` (static) |
| `Tile::getFieldItem` | 1 | `Tile::getFieldItem` (static) |
| `Tile::getTrashHolder` | 1 | `Tile::getTrashHolder` (static) |
| `Tile::getMailbox` | 1 | `Tile::getMailbox` (static) |
| `Tile::getBedItem` | 1 | `Tile::getBedItem` (static) |
| `Tile::getTopCreature` | 1 | `Tile::getTopCreature` (static) |
| `Tile::getBottomCreature` | 1 | `Tile::getBottomCreature` (static) |
| `Tile::getTopVisibleCreature` | 1 | `Tile::getTopVisibleCreature` (static) |
| `Tile::getBottomVisibleCreature` | 1 | `Tile::getBottomVisibleCreature` (static) |
| `Tile::getTopDownItem` | 1 | `Tile::getTopDownItem` (static) |
| `Tile::getTopTopItem` | 1 | `Tile::getTopTopItem` (static) |
| `Tile::getItemByTopOrder` | 1 | `Tile::getItemByTopOrder` (static) |
| `Tile::getTopVisibleThing` | 1 | `Tile::getTopVisibleThing` (static) |
| `Tile::onAddTileItem` | 1 | `Tile::onAddTileItem` (static) |
| `Tile::onUpdateTileItem` | 1 | `Tile::onUpdateTileItem` (static) |
| `Tile::onRemoveTileItem` | 1 | `Tile::onRemoveTileItem` (static) |
| `Tile::queryAdd` | 1 | `Tile::queryAdd` (static) |
| `Tile::queryMaxCount` | 1 | `Tile::queryMaxCount` (static) |
| `Tile::queryRemove` | 1 | `Tile::queryRemove` (static) |
| `Tile::queryDestination` | 1 | `Tile::queryDestination` (static) |
| `Tile::addThing` | 2 | `Game::ReleaseItem` (static); `Tile::addThing` (static) |
| `Tile::updateThing` | 1 | `Tile::updateThing` (static) |
| `Tile::replaceThing` | 1 | `Tile::replaceThing` (static) |
| `Tile::removeThing` | 1 | `Tile::removeThing` (static) |
| `Tile::hasCreature` | 1 | `Tile::hasCreature` (static) |
| `Tile::removeCreature` | 1 | `Tile::removeCreature` (static) |
| `Tile::getThingIndex` | 1 | `Tile::getThingIndex` (static) |
| `Tile::getClientIndexOfCreature` | 1 | `Tile::getClientIndexOfCreature` (static) |
| `Tile::getStackposOfItem` | 1 | `Tile::getStackposOfItem` (static) |
| `Tile::getItemTypeCount` | 1 | `Tile::getItemTypeCount` (static) |
| `Tile::getThing` | 1 | `Tile::getThing` (static) |
| `Tile::postAddNotification` | 5 | `Game::ReleaseCreature` (static); `Game::ReleaseItem` (static); `MoveEvents::onCreatureMove` (static); `MoveEvents::onItemMove` (static); `Tile::postAddNotification` (static) |
| `Tile::postRemoveNotification` | 3 | `MoveEvents::onCreatureMove` (static); `MoveEvents::onItemMove` (static); `Tile::postRemoveNotification` (static) |
| `Tile::internalAddThing` | 1 | `Tile::internalAddThing` (static) |
| `Tile::setTileFlags` | 1 | `Tile::setTileFlags` (static) |
| `Tile::resetTileFlags` | 1 | `Tile::resetTileFlags` (static) |
| `Tile::isMoveableBlocking` | 1 | `Tile::isMoveableBlocking` (static) |
| `Tile::getUseItem` | 1 | `Tile::getUseItem` (static) |

### `tools.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_TOOLS_H` | 0 | — |

### `tools`

39 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `printXMLError` | 0 | — |
| `transformToSHA1` | 0 | — |
| `hmac` | 0 | — |
| `generateToken` | 0 | — |
| `caseInsensitiveEqual` | 0 | — |
| `caseInsensitiveStartsWith` | 0 | — |
| `explodeString` | 0 | — |
| `vectorAtoi` | 0 | — |
| `getRandomGenerator` | 0 | — |
| `uniform_random` | 0 | — |
| `normal_random` | 0 | — |
| `boolean_random` | 0 | — |
| `randomBytes` | 0 | — |
| `formatDateShort` | 0 | — |
| `getNextPosition` | 0 | — |
| `getDirectionTo` | 0 | — |
| `getDepotBoxId` | 0 | — |
| `getMagicEffect` | 0 | — |
| `getShootType` | 0 | — |
| `getCombatName` | 0 | — |
| `getAmmoType` | 0 | — |
| `getWeaponAction` | 0 | — |
| `getSkullType` | 0 | — |
| `getSpecialSkillName` | 0 | — |
| `getSkillName` | 0 | — |
| `adlerChecksum` | 0 | — |
| `ucfirst` | 0 | — |
| `ucwords` | 0 | — |
| `booleanString` | 0 | — |
| `combatTypeToIndex` | 0 | — |
| `indexToCombatType` | 0 | — |
| `serverFluidToClient` | 0 | — |
| `clientFluidToServer` | 0 | — |
| `stringToItemAttribute` | 0 | — |
| `getFirstLine` | 0 | — |
| `getReturnMessage` | 0 | — |
| `OTSYS_TIME` | 0 | — |
| `stringToSpellGroup` | 0 | — |
| `getShuffleDirections` | 0 | — |

### `town.h` (header declarations)

17 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Towns` | 0 | — |
| `Town::id` | 0 | — |
| `Town::name` | 0 | — |
| `Town::templePosition` | 0 | — |
| `Towns::townMap` | 0 | — |
| `Towns::getTown` | 0 | — |
| `Towns::getTowns` | 0 | — |
| `Towns::setTown` | 0 | — |
| `Town` | 0 | — |
| `TownMap` | 0 | — |
| `FS_TOWN_H` | 0 | — |
| `Town` | 0 | — |
| `Towns` | 0 | — |
| `Towns::it` | 0 | — |
| `Towns::find` | 0 | — |
| `Towns::nullptr` | 0 | — |
| `Towns::get` | 0 | — |

### `trashholder.h` (header declarations)

10 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `TrashHolder` | 0 | — |
| `TrashHolder::TrashHolder` | 0 | — |
| `TrashHolder::addThing (Thing*)` | 0 | — |
| `TrashHolder::getReceiver (const)` | 0 | — |
| `TrashHolder::getReceiver (non-const)` | 0 | — |
| `TrashHolder::getTrashHolder (const)` | 0 | — |
| `TrashHolder::getTrashHolder (non-const)` | 0 | — |
| `TrashHolder::queryAdd` | 0 | — |
| `TrashHolder::queryDestination` | 0 | — |
| `FS_TRASHHOLDER_H` | 0 | — |

### `trashholder`

8 node(s), 6 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `TrashHolder::addThing (int32_t, Thing*)` | 0 | — |
| `TrashHolder::postAddNotification` | 0 | — |
| `TrashHolder::postRemoveNotification` | 0 | — |
| `TrashHolder::queryMaxCount` | 0 | — |
| `TrashHolder::queryMaxCount` | 1 | `TrashHolder::queryMaxCount` (static) |
| `TrashHolder::addThing` | 3 | `Game::addMagicEffect` (static); `Game::internalRemoveItem` (static); `TrashHolder::addThing` (static) |
| `TrashHolder::postAddNotification` | 1 | `TrashHolder::postAddNotification` (static) |
| `TrashHolder::postRemoveNotification` | 1 | `TrashHolder::postRemoveNotification` (static) |

### `vocation.h` (header declarations)

57 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Vocation` | 0 | — |
| `Vocation::Vocation` | 0 | — |
| `Vocation::getVocName` | 0 | — |
| `Vocation::getVocDescription` | 0 | — |
| `Vocation::getReqSkillTries` | 0 | — |
| `Vocation::getReqMana` | 0 | — |
| `Vocation::getId` | 0 | — |
| `Vocation::getClientId` | 0 | — |
| `Vocation::getHPGain` | 0 | — |
| `Vocation::getManaGain` | 0 | — |
| `Vocation::getCapGain` | 0 | — |
| `Vocation::getManaGainTicks` | 0 | — |
| `Vocation::getManaGainAmount` | 0 | — |
| `Vocation::getHealthGainTicks` | 0 | — |
| `Vocation::getHealthGainAmount` | 0 | — |
| `Vocation::getSoulMax` | 0 | — |
| `Vocation::getSoulGainTicks` | 0 | — |
| `Vocation::getAttackSpeed` | 0 | — |
| `Vocation::getBaseSpeed` | 0 | — |
| `Vocation::getFromVocation` | 0 | — |
| `Vocation::getNoPongKickTime` | 0 | — |
| `Vocation::allowsPvp` | 0 | — |
| `Vocation::getMagicShield` | 0 | — |
| `Vocation::meleeDamageMultiplier` | 0 | — |
| `Vocation::distDamageMultiplier` | 0 | — |
| `Vocation::defenseMultiplier` | 0 | — |
| `Vocation::armorMultiplier` | 0 | — |
| `Vocation::friend_Vocations` | 0 | — |
| `Vocation::name` | 0 | — |
| `Vocation::description` | 0 | — |
| `Vocation::skillMultipliers` | 0 | — |
| `Vocation::manaMultiplier` | 0 | — |
| `Vocation::gainHealthTicks` | 0 | — |
| `Vocation::gainHealthAmount` | 0 | — |
| `Vocation::gainManaTicks` | 0 | — |
| `Vocation::gainManaAmount` | 0 | — |
| `Vocation::gainCap` | 0 | — |
| `Vocation::gainMana` | 0 | — |
| `Vocation::gainHP` | 0 | — |
| `Vocation::fromVocation` | 0 | — |
| `Vocation::attackSpeed` | 0 | — |
| `Vocation::baseSpeed` | 0 | — |
| `Vocation::noPongKickTime` | 0 | — |
| `Vocation::id` | 0 | — |
| `Vocation::gainSoulTicks` | 0 | — |
| `Vocation::soulMax` | 0 | — |
| `Vocation::clientId` | 0 | — |
| `Vocation::allowPvp` | 0 | — |
| `Vocation::magicShield` | 0 | — |
| `VocationMap` | 0 | — |
| `Vocations` | 0 | — |
| `Vocations::loadFromXml` | 0 | — |
| `Vocations::getVocation` | 0 | — |
| `Vocations::getVocationId` | 0 | — |
| `Vocations::getPromotedVocation` | 0 | — |
| `Vocations::getVocations` | 0 | — |
| `Vocations::vocationsMap` | 0 | — |

### `vocation`

7 node(s), 6 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Vocations::loadFromXml` | 1 | `Vocations::loadFromXml` (static) |
| `Vocations::getVocation` | 1 | `Vocations::getVocation` (static) |
| `Vocations::getVocationId` | 1 | `Vocations::getVocationId` (static) |
| `Vocations::getPromotedVocation` | 1 | `Vocations::getPromotedVocation` (static) |
| `skillBase` | 0 | — |
| `Vocation::getReqSkillTries` | 1 | `Vocation::getReqSkillTries` (static) |
| `Vocation::getReqMana` | 1 | `Vocation::getReqMana` (static) |

### `weapons.h` (header declarations)

22 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_WEAPONS_H` | 0 | — |
| `Weapon` | 0 | — |
| `Weapon::Weapon` | 0 | — |
| `Weapon::configureEvent` | 0 | — |
| `Weapon::configureWeapon` | 0 | — |
| `Weapon::playerWeaponCheck` | 0 | — |
| `Weapon::ammoCheck` | 0 | — |
| `Weapon::useFist` | 0 | — |
| `Weapon::useWeapon` | 0 | — |
| `Weapon::getElementDamage` | 0 | — |
| `Weapon::getElementType` | 0 | — |
| `Weapon::vocationId` | 0 | — |
| `Weapon::getVocationId` | 0 | — |
| `Weapon::insert` | 0 | — |
| `Weapon::empty` | 0 | — |
| `Weapon::internalUseWeapon` | 0 | — |
| `Weapon::internalUseWeapon` | 0 | — |
| `Weapon::getManaCost` | 0 | — |
| `Weapon::getHealthCost` | 0 | — |
| `Weapon::executeUseWeapon` | 0 | — |
| `Weapon::onUsedWeapon` | 0 | — |
| `Weapon::decrementItemCount` | 0 | — |

### `weapons`

35 node(s), 59 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `Weapons::getWeapon` | 1 | `Weapons::getWeapon` (static) |
| `Weapons::clear` | 1 | `Weapons::clear` (static) |
| `Weapons::getScriptInterface` | 1 | `Weapons::getScriptInterface` (static) |
| `Weapons::loadDefaults` | 1 | `Weapons::loadDefaults` (static) |
| `Weapons::getEvent` | 1 | `Weapons::getEvent` (static) |
| `Weapons::registerEvent` | 1 | `Weapons::registerEvent` (static) |
| `Weapons::registerLuaEvent` | 1 | `Weapons::registerLuaEvent` (static) |
| `Weapons::getMaxMeleeDamage` | 1 | `Weapons::getMaxMeleeDamage` (static) |
| `Weapons::getMaxWeaponDamage` | 1 | `Weapons::getMaxWeaponDamage` (static) |
| `Weapon::configureEvent` | 3 | `Vocations::getPromotedVocation` (static); `Vocations::getVocationId` (static); `Weapon::configureEvent` (static) |
| `Weapon::configureWeapon` | 1 | `Weapon::configureWeapon` (static) |
| `Weapon::playerWeaponCheck` | 1 | `Weapon::playerWeaponCheck` (static) |
| `Weapon::ammoCheck` | 1 | `Weapon::ammoCheck` (static) |
| `Weapon::useWeapon` | 1 | `Weapon::useWeapon` (static) |
| `Weapon::useFist` | 3 | `Combat::doTargetCombat` (static); `Weapon::useFist` (static); `Weapons::getMaxWeaponDamage` (static) |
| `Weapon::internalUseWeapon` | 0 | — |
| `Weapon::internalUseWeapon` | 3 | `Combat::postCombatEffects` (static); `Game::addMagicEffect` (static); `Weapon::internalUseWeapon` (static) |
| `Weapon::onUsedWeapon` | 4 | `Game::internalMoveItem` (static); `Game::transformItem` (static); `Weapon::decrementItemCount` (static); `Weapon::onUsedWeapon` (static) |
| `Weapon::getManaCost` | 1 | `Weapon::getManaCost` (static) |
| `Weapon::getHealthCost` | 1 | `Weapon::getHealthCost` (static) |
| `Weapon::executeUseWeapon` | 6 | `tfs::lua::getScriptEnv` (static); `tfs::lua::pushVariant` (static); `tfs::lua::reserveScriptEnv` (static); `tfs::lua::setMetatable` (static); `tfs::lua::pushUserdata` (static); … +1 more |
| `Weapon::decrementItemCount` | 3 | `Game::internalRemoveItem` (static); `Game::transformItem` (static); `Weapon::decrementItemCount` (static) |
| `WeaponMelee::configureWeapon` | 2 | `Weapon::configureWeapon` (static); `WeaponMelee::configureWeapon` (static) |
| `WeaponMelee::useWeapon` | 1 | `WeaponMelee::useWeapon` (static) |
| `WeaponMelee::getSkillType` | 1 | `WeaponMelee::getSkillType` (static) |
| `WeaponMelee::getElementDamage` | 2 | `WeaponMelee::getElementDamage` (static); `Weapons::getMaxWeaponDamage` (static) |
| `WeaponMelee::getWeaponDamage` | 2 | `WeaponMelee::getWeaponDamage` (static); `Weapons::getMaxWeaponDamage` (static) |
| `WeaponDistance::configureWeapon` | 2 | `Weapon::configureWeapon` (static); `WeaponDistance::configureWeapon` (static) |
| `WeaponDistance::useWeapon` | 2 | `WeaponDistance::useWeapon` (static); `Weapons::getWeapon` (static) |
| `WeaponDistance::getElementDamage` | 2 | `WeaponDistance::getElementDamage` (static); `Weapons::getMaxWeaponDamage` (static) |
| `WeaponDistance::getWeaponDamage` | 2 | `WeaponDistance::getWeaponDamage` (static); `Weapons::getMaxWeaponDamage` (static) |
| `WeaponDistance::getSkillType` | 1 | `WeaponDistance::getSkillType` (static) |
| `WeaponWand::configureEvent` | 2 | `Weapon::configureEvent` (static); `WeaponWand::configureEvent` (static) |
| `WeaponWand::configureWeapon` | 2 | `Weapon::configureWeapon` (static); `WeaponWand::configureWeapon` (static) |
| `WeaponWand::getWeaponDamage` | 1 | `WeaponWand::getWeaponDamage` (static) |

### `wildcardtree.h` (header declarations)

11 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_WILDCARDTREE_H` | 0 | — |
| `WildcardTreeNode` | 0 | — |
| `WildcardTreeNode::WildcardTreeNode` | 0 | — |
| `WildcardTreeNode::WildcardTreeNode` | 0 | — |
| `WildcardTreeNode::WildcardTreeNode` | 0 | — |
| `WildcardTreeNode::getChild` | 0 | — |
| `WildcardTreeNode::getChild` | 0 | — |
| `WildcardTreeNode::addChild` | 0 | — |
| `WildcardTreeNode::insert` | 0 | — |
| `WildcardTreeNode::remove` | 0 | — |
| `WildcardTreeNode::findOne` | 0 | — |

### `wildcardtree`

6 node(s), 5 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `WildcardTreeNode::getChild` | 0 | — |
| `WildcardTreeNode::getChild` | 1 | `WildcardTreeNode::getChild` (static) |
| `WildcardTreeNode::addChild` | 1 | `WildcardTreeNode::addChild` (static) |
| `WildcardTreeNode::insert` | 1 | `WildcardTreeNode::insert` (static) |
| `WildcardTreeNode::remove` | 1 | `WildcardTreeNode::remove` (static) |
| `WildcardTreeNode::findOne` | 1 | `WildcardTreeNode::findOne` (static) |

### `xtea.h` (header declarations)

1 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `FS_XTEA_H` | 0 | — |

### `xtea`

3 node(s), 0 edge(s) total

| Node | Edges | Edge detail |
|---|---|---|
| `expand_key` | 0 | — |
| `encrypt` | 0 | — |
| `decrypt` | 0 | — |

## Statistics

| Metric | Value |
|---|---|
| Shards | 171 |
| Total nodes | 6194 |
| Total edges | 2959 |

