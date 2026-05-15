use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};

use crate::domain::{GuildId, GuildSettings, UserId};
use crate::ports::{GuildSettingsRepoError, GuildSettingsRepository, GuildSettingsUpdate};

#[derive(Default)]
pub struct FakeGuildSettingsRepository {
    inner: Mutex<HashMap<GuildId, GuildSettings>>,
}

impl FakeGuildSettingsRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> HashMap<GuildId, GuildSettings> {
        self.inner
            .lock()
            .expect("FakeGuildSettings poisoned")
            .clone()
    }
}

impl GuildSettingsRepository for FakeGuildSettingsRepository {
    async fn find(
        &self,
        guild_id: GuildId,
    ) -> Result<Option<GuildSettings>, GuildSettingsRepoError> {
        let g = self.inner.lock().expect("FakeGuildSettings poisoned");
        Ok(g.get(&guild_id).cloned())
    }

    async fn ensure_exists(
        &self,
        guild_id: GuildId,
        now: DateTime<Utc>,
    ) -> Result<GuildSettings, GuildSettingsRepoError> {
        let mut g = self.inner.lock().expect("FakeGuildSettings poisoned");
        let entry = g
            .entry(guild_id)
            .or_insert_with(|| GuildSettings::empty(guild_id, now));
        Ok(entry.clone())
    }

    async fn apply(
        &self,
        guild_id: GuildId,
        update: GuildSettingsUpdate,
        actor: Option<UserId>,
        now: DateTime<Utc>,
    ) -> Result<GuildSettings, GuildSettingsRepoError> {
        let mut g = self.inner.lock().expect("FakeGuildSettings poisoned");
        let row = g
            .entry(guild_id)
            .or_insert_with(|| GuildSettings::empty(guild_id, now));
        match update {
            GuildSettingsUpdate::UserRole(v) => row.user_role_id = v,
            GuildSettingsUpdate::ForumChannel(v) => row.forum_channel_id = v,
            GuildSettingsUpdate::ActiveChannel(v) => row.active_channel_id = v,
            GuildSettingsUpdate::NotifyChannel(v) => row.notify_channel_id = v,
            GuildSettingsUpdate::WatchCategory(v) => row.watch_category_id = v,
            GuildSettingsUpdate::IdleThresholdNotStarted(v) => row.idle_threshold_not_started = v,
            GuildSettingsUpdate::IdleThresholdInProgress(v) => row.idle_threshold_in_progress = v,
            GuildSettingsUpdate::NotifyCooldown(v) => row.notify_cooldown = v,
            GuildSettingsUpdate::Locale(v) => row.locale = v,
        }
        row.updated_at = now;
        row.updated_by = actor;
        Ok(row.clone())
    }
}
