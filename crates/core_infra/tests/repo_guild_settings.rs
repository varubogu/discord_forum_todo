mod common;

use std::sync::Arc;

use core_infra::SeaGuildSettingsRepository;
use core_lib::domain::{ChannelId, GuildId, RoleId};
use core_lib::ports::{GuildSettingsRepository, GuildSettingsUpdate};

#[tokio::test]
async fn ensure_exists_is_idempotent() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaGuildSettingsRepository::new(Arc::new(db));
    let guild = GuildId(1001);
    let now = common::now();

    let s1 = repo.ensure_exists(guild, now).await.unwrap();
    let s2 = repo.ensure_exists(guild, now).await.unwrap();

    assert_eq!(s1.guild_id, s2.guild_id);
    assert!(s1.user_role_id.is_none());
}

#[tokio::test]
async fn apply_user_role_updates_single_field() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaGuildSettingsRepository::new(Arc::new(db));
    let guild = GuildId(1002);
    let now = common::now();

    repo.ensure_exists(guild, now).await.unwrap();

    let updated = repo
        .apply(
            guild,
            GuildSettingsUpdate::UserRole(Some(RoleId(42))),
            None,
            now,
        )
        .await
        .unwrap();

    assert_eq!(updated.user_role_id, Some(RoleId(42)));
    assert!(updated.forum_channel_id.is_none());
}

#[tokio::test]
async fn apply_does_not_overwrite_other_fields() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaGuildSettingsRepository::new(Arc::new(db));
    let guild = GuildId(1003);
    let now = common::now();

    repo.ensure_exists(guild, now).await.unwrap();
    repo.apply(
        guild,
        GuildSettingsUpdate::UserRole(Some(RoleId(10))),
        None,
        now,
    )
    .await
    .unwrap();
    repo.apply(
        guild,
        GuildSettingsUpdate::NotifyChannel(Some(ChannelId(20))),
        None,
        now,
    )
    .await
    .unwrap();

    let s = repo.find(guild).await.unwrap().unwrap();
    assert_eq!(s.user_role_id, Some(RoleId(10)));
    assert_eq!(s.notify_channel_id, Some(ChannelId(20)));
}
