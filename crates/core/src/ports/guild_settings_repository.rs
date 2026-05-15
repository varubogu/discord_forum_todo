//! `GuildSettingsRepository` — per-guild configuration.

use crate::domain::{GuildId, GuildSettings};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuildSettingsRepoError {
    #[error("underlying store: {0}")]
    Backend(String),
}

/// Atomic mutation of a single field. The full struct is intentionally not
/// updated in one call; each setting has its own setter so audit log entries
/// are 1:1 with admin actions.
#[derive(Debug, Clone)]
pub enum GuildSettingsUpdate {
    UserRole(Option<crate::domain::RoleId>),
    ForumChannel(Option<crate::domain::ChannelId>),
    ActiveChannel(Option<crate::domain::ChannelId>),
    NotifyChannel(Option<crate::domain::ChannelId>),
    WatchCategory(Option<crate::domain::CategoryId>),
    IdleThresholdNotStarted(Option<chrono::Duration>),
    IdleThresholdInProgress(Option<chrono::Duration>),
    NotifyCooldown(Option<chrono::Duration>),
    Locale(Option<String>),
}

pub trait GuildSettingsRepository: Send + Sync {
    /// Find the row for `guild_id`. Returns `None` if the guild has never been
    /// touched.
    fn find(
        &self,
        guild_id: GuildId,
    ) -> impl std::future::Future<Output = Result<Option<GuildSettings>, GuildSettingsRepoError>> + Send;

    /// Ensure a row exists. Idempotent.
    fn ensure_exists(
        &self,
        guild_id: GuildId,
        now: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<GuildSettings, GuildSettingsRepoError>> + Send;

    /// Apply a single mutation. The implementation also bumps `updated_at`
    /// and records `updated_by` when supplied.
    fn apply(
        &self,
        guild_id: GuildId,
        update: GuildSettingsUpdate,
        actor: Option<crate::domain::UserId>,
        now: DateTime<Utc>,
    ) -> impl std::future::Future<Output = Result<GuildSettings, GuildSettingsRepoError>> + Send;
}
