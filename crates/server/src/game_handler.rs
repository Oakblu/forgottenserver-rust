use forgottenserver_common::position::Position;
use forgottenserver_database::iologindata::{IoLoginData, LoginDb, PlayerLoginData};
use forgottenserver_game::{
    action_registry::ActionRegistry,
    chat::{ChannelId, ChatManager, EntityId, SpeakType},
    market::MarketManager,
    party::PartyManager,
    quest_registry::QuestRegistry,
    trade::{TradeId, TradeItem, TradeManager, TradeResult},
    weapon_registry::CombatResolver,
};
use forgottenserver_map::pathfinder::Pathfinder;
use forgottenserver_world::World;

use crate::{
    channel_session::ChannelSession,
    codec::{encode, ServerPacket},
    game_state::{GameState, OutfitAppearance},
    map_description::MapDescriptionBuilder,
};

/// Produce the encoded map description for a player entering the game at `player_pos`.
pub fn on_enter_game(world: &World, player_pos: Position) -> Vec<u8> {
    let builder = MapDescriptionBuilder::new(world);
    let tiles = builder.build(player_pos);
    encode(&ServerPacket::FullMapDescription { player_pos, tiles })
}

/// Produce the encoded map description after the player walks to `new_pos`.
pub fn on_walk(world: &World, new_pos: Position) -> Vec<u8> {
    let builder = MapDescriptionBuilder::new(world);
    let tiles = builder.build(new_pos);
    encode(&ServerPacket::FullMapDescription {
        player_pos: new_pos,
        tiles,
    })
}

/// Assemble the enter-world burst sent to a player immediately on login.
///
/// The burst contains three concatenated packets in wire order:
/// 1. `0x0A` — enter-world acknowledgement byte
/// 2. `0x64` — full map description centred on the player's position
/// 3. `0xA0` — player stats (HP, mana, level, stamina)
pub fn build_enter_world_burst(player: &PlayerLoginData, world: &World) -> Vec<u8> {
    let mut burst = vec![0x0A];
    let player_pos = Position::new(player.posx, player.posy, player.posz);
    burst.extend_from_slice(&on_enter_game(world, player_pos));
    burst.extend_from_slice(&encode(&ServerPacket::PlayerStats {
        health: player.health as i32,
        max_health: player.healthmax as i32,
        mana: player.mana as i32,
        max_mana: player.manamax as i32,
        level: player.level,
        stamina: player.stamina as u16,
    }));
    burst
}

// ---------------------------------------------------------------------------
// Chat dispatch handlers
// ---------------------------------------------------------------------------

/// Encode the list of available channels to send to a connecting player.
pub fn handle_get_channels(chat: &ChatManager) -> Vec<u8> {
    encode(&ServerPacket::ChannelList {
        channels: chat.available_channels(),
    })
}

/// Subscribe the player to the channel and return the `OpenChannel` acknowledgement.
pub fn handle_open_channel(
    chat: &mut ChatManager,
    session: &mut ChannelSession,
    channel_id: ChannelId,
) -> Vec<u8> {
    chat.subscribe(channel_id, session.player_id);
    session.add_channel(channel_id);
    let name = chat
        .available_channels()
        .into_iter()
        .find(|c| c.id == channel_id)
        .map(|c| c.name)
        .unwrap_or_default();
    encode(&ServerPacket::OpenChannel { channel_id, name })
}

/// Unsubscribe the player from the channel.  No response packet is sent.
pub fn handle_close_channel(
    chat: &mut ChatManager,
    session: &mut ChannelSession,
    channel_id: ChannelId,
) {
    chat.unsubscribe(channel_id, session.player_id);
    session.remove_channel(channel_id);
}

/// Route a Say/Yell/Whisper or channel message. Returns (recipient_id, encoded_bytes) pairs.
pub fn handle_say(
    chat: &ChatManager,
    sender_name: &str,
    speak_type: SpeakType,
    text: &str,
    channel_id: Option<ChannelId>,
) -> Vec<(EntityId, Vec<u8>)> {
    use forgottenserver_game::chat::CHANNEL_WORLD;
    let broadcast_channel = channel_id.unwrap_or(CHANNEL_WORLD);
    chat.broadcast(broadcast_channel, sender_name, speak_type.clone(), text)
        .into_iter()
        .map(|(pid, msg)| {
            let packet = ServerPacket::Talk {
                speaker: msg.speaker,
                speak_type: msg.speak_type,
                channel_id,
                text: msg.text,
            };
            (pid, encode(&packet))
        })
        .collect()
}

/// Return an `OpenPrivateChannel` packet for the requesting player.
pub fn handle_open_private_channel(receiver_name: &str) -> Vec<u8> {
    encode(&ServerPacket::OpenPrivateChannel {
        receiver: receiver_name.to_string(),
    })
}

/// Route a private message.  Returns (receiver_id, encoded_bytes).
pub fn handle_say_private(
    chat: &ChatManager,
    sender_name: &str,
    receiver_id: EntityId,
    text: &str,
) -> (EntityId, Vec<u8>) {
    let (pid, msg) = chat.send_private(sender_name, receiver_id, text);
    let packet = ServerPacket::Talk {
        speaker: msg.speaker,
        speak_type: msg.speak_type,
        channel_id: None,
        text: msg.text,
    };
    (pid, encode(&packet))
}

// ---------------------------------------------------------------------------
// Phase 1 — Attack dispatch
// ---------------------------------------------------------------------------

