## ADDED Requirements

### Requirement: parseAutoWalk decodes a walk direction sequence
`parseAutoWalk` SHALL decode a list of movement directions from the message and dispatch `Game::playerMove` for each step.

#### Scenario: walk sequence is decoded correctly
- **WHEN** a message contains direction bytes [6, 4, 5] (north, east, south)
- **THEN** three `Game::playerMove` calls are dispatched in order

### Requirement: parseSay decodes chat message fields
`parseSay` SHALL decode type, receiver name (for private messages), channel ID (for channel messages), and message text, then dispatch to the game.

#### Scenario: private message fields are decoded
- **WHEN** a say packet with type PRIVATE contains receiver "Bob" and text "hi"
- **THEN** the dispatched action has receiver "Bob" and text "hi"

### Requirement: parseUseItem decodes item position, item ID, and use index
`parseUseItem` SHALL read position, item ID (2 bytes), and stack index, then dispatch `Game::playerUseItem`.

#### Scenario: use item payload decoded correctly
- **WHEN** a use-item packet contains position (100,200,7), itemId 2160, stackpos 0
- **THEN** the dispatched action contains those exact values

### Requirement: parseLookAt decodes position and stack position
`parseLookAt` SHALL read a position and stack index from the message and dispatch a look-at action to the game.

#### Scenario: look-at coordinates are decoded correctly
- **WHEN** a look-at packet contains position (100,200,7) and stackpos 1
- **THEN** the dispatched look-at action contains (100,200,7) stackpos 1

### Requirement: parseAttack decodes creature ID
`parseAttack` SHALL read a 4-byte creature ID and dispatch `Game::playerSetAttackedCreature`.

#### Scenario: attack target is decoded correctly
- **WHEN** an attack packet contains creature ID 12345
- **THEN** `Game::playerSetAttackedCreature` is dispatched with ID 12345

### Requirement: parseFollow decodes creature ID for follow
`parseFollow` SHALL read a 4-byte creature ID and dispatch a follow action.

#### Scenario: follow target decoded correctly
- **WHEN** a follow packet contains creature ID 99
- **THEN** the follow action is dispatched with ID 99

### Requirement: parseFightModes decodes all three mode bytes
`parseFightModes` SHALL read attack mode, chase mode, and secure mode bytes and dispatch the combined fight-modes update.

#### Scenario: fight mode bytes decoded correctly
- **WHEN** a fight-modes packet contains attack=1, chase=1, secure=0
- **THEN** the dispatched fight-mode action contains those exact values

### Requirement: parseThrow decodes from-position, to-position, and item fields
`parseThrow` SHALL read two positions, item ID, stack index, and count, then dispatch a throw/move action.

#### Scenario: throw fields decoded correctly
- **WHEN** a throw packet contains from (1,2,7), to (3,4,7), itemId 100, stackpos 0, count 1
- **THEN** the dispatched action contains all six fields

### Requirement: parseCloseContainer decodes container ID
`parseCloseContainer` SHALL read a 1-byte container ID and dispatch `Game::playerCloseContainer`.

#### Scenario: container ID decoded correctly
- **WHEN** a close-container packet contains ID 5
- **THEN** the dispatched action has container ID 5

### Requirement: parseUpArrowContainer decodes container ID
`parseUpArrowContainer` SHALL read a 1-byte container ID and dispatch a container-up-arrow action.

#### Scenario: container ID decoded correctly
- **WHEN** an up-arrow-container packet contains ID 3
- **THEN** the dispatched action has container ID 3

### Requirement: parseSeekInContainer decodes container ID and index
`parseSeekInContainer` SHALL read a 1-byte container ID and 2-byte seek index.

#### Scenario: seek-in-container fields decoded correctly
- **WHEN** the packet contains container ID 2 and index 10
- **THEN** the dispatched action has container ID 2 and index 10

### Requirement: parseLookInBattleList decodes creature ID
`parseLookInBattleList` SHALL read a 4-byte creature ID and dispatch a look-in-battle-list action.

#### Scenario: creature ID decoded correctly
- **WHEN** the packet contains creature ID 777
- **THEN** the dispatched action contains ID 777

### Requirement: parseOpenChannel decodes channel ID
`parseOpenChannel` SHALL read a 2-byte channel ID and dispatch `Game::playerOpenChannel`.

#### Scenario: channel ID decoded correctly
- **WHEN** the packet contains channel ID 3
- **THEN** the dispatched action has channel ID 3

### Requirement: parseCloseChannel decodes channel ID
`parseCloseChannel` SHALL read a 2-byte channel ID and dispatch a close-channel action.

