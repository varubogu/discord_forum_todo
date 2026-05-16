mod common;

use std::sync::Arc;

use core_infra::SeaActiveMessageRepository;
use core_lib::domain::{ActiveMessage, ChannelId, GuildId, MessageId};
use core_lib::ports::ActiveMessageRepository;

fn dummy_hash() -> [u8; 32] {
    [0u8; 32]
}

#[tokio::test]
async fn upsert_and_find() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaActiveMessageRepository::new(Arc::new(db));
    let now = common::now();

    let msg = ActiveMessage::new(
        GuildId(6001),
        ChannelId(7001),
        vec![MessageId(8001), MessageId(8002)],
        dummy_hash(),
        now,
    );

    repo.upsert(msg.clone()).await.unwrap();

    let found = repo.find(GuildId(6001)).await.unwrap().unwrap();
    assert_eq!(found.channel_id, ChannelId(7001));
    assert_eq!(found.message_ids, vec![MessageId(8001), MessageId(8002)]);
    assert_eq!(found.last_rendered_hash, dummy_hash());
}

#[tokio::test]
async fn upsert_overwrites_message_ids() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaActiveMessageRepository::new(Arc::new(db));
    let now = common::now();

    repo.upsert(ActiveMessage::new(
        GuildId(6002),
        ChannelId(7002),
        vec![MessageId(9001)],
        dummy_hash(),
        now,
    ))
    .await
    .unwrap();

    let updated_hash = [1u8; 32];
    repo.upsert(ActiveMessage::new(
        GuildId(6002),
        ChannelId(7002),
        vec![MessageId(9001), MessageId(9002)],
        updated_hash,
        now,
    ))
    .await
    .unwrap();

    let found = repo.find(GuildId(6002)).await.unwrap().unwrap();
    assert_eq!(found.message_ids.len(), 2);
    assert_eq!(found.last_rendered_hash, updated_hash);
}

#[tokio::test]
async fn delete_removes_record() {
    let (db, _c) = common::setup_db().await;
    let repo = SeaActiveMessageRepository::new(Arc::new(db));
    let now = common::now();

    repo.upsert(ActiveMessage::new(
        GuildId(6003),
        ChannelId(7003),
        vec![MessageId(1)],
        dummy_hash(),
        now,
    ))
    .await
    .unwrap();

    repo.delete(GuildId(6003)).await.unwrap();

    let found = repo.find(GuildId(6003)).await.unwrap();
    assert!(found.is_none());
}
