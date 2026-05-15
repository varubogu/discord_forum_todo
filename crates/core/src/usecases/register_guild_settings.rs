//! Apply a single guild-settings mutation. Each `/config set …` slash command
//! invokes this use case with one update payload, keeping audit-log entries
//! 1:1 with admin actions.

use crate::domain::{GuildId, GuildSettings, UserId};
use crate::ports::{Clock, GuildSettingsRepoError, GuildSettingsRepository, GuildSettingsUpdate};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegisterGuildSettingsError {
    #[error(transparent)]
    Repo(#[from] GuildSettingsRepoError),
}

pub struct RegisterGuildSettings<R, C>
where
    R: GuildSettingsRepository,
    C: Clock,
{
    repo: R,
    clock: C,
}

impl<R, C> RegisterGuildSettings<R, C>
where
    R: GuildSettingsRepository,
    C: Clock,
{
    pub fn new(repo: R, clock: C) -> Self {
        Self { repo, clock }
    }

    pub async fn execute(
        &self,
        guild_id: GuildId,
        update: GuildSettingsUpdate,
        actor: Option<UserId>,
    ) -> Result<GuildSettings, RegisterGuildSettingsError> {
        let now = self.clock.now();
        self.repo.ensure_exists(guild_id, now).await?;
        let updated = self.repo.apply(guild_id, update, actor, now).await?;
        Ok(updated)
    }
}
