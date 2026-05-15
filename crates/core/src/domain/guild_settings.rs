//! Per-guild bot configuration.

use crate::domain::ids::{CategoryId, ChannelId, GuildId, RoleId, UserId};
use chrono::{DateTime, Duration, Utc};

/// How long a task may stay in a given open state before it counts as stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdleThresholds {
    pub not_started: Duration,
    pub in_progress: Duration,
}

impl IdleThresholds {
    /// Default thresholds: 3 days each. Applied at use-case level when the
    /// guild has not configured a value yet.
    #[must_use]
    pub fn default_three_days() -> Self {
        Self {
            not_started: Duration::days(3),
            in_progress: Duration::days(3),
        }
    }
}

/// All persisted settings for a single guild. Most fields are optional so the
/// row can exist before the admin has finished `/setup`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildSettings {
    pub guild_id: GuildId,
    pub user_role_id: Option<RoleId>,
    pub forum_channel_id: Option<ChannelId>,
    pub active_channel_id: Option<ChannelId>,
    pub notify_channel_id: Option<ChannelId>,
    pub watch_category_id: Option<CategoryId>,
    pub idle_threshold_not_started: Option<Duration>,
    pub idle_threshold_in_progress: Option<Duration>,
    pub notify_cooldown: Option<Duration>,
    pub locale: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<UserId>,
}

impl GuildSettings {
    /// Build a brand-new row with all optional fields unset.
    #[must_use]
    pub fn empty(guild_id: GuildId, now: DateTime<Utc>) -> Self {
        Self {
            guild_id,
            user_role_id: None,
            forum_channel_id: None,
            active_channel_id: None,
            notify_channel_id: None,
            watch_category_id: None,
            idle_threshold_not_started: None,
            idle_threshold_in_progress: None,
            notify_cooldown: None,
            locale: None,
            updated_at: now,
            updated_by: None,
        }
    }

    /// Effective thresholds, applying the 3-day default when the guild has not
    /// chosen one.
    #[must_use]
    pub fn effective_thresholds(&self) -> IdleThresholds {
        let default = IdleThresholds::default_three_days();
        IdleThresholds {
            not_started: self
                .idle_threshold_not_started
                .unwrap_or(default.not_started),
            in_progress: self
                .idle_threshold_in_progress
                .unwrap_or(default.in_progress),
        }
    }

    /// Default notify cooldown is 24 hours.
    #[must_use]
    pub fn effective_notify_cooldown(&self) -> Duration {
        self.notify_cooldown.unwrap_or(Duration::hours(24))
    }

    /// A guild is "ready" once the minimum fields required for stale-task
    /// notifications are set. Other features (auto-task generation, channel
    /// counting) degrade gracefully when their inputs are missing.
    #[must_use]
    pub fn is_ready_for_notifications(&self) -> bool {
        self.user_role_id.is_some() && self.notify_channel_id.is_some()
    }

    /// Whether the guild can render the active-tasks message.
    #[must_use]
    pub fn is_ready_for_active_message(&self) -> bool {
        self.active_channel_id.is_some() && self.forum_channel_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t0() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn empty_settings_are_not_ready() {
        let s = GuildSettings::empty(GuildId(1), t0());
        assert!(!s.is_ready_for_notifications());
        assert!(!s.is_ready_for_active_message());
    }

    #[test]
    fn default_thresholds_apply() {
        let s = GuildSettings::empty(GuildId(1), t0());
        let th = s.effective_thresholds();
        assert_eq!(th.not_started, Duration::days(3));
        assert_eq!(th.in_progress, Duration::days(3));
        assert_eq!(s.effective_notify_cooldown(), Duration::hours(24));
    }

    #[test]
    fn ready_when_role_and_notify_channel_set() {
        let mut s = GuildSettings::empty(GuildId(1), t0());
        s.user_role_id = Some(RoleId(10));
        s.notify_channel_id = Some(ChannelId(20));
        assert!(s.is_ready_for_notifications());
    }
}
