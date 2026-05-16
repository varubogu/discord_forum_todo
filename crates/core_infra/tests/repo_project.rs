mod common;

use std::sync::Arc;

use core_infra::SeaProjectRepository;
use core_lib::domain::{ChannelId, GuildId, NewProject};
use core_lib::ports::ProjectRepository;

#[tokio::test]
async fn upsert_creates_and_returns_id() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaProjectRepository::new(Arc::new(db));

    let p = repo
        .upsert(NewProject {
            guild_id: GuildId(2001),
            forum_channel_id: ChannelId(3001),
            name: "Alpha".into(),
            created_at: common::now(),
        })
        .await
        .unwrap();

    assert_eq!(p.guild_id, GuildId(2001));
    assert_eq!(p.forum_channel_id, ChannelId(3001));
    assert_eq!(p.name, "Alpha");
    assert!(p.id.get() > 0);
}

#[tokio::test]
async fn upsert_is_idempotent_on_forum_channel() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaProjectRepository::new(Arc::new(db));
    let now = common::now();

    let p1 = repo
        .upsert(NewProject {
            guild_id: GuildId(2002),
            forum_channel_id: ChannelId(3002),
            name: "Beta".into(),
            created_at: now,
        })
        .await
        .unwrap();

    let p2 = repo
        .upsert(NewProject {
            guild_id: GuildId(2002),
            forum_channel_id: ChannelId(3002),
            name: "BetaRenamed".into(),
            created_at: now,
        })
        .await
        .unwrap();

    assert_eq!(p1.id, p2.id);
    assert_eq!(p2.name, "BetaRenamed");
}

#[tokio::test]
async fn find_by_forum_returns_none_when_missing() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaProjectRepository::new(Arc::new(db));

    let result = repo
        .find_by_forum(GuildId(9999), ChannelId(9999))
        .await
        .unwrap();

    assert!(result.is_none());
}
