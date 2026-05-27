use forgottenserver_common::position::Position;
use forgottenserver_game::chat::{ChannelId, ChannelInfo, SpeakType};

use crate::map_description::TileDescription;

pub enum ServerPacket {
    FullMapDescription {
        player_pos: Position,
        tiles: Vec<TileDescription>,
    },
    /// 0xAC — list of available channels sent to client
    ChannelList { channels: Vec<ChannelInfo> },
    /// 0xAB — acknowledgement after joining a channel
    OpenChannel { channel_id: ChannelId, name: String },
    /// 0xAA — chat message delivered to receivers
    Talk {
        speaker: String,
        speak_type: SpeakType,
        channel_id: Option<ChannelId>,
        text: String,
    },
    /// 0xAF — open a private chat session
    OpenPrivateChannel { receiver: String },
    /// 0x84 — animated damage number displayed over creature
    DamageEffect { creature_id: u32, damage: i32 },
    /// 0x8C — creature health bar percentage
    CreatureHealth {
        creature_id: u32,
        health_percent: u8,
    },
    /// 0xB4 — text message displayed in the client message log
    TextMessage { message_type: u8, text: String },
    /// 0x8E — creature outfit changed (broadcast to nearby players)
    CreatureOutfit {
        creature_id: u32,
        look_type: u16,
        look_head: u8,
        look_body: u8,
        look_legs: u8,
        look_feet: u8,
        look_addons: u8,
    },
    /// 0xF0 — list of quests for the current player
    QuestList { quests: Vec<(u16, String, bool)> },
    /// 0xF1 — quest missions detail
    QuestDetails {
        quest_id: u16,
        name: String,
        missions: Vec<(String, String)>,
    },
    /// 0x8B — party membership update (leader changed or member joined/left)
    PartyUpdate { new_leader_id: u32 },
    /// 0x7D — items offered in a trade window
    TradeItems {
        your_type_id: u16,
        their_type_id: Option<u16>,
    },
    /// 0x7C — item description shown when inspecting a trade slot
    ItemDescription { type_id: u16, text: String },
    /// 0xF2 — market buy/sell offers for an item
    MarketDetail {
        buy_offers: Vec<(u32, u16)>,
        sell_offers: Vec<(u32, u16)>,
    },
    /// 0x58 — guild name + rank for the entering player
    PlayerGuildInfo { name: String, rank: String },
    /// 0x14 — player death notification
    PlayerDeath,
    /// 0xA0 — player stats update (HP, mana, level, stamina)
    PlayerStats {
        health: i32,
        max_health: i32,
        mana: i32,
        max_mana: i32,
        level: u32,
        stamina: u16,
    },
}

pub enum ClientPacket {
    GetChannels,
    OpenChannel {
        channel_id: ChannelId,
    },
    CloseChannel {
        channel_id: ChannelId,
    },
    Say {
        speak_type: SpeakType,
        text: String,
        channel_id: Option<ChannelId>,
    },
    OpenPrivateChannel {
        receiver: String,
    },
    /// Attack a creature by ID.
    Attack {
        creature_id: u32,
    },
    /// Use an item at a stack position.
    UseItem {
        item_id: u16,
        stack_pos: u8,
        index: u8,
    },
    /// Follow a creature by ID.
    Follow {
        creature_id: u32,
    },
    /// Update fight mode, chase mode and secure mode.
    FightModes {
        fight_mode: u8,
        chase_mode: u8,
        secure_mode: bool,
    },
    /// Walk along a pre-computed path (direction bytes).
    AutoWalk {
        path: Vec<u8>,
    },
    /// Change the player's visible outfit.
    SetOutfit {
        look_type: u16,
        look_head: u8,
        look_body: u8,
        look_legs: u8,
        look_feet: u8,
        look_addons: u8,
    },
    /// Remove a VIP entry by GUID.
    VipRemove {
        guid: u32,
    },
    /// Request list of available quests.
    Quests,
    /// Request mission details for a quest.
    QuestInfo {
        quest_id: u16,
    },
    /// Submit a bug report.
    BugReport {
        comment: String,
    },
    /// Client debug assertion.
    DebugAssert,
    /// Accept a party invite from `inviter_id`.
    JoinParty {
        inviter_id: u32,
    },
    /// Revoke a pending party invite for `player_id`.
    RevokePartyInvite {
        player_id: u32,
    },
    /// Transfer party leadership to `new_leader_id`.
    PassPartyLeadership {
        new_leader_id: u32,
    },
    /// Leave the current party.
    LeaveParty,
    /// Toggle shared experience in the party.
    SharePartyExperience {
        active: bool,
    },
    /// Initiate a trade with `target_player_id` offering item at `stack_pos`.
    RequestTrade {
        sprite_id: u16,
        stack_pos: u8,
        target_player_id: u32,
    },
    /// Inspect the item at `index` in the trade (counter_offer selects the other side).
    LookInTrade {
        counter_offer: bool,
        index: u8,
    },
    /// Accept the current trade.
    AcceptTrade,
    /// Cancel the current trade.
    CloseTrade,
    /// Buy an item from a market offer.
    PlayerPurchase {
        item_id: u16,
        amount: u16,
    },
    /// Sell an item via the market.
    PlayerSale {
        item_id: u16,
        amount: u16,
    },
    /// Browse market offers for an item.
    LookInShop {
        item_id: u16,
    },
    /// Close the market/shop session.
    CloseShop,
}

