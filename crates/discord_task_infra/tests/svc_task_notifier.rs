mod common;

use std::sync::Arc;

use chrono::Utc;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, ResponseTemplate};

use core_lib::domain::{ChannelId, GuildId, RoleId, Task, TaskId, TaskState, ThreadId};
use core_lib::i18n::Locale;
use core_lib::test_support::FakeTranslator;
use discord_task::ports::TaskNotifier;
use discord_task_infra::TwilightTaskNotifier;

fn dummy_task() -> Task {
    let now = Utc::now();
    Task {
        id: TaskId::new(1),
        guild_id: GuildId::new(1),
        project_id: None,
        thread_id: ThreadId::new(555),
        title: "Fix the bug".to_string(),
        state: TaskState::NotStarted,
        created_at: now,
        last_state_changed_at: now,
        last_notified_at: None,
        deleted_at: None,
    }
}

fn message_json() -> serde_json::Value {
    serde_json::json!({
        "id": "1001",
        "channel_id": "300",
        "author": {
            "id": "1",
            "username": "bot",
            "discriminator": "0000",
            "avatar": null,
            "public_flags": 0
        },
        "content": "notification",
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

#[tokio::test]
async fn notify_stale_sends_message_to_channel() {
    let (client, server) = common::mock_client().await;

    Mock::given(method("POST"))
        .and(path_regex("/api/v10/channels/\\d+/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_json()))
        .mount(&server)
        .await;

    // FakeTranslator echoes the key, so the message content will contain the
    // key "notification-stale-task" rather than a real translation. That is
    // fine — this test only verifies the HTTP call is made correctly.
    let notifier = TwilightTaskNotifier::new(client, Arc::new(FakeTranslator::new()));
    notifier
        .notify_stale(
            ChannelId::new(300),
            RoleId::new(400),
            &dummy_task(),
            &Locale::fallback(),
        )
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);

    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert!(
        body["content"].as_str().is_some(),
        "content field is present"
    );
}