#### Scenario: channel ID decoded correctly
- **WHEN** the packet contains channel ID 5
- **THEN** the dispatched action has channel ID 5

### Requirement: parseChannelInvite decodes invitee name
`parseChannelInvite` SHALL read a string (invitee name) and dispatch a channel-invite action.

#### Scenario: name decoded correctly
- **WHEN** the packet contains name "Alice"
- **THEN** the dispatched action has name "Alice"

### Requirement: parseChannelExclude decodes excluded name
`parseChannelExclude` SHALL read a string and dispatch a channel-exclude action.

#### Scenario: name decoded correctly
- **WHEN** the packet contains name "Bob"
- **THEN** the dispatched action has name "Bob"

### Requirement: parseOpenPrivateChannel decodes receiver name
`parseOpenPrivateChannel` SHALL read a string and dispatch `Game::playerCreatePrivateChannel`.

#### Scenario: name decoded correctly
- **WHEN** the packet contains name "Carol"
- **THEN** the dispatched action has name "Carol"

### Requirement: parseInviteToParty decodes creature ID
`parseInviteToParty` SHALL read a 4-byte creature ID and dispatch a party-invite action.

#### Scenario: creature ID decoded correctly
- **WHEN** the packet contains ID 55
- **THEN** the dispatched action has ID 55

### Requirement: parseJoinParty decodes creature ID
`parseJoinParty` SHALL read a 4-byte creature ID and dispatch a join-party action.

#### Scenario: creature ID decoded correctly
- **WHEN** the packet contains ID 66
- **THEN** the dispatched action has ID 66

### Requirement: parseRevokePartyInvite decodes creature ID
`parseRevokePartyInvite` SHALL read a 4-byte creature ID and dispatch a revoke-party-invite action.

#### Scenario: creature ID decoded correctly
- **WHEN** the packet contains ID 77
- **THEN** the dispatched action has ID 77

### Requirement: parsePassPartyLeadership decodes creature ID
`parsePassPartyLeadership` SHALL read a 4-byte creature ID and dispatch a pass-leadership action.

#### Scenario: creature ID decoded correctly
- **WHEN** the packet contains ID 88
- **THEN** the dispatched action has ID 88

### Requirement: parseEnableSharedPartyExperience decodes bool flag
`parseEnableSharedPartyExperience` SHALL read a 1-byte boolean and dispatch a shared-XP toggle.

#### Scenario: enable flag decoded correctly
- **WHEN** the packet contains byte 1 (enable)
- **THEN** the dispatched action has enabled=true

### Requirement: parseLookInShop decodes item ID and count
`parseLookInShop` SHALL read a 2-byte item ID and 1-byte count and dispatch a look-in-shop action.

#### Scenario: shop look fields decoded correctly
- **WHEN** the packet contains itemId 100 and count 5
- **THEN** the dispatched action has itemId 100 and count 5

### Requirement: parsePlayerPurchase decodes shop purchase fields
`parsePlayerPurchase` SHALL read item ID, sub-type, count, ignore capacity flag, and buy-with-backpack flag.

#### Scenario: purchase fields decoded correctly
- **WHEN** packet contains itemId 2160, subtype 0, count 1, ignoreCapacity false, buyWithBackpack false
- **THEN** the dispatched action contains those values

### Requirement: parsePlayerSale decodes shop sale fields
`parsePlayerSale` SHALL read item ID, sub-type, count, and ignore-equipped flag.

#### Scenario: sale fields decoded correctly
- **WHEN** packet contains itemId 2160, subtype 0, count 1, ignoreEquipped false
- **THEN** the dispatched action contains those values

### Requirement: parseCloseShop dispatches close-shop action
`parseCloseShop` SHALL dispatch `Game::playerCloseShop` with no additional fields.

#### Scenario: close-shop is dispatched
- **WHEN** a close-shop packet is received
- **THEN** `Game::playerCloseShop` is dispatched

### Requirement: parseRequestTrade decodes position, item ID, and creature ID
`parseRequestTrade` SHALL read position, item ID, stack index, and target creature ID, then dispatch a request-trade action.

#### Scenario: trade fields decoded correctly
- **WHEN** packet contains position (1,2,7), itemId 100, stackpos 0, creature 42
- **THEN** the dispatched action contains those values

### Requirement: parseLookInTrade decodes counter-party and slot
`parseLookInTrade` SHALL read an "other player" boolean and slot index.

#### Scenario: look-in-trade fields decoded correctly
- **WHEN** packet contains otherPlayer=true, index=2
- **THEN** the dispatched action contains otherPlayer=true and index 2

