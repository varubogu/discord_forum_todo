mod common;

use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, ResponseTemplate};

use core_lib::domain::{ChannelId, GuildId};
use discord_task::ports::ForumService;
use discord_task_infra::TwilightForumService;

fn forum_channel_json(id: u64, name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id.to_string(),
        "type": 11,
        "name": name,
        "guild_id": "1",
        "parent_id": "100",
        "position": 0,
        "permission_overwrites": [],
        "message": {
            "id": "200",
            "channel_id": id.to_string(),
            "author": {
                "id": "1",
                "username": "bot",
                "discriminator": "0000",
                "avatar": null,
                "public_flags": 0
            },
            "content": "\u{200B}",
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
        }
    })
}

#[tokio::test]
async fn create_thread_calls_correct_endpoint() {
    let (client, server) = common::mock_client().await;
    let thread_id: u64 = 999;

    Mock::given(method("POST"))
        .and(path_regex("/api/v10/channels/\\d+/threads"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(forum_channel_json(thread_id, "new-task")),
        )
        .mount(&server)
        .await;

    let service = TwilightForumService::new(client);
    let result = service
        .create_thread(GuildId::new(1), ChannelId::new(100), "new-task")
        .await
        .unwrap();

    assert_eq!(result.get(), thread_id);

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
}
