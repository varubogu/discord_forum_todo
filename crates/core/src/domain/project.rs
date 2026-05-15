//! Project entity. A project maps 1:1 to a forum channel where tasks live.

use crate::domain::ids::{ChannelId, GuildId};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId(pub i64);

impl ProjectId {
    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
    #[inline]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub guild_id: GuildId,
    pub forum_channel_id: ChannelId,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProject {
    pub guild_id: GuildId,
    pub forum_channel_id: ChannelId,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
