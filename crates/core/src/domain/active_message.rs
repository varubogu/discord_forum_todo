//! State of the bot-owned "active tasks" message(s) in each guild.

use crate::domain::ids::{ChannelId, GuildId, MessageId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveMessage {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    /// Ordered. First element is the header message. Subsequent elements are
    /// continuation pages used when content exceeds Discord limits.
    pub message_ids: Vec<MessageId>,
    /// SHA-256 of the last rendered payload. Used to skip identical edits.
    pub last_rendered_hash: [u8; 32],
    pub updated_at: DateTime<Utc>,
}

impl ActiveMessage {
    /// Build a new record on first creation.
    #[must_use]
    pub fn new(
        guild_id: GuildId,
        channel_id: ChannelId,
        message_ids: Vec<MessageId>,
        hash: [u8; 32],
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            guild_id,
            channel_id,
            message_ids,
            last_rendered_hash: hash,
            updated_at: now,
        }
    }
}
