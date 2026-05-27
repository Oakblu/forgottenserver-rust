use forgottenserver_game::chat::{ChannelId, EntityId};

/// Tracks which channels a connected player currently has open.
pub struct ChannelSession {
    pub player_id: EntityId,
    open_channels: Vec<ChannelId>,
}

impl ChannelSession {
    pub fn new(player_id: EntityId) -> Self {
        ChannelSession {
            player_id,
            open_channels: Vec::new(),
        }
    }

    pub fn add_channel(&mut self, channel_id: ChannelId) {
        if !self.open_channels.contains(&channel_id) {
            self.open_channels.push(channel_id);
        }
    }

    pub fn remove_channel(&mut self, channel_id: ChannelId) {
        self.open_channels.retain(|&id| id != channel_id);
    }

    pub fn open_channels(&self) -> &[ChannelId] {
        &self.open_channels
    }
}
