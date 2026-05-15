//! `ActiveMessageRepository` — persisted state of the active-tasks message.

use crate::domain::{ActiveMessage, GuildId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActiveMessageRepoError {
    #[error("underlying store: {0}")]
    Backend(String),
}

pub trait ActiveMessageRepository: Send + Sync {
    fn find(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<Option<ActiveMessage>, ActiveMessageRepoError>> + Send;

    fn upsert(
        &self,
        record: ActiveMessage,
    ) -> impl std::future::Future<Output = Result<(), ActiveMessageRepoError>> + Send;

    fn delete(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<(), ActiveMessageRepoError>> + Send;
}