/// Encode a `ServerPacket` into its Tibia 8.6 wire representation.
pub fn encode(packet: &ServerPacket) -> Vec<u8> {
    match packet {
        ServerPacket::FullMapDescription { player_pos, tiles } => {
            encode_full_map_description(*player_pos, tiles)
        }
        ServerPacket::ChannelList { channels } => encode_channel_list(channels),
        ServerPacket::OpenChannel { channel_id, name } => encode_open_channel(*channel_id, name),
        ServerPacket::Talk {
            speaker,
            speak_type,
            channel_id,
            text,
        } => encode_talk(speaker, speak_type, *channel_id, text),
        ServerPacket::OpenPrivateChannel { receiver } => encode_open_private_channel(receiver),
        ServerPacket::DamageEffect {
            creature_id,
            damage,
        } => encode_damage_effect(*creature_id, *damage),
        ServerPacket::CreatureHealth {
            creature_id,
            health_percent,
        } => encode_creature_health(*creature_id, *health_percent),
        ServerPacket::TextMessage { message_type, text } => {
            encode_text_message(*message_type, text)
        }
        ServerPacket::CreatureOutfit {
            creature_id,
            look_type,
            look_head,
            look_body,
            look_legs,
            look_feet,
            look_addons,
        } => encode_creature_outfit(
            *creature_id,
            *look_type,
            *look_head,
            *look_body,
            *look_legs,
            *look_feet,
            *look_addons,
        ),
        ServerPacket::QuestList { quests } => encode_quest_list(quests),
        ServerPacket::QuestDetails {
            quest_id,
            name,
            missions,
        } => encode_quest_details(*quest_id, name, missions),
        ServerPacket::PartyUpdate { new_leader_id } => encode_party_update(*new_leader_id),
        ServerPacket::TradeItems {
            your_type_id,
            their_type_id,
        } => encode_trade_items(*your_type_id, *their_type_id),
        ServerPacket::ItemDescription { type_id, text } => encode_item_description(*type_id, text),
        ServerPacket::MarketDetail {
            buy_offers,
            sell_offers,
        } => encode_market_detail(buy_offers, sell_offers),
        ServerPacket::PlayerGuildInfo { name, rank } => encode_player_guild_info(name, rank),
        ServerPacket::PlayerDeath => encode_player_death(),
        ServerPacket::PlayerStats {
            health,
            max_health,
            mana,
            max_mana,
            level,
            stamina,
        } => encode_player_stats(*health, *max_health, *mana, *max_mana, *level, *stamina),
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn write_string(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    out.extend_from_slice(bytes);
}

// ---------------------------------------------------------------------------
// Channel packet encoders
// ---------------------------------------------------------------------------

fn encode_channel_list(channels: &[ChannelInfo]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xAC);
    out.push(channels.len() as u8);
    for ch in channels {
        out.extend_from_slice(&ch.id.to_le_bytes());
        write_string(&mut out, &ch.name);
    }
    out
}

fn encode_open_channel(channel_id: ChannelId, name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xAB);
    out.extend_from_slice(&channel_id.to_le_bytes());
    write_string(&mut out, name);
    out
}

fn encode_talk(
    speaker: &str,
    speak_type: &SpeakType,
    channel_id: Option<ChannelId>,
    text: &str,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xAA);
    write_string(&mut out, speaker);
    out.push(speak_type.to_byte());
    if let Some(cid) = channel_id {
        out.extend_from_slice(&cid.to_le_bytes());
    }
    write_string(&mut out, text);
    out
}

fn encode_open_private_channel(receiver: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xAF);
    write_string(&mut out, receiver);
    out
}

