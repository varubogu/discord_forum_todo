mod common;

use std::sync::Arc;

use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, ResponseTemplate};

use core_infra::SeaActiveMessageRepository;
use core_lib::domain::{ActiveMessage, ChannelId, GuildId, MessageId};
use core_lib::ports::ActiveMessageRepository;
use discord_task::ports::ActiveMessageService;
use discord_task_infra::TwilightActiveMessageService;

fn message_json(id: u64) -> serde_json::Value {
    serde_json::json!({
        "id": id.to_string(),
        "channel_id": "700",
        "author": {
            "id": "1",
            "username": "bot",
            "discriminator": "0000",
            "avatar": null,
            "public_flags": 0
        },
        "content": "page",
        "timestamp": "2026-01-01T00:00:00+00:00",
        "edited_timestamp": null,
        "tts": false,
        "mention_everyone": false,
        "mentions": [],
        "mention_roles": [],
        "attachments": [],
        "embeds": [],
        "pinned": false,
        "type": 0
    })
}

fn hash_a() -> Vec<u8> {
    vec![1u8; 32]
}

fn hash_b() -> Vec<u8> {
    vec![2u8; 32]
}

#[tokio::test]
async fn first_sync_posts_new_messages() {
    let (db, _c) = common::setup_db().await;
    let (client, server) = common::mock_client().await;

    Mock::given(method("POST"))
        .and(path_regex("/api/v10/channels/\\d+/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_json(801)))
        .mount(&server)
        .await;

    let repo = SeaActiveMessageRepository::new(Arc::new(db));
    let svc = TwilightActiveMessageService::new(client, repo);

    let ids = svc
        .sync_active_message(
            GuildId::new(9001),
            ChannelId::new(700),
            vec!["page one".to_string()],
            hash_a(),
        )
        .await
        .unwrap();

    assert_eq!(ids, vec![MessageId::new(801)]);
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn same_hash_skips_discord_calls() {
    let (db, _c) = common::setup_db().await;
    let (client, server) = common::mock_client().await;

    // Pre-insert a record with hash_a.
    let repo = SeaActiveMessageRepository::new(Arc::new(db));
    repo.upsert(ActiveMessage::new(
        GuildId::new(9002),
        ChannelId::new(700),
        vec![MessageId::new(802)],
        hash_a().try_into().unwrap(),
        common::now(),
    ))
    .await
    .unwrap();

    let svc = TwilightActiveMessageService::new(client, repo);
    let ids = svc
        .sync_active_message(
            GuildId::new(9002),
            ChannelId::new(700),
            vec!["page one".to_string()],
            hash_a(),
        )
        .await
        .unwrap();

    assert_eq!(ids, vec![MessageId::new(802)]);
    // No HTTP calls should have been made.
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 0);
}

#[tokio::test]
async fn sync_edits_existing_messages() {
    let (db, _c) = common::setup_db().await;
    let (client, server) = common::mock_client().await;

    Mock::given(method("PATCH"))
        .and(path_regex("/api/v10/channels/\\d+/messages/\\d+"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_json(803)))
        .mount(&server)
        .await;

    let repo = SeaActiveMessageRepository::new(Arc::new(db));
    repo.upsert(ActiveMessage::new(
        GuildId::new(9003),
        ChannelId::new(700),
        vec![MessageId::new(803)],
        hash_a().try_into().unwrap(),
        common::now(),
    ))
    .await
    .unwrap();

    let svc = TwilightActiveMessageService::new(client, repo);
    let ids = svc
        .sync_active_message(
            GuildId::new(9003),
            ChannelId::new(700),
            vec!["updated page".to_string()],
            hash_b(),
        )
        .await
        .unwrap();

    assert_eq!(ids, vec![MessageId::new(803)]);
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1); // PATCH only
}

#[tokio::test]
async fn sync_deletes_excess_messages() {
    let (db, _c) = common::setup_db().await;
    let (client, server) = common::mock_client().await;

    Mock::given(method("PATCH"))
        .and(path_regex("/api/v10/channels/\\d+/messages/\\d+"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_json(804)))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path_regex("/api/v10/channels/\\d+/messages/\\d+"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let repo = SeaActiveMessageRepository::new(Arc::new(db));
    repo.upsert(ActiveMessage::new(
        GuildId::new(9004),
        ChannelId::new(700),
        vec![MessageId::new(804), MessageId::new(805)],
        hash_a().try_into().unwrap(),
        common::now(),
    ))
    .await
    .unwrap();

    let svc = TwilightActiveMessageService::new(client, repo);
    let ids = svc
        .sync_active_message(
            GuildId::new(9004),
            ChannelId::new(700),
            vec!["single page".to_string()],
            hash_b(),
        )
        .await
        .unwrap();

    assert_eq!(ids.len(), 1);
    let requests = server.received_requests().await.unwrap();
    // 1 PATCH (edit first) + 1 DELETE (remove second)
    assert_eq!(requests.len(), 2);
}