### Requirement: parseAcceptTrade dispatches accept-trade with no fields
`parseAcceptTrade` SHALL dispatch `Game::playerAcceptTrade`.

#### Scenario: accept-trade is dispatched
- **WHEN** an accept-trade packet is received
- **THEN** `Game::playerAcceptTrade` is dispatched

### Requirement: parseRejectTrade dispatches close-trade
`parseRejectTrade` SHALL dispatch `Game::playerCloseTrade`.

#### Scenario: reject-trade is dispatched
- **WHEN** a reject-trade packet is received
- **THEN** `Game::playerCloseTrade` is dispatched

### Requirement: parseEquipObject decodes item ID and sub-type
`parseEquipObject` SHALL read a 2-byte item ID and 1-byte sub-type.

#### Scenario: equip object fields decoded correctly
- **WHEN** packet contains itemId 2471 and subtype 0
- **THEN** the dispatched action has itemId 2471 and subtype 0

### Requirement: parseTextWindow decodes window ID and text
`parseTextWindow` SHALL read a 4-byte window ID and a string, then dispatch a text-window update.

#### Scenario: text window fields decoded correctly
- **WHEN** packet contains windowId 1 and text "Hello"
- **THEN** the dispatched action has windowId 1 and text "Hello"

### Requirement: parseHouseWindow decodes window ID, door ID, and text
`parseHouseWindow` SHALL read a 1-byte window ID, 4-byte door ID, and string.

#### Scenario: house window fields decoded correctly
- **WHEN** packet contains windowId 0, doorId 123, text "pass"
- **THEN** the dispatched action has all three values

### Requirement: parseMarketBrowse decodes market request type or item ID
`parseMarketBrowse` SHALL read a 2-byte value. If it equals `MARKETREQUEST_OWN_OFFERS` or `MARKETREQUEST_OWN_HISTORY`, dispatch accordingly; otherwise treat as item ID.

#### Scenario: own-offers request dispatches correctly
- **WHEN** packet contains value MARKETREQUEST_OWN_OFFERS
- **THEN** the dispatched action requests the player's own offers

#### Scenario: item ID request dispatches correctly
- **WHEN** packet contains item ID 2148
- **THEN** the dispatched action browses item 2148

### Requirement: parseMarketCreateOffer decodes offer type, item ID, and price
`parseMarketCreateOffer` SHALL read type (buy/sell), item ID, and price.

#### Scenario: create-offer fields decoded correctly
- **WHEN** packet contains type SELL, itemId 2148, amount 1, price 1000
- **THEN** the dispatched action contains those values

### Requirement: parseMarketCancelOffer decodes offer ID
`parseMarketCancelOffer` SHALL read a 4-byte offer ID.

#### Scenario: cancel-offer ID decoded correctly
- **WHEN** packet contains offer ID 999
- **THEN** the dispatched action has offer ID 999

### Requirement: parseMarketAcceptOffer decodes offer ID and amount
`parseMarketAcceptOffer` SHALL read a 4-byte offer ID and 2-byte amount.

#### Scenario: accept-offer fields decoded correctly
- **WHEN** packet contains offer ID 500 and amount 3
- **THEN** the dispatched action has those values

### Requirement: parseMarketLeave dispatches leave-market action
`parseMarketLeave` SHALL dispatch a leave-market action with no fields.

#### Scenario: leave-market is dispatched
- **WHEN** a market-leave packet is received
- **THEN** a leave-market action is dispatched

### Requirement: parseAddVip decodes creature ID for VIP addition
`parseAddVip` SHALL read a 4-byte creature ID and dispatch an add-VIP action.

#### Scenario: VIP creature ID decoded correctly
- **WHEN** packet contains creature ID 123
- **THEN** the dispatched action has ID 123

### Requirement: parseAddVipByName decodes player name for VIP addition
`parseAddVipByName` SHALL read a string (player name) and dispatch an add-VIP-by-name action.

#### Scenario: VIP name decoded correctly
- **WHEN** packet contains name "Dave"
- **THEN** the dispatched action has name "Dave"

### Requirement: parseRemoveVip decodes creature ID for VIP removal
`parseRemoveVip` SHALL read a 4-byte creature ID and dispatch a remove-VIP action.

#### Scenario: VIP removal ID decoded correctly
- **WHEN** packet contains creature ID 456
- **THEN** the dispatched action has ID 456

### Requirement: parseEditVip decodes creature ID, description, and icon
`parseEditVip` SHALL read creature ID, description string, icon byte, and notify-offline bool.

#### Scenario: edit-VIP fields decoded correctly
- **WHEN** packet contains ID 1, description "friend", icon 0, notify false
- **THEN** the dispatched action contains those values

