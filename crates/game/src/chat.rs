use std::collections::{HashMap, HashSet};

pub type ChannelId = u16;
pub type EntityId = u32;

/// Error returned when a player cannot join a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelJoinError {
    /// The channel does not exist.
    NotFound,
    /// The channel is private and the player has no invite.
    NotInvited,
    /// The channel requires guild membership.
    NotGuildMember,
    /// The channel requires party membership.
    NotPartyMember,
}

/// Error returned when a player cannot speak in a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TalkError {
    /// The channel does not exist.
    ChannelNotFound,
    /// The player is not a member of the channel.
    NotMember,
}

pub const CHANNEL_WORLD: ChannelId = 0;
pub const CHANNEL_TRADE: ChannelId = 1;
pub const CHANNEL_HELP: ChannelId = 2;

// ---------------------------------------------------------------------------
// ChatLuaCallbacks — Lua hook abstraction (Piece C of wire-parity change)
// ---------------------------------------------------------------------------

/// Optional callback trait that the scripting crate implements to wire the
/// chat channel's four Lua hooks without creating a `game → scripting` crate
/// cycle (`scripting` already depends on `game`).
///
/// Mirrors the C++ `ChatChannel::executeCanJoinEvent`,
/// `executeOnJoinEvent`, `executeOnLeaveEvent`, and `executeOnSpeakEvent`
/// in `forgottenserver/src/chat.cpp`. Each method receives the channel id
/// and the player id; `on_speak` additionally receives the raw speak-class
/// byte and the message text. Implementations look up the registered Lua
/// callback for the channel and dispatch through the script environment.
///
/// All methods are infallible; failures inside the Lua dispatcher are
/// expected to be logged by the implementation and surfaced as the
/// C++-equivalent default return value (false for `can_join`/`on_join`
/// per the C++ "call stack overflow" guard, and `None` for `on_speak`).
pub trait ChatLuaCallbacks: Send + Sync {
    /// Called before a player is added to a channel. Returning `false`
    /// prevents the join. Mirrors C++ `executeCanJoinEvent(const Player&)`.
    fn can_join(&self, channel_id: ChannelId, player_id: EntityId) -> bool;

    /// Called after a player has joined a channel. Returning `false`
    /// signals an error inside the Lua handler. Mirrors C++
    /// `executeOnJoinEvent(const Player&)`.
    fn on_join(&self, channel_id: ChannelId, player_id: EntityId) -> bool;

    /// Called when a player leaves a channel. Mirrors C++
    /// `executeOnLeaveEvent(const Player&)`. Return value is unused by
    /// callers but kept for parity (always returns `bool` in C++).
    fn on_leave(&self, channel_id: ChannelId, player_id: EntityId);

    /// Called when a player speaks in a channel.
    /// Mirrors C++ `executeOnSpeakEvent(player, type, message)`.
    ///
    /// Return semantics:
    /// - `None`: no handler is registered for this channel — the caller
    ///   should fall through to the default delivery path.
    /// - `Some(true)`: handler accepted the message (deliver as normal).
    /// - `Some(false)`: handler rejected the message (do not deliver).
    fn on_speak(
        &self,
        channel_id: ChannelId,
        player_id: EntityId,
        speak_class: u8,
        message: &str,
    ) -> Option<bool>;
}

/// Special channel ids matching C++ constants.
pub const CHANNEL_GUILD: ChannelId = 0x00; // resolved via guild map; use a sentinel outside normal range
pub const CHANNEL_PARTY: ChannelId = 0xFFFF; // resolved via party map
pub const CHANNEL_PRIVATE: ChannelId = 0xFFFE;

// Use non-overlapping real ids for the special channel buckets.
// In C++ these are separate maps; we replicate by using high ids as sentinels.
pub const GUILD_CHANNEL_BASE: ChannelId = 40_000; // guild channels start here
pub const PARTY_CHANNEL_BASE: ChannelId = 50_000; // party channels start here
pub const PRIVATE_CHANNEL_BASE: ChannelId = 100; // private channels 100..9999 (matches C++)

/// Tibia 8.6 speak_type byte mapping (see design.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeakType {
    Say,           // 0x01
    Whisper,       // 0x02
    Yell,          // 0x03
    Private,       // 0x05
    ChannelYellow, // 0x07
    ChannelOrange, // 0x08
}

impl SpeakType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Say),
            0x02 => Some(Self::Whisper),
            0x03 => Some(Self::Yell),
            0x05 => Some(Self::Private),
            0x07 => Some(Self::ChannelYellow),
            0x08 => Some(Self::ChannelOrange),
            _ => None,
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Say => 0x01,
            Self::Whisper => 0x02,
            Self::Yell => 0x03,
            Self::Private => 0x05,
            Self::ChannelYellow => 0x07,
            Self::ChannelOrange => 0x08,
        }
    }
}

/// Discriminant for the kind of access rules a channel uses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChannelKind {
    /// Anyone may join (normal public channels).
    Public,
    /// Only players in the same guild may join.
    Guild { guild_id: u32 },
    /// Only players in the same party may join.
    Party { party_id: u32 },
    /// Only the owner and explicitly invited players may join.
    Private { owner_id: EntityId },
}

#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub id: ChannelId,
    pub name: String,
    /// Public channels can be joined by anyone; private channels require an invite.
    pub is_public: bool,
}

// ---------------------------------------------------------------------------
// ChatChannel — full channel with membership tracking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ChatChannel {
    pub id: ChannelId,
    pub name: String,
    pub is_public: bool,
    pub kind: ChannelKind,
    pub members: HashSet<EntityId>,
    pub invite_list: HashSet<EntityId>,
    pub exclude_list: HashSet<EntityId>,
}

impl ChatChannel {
    pub fn new(id: ChannelId, name: impl Into<String>, is_public: bool) -> Self {
        ChatChannel {
            id,
            name: name.into(),
            is_public,
            kind: if is_public {
                ChannelKind::Public
            } else {
                ChannelKind::Private { owner_id: 0 }
            },
            members: HashSet::new(),
            invite_list: HashSet::new(),
            exclude_list: HashSet::new(),
        }
    }

    pub fn new_with_kind(id: ChannelId, name: impl Into<String>, kind: ChannelKind) -> Self {
        let is_public = matches!(kind, ChannelKind::Public);
        ChatChannel {
            id,
            name: name.into(),
            is_public,
            kind,
            members: HashSet::new(),
            invite_list: HashSet::new(),
            exclude_list: HashSet::new(),
        }
    }

    pub fn add_member(&mut self, player_id: EntityId) {
        self.members.insert(player_id);
    }

    pub fn remove_member(&mut self, player_id: EntityId) -> bool {
        self.members.remove(&player_id)
    }

    pub fn has_member(&self, player_id: EntityId) -> bool {
        self.members.contains(&player_id)
    }

    pub fn add_to_invite_list(&mut self, player_id: EntityId) {
        self.invite_list.insert(player_id);
    }

    pub fn remove_from_invite_list(&mut self, player_id: EntityId) -> bool {
        self.invite_list.remove(&player_id)
    }

    pub fn is_invited(&self, player_id: EntityId) -> bool {
        // owner is always considered invited (mirrors C++ PrivateChatChannel::isInvited)
        match &self.kind {
            ChannelKind::Private { owner_id } if *owner_id == player_id => true,
            _ => self.invite_list.contains(&player_id),
        }
    }

    pub fn add_to_exclude_list(&mut self, player_id: EntityId) {
        self.exclude_list.insert(player_id);
    }

    pub fn is_excluded(&self, player_id: EntityId) -> bool {
        self.exclude_list.contains(&player_id)
    }

    /// Returns the owner of the channel if it is a private channel, else 0.
    pub fn get_owner(&self) -> EntityId {
        match &self.kind {
            ChannelKind::Private { owner_id } => *owner_id,
            _ => 0,
        }
    }

