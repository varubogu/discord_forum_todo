mod common;

use core_lib::domain::GuildId;
use core_lib::ports::FakeClock;
use core_lib::ports::GuildSettingsUpdate;
use core_lib::test_support::FakeGuildSettingsRepository;
use discord_task::commands::{ConfigSetHandler, ConfigSetInput};

use common::FakeInteractionResponder;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

#[tokio::test]
async fn config_set_replies_ephemerally() {
    let repo = FakeGuildSettingsRepository::new();
    let clock = FakeClock::new(now());
    let responder = FakeInteractionResponder::default();

    let handler = ConfigSetHandler::new(repo, clock, responder);
    handler
        .handle(ConfigSetInput {
            guild_id: GuildId(1001),
            interaction_id: 42,
            interaction_token: "tok".to_string(),
            update: GuildSettingsUpdate::Locale(Some("ja-JP".to_string())),
            actor: None,
        })
        .await
        .unwrap();

    let responses = handler.responder.all();
    assert_eq!(responses.len(), 1);
    assert!(responses[0].ephemeral, "config set must reply ephemerally");
    assert_eq!(responses[0].interaction_id, 42);
}

#[tokio::test]
async fn config_set_is_idempotent() {
    // Calling config set twice for the same guild should both succeed.
    let repo = FakeGuildSettingsRepository::new();
    let clock = FakeClock::new(now());
    let responder = FakeInteractionResponder::default();

    let handler = ConfigSetHandler::new(repo, clock, responder);

    for _ in 0..2 {
        handler
            .handle(ConfigSetInput {
                guild_id: GuildId(2002),
                interaction_id: 1,
                interaction_token: "t".to_string(),
                update: GuildSettingsUpdate::Locale(Some("en-US".to_string())),
                actor: None,
            })
            .await
            .expect("both invocations must succeed");
    }

    assert_eq!(
        handler.responder.all().len(),
        2,
        "two calls must each produce a response"
    );
}