### Requirement: parseBrowseField decodes position
`parseBrowseField` SHALL read a position and dispatch a browse-field action.

#### Scenario: position decoded correctly
- **WHEN** packet contains position (5,5,7)
- **THEN** the dispatched action has position (5,5,7)

### Requirement: parseRotateItem decodes position, item ID, and stack index
`parseRotateItem` SHALL read position, item ID, and stack index and dispatch a rotate-item action.

#### Scenario: rotate-item fields decoded correctly
- **WHEN** packet contains position (1,2,7), itemId 2160, stackpos 0
- **THEN** the dispatched action has those values

### Requirement: parseWrapItem decodes position, item ID, and stack index
`parseWrapItem` SHALL read position, item ID, and stack index and dispatch a wrap/unwrap action.

#### Scenario: wrap-item fields decoded correctly
- **WHEN** packet contains position (1,2,7), itemId 100, stackpos 0
- **THEN** the dispatched action has those values

### Requirement: parseModalWindowAnswer decodes window ID and button/choice
`parseModalWindowAnswer` SHALL read a 4-byte window ID, button ID, and choice ID.

#### Scenario: modal window answer fields decoded correctly
- **WHEN** packet contains windowId 1, buttonId 0, choiceId 2
- **THEN** the dispatched action has those values

### Requirement: parseEditPodiumRequest decodes position and outfit
`parseEditPodiumRequest` SHALL read a position, stack index, item ID, outfit ID, and directional flag.

#### Scenario: podium edit fields decoded correctly
- **WHEN** packet contains position (3,3,7), outfit ID 128, direction east
- **THEN** the dispatched action has those values

### Requirement: parseSetOutfit decodes outfit fields
`parseSetOutfit` SHALL read outfit look type, head/body/legs/feet/addons colors, and mount ID.

#### Scenario: outfit colors decoded correctly
- **WHEN** packet contains lookType 128, head 10, body 20, legs 30, feet 40, addons 3, mountId 0
- **THEN** the dispatched action contains those values

### Requirement: parseUpdateContainer decodes container ID
`parseUpdateContainer` SHALL read a 1-byte container ID and dispatch an update-container action.

#### Scenario: container ID decoded correctly
- **WHEN** packet contains container ID 2
- **THEN** the dispatched action has container ID 2

### Requirement: parseUseItemEx decodes from-position, item ID, and to-position
`parseUseItemEx` SHALL read source position, item ID, stack index, and target position (creature or item).

#### Scenario: use-item-ex fields decoded correctly
- **WHEN** packet contains from (1,1,7), itemId 2160, stackpos 0, to (2,2,7)
- **THEN** the dispatched action has those values

### Requirement: parseUseWithCreature decodes item position and creature ID
`parseUseWithCreature` SHALL read position, item ID, stack index, and creature ID.

#### Scenario: use-with-creature fields decoded correctly
- **WHEN** packet contains position (1,1,7), itemId 2160, stackpos 0, creatureId 999
- **THEN** the dispatched action has those values

### Requirement: parseExtendedOpcode decodes channel byte and string
`parseExtendedOpcode` SHALL read a 1-byte channel ID and a string payload, then forward to the scripting extension handler.

#### Scenario: extended opcode fields decoded correctly
- **WHEN** packet contains channel 0 and payload "test"
- **THEN** the dispatched action has channel 0 and payload "test"

### Requirement: parseDebugAssert reads and discards the assert payload
`parseDebugAssert` SHALL read the assert fields (assert line, date, description, comment) without crashing, and optionally log them.

#### Scenario: debug assert packet is consumed without panic
- **WHEN** a debug-assert packet is received with valid fields
- **THEN** the packet is consumed without error

### Requirement: parseRuleViolationReport reads report fields
`parseRuleViolationReport` SHALL read type, reason, comment, translation, and optional targets.

#### Scenario: report fields are consumed without panic
- **WHEN** a rule-violation packet is received
- **THEN** it is consumed without error

### Requirement: parseReceivePing dispatches ping reply
`parseReceivePing` SHALL dispatch `Game::playerReceivePing`.

#### Scenario: ping is dispatched
- **WHEN** a ping packet is received
- **THEN** `Game::playerReceivePing` is dispatched

### Requirement: parseReceivePingBack dispatches ping-back
`parseReceivePingBack` SHALL dispatch `Game::playerReceivePingBack`.

#### Scenario: ping-back is dispatched
- **WHEN** a ping-back packet is received
- **THEN** `Game::playerReceivePingBack` is dispatched
