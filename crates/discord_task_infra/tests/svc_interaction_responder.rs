mod common;

use twilight_model::id::Id;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, ResponseTemplate};

use discord_task::ports::InteractionResponder;
use discord_task_infra::TwilightInteractionResponder;

const APP_ID: u64 = 1;
const INTERACTION_ID: u64 = 12345;
const INTERACTION_TOKEN: &str = "fake-token";

#[tokio::test]
async fn respond_ephemeral_sets_ephemeral_flag() {
    let (client, server) = common::mock_client().await;

    Mock::given(method("POST"))
        .and(path_regex("/api/v10/interactions/\\d+/[^/]+/callback"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let responder = TwilightInteractionResponder::new(client, Id::new(APP_ID));
    responder
        .respond_ephemeral(INTERACTION_ID, INTERACTION_TOKEN, "Hello!")
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);

    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    // EPHEMERAL flag = 64
    assert_eq!(body["data"]["flags"], 64);
}

#[tokio::test]
async fn respond_public_has_no_ephemeral_flag() {
    let (client, server) = common::mock_client().await;

    Mock::given(method("POST"))
        .and(path_regex("/api/v10/interactions/\\d+/[^/]+/callback"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let responder = TwilightInteractionResponder::new(client, Id::new(APP_ID));
    responder
        .respond_public(INTERACTION_ID, INTERACTION_TOKEN, "Hello!")
        .await
        .unwrap();

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);

    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    let flags = body["data"]["flags"].as_u64().unwrap_or(0);
    // EPHEMERAL bit (64) must NOT be set
    assert_eq!(flags & 64, 0);
}
