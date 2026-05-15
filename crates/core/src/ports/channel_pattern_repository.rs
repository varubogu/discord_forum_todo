//! `ChannelPatternRepository` — CRUD over per-guild channel-name patterns.

use crate::domain::{ChannelPattern, ChannelPatternId, GuildId, NewChannelPattern};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChannelPatternRepoError {
    #[error("pattern not found")]
    NotFound,
    #[error("a pattern named `{0}` already exists for this guild")]
    DuplicateName(String),
    #[error("underlying store: {0}")]
    Backend(String),
}

pub trait ChannelPatternRepository: Send + Sync {
    fn list(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<Vec<ChannelPattern>, ChannelPatternRepoError>> + Send;

    fn insert(
        &self,
        pattern: NewChannelPattern,
    ) -> impl std::future::Future<Output = Result<ChannelPattern, ChannelPatternRepoError>> + Send;

    fn delete_by_name(
        &self,
        guild_id: GuildId,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), ChannelPatternRepoError>> + Send;

    fn find_by_id(
        &self,
        id: ChannelPatternId,
    ) -> impl std::future::Future<Output = Result<Option<ChannelPattern>, ChannelPatternRepoError>> + Send;
}
