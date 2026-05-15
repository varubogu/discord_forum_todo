use chrono::{Duration, TimeZone, Utc};

use core_lib::domain::{ChannelId, GuildId, RoleId, UserId};
use core_lib::ports::{Clock, GuildSettingsUpdate};
use core_lib::test_support::FakeGuildSettingsRepository;
use core_lib::usecases::RegisterGuildSettings;

#[derive(Clone)]
struct PinnedClock(chrono::DateTime<Utc>);
impl Clock for PinnedClock {
    fn now(&self) -> chrono::DateTime<Utc> {
        self.0
    }
}

#[tokio::test]
async fn applies_user_role_update() {
    let repo = FakeGuildSettingsRepository::new();
    let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
    let uc = RegisterGuildSettings::new(repo, PinnedClock(now));

    let after = uc
        .execute(
            GuildId(1),
            GuildSettingsUpdate::UserRole(Some(RoleId(99))),
            Some(UserId(42)),
        )
        .await
        .unwrap();

    assert_eq!(after.user_role_id, Some(RoleId(99)));
    assert_eq!(after.updated_by, Some(UserId(42)));
    assert_eq!(after.updated_at, now);
}

#[tokio::test]
async fn applies_multiple_updates_in_sequence() {
    let repo = FakeGuildSettingsRepository::new();
    let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
    let uc = RegisterGuildSettings::new(repo, PinnedClock(now));

    uc.execute(
        GuildId(1),
        GuildSettingsUpdate::UserRole(Some(RoleId(99))),
        None,
    )
    .await
    .unwrap();
    let after = uc
        .execute(
            GuildId(1),
            GuildSettingsUpdate::NotifyChannel(Some(ChannelId(7))),
            None,
        )
        .await
        .unwrap();

    assert_eq!(after.user_role_id, Some(RoleId(99)));
    assert_eq!(after.notify_channel_id, Some(ChannelId(7)));
    assert!(after.is_ready_for_notifications());
}

#[tokio::test]
async fn supports_clearing_with_none() {
    let repo = FakeGuildSettingsRepository::new();
    let now = Utc.with_ymd_and_hms(2026, 5, 14, 12, 0, 0).unwrap();
    let uc = RegisterGuildSettings::new(repo, PinnedClock(now));
    uc.execute(
        GuildId(1),
        GuildSettingsUpdate::NotifyCooldown(Some(Duration::hours(6))),
        None,
    )
    .await
    .unwrap();
    let after = uc
        .execute(GuildId(1), GuildSettingsUpdate::NotifyCooldown(None), None)
        .await
        .unwrap();
    assert!(after.notify_cooldown.is_none());
}