/// Compute melee damage and apply it to `target_id` in `state`.
/// Returns `[DamageEffect, CreatureHealth]` encoded packets, or empty if target unknown.
pub fn handle_attack(
    resolver: &CombatResolver,
    attacker_item_id: u16,
    target_id: u32,
    state: &mut GameState,
) -> Vec<Vec<u8>> {
    let damage = resolver.compute_melee(attacker_item_id, 1);
    match state.apply_damage(target_id, damage) {
        Some((_remaining, percent)) => vec![
            encode(&ServerPacket::DamageEffect {
                creature_id: target_id,
                damage,
            }),
            encode(&ServerPacket::CreatureHealth {
                creature_id: target_id,
                health_percent: percent,
            }),
        ],
        None => vec![],
    }
}

// ---------------------------------------------------------------------------
// Phase 2 — UseItem dispatch
// ---------------------------------------------------------------------------

/// Dispatch a UseItem action through the `ActionRegistry`.
/// Always returns a `TextMessage` packet: the callback's message or a fallback.
pub fn handle_use_item(registry: &ActionRegistry, item_id: u16) -> Vec<u8> {
    let text = registry
        .dispatch(item_id)
        .unwrap_or_else(|| "Sorry, not possible.".to_string());
    encode(&ServerPacket::TextMessage {
        message_type: 0x13,
        text,
    })
}

// ---------------------------------------------------------------------------
// Phase 3 — Follow dispatch
// ---------------------------------------------------------------------------

/// Compute path from `player_pos` to `target_pos` and store as the player's follow target.
pub fn handle_follow(
    pathfinder: &Pathfinder,
    player_pos: Position,
    target_pos: Position,
    player_id: u32,
    target_id: u32,
    state: &mut GameState,
) {
    let path = pathfinder.find_path(player_pos, target_pos);
    state.set_follow_target(player_id, target_id, path);
}

// ---------------------------------------------------------------------------
// Phase 4 — FightModes dispatch
// ---------------------------------------------------------------------------

/// Update the player's fight/chase/secure mode fields in game state.
pub fn handle_fight_modes(
    player_id: u32,
    fight_mode: u8,
    chase_mode: u8,
    secure_mode: bool,
    state: &mut GameState,
) {
    state.set_fight_mode(player_id, fight_mode, chase_mode, secure_mode);
}

// ---------------------------------------------------------------------------
// Phase 5 — AutoWalk dispatch
// ---------------------------------------------------------------------------

/// Store the auto-walk path for a player. An empty path cancels movement.
pub fn handle_auto_walk(player_id: u32, path: Vec<u8>, state: &mut GameState) {
    state.set_auto_walk(player_id, path);
}

// ---------------------------------------------------------------------------
// Phase 6 — SetOutfit dispatch
// ---------------------------------------------------------------------------

