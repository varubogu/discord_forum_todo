mod common;

use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, ResponseTemplate};

use core_lib::domain::GuildId;
use discord_task::ports::ChannelInspector;
use discord_task_infra::TwilightChannelInspector;

fn channels_json() -> serde_json::Value {
    serde_json::json!([
        {"id": "1", "type": 0, "name": "task-alpha", "position": 0, "permission_overwrites": []},
        {"id": "2", "type": 0, "name": "task-beta",  "position": 1, "permission_overwrites": []},
        {"id": "3", "type": 0, "name": "other",       "position": 2, "permission_overwrites": []}
    ])
}

#[tokio::test]
async fn count_matching_channels_filters_by_pattern() {
    let (client, server) = common::mock_client().await;

    Mock::given(method("GET"))
        .and(path_regex("/api/v10/guilds/\\d+/channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(channels_json()))
        .mount(&server)
        .await;

    let inspector = TwilightChannelInspector::new(client);
    let count = inspector
        .count_matching_channels(GuildId::new(1), "^task-")
        .await
        .unwrap();

    assert_eq!(count, 2);
}

#[tokio::test]
async fn count_with_invalid_regex_returns_zero() {
    let (client, server) = common::mock_client().await;

    // Mock is registered but should not be called for invalid regex.
    Mock::given(method("GET"))
        .and(path_regex("/api/v10/guilds/\\d+/channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(channels_json()))
        .mount(&server)
        .await;

    let inspector = TwilightChannelInspector::new(client);
    let count = inspector
        .count_matching_channels(GuildId::new(1), "[invalid(")
        .await
        .unwrap();

    assert_eq!(count, 0);
}
