mod common;

use std::sync::Arc;

use core_infra::SeaChannelPatternRepository;
use core_lib::domain::{ChannelPatternKind, GuildId, NewChannelPattern};
use core_lib::ports::{ChannelPatternRepoError, ChannelPatternRepository};

fn new_pattern(guild: u64, name: &str) -> NewChannelPattern {
    NewChannelPattern {
        guild_id: GuildId(guild),
        name: name.to_owned(),
        kind: ChannelPatternKind::Trigger,
        pattern: format!("^{name}-"),
        created_at: common::now(),
    }
}

#[tokio::test]
async fn insert_and_list() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaChannelPatternRepository::new(Arc::new(db));

    repo.insert(new_pattern(7001, "alpha")).await.unwrap();
    repo.insert(new_pattern(7001, "beta")).await.unwrap();

    let list = repo.list(GuildId(7001)).await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn duplicate_name_returns_error() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaChannelPatternRepository::new(Arc::new(db));

    repo.insert(new_pattern(7002, "gamma")).await.unwrap();
    let err = repo.insert(new_pattern(7002, "gamma")).await.unwrap_err();

    assert!(
        matches!(err, ChannelPatternRepoError::DuplicateName(_)),
        "expected DuplicateName, got {err:?}"
    );
}

#[tokio::test]
async fn delete_by_name() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaChannelPatternRepository::new(Arc::new(db));

    repo.insert(new_pattern(7003, "delta")).await.unwrap();
    repo.delete_by_name(GuildId(7003), "delta").await.unwrap();

    let list = repo.list(GuildId(7003)).await.unwrap();
    assert_eq!(list, []);
}

#[tokio::test]
async fn delete_by_name_not_found_returns_error() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaChannelPatternRepository::new(Arc::new(db));

    let err = repo
        .delete_by_name(GuildId(7004), "nonexistent")
        .await
        .unwrap_err();

    assert!(
        matches!(err, ChannelPatternRepoError::NotFound),
        "expected NotFound, got {err:?}"
    );
}

#[tokio::test]
async fn find_by_id_returns_pattern() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaChannelPatternRepository::new(Arc::new(db));

    let inserted = repo.insert(new_pattern(7005, "epsilon")).await.unwrap();
    let found = repo.find_by_id(inserted.id).await.unwrap().unwrap();

    assert_eq!(found.name, "epsilon");
    assert_eq!(found.kind, ChannelPatternKind::Trigger);
}