    /// Deliver a message from `speaker` to every member. Returns `Err` if
    /// the speaker is not in the channel (mirrors C++ `ChatChannel::talk`).
    pub fn talk(
        &self,
        speaker_id: EntityId,
        speaker_name: &str,
        speak_type: SpeakType,
        text: &str,
    ) -> Result<Vec<(EntityId, ChatMessage)>, TalkError> {
        if !self.members.contains(&speaker_id) {
            return Err(TalkError::NotMember);
        }
        let msgs = self
            .members
            .iter()
            .map(|&pid| {
                (
                    pid,
                    ChatMessage {
                        speaker: speaker_name.to_string(),
                        speak_type: speak_type.clone(),
                        text: text.to_string(),
                    },
                )
            })
            .collect();
        Ok(msgs)
    }

    // -----------------------------------------------------------------------
    // Lua hook dispatchers (Piece C of wire-parity change)
    //
    // These mirror the four C++ `ChatChannel::execute*Event` methods. They
    // take the optional callbacks struct as a borrowed parameter rather than
    // owning it on `ChatChannel` so the trait abstraction stays cycle-free
    // (`scripting` already depends on `game`; the inverse cycle is forbidden).
    // The chat manager owns the callbacks; channels are pure data.
    // -----------------------------------------------------------------------

    /// Mirrors C++ `ChatChannel::executeCanJoinEvent`. When no callbacks are
    /// wired, defaults to `true` (allow join), matching the C++ early-return
    /// for `canJoinEvent == -1`.
    pub fn execute_can_join_event(
        &self,
        player_id: EntityId,
        callbacks: Option<&dyn ChatLuaCallbacks>,
    ) -> bool {
        match callbacks {
            Some(cb) => cb.can_join(self.id, player_id),
            None => true,
        }
    }

    /// Mirrors C++ `ChatChannel::executeOnJoinEvent`. When no callbacks are
    /// wired, defaults to `true` matching the C++ early-return for
    /// `onJoinEvent == -1`.
    pub fn execute_on_join_event(
        &self,
        player_id: EntityId,
        callbacks: Option<&dyn ChatLuaCallbacks>,
    ) -> bool {
        match callbacks {
            Some(cb) => cb.on_join(self.id, player_id),
            None => true,
        }
    }

    /// Mirrors C++ `ChatChannel::executeOnLeaveEvent`. When no callbacks are
    /// wired, this is a no-op. Note that the C++ method returns a `bool`,
    /// but no caller in `chat.cpp` ever inspects that value, so the Rust
    /// signature returns `()` to keep the contract minimal.
    pub fn execute_on_leave_event(
        &self,
        player_id: EntityId,
        callbacks: Option<&dyn ChatLuaCallbacks>,
    ) {
        if let Some(cb) = callbacks {
            cb.on_leave(self.id, player_id);
        }
    }

    /// Mirrors C++ `ChatChannel::executeOnSpeakEvent`. Returns:
    /// - `None` when no callbacks are wired — caller must fall through to
    ///   the normal delivery path.
    /// - `Some(true)` when the handler approves delivery.
    /// - `Some(false)` when the handler rejects delivery.
    ///
    /// `speak_class` is passed as a raw `u8` matching the on-the-wire byte
    /// the C++ code uses; converting to/from `SpeakType` is the caller's
    /// responsibility.
    pub fn execute_on_speak_event(
        &self,
        player_id: EntityId,
        speak_class: u8,
        message: &str,
        callbacks: Option<&dyn ChatLuaCallbacks>,
    ) -> Option<bool> {
        callbacks.and_then(|cb| cb.on_speak(self.id, player_id, speak_class, message))
    }