// ---------------------------------------------------------------------------
// New dispatch packet encoders
// ---------------------------------------------------------------------------

fn encode_damage_effect(creature_id: u32, damage: i32) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x84);
    out.extend_from_slice(&creature_id.to_le_bytes());
    out.extend_from_slice(&damage.to_le_bytes());
    out
}

fn encode_creature_health(creature_id: u32, health_percent: u8) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x8C);
    out.extend_from_slice(&creature_id.to_le_bytes());
    out.push(health_percent);
    out
}

fn encode_text_message(message_type: u8, text: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xB4);
    out.push(message_type);
    write_string(&mut out, text);
    out
}

fn encode_creature_outfit(
    creature_id: u32,
    look_type: u16,
    look_head: u8,
    look_body: u8,
    look_legs: u8,
    look_feet: u8,
    look_addons: u8,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x8E);
    out.extend_from_slice(&creature_id.to_le_bytes());
    out.extend_from_slice(&look_type.to_le_bytes());
    out.push(look_head);
    out.push(look_body);
    out.push(look_legs);
    out.push(look_feet);
    out.push(look_addons);
    out
}

fn encode_quest_list(quests: &[(u16, String, bool)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xF0);
    out.extend_from_slice(&(quests.len() as u16).to_le_bytes());
    for (id, name, completed) in quests {
        out.extend_from_slice(&id.to_le_bytes());
        write_string(&mut out, name);
        out.push(u8::from(*completed));
    }
    out
}

fn encode_quest_details(quest_id: u16, name: &str, missions: &[(String, String)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xF1);
    out.extend_from_slice(&quest_id.to_le_bytes());
    write_string(&mut out, name);
    out.push(missions.len() as u8);
    for (m_name, m_desc) in missions {
        write_string(&mut out, m_name);
        write_string(&mut out, m_desc);
    }
    out
}

// ---------------------------------------------------------------------------
// Party / Trade packet encoders
// ---------------------------------------------------------------------------

fn encode_party_update(new_leader_id: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x8B);
    out.extend_from_slice(&new_leader_id.to_le_bytes());
    out
}

fn encode_trade_items(your_type_id: u16, their_type_id: Option<u16>) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x7D);
    out.extend_from_slice(&your_type_id.to_le_bytes());
    let has_their = their_type_id.is_some() as u8;
    out.push(has_their);
    if let Some(id) = their_type_id {
        out.extend_from_slice(&id.to_le_bytes());
    }
    out
}

fn encode_item_description(type_id: u16, text: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x7C);
    out.extend_from_slice(&type_id.to_le_bytes());
    write_string(&mut out, text);
    out
}

fn encode_market_detail(buy_offers: &[(u32, u16)], sell_offers: &[(u32, u16)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xF2);
    out.push(buy_offers.len() as u8);
    for (price, amount) in buy_offers {
        out.extend_from_slice(&price.to_le_bytes());
        out.extend_from_slice(&amount.to_le_bytes());
    }
    out.push(sell_offers.len() as u8);
    for (price, amount) in sell_offers {
        out.extend_from_slice(&price.to_le_bytes());
        out.extend_from_slice(&amount.to_le_bytes());
    }
    out
}

fn encode_player_guild_info(name: &str, rank: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x58);
    write_string(&mut out, name);
    write_string(&mut out, rank);
    out
}

fn encode_player_death() -> Vec<u8> {
    vec![0x14]
}

fn encode_player_stats(
    health: i32,
    max_health: i32,
    mana: i32,
    max_mana: i32,
    level: u32,
    stamina: u16,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0xA0);
    out.extend_from_slice(&health.to_le_bytes());
    out.extend_from_slice(&max_health.to_le_bytes());
    out.extend_from_slice(&mana.to_le_bytes());
    out.extend_from_slice(&max_mana.to_le_bytes());
    out.extend_from_slice(&level.to_le_bytes());
    out.extend_from_slice(&stamina.to_le_bytes());
    out
}

// ---------------------------------------------------------------------------
// Map description encoder
// ---------------------------------------------------------------------------

fn encode_full_map_description(player_pos: Position, tiles: &[TileDescription]) -> Vec<u8> {
    let origin_x = (player_pos.x as i32 - 8).max(0) as u16;
    let origin_y = (player_pos.y as i32 - 6).max(0) as u16;

    let mut out = Vec::new();
    out.push(0x64);
    out.extend_from_slice(&origin_x.to_le_bytes());
    out.extend_from_slice(&origin_y.to_le_bytes());
    out.push(player_pos.z);

    let mut skip_count: u32 = 0;

    let flush_skip = |out: &mut Vec<u8>, count: u32| {
        if count > 0 {
            out.push(0xFF);
            out.push((count - 1) as u8);
        }
    };

    for tile in tiles {
        if tile.is_empty() {
            skip_count += 1;
        } else {
            flush_skip(&mut out, skip_count);
            skip_count = 0;
            encode_tile(&mut out, tile);
        }
    }
    flush_skip(&mut out, skip_count);
    out
}

