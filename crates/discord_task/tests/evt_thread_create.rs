mod common;

use core_lib::domain::ThreadId;
use core_lib::ports::{FakeClock, TaskRepository};
use core_lib::test_support::FakeTaskRepository;
use discord_task::events::{GatewayEventHandler, ThreadCreateHandler};
use twilight_gateway::Event;
use twilight_model::channel::Channel;
use twilight_model::gateway::payload::incoming::ThreadCreate;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

/// Build a minimal `ThreadCreate` gateway event from JSON.
fn make_thread_create_event(guild_id: u64, thread_id: u64, name: &str) -> Event {
    let json = serde_json::json!({
        "id": thread_id.to_string(),
        "type": 11,   // PublicThread
        "guild_id": guild_id.to_string(),
        "name": name
    });
    let channel: Channel = serde_json::from_value(json).expect("valid Channel JSON");
    Event::ThreadCreate(Box::new(ThreadCreate(channel)))
}

#[tokio::test]
async fn thread_create_registers_new_task() {
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(now());
    let handler = ThreadCreateHandler::new(repo, clock);

    let event = make_thread_create_event(4001, 5001, "My task thread");
    handler.handle(&event).await.unwrap();

    // Verify via find_by_thread: the use_case's repo must have the task.
    let task = handler
        .use_case
        .repo
        .find_by_thread(ThreadId(5001))
        .await
        .unwrap();
    assert!(task.is_some(), "task must be registered after ThreadCreate");
    assert_eq!(task.unwrap().title, "My task thread");
}

#[tokio::test]
async fn thread_create_is_idempotent() {
    // Firing the same ThreadCreate event twice must not create a second task.
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(now());
    let handler = ThreadCreateHandler::new(repo, clock);

    let event = make_thread_create_event(4001, 5001, "Idempotent");
    handler.handle(&event).await.unwrap();
    handler.handle(&event).await.unwrap();

    let all_tasks = handler.use_case.repo.snapshot();
    assert_eq!(
        all_tasks.len(),
        1,
        "duplicate ThreadCreate must not add a second task"
    );
}

#[tokio::test]
async fn thread_create_ignores_non_thread_events() {
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(now());
    let handler = ThreadCreateHandler::new(repo, clock);

    // ThreadDelete is not a ThreadCreate — must be silently ignored.
    let td_json = serde_json::json!({
        "guild_id": "4001",
        "id": "5001",
        "type": 11,
        "parent_id": "3001"
    });
    let td: twilight_model::gateway::payload::incoming::ThreadDelete =
        serde_json::from_value(td_json).expect("valid ThreadDelete JSON");
    let event = Event::ThreadDelete(td);

    handler.handle(&event).await.unwrap(); // Must not error.
    assert!(
        handler.use_case.repo.snapshot().is_empty(),
        "no task must be created for a ThreadDelete event"
    );
}

#[tokio::test]
async fn thread_create_skips_event_without_guild_id() {
    let repo = FakeTaskRepository::new();
    let clock = FakeClock::new(now());
    let handler = ThreadCreateHandler::new(repo, clock);

    // A thread without a guild_id (DM-style) must be skipped.
    let json = serde_json::json!({
        "id": "5001",
        "type": 11,
        "name": "no-guild"
        // no guild_id field
    });
    let channel: Channel = serde_json::from_value(json).expect("valid Channel JSON");
    let event = Event::ThreadCreate(Box::new(ThreadCreate(channel)));

    handler.handle(&event).await.unwrap();
    assert!(
        handler.use_case.repo.snapshot().is_empty(),
        "event without guild_id must be skipped"
    );
}