    /// Send a message to all members regardless of who sent it
    /// (mirrors C++ `ChatChannel::sendToAll`).
    pub fn send_to_all(&self, text: &str, speak_type: SpeakType) -> Vec<(EntityId, ChatMessage)> {
        self.members
            .iter()
            .map(|&pid| {
                (
                    pid,
                    ChatMessage {
                        speaker: String::new(),
                        speak_type: speak_type.clone(),
                        text: text.to_string(),
                    },
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub speaker: String,
    pub speak_type: SpeakType,
    pub text: String,
}

fn default_channels() -> Vec<ChannelInfo> {
    vec![
        ChannelInfo {
            id: CHANNEL_WORLD,
            name: "World".to_string(),
            is_public: true,
        },
        ChannelInfo {
            id: CHANNEL_TRADE,
            name: "Trade".to_string(),
            is_public: true,
        },
        ChannelInfo {
            id: CHANNEL_HELP,
            name: "Help".to_string(),
            is_public: true,
        },
    ]
}

pub struct ChatManager {
    subscribers: HashMap<ChannelId, HashSet<EntityId>>,
    channels: HashMap<ChannelId, ChatChannel>,
    guild_channels: HashMap<ChannelId, u32>, // channel_id → guild_id
    player_guilds: HashMap<EntityId, u32>,   // player_id → guild_id
    /// party_id → channel_id for party channels
    party_channels: HashMap<u32, ChannelId>,
    /// player_id → party_id
    player_parties: HashMap<EntityId, u32>,
    /// next id to assign for private channel slots (100..9999)
    next_private_id: ChannelId,
    /// Optional Lua callback dispatcher, installed by the scripting crate.
    /// Mirrors the per-channel `canJoinEvent`/`onJoinEvent`/`onLeaveEvent`/
    /// `onSpeakEvent` Lua refs in C++ `ChatChannel`. Stored on the manager
    /// (not per-channel) so the trait abstraction stays cycle-free.
    lua_callbacks: Option<Box<dyn ChatLuaCallbacks>>,
}

impl std::fmt::Debug for ChatManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatManager")
            .field("channels", &self.channels)
            .field("guild_channels", &self.guild_channels)
            .field("player_guilds", &self.player_guilds)
            .field("party_channels", &self.party_channels)
            .field("player_parties", &self.player_parties)
            .field("next_private_id", &self.next_private_id)
            .field(
                "lua_callbacks",
                &self
                    .lua_callbacks
                    .as_ref()
                    .map(|_| "<dyn ChatLuaCallbacks>"),
            )
            .finish()
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatManager {
    pub fn new() -> Self {
        ChatManager {
            subscribers: HashMap::new(),
            channels: HashMap::new(),
            guild_channels: HashMap::new(),
            player_guilds: HashMap::new(),
            party_channels: HashMap::new(),
            player_parties: HashMap::new(),
            next_private_id: PRIVATE_CHANNEL_BASE,
            lua_callbacks: None,
        }
    }

    /// Install the Lua callback dispatcher used by every channel for
    /// `execute*Event` hooks. Mirrors the role of the per-channel C++
    /// `canJoinEvent`/`onJoinEvent`/`onLeaveEvent`/`onSpeakEvent` Lua refs,
    /// but lifted to the manager so the trait abstraction stays cycle-free.
    /// The scripting crate's `ScriptingChatCallbacks` is the production
    /// implementor; tests can pass any `Box<dyn ChatLuaCallbacks>` stub.
    pub fn set_lua_callbacks(&mut self, callbacks: Box<dyn ChatLuaCallbacks>) {
        self.lua_callbacks = Some(callbacks);
    }

    /// Borrow the installed Lua callback dispatcher, if any. Channels'
    /// `execute_*` helpers take this borrow as a parameter.
    pub fn lua_callbacks(&self) -> Option<&dyn ChatLuaCallbacks> {
        self.lua_callbacks.as_deref()
    }

    // -----------------------------------------------------------------------
    // Channel management
    // -----------------------------------------------------------------------

    pub fn add_channel(&mut self, channel: ChatChannel) {
        self.channels.insert(channel.id, channel);
    }

    pub fn get_channel(&self, id: ChannelId) -> Option<&ChatChannel> {
        self.channels.get(&id)
    }

    pub fn get_channel_mut(&mut self, id: ChannelId) -> Option<&mut ChatChannel> {
        self.channels.get_mut(&id)
    }

    /// Create a guild channel for `guild_id`, returning the assigned channel id.
    /// Mirrors C++ `Chat::createChannel` for `CHANNEL_GUILD`.
    pub fn create_guild_channel(
        &mut self,
        guild_id: u32,
        guild_name: impl Into<String>,
    ) -> ChannelId {
        let channel_id =
            GUILD_CHANNEL_BASE + (guild_id as u16 % (PARTY_CHANNEL_BASE - GUILD_CHANNEL_BASE));
        let channel =
            ChatChannel::new_with_kind(channel_id, guild_name, ChannelKind::Guild { guild_id });
        self.channels.insert(channel_id, channel);
        self.guild_channels.insert(channel_id, guild_id);
        channel_id
    }

    /// Create a party channel for `party_id`, returning the assigned channel id.
    pub fn create_party_channel(
        &mut self,
        party_id: u32,
        party_name: impl Into<String>,
    ) -> ChannelId {
        if let Some(&existing) = self.party_channels.get(&party_id) {
            return existing;
        }
        let channel_id = PARTY_CHANNEL_BASE + (party_id as u16 % 10_000);
        let channel =
            ChatChannel::new_with_kind(channel_id, party_name, ChannelKind::Party { party_id });
        self.channels.insert(channel_id, channel);
        self.party_channels.insert(party_id, channel_id);
        channel_id
    }

    /// Create a private channel owned by `owner_id`, returning the channel id.
    /// Returns `None` if no free slot exists (mirrors C++ limit 100..9999).
    pub fn create_private_channel(
        &mut self,
        owner_id: EntityId,
        channel_name: impl Into<String>,
    ) -> Option<ChannelId> {
        // find a free slot in [next_private_id, 10_000)
        let mut id = self.next_private_id;
        while self.channels.contains_key(&id) {
            id += 1;
            if id >= 10_000 {
                return None; // no free slot
            }
        }
        self.next_private_id = id + 1;
        let channel =
            ChatChannel::new_with_kind(id, channel_name, ChannelKind::Private { owner_id });
        self.channels.insert(id, channel);
        Some(id)
    }

    /// Delete a channel by id. Returns true if it existed.
    /// Mirrors C++ `Chat::deleteChannel`.
    pub fn delete_channel(&mut self, channel_id: ChannelId) -> bool {
        if let Some(ch) = self.channels.remove(&channel_id) {
            // Remove from index maps
            self.guild_channels.remove(&channel_id);
            if let ChannelKind::Party { party_id } = ch.kind {
                self.party_channels.remove(&party_id);
            }
            true
        } else {
            false
        }
    }

    /// Join a channel. Returns `Err` if the channel does not exist or the
    /// player fails the access rules for that channel type.
    pub fn add_user_to_channel(
        &mut self,
        player_id: EntityId,
        channel_id: ChannelId,
    ) -> Result<(), ChannelJoinError> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(ChannelJoinError::NotFound)?;

        match &channel.kind {
            ChannelKind::Public => {}
            ChannelKind::Guild { guild_id } => {
                let gid = *guild_id;
                let player_guild = self.player_guilds.get(&player_id).copied();
                if player_guild != Some(gid) {
                    return Err(ChannelJoinError::NotGuildMember);
                }
            }
            ChannelKind::Party { party_id } => {
                let pid = *party_id;
                let player_party = self.player_parties.get(&player_id).copied();
                if player_party != Some(pid) {
                    return Err(ChannelJoinError::NotPartyMember);
                }
            }
            ChannelKind::Private { .. } => {
                if !channel.invite_list.contains(&player_id) {
                    // owner check
                    let is_owner = channel.get_owner() == player_id;
                    if !is_owner {
                        return Err(ChannelJoinError::NotInvited);
                    }
                }
            }
        }

        channel.add_member(player_id);
        Ok(())
    }

    /// Remove a player from a channel. Returns true if removed.
    /// If the removed player was the owner of a private channel, the channel
    /// is deleted (mirrors C++ `Chat::removeUserFromChannel`).
    pub fn remove_user_from_channel(&mut self, player_id: EntityId, channel_id: ChannelId) -> bool {
        let owner = self
            .channels
            .get(&channel_id)
            .map(|c| c.get_owner())
            .unwrap_or(0);

        let removed = self
            .channels
            .get_mut(&channel_id)
            .map(|c| c.remove_member(player_id))
            .unwrap_or(false);

        if removed && owner != 0 && owner == player_id {
            self.delete_channel(channel_id);
        }
        removed
    }

    /// Remove a player from every channel they belong to.
    /// Mirrors C++ `Chat::removeUserFromAllChannels`.
    pub fn remove_user_from_all_channels(&mut self, player_id: EntityId) {
        let channel_ids: Vec<ChannelId> = self.channels.keys().copied().collect();
        let mut to_delete = vec![];
        for cid in channel_ids {
            // `cid` came from `self.channels.keys()` a moment ago and nothing
            // mutates `self.channels` in this loop, so the lookup always succeeds.
            let ch = self
                .channels
                .get_mut(&cid)
                .expect("channel id collected from keys must still exist");
            ch.remove_member(player_id);
            ch.remove_from_invite_list(player_id);
            // if the player owned a private channel, schedule it for deletion
            if ch.get_owner() == player_id && player_id != 0 {
                to_delete.push(cid);
            }
        }
        for cid in to_delete {
            self.delete_channel(cid);
        }
    }

    pub fn has_user(&self, player_id: EntityId, channel_id: ChannelId) -> bool {
        self.channels
            .get(&channel_id)
            .map(|c| c.has_member(player_id))
            .unwrap_or(false)
    }

    /// Talk to a channel. The speaker must be a member.
    /// Mirrors C++ `Chat::talkToChannel`.
    pub fn talk_to_channel(
        &self,
        speaker_id: EntityId,
        speaker_name: &str,
        channel_id: ChannelId,
        speak_type: SpeakType,
        text: &str,
    ) -> Result<Vec<(EntityId, ChatMessage)>, TalkError> {
        let channel = self
            .channels
            .get(&channel_id)
            .ok_or(TalkError::ChannelNotFound)?;
        channel.talk(speaker_id, speaker_name, speak_type, text)
    }

    pub fn subscribe(&mut self, channel_id: ChannelId, player_id: EntityId) {
        self.subscribers
            .entry(channel_id)
            .or_default()
            .insert(player_id);
    }

    pub fn unsubscribe(&mut self, channel_id: ChannelId, player_id: EntityId) {
        if let Some(subs) = self.subscribers.get_mut(&channel_id) {
            subs.remove(&player_id);
        }
    }

    pub fn broadcast(
        &self,
        channel_id: ChannelId,
        sender: &str,
        speak_type: SpeakType,
        text: &str,
    ) -> Vec<(EntityId, ChatMessage)> {
        let subs = match self.subscribers.get(&channel_id) {
            Some(s) => s,
            None => return vec![],
        };
        subs.iter()
            .map(|&pid| {
                (
                    pid,
                    ChatMessage {
                        speaker: sender.to_string(),
                        speak_type: speak_type.clone(),
                        text: text.to_string(),
                    },
                )
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Guild channel management
    // -----------------------------------------------------------------------

    /// Register `channel_id` as belonging to `guild_id`.
    pub fn add_guild_channel(&mut self, channel_id: ChannelId, guild_id: u32) {
        self.guild_channels.insert(channel_id, guild_id);
    }

    /// Record that `player_id` is a member of `guild_id`.
    pub fn set_player_guild(&mut self, player_id: EntityId, guild_id: u32) {
        self.player_guilds.insert(player_id, guild_id);
    }

    /// Record that `player_id` is in party `party_id`.
    pub fn set_player_party(&mut self, player_id: EntityId, party_id: u32) {
        self.player_parties.insert(player_id, party_id);
    }

    /// Remove party membership for a player.
    pub fn remove_player_party(&mut self, player_id: EntityId) {
        self.player_parties.remove(&player_id);
    }

    /// Returns `true` only when the channel is a guild channel and the player
    /// belongs to that guild.
    pub fn is_guild_member(&self, channel_id: ChannelId, player_id: EntityId) -> bool {
        let Some(&channel_guild) = self.guild_channels.get(&channel_id) else {
            return false;
        };
        self.player_guilds.get(&player_id).copied() == Some(channel_guild)
    }

    // -----------------------------------------------------------------------
    // Channel list (mirrors C++ Chat::getChannelList)
    // -----------------------------------------------------------------------

    /// Return the list of channels accessible to a given player.
    /// - Always includes all public normal channels they can see.
    /// - Includes the guild channel iff the player is in a guild.
    /// - Includes the party channel iff the player is in a party.
    /// - Includes private channels where the player is invited/owner.
    pub fn get_channel_list(&self, player_id: EntityId) -> Vec<ChannelId> {
        let mut list = Vec::new();

        let player_guild = self.player_guilds.get(&player_id).copied();
        let player_party = self.player_parties.get(&player_id).copied();

        for (&cid, ch) in &self.channels {
            let accessible = match &ch.kind {
                ChannelKind::Public => true,
                ChannelKind::Guild { guild_id } => player_guild == Some(*guild_id),
                ChannelKind::Party { party_id } => player_party == Some(*party_id),
                ChannelKind::Private { owner_id } => {
                    *owner_id == player_id || ch.invite_list.contains(&player_id)
                }
            };
            if accessible {
                list.push(cid);
            }
        }
        list
    }

    pub fn available_channels(&self) -> Vec<ChannelInfo> {
        default_channels()
    }

    /// Look up the private channel owned by `owner_id`, if any.
    /// Mirrors C++ `Chat::getPrivateChannel`.
    pub fn get_private_channel(&self, owner_id: EntityId) -> Option<&ChatChannel> {
        self.channels
            .values()
            .find(|ch| matches!(ch.kind, ChannelKind::Private { owner_id: oid } if oid == owner_id))
    }

    /// Look up the guild channel for `guild_id`, if any.
    /// Mirrors C++ `Chat::getGuildChannelById`.
    pub fn get_guild_channel_by_id(&self, guild_id: u32) -> Option<&ChatChannel> {
        self.channels
            .values()
            .find(|ch| matches!(ch.kind, ChannelKind::Guild { guild_id: gid } if gid == guild_id))
    }

    /// Apply the guild-rank / channel-type rewrite that the C++
    /// `Chat::talkToChannel` performs before dispatching:
    /// - In a guild channel, rank level > 1 ⇒ ChannelOrange (officer)
    ///   else force ChannelYellow.
    /// - In a private/party channel, force ChannelYellow.
    /// - Otherwise pass-through.
    pub fn resolve_channel_talk_type(
        kind: &ChannelKind,
        speaker_guild_rank_level: Option<u32>,
        requested: SpeakType,
    ) -> SpeakType {
        match kind {
            ChannelKind::Guild { .. } => {
                if matches!(speaker_guild_rank_level, Some(level) if level > 1) {
                    SpeakType::ChannelOrange
                } else {
                    SpeakType::ChannelYellow
                }
            }
            ChannelKind::Party { .. } | ChannelKind::Private { .. } => SpeakType::ChannelYellow,
            ChannelKind::Public => requested,
        }
    }

    pub fn open_private(&self, _sender_id: EntityId, receiver_name: &str) -> String {
        receiver_name.to_string()
    }

    pub fn send_private(
        &self,
        sender: &str,
        receiver_id: EntityId,
        text: &str,
    ) -> (EntityId, ChatMessage) {
        (
            receiver_id,
            ChatMessage {
                speaker: sender.to_string(),
                speak_type: SpeakType::Private,
                text: text.to_string(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Phase 1
    // -----------------------------------------------------------------------

    #[test]
    fn subscribe_and_broadcast_sends_to_subscriber() {
        let mut chat = ChatManager::new();
        chat.subscribe(CHANNEL_WORLD, 1);
        let messages = chat.broadcast(CHANNEL_WORLD, "Alice", SpeakType::Say, "Hello");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, 1);
        assert_eq!(messages[0].1.text, "Hello");
        assert_eq!(messages[0].1.speaker, "Alice");
        assert_eq!(messages[0].1.speak_type, SpeakType::Say);
    }

    #[test]
    fn unsubscribe_removes_from_channel() {
        let mut chat = ChatManager::new();
        chat.subscribe(CHANNEL_WORLD, 1);
        chat.unsubscribe(CHANNEL_WORLD, 1);
        let messages = chat.broadcast(CHANNEL_WORLD, "Alice", SpeakType::Say, "Hello");
        assert!(messages.is_empty(), "No messages after unsubscribe");
    }

    // -----------------------------------------------------------------------
    // Phase 3
    // -----------------------------------------------------------------------

    #[test]
    fn say_yell_broadcasts_wider_range() {
        let mut chat = ChatManager::new();
        chat.subscribe(CHANNEL_WORLD, 1);
        chat.subscribe(CHANNEL_WORLD, 2);

        let say_msgs = chat.broadcast(CHANNEL_WORLD, "Alice", SpeakType::Say, "hello");
        let yell_msgs = chat.broadcast(CHANNEL_WORLD, "Alice", SpeakType::Yell, "HELLO");

        // Both broadcast to all world channel subscribers
        assert_eq!(say_msgs.len(), 2, "Say must reach all world subscribers");
        assert_eq!(yell_msgs.len(), 2, "Yell must reach all world subscribers");

        // speak_type byte is preserved so client renders correctly (yell = all-caps, wider display)
        assert!(say_msgs.iter().all(|(_, m)| m.speak_type == SpeakType::Say));
        assert!(yell_msgs
            .iter()
            .all(|(_, m)| m.speak_type == SpeakType::Yell));
    }

    // -----------------------------------------------------------------------
    // Phase 4
    // -----------------------------------------------------------------------

    #[test]
    fn say_private_routes_to_single_player() {
        let chat = ChatManager::new();
        let (receiver_id, msg) = chat.send_private("Alice", 99, "Secret");
        assert_eq!(
            receiver_id, 99,
            "Private message must be directed to receiver"
        );
        assert_eq!(msg.speak_type, SpeakType::Private);
        assert_eq!(msg.text, "Secret");
        assert_eq!(msg.speaker, "Alice");
    }

    // -----------------------------------------------------------------------
    // Phase 2 — available_channels
    // -----------------------------------------------------------------------

    #[test]
    fn get_channels_returns_default_channel_list() {
        let chat = ChatManager::new();
        let channels = chat.available_channels();
        assert!(channels.len() >= 2, "Must have at least 2 default channels");
        assert!(
            channels.iter().any(|c| c.id == CHANNEL_WORLD),
            "World channel required"
        );
        assert!(
            channels.iter().any(|c| c.id == CHANNEL_TRADE),
            "Trade channel required"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 9 — ChatChannel
    // -----------------------------------------------------------------------

    #[test]
    fn chat_channel_new_has_no_members() {
        let ch = ChatChannel::new(1, "Test", true);
        assert!(ch.members.is_empty());
    }

    #[test]
    fn add_member_and_has_member() {
        let mut ch = ChatChannel::new(1, "Test", true);
        ch.add_member(42);
        assert!(ch.has_member(42));
    }

    #[test]
    fn remove_member_works() {
        let mut ch = ChatChannel::new(1, "Test", true);
        ch.add_member(42);
        ch.remove_member(42);
        assert!(!ch.has_member(42));
    }

    #[test]
    fn invite_list_tracks_invited_players() {
        let mut ch = ChatChannel::new(1, "Test", false);
        ch.add_to_invite_list(7);
        assert!(ch.is_invited(7));
        assert!(!ch.is_invited(8));
    }

    #[test]
    fn exclude_list_tracks_excluded_players() {
        let mut ch = ChatChannel::new(1, "Test", true);
        ch.add_to_exclude_list(99);
        assert!(ch.is_excluded(99));
        assert!(!ch.is_excluded(100));
    }

    #[test]
    fn add_user_to_public_channel_succeeds() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new(10, "Public", true);
        mgr.add_channel(ch);
        assert!(mgr.add_user_to_channel(1, 10).is_ok());
        assert!(mgr.has_user(1, 10));
    }

    #[test]
    fn add_user_to_private_channel_fails_without_invite() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new(20, "Private", false);
        mgr.add_channel(ch);
        assert!(mgr.add_user_to_channel(1, 20).is_err());
    }

    #[test]
    fn add_user_to_private_channel_succeeds_with_invite() {
        let mut mgr = ChatManager::new();
        let mut ch = ChatChannel::new(20, "Private", false);
        ch.add_to_invite_list(1);
        mgr.add_channel(ch);
        assert!(mgr.add_user_to_channel(1, 20).is_ok());
        assert!(mgr.has_user(1, 20));
    }

    #[test]
    fn has_user_returns_correct_result() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new(30, "Test", true);
        mgr.add_channel(ch);
        assert!(!mgr.has_user(5, 30));
        mgr.add_user_to_channel(5, 30).unwrap();
        assert!(mgr.has_user(5, 30));
    }

    // -----------------------------------------------------------------------
    // Phase 12.5 — new tests added to match C++ behaviour
    // -----------------------------------------------------------------------

    // --- add_user / remove_user ---

    #[test]
    fn add_user_joins_channel_as_member() {
        let mut ch = ChatChannel::new(1, "General", true);
        ch.add_member(10);
        assert!(
            ch.has_member(10),
            "player should be member after add_member"
        );
    }

    #[test]
    fn remove_user_is_no_longer_member() {
        let mut ch = ChatChannel::new(1, "General", true);
        ch.add_member(10);
        let removed = ch.remove_member(10);
        assert!(removed, "remove_member should return true");
        assert!(
            !ch.has_member(10),
            "player should not be member after remove_member"
        );
    }

    #[test]
    fn remove_member_returns_false_if_not_present() {
        let mut ch = ChatChannel::new(1, "General", true);
        assert!(
            !ch.remove_member(999),
            "removing absent player should return false"
        );
    }

    // --- talk — message delivered to all members ---

    #[test]
    fn talk_delivers_to_all_members() {
        let mut ch = ChatChannel::new(5, "Party", true);
        ch.add_member(1);
        ch.add_member(2);
        ch.add_member(3);

        let result = ch.talk(1, "Alice", SpeakType::ChannelYellow, "Hello everyone");
        let msgs = result.expect("talk should succeed");

        assert_eq!(msgs.len(), 3, "message must reach all 3 members");
        assert!(msgs.iter().all(|(_, m)| m.text == "Hello everyone"));
        assert!(msgs.iter().all(|(_, m)| m.speaker == "Alice"));
        assert!(msgs
            .iter()
            .all(|(_, m)| m.speak_type == SpeakType::ChannelYellow));
    }

    #[test]
    fn talk_returns_error_if_speaker_not_member() {
        let ch = ChatChannel::new(5, "Party", true);
        // nobody added; outsider tries to speak
        let result = ch.talk(99, "Outsider", SpeakType::Say, "Hello");
        assert_eq!(result, Err(TalkError::NotMember));
    }

    #[test]
    fn talk_to_channel_via_manager_succeeds_for_member() {
        let mut mgr = ChatManager::new();
        let mut ch = ChatChannel::new(7, "World", true);
        ch.add_member(1);
        ch.add_member(2);
        mgr.add_channel(ch);

        let result = mgr.talk_to_channel(1, "Alice", 7, SpeakType::ChannelYellow, "Hi");
        assert!(result.is_ok());
        let msgs = result.unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn talk_to_channel_via_manager_fails_for_non_member() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new(7, "World", true);
        mgr.add_channel(ch);

        let result = mgr.talk_to_channel(99, "Outsider", 7, SpeakType::Say, "Hi");
        assert_eq!(result, Err(TalkError::NotMember));
    }

    #[test]
    fn talk_to_channel_fails_if_channel_not_found() {
        let mgr = ChatManager::new();
        let result = mgr.talk_to_channel(1, "Alice", 999, SpeakType::Say, "Hi");
        assert_eq!(result, Err(TalkError::ChannelNotFound));
    }

    // --- guild channel access ---

    #[test]
    fn guild_channel_rejects_non_guild_member() {
        let mut mgr = ChatManager::new();
        mgr.set_player_guild(1, 100); // player 1 → guild 100
        let ch = ChatChannel::new_with_kind(500, "Warriors", ChannelKind::Guild { guild_id: 200 });
        mgr.add_channel(ch);

        let result = mgr.add_user_to_channel(1, 500);
        assert_eq!(result, Err(ChannelJoinError::NotGuildMember));
    }

    #[test]
    fn guild_channel_accepts_guild_member() {
        let mut mgr = ChatManager::new();
        mgr.set_player_guild(1, 100); // player 1 → guild 100
        let ch = ChatChannel::new_with_kind(500, "Warriors", ChannelKind::Guild { guild_id: 100 });
        mgr.add_channel(ch);

        let result = mgr.add_user_to_channel(1, 500);
        assert!(result.is_ok(), "guild member must join guild channel");
        assert!(mgr.has_user(1, 500));
    }

    #[test]
    fn guild_channel_rejects_player_with_no_guild() {
        let mut mgr = ChatManager::new();
        // player 2 has no guild set
        let ch = ChatChannel::new_with_kind(501, "Warriors", ChannelKind::Guild { guild_id: 100 });
        mgr.add_channel(ch);

        let result = mgr.add_user_to_channel(2, 501);
        assert_eq!(result, Err(ChannelJoinError::NotGuildMember));
    }

    // --- party channel access ---

    #[test]
    fn party_channel_rejects_non_party_member() {
        let mut mgr = ChatManager::new();
        mgr.set_player_party(1, 10);
        let ch = ChatChannel::new_with_kind(600, "Party", ChannelKind::Party { party_id: 20 });
        mgr.add_channel(ch);

        let result = mgr.add_user_to_channel(1, 600);
        assert_eq!(result, Err(ChannelJoinError::NotPartyMember));
    }

    #[test]
    fn party_channel_accepts_party_member() {
        let mut mgr = ChatManager::new();
        mgr.set_player_party(1, 10);
        let ch = ChatChannel::new_with_kind(600, "Party", ChannelKind::Party { party_id: 10 });
        mgr.add_channel(ch);

        assert!(mgr.add_user_to_channel(1, 600).is_ok());
        assert!(mgr.has_user(1, 600));
    }

    // --- invite-only channel ---

    #[test]
    fn invite_only_channel_rejects_uninvited() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new_with_kind(200, "VIP", ChannelKind::Private { owner_id: 1 });
        mgr.add_channel(ch);

        // player 5 not invited
        let result = mgr.add_user_to_channel(5, 200);
        assert_eq!(result, Err(ChannelJoinError::NotInvited));
    }

    #[test]
    fn invite_only_channel_allows_invited_player() {
        let mut mgr = ChatManager::new();
        let mut ch = ChatChannel::new_with_kind(200, "VIP", ChannelKind::Private { owner_id: 1 });
        ch.add_to_invite_list(5);
        mgr.add_channel(ch);

        assert!(mgr.add_user_to_channel(5, 200).is_ok());
        assert!(mgr.has_user(5, 200));
    }

    #[test]
    fn private_channel_owner_is_always_considered_invited() {
        let ch = ChatChannel::new_with_kind(200, "VIP", ChannelKind::Private { owner_id: 42 });
        // owner not explicitly in invite_list, but is_invited should return true
        assert!(ch.is_invited(42), "owner must always be considered invited");
        assert!(!ch.is_invited(99));
    }

    #[test]
    fn owner_can_join_own_private_channel() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new_with_kind(
            201,
            "Alice's Channel",
            ChannelKind::Private { owner_id: 1 },
        );
        mgr.add_channel(ch);

        assert!(mgr.add_user_to_channel(1, 201).is_ok());
        assert!(mgr.has_user(1, 201));
    }

    // --- get_channel_list ---

    #[test]
    fn get_channel_list_returns_only_accessible_channels() {
        let mut mgr = ChatManager::new();

        // Public channel — everyone sees it
        mgr.add_channel(ChatChannel::new_with_kind(10, "World", ChannelKind::Public));

        // Guild channel for guild 100
        mgr.set_player_guild(1, 100);
        let gch = ChatChannel::new_with_kind(500, "Warriors", ChannelKind::Guild { guild_id: 100 });
        mgr.add_channel(gch);

        // Guild channel for guild 200 — player 1 should NOT see this
        let gch2 = ChatChannel::new_with_kind(501, "Mages", ChannelKind::Guild { guild_id: 200 });
        mgr.add_channel(gch2);

        let list = mgr.get_channel_list(1);
        assert!(list.contains(&10), "public channel must be visible");
        assert!(list.contains(&500), "own guild channel must be visible");
        assert!(
            !list.contains(&501),
            "other guild channel must not be visible"
        );
    }

    #[test]
    fn get_channel_list_includes_private_channel_if_invited() {
        let mut mgr = ChatManager::new();
        let mut priv_ch =
            ChatChannel::new_with_kind(150, "Bob's Channel", ChannelKind::Private { owner_id: 2 });
        priv_ch.add_to_invite_list(7); // player 7 is invited
        mgr.add_channel(priv_ch);

        let list_7 = mgr.get_channel_list(7);
        let list_9 = mgr.get_channel_list(9);

        assert!(
            list_7.contains(&150),
            "invited player must see private channel"
        );
        assert!(
            !list_9.contains(&150),
            "uninvited player must not see private channel"
        );
    }

    #[test]
    fn get_channel_list_includes_party_channel_for_party_member() {
        let mut mgr = ChatManager::new();
        mgr.set_player_party(3, 42);
        mgr.set_player_party(4, 99); // different party

        let pch = ChatChannel::new_with_kind(600, "Party", ChannelKind::Party { party_id: 42 });
        mgr.add_channel(pch);

        let list_3 = mgr.get_channel_list(3);
        let list_4 = mgr.get_channel_list(4);

        assert!(list_3.contains(&600));
        assert!(!list_4.contains(&600));
    }

    // --- create_channel / delete_channel ---

    #[test]
    fn create_guild_channel_registers_correctly() {
        let mut mgr = ChatManager::new();
        let cid = mgr.create_guild_channel(100, "Warriors");
        assert!(
            mgr.get_channel(cid).is_some(),
            "guild channel must exist after creation"
        );
        let ch = mgr.get_channel(cid).unwrap();
        assert_eq!(ch.kind, ChannelKind::Guild { guild_id: 100 });
    }

    #[test]
    fn create_party_channel_returns_same_id_if_called_twice() {
        let mut mgr = ChatManager::new();
        let id1 = mgr.create_party_channel(77, "Party");
        let id2 = mgr.create_party_channel(77, "Party");
        assert_eq!(id1, id2, "same party should reuse the same channel id");
    }

    #[test]
    fn create_private_channel_assigns_unique_ids() {
        let mut mgr = ChatManager::new();
        let id1 = mgr.create_private_channel(1, "Alice's Channel").unwrap();
        let id2 = mgr.create_private_channel(2, "Bob's Channel").unwrap();
        assert_ne!(id1, id2, "private channels must have unique ids");
    }

    #[test]
    fn delete_channel_removes_it() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new(40, "Temp", true);
        mgr.add_channel(ch);
        assert!(mgr.get_channel(40).is_some());
        let deleted = mgr.delete_channel(40);
        assert!(deleted, "delete_channel should return true");
        assert!(
            mgr.get_channel(40).is_none(),
            "channel must not exist after deletion"
        );
    }

    #[test]
    fn delete_channel_returns_false_for_nonexistent() {
        let mut mgr = ChatManager::new();
        assert!(!mgr.delete_channel(9999));
    }

    // --- remove_user_from_channel ---

    #[test]
    fn remove_user_from_channel_succeeds() {
        let mut mgr = ChatManager::new();
        let mut ch = ChatChannel::new(10, "Test", true);
        ch.add_member(5);
        mgr.add_channel(ch);

        let ok = mgr.remove_user_from_channel(5, 10);
        assert!(ok, "should return true when player is removed");
        assert!(!mgr.has_user(5, 10));
    }

    #[test]
    fn remove_user_from_channel_returns_false_if_not_member() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new(10, "Test", true);
        mgr.add_channel(ch);

        assert!(!mgr.remove_user_from_channel(5, 10));
    }

    #[test]
    fn removing_owner_from_private_channel_deletes_channel() {
        let mut mgr = ChatManager::new();
        let ch = ChatChannel::new_with_kind(
            200,
            "Alice's Channel",
            ChannelKind::Private { owner_id: 1 },
        );
        // Add owner as member first
        let mut ch2 = ch;
        ch2.add_member(1);
        mgr.add_channel(ch2);

        mgr.remove_user_from_channel(1, 200);
        assert!(
            mgr.get_channel(200).is_none(),
            "channel must be deleted when owner leaves"
        );
    }

    // --- remove_user_from_all_channels ---

    #[test]
    fn remove_user_from_all_channels_clears_membership() {
        let mut mgr = ChatManager::new();

        let mut ch1 = ChatChannel::new(10, "A", true);
        ch1.add_member(7);
        mgr.add_channel(ch1);

        let mut ch2 = ChatChannel::new(11, "B", true);
        ch2.add_member(7);
        mgr.add_channel(ch2);

        mgr.remove_user_from_all_channels(7);

        assert!(!mgr.has_user(7, 10), "player 7 must leave channel 10");
        assert!(!mgr.has_user(7, 11), "player 7 must leave channel 11");
    }

    #[test]
    fn remove_user_from_all_channels_deletes_owned_private_channels() {
        let mut mgr = ChatManager::new();
        let mut ch = ChatChannel::new_with_kind(
            150,
            "Owner's Channel",
            ChannelKind::Private { owner_id: 3 },
        );
        ch.add_member(3);
        mgr.add_channel(ch);

        mgr.remove_user_from_all_channels(3);

        assert!(
            mgr.get_channel(150).is_none(),
            "owned private channel must be deleted"
        );
    }

    // --- send_to_all ---

    #[test]
    fn send_to_all_reaches_every_member() {
        let mut ch = ChatChannel::new(1, "Announce", true);
        ch.add_member(1);
        ch.add_member(2);
        ch.add_member(3);

        let msgs = ch.send_to_all("Server restart in 5 minutes", SpeakType::ChannelOrange);
        assert_eq!(msgs.len(), 3);
        assert!(msgs
            .iter()
            .all(|(_, m)| m.text == "Server restart in 5 minutes"));
        assert!(msgs
            .iter()
            .all(|(_, m)| m.speak_type == SpeakType::ChannelOrange));
    }

    // --- ChannelKind equality / get_owner ---

    #[test]
    fn get_owner_returns_zero_for_public_channel() {
        let ch = ChatChannel::new(1, "World", true);
        assert_eq!(ch.get_owner(), 0);
    }

    #[test]
    fn get_owner_returns_owner_id_for_private_channel() {
        let ch = ChatChannel::new_with_kind(
            200,
            "Alice's Channel",
            ChannelKind::Private { owner_id: 42 },
        );
        assert_eq!(ch.get_owner(), 42);
    }

    // --- SpeakType byte roundtrip ---

    #[test]
    fn speak_type_byte_roundtrip() {
        let types = [
            SpeakType::Say,
            SpeakType::Whisper,
            SpeakType::Yell,
            SpeakType::Private,
            SpeakType::ChannelYellow,
            SpeakType::ChannelOrange,
        ];
        for t in &types {
            let b = t.to_byte();
            let back = SpeakType::from_byte(b).expect("valid byte");
            assert_eq!(*t, back);
        }
    }

    #[test]
    fn speak_type_from_byte_unknown_returns_none() {
        assert!(SpeakType::from_byte(0xFF).is_none());
    }

    // --- is_guild_member helper ---

    #[test]
    fn is_guild_member_false_for_non_guild_channel() {
        let mgr = ChatManager::new();
        assert!(!mgr.is_guild_member(999, 1));
    }

    #[test]
    fn is_guild_member_true_when_guild_matches() {
        let mut mgr = ChatManager::new();
        mgr.add_guild_channel(500, 100);
        mgr.set_player_guild(1, 100);
        assert!(mgr.is_guild_member(500, 1));
    }

    #[test]
    fn is_guild_member_false_when_guild_differs() {
        let mut mgr = ChatManager::new();
        mgr.add_guild_channel(500, 100);
        mgr.set_player_guild(1, 200);
        assert!(!mgr.is_guild_member(500, 1));
    }

    // -----------------------------------------------------------------------
    // Phase 9 audit — gap-closing tests
    // -----------------------------------------------------------------------

    #[test]
    fn chat_manager_default_matches_new() {
        // Default impl must produce an empty manager just like `new`.
        let default_mgr: ChatManager = Default::default();
        assert!(default_mgr.get_channel(0).is_none());
        assert!(default_mgr.get_channel_list(1).is_empty());
    }

    #[test]
    fn get_channel_mut_returns_mutable_reference() {
        let mut mgr = ChatManager::new();
        mgr.add_channel(ChatChannel::new(77, "Mutable", true));
        {
            let ch_mut = mgr.get_channel_mut(77).expect("must return mut ref");
            ch_mut.add_member(99);
        }
        assert!(
            mgr.has_user(99, 77),
            "mutation through get_channel_mut must persist"
        );
    }

    #[test]
    fn get_channel_mut_returns_none_for_unknown_id() {
        let mut mgr = ChatManager::new();
        assert!(mgr.get_channel_mut(12345).is_none());
    }

    #[test]
    fn create_private_channel_returns_none_when_all_slots_taken() {
        // Fill all private slots 100..9999 so create_private_channel must fail
        // (exercises the `id >= 10_000` return in the slot loop).
        let mut mgr = ChatManager::new();
        for slot in PRIVATE_CHANNEL_BASE..10_000u16 {
            mgr.channels.insert(
                slot,
                ChatChannel::new_with_kind(
                    slot,
                    "filler",
                    ChannelKind::Private {
                        owner_id: slot as EntityId,
                    },
                ),
            );
        }
        // next_private_id starts at PRIVATE_CHANNEL_BASE; loop must increment past 9999.
        assert!(
            mgr.create_private_channel(1, "Alice").is_none(),
            "should return None when no slot is free"
        );
    }

    #[test]
    fn create_private_channel_succeeds_after_releasing_slot() {
        // Re-using a slot after deletion: takes the first free id starting from
        // next_private_id. Mirrors C++ behaviour that uses the first free slot.
        let mut mgr = ChatManager::new();
        let id1 = mgr
            .create_private_channel(1, "Alice")
            .expect("first slot free");
        let id2 = mgr
            .create_private_channel(2, "Bob")
            .expect("second slot free");
        assert_ne!(id1, id2);
        assert!(mgr.delete_channel(id1));
        // Next channel should still get a new id (next_private_id advances).
        let id3 = mgr
            .create_private_channel(3, "Carol")
            .expect("third slot free");
        assert_ne!(id3, id1, "should pick a fresh slot, not reuse");
        assert_ne!(id3, id2);
    }

    #[test]
    fn delete_channel_cleans_up_party_index() {
        // Exercise the Party branch of delete_channel which removes the
        // party_id → channel_id mapping (line 373 in the original file).
        let mut mgr = ChatManager::new();
        let cid = mgr.create_party_channel(123, "Party");
        assert!(mgr.party_channels.contains_key(&123));
        assert!(mgr.delete_channel(cid));
        assert!(
            !mgr.party_channels.contains_key(&123),
            "party_channels map must be cleaned up after delete_channel"
        );
        // Creating again returns a fresh id (no reuse from stale index).
        let cid2 = mgr.create_party_channel(123, "Party");
        assert!(mgr.get_channel(cid2).is_some());
    }

    #[test]
    fn broadcast_returns_empty_when_channel_has_no_subscribers() {
        let mgr = ChatManager::new();
        // Channel id 9999 has never been subscribed to → subscribers.get returns None.
        let msgs = mgr.broadcast(9999, "Alice", SpeakType::Say, "hi");
        assert!(msgs.is_empty());
    }

    #[test]
    fn remove_player_party_clears_party_membership() {
        let mut mgr = ChatManager::new();
        mgr.set_player_party(1, 42);
        let ch = ChatChannel::new_with_kind(600, "Party", ChannelKind::Party { party_id: 42 });
        mgr.add_channel(ch);
        // Sanity: player 1 can join.
        assert!(mgr.add_user_to_channel(1, 600).is_ok());

        mgr.remove_player_party(1);

        // After removal player 1 is no longer a member of party 42.
        let mut mgr2 = ChatManager::new();
        mgr2.set_player_party(1, 42);
        mgr2.remove_player_party(1);
        let ch2 = ChatChannel::new_with_kind(601, "Party", ChannelKind::Party { party_id: 42 });
        mgr2.add_channel(ch2);
        assert_eq!(
            mgr2.add_user_to_channel(1, 601),
            Err(ChannelJoinError::NotPartyMember),
            "player without party must be rejected"
        );
    }

    #[test]
    fn open_private_echoes_receiver_name() {
        let mgr = ChatManager::new();
        assert_eq!(mgr.open_private(1, "Bob"), "Bob");
        assert_eq!(mgr.open_private(99, ""), "");
    }

    #[test]
    fn get_private_channel_finds_owned_channel() {
        let mut mgr = ChatManager::new();
        let cid = mgr.create_private_channel(7, "Alice's").expect("slot free");
        let ch = mgr
            .get_private_channel(7)
            .expect("owner must find own channel");
        assert_eq!(ch.id, cid);
        assert_eq!(ch.get_owner(), 7);
    }

    #[test]
    fn get_private_channel_returns_none_when_no_channel_owned() {
        let mgr = ChatManager::new();
        assert!(mgr.get_private_channel(123).is_none());
    }

    #[test]
    fn get_guild_channel_by_id_returns_correct_channel() {
        let mut mgr = ChatManager::new();
        let cid = mgr.create_guild_channel(50, "Warriors");
        let ch = mgr
            .get_guild_channel_by_id(50)
            .expect("must find guild channel");
        assert_eq!(ch.id, cid);
    }

    #[test]
    fn get_guild_channel_by_id_returns_none_for_unknown_guild() {
        let mgr = ChatManager::new();
        assert!(mgr.get_guild_channel_by_id(9999).is_none());
    }

    // --- resolve_channel_talk_type — mirrors C++ Chat::talkToChannel rewrite ---

    #[test]
    fn talk_type_in_guild_for_officer_becomes_orange() {
        let kind = ChannelKind::Guild { guild_id: 1 };
        // rank level 2 (officer) → ChannelOrange
        let out = ChatManager::resolve_channel_talk_type(&kind, Some(2), SpeakType::Say);
        assert_eq!(out, SpeakType::ChannelOrange);
    }

    #[test]
    fn talk_type_in_guild_for_member_becomes_yellow() {
        let kind = ChannelKind::Guild { guild_id: 1 };
        // rank level 1 (regular member) → ChannelYellow
        let out = ChatManager::resolve_channel_talk_type(&kind, Some(1), SpeakType::Say);
        assert_eq!(out, SpeakType::ChannelYellow);
    }

    #[test]
    fn talk_type_in_guild_with_no_rank_becomes_yellow() {
        let kind = ChannelKind::Guild { guild_id: 1 };
        let out = ChatManager::resolve_channel_talk_type(&kind, None, SpeakType::Whisper);
        assert_eq!(out, SpeakType::ChannelYellow);
    }

    #[test]
    fn talk_type_in_private_channel_forced_yellow() {
        let kind = ChannelKind::Private { owner_id: 1 };
        let out = ChatManager::resolve_channel_talk_type(&kind, Some(5), SpeakType::Say);
        assert_eq!(out, SpeakType::ChannelYellow);
    }

    #[test]
    fn talk_type_in_party_channel_forced_yellow() {
        let kind = ChannelKind::Party { party_id: 1 };
        let out = ChatManager::resolve_channel_talk_type(&kind, Some(5), SpeakType::Yell);
        assert_eq!(out, SpeakType::ChannelYellow);
    }

    #[test]
    fn talk_type_in_public_channel_passes_through() {
        let kind = ChannelKind::Public;
        let out = ChatManager::resolve_channel_talk_type(&kind, None, SpeakType::Whisper);
        assert_eq!(out, SpeakType::Whisper);
    }

    // -----------------------------------------------------------------------
    // Piece C — ChatLuaCallbacks hooks
    // -----------------------------------------------------------------------

    use std::cell::RefCell;
    use std::sync::Mutex;

    /// Per-call record: (op_name, channel_id, player_id, speak_class_or_zero, message).
    /// Aliased to silence `clippy::type_complexity` on the `RecordingCallbacks`
    /// field which would otherwise expose the full tuple.
    type CallRecord = (&'static str, ChannelId, EntityId, u8, String);

    /// Test-only stub recording every call so each parity test can assert
    /// on the C++-equivalent sequence of invocations.
    #[derive(Debug)]
    struct RecordingCallbacks {
        calls: Mutex<Vec<CallRecord>>,
        can_join_result: bool,
        on_join_result: bool,
        on_speak_result: Option<bool>,
    }

    impl RecordingCallbacks {
        fn allow() -> Self {
            Self {
                calls: Mutex::new(vec![]),
                can_join_result: true,
                on_join_result: true,
                on_speak_result: Some(true),
            }
        }

        fn reject() -> Self {
            Self {
                calls: Mutex::new(vec![]),
                can_join_result: false,
                on_join_result: false,
                on_speak_result: Some(false),
            }
        }

        fn calls(&self) -> Vec<CallRecord> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl ChatLuaCallbacks for RecordingCallbacks {
        fn can_join(&self, channel_id: ChannelId, player_id: EntityId) -> bool {
            self.calls
                .lock()
                .unwrap()
                .push(("can_join", channel_id, player_id, 0, String::new()));
            self.can_join_result
        }

        fn on_join(&self, channel_id: ChannelId, player_id: EntityId) -> bool {
            self.calls
                .lock()
                .unwrap()
                .push(("on_join", channel_id, player_id, 0, String::new()));
            self.on_join_result
        }

        fn on_leave(&self, channel_id: ChannelId, player_id: EntityId) {
            self.calls
                .lock()
                .unwrap()
                .push(("on_leave", channel_id, player_id, 0, String::new()));
        }

        fn on_speak(
            &self,
            channel_id: ChannelId,
            player_id: EntityId,
            speak_class: u8,
            message: &str,
        ) -> Option<bool> {
            self.calls.lock().unwrap().push((
                "on_speak",
                channel_id,
                player_id,
                speak_class,
                message.to_string(),
            ));
            self.on_speak_result
        }
    }

    // --- Default behaviour (no callbacks installed) ---

    #[test]
    fn execute_can_join_event_defaults_true_without_callbacks() {
        let ch = ChatChannel::new(7, "Test", true);
        assert!(
            ch.execute_can_join_event(42, None),
            "default canJoinEvent must be true (mirrors C++ canJoinEvent == -1 branch)"
        );
    }

    #[test]
    fn execute_on_join_event_defaults_true_without_callbacks() {
        let ch = ChatChannel::new(7, "Test", true);
        assert!(
            ch.execute_on_join_event(42, None),
            "default onJoinEvent must be true (mirrors C++ onJoinEvent == -1 branch)"
        );
    }

    #[test]
    fn execute_on_leave_event_is_noop_without_callbacks() {
        let ch = ChatChannel::new(7, "Test", true);
        // Must not panic; no observable effect.
        ch.execute_on_leave_event(42, None);
    }

    #[test]
    fn execute_on_speak_event_returns_none_without_callbacks() {
        let ch = ChatChannel::new(7, "Test", true);
        assert_eq!(
            ch.execute_on_speak_event(42, SpeakType::Say.to_byte(), "hi", None),
            None,
            "no handler installed → None (caller falls through to default delivery)"
        );
    }

    // --- Dispatch path (callbacks installed) ---

    #[test]
    fn execute_can_join_event_dispatches_to_callbacks() {
        let ch = ChatChannel::new(7, "Test", true);
        let cb = RecordingCallbacks::allow();
        assert!(ch.execute_can_join_event(99, Some(&cb)));
        assert_eq!(
            cb.calls(),
            vec![("can_join", 7, 99, 0, String::new())],
            "must dispatch exactly once with channel id and player id"
        );
    }

    #[test]
    fn execute_can_join_event_can_reject() {
        let ch = ChatChannel::new(7, "Test", true);
        let cb = RecordingCallbacks::reject();
        assert!(
            !ch.execute_can_join_event(99, Some(&cb)),
            "false return must propagate so join is blocked"
        );
    }

    #[test]
    fn execute_on_join_event_dispatches_to_callbacks() {
        let ch = ChatChannel::new(11, "Test", true);
        let cb = RecordingCallbacks::allow();
        assert!(ch.execute_on_join_event(55, Some(&cb)));
        assert_eq!(cb.calls(), vec![("on_join", 11, 55, 0, String::new())]);
    }

    #[test]
    fn execute_on_leave_event_dispatches_to_callbacks() {
        let ch = ChatChannel::new(11, "Test", true);
        let cb = RecordingCallbacks::allow();
        ch.execute_on_leave_event(55, Some(&cb));
        assert_eq!(cb.calls(), vec![("on_leave", 11, 55, 0, String::new())]);
    }

    #[test]
    fn execute_on_speak_event_dispatches_with_class_and_message() {
        let ch = ChatChannel::new(11, "Test", true);
        let cb = RecordingCallbacks::allow();
        let speak_class = SpeakType::ChannelYellow.to_byte();
        let result = ch.execute_on_speak_event(55, speak_class, "hello", Some(&cb));
        assert_eq!(result, Some(true));
        assert_eq!(
            cb.calls(),
            vec![("on_speak", 11, 55, speak_class, "hello".to_string())]
        );
    }

    #[test]
    fn execute_on_speak_event_can_reject() {
        let ch = ChatChannel::new(11, "Test", true);
        let cb = RecordingCallbacks::reject();
        assert_eq!(
            ch.execute_on_speak_event(55, SpeakType::Say.to_byte(), "spam", Some(&cb)),
            Some(false),
            "Some(false) must propagate so caller drops the message"
        );
    }

    // --- Manager-level set_lua_callbacks ---

    /// Boxed wrapper so we can install via `set_lua_callbacks` and then
    /// observe the recorded calls via a shared RefCell.
    struct BoxedRecorder {
        recorded: std::sync::Arc<Mutex<Vec<&'static str>>>,
    }

    impl ChatLuaCallbacks for BoxedRecorder {
        fn can_join(&self, _: ChannelId, _: EntityId) -> bool {
            self.recorded.lock().unwrap().push("can_join");
            true
        }
        fn on_join(&self, _: ChannelId, _: EntityId) -> bool {
            self.recorded.lock().unwrap().push("on_join");
            true
        }
        fn on_leave(&self, _: ChannelId, _: EntityId) {
            self.recorded.lock().unwrap().push("on_leave");
        }
        fn on_speak(&self, _: ChannelId, _: EntityId, _: u8, _: &str) -> Option<bool> {
            self.recorded.lock().unwrap().push("on_speak");
            Some(true)
        }
    }

    #[test]
    fn chat_manager_lua_callbacks_initially_none() {
        let mgr = ChatManager::new();
        assert!(
            mgr.lua_callbacks().is_none(),
            "fresh manager must have no callbacks installed"
        );
    }

    #[test]
    fn set_lua_callbacks_installs_dispatcher_used_by_channels() {
        let mut mgr = ChatManager::new();
        let recorded = std::sync::Arc::new(Mutex::new(vec![]));
        mgr.set_lua_callbacks(Box::new(BoxedRecorder {
            recorded: recorded.clone(),
        }));
        assert!(mgr.lua_callbacks().is_some());

        let ch = ChatChannel::new(1, "Test", true);
        assert!(ch.execute_can_join_event(7, mgr.lua_callbacks()));
        assert!(ch.execute_on_join_event(7, mgr.lua_callbacks()));
        ch.execute_on_leave_event(7, mgr.lua_callbacks());
        assert_eq!(
            ch.execute_on_speak_event(7, SpeakType::Say.to_byte(), "hi", mgr.lua_callbacks()),
            Some(true)
        );

        let calls = recorded.lock().unwrap().clone();
        assert_eq!(calls, vec!["can_join", "on_join", "on_leave", "on_speak"]);
    }

    #[test]
    fn chat_manager_debug_does_not_panic_with_lua_callbacks_installed() {
        // The custom Debug impl must render the boxed-trait callback slot.
        let mut mgr = ChatManager::new();
        mgr.set_lua_callbacks(Box::new(BoxedRecorder {
            recorded: std::sync::Arc::new(Mutex::new(vec![])),
        }));
        let s = format!("{:?}", mgr);
        assert!(s.contains("ChatManager"));
        assert!(s.contains("lua_callbacks"));
    }

    // Sanity: the unused std::cell::RefCell import the file already had
    // is still in scope; touch it once so clippy doesn't flag this test
    // module's import block.
    #[test]
    fn refcell_in_test_module_compiles() {
        let cell: RefCell<u32> = RefCell::new(0);
        *cell.borrow_mut() = 1;
        assert_eq!(*cell.borrow(), 1);
    }
}