fn encode_tile(out: &mut Vec<u8>, tile: &TileDescription) {
    let mut flags_byte: u8 = 0;
    if tile.flags.protection_zone {
        flags_byte |= 0x01;
    }
    if tile.flags.no_logout {
        flags_byte |= 0x02;
    }
    out.push(flags_byte);

    if let Some(ground) = &tile.ground {
        out.extend_from_slice(&ground.type_id.to_le_bytes());
    }
    for item in &tile.top_items {
        out.extend_from_slice(&item.type_id.to_le_bytes());
    }
    // u16 zero terminator
    out.push(0x00);
    out.push(0x00);
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_world::{World, WorldTile, WorldTileFlags};

    use crate::map_description::MapDescriptionBuilder;

    // -----------------------------------------------------------------------
    // Phase 3 — map description
    // -----------------------------------------------------------------------

    #[test]
    fn pz_flag_tile_encodes_tile_flags() {
        let mut world = World::new();
        let tile_pos = Position::new(100, 100, 7);
        world.set_tile(
            tile_pos,
            WorldTile {
                flags: WorldTileFlags {
                    protection_zone: true,
                    no_logout: false,
                },
                ..WorldTile::default()
            },
        );

        let builder = MapDescriptionBuilder::new(&world);
        let player_pos = Position::new(100, 100, 7);
        let tiles = builder.build(player_pos);

        let packet = ServerPacket::FullMapDescription { player_pos, tiles };
        let encoded = encode(&packet);

        assert_eq!(encoded[0], 0x64, "opcode must be 0x64");

        let flags_idx = 6 + 2;
        assert!(encoded.len() > flags_idx, "encoded output too short");
        let flags_byte = encoded[flags_idx];
        assert!(
            flags_byte & 0x01 != 0,
            "PZ flag bit must be set in flags_byte, got 0x{:02X}",
            flags_byte
        );
    }

    // -----------------------------------------------------------------------
    // New dispatch packet encoders
    // -----------------------------------------------------------------------

    #[test]
    fn damage_effect_opcode_is_0x84() {
        let encoded = encode(&ServerPacket::DamageEffect {
            creature_id: 1,
            damage: 20,
        });
        assert_eq!(encoded[0], 0x84);
        let cid = u32::from_le_bytes([encoded[1], encoded[2], encoded[3], encoded[4]]);
        assert_eq!(cid, 1);
    }

    #[test]
    fn creature_health_opcode_is_0x8c() {
        let encoded = encode(&ServerPacket::CreatureHealth {
            creature_id: 5,
            health_percent: 75,
        });
        assert_eq!(encoded[0], 0x8C);
        assert_eq!(encoded[5], 75);
    }

    #[test]
    fn text_message_opcode_is_0xb4() {
        let encoded = encode(&ServerPacket::TextMessage {
            message_type: 0x13,
            text: "hi".to_string(),
        });
        assert_eq!(encoded[0], 0xB4);
        assert_eq!(encoded[1], 0x13);
    }

    #[test]
    fn creature_outfit_opcode_is_0x8e() {
        let encoded = encode(&ServerPacket::CreatureOutfit {
            creature_id: 7,
            look_type: 128,
            look_head: 10,
            look_body: 20,
            look_legs: 30,
            look_feet: 40,
            look_addons: 0,
        });
        assert_eq!(encoded[0], 0x8E);
    }

    #[test]
    fn quest_list_opcode_is_0xf0() {
        let encoded = encode(&ServerPacket::QuestList {
            quests: vec![(1, "My Quest".to_string(), false)],
        });
        assert_eq!(encoded[0], 0xF0);
        let count = u16::from_le_bytes([encoded[1], encoded[2]]);
        assert_eq!(count, 1);
    }

    #[test]
    fn quest_details_opcode_is_0xf1() {
        let encoded = encode(&ServerPacket::QuestDetails {
            quest_id: 3,
            name: "Epic".to_string(),
            missions: vec![("M1".to_string(), "desc".to_string())],
        });
        assert_eq!(encoded[0], 0xF1);
    }
}