/// Update the player's outfit and broadcast `CreatureOutfit` to all players in viewport.
/// Returns (recipient_player_id, encoded_packet) pairs.
pub fn handle_set_outfit(
    player_id: u32,
    player_pos: Position,
    outfit: OutfitAppearance,
    state: &mut GameState,
) -> Vec<(u32, Vec<u8>)> {
    state.set_outfit(player_id, outfit);
    let in_viewport = state.get_players_in_viewport(player_pos);
    in_viewport
        .into_iter()
        .map(|(pid, _pos)| {
            let packet = encode(&ServerPacket::CreatureOutfit {
                creature_id: player_id,
                look_type: outfit.look_type,
                look_head: outfit.look_head,
                look_body: outfit.look_body,
                look_legs: outfit.look_legs,
                look_feet: outfit.look_feet,
                look_addons: outfit.look_addons,
            });
            (pid, packet)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Phase 7 — Remaining arms
// ---------------------------------------------------------------------------

/// Remove a VIP entry from the player's list.
pub fn handle_vip_remove(player_id: u32, guid: u32, state: &mut GameState) {
    state.remove_vip(player_id, guid);
}

/// Return the encoded `QuestList` packet for the player.
pub fn handle_quests(quest_registry: &QuestRegistry) -> Vec<u8> {
    let quests = quest_registry
        .list_for_player()
        .iter()
        .map(|q| (q.id, q.name.clone(), q.completed))
        .collect();
    encode(&ServerPacket::QuestList { quests })
}

/// Return the encoded `QuestDetails` packet for `quest_id`.
pub fn handle_quest_info(quest_registry: &QuestRegistry, quest_id: u16) -> Vec<u8> {
    match quest_registry.mission_info(quest_id) {
        Some(quest) => {
            let missions = quest
                .missions
                .iter()
                .map(|m| (m.name.clone(), m.description.clone()))
                .collect();
            encode(&ServerPacket::QuestDetails {
                quest_id,
                name: quest.name.clone(),
                missions,
            })
        }
        None => encode(&ServerPacket::QuestDetails {
            quest_id,
            name: String::new(),
            missions: vec![],
        }),
    }
}

/// Log a bug report from the client.
pub fn handle_bug_report(comment: &str) {
    eprintln!("[BugReport] {comment}");
}

/// Log a client debug assertion.
pub fn handle_debug_assert(message: &str) {
    eprintln!("[DebugAssert] {message}");
}

// ---------------------------------------------------------------------------
// Phase 1 — Party dispatch handlers
// ---------------------------------------------------------------------------

/// Accept a party invite. Returns `(recipient_id, encoded_PartyUpdate)` pairs.
pub fn handle_join_party(
    party_mgr: &mut PartyManager,
    invitee_id: EntityId,
    inviter_id: EntityId,
) -> Vec<(EntityId, Vec<u8>)> {
    party_mgr
        .accept_invite(invitee_id, inviter_id)
        .into_iter()
        .map(|(pid, leader_id)| {
            (
                pid,
                encode(&ServerPacket::PartyUpdate {
                    new_leader_id: leader_id,
                }),
            )
        })
        .collect()
}

/// Revoke a pending party invite. No packet is sent.
pub fn handle_revoke_party_invite(
    party_mgr: &mut PartyManager,
    leader_id: EntityId,
    player_id: EntityId,
) {
    party_mgr.revoke_invite(leader_id, player_id);
}

/// Transfer leadership. Returns `(recipient_id, encoded_PartyUpdate)` pairs.
pub fn handle_pass_party_leadership(
    party_mgr: &mut PartyManager,
    old_leader_id: EntityId,
    new_leader_id: EntityId,
) -> Vec<(EntityId, Vec<u8>)> {
    party_mgr
        .pass_leadership(old_leader_id, new_leader_id)
        .into_iter()
        .map(|(pid, leader_id)| {
            (
                pid,
                encode(&ServerPacket::PartyUpdate {
                    new_leader_id: leader_id,
                }),
            )
        })
        .collect()
}

/// Leave the party. Returns `(recipient_id, encoded_PartyUpdate)` pairs.
pub fn handle_leave_party(
    party_mgr: &mut PartyManager,
    player_id: EntityId,
) -> Vec<(EntityId, Vec<u8>)> {
    party_mgr
        .leave(player_id)
        .into_iter()
        .map(|(pid, _remaining)| (pid, encode(&ServerPacket::PartyUpdate { new_leader_id: 0 })))
        .collect()
}

/// Toggle shared experience for the player's party. No packet is sent.
pub fn handle_share_party_experience(
    party_mgr: &mut PartyManager,
    player_id: EntityId,
    active: bool,
) {
    party_mgr.set_shared_xp(player_id, active);
}

// ---------------------------------------------------------------------------
// Phase 2 — Trade dispatch handlers
// ---------------------------------------------------------------------------

/// Open a trade. Returns encoded `TradeItems` or a `TextMessage` on error.
pub fn handle_request_trade(
    trade_mgr: &mut TradeManager,
    player_id: EntityId,
    target_player_id: EntityId,
    item_type_id: Option<u16>,
) -> Vec<u8> {
    let type_id = match item_type_id {
        Some(id) => id,
        None => {
            return encode(&ServerPacket::TextMessage {
                message_type: 0x13,
                text: "Item not found.".to_string(),
            })
        }
    };
    let item = TradeItem { type_id };
    match trade_mgr.open(player_id, target_player_id, item) {
        Ok(_) => encode(&ServerPacket::TradeItems {
            your_type_id: type_id,
            their_type_id: None,
        }),
        Err(_) => encode(&ServerPacket::TextMessage {
            message_type: 0x13,
            text: "You already have a trade open.".to_string(),
        }),
    }
}

/// Inspect the opposite side's item in a trade.
pub fn handle_look_in_trade(
    trade_mgr: &TradeManager,
    player_id: EntityId,
    trade_id: TradeId,
) -> Vec<u8> {
    let type_id = trade_mgr
        .inspect(trade_id, player_id)
        .map(|i| i.type_id)
        .unwrap_or(0);
    encode(&ServerPacket::ItemDescription {
        type_id,
        text: format!("Item {type_id}"),
    })
}

/// Accept the trade. Returns a `TextMessage` indicating outcome.
pub fn handle_accept_trade(
    trade_mgr: &mut TradeManager,
    player_id: EntityId,
    trade_id: TradeId,
) -> Vec<u8> {
    match trade_mgr.accept(trade_id, player_id) {
        TradeResult::Completed => encode(&ServerPacket::TextMessage {
            message_type: 0x13,
            text: "Trade complete.".to_string(),
        }),
        TradeResult::Pending => encode(&ServerPacket::TextMessage {
            message_type: 0x13,
            text: "Waiting for other player.".to_string(),
        }),
    }
}

/// Cancel the trade silently.
pub fn handle_close_trade(trade_mgr: &mut TradeManager, trade_id: TradeId) {
    trade_mgr.close(trade_id);
}

// ---------------------------------------------------------------------------
// Phase 3 — Market dispatch handlers
// ---------------------------------------------------------------------------

/// Place a buy offer in the market. Returns encoded `MarketDetail` packet.
pub fn handle_player_purchase(
    market_mgr: &mut MarketManager,
    player_id: EntityId,
    item_id: u16,
    amount: u16,
    price: u32,
) -> Vec<u8> {
    market_mgr.place_buy_offer(player_id, item_id, amount, price);
    let (buys, sells) = market_mgr.list_offers(item_id);
    encode(&ServerPacket::MarketDetail {
        buy_offers: buys.iter().map(|o| (o.price, o.amount)).collect(),
        sell_offers: sells.iter().map(|o| (o.price, o.amount)).collect(),
    })
}

/// Place a sell offer in the market. Returns encoded `MarketDetail` packet.
pub fn handle_player_sale(
    market_mgr: &mut MarketManager,
    player_id: EntityId,
    item_id: u16,
    amount: u16,
    price: u32,
) -> Vec<u8> {
    market_mgr.place_sell_offer(player_id, item_id, amount, price);
    let (buys, sells) = market_mgr.list_offers(item_id);
    encode(&ServerPacket::MarketDetail {
        buy_offers: buys.iter().map(|o| (o.price, o.amount)).collect(),
        sell_offers: sells.iter().map(|o| (o.price, o.amount)).collect(),
    })
}

/// List market offers for an item. Returns encoded `MarketDetail` packet.
pub fn handle_look_in_shop(market_mgr: &MarketManager, item_id: u16) -> Vec<u8> {
    let (buys, sells) = market_mgr.list_offers(item_id);
    encode(&ServerPacket::MarketDetail {
        buy_offers: buys.iter().map(|o| (o.price, o.amount)).collect(),
        sell_offers: sells.iter().map(|o| (o.price, o.amount)).collect(),
    })
}

/// Close a player's market session. No response packet.
pub fn handle_close_shop(market_mgr: &mut MarketManager, player_id: EntityId) {
    market_mgr.close_session(player_id);
}

// ---------------------------------------------------------------------------
// Phase 4 — Guild integration handlers
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Phase 1 — Player death dispatch
// ---------------------------------------------------------------------------

/// Apply player death in state and return [PlayerDeath, PlayerStats] encoded packets.
/// Returns an empty vec if the player is not found.
pub fn handle_on_player_death(player_id: u32, state: &mut GameState) -> Vec<Vec<u8>> {
    if !state.on_player_death(player_id) {
        return vec![];
    }
    let Some(player) = state.get_player_entity(player_id) else {
        return vec![];
    };
    vec![
        encode(&ServerPacket::PlayerDeath),
        encode(&ServerPacket::PlayerStats {
            health: player.get_health(),
            max_health: player.get_max_health(),
            mana: player.get_mana(),
            max_mana: player.get_max_mana(),
            level: player.get_level(),
            stamina: player.get_stamina(),
        }),
    ]
}

// ---------------------------------------------------------------------------
// Phase 2 — XP / level-up dispatch
// ---------------------------------------------------------------------------

/// Grant XP to a player (stamina-adjusted). If the player levels up, returns
/// encoded [PlayerStats, TextMessage] packets; otherwise returns an empty vec.
pub fn handle_grant_xp(player_id: u32, base_xp: u64, state: &mut GameState) -> Vec<Vec<u8>> {
    let Some(new_level) = state.grant_xp(player_id, base_xp) else {
        return vec![];
    };
    let Some(player) = state.get_player_entity(player_id) else {
        return vec![];
    };
    vec![
        encode(&ServerPacket::PlayerStats {
            health: player.get_health(),
            max_health: player.get_max_health(),
            mana: player.get_mana(),
            max_mana: player.get_max_mana(),
            level: new_level,
            stamina: player.get_stamina(),
        }),
        encode(&ServerPacket::TextMessage {
            message_type: 0x13,
            text: format!("You advanced to level {new_level}."),
        }),
    ]
}

// ---------------------------------------------------------------------------
// Phase 5 — Logout / player save dispatch
// ---------------------------------------------------------------------------

/// Save the player's current state to DB and remove from active game state.
/// Returns `true` if the player was found and saved.
pub fn handle_logout(
    player_id: u32,
    account_id: u32,
    state: &mut GameState,
    io: &IoLoginData,
    db: &mut LoginDb,
) -> bool {
    let Some(player) = state.get_player_entity(player_id) else {
        return false;
    };
    io.save_player_entity(db, player, account_id, 0);
    state.remove_player_entity(player_id);
    true
}

/// Encode a `PlayerGuildInfo` packet if `guild_name` is non-empty, or return `None`.
pub fn handle_player_guild_info(guild_name: &str, guild_rank: &str) -> Option<Vec<u8>> {
    if guild_name.is_empty() {
        None
    } else {
        Some(encode(&ServerPacket::PlayerGuildInfo {
            name: guild_name.to_string(),
            rank: guild_rank.to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_database::iologindata::{IoLoginData, LoginDb};
    use forgottenserver_entity::creature::Creature;
    use forgottenserver_entity::player::{xp_for_level, Player};
    use forgottenserver_game::{
        action_registry::ActionRegistry,
        chat::{ChatManager, CHANNEL_WORLD},
        market::MarketManager,
        party::PartyManager,
        quest_registry::{MissionDef, QuestDef, QuestRegistry},
        trade::TradeManager,
        weapon_registry::{CombatResolver, WeaponDef, WeaponRegistry, WeaponType},
    };
    use forgottenserver_map::pathfinder::Pathfinder;
    use forgottenserver_world::{World, WorldTile};

    use crate::game_state::{GameState, OutfitAppearance};

    // -----------------------------------------------------------------------
    // Phase 2 (chat) — existing tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_channels_returns_default_channel_list() {
        let chat = ChatManager::new();
        let encoded = handle_get_channels(&chat);

        assert_eq!(encoded[0], 0xAC, "opcode must be 0xAC");
        let count = encoded[1] as usize;
        assert!(count >= 2, "Must list at least 2 channels, got {count}");
    }

    #[test]
    fn open_channel_acks_with_channel_name() {
        let mut chat = ChatManager::new();
        let mut session = ChannelSession::new(42);
        let encoded = handle_open_channel(&mut chat, &mut session, CHANNEL_WORLD);

        assert_eq!(encoded[0], 0xAB, "opcode must be 0xAB");
        let cid = u16::from_le_bytes([encoded[1], encoded[2]]);
        assert_eq!(cid, CHANNEL_WORLD);
        let name_len = u16::from_le_bytes([encoded[3], encoded[4]]) as usize;
        assert!(name_len > 0, "Channel name must be non-empty");
        let name = std::str::from_utf8(&encoded[5..5 + name_len]).expect("valid utf8");
        assert_eq!(name, "World");
        assert!(session.open_channels().contains(&CHANNEL_WORLD));
    }

    // -----------------------------------------------------------------------
    // Phase 4 — private channel
    // -----------------------------------------------------------------------

    #[test]
    fn say_private_routes_to_single_player() {
        let chat = ChatManager::new();
        let (receiver_id, bytes) = handle_say_private(&chat, "Alice", 99, "Hello privately");

        assert_eq!(receiver_id, 99);
        assert_eq!(bytes[0], 0xAA, "talk opcode must be 0xAA");
        let speaker_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
        let speak_type_idx = 3 + speaker_len;
        assert_eq!(
            bytes[speak_type_idx], 0x05,
            "speak_type byte must be 0x05 (Private)"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 4 — walk (map description)
    // -----------------------------------------------------------------------

    #[test]
    fn walk_sends_updated_map_description() {
        let mut world = World::new();
        let dest = Position::new(101, 100, 7);
        world.set_tile(
            dest,
            WorldTile {
                ground_item_id: Some(200),
                ..WorldTile::default()
            },
        );

        let initial_bytes = on_enter_game(&world, Position::new(100, 100, 7));
        let walk_bytes = on_walk(&world, dest);

        assert_eq!(initial_bytes[0], 0x64, "enter-game must use map opcode");
        assert_eq!(walk_bytes[0], 0x64, "walk must use map opcode");
        assert_ne!(
            initial_bytes, walk_bytes,
            "Walk must produce a different map description"
        );

        let origin_x = u16::from_le_bytes([walk_bytes[1], walk_bytes[2]]);
        let origin_y = u16::from_le_bytes([walk_bytes[3], walk_bytes[4]]);
        assert_eq!(origin_x, dest.x - 8, "Walk viewport origin x must update");
        assert_eq!(origin_y, dest.y - 6, "Walk viewport origin y must update");
    }

    // -----------------------------------------------------------------------
    // Phase 1 — Attack
    // -----------------------------------------------------------------------

    #[test]
    fn attack_calls_combat_resolver_and_returns_damage_effect() {
        let mut weapon_reg = WeaponRegistry::new();
        weapon_reg.register(WeaponDef {
            item_id: 100,
            weapon_type: WeaponType::Melee,
            attack: 20,
            defense: 10,
            min_level: 1,
            vocations: vec![],
            element: None,
        });
        let resolver = CombatResolver::new(&weapon_reg);

        let mut state = GameState::new();
        state.add_creature(Creature::new(42, "Rat"));

        let packets = handle_attack(&resolver, 100, 42, &mut state);

        assert_eq!(
            packets.len(),
            2,
            "Must return DamageEffect + CreatureHealth"
        );
        assert_eq!(packets[0][0], 0x84, "DamageEffect opcode must be 0x84");
        assert_eq!(packets[1][0], 0x8C, "CreatureHealth opcode must be 0x8C");

        let creature = state.get_creature(42).unwrap();
        assert!(creature.health < 100, "Health must decrease after attack");
    }

    // -----------------------------------------------------------------------
    // Phase 2 — UseItem
    // -----------------------------------------------------------------------

    #[test]
    fn use_item_registered_action_fires_callback() {
        let mut registry = ActionRegistry::new();
        registry.register(200, || Some("You use the item.".to_string()));

        let bytes = handle_use_item(&registry, 200);

        assert_eq!(bytes[0], 0xB4, "TextMessage opcode must be 0xB4");
    }

    #[test]
    fn use_item_unknown_id_returns_not_yet_supported() {
        let registry = ActionRegistry::new();

        let bytes = handle_use_item(&registry, 9999);

        assert_eq!(bytes[0], 0xB4, "TextMessage opcode must be 0xB4");
    }

    // -----------------------------------------------------------------------
    // Phase 3 — Follow
    // -----------------------------------------------------------------------

    #[test]
    fn follow_sets_path_via_pathfinder() {
        let pathfinder = Pathfinder::new();
        let mut state = GameState::new();

        let player_pos = Position::new(100, 100, 7);
        let target_pos = Position::new(105, 100, 7);

        state.add_creature(Creature::new(42, "Orc"));

        handle_follow(&pathfinder, player_pos, target_pos, 1, 42, &mut state);

        let follow = state.get_follow_target(1);
        assert!(follow.is_some(), "Follow target must be stored");
        let (target_id, _path) = follow.unwrap();
        assert_eq!(target_id, 42);
    }

    // -----------------------------------------------------------------------
    // Phase 4 — FightModes
    // -----------------------------------------------------------------------

    #[test]
    fn fight_modes_packet_updates_player_fields() {
        let mut state = GameState::new();

        handle_fight_modes(1, 2, 1, true, &mut state);

        let (fight, chase, secure) = state.get_fight_mode(1).unwrap();
        assert_eq!(fight, 2);
        assert_eq!(chase, 1);
        assert!(secure);
    }

    // -----------------------------------------------------------------------
    // Phase 5 — AutoWalk
    // -----------------------------------------------------------------------

    #[test]
    fn auto_walk_empty_path_is_no_op() {
        let mut state = GameState::new();

        handle_auto_walk(1, vec![], &mut state);

        let walk = state.get_auto_walk(1);
        assert!(
            walk.is_some(),
            "Walk state must be stored even for empty path"
        );
        assert!(walk.unwrap().is_empty(), "Empty path must remain empty");
    }

    // -----------------------------------------------------------------------
    // Phase 6 — SetOutfit
    // -----------------------------------------------------------------------

    #[test]
    fn set_outfit_broadcasts_to_viewport() {
        let mut state = GameState::new();

        let player_pos = Position::new(100, 100, 7);
        let other_pos = Position::new(103, 100, 7);
        let far_pos = Position::new(200, 200, 7);

        state.set_player_position(1, player_pos);
        state.set_player_position(2, other_pos);
        state.set_player_position(3, far_pos);

        let outfit = OutfitAppearance {
            look_type: 128,
            look_head: 10,
            look_body: 20,
            look_legs: 30,
            look_feet: 40,
            look_addons: 0,
        };

        let results = handle_set_outfit(1, player_pos, outfit, &mut state);

        assert!(
            results.iter().any(|(pid, _)| *pid == 2),
            "Player 2 (nearby) must receive outfit update"
        );
        assert!(
            !results.iter().any(|(pid, _)| *pid == 3),
            "Player 3 (far) must NOT receive outfit update"
        );
        for (_, bytes) in &results {
            assert_eq!(bytes[0], 0x8E, "CreatureOutfit opcode must be 0x8E");
        }
    }

    // -----------------------------------------------------------------------
    // Phase 7 — VipRemove
    // -----------------------------------------------------------------------

    #[test]
    fn vip_remove_removes_entry_from_list() {
        let mut state = GameState::new();
        state.add_vip(1, 42);
        state.add_vip(1, 99);

        handle_vip_remove(1, 42, &mut state);

        let vips = state.get_vip_list(1);
        assert!(!vips.contains(&42), "GUID 42 must be removed");
        assert!(vips.contains(&99), "GUID 99 must remain");
    }

    // -----------------------------------------------------------------------
    // Phase 7 + wire-market-guild-quest Phase 1 — Quest dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn quest_list_returns_active_quests_for_player() {
        let mut registry = QuestRegistry::new();
        registry.register(QuestDef {
            id: 2,
            name: "Dragon Slayer".to_string(),
            completed: false,
            missions: vec![MissionDef {
                name: "Kill a dragon".to_string(),
                description: "Find and kill a dragon.".to_string(),
            }],
        });

        let bytes = handle_quests(&registry);

        assert_eq!(bytes[0], 0xF0, "QuestList opcode must be 0xF0");
        let count = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
        assert_eq!(count, 1, "Must list exactly 1 active quest");
    }

    #[test]
    fn quest_info_returns_mission_text() {
        let mut registry = QuestRegistry::new();
        registry.register(QuestDef {
            id: 7,
            name: "The Mission".to_string(),
            completed: false,
            missions: vec![MissionDef {
                name: "Step 1".to_string(),
                description: "Go north.".to_string(),
            }],
        });

        let bytes = handle_quest_info(&registry, 7);

        assert_eq!(bytes[0], 0xF1, "QuestDetails opcode must be 0xF1");
        let qid = u16::from_le_bytes([bytes[1], bytes[2]]);
        assert_eq!(qid, 7);
    }

    #[test]
    fn quests_returns_quest_list_packet() {
        let mut registry = QuestRegistry::new();
        registry.register(QuestDef {
            id: 1,
            name: "The First Quest".to_string(),
            completed: false,
            missions: vec![],
        });

        let bytes = handle_quests(&registry);

        assert_eq!(bytes[0], 0xF0, "QuestList opcode must be 0xF0");
        let count = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
        assert_eq!(count, 1, "Must list exactly 1 quest");
    }

    // -----------------------------------------------------------------------
    // Phase 7 — QuestInfo
    // -----------------------------------------------------------------------

    #[test]
    fn quest_info_returns_mission_data() {
        let mut registry = QuestRegistry::new();
        registry.register(QuestDef {
            id: 5,
            name: "Epic Quest".to_string(),
            completed: true,
            missions: vec![MissionDef {
                name: "Phase 1".to_string(),
                description: "Do stuff.".to_string(),
            }],
        });

        let bytes = handle_quest_info(&registry, 5);

        assert_eq!(bytes[0], 0xF1, "QuestDetails opcode must be 0xF1");
    }

    // -----------------------------------------------------------------------
    // Phase 7 — BugReport / DebugAssert
    // -----------------------------------------------------------------------

    #[test]
    fn bug_report_logs_without_panic() {
        handle_bug_report("This is a bug report.");
    }

    #[test]
    fn debug_assert_logs_without_panic() {
        handle_debug_assert("Assertion message.");
    }

    // -----------------------------------------------------------------------
    // Phase 1 — Party dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn party_accept_invite_adds_member_and_broadcasts_update() {
        let mut party_mgr = PartyManager::new();
        let broadcasts = handle_join_party(&mut party_mgr, 2, 1);

        assert!(!broadcasts.is_empty(), "Must broadcast party update");
        assert!(
            broadcasts.iter().any(|(pid, _)| *pid == 1),
            "Leader must receive update"
        );
        assert!(
            broadcasts.iter().any(|(pid, _)| *pid == 2),
            "Invitee must receive update"
        );
        for (_, bytes) in &broadcasts {
            assert_eq!(bytes[0], 0x8B, "PartyUpdate opcode must be 0x8B");
        }
    }

    #[test]
    fn party_leave_disbands_when_solo_remaining() {
        let mut party_mgr = PartyManager::new();
        handle_join_party(&mut party_mgr, 2, 1);
        let broadcasts = handle_leave_party(&mut party_mgr, 2);

        assert!(
            !broadcasts.is_empty(),
            "Must broadcast disband notification"
        );
        assert!(
            broadcasts.iter().any(|(pid, _)| *pid == 1),
            "Leader must receive disband notice"
        );
        for (_, bytes) in &broadcasts {
            assert_eq!(bytes[0], 0x8B, "PartyUpdate opcode must be 0x8B");
            let leader_in_packet = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
            assert_eq!(leader_in_packet, 0, "Disbanded party sends leader_id=0");
        }
    }

    #[test]
    fn party_pass_leadership_updates_leader_id() {
        let mut party_mgr = PartyManager::new();
        handle_join_party(&mut party_mgr, 2, 1);
        let broadcasts = handle_pass_party_leadership(&mut party_mgr, 1, 2);

        assert!(!broadcasts.is_empty(), "Must broadcast leadership change");
        for (_, bytes) in &broadcasts {
            assert_eq!(bytes[0], 0x8B, "PartyUpdate opcode must be 0x8B");
            let new_leader_in_packet = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
            assert_eq!(new_leader_in_packet, 2, "New leader must be player 2");
        }
    }

    // -----------------------------------------------------------------------
    // Phase 3 — Trade dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn trade_request_missing_item_returns_text_error() {
        let mut trade_mgr = TradeManager::new();
        let bytes = handle_request_trade(&mut trade_mgr, 1, 2, None);

        assert_eq!(bytes[0], 0xB4, "TextMessage opcode must be 0xB4");
        let msg_type = bytes[1];
        assert_eq!(msg_type, 0x13);
        let text_len = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
        let text = std::str::from_utf8(&bytes[4..4 + text_len]).unwrap();
        assert!(
            text.contains("not found"),
            "Error text must mention 'not found'"
        );
    }

    #[test]
    fn trade_request_valid_item_returns_trade_items_packet() {
        let mut trade_mgr = TradeManager::new();
        let bytes = handle_request_trade(&mut trade_mgr, 1, 2, Some(500));

        assert_eq!(bytes[0], 0x7D, "TradeItems opcode must be 0x7D");
        let your_type = u16::from_le_bytes([bytes[1], bytes[2]]);
        assert_eq!(your_type, 500);
    }

    #[test]
    fn trade_look_returns_item_description_packet() {
        let mut trade_mgr = TradeManager::new();
        let trade_id = trade_mgr
            .open(
                1,
                2,
                forgottenserver_game::trade::TradeItem { type_id: 300 },
            )
            .unwrap();
        // Player 2 inspects → sees player 1's item
        let bytes = handle_look_in_trade(&trade_mgr, 2, trade_id);

        assert_eq!(bytes[0], 0x7C, "ItemDescription opcode must be 0x7C");
    }

    // -----------------------------------------------------------------------
    // Phase 3 — Market dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn market_place_buy_offer_creates_pending_offer() {
        let mut mgr = MarketManager::new();
        let bytes = handle_player_purchase(&mut mgr, 1, 300, 5, 2000);
        assert_eq!(bytes[0], 0xF2, "MarketDetail opcode must be 0xF2");
        let buy_count = bytes[1] as usize;
        assert_eq!(buy_count, 1, "One buy offer must be listed");
    }

    #[test]
    fn market_list_offers_returns_matching_buy_and_sell() {
        let mut mgr = MarketManager::new();
        handle_player_purchase(&mut mgr, 1, 400, 3, 900);
        handle_player_sale(&mut mgr, 2, 400, 2, 850);
        let bytes = handle_look_in_shop(&mgr, 400);
        assert_eq!(bytes[0], 0xF2, "MarketDetail opcode must be 0xF2");
        let buy_count = bytes[1] as usize;
        assert_eq!(buy_count, 1, "One buy offer");
        // sell_offers start after buy_offers (1 buy × 6 bytes) + count byte at index 8
        let sell_count_idx = 2 + buy_count * 6;
        assert_eq!(bytes[sell_count_idx], 1, "One sell offer");
    }

    #[test]
    fn market_close_session_removes_player_session() {
        let mut mgr = MarketManager::new();
        handle_player_purchase(&mut mgr, 5, 100, 1, 50);
        handle_close_shop(&mut mgr, 5);
        assert!(!mgr.has_session(5), "Session must be removed after close");
    }

    // -----------------------------------------------------------------------
    // Phase 4 — Guild integration
    // -----------------------------------------------------------------------

    #[test]
    fn guild_info_packet_sent_on_enter_game_when_guild_set() {
        let result = handle_player_guild_info("Knights of Tibia", "Leader");
        assert!(result.is_some(), "Must return Some when guild is set");
        let bytes = result.unwrap();
        assert_eq!(bytes[0], 0x58, "PlayerGuildInfo opcode must be 0x58");
    }

    #[test]
    fn guild_info_returns_none_when_no_guild() {
        let result = handle_player_guild_info("", "");
        assert!(
            result.is_none(),
            "Must return None when player has no guild"
        );
    }

    #[test]
    fn guild_channel_rejects_non_member_subscription() {
        let mut chat = ChatManager::new();
        chat.add_guild_channel(10, 42); // channel 10 belongs to guild 42
        chat.set_player_guild(1, 42); // player 1 is in guild 42
        chat.set_player_guild(2, 99); // player 2 is in a different guild

        assert!(
            chat.is_guild_member(10, 1),
            "Player 1 must be accepted (same guild)"
        );
        assert!(
            !chat.is_guild_member(10, 2),
            "Player 2 must be rejected (different guild)"
        );
        assert!(
            !chat.is_guild_member(10, 3),
            "Player 3 must be rejected (no guild)"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 1 — Player death dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn handle_on_player_death_returns_two_packets() {
        let mut state = GameState::new();
        let mut player = Player::new(1, "Hero", 1);
        player.set_max_health(100);
        player.set_health(100);
        state.add_player_entity(player);

        let packets = handle_on_player_death(1, &mut state);

        assert_eq!(packets.len(), 2, "Must return PlayerDeath + PlayerStats");
        assert_eq!(
            packets[0],
            vec![0x14],
            "First packet must be PlayerDeath (0x14)"
        );
        assert_eq!(
            packets[1][0], 0xA0,
            "Second packet must be PlayerStats (0xA0)"
        );
    }

    #[test]
    fn handle_on_player_death_unknown_player_returns_empty() {
        let mut state = GameState::new();
        let packets = handle_on_player_death(999, &mut state);
        assert!(packets.is_empty());
    }

    // -----------------------------------------------------------------------
    // Phase 2 — XP / level-up dispatch
    // -----------------------------------------------------------------------

    #[test]
    fn handle_grant_xp_level_up_emits_stats_and_text_message() {
        let mut state = GameState::new();
        let player = Player::new(1, "Hero", 1);
        state.add_player_entity(player);

        // Enough XP to reach level 2
        let packets = handle_grant_xp(1, xp_for_level(2), &mut state);

        assert_eq!(
            packets.len(),
            2,
            "Level-up must emit PlayerStats + TextMessage"
        );
        assert_eq!(packets[0][0], 0xA0, "First packet must be PlayerStats");
        assert_eq!(packets[1][0], 0xB4, "Second packet must be TextMessage");

        // Check level text contains "2"
        let text_len = u16::from_le_bytes([packets[1][2], packets[1][3]]) as usize;
        let text = std::str::from_utf8(&packets[1][4..4 + text_len]).unwrap();
        assert!(
            text.contains('2'),
            "Level-up message must mention new level"
        );
    }

    #[test]
    fn handle_grant_xp_no_level_up_returns_empty() {
        let mut state = GameState::new();
        let player = Player::new(1, "Hero", 1);
        state.add_player_entity(player);

        // Only 50 XP — not enough for level 2 (needs 100)
        let packets = handle_grant_xp(1, 50, &mut state);
        assert!(packets.is_empty(), "No packets when no level-up occurs");
    }

    // -----------------------------------------------------------------------
    // Phase 5 — Logout / player save
    // -----------------------------------------------------------------------

    #[test]
    fn save_player_persists_full_state_to_db() {
        let mut state = GameState::new();
        let io = IoLoginData::new();
        let mut db = LoginDb::new();

        let mut player = Player::new(1, "Alice", 1);
        player.set_health(80);
        player.set_mana(40);
        player.drain_stamina(100);
        state.add_player_entity(player);

        let result = handle_logout(1, 100, &mut state, &io, &mut db);

        assert!(result, "handle_logout must return true for known player");
        assert!(
            state.get_player_entity(1).is_none(),
            "Player must be removed from state"
        );

        let record = io.load_player(&db, "Alice").unwrap();
        assert_eq!(record.health, 80);
        assert_eq!(record.mana, 40);
    }

    // -----------------------------------------------------------------------
    // Task 6.3 — enter-world burst
    // -----------------------------------------------------------------------

    #[test]
    fn enter_world_burst_starts_with_0x0a_then_map_then_stats() {
        use forgottenserver_database::iologindata::PlayerLoginData;

        let world = World::new();
        let player = PlayerLoginData {
            name: "Hero".to_string(),
            level: 10,
            health: 200,
            healthmax: 200,
            mana: 100,
            manamax: 100,
            stamina: 2520,
            posx: 100,
            posy: 100,
            posz: 7,
        };
        let burst = build_enter_world_burst(&player, &world);

        assert_eq!(burst[0], 0x0A, "first byte must be enter-world ack");
        assert_eq!(burst[1], 0x64, "second byte must be map opcode");
        // Find 0xA0 somewhere after the map packet
        assert!(
            burst[2..].contains(&0xA0),
            "burst must contain PlayerStats opcode 0xA0"
        );
    }

    #[test]
    fn handle_logout_unknown_player_returns_false() {
        let mut state = GameState::new();
        let io = IoLoginData::new();
        let mut db = LoginDb::new();
        assert!(!handle_logout(999, 0, &mut state, &io, &mut db));
    }

    #[test]
    fn trade_accept_by_both_completes() {
        let mut trade_mgr = TradeManager::new();
        let trade_id = trade_mgr
            .open(
                1,
                2,
                forgottenserver_game::trade::TradeItem { type_id: 300 },
            )
            .unwrap();
        let r1 = handle_accept_trade(&mut trade_mgr, 1, trade_id);
        // After first accept, still pending
        assert_eq!(r1[0], 0xB4);

        let r2 = handle_accept_trade(&mut trade_mgr, 2, trade_id);
        // After both accept, completed
        assert_eq!(r2[0], 0xB4);
        let text_len = u16::from_le_bytes([r2[2], r2[3]]) as usize;
        let text = std::str::from_utf8(&r2[4..4 + text_len]).unwrap();
        assert!(
            text.contains("complete"),
            "Completion message must say 'complete'"
        );
    }
}
