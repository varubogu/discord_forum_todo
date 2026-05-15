//! Per-guild channel-name patterns used for counting and auto-task generation.

use crate::domain::ids::GuildId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelPatternId(pub i64);

impl ChannelPatternId {
    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelPatternKind {
    /// Channels matching this pattern get a forum task created automatically
    /// on `ChannelCreate`.
    Trigger,
    /// Channels matching this pattern are tallied by the `/count` command.
    Counter,
}

impl ChannelPatternKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ChannelPatternKind::Trigger => "trigger",
            ChannelPatternKind::Counter => "counter",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "trigger" => Some(Self::Trigger),
            "counter" => Some(Self::Counter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPattern {
    pub id: ChannelPatternId,
    pub guild_id: GuildId,
    pub name: String,
    pub kind: ChannelPatternKind,
    pub pattern: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewChannelPattern {
    pub guild_id: GuildId,
    pub name: String,
    pub kind: ChannelPatternKind,
    pub pattern: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_round_trip() {
        for k in [ChannelPatternKind::Trigger, ChannelPatternKind::Counter] {
            assert_eq!(ChannelPatternKind::parse(k.as_str()), Some(k));
        }
    }
}
